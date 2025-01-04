use std::process::Command;

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
    std::env::set_var("RUST_LOG", "info");
    env_logger::builder().format_timestamp(None).init();

    let generated_path = "test-codegen/snapshots/typescript-generated";
    let expected_path = "test-codegen/snapshots/typescript-expected";

    log::info!("Generating bindings...");
    schemas::export_to(generated_path);

    log::info!("Type-checking bindings...");

    let output = Command::new("bunx")
        .args(&["tsc", "--build", "test-codegen/snapshots/tsconfig.json"])
        .output()
        .expect("failed to execute process");

    if !output.status.success() {
        log::error!(
            "Type-checking failed with the following output:\n{}",
            String::from_utf8_lossy(&output.stdout)
        );
        panic!("Type-checking failed");
    }

    log::info!("Comparing bindings with snapshot...");
    diff::compare_directories(expected_path, generated_path).unwrap();
}
