use super::{adjectives::when, nouns::week_noun};
use crate::tokens::{Tokens, Weekday};
use crate::{best_fit, recognize_word, rules::RuleResult, stub, TokenDesc, match_bounds};
use crate::rules::MatchBounds;

use nom::{
    alt, apply, call, closure, eof, many_till, named, named_args, recognize, take, tuple,
    types::CompleteStr, IResult,
};

define!(
    monday,
    [Tokens::Weekday(Weekday::Monday), "monday", 1],
    [Tokens::Weekday(Weekday::Monday), "mon", 0]
);

define!(
    tuesday,
    [Tokens::Weekday(Weekday::Tuesday), "tuesday", 1],
    [Tokens::Weekday(Weekday::Tuesday), "tue", 0]
);

define!(
    wednesday,
    [Tokens::Weekday(Weekday::Wednesday), "wednesday", 2],
    [Tokens::Weekday(Weekday::Wednesday), "wed", 0]
);

define!(
    thursday,
    [Tokens::Weekday(Weekday::Thursday), "thursday", 2],
    [Tokens::Weekday(Weekday::Thursday), "thur", 0]
);

define!(
    friday,
    [Tokens::Weekday(Weekday::Friday), "friday", 1],
    [Tokens::Weekday(Weekday::Friday), "fri", 0]
);

define!(
    saturday,
    [Tokens::Weekday(Weekday::Saturday), "saturday", 2],
    [Tokens::Weekday(Weekday::Saturday), "sat", 0]
);

define!(
    sunday,
    [Tokens::Weekday(Weekday::Sunday), "sunday", 1],
    [Tokens::Weekday(Weekday::Sunday), "sun", 0]
);

combine!(day_of_week => monday, tuesday, wednesday, thursday, friday, saturday, sunday);

named_args!(parse<'a>(exact_match: bool)<CompleteStr<'a>, (Vec<CompleteStr<'a>>,
                             ( TokenDesc, TokenDesc, TokenDesc ) )>,

    many_till!(take!(1),
        alt!(
            // day of week, when, "week", e.g. "tuesday next week"
            tuple!(apply!(day_of_week, exact_match), apply!(when, exact_match),
                   apply!(week_noun, exact_match)) |
            // when and then any day of week, e.g. "last friday"
            tuple!(apply!(when, exact_match), apply!(day_of_week, exact_match), call!(stub)) |
            // day of week only, e.g. "sunday", "monday", etc
            tuple!(apply!(day_of_week, exact_match), call!(stub), call!(stub))
        )
    )

);

pub(crate) fn apply(input: &str, exact_match: bool) -> RuleResult {
    if let Ok((tail, (skipped, tt))) = parse(CompleteStr(input), exact_match) {
        let bounds = match_bounds(skipped, input, tail);
        return RuleResult::new(*tail, vec![tt.0, tt.1, tt.2], Some(bounds));
    }
    RuleResult::new(input, vec![], None)
}

//#[test]
//fn parse_monday() {
//    println!("{:?}", apply(" sdfsd ths frday sc", false));
//}
