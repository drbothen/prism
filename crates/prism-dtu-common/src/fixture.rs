//! Fixture loader for TOML and JSON test fixtures.

/// Load a fixture file by name from the given crate directory and return it as
/// a [`serde_json::Value`].
///
/// `crate_dir` should be `env!("CARGO_MANIFEST_DIR")` at the call site.
/// `name` is the fixture filename without extension (e.g. `"crowdstrike_alert"`).
pub fn load_fixture(crate_dir: &str, name: &str) -> serde_json::Value {
    let path = std::path::PathBuf::from(crate_dir)
        .join("fixtures")
        .join(format!("{name}.json"));
    let contents = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("fixture file not found: {}", path.display()));
    serde_json::from_str(&contents)
        .unwrap_or_else(|e| panic!("failed to parse fixture '{}': {e}", path.display()))
}

/// Load and deserialize a fixture file into a concrete type `T`.
///
/// `crate_dir` should be `env!("CARGO_MANIFEST_DIR")` at the call site.
/// `name` is the fixture filename without extension.
pub fn load_fixture_as<T: serde::de::DeserializeOwned>(crate_dir: &str, name: &str) -> T {
    let value = load_fixture(crate_dir, name);
    serde_json::from_value(value)
        .unwrap_or_else(|e| panic!("failed to deserialize fixture '{name}': {e}"))
}
