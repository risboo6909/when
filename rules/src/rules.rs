use nom::{types::CompleteStr, IResult};
use chrono::prelude::{DateTime, Local};

use crate::tokens::{Token, PToken, Priority};
use crate::errors::DateTimeError;
use crate::Dist;

pub type MyResult<'a> = IResult<CompleteStr<'a>, TokenDesc>;

#[derive(Debug, Clone)]
pub struct TokenDesc {
    pub token: PToken,
    pub dist: Dist,
}

impl TokenDesc {
    pub(crate) fn new(token: PToken, dist: Dist) -> Self {
        Self { token, dist }
    }
}

#[derive(Debug, PartialEq)]
pub struct MatchBounds {
    pub start_idx: usize,
    pub end_idx: usize,
}

impl MatchBounds {
    pub fn new(start_idx: usize, end_idx: usize) -> Self {
        Self {
            start_idx,
            end_idx,
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct TimeShift {
    // relative value
    pub offset: i64,

    // absolute values
    pub hour: usize,
    pub month: usize,
    pub year: usize,
}

#[derive(Debug)]
pub struct RuleResult<'a> {

    pub tail: &'a str,
    pub tokens: Option<Vec<PToken>>,
    pub bounds: Option<MatchBounds>,

    pub time_shift: Result<TimeShift, DateTimeError>,

}

pub(crate) type FnRule = for<'r> fn(&'r str, bool, DateTime<Local>) -> RuleResult<'r>;

impl<'a> RuleResult<'a> {

    pub fn new() -> Self {

        Self {
            tail: "",
            tokens: None,
            bounds: None,
            time_shift: Ok(Default::default()),
        }

    }

    pub fn set_tokens(&mut self, tokens: Vec<TokenDesc>) -> &mut Self {

        // remove stub tokens
        let mut filtered_tokens: Vec<PToken> = tokens
            .iter()
            .filter_map(|item| {
                let token = item.token.clone();
                match token {
                    PToken::Stub | PToken:: None => None,
                    _ => Some(token),
                }
            })
            .collect();

        if !filtered_tokens.is_empty() {
            self.tokens = Some(filtered_tokens);
        }

        self

    }

    pub fn filter_by_priority(&self, priority: Priority) -> Vec<&Token> {
        match &self.tokens {
            Some(tokens) => {
                tokens.iter().
                filter_map(|t| {
                    match t {
                        PToken::PToken(token, p) => {
                            if *p == priority {
                                Some(token)
                            } else {
                                None
                            }
                        },
                        _ => unreachable!(),
                    }
                }).collect()
            },
            None => Vec::new(),
        }
    }

    pub fn token_by_priority(&self, priority: Priority) -> Option<&Token> {
        let res = self.filter_by_priority(priority);
        if !res.is_empty() {
            return Some(res[0])
        }
        return None
    }

    pub fn set_tail(&mut self, tail: &'a str) -> &mut Self {
        self.tail = tail;
        self
    }

    pub fn set_bounds(&mut self, bounds: Option<MatchBounds>) -> &mut Self {
        self.bounds = bounds;
        self
    }

    pub fn get_offset(&self) -> i64 {
        match &self.time_shift {
            Ok(x) => x.offset,
            Err(_) => 0,
        }
    }

    pub fn get_hours(&self) -> usize {
        match &self.time_shift {
            Ok(x) => x.hour,
            Err(_) => 0,
        }
    }

}

#[derive(Debug)]
pub struct MatchResult {
    pub bounds: MatchBounds,
    pub time_shift: Result<TimeShift, DateTimeError>,
}

impl MatchResult {
    pub fn new(time_shift: Result<TimeShift, DateTimeError>, start_idx: usize,
               end_idx: usize) -> Self {
        Self {
            bounds: MatchBounds::new(start_idx, end_idx),
            time_shift,
        }
    }
}
