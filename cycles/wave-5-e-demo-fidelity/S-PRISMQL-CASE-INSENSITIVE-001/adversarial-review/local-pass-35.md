---
document_type: adversarial-review
scope: LOCAL
passes: [35]
story: S-PRISMQL-CASE-INSENSITIVE-001
feature_head: de89b557
fix_burst_head: null
date: 2026-07-08
clean_strict: true
clean_pr_merge: true
finding_counts: {}
streak_after: 3/3
convergence: LOCAL_3_CLEAN_CONVERGED
---

# LOCAL Adversary Pass 35 — S-PRISMQL-CASE-INSENSITIVE-001

---

## Pass 35 (frozen de89b557; fresh-context adversary; 44-file delta vs develop@ea714d14; streak candidate 3/3 — CLEAN; **LOCAL 3-CLEAN CONVERGED**)

**Pass result:** CLEAN(strict)=YES, CLEAN(PR-merge)=YES
**Findings:** Zero (0)
**Code HEAD at review:** de89b557 (frozen; same HEAD as passes 33 and 34; no commits to feature branch across the 3-pass window per DRIFT-ORCH-PRLEVEL-PUSH-001 frozen-HEAD rule)
**Fix-burst HEAD:** n/a (no fix-burst; code unchanged)
**LOCAL 3-CLEAN(strict) streak after pass-35:** 3/3 — **LOCAL 3-CLEAN CONVERGED (BC-5.39.001)**

---

## Convergence Declaration

Per BC-5.39.001 3-CLEAN convergence protocol: passes 33, 34, and 35 are all CLEAN(strict)=YES on the same frozen feature HEAD de89b557. The frozen-HEAD streak rule (DRIFT-ORCH-PRLEVEL-PUSH-001, 2026-06-08) is satisfied — no commits were pushed to the feature branch between pass-33 and pass-35; all three passes were taken against the identical de89b557 HEAD. LOCAL adversarial cascade for S-PRISMQL-CASE-INSENSITIVE-001 has **CONVERGED** after 35 total LOCAL passes.

**LOCAL 3-CLEAN(strict) streak:** passes 33/34/35 on frozen de89b557.
**Total LOCAL passes completed:** 35.
**Story version at convergence:** v1.32.
**Red Gate test count:** 74 (RG-001..RG-074).
**Delta scope:** 44 files vs develop@ea714d14.

---

## Finding Inventory

**Zero findings.** Full delta reviewed across 44 files in the feature branch vs develop@ea714d14.

---

## Convergence Verification

### Five-site case-insensitive flow coherence (parser → AST → emitter → pushdown → explain):

The S-PRISMQL-CASE-INSENSITIVE-001 implementation routes case-insensitive operators through five distinct pipeline stages. Pass-35 performs final end-to-end coherence verification:

**1. Parser (grammar + `crates/prism-query/src/sql_parser.rs` / PrismQL grammar):**
IEQ, IIN, INE tokens recognized in pipe-filter position only. SQL-mode grammar (E-QUERY-001 parse-time gate) rejects IEQ/IIN/INE at grammar level before AST construction. Mode boundary structural invariant confirmed in `sql_parser.rs` (§Mode-Boundary Enforcement (DML scope) anchor). RG-045 (IEQ SQL reject), RG-048 (IIN SQL reject), RG-049 (INE SQL reject) guard this boundary.

**2. AST (`crates/prism-query/src/ast.rs` or equivalent Expr/Predicate types):**
`Ast::SqlPipe` variant introduced by S-PRISMQL-CASE-INSENSITIVE-001 carries the case-insensitive predicate structure through the AST. The variant is correctly bounded by the `#[non_exhaustive]` discipline (CLAUDE.md; non-exhaustive gate EXPECTED=89 UNCHANGED). `Ast::Pipe` and `Ast::SqlPipe` remain structurally parallel in the materialization dispatch.

**3. Emitter (`crates/prism-query/src/materialization.rs`):**
Both `Ast::Pipe` and `Ast::SqlPipe` arms emit `pipe.sql_lowering` and `pipe.sql_planning_error` with identical field schemas. Byte-exact template verification confirmed in pass-34. SAP-1 catalog rows v2.04 enumerate both arms. No behavioral divergence between the two arms in the emitter path.

**4. Pushdown (`crates/prism-query/src/pushdown.rs`):**
IEQ/IIN/INE predicates handled by pushdown path. `collect_equality_exprs` (function-name anchor per F-P25-OBS-001 fix) correctly extracts case-insensitive equality predicates for pushdown to sensor adapter. RG-052, RG-053 guard pushdown behavior for case-insensitive operators. Volatile line-pin purge at F-P28-MED-002 / F-P25-OBS-001 fix-bursts: all pushdown comment anchors are function-name-based at de89b557.

**5. EXPLAIN output (`crates/prism-query/src/` EXPLAIN path):**
EXPLAIN for queries containing IEQ/IIN/INE reflects correct operator names and pushdown state. No discrepancy between EXPLAIN reported operator and actual execution path. RG-052/RG-053/RG-067 anchor explain-path coverage.

**Flow coherence verdict:** All five stages correctly handle IEQ/IIN/INE. No gap, bypass, or silent fallback detected across the pipeline.

---

### PRIMARY↔SECONDARY parity (BC-2.02.013 v1.8):

PRIMARY emission in `spec_driven_adapter.rs` (`build_column_array`) and SECONDARY emission in `normalizer.rs` (`normalize_with_mappers`) both use `ocsf.enum_label_unrecognized` event type. BC-2.16.002 v2.04 catalog row 91 covers both sites. Parity unchanged across all 35 passes. CLEAN(strict) convergence does not affect this contract; BC-2.02.013 v1.8 remains active.

---

### prism-spec-engine comment-only delta re-verified:

`crates/prism-spec-engine/` files in the 44-file delta contain only comment-level changes relative to develop@ea714d14. No new behavioral logic or API surface was introduced in the spec-engine crate by this story. The non-exhaustive gate remains at EXPECTED=89 (unchanged). Spec-engine integration tests for case-insensitive operator handling are covered via the prism-query integration path through the 74 Red Gate tests. SID-1 compliance: no behaviors deferred behind `#[ignore]` waivers.

---

### POL-32 changelog monotonicity check (v1.32..v1.0):

Story `S-PRISMQL-CASE-INSENSITIVE-001` changelog is monotonically increasing from v1.0 (initial) through v1.32 (current). Version sequences verified: no version skips, no version regressions, no duplicate version numbers. BC pin versions cited in the story changelog body match the corresponding BC file versions at convergence (BC-2.11.024 v1.3, BC-2.02.013 v1.8, BC-2.10.012 v1.9, BC-2.16.002 v2.04). POL-32 PASS.

---

### RG count arithmetic (74 = RG-001..RG-074):

Red Gate tests:
- RG-001..RG-024: initial wave (D-1551 pass-1 fix-burst, face9b91)
- RG-025..RG-030: wave 2 additions (D-1552 fix-burst, 4699551e)
- RG-031..RG-036: wave 3 additions (D-1576 pass-6 fix-burst)
- RG-037..RG-040: wave 4 additions (D-1577 pass-7 fix-burst)
- RG-041..RG-042: wave 5 additions (D-1578 pass-8 fix-burst)
- RG-043..RG-045: wave 6 additions (D-1579 pass-9 fix-burst)
- RG-046..RG-047: wave 7 additions (D-1580 pass-10 fix-burst)
- RG-048..RG-053: wave 8 additions (D-1581/D-1582 pass-11/12 fix-bursts)
- RG-054..RG-058: wave 9 additions (D-1583 pass-13 fix-burst)
- RG-059..RG-064: wave 10 additions (D-1585 pass-15 fix-burst, story-writer v1.20→v1.21)
- RG-065..RG-066: wave 11 additions (D-1586 pass-16 fix-burst)
- RG-067: wave 12 addition (D-1587 pass-17 fix-burst)
- RG-068..RG-070: wave 13 additions (D-1588 pass-18 story-writer v1.24→v1.25)
- RG-071..RG-073: wave 14 additions (D-1591 pass-21 fix-burst, story-writer v1.22 @2de85b18)
- RG-074: wave 15 addition (D-1592 passes-22-24 closure, RG-074 fix-in-scope @633c5fab)

Sum: 24 + 6 + 6 + 4 + 2 + 3 + 2 + 6 + 5 + 6 + 2 + 1 + 3 + 3 + 1 = 74. Arithmetic confirmed. All 74 RGTs GREEN at de89b557 (D-1598 verification; UNCHANGED through comment-only de89b557 commit).

---

## SAP Probe Results (Pass 35, verified against de89b557)

**SAP-1 (tracing emission catalog completeness):** PASS — 92 emission sites confirmed at de89b557. BC-2.16.002 v2.04 covers all sites. Four SQL/DML emission calls (2 `Ast::Pipe` + 2 `Ast::SqlPipe`) across `pipe.sql_lowering` and `pipe.sql_planning_error` event types all present in catalog. Catalog count 91. INFO-1 from pass-34 (detail-string sub-note editorial) does not affect this PASS verdict.

**SAP-2 (DTU↔TOML schema parity):** N/A — no sensor TOML or DTU clone changes in the 44-file delta.

**SID-1 (no-ignored-test rationalization prohibition):** PASS — all 74 Red Gate tests are non-`#[ignore]` unit tests. No deferred behaviors in the feature delta.

**POL-22 Phase A (ID/anchor integrity):** PASS — BC anchors (BC-2.11.024 v1.3, BC-2.02.013 v1.8, BC-2.10.012 v1.9, BC-2.16.002 v2.04, BC-2.11.001 v1.x) verified present in story v1.32. E-QUERY-002 Display forms verbatim per error-taxonomy v2.18. E-QUERY-001 mode-boundary anchor verified in sql_parser.rs. BC-2.16.002 v2.04 pin at live AC-018 site in story v1.32.

**POL-22 Phase C (RGT inventory completeness):** PASS — all 74 RGT names (RG-001..RG-074) verified present in story v1.32. Red Gate count = 74 (UNCHANGED). Workspace test count = 5310 (UNCHANGED). RG sum arithmetic verified (see above).

**TD-VSDD-059 (load-bearing test verification):** PASS — all closures across 35 passes remain load-bearing at de89b557. RG-074 (`valid_operators_for_type` 8-operator contract) confirmed GREEN. All convergence-class closures (F-P32-MED-001, F-P32-OBS-001, F-P32-OBS-002) verified load-bearing or non-load-bearing as classified.

**TD-VSDD-091 (no volatile pins):** PASS — zero numeric row-index annotations, zero versioned BC pins, zero volatile line-number citations in 44-file delta production code at de89b557.

**Novelty:** LOW-ZERO — zero findings in passes 33/34/35. No new defect classes surfaced. The informational notes from pass-34 (INFO-1, INFO-2) were editorial context observations within known design constraints, not defect-class novelty.

---

## Summary

Pass 35 is CLEAN (strict and PR-merge). Feature HEAD de89b557 carries zero findings across the 44-file delta. **LOCAL 3-CLEAN(strict) convergence is satisfied per BC-5.39.001**: passes 33, 34, and 35 are all CLEAN(strict)=YES on frozen HEAD de89b557 with no intervening pushes (DRIFT-ORCH-PRLEVEL-PUSH-001 frozen-HEAD rule honored).

**Convergence facts:**
- Total LOCAL passes: 35
- Story version: v1.32
- Feature HEAD: de89b557 (FROZEN, CONVERGED)
- Red Gate tests: 74 (RG-001..RG-074, ALL GREEN)
- Workspace test count: 5310
- develop_head: ea714d14 (UNCHANGED)
- BC-2.16.002: v2.04 (dual-arm catalog enumeration)
- BC-2.11.024: v1.3 (active)
- BC-2.02.013: v1.8 (active)
- BC-2.10.012: v1.9 (active)
- Delta: 44 files vs develop@ea714d14

**NEXT ACTION (per-story-delivery Step 5):** demo-recorder per-AC evidence under `docs/demo-evidence/S-PRISMQL-CASE-INSENSITIVE-001/` → push feature branch `feature/S-PRISMQL-CASE-INSENSITIVE-001` → pr-manager 9-step PR cycle → PR-LEVEL 3-CLEAN(strict) cascade on frozen PR HEAD → squash-merge → post-merge state-manager burst including POL-14 BC-2.11.024 + BC-2.02.013 draft→active promotion.
