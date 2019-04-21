use chrono::offset::TimeZone;
use chrono::offset::Utc;
use chrono::{DateTime, Datelike, Timelike};
use rules::rules::{Context, MatchResult};
use rules::DateTimeError;
use std::ops::Add;

type ParserType<Tz> = Fn(Tz, &str, bool) -> Vec<Result<MatchResult, DateTimeError>>;

pub struct Parser<Tz: TimeZone> {
    parser_func: Box<ParserType<Tz>>,
    exact_match: bool,
    tz: Tz,
    max_dist: usize,
}

impl<Tz: TimeZone> Parser<Tz> {
    pub fn new(
        parser_func: Box<ParserType<Tz>>,
        tz: Tz,
        max_dist: usize,
        exact_match: bool,
    ) -> Self {
        Parser {
            parser_func,
            exact_match,
            tz,
            max_dist,
        }
    }

    pub fn recognize(&self, input: &str) -> Vec<Result<DateTime<Tz>, DateTimeError>> {
        let res = (self.parser_func)(self.tz.clone(), input, self.exact_match);
        let merged = self.merge(res);

        self.to_chrono(merged)
    }

    fn merge_group(&self, group: &Vec<&MatchResult>) -> Context {
        let mut result = Context::default();
        for item in group.iter() {
            result.update(&item.time_shift);
        }
        result
    }

    fn merge(
        &self,
        parsed: Vec<Result<MatchResult, DateTimeError>>,
    ) -> Vec<Result<Context, DateTimeError>> {
        let mut groups: Vec<&MatchResult> = Vec::new();
        let mut merged: Vec<Result<Context, DateTimeError>> = Vec::new();

        // parse results are ordered from leftmost match to rightmost
        for item in parsed.iter() {
            match item {
                Ok(match_result) => {
                    let last = groups.last();
                    if last.is_some() {
                        if match_result.bounds.start_idx - last.unwrap().bounds.end_idx
                            <= self.max_dist
                        {
                            groups.push(match_result);
                        } else {
                            // distance is bigger than allowed threshold
                            merged.push(Ok(self.merge_group(&groups)));
                            groups.clear();
                        }
                    } else {
                        groups.push(match_result);
                    }
                }
                Err(e) => {
                    merged.push(Err(e.clone()));
                    groups.clear();
                }
            }
        }

        // merge everything that left
        if !groups.is_empty() {
            merged.push(Ok(self.merge_group(&groups)));
        }

        merged
    }

    fn to_chrono(
        &self,
        merged: Vec<Result<Context, DateTimeError>>,
    ) -> Vec<Result<DateTime<Tz>, DateTimeError>> {
        let mut ready: Vec<Result<DateTime<Tz>, DateTimeError>> = Vec::new();

        for ctx in merged {
            if ctx.is_err() {
                ready.push(Err(ctx.unwrap_err()));
                continue;
            }

            let mut tz_aware = self
                .tz
                .from_local_datetime(&Utc::now().naive_utc())
                .single()
                .unwrap();

            let ctx = ctx.unwrap();

            tz_aware = tz_aware.add(ctx.duration);

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
            ready.push(Ok(tz_aware));
        }
        ready
    }
}
