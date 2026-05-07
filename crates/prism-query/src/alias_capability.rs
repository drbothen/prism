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
    scope: &AliasScope,
    evaluator: &FeatureFlagEvaluator,
    compile_gate: CompileTimeGate,
) -> Result<(), PrismError> {
    let client_id = match scope {
        AliasScope::Global => "__global__",
        AliasScope::Client(id) => id.0.as_str(),
    };

    let result = evaluator.check_permission(compile_gate, client_id, ALIAS_WRITE_CAPABILITY);

    match result {
        CapabilityCheckResult::Allowed => Ok(()),
        other => Err(denied_error(other, scope)),
    }
}

/// Check whether `alias.write` is enabled for at least one client in the set.
///
/// Used for `Global` scope authorization (visible-to-any-client semantics).
///
/// Returns `Ok(())` if any client allows `alias.write`, otherwise returns
/// `Err(PrismError::CapabilityDenied)` with the most specific denial reason.
#[allow(dead_code)]
pub(crate) fn check_alias_write_any_client(
    evaluator: &FeatureFlagEvaluator,
    compile_gate: CompileTimeGate,
    client_capabilities: &[(String, ClientCapabilities)],
) -> Result<(), PrismError> {
    let mut last_err: Option<PrismError> = None;

    for (client_id, _caps) in client_capabilities {
        let result = evaluator.check_permission(compile_gate, client_id, ALIAS_WRITE_CAPABILITY);
        match result {
            CapabilityCheckResult::Allowed => return Ok(()),
            other => {
                last_err = Some(denied_error(other, &AliasScope::Global));
            }
        }
    }

    Err(last_err.unwrap_or_else(|| PrismError::CapabilityDenied {
        capability: ALIAS_WRITE_CAPABILITY.to_string(),
        client_id: "__global__".to_string(),
        reason: "no clients configured".to_string(),
        suggestion: "configure at least one client with alias.write = allow".to_string(),
        resolution_trace: vec![],
    }))
}

/// Build a `CapabilityDenied` error for alias write operations.
///
/// This is a helper that formats the structured error expected by MCP callers
/// (BC-2.04.015 / E-FLAG-001).
#[allow(dead_code)]
pub(crate) fn denied_error(result: CapabilityCheckResult, scope: &AliasScope) -> PrismError {
    let client_id = scope.token_client_id().to_string();
    match result {
        CapabilityCheckResult::DeniedCompileTime {
            capability,
            resolution_trace,
            ..
        } => PrismError::CapabilityDenied {
            capability,
            client_id,
            reason: "compile-time feature gate not present".to_string(),
            suggestion: "rebuild with the alias-write feature enabled".to_string(),
            resolution_trace,
        },
        CapabilityCheckResult::DeniedRuntime {
            capability,
            resolution_trace,
            ..
        } => PrismError::CapabilityDenied {
            capability,
            client_id,
            reason: "runtime capability denied".to_string(),
            suggestion: "set alias.write = allow in the client's capability TOML".to_string(),
            resolution_trace,
        },
        CapabilityCheckResult::Allowed => {
            // This should never be called with Allowed — defensive guard.
            PrismError::CapabilityDenied {
                capability: ALIAS_WRITE_CAPABILITY.to_string(),
                client_id,
                reason: "unexpected: denied_error called with Allowed result".to_string(),
                suggestion: String::new(),
                resolution_trace: vec![],
            }
        }
    }
}
