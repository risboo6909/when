mod adjectives;
mod nouns;
mod weekdays;

use crate::apply_generic;
use crate::tokens::Tokens;
use crate::rules::FnRule;
use crate::rules::MatchResult;

const rules: [FnRule; 1] = [weekdays::apply];

pub fn parse(input: &str, exact_match: bool) -> Vec<MatchResult> {
    apply_generic(input, &rules, exact_match)
}

#[test]
fn test_apply_rules() {
    println!(
        "{:?}",
        parse(
            " you are invited to interview ths frday or monday next week!",
            false
        )
    );
}
