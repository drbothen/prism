//! prism-security — Feature Flags, Capability Gating, Prompt Injection Defense,
//! and Audit Logging (SS-04 + SS-09).
//!
//! S-1.08: P0 Core — Two-tier write gate, hidden tools pattern, list_capabilities,
//! and write operation audit logging.
//!
//! S-1.10: Four-layer prompt injection defense system:
//!   1. Structural separation of untrusted data (BC-2.09.001)
//!   2. Provenance framing in tool descriptions (BC-2.09.002)
//!   3. Suspicious pattern detection with NFKC normalization (BC-2.09.003)
//!   4. Trust-level metadata on every response (BC-2.09.005)
//!
//! # Subsystem
//! SS-04: Security / Safety (feature flags)
//! SS-09: Prompt Injection Defense
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

// ── S-1.08: Feature Flags (SS-04) ────────────────────────────────────────────
pub mod feature_flag;
pub mod flag_audit;
pub mod hidden_tools;
pub mod list_capabilities;

// ── S-1.09: Confirmation Tokens (SS-04) ──────────────────────────────────────
pub mod confirmation_token;
pub mod content_hash;
pub mod risk_tier;

// ── S-1.10: Prompt Injection Defense (SS-09) ─────────────────────────────────
pub mod injection_scanner;
pub mod output_schema;
pub mod provenance;
pub mod trust_level;

// ─────────────────────────────────────────────────────────────
// Public re-exports — S-1.08
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

// ─────────────────────────────────────────────────────────────
// Public re-exports — S-1.09
// ─────────────────────────────────────────────────────────────

pub use confirmation_token::{ConfirmationToken, ConfirmationTokenStore, TOKEN_CAP, TOKEN_TTL};
pub use content_hash::compute_action_hash;
pub use risk_tier::{DryRunResponse, GateDecision, RiskTier};

// ─────────────────────────────────────────────────────────────
// Public re-exports — S-1.10
// ─────────────────────────────────────────────────────────────

pub use injection_scanner::{InjectionScanner, ScanInput, ScanResult};
pub use output_schema::{MetaEnvelopeSchema, OutputSchema, ResultsItemSchema};
pub use provenance::{ProvenanceFraming, SecurityWarning, ToolDescriptionTemplate};
pub use trust_level::TrustLevelExt;
