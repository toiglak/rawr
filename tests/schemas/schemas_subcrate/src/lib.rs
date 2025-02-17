use rawr::{FieldDef, PrimitiveDef, Schema, SchemaDef, SchemaPtr, Shape, StructDef};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct StructFromOtherCrate {
    pub value: i32,
}

impl Schema for StructFromOtherCrate {
    fn schema() -> SchemaDef {
        SchemaDef::Struct(StructDef {
            name: "StructFromOtherCrate",
            module_path: "schemas_subcrate",
            shape: Shape::Map(&[FieldDef {
                name: "value",
                schema: SchemaPtr(|| SchemaDef::Primitive(PrimitiveDef::I32)),
            }]),
            generic: None,
        })
    }
}
