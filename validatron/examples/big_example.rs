use validatron::{Error, Result, Validate};

use std::collections::HashMap;

fn custom_str_compare(value: &str) -> Result<()> {
    const LIT: &str = "hello world";
    if value != LIT {
        Err(Error::new(format!("'{}' does not equal '{}'", value, LIT)))
    } else {
        Ok(())
    }
}

fn main() {
    #[derive(Validate)]
    struct NestedValidateStruct {
        #[validatron(min = 14)]
        in_a: u64,
        #[validatron(min_len = 3)]
        in_b: Vec<bool>,
        #[validatron(required)]
        in_c: Option<bool>,
    }

    #[derive(Validate)]
    struct OuterValidatedStruct {
        #[validatron(min = 2., min = 3.)]
        #[validatron(max = 0., max = 2., equal = 3., equal = 1.)]
        out_a: f64,
        #[validatron]
        out_b: NestedValidateStruct,
        #[validatron(required)]
        out_c: Option<bool>,
        #[validatron]
        out_d: HashMap<&'static str, NestedValidateStruct>,
        #[validatron(function = "custom_str_compare")]
        #[validatron(equal = "fluffy")]
        out_e: &'static str,
        #[validatron(function = "custom_str_compare")]
        out_e2: String,
        #[validatron]
        out_f: Vec<NestedValidateStruct>,
    }

    let f = OuterValidatedStruct {
        out_a: 1.,
        out_b: NestedValidateStruct {
            in_a: 12,
            in_b: vec![true, true],
            in_c: None,
        },
        out_c: None,
        out_d: vec![
            (
                "a good example",
                NestedValidateStruct {
                    in_a: 12,
                    in_b: vec![],
                    in_c: Some(true),
                },
            ),
            (
                "a bad example",
                NestedValidateStruct {
                    in_a: 0,
                    in_b: vec![],
                    in_c: Some(true),
                },
            ),
        ]
        .into_iter()
        .collect(),
        out_e: "goodbye cruel world",
        out_e2: "goodbye cruel world".into(),
        out_f: vec![NestedValidateStruct {
            in_a: 0,
            in_b: vec![],
            in_c: Some(true),
        }],
    };

    let e = f.validate().unwrap_err();
    println!("{}", serde_yaml::to_string(&e).unwrap());
}
