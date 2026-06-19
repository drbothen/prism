//! Built-in infusion source backends.
//!
//! Dispatches to the appropriate source implementation based on `BuiltInSourceType`.
//! Called from `InfusionRegistry::load_spec` / `load_all` to wire the real file-backed
//! source for LocalLookup specs (BC-2.19.001).
//!
//! # SEC-001 (CWE-400) — source file size guard
//! All three built-in source loaders (`CsvSource::load`, `JsonLookupSource::load`,
//! `MmdbSource::load`) check `fs::metadata(&path)?.len()` against `MAX_SOURCE_FILE_BYTES`
//! BEFORE reading the file into memory. A file that exceeds the limit is rejected with
//! `InfusionError::SourceFileTooLarge` (E-INFUSE-012), preventing unbounded-memory OOM.

pub mod csv;
pub mod json_lookup;
pub mod mmdb;

/// Maximum allowed size (in bytes) for an infusion source data file.
///
/// 100 MiB = 104,857,600 bytes. Enforced at load time and hot-reload time for
/// CSV, JSON-lookup, and MMDB sources — before any file read — to prevent
/// unbounded-memory OOM (CWE-400). SEC-001, BC-2.19.001 §Error Conditions E-INFUSE-012.
///
/// Operators needing larger files may raise this limit by modifying this constant
/// and rebuilding. The error message (E-INFUSE-012) includes the remedy text:
/// "reduce the file or raise MAX_SOURCE_FILE_BYTES".
pub const MAX_SOURCE_FILE_BYTES: u64 = 104_857_600;

use std::path::Path;
use std::sync::Arc;

use prism_core::InfusionError;

use super::{BuiltInSourceType, InfusionSource, InfusionSourceConfig};
use csv::CsvSource;
use json_lookup::JsonLookupSource;
use mmdb::MmdbSource;

/// Load the appropriate `InfusionSource` implementation for the given config.
///
/// Dispatches to `MmdbSource::load`, `CsvSource::load`, or `JsonLookupSource::load`
/// based on `config.source_type`. Called from `InfusionRegistry::load_spec` (and
/// `load_spec_with_runtime` / `hot_reload`) when constructing the per-spec source
/// backend for a `LocalLookup`-type spec (BC-2.19.001).
///
/// Returns `Err(InfusionError::MissingRequiredField)` for backend load failures
/// (missing file, parse error, etc.).
pub fn load_source(
    config: &InfusionSourceConfig,
) -> Result<Arc<dyn InfusionSource>, InfusionError> {
    // `#[allow(unreachable_patterns)]`: `BuiltInSourceType` is `#[non_exhaustive]` so
    // the wildcard arm is required for external crates but is unreachable within this crate
    // (the compiler knows all variants here). This is intentional forward-compat scaffolding.
    #[allow(unreachable_patterns)]
    match config.source_type {
        BuiltInSourceType::MaxmindMmdb => {
            // MmdbSource::load takes only the path — column projection is handled
            // at the UDF layer via InfusionUdfDescriptor::source_column.
            let source = MmdbSource::load(Path::new(&config.file_path))?;
            Ok(Arc::new(source))
        }
        BuiltInSourceType::Csv => {
            let key_column = config.key_column.as_deref().unwrap_or("id");
            let source = CsvSource::load(&config.file_path, key_column)?;
            Ok(Arc::new(source))
        }
        BuiltInSourceType::JsonLookup => {
            let source = JsonLookupSource::load(&config.file_path)?;
            Ok(Arc::new(source))
        }
        // `BuiltInSourceType` is `#[non_exhaustive]`; future variants handled here.
        _ => Err(InfusionError::UnknownSourceType {
            type_name: format!("{:?}", config.source_type),
        }),
    }
}
