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

/// Parse a fixture that was embedded at compile time with [`include_str!`].
///
/// This is the relocatable counterpart to [`load_fixture`]. `load_fixture`
/// resolves `CARGO_MANIFEST_DIR` at *runtime*, so a binary built on one machine
/// looks for fixtures at that machine's absolute path and fails to construct its
/// clones anywhere else. Embedding the bytes with `include_str!` at the call site
/// and parsing them here keeps the fixture in the binary, which is what lets a
/// clone run from a container or any checkout-free host.
///
/// `name` is used only for the panic message.
///
/// # Panics
///
/// Panics if `raw` is not valid JSON. The fixture is compiled in, so a parse
/// failure means a corrupt build artifact rather than a runtime condition, and
/// failing at startup is the correct behaviour.
#[must_use]
#[allow(clippy::expect_used)]
pub fn embedded_fixture(raw: &str, name: &str) -> serde_json::Value {
    serde_json::from_str(raw)
        .unwrap_or_else(|e| panic!("embedded fixture '{name}' is not valid JSON: {e}"))
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
