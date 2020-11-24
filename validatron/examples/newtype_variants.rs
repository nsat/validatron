use validatron::{Error, Result, Validate};

fn main() {
    #[derive(Validate)]
    struct Unit();
    assert!(Unit().validate().is_ok());

    #[derive(Validate)]
    struct NewTypeA(Option<i32>);
    assert!(NewTypeA(None).validate().is_ok());
    assert!(NewTypeA(Some(1)).validate().is_ok());

    struct Dummy(bool);
    impl Validate for Dummy {
        fn validate(&self) -> Result<()> {
            if self.0 {
                Ok(())
            } else {
                Err(Error::new("value is false"))
            }
        }
    }

    #[derive(Validate)]
    struct NewTypeB(#[validatron] Option<Dummy>);

    let mut v = NewTypeB(None);
    assert!(v.validate().is_ok());

    v = NewTypeB(Some(Dummy(true)));
    assert!(v.validate().is_ok());

    v = NewTypeB(Some(Dummy(false)));
    assert!(v.validate().is_err());
}
