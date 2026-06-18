//! Built-in infusion source backends.
//!
//! Dispatches to the appropriate source implementation based on `BuiltInSourceType`.
//! Called from `InfusionRegistry::load_spec` / `load_all` to wire the real file-backed
//! source for LocalLookup specs (BC-2.19.001).

pub mod csv;
pub mod json_lookup;
pub mod mmdb;

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
