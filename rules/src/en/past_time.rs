use chrono::prelude::*;

use crate::tokens::{Adverbs, Articles, IntWord, Priority, TimeInterval, Token};
use crate::{consts, rules::RuleResult, stub, Dist, TokenDesc};
use nom::{alt, apply, call, many_till, named_args, take, tuple, types::CompleteStr};

define!(one: (Token::IntWord(IntWord::One), Priority(0)), "one", Dist(0));
define!(two: (Token::IntWord(IntWord::Two), Priority(0)), "two", Dist(0));
define!(three: (Token::IntWord(IntWord::Three), Priority(0)), "three", Dist(1));
define!(four: (Token::IntWord(IntWord::Four), Priority(0)), "four", Dist(1));
define!(five: (Token::IntWord(IntWord::Five), Priority(0)), "five", Dist(1));
define!(six: (Token::IntWord(IntWord::Six), Priority(0)), "six", Dist(0));
define!(seven: (Token::IntWord(IntWord::Seven), Priority(0)), "seven", Dist(1));
define!(eight: (Token::IntWord(IntWord::Eight), Priority(0)), "eight", Dist(1));
define!(nine: (Token::IntWord(IntWord::Nine), Priority(0)), "nine", Dist(1));
define!(ten: (Token::IntWord(IntWord::Ten), Priority(0)), "ten", Dist(0));
define!(eleven: (Token::IntWord(IntWord::Eleven), Priority(0)), "eleven", Dist(1));
define!(twelve: (Token::IntWord(IntWord::Twelve), Priority(0)), "twelve", Dist(1));

combine!(int_word => one | two | three | four | five | six | seven | eight | nine | ten
                         | eleven | twelve);

define_num!(number: (Token::Number, Priority(0)));

define!(
    article:
    [(Token::Articles(Articles::A), Priority(0)), "a", Dist(0)] |
    [(Token::Articles(Articles::An), Priority(0)), "an", Dist(0)] |
    [(Token::Articles(Articles::The), Priority(0)), "the", Dist(0)]
);

define!(few_half:
    [(Token::Adverbs(Adverbs::Few), Priority(0)), "few", Dist(0)] |
    [(Token::Adverbs(Adverbs::Half), Priority(0)), "half", Dist(1)]
);

combine!(a_few_half => article | few_half);

define!(seconds: (Token::TimeInterval(TimeInterval::Second), Priority(1)), "second", Dist(2));
define!(minutes: (Token::TimeInterval(TimeInterval::Minute), Priority(1)), "minute", Dist(2));
define!(hours: (Token::TimeInterval(TimeInterval::Hour), Priority(1)), "hour", Dist(1));
define!(days: (Token::TimeInterval(TimeInterval::Day), Priority(1)), "days", Dist(1));
define!(weeks: (Token::TimeInterval(TimeInterval::Week), Priority(1)), "week", Dist(1));
define!(months: (Token::TimeInterval(TimeInterval::Month), Priority(1)), "month", Dist(1));
define!(years: (Token::TimeInterval(TimeInterval::Year), Priority(1)), "year", Dist(1));

combine!(time_interval => seconds | minutes | hours | days | weeks | months | years);

define!(ago: (Token::Ago, Priority(2)), "ago", Dist(0));

named_args!(parse<'a>(exact_match: bool)<CompleteStr<'a>, (Vec<CompleteStr<'a>>,
                             ( TokenDesc, TokenDesc, TokenDesc, TokenDesc ) )>,
    many_till!(take!(1),
        alt!(
            // half an hour ago
            tuple!(apply!(a_few_half, true), apply!(article, true), apply!(time_interval, true),
                   apply!(ago, exact_match)) |
            // few hours ago, a minute ago, half month ago
            tuple!(apply!(a_few_half, true), apply!(time_interval, true), apply!(ago, exact_match), stub) |
            // two hours ago, one minute ago, five month ago
            tuple!(apply!(int_word, true), apply!(time_interval, true), apply!(ago, exact_match), stub) |
            // 2 hours ago, 1 minute ago, 5 month ago
            tuple!(apply!(number, true), apply!(time_interval, true), apply!(ago, exact_match), stub)
        )
    )
);

make_interpreter!(positions = 4);

fn make_time(res: &mut RuleResult, local: DateTime<Local>, _input: &str) {
    let mut num = 0;
    let mut half = false;

    let token = res.token_by_priority(Priority(0));
    token.map_or((), |t| match t {
        Token::Articles(_) => {
            num = 1;
        }
        Token::Adverbs(Adverbs::Few) => {
            num = 3;
        }
        Token::Adverbs(Adverbs::Half) => {
            half = true;
        }
        Token::Number(n) => {
            num = n as i32;
        }
        _ => unreachable!(),
    });

    let token = res.token_by_priority(Priority(1));
    token.map_or((), |t| match t {
        Token::TimeInterval(TimeInterval::Second) => {
            res.set_duration(num);
        }
        Token::TimeInterval(TimeInterval::Minute) => res.set_duration(if half {
            -30 * consts::SECOND
        } else {
            -num * consts::MINUTE
        }),
        Token::TimeInterval(TimeInterval::Hour) => res.set_duration(if half {
            -30 * consts::MINUTE
        } else {
            -num * consts::HOUR
        }),
        Token::TimeInterval(TimeInterval::Day) => res.set_duration(if half {
            -12 * consts::HOUR
        } else {
            -num * consts::DAY
        }),
        Token::TimeInterval(TimeInterval::Week) => res.set_duration(if half {
            -7 * 12 * consts::HOUR
        } else {
            -num * consts::WEEK
        }),
        Token::TimeInterval(TimeInterval::Month) => {
            if half {
                res.set_duration(-14 * consts::DAY);
            } else {
                res.set_month(-(local.month() as i32 + num));
            }
        }
        Token::TimeInterval(TimeInterval::Year) => {
            if half {
                res.set_month(-(local.month() as i32 + 6));
            } else {
                res.set_year(-(local.year() + num));
            }
        }
        _ => unreachable!(),
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
    fn test_past_time() {
        let result = interpret("half an hour ago", false, fixed_time());
        assert_eq!(result.get_duration_sec() as i32, -30 * consts::MINUTE);
        // TODO: Check Levenshtein
        //        let result = interpret("1 hours ago", false, fixed_time());
        //        assert_eq!(result.get_duration_sec() as i32, -consts::HOUR);
    }
}
