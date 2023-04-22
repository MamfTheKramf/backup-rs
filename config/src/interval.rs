//! Use the [Interval] struct to specify the fequency of certain events (similar to cronjobs on Linux).

use std::ops::RangeInclusive;

mod date_time_match;
mod months;
mod specifier;
mod weekdays;

use chrono::{Datelike, Days, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

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
#[derive(Debug, PartialEq, Builder, Serialize, Deserialize)]
pub struct Interval {
    /// Range 0-59
    #[builder(
        default = "Specifier::new(*MINUTES_RANGE.start(), *MINUTES_RANGE.end(), SpecifierKind::All)",
        setter(custom)
    )]
    pub minutes: Specifier<u32>,

    /// Range 0-23
    #[builder(
        default = "Specifier::new(*HOURS_RANGE.start(), *HOURS_RANGE.end(), SpecifierKind::All)",
        setter(custom)
    )]
    pub hours: Specifier<u32>,

    /// Range Monday-Sunday
    #[builder(
        default = "Specifier::new(*WEEKDAYS_RANGE.start(), *WEEKDAYS_RANGE.end(), SpecifierKind::All)",
        setter(custom)
    )]
    pub weekdays: Specifier<weekdays::Weekday>,

    /// Range 0-31
    #[builder(
        default = "Specifier::new(*MONTHDAYS_RANGE.start(), *MONTHDAYS_RANGE.end(), SpecifierKind::All)",
        setter(custom)
    )]
    pub monthdays: Specifier<u32>,

    /// Range 0-52
    #[builder(
        default = "Specifier::new(*WEEKS_RANGE.start(), *WEEKS_RANGE.end(), SpecifierKind::All)",
        setter(custom)
    )]
    pub weeks: Specifier<u32>,

    /// Range January-December
    #[builder(
        default = "Specifier::new(*MONTHS_RANGE.start(), *MONTHS_RANGE.end(), SpecifierKind::All)",
        setter(custom)
    )]
    pub months: Specifier<months::Month>,
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

        let minutes_spec = SpecifierKind::Nth(minute);
        let hours_spec = SpecifierKind::Nth(hour);

        match IntervalBuilder::default()
            .minutes(minutes_spec)
            .hours(hours_spec)
            .build()
        {
            Ok(interval) => Ok(interval),
            Err(err) => Err(err.to_string()),
        }
    }

    /// Checks that the [Specifier]s all have the correct ranges.
    /// This is already enforced when using the [IntervalBuilder]. However, when deserializing a JSON there might be some wrong values.
    ///
    /// # Returns
    /// [Ok] if everything is alright.
    /// [Err] describing a missconfigured specifier. It will only report one specifier hat a time.
    ///
    /// # Example
    /// ```
    /// use config::interval::*;
    ///
    /// let interval = Interval {
    ///     minutes: Specifier::new(0, 59, SpecifierKind::All),
    ///     hours: Specifier::new(0, 23, SpecifierKind::All),
    ///     weekdays: Specifier::new(Weekday::Monday(), Weekday::Sunday(), SpecifierKind::All),
    ///     monthdays: Specifier::new(0, 31, SpecifierKind::All),
    ///     weeks: Specifier::new(0, 52, SpecifierKind::All),
    ///     months: Specifier::new(Month::January(), Month::December(), SpecifierKind::All)
    /// };
    /// assert!(interval.validate().is_ok());
    ///
    /// let bad_interval = Interval {
    ///     minutes: Specifier::new(0, 100, SpecifierKind::All),
    ///     hours: Specifier::new(0, 23, SpecifierKind::All),
    ///     weekdays: Specifier::new(Weekday::Monday(), Weekday::Sunday(), SpecifierKind::All),
    ///     monthdays: Specifier::new(0, 31, SpecifierKind::All),
    ///     weeks: Specifier::new(0, 52, SpecifierKind::All),
    ///     months: Specifier::new(Month::January(), Month::December(), SpecifierKind::All)
    /// };
    /// assert!(bad_interval.validate().is_err());
    /// ```
    pub fn validate(&self) -> Result<(), String> {
        let own_range = self.minutes.min()..=self.minutes.max();
        if own_range != MINUTES_RANGE {
            return Err(format!("Minutes are not in range {:?}. Got {:?}", MINUTES_RANGE, own_range));
        }

        let own_range = self.hours.min()..=self.hours.max();
        if own_range != HOURS_RANGE {
            return Err(format!("Hours are not in range {:?}. Got {:?}", HOURS_RANGE, own_range));
        }

        let own_range = self.weekdays.min()..=self.weekdays.max();
        if own_range != WEEKDAYS_RANGE {
            return Err(format!("Weekdays are not in range {:?}. Got {:?}", WEEKDAYS_RANGE, own_range));
        }

        let own_range = self.monthdays.min()..=self.monthdays.max();
        if own_range != MONTHDAYS_RANGE {
            return Err(format!("Monthdays are not in range {:?}. Got {:?}", MONTHDAYS_RANGE, own_range));
        }

        let own_range = self.weeks.min()..=self.weeks.max();
        if own_range != WEEKS_RANGE {
            return Err(format!("Weeks are not in range {:?}. Got {:?}", WEEKS_RANGE, own_range));
        }

        let own_range = self.months.min()..=self.months.max();
        if own_range != MONTHS_RANGE {
            return Err(format!("Months are not in range {:?}. Got {:?}", MONTHS_RANGE, own_range));
        }
    
        Ok(())
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
    ///     .monthdays(SpecifierKind::Nth(10))
    ///     .months(SpecifierKind::Nth(3))
    ///     .build()
    ///     .unwrap();
    /// assert!(yearly.matches_date(NaiveDate::from_ymd_opt(2000, 4, 11).unwrap()));
    /// assert!(yearly.matches_date(NaiveDate::from_ymd_opt(2012, 4, 11).unwrap()));
    /// assert!(yearly.matches_date(NaiveDate::from_ymd_opt(2020, 4, 11).unwrap()));
    /// assert!(!yearly.matches_date(NaiveDate::from_ymd_opt(2020, 6, 9).unwrap()));
    ///
    /// // matches both mondays and sundays *and* every 13th
    /// let week_and_monthdays = IntervalBuilder::default()
    ///     .weekdays(SpecifierKind::ExplicitNths(vec![0, 6]))
    ///     .monthdays(SpecifierKind::Nth(12))
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
    ///     .minutes(SpecifierKind::Nth(30))
    ///     .hours(SpecifierKind::Nth(12))
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
    ///     .minutes(SpecifierKind::First)
    ///     .hours(SpecifierKind::Nth(7))
    ///     .monthdays(SpecifierKind::First)
    ///     .months(SpecifierKind::Last)
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

    /// Checks if any of the specifiers are [SpecifierKind::None]
    pub fn has_none_specifier(&self) -> bool {
        return self.minutes.kind() == &SpecifierKind::None
            || self.hours.kind() == &SpecifierKind::None
            || self.weekdays.kind() == &SpecifierKind::None
            || self.monthdays.kind() == &SpecifierKind::None
            || self.weeks.kind() == &SpecifierKind::None
            || self.months.kind() == &SpecifierKind::None;
    }

    /// Returns the next matching time of day after the given time, if one exists.
    /// All returned [NaiveTime]s have their seconds-value set to `0`.
    ///
    /// # Example
    /// ```
    /// use config::interval::*;
    /// use chrono::NaiveTime;
    ///
    /// let noon = IntervalBuilder::default()
    ///     .minutes(SpecifierKind::First)
    ///     .hours(SpecifierKind::Nth(12))
    ///     .build()
    ///     .unwrap();
    ///
    /// let morning = NaiveTime::from_hms_opt(6, 0, 0).unwrap();
    /// assert_eq!(noon.next_daytime(morning).unwrap(), NaiveTime::from_hms_opt(12, 0, 0).unwrap());
    ///
    /// let afternoon = NaiveTime::from_hms_opt(15, 0, 0).unwrap();
    /// assert!(noon.next_daytime(afternoon).is_none());
    /// ```
    pub fn next_daytime(&self, time: NaiveTime) -> Option<NaiveTime> {
        let cyclic_next_match = self.cyclic_next_daytime(time)?;
        if cyclic_next_match <= time {
            return None;
        }
        Some(cyclic_next_match)
    }

    /// Returns the next matching time of the day or cycles to the next day if needed.
    /// All returned [NaiveTime]s have their seconds-value set to `0`.
    ///
    /// # Example
    /// ```
    /// use config::interval::*;
    /// use chrono::NaiveTime;
    ///
    /// let noon = IntervalBuilder::default()
    ///     .minutes(SpecifierKind::First)
    ///     .hours(SpecifierKind::Nth(12))
    ///     .build()
    ///     .unwrap();
    ///
    /// let morning = NaiveTime::from_hms_opt(6, 0, 0).unwrap();
    /// assert_eq!(noon.cyclic_next_daytime(morning).unwrap(), NaiveTime::from_hms_opt(12, 0, 0).unwrap());
    ///
    /// let afternoon = NaiveTime::from_hms_opt(15, 0, 0).unwrap();
    /// assert_eq!(noon.cyclic_next_daytime(afternoon).unwrap(), NaiveTime::from_hms_opt(12, 0, 0).unwrap());
    /// ```
    pub fn cyclic_next_daytime(&self, time: NaiveTime) -> Option<NaiveTime> {
        let hour_matches = self.hours.matches(time.hour());

        let next_minute = if hour_matches {
            // we can try to find the next match in the same hour
            self.minutes.cyclic_next(time.minute())?
        } else {
            // we have to cycle anyways
            self.minutes.first_match()?
        };

        let next_hour = if hour_matches && next_minute > time.minute() {
            // hour does match and we found a matching minute in this hour
            time.hour()
        } else {
            self.hours.cyclic_next(time.hour())?
        };

        return NaiveTime::from_hms_opt(next_hour, next_minute, 0);
    }

    /// Tries to find the next matching [NaiveDateTime] after the provided `datetime`.
    /// Only tries to find a match within 1 year (365 days) from the provided `datetime`.
    ///
    /// The seconds-value of a returned value will always be `0`.
    ///
    /// # Returns
    /// `Some` variant containing a [NaiveDateTime] representing the next matching `datetime` after the provided if one is found.
    /// `None` if there is no matching datetime within the next year.
    ///
    /// # Example
    /// ```
    /// use config::interval::*;
    /// use chrono::{ NaiveDate, NaiveTime };
    ///
    /// let first_of_month = IntervalBuilder::default()
    ///     .minutes(SpecifierKind::First)
    ///     .hours(SpecifierKind::First)
    ///     .monthdays(SpecifierKind::First)
    ///     .build()
    ///     .unwrap();
    /// let feb_1st = NaiveDate::from_ymd_opt(2003, 2, 1).unwrap()
    ///     .and_hms_opt(0, 0, 0).unwrap();
    /// assert_eq!(first_of_month.next_datetime(feb_1st).unwrap().date(), NaiveDate::from_ymd_opt(2003, 3, 1).unwrap());
    ///
    /// let daily = IntervalBuilder::default()
    ///     .minutes(SpecifierKind::First)
    ///     .hours(SpecifierKind::Nth(12))
    ///     .build()
    ///     .unwrap();
    /// let noon = daily.next_datetime(feb_1st).unwrap();
    /// assert_eq!(noon.date(), feb_1st.date());
    /// assert_eq!(noon.time(), NaiveTime::from_hms_opt(12, 0, 0).unwrap());
    /// ```
    pub fn next_datetime(&self, datetime: NaiveDateTime) -> Option<NaiveDateTime> {
        let date_matches = self.matches_date(datetime.date());

        let next_time = if date_matches {
            self.cyclic_next_daytime(datetime.time())?
        } else {
            self.cyclic_next_daytime(NaiveTime::from_hms_opt(23, 59, 59)?)?
        };

        let next_date = if date_matches && next_time > datetime.time() {
            datetime.date()
        } else {
            let mut curr_date = datetime.date().checked_add_days(Days::new(1))?;
            let mut matched_date = false;
            for _ in 0..365 {
                if self.matches_date(curr_date) {
                    matched_date = true;
                    break;
                }

                curr_date = curr_date.checked_add_days(Days::new(1))?;
            }
            if !matched_date {
                return None;
            }
            curr_date
        };

        Some(next_date.and_time(next_time))
    }
}

impl IntervalBuilder {
    pub fn minutes(&mut self, spec_kind: SpecifierKind) -> &mut Self {
        self.minutes = Some(Specifier::new(
            *MINUTES_RANGE.start(),
            *MINUTES_RANGE.end(),
            spec_kind,
        ));
        self
    }

    pub fn hours(&mut self, spec_kind: SpecifierKind) -> &mut Self {
        self.hours = Some(Specifier::new(
            *HOURS_RANGE.start(),
            *HOURS_RANGE.end(),
            spec_kind,
        ));
        self
    }

    pub fn weekdays(&mut self, spec_kind: SpecifierKind) -> &mut Self {
        self.weekdays = Some(Specifier::new(
            *WEEKDAYS_RANGE.start(),
            *WEEKDAYS_RANGE.end(),
            spec_kind,
        ));
        self
    }

    pub fn monthdays(&mut self, spec_kind: SpecifierKind) -> &mut Self {
        self.monthdays = Some(Specifier::new(
            *MONTHDAYS_RANGE.start(),
            *MONTHDAYS_RANGE.end(),
            spec_kind,
        ));
        self
    }

    pub fn weeks(&mut self, spec_kind: SpecifierKind) -> &mut Self {
        self.weeks = Some(Specifier::new(
            *WEEKS_RANGE.start(),
            *WEEKS_RANGE.end(),
            spec_kind,
        ));
        self
    }

    pub fn months(&mut self, spec_kind: SpecifierKind) -> &mut Self {
        self.months = Some(Specifier::new(
            *MONTHS_RANGE.start(),
            *MONTHS_RANGE.end(),
            spec_kind,
        ));
        self
    }
}

#[cfg(test)]
mod interval_tests {
    use super::*;

    mod validate_test {
        use super::*;

        #[test]
        fn valid_interval() {
            let interval = Interval {
                minutes: Specifier::new(0, 59, SpecifierKind::All),
                hours: Specifier::new(0, 23, SpecifierKind::All),
                weekdays: Specifier::new(Weekday::Monday(), Weekday::Sunday(), SpecifierKind::All),
                monthdays: Specifier::new(0, 31, SpecifierKind::All),
                weeks: Specifier::new(0, 52, SpecifierKind::All),
                months: Specifier::new(Month::January(), Month::December(), SpecifierKind::All),
            };

            assert!(interval.validate().is_ok());
        }

        #[test]
        fn invalid_u32_interval() {
            let mut interval = Interval {
                minutes: Specifier::new(0, 59, SpecifierKind::All),
                hours: Specifier::new(0, 23, SpecifierKind::All),
                weekdays: Specifier::new(Weekday::Monday(), Weekday::Sunday(), SpecifierKind::All),
                monthdays: Specifier::new(17, 31, SpecifierKind::All),
                weeks: Specifier::new(0, 52, SpecifierKind::All),
                months: Specifier::new(Month::January(), Month::December(), SpecifierKind::All),
            };

            assert!(interval.validate().is_err());

            interval.monthdays = Specifier::new(0, 31, SpecifierKind::All);
            interval.minutes = Specifier::new(0, 100, SpecifierKind::All);

            assert!(interval.validate().is_err());

            interval.minutes = Specifier::new(200, 18, SpecifierKind::All);

            assert!(interval.validate().is_err());
        }

        #[test]
        fn invalid_struct_interval() {
            let mut interval = Interval {
                minutes: Specifier::new(0, 59, SpecifierKind::All),
                hours: Specifier::new(0, 23, SpecifierKind::All),
                weekdays: Specifier::new(Weekday::Wednesday(), Weekday::Sunday(), SpecifierKind::All),
                monthdays: Specifier::new(0, 31, SpecifierKind::All),
                weeks: Specifier::new(0, 52, SpecifierKind::All),
                months: Specifier::new(Month::January(), Month::December(), SpecifierKind::All),
            };

            assert!(interval.validate().is_err());

            interval.weekdays = Specifier::new(Weekday::Monday(), Weekday::Sunday(), SpecifierKind::All);
            interval.months = Specifier::new(Month::January(), Month::October(), SpecifierKind::All);

            assert!(interval.validate().is_err());

            interval.months = Specifier::new(Month::November(), Month::April(), SpecifierKind::All);

            assert!(interval.validate().is_err());
        }
    }

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
            let monthdays_spec = SpecifierKind::Nth(4);
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
            assert_eq!(interval.monthdays.kind(), &monthdays_spec);
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
                .weekdays(SpecifierKind::None)
                .monthdays(SpecifierKind::None)
                .weeks(SpecifierKind::None)
                .months(SpecifierKind::None)
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
                .months(SpecifierKind::Nth(0))
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
                .weeks(SpecifierKind::EveryNth(2, 0))
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
                .weekdays(SpecifierKind::Nth(0))
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
                .weekdays(SpecifierKind::ExplicitNths(vec![5, 6]))
                .monthdays(SpecifierKind::ExplicitNths(vec![0, 9, 19, 29]))
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
                .minutes(SpecifierKind::None)
                .hours(SpecifierKind::None)
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
                .minutes(SpecifierKind::Nth(0))
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

            let datetime = NaiveDate::from_ymd_opt(1970, 5, 20)
                .unwrap()
                .and_hms_opt(18, 55, 19)
                .unwrap();
            assert_eq!(interval.matches_datetime(datetime), DateTimeMatch::Ok);
            let datetime = NaiveDate::from_ymd_opt(2001, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap();
            assert_eq!(interval.matches_datetime(datetime), DateTimeMatch::Ok);
            let datetime = NaiveDate::from_ymd_opt(2023, 8, 31)
                .unwrap()
                .and_hms_opt(23, 59, 59)
                .unwrap();
            assert_eq!(interval.matches_datetime(datetime), DateTimeMatch::Ok);
        }

        #[test]
        fn no_datetimes() {
            let interval = IntervalBuilder::default()
                .minutes(SpecifierKind::None)
                .months(SpecifierKind::None)
                .build()
                .unwrap();

            let datetime = NaiveDate::from_ymd_opt(1970, 5, 20)
                .unwrap()
                .and_hms_opt(18, 55, 19)
                .unwrap();
            assert_eq!(
                interval.matches_datetime(datetime),
                DateTimeMatch::DateNotMatched
            );
            let datetime = NaiveDate::from_ymd_opt(2001, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap();
            assert_eq!(
                interval.matches_datetime(datetime),
                DateTimeMatch::DateNotMatched
            );
            let datetime = NaiveDate::from_ymd_opt(2023, 8, 31)
                .unwrap()
                .and_hms_opt(23, 59, 59)
                .unwrap();
            assert_eq!(
                interval.matches_datetime(datetime),
                DateTimeMatch::DateNotMatched
            );
        }

        #[test]
        fn wrong_date() {
            // matches each full hour of the first day of each month
            let interval = IntervalBuilder::default()
                .minutes(SpecifierKind::First)
                .monthdays(SpecifierKind::First)
                .build()
                .unwrap();

            let datetime = NaiveDate::from_ymd_opt(1970, 5, 20)
                .unwrap()
                .and_hms_opt(18, 55, 19)
                .unwrap();
            assert_eq!(
                interval.matches_datetime(datetime),
                DateTimeMatch::DateNotMatched
            );
            let datetime = NaiveDate::from_ymd_opt(2000, 9, 13)
                .unwrap()
                .and_hms_opt(18, 0, 19)
                .unwrap();
            assert_eq!(
                interval.matches_datetime(datetime),
                DateTimeMatch::DateNotMatched
            );
        }

        #[test]
        fn wrong_time() {
            // matches each full hour of the first day of each month
            let interval = IntervalBuilder::default()
                .minutes(SpecifierKind::First)
                .monthdays(SpecifierKind::First)
                .build()
                .unwrap();

            let datetime = NaiveDate::from_ymd_opt(1970, 5, 1)
                .unwrap()
                .and_hms_opt(18, 55, 19)
                .unwrap();
            assert_eq!(
                interval.matches_datetime(datetime),
                DateTimeMatch::TimeNotMatched
            );
            let datetime = NaiveDate::from_ymd_opt(2000, 9, 1)
                .unwrap()
                .and_hms_opt(3, 17, 19)
                .unwrap();
            assert_eq!(
                interval.matches_datetime(datetime),
                DateTimeMatch::TimeNotMatched
            );
        }

        #[test]
        fn correct_datetime() {
            // matches each full hour of the first day of each month
            let interval = IntervalBuilder::default()
                .minutes(SpecifierKind::First)
                .monthdays(SpecifierKind::First)
                .build()
                .unwrap();

            let datetime = NaiveDate::from_ymd_opt(1970, 5, 1)
                .unwrap()
                .and_hms_opt(18, 0, 19)
                .unwrap();
            assert_eq!(interval.matches_datetime(datetime), DateTimeMatch::Ok);
            let datetime = NaiveDate::from_ymd_opt(2000, 9, 1)
                .unwrap()
                .and_hms_opt(3, 0, 19)
                .unwrap();
            assert_eq!(interval.matches_datetime(datetime), DateTimeMatch::Ok);
        }
    }

    mod next_daytime_tests {
        use super::*;

        #[test]
        fn no_match() {
            let interval = IntervalBuilder::default()
                .minutes(SpecifierKind::None)
                .build()
                .unwrap();

            let sandman_time = NaiveTime::from_hms_opt(19, 0, 0).unwrap();
            assert!(interval.next_daytime(sandman_time).is_none());

            let time = NaiveTime::from_hms_opt(18, 16, 0).unwrap();
            assert!(interval.next_daytime(time).is_none());

            let time = NaiveTime::from_hms_opt(18, 15, 0).unwrap();
            assert!(interval.next_daytime(time).is_none());
        }

        #[test]
        fn next_match_tomorrow() {
            let interval = IntervalBuilder::default()
                .minutes(SpecifierKind::Nth(15))
                .hours(SpecifierKind::ExplicitNths(vec![0, 6, 12, 18]))
                .build()
                .unwrap();

            let sandman_time = NaiveTime::from_hms_opt(19, 0, 0).unwrap();
            assert!(interval.next_daytime(sandman_time).is_none());

            let little_too_late = NaiveTime::from_hms_opt(18, 16, 0).unwrap();
            assert!(interval.next_daytime(little_too_late).is_none());

            let last_match = NaiveTime::from_hms_opt(18, 15, 0).unwrap();
            assert!(interval.next_daytime(last_match).is_none());
        }

        #[test]
        fn only_midnight() {
            let interval = IntervalBuilder::default()
                .minutes(SpecifierKind::First)
                .hours(SpecifierKind::First)
                .build()
                .unwrap();

            let midnight = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
            assert!(interval.next_daytime(midnight).is_none());

            let time = NaiveTime::from_hms_opt(14, 39, 55).unwrap();
            assert!(interval.next_daytime(time).is_none());
        }

        #[test]
        fn in_same_hour() {
            let interval = IntervalBuilder::default()
                .minutes(SpecifierKind::Nth(15))
                .hours(SpecifierKind::ExplicitNths(vec![0, 6, 12, 18]))
                .build()
                .unwrap();

            let time = NaiveTime::from_hms_opt(6, 0, 0).unwrap();
            assert_eq!(
                interval.next_daytime(time).unwrap(),
                NaiveTime::from_hms_opt(6, 15, 0).unwrap()
            );

            let interval = IntervalBuilder::default()
                .minutes(SpecifierKind::EveryNth(10, 0))
                .build()
                .unwrap();
            let time = NaiveTime::from_hms_opt(15, 35, 0).unwrap();
            assert_eq!(
                interval.next_daytime(time).unwrap(),
                NaiveTime::from_hms_opt(15, 40, 0).unwrap()
            );
        }

        #[test]
        fn no_hour_match() {
            let interval = IntervalBuilder::default()
                .minutes(SpecifierKind::Nth(15))
                .hours(SpecifierKind::ExplicitNths(vec![0, 6, 12, 18]))
                .build()
                .unwrap();

            let time = NaiveTime::from_hms_opt(7, 37, 0).unwrap();
            assert_eq!(
                interval.next_daytime(time).unwrap(),
                NaiveTime::from_hms_opt(12, 15, 0).unwrap()
            );
        }

        #[test]
        fn hour_match_next_hour() {
            let interval = IntervalBuilder::default()
                .minutes(SpecifierKind::Nth(15))
                .hours(SpecifierKind::ExplicitNths(vec![0, 6, 12, 18]))
                .build()
                .unwrap();

            let time = NaiveTime::from_hms_opt(6, 37, 0).unwrap();
            assert_eq!(
                interval.next_daytime(time).unwrap(),
                NaiveTime::from_hms_opt(12, 15, 0).unwrap()
            );
        }
    }

    mod cyclic_next_daytime_tests {
        use super::*;

        #[test]
        fn no_match() {
            let interval = IntervalBuilder::default()
                .minutes(SpecifierKind::None)
                .build()
                .unwrap();

            let sandman_time = NaiveTime::from_hms_opt(19, 0, 0).unwrap();
            assert!(interval.cyclic_next_daytime(sandman_time).is_none());

            let time = NaiveTime::from_hms_opt(18, 16, 0).unwrap();
            assert!(interval.cyclic_next_daytime(time).is_none());

            let time = NaiveTime::from_hms_opt(18, 15, 0).unwrap();
            assert!(interval.cyclic_next_daytime(time).is_none());
        }

        #[test]
        fn next_match_tomorrow() {
            let interval = IntervalBuilder::default()
                .minutes(SpecifierKind::Nth(15))
                .hours(SpecifierKind::ExplicitNths(vec![0, 6, 12, 18]))
                .build()
                .unwrap();

            let sandman_time = NaiveTime::from_hms_opt(19, 0, 0).unwrap();
            assert_eq!(
                interval.cyclic_next_daytime(sandman_time),
                NaiveTime::from_hms_opt(0, 15, 0)
            );

            let little_too_late = NaiveTime::from_hms_opt(18, 16, 0).unwrap();
            assert_eq!(
                interval.cyclic_next_daytime(little_too_late),
                NaiveTime::from_hms_opt(0, 15, 0)
            );

            let last_match = NaiveTime::from_hms_opt(18, 15, 0).unwrap();
            assert_eq!(
                interval.cyclic_next_daytime(last_match),
                NaiveTime::from_hms_opt(0, 15, 0)
            );
        }

        #[test]
        fn only_midnight() {
            let interval = IntervalBuilder::default()
                .minutes(SpecifierKind::First)
                .hours(SpecifierKind::First)
                .build()
                .unwrap();

            let midnight = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
            assert_eq!(
                interval.cyclic_next_daytime(midnight),
                NaiveTime::from_hms_opt(0, 0, 0)
            );

            let time = NaiveTime::from_hms_opt(14, 39, 55).unwrap();
            assert_eq!(
                interval.cyclic_next_daytime(time),
                NaiveTime::from_hms_opt(0, 0, 0)
            );
        }

        #[test]
        fn in_same_hour() {
            let interval = IntervalBuilder::default()
                .minutes(SpecifierKind::Nth(15))
                .hours(SpecifierKind::ExplicitNths(vec![0, 6, 12, 18]))
                .build()
                .unwrap();

            let time = NaiveTime::from_hms_opt(6, 0, 0).unwrap();
            assert_eq!(
                interval.cyclic_next_daytime(time).unwrap(),
                NaiveTime::from_hms_opt(6, 15, 0).unwrap()
            );

            let interval = IntervalBuilder::default()
                .minutes(SpecifierKind::EveryNth(10, 0))
                .build()
                .unwrap();
            let time = NaiveTime::from_hms_opt(15, 35, 0).unwrap();
            assert_eq!(
                interval.cyclic_next_daytime(time).unwrap(),
                NaiveTime::from_hms_opt(15, 40, 0).unwrap()
            );
        }

        #[test]
        fn no_hour_match() {
            let interval = IntervalBuilder::default()
                .minutes(SpecifierKind::Nth(15))
                .hours(SpecifierKind::ExplicitNths(vec![0, 6, 12, 18]))
                .build()
                .unwrap();

            let time = NaiveTime::from_hms_opt(7, 37, 0).unwrap();
            assert_eq!(
                interval.cyclic_next_daytime(time).unwrap(),
                NaiveTime::from_hms_opt(12, 15, 0).unwrap()
            );
        }

        #[test]
        fn hour_match_next_hour() {
            let interval = IntervalBuilder::default()
                .minutes(SpecifierKind::Nth(15))
                .hours(SpecifierKind::ExplicitNths(vec![0, 6, 12, 18]))
                .build()
                .unwrap();

            let time = NaiveTime::from_hms_opt(6, 37, 0).unwrap();
            assert_eq!(
                interval.cyclic_next_daytime(time).unwrap(),
                NaiveTime::from_hms_opt(12, 15, 0).unwrap()
            );
        }
    }

    mod next_datetime_tests {
        use super::*;

        #[test]
        fn no_match() {
            let interval = IntervalBuilder::default()
                .weekdays(SpecifierKind::None)
                .build()
                .unwrap();

            let datetime = NaiveDate::from_ymd_opt(2023, 9, 9)
                .unwrap()
                .and_hms_opt(12, 40, 29)
                .unwrap();
            assert!(interval.next_datetime(datetime).is_none());

            let datetime = NaiveDate::from_ymd_opt(1990, 1, 31)
                .unwrap()
                .and_hms_opt(0, 12, 18)
                .unwrap();
            assert!(interval.next_datetime(datetime).is_none());
        }

        #[test]
        fn not_within_a_year() {
            let interval = IntervalBuilder::default()
                .monthdays(SpecifierKind::Nth(28))
                .months(SpecifierKind::Nth(1))
                .build()
                .unwrap();

            let datetime = NaiveDate::from_ymd_opt(2021, 4, 8)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap();
            assert!(interval.next_datetime(datetime).is_none());
        }

        #[test]
        fn valid_interval() {
            let interval = IntervalBuilder::default()
                .monthdays(SpecifierKind::Nth(28))
                .months(SpecifierKind::Nth(1))
                .build()
                .unwrap();

            let datetime = NaiveDate::from_ymd_opt(2020, 2, 20)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap();
            assert_eq!(
                interval.next_datetime(datetime),
                NaiveDate::from_ymd_opt(2020, 2, 29)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
            );
            let datetime = NaiveDate::from_ymd_opt(2019, 4, 8)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap();
            assert_eq!(
                interval.next_datetime(datetime),
                NaiveDate::from_ymd_opt(2020, 2, 29)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
            );
            let datetime = NaiveDate::from_ymd_opt(2020, 1, 5)
                .unwrap()
                .and_hms_opt(20, 55, 36)
                .unwrap();
            assert_eq!(
                interval.next_datetime(datetime),
                NaiveDate::from_ymd_opt(2020, 2, 29)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
            );
        }

        #[test]
        fn same_day() {
            let interval = IntervalBuilder::default()
                .minutes(SpecifierKind::First)
                .build()
                .unwrap();

            let datetime = NaiveDate::from_ymd_opt(2019, 4, 8)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap();
            assert_eq!(
                interval.next_datetime(datetime),
                NaiveDate::from_ymd_opt(2019, 4, 8)
                    .unwrap()
                    .and_hms_opt(1, 0, 0)
            );
            let datetime = NaiveDate::from_ymd_opt(2023, 8, 5)
                .unwrap()
                .and_hms_opt(20, 35, 55)
                .unwrap();
            assert_eq!(
                interval.next_datetime(datetime),
                NaiveDate::from_ymd_opt(2023, 8, 5)
                    .unwrap()
                    .and_hms_opt(21, 0, 0)
            );
        }

        #[test]
        fn next_day() {
            let interval = IntervalBuilder::default()
                .minutes(SpecifierKind::First)
                .build()
                .unwrap();

            let datetime = NaiveDate::from_ymd_opt(2019, 4, 8)
                .unwrap()
                .and_hms_opt(23, 0, 0)
                .unwrap();
            assert_eq!(
                interval.next_datetime(datetime),
                NaiveDate::from_ymd_opt(2019, 4, 9)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
            );
            let datetime = NaiveDate::from_ymd_opt(2023, 8, 5)
                .unwrap()
                .and_hms_opt(23, 35, 55)
                .unwrap();
            assert_eq!(
                interval.next_datetime(datetime),
                NaiveDate::from_ymd_opt(2023, 8, 6)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
            );
        }
    }
}
