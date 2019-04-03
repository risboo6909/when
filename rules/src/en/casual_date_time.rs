use chrono::prelude::*;
use time::Duration;

use crate::tokens::{Priority, Pronouns, TimeOfDay, Token, When};
use crate::{
    consts,
    rules::{MatchBounds, RuleResult},
    stub, Dist, TokenDesc,
};

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

make_interpreter!(indices[0, 1]);

fn make_time(res: &mut RuleResult, _local: DateTime<Local>, _input: &str) {
    let token = res.token_by_priority(Priority(1));

    token.map_or((), |t| match t {
        Token::When(When::Last) => {
            res.set_hour(23u32);
            res.unwrap_mut().duration -= 24 * consts::HOUR as i64;
        }
        Token::When(When::Tomorrow) => {
            res.unwrap_mut().duration += 24 * consts::HOUR as i64;
        }
        Token::When(When::Yesterday) => {
            res.unwrap_mut().duration -= 24 * consts::HOUR as i64;
        }
        Token::When(When::Tonight) => {
            res.set_hour(23u32);
            res.set_minute(0u32);
        }
        _ => (),
    });

    let token = res.token_by_priority(Priority(2));

    token.map_or((), |t| match t {
        Token::TimeOfDay(TimeOfDay::Morning) => {
            res.set_hour(8u32);
            res.set_minute(0u32);
        }
        Token::TimeOfDay(TimeOfDay::Noon) => {
            res.set_hour(12u32);
            res.set_minute(0u32);
        }
        Token::TimeOfDay(TimeOfDay::Evening) => {
            res.set_hour(18u32);
            res.set_minute(0u32);
        }
        Token::TimeOfDay(TimeOfDay::Night) => {
            res.set_hour(23u32);
            res.set_minute(0u32);
        }
        _ => (),
    });
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
        let result = interpret("The deadline is now, ok", false, fixed_time());
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 16,
                end_idx: 18
            })
        );
        assert_eq!(result.unwrap().duration, 0);

        let result = interpret("The deadline is today", false, fixed_time());
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 16,
                end_idx: 20
            })
        );
        assert_eq!(result.unwrap().duration, 0);

        let result = interpret("The deadline is tonight", false, fixed_time());
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 16,
                end_idx: 22
            })
        );
        assert_eq!(result.unwrap().hour, 23);
        assert_eq!(result.unwrap().minute, 0);

        let result = interpret("The deadline is tomorrow", false, fixed_time());
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 16,
                end_idx: 23
            })
        );
        assert_eq!(result.unwrap().duration, 24 * consts::HOUR as i64);

        let result = interpret("The deadline was yesterday", false, fixed_time());
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 17,
                end_idx: 25
            })
        );
        assert_eq!(result.unwrap().duration, -24 * consts::HOUR as i64);

        let result = interpret("Please call me tomorrow evening", false, fixed_time());
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 15,
                end_idx: 30
            })
        );
        assert_eq!(result.unwrap().duration, 24 * consts::HOUR as i64);
        assert_eq!(result.unwrap().hour, 18);

        let result = interpret("He told me that yesterday morning", false, fixed_time());
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 16,
                end_idx: 32
            })
        );
        assert_eq!(result.unwrap().duration, -24 * consts::HOUR as i64);
        assert_eq!(result.unwrap().hour, 8);

        let result = interpret("Last night I fell asleep", false, fixed_time());
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 0,
                end_idx: 9
            })
        );
        assert_eq!(result.unwrap().duration, -24 * consts::HOUR as i64);
        assert_eq!(result.unwrap().hour, 23);
    }
}
