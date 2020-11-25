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

        // todo: simplify once https://github.com/rust-lang/rust/issues/68354
        // is stabilised
        match self {
            Error::Unstructured(x) => match other {
                Error::Unstructured(y) => x.extend(y.into_iter()),
                _ => panic!("can only merge duplicate variants"),
            },
            Error::Structured(x) => match other {
                Error::Structured(y) => {
                    x.extend(y.into_iter());
                }
                _ => panic!("can only merge duplicate variants"),
            },
        };
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

fn build_structured(errs: &mut Option<Error>, loc: Location, result: Result<()>) {
    if let Err(e) = result {
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
                entry.get_mut().merge(e);
            }
            Entry::Vacant(entry) => {
                entry.insert(e);
            }
        };

        *errs = Some(Error::Structured(structured_errs));
    }
}

impl ErrorBuilder {
    /// does the builder contain any error messages, used to short circuit
    /// various functions if no error has been detected.
    pub fn contains_errors(&self) -> bool {
        self.errors.is_some()
    }

    /// Consume the builder and produce a [`Result`]
    pub fn build(&mut self) -> Result<()> {
        if let Some(e) = self.errors.take() {
            Err(e)
        } else {
            Ok(())
        }
    }

    /// extend the existing builder with an error at the specified location
    pub fn at_location(&mut self, location: Location, result: Result<()>) -> &mut Self {
        build_structured(&mut self.errors, location, result);

        self
    }

    /// extend an existing builder with an error at a named location
    pub fn at_named(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        result: Result<()>,
    ) -> &mut Self {
        self.at_location(Location::Named(name.into()), result)
    }

    /// extend an existing builder with an error at an indexed location
    pub fn at_index(&mut self, index: usize, result: Result<()>) -> &mut Self {
        self.at_location(Location::Index(index), result)
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
                eb.at_index(i, is_positive(v));

                if i % 2 == 1 {
                    eb.at_index(i, Err(Error::new("must be multiple of 2")));
                }
            }

            eb.build()
        }

        fn validate_a_map(x: &HashMap<String, Bar>) -> Result<()> {
            let mut eb = Error::build();

            if x.contains_key("Foo") {
                eb.at_named(
                    "Foo",
                    Err(Error::new("must not contain a key with this name")),
                );
            }

            for (k, v) in x {
                eb.at_named(k.to_string(), v.validate());
            }

            eb.build()
        }

        fn validate_foo(x: &Foo) -> Result<()> {
            Error::build()
                .at_named("a", crate::validators::min(&x.a, 5))
                .at_named("b", validate_a_vector(&x.b))
                .at_named("c", validate_a_map(&x.c))
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
    fn test_error_merge() {
        let mut a = Error::new("a");
        let b = Error::new("b");

        a.merge(b);

        match a {
            Error::Unstructured(x) => {
                assert_eq!(x, vec!["a", "b"]);
            }
            Error::Structured(_) => panic!("should not happen"),
        }

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
}
