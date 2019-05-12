use chrono::offset::TimeZone;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime};
use std::str::FromStr;
use when;

fn fixed_time() -> NaiveDateTime {
    let d = NaiveDate::from_ymd(2018, 8, 3);
    let t = NaiveTime::from_hms(12, 34, 56);

    NaiveDateTime::new(d, t)
}

#[test]
fn test_basic() {
    let mut parser =
        when::parser::Parser::new(Box::new(when::en), chrono_tz::Europe::Moscow, 8, false);

    let res = parser.recognize_fixed_time(
        fixed_time(),
        "if I will finish this project in the hlf of yar",
    );

    assert_eq!(res.len(), 1);

    let naive = chrono::NaiveDateTime::from_str("2019-02-03T12:34:56").unwrap();
    assert_eq!(
        res[0],
        Ok(chrono_tz::Europe::Moscow
            .from_local_datetime(&naive)
            .unwrap())
    );
}

#[test]
fn test_merge_parse_results() {
    let mut parser =
        when::parser::Parser::new(Box::new(when::en), chrono_tz::Europe::Moscow, 5, false);

    let res = parser.recognize_fixed_time(fixed_time(), "Call me next mnday at 6P.m.");

    assert_eq!(res.len(), 1);

    let naive = chrono::NaiveDateTime::from_str("2018-08-06T18:00:56").unwrap();

    assert_eq!(
        res[0],
        Ok(chrono_tz::Europe::Moscow
            .from_local_datetime(&naive)
            .unwrap())
    );
}

#[test]
fn test_multiple_results() {
    let mut parser =
        when::parser::Parser::new(Box::new(when::en), chrono_tz::Europe::Moscow, 2, false);

    let res = parser.recognize_fixed_time(
        fixed_time(),
        "Today 21:50 and tomorrow 22:00 also yesterday 5a.m.",
    );

    println!("{:?}", res);

    //    assert_eq!(res.len(), 1);
    //
    //    let naive = chrono::NaiveDateTime::from_str("2018-08-06T18:00:56").unwrap();
    //
    //    assert_eq!(
    //        res[0],
    //        Ok(chrono_tz::Europe::Moscow
    //            .from_local_datetime(&naive)
    //            .unwrap())
    //    );
}

#[test]
fn test_overlap_error() {
    let mut parser =
        when::parser::Parser::new(Box::new(when::en), chrono_tz::Europe::Moscow, 5, false);

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

    let naive = chrono::NaiveDateTime::from_str("2018-08-06T12:34:56").unwrap();
    assert_eq!(
        res[1],
        Ok(chrono_tz::Europe::Moscow
            .from_local_datetime(&naive)
            .unwrap())
    );
}
