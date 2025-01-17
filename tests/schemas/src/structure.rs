use rawr::{FieldDef, Schema, SchemaDef, StructDef};
use schemas_subcrate::StructFromOtherCrate;
use serde::{Deserialize, Serialize};

use crate::module::{nested_module::NestedModuleStruct, ImportedStruct};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Structure {
    pub name: String,
    pub count: i32,
    pub is_active: bool,
    pub imported: ImportedStruct,
    pub tuple: (char, ImportedStruct),
    pub nested_tuple: (char, (i32, NestedModuleStruct)),
    pub crate_dependency: StructFromOtherCrate,
}

impl Schema for Structure {
    fn schema() -> SchemaDef {
        SchemaDef::Struct(StructDef {
            name: "Structure",
            module_path: ::core::module_path!(),
            fields: rawr::Fields::Named( &[
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
                    schema: <(char, (i32, NestedModuleStruct)) as Schema>::schema,
                },
                FieldDef {
                    name: "crate_dependency",
                    schema: <StructFromOtherCrate as Schema>::schema,
                },
            ]),
        })
    }
}
