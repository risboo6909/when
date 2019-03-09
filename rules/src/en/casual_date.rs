use time::Duration;
use chrono::prelude::*;

use crate::tokens::{Token, When};
use crate::{best_fit, recognize_word, rules::RuleResult, stub, TokenDesc};
use crate::rules::MatchBounds;
use tuple::TupleElements;

use nom::{
    alt, apply, call, many_till, named_args, take, tuple, types::CompleteStr
};

define!(
    now:
    [(Token::When(When::Now), 0), "now", 0]
);

define!(
    today:
    [(Token::When(When::Today), 0), "today", 1]
);

define!(
    tonight:
    [(Token::When(When::Tonight), 0), "tonight", 2]
);

define!(
    tomorrow:
    [(Token::When(When::Tomorrow), 0), "tomorrow", 2] |
    [(Token::When(When::Tomorrow), 0), "tmr", 0]
);

define!(
    yesterday:
    [(Token::When(When::Yesterday), 0), "yesterday", 2]
);
