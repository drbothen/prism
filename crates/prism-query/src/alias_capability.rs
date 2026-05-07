//! `alias.write` capability gate for alias CRUD operations.
//!
//! Wraps the S-1.08 `FeatureFlagEvaluator` to enforce the `alias.write`
//! capability check required by BC-2.11.008 preconditions.
//!
//! ## Gate semantics (BC-2.11.008)
//!
//! - For `client:<client_id>` scope: check `alias.write` against the target
//!   client's capability set.
//! - For `global` scope: check `alias.write` against AT LEAST ONE configured
//!   client's capability set. The operation is authorized if any single client
//!   allows it (hidden-tools pattern: the tool appears in `tools/list` if any
//!   client enables it).
//!
//! Both tiers (compile-time Cargo feature + runtime TOML flag) must
//! independently return Allow (BC-2.04.004).
//!
//! Story: S-3.04 — prism-query: Alias System (P1)
//! BCs:   BC-2.11.008

use prism_core::capability::ClientCapabilities;
use prism_core::error::PrismError;
use prism_security::feature_flag::{CapabilityCheckResult, CompileTimeGate, FeatureFlagEvaluator};

use crate::alias_types::AliasScope;

/// Capability path for alias write operations (matches the TOML key).
pub const ALIAS_WRITE_CAPABILITY: &str = "alias.write";

/// Check the `alias.write` capability for the given scope.
///
/// # Arguments
/// - `scope`        — the alias scope being targeted.
/// - `evaluator`    — the configured `FeatureFlagEvaluator` for the current client set.
/// - `compile_gate` — compile-time feature gate status (caller passes
///   `CompileTimeGate::Present` / `Absent` based on `#[cfg(feature)]`).
///
/// # Returns
/// - `Ok(())` if the capability is allowed.
/// - `Err(PrismError::CapabilityDenied)` if either tier denies the operation.
pub fn check_alias_write(
    _scope: &AliasScope,
    _evaluator: &FeatureFlagEvaluator,
    _compile_gate: CompileTimeGate,
) -> Result<(), PrismError> {
    todo!()
}

/// Check whether `alias.write` is enabled for at least one client in the set.
///
/// Used for `Global` scope authorization (visible-to-any-client semantics).
///
/// Returns `Ok(())` if any client allows `alias.write`, otherwise returns
/// `Err(PrismError::CapabilityDenied)` with the most specific denial reason.
#[allow(dead_code)]
pub(crate) fn check_alias_write_any_client(
    _evaluator: &FeatureFlagEvaluator,
    _compile_gate: CompileTimeGate,
    _client_capabilities: &[(String, ClientCapabilities)],
) -> Result<(), PrismError> {
    todo!()
}

/// Build a `CapabilityDenied` error for alias write operations.
///
/// This is a helper that formats the structured error expected by MCP callers
/// (BC-2.04.015 / E-FLAG-001).
#[allow(dead_code)]
pub(crate) fn denied_error(_result: CapabilityCheckResult, _scope: &AliasScope) -> PrismError {
    todo!()
}
