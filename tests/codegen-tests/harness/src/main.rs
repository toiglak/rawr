use std::process::Command;

mod diff;

fn main() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::builder().init();

    let snapshots_path = &format!("{}/../snapshots", env!("CARGO_MANIFEST_DIR"));
    let generated_path = &format!("{snapshots_path}/typescript-generated");
    let expected_path = &format!("{snapshots_path}/typescript-expected");
    let tsconfig_path = &format!("{snapshots_path}/tsconfig.json");

    log::info!("Generating bindings...");
    schemas::export_to(generated_path);

    log::info!("Type-checking bindings...");

    let output = Command::new("bunx")
        .args(&["tsc", "--build", tsconfig_path])
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
