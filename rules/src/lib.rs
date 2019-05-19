mod common_matchers;
mod consts;
mod errors;
mod tokens;

pub mod rules;

use crate::rules::{Context, FnRule, MatchBounds, MatchResult, MyResult, RuleResult, TokenDesc};

use core::borrow::BorrowMut;
use nom::{
    self, alt, char, map, map_res, named, named_args, opt, pair, preceded, recognize, tag,
    take_while, tuple, types::CompleteStr, ErrorKind,
};

pub use crate::errors::{intersection_error, DateTimeError, SemanticError};
use chrono::{DateTime, TimeZone};
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
///
/// It's important to note that variants must be mutually exclusive, otherwise first (in order of
/// declaration) match will be selected.
///
/// For example:
///
/// define!(
///    foo_bar:
///    [(Token::Foo(Foo), Priority(0)), "foo", Dist(3)] |
///    [(Token::Bar(Bar), Priority(0)), "bar", Dist(3)]
/// );
///
/// passing string "bar" to foo_bar! combinator will lead to "Foo" token recognition, because
/// there is 3 units editing distance between strings "foo" and "bar" and therefor first match will
/// be chosen, which is "Foo".
///
/// Use "combine!" combinator if you want to choose the best fit word among the list.
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

macro_rules! define_char {
    ( $func_name: ident: $p: expr, $repr: expr ) => {
        fn $func_name(input: CompleteStr) -> crate::MyResult {
            if let Ok((tail, _)) = crate::recognize_symbol(input, $repr) {
                return Ok((
                    tail,
                    TokenDesc::new(
                        crate::tokens::PToken::PToken(Token::Char, $p),
                        crate::Dist(0),
                    ),
                ));
            }
            crate::wrap_error(input, crate::errors::UNKNOWN)
        }
    };
}

/// Macro simplifies bounded number parsers definition.
///
/// Example:
///
/// define_num!(hour, (Token::Hour, 0));
///
macro_rules! define_num {
    ( $func_name: ident: ($ctor: expr, $p: expr) ) => {
        fn $func_name(input: CompleteStr) -> crate::MyResult {
            if let Ok((tail, n)) = crate::recognize_int(input) {
                return Ok((
                    tail,
                    TokenDesc::new(
                        crate::tokens::PToken::PToken($ctor(n as i32), $p),
                        crate::Dist(0),
                    ),
                ));
            }
            crate::wrap_error(input, crate::errors::UNKNOWN)
        }
    };
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
    ( positions = $n: expr ) => {
        use tuple::TupleElements;

        pub(crate) fn interpret<Tz: TimeZone>(
            input: &str,
            exact_match: bool,
            tz: DateTime<Tz>,
        ) -> Result<RuleResult, crate::errors::SemanticError> {
            let mut res = RuleResult::new();
            match parse(CompleteStr(input), exact_match) {
                Ok((tail, (skipped, tt))) => {
                    let bounds = crate::match_bounds(skipped.iter().sum(), input, tail);
                    res.set_bounds(Some(bounds));
                    for idx in 0..$n {
                        res.set_token(tt.get(idx).unwrap());
                    }
                    res.set_tail(*tail);
                    match make_time(&res, tz, &input[bounds.start_idx..bounds.end_idx]) {
                        Ok(ctx) => res.set_context(ctx),
                        Err(mut err) => {
                            err.set_bounds(bounds);
                            err.set_tail(tail);
                            return Err(err);
                        }
                    }
                }
                Err(nom::Err::Error(nom::Context::Code(ref input, nom::ErrorKind::ManyTill))) => {
                    res.set_tail(input);
                }
                _ => unreachable!(),
            }

            Ok(res)
        }
    };
}

pub mod common;
pub mod en;

fn is_ignorable(c: char) -> bool {
    !(c == '/' || c == ':' || c == '-' || c.is_alphanumeric())
}

/// Trim spaces, special symbols and commas until any non-whitespace character appears
named!(trim<CompleteStr, CompleteStr>,
    take_while!(is_ignorable)
);

fn is_word_symbol(c: char) -> bool {
    c == '.' || c == ':' || c.is_alphanumeric()
}

/// Ignores whitespaces using "trim" and then consumes alphabetical characters in a string until
/// any non alpha-numeric character appears or the string has been exhausted:
///
/// "  , abracadabra  " -> "abracadabra"
named!(tokenize_word<CompleteStr, CompleteStr>,
    preceded!(trim, take_while!(is_word_symbol))
);

/// Consumes all spaces before a word, the word itself and all spaces after the word and returns
/// total number of consumed characters:
///
/// "  , abracadabra  " -> 17
named!(tokenize_count_symbols<CompleteStr, usize>,
    map!(tuple!(trim, take_while!(|c: char| c == '.' || c == ':' || c.is_alphanumeric()), trim),
    |(prefix, word, suffix)| {
        prefix.len() + word.len() + suffix.len()
    })
);

/// Ignores whitespaces using "trim" and then consumes digits in a string until
/// any non digit character appears or the string has been exhausted, and in case of success
/// converts the number from the string representation into i32:
///
/// "  , -321  " -> -321
named!(recognize_int<CompleteStr, i32>,
    map!(
        preceded!(trim, pair!(
            opt!(alt!(tag!("+") | tag!("-"))),
            map_res!(recognize!(nom::digit), |s: CompleteStr| s.parse::<i32>()))
            ),
        |(sign, value): (Option<nom::types::CompleteStr<'_>>, i32)| {
        sign.and_then(|s| if s == CompleteStr("-") { Some(-1) } else { None }).unwrap_or(1) * value
     })
);

named_args!(recognize_symbol<'a>(c: char)<CompleteStr<'a>, char>,
    preceded!(trim, char!(c))
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
    Ok((
        input,
        TokenDesc::new(crate::tokens::PToken::Stub, crate::Dist(0)),
    ))
}

#[inline]
fn wrap_error(input: CompleteStr, error_code: u32) -> MyResult {
    Err(nom::Err::Error(nom::simple_errors::Context::Code(
        input,
        ErrorKind::Custom(error_code),
    )))
}

/// Tries to recognize a word using Domerau-Levenshtein algorithm, it is both simple enough and
/// efficient.
fn recognize_word<'a>(
    input: CompleteStr<'a>,
    pattern: CompleteStr<'a>,
    max_dist: crate::Dist,
    token: crate::tokens::PToken,
) -> MyResult<'a> {
    if let Ok((tail, mut word)) = tokenize_word(input) {
        // TODO: add comment why we replace "." with "" (examples a.m., p.m.)
        let normalized_word = word.borrow_mut().replace(".", "");
        if max_dist == crate::Dist(0) {
            // when max_dist is 0 perform just plain string comparison
            if normalized_word == *pattern {
                return Ok((tail, TokenDesc::new(token, crate::Dist(0))));
            }
        } else {
            let dist = Dist(damerau_levenshtein(&normalized_word, *pattern));
            if dist <= max_dist {
                return Ok((tail, TokenDesc::new(token, dist)));
            }
        }
    }

    wrap_error(input, errors::UNKNOWN)
}

/// Finds a minimal distance between an input word by applying all combinators from funcs.
/// Each function accepts an input string and a flag which denotes whether exact match is required.
fn best_fit<'a>(
    input: CompleteStr<'a>,
    exact_match: bool,
    combinators: Vec<&Fn(CompleteStr<'a>, bool) -> MyResult<'a>>,
) -> MyResult<'a> {
    let mut min_dist = Dist(std::usize::MAX);

    let mut selected_token = crate::tokens::PToken::Stub;
    let mut selected_count = 0;
    let mut selected_tail = CompleteStr("");

    for comb in combinators {
        if let Ok((tail, TokenDesc { token, dist })) = comb(input, exact_match) {
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
    }

    wrap_error(input, errors::UNKNOWN)
}

pub(crate) fn remove_overlapped<'a>(
    source_str: &'a str,
    matched_tokens: &'a [Result<MatchResult, SemanticError<'a>>],
) -> Vec<Result<MatchResult, SemanticError<'a>>> {
    let mut result: Vec<Result<MatchResult, SemanticError>> = Vec::new();
    let mut overlap: Option<MatchBounds> = None;

    let mut min_idx = 0;
    let mut max_idx = None;

    let mut prev_elem = None;

    let mut f = |item: &'a Result<MatchResult, SemanticError>, start_idx, end_idx| {
        if max_idx.map_or(false, |x| x >= start_idx) {
            // maintain maximum position in text for overlapped interval
            overlap = match overlap {
                None => Some(MatchBounds::new(min_idx, end_idx)),
                Some(bounds) => Some(MatchBounds::new(bounds.start_idx, end_idx)),
            };
        } else if overlap.is_some() {
            result.push(Err(intersection_error(
                &source_str[overlap.unwrap().start_idx..overlap.unwrap().end_idx],
            )));
            overlap = None;
        } else if prev_elem.is_some() {
            result.push(prev_elem.take().unwrap());
        }

        if max_idx.map_or(true, |x| end_idx > x) {
            min_idx = start_idx;
            max_idx = Some(end_idx);

            prev_elem = Some(item.clone());
        }
    };

    let mut last_item = None;
    for item in matched_tokens.iter() {
        match item {
            Ok(token) => f(&item, token.get_start_idx(), token.get_end_idx()),
            Err(token) => f(&item, token.get_start_idx(), token.get_end_idx()),
        }
        last_item = Some(item);
    }

    if overlap.is_none() {
        result.push(last_item.unwrap().clone());
    } else {
        result.push(Err(intersection_error(
            &source_str[overlap.unwrap().start_idx..overlap.unwrap().end_idx],
        )));
    }

    result
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
#[inline]
pub(crate) fn apply_generic<'a, Tz: TimeZone + 'a>(
    date_time: DateTime<Tz>,
    source_str: &'a str,
    rules: &'a [FnRule<Tz>],
    exact_match: bool,
) -> Vec<Result<MatchResult, DateTimeError>> {
    // empty vector of matched tokens and errors
    let mut matched_tokens = Vec::new();

    for rule in rules {
        let mut input = source_str;
        let mut end_of_last_match_idx = 0;

        // try to apply one rule as many times as possible
        loop {
            match rule(input, exact_match, date_time.clone()) {
                Ok(RuleResult {
                    tail,
                    bounds: Some(bounds),
                    context,
                    ..
                }) => {
                    // applied rule had a match
                    matched_tokens.push(Ok(MatchResult::new(
                        context,
                        end_of_last_match_idx + bounds.start_idx,
                        end_of_last_match_idx + bounds.end_idx,
                    )));

                    // continue with the rest of the string
                    end_of_last_match_idx += bounds.end_idx;
                    input = tail;
                }
                Ok(RuleResult { bounds: None, .. }) => {
                    // being inside this branch means that no more matches were found, we consider
                    // current rule as fully applied and must continue with the next rule
                    break;
                }
                Err(err) => {
                    input = err.get_tail();
                    matched_tokens.push(Err(err));
                }
            }
        }
    }

    // all rules were applied at this step and the results were saved into matched_tokens vector

    // first of all we sort resulting vector by tokens start offsets
    matched_tokens.sort_by_key(|k| match k {
        Ok(x) => x.get_start_idx(),
        Err(x) => x.get_start_idx(),
    });

    // then look for tokens bounds intersections, and treat them as errors
    let tmp = remove_overlapped(source_str, &matched_tokens);

    // unbox errors
    tmp.iter()
        .map(|item| match item {
            Err(x) => Err(x.extract_error()),
            Ok(x) => Ok(*x),
        })
        .collect()
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
/// start_idx = prefix.len() or 0 if there is no prefix
/// end_idx = input.len() - tail.len()
#[inline]
pub(crate) fn match_bounds(
    prefix_len: usize,
    input: &str,
    tail: CompleteStr,
) -> crate::MatchBounds {
    crate::MatchBounds::new(prefix_len, input.len() - tail.len())
}
