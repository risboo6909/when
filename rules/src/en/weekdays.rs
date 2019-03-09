use time::Duration;
use chrono::prelude::*;

use crate::tokens::{Token, Weekday as Day, When};
use crate::{rules::RuleResult, TokenDesc, stub};
use crate::errors::DateTimeError;
use tuple::TupleElements;

use nom::{
    alt, apply, call, many_till, named_args, take, tuple, types::CompleteStr,
};

// days of week have biggest priority

define!(
    monday:
    [(Token::Weekday(Day::Monday), 0), "monday", 1] |
    [(Token::Weekday(Day::Monday), 0), "mon", 0]
);

define!(
    tuesday:
    [(Token::Weekday(Day::Tuesday), 0), "tuesday", 1] |
    [(Token::Weekday(Day::Tuesday), 0), "tue", 0]
);

define!(
    wednesday:
    [(Token::Weekday(Day::Wednesday), 0), "wednesday", 2] |
    [(Token::Weekday(Day::Wednesday), 0), "wed", 0]
);

define!(
    thursday:
    [(Token::Weekday(Day::Thursday), 0), "thursday", 2] |
    [(Token::Weekday(Day::Thursday), 0), "thur", 0]
);

define!(
    friday:
    [(Token::Weekday(Day::Friday), 0), "friday", 1] |
    [(Token::Weekday(Day::Friday), 0), "fri", 0]
);

define!(
    saturday:
    [(Token::Weekday(Day::Saturday), 0), "saturday", 2] |
    [(Token::Weekday(Day::Saturday), 0), "sat", 0]
);

define!(
    sunday:
    [(Token::Weekday(Day::Sunday), 0), "sunday", 1] |
    [(Token::Weekday(Day::Sunday), 0), "sun", 0]
);

combine!(day_of_week => monday | tuesday | wednesday | thursday | friday | saturday | sunday);

// then adjectives

define!(this: (Token::When(When::This), 1), "this", 1);
define!(last: (Token::When(When::Last), 1), "last", 1);
define!(past: (Token::When(When::Past), 1), "past", 1);
define!(next: (Token::When(When::Next), 1), "next", 1);

combine!(when => this | last | past | next);

define!(week_noun: (Token::Week, 2), "week", 1);

named_args!(parse<'a>(exact_match: bool)<CompleteStr<'a>, (Vec<CompleteStr<'a>>,
                             ( TokenDesc, TokenDesc, TokenDesc ) )>,

    many_till!(take!(1),
        alt!(
            // day of week, when, "week", e.g. "tuesday next week"
            tuple!(apply!(day_of_week, exact_match), apply!(when, exact_match),
                   apply!(week_noun, exact_match)) |
            // when and then any day of week, e.g. "last friday"
            tuple!(apply!(when, exact_match), apply!(day_of_week, exact_match), call!(stub))
        )
    )

);

fn make_time(res: &mut RuleResult, local: DateTime<Local>, input: &str) {
    let mut offset = 0;
    let mut day = 0;

    // if we are here, tokens MUST be Some
    let tokens = res.tokens.as_mut().unwrap();

    for token in tokens {
        match token {
            Token::Weekday(Day::Monday) => {
                day = 0;
            },
            Token::Weekday(Day::Tuesday) => {
                day = 1;
            },
            Token::Weekday(Day::Wednesday) => {
                day = 2;
            },
            Token::Weekday(Day::Thursday) => {
                day = 3;
            },
            Token::Weekday(Day::Friday) => {
                day = 4;
            },
            Token::Weekday(Day::Saturday) => {
                day = 5;
            },
            Token::Weekday(Day::Sunday) => {
                day = 6;
            },
            Token::When(When::Next) => {
                let delta = day - local.weekday() as i64;
                if delta > 0 {
                    offset = Duration::days(delta).num_seconds();
                } else {
                    offset = Duration::days(7 + delta).num_seconds();
                }
            },
            Token::When(When::Last) | Token::When(When::Past) => {
                let delta = local.weekday() as i64 - day;
                if delta > 0 {
                    offset = -Duration::days(delta).num_seconds();
                } else {
                    offset = -Duration::days(7 + delta).num_seconds();
                }
            },
            Token::When(When::This) => {
                let weekday_i64 = local.weekday() as i64;
                let delta = day - weekday_i64;

                if weekday_i64 < day {
                    offset = Duration::days(delta).num_seconds();
                } else if weekday_i64 > day {
                    // what did user mean? previous week day or this week day or next
                    // week day? we don't know!
                    res.time_shift = Err(DateTimeError::AmbiguousTime {
                        msg: input.to_string(),
                    });
                } else {
                    offset = 0;
                }
            },
            Token::Week => {},
            _ => unreachable!(),
        }
    }

    if res.time_shift.is_ok() {
        res.time_shift.as_mut().unwrap().offset = offset;
    }

}

make_interpreter!(indices[0, 1, 2]);

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
    fn test_past_last() {
        let mut result = interpret("do it for the past Monday", false, fixed_time());
        assert_eq!(result.tokens, Some(vec![Token::Weekday(Day::Monday), Token::When(When::Past)]));
        assert_eq!(result.bounds, Some(MatchBounds { start_idx: 14, end_idx: 24 }));
        assert_eq!(result.get_offset(), -86400);

        result = interpret("past saturday", false, fixed_time());
        assert_eq!(result.tokens, Some(vec![Token::Weekday(Day::Saturday), Token::When(When::Past)]));
        assert_eq!(result.bounds, Some(MatchBounds { start_idx: 0, end_idx: 12 }));
        assert_eq!(result.get_offset(), -259200);

        result = interpret("pst frday", false, fixed_time());
        assert_eq!(result.tokens, Some(vec![Token::Weekday(Day::Friday), Token::When(When::Past)]));
        assert_eq!(result.bounds, Some(MatchBounds { start_idx: 0, end_idx: 8 }));
        assert_eq!(result.get_offset(), -345600);

        result = interpret("pat thrday", false, fixed_time());
        assert_eq!(result.tokens, Some(vec![Token::Weekday(Day::Thursday), Token::When(When::Past)]));
        assert_eq!(result.bounds, Some(MatchBounds { start_idx: 0, end_idx: 9 }));
        assert_eq!(result.get_offset(), -432000);

        result = interpret("past wednesday", true, fixed_time());
        assert_eq!(result.tokens, Some(vec![Token::Weekday(Day::Wednesday), Token::When(When::Past)]));
        assert_eq!(result.bounds, Some(MatchBounds { start_idx: 0, end_idx: 13 }));
        assert_eq!(result.get_offset(), -518400);

        result = interpret("past tuesday", true, fixed_time());
        assert_eq!(result.tokens, Some(vec![Token::Weekday(Day::Tuesday), Token::When(When::Past)]));
        assert_eq!(result.bounds, Some(MatchBounds { start_idx: 0, end_idx: 11 }));
        assert_eq!(result.get_offset(), -604800);

        result = interpret("lst monday", false, fixed_time());
        assert_eq!(result.tokens, Some(vec![Token::Weekday(Day::Monday), Token::When(When::Last)]));
        assert_eq!(result.bounds, Some(MatchBounds { start_idx: 0, end_idx: 9 }));
        assert_eq!(result.get_offset(), -86400);
    }

    #[test]
    fn test_next() {
        let mut result = interpret("next monday", false, fixed_time());
        assert_eq!(result.tokens, Some(vec![Token::Weekday(Day::Monday), Token::When(When::Next)]));
        assert_eq!(result.bounds, Some(MatchBounds { start_idx: 0, end_idx: 10 }));
        assert_eq!(result.get_offset(), 518400);

        let mut result = interpret("drop me a line at next wednesday", false, fixed_time());
        assert_eq!(result.tokens, Some(vec![Token::Weekday(Day::Wednesday), Token::When(When::Next)]));
        assert_eq!(result.bounds, Some(MatchBounds { start_idx: 18, end_idx: 31 }));
        assert_eq!(result.get_offset(), 86400);
    }

    #[test]
    fn test_this() {
        let mut result = interpret("drop me a line at this monday", false, fixed_time());
        assert_eq!(result.tokens, Some(vec![Token::Weekday(Day::Monday), Token::When(When::This)]));
        assert_eq!(result.time_shift, Err(AmbiguousTime
            {msg: "drop me a line at this monday".to_string()}));

        let result = interpret("this friday", false, fixed_time());
        assert_eq!(result.tokens, Some(vec![Token::Weekday(Day::Friday), Token::When(When::This)]));
        assert_eq!(result.bounds, Some(MatchBounds { start_idx: 0, end_idx: 10 }));
        assert_eq!(result.get_offset(), 259200);
    }
}
