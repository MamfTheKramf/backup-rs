//! Contains struct for Weekdays

/// Number of weekdays.
/// `N-1` is the largest number, a `try_from` will work with.
pub const N: u8 = 7;

/// Struct for representing a Weekday.
/// Allows to associate numbers with days of the week starting with Monday as 0 and ending with Sunday as 6
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Weekday {
    day: u8
}

impl Weekday {
    /// Creates a weekday for the given number
    const fn new(day: u8) -> Weekday {
        Weekday { day }
    }
    
    /// Creates a weekday representing Monday
    #[allow(non_snake_case)]
    pub const fn Monday() -> Weekday {
        Weekday::new(0)
    }
    /// Creates a weekday representing Tuesday
    #[allow(non_snake_case)]
    pub const fn Tuesday() -> Weekday {
        Weekday::new(1)
    }
    /// Creates a weekday representing Wednesday
    #[allow(non_snake_case)]
    pub const fn Wednesday() -> Weekday {
        Weekday::new(2)
    }
    /// Creates a weekday representing Thursday
    #[allow(non_snake_case)]
    pub const fn Thursday() -> Weekday {
        Weekday::new(3)
    }
    /// Creates a weekday representing Friday
    #[allow(non_snake_case)]
    pub const fn Friday() -> Weekday {
        Weekday::new(4)
    }
    /// Creates a weekday representing Saturday
    #[allow(non_snake_case)]
    pub const fn Saturday() -> Weekday {
        Weekday::new(5)
    }
    /// Creates a weekday representing Sunday
    #[allow(non_snake_case)]
    pub const fn Sunday() -> Weekday {
        Weekday::new(6)
    }

    /// Turns into the weekday representing tomorrow
    /// 
    /// # Example
    /// ```
    /// use config::interval::weekdays::*;
    /// 
    /// let today = Weekday::Monday();
    /// let tomorrow = today.tomorrow();
    /// let tue = Weekday::Tuesday();
    /// assert_eq!(tomorrow, tue);
    /// ```
    pub fn tomorrow(self) -> Weekday {
        Weekday::new((self.day + 1) % 7)
    }

    /// Turns into the weekday representing tomorrow
    /// 
    /// # Example
    /// ```
    /// use config::interval::weekdays::*;
    /// 
    /// let today = Weekday::Monday();
    /// let yesterday = today.yesterday();
    /// let sun = Weekday::Sunday();
    /// assert_eq!(yesterday, sun);
    /// ```
    pub fn yesterday(self) -> Weekday {
        Weekday::new((7 + self.day - 1) % 7)
    }
}

impl Into<u32> for Weekday {
    fn into(self) -> u32 {
        self.day as u32
    }
}

impl From<u32> for Weekday {
    /// Converts [u32] cyclicly into [Weekday].
    /// 
    /// # Example
    /// ```
    /// use config::interval::weekdays::*;
    /// 
    /// assert_eq!(Weekday::from(0), Weekday::Monday());
    /// assert_eq!(Weekday::from(5), Weekday::Saturday());
    /// assert_eq!(Weekday::from(25), Weekday::Friday());
    /// ```
    fn from(value: u32) -> Self {
        Weekday::new((value % N as u32) as u8)
    }
}

#[cfg(test)]
mod weekday_tests {
    use super::*;

    #[test]
    fn sunday_tomorrow() {
        let sun = Weekday::Sunday();
        let mon = Weekday::Monday();
        assert_eq!(sun.tomorrow(), mon);
    }

    #[test]
    fn into() {
        let num: u32 = Weekday::Monday().into();
        assert_eq!(0, num);
        let num: u32 = Weekday::Tuesday().into();
        assert_eq!(1, num);
        let num: u32 = Weekday::Wednesday().into();
        assert_eq!(2, num);
        let num: u32 = Weekday::Thursday().into();
        assert_eq!(3, num);
        let num: u32 = Weekday::Friday().into();
        assert_eq!(4, num);
        let num: u32 = Weekday::Saturday().into();
        assert_eq!(5, num);
        let num: u32 = Weekday::Sunday().into();
        assert_eq!(6, num);
    }

    #[test]
    fn valid_from() {
        assert_eq!(Weekday::Monday(), Weekday::from(0));
        assert_eq!(Weekday::Tuesday(), Weekday::from(1));
        assert_eq!(Weekday::Wednesday(), Weekday::from(2));
        assert_eq!(Weekday::Thursday(), Weekday::from(3));
        assert_eq!(Weekday::Friday(), Weekday::from(4));
        assert_eq!(Weekday::Saturday(), Weekday::from(5));
        assert_eq!(Weekday::Sunday(), Weekday::from(6));
    }

    #[test]
    fn overflow_from() {
        assert_eq!(Weekday::Monday(), Weekday::from(7));
        assert_eq!(Weekday::Thursday(), Weekday::from(10));
        assert_eq!(Weekday::Sunday(), Weekday::from(13));
        assert_eq!(Weekday::Monday(), Weekday::from(70));
    }
}