use super::rules::MatchBounds;
use serde::Serialize;
use std::fmt;

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum DateTimeError {
    AmbiguousTime {
        msg: String,
    },
    InvalidTime {
        text: String,
        what: String,
        value: i32,
    },
    IntersectionError {
        text: String,
    },
}

impl fmt::Display for DateTimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AmbiguousTime { msg } => write!(f, "can't parse time unambiguously in: {msg}"),
            Self::InvalidTime { text, what, value } => {
                write!(f, "invalid time in: {text}, {what} can't be {value}")
            }
            Self::IntersectionError { text } => {
                write!(f, "can't parse, rules intersection detected in {text}")
            }
        }
    }
}

impl std::error::Error for DateTimeError {}

#[derive(Debug, Clone, Serialize)]
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

    pub fn set_tail(&mut self, tail: &'a str) {
        self.tail = tail;
    }

    pub fn get_tail(&self) -> &'a str {
        self.tail
    }

    pub fn set_bounds(&mut self, bounds: MatchBounds) {
        self.bounds = bounds;
    }

    pub fn get_start_idx(&self) -> usize {
        self.bounds.start_idx
    }

    pub fn get_end_idx(&self) -> usize {
        self.bounds.end_idx
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
            text: msg.to_owned(),
            what: what.to_owned(),
            value,
        },
    }
}

pub fn intersection_error(text: &str) -> SemanticError {
    SemanticError {
        bounds: MatchBounds::new(0, 0),
        tail: "",
        error: DateTimeError::IntersectionError {
            text: text.to_owned(),
        },
    }
}
