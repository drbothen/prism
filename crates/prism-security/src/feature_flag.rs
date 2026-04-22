// S-1.08: Feature Flag Evaluator — STUB (Red Gate)
//
// All function bodies are `unimplemented!()`.  The implementer must fill them
// in to make the test suite green.
//
// Story:  S-1.08 — prism-security: Feature Flags (P0 Core)
// BCs:    BC-2.04.001, BC-2.04.002, BC-2.04.003, BC-2.04.004
// VP:     VP-020 (Kani proof: compile-time disabled → always Deny)
//
// Architecture compliance rules:
//   - `check_permission` MUST default to Deny when no capability config is present (AD-019).
//   - Compile-time gate (Cargo feature absent) CANNOT be overridden by runtime TOML (BC-2.04.001).
//   - `BTreeMap` MUST be used for capability storage — NOT HashMap (BC-2.04.003).
//   - Both tiers must independently return Allow for the combined result to be Allow (BC-2.04.004).

use std::collections::BTreeMap;

use prism_core::capability::{CapabilityEffect, CapabilityPath, ClientCapabilities};
use prism_core::error::PrismError;

// ─────────────────────────────────────────────────────────────
// Tier-1: Compile-time feature gate model
// ─────────────────────────────────────────────────────────────

/// Represents the compile-time feature gate status for a write code family.
///
/// In production, this is determined by `#[cfg(feature = "...")]` gating.
/// Tests model it as a runtime bool per VP-020 feasibility assessment:
/// "Compile-time gate modeled as runtime bool in test; separate build-matrix
/// test covers the real cfg gate."
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompileTimeGate {
    /// The Cargo feature is present in this binary.
    Present,
    /// The Cargo feature is absent — write code does not exist in this binary.
    Absent,
}

// ─────────────────────────────────────────────────────────────
// CapabilityCheckResult
// ─────────────────────────────────────────────────────────────

/// The outcome of a two-tier capability check, including the denial tier and
/// resolution trace required by E-FLAG-001 (BC-2.04.015).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CapabilityCheckResult {
    /// Both tiers passed — the operation is permitted.
    Allowed,
    /// Denied by the compile-time tier (Cargo feature absent).
    DeniedCompileTime {
        capability: String,
        client_id: String,
        /// Ordered resolution trace for E-FLAG-001 structured error.
        resolution_trace: Vec<String>,
    },
    /// Denied by the runtime tier (capability not in client config).
    DeniedRuntime {
        capability: String,
        client_id: String,
        /// Ordered resolution trace for E-FLAG-001 structured error.
        resolution_trace: Vec<String>,
    },
}

// ─────────────────────────────────────────────────────────────
// FeatureFlagEvaluator
// ─────────────────────────────────────────────────────────────

/// Two-tier feature flag evaluator for write operations (BC-2.04.004).
///
/// Tier 1: compile-time Cargo feature gate (BC-2.04.001).
/// Tier 2: runtime per-client TOML capability configuration (BC-2.04.002).
///
/// Both tiers must independently return Allow for the combined result to be
/// Allowed. The compile-time gate is modeled here as a `CompileTimeGate`
/// enum passed at construction time; in production binaries the calling code
/// is absent if the feature is not compiled in.
///
/// Client capabilities are stored as `BTreeMap<String, ClientCapabilities>`
/// for deterministic iteration order required by the resolution trace
/// (BC-2.04.003 architecture compliance rule).
pub struct FeatureFlagEvaluator {
    /// Per-client capability maps keyed by client ID.
    /// `BTreeMap` required — NOT `HashMap` — for deterministic trace order.
    client_capabilities: BTreeMap<String, ClientCapabilities>,
}

impl FeatureFlagEvaluator {
    /// Construct a `FeatureFlagEvaluator` with pre-resolved per-client
    /// capability maps.
    ///
    /// `client_capabilities` MUST be a `BTreeMap` — see architecture
    /// compliance rule in story spec.
    pub fn new(client_capabilities: BTreeMap<String, ClientCapabilities>) -> Self {
        unimplemented!("S-1.08: FeatureFlagEvaluator::new — implement construction")
    }

    /// Perform a two-tier capability check.
    ///
    /// # Parameters
    /// - `compile_gate`: whether the write code family is compiled in (Tier 1).
    /// - `client_id`: the client whose runtime capabilities are consulted (Tier 2).
    /// - `capability`: the dot-separated path to check (e.g., `"sensor.crowdstrike.containment"`).
    ///
    /// # Returns
    /// - `CapabilityCheckResult::Allowed` — both tiers pass.
    /// - `CapabilityCheckResult::DeniedCompileTime` — compile gate absent.
    /// - `CapabilityCheckResult::DeniedRuntime` — runtime capability missing or denied.
    ///
    /// # Invariant (VP-020)
    /// When `compile_gate == CompileTimeGate::Absent`, the result is ALWAYS
    /// `DeniedCompileTime` regardless of runtime capability configuration.
    pub fn check_permission(
        &self,
        compile_gate: CompileTimeGate,
        client_id: &str,
        capability: &str,
    ) -> CapabilityCheckResult {
        unimplemented!("S-1.08: FeatureFlagEvaluator::check_permission — implement two-tier gate")
    }

    /// Convert a `CapabilityCheckResult::Denied*` into a structured
    /// `PrismError::CapabilityDenied` (E-FLAG-001, BC-2.04.015).
    ///
    /// Returns `None` if the result is `Allowed`.
    pub fn to_error(&self, result: &CapabilityCheckResult) -> Option<PrismError> {
        unimplemented!("S-1.08: FeatureFlagEvaluator::to_error — implement E-FLAG-001 error construction")
    }

    /// Return true if `client_id` is present in the configured client map.
    pub fn client_exists(&self, client_id: &str) -> bool {
        unimplemented!("S-1.08: FeatureFlagEvaluator::client_exists — implement client lookup")
    }
}

// ─────────────────────────────────────────────────────────────
// Compile-time write feature gate wrappers
// ─────────────────────────────────────────────────────────────
//
// These functions return the compile-time gate status for each write code
// family. In production code they use `#[cfg(feature = "...")]` to determine
// the value. Tests can call them to verify the real binary gate, but the
// `check_permission` tests use the `CompileTimeGate` enum directly to model
// the 2×2 truth table per VP-020.

/// Returns `CompileTimeGate::Present` if `crowdstrike-write` is compiled in,
/// `CompileTimeGate::Absent` otherwise (BC-2.04.001).
pub fn crowdstrike_write_gate() -> CompileTimeGate {
    #[cfg(feature = "crowdstrike-write")]
    {
        CompileTimeGate::Present
    }
    #[cfg(not(feature = "crowdstrike-write"))]
    {
        CompileTimeGate::Absent
    }
}

/// Returns `CompileTimeGate::Present` if `cyberint-write` is compiled in.
pub fn cyberint_write_gate() -> CompileTimeGate {
    #[cfg(feature = "cyberint-write")]
    {
        CompileTimeGate::Present
    }
    #[cfg(not(feature = "cyberint-write"))]
    {
        CompileTimeGate::Absent
    }
}

/// Returns `CompileTimeGate::Present` if `claroty-write` is compiled in.
pub fn claroty_write_gate() -> CompileTimeGate {
    #[cfg(feature = "claroty-write")]
    {
        CompileTimeGate::Present
    }
    #[cfg(not(feature = "claroty-write"))]
    {
        CompileTimeGate::Absent
    }
}

/// Returns `CompileTimeGate::Present` if `armis-write` is compiled in.
pub fn armis_write_gate() -> CompileTimeGate {
    #[cfg(feature = "armis-write")]
    {
        CompileTimeGate::Present
    }
    #[cfg(not(feature = "armis-write"))]
    {
        CompileTimeGate::Absent
    }
}
