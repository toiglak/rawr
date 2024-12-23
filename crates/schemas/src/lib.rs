use rawr::{distributed_slice, FieldDef, HasSchema, SchemaDef, StructDef};

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

impl HasSchema for MyData {
    fn schema() -> SchemaDef {
        SchemaDef::Struct(StructDef {
            fields: vec![
                FieldDef {
                    name: "name".to_string(),
                    schema: <String as HasSchema>::schema(),
                },
                FieldDef {
                    name: "count".to_string(),
                    schema: <i32 as HasSchema>::schema(),
                },
                FieldDef {
                    name: "is_active".to_string(),
                    schema: <bool as HasSchema>::schema(),
                },
            ],
        })
    }
}

const _: () = {
    #[distributed_slice(rawr::REGISTRY)]
    static MY_DATA_SCHEMA_DEF: fn() -> SchemaDef = <MyData as HasSchema>::schema;
};
