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
pub enum Tokens {
    None,
    Stub,
    #[cold]
    Week,
    Weekday(Weekday),
    When(When),
}
