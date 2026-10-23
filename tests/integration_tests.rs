use chrono::{NaiveDate, NaiveDateTime, NaiveTime, offset::TimeZone};
use std::str::FromStr;
use when;
use when::Parser;

fn fixed_time() -> NaiveDateTime {
    let d = NaiveDate::from_ymd_opt(2018, 8, 3).unwrap();
    let t = NaiveTime::from_hms_opt(12, 34, 56).unwrap();

    NaiveDateTime::new(d, t)
}

fn assert_date_time<'a, Tz: TimeZone>(
    parser: Parser<'a, Tz>,
    input: &'a str,
    expected_date_time_strs: &[&str],
    expected_length: usize,
) {
    let res = parser.parse_fixed_time(fixed_time(), input);

    assert_eq!(res.len(), expected_length);

    for (i, date_time_str) in expected_date_time_strs.iter().enumerate() {
        let naive = chrono::NaiveDateTime::from_str(date_time_str).unwrap();
        assert_eq!(
            res[i],
            Ok(parser.get_tz().from_local_datetime(&naive).unwrap())
        );
    }
}

#[test]
fn test_basic() {
    let parser = when::parser::Parser::new(chrono_tz::Europe::Moscow).max_dist(8);
    assert_date_time(
        parser,
        "if I will finish this project in the hlf of yar",
        &["2019-02-03T15:34:00"],
        1,
    );
}

#[test]
fn test_merge_parse_results() {
    let parser = when::parser::Parser::new(chrono_tz::Europe::Moscow);
    assert_date_time(
        parser,
        "Call me next mnday at 6P.m.",
        &["2018-08-06T18:00:00"],
        1,
    );
}

#[test]
fn test_multiple_results() {
    let parser = when::parser::Parser::new(chrono_tz::Europe::Moscow).max_dist(3);
    assert_date_time(
        parser,
        "Today 21:50 and tomorrow 22:00 also yesterday   5a.m.",
        &[
            "2018-08-03T21:50:00",
            "2018-08-04T22:00:00",
            "2018-08-02T05:00:00",
        ],
        3,
    );
}

#[test]
fn test_overlap_error() {
    let parser = when::parser::Parser::new(chrono_tz::Europe::Moscow);

    let res = parser.parse_fixed_time(
        fixed_time(),
        "Call me next march 13 seconds ago, something else, next monday",
    );

    assert_eq!(res.len(), 2);

    assert_eq!(
        res[0],
        Err(when::DateTimeError::IntersectionError {
            text: "march 13 seconds ago".to_owned()
        })
    );

    let naive = chrono::NaiveDateTime::from_str("2018-08-06T15:34:00").unwrap();
    assert_eq!(
        res[1],
        Ok(chrono_tz::Europe::Moscow
            .from_local_datetime(&naive)
            .unwrap())
    );
}

#[test]
fn test_seconds_delta() {
    let parser = when::parser::Parser::new(chrono_tz::Europe::Moscow);
    assert_date_time(parser, "in 20 seconds", &["2018-08-03T15:35:16"], 1);
}

#[test]
fn test_various_requests() {
    let parser = when::parser::Parser::new(chrono_tz::Europe::Moscow);
    assert_date_time(parser, "at Saturday afternoon", &["2018-08-04T15:00:00"], 1);

    let parser = when::parser::Parser::new(chrono_tz::Europe::Moscow);
    assert_date_time(
        parser,
        "drop me a line next wednesday at 2:25 p.m",
        &["2018-08-08T14:25:00"],
        1,
    );

    let parser = when::parser::Parser::new(chrono_tz::Europe::Moscow);
    assert_date_time(parser, "in 1 hour", &["2018-08-03T16:34:00"], 1);
}

#[test]
fn test_exact_and_fuzzy_modes() {
    let fuzzy = Parser::new(chrono_tz::Europe::Moscow);
    assert_date_time(fuzzy, "tomorow", &["2018-08-04T15:34:00"], 1);

    let exact = Parser::new(chrono_tz::Europe::Moscow).fuzzy_parse(false);
    assert!(exact.parse_fixed_time(fixed_time(), "tomorow").is_empty());
    assert_date_time(exact, "tomorrow", &["2018-08-04T15:34:00"], 1);
}

#[test]
fn test_dates_and_unicode_prefix() {
    for (input, expected) in [
        ("20/12/2010", "2010-12-20T15:34:00"),
        ("twenty seventh of april", "2018-04-27T15:34:00"),
        ("five minutes ago", "2018-08-03T15:29:00"),
        ("Привет, tomorrow at 14:30", "2018-08-04T14:30:00"),
    ] {
        assert_date_time(
            Parser::new(chrono_tz::Europe::Moscow),
            input,
            &[expected],
            1,
        );
    }
}

#[test]
fn test_unrecognized_input() {
    let parser = Parser::new(chrono_tz::UTC).fuzzy_parse(false);
    for input in ["", "   ", "???", "/", "-", "just some text", "こんにちは"] {
        assert!(
            parser.parse_fixed_time(fixed_time(), input).is_empty(),
            "{input:?}"
        );
    }
}

#[test]
fn test_invalid_time_is_standard_error() {
    let parser = Parser::new(chrono_tz::UTC).fuzzy_parse(false);
    let result = parser.parse_fixed_time(fixed_time(), "24:10");
    assert_eq!(
        result,
        vec![Err(when::DateTimeError::InvalidTime {
            text: "24:10".to_owned(),
            what: "hours".to_owned(),
            value: 24,
        })]
    );
    let error: &dyn std::error::Error = result[0].as_ref().unwrap_err();
    assert_eq!(
        error.to_string(),
        "invalid time in: 24:10, hours can't be 24"
    );
}

#[test]
fn test_explicit_language_parser() {
    let parser = Parser::new(chrono_tz::UTC).parser(Box::new(&when::en));
    assert_date_time(parser, "in 20 seconds", &["2018-08-03T12:35:16"], 1);
}
