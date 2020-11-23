use crate::{Error, ErrorBuilder, Result};
use std::fmt::Display;

pub fn is_required<T>(value: &Option<T>) -> Result<()> {
    let mut eb = ErrorBuilder::new();

    if value.is_none() {
        eb.because("Option is required to have a value");
    }

    eb.build()
}

pub fn is_equal<L, R>(value: &L, other: R) -> Result<()>
where
    L: PartialEq<R> + Display,
    R: Display,
{
    if *value == other {
        Ok(())
    } else {
        Err(Error::new(format!("{} != {}", value, other)))
    }
}

pub fn min<L, R>(value: &L, min: R) -> Result<()>
where
    L: PartialOrd<R> + Display,
    R: Display,
{
    if *value < min {
        Err(Error::new(format!("{} is less than {}", value, min)))
    } else {
        Ok(())
    }
}

pub fn max<L, R>(value: &L, max: R) -> Result<()>
where
    L: PartialOrd<R> + Display,
    R: Display,
{
    if *value > max {
        Err(Error::new(format!("'{}' is greater than '{}'", value, max)))
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

pub fn is_min_length<C>(iterable: C, min_length: usize) -> Result<()>
where
    C: IntoIterator,
{
    let mut eb = ErrorBuilder::new();

    let len = sequence_length(iterable);

    if len < min_length {
        eb.because(format!(
            "sequence does not have enough elements, it has {} but the minimum is {}",
            len, min_length
        ));
    }

    eb.build()
}

pub fn is_max_length<C>(iterable: C, max_length: usize) -> Result<()>
where
    C: IntoIterator,
{
    let mut eb = ErrorBuilder::new();

    let len = sequence_length(iterable);

    if len > max_length {
        eb.because(format!(
            "sequence has too many elements, it has {} but the maximum is {}",
            len, max_length
        ));
    }

    eb.build()
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
