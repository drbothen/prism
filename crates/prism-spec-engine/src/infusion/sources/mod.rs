//! Built-in infusion source backends.
//!
//! Dispatches to the appropriate source implementation based on `BuiltInSourceType`.
//! All source types are stubs (`unimplemented!()`) — implementation in S-1.14.

pub mod csv;
pub mod json_lookup;
pub mod mmdb;

use std::sync::Arc;

use prism_core::InfusionError;

use super::{BuiltInSourceType, InfusionSource, InfusionSourceConfig};

/// Load the appropriate `InfusionSource` implementation for the given config.
///
/// Returns a boxed `InfusionSource` or `InfusionError::UnknownSourceType` for
/// unrecognized source types.
pub fn load_source(config: &InfusionSourceConfig) -> Result<Arc<dyn InfusionSource>, InfusionError> {
    unimplemented!(
        "load_source — implement in S-1.14 (BC-2.19.001)"
    )
}
