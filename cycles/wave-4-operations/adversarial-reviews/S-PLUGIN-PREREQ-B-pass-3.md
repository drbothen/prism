---
document_type: adversarial-review
level: LOCAL
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-11T21:00:00Z
phase: 3
inputs: []
input-hash: "a6895d7"
traces_to: prd.md
pass: 3
previous_review: S-PLUGIN-PREREQ-B-pass-2.md
target_artifact: S-PLUGIN-PREREQ-B
target_sha: a6895d7a
base_sha: 90d7c80f
verdict: CLEAN
streak: 1/3
finding_summary: { critical: 0, high: 0, medium: 0, low: 2, obs: 2 }
prior_passes: pass-1 BLOCKED-hard(20) + fix-burst-1(12 closed); pass-2 BLOCKED-hard(10) + fix-burst-2(8 closed)
---

# Adversarial Review: S-PLUGIN-PREREQ-B (Pass 3)

**Verdict:** CLEAN — streak 0/3 → **1/3** (FIRST CLEAN PASS)
**Target SHA:** a6895d7a (UNCHANGED from fix-burst-2 close)
**Base SHA:** 90d7c80f (develop HEAD, S-PLUGIN-PREREQ-A merged)
**Finding summary:** 0 CRITICAL / 0 HIGH / 0 MEDIUM / 2 LOW / 2 OBS
**Streak:** 1/3
**Trajectory:** 20 → 10 → 4 (all LOW+OBS, ZERO actionable CRIT/HIGH/MED)
**Date:** 2026-05-11

## Finding ID Convention

Finding IDs use the format: `F-LP<PASS>-<SEV>-<SEQ>` for LOCAL-pass reviews in this cascade.

- `F`: Fixed prefix
- `LP<PASS>`: LOCAL pass number (e.g., `LP3`)
- `<SEV>`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`, `OBS`)
- `<SEQ>`: Three-digit sequence within the pass (e.g., `001`)

Examples: `F-LP3-LOW-001`, `F-LP3-OBS-001`

## Part A — Fix Verification (fix-burst-2 closures — 8/8 actionable + 2 ACKNOWLEDGED OBS)

All 8 actionable findings from pass-2 verified paper-fix-free at HEAD a6895d7a. Evidence via file:line grep.

| Finding | Fix-burst-2 Claim | Verification Result | Notes |
|---------|-------------------|---------------------|-------|
| F-LP2-HIGH-001 (fan-out paper-fix regression) | Closed — paper-fix-proof test with dual-assertion `unique_count == 3` AND `q.len() < 700` | CLOSED | `fan_out_test.rs:89` unique_count assert + `:97` size guard — full-array regression would produce 900+ char query strings; both assertions would fail |
| F-LP2-HIGH-002 (cursor infinite-loop) | Closed — MAX_PAGES_PER_STEP=1000 cap + prev_cursor non-advance guard | CLOSED | `paginator.rs:14` const; `:47` enforce; `:52-58` non-advance break guard — defense-in-depth |
| F-LP2-MED-001 (red_gate_tests count divergence) | Closed — story v1.2 `red_gate_tests: 29` per grep re-count | CLOSED | `grep -rn "test_BC_2_16_002" crates/prism-spec-engine/src/` confirms 29 matches |
| F-LP2-MED-002 (Content-Type misclassifies JSON arrays) | Closed — array case routes to `application/json` | CLOSED | `http_client.rs:134` array → JSON branch; `http_client_test.rs:118` array Content-Type Red Gate |
| F-LP2-MED-003 (numeric cursor silent-failure) | Closed — numeric cursor coerced via `serde_json::Number::to_string()` with `warn!` | CLOSED | `paginator.rs:78` coerce; `:82` warn; wiremock test `query_param("cursor", "42")` verifies string form reaches HTTP layer |
| F-LP2-LOW-001 (AuthAcquisitionFailed unit test absent) | Closed — unit test added for failure path returning E-AUTH-001 | CLOSED | `auth_test.rs:201` asserts `AuthAcquisitionFailed` error variant |
| F-LP2-LOW-002 (RFC 6901 escape ordering) | Closed — `~`→`~0` before `/`→`~1` correct per RFC 6901 §3; inline citation | CLOSED | `json_pointer.rs:34-35` ordering correct; inline comment cites TD-S-PLUGIN-PREREQ-B-003 + RFC 6901 §3 |
| F-LP2-LOW-003 (percent_encoding import inline) | Closed — hoisted to crate-level `use` declaration | CLOSED | `paginator.rs:7` crate-level `use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};` |
| OBS-LP2-001 (MAX_REQUESTS_PER_PIPELINE partial coverage) | Acknowledged — TD-S-PLUGIN-PREREQ-B-004 P3 filed | ACKNOWLEDGED | Non-blocking; TD filed for PREREQ-D scope |
| OBS-LP2-002 (red_gate_tests count discipline) | Acknowledged — links to PG-LP7-002 | ACKNOWLEDGED | Process-gap already codified in S-PLUGIN-PREREQ-A cascade |

**8/8 CLOSED (paper-fix-free) + 2 ACKNOWLEDGED OBS**

### Sibling-Site Sweep (TD-VSDD-060 discipline)

Sweep of all value/signature changes introduced in fix-burst-2.

**find_fan_out_array signature change (tuple key refactor):**
- `grep -rn "find_fan_out_array" crates/prism-spec-engine/src/` — 1 callsite at `executor/pipeline.rs:183`
- Callsite destructures tuple: `let (source_key, arr) = find_fan_out_array(step, &vars)?;` — CORRECT
- No orphan callers using old single-value return. CLEAN.

**extract_at_path signature change (Result<_, String>):**
- `grep -rn "extract_at_path" crates/prism-spec-engine/src/` — 5 callsites found:
  1. `executor/pipeline.rs:201` — `?` propagation — updated
  2. `executor/pipeline.rs:247` — `?` propagation — updated
  3. `executor/fan_out.rs:89` — `?` propagation — updated
  4. `executor/template.rs:134` — `?` propagation — updated
  5. `tests/json_pointer_test.rs:45` — test direct call with `unwrap()` — updated
- All 5 callsites handle `Result`. No orphan `Option`-style callers. CLEAN.

**MAX_PAGES_PER_STEP constant references:**
- `grep -rn "MAX_PAGES_PER_STEP" crates/prism-spec-engine/` — 4 hits: `paginator.rs:14` (const def), `:47` (enforce), `:103` (test), `:23` (doc comment)
- No stale references in story, ARCH-INDEX, or STORY-INDEX. CLEAN.

**BC-2.16.002 v1.4 postconditions vs implementation:**
- POST-1 (fan-out distinct URLs with correct key interpolation): `execute_fan_out_sends_distinct_batch_urls` verifies — MATCHES
- POST-2 (cursor advance terminates): `MAX_PAGES_PER_STEP` + non-advance guard — MATCHES
- POST-3 (Content-Type derivation per payload shape): array → `application/json` path — MATCHES
- POST-4 (percent-encoded cursor in URL): `utf8_percent_encode` applied before query-param insertion — MATCHES
- 4/4 postconditions MATCH implementation. CLEAN.

### New Audit Dimensions (Pass-3 fresh-context angles)

**A — Regression detection:** All 20 pass-1 + 10 pass-2 findings stable at a6895d7a. No regressions introduced by fix-burst-2. CLEAN.

**B — Concurrency safety:** `ExecutionContext` and `StepVarStore` not shared across tasks; all `Arc<_>` references read-only across async boundaries. No Mutex/RwLock misuse. `tokio::spawn` usages hold owned values only. CLEAN.

**C — POL-12 (no `unwrap()` in production):** `grep -rn "\.unwrap()" crates/prism-spec-engine/src/executor/` — 0 hits in production executor code. CLEAN.

**D — POL-16 (no `println!`/`eprintln!` in production):** `grep -rn "println!\|eprintln!" crates/prism-spec-engine/src/executor/` — 0 hits. All output via `tracing::` macros. CLEAN.

**E — POL-1 (no `todo!()`/`unimplemented!()`):** `grep -rn "todo!()\|unimplemented!()" crates/prism-spec-engine/src/executor/` — 0 hits. CLEAN.

**F — AC coverage (9/9):**
- AC-1 (auth acquire): `auth.rs` + `auth_test.rs` — SATISFIED
- AC-2 (body interpolation): `template.rs` + `template_test.rs` — SATISFIED
- AC-3 (Content-Type derivation): `http_client.rs` + `http_client_test.rs` — SATISFIED
- AC-4 (cursor pagination): `paginator.rs` + `paginator_test.rs` — SATISFIED
- AC-5 (audit log): `audit.rs` + `audit_test.rs` — SATISFIED
- AC-6 (fan-out distinct URLs): `fan_out.rs` + `fan_out_test.rs` — SATISFIED (paper-fix-proof test)
- AC-7 (rate-limit positive case): `rate_limiter.rs` + `rate_limiter_test.rs` — SATISFIED
- AC-8 (truncate truthy case): `truncate.rs` + `truncate_test.rs` — SATISFIED
- AC-9 (step-var store): `step_vars.rs` + `step_vars_test.rs` — SATISFIED
- 9/9 ACs satisfied. CLEAN.

**G — Story coherence:** Story v1.2 accurately reflects `red_gate_tests: 29`, all 9 ACs, BC-2.16.002 v1.4 reference, 4 TDs filed. CLEAN.

**H — BC satisfaction:** BC-2.16.002 v1.4 all 4 postconditions verified in sibling-site sweep. CLEAN.

**I — Code quality:** Clippy 0 warnings; no dead code; no `#[allow(dead_code)]` in executor module. CLEAN.

**J — Workspace build:** `just check` at a6895d7a: 263/263 prism-spec-engine tests pass; workspace clean. CLEAN.

**K — New PG candidates:** No new process-gaps beyond pass-2 (OBS-LP2-002 → PG-LP7-002 linkage). CLEAN.

## Part B — New Findings (or all findings for pass 1)

### LOW

#### F-LP3-LOW-001: store_step_vars asymmetric insert vs or_insert_with

- **Severity:** LOW
- **Category:** missing-edge-cases
- **Location:** `crates/prism-spec-engine/src/executor/step_vars.rs` — `store_step_vars()` function body
- **Description:** `store_step_vars` inserts step output into the var store using direct `HashMap::insert`, while the fan-out batch store uses `entry(...).or_insert_with(Vec::new)`. This asymmetry means if two steps share the same key due to templating collision, later-step output silently overwrites earlier-step output for the non-fan-out path, while the fan-out path accumulates. The story does not define overwrite vs accumulate semantics for key collision — this is a pre-existing pagination semantics gap, NOT a PREREQ-B regression.
- **Evidence:** `step_vars.rs` — `store_step_vars` uses `HashMap::insert` (overwrite semantics); fan-out batch aggregation uses `entry().or_insert_with` (accumulate semantics). No story AC defines expected collision behavior.
- **Proposed Fix:** Non-blocking. Deferrable to PREREQ-C scope or maintenance pass. If key-collision semantics need to be defined, update story and BC-2.16.002 with explicit postcondition before implementation.

#### F-LP3-LOW-002: AC-2 wiremock matcher does not verify interpolated value in URL

- **Severity:** LOW
- **Category:** coverage-gap
- **Location:** `crates/prism-spec-engine/src/tests/fan_out_test.rs` — `test_BC_2_16_002_execute_fan_out_sends_distinct_batch_urls`
- **Description:** The paper-fix-proof test (F-LP2-HIGH-001 closure) asserts `unique_count == 3` and `q.len() < 700`. These assertions prevent the full-array regression. However, the test does not verify that the specific interpolated value (e.g., `ids=id-001`) appears in the URL — only uniqueness + size bound. A future implementation producing 3 distinct but nonsensical query strings would pass both assertions. The dual-assertion is a strong regression guard for the specific F-LP2-HIGH-001 failure mode; this is a test completeness gap in the paper-fix-proof methodology, not a current AC satisfaction defect.
- **Evidence:** `fan_out_test.rs:89-97` — assertions are `unique_count == 3` and `q.len() < 700`; no `assert!(q.contains("id-001"))` or equivalent value verification.
- **Proposed Fix:** Non-blocking. Add value-verification assertion to close the paper-fix-proof gap. OBS-LP3-001 proposes codifying this as a standing methodology requirement.

### OBS

#### OBS-LP3-001 [OBS / process-gap]: Paper-fix-proof-test methodology codification

The paper-fix-proof pattern (dual-assertion: regression-impossible condition + secondary structural guard) pioneered in this cascade is highly effective but not yet codified as a standing methodology. Future adversary passes may accept weaker single-assertion closures. Recommend codifying paper-fix-proof-test as a standing technique in the process-gap registry, including the value-verification requirement exposed by F-LP3-LOW-002.

#### OBS-LP3-002 [OBS]: red_gate_tests scope ambiguity — tests/ vs src/ unit tests

The canonical `red_gate_tests: 29` count (grep `test_BC_2_16_002` in `src/`) and the fix-burst-2 report's "+1 error.rs unit = 30 total" create a dual-counting convention that is clear in context but may confuse future auditors. Story v1.2 frontmatter `red_gate_tests: 29` reflects only canonical BC-prefixed naming pattern; this is internally consistent. Non-blocking observational.

### KUDOs (6)

1. **Paper-fix-proof exemplar** — `test_BC_2_16_002_execute_fan_out_sends_distinct_batch_urls` with dual-assertion (unique_count == 3 AND q.len() < 700) establishes the PREREQ cascade gold standard for regression-impossible closure evidence.
2. **RFC 6901 escape ordering + inline TD citation** — `~`→`~0` before `/`→`~1` correct per §3; inline comment cites TD-S-PLUGIN-PREREQ-B-003 and RFC 6901 §3. Highest traceability of any inline comment in this cascade.
3. **AuthProvider object-safety with manually-boxed Future** — Manual `Pin<Box<dyn Future>>` return for `async fn` in object-safe trait (no `async_trait` due to no-alloc constraint). Non-trivial Rust async constraint handled correctly without guidance.
4. **Page-cap + non-advance defense-in-depth** — Two independent termination conditions (MAX_PAGES_PER_STEP liveness cap + prev_cursor non-advance guard) rather than a single guard. Matches production-grade API pagination client patterns.
5. **AuthToken Debug redaction** — `AuthToken` implements `fmt::Debug` with redacted output (`AuthToken([REDACTED])`) proactively without story AC requirement.
6. **Numeric-cursor coercion + warn-on-invalid-type** — `serde_json::Number::to_string()` coercion with `warn!` log on unexpected type. Correct production-grade handling — operators see diagnostic without data loss.

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 0 |
| LOW | 2 |
| OBS | 2 |

**Overall Assessment:** pass
**Convergence:** Trajectory 20 → 10 → 4. ZERO actionable findings. Streak 1/3 — FIRST CLEAN PASS in PREREQ-B LOCAL cascade.
**Readiness:** Advancing toward convergence — requires 2 additional CLEAN passes (pass-4 → streak 2/3; pass-5 → streak 3/3 CONVERGED).

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 3 |
| **New findings** | 2 (F-LP3-LOW-001, F-LP3-LOW-002) |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 2/2 = 1.0 (but all LOW/OBS — zero actionable) |
| **Median severity** | LOW (1.5 on 1.0-5.0 scale) |
| **Trajectory** | 20 → 10 → 4 |
| **Verdict** | CONVERGENCE_REACHED — zero CRIT/HIGH/MED; 2 LOW non-blocking; streak 1/3 |
