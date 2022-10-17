use validatron::{Error, Location, Validate};

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

#[derive(Validate)]
#[validatron(function = "my::nested::validators::check_foo")]
pub(crate) struct Foo {
    bar: String,
    baz: String,
}

pub(crate) mod my {
    pub(crate) mod nested {
        pub(crate) mod validators {
            use crate::Foo;
            use validatron::Error;

            pub(crate) fn check_foo(x: &Foo) -> Result<(), Error> {
                if x.bar == x.baz && &x.baz == "foo" {
                    Ok(())
                } else {
                    Err(Error::new("not foo"))
                }
            }
        }
    }
}

#[test]
fn custom_fn_module() {
    assert!(Foo {
        bar: "foo".to_string(),
        baz: "foo".to_string()
    }
    .validate()
    .is_ok());

    let e = Foo {
        bar: "baz".to_string(),
        baz: "foo".to_string(),
    }
    .validate()
    .unwrap_err();

    assert_eq!(
        e,
        Error::Structured(
            vec![(
                Location::Named("check_foo".into()),
                Error::Unstructured(vec!["not foo".into()])
            )]
            .into_iter()
            .collect()
        )
    )
}

#[test]
fn uses_existing_function() {
    #[derive(Validate)]
    struct Foo(#[validatron(predicate = "Option::is_some")] Option<i32>);

    assert!(Foo(Some(32)).validate().is_ok());

    let x = Foo(None);
    let e = x.validate().unwrap_err();

    println!("{:#?}", e);
    assert_eq!(
        e,
        Error::Structured(
            vec![(
                Location::Index(0),
                Error::Unstructured(vec!["Predicate \"is_some\" failed".into()])
            )]
            .into_iter()
            .collect()
        )
    )
}
