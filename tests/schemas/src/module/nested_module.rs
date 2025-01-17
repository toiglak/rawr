use rawr::{FieldDef, Schema, SchemaDef, StructDef};
use serde::{Deserialize, Serialize};

use crate::enumeration::EnumAdjacent;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct NestedModuleStruct {
    pub value: EnumAdjacent,
}

impl Schema for NestedModuleStruct {
    fn schema() -> SchemaDef {
        SchemaDef::Struct(StructDef {
            name: "NestedModuleStruct",
            module_path: ::core::module_path!(),
            fields: rawr::Fields::Named(&[FieldDef {
                name: "value",
                schema: <EnumAdjacent as Schema>::schema,
            }]),
        })
    }
}
