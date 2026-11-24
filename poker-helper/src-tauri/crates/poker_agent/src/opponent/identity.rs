use crate::table::SeatIndex;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

const NAME_PREFIX: &str = "name:";
const SEAT_PREFIX: &str = "seat:";

/// Who sits in a seat: the screen name when the table shows one, otherwise
/// the seat itself. Serialized as `name:<screen name>` or `seat:<index>`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub enum PlayerId {
    Named(String),
    Seat(SeatIndex),
}

impl PlayerId {
    /// The player in `seat`, known by `name` unless it is missing or blank.
    pub fn for_seat(seat: SeatIndex, name: Option<&str>) -> PlayerId {
        match name.map(str::trim) {
            Some(name) if !name.is_empty() => PlayerId::Named(name.to_string()),
            Some(_) | None => PlayerId::Seat(seat),
        }
    }

    pub fn is_seat(&self) -> bool {
        match self {
            PlayerId::Seat(_) => true,
            PlayerId::Named(_) => false,
        }
    }
}

impl fmt::Display for PlayerId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlayerId::Named(name) => write!(formatter, "{NAME_PREFIX}{name}"),
            PlayerId::Seat(seat) => write!(formatter, "{SEAT_PREFIX}{}", seat.0),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsePlayerIdError(String);

impl fmt::Display for ParsePlayerIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid player id {:?}: expected {NAME_PREFIX}<name> or {SEAT_PREFIX}<index>",
            self.0
        )
    }
}

impl std::error::Error for ParsePlayerIdError {}

impl FromStr for PlayerId {
    type Err = ParsePlayerIdError;

    fn from_str(text: &str) -> Result<PlayerId, ParsePlayerIdError> {
        let invalid = || ParsePlayerIdError(text.to_string());
        if let Some(name) = text.strip_prefix(NAME_PREFIX) {
            return Ok(PlayerId::Named(name.to_string()));
        }
        let index = text.strip_prefix(SEAT_PREFIX).ok_or_else(invalid)?;
        let index = index.parse().map_err(|_| invalid())?;
        Ok(PlayerId::Seat(SeatIndex(index)))
    }
}

impl From<PlayerId> for String {
    fn from(player: PlayerId) -> String {
        player.to_string()
    }
}

impl TryFrom<String> for PlayerId {
    type Error = ParsePlayerIdError;

    fn try_from(text: String) -> Result<PlayerId, ParsePlayerIdError> {
        text.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_blank_name_falls_back_to_the_seat() {
        assert_eq!(
            PlayerId::for_seat(SeatIndex(2), Some("  ")),
            PlayerId::Seat(SeatIndex(2))
        );
        assert_eq!(
            PlayerId::for_seat(SeatIndex(2), Some(" Ana ")),
            PlayerId::Named("Ana".to_string())
        );
        assert_eq!(
            PlayerId::for_seat(SeatIndex(4), None),
            PlayerId::Seat(SeatIndex(4))
        );
    }

    #[test]
    fn round_trips_through_text() {
        for player in [
            PlayerId::Named("seat:3 fan".to_string()),
            PlayerId::Seat(SeatIndex(5)),
        ] {
            assert_eq!(player.to_string().parse::<PlayerId>().unwrap(), player);
        }
        assert!("Ana".parse::<PlayerId>().is_err());
        assert!("seat:x".parse::<PlayerId>().is_err());
    }
}
