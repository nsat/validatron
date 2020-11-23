use validatron::{Result, Validate};

#[derive(Debug, Validate, Clone, Copy)]
struct Bar {
    #[validatron(equal = true)]
    a: bool,
}

impl Bar {
    fn new(a: bool) -> Self {
        Self { a }
    }
}

fn is_bar_good(a: &Bar) -> Result<()> {
    a.validate()
}
fn is_2bar_good(a: &Bar, b: &Bar) -> Result<()> {
    let first = a.validate();
    let second = b.validate();

    match (first, second) {
        (Err(mut e1), Err(e2)) => {
            e1.merge(e2);
            Err(e1)
        }
        (_, Err(e2)) => Err(e2),
        (Err(e1), _) => Err(e1),
        _ => Ok(()),
    }
}

#[derive(Debug, Validate)]
enum Foo {
    #[validatron]
    Unit,
    #[validatron]
    #[validatron(function = "is_bar_good")]
    NewType(Bar),
    #[validatron]
    #[validatron(function = "is_2bar_good")]
    Tuple(Bar, Bar),
    #[validatron]
    #[validatron(function = "is_2bar_good")]
    Struct { a: Bar, b: Bar },
}

fn main() {
    let _good = Bar::new(true);
    let bad = Bar::new(false);

    let f = vec![
        Foo::Unit,
        Foo::NewType(bad),
        Foo::Tuple(bad, bad),
        Foo::Struct { a: bad, b: bad },
    ];

    if let Err(e) = f.validate() {
        println!("{}", serde_yaml::to_string(&e).unwrap());
    }
}
