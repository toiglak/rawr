use rawr::Schema;
use serde::{Deserialize, Serialize};

use crate::module::ImportedStruct;

#[derive(Debug, Clone, Schema, Default, Serialize, Deserialize, PartialEq)]
pub struct TestEnums {
    pub external: EnumExternallyTagged,
    pub adjecent: EnumAdjacentlyTagged,
}

#[derive(Debug, Clone, Schema, Default, Serialize, Deserialize, PartialEq)]
pub enum EnumExternallyTagged {
    // "VariantA"
    #[default]
    VariantA,
    // {"VariantB":[]}
    VariantB(),
    // {"VariantC":0}
    VariantC(i32),
    // {"VariantD":null}
    VariantD(()),
    // {"VariantE":{"value":"string"}}
    VariantE(ImportedStruct),
    // {"VariantF":[0,{"value":"string"}]}
    VariantF((i32, ImportedStruct)),
    // {"VariantG":[0,{"value":"string"}]}
    VariantG(i32, ImportedStruct),
    // {"VariantH":{}}
    VariantH {},
    // {"VariantI":{"a":0,"b":{"value":"string"}}}
    VariantI {
        a: i32,
        b: ImportedStruct,
    },
}

#[derive(Debug, Clone, Schema, Default, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum EnumAdjacentlyTagged {
    #[default]
    VariantA,
    VariantB(),
    VariantC(i32),
    VariantD(()),
    VariantE(ImportedStruct),
    VariantF((i32, ImportedStruct)),
    VariantG(i32, ImportedStruct),
    VariantH {},
    VariantI {
        a: i32,
        b: ImportedStruct,
    },
}
