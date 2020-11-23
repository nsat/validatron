use validatron::Validate;

// todo Implement NewType support
fn main() {
    #[derive(Validate)]
    struct Unit();

    assert!(Unit().validate().is_ok());

    #[derive(Validate)]
    struct NewTypeA(Option<u32>);
    assert!(NewTypeA(None).validate().is_ok());
    assert!(NewTypeA(Some(1)).validate().is_ok());

    #[derive(Validate)]
    struct NewTypeB(#[validatron] Option<u32>);
    assert!(NewTypeB(Some(12)).validate().is_ok());
}
