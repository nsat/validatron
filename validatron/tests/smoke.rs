use validatron::Validate;

#[test]
fn test_unit_struct_is_valid() {
    #[derive(Validate)]
    struct Foo {}

    let f = Foo {};

    assert_eq!(f.validate(), Ok(()));
}

#[test]
fn test_struct_with_nested_fields() {
    #[derive(Validate)]
    struct NestedValidateStruct {
        #[validatron(min = 1)]
        a: u64,
    }

    #[derive(Validate)]
    struct OuterValidatedStruct {
        #[validatron]
        b: NestedValidateStruct,
    }

    let mut f = OuterValidatedStruct {
        b: NestedValidateStruct { a: 12 },
    };

    assert!(f.validate().is_ok());

    f.b.a = 0;
    assert!(f.validate().is_err());
}

#[test]
fn newtype_struct_derive() {
    #[derive(Validate)]
    struct Unit();

    assert!(Unit().validate().is_ok());

    #[derive(Validate)]
    struct NewType(u32);
    assert!(NewType(12).validate().is_ok());
}

#[test]
fn enum_derive() {
    // todo doesn't work yet
    #[allow(dead_code)]
    enum Foo {
        Unit,
        NewType(u64),
        TupleType(u64, u32),
        StructType { a: u64, b: u32 },
    }
}
