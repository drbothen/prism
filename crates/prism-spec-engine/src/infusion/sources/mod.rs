//! Built-in infusion source backends.
//!
//! Dispatches to the appropriate source implementation based on `BuiltInSourceType`.
//! All source types are stubs (`unimplemented!()`) — implementation in S-1.14.

pub mod csv;
pub mod json_lookup;
pub mod mmdb;

use std::sync::Arc;

use prism_core::InfusionError;

use super::{InfusionSource, InfusionSourceConfig};

/// Load the appropriate `InfusionSource` implementation for the given config.
///
/// Dispatches to `MmdbSource::load`, `CsvSource::load`, or `JsonLookupSource::load`
/// based on `config.source_type`. Called from `InfusionLoader::load_all` when
/// constructing the per-spec source backend.
///
/// Returns `Err(InfusionError::UnknownSourceType)` for unrecognized source types.
pub fn load_source(
    _config: &InfusionSourceConfig,
) -> Result<Arc<dyn InfusionSource>, InfusionError> {
    todo!(
        "S-1.14-REDO: implement load_source — dispatch to MmdbSource/CsvSource/JsonLookupSource \
         based on config.source_type (BC-2.19.001)"
    )
}
