use rawr::Schema;
use serde::{Deserialize, Serialize};

use crate::module::ImportedStruct;

// Comments show how the JSON representation of each variant looks like.
// FIXME: It actually shows the JSON for externally tagged enums, not adjacently tagged.

#[derive(Debug, Schema, Default, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum EnumAdjacentlyTagged {
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
