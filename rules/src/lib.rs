mod errors;
mod consts;
mod tokens;
pub mod rules;

use crate::errors as my_errors;
use crate::rules::{TokenDesc, MyResult, RuleResult, FnRule, MatchBounds, MatchResult};
use crate::tokens::PToken;

use chrono::prelude::Local;
use nom::{
    named, preceded, take_while, types::CompleteStr, Context, ErrorKind, IResult, map_res, recognize
};
use strsim::damerau_levenshtein;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Dist(pub usize);

macro_rules! set {
    ( max_dist = $max_dist: expr, $exact_match: expr ) => {
        if !$exact_match {
            $max_dist
        } else {
            crate::Dist(0)
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
/// instead of clumsy:
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
    ( $func_name: ident: ($token: expr, $p: expr), $repr: expr, $max_dist: expr ) => (
        named_args!(pub $func_name<'a>(exact_match: bool)<CompleteStr<'a>, TokenDesc>,
            call!(crate::recognize_word, CompleteStr($repr), set!(max_dist=$max_dist,
                  exact_match), crate::tokens::PToken::PToken($token, $p))
        );
    );
    ( $func_name: ident: $([($token: expr, $p: expr), $repr: expr, $max_dist: expr])|* ) => (
        named_args!(pub $func_name<'a>(exact_match: bool)<CompleteStr<'a>, TokenDesc>,
            alt!(
                $(call!(crate::recognize_word, CompleteStr($repr), set!(max_dist=$max_dist,
                        exact_match), crate::tokens::PToken::PToken($token, $p))) |*
            )
        );
    );
}

/// Macro simplifies bounded number parsers definition.
///
/// Examples:
///
/// Let's define Hour to be any number from 0 to 24:
///
/// define_num!(hour, (Token::Hour, 0), 0, 24);
///
/// Define minutes:
///
/// define_num!(hour, (Token::Minute, 0), 0, 60);
macro_rules! define_num {
    ( $func_name: ident, ($ctor: expr, $p: expr), $lower_bound: expr, $upper_bound: expr ) => (

        fn $func_name(input: CompleteStr, _: bool) -> crate::MyResult {
            let mut err_code = crate::my_errors::UNKNOWN;
            if let Ok((tail, n)) = crate::recognize_uint(input) {
                if n >= $lower_bound && n <= $upper_bound {
                    return Ok((tail, TokenDesc::new(crate::tokens::PToken::PToken($ctor(n), $p),
                                crate::Dist(0))));
                }
                err_code = crate::my_errors::OUT_OF_BOUNDS;
            }
            return crate::wrap_error(input, err_code);
        }
    );
}

/// Macro helps to combine tokens defined by define! macro into one, i.e.
///
/// combine!(day_of_week => monday | tuesday | wednesday | thursday | friday | saturday | sunday);
///
/// Defines "day_of_week" combinator which looks for the best match among the given variants.
///
/// Match considered as the best one if its Levenshtein distance is minimal compared to others.
macro_rules! combine {
    ( $func_name: ident => $($f: ident) |* ) => (
        named_args!(pub $func_name<'a>(exact_match: bool)<CompleteStr<'a>, TokenDesc>,
            call!(crate::best_fit, exact_match, vec![$(&$f),*])
        );
    );
}


/// TODO: add comment
macro_rules! make_interpreter {

    ( indices[$($n: expr),*] ) => (

        pub(crate) fn interpret(input: &str, exact_match: bool, local_time: DateTime<Local>) ->

            RuleResult {

            let mut res = RuleResult::new();

            if let Ok((tail, (skipped, tt))) = parse(CompleteStr(input), exact_match) {

                let bounds = crate::match_bounds(skipped, input, tail);

                res.set_bounds(Some(bounds))
                   .set_tokens(vec![$(tt.get($n).cloned().unwrap()),*])
                   .set_tail(*tail);

                make_time(&mut res, local_time, input);

            } else {
                res.set_tail(input);
            }

            res
        }

    );
}

pub mod en;

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
    preceded!(ltrim, take_while!(|c: char| c == '.' || c.is_alphabetic()))
);

/// Ignores whitespaces using "ltrim" and then consumes digits in a string until
/// any non digit character appears or the string has been exhausted, and in case of success
/// converts the number from the string representation into usize:
///
/// "  , 321  " -> 321
named!(recognize_uint<CompleteStr, usize>,
    map_res!(preceded!(ltrim, recognize!(nom::digit)),
                        |s: CompleteStr| s.parse::<usize>())
);

/// Stub combinator should be used in situations when there are several alternatives
/// are considered but they are differ in length, example:
///
/// named_args!(parse<'a>(exact_match: bool)<CompleteStr<'a>, (Vec<CompleteStr<'a>>,
///                             ( TokenDesc, TokenDesc, TokenDesc ) )>,
///     many_till!(take!(1),
///        alt!(
///            // day of week, when, "week", e.g. "tuesday next week"
///            tuple!(apply!(day_of_week, exact_match), apply!(when, exact_match),
///                   apply!(week_noun, exact_match)) |
///            // when and then any day of week, e.g. "last friday"
///            tuple!(apply!(when, exact_match), apply!(day_of_week, exact_match), call!(stub))
///        )
///    )
///
/// );
///
/// Note that the first variant expects 3 tokens to match with and the second one expects only
/// 2 tokens, function result type is a tuple with three elements, so tuple sizes of both match
/// arms must be the same size.
fn stub(input: CompleteStr) -> MyResult {
    Ok((input, TokenDesc::new(crate::tokens::PToken::Stub, crate::Dist(0))))
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
fn recognize_word<'a>(
    input: CompleteStr<'a>,
    pattern: CompleteStr<'a>,
    max_dist: crate::Dist,
    token: crate::tokens::PToken,
) -> MyResult<'a> {

    if let Ok((tail, word)) = tokenize_word(input) {
        if *word == "" {
            // skip empty strings
            return wrap_error(input, my_errors::EMPTY);
        }
        if max_dist == crate::Dist(0) {
            // when max_dist is 0 perform just plain string comparison
            if *word == *pattern {
                return Ok((tail, TokenDesc::new(token, crate::Dist(0))));
            }
        } else {
            let dist = Dist(damerau_levenshtein(*word, *pattern));
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

    let mut min_dist = Dist(std::usize::MAX);

    let mut selected_token = crate::tokens::PToken::None;
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
            match rule(input, exact_match, Local::now()) {
                RuleResult {
                    tail,
                    tokens: _,
                    bounds: Some(bounds),
                    time_shift,
                } => {
                    // applied rule had a match
                    matched_tokens.push(
                        MatchResult::new(time_shift, end_of_last_match_idx + bounds.start_idx,
                                         end_of_last_match_idx + bounds.end_idx)
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

/// Returns start and end indices of a match, accepts following arguments:
///
/// prefix - vector of characters before match start
/// input - input string
/// tail - everything that left after match
///
/// consider example below:
///
/// "I will meet you next friday evening"
///  |----prefix----|          |--tail--|
///  |---------------input--------------|
///
/// start_idx = prefix.len() + 1 or 0 if there is no prefix
/// end_idx = input.len() - tail.len() - 1
#[inline]
pub(crate) fn match_bounds(prefix: Vec<CompleteStr>, input: &str, tail: CompleteStr) -> crate::MatchBounds {
    crate::MatchBounds::new(if prefix.is_empty() { 0 } else { prefix.len() + 1 },
                            input.len() - tail.len() - 1)
}
