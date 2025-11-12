# cargo-subunit

A Cargo extension that runs Rust tests and outputs results in [subunit v2](https://github.com/testing-cabal/subunit) format.

## Features

- **List tests**: Display all available tests with unique namespaces
- **Run tests**: Execute tests and stream results in subunit format
- **Load test list**: Run specific tests from a file
- **Standard cargo test args**: Forward any cargo test arguments

## Installation

```bash
cargo install --path .
```

## Usage

### List all available tests

```bash
cargo subunit --list
```

This outputs test names in their fully-qualified format (e.g., `module::submodule::test_name`), one per line.

### Run all tests with subunit output

```bash
cargo subunit
```

The subunit v2 binary output is written to stdout. You can redirect it to a file:

```bash
cargo subunit > results.subunit
```

### Run specific tests

Pass test filters just like with `cargo test`:

```bash
cargo subunit test_name
cargo subunit module::
```

### Run tests from a file

Create a file with test names (one per line):

```bash
echo "module::test_one" > tests.txt
echo "module::test_two" >> tests.txt
cargo subunit --load-list tests.txt
```

### Pass additional cargo test arguments

Any arguments after `--` are forwarded to cargo test:

```bash
cargo subunit -- --nocapture
cargo subunit -- --test-threads=1
```

## Integration with testrepository

cargo-subunit integrates seamlessly with [testrepository](https://testrepository.readthedocs.io/) for tracking test history and running tests efficiently.

Create a `.testr.conf` file in your project root:

```ini
[DEFAULT]
test_command=cargo subunit $LISTOPT $IDOPTION
test_id_option=--load-list $IDFILE
test_list_option=--list
```

Then you can use testrepository commands:

```bash
# Run all tests and record results
testr run

# Run only failed tests from the last run
testr run --failing

# List test runs
testr last

# Show results from the last run
testr last --subunit | subunit-stats
```

## How it works

cargo-subunit uses cargo test's unstable JSON output format (enabled via `RUSTC_BOOTSTRAP=1`) to capture test events, then converts them to subunit v2 format. Each test generates:

1. An `inprogress` event when the test starts
2. A `success`, `fail`, `skip`, or `fail` (timeout) event when complete
3. For failures, stdout/stderr are attached as file content

## Requirements

- Rust 2021 edition or later
- The `subunit` crate for protocol serialization

## License

Apache-2.0
