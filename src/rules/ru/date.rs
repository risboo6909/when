use super::lexicon::{
    DAYS, DIRECTIONS, Lexeme, MONTHS, WEEKDAYS, integer, integer_end, keyword, number, word,
};
use super::{Parsed, date_context, invalid, out_of_range, span};
use crate::rules::errors::DateTimeError;
use crate::rules::rules::Context;
use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone};

fn prefix(tokens: &[Lexeme<'_>], index: usize) -> usize {
    index + usize::from(matches!(word(tokens, index), "в" | "во" | "на" | "к"))
}

fn make_date(year: i32, month: i32, day: i32, text: &str) -> Result<Context, DateTimeError> {
    if year <= 0 || NaiveDate::from_ymd_opt(year, 1, 1).is_none() {
        return Err(invalid(text, "year", year));
    }
    if !(1..=12).contains(&month) {
        return Err(invalid(text, "month", month));
    }
    NaiveDate::from_ymd_opt(year, month as u32, day as u32)
        .map(date_context)
        .ok_or_else(|| invalid(text, "day", day))
}

pub(super) fn numeric<Tz: TimeZone>(
    now: &DateTime<Tz>,
    input: &str,
    tokens: &[Lexeme<'_>],
    index: usize,
) -> Parsed {
    let start = prefix(tokens, index);
    let (after_first, first) = integer(tokens, start)?;
    let delimiter = word(tokens, after_first);
    if !matches!(delimiter, "." | "/" | "-") {
        return None;
    }
    let (mut end, second) = integer(tokens, after_first + 1)?;
    let mut year = now.year();
    let mut month = second;
    let mut day = first;
    if word(tokens, end) == delimiter {
        if let Some((after_year, third)) = integer(tokens, end + 1) {
            if delimiter == "-" && tokens[start].text.len() == 4 {
                year = first;
                month = second;
                day = third;
            } else {
                year = third;
                if tokens[after_year - 1].text.len() < 4 {
                    return Some((
                        after_year,
                        Err(invalid(
                            span(input, tokens, index, after_year),
                            "year",
                            year,
                        )),
                    ));
                }
            }
            end = after_year;
        } else if let Some(end) = integer_end(tokens, end + 1) {
            return Some((end, Err(out_of_range(span(input, tokens, index, end)))));
        } else if delimiter == "-" {
            return None;
        }
    } else if delimiter == "-" {
        return None;
    }
    Some((
        end,
        make_date(year, month, day, span(input, tokens, index, end)),
    ))
}

pub(super) fn named<Tz: TimeZone>(
    now: &DateTime<Tz>,
    input: &str,
    tokens: &[Lexeme<'_>],
    index: usize,
    exact: bool,
) -> Parsed {
    let start = prefix(tokens, index);
    if let Some(end) = integer_end(tokens, start) {
        if integer(tokens, start).is_none() && keyword(word(tokens, end), exact, MONTHS).is_some() {
            return Some((
                end + 1,
                Err(out_of_range(span(input, tokens, index, end + 1))),
            ));
        }
    }
    let (month_at, day) = if let Some((end, day)) = number(tokens, start, true) {
        (end, day)
    } else {
        (start, 1)
    };
    let month = keyword(word(tokens, month_at), exact, MONTHS)?;
    let mut end = month_at + 1;
    // The period in abbreviated month names is part of the spelling.
    if tokens[month_at].word.chars().count() <= 4 && word(tokens, end) == "." {
        end += 1;
    }
    let mut year = now.year();
    if let Some((after_year, explicit_year)) = integer(tokens, end) {
        // A following qualified clock is a separate match, not a short year.
        let next = word(tokens, after_year);
        let clock = matches!(next, ":" | "-")
            || keyword(next, exact, super::lexicon::UNITS) == Some(3600)
            || keyword(next, exact, super::lexicon::DAYTIMES).is_some();
        if !clock {
            year = explicit_year;
            end = after_year;
            if tokens[end - 1].text.len() < 4 {
                return Some((
                    end,
                    Err(invalid(span(input, tokens, index, end), "year", year)),
                ));
            }
            if matches!(word(tokens, end), "г" | "год" | "года") {
                end += 1;
                if word(tokens, end) == "." {
                    end += 1;
                }
            }
        }
    }
    if integer_end(tokens, end).is_some() && integer(tokens, end).is_none() {
        let end = integer_end(tokens, end)?;
        return Some((end, Err(out_of_range(span(input, tokens, index, end)))));
    }
    Some((
        end,
        make_date(year, month, day, span(input, tokens, index, end)),
    ))
}

fn shift_days<Tz: TimeZone>(
    now: &DateTime<Tz>,
    days: i64,
    text: &str,
) -> Result<Context, DateTimeError> {
    if days == 0 {
        return Ok(Context::default());
    }
    now.date_naive()
        .checked_add_signed(Duration::days(days))
        .map(date_context)
        .ok_or_else(|| out_of_range(text))
}

pub(super) fn weekday<Tz: TimeZone>(
    now: &DateTime<Tz>,
    input: &str,
    tokens: &[Lexeme<'_>],
    index: usize,
    exact: bool,
) -> Parsed {
    let mut position = prefix(tokens, index);
    let direction = keyword(word(tokens, position), exact, DIRECTIONS);
    if direction.is_some() {
        position += 1;
    }
    let weekday = keyword(word(tokens, position), exact, WEEKDAYS)?;
    let mut end = position + 1;
    if tokens[position].word.chars().count() == 2 && word(tokens, end) == "." {
        end += 1;
    }
    let delta = weekday - now.weekday().num_days_from_monday() as i32;
    let days = match direction {
        Some(1) => {
            if delta > 0 {
                delta
            } else {
                delta + 7
            }
        }
        Some(-1) => {
            if delta < 0 {
                delta
            } else {
                delta - 7
            }
        }
        Some(0) if delta < 0 => {
            return Some((
                end,
                Err(DateTimeError::AmbiguousTime {
                    msg: span(input, tokens, index, end).to_owned(),
                }),
            ));
        }
        Some(0) => delta,
        _ => delta.rem_euclid(7),
    };
    Some((
        end,
        shift_days(now, i64::from(days), span(input, tokens, index, end)),
    ))
}

pub(super) fn casual<Tz: TimeZone>(
    now: &DateTime<Tz>,
    input: &str,
    tokens: &[Lexeme<'_>],
    index: usize,
    exact: bool,
) -> Parsed {
    let days = keyword(word(tokens, index), exact, DAYS)?;
    // A word followed immediately by digits belongs to an identifier, not a date.
    if tokens.get(index + 1).is_some_and(|next| {
        next.start == tokens[index].end && next.text.bytes().all(|byte| byte.is_ascii_digit())
    }) {
        return None;
    }
    Some((
        index + 1,
        shift_days(now, i64::from(days), span(input, tokens, index, index + 1)),
    ))
}
