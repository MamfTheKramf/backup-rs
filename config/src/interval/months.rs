//! Contains struct for Months

/// Number of months.
/// `N-1` is the largest number, a `try_from` will work with.
pub const N: u8 = 12;

/// Struct for representing a Month.
/// Allows to associate numbers with days of the week starting with January as 0 and ending with December as 12
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Month {
    month: u8
}

impl Month {
    /// Creates a weekday for the given number
    fn new(month: u8) -> Month {
        Month { month }
    }
    
    /// Creates a weekday representing January
    #[allow(non_snake_case)]
    pub fn January() -> Month {
        Month::new(0)
    }
    /// Creates a weekday representing February
    #[allow(non_snake_case)]
    pub fn February() -> Month {
        Month::new(1)
    }
    /// Creates a weekday representing March
    #[allow(non_snake_case)]
    pub fn March() -> Month {
        Month::new(2)
    }
    /// Creates a weekday representing April
    #[allow(non_snake_case)]
    pub fn April() -> Month {
        Month::new(3)
    }
    /// Creates a weekday representing May
    #[allow(non_snake_case)]
    pub fn May() -> Month {
        Month::new(4)
    }
    /// Creates a weekday representing June
    #[allow(non_snake_case)]
    pub fn June() -> Month {
        Month::new(5)
    }
    /// Creates a weekday representing July
    #[allow(non_snake_case)]
    pub fn July() -> Month {
        Month::new(6)
    }
    /// Creates a weekday representing August
    #[allow(non_snake_case)]
    pub fn August() -> Month {
        Month::new(7)
    }
    /// Creates a weekday representing September
    #[allow(non_snake_case)]
    pub fn September() -> Month {
        Month::new(8)
    }
    /// Creates a weekday representing October
    #[allow(non_snake_case)]
    pub fn October() -> Month {
        Month::new(9)
    }
    /// Creates a weekday representing November
    #[allow(non_snake_case)]
    pub fn November() -> Month {
        Month::new(10)
    }
    /// Creates a weekday representing December
    #[allow(non_snake_case)]
    pub fn December() -> Month {
        Month::new(11)
    }
}

impl Into<u32> for Month {
    fn into(self) -> u32 {
        self.month as u32
    }
}

impl TryFrom<u8> for Month {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value >= N {
            Err(format!("Can only convert numbers between 0 and {} (inclusive) into months. Got {}", N - 1, value))
        } else {
            Ok(Month::new(value))
        }
    }
}

#[cfg(test)]
mod months_tests {
    use super::*;

    #[test]
    fn into() {
        let mut num: u32 = Month::January().into();
        assert_eq!(num, 0);
        num = Month::February().into();
        assert_eq!(num, 1);
        num = Month::March().into();
        assert_eq!(num, 2);
        num = Month::April().into();
        assert_eq!(num, 3);
        num = Month::May().into();
        assert_eq!(num, 4);
        num = Month::June().into();
        assert_eq!(num, 5);
        num = Month::July().into();
        assert_eq!(num, 6);
        num = Month::August().into();
        assert_eq!(num, 7);
        num = Month::September().into();
        assert_eq!(num, 8);
        num = Month::October().into();
        assert_eq!(num, 9);
        num = Month::November().into();
        assert_eq!(num, 10);
        num = Month::December().into();
        assert_eq!(num, 11);
    }

    #[test]
    fn valid_try() {
        assert_eq!(Month::January(), Month::try_from(0).unwrap());
        assert_eq!(Month::February(), Month::try_from(1).unwrap());
        assert_eq!(Month::March(), Month::try_from(2).unwrap());
        assert_eq!(Month::April(), Month::try_from(3).unwrap());
        assert_eq!(Month::May(), Month::try_from(4).unwrap());
        assert_eq!(Month::June(), Month::try_from(5).unwrap());
        assert_eq!(Month::July(), Month::try_from(6).unwrap());
        assert_eq!(Month::August(), Month::try_from(7).unwrap());
        assert_eq!(Month::September(), Month::try_from(8).unwrap());
        assert_eq!(Month::October(), Month::try_from(9).unwrap());
        assert_eq!(Month::November(), Month::try_from(10).unwrap());
        assert_eq!(Month::December(), Month::try_from(11).unwrap());
    }

    #[test]
    fn invalid_try() {
        assert!(Month::try_from(12).is_err());
        assert!(Month::try_from(128).is_err());
        assert!(Month::try_from(185).is_err());
        assert!(Month::try_from(94).is_err());
        assert!(Month::try_from(50).is_err());
    }
}