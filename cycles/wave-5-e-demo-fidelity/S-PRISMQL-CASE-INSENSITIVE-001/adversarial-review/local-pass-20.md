---
document_type: adversarial-review
scope: LOCAL
passes: [20]
story: S-PRISMQL-CASE-INSENSITIVE-001
feature_head: eb7256b7
fix_burst_head: 257074af
date: 2026-07-07
clean_strict: false
clean_pr_merge: false
finding_counts: {HIGH: 1, MED: 2, LOW: 2}
streak_after: 0/3
---

# LOCAL Adversary Pass 20 — S-PRISMQL-CASE-INSENSITIVE-001

---

## Pass 20 (frozen eb7256b7; delta 43 files vs develop@ea714d14)

**Pass result:** CLEAN(strict)=no, CLEAN(PR-merge)=no
**Findings:** 1 HIGH + 2 MED + 2 LOW (5 total; all closed by fix-burst or adjudication)
**Code HEAD at review:** eb7256b7
**Fix-burst HEAD (new frozen candidate for pass-21):** 257074af
**Fix-burst commits on feature branch:** 345d4154 (RG-071 test-writer), 257074af (RG-072 + prism_describe gate tightening)
**LOCAL 3-CLEAN(strict) streak after pass-20:** 0/3 (reset by fix-burst pushes per DRIFT-ORCH-PRLEVEL-PUSH-001)
**Next:** LOCAL pass-21 on frozen 257074af

---

## Finding Inventory

### F-P20-HIGH-001 (HIGH) — AC-019 PRIMARY-path GROUP BY invariant: RG-022 tests SECONDARY normalize_with_mappers path (zero production callers)

**Finding:** AC-019 asserts that the PRIMARY insertion point (`build_column_array` in
`spec_driven_adapter.rs`) is the exclusive production normalization path. However, the sole
group-by-fragmentation guard test at the time (RG-022, `test_BC_2_02_013_group_by_severity_no_fragmentation`)
exercised the SECONDARY path (`normalize_with_mappers` via `OcsfNormalizer`) — a path
with zero production callers in the current architecture (BC-2.02.013 §Invariants PRIMARY/SECONDARY
distinction; established D-1575 F-CRIT-002). The PRIMARY path `build_column_array` processes
records cross-sensor in a GROUP BY context but had no test asserting that records from multiple
sensors with differing raw severity labels (e.g., "high" from CrowdStrike + "High" from Armis)
emerge as the same canonical label (no row-fragmentation at GROUP BY boundary).

This is a coverage gap on the production-critical path, not a production defect. The implementation
at `build_column_array` already calls `normalize_enum_label` correctly. However, a test
exercising the SECONDARY path cannot serve as ground-truth evidence that the PRIMARY path is
correct — the two paths are independent code routes.

**Severity:** HIGH (AC-019 PRIMARY-path GROUP BY contract untested by load-bearing test; SECONDARY
test is structural mismatch with spec contract).

**Closure:** CLOSED by test-writer @345d4154 — `test_BC_2_02_013_build_column_array_group_by_severity_cross_sensor_no_fragmentation`
added in `crates/prism-bin/tests/bc_2_02_013_build_column_array_tests.rs`. Test constructs
two sensor records ("high" from CrowdStrike + "High" from Armis), processes both through
`build_column_array`, and asserts both emerge as `"High"` (no row-fragmentation). Test is
RED-before-commit (GREEN at 345d4154; PRIMARY path already correct — confirms coverage gap
was documentation/test-structure, not production defect). RG-071 row added to story v1.26.

---

### F-P20-MEDIUM-001 (MED) — Story frontmatter `subsystems` missing SS-22

**Finding:** Story frontmatter listed `subsystems: [SS-11, SS-02]`, omitting SS-22 (Process
Lifecycle / `prism-bin`). BC-2.02.013 v1.7 §Traceability explicitly names PRIMARY insertion
point in `prism-bin::build_column_array` (SS-22). The story body referenced SS-22 in its
Architecture Mapping section but the frontmatter field was inconsistent. POL-6 requires
ARCH-INDEX-verbatim subsystem declarations in story frontmatter.

**Severity:** MED (frontmatter vs body inconsistency; POL-6 violation; doc drift).

**Closure:** CLOSED by story-writer in story v1.26 — frontmatter updated to
`subsystems: [SS-11, SS-02, SS-22]`; SS-22 anchor justification added in body
(prism-bin::build_column_array is PRIMARY normalization insertion point per BC-2.02.013
v1.7 F-CRIT-002 adjudication).

---

### F-P20-MEDIUM-002 (MED) [process-gap] — prism-spec-engine absent from story `crates_touched`

**Finding:** `crates_touched` in story frontmatter did not include `prism-spec-engine`.
The adversary found that pass-8 fix-burst (commit 0b2c0983) touched
`crates/prism-spec-engine/` for a TD-VSDD-091 anti-volatile-pin sweep of comment-only
line references in test files. The sweep produced zero code logic changes, only
comment edits. The absence from `crates_touched` therefore misrepresented the delta.

**Severity:** MED (doc drift; `crates_touched` inaccurate).

**Orchestrator adjudication:** ADJUDICATED legitimate. The pass-8 fix-burst 0b2c0983
was a comment-only TD-VSDD-091 de-pin sweep; zero production code changes in
prism-spec-engine. Story v1.26 re-adds prism-spec-engine to crates_touched with
explicit scope annotation ("comment-only TD-VSDD-091 anti-volatile-pin sweep;
pass-8 fix-burst 0b2c0983; zero production code changes"). The `File Structure Requirements`
section updated accordingly (REMOVED annotation corrected to RE-ADDED with comment-only scope).
`tdd_mode` rationale comment updated to reflect accurate crate scope.

**Closure:** CLOSED — story v1.26 documents prism-spec-engine with accurate scope.

---

### F-P20-LOW-001 (LOW) — `prism_describe` `has_severity` gate: name-only check; Integer severity column would emit invalid IEQ example

**Finding:** The `prism_describe` server generates an IEQ example for tables with a
severity column. The gate checked `has_severity` by column name only (`column.name == "severity"`
or equivalent). A hypothetical sensor TOML declaring `severity` as `column_type = "Integer"`
(instead of `String`) would pass the gate and receive an IEQ example in the query tool
description. However, IEQ operates on OCSF canonical string labels; an Integer severity
column is not a valid IEQ target and the generated example would produce a runtime type error.

This is a latent defect: no current sensor TOML declares severity as Integer. But the gate
was specifying name-only, not name+type, and the BC-2.02.013 contract only applies to
String-typed OCSF enum-label fields.

**Severity:** LOW (latent; no current sensor has Integer severity; but gate
is under-specified relative to BC constraint).

**Closure:** CLOSED by implementer @257074af — gate tightened to
`name == "severity" AND column_type == ColumnType::String`. RG-072
(`test_f_p20_low001_severity_integer_type_does_not_get_ieq`, `crates/prism-mcp/tests/`)
RED before fix, GREEN at 257074af. just check 447/447 prism-mcp tests GREEN.

---

### F-P20-LOW-002 (LOW) [process-gap] — BC-2.02.013 frontmatter: scalar `subsystem: "SS-02"` vs Traceability dual-subsystem (PRIMARY SS-22 / SECONDARY SS-02)

**Finding:** BC-2.02.013 frontmatter carried scalar `subsystem: "SS-02"` while the
contract body §Traceability declared a dual-subsystem model (PRIMARY SS-22 Process
Lifecycle `prism-bin`, SECONDARY SS-02 OCSF Normalization). The BC file ID namespace
correctly remains `2.02` (per D-1575 F-CRIT-002 adjudication); the scalar `subsystem`
frontmatter was not updated to reflect the dual model.

**Severity:** LOW [process-gap] — doc inconsistency within the BC file itself.

**Closure:** CLOSED by product-owner in BC-2.02.013 v1.7 — additive
`subsystems_multi: ["SS-22", "SS-02"]` field added to frontmatter (PRIMARY SS-22;
scalar `subsystem: "SS-02"` retained for legacy tooling compatibility; ID family
remains 2.02). No semantic contract change.

---

## Orchestrator-Caught Residual (not adversary-found, caught at orchestrator review)

**Story changelog ordering (POL-32 violation):** The v1.25 story changelog rows were in
ascending version order (oldest first). POL-32 requires descending order (newest first).
This was a pre-existing violation that persisted through passes 1-19 without detection.
The orchestrator caught it at the D-1590 burst prep. Story v1.26 reorders the changelog
descending per POL-32. This reorder is cosmetic (no content change).

---

## SAP Probe Results (Pass 20, verified against eb7256b7)

**SAP-1 (tracing emission catalog completeness):** PASS — `ocsf.enum_label_unrecognized`
dual emission sites (`build_column_array` in `spec_driven_adapter.rs` + `normalize_with_mappers`
in `normalizer.rs`) both match BC-2.16.002 catalog row 91. No new `event_type` sites
introduced in eb7256b7 delta. Catalog count UNCHANGED 91.

**SAP-2 (DTU↔TOML schema parity):** N/A — delta does not touch `.prism/specs/sensors/*.toml`
or DTU clone types/routes. NOTE: The DTU vendor-casing test-vector concern (crowdstrike
SENSOR_COLUMN_VOCABULARIES `status: 'new'→'New'` change in eb7256b7) is a Track-E follow-up
watch item — the fix aligns the in-code vocabulary constant with the actual post-normalization
canonical value; no TOML spec column type change; SAP-2 not triggered.

**SID-1 (no-ignored-test rationalization prohibition):** PASS — all RG-071 and RG-072
tests are non-`#[ignore]` unit/integration tests with no external dependency.

**POL-22 Phase A (ID/anchor integrity):** PASS — 6 BC anchors verified in story body
(BC-2.10.012, BC-2.11.024, BC-2.02.013, BC-2.11.002, BC-2.11.004, BC-2.11.018,
BC-2.02.002, BC-2.02.010). All BC IDs resolve to files in `.factory/specs/behavioral-contracts/`.

**POL-22 Phase C (RGT inventory completeness):** PASS — 15+ domain entities and all
72 RGT names (RG-001..RG-072) verified present in story v1.26 §Red Gate Tests table.

---

## Fix-Burst Commit Log (feature/S-PRISMQL-CASE-INSENSITIVE-001)

| Commit | Author | Change |
|--------|--------|--------|
| 345d4154 | test-writer | RG-071 `test_BC_2_02_013_build_column_array_group_by_severity_cross_sensor_no_fragmentation` — PRIMARY path cross-sensor GROUP BY invariant test (prism-bin); RED before commit, GREEN at 345d4154; F-P20-HIGH-001 coverage gap closed |
| 257074af | implementer | RG-072 `test_f_p20_low001_severity_integer_type_does_not_get_ieq` (prism-mcp); `has_severity` gate tightened to name+ColumnType::String; RED before commit, GREEN at 257074af; 447/447 prism-mcp GREEN; F-P20-LOW-001 closed |

---

## Post-Fix-Burst State

- Feature HEAD: **257074af** (new frozen candidate for pass-21)
- just check: GREEN (5306+ tests; non-exhaustive 89/89 UNCHANGED; RG-001..072 GREEN)
- prism-mcp specific: 447/447 GREEN
- LOCAL 3-CLEAN(strict) streak: **0/3** (reset by fix-burst pushes 345d4154 + 257074af)
- NEXT ACTION: LOCAL adversary pass-21 on frozen 257074af
