---
document_type: adversarial-review
scope: LOCAL
passes: [34]
story: S-PRISMQL-CASE-INSENSITIVE-001
feature_head: de89b557
fix_burst_head: null
date: 2026-07-08
clean_strict: true
clean_pr_merge: true
finding_counts: {}
streak_after: 2/3
informational_notes: [INFO-1, INFO-2]
---

# LOCAL Adversary Pass 34 — S-PRISMQL-CASE-INSENSITIVE-001

---

## Pass 34 (frozen de89b557; fresh-context adversary; 44-file delta vs develop@ea714d14; streak candidate 2/3 — CLEAN)

**Pass result:** CLEAN(strict)=YES, CLEAN(PR-merge)=YES
**Findings:** Zero (0)
**Informational notes:** 2 (INFO-1, INFO-2 — non-finding, no streak impact)
**Code HEAD at review:** de89b557 (frozen; same HEAD as pass-33; no commits to feature branch between pass-33 and pass-34 per DRIFT-ORCH-PRLEVEL-PUSH-001)
**Fix-burst HEAD:** n/a (no fix-burst; code unchanged)
**LOCAL 3-CLEAN(strict) streak after pass-34:** 2/3

---

## Finding Inventory

**Zero findings.** Full delta reviewed across 44 files in the feature branch vs develop@ea714d14.

---

## Informational Notes (non-finding; no streak impact)

### INFO-1 — BC-2.16.002 `pipe.sql_planning_error` row detail-string prose cites Ast::Pipe variant only (inside CWE-209 redacted region; editorial-only)

**Classification:** Informational note (NOT a finding). BC-2.16.002 v2.04 `pipe.sql_planning_error` catalog row's "detail-string content" sub-note references the `Ast::Pipe` error formatting detail in its prose example (the `format!("{pipe}")` display string from the original ENRICH-4-B registration). The `Ast::SqlPipe` arm emits `error: %display` with the equivalent `spq` detail string, which is behaviorally symmetric but not separately narrated in the sub-note. This divergence is entirely within the CWE-209 error-detail region (the catalog row already covers both arms in its trigger-condition column per the F-P32-MED-001 v2.04 fix). The sub-note prose is editorial background, not a contract specification. No BC-specified field schema, recurrence policy, or audit role is affected.

**Rationale for non-finding classification:** (1) CWE-209 scope: the detail-string content ("what the error display looks like") is redacted context for security reviewers, not a behavioral contract term. (2) Both arms emit `error: %display` — the field schema is identical; the note is clarifying prose, not a contract boundary. (3) No test assertion, no code path, and no validation rule depends on this prose text. (4) The v2.04 fix already satisfied the SAP-1 / POL-29 requirement (dual-arm enumeration in trigger-condition column). Further editorial refinement of the detail-string sub-note is optional grooming, not a defect.

**Disposition:** Record only. No action required. Not a streak-blocking item.

---

### INFO-2 — IEQ/INE parser messages intentionally non-spec-pinned per story v1.32 design

**Classification:** Informational note (NOT a finding). The IEQ and INE parse-time non-string RHS guard messages in `crates/prism-query/` are implementation-level UX messages without a BC-specified verbatim contract. Story v1.32 explicitly documents that IEQ/INE error messages are non-spec-pinned (product-owner confirmation per F-P32-OBS-002 rationale: "neither BC-2.11.024 nor BC-2.02.013 pin the exact text of the IEQ/INE non-string RHS message"). The F-P32-OBS-002 fix at de89b557 removed the inaccurate parenthetical "(integer, float, or boolean)" — the message is now accurate without overspecifying. Contrast: IIN message IS BC-pinned per BC-2.11.024 v1.3 §EC-11-024-012 and verified verbatim-intact at de89b557.

**Rationale for non-finding classification:** A non-spec-pinned message that accurately describes the behavior is correct by definition — there is no contract to violate. The v1.32 story design deliberately left IEQ/INE messages at implementation-level UX to allow future editorial refinement without BC amendments. This is a product-owner-intentional design choice, not a gap.

**Disposition:** Record only for session context. No action required. Not a streak-blocking item.

---

## Delta Review (44-file delta, frozen de89b557)

### Byte-exact template verification across SQL/DML emission sites:

**Four SQL/DML emission sites in `materialization.rs` verified byte-exact against BC-2.16.002 v2.04:**

1. `Ast::Pipe` → `pipe.sql_lowering` (DEBUG): `pipe_sql: %display` field emitted after `pipe_to_executable_sql`; event_type = "pipe.sql_lowering". SAP-1 catalog row trigger-condition cites this site. Template match: PASS.

2. `Ast::Pipe` → `pipe.sql_planning_error` (ERROR): `error: %display, pipe_sql: %display` fields emitted in `Err` arm; event_type = "pipe.sql_planning_error". Catalog row trigger-condition cites this site. Template match: PASS.

3. `Ast::SqlPipe` → `pipe.sql_lowering` (DEBUG): `pipe_sql: %display` field emitted after `sqlpipe_to_executable_sql`; event_type = "pipe.sql_lowering"; SAP-1 durable comment anchor present ("// SAP-1: reuse `pipe.sql_lowering` catalog event_type"). Template match: PASS. Zero numeric index annotations.

4. `Ast::SqlPipe` → `pipe.sql_planning_error` (ERROR): `error: %display, pipe_sql: %display` fields; event_type = "pipe.sql_planning_error"; SAP-1 durable comment anchor present. Template match: PASS. Zero numeric index annotations.

All four sites consistent with BC-2.16.002 v2.04 field schema (event_type, trace context, audit role, recurrence policy). The INFO-1 editorial observation about detail-string sub-note prose does not affect any template-match criterion — the field schema is identical across both arms.

**Case-insensitive operator construction (BC-2.11.024 v1.3):**
IEQ/IIN/INE construction sites in the 44-file delta re-verified. The non-string RHS guard messages (IEQ/INE) have no "(integer, float, or boolean)" parenthetical. IIN guard message ("IIN operand list must contain only string literals") verified verbatim per §EC-11-024-012. Construction-site count unchanged from pass-33. TD-VSDD-060 mini-check: no sibling IEQ/INE message sites missed — all in the same operator-dispatch function family.

**Grammar and pushdown coherence:**
PrismQL parser pipeline at de89b557: grammar correctly routes IEQ/IIN/INE as pipe-filter operators, not SQL-mode predicates. E-QUERY-001 SQL-mode rejection guard verified structurally intact in `sql_parser.rs`. Pushdown path (`crates/prism-query/src/pushdown.rs`) confirmed to handle IEQ/IIN/INE predicate extraction correctly. EXPLAIN output for case-insensitive operators verified to show pushdown state accurately. No regressions from de89b557 (code-comment-only commit).

**prism-spec-engine (comment-only scope):**
`crates/prism-spec-engine/` files in the 44-file delta are comment-only (no behavioral logic changes relative to develop). Spec-engine integration tests for case-insensitive operator dispatch verified via RGT coverage through prism-query integration path. No net-new spec-engine behavioral code in de89b557 vs b341cdd7..de89b557 range.

---

## SAP Probe Results (Pass 34, verified against de89b557)

**SAP-1 (tracing emission catalog completeness):** PASS — 92 emission sites confirmed. BC-2.16.002 v2.04 catalog rows for `pipe.sql_lowering` and `pipe.sql_planning_error` enumerate both `Ast::Pipe` and `Ast::SqlPipe` arms. All 92 sites in catalog. INFO-1 sub-note editorial observation: non-finding (CWE-209 detail-string region, not a catalog coverage gap).

**SAP-2 (DTU↔TOML schema parity):** N/A — no sensor TOML or DTU clone changes in the 44-file delta.

**SID-1 (no-ignored-test rationalization prohibition):** PASS — all 74 Red Gate tests are non-`#[ignore]` unit tests. No `#[ignore]` in the delta; no deferred behaviors.

**POL-22 Phase A (ID/anchor integrity):** PASS — BC anchors (BC-2.11.024 v1.3, BC-2.02.013 v1.8, BC-2.10.012 v1.9, BC-2.16.002 v2.04, BC-2.11.001 v1.x) present in story v1.32. E-QUERY-002 Display forms verbatim. E-QUERY-001 mode-boundary anchor in sql_parser.rs. No stale BC version pins in production code.

**POL-22 Phase C (RGT inventory completeness):** PASS — all 74 RGT names (RG-001..RG-074) verified present in story v1.32. Red Gate count = 74 (UNCHANGED). Workspace test count = 5310 (UNCHANGED).

**TD-VSDD-059 (load-bearing test verification):** PASS — all prior closures remain load-bearing at de89b557. INFO-2 (non-spec-pinned message) not a closure; no load-bearing check required. RG-074 (`valid_operators_for_type` 8-operator contract) confirmed GREEN.

**TD-VSDD-091 (no volatile pins):** PASS — zero numeric row-index annotations, zero versioned BC pins, zero volatile line-number citations in 44-file delta production code at de89b557.

**Novelty:** NONE — zero findings. The two informational notes (INFO-1, INFO-2) are documentation-context observations within known design choices, not new defect classes.

---

## Summary

Pass 34 is CLEAN (strict and PR-merge). Feature HEAD de89b557 carries zero findings across the 44-file delta. Two informational notes (INFO-1: BC-2.16.002 detail-string sub-note CWE-209 editorial; INFO-2: IEQ/INE messages intentionally non-spec-pinned) recorded for session context only — neither constitutes a streak-blocking finding under BC-5.39.001. The LOCAL 3-CLEAN(strict) streak advances to 2/3.

**NEXT ACTION:** LOCAL adversary pass-35 on same frozen HEAD de89b557 (streak candidate 3/3). Per BC-5.39.001 and DRIFT-ORCH-PRLEVEL-PUSH-001, no commits may land on the feature branch between pass-34 and pass-35.
