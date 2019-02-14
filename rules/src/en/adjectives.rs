use crate::{best_fit, recognize_word, Token, MatchResult};
use nom::{call, named_args, types::CompleteStr};

#[derive(Debug)]
enum When {
    This,
    Last,
    Past,
    Next,
}

impl Token for When {}

define!(this, When::This, "this", 1);
define!(last, When::Last, "last", 1);
define!(past, When::Past, "past", 1);
define!(next, When::Next, "next", 1);

combine!(when => this, last, past, next);
