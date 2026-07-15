---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: DEFECT-MCP-ROWSHAPE-NULLS-001
passes: [27]
feature_head_at_review: 5d2624aa
date: 2026-07-14
clean_strict: true
clean_pr_merge: true
finding_counts:
  total: 0
  crit: 0
  high: 0
  med: 0
  low: 0
  obs: 0
  process_gap: 0
  out_of_scope_obs: 0
code_behavior_defects: 0
streak_after: 3/3
convergence: CONVERGED
status: CLOSED
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 27 — DEFECT-MCP-ROWSHAPE-NULLS-001

---

## Pass 27 (frozen 5d2624aa; fresh-context adversary; PR #222 MCP row-shape null serialization cascade; streak 2/3 → 3/3 CONVERGED)

**Pass result:** CLEAN(strict)=YES, CLEAN(PR-merge)=YES

Streak: **3/3 — CONVERGED** (BC-5.39.001: three consecutive CLEAN(strict) passes on unchanged frozen HEAD 5d2624aa; DRIFT-ORCH-PRLEVEL-PUSH-001 clean; HEAD UNCHANGED since pass-18; passes 25/26/27 consecutive CLEAN(strict))

Cascade tally: **47 passes / 28 fix-bursts**

---

## Findings

None.

---

## Positive Verifications

- **`with_explicit_nulls(true)` sole production site (server.rs 1953-1955):** develop baseline has ZERO occurrences — fix is real; no accidental removal or re-introduction of null suppression.

- **H8b catch-all load-bearing:** `error_mapping.rs` arms at lines 458/462 return `(codes::INTERNAL_ERROR, "Internal error")`; retired form only in test docstrings at lines 4267/4296/4313/4325.

- **EC-11-079/EC-11-081 locks load-bearing:** `server.rs` line ~1954 sole `with_explicit_nulls(true)` production site; JSON-semantic assertions, not string-match. EC-11-081 test carries arrow-json version-bump regression anchor at line 962.

- **McpSerializationError VariantMeta arm verified:** `error_mapping.rs` line 1789 carries the `McpSerializationError` arm with E-MCP-003 override; load-bearing test at line 4682.

- **117-variant sentinel compile-time enforced:** `error_category_coverage.rs` — compile-time guard against variant omission from the sentinel table remains intact.

- **STORY-INDEX slug correct (line 818):** `S-MCP-E003-SERIALIZATION-MIGRATION-001-mcp-serialization-error-code-migration.md` verbatim; file exists on disk (v0.9); wrong-slug residue = 5 occurrences, ALL narrative-only (STATE.md ×2, STORY-INDEX changelog ×1, pass-24 narrative ×2), zero in `file:` field positions.

- **SAP-1 PASS (fresh re-derivation):** 230 occurrences across 34 files; ~90 unique in-scope values corroborated against BC-2.16.002 v1.61 catalog (92 rows); carve-outs verified at file:line — `capability_check` @ `prism-security/src/flag_audit.rs:72` (BC-2.04.013), `credential_access` @ `prism-credentials/src/audit.rs:56/71` (BC-2.03.010), `boot.audit.initialized` @ `prism-bin/src/boot.rs:1857` (BC-2.05.012).

- **SAP-2 N/A.** CI four-layer committed-.prx defense intact (source-hash sidecar, wasm-tools validate pre-build, sidecar-ancestor check, manifest SHA-256 post-rebuild comparison; reachability gate + 20-min timeout). `hash-plugin-source.py` repo-root-anchored with loud-fail. Threatintel manifest byte-identical both locations (threat_intel v1.0.2, sidecar ac5bf335).

- **BC-INDEX v8.25 cascade rows accurate.** All 5 cascade BC-INDEX rows at v8.25 accurate; no regression.

- **TD-VSDD-059/091 PASS.** All prior cascade closures hold under fresh-context re-derivation. No paper-fix or volatile line-pin residue detected.

- **POL-22 Phase A+C PASS.** Novelty VERY LOW; zero new findings.

---

## Novelty Assessment

**VERY LOW.** Zero new findings. All prior cascade closures hold under fresh-context re-derivation. SAP-1 re-derivation (230 occurrences / 34 files; ~90 unique in-scope values) fully consistent with pass-26 result. Spec has converged for this defect scope.

---

## CONVERGENCE

**BC-5.39.001 CONVERGED — 3/3 consecutive CLEAN(strict) passes on frozen HEAD 5d2624aa (passes 25, 26, 27). DRIFT-ORCH-PRLEVEL-PUSH-001 clean — HEAD unchanged since pass-18. Cascade tally: 47 passes / 28 fix-bursts.**

**NEXT ACTION: HUMAN MERGE GATE for PR #222.**

### Human Merge-Gate Disclosure Items

The following items are disclosed verbatim for human review before merging PR #222:

1. **BC-2.11.019 §Injection-safety cross-branch gap:** If PR #222 merges before the PQL PR, develop carries the unenforced §Injection-safety subsection until the PQL PR lands (fix + load-bearing test live on the PQL branch at 3e0d3585). This is NOT a PR #222 regression — it is the pre-existing develop status quo. Must also surface at the PQL merge gate.

2. **PR #222 merge does NOT auto-promote BC-2.11.019 (POL-14 vehicle is the PQL PR).** BC-2.11.019 `draft_contracts` remains pending auto-promotion via the PQL PR's squash-merge, not this one.

3. **Non-blocking sequencing recommendation:** Merging the PQL PR first eliminates item 1 as a live gap on develop.

---

## Summary

**CLEAN(strict): YES** (zero findings)
**CLEAN(PR-merge): YES** (zero findings)

Streak: **3/3 — CONVERGED** (BC-5.39.001; third consecutive CLEAN(strict) pass on frozen HEAD 5d2624aa; DRIFT-ORCH-PRLEVEL-PUSH-001 clean; HEAD unchanged since pass-18; no push)

Note: `with_explicit_nulls(true)` sole production site (server.rs 1953-1955; develop baseline ZERO occurrences — fix is real). H8b catch-all load-bearing (error_mapping.rs lines 458/462). EC-11-081 test JSON-semantic with arrow-json version-bump regression anchor (line 962). McpSerializationError VariantMeta arm load-bearing (error_mapping.rs line 1789; E-MCP-003 override; test at 4682). 117-variant sentinel compile-time enforced (error_category_coverage.rs). STORY-INDEX slug correct (line 818). SAP-1 PASS (230 occurrences / 34 files; ~90 unique in-scope values; BC-2.16.002 catalog v1.61 92 rows; carve-outs confirmed at file:line). SAP-2 N/A. CI four-layer .prx defense intact; reachability gate intact; hash-plugin-source.py repo-root-anchored; threatintel manifest byte-identical (threat_intel v1.0.2, sidecar ac5bf335). BC-INDEX v8.25 cascade rows accurate. POL-22 Phase A+C PASS. TD-VSDD-059/091 PASS. Novelty VERY LOW.

CASCADE TALLY: 47 passes / 28 fix-bursts. Frozen HEAD @5d2624aa UNCHANGED; streak 3/3 CONVERGED per BC-5.39.001. NEXT: HUMAN MERGE GATE for PR #222 (see §CONVERGENCE disclosure items 1-3 above).
