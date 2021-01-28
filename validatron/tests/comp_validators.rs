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
fn field_option_min_validator() {
    #[derive(Validate)]
    struct Foo {
        #[validatron(option_min = 10)]
        a: Option<u64>,
    }

    assert_eq!(Foo { a: None }.validate().is_ok(), true);
    assert_eq!(Foo { a: Some(10) }.validate().is_ok(), true);
    assert_eq!(Foo { a: Some(20) }.validate().is_ok(), true);
    assert_eq!(Foo { a: Some(0) }.validate().is_ok(), false);
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
fn field_option_max_validator() {
    #[derive(Validate)]
    struct Foo {
        #[validatron(option_max = 10)]
        a: Option<u64>,
    }
    assert_eq!(Foo { a: None }.validate().is_ok(), true);
    assert_eq!(Foo { a: Some(10) }.validate().is_ok(), true);
    assert_eq!(Foo { a: Some(20) }.validate().is_ok(), false);
    assert_eq!(Foo { a: Some(0) }.validate().is_ok(), true);
}

#[test]
fn field_equal_validator() {
    #[derive(Validate)]
    struct Foo {
        #[validatron(equal = 10)]
        a: u64,
    }

    assert_eq!(Foo { a: 10 }.validate().is_ok(), true);
    assert_eq!(Foo { a: 20 }.validate().is_ok(), false);

    #[derive(Validate)]
    struct Bar {
        #[validatron(equal = "hello world")]
        a: &'static str,
    }

    assert_eq!(Bar { a: "hello world" }.validate().is_ok(), true);
    assert_eq!(
        Bar {
            a: "goodbye cruel world"
        }
        .validate()
        .is_ok(),
        false
    );
}
