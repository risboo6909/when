use std::fmt::Debug;

use crate::tokens::Tokens;
use nom::{types::CompleteStr, IResult};

pub type MyResult<'a> = IResult<CompleteStr<'a>, TokenDesc>;

#[derive(Debug)]
pub struct TokenDesc {
    pub token: Tokens,
    pub dist: usize,
}

impl TokenDesc {
    pub(crate) fn new(token: Tokens, dist: usize) -> Self {
        Self { token, dist }
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct RuleResult<'a> {
    pub tail: &'a str,
    pub tokens: Option<Vec<Tokens>>,
    pub bounds: Option<MatchBounds>,
}

pub(crate) type FnRule = for<'r> fn(&'r str, bool) -> RuleResult<'r>;

impl<'a> RuleResult<'a> {
    pub fn new(tail: &'a str, tokens: Vec<TokenDesc>, bounds: Option<MatchBounds>) -> Self {
        // remove stub tokens
        let filtered_tokens: Vec<Tokens> = tokens
            .iter()
            .map(|item| item.token.clone())
            .filter(|item| match item {
                Tokens::Stub => false,
                _ => true,
            })
            .collect();

        if filtered_tokens.len() > 0 {
            return Self {
                tail,
                tokens: Some(filtered_tokens),
                bounds,
            };
        }

        Self { tail, tokens: None, bounds}
    }
}

#[derive(Debug)]
pub struct MatchResult {
    pub bounds: MatchBounds,
    pub tokens: Vec<Tokens>,
}

impl MatchResult {
    pub fn new(tokens: Vec<Tokens>, start_idx: usize, end_idx: usize) -> Self {
        Self {
            bounds: MatchBounds::new(start_idx, end_idx),
            tokens,
        }
    }
}