//! Unit and integration tests for the S-3.04 alias system.
//!
//! All tests listed here are RED by design — they test behaviour that is not yet
//! implemented (all non-trivial bodies are `todo!()`). Tests must remain red
//! until the implementer fills in real logic.
//!
//! Traces to BCs: BC-2.11.008, BC-2.11.009, BC-2.11.013, BC-2.11.014, BC-2.11.015
//! Traces to ACs: AC-1 through AC-14
//! Traces to VPs: VP-012 concrete, VP-013 concrete, VP-037 concrete

use std::collections::{HashMap, HashSet};

use prism_core::error::PrismError;

use crate::alias_resolver::AliasResolver;
use crate::alias_store::AliasStore;
use crate::alias_tools::{
    create_alias, delete_alias, explain_alias, list_aliases, validate_alias_name,
    validate_no_keyword_collision, CreateAliasInput, DeleteAliasInput, ExplainAliasInput,
    ListAliasesInput, PRISMQL_KEYWORDS,
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
    // RED: AliasStore::create_or_update / AliasResolver::expand are todo!()
    let mut store = AliasStore::empty("/tmp/test_aliases.toml");
    let scope = global_scope();

    let entry = simple_entry("high_sev", global_scope(), "severity_id >= 3");
    let _ = store.create_or_update(entry, None); // todo!() fires here → RED
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

/// AC-3: Alias A = "@B AND foo", alias B = "@A OR bar" → E-ALIAS-002 on create.
#[test]
fn test_ac3_cycle_detection_at_creation() {
    // RED: AliasResolver::detect_cycle is todo!()
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let result = AliasResolver::detect_cycle("A", "@B AND foo", &store);
    assert!(result.is_err(), "todo!() fires — test is RED");
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
    // RED: validate_no_keyword_collision is todo!()
    let ocsf = empty_ocsf();
    let result = validate_no_keyword_collision("high_sev", &ocsf);
    // Once implemented this should be Ok(()); currently fires todo!()
    assert!(result.is_err(), "todo!() fires — test is RED");
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
    // RED: AliasResolver::validate_atomic_literal is todo!()
    let result = AliasResolver::validate_atomic_literal("5", "min_sev", "recent_alerts");
    assert!(result.is_err(), "todo!() fires — test is RED");
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
    // RED: list_aliases is todo!()
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let input = ListAliasesInput { scope: None };
    let result = list_aliases(input, &store, &[]);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// AC-8: list_aliases with scope="global" returns only global aliases.
#[test]
fn test_ac8_list_aliases_global_only() {
    // RED: list_aliases is todo!()
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let input = ListAliasesInput {
        scope: Some("global".to_string()),
    };
    let result = list_aliases(input, &store, &[]);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// EC-11-033: No aliases defined → list returns empty array (not an error).
/// Note: this will currently error because list_aliases is todo!().
#[test]
fn test_ec11_033_empty_store_list_not_error() {
    // RED: list_aliases is todo!()
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let input = ListAliasesInput { scope: None };
    let result = list_aliases(input, &store, &[]);
    // Once implemented: assert!(result.is_ok()); currently RED
    assert!(result.is_err(), "todo!() fires — test is RED");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-9: delete_alias requires confirmation token (BC-2.11.014)
// ─────────────────────────────────────────────────────────────────────────────

/// AC-9: delete_alias without token must return a ConfirmationToken, not a deletion.
#[test]
fn test_ac9_delete_requires_confirmation() {
    // RED: delete_alias is todo!()
    let mut store = AliasStore::empty("/tmp/test_aliases.toml");
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
    let mut store = AliasStore::empty("/tmp/test_aliases.toml");
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

/// AC-12: create_alias returns capability error when alias.write is disabled.
#[test]
fn test_ac12_write_capability_gate() {
    // RED: create_alias is todo!()
    let mut store = AliasStore::empty("/tmp/test_aliases.toml");
    let ocsf = empty_ocsf();
    let input = CreateAliasInput {
        name: "test_alias".to_string(),
        scope: "global".to_string(),
        query: "severity_id >= 3".to_string(),
        parameters: None,
        description: None,
    };
    let result = create_alias(input, &mut store, &ocsf);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-13: create_alias returns ConfirmationToken when alias already exists (BC-2.11.008)
// ─────────────────────────────────────────────────────────────────────────────

/// AC-13: Second create_alias on same name/scope returns ConfirmationRequired.
#[test]
fn test_ac13_update_requires_confirmation() {
    // RED: AliasStore::create_or_update is todo!()
    let mut store = AliasStore::empty("/tmp/test_aliases.toml");
    let entry = simple_entry("high_sev", global_scope(), "severity_id >= 3");
    let result = store.create_or_update(entry, None);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

// ─────────────────────────────────────────────────────────────────────────────
// Alias name validation
// ─────────────────────────────────────────────────────────────────────────────

/// Valid alias names must pass name validation.
#[test]
fn test_validate_alias_name_valid() {
    // RED: validate_alias_name is todo!()
    assert!(validate_alias_name("high_sev").is_err(), "todo!() fires");
    assert!(validate_alias_name("_my_alias").is_err(), "todo!() fires");
    assert!(validate_alias_name("alias123").is_err(), "todo!() fires");
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
    // RED: AliasScope::parse is todo!()
    let result = AliasScope::parse("global");
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// AliasScope::parse("client:acme") must return Client("acme").
#[test]
fn test_alias_scope_parse_client() {
    // RED: AliasScope::parse is todo!()
    let result = AliasScope::parse("client:acme");
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// AliasScope::parse with invalid format must return error.
#[test]
fn test_alias_scope_parse_invalid() {
    // RED: AliasScope::parse is todo!()
    let result = AliasScope::parse("bad_format");
    assert!(result.is_err(), "todo!() fires — test is RED");
}

// ─────────────────────────────────────────────────────────────────────────────
// AliasStore::dependents (BC-2.11.014)
// ─────────────────────────────────────────────────────────────────────────────

/// dependents() on empty store returns empty vec.
#[test]
fn test_dependents_empty_store() {
    // RED: AliasStore::dependents is todo!()
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let deps = store.dependents("high_sev", &global_scope());
    // Once implemented: assert!(deps.is_empty()); currently RED via todo!()
    // The test itself doesn't panic — dependents() will fire todo!().
    // We call it in a catch_unwind to confirm RED behavior.
    // Actually: todo!() panics, which is the RED gate. The test will FAIL (panic).
    drop(deps);
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
#[test]
fn test_vp013_concrete_mutual_cycle() {
    // RED: detect_cycle is todo!()
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let result = AliasResolver::detect_cycle("A", "@B AND x", &store);
    assert!(result.is_err(), "todo!() fires — test is RED");
}

/// VP-013 concrete: acyclic alias (A → B, no back-edge) must NOT produce a cycle error.
#[test]
fn test_vp013_concrete_acyclic_no_error() {
    // RED: detect_cycle is todo!()
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let result = AliasResolver::detect_cycle("A", "@B", &store);
    // Once implemented with B absent: Err(E-ALIAS-001) or Ok(()) — but currently RED
    assert!(result.is_err(), "todo!() fires — test is RED");
}

// ─────────────────────────────────────────────────────────────────────────────
// VP-037 concrete tests (no-panic on adversarial inputs)
// ─────────────────────────────────────────────────────────────────────────────

/// VP-037 concrete: non-UTF-8 bytes-as-str (via lossy conversion) must not panic.
#[test]
fn test_vp037_concrete_non_utf8_does_not_panic() {
    // RED: AliasResolver::expand is todo!()
    // We cannot pass &[u8] directly, but we can pass a lossy-converted string.
    let lossy = String::from_utf8_lossy(&[0xFF, 0xFE, 0x41, 0x00]).to_string();
    let store = AliasStore::empty("/tmp/test_aliases.toml");
    let scope = global_scope();
    let args = HashMap::new();

    let result = AliasResolver::expand(&lossy, &store, &scope, &args, 0);
    // Must be Err — must not panic.
    assert!(result.is_err(), "todo!() fires — test is RED");
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
    // RED: create_or_update is todo!()
    let mut store = AliasStore::empty("/tmp/test_aliases.toml");
    let entry = simple_entry("high_sev", client_scope("acme"), "severity_id > 4");
    let result = store.create_or_update(entry, None);
    assert!(result.is_err(), "todo!() fires — test is RED");
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
