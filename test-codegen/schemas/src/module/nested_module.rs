use rawr::{FieldDef, Schema, SchemaDef, StructDef};

use crate::enumeration::EnumAdjacent;

pub struct NestedModuleStruct {
    pub value: EnumAdjacent,
}

impl Schema for NestedModuleStruct {
    fn schema() -> SchemaDef {
        SchemaDef::Struct(StructDef {
            name: "NestedModuleStruct",
            module_path: ::core::module_path!(),
            fields: &[FieldDef {
                name: "value",
                schema: <EnumAdjacent as Schema>::schema,
            }],
        })
    }
}
