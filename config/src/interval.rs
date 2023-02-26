//! Use the [Interval] struct to specify the fequency of certain events (similar to cronjobs on Linux).

use std::ops::RangeInclusive;

mod months;
mod specifier;
mod weekdays;
mod date_time_match;

use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use derive_builder::Builder;

pub use self::{
    date_time_match::DateTimeMatch,
    months::Month,
    specifier::{Specifier, SpecifierKind},
    weekdays::Weekday,
};

/// Allows to specify specific times in the range of every minute up to only once a year.
/// Has similar capabilities as the time specifier in crontabs on Linux.
///
/// If weekdays and monthdays are both not [SpecifierKind::All], then only one of them has to match.
///
/// Weeknumbers are handles as [ISO-Weeks](https://en.wikipedia.org/wiki/ISO_week_date). I.e., the first week with 4 days or mor in a year is week 0.
#[derive(Builder)]
#[builder(build_fn(validate = "Self::validate"))]
pub struct Interval {
    /// Range 0-59
    #[builder(
        default = "Specifier::new(*MINUTES_RANGE.start(), *MINUTES_RANGE.end(), SpecifierKind::All)"
    )]
    minutes: Specifier<u32>,
    /// Range 0-23
    #[builder(
        default = "Specifier::new(*HOURS_RANGE.start(), *HOURS_RANGE.end(), SpecifierKind::All)"
    )]
    hours: Specifier<u32>,
    /// Range Monday-Sunday
    #[builder(
        default = "Specifier::new(*WEEKDAYS_RANGE.start(), *WEEKDAYS_RANGE.end(), SpecifierKind::All)"
    )]
    weekdays: Specifier<weekdays::Weekday>,
    /// Range 0-32
    #[builder(
        default = "Specifier::new(*MONTHDAYS_RANGE.start(), *MONTHDAYS_RANGE.end(), SpecifierKind::All)"
    )]
    monthdays: Specifier<u32>,
    /// Range 0-52
    #[builder(
        default = "Specifier::new(*WEEKS_RANGE.start(), *WEEKS_RANGE.end(), SpecifierKind::All)"
    )]
    weeks: Specifier<u32>,
    /// Range January-December
    #[builder(
        default = "Specifier::new(*MONTHS_RANGE.start(), *MONTHS_RANGE.end(), SpecifierKind::All)"
    )]
    months: Specifier<months::Month>,
}

const MINUTES_RANGE: RangeInclusive<u32> = 0..=59;
const HOURS_RANGE: RangeInclusive<u32> = 0..=23;
const WEEKDAYS_RANGE: RangeInclusive<weekdays::Weekday> =
    weekdays::Weekday::Monday()..=weekdays::Weekday::Sunday();
const MONTHDAYS_RANGE: RangeInclusive<u32> = 0..=31;
const WEEKS_RANGE: RangeInclusive<u32> = 0..=52;
const MONTHS_RANGE: RangeInclusive<months::Month> =
    months::Month::January()..=months::Month::December();

impl Interval {
    /// Creates an [Interval] that specifies the given time on every day.
    ///
    /// # Returns
    /// [Ok] containig the corresponding [Interval] or [Err] descibing the issue, when `minute` or `hour` don't have meaningful values (e.g., `minute == 100`).
    pub fn daily(minute: u32, hour: u32) -> Result<Interval, String> {
        if !MINUTES_RANGE.contains(&minute) {
            return Err(format!(
                "Expect 'minute' to be in range {:?}. Got {}",
                MINUTES_RANGE, minute
            ));
        }
        if !HOURS_RANGE.contains(&hour) {
            return Err(format!(
                "Expect 'hour' to be in range {:?}. Got {}",
                HOURS_RANGE, hour
            ));
        }

        let minutes_spec = Specifier::new(
            *MINUTES_RANGE.start(),
            *MINUTES_RANGE.end(),
            SpecifierKind::Nth(minute),
        );
        let hours_spec = Specifier::new(
            *HOURS_RANGE.start(),
            *HOURS_RANGE.end(),
            SpecifierKind::Nth(hour),
        );

        match IntervalBuilder::default()
            .minutes(minutes_spec)
            .hours(hours_spec)
            .build()
        {
            Ok(interval) => Ok(interval),
            Err(err) => Err(err.to_string()),
        }
    }

    /// Checks if the provided [NaiveDate] is matched by the interval.
    ///
    /// If both wekkdays specifier and monthday specifier are not [SpecifierKind::All], only one of them has to match.
    ///
    /// # Example
    /// ```
    /// use config::interval::*;
    /// use chrono::NaiveDate;
    ///
    /// let yearly = IntervalBuilder::default()
    ///     .monthdays(Specifier::new(0, 31, SpecifierKind::Nth(10)))
    ///     .months(Specifier::new(Month::January(), Month::December(), SpecifierKind::Nth(3)))
    ///     .build()
    ///     .unwrap();
    /// assert!(yearly.matches_date(NaiveDate::from_ymd_opt(2000, 4, 11).unwrap()));
    /// assert!(yearly.matches_date(NaiveDate::from_ymd_opt(2012, 4, 11).unwrap()));
    /// assert!(yearly.matches_date(NaiveDate::from_ymd_opt(2020, 4, 11).unwrap()));
    /// assert!(!yearly.matches_date(NaiveDate::from_ymd_opt(2020, 6, 9).unwrap()));
    ///
    /// // matches both mondays and sundays *and* every 13th
    /// let week_and_monthdays = IntervalBuilder::default()
    ///     .weekdays(Specifier::new(Weekday::Monday(), Weekday::Sunday(), SpecifierKind::ExplicitNths(vec![0, 6])))
    ///     .monthdays(Specifier::new(0, 31, SpecifierKind::Nth(12)))
    ///     .build()
    ///     .unwrap();
    /// let sunday = NaiveDate::from_ymd_opt(2023, 4, 23).unwrap();
    /// assert!(week_and_monthdays.matches_date(sunday));
    /// let friday_13th = NaiveDate::from_ymd_opt(2023, 10, 13).unwrap();
    /// assert!(week_and_monthdays.matches_date(friday_13th));
    /// let wednesday_1st = NaiveDate::from_ymd_opt(2023, 3, 1).unwrap();
    /// assert!(!week_and_monthdays.matches_date(wednesday_1st));
    /// ```
    pub fn matches_date(&self, date: NaiveDate) -> bool {
        let weekday_match = self
            .weekdays
            .matches(Weekday::from(date.weekday().num_days_from_monday()));
        let monthday_match = self.monthdays.matches(date.day0());
        let day_match = if self.weekdays.kind() != &SpecifierKind::All
            && self.monthdays.kind() != &SpecifierKind::All
        {
            weekday_match || monthday_match
        } else {
            weekday_match && monthday_match
        };

        let week_match = self.weeks.matches(date.iso_week().week0());
        let month_match = self.months.matches(Month::from(date.month0()));

        day_match && week_match && month_match
    }

    /// Checks if the provided [NaiveTime] is matched by the interval
    ///
    /// # Example
    /// ```
    /// use config::interval::*;
    /// use chrono::NaiveTime;
    ///
    /// let interval = IntervalBuilder::default()
    ///     .minutes(Specifier::new(0, 59, SpecifierKind::Nth(30)))
    ///     .hours(Specifier::new(0, 23, SpecifierKind::Nth(12)))
    ///     .build()
    ///     .unwrap();
    /// let half_past_12 = NaiveTime::from_hms_opt(12, 30, 0).unwrap();
    /// assert!(interval.matches_time(half_past_12));
    /// let seven_am = NaiveTime::from_hms_opt(7, 0, 0).unwrap();
    /// assert!(!interval.matches_time(seven_am));
    /// ```
    pub fn matches_time(&self, time: NaiveTime) -> bool {
        self.minutes.matches(time.minute()) && self.hours.matches(time.hour())
    }

    /// Checks if the given [NaiveDateTime] is matched by the interval.
    /// 
    /// # Returns
    /// [DateTimeMatch::Ok] if it is matched.
    /// [DateTimeMatch::TimeNotMatched] if the date is matched but not the time.
    /// [DateTimeMatch::DateNotMatched] if the date is not matched. Is also returned when both are not matched.
    /// 
    /// # Example
    /// ```
    /// use config::interval::*;
    /// use chrono::NaiveDate;
    /// 
    /// let first_dec_7am = IntervalBuilder::default()
    ///     .minutes(Specifier::new(0, 59, SpecifierKind::First))
    ///     .hours(Specifier::new(0, 23, SpecifierKind::Nth(7)))
    ///     .monthdays(Specifier::new(0, 31, SpecifierKind::First))
    ///     .months(Specifier::new(Month::January(), Month::December(), SpecifierKind::Last))
    ///     .build()
    ///     .unwrap();
    /// 
    /// let matched_datetime = NaiveDate::from_ymd_opt(2000, 12, 1).unwrap()
    ///     .and_hms_opt(7, 0, 25).unwrap();
    /// assert_eq!(first_dec_7am.matches_datetime(matched_datetime), DateTimeMatch::Ok);
    /// 
    /// let wrong_date = NaiveDate::from_ymd_opt(2023, 9, 5).unwrap()
    ///     .and_hms_opt(7, 0, 17).unwrap();
    /// assert_eq!(first_dec_7am.matches_datetime(wrong_date), DateTimeMatch::DateNotMatched);
    /// 
    /// let wrong_time = NaiveDate::from_ymd_opt(2000, 12, 1).unwrap()
    ///     .and_hms_opt(19, 5, 33).unwrap();
    /// assert_eq!(first_dec_7am.matches_datetime(wrong_time), DateTimeMatch::TimeNotMatched);
    /// 
    /// let both_wrong = NaiveDate::from_ymd_opt(1999, 5, 27).unwrap()
    ///     .and_hms_opt(12, 49, 14).unwrap();
    /// assert_eq!(first_dec_7am.matches_datetime(both_wrong), DateTimeMatch::DateNotMatched);
    /// ```
    pub fn matches_datetime(&self, datetime: NaiveDateTime) -> DateTimeMatch {
        let date = datetime.date();
        if !self.matches_date(date) {
            return DateTimeMatch::DateNotMatched;
        }

        let time = datetime.time();
        if !self.matches_time(time) {
            return DateTimeMatch::TimeNotMatched;
        }

        DateTimeMatch::Ok
    }
}

impl IntervalBuilder {
    /// Checks that all provided ranges are valid
    pub fn validate(&self) -> Result<(), String> {
        in_range(MINUTES_RANGE, &self.minutes, "minutes")?;
        in_range(HOURS_RANGE, &self.hours, "hours")?;
        in_range(WEEKDAYS_RANGE, &self.weekdays, "weekdays")?;
        in_range(MONTHDAYS_RANGE, &self.monthdays, "monthdays")?;
        in_range(WEEKS_RANGE, &self.weeks, "weeks")?;
        in_range(MONTHS_RANGE, &self.months, "months")?;

        Ok(())
    }
}

/// Checks that the range of a given [Specifier] is inside the `reference_range`.
///
/// # Returns
/// Empty [Ok] if the [Specifier]-range is within the `reference_range`.
/// [Err] with a [String] describing the issue if the [Specifier]-range is out of the `reference_range`
fn in_range<T: Into<u32> + From<u32> + Copy + std::fmt::Debug>(
    reference_range: RangeInclusive<T>,
    spec_opt: &Option<Specifier<T>>,
    range_name: &str,
) -> Result<(), String> {
    let reference_range: RangeInclusive<u32> =
        (*reference_range.start()).into()..=(*reference_range.end()).into();
    match spec_opt {
        Some(spec) => {
            if !reference_range.contains(&spec.min().into())
                || !reference_range.contains(&spec.max().into())
            {
                Err(format!(
                    "Expect {} specifier to be in range {:?}. Got {:?}",
                    range_name,
                    reference_range,
                    spec.min()..=spec.max()
                ))
            } else {
                Ok(())
            }
        }
        None => Ok(()),
    }
}

#[cfg(test)]
mod interval_tests {
    use super::*;

    mod builder_tests {
        use super::*;

        #[test]
        fn all_defaults() {
            let interval = IntervalBuilder::default().build().unwrap();

            assert_eq!(
                interval.minutes,
                Specifier::new(
                    *MINUTES_RANGE.start(),
                    *MINUTES_RANGE.end(),
                    SpecifierKind::All
                )
            );
            assert_eq!(
                interval.hours,
                Specifier::new(*HOURS_RANGE.start(), *HOURS_RANGE.end(), SpecifierKind::All)
            );
            assert_eq!(
                interval.weekdays,
                Specifier::new(
                    *WEEKDAYS_RANGE.start(),
                    *WEEKDAYS_RANGE.end(),
                    SpecifierKind::All
                )
            );
            assert_eq!(
                interval.monthdays,
                Specifier::new(
                    *MONTHDAYS_RANGE.start(),
                    *MONTHDAYS_RANGE.end(),
                    SpecifierKind::All
                )
            );
            assert_eq!(
                interval.weeks,
                Specifier::new(*WEEKS_RANGE.start(), *WEEKS_RANGE.end(), SpecifierKind::All)
            );
            assert_eq!(
                interval.months,
                Specifier::new(
                    *MONTHS_RANGE.start(),
                    *MONTHS_RANGE.end(),
                    SpecifierKind::All
                )
            );
        }

        #[test]
        fn variations() {
            let monthdays_spec = Specifier::new(5, 17, SpecifierKind::Nth(4));
            let interval = IntervalBuilder::default()
                .monthdays(monthdays_spec.clone())
                .build()
                .unwrap();

            assert_eq!(
                interval.minutes,
                Specifier::new(
                    *MINUTES_RANGE.start(),
                    *MINUTES_RANGE.end(),
                    SpecifierKind::All
                )
            );
            assert_eq!(
                interval.hours,
                Specifier::new(*HOURS_RANGE.start(), *HOURS_RANGE.end(), SpecifierKind::All)
            );
            assert_eq!(
                interval.weekdays,
                Specifier::new(
                    *WEEKDAYS_RANGE.start(),
                    *WEEKDAYS_RANGE.end(),
                    SpecifierKind::All
                )
            );
            assert_eq!(interval.monthdays, monthdays_spec);
            assert_eq!(
                interval.weeks,
                Specifier::new(*WEEKS_RANGE.start(), *WEEKS_RANGE.end(), SpecifierKind::All)
            );
            assert_eq!(
                interval.months,
                Specifier::new(
                    *MONTHS_RANGE.start(),
                    *MONTHS_RANGE.end(),
                    SpecifierKind::All
                )
            );
        }

        #[test]
        fn invalid_range() {
            let weeks_spec = Specifier::new(0, 100, SpecifierKind::All);
            let interval = IntervalBuilder::default().weeks(weeks_spec).build();

            assert!(interval.is_err());
        }
    }

    mod matches_date_tests {
        use super::*;

        #[test]
        fn all_days() {
            let interval = IntervalBuilder::default().build().unwrap();

            let date = NaiveDate::from_ymd_opt(2023, 2, 25).unwrap();
            assert!(interval.matches_date(date));

            let date = NaiveDate::from_ymd_opt(123, 9, 12).unwrap();
            assert!(interval.matches_date(date));

            let date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
            assert!(interval.matches_date(date));

            let date = NaiveDate::from_ymd_opt(2023, 12, 31).unwrap();
            assert!(interval.matches_date(date));
        }

        #[test]
        fn no_days() {
            let interval = IntervalBuilder::default()
                .weekdays(Specifier::new(
                    Weekday::Monday(),
                    Weekday::Sunday(),
                    SpecifierKind::None,
                ))
                .monthdays(Specifier::new(0, 31, SpecifierKind::None))
                .weeks(Specifier::new(0, 52, SpecifierKind::None))
                .months(Specifier::new(
                    Month::January(),
                    Month::December(),
                    SpecifierKind::None,
                ))
                .build()
                .unwrap();

            let date = NaiveDate::from_ymd_opt(2023, 2, 25).unwrap();
            assert!(!interval.matches_date(date));

            let date = NaiveDate::from_ymd_opt(123, 9, 12).unwrap();
            assert!(!interval.matches_date(date));

            let date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
            assert!(!interval.matches_date(date));

            let date = NaiveDate::from_ymd_opt(2023, 12, 31).unwrap();
            assert!(!interval.matches_date(date));
        }

        #[test]
        fn only_jan() {
            let interval = IntervalBuilder::default()
                .months(Specifier::new(
                    Month::January(),
                    Month::December(),
                    SpecifierKind::Nth(0),
                ))
                .build()
                .unwrap();

            let date = NaiveDate::from_ymd_opt(2023, 2, 25).unwrap();
            assert!(!interval.matches_date(date));

            let date = NaiveDate::from_ymd_opt(123, 9, 12).unwrap();
            assert!(!interval.matches_date(date));

            let date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
            assert!(interval.matches_date(date));
            let date = NaiveDate::from_ymd_opt(2023, 1, 17).unwrap();
            assert!(interval.matches_date(date));
            let date = NaiveDate::from_ymd_opt(2023, 1, 31).unwrap();
            assert!(interval.matches_date(date));

            let date = NaiveDate::from_ymd_opt(2023, 12, 31).unwrap();
            assert!(!interval.matches_date(date));
        }

        #[test]
        fn every_second_week() {
            let interval = IntervalBuilder::default()
                .weeks(Specifier::new(0, 52, SpecifierKind::EveryNth(2, 0)))
                .build()
                .unwrap();
            // matches_date uses iso_weeks and 2023-01-01 is a sunday -> 0th week starts at 2023-01-02
            let date = NaiveDate::from_ymd_opt(2023, 1, 2).unwrap();
            assert!(interval.matches_date(date));
            let date = NaiveDate::from_ymd_opt(2023, 1, 5).unwrap();
            assert!(interval.matches_date(date));
            let date = NaiveDate::from_ymd_opt(2023, 1, 8).unwrap();
            assert!(interval.matches_date(date));

            let date = NaiveDate::from_ymd_opt(2023, 1, 9).unwrap();
            assert!(!interval.matches_date(date));
            let date = NaiveDate::from_ymd_opt(2023, 1, 11).unwrap();
            assert!(!interval.matches_date(date));
            let date = NaiveDate::from_ymd_opt(2023, 1, 15).unwrap();
            assert!(!interval.matches_date(date));

            let date = NaiveDate::from_ymd_opt(2023, 1, 16).unwrap();
            assert!(interval.matches_date(date));
            let date = NaiveDate::from_ymd_opt(2023, 1, 20).unwrap();
            assert!(interval.matches_date(date));
            let date = NaiveDate::from_ymd_opt(2023, 1, 22).unwrap();
            assert!(interval.matches_date(date));

            let date = NaiveDate::from_ymd_opt(2023, 7, 24).unwrap();
            assert!(!interval.matches_date(date));
            let date = NaiveDate::from_ymd_opt(2023, 10, 18).unwrap();
            assert!(!interval.matches_date(date));
            let date = NaiveDate::from_ymd_opt(2023, 12, 15).unwrap();
            assert!(!interval.matches_date(date));

            let date = NaiveDate::from_ymd_opt(2023, 12, 24).unwrap();
            assert!(interval.matches_date(date));
            let date = NaiveDate::from_ymd_opt(2024, 3, 13).unwrap();
            assert!(interval.matches_date(date));
            let date = NaiveDate::from_ymd_opt(1968, 5, 20).unwrap();
            assert!(interval.matches_date(date));
        }

        #[test]
        fn every_monday() {
            let interval = IntervalBuilder::default()
                .weekdays(Specifier::new(
                    Weekday::Monday(),
                    Weekday::Sunday(),
                    SpecifierKind::Nth(0),
                ))
                .build()
                .unwrap();

            let date = NaiveDate::from_ymd_opt(1968, 5, 20).unwrap();
            assert!(interval.matches_date(date));
            let date = NaiveDate::from_ymd_opt(1976, 11, 8).unwrap();
            assert!(interval.matches_date(date));
            let date = NaiveDate::from_ymd_opt(2000, 1, 3).unwrap();
            assert!(interval.matches_date(date));

            let date = NaiveDate::from_ymd_opt(2004, 9, 17).unwrap();
            assert!(!interval.matches_date(date));
            let date = NaiveDate::from_ymd_opt(2007, 1, 31).unwrap();
            assert!(!interval.matches_date(date));
            let date = NaiveDate::from_ymd_opt(2013, 12, 1).unwrap();
            assert!(!interval.matches_date(date));
        }

        #[test]
        fn weekdays_and_monthdays() {
            let interval = IntervalBuilder::default()
                .weekdays(Specifier::new(
                    Weekday::Monday(),
                    Weekday::Sunday(),
                    SpecifierKind::ExplicitNths(vec![5, 6]),
                ))
                .monthdays(Specifier::new(
                    0,
                    31,
                    SpecifierKind::ExplicitNths(vec![0, 9, 19, 29]),
                ))
                .build()
                .unwrap();

            // matches monthday
            let date = NaiveDate::from_ymd_opt(1900, 6, 1).unwrap();
            assert!(interval.matches_date(date));
            let date = NaiveDate::from_ymd_opt(1750, 4, 30).unwrap();
            assert!(interval.matches_date(date));
            let date = NaiveDate::from_ymd_opt(2020, 8, 20).unwrap();
            assert!(interval.matches_date(date));

            // matches weekday
            let date = NaiveDate::from_ymd_opt(2023, 2, 25).unwrap();
            assert!(interval.matches_date(date));

            // matches both
            let date = NaiveDate::from_ymd_opt(2004, 2, 1).unwrap();
            assert!(interval.matches_date(date));
            let date = NaiveDate::from_ymd_opt(2004, 10, 30).unwrap();
            assert!(interval.matches_date(date));

            // matches none
            let date = NaiveDate::from_ymd_opt(2007, 4, 18).unwrap();
            assert!(!interval.matches_date(date));
            let date = NaiveDate::from_ymd_opt(2011, 7, 26).unwrap();
            assert!(!interval.matches_date(date));
        }
    }

    mod matches_time_tests {
        use super::*;

        #[test]
        fn all_times() {
            let interval = IntervalBuilder::default().build().unwrap();

            let time = NaiveTime::from_hms_opt(12, 12, 12).unwrap();
            assert!(interval.matches_time(time));
            let time = NaiveTime::from_hms_opt(0, 40, 8).unwrap();
            assert!(interval.matches_time(time));
            let time = NaiveTime::from_hms_opt(23, 59, 59).unwrap();
            assert!(interval.matches_time(time));
            let time = NaiveTime::from_hms_opt(22, 1, 17).unwrap();
            assert!(interval.matches_time(time));
        }

        #[test]
        fn no_times() {
            let interval = IntervalBuilder::default()
                .minutes(Specifier::new(0, 59, SpecifierKind::None))
                .hours(Specifier::new(0, 23, SpecifierKind::None))
                .build()
                .unwrap();

            let time = NaiveTime::from_hms_opt(12, 12, 12).unwrap();
            assert!(!interval.matches_time(time));
            let time = NaiveTime::from_hms_opt(0, 40, 8).unwrap();
            assert!(!interval.matches_time(time));
            let time = NaiveTime::from_hms_opt(23, 59, 59).unwrap();
            assert!(!interval.matches_time(time));
            let time = NaiveTime::from_hms_opt(22, 1, 17).unwrap();
            assert!(!interval.matches_time(time));
        }

        #[test]
        fn hourly() {
            let interval = IntervalBuilder::default()
                .minutes(Specifier::new(0, 59, SpecifierKind::Nth(0)))
                .build()
                .unwrap();

            let time = NaiveTime::from_hms_opt(12, 0, 0).unwrap();
            assert!(interval.matches_time(time));
            let time = NaiveTime::from_hms_opt(13, 0, 30).unwrap();
            assert!(interval.matches_time(time));
            let time = NaiveTime::from_hms_opt(23, 0, 12).unwrap();
            assert!(interval.matches_time(time));
            let time = NaiveTime::from_hms_opt(23, 15, 12).unwrap();
            assert!(!interval.matches_time(time));
            let time = NaiveTime::from_hms_opt(1, 40, 12).unwrap();
            assert!(!interval.matches_time(time));
        }
    }

    mod matches_datetime_tests {
        use super::*;

        #[test]
        fn all_datetimes() {
            let interval = IntervalBuilder::default().build().unwrap();

            let datetime = NaiveDate::from_ymd_opt(1970, 5, 20).unwrap()
                .and_hms_opt(18, 55, 19).unwrap();
            assert_eq!(interval.matches_datetime(datetime), DateTimeMatch::Ok);
            let datetime = NaiveDate::from_ymd_opt(2001, 1, 1).unwrap()
                .and_hms_opt(0, 0, 0).unwrap();
            assert_eq!(interval.matches_datetime(datetime), DateTimeMatch::Ok);
            let datetime = NaiveDate::from_ymd_opt(2023, 8, 31).unwrap()
                .and_hms_opt(23, 59, 59).unwrap();
            assert_eq!(interval.matches_datetime(datetime), DateTimeMatch::Ok);
        }

        #[test]
        fn no_datetimes() {
            let interval = IntervalBuilder::default()
                .minutes(Specifier::new(0, 59, SpecifierKind::None))
                .months(Specifier::new(Month::January(), Month::December(), SpecifierKind::None))
                .build().unwrap();

            let datetime = NaiveDate::from_ymd_opt(1970, 5, 20).unwrap()
                .and_hms_opt(18, 55, 19).unwrap();
            assert_eq!(interval.matches_datetime(datetime), DateTimeMatch::DateNotMatched);
            let datetime = NaiveDate::from_ymd_opt(2001, 1, 1).unwrap()
                .and_hms_opt(0, 0, 0).unwrap();
            assert_eq!(interval.matches_datetime(datetime), DateTimeMatch::DateNotMatched);
            let datetime = NaiveDate::from_ymd_opt(2023, 8, 31).unwrap()
                .and_hms_opt(23, 59, 59).unwrap();
            assert_eq!(interval.matches_datetime(datetime), DateTimeMatch::DateNotMatched);
        }

        #[test]
        fn wrong_date() {
            // matches each full hour of the first day of each month
            let interval = IntervalBuilder::default()
                .minutes(Specifier::new(0, 59, SpecifierKind::First))
                .monthdays(Specifier::new(0, 31, SpecifierKind::First))
                .build().unwrap();

            let datetime = NaiveDate::from_ymd_opt(1970, 5, 20).unwrap()
                .and_hms_opt(18, 55, 19).unwrap();
            assert_eq!(interval.matches_datetime(datetime), DateTimeMatch::DateNotMatched);
            let datetime = NaiveDate::from_ymd_opt(2000, 9, 13).unwrap()
                .and_hms_opt(18, 0, 19).unwrap();
            assert_eq!(interval.matches_datetime(datetime), DateTimeMatch::DateNotMatched);
        }

        #[test]
        fn wrong_time() {
            // matches each full hour of the first day of each month
            let interval = IntervalBuilder::default()
                .minutes(Specifier::new(0, 59, SpecifierKind::First))
                .monthdays(Specifier::new(0, 31, SpecifierKind::First))
                .build().unwrap();

            let datetime = NaiveDate::from_ymd_opt(1970, 5, 1).unwrap()
                .and_hms_opt(18, 55, 19).unwrap();
            assert_eq!(interval.matches_datetime(datetime), DateTimeMatch::TimeNotMatched);
            let datetime = NaiveDate::from_ymd_opt(2000, 9, 1).unwrap()
                .and_hms_opt(3, 17, 19).unwrap();
            assert_eq!(interval.matches_datetime(datetime), DateTimeMatch::TimeNotMatched);
        }

        #[test]
        fn correct_datetime() {
            // matches each full hour of the first day of each month
            let interval = IntervalBuilder::default()
                .minutes(Specifier::new(0, 59, SpecifierKind::First))
                .monthdays(Specifier::new(0, 31, SpecifierKind::First))
                .build().unwrap();

            let datetime = NaiveDate::from_ymd_opt(1970, 5, 1).unwrap()
                .and_hms_opt(18, 0, 19).unwrap();
            assert_eq!(interval.matches_datetime(datetime), DateTimeMatch::Ok);
            let datetime = NaiveDate::from_ymd_opt(2000, 9, 1).unwrap()
                .and_hms_opt(3, 0, 19).unwrap();
            assert_eq!(interval.matches_datetime(datetime), DateTimeMatch::Ok);
        }
    }
}
