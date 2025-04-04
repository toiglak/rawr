fn main() {
    let service_tests_path = format!("{}/..", env!("CARGO_MANIFEST_DIR")).leak();
    let typescript_bindings_path = format!("{service_tests_path}/typescript-bindings").leak();

    println!("Generating TypeScript bindings...");
    schemas::export_to(typescript_bindings_path);
}
