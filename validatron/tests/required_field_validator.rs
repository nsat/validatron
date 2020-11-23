use validatron::Validate;

#[test]
fn test_struct_with_derive_required_validator() {
    #[derive(Validate)]
    struct Foo {
        // #[validatron(function = "Option::is_some")]
        #[validatron(required)]
        a: Option<u64>,
    }

    assert_eq!(Foo { a: None }.validate().is_ok(), false);
    assert_eq!(Foo { a: Some(12) }.validate().is_ok(), true);
}
