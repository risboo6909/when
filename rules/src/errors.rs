use crate::rules::MatchBounds;
use chrono::DateTime;
use failure::Fail;

pub(crate) const UNKNOWN: u32 = 1; // couldn't recognize token

#[derive(Debug, Fail, PartialEq, Clone)]
pub enum DateTimeError {
    #[fail(display = "can't parse time unambiguously in: {}", msg)]
    AmbiguousTime { msg: String, bounds: MatchBounds },
    #[fail(display = "invalid time in: {}, {} can't be {}", msg, what, value)]
    InvalidTime {
        msg: String,
        what: String,
        value: i32,
        bounds: MatchBounds,
    },
}

impl DateTimeError {
    pub fn ambiguous_time_error(msg: &str, bounds: MatchBounds) -> DateTimeError {
        DateTimeError::AmbiguousTime {
            msg: msg.to_owned(),
            bounds,
        }
    }

    pub fn invalid_time_error(
        msg: &str,
        what: &str,
        value: i32,
        bounds: MatchBounds,
    ) -> DateTimeError {
        DateTimeError::InvalidTime {
            msg: msg.to_owned(),
            what: what.to_owned(),
            value,
            bounds,
        }
    }
}
