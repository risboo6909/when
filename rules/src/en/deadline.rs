use chrono::prelude::*;

use super::super::Context;
use crate::common_matchers::match_num;
use crate::errors::{invalid_time_error, SemanticError};
use crate::tokens::{
    Adverbs, Articles, IntWord, Prepositions, Priority, TimeInterval, Token, When,
};
use crate::{
    consts,
    rules::{MatchBounds, RuleResult},
    stub, tokenize_count_symbols, Dist, TokenDesc,
};

use nom::{
    alt, apply, call, is_alphanumeric, many_till, named_args, take_while, tuple, types::CompleteStr,
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

define!(prepos: (Token::Prepositions(Prepositions::Of), Priority(2)), "of", Dist(0));

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

named_args!(parse<'a>(exact_match: bool)<CompleteStr<'a>, (Vec<usize>,
                             ( TokenDesc, TokenDesc, TokenDesc, TokenDesc, TokenDesc ) )>,

    many_till!(tokenize_count_symbols,
        alt!(
            // e.g.: in the half of year
            tuple!(apply!(when, exact_match), apply!(article, true), apply!(adverb, exact_match),
                   apply!(prepos, true), apply!(time_interval, exact_match)) |
            // e.g.: in half of year
            tuple!(apply!(when, exact_match), apply!(adverb, exact_match), apply!(prepos, true),
                   apply!(time_interval, exact_match), stub) |
            // e.g.: in a five months
            tuple!(apply!(when, exact_match), apply!(article, true),
                   apply!(int_word, exact_match), apply!(time_interval, exact_match), stub) |
            // e.g.: in a 5 months
            tuple!(apply!(when, exact_match), apply!(article, true), number,
                   apply!(time_interval, exact_match), stub) |
            // e.g.: in five months
            tuple!(apply!(when, exact_match), apply!(int_word, exact_match),
                   apply!(time_interval, exact_match), stub, stub) |
            // e.g.: in 5 months
            tuple!(apply!(when, exact_match), number, apply!(time_interval, exact_match), stub, stub) |
            // e.g.: in the few days
            tuple!(apply!(when, exact_match), apply!(article, true),
                   apply!(adverb, exact_match), apply!(time_interval, exact_match), stub) |
            // e.g.: in few days
            tuple!(apply!(when, exact_match), apply!(adverb, exact_match),
                   apply!(time_interval, exact_match), stub, stub) |
            // e.g.: in a month, in a second, etc.
            tuple!(apply!(when, exact_match), apply!(time_interval, exact_match), stub, stub, stub)
        )
    )
);

make_interpreter!(positions = 5);

fn make_time<'a, 'b, Tz: TimeZone>(
    res: &'a RuleResult,
    tz_aware: DateTime<Tz>,
    input: &'b str,
) -> Result<Context, SemanticError<'b>> {
    let mut ctx = Context::default();

    let mut half = false;
    let mut num: i32 = 1;

    let token = res.token_by_priority(Priority(0));

    if token.is_some() {
        match token.unwrap() {
            Token::Adverbs(Adverbs::Few) => num = 3,
            Token::Adverbs(Adverbs::Half) => half = true,
            _ => (),
        }
    }

    let num = match_num(res.token_by_priority(Priority(3))).unwrap_or(num);

    if num < 0 {
        return Err(invalid_time_error(input, "number", num));
    }

    let token = res.token_by_priority(Priority(4));

    if token.is_some() {
        match token.unwrap() {
            Token::TimeInterval(TimeInterval::Second) => {
                ctx.set_duration(num);
            }
            Token::TimeInterval(TimeInterval::Minute) => ctx.set_duration(if half {
                30 * consts::SECOND
            } else {
                num * consts::MINUTE
            }),
            Token::TimeInterval(TimeInterval::Hour) => ctx.set_duration(if half {
                30 * consts::MINUTE
            } else {
                num * consts::HOUR
            }),
            Token::TimeInterval(TimeInterval::Day) => ctx.set_duration(if half {
                12 * consts::HOUR
            } else {
                num * consts::DAY
            }),
            Token::TimeInterval(TimeInterval::Week) => ctx.set_duration(if half {
                7 * 12 * consts::HOUR
            } else {
                num * consts::WEEK
            }),
            Token::TimeInterval(TimeInterval::Month) => {
                if half {
                    ctx.set_duration(14 * consts::DAY);
                } else {
                    ctx.month = Some(tz_aware.month() as i32 + num);
                }
            }
            Token::TimeInterval(TimeInterval::Year) => {
                if half {
                    let mut new_month = tz_aware.month() as i32 + 6;
                    if new_month > 12 {
                        ctx.year = Some(tz_aware.year() + 1);
                        new_month = new_month % 12;
                    }
                    ctx.month = Some(new_month);
                } else {
                    ctx.year = Some(tz_aware.year() + num);
                }
            }
            _ => (),
        }
    }

    Ok(ctx)
}

#[cfg(test)]
mod tests {
    use super::interpret;
    use crate::errors::invalid_time_error;
    use crate::{consts, MatchBounds};
    use chrono::prelude::*;

    fn fixed_time() -> DateTime<Local> {
        // 2019 1st January, Tuesday
        Local.ymd(2019, 1, 1).and_hms(0, 0, 1)
    }

    #[test]
    fn test_deadline() {
        let result = interpret("in 2 months", false, fixed_time()).unwrap();
        assert_eq!(result.get_month(), 3);

        let result = interpret("in three months", false, fixed_time()).unwrap();
        assert_eq!(result.get_month(), 4);

        let result = interpret("in a half year", false, fixed_time()).unwrap();
        assert_eq!(result.get_month(), 7);

        let result = interpret("in the few days", false, fixed_time()).unwrap();
        assert_eq!(result.get_duration_sec(), 3 * consts::DAY as i64);

        let result = interpret("in 5 minutes", false, fixed_time()).unwrap();
        assert_eq!(result.get_duration_sec(), 5 * consts::MINUTE as i64);

        let result = interpret("in 5 minutes I will go home", false, fixed_time()).unwrap();
        assert_eq!(result.get_duration_sec(), 5 * consts::MINUTE as i64);

        let result = interpret("in -3 minute", false, fixed_time());
        assert_eq!(
            result.unwrap_err().extract_error(),
            invalid_time_error("in -3 minute", "number", -3).extract_error()
        );

        let result = interpret(
            "we have to do something within 10 days.",
            false,
            fixed_time(),
        )
        .unwrap();
        assert_eq!(result.get_duration_sec(), 10 * consts::DAY as i64);

        let result = interpret(
            "we have to do something within five days.",
            false,
            fixed_time(),
        )
        .unwrap();
        assert_eq!(result.get_duration_sec(), 5 * consts::DAY as i64);

        let result = interpret("in a half year", false, fixed_time()).unwrap();
        assert_eq!(result.get_month(), 7);

        let result = interpret("drop me a line in a half hour", false, fixed_time()).unwrap();
        assert_eq!(result.get_duration_sec(), 30 * consts::MINUTE as i64);

        let result = interpret(
            "if will finish this project in the hlf of mnth",
            false,
            fixed_time(),
        )
        .unwrap();
        assert_eq!(result.get_duration_sec(), 2 * consts::WEEK as i64);

        let result = interpret(
            "if will finish this project in hlf of mnth",
            false,
            fixed_time(),
        )
        .unwrap();
        assert_eq!(result.get_duration_sec(), 2 * consts::WEEK as i64);
    }
}
