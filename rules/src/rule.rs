use std::fmt::Debug;

use crate::tokens::Tokens;
use nom::{IResult, types::CompleteStr};

pub type MyResult<'a> = IResult<CompleteStr<'a>, MatchResult>;

#[derive(Debug)]
pub struct MatchResult {
    pub token: Tokens,
    pub dist: usize,
}

impl MatchResult {
    pub(crate) fn new(token: Tokens, dist: usize) -> Self {
        MatchResult{token, dist}
    }
}

#[derive(Debug)]
pub struct ApplyResult<'a> {
    pub tail: &'a str,
    pub tokens: Vec<Tokens>,
}

impl<'a> ApplyResult<'a> {

    pub fn new(tail: &'a str, tokens: Vec<MatchResult>) -> Self {

        // remove stub tokens
        let filtered_tokens =
            tokens.iter()
                  .map(|item| item.token.clone())
                  .filter(|item|
                    match item {
                        Tokens::Stub => false,
                        _ => true,
                    })
                  .collect();

        Self { tail, tokens: filtered_tokens }

    }

}
