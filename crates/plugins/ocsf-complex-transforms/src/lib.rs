//! OCSF complex-transform WASM plugin scaffold.
//!
//! # AC-008 — PLUGIN-MIGRATION-001-C
//!
//! This crate provides stub WIT export implementations for the `ocsf-transform` interface.
//! The stubs compile (returning `todo!()`) but are not yet wired to a call site in
//! `SpecDrivenMapper` — the `call_ocsf_transform` WIT dispatch interface will be
//! implemented in the story that expands WASM dispatch beyond the deferred path
//! (`ocsf.wasm_dispatch_deferred` in BC-2.16.002 row 44).
//!
//! # Deployment
//!
//! Compile to `.prx` artifact:
//! ```sh
//! cargo build --manifest-path crates/plugins/ocsf-complex-transforms/Cargo.toml \
//!   --target wasm32-wasip1 --release
//! cp target/wasm32-wasip1/release/ocsf_complex_transforms.wasm \
//!   .prism/plugins/ocsf-complex-transforms.prx
//! ```
//!
//! # WIT Contract
//!
//! The plugin implements the `ocsf-transform` interface (to be defined in a future
//! WIT file when the `call_ocsf_transform` API is stabilized on `PluginRuntime`).
//! Until then, this crate serves as the scaffold that proves the plugin structure
//! and the Cargo.toml `cdylib` configuration are correct.

/// Transform a raw JSON column value into an OCSF-normalized representation.
///
/// # Parameters
///
/// - `column_name`: the sensor TOML column name (e.g., `"behaviors"`).
/// - `ocsf_field`: the target OCSF field path (e.g., `"attacks"`).
/// - `raw_json`: the raw vendor JSON value as a UTF-8 string.
///
/// # Returns
///
/// The normalized OCSF JSON value as a UTF-8 string, or an error message string
/// if normalization fails.
///
/// # Status
///
/// STUB — not yet wired to `SpecDrivenMapper`. Full implementation pending
/// stabilization of the `call_ocsf_transform` WIT interface on `PluginRuntime`.
pub fn ocsf_transform(
    _column_name: &str,
    _ocsf_field: &str,
    _raw_json: &str,
) -> Result<String, String> {
    todo!(
        "ocsf_transform stub — full implementation pending WIT interface stabilization \
         (AC-008, PLUGIN-MIGRATION-001-C)"
    )
}
