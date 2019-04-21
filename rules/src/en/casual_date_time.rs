use super::super::Context;
use crate::errors::DateTimeError;
use crate::tokens::{Priority, Pronouns, TimeOfDay, Token, When};
use crate::{consts, rules::RuleResult, stub, Dist, TokenDesc};
use chrono::prelude::*;

use nom::{alt, apply, call, many_till, named_args, take, tuple, types::CompleteStr};

define!(now: (Token::When(When::Now), Priority(0)), "now", Dist(0));

define!(last: (Token::When(When::Last), Priority(1)), "last", Dist(1));
define!(this: (Token::Pronouns(Pronouns::This), Priority(1)), "this", Dist(1));

combine!(last_this => last | this);

define!(today: (Token::When(When::Today), Priority(1)), "today", Dist(1));
define!(tonight: (Token::When(When::Tonight), Priority(1)), "tonight", Dist(2));
define!(
    tomorrow:
    [(Token::When(When::Tomorrow), Priority(1)), "tomorrow", Dist(2)] |
    [(Token::When(When::Tomorrow), Priority(1)), "tmr", Dist(0)]
);

define!(yesterday: (Token::When(When::Yesterday), Priority(1)), "yesterday", Dist(2));
combine!(when => today | tonight | yesterday | tomorrow);

define!(night: (Token::TimeOfDay(TimeOfDay::Night), Priority(2)), "night", Dist(1));
define!(morning: (Token::TimeOfDay(TimeOfDay::Morning), Priority(2)), "morning", Dist(2));
define!(evening: (Token::TimeOfDay(TimeOfDay::Evening), Priority(2)), "evening", Dist(2));
define!(noon: (Token::TimeOfDay(TimeOfDay::Noon), Priority(2)), "noon", Dist(1));

combine!(time_of_day => night | morning | evening | noon);

named_args!(parse<'a>(exact_match: bool)<CompleteStr<'a>, (Vec<CompleteStr<'a>>,
                             ( TokenDesc, TokenDesc, ) )>,
    many_till!(take!(1),
        alt!(
            // last night, this morning, etc.
            tuple!(apply!(last_this, exact_match), apply!(time_of_day, exact_match)) |
            // tomorrow evening, today morning, etc.
            tuple!(apply!(when, exact_match), apply!(time_of_day, exact_match)) |
            // today, tomorrow, yesterday, etc.
            tuple!(apply!(when, exact_match), stub) |
            // now
            tuple!(apply!(now, exact_match), stub)
        )
    )
);

make_interpreter!(positions = 2);

fn make_time<Tz: TimeZone>(
    res: &RuleResult,
    _tz_aware: DateTime<Tz>,
    _input: &str,
) -> Result<Context, DateTimeError> {
    let mut ctx = Context::default();
    let token = res.token_by_priority(Priority(1));

    if token.is_some() {
        match token.unwrap() {
            Token::When(When::Last) => {
                ctx.hour = 23;
                ctx.set_duration(-24 * i64::from(consts::HOUR));
            }
            Token::When(When::Tomorrow) => {
                ctx.set_duration(24 * i64::from(consts::HOUR));
            }
            Token::When(When::Yesterday) => {
                ctx.set_duration(-24 * i64::from(consts::HOUR));
            }
            Token::When(When::Tonight) => {
                ctx.hour = 23;
                ctx.minute = 0;
            }
            _ => (),
        }
    }

    let token = res.token_by_priority(Priority(2));

    if token.is_some() {
        match token.unwrap() {
            Token::TimeOfDay(TimeOfDay::Morning) => {
                ctx.hour = 8;
                ctx.minute = 0;
            }
            Token::TimeOfDay(TimeOfDay::Noon) => {
                ctx.hour = 12;
                ctx.minute = 0;
            }
            Token::TimeOfDay(TimeOfDay::Evening) => {
                ctx.hour = 18;
                ctx.minute = 0;
            }
            Token::TimeOfDay(TimeOfDay::Night) => {
                ctx.hour = 23;
                ctx.minute = 0;
            }
            _ => (),
        }
    }

    Ok(ctx)
}

#[cfg(test)]
mod tests {
    use super::interpret;
    use crate::tokens::{Priority, Pronouns, TimeOfDay, Token, When};
    use crate::{consts, MatchBounds};
    use chrono::prelude::*;

    fn fixed_time() -> DateTime<Local> {
        Local.ymd(2019, 1, 1).and_hms(0, 0, 0)
    }

    #[test]
    fn test_casual_date() {
        let result = interpret("The deadline is now, ok", false, fixed_time()).unwrap();
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 16,
                end_idx: 18
            })
        );
        assert_eq!(result.get_duration_sec(), 0);

        let result = interpret("The deadline is today", false, fixed_time()).unwrap();
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 16,
                end_idx: 20
            })
        );
        assert_eq!(result.get_duration_sec(), 0);

        let result = interpret("The deadline is tonight", false, fixed_time()).unwrap();
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 16,
                end_idx: 22
            })
        );
        assert_eq!(result.get_hours(), 23);
        assert_eq!(result.get_minutes(), 0);

        let result = interpret("The deadline is tomorrow", false, fixed_time()).unwrap();
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 16,
                end_idx: 23
            })
        );
        assert_eq!(result.get_duration_sec(), 24 * consts::HOUR as i64);

        let result = interpret("The deadline was yesterday", false, fixed_time()).unwrap();
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 17,
                end_idx: 25
            })
        );
        assert_eq!(result.get_duration_sec(), -24 * consts::HOUR as i64);

        let result = interpret("Please call me tomorrow evening", false, fixed_time()).unwrap();
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 15,
                end_idx: 30
            })
        );
        assert_eq!(result.get_duration_sec(), 24 * consts::HOUR as i64);
        assert_eq!(result.get_hours(), 18);

        let result = interpret("He told me that yesterday morning", false, fixed_time()).unwrap();
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 16,
                end_idx: 32
            })
        );
        assert_eq!(result.get_duration_sec(), -24 * consts::HOUR as i64);
        assert_eq!(result.get_hours(), 8);

        let result = interpret("Last night I fell asleep", false, fixed_time()).unwrap();
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 0,
                end_idx: 9
            })
        );
        assert_eq!(result.get_duration_sec(), -24 * consts::HOUR as i64);
        assert_eq!(result.get_hours(), 23);
    }
}
