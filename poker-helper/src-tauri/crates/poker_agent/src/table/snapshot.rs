use crate::core::card::{Card, ParseCardError};
use crate::opponent::PlayerId;
use crate::table::chips::Chips;
use crate::table::seat::{Seat, SeatIndex};
use crate::table::state::{Blinds, TableSetup};
use serde::Deserialize;
use std::fmt;

/// Version of the table snapshot that the vision sidecar sends.
pub const SNAPSHOT_VERSION: u32 = 2;

/// One frame of the table as read from the screen (contract version 2).
///
/// ```json
/// {
///   "version": 2,
///   "button_seat": 0,
///   "hero_seat": 3,
///   "blinds": { "small": 1, "big": 2, "ante": 0 },
///   "seats": [
///     { "stack": 200, "committed_this_street": 0, "status": "active", "name": "Ana" },
///     { "stack": 199, "committed_this_street": 1, "status": "active" },
///     { "stack": 198, "committed_this_street": 2, "status": "active" },
///     { "stack": 200, "committed_this_street": 0, "status": "active" },
///     { "stack": 0, "committed_this_street": 0, "status": "empty" },
///     { "stack": 194, "committed_this_street": 6, "status": "active" }
///   ],
///   "hero_cards": ["Ah", "Kd"],
///   "board": [],
///   "pot": 0
/// }
/// ```
///
/// `pot` holds the chips from earlier streets and antes only; chips in front
/// of the players this street go in `committed_this_street`. `name` is the
/// optional screen name of the player in the seat.
#[derive(Clone, Debug, Deserialize)]
pub struct TableSnapshot {
    pub version: u32,
    pub button_seat: SeatIndex,
    pub hero_seat: SeatIndex,
    pub blinds: Blinds,
    pub seats: Vec<SnapshotSeat>,
    pub hero_cards: [String; 2],
    #[serde(default)]
    pub board: Vec<String>,
    pub pot: Chips,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SnapshotSeat {
    #[serde(flatten)]
    pub seat: Seat,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SnapshotError {
    Json(String),
    UnsupportedVersion(u32),
    Card(ParseCardError),
}

impl fmt::Display for SnapshotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SnapshotError::Json(message) => write!(formatter, "invalid table snapshot: {message}"),
            SnapshotError::UnsupportedVersion(version) => write!(
                formatter,
                "table snapshot version {version} is not supported, expected {SNAPSHOT_VERSION}"
            ),
            SnapshotError::Card(error) => write!(formatter, "invalid table snapshot: {error}"),
        }
    }
}

impl std::error::Error for SnapshotError {}

impl From<ParseCardError> for SnapshotError {
    fn from(error: ParseCardError) -> SnapshotError {
        SnapshotError::Card(error)
    }
}

impl TableSnapshot {
    pub fn from_json(text: &str) -> Result<TableSnapshot, SnapshotError> {
        let snapshot: TableSnapshot =
            serde_json::from_str(text).map_err(|error| SnapshotError::Json(error.to_string()))?;
        snapshot.ensure_supported_version()?;
        Ok(snapshot)
    }

    /// Fails unless the snapshot uses contract version [`SNAPSHOT_VERSION`].
    /// [`from_json`](TableSnapshot::from_json) already checks it.
    pub fn ensure_supported_version(&self) -> Result<(), SnapshotError> {
        if self.version != SNAPSHOT_VERSION {
            return Err(SnapshotError::UnsupportedVersion(self.version));
        }
        Ok(())
    }

    /// Who sits in each seat, by screen name when the frame has one.
    pub fn player_ids(&self) -> Vec<PlayerId> {
        self.seats
            .iter()
            .enumerate()
            .map(|(index, seat)| PlayerId::for_seat(SeatIndex(index as u8), seat.name.as_deref()))
            .collect()
    }

    /// Converts the frame into a table without action history. Feed it to a
    /// [`HandTracker`](crate::table::HandTracker) to rebuild the history.
    pub fn into_setup(self) -> Result<TableSetup, SnapshotError> {
        let hero_cards = [self.hero_cards[0].parse()?, self.hero_cards[1].parse()?];
        let board = self
            .board
            .iter()
            .map(|card| card.parse())
            .collect::<Result<Vec<Card>, _>>()?;
        Ok(TableSetup {
            seats: self.seats.into_iter().map(|seat| seat.seat).collect(),
            button: self.button_seat,
            hero: self.hero_seat,
            hero_cards,
            board,
            blinds: self.blinds,
            pot_from_previous_streets: self.pot,
            history: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::seat::SeatStatus;
    use crate::table::state::TableState;

    const EXAMPLE: &str = r#"{
        "version": 2,
        "button_seat": 0,
        "hero_seat": 3,
        "blinds": { "small": 1, "big": 2 },
        "seats": [
            { "stack": 200, "committed_this_street": 0, "status": "active", "name": "Ana" },
            { "stack": 199, "committed_this_street": 1, "status": "active" },
            { "stack": 198, "committed_this_street": 2, "status": "active" },
            { "stack": 200, "committed_this_street": 0, "status": "active" },
            { "stack": 0, "committed_this_street": 0, "status": "empty" },
            { "stack": 194, "committed_this_street": 6, "status": "active" }
        ],
        "hero_cards": ["Ah", "Kd"],
        "board": [],
        "pot": 0
    }"#;

    #[test]
    fn parses_the_documented_example() {
        let setup = TableSnapshot::from_json(EXAMPLE)
            .unwrap()
            .into_setup()
            .unwrap();
        assert_eq!(setup.seats[4].status, SeatStatus::Empty);
        assert_eq!(setup.blinds.ante, Chips::ZERO);
        let table = TableState::new(setup).unwrap();
        assert_eq!(table.dealt_in_player_count(), 5);
        assert_eq!(table.amount_to_call(SeatIndex(3)), Chips(6));
    }

    #[test]
    fn names_players_by_screen_name_or_seat() {
        let snapshot = TableSnapshot::from_json(EXAMPLE).unwrap();
        let players = snapshot.player_ids();
        assert_eq!(players[0], PlayerId::Named("Ana".to_string()));
        assert_eq!(players[1], PlayerId::Seat(SeatIndex(1)));
    }

    #[test]
    fn rejects_other_versions_and_bad_cards() {
        let old = EXAMPLE.replace("\"version\": 2", "\"version\": 1");
        assert_eq!(
            TableSnapshot::from_json(&old).unwrap_err(),
            SnapshotError::UnsupportedVersion(1)
        );

        let bad_card = EXAMPLE.replace("\"Kd\"", "\"Kx\"");
        let error = TableSnapshot::from_json(&bad_card)
            .unwrap()
            .into_setup()
            .unwrap_err();
        assert!(matches!(error, SnapshotError::Card(_)));

        assert!(matches!(
            TableSnapshot::from_json("{}").unwrap_err(),
            SnapshotError::Json(_)
        ));
    }
}
