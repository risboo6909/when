use super::combinators::{Dist, stub, tokenize_count_symbols};
use super::errors::{DateTimeError, SemanticError, invalid_time_error};
use super::rules::{Context, MatchResult, MyResult, RuleResult, TokenDesc};
use super::tokens::{AmPm, Priority, Token};
use chrono::prelude::*;
use nom::{Parser, branch::alt, multi::many_till};

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
    [(Token::AmPm(AmPm::Pm), Priority(3)), "p.m.", Dist(0)] |
    [(Token::AmPm(AmPm::Pm), Priority(3)), "p.m", Dist(0)] |
    [(Token::AmPm(AmPm::Pm), Priority(3)), "pm.", Dist(0)] |
    [(Token::AmPm(AmPm::Pm), Priority(3)), "pm", Dist(0)] |
    [(Token::AmPm(AmPm::Pm), Priority(3)), "p.", Dist(0)] |
    [(Token::AmPm(AmPm::Pm), Priority(3)), "p", Dist(0)]
);

combine!(am_pm => am | pm);

define!(at: (Token::Char, Priority(1)), "at", Dist(0));

fn whole_hours(input: &str) -> MyResult<'_> {
    let (tail, token) = hours(input)?;
    // Keep identifiers, dates, decimal values and incomplete clock forms intact.
    if tail
        .chars()
        .next()
        .is_some_and(|c| c.is_alphanumeric() || matches!(c, '_' | ':' | '/' | '-' | '+'))
        || tail
            .strip_prefix('.')
            .and_then(|rest| rest.chars().next())
            .is_some_and(char::is_numeric)
    {
        return super::combinators::wrap_error(input);
    }
    Ok((tail, token))
}

fn parse_at_hour(
    input: &str,
    _exact_match: bool,
) -> nom::IResult<&str, (Vec<usize>, [TokenDesc; 4])> {
    many_till(
        tokenize_count_symbols,
        (|input| at(input, true), whole_hours, stub, stub),
    )
    .map(|(skipped, (t0, t1, t2, t3))| (skipped, [t0, t1, t2, t3]))
    .parse(input)
}

make_interpreter!(interpret_at_hour, parse_at_hour);

fn parse(input: &str, exact_match: bool) -> nom::IResult<&str, (Vec<usize>, [TokenDesc; 4])> {
    many_till(
        tokenize_count_symbols,
        alt((
            // hours:minutes am/pm, for example 5:30am, 4:44pm, etc.
            (hours, colon, minutes, |input| am_pm(input, exact_match)),
            // hours:minutes am/pm, for example 5-30am, 4-44pm, etc.
            (hours, dash, minutes, |input| am_pm(input, exact_match)),
            // hours:minutes, for example 5:30, 4:44, etc.
            (hours, colon, minutes, stub),
            // hours-minutes, for example 5-30, 4-44, etc.
            (hours, dash, minutes, stub),
            // hours, for example 5am, 6p.m., 4a., 3 p.m.
            (hours, |input| am_pm(input, exact_match), stub, stub),
        )),
    )
    .map(|(skipped, (t0, t1, t2, t3))| (skipped, [t0, t1, t2, t3]))
    .parse(input)
}

make_interpreter!();

/// Recognize a standalone hour only after structured rules found no matches.
pub(super) fn interpret_hour_only<Tz: TimeZone>(
    input: &str,
    tz_aware: DateTime<Tz>,
) -> Option<Result<MatchResult, DateTimeError>> {
    let trimmed = input.trim();
    if trimmed.is_empty() || !trimmed.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }

    let (_, hour) = hours(trimmed).ok()?;
    let mut res = RuleResult::new();
    res.set_token(&hour);
    let start = input.len() - input.trim_start().len();
    Some(
        make_time(&res, tz_aware, trimmed)
            .map(|ctx| MatchResult::new(ctx, start, start + trimmed.len()))
            .map_err(|error| error.extract_error()),
    )
}

fn make_time<'a, 'b, Tz: TimeZone>(
    res: &'a RuleResult,
    _tz_aware: DateTime<Tz>,
    input: &'b str,
) -> Result<Context, SemanticError<'b>> {
    let mut ctx = Context::default();
    let mut hrs: i32 = 0;

    let token = res.token_by_priority(Priority(0));
    if let Some(Token::Number(n)) = token {
        hrs = n;
    }
    if !(0..=23).contains(&hrs) {
        return Err(invalid_time_error(input, "hours", hrs));
    }

    let token = res.token_by_priority(Priority(2));
    if let Some(Token::Number(minutes)) = token {
        if !(0..=59).contains(&minutes) {
            return Err(invalid_time_error(input, "minutes", minutes));
        }

        ctx.minute = Some(minutes);
    } else {
        ctx.minute = Some(0);
    }

    hrs = match res.token_by_priority(Priority(3)) {
        Some(Token::AmPm(AmPm::Am)) if hrs == 12 => 0,
        Some(Token::AmPm(AmPm::Pm)) if hrs < 12 => hrs + 12,
        _ => hrs,
    };

    ctx.hour = Some(hrs);

    Ok(ctx)
}

#[cfg(test)]
mod tests {
    use super::interpret;
    use crate::rules::errors::invalid_time_error;
    use crate::rules::rules::MatchBounds;
    use chrono::prelude::*;

    fn fixed_time() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2019, 1, 1, 0, 0, 0).unwrap()
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

        assert_eq!(result.get_hours(), 12);
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
        assert_eq!(result.get_hours(), 0);
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
