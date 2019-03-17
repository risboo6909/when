use time::Duration;
use chrono::prelude::*;

use crate::tokens::{Token, When, Priority, Pronouns, TimeOfDay};
use crate::{best_fit, recognize_word, rules::RuleResult, stub, TokenDesc, Dist};
use crate::rules::MatchBounds;

use nom::{
    alt, apply, call, many_till, named_args, take, tuple, types::CompleteStr
};

define!(
    last:
    [(Token::When(When::Last), Priority(0)), "last", Dist(2)]
);

define!(
    now:
    [(Token::When(When::Now), Priority(0)), "now", Dist(0)]
);

define!(
    today:
    [(Token::When(When::Today), Priority(0)), "today", Dist(1)]
);

define!(
    tonight:
    [(Token::When(When::Tonight), Priority(0)), "tonight", Dist(2)]
);

define!(
    tomorrow:
    [(Token::When(When::Tomorrow), Priority(0)), "tomorrow", Dist(2)] |
    [(Token::When(When::Tomorrow), Priority(0)), "tmr", Dist(0)]
);

define!(
    yesterday:
    [(Token::When(When::Yesterday), Priority(0)), "yesterday", Dist(2)]
);

combine!(when => today | tonight | yesterday | tomorrow);

define!(
    this:
    [(Token::Pronouns(Pronouns::This), Priority(0)), "this", Dist(1)]
);

define!(
    night:
    [(Token::TimeOfDay(TimeOfDay::Night), Priority(0)), "night", Dist(1)]
);

define!(
    morning:
    [(Token::TimeOfDay(TimeOfDay::Morning), Priority(0)), "morning", Dist(2)]
);

define!(
    evening:
    [(Token::TimeOfDay(TimeOfDay::Evening), Priority(0)), "evening", Dist(2)]
);

define!(
    noon:
    [(Token::TimeOfDay(TimeOfDay::Noon), Priority(0)), "noon", Dist(1)]
);

combine!(time_of_day => night | morning | evening | noon);

named_args!(parse<'a>(exact_match: bool)<CompleteStr<'a>, (Vec<CompleteStr<'a>>,
                             ( TokenDesc, TokenDesc, TokenDesc ) )>,
    many_till!(take!(1),
        alt!(
            tuple!(apply!(when, exact_match), apply!(this, exact_match),
                   apply!(time_of_day, exact_match)) |
            tuple!(apply!(when, exact_match), apply!(time_of_day, exact_match), call!(stub)) |
            tuple!(apply!(this, exact_match), apply!(time_of_day, exact_match), call!(stub)) |
            tuple!(apply!(when, exact_match), call!(stub), call!(stub)) |
            tuple!(apply!(now, exact_match), call!(stub), call!(stub))
        )
    )
);

make_interpreter!(indices[0, 1, 2]);

fn make_time(res: &mut RuleResult, local: DateTime<Local>, input: &str) {}

#[cfg(test)]
mod tests {
    use chrono::prelude::*;
    use crate::tokens::{Token, When, Priority, Pronouns, TimeOfDay};
    use crate::MatchBounds;
    use super::interpret;

    fn fixed_time() -> DateTime<Local> {
        Local.ymd(2019, 1, 1).and_hms(0, 0, 0)
    }

    #[test]
    fn test_casual_date() {

    }
}