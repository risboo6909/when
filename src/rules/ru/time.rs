use super::lexicon::{DAYTIMES, Lexeme, UNITS, keyword, number, word};
use super::{Parsed, invalid, span};
use crate::rules::rules::Context;

pub(super) fn parse(input: &str, tokens: &[Lexeme<'_>], index: usize, exact: bool) -> Parsed {
    let prefixed = matches!(word(tokens, index), "в" | "во");
    let start = index + usize::from(prefixed);
    if let Some(hour) = keyword(word(tokens, start), exact, DAYTIMES) {
        return Some((
            start + 1,
            Ok(Context {
                hour: Some(hour),
                minute: Some(0),
                ..Context::default()
            }),
        ));
    }
    let (mut end, mut hour) = number(tokens, start, false)?;
    let raw_hour = hour;
    let mut minute = 0;
    let mut qualified = prefixed;
    if matches!(word(tokens, end), ":" | "-") {
        let (after_minutes, value) = number(tokens, end + 1, false)?;
        minute = value;
        end = after_minutes;
        qualified = true;
    }
    if keyword(word(tokens, end), exact, UNITS) == Some(3600) {
        end += 1;
        qualified = true;
        if let Some((after_minutes, value)) = number(tokens, end, false) {
            if keyword(word(tokens, after_minutes), exact, UNITS) == Some(60) {
                minute = value;
                end = after_minutes + 1;
            }
        }
    }
    if let Some(daytime) = keyword(word(tokens, end), exact, DAYTIMES) {
        hour = match daytime {
            8 if hour == 12 => 0,
            15 | 18 if hour < 12 => hour + 12,
            23 if hour == 12 => 0,
            23 if (6..12).contains(&hour) => hour + 12,
            _ => hour,
        };
        end += 1;
        qualified = true;
    }
    if !qualified && !(index == 0 && end == tokens.len()) {
        return None;
    }
    // Do not accept the numeric prefix of an identifier, decimal value or date.
    if let Some(next) = tokens.get(end) {
        if next.start == tokens[end - 1].end
            && (next.word.chars().any(char::is_alphabetic)
                || matches!(next.word.as_str(), ":" | "/" | "-" | "_"))
        {
            return None;
        }
        if next.word == "."
            && tokens
                .get(end + 1)
                .is_some_and(|token| token.text.bytes().all(|byte| byte.is_ascii_digit()))
        {
            return None;
        }
    }
    let text = span(input, tokens, index, end);
    let result = if !(0..=23).contains(&raw_hour) {
        Err(invalid(text, "hours", raw_hour))
    } else if !(0..=59).contains(&minute) {
        Err(invalid(text, "minutes", minute))
    } else {
        Ok(Context {
            hour: Some(hour),
            minute: Some(minute),
            ..Context::default()
        })
    };
    Some((end, result))
}
