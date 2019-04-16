use chrono::prelude::*;

use super::{is_leap_year, DAYS_IN_MONTH};
use crate::errors::DateTimeError;
use crate::tokens::{Adverbs, Articles, IntWord, Priority, TimeInterval, Token, When};
use crate::{consts, rules::RuleResult, stub, Dist, TokenDesc};

use nom::{alt, apply, call, many_till, named_args, take, tuple, types::CompleteStr};

define_num!(day: (Token::Number, Priority(0)));
define_num!(month: (Token::Number, Priority(1)));
define_num!(year: (Token::Number, Priority(2)));

define_char!(slash: Priority(10), '/');

named_args!(parse<'a>(exact_match: bool)<CompleteStr<'a>, (Vec<CompleteStr<'a>>,
                             ( TokenDesc, TokenDesc, TokenDesc, TokenDesc, TokenDesc ) )>,

    many_till!(take!(1),
        alt!(
            tuple!(day, slash, month, slash, year) |
            tuple!(day, slash, month, stub, stub)
        )
    )
);

make_interpreter!(positions = 5);

fn make_time(res: &mut RuleResult, local: DateTime<Local>, input: &str) {
    let mut year: Option<i32> = None;
    let mut month = 0;
    let mut day = 0;

    let token = res.token_by_priority(Priority(0));
    token.map_or((), |t| match t {
        Token::Number(n) => day = n,
        _ => unreachable!(),
    });

    let token = res.token_by_priority(Priority(1));
    token.map_or((), |t| match t {
        Token::Number(n) => month = n,
        _ => unreachable!(),
    });

    let token = res.token_by_priority(Priority(2));
    token.map_or((), |t| match t {
        Token::Number(n) => year = Some(n),
        _ => unreachable!(),
    });

    let year_int = match year {
        Some(n) => n,
        None => local.year(),
    };

    // only A.C. dates are supported yet
    if year_int <= 0 {
        res.set_error(DateTimeError::InvalidTime {
            msg: input.to_string(),
            what: "year".to_owned(),
            value: year_int,
        });
        return;
    }

    if month < 1 || month > 12 {
        res.set_error(DateTimeError::InvalidTime {
            msg: input.to_string(),
            what: "month".to_owned(),
            value: month,
        });
        return;
    }

    // DAYS_IN_MONTH slice counts from 0, however humans count months from 1
    let mut days_in_month = DAYS_IN_MONTH[month as usize - 1];

    // 29 days in february for leap years
    if month == 2 && is_leap_year(year_int) {
        days_in_month = 29;
    }

    if day < 1 || day > days_in_month {
        res.set_error(DateTimeError::InvalidTime {
            msg: input.to_string(),
            what: "day".to_owned(),
            value: day,
        });
        return;
    }

    res.set_year(year_int);
    res.set_month(month);
    res.set_day(day);
}

#[cfg(test)]
mod tests {
    use super::interpret;
    use crate::errors::DateTimeError::{AmbiguousTime, InvalidTime};
    use crate::{consts, MatchBounds};
    use chrono::prelude::*;

    fn fixed_time() -> DateTime<Local> {
        // 2019 1st January, Tuesday
        Local.ymd(2019, 1, 1).and_hms(0, 0, 1)
    }

    #[test]
    fn test_slash_dmy() {
        let result = interpret("20/12/2010", false, fixed_time());
        assert_eq!(result.get_day(), 20);
        assert_eq!(result.get_month(), 12);
        assert_eq!(result.get_year(), 2010);

        let result = interpret("3/10", false, fixed_time());
        assert_eq!(result.get_day(), 3);
        assert_eq!(result.get_month(), 10);
        assert_eq!(result.get_year(), 2019);

        let result = interpret("30/2/2018", false, fixed_time());
        assert_eq!(
            result.context,
            Err(InvalidTime {
                msg: "30/2/2018".to_owned(),
                what: "day".to_owned(),
                value: 30,
            })
        );

        let result = interpret("25/13/2018", false, fixed_time());
        assert_eq!(
            result.context,
            Err(InvalidTime {
                msg: "25/13/2018".to_owned(),
                what: "month".to_owned(),
                value: 13,
            })
        );

        let result = interpret("25/10/-2", false, fixed_time());
        assert_eq!(
            result.context,
            Err(InvalidTime {
                msg: "25/10/-2".to_owned(),
                what: "year".to_owned(),
                value: -2,
            })
        );
    }
}
