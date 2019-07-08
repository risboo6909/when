use super::combinators::{stub, tokenize_count_symbols, Dist};
use super::errors::{ambiguous_time_error, SemanticError};
use super::tokens::{Priority, Token, Weekday as Day, When};
use super::types::{Context, RuleResult, TokenDesc};
use chrono::prelude::*;
use time::Duration;

use nom::{alt, apply, call, many_till, named_args, tuple, types::CompleteStr};

// days of week have biggest priority

define!(
    monday:
    [(Token::Weekday(Day::Monday), Priority(0)), "monday", Dist(1)] |
    [(Token::Weekday(Day::Monday), Priority(0)), "mon", Dist(0)]
);

define!(
    tuesday:
    [(Token::Weekday(Day::Tuesday), Priority(0)), "tuesday", Dist(1)] |
    [(Token::Weekday(Day::Tuesday), Priority(0)), "tue", Dist(0)]
);

define!(
    wednesday:
    [(Token::Weekday(Day::Wednesday), Priority(0)), "wednesday", Dist(2)] |
    [(Token::Weekday(Day::Wednesday), Priority(0)), "wed", Dist(0)]
);

define!(
    thursday:
    [(Token::Weekday(Day::Thursday), Priority(0)), "thursday", Dist(2)] |
    [(Token::Weekday(Day::Thursday), Priority(0)), "thur", Dist(0)]
);

define!(
    friday:
    [(Token::Weekday(Day::Friday), Priority(0)), "friday", Dist(1)] |
    [(Token::Weekday(Day::Friday), Priority(0)), "fri", Dist(0)]
);

define!(
    saturday:
    [(Token::Weekday(Day::Saturday), Priority(0)), "saturday", Dist(1)] |
    [(Token::Weekday(Day::Saturday), Priority(0)), "sat", Dist(0)]
);

define!(
    sunday:
    [(Token::Weekday(Day::Sunday), Priority(0)), "sunday", Dist(1)] |
    [(Token::Weekday(Day::Sunday), Priority(0)), "sun", Dist(0)]
);

combine!(day_of_week => monday | tuesday | wednesday | thursday | friday | saturday | sunday);

// then adjectives

define!(this: (Token::When(When::This), Priority(1)), "this", Dist(1));
define!(last: (Token::When(When::Last), Priority(1)), "last", Dist(1));
define!(past: (Token::When(When::Past), Priority(1)), "past", Dist(1));
define!(next: (Token::When(When::Next), Priority(1)), "next", Dist(1));

combine!(when => this | last | past | next);

define!(week_noun: (Token::Week, Priority(2)), "week", Dist(1));

named_args!(parse<'a>(exact_match: bool)<CompleteStr<'a>, (Vec<usize>,
                             ( TokenDesc, TokenDesc, TokenDesc ) )>,
    many_till!(tokenize_count_symbols,
        alt!(
            // day of week, when, "week", e.g. "tuesday next week"
            tuple!(apply!(day_of_week, exact_match), apply!(when, exact_match),
                   apply!(week_noun, exact_match)) |
            // when and then any day of week, e.g. "last friday"
            tuple!(apply!(when, exact_match), apply!(day_of_week, exact_match), stub) |
            // day of week
            tuple!(apply!(day_of_week, exact_match), stub, stub)
        )
    )
);

make_interpreter!(positions = 3);

fn make_time<'a, 'b, Tz: TimeZone>(
    res: &'a RuleResult,
    tz_aware: DateTime<Tz>,
    input: &'b str,
) -> Result<Context, SemanticError<'b>> {
    let mut ctx = Context::default();

    let mut day = 0;

    let token = res.token_by_priority(Priority(0));

    token.map_or((), |t| match t {
        Token::Weekday(Day::Monday) => {
            day = 0;
        }
        Token::Weekday(Day::Tuesday) => {
            day = 1;
        }
        Token::Weekday(Day::Wednesday) => {
            day = 2;
        }
        Token::Weekday(Day::Thursday) => {
            day = 3;
        }
        Token::Weekday(Day::Friday) => {
            day = 4;
        }
        Token::Weekday(Day::Saturday) => {
            day = 5;
        }
        Token::Weekday(Day::Sunday) => {
            day = 6;
        }
        _ => (),
    });

    let mut token = res.token_by_priority(Priority(1));

    if token.is_none() {
        token = Some(Token::When(When::This));
    }

    if token.is_some() {
        match token.unwrap() {
            Token::When(When::Next) => {
                let delta = day - tz_aware.weekday() as i64;
                if delta > 0 {
                    ctx.set_duration(Duration::days(delta).num_seconds());
                } else {
                    ctx.set_duration(Duration::days(7 + delta).num_seconds());
                }
            }
            Token::When(When::Last) | Token::When(When::Past) => {
                let delta = tz_aware.weekday() as i64 - day;
                if delta > 0 {
                    ctx.set_duration(-Duration::days(delta).num_seconds());
                } else {
                    ctx.set_duration(-Duration::days(7 + delta).num_seconds());
                }
            }
            Token::When(When::This) => {
                let weekday = tz_aware.weekday() as i64;
                if weekday <= day {
                    ctx.set_duration(Duration::days(day - weekday).num_seconds());
                } else {
                    // what did user mean? previous week day or this week day or next
                    // week day? we don't know!
                    return Err(ambiguous_time_error(input));
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
    use crate::rules::errors::ambiguous_time_error;
    use crate::rules::types::MatchBounds;
    use chrono::prelude::*;

    fn fixed_time() -> DateTime<Local> {
        // 2019 1st January, Tuesday
        Local.ymd(2019, 1, 1).and_hms(0, 0, 1)
    }

    #[test]
    fn test_past_last() {
        let result = interpret("do it for the past Monday", false, fixed_time()).unwrap();
        assert_eq!(result.bounds, Some(MatchBounds::new(14, 25)));
        assert_eq!(result.get_duration_sec(), -86400);

        let result = interpret("past saturday", false, fixed_time()).unwrap();
        assert_eq!(result.bounds, Some(MatchBounds::new(0, 13)));
        assert_eq!(result.get_duration_sec(), -259200);

        let result = interpret("pst frday", false, fixed_time()).unwrap();
        assert_eq!(result.bounds, Some(MatchBounds::new(0, 9)));
        assert_eq!(result.get_duration_sec(), -345600);

        let result = interpret("pat thrday", false, fixed_time()).unwrap();
        assert_eq!(result.bounds, Some(MatchBounds::new(0, 10)));
        assert_eq!(result.get_duration_sec(), -432000);

        let result = interpret("past wednesday", true, fixed_time()).unwrap();
        assert_eq!(result.bounds, Some(MatchBounds::new(0, 14)));
        assert_eq!(result.get_duration_sec(), -518400);

        let result = interpret("past tuesday", true, fixed_time()).unwrap();
        assert_eq!(result.bounds, Some(MatchBounds::new(0, 12)));
        assert_eq!(result.get_duration_sec(), -604800);

        let result = interpret("lst monday", false, fixed_time()).unwrap();
        assert_eq!(result.bounds, Some(MatchBounds::new(0, 10)));
        assert_eq!(result.get_duration_sec(), -86400);
    }

    #[test]
    fn test_next() {
        let result = interpret("next monday", false, fixed_time()).unwrap();
        assert_eq!(result.bounds, Some(MatchBounds::new(0, 11)));
        assert_eq!(result.get_duration_sec(), 518400);

        let result = interpret("drop me a line at next wednesday", false, fixed_time()).unwrap();
        assert_eq!(result.bounds, Some(MatchBounds::new(18, 32)));
        assert_eq!(result.get_duration_sec(), 86400);
    }

    #[test]
    fn test_this() {
        let result = interpret("drop me a line at this monday", false, fixed_time());
        assert_eq!(
            result.unwrap_err().extract_error(),
            ambiguous_time_error("this monday").extract_error()
        );

        let result = interpret("this friday", false, fixed_time()).unwrap();
        assert_eq!(result.bounds, Some(MatchBounds::new(0, 11)));
        assert_eq!(result.get_duration_sec(), 259200);
    }
}
