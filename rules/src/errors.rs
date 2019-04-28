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

#[derive(Debug)]
pub struct SemanticError<'a> {
    // meta info for parser
    bounds: MatchBounds,
    tail: &'a str,

    // user visible error info
    error: DateTimeError,
}

impl<'a> SemanticError<'a> {
    pub fn extract_error(&self) -> DateTimeError {
        self.error.clone()
    }
    pub fn set_tail(&mut self, tail: CompleteStr<'a>) {
        self.tail = *tail;
    }
    pub fn get_tail(&self) -> &'a str {
        self.tail
    }
    pub fn set_bounds(&mut self, bounds: MatchBounds) {
        self.bounds = bounds;
    }
    pub fn get_bounds(&self) -> MatchBounds {
        self.bounds
    }
}

pub fn ambiguous_time_error(msg: &str) -> SemanticError {
    SemanticError {
        bounds: MatchBounds::new(0, 0),
        tail: "",
        error: DateTimeError::AmbiguousTime {
            msg: msg.to_owned(),
        },
    }
}

pub fn invalid_time_error<'a>(msg: &'a str, what: &'a str, value: i32) -> SemanticError<'a> {
    SemanticError {
        bounds: MatchBounds::new(0, 0),
        tail: "",
        error: DateTimeError::InvalidTime {
            msg: msg.to_owned(),
            what: what.to_owned(),
            value,
        },
    }
}
