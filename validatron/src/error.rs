use std::borrow::Cow;
use std::collections::HashMap;
use thiserror::Error;

use crate::Result;

#[cfg(feature = "use-serde")]
use serde::Serialize;

/// The location within a data structure in which a validation error could
/// occur. Similar to serde we only support json style data structures with
/// either numerically indexed or keyed locations.
#[derive(Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "use-serde", derive(Serialize), serde(untagged))]
pub enum Location {
    // todo: can this be <'a>?
    /// A keyed location, this could be a struct field or a map key
    Named(Cow<'static, str>),
    /// An indexed location, this could be a tuple or a vector index
    Index(usize),
}

// todo: use a none-str type as the reason type?
/// A type that represents all validation issues that arise during the validation
/// of the given data type.
#[derive(Error, Debug, PartialEq)]
#[cfg_attr(feature = "use-serde", derive(Serialize), serde(untagged))]
pub enum Error {
    /// A flat, unstructured list of failure reasons
    #[error("{0:#?}")]
    Unstructured(Vec<Cow<'static, str>>),

    /// A structured, potentially nested set of failure reasons
    ///
    /// a vector or a nested map can attribute errors to the correct locations
    #[error("{0:#?}")]
    Structured(HashMap<Location, Error>),
}

impl Error {
    /// Constructs a new unstructured [`enum@Error`] with a single message
    ///
    /// ```
    /// # use validatron::Error;
    /// let e = Error::new("the universe divided by 0");
    /// ```
    pub fn new<S>(message: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self::Unstructured(vec![message.into()])
    }

    /// Merge 2 existing [`enum@Error`] types
    ///
    /// ```
    /// # use validatron::Error;
    /// let mut e1 = Error::new("the universe divided by 0");
    /// let e2 = Error::new("an unstoppable force collided with an improvable object");
    ///
    /// e1.merge(e2);
    /// ```
    pub fn merge(&mut self, other: Error) {
        // Multi + Multi -> Multi
        // Located + Located -> Located

        match (self, other) {
            (Error::Unstructured(xs), Error::Unstructured(ys)) => {
                xs.extend(ys);
            }
            (x @ Error::Unstructured(_), mut y @ Error::Structured(_)) => {
                std::mem::swap(x, &mut y);

                x.merge(y)
            }
            (x @ Error::Structured(_), y @ Error::Unstructured(_)) => {
                let mut map = HashMap::new();
                map.insert(Location::Named(Cow::from("errors")), y);

                x.merge(Error::Structured(map));
            }
            (Error::Structured(x), Error::Structured(ys)) => {
                use std::collections::hash_map::Entry;
                for (k, v) in ys {
                    match x.entry(k) {
                        Entry::Occupied(mut entry) => {
                            entry.get_mut().merge(v);
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(v);
                        }
                    }
                }
            }
        }
    }

    /// create a new [`ErrorBuilder`] instance
    pub fn build() -> ErrorBuilder {
        ErrorBuilder { errors: None }
    }
}

/// A convenience type for building a structured error type
pub struct ErrorBuilder {
    errors: Option<Error>,
}

fn build_structured(errs: &mut Option<Error>, loc: Location, error: Error) {
    let mut structured_errs = errs
        .take()
        .map(|e| match e {
            Error::Unstructured(_) => panic!("should never happen"),
            Error::Structured(hm) => hm,
        })
        .unwrap_or_else(HashMap::new);

    use std::collections::hash_map::Entry;
    match structured_errs.entry(loc) {
        Entry::Occupied(mut entry) => {
            entry.get_mut().merge(error);
        }
        Entry::Vacant(entry) => {
            entry.insert(error);
        }
    };

    *errs = Some(Error::Structured(structured_errs));
}

impl ErrorBuilder {
    /// does the builder contain any error messages, used to short circuit
    /// various functions if no error has been detected.
    pub fn contains_errors(&self) -> bool {
        self.errors.is_some()
    }

    /// Consume the builder and produce a [`Result`]
    ///
    /// ```
    /// # use validatron::Error;
    /// let e = Error::build()
    ///     .at_named("a", "flat out broken")
    ///     .build();
    /// assert!(e.is_err());
    /// ```
    pub fn build(&mut self) -> Result<()> {
        if let Some(e) = self.errors.take() {
            Err(e)
        } else {
            Ok(())
        }
    }

    /// extend the existing builder with an error at the specified location
    pub fn at_location(
        &mut self,
        location: Location,
        message: impl Into<Cow<'static, str>>,
    ) -> &mut Self {
        let e = Error::new(message);

        build_structured(&mut self.errors, location, e);

        self
    }

    /// extend an existing builder with an error at a named location
    ///
    /// ```
    /// # use validatron::Error;
    /// let e = Error::build()
    ///     .at_named("field_1", "should be empty")
    ///     .build();
    /// ```
    pub fn at_named(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        message: impl Into<Cow<'static, str>>,
    ) -> &mut Self {
        self.at_location(Location::Named(name.into()), message)
    }

    /// extend an existing builder with an error at an indexed location
    ///
    /// ```
    /// # use validatron::Error;
    /// let e = Error::build()
    ///     .at_index(1, "value should be even")
    ///     .build();
    /// ```
    pub fn at_index(&mut self, index: usize, message: impl Into<Cow<'static, str>>) -> &mut Self {
        self.at_location(Location::Index(index), message)
    }

    /// extend the existing builder at the specified location if the result is an error
    ///
    /// ```
    /// # use validatron::{Error, Location};
    /// let e = Error::build()
    ///     .try_at_location(Location::Index(1), Ok(()))
    ///     .build();
    /// assert!(e.is_ok());
    /// ```
    pub fn try_at_location(&mut self, location: Location, result: Result<()>) -> &mut Self {
        if let Err(e) = result {
            build_structured(&mut self.errors, location, e);
        }

        self
    }

    /// extend an existing builder with an error at a named location if the result is an error
    ///
    /// ```
    /// # use validatron::{Error, Location};
    /// let e = Error::build()
    ///     .try_at_named("field", Ok(()))
    ///     .build();
    /// assert!(e.is_ok());
    /// ```
    pub fn try_at_named(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        result: Result<()>,
    ) -> &mut Self {
        self.try_at_location(Location::Named(name.into()), result)
    }

    /// extend an existing builder with an error at an indexed location if the result is an error
    ///
    /// ```
    /// # use validatron::{Error, Location};
    /// let e = Error::build()
    ///     .try_at_index(42, Ok(()))
    ///     .build();
    /// assert!(e.is_ok());
    /// ```
    pub fn try_at_index(&mut self, index: usize, result: Result<()>) -> &mut Self {
        self.try_at_location(Location::Index(index), result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Validate;

    #[test]
    fn builder_works() {
        struct Bar(f64);
        impl Validate for Bar {
            fn validate(&self) -> Result<()> {
                if (self.0 - 42.).abs() < 0.1 {
                    Err(Error::new(
                        "cannot comprehend the meaning of life the universe and everything.",
                    ))
                } else {
                    Ok(())
                }
            }
        }

        struct Foo {
            a: u64,
            b: Vec<i64>,
            c: HashMap<String, Bar>,
        }

        fn is_positive(x: &i64) -> Result<()> {
            if *x < 0 {
                Err(Error::new("value must be positive"))
            } else {
                Ok(())
            }
        }

        fn validate_a_vector(x: &[i64]) -> Result<()> {
            let mut eb = Error::build();

            for (i, v) in x.iter().enumerate() {
                eb.try_at_index(i, is_positive(v));

                if i % 2 == 1 {
                    eb.at_index(i, "must be multiple of 2");
                }
            }

            eb.build()
        }

        fn validate_a_map(x: &HashMap<String, Bar>) -> Result<()> {
            let mut eb = Error::build();

            if x.contains_key("Foo") {
                eb.at_named("Foo", "must not contain a key with this name");
            }

            for (k, v) in x {
                eb.try_at_named(k.to_string(), v.validate());
            }

            eb.build()
        }

        fn validate_foo(x: &Foo) -> Result<()> {
            Error::build()
                .try_at_named("a", crate::validators::min(&x.a, 5))
                .try_at_named("b", validate_a_vector(&x.b))
                .try_at_named("c", validate_a_map(&x.c))
                .build()
        }

        let value = Foo {
            a: 10,
            b: vec![-1, 0, 1, 2],
            c: vec![
                ("Foo".to_string(), Bar(42.)),
                ("Not Foo".to_string(), Bar(666.)),
            ]
            .into_iter()
            .collect(),
        };

        assert!(validate_foo(&value).is_err());
    }

    #[test]
    fn test_errors() {
        let _e = Error::new("foo");
        let _e = Error::new("foo".to_string());
    }

    #[test]
    fn test_error_merge_2x_unstructured() {
        let mut a = Error::new("a");
        let b = Error::new("b");

        a.merge(b);

        match a {
            Error::Unstructured(x) => {
                assert_eq!(x, vec!["a", "b"]);
            }
            Error::Structured(_) => panic!("should not happen"),
        }
    }

    fn test_unstruct_plus_struc_merge(e: &Error) {
        match e {
            Error::Unstructured(_) => panic!("should not happen"),
            Error::Structured(x) => {
                assert_eq!(
                    x[&Location::Named("dummy".into())],
                    Error::new("something happened")
                );
                assert_eq!(x[&Location::Named("errors".into())], Error::new("b"));
            }
        }
    }

    #[test]
    fn test_error_merge_unstructured_plus_structured() {
        let mut struc = Error::Structured(
            [(
                Location::Named("dummy".into()),
                Error::new("something happened"),
            )]
            .into(),
        );
        let unstruc = Error::new("b");

        struc.merge(unstruc);

        test_unstruct_plus_struc_merge(&struc);
    }

    #[test]
    fn test_error_merge_structured_plus_unstructured() {
        let struc = Error::Structured(
            [(
                Location::Named("dummy".into()),
                Error::new("something happened"),
            )]
            .into(),
        );
        let mut unstruc = Error::new("b");

        unstruc.merge(struc);

        test_unstruct_plus_struc_merge(&unstruc);
    }

    #[test]
    fn test_error_merge_structured_recurse() {
        let mut a = Error::Structured([(Location::Named("dummy".into()), Error::new("a"))].into());
        let b = Error::Structured(
            [
                (Location::Named("dummy".into()), Error::new("b")),
                (Location::Named("also_dummy".into()), Error::new("c")),
            ]
            .into(),
        );

        a.merge(b);

        match a {
            Error::Unstructured(_) => panic!("should not happen"),
            Error::Structured(x) => {
                assert_eq!(
                    x[&Location::Named("dummy".into())],
                    Error::Unstructured(vec!["a".into(), "b".into()])
                );
                assert_eq!(
                    x[&Location::Named("also_dummy".into())],
                    Error::Unstructured(vec!["c".into()])
                );
            }
        }
    }
}
