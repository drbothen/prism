---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: state-manager
timestamp: "2026-05-11T22:30:00Z"
phase: 3
inputs:
  - "crates/prism-spec-engine/src/pipeline.rs"
  - "crates/prism-spec-engine/src/auth_provider.rs"
  - "crates/prism-spec-engine/src/validation.rs"
  - "crates/prism-spec-engine/tests/pipeline_http_integration.rs"
  - "crates/prism-spec-engine/tests/pipeline_oauth_retry.rs"
  - ".factory/cycles/wave-4-operations/adversarial-reviews/S-PLUGIN-PREREQ-B-pass-4.md"
input-hash: "d5a12e4"
traces_to: ".factory/stories/S-PLUGIN-PREREQ-B-real-pipeline-executor.md"
pass: 4
previous_review: "S-PLUGIN-PREREQ-B-pass-4.md"
review_level: LOCAL
target_artifact: S-PLUGIN-PREREQ-B
fix_burst_for_pass: 4
target_sha: d5a12e4a
base_sha: a6895d7a
verdict: CLOSED
finding_summary_closed:
  critical: 0
  high: 1
  medium: 2
  low: 4
  obs_acknowledged: 0
prior_passes: "pass-4 BLOCKED-hard 7 findings (1H+2M+4L); caught pass-3 false-CLEAN"
---

# Adversarial Review: S-PLUGIN-PREREQ-B Fix-Burst-3 Closure Report (Pass 4 Remediation)

**Verdict: CLOSED** — All 7 actionable findings from LOCAL pass-4 closed at implementer commit
`d5a12e4a`. 0 OBS items acknowledged. Streak stays 0/3 (fix-bursts do not advance streak;
adversary pass-5 advances streak). STATE+HANDOFF v7.138→v7.139. Story v1.2→v1.3
(red_gate_tests 29→33). 1 new TD filed: TD-S-PLUGIN-PREREQ-B-005 P2.

## Finding ID Convention

Finding IDs for this closure report use the pass-4 source format: `F-LP4-<SEV>-<SEQ>`
(LOCAL pass-4 findings from S-PLUGIN-PREREQ-B-pass-4.md). This file records closure
dispositions only — no new adversary findings are raised here.

## Part A — Fix Verification (pass >= 2 only)

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| F-LP4-HIGH-001 | HIGH | RESOLVED | Two defense layers: `validation.rs:247` rejects `fan_out_batch_size=0` (validator-layer); `pipeline.rs:451` adds `.max(1)` runtime clamp (defense-in-depth). 3 new Red Gate tests verify: rejection path, accept-1, accept-None (defaults to 100). Paper-fix-proof test verifies validation path specifically. |
| F-LP4-MED-001 | MEDIUM | RESOLVED + TD filed | 16 test fixture sites in `pipeline_http_integration.rs` + 2 sites in `pipeline_oauth_retry.rs` updated to `Client::builder().timeout(Duration::from_secs(30)).build()` builder pattern. `pipeline.rs:93-97` doc-comment added citing TD-S-PLUGIN-PREREQ-B-005 P2 for production wiring (boot.rs/chassis scope = PREREQ-D). Convention established in test fixtures. |
| F-LP4-MED-002 | MEDIUM | RESOLVED | `pipeline_http_integration.rs:1538` — `test_BC_2_16_002_execute_aborts_at_max_pages_per_step` added; triggers page-cap at page 1001. Paper-fix-proof style: test drives the code path directly, verifying cap fires not just that cap constant exists. |
| F-LP4-LOW-001 | LOW | RESOLVED | `pipeline.rs:368-391` — `execute_step` docstring rewritten. Now accurately describes single-request semantics: one HTTP call per invocation, pagination handled by caller loop, no implicit iteration within execute_step. |
| F-LP4-LOW-002 | LOW | RESOLVED | `pipeline.rs:40` — `PipelineResult` marked `#[non_exhaustive]`; `pipeline.rs:53` — `FetchContext` marked `#[non_exhaustive]`; `FetchContext::new()` constructor added at `pipeline.rs:56-67`; all external construction sites updated to use constructor. `#[non_exhaustive]` semantics correctly enforced. |
| F-LP4-LOW-003 | LOW | RESOLVED | `pipeline.rs:230` — `(1.0 / rps).min(3600.0)` clamp applied before `Duration::from_secs_f64`. 1-hour maximum cap prevents overflow from extremely low `rps` values (< 1e-10). |
| F-LP4-LOW-004 | LOW | RESOLVED | `auth_provider.rs:46` — `AuthToken` inner field made private; `auth_provider.rs:48-63` — `AuthToken::new()` and `AuthToken::as_str()` accessor methods added; `pipeline.rs:625-626` — call sites updated to `token.as_str()`. Type-system enforcement replaces convention for token non-leakage. |

## Part B — New Findings (or all findings for pass 1)

No new findings raised in this closure report. This is a fix-burst closure document only.
All findings from pass-4 are dispositioned in Part A above.

### CRITICAL

None.

### HIGH

None.

### MEDIUM

None.

### LOW

None.

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 0 |
| LOW | 0 |

**Overall Assessment:** pass — fix-burst closure (no new findings)
**Convergence:** Findings remain (streak 0/3) — pass-5 required to advance streak
**Readiness:** Fix-burst-3 complete; pass-5 dispatch next against HEAD `d5a12e4a`

### New TDs Filed This Burst

| TD ID | Priority | Description |
|-------|----------|-------------|
| TD-S-PLUGIN-PREREQ-B-005 | P2 | Production `reqwest::Client` construction MUST use `Client::builder().timeout(Duration::from_secs(30))`. Without per-request timeout, slow-loris API server can hang executor indefinitely. Test fixtures updated to builder pattern (convention set); production wiring is PREREQ-D scope. Source: F-LP4-MED-001. |

### Test Coverage Added

- **New Red Gate tests this burst:** 4
  - `test_BC_2_16_002_execute_rejects_fan_out_batch_size_zero` (validation rejection — HIGH-001 paper-fix-proof)
  - `test_BC_2_16_002_execute_accepts_fan_out_batch_size_one` (accept-1 path)
  - `test_BC_2_16_002_execute_accepts_fan_out_batch_size_none` (defaults to 100)
  - `test_BC_2_16_002_execute_aborts_at_max_pages_per_step` (MED-002 regression)
- **Red Gate count:** 29 → 33 canonical-name grep
- **Full crate tests:** 267/267 pass + 1 skipped (was 263; +4 from fix-burst-3)
- **Workspace:** builds clean

### Cascade Trajectory

| Pass | Findings | Status |
|------|---------|--------|
| pass-1 | 20 (4C+6H+5M+2L+3O) | BLOCKED-hard — REMEDIATED at fix-burst-1 (`7511e749`) |
| fix-burst-1 | 12 closed + 2 TDs | CLOSED at `7511e749` |
| pass-2 | 10 (0C+2H+3M+3L+2O) | BLOCKED-hard — REMEDIATED at fix-burst-2 (`a6895d7a`) |
| fix-burst-2 | 8 closed + 2 TDs | CLOSED at `a6895d7a` |
| pass-3 | 4 (0C+0H+0M+2L+2O) | CLEAN — FALSE-CLEAN (pass-4 caught) |
| pass-4 | 7 (0C+1H+2M+4L) | BLOCKED-hard — REMEDIATED at fix-burst-3 (`d5a12e4a`) |
| fix-burst-3 | 7 closed + 1 TD | CLOSED at `d5a12e4a` |

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 4 (fix-burst-3 closure) |
| **New findings** | 0 (closure report — no new adversary findings raised) |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | N/A (fix-burst closure document) |
| **Median severity** | N/A |
| **Trajectory** | 20→10→4→7 (non-monotonic; HIGH-001 security axis missed by passes 1-3) |
| **Verdict** | FINDINGS_REMAIN — pass-5 required to advance streak toward 3/3 CONVERGED |
