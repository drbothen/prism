---
pass: 46
lane: DEFECT-PQL-FNCALL-LHS-001
frozen_head: 973aedcf
date: 2026-07-14
authored_by: orchestrator-relay
clean_strict: false
clean_pr_merge: false
finding_count: 1
streak_before: 0/3
streak_after: 0/3
status: CLOSED
fix_burst: fix-burst-35
fix_burst_pending: false
fix_burst_spec_only: true
fix_burst_bc: []
---

# LOCAL Adversary Pass 46 — DEFECT-PQL-FNCALL-LHS-001

**Frozen HEAD: 973aedcf** (fix/DEFECT-PQL-FNCALL-LHS-001; LOCAL-ONLY; UNCHANGED since fix-burst-33; pass-46 is the second pass on this HEAD)
**CLEAN(strict): NO** (1 finding: 1 MED — CLOSED by fix-burst-35)
**CLEAN(PR-merge): NO** (1 MED finding present; MED severity blocks PR-merge gate)
**Streak: 0/3** (UNCHANGED — pass-46 NOT CLEAN(strict); BC-5.39.001 streak-reset rule; streak remains 0/3; pass-47 opens fresh on same frozen 973aedcf)
**Fix-Burst-35:** COMPLETE (spec-only; BC-INDEX v8.24→v8.25; echo corrections in pass-30 + pass-42; Lesson 57 extended; HEAD 973aedcf UNCHANGED; pass-47 gates on same frozen 973aedcf)

---

## Pass-45 Closure Verification

Pass-45 had 1 finding (F-PQLFN-P45-LOW-001 POL-23 sweep miss: EC-ID grep vs version-pin-string grep), CLOSED in fix-burst-34 (spec-only). Closure VERIFIED SUBSTANTIVE:

- **F-PQLFN-P45-LOW-001 — SUBSTANTIVE:** BC-2.11.004 v1.47 §Changelog story-sweep sentence corrected in place (version-pin sweep rationale, not EC-ID grep rationale). S-PRISMQL-CASE-INSENSITIVE-001 v1.71→v1.72: 4 BC-2.11.004 v1.46→v1.47 pin sites updated (frontmatter comment; §BC body table version cell; §Token Budget row; AC-013b trace). STORY-INDEX v2.684→v2.685. Post-fix exhaustive grep: zero `BC-2.11.004 v1.46` live pins remain.

---

## Findings

### F-PQLFN-P46-MED-001 [MED][fact-error, POL-4/POL-22] — OPEN → CLOSED (fix-burst-35)

**Affected artifacts:**
- `specs/behavioral-contracts/BC-INDEX.md` — 2 sites:
  - Line ~165: BC-2.11.004 row v1.42 changelog note
  - Line ~440: v8.11 BC-INDEX changelog entry

**Finding:** BC-INDEX.md carries a FABRICATED LOW-006 reserved-keyword list at 2 sites. Both sites claim the 20-keyword list is `NOT/AND/OR/XOR/IN/LIKE/ILIKE/BETWEEN/IS/NULL/TRUE/FALSE/CAST/CASE/WHEN/THEN/ELSE/END/EXTRACT/INTERVAL` (SQL DDL keywords — XOR, ILIKE, NULL, TRUE, FALSE, CAST, CASE, WHEN, THEN, ELSE, END, EXTRACT, INTERVAL are SQL constructs absent from PrismQL).

**Ground truth** (read verbatim from `crates/prism-query/src/filter_parser.rs` RESERVED_KEYWORDS const, lines 1492–1496):
```rust
const RESERVED_KEYWORDS: &[&str] = &[
    "NOT", "AND", "OR", "IN", "IIN", "IEQ", "INE", "IS", "BETWEEN",
    "LIKE", "CIDR", "MATCHES", "HAS", "MISSING", "CONTAINS",
    "ICONTAINS", "STARTSWITH", "ISTARTSWITH", "ENDSWITH", "IENDSWITH",
];
```
Actual 20 keywords: NOT, AND, OR, IN, IIN, IEQ, INE, IS, BETWEEN, LIKE, CIDR, MATCHES, HAS, MISSING, CONTAINS, ICONTAINS, STARTSWITH, ISTARTSWITH, ENDSWITH, IENDSWITH.

This matches BC-2.11.004 v1.47 body LOW-006 §Error Cases (live spec body at frozen HEAD 973aedcf), the v1.42 body changelog row ~line 169, and the EC-11-004-006 cell ~line 116.

**Evidence chain — fabricated list also echoed in prior pass reports:**
- `local-pass-30.md` line ~48: Phase C "Reserved keyword gate — byte-matched" section echoed the fabricated SQL-DDL list as if byte-matched against source.
- `local-pass-42.md` line ~71: Positive Verifications "LOW-006 keyword-list exhaustiveness" bullet echoed the fabricated SQL-DDL list with "NULL is correctly included" claim (factually wrong — NULL is absent from actual RESERVED_KEYWORDS; the bullet header "(NULL correctly omitted)" was itself inconsistent with the body text, reflecting the fabrication origin).

Both echo sites were authored by orchestrator-relay from memory rather than quoted verbatim from adversary output or a fresh codebase grep — consistent with Lesson 57 cross-lane fabrication class (D-1767).

**Root cause:** Relay/state-manager authoring of value lists without verbatim grep verification — same class as Lesson 57 MCP pass-22/23 SAP-1 enumeration fabrication.

**Fix (fix-burst-35, state-manager; spec-only):**

1. **BC-INDEX.md — 2 in-place corrections:** Both occurrences of the fabricated SQL-DDL list replaced with the verbatim RESERVED_KEYWORDS list from `filter_parser.rs` lines 1492–1496 + marker `[narrative-only correction per F-PQLFN-P46-MED-001; no contract change]` appended. BC-INDEX frontmatter `version: "8.24"→"8.25"`. BC-INDEX v8.24→v8.25 changelog entry added (D-1768).

2. **local-pass-30.md ~line 48 — in-place correction:** Phase C "byte-matched" list replaced with verbatim RESERVED_KEYWORDS + marker `[narrative-only correction per F-PQLFN-P46-MED-001; original list was not derived from grep]`.

3. **local-pass-42.md ~line 71 — in-place correction:** "LOW-006 keyword-list exhaustiveness" bullet: fabricated list + "NULL is a reserved keyword and is correctly included" sentence replaced with verbatim RESERVED_KEYWORDS + marker. Bullet header updated to remove the incoherent "(NULL correctly omitted)" parenthetical (it was internally inconsistent with the body's "NULL is correctly included" claim; both arose from fabrication).

4. **Lesson 57 extension:** PQL BC-INDEX LOW-006 instance codified per adversary ask — state-manager narratives citing implementation-specific value lists (keyword arrays, constant enumerations, function-name lists) must be sourced verbatim from code or BC body. Tagged [process-gap].

**TD-VSDD-060 sibling sweep evidence (adversary pre-verified; state-manager reconfirmed):**

Sweep targets: XOR/ILIKE adjacent to LOW-006 context; CAST/CASE/WHEN markers:
```
grep -rn "NOT/AND/OR/XOR\|ILIKE\|CAST/CASE\|EXTRACT/INTERVAL" \
  .factory/specs/ .factory/stories/ (excluding .svg/.drawio/.png/.pdf)
```

Results:
- `BC-INDEX.md`: 2 sites — the 2 corrected sites above (confirmed carriers)
- `specs/day2-ui-design/mockups/S2-10-saved-queries.html`: 2 `ILIKE` SQL syntax occurrences in UI mockup — excluded (UI syntax, not LOW-006 list)
- `stories/S-PRISMQL-CASE-INSENSITIVE-001-*.md`: 4 `ILIKE` operator-prose occurrences referencing ADR-047 context — excluded (operator semantics prose, not LOW-006 list)
- ADR-009 XOR-seed crypto references — excluded (unrelated cryptographic usage)
- ADR-047/BC-2.11.024 ILIKE-as-operator prose — excluded (ILIKE as operator, not LOW-006)

**Zero additional carriers beyond BC-INDEX 2 sites confirmed.**

**Status:** CLOSED — fix-burst-35 COMPLETE. BC-INDEX v8.25 with 2 in-place corrections. Pass-30 + pass-42 echo corrections applied. Lesson 57 extended. HEAD 973aedcf UNCHANGED. Pass-47 gates on same frozen 973aedcf.

---

## SAP-1 Result

**PASS.** SAP-1 from pass-45 carried forward (no code change at frozen HEAD 973aedcf between pass-45 and pass-46). 55 raw prism-query occurrences of `event_type\s*=`; 12 distinct live production emission values; all 12 catalogued in BC-2.16.002 §Postconditions Canonical Structured Event Catalog. Settled counting methodology documented in pass-45 SAP-1 section. No new or removed emission sites in pass-46 window (fix-burst-35 spec-only; zero crate changes).

---

## Positive Verifications

- **Seven-position walker verified (lines 1930–2107, all 7 positions per ADR-048 §D.7.1):** All seven predicate positions gated by the fn_call_comparison walker confirmed at frozen HEAD 973aedcf. Position 7 (INSERT source_select WHERE — OD-7) added in fix-burst-24; gate present at engine.rs ~lines 1930–2107. ADR-048 v1.15 §D.7.1 attribution correct.

- **LOW-006 gate implemented correctly at filter_parser.rs 1492–1508:** RESERVED_KEYWORDS const at lines 1492–1496 contains the actual 20 PrismQL predicate-operator keywords. Case-insensitive check via `.eq_ignore_ascii_case(kw)` at line 1499. E-QUERY-001 emitter at lines 1501–1506 with exact template. Gate is load-bearing (TD-VSDD-059): the HashSet-style `any()` check fires before the fn_call_comparison production can succeed, structurally blocking reserved keywords from being parsed as fn-names.

- **ADR-048 v1.15 §D.7.1 attribution correct:** BC-2.11.004 §Error Cases LOW-006 and the corresponding code gate are both attributed to ADR-048 §D.7 in the current live BC body. ADR-048 v1.15 is the live version at 973aedcf (13 pin sites in BC-2.11.004 all updated in fix-burst-32; no stale v1.13 or earlier pin remains).

- **BC-2.11.019 OBS-001 InSubquery fail-open documented:** BC-2.11.019 §Postconditions notes the InSubquery arm fail-open behavior (plan-time rejection via E-QUERY-043; not a fn_call_comparison interaction). No regression from fix-burst-35 (spec-only; no code change).

- **Injection-safety clause correct:** BC-2.11.004 §Postconditions `sanitize_for_log` clause at the `EnrichUdfNotFoundDetails` construction site verified present and accurate. Load-bearing at construction (TD-VSDD-059): sanitization occurs before the value is ever stored or propagated.

- **POL-22 Phase A+C pass on §Postconditions v1.47:** BC-2.11.004 v1.47 body code-truth check: Phase A (BC prose → code match) and Phase C (code path → BC coverage) both PASS at frozen 973aedcf. Fix-burst-35 (spec-only) makes no changes to BC-2.11.004 body; v1.47 behavioral promises remain accurate against codebase.

- **Novelty assessment:** MEDIUM — fabricated keyword list in BC-INDEX is a POL-4/POL-22 fact-error (index document must accurately describe BC content); structural correctness of the LOW-006 gate in BC-2.11.004 body and filter_parser.rs is unaffected; the index narrative was wrong but the implementation and primary BC body were correct.

---

## Status

```
NOT CLEAN(strict) — pass 46 complete. 1 finding (1 MED). CLOSED fix-burst-35.

CASCADE TALLY: 46 passes / 35 fix-bursts

STREAK: 0/3 (UNCHANGED — pass-46 NOT CLEAN(strict); BC-5.39.001 streak-reset rule)
DRIFT-ORCH-PRLEVEL-PUSH-001: feature branch fix/DEFECT-PQL-FNCALL-LHS-001 is LOCAL-ONLY.
Frozen HEAD for pass-47: 973aedcf UNCHANGED (fix-burst-35 spec-only; no code commit).

FIX-BURST-35 STATUS (COMPLETE; spec-only):
  F-PQLFN-P46-MED-001: CLOSED — BC-INDEX v8.24→v8.25 (2 fabricated list corrections +
                                 v8.25 changelog entry D-1768). local-pass-30.md ~line 48
                                 + local-pass-42.md ~line 71 corrected in-place. Lesson 57
                                 extended with PQL BC-INDEX LOW-006 instance.

FINDINGS BREAKDOWN:
  MED: 1 (F-PQLFN-P46-MED-001 BC-INDEX fabricated LOW-006 keyword list) — CLOSED
  Total: 1

CLEAN(strict): NO (pass-46 finding drove fix-burst-35; pass-47 gates on same frozen HEAD)
CLEAN(PR-merge): NO (1 MED finding present)

NEXT ACTION: LOCAL pass-47 on SAME frozen HEAD 973aedcf (streak 0/3; BC-INDEX v8.25 live;
             HEAD UNCHANGED — fix-burst-35 spec-only)
```
