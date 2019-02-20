use crate::tokens::{Tokens, Weekday};
use crate::{stub, recognize_word, best_fit, MatchResult, rule::ApplyResult};
use super::{adjectives::when, nouns::week_noun};

use nom::{apply, alt, call, closure, named_args, named, many_till, take_s, tuple, eof, IResult,
          types::CompleteStr, recognize};


define!(monday, [Tokens::Weekday(Weekday::Monday), "monday", 2],
                [Tokens::Weekday(Weekday::Monday), "mon", 1]);

define!(tuesday, [Tokens::Weekday(Weekday::Tuesday), "tuesday", 2],
                 [Tokens::Weekday(Weekday::Tuesday), "tue", 1]);

define!(wednesday, [Tokens::Weekday(Weekday::Wednesday), "wednesday", 2],
                   [Tokens::Weekday(Weekday::Wednesday), "wed", 1]);

define!(thursday, [Tokens::Weekday(Weekday::Thursday), "thursday", 2],
                  [Tokens::Weekday(Weekday::Thursday), "thur", 1]);

define!(friday, [Tokens::Weekday(Weekday::Friday), "friday", 2],
                [Tokens::Weekday(Weekday::Friday), "fri", 1]);

define!(saturday, [Tokens::Weekday(Weekday::Saturday), "saturday", 2],
                  [Tokens::Weekday(Weekday::Saturday), "sat", 1]);

define!(sunday, [Tokens::Weekday(Weekday::Sunday), "sunday", 2],
                [Tokens::Weekday(Weekday::Sunday), "sun", 1]);

combine!(day_of_week => monday, tuesday, wednesday, thursday, friday, saturday, sunday);

named_args!(parse<'a>(exact_match: bool)<CompleteStr<'a>, (Vec<CompleteStr<'a>>,
                             ( MatchResult, MatchResult, MatchResult ) )>,

    many_till!(take_s!(1),
        alt!(
            // day of week, when, "week", e.g. "tuesday next week"
            tuple!(apply!(day_of_week, exact_match), apply!(when, exact_match),
                   apply!(week_noun, exact_match)) |
            // when and then any day of week, e.g. "last friday"
            tuple!(call!(stub), apply!(when, exact_match), apply!(day_of_week, exact_match)) |
            // nothing and then any day of week, e.g. "sunday"
            tuple!(call!(stub), call!(stub), apply!(day_of_week, exact_match))
        )
    )

);

fn apply(input: &str, exact_match: bool) -> ApplyResult {
    if let Ok((tail, (_, (token1, token2, token3)))) =
                                parse(CompleteStr(input), exact_match) {
        return ApplyResult::new(*tail, vec![token1, token2, token3]);
    }
    ApplyResult::new(input, vec![])
}

#[test]
fn parse_monday() {
    println!("{:?}", apply(" sdfsd ths frday sc", false));
}
