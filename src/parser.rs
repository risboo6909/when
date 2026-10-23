use crate::rules::errors::DateTimeError;
use crate::rules::rules::{Context, MatchResult};
use chrono::offset::{TimeZone, Utc};
use chrono::{DateTime, Datelike, LocalResult, NaiveDate, NaiveDateTime, Timelike};

type ParserType<'a, Tz> =
    &'a dyn Fn(DateTime<Tz>, &'a str, bool) -> Vec<Result<MatchResult, DateTimeError>>;

pub struct Parser<'a, Tz: TimeZone + 'a> {
    lang_parser:
        Box<&'a dyn Fn(DateTime<Tz>, &'a str, bool) -> Vec<Result<MatchResult, DateTimeError>>>,
    exact_match: bool,
    max_dist: usize,
    tz: Tz,
}

impl<'a, Tz: TimeZone + 'a> Parser<'a, Tz> {
    pub fn new(tz: Tz) -> Self {
        Parser {
            lang_parser: Box::new(&super::en),
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
        self.to_chrono(tz_aware, merged, input)
    }

    // convert date/time to chrono
    pub fn parse_fixed_time(
        &self,
        now: NaiveDateTime,
        input: &'a str,
    ) -> Vec<Result<DateTime<Tz>, DateTimeError>> {
        let (tz_aware, merged) = self.parser_helper(now, input);
        self.to_chrono(tz_aware, merged, input)
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
        input: &str,
    ) -> Vec<Result<DateTime<Tz>, DateTimeError>> {
        merged
            .into_iter()
            .map(|ctx| ctx.and_then(|ctx| self.apply_context(&date_time, ctx, input)))
            .collect()
    }

    fn apply_context(
        &self,
        date_time: &DateTime<Tz>,
        ctx: Context,
        input: &str,
    ) -> Result<DateTime<Tz>, DateTimeError> {
        let invalid = |what: &str, value| DateTimeError::InvalidTime {
            text: input.to_owned(),
            what: what.to_owned(),
            value,
        };
        let out_of_range = || DateTimeError::AmbiguousTime {
            msg: format!("{input} (outside the supported date/time range)"),
        };
        let shifted = date_time
            .clone()
            .checked_add_signed(ctx.duration)
            .ok_or_else(out_of_range)?;
        let year = ctx.year.unwrap_or(shifted.year());
        let month = ctx.month.unwrap_or(shifted.month() as i32);
        let day = ctx.day.unwrap_or(shifted.day() as i32);
        let hour = ctx.hour.unwrap_or(shifted.hour() as i32);
        let minute = ctx.minute.unwrap_or(shifted.minute() as i32);
        let second = if ctx.duration.num_seconds() % 60 == 0 {
            0
        } else {
            shifted.second()
        };

        if NaiveDate::from_ymd_opt(year, 1, 1).is_none() {
            return Err(invalid("year", year));
        }
        if !(1..=12).contains(&month) {
            return Err(invalid("month", month));
        }
        // Construct the final calendar date, avoiding invalid intermediate dates
        // such as January 31 -> February 31 -> February 1.
        let date = NaiveDate::from_ymd_opt(year, month as u32, day as u32)
            .ok_or_else(|| invalid("day", day))?;
        if !(0..=23).contains(&hour) {
            return Err(invalid("hours", hour));
        }
        let local = date
            .and_hms_opt(hour as u32, minute as u32, second)
            .ok_or_else(|| invalid("minutes", minute))?;

        if ctx.year.is_none()
            && ctx.month.is_none()
            && ctx.day.is_none()
            && ctx.hour.is_none()
            && ctx.minute.is_none()
        {
            // A duration already identifies an instant, even during a clock fold.
            // Only remove sub-minute precision here; do not reinterpret the clock.
            let rounding = local - shifted.naive_local();
            return shifted
                .checked_add_signed(rounding)
                .ok_or_else(out_of_range);
        }

        // Resolve the timezone after applying both the date and the clock time.
        match self.tz.from_local_datetime(&local) {
            LocalResult::Single(value) => Ok(value),
            LocalResult::Ambiguous(_, _) => Err(DateTimeError::AmbiguousTime {
                msg: input.to_owned(),
            }),
            LocalResult::None => Err(invalid("local hours", hour)),
        }
    }
}
