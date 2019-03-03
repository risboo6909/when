use time::Duration;
use chrono::prelude::*;

use crate::tokens::{Token, PToken::PToken, Weekday as Day, When};
use crate::{best_fit, recognize_word, rules::RuleResult, stub, TokenDesc, match_bounds};
use crate::rules::MatchBounds;
use tuple::TupleElements;

use nom::{
    alt, apply, call, many_till, named_args, take, tuple, types::CompleteStr
};

//define!(this, PToken(Token::When(When::This), 0), "this", 1);
//define!(last, PToken(Token::When(When::Last), 0), "last", 1);
//define!(past, PToken(Token::When(When::Past), 0), "past", 1);
//define!(next, PToken(Token::When(When::Next), 0), "next", 1);