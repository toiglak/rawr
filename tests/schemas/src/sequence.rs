use rawr::Schema;
use serde::{Deserialize, Serialize};

use crate::module::ImportedStruct;

#[derive(Debug, Default, Schema, Serialize, Deserialize, PartialEq)]
pub struct SequenceTypes((Vec<String>, [i32; 3], Vec<Vec<ImportedStruct>>));
