use crate::{best_fit, recognize_word, Token, MatchResult};
use nom::{call, named_args, types::CompleteStr};

#[derive(Debug)]
enum Week {
    Week,
}

impl Token for Week {}

define!(week, Week::Week, "week", 2);

combine!(week_noun => week);
