# Evidence Report: OCSF-CLASS-MIGRATION-001

**Story:** OCSF-CLASS-MIGRATION-001 — prism-ocsf + sensor TOMLs: Migrate ocsf_class security_finding → detection_finding (OCSF v1.1 deprecation)
**Version:** 1.9
**LOCAL Adversary Cascade:** Converged 3/3 (per-story-delivery step complete)
**Branch:** feature/OCSF-CLASS-MIGRATION-001
**Date:** 2026-06-06

---

## Coverage Summary

| AC | Description | Evidence File | Evidence Type | Status |
|----|-------------|---------------|---------------|--------|
| AC-001 | All 4 production sensor TOMLs declare `ocsf_class = "detection_finding"`; `rg 'ocsf_class.*security_finding'` returns zero results | `AC-001-no-production-toml-uses-security-finding.txt` | grep audit + Red Gate test | PASS |
| AC-002 | `select_by_class_name("detection_finding")` returns `Ok(2004)`; no `ocsf.deprecated_class_alias` WARN emitted | `AC-002-detection-finding-returns-2004-no-warn.txt` | Red Gate test | PASS |
| AC-003 | `select_by_class_name("security_finding")` returns `Ok(2004)` (NOT 2001) with `ocsf.deprecated_class_alias` WARN | `AC-003-security-finding-returns-2004-with-warn.txt` | Red Gate test | PASS |
| AC-004 | `EventClassSelector::select()` path returns class_uid 2001 for ZERO tokens (INV-NO-2001-SELECT-PATH) | `AC-004-select-path-no-token-returns-2001.txt` | Red Gate test | PASS |
| AC-005 | No test in workspace asserts `class_uid == 2001` for any production sensor record; `rg '2001'` audit confirms | `AC-005-no-stale-2001-assertions-in-workspace.txt` | grep audit + Red Gate test | PASS |

---

## Red Gate Tests (5 required, 5 pass)

| Test Name | AC | Crate | Module | Result |
|-----------|----|----|-------|--------|
| `test_BC_2_02_012_no_production_toml_uses_security_finding` | AC-001 | prism-ocsf | `tests::bc_2_02_012_class_selector::ocsf_migration_red_gate` | PASS |
| `test_BC_2_02_012_select_by_class_name_detection_finding_returns_2004_no_warn` | AC-002 | prism-ocsf | `tests::bc_2_02_012_class_selector::ocsf_migration_red_gate` | PASS |
| `test_BC_2_02_012_select_by_class_name_security_finding_returns_2004_with_warn` | AC-003 | prism-ocsf | `tests::bc_2_02_012_class_selector::ocsf_migration_red_gate` | PASS |
| `test_BC_2_02_012_select_path_no_token_returns_2001` | AC-004 | prism-ocsf | `tests::bc_2_02_012_class_selector::ocsf_migration_red_gate` | PASS |
| `test_BC_2_02_012_no_stale_2001_assertions_in_workspace` | AC-005 | prism-ocsf | `tests::bc_2_02_012_class_selector::ocsf_migration_red_gate` | PASS |

Full crate run: **65 tests run, 65 passed, 1 skipped** (skipped test is unrelated infrastructure test).

---

## VHS Recording

| File | Content | ACs Covered |
|------|---------|-------------|
| `AC-ALL-ocsf-class-migration-red-gate-suite.gif` | Terminal recording of all 5 Red Gate tests passing in sequence, preceded by the `rg` grep audits for AC-001 | AC-001 through AC-005 |
| `AC-ALL-ocsf-class-migration-red-gate-suite.webm` | Same recording, archival WebM format | AC-001 through AC-005 |
| `AC-ALL-ocsf-class-migration-red-gate-suite.tape` | VHS tape script source | — |

---

## AC-001: TOML Migration Evidence

**BC Anchor:** BC-2.02.012 v1.6 — `INV-PRODUCTION-TOML-NO-SECURITY-FINDING` + `TV-BC-2.02.012-009`

Grep audit confirms zero production TOMLs declare `ocsf_class = "security_finding"`:

```
$ rg 'ocsf_class.*security_finding' crates/prism-sensors/specs/
(no output — exit code 1 = zero matches)
```

All 4 production TOMLs declare `ocsf_class = "detection_finding"`:

```
crates/prism-sensors/specs/claroty.sensor.toml:ocsf_class = "detection_finding"
crates/prism-sensors/specs/crowdstrike.sensor.toml:ocsf_class = "detection_finding"
crates/prism-sensors/specs/cyberint.sensor.toml:ocsf_class = "detection_finding"
crates/prism-sensors/specs/armis.sensor.toml:ocsf_class = "detection_finding"
```

---

## AC-002: `detection_finding` → 2004 (PRIMARY, no WARN)

**BC Anchor:** BC-2.02.012 v1.6 `TV-BC-2.02.012-007`; BC-2.01.013 v1.14 OCSF Conformance Clause

`select_by_class_name("detection_finding")` returns `Ok(2004)`. The `"detection_finding"` match arm is the canonical PRIMARY entry — no `ocsf.deprecated_class_alias` WARN is emitted.

---

## AC-003: `security_finding` → 2004 (transitional alias, WARN emitted)

**BC Anchor:** BC-2.02.012 v1.6 `TV-BC-2.02.012-008`; BC-2.01.013 v1.14 `TV-BC-2.01.013-005`

`select_by_class_name("security_finding")` returns `Ok(2004)` (NOT the deprecated `Ok(2001)`).
Option A (D-989 PO decision, 2026-06-03): transitional alias kept working with deprecation warning to support external TOMLs not under Prism control.

The implementation emits exactly:
```rust
tracing::warn!(
    event_type = "ocsf.deprecated_class_alias",
    class_name = "security_finding",
    resolved_class_uid = 2004,
    "sensor TOML uses deprecated ocsf_class value 'security_finding'; update to 'detection_finding'"
)
```

SAP-1 verification: `ocsf.deprecated_class_alias` catalogued in BC-2.16.002 Structured Event Catalog row 134 (Wave-5 Phase-A). No new catalog row required.

---

## AC-004: `select()` path — INV-NO-2001-SELECT-PATH

**BC Anchor:** BC-2.02.012 v1.6 `INV-NO-2001-SELECT-PATH`

The `EventClassSelector::select(sensor_id, record_type)` function — the record-type-token dispatch path — never returned 2001 historically (no match arm used `CLASS_UID_SECURITY_FINDING`). The Red Gate test is a regression guard that exhaustively iterates all registered sensor × record_type pairs and asserts none return class_uid 2001.

---

## AC-005: No Stale 2001 Assertions

**BC Anchor:** BC-2.02.012 v1.6 — production tests must use current class UID

The `rg '2001'` workspace audit confirms all occurrences of `2001` in `crates/` are:
- Documentation/comments explaining the deprecated UID
- The `CLASS_UID_SECURITY_FINDING: u32 = 2001` constant definition
- Test-internal variables asserting that 2001 is NOT returned
- Unrelated JSON-RPC error code `-32001` in `prism-mcp`

Zero occurrences of `assert_eq!(..., Ok(2001))` or equivalent where 2001 is the expected production return value.

The `bc_2_01_013_spec_driven_adapter.rs` conformance test fixture (S-DEMO-001's test in prism-bin) was updated to assert `class_uid == 2004` (not 2001) as part of this story's implementation.

---

## Stable Reference

This evidence report is anchored to story OCSF-CLASS-MIGRATION-001 v1.9 and behavioral contracts BC-2.02.012 v1.6 + BC-2.01.013 v1.14. No volatile HEAD-SHA pins are used in this report per TD-VSDD-091 anti-volatile-pin convention. The evidence files and VHS recordings are committed to branch `feature/OCSF-CLASS-MIGRATION-001`.
