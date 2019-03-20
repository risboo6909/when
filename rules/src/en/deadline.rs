use chrono::prelude::*;

use crate::tokens::{Adverbs, Articles, Token, TimeInterval, IntWord, When, Priority};
use crate::{rules::RuleResult, TokenDesc, Dist, stub, consts};
use crate::common_matchers::match_num;

use super::common::int_word;

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

define_num!(number, (Token::Number, Priority(3)), 0, 12);

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

    let mut half = false;
    let mut num: i64 = 1;

    let token = res.token_by_priority(Priority(0));

    match token.unwrap_or(&Token::None) {
        Token::Adverbs(Adverbs::Few) => num = 3,
        Token::Adverbs(Adverbs::Half) => half = true,
        _ => (),
    }

    let num = match_num(res.token_by_priority(Priority(3))).unwrap_or(num);

    let token = res.token_by_priority(Priority(4));

    match token.unwrap_or(&Token::None) {
        Token::TimeInterval(TimeInterval::Second) => {
            res.unwrap_mut().duration = num;
        },
        Token::TimeInterval(TimeInterval::Minute) => {
            res.unwrap_mut().duration = if half {
                30 * consts::SECOND as i64
            } else {
                num * consts::MINUTE as i64
            }
        },
        Token::TimeInterval(TimeInterval::Hour) => {
            res.unwrap_mut().duration = if half {
                30 * consts::MINUTE as i64
            } else {
                num * consts::HOUR as i64
            }
        },
        Token::TimeInterval(TimeInterval::Day) => {
            res.unwrap_mut().duration = if half {
                12 * consts::HOUR as i64
            } else {
                num * consts::DAY as i64
            }
        },
        Token::TimeInterval(TimeInterval::Week) => {
            res.unwrap_mut().duration = if half {
                7 * 12 * consts::HOUR as i64
            } else {
                num * consts::WEEK as i64
            }
        },
        Token::TimeInterval(TimeInterval::Month) => {
            if half {
                res.unwrap_mut().duration = 14 * consts::DAY as i64;
            } else {
                res.unwrap_mut().month =
                    ((local.month() as i64 + num) % 12) as usize;
            }
        },
        Token::TimeInterval(TimeInterval::Year) => {
            if half {
                res.unwrap_mut().month =
                    ((local.month() as i64 + 6) % 12) as usize;
            } else {
                res.unwrap_mut().year =
                    (local.year() as i64 + num) as usize;
            }
        },
        _ => (),
    }

}


#[cfg(test)]
mod tests {
    use chrono::prelude::*;
    use crate::{MatchBounds, consts};
    use super::interpret;
    use crate::errors::DateTimeError::AmbiguousTime;

    fn fixed_time() -> DateTime<Local> {
        // 2019 1st January, Tuesday
        Local.ymd(2019, 1, 1).and_hms(0, 0, 1)
    }

    #[test]
    fn test_deadline() {
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

        let result = interpret("we have to do something within 10 days.", false, fixed_time());
        assert_eq!(result.context.unwrap().duration, 10 * consts::DAY as i64);

        let result = interpret("we have to do something within five days.", false, fixed_time());
        assert_eq!(result.context.unwrap().duration, 5 * consts::DAY as i64);

        let result = interpret("in a half year", false, fixed_time());
        assert_eq!(result.context.unwrap().month, 7);

        let result = interpret("drop me a line in a half hour", false, fixed_time());
        assert_eq!(result.context.unwrap().duration, 30 * consts::MINUTE as i64);

    }
}
