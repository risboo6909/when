mod casual_date_time;
mod deadline;
mod exact_month_date;
mod past_time;
mod time;
mod weekdays;

use super::common;
use crate::apply_generic;
use crate::errors::DateTimeError;
use crate::rules::MatchResult;
use chrono::offset::TimeZone;
use chrono::DateTime;

pub fn parse<'a, Tz: TimeZone + 'a>(
    tz_aware: DateTime<Tz>,
    input: &'a str,
    exact_match: bool,
) -> Vec<Result<MatchResult, DateTimeError>> {
    let input_lowered = input.to_lowercase();
    apply_generic(
        tz_aware,
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
        exact_match,
    )
}
