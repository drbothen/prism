//! Fixture loader for TOML and JSON test fixtures.

/// Load a fixture file by name from the given crate directory and return it as
/// a [`serde_json::Value`].
///
/// `crate_dir` should be `env!("CARGO_MANIFEST_DIR")` at the call site.
/// `name` is the fixture filename without extension (e.g. `"crowdstrike_alert"`).
///
/// # Errors
///
/// Returns an error if:
/// - `name` contains path separators or `..` (path traversal guard)
/// - the fixture file does not exist or cannot be read
/// - the file contents are not valid JSON
pub fn load_fixture(crate_dir: &str, name: &str) -> anyhow::Result<serde_json::Value> {
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(anyhow::anyhow!(
            "fixture name must not contain path separators or '..': {name}"
        ));
    }
    let path = std::path::PathBuf::from(crate_dir)
        .join("fixtures")
        .join(format!("{name}.json"));
    let contents = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("fixture file not found: {}: {e}", path.display()))?;
    serde_json::from_str(&contents)
        .map_err(|e| anyhow::anyhow!("failed to parse fixture '{}': {e}", path.display()))
}

/// Load and deserialize a fixture file into a concrete type `T`.
///
/// `crate_dir` should be `env!("CARGO_MANIFEST_DIR")` at the call site.
/// `name` is the fixture filename without extension.
///
/// # Errors
///
/// Returns an error if [`load_fixture`] fails or if the JSON cannot be
/// deserialized into `T`.
pub fn load_fixture_as<T: serde::de::DeserializeOwned>(
    crate_dir: &str,
    name: &str,
) -> anyhow::Result<T> {
    let value = load_fixture(crate_dir, name)?;
    serde_json::from_value(value)
        .map_err(|e| anyhow::anyhow!("failed to deserialize fixture '{name}': {e}"))
}
