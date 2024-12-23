use rawr::{distributed_slice, FieldDef, Schema, SchemaDef, StructDef};

pub fn import() {
    // Dummy function to force the compiler to include the static variables from
    // this crate in the binary, thus allowing the types marked for export to be
    // registered.

    // NOTE: It seems like this function gets optimized out when running in release
    // mode, thus statics in this crate are not included in the final binary.
    //
    // This fixes that problem.
    std::hint::black_box(());
}

pub struct MyData {
    pub name: String,
    pub count: i32,
    pub is_active: bool,
}

impl Schema for MyData {
    fn schema() -> SchemaDef {
        SchemaDef::Struct(StructDef {
            name: "MyData",
            fields: vec![
                FieldDef {
                    name: "name",
                    schema: <String as Schema>::schema(),
                },
                FieldDef {
                    name: "count",
                    schema: <i32 as Schema>::schema(),
                },
                FieldDef {
                    name: "is_active",
                    schema: <bool as Schema>::schema(),
                },
            ],
        })
    }
}

const _: () = {
    #[distributed_slice(rawr::SCHEMA_REGISTRY)]
    static MY_DATA_SCHEMA_DEF: fn() -> SchemaDef = <MyData as Schema>::schema;
};
