use validatron::Validate;

#[test]
fn newtype_does_not_impl_validate() {
    #[derive(Validate)]
    struct NewType(i32);

    let x = NewType(23);
    assert!(x.validate().is_ok());
}

#[test]
fn newtype_has_custom_attr() {
    #[derive(Validate)]
    struct NewType(#[validatron(equal = 42)] i32);

    let x = NewType(42);
    assert!(x.validate().is_ok());

    let x = NewType(36);
    assert!(x.validate().is_err());
}

#[test]
fn newtype_recurses() {
    #[derive(Validate)]
    struct NewType(#[validatron(equal = 42)] i32);

    #[derive(Validate)]
    struct SecondNewType(#[validatron] NewType);
    let x = SecondNewType(NewType(42));
    assert!(x.validate().is_ok());

    let x = SecondNewType(NewType(36));
    assert!(x.validate().is_err());
}
