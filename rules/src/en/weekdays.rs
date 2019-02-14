use crate::{stub, recognize_word, best_fit, Token, MatchResult};
use super::adjectives::when;
use super::nouns::week_noun;

use nom::{apply, alt, call, closure, named_args, named, many_till, take_s, tuple, eof, IResult,
          types::CompleteStr, recognize};

#[derive(Debug)]
enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl Token for Weekday {}

define!(monday, [Weekday::Monday, "monday", 2], [Weekday::Monday, "mon", 1]);
define!(tuesday, [Weekday::Tuesday, "tuesday", 2], [Weekday::Tuesday, "tue", 1]);
define!(wednesday, [Weekday::Wednesday, "wednesday", 2], [Weekday::Wednesday, "wed", 1]);
define!(thursday, [Weekday::Thursday, "thursday", 2], [Weekday::Thursday, "thur", 1]);
define!(friday, [Weekday::Friday, "friday", 2], [Weekday::Friday, "fri", 1]);
define!(saturday, [Weekday::Saturday, "saturday", 2], [Weekday::Saturday, "sat", 1]);
define!(sunday,  [Weekday::Sunday, "sunday", 2], [Weekday::Sunday, "sun", 1]);

combine!(day_of_week => monday, tuesday, wednesday, thursday, friday, saturday, sunday);

named_args!(parse<'a>(exact_match: bool)<CompleteStr<'a>, (Vec<CompleteStr<'a>>,
                                        (MatchResult<'a>, MatchResult<'a>, MatchResult<'a>))>,

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

fn apply(input: &str, exact_match: bool) {
    if let Ok((tail, (ignored_vec, (token1, token2, token3)))) = parse(CompleteStr(input), exact_match) {
        println!("parsed: {:?}, {:?}, {:?}", token1, token2, token3);
    }
}

#[test]
fn parse_monday() {
    println!("{:?}", apply(" sdfsd fri nxt wk", false));
}
