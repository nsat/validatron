use validatron::{Error, Result, Validate};

fn main() {
    #[derive(Validate)]
    #[validatron(function = "custom_compare")]
    struct Comp {
        #[validatron(min = 5)]
        a: u64,
        b: u64,
    }

    fn custom_compare(value: &Comp) -> Result<()> {
        if value.a < value.b {
            Ok(())
        } else {
            Err(Error::new("the following is not true: .a < .b".to_string()))
        }
    }

    assert!(Comp { a: 5, b: 6 }.validate().is_ok());

    let e = Comp { a: 4, b: 2 }.validate().unwrap_err();
    println!("{}", serde_yaml::to_string(&e).unwrap());
}
