#[derive(Debug, Clone)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(Debug, Clone)]
pub enum When {
    This,
    Last,
    Past,
    Next,
}

#[derive(Debug, Clone)]
pub enum Token {
    Week,
    Weekday(Weekday),
    When(When),
}

// This enum adds priority value to token, tokens with smaller priority numbers are
// being parsed first
#[derive(Debug, Clone)]
pub enum PToken {
    None,
    Stub,
    PToken(Token, isize)
}
