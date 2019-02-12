use std::fmt::Debug;

use strsim::damerau_levenshtein;
use nom::{apply, named, named_args, take_while, preceded, many_till, Context, IResult, ErrorKind, types::CompleteStr};

macro_rules! set {
    (max_dist = $max_dist: expr, $fuzzy_scan: expr) => (if $fuzzy_scan {Some($max_dist)} else {None});
}

mod en;

const MAX_DIST: usize = 1000;

// errors
const AMBIGUOUS: u32 = 0;
const UNKNOWN: u32 = 1;

#[derive(Debug)]
enum Common {
    None,
}

trait Token: Debug {}

impl Token for Common {}

type MyResult<'a> = IResult<CompleteStr<'a>, MatchResult<'a>>;

#[derive(Debug)]
pub struct MatchResult<'a> {
    token: &'a dyn Token,
    dist: usize,
}

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


#[inline]
fn throw_error(input: CompleteStr, error_code: u32) -> MyResult {
    Err(nom::Err::Error(Context::Code(input, ErrorKind::Custom(error_code))))
}


/// Tries to recognize a word in a sentence using Domerau-Levenshtein algorithm, it is both simple
/// enough and efficient.
#[inline]
fn recognize_word<'a>(input: CompleteStr<'a>, pattern: CompleteStr<'a>, max_dist: Option<usize>,
                      token: &'a dyn Token) -> MyResult<'a> {
    if let Ok((tail, word)) = tokenize_word(input) {
        if *word == *pattern {
            return Ok((tail, MatchResult { token, dist: 0 }));
        }

        if max_dist.is_some() {
            let dist = damerau_levenshtein(*word, *pattern);
            if dist <= max_dist.unwrap() {
                return Ok((tail, MatchResult { token, dist }));
            }
        }
    }

    throw_error(input, UNKNOWN)

}

/// Finds a minimal distance between an input word by applying all combinators from funcs.
/// Each function accepts an input string and a flag which denotes whether exact match is required.
fn best_fit<'a>(input: CompleteStr<'a>, fuzzy_scan: bool, funcs: Vec<&Fn(CompleteStr<'a>, bool) ->
                                                        MyResult<'a>>) -> MyResult<'a>
{
    let mut min_dist = MAX_DIST;

    let mut selected_token: &'a dyn Token = &Common::None;
    let mut selected_count = 0;
    let mut selected_tail = CompleteStr("");

    for f in funcs {
        if let Ok((tail, MatchResult { token, dist })) = f(input, fuzzy_scan) {
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
        return Ok((selected_tail, MatchResult { token: selected_token, dist: min_dist }));
    } else if selected_count > 1 {
        return throw_error(input, AMBIGUOUS);
    }

    throw_error(input, UNKNOWN)

}
