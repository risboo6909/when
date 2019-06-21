use crate::rules::errors::DateTimeError;
use crate::rules::rules::{Context, MatchResult};
use chrono::offset::{TimeZone, Utc};
use chrono::{DateTime, Datelike, NaiveDateTime, Timelike};

type ParserType<'a, Tz> =
    Fn(DateTime<Tz>, &'a str, bool) -> Vec<Result<MatchResult, DateTimeError>>;

pub struct Parser<'a, Tz: TimeZone + 'a> {
    lang_parser:
        Box<Fn(DateTime<Tz>, &'a str, bool) -> Vec<Result<MatchResult, DateTimeError>> + 'a>,
    exact_match: bool,
    max_dist: usize,
    tz: Tz,
}

impl<'a, Tz: TimeZone + 'a> Parser<'a, Tz> {
    pub fn new(tz: Tz) -> Self {
        Parser {
            lang_parser: Box::new(super::en),
            exact_match: false,
            max_dist: 5,
            tz,
        }
    }

    pub fn parser(mut self, parser_func: Box<ParserType<'a, Tz>>) -> Self {
        self.lang_parser = parser_func;
        self
    }

    pub fn max_dist(mut self, max_dist: usize) -> Self {
        self.max_dist = max_dist;
        self
    }

    pub fn fuzzy_parse(mut self, fuzzy_parse: bool) -> Self {
        self.exact_match = !fuzzy_parse;
        self
    }

    pub fn get_tz(&self) -> &Tz {
        &self.tz
    }

    fn parser_helper(
        &self,
        now: NaiveDateTime,
        input: &'a str,
    ) -> (DateTime<Tz>, Vec<Result<Context, DateTimeError>>) {
        let tz_aware = self.tz.from_utc_datetime(&now);

        let res = (self.lang_parser)(tz_aware.clone(), input, self.exact_match);
        let merged = self.merge(res);

        (tz_aware, merged)
    }

    // convert date/time to chrono
    pub fn parse(&self, input: &'a str) -> Vec<Result<DateTime<Tz>, DateTimeError>> {
        let (tz_aware, merged) = self.parser_helper(Utc::now().naive_utc(), input);
        self.to_chrono(tz_aware, merged)
    }

    // convert date/time to chrono
    pub fn parse_fixed_time(
        &self,
        now: NaiveDateTime,
        input: &'a str,
    ) -> Vec<Result<DateTime<Tz>, DateTimeError>> {
        let (tz_aware, merged) = self.parser_helper(now, input);
        self.to_chrono(tz_aware, merged)
    }

    fn merge_group(&self, group: &[&MatchResult]) -> Context {
        let mut result = Context::default();
        for item in group.iter() {
            result.update(item.get_timeshift());
        }
        result
    }

    fn merge(
        &self,
        parsed: Vec<Result<MatchResult, DateTimeError>>,
    ) -> Vec<Result<Context, DateTimeError>> {
        let mut group: Vec<&MatchResult> = Vec::new();
        let mut merged = Vec::new();

        // parse results are ordered from leftmost match to the rightmost
        for item in parsed.iter() {
            match item {
                Ok(match_result) => {
                    let last = group.last();
                    if last.is_some()
                        && match_result.get_start_idx() - last.unwrap().get_end_idx()
                            > self.max_dist
                    {
                        // distance is bigger than allowed threshold, finish previous group
                        merged.push(Ok(self.merge_group(&group)));
                        group.clear();
                    }
                    // and start building a new one
                    group.push(match_result);
                }
                Err(e) => {
                    merged.push(Err(e.clone()));
                    group.clear();
                }
            }
        }

        // merge everything that left
        if !group.is_empty() {
            merged.push(Ok(self.merge_group(&group)));
        }

        merged
    }

    fn to_chrono(
        &self,
        date_time: DateTime<Tz>,
        merged: Vec<Result<Context, DateTimeError>>,
    ) -> Vec<Result<DateTime<Tz>, DateTimeError>> {
        let mut ready: Vec<Result<DateTime<Tz>, DateTimeError>> = Vec::new();

        for ctx in merged {
            if ctx.is_err() {
                ready.push(Err(ctx.unwrap_err()));
                continue;
            }

            let ctx = ctx.unwrap();

            let mut tz_aware = date_time.clone();

            tz_aware = tz_aware + ctx.duration;

            if ctx.duration.num_seconds() % 60 == 0 {
                tz_aware = tz_aware.with_second(0).unwrap();
            }

            if ctx.year.is_some() {
                tz_aware = tz_aware.with_year(ctx.year.unwrap()).unwrap();
            }
            if ctx.month.is_some() {
                tz_aware = tz_aware.with_month(ctx.month.unwrap() as u32).unwrap();
            }
            if ctx.day.is_some() {
                tz_aware = tz_aware.with_day(ctx.day.unwrap() as u32).unwrap();
            }
            if ctx.hour.is_some() {
                tz_aware = tz_aware.with_hour(ctx.hour.unwrap() as u32).unwrap();
            }
            if ctx.minute.is_some() {
                tz_aware = tz_aware.with_minute(ctx.minute.unwrap() as u32).unwrap();
            }

            tz_aware = tz_aware.with_nanosecond(0).unwrap();

            ready.push(Ok(tz_aware));
        }
        ready
    }
}
