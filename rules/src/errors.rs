use crate::rules::MatchBounds;
use failure::Fail;
use nom::types::CompleteStr;

pub(crate) const UNKNOWN: u32 = 1; // couldn't recognize token

#[derive(Debug, Fail, PartialEq, Clone)]
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

#[derive(Debug, PartialEq)]
pub struct SemanticError<'a> {
    // meta info for parser
    bounds: MatchBounds,
    tail: &'a str,

    // user visible error info
    error: DateTimeError,
}

impl<'a> SemanticError<'a> {
    pub fn extract_error(self) -> DateTimeError {
        self.error
    }
    pub fn set_tail(&mut self, tail: CompleteStr<'a>) {
        self.tail = *tail;
    }
}

pub fn ambiguous_time_error(msg: &str, bounds: MatchBounds) -> SemanticError {
    SemanticError {
        bounds,
        tail: "",
        error: DateTimeError::AmbiguousTime {
            msg: msg.to_owned(),
        },
    }
}

pub fn invalid_time_error<'a>(
    msg: &'a str,
    what: &'a str,
    value: i32,
    bounds: MatchBounds,
) -> SemanticError<'a> {
    SemanticError {
        bounds,
        tail: "",
        error: DateTimeError::InvalidTime {
            msg: msg.to_owned(),
            what: what.to_owned(),
            value,
        },
    }
}
