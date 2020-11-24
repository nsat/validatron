# Validatron

Validatron is a data structure validation library for rust that is designed for performing extensive
integrity checks on user supplied data prior.

It is heavily inspired by the [keats/validator][1] crate but with a few orthogonal design choices. These are primarily:

- Do not fail fast, return as many error as possible


## A quick example

```rust
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
```

[1]: https://github.com/Keats/validator
