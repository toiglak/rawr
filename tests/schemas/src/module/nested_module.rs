use rawr::Schema;
use serde::{Deserialize, Serialize};

use crate::enumeration::EnumAdjacent;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Schema)]
pub struct NestedModuleStruct {
    pub value: EnumAdjacent,
}
