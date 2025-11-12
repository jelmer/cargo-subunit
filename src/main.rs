use anyhow::{Context, Result};
use clap::Parser;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

mod json_parser;
mod subunit_writer;

use subunit_writer::SubunitWriter;

#[derive(Parser, Debug)]
#[command(
    name = "cargo-subunit",
    about = "Run Rust tests and output results in subunit format",
    bin_name = "cargo"
)]
struct Cli {
    /// Cargo subcommand name (always "subunit")
    #[arg(value_name = "subunit", hide = true)]
    _subcommand: Option<String>,

    /// List all available tests without running them
    #[command(flatten)]
    mode: Mode,

    /// Additional arguments to pass to cargo test
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    cargo_args: Vec<String>,
}

#[derive(Parser, Debug)]
#[group(multiple = false)]
struct Mode {
    /// List all available tests without running them
    #[arg(long)]
    list: bool,

    /// Load test names from a file (one per line) and run only those tests
    #[arg(long, value_name = "FILE")]
    load_list: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.mode.list {
        list_tests(&cli.cargo_args)
    } else if let Some(load_list_file) = &cli.mode.load_list {
        run_tests_from_file(load_list_file, &cli.cargo_args)
    } else {
        run_tests(&cli.cargo_args)
    }
}

/// List all available tests
fn list_tests(cargo_args: &[String]) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("test");
    cmd.args(cargo_args);
    cmd.args(["--", "--list", "--format", "terse"]);

    let output = cmd.output().context("Failed to run cargo test --list")?;

    if !output.status.success() {
        anyhow::bail!(
            "cargo test --list failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Parse and print test names
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let line = line.trim();
        // Skip empty lines and the summary line
        if line.is_empty() || line.ends_with(" tests, ") || line.contains(" benchmarks") {
            continue;
        }
        // Remove ": test" or ": bench" suffix
        if let Some(test_name) = line.strip_suffix(": test") {
            println!("{}", test_name);
        } else if let Some(test_name) = line.strip_suffix(": bench") {
            println!("{}", test_name);
        } else {
            // Fallback: print as-is
            println!("{}", line);
        }
    }

    Ok(())
}

/// Run tests specified in a file
fn run_tests_from_file(file_path: &str, cargo_args: &[String]) -> Result<()> {
    let file = std::fs::File::open(file_path)
        .context(format!("Failed to open test list file: {}", file_path))?;
    let reader = BufReader::new(file);

    let test_names: Vec<String> = reader
        .lines()
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to read test names from file")?
        .into_iter()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    if test_names.is_empty() {
        anyhow::bail!("No test names found in file: {}", file_path);
    }

    run_tests_with_filters(&test_names, cargo_args)
}

/// Run tests with optional test name filters
fn run_tests_with_filters(test_filters: &[String], cargo_args: &[String]) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("test");
    cmd.args(cargo_args);

    // Add unstable JSON output flags
    cmd.args([
        "--",
        "-Z",
        "unstable-options",
        "--format",
        "json",
        "--report-time",
    ]);

    // Add test filters
    for filter in test_filters {
        cmd.arg(filter);
    }

    // Enable unstable features on stable Rust
    cmd.env("RUSTC_BOOTSTRAP", "1");

    // Capture stdout for parsing
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::inherit());

    let mut child = cmd.spawn().context("Failed to spawn cargo test")?;
    let stdout = child.stdout.take().context("Failed to capture stdout")?;
    let reader = BufReader::new(stdout);

    let mut writer = SubunitWriter::new(std::io::stdout());

    // Process JSON events line by line
    for line in reader.lines() {
        let line = line.context("Failed to read line from cargo test output")?;

        if line.trim().is_empty() {
            continue;
        }

        match json_parser::parse_event(&line) {
            Ok(Some(event)) => {
                writer.write_event(&event)?;
            }
            Ok(None) => {
                // Non-test event, skip
            }
            Err(e) => {
                eprintln!("Warning: Failed to parse JSON line: {}", e);
                eprintln!("Line: {}", line);
            }
        }
    }

    let status = child.wait().context("Failed to wait for cargo test")?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

/// Run all tests
fn run_tests(cargo_args: &[String]) -> Result<()> {
    run_tests_with_filters(&[], cargo_args)
}
