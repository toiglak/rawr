use rawr::{FieldDef, Schema, SchemaDef, StructDef};

use crate::structure::MyData;

pub struct NestedModuleStruct {
    pub value: MyData,
}

impl Schema for NestedModuleStruct {
    fn schema() -> SchemaDef {
        SchemaDef::Struct(StructDef {
            name: "NestedModuleStruct",
            module_path: ::core::module_path!(),
            fields: &[FieldDef {
                name: "value",
                schema: <MyData as Schema>::schema,
            }],
        })
    }
}

const _: () = {
    #[linkme::distributed_slice(rawr::SCHEMA_REGISTRY)]
    static __: fn() -> SchemaDef = <NestedModuleStruct as Schema>::schema;
};
