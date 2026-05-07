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
//! ## `alias-write` Cargo feature — advisory, not compile-exclusion (F-LOCAL-P2-HIGH-004)
//!
//! The `alias-write` Cargo feature is a **runtime-advisory** gate, not a
//! compile-time exclusion. When the feature is absent, `alias_write_compile_gate()`
//! returns `CompileTimeGate::Absent`, which causes `check_alias_write` to return
//! `CapabilityDenied` without touching the runtime evaluator. The public entry
//! points (`create_alias_with_clients_gated`, `delete_alias_gated`) remain
//! compiled and accessible in all feature configurations; they are not gated with
//! `#[cfg(feature = "alias-write")]`. This is intentional: the security guarantee
//! is provided by the `CompileTimeGate::Absent` path returning `CapabilityDenied`
//! before any mutation, not by conditional compilation. Callers MUST pass the gate
//! value from `alias_write_compile_gate()` rather than hardcoding
//! `CompileTimeGate::Present`.
//!
//! Story: S-3.04 — prism-query: Alias System (P1)
//! BCs:   BC-2.11.008, BC-2.11.006 v1.17

use prism_core::error::PrismError;
use prism_security::feature_flag::{CapabilityCheckResult, CompileTimeGate, FeatureFlagEvaluator};

use crate::alias_types::AliasScope;

/// Capability path for alias write operations (matches the TOML key).
pub const ALIAS_WRITE_CAPABILITY: &str = "alias.write";

/// Return the compile-time gate value for alias write operations.
///
/// Returns `CompileTimeGate::Present` when the `alias-write` Cargo feature is
/// enabled, and `CompileTimeGate::Absent` otherwise. Callers in the MCP dispatch
/// layer use this to avoid embedding `#[cfg(feature = "alias-write")]` throughout
/// the codebase (F-HIGH-002).
pub fn alias_write_compile_gate() -> CompileTimeGate {
    // The alias-write feature controls whether alias mutation tools are compiled in.
    // When the feature is absent, CompileTimeGate::Absent causes CapabilityDenied
    // to be returned without touching the runtime flag (BC-2.04.004 two-tier model).
    #[cfg(feature = "alias-write")]
    {
        CompileTimeGate::Present
    }
    #[cfg(not(feature = "alias-write"))]
    {
        CompileTimeGate::Absent
    }
}

/// Check the `alias.write` capability for the given scope.
///
/// # Arguments
/// - `scope`            — the alias scope being targeted.
/// - `evaluator`        — the configured `FeatureFlagEvaluator` for the current client set.
/// - `compile_gate`     — compile-time feature gate status (caller passes
///   `CompileTimeGate::Present` / `Absent` based on `#[cfg(feature)]`).
/// - `valid_client_ids` — the list of all known client IDs in the current tenant set.
///   Required to implement the BC-2.11.008 `Global` scope rule: "allow if AT LEAST ONE
///   configured client has `alias.write = Allow`." Pass `&[]` when no client list is
///   available (e.g., in tests that only exercise the per-client branch).
///
/// ## Scope semantics (BC-2.11.008)
///
/// - **`Client(id)`** — check `alias.write` against that specific client's capability.
/// - **`Global`** — iterate all `valid_client_ids`; allow if ANY returns Allow; deny only
///   if ALL return Deny/Indeterminate (hidden-tools pattern). The `"__global__"` sentinel
///   is NOT used for permission checks (CR-P6-002).
///
/// # Returns
/// - `Ok(())` if the capability is allowed.
/// - `Err(PrismError::CapabilityDenied)` if either tier denies the operation.
pub fn check_alias_write(
    scope: &AliasScope,
    evaluator: &FeatureFlagEvaluator,
    compile_gate: CompileTimeGate,
    valid_client_ids: &[String],
) -> Result<(), PrismError> {
    match scope {
        AliasScope::Client(id) => {
            // Per-client scope: check that specific client only.
            let result =
                evaluator.check_permission(compile_gate, id.0.as_str(), ALIAS_WRITE_CAPABILITY);
            match result {
                CapabilityCheckResult::Allowed => Ok(()),
                other => Err(denied_error(other, scope)),
            }
        }
        AliasScope::Global => {
            // Global scope: allow if AT LEAST ONE configured client has alias.write = Allow.
            // Deny only when ALL clients deny or no clients are configured (BC-2.11.008).
            let mut last_err: Option<PrismError> = None;

            for client_id in valid_client_ids {
                let result =
                    evaluator.check_permission(compile_gate, client_id, ALIAS_WRITE_CAPABILITY);
                match result {
                    CapabilityCheckResult::Allowed => return Ok(()),
                    other => {
                        last_err = Some(denied_error(other, scope));
                    }
                }
            }

            Err(last_err.unwrap_or_else(|| PrismError::CapabilityDenied {
                capability: ALIAS_WRITE_CAPABILITY.to_string(),
                client_id: scope.token_client_id().to_string(),
                reason: "no clients configured".to_string(),
                suggestion: "configure at least one client with alias.write = allow".to_string(),
                resolution_trace: vec![],
            }))
        }
    }
}

/// Build a `CapabilityDenied` error for alias write operations.
///
/// This is a helper that formats the structured error expected by MCP callers
/// (BC-2.04.015 / E-FLAG-001).
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
