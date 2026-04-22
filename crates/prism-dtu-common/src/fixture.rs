//! Fixture loader for TOML and JSON test fixtures.

/// Load a fixture file by name from the given crate directory and return it as
/// a [`serde_json::Value`].
///
/// `crate_dir` should be `env!("CARGO_MANIFEST_DIR")` at the call site.
/// `name` is the fixture filename without extension (e.g. `"crowdstrike_alert"`).
pub fn load_fixture(crate_dir: &str, name: &str) -> serde_json::Value {
    todo!("implement fixture loader per AC-6")
}

/// Load and deserialize a fixture file into a concrete type `T`.
///
/// `crate_dir` should be `env!("CARGO_MANIFEST_DIR")` at the call site.
/// `name` is the fixture filename without extension.
pub fn load_fixture_as<T: serde::de::DeserializeOwned>(crate_dir: &str, name: &str) -> T {
    todo!("implement typed fixture loader per AC-6")
}
