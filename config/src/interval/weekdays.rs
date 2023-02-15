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
    fn new(day: u8) -> Weekday {
        Weekday { day }
    }
    
    /// Creates a weekday representing Monday
    #[allow(non_snake_case)]
    pub fn Monday() -> Weekday {
        Weekday::new(0)
    }
    /// Creates a weekday representing Tuesday
    #[allow(non_snake_case)]
    pub fn Tuesday() -> Weekday {
        Weekday::new(1)
    }
    /// Creates a weekday representing Wednesday
    #[allow(non_snake_case)]
    pub fn Wednesday() -> Weekday {
        Weekday::new(2)
    }
    /// Creates a weekday representing Thursday
    #[allow(non_snake_case)]
    pub fn Thursday() -> Weekday {
        Weekday::new(3)
    }
    /// Creates a weekday representing Friday
    #[allow(non_snake_case)]
    pub fn Friday() -> Weekday {
        Weekday::new(4)
    }
    /// Creates a weekday representing Saturday
    #[allow(non_snake_case)]
    pub fn Saturday() -> Weekday {
        Weekday::new(5)
    }
    /// Creates a weekday representing Sunday
    #[allow(non_snake_case)]
    pub fn Sunday() -> Weekday {
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

impl TryFrom<u8> for Weekday {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value >= N {
            Err(format!("Can only convert numbers between 0 and {} (inclusive) into weekdays. Got {}", N - 1, value))
        } else {
            Ok(Weekday::new(value))
        }
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
        assert_eq!(Weekday::Monday(), Weekday::try_from(0).unwrap());
        assert_eq!(Weekday::Tuesday(), Weekday::try_from(1).unwrap());
        assert_eq!(Weekday::Wednesday(), Weekday::try_from(2).unwrap());
        assert_eq!(Weekday::Thursday(), Weekday::try_from(3).unwrap());
        assert_eq!(Weekday::Friday(), Weekday::try_from(4).unwrap());
        assert_eq!(Weekday::Saturday(), Weekday::try_from(5).unwrap());
        assert_eq!(Weekday::Sunday(), Weekday::try_from(6).unwrap());
    }

    #[test]
    fn invalid_from() {
        assert!(Weekday::try_from(17).is_err());
        assert!(Weekday::try_from(7).is_err());
        assert!(Weekday::try_from(255).is_err());
        assert!(Weekday::try_from(128).is_err());
    }
}