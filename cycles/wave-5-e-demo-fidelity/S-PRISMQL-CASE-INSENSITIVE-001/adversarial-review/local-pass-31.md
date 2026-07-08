---
document_type: adversarial-review
scope: LOCAL
passes: [31]
story: S-PRISMQL-CASE-INSENSITIVE-001
feature_head: 669080f5
fix_burst_head: f7040e96
date: 2026-07-08
clean_strict: false
clean_pr_merge: true
finding_counts: {OBS: 1}
streak_after: 0/3
---

# LOCAL Adversary Pass 31 — S-PRISMQL-CASE-INSENSITIVE-001

---

## Pass 31 (frozen 669080f5; fresh-context adversary; 44-file delta vs develop@ea714d14; streak candidate 1/3 — NOT CLEAN)

**Pass result:** CLEAN(strict)=NO (1 OBS), CLEAN(PR-merge)=YES
**Findings:** 1 (F-P31-OBS-001 OBS — stale test-file header docstring; CLOSED implementer @f7040e96; doc-only)
**Code HEAD at review:** 669080f5 (frozen; unchanged from pass-30)
**Fix-burst HEAD:** f7040e96 (doc-comment-only; no behavioral code change; no test logic change; cargo check clean)
**LOCAL 3-CLEAN(strict) streak after pass-31:** 0/3 (RESET by 1 OBS finding; new frozen candidate HEAD for pass-32: f7040e96)

---

## Finding Inventory

### F-P31-OBS-001 (OBS) — stale file-header docstring in test_case_insensitive_operators.rs

**Severity:** OBS — TD-VSDD-091 (narrative doc-drift class). The file-level module doc comment (`//! ...`) in `crates/prism-query/src/tests/test_case_insensitive_operators.rs` claimed a stale inventory: "24 tests, RG-001..RG-018+3" (or similar enumeration referencing an early RG count). The actual file contains 46 test functions spanning a much broader RG range (RG-001 through RG-074 via multiple AC groups). The discrepancy between the header's static count/range claim and the actual file contents constitutes a doc-accuracy finding under TD-VSDD-091 (narrative spec content must cite behavioral anchors and area descriptions, not static counts/line ranges that decay with each added test).

**Root cause:** The file header was written during early development when 24 tests covered RG-001..RG-018 plus a few supplemental cases. Subsequent fix-bursts (D-1571..D-1596) added RG-019 through RG-074 across 20+ burst commits without updating the module-level docstring. By pass-31 (feature HEAD 669080f5) the header's claimed inventory was 22 tests behind reality.

**POL-24 / TD-VSDD-091 trigger:** Static count claims ("24 tests", "RG-001..RG-018+3") in test-file headers become false as soon as a test is added. The correct pattern (established in TD-VSDD-091 precedent) is area-based non-exhaustive descriptions that point readers to the canonical Red Gate Inventory in the story spec rather than duplicating a count in the file.

**No behavioral code change required:** The feature code at 669080f5 is correct. All 46 test functions, their assertion logic, and the RG-001..RG-074 inventory are sound. This is a documentation accuracy finding only.

**TD-VSDD-060 mini-sweep:** Fresh-context adversary also checked sibling test files in the same story delta for the same stale-count anti-pattern. `crates/prism-query/src/tests/test_adapter_normalization.rs` had a header that named only 3 of 7 OCSF normalization areas covered by the file. This was also corrected in the same fix-burst (same doc-only commit). Three other story test files (`test_case_insensitive_describe.rs`, `test_case_insensitive_errors.rs`, `test_case_insensitive_pipe_mode.rs`) verified accurate — their headers use area-based descriptions rather than static counts.

**Closure:** CLOSED — implementer @f7040e96: `test_case_insensitive_operators.rs` module doc comment rewritten to durable area-based non-exhaustive description (lists the 8 behavioral areas covered — operator parsing, enum-label normalization, PRIMARY↔SECONDARY parity, mode-boundary enforcement, type-mismatch feedback, string-operator set, pipe-mode IEQ precedence, RGT coverage) and directs readers to story S-PRISMQL-CASE-INSENSITIVE-001 Red Gate Inventory as the authoritative RGT count. `test_adapter_normalization.rs` header updated to name all 7 OCSF normalization areas. No test logic, no assertion, no production code touched. `cargo check` clean at f7040e96. Feature code at f7040e96 is now the frozen candidate HEAD for pass-32.

---

## Observations (non-finding)

### OBS-P31-001 — Streak reset mechanics per BC-5.39.001 + DRIFT-ORCH-PRLEVEL-PUSH-001

**Classification:** Process observation; NOT a new finding class.

**Observation:** F-P31-OBS-001 is a doc-accuracy finding (TD-VSDD-091 class). The finding was present in the pass-31 review of frozen HEAD 669080f5. Per BC-5.39.001, CLEAN(strict) requires ZERO findings of ANY severity including OBS. This pass is NOT CLEAN(strict). The LOCAL 3-CLEAN(strict) streak resets to 0/3.

Per DRIFT-ORCH-PRLEVEL-PUSH-001, the fix-burst commit f7040e96 lands on the feature branch. The new frozen HEAD for pass-32 is f7040e96. The streak counter resets at 0/3 and restarts from pass-32 on f7040e96.

**Note on CLEAN(PR-merge) = YES:** F-P31-OBS-001 is OBS severity only. CLEAN(PR-merge) criterion (ZERO CRIT + HIGH + MED) is satisfied. This finding would not block a PR merge gate, but it DOES reset the strict streak per BC-5.39.001.

### OBS-P31-002 — SAP-1 consistency with passes 29-30 (92 emission sites)

The fix-burst at f7040e96 is doc-comment-only. No new `event_type =` emission sites were added. SAP-1 result is identical to passes 29 and 30: 92 emission sites, all catalogued. Catalog row 91 (dual-site `ocsf.enum_label_unrecognized`: PRIMARY `build_column_array` in `spec_driven_adapter.rs`, SECONDARY `normalize_with_mappers` in `normalizer.rs`) verified structurally present.

---

## SAP Probe Results (Pass 31, verified against 669080f5 → fix-burst f7040e96)

**SAP-1 (tracing emission catalog completeness):** PASS — 92 emission sites; catalog row 91 dual-site `ocsf.enum_label_unrecognized` verified. Fix-burst f7040e96 is doc-comment-only; no new emission sites introduced. All 92 sites remain catalogued in BC-2.16.002 §Postconditions.

**SAP-2 (DTU↔TOML schema parity):** N/A — no sensor TOML or DTU clone changes in the 44-file delta.

**SID-1 (no-ignored-test rationalization prohibition):** PASS — all 74 Red Gate tests are non-`#[ignore]` unit tests. Fix-burst f7040e96 is doc-comment-only; no `#[ignore]` added; unchanged from pass-30.

**POL-22 Phase A (ID/anchor integrity):** PASS — BC anchors (BC-2.11.024 v1.3, BC-2.02.013 v1.8, BC-2.10.012 v1.9, BC-2.16.002 v2.03, BC-2.11.001 v1.x) verified present in story v1.31. E-QUERY-002 Display forms verified verbatim per taxonomy v2.18/v2.19. E-QUERY-001 mode-boundary anchor BC-2.11.024 §Mode-Boundary Enforcement (DML scope) verified in sql_parser.rs comment.

**POL-22 Phase C (RGT inventory completeness):** PASS — all 74 RGT names (RG-001..RG-074) verified present in story v1.31. Red Gate count = 74. Workspace test count = 5310 (UNCHANGED; fix-burst f7040e96 is doc-comment-only; no new tests).

**TD-VSDD-059 (load-bearing test verification for recent closures):** PASS — all findings closed in passes 28-30 confirmed load-bearing:
- F-P28-MED-001 (POL-4 temporal-indexical class): section-heading rename has no assertion dependency; the finding was a doc-accuracy issue (structural).
- F-P28-MED-002 (TD-VSDD-091 versioned BC pin strip): zero versioned BC pin citations remain in delta production files at 669080f5/f7040e96 (grep confirmed).
- F-P28-LOW-001 (§Mode-Boundary Enforcement cite): code comment navigability fix; doc-accuracy; correct section reference present.
- F-P30-MED-001 (POL-26/POL-32 provenance row): error-taxonomy.md v2.19 changelog row present; POL-26 provenance completeness satisfied; POL-32 monotonic ordering intact.
No paper-fixes (doc-comment-only-without-structural-change to a behavioral claim) detected.

**TD-VSDD-060 (Compare/In construction-site sweep):** PASS — grep for `IEQ`/`IIN`/`INE` operator construction sites across `crates/prism-query/src/` confirms all sites (parser, engine, error_mapping, e_query_pedagogical) are consistent with the 8-operator contract. No new construction sites added in fix-burst f7040e96 (doc-only). Mini-sweep of test-file headers during F-P31-OBS-001 closure extended TD-VSDD-060 coverage to the 5 story test files.

**TD-VSDD-091 (no versioned BC pins in spec-engine production code):** PASS — 0 versioned BC pin citations in `crates/prism-spec-engine/` production code. Unchanged from pass-29/30. The fix-burst f7040e96 touched only `crates/prism-query/src/tests/` doc comments; no spec-engine files modified.

**Novelty:** LOW — F-P31-OBS-001 is a familiar doc-drift class (TD-VSDD-091 stale-count anti-pattern; same class as the volatile line-pin sweep in pass-9 and the doc-comment sweeps in passes 5-7). No new finding classes. No structural defect candidates observed.

---

## Fix Summary

| Finding | Fix | Files | Commit |
|---------|-----|-------|--------|
| F-P31-OBS-001 | Area-based non-exhaustive header rewrite replacing stale "24 tests, RG-001..RG-018+3" static count; TD-VSDD-060 mini-sweep fixed `test_adapter_normalization.rs` header (all 7 areas named); 3 other story test files verified accurate | `crates/prism-query/src/tests/test_case_insensitive_operators.rs`, `crates/prism-query/src/tests/test_adapter_normalization.rs` | implementer @f7040e96 (doc-comment-only; no behavioral change) |

---

## Post-Fix State

- Feature HEAD: **f7040e96** (doc-comment-only; no behavioral code change; frozen candidate for pass-32)
- Prior feature review HEAD: 669080f5 (UNCHANGED behaviorally)
- 1407/1407 prism-query tests GREEN (UNCHANGED)
- 447/447 prism-mcp tests GREEN (UNCHANGED)
- non-exhaustive: 89/89 UNCHANGED
- RG-001..074 GREEN (UNCHANGED)
- LOCAL 3-CLEAN(strict) streak: **0/3** (RESET by 1 OBS finding in this pass; new frozen HEAD f7040e96)
- Novelty: LOW (TD-VSDD-091 doc-drift class; well-precedented)
- NEXT ACTION: LOCAL adversary pass-32 on frozen f7040e96 (streak candidate 1/3; no commits to feature branch between pass-32 and 33 per DRIFT-ORCH-PRLEVEL-PUSH-001)
