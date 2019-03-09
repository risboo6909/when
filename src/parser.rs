use rules::rules::MatchResult;

type ParserType = Fn(&str, bool) -> Vec<MatchResult>;

pub struct Parser {
    parsers: Vec<Box<ParserType>>,
    exact_match: bool,
}

impl Parser {
    pub fn new(exact_match: bool) -> Self {
        Parser {
            parsers: Vec::new(),
            exact_match,
        }
    }

    pub fn add_lang(&mut self, parser_func: Box<ParserType>) {
        self.parsers.push(parser_func);
    }

    pub fn recognize(&self, input: &str) -> Vec<Vec<MatchResult>> {
        let mut results = Vec::new();

        for parser_func in &self.parsers {
            let res = parser_func(input, self.exact_match);
            results.push(res);
            // TODO: merge results
        }

        results
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn create_parser() {}
}
