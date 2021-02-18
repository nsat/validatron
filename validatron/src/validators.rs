use crate::{Error, Result};
use std::fmt::Display;

/// Check that an option has a value
///
/// ```
/// # use validatron::validators::is_required;
/// let x = Some(42);
/// assert!(is_required(&x).is_ok());
///
/// let y = None;
/// assert!(is_required::<i64>(&y).is_err());
/// ```
pub fn is_required<T>(value: &Option<T>) -> Result<()> {
    if value.is_none() {
        Err(Error::new("Option is required to have a value"))
    } else {
        Ok(())
    }
}

/// Check that values are equal
///
/// ```
/// # use validatron::validators::is_equal;
/// assert!(is_equal(&42, 42).is_ok());
/// assert!(is_equal(&String::from("hello world"), "hello world").is_ok());
/// assert!(is_equal(&1.0, 2.0).is_err());
/// ```
pub fn is_equal<L, R>(value: &L, other: R) -> Result<()>
where
    L: PartialEq<R> + Display,
    R: Display,
{
    if *value == other {
        Ok(())
    } else {
        Err(Error::new(format!("'{}' must equal '{}'", value, other)))
    }
}

/// Check that a value is greater than a value
///
/// ```
/// # use validatron::validators::min;
/// assert!(min(&42, 0).is_ok());
/// assert!(min(&1.0, 2.0).is_err());
/// ```
pub fn min<L, R>(value: &L, min: R) -> Result<()>
where
    L: PartialOrd<R> + Display,
    R: Display,
{
    if *value < min {
        Err(Error::new(format!("'{}' must be greater than or equal to '{}'", value, min)))
    } else {
        Ok(())
    }
}

/// Check that an optional value is either none or greater than a value
///
/// ```
/// # use validatron::validators::option_min;
/// let x = None::<u64>;
/// assert!(option_min(&x, 1).is_ok());
/// assert!(option_min(&Some(1.0), 2.0).is_err());
/// assert!(option_min(&Some(3), 2).is_ok());
/// ```
///
pub fn option_min<L, R>(value: &Option<L>, min_value: R) -> Result<()>
where
    L: PartialOrd<R> + Display,
    R: Display,
{
    if let Some(x) = value {
        min(x, min_value)
    } else {
        Ok(())
    }
}

/// Check that a value is less than a max
///
/// ```
/// # use validatron::validators::max;
/// assert!(max(&42, 128).is_ok());
/// assert!(max(&2.0, 1.0).is_err());
/// ```
pub fn max<L, R>(value: &L, max: R) -> Result<()>
where
    L: PartialOrd<R> + Display,
    R: Display,
{
    if *value > max {
        Err(Error::new(format!("'{}' must be less than or equal to '{}'", value, max)))
    } else {
        Ok(())
    }
}

/// Check that an optional value is either none or less than a value
///
/// ```
/// # use validatron::validators::option_max;
/// let x = None::<u64>;
/// assert!(option_max(&x, 1).is_ok());
/// assert!(option_max(&Some(1.0), 2.0).is_ok());
/// assert!(option_max(&Some(3), 2).is_err());
/// ```
///
pub fn option_max<L, R>(value: &Option<L>, max_value: R) -> Result<()>
where
    L: PartialOrd<R> + Display,
    R: Display,
{
    if let Some(x) = value {
        max(x, max_value)
    } else {
        Ok(())
    }
}

fn sequence_length<C>(iterable: C) -> usize
where
    C: IntoIterator,
{
    iterable.into_iter().count()
}

/// Check that a sequence is at least a certain length
///
/// ```
/// # use validatron::validators::is_min_length;
/// let x = vec![1,2,3,4,5];
/// assert!(is_min_length(&x, 0).is_ok());
/// assert!(is_min_length(&x, 2).is_ok());
/// assert!(is_min_length(&x, 6).is_err());
/// ```
pub fn is_min_length<C>(iterable: C, min_length: usize) -> Result<()>
where
    C: IntoIterator,
{
    let len = sequence_length(iterable);

    if len < min_length {
        Err(Error::new(format!(
            "sequence does not have enough elements, it has {} but the minimum is {}",
            len, min_length
        )))
    } else {
        Ok(())
    }
}

/// Check that a sequence is at most a certain length
///
/// ```
/// # use validatron::validators::is_max_length;
/// let x = vec![1,2,3,4,5];
/// assert!(is_max_length(&x, 10).is_ok());
/// assert!(is_max_length(&x, 5).is_ok());
/// assert!(is_max_length(&x, 2).is_err());
/// ```
pub fn is_max_length<C>(iterable: C, max_length: usize) -> Result<()>
where
    C: IntoIterator,
{
    let len = sequence_length(iterable);

    if len > max_length {
        Err(Error::new(format!(
            "sequence has too many elements, it has {} but the maximum is {}",
            len, max_length
        )))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_equal_comp() {
        assert_eq!(is_equal(&1, 1), Ok(()));

        assert!(is_equal(&1, 2).is_err());
        assert!(is_equal(&2, 1).is_err());

        assert!(is_equal(&"foo", "foo").is_ok());

        let a = "foo";
        let b = "bar".to_string();

        assert!(is_equal(&a, b).is_err());
    }

    #[test]
    fn test_min() {
        assert!(min(&0, 0).is_ok());
        assert!(min(&1, 0).is_ok());
        assert!(min(&20., 0.).is_ok());
        assert!(min(&6, 5).is_ok());

        assert!(min(&0, 1).is_err());
        assert!(min(&5, 6).is_err());
        assert!(min(&10., 42.).is_err());
    }

    #[test]
    fn test_max() {
        assert!(max(&0, 0).is_ok());
        assert!(max(&0, 1).is_ok());
        assert!(max(&0., 20.).is_ok());
        assert!(max(&5, 6).is_ok());

        assert!(max(&1, 0).is_err());
        assert!(max(&6, 5).is_err());
        assert!(max(&42., 10.).is_err());
    }

    #[test]
    fn test_min_length() {
        assert!(is_min_length(vec![1, 2, 3], 3).is_ok());
        assert!(is_min_length(vec![1, 2, 3], 4).is_err());
        assert!(is_min_length(vec![1, 2], 3).is_err());

        assert!(is_min_length(&[1, 2], 2).is_ok());
        assert!(is_min_length(&[1, 2, 3, 4, 5], 0).is_ok());
    }

    #[test]
    fn test_max_length() {
        assert!(is_max_length(vec![1, 2, 3], 3).is_ok());
        assert!(is_max_length(vec![1, 2, 3], 2).is_err());

        assert!(is_max_length(&[1, 2], 2).is_ok());

        assert!(is_max_length(Vec::<i32>::new(), 0).is_ok());
    }
}
