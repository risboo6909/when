use std::fmt::Debug;

use crate::tokens::Tokens;
use nom::{types::CompleteStr, IResult};

pub type MyResult<'a> = IResult<CompleteStr<'a>, MatchResult>;

#[derive(Debug)]
pub struct MatchResult {
    pub token: Tokens,
    pub dist: usize,
}

impl MatchResult {
    pub(crate) fn new(token: Tokens, dist: usize) -> Self {
        MatchResult { token, dist }
    }
}

#[derive(Debug)]
pub struct RuleResult<'a> {
    pub tail: &'a str,
    pub tokens: Option<Vec<Tokens>>,
}

impl<'a> RuleResult<'a> {
    pub fn new(tail: &'a str, tokens: Vec<MatchResult>) -> Self {
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
            };
        }

        Self { tail, tokens: None }
    }
}
