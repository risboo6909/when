use chrono::prelude::*;

use crate::tokens::{AmPm, Priority, Token, When};
use crate::{consts, rules::RuleResult, stub, Dist, TokenDesc};
use nom::{alt, apply, call, many_till, named_args, take, tuple, types::CompleteStr};

define_num!(hours: (Token::Number, Priority(0)), 0, 24);

define_char!(colon: Priority(1), ':');

define_num!(minutes: (Token::Number, Priority(2)), 0, 60);

define!(
    am:
    [(Token::AmPm(AmPm::Am), Priority(3)), "a.m.", Dist(0)] |
    [(Token::AmPm(AmPm::Am), Priority(3)), "a.", Dist(0)] |
    [(Token::AmPm(AmPm::Am), Priority(3)), "am", Dist(0)]
);

define!(
    pm:
    [(Token::AmPm(AmPm::Pm), Priority(3)), "p.m.", Dist(0)] |
    [(Token::AmPm(AmPm::Pm), Priority(3)), "p.", Dist(0)] |
    [(Token::AmPm(AmPm::Pm), Priority(3)), "pm", Dist(0)]
);

combine!(am_pm => am | pm);

named_args!(parse<'a>(exact_match: bool)<CompleteStr<'a>, (Vec<CompleteStr<'a>>,
                             ( TokenDesc, TokenDesc, TokenDesc, TokenDesc ) )>,
    many_till!(take!(1),
        alt!(
            // hours:minutes am/pm, for example 5:30am, 4:44pm, etc.
            tuple!(apply!(hours, true), colon, apply!(minutes, true), apply!(am_pm, exact_match)) |
            // hours, for example 5am, 6p.m., 4a., 3 p.m.
            tuple!(apply!(hours, true), apply!(am_pm, exact_match), stub, stub)
        )
    )
);

fn make_time(res: &mut RuleResult, _local: DateTime<Local>, _input: &str) {
    let token = res.token_by_priority(Priority(0));
    let mut hrs = 0;

    if let Some(Token::Number(n)) = token {
        hrs = n;
    }

    let token = res.token_by_priority(Priority(2));
    if let Some(Token::Number(n)) = token {
        res.unwrap_mut().minute = n;
    }

    let token = res.token_by_priority(Priority(3));

    token.map_or((), |t| match t {
        Token::AmPm(AmPm::Pm) => {
            hrs += 12;
        }
        Token::AmPm(AmPm::Am) => {}
        _ => (),
    });

    res.unwrap_mut().hour = hrs * consts::HOUR;
}

make_interpreter!(indices[0, 1, 2, 3]);

#[cfg(test)]
mod tests {
    use super::interpret;
    use crate::MatchBounds;
    use chrono::prelude::*;

    fn fixed_time() -> DateTime<Local> {
        Local.ymd(2019, 1, 1).and_hms(0, 0, 0)
    }

    #[test]
    fn test_hours_pm() {
        let result = interpret("5pm", false, fixed_time());
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 0,
                end_idx: 2
            })
        );
        assert_eq!(result.get_hours(), 61200);

        let result = interpret("at 5 pm", false, fixed_time());
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 3,
                end_idx: 6
            })
        );
        assert_eq!(result.get_hours(), 61200);

        let result = interpret("at 12 p.", false, fixed_time());
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 3,
                end_idx: 7
            })
        );
        assert_eq!(result.get_hours(), 86400);
    }

    #[test]
    fn test_hours_am() {
        let result = interpret("5am", false, fixed_time());
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 0,
                end_idx: 2
            })
        );
        assert_eq!(result.get_hours(), 18000);

        let result = interpret("at 5 a.m.", false, fixed_time());
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 3,
                end_idx: 8
            })
        );
        assert_eq!(result.get_hours(), 18000);

        let result = interpret("at 12 a.", false, fixed_time());
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 3,
                end_idx: 7
            })
        );
        assert_eq!(result.get_hours(), 43200);
    }

    #[test]
    fn test_time_pm() {
        let result = interpret("5:30am", false, fixed_time());
        println!("++ {:?}", result);
        assert_eq!(
            result.bounds,
            Some(MatchBounds {
                start_idx: 0,
                end_idx: 5
            })
        );
        // assert_eq!(result.get_hours(), 5);
    }

}
