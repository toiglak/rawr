use std::fs::File;
use std::io::Write;

fn main() {
    // Normally it would be handled by the schemas::export() function...
    let registry = schemas::export();

    // But for now, let's visualize this by writing the registry to a file.
    let serialized_registry =
        serde_json::to_string(&*registry).expect("Failed to serialize registry");
    let mut file = File::create("registry.json").expect("Failed to create file");
    file.write_all(serialized_registry.as_bytes())
        .expect("Failed to write to file");
}
