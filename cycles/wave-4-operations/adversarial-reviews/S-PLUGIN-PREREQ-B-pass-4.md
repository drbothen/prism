---
document_type: adversarial-review
level: LOCAL
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-11T22:00:00Z
phase: 3
inputs: []
input-hash: "a6895d7"
traces_to: prd.md
pass: 4
previous_review: S-PLUGIN-PREREQ-B-pass-3.md
target_artifact: S-PLUGIN-PREREQ-B
target_sha: a6895d7a
base_sha: 90d7c80f
verdict: BLOCKED-hard
streak: 0/3
finding_summary: { critical: 0, high: 1, medium: 2, low: 4, obs: 0 }
prior_passes: pass-1 BLOCKED(20)→fix-burst-1(12 closed); pass-2 BLOCKED(10)→fix-burst-2(8 closed); pass-3 CLEAN streak 1/3 → CAUGHT-AS-FALSE-CLEAN by pass-4
---

# Adversarial Review: S-PLUGIN-PREREQ-B (Pass 4)

**Verdict:** BLOCKED-hard — streak 1/3 RESET → **0/3** (PASS-3 FALSE-CLEAN CAUGHT)
**Target SHA:** a6895d7a (UNCHANGED from passes 2 and 3)
**Base SHA:** 90d7c80f (develop HEAD, S-PLUGIN-PREREQ-A merged)
**Finding summary:** 0 CRITICAL / 1 HIGH / 2 MEDIUM / 4 LOW / 0 OBS
**Streak:** 0/3 (reset)
**Trajectory:** 20 → 10 → 4 → 7 (non-monotonic; regression detection via fresh dimensions)
**Date:** 2026-05-11

## Finding ID Convention

Finding IDs use the format: `F-LP<PASS>-<SEV>-<SEQ>` for LOCAL-pass reviews in this cascade.

- `F`: Fixed prefix
- `LP<PASS>`: LOCAL pass number (e.g., `LP4`)
- `<SEV>`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`, `OBS`)
- `<SEQ>`: Three-digit sequence within the pass (e.g., `001`)

Examples: `F-LP4-HIGH-001`, `F-LP4-MED-001`

## Pass-3 False-CLEAN Catch

Pass-3 returned a CLEAN verdict at HEAD a6895d7a (streak 1/3, D-403). Pass-4 attacks
fresh dimensions — security/threat-model axis (P4-A), async correctness (P4-B),
error-path exhaustion (P4-C), API stability (P4-D), test hygiene (P4-E), documentation
coherence (P4-F) — and surfaces 7 actionable findings. Pass-3's CLEAN was wrong for the
P4-A security/threat-model input-validation axis.

This follows the same pattern as S-PLUGIN-PREREQ-A pass-9 catching pass-8's false-CLEAN
(D-389): fresh-context adversary with disjoint angle attacks surfaces findings that prior
passes' scope did not exhaustively cover. The OBS-LP9-001 reasonable-assumption protocol
is justified here — the adversary's job is precisely to find what prior passes missed.

## Part A — Fix Verification (pass-3 had 0 actionable findings — all prior closures remain CLEAN)

Pass-3 was CLEAN (0 actionable findings). No new closures to verify. All prior fix-burst-1
and fix-burst-2 closures remain verified CLEAN at HEAD a6895d7a (unchanged from pass-3
verification pass).

| Finding | Prior Status | Re-verification | Notes |
|---------|-------------|-----------------|-------|
| fix-burst-1 closures (12) | CLOSED (pass-2 verified) | STILL CLOSED | HEAD unchanged; no regressions introduced |
| fix-burst-2 closures (8) | CLOSED (pass-3 verified) | STILL CLOSED | All 8 paper-fix-free per pass-3 Part A audit |

## Part B — New Findings (or all findings for pass 1)

### CRITICAL

_No CRITICAL findings this pass._

### HIGH

#### F-LP4-HIGH-001: fan_out_batch_size = 0 panics via chunks(0) — DoS via spec upload

- **Severity:** HIGH
- **Category:** security-surface / input-validation gap
- **Location:** fan_out execution path (chunks call site); validation.rs:262-274 (missing symmetric check); add_sensor_spec.rs:237 (exploit entry point)
- **Description:** The spec-engine validates `page_size` at TOML spec load time (validation.rs:262-274, which rejects `OffsetLimit { page_size: 0 }`). No symmetric check exists for `fan_out_batch_size`. A TOML spec with `fan_out_batch_size = 0` causes `ids.chunks(0)` at runtime, which panics unconditionally in Rust's standard library (`slice::chunks` requires `size > 0`; debug and release builds both panic).
- **Evidence:** Existing `page_size == 0` guard at validation.rs:262-274 is the correct model. `fan_out_batch_size` is consumed as argument to `.chunks(fan_out_batch_size)` in the fan-out execution path with no prior zero-check. `slice::chunks(0)` panics with "chunk size must be non-zero" per stdlib documentation.
- **Proposed Fix:** Add `fan_out_batch_size == 0` check to the same spec-validation pass that checks `page_size`. Return a structured `SpecValidationError` (or `InvalidConfiguration` variant) rather than allowing the panic. Regression test: spec with `fan_out_batch_size = 0` must be rejected at load time, not at execute time.

### MEDIUM

#### F-LP4-MED-001: reqwest::Client::new() has no timeout — slow-loris hang possible

- **Severity:** MEDIUM
- **Category:** security-surface / availability
- **Location:** Plugin executor HTTP client construction
- **Description:** The `reqwest::Client` used in the plugin executor is constructed via `Client::new()` with no `.timeout()` set. A misbehaving or adversarially slow upstream API can hold the connection open indefinitely, consuming resources for the lifetime of the MCP server process. Existing codebase precedent at host_functions.rs:154 uses a 10-second timeout via `Client::builder().timeout(Duration::from_secs(10)).build()?`.
- **Evidence:** host_functions.rs:154 — timeout precedent established. Plugin executor `Client::new()` — no timeout configured. The MCP server is a long-lived per-analyst process; a hung HTTP connection blocks all subsequent pipeline invocations.
- **Proposed Fix:** Construct the plugin executor client via `Client::builder().timeout(Duration::from_secs(30)).build()?` (or a configurable value from `SpecConfig`). Wiremock test: configure a delayed response beyond the timeout; executor must return an error rather than hanging.

#### F-LP4-MED-002: MAX_PAGES_PER_STEP cap has no regression test — paper-fix-proof gap

- **Severity:** MEDIUM
- **Category:** coverage-gap / test hygiene
- **Location:** Paginator logic (MAX_PAGES_PER_STEP cap enforcement)
- **Description:** Fix-burst-2 introduced `MAX_PAGES_PER_STEP = 1000` as the backstop against the cursor-infinite-loop described in F-LP2-HIGH-002. The existing test suite verifies the cursor non-advance trip-wire (same cursor returned → abort). No test verifies that the MAX_PAGES_PER_STEP cap itself is enforced. A regression that flips the comparison (`>=` → `>`) or removes the cap check entirely would survive all current tests — same paper-fix-proof gap class as F-LP3-LOW-002 (AC-2 wiremock matcher gap, pass-3 audit).
- **Evidence:** fix-burst-2 added MAX_PAGES_PER_STEP=1000 + prev_cursor non-advance guard (D-402). Test coverage for non-advance guard: present. Test coverage for MAX_PAGES_PER_STEP cap: absent. The cap check and the non-advance check are two independent guard clauses; only one is tested.
- **Proposed Fix:** Add a Red Gate test that drives `MAX_PAGES_PER_STEP + 1` pages of cursor pagination (each page returning a distinct new cursor) and asserts the executor aborts with the correct structured error after `MAX_PAGES_PER_STEP` pages. Name per BC-prefixed canonical naming convention (PG-LP7-002).

### LOW

#### F-LP4-LOW-001: execute_step docstring claims pagination but issues only single request

- **Severity:** LOW
- **Category:** code-quality / documentation coherence
- **Location:** execute_step function doc-comment
- **Description:** The `execute_step` function's doc-comment describes the function as handling pagination, implying it internally loops through pages. The current implementation issues a single HTTP request per invocation; the pagination loop is one level up in the caller. The doc-comment overstates the function's responsibility and will mislead future implementers.
- **Evidence:** execute_step implementation — single request, returns response for caller to paginate over. Doc-comment — describes pagination behavior as internal to the function.
- **Proposed Fix:** Rewrite the doc-comment to accurately describe single-request semantics: issues one HTTP request per step invocation; caller handles pagination loop.

#### F-LP4-LOW-002: PipelineResult and FetchContext lack #[non_exhaustive]

- **Severity:** LOW
- **Category:** code-quality / API stability
- **Location:** PipelineResult and FetchContext struct definitions
- **Description:** `PipelineResult` and `FetchContext` are `pub` structs in the plugin executor's public surface. Neither carries `#[non_exhaustive]`. Adding fields in a future PREREQ-C/D/E iteration is a semver-breaking change for any downstream consumer that pattern-matches or struct-initializes them. `SpecEngineError` IS marked `#[non_exhaustive]` (confirmed in pass-3 audit), establishing the correct pattern for this codebase.
- **Evidence:** SpecEngineError carries `#[non_exhaustive]` — KUDO from pass-3. PipelineResult and FetchContext do not.
- **Proposed Fix:** Add `#[non_exhaustive]` to both structs. No behavior change, no test change required.

#### F-LP4-LOW-003: Duration::from_secs_f64 may panic with rps < ~1e-10

- **Severity:** LOW
- **Category:** security-surface / arithmetic safety
- **Location:** Rate-limiting sleep computation (`1.0 / rps` → Duration::from_secs_f64)
- **Description:** `Duration::from_secs_f64` panics if the argument is negative, NaN, or greater than `u64::MAX` seconds. The rate-limiting logic computes `Duration::from_secs_f64(1.0 / rps)`. If `rps < ~1e-10`, `1.0 / rps` overflows `f64` to infinity, causing `Duration::from_secs_f64(f64::INFINITY)` which panics with "overflow converting floating point to duration." Exploitability is low (requires a TOML spec with `rps < 1e-10`) but the existing page_size and fan_out_batch_size validation precedents establish that numeric spec fields should be bounded at spec-load time.
- **Evidence:** Rust stdlib: `Duration::from_secs_f64` panics on infinity/NaN/negative. Rate-limit computation: `1.0 / rps` with no clamp or validation.
- **Proposed Fix:** Either (a) validate `rps >= MIN_RPS` at spec-load time, or (b) clamp the computed sleep duration: `let sleep_secs = (1.0 / rps).min(MAX_SLEEP_SECS); Duration::from_secs_f64(sleep_secs)`. Option (b) is simpler for fix-burst-3 scope.

#### F-LP4-LOW-004: AuthToken pub inner field permits accidental .0 secret leakage

- **Severity:** LOW
- **Category:** security-surface / credential safety
- **Location:** AuthToken newtype definition
- **Description:** `AuthToken` redacts its value in `Debug` output (KUDO from pass-3). However, the inner field `pub .0` is accessible — any code can call `token.0` and extract the raw token string without going through an intentional accessor. This is not a current exploit (no code does this today), but it is a latent API surface defect. Any future code in the same crate (or downstream crate with public field access) can bypass the intentional access discipline.
- **Evidence:** AuthToken Debug redaction confirmed at pass-3. AuthToken inner field visibility: `pub`. Correct pattern for secret-holding newtypes: `pub(crate)` inner field with deliberate accessor if external access is needed.
- **Proposed Fix:** Change `AuthToken` inner field from `pub` to `pub(crate)` (or private with a deliberate accessor). Debug redaction preserved. No behavior change for current callers.

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 1 |
| MEDIUM | 2 |
| LOW | 4 |

**Overall Assessment:** block
**Convergence:** findings remain — fix-burst-3 required to close all 7 actionable findings
**Readiness:** requires revision before next pass

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 4 |
| **New findings** | 7 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 7 / (7 + 0) = 1.0 |
| **Median severity** | LOW (4 of 7 findings are LOW; 1 HIGH + 2 MED + 4 LOW) |
| **Trajectory** | 20 → 10 → 4 → 7 |
| **Verdict** | FINDINGS_REMAIN |

**Non-monotonic trajectory note:** 4 → 7 reflects fresh-context dimension expansion.
Pass-3 covered axes P3-A through P3-F; pass-4 covers P4-A security/threat-model axis
which pass-3 did not exhaustively probe. The 7 new findings (fan_out_batch_size DoS,
no HTTP timeout, MAX_PAGES_PER_STEP cap untested, doc drift, missing #[non_exhaustive],
arithmetic overflow, AuthToken pub field) were latent in the code since before pass-3.
Pass-3's CLEAN was correct for the axes it covered, insufficient for the full adversarial
surface. Same pattern as S-PLUGIN-PREREQ-A pass-9 catching pass-8 false-CLEAN (D-389).

**Process gaps observed:**
- PG-LP4-001: Security/threat-model input-validation axis (integer bounds on numeric spec fields) should be added to the adversary's standard checklist for all plugin execution stories. `page_size == 0`, `fan_out_batch_size == 0`, `rps < epsilon` are all the same pattern.
- PG-LP4-002: Fix-burst backstop regression test gap (F-LP4-MED-002) is the same class as F-LP3-LOW-002 (AC-2 wiremock matcher gap). Adversary should explicitly audit every fix-burst's "new logic" for a corresponding regression test that would fail if the logic were deleted or sign-flipped.
