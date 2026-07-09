---
document_type: adversarial-review
scope: LOCAL
fix_pr: FIX-IEQ-ERRPATH-001
passes: [11]
feature_head_at_review: e5170899
fix_burst_head: e5170899
date: 2026-07-09
clean_strict: true
clean_pr_merge: true
finding_counts:
  total: 0
code_behavior_defects: 0
streak_after: 1/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# LOCAL Adversary Pass 11 — FIX-IEQ-ERRPATH-001

---

## Pass 11 (frozen e5170899; fresh-context adversary; fix-PR IEQ non-existent column error path; streak candidate 2/3)

**Pass result:** CLEAN(strict)=YES, CLEAN(PR-merge)=YES

**Findings:** 0 (zero CRIT/HIGH/MED/LOW/OBS/PROCESS-GAP — verdict arithmetic valid; no findings of any severity)

**Code HEAD at review:** e5170899 (frozen; D-1621 last-segment fallback seeding with DERIVED provenance in branches (b) and (c) of compute_sqlpipe_head_binding; head-seeding only; 5369/5369 GREEN; non-exhaustive 89/89)

**Fix-burst HEAD:** e5170899 (UNCHANGED — no fix-burst required; CLEAN strict on first attempt)

**LOCAL 3-CLEAN(strict) streak after pass-11:** 1/3 (first consecutive CLEAN(strict) pass on frozen e5170899)

---

## Coverage Summary — All 14 Positions + Full Edge-Case Catalog

The adversary performed a full fresh-context trace of all 14 gate positions defined in BC-2.11.016 v1.17 and the complete EC-11-039..069 edge-case catalog at frozen e5170899.

**Position coverage results:**

| Position | Description | Result |
|----------|-------------|--------|
| 1 | Unqualified bare Field head SELECT item | PASS — seeded as-is (Branch a) |
| 2 | FROM-table-qualified head SELECT item | PASS — seeded via Branch (b) last-segment |
| 3 | FROM-alias-qualified head SELECT item | PASS — seeded via Branch (b) last-segment with from_alias threaded |
| 4 | Join-alias-qualified head SELECT item (EC-11-069) | PASS — seeded via Branch (c) LAST-SEGMENT rule DERIVED provenance |
| 5 | `AS alias` expression head SELECT item | PASS — DERIVED alias seeded |
| 6 | `SELECT *` wildcard head | PASS — MIXED-STAR branch (c) EC-11-062..064 |
| 7 | MIXED-STAR `SELECT *, expr AS alias` | PASS — alias seeded alongside star expansion |
| 8 | Chained `| stats` aggregate (position 11) | PASS — aggregate-arg walk covers agg-arg positions |
| 9 | Enrich-head `| enrich` | PASS — E-QUERY-039 pre-gate ordering verified (gates before walk per BC-2.11.019) |
| 10 | SIBLING-GATE CONSISTENCY (shadow alias `AS severity`) | PASS — per-name RAW/DERIVED provenance check; no false E-QUERY-002 |
| 11 | FROM-alias threading both caller paths | PASS — table_alias threaded in both execute branches (direct + fallback) |
| 12 | EC-11-041 single-tenant zero-column gate | PASS — is_registered() disambiguation; registered+empty→E-QUERY-038; unregistered→fail-open |
| 13 | EC-11-069 join-qualified last-segment both shapes | PASS — `SELECT j.col FROM t JOIN other j ON t.id = j.id \| where col > 0` + multi-table alias variant both resolve |
| 14 | Gate ordering E-QUERY-037→038→039 | PASS — verified across both execute paths (direct + SqlPipe fallback) |

---

## DERIVED Provenance State Machine Verification

The adversary traced the full provenance state machine introduced across passes 2–10:

**stats (REPLACE):** `| stats count(*) AS n` replaces binding context entirely with aggregate output columns (REPLACE semantics). Downstream `| where n > 0` resolves via the replaced context. PASS.

**enrich (UNION):** `| enrich fieldname FROM table` unions the enrich output columns into the existing binding context (UNION semantics). Downstream references to both pre-enrich and enrich-output columns resolve. PASS.

**| fields (TRANSITION):** `| fields col1, col2` transitions binding context to the explicit field list (TRANSITION semantics; EC-11-066..068). Downstream pipe stages see only the transitioned set. PASS.

All three provenance transitions verified at e5170899 without deviation.

---

## SIBLING-GATE CONSISTENCY Skip Verification

When `SIBLING-GATE CONSISTENCY` is skipped (the condition where the per-name provenance lookup finds no entry for a name in the binding context), the gate fails open — no false E-QUERY-002. The adversary verified this at e5170899: a column name absent from the binding context due to legitimate scope (e.g., `| where col_from_enrich_not_in_head` after an enrich without a prior head projection of that column) falls through to a fail-open, not a false ColumnNotFound. PASS.

---

## FP-001 Static Probe Results (all six probes)

The adversary applied the full FP-001 false-positive probe suite at e5170899:

**Probe 1 — Shadow alias (`SELECT count(*) AS severity ... | where severity > 5`):** PASS — SIBLING-GATE CONSISTENCY per-name RAW/DERIVED provenance applies; `severity` is seeded as DERIVED; gate resolves correctly without false E-QUERY-002.

**Probe 2 — Join-qualified reference (`SELECT j.col FROM t JOIN other j ON t.id = j.id | where col > 0`):** PASS — EC-11-069 LAST-SEGMENT RULE seeds `col` as DERIVED from Branch (c); downstream `| where col` resolves via binding context; no false E-QUERY-038.

**Probe 3 — MIXED-STAR (`SELECT *, severity AS sev_alias | where sev_alias > 3`):** PASS — alias seeded alongside star expansion; no false E-QUERY-038 on `sev_alias`.

**Probe 4 — Chained stats (`| stats ... | stats ...`):** PASS — second `| stats` operates on the REPLACE-context from the first; aggregate-arg walk (position 11) resolves nested aggregate args.

**Probe 5 — Enrich union (`| enrich fieldname FROM table | where enrich_output_col IS NOT NULL`):** PASS — UNION semantics; enrich output column seeded; downstream `| where` resolves.

**Probe 6 — Suspension propagation (E-QUERY-039 pre-gate):** PASS — E-QUERY-039 gates `| enrich` before the walk; if the enrich spec is missing at plan-time, E-QUERY-039 fires and the walk does not proceed; no false ColumnNotFound from a suspended walk.

---

## Standing Probe Results

**SAP-1 (Structured Event Catalog — BC-2.16.002):** PASS — adversary grepped `event_type\s*=` across the entire `crates/` workspace (not just changed files). Three `column_not_found.rejected` sites found; all three have corresponding catalog rows in BC-2.16.002 §Postconditions (established in D-1618 per pass-7 SAP-1 verification). No new `event_type =` sites introduced since pass-10. No new catalog rows required.

**POL-24 (byte-verbatim EC-body):** PASS — EC-11-069 was added to BC-2.11.016 v1.17 in the pass-10 fix-burst (D-1621). The adversary verified the EC-11-069 entry carries full field schema, audit role, and recurrence policy matching the EC-11-039..068 body format. Byte-parity with the canonical EC template confirmed. No new EC entries required for pass-11 (CLEAN pass; no new behaviors).

**TD-VSDD-060 (sibling-site sweep):** PASS — pass-11 introduces no code changes; no signature changes, no constant changes. The sweep obligation from the pass-10 fix-burst (compute_sqlpipe_head_binding signature unchanged; columns_for_table callers swept at e5170899) was already satisfied. No new sweep required.

**TD-VSDD-091 (no volatile line-pin citations):** PASS — no new `file.rs:NNN` line-number citations introduced. All behavioral anchors cite function names and EC-NN-NNN identifiers, not line numbers.

---

## Production Discipline Checks

**`unwrap()` / `expect()` in non-test code:** PASS — no new `unwrap()` or `expect()` calls in production code paths. One `NonZeroUsize::new(N).unwrap()` in a const-eval context is the only unwrap at e5170899; this is the established const-eval exception (D-1610 precedent).

**`println!` in production code:** PASS — no `println!` calls in production code at e5170899. All diagnostic output uses `tracing::*!` structured fields.

**`#[non_exhaustive]` preserved:** PASS — no new public types introduced at e5170899. The non-exhaustive gate EXPECTED=89 is unchanged. All 89 registered `#[non_exhaustive]` public types remain intact.

**Story pins current:** PASS — BC-2.11.016 v1.17 propagated to all 4 carrier stories in the pass-10 fix-burst (D-1621): S-PRISMQL-CASE-INSENSITIVE-001 v1.46, S-DEMO-FIDELITY-REMEDIATION-001 v2.35, S-DEMO-PRISMQL-ONBOARDING-001-B v2.12, S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 v1.21. No further story pin rounds required for pass-11.

---

## DML Fail-Open Adjudication

The adversary identified that DML verbs (`INSERT`, `UPDATE`, `DELETE`) follow a separate validation path from the SqlPipe `| where` / `| stats` / `| enrich` path. DML fail-open behavior (DML queries that reference non-existent columns do not trigger E-QUERY-038 via the plan-time gate) is a pre-existing design choice: the write-verb validation path is owned by a separate spec surface (write-path BCs). This is not an FP-001 violation — FP-001 covers the SqlPipe pipe-stage gate only. The DML validation path is legitimately out of scope for FIX-IEQ-ERRPATH-001, which targets the read-path E-QUERY-038 plan-time gate. No finding raised.

---

## Convergence Assessment

**Trajectory:** 6 → 3 → 3 → 2 → 1 → 1 → 0 (per-pass total finding count across passes 1–11)

**Pattern:** Steady decay consistent with genuine convergence. The jump from 1 (pass-10) to 0 (pass-11) is the expected terminal step in a well-converged gate: the pass-10 finding (EC-11-069 join-qualified seeding gap) was a natural extension of the established seeding logic, not a sign of regression or systemic instability. Pass-11 is structurally sound as a streak-start.

**Novelty assessment:** NONE — pass-11 introduced no new behavioral deviations. All 14 gate positions, the full EC-11-039..069 catalog, the three provenance state machine transitions, the six FP-001 probes, SAP-1, POL-24, TD-VSDD-060, TD-VSDD-091, and all production-discipline checks returned PASS. The gate surface at e5170899 is stable.

**Streak status:** 1/3 (BC-5.39.001). NEXT: LOCAL adversary pass 12 on UNCHANGED HEAD e5170899 (fresh context, strict; per DRIFT-ORCH-PRLEVEL-PUSH-001 no commits may be pushed to the fix-branch between passes). If CLEAN(strict), pass 13 completes the 3-CLEAN → push branch → open fix-PR via pr-manager.
