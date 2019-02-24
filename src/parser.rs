pub struct Parser {}

impl Parser {
    pub fn new() -> Self {
        Parser {}
    }

    pub fn set_lang(&mut self) -> &mut Self {
        self
    }

    pub fn tokenize(&mut self) -> &mut Self {
        self
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn create_parser() {}
}
