//! Contains [Specifier] Struct that can be used to specify certain numbers from a range.
//! For example, can be used to always take the first or the last or every `n`-th with an offset.

/// Represents a specifier.
/// Has a range of possible values and a specifier rule that filters out all none-specified values in that range.
/// The represented range is inclusive. I.e., `min` and `max` can be matched by the specifier-rule as well.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Specifier<T>
where
    T: Into<u32> + From<u32> + Copy,
{
    min: T,
    max: T,
    kind: SpecifierKind,
}

/// Represents the kind of specifier to mathc elements from a range.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SpecifierKind {
    /// Maches no element from range.
    ///
    /// # Example
    /// ```
    /// use config::interval::*;
    ///
    /// let spec = Specifier::new(5 as u32, 15 as u32, SpecifierKind::None);
    /// for x in 5..=15 {
    ///     assert!(!spec.matches(x));
    ///     assert_eq!(spec.cyclic_next(x), None);
    /// }
    /// ```
    None,

    /// Matches all elements from range.
    ///
    /// # Example
    /// ```
    /// use config::interval::*;
    ///
    /// let spec = Specifier::new(11 as u32, 27 as u32, SpecifierKind::All);
    /// for x in 11..=27 {
    ///     assert!(spec.matches(x));
    ///     assert!(spec.cyclic_next(x).is_some());
    /// }
    /// ```
    All,

    /// Only the first element of range. I.e., `min`
    ///
    /// # Example
    /// ```
    /// use config::interval::*;
    ///
    /// let spec = Specifier::new(128 as u32, 256 as u32, SpecifierKind::First);
    /// for x in 128..=256 {
    ///     assert_eq!(spec.matches(x), x == 128);
    ///     assert_eq!(spec.cyclic_next(x), Some(128));
    /// }
    /// ```
    First,

    /// Only the last element of range. I.e., `max`
    ///
    /// # Example
    /// ```
    /// use config::interval::*;
    ///
    /// let spec = Specifier::new(0 as u32, 19 as u32, SpecifierKind::Last);
    /// for x in 0..=19 {
    ///     assert_eq!(spec.matches(x), x == 19);
    ///     assert_eq!(spec.cyclic_next(x), Some(19));
    /// }
    /// ```
    Last,

    /// Only the `n`-th element of range. I.e., `min + n` (if smaller than `max`).
    ///
    /// # Example
    /// ```
    /// use config::interval::*;
    ///
    /// let spec = Specifier::new(0 as u32, 10 as u32, SpecifierKind::Nth(5));
    /// for x in 0..=10 {
    ///     assert_eq!(spec.matches(x), x == 0 + 5);
    ///     assert_eq!(spec.cyclic_next(x), Some(5));
    /// }
    /// ```
    Nth(u32),

    /// Only the `n`-th last element of range. I.e., `max - n` (if larger than `min`)
    ///
    /// # Example
    /// ```
    /// use config::interval::*;
    ///
    /// let spec = Specifier::new(100 as u32, 1000 as u32, SpecifierKind::BackNth(200));
    /// for x in 100..=1000 {
    ///     assert_eq!(spec.matches(x), x == 1000 - 200);
    ///     assert_eq!(spec.cyclic_next(x), Some(800));
    /// }
    /// ```
    BackNth(u32),

    /// Only the elements with `min + i` for `i` in `vec`
    ///
    /// # Example
    /// ```
    /// use config::interval::*;
    ///
    /// let spec = Specifier::new(20 as u32, 50 as u32, SpecifierKind::ExplicitNths(vec![0, 10, 15, 30]));
    /// assert!(spec.matches(20));
    /// assert!(spec.matches(35));
    /// assert!(!spec.matches(24));
    /// assert!(!spec.matches(32));
    /// 
    /// assert_eq!(spec.cyclic_next(20), Some(30));
    /// assert_eq!(spec.cyclic_next(30), Some(35));
    /// assert_eq!(spec.cyclic_next(35), Some(50));
    /// assert_eq!(spec.cyclic_next(50), Some(20));
    /// ```
    ExplicitNths(Vec<u32>),

    /// Every `n`-th element with an offset. I.e., `min + i * n + off` for each `i`
    ///
    /// # Example
    /// ```
    /// use config::interval::*;
    ///
    /// let spec = Specifier::new(0 as u32, 6 as u32, SpecifierKind::EveryNth(2, 1));
    /// assert!(!spec.matches(0));
    /// assert!(spec.matches(1));
    /// assert!(!spec.matches(2));
    /// assert!(spec.matches(5));
    /// assert!(!spec.matches(6));
    /// 
    /// assert_eq!(spec.cyclic_next(0), Some(1));
    /// assert_eq!(spec.cyclic_next(1), Some(3));
    /// assert_eq!(spec.cyclic_next(3), Some(5));
    /// assert_eq!(spec.cyclic_next(5), Some(1));
    /// ```
    EveryNth(u32, u32),

    /// Only elements that are explicitely specified in `vec`
    ///
    /// # Example
    /// ```
    /// use config::interval::*;
    ///
    /// let spec = Specifier::new(128 as u32, 1024 as u32, SpecifierKind::ExplicitList(vec![128, 256, 512, 1024]));
    /// assert!(spec.matches(256));
    /// assert!(spec.matches(1024));
    /// assert!(!spec.matches(140));
    /// assert!(!spec.matches(587));
    /// assert!(!spec.matches(1000));
    /// 
    /// assert_eq!(spec.cyclic_next(128), Some(256));
    /// assert_eq!(spec.cyclic_next(256), Some(512));
    /// assert_eq!(spec.cyclic_next(512), Some(1024));
    /// assert_eq!(spec.cyclic_next(1024), Some(128));
    /// ```
    ExplicitList(Vec<u32>),
}

impl<T> Specifier<T>
where
    T: Into<u32> + From<u32> + Copy,
{
    /// Creates new [Specifier], but makes sure that `min` is actually less thab `max`.
    ///
    /// # Note
    /// If the `n` for [SpecifierKind::Nth] and [SpecifierKind::BackNth] exceeds the range, no element will be matched.
    ///
    /// Also makes sure, that for [SpecifierKind::ExplicitNths] and [SpecifierKind::ExplicitList], the elements don't overflow the range.
    /// If they do, overflowing elements are removed from the list. Those 2 variants will also have their values sorted and dedupped.
    ///
    /// If the `offset` for [SpecifierKind::EveryNth] exceeds that range, no element from the range will be matched.
    ///
    /// # Example
    /// ```
    /// use config::interval::*;
    ///
    /// let a = 0 as u32;
    /// let b = 10 as u32;
    /// let spec = Specifier::new(a, b, SpecifierKind::All);
    /// assert_eq!(spec.min(), a);
    /// assert_eq!(spec.max(), b);
    ///
    /// let swapped_spec = Specifier::new(b, a, SpecifierKind::All);
    /// assert_eq!(swapped_spec.min(), a);
    /// assert_eq!(swapped_spec.max(), b);
    /// ```
    pub fn new(mut min: T, mut max: T, kind: SpecifierKind) -> Specifier<T> {
        if min.into() > max.into() {
            std::mem::swap(&mut min, &mut max);
        }

        // filter out unwanted values from List-type specifier kinds
        let kind = match kind {
            SpecifierKind::ExplicitList(values) => SpecifierKind::ExplicitList({
                let mut values: Vec<u32> = values
                    .into_iter()
                    .filter(|val| val >= &min.into() && val <= &max.into())
                    .collect();
                values.sort_unstable();
                values.dedup();
                values
            }),
            SpecifierKind::ExplicitNths(indices) => SpecifierKind::ExplicitNths({
                let max_index = max.into() - min.into();
                let mut indices: Vec<u32> = indices
                    .into_iter()
                    .filter(|index| index <= &max_index)
                    .collect();
                indices.sort_unstable();
                indices.dedup();
                indices
            }),
            other => other,
        };

        Specifier { min, max, kind }
    }

    pub fn min(&self) -> T {
        self.min
    }

    pub fn max(&self) -> T {
        self.max
    }

    pub fn range_len(&self) -> u32 {
        self.max.into() - self.min.into() + 1
    }

    pub fn kind(&self) -> &SpecifierKind {
        &self.kind
    }

    /// Returns if `x` is contained in the range.
    /// Ignores the specifier-rule.
    ///
    /// # Example
    /// ```
    /// use config::interval::*;
    ///
    /// let spec = Specifier::new(20 as u32, 100 as u32, SpecifierKind::None);
    /// assert!(spec.is_in_range(20));
    /// assert!(spec.is_in_range(60));
    /// assert!(spec.is_in_range(100));
    /// assert!(!spec.is_in_range(10));
    /// assert!(!spec.is_in_range(106));
    /// ```
    pub fn is_in_range(&self, x: T) -> bool {
        self.min.into() <= x.into() && self.max.into() >= x.into()
    }

    /// Returns the first (smallest) element from the range that is matched, ot none if there is no match
    /// 
    /// # Example
    /// ```
    /// use config::interval::*;
    /// let spec = Specifier::new(10u32, 20u32, SpecifierKind::EveryNth(5, 5));
    /// assert_eq!(spec.first_match().unwrap(), 15);
    /// 
    /// let spec = Specifier::new(10u32, 20u32, SpecifierKind::EveryNth(100, 400));
    /// assert!(spec.first_match().is_none());
    /// ```
    pub fn first_match(&self) -> Option<T> {
        self.cyclic_next(self.max)
    }

    /// Returns if `x` is matches by the given specifier-rule
    ///
    /// # Example
    /// ```
    /// use config::interval::*;
    ///
    /// let spec = Specifier::new(10u32, 20u32, SpecifierKind::None);
    /// assert!(!spec.matches(11));
    /// assert!(!spec.matches(15));
    /// assert!(!spec.matches(20));
    ///
    /// let spec = Specifier::new(10u32, 20u32, SpecifierKind::All);
    /// assert!(spec.matches(11));
    /// assert!(spec.matches(15));
    /// assert!(spec.matches(20));
    ///
    /// let spec = Specifier::new(10u32, 20u32, SpecifierKind::First);
    /// assert!(spec.matches(10));
    /// assert!(!spec.matches(15));
    /// assert!(!spec.matches(20));
    ///
    /// let spec = Specifier::new(10u32, 20u32, SpecifierKind::Last);
    /// assert!(!spec.matches(11));
    /// assert!(!spec.matches(15));
    /// assert!(spec.matches(20));
    ///
    /// let spec = Specifier::new(10u32, 20u32, SpecifierKind::Nth(5));
    /// assert!(!spec.matches(11));
    /// assert!(spec.matches(15));
    /// assert!(!spec.matches(20));
    ///
    /// let spec = Specifier::new(10u32, 20u32, SpecifierKind::BackNth(7));
    /// assert!(!spec.matches(11));
    /// assert!(spec.matches(13));
    /// assert!(!spec.matches(20));
    ///
    /// let spec = Specifier::new(10u32, 20u32, SpecifierKind::ExplicitNths(vec![0, 5, 7]));
    /// assert!(spec.matches(10));
    /// assert!(spec.matches(15));
    /// assert!(spec.matches(17));
    /// assert!(!spec.matches(11));
    /// assert!(!spec.matches(14));
    /// assert!(!spec.matches(20));
    ///
    /// let spec = Specifier::new(10u32, 20u32, SpecifierKind::EveryNth(2, 3));
    /// assert!(!spec.matches(11));
    /// assert!(spec.matches(13));
    /// assert!(spec.matches(15));
    /// assert!(spec.matches(17));
    /// assert!(spec.matches(19));
    /// assert!(!spec.matches(10));
    /// assert!(!spec.matches(14));
    /// assert!(!spec.matches(18));
    ///
    /// let spec = Specifier::new(10u32, 20u32, SpecifierKind::ExplicitList(vec![10, 17, 20]));
    /// assert!(spec.matches(10));
    /// assert!(spec.matches(17));
    /// assert!(spec.matches(20));
    /// assert!(!spec.matches(12));
    /// assert!(!spec.matches(15));
    /// ```
    pub fn matches(&self, x: T) -> bool {
        if !self.is_in_range(x) {
            return false;
        }

        match &self.kind {
            SpecifierKind::None => false,
            SpecifierKind::All => true,
            SpecifierKind::First => x.into() == self.min.into(),
            SpecifierKind::Last => x.into() == self.max.into(),
            SpecifierKind::Nth(n) => self.min.into() + n == x.into(),
            SpecifierKind::BackNth(n) => self.max.into() - n == x.into(),
            SpecifierKind::ExplicitNths(indices) => {
                indices.iter().any(|n| self.min.into() + n == x.into())
            }
            SpecifierKind::EveryNth(n, offset) => {
                // if offset is out of range or larger than x -> we can't match
                let min_offset = self.min.into() + offset;
                if min_offset > self.max.into() || min_offset > x.into() {
                    return false;
                }
                // if n == 0 we can only match, when x == min + offset
                if n == &0 {
                    return x.into() == min_offset;
                }
                // else x ?= min + offset * i * n for integer i
                // ==> n | (x - (min + offset))
                (x.into() - min_offset).rem_euclid(*n) == 0
            }
            SpecifierKind::ExplicitList(values) => values.contains(&x.into()),
        }
    }

    /// Returns the next specified element from the range, if one exists.
    ///
    /// # Example
    /// ```
    /// use config::interval::*;
    ///
    /// let a = 0 as u32;
    /// let b = 10 as u32;
    ///
    /// let none_spec = Specifier::new(a, b, SpecifierKind::None);
    /// assert_eq!(none_spec.cyclic_next(5 as u32), None);
    ///
    /// let all_spec = Specifier::new(a, b, SpecifierKind::All);
    /// assert_eq!(all_spec.cyclic_next(10 as u32), Some(0));
    ///
    /// let even_spec = Specifier::new(a, b, SpecifierKind::EveryNth(2, 0));
    /// assert_eq!(even_spec.cyclic_next(4 as u32), Some(6));
    /// ```
    pub fn cyclic_next(&self, x: T) -> Option<T> {
        if !self.is_in_range(x) {
            return None;
        }

        match &self.kind {
            SpecifierKind::None => None,
            SpecifierKind::All => Some(T::from(
                self.min.into() + (x.into() - self.min.into() + 1) % self.range_len(),
            )),
            SpecifierKind::First => Some(self.min),
            SpecifierKind::Last => Some(self.max),
            SpecifierKind::Nth(n) => {
                let val = self.min.into() + n;
                if val <= self.max.into() {
                    Some(T::from(val))
                } else {
                    None
                }
            }
            SpecifierKind::BackNth(n) => {
                let val = self.max.into() - n;
                if val >= self.min.into() {
                    Some(T::from(val))
                } else {
                    None
                }
            }
            SpecifierKind::ExplicitNths(indices) => {
                // if there are no indices -> return None
                if indices.is_empty() {
                    return None;
                }
                // find the first n for which we are larger than x
                let res = indices.iter().find_map(|n| {
                    let val: u32 = self.min.into() + n;
                    if val > x.into() {
                        Some((n, val))
                    } else {
                        None
                    }
                });
                // if we found one -> return it
                // if it's a None -> use the first N
                Some(T::from(
                    res.unwrap_or_else(|| (&0, self.min.into() + indices[0])).1,
                ))
            }
            SpecifierKind::EveryNth(n, offset) => {
                // if n == 0 return self.min + offset
                // else exists i from R such that self.min + offset + i * n = x
                // ==> find i -> if i < 0 return self.min + offset
                // ==> add .5 to i and round -> we alway end at the next larger int even if i is a perfect int
                // ==> return self.min + offset + i_round_up * n if <= self.max; else return self.min + offset
                let min_offset = self.min.into() + offset;
                if min_offset > self.max.into() {
                    return None;
                }
                if n == &0 {
                    return Some(T::from(min_offset));
                }
                let i = (x.into() as f64 - min_offset as f64) / (*n as f64);
                if i < 0.0 {
                    return Some(T::from(min_offset));
                }
                let i_round_up = (i + 0.5).round() as u32;
                let candidate = min_offset + i_round_up * n;
                let val = if candidate <= self.max.into() {
                    candidate
                } else {
                    min_offset
                };
                Some(T::from(val))
            }
            SpecifierKind::ExplicitList(values) => {
                // if there are no elements -> return None
                if values.is_empty() {
                    return None;
                }
                // find the index of the first value that is larger than x
                let target_index = match values.binary_search(&x.into()) {
                    Ok(index) => index + 1,
                    Err(index) => index,
                };
                let val = if values.len() > target_index {
                    values[target_index]
                } else {
                    values[0]
                };
                Some(T::from(val))
            }
        }
    }
}

#[cfg(test)]
mod specifier_tests {
    use super::*;

    mod new_tests {
        use super::*;

        #[test]
        fn sort_list_kinds() {
            let indices = vec![0, 10, 5, 15, 7, 80, 1];
            let mut sorted = indices.clone();
            sorted.sort_unstable();
            let spec = Specifier::new(0 as u32, 100 as u32, SpecifierKind::ExplicitNths(indices));
            assert_eq!(spec.kind(), &SpecifierKind::ExplicitNths(sorted));

            let values = vec![109, 103, 87, 150, 100];
            let mut sorted = values.clone();
            sorted.sort_unstable();
            let spec = Specifier::new(50 as u32, 200 as u32, SpecifierKind::ExplicitList(values));
            assert_eq!(spec.kind(), &SpecifierKind::ExplicitList(sorted));
        }

        #[test]
        fn dedup_list_kinds() {
            let indices = vec![8, 20, 7, 8, 1, 20, 1, 17];
            let mut dedupped = indices.clone();
            dedupped.sort_unstable();
            dedupped.dedup();
            let spec = Specifier::new(20 as u32, 5000 as u32, SpecifierKind::ExplicitNths(indices));
            assert_eq!(spec.kind(), &SpecifierKind::ExplicitNths(dedupped));

            let values = vec![55, 21, 137, 21, 99, 137, 200];
            let mut dedupped = values.clone();
            dedupped.sort_unstable();
            dedupped.dedup();
            let spec = Specifier::new(20 as u32, 1234 as u32, SpecifierKind::ExplicitList(values));
            assert_eq!(spec.kind(), &SpecifierKind::ExplicitList(dedupped));
        }

        #[test]
        fn filter_out_of_range() {
            let min = 100;
            let max = 133;
            let indices = vec![66, 0, 12, 34, 180, 33, 0, 15];
            let mut dedupped: Vec<u32> = indices
                .clone()
                .into_iter()
                .filter(|index| index <= &(max - min))
                .collect();
            dedupped.sort_unstable();
            dedupped.dedup();
            println!("{:?}", dedupped);
            let spec = Specifier::new(min, max, SpecifierKind::ExplicitNths(indices));
            assert_eq!(spec.kind(), &SpecifierKind::ExplicitNths(dedupped));

            let min = 320;
            let max = 490;
            let values = vec![12, 330, 444, 325, 3000, 490, 6, 400, 570, 320];
            let mut dedupped: Vec<u32> = values
                .clone()
                .into_iter()
                .filter(|value| value <= &max && value >= &min)
                .collect();
            dedupped.sort_unstable();
            dedupped.dedup();
            println!("{:?}", dedupped);
            let spec = Specifier::new(min, max, SpecifierKind::ExplicitList(values));
            assert_eq!(spec.kind(), &SpecifierKind::ExplicitList(dedupped));
        }
    }

    mod first_match_tests {
        use super::*;

        #[test]
        fn no_matches() {
            let spec = Specifier::new(0u32, 100u32, SpecifierKind::None);
            assert!(spec.first_match().is_none());

            let spec = Specifier::new(55u32, 77u32, SpecifierKind::ExplicitList(vec![0, 7, 35]));
            assert!(spec.first_match().is_none());
        }

        #[test]
        fn first_match_is_min() {
            let spec = Specifier::new(0u32, 10u32, SpecifierKind::All);
            assert_eq!(spec.first_match().unwrap(), 0);

            let spec = Specifier::new(4328u32, 9999u32, SpecifierKind::EveryNth(2, 0));
            assert_eq!(spec.first_match().unwrap(), 4328);
        }

        #[test]
        fn first_match_is_max() {
            let spec = Specifier::new(4328u32, 9999u32, SpecifierKind::BackNth(0));
            assert_eq!(spec.first_match().unwrap(), 9999);

            let spec = Specifier::new(12u32, 7000u32, SpecifierKind::ExplicitList(vec![7000]));
            assert_eq!(spec.first_match().unwrap(), 7000);
        }

        #[test]
        fn valid_first_match() {
            let spec = Specifier::new(7u32, 100u32, SpecifierKind::ExplicitList(vec![19, 28, 99]));
            assert_eq!(spec.first_match().unwrap(), 19);
        }
    }

    mod matches_tests {
        use super::*;

        #[test]
        fn out_of_range_test() {
            let spec = Specifier::new(0 as u32, 100 as u32, SpecifierKind::None);
            assert_eq!(spec.matches(264), false);
            assert_eq!(spec.matches(400), false);
            assert_eq!(spec.matches(101), false);

            let spec = Specifier::new(25 as u32, 50 as u32, SpecifierKind::None);
            assert_eq!(spec.matches(0), false);
            assert_eq!(spec.matches(19), false);
            assert_eq!(spec.matches(24), false);
            assert_eq!(spec.matches(51), false);
        }

        #[test]
        fn none_test() {
            let a = 17u32;
            let b = 77u32;
            let spec = Specifier::new(a, b, SpecifierKind::None);
            for x in a..=b {
                assert!(!spec.matches(x));
            }
        }

        #[test]
        fn all_test() {
            let a = 17u32;
            let b = 77u32;
            let spec = Specifier::new(a, b, SpecifierKind::All);
            for x in a..=b {
                assert!(spec.matches(x));
            }
        }

        #[test]
        fn first_test() {
            let a = 17u32;
            let b = 77u32;
            let spec = Specifier::new(a, b, SpecifierKind::First);
            assert!(spec.matches(a));
            for x in (a+1)..=b {
                assert!(!spec.matches(x));
            }
        }

        #[test]
        fn last_test() {
            let a = 17u32;
            let b = 77u32;
            let spec = Specifier::new(a, b, SpecifierKind::Last);
            assert!(spec.matches(b));
            for x in a..b {
                assert!(!spec.matches(x));
            }
        }

        #[test]
        fn nth_test() {
            let a = 17u32;
            let b = 77u32;
            let n = 8u32;
            let spec = Specifier::new(a, b, SpecifierKind::Nth(n));
            for i in 0..=(b - a) {
                if i == n {
                    assert!(spec.matches(a + i));
                } else {
                    assert!(!spec.matches(a + i));
                }
            }
        }

        #[test]
        fn back_nth_test() {
            let a = 17u32;
            let b = 77u32;
            let n = 8u32;
            let spec = Specifier::new(a, b, SpecifierKind::BackNth(n));
            for i in 0..=(b - a) {
                if i == n {
                    assert!(spec.matches(b - i));
                } else {
                    assert!(!spec.matches(b - i));
                }
            }
        }

        #[test]
        fn explicit_nth_test() {
            let a = 17u32;
            let b = 77u32;
            let indices = vec![0, 5, 28, 1000];
            let spec = Specifier::new(a, b, SpecifierKind::ExplicitNths(indices.clone()));
            for i in 0..=(b - a) {
                if indices.contains(&i) {
                    assert!(spec.matches(a + i));
                } else {
                    assert!(!spec.matches(a + i));
                }
            }
            assert!(!spec.matches(a + 1000));
        }

        #[test]
        fn explicit_nths_empty_test() {
            let a = 17u32;
            let b = 77u32;
            let spec = Specifier::new(a, b, SpecifierKind::ExplicitNths(vec![]));
            for x in a..=b {
                assert!(!spec.matches(x));
            }
        }

        #[test]
        fn explicit_list_empty() {
            let a = 17u32;
            let b = 77u32;
            let spec = Specifier::new(a, b, SpecifierKind::ExplicitList(vec![]));
            for x in a..=b {
                assert!(!spec.matches(x));
            }
        }

        #[test]
        fn explicit_list_test() {
            let a = 17u32;
            let b = 77u32;
            let values = vec![0, 5, 20, 28, 40, 55, 1000];
            let spec = Specifier::new(a, b, SpecifierKind::ExplicitList(values.clone()));
            for x in a..=b {
                if values.contains(&x) {
                    assert!(spec.matches(x));
                } else {
                    assert!(!spec.matches(x));
                }
            }
            assert!(!spec.matches(0));
            assert!(!spec.matches(5));
            assert!(!spec.matches(1000));
        }

        #[test]
        fn every_0th() {
            let a = 17u32;
            let b = 77u32;
            let spec = Specifier::new(a, b, SpecifierKind::EveryNth(0, 3));
            for x in a..=b {
                assert_eq!(spec.matches(x), x == 20);
            }
        }

        #[test]
        fn every_nth_offset_too_big() {
            let a = 17u32;
            let b = 77u32;
            let spec = Specifier::new(a, b, SpecifierKind::EveryNth(0, 1000));
            for x in a..=b {
                assert!(!spec.matches(x));
            }
        }

        #[test]
        fn every_nth() {
            let spec = Specifier::new(10u32, 20u32, SpecifierKind::EveryNth(3, 2));
            assert!(spec.matches(12));
            assert!(spec.matches(15));
            assert!(spec.matches(18));
            assert!(!spec.matches(10));
            assert!(!spec.matches(11));
            assert!(!spec.matches(13));
            assert!(!spec.matches(14));
            assert!(!spec.matches(16));
            assert!(!spec.matches(17));
            assert!(!spec.matches(19));
            assert!(!spec.matches(20));
        }
    }

    mod cyclic_next_tests {
        use super::*;

        #[test]
        fn out_of_range_test() {
            let spec = Specifier::new(0 as u32, 100 as u32, SpecifierKind::None);
            assert_eq!(spec.cyclic_next(264), None);
            assert_eq!(spec.cyclic_next(400), None);
            assert_eq!(spec.cyclic_next(101), None);

            let spec = Specifier::new(25 as u32, 50 as u32, SpecifierKind::None);
            assert_eq!(spec.cyclic_next(0), None);
            assert_eq!(spec.cyclic_next(19), None);
            assert_eq!(spec.cyclic_next(24), None);
            assert_eq!(spec.cyclic_next(51), None);
        }

        #[test]
        fn none_test() {
            let spec = Specifier::new(0 as u32, 100 as u32, SpecifierKind::None);
            assert_eq!(spec.cyclic_next(19), None);
            assert_eq!(spec.cyclic_next(5), None);
            assert_eq!(spec.cyclic_next(200), None);
            assert_eq!(spec.cyclic_next(99), None);

            let spec = Specifier::new(200 as u32, 500 as u32, SpecifierKind::None);
            assert_eq!(spec.cyclic_next(385), None);
            assert_eq!(spec.cyclic_next(500), None);
            assert_eq!(spec.cyclic_next(18), None);
            assert_eq!(spec.cyclic_next(45), None);
        }

        #[test]
        fn all_test() {
            let spec = Specifier::new(0 as u32, 10 as u32, SpecifierKind::All);
            for i in 0..=10 {
                assert_eq!(spec.cyclic_next(i), Some((i + 1) % 11));
            }

            let spec = Specifier::new(10 as u32, 100 as u32, SpecifierKind::All);
            for i in 10..100 {
                assert_eq!(spec.cyclic_next(i), Some(i + 1))
            }
            assert_eq!(spec.cyclic_next(100), Some(10));
        }

        #[test]
        fn first_test() {
            let spec = Specifier::new(0 as u32, 10 as u32, SpecifierKind::First);
            assert_eq!(spec.cyclic_next(0), Some(0));
            assert_eq!(spec.cyclic_next(4), Some(0));
            assert_eq!(spec.cyclic_next(7), Some(0));
            assert_eq!(spec.cyclic_next(10), Some(0));

            let spec = Specifier::new(17 as u32, 4325 as u32, SpecifierKind::First);
            assert_eq!(spec.cyclic_next(453), Some(17));
            assert_eq!(spec.cyclic_next(17), Some(17));
            assert_eq!(spec.cyclic_next(4000), Some(17));
            assert_eq!(spec.cyclic_next(1234), Some(17));
        }

        #[test]
        fn last_test() {
            let spec = Specifier::new(0 as u32, 10 as u32, SpecifierKind::Last);
            assert_eq!(spec.cyclic_next(0), Some(10));
            assert_eq!(spec.cyclic_next(4), Some(10));
            assert_eq!(spec.cyclic_next(7), Some(10));
            assert_eq!(spec.cyclic_next(10), Some(10));

            let spec = Specifier::new(17 as u32, 4325 as u32, SpecifierKind::Last);
            assert_eq!(spec.cyclic_next(453), Some(4325));
            assert_eq!(spec.cyclic_next(17), Some(4325));
            assert_eq!(spec.cyclic_next(4000), Some(4325));
            assert_eq!(spec.cyclic_next(1234), Some(4325));
        }

        #[test]
        fn nth_test() {
            let spec = Specifier::new(0 as u32, 10 as u32, SpecifierKind::Nth(4));
            assert_eq!(spec.cyclic_next(0), Some(4));
            assert_eq!(spec.cyclic_next(4), Some(4));
            assert_eq!(spec.cyclic_next(7), Some(4));
            assert_eq!(spec.cyclic_next(10), Some(4));

            let spec = Specifier::new(12 as u32, 50 as u32, SpecifierKind::Nth(10));
            assert_eq!(spec.cyclic_next(12), Some(22));
            assert_eq!(spec.cyclic_next(15), Some(22));
            assert_eq!(spec.cyclic_next(36), Some(22));
            assert_eq!(spec.cyclic_next(49), Some(22));
        }

        #[test]
        fn back_nth_test() {
            let spec = Specifier::new(0 as u32, 10 as u32, SpecifierKind::BackNth(4));
            assert_eq!(spec.cyclic_next(0), Some(6));
            assert_eq!(spec.cyclic_next(4), Some(6));
            assert_eq!(spec.cyclic_next(7), Some(6));
            assert_eq!(spec.cyclic_next(10), Some(6));

            let spec = Specifier::new(12 as u32, 50 as u32, SpecifierKind::BackNth(10));
            assert_eq!(spec.cyclic_next(12), Some(40));
            assert_eq!(spec.cyclic_next(15), Some(40));
            assert_eq!(spec.cyclic_next(36), Some(40));
            assert_eq!(spec.cyclic_next(49), Some(40));
        }

        #[test]
        fn explicit_nths() {
            let spec = Specifier::new(
                0 as u32,
                10 as u32,
                SpecifierKind::ExplicitNths(vec![1, 3, 7, 10]),
            );
            assert_eq!(spec.cyclic_next(0), Some(1));
            assert_eq!(spec.cyclic_next(1), Some(3));
            assert_eq!(spec.cyclic_next(3), Some(7));
            assert_eq!(spec.cyclic_next(7), Some(10));
            assert_eq!(spec.cyclic_next(10), Some(1)); // test a simple overflow here

            let spec = Specifier::new(
                20 as u32,
                45 as u32,
                SpecifierKind::ExplicitNths(vec![0, 5, 10, 11, 15, 20]),
            );
            assert_eq!(spec.cyclic_next(20), Some(25));
            assert_eq!(spec.cyclic_next(21), Some(25));
            assert_eq!(spec.cyclic_next(24), Some(25));
            assert_eq!(spec.cyclic_next(25), Some(30));
            assert_eq!(spec.cyclic_next(30), Some(31));
            assert_eq!(spec.cyclic_next(31), Some(35));
            assert_eq!(spec.cyclic_next(35), Some(40));
            assert_eq!(spec.cyclic_next(38), Some(40));
            assert_eq!(spec.cyclic_next(40), Some(20));
            assert_eq!(spec.cyclic_next(43), Some(20));
            assert_eq!(spec.cyclic_next(45), Some(20));
        }

        #[test]
        fn explicit_nths_empty() {
            let spec = Specifier::new(14 as u32, 200 as u32, SpecifierKind::ExplicitNths(vec![]));
            assert_eq!(spec.cyclic_next(14), None);
            assert_eq!(spec.cyclic_next(34), None);
            assert_eq!(spec.cyclic_next(100), None);
            assert_eq!(spec.cyclic_next(164), None);
            assert_eq!(spec.cyclic_next(200), None);
        }

        #[test]
        fn explicit_list_empty() {
            let spec = Specifier::new(37 as u32, 128 as u32, SpecifierKind::ExplicitList(vec![]));
            assert_eq!(spec.cyclic_next(37), None);
            assert_eq!(spec.cyclic_next(55), None);
            assert_eq!(spec.cyclic_next(62), None);
            assert_eq!(spec.cyclic_next(100), None);
            assert_eq!(spec.cyclic_next(128), None);
        }

        #[test]
        fn explicit_list() {
            let spec = Specifier::new(
                1000 as u32,
                1500 as u32,
                SpecifierKind::ExplicitList(vec![1000, 1075, 1080, 1100, 1360, 1400, 1450]),
            );
            assert_eq!(spec.cyclic_next(1000), Some(1075));
            assert_eq!(spec.cyclic_next(1020), Some(1075));
            assert_eq!(spec.cyclic_next(1077), Some(1080));
            assert_eq!(spec.cyclic_next(1090), Some(1100));
            assert_eq!(spec.cyclic_next(1100), Some(1360));
            assert_eq!(spec.cyclic_next(1240), Some(1360));
            assert_eq!(spec.cyclic_next(1399), Some(1400));
            assert_eq!(spec.cyclic_next(1420), Some(1450));
            assert_eq!(spec.cyclic_next(1450), Some(1000));
            assert_eq!(spec.cyclic_next(1470), Some(1000));
            assert_eq!(spec.cyclic_next(1500), Some(1000));
        }

        #[test]
        fn every_0th() {
            let spec = Specifier::new(
                25 as u32,
                1200 as u32,
                SpecifierKind::EveryNth(0 as u32, 0 as u32),
            );
            assert_eq!(spec.cyclic_next(25), Some(25));
            assert_eq!(spec.cyclic_next(100), Some(25));
            assert_eq!(spec.cyclic_next(230), Some(25));
            assert_eq!(spec.cyclic_next(1000), Some(25));
            assert_eq!(spec.cyclic_next(1200), Some(25));

            let spec = Specifier::new(
                1000 as u32,
                1100 as u32,
                SpecifierKind::EveryNth(0 as u32, 37 as u32),
            );
            assert_eq!(spec.cyclic_next(1000), Some(1037));
            assert_eq!(spec.cyclic_next(1020), Some(1037));
            assert_eq!(spec.cyclic_next(1037), Some(1037));
            assert_eq!(spec.cyclic_next(1040), Some(1037));
            assert_eq!(spec.cyclic_next(1100), Some(1037));
        }

        #[test]
        fn every_nth_offet_too_big() {
            let spec = Specifier::new(0 as u32, 10 as u32, SpecifierKind::EveryNth(1, 20));
            assert_eq!(spec.cyclic_next(0), None);
            assert_eq!(spec.cyclic_next(1), None);
            assert_eq!(spec.cyclic_next(2), None);
            assert_eq!(spec.cyclic_next(3), None);
            assert_eq!(spec.cyclic_next(7), None);
            assert_eq!(spec.cyclic_next(8), None);
            assert_eq!(spec.cyclic_next(9), None);
            assert_eq!(spec.cyclic_next(10), None);
        }

        #[test]
        fn exery_nth() {
            let spec = Specifier::new(0 as u32, 6 as u32, SpecifierKind::EveryNth(2, 0));
            assert_eq!(spec.cyclic_next(0), Some(2));
            assert_eq!(spec.cyclic_next(1), Some(2));
            assert_eq!(spec.cyclic_next(2), Some(4));
            assert_eq!(spec.cyclic_next(3), Some(4));
            assert_eq!(spec.cyclic_next(4), Some(6));
            assert_eq!(spec.cyclic_next(5), Some(6));
            assert_eq!(spec.cyclic_next(6), Some(0));

            let spec = Specifier::new(10 as u32, 100 as u32, SpecifierKind::EveryNth(10, 5));
            assert_eq!(spec.cyclic_next(10), Some(15));
            assert_eq!(spec.cyclic_next(11), Some(15));
            assert_eq!(spec.cyclic_next(16), Some(25));
            assert_eq!(spec.cyclic_next(20), Some(25));
            assert_eq!(spec.cyclic_next(25), Some(35));
            assert_eq!(spec.cyclic_next(55), Some(65));
            assert_eq!(spec.cyclic_next(64), Some(65));
            assert_eq!(spec.cyclic_next(90), Some(95));
            assert_eq!(spec.cyclic_next(90), Some(95));
            assert_eq!(spec.cyclic_next(95), Some(15));
            assert_eq!(spec.cyclic_next(97), Some(15));
            assert_eq!(spec.cyclic_next(100), Some(15));

            let spec = Specifier::new(100 as u32, 1000 as u32, SpecifierKind::EveryNth(50, 150));
            assert_eq!(spec.cyclic_next(100), Some(250));
            assert_eq!(spec.cyclic_next(110), Some(250));
            assert_eq!(spec.cyclic_next(250), Some(300));
            assert_eq!(spec.cyclic_next(900), Some(950));
            assert_eq!(spec.cyclic_next(950), Some(1000));
            assert_eq!(spec.cyclic_next(960), Some(1000));
            assert_eq!(spec.cyclic_next(1000), Some(250));
        }
    }
}
