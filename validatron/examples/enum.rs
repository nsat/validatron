use validatron::{Error, Result, Validate};

fn evaluate_basic(x: &Basic) -> Result<()> {
    match x {
        Basic::Good => Ok(()),
        Basic::Bad => Err(Error::new("is bad")),
    }
}

#[derive(Validate)]
#[validatron(function = "evaluate_basic")]
enum Basic {
    Good,
    Bad,
}

fn main() {
    assert!(Basic::Good.validate().is_ok());
    assert!(Basic::Bad.validate().is_err());
}
