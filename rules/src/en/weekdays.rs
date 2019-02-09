use crate::{recognize_word, Token};

use nom::{alt, call, named, named_args, apply, tuple, IResult, types::CompleteStr};
use std::fmt::Debug;

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

named_args!(monday<'a>(fuzzyfy: bool)<CompleteStr<'a>, &dyn Token>,
    alt!(
            call!(recognize_word, CompleteStr("monday"),
                set!(max_dist=2, fuzzyfy), &Weekday::Monday) |
            call!(recognize_word, CompleteStr("mon"),
                set!(max_dist=1, fuzzyfy), &Weekday::Monday)
        )
);

named_args!(tuesday<'a>(fuzzyfy: bool)<CompleteStr<'a>, &dyn Token>,
    alt!(
            call!(recognize_word, CompleteStr("tuesday"),
                set!(max_dist=2, fuzzyfy), &Weekday::Tuesday) |
            call!(recognize_word, CompleteStr("tue"),
                set!(max_dist=1, fuzzyfy), &Weekday::Tuesday)
    )
);

named_args!(wednesday<'a>(fuzzyfy: bool)<CompleteStr<'a>, &dyn Token>,
    call!(recognize_word, CompleteStr("wednesday"), set!(max_dist=2, fuzzyfy), &Weekday::Wednesday)
);

named_args!(thursday<'a>(fuzzyfy: bool)<CompleteStr<'a>, &dyn Token>,
    call!(recognize_word, CompleteStr("thursday"), set!(max_dist=2, fuzzyfy), &Weekday::Thursday)
);

named_args!(friday<'a>(fuzzyfy: bool)<CompleteStr<'a>, &dyn Token>,
    call!(recognize_word, CompleteStr("friday"), set!(max_dist=2, fuzzyfy), &Weekday::Friday)
);

named_args!(saturday<'a>(fuzzyfy: bool)<CompleteStr<'a>, &dyn Token>,
    call!(recognize_word, CompleteStr("saturday"), set!(max_dist=2, fuzzyfy), &Weekday::Saturday)
);

named_args!(sunday<'a>(fuzzyfy: bool)<CompleteStr<'a>, &dyn Token>,
    call!(recognize_word, CompleteStr("sunday"), set!(max_dist=2, fuzzyfy), &Weekday::Sunday)
);

named_args!(day_of_week<'a>(fuzzy_scan: bool)<CompleteStr<'a>, &dyn Token>,
    alt!(call!(monday, fuzzy_scan) |
         call!(tuesday, fuzzy_scan) |
         call!(wednesday, fuzzy_scan) |
         call!(thursday, fuzzy_scan) |
         call!(friday, fuzzy_scan) |
         call!(saturday, fuzzy_scan) |
         call!(sunday, fuzzy_scan))
);

fn apply(input: &str) {

}

#[test]
fn parse_monday() {

    named!(z(CompleteStr) -> (&dyn Token, &dyn Token),
                                 tuple!(apply!(day_of_week, true), apply!(day_of_week, true)));

    println!("{:?}", z(CompleteStr(" moday tue   ")));
}
