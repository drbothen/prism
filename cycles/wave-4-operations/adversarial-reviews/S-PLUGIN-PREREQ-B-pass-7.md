---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-11T23:50:00Z
phase: 3
inputs:
  - "crates/prism-spec-engine/src/pipeline.rs"
  - "crates/prism-spec-engine/src/auth_provider.rs"
  - "crates/prism-spec-engine/tests/pipeline_http_integration.rs"
  - ".factory/specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md"
input-hash: "527a4e1"
traces_to: ".factory/specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md"
pass: 7
previous_review: "S-PLUGIN-PREREQ-B-fix-burst-6.md"
review_level: LOCAL
target_artifact: S-PLUGIN-PREREQ-B
target_sha: 8e9a92d0
base_sha: 90d7c80f
verdict: BLOCKED-soft
streak: 0/3
finding_summary: { critical: 0, high: 0, medium: 3, low: 1, obs: 3 }
prior_passes: 6 passes + 6 fix-bursts; trajectory 20→10→4→7→10→9→8
---

# Adversarial Review: S-PLUGIN-PREREQ-B (Pass 7)

## Finding ID Convention

Finding IDs use the format: `F-LP7-<SEV>-<SEQ>` for this LOCAL pass.

- `F-LP7`: LOCAL Pass 7 prefix for S-PLUGIN-PREREQ-B
- `<SEV>`: Severity abbreviation (`MED`, `LOW`, `OBS`)
- `<SEQ>`: Three-digit sequence within the pass

## Part A — Fix Verification (pass >= 2 only)

All 5 fix-burst-6 actionable closures verified CLEAN. No paper-fixes detected per TD-VSDD-059 discipline.

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| F-LP6-HIGH-001 VP-PLUGIN-002 anchor drift | HIGH | RESOLVED | VP-INDEX.md:168 now cites S-PLUGIN-PREREQ-B anchor; PLUGIN-MIGRATION-001-D reference removed |
| F-LP6-HIGH-002 VP-PLUGIN-005 contradiction | HIGH | RESOLVED | VP-INDEX.md lines 171 and 187 reconciled; single VP-150 entry, no dual definition |
| F-LP6-MED-001 NullAuth/Mock public-API leak | MEDIUM | RESOLVED | lib.rs:89 moved behind `#[cfg(feature = "test-helpers")]`; production binary cannot reach test doubles |
| F-LP6-MED-002 VP-PLUGIN-005 missing from story frontmatter | MEDIUM | RESOLVED | Story v1.6 frontmatter verification_properties now contains VP-PLUGIN-005 |
| F-LP6-MED-003 execute_step eager-token paper-fix regression | MEDIUM | RESOLVED | pipeline.rs:436-483 execute_step calls acquire_token eagerly at entry (Option A symmetric with execute()); unit-level inspection clean |

## Part B — New Findings (or all findings for pass 1)

### CRITICAL

_(none)_

### HIGH

_(none)_

### MEDIUM

#### F-LP7-MED-001: Empty bearer token yields misleading audit signal

- **Severity:** MEDIUM
- **Category:** coverage-gap / spec-fidelity
- **Location:** `crates/prism-spec-engine/src/pipeline.rs` — `execute()` auth resolution path
- **Description:** After fix-burst-5 introduced eager-token, `AuthProvider::acquire_token` can return `Ok(AuthToken::new(String::new()))` from `NullAuthProvider` (used in tests and no-auth sensor specs). When `bearer_token.as_str().is_empty()` the pipeline continues without adding an `Authorization` header — this is correct behavior. However, the audit event emitted is `auth_initial_acquired` (info level) with an empty token value. This creates a misleading audit signal: a consumer reading the audit log sees "auth acquired" but the pipeline actually made unauthenticated requests. Audit-log consumers (SIEM, SOC dashboards) cannot distinguish "authenticated pipeline run" from "unauthenticated pipeline run" via the audit event alone.
- **Evidence:** Both NullAuthProvider (empty token) and a real BearerAuthProvider (non-empty token) flow through the same `auth_initial_acquired` emit path in pipeline.rs after fix-burst-5. The token value is not included in the audit event body, so the signal is semantically identical for both cases.
- **Proposed Fix:** Branch the post-acquire audit emission: if `bearer_token.as_str().is_empty()`, emit a distinct `auth_initial_acquired_empty` (debug level) or annotate the `auth_initial_acquired` event with `auth_type: none`. This preserves the `NullAuthProvider` fast-path while giving audit consumers a distinguishing signal.

#### F-LP7-MED-002: `auth_initial_failed` abort path untested

- **Severity:** MEDIUM
- **Category:** coverage-gap
- **Location:** `crates/prism-spec-engine/src/pipeline.rs` — `execute()` eager-token block; `crates/prism-spec-engine/tests/pipeline_http_integration.rs`
- **Description:** Fix-burst-5 added `auth_initial_acquired` and `auth_initial_failed` audit events. The `auth_initial_acquired` path is covered by `test_BC_2_16_002_execute_acquires_token_eagerly_before_first_request`. The `auth_initial_failed` path — where `AuthProvider::acquire_token` returns `Err(...)` and `execute()` returns early before making any HTTP request — has zero test coverage. BC-2.16.002 postcondition states "if acquire_token fails, execute returns Err immediately with zero HTTP requests". This postcondition is untested.
- **Evidence:** Grep of `pipeline_http_integration.rs` finds no test referencing `auth_initial_failed`, `FailingAuth`, or any auth-error injection. The `auth_initial_failed` event emit site in pipeline.rs has no corresponding Red Gate test.
- **Proposed Fix:** Add a new `FailingAuthProvider` under `cfg(feature = "test-helpers")` that returns `Err(...)` from `acquire_token`. Add a Red Gate test asserting: (1) `execute()` returns `Err`; (2) wiremock `.expect(0)` fires — zero HTTP requests made.

#### F-LP7-MED-003: Partial-record discard policy undocumented and untested

- **Severity:** MEDIUM
- **Category:** spec-fidelity / coverage-gap
- **Location:** `crates/prism-spec-engine/src/pipeline.rs` — step-result accumulation; `crates/prism-spec-engine/tests/pipeline_http_integration.rs`; `.factory/specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md`
- **Description:** When a multi-step pipeline fails at step N (e.g., step 2 returns HTTP 500 after step 1 returned records), `PipelineExecutor::execute` propagates the error and records from step 1 are silently discarded. This is a reasonable design choice (all-or-nothing semantics), but: (1) BC-2.16.002 does not document this postcondition — a reader could infer partial results are returned; (2) no test exercises the mid-pipeline-failure path to verify records from successful earlier steps are NOT leaked into the result.
- **Evidence:** BC-2.16.002 Postconditions section contains no entry for "on mid-pipeline failure, accumulated records are discarded". `pipeline_http_integration.rs` has no test with a 2-step pipeline where step 1 succeeds and step 2 fails.
- **Proposed Fix (dual):** Code: Add a Red Gate test — 2-step pipeline where step 1 returns 200 with records and step 2 returns 500. Assert `execute()` returns `Err` and PipelineResult from step 1 is NOT in the error return. BC: Add postcondition to BC-2.16.002 v1.6: "On mid-pipeline failure, execute returns Err; records accumulated from prior steps are discarded (all-or-nothing semantics)."

### LOW

#### F-LP7-LOW-001: `execute_step` has no test coverage in PREREQ-B scope

- **Severity:** LOW
- **Category:** coverage-gap
- **Location:** `crates/prism-spec-engine/src/pipeline.rs:424` — `execute_step` function
- **Description:** `execute_step` was implemented in fix-burst-6 (Option A — symmetric eager-token with `execute()`). Per story §94–96, `execute_step` is intentionally not wired into any PREREQ-B call site — wiring happens in PREREQ-D. The function has no direct test coverage in PREREQ-B. If the eager-token invariant in `execute_step` regresses, no PREREQ-B test would catch it.
- **Evidence:** Grep of `pipeline_http_integration.rs` finds no test calling `execute_step`. The function has zero callers in PREREQ-B scope.
- **Proposed Fix:** DEFER to PREREQ-D scope. File as TD-S-PLUGIN-PREREQ-B-012 P3. Add doc comment to `execute_step` at pipeline.rs:424 citing the TD and the deferral reason (PREREQ-D wiring will add the test vehicle).

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 3 |
| LOW | 1 |
| OBS | 3 |

**Overall Assessment:** block
**Convergence:** findings remain — fix-burst-7 required before pass-8
**Readiness:** requires revision (3 MED actionable + 1 LOW deferred as TD)

### Observations (Non-Blocking)

**OBS-LP7-A:** Self-dependency dev-dep idiom (`prism-spec-engine` in its own `[dev-dependencies]` with `features = ["test-helpers"]`) is the standard Rust self-dep pattern. No transitive leakage into production — dev-dependencies are excluded from downstream crates. SAFE. No action required.

**OBS-LP7-H:** `mutants.out/` and `mutants.out.old/` directories appear as untracked in git status. Root `.gitignore` does not have entries for these cargo-mutants output directories. Cross-project process gap, not PREREQ-B scope.

**OBS-LP7-I:** `SensorSpec.sensor_id` field in prism-spec-engine is still typed `String` rather than `SensorId` (the newtype from PREREQ-A). Pre-existing; not a PREREQ-B regression. PREREQ-D or a migration story should sweep prism-spec-engine for `sensor_id: String` and migrate to `SensorId`.

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 7 |
| **New findings** | 4 (3 MED + 1 LOW) |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 4 / (4 + 0) = 1.0 (all new dimensions) |
| **Median severity** | MEDIUM |
| **Trajectory** | 20→10→4→7→10→9→8 |
| **Verdict** | FINDINGS_REMAIN — fix-burst-7 required; streak stays 0/3 |
