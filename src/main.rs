fn main() {
    // We need to invoke linker on the schemas crate so that the static variables
    // are actually "initialized" and added to the binary. Otherwise, the registry
    // will be empty and we won't generate any bindings.
    schemas::import();

    rawr::Codegen::new().export_to("bindings").debug().run();

    println!("Bindings generation complete!");
}
