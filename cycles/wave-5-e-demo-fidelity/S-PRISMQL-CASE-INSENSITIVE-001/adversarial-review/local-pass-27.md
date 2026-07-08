---
document_type: adversarial-review
scope: LOCAL
passes: [27]
story: S-PRISMQL-CASE-INSENSITIVE-001
feature_head: b341cdd7
fix_burst_head: null
date: 2026-07-08
clean_strict: false
clean_pr_merge: false
finding_counts: {HIGH: 1}
streak_after: 0/3
---

# LOCAL Adversary Pass 27 — S-PRISMQL-CASE-INSENSITIVE-001

---

## Pass 27 (frozen b341cdd7; fresh-context adversary; 44-file delta vs develop@ea714d14; streak candidate 2/3 — NOT CLEAN)

**Pass result:** CLEAN(strict)=NO (1 HIGH), CLEAN(PR-merge)=NO (HIGH finding blocks)
**Findings:** 1 (F-P27-HIGH-001 HIGH — CLOSED story-writer v1.30 — NO CODE CHANGE; feature HEAD b341cdd7 UNCHANGED)
**Code HEAD at review:** b341cdd7 (frozen)
**Fix-burst HEAD:** none — story-writer spec-only correction; no code committed; feature HEAD UNCHANGED b341cdd7
**LOCAL 3-CLEAN(strict) streak after pass-27:** 0/3 (RESET by HIGH finding; streak does NOT reset due to a push since code HEAD is unchanged — the streak resets because the pass was NOT CLEAN(strict))

---

## Finding Inventory

### F-P27-HIGH-001 (HIGH) — story AC-025 + Task 23 illustrated example_query as SQL-mode SELECT — unparseable per AC-023 / BC-2.11.024 v1.3 §Mode-Boundary Enforcement (POL-4)

**Finding:** Two sites in story v1.29 illustrated the `example_query` field using SQL-mode syntax, which the story's own AC-023 and BC-2.11.024 v1.3 §Mode-Boundary Enforcement explicitly rejects with `E-QUERY-001` at parse time:

**Site 1 — AC-025 (story line ~703):**
The acceptance criterion body for AC-025 included an illustrative `example_query` snippet:
```
SELECT * FROM <table> WHERE severity IEQ 'high'
```
This is SQL-mode DML syntax. Per AC-023 (BC-2.11.024 v1.3 §Mode-Boundary Enforcement), `IEQ`/`IIN`/`INE` operators are rejected in SQL/DML mode with `E-QUERY-001` at parse time. The actual implementation correctly emits:
```
FROM <table> | where severity IEQ 'high' | limit 50
```
(pipe-mode canonical form per BC-2.10.012 v1.9 pure-PQL invariant).

**Site 2 — Task 23 (story line ~1404):**
Task 23 ("Add IEQ example with OCSF casing note to `prism describe` output") contained a code block illustrating the correct output structure:
```
example_query: "SELECT * FROM crowdstrike_detections WHERE severity IEQ 'high'"
```
Again SQL-mode. The implementation produces:
```
example_query: "FROM crowdstrike_detections | where severity IEQ 'high' | limit 50"
```

**Why this is HIGH (not MED):** A literal re-implementer reading the story as the sole specification source would code `build_example_query` to produce `SELECT * FROM <table> WHERE severity IEQ 'high'`. This would cause RG-062 (`test_BC_2_10_012_build_example_note_query_parses_without_stripping`) and RG-028 (`test_BC_2_11_024_describe_output_includes_ieq_example_and_ocsf_casing_note`) to fail, and would produce a describe output that is unparseable by the PrismQL engine — a runtime regression. The defect is in the illustrative examples embedded in spec prose, not in the AC precondition/postcondition tables which are correctly specified.

**Survival history:** This defect survived five AC-025 amendments across story versions v1.7 (original describe note), v1.21 (example_note reframing), v1.24 (BC-2.10.012 v1.9 alignment), v1.25 (suppression-guard note), and v1.28 (OCSF casing prose refinements). Each amendment correctly addressed the contract semantics but left the illustrative SQL-mode example_query string in the prose example block uncorrected.

**POL-25 sweep audit:** Reviewed all 8 `SELECT`+case-insensitive-operator sites in the story. 6 sites are intentional SQL-mode rejection-test inputs in Red Gate test tasks (RG-037, RG-038, and 4 related tests); those are correctly framed as "given a SQL-mode query, the parser rejects it with E-QUERY-001" and must remain as SQL-mode to test the rejection path. Only the 2 illustrative example sites above were incorrect; they have been corrected.

**Severity:** HIGH — POL-4 spec accuracy (implementation-guiding prose); BC-2.11.024 v1.3 §Mode-Boundary Enforcement and BC-2.10.012 v1.9 pure-PQL invariant both violated in the illustrative example; a re-implementer following the prose would produce a regression.

**Closure:** CLOSED — story-writer v1.30 (this burst): both sites corrected to pipe-mode canonical form:
- AC-025 (~line 703): `SELECT * FROM <table> WHERE severity IEQ 'high'` → `FROM <table> | where severity IEQ 'high' | limit 50`
- Task 23 (~line 1404): `"SELECT * FROM crowdstrike_detections WHERE severity IEQ 'high'"` → `"FROM crowdstrike_detections | where severity IEQ 'high' | limit 50"`

NO code change. Feature HEAD b341cdd7 UNCHANGED. RG-062 and RG-028 continue GREEN (they test the implementation output which was already correct).

---

## SAP Probe Results (Pass 27, verified against b341cdd7)

**SAP-1 (tracing emission catalog completeness):** PASS — no change to `event_type` values in b341cdd7 delta. `ocsf.enum_label_unrecognized` dual sites match BC-2.16.002 catalog row 91. Catalog count UNCHANGED 91.

**SAP-2 (DTU↔TOML schema parity):** N/A — no sensor TOML or DTU changes in this delta.

**SID-1 (no-ignored-test rationalization prohibition):** PASS — all 74 Red Gate tests are non-`#[ignore]` unit tests; no external dependency waivers present.

**POL-22 Phase A (ID/anchor integrity):** PASS — all 8 BC anchors and E-QUERY-002 reference verified present in story v1.29 (pre-correction) and v1.30 (post-correction). The AC-025 correction is a prose example fix, not an AC contract change; no BC anchor is affected.

**POL-22 Phase C (RGT inventory completeness):** PASS-with-1-HIGH (the AC-025 mode mismatch). Story v1.30 corrects the prose; all 74 RGT names (RG-001..RG-074) verified present. All domain entities present. The 6 intentional SQL-mode rejection-test inputs in Red Gate task descriptions are correctly framed as rejection paths and left as-is.

**Novelty:** MEDIUM-HIGH — this finding category (spec-prose illustrative example contradicts the story's own AC and BC) is novel to this pass. Not a recurrence of any prior finding category. The defect survived 5 prior story amendments without detection because each amendment targeted contract semantics (precondition/postcondition tables) rather than prose examples. This represents a new class of spec-accuracy vulnerability: illustrative prose examples that trail behind contract amendments.

---

## Fix Summary (Story-Only; No Code)

| Site | Before (v1.29) | After (v1.30) |
|------|----------------|---------------|
| AC-025 ~line 703 | `SELECT * FROM <table> WHERE severity IEQ 'high'` | `FROM <table> \| where severity IEQ 'high' \| limit 50` |
| Task 23 ~line 1404 | `"SELECT * FROM crowdstrike_detections WHERE severity IEQ 'high'"` | `"FROM crowdstrike_detections \| where severity IEQ 'high' \| limit 50"` |

---

## Post-Fix-Burst State

- Feature HEAD: **b341cdd7** (UNCHANGED — story-only fix; no code committed)
- 1407/1407 prism-query tests GREEN (UNCHANGED)
- non-exhaustive: 89/89 UNCHANGED
- RG-001..074 GREEN (UNCHANGED)
- LOCAL 3-CLEAN(strict) streak: **0/3** (RESET by F-P27-HIGH-001 finding in this pass)
- Novelty: MEDIUM-HIGH (spec-prose illustrative example contradiction — new finding class)
- NEXT ACTION: LOCAL adversary pass-28 on frozen b341cdd7 with story v1.30 (streak candidate 1/3)
