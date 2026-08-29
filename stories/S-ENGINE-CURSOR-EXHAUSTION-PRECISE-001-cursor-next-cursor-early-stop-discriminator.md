---
document_type: story
story_id: S-ENGINE-CURSOR-EXHAUSTION-PRECISE-001
title: "Cursor pagination: next-cursor-presence-based early-stop discriminator (ADR-060 §D8.4)"
level: "L4"
wave: TBD
epic_id: E-XDOME-EXPANSION
priority: P2
status: draft
# BC status: pending PO authorship — behavioral_contracts is empty. Per S-7.01, this story
# MUST remain draft until a product-owner authors and anchors BCs with canonical IDs matching
# BC-\d+\.\d{2}\.\d{3}. No BC covers next-cursor-presence-based exhaustion detection at this time.
producer: story-writer
timestamp: "2026-08-29T00:00:00Z"
version: "1.0"
modified: "2026-08-29"
phase: 3
cycle: v1.0.0-brownfield
inputs:
  - ".factory/specs/architecture/decisions/ADR-060-limit-aware-early-stop-pagination.md"
input-hash: "69b70a3"
# input-hash: 69b70a3 — computed from ADR-060 v1.13 (ADR-060 §D8.4 cursor revert ruling)
traces_to: ["BC-2.16.002", "BC-2.11.001"]
# traces_to: BC-2.16.002 + BC-2.11.001 govern the early-stop signal and truncation surface
# that this story extends. Precise BCs pending PO authorship.
points: 5
# points: 5 estimated — cursor presence extraction is a targeted addition to execute_impl; no
#   new FetchContext field needed; scope is a single-branch discriminator replacement.
estimated_days: 1
tdd_mode: strict
subsystems: [SS-16, SS-07]
# Subsystem anchor justifications:
#   SS-16 (Spec Engine) owns this story's scope: implementation is in
#     `crates/prism-spec-engine/src/pipeline.rs §PipelineExecutor::execute_impl` (cursor extraction).
#   SS-07 (Adapter Pagination & Response Cache) owns this story's scope: the per-page
#     cursor-exhaustion check fires within the CursorToken pagination loop (SS-07 governs
#     adapter pagination per ARCH-INDEX).
target_module: prism-spec-engine
crates_touched: [prism-spec-engine]
capabilities:
  - CAP-029
behavioral_contracts: []
# BC status: pending PO authorship (S-7.01 gate — behavioral_contracts: [] blocks status=ready)
# Traceability parent: BC-2.16.002 §Postconditions LIMIT-Aware Early-Stop (mode-scope §D8.4),
# BC-2.11.001 EC-11-094 (partial-final-page discriminator). PO must author or amend BCs before
# this story can be dispatched.
verification_properties: []
holdout_scenarios: []
# holdout_scenarios: PO authors 2–4 hidden SINGLE-USE scenarios at remove-uncertainty time.
# Story-level holdout gate is BLOCKING before demo/push (human-approved 2026-07-13).
depends_on: [S-ENGINE-LIMIT-EARLY-STOP-001, S-OCSF-FIDELITY-CYBERINT-001]
# depends_on justifications:
#   S-ENGINE-LIMIT-EARLY-STOP-001: establishes `active_page_size` match with `_ => 0` conservative
#     CursorToken policy. This story replaces the conservative fallback with precise detection;
#     the parent story must be fully delivered first.
#   S-OCSF-FIDELITY-CYBERINT-001: first cursor-pagination sensor delivery (post-v1). This story
#     requires a cursor-paginated sensor in production to validate the exhaustion signal against
#     real page bodies. Blocked until that sensor ships.
blocks: []
acceptance_criteria_count: 0
# acceptance_criteria_count: 0 — draft stub; ACs to be authored when PO writes BCs
red_gate_tests: 0
# red_gate_tests: 0 — draft stub; RG tests to be authored when ACs exist
risk: MEDIUM
# Risk justification: next-cursor extraction requires parsing the page response body in the
#   pagination loop before the early-stop break — must be correct for all cursor-paginated
#   sensors. Wrong extraction yields incorrect is_truncated signals at the MCP surface.
assumption_validations: []
risk_mitigations: []
---

# S-ENGINE-CURSOR-EXHAUSTION-PRECISE-001: Cursor Pagination — Next-Cursor-Presence-Based Early-Stop Discriminator

> **DRAFT STUB — NOT FOR IMMEDIATE DELIVERY.** This story captures the ADR-060 §D8.4
> architectural deferral and defines the precise cursor-exhaustion detection approach.
> Blocked by S-OCSF-FIDELITY-CYBERINT-001 (first cursor-paginated sensor delivery).
> PO must author BCs before this story can be dispatched. Status MUST remain `draft`
> until `behavioral_contracts:` is populated per S-7.01.

## Authority

**ADR-060 §D8.4** is the governing architectural decision. It specifies:

> "Page-fill (`page_record_count >= page_size`) is not a valid cursor exhaustion signal
> because cursor-paginated APIs do not guarantee full pages — many return variable-size
> pages even when more pages exist. Permitting early-stop based on page-fill for CursorToken
> yields incorrect `early_stopped=false` (and thus `is_truncated=false`) on partial pages
> even when the cursor API has more data to return. Conservative treatment:
> `CursorToken (all sub-cases) → active_page_size = 0 → early_stopped = true`.
> The correct discriminator for cursor exhaustion is **next-cursor presence**: if the page
> response contains a non-empty next-cursor token, more pages exist and `early_stopped = true`;
> if the cursor is absent or empty, the source is exhausted and `early_stopped = false`.
> Precise detection deferred to S-ENGINE-CURSOR-EXHAUSTION-PRECISE-001 (blocked by
> S-OCSF-FIDELITY-CYBERINT-001, the first cursor-pagination sensor delivery)."

**BC-2.16.002** and **BC-2.11.001** are the parent behavioral contracts. Precise BCs governing
next-cursor extraction are pending PO authorship.

---

## Narrative

As a SOC analyst issuing a `LIMIT N` query against a cursor-paginated sensor table,
I want the pipeline to correctly detect when the cursor API has no more pages to return,
so that `is_truncated=false` is emitted when the source is genuinely exhausted (not just
because the final page happened to be smaller than the declared page size).

## Background

S-ENGINE-LIMIT-EARLY-STOP-001 delivered the partial-final-page discriminator for `OffsetLimit`
pagination (ADR-060 §D8.2). For `CursorToken` pagination, the discriminator was intentionally
kept conservative (`active_page_size = 0` → `early_stopped = true` always) per ADR-060 §D8.4,
because page-fill is not a valid exhaustion signal for cursor APIs.

The precise cursor-exhaustion discriminator works differently: after fetching each page,
extract the `next_cursor` value from the response body BEFORE the early-stop `break`. If
`next_cursor` is non-empty, more pages exist → `early_stopped = true`. If absent or empty,
the API has returned all data → `early_stopped = false` → `is_truncated = false`.

This change affects the conservative over-reporting of `is_truncated=true` for cursor-paginated
sensors when the final page is a partial page (the common "small final page" case).

## Behavioral Contracts

| BC | Title | Version | Role |
|----|-------|---------|------|
| BC-2.16.002 | Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation | pending | Parent contract — §Postconditions LIMIT-Aware Early-Stop §D8.4 mode-scope to be amended when PO authors precise cursor-exhaustion BC. |
| BC-2.11.001 | MCP Query Tool Response — Pagination Signals | pending | Parent trace — EC-11-094 partial-final-page discriminator scope to be extended to CursorToken when PO authors precise BC. |

*No BCs are in `behavioral_contracts:` frontmatter (stub; PO authorship required per S-7.01).*

## Acceptance Criteria

*N/A — draft stub. ACs to be authored by PO when BCs are written. The scope is:*

- *AC-001 (placeholder): In `execute_impl`, after fetching each CursorToken page and BEFORE the early-stop `break`, extract `next_cursor` from the page response body. Use next-cursor presence (`next_cursor.is_some() && !next_cursor_value.is_empty()`) as the exhaustion signal: present → `early_stopped = true`; absent → `early_stopped = false`. Discriminator replaces the conservative `active_page_size = 0` CursorToken path.*
- *AC-002 (placeholder): RG test verifying a partial final cursor page with absent next_cursor → `early_stopped = false` → `is_truncated = false`.*
- *AC-003 (placeholder): RG test verifying a partial cursor page with present next_cursor → `early_stopped = true` → `is_truncated = true`.*

## Red Gate Tests

*N/A — draft stub. RG tests enumerated after ACs are authored.*

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `PipelineExecutor::execute_impl` cursor-exhaustion check | `crates/prism-spec-engine/src/pipeline.rs §PipelineExecutor::execute_impl` | Effectful (within HTTP pagination loop; discriminator is pure logic on response body) |
| Next-cursor extraction | `crates/prism-spec-engine/src/pipeline.rs §CursorToken pagination branch` | Pure (read from page response; no I/O) |

Architecture section references:
- `architecture/module-decomposition.md` §SS-16 Spec Engine (prism-spec-engine; PipelineExecutor)
- `architecture/module-decomposition.md` §SS-07 Adapter Pagination & Response Cache (CursorToken loop)
- ADR-060 §D8.4 — cursor exhaustion discriminator (precise detection deferred from §D8)

---

## Purity Classification

*N/A — draft stub. Full purity table to be authored when ACs and implementation scope are finalized.*

| Element | Classification | Rationale |
|---------|---------------|-----------|
| Next-cursor extraction from page response | **Pure** | Reads a field from already-fetched page data; no I/O. |
| Early-stop discriminator decision | **Pure decision inside an Effectful loop** | The `next_cursor.is_empty()` comparison is pure control flow within the effectful HTTP pagination loop. |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Cursor API returns empty string for next_cursor on final page | Treated as absent (exhausted) → `early_stopped = false` |
| EC-002 | Cursor API returns null for next_cursor field | Treated as absent → `early_stopped = false` |
| EC-003 | Cursor API omits next_cursor field entirely on final page | Treated as absent → `early_stopped = false` |
| EC-004 | Cursor API returns a valid non-empty next_cursor on partial page | `early_stopped = true` → `is_truncated = true` |
| EC-005 | Different cursor field name across sensor adapters | Extraction must be sensor-spec-driven (field name from TOML cursor config, not hardcoded) |

## Token Budget Estimate

*N/A — draft stub. Token budget to be estimated when ACs are authored and file list is finalized.*

## Tasks

*N/A — draft stub. Tasks to be authored when ACs are written.*

1. (placeholder) Write RG tests for next-cursor-present and next-cursor-absent scenarios
2. (placeholder) Extend `execute_impl` CursorToken branch to extract next_cursor before break
3. (placeholder) Update `active_page_size` match: replace `_ => 0` CursorToken conservative path with precise detection
4. (placeholder) TD-VSDD-060 sibling sweep: grep for CursorToken early-stop callers

## Previous Story Intelligence

**S-ENGINE-LIMIT-EARLY-STOP-001 (predecessor):** Delivered the `active_page_size` match with
`_ => 0` conservative CursorToken policy and RG-PSG-041/042/043 regression sentinels. This
story replaces the `_ => 0` catch-all with a `CursorToken` arm that extracts next-cursor from
the page response. The regression sentinels from the predecessor story must be updated/removed
when this story ships: RG-PSG-041 (`test_cursor_token_partial_page_conservative_early_stopped`)
will become incorrect (it asserts `early_stopped=true`; the new precise behavior would yield
`early_stopped=false` for an absent next_cursor). RG-PSG-042/043 remain valid.

N/A — first story implementing precise cursor exhaustion detection.

## Architecture Compliance Rules

From ADR-060 §D8.4:
- Next-cursor extraction MUST occur BEFORE the early-stop `break` in `execute_impl`. Extracting
  after the break means the cursor value is unavailable (the page fetch already completed;
  the cursor is part of the response body returned in the same page fetch).
- The cursor field name MUST be read from the sensor TOML `cursor_token` config block, not
  hardcoded. Different sensors may use different field names for their cursor/pagination token.
- The `_ => 0` CursorToken conservative catch-all in `active_page_size` match MUST remain
  in place for any `PaginationConfig` variant that is NOT `CursorToken` (or is a new
  `#[non_exhaustive]` variant) — the conservative default is correct for unknown modes.

## Library & Framework Requirements

Same as S-ENGINE-LIMIT-EARLY-STOP-001:
- Rust stable (per `rust-toolchain.toml`)
- `wiremock` for HTTP mocking in integration tests
- No new external dependencies expected

*Versions: N/A — draft stub. Precise library versions deferred until implementation planning.*

## File Structure Requirements

*N/A — draft stub. File list to be authored when ACs are written.*

Expected files to modify:
- `crates/prism-spec-engine/src/pipeline.rs` — CursorToken branch in `execute_impl`
- `crates/prism-spec-engine/tests/bc_2_16_002_early_stop_tests.rs` — update/add RG tests

## References

- ADR-060 §D8.4 — cursor exhaustion discriminator architectural decision and deferral rationale
- BC-2.16.002 §Postconditions LIMIT-Aware Early-Stop (§D8.4 mode-scope paragraph) — parent contract
- BC-2.11.001 EC-11-094 (trace reference) — partial-final-page discriminator, OffsetLimit scope
- S-ENGINE-LIMIT-EARLY-STOP-001 — predecessor story establishing conservative cursor policy
- S-OCSF-FIDELITY-CYBERINT-001 — blocking story (first cursor-paginated sensor delivery)

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.0 | 2026-08-29 | story-writer | Initial draft stub. Discharges ADR-060 §D8.4 TD-VSDD-097 Dim-3 mandate anchor obligation: the `MUST remain draft` note in S-ENGINE-LIMIT-EARLY-STOP-001 §D8.4 is now anchored to this concrete story ID. `behavioral_contracts: []` per S-7.01 (pending PO authorship). `depends_on: [S-ENGINE-LIMIT-EARLY-STOP-001, S-OCSF-FIDELITY-CYBERINT-001]` per ADR-060 §D8.4 deferral rationale. |
