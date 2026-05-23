---
document_type: adversary-pass-report
cascade: PLUGIN-MIGRATION-001-E LOCAL
pass_number: 7
date: 2026-05-23
feature_head: 7702ea78
develop_head_baseline: f19575ff
streak_before: 0/3
streak_after: 0/3
clean_strict: false
clean_pr_merge: false
findings_total: 3
findings_by_severity:
  CRIT: 0
  HIGH: 0
  MED: 2
  LOW: 1
  OBS: 0
  PROCESS-GAP: 0
decay_trajectory: "20 → 12 → 3 → 0 → 2 → 3 → 3"
standing_axes_applied:
  - structural-coverage (pass-5)
  - EC-test-vs-spec fidelity (pass-6)
  - partial-fix regression discipline (S-7.01)
new_sub_dimensions_surfaced:
  - "spec-named-artifact existence (emissions, deferral citations) — F-LP7-MED-001"
  - "deferral-citation specificity per SID-1 §5 — F-LP7-MED-002"
  - "test-assertion sibling-symmetry within EC family — F-LP7-LOW-001"
inputs:
  - .factory/cycles/wave-0-plugin-prereqs/PLUGIN-MIGRATION-001-E/local-pass-{1,2,3,4,5,6}.md
  - .factory/cycles/wave-0-plugin-prereqs/PLUGIN-MIGRATION-001-E/fix-burst-{1,2,3,4,5}.md
  - .factory/stories/PLUGIN-MIGRATION-001-E-crowdstrike-oauth2-refresh-on-401-prx-wasm-plugin.md (v1.2)
  - crates/prism-spec-engine/plugins/crowdstrike-oauth2/src/lib.rs
  - crates/prism-spec-engine/plugins/crowdstrike-oauth2/wit/sensor-auth.wit
  - crates/prism-spec-engine/wit/prism-{sensor,infusion,action}-plugin.wit
  - crates/prism-spec-engine/src/plugin/discovery.rs
  - crates/prism-spec-engine/src/plugin/mod.rs
  - crates/prism-spec-engine/src/plugin/host_functions.rs
  - crates/prism-spec-engine/src/plugin/loader.rs
  - crates/prism-spec-engine/tests/crowdstrike_oauth2_plugin_tests.rs
  - .github/workflows/ci.yml
input-hash: "[live-pass-7]"
---

# PLUGIN-MIGRATION-001-E — LOCAL Adversary Pass-7

**Date:** 2026-05-23
**Feature HEAD:** `7702ea78`
**Develop HEAD baseline (unchanged):** `f19575ff`
**Cascade state at start:** streak 0/3 (held since pass-5 reset), attempting 0/3 → 1/3
**Decay trajectory:** 20 → 12 → 3 → 0 (false CLEAN) → 2 → 3 → 3 (this pass)

## Streak after this pass: stays at **0/3**

```
CLEAN (strict):    no
CLEAN (PR-merge):  no
```

Reason: 3 findings (0 CRIT, 0 HIGH, 2 MED, 1 LOW). CLEAN(strict) requires zero ANY severity → fails. CLEAN(PR-merge) requires zero CRIT+HIGH+MED → fails (2 MED).

## Part A — Durability of pass-3 / pass-4 / pass-5 / pass-6 closures

Sampled 6 closures across the cascade (>50% coverage):

| Finding | Closure mechanism in HEAD `7702ea78` | Structural-coverage verdict | EC-fidelity verdict |
|---|---|---|---|
| F-LP3-HIGH-001 (wit-bindgen Guest exports) | `impl Guest for Component` + `export!(Component)` in `lib.rs::host_impl`; `wasm32-compile-check` CI job + reachability assertion in `ci.yml` `verify-workflow-structure` step | DURABLE | N/A |
| F-LP5-HIGH-001 (wasm32 compile gate + WIT-syntax fix) | `sensor-auth.wit` types nested in `host` interface; CI job unconditional with runtime-computed grep reachability assertion | DURABLE | N/A |
| F-LP5-HIGH-002 (HostInterface trait + MockHost + tests) | `trait HostInterface` defined; `WasmHost` (wasm32) + `MockHost` (test); `acquire_token`/`get_token` accept `&impl HostInterface`; EC-001..EC-005 tests + cache tests all variant-matching | DURABLE | DURABLE for EC-001..EC-005 — sibling-symmetry micro-gap raised as F-LP7-LOW-001 |
| F-LP6-MED-001 (EC-002 invalid_json test + rename) | `test_acquire_token_EC_002_invalid_json_returns_response_parse` uses `push_http_response(200, "this is not JSON {[")`, asserts `matches!(err, AuthError::ResponseParse(_))` + non-empty detail. Renamed test `test_acquire_token_non_2xx_returns_response_parse` carries doc comment explaining defense-in-depth status-check branch. Story spec v1.2 EC-002 row updated. | DURABLE | DURABLE for the JSON-parse branch; spec-emission gap raised as F-LP7-MED-001 |
| F-LP6-MED-002 (sibling-WIT files restructured) | `prism-{sensor,infusion,action}-plugin.wit` all have types-in-interfaces; worlds renamed to `{sensor,infusion,action}-plugin-world`. Discovery.rs validates by EXPORT names (kebab-case `name`, `version`, `fetch-page` etc.) NOT world names, so renames are non-load-bearing for current validation | DURABLE | DURABLE |
| F-LP6-LOW-001 (EC-004 zero case test) | `test_acquire_token_EC_004_zero_expires_in_defaults_to_1799` uses `expires_in: 0`, asserts `expires_at == now + 1769` (= now + 1799 - 30) | DURABLE | DURABLE |

**Regression count: 0. Paper-fix count: 0.** All sampled closures structurally durable under both standing axes. New findings below are NEW sub-dimensions of the fidelity axis, not recurrences.

## Part B — NEW findings

### F-LP7-MED-001 — Story spec EC-002 names a tracing emission `event_type = "plugin.auth_token_parse_error"` that does not exist in any code path; spec-vs-impl drift [MED, HIGH confidence]

**Surface:** Story spec EC-002 "Expected Behavior" column; guest crate `crates/prism-spec-engine/plugins/crowdstrike-oauth2/src/lib.rs` (`acquire_token` JSON parse error branch); host dispatch `crates/prism-spec-engine/src/plugin/mod.rs` `dispatch_plugin_acquire_token`.

**Evidence:**
- Story spec EC-002 expected-behavior: "Plugin returns `AuthError::ResponseParse`; host logs `tracing::error!(event_type = "plugin.auth_token_parse_error")`; no token cached".
- Grep `rg 'auth_token_parse_error|plugin\.auth_token|token_parse_error' .worktrees/PLUGIN-MIGRATION-001-E/` returns ZERO matches across the entire worktree.
- The event_type does not exist in:
  - The plugin guest (`crowdstrike-oauth2/src/lib.rs`) — no `tracing::*!` emissions at all
  - The host dispatch path (`plugin/mod.rs` `dispatch_plugin_acquire_token`) — uses `?` propagation on `func.call(...)`, no emission
  - Any other crate
- Inverse SAP-1 pattern: spec commits to a structured event with specific `event_type` but no emission site exists.
- An operator searching audit logs for `event_type = "plugin.auth_token_parse_error"` after a token-parse failure will find nothing. Spec promises observability the code does not deliver.

**Why it fails:**
- Pass-6 surfaced EC-test-vs-spec fidelity (test-body dimension). This is a NEW sub-dimension: spec-to-emission fidelity.
- Source-of-Truth Precedence Rule 7: SPEC wins; code is brought up to spec. Implementer adds emission + BC-2.16.002 catalog row.
- Audit observability gap is real.

**Routing:** implementer (code emission + BC-2.16.002 in-burst per Companion Principle precedent "BC ↔ tracing-emission catalog drift discovered during implementation") with capturing-tracing-subscriber test. Concrete action: `tracing::error!(event_type = "plugin.auth_token_parse_error", plugin_id = %plugin_id, error = %e, "plugin auth token JSON parse failed")` at the host-side `dispatch_plugin_acquire_token` `Err(...)` branch when the underlying `AuthError::ResponseParse` is detected; BC-2.16.002 catalog row added in same commit; capturing-subscriber unit/integration test asserting the event fires.

**Paper-fix risk:** MEDIUM. A doc-comment-only closure ("event_type added in spec footnote") without actual emission + catalog row + capturing-subscriber test would recurse this pattern.

---

### F-LP7-MED-002 — EC-006..EC-009 "Test Reference" cells use plural-vague deferral citation that violates SID-1 §5 specificity discipline [MED, HIGH confidence]

**Surface:** Story spec (v1.2) lines 479-482 (EC-006 through EC-009 "Test Reference" column).

**Evidence:**
- Lines 479-482 each cite the same string: "wasm32 Guest impl / WAT-fixture / integration tests (not closed in FB-IMPL-4)".
- SID-1 §5: "'Deferred to non-ignored test' is ONLY valid if a SPECIFIC story ID and SPECIFIC test name are cited in the deferral."
- The citation:
  - Names NO specific test function
  - Names NO specific story ID (FB-IMPL-4 is a historical fix-burst artifact, not a forward-pointing scope identifier)
  - Uses plural categorical phrasing ("integration tests") with no specific file or test
- Grep across `crates/prism-spec-engine/tests/` confirms no test named with `_EC_006_` / `_EC_007_` / `_EC_008_` / `_EC_009_` convention.
- Detailed audit:
  - EC-006 (plugin binary missing): no test asserting `PluginRuntime::load_plugin` failure + ERROR-level emission + boot continues
  - EC-007 (WIT validation fails): tests exist for `PluginError::InvalidInterface` (e.g., `plugin_tests.rs:248`) but not under `_EC_007_*` convention
  - EC-008 (allowlist rejection of `/oauth2/token`): tests exist for `PluginError::SandboxViolation` (`loader.rs:307` + `plugin_tests.rs:248`), no test specifically driving EC-008 (OAuth2 token endpoint blocked by allowlist with plugin returning `AuthError::Internal`)
  - EC-009 (double 401 → `SpecEngineError::AuthRefreshFailed`): no test driving the plugin path through `PipelineExecutor::issue_request_with_retry` for the double-401 case

**Why it fails:**
- SID-1 §5 explicitly requires SPECIFIC story ID + SPECIFIC test name.
- No machine-greppable way to verify EC-006..EC-009 coverage.
- Production-Grade Default Rule 1 (no MVP-driven deferrals) + Rule 6 (no "TODO for future story" when answerable in scope): EC-007 + EC-008 are answerable now — existing `PluginError::InvalidInterface` and `PluginError::SandboxViolation` tests just need EC-table citation.
- EC-006 and EC-009 are partially out-of-scope; legitimate deferral IF citing specific story ID.

**Routing:** implementer (story spec EC-table edit per Companion Principle implementer-test-citation-metadata precedent). Concrete action:
- EC-007: cite specific existing test from `plugin_tests.rs` where `PluginError::InvalidInterface` is exercised.
- EC-008: cite specific existing test exercising `PluginError::SandboxViolation` against URL allowlist; if no specific OAuth2-endpoint allowlist test exists, add one in this fix-burst (~10 min).
- EC-006: cite specific deferred story ID + specific future test name (e.g., "deferred to S-PLUGIN-CI-001 as `test_S_PLUGIN_CI_001_001_missing_prx_at_boot_continues_with_error_log`"). If no such story exists yet, surface to orchestrator for routing.
- EC-009: cite specific story (`S-PLUGIN-CI-001` or `S-PLUGIN-PREREQ-B` if AC-5 covers the non-plugin path) with specific test name.

Story spec bump v1.2 → v1.3.

**Paper-fix risk:** LOW. The fix is enumeration + specific citations. A paper-fix would re-substitute plural-vague phrasing — easy to detect.

---

### F-LP7-LOW-001 — EC-003 test does not assert "no token cached" despite spec EC-003 listing it and sibling tests EC-001 + EC-002 both asserting it [LOW, HIGH confidence]

**Surface:** `crates/prism-spec-engine/plugins/crowdstrike-oauth2/src/lib.rs` `test_acquire_token_EC_003_missing_access_token_returns_response_parse`; story spec EC-003 row.

**Evidence:**
- Story spec EC-003: "Token response is missing `access_token` field | Plugin returns `AuthError::ResponseParse` with detail 'missing access_token field'; no token cached".
- The expected-behavior column includes "no token cached" — same invariant as EC-001 ("no token cached") and EC-002 ("no token cached").
- `test_acquire_token_EC_001_401_returns_invalid_credentials` asserts `host.kv_store.borrow().get("token") == None`.
- `test_acquire_token_EC_002_invalid_json_returns_response_parse` asserts `host.kv_store.borrow().get("token") == None`.
- `test_acquire_token_EC_003_missing_access_token_returns_response_parse` asserts ONLY variant + detail message. Does NOT assert no-token-cached.
- Production code path: missing-access_token error returned BEFORE any `kv_set` call — invariant holds by code structure. But a regression moving access_token extraction AFTER kv_set calls would not be caught.
- Coverage-completeness gap: test name + spec promise imply full EC-003 verification; assertions cover only 2 of 3 spec promises.

**Why it fails:**
- Pass-6's EC-test-vs-spec fidelity axis applied to scenarios; this pass extends to ASSERTION COMPLETENESS.
- EC-001 + EC-002 tests both assert no-token-cached; EC-003 does not, despite spec parity.
- Severity LOW because production code-path naturally short-circuits before any kv_set + sibling tests cover the pattern + missing assertion is a sibling-symmetry gap rather than actively undetected defect.

**Routing:** implementer — add 3-line kv_store assertion mirroring EC-001/EC-002 pattern:
```rust
assert_eq!(
    host.kv_store.borrow().get("token"),
    None,
    "EC-003: token MUST NOT be cached when access_token field is missing"
);
```

**Paper-fix risk:** LOW. Single-assertion addition; bool-checkable.

## Probe sweep summary (negative results)

| Probe | Result |
|---|---|
| WIT world rename ripple (sensor-plugin-world / infusion-plugin-world / action-plugin-world) | `rg` finds names ONLY in the 3 WIT files themselves. No external references. CLEAN. |
| OLD world names (sensor-plugin / infusion-plugin / action-plugin as world identifiers) | `rg 'world sensor-plugin\b\|world infusion-plugin\b\|world action-plugin\b'` returns matches ONLY in `.worktrees/S-3.09/` and `.worktrees/W3-FIX-S307-001/` (other branches on develop baseline). PLUGIN-MIGRATION-001-E worktree zero stale references. CLEAN. |
| Test rename ripple | Zero stale references to old test name in code/tests/CI/factory artifacts in this worktree. CLEAN. |
| wasm32-compile-check job + reachability assertion | Job present, runtime-computed reachability assertion, generous timeout. CLEAN. |
| WAT-fixture WIT-world rename impact | WAT uses export names, discovery.rs validates by EXPORT names not world names. CLEAN. |
| WIT inter-file `use` consistency | All `use` clauses match new interface-scoped type locations. CLEAN. |
| EC-001 / EC-002 / EC-003 / EC-004 / EC-005 fidelity | EC-001/002/004/005 CLEAN; EC-003 sibling-assertion gap raised as F-LP7-LOW-001; EC-002 spec-emission gap raised as F-LP7-MED-001. |
| EC-006..EC-009 deferral specificity | FAIL — raised as F-LP7-MED-002. |
| SAP-1 / SAP-2 / SID-1 standing probes | SAP-1 N/A for new-emission direction; INVERSE direction surfaced as F-LP7-MED-001. SAP-2 N/A. SID-1 §5 deferral-specificity surfaced as F-LP7-MED-002. |
| POL-1, POL-3, POL-6, POL-12, POL-22, POL-25, POL-29, POL-11 | All CLEAN per detailed pass body. |

## Novelty Assessment

MEDIUM novelty. Three findings at lower severity but all open NEW sub-dimensions of the EC-test-vs-spec fidelity axis pass-6 introduced:

1. **F-LP7-MED-001 — Spec-to-emission fidelity** (inverse SAP-1): spec commits to a `tracing::*!` event_type but the code doesn't emit it. New audit-observability sub-dimension.
2. **F-LP7-MED-002 — Deferral-citation specificity (SID-1 §5)**: EC table's deferred ECs use plural-vague phrasing. New deferral-discipline sub-dimension.
3. **F-LP7-LOW-001 — Test-assertion sibling-symmetry**: EC-003 misses an assertion EC-001 and EC-002 both include. New within-test sibling-symmetry sub-dimension.

The structural-coverage axis (pass-5) is now stable. The fidelity axis has now decomposed into:
- Pass-6 dimension: does test BODY exercise spec scenario?
- Pass-7 sub-dim A: do spec's named ARTIFACTS (emissions, deferral citations) exist?
- Pass-7 sub-dim B: do test ASSERTIONS cover all spec PROMISES (sibling-symmetry)?

These sub-dimensions should propagate as standing cascade discipline.

## Decay trajectory

| Pass | Findings | Severity high-water | New axis surfaced |
|---|---|---|---|
| 1 | 20 | 4 CRIT, 7 HIGH | code-level review |
| 2 | 12 | 2 CRIT, 5 HIGH | wire-up verification |
| 3 | 3 | 0 CRIT, 1 HIGH | wit-bindgen exports + extraction |
| 4 | 0 | — | code-level durability sample (false-CLEAN) |
| 5 | 2 | 0 CRIT, 2 HIGH | **structural-coverage verification** |
| 6 | 3 | 0 CRIT, 0 HIGH, 2 MED, 1 LOW | **EC-test-vs-spec fidelity (test-body)** |
| 7 | 3 | 0 CRIT, 0 HIGH, 2 MED, 1 LOW | **fidelity sub-dims: spec-emission + deferral-specificity + sibling-symmetry** |

Severity high-water remains decisively below HIGH for 2 passes. Finding count flat at 3 — but the THREE findings are NEW dimensions, not recurrences. Cascade in refinement phase: structural defects extinct; remaining gaps are fidelity-axis sub-dimensions.

## Recommended next action

Dispatch **FB-IMPL-6** to implementer with three closure tasks:

1. **F-LP7-MED-001** — Add `tracing::error!(event_type = "plugin.auth_token_parse_error", plugin_id = %plugin_id, error = %e, "plugin auth token JSON parse failed")` at host-side dispatch `Err(...)` branch when `AuthError::ResponseParse`. Add BC-2.16.002 catalog row in same commit. Add capturing-tracing-subscriber test asserting event fires.

2. **F-LP7-MED-002** — Amend story spec EC-006/EC-007/EC-008/EC-009 with SID-1-compliant specific citations. Story spec v1.2 → v1.3.

3. **F-LP7-LOW-001** — Add 3-line kv_store assertion to `test_acquire_token_EC_003_*`.

Pass-8 (after FB-IMPL-6) attempts streak 0/3 → 1/3.

## Total counts

| Severity | Count |
|---|---|
| CRIT | 0 |
| HIGH | 0 |
| MED | 2 |
| LOW | 1 |
| OBS | 0 |
| PROCESS-GAP | 0 |
| **TOTAL** | **3** |
