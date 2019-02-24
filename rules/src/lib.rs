mod errors;
mod rules;
mod tokens;

use std::fmt::Debug;

use nom::{
    named, named_args, preceded, take_while, types::CompleteStr, Context, ErrorKind, IResult,
};
use strsim::damerau_levenshtein;

use self::errors as my_errors;
use self::rules::{TokenDesc, MyResult, RuleResult, FnRule, MatchBounds, MatchResult};
use crate::tokens::Tokens;

macro_rules! set {
    ( max_dist = $max_dist: expr, $exact_match: expr ) => {
        if !$exact_match {
            $max_dist
        } else {
            0
        }
    };
}

/// Macro helps to simplify compound combinator definition, for example:
///
/// define!(
///    saturday,
///    [Tokens::Weekday(Weekday::Saturday), "saturday", 2],
///    [Tokens::Weekday(Weekday::Saturday), "sat", 0]
///  );
///
/// instead of clumsy
///
/// named_args!(saturday<'a>(exact_match: bool)<CompleteStr<'a>, MatchResult<'a>>,
///    alt!(
///        call!(recognize_word, CompleteStr("saturday"), set!(max_dist=2, exact_match),
///            &Weekday::Saturday) |
///        call!(recognize_word, CompleteStr("sat"), set!(max_dist=0, exact_match),
///            &Weekday::Saturday)
///    )
/// );
macro_rules! define {
    ( $func_name: ident, $token: expr, $repr: expr, $max_dist: expr ) => (
        named_args!(pub $func_name<'a>(exact_match: bool)<CompleteStr<'a>, TokenDesc>,
            call!(recognize_word, CompleteStr($repr), set!(max_dist=$max_dist, exact_match), $token)
        );
    );
    ( $func_name: ident, $([$token: expr, $repr: expr, $max_dist: expr]),* ) => (
        named_args!(pub $func_name<'a>(exact_match: bool)<CompleteStr<'a>, TokenDesc>,
            alt!(
                $(call!(recognize_word, CompleteStr($repr), set!(max_dist=$max_dist, exact_match),
                        $token)) |*
            )
        );
    );
}

/// Macro helps to combine tokens defined by define! macro into one, i.e.
///
/// combine!(day_of_week => monday, tuesday, wednesday, thursday, friday, saturday, sunday);
///
/// defines "day_of_week" combinator which matches any of listed combinators
macro_rules! combine {
    ( $func_name: ident => $($f: ident),* ) => (
        named_args!(pub $func_name<'a>(exact_match: bool)<CompleteStr<'a>, TokenDesc>,
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
    Ok((input, TokenDesc::new(Tokens::Stub, 0)))
}

#[inline]
fn wrap_error(input: CompleteStr, error_code: u32) -> MyResult {
    Err(nom::Err::Error(Context::Code(
        input,
        ErrorKind::Custom(error_code),
    )))
}

/// Tries to recognize a word in a sentence using Domerau-Levenshtein algorithm, it is both simple
/// enough and efficient.
#[inline]
fn recognize_word<'a>(
    input: CompleteStr<'a>,
    pattern: CompleteStr<'a>,
    max_dist: usize,
    token: Tokens,
) -> MyResult<'a> {
    if let Ok((tail, word)) = tokenize_word(input) {
        if *word == "" {
            // skip empty strings
            return wrap_error(input, my_errors::EMPTY);
        }

        if max_dist == 0 {
            // when max_dist is 0 perform just plain string comparison
            if *word == *pattern {
                return Ok((tail, TokenDesc::new(token, 0)));
            }
        } else {
            let dist = damerau_levenshtein(*word, *pattern);
            if dist <= max_dist {
                return Ok((tail, TokenDesc::new(token, dist)));
            }
        }
    }

    wrap_error(input, my_errors::UNKNOWN)
}

/// Finds a minimal distance between an input word by applying all combinators from funcs.
/// Each function accepts an input string and a flag which denotes whether exact match is required.
fn best_fit<'a>(
    input: CompleteStr<'a>,
    exact_match: bool,
    funcs: Vec<&Fn(CompleteStr<'a>, bool) -> MyResult<'a>>,
) -> MyResult<'a> {
    let mut min_dist = std::usize::MAX;

    let mut selected_token = Tokens::None;
    let mut selected_count = 0;
    let mut selected_tail = CompleteStr("");

    for f in funcs {
        if let Ok((tail, TokenDesc { token, dist })) = f(input, exact_match) {
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
        return Ok((selected_tail, TokenDesc::new(selected_token, min_dist)));
    } else if selected_count > 1 {
        return wrap_error(input, my_errors::AMBIGUOUS);
    }

    wrap_error(input, my_errors::UNKNOWN)
}

/// Generic rules applier, accepts a string to parse as its input and a slice of rules,
/// then it tries to apply each rule from the list one by one, appending parsed tokens (if succeed)
/// to the output vector.
///
/// Let's consider an example:
///
/// input: "You can call me this friday or next monday."
///
/// output will be as follows: [[When(This), Weekday(Friday)], [When(Next), Weekday(Monday)]]
pub(crate) fn apply_generic(
    mut input: &str,
    rules: &[FnRule],
    exact_match: bool,
) -> Vec<MatchResult> {

    // empty vector of matched tokens
    let mut matched_tokens = Vec::new();
    let mut end_of_last_match_idx = 0;

    loop {
        let mut had_match = false;
        for rule in rules {
            match rule(input, exact_match) {
                RuleResult {
                    tail,
                    tokens: Some(tokens),
                    bounds: Some(bounds),
                } => {
                    // applied rule had a match
                    matched_tokens.push(
                        MatchResult::new(tokens, end_of_last_match_idx + bounds.start_idx,
                                         end_of_last_match_idx +bounds.end_idx)
                    );
                    // continue with the rest of the string
                    had_match = true;

                    input = tail;
                    end_of_last_match_idx += bounds.end_idx;
                    break;
                }
                _ => continue,
            }
        }

        if !had_match {
            break;
        }
    }

    matched_tokens
}

#[inline]
pub(crate) fn match_bounds(prefix: Vec<CompleteStr>, input: &str, tail: CompleteStr) -> MatchBounds {
    // TODO: add description
    MatchBounds::new(prefix.len() + 1, input.len() - tail.len() - 1)
}
