use chrono::prelude::{DateTime, Local};
use nom::{types::CompleteStr, IResult};
use std::convert::From;

use crate::errors::DateTimeError;
use crate::tokens::{PToken, Priority, Token};
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
        Self { start_idx, end_idx }
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct Context {
    // relative value
    pub duration: i64,

    // absolute values
    pub year: usize,
    pub month: u32,
    pub hour: usize,
    pub minute: usize,
}

#[derive(Debug)]
pub struct RuleResult<'a> {
    pub tail: &'a str,
    pub tokens: Option<Vec<PToken>>,
    pub bounds: Option<MatchBounds>,

    pub context: Result<Context, DateTimeError>,
}

pub(crate) type FnRule = for<'r> fn(&'r str, bool, DateTime<Local>) -> RuleResult<'r>;

impl<'a> RuleResult<'a> {
    pub fn new() -> Self {
        Self {
            tail: "",
            tokens: None,
            bounds: None,
            context: Ok(Default::default()),
        }
    }

    pub fn set_tokens(&mut self, tokens: Vec<TokenDesc>) -> &mut Self {
        // remove stub tokens
        let filtered_tokens: Vec<PToken> = tokens
            .iter()
            .filter_map(|item| {
                let token = item.token.clone();
                match token {
                    PToken::Stub => None,
                    _ => Some(token),
                }
            })
            .collect();

        if !filtered_tokens.is_empty() {
            self.tokens = Some(filtered_tokens);
        }

        self
    }

    fn filter_by_priority(&self, priority: Priority) -> Vec<&Token> {
        match &self.tokens {
            Some(tokens) => tokens
                .iter()
                .filter_map(|t| match t {
                    PToken::PToken(token, p) => {
                        if *p == priority {
                            Some(token)
                        } else {
                            None
                        }
                    }
                    _ => unreachable!(),
                })
                .collect(),
            None => Vec::new(),
        }
    }

    pub fn token_by_priority(&self, priority: Priority) -> Option<Token> {
        let res = self.filter_by_priority(priority);
        if !res.is_empty() {
            return Some(res[0].clone());
        }
        return None;
    }

    pub fn set_tail(&mut self, tail: &'a str) -> &mut Self {
        self.tail = tail;
        self
    }

    pub fn set_bounds(&mut self, bounds: Option<MatchBounds>) -> &mut Self {
        self.bounds = bounds;
        self
    }

    pub fn set_duration<T>(&mut self, duration: T)
    where
        i64: From<T>,
    {
        if self.context.is_ok() {
            self.context.as_mut().unwrap().duration = i64::from(duration);
        }
    }

    pub fn set_month<T>(&mut self, month: T)
    where
        u32: From<T>,
    {
        if self.context.is_ok() {
            self.context.as_mut().unwrap().month = u32::from(month);
        }
    }

    pub fn set_year<T>(&mut self, year: T)
    where
        u32: From<T>,
    {
        if self.context.is_ok() {
            self.context.as_mut().unwrap().month = u32::from(year);
        }
    }

    pub fn get_offset(&self) -> i64 {
        self.context.as_ref().map(|s| s.duration).unwrap_or(0)
    }

    pub fn get_hours(&self) -> usize {
        self.context.as_ref().map(|s| s.hour).unwrap_or(0)
    }

    pub fn get_minutes(&self) -> usize {
        self.context.as_ref().map(|s| s.minute).unwrap_or(0)
    }

    pub fn unwrap(&self) -> &Context {
        self.context.as_ref().unwrap()
    }

    pub fn unwrap_mut(&mut self) -> &mut Context {
        self.context.as_mut().unwrap()
    }
}

#[derive(Debug)]
pub struct MatchResult {
    pub bounds: MatchBounds,
    pub time_shift: Result<Context, DateTimeError>,
}

impl MatchResult {
    pub fn new(
        time_shift: Result<Context, DateTimeError>,
        start_idx: usize,
        end_idx: usize,
    ) -> Self {
        Self {
            bounds: MatchBounds::new(start_idx, end_idx),
            time_shift,
        }
    }
}
