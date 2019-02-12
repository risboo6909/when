use crate::{recognize_word, Token};
use nom::{call, named_args, types::CompleteStr};

#[derive(Debug)]
enum Nouns {
    Week,
}

impl Token for When {}

named_args!(pub week<'a>(exact_match: bool)<CompleteStr<'a>, MatchResult<'a>>,
    call!(recognize_word, CompleteStr("week"), set!(max_dist=1, exact_match), &Nouns::Week)
);
