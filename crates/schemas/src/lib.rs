use module::ImportedStruct;
use rawr::{FieldDef, Schema, SchemaDef, StructDef};

pub mod module;

pub fn import() {
    // Dummy function to force the compiler to include static variables from this
    // crate in the final binary, thus allowing the types marked for export to be
    // registered.

    // It seems like this function gets optimized out when running in release mode.
    // This fixes that problem.
    std::hint::black_box(());
}

pub struct MyData {
    pub name: String,
    pub count: i32,
    pub is_active: bool,
    pub imported: ImportedStruct,
    pub tuple_field: (i32, String),
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
                    name: "tuple_field",
                    schema: <(i32, String) as Schema>::schema,
                },
            ],
        })
    }
}

const _: () = {
    #[linkme::distributed_slice(rawr::SCHEMA_REGISTRY)]
    static __: fn() -> SchemaDef = <MyData as Schema>::schema;
};
