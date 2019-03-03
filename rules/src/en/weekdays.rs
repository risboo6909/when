use time::Duration;
use chrono::prelude::*;

use crate::tokens::{Token, PToken::PToken, Weekday as Day, When};
use crate::{best_fit, recognize_word, rules::RuleResult, stub, TokenDesc, match_bounds, MatchBounds};
use tuple::TupleElements;

use nom::{
    alt, apply, call, many_till, named_args, take, tuple, types::CompleteStr
};

// days of week have biggest priority

define!(
    monday,
    [PToken(Token::Weekday(Day::Monday), 0), "monday", 1],
    [PToken(Token::Weekday(Day::Monday), 0), "mon", 0]
);

define!(
    tuesday,
    [PToken(Token::Weekday(Day::Tuesday), 0), "tuesday", 1],
    [PToken(Token::Weekday(Day::Tuesday), 0), "tue", 0]
);

define!(
    wednesday,
    [PToken(Token::Weekday(Day::Wednesday), 0), "wednesday", 2],
    [PToken(Token::Weekday(Day::Wednesday), 0), "wed", 0]
);

define!(
    thursday,
    [PToken(Token::Weekday(Day::Thursday), 0), "thursday", 2],
    [PToken(Token::Weekday(Day::Thursday), 0), "thur", 0]
);

define!(
    friday,
    [PToken(Token::Weekday(Day::Friday), 0), "friday", 1],
    [PToken(Token::Weekday(Day::Friday), 0), "fri", 0]
);

define!(
    saturday,
    [PToken(Token::Weekday(Day::Saturday), 0), "saturday", 2],
    [PToken(Token::Weekday(Day::Saturday), 0), "sat", 0]
);

define!(
    sunday,
    [PToken(Token::Weekday(Day::Sunday), 0), "sunday", 1],
    [PToken(Token::Weekday(Day::Sunday), 0), "sun", 0]
);

combine!(day_of_week => monday, tuesday, wednesday, thursday, friday, saturday, sunday);

// then adjectives

define!(this, PToken(Token::When(When::This), 1), "this", 1);
define!(last, PToken(Token::When(When::Last), 1), "last", 1);
define!(past, PToken(Token::When(When::Past), 1), "past", 1);
define!(next, PToken(Token::When(When::Next), 1), "next", 1);

combine!(when => this, last, past, next);

define!(week_noun, PToken(Token::Week, 2), "week", 1);

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

fn make_time(res: &mut RuleResult, local: DateTime<Local>) {
    let mut ts = local.timestamp();
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
                    ts += Duration::days(delta).num_seconds();
                } else {
                    ts += Duration::days(7+delta).num_seconds();
                }
            },
            Token::When(When::Last) | Token::When(When::Past) => {
                let delta = local.weekday() as i64 - day;
                if delta > 0 {
                    ts -= Duration::days(delta).num_seconds();
                } else {
                    ts -= Duration::days(7+delta).num_seconds();
                }
            },
            Token::When(When::This) => {
                if (local.weekday() as i64) < day {
                    let delta = day - local.weekday() as i64;
                    if delta > 0 {
                        ts += Duration::days(delta).num_seconds();
                    } else {
                        ts += Duration::days(7+delta).num_seconds();
                    }
                } else {
                    let delta = local.weekday() as i64 - day;
                    if delta > 0 {
                        ts -= Duration::days(delta).num_seconds();
                    } else {
                        ts -= Duration::days(7+delta).num_seconds();
                    }
                }
            },
            _ => {},
        }
    }

    res.set_time(ts as usize);

}

interpreter!(indices[0, 1, 2]);

#[cfg(test)]
mod tests {
    use chrono::prelude::*;
    use crate::tokens::{Token, Weekday as Day, When};
    use crate::MatchBounds;
    use super::interpret;

    fn fixed_time() -> DateTime<Local> {
        Local.ymd(2019, 1, 1).and_hms(0, 0, 0)
    }

    #[test]
    fn test_past_last() {
        let mut result = interpret("do it for the past Monday", false, fixed_time());
        assert_eq!(result.tokens, Some(vec![Token::Weekday(Day::Monday), Token::When(When::Past)]));
        assert_eq!(result.bounds, Some(MatchBounds { start_idx: 14, end_idx: 24 }));
        assert_eq!(result.ts, 1546203600);

        result = interpret("past saturday", false, fixed_time());
        assert_eq!(result.tokens, Some(vec![Token::Weekday(Day::Saturday), Token::When(When::Past)]));
        assert_eq!(result.bounds, Some(MatchBounds { start_idx: 0, end_idx: 12 }));
        assert_eq!(result.ts, 1546030800);

        result = interpret("pst frday", false, fixed_time());
        assert_eq!(result.tokens, Some(vec![Token::Weekday(Day::Friday), Token::When(When::Past)]));
        assert_eq!(result.bounds, Some(MatchBounds { start_idx: 0, end_idx: 8 }));
        assert_eq!(result.ts, 1545944400);

        result = interpret("pat thrday", false, fixed_time());
        assert_eq!(result.tokens, Some(vec![Token::Weekday(Day::Thursday), Token::When(When::Past)]));
        assert_eq!(result.bounds, Some(MatchBounds { start_idx: 0, end_idx: 9 }));
        assert_eq!(result.ts, 1545858000);

        result = interpret("past wednesday", true, fixed_time());
        assert_eq!(result.tokens, Some(vec![Token::Weekday(Day::Wednesday), Token::When(When::Past)]));
        assert_eq!(result.bounds, Some(MatchBounds { start_idx: 0, end_idx: 13 }));
        assert_eq!(result.ts, 1545771600);

        result = interpret("past tuesday", true, fixed_time());
        assert_eq!(result.tokens, Some(vec![Token::Weekday(Day::Tuesday), Token::When(When::Past)]));
        assert_eq!(result.bounds, Some(MatchBounds { start_idx: 0, end_idx: 11 }));
        assert_eq!(result.ts, 1545685200);

        result = interpret("lst monday", false, fixed_time());
        assert_eq!(result.tokens, Some(vec![Token::Weekday(Day::Monday), Token::When(When::Last)]));
        assert_eq!(result.bounds, Some(MatchBounds { start_idx: 0, end_idx: 9 }));
        assert_eq!(result.ts, 1546203600);
    }

    #[test]
    fn test_next() {
        let mut result = interpret("next monday", false, fixed_time());
        assert_eq!(result.tokens, Some(vec![Token::Weekday(Day::Monday), Token::When(When::Next)]));
        assert_eq!(result.bounds, Some(MatchBounds { start_idx: 0, end_idx: 10 }));
        assert_eq!(result.ts, 1546808400);

        let mut result = interpret("drop me a line at next wednesday", false, fixed_time());
        assert_eq!(result.tokens, Some(vec![Token::Weekday(Day::Wednesday), Token::When(When::Next)]));
        assert_eq!(result.bounds, Some(MatchBounds { start_idx: 18, end_idx: 31 }));
        assert_eq!(result.ts, 1546376400);
    }

    #[test]
    fn test_this() {
        let mut result = interpret("drop me a line at this monday", false, fixed_time());
        assert_eq!(result.tokens, Some(vec![Token::Weekday(Day::Monday), Token::When(When::This)]));
        assert_eq!(result.bounds, Some(MatchBounds { start_idx: 18, end_idx: 28 }));
        assert_eq!(result.ts, 1546203600);

        let mut result = interpret("this friday", false, fixed_time());
        assert_eq!(result.tokens, Some(vec![Token::Weekday(Day::Friday), Token::When(When::This)]));
        assert_eq!(result.bounds, Some(MatchBounds { start_idx: 0, end_idx: 10 }));
        assert_eq!(result.ts, 1546549200);
    }

}
