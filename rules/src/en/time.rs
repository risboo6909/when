use chrono::prelude::*;

use super::super::Context;
use crate::errors::DateTimeError;
use crate::tokens::{AmPm, Priority, Token};
use crate::{rules::RuleResult, stub, Dist, TokenDesc};
use nom::{alt, apply, call, many_till, named_args, take, tuple, types::CompleteStr};

define_num!(hours: (Token::Number, Priority(0)));

define_char!(colon: Priority(1), ':');
define_char!(dash: Priority(1), '-');

define_num!(minutes: (Token::Number, Priority(2)));

define!(
    am:
    [(Token::AmPm(AmPm::Am), Priority(3)), "am", Dist(1)] |
    [(Token::AmPm(AmPm::Am), Priority(3)), "a", Dist(0)]
);

define!(
    pm:
    [(Token::AmPm(AmPm::Pm), Priority(3)), "pm", Dist(1)] |
    [(Token::AmPm(AmPm::Pm), Priority(3)), "p", Dist(0)]
);

combine!(am_pm => am | pm);

named_args!(parse<'a>(exact_match: bool)<CompleteStr<'a>, (Vec<CompleteStr<'a>>,
                             ( TokenDesc, TokenDesc, TokenDesc, TokenDesc ) )>,
    many_till!(take!(1),
        alt!(
            // hours:minutes am/pm, for example 5:30am, 4:44pm, etc.
            tuple!(hours, colon, minutes, apply!(am_pm, exact_match)) |
            // hours:minutes am/pm, for example 5-30am, 4-44pm, etc.
            tuple!(hours, dash, minutes, apply!(am_pm, exact_match)) |
            // hours:minutes am/pm, for example 5:30, 4:44, etc.
            tuple!(hours, colon, minutes, stub) |
            // hours:minutes am/pm, for example 5-30, 4-44, etc.
            tuple!(hours, dash, minutes, stub) |
            // hours, for example 5am, 6p.m., 4a., 3 p.m.
            tuple!(hours, apply!(am_pm, exact_match), stub, stub)
        )
    )
);

make_interpreter!(positions = 4);

fn make_time<Tz: TimeZone>(
    res: &RuleResult,
    _tz_aware: DateTime<Tz>,
    input: &str,
) -> Result<Context, DateTimeError> {
    let mut ctx = Context::default();
    let mut hrs: i32 = 0;

    let token = res.token_by_priority(Priority(0));
    if let Some(Token::Number(n)) = token {
        hrs = n as i32;
    }

    let token = res.token_by_priority(Priority(2));
    if let Some(Token::Number(minutes)) = token {
        if minutes > 59 {
            return Err(DateTimeError::InvalidTime {
                msg: input.to_string(),
                what: "minutes".to_string(),
                value: minutes,
            });
        }

        if hrs <= 23 {
            ctx.minute = Some(minutes);
        } else {
            return Err(DateTimeError::InvalidTime {
                msg: input.to_string(),
                what: "hours".to_string(),
                value: hrs,
            });
        }
    }

    let token = res.token_by_priority(Priority(3));
    if let Some(Token::AmPm(AmPm::Pm)) = token {
        if hrs <= 12 {
            // TODO: Ensure correctness
            hrs = (hrs + 12) % 24;
        }
    }

    ctx.hour = Some(hrs);

    Ok(ctx)
}

#[cfg(test)]
mod tests {
    use super::interpret;
    use crate::errors::DateTimeError::InvalidTime;
    use crate::MatchBounds;
    use chrono::prelude::*;

    fn fixed_time() -> DateTime<Local> {
        Local.ymd(2019, 1, 1).and_hms(0, 0, 0)
    }

    #[test]
    fn test_hours_pm() {
        let result = interpret("5pm", false, fixed_time()).unwrap();
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 0,
                end_idx: 2,
            })
        );
        assert_eq!(result.get_hours(), 17);

        let result = interpret("at 5 pm", false, fixed_time()).unwrap();
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 3,
                end_idx: 6,
            })
        );
        assert_eq!(result.get_hours(), 17);

        let result = interpret("at 12 p.", false, fixed_time()).unwrap();
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 3,
                end_idx: 7,
            })
        );

        assert_eq!(result.get_hours(), 0);
        let result = interpret("at 11p.m.", false, fixed_time()).unwrap();
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 3,
                end_idx: 8,
            })
        );
        assert_eq!(result.get_hours(), 23);
    }

    #[test]
    fn test_hours_am() {
        let result = interpret("5am", false, fixed_time()).unwrap();
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 0,
                end_idx: 2,
            })
        );
        assert_eq!(result.get_hours(), 5);

        let result = interpret("at 5 a.m.", false, fixed_time()).unwrap();
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 3,
                end_idx: 8,
            })
        );
        assert_eq!(result.get_hours(), 5);

        let result = interpret("at 12 a.", false, fixed_time()).unwrap();
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 3,
                end_idx: 7,
            })
        );
        assert_eq!(result.get_hours(), 12);
    }

    #[test]
    fn test_with_minutes() {
        let result = interpret("5:30am", false, fixed_time()).unwrap();
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 0,
                end_idx: 5,
            })
        );
        assert_eq!(result.get_hours(), 5);
        assert_eq!(result.get_minutes(), 30);

        let result = interpret("5:59 pm", false, fixed_time()).unwrap();
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 0,
                end_idx: 6,
            })
        );
        assert_eq!(result.get_hours(), 17);
        assert_eq!(result.get_minutes(), 59);

        let result = interpret("17-59 pm", false, fixed_time()).unwrap();
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 0,
                end_idx: 7,
            })
        );
        assert_eq!(result.get_hours(), 17);
        assert_eq!(result.get_minutes(), 59);

        let result = interpret("up to 11-10 pm", false, fixed_time()).unwrap();
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 6,
                end_idx: 13,
            })
        );
        assert_eq!(result.get_hours(), 23);
        assert_eq!(result.get_minutes(), 10);

        let result = interpret("it is 10:30 o'clock", false, fixed_time()).unwrap();
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 6,
                end_idx: 10,
            })
        );
        assert_eq!(result.get_hours(), 10);
        assert_eq!(result.get_minutes(), 30);
    }

    #[test]
    fn test_wrong_times() {
        let result = interpret("24:10", false, fixed_time());
        assert_eq!(
            result.unwrap_err(),
            InvalidTime {
                msg: "24:10".to_string(),
                what: "hours".to_string(),
                value: 24,
            }
        );

        let result = interpret("12:60", false, fixed_time());
        assert_eq!(
            result.unwrap_err(),
            InvalidTime {
                msg: "12:60".to_string(),
                what: "minutes".to_string(),
                value: 60,
            }
        );
    }

}
