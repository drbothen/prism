# Demo Evidence Report — S-3.7.03

**Story:** Cyberint fixture generator — all 8 archetypes from 4 poller-express specs
**Branch:** feature/S-3.7.03
**Recorded:** 2026-04-28
**Tool:** VHS

## Coverage Summary

| Recording | Acceptance Criteria | BCs / VPs | Result |
|-----------|---------------------|-----------|--------|
| AC-001-all-35-tests-green | AC-001..007 (all 35 tests) | BC-3.4.001, BC-3.4.002, BC-3.4.004 / VP-108, VP-112–114, VP-119–120 | GREEN |
| AC-003-schema-drift-behavior | AC-003 (SchemaDrift archetype) | BC-3.4.002 / VP-113 | GREEN |

## Recordings

### Group 1 — All 35 tests GREEN (BC-3.4.001/002/004 + 8 archetypes + 4 endpoints)

**Tape:** `AC-001-all-35-tests-green.tape`
**GIF:** `AC-001-all-35-tests-green.gif` (331K)
**WEBM:** `AC-001-all-35-tests-green.webm` (542K)

Demonstrates: `cargo test -p prism-dtu-cyberint --features fixture-gen --test bc_3_4_cyberint_generator` — 35 tests covering all 8 archetypes (HealthyOtEnvironment, CompromisedEndpoint, LargeScale, DormantTenant, HighChurn, AuthOutage, SchemaDrift, PaginationEdgeCases) across all 4 API surfaces (alert, asm_asset, cve, ioc). Output shows `test result: ok. 35 passed; 0 failed`.

### Group 2 — SchemaDrift behavior (AC-003 / VP-113)

**Tape:** `AC-003-schema-drift-behavior.tape`
**GIF:** `AC-003-schema-drift-behavior.gif` (114K)
**WEBM:** `AC-003-schema-drift-behavior.webm` (125K)

Demonstrates: schema_drift-filtered test run with `--nocapture`. Shows that the `SchemaDrift` archetype marks `alert[0]` with `schema_valid: false` while non-alert surfaces (asm_asset, cve, ioc) remain valid. Three schema_drift tests pass: `schema_drift_alert_surface_index_0_marked_invalid`, `schema_drift_non_alert_surfaces_remain_valid`, `schema_drift_provenance_schema_valid_false`.

## Acceptance Criteria Coverage

| AC | Description | Covered By | Status |
|----|-------------|-----------|--------|
| AC-001 | 8 archetypes × 4 surfaces produce correct record counts | AC-001 recording (35 tests) | PASS |
| AC-002 | Each record validates against correct sub-spec | AC-001 recording | PASS |
| AC-003 | SchemaDrift: alert[0] invalid, others valid | AC-003 recording | PASS |
| AC-004 | Record IDs carry org-slug prefix per surface | AC-001 recording (VP-119/120 tests) | PASS |
| AC-005 | Two calls with identical inputs byte-identical | AC-001 recording (VP-108 test) | PASS |
| AC-006 | OrgA/OrgB ID sets disjoint across all surfaces | AC-001 recording | PASS |
| AC-007 | Schema validation absent from release build | AC-001 recording (VP-114 test) | PASS |
