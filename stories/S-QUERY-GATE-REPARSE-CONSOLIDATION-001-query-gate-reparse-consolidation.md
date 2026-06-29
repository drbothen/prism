---
document_type: story
story_id: S-QUERY-GATE-REPARSE-CONSOLIDATION-001
title: "prism-query: Query Gate Re-Parse Consolidation — Single Parse Pass Shared Across execute_inner / execute_scheduled_inner Availability Gates (E-QUERY-037/038/039)"
wave: post-demo-backlog
# Wave assignment: NOT wave-scheduled. Post-demo backlog — this is a performance
# optimization with no correctness impact at current demo scale (single-pass gate
# sequence already correct post-S-DEMO-FIDELITY-REMEDIATION-001). Dispatched after
# the T13/T14 demo pipeline completes.
epic_id: maintenance
priority: P3
# Priority rationale: P3 (performance optimization only). The 4x re-parse is
# observable in profiling but produces no user-visible correctness difference at demo
# scale. All four gates (check_table_availability, check_query_column_availability,
# check_enrich_udf_availability, run_materialization_pipeline) already operate on the
# correct logical parse result; this story eliminates redundant parse overhead by
# threading the already-parsed AST through the gate sequence instead of re-parsing
# the raw query string at each gate entry point.
status: draft
# BC status: pending PO authorship. No BC has been authored for this performance-
# only surface yet. Per Spec-First Gate S-7.01, behavioral_contracts MUST be
# non-empty before status can advance to ready. A PO must author and anchor a new
# BC (or extend BC-2.11.001 or a sibling query-execution contract) covering the
# single-parse-pass invariant before this story is dispatched.
# Origin: deferred from S-DEMO-FIDELITY-REMEDIATION-001 §Deferred/Out-of-Perimeter
# Items table (ADV-P208-P01-001 deferral anchor) — 2026-06-29.
version: "1.0"
created: "2026-06-29"
modified: "2026-06-29"
producer: story-writer
timestamp: "2026-06-29T00:00:00Z"
phase: 3
tdd_mode: strict
level: "L4"
subsystems: [SS-11]
# Subsystem anchor justification (per ARCH-INDEX Subsystem Registry):
#   SS-11 (Query Execution, query-engine.md, crate prism-query) owns this story's
#   entire scope: execute_inner, execute_scheduled_inner, and all plan-time
#   availability gates reside in prism-query/src/engine.rs and
#   prism-query/src/table_registry.rs. No other subsystem is touched.
target_module: prism-query
crates_touched: [prism-query]
points: 5
# Points estimate:
#   - Understand current parse call sites in execute_inner + execute_scheduled_inner: 1 pt
#   - Refactor to parse once at entry point, thread ParsedQuery/Ast through all 4 gates: 3 pts
#   - TDD Red Gate tests (benchmark or assert parse count = 1 per execute call): 1 pt
#   Total: 5 pts
estimated_days: 1
depends_on: [S-DEMO-FIDELITY-REMEDIATION-001]
# Dependency anchor: S-DEMO-FIDELITY-REMEDIATION-001 introduces the 4-gate sequence
# (check_table_availability → check_query_column_availability →
# check_enrich_udf_availability → run_materialization_pipeline) in engine.rs.
# This consolidation story builds on top of that landed gate sequence — the refactor
# targets the already-correct gate order delivered by S-DEMO-FIDELITY-REMEDIATION-001.
blocks: []
behavioral_contracts: []
# BC status: pending PO authorship (Spec-First Gate S-7.01 — no BC may be
# empty or placeholder; story remains draft until PO authors and anchors a BC
# covering the single-parse-pass invariant. Candidate: extend BC-2.11.001
# postconditions with a "parse count ≤ 1 per execute call" invariant, or author
# a net-new query-engine performance contract.)
verification_properties: []
assumption_validations: []
risk_mitigations: []
red_gate_tests: 0
# Red Gate tests: 0 at authoring time (no BC yet, no contract to gate against).
# After PO authors the BC, test-writer will derive tests measuring parse invocation
# count per execute_inner call under controlled conditions.
acceptance_criteria_count: 0
# ACs: 0 at stub authoring. To be populated after PO authors the BC and product-
# owner confirms the performance target (e.g., "parse invoked ≤ 1 time per
# execute call on the fast path; existing correctness tests unaffected").
input-hash: "TBD"
inputs:
  - ".factory/stories/S-DEMO-FIDELITY-REMEDIATION-001-demo-fidelity-code-fixes.md"
  # Primary input: the gate-sequence implementation delivered by the parent story.
  # Secondary inputs (add at story elaboration time):
  # - .factory/specs/behavioral-contracts/BC-2.11.001-query-mcp-tool.md (candidate anchor BC)
  # - crates/prism-query/src/engine.rs (current parse call sites)
cycle: "v1.0.0-greenfield"
---

# S-QUERY-GATE-REPARSE-CONSOLIDATION-001: Query Gate Re-Parse Consolidation

## Status: Draft Stub

> **This is a draft stub.** It was registered as the concrete deferral anchor for
> the "4x-query-reparse perf" item deferred from S-DEMO-FIDELITY-REMEDIATION-001
> (ADV-P208-P01-001 finding). The story body requires elaboration after PO authors
> the behavioral contract. All six context-engineering sections are present per
> story-writer rules; sections that require BC authorship are marked N/A with
> explicit rationale.

---

## Narrative

As a Prism developer optimizing query-engine throughput, I want the
`execute_inner` and `execute_scheduled_inner` functions in
`crates/prism-query/src/engine.rs` to parse the raw query string ONCE at the
entry point and thread the resulting `Ast` (or equivalent internal parse tree)
through all downstream availability gates — `check_table_availability` (E-QUERY-037),
`check_query_column_availability` (E-QUERY-038), `check_enrich_udf_availability`
(E-QUERY-039), and `run_materialization_pipeline` — so that each query execution
incurs a single parse pass rather than re-parsing the same query string at each
gate entry.

## Behavioral Contracts

| BC ID | Version | Title |
|-------|---------|-------|
| _(pending PO authorship)_ | — | Behavioral contract for the single-parse-pass invariant has not yet been authored. PO must author and anchor before `behavioral_contracts:` can be non-empty and status can advance to `ready`. |

## Acceptance Criteria

> N/A — pending PO authorship of the behavioral contract. ACs will be derived
> from BC postconditions once the BC is authored and anchored. Candidate AC:
>
> **AC-001**: For any well-formed query string Q that is executed via
> `execute_inner` or `execute_scheduled_inner`, the prism-query parser is invoked
> AT MOST ONCE on Q during a single execute call. The resulting `Ast` is reused
> by all downstream gates. (traces to pending BC postcondition N)

---

## Token Budget Estimate

> N/A — first story in maintenance epic; no predecessor story intelligence.
> Token budget to be established at story elaboration time when BC is authored.

| Component | Estimated Tokens |
|-----------|-----------------|
| Story spec (this file at elaboration) | ~6,000 |
| `crates/prism-query/src/engine.rs` (execute_inner + gates, ~1,200 lines) | ~18,000 |
| `crates/prism-query/src/table_registry.rs` | ~5,000 |
| BC files (1 BC, pending) | ~3,000 |
| Test files (Red Gate tests, pending) | ~3,000 |
| **Estimated total** | **~35,000 tokens (~14% of 256K context)** |

---

## Tasks

> N/A at stub stage. Tasks will be populated at story elaboration time, after BC
> authorship. Anticipated task outline:
>
> 1. Read `engine.rs` `execute_inner` call chain and identify all parse invocation
>    sites (search for `prismql_parser::parse` or equivalent).
> 2. Identify which gate function signatures accept a raw query string and re-parse
>    internally vs those that already accept a parsed AST.
> 3. Refactor entry point to parse once; thread `Ast` ref through gate functions.
> 4. Write Red Gate test asserting parse count = 1 per execute call.
> 5. Run `just check` — verify all 5000+ tests still pass.

---

## Previous Story Intelligence

N/A — first story registered against the query-gate-reparse-consolidation surface.

**Parent story context (S-DEMO-FIDELITY-REMEDIATION-001):** The 4-gate execute_inner
sequence was introduced / verified correct by S-DEMO-FIDELITY-REMEDIATION-001. That
story confirmed gate ORDER is correct (E-QUERY-037 → 038 → 039 → materialization).
This consolidation story does NOT change gate order or semantics; it only eliminates
the redundant re-parse overhead between gates.

---

## Architecture Compliance Rules

> Extracted from `architecture/module-decomposition.md` (SS-11 boundary) and ADR-022.

| Rule | Source | Implication |
|------|--------|-------------|
| SS-11 (prism-query) owns all plan-time gates | ARCH-INDEX Subsystem Registry | Refactor stays entirely within `crates/prism-query/` |
| Arc-DI plumbing: no placeholder-construct | ADR-022 §C | No new `Arc::new(...)` stub construction; thread parsed AST as a value (not a service) |
| No `unwrap()` / `expect()` on parse results in non-test code | CLAUDE.md Error taxonomy | Use `?` propagation; parse errors already surface as `PrismError::ParseError(E-QUERY-001)` |
| `just check` must pass after the refactor | CLAUDE.md Build & Test | All ~5000 workspace tests must remain green |

---

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `chumsky` | 0.12.0 | Per `Cargo.lock` (pinned); parser combinator used by prism-query's PrismQL parser |
| `datafusion` | as in `Cargo.lock` | Query execution context; no version change expected for this refactor |

> All versions pinned to workspace `Cargo.lock`. Do NOT use training-data versions;
> read `Cargo.lock` at implementation time to confirm exact versions.

---

## File Structure Requirements

> N/A at stub stage. Files to modify will be confirmed at elaboration time after
> a code-read pass to identify all parse call sites. Anticipated files:

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-query/src/engine.rs` | Modify | Consolidate parse call sites in `execute_inner` + `execute_scheduled_inner` |
| `crates/prism-query/src/table_registry.rs` | Possibly modify | If `check_table_availability` re-parses internally |
| `crates/prism-query/src/tests/bc_NNN_reparse_consolidation_test.rs` | Create | Red Gate test asserting parse count ≤ 1 per execute call |

---

## Architecture Component Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `execute_inner` / `execute_scheduled_inner` | `prism-query/src/engine.rs` | Effectful (async, I/O fan-out) |
| `check_table_availability` | `prism-query/src/engine.rs` | Pure (plan-time check against TableRegistry) |
| `check_query_column_availability` | `prism-query/src/engine.rs` | Pure (plan-time check against column map) |
| `check_enrich_udf_availability` | `prism-query/src/engine.rs` | Pure (plan-time check against InfusionRegistry) |
| `run_materialization_pipeline` | `prism-query/src/engine.rs` | Effectful (fan-out sensor I/O) |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Parse failure on the single parse pass | Returns `PrismError::ParseError` (E-QUERY-001) at entry; gates never reached |
| EC-002 | Gate returns error after parse succeeds | Parse result discarded; downstream gates not reached; same behavior as pre-consolidation |
| EC-003 | `execute_scheduled_inner` shares gate sequence | Must receive the same single-parse refactor as `execute_inner` |

---

## §Changelog

| Version | Burst | Date | Author | Summary |
|---------|-------|------|--------|---------|
| 1.0 | adv-p208-p01-001-deferral-anchor | 2026-06-29 | story-writer | **Draft stub created as concrete deferral anchor for ADV-P208-P01-001.** Registered to close the "follow-up story" Target gap in S-DEMO-FIDELITY-REMEDIATION-001 §Deferred/Out-of-Perimeter Items table, per Canonical Principle Rule 3 (deferred work must have a concrete story ID anchor, not "follow-up story"). Story body is a stub with N/A sections marked; full elaboration (BC authorship, ACs, tasks, Red Gate tests) required before status can advance to ready. Depends on S-DEMO-FIDELITY-REMEDIATION-001. |
