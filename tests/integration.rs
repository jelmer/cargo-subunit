use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

#[test]
fn test_list_tests() {
    // Run cargo-subunit --list on this project itself
    let output = Command::new("cargo")
        .args(&["run", "--", "--list"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run cargo-subunit --list");

    assert!(output.status.success(), "cargo-subunit --list failed");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should list some tests (at least the ones in json_parser and subunit_writer)
    assert!(
        stdout.contains("json_parser::tests::test_parse_suite_started")
            || stdout.contains("test_parse_suite_started"),
        "Expected to find test names in list output"
    );
}

#[test]
fn test_run_tests_with_subunit_output() {
    // Run cargo-subunit on this project itself
    let output = Command::new("cargo")
        .args(&["run", "--"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run cargo-subunit");

    // The command might fail if tests fail, but we should get some output
    let stdout = output.stdout;

    // Subunit v2 packets start with signature 0xb3
    assert!(!stdout.is_empty(), "Expected subunit output");
    assert_eq!(stdout[0], 0xb3, "Expected subunit v2 signature byte");
}

#[test]
fn test_load_list_functionality() {
    // Create a temporary file with a test name
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    writeln!(temp_file, "json_parser::tests::test_parse_suite_started")
        .expect("Failed to write to temp file");

    let temp_path = temp_file.path().to_str().unwrap();

    // Run cargo-subunit with --load-list
    let output = Command::new("cargo")
        .args(&["run", "--", "--load-list", temp_path])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run cargo-subunit --load-list");

    // Should succeed
    assert!(output.status.success(), "cargo-subunit --load-list failed");

    let stdout = output.stdout;

    // Should have subunit output
    assert!(!stdout.is_empty(), "Expected subunit output");
    assert_eq!(stdout[0], 0xb3, "Expected subunit v2 signature byte");
}
