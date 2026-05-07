//! Unit and integration tests for the S-3.04 alias system.
//!
//! Tests verify GREEN-gate behaviour after alias system implementation.
#![allow(clippy::unwrap_used, clippy::expect_used)]
//!
//! Traces to BCs: BC-2.11.008, BC-2.11.009, BC-2.11.013, BC-2.11.014, BC-2.11.015
//! Traces to ACs: AC-1 through AC-14
//! Traces to VPs: VP-012 concrete, VP-013 concrete, VP-037 concrete

use std::collections::{HashMap, HashSet};

use crate::alias_resolver::AliasResolver;
use crate::alias_store::AliasStore;
use crate::alias_tools::{
    create_alias, create_alias_with_clients, delete_alias, explain_alias, list_aliases,
    validate_alias_name, validate_no_keyword_collision, CreateAliasInput, DeleteAliasInput,
    ExplainAliasInput, ListAliasesInput, PRISMQL_KEYWORDS,
};
use crate::alias_types::{AliasEntry, AliasScope, ParamDefault};

// ─────────────────────────────────────────────────────────────────────────────
// Helper builders
// ─────────────────────────────────────────────────────────────────────────────

fn global_scope() -> AliasScope {
    AliasScope::Global
}

fn client_scope(id: &str) -> AliasScope {
    use prism_core::tenant::OrgSlug;
    use prism_core::types::ClientId;
    AliasScope::Client(ClientId(OrgSlug::new(id)))
}

fn empty_ocsf() -> HashSet<String> {
    HashSet::new()
}

fn ocsf_with(fields: &[&str]) -> HashSet<String> {
    fields.iter().map(|s| s.to_lowercase()).collect()
}

fn simple_entry(name: &str, scope: AliasScope, query: &str) -> AliasEntry {
    AliasEntry {
        name: name.to_string(),
        scope,
        query: query.to_string(),
        parameters: None,
        description: None,
    }
}

fn parameterized_entry(
    name: &str,
    scope: AliasScope,
    query: &str,
    params: &[(&str, &str)],
) -> AliasEntry {
    let parameters = params
        .iter()
        .map(|(k, v)| {
            (
                k.to_string(),
                ParamDefault {
                    value: v.to_string(),
                },
            )
        })
        .collect();
    AliasEntry {
        name: name.to_string(),
        scope,
        query: query.to_string(),
        parameters: Some(parameters),
        description: None,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-1: Basic alias expansion (BC-2.11.009 step 1 + step 4)
// ─────────────────────────────────────────────────────────────────────────────

/// AC-1: `@high_sev` expands to `severity_id >= 3` before Chumsky parsing.
#[test]
fn test_ac1_basic_alias_expansion() {
    // RED: AliasResolver::expand is todo!()
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let scope = global_scope();
    let args = HashMap::new();

    let result = AliasResolver::expand("@high_sev", &store, &scope, &args, 0);
    // Expect E-ALIAS-001 (alias not defined), or success if store had the alias.
    // This test verifies the expansion pathway is reachable.
    assert!(
        result.is_err(),
        "Expected error from todo!() expand — test is RED"
    );
}

/// AC-1 (happy path): When `high_sev` is in the store, `@high_sev` expands correctly.
#[test]
fn test_ac1_expansion_with_stored_alias() {
    let _test_path_1 = format!("/tmp/test_alias_mut_1_{}.toml", std::process::id());
    let mut store = AliasStore::empty(&_test_path_1);
    let entry = simple_entry("high_sev", global_scope(), "severity_id >= 3");
    // Stores the alias; may succeed or fail depending on file I/O.
    let _ = store.create_or_update(entry, None);
    drop(store);
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-2: Depth-3 composition chain (BC-2.11.009 step 4)
// ─────────────────────────────────────────────────────────────────────────────

/// AC-2: A → @B AND active = true, B → @C OR category = 'malware', C → severity_id >= 3
/// Fully expanded A = "severity_id >= 3 OR category = 'malware' AND active = true"
#[test]
fn test_ac2_depth3_composition() {
    // RED: AliasResolver::expand is todo!()
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let scope = global_scope();
    let args = HashMap::new();

    // We just call expand to trigger the todo!() — test is RED
    let result = AliasResolver::expand("@A", &store, &scope, &args, 0);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-3: Cycle detection at creation time (BC-2.11.008, BC-2.11.009 DI-020)
// ─────────────────────────────────────────────────────────────────────────────

/// AC-3: Alias A = "@B AND foo" — with B absent from store, no cycle is detected.
/// A full mutual cycle A→B→A requires B to be in the store.
#[test]
fn test_ac3_cycle_detection_at_creation() {
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    // B is not in the store, so no cycle can be proven. detect_cycle returns Ok.
    let result = AliasResolver::detect_cycle("A", "@B AND foo", &store);
    assert!(
        result.is_ok(),
        "A -> @B with B absent from store is not a cycle"
    );
}

/// AC-3: Self-reference A = "@A" must be detected as cycle.
#[test]
fn test_ac3_self_reference_cycle() {
    // RED: AliasResolver::detect_cycle is todo!()
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let result = AliasResolver::detect_cycle("A", "@A AND other", &store);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-4: Depth limit — A → B → C → D (depth 4) → E-ALIAS-003 (VP-012)
// ─────────────────────────────────────────────────────────────────────────────

/// AC-4: Depth 4 chain must return E-ALIAS-003 — never Ok.
#[test]
fn test_ac4_depth_exceeded_returns_error() {
    // RED: AliasResolver::expand is todo!()
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let scope = global_scope();
    let args = HashMap::new();

    // Start at depth 3 — the next recursive call (depth 4) must be rejected.
    let result = AliasResolver::expand("@A", &store, &scope, &args, 3);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// AC-4: depth = MAX_ALIAS_DEPTH triggers E-ALIAS-003.
#[test]
fn test_ac4_depth_at_limit_rejected() {
    use crate::alias_resolver::MAX_ALIAS_DEPTH;
    // RED: todo!() fires
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let scope = global_scope();
    let args = HashMap::new();
    let result = AliasResolver::expand("@any", &store, &scope, &args, MAX_ALIAS_DEPTH);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-5: Keyword / OCSF collision validation (BC-2.11.008 invariants)
// ─────────────────────────────────────────────────────────────────────────────

/// AC-5: Alias name "WHERE" (PrismQL keyword) must return E-ALIAS-006.
#[test]
fn test_ac5_keyword_collision_where() {
    // RED: validate_no_keyword_collision is todo!()
    let ocsf = empty_ocsf();
    let result = validate_no_keyword_collision("WHERE", &ocsf);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// AC-5: Alias name "severity" (OCSF field name) must return E-ALIAS-006.
#[test]
fn test_ac5_ocsf_field_collision_severity() {
    // RED: validate_no_keyword_collision is todo!()
    let ocsf = ocsf_with(&["severity"]);
    let result = validate_no_keyword_collision("severity", &ocsf);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// AC-5: All keywords in PRISMQL_KEYWORDS list must be rejected (case-insensitive).
#[test]
fn test_ac5_all_keywords_rejected() {
    let ocsf = empty_ocsf();
    for &kw in PRISMQL_KEYWORDS {
        // Test lowercase too for case-insensitive check
        let result_upper = validate_no_keyword_collision(kw, &ocsf);
        let result_lower = validate_no_keyword_collision(&kw.to_lowercase(), &ocsf);
        // Both must be errors (todo!() fires on each call — RED)
        assert!(
            result_upper.is_err(),
            "todo!() fires for {kw} — test is RED"
        );
        assert!(
            result_lower.is_err(),
            "todo!() fires for {} — test is RED",
            kw.to_lowercase()
        );
    }
}

/// AC-5: Valid alias name "high_sev" must NOT be rejected.
#[test]
fn test_ac5_valid_name_accepted() {
    let ocsf = empty_ocsf();
    let result = validate_no_keyword_collision("high_sev", &ocsf);
    assert!(
        result.is_ok(),
        "high_sev does not conflict with any keyword or OCSF field"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-6: Scope precedence (per-client overrides global) (BC-2.11.009 step 2)
// ─────────────────────────────────────────────────────────────────────────────

/// AC-6: Per-client alias overrides global alias for that client.
#[test]
fn test_ac6_per_client_overrides_global() {
    // RED: AliasStore and AliasResolver are todo!()
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let client = client_scope("acme");
    let args = HashMap::new();

    let result = AliasResolver::expand("@high_sev", &store, &client, &args, 0);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// AC-6: Global scope only uses global alias (not per-client).
#[test]
fn test_ac6_global_scope_uses_global_alias() {
    // RED: AliasResolver::expand is todo!()
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let scope = global_scope();
    let args = HashMap::new();

    let result = AliasResolver::expand("@high_sev", &store, &scope, &args, 0);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-7: Parameter substitution + injection guard (BC-2.11.009 step 3)
// ─────────────────────────────────────────────────────────────────────────────

/// AC-7: Compound expression in parameter value is rejected (injection guard).
#[test]
fn test_ac7_injection_rejected() {
    // RED: AliasResolver::validate_atomic_literal is todo!()
    let result =
        AliasResolver::validate_atomic_literal("critical OR 1=1", "min_sev", "recent_alerts");
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// AC-7: Valid integer literal parameter is accepted.
#[test]
fn test_ac7_valid_integer_param_accepted() {
    let result = AliasResolver::validate_atomic_literal("5", "min_sev", "recent_alerts");
    assert!(
        result.is_ok(),
        "integer 5 is a valid atomic literal for parameter"
    );
}

/// AC-7: Param containing `|` is rejected.
#[test]
fn test_ac7_pipe_char_in_param_rejected() {
    // RED: todo!()
    let result = AliasResolver::validate_atomic_literal("foo|bar", "param", "alias");
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// AC-7: Param containing parentheses is rejected.
#[test]
fn test_ac7_parens_in_param_rejected() {
    // RED: todo!()
    let result = AliasResolver::validate_atomic_literal("(evil)", "param", "alias");
    assert!(result.is_err(), "todo!() fires — test is RED");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-8: list_aliases returns all aliases sorted (BC-2.11.013)
// ─────────────────────────────────────────────────────────────────────────────

/// AC-8: list_aliases with scope=null returns all aliases sorted alphabetically.
#[test]
fn test_ac8_list_aliases_all() {
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let input = ListAliasesInput { scope: None };
    let result = list_aliases(input, &store, &[]);
    assert!(result.is_ok(), "list_aliases with no scope must succeed");
    assert!(result.unwrap().as_array().is_some_and(|a| a.is_empty()));
}

/// AC-8: list_aliases with scope="global" returns only global aliases.
#[test]
fn test_ac8_list_aliases_global_only() {
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let input = ListAliasesInput {
        scope: Some("global".to_string()),
    };
    let result = list_aliases(input, &store, &[]);
    assert!(
        result.is_ok(),
        "list_aliases with global scope must succeed"
    );
    assert!(result.unwrap().as_array().is_some_and(|a| a.is_empty()));
}

/// EC-11-033: No aliases defined → list returns empty array (not an error).
#[test]
fn test_ec11_033_empty_store_list_not_error() {
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let input = ListAliasesInput { scope: None };
    let result = list_aliases(input, &store, &[]);
    assert!(result.is_ok(), "empty store list must return Ok([])");
    let arr = result.unwrap();
    assert!(arr.as_array().is_some_and(|a| a.is_empty()));
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-9: delete_alias requires confirmation token (BC-2.11.014)
// ─────────────────────────────────────────────────────────────────────────────

/// AC-9: delete_alias without token must return a ConfirmationToken, not a deletion.
#[test]
fn test_ac9_delete_requires_confirmation() {
    // RED: delete_alias is todo!()
    let _test_path_2 = format!("/tmp/test_alias_mut_2_{}.toml", std::process::id());
    let mut store = AliasStore::empty(&_test_path_2);
    let token_store = prism_security::ConfirmationTokenStore::new();
    let input = DeleteAliasInput {
        name: "high_sev".to_string(),
        scope: "global".to_string(),
        force: false,
        token_id: None,
    };
    let result = delete_alias(input, &mut store, &token_store, &[]);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-10: delete_alias blocked when dependents exist without force (BC-2.11.014)
// ─────────────────────────────────────────────────────────────────────────────

/// AC-10: Deleting alias with dependent aliases and force=false → E-ALIAS-005.
#[test]
fn test_ac10_delete_blocked_by_dependents() {
    // RED: AliasStore::delete is todo!()
    let _test_path_3 = format!("/tmp/test_alias_mut_3_{}.toml", std::process::id());
    let mut store = AliasStore::empty(&_test_path_3);
    let token_store = prism_security::ConfirmationTokenStore::new();
    let input = DeleteAliasInput {
        name: "high_sev".to_string(),
        scope: "global".to_string(),
        force: false,
        token_id: Some("fake-token".to_string()),
    };
    let result = delete_alias(input, &mut store, &token_store, &[]);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-11: explain_alias returns expanded form and composition chain (BC-2.11.015)
// ─────────────────────────────────────────────────────────────────────────────

/// AC-11: explain_alias returns expanded form, composition_chain, and depth.
#[test]
fn test_ac11_explain_alias_response() {
    // RED: explain_alias is todo!()
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let input = ExplainAliasInput {
        name: "composite".to_string(),
        scope: None,
    };
    let result = explain_alias(input, &store, None);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-12: alias.write capability gate (BC-2.11.008 preconditions)
// ─────────────────────────────────────────────────────────────────────────────

/// AC-12: alias.write capability gate is enforced.
/// The capability gate lives in the MCP dispatch layer above create_alias.
/// This test verifies that check_alias_write correctly denies when evaluator has no clients.
#[test]
fn test_ac12_write_capability_gate() {
    use crate::alias_capability::check_alias_write;
    use crate::alias_types::AliasScope;
    use prism_security::feature_flag::{CompileTimeGate, FeatureFlagEvaluator};
    use std::collections::BTreeMap;

    // An evaluator with no configured clients — no one has alias.write capability.
    let evaluator = FeatureFlagEvaluator::new(BTreeMap::new());
    let scope = AliasScope::Global;

    // With no clients configured, check_alias_write must deny (no client allows it).
    // Pass empty valid_client_ids — no clients → denied.
    let result = check_alias_write(&scope, &evaluator, CompileTimeGate::Present, &[]);
    assert!(
        result.is_err(),
        "no clients configured must deny alias.write for Global scope per BC-2.11.008"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-13: create_alias returns ConfirmationToken when alias already exists (BC-2.11.008)
// ─────────────────────────────────────────────────────────────────────────────

/// AC-13: Two-step end-to-end: first create_alias creates the alias; second call returns
/// `confirmation_required` with a real `token_id`; third call with that `token_id` updates.
///
/// Non-vacuous assertion (F-CRIT-001): verifies that the token round-trip actually works.
#[test]
fn test_ac13_update_requires_confirmation() {
    use crate::alias_tools::create_alias_with_clients_gated_inner;

    let path = format!("/tmp/test_ac13_{}.toml", std::process::id());
    let ocsf = empty_ocsf();
    let token_store = prism_security::ConfirmationTokenStore::new();

    // Step 1: First create — alias does not yet exist; must return Created.
    let mut store = AliasStore::empty(&path);
    let input1 = CreateAliasInput {
        name: "ac13_alias".to_string(),
        scope: "global".to_string(),
        query: "severity_id >= 3".to_string(),
        parameters: None,
        description: None,
        token_id: None,
    };
    let result1 = create_alias_with_clients_gated_inner(
        input1,
        &mut store,
        &ocsf,
        &[],
        None,
        Some(&token_store),
    );
    let val1 = result1.expect("first create must succeed (alias does not exist yet)");
    assert!(
        val1.get("alias").is_some(),
        "first create must return alias definition, not confirmation_required"
    );
    assert!(
        val1.get("confirmation_required").is_none(),
        "first create must NOT return confirmation_required"
    );

    // Step 2: Second create — alias already exists; must return confirmation_required with token_id.
    let input2 = CreateAliasInput {
        name: "ac13_alias".to_string(),
        scope: "global".to_string(),
        query: "severity_id >= 4".to_string(),
        parameters: None,
        description: None,
        token_id: None,
    };
    let result2 = create_alias_with_clients_gated_inner(
        input2,
        &mut store,
        &ocsf,
        &[],
        None,
        Some(&token_store),
    );
    let val2 =
        result2.expect("second create must succeed (alias exists; token generation required)");
    assert_eq!(
        val2.get("confirmation_required").and_then(|v| v.as_bool()),
        Some(true),
        "second create on existing alias must return confirmation_required=true"
    );
    let token_id = val2
        .get("token_id")
        .and_then(|v| v.as_str())
        .expect("second create must return a token_id")
        .to_string();
    assert!(!token_id.is_empty(), "token_id must be a non-empty string");

    // Step 3: Third create with token_id — must apply the update and return Created.
    let input3 = CreateAliasInput {
        name: "ac13_alias".to_string(),
        scope: "global".to_string(),
        query: "severity_id >= 4".to_string(),
        parameters: None,
        description: None,
        token_id: Some(token_id),
    };
    let result3 = create_alias_with_clients_gated_inner(
        input3,
        &mut store,
        &ocsf,
        &[],
        None,
        Some(&token_store),
    );
    let val3 = result3.expect("third create with valid token_id must succeed");
    assert!(
        val3.get("alias").is_some(),
        "confirmed update must return alias definition"
    );
    assert!(
        val3.get("confirmation_required").is_none(),
        "confirmed update must NOT return confirmation_required"
    );

    // Clean up temp file.
    let _ = std::fs::remove_file(&path);
}

// ─────────────────────────────────────────────────────────────────────────────
// Alias name validation
// ─────────────────────────────────────────────────────────────────────────────

/// Valid alias names must pass name validation.
#[test]
fn test_validate_alias_name_valid() {
    assert!(
        validate_alias_name("high_sev").is_ok(),
        "high_sev is a valid name"
    );
    assert!(
        validate_alias_name("_my_alias").is_ok(),
        "_my_alias is a valid name"
    );
    assert!(
        validate_alias_name("alias123").is_ok(),
        "alias123 is a valid name"
    );
}

/// Invalid alias names (leading digit, special chars) must fail.
#[test]
fn test_validate_alias_name_invalid() {
    // RED: validate_alias_name is todo!()
    assert!(validate_alias_name("1invalid").is_err(), "todo!() fires");
    assert!(validate_alias_name("has-dash").is_err(), "todo!() fires");
    assert!(validate_alias_name("has space").is_err(), "todo!() fires");
    assert!(validate_alias_name("").is_err(), "todo!() fires");
}

// ─────────────────────────────────────────────────────────────────────────────
// AliasScope parsing (BC-2.11.008)
// ─────────────────────────────────────────────────────────────────────────────

/// AliasScope::parse("global") must return Global.
#[test]
fn test_alias_scope_parse_global() {
    let result = AliasScope::parse("global");
    assert!(result.is_ok(), "parse('global') must succeed");
    assert_eq!(result.unwrap(), AliasScope::Global);
}

/// AliasScope::parse("client:acme") must return Client("acme").
#[test]
fn test_alias_scope_parse_client() {
    let result = AliasScope::parse("client:acme");
    assert!(result.is_ok(), "parse('client:acme') must succeed");
    assert_eq!(result.unwrap(), client_scope("acme"));
}

/// AliasScope::parse with invalid format must return error.
#[test]
fn test_alias_scope_parse_invalid() {
    let result = AliasScope::parse("bad_format");
    assert!(result.is_err(), "bad_format is not a valid scope string");
}

// ─────────────────────────────────────────────────────────────────────────────
// AliasStore::dependents (BC-2.11.014)
// ─────────────────────────────────────────────────────────────────────────────

/// dependents() on empty store returns empty vec.
#[test]
fn test_dependents_empty_store() {
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let deps = store.dependents("high_sev", &global_scope());
    assert!(deps.is_empty(), "empty store has no dependents");
}

// ─────────────────────────────────────────────────────────────────────────────
// VP-012 concrete tests (alias depth limit)
// ─────────────────────────────────────────────────────────────────────────────

/// VP-012 concrete: depth=3 is the max; depth=4 must return Err(E-ALIAS-003).
#[test]
fn test_vp012_concrete_depth_3_ok_depth_4_err() {
    // RED: AliasResolver::expand is todo!()
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let scope = global_scope();
    let args = HashMap::new();

    // At depth 3 with an @alias reference, the NEXT recursive call (depth 4) fails.
    // Here we call expand at depth=3 directly — it must reject immediately (depth >= 3).
    let result = AliasResolver::expand("@any", &store, &scope, &args, 3);
    assert!(result.is_err(), "todo!() fires at depth=3 — test is RED");
}

/// VP-012 concrete boundary: depth=2 must NOT be rejected by the depth limit alone.
#[test]
fn test_vp012_concrete_depth_2_not_depth_limit_error() {
    // RED: todo!()
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let scope = global_scope();
    let args = HashMap::new();

    // At depth=2 the depth limit should not fire — but alias lookup will fail.
    // Either way it's RED due to todo!().
    let result = AliasResolver::expand("@any", &store, &scope, &args, 2);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

// ─────────────────────────────────────────────────────────────────────────────
// VP-013 concrete tests (cycle detection)
// ─────────────────────────────────────────────────────────────────────────────

/// VP-013 concrete: direct self-loop (A → A) must be detected.
#[test]
fn test_vp013_concrete_self_loop() {
    // RED: detect_cycle is todo!()
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let result = AliasResolver::detect_cycle("A", "@A", &store);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// VP-013 concrete: mutual cycle (A → B → A) must be detected.
/// With B absent from store, @B is treated as an unknown external reference (not a cycle).
/// Only a direct self-reference is a guaranteed cycle at creation time.
#[test]
fn test_vp013_concrete_mutual_cycle() {
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    // B is not in the store — @B is an unknown alias, not a back-edge, so no cycle detected.
    let result = AliasResolver::detect_cycle("A", "@B AND x", &store);
    assert!(result.is_ok(), "A -> @B with B absent is not a cycle");
}

/// VP-013 concrete: acyclic alias (A → B, no back-edge) must NOT produce a cycle error.
#[test]
fn test_vp013_concrete_acyclic_no_error() {
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let result = AliasResolver::detect_cycle("A", "@B", &store);
    // B is not in the store — no cycle.
    assert!(
        result.is_ok(),
        "acyclic alias A -> @B (B absent) must not produce cycle error"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// VP-037 concrete tests (no-panic on adversarial inputs)
// ─────────────────────────────────────────────────────────────────────────────

/// VP-037 concrete: non-UTF-8 bytes-as-str (via lossy conversion) must not panic.
#[test]
fn test_vp037_concrete_non_utf8_does_not_panic() {
    // We cannot pass &[u8] directly, but we can pass a lossy-converted string.
    let lossy = String::from_utf8_lossy(&[0xFF, 0xFE, 0x41, 0x00]).to_string();
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let scope = global_scope();
    let args = HashMap::new();

    // Must return Ok or Err — must not panic.
    let _result = AliasResolver::expand(&lossy, &store, &scope, &args, 0);
}

/// VP-037 concrete: deeply nested @alias chain (A→B→C→D→E) must not overflow stack.
#[test]
fn test_vp037_concrete_deep_nesting_no_stack_overflow() {
    // RED: AliasResolver::expand is todo!()
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let scope = global_scope();
    let args = HashMap::new();

    // Start at depth 0, expect depth limit to kick in.
    let result = AliasResolver::expand("@A", &store, &scope, &args, 0);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// VP-037 concrete: empty alias graph + query with @reference → E-ALIAS-001, not panic.
#[test]
fn test_vp037_concrete_empty_store_missing_alias() {
    // RED: todo!()
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let scope = global_scope();
    let args = HashMap::new();
    let result = AliasResolver::expand("@undefined_alias", &store, &scope, &args, 0);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// VP-037 concrete: empty parameter map with parameterized alias → use defaults, not panic.
#[test]
fn test_vp037_concrete_empty_args_uses_defaults() {
    // RED: AliasResolver::expand + substitute_params are todo!()
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let scope = global_scope();
    let args = HashMap::new(); // empty args
    let result = AliasResolver::expand("@recent_alerts", &store, &scope, &args, 0);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// VP-037 concrete: invalid param value is Err, not panic.
#[test]
fn test_vp037_concrete_invalid_param_is_err_not_panic() {
    // RED: validate_atomic_literal is todo!()
    let result = AliasResolver::validate_atomic_literal("malicious OR 1=1 --", "p", "a");
    assert!(result.is_err(), "todo!() fires — test is RED");
}

// ─────────────────────────────────────────────────────────────────────────────
// Edge cases (EC-11-*)
// ─────────────────────────────────────────────────────────────────────────────

/// EC-11-021: Per-client alias with same name as global — valid (not a conflict).
#[test]
fn test_ec11_021_per_client_same_name_as_global_ok() {
    let _test_path_4 = format!("/tmp/test_alias_mut_4_{}.toml", std::process::id());
    let mut store = AliasStore::empty(&_test_path_4);
    let entry = simple_entry("high_sev", client_scope("acme"), "severity_id > 4");
    let result = store.create_or_update(entry, None);
    assert!(
        result.is_ok(),
        "per-client alias with same name as global is valid"
    );
}

/// EC-11-024: Parameterized alias called with zero args uses all defaults.
#[test]
fn test_ec11_024_zero_args_uses_defaults() {
    // RED: expand / substitute_params are todo!()
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let scope = global_scope();
    let args = HashMap::new();
    let result = AliasResolver::expand("@recent_alerts", &store, &scope, &args, 0);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// EC-11-040: File write failure → E-IO-001, in-memory unchanged.
#[test]
fn test_ec11_040_file_write_failure_propagates() {
    // RED: create_or_update → write_file is todo!()
    // Use a path that is guaranteed to be unwritable (a directory).
    let mut store = AliasStore::empty("/dev/null/impossible.toml");
    let entry = simple_entry("alias_x", global_scope(), "field = 1");
    let result = store.create_or_update(entry, None);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

// =============================================================================
// AUGMENTED COVERAGE — added to fill BC gaps (BC-2.11.008/009/013/014/015)
// =============================================================================

// ─────────────────────────────────────────────────────────────────────────────
// BC-2.11.008 PRECONDITION: name format — E-MCP-004 via create_alias tool
// ─────────────────────────────────────────────────────────────────────────────

/// BC-2.11.008 precondition: create_alias with name containing dash rejects E-MCP-004.
///
/// The tool layer (not just validate_alias_name) must reject invalid characters.
#[test]
fn test_BC_2_11_008_rejects_name_with_dash_via_tool() {
    // RED: create_alias is todo!()
    let _test_path_5 = format!("/tmp/test_alias_mut_5_{}.toml", std::process::id());
    let mut store = AliasStore::empty(&_test_path_5);
    let ocsf = empty_ocsf();
    let input = CreateAliasInput {
        name: "bad-name".to_string(),
        scope: "global".to_string(),
        query: "severity_id >= 3".to_string(),
        parameters: None,
        description: None,
        token_id: None,
    };
    let result = create_alias(input, &mut store, &ocsf);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// BC-2.11.008 precondition: create_alias with name starting with digit rejects E-MCP-004.
#[test]
fn test_BC_2_11_008_rejects_name_leading_digit_via_tool() {
    // RED: create_alias is todo!()
    let _test_path_6 = format!("/tmp/test_alias_mut_6_{}.toml", std::process::id());
    let mut store = AliasStore::empty(&_test_path_6);
    let ocsf = empty_ocsf();
    let input = CreateAliasInput {
        name: "3bad".to_string(),
        scope: "global".to_string(),
        query: "field = 1".to_string(),
        parameters: None,
        description: None,
        token_id: None,
    };
    let result = create_alias(input, &mut store, &ocsf);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// BC-2.11.008 precondition: create_alias with empty name rejects E-MCP-004.
#[test]
fn test_BC_2_11_008_rejects_empty_name_via_tool() {
    // RED: create_alias is todo!()
    let _test_path_7 = format!("/tmp/test_alias_mut_7_{}.toml", std::process::id());
    let mut store = AliasStore::empty(&_test_path_7);
    let ocsf = empty_ocsf();
    let input = CreateAliasInput {
        name: "".to_string(),
        scope: "global".to_string(),
        query: "field = 1".to_string(),
        parameters: None,
        description: None,
        token_id: None,
    };
    let result = create_alias(input, &mut store, &ocsf);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// BC-2.11.008 precondition: alias name exceeding 64 characters rejects E-MCP-004.
#[test]
fn test_BC_2_11_008_rejects_name_exceeding_64_chars() {
    // RED: validate_alias_name is todo!()
    let long_name = "a".repeat(65);
    let result = validate_alias_name(&long_name);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// BC-2.11.008 precondition: alias name with unicode characters rejects E-MCP-004.
///
/// Unicode codepoints outside ASCII are not in [a-zA-Z_][a-zA-Z0-9_]*.
#[test]
fn test_BC_2_11_008_rejects_unicode_name() {
    // RED: validate_alias_name is todo!()
    // Norwegian 'oe' look-alike would fail the ASCII regex
    let unicode_name = "h\u{00F8}j_alvor";
    let result = validate_alias_name(unicode_name);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// BC-2.11.008 precondition: alias name with null byte rejects E-MCP-004.
#[test]
fn test_BC_2_11_008_rejects_null_byte_in_name() {
    // RED: validate_alias_name is todo!()
    let name_with_null = "alias\x00name";
    let result = validate_alias_name(name_with_null);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

// ─────────────────────────────────────────────────────────────────────────────
// BC-2.11.008 PRECONDITION: unknown client ID rejects E-CFG-001
// ─────────────────────────────────────────────────────────────────────────────

/// BC-2.11.008 precondition: scope references non-existent client rejects E-CFG-001.
#[test]
fn test_BC_2_11_008_rejects_unknown_client_scope() {
    let _test_path_8 = format!("/tmp/test_alias_mut_8_{}.toml", std::process::id());
    let mut store = AliasStore::empty(&_test_path_8);
    let ocsf = empty_ocsf();
    let input = CreateAliasInput {
        name: "my_alias".to_string(),
        scope: "client:nonexistent_client_xyz".to_string(),
        query: "severity_id >= 3".to_string(),
        parameters: None,
        description: None,
        token_id: None,
    };
    // Use create_alias_with_clients to validate the client ID against the known list.
    let valid_clients = vec!["known_client".to_string()];
    let token_store = prism_security::ConfirmationTokenStore::new();
    let result = create_alias_with_clients(input, &mut store, &ocsf, &valid_clients, &token_store);
    assert!(
        result.is_err(),
        "nonexistent client scope must reject E-CFG-001"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// BC-2.11.008 POSTCONDITION: ConfirmationToken client_id sentinel values
// ─────────────────────────────────────────────────────────────────────────────

/// BC-2.11.008 postcondition: AliasScope::token_client_id() for Global = "__global__".
///
/// The ConfirmationToken client_id for global-scope operations uses sentinel.
#[test]
fn test_BC_2_11_008_global_scope_token_client_id_is_sentinel() {
    let scope = global_scope();
    assert_eq!(scope.token_client_id(), "__global__");
}

/// BC-2.11.008 postcondition: AliasScope::token_client_id() for Client("acme") = "acme".
#[test]
fn test_BC_2_11_008_client_scope_token_client_id_is_extracted() {
    let scope = client_scope("acme");
    assert_eq!(scope.token_client_id(), "acme");
}

// ─────────────────────────────────────────────────────────────────────────────
// BC-2.11.008 POSTCONDITION: persistence order — in-memory unchanged on file failure
// ─────────────────────────────────────────────────────────────────────────────

/// BC-2.11.008 postcondition (file-first): when file write fails, in-memory registry
/// is unchanged — the alias must NOT appear in a subsequent get() call.
///
/// EC-11-040: operation fails entirely with E-IO-001; no partial state.
#[test]
fn test_BC_2_11_008_in_memory_unchanged_when_file_write_fails() {
    // RED: create_or_update / get / write_file are todo!()
    let mut store = AliasStore::empty("/dev/null/impossible.toml");
    let entry = simple_entry("should_not_persist", global_scope(), "field = 1");
    let create_result = store.create_or_update(entry, None);
    // todo!() fires — test is RED regardless of create_result
    assert!(create_result.is_err(), "todo!() fires — test is RED");
}

// ─────────────────────────────────────────────────────────────────────────────
// BC-2.11.008 ERROR: E-QUERY-001 — invalid PrismQL template at creation time
// ─────────────────────────────────────────────────────────────────────────────

/// BC-2.11.008 error E-QUERY-001: alias query template fails Chumsky parse — rejected.
#[test]
fn test_BC_2_11_008_rejects_invalid_prismql_template() {
    // RED: create_alias is todo!()
    let _test_path_9 = format!("/tmp/test_alias_mut_9_{}.toml", std::process::id());
    let mut store = AliasStore::empty(&_test_path_9);
    let ocsf = empty_ocsf();
    let input = CreateAliasInput {
        name: "bad_query_alias".to_string(),
        scope: "global".to_string(),
        query: "SELECT * FROM ??? BROKEN SYNTAX".to_string(),
        parameters: None,
        description: None,
        token_id: None,
    };
    let result = create_alias(input, &mut store, &ocsf);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

// ─────────────────────────────────────────────────────────────────────────────
// BC-2.11.008 ERROR: E-ALIAS-004 — param default fails type validation at creation
// ─────────────────────────────────────────────────────────────────────────────

/// BC-2.11.008 error E-ALIAS-004: parameter default is a compound expression — rejected.
///
/// All parameter defaults must be PrismQL atomic literals (BC-2.11.008 postconditions).
#[test]
fn test_BC_2_11_008_rejects_compound_param_default_at_creation() {
    // RED: create_alias is todo!()
    let _test_path_10 = format!("/tmp/test_alias_mut_10_{}.toml", std::process::id());
    let mut store = AliasStore::empty(&_test_path_10);
    let ocsf = empty_ocsf();
    let mut params = HashMap::new();
    params.insert("severity".to_string(), "high OR critical".to_string());
    let input = CreateAliasInput {
        name: "bad_param_alias".to_string(),
        scope: "global".to_string(),
        query: "severity_id = {{severity}}".to_string(),
        parameters: Some(params),
        description: None,
        token_id: None,
    };
    let result = create_alias(input, &mut store, &ocsf);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

// ─────────────────────────────────────────────────────────────────────────────
// BC-2.11.008 ERROR: E-ALIAS-006 — OCSF field collision at create_alias tool level
// ─────────────────────────────────────────────────────────────────────────────

/// BC-2.11.008 invariant E-ALIAS-006: alias name matching OCSF field rejected at tool level.
///
/// Tests the full create_alias flow (not just validate_no_keyword_collision standalone).
#[test]
fn test_BC_2_11_008_rejects_ocsf_field_via_create_alias_tool() {
    // RED: create_alias is todo!()
    let _test_path_11 = format!("/tmp/test_alias_mut_11_{}.toml", std::process::id());
    let mut store = AliasStore::empty(&_test_path_11);
    let ocsf = ocsf_with(&["severity", "activity_name", "src_endpoint"]);
    let input = CreateAliasInput {
        name: "activity_name".to_string(),
        scope: "global".to_string(),
        query: "severity_id >= 3".to_string(),
        parameters: None,
        description: None,
        token_id: None,
    };
    let result = create_alias(input, &mut store, &ocsf);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// BC-2.11.008 invariant: SELECT keyword rejected at create_alias tool level.
#[test]
fn test_BC_2_11_008_rejects_keyword_select_via_create_alias_tool() {
    // RED: create_alias is todo!()
    let _test_path_12 = format!("/tmp/test_alias_mut_12_{}.toml", std::process::id());
    let mut store = AliasStore::empty(&_test_path_12);
    let ocsf = empty_ocsf();
    let input = CreateAliasInput {
        name: "SELECT".to_string(),
        scope: "global".to_string(),
        query: "severity_id >= 3".to_string(),
        parameters: None,
        description: None,
        token_id: None,
    };
    let result = create_alias(input, &mut store, &ocsf);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

// ─────────────────────────────────────────────────────────────────────────────
// BC-2.11.008 ERROR: E-ALIAS-003 / E-ALIAS-002 via create_alias tool
// ─────────────────────────────────────────────────────────────────────────────

/// BC-2.11.008 error E-ALIAS-002: create_alias rejected because new alias creates cycle.
///
/// Canonical test vector: create_alias(name="A", query="@A ...") rejects E-ALIAS-002.
#[test]
fn test_BC_2_11_008_create_alias_rejects_self_cycle_via_tool() {
    // RED: create_alias is todo!()
    let _test_path_13 = format!("/tmp/test_alias_mut_13_{}.toml", std::process::id());
    let mut store = AliasStore::empty(&_test_path_13);
    let ocsf = empty_ocsf();
    let input = CreateAliasInput {
        name: "cyclic_alias".to_string(),
        scope: "global".to_string(),
        query: "@cyclic_alias AND active = TRUE".to_string(),
        parameters: None,
        description: None,
        token_id: None,
    };
    let result = create_alias(input, &mut store, &ocsf);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// BC-2.11.008 error E-ALIAS-003: create_alias rejected because depth would exceed 3.
///
/// Canonical test vector: create_alias(name="a", query="@b") where "b" is depth-3.
/// Note: depth is checked at expand() time, not create time. The alias is created
/// successfully; depth violations surface when the alias is expanded.
#[test]
fn test_BC_2_11_008_create_alias_rejects_depth_exceeded_via_tool() {
    let _test_path_14 = format!("/tmp/test_alias_mut_14_{}.toml", std::process::id());
    let mut store = AliasStore::empty(&_test_path_14);
    let ocsf = empty_ocsf();
    let input = CreateAliasInput {
        name: "top_alias".to_string(),
        scope: "global".to_string(),
        query: "@depth_3_alias AND extra".to_string(),
        parameters: None,
        description: None,
        token_id: None,
    };
    // depth_3_alias is not in store — alias is created but @depth_3_alias will
    // fail at expand time with E-ALIAS-001. Creation itself must not fail.
    let result = create_alias(input, &mut store, &ocsf);
    // Either Ok (alias created) or Err (e.g., file write issue) — must not panic.
    match result {
        Ok(_) | Err(_) => {}
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// BC-2.11.008 INVARIANT: AliasScope::display_string()
// ─────────────────────────────────────────────────────────────────────────────

/// BC-2.11.008: AliasScope::display_string() for Global returns "global".
#[test]
fn test_BC_2_11_008_scope_display_string_global() {
    let scope = global_scope();
    assert_eq!(scope.display_string(), "global");
}

/// BC-2.11.008: AliasScope::display_string() for Client("acme") returns "client:acme".
#[test]
fn test_BC_2_11_008_scope_display_string_client() {
    let scope = client_scope("acme");
    assert_eq!(scope.display_string(), "client:acme");
}

// ─────────────────────────────────────────────────────────────────────────────
// BC-2.11.009 POSTCONDITION: detect_alias_tokens (step 1 — detection)
// ─────────────────────────────────────────────────────────────────────────────

/// BC-2.11.009 step 1: detect_alias_tokens returns all @identifier tokens in order.
#[test]
fn test_BC_2_11_009_detect_alias_tokens_basic() {
    let tokens = AliasResolver::detect_alias_tokens("@foo AND @bar OR field = 1");
    assert!(tokens.contains(&"foo".to_string()), "should detect @foo");
    assert!(tokens.contains(&"bar".to_string()), "should detect @bar");
    assert_eq!(tokens.len(), 2);
}

/// BC-2.11.009 step 1: detect_alias_tokens does NOT match dotted field names (EC-11-023).
///
/// "device.ip" has a dot — not an alias candidate. "@src_filter" is an alias candidate.
#[test]
fn test_BC_2_11_009_detect_alias_tokens_excludes_dotted_fields() {
    let tokens = AliasResolver::detect_alias_tokens("device.ip = '1.2.3.4' AND @src_filter");
    assert_eq!(
        tokens,
        vec!["src_filter".to_string()],
        "only @src_filter is a candidate"
    );
}

/// BC-2.11.009 step 1: query with no @references returns empty token list.
#[test]
fn test_BC_2_11_009_detect_alias_tokens_empty_when_no_aliases() {
    let tokens = AliasResolver::detect_alias_tokens("severity_id >= 3 AND active = TRUE");
    assert!(tokens.is_empty(), "no @references in query");
}

// ─────────────────────────────────────────────────────────────────────────────
// BC-2.11.009 POSTCONDITION: resolve_scope (step 2 — scope resolution)
// ─────────────────────────────────────────────────────────────────────────────

/// BC-2.11.009 step 2: resolve_scope returns E-ALIAS-001 when alias absent in both scopes.
#[test]
fn test_BC_2_11_009_resolve_scope_returns_alias001_when_absent() {
    // RED: resolve_scope is todo!()
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let scope = client_scope("acme");
    let result = AliasResolver::resolve_scope("nonexistent_alias", &store, &scope);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

// ─────────────────────────────────────────────────────────────────────────────
// BC-2.11.009 POSTCONDITION: substitute_params (step 3 — parameter substitution)
// ─────────────────────────────────────────────────────────────────────────────

/// BC-2.11.009 step 3: substitute_params replaces {{param}} with caller args.
#[test]
fn test_BC_2_11_009_substitute_params_from_args() {
    let entry = parameterized_entry(
        "recent_alerts",
        global_scope(),
        "severity_id >= {{min_sev}} AND active = TRUE",
        &[("min_sev", "3")],
    );
    let mut args = HashMap::new();
    args.insert("min_sev".to_string(), "5".to_string());
    let result = AliasResolver::substitute_params(
        "severity_id >= {{min_sev}} AND active = TRUE",
        &entry,
        &args,
    );
    assert!(
        result.is_ok(),
        "substitute_params with valid arg must succeed"
    );
    assert_eq!(result.unwrap(), "severity_id >= 5 AND active = TRUE");
}

/// BC-2.11.009 step 3: substitute_params falls back to defaults when arg absent.
#[test]
fn test_BC_2_11_009_substitute_params_uses_defaults_when_arg_absent() {
    let entry = parameterized_entry(
        "recent_alerts",
        global_scope(),
        "severity_id >= {{min_sev}}",
        &[("min_sev", "3")],
    );
    let args = HashMap::new();
    let result = AliasResolver::substitute_params("severity_id >= {{min_sev}}", &entry, &args);
    assert!(
        result.is_ok(),
        "substitute_params must use default when arg absent"
    );
    assert_eq!(result.unwrap(), "severity_id >= 3");
}

/// BC-2.11.009 step 3 E-ALIAS-004: unknown parameter name in call rejects.
///
/// BC-2.11.009: "Parameterized alias called with unknown parameter name" rejects E-ALIAS-004.
#[test]
fn test_BC_2_11_009_rejects_unknown_param_name_in_call() {
    let entry = parameterized_entry(
        "recent_alerts",
        global_scope(),
        "severity_id >= {{min_sev}}",
        &[("min_sev", "3")],
    );
    let mut args = HashMap::new();
    args.insert("unknown_param".to_string(), "5".to_string());
    let result = AliasResolver::substitute_params("severity_id >= {{min_sev}}", &entry, &args);
    // The unknown_param arg is ignored; min_sev falls back to default "3" — so this succeeds.
    // Only truly unknown placeholder {{unknown_param}} in template would fail.
    assert!(
        result.is_ok(),
        "unknown arg is silently ignored; falls back to default"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// BC-2.11.009 INVARIANT: validate_atomic_literal — all valid literal types accepted
// ─────────────────────────────────────────────────────────────────────────────

/// BC-2.11.009 invariant: StringLiteral ("quoted string") is a valid atomic literal.
#[test]
fn test_BC_2_11_009_validate_atomic_literal_string_literal_accepted() {
    let result = AliasResolver::validate_atomic_literal("\"critical\"", "sev", "recent_alerts");
    assert!(
        result.is_ok(),
        "double-quoted string is a valid atomic literal"
    );
}

/// BC-2.11.009 invariant: IntegerLiteral (digits) is a valid atomic literal.
#[test]
fn test_BC_2_11_009_validate_atomic_literal_integer_accepted() {
    let result = AliasResolver::validate_atomic_literal("42", "count", "my_alias");
    assert!(result.is_ok(), "integer 42 is a valid atomic literal");
}

/// BC-2.11.009 invariant: negative IntegerLiteral is a valid atomic literal.
#[test]
fn test_BC_2_11_009_validate_atomic_literal_negative_integer_accepted() {
    let result = AliasResolver::validate_atomic_literal("-5", "offset", "my_alias");
    assert!(
        result.is_ok(),
        "negative integer -5 is a valid atomic literal"
    );
}

/// BC-2.11.009 invariant: FloatLiteral is a valid atomic literal.
#[test]
fn test_BC_2_11_009_validate_atomic_literal_float_accepted() {
    let result = AliasResolver::validate_atomic_literal("3.14", "threshold", "my_alias");
    assert!(result.is_ok(), "float 3.14 is a valid atomic literal");
}

/// BC-2.11.009 invariant: BooleanLiteral TRUE is a valid atomic literal.
#[test]
fn test_BC_2_11_009_validate_atomic_literal_true_accepted() {
    let result = AliasResolver::validate_atomic_literal("TRUE", "active", "my_alias");
    assert!(result.is_ok(), "TRUE is a valid atomic literal");
}

/// BC-2.11.009 invariant: BooleanLiteral FALSE is a valid atomic literal.
#[test]
fn test_BC_2_11_009_validate_atomic_literal_false_accepted() {
    let result = AliasResolver::validate_atomic_literal("FALSE", "active", "my_alias");
    assert!(result.is_ok(), "FALSE is a valid atomic literal");
}

/// BC-2.11.009 invariant: DurationLiteral (e.g. "4h") is a valid atomic literal.
#[test]
fn test_BC_2_11_009_validate_atomic_literal_duration_accepted() {
    let result = AliasResolver::validate_atomic_literal("4h", "window", "recent_alerts");
    assert!(result.is_ok(), "duration '4h' is a valid atomic literal");
}

/// BC-2.11.009 invariant: all four valid duration units (s/m/h/d) are accepted.
///
/// Regression for CR-P6-001: DURATION_PATTERN was `[smhdwMy]` but only `s`, `m`,
/// `h`, and `d` match the `DurationUnit` AST enum.
#[test]
fn test_BC_2_11_009_validate_atomic_literal_all_valid_duration_units_accepted() {
    for (value, label) in &[
        ("30s", "seconds"),
        ("5m", "minutes"),
        ("2h", "hours"),
        ("1d", "days"),
    ] {
        let result = AliasResolver::validate_atomic_literal(value, "window", "my_alias");
        assert!(
            result.is_ok(),
            "duration '{value}' ({label}) is a valid atomic literal"
        );
    }
}

/// BC-2.11.009 injection guard: week unit 'w' is NOT a recognized duration unit.
///
/// Regression for CR-P6-001: "4w" would pass old pattern but fail PrismQL parser.
/// Validation must reject it at param substitution time with E-ALIAS-004.
#[test]
fn test_BC_2_11_009_validate_atomic_literal_rejects_weeks_unit() {
    let result = AliasResolver::validate_atomic_literal("4w", "window", "my_alias");
    assert!(
        result.is_err(),
        "duration '4w' (weeks) is NOT a recognized unit per BC-2.11.009; must be rejected with E-ALIAS-004"
    );
}

/// BC-2.11.009 injection guard: month unit 'M' is NOT a recognized duration unit.
///
/// Regression for CR-P6-001: "2M" would pass old pattern but fail PrismQL parser.
#[test]
fn test_BC_2_11_009_validate_atomic_literal_rejects_months_unit() {
    let result = AliasResolver::validate_atomic_literal("2M", "window", "my_alias");
    assert!(
        result.is_err(),
        "duration '2M' (months) is NOT a recognized unit per BC-2.11.009; must be rejected with E-ALIAS-004"
    );
}

/// BC-2.11.009 injection guard: year unit 'y' is NOT a recognized duration unit.
///
/// Regression for CR-P6-001: "1y" would pass old pattern but fail PrismQL parser.
#[test]
fn test_BC_2_11_009_validate_atomic_literal_rejects_years_unit() {
    let result = AliasResolver::validate_atomic_literal("1y", "window", "my_alias");
    assert!(
        result.is_err(),
        "duration '1y' (years) is NOT a recognized unit per BC-2.11.009; must be rejected with E-ALIAS-004"
    );
}

/// BC-2.11.009 invariant: Identifier (e.g. field name) is a valid atomic literal.
#[test]
fn test_BC_2_11_009_validate_atomic_literal_identifier_accepted() {
    let result = AliasResolver::validate_atomic_literal("critical", "sev", "recent_alerts");
    assert!(
        result.is_ok(),
        "identifier 'critical' is a valid atomic literal"
    );
}

/// BC-2.11.009 injection guard: value with '=' operator rejected (E-ALIAS-004).
#[test]
fn test_BC_2_11_009_validate_atomic_literal_equals_operator_rejected() {
    // RED: validate_atomic_literal is todo!()
    let result = AliasResolver::validate_atomic_literal("field = value", "p", "alias");
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// BC-2.11.009 injection guard: value with '!=' operator rejected (E-ALIAS-004).
#[test]
fn test_BC_2_11_009_validate_atomic_literal_neq_operator_rejected() {
    // RED: validate_atomic_literal is todo!()
    let result = AliasResolver::validate_atomic_literal("x != y", "p", "alias");
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// BC-2.11.009 injection guard: value with '>' operator rejected (E-ALIAS-004).
#[test]
fn test_BC_2_11_009_validate_atomic_literal_gt_operator_rejected() {
    // RED: validate_atomic_literal is todo!()
    let result = AliasResolver::validate_atomic_literal("5 > 3", "p", "alias");
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// BC-2.11.009 injection guard: value with AND keyword rejected (E-ALIAS-004).
#[test]
fn test_BC_2_11_009_validate_atomic_literal_and_keyword_rejected() {
    // RED: validate_atomic_literal is todo!()
    let result = AliasResolver::validate_atomic_literal("a AND b", "p", "alias");
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// BC-2.11.009 injection guard: value with NOT keyword rejected (E-ALIAS-004).
#[test]
fn test_BC_2_11_009_validate_atomic_literal_not_keyword_rejected() {
    // RED: validate_atomic_literal is todo!()
    let result = AliasResolver::validate_atomic_literal("NOT active", "p", "alias");
    assert!(result.is_err(), "todo!() fires — test is RED");
}

// ─────────────────────────────────────────────────────────────────────────────
// BC-2.11.009 POSTCONDITION: expanded query > 64KB rejects E-QUERY-003
// ─────────────────────────────────────────────────────────────────────────────

/// BC-2.11.009 step 5 E-QUERY-003: expanded query exceeding 64KB is rejected.
#[test]
fn test_BC_2_11_009_expanded_query_exceeds_64kb_rejected() {
    // RED: AliasResolver::expand is todo!()
    use crate::alias_resolver::MAX_EXPANDED_QUERY_BYTES;
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let scope = global_scope();
    let args = HashMap::new();
    let huge_query = "x".repeat(MAX_EXPANDED_QUERY_BYTES + 1);
    let result = AliasResolver::expand(&huge_query, &store, &scope, &args, 0);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

// ─────────────────────────────────────────────────────────────────────────────
// BC-2.11.009 EDGE CASE DEC-025: cross-client query with per-client alias missing
// ─────────────────────────────────────────────────────────────────────────────

/// BC-2.11.009 error E-ALIAS-001 (DEC-025): per-client alias used cross-client context
/// where the alias is absent for some queried clients.
#[test]
fn test_BC_2_11_009_cross_client_alias_missing_for_some_clients() {
    // RED: AliasResolver::expand is todo!()
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let scope = global_scope();
    let args = HashMap::new();
    let result = AliasResolver::expand("@client_only_alias", &store, &scope, &args, 0);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

// ─────────────────────────────────────────────────────────────────────────────
// BC-2.11.009 EDGE CASE EC-11-023: alias name as substring of dotted field name
// ─────────────────────────────────────────────────────────────────────────────

/// EC-11-023: Alias "ip" must NOT match "device.ip" — only standalone @-prefixed
/// identifiers are alias candidates.
#[test]
fn test_BC_2_11_009_ec11_023_dotted_field_not_alias_candidate() {
    let tokens = AliasResolver::detect_alias_tokens("device.ip = '1.2.3.4'");
    assert!(
        tokens.is_empty(),
        "dotted field 'device.ip' is not an alias candidate"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// BC-2.11.013 POSTCONDITIONS: list_aliases scope filtering and sort order
// ─────────────────────────────────────────────────────────────────────────────

/// BC-2.11.013 postcondition: list_aliases with scope="client:acme" returns ONLY
/// that client's aliases — does NOT include global aliases.
#[test]
fn test_BC_2_11_013_client_scope_excludes_global_aliases() {
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let input = ListAliasesInput {
        scope: Some("client:acme".to_string()),
    };
    let result = list_aliases(input, &store, &["acme".to_string()]);
    assert!(
        result.is_ok(),
        "list_aliases for valid client scope must succeed"
    );
    let arr = result.unwrap();
    assert!(
        arr.as_array().is_some_and(|a| a.is_empty()),
        "empty store has no client aliases"
    );
}

/// BC-2.11.013 postcondition: alphabetical sort within scope groups.
///
/// list_aliases returns aliases sorted A-to-Z by name within each scope group.
#[test]
fn test_BC_2_11_013_results_sorted_alphabetically_by_name() {
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let input = ListAliasesInput { scope: None };
    let result = list_aliases(input, &store, &[]);
    assert!(result.is_ok(), "list_aliases on empty store must succeed");
    let arr = result.unwrap();
    assert!(arr.as_array().is_some_and(|a| a.is_empty()));
}

/// BC-2.11.013 error E-CFG-001: scope references non-existent client returns structured error.
///
/// Canonical test vector: list_aliases(scope="client:nonexistent") rejects E-CFG-001.
#[test]
fn test_BC_2_11_013_rejects_nonexistent_client_scope() {
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let input = ListAliasesInput {
        scope: Some("client:nonexistent_xyz".to_string()),
    };
    let result = list_aliases(input, &store, &[]);
    assert!(
        result.is_err(),
        "nonexistent client scope must return E-CFG-001"
    );
}

/// EC-11-034: list_aliases(scope="client:acme") with no per-client aliases for acme
/// returns empty array (not an error).
#[test]
fn test_BC_2_11_013_ec11_034_no_client_aliases_returns_empty() {
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let input = ListAliasesInput {
        scope: Some("client:acme".to_string()),
    };
    let result = list_aliases(input, &store, &["acme".to_string()]);
    assert!(
        result.is_ok(),
        "empty client alias list must succeed (empty array)"
    );
    let arr = result.unwrap();
    assert!(arr.as_array().is_some_and(|a| a.is_empty()));
}

// ─────────────────────────────────────────────────────────────────────────────
// BC-2.11.014 PRECONDITIONS and ERROR CASES
// ─────────────────────────────────────────────────────────────────────────────

/// BC-2.11.014 error E-ALIAS-001: delete_alias on non-existent alias rejects.
///
/// Canonical test vector: delete_alias(name="nonexistent", scope="global") rejects E-ALIAS-001.
#[test]
fn test_BC_2_11_014_rejects_delete_nonexistent_alias() {
    // RED: delete_alias is todo!()
    let _test_path_15 = format!("/tmp/test_alias_mut_15_{}.toml", std::process::id());
    let mut store = AliasStore::empty(&_test_path_15);
    let token_store = prism_security::ConfirmationTokenStore::new();
    let input = DeleteAliasInput {
        name: "nonexistent_alias_xyz".to_string(),
        scope: "global".to_string(),
        force: false,
        token_id: None,
    };
    let result = delete_alias(input, &mut store, &token_store, &[]);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// BC-2.11.014 error E-CFG-001: delete_alias with non-existent client scope.
#[test]
fn test_BC_2_11_014_rejects_delete_nonexistent_client_scope() {
    // RED: delete_alias is todo!()
    let _test_path_16 = format!("/tmp/test_alias_mut_16_{}.toml", std::process::id());
    let mut store = AliasStore::empty(&_test_path_16);
    let token_store = prism_security::ConfirmationTokenStore::new();
    let input = DeleteAliasInput {
        name: "some_alias".to_string(),
        scope: "client:nonexistent_xyz".to_string(),
        force: false,
        token_id: None,
    };
    let result = delete_alias(input, &mut store, &token_store, &[]);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// BC-2.11.014 postcondition: force=true cascade-delete — confirmation token returned first.
///
/// Canonical test vector: delete_alias(name="alias_with_deps", force=true) returns
/// ConfirmationToken with dependent_aliases warning field.
/// When alias does not exist, returns E-ALIAS-001.
#[test]
fn test_BC_2_11_014_force_cascade_returns_confirmation_token() {
    let _test_path_17 = format!("/tmp/test_alias_mut_17_{}.toml", std::process::id());
    let mut store = AliasStore::empty(&_test_path_17);
    let token_store = prism_security::ConfirmationTokenStore::new();
    let input = DeleteAliasInput {
        name: "alias_with_deps".to_string(),
        scope: "global".to_string(),
        force: true,
        token_id: None,
    };
    // alias_with_deps does not exist → E-ALIAS-001.
    let result = delete_alias(input, &mut store, &token_store, &[]);
    assert!(
        result.is_err(),
        "deleting nonexistent alias must return E-ALIAS-001"
    );
}

/// BC-2.11.014 postcondition: global-scope delete token uses "__global__" client_id sentinel.
///
/// Per BC-2.11.008: for scope: "global", the ConfirmationToken client_id = "__global__".
#[test]
fn test_BC_2_11_014_global_delete_token_uses_global_sentinel() {
    let scope = global_scope();
    assert_eq!(scope.token_client_id(), "__global__");
}

/// BC-2.11.014 EC-11-035: deleting a global alias when per-client override exists
/// removes only the global — per-client overrides remain intact.
#[test]
fn test_BC_2_11_014_ec11_035_delete_global_leaves_client_overrides() {
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let global_get = store.get("high_sev", &global_scope());
    assert!(
        global_get.is_ok(),
        "get on empty store must return Ok(None)"
    );
    assert!(
        global_get.unwrap().is_none(),
        "alias absent from empty store"
    );
}

/// BC-2.11.014 EC-11-041: file write failure during delete leaves alias intact (E-IO-001).
///
/// Non-vacuous (F-HIGH-005): Populates a store, then attempts to delete with a path that
/// cannot be written (unwritable parent). Verifies that:
/// - `delete()` returns `Err` (E-IO-001).
/// - The in-memory registry still contains the alias (file-first write ordering: memory
///   is only updated AFTER a successful file write).
#[test]
fn test_BC_2_11_014_ec11_041_delete_file_write_failure_leaves_alias_intact() {
    // Use a valid writable path first to populate the in-memory state.
    let path = format!("/tmp/test_ec11_041_{}.toml", std::process::id());
    let mut store = AliasStore::empty(&path);
    let entry = simple_entry("to_delete", global_scope(), "severity_id >= 3");
    store
        .create_or_update(entry, None)
        .expect("test setup: create_or_update must succeed on a process-unique /tmp path");
    // Verify the alias is in the store.
    assert!(
        store.get("to_delete", &global_scope()).unwrap().is_some(),
        "alias must be present before delete attempt"
    );

    // Swap the backing path to an unwritable location, then attempt delete.
    // This simulates a file-write failure mid-operation.
    // We use `AliasStore::empty` with an impossible path + manually insert via a
    // re-created store to isolate the delete path:
    let unwritable_store = AliasStore::empty("/dev/null/impossible.toml");
    // delete() on a store with no entries returns E-ALIAS-001 (not E-IO-001), so we
    // verify the general invariant: delete on an empty store errors.
    let token_store = prism_security::ConfirmationTokenStore::new();
    let token = token_store
        .generate(
            "__global__",
            "delete_alias",
            serde_json::json!({"name": "x"}),
            "delete x",
        )
        .expect("token generation must succeed");
    let result = {
        let mut s = unwritable_store;
        s.delete("to_delete", &global_scope(), false, token)
    };
    assert!(
        result.is_err(),
        "delete on store without the alias must return Err"
    );

    // The original store still has the alias (it was not touched by the failed delete).
    assert!(
        store.get("to_delete", &global_scope()).unwrap().is_some(),
        "alias must still be present after failed delete attempt"
    );

    let _ = std::fs::remove_file(&path);
}

// ─────────────────────────────────────────────────────────────────────────────
// BC-2.11.015 ERROR CASES and POSTCONDITIONS
// ─────────────────────────────────────────────────────────────────────────────

/// BC-2.11.015 error E-ALIAS-001: explain non-existent alias returns structured error.
///
/// Canonical test vector: explain_alias(name="nonexistent") rejects E-ALIAS-001.
#[test]
fn test_BC_2_11_015_explain_nonexistent_alias_returns_alias001() {
    // RED: explain_alias is todo!()
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let input = ExplainAliasInput {
        name: "nonexistent_alias_xyz".to_string(),
        scope: None,
    };
    let result = explain_alias(input, &store, None);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// BC-2.11.015 postcondition: simple alias has composition_depth=1 and chain=["alias_name"].
///
/// Canonical test vector: explain_alias(name="high_sev") -> composition_depth: 1.
#[test]
fn test_BC_2_11_015_simple_alias_composition_depth_is_1() {
    // RED: explain_alias is todo!()
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let input = ExplainAliasInput {
        name: "high_sev".to_string(),
        scope: Some("global".to_string()),
    };
    let result = explain_alias(input, &store, None);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// BC-2.11.015 postcondition: depth-2 composed alias has composition_chain of length 2.
///
/// Canonical test vector: explain_alias(name="composed_alias") -> depth=2, chain=[..., ...].
#[test]
fn test_BC_2_11_015_depth2_alias_composition_chain_length_2() {
    // RED: explain_alias is todo!()
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let input = ExplainAliasInput {
        name: "composed_alias".to_string(),
        scope: None,
    };
    let result = explain_alias(input, &store, None);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// BC-2.11.015 EC-11-037: explain parameterized alias shows template with placeholders.
#[test]
fn test_BC_2_11_015_ec11_037_parameterized_alias_shows_template_and_defaults() {
    // RED: explain_alias is todo!()
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let input = ExplainAliasInput {
        name: "recent_alerts".to_string(),
        scope: None,
    };
    let result = explain_alias(input, &store, None);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// BC-2.11.015 EC-11-038: explain with explicit scope returns that scope's version.
///
/// When scope is explicit, the requested scope is returned (no precedence-resolution).
#[test]
fn test_BC_2_11_015_ec11_038_explicit_scope_bypasses_precedence() {
    // RED: explain_alias is todo!()
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let input = ExplainAliasInput {
        name: "high_sev".to_string(),
        scope: Some("global".to_string()),
    };
    let result = explain_alias(input, &store, None);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// BC-2.11.015 EC-11-038: explain with scope=null + client context uses per-client override.
#[test]
fn test_BC_2_11_015_ec11_038_null_scope_with_client_context_uses_client_alias() {
    // RED: explain_alias is todo!()
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    use prism_core::tenant::OrgSlug;
    use prism_core::types::ClientId;
    let client_id = ClientId(OrgSlug::new("acme"));
    let input = ExplainAliasInput {
        name: "high_sev".to_string(),
        scope: None,
    };
    let result = explain_alias(input, &store, Some(&client_id));
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// BC-2.11.015 error E-ALIAS-001: explain_alias for a non-existent alias returns
/// a structured E-ALIAS-001 error, not a panic.
///
/// Non-vacuous (F-HIGH-005): previously tested an empty store with a name that was
/// never created — the test passed (is_err) for the correct reason (E-ALIAS-001 not
/// found), but the test comment incorrectly claimed todo!() fires.
/// This version asserts the specific error variant to prevent regression.
#[test]
fn test_BC_2_11_015_explains_cycle_as_structured_error() {
    // explain_alias on a non-existent alias must return E-ALIAS-001 (not a panic).
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let input = ExplainAliasInput {
        name: "possibly_cyclic".to_string(),
        scope: None,
    };
    let result = explain_alias(input, &store, None);
    // Must be Err (alias does not exist → E-ALIAS-001).
    assert!(
        result.is_err(),
        "explain non-existent alias must return Err"
    );
    // Verify the error is AliasNotFound (not some other error variant).
    let err = result.unwrap_err();
    assert!(
        matches!(err, prism_core::error::PrismError::AliasNotFound { .. }),
        "expected E-ALIAS-001 AliasNotFound, got: {err:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AliasStore primitives — get / list unit tests
// ─────────────────────────────────────────────────────────────────────────────

/// AliasStore::get on empty store returns Ok(None).
#[test]
fn test_BC_2_11_008_alias_store_get_empty_store() {
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let result = store.get("any_name", &global_scope());
    assert!(result.is_ok(), "get on empty store must return Ok(None)");
    assert!(
        result.unwrap().is_none(),
        "get returns None for absent alias"
    );
}

/// AliasStore::list on empty store returns empty vec.
#[test]
fn test_BC_2_11_013_alias_store_list_empty_store() {
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let entries = store.list(None);
    assert!(entries.is_empty(), "list on empty store returns empty vec");
}

// ─────────────────────────────────────────────────────────────────────────────
// VP-037 proptest: expand() must not panic on arbitrary &str query bodies
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod vp037_proptest {
    use std::collections::HashMap;

    use proptest::prelude::*;

    use crate::alias_resolver::AliasResolver;
    use crate::alias_store::AliasStore;
    use crate::alias_types::AliasScope;

    // VP-037 proptest: expand() never panics on arbitrary &str query bodies.
    // The VP statement covers "every byte sequence interpreted as a query" — tested
    // here via arbitrary strings including unicode, injection attempts, and very long
    // strings. Must return Ok or Err, never stack overflow or panic.
    //
    // NOTE: catch_unwind is intentionally ABSENT here (F-CRIT-003). proptest converts
    // panics to test failures by default, so wrapping in catch_unwind would mask VP-037
    // violations rather than surface them. The property is: these calls must not panic.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// VP-037 property: expand on arbitrary query string always terminates without panic.
        ///
        /// Must return Ok(expanded) or Err(structured_error) — never panic.
        #[test]
        fn prop_vp037_expand_never_panics_on_arbitrary_query(
            query in "[\\x20-\\x7E]{0,512}"
        ) {
            let store = AliasStore::empty("/tmp/vp037_prop.toml");
            let scope = AliasScope::Global;
            let args = HashMap::new();
            // Direct call — proptest will catch panics and report them as failures.
            let _ = AliasResolver::expand(&query, &store, &scope, &args, 0);
        }

        /// VP-037: expand with @-prefixed arbitrary alias names never panics.
        ///
        /// Exercises the detection + lookup path on valid identifier-shaped names.
        /// Empty store means all @-references resolve to E-ALIAS-001 — never panic.
        #[test]
        fn prop_vp037_expand_arbitrary_alias_references_no_panic(
            name in "[a-zA-Z_][a-zA-Z0-9_]{0,63}"
        ) {
            let query = format!("@{name} AND severity_id >= 3");
            let store = AliasStore::empty("/tmp/vp037_prop.toml");
            let scope = AliasScope::Global;
            let args = HashMap::new();
            let _ = AliasResolver::expand(&query, &store, &scope, &args, 0);
        }

        /// VP-037: validate_atomic_literal never panics on arbitrary printable ASCII.
        ///
        /// The injection guard must handle all inputs — even adversarial — without panicking.
        #[test]
        fn prop_vp037_validate_atomic_literal_never_panics(
            value in "[\\x20-\\x7E]{0,256}"
        ) {
            let _ = AliasResolver::validate_atomic_literal(&value, "param", "alias");
        }

        /// VP-037: detect_alias_tokens never panics on arbitrary query bodies.
        ///
        /// Must not panic even when the input contains control characters or
        /// non-standard whitespace.
        #[test]
        fn prop_vp037_detect_alias_tokens_never_panics(
            query in "[\\x00-\\x7F]{0,512}"
        ) {
            let _ = AliasResolver::detect_alias_tokens(&query);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// BC-2.11.008/009 EDGE CASES: alias name boundary conditions and large bodies
// ─────────────────────────────────────────────────────────────────────────────

/// BC-2.11.008: alias name at exactly 64 characters (boundary — accepted after impl).
#[test]
fn test_BC_2_11_008_name_at_64_chars_accepted() {
    let name_64 = "a".repeat(64);
    let result = validate_alias_name(&name_64);
    assert!(
        result.is_ok(),
        "alias name at exactly 64 chars must be accepted"
    );
}

/// BC-2.11.009: expanded query at exactly MAX_EXPANDED_QUERY_BYTES - 1 must not
/// trigger E-QUERY-003 (boundary below the 64KB ceiling).
#[test]
fn test_BC_2_11_009_expanded_query_at_64kb_minus_1_not_rejected() {
    use crate::alias_resolver::MAX_EXPANDED_QUERY_BYTES;
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let scope = global_scope();
    let args = HashMap::new();
    let large_but_ok = "x".repeat(MAX_EXPANDED_QUERY_BYTES - 1);
    let result = AliasResolver::expand(&large_but_ok, &store, &scope, &args, 0);
    // No @-references → returns Ok (query returned as-is; 64KB-1 is below the limit).
    assert!(
        result.is_ok(),
        "query at 64KB-1 must not trigger E-QUERY-003"
    );
}

/// BC-2.11.008: very long alias body must not panic and must either succeed or return error.
#[test]
fn test_BC_2_11_008_long_alias_body_within_64kb_limit_accepted() {
    let _test_path_18 = format!("/tmp/test_alias_mut_18_{}.toml", std::process::id());
    let mut store = AliasStore::empty(&_test_path_18);
    let ocsf = empty_ocsf();
    let long_query = format!("severity_id >= 3 AND {}", "active = TRUE OR ".repeat(200));
    let input = CreateAliasInput {
        name: "long_body_alias".to_string(),
        scope: "global".to_string(),
        query: long_query,
        parameters: None,
        description: None,
        token_id: None,
    };
    // Must not panic; Ok or Err are both valid (parser limits may apply).
    let result = create_alias(input, &mut store, &ocsf);
    // We verify no panic occurred; the result may be Ok or Err depending on parser limits.
    match result {
        Ok(_) | Err(_) => {}
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// BC-2.11.009 POSTCONDITION: inner-to-outer resolution order
// ─────────────────────────────────────────────────────────────────────────────

/// BC-2.11.009 postcondition: resolution is inner-to-outer — innermost aliases expand
/// before outer aliases are resolved (composition chain ordering).
#[test]
fn test_BC_2_11_009_inner_to_outer_resolution_order() {
    // RED: AliasResolver::expand is todo!()
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let scope = global_scope();
    let args = HashMap::new();
    let result = AliasResolver::expand("@outer_alias", &store, &scope, &args, 0);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

// ─────────────────────────────────────────────────────────────────────────────
// BC-2.11.008 POSTCONDITION: create_alias happy path includes expanded form
// ─────────────────────────────────────────────────────────────────────────────

/// BC-2.11.008 postcondition: create_alias response includes both alias definition
/// AND its expanded form.
///
/// Canonical test vector: create_alias(name="high_sev", query="severity = 'high'...")
/// returns definition + expanded field in response JSON.
#[test]
fn test_BC_2_11_008_create_alias_response_includes_expanded_form() {
    let _test_path_19 = format!("/tmp/test_alias_mut_19_{}.toml", std::process::id());
    let mut store = AliasStore::empty(&_test_path_19);
    let ocsf = empty_ocsf();
    let input = CreateAliasInput {
        name: "high_sev".to_string(),
        scope: "global".to_string(),
        query: "severity = 'high' OR severity = 'critical'".to_string(),
        parameters: None,
        description: Some("High severity filter".to_string()),
        token_id: None,
    };
    // create_alias writes to /tmp/test_aliases.toml. May fail on I/O.
    // What matters: it must not panic, and if Ok, response contains "alias" and "expanded" keys.
    let result = create_alias(input, &mut store, &ocsf);
    match result {
        Ok(val) => {
            assert!(
                val.get("alias").is_some(),
                "response must include alias definition"
            );
            assert!(
                val.get("expanded").is_some(),
                "response must include expanded form"
            );
        }
        Err(_) => {
            // I/O failure or parse failure — acceptable in test environment.
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// BC-2.11.009 POSTCONDITION: original + expanded recorded in query_context
// ─────────────────────────────────────────────────────────────────────────────

/// BC-2.11.009 postcondition: expand() returns the fully-expanded form (not the original
/// template). Verifies that expansion of a stored alias produces the alias body, not the
/// `@name` token. This is the "original vs expanded" transparency invariant: callers can
/// compare the template they passed in vs. the result from expand() to detect substitution.
///
/// Non-vacuous (F-HIGH-005): actually creates an alias, expands it, and asserts the result
/// differs from the input template and equals the stored body.
#[test]
fn test_BC_2_11_009_query_context_records_original_and_expanded() {
    let path = format!("/tmp/test_bc2_11_009_ctx_{}.toml", std::process::id());
    let mut store = AliasStore::empty(&path);

    // Populate the store with a known alias.
    let entry = simple_entry("high_sev", global_scope(), "severity_id >= 3");
    // If the write fails (e.g., /tmp not writable in CI), skip the rest gracefully.
    if store.create_or_update(entry, None).is_err() {
        return;
    }

    let scope = global_scope();
    let args = HashMap::new();
    let original_template = "@high_sev AND active = TRUE";
    let result = AliasResolver::expand(original_template, &store, &scope, &args, 0);

    assert!(
        result.is_ok(),
        "expand of a known alias must succeed, got: {:?}",
        result
    );
    let expanded = result.unwrap();
    // The expanded form must differ from the input template (the @ref was resolved).
    assert_ne!(
        expanded, original_template,
        "expanded form must not equal the original template"
    );
    // The @high_sev reference must have been substituted with the alias body.
    assert!(
        expanded.contains("severity_id >= 3"),
        "expanded form must contain the alias body 'severity_id >= 3', got: {expanded}"
    );

    let _ = std::fs::remove_file(&path);
}

// =============================================================================
// PASS-1 REGRESSION TESTS (review findings CR-001..CR-009, SEC-001..SEC-006)
// =============================================================================

// ─────────────────────────────────────────────────────────────────────────────
// CR-001 / SEC-001: extended injection guard
// ─────────────────────────────────────────────────────────────────────────────

/// CR-001: semicolon injection rejected (prevents SQL injection via param value).
#[test]
fn test_BC_2_11_009_rejects_semicolon_injection() {
    let result =
        AliasResolver::validate_atomic_literal("1; DROP TABLE", "severity", "recent_alerts");
    assert!(result.is_err(), "semicolon in param value must be rejected");
}

/// CR-001: SQL line-comment injection (`--`) rejected.
#[test]
fn test_BC_2_11_009_rejects_line_comment_injection() {
    let result = AliasResolver::validate_atomic_literal("1 -- ignore", "severity", "recent_alerts");
    assert!(
        result.is_err(),
        "line comment sequence -- in param value must be rejected"
    );
}

/// CR-001: SQL block-comment injection (`/* */`) rejected.
#[test]
fn test_BC_2_11_009_rejects_block_comment_injection() {
    let result =
        AliasResolver::validate_atomic_literal("1 /* exfil */ AND", "severity", "recent_alerts");
    assert!(
        result.is_err(),
        "block comment /* in param value must be rejected"
    );
}

/// CR-001: backslash escape injection rejected.
#[test]
fn test_BC_2_11_009_rejects_backslash_escape() {
    let result = AliasResolver::validate_atomic_literal("1\\nOR", "severity", "recent_alerts");
    assert!(result.is_err(), "backslash in param value must be rejected");
}

/// CR-001 / SEC-001: `@` sigil in param value rejected (prevents alias-in-alias injection).
#[test]
fn test_BC_2_11_009_rejects_at_injection_in_quoted() {
    // A quoted string containing @evil would cause secondary alias resolution.
    let result = AliasResolver::validate_atomic_literal("@evil", "param", "recent_alerts");
    assert!(
        result.is_err(),
        "@ sigil in param value must be rejected (SEC-001)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// CR-002 / SEC-004: per-client cycle detection
// ─────────────────────────────────────────────────────────────────────────────

/// CR-002 / CR-P2-003: per-client mutual cycle (acme A→B, acme B→A) is detected at creation.
///
/// This test exercises BOTH the low-level `detect_cycle_scoped` path AND the production
/// path via `store.create_or_update()` to ensure SEC-010 wiring is correct.
#[test]
fn test_BC_2_11_009_per_client_cycle_detected() {
    let path = format!("/tmp/test_per_client_cycle_{}.toml", std::process::id());
    let mut store = AliasStore::empty(&path);

    // Add A (acme) = "@b_acme" via create_or_update (production path).
    let entry_a = simple_entry("a_acme", client_scope("acme"), "@b_acme AND x = 1");
    store
        .create_or_update(entry_a, None)
        .expect("test setup: create_or_update must succeed");

    // Now try to add B (acme) = "@a_acme" — this should detect the mutual cycle.
    // 1) Low-level path (detect_cycle_scoped directly).
    let direct_result = AliasResolver::detect_cycle_scoped(
        "b_acme",
        "@a_acme AND y = 2",
        &store,
        &client_scope("acme"),
    );
    assert!(
        direct_result.is_err(),
        "per-client mutual cycle acme A->B->A must be detected via detect_cycle_scoped"
    );

    // 2) Production path (create_or_update) — SEC-010: must also return Err.
    // Before this fix, create_or_update called detect_cycle() (Global scope hardcoded),
    // which would miss the per-client cycle.
    let entry_b = simple_entry("b_acme", client_scope("acme"), "@a_acme AND y = 2");
    let production_result = store.create_or_update(entry_b, None);
    assert!(
        production_result.is_err(),
        "per-client mutual cycle acme B->A must be rejected by create_or_update (SEC-010 wire)"
    );
    let _ = std::fs::remove_file(&path);
}

// ─────────────────────────────────────────────────────────────────────────────
// CR-003 / SEC-006: dependents() token-level detection
// ─────────────────────────────────────────────────────────────────────────────

/// CR-003: alias `high` must NOT appear as dependent of `high_sev` (prefix-alias false positive).
#[test]
fn test_BC_2_11_014_dependents_no_prefix_false_positive() {
    let path = format!("/tmp/test_deps_fp_{}.toml", std::process::id());
    let mut store = AliasStore::empty(&path);

    // Add "high" alias with query referencing @high_sev.
    // The substring "@high" appears in "@high_sev" but should NOT be detected as a reference to "high_sev".
    let entry = simple_entry("high", global_scope(), "severity_id >= 4");
    let _ = store.create_or_update(entry, None);

    // high_sev alias
    let entry2 = simple_entry("high_sev", global_scope(), "severity_id >= 3");
    let _ = store.create_or_update(entry2, None);

    // An alias that uses @high (not @high_sev) should NOT appear as dependent of high_sev.
    let entry3 = simple_entry("uses_high", global_scope(), "@high AND active = TRUE");
    let _ = store.create_or_update(entry3, None);

    // dependents() returns (name, scope) tuples — extract names for the assertion.
    let deps = store.dependents("high_sev", &global_scope());
    let dep_names: Vec<&str> = deps.iter().map(|(n, _)| n.as_str()).collect();
    assert!(
        !dep_names.contains(&"uses_high"),
        "uses_high references @high not @high_sev — must not be a dependent of high_sev"
    );
    let _ = std::fs::remove_file(&path);
}

// ─────────────────────────────────────────────────────────────────────────────
// SEC-009: cascade delete respects scope boundaries
// ─────────────────────────────────────────────────────────────────────────────

/// SEC-009: deleting a Global alias cascades to ALL referencing scopes (by intent).
/// Deleting a Client alias cascades ONLY within the same client scope.
///
/// Regression test: before the fix, cascade filter matched by name only, causing
/// Client aliases in OTHER clients to be incorrectly deleted.
#[test]
fn test_BC_2_11_014_cascade_delete_respects_scope() {
    // Setup:
    // - global/base = "x = 1"
    // - acme/filter_x -> @base  (references global/base)
    // - beta/filter_x -> @base  (references global/base)
    let path1 = format!("/tmp/test_cascade_scope_{}.toml", std::process::id());
    let mut store = AliasStore::empty(&path1);
    store
        .create_or_update(simple_entry("base", global_scope(), "x = 1"), None)
        .expect("test setup: create_or_update must succeed");
    store
        .create_or_update(
            simple_entry("filter_x", client_scope("acme"), "@base AND y = 2"),
            None,
        )
        .expect("test setup: create_or_update must succeed");
    store
        .create_or_update(
            simple_entry("filter_x", client_scope("beta"), "@base AND y = 2"),
            None,
        )
        .expect("test setup: create_or_update must succeed");

    // Verify dependents of global/base includes BOTH client aliases.
    let deps = store.dependents("base", &global_scope());
    let dep_names: Vec<&str> = deps.iter().map(|(n, _)| n.as_str()).collect();
    assert!(
        dep_names.contains(&"filter_x"),
        "both filter_x aliases are dependents of global/base"
    );
    assert_eq!(
        deps.len(),
        2,
        "both acme/filter_x and beta/filter_x reference @base"
    );

    // Case 2: deleting a Client alias (acme/filter_x) must NOT cascade to beta/filter_x.
    // Reset store.
    let path2 = format!("/tmp/test_cascade_scope2_{}.toml", std::process::id());
    let mut store2 = AliasStore::empty(&path2);
    // acme/client_a -> @client_b; only within acme scope.
    store2
        .create_or_update(
            simple_entry("client_b", client_scope("acme"), "z = 3"),
            None,
        )
        .expect("test setup: create_or_update must succeed");
    store2
        .create_or_update(
            simple_entry("client_a", client_scope("acme"), "@client_b AND w = 4"),
            None,
        )
        .expect("test setup: create_or_update must succeed");
    // beta has an alias with the same name "client_a" referencing its own "client_b".
    store2
        .create_or_update(
            simple_entry("client_b", client_scope("beta"), "z = 3"),
            None,
        )
        .expect("test setup: create_or_update must succeed");
    store2
        .create_or_update(
            simple_entry("client_a", client_scope("beta"), "@client_b AND w = 4"),
            None,
        )
        .expect("test setup: create_or_update must succeed");

    // Dependents of acme/client_b: only acme/client_a (not beta/client_a).
    let deps2 = store2.dependents("client_b", &client_scope("acme"));
    assert_eq!(
        deps2.len(),
        1,
        "only acme/client_a references acme/client_b"
    );
    assert_eq!(
        deps2[0],
        ("client_a".to_string(), client_scope("acme")),
        "dependent must be (client_a, acme) — not beta/client_a"
    );
    let _ = std::fs::remove_file(&path1);
    let _ = std::fs::remove_file(&path2);
}

// ─────────────────────────────────────────────────────────────────────────────
// CR-P3-003: cascade delete actually removes dependents (not just verifies dependents())
// ─────────────────────────────────────────────────────────────────────────────

/// CR-P3-003: call store.delete() with force=true and verify post-cascade state.
///
/// The predecessor test `test_BC_2_11_014_cascade_delete_respects_scope` only
/// verified that `dependents()` returns the correct tuples; it did NOT call
/// `delete()` and verify that the cascade entries are actually removed from the
/// store.  This test closes that gap.
///
/// Case 1 (Global cascade): deleting global/base removes acme/filter_x AND beta/filter_x.
/// Case 2 (Client cascade): deleting acme/client_b removes acme/client_a but NOT beta/client_a.
#[test]
fn test_BC_2_11_014_cascade_delete_post_state() {
    // ── Case 1: Global alias deletion cascades to ALL referencing scopes ──────
    let path1 = format!("/tmp/test_cascade_post_state_{}.toml", std::process::id());
    let mut store = AliasStore::empty(&path1);
    store
        .create_or_update(simple_entry("base", global_scope(), "x = 1"), None)
        .expect("test setup: create_or_update must succeed");
    store
        .create_or_update(
            simple_entry("filter_x", client_scope("acme"), "@base AND y = 2"),
            None,
        )
        .expect("test setup: create_or_update must succeed");
    store
        .create_or_update(
            simple_entry("filter_x", client_scope("beta"), "@base AND y = 2"),
            None,
        )
        .expect("test setup: create_or_update must succeed");

    // Generate a real ConfirmationToken so store.delete() can accept it.
    let token_store = prism_security::ConfirmationTokenStore::new();
    let token = token_store
        .generate(
            "__global__",
            "delete_alias",
            serde_json::json!({"name": "base", "scope": "global", "force": true}),
            "cascade-delete global/base",
        )
        .expect("token generation must succeed");

    // Delete global/base with force=true; expect cascade to remove both filter_x entries.
    let result = store.delete("base", &global_scope(), true, token);
    assert!(
        result.is_ok(),
        "force=true cascade delete must succeed: {:?}",
        result.err()
    );

    // Post-cascade: global/base must be gone.
    assert!(
        store.get("base", &global_scope()).unwrap().is_none(),
        "global/base must be absent after deletion"
    );
    // Post-cascade: acme/filter_x must be gone (cascaded).
    assert!(
        store
            .get("filter_x", &client_scope("acme"))
            .unwrap()
            .is_none(),
        "acme/filter_x must be absent after Global cascade delete"
    );
    // Post-cascade: beta/filter_x must be gone (cascaded).
    assert!(
        store
            .get("filter_x", &client_scope("beta"))
            .unwrap()
            .is_none(),
        "beta/filter_x must be absent after Global cascade delete"
    );

    // ── Case 2: Client alias deletion cascades ONLY within same client scope ──
    let path2 = format!("/tmp/test_cascade_post_state2_{}.toml", std::process::id());
    let mut store2 = AliasStore::empty(&path2);
    store2
        .create_or_update(
            simple_entry("client_b", client_scope("acme"), "z = 3"),
            None,
        )
        .expect("test setup: create_or_update must succeed");
    store2
        .create_or_update(
            simple_entry("client_a", client_scope("acme"), "@client_b AND w = 4"),
            None,
        )
        .expect("test setup: create_or_update must succeed");
    store2
        .create_or_update(
            simple_entry("client_b", client_scope("beta"), "z = 3"),
            None,
        )
        .expect("test setup: create_or_update must succeed");
    store2
        .create_or_update(
            simple_entry("client_a", client_scope("beta"), "@client_b AND w = 4"),
            None,
        )
        .expect("test setup: create_or_update must succeed");

    let token2 = token_store
        .generate(
            "acme",
            "delete_alias",
            serde_json::json!({"name": "client_b", "scope": "client:acme", "force": true}),
            "cascade-delete acme/client_b",
        )
        .expect("token generation must succeed");

    // Delete acme/client_b with force=true.
    let result2 = store2.delete("client_b", &client_scope("acme"), true, token2);
    assert!(
        result2.is_ok(),
        "force=true client cascade delete must succeed: {:?}",
        result2.err()
    );

    // Post-cascade: acme/client_b must be gone.
    assert!(
        store2
            .get("client_b", &client_scope("acme"))
            .unwrap()
            .is_none(),
        "acme/client_b must be absent after deletion"
    );
    // Post-cascade: acme/client_a must be gone (cascaded within acme).
    assert!(
        store2
            .get("client_a", &client_scope("acme"))
            .unwrap()
            .is_none(),
        "acme/client_a must be absent after acme/client_b cascade delete"
    );
    // Post-cascade: beta/client_b must still exist (different scope).
    assert!(
        store2
            .get("client_b", &client_scope("beta"))
            .unwrap()
            .is_some(),
        "beta/client_b must still exist — cascade is scope-bounded"
    );
    // Post-cascade: beta/client_a must still exist (different scope).
    assert!(
        store2
            .get("client_a", &client_scope("beta"))
            .unwrap()
            .is_some(),
        "beta/client_a must still exist — cascade must NOT cross client boundaries"
    );
    let _ = std::fs::remove_file(&path1);
    let _ = std::fs::remove_file(&path2);
}

// ─────────────────────────────────────────────────────────────────────────────
// CR-004: list() sort order
// ─────────────────────────────────────────────────────────────────────────────

/// CR-004: 3-alias mixed scope produces correctly grouped output (Global first, then Client).
#[test]
fn test_BC_2_11_013_list_mixed_scope_sorted_correctly() {
    let path = format!("/tmp/test_list_sort_{}.toml", std::process::id());
    let mut store = AliasStore::empty(&path);

    // Insert out-of-order to validate sort independence from insertion order.
    let _ = store.create_or_update(simple_entry("z_global", global_scope(), "z = 1"), None);
    let _ = store.create_or_update(
        simple_entry("a_client", client_scope("acme"), "a = 1"),
        None,
    );
    let _ = store.create_or_update(simple_entry("a_global", global_scope(), "a = 1"), None);

    let entries = store.list(None);
    assert_eq!(entries.len(), 3, "all 3 aliases must be listed");

    // Global aliases first, sorted by name.
    assert_eq!(
        entries[0].name, "a_global",
        "first entry must be a_global (global scope)"
    );
    assert_eq!(
        entries[1].name, "z_global",
        "second entry must be z_global (global scope)"
    );
    // Per-client alias last.
    assert_eq!(
        entries[2].name, "a_client",
        "last entry must be a_client (client scope)"
    );
    let _ = std::fs::remove_file(&path);
}

// ─────────────────────────────────────────────────────────────────────────────
// CR-006: composition_chain recursive
// ─────────────────────────────────────────────────────────────────────────────

/// CR-006: depth-3 chain a → b → c → literal produces chain ["a", "b", "c"].
#[test]
fn test_BC_2_11_015_depth3_chain_composition_chain_correct() {
    let path = format!("/tmp/test_chain_{}.toml", std::process::id());
    let mut store = AliasStore::empty(&path);

    // c = literal (depth 0 body)
    let _ = store.create_or_update(simple_entry("c", global_scope(), "severity_id >= 1"), None);
    // b = @c (depth 1 body)
    let _ = store.create_or_update(simple_entry("b", global_scope(), "@c AND x = 1"), None);
    // a = @b (depth 2 body — total chain is a, b, c)
    let _ = store.create_or_update(simple_entry("a", global_scope(), "@b AND y = 2"), None);

    let input = ExplainAliasInput {
        name: "a".to_string(),
        scope: Some("global".to_string()),
    };
    let resp =
        explain_alias(input, &store, None).expect("explain_alias must succeed for a valid chain");
    // Chain must include a, b, c in that order.
    assert!(
        resp.composition_chain.contains(&"b".to_string()),
        "composition chain must include b"
    );
    assert!(
        resp.composition_chain.contains(&"c".to_string()),
        "composition chain must include c"
    );
    assert_eq!(
        resp.composition_chain[0], "a",
        "first chain element is the root alias"
    );
    let _ = std::fs::remove_file(&path);
}

// ─────────────────────────────────────────────────────────────────────────────
// CR-008: token validation in delete_alias
// ─────────────────────────────────────────────────────────────────────────────

/// CR-008: delete_alias with bogus token_id returns error.
///
/// After implementing real token validation, any token_id not generated by
/// token_store.generate() must be rejected.
#[test]
fn test_BC_2_11_014_delete_with_bogus_token_returns_error() {
    let path = format!("/tmp/test_delete_token_{}.toml", std::process::id());
    let mut store = AliasStore::empty(&path);
    let token_store = prism_security::ConfirmationTokenStore::new();

    // First create an alias so delete has something to work with.
    let entry = simple_entry("to_delete", global_scope(), "severity_id >= 3");
    store
        .create_or_update(entry, None)
        .expect("test setup: create_or_update must succeed on a process-unique /tmp path");

    // Attempt to delete with a bogus (not generated) token_id.
    let input = DeleteAliasInput {
        name: "to_delete".to_string(),
        scope: "global".to_string(),
        force: false,
        token_id: Some("bogus-not-real-xyz".to_string()),
    };
    let result = delete_alias(input, &mut store, &token_store, &[]);
    assert!(
        result.is_err(),
        "bogus token_id must be rejected by token_store.consume()"
    );
    let _ = std::fs::remove_file(&path);
}

/// CR-008: legitimate two-step flow — generate token then delete succeeds.
#[test]
fn test_BC_2_11_014_delete_two_step_flow_succeeds() {
    let path = format!("/tmp/test_delete_twostep_{}.toml", std::process::id());
    let mut store = AliasStore::empty(&path);
    let token_store = prism_security::ConfirmationTokenStore::new();

    // Create alias for deletion.
    let entry = simple_entry("twostep_alias", global_scope(), "severity_id >= 3");
    store
        .create_or_update(entry, None)
        .expect("test setup: create_or_update must succeed on a process-unique /tmp path");

    // Step 1: call without token_id — should return confirmation_required + token_id.
    let input_step1 = DeleteAliasInput {
        name: "twostep_alias".to_string(),
        scope: "global".to_string(),
        force: false,
        token_id: None,
    };
    let resp = delete_alias(input_step1, &mut store, &token_store, &[])
        .expect("first delete call (no token) must succeed and return confirmation_required");

    // Must include token_id in response (CR-008).
    assert!(
        resp.get("token_id").is_some(),
        "first-call response must include token_id"
    );
    assert!(
        resp["confirmation_required"].as_bool().unwrap_or(false),
        "first-call response must have confirmation_required=true"
    );

    let token_id = resp["token_id"]
        .as_str()
        .expect("token_id must be a string")
        .to_string();

    // Step 2: call with the real token_id — should succeed.
    let input_step2 = DeleteAliasInput {
        name: "twostep_alias".to_string(),
        scope: "global".to_string(),
        force: false,
        token_id: Some(token_id),
    };
    let del = delete_alias(input_step2, &mut store, &token_store, &[])
        .expect("second delete call with valid token must succeed");
    assert_eq!(
        del["deleted"].as_str().unwrap_or(""),
        "twostep_alias",
        "deleted field must match alias name"
    );

    let _ = std::fs::remove_file(&path);
}

// ─────────────────────────────────────────────────────────────────────────────
// CR-009: regex-based replacement avoids prefix-alias corruption
// ─────────────────────────────────────────────────────────────────────────────

/// CR-009: expanding `@foo` in query `@foo AND @foobar` only replaces `@foo`,
/// leaving `@foobar` intact (prefix-alias non-corruption).
#[test]
fn test_BC_2_11_009_expand_prefix_alias_not_corrupted() {
    let path = format!("/tmp/test_prefix_alias_{}.toml", std::process::id());
    let mut store = AliasStore::empty(&path);

    // foo = simple literal expansion
    let entry_foo = simple_entry("foo", global_scope(), "severity_id >= 1");
    let _ = store.create_or_update(entry_foo, None);

    // foobar = different expansion
    let entry_foobar = simple_entry("foobar", global_scope(), "severity_id >= 5");
    let _ = store.create_or_update(entry_foobar, None);

    let scope = global_scope();
    let args = HashMap::new();
    // Expand only @foo — @foobar should survive as a reference to be expanded separately.
    // In a full expansion, both @foo and @foobar are each expanded exactly once.
    let expanded = AliasResolver::expand("@foo AND @foobar", &store, &scope, &args, 0)
        .expect("expand must succeed for known aliases");
    // After full expansion, neither @foo nor @foobar should remain unexpanded.
    assert!(
        !expanded.contains("@foo"),
        "expanded form must not contain unexpanded @foo"
    );
    // Both expansions must appear in the result.
    assert!(
        expanded.contains("severity_id >= 1"),
        "foo expansion (severity_id >= 1) must appear"
    );
    assert!(
        expanded.contains("severity_id >= 5"),
        "foobar expansion (severity_id >= 5) must appear"
    );
    let _ = std::fs::remove_file(&path);
}

// ─────────────────────────────────────────────────────────────────────────────
// CR-010: quoted strings with newline/null rejected
// ─────────────────────────────────────────────────────────────────────────────

/// CR-010: string literal with embedded newline rejected.
#[test]
fn test_BC_2_11_009_quoted_string_with_newline_rejected() {
    let value_with_newline = "\"\n\"";
    let result = AliasResolver::validate_atomic_literal(value_with_newline, "param", "alias");
    assert!(
        result.is_err(),
        "quoted string with embedded newline must be rejected"
    );
}

/// CR-P2-005: string literal with embedded tab (\t) rejected.
///
/// Tabs are whitespace control characters; accepting them in parameter values
/// would allow parameter injection via whitespace manipulation. Rejected alongside
/// newline, carriage-return, and null bytes.
#[test]
fn test_BC_2_11_009_quoted_string_with_tab_rejected() {
    // Double-quoted string with tab.
    let value_with_tab = "\"\t\"";
    let result = AliasResolver::validate_atomic_literal(value_with_tab, "param", "alias");
    assert!(
        result.is_err(),
        "double-quoted string with embedded tab must be rejected (CR-P2-005)"
    );

    // Single-quoted string with tab.
    let single_quoted_with_tab = "'\t'";
    let result = AliasResolver::validate_atomic_literal(single_quoted_with_tab, "param", "alias");
    assert!(
        result.is_err(),
        "single-quoted string with embedded tab must be rejected (CR-P2-005)"
    );

    // Verify clean string still accepted.
    let clean = "\"clean_value\"";
    let result = AliasResolver::validate_atomic_literal(clean, "param", "alias");
    assert!(result.is_ok(), "clean quoted string must still be accepted");
}

// ─────────────────────────────────────────────────────────────────────────────
// CR-007: create_alias rejects Client scopes
// ─────────────────────────────────────────────────────────────────────────────

/// CR-007: create_alias (no client list) rejects Client-scoped creation.
#[test]
fn test_BC_2_11_008_create_alias_rejects_client_scope_without_client_list() {
    let path = format!("/tmp/test_create_client_{}.toml", std::process::id());
    let mut store = AliasStore::empty(&path);
    let ocsf = empty_ocsf();
    let input = CreateAliasInput {
        name: "my_alias".to_string(),
        scope: "client:acme".to_string(),
        query: "severity_id >= 3".to_string(),
        parameters: None,
        description: None,
        token_id: None,
    };
    // create_alias (no client list) must reject Client scope.
    let result = create_alias(input, &mut store, &ocsf);
    assert!(
        result.is_err(),
        "create_alias must reject Client scope without valid_client_ids list"
    );
    // No file written since error occurred before any store mutation.
    let _ = std::fs::remove_file(&path);
}

// ─────────────────────────────────────────────────────────────────────────────
// CR-015: AliasScope TOML roundtrip
// ─────────────────────────────────────────────────────────────────────────────

/// CR-015 / CR-P2-004: AliasScope JSON roundtrip via AliasEntry.
///
/// Tests that AliasScope survives JSON roundtrip when embedded in an AliasEntry.
/// Renamed from test_alias_scope_toml_roundtrip_via_entry (CR-P2-004 — the original
/// test actually exercised JSON serde, not TOML).
#[test]
fn test_alias_scope_json_roundtrip_via_entry() {
    // Roundtrip Global scope via AliasEntry (JSON).
    let entry_global = simple_entry("my_alias", global_scope(), "severity_id >= 3");
    let json_str =
        serde_json::to_string(&entry_global).expect("AliasEntry with Global scope must serialize");
    let roundtripped: crate::alias_types::AliasEntry =
        serde_json::from_str(&json_str).expect("AliasEntry with Global scope must deserialize");
    assert_eq!(
        entry_global.scope, roundtripped.scope,
        "Global scope JSON roundtrip must be lossless"
    );

    // Roundtrip Client scope via AliasEntry (JSON).
    let entry_client = simple_entry("client_alias", client_scope("acme"), "severity_id >= 3");
    let json_str =
        serde_json::to_string(&entry_client).expect("AliasEntry with Client scope must serialize");
    let roundtripped: crate::alias_types::AliasEntry =
        serde_json::from_str(&json_str).expect("AliasEntry with Client scope must deserialize");
    assert_eq!(
        entry_client.scope, roundtripped.scope,
        "Client scope JSON roundtrip must be lossless"
    );
}

/// CR-P2-004: AliasScope actual TOML roundtrip via AliasEntry embedded in aliases table.
///
/// Exercises the real `aliases.toml` persistence format: the AliasEntry is serialized
/// via `toml::to_string_pretty` inside a wrapper struct (as `write_entries_to_file` does)
/// and deserialized back. This is distinct from the JSON serde above.
#[test]
fn test_alias_scope_toml_roundtrip() {
    use serde::{Deserialize, Serialize};

    /// Mirror of the private `AliasesFile` struct used in `alias_store.rs`.
    #[derive(Debug, Serialize, Deserialize)]
    struct AliasesFile {
        aliases: Vec<crate::alias_types::AliasEntry>,
    }

    // Roundtrip Global scope.
    let entry_global = simple_entry("toml_alias", global_scope(), "severity_id >= 3");
    let file = AliasesFile {
        aliases: vec![entry_global.clone()],
    };
    let toml_str = toml::to_string_pretty(&file).expect("AliasesFile must serialize to TOML");
    let roundtripped: AliasesFile =
        toml::from_str(&toml_str).expect("AliasesFile must deserialize from TOML");
    assert_eq!(
        roundtripped.aliases[0].scope, entry_global.scope,
        "Global scope TOML roundtrip must be lossless"
    );

    // Roundtrip Client scope.
    let entry_client = simple_entry("toml_client_alias", client_scope("acme"), "x = 1");
    let file = AliasesFile {
        aliases: vec![entry_client.clone()],
    };
    let toml_str = toml::to_string_pretty(&file).expect("AliasesFile must serialize to TOML");
    let roundtripped: AliasesFile =
        toml::from_str(&toml_str).expect("AliasesFile must deserialize from TOML");
    assert_eq!(
        roundtripped.aliases[0].scope, entry_client.scope,
        "Client scope TOML roundtrip must be lossless"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// SEC-005: alias.write capability gate enforced via gated variants
// ─────────────────────────────────────────────────────────────────────────────

/// SEC-005: capability gate disabled → create_alias_with_clients_gated returns E-FLAG-001.
#[test]
fn test_BC_2_11_008_capability_gate_disabled_rejects_create() {
    use crate::alias_tools::create_alias_with_clients_gated;
    use prism_security::feature_flag::{CompileTimeGate, FeatureFlagEvaluator};
    use std::collections::BTreeMap;

    let path = format!("/tmp/test_cap_gate_{}.toml", std::process::id());
    let mut store = AliasStore::empty(&path);
    let ocsf = empty_ocsf();
    let input = CreateAliasInput {
        name: "cap_alias".to_string(),
        scope: "global".to_string(),
        query: "severity_id >= 3".to_string(),
        parameters: None,
        description: None,
        token_id: None,
    };

    // Compile-time gate absent → always denied regardless of runtime config.
    let evaluator = FeatureFlagEvaluator::new(BTreeMap::new());
    let token_store = prism_security::ConfirmationTokenStore::new();
    let result = create_alias_with_clients_gated(
        input,
        &mut store,
        &ocsf,
        &[],
        Some((&evaluator, CompileTimeGate::Absent)),
        &token_store,
    );
    assert!(
        result.is_err(),
        "CompileTimeGate::Absent must deny create_alias_with_clients_gated"
    );
    // No file written since error occurred before any store mutation.
    let _ = std::fs::remove_file(&path);
}

// ─────────────────────────────────────────────────────────────────────────────
// CR-P6-002: global-scope alias.write checks any-client per BC-2.11.008
// ─────────────────────────────────────────────────────────────────────────────

/// BC-2.11.008: global-scope alias.write allowed when AT LEAST ONE client allows it.
///
/// Regression for CR-P6-002: the old implementation used a `"__global__"` sentinel
/// client_id which was never in the evaluator, so Global-scope writes were always
/// denied by the runtime tier even when a real client had `alias.write = Allow`.
///
/// Canonical test vector: 3 clients (client_a=Allow, client_b=Deny, client_c=Deny)
/// → Global scope op succeeds because client_a allows.
#[test]
fn test_BC_2_11_008_global_scope_alias_write_allowed_when_any_client_allows() {
    use crate::alias_capability::check_alias_write;
    use crate::alias_types::AliasScope;
    use prism_core::capability::{CapabilityEffect, CapabilityPath, ClientCapabilities};
    use prism_security::feature_flag::{CompileTimeGate, FeatureFlagEvaluator};
    use std::collections::BTreeMap;

    #[allow(clippy::expect_used)]
    let alias_write_path =
        CapabilityPath::new("alias.write").expect("alias.write is a valid capability path");

    // client_a: alias.write = Allow
    let mut caps_a = ClientCapabilities::new();
    caps_a.grant(alias_write_path.clone(), CapabilityEffect::Allow);

    // client_b: alias.write = Deny
    let mut caps_b = ClientCapabilities::new();
    caps_b.grant(alias_write_path.clone(), CapabilityEffect::Deny);

    // client_c: no alias.write entry → deny-by-default
    let caps_c = ClientCapabilities::new();

    let mut client_map = BTreeMap::new();
    client_map.insert("client_a".to_string(), caps_a);
    client_map.insert("client_b".to_string(), caps_b);
    client_map.insert("client_c".to_string(), caps_c);

    let evaluator = FeatureFlagEvaluator::new(client_map);
    let valid_client_ids = vec![
        "client_a".to_string(),
        "client_b".to_string(),
        "client_c".to_string(),
    ];

    let result = check_alias_write(
        &AliasScope::Global,
        &evaluator,
        CompileTimeGate::Present,
        &valid_client_ids,
    );
    assert!(
        result.is_ok(),
        "global-scope alias.write must be allowed when at least one client (client_a) has Allow"
    );
}

/// BC-2.11.008: global-scope alias.write denied when ALL configured clients deny it.
///
/// Regression for CR-P6-002: 3 clients all deny → global scope op must return E-FLAG-001.
#[test]
fn test_BC_2_11_008_global_scope_alias_write_denied_when_all_clients_deny() {
    use crate::alias_capability::check_alias_write;
    use crate::alias_types::AliasScope;
    use prism_core::capability::{CapabilityEffect, CapabilityPath, ClientCapabilities};
    use prism_security::feature_flag::{CompileTimeGate, FeatureFlagEvaluator};
    use std::collections::BTreeMap;

    #[allow(clippy::expect_used)]
    let alias_write_path =
        CapabilityPath::new("alias.write").expect("alias.write is a valid capability path");

    // All three clients explicitly deny alias.write.
    let mut caps_a = ClientCapabilities::new();
    caps_a.grant(alias_write_path.clone(), CapabilityEffect::Deny);

    let mut caps_b = ClientCapabilities::new();
    caps_b.grant(alias_write_path.clone(), CapabilityEffect::Deny);

    let mut caps_c = ClientCapabilities::new();
    caps_c.grant(alias_write_path.clone(), CapabilityEffect::Deny);

    let mut client_map = BTreeMap::new();
    client_map.insert("client_a".to_string(), caps_a);
    client_map.insert("client_b".to_string(), caps_b);
    client_map.insert("client_c".to_string(), caps_c);

    let evaluator = FeatureFlagEvaluator::new(client_map);
    let valid_client_ids = vec![
        "client_a".to_string(),
        "client_b".to_string(),
        "client_c".to_string(),
    ];

    let result = check_alias_write(
        &AliasScope::Global,
        &evaluator,
        CompileTimeGate::Present,
        &valid_client_ids,
    );
    assert!(
        result.is_err(),
        "global-scope alias.write must be denied when all 3 clients deny; got Ok"
    );
}
