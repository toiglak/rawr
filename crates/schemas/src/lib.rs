use module::ImportedStruct;
use rawr::{EnumDef, EnumRepresentation, EnumVariant, FieldDef, Schema, SchemaDef, StructDef};
use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum EnumAdjacent {
    VariantA,
    VariantB(),
    VariantC(i32),
    VariantD(i32, ImportedStruct),
    VariantE {},
    VariantF {
        a: char,
        b: (i32, ImportedStruct),
        c: (char, (i32, ImportedStruct)),
    },
}

impl Schema for EnumAdjacent {
    fn schema() -> SchemaDef {
        SchemaDef::Enum(EnumDef {
            name: "EnumAdjacent",
            module_path: ::core::module_path!(),
            representation: EnumRepresentation::Adjacent {
                tag: "type",
                content: "data",
            },
            variants: &[
                EnumVariant::Unit { name: "VariantA" },
                EnumVariant::Tuple {
                    name: "VariantB",
                    fields: &[],
                },
                EnumVariant::Tuple {
                    name: "VariantC",
                    fields: &[<i32 as Schema>::schema],
                },
                EnumVariant::Tuple {
                    name: "VariantD",
                    fields: &[<i32 as Schema>::schema, <ImportedStruct as Schema>::schema],
                },
                EnumVariant::Struct {
                    name: "VariantE",
                    fields: &[],
                },
                EnumVariant::Struct {
                    name: "VariantF",
                    fields: &[
                        FieldDef {
                            name: "a",
                            schema: <char as Schema>::schema,
                        },
                        FieldDef {
                            name: "b",
                            schema: <(i32, ImportedStruct) as Schema>::schema,
                        },
                        FieldDef {
                            name: "c",
                            schema: <(char, (i32, ImportedStruct)) as Schema>::schema,
                        },
                    ],
                },
            ],
        })
    }
}

const _: () = {
    #[linkme::distributed_slice(rawr::SCHEMA_REGISTRY)]
    static __: fn() -> SchemaDef = <EnumAdjacent as Schema>::schema;
};
