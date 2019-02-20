mod errors;
mod rule;
mod tokens;

use std::fmt::Debug;

use strsim::damerau_levenshtein;
use nom::{apply, named, named_args, take_while, preceded, many_till, alt, Context, IResult,
          ErrorKind, types::CompleteStr};

use self::errors::{AMBIGUOUS, UNKNOWN};
use self::rule::{MatchResult, MyResult};
use crate::tokens::Tokens;

macro_rules! set {
    ( max_dist = $max_dist: expr, $exact_match: expr ) => (
        if !$exact_match {Some($max_dist)} else {None}
    );
}

macro_rules! define {
    ( $func_name: ident, $token: expr, $repr: expr, $max_dist: expr ) => (
        named_args!(pub $func_name<'a>(exact_match: bool)<CompleteStr<'a>, MatchResult>,
            call!(recognize_word, CompleteStr($repr), set!(max_dist=$max_dist, exact_match), $token)
        );
    );
    ( $func_name: ident, $([$token: expr, $repr: expr, $max_dist: expr]),* ) => (
        named_args!(pub $func_name<'a>(exact_match: bool)<CompleteStr<'a>, MatchResult>,
            alt!(
                $(call!(recognize_word, CompleteStr($repr), set!(max_dist=$max_dist, exact_match),
                        $token)) |*
            )
        );
    );
}

macro_rules! combine {
    ( $func_name: ident => $($f: ident),* ) => (
        named_args!(pub $func_name<'a>(exact_match: bool)<CompleteStr<'a>, MatchResult>,
            call!(best_fit, exact_match, vec![$(&$f),*])
        );
    );
}

mod en;

/// Trim spaces, special symbols and commas until any non-whitespace character appears
named!(ltrim<CompleteStr, CompleteStr>,
    take_while!(
        |c: char|
          c.is_whitespace() ||
          c == ','
    )
);

/// Ignores whitespaces using "ltrim" and then consumes alphabetical characters in a string until
/// any non alphabetical character appears or the string has been exhausted:
///
/// "  , abracadabra  " -> "abracadabra"
named!(tokenize_word<CompleteStr, CompleteStr>,
    preceded!(ltrim, take_while!(|c: char| c.is_alphabetic()))
);

/// This function is required to ...
fn stub(input: CompleteStr) -> MyResult {
    Ok((input, MatchResult::new(Tokens::Stub, 0)))
}

#[inline]
fn throw_error(input: CompleteStr, error_code: u32) -> MyResult {
    Err(nom::Err::Error(Context::Code(input, ErrorKind::Custom(error_code))))
}

/// Tries to recognize a word in a sentence using Domerau-Levenshtein algorithm, it is both simple
/// enough and efficient.
#[inline]
fn recognize_word<'a>(input: CompleteStr<'a>, pattern: CompleteStr<'a>, max_dist: Option<usize>,
                      token: Tokens) -> MyResult<'a> {
    if let Ok((tail, word)) = tokenize_word(input) {
        if *word == *pattern {
            return Ok((tail, MatchResult::new(token, 0)));
        }

        if max_dist.is_some() {
            let dist = damerau_levenshtein(*word, *pattern);
            if dist <= max_dist.unwrap() {
                return Ok((tail, MatchResult::new(token, dist)));
            }
        }
    }

    throw_error(input, UNKNOWN)

}

/// Finds a minimal distance between an input word by applying all combinators from funcs.
/// Each function accepts an input string and a flag which denotes whether exact match is required.
fn best_fit<'a>(input: CompleteStr<'a>, exact_match: bool, funcs: Vec<&Fn(CompleteStr<'a>, bool) ->
                                                        MyResult<'a>>) -> MyResult<'a>
{
    let mut min_dist = std::usize::MAX;

    let mut selected_token = Tokens::None;
    let mut selected_count = 0;
    let mut selected_tail = CompleteStr("");

    for f in funcs {
        if let Ok((tail, MatchResult { token, dist })) = f(input, exact_match) {
            if min_dist > dist {
                selected_token = token;
                selected_tail = tail;
                selected_count = 1;
                min_dist = dist;
            } else if min_dist == dist {
                selected_count += 1;
            }
        }
    }

    if selected_count == 1 {
        return Ok((selected_tail, MatchResult::new(selected_token, min_dist)));
    } else if selected_count > 1 {
        return throw_error(input, AMBIGUOUS);
    }

    throw_error(input, UNKNOWN)

}
