use rawr::{FieldDef, Schema, SchemaDef, StructDef};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct StructFromOtherCrate {
    pub value: i32,
}

impl Schema for StructFromOtherCrate {
    fn schema() -> SchemaDef {
        SchemaDef::Struct(StructDef {
            name: "StructFromOtherCrate",
            module_path: ::core::module_path!(),
            fields: &[FieldDef {
                name: "value",
                schema: <i32 as Schema>::schema,
            }],
        })
    }
}
