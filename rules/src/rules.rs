use chrono::prelude::{DateTime, Local};
use nom::{types::CompleteStr, IResult};
use std::convert::From;

use crate::errors::DateTimeError;
use crate::tokens::{PToken, Priority, TimeInterval, Token};
use crate::Dist;
use failure::Fail;

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
    pub(crate) fn clone_content(&self) -> PToken {
        self.token.clone()
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

#[derive(Debug, PartialEq)]
pub struct Context {
    // relative value
    pub duration: time::Duration,

    // absolute values
    pub year: i32,
    pub month: i32,
    pub hour: i32,
    pub minute: i32,
}

impl Default for Context {
    fn default() -> Self {
        Context {
            duration: time::Duration::zero(),
            year: 0,
            month: 0,
            hour: 0,
            minute: 0,
        }
    }
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

    pub fn set_token(&mut self, token_descr: &TokenDesc) -> &mut Self {
        if token_descr.token != PToken::Stub {
            if let Some(ref mut tokens) = self.tokens {
                tokens.push(token_descr.clone_content());
            } else {
                self.tokens = Some(vec![token_descr.clone_content()]);
            }
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

    pub fn set_error(&mut self, err: DateTimeError) {
        self.context = Err(err);
    }

    pub fn set_duration<T>(&mut self, duration: T)
    where
        i64: From<T>,
    {
        if self.context.is_ok() {
            self.context.as_mut().unwrap().duration = time::Duration::seconds(i64::from(duration));
        }
    }

    pub fn get_duration_sec(&self) -> i64 {
        match &self.context {
            Ok(ref ctx) => ctx.duration.num_seconds(),
            Err(_) => 0,
        }
    }

    pub fn set_minute(&mut self, minute: i32) {
        if self.context.is_ok() {
            self.context.as_mut().unwrap().minute = minute;
        }
    }

    pub fn get_minutes(&self) -> i32 {
        self.context.as_ref().map(|s| s.minute).unwrap_or(0)
    }

    pub fn set_hour(&mut self, hour: i32) {
        if self.context.is_ok() {
            self.context.as_mut().unwrap().hour = hour;
        }
    }

    pub fn get_hours(&self) -> i32 {
        self.context.as_ref().map(|s| s.hour).unwrap_or(0)
    }

    pub fn set_month(&mut self, month: i32) {
        if self.context.is_ok() {
            self.context.as_mut().unwrap().month = month;
        }
    }

    pub fn get_month(&self) -> i32 {
        self.context.as_ref().map(|s| s.month).unwrap_or(0)
    }

    pub fn set_year(&mut self, year: i32) {
        if self.context.is_ok() {
            self.context.as_mut().unwrap().year = year;
        }
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
