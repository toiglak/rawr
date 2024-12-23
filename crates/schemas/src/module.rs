use rawr::{FieldDef, Schema, SchemaDef, StructDef};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImportedStruct {
    pub value: String,
}

impl Schema for ImportedStruct {
    fn schema() -> SchemaDef {
        SchemaDef::Struct(StructDef {
            name: "FieldDefinition",
            fields: vec![FieldDef {
                name: "value",
                schema: <String as Schema>::schema(),
            }],
        })
    }
}

const _: () = {
    #[linkme::distributed_slice(rawr::SCHEMA_REGISTRY)]
    static __: fn() -> SchemaDef = <ImportedStruct as Schema>::schema;
};
