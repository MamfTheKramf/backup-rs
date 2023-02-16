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

impl From<u32> for Month {
    /// Converts [u32] cyclicly into [Month]. 
    ///
    /// # Example
    /// ```
    /// use config::interval::months::*;
    /// 
    /// assert_eq!(Month::from(0), Month::January());
    /// assert_eq!(Month::from(7), Month::August());
    /// assert_eq!(Month::from(17), Month::June());
    /// ```
    fn from(value: u32) -> Self {
        Month::new((value % N as u32) as u8)
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
    fn normal_from() {
        assert_eq!(Month::January(), Month::from(0));
        assert_eq!(Month::February(), Month::from(1));
        assert_eq!(Month::March(), Month::from(2));
        assert_eq!(Month::April(), Month::from(3));
        assert_eq!(Month::May(), Month::from(4));
        assert_eq!(Month::June(), Month::from(5));
        assert_eq!(Month::July(), Month::from(6));
        assert_eq!(Month::August(), Month::from(7));
        assert_eq!(Month::September(), Month::from(8));
        assert_eq!(Month::October(), Month::from(9));
        assert_eq!(Month::November(), Month::from(10));
        assert_eq!(Month::December(), Month::from(11));
    }

    #[test]
    fn overflow_try() {
        assert_eq!(Month::January(), Month::from(12));
        assert_eq!(Month::February(), Month::from(25));
        assert_eq!(Month::July(), Month::from(18));
        assert_eq!(Month::September(), Month::from(20));
    }
}