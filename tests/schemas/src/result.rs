use rawr::Schema;
use serde::{Deserialize, Serialize};

#[derive(Schema, Debug, Serialize, Deserialize, PartialEq)]
pub struct ResultsTest<T> {
    a: Result<String, String>,
    b: Result<(String, String), (i32, u32)>,
    c: Result<T, char>,
}

impl<T> Default for ResultsTest<T> {
    fn default() -> Self {
        Self {
            a: Ok("hello".to_string()),
            b: Err((42, 42)),
            c: Err('c'),
        }
    }
}
