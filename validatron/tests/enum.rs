use validatron::{Error, Result, Validate};

#[derive(Validate)]
struct Dummy(#[validatron(equal = true)] bool);

#[test]
fn smoke() {
    #[derive(Validate)]
    enum MyEnum {
        Unit,
        NewType(i32),
        Tuple(i32, i32),
        Struct { a: i32 },
    }

    assert!(MyEnum::Unit.validate().is_ok());
    assert!(MyEnum::NewType(42).validate().is_ok());
    assert!(MyEnum::Tuple(42, 101).validate().is_ok());
    assert!(MyEnum::Struct { a: 42 }.validate().is_ok());
}

#[test]
fn enum_custom_func() {
    #[derive(Validate)]
    #[validatron(function = "evaluate")]
    enum MyEnum {
        Good,
        Bad,
    }

    fn evaluate(x: &MyEnum) -> Result<()> {
        match x {
            MyEnum::Good => Ok(()),
            MyEnum::Bad => Err(Error::new("is bad")),
        }
    }

    assert!(MyEnum::Good.validate().is_ok());
    assert!(MyEnum::Bad.validate().is_err());
}

#[test]
fn enum_new_type() {
    #[derive(Validate)]
    enum MyEnum {
        WithAttr(#[validatron] Dummy),

        WithCustomAttr(#[validatron(equal = "32")] i64),
    }

    assert!(MyEnum::WithAttr(Dummy(true)).validate().is_ok());
    assert!(MyEnum::WithAttr(Dummy(false)).validate().is_err());

    assert!(MyEnum::WithCustomAttr(32).validate().is_ok());
    assert!(MyEnum::WithCustomAttr(1).validate().is_err());
}

#[test]
fn enum_tuple_type() {
    #[derive(Validate)]
    enum MyEnum {
        Recurse(#[validatron] Dummy, #[validatron] Dummy),

        Custom(
            #[validatron(equal = true)] bool,
            #[validatron(equal = true)] bool,
        ),

        Mixed(#[validatron] Dummy, #[validatron(equal = true)] bool),
    }

    assert!(MyEnum::Recurse(Dummy(true), Dummy(true)).validate().is_ok());
    assert!(MyEnum::Recurse(Dummy(true), Dummy(false))
        .validate()
        .is_err());

    assert!(MyEnum::Custom(true, true).validate().is_ok());
    assert!(MyEnum::Custom(true, false).validate().is_err());

    assert!(MyEnum::Mixed(Dummy(true), true).validate().is_ok());
    assert!(MyEnum::Mixed(Dummy(false), true).validate().is_err());
    assert!(MyEnum::Mixed(Dummy(true), false).validate().is_err());
}

#[test]
fn enum_struct_var() {
    #[derive(Validate)]
    enum MyEnum {
        #[validatron]
        Recurse {
            #[validatron]
            a: Dummy,
        },

        #[validatron]
        Custom {
            #[validatron(equal = true)]
            a: bool,
        },

        #[validatron]
        Mixed {
            #[validatron]
            a: Dummy,

            #[validatron(equal = true)]
            b: bool,
        },
    }

    assert!(MyEnum::Recurse { a: Dummy(true) }.validate().is_ok());
    assert!(MyEnum::Recurse { a: Dummy(false) }.validate().is_err());

    assert!(MyEnum::Custom { a: true }.validate().is_ok());
    assert!(MyEnum::Custom { a: false }.validate().is_err());

    assert!(MyEnum::Mixed {
        a: Dummy(true),
        b: true
    }
    .validate()
    .is_ok());
    assert!(MyEnum::Mixed {
        a: Dummy(false),
        b: true
    }
    .validate()
    .is_err());
    assert!(MyEnum::Mixed {
        a: Dummy(true),
        b: false
    }
    .validate()
    .is_err());
}
