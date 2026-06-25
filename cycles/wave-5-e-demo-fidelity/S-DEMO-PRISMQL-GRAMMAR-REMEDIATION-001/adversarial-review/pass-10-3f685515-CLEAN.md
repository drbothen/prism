---
pass: 10
story: S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001
frozen_head: 3f685515
diff_range: "903c8fcb..3f685515"
reviewer_scope: "story v1.7 (27 ACs) + 8 BCs + ADR-043/044/045/046 + error-taxonomy v1.99"
date: 2026-06-25
clean_strict: true
clean_pr_merge: true
streak_before: 0
streak_after: 1
findings_count: 0
severity_counts: {CRIT: 0, HIGH: 0, MED: 0, LOW: 0, OBS: 0}
disposition: CLEAN
---

# LOCAL Adversary Pass 10 — S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001

**Frozen code HEAD reviewed:** `3f685515`
**Diff range:** `903c8fcb..3f685515`
**Story version read:** v1.7 (27 ACs, 8 BCs)
**Date:** 2026-06-25

## Verdict

- **CLEAN(strict):** YES — zero findings of any severity
- **CLEAN(PR-merge):** YES — zero findings of any severity
- **Findings:** 0
- **Streak:** 1/3 on `3f685515` (CLEAN streak opened; passes 10, 11, 12 required on UNCHANGED HEAD)

**Note on streak validity:** This pass is streak-eligible as clean-streak pass #1 on HEAD `3f685515`. However, the streak was subsequently RESET to 0/3 when LOCAL Pass 11 (same HEAD, same diff) returned a HIGH finding (HIGH-1: FORBID-BOTH `| tail` bypass). See pass-11-3f685515.md.

## Full AC-001..AC-027 Audit

All 27 acceptance criteria verified against HEAD `3f685515`. Summary:

| AC Range | Status |
|----------|--------|
| AC-001..AC-004 (temporal NOW/INTERVAL production wiring) | PASS |
| AC-005..AC-006 (FORBID-BOTH 0-row hoist, LIMIT gate) | PASS |
| AC-007..AC-008 (row-limit enforcement, 10k cap) | PASS |
| AC-009 (D1 mode-bridge negative control) | PASS |
| AC-010..AC-015 (PrismQL normalize, org scope, table-not-found) | PASS |
| AC-016..AC-019 (E-QUERY-036/037, -32602, AC-019 deferred) | PASS / CARRY |
| AC-020 (runbook v1.4 present) | PASS — do-not-flag LOCAL |
| AC-021..AC-023 (E-QUERY-040, guided-pipe DISTINCT, IS-NOT-NULL) | PASS |
| AC-024 (GRAMMAR-013 table in PR description) | CARRY — PR-LEVEL deliverable |
| AC-025 (guided errors all pipeline positions) | PASS |
| AC-026..AC-027 (D2 mode-bridge, error_recovery field) | PASS |
| SAP-1 (tracing emission catalog) | PASS — no new event_type sites |
| SAP-2 (DTU TOML schema parity) | N/A |
| SID-1 (no-ignored-test rationalization prohibition) | PASS |

## Probe Coverage

Adversary probed all classic failure modes for this story:

- **Temporal production wiring** — `NOW()` and `INTERVAL '...'` folding into ISO-8601 strings across all 4 AST arms (Filter, Dedup, Stats, PipeStage). PASS.
- **FORBID-BOTH E-QUERY-040** — confirmed 0-row hoist Step 1b triggers when BOTH a head `LIMIT N` and a SQL `LIMIT N` are present in the plan. PASS.
- **`| limit N` after head LIMIT N** — RedundantRowLimit error returned. PASS.
- **parse_with_registry / parse_with_limits parity** — both entry points delegate to `parse_select_mode`; D1/D2/enrich rewrites applied uniformly. PASS (closed D-1345 via `3f685515`).
- **SqlPipe enrich/D2 parity** — unified pipe-stage error path via shared helper. PASS (closed D-1344 via `f58bb9a0`).
- **E-QUERY-036/037 label distinction** — "Available tables" vs "Available sensors" messages distinct. PASS.
- **-32602 mapping** — unknown table returns structured JSON-RPC -32602 with actionable details. PASS.
- **Filter-arm unfolded-temporal guard** — `predicate_has_unfolded_temporal_pub` guard present on `Ast::Filter` arm. PASS.
- **OrgRegistry Arc-DI wiring** — no placeholder-construct anti-pattern. PASS.
- **non-exhaustive gate** — 87 types (UNCHANGED). PASS.
- **BC-2.16.002 catalog completeness (SAP-1)** — all `event_type` values registered. PASS.
- **Paper-fix detection (TD-VSDD-059)** — all closures have load-bearing tests or structural assertions. PASS.

**Adversary assessment:** This diff is unusually complete. All classic failure modes probed; none found to be open. Implementation exhibits layered defense: structural DRY helpers (parse_select_mode, shared pipe-stage error helper) eliminate entire categories of parity divergence rather than patching individual call sites.

## Positives Verified (do not reflag in future passes)

- Temporal `NOW()` / `INTERVAL` production wiring — all 4 AST arms: PASS
- FORBID-BOTH 0-row hoist Step 1b: PASS
- D1 / D2 / enrich mode-bridge parity (parse_select_mode): PASS
- E-QUERY-036/037 label distinction: PASS — CLOSED prior passes; do-not-reflag
- E-QUERY-040 verbatim message: PASS
- BC-2.16.002 catalog completeness (SAP-1): PASS
- -32602 mapping + negative -32000 controls: PASS
- BC version pins at v1.7: PASS
- `#[non_exhaustive]` gate 87/87: PASS
- SqlPipe enrich/D2 parity (AC-022/025): PASS — unified shared helper
- `parse_with_registry` / `parse_with_limits` parity: PASS — unified via `parse_select_mode` (CLOSED 3f685515)
- Filter-arm unfolded-temporal guard: PASS
- OrgRegistry Arc-DI wiring: PASS
- AC-019 deferral: CARRY FORWARD (not a LOCAL finding)
- AC-020 (runbook v1.4): CARRY FORWARD (satisfied — do-not-flag LOCAL)
- AC-024 (PR-description GRAMMAR-013 table): CARRY FORWARD (PR-LEVEL deliverable — do-not-flag LOCAL)
- Temporal plain-string (D-1335): CARRY FORWARD
- Paper-fix detection (TD-VSDD-059): PASS — structural helpers with load-bearing tests
- Production-grade assessment: PASS

## Severity Trend (through this pass)

```
Pass 1  (e518d96c): 1H + 1M + 3OBS
Pass 2  (f03679b2 v1.3): 1M (BC pins)
Pass 3  (f03679b2 v1.4): 1H + 1M + 1L
Pass 4  (81372a22 v1.5): 2H + 2M
Pass 5  (9eb55cfe v1.6): CLEAN(strict) — streak 1/3 (RESET by Pass 6)
Pass 6  (9eb55cfe v1.6): 1M (AC-023 IS-NOT-NULL note)
Pass 7  (64d91111 v1.7): 3 LOW/OBS
Pass 8  (a0ebd740 v1.7): 1 LOW (SqlPipe enrich parity — CLOSED f58bb9a0)
Pass 9  (f58bb9a0 v1.7): 1 MED (parse_with_registry parity — CLOSED 3f685515)
Pass 10 (3f685515 v1.7): CLEAN(strict) — streak 1/3 (RESET by Pass 11)
```

## Next Step

LOCAL adversary Pass 11 on UNCHANGED `3f685515` — fresh adversary instance, same frozen HEAD.
