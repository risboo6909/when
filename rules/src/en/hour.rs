use chrono::prelude::*;

use crate::tokens::{Token, When, Priority};
use crate::{rules::RuleResult, TokenDesc, Dist, consts};

use nom::{
    alt, apply, call, many_till, named_args, take, tuple, types::CompleteStr
};

define_num!(hour, (Token::Number, Priority(0)), 0, 12);

define!(
    am:
    [(Token::When(When::AM), Priority(1)), "a.m.", Dist(0)] |
    [(Token::When(When::AM), Priority(1)), "a.", Dist(0)] |
    [(Token::When(When::AM), Priority(1)), "am", Dist(0)]
);

define!(
    pm:
    [(Token::When(When::PM), Priority(1)), "p.m.", Dist(0)] |
    [(Token::When(When::PM), Priority(1)), "p.", Dist(0)] |
    [(Token::When(When::PM), Priority(1)), "pm", Dist(0)]
);

combine!(when => am | pm);

named_args!(parse<'a>(exact_match: bool)<CompleteStr<'a>, (Vec<CompleteStr<'a>>,
                             ( TokenDesc, TokenDesc) )>,

    many_till!(take!(1),
        // time (hours), for example 5am, 6p.m., 4a., 3 p.m.
        tuple!(apply!(hour, true), apply!(when, exact_match))
    )

);

fn make_time(res: &mut RuleResult, _local: DateTime<Local>, _input: &str) {

    let token = res.token_by_priority(Priority(0));

    let mut hrs = 0;

    match token.unwrap_or(&Token::None) {
        Token::Number(n) => {
            hrs = *n;
        },
        _ => (),
    }

    let token = res.token_by_priority(Priority(1));

    match token.unwrap_or(&Token::None) {
        Token::When(When::PM) => {
            hrs += 12;
        },
        Token::When(When::AM) => {},
        _ => (),
    }

    res.unwrap_ctx().hour = hrs * consts::HOUR;

}

make_interpreter!(indices[0, 1]);

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
    fn test_pm() {
        let mut result = interpret("5pm", false, fixed_time());
        assert_eq!(result.bounds, Some(MatchBounds { start_idx: 0, end_idx: 2 }));
        assert_eq!(result.get_hours(), 61200);

        result = interpret("at 5 pm", false, fixed_time());
        assert_eq!(result.bounds, Some(MatchBounds { start_idx: 3, end_idx: 6 }));
        assert_eq!(result.get_hours(), 61200);

        result = interpret("at 12 p.", false, fixed_time());
        assert_eq!(result.bounds, Some(MatchBounds { start_idx: 3, end_idx: 7 }));
        assert_eq!(result.get_hours(), 86400);
    }

    #[test]
    fn test_am() {
        let mut result = interpret("5am", false, fixed_time());
        assert_eq!(result.bounds, Some(MatchBounds { start_idx: 0, end_idx: 2 }));
        assert_eq!(result.get_hours(), 18000);

        result = interpret("at 5 a.m.", false, fixed_time());
        assert_eq!(result.bounds, Some(MatchBounds { start_idx: 3, end_idx: 8 }));
        assert_eq!(result.get_hours(), 18000);

        result = interpret("at 12 a.", false, fixed_time());
        assert_eq!(result.bounds, Some(MatchBounds { start_idx: 3, end_idx: 7 }));
        assert_eq!(result.get_hours(), 43200);
    }

}
