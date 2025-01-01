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

const _: () = {
    #[linkme::distributed_slice(rawr::SCHEMA_REGISTRY)]
    static __: fn() -> SchemaDef = <MainStruct as Schema>::schema;
};

fn main() {
    // TODO: Manually export the types using Codegen builder methods.

    // We need to invoke linker on the schemas crate so that the static variables
    // are actually "initialized" and added to the binary. Otherwise, the registry
    // will be empty and we won't generate any bindings.
    schemas::import();

    let generated_path = "test-codegen/snapshots/typescript-generated";
    let expected_path = "test-codegen/snapshots/typescript-expected";

    rawr::Codegen::new().export_to(generated_path).run();
    diff::compare_directories(expected_path, generated_path).unwrap();
}
