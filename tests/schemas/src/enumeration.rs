use rawr::Schema;
use serde::{Deserialize, Serialize};

use crate::module::ImportedStruct;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Schema)]
#[serde(tag = "type", content = "data")]
pub enum EnumAdjacent {
    #[default]
    VariantA,
    VariantB(),
    VariantC(i32),
    VariantD(i32, ImportedStruct),
    VariantE {},
    VariantF {
        a: char,
        b: (i32, ImportedStruct),
        c: (char, (i32, ImportedStruct)),
    },
}
