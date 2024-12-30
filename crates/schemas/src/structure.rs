use rawr::{FieldDef, Schema, SchemaDef, StructDef};

use crate::module::ImportedStruct;

pub struct MyData {
    pub name: String,
    pub count: i32,
    pub is_active: bool,
    pub imported: ImportedStruct,
    pub tuple: (char, ImportedStruct),
    pub nested_tuple: (char, (i32, ImportedStruct)),
}

impl Schema for MyData {
    fn schema() -> SchemaDef {
        SchemaDef::Struct(StructDef {
            name: "MyData",
            module_path: ::core::module_path!(),
            fields: &[
                FieldDef {
                    name: "name",
                    schema: <String as Schema>::schema,
                },
                FieldDef {
                    name: "count",
                    schema: <i32 as Schema>::schema,
                },
                FieldDef {
                    name: "is_active",
                    schema: <bool as Schema>::schema,
                },
                FieldDef {
                    name: "imported",
                    schema: <ImportedStruct as Schema>::schema,
                },
                FieldDef {
                    name: "tuple",
                    schema: <(char, ImportedStruct) as Schema>::schema,
                },
                FieldDef {
                    name: "nested_tuple",
                    schema: <(char, (i32, ImportedStruct)) as Schema>::schema,
                },
            ],
        })
    }
}

const _: () = {
    #[linkme::distributed_slice(rawr::SCHEMA_REGISTRY)]
    static __: fn() -> SchemaDef = <MyData as Schema>::schema;
};
