use time::Duration;
use chrono::prelude::*;

use crate::tokens::{Token, When};
use crate::{rules::RuleResult, TokenDesc};
use tuple::TupleElements;

use nom::{
    alt, apply, call, many_till, named_args, take, tuple, types::CompleteStr
};

define_num!(hour, (Token::Hour, 0), 0, 12);

define!(
    am:
    [(Token::When(When::AM), 0), "a.m.", 0] |
    [(Token::When(When::AM), 0), "a.", 0] |
    [(Token::When(When::AM), 0), "am", 0]
);

define!(
    pm:
    [(Token::When(When::PM), 0), "p.m.", 0] |
    [(Token::When(When::PM), 0), "p.", 0] |
    [(Token::When(When::PM), 0), "pm", 0]
);

combine!(when => am | pm);

named_args!(parse<'a>(exact_match: bool)<CompleteStr<'a>, (Vec<CompleteStr<'a>>,
                             ( TokenDesc, TokenDesc) )>,

    many_till!(take!(1),
        // time (hours), for example 5am, 6p.m., 4a, 3 p.m.
        tuple!(hour, apply!(when, exact_match))
    )

);

fn make_time(res: &mut RuleResult, local: DateTime<Local>, input: &str) {

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
    fn test_am() {
        let result = interpret("5am", false, fixed_time());
        assert_eq!(result.tokens, Some(vec![Token::Hour(5), Token::When(When::AM)]));
        assert_eq!(result.bounds, Some(MatchBounds { start_idx: 0, end_idx: 2 }));
    //    assert_eq!(result.ts, 1546203600);
    }
}
