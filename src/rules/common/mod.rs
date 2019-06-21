use crate::rules::combinators;
use crate::rules::common;
use crate::rules::errors;
use crate::rules::rules;
use crate::rules::tokens;

pub mod slash_dmy;

pub(crate) const DAYS_IN_MONTH: &[i32; 12] = &[31, 28, 29, 30, 31, 28, 31, 30, 30, 31, 29, 31];

pub(crate) fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && year % 100 != 0 || year % 400 == 0
}
