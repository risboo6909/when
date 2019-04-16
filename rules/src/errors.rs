use crate::errors::DateTimeError::InvalidTime;
use failure::Fail;

pub(crate) const UNKNOWN: u32 = 1; // couldn't recognize token

#[derive(Debug, Fail, PartialEq)]
pub enum DateTimeError {
    #[fail(display = "can't parse time unambiguously in: {}", msg)]
    AmbiguousTime { msg: String },
    #[fail(display = "invalid time in: {}, {} can't be {}", msg, what, value)]
    InvalidTime {
        msg: String,
        what: String,
        value: i32,
    },
}

impl DateTimeError {
    pub fn invalid_time_error(msg: &str, what: &str, value: i32) -> DateTimeError {
        InvalidTime {
            msg: msg.to_owned(),
            what: what.to_owned(),
            value,
        }
    }
}
