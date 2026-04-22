# AC-9: load_fixture returns parsed JSON for an existing fixture file

## Acceptance Criterion

Given `load_fixture("prism-dtu-crowdstrike", "devices-page1")` is called,
When the file `crates/prism-dtu-crowdstrike/fixtures/devices-page1.json` exists,
Then the function returns the parsed JSON value without error.

## Test

- File: `crates/prism-dtu-common/tests/ac_9_fixture_loader.rs`
- Function: `ac_9_load_fixture_returns_parsed_json_for_existing_file`
- Test command: `cargo test --features prism-dtu-common/dtu --test ac_9_fixture_loader`

## Implementation (excerpt)

File: `crates/prism-dtu-common/src/fixture.rs`

```rust
pub fn load_fixture(crate_dir: &str, name: &str) -> serde_json::Value {
    let path = std::path::PathBuf::from(crate_dir)
        .join("fixtures")
        .join(format!("{name}.json"));
    let contents = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("fixture file not found: {}", path.display()));
    serde_json::from_str(&contents)
        .unwrap_or_else(|e| panic!("failed to parse fixture '{}': {e}", path.display()))
}
```

## Test output

```
running 1 test
test ac_9_load_fixture_returns_parsed_json_for_existing_file ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Mapping

`load_fixture` constructs the path `{crate_dir}/fixtures/{name}.json`, reads it with `std::fs::read_to_string` (panicking with a clear message on missing file), and parses it with `serde_json`; the test writes a temp fixture containing `{"devices":[],"total":0}` and asserts `value["devices"]` and `value["total"]` are present and correct.
