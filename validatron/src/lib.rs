use std::collections::HashMap;
use thiserror::Error;

#[cfg(feature = "use-serde")]
use serde::Serialize;

pub mod validators;

// re-export derive macro
pub use validatron_derive::Validate;

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

pub type Result<T> = std::result::Result<T, Error>;

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

pub trait Validate {
    fn validate(&self) -> Result<()>;
}

fn validate_seq<T>(sequence: T) -> Result<()>
where
    T: IntoIterator,
    <T as IntoIterator>::Item: Validate,
{
    let mut eb = ErrorBuilder::new();

    for (i, x) in sequence.into_iter().enumerate() {
        eb.at_index(i, x.validate());
    }

    eb.build()
}

impl<T> Validate for Vec<T>
where
    T: Validate,
{
    fn validate(&self) -> Result<()> {
        let mut eb = ErrorBuilder::new();

        for (i, x) in self.iter().enumerate() {
            eb.at_index(i, x.validate());
        }

        eb.build()
    }
}

impl<T> Validate for Option<T>
where
    T: Validate,
    for<'a> &'a T: Validate,
{
    fn validate(&self) -> Result<()> {
        validate_seq(self)
    }
}

impl<K, V, S> Validate for HashMap<K, V, S>
where
    K: ToString,
    V: Validate,
{
    fn validate(&self) -> Result<()> {
        let mut eb = ErrorBuilder::new();

        for (k, v) in self {
            let result = v.validate();

            if result.is_err() {
                eb.at_key(k.to_string(), result);
            }
        }

        eb.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
                .at_field("a", super::validators::min(&x.a, 5))
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

    #[test]
    fn basic() {
        struct Foo {};

        impl Validate for Foo {
            fn validate(&self) -> Result<()> {
                Ok(())
            }
        }
        let x = Foo {};
        assert_eq!(Ok(()), x.validate());
    }

    struct Dummy(bool);

    impl Validate for Dummy {
        fn validate(&self) -> Result<()> {
            if self.0 {
                Ok(())
            } else {
                Err(Error::new("false"))
            }
        }
    }
    impl<'a> Validate for &'a Dummy {
        fn validate(&self) -> Result<()> {
            (*self).validate()
        }
    }

    #[test]
    fn option_impl() {
        let a: Option<Dummy> = None;
        assert_eq!(validate_seq(&a).is_ok(), true);

        assert_eq!(Validate::validate(&a).is_ok(), true);
        assert_eq!(a.validate().is_ok(), true);

        let b = Some(Dummy(true));
        assert_eq!(b.validate().is_ok(), true);

        let b = Some(Dummy(false));
        assert_eq!(b.validate().is_ok(), false);
    }

    #[test]
    fn valid_sequence_test() {
        let inp: Vec<Dummy> = vec![];
        assert_eq!(validate_seq(inp).is_ok(), true);

        let mut inp = vec![Dummy(true)];
        assert_eq!(validate_seq(&inp).is_ok(), true);

        inp.push(Dummy(false));
        assert_eq!(validate_seq(&inp).is_ok(), false);

        inp.push(Dummy(true));
        inp.push(Dummy(false));

        let e = inp.validate().unwrap_err();
        match e {
            Error::Structured(map) => {
                assert_eq!(map.len(), 2);

                assert_eq!(map.contains_key(&Location::Index(0)), false);
                assert_eq!(map.contains_key(&Location::Index(1)), true);
                assert_eq!(map.contains_key(&Location::Index(2)), false);
                assert_eq!(map.contains_key(&Location::Index(3)), true);
            }
            _ => panic!("cannot happen"),
        }
    }

    #[test]
    fn valid_mapping() {
        let mut data = HashMap::new();

        assert!(data.validate().is_ok());

        data.insert("a place", Dummy(true));
        assert!(data.validate().is_ok());

        data.insert("a different place", Dummy(false));
        assert!(data.validate().is_err());

        let e = data.validate().unwrap_err();
        match e {
            Error::Field(_) => panic!("should happen"),
            Error::Structured(x) => {
                assert_eq!(x.len(), 1);
                assert!(x.contains_key(&Location::MapKey("a different place".into())));
            }
        }
    }

    #[test]
    fn large_test() -> Result<()> {
        struct Inner {
            first: u64,
            second: Vec<u64>,
        }

        impl Validate for Inner {
            fn validate(&self) -> super::Result<()> {
                ErrorBuilder::new()
                    .at_field("first", super::validators::is_equal(&self.first, 1))
                    .at_field("second", super::validators::is_min_length(&self.second, 1))
                    .build()
            }
        }

        struct Outer {
            a: u64,
            b: Inner,
        }

        impl Validate for Outer {
            fn validate(&self) -> super::Result<()> {
                ErrorBuilder::new()
                    .at_field("a", super::validators::is_equal(&self.a, 1))
                    .at_field("b", self.b.validate())
                    .build()
            }
        }

        let a = Outer {
            a: 24,
            b: Inner {
                first: 2,
                second: vec![1, 2, 3],
            },
        };

        let e = a.validate().unwrap_err();
        println!("{:#?}", e);

        Ok(())
    }
}
