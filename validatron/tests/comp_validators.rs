use validatron::Validate;

#[test]
fn field_min_validator() {
    #[derive(Validate)]
    struct Foo {
        #[validatron(min = 10)]
        a: u64,
    }

    assert_eq!(Foo { a: 10 }.validate().is_ok(), true);
    assert_eq!(Foo { a: 20 }.validate().is_ok(), true);
    assert_eq!(Foo { a: 0 }.validate().is_ok(), false);
}

#[test]
fn field_max_validator() {
    #[derive(Validate)]
    struct Foo {
        #[validatron(max = 10)]
        a: u64,
    }

    assert_eq!(Foo { a: 10 }.validate().is_ok(), true);
    assert_eq!(Foo { a: 20 }.validate().is_ok(), false);
    assert_eq!(Foo { a: 0 }.validate().is_ok(), true);
}

#[test]
fn field_equal_validator() {
    #[derive(Validate)]
    struct Foo {
        #[validatron(equal = 10)]
        #[validatron(equal = "10")]
        #[validatron(equal = "9 + 1")]
        a: u64,

        #[validatron(equal = "\"hello world!\"")]
        b: String,
    }

    assert_eq!(
        Foo {
            a: 10,
            b: "hello world!".into()
        }
        .validate()
        .is_ok(),
        true
    );
    assert_eq!(
        Foo {
            a: 20,
            b: "".into()
        }
        .validate()
        .is_ok(),
        false
    );
}
