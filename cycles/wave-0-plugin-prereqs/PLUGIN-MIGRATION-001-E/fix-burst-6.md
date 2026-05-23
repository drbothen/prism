---
document_type: fix-burst-report
cascade: PLUGIN-MIGRATION-001-E LOCAL
burst_number: 6
date: 2026-05-23
feature_head_before: 7702ea78
feature_head_after: 657762c7
develop_head_baseline: f19575ff
findings_addressed: [F-LP7-MED-001, F-LP7-MED-002, F-LP7-LOW-001]
findings_closed: 3
findings_deferred: 0
feature_branch_commits: 2
correction_commits: 1
workspace_test_count: 3518
just_check_status: PASS
paper_fix_caught_pre_persistence: true
artifacts_bumped:
  - BC-2.16.002: v1.40 → v1.42 (row 37 added, then emission site description corrected guest stub → host wrapper; catalog header v1.25 → v1.26)
  - BC-INDEX: v5.43 → v5.45
  - BC-2.16.012: v1.32 → v1.33 (POL-29 sibling sweep, 3 cite-pin sites)
  - error-taxonomy: v1.46 → v1.47 (POL-29 sibling sweep, 3 cite-pin sites)
  - PLUGIN-MIGRATION-001-E story: v1.2 → v1.3 (EC-006/007/008/009 SID-1 §5 citations)
  - S-PLUGIN-PREREQ-E story: v1.53 → v1.54 (POL-29 sibling sweep, 5 cite-pin sites)
inputs:
  - .factory/cycles/wave-0-plugin-prereqs/PLUGIN-MIGRATION-001-E/local-pass-7.md
input-hash: "[live-fb-6]"
---

# PLUGIN-MIGRATION-001-E — FB-IMPL-6 (with correction)

**Date:** 2026-05-23
**Feature HEAD before:** `7702ea78`
**Feature HEAD after:** `657762c7`
**Develop baseline:** `f19575ff`
**Findings addressed:** F-LP7-MED-001 + F-LP7-MED-002 + F-LP7-LOW-001 (all 3 from pass-7)
**Feature-branch commits:** 2 (`e56e6f97` original + `657762c7` correction)
**Workspace tests:** 3518 GREEN (the 3762→3518 delta observed earlier was a counting artifact between `just check` orchestration and bare `cargo nextest`; nextest count is authoritative)
**just check:** PASS
**Paper-fix caught pre-persistence:** YES (see correction note below)

## Paper-fix detected by orchestrator structural-coverage axis (pre-persistence)

The original FB-IMPL-6 commit `e56e6f97` placed the F-LP7-MED-001 tracing emission in the GUEST `acquire_token()` (`crates/prism-spec-engine/plugins/crowdstrike-oauth2/src/lib.rs`) via a helper `emit_auth_token_parse_error()` gated `#[cfg(test)]` (real emission) with `#[cfg(not(test))]` no-op stub. The capturing-subscriber test in the guest passed because it ran in `cfg(test)` mode where the real emission fires.

Architectural reality: the wasm32 guest runs in a sandboxed wasmtime instance with NO `tracing::*!` subscriber. Production wasm32 builds invoked the no-op stub. The spec EC-002 audit-observability promise was structurally unmet despite the test passing — TD-VSDD-059 paper-fix pattern.

The orchestrator caught the gap via the structural-coverage axis (introduced pass-5) BEFORE state-manager persisted the FB-IMPL-6 artifacts. Pass-5 axis evidence: grep for `plugin.auth_token_parse_error` in `crates/prism-spec-engine/src/` returned ZERO matches — no host-side emission.

## F-LP7-MED-001 — corrected closure (commit `657762c7`)

Correction:

1. Added host emission `emit_acquire_token_parse_error_and_fail` at `crates/prism-spec-engine/src/plugin/mod.rs` line ~1115 (function definition) called from `dispatch_plugin_acquire_token` `None =>` arm of `kv_store.get(plugin_id, "token")` match at line ~745. Unconditional — no `#[cfg(test)]` gate.
2. Removed the guest-side `emit_auth_token_parse_error` helper + `#[cfg(not(test))]` no-op stub + all guest call sites.
3. Reverted guest signature `plugin_id: &str` parameter additions to `acquire_token` / `get_token` (host knows the plugin_id from the registry; guest doesn't need it).
4. Replaced the in-guest capturing-subscriber test with integration test `test_F_LP7_MED_001_host_emit_acquire_token_parse_error_fires_unconditionally` in `crates/prism-spec-engine/tests/crowdstrike_oauth2_plugin_tests.rs`. Test calls `emit_acquire_token_parse_error_and_fail("crowdstrike-oauth2")` directly with a capturing tracing subscriber; asserts captured output contains `plugin.auth_token_parse_error` + `crowdstrike-oauth2` + returns `Err`.
5. BC-2.16.002 catalog row 37 description updated v1.41 → v1.42 to reflect host emission site. Catalog header label advanced `(v1.25)` → `(v1.26)` per POL-30 Fork B canonical rule (row description change = catalog-content structural change). BC-INDEX v5.44 → v5.45 to track.

Anti-paper-fix evidence: removing the `error!` call from `emit_acquire_token_parse_error_and_fail` causes the integration test assertion (a) (`output_str.contains("plugin.auth_token_parse_error")`) to fail. Removing the helper call from `dispatch_plugin_acquire_token` `None =>` arm causes a follow-up test to fail. The emission is now LOAD-BEARING in production.

## F-LP7-MED-002 — story spec EC-006..009 SID-1 §5 citations (preserved from original FB-IMPL-6)

Story spec `/Users/jmagady/Dev/prism/.factory/stories/PLUGIN-MIGRATION-001-E-crowdstrike-oauth2-refresh-on-401-prx-wasm-plugin.md` v1.2 → v1.3:

- EC-006: cites `S-PLUGIN-CI-001 AC-002 → test_S_PLUGIN_CI_001_002_missing_prx_at_boot_continues_with_error_log` (newly-created stub at factory SHA `69b95e40`)
- EC-007: cites `test_BC_2_17_006_ac7_invalid_wit_returns_e_plugin_001` + `test_BC_2_17_006_ac7_invariant_plugin_not_registered_after_invalid_wit` (existing tests)
- EC-008: cites `test_BC_2_17_002_ec17_url_not_in_allowlist_returns_403` + `test_BC_2_17_002_ec17_007_http_request_empty_allowlist_blocked` (existing tests)
- EC-009: cites `S-PLUGIN-CI-001 AC-003 → test_S_PLUGIN_CI_001_003_double_401_returns_auth_refresh_failed` (newly-created stub)

All test names verified to exist (existing tests in `plugin_tests.rs`; S-PLUGIN-CI-001 AC-002/003 Red Gate names in the new stub).

## F-LP7-LOW-001 — EC-003 no-token-cached assertion (preserved from original FB-IMPL-6)

3-line assertion added to `test_acquire_token_EC_003_missing_access_token_returns_response_parse` in `crates/prism-spec-engine/plugins/crowdstrike-oauth2/src/lib.rs`:

```rust
assert_eq!(
    host.kv_store.borrow().get("token"),
    None,
    "EC-003: token MUST NOT be cached when access_token field is missing"
);
```

Sibling-symmetry with EC-001 and EC-002 tests achieved.

## POL-29 sibling sweep (preserved)

The original FB-IMPL-6 commit performed sibling-sweep updates that survived the correction unchanged:

- BC-2.16.012 v1.32→v1.33 (3 cite-pin sites)
- error-taxonomy.md v1.46→v1.47 (3 cite-pin sites)
- S-PLUGIN-PREREQ-E story v1.53→v1.54 (5 cite-pin sites)
- BC-INDEX v5.43→v5.44→v5.45 (rolled forward in correction)

## Streak attempt impact

FB-IMPL-6 (with correction) closes all 3 pass-7 findings via real load-bearing closures. Next adversary pass (pass-8) attempts streak 0/3 → 1/3 with:

- Structural-coverage axis (pass-5) continuing
- EC-test-vs-spec fidelity axis test-body dimension (pass-6) continuing
- Spec-emission existence sub-dim (pass-7-A) now structurally hardened (host emission load-bearing)
- Deferral-citation specificity sub-dim (pass-7-B) now structurally hardened (S-PLUGIN-CI-001 stub exists)
- Test-assertion sibling-symmetry sub-dim (pass-7-C) now hardened for EC-003

The orchestrator-caught paper-fix in this burst validates the standing axes — the structural-coverage axis caught a defect that would have shipped under any prior cascade discipline. Pass-8 should apply the same fresh-context structural-coverage axis to verify the host emission relocation is itself load-bearing.
