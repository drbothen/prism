---
document_type: fix-burst-closure
story_id: PLUGIN-MIGRATION-001-D
fix_burst_number: 1
pass_addressed: 1
closure_date: 2026-05-20
closure_decision: D-733
streak_status: 0/3 (unchanged; awaiting pass-2 fresh-context verification)
findings_closed: 14
findings_deferred_process_gap: 3 (F-010, F-012, O-002)
---

# PLUGIN-MIGRATION-001-D Fix-Burst-1 Closure

## Findings Closure Status

### HIGH (5/5 closed in-scope)

| Finding | Closure Status | Owner | Closure Evidence |
|---|---|---|---|
| F-001 (DTU API fabrication) | CLOSED | product-owner + story-writer | BC-2.16.013 v1.1 §Postconditions §2 documents real `BehavioralClone::start_on(bind, shutdown, tls)` API; story AC-007..AC-010 rewritten to cite real API |
| F-002 (PipelineExecutor signature) | CLOSED | product-owner + story-writer | BC-2.16.013 v1.1 cites 5-arg `(spec, table, ctx, http_client, auth_provider)` signature; 4 ACs rewritten to construct FetchContext + http_client + 30s timeout |
| F-003 (HS-MIGRATION-D-NNN) | CLOSED via Option A | product-owner | 6 HS files created HS-013..HS-018 in sequential numbering; HOLDOUT-INDEX v1.3→v1.4; story frontmatter updated to [HS-013..HS-018] |
| F-004 (E-SPEC-015/016) | CLOSED via mixed approach | product-owner | E-SPEC-015 RETIRED (parity FAIL is test verdict, not runtime code); E-SPEC-016 → E-SPEC-009 reuse (existing canonical for sensor_id ≠ filename mismatch); error-taxonomy.md UNCHANGED (no new codes needed); BC-2.16.013 v1.1 §Error Conditions rewritten |
| F-005 (BC title truncation) | CLOSED | story-writer | All 7 BC titles in story §Behavioral Contracts table now match canonical H1 (minus `BC-NNN:` prefix) |

### MED (3/3 closed in-scope)

| Finding | Closure Status | Owner | Closure Evidence |
|---|---|---|---|
| F-006 (ADR-023 §Rule 1/3 phantom) | CLOSED | product-owner | BC-2.16.013 v1.1 cites `ADR-023 §Decision Rules — Rule 1` / `§Decision Rules — Rule 3` |
| F-007 (ADR-022 §C2 phantom) | CLOSED | product-owner | BC-2.16.013 v1.1 cites `ADR-022 §C — Wiring Contracts — QueryEngine` |
| F-008 (Sensor Adapter Layer comment) | CLOSED | story-writer | Story frontmatter comment line 24: `Sensor Adapter Layer` → `Sensor Adapters` |

### LOW (2/4 closed in-scope; 2/4 deferred as process-gap)

| Finding | Closure Status | Owner | Closure Evidence |
|---|---|---|---|
| F-009 (BC-2.16.002 §Catalog anchor) | CLOSED | story-writer | Story line 683 fixed: `§Canonical Structured Event Catalog` → `§Postconditions Canonical Structured Event Catalog` |
| F-010 (capabilities.md flat-table) | DEFERRED [process-gap] | architect (cycle-close) | Project-wide structural pattern; not per-story defect |
| F-011 (AC-006 positional cite) | CLOSED | story-writer | AC-006 trace replaced with `§Postconditions + §Table Registration with DataFusion + §Auth Type Resolution` |
| F-012 (BC introduced date format) | DEFERRED [process-gap] | policies-steward (cycle-close) | Project-wide pattern across PREREQ-D/E BCs |

### OBS (1/2 closed; 1/2 deferred as process-gap)

| Finding | Closure Status | Owner | Closure Evidence |
|---|---|---|---|
| O-001 (grammar-verification deferral) | CLOSED — orchestrator escalated to HIGH under production-grade lens | product-owner | Grammar verified per-field: `fan_out_batch_size` SUPPORTED, `${query.filter.aql}` SUPPORTED (cite corrected from `${query.aql}`), `timestamp_format = "multi"` + `timestamp_fallback_chain` NOT SUPPORTED with explicit implementer Option A (grammar extension) / Option B (WASM plugin) documented in-scope |
| O-002 (VP-148 file absence) | DEFERRED [process-gap] | architect (cycle-close) | VP-INDEX-row-only pattern across VP-PLUGIN-NNN siblings; needs policy-level adjudication |

## Files Changed in FB-IMPL-P1

**PO scope (FB-IMPL-P1-PO):**
- BC-2.16.013-bundled-sensor-spec-dtu-parity.md (v1.0 → v1.1)
- BC-INDEX.md (v5.21 → v5.22)
- HOLDOUT-INDEX.md (v1.3 → v1.4)
- 6 NEW: HS-013..HS-018

**Story-writer scope (FB-IMPL-P1-SW):**
- PLUGIN-MIGRATION-001-D-author-4-production-toml-sensor-specs.md (v1.0 → v1.1)
- STORY-INDEX.md (v2.157 → v2.158)

**State-manager scope (this D-733 burst):**
- code-delivery/PLUGIN-MIGRATION-001-D/adversarial-review/local-pass-1.md (NEW)
- code-delivery/PLUGIN-MIGRATION-001-D/adversarial-review/PLUGIN-MIGRATION-001-D-fix-burst-1.md (NEW)
- STATE.md (v7.419 → v7.420)

## Process-Gap Items Forwarded to Cycle-Close

1. F-010 — capabilities.md flat-table structure: no `§CAP-NNN` headings — adjudicate at cycle-close (architect amendment OR POL-21 clarification)
2. F-012 — BC `introduced:` ISO date format vs POL-20 canonical — adjudicate at cycle-close (policies-steward normalization OR POL-20 amendment)
3. O-002 — VP-PLUGIN-NNN indexed-only-no-file pattern — adjudicate at cycle-close (architect VP file authoring OR POL-9 amendment for `method: integration_test` VPs)

## Next Action

Dispatch fresh-context adversary for pass-2 with target streak 1/3.
