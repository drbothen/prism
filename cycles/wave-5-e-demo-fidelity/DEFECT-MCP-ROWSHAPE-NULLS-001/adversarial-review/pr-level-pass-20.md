---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: DEFECT-MCP-ROWSHAPE-NULLS-001
passes: [20]
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

# PR-LEVEL Adversary Pass 20 — DEFECT-MCP-ROWSHAPE-NULLS-001

---

## Pass 20 (frozen 5d2624aa; fresh-context adversary; PR #222 MCP row-shape null serialization cascade; streak 2/3 → RESET 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO

Streak: **0/3** (RESET from 2/3 — 1 MED finding; BC-5.39.001: any finding resets streak regardless of fix-burst type)

---

## Findings

### F-MCPRS-PRL20-MED-001 [MED][spec-drift/index-truth] — CLOSED this burst (BC-INDEX v8.19→v8.20)

**Severity:** MED
**Classification:** spec-drift/index-truth — BC-INDEX governance discovery surface

**Finding:** BC-INDEX.md line ~232, BC-2.15.009 Status cell parenthetical (at `draft — v1.7 (D-1718 2026-07-13: ...`) claimed: _"DEFECT-MCP-ROWSHAPE-NULLS-001 cascade — null-not-absent constraint propagation to context decorator; explicit_nulls interaction; v1.6 intermediate"_

This description is factually wrong on both the v1.7 and v1.6 accounts:

**v1.7 misdescribed.** The actual BC-2.15.009 v1.7 change (per BC-2.15.009 changelog row, burst `DEFECT-MCP-ROWSHAPE-NULLS-001-pass3-rows-terminology`, authored by product-owner 2026-07-13) is a 3-site cosmetic rows-terminology sweep under F-MCPNULL-P3-LOW-001: (a) §Canonical Test Vectors happy-path row: "All virtual fields in events" → "All virtual fields in result rows"; (b) §Canonical Test Vectors cross-client row: "Each event row has its own `_client` value" → "Each result row has its own `_client` value"; (c) §Edge Cases EC-15-033: "virtual field in event row, queryable" → "virtual field in result row, queryable". POL-25 full-file sweep performed; all other sites clean per changelog. **No postcondition semantics were changed.** The concepts "null-not-absent constraint propagation" and "explicit_nulls interaction" belong exclusively to BC-2.11.001 (the `query` MCP tool response contract), not BC-2.15.009 (the context decorator injection contract).

**v1.6 misdescribed as "intermediate."** The actual BC-2.15.009 v1.6 change (per BC-2.15.009 changelog row, burst `DEFECT-MCP-ROWSHAPE-NULLS-001-events-to-rows`, 2026-07-13) is: POL-23 sibling sweep — §Postconditions JSON response example key renamed from `"events": [...]` to `"rows": [...]` (alignment with BC-2.11.001 v1.17 and shipped behavior; F-MCPNULL-P1-MED-001 closure per human adjudication 2026-07-13). One site. This is a standalone complete change, not an "intermediate" step toward anything.

**Impact:** BC-INDEX is the governance-critical discovery surface consulted at every session start and adversary review gate. The v1.7 summary invites incorrect downstream reasoning: any agent or reviewer reading BC-INDEX to understand BC-2.15.009's version history would conclude that v1.7 introduced null serialization logic changes to the context decorator injection mechanism — which is false. This creates incorrect dependency tracking between DEFECT-MCP-ROWSHAPE-NULLS-001 cascade changes and BC-2.15.009, and misattributes the cascade's impact surface.

**Status:** CLOSED this burst — BC-INDEX v8.19→v8.20: line-232 BC-2.15.009 parenthetical rewritten to changelog ground truth. PR HEAD 5d2624aa UNCHANGED (spec-only fix in factory-artifacts; no feature-branch commit). Pass-21 gates on same frozen 5d2624aa.

**Streak note:** Streak RESET 0/3 per BC-5.39.001 (1 MED finding present at pass-20 gate; DRIFT-ORCH-PRLEVEL-PUSH-001: no push to feature branch 5d2624aa).

---

## Positive Verifications

- **Full scope-foreign diff audit confirmed clean:** Scope-foreign reference audit across 13 files in this PR diff that reference DEFECT-MCP-ROWSHAPE-NULLS-001 or cascade-related artifact IDs (E-QUERY-042, E-QUERY-043, CursorCapExceeded, EC-11-079, EC-11-081, VariantMeta, RetryableCategory). All 13 files are in-scope for this cascade. E-QUERY-042 and E-QUERY-043 references in the diff are inherited from base lineage (develop@5f1b5771 prior PRs #219 and #221) — not scope-foreign introductions by this PR.

- **SAP-1 PASS (~85 unique values; catalog exemptions verified):** `event_type =` emission sweep at 5d2624aa: ~85 unique values across ~230 sites; all catalogued in BC-2.16.002 §Postconditions Canonical Structured Event Catalog. `credential_access` (BC-2.03.010) exemption verified intentional per AD-017 opacity scope. `boot.audit.initialized` (BC-2.05.012) exemption verified intentional. Stale `timestamp_parse_failure` comment site = removal record per D-765 (event_type removed from codebase; no live uncatalogued emission).

- **Story frontmatter-body coherence confirmed:** S-TEST-WIRESHAPE-SWEEP-001 at v0.20 and S-MCP-E003-SERIALIZATION-MIGRATION-001 at v0.9 — frontmatter `version:` fields match the highest changelog row version in each story's body. No version drift between frontmatter and inline changelog.

- **ADR-051 §D2 verified:** BC-2.11.001 v1.22 §Postconditions correctly attributes EC-11-079 (null-not-absent invariant) to ADR-051 §D2 (source-returns-None precedent), not §D4 (null-input short-circuit). Citation chain coherent; EC-11-081 (NaN/±Inf→null) attributed to arrow-json 58.2.0 behavior boundary, orthogonal to ADR-051 §D4 typed-output scope. No cross-citation confusion.

- **RETRYABLE-503 byte-parity BC snippet vs code confirmed:** BC-2.10.007 v1.19 §RETRYABLE-503 §Implementer Code Follow-Up snippet uses `.code()` form. Verified against `server.rs` at 5d2624aa: the retryable status check uses the `.code()` accessor on `StatusCode`. Byte-parity confirmed; BC snippet accurately reflects the production code form adopted by the implementer.

---

## Summary

**CLEAN(strict): NO** (1 MED finding — BC-INDEX index-truth summary drift for BC-2.15.009 v1.7/v1.6)
**CLEAN(PR-merge): NO** (1 MED finding; CLOSED within this burst)

Streak: **0/3** (RESET from 2/3 — F-MCPRS-PRL20-MED-001 MED finding present at pass gate; BC-5.39.001 streak-reset rule; spec-only fix does not prevent reset)

Note: F-MCPRS-PRL20-MED-001 is a factory-artifacts spec fix (BC-INDEX v8.20); feature branch HEAD 5d2624aa UNCHANGED. Pass-21 gates on same frozen 5d2624aa. The finding originated in BC-INDEX state, not the feature branch code.

CASCADE TALLY: 40 passes / 27 fix-bursts. Frozen HEAD @5d2624aa UNCHANGED; streak 0/3 (RESET); next = PR-LEVEL pass 21 on same frozen 5d2624aa.
