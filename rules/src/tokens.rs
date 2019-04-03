use chrono::format::StrftimeItems;

#[derive(Debug, Clone, PartialEq)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pronouns {
    This,
}

#[derive(Debug, Clone, PartialEq)]
pub enum When {
    Within,
    In,
    This,
    Last,
    Past,
    Next,
    Now,
    Today,
    Tonight,
    Tomorrow,
    Yesterday,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AmPm {
    Am,
    Pm,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TimeOfDay {
    Night,
    Morning,
    Evening,
    Noon,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IntWord {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Eleven,
    Twelve,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ordinals {
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Sixth,
    Seventh,
    Eighth,
    Ninth,
    Tenth,
    Eleventh,
    Twelfth,
    Thirteenth,
    Fourteenth,
    Fifteenth,
    Sixteenth,
    Seventeenth,
    Eighteenth,
    Nineteenth,
    Twentieth,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TimeInterval {
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Year,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Adverbs {
    Half,
    Few,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Articles {
    A,
    An,
    The,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Char, // stands for any character
    Week,
    Articles(Articles),
    Weekday(Weekday),
    When(When),
    AmPm(AmPm),
    Number(u32),
    IntWord(IntWord),
    TimeInterval(TimeInterval),
    TimeOfDay(TimeOfDay),
    Adverbs(Adverbs),
    Pronouns(Pronouns),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Priority(pub isize);

// This enum adds priority value to token, tokens with smaller priority numbers are
// being parsed first
#[derive(Debug, Clone, PartialEq)]
pub enum PToken {
    Stub,
    PToken(Token, Priority),
}
