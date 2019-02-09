use std::fmt::Debug;

use strsim::damerau_levenshtein;
use nom::{named, take_while, preceded, tuple, Context, IResult, ErrorKind, types::CompleteStr};

macro_rules! set {
    (max_dist = $max_dist: expr, $fuzzy_scan: expr) => (if $fuzzy_scan {$max_dist} else {0});
}

mod en;

trait Token: Debug {}

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


/// Tries to recognize a word in a sentence using Domerau-Levenshtein algorithm, it is both simple
/// enough and efficient.
fn recognize_word<'a>(input: CompleteStr<'a>, pattern: CompleteStr<'a>, max_distance: usize,
                      token: &'a dyn Token) -> IResult<CompleteStr<'a>, &'a dyn Token> {
    if let Ok((tail, word)) = tokenize_word(input) {
        if damerau_levenshtein(*word, *pattern) <= max_distance {
            return Ok((tail, token));
        }
    }
    Err(nom::Err::Error(Context::Code(input, ErrorKind::Custom(1))))
}
