//! Route modules for the CrowdStrike DTU.

pub mod detections;
pub mod hosts;
pub mod oauth;
pub mod writes;

use std::sync::Arc;

use axum::Router;

use crate::state::CrowdstrikeState;

/// Build the full axum router for the CrowdStrike DTU.
///
/// Wires all 8 in-scope endpoints (4 read, 4 write) plus the OAuth token endpoint.
pub fn build_router(_state: Arc<CrowdstrikeState>) -> Router {
    unimplemented!("build_router — not yet implemented")
}
