use chrono::offset::TimeZone;
use rules::rules::MatchResult;
use rules::DateTimeError;

type ParserType<Tz: TimeZone> = Fn(Tz, &str, bool) -> Vec<Result<MatchResult, DateTimeError>>;

pub struct Parser<Tz: TimeZone> {
    parser_func: Box<ParserType<Tz>>,
    exact_match: bool,
    tz: Tz,
}

impl<Tz: TimeZone> Parser<Tz> {
    pub fn new(parser_func: Box<ParserType<Tz>>, tz: Tz, exact_match: bool) -> Self {
        Parser {
            parser_func,
            exact_match,
            tz,
        }
    }

    pub fn recognize(&self, input: &str) {
        let res = (self.parser_func)(self.tz.clone(), input, self.exact_match);
        self.to_chrono(res);
    }

    fn to_chrono(&self, tokenized: Vec<Result<MatchResult, DateTimeError>>) {}
}

#[cfg(test)]
mod tests {
    #[test]
    fn create_parser() {}
}
