//! Sensor-specific harness clone routers.
//!
//! Each sub-module provides a self-contained axum router for a specific DTU type.
//! The builder dispatches on `DtuType` to select the correct router.
//!
//! # Architecture
//!
//! - Logical-mode clones: one router per `(OrgId, DtuType)`, all in-process.
//! - Each module owns its own state type with org-scoped IDs (D-059).
//!
//! # Story
//!
//! S-3.4.03 — Migrate prism-dtu-crowdstrike tests to prism-dtu-harness.

pub mod crowdstrike;
