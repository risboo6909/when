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
    AM,
    PM,
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
    None,
    Week,
    Articles(Articles),
    Weekday(Weekday),
    When(When),
    Number(usize),
    IntWord(IntWord),
    TimeInterval(TimeInterval),
    Adverbs(Adverbs),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Priority(pub isize);

// This enum adds priority value to token, tokens with smaller priority numbers are
// being parsed first
#[derive(Debug, Clone, PartialEq)]
pub enum PToken {
    None,
    Stub,
    // PToken consists of Token and priority of type isize
    PToken(Token, Priority),
}
