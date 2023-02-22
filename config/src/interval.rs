//! Use the [Interval] struct to specify the fequency of certain events (similar to cronjobs on Linux).

use std::ops::RangeInclusive;

pub mod months;
pub mod specifier;
pub mod weekdays;

use derive_builder::Builder;
use specifier::{Specifier, SpecifierKind};

/// Allows to specify specific times in the range of every minute up to only once a year.
/// Has similar capabilities as the time specifier in crontabs on Linux
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
    /// Range 0-53
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
const MONTHDAYS_RANGE: RangeInclusive<u32> = 0..=32;
const WEEKS_RANGE: RangeInclusive<u32> = 0..=53;
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

        let minutes_spec = Specifier::new(*MINUTES_RANGE.start(), *MINUTES_RANGE.end(), SpecifierKind::Nth(minute));
        let hours_spec = Specifier::new(*HOURS_RANGE.start(), *HOURS_RANGE.end(), SpecifierKind::Nth(hour));
        
        match IntervalBuilder::default().minutes(minutes_spec).hours(hours_spec).build() {
            Ok(interval) => Ok(interval),
            Err(err) => Err(err.to_string()),
        }
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

            assert_eq!(interval.minutes, Specifier::new(*MINUTES_RANGE.start(), *MINUTES_RANGE.end(), SpecifierKind::All));
            assert_eq!(interval.hours, Specifier::new(*HOURS_RANGE.start(), *HOURS_RANGE.end(), SpecifierKind::All));
            assert_eq!(interval.weekdays, Specifier::new(*WEEKDAYS_RANGE.start(), *WEEKDAYS_RANGE.end(), SpecifierKind::All));
            assert_eq!(interval.monthdays, Specifier::new(*MONTHDAYS_RANGE.start(), *MONTHDAYS_RANGE.end(), SpecifierKind::All));
            assert_eq!(interval.weeks, Specifier::new(*WEEKS_RANGE.start(), *WEEKS_RANGE.end(), SpecifierKind::All));
            assert_eq!(interval.months, Specifier::new(*MONTHS_RANGE.start(), *MONTHS_RANGE.end(), SpecifierKind::All));
        }

        #[test]
        fn variations() {
            let monthdays_spec = Specifier::new(5, 17, SpecifierKind::Nth(4));
            let interval = IntervalBuilder::default().monthdays(monthdays_spec.clone()).build().unwrap();

            assert_eq!(interval.minutes, Specifier::new(*MINUTES_RANGE.start(), *MINUTES_RANGE.end(), SpecifierKind::All));
            assert_eq!(interval.hours, Specifier::new(*HOURS_RANGE.start(), *HOURS_RANGE.end(), SpecifierKind::All));
            assert_eq!(interval.weekdays, Specifier::new(*WEEKDAYS_RANGE.start(), *WEEKDAYS_RANGE.end(), SpecifierKind::All));
            assert_eq!(interval.monthdays, monthdays_spec);
            assert_eq!(interval.weeks, Specifier::new(*WEEKS_RANGE.start(), *WEEKS_RANGE.end(), SpecifierKind::All));
            assert_eq!(interval.months, Specifier::new(*MONTHS_RANGE.start(), *MONTHS_RANGE.end(), SpecifierKind::All));
        }

        #[test]
        fn invalid_range() {
            let weeks_spec = Specifier::new(0, 100, SpecifierKind::All);
            let interval = IntervalBuilder::default().weeks(weeks_spec).build();

            assert!(interval.is_err());
        }
    }
}