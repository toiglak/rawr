use rawr::Schema;
use serde::{Deserialize, Serialize};

use crate::module::ImportedStruct;

// 1. In this file we check if `ImportedStruct` is getting imported, even though the
// type (in this case ImportedStruct) is deeply nested.

#[derive(Debug, Default, Schema, Serialize, Deserialize, PartialEq)]
pub struct ArrayLike((Vec<String>, [i32; 3], Vec<Vec<ImportedStruct>>));

// 2. We also check if tuple inside of tuple struct (when it's the only argument)
// is correctly inlined (just like serde does it when serializing to json). In
// other words, `ArrayLike` should have the same generated schema as `TupleStruct`
// in typescript.

#[derive(Debug, Default, Schema, Serialize, Deserialize, PartialEq)]
pub struct TupleStruct(Vec<String>, [i32; 3], Vec<Vec<ImportedStruct>>);
