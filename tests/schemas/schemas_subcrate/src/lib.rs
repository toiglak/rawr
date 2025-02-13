use rawr::{FieldDef, PrimitiveDef, Schema, SchemaDef, Shape, StructDef};
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
                schema: || SchemaDef::Primitive(PrimitiveDef::I32),
            }]),
        })
    }
}
