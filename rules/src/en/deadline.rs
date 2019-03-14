use chrono::prelude::*;

use crate::tokens::{Adverbs, Articles, Token, TimeInterval, IntWord, When, Priority};
use crate::{rules::RuleResult, TokenDesc, Dist, stub};
use crate::consts;
use tuple::TupleElements;

use nom::{
    alt, apply, call, many_till, named_args, take, tuple, types::CompleteStr
};

define!(
    adverb:
    [(Token::Adverbs(Adverbs::Few), Priority(0)), "few", Dist(0)] |
    [(Token::Adverbs(Adverbs::Half), Priority(0)), "half", Dist(1)]
);

define!(
    when:
    [(Token::When(When::Within), Priority(1)), "within", Dist(1)] |
    [(Token::When(When::In), Priority(1)), "in", Dist(0)]
);

define!(
    article:
    [(Token::Articles(Articles::A), Priority(2)), "a", Dist(0)] |
    [(Token::Articles(Articles::An), Priority(2)), "an", Dist(0)] |
    [(Token::Articles(Articles::The), Priority(2)), "the", Dist(0)]
);

define!(one: (Token::IntWord(IntWord::One), Priority(3)), "one", Dist(0));
define!(two: (Token::IntWord(IntWord::Two), Priority(3)), "two", Dist(0));
define!(three: (Token::IntWord(IntWord::Three), Priority(3)), "three", Dist(1));
define!(four: (Token::IntWord(IntWord::Four), Priority(3)), "four", Dist(1));
define!(five: (Token::IntWord(IntWord::Five), Priority(3)), "five", Dist(1));
define!(six: (Token::IntWord(IntWord::Six), Priority(3)), "six", Dist(0));
define!(seven: (Token::IntWord(IntWord::Seven), Priority(3)), "seven", Dist(1));
define!(eight: (Token::IntWord(IntWord::Eight), Priority(3)), "eight", Dist(1));
define!(nine: (Token::IntWord(IntWord::Nine), Priority(3)), "nine", Dist(1));
define!(ten: (Token::IntWord(IntWord::Ten), Priority(3)), "ten", Dist(0));
define!(eleven: (Token::IntWord(IntWord::Eleven), Priority(3)), "eleven", Dist(1));
define!(twelve: (Token::IntWord(IntWord::Twelve), Priority(3)), "twelve", Dist(1));

combine!(int_word => one | two | three | four | five | six | seven | eight | nine | ten
                         | eleven | twelve);

define_num!(number, (Token::Number, Priority(3)), 0, 12);

combine!(number_or_intword => int_word | number);

define!(
    seconds:
    [(Token::TimeInterval(TimeInterval::Seconds), Priority(4)), "seconds", Dist(1)] |
    [(Token::TimeInterval(TimeInterval::Seconds), Priority(4)), "second", Dist(1)]
);

define!(
    minutes:
    [(Token::TimeInterval(TimeInterval::Minutes), Priority(4)), "minutes", Dist(1)] |
    [(Token::TimeInterval(TimeInterval::Minutes), Priority(4)), "minute", Dist(1)] |
    [(Token::TimeInterval(TimeInterval::Minutes), Priority(4)), "min", Dist(0)]
);

define!(
    hours:
    [(Token::TimeInterval(TimeInterval::Hours), Priority(4)), "hours", Dist(1)] |
    [(Token::TimeInterval(TimeInterval::Hours), Priority(4)), "hour", Dist(1)]
);

define!(
    days:
    [(Token::TimeInterval(TimeInterval::Days), Priority(4)), "days", Dist(1)] |
    [(Token::TimeInterval(TimeInterval::Days), Priority(4)), "day", Dist(0)]
);

define!(
    weeks:
    [(Token::TimeInterval(TimeInterval::Weeks), Priority(4)), "weeks", Dist(1)] |
    [(Token::TimeInterval(TimeInterval::Weeks), Priority(4)), "week", Dist(1)]
);

define!(
    months:
    [(Token::TimeInterval(TimeInterval::Months), Priority(4)), "months", Dist(1)] |
    [(Token::TimeInterval(TimeInterval::Months), Priority(4)), "month", Dist(1)]
);

define!(
    years:
    [(Token::TimeInterval(TimeInterval::Years), Priority(4)), "years", Dist(1)] |
    [(Token::TimeInterval(TimeInterval::Years), Priority(4)), "year", Dist(1)]
);

combine!(time_interval => seconds | minutes | hours | days | weeks | months | years);

named_args!(parse<'a>(exact_match: bool)<CompleteStr<'a>, (Vec<CompleteStr<'a>>,
                             ( TokenDesc, TokenDesc, TokenDesc, TokenDesc ) )>,

    many_till!(take!(1),
        alt!(
            // e.g.: in a five months
            tuple!(apply!(when, exact_match), apply!(article, true),
                   apply!(number_or_intword, exact_match), apply!(time_interval, exact_match)) |
            // e.g.: in five months
            tuple!(apply!(when, exact_match), call!(stub),
                   apply!(number_or_intword, exact_match), apply!(time_interval, exact_match)) |
            // e.g.: in the few days
            tuple!(apply!(when, exact_match), apply!(article, true),
                   apply!(adverb, exact_match), apply!(time_interval, exact_match)) |
            // e.g.: in few days
            tuple!(apply!(when, exact_match), call!(stub),
                   apply!(adverb, exact_match), apply!(time_interval, exact_match)) |
            // e.g.: in a month, in a second, etc.
            tuple!(apply!(when, exact_match), call!(stub), call!(stub),
                   apply!(time_interval, exact_match))
        )
    )

);

make_interpreter!(indices[0, 1, 2, 3]);

fn make_time(res: &mut RuleResult, local: DateTime<Local>, input: &str) {

    let mut offset: i64 = 0;
    let mut num: i64 = 0;

    // if we are here, tokens MUST be Some
    //let tokens = res.tokens.as_mut().unwrap();

    let mut half = false;

    let token = res.token_by_priority(Priority(0));

    match token.unwrap_or(&Token::None) {
        Token::Adverbs(Adverbs::Few) => num = 3,
        Token::Adverbs(Adverbs::Half) => half = true,
        _ => (),
    }

    let token = res.token_by_priority(Priority(2));

    match token.unwrap_or(&Token::None) {
        Token::Articles(_) => num = 1,
        _ => (),
    }

    let token = res.token_by_priority(Priority(3));

    match token.unwrap_or(&Token::None) {
        Token::IntWord(IntWord::One) => num = 1,
        Token::IntWord(IntWord::Two) => num = 2,
        Token::IntWord(IntWord::Three) => num = 3,
        Token::IntWord(IntWord::Four) => num = 4,
        Token::IntWord(IntWord::Five) => num = 5,
        Token::IntWord(IntWord::Six) => num = 6,
        Token::IntWord(IntWord::Seven) => num = 7,
        Token::IntWord(IntWord::Eight) => num = 8,
        Token::IntWord(IntWord::Nine) => num = 9,
        Token::IntWord(IntWord::Ten) => num = 10,
        Token::IntWord(IntWord::Eleven) => num = 11,
        Token::IntWord(IntWord::Twelve) => num = 12,

        Token::Number(n) => num = *n as i64,
        _ => (),
    }

    let token = res.token_by_priority(Priority(4));

    match token.unwrap_or(&Token::None) {
        Token::TimeInterval(TimeInterval::Months) => {
            if half {
                offset = 14 * consts::DAY as i64;
            } else {
                res.time_shift.as_mut().unwrap().month =
                    ((local.month() as i64 + num) % 12) as usize;
            }
        },
        Token::TimeInterval(TimeInterval::Years) => {
            if half {
                res.time_shift.as_mut().unwrap().month =
                    ((local.month() as i64 + 6) % 12) as usize;
            } else {
                res.time_shift.as_mut().unwrap().year =
                    (local.year() as i64 + num) as usize;
            }
        },
        _ => (),
    }

    res.time_shift.as_mut().unwrap().offset = offset;

}


#[cfg(test)]
mod tests {
    use chrono::prelude::*;
    use crate::tokens::{Token, Weekday as Day, When};
    use crate::MatchBounds;
    use super::interpret;
    use crate::errors::DateTimeError::AmbiguousTime;

    fn fixed_time() -> DateTime<Local> {
        // 2019 1st January, Tuesday
        Local.ymd(2019, 1, 1).and_hms(0, 0, 1)
    }

    #[test]
    fn test_deadline() {
        let mut result = interpret("in three months", false, fixed_time());
        assert_eq!(result.time_shift.unwrap().month, 4);

        let mut result = interpret("in a half year", false, fixed_time());
        assert_eq!(result.time_shift.unwrap().month, 7);

    }
}
