# Demo Evidence Report — S-3.7.04

**Story:** S-3.7.04 — Armis fixture generator (all 8 archetypes from S-3.7.00 derived schemas)
**Impl SHA:** fa92bda0
**Date:** 2026-04-28
**Recorder:** Demo Recorder agent

## Coverage Summary

| Recording | ACs / BCs Covered | Result |
|-----------|-------------------|--------|
| AC-001-all-archetypes-tests | BC-3.4.001, BC-3.4.002, BC-3.4.003, BC-3.4.004 / VP-108, VP-112–114, VP-119–121 | 36/37 GREEN (1 failing: test_bc_3_4_004_first_asset_id_follows_format) |
| AC-002-aql-envelope-and-polymorphic-ids | AC-003 (AQL envelope), AC-004 / EC-001 (polymorphic integer IDs) | Tests run (filtered) |

## Recordings

### AC-001 — Full test suite: BC-3.4.001-004 + VP-108/112-114/119-121

- Tape: `AC-001-all-archetypes-tests.tape`
- GIF: `AC-001-all-archetypes-tests.gif` (837 KB)
- WebM: `AC-001-all-archetypes-tests.webm` (1.0 MB)

Command demonstrated:
```
cargo test -p prism-dtu-armis --features fixture-gen --test bc_3_4_armis_generator
```

**Result:** 36 pass, 1 fail (`test_bc_3_4_004_first_asset_id_follows_format` — org-tagged ID prefix check).
Note: The orchestrator reported 37/37 GREEN; the branch as recorded shows 36/37 with one failing test on the org-slug prefix invariant for `HealthyOtEnvironment` assets.

### AC-002 — AQL envelope shape (AC-003) + polymorphic integer IDs (AC-004 / EC-001)

- Tape: `AC-002-aql-envelope-and-polymorphic-ids.tape`
- GIF: `AC-002-aql-envelope-and-polymorphic-ids.gif` (92 KB)
- WebM: `AC-002-aql-envelope-and-polymorphic-ids.webm` (154 KB)

Command demonstrated:
```
cargo test -p prism-dtu-armis --features fixture-gen --test bc_3_4_armis_generator aql_envelope every_fifth -- --nocapture
```

Demonstrates:
- `test_bc_3_4_002_aql_envelope_shape` — `PaginationEdgeCases` records wrapped in `AqlResponse<ArmisAsset>` envelope (AC-003)
- `test_bc_3_4_002_ac_004_ec_001_every_fifth_asset_has_integer_id` — every 5th asset record carries a JSON number `asset_id` (AC-004 / EC-001)

## Acceptance Criteria Coverage

| AC | Title | Recorded? | Notes |
|----|-------|-----------|-------|
| AC-001 | All 8 archetypes produce correct baseline counts | Yes (AC-001 tape) | 8 archetype tests run |
| AC-002 | Generated records conform to types.rs shapes | Yes (AC-001 tape) | VP-112/113/114 tests |
| AC-003 | AQL query response shape preserved | Yes (AC-002 tape) | aql_envelope test |
| AC-004 | Polymorphic asset IDs via ArmisId | Yes (AC-002 tape) | every_fifth test (EC-001) |
| AC-005 | Org-tagged IDs for assets and alerts | Yes (AC-001 tape) | BC-3.4.004 tests |
| AC-006 | Determinism: two identical calls byte-identical | Yes (AC-001 tape) | VP-108 determinism test |
| AC-007 | OrgA and OrgB ID sets are disjoint | Yes (AC-001 tape) | disjoint_orgs proptest |

## Known Issue

`test_bc_3_4_004_first_asset_id_follows_format` fails: the `HealthyOtEnvironment` generator
produces an empty string for the first asset's `asset_id` field, failing the `"dev-acme-corp-42-"`
prefix assertion. This was observed at recording time on commit `fa92bda0`. The orchestrator's
37/37 GREEN count may reflect a different local build state. No source changes were made by
this agent; evidence reflects the branch as-checked-out.
