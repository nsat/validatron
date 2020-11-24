use std::collections::HashMap;
use thiserror::Error;

use crate::Result;

#[cfg(feature = "use-serde")]
use serde::Serialize;

#[derive(Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "use-serde", derive(Serialize), serde(untagged))]
pub enum Location {
    NamedField(&'static str),
    MapKey(String),
    Index(usize),
}

#[derive(Error, Debug, PartialEq)]
#[cfg_attr(feature = "use-serde", derive(Serialize), serde(untagged))]
pub enum Error {
    #[error("{0:#?}")]
    Field(Vec<String>),
    #[error("{0:#?}")]
    Structured(HashMap<Location, Error>),
}

impl Error {
    pub fn new<S: Into<String>>(message: S) -> Self {
        Self::Field(vec![message.into()])
    }

    pub fn merge(&mut self, other: Error) {
        // Multi + Multi -> Multi
        // Located + Located -> Located

        match self {
            Error::Field(x) => match other {
                Error::Field(y) => x.extend(y.into_iter()),
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
}

pub struct ErrorBuilder {
    errors: Option<Error>,
}

impl Default for ErrorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

fn build_structured(errs: &mut Option<Error>, loc: Location, result: Result<()>) {
    if let Err(e) = result {
        let mut structured_errs = errs
            .take()
            .map(|e| match e {
                Error::Field(_) => panic!("should never happen"),
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
    pub fn new() -> Self {
        Self { errors: None }
    }

    pub fn extend(existing: Result<()>) -> Self {
        Self {
            errors: existing.err(),
        }
    }

    pub fn contains_errors(&self) -> bool {
        self.errors.is_some()
    }

    pub fn build(&mut self) -> Result<()> {
        if let Some(e) = self.errors.take() {
            Err(e)
        } else {
            Ok(())
        }
    }

    pub fn because(&mut self, message: impl Into<String>) -> &mut Self {
        if let Some(e) = &mut self.errors {
            e.merge(Error::Field(vec![message.into()]))
        } else {
            self.errors = Some(Error::Field(vec![message.into()]))
        }

        self
    }

    pub fn at_location(&mut self, location: Location, result: Result<()>) -> &mut Self {
        build_structured(&mut self.errors, location, result);

        self
    }

    pub fn at_field(&mut self, field: &'static str, result: Result<()>) -> &mut Self {
        build_structured(&mut self.errors, Location::NamedField(field), result);

        self
    }

    pub fn at_index(&mut self, index: usize, result: Result<()>) -> &mut Self {
        build_structured(&mut self.errors, Location::Index(index), result);

        self
    }

    pub fn at_key(&mut self, key: impl Into<String>, result: Result<()>) -> &mut Self {
        build_structured(&mut self.errors, Location::MapKey(key.into()), result);

        self
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
                let mut eb = ErrorBuilder::new();

                if (self.0 - 42.).abs() < 0.1 {
                    eb.because(
                        "cannot comprehend the meaning of life the universe and everything.",
                    );
                }

                eb.build()
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
            let mut eb = ErrorBuilder::new();

            for (i, v) in x.iter().enumerate() {
                eb.at_index(i, is_positive(v));

                if i % 2 == 1 {
                    eb.at_index(i, Err(Error::new("must be multiple of 2")));
                }
            }

            eb.build()
        }

        fn validate_a_map(x: &HashMap<String, Bar>) -> Result<()> {
            let mut eb = ErrorBuilder::new();

            if x.contains_key("Foo") {
                eb.at_key(
                    "Foo",
                    Err(Error::new("must not contain a key with this name")),
                );
            }

            for (k, v) in x {
                eb.at_key(k, v.validate());
            }

            eb.build()
        }

        fn validate_foo(x: &Foo) -> Result<()> {
            ErrorBuilder::new()
                .at_field("a", crate::validators::min(&x.a, 5))
                .at_field("b", validate_a_vector(&x.b))
                .at_field("c", validate_a_map(&x.c))
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
            Error::Field(x) => {
                assert_eq!(x, vec!["a", "b"]);
            }
            Error::Structured(_) => panic!("should not happen"),
        }

        let mut a = Error::new("a");
        let b = Error::new("b");

        a.merge(b);

        match a {
            Error::Field(x) => {
                assert_eq!(x, vec!["a", "b"]);
            }
            Error::Structured(_) => panic!("should not happen"),
        }
    }
}
