pub mod error;
pub mod validators;

// re-export derive macro
pub use error::{Error, ErrorBuilder, Location};
pub use validatron_derive::Validate;

pub type Result<T> = std::result::Result<T, Error>;

/// The core Validatron trait, types that implement this trait can
/// be exhaustively validated.
///
/// Implementors should recursively validate internal structures.
pub trait Validate {
    fn validate(&self) -> Result<()>;
}

fn validate_seq<'a, I, T: 'a>(sequence: I) -> Result<()>
where
    I: IntoIterator<Item = &'a T>,
    T: Validate,
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

impl<K, V> Validate for std::collections::BTreeMap<K, V>
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

impl<T> Validate for Option<T>
where
    T: Validate,
    for<'a> &'a T: Validate,
{
    fn validate(&self) -> Result<()> {
        validate_seq(self)
    }
}
