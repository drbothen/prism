---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: state-manager
timestamp: "2026-05-11T23:00:00Z"
phase: 3
inputs:
  - "crates/prism-spec-engine/src/pipeline.rs"
  - "crates/prism-spec-engine/src/validation.rs"
  - "crates/prism-spec-engine/Cargo.toml"
  - "crates/prism-spec-engine/tests/pipeline_http_integration.rs"
  - ".factory/cycles/wave-4-operations/adversarial-reviews/S-PLUGIN-PREREQ-B-pass-5.md"
input-hash: "e19372f"
traces_to: ".factory/stories/S-PLUGIN-PREREQ-B-real-pipeline-executor.md"
pass: 5
previous_review: "S-PLUGIN-PREREQ-B-pass-5.md"
review_level: LOCAL
target_artifact: S-PLUGIN-PREREQ-B
fix_burst_for_pass: 5
target_sha: e19372f4
base_sha: d5a12e4a
verdict: PARTIALLY_CLOSED
finding_summary_closed:
  critical: 0
  high: 0
  medium: 2
  low: 1
  deferred_as_td: 4
  surfaced_to_orchestrator: 1
  obs_acknowledged: 2
prior_passes: "pass-5 BLOCKED-soft 10 findings (2M+5L+3O)"
decision_ref: D-407
---

# Adversarial Review: S-PLUGIN-PREREQ-B Fix-Burst-4 Closure Report (Pass 5 Remediation)

**Verdict: PARTIALLY_CLOSED** — 3 actionable findings from LOCAL pass-5 closed at implementer
commit `e19372f4`. 4 findings deferred as TDs. 1 design-level finding surfaced to orchestrator
(F-LP5-LOW-003). 2 OBS acknowledged. Streak stays 0/3 (fix-bursts do not advance streak; pass-6
advances streak AFTER human decision on F-LP5-LOW-003). STATE+HANDOFF v7.140→v7.141.
Story v1.3→v1.4 (red_gate_tests 33→37). 5 new TDs filed: TD-S-PLUGIN-PREREQ-B-006 through -010.

## Finding ID Convention

Finding IDs for this closure report use the pass-5 source format: `F-LP5-<SEV>-<SEQ>`
(LOCAL pass-5 findings from S-PLUGIN-PREREQ-B-pass-5.md). This file records closure
dispositions only — no new adversary findings are raised here.

## Part A — Fix Verification (pass >= 2 only)

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| F-LP5-MED-001 | MEDIUM | RESOLVED | `Cargo.toml:34` — reqwest `features` expanded to `["json","rustls-tls","gzip","deflate","brotli"]`. CrowdStrike/Cyberint gzipped responses now decode transparently. New Red Gate test `test_BC_2_16_002_execute_decodes_gzipped_response` uses wiremock+flate2 fixture to verify gzip decode path. |
| F-LP5-MED-002 (a) | MEDIUM | RESOLVED | `pipeline.rs` — explicit match on `acquire_token` result: emits `info!(target:"prism_audit", event="auth_refresh_succeeded")` on `Ok`; emits `error!(target:"prism_audit", event="auth_refresh_failed")` on `Err`. Audit symmetry restored for successful token refresh path. |
| F-LP5-MED-002 (b) | MEDIUM | RESOLVED | `pipeline.rs` — before returning `AuthRefreshFailed`, emits `error!(target:"prism_audit", event="auth_refresh_double_401")` with all required fields (sensor, step, status_code). Event fires before abort. |
| F-LP5-MED-002 (c) | MEDIUM | RESOLVED | `pipeline.rs` — before truncate+break at 10K record cap, emits `warn!(target:"prism_audit", event="pipeline_truncated")` with `records_kept`, `records_dropped` fields. New Red Gate test `test_BC_2_16_002_emits_pipeline_truncated_event_on_10k_cap` captures tracing events to verify emission. |
| F-LP5-LOW-001 | LOW | RESOLVED — double defense | `pipeline.rs:extract_at_path` — runtime guard rejects `"$."` (path has no segment after dot) before pointer parse. `validation.rs` — validator-layer `validate_json_path` rejects `"$."` at spec-load time. Two new tests: `test_extract_at_path_rejects_dollar_dot_malformed` (negative path) + `test_extract_at_path_rejects_dollar_dot_validator` (validator rejection). |
| F-LP5-LOW-002 | LOW | DEFERRED → TD-S-PLUGIN-PREREQ-B-006 P2 | Pure functions (fan_out_batches, extract_at_path, Interpolator::interpolate, Interpolator::extract_references) lack proptest coverage. PREREQ-C scope. Inline comment at `pipeline.rs:fan_out_batches`. No code change. |
| F-LP5-LOW-003 | LOW | SURFACED-TO-ORCHESTRATOR → TD-S-PLUGIN-PREREQ-B-010 P2 | Lazy-token-on-401 design — ORCHESTRATOR-DECISION-PENDING. See §Orchestrator Decision below. No code change this burst. |
| F-LP5-LOW-004 | LOW | DEFERRED → TD-S-PLUGIN-PREREQ-B-007 P3 | `SpecEngineError::HttpRequestFailed.status_code: u16 = 0` overloaded across 11 distinct error origins. Inline comment near path-interpolation error origin. No code change. |
| F-LP5-LOW-005 | LOW | DEFERRED → TD-S-PLUGIN-PREREQ-B-008 P3 | Interpolator grammar has no escape mechanism for literal `${...}` in templates. PREREQ-C scope. Inline comment near body_template interpolation. No code change. |
| F-LP5-OBS-001 | OBS | DEFERRED → TD-S-PLUGIN-PREREQ-B-009 P3 | `fan_out_batches` scalar match arm unreachable from production callers (`find_fan_out_array` filters on `.is_array()`). Inline comment in scalar match arm. No code change. |
| F-LP5-OBS-002 | OBS | ACKNOWLEDGED | `find_fan_out_array` first-only fan-out. PREREQ-C scope: validator warning for multi-array case. No code change. |
| F-LP5-OBS-003 | OBS | ACKNOWLEDGED | Hot-reload race. PREREQ-D scope: arc-swap wiring concern. No code change. |

## Part B — New Findings (or all findings for pass 1)

No new findings raised in this closure report. This is a fix-burst closure document only.
All findings from pass-5 are dispositioned in Part A above.

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

**Overall Assessment:** pass — fix-burst closure (no new findings raised)
**Convergence:** Findings remain (streak 0/3) — pass-6 required to advance streak (HOLD until F-LP5-LOW-003 orchestrator decision)
**Readiness:** Fix-burst-4 complete at `e19372f4`; pass-6 dispatch blocked pending human decision on lazy-token design

### Orchestrator Decision Required — F-LP5-LOW-003 / TD-S-PLUGIN-PREREQ-B-010

**Issue:** Lazy-token initialization at `let mut bearer_token = AuthToken::new(String::new())` means
every production execution against bearer-required APIs (CrowdStrike, Cyberint, Claroty, Armis)
incurs a guaranteed 401 round-trip before receiving any data.

**Implications of current lazy design:**
- `auth_refresh_triggered` event fires on every legitimate execution — audit signal pollution
- `request_count` inflated by +1 (the phantom 401)
- Doubles API quota usage per execution

**Option A (Recommended): Switch to eager-token acquisition**
- Acquire token before first request when `spec.auth_type != NullAuth`
- Eliminates 401 round-trip from legitimate executions
- `auth_refresh_triggered` fires only on actual re-auth (accurate audit signal)
- Accurate `request_count` (no phantom 401 inflating count)
- Halves API quota usage per execution
- **Requires:** BC-2.16.002 v1.4 → v1.5 amendment (precondition lifecycle change) + story v1.4 → v1.5

**Option B: Keep lazy-token (current implementation)**
- Every execution emits `auth_refresh_triggered` as first event
- `request_count` inflated by +1
- Doubles API quota usage per execution
- No spec changes required

**Hold pass-6 dispatch until decision received** to avoid potential BC drift re-trigger if Option A is chosen.

### New TDs Filed This Burst

| TD ID | Priority | Description | Scope |
|-------|----------|-------------|-------|
| TD-S-PLUGIN-PREREQ-B-006 | P2 | Pure function proptest coverage gap: fan_out_batches, extract_at_path, Interpolator::interpolate, Interpolator::extract_references | PREREQ-C |
| TD-S-PLUGIN-PREREQ-B-007 | P3 | `status_code: u16 = 0` overloaded across 11 distinct error origins; operators cannot distinguish error category without parsing `detail` string | P3 post-keystone |
| TD-S-PLUGIN-PREREQ-B-008 | P3 | Interpolator grammar has no escape mechanism for literal `${...}` in templates | PREREQ-C |
| TD-S-PLUGIN-PREREQ-B-009 | P3 | `fan_out_batches` scalar match arm unreachable from production callers | P3 post-keystone |
| TD-S-PLUGIN-PREREQ-B-010 | P2 | Lazy-token-on-401 design — every bearer-auth execution incurs guaranteed 401 round-trip | ORCHESTRATOR-DECISION-PENDING |

### Test Coverage Added

- **New Red Gate tests this burst:** 4
  - `test_BC_2_16_002_execute_decodes_gzipped_response` (MED-001 gzip decode — wiremock+flate2)
  - `test_BC_2_16_002_emits_pipeline_truncated_event_on_10k_cap` (MED-002(c) truncation event)
  - `test_extract_at_path_rejects_dollar_dot_malformed` (LOW-001 runtime guard negative)
  - `test_extract_at_path_rejects_dollar_dot_validator` (LOW-001 validator rejection)
- **Red Gate count:** 33 → 37 canonical-name grep
- **Full crate tests:** 271/271 pass + 1 skipped (was 267; +4 from fix-burst-4)
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
| pass-5 | 10 (0C+0H+2M+5L+3O) | BLOCKED-soft — PARTIALLY_CLOSED at fix-burst-4 (`e19372f4`) |
| fix-burst-4 | 3 closed + 4 TDs + 1 surfaced + 2 ack | PARTIALLY_CLOSED at `e19372f4` |

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 5 (fix-burst-4 closure) |
| **New findings** | 0 (closure report — no new adversary findings raised) |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | N/A (fix-burst closure document) |
| **Median severity** | N/A |
| **Trajectory** | 20→10→4→7→10 (non-monotonic; fresh-context dimensions compound) |
| **Verdict** | FINDINGS_REMAIN — pass-6 required to advance streak toward 3/3 CONVERGED (HOLD pending orchestrator F-LP5-LOW-003 decision) |
