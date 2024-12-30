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
    // We need to invoke linker on the schemas crate so that the static variables
    // are actually "initialized" and added to the binary. Otherwise, the registry
    // will be empty and we won't generate any bindings.
    schemas::import();
    rawr::Codegen::new().export_to("snapshot/generated").run();
    diff::compare_directories("snapshot/expected", "snapshot/generated").unwrap();
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn assert_snapshot() {
        // Currently following test doesn't work, probably because Rust runs tests
        // in a special way which doesn't include the exports for some reason.
        //
        // This is fine, we should manually export the types using Codegen builder
        // methods either way. TODO: Do it later.

        // // We need to invoke linker on the schemas crate so that the static variables
        // // are actually "initialized" and added to the binary. Otherwise, the registry
        // // will be empty and we won't generate any bindings.
        // schemas::import();
        // rawr::Codegen::new().export_to("snapshot/generated").run();
        // diff::compare_directories("snapshot/expected", "snapshot/generated").unwrap();
    }
}
