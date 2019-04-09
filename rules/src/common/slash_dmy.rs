use chrono::prelude::*;

use crate::common_matchers::match_num;
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

fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && year % 100 != 0 || year % 400 == 0
}

fn make_time(res: &mut RuleResult, local: DateTime<Local>, _input: &str) {
    let token = res.token_by_priority(Priority(0));
    let mut year = 0;
    token.map_or((), |t| match t {
        Token::Number(n) => year = n,
        _ => unreachable!(),
    });
}

#[cfg(test)]
mod tests {
    use super::interpret;
    use crate::errors::DateTimeError::AmbiguousTime;
    use crate::{consts, MatchBounds};
    use chrono::prelude::*;

    fn fixed_time() -> DateTime<Local> {
        // 2019 1st January, Tuesday
        Local.ymd(2019, 1, 1).and_hms(0, 0, 1)
    }

    #[test]
    fn test_slash_dmy() {
        let result = interpret("20/12/10", false, fixed_time());
        println!("{:?}", result);
        //assert_eq!(result.get_month(), 3);
    }
}
