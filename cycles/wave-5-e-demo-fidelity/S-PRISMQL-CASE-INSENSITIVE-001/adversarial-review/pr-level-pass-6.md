---
document_type: adversarial-review
scope: PR-LEVEL
passes: [6]
story: S-PRISMQL-CASE-INSENSITIVE-001
pr: 217
feature_head_at_review: 36a094d6
base_develop_head: 7b1f6c51
closure_head: 36a094d6
date: 2026-07-08
clean_strict: true
clean_pr_merge: true
finding_counts:
  total: 0
streak_after: 1/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay from adversary pass-6 output
---
# PR-LEVEL Adversarial Review — Pass 6
## S-PRISMQL-CASE-INSENSITIVE-001

**Frozen HEAD:** 36a094d6 (feature/S-PRISMQL-CASE-INSENSITIVE-001)
**Base:** develop@7b1f6c51
**Date:** 2026-07-08
**Authored by:** orchestrator-relay from adversary pass-6 output

---

## Verdict

| Criterion | Result |
|-----------|--------|
| CLEAN (strict) | **yes** |
| CLEAN (PR-merge) | **yes** |

**Finding summary:** 0 findings total. Zero CRIT, HIGH, MED, LOW, OBS, PROCESS-GAP.

**Novelty:** LOW — no new defect classes surfaced.

**Streak status:** 1/3 — first CLEAN(strict) pass on frozen 36a094d6 (new frozen HEAD after D-1605 fix-burst push; prior streak reset per DRIFT-ORCH-PRLEVEL-PUSH-001).

---

## Findings

None.

---

## Candidate Probes Examined and Dropped (With Rationale)

Two candidate probe angles were evaluated and dropped before the pass report was finalized:

**1. SqlPipe alias-projection pre-flight bypass** — The adversary examined whether the IEQ/INE/IIN operators applied to a SELECT-alias column (e.g., `SELECT lower(severity) AS s ... WHERE s IEQ 'high'`) would bypass case-insensitive normalization because the alias is resolved post-normalization. Investigation confirmed this is a pre-existing documented design constraint: SqlPipe's WHERE clause operates on the pre-alias column stream per the grammar spec; the alias-projection pre-flight behavior is correct-by-design per the documented SqlPipe evaluation order. Not a defect.

**2. Catalog/schema None bypass via `catalog_manager.table_schema()`** — A probe examined whether `catalog_manager.table_schema()` returning `None` for an unregistered sensor could bypass CI column-type guards in a way analogous to ADV-PR-P5-OBS-002 (the `check_ci_column_types` `.ok()` bypass closed @36a094d6). Investigation found that the `catalog_manager.table_schema()` path is protected by a doc-commented, intentional early-return gate (the schema is `None` only when the sensor spec has not been registered, which happens before query execution reaches the CI normalization path). This is correct-by-design and is explicitly doc-commented. Not a defect.

---

## Probe Results

### SAP-1 — Tracing emission catalog completeness

**Result: CLEAN**

All `event_type =` sites re-verified at frozen 36a094d6 (includes the fix-burst additions from @36a094d6). Row-91 `ocsf.enum_label_unrecognized` sites (PRIMARY: `crates/prism-bin/src/spec_driven_adapter.rs`; SECONDARY: `crates/prism-ocsf/src/normalizer.rs`) verified present with matching catalog entries. The bare `tracing::error!` added in the @36a094d6 fix-burst (OBS-002 closure — `check_ci_column_types` error propagation) carries no `event_type` field and is D-765-class exempt. No new catalog rows required.

### SAP-2 — DTU↔TOML schema parity

**Result: N/A** — this story does not modify `.prism/specs/sensors/*.toml` or DTU clone route/type files.

### POL-22 — Phase A+C gates

**Result: CLEAN** — Phase A (story frontmatter completeness, v1.36 at closure HEAD 36a094d6) and Phase C (BC traceability: all 8 BCs present including BC-2.16.002 v2.06 + error-taxonomy v2.20 pins) both verified clean.

### CWE-117 — Log injection order at PRIMARY+SECONDARY

**Result: CLEAN both sites** — RG-079 (SECONDARY load-bearing helper test), RG-080 (PRIMARY order-of-operations vector test with sensor_type mirror), RG-082 (Unicode Cc + U+2028/U+2029 widened scope test) all GREEN at 36a094d6. Sanitize-before-truncate order correct per BC-2.16.002 v2.06 row 91. The widened `sanitize_for_log` (introduced @36a094d6) strips Unicode Cc category (includes ASCII controls AND C1 U+0080–U+009F) plus U+2028/U+2029. No new log injection surface.

### Memory/performance probe

**Result: CLEAN** — The `OcsfEnumMap` static is populated once at startup via `OnceLock` (standard lazy-init pattern). No per-query allocation hotpath. The `shared_enum_map()` single-access-point consolidation @f2215872 (D-1586) eliminated the duplicate statics. No memory regression.

### Error-path completeness probe

**Result: CLEAN** — `check_ci_column_types` now propagates `SchemaProvider::table()` errors via `?` (ADV-PR-P5-OBS-002 closure @36a094d6). RG-083 covers the `Err` → `QueryExecutionFailed` path. No remaining `.ok()` silencing patterns found in CI validation paths.

### Cross-crate coherence probe

**Result: CLEAN** — `prism_core::sanitize_for_log` is the single implementation. `prism-ocsf` and `prism-bin` both import from `prism-core`. No divergent copies exist.

### MCP tool text probe

**Result: CLEAN** — The `query` tool description in `resources.rs` teaches post-normalization casing per BC-2.10.012 v1.9 (IEQ-first, Title-case canonical). No vendor-casing instructions present. Consistent with story v1.36 AC-025.

### Assertion strength probe

**Result: CLEAN** — RG-001..083 all GREEN. Key assertion classes: RG-079/080/082 (CWE-117 sanitize order + widened scope), RG-083 (error propagation), RG-075 (idempotence guard), RG-076 (warn before IEQ fallback). No vacuous `assert!(true)` patterns detected.

### Paper-fix audit

**Result: none** — All closures from the D-1605 fix-burst are load-bearing (RG-082 RED→GREEN, RG-083 RED→GREEN, production code changed). No doc-comment-only closures.

---

## Convergence Trajectory (PR-LEVEL)

| Pass | Frozen HEAD | CLEAN(strict) | CLEAN(PR-merge) | Findings | Streak |
|------|------------|---------------|-----------------|----------|--------|
| 1    | a2fc8940   | no            | no              | 2 MED + 2 LOW + 2 OBS (total 6) | 0/3 reset |
| 2    | 1172b15a   | no            | yes             | 1 LOW (total 1)                 | 0/3 (push resets) |
| 3    | dcb37099   | no            | yes             | 2 OBS (total 2)                 | 0/3 (push resets) |
| 4    | fab7df00   | yes           | yes             | 0 (total 0)                     | 1/3 |
| 5    | fab7df00   | no            | yes             | 3 OBS (total 3)                 | 0/3 RESET |
| 6    | **36a094d6** | **yes**     | **yes**         | 0 (total 0)                     | **1/3** |

---

## Post-Pass Action

No fix-burst required. **VERY NEXT ACTION:** PR-LEVEL adversary pass-7 on same frozen HEAD 36a094d6. Per DRIFT-ORCH-PRLEVEL-PUSH-001, no push occurred — streak carries forward. If pass-7 is also CLEAN(strict), streak advances to 2/3.
