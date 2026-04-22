//! Plugin discovery — scan `{config_dir}/plugins/*.prx`, WIT validation, startup loading.
//!
//! On `PluginRuntime` construction, scans the plugins directory and attempts to load
//! every `.prx` file. Valid plugins are registered; invalid ones are logged and skipped.
//! A single bad plugin does NOT block other plugins from loading (EC-17-026).
//!
//! WIT validation checks that the component exports `name()`, `version()`, and the
//! primary dispatch function for one of the three recognised Prism plugin types.
//!
//! # Red Gate stubs (S-1.15)
//! All functions are `unimplemented!()`.

use std::path::Path;
use std::sync::Arc;

use prism_core::PluginError;

use super::{LoadedPlugin, PluginType};
use super::loader::HostState;

/// Required WIT exports for a sensor plugin (`prism:sensor-plugin`).
pub const SENSOR_REQUIRED_EXPORTS: &[&str] = &["name", "version", "fetch-page"];

/// Required WIT exports for an infusion plugin (`prism:infusion-plugin`).
pub const INFUSION_REQUIRED_EXPORTS: &[&str] = &["name", "version", "enrich-single"];

/// Required WIT exports for an action plugin (`prism:action-plugin`).
pub const ACTION_REQUIRED_EXPORTS: &[&str] = &[
    "name",
    "version",
    "fire-alert",
    "fire-case",
    "fire-report",
];

/// Validate that a compiled WASM Component implements a recognized Prism WIT interface.
///
/// Checks for the presence of required exports (`name`, `version`, and the primary
/// dispatch function) on the component. If any required export is missing, returns
/// `Err(PluginError::InvalidInterface)` naming the **first** missing export in the
/// error message.
///
/// This is the target function for VP-043 (proptest):
/// - For any component with a strict subset of required exports → `Err(InvalidInterface)`
///   naming the missing export.
/// - For a component with all required exports → `Ok(PluginType)`.
///
/// The function is **deterministic**: same component + required export set → same result.
///
/// `component_exports` is a slice of export names present on the component. In
/// production this comes from introspecting the wasmtime `Component`; in tests and
/// VP-043 this is driven by a synthetic mock.
pub fn validate_wit_interface(
    component_exports: &[&str],
    path: &str,
) -> Result<PluginType, PluginError> {
    unimplemented!("S-1.15 Red Gate: validate_wit_interface not yet implemented")
}

/// Scan `plugins_dir/*.prx` and attempt to load each file via `PluginRuntime::load_plugin`.
///
/// Returns a list of successfully loaded `Arc<LoadedPlugin>`s. Failed loads are logged
/// at `ERROR` level and skipped — they do not prevent other plugins from loading.
///
/// Called once at `PluginRuntime::new()` startup.
pub fn discover_plugins(
    plugins_dir: &Path,
    engine: &wasmtime::Engine,
    linker: &wasmtime::component::Linker<HostState>,
) -> Vec<Arc<LoadedPlugin>> {
    unimplemented!("S-1.15 Red Gate: discover_plugins not yet implemented")
}
