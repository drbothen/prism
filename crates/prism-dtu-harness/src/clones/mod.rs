//! Per-DTU clone routers for `prism-dtu-harness`.
//!
//! Each module provides a self-contained axum `Router` factory for a specific
//! DTU type, allowing the harness builder to dispatch on `DtuType` and serve
//! sensor-specific endpoints alongside the shared `CloneState` (failure injection,
//! admin configure, health checks).
//!
//! # Architecture Anchor
//!
//! - S-3.4.04: Cyberint harness migration — cookie auth + alert lifecycle routes

pub mod cyberint;
