use super::rules::{FnRule, MatchBounds, MatchResult, MyResult, RuleResult, TokenDesc};

use nom::{
    IResult, Parser,
    bytes::complete::take_while,
    character::complete::{char, digit1, one_of},
    combinator::{opt, recognize},
    error::{Error, ErrorKind},
    sequence::preceded,
};

use chrono::{DateTime, TimeZone};
use strsim::damerau_levenshtein;

pub use super::errors::{DateTimeError, SemanticError, intersection_error};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Dist(pub usize);

/// Define word tokens, choosing the first spelling within its allowed distance.
/// Use `combine!` to choose the closest match between different tokens.
macro_rules! define {
    ($name:ident: ($token:expr, $priority:expr), $repr:expr, $distance:expr) => {
        define!($name: [($token, $priority), $repr, $distance]);
    };
    ($name:ident: $([($token:expr, $priority:expr), $repr:expr, $distance:expr])|+) => {
        pub fn $name(input: &str, exact_match: bool) -> crate::rules::rules::MyResult<'_> {
            $(
                if let Ok(result) = crate::rules::combinators::recognize_word(
                    input,
                    $repr,
                    if exact_match { crate::rules::combinators::Dist(0) } else { $distance },
                    crate::rules::tokens::PToken::PToken($token, $priority),
                ) {
                    return Ok(result);
                }
            )+
            crate::rules::combinators::wrap_error(input)
        }
    };
}

macro_rules! define_char {
    ($name:ident: $priority:expr, $repr:expr) => {
        fn $name(input: &str) -> crate::rules::rules::MyResult<'_> {
            let (tail, _) = crate::rules::combinators::recognize_symbol(input, $repr)?;
            Ok((
                tail,
                TokenDesc::new(
                    crate::rules::tokens::PToken::PToken(Token::Char, $priority),
                    crate::rules::combinators::Dist(0),
                ),
            ))
        }
    };
}

macro_rules! define_num {
    ($name:ident: ($ctor:expr, $priority:expr)) => {
        fn $name(input: &str) -> crate::rules::rules::MyResult<'_> {
            let (tail, value) = crate::rules::combinators::recognize_int(input)?;
            Ok((
                tail,
                TokenDesc::new(
                    crate::rules::tokens::PToken::PToken($ctor(value), $priority),
                    crate::rules::combinators::Dist(0),
                ),
            ))
        }
    };
}

/// Choose a unique token with the smallest editing distance.
macro_rules! combine {
    ($name:ident => $($parser:ident)|+) => {
        pub fn $name(input: &str, exact_match: bool) -> crate::rules::rules::MyResult<'_> {
            crate::rules::combinators::best_fit(input, exact_match, &[$($parser),+])
        }
    };
}

/// Convert the matched token array into a context and retain byte offsets.
macro_rules! make_interpreter {
    () => {
        make_interpreter!(interpret, parse);
    };
    ($interpreter:ident, $parser:ident) => {
        pub(crate) fn $interpreter<Tz: TimeZone>(
            input: &str,
            exact_match: bool,
            tz: DateTime<Tz>,
        ) -> Result<RuleResult<'_>, crate::rules::errors::SemanticError<'_>> {
            let mut res = RuleResult::new();
            match $parser(input, exact_match) {
                Ok((tail, (skipped, tokens))) => {
                    let bounds =
                        crate::rules::combinators::match_bounds(skipped.iter().sum(), input, tail);
                    res.set_bounds(Some(bounds));
                    for token in &tokens {
                        res.set_token(token);
                    }
                    res.set_tail(tail);
                    match make_time(&res, tz, &input[bounds.start_idx..bounds.end_idx]) {
                        Ok(ctx) => res.set_context(ctx),
                        Err(mut err) => {
                            err.set_bounds(bounds);
                            err.set_tail(tail);
                            return Err(err);
                        }
                    }
                }
                Err(nom::Err::Error(error)) => res.set_tail(error.input),
                _ => unreachable!("complete rule parsers only return recoverable errors"),
            }
            Ok(res)
        }
    };
}

fn is_ignorable(c: char) -> bool {
    !(c == '/' || c == ':' || c == '-' || c.is_alphanumeric())
}

/// Skip whitespace and punctuation before a token.
fn trim(input: &str) -> IResult<&str, &str> {
    take_while(is_ignorable).parse(input)
}

fn is_word_symbol(c: char) -> bool {
    c == '.' || c == ':' || c.is_alphanumeric()
}

fn tokenize_word(input: &str) -> IResult<&str, &str> {
    preceded(trim, take_while(is_word_symbol)).parse(input)
}

/// Consume a word with surrounding punctuation and count the consumed bytes.
pub(crate) fn tokenize_count_symbols(input: &str) -> IResult<&str, usize> {
    (trim, take_while(is_word_symbol), trim)
        .map(|(prefix, word, suffix): (&str, &str, &str)| prefix.len() + word.len() + suffix.len())
        .parse(input)
}

pub(crate) fn recognize_int(input: &str) -> IResult<&str, i32> {
    preceded(trim, recognize((opt(one_of("+-")), digit1)))
        .map_res(str::parse::<i32>)
        .parse(input)
}

pub fn recognize_symbol(input: &str, c: char) -> IResult<&str, char> {
    preceded(trim, char(c)).parse(input)
}

/// Pad shorter alternatives to the same token array length.
pub(crate) fn stub(input: &str) -> MyResult<'_> {
    Ok((input, TokenDesc::new(super::tokens::PToken::Stub, Dist(0))))
}

pub(crate) fn wrap_error(input: &str) -> MyResult<'_> {
    Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)))
}

pub(crate) fn recognize_word<'a>(
    input: &'a str,
    pattern: &str,
    max_dist: Dist,
    token: super::tokens::PToken,
) -> MyResult<'a> {
    if let Ok((tail, word)) = tokenize_word(input) {
        // Ignore periods in spellings such as a.m. and p.m.
        let normalized_word = word.replace('.', "");
        let dist = if max_dist == Dist(0) {
            if normalized_word != pattern {
                return wrap_error(input);
            }
            Dist(0)
        } else {
            Dist(damerau_levenshtein(&normalized_word, pattern))
        };
        if dist <= max_dist {
            return Ok((tail, TokenDesc::new(token, dist)));
        }
    }
    wrap_error(input)
}

type WordParser = for<'a> fn(&'a str, bool) -> MyResult<'a>;

pub(crate) fn best_fit<'a>(
    input: &'a str,
    exact_match: bool,
    combinators: &[WordParser],
) -> MyResult<'a> {
    let mut min_dist = Dist(usize::MAX);
    let mut selected_token = super::tokens::PToken::Stub;
    let mut selected_count = 0;
    let mut selected_tail = "";

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
    wrap_error(input)
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

    // handle possible remaining items
    let mut last_item = None;
    for item in matched_tokens.iter() {
        match item {
            Ok(token) => f(&item, token.get_start_idx(), token.get_end_idx()),
            Err(token) => f(&item, token.get_start_idx(), token.get_end_idx()),
        }
        last_item = Some(item);
    }

    if overlap.is_none() {
        if last_item.is_some() {
            result.push(last_item.unwrap().clone());
        }
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
    fallback_rules: &'a [FnRule<Tz>],
    exact_match: bool,
) -> Vec<Result<MatchResult, DateTimeError>> {
    // empty vector of matched tokens and errors
    let mut matched_tokens = Vec::new();

    // Explicit structured matches reserve their spans before fallback rules run.
    for (rule, fallback) in rules
        .iter()
        .map(|rule| (rule, false))
        .chain(fallback_rules.iter().map(|rule| (rule, true)))
    {
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
                    let bounds = MatchBounds::new(
                        end_of_last_match_idx + bounds.start_idx,
                        end_of_last_match_idx + bounds.end_idx,
                    );
                    if !fallback || !overlaps_existing(&matched_tokens, bounds) {
                        matched_tokens.push(Ok(MatchResult::new(
                            context,
                            bounds.start_idx,
                            bounds.end_idx,
                        )));
                    }

                    // continue with the rest of the string
                    end_of_last_match_idx = bounds.end_idx;
                    input = tail;
                }
                Ok(RuleResult { bounds: None, .. }) => {
                    // being inside this branch means that no more matches were found, we consider
                    // current rule as fully applied and must continue with the next rule
                    break;
                }
                Err(mut err) => {
                    let bounds = MatchBounds::new(
                        end_of_last_match_idx + err.get_start_idx(),
                        end_of_last_match_idx + err.get_end_idx(),
                    );
                    input = err.get_tail();
                    end_of_last_match_idx = bounds.end_idx;
                    err.set_bounds(bounds);
                    if !fallback || !overlaps_existing(&matched_tokens, bounds) {
                        matched_tokens.push(Err(err));
                    }
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

fn overlaps_existing(
    matches: &[Result<MatchResult, SemanticError<'_>>],
    bounds: MatchBounds,
) -> bool {
    matches.iter().any(|item| {
        let (start, end) = match item {
            Ok(item) => (item.get_start_idx(), item.get_end_idx()),
            Err(item) => (item.get_start_idx(), item.get_end_idx()),
        };
        start < bounds.end_idx && bounds.start_idx < end
    })
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
    tail: &str,
) -> crate::rules::rules::MatchBounds {
    crate::rules::rules::MatchBounds::new(prefix_len, input.len() - tail.len())
}
