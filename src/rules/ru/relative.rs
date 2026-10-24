use super::lexicon::{Lexeme, UNITS, integer, integer_end, keyword, number, word};
use super::{Parsed, date_context, invalid, out_of_range, span};
use crate::rules::errors::DateTimeError;
use crate::rules::rules::Context;
use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Timelike};

// Quantities are represented in half-units, avoiding floating-point rounding.
fn part(tokens: &[Lexeme<'_>], index: usize, exact: bool) -> Option<(usize, i64, i32)> {
    let short = match word(tokens, index) {
        "полчаса" => Some(3600),
        "полминуты" => Some(60),
        "полдня" => Some(86400),
        "полгода" => Some(-12),
        _ => None,
    };
    if let Some(unit) = short {
        return Some((index + 1, 1, unit));
    }
    let (end, twice) = match word(tokens, index) {
        "полтора" | "полторы" => (index + 1, 3),
        "половина" | "половину" | "пол" => (index + 1, 1),
        "несколько" => (index + 1, 6),
        _ => {
            if let Some((end, value)) = number(tokens, index, false) {
                (end, i64::from(value) * 2)
            } else {
                (index, 2)
            }
        }
    };
    let unit = keyword(word(tokens, end), exact, UNITS)?;
    Some((end + 1, twice, unit))
}

fn convert<Tz: TimeZone>(
    now: &DateTime<Tz>,
    months: i64,
    seconds: i64,
    text: &str,
) -> Result<Context, DateTimeError> {
    let duration = Duration::try_seconds(seconds).ok_or_else(|| out_of_range(text))?;
    if months == 0 {
        return Ok(Context {
            duration,
            ..Context::default()
        });
    }
    let calendar = (i64::from(now.year()) * 12 + i64::from(now.month0()))
        .checked_add(months)
        .ok_or_else(|| out_of_range(text))?;
    let year = i32::try_from(calendar.div_euclid(12)).map_err(|_| out_of_range(text))?;
    let month = calendar.rem_euclid(12) as u32 + 1;
    if year <= 0 || NaiveDate::from_ymd_opt(year, 1, 1).is_none() {
        return Err(out_of_range(text));
    }
    let date = NaiveDate::from_ymd_opt(year, month, now.day())
        .ok_or_else(|| invalid(text, "day", now.day() as i32))?;
    if seconds == 0 {
        return Ok(date_context(date));
    }
    let shifted = now
        .timezone()
        .from_local_datetime(&date.and_time(now.time()))
        .single()
        .ok_or_else(|| DateTimeError::AmbiguousTime {
            msg: text.to_owned(),
        })?
        .checked_add_signed(duration)
        .ok_or_else(|| out_of_range(text))?;
    Ok(Context {
        duration,
        hour: Some(shifted.hour() as i32),
        minute: Some(shifted.minute() as i32),
        ..date_context(shifted.date_naive())
    })
}

pub(super) fn parse<Tz: TimeZone>(
    now: &DateTime<Tz>,
    input: &str,
    tokens: &[Lexeme<'_>],
    index: usize,
    exact: bool,
) -> Parsed {
    let future = keyword(word(tokens, index), exact, &[(1, &["через"])]) == Some(1);
    let start = index + usize::from(future);
    let first = part(tokens, start, exact);
    if first.is_none() && integer(tokens, start).is_none() {
        if let Some(end) = integer_end(tokens, start) {
            if keyword(word(tokens, end), exact, UNITS).is_some() {
                let past = keyword(word(tokens, end + 1), exact, &[(1, &["назад"])]) == Some(1);
                if future || past {
                    let end = end + 1 + usize::from(past);
                    return Some((end, Err(out_of_range(span(input, tokens, index, end)))));
                }
            }
        }
    }
    let (mut end, quantity, unit) = first?;
    let mut parts = vec![(quantity, unit)];
    loop {
        let next = end + usize::from(word(tokens, end) == "и");
        if let Some((after, quantity, unit)) = part(tokens, next, exact) {
            parts.push((quantity, unit));
            end = after;
        } else {
            break;
        }
    }
    if !future {
        if keyword(word(tokens, end), exact, &[(1, &["назад"])]) != Some(1) {
            return None;
        }
        end += 1;
    }
    let text = span(input, tokens, index, end);
    let mut seconds = 0_i64;
    let mut months = 0_i64;
    for (quantity, unit) in parts {
        if quantity < 0 {
            return Some((end, Err(invalid(text, "number", (quantity / 2) as i32))));
        }
        let value = quantity * i64::from(unit.abs());
        if value % 2 != 0 {
            return Some((
                end,
                Err(DateTimeError::AmbiguousTime {
                    msg: format!("{text} (fractional calendar months or seconds are unsupported)"),
                }),
            ));
        }
        let total = if unit < 0 { &mut months } else { &mut seconds };
        let Some(value) = total.checked_add(value / 2) else {
            return Some((end, Err(out_of_range(text))));
        };
        *total = value;
    }
    let direction = if future { 1 } else { -1 };
    Some((
        end,
        convert(now, months * direction, seconds * direction, text),
    ))
}
