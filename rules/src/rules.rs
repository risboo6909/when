use std::fmt::Debug;

use nom::{types::CompleteStr, IResult};
use chrono::prelude::{DateTime, Local};

use crate::tokens::{Token, PToken};
use crate::errors::DateTimeError;

pub type MyResult<'a> = IResult<CompleteStr<'a>, TokenDesc>;

#[derive(Debug, Clone)]
pub struct TokenDesc {
    pub token: PToken,
    pub dist: usize,
}

impl TokenDesc {
    pub(crate) fn new(token: PToken, dist: usize) -> Self {
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
    pub hours: usize,
}

#[derive(Debug)]
pub struct RuleResult<'a> {

    pub tail: &'a str,
    pub tokens: Option<Vec<Token>>,
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

        // sort tokens by priority, the smaller value - the bigger priority
        filtered_tokens.sort_by(|a, b| {

            let p1 = match *a {
                PToken::PToken(_, p) => p,
                _ => 0,
            };

            let p2 = match *b {
                PToken::PToken(_, p) => p,
                _ => 0,
            };

            p1.cmp(&p2)

        });

        // we don't need tokens priorities anymore, so unwrap them and get rid of PToken
        let mut tokens = Vec::new();
        for item in filtered_tokens {
            match item {
                PToken::PToken(token, _) => tokens.push(token),
                _ => unreachable!(),
            }
        }

        if tokens.len() > 0 {
            self.tokens = Some(tokens);
        }

        self

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
            Ok(x) => x.hours,
            Err(_) => 0,
        }
    }

}

#[derive(Debug)]
pub struct MatchResult {
    pub bounds: MatchBounds,
    pub tokens: Vec<Token>,

    pub time_shift: Result<TimeShift, DateTimeError>,
}

impl MatchResult {
    pub fn new(tokens: Vec<Token>, time_shift:  Result<TimeShift, DateTimeError>, start_idx: usize,
               end_idx: usize) -> Self {
        Self {
            bounds: MatchBounds::new(start_idx, end_idx),
            tokens,
            time_shift,
        }
    }
}
