use crate::{recognize_word, Token, MatchResult};
use nom::{call, named_args, types::CompleteStr};

#[derive(Debug)]
enum When {
    This,
    Last,
    Past,
    Next,
}

impl Token for When {}

named_args!(pub this<'a>(exact_match: bool)<CompleteStr<'a>, MatchResult<'a>>,
    call!(recognize_word, CompleteStr("this"), set!(max_dist=1, exact_match), &When::This)
);

named_args!(pub last<'a>(exact_match: bool)<CompleteStr<'a>, MatchResult<'a>>,
    call!(recognize_word, CompleteStr("last"), set!(max_dist=1, exact_match), &When::Last)
);

named_args!(pub past<'a>(exact_match: bool)<CompleteStr<'a>, MatchResult<'a>>,
    call!(recognize_word, CompleteStr("past"), set!(max_dist=1, exact_match), &When::Past)
);

named_args!(pub next<'a>(exact_match: bool)<CompleteStr<'a>, MatchResult<'a>>,
    call!(recognize_word, CompleteStr("next"), set!(max_dist=1, exact_match), &When::Next)
);
