# Demo Evidence Report — S-CLAROTY-AUDITLOG-TIMEBOX-001

**Story:** Claroty audit_logs time-filter push-down with bounded 7-day default look-back  
**Story version:** v2.4  
**Evidence date:** 2026-08-16  
**Recorder:** demo-recorder  
**Product type:** CLI (Rust workspace)  
**Recording tool:** VHS 0.11.0 (terminal session recordings)

---

## Coverage Summary

All 8 acceptance criteria covered across 7 Red Gate tests (RG-001 through RG-007).
All 7 Red Gate tests PASS in the story worktree on `feature/S-CLAROTY-AUDITLOG-TIMEBOX-001`.

| AC | Red Gate | Recording | Status |
|----|----------|-----------|--------|
| AC-001 | RG-001 | AC-001-003-default-explicit-compound-filter | PASS |
| AC-002 | RG-002 | AC-001-003-default-explicit-compound-filter | PASS |
| AC-003 | RG-003 | AC-001-003-default-explicit-compound-filter | PASS |
| AC-004 | RG-004 | AC-004-pipeline-json-parse-backward-compat | PASS |
| AC-005 | (TOML parse — embedded in RG-001 setup) | AC-001-003-default-explicit-compound-filter | PASS |
| AC-006 | RG-005 | AC-006-007-error-and-end-only-filter | PASS |
| AC-007 | RG-006 | AC-006-007-error-and-end-only-filter | PASS |
| AC-INDEX-CLARO-001 | RG-007 | AC-INDEX-CLARO-001-parser-surface-index-option | PASS |

---

## Recording Files

All recordings run tests against the story worktree
(`/Users/jmagady/Dev/prism/.worktrees/S-CLAROTY-AUDITLOG-TIMEBOX-001/`) using
`cargo nextest` with `wiremock` mock servers. No external Claroty DTU or live credentials required.

### AC-001-003-default-explicit-compound-filter

**Covers:** AC-001 (RG-001), AC-002 (RG-002), AC-003 (RG-003), AC-005 (TOML parse embedded)

- `AC-001-003-default-explicit-compound-filter.tape` — VHS script source
- `AC-001-003-default-explicit-compound-filter.gif` — PR-embeddable recording (282 KB)
- `AC-001-003-default-explicit-compound-filter.webm` — archival recording (661 KB)

**What it proves:**

- **AC-001 / RG-001:** A Claroty `audit_logs` fetch with no time filter (`start_time=None`,
  `end_time=None`) produces an outbound POST body containing:
  ```json
  {"filter_by": {"field": "timestamp", "operation": "greater_or_equal", "value": "<now-7d ISO-8601>"}, "offset": 0, "limit": 1000}
  ```
  The value is an ISO-8601 RFC3339 string approximately 604,800 seconds before now (±60s tolerance).
  Never unbounded; eliminates E-QUERY-004 timeout. Traces to BC-2.01.013 EC-01-030.

- **AC-002 / RG-002:** A fetch with `start_time="2026-07-01T00:00:00Z"` (~45 days ago) produces
  `filter_by.value = "2026-07-01T..."` — the explicit user-supplied bound, NOT the 7-day default.
  The test confirms the value deviates from the 7-day default by more than 30 days.
  Traces to BC-2.01.013 EC-01-031.

- **AC-003 / RG-003:** A fetch with both `start_time` and `end_time` produces:
  ```json
  {"filter_by": {"operation": "and", "operands": [{"field": "timestamp", "operation": "greater_or_equal", "value": "..."}, {"field": "timestamp", "operation": "less_or_equal", "value": "..."}]}, "offset": 0, "limit": 1000}
  ```
  The compound key is `"operands"` (NOT `"conditions"`). Both values are ISO-8601 strings.
  Traces to BC-2.01.013 EC-01-033.

- **AC-005 / TOML parse:** `SpecLoader::parse` on `claroty.sensor.toml` succeeds as part of
  `make_claroty_adapter()` test fixture setup. If the TOML has a syntax error or invalid
  body_template, all three tests fail at fixture construction, not at assertions.
  Traces to BC-2.16.013 §1.

---

### AC-004-pipeline-json-parse-backward-compat

**Covers:** AC-004 (RG-004)

- `AC-004-pipeline-json-parse-backward-compat.tape` — VHS script source
- `AC-004-pipeline-json-parse-backward-compat.gif` — PR-embeddable recording (209 KB)
- `AC-004-pipeline-json-parse-backward-compat.webm` — archival recording (428 KB)

**What it proves:**

- **AC-004 / RG-004:** `pipeline.rs` `step_vars` seeding auto-parses query_filter values that
  start with `{` into `serde_json::Value::Object`. The test injects:
  - JSON-object string `'{"field": "timestamp", "operation": "greater_or_equal", "value": "2026-01-01T00:00:00Z"}'`
    → asserted as `Value::Object` in step_vars (the parsed path)
  - FQL string `'created_timestamp:>2026-01-01'` → asserted as `Value::String` via
    `assert_eq!(result, Value::String(...))` (positive backward-compat gate; NOT merely absence of panic)

  Traces to BC-2.16.013 §1 Layer-2 block — pipeline.rs auto-parse clause; BC-2.01.013 EC-003/EC-004.

---

### AC-006-007-error-and-end-only-filter

**Covers:** AC-006 (RG-005), AC-007 (RG-006)

- `AC-006-007-error-and-end-only-filter.tape` — VHS script source
- `AC-006-007-error-and-end-only-filter.gif` — PR-embeddable recording (240 KB)
- `AC-006-007-error-and-end-only-filter.webm` — archival recording (403 KB)

**What it proves:**

- **AC-006 / RG-005:** When the wiremock server returns HTTP 400, the adapter returns
  `Err(SensorError::HttpError { status: 400 })` — NOT panic, NOT silent `Vec::new()`.
  The test has a dual gate: primary (filter_by present in POST body) and secondary (error type).
  Traces to BC-2.01.013 §Postconditions error propagation clause; EC-006.

- **AC-007 / RG-006:** A fetch with `end_time="2026-01-01T00:00:00Z"` (>7 months ago) and
  `start_time=None` produces a SINGLE `less_or_equal` filter:
  ```json
  {"filter_by": {"field": "timestamp", "operation": "less_or_equal", "value": "2026-01-01T00:00:00+00:00"}, "offset": 0, "limit": 1000}
  ```
  The POST body does NOT contain `"greater_or_equal"` and does NOT contain compound `"operation": "and"`.
  Adding a 7-day floor when `end_time < now-7d` would produce an inverted/empty result window
  (SOUL.md §4 silent-wrong-result). Traces to BC-2.01.013 EC-01-032.

---

### AC-INDEX-CLARO-001-parser-surface-index-option

**Covers:** AC-INDEX-CLARO-001 (RG-007)

- `AC-INDEX-CLARO-001-parser-surface-index-option.tape` — VHS script source
- `AC-INDEX-CLARO-001-parser-surface-index-option.gif` — PR-embeddable recording (207 KB)
- `AC-INDEX-CLARO-001-parser-surface-index-option.webm` — archival recording (354 KB)

**What it proves:**

- **AC-INDEX-CLARO-001 / RG-007:** The SAP-3 parser-surface end-to-end reachability test.
  Input: PrismQL string `SELECT * FROM claroty_audit_logs WHERE timestamp > '2025-01-01T00:00:00Z'`
  Call graph: `PrismQlParser::parse` → predicate AST → `extract_time_window_from_ast`
  (reads `options = ["INDEX"]` from `claroty.sensor.toml §audit_logs.timestamp`) →
  `QueryParams.start_time = Some("2025-01-01T00:00:00+00:00")` → `build_claroty_audit_filter_by`
  → `filter_by.value ≈ "2025-01-01T..."` (NOT now-7d default).

  Without `options = ["INDEX"]` on `audit_logs.timestamp`, `extract_time_window_from_ast`
  returns `(None, None)` and the 7-day default is silently injected, discarding the user's
  explicit WHERE predicate. EC-01-031/032/033 would be unreachable from the parser surface
  (SAP-3 violation / EC-01-034).

  Traces to BC-2.01.013 EC-01-034; BC-2.16.013 v1.41 §1 `audit_logs.timestamp` INDEX prerequisite.

---

### ALL-ACs-full-suite

**Covers:** All 7 Red Gate tests (AC-001 through AC-007 + AC-INDEX-CLARO-001)

- `ALL-ACs-full-suite.tape` — VHS script source
- `ALL-ACs-full-suite.gif` — PR-embeddable recording (211 KB)
- `ALL-ACs-full-suite.webm` — archival recording (570 KB)

**What it proves:** Single composite run confirming all 7 Red Gate tests pass in the
story worktree. Authoritative PASS evidence for the full story acceptance suite.

---

## DTU Limitation Note

The Claroty DTU behavioral clone (`prism-dtu-claroty`) accepts and deserializes incoming
`filter_by` POST bodies (via `ApiQueryFilter = HashMap<String, serde_json::Value>`) but does not
apply server-side row-reduction based on the filter. It returns the full fixture regardless of
the time window requested.

**Consequence for demo evidence:** The demo recordings above prove the OUTBOUND request shape
(what prism sends to xDome) via `wiremock` request-body capture. They do not demonstrate
server-side row-reduction.

**Definitive row-reduction proof:** Obtained separately via live xDome API validation
(differential controls — comparing unfiltered vs filtered response row counts on the live
tenant). That evidence is in `docs/demo-evidence/S-CLAROTY-AUDITLOG-TIMEBOX-001/live/`
(gitignored — customer data). The implementer confirmed row-reduction behavior during
live-API validation in the story delivery session.

The DTU-level demos are sufficient to prove all behavioral contracts at the wire-shape
assertion level per CLAUDE.md wire-shape assertion discipline (2026-07-13, human-approved):
outbound `filter_by` structure, ISO-8601 value format, `operands` key correctness, error
surfacing, and end-only filter correctness are all asserted on the serialized POST body.

---

## Implementation Verification

Three code sites changed in coordination (confirmed GREEN before recording):

| File | Change | Verification |
|------|--------|--------------|
| `crates/prism-bin/src/spec_driven_adapter.rs` | Added `build_claroty_audit_filter_by` + injection block for `sensor_id="claroty" && table_name="audit_logs"` (four filter cases EC-01-030..033) | RG-001/002/003/005/006 pass |
| `crates/prism-spec-engine/src/pipeline.rs` | Extended `step_vars` seeding: JSON-object strings (`{`/`[` prefix) → `Value::Object`; plain strings remain `Value::String` | RG-004 pass |
| `crates/prism-sensors/specs/claroty.sensor.toml` | (a) `body_template = '{"filter_by": ${query.filter._claroty_audit_filter_by}}'`; (b) `options = ["INDEX"]` on `audit_logs.timestamp` column | RG-001 setup (TOML parse); RG-007 pass |

---

## BC Traceability

| Recording | AC | BC | EC |
|-----------|----|----|-----|
| AC-001-003-default-explicit-compound-filter | AC-001 | BC-2.01.013, BC-2.16.013 | EC-01-030 |
| AC-001-003-default-explicit-compound-filter | AC-002 | BC-2.01.013 | EC-01-031 |
| AC-001-003-default-explicit-compound-filter | AC-003 | BC-2.01.013 | EC-01-033 |
| AC-001-003-default-explicit-compound-filter | AC-005 | BC-2.16.013 §1 | TOML parse |
| AC-004-pipeline-json-parse-backward-compat | AC-004 | BC-2.16.013 §1, BC-2.01.013 | EC-003, EC-004 |
| AC-006-007-error-and-end-only-filter | AC-006 | BC-2.01.013 | EC-006, error surface |
| AC-006-007-error-and-end-only-filter | AC-007 | BC-2.01.013 | EC-01-032 |
| AC-INDEX-CLARO-001-parser-surface-index-option | AC-INDEX-CLARO-001 | BC-2.01.013, BC-2.16.013 | EC-01-034 |
| ALL-ACs-full-suite | All 8 ACs | BC-2.01.013 v1.22, BC-2.16.013 v1.41 | all |
