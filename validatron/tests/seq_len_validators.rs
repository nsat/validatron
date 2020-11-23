use validatron::Validate;

#[test]
fn test_min_seq_len() {
    #[derive(Validate)]
    struct Foo {
        #[validatron(min_len = 5)]
        a: Vec<i32>,
    }

    let f = Foo { a: vec![] };
    assert!(f.validate().is_err());

    let f = Foo {
        a: vec![1, 2, 3, 4],
    };
    assert!(f.validate().is_err());

    let f = Foo {
        a: vec![1, 2, 3, 4, 5],
    };
    assert!(f.validate().is_ok());
}

#[test]
fn test_max_seq_len() {
    #[derive(Validate)]
    struct Foo {
        #[validatron(max_len = 5)]
        a: Vec<i32>,
    }

    let f = Foo { a: vec![] };
    assert!(f.validate().is_ok());

    let f = Foo {
        a: vec![1, 2, 3, 4, 5],
    };
    assert!(f.validate().is_ok());

    let f = Foo {
        a: vec![1, 2, 3, 4, 5, 6],
    };
    assert!(f.validate().is_err());
}

#[test]
fn test_min_seq_len_map() {
    use std::collections::HashMap;

    #[derive(Validate)]
    struct Foo {
        #[validatron(min_len = 2)]
        a: HashMap<&'static str, i32>,
    }

    let f = Foo { a: HashMap::new() };
    assert!(f.validate().is_err());

    let f = Foo {
        a: vec![("hello", 1), ("world", 42)].into_iter().collect(),
    };
    assert!(f.validate().is_ok());
}
