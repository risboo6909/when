use chrono::prelude::*;

use super::super::Context;
use crate::common_matchers::match_num;
use crate::errors::DateTimeError;
use crate::tokens::{Adverbs, Articles, IntWord, Priority, TimeInterval, Token};
use crate::{consts, rules::RuleResult, stub, tokenize_count_symbols, Dist, TokenDesc};
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

define!(seconds: (Token::TimeInterval(TimeInterval::Second), Priority(1)), "seconds", Dist(3));
define!(minutes: (Token::TimeInterval(TimeInterval::Minute), Priority(1)), "minutes", Dist(3));
define!(hours: (Token::TimeInterval(TimeInterval::Hour), Priority(1)), "hours", Dist(2));
define!(days: (Token::TimeInterval(TimeInterval::Day), Priority(1)), "days", Dist(2));
define!(weeks: (Token::TimeInterval(TimeInterval::Week), Priority(1)), "weeks", Dist(2));
define!(months: (Token::TimeInterval(TimeInterval::Month), Priority(1)), "months", Dist(2));
define!(years: (Token::TimeInterval(TimeInterval::Year), Priority(1)), "years", Dist(2));

combine!(time_interval => seconds | minutes | hours | days | weeks | months | years);

define!(ago: (Token::Ago, Priority(2)), "ago", Dist(0));

named_args!(parse<'a>(exact_match: bool)<CompleteStr<'a>, (Vec<usize>,
                             ( TokenDesc, TokenDesc, TokenDesc, TokenDesc ) )>,
    many_till!(tokenize_count_symbols,
        alt!(
            // half an hour ago
            tuple!(apply!(a_few_half, exact_match), apply!(article, true), apply!(time_interval, exact_match),
                   apply!(ago, exact_match)) |
            // few hours ago, a minute ago, half month ago
            tuple!(apply!(a_few_half, exact_match), apply!(time_interval, exact_match),
                   apply!(ago, true), stub) |
            // two hours ago, one minute ago, five month ago
            tuple!(apply!(int_word, exact_match), apply!(time_interval, exact_match),
                   apply!(ago, true), stub) |
            // 2 hours ago, 1 minute ago, 5 month ago
            tuple!(number, apply!(time_interval, exact_match), apply!(ago, true), stub) |
            // a month ago, an year ago
            tuple!(apply!(a_few_half, exact_match), apply!(time_interval, exact_match),
                   apply!(ago, true), stub)
        )
    )
);

make_interpreter!(positions = 4);

fn make_time<Tz: TimeZone>(
    res: &RuleResult,
    tz_aware: DateTime<Tz>,
    input: &str,
) -> Result<Context, DateTimeError> {
    let mut ctx = Context::default();
    let mut num = 0;
    let mut half = false;

    let token = res.token_by_priority(Priority(0));
    if token.is_some() {
        if let Some(n) = match_num(token.clone()) {
            num = n;
        } else {
            match token.unwrap() {
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
            };
        }
    }

    if num < 0 {
        return Err(DateTimeError::InvalidTime {
            msg: input.to_string(),
            what: "number".to_owned(),
            value: num,
        });
    }

    let token = res.token_by_priority(Priority(1));
    if token.is_some() {
        match token.unwrap() {
            Token::TimeInterval(TimeInterval::Second) => {
                ctx.set_duration(-num);
            }
            Token::TimeInterval(TimeInterval::Minute) => {
                ctx.set_duration(if half {
                    -30 * consts::SECOND
                } else {
                    -num * consts::MINUTE
                });
            }
            Token::TimeInterval(TimeInterval::Hour) => {
                ctx.set_duration(if half {
                    -30 * consts::MINUTE
                } else {
                    -num * consts::HOUR
                });
            }
            Token::TimeInterval(TimeInterval::Day) => {
                ctx.set_duration(if half {
                    -12 * consts::HOUR
                } else {
                    -num * consts::DAY
                });
            }
            Token::TimeInterval(TimeInterval::Week) => {
                ctx.set_duration(if half {
                    -7 * 12 * consts::HOUR
                } else {
                    -num * consts::WEEK
                });
            }
            Token::TimeInterval(TimeInterval::Month) => {
                if half {
                    ctx.set_duration(-14 * consts::DAY);
                } else {
                    ctx.month = Some(tz_aware.month() as i32 - num);
                }
            }
            Token::TimeInterval(TimeInterval::Year) => {
                if half {
                    ctx.month = Some(tz_aware.month() as i32 - 6);
                } else {
                    ctx.year = Some(tz_aware.year() - num);
                }
            }
            _ => unreachable!(),
        };
    }

    Ok(ctx)
}

#[cfg(test)]
mod tests {
    use super::interpret;
    use crate::errors::DateTimeError::InvalidTime;
    use crate::{consts, MatchBounds};
    use chrono::prelude::*;

    fn fixed_time() -> DateTime<Local> {
        // 2019 1st January, Tuesday
        Local.ymd(2019, 1, 1).and_hms(0, 0, 1)
    }

    #[test]
    fn test_past_time() {
        let result = interpret("half an hour ago", false, fixed_time()).unwrap();
        assert_eq!(result.get_duration_sec() as i32, -30 * consts::MINUTE);

        let result = interpret("2 hour ago", false, fixed_time()).unwrap();
        assert_eq!(result.get_duration_sec() as i32, -2 * consts::HOUR);

        let result = interpret("5 minuts ago", false, fixed_time()).unwrap();
        assert_eq!(result.get_duration_sec() as i32, -5 * consts::MINUTE);

        let result = interpret("5 mnte ago I went to the zoo", false, fixed_time()).unwrap();
        assert_eq!(result.get_duration_sec() as i32, -5 * consts::MINUTE);

        let result = interpret("-5 mnte ago I went to the zoo", false, fixed_time());
        assert_eq!(
            result.unwrap_err(),
            InvalidTime {
                msg: "-5 mnte ago I went to the zoo".to_owned(),
                what: "number".to_owned(),
                value: -5,
            }
        );

        let result = interpret("we did something 10 days ago.", false, fixed_time()).unwrap();
        assert_eq!(result.get_duration_sec() as i32, -10 * consts::DAY);

        let result = interpret("we did something five days ago.", false, fixed_time()).unwrap();
        assert_eq!(result.get_duration_sec() as i32, -5 * consts::DAY);

        let result = interpret("5 seconds ago a car was moved", false, fixed_time()).unwrap();
        assert_eq!(result.get_duration_sec() as i32, -5 * consts::SECOND);

        let result = interpret("two weks ago", false, fixed_time()).unwrap();
        assert_eq!(result.get_duration_sec() as i32, -2 * consts::WEEK);

        let result = interpret("a month ago", false, fixed_time()).unwrap();
        assert_eq!(result.get_month(), 0);

        let result = interpret("a few months ago", false, fixed_time()).unwrap();
        assert_eq!(result.get_month(), -2);

        let result = interpret("half year ago", false, fixed_time()).unwrap();
        assert_eq!(result.get_month(), -5);
    }
}
