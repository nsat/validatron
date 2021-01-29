use validatron::Validate;

#[derive(Debug, Validate)]
struct MyStruct {
    #[validatron(min = 42)]
    a: i64,

    #[validatron(min_len = 10)]
    b: Vec<i32>,

    #[validatron(min = "42 + 12")]
    c: i32,
}

fn main() {
    let good = MyStruct {
        a: 666,
        b: vec![0; 15],
        c: 666,
    };

    println!("{:#?}", good.validate());
    assert!(good.validate().is_ok());

    let bad = MyStruct {
        a: 1,
        b: vec![],
        c: 5,
    };

    let result = bad.validate();
    assert!(result.is_err());

    println!("{:#?}", result);
}
