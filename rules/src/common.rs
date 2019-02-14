use std::fmt::Debug;
use nom::{IResult, types::CompleteStr};

// This marker trait MUST be implemented by all tokens
pub trait Token: Debug {}

#[derive(Debug)]
pub(crate) enum Common {
    Stub,
    None,
}

impl Token for Common {}

pub type MyResult<'a> = IResult<CompleteStr<'a>, MatchResult<'a>>;

#[derive(Debug)]
pub struct MatchResult<'a> {
    pub token: &'a dyn Token,
    pub dist: usize,
}

impl<'a> MatchResult<'a> {
    pub(crate) fn new(token: &'a dyn Token, dist: usize) -> Self {
        MatchResult{token, dist}
    }
}
