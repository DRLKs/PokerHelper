use crate::advisor::{advise_with_reads, Advice, AdviceError, ConfidenceIssue};
use crate::opponent::{OpponentBook, PlayerId, PlayerProfile};
use crate::table::{
    HandTracker, RecordedAction, SeatIndex, SnapshotError, Street, TableError, TableSnapshot,
};
use serde::Serialize;
use std::fmt;

/// Everything that lasts across frames while the user plays: the hand being
/// tracked, who sits where, and the stats of every opponent seen.
#[derive(Debug, Default)]
pub struct AdvisorSession {
    tracker: HandTracker,
    book: OpponentBook,
    players: Vec<PlayerId>,
}

/// What the session knows after one frame.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TableUpdate {
    /// `None` when the hero has no decision to make or the advice failed.
    pub advice: Option<Advice>,
    /// Why the advice failed, when it did.
    pub advice_error: Option<String>,
    pub street: Street,
    pub history: Vec<RecordedAction>,
    /// The profile of the player in every occupied seat but the hero's.
    pub opponents: Vec<SeatRead>,
    /// Whether this frame started a new hand and the previous one was
    /// added to the opponent stats.
    pub recorded_a_finished_hand: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SeatRead {
    pub seat: SeatIndex,
    pub player: PlayerId,
    pub profile: PlayerProfile,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionError {
    Snapshot(SnapshotError),
    Table(TableError),
}

impl fmt::Display for SessionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SessionError::Snapshot(error) => write!(formatter, "{error}"),
            SessionError::Table(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for SessionError {}

impl From<SnapshotError> for SessionError {
    fn from(error: SnapshotError) -> SessionError {
        SessionError::Snapshot(error)
    }
}

impl From<TableError> for SessionError {
    fn from(error: TableError) -> SessionError {
        SessionError::Table(error)
    }
}

impl AdvisorSession {
    pub fn new(book: OpponentBook) -> AdvisorSession {
        AdvisorSession {
            book,
            ..AdvisorSession::default()
        }
    }

    pub fn book(&self) -> &OpponentBook {
        &self.book
    }

    pub fn book_mut(&mut self) -> &mut OpponentBook {
        &mut self.book
    }

    /// Reads one frame of the table, records the previous hand when this one
    /// is new, and advises the hero when the hero can act.
    ///
    /// # Errors
    ///
    /// Fails when the snapshot has another contract version, holds an
    /// invalid card, or describes an impossible table. The session keeps its
    /// state from before the frame.
    pub fn observe(&mut self, snapshot: TableSnapshot) -> Result<TableUpdate, SessionError> {
        snapshot.ensure_supported_version()?;
        let players = snapshot.player_ids();
        let table = self.tracker.observe(snapshot.into_setup()?)?.clone();

        let finished_hand = self.tracker.take_finished_hand();
        let recorded_a_finished_hand = finished_hand.is_some();
        if let Some(finished) = finished_hand {
            self.book.record_hand(&finished, &self.players);
        }
        self.players = players;

        let reads = self.book.reads(&self.players);
        let (advice, advice_error) = match advise_with_reads(&table, &reads) {
            Ok(mut advice) => {
                if !self.tracker.chips_add_up() {
                    advice.confidence.add(ConfidenceIssue::ChipsDoNotAddUp);
                }
                (Some(advice), None)
            }
            Err(AdviceError::HeroCannotAct) => (None, None),
            Err(error) => (None, Some(error.to_string())),
        };
        let opponents = self
            .players
            .iter()
            .enumerate()
            .map(|(index, player)| (SeatIndex(index as u8), player))
            .filter(|&(seat, _)| {
                seat != table.hero()
                    && table
                        .seat(seat)
                        .is_some_and(|occupant| occupant.status.is_dealt_in())
            })
            .map(|(seat, player)| SeatRead {
                seat,
                player: player.clone(),
                profile: self.book.profile(player),
            })
            .collect();
        Ok(TableUpdate {
            advice,
            advice_error,
            street: table.street(),
            history: table.history().to_vec(),
            opponents,
            recorded_a_finished_hand,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::advisor::{ConfidenceLevel, Recommendation};

    fn frame(hero_cards: [&str; 2], seat_three: &str) -> TableSnapshot {
        let json = format!(
            r#"{{
                "version": 2,
                "button_seat": 0,
                "hero_seat": 0,
                "blinds": {{ "small": 1, "big": 2 }},
                "seats": [
                    {{ "stack": 200, "committed_this_street": 0, "status": "active" }},
                    {{ "stack": 199, "committed_this_street": 1, "status": "active", "name": "Ana" }},
                    {{ "stack": 198, "committed_this_street": 2, "status": "active" }},
                    {seat_three},
                    {{ "stack": 0, "committed_this_street": 0, "status": "empty" }},
                    {{ "stack": 200, "committed_this_street": 0, "status": "folded" }}
                ],
                "hero_cards": ["{}", "{}"],
                "pot": 0
            }}"#,
            hero_cards[0], hero_cards[1]
        );
        TableSnapshot::from_json(&json).unwrap()
    }

    const LIMPER: &str = r#"{ "stack": 198, "committed_this_street": 2, "status": "active" }"#;

    #[test]
    fn advises_the_hero_and_reads_every_dealt_in_opponent() {
        let mut session = AdvisorSession::default();
        let update = session.observe(frame(["Ah", "Ad"], LIMPER)).unwrap();
        let advice = update.advice.unwrap();
        assert!(matches!(
            advice.recommendation,
            Recommendation::RaiseTo { .. }
        ));
        assert_eq!(advice.confidence.level, ConfidenceLevel::Medium);
        let seats: Vec<u8> = update.opponents.iter().map(|read| read.seat.0).collect();
        assert_eq!(seats, [1, 2, 3, 5]);
        assert_eq!(
            update.opponents[0].player,
            PlayerId::Named("Ana".to_string())
        );
        assert!(!update.recorded_a_finished_hand);
    }

    #[test]
    fn records_the_previous_hand_when_a_new_one_starts() {
        let mut session = AdvisorSession::default();
        session.observe(frame(["Ah", "Ad"], LIMPER)).unwrap();
        let update = session.observe(frame(["7c", "2d"], LIMPER)).unwrap();
        assert!(update.recorded_a_finished_hand);
        let limper = session.book().stats(&PlayerId::Seat(SeatIndex(3)));
        assert_eq!(limper.hands(), 1);
        assert_eq!(limper.voluntarily_played.hits, 1);
        assert_eq!(
            session
                .book()
                .stats(&PlayerId::Named("Ana".to_string()))
                .hands(),
            1
        );
    }

    #[test]
    fn lowers_confidence_when_the_chips_do_not_add_up() {
        let mut session = AdvisorSession::default();
        session.observe(frame(["Ah", "Ad"], LIMPER)).unwrap();
        let misread = r#"{ "stack": 150, "committed_this_street": 2, "status": "active" }"#;
        let update = session.observe(frame(["Ah", "Ad"], misread)).unwrap();
        let confidence = update.advice.unwrap().confidence;
        assert_eq!(confidence.level, ConfidenceLevel::Low);
        assert!(confidence
            .issues
            .contains(&ConfidenceIssue::ChipsDoNotAddUp));
    }

    #[test]
    fn rejects_an_impossible_table_and_keeps_going() {
        let mut session = AdvisorSession::default();
        session.observe(frame(["Ah", "Ad"], LIMPER)).unwrap();
        assert!(session.observe(frame(["Ah", "Ah"], LIMPER)).is_err());
        assert!(session.observe(frame(["Ah", "Ad"], LIMPER)).is_ok());
    }
}
