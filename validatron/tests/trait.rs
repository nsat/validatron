use validatron::validators::{is_equal, is_min_length};
use validatron::{ErrorBuilder, Result, Validate};

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

#[test]
fn large_test() -> Result<()> {
    struct Inner {
        first: u64,
        second: Vec<u64>,
    }

    impl Validate for Inner {
        fn validate(&self) -> Result<()> {
            ErrorBuilder::new()
                .at_field("first", is_equal(&self.first, 1))
                .at_field("second", is_min_length(&self.second, 1))
                .build()
        }
    }

    struct Outer {
        a: u64,
        b: Inner,
    }

    impl Validate for Outer {
        fn validate(&self) -> Result<()> {
            ErrorBuilder::new()
                .at_field("a", is_equal(&self.a, 1))
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
