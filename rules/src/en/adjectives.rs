use crate::{recognize_word, best_fit, Token, MatchResult};
use nom::{call, named_args, types::CompleteStr};

#[derive(Debug)]
enum When {
    This,
    Last,
    Past,
    Next,
}

impl Token for When {}

make_token!(this, When::This, 1);
make_token!(last, When::Last, 1);
make_token!(past, When::Past, 1);
make_token!(next, When::Next, 1);

named_args!(when_adj<'a>(exact_match: bool)<CompleteStr<'a>, MatchResult<'a>>,
    call!(best_fit, exact_match, vec![&this, &last, &past, &next])
);
