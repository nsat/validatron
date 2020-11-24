use validatron::Validate;

#[derive(Debug, Validate)]
struct MyStruct {
    #[validatron(min = 42)]
    a: i64,
    #[validatron(equal = "hello world!")]
    b: String,
}

fn main() {
    let good = MyStruct {
        a: 666,
        b: "hello world!".into(),
    };

    assert!(good.validate().is_ok());

    let bad = MyStruct {
        a: 1,
        b: "so long and thanks for all the fish".into(),
    };

    let result = bad.validate();
    assert!(result.is_err());

    println!("{:#?}", result);
}
