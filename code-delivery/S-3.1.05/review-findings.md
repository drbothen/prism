# Review Findings — S-3.1.05

**PR:** #98 — feat(S-3.1.05): prism-spec-engine OrgId-scoped sensor specs
**BC Anchor:** BC-3.1.001
**Date:** 2026-04-29

## Convergence Tracking

| Cycle | Findings | Blocking | Fixed | Remaining |
|-------|----------|----------|-------|-----------|
| 1 | 1 | 0 | 0 | 0 → APPROVE |

**Result: APPROVED in 1 cycle (0 blocking findings)**

## Finding Detail

### F-001 — RegistryNotInitialized variant is dead code (suggestion)

| Field | Value |
|-------|-------|
| Severity | suggestion |
| Category | code-quality |
| Blocking | NO |
| File | `crates/prism-spec-engine/src/error.rs` + `org_scoped_store.rs` |
| Status | open / deferred |

**Description:** `SpecEngineError::RegistryNotInitialized` is declared in `error.rs` and documented in the module-level contract comment but `get_spec` never emits it. An empty pre-startup registry causes `OrgRegistry::resolve` to return `None`, which maps to `UnknownOrg`. The test accepts either variant and the no-panic invariant is satisfied.

**Recommendation:** Either annotate with `#[allow(dead_code)]` + a forward-reference comment, or remove the variant. Defer to S-3.1.06 or log as tech debt.

**Routed to:** Deferred — not blocking merge.

## Traceability Verification

| AC | Test | BC Anchor | Status |
|----|------|-----------|--------|
| AC-1 (slug→OrgId, UnknownOrg) | test_BC_3_1_001_get_spec_resolves_slug_to_org_id | BC-3.1.001 post-1 | PASS |
| AC-1 (UnknownOrg on miss) | test_BC_3_1_001_get_spec_unknown_org_returns_error | BC-3.1.001 EC-001 | PASS |
| AC-2 (resolve once, OrgId only) | test_BC_3_1_001_two_orgs_same_sensor_name_no_collision | BC-3.1.001 inv-1 | PASS |
| AC-3 (no panic on uninit) | test_BC_3_1_001_empty_registry_returns_err_not_panic | BC-3.1.001 inv-3 | PASS |
| AC-4 (UUID isolation) | test_BC_3_1_001_cross_org_spec_isolation | BC-3.1.001 post-3 | PASS |
| EC-002 (known org, missing sensor) | test_BC_3_1_001_known_org_missing_sensor_returns_sensor_not_found | BC-3.1.001 EC-002 | PASS |
| EC-003 (rename stability) | test_BC_3_1_001_org_rename_preserves_spec_access | ADR-006 §4 | PASS |
