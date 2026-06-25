---
pass: 9
story: S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001
frozen_head: f58bb9a0
diff_range: "903c8fcb..f58bb9a0"
reviewer_scope: "story v1.7 (27 ACs) + 8 BCs + ADR-043/044/045/046 + error-taxonomy v1.99"
date: 2026-06-25
clean_strict: false
clean_pr_merge: false
streak_before: 0
streak_after: 0
findings_count: 1
severity_counts: {CRIT: 0, HIGH: 0, MED: 1, LOW: 0, OBS: 0}
disposition: ALL_CLOSED_IN_SCOPE
new_frozen_head: 3f685515
---

# LOCAL Adversary Pass 9 — S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001

**Frozen code HEAD reviewed:** `f58bb9a0`
**Diff range:** `903c8fcb..f58bb9a0`
**Story version read:** v1.7 (27 ACs, 8 BCs)
**Date:** 2026-06-25

## Verdict

- **CLEAN(strict):** NO
- **CLEAN(PR-merge):** NO
- **Findings:** 1 (MED-1)
- **Streak:** RESET 0/3 on new HEAD `3f685515`

## Full AC-001..AC-027 Audit

All 27 acceptance criteria verified. Summary:

| AC Range | Status |
|----------|--------|
| AC-001..AC-008 (temporal NOW/INTERVAL, FORBID-BOTH, row-limit) | PASS |
| AC-009 (D1 mode-bridge negative control) | PASS |
| AC-010..AC-015 (PrismQL normalize, org scope, table-not-found) | PASS |
| AC-016..AC-021 (E-QUERY-036/040, -32602 mapping) | PASS |
| AC-022..AC-025 (guided pipe errors, all pipeline positions) | PASS |
| AC-026..AC-027 (D2 mode-bridge, error_recovery) | PASS |
| SAP-1 (tracing emission catalog) | PASS — no new event_type sites |
| SAP-2 (DTU TOML schema parity) | N/A |
| SID-1 (no-ignored-test rationalization prohibition) | PASS |

## Finding: MED-1 (TD-VSDD-060 sibling-sweep gap)

**Severity:** MED
**Classification:** TD-VSDD-060 sibling-sweep — co-equal public entry point parity

**Description:**

`parse_with_registry` is a co-equal public entry point to `parse_with_limits` in `crates/prism-query/src/lib.rs`. The story and architecture mapping document both entry points as part of the public query surface. However, `parse_with_registry`'s `Select` arm called `parse_sql_internal` directly WITHOUT applying the BC-2.11.023 §D1/§D2 mode-bridge rewrites and enrich parity routing that `parse_with_limits` applies.

**Proof of defect:** The negative-control query `SELECT * FROM t | INVALID_KEYWORD` routed through `parse_with_registry` returned a raw Chumsky parse dump rather than the AC-009-mandated guided error message. The canonical rewrite chain (`rewrite_d1_filter_parser_mode_bridge`, `rewrite_d2_sql_keyword_in_pipe_position`, `rewrite_enrich_parse_errors`) was entirely absent from the `parse_with_registry` code path.

**Latency assessment:** Latent — no current production caller routes through `parse_with_registry` in the demo flow. However, the codebase documents both entry points as co-equal in the architecture mapping, and the parity invariant is explicitly required by the story spec. A latent divergence of this kind constitutes a production-grade defect under the canonical principle.

**Adjudication:** Fix in scope. Structural extraction of shared helper eliminates divergence at the source.

**Status:** CLOSED by implementer (code `f58bb9a0 → 3f685515`)

**Fix description:** Extracted `parse_select_mode` private helper in `crates/prism-query/src/lib.rs` containing the single authoritative D1/D2/enrich routing logic (the full rewrite chain). Both `parse_with_limits` and `parse_with_registry` now delegate their `Select` arms to `parse_select_mode` — divergence eliminated structurally, not by duplication.

**Load-bearing tests added (3):**
- `test_bc_2_11_parse_with_registry_d1_sql_bare_pipe_guided_error` — D1 SQL+bare-pipe path via `parse_with_registry`; asserts guided message (not raw Chumsky dump)
- `test_bc_2_11_parse_with_registry_d2_sql_keyword_in_pipe_position` — D2 SQL-keyword-in-pipe path via `parse_with_registry`; asserts D2 guided message
- `test_bc_2_11_parse_with_registry_enrich_parity` — enrich path parity; `parse_with_registry` and `parse_with_limits` produce identical results for the same enrich query

**Verification:** `just check` EXIT=0 (4924/4924 tests pass); non-exhaustive gate 87 (UNCHANGED); SAP-1 clean (no new `event_type` sites).

## Positives Verified (do not reflag in future passes)

- Temporal `NOW()` / `INTERVAL` production wiring — all 4 AST arms: PASS
- FORBID-BOTH 0-row hoist Step 1b: PASS
- Filter-mode load-bearing tests: PASS
- E-QUERY-036/037 label distinction (Available tables vs Available sensors): PASS — CLOSED in prior passes; do-not-reflag
- E-QUERY-040 verbatim message: PASS
- BC-2.16.002 catalog completeness (SAP-1): PASS
- -32602 mapping + negative -32000 controls: PASS
- BC version pins at v1.7: PASS
- `#[non_exhaustive]` gate 87/87: PASS
- SqlPipe enrich/D2 parity (AC-022/025): PASS — unified shared helper `parse_select_mode`
- `parse_with_registry` / `parse_with_limits` parity: PASS — unified via `parse_select_mode` (CLOSED this pass)
- AC-019 deferral: CARRY FORWARD (not a LOCAL finding)
- AC-020 (runbook v1.4): CARRY FORWARD (satisfied — do-not-flag LOCAL)
- AC-024 (PR-description GRAMMAR-013 table): CARRY FORWARD (PR-LEVEL deliverable — do-not-flag LOCAL)
- Temporal plain-string (D-1335): CARRY FORWARD
- Paper-fix detection (TD-VSDD-059): PASS — structural helper extraction with 3 load-bearing tests
- Production-grade assessment: PASS

## Severity Trend

```
Pass 1 (e518d96c): 1H + 1M + 3OBS
Pass 2 (f03679b2 v1.3): 1M (BC pins)
Pass 3 (f03679b2 v1.4): 1H + 1M + 1L
Pass 4 (81372a22 v1.5): 2H + 2M
Pass 5 (9eb55cfe v1.6): CLEAN(strict) — streak 1/3
Pass 6 (9eb55cfe v1.6): 1M (AC-023 IS-NOT-NULL note)
Pass 7 (64d91111 v1.7): 3 LOW/OBS (OBS-1 CLOSED a0ebd740; OBS-2 adjudicated; OBS-3 CLOSED story v1.7)
Pass 8 (a0ebd740 v1.7): 1 LOW OBS-1 (SqlPipe enrich parity — CLOSED f58bb9a0)
Pass 9 (f58bb9a0 v1.7): 1 MED (parse_with_registry parity gap — CLOSED 3f685515)
→ EXPECT CLEAN on 3f685515
```

## Next Step

LOCAL adversary Pass 1/2/3 on UNCHANGED `3f685515` — 3 consecutive CLEAN(strict) required per BC-5.39.001.
