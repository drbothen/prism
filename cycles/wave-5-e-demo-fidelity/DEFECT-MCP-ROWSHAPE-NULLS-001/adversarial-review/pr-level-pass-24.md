---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: DEFECT-MCP-ROWSHAPE-NULLS-001
passes: [24]
feature_head_at_review: 5d2624aa
date: 2026-07-14
clean_strict: false
clean_pr_merge: true
finding_counts:
  total: 1
  crit: 0
  high: 0
  med: 0
  low: 1
  obs: 0
  process_gap: 0
  out_of_scope_obs: 0
code_behavior_defects: 0
streak_after: 0/3
convergence: NOT_CONVERGED
status: CLOSED
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 24 — DEFECT-MCP-ROWSHAPE-NULLS-001

---

## Pass 24 (frozen 5d2624aa; fresh-context adversary; PR #222 MCP row-shape null serialization cascade; streak 1/3 → RESET 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=YES

Streak: **0/3** (RESET — 1 LOW finding; BC-5.39.001: any finding resets streak regardless of finding severity)

---

## Findings

### F-MCPRS-PRL24-LOW-001 [LOW][index-truth/filename-anchor drift, POL-13+POL-22 Phase C] — CLOSED (D-1767 STORY-INDEX v2.686 spec-only fix-burst)

**Severity:** LOW
**Classification:** index-truth/filename-anchor drift — POL-13 (STORY-INDEX row registration accuracy) + POL-22 Phase C (post-registration accuracy maintenance)

**Finding:** STORY-INDEX.md line ~817, row for S-MCP-E003-SERIALIZATION-MIGRATION-001, cites `file: S-MCP-E003-SERIALIZATION-MIGRATION-001-mcp-serialization-error-migration.md` inside the draft v0.1 D-1729 registration parenthetical. Actual on-disk filename: `S-MCP-E003-SERIALIZATION-MIGRATION-001-mcp-serialization-error-code-migration.md` ("code-" segment missing from the cite). Wrong slug appears exactly once in the workspace (adversary grep-verified); 5+ other recently-registered rows spot-checked clean.

**Impact:** STORY-INDEX is the canonical story registry consulted at session start and at adversarial review gates. A `file:` field citing a non-existent slug breaks POL-13 row-accuracy and POL-22 Phase C anchor integrity. Any agent performing a story lookup via the STORY-INDEX `file:` field would reference a non-existent path.

**Fix plan:** STORY-INDEX line ~817: `mcp-serialization-error-migration.md` → `mcp-serialization-error-code-migration.md`. Bump STORY-INDEX version v2.685→v2.686 per POL-11 with extractive changelog entry.

**Status:** CLOSED — D-1767 spec-only fix-burst (state-manager; STORY-INDEX v2.685→v2.686; file: slug corrected; PR HEAD 5d2624aa UNCHANGED). Pass-25 gates on same frozen 5d2624aa.

**Streak note:** Streak RESET 1/3→0/3 per BC-5.39.001 (1 LOW finding present at pass-24 gate; DRIFT-ORCH-PRLEVEL-PUSH-001: no push to feature branch 5d2624aa).

---

## Positive Verifications

- **F-MCPRS-PRL22-MED-001 closure re-verified SUBSTANTIVE (TD-VSDD-059):** BC-INDEX BC-3.2.001 row (BC-INDEX v8.24) is a complete extractive restatement of BC-3.2.001 §Changelog v0.10. The fabricated phrase "null-not-absent row isolation check" is absent. Every noun phrase in the Status cell traces directly to the BC-3.2.001 §Changelog v0.10 row (POL-25/POL-29 sweep — §Postconditions.5 caller-visible internal-error message updated to BC-2.10.007 message/suggestion split contract). No reintroduction of prior fabricated content.

- **H8b catch-all fix verified load-bearing:** `map_prism_error` in `crates/prism-mcp/src/error_mapping.rs` at line 462 returns `(codes::INTERNAL_ERROR, "Internal error")`. The retired all-in-one form `"Internal error; see audit log"` is absent from all production paths. Lines 4267/4296/4313/4325 are Red Gate docstring citation sites only (test narrative, not production code). Fix is structural (production code path change), not doc-comment or assertion-only (TD-VSDD-059 satisfied).

- **EC-11-081 + EC-11-079 locks load-bearing:** `with_explicit_nulls(true)` at `crates/prism-mcp/src/server.rs` ~lines 1953–1955 is the sole `arrow_json` production site. `test_BC_2_11_001_EC_11_081_nonfinite_float_serializes_as_json_null` exercises the NaN/±Inf→null boundary. `test_BC_2_11_001_EC_11_079` exercises the null-not-absent WriterBuilder path. Both tests load-bearing (TD-VSDD-059 satisfied for EC-11-079 and EC-11-081).

- **All 5 cascade BC-INDEX rows re-verified accurate (exhaustive audit):**
  - BC-2.10.007 v1.19: verified accurate (extractive restatement of §Changelog v1.19; CursorCapExceeded category correction + 28 explicit VariantMeta arms; no drift).
  - BC-2.11.001 v1.22: verified accurate (EC-11-081 Float64 NaN/±Inf arrow-json boundary codification; no cross-citation confusion with BC-2.11.018 echo path).
  - BC-2.11.018 v1.5: verified accurate (D-1762 F-MCPRS-PRL21-MED-001 closure confirmed; v1.5 phantom-field sweep and v1.4 events→rows sweep correctly described).
  - BC-2.15.009 v1.7: verified accurate (D-1759 F-MCPRS-PRL20-MED-001 closure confirmed; v1.7 rows-terminology sweep correctly described).
  - BC-3.2.001 v0.10: verified accurate — D-1763 fix confirmed (F-MCPRS-PRL22-MED-001 closure substantive per pass-23 re-verification).

- **Story version pins current:** S-MCP-E003-SERIALIZATION-MIGRATION-001 v0.9 pins BC-2.10.007 v1.19 (consistent with BC-INDEX v8.24). S-TEST-WIRESHAPE-SWEEP-001 v0.20 pins BC-2.10.007 v1.19 + BC-2.11.001 v1.22 (consistent).

- **SAP-1 REVISED PASS:** Grep of `event_type =` across `crates/` workspace at 5d2624aa: 84 unique in-scope production `event_type` values verified against BC-2.16.002 §Postconditions Canonical Structured Event Catalog v1.61 (92 rows). 2 out-of-scope events governed by BC-2.04.013 (`capability_check`) and BC-2.03.010 (`credential_access`) per explicit scope carve-out. `boot.audit.initialized` governed by BC-2.05.012 (pre-existing scope partition). Zero in-scope emission sites without catalog row. **Process-gap note: passes 22 and 23 carried a fabricated 12-item enumeration (fetch.started, fetch.completed, fetch.partial_failure, query.started, query.completed, query.error, mcp.tool_call.started, mcp.tool_call.completed, mcp.tool_call.error, audit.write.attempted, audit.write.succeeded, audit.write.failed) that does not originate from a grep of the codebase — ZERO of these values exist in prism-mcp; real values include mcp.tool.rejected, mcp.tool.called, mcp.server.shutdown.*, schema_enumeration.*. Corrected in-place in passes 22/23 with narrative-only correction marker per Lesson 57.**

- **SAP-2 N/A:** No sensor TOML spec or DTU type changes in scope of 5d2624aa.

- **TD-VSDD-059 PASS:** All claimed closures from prior fix-bursts verified structural (production code path changes), not paper fixes.

- **TD-VSDD-091 PASS:** No line-number citations in narrative spec content added in this branch. Behavioral anchors (function names, event_type values, EC identifiers, error codes) used throughout.

---

## Novelty Assessment

**LOW.** One new process-gap class identified: relay/state-manager authoring of SAP-1 enumerations in pass reports without grepping the codebase. This manifested in passes 22 and 23, and cross-lane in PQL passes. Recorded as Lesson 57. The underlying code behavior findings (F-MCPRS-PRL24-LOW-001) are LOW-severity index-accuracy class.

---

## Summary

**CLEAN(strict): NO** (1 LOW finding — STORY-INDEX filename-anchor drift for S-MCP-E003-SERIALIZATION-MIGRATION-001; F-MCPRS-PRL24-LOW-001)
**CLEAN(PR-merge): YES** (1 LOW finding only; CLOSED D-1767 spec-only fix-burst)

Streak: **0/3** (RESET — F-MCPRS-PRL24-LOW-001 LOW finding present at pass gate; BC-5.39.001 streak-reset rule; spec-only fix does not prevent reset)

Note: F-MCPRS-PRL22-MED-001 closure re-verified SUBSTANTIVE (TD-VSDD-059 PASS). All 5 cascade BC-INDEX rows accurate. H8b fix load-bearing. EC-11-079+EC-11-081 locks load-bearing. SAP-1 REVISED PASS (84 unique values; passes 22/23 fabricated enumerations corrected in-place; Lesson 57 appended). SAP-2 N/A. TD-VSDD-059/091 PASS. Novelty LOW.

CASCADE TALLY: 44 passes / 28 fix-bursts. Frozen HEAD @5d2624aa UNCHANGED; streak 0/3 (RESET); next = PR-LEVEL pass 25 on same frozen 5d2624aa.
