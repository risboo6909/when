use chrono::prelude::*;

use super::super::Context;
use crate::errors::{invalid_time_error, SemanticError};
use crate::tokens::{AmPm, Priority, Token};
use crate::{rules::RuleResult, stub, tokenize_count_symbols, Dist, TokenDesc};
use nom::{alt, apply, call, many_till, named_args, tuple, types::CompleteStr};

define_num!(hours: (Token::Number, Priority(0)));

define_char!(colon: Priority(1), ':');
define_char!(dash: Priority(1), '-');

define_num!(minutes: (Token::Number, Priority(2)));

define!(
    am:
    [(Token::AmPm(AmPm::Am), Priority(3)), "a.m.", Dist(0)] |
    [(Token::AmPm(AmPm::Am), Priority(3)), "a.m", Dist(0)] |
    [(Token::AmPm(AmPm::Am), Priority(3)), "am.", Dist(0)] |
    [(Token::AmPm(AmPm::Am), Priority(3)), "am", Dist(0)] |
    [(Token::AmPm(AmPm::Am), Priority(3)), "a.", Dist(0)] |
    [(Token::AmPm(AmPm::Am), Priority(3)), "a", Dist(0)]
);

define!(
    pm:
    [(Token::AmPm(AmPm::Am), Priority(3)), "p.m.", Dist(0)] |
    [(Token::AmPm(AmPm::Am), Priority(3)), "p.m", Dist(0)] |
    [(Token::AmPm(AmPm::Am), Priority(3)), "pm.", Dist(0)] |
    [(Token::AmPm(AmPm::Pm), Priority(3)), "pm", Dist(0)] |
    [(Token::AmPm(AmPm::Pm), Priority(3)), "p.", Dist(0)] |
    [(Token::AmPm(AmPm::Pm), Priority(3)), "p", Dist(0)]
);

combine!(am_pm => am | pm);

named_args!(parse<'a>(exact_match: bool)<CompleteStr<'a>, (Vec<usize>,
                             ( TokenDesc, TokenDesc, TokenDesc, TokenDesc ) )>,
    many_till!(tokenize_count_symbols,
        alt!(
            // hours:minutes am/pm, for example 5:30am, 4:44pm, etc.
            tuple!(hours, colon, minutes, apply!(am_pm, exact_match)) |
            // hours:minutes am/pm, for example 5-30am, 4-44pm, etc.
            tuple!(hours, dash, minutes, apply!(am_pm, exact_match)) |
            // hours:minutes, for example 5:30, 4:44, etc.
            tuple!(hours, colon, minutes, stub) |
            // hours-minutes, for example 5-30, 4-44, etc.
            tuple!(hours, dash, minutes, stub) |
            // hours, for example 5am, 6p.m., 4a., 3 p.m.
            tuple!(hours, apply!(am_pm, exact_match), stub, stub)
        )
    )
);

make_interpreter!(positions = 4);

fn make_time<'a, 'b, Tz: TimeZone>(
    res: &'a RuleResult,
    _tz_aware: DateTime<Tz>,
    input: &'b str,
) -> Result<Context, SemanticError<'b>> {
    let mut ctx = Context::default();
    let mut hrs: i32 = 0;

    let token = res.token_by_priority(Priority(0));
    if let Some(Token::Number(n)) = token {
        hrs = n as i32;
    }

    let token = res.token_by_priority(Priority(2));
    if let Some(Token::Number(minutes)) = token {
        if minutes > 59 {
            return Err(invalid_time_error(input, "minutes", minutes));
        }

        if hrs <= 23 {
            ctx.minute = Some(minutes);
        } else {
            return Err(invalid_time_error(input, "hours", hrs));
        }
    } else {
        ctx.minute = Some(0);
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
    use crate::errors::invalid_time_error;
    use crate::MatchBounds;
    use chrono::prelude::*;

    fn fixed_time() -> DateTime<Local> {
        Local.ymd(2019, 1, 1).and_hms(0, 0, 0)
    }

    #[test]
    fn test_hours_pm() {
        let result = interpret("5pm", false, fixed_time()).unwrap();
        assert_eq!(result.bounds, Some(MatchBounds::new(0, 3)));
        assert_eq!(result.get_hours(), 17);

        let result = interpret("at 5 pm", false, fixed_time()).unwrap();
        assert_eq!(result.bounds, Some(MatchBounds::new(3, 7)));
        assert_eq!(result.get_hours(), 17);

        let result = interpret("at 12 p.", false, fixed_time()).unwrap();
        assert_eq!(result.bounds, Some(MatchBounds::new(3, 8)));

        assert_eq!(result.get_hours(), 0);
        let result = interpret("at 11p.m.", false, fixed_time()).unwrap();
        assert_eq!(result.bounds, Some(MatchBounds::new(3, 9)));
        assert_eq!(result.get_hours(), 23);
    }

    #[test]
    fn test_hours_am() {
        let result = interpret("5am", false, fixed_time()).unwrap();
        assert_eq!(result.bounds, Some(MatchBounds::new(0, 3)));
        assert_eq!(result.get_hours(), 5);

        let result = interpret("at 5 a.m.", false, fixed_time()).unwrap();
        assert_eq!(result.bounds, Some(MatchBounds::new(3, 9)));
        assert_eq!(result.get_hours(), 5);

        let result = interpret("at 12 a.", false, fixed_time()).unwrap();
        assert_eq!(result.bounds, Some(MatchBounds::new(3, 8)));
        assert_eq!(result.get_hours(), 12);
    }

    #[test]
    fn test_with_minutes() {
        let result = interpret("5:30am", false, fixed_time()).unwrap();
        assert_eq!(result.bounds, Some(MatchBounds::new(0, 6)));
        assert_eq!(result.get_hours(), 5);
        assert_eq!(result.get_minutes(), 30);

        let result = interpret("5:59 pm", false, fixed_time()).unwrap();
        assert_eq!(result.bounds, Some(MatchBounds::new(0, 7)));
        assert_eq!(result.get_hours(), 17);
        assert_eq!(result.get_minutes(), 59);

        let result = interpret("17-59 pm", false, fixed_time()).unwrap();
        assert_eq!(result.bounds, Some(MatchBounds::new(0, 8)));
        assert_eq!(result.get_hours(), 17);
        assert_eq!(result.get_minutes(), 59);

        let result = interpret("up to 11-10 pm", false, fixed_time()).unwrap();
        assert_eq!(result.bounds, Some(MatchBounds::new(6, 14)));
        assert_eq!(result.get_hours(), 23);
        assert_eq!(result.get_minutes(), 10);

        let result = interpret("it is 10:30 o'clock", false, fixed_time()).unwrap();
        assert_eq!(result.bounds, Some(MatchBounds::new(6, 11)));
        assert_eq!(result.get_hours(), 10);
        assert_eq!(result.get_minutes(), 30);
    }

    #[test]
    fn test_wrong_times() {
        let result = interpret("24:10", false, fixed_time());
        assert_eq!(
            result.unwrap_err().extract_error(),
            invalid_time_error("24:10", "hours", 24).extract_error()
        );

        let result = interpret("12:60", false, fixed_time());
        assert_eq!(
            result.unwrap_err().extract_error(),
            invalid_time_error("12:60", "minutes", 60).extract_error()
        );
    }

}
