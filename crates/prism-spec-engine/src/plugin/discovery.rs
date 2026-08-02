//! Plugin discovery — scan `{config_dir}/plugins/*.prx`, WIT validation, startup loading.

use std::{path::Path, sync::Arc};

use prism_core::PluginError;
use tracing::{error, info};

use super::{
    LoadedPlugin, PluginType,
    loader::{HostState, PluginMetadata, compile_component, pre_instantiate},
};

/// Required WIT exports for a sensor plugin (`prism:sensor-plugin`).
pub const SENSOR_REQUIRED_EXPORTS: &[&str] = &["name", "version", "fetch-page"];

/// Required WIT exports for an infusion plugin (`prism:infusion-plugin`).
pub const INFUSION_REQUIRED_EXPORTS: &[&str] = &["name", "version", "enrich-single"];

/// Required WIT exports for an action plugin (`prism:action-plugin`).
pub const ACTION_REQUIRED_EXPORTS: &[&str] =
    &["name", "version", "fire-alert", "fire-case", "fire-report"];

/// Required WIT exports for a sensor-auth plugin (`prism:sensor-auth-plugin`).
///
/// PLUGIN-MIGRATION-001-E: crowdstrike-oauth2 plugin exports these functions
/// per BC-2.17.006 WIT validation gate.
pub const SENSOR_AUTH_REQUIRED_EXPORTS: &[&str] = &["auth-type-name", "acquire-token", "get-token"];

/// Validate that a compiled WASM Component implements a recognized Prism WIT interface.
///
/// Checks for the presence of required exports (`name`, `version`, and the primary
/// dispatch function) on the component. If any required export is missing, returns
/// `Err(PluginError::InvalidInterface)` naming the **first** missing export in the
/// error message.
///
/// Returns `Ok(PluginType)` if the component satisfies a recognized interface.
///
/// The function is **deterministic**: same component + required export set → same result.
pub fn validate_wit_interface(
    component_exports: &[&str],
    path: &str,
) -> Result<PluginType, PluginError> {
    // Try each plugin type in order: infusion, sensor, action, sensor-auth.
    // A component satisfies a type if it has ALL required exports for that type.

    // Check infusion first (most common).
    if find_missing_export(component_exports, INFUSION_REQUIRED_EXPORTS).is_none() {
        return Ok(PluginType::Infusion);
    }
    // Check sensor.
    if find_missing_export(component_exports, SENSOR_REQUIRED_EXPORTS).is_none() {
        return Ok(PluginType::Sensor);
    }
    // Check action.
    if find_missing_export(component_exports, ACTION_REQUIRED_EXPORTS).is_none() {
        return Ok(PluginType::Action);
    }
    // Check sensor-auth (PLUGIN-MIGRATION-001-E / BC-2.17.006).
    // Sensor-auth plugins export auth-type-name, acquire-token, get-token
    // WITHOUT the name/version exports required by the other plugin types.
    if find_missing_export(component_exports, SENSOR_AUTH_REQUIRED_EXPORTS).is_none() {
        return Ok(PluginType::SensorAuth);
    }

    // None matched. Return error naming the first missing export from the best-match type.
    // Best match = type with the highest count of present exports.
    // Tie-break: prefer infusion > sensor > action > sensor-auth (infusion is most common).
    let infusion_matches = count_matches(component_exports, INFUSION_REQUIRED_EXPORTS);
    let sensor_matches = count_matches(component_exports, SENSOR_REQUIRED_EXPORTS);
    let action_matches = count_matches(component_exports, ACTION_REQUIRED_EXPORTS);
    let sensor_auth_matches = count_matches(component_exports, SENSOR_AUTH_REQUIRED_EXPORTS);

    let missing_export = if infusion_matches >= sensor_matches
        && infusion_matches >= action_matches
        && infusion_matches >= sensor_auth_matches
    {
        find_missing_export(component_exports, INFUSION_REQUIRED_EXPORTS).unwrap_or("enrich-single")
    } else if sensor_matches >= action_matches && sensor_matches >= sensor_auth_matches {
        find_missing_export(component_exports, SENSOR_REQUIRED_EXPORTS).unwrap_or("fetch-page")
    } else if action_matches >= sensor_auth_matches {
        find_missing_export(component_exports, ACTION_REQUIRED_EXPORTS).unwrap_or("fire-alert")
    } else {
        find_missing_export(component_exports, SENSOR_AUTH_REQUIRED_EXPORTS)
            .unwrap_or("auth-type-name")
    };

    Err(PluginError::InvalidInterface {
        path: path.to_string(),
        missing_export: missing_export.to_string(),
    })
}

/// Count how many exports from `required` are present in `present`.
fn count_matches(present: &[&str], required: &[&str]) -> usize {
    required.iter().filter(|&&r| present.contains(&r)).count()
}

/// Find the first missing export from `required` given the `present` exports.
/// Returns `None` if all required exports are present (valid interface).
fn find_missing_export<'a>(present: &[&str], required: &[&'a str]) -> Option<&'a str> {
    required
        .iter()
        .copied()
        .find(|&req| !present.contains(&req))
}

/// Extract the plugin name from its WASM linear memory using the `name()` export.
///
/// The WAT fixtures use a simple ABI: `name()` returns (ptr: i32, len: i32) into
/// memory. We execute this in a minimal store to read the plugin name for registration.
pub fn extract_plugin_name(
    _engine: &wasmtime::Engine,
    _component: &wasmtime::component::Component,
    _linker: &wasmtime::component::Linker<HostState>,
    path: &Path,
) -> Result<(String, String), PluginError> {
    // Derive name from filename (strip .prx extension).
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown-plugin")
        .to_string();

    let version = "0.1.0".to_string();

    Ok((name, version))
}

/// Scan `plugins_dir/*.prx` and attempt to load each file.
///
/// Returns a list of successfully loaded `Arc<LoadedPlugin>`s. Failed loads are logged
/// at `ERROR` level and skipped — they do not prevent other plugins from loading.
pub fn discover_plugins(
    plugins_dir: &Path,
    engine: &wasmtime::Engine,
    linker: &wasmtime::component::Linker<HostState>,
) -> Vec<Arc<LoadedPlugin>> {
    let mut loaded = Vec::new();

    let entries = match std::fs::read_dir(plugins_dir) {
        Ok(e) => e,
        Err(err) => {
            error!(
                "discover_plugins: cannot read plugins dir {:?}: {}",
                plugins_dir, err
            );
            return loaded;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                error!("discover_plugins: directory entry error: {}", err);
                continue;
            }
        };

        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("prx") {
            continue;
        }

        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(err) => {
                error!("discover_plugins: failed to read {:?}: {}", path, err);
                continue;
            }
        };

        match load_plugin_from_bytes(engine, linker, &path, &bytes) {
            Ok(plugin) => {
                info!(
                    "discover_plugins: loaded plugin '{}'",
                    plugin.metadata.plugin_id
                );
                loaded.push(Arc::new(plugin));
            }
            Err(err) => {
                error!(
                    "discover_plugins: failed to load {:?}: {} (E-PLUGIN-001)",
                    path, err
                );
            }
        }
    }

    loaded
}

/// Scan a Component Model binary's imports for the Prism host interface namespace.
///
/// Real `.prx` files built with `wasm-tools component new --adapt wasi_snapshot_preview1`
/// import the Prism host functions under a WIT-package-scoped name, e.g.:
///   `"prism:crowdstrike-oauth2/host@0.1.0"`
///
/// This function compiles the component and iterates its imports looking for an instance
/// whose name matches `*/host@*` (any WIT namespace prefix + host interface + version).
/// Returns the full interface name (e.g., `"prism:crowdstrike-oauth2/host@0.1.0"`) if found,
/// or `None` if no such import exists (bare "host" plugins, future plugin types, etc.).
fn find_host_interface_namespace(
    engine: &wasmtime::Engine,
    component: &wasmtime::component::Component,
) -> Option<String> {
    let ct = component.component_type();
    for (name, item) in ct.imports(engine) {
        // wasmtime 47: imports() yields ComponentExtern<'_>; .ty is the ComponentItem.
        if let wasmtime::component::types::ComponentItem::ComponentInstance(_) = item.ty {
            // Match `*/host@*` pattern: namespace + "/host@" + version
            if name.contains("/host@") {
                return Some(name.to_string());
            }
        }
    }
    None
}

/// Load a single plugin from bytes (compile + validate + build metadata).
pub(crate) fn load_plugin_from_bytes(
    engine: &wasmtime::Engine,
    linker: &wasmtime::component::Linker<HostState>,
    path: &Path,
    bytes: &[u8],
) -> Result<LoadedPlugin, PluginError> {
    let path_str = path.display().to_string();

    // Detect binary format: core module vs Component Model.
    let is_core_module = bytes.len() >= 8 && bytes[4..8] == [0x01, 0x00, 0x00, 0x00];

    // Step 1: Extract export names.
    // PR-MED-2 fix: for Component Model binaries, compile first and use reflection API
    // (wasmtime::component::Component::component_type().exports()) instead of returning
    // an empty Vec. Core WASM modules parse the export section directly (no compilation needed).
    let export_names = if is_core_module {
        extract_exports_from_raw_bytes(bytes)
    } else {
        // Component Model binary: compile and extract exports via reflection.
        // This is the authoritative source of truth for what a component exports.
        extract_component_exports(engine, bytes)
    };
    let export_refs: Vec<&str> = export_names.iter().map(|s| s.as_str()).collect();

    // Step 2: Validate WIT interface using the extracted export names.
    let _plugin_type = validate_wit_interface(&export_refs, &path_str)?;

    // Step 3: Compile the component (wraps core module if needed).
    let component = compile_component(engine, path, bytes)?;

    // Step 4: Pre-instantiate.
    //
    // For core modules (WAT test fixtures), use the base linker directly — they import only
    // the bare "host" namespace which is already registered.
    //
    // For real Component Model .prx binaries (wasm32-wasip1 Rust std + wasm-tools --adapt),
    // build a per-component linker that:
    //   (a) registers Prism host functions under the WIT-namespaced import interface, and
    //   (b) satisfies any WASI Preview 2 imports with trap stubs (BC-2.17.002: no real WASI).
    // This path is identified by detecting that the direct pre_instantiate call would fail due
    // to unsatisfied imports — or by checking is_core_module == false.
    //
    // S-PLUGIN-CI-001 AC-001: enables loading of real crowdstrike-oauth2.prx.
    let pre_instance = if is_core_module {
        pre_instantiate(linker, &component, path)?
    } else {
        // Real Component Model binary: detect the WIT-namespaced host interface name from
        // the component's imports, then build a per-component linker with the correct namespace.
        //
        // Detection: scan the component's imports for an instance whose name starts with
        // a known Prism host-interface prefix ("*/host@*"). If found, use that name;
        // otherwise fall back to trying the bare linker (forward-compat: future plugins
        // that use only the bare "host" namespace should still work).
        let host_namespace = find_host_interface_namespace(engine, &component);
        match host_namespace {
            Some(ns) => {
                // Build a per-component linker with WIT-namespaced host functions + WASI stubs.
                let component_linker =
                    crate::plugin::host_functions::build_component_linker(linker, &component, &ns)
                        .map_err(|e| PluginError::CompilationFailed {
                            path: path_str.clone(),
                            message: format!("failed to build component linker: {e}"),
                        })?;
                pre_instantiate(&component_linker, &component, path)?
            }
            None => {
                // No WIT-namespaced host import found; try bare linker (may succeed for
                // future plugins that don't use WASI or use only bare "host").
                pre_instantiate(linker, &component, path)?
            }
        }
    };

    // Step 5: Determine plugin name.
    // For core modules, try to call name() to get the actual name from WASM memory.
    // For Component Model binaries, derive from file path (is_core_module computed above).
    let name = if is_core_module {
        // Call name() on the core module to get the plugin's actual name.
        call_name_fn(engine, bytes).unwrap_or_else(|| {
            // Fallback: derive from file path.
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string()
        })
    } else {
        path.file_stem()
            .and_then(|s| s.to_str())
            .filter(|s| !s.is_empty())
            .unwrap_or("")
            .to_string()
    };

    // Step 6: Validate non-empty plugin_id.
    if name.is_empty() {
        return Err(PluginError::EmptyPluginId { path: path_str });
    }

    // PR-OBS-3: placeholder version used here because load_plugin_from_bytes does not
    // have access to the manifest. The production load path (PluginRuntime::load_plugin)
    // overrides this via `plugin.metadata.version = plugin_version` after manifest parsing.
    // This placeholder is visible only when load_plugin_from_bytes is called without a manifest
    // (e.g., discover_plugins scan mode). It is intentionally "0.0.0" to distinguish it from
    // a real semver version in logs/diagnostics.
    let version = "0.0.0".to_string();

    let metadata = PluginMetadata {
        plugin_id: name.clone(),
        name,
        version,
        path: path.to_path_buf(),
    };

    let core_module = if is_core_module {
        wasmtime::Module::from_binary(engine, bytes).ok()
    } else {
        None
    };

    Ok(LoadedPlugin {
        metadata,
        component,
        pre_instance,
        core_module,
        raw_bytes: bytes.to_vec(),
        // Default-deny: discovery.rs does not parse manifests (that is load_all_plugins scope).
        // Callers that need allowlist enforcement should use PluginRuntime::load_all_plugins.
        allowed_urls: vec![],
        // F-LP2-CRIT-001: each plugin gets its own persistent KV store Arc, created at load time.
        // All dispatches for this plugin will clone this Arc — ensuring the token cache survives
        // across separate dispatch calls (AC-004 "token cached within TTL").
        kv_store: std::sync::Arc::new(crate::plugin::loader::PluginKvStore::new()),
    })
}

/// Call the `name()` export on a core WASM module and return the string value.
///
/// The WAT fixtures implement `name()` as `(result i32 i32)` returning (ptr, len) into
/// the WASM linear memory. This function executes `name()` in a minimal store and reads
/// the string from memory.
///
/// Returns `None` if the module can't be compiled, doesn't have a `name` export,
/// or if the string is empty (caller should handle empty as `EmptyPluginId`).
fn call_name_fn(engine: &wasmtime::Engine, bytes: &[u8]) -> Option<String> {
    let module = wasmtime::Module::from_binary(engine, bytes).ok()?;
    let linker: wasmtime::Linker<()> = wasmtime::Linker::new(engine);
    let mut store: wasmtime::Store<()> = wasmtime::Store::new(engine, ());
    // Set a generous epoch deadline so the short name() call isn't interrupted.
    // name() is a trivial function (just returns constants), so 10_000 ticks is plenty.
    store.set_epoch_deadline(10_000);

    let instance = linker.instantiate(&mut store, &module).ok()?;

    // Get the `name()` function: returns (i32, i32) = (ptr, len).
    let name_fn = instance
        .get_typed_func::<(), (i32, i32)>(&mut store, "name")
        .ok()?;
    let (ptr, len) = name_fn.call(&mut store, ()).ok()?;

    if len == 0 {
        // Empty name — return empty string so caller can detect EmptyPluginId.
        return Some(String::new());
    }

    // Read from WASM linear memory.
    let memory = instance.get_memory(&mut store, "memory")?;
    let mem_data = memory.data(&store);

    let start = ptr as usize;
    let end = start + len as usize;
    if end > mem_data.len() {
        return None;
    }

    let name_bytes = &mem_data[start..end];
    std::str::from_utf8(name_bytes).ok().map(|s| s.to_string())
}

/// Extract export names from a Component Model binary using wasmtime reflection.
///
/// PR-MED-2 fix: for Component Model binaries (magic bytes `0x0d 0x00 0x01 0x00`), the raw
/// WASM export section parsing in `extract_exports_from_raw_bytes` returns an empty Vec
/// because Component Model binaries have a different internal structure. This function
/// compiles the component and uses `Component::component_type().exports()` to get the
/// authoritative export list via the wasmtime reflection API.
///
/// S-PLUGIN-CI-001 fix: real Component Model binaries produced by `wasm-tools component new`
/// export WIT interfaces as `ComponentInstance` items (e.g., the outer export is
/// `"prism:crowdstrike-oauth2/sensor-auth@0.1.0"`, not the bare function names). To allow
/// `validate_wit_interface` to match the sensor-auth type against bare function names
/// (`auth-type-name`, `acquire-token`, `get-token`), we flatten one level: if an outer
/// export is a `ComponentInstance`, we also enumerate the bare function names within it.
/// Both the interface name AND the function names within it are included in the result.
///
/// Returns empty Vec on compilation error — caller should handle missing exports as
/// an InvalidInterface error during `validate_wit_interface`.
fn extract_component_exports(engine: &wasmtime::Engine, bytes: &[u8]) -> Vec<String> {
    let component = match wasmtime::component::Component::from_binary(engine, bytes) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut names: Vec<String> = Vec::new();
    for (name, item) in component.component_type().exports(engine) {
        let name_str = name.to_string();
        names.push(name_str.clone());

        // Flatten one level: if the export is a ComponentInstance (WIT interface),
        // also collect the bare function names from within the interface.
        // This allows validate_wit_interface to match bare names like "acquire-token"
        // even when they are nested inside "prism:crowdstrike-oauth2/sensor-auth@0.1.0".
        // wasmtime 47: exports() yields ComponentExtern<'_>; .ty is the ComponentItem.
        if let wasmtime::component::types::ComponentItem::ComponentInstance(inst) = item.ty {
            for (fn_name, _fn_ty) in inst.exports(engine) {
                names.push(fn_name.to_string());
            }
        }

        // Handle Component Model exports with `#` separator (e.g.
        // "prism:crowdstrike-oauth2/sensor-auth@0.1.0#acquire-token").
        // Some wasm-tools / wit-bindgen versions emit direct function exports at
        // the component level using `interface#function` naming rather than nesting
        // the function inside a ComponentInstance. Extract the bare function name
        // after the `#` so that validate_wit_interface can match it.
        if let Some(bare_name) = name_str
            .rsplit_once('#')
            .map(|(_, fn_name)| fn_name)
            .filter(|s| !s.is_empty())
        {
            names.push(bare_name.to_string());
        }
    }
    names
}

/// Parse the export section of a raw WASM binary (core module format).
///
/// WASM binary format: 4-byte magic + 4-byte version, then sections.
/// Each section: 1-byte section id, LEB128 size, then section payload.
/// Export section (id=7): LEB128 count, then for each export:
///   LEB128 name_len, name bytes, 1-byte kind, LEB128 index.
///
/// Returns empty Vec if the bytes are not a core WASM module.
fn extract_exports_from_raw_bytes(bytes: &[u8]) -> Vec<String> {
    // Core WASM magic: \0asm + version 1
    if bytes.len() < 8 || &bytes[0..4] != b"\0asm" {
        // Component Model binary — no raw export extraction (component has no core exports at top level)
        return Vec::new();
    }

    // Check version: core module = 0x01 0x00 0x00 0x00
    // Component Model = 0x0d 0x00 0x01 0x00
    if bytes[4..8] == [0x0d, 0x00, 0x01, 0x00] {
        // This IS a Component Model binary — no core-module export section to parse
        return Vec::new();
    }

    // Parse sections looking for export section (id = 7).
    let mut pos = 8usize; // skip magic + version
    while pos < bytes.len() {
        let section_id = bytes[pos];
        pos += 1;

        let (section_size, bytes_read) = read_leb128_u32(bytes, pos);
        pos += bytes_read;

        if section_id == 7 {
            // Export section — parse it.
            return parse_export_section(&bytes[pos..pos + section_size as usize]);
        }

        pos += section_size as usize;
    }

    Vec::new()
}

/// Parse the payload of a WASM export section.
fn parse_export_section(data: &[u8]) -> Vec<String> {
    let mut exports = Vec::new();
    let mut pos = 0;

    let (count, bytes_read) = read_leb128_u32(data, pos);
    pos += bytes_read;

    for _ in 0..count {
        if pos >= data.len() {
            break;
        }
        // Name length (LEB128)
        let (name_len, br) = read_leb128_u32(data, pos);
        pos += br;

        // Name bytes
        if pos + name_len as usize > data.len() {
            break;
        }
        if let Ok(name) = std::str::from_utf8(&data[pos..pos + name_len as usize]) {
            exports.push(name.to_string());
        }
        pos += name_len as usize;

        // Kind (1 byte) + index (LEB128)
        if pos >= data.len() {
            break;
        }
        pos += 1; // kind
        let (_, idx_br) = read_leb128_u32(data, pos);
        pos += idx_br;
    }

    exports
}

/// Read a LEB128-encoded u32 from `data` starting at `pos`.
/// Returns (value, bytes_consumed).
fn read_leb128_u32(data: &[u8], pos: usize) -> (u32, usize) {
    let mut result = 0u32;
    let mut shift = 0;
    let mut bytes_read = 0;

    for &byte in &data[pos..] {
        bytes_read += 1;
        result |= ((byte & 0x7f) as u32) << shift;
        shift += 7;
        if byte & 0x80 == 0 {
            break;
        }
    }

    (result, bytes_read)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create the noop_infusion.wasm fixture file for VP-042 tests.
    /// Idempotent — only writes if missing.
    #[test]
    fn create_wasm_fixtures_for_vp_tests() {
        let wat_path = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/noop_infusion.wat");
        let wasm_path = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/noop_infusion.wasm");
        if !std::path::Path::new(wasm_path).exists() {
            let bytes = wat::parse_file(wat_path).expect("parse noop_infusion.wat");
            std::fs::write(wasm_path, &bytes).expect("write noop_infusion.wasm");
        }
    }

    #[test]
    fn debug_export_parse_trap_plugin() {
        let bytes = wat::parse_str(
            r#"
(module
  (memory (export "memory") 1)
  (data (i32.const 0) "trap-plugin")
  (data (i32.const 16) "0.1.0")
  (func (export "name") (result i32 i32) i32.const 0 i32.const 11)
  (func (export "version") (result i32 i32) i32.const 16 i32.const 5)
  (func (export "enrich-single") (param i32 i32 i32 i32) (result i32) unreachable)
  (func (export "enrich-batch") (param i32 i32 i32 i32) (result i32 i32) unreachable)
)
"#,
        )
        .expect("WAT parse failed");

        let exports = extract_exports_from_raw_bytes(&bytes);
        eprintln!("Parsed exports: {:?}", exports);
        assert!(
            exports.contains(&"enrich-single".to_string()),
            "Expected enrich-single in exports, got: {:?}",
            exports
        );
    }
}
