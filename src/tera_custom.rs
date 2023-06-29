use tera::{Result, Value, to_value, Function, from_value};

pub mod testers {
    use super::*;
    pub fn my_odd() -> Box<dyn Fn(Option<&Value>, &[Value]) -> Result<bool> + Send + Sync> {
        Box::new(|val: Option<&Value>, _: &[Value]| -> Result<bool> {
            match val {
                Some(val) => {
                    match from_value::<i64>(val.clone()) {
                        Ok(v) => Ok(v % 2 != 0),
                        Err(_) => Ok(false),
                    }
                }
                None => Ok(false),
            }
        })
    }
}

// pub mod functions {
//     use super::*;
// }

// pub mod filters {
//     use super::*;
// }