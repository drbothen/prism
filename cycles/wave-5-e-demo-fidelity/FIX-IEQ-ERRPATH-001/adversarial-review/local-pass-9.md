---
document_type: adversarial-review
scope: LOCAL
fix_pr: FIX-IEQ-ERRPATH-001
passes: [9]
feature_head_at_review: 7a2a0f73
fix_burst_head: 656bc7a0
date: 2026-07-09
clean_strict: false
clean_pr_merge: true
finding_counts:
  OBS: 1
  total: 1
code_behavior_defects: 0
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# LOCAL Adversary Pass 9 — FIX-IEQ-ERRPATH-001

---

## Pass 9 (frozen 7a2a0f73; fresh-context adversary; fix-PR IEQ non-existent column error path; streak candidate 1/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=YES

**Findings:** 1 (0 CRIT/HIGH/MED + 1 OBS — adversary-marked "backlog"; orchestrator applied production-grade lens and fixed in-scope)

**Code HEAD at review:** 7a2a0f73 (frozen; D-1619 test-writer 5 alias-qualified regression locks positions 10-14; BC-2.11.016 v1.16 narrative-only; 5365/5365 GREEN; non-exhaustive 89/89)

**Fix-burst HEAD:** 656bc7a0 (implementer: is_registered() disambiguation at both gate sites; EC-11-041 single-tenant zero-column; test_m1_single_tenant_no_columns_registered_fails_open updated + bc_2_11_019_n1b_test.rs fixture gained real columns + table_registry.rs doc-comments updated; 5367/5367 GREEN; non-exhaustive 89/89)

**LOCAL 3-CLEAN(strict) streak after pass-9 fix-burst:** 0/3 (NOT CLEAN(strict); fix-burst dispatched; RESET by @656bc7a0 push per DRIFT-ORCH-PRLEVEL-PUSH-001)

**Module test-count reconciliation:** Pass-8 report cited 46 tests in module; actual pre-@a1b03ecb count was 47 (one additional test was present prior to this burst). Post this burst: 48 module tests. Workspace authoritative series remains canonical (5365→5367).

---

## Finding ADV-FIX-P9-OBS-001 — EC-11-041 single-tenant/multi-tenant asymmetry on zero-column registered tables

**Severity:** OBS (LOW-confidence; adversary-marked as "backlog" candidate)

**Classification:** behavioral asymmetry — single-tenant path failed open where multi-tenant path correctly fired E-QUERY-038; root cause: `TableRegistry::register_sensor` inserts into `registered` unconditionally but into `columns_by_table` only when non-empty, making "unregistered" and "registered-with-zero-columns" states indistinguishable at the column gate

**Affected files:** `crates/prism-query/src/engine.rs` — `get_initial_available_columns` and `check_column_availability` gate sites; `crates/prism-query/src/table_registry.rs` doc-comments

**BC reference:** BC-2.11.016 v1.16 §Gate Positions; EC-11-041 (zero-column registered table behavior); FP-001 (fail-open discipline)

**Finding:** At frozen 7a2a0f73, the multi-tenant path (table_registry Some) correctly fired E-QUERY-038 with `available_columns: []` when a table was registered but had zero columns — consistent with EC-11-041. The single-tenant path (table_registry None or unresolved) failed open for the same query shape because the gate sites (`get_initial_available_columns` and `check_column_availability`) did not call `is_registered()` to distinguish the "registered-with-zero-columns" state from the "genuinely unregistered" state. Both states produced the same `columns_by_table` result (empty/absent), so the gate treated zero-column-registered tables as unregistered and preserved fail-open behavior.

This is an EC-11-041 asymmetry: a registered table with zero columns should fire E-QUERY-038 (per BC-2.11.016 v1.16) regardless of the tenant path; an unregistered table should preserve fail-open (E-QUERY-037 domain). The `register_sensor` implementation inserts into `registered` unconditionally (registration is always recorded) but inserts into `columns_by_table` only when the column list is non-empty, creating a structural gap where a registered sensor with an empty schema is indistinguishable from an unregistered one at the gate lookup level.

The adversary marked this as a "backlog" candidate due to the LOW severity (zero demo-blocking impact; no CRIT/HIGH/MED surface). The orchestrator applied the production-grade lens (Canonical Principle Rule 3/4): the fix is in-scope and definite — not a genuine future-dependency deferral requiring human authorization. Fixed in-scope.

**Empirical confirmation by test-writer:** RED test authored at @a1b03ecb: `test_m1_single_tenant_no_columns_registered_fails_open` exercised the single-tenant path for a zero-column registered table, asserting fail-open behavior (incorrect per BC-2.11.016 v1.16 EC-11-041). Multi-tenant equivalent confirmed GREEN (gate was already correct on that path). RED confirmed the asymmetry as a genuine behavioral deviation.

**Fix at @656bc7a0:** Both gate sites (`get_initial_available_columns` and `check_column_availability`) now call `is_registered()` before consulting `columns_by_table`. Result: registered+empty → gate fires E-QUERY-038 with `available_columns: []` and no `did_you_mean` suggestion per EC-11-041; unregistered → fail-open preserved (E-QUERY-037 domain). No BC version bump required (BC-2.11.016 v1.16 EC-11-041 already defined this behavior correctly; the fix aligns the implementation with the existing spec).

**Collateral fixes:**
- `test_m1_single_tenant_no_columns_registered_fails_open`: old fail-open expectation superseded by BC-2.11.016 v1.16 gate behavior; updated to assert E-QUERY-038 with empty available_columns.
- `bc_2_11_019_n1b_test.rs`: fixture was relying on fail-open behavior for a zero-column registration scenario; gained real columns to preserve the test's intent without depending on fail-open.
- `table_registry.rs` doc-comments: updated to document the `registered` vs `columns_by_table` invariant explicitly (TD-VSDD-059 doc-comment accuracy discipline).

**Routed:** implementer (fix-in-scope; orchestrator production-grade adjudication)

**Closure:** CLOSED — RED @a1b03ecb (single-tenant RED + multi-tenant green-lock); GREEN @656bc7a0 (is_registered() disambiguation at both gate sites; 48/48 module GREEN; just check 5367/5367 GREEN; non-exhaustive 89/89).

---

## Pass Notes

**All 14 positions + EC-11-039..068 traced:** PASS — adversary performed exhaustive trace of all 14 gate positions and the full EC-11-039..068 edge-case catalog at frozen 7a2a0f73. ADV-FIX-P9-OBS-001 is the only deviation found across the entire catalog.

**DERIVED provenance state machine verified:** PASS — the three-state provenance machine (UNKNOWN/RAW/DERIVED) and its binding-context interactions verified across all gate positions and pipeline compositions. No new provenance-state asymmetries identified beyond ADV-FIX-P9-OBS-001.

**POL-24 (byte-parity EC-body):** PASS — no new EC bodies added in this fix-burst; no EC text changes.

**SAP-1 (Structured Event Catalog):** PASS — 3 `column_not_found.rejected` emission sites verified cataloged in BC-2.16.002 v2.07 Canonical Structured Event Catalog. No new `event_type =` sites introduced in the fix-burst.

**TD-VSDD-091 (volatile pin prohibition):** PASS — doc-comment updates in `table_registry.rs` use behavioral anchors and invariant descriptions, not `file.rs:NNN` line-number citations.

**Story pins:** PASS — carrier stories current to BC-2.11.016 v1.16 at frozen 7a2a0f73. No new pin-sync required from this burst (BC-2.11.016 version UNCHANGED at v1.16; no spec amendments in fix-burst).

**Novelty:** LOW — "genuine convergence point". The finding is an EC-11-041 asymmetry that falls out naturally from the `is_registered()` / `columns_by_table` split in `TableRegistry`. It is a structural residual of the zero-column registration edge case that was not exercised in prior pass-bursts. No new BC logic or gate architecture required; the fix is a one-guard extension at two existing gate sites. Cascade finding trajectory: 6→3→3→2→1 (per-pass total finding count).
