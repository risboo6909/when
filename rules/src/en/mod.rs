mod weekdays;
mod casual_date_time;
mod hour;
mod deadline;

use crate::apply_generic;
use crate::rules::{FnRule, MatchResult};

const RULES: [FnRule; 2] = [weekdays::interpret, hour::interpret];

pub fn parse(input: &str, exact_match: bool) -> Vec<MatchResult> {
    let input_lowered = input.to_lowercase();
    apply_generic(&input_lowered, &RULES, exact_match)
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
