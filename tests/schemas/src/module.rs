use rawr::Schema;
use serde::{Deserialize, Serialize};

pub mod nested_module;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Schema)]
pub struct ImportedStruct {
    pub value: String,
}
