//! A data structure validation library
//!
//! ```
//! use validatron::Validate;
//!
//! #[derive(Debug, Validate)]
//! struct MyStruct {
//!     #[validatron(min = 42)]
//!     a: i64,
//!     #[validatron(max_len = 5)]
//!     b: Vec<u32>,
//! }
//!
//! let x = MyStruct {
//!     a: 36,
//!     b: vec![]
//! };
//!
//! x.validate().is_err();
//! ```

/// An [`Error`](trait@std::error::Error) type for representing validation failures
pub mod error;

/// pre-rolled validators for data structures
pub mod validators;

// re-export derive macro
pub use error::{Error, Location};

/// A derive macro for validating data structures
pub use validatron_derive::Validate;

/// A convenience type for Results using the [`Error`] error type.
pub type Result<T> = std::result::Result<T, Error>;

/// The core Validatron trait, types that implement this trait can
/// be exhaustively validated.
///
/// Implementors should recursively validate internal structures.
pub trait Validate {
    /// Validate the implemented type exhaustively, returning all errors.
    fn validate(&self) -> Result<()>;
}

fn validate_seq<'a, I, T: 'a>(sequence: I) -> Result<()>
where
    I: IntoIterator<Item = &'a T>,
    T: Validate,
{
    let mut eb = Error::build();

    for (i, x) in sequence.into_iter().enumerate() {
        eb.try_at_index(i, x.validate());
    }

    eb.build()
}

impl<T> Validate for Vec<T>
where
    T: Validate,
{
    fn validate(&self) -> Result<()> {
        validate_seq(self)
    }
}

impl<T> Validate for std::collections::VecDeque<T>
where
    T: Validate,
{
    fn validate(&self) -> Result<()> {
        validate_seq(self)
    }
}

impl<T> Validate for std::collections::LinkedList<T>
where
    T: Validate,
{
    fn validate(&self) -> Result<()> {
        validate_seq(self)
    }
}

impl<K, V, S> Validate for std::collections::HashMap<K, V, S>
where
    K: std::fmt::Display,
    V: Validate,
{
    fn validate(&self) -> Result<()> {
        let mut eb = Error::build();

        for (k, v) in self {
            eb.try_at_named(k.to_string(), v.validate());
        }

        eb.build()
    }
}

impl<K, V> Validate for std::collections::BTreeMap<K, V>
where
    K: std::fmt::Display,
    V: Validate,
{
    fn validate(&self) -> Result<()> {
        let mut eb = Error::build();

        for (k, v) in self {
            eb.try_at_named(k.to_string(), v.validate());
        }

        eb.build()
    }
}

impl<T, S> Validate for std::collections::HashSet<T, S>
where
    T: Validate,
{
    fn validate(&self) -> Result<()> {
        validate_seq(self)
    }
}

impl<T> Validate for std::collections::BTreeSet<T>
where
    T: Validate,
{
    fn validate(&self) -> Result<()> {
        validate_seq(self)
    }
}

impl<T> Validate for std::collections::BinaryHeap<T>
where
    T: Validate,
{
    fn validate(&self) -> Result<()> {
        validate_seq(self)
    }
}

#[cfg(feature = "use-indexmap")]
impl<K, V> Validate for indexmap::IndexMap<K, V>
where
    K: std::fmt::Display,
    V: Validate,
{
    fn validate(&self) -> Result<()> {
        let mut eb = Error::build();

        for (k, v) in self {
            eb.try_at_named(k.to_string(), v.validate());
        }

        eb.build()
    }
}

#[cfg(feature = "use-indexmap")]
impl<T, S> Validate for indexmap::IndexSet<T, S>
where
    T: Validate,
{
    fn validate(&self) -> Result<()> {
        validate_seq(self)
    }
}

impl<T> Validate for Option<T>
where
    T: Validate,
{
    fn validate(&self) -> Result<()> {
        validate_seq(self)
    }
}

impl<T, E> Validate for std::result::Result<T, E>
where
    T: Validate,
{
    fn validate(&self) -> Result<()> {
        if let Ok(value) = self {
            value.validate()
        } else {
            Err(Error::new("value is an Error"))
        }
    }
}
