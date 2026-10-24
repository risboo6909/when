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

#[test]
fn test_noon_and_midnight_spellings() {
    for (suffix, expected_hour) in [
        ("am", 0),
        ("a.m.", 0),
        ("a.m", 0),
        ("am.", 0),
        ("a.", 0),
        ("a", 0),
        ("pm", 12),
        ("p.m.", 12),
        ("p.m", 12),
        ("pm.", 12),
        ("p.", 12),
        ("p", 12),
        ("AM", 0),
        ("P.M.", 12),
    ] {
        for exact_match in [false, true] {
            for minute in [0, 30] {
                let input = format!("12:{minute:02} {suffix}");
                let parser = Parser::new(chrono_tz::UTC).fuzzy_parse(!exact_match);
                let expected = chrono_tz::UTC
                    .with_ymd_and_hms(2018, 8, 3, expected_hour, minute, 0)
                    .unwrap();
                assert_eq!(
                    parser.parse_fixed_time(fixed_time(), &input),
                    vec![Ok(expected)],
                    "{input}"
                );
            }
        }
    }
}

#[test]
fn test_complete_date_is_built_atomically() {
    for (now, input, expected) in [
        ("2026-01-31T12:34:56", "1 february", "2026-02-01T12:34:00"),
        ("2026-01-31T12:34:56", "1/2/2026", "2026-02-01T12:34:00"),
        ("2026-03-31T12:34:56", "30 april", "2026-04-30T12:34:00"),
        ("2024-02-29T12:34:56", "1/3/2023", "2023-03-01T12:34:00"),
        ("2023-01-31T12:34:56", "29/2/2024", "2024-02-29T12:34:00"),
    ] {
        let parser = Parser::new(chrono_tz::UTC);
        let expected =
            chrono_tz::UTC.from_utc_datetime(&NaiveDateTime::from_str(expected).unwrap());
        assert_eq!(
            parser.parse_fixed_time(NaiveDateTime::from_str(now).unwrap(), input),
            vec![Ok(expected)],
            "{now}: {input}"
        );
    }
}

#[test]
fn test_invalid_final_date_returns_error() {
    let parser = Parser::new(chrono_tz::UTC);
    for (now, input) in [
        ("2026-01-31T12:34:56", "in 1 month"),
        ("2024-02-29T12:34:56", "in 1 year"),
        ("2026-01-31T12:34:56", "31/2/2026"),
        ("2026-01-31T12:34:56", "1/2/2147483647"),
    ] {
        let results = parser.parse_fixed_time(NaiveDateTime::from_str(now).unwrap(), input);
        assert_eq!(results.len(), 1, "{input}");
        assert!(
            matches!(results[0], Err(when::DateTimeError::InvalidTime { .. })),
            "{input}: {results:?}"
        );
    }
}

#[test]
fn test_date_and_time_are_resolved_together_in_timezone() {
    let parser = Parser::new(chrono_tz::Europe::Amsterdam);
    let now = NaiveDateTime::from_str("2026-01-31T01:30:56").unwrap();
    let input = "3:30 at 29/3/2026";
    let expected = chrono_tz::Europe::Amsterdam
        .with_ymd_and_hms(2026, 3, 29, 3, 30, 0)
        .unwrap();
    assert_eq!(parser.parse_fixed_time(now, input), vec![Ok(expected)]);
}

#[test]
fn test_timezone_gap_and_fold_return_errors() {
    let parser = Parser::new(chrono_tz::Europe::Amsterdam);
    for input in ["2:30 at 29/3/2026", "2:30 at 25/10/2026"] {
        let results = parser.parse_fixed_time(fixed_time(), input);
        assert_eq!(results.len(), 1, "{input}");
        assert!(results[0].is_err(), "{input}");
    }
}

#[test]
fn test_relative_time_preserves_instant_during_clock_fold() {
    for (now, input, expected) in [
        ("2026-10-25T00:30:56", "in 1 hour", "2026-10-25T01:30:00"),
        (
            "2026-10-25T00:59:56",
            "in 20 seconds",
            "2026-10-25T01:00:16",
        ),
    ] {
        let parser = Parser::new(chrono_tz::Europe::Amsterdam);
        let expected = chrono_tz::UTC
            .from_utc_datetime(&NaiveDateTime::from_str(expected).unwrap())
            .with_timezone(&chrono_tz::Europe::Amsterdam);
        assert_eq!(
            parser.parse_fixed_time(NaiveDateTime::from_str(now).unwrap(), input),
            vec![Ok(expected)],
            "{now}: {input}"
        );
    }
}

#[test]
fn test_invalid_clock_components_return_errors() {
    let parser = Parser::new(chrono_tz::UTC);
    for input in ["24pm", "25am", "-1pm", "-1:30", "12:-1"] {
        let results = parser.parse_fixed_time(fixed_time(), input);
        assert_eq!(results.len(), 1, "{input}");
        assert!(
            matches!(results[0], Err(when::DateTimeError::InvalidTime { .. })),
            "{input}: {results:?}"
        );
    }
}

#[test]
fn test_hour_without_minutes() {
    for fuzzy in [false, true] {
        for (input, hour) in [
            ("0", 0),
            ("15", 15),
            ("20", 20),
            ("23", 23),
            ("07", 7),
            (" 20 \n", 20),
        ] {
            let parser = Parser::new(chrono_tz::UTC).fuzzy_parse(fuzzy);
            let expected = chrono_tz::UTC
                .with_ymd_and_hms(2018, 8, 3, hour, 0, 0)
                .unwrap();
            assert_eq!(
                parser.parse_fixed_time(fixed_time(), input),
                vec![Ok(expected)],
                "{input:?}"
            );
        }
    }
}

#[test]
fn test_hour_without_minutes_preserves_local_date() {
    for (timezone, now, expected) in [
        (
            chrono_tz::Europe::Moscow,
            "2018-08-03T23:34:56.123456789",
            "2018-08-04T20:00:00",
        ),
        (
            chrono_tz::America::Los_Angeles,
            "2018-08-03T01:34:56.123456789",
            "2018-08-02T20:00:00",
        ),
    ] {
        let parser = Parser::new(timezone);
        let expected = timezone
            .from_local_datetime(&NaiveDateTime::from_str(expected).unwrap())
            .unwrap();
        assert_eq!(
            parser.parse_fixed_time(NaiveDateTime::from_str(now).unwrap(), "20"),
            vec![Ok(expected)]
        );
    }
}

#[test]
fn test_hour_without_minutes_rejects_invalid_values() {
    let parser = Parser::new(chrono_tz::UTC);
    for (input, value) in [
        ("24", 24),
        ("99", 99),
        (" 24 ", 24),
        ("2147483647", i32::MAX),
    ] {
        assert_eq!(
            parser.parse_fixed_time(fixed_time(), input),
            vec![Err(when::DateTimeError::InvalidTime {
                text: input.trim().to_owned(),
                what: "hours".to_owned(),
                value,
            })],
            "{input:?}"
        );
    }
}

#[test]
fn test_hour_only_fallback_keeps_structured_rules() {
    for (input, expected) in [
        ("20:30", "2018-08-03T20:30:00"),
        ("20/12/2010", "2010-12-20T15:34:00"),
        ("20 december", "2018-12-20T15:34:00"),
        ("in 20 seconds", "2018-08-03T15:35:16"),
        ("20 seconds ago", "2018-08-03T15:34:36"),
    ] {
        assert_date_time(
            Parser::new(chrono_tz::Europe::Moscow),
            input,
            &[expected],
            1,
        );
    }
    let parser = Parser::new(chrono_tz::UTC).fuzzy_parse(false);
    for input in [
        "",
        " ",
        "-1",
        "+15",
        "15.0",
        "15x",
        "20 people",
        "２０",
        "2147483648",
    ] {
        assert!(
            parser.parse_fixed_time(fixed_time(), input).is_empty(),
            "{input:?}"
        );
    }
}

#[test]
fn test_hour_without_minutes_after_at() {
    for fuzzy in [false, true] {
        for (input, expected) in [
            ("tomorrow at 17", "2018-08-04T17:00:00"),
            ("tomorrow at 17:00", "2018-08-04T17:00:00"),
            ("tomorrow at 0", "2018-08-04T00:00:00"),
            ("today at 23", "2018-08-03T23:00:00"),
            ("next monday at 7", "2018-08-06T07:00:00"),
            ("call me at 20", "2018-08-03T20:00:00"),
            ("at 15", "2018-08-03T15:00:00"),
            ("Привет, tomorrow at 17", "2018-08-04T17:00:00"),
        ] {
            assert_date_time(
                Parser::new(chrono_tz::UTC).fuzzy_parse(fuzzy),
                input,
                &[expected],
                1,
            );
        }
    }
}

#[test]
fn test_at_hour_keeps_structured_rules_and_errors() {
    for (input, expected) in [
        ("tomorrow at 5pm", "2018-08-04T17:00:00"),
        ("tomorrow at 12 am", "2018-08-04T00:00:00"),
        ("at 20:30", "2018-08-03T20:30:00"),
        ("at 17-30", "2018-08-03T17:30:00"),
        ("at 20 december", "2018-12-20T12:34:00"),
        ("at 20/12/2010", "2010-12-20T12:34:00"),
        ("at 20 seconds ago", "2018-08-03T12:34:36"),
        ("at in 20 seconds", "2018-08-03T12:35:16"),
    ] {
        assert_date_time(Parser::new(chrono_tz::UTC), input, &[expected], 1);
    }
    let parser = Parser::new(chrono_tz::UTC);
    for (input, what, value) in [
        ("tomorrow at 24", "hours", 24),
        ("at -1", "hours", -1),
        ("tomorrow at 17:60", "minutes", 60),
        ("at 31 february", "day", 31),
    ] {
        let results = parser.parse_fixed_time(fixed_time(), input);
        assert_eq!(results.len(), 1, "{input}: {results:?}");
        assert!(
            matches!(&results[0], Err(when::DateTimeError::InvalidTime { what: actual_what, value: actual_value, .. }) if actual_what == what && *actual_value == value),
            "{input}: {results:?}"
        );
    }
}

#[test]
fn test_at_hour_requires_whole_number() {
    let parser = Parser::new(chrono_tz::UTC).fuzzy_parse(false);
    for input in [
        "at 17people",
        "at 15.5",
        "at 15/",
        "at 15:",
        "at 15-",
        "at １５",
        "format 17",
        "at17",
    ] {
        assert!(
            parser.parse_fixed_time(fixed_time(), input).is_empty(),
            "{input:?}"
        );
    }
}

#[test]
fn test_multiple_at_hours_keep_offsets_after_error() {
    let parser = Parser::new(chrono_tz::UTC).max_dist(3);
    let results = parser.parse_fixed_time(fixed_time(), "at 24; and tomorrow at 20");
    assert_eq!(results.len(), 2, "{results:?}");
    assert!(matches!(
        results[0],
        Err(when::DateTimeError::InvalidTime { value: 24, .. })
    ));
    assert_eq!(
        results[1],
        Ok(chrono_tz::UTC
            .with_ymd_and_hms(2018, 8, 4, 20, 0, 0)
            .unwrap())
    );
}
