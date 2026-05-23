---
document_type: adversary-pass-report
cascade: PLUGIN-MIGRATION-001-E LOCAL
pass_number: 6
date: 2026-05-22
feature_head: 639d89e1
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
decay_trajectory: "20 → 12 → 3 → 0 → 2 → 3"
new_axis_surfaced: "EC-test-vs-spec fidelity + sibling-WIT-file sweep"
inputs:
  - .factory/cycles/wave-0-plugin-prereqs/PLUGIN-MIGRATION-001-E/local-pass-{1,2,3,4,5}.md
  - .factory/cycles/wave-0-plugin-prereqs/PLUGIN-MIGRATION-001-E/fix-burst-{1,2,3,4}.md
  - .factory/stories/PLUGIN-MIGRATION-001-E-crowdstrike-oauth2-refresh-on-401-prx-wasm-plugin.md
  - crates/prism-spec-engine/plugins/crowdstrike-oauth2/src/lib.rs
  - crates/prism-spec-engine/plugins/crowdstrike-oauth2/wit/sensor-auth.wit
  - crates/prism-spec-engine/wit/prism-{sensor,infusion,action}-plugin.wit
  - .github/workflows/ci.yml
input-hash: "[live-pass-6]"
---

# PLUGIN-MIGRATION-001-E — LOCAL Adversary Pass-6

**Date:** 2026-05-22
**Feature HEAD:** `639d89e1`
**Develop HEAD baseline (unchanged):** `f19575ff`
**Cascade state at start:** streak 0/3 (reset by pass-5), attempting 0/3 → 1/3
**Decay trajectory:** 20 → 12 → 3 → 0 (false CLEAN) → 2 → 3 (this pass)

## Streak after this pass: stays at **0/3**

```
CLEAN (strict):    no
CLEAN (PR-merge):  no
```

Reason: 3 findings (0 CRIT, 0 HIGH, 2 MED, 1 LOW). CLEAN(strict) requires zero ANY severity → fails. CLEAN(PR-merge) requires zero CRIT+HIGH+MED → fails (2 MED).

## Part A — Durability of pass-3, pass-4, pass-5 closures

| Finding | Closure mechanism in HEAD `639d89e1` | Verdict (structural-coverage axis) |
|---|---|---|
| F-LP3-HIGH-001 (wit-bindgen Guest exports) | `impl Guest for Component` + `export!(Component)` present in `lib.rs::host_impl`; **now load-bearing** via `wasm32-compile-check` CI job | DURABLE (structurally hardened) |
| F-LP3-MED-001 (validate_and_construct_auth_providers + 4 tests) | Function present; 4 tests `test_validate_and_construct_auth_providers_{happy_path,typo_returns_error,empty_returns_empty_map,mixed_sensors_one_with_auth_plugin}` in `plugin_boot_tests.rs` | DURABLE |
| F-LP3-LOW-001 (event_type field + BC-2.16.002 row) | `event_type = "plugin_auth_provider_constructed"` emission present; catalog row 113 present | DURABLE |
| F-LP5-HIGH-001 (wit-bindgen wasm32 compile gate + WIT-syntax fix) | `sensor-auth.wit` types inside `host` interface; `wasm32-compile-check` CI job unconditional + has reachability assertion | DURABLE — CI structural change cannot silently drop the job |
| F-LP5-HIGH-002 (HostInterface trait + MockHost + 9 tests) | `trait HostInterface` defined; `WasmHost` (wasm32) + `MockHost` (test); `acquire_token`/`get_token` accept `&impl HostInterface`; 11 tests present (2 pre-existing + 9 new, all variant-matching) | MOSTLY DURABLE with two coverage gaps — see F-LP6-MED-001 and F-LP6-LOW-001 |

**Regression count: 0. Paper-fix count: 0.** Pass-5 HIGH closures are structurally durable. New findings below are coverage-completeness + sibling-sweep gaps, NOT paper-fix recurrences.

## Part B — NEW findings

### F-LP6-MED-001 — `test_acquire_token_EC_002_non_2xx_returns_response_parse` does NOT cover the spec scenario for EC-002; the 200-with-invalid-JSON branch is unexercised [MED, HIGH confidence]

**Surface:** `crates/prism-spec-engine/plugins/crowdstrike-oauth2/src/lib.rs` `acquire_token` (the `serde_json::from_str(body_str).map_err(...)` branch after the 2xx status check); `mod tests::test_acquire_token_EC_002_non_2xx_returns_response_parse`.

**Evidence:**
- Story spec EC-002 row (PLUGIN-MIGRATION-001-E §Edge Cases): "`POST /oauth2/token` returns HTTP 200 but response body is not valid JSON. Expected: Plugin returns `AuthError::ResponseParse`."
- Production code path for EC-002 (200 + invalid JSON) is the `serde_json::from_str(body_str).map_err(|e| AuthError::ResponseParse(e.to_string()))` call — a separate branch from the non-2xx-status branch.
- `test_acquire_token_EC_002_non_2xx_returns_response_parse` constructs HTTP 503 with body `"Service Unavailable"` (non-JSON, non-2xx). The 503 fails the `response.status < 200 || response.status >= 300` check BEFORE reaching `serde_json::from_str`. Test exercises the non-2xx branch, NOT the JSON-parse branch named by EC-002.
- Story spec EC-002 "Red Gate Test" column cites this test name — spec-test drift.
- A regression flipping `AuthError::ResponseParse(e.to_string())` to `AuthError::Internal(e.to_string())` on JSON parse failure would compile clean and pass all 11 tests.

**Why it fails:** Two interlocked issues:
1. **Coverage gap**: The story spec's named EC-002 scenario (200 + invalid JSON) has zero test coverage.
2. **Spec-test drift**: The story spec EC-002 table cites a test name that exercises a DIFFERENT scenario than the spec describes.

This is not a paper-fix of a prior HIGH — F-LP5-HIGH-002 was the trait-extraction + 9-test addition, and FB-IMPL-4 delivered 9 tests as claimed. But spec-vs-test fidelity wasn't audited against the story's EC table.

**Routing:** implementer — add `test_acquire_token_EC_002_invalid_json_returns_response_parse` with `push_http_response(200, "this is not JSON {[")` and assert `matches!(err, AuthError::ResponseParse(_))`. Rename existing test to `test_acquire_token_non_2xx_returns_response_parse` (drop EC-002 marker), OR amend story EC table to add a new EC row for non-2xx behavior. Production-grade default: do both.

**Paper-fix risk:** MEDIUM. A doc-comment closure ("renamed the test") without adding actual JSON-parse coverage would re-instate the gap.

---

### F-LP6-MED-002 — Sibling-WIT files (`prism-sensor-plugin.wit`, `prism-infusion-plugin.wit`, `prism-action-plugin.wit`) still carry the top-level-type anti-pattern that broke `sensor-auth.wit`; sibling-sweep gap (POL-29 / S-7.01 partial-fix discipline) [MED, HIGH confidence]

**Surface:** `crates/prism-spec-engine/wit/prism-sensor-plugin.wit`, `crates/prism-spec-engine/wit/prism-infusion-plugin.wit`, `crates/prism-spec-engine/wit/prism-action-plugin.wit`.

**Evidence:**
- FB-IMPL-4 root-cause for F-LP5-HIGH-001: "wit-bindgen 0.51+ requires type definitions inside `interface` blocks". Fix moved `record http-response`, `enum log-level`, `variant auth-error` from package-top-level into the `host` interface in `sensor-auth.wit`.
- `prism-sensor-plugin.wit` lines 14-34 still declare top-level `enum log-level` + `record http-response` + `record page-result` OUTSIDE any interface block — exact pattern that broke `sensor-auth.wit`.
- `prism-infusion-plugin.wit` and `prism-action-plugin.wit` follow the same authoring pattern (verified via Glob + parallel file structure).
- None of the 3 sibling WIT files are currently consumed by `wit_bindgen::generate!` (only `crowdstrike-oauth2` uses the macro today), so they don't break the build TODAY.
- BUT: the story spec frontmatter `subsystems: [SS-01, SS-16, SS-17]` lists SS-17 (WASM Plugin Runtime) as owned scope. Sibling WIT files are SS-17 artifacts. A future story implementing a sensor-plugin or infusion-plugin via wit-bindgen — the documented future roadmap for AD-019 plugin types — will hit the EXACT structural compile failure FB-IMPL-4 just fixed.

**Why it fails:** Per S-7.01 partial-fix regression discipline:
- **(b) Sibling files in the same architectural layer**: 3 sibling WIT files (same `crates/prism-spec-engine/wit/` directory, same plugin-WIT architectural layer) carry the same defect pattern.
- Blast radius = 3 files → MEDIUM (per S-7.01's "Blast radius = 2+ files: HIGH" demoted because files are not currently compiled — defect is latent, not active).
- Per Canonical Principle Rule 4: the right default is to fix the 3 sibling files in-scope (~5 minutes each).
- Per POL-29 within-FB sibling-sweep discipline: when FB-IMPL-4 fixed the WIT-syntax pattern, the sweep should have covered all 4 WIT files in the same directory.

**Routing:** implementer — apply the same top-level-types-into-interface restructure to the 3 sibling files. No code-side change needed (no wit-bindgen consumers yet). 3-file mechanical fix that prevents the same defect class from re-emerging. **Intent verification:** if these 3 WITs are documentation-only and will be re-authored from scratch when wit-bindgen consumption begins, demote to LOW with `(pending intent verification)`. Default action under production-grade lens: fix now.

**Paper-fix risk:** LOW. Structural fix; once applied, the structural compile-fail gate prevents regression.

---

### F-LP6-LOW-001 — EC-004 test covers only the missing-`expires_in` case; the zero-`expires_in` case is uncovered despite being in the story spec's EC-004 expected-behavior column [LOW, HIGH confidence]

**Surface:** `crates/prism-spec-engine/plugins/crowdstrike-oauth2/src/lib.rs` `acquire_token`'s `.filter(|&v| v > 0).unwrap_or(1799)` chain; `mod tests::test_acquire_token_EC_004_missing_expires_in_defaults_to_1799`.

**Evidence:**
- Story spec EC-004 row: "Token response `expires_in` field is missing **or zero**" — both conditions explicitly named.
- Production code:
  ```rust
  let expires_in: u64 = json
      .get("expires_in")
      .and_then(|v| v.as_u64())
      .filter(|&v| v > 0)        // handles the zero case
      .unwrap_or(1799);          // handles the missing case
  ```
- `test_acquire_token_EC_004_missing_expires_in_defaults_to_1799` uses body without `expires_in`. Exercises only the `unwrap_or` branch.
- A regression removing `.filter(|&v| v > 0)` would compile clean and pass all 11 tests. Zero-`expires_in` server response would then produce `expires_at = now + 0.saturating_sub(30) = now`, immediately stale-cached → infinite token-refresh loops in production.

**Why it fails:** Coverage-completeness gap. Production behavior is correct, but the EC-004 test name claims to cover EC-004 completely and does not. LOW because the missing case IS covered and the filter is defensive logic — but a single-line removal would be undetected.

**Routing:** implementer — add sibling test `test_acquire_token_EC_004_zero_expires_in_defaults_to_1799` using body with `expires_in: 0` and assert `expires_at = now + 1769`. ~10 lines.

**Paper-fix risk:** LOW. Purely additive (one more test).

## Probe sweep summary (negative results)

| Probe | Result |
|---|---|
| HostInterface vs host_impl fidelity | MockHost lacks KV size-limit *simulation* but production delegates to host_impl::kv_set which enforces 1MB at host. EC-005 tests propagation, not size logic. Not a finding. |
| `host_impl::*` native stub removal | `Grep('host_impl::', crates/)` returns only doc-comment refs + legitimate WasmHost callsites. No leaked direct calls. CLEAN. |
| wit-bindgen WIT-syntax ripple | `sensor-auth.wit`'s `use host.{auth-error}` is valid wit-bindgen 0.51 syntax. Guest trait regeneration handled — `impl Guest for Component` uses regenerated scoped type. 3 sibling WIT files surfaced as F-LP6-MED-002. |
| CI workflow correctness | wasm32-compile-check unconditional, reachability assertion present, target spec correct, timeout 10min generous. CLEAN. |
| Cargo.lock churn | wit-bindgen 0.51.0 + 0.57.1 (transitive); deny.toml warns on duplicates; no known advisories. Acceptable. |
| SAP-1 tracing emission catalog | Plugin crate adds zero new tracing emissions. N/A. |
| SAP-2 DTU↔TOML schema parity | No .prism/specs/sensors/*.toml diffs. N/A. |
| SID-1 no-ignored-test rationalization | F-LP5-HIGH-001 option (a) deferral to S-PLUGIN-CI-001 properly cited in fix-burst-4. Option (b) delivered as wasm32-compile-check job. MED-001's `#[ignore]`'d test continues to cite S-PLUGIN-CI-001. Compliant. |
| POL-1, POL-3, POL-6, POL-12, POL-16, POL-22, POL-23, POL-25 | All CLEAN (see full rubric in pass body). |
| POL-29 within-FB sibling-sweep | F-LP6-MED-002 IS the POL-29 violation — sibling WIT files weren't swept. |

## Novelty Assessment

MEDIUM novelty. Pass-6 surfaces a NEW axis: **EC-test-vs-spec fidelity** — does a test that claims to cover EC-N actually exercise the EC-N named scenario? F-LP6-MED-001 catches EC-002 drift (test covers non-2xx but spec names 200+invalid-JSON). F-LP6-LOW-001 catches EC-004 partial coverage (spec names missing-or-zero; test covers only missing). Two instances elevate this to a pattern flag for future cascades.

F-LP6-MED-002 catches a sibling-sweep gap under existing POL-29 / S-7.01 — well-known discipline, latent defect.

The structural-coverage axis pass-5 introduced is now stable and load-bearing. EC-test-vs-spec-fidelity axis from pass-6 should propagate as a standing cascade discipline.

## Decay trajectory

| Pass | Findings | Severity high-water | New axis surfaced |
|---|---|---|---|
| 1 | 20 | 4 CRIT, 7 HIGH | code-level review |
| 2 | 12 | 2 CRIT, 5 HIGH | wire-up verification |
| 3 | 3 | 0 CRIT, 1 HIGH | wit-bindgen exports + extraction |
| 4 | 0 | — | code-level durability sample |
| 5 | 2 | 0 CRIT, 2 HIGH | **structural-coverage verification** |
| 6 | 3 | 0 CRIT, 0 HIGH, 2 MED, 1 LOW | **EC-test-vs-spec fidelity + sibling-WIT sweep** |

Trajectory: severity high-water decisively below HIGH for 1 pass. Cascade structurally hardened on wit-bindgen + host-interface axes; NOT yet hardened on EC-test-vs-spec fidelity axis.

## Recommended next action

Dispatch **FB-IMPL-5** to implementer with three closure tasks:

1. **F-LP6-MED-001** — Add `test_acquire_token_EC_002_invalid_json_returns_response_parse` (HTTP 200 + non-JSON body). Either rename existing test to `test_acquire_token_non_2xx_returns_response_parse` (update story EC-table reference), OR amend story EC table for non-2xx behavior. Production-grade default: do both.
2. **F-LP6-MED-002** — Apply top-level-types-into-interface restructure to `prism-sensor-plugin.wit`, `prism-infusion-plugin.wit`, `prism-action-plugin.wit`. ~5 min/file. Intent verification: if docs-only and to-be-rewritten, demote to LOW (pending intent verification); else fix in-scope per Rule 4.
3. **F-LP6-LOW-001** — Add `test_acquire_token_EC_004_zero_expires_in_defaults_to_1799`. ~10 lines.

Pass-7 (after FB-IMPL-5) attempts streak 0/3 → 1/3. Standing probes for pass-7+: structural-coverage axis (pass-5) + EC-test-vs-spec-fidelity axis (pass-6) to prevent recurrence.

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
