//! `prism-dtu-demo-server` — Unified multi-clone demo harness for Prism DTU clones.
//!
//! This crate is gated behind `#[cfg(any(test, feature = "dtu"))]` and must NEVER
//! link into a production binary. The binary target enforces this via
//! `required-features = ["dtu"]` in `Cargo.toml`.
//!
//! # Architecture
//!
//! The demo harness boots multiple DTU clones in a single process. It orchestrates
//! the six merged DTU clones via their `BehavioralClone` trait interfaces
//! (`prism-dtu-crowdstrike`, `-claroty`, `-cyberint`, `-armis`, `-threatintel`, `-nvd`).
//! It does NOT duplicate any clone logic.
//!
//! See `S-6.20` story spec and `ADR-002 Amendment` for the design rationale.
#![cfg(any(test, feature = "dtu"))]

pub mod config;
pub mod harness;
pub mod multi_instance;
pub mod multi_org_cmd;
pub mod tls;

/// Name of the flat URL sidecar file written by `start` subcommand.
///
/// Format: `{name: url}` (one entry per clone).
/// Shared between `main.rs` (binary) and `multi_org_cmd.rs` (library) so that
/// `resolve_configure_url` can reference it in error messages.
pub const URL_FILE: &str = ".prism-dtu-demo-server.urls.json";

/// Name of the nested URL sidecar file written by `start-multi` subcommand.
///
/// Format: `{org_slug: {sensor_id: url}}`.
/// Shared between `main.rs` (binary) and `multi_org_cmd.rs` (library) so that
/// `resolve_configure_url` can reference it in error messages.
pub const URL_MULTI_FILE: &str = ".prism-dtu-demo-server.urls-multi.json";

/// Name of the flat admin-token sidecar file written by `start` subcommand.
///
/// Format: `{name: token}` (one entry per clone, mirroring `URL_FILE`).
/// Written atomically (tmp+rename) alongside `URL_FILE` so that `cmd_configure`
/// can obtain the per-clone admin token for the `X-Admin-Token` header
/// (ADR-003 Amendment #5 / AC-002 of DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001).
pub const TOKEN_FILE: &str = ".prism-dtu-demo-server.admin-tokens.json";

/// Name of the nested admin-token sidecar file written by `start-multi` subcommand.
///
/// Format: `{org_slug: {sensor_id: token}}` (mirroring `URL_MULTI_FILE`).
/// Written atomically (tmp+rename) alongside `URL_MULTI_FILE`.
pub const TOKEN_MULTI_FILE: &str = ".prism-dtu-demo-server.admin-tokens-multi.json";

// Re-export primary types for test usage.
pub use config::{
    DemoConfig, EnrichmentConfig, MultiOrgDemoConfig, OrgConfig, KNOWN_ENRICHMENT_CLONES,
    KNOWN_SENSORS,
};
pub use harness::{write_token_sidecar_to_path, ClonePair, DemoHarness, StartReport};
pub use multi_instance::{
    start_instances, DemoBindError, InstanceEntry, MultiInstanceBindError, MultiInstanceConfig,
    MultiInstanceServers,
};
// S-DEMO-LAUNCHER-CONSOLIDATION-001: testable extracted functions for `start-multi`.
// Re-exported so integration tests in tests/multi_org.rs can call them directly
// without subprocess overhead (Architecture Compliance Rule).
pub use multi_org_cmd::{
    build_multi_clone_factory, resolve_configure_token, resolve_configure_url,
    start_multi_for_config, write_multi_admin_token_sidecar_to_path,
    write_multi_url_sidecar_to_path,
};
