---
document_type: fix-burst-closure
story_id: PLUGIN-MIGRATION-001-D
fix_burst_number: 2
pass_addressed: 2
closure_date: 2026-05-20
closure_decision: D-734
streak_status: 0/3 (unchanged; awaiting pass-3 fresh-context verification)
findings_closed: 8 in-scope + 2 OBS noted
findings_deferred_code_side_techdebt: 1 (Cyberint auth_type_name label-vs-behavior inconsistency)
---

# PLUGIN-MIGRATION-001-D Fix-Burst-2 Closure

## Findings Closure Status

### HIGH (3/3 closed in-scope)

| Finding | Closure | Owner | Evidence |
|---|---|---|---|
| F-001 (auth_type swap) | CLOSED | product-owner + story-writer | BC-2.16.013 v1.2: cyberint=bearer_static, claroty=cookie_roundtrip. Story v1.2 propagated AC-002/003/004/011/Task descriptions/file-list. HS-014/HS-015 corpus updated. Code-grounded against cyberint.rs:57-59 + claroty.rs:63-65 + mod.rs:13-14. |
| F-002 (E-SPEC-009 phantom semantics) | CLOSED | product-owner | New E-SPEC-017 registered in error-taxonomy.md v1.41 for filename-stem mismatch. BC-2.16.001 v1.4 §Error Conditions amended. BC-2.16.013 §Error Conditions corrected. HS-018 + story AC-001 + RG-09 updated. POL-1 append-only honored (E-SPEC-015/016 tombstone rows added). |
| F-003 (fetch_page phantom) | CLOSED | product-owner + story-writer | BC-2.16.013 + HS-013 + story line 562 corrected to `<SensorAdapter as fetch>(...)` (the SensorAdapter trait method at line 391 of crowdstrike.rs). Verified all 4 sensors implement SensorAdapter::fetch as the canonical entry point. |

### MED (3/3 closed in-scope)

| Finding | Closure | Owner | Evidence |
|---|---|---|---|
| F-004 (${query.aql} in BC line 276) | CLOSED | product-owner | BC-2.16.013 v1.2 line 276 corrected to `${query.filter.aql}`. |
| F-005 (line-number citations) | CLOSED | product-owner + story-writer | BC-2.16.013 §Preconditions O-001 + story Task 1 lines 524-525 now cite `FetchStep::fan_out_batch_size field` and `PipelineExecutor::execute_impl query.filter.{key} step_vars seeding` per TD-VSDD-091. |
| F-006 (epic_id divergence) | CLOSED | product-owner | 6 HS files aligned on `epic_id: "PLUGIN-MIGRATION-001"`. |

### LOW (2/2 closed in-scope; adjacent BC amendment)

| Finding | Closure | Owner | Evidence |
|---|---|---|---|
| F-007 (BC-2.16.009 E-SPEC-002/003 enumeration) | CLOSED | product-owner | BC-2.16.009 v1.4 §Error Conditions adds E-SPEC-002 + E-SPEC-003 rows. |
| F-008 (4 vs 5 auth_type set) | CLOSED | product-owner + story-writer | BC-2.16.009 v1.4 §Validation Rules 1 lists 5 canonical values; story AC-011 line 449 updated to match. |

### OBS

| Finding | Status |
|---|---|
| O-001 (pass-1 vs pass-2 novelty) | Noted — confirms fresh-context principle |
| O-002 (VP-148 indexed-only-no-file) | Deferred to cycle-close process-gap (per FB-IMPL-P1 record) |

## Code-Side Tech-Debt Surfaced (Out-of-Scope for This Story)

**Cyberint `auth_type_name()` label-vs-behavior inconsistency:**
- `cyberint.rs:8` file header documents "Cookie-based auth: POST /login → Set-Cookie session cookie"
- `cyberint.rs:57-59` `auth_type_name()` returns `"bearer_static"`
- Either rename `auth_type_name()` to match behavior OR correct the file header
- **Recommendation:** orchestrator attaches as follow-up tech-debt or sub-story for architect+implementer adjudication at PLUGIN-MIGRATION-001-D cycle-close

## Files Changed in FB-IMPL-P2

**PO (12 files):** BC-2.16.013 v1.2, BC-2.16.001 v1.4, BC-2.16.009 v1.4, error-taxonomy.md v1.41, BC-INDEX v5.23, HOLDOUT-INDEX, HS-013..018 (6 files).

**Story-writer (2 files):** PLUGIN-MIGRATION-001-D story v1.2, STORY-INDEX v2.159.

**State-manager (this D-734 burst):** local-pass-2.md, PLUGIN-MIGRATION-001-D-fix-burst-2.md, STATE.md v7.420 → v7.421.

## Next Action

Dispatch fresh-context adversary for pass-3 (streak target 0/3 → 1/3).
