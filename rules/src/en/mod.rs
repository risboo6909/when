mod casual_date_time;
mod deadline;
mod exact_month_date;
mod past_time;
mod time;
mod weekdays;

use super::common;
use crate::apply_generic;
use crate::errors::DateTimeError;
use crate::rules::{FnRule, MatchResult};

const RULES: [FnRule; 2] = [weekdays::interpret, time::interpret];

pub fn parse(input: &str, exact_match: bool) -> Vec<Result<MatchResult, DateTimeError>> {
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
