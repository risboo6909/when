mod weekdays;
mod adjectives;
mod nouns;

struct En {
    max_error: usize,
}

impl En {
    fn set_max_error(&mut self, max_error: usize) {
        self.max_error = max_error
    }
}
