

/// Represents result of a datetime match.
/// 
/// Can also be casted into a bool
#[derive(Debug, PartialEq, Eq)]
pub enum DateTimeMatch {
    Ok,
    TimeNotMatched,
    DateNotMatched,
}

impl Into<bool> for DateTimeMatch {
    fn into(self) -> bool {
        self == DateTimeMatch::Ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversion_test() {
        assert_eq!(<DateTimeMatch as Into<bool>>::into(DateTimeMatch::Ok), true);
        assert_eq!(<DateTimeMatch as Into<bool>>::into(DateTimeMatch::DateNotMatched), false);
        assert_eq!(<DateTimeMatch as Into<bool>>::into(DateTimeMatch::TimeNotMatched), false);
    }
}