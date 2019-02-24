use crate::tokens::{Tokens, When};
use crate::{best_fit, recognize_word, TokenDesc};
use nom::{call, named_args, types::CompleteStr};

define!(this, Tokens::When(When::This), "this", 1);
define!(last, Tokens::When(When::Last), "last", 1);
define!(past, Tokens::When(When::Past), "past", 1);
define!(next, Tokens::When(When::Next), "next", 1);

combine!(when => this, last, past, next);
