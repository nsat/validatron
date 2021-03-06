# Validatron [![Build Status]][actions] [![Docs]][docs.rs] [![Latest Version]][crates.io]


[Build Status]: https://img.shields.io/github/workflow/status/nsat/validatron/Rust/master
[actions]: https://github.com/nsat/validatron/actions?query=branch%3Amaster
[Docs]: https://docs.rs/validatron/badge.svg
[docs.rs]: https://docs.rs/validatron/
[Latest Version]: https://img.shields.io/crates/v/validatron.svg
[crates.io]: https://crates.io/crates/validatron


**Validatron is a data structure validation library for Rust that is designed for performing
extensive integrity checks on user supplied data prior to use.**

It is heavily inspired by the [keats/validator][1] crate but with different design choices:

- do not fail fast, return as many errors as possible
- return a serializable error type
- provide easily extendable validators

## Example

(Check the [examples](/validatron/examples) directory for additional examples.)

```rust
use validatron::Validate;

#[derive(Debug, Validate)]
struct MyStruct {
    #[validatron(min = 42)]
    a: i64,
    #[validatron(max_len = 5)]
    b: Vec<u32>,
}

fn main() {
    let good = MyStruct {
        a: 666,
        b: vec![],
    };

    assert!(good.validate().is_ok());

    let bad = MyStruct {
        a: 1,
        b: vec![42; 25],
    };

    let result = bad.validate();
    assert!(result.is_err());

    println!("{:#?}", result);
}
```

## License

`validatron` is licensed under the MIT license; see the [LICENSE](./LICENSE) file for more details.

[1]: https://github.com/Keats/validator
