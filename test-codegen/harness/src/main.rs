use rawr::{FieldDef, Schema, SchemaDef, StructDef};
use schemas::module::ImportedStruct;

mod diff;

#[allow(unused)]
#[derive(Debug, Clone)]
struct MainStruct {
    a: i32,
    b: ImportedStruct,
}

impl Schema for MainStruct {
    fn schema() -> SchemaDef {
        SchemaDef::Struct(StructDef {
            name: "MainStruct",
            module_path: ::core::module_path!(),
            fields: &[
                FieldDef {
                    name: "a",
                    schema: <i32 as Schema>::schema,
                },
                FieldDef {
                    name: "b",
                    schema: <ImportedStruct as Schema>::schema,
                },
            ],
        })
    }
}

fn main() {
    let generated_path = "test-codegen/snapshots/typescript-generated";
    let expected_path = "test-codegen/snapshots/typescript-expected";

    schemas::export_to(generated_path);
    diff::compare_directories(expected_path, generated_path).unwrap();
}
