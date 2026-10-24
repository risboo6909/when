mod casual_date_time;
mod deadline;
mod exact_month_date;
mod past_time;
mod time;
mod weekdays;

use crate::rules::combinators;
use crate::rules::common;
use crate::rules::common_matchers;
use crate::rules::consts;
use crate::rules::errors;
use crate::rules::rules;
use crate::rules::tokens;
use chrono::{DateTime, offset::TimeZone};

pub fn parse<'a, Tz: TimeZone + 'a>(
    tz_aware: DateTime<Tz>,
    input: &'a str,
    exact_match: bool,
) -> Vec<Result<rules::MatchResult, errors::DateTimeError>> {
    let input_lowered = input.to_lowercase();
    let mut matches = combinators::apply_generic(
        tz_aware.clone(),
        &input_lowered,
        &[
            weekdays::interpret::<Tz>,
            time::interpret::<Tz>,
            past_time::interpret::<Tz>,
            exact_month_date::interpret::<Tz>,
            deadline::interpret::<Tz>,
            casual_date_time::interpret::<Tz>,
            common::slash_dmy::interpret::<Tz>,
        ],
        &[time::interpret_at_hour::<Tz>],
        exact_match,
    );
    if matches.is_empty() {
        if let Some(hour) = time::interpret_hour_only(&input_lowered, tz_aware) {
            matches.push(hour);
        }
    }
    matches
}
