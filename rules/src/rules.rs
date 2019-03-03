use std::fmt::Debug;
use crate::tokens::{Token, PToken};

use nom::{types::CompleteStr, IResult};

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
    pub tokens: Option<Vec<Token>>,
    pub bounds: Option<MatchBounds>,
    pub ts: usize,
}

pub(crate) type FnRule = for<'r> fn(&'r str, bool) -> RuleResult<'r>;

impl<'a> RuleResult<'a> {

    pub fn new() -> Self {

        Self {
            tail: "",
            tokens: None,
            bounds: None,
            ts: 0,
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
                _ => (),
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

    pub fn set_time(&mut self, ts: usize) -> &mut Self {
        self.ts = ts;
        self
    }

}

#[derive(Debug)]
pub struct MatchResult {
    pub bounds: MatchBounds,
    pub tokens: Vec<Token>,
    pub ts: usize,
}

impl MatchResult {
    pub fn new(tokens: Vec<Token>, ts: usize, start_idx: usize, end_idx: usize) -> Self {
        Self {
            bounds: MatchBounds::new(start_idx, end_idx),
            tokens,
            ts,
        }
    }
}
