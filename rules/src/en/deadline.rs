use chrono::prelude::*;

use crate::common_matchers::match_num;
use crate::tokens::{Adverbs, Articles, IntWord, Priority, TimeInterval, Token, When};
use crate::{consts, rules::RuleResult, stub, Dist, TokenDesc};

use nom::{alt, apply, call, many_till, named_args, take, tuple, types::CompleteStr};

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

define_num!(number: (Token::Number, Priority(3)));

combine!(number_or_intword => int_word | number);

define!(
    seconds:
    [(Token::TimeInterval(TimeInterval::Second), Priority(4)), "seconds", Dist(1)] |
    [(Token::TimeInterval(TimeInterval::Second), Priority(4)), "second", Dist(1)]
);

define!(
    minutes:
    [(Token::TimeInterval(TimeInterval::Minute), Priority(4)), "minutes", Dist(1)] |
    [(Token::TimeInterval(TimeInterval::Minute), Priority(4)), "minute", Dist(1)] |
    [(Token::TimeInterval(TimeInterval::Minute), Priority(4)), "min", Dist(0)]
);

define!(
    hours:
    [(Token::TimeInterval(TimeInterval::Hour), Priority(4)), "hours", Dist(1)] |
    [(Token::TimeInterval(TimeInterval::Hour), Priority(4)), "hour", Dist(1)]
);

define!(
    days:
    [(Token::TimeInterval(TimeInterval::Day), Priority(4)), "days", Dist(1)] |
    [(Token::TimeInterval(TimeInterval::Day), Priority(4)), "day", Dist(0)]
);

define!(
    weeks:
    [(Token::TimeInterval(TimeInterval::Week), Priority(4)), "weeks", Dist(1)] |
    [(Token::TimeInterval(TimeInterval::Week), Priority(4)), "week", Dist(1)]
);

define!(
    months:
    [(Token::TimeInterval(TimeInterval::Month), Priority(4)), "months", Dist(1)] |
    [(Token::TimeInterval(TimeInterval::Month), Priority(4)), "month", Dist(1)]
);

define!(
    years:
    [(Token::TimeInterval(TimeInterval::Year), Priority(4)), "years", Dist(1)] |
    [(Token::TimeInterval(TimeInterval::Year), Priority(4)), "year", Dist(1)]
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
            tuple!(apply!(when, exact_match), apply!(number_or_intword, exact_match),
                   apply!(time_interval, exact_match), stub) |
            // e.g.: in the few days
            tuple!(apply!(when, exact_match), apply!(article, true),
                   apply!(adverb, exact_match), apply!(time_interval, exact_match)) |
            // e.g.: in few days
            tuple!(apply!(when, exact_match), apply!(adverb, exact_match),
                   apply!(time_interval, exact_match), stub) |
            // e.g.: in a month, in a second, etc.
            tuple!(apply!(when, exact_match), apply!(time_interval, exact_match), stub, stub)
        )
    )
);

make_interpreter!(indices[0, 1, 2, 3]);

fn make_time(res: &mut RuleResult, local: DateTime<Local>, _input: &str) {
    let mut half = false;
    let mut num: i64 = 1;

    let token = res.token_by_priority(Priority(0));

    token.map_or((), |t| match t {
        Token::Adverbs(Adverbs::Few) => num = 3,
        Token::Adverbs(Adverbs::Half) => half = true,
        _ => (),
    });

    let num = match_num(res.token_by_priority(Priority(3))).unwrap_or(num);

    let token = res.token_by_priority(Priority(4));

    token.map_or((), |t| match t {
        Token::TimeInterval(TimeInterval::Second) => {
            res.set_duration(num);
        }
        Token::TimeInterval(TimeInterval::Minute) => res.set_duration(if half {
            30 * consts::SECOND as i64
        } else {
            num * consts::MINUTE as i64
        }),
        Token::TimeInterval(TimeInterval::Hour) => res.set_duration(if half {
            30 * consts::MINUTE as i64
        } else {
            num * consts::HOUR as i64
        }),
        Token::TimeInterval(TimeInterval::Day) => res.set_duration(if half {
            12 * consts::HOUR as i64
        } else {
            num * consts::DAY as i64
        }),
        Token::TimeInterval(TimeInterval::Week) => res.set_duration(if half {
            7 * 12 * consts::HOUR as i64
        } else {
            num * consts::WEEK as i64
        }),
        Token::TimeInterval(TimeInterval::Month) => {
            if half {
                res.set_duration(14 * consts::DAY as i64);
            } else {
                res.unwrap_mut().month = ((local.month() as i64 + num) % 12) as usize;
            }
        }
        Token::TimeInterval(TimeInterval::Year) => {
            if half {
                res.unwrap_mut().month = ((local.month() as i64 + 6) % 12) as usize;
            } else {
                res.unwrap_mut().year = (local.year() as i64 + num) as usize;
            }
        }
        _ => (),
    });
}

#[cfg(test)]
mod tests {
    use super::interpret;
    use crate::errors::DateTimeError::AmbiguousTime;
    use crate::{consts, MatchBounds};
    use chrono::prelude::*;

    fn fixed_time() -> DateTime<Local> {
        // 2019 1st January, Tuesday
        Local.ymd(2019, 1, 1).and_hms(0, 0, 1)
    }

    #[test]
    fn test_deadline() {
        let result = interpret("in 2 months", false, fixed_time());
        assert_eq!(result.context.unwrap().month, 3);

        let result = interpret("in three months", false, fixed_time());
        assert_eq!(result.context.unwrap().month, 4);

        let result = interpret("in a half year", false, fixed_time());
        assert_eq!(result.context.unwrap().month, 7);

        let result = interpret("in the few days", false, fixed_time());
        assert_eq!(result.context.unwrap().duration, 3 * consts::DAY as i64);

        let result = interpret("in 5 minutes", false, fixed_time());
        assert_eq!(result.context.unwrap().duration, 5 * consts::MINUTE as i64);

        let result = interpret("in 5 minutes I will go home", false, fixed_time());
        assert_eq!(result.context.unwrap().duration, 5 * consts::MINUTE as i64);

        let result = interpret(
            "we have to do something within 10 days.",
            false,
            fixed_time(),
        );
        assert_eq!(result.context.unwrap().duration, 10 * consts::DAY as i64);

        let result = interpret(
            "we have to do something within five days.",
            false,
            fixed_time(),
        );
        assert_eq!(result.context.unwrap().duration, 5 * consts::DAY as i64);

        let result = interpret("in a half year", false, fixed_time());
        assert_eq!(result.context.unwrap().month, 7);

        let result = interpret("drop me a line in a half hour", false, fixed_time());
        assert_eq!(result.context.unwrap().duration, 30 * consts::MINUTE as i64);
    }
}
