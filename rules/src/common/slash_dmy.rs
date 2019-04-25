use chrono::prelude::*;

use super::super::Context;
use super::{is_leap_year, DAYS_IN_MONTH};
use crate::tokens::{Priority, Token};
use crate::{
    rules::{MatchBounds, RuleResult},
    stub, tokenize_count_symbols, DateTimeError, TokenDesc,
};

use nom::{alt, many_till, named_args, take, tuple, types::CompleteStr};

define_num!(day: (Token::Number, Priority(0)));
define_num!(month: (Token::Number, Priority(1)));
define_num!(year: (Token::Number, Priority(2)));

define_char!(slash: Priority(10), '/');

named_args!(parse<'a>(_exact_match: bool)<CompleteStr<'a>, (Vec<usize>,
                             ( TokenDesc, TokenDesc, TokenDesc, TokenDesc, TokenDesc ) )>,

    many_till!(tokenize_count_symbols,
        alt!(
            tuple!(day, slash, month, slash, year) |
            tuple!(day, slash, month, stub, stub)
        )
    )
);

make_interpreter!(positions = 5);

fn make_time<Tz: TimeZone>(
    res: &RuleResult,
    tz_aware: DateTime<Tz>,
    input: &str,
    bounds: MatchBounds,
) -> Result<Context, DateTimeError> {
    let mut ctx = Context::default();

    let mut month = 0;
    let mut day = 0;

    let token = res.token_by_priority(Priority(0));

    if let Some(Token::Number(n)) = token {
        day = n;
    }

    let token = res.token_by_priority(Priority(1));
    if let Some(Token::Number(n)) = token {
        month = n;
    }

    let token = res.token_by_priority(Priority(2));
    let year = if let Some(Token::Number(n)) = token {
        n
    } else {
        tz_aware.year()
    };

    // only A.C. dates are supported yet
    if year <= 0 {
        return Err(DateTimeError::invalid_time_error(
            input, "year", year, bounds,
        ));
    }

    if month < 1 || month > 12 {
        return Err(DateTimeError::invalid_time_error(
            input, "month", month, bounds,
        ));
    }

    // DAYS_IN_MONTH slice counts from 0, however humans count months from 1
    // 29 days in february for leap years
    let days_in_month = if month == 2 && is_leap_year(year) {
        29
    } else {
        DAYS_IN_MONTH[month as usize - 1]
    };

    if day < 1 || day > days_in_month {
        return Err(DateTimeError::invalid_time_error(input, "day", day, bounds));
    }

    ctx.year = Some(year);
    ctx.month = Some(month);
    ctx.day = Some(day);

    Ok(ctx)
}

#[cfg(test)]
mod tests {
    use super::interpret;
    use crate::errors::DateTimeError;
    use crate::{consts, MatchBounds};
    use chrono::prelude::*;

    fn fixed_time() -> DateTime<Local> {
        // 2019 1st January, Tuesday
        Local.ymd(2019, 1, 1).and_hms(0, 0, 1)
    }

    #[test]
    fn test_slash_dmy() {
        let result = interpret("20/12/2010", false, fixed_time()).unwrap();
        assert_eq!(result.get_day(), 20);
        assert_eq!(result.get_month(), 12);
        assert_eq!(result.get_year(), 2010);

        let result = interpret("3/10", false, fixed_time()).unwrap();
        assert_eq!(result.get_day(), 3);
        assert_eq!(result.get_month(), 10);
        assert_eq!(result.get_year(), 2019);

        let result = interpret("30/2/2018", false, fixed_time());
        assert_eq!(
            result.unwrap_err(),
            DateTimeError::invalid_time_error("30/2/2018", "day", 30, MatchBounds::new(0, 9))
        );

        let result = interpret("25/13/2018", false, fixed_time());
        assert_eq!(
            result.unwrap_err(),
            DateTimeError::invalid_time_error("25/13/2018", "month", 13, MatchBounds::new(0, 10))
        );

        let result = interpret("25/10/-2", false, fixed_time());
        assert_eq!(
            result.unwrap_err(),
            DateTimeError::invalid_time_error("25/10/-2", "year", -2, MatchBounds::new(0, 8))
        );
    }
}
