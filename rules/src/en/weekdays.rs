use time::Duration;
use chrono::prelude::*;

use crate::tokens::{Token, PToken::PToken, Weekday as Day, When};
use crate::{best_fit, recognize_word, rules::RuleResult, stub, TokenDesc, match_bounds};
use crate::rules::MatchBounds;
use tuple::TupleElements;

use nom::{
    alt, apply, call, closure, eof, many_till, named, named_args, recognize, take, tuple,
    types::CompleteStr, IResult,
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

fn make_time(res: &mut RuleResult) {
    let local: DateTime<Local> = Local::now();

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

//#[test]
//fn parse_monday() {
//    println!("{:?}", apply(" sdfsd ths frday sc", false));
//}
