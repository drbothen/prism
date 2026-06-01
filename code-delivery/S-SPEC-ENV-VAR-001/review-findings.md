# Review Findings — S-SPEC-ENV-VAR-001 / PR #165

**PR:** https://github.com/drbothen/prism/pull/165
**Branch:** feature/S-SPEC-ENV-VAR-001
**HEAD at PR creation:** df80d47b7bd66678a5fcc35c643d67049b06d7db
**Story:** S-SPEC-ENV-VAR-001 v1.2

---

## Review Summary

| Reviewer | Type | Cycle | Verdict |
|----------|------|-------|---------|
| pr-manager (inline) | security-review + pr-review + code-review | 1 | REQUEST_CHANGES (M-001 medium) |

---

## Finding Table

| ID | Severity | Category | Description | Routing | Status |
|----|----------|----------|-------------|---------|--------|
| SEC-001/M-001 | LOW/MEDIUM | Security + Code | `step.method` field resolved by env resolver but not whitelist-validated at spec-load. Resolved value flows to pipeline.rs where `_` → GET provides safe fallback, but no E-SPEC-0xx for invalid HTTP method at load time. | `implementer` (add method validation in `validation.rs`) | OPEN |
| M-002 | LOW | Code efficiency | Duplicate token in `resolved_pairs` when same `${env.VAR}` appears multiple times in one field — `String::replace` is called twice; second call is a no-op. Minor inefficiency; not a correctness bug. | `implementer` (dedup with HashSet) — OR WAIVE as pre-merge concern | OPEN |
| L-001 | OBS | Convention | `test_E_SPEC_024_*` test name in `error.rs` is non-snake-case; no `#[allow(non_snake_case)]` attribute. Clippy passes (CI confirms), so this is a style note only. | WAIVE — Clippy passes, pre-existing pattern | WAIVED |
| OBS-001 | OBS | Security | TOCTOU on `std::env::var` — inherent language limitation at spec-load time. Accepted risk (single-threaded boot context). | ACCEPT | WAIVED |
| OBS-002 | OBS | Code | SAP-1 tracing emissions scan — zero new `event_type =` emissions in diff. | N/A | CLEAN |
| OBS-003 | OBS | Code | Second-order substitution: `resolve_field` does not re-scan after replacement, so env var values containing `${env.X}` are inserted literally without triggering a second pass. Correct behavior. | N/A | CLEAN |

---

## Convergence Tracking

| Cycle | Total Findings | Blocking | Fixed | Remaining | Streak |
|-------|---------------|----------|-------|-----------|--------|
| LOCAL-1 | 3 | 3 | 3 | 0 | — |
| LOCAL-2 | 1 | 1 | 1 | 0 | — |
| LOCAL-3 | 0 | 0 | 0 | 0 | 1/3 |
| LOCAL-4 | 0 | 0 | 0 | 0 | 2/3 |
| LOCAL-5 | 0 | 0 | 0 | 0 | 3/3 CONVERGED |
| PR-1 (pr-manager inline) | 2 MEDIUM + 1 LOW + 3 OBS | 0 blocking | — | 2 open (M-001, M-002) | pending adversary cycle |

**Note:** M-001 and M-002 are non-blocking at PR-merge level (CLEAN(PR-merge): yes). They are flagged for orchestrator disposition — the orchestrator will decide whether to route to implementer for a fix-burst before merge, or record as follow-up stories.

---

## CI Status (at time of report)

| Check | Status |
|-------|--------|
| Format check | PASS |
| Clippy (AD-008) | PASS |
| Cargo audit (RustSec) | PASS |
| Cargo deny (license + advisory) | PASS |
| Semver compatibility | PASS |
| WASM32 compile check | PASS |
| Workspace crate layout (ADR-012) | PASS |
| Perimeter symbols sync (OBS-001) | PASS |
| Deep-recursion lint (OBS-002) | PASS |
| Verify workflow structure | PASS |
| Test matrix (6 platforms) | PENDING |
| Compile-fail gates (3 checks) | PENDING |
| Clippy (second matrix runner) | PENDING |
