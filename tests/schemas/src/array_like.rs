use rawr::Schema;
use serde::{Deserialize, Serialize};

use crate::module::ImportedStruct;

// In this file we check if `ImportedStruct` is getting imported, even though the
// type (in this case ImportedStruct) is deeply nested.

#[derive(Debug, Default, Schema, Serialize, Deserialize, PartialEq)]
pub struct ArrayLike((Vec<String>, [i32; 3], Vec<Vec<ImportedStruct>>));
