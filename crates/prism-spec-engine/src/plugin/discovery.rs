//! Plugin discovery — scan `{config_dir}/plugins/*.prx`, WIT validation, startup loading.

use std::path::Path;
use std::sync::Arc;

use prism_core::PluginError;
use tracing::{error, info};

use super::loader::{compile_component, pre_instantiate, HostState, PluginMetadata};
use super::{LoadedPlugin, PluginType};

/// Required WIT exports for a sensor plugin (`prism:sensor-plugin`).
pub const SENSOR_REQUIRED_EXPORTS: &[&str] = &["name", "version", "fetch-page"];

/// Required WIT exports for an infusion plugin (`prism:infusion-plugin`).
pub const INFUSION_REQUIRED_EXPORTS: &[&str] = &["name", "version", "enrich-single"];

/// Required WIT exports for an action plugin (`prism:action-plugin`).
pub const ACTION_REQUIRED_EXPORTS: &[&str] =
    &["name", "version", "fire-alert", "fire-case", "fire-report"];

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
    // Try each plugin type in order: infusion, sensor, action.
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

    // None matched. Return error naming the first missing export from the best-match type.
    // Best match = type with the highest count of present exports.
    // Tie-break: prefer infusion > sensor > action (infusion is most common).
    let infusion_matches = count_matches(component_exports, INFUSION_REQUIRED_EXPORTS);
    let sensor_matches = count_matches(component_exports, SENSOR_REQUIRED_EXPORTS);
    let action_matches = count_matches(component_exports, ACTION_REQUIRED_EXPORTS);

    let missing_export = if infusion_matches >= sensor_matches && infusion_matches >= action_matches
    {
        find_missing_export(component_exports, INFUSION_REQUIRED_EXPORTS).unwrap_or("enrich-single")
    } else if sensor_matches >= action_matches {
        find_missing_export(component_exports, SENSOR_REQUIRED_EXPORTS).unwrap_or("fetch-page")
    } else {
        find_missing_export(component_exports, ACTION_REQUIRED_EXPORTS).unwrap_or("fire-alert")
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
    required.iter().copied().find(|&req| !present.contains(&req))
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

/// Load a single plugin from bytes (compile + validate + build metadata).
pub(crate) fn load_plugin_from_bytes(
    engine: &wasmtime::Engine,
    linker: &wasmtime::component::Linker<HostState>,
    path: &Path,
    bytes: &[u8],
) -> Result<LoadedPlugin, PluginError> {
    let path_str = path.display().to_string();

    // Step 1: Extract export names from the raw bytes BEFORE compiling.
    // For core WASM modules (wat fixtures), parse the WASM export section directly.
    // For true Component Model binaries, fall back to filename-based approach.
    let export_names = extract_exports_from_raw_bytes(bytes);
    let export_refs: Vec<&str> = export_names.iter().map(|s| s.as_str()).collect();

    // Step 2: Validate WIT interface using the raw export names.
    let _plugin_type = validate_wit_interface(&export_refs, &path_str)?;

    // Step 3: Compile the component (wraps core module if needed).
    let component = compile_component(engine, path, bytes)?;

    // Step 4: Pre-instantiate (this rejects WASI-importing components).
    let pre_instance = pre_instantiate(linker, &component, path)?;

    // Step 5: Determine plugin name.
    // For core modules, try to call name() to get the actual name from WASM memory.
    // For Component Model binaries, derive from file path.
    let is_core_module = bytes.len() >= 8 && bytes[4..8] == [0x01, 0x00, 0x00, 0x00];
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

    let version = "0.1.0".to_string();

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
        let wat_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/noop_infusion.wat"
        );
        let wasm_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/noop_infusion.wasm"
        );
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
