use crate::rules::errors::DateTimeError;
use crate::rules::rules::{Context, MatchResult};
use chrono::{DateTime, Datelike, NaiveDate, TimeZone};

mod date;
mod lexicon;
mod relative;
mod time;

use lexicon::Lexeme;

type Parsed = Option<(usize, Result<Context, DateTimeError>)>;

fn span<'a>(input: &'a str, tokens: &[Lexeme<'_>], start: usize, end: usize) -> &'a str {
    &input[tokens[start].start..tokens[end - 1].end]
}

fn invalid(text: &str, what: &str, value: i32) -> DateTimeError {
    DateTimeError::InvalidTime {
        text: text.to_owned(),
        what: what.to_owned(),
        value,
    }
}

fn out_of_range(text: &str) -> DateTimeError {
    DateTimeError::AmbiguousTime {
        msg: format!("{text} (outside the supported date/time range)"),
    }
}

fn date_context(date: NaiveDate) -> Context {
    Context {
        year: Some(date.year()),
        month: Some(date.month() as i32),
        day: Some(date.day() as i32),
        ..Context::default()
    }
}

pub fn parse<Tz: TimeZone>(
    now: DateTime<Tz>,
    input: &str,
    exact_match: bool,
) -> Vec<Result<MatchResult, DateTimeError>> {
    let tokens = lexicon::tokenize(input);
    let mut result = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        // Consume structured expressions before considering their individual words
        // and numbers. This avoids intersections between dates, clocks and durations.
        let parsed = relative::parse(&now, input, &tokens, index, exact_match)
            .or_else(|| date::numeric(&now, input, &tokens, index))
            .or_else(|| date::named(&now, input, &tokens, index, exact_match))
            .or_else(|| date::weekday(&now, input, &tokens, index, exact_match))
            .or_else(|| date::casual(&now, input, &tokens, index, exact_match))
            .or_else(|| time::parse(input, &tokens, index, exact_match));
        if let Some((end, parsed)) = parsed {
            result.push(
                parsed.map(|ctx| MatchResult::new(ctx, tokens[index].start, tokens[end - 1].end)),
            );
            index = end;
        } else {
            index += 1;
        }
    }
    result
}
