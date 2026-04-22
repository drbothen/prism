//! prism-security — Feature Flags, Capability Gating, and Audit Logging (SS-04).
//!
//! S-1.08: P0 Core — Two-tier write gate, hidden tools pattern, list_capabilities,
//! and write operation audit logging.
//!
//! # Subsystem
//! SS-04: Security / Safety
//!
//! # Compile-time write feature gates (BC-2.04.001)
//! Write operation code families are entirely absent from the binary unless
//! the corresponding Cargo feature is explicitly enabled at build time:
//! - `crowdstrike-write`
//! - `cyberint-write`
//! - `claroty-write`
//! - `armis-write`
//! - `all-write` (enables all four)
//!
//! Read operations (`read-all` default feature) are always available.

// cfg(kani) is set by the Kani verification toolchain, not by Cargo features.
#![allow(unexpected_cfgs)]

pub mod feature_flag;
pub mod flag_audit;
pub mod hidden_tools;
pub mod list_capabilities;

// ─────────────────────────────────────────────────────────────
// Public re-exports
// ─────────────────────────────────────────────────────────────

pub use feature_flag::{
    armis_write_gate, claroty_write_gate, crowdstrike_write_gate, cyberint_write_gate,
    CapabilityCheckResult, CompileTimeGate, FeatureFlagEvaluator,
};
pub use flag_audit::{CapabilityCheckEvent, FlagAuditEmitter};
pub use hidden_tools::{HiddenToolsRegistry, RegisteredTool, ToolKind};
pub use list_capabilities::{
    CapabilityMatrixEntry, CapabilityStatus, ListCapabilitiesEngine, ListCapabilitiesQuery,
};
