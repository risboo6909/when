use crate::{best_fit, recognize_word, tokens::Tokens, TokenDesc};
use nom::{call, named_args, types::CompleteStr};

define!(week_noun, Tokens::Week, "week", 1);
