use chrono::{offset::TimeZone, NaiveDate, NaiveDateTime, NaiveTime};
use std::str::FromStr;
use when;
use when::Parser;

fn fixed_time() -> NaiveDateTime {
    let d = NaiveDate::from_ymd(2018, 8, 3);
    let t = NaiveTime::from_hms(12, 34, 56);

    NaiveDateTime::new(d, t)
}

fn assert_date_time<'a, Tz: TimeZone>(
    parser: Parser<'a, Tz>,
    input: &'a str,
    expected_date_time_strs: &[&str],
    expected_length: usize,
) {
    let res = parser.recognize_fixed_time(fixed_time(), input);

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
    let parser = when::parser::Parser::new(Box::new(when::en), chrono_tz::Europe::Moscow, 8, false);
    assert_date_time(
        parser,
        "if I will finish this project in the hlf of yar",
        &["2019-02-03T12:34:00"],
        1,
    );
}

#[test]
fn test_merge_parse_results() {
    let parser = when::parser::Parser::new(Box::new(when::en), chrono_tz::Europe::Moscow, 5, false);
    assert_date_time(
        parser,
        "Call me next mnday at 6P.m.",
        &["2018-08-06T18:00:00"],
        1,
    );
}

#[test]
fn test_multiple_results() {
    let parser = when::parser::Parser::new(Box::new(when::en), chrono_tz::Europe::Moscow, 3, false);
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
    let parser = when::parser::Parser::new(Box::new(when::en), chrono_tz::Europe::Moscow, 5, false);

    let res = parser.recognize_fixed_time(
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

    let naive = chrono::NaiveDateTime::from_str("2018-08-06T12:34:00").unwrap();
    assert_eq!(
        res[1],
        Ok(chrono_tz::Europe::Moscow
            .from_local_datetime(&naive)
            .unwrap())
    );
}

#[test]
fn test_various_requests() {
    let parser = when::parser::Parser::new(Box::new(when::en), chrono_tz::Europe::Moscow, 5, false);
    assert_date_time(parser, "at Friday afternoon", &["2018-08-03T15:00:00"], 1);

    let parser = when::parser::Parser::new(Box::new(when::en), chrono_tz::Europe::Moscow, 5, false);
    assert_date_time(
        parser,
        "drop me a line next wednesday at 2:25 p.m",
        &["2018-08-08T14:25:00"],
        1,
    );
}
