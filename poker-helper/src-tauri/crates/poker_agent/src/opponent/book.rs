use crate::opponent::identity::PlayerId;
use crate::opponent::population::Population;
use crate::opponent::profile::PlayerProfile;
use crate::opponent::stats::PlayerStats;
use crate::table::{SeatIndex, SeatStatus, TableState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The stats of every player seen so far and the population they are
/// compared with. Serializes to JSON so it can be saved between sessions.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OpponentBook {
    population: Population,
    players: HashMap<PlayerId, PlayerStats>,
}

impl OpponentBook {
    pub fn new(population: Population) -> OpponentBook {
        OpponentBook {
            population,
            players: HashMap::new(),
        }
    }

    pub fn population(&self) -> Population {
        self.population
    }

    pub fn set_population(&mut self, population: Population) {
        self.population = population;
    }

    pub fn stats(&self, player: &PlayerId) -> PlayerStats {
        self.players.get(player).copied().unwrap_or_default()
    }

    pub fn profile(&self, player: &PlayerId) -> PlayerProfile {
        PlayerProfile::estimate(&self.stats(player), self.population)
    }

    /// Adds a finished hand, given its last frame and who sat in each seat
    /// (`players[i]` sits in seat `i`). Call it once per hand.
    ///
    /// A player known only by seat is forgotten when that seat is empty,
    /// because whoever sits there next is someone else.
    pub fn record_hand(&mut self, table: &TableState, players: &[PlayerId]) {
        for (index, player) in players.iter().enumerate() {
            let seat = SeatIndex(index as u8);
            let Some(status) = table.seat(seat).map(|seat| seat.status) else {
                continue;
            };
            if status == SeatStatus::Empty && player.is_seat() {
                self.players.remove(player);
                continue;
            }
            if status.is_dealt_in() {
                self.players
                    .entry(player.clone())
                    .or_default()
                    .record_hand(table, seat);
            }
        }
    }

    /// The profile of each player in `players`, indexed by seat.
    pub fn reads(&self, players: &[PlayerId]) -> TableReads {
        TableReads {
            by_seat: players
                .iter()
                .map(|player| Some(self.profile(player)))
                .collect(),
        }
    }
}

/// What is known about the player in each seat. A seat without a profile is
/// played against the plain ranges and fold rates of the strategy engine.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TableReads {
    by_seat: Vec<Option<PlayerProfile>>,
}

impl TableReads {
    /// No reads on anyone.
    pub fn none() -> TableReads {
        TableReads::default()
    }

    pub fn from_profiles(by_seat: Vec<Option<PlayerProfile>>) -> TableReads {
        TableReads { by_seat }
    }

    pub fn profile(&self, seat: SeatIndex) -> Option<&PlayerProfile> {
        self.by_seat.get(seat.index()).and_then(Option::as_ref)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::state::tests::six_max_preflop;
    use crate::table::{Chips, PlayerAction, RecordedAction, Street};

    fn seat_players() -> Vec<PlayerId> {
        (0..6)
            .map(|index| PlayerId::Seat(SeatIndex(index)))
            .collect()
    }

    fn hand_where_seat_three_limps() -> TableState {
        let mut setup = six_max_preflop(0, "Ah Kd");
        setup.history = vec![RecordedAction {
            seat: SeatIndex(3),
            street: Street::Preflop,
            action: PlayerAction::Call { to: Chips(2) },
        }];
        TableState::new(setup).unwrap()
    }

    #[test]
    fn records_each_dealt_in_player_once_per_hand() {
        let mut book = OpponentBook::default();
        let players = seat_players();
        book.record_hand(&hand_where_seat_three_limps(), &players);
        book.record_hand(&hand_where_seat_three_limps(), &players);
        let limper = book.stats(&players[3]);
        assert_eq!(limper.hands(), 2);
        assert_eq!(limper.voluntarily_played.hits, 2);
        assert_eq!(book.stats(&players[4]).voluntarily_played.hits, 0);
    }

    #[test]
    fn forgets_a_seat_player_when_the_seat_empties_but_keeps_named_players() {
        let mut book = OpponentBook::default();
        let mut players = seat_players();
        players[4] = PlayerId::Named("Ana".to_string());
        book.record_hand(&hand_where_seat_three_limps(), &players);

        let mut setup = six_max_preflop(0, "Ah Kd");
        setup.seats[3].status = SeatStatus::Empty;
        setup.seats[4].status = SeatStatus::Empty;
        book.record_hand(&TableState::new(setup).unwrap(), &players);
        assert_eq!(book.stats(&players[3]).hands(), 0);
        assert_eq!(book.stats(&players[4]).hands(), 1);
    }

    #[test]
    fn reads_give_every_seat_a_profile() {
        let book = OpponentBook::new(Population::Regular);
        let reads = book.reads(&seat_players());
        assert!(reads.profile(SeatIndex(5)).is_some());
        assert!(reads.profile(SeatIndex(6)).is_none());
        assert!(TableReads::none().profile(SeatIndex(0)).is_none());
    }

    #[test]
    fn survives_a_json_round_trip() {
        let mut book = OpponentBook::new(Population::Regular);
        let mut players = seat_players();
        players[3] = PlayerId::Named("Bea".to_string());
        book.record_hand(&hand_where_seat_three_limps(), &players);
        let json = serde_json::to_string(&book).unwrap();
        assert!(json.contains("name:Bea"));
        let restored: OpponentBook = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, book);
    }
}
