---
document_type: adversary-pass-report
cascade: PLUGIN-MIGRATION-001-E LOCAL
pass_number: 8
date: 2026-05-23
feature_head: 657762c7
develop_head_baseline: f19575ff
streak_before: 0/3
streak_after: 0/3
clean_strict: false
clean_pr_merge: false
findings_total: 7
findings_by_severity:
  CRIT: 0
  HIGH: 0
  MED: 4
  LOW: 2
  OBS: 1
  PROCESS-GAP: 0
decay_trajectory: "20 → 12 → 3 → 0 → 2 → 3 → 3 → 7"
standing_axes_applied:
  - structural-coverage (pass-5)
  - EC-test-vs-spec fidelity (pass-6)
  - spec-emission existence sub-dim (pass-7-A)
  - deferral-citation specificity sub-dim (pass-7-B)
  - test-assertion sibling-symmetry sub-dim (pass-7-C)
  - partial-fix regression discipline (S-7.01)
new_sub_dimensions_surfaced:
  - "test-as-paper-fix: integration test with silent-fallback success branch (POL-11 pattern in test code)"
  - "error-variant semantic fidelity (CompilationFailed used for non-compilation error)"
  - "spec-row format-specifier accuracy (catalog row %display vs literal string)"
  - "correction-burst orphan sweep (sibling tests after reverted signature change)"
  - "emission reachability under all dispatch branches (WAT-fixture short-circuit)"
inputs:
  - .factory/cycles/wave-0-plugin-prereqs/PLUGIN-MIGRATION-001-E/local-pass-{1..7}.md
  - .factory/cycles/wave-0-plugin-prereqs/PLUGIN-MIGRATION-001-E/fix-burst-{1..6}.md
  - .factory/stories/PLUGIN-MIGRATION-001-E-crowdstrike-oauth2-refresh-on-401-prx-wasm-plugin.md (v1.3)
  - .factory/stories/S-PLUGIN-CI-001-plugin-component-adapter-ci-toolchain-and-integration.md (v0.1)
  - .factory/specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md (v1.42)
  - .factory/specs/behavioral-contracts/BC-INDEX.md (v5.45)
  - crates/prism-spec-engine/src/plugin/mod.rs
  - crates/prism-spec-engine/plugins/crowdstrike-oauth2/src/lib.rs
  - crates/prism-spec-engine/tests/crowdstrike_oauth2_plugin_tests.rs
  - crates/prism-spec-engine/tests/plugin_tests.rs
  - crates/prism-core/src/error.rs
  - crates/prism-spec-engine/src/error.rs
input-hash: "[live-pass-8]"
---

# PLUGIN-MIGRATION-001-E — LOCAL Adversary Pass-8

**Date:** 2026-05-23
**Feature HEAD:** `657762c7`
**Develop HEAD baseline (unchanged):** `f19575ff`
**Cascade state at start:** streak 0/3, attempting 0/3 → 1/3
**Decay trajectory:** 20 → 12 → 3 → 0 (false CLEAN) → 2 → 3 → 3 → 7 (this pass)

## Streak after this pass: stays at 0/3

CLEAN (strict): no
CLEAN (PR-merge): no

Reason: 7 findings (0 CRIT, 0 HIGH, 4 MED, 2 LOW, 1 OBS). CLEAN(strict) requires zero ANY severity → fails. CLEAN(PR-merge) requires zero CRIT+HIGH+MED → fails (4 MED).

This pass is a NOVELTY UPTICK — fresh-context analysis of the FB-IMPL-6 correction surfaces multiple new fidelity sub-dimensions the prior 7 passes did not probe. Validates "Fresh-Context Compounding Value" principle.

## Findings (full pass-7 content)

### F-LP8-MED-001 — Integration test silently passes when WAT-Component-Model parse fails (test-as-paper-fix pattern)

Surface: `crates/prism-spec-engine/tests/crowdstrike_oauth2_plugin_tests.rs` lines 1335-1496, specifically the `match result` block at lines 1451-1495.

Evidence:
- The integration test returns `None` in TWO failure paths (line 1416 WAT parse fail, line 1436 load_plugin fail). The `match result` at line 1451 `None =>` arm only `eprintln!`s and does not assert. Test passes silently.
- Doc-comment at lines 1332-1333 claims "Load-bearing" but the test can report PASS without exercising production code.
- The load-bearing UNIT test in `plugin/mod.rs:1352` IS structurally load-bearing. The integration test's misleading PASS signal is itself a regression vector.
- Stale name reference at line 1460 cites `test_dispatch_plugin_acquire_token_response_parse_emits_audit_event` which no longer exists.

Why it fails: TD-VSDD-059 paper-fix detection — test that can report PASS without exercising production path under any input condition is a paper-fix. POL-11 axis applied to test code.

Routing: implementer — convert `None` arm to hard `panic!`, OR delete the integration test, OR add `#[ignore]` with S-PLUGIN-CI-001 citation, OR add smoke assertion before WAT parse.

Paper-fix risk: HIGH if treated as comment-only update.

### F-LP8-MED-002 — `emit_acquire_token_parse_error_and_fail` returns `PluginError::CompilationFailed` for non-compilation runtime failure

Surface: `crates/prism-spec-engine/src/plugin/mod.rs` lines 1125-1129; `crates/prism-core/src/error.rs` lines 1045-1048.

Evidence:
- Helper returns `PluginError::CompilationFailed` documented as "E-PLUGIN-008: Plugin binary failed WASM Component Model compilation".
- Actual scenario: WASM compilation SUCCEEDED, dispatch SUCCEEDED, guest produced no token — runtime-behavioral failure, not compilation failure.
- Operators searching `journalctl | grep E-PLUGIN-008` will mix token-parse errors with real compilation failures.

Why it fails: Operator-facing audit observability accuracy. Source-of-Truth Precedence Rule 7 — SPEC names the trigger ("guest AuthError::ResponseParse") but code returns a semantically wrong variant.

Routing: implementer + (PO if new variant) — introduce `PluginError::AuthTokenNotCached` variant with new E-PLUGIN code; amend error-taxonomy.md; update BC-2.16.002 row 37; update unit test assertion to variant-match.

Paper-fix risk: MEDIUM — doc-comment-only claim "intentional reuse" would not fix triage accuracy.

### F-LP8-MED-003 — BC-2.16.002 row 37 `%display (source: fixed string)` annotation internally contradictory

Surface: `.factory/specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md` line 114 (row 37 fields cell).

Evidence:
- Row 37: `error: %display (source: fixed string "acquire-token dispatch completed but no token was cached in KV store ...")`.
- Code at `plugin/mod.rs:1121-1122` passes a string LITERAL — no `%display` Display-trait formatting occurs.
- `%display` syntax in tracing implies runtime Display invocation; literal strings have no Display call at emission site.

Why it fails: BC spec format-specifier accuracy axis — spec says one thing, code does another. Implementers reading the spec might mis-format future emissions.

Routing: product-owner (BC catalog row description edit). Drop `%display` annotation OR change emission to use `error = %e.to_string()`.

Paper-fix risk: LOW.

### F-LP8-MED-004 — Duplicate EC-002 tests in guest crate

Surface: `crates/prism-spec-engine/plugins/crowdstrike-oauth2/src/lib.rs` lines 731-759 (original) + lines 1106-1129 (FB-IMPL-6 leftover).

Evidence:
- Two EC-002 tests; the new one (added in original FB-IMPL-6 commit `e56e6f97`, survived correction commit `657762c7`) is a STRICT SUBSET of the original.
- BC-2.16.002 v1.42 changelog says "Replaced guest capturing-subscriber test" — but the replacement happened ALONGSIDE the original (not replacing it).
- TD-VSDD-060 sibling-sweep gap on correction-burst: CORRECTION reverted signature change but didn't sweep the orphan test.

Why it fails: Documentation-discipline drift; redundant test signals sibling-symmetry confusion.

Routing: implementer — delete redundant test at lines 1105-1129. Remove orphan comment block at lines 1076-1078.

Paper-fix risk: LOW.

### F-LP8-LOW-001 — WAT-fixture short-circuit in `dispatch_plugin_acquire_token` bypasses emission path

Surface: `crates/prism-spec-engine/src/plugin/mod.rs` lines 678-696.

Evidence:
- Core-module path returns `Ok("wat-fixture-token")` UNCONDITIONALLY for any WAT-core-module plugin.
- Emission `plugin.auth_token_parse_error` ONLY fires on Component Model path.
- Production plugins load as Component Model, but the branch is `pub fn`-reachable.

Why it fails: Structural-coverage axis applied to emission reachability under all dispatch branches.

Routing: implementer — gate WAT-fixture branch behind `#[cfg(test)]` OR add `debug_assert!(cfg!(test))` OR add warn-level audit emission.

Paper-fix risk: LOW.

### F-LP8-LOW-002 — `#[ignore]` reason cites S-PLUGIN-CI-001 story but not specific test name (SID-1 §5 borderline)

Surface: `crates/prism-spec-engine/src/plugin/mod.rs` lines 1435-1437.

Evidence:
- `#[ignore]` reason cites S-PLUGIN-CI-001 but not specific AC test name.
- Literal SID-1 §5: deferral must cite specific test name.
- Borderline: this test IS the deferred one; the un-blocker is S-PLUGIN-CI-001 AC-001 (CI infrastructure).

Why it fails: borderline literal SID-1 §5 reading. Pending intent verification.

Routing: implementer — 1-line edit tightening citation to "S-PLUGIN-CI-001 AC-001 (test_PLUGIN_MIGRATION_001_E_med_001_built_prx_loads_via_plugin_runtime un-ignored)".

Paper-fix risk: NONE.

### F-LP8-LOW-003 — Stale test-name reference at crowdstrike_oauth2_plugin_tests.rs:1460

Surface: Line 1460 cites `test_dispatch_plugin_acquire_token_response_parse_emits_audit_event` (no longer exists).

Evidence: Function-definition grep returns zero matches. Only historical doc-comment references.

Routing: implementer — 1-line comment edit to cite `test_F_LP7_MED_001_host_emit_acquire_token_parse_error_fires_unconditionally` at `plugin/mod.rs:1352`.

Paper-fix risk: NONE.

(Note: adversary's original report used `F-LP8-LOW-001` for TWO different findings — relabeled here per orchestrator routing convention; the WAT-fixture finding stays F-LP8-LOW-001, the stale-comment finding becomes F-LP8-LOW-003.)

### F-LP8-OBS-001 — Workspace test count delta (3762 ↔ 3518) deserves numerical evidence breakdown

Surface: FB-IMPL-6 report assertion "the 3762→3518 delta was a counting artifact" without numerical investigation.

Routing: orchestrator — run `cargo nextest list --workspace 2>&1 | tail -3` + `just check 2>&1 | grep -E '^test result' | tail -5` and persist breakdown.

Paper-fix risk: NONE if orchestrator actually counts.

## Probe sweep summary

| Probe | Result |
|---|---|
| `emit_auth_token_parse_error` (OLD guest helper) presence in `crates/` | CLEAN — zero matches. Only narrative refs in changelog + fix-burst-6.md (historical). |
| EC-007 cited tests existence | CLEAN — both tests exist in plugin_tests.rs:395 + 435, assert PluginError::InvalidInterface. |
| EC-008 cited tests existence | CLEAN — both tests exist in plugin_tests.rs:976 + 919. |
| S-PLUGIN-CI-001 stub readiness | CLEAN — file exists v0.1 status=draft; AC-002/003 Red Gate test names match story spec EC-006/EC-009 citations. |
| BC-2.16.002 v1.41/v1.42 sibling-sweep | CLEAN in active spec artifacts. |
| BC-INDEX v5.43/v5.44 sibling-sweep | CLEAN in active artifacts. |
| BC-2.16.002 row 37 catalog header sync | CLEAN — header label "(v1.26)" matches frontmatter v1.42. |
| BC-2.16.002 row 37 host emission site description | CLEAN — describes host wrapper, matches code. |
| Guest signature revert | CLEAN — no `plugin_id: &str` parameter on guest acquire_token/get_token. |
| Capturing-subscriber test framework consistency | CLEAN — uses tracing_subscriber::fmt() with set_default (scoped). |
| Test rename ripple (EC_002_non_2xx) | CLEAN — zero stale references. |
| SAP-1 new plugin.auth_token_parse_error emission catalog row | CLEAN — full field schema, audit role, recurrence policy present. |
| SAP-2 DTU↔TOML | N/A. |
| SID-1 ignore rationalization | DURABLE for EC-006..009. F-LP8-LOW-002 borderline. |
| POL-1, POL-3, POL-6, POL-7, POL-12, POL-22, POL-25, POL-29, POL-11 | All CLEAN in spirit. F-LP8-MED-001 raises POL-11 in test-code dimension. |

## Decay trajectory

| Pass | Findings | Severity high-water | New axis surfaced |
|---|---|---|---|
| 1 | 20 | 4 CRIT, 7 HIGH | code-level review |
| 2 | 12 | 2 CRIT, 5 HIGH | wire-up verification |
| 3 | 3 | 0 CRIT, 1 HIGH | wit-bindgen exports |
| 4 | 0 | — | code-level durability (false-CLEAN) |
| 5 | 2 | 0 CRIT, 2 HIGH | structural-coverage verification |
| 6 | 3 | 0 CRIT, 0 HIGH, 2 MED, 1 LOW | EC-test-vs-spec fidelity (test-body) |
| 7 | 3 | 0 CRIT, 0 HIGH, 2 MED, 1 LOW | fidelity sub-dims: spec-emission + deferral-specificity + sibling-symmetry |
| 8 | 7 | 0 CRIT, 0 HIGH, 4 MED, 2 LOW, 1 OBS | paper-fix axis extended to test code (POL-11 in tests); error-variant semantic fidelity; spec format-specifier accuracy; correction-burst orphan sweep; emission reachability under all branches |

Severity high-water remains decisively below HIGH for 3 passes. Finding count UPTICK from 3 → 7 (the FB-IMPL-6 CORRECTION introduced new artifacts that fresh-context scrutinizes for the first time).

## Recommended next action

Dispatch FB-IMPL-7 to close 7 findings.

## Total counts

| Severity | Count |
|---|---|
| CRIT | 0 |
| HIGH | 0 |
| MED | 4 |
| LOW | 2 |
| OBS | 1 |
| **TOTAL** | **7** |
