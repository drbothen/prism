---
document_type: gate-step-report
gate_step: e
gate_step_name: consistency-validation
cycle: phase-3-dtu-wave-2
gate: wave-2-integration-gate
scope: 11 Wave 2 stories + indexes
reviewer: vsdd-factory:consistency-validator
date: 2026-04-26
verdict: CONDITIONAL_FAIL
total_items: 16
critical: 1
high: 1_FAIL_plus_5_CLEAN
medium: 8_CLEAN
low: 2_ACCEPTABLE
---

# Wave 2 Integration Gate — Gate Step E: Consistency Validation

**Scope:** 11 Wave 2 stories + STORY-INDEX + BC-INDEX + VP-INDEX + ARCH-INDEX
**Reviewer:** vsdd-factory:consistency-validator
**Date:** 2026-04-26
**Verdict:** CONDITIONAL FAIL — 2 blocking items (WGCV-W2-001 CRITICAL + WGCV-W2-002 HIGH FAIL)

---

## CRITICAL Finding (Blocking)

### WGCV-W2-001 (CRITICAL): All 11 Wave 2 Story Files Have `status: draft` — STORY-INDEX Shows `MERGED`

**Files affected:**
- `S-2.01` frontmatter line 10: `status: draft`
- `S-2.02` frontmatter line 10: `status: draft`
- `S-2.03` frontmatter line 10: `status: draft`
- `S-2.04` frontmatter line 10: `status: draft`
- `S-2.05` frontmatter line 10: `status: draft`
- `S-2.06` frontmatter line 10: `status: draft`
- `S-2.07` frontmatter line 10: `status: draft`
- `S-2.08` frontmatter line 10: `status: draft`
- `S-6.11` frontmatter line 13: `status: draft`
- `S-6.12` frontmatter line 13: `status: draft`
- `S-6.13` frontmatter line 13: `status: draft`

**STORY-INDEX status:** v1.53 records all 11 stories as merged with PR# + SHA + `+Nt` test annotations.

**VSDD convention:** Story frontmatter is the authoritative per-story record. `status: draft` means "not implemented." All 11 stories are implemented and merged; the frontmatter has not been updated to reflect this.

**Recommendation:** Update all 11 story files `status: draft` → `status: merged`. This is a pure factory-artifacts update (state-manager scope). No source code changes required. File as W2-FIX-G.

---

## HIGH Finding (Blocking)

### WGCV-W2-002 (HIGH FAIL): S-2.01 Lacks MERGED Status Annotation in STORY-INDEX

**Evidence:**
- S-2.01 title cell (STORY-INDEX v1.53 line 137) has no `[MERGED ...]` bracket annotation.
- S-2.02 and S-2.03 (which depend on S-2.01) ARE correctly marked with `[MERGED #PR (SHA)]` annotations.
- No changelog entry for S-2.01 merge exists in STORY-INDEX changelog in the v1.44–v1.53 range.

**Recommendation:** Verify the S-2.01 PR number and SHA (PR #43, 0d24ab79 per STATE.md), add `[MERGED #43 (0d24ab79)]` annotation to the S-2.01 title cell, update S-2.01 frontmatter `status: draft` → `status: merged` (covered by WGCV-W2-001), and add a v1.54 changelog entry. Bundle with W2-FIX-G.

---

## HIGH Items — CLEAN (Non-Blocking)

### WGCV-W2-003 (HIGH but CLEAN): S-2.08 `behavioral_contracts: []` Verified Intentional

S-2.08 (`event-tables`) has `behavioral_contracts: []` in frontmatter. Validated: S-2.08 is the table-dispatch routing story that creates the prism-query scaffolding. The behavioral contracts are anchored in the stories that implement the query-execution logic (S-3.02 et al.). Empty BC list is intentional per VSDD convention for routing/scaffolding stories.

### WGCV-W2-004 (HIGH but CLEAN): Dependency Graph S-2.01 → S-2.02/S-2.03 Bidirectionally Consistent

`dependency-graph.md` and STORY-INDEX `dependencies:` fields correctly reflect S-2.01 as a prerequisite for S-2.02 and S-2.03. No orphaned edges.

### WGCV-W2-005 (HIGH but CLEAN): `TableType` Canonical Home (prism-core) Verified

D-026 mandated `TableType` live in `prism-core`. Verified: `prism-core/src/table_type.rs` defines the enum; `prism-spec-engine` and `prism-sensors` import via `pub use prism_core::TableType`. No duplicate definitions found.

### WGCV-W2-006 (HIGH but CLEAN): `SensorQueryDescriptor` vs `InternalTableDescriptor` Distinction Verified

D-025 established these as semantically distinct types in separate crates. Verified: `InternalTableDescriptor` in `prism-core` (S-2.03 scope) and `SensorQueryDescriptor` in `prism-query` (S-2.08 scope). No conflation in story frontmatter or architecture docs.

### WGCV-W2-007 (MEDIUM): `RouteDecision` in prism-sensors but Consumed by prism-query — Dependency Undocumented

`RouteDecision` is defined in `crates/prism-sensors/src/lib.rs` but is consumed by the query routing logic in `prism-query`. This creates an undocumented Cargo.toml dependency: S-3.02 (which implements query routing) will need to add `prism-sensors` to `prism-query/Cargo.toml`. This dependency is not currently documented in the S-3.02 spec or in `dependency-graph.md`.

**Non-blocking observation.** File as TD-W2-CONS-001 for S-3.02 spec pre-work.

---

## MEDIUM Items — CLEAN (Non-Blocking)

### WGCV-W2-008 (HIGH but CLEAN): AC-5a/AC-5b Split Consistent

AC-5a (routing PASS) and AC-5b (deferred to Wave 3 query story) split is consistently represented across S-2.08 v1.9, S-3.02 v1.7, and STORY-INDEX v1.53. D-030 decision correctly propagated.

### WGCV-W2-009 (MEDIUM but CLEAN): Token Budget BC Count Arithmetic Correct

BC count arithmetic verified for S-2.01/2.02/2.04/2.06/2.07. No double-counting. BC-INDEX total 200 consistent with per-story BC tables.

### WGCV-W2-010 (MEDIUM but CLEAN): VP-INDEX Arithmetic Verified

Total 62 VPs = 26 + 28 + 6 + 2 = 43 P0 + 19 P1. Row-by-row check passes. VP-INDEX v1.11 consistent with story BC tables.

### WGCV-W2-011 (MEDIUM but CLEAN): VP-INDEX → Coverage Matrix Per-Module Mapping Consistent

VP-INDEX coverage-matrix per-module columns match the ARCH-INDEX subsystem names. No orphaned VP rows.

### WGCV-W2-012 (MEDIUM but CLEAN): Wave 2 BC Traceability Matrix Matches Story Frontmatter

Traceability matrix (when present) correctly cross-references story-level BCs to VP-INDEX entries. No orphaned BC-to-VP mappings.

### WGCV-W2-013 (MEDIUM but CLEAN): Subsystem Naming Matches ARCH-INDEX Canonical Names

Verified: SS-15 Storage Layer, SS-05 Audit Trail, SS-01 Sensor Adapters, SS-16 Spec Engine, SS-18 Action Delivery Engine — all correctly referenced in story frontmatter `subsystem:` fields.

### WGCV-W2-014 (MEDIUM, NO FINDING): AuditRiskLevel Cross-Story Usage

`AuditRiskLevel` used in S-2.04/2.05/2.06 — no inconsistency in value semantics across stories. D-017 decision (RiskTier→AuditRiskLevel new type) correctly propagated.

---

## LOW Items — Acceptable

### WGCV-W2-015 (LOW, ACCEPTABLE): S-3.02 STORY-INDEX Title Annotation

S-3.02 (Wave 3 query engine story) has a brief title annotation in STORY-INDEX. The annotation is informative rather than authoritative; does not require correction.

### WGCV-W2-016 (LOW, ACCEPTABLE): S-2.08 STORY-INDEX Title Cell Density

S-2.08 title cell is denser than surrounding rows due to multi-point annotations (v1.4→v1.5→v1.6 reconciliation notes). Acceptable; STORY-INDEX density is a style choice not a correctness issue.

---

## Summary

| Severity | Count | Disposition |
|----------|-------|-------------|
| CRITICAL | 1 | WGCV-W2-001 — BLOCKING: 11 story files `status: draft` → must update to `merged` |
| HIGH FAIL | 1 | WGCV-W2-002 — BLOCKING: S-2.01 annotation gap in STORY-INDEX |
| HIGH CLEAN | 5 | WGCV-W2-003..007 — Non-blocking; validated as intentional or consistent |
| MEDIUM CLEAN | 7 | WGCV-W2-008..014 — All consistent; no action required |
| LOW ACCEPTABLE | 2 | WGCV-W2-015..016 — Style; no action required |

**Path to PASS:** Dispatch W2-FIX-G (state-manager only — pure factory-artifacts update):
1. Update all 11 story files: `status: draft` → `status: merged` (closes WGCV-W2-001)
2. Add S-2.01 MERGED annotation to STORY-INDEX (closes WGCV-W2-002)
3. Bump STORY-INDEX to v1.54 with changelog entry
4. State-manager commits to factory-artifacts branch
