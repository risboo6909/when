mod weekdays;
mod casual_date;
mod hour;

use crate::apply_generic;
use crate::tokens::Token;
use crate::rules::{FnRule, MatchResult};

const rules: [FnRule; 1] = [weekdays::interpret];

pub fn parse(input: &str, exact_match: bool) -> Vec<MatchResult> {
    apply_generic(input, &rules, exact_match)
}

#[test]
fn test_apply_rules() {
    println!(
        "{:?}",
        parse(
            " you are invited to interview this saturday or friday last week!",
            false
        )
    );
}
