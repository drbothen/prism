---
document_type: story
story_id: S-DEMO-CLAROTY-PAGINATION-001
title: "prism-spec-engine: OffsetLimit POST-Body Pagination for Claroty (closes Gap-CL-004 / multi-page support)"
wave: 5
epic_id: E-DEMO
priority: P1
status: draft
# BC status: BC-2.16.002 is active (BC-INDEX v5.56). BC-2.16.013 is active (BC-INDEX v5.56).
# BC-2.01.013 is active. All BCs are confirmed active.
# S-7.01 gate: behavioral_contracts is non-empty and all BCs are active.
# BC GAP NOTE: CLOSED D-1059 2026-06-08 — BC-2.16.002 v1.70 now contains the explicit
#   §Postconditions "OffsetLimit Pagination Dispatch: POST-body vs GET-URL" clause.
#   DRIFT-D850-001 RESOLVED. Story may proceed to story-writer materialization per S-7.01.
version: "1.0"
level: "L4"
producer: story-writer
timestamp: "2026-05-29T00:00:00Z"
modified: "2026-05-29"
tdd_mode: strict
subsystems: [SS-16]
# Subsystem anchor justifications:
#   SS-16 (Spec Engine) owns `prism-spec-engine/src/pipeline.rs` where `build_paged_url_impl`
#   and the `OffsetLimit` pagination loop reside. The fix is entirely in SS-16 territory.
#   No DTU changes are required: the Claroty DTU's `GetAlertsBody`, `GetDevicesBody`, and
#   `GetAuditLogBody` already declare `offset: Option<u32>` and `limit: Option<u32>` fields
#   in their POST body structs (types.rs). The DTU accepts body-based pagination already.
crates_touched: [prism-spec-engine]
target_module: prism-spec-engine
capabilities: [CAP-029]
behavioral_contracts:
  - BC-2.16.002  # Structured Event Catalog + Pipeline Contract — the OffsetLimit pagination
                 # engine is governed by BC-2.16.002 §Postconditions. This story amends
                 # `build_paged_url_impl` behavior: for POST steps, offset+limit go in the
                 # request body instead of URL query params.
                 # NOTE: See §BC Gap below — a specific postcondition clause for POST-body
                 # dispatch may need PO authorship before status=ready.
  - BC-2.16.013  # DTU-Parity Verification — Gap-CL-004 is a parity gap between the TOML spec's
                 # declared pagination semantics and the pipeline's behavior. Closing it restores
                 # DTU parity for Claroty alerts and audit_logs tables.
  - BC-2.01.013  # DataSource Trait — pagination is part of the data fetch contract; multi-page
                 # results (>100 rows) require the pipeline to correctly advance the offset.
verification_properties:
  - VP-148  # VP-PLUGIN-003 DTU parity — pagination correctness is exercised by the parity
            # test that queries Claroty alerts with >100 rows against the DTU.
depends_on:
  - PLUGIN-MIGRATION-001-A  # Must merge first: established the spec-driven pipeline executor
                             # and OffsetLimit pagination loop. PR #156 merged to develop@948a709f.
  - develop@72baf413        # TOML spec has Gap-CL-004 comment citing this story. The TOML
                             # changes that set up the audit_logs and devices tables (and their
                             # POST-body pagination steps) are already in develop at this SHA.
blocks:
  - S-DEMO-002  # S-DEMO-002 AC-007 requires `FROM claroty_alerts LIMIT 150` to return 150
                # rows (multi-page). Without this fix, the pipeline returns only page 1 (100
                # rows max) because Claroty's POST endpoint ignores URL-based offset params.
# Dependency anchor justifications:
#   depends_on PLUGIN-MIGRATION-001-A: The `PipelineExecutor::execute` loop and `build_paged_url_impl`
#   were established in PLUGIN-MIGRATION-001-B/E. PLUGIN-MIGRATION-001-A is the root prerequisite
#   for the pipeline's stable surface. Already merged.
#   depends_on develop@72baf413: The TOML tables (alerts, audit_logs, devices) declaring
#   `method = "POST"` with offset_limit pagination already exist in the TOML. The pipeline
#   reads these; the fix is purely in how it dispatches offset/limit.
#   blocks S-DEMO-002: S-DEMO-002 needs multi-page queries (>100 rows) for the Claroty demo
#   to be compelling. URL-based offset params are silently ignored by the Claroty API/DTU,
#   so page 2+ is never fetched without this fix.
points: 5
# Points justification:
#   - Amend `build_paged_url_impl` to not append ?offset=&limit= for POST steps: ~0.5 pts
#   - Thread `method` (or the full `FetchStep`) through to the call site for dispatch: ~0.5 pts
#   - Amend the request-issuing path to inject offset+limit into the request body for POST: ~1 pt
#   - 4-axis Red Gate tests (POST body, GET URL, regression, multi-page): ~2 pts
#   - BC-2.16.002 catalog update if any new tracing emissions: ~0.5 pts
#   - BC-2.16.002 postcondition amendment (if PO confirms gap): ~0.5 pts
#   Total: 5 points (~1 day of focused TDD work)
#   Risk adjustment: MEDIUM (existing OffsetLimit callers that use GET must not regress).
#   4-axis test matrix is critical to safe delivery.
estimated_days: 1
risk: MEDIUM
# Risk justification:
#   `build_paged_url_impl` is called by `PipelineExecutor::execute` for ALL sensors using
#   OffsetLimit pagination. A GET-sensor regression (Cyberint, Armis, CrowdStrike) would break
#   existing tests. The 4-axis test matrix (GET URL, POST body, GET regression, POST regression)
#   is the primary mitigation. The logic change is small but the blast radius is the full
#   OffsetLimit sensor family.
assumption_validations: []
risk_mitigations: []
---

# S-DEMO-CLAROTY-PAGINATION-001: OffsetLimit POST-Body Pagination for Claroty

## Narrative

As a demo operator running a multi-page Claroty query (`FROM claroty_alerts LIMIT 150`),
I want the pipeline executor to send `offset` and `limit` in the POST request body (not URL
query params) when the fetch step's `method = "POST"`,
so that the Claroty API (and DTU) honors the pagination params and returns page 2+ data,
enabling demos that show more than 100 rows.

## Background

The current `build_paged_url_impl` in `prism-spec-engine/src/pipeline.rs` always appends
`?offset=N&limit=M` to the URL regardless of HTTP method:

```rust
Some(PaginationConfig::OffsetLimit { page_size }) => {
    let sep = if base_url.contains('?') { '&' } else { '?' };
    format!("{base_url}{sep}offset={offset}&limit={page_size}")
}
```

The Claroty xDome API (and DTU) uses `POST` for all read endpoints (`/api/v1/alerts`,
`/api/v1/devices`, `/api/v1/audit_log/get`). The Claroty API expects `offset` and `limit`
in the POST request body, not URL query parameters. The DTU's `GetAlertsBody` struct already
declares `offset: Option<u32>` and `limit: Option<u32>` — the DTU is ready; the pipeline is not.

Gap-CL-004 is registered in `claroty.sensor.toml` header comments and in
POLLER-DTU-FIDELITY-AUDIT-2026-05-29 v1.1 §3 Claroty section.

## BC Gap

**CLOSED — D-1059 2026-06-08: BC-2.16.002 v1.70 authored §Postconditions "OffsetLimit Pagination
Dispatch: POST-body vs GET-URL (DRIFT-D850-001)" clause. The explicit postcondition distinguishing
POST-body vs GET-URL offset/limit dispatch now exists. DRIFT-D850-001 is RESOLVED.**

_Historical context:_ BC-2.16.002 governed the pagination pipeline contract but did not contain an
explicit postcondition clause distinguishing POST-body vs GET-URL offset/limit dispatch. The gap
was registered as DRIFT-D850-001 and has been resolved by the PO amendment at D-1059. This story
may now proceed to story-writer materialization + PO BC-array review per S-7.01 before dispatch.
Status remains `draft` — story-writer must materialize remaining ACs and PO must confirm
`behavioral_contracts` array is complete before status advances to `ready`.

## Behavioral Contracts

| BC | Title | Version | Role |
|----|-------|---------|------|
| BC-2.16.002 | Structured Event Catalog + Pipeline Contract | v1.49 | OffsetLimit pagination engine contract; postcondition: pipeline follows declared pagination config; this story amends the dispatch logic for POST steps |
| BC-2.16.013 | Bundled Sensor Spec Authoring and DTU-Parity Verification — 4 Initial Sensors | v1.17 | Gap-CL-004 is an open DTU-parity gap; closing it restores parity for Claroty's POST-based pagination |
| BC-2.01.013 | DataSource Trait Eliminates Per-Sensor Code Duplication | v1.7 | Multi-page data fetch is part of the DataSource contract; pagination must work for all table types |

## Acceptance Criteria

### AC-001: POST steps send offset+limit in body, not URL (traces to BC-2.16.002 postcondition — pagination follows sensor spec declared config)
When `FetchStep::method == "POST"` and `PaginationConfig::OffsetLimit { page_size }` is active,
`build_paged_url_impl` returns the base URL unchanged (no `?offset=&limit=` appended). The
offset and limit values are injected into the JSON request body instead (as top-level keys
`"offset"` and `"limit"` on the existing `body_template` JSON object).

### AC-002: GET steps continue appending offset+limit as URL query params (traces to BC-2.16.002 postcondition — pagination follows sensor spec declared config / regression guard)
When `FetchStep::method == "GET"` (or absent — default GET) and `PaginationConfig::OffsetLimit`
is active, `build_paged_url_impl` continues to append `?offset=N&limit=M` to the URL.
No existing GET-sensor behavior changes. This AC is the regression guard for Cyberint, Armis,
and CrowdStrike sensors that may use GET with OffsetLimit pagination.

### AC-003: Multi-page Claroty query returns >100 rows (traces to BC-2.01.013 postcondition §1 — adapter returns all records within query limits)
A test issuing `FROM claroty_alerts LIMIT 150` against a DTU that serves exactly 102 synthetic
alert entries returns 102 rows (all entries across 2 pages). Without this fix, only 100 rows
are returned (page 1 only). This test is the integration Red Gate for demo readiness.

### AC-004: Body template merging preserves existing body fields (traces to BC-2.16.002 postcondition — body_template)
When offset+limit are injected into the POST body, any existing keys from `body_template`
(e.g., `{}` for Claroty, or filter params if present) are preserved. The pagination params
are merged into the existing body object, not replacing it.

### AC-005: First-page request uses offset=0 (traces to BC-2.16.002 postcondition — OffsetLimit starts at offset 0)
For the first pagination step, `offset = 0` and `limit = page_size` are included in the body.
Subsequent pages increment offset by `page_size`. This matches the semantics of the existing
URL-based OffsetLimit implementation.

### AC-006: `build_paged_url_for_test` public test helper remains callable for GET paths (traces to BC-2.16.002 postcondition — test regression guard)
The existing `build_paged_url_for_test` public test helper (used in `#[cfg(test)]` modules)
remains usable and returns correct URL-appended results for GET steps. If the function signature
changes to include `method`, the public test helper wrapper `build_paged_url_for_test` must
be updated accordingly.

## Red Gate Tests

| Test name | Test type | What it gates |
|-----------|-----------|---------------|
| `test_BC_2_16_002_pagination_post_method_sends_offset_limit_in_body` | Unit (pipeline.rs) | AC-001: POST step body contains offset+limit; URL unchanged |
| `test_BC_2_16_002_pagination_get_method_continues_url_params` | Unit (pipeline.rs) | AC-002: GET step URL still has ?offset=&limit= appended |
| `test_BC_2_16_002_pagination_body_template_merge_preserves_existing_keys` | Unit (pipeline.rs) | AC-004: merge does not clobber body_template existing keys |
| `test_BC_2_16_002_pagination_claroty_alerts_page_2_returns_data` | Integration (against DTU) | AC-003: 102-row fixture returns 102 rows via 2 paginated POST requests |

Test naming follows the prism convention: `test_BC_<id>_<description>` (CLAUDE.md §Conventions).

The integration test `test_BC_2_16_002_pagination_claroty_alerts_page_2_returns_data` requires
the ClarotyClone DTU running (with a 102-entry alerts fixture). It may be `#[ignore]` gated
pending S-DEMO-001 full boot wiring, with a companion unit test driving the pagination logic
directly through `PipelineExecutor::execute_with_max_requests` and a mock HTTP layer.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `build_paged_url_impl` | `crates/prism-spec-engine/src/pipeline.rs` | Pure (URL construction / body mutation) |
| `build_paged_url` | `crates/prism-spec-engine/src/pipeline.rs` | Pure (wrapper) |
| `build_paged_url_for_test` | `crates/prism-spec-engine/src/pipeline.rs` | Pure (test helper) |
| `PipelineExecutor::execute` (body injection site) | `crates/prism-spec-engine/src/pipeline.rs` | Effectful (HTTP dispatch) |
| `FetchStep::method` | `crates/prism-spec-engine/src/spec_parser.rs` | Pure (data) |

Architecture section references:
- `architecture/module-decomposition.md` §SS-16 Spec Engine (PipelineExecutor owns pipeline.rs)
- `architecture/dependency-graph.md` §Wave-5 demo stories

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | POST step with empty `body_template` (`{}`) | Offset+limit merged into empty object: `{"offset": 0, "limit": 100}` |
| EC-002 | POST step with non-object `body_template` (e.g., raw string) | Treat as parse error; surface `SpecEngineError` with sensor_id and step_name. Do NOT panic |
| EC-003 | GET step with OffsetLimit pagination (regression) | URL params appended as before: `?offset=0&limit=100`. Body unchanged (GET has no body) |
| EC-004 | `method` field absent from `FetchStep` TOML | Defaults to GET behavior (URL params). No change to existing behavior |
| EC-005 | First page has exactly `page_size` records (boundary) | Pipeline requests page 2. If page 2 has 0 records, loop terminates. This is correct OffsetLimit termination logic — NOT changed by this story |
| EC-006 | `page_size = 0` in TOML | Treated as degenerate config — existing behavior applies (divide-by-zero guard should already exist in OffsetLimit loop; if not, add one) |

## Token Budget Estimate

| Item | Estimated tokens |
|------|-----------------|
| This story spec | ~4,000 |
| `crates/prism-spec-engine/src/pipeline.rs` (full read required) | ~40,000 |
| `crates/prism-spec-engine/src/spec_parser.rs` (PaginationConfig + FetchStep) | ~8,000 |
| `crates/prism-dtu-claroty/src/types.rs` (verify body fields) | ~4,000 |
| `crates/prism-sensors/specs/claroty.sensor.toml` (method + pagination config) | ~3,000 |
| BC files (3 BCs: BC-2.16.002, BC-2.16.013, BC-2.01.013) | ~12,000 |
| **Total estimate** | **~71,000 tokens** |

pipeline.rs is large (~40K tokens). This is within budget (20-30% of 200K = 40-60K for code
alone + story + BCs). If context pressure is felt, load only §pagination-related functions
(lines near `build_paged_url_impl`, `OffsetLimit` match arms, and `issue_request_with_retry`).

## Tasks

- [ ] **Task 1: Understand current implementation** — Read `pipeline.rs` §`build_paged_url_impl`
  (around line 939) and the `PipelineExecutor::execute` pagination loop (around line 498).
  Understand where `build_paged_url` is called and where the HTTP request body is constructed.
  Read `spec_parser.rs` `FetchStep` struct to confirm the `method` field type and how it is
  threaded.

- [ ] **Task 2: Amend `build_paged_url_impl` signature** — Add `method: &str` (or `step: &FetchStep`)
  parameter to distinguish GET vs POST dispatch. For POST: return `base_url.to_string()` unchanged
  (no ?offset= appended). For GET (or method absent): preserve existing `?offset=N&limit=M`
  behavior.

- [ ] **Task 3: Body injection** — In the request-issuing path (`issue_request_with_retry` or
  the main execute loop), when `step.method == "POST"` and `PaginationConfig::OffsetLimit`:
  parse `step.body_template` as `serde_json::Map`, insert `"offset": offset_u64` and
  `"limit": page_size_u64`, serialize back to string. Pass as request body.
  If `body_template` is not a valid JSON object, surface `SpecEngineError` (use an existing
  error variant that best fits; do NOT invent a new error code without PO authorship of
  the error-taxonomy entry).

- [ ] **Task 4: Update `build_paged_url_for_test` wrapper** — If `build_paged_url_impl`
  signature changes, update the public test helper `build_paged_url_for_test` accordingly.
  Check all existing callers in the test module (`test_BC_2_16_002_cursor_pagination_*`)
  and update their call sites (TD-VSDD-060 sibling-site sweep).

- [ ] **Task 5: Red Gate tests** — Write all 4 Red Gate tests listed above. The integration
  test against a live DTU clone requires `ClarotyClone` in `[dev-dependencies]` if not already
  present. Verify before adding — check `crates/prism-spec-engine/Cargo.toml` dev-dependencies.

- [ ] **Task 6: Regression sweep** — After implementation, run `just iter prism-spec-engine`
  (all prism-spec-engine tests). Confirm 0 regressions in existing OffsetLimit or CursorToken
  pagination tests. Any regression must be fixed before declaring the Red Gate clean.

- [ ] **Task 7: BC-2.16.002 catalog check (SAP-1)** — If any new `tracing::*!(event_type = ...)`
  emissions are added (e.g., for body-merge errors), add catalog rows to BC-2.16.002 in the
  same commit. Check existing catalog for any emission that covers body merge failure;
  reuse if semantically appropriate.

- [ ] **Task 8: BC gap report** — After implementation, confirm whether BC-2.16.002 explicitly
  covers POST-body OffsetLimit dispatch. If no existing postcondition clause covers it, draft
  a finding for the product-owner: "BC-2.16.002 postcondition N (new) needed: OffsetLimit
  POST-step dispatch sends offset+limit in body." Do NOT author the BC clause yourself
  (product-owner owns BC authorship per Agent Routing Table in CLAUDE.md).

## Previous Story Intelligence

This story is in E-DEMO epic (not E-DTU-FIDELITY). It shares the Wave 5 slot with
S-DEMO-CLAROTY-AUDIT-DTU-001 (E-DTU-FIDELITY). Key lessons from co-authored wave:

1. **Shared DTU struct awareness.** `S-DEMO-CLAROTY-AUDIT-DTU-001` (authored in same burst)
   adds `GetAuditLogBody` with `offset: Option<u32>` and `limit: Option<u32>`. When
   S-DEMO-CLAROTY-PAGINATION-001 lands, the DTU already accepts body-based pagination for
   all 3 Claroty POST endpoints (alerts, devices, audit_log). No DTU changes needed here.

2. **pipeline.rs context size.** `pipeline.rs` is large (~2200+ lines). Load only the
   pagination-relevant sections first (lines ~360-510 for the execute loop, lines ~908-968
   for `build_paged_url_impl`, lines ~2250-2350 for existing pagination unit tests).
   Do not load the entire file if the context budget is under pressure.

3. **`PaginationConfig::OffsetLimit` is matched in 2 places** in `pipeline.rs`:
   - `build_paged_url_impl` (URL construction)
   - The pagination advance logic in `PipelineExecutor::execute` (loop termination)
   Only `build_paged_url_impl` needs to change (plus the body injection site). The advance
   logic is method-agnostic and must NOT be changed.

4. **Existing Red Gate tests for pagination.** `test_BC_2_16_002_cursor_pagination_*` tests
   exist in `pipeline.rs` (around line 2290). These test CursorToken pagination. The new tests
   are OffsetLimit-specific. Do not modify the existing CursorToken tests.

## Architecture Compliance Rules

From `architecture/module-decomposition.md` §SS-16 Spec Engine:

- `PipelineExecutor` is the sole owner of pagination logic. Pagination dispatch must NOT
  be duplicated in sensor adapters or the query engine.
- `build_paged_url_impl` is a pure function (no side effects). The body injection happens
  at the request-issuing site, not inside `build_paged_url_impl`.
- Error codes for new failure modes must be registered in
  `.factory/specs/prd-supplements/error-taxonomy.md` before shipping (PO authors; implementer
  uses existing codes if semantically applicable).

From ADRs:

- ADR-028 §D8 governs the OffsetLimit pagination engine. Any change to its semantics should
  cite this ADR in the implementation PR description.
- ADR-028 §D1: TOML spec method field is the source of truth for HTTP method. The dispatch
  in this story reads `step.method` — consistent with ADR-028 §D1.

## Library & Framework Requirements

| Library | Version | Source |
|---------|---------|--------|
| `serde_json` | per `Cargo.toml` workspace pin | Body JSON manipulation (`Map`, `Value`) |
| `reqwest` | per `Cargo.toml` workspace pin | HTTP client (existing; body injection) |
| `prism-dtu-claroty` | workspace path (dev-dep) | Integration test DTU clone |

Check `crates/prism-spec-engine/Cargo.toml` before adding `prism-dtu-claroty` to dev-deps.
If it is not already present, add it under `[dev-dependencies]` only.

## File Structure Requirements

| Action | File path | Notes |
|--------|-----------|-------|
| MODIFY | `crates/prism-spec-engine/src/pipeline.rs` | Amend `build_paged_url_impl`, add body injection, add Red Gate tests |
| MODIFY (if needed) | `crates/prism-spec-engine/Cargo.toml` | Add `prism-dtu-claroty` to `[dev-dependencies]` if not present |
| NO CHANGE | `crates/prism-sensors/specs/claroty.sensor.toml` | TOML already correct |
| NO CHANGE | `crates/prism-dtu-claroty/src/types.rs` | DTU already has offset/limit body fields |

## Forbidden Dependencies

`prism-spec-engine` MUST NOT gain a new PRODUCTION dependency on `prism-dtu-claroty`.
The DTU crate may appear in `[dev-dependencies]` only (test infrastructure).

## Notes for Implementer

1. **Identify the body injection site precisely.** The HTTP request body is constructed
   in `issue_request_with_retry` (around line 703). Find where `step.body_template` is
   serialized into the request body. The injection point is there — not inside
   `build_paged_url_impl` (which is a URL builder, not a body builder).

2. **Four-axis regression test is mandatory.** The MEDIUM risk rating comes from the blast
   radius of changing `build_paged_url_impl`. Write all 4 Red Gate tests before declaring
   the story complete. Do not declare "tests follow after PR" — Red Gate tests are the gate.

3. **BC gap report is non-optional.** At the end of implementation, file a finding for the
   PO (or record in your PR description) whether BC-2.16.002 covers POST-body OffsetLimit
   dispatch. This is a tracking obligation, not a blocker for the implementation itself.

4. **Error variant for body merge failure.** If `body_template` is not a JSON object and
   cannot be merged with offset/limit, use the most semantically appropriate existing
   `SpecEngineError` variant. Check `prism-spec-engine/src/error.rs` for candidates
   (e.g., `JsonPathExtractionFailed`, `InvalidSpec`, or similar). Do NOT invent new variants
   without PO authorship of the corresponding error-taxonomy entry.

5. **`claroty.sensor.toml` comment in body_template section.** The TOML comment reads:
   "F-LP3-HIGH-004: removed `{'size': 100}` from body_template. The OffsetLimit engine
   appends ?offset=N&limit=M to URL." After this story lands, that comment should be updated
   to reflect that OffsetLimit POST steps inject into body. Include the TOML comment update
   in your PR (it is the same file; a follow-up is not warranted for a doc-comment update).
