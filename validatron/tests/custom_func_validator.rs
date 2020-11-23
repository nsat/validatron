use validatron::{Error, Validate};

#[test]
fn test_custom_field_validator() {
    fn is_u64_valid(x: &u64) -> Result<(), Error> {
        if *x <= 1 {
            Err(Error::new("is greater than 1"))
        } else {
            Ok(())
        }
    }

    #[derive(Validate)]
    struct Foo {
        #[validatron(function = "is_u64_valid")]
        a: u64,
    }

    assert!(Foo { a: 72 }.validate().is_ok());
    assert!(Foo { a: 0 }.validate().is_err());
}
