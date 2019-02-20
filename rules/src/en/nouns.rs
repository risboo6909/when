use crate::{best_fit, recognize_word, MatchResult, tokens::Tokens};
use nom::{call, named_args, types::CompleteStr};

define!(week_noun, Tokens::Week, "week", 2);
