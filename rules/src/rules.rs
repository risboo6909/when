use chrono::prelude::DateTime;
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
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub day: Option<i32>,
    pub hour: Option<i32>,
    pub minute: Option<i32>,
}

impl Context {
    pub fn set_duration<T>(&mut self, duration: T)
    where
        i64: From<T>,
    {
        self.duration = time::Duration::seconds(i64::from(duration));
    }

    pub fn update(&mut self, other: &Self) {
        self.duration = self.duration + other.duration;
        if other.year.is_some() {
            self.year = other.year;
        }
        if other.month.is_some() {
            self.month = other.month;
        }
        if other.day.is_some() {
            self.day = other.day;
        }
        if other.hour.is_some() {
            self.hour = other.hour;
        }
        if other.minute.is_some() {
            self.minute = other.minute;
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Context {
            duration: time::Duration::zero(),
            year: None,
            month: None,
            day: None,
            hour: None,
            minute: None,
        }
    }
}

#[derive(Debug)]
pub struct RuleResult<'a> {
    pub tail: &'a str,
    pub tokens: Option<Vec<PToken>>,
    pub bounds: Option<MatchBounds>,

    pub context: Context,
}

pub(crate) type FnRule<Tz> =
    for<'r> fn(&'r str, bool, DateTime<Tz>) -> Result<RuleResult<'r>, DateTimeError>;

impl<'a> RuleResult<'a> {
    pub fn new() -> Self {
        Self {
            tail: "",
            tokens: None,
            bounds: None,
            context: Default::default(),
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
        None
    }

    pub fn set_tail(&mut self, tail: &'a str) {
        self.tail = tail;
    }

    pub fn set_bounds(&mut self, bounds: Option<MatchBounds>) {
        self.bounds = bounds;
    }

    pub fn set_context(&mut self, ctx: Context) {
        self.context = ctx;
    }

    pub fn get_duration_sec(&self) -> i64 {
        self.context.duration.num_seconds()
    }

    pub fn get_minutes(&self) -> i32 {
        self.context.minute.unwrap_or(0)
    }

    pub fn get_hours(&self) -> i32 {
        self.context.hour.unwrap_or(0)
    }

    pub fn get_day(&self) -> i32 {
        self.context.day.unwrap_or(0)
    }

    pub fn get_month(&self) -> i32 {
        self.context.month.unwrap_or(0)
    }

    pub fn get_year(&self) -> i32 {
        self.context.year.unwrap_or(0)
    }
}

#[derive(Debug)]
pub struct MatchResult {
    pub bounds: MatchBounds,
    pub time_shift: Context,
}

impl MatchResult {
    pub fn new(time_shift: Context, start_idx: usize, end_idx: usize) -> Self {
        Self {
            bounds: MatchBounds::new(start_idx, end_idx),
            time_shift,
        }
    }
}
