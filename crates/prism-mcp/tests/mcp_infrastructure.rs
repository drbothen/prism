//! Red Gate tests for S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 Area E.
//!
//! BC-2.10.015: `FeatureFlagEvaluator::client_exists` Arc<OrgRegistry> DI wiring.
//! BC-2.10.016: Prompt hang investigation + fast-return guarantee.
//! BC-2.10.017: Not-yet-available tools fast-fail guard ordering.
//!
//! All test bodies call `todo!()` — the implementer writes the assertions.
//!
//! Red Gate tests: 6.

// ─── Area E: BC-2.10.015 — FeatureFlagEvaluator Arc<OrgRegistry> DI ──────────

/// AC-013 / BC-2.10.015 postcondition — construct a `FeatureFlagEvaluator` with a
/// populated `OrgRegistry` containing org slug `"acme"`, and assert `client_exists("acme")` → true
/// and `client_exists("unknown-org")` → false.
#[test]
fn test_bc_2_10_015_client_registered_true_from_org_registry() {
    todo!(
        "BC-2.10.015 AC-013: construct FeatureFlagEvaluator with OrgRegistry and assert \
         client_exists uses slug_exists; implementer implements the todo!() body"
    )
}

/// AC-014 / BC-2.10.015 postcondition — demo org `org-c` must be registered when the
/// demo prism.toml is loaded. Assert `client_exists("org-c")` → true against the
/// demo fixture OrgRegistry.
#[test]
fn test_bc_2_10_015_demo_provisioned_org_registered() {
    todo!(
        "BC-2.10.015 AC-014: assert org-c is registered in the demo OrgRegistry fixture \
         and client_exists returns true; implementer loads demo prism.toml"
    )
}

// ─── Area E: BC-2.10.016 — Prompt fast-return guarantee ────────────────────────

/// AC-015 / BC-2.10.016 postcondition — start a real rmcp server in-process
/// (tokio::time::timeout 5 s) and call `get_prompt` with a prompt name that requires
/// a missing required argument. Assert the response arrives within 5 s (no hang).
#[test]
fn test_bc_2_10_016_prompts_fast_return_within_5s() {
    todo!(
        "BC-2.10.016 AC-015: call get_prompt with missing required arg under 5s timeout \
         and assert no hang; implementer investigates PromptRouter dispatch + fixes hang"
    )
}

/// AC-016 / BC-2.10.016 postcondition — send `get_prompt` with a missing required arg
/// and assert the response is an error (not a successful prompt render) within 5 s.
#[test]
fn test_bc_2_10_016_missing_required_arg_fast_error() {
    todo!(
        "BC-2.10.016 AC-016: assert missing required arg returns error response within 5s; \
         implementer wires required-arg validation in PromptRouter dispatch"
    )
}

// ─── Area E: BC-2.10.017 — Not-yet-available fast-fail guard ordering ──────────

/// AC-017 / BC-2.10.017 postcondition — invoke a not-yet-available tool
/// (e.g., `create_schedule`) and assert the response arrives in under 1 s.
#[test]
fn test_bc_2_10_017_not_yet_available_fast_fail_under_1s() {
    todo!(
        "BC-2.10.017 AC-017: invoke not-yet-available tool and assert response within 1s; \
         implementer moves not_yet_available_msg guard before audit-await in handler"
    )
}

/// AC-018 / BC-2.10.017 invariant — guard ordering: the not-yet-available error must be
/// returned BEFORE any `emit_tool_audit().await` call in the handler. Verified by
/// code review / cargo-expand inspection that the guard precedes the await site.
#[test]
fn test_bc_2_10_017_not_yet_available_guard_precedes_audit() {
    todo!(
        "BC-2.10.017 AC-018: assert not_yet_available guard fires before emit_tool_audit await; \
         implementer verifies by inspection / cargo-expand that guard order is correct"
    )
}
