use crate::{recognize_word, best_fit, Token, MatchResult};
use super::adjectives::{this, last, past, next};

use nom::{apply, alt, call, closure, named_args, named, many_till, take_s, tuple, eof, IResult,
          types::CompleteStr};

#[derive(Debug)]
enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}


impl Token for Weekday {}

named_args!(monday<'a>(exact_match: bool)<CompleteStr<'a>, MatchResult<'a>>,
    alt!(
        call!(recognize_word, CompleteStr("monday"), set!(max_dist=2, exact_match),
            &Weekday::Monday) |
        call!(recognize_word, CompleteStr("mon"), set!(max_dist=1, exact_match),
            &Weekday::Monday)
    )
);

named_args!(tuesday<'a>(exact_match: bool)<CompleteStr<'a>, MatchResult<'a>>,
    alt!(
        call!(recognize_word, CompleteStr("tuesday"), set!(max_dist=2, exact_match),
            &Weekday::Tuesday) |
        call!(recognize_word, CompleteStr("tue"), set!(max_dist=1, exact_match),
            &Weekday::Tuesday)
    )
);

named_args!(wednesday<'a>(exact_match: bool)<CompleteStr<'a>, MatchResult<'a>>,
    alt!(
        call!(recognize_word, CompleteStr("wednesday"), set!(max_dist=2, exact_match),
            &Weekday::Wednesday) |
        call!(recognize_word, CompleteStr("wed"), set!(max_dist=1, exact_match),
            &Weekday::Wednesday)
    )
);

named_args!(thursday<'a>(exact_match: bool)<CompleteStr<'a>, MatchResult<'a>>,
    alt!(
        call!(recognize_word, CompleteStr("thursday"), set!(max_dist=2, exact_match),
            &Weekday::Thursday) |
        call!(recognize_word, CompleteStr("thur"), set!(max_dist=1, exact_match),
            &Weekday::Thursday)
    )
);

named_args!(friday<'a>(exact_match: bool)<CompleteStr<'a>, MatchResult<'a>>,
    alt!(
        call!(recognize_word, CompleteStr("friday"), set!(max_dist=2, exact_match),
            &Weekday::Friday) |
        call!(recognize_word, CompleteStr("fri"), set!(max_dist=1, exact_match),
            &Weekday::Friday)
    )
);

named_args!(saturday<'a>(exact_match: bool)<CompleteStr<'a>, MatchResult<'a>>,
    alt!(
        call!(recognize_word, CompleteStr("saturday"), set!(max_dist=2, exact_match),
            &Weekday::Saturday) |
        call!(recognize_word, CompleteStr("sat"), set!(max_dist=1, exact_match),
            &Weekday::Saturday)
    )
);

named_args!(sunday<'a>(exact_match: bool)<CompleteStr<'a>, MatchResult<'a>>,
    alt!(
        call!(recognize_word, CompleteStr("sunday"), set!(max_dist=2, exact_match),
            &Weekday::Sunday) |
        call!(recognize_word, CompleteStr("sun"), set!(max_dist=1, exact_match),
            &Weekday::Sunday)
    )
);

named_args!(day_of_week<'a>(exact_match: bool)<CompleteStr<'a>, MatchResult<'a>>,
    call!(best_fit, exact_match, vec![&monday, &tuesday, &wednesday, &thursday,
                                      &friday, &saturday, &sunday])
);

named_args!(parse<'a>(exact_match: bool)<CompleteStr<'a>, (Vec<CompleteStr<'a>>, (MatchResult<'a>,
                                         MatchResult<'a>))>,
    many_till!(take_s!(1), tuple!(apply!(day_of_week, exact_match),
                                  apply!(day_of_week, exact_match)))
);

fn apply(input: &str, exact_match: bool) {
    if let Ok((tail, (ignored_vec, (token1, token2)))) = parse(CompleteStr(input), exact_match) {
        println!("parsed: {:?}, {:?}", token1, token2);
    }
}

#[test]
fn parse_monday() {
    println!("{:?}", apply(" dfdsf ths thr sunday   ", false));
}
