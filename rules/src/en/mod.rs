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
use crate::tokens::TimeInterval::Day;
use chrono::offset::TimeZone;
use chrono::offset::Utc;
use chrono::{DateTime, NaiveDateTime};

pub fn parse<Tz: TimeZone>(
    tz: Tz,
    input: &str,
    exact_match: bool,
) -> Vec<Result<MatchResult, DateTimeError>> {
    let input_lowered = input.to_lowercase();
    let tz_aware = tz
        .from_local_datetime(&Utc::now().naive_utc())
        .single()
        .unwrap();
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
        ],
        exact_match,
    )
}

#[test]
fn test_apply_rules() {
    println!(
        "{:?}",
        parse(
            chrono_tz::Europe::Moscow,
            " you are invited to interview this saturday or friday last week!",
            false
        )
    );
}
