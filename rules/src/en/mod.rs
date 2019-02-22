mod nouns;
mod adjectives;
mod weekdays;

use crate::apply_generic;
use crate::tokens::Tokens;

pub fn apply_all(input: &str, exact_match: bool) -> Vec<Vec<Tokens>> {
    apply_generic(input, vec![weekdays::apply], exact_match)
}

#[test]
fn test_apply_rules() {
    println!("{:?}", apply_all(" Anton you are invited to interview ths Frday or monday next week!", false));
}
