---
document_type: adversarial-review-pass
story_id: PLUGIN-MIGRATION-001-D
pass_number: 1
pass_scope: LOCAL-SPEC-LEVEL
pass_date: 2026-05-20
adversary_model: Claude Opus 4.7 (1M context, fresh)
streak_before: 0/3
streak_after: 0/3
findings_summary: "5 HIGH + 3 MED + 4 LOW + 2 OBS (14 total)"
---

# PLUGIN-MIGRATION-001-D Pass-1 Adversarial Review (LOCAL Spec-Level)

## Scope
- `.factory/stories/PLUGIN-MIGRATION-001-D-author-4-production-toml-sensor-specs.md` (819 lines, v1.0 draft)
- `.factory/specs/behavioral-contracts/BC-2.16.013-bundled-sensor-spec-dtu-parity.md` (265 lines, v1.0 draft)
- Cross-reference: BC-2.01.013, BC-2.01.016, BC-2.16.001, BC-2.16.002, BC-2.16.009, BC-2.16.012
- Cross-reference: TS-PLUGIN-PARITY-001 (verified extant — pre-flight observation resolved), VP-INDEX entry for VP-148/VP-PLUGIN-003
- Cross-reference: ADR-022, ADR-023, ARCH-INDEX, BC-INDEX v5.21, STORY-INDEX v2.157, capabilities.md, invariants.md, error-taxonomy.md
- Code parity surveys: `crates/prism-dtu-crowdstrike/src/clone.rs`, `crates/prism-spec-engine/src/pipeline.rs`, `crates/prism-spec-engine/src/auth_provider.rs`

## Methodology
Two-phase verification per POL-22:
- Phase A (lexical-vs-semantic): every BC ID, VP ID, ADR ID, file path, sensor name verified against source-of-truth artifact files
- Phase C (named entity existence): every cited function, struct, capability ID, holdout ID, error code, fixture path checked against codebase + specs
- POLICY-rubric pass on all 27 active policies; POL-7 H1 source-of-truth sweep across 7 referenced BCs; POL-21 phantom-anchor sweep on all §-citations; POL-22 fabricated-entity sweep; POL-25 propagation sweep on D-731/D-732 burst deltas.

## Findings

### HIGH

#### F-001 [HIGH] PolicyViolation:POL-22 — Fabricated DTU API surface (CrowdstrikeClone::new().start().await, server::spawn(), DtuHandle)
**Location:** Story spec AC-007 (line 316), AC-008 (line 333), AC-009 (line 348), AC-010 (line 365); BC-2.16.013 §Postconditions Postcondition 2 (line 126).
**Evidence:** Actual API in `crates/prism-dtu-crowdstrike/src/clone.rs:92`: `BehavioralClone::start_on(&mut self, bind: SocketAddr, shutdown: Option<broadcast::Receiver<()>>, tls: Option<...>) -> anyhow::Result<SocketAddr>`. No `start()` method. No `server` module exists. No `DtuHandle` type exists.
**Narrative:** Story + BC prescribe a DTU bring-up API that does not exist. Implementer following spec would write code that does not compile.
**Policy:** POL-22 (Phase C), POL-4, POL-5.
**Routing:** product-owner (BC-2.16.013) + story-writer (AC-007..AC-010).

#### F-002 [HIGH] PolicyViolation:POL-22 — `PipelineExecutor::execute(spec, "<table_name>", &NullAuthProvider, ...)` signature wrong
**Location:** Story AC-007 line 318, AC-008 line 334, AC-009 line 349, AC-010 lines 366+368; BC-2.16.013 line 129.
**Evidence:** Actual signature `crates/prism-spec-engine/src/pipeline.rs:136`: `execute(spec: &SensorSpec, table: &TableSpec, context: &FetchContext, http_client: &reqwest::Client, auth_provider: &dyn AuthProvider)`. 5 args, not 3; 2nd arg is `&TableSpec` not string.
**Policy:** POL-22, POL-4, POL-25.
**Routing:** product-owner + story-writer.

#### F-003 [HIGH] PolicyViolation:POL-22, POL-1, POL-25 — Six holdout scenarios HS-MIGRATION-D-001..006 declared without artifacts; naming breaks HS-NNN convention
**Location:** Story frontmatter lines 86-92.
**Evidence:** No file named `HS-MIGRATION-D-*` exists anywhere. Existing convention: 3-digit sequential `HS-NNN`.
**Policy:** POL-22 (Phase C), POL-1 (append-only numbering), POL-25.
**Routing:** product-owner (author 6 HS files in sequential numbering OR remove array).

#### F-004 [HIGH] PolicyViolation:POL-22 Phase C, POL-1, POL-24 — E-SPEC-015 + E-SPEC-016 cited in BC-2.16.013 without error-taxonomy.md registration
**Location:** BC-2.16.013 §Error Conditions lines 194-195; story line 591.
**Evidence:** error-taxonomy.md ends at E-SPEC-014. Story explicitly defers: "implementer: confirm the exact error code".
**Policy:** POL-22, POL-1, POL-24, POL-25, CLAUDE.md Rule 6.
**Routing:** product-owner (register codes OR rewrite to reuse E-SPEC-009).

#### F-005 [HIGH] PolicyViolation:POL-7 — Systematic BC title truncation in story body BC table (6 of 7 BCs)
**Location:** Story §Behavioral Contracts table, lines 184-190.
**Evidence:** 6 of 7 BC titles in story body table are truncated versions of the canonical BC H1.
**Policy:** POL-7 (HIGH), POL-23, POL-25.
**Routing:** story-writer.

### MED

#### F-006 [MED] PolicyViolation:POL-21 — Phantom section anchor: `ADR-023 §Rule 1`, `§Rule 3`
**Location:** BC-2.16.013 lines 231, 232, 253, 260; story line 604.
**Evidence:** ADR-023 has `### Decision Rules` containing bold-paragraph "Rule N" subsections — not `##` headings per POL-21 strict reading.
**Policy:** POL-21.
**Routing:** product-owner (rewrite cite to `§Decision Rules — Rule N`).

#### F-007 [MED] PolicyViolation:POL-21 — Phantom section anchor: `ADR-022 §C2`
**Location:** BC-2.16.013 line 234.
**Evidence:** ADR-022 has §A..§G — no `§C2` subdivision exists.
**Policy:** POL-21.
**Routing:** product-owner.

#### F-008 [MED] PolicyViolation:POL-6, POL-4 — Frontmatter comment "Sensor Adapter Layer" vs canonical "Sensor Adapters"
**Location:** Story spec frontmatter line 24.
**Evidence:** ARCH-INDEX canonical: SS-01 = "Sensor Adapters". Story uses "Sensor Adapter Layer".
**Policy:** POL-6, POL-4.
**Routing:** story-writer.

### LOW

#### F-009 [LOW] PolicyViolation:POL-21 — `BC-2.16.002 §Canonical Structured Event Catalog` phantom anchor (story line 683)
**Location:** Story lines 573, 683.
**Evidence:** "Canonical Structured Event Catalog" is bold-bullet inside BC-2.16.002 §Postconditions, not a `###` heading.
**Routing:** story-writer (fix line 683 to include parent context).

#### F-010 [LOW] [process-gap] PolicyViolation:POL-21 — `capabilities.md §CAP-029` cited; capabilities.md is flat-table with no `##` per-CAP headings
**Routing:** process-gap — architect amendment OR POL-21 clarification at cycle-close.

#### F-011 [LOW] PolicyViolation:POL-22 Phase C — AC-006 trace cites BC-2.16.001 postconditions by numeric index of unnumbered bullets
**Location:** Story AC-006 line 310.
**Routing:** story-writer (replace positional cite with named-section cite).

#### F-012 [LOW] [process-gap] PolicyViolation:POL-20 — BC-2.16.013 `introduced: "2026-05-20"` not in canonical `cycle-N`/`wave-N` format
**Routing:** process-gap — policies-steward at cycle-close.

### OBS

#### O-001 [OBS — escalated to HIGH under production-grade lens by orchestrator] — "Implementer must verify TOML grammar" deferral in story Risk #1 + Task 1
**Location:** Story lines 466, 723-729.
**Narrative:** Per CLAUDE.md Canonical Principle Rule 6 and Standing Rule 3 §1, this deferral pattern is a production-grade-default violation.
**Routing:** product-owner (perform grammar verification now).

#### O-002 [OBS] [process-gap] — VP-148 has no standalone file; only VP-INDEX row (sibling pattern across VP-PLUGIN-NNN)
**Routing:** process-gap candidate; architect at cycle-close.

## Verdict
**BLOCKED-soft.** 5 HIGH + 3 MED + 4 LOW + 2 OBS = 14 findings. Streak 0/3.

## Routing Summary
- product-owner: F-001, F-002 (BC), F-003, F-004, F-006, F-007, O-001 → closed in FB-IMPL-P1-PO burst
- story-writer: F-001 (4 ACs), F-002 (4 ACs), F-005, F-008, F-009, F-011 → closed in FB-IMPL-P1-SW burst
- process-gap (architect / policies-steward / cycle-close): F-010, F-012, O-002

## Novelty Assessment
**Novelty: HIGH.** F-001 + F-002 are concrete code-grounded Phase-C gaps unique to PLUGIN-MIGRATION-001-D (first Wave-1 spec story prescribing test-harness code against DTU API). F-003 (HS naming) is novel — PREREQ stories used HS-NNN sequential. F-004 (unregistered E-SPEC codes) regresses propagation discipline from PREREQ-E (where E-SPEC-013/014 were authored alongside taxonomy rows in burst). F-005 (BC title truncation) is recurrence of BC Title Sync axis with 6/7 systematic threshold.

## Streak Update
- streak_before: 0/3
- streak_after: 0/3 (BLOCKED-soft; pass-1 not CLEAN)
- next_action: fix-burst FB-IMPL-P1 (PO + story-writer) → pass-2 with fresh-context adversary
