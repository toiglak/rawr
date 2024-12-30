use colored::Colorize;
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

pub fn compare_directories(expected_dir: &str, actual_dir: &str) -> Result<(), &'static str> {
    let mut matching = true;

    let mut extra_files = Vec::new();
    let mut missing_files = Vec::new();

    // Find missing files or files with differences.
    for entry in WalkDir::new(expected_dir) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            let relative_path = entry.path().strip_prefix(expected_dir).unwrap();
            let expected_file_path = entry.path().to_string_lossy().to_string();
            let generated_file_path = format!("{}/{}", actual_dir, relative_path.display());

            // Diff using git.
            if Path::new(&generated_file_path).exists() {
                let output = Command::new("git")
                    .args([
                        "diff",
                        "--no-index",
                        "--color=always",
                        &expected_file_path,
                        &generated_file_path,
                    ])
                    .output()
                    .expect("Failed to execute git diff");
                if !output.stdout.is_empty() {
                    matching = false;
                    let diff_output = String::from_utf8_lossy(&output.stdout);
                    let trimmed_diff = diff_output
                        .lines()
                        .skip(2)
                        .collect::<Vec<&str>>()
                        .join("\n");
                    println!("{}", trimmed_diff);
                }
            } else {
                matching = false;
                missing_files.push(expected_file_path);
            }
        }
    }

    // Find "extra" files.
    for entry in WalkDir::new(actual_dir) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            let relative_path = entry.path().strip_prefix(actual_dir).unwrap();
            let generated_file_path = entry.path().to_string_lossy().to_string();
            let expected_file_path = format!("{}/{}", expected_dir, relative_path.display());

            if !Path::new(&expected_file_path).exists() {
                matching = false;
                extra_files.push(generated_file_path);
            }
        }
    }

    if !missing_files.is_empty() || !extra_files.is_empty() {
        println!();

        for file in missing_files {
            println!("{}", format!("- {}", file).red(),);
        }
        for file in extra_files {
            println!("{}", format!("+ {}", file).green(),);
        }
    }

    println!();

    match matching {
        true => Ok(()),
        false => Err("Directories do not match"),
    }
}
