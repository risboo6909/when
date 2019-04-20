use rules::rules::MatchResult;
use rules::DateTimeError;

type ParserType = Fn(&str, bool) -> Vec<Result<MatchResult, DateTimeError>>;

pub struct Parser {
    parser_func: Box<ParserType>,
    exact_match: bool,
}

impl Parser {
    pub fn new(parser_func: Box<ParserType>, exact_match: bool) -> Self {
        Parser {
            parser_func,
            exact_match,
        }
    }

    pub fn recognize(&self, input: &str) -> Vec<Result<MatchResult, DateTimeError>> {
        let res = (self.parser_func)(input, self.exact_match);
        res
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn create_parser() {}
}
