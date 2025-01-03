use rawr::{EnumDef, EnumRepresentation, EnumVariant, FieldDef, Schema, SchemaDef};
use serde::{Deserialize, Serialize};

use crate::module::ImportedStruct;

#[derive(Debug, Serialize, Deserialize)]
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
