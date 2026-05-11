---
document_type: adversarial-review
level: LOCAL
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-11T23:00:00Z
phase: 3
inputs: []
input-hash: "d5a12e4"
traces_to: prd.md
pass: 5
previous_review: S-PLUGIN-PREREQ-B-pass-4.md
target_artifact: S-PLUGIN-PREREQ-B
target_sha: d5a12e4a
base_sha: 90d7c80f
verdict: BLOCKED-soft
streak: 0/3
finding_summary: { critical: 0, high: 0, medium: 2, low: 5, obs: 3 }
prior_passes: pass-1 BLOCKED(20)→fix-burst-1(12); pass-2 BLOCKED(10)→fix-burst-2(8); pass-3 CLEAN→FALSE-CLEAN caught by pass-4; pass-4 BLOCKED(7)→fix-burst-3(7 closed); pass-5 surfaces 10 from NEW P5-A..P5-K dimensions
---

# Adversarial Review: S-PLUGIN-PREREQ-B (Pass 5)

**Verdict:** BLOCKED-soft — streak stays **0/3** (2 MED block advancement)
**Target SHA:** d5a12e4a
**Base SHA:** 90d7c80f (develop HEAD, S-PLUGIN-PREREQ-A merged)
**Finding summary:** 0 CRITICAL / 0 HIGH / 2 MEDIUM / 5 LOW / 3 OBS
**Streak:** 0/3 (no advancement; 2 MED block)
**Trajectory:** 20→10→4→7→10 (non-monotonic; fresh-context value compounds — each pass surfaces NEW dimensions)

## Finding ID Convention

Finding IDs use the format: `F-LP5-<SEV>-<SEQ>` (LOCAL pass 5, S-PLUGIN-PREREQ-B cascade).

## Part A — Fix Verification (pass 4 closures)

All 7 pass-4 findings verified CLOSED at HEAD d5a12e4a. No paper-closes detected.

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| F-LP4-HIGH-001 | HIGH | RESOLVED | validation.rs:247 validator reject + pipeline.rs:451 `.max(1)` runtime clamp; paper-fix-proof test verifies validation path specifically. Double defense confirmed. |
| F-LP4-MED-001 | MEDIUM | RESOLVED | 18 test fixture sites updated to `Client::builder().timeout(Duration::from_secs(30))` pattern. TD-005 P2 filed for production boot.rs wiring (PREREQ-D scope). |
| F-LP4-MED-002 | MEDIUM | RESOLVED | Regression test at pipeline_http_integration.rs:1538 verifies MAX_PAGES_PER_STEP cap trip, not just cursor non-advance. |
| F-LP4-LOW-001 | LOW | RESOLVED | execute_step docstring rewritten to accurately describe single-request semantics. |
| F-LP4-LOW-002 | LOW | RESOLVED | PipelineResult and FetchContext both annotated `#[non_exhaustive]`; FetchContext::new() constructor added. |
| F-LP4-LOW-003 | LOW | RESOLVED | `(1.0/rps).min(3600.0)` clamp at pipeline.rs:230 prevents Duration overflow. |
| F-LP4-LOW-004 | LOW | RESOLVED | AuthToken field made private; `new()` + `as_str()` accessors added at auth_provider.rs:46-63. |

**KUDOs (5 from Part A closure verification):**
- Double defense for fan_out_batch_size (validator + runtime clamp) is exemplary defense-in-depth.
- Paper-fix-proof test for validation path (not just runtime path) prevents regression inversion.
- AuthToken private field + as_str() accessor follows best practice for secret-bearing types.
- FetchContext::new() constructor pattern is correct API discipline for #[non_exhaustive] structs.
- rps clamp at 3600s prevents Duration overflow without silently accepting unbounded delays.

## Part B — New Findings (Pass 5 Dimensions: P5-A through P5-K)

### CRITICAL

_None._

### HIGH

_None._

### MEDIUM

#### F-LP5-MED-001: reqwest gzip feature gap — CrowdStrike/Cyberint gzipped responses fail opaquely

- **Severity:** MEDIUM
- **Category:** spec-fidelity / security-surface (silent data loss)
- **Location:** Cargo.toml:34 — `default-features = false, features = ["json","rustls-tls"]`
- **Description:** reqwest is configured without `gzip`, `deflate`, or `brotli` compression features. CrowdStrike and Cyberint APIs return gzip-compressed responses (Content-Encoding: gzip). When reqwest receives a gzip body without the `gzip` feature enabled, it returns raw compressed bytes. Calling `.json()` on compressed bytes fails opaquely — the error presents as JSON parse failure on bytes `0x1F 0x8B` (gzip magic number), not a decompression error. The caller cannot distinguish "API returned malformed JSON" from "API returned compressed JSON we cannot decode."
- **Evidence:** Cargo.toml line 34: `features = ["json", "rustls-tls"]` — `gzip` absent. reqwest docs state compression features must be explicitly enabled when `default-features = false`.
- **Proposed Fix:** Add `"gzip"` to the features list: `features = ["json", "rustls-tls", "gzip"]`. Single-line fix.

#### F-LP5-MED-002: audit-log asymmetry across three event paths — operator cannot observe auth-failure / data-loss

- **Severity:** MEDIUM
- **Category:** coverage-gap (SOUL.md §4 — operator observability)
- **Location:** pipeline.rs — auth_refresh path, double-401 abort path, truncation path (DI-019 10K cap)
- **Description:** Three distinct operator-observable events are not emitted: (a) `auth_refresh_triggered` fires before `acquire_token` but no closure event is emitted on Ok _or_ Err after acquire_token returns — operator cannot confirm whether refresh succeeded or failed; (b) double-401 abort returns `AuthRefreshFailed` error but no `tracing::error!` event is emitted at the abort site — the error propagates silently through the call stack; (c) truncation (DI-019 10K record cap) sets a boolean but emits no log event — operator cannot observe data-loss events without instrumenting callers.
- **Evidence:** Absence of `tracing::info!`/`tracing::error!` at the three event sites identified above.
- **Proposed Fix:** Add three `tracing` events: (a) `tracing::info!(result = ?outcome, "auth_refresh_complete")` after acquire_token; (b) `tracing::error!("double_401_abort: auth refresh failed, aborting step")` at abort site; (c) `tracing::warn!(records_dropped = drop_count, "truncation_applied: DI-019 10K cap reached")` at truncation site.

### LOW

#### F-LP5-LOW-001: extract_at_path silently accepts malformed `"$."` path

- **Severity:** LOW
- **Category:** missing-edge-cases
- **Location:** pipeline.rs — extract_at_path function
- **Description:** Path `"$."` passes validation (non-empty, starts with `$`). Split on `.` yields segments `["$", ""]`. The empty string becomes a JSON Pointer segment `""`, producing pointer path `"/"`. RFC 6901 states `""` is a valid key. On input `{"": "value"}`, this silently "succeeds" with a non-empty result. A spec author who accidentally types `"$."` receives a result when the body happens to contain an empty-string key, rather than a parse error.
- **Evidence:** Parsing logic does not reject empty path segments.
- **Proposed Fix:** Add validation step: reject paths containing empty segments after the `$` prefix (i.e., `segments.iter().skip(1).any(|s| s.is_empty())` → error).

#### F-LP5-LOW-002: PipelineExecutor + Interpolator have ZERO proptest coverage

- **Severity:** LOW
- **Category:** coverage-gap
- **Location:** pipeline.rs + interpolator.rs
- **Description:** `fan_out_batches`, `extract_at_path`, and `interpolate` are pure functions with rich edge-case space (empty inputs, unicode, special characters, boundary batch sizes, nested path segments). All three are currently covered only by deterministic unit tests. Property-based testing (proptest) would surface inputs like batch_size=1, batch_size=MAX, empty arrays, paths with dots in key names, and template variables with no binding. This gap is inconsistent with the proptest discipline established in the PREREQ-A cascade (PG-LP5-003).
- **Evidence:** No `proptest` dependency or `proptest!` macro usage in pipeline.rs or interpolator.rs.
- **Proposed Fix:** File as TD for PREREQ-C scope (proptest coverage parity audit per PG-LP5-003).

#### F-LP5-LOW-003: DESIGN-LEVEL — lazy initial-token guarantees double request on FIRST step against bearer-required APIs

- **Severity:** LOW (DESIGN-LEVEL — surfaced to orchestrator for human decision)
- **Category:** spec-fidelity / interface-gaps
- **Location:** pipeline.rs — PipelineExecutor initialization, auth_provider wiring
- **Description:** The current design initializes with `NullAuthProvider` as the default and acquires the first bearer token only upon receiving a 401 from the remote API. This guarantees that every production execution against a bearer-required API begins with: (1) request with empty/null bearer → (2) 401 response → (3) token refresh → (4) retry. This pollutes the audit log with a spurious auth failure; inflates `request_count` by 1 per pipeline execution; and counts against API quotas and rate limits. `NullAuthProvider` is semantically a test-only provider — its use as the production default inverts the intended semantics.
- **Evidence:** PipelineExecutor initialization uses NullAuthProvider unless explicitly wired; no eager-token acquisition path exists.
- **Proposed Fix:** REQUIRES ORCHESTRATOR DECISION before fix-burst-4: option (a) acquire token eagerly during pipeline initialization when AuthProvider is non-null, or option (b) pass the configured AuthProvider at construction time and require non-null. Fix-burst-4 MUST NOT close this without explicit orchestrator decision.

#### F-LP5-LOW-004: `status_code: 0` overloaded as sentinel across 11 distinct failure origins

- **Severity:** LOW
- **Category:** code-quality / interface-gaps
- **Location:** pipeline.rs — PipelineResult, StepResult, error paths
- **Description:** `status_code: 0` is used as a sentinel value across at least 11 distinct failure origins: interpolation error, network error before connect, network timeout, JSON parse error, page-cap exceeded, cursor non-advance abort, auth refresh failure, missing field in response, extract_at_path failure, rate-limit pre-check, and unknown. Downstream operators and telemetry consumers cannot distinguish these failure modes.
- **Evidence:** Multiple error arms in pipeline.rs set `status_code: 0` without differentiating the failure source.
- **Proposed Fix:** File as TD for PREREQ-D scope (status_code error classification refactor). Structural change; not a fix-burst-4 item.

#### F-LP5-LOW-005: Interpolator grammar has no escape mechanism for literal `${...}` in templates

- **Severity:** LOW
- **Category:** missing-edge-cases / interface-gaps
- **Location:** interpolator.rs — template parsing
- **Description:** The interpolation grammar treats every `${...}` token as a variable reference. There is no escape sequence (e.g., `$${...}` → literal `${...}`). Spec authors cannot include documentation strings in body templates that contain template syntax — for example, a CrowdStrike query body that explains its own format using `${variable}` notation as documentation.
- **Evidence:** No escape handling in interpolator.rs grammar.
- **Proposed Fix:** File as TD for PREREQ-C scope (grammar extensions). Not a fix-burst-4 item.

### Observations (Non-Blocking)

**O-LP5-OBS-001** — `fan_out_batches` scalar arm is unreachable from production callers
- The scalar arm of `fan_out_batches` (handling a non-array FanOutValue) is defensive dead code from the current production call path. The function is `pub` so external callers could theoretically reach it. Either make it `pub(crate)` with invariant documentation, or add a comment explaining why the defensive arm is retained. Non-blocking; does not affect correctness.

**O-LP5-OBS-002** — `find_fan_out_array` only fans out the FIRST array variable; multi-array case silently narrows
- If a spec template contains two array-typed variables, `find_fan_out_array` returns only the first match and silently ignores the second. The validator does not warn about this. For the current PREREQ-B use cases this is fine, but spec authors writing multi-array templates will receive confusing results. Validator should warn when multiple array variables are present and fan-out is configured.

**O-LP5-OBS-003** (process-gap / cross-system) — hot-reload race with in-flight `execute()`
- A `&SensorSpec` borrow held across an async `execute()` call races with arc-swap hot-reload if the spec registry swaps under the borrow. Cross-system concern — PREREQ-D scope (production host-functions wiring + arc-swap lifecycle). Noted for orchestrator to ensure PREREQ-D scope covers this interaction.

## Process-Gap Codifications

**PG-LP5-001** — Cargo feature-flag review axis (HTTP/TLS/compression/JSON dependency interaction)
- Every story that introduces or modifies a reqwest/hyper/http dependency must include explicit review of enabled features: json, rustls-tls/native-tls, gzip/deflate/brotli, stream, cookies. Missing compression features cause silent parse failures on compressed API responses. This axis must be part of the pre-merge checklist for all plugin-migration stories.

**PG-LP5-002** — Audit-log completeness review axis (every warn/error must have closure event in success and failure paths)
- Each adversarial pass must explicitly enumerate all `tracing::warn!` and `tracing::error!` call sites and verify: (a) each has a corresponding closure event for both Ok and Err outcomes, and (b) each data-loss event (truncation, cap, abort) emits a log line at the event site. SOUL.md §4 compliance requires it.

**PG-LP5-003** — Proptest-coverage parity audit (compare pure-function inventory vs proptest inventory per module)
- Each adversarial pass should explicitly enumerate pure functions in the target module and verify each has at least one proptest. For PREREQ-B: `fan_out_batches`, `extract_at_path`, `interpolate`, `find_fan_out_array`, `percent_encode_cursor` are all pure-function proptest targets.

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 2 |
| LOW | 5 |
| OBS | 3 |

**Overall Assessment:** block
**Convergence:** findings remain — fix-burst-4 required
**Readiness:** requires revision (2 MED must close before streak can advance)

**fix-burst-4 scope:**
- MUST CLOSE: F-LP5-MED-001 (reqwest gzip — single Cargo.toml line), F-LP5-MED-002 (audit-log symmetry — 3 tracing events), F-LP5-LOW-001 (malformed path detection)
- FILE AS TD (deferred): F-LP5-LOW-002 (proptest — PREREQ-C), F-LP5-LOW-004 (status_code refactor — PREREQ-D), F-LP5-LOW-005 (escape grammar — PREREQ-C)
- SURFACE TO ORCHESTRATOR: F-LP5-LOW-003 (lazy-token design — human decision required before fix)
- ACKNOWLEDGE: O-LP5-OBS-001, O-LP5-OBS-002 (non-blocking)
- NOTE FOR PREREQ-D: O-LP5-OBS-003 (hot-reload race — out of scope)

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 5 |
| **New findings** | 10 (2M + 5L + 3O — all from dimensions not covered in passes 1-4) |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 10/10 = 1.0 (all genuinely new) |
| **Median severity** | LOW (2.0 on 1-5 scale) |
| **Trajectory** | 20→10→4→7→10 |
| **Verdict** | FINDINGS_REMAIN — 2 MED block streak advancement; fix-burst-4 required |
