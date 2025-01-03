use rawr::{FieldDef, Schema, SchemaDef, StructDef};

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
