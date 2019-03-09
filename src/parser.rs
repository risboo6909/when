use rules::rules::MatchResult;

type ParserType = Fn(&str, bool) -> Vec<MatchResult>;

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

    pub fn recognize(&self, input: &str) -> Vec<MatchResult> {
        let res = (self.parser_func)(input, self.exact_match);
        res
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn create_parser() {

    }
}
