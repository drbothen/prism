---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: DEFECT-MCP-ROWSHAPE-NULLS-001
passes: [22]
feature_head_at_review: 5d2624aa
date: 2026-07-14
clean_strict: false
clean_pr_merge: false
finding_counts:
  total: 1
  crit: 0
  high: 0
  med: 1
  low: 0
  obs: 0
  process_gap: 0
  out_of_scope_obs: 0
code_behavior_defects: 0
streak_after: 0/3
convergence: NOT_CONVERGED
status: CLOSED
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 22 — DEFECT-MCP-ROWSHAPE-NULLS-001

---

## Pass 22 (frozen 5d2624aa; fresh-context adversary; PR #222 MCP row-shape null serialization cascade; streak 0/3 → RESET 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO

Streak: **0/3** (RESET — 1 MED finding; BC-5.39.001: any finding resets streak regardless of fix-burst type)

---

## Findings

### F-MCPRS-PRL22-MED-001 [MED][BC-INDEX summary-row truth — 3rd instance of class] — CLOSED (D-1763 BC-INDEX v8.23 fix-burst)

**Severity:** MED
**Classification:** spec-drift/index-truth — BC-INDEX governance discovery surface (same class as F-MCPRS-PRL20-MED-001 BC-2.15.009 pass-20 and F-MCPRS-PRL21-MED-001 BC-2.11.018 pass-21)

**Finding:** BC-INDEX.md BC-3.2.001 Status cell v0.10 parenthetical states "null-not-absent row isolation check" — this is FABRICATED. A grep of BC-3.2.001 for the phrase "null-not-absent" returns zero hits. A grep for "row isolation check" returns zero hits. The concept "null-not-absent row isolation check" belongs exclusively to BC-2.11.001 (the `query` MCP tool response contract for WriterBuilder null serialization). BC-3.2.001's actual v0.10 change (per its own §Changelog row) was a POL-25/POL-29 sweep that updated postcondition 5's caller-visible internal-error message from the retired all-in-one `"Internal error; see audit log"` format to the BC-2.10.007 message/suggestion split contract (`message="Internal error"`, `suggestion="See audit log for details."`), as part of the DEFECT-MCP-ROWSHAPE-NULLS-001 [H8b] burst on 2026-07-13.

**Impact:** BC-INDEX is the governance-critical discovery surface read at every session start and at every adversary review gate. An agent or reviewer reading BC-INDEX to understand BC-3.2.001's version history would conclude that v0.10 introduced a "null-not-absent row isolation check" into the per-org data isolation contract — which is false. This creates an incorrect attribution linking null serialization semantics to the cross-tenant isolation contract, and misrepresents which aspect of DEFECT-MCP-ROWSHAPE-NULLS-001 touched BC-3.2.001. The [H8b] fix was an error-message format update (message/suggestion split), not an isolation invariant change. Contract semantics (E-QUERY-032 error surface for cross-org queries) were explicitly unchanged in v0.10.

**Context — third instance of class:**
- Pass 20: F-MCPRS-PRL20-MED-001 — BC-2.15.009 v1.7 described as "null-not-absent constraint propagation" (actually: 3-site cosmetic rows-terminology sweep). CLOSED D-1759 (BC-INDEX v8.20).
- Pass 21: F-MCPRS-PRL21-MED-001 — BC-2.11.018 v1.5 described as "normalized_pql echo interaction with null-not-absent constraint" (actually: F-MCPNULL-P3-MED-001 phantom-field sweep). CLOSED D-1762 (BC-INDEX v8.22).
- Pass 22: F-MCPRS-PRL22-MED-001 — BC-3.2.001 v0.10 described as "null-not-absent row isolation check" (actually: POL-25/POL-29 sweep updating §Postconditions.5 internal-error message format). CLOSED D-1763 (BC-INDEX v8.23).

All three instances share the same root cause: when multiple BCs are updated in the same cascade burst, state-manager BC-INDEX row authoring cross-contaminates summary descriptions with the dominant semantic theme of the burst ("null-not-absent") rather than reading the specific BC's §Changelog row. This is a process-gap defect, not a random error — the paraphrase pattern is structurally predictable.

**Fix plan:** BC-INDEX v8.22→v8.23: rewrite BC-3.2.001 row Status cell v0.10 parenthetical to extractive restatement of BC-3.2.001 §Changelog v0.10 row (every noun phrase verified against BC body). Conduct exhaustive cascade-touched BC-INDEX audit to confirm no 6th row.

**Status:** CLOSED — D-1763 fix-burst (state-manager; BC-INDEX v8.22→v8.23; BC-3.2.001 row rewritten to changelog ground truth; spec-only; PR HEAD 5d2624aa UNCHANGED). Pass-23 gates on same frozen 5d2624aa.

**Streak note:** Streak RESET 0/3 per BC-5.39.001 (1 MED finding present at pass-22 gate; DRIFT-ORCH-PRLEVEL-PUSH-001: no push to feature branch 5d2624aa).

---

## Positive Verifications

- **F-MCPRS-PRL21-MED-001 closure re-verified (TD-VSDD-059 pass):** BC-INDEX v8.22 BC-2.11.018 row rewrite (D-1762) confirmed substantively correct. BC-2.11.018 v1.5 parenthetical now reads "F-MCPNULL-P3-MED-001 phantom-field sweep (POL-25 full-file): four phantom envelope fields corrected in §Postconditions structured response example" — extractive restatement verified against BC-2.11.018 §Changelog v1.5. No regression from D-1762 fix.

- **EC-11-079 null-not-absent path verified load-bearing:** `test_BC_2_11_001_EC_11_079` exercises the WriterBuilder `.with_explicit_nulls(true)` path at 5d2624aa. Green and load-bearing (not a doc-comment or assertion-only paper fix; TD-VSDD-059 satisfied for EC-11-079).

- **EC-11-081 NaN/±Inf locking test verified:** `test_BC_2_11_001_EC_11_081_nonfinite_float_serializes_as_json_null` exercises the arrow-json 58.2.0 NaN/±Inf→null boundary at 5d2624aa. Green; 481/481 prism-mcp GREEN.

- **CI gate ordering (.prx extension guard) verified:** CI .prx platform gate executes BEFORE the MCP tool dispatch; E-QUERY-004 (.prx extension rejected) is consistent across all 5d2624aa CI runs. No regression.

- **Four-of-five cascade-touched BC-INDEX rows verified accurate (exhaustive audit):**
  - BC-2.10.007 (line 151) v1.19: verified accurate (pass-22 independent read; row describes CursorCapExceeded category correction + 28 explicit VariantMeta arms; matches BC-2.10.007 §Changelog v1.19).
  - BC-2.11.001 (line 162) v1.22: verified accurate (pass-22 independent read; row describes EC-11-081 Float64 NaN/±Inf arrow-json boundary codification; matches BC-2.11.001 §Changelog v1.22; no cross-citation confusion with BC-2.11.018 echo path).
  - BC-2.11.018 (line 179) v1.5: verified accurate (D-1762 F-MCPRS-PRL21-MED-001 closure confirmed; v1.5 phantom-field sweep correctly described).
  - BC-2.15.009 (line 232) v1.7: verified accurate (D-1759 F-MCPRS-PRL20-MED-001 closure confirmed; v1.7 rows-terminology sweep correctly described).
  - BC-3.2.001 (line 294) v0.10: **FINDING** F-MCPRS-PRL22-MED-001 — CLOSED this burst (D-1763 BC-INDEX v8.23).
  - **No 6th cascade-touched row found:** grep of BC-INDEX for "DEFECT-MCP-ROWSHAPE-NULLS-001" across all active-BC rows returns exactly 5 hits (lines 151/162/179/232/294); changelog/archive-section occurrences are not BC-row Status cells.

- **SAP-1 PASS:** Grep of `event_type =` across `crates/` workspace at 5d2624aa: 12 unique `event_type` values found [narrative-only correction per pass-24 SAP-1 revision; original enumeration was not derived from grep — see Lesson 57; pass-24 revised SAP-1: 84 unique in-scope production event_type values verified against BC-2.16.002 catalog v1.61 (92 rows); real prism-mcp values include mcp.tool.rejected, mcp.tool.called, mcp.server.shutdown.*, schema_enumeration.* — not the fabricated list in original authoring]. Verified present in BC-2.16.002 §Postconditions Canonical Structured Event Catalog. Zero emission sites without catalog row. SAP-1 PASS.

- **SAP-2 N/A:** No sensor TOML spec or DTU type changes in scope of 5d2624aa.

---

## Summary

**CLEAN(strict): NO** (1 MED finding — BC-INDEX summary-row truth drift for BC-3.2.001 v0.10; 3rd instance of class; F-MCPRS-PRL22-MED-001)
**CLEAN(PR-merge): NO** (1 MED finding; CLOSED D-1763 BC-INDEX v8.23 fix-burst)

Streak: **0/3** (RESET — F-MCPRS-PRL22-MED-001 MED finding present at pass gate; BC-5.39.001 streak-reset rule; spec-only fix does not prevent reset)

Note: F-MCPRS-PRL22-MED-001 CLOSED — D-1763 fix-burst (BC-INDEX v8.22→v8.23; BC-3.2.001 row rewritten to changelog ground truth; spec-only; PR HEAD 5d2624aa UNCHANGED). F-MCPRS-PRL21-MED-001 closure re-verified substantive (TD-VSDD-059 PASS). Process-gap Lesson 55 appended (3 consecutive same-class instances: passes 20/21/22).

CASCADE TALLY: 42 passes / 27 fix-bursts. Frozen HEAD @5d2624aa UNCHANGED; streak 0/3 (RESET); next = PR-LEVEL pass 23 on same frozen 5d2624aa.
