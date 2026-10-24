use chrono::{DateTime, NaiveDateTime, TimeZone};
use when::{DateTimeError, Parser};

fn reference() -> NaiveDateTime {
    "2018-08-03T12:34:56".parse().unwrap()
}

fn check(input: &str, expected: &str) {
    let parser = Parser::new(chrono_tz::UTC).parser(Box::new(&when::ru));
    let expected = chrono_tz::UTC.from_utc_datetime(&expected.parse().unwrap());
    assert_eq!(
        parser.parse_fixed_time(reference(), input),
        vec![Ok(expected)],
        "{input}"
    );
}

#[test]
fn russian_relative_dates_and_clock() {
    for (input, expected) in [
        ("сегодня", "2018-08-03T12:34:00"),
        ("сейчас", "2018-08-03T12:34:00"),
        ("завтра в 17", "2018-08-04T17:00:00"),
        ("ЗАВТРА В 17:30", "2018-08-04T17:30:00"),
        ("послезавтра в 7", "2018-08-05T07:00:00"),
        ("вчера в 23", "2018-08-02T23:00:00"),
        ("позавчера", "2018-08-01T12:34:00"),
        ("Напомни мне завтра в семнадцать", "2018-08-04T17:00:00"),
    ] {
        check(input, expected);
    }
}

#[test]
fn russian_times_of_day() {
    for (input, expected) in [
        ("сегодня утром", "2018-08-03T08:00:00"),
        ("завтра днём", "2018-08-04T15:00:00"),
        ("завтра днем", "2018-08-04T15:00:00"),
        ("сегодня вечером", "2018-08-03T18:00:00"),
        ("завтра ночью", "2018-08-04T23:00:00"),
        ("в полдень", "2018-08-03T12:00:00"),
        ("завтра в полночь", "2018-08-04T00:00:00"),
        ("в пять вечера", "2018-08-03T17:00:00"),
        ("в 12 дня", "2018-08-03T12:00:00"),
        ("в 12 ночи", "2018-08-03T00:00:00"),
        ("в три ночи", "2018-08-03T03:00:00"),
        ("в 11 ночи", "2018-08-03T23:00:00"),
        ("в 5:30 вечера", "2018-08-03T17:30:00"),
    ] {
        check(input, expected);
    }
}

#[test]
fn russian_clock_forms() {
    for (input, expected) in [
        ("17", "2018-08-03T17:00:00"),
        (" 0 ", "2018-08-03T00:00:00"),
        ("двадцать три", "2018-08-03T23:00:00"),
        ("в17", "2018-08-03T17:00:00"),
        ("в 17 часов 30 минут", "2018-08-03T17:30:00"),
        ("в два часа", "2018-08-03T02:00:00"),
        ("17:30", "2018-08-03T17:30:00"),
        ("17-30", "2018-08-03T17:30:00"),
    ] {
        check(input, expected);
    }
}

#[test]
fn russian_weekdays() {
    for (input, expected) in [
        ("в понедельник в 9", "2018-08-06T09:00:00"),
        ("во вторник в 17", "2018-08-07T17:00:00"),
        ("в среду", "2018-08-08T12:34:00"),
        ("в следующий четверг", "2018-08-09T12:34:00"),
        ("в пятницу", "2018-08-03T12:34:00"),
        ("в следующую пятницу в 20", "2018-08-10T20:00:00"),
        ("в эту пятницу", "2018-08-03T12:34:00"),
        ("в прошлую пятницу", "2018-07-27T12:34:00"),
        ("в прошлый понедельник", "2018-07-30T12:34:00"),
        ("в субботу утром", "2018-08-04T08:00:00"),
        ("в воскресенье вечером", "2018-08-05T18:00:00"),
        ("в пн. в 9", "2018-08-06T09:00:00"),
    ] {
        check(input, expected);
    }
}

#[test]
fn russian_calendar_dates() {
    for (input, expected) in [
        ("5 мая в 17", "2018-05-05T17:00:00"),
        ("первого января", "2018-01-01T12:34:00"),
        ("двадцать третьего февраля", "2018-02-23T12:34:00"),
        ("31 декабря 2026 года в 23:30", "2026-12-31T23:30:00"),
        ("1 янв. 2026 г.", "2026-01-01T12:34:00"),
        ("29 февраля 2024 года", "2024-02-29T12:34:00"),
        ("в мае", "2018-05-01T12:34:00"),
        ("20.12.2026 в 17:30", "2026-12-20T17:30:00"),
        ("20/12/2026 в 17", "2026-12-20T17:00:00"),
        ("20-12-2026", "2026-12-20T12:34:00"),
        ("2026-12-20 в 17", "2026-12-20T17:00:00"),
        ("20.12", "2018-12-20T12:34:00"),
    ] {
        check(input, expected);
    }
}

#[test]
fn russian_relative_intervals() {
    for (input, expected) in [
        ("через два часа", "2018-08-03T14:34:00"),
        ("через 2 часа 30 минут", "2018-08-03T15:04:00"),
        ("через один час и пятнадцать минут", "2018-08-03T13:49:00"),
        ("через одну минуту", "2018-08-03T12:35:00"),
        ("через 20 секунд", "2018-08-03T12:35:16"),
        ("10 секунд назад", "2018-08-03T12:34:46"),
        ("час назад", "2018-08-03T11:34:00"),
        ("три дня назад", "2018-07-31T12:34:00"),
        ("через две недели", "2018-08-17T12:34:00"),
        ("через месяц", "2018-09-03T12:34:00"),
        ("три месяца назад", "2018-05-03T12:34:00"),
        ("через год", "2019-08-03T12:34:00"),
        ("через полчаса", "2018-08-03T13:04:00"),
        ("через полтора часа", "2018-08-03T14:04:00"),
        ("через полторы минуты", "2018-08-03T12:36:26"),
        ("через полдня", "2018-08-04T00:34:00"),
        ("через полгода", "2019-02-03T12:34:00"),
        ("через два дня в 17", "2018-08-05T17:00:00"),
        ("через год и два месяца", "2019-10-03T12:34:00"),
    ] {
        check(input, expected);
    }
}

#[test]
fn russian_calendar_intervals_cross_year() {
    let parser = Parser::new(chrono_tz::UTC).parser(Box::new(&when::ru));
    let now = "2023-12-31T12:34:56".parse().unwrap();
    let expected = chrono_tz::UTC
        .with_ymd_and_hms(2024, 1, 31, 12, 34, 0)
        .unwrap();
    assert_eq!(
        parser.parse_fixed_time(now, "через месяц"),
        vec![Ok(expected)]
    );
    let now = "2024-02-29T12:34:56".parse().unwrap();
    assert!(matches!(
        parser.parse_fixed_time(now, "через год")[0],
        Err(DateTimeError::InvalidTime { .. })
    ));
}

#[test]
fn russian_exact_and_fuzzy_modes() {
    let fuzzy = Parser::new(chrono_tz::UTC).parser(Box::new(&when::ru));
    let exact = Parser::new(chrono_tz::UTC)
        .parser(Box::new(&when::ru))
        .fuzzy_parse(false);
    let expected = chrono_tz::UTC
        .with_ymd_and_hms(2018, 8, 4, 12, 34, 0)
        .unwrap();
    assert_eq!(
        fuzzy.parse_fixed_time(reference(), "завтро"),
        vec![Ok(expected)]
    );
    assert!(exact.parse_fixed_time(reference(), "завтро").is_empty());
    let expected = chrono_tz::UTC
        .with_ymd_and_hms(2018, 8, 3, 15, 0, 0)
        .unwrap();
    assert_eq!(
        exact.parse_fixed_time(reference(), "ДНЁМ"),
        vec![Ok(expected)]
    );
}

#[test]
fn russian_invalid_components_return_errors() {
    let parser = Parser::new(chrono_tz::UTC).parser(Box::new(&when::ru));
    for input in [
        "в 24",
        "в -1:30",
        "в 17:60",
        "31 февраля",
        "29.02.2023",
        "20.13.2026",
        "через -2 дня",
    ] {
        let results = parser.parse_fixed_time(reference(), input);
        assert_eq!(results.len(), 1, "{input}: {results:?}");
        assert!(
            matches!(results[0], Err(DateTimeError::InvalidTime { .. })),
            "{input}: {results:?}"
        );
    }
    assert!(matches!(
        parser.parse_fixed_time(reference(), "в эту среду")[0],
        Err(DateTimeError::AmbiguousTime { .. })
    ));
}

#[test]
fn russian_utf8_offsets_and_multiple_matches() {
    let text = "Ёжик: ЗАВТРА в 17";
    let raw = when::ru(chrono_tz::UTC.from_utc_datetime(&reference()), text, false);
    let bounds = raw[0].as_ref().unwrap();
    assert_eq!(
        &text[bounds.get_start_idx()..bounds.get_end_idx()],
        "ЗАВТРА"
    );
    let parser = Parser::new(chrono_tz::UTC)
        .parser(Box::new(&when::ru))
        .max_dist(3);
    let results = parser.parse_fixed_time(reference(), "завтра в 17 и послезавтра в 18");
    assert_eq!(
        results,
        vec![
            Ok(chrono_tz::UTC
                .with_ymd_and_hms(2018, 8, 4, 17, 0, 0)
                .unwrap()),
            Ok(chrono_tz::UTC
                .with_ymd_and_hms(2018, 8, 5, 18, 0, 0)
                .unwrap()),
        ]
    );
}

#[test]
fn russian_preserves_selected_timezone() {
    let parser = Parser::new(chrono_tz::Europe::Moscow).parser(Box::new(&when::ru));
    let now = "2018-08-03T23:34:56".parse().unwrap();
    let expected = chrono_tz::Europe::Moscow
        .with_ymd_and_hms(2018, 8, 5, 17, 0, 0)
        .unwrap();
    assert_eq!(
        parser.parse_fixed_time(now, "завтра в 17"),
        vec![Ok(expected)]
    );
}

#[test]
fn russian_calendar_days_and_elapsed_hours_across_dst() {
    let parser = Parser::new(chrono_tz::Europe::Amsterdam).parser(Box::new(&when::ru));
    let now = "2026-03-28T22:30:56".parse().unwrap();
    let tomorrow = chrono_tz::Europe::Amsterdam
        .with_ymd_and_hms(2026, 3, 29, 23, 30, 0)
        .unwrap();
    assert_eq!(parser.parse_fixed_time(now, "завтра"), vec![Ok(tomorrow)]);
    let elapsed: DateTime<chrono_tz::Tz> = chrono_tz::Europe::Amsterdam
        .with_ymd_and_hms(2026, 3, 30, 0, 30, 0)
        .unwrap();
    assert_eq!(
        parser.parse_fixed_time(now, "через 24 часа"),
        vec![Ok(elapsed)]
    );
    for input in ["29 марта 2026 в 2:30", "25 октября 2026 в 2:30"] {
        let results = parser.parse_fixed_time(reference(), input);
        assert_eq!(results.len(), 1, "{input}");
        assert!(results[0].is_err(), "{input}");
    }
}

#[test]
fn russian_ignores_unknown_text_and_english_is_unchanged() {
    let ru = Parser::new(chrono_tz::UTC)
        .parser(Box::new(&when::ru))
        .fuzzy_parse(false);
    for input in [
        "",
        "???",
        "обычный текст",
        "tomorrow",
        "код17",
        "в 17people",
    ] {
        assert!(
            ru.parse_fixed_time(reference(), input).is_empty(),
            "{input}"
        );
    }
    let en = Parser::new(chrono_tz::UTC).fuzzy_parse(false);
    assert!(en.parse_fixed_time(reference(), "завтра").is_empty());
    assert_eq!(
        en.parse_fixed_time(reference(), "tomorrow at 17"),
        vec![Ok(chrono_tz::UTC
            .with_ymd_and_hms(2018, 8, 4, 17, 0, 0)
            .unwrap())]
    );
}

#[test]
fn russian_invalid_raw_hours_and_years_are_not_normalized() {
    let parser = Parser::new(chrono_tz::UTC).parser(Box::new(&when::ru));
    for input in [
        "в -1 вечера",
        "в 24 ночи",
        "5 мая -2026 года",
        "5 мая 26 года",
        "20.12.26",
        "20.12.99999999999999999",
        "5 мая 99999999999999999",
        "99999999999999999 мая",
        "через 99999999999999999 дней",
    ] {
        let results = parser.parse_fixed_time(reference(), input);
        assert_eq!(results.len(), 1, "{input}: {results:?}");
        assert!(results[0].is_err(), "{input}: {results:?}");
    }
}
