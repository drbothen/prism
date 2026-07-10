---
document_type: adversarial-review-cascade-summary
scope: LOCAL
defect: DEFECT-CSDEVICES-EMPTY-PIPELINE-001
fix_branch: fix/csdevices-empty-pipeline
date_pass4: 2026-07-10
total_passes_to_date: 9
date_pass5: 2026-07-10
streak_at_pass5: 0
date_pass6: 2026-07-10
streak_at_pass6: 0
date_pass7: 2026-07-10
streak_at_pass7: 0
date_pass9: 2026-07-10
streak_at_pass9: 0
date_pass10: 2026-07-10
streak_at_pass10: 0
total_passes_to_date: 10
convergence: IN_PROGRESS
authored_by: state-manager
---

# LOCAL Adversary Cascade Summary — DEFECT-CSDEVICES-EMPTY-PIPELINE-001

**Defect (two independent sub-defects):**

1. **Sub-defect 1 — Sensor TOML spec defect:** `crowdstrike.sensor.toml` `fetch_devices` step
   had no `${...}` interpolation reference in `path` or `body_template`; `find_fan_out_array()`
   returned `None`; GET `/devices/entities/devices/v2` issued with 0 `ids` params → empty DTU
   response. Fix: POST conversion (DTU `PostDeviceDetailsV2` route addition + TOML
   `method`/`body_template` change per architect Option 1 ratification D-1652).

2. **Sub-defect 2 — Product-code defect:** `materialization.rs` skipped DataFusion registration
   for 0-batch tables; mixed JOIN → `DataFusionError::Plan("table not found")` → `-32000`
   "Internal error". Fix: register schema-only empty `MemTable` from declared spec columns
   (BC-2.01.010 / BC-2.11.005 DEC-022).

**Root-cause artifact:** `.factory/research/defect-csdevices-empty-pipeline-rootcause-2026-07-10.md`

---

## Cascade Table (10 passes to date)

| Pass | Frozen HEAD | CLEAN(strict) | CLEAN(PR-merge) | Findings | Fix-burst HEAD | Streak |
|------|-------------|---------------|-----------------|----------|----------------|--------|
| 1 | (initial fix branch @22f429d0 area) | NO | NO | 2 HIGH (F-CSD-P1-001 contract-fidelity, F-CSD-P1-002) + 3 MED + LOWs | spec + code burst | 0/3 |
| 2 | (post-pass-1 fix HEAD) | YES | YES | 0 | — | 1/3 |
| 3 | (post-pass-2 HEAD) | NO | NO | 1 HIGH F-CSD-P3-001 (subquery walk — Expr::InSubquery projection-position execution) | architect adjudication + fix-burst | 0/3 |
| 4 | (frozen @22f429d0 + spec layer) | NO | NO | 1 HIGH F-CSD-P4-001 (unauthorized COUNT rewrite) + 2 MED (F-CSD-P4-002/003) + 1 LOW (F-CSD-P4-004) + 2 OBS incl. F-CSD-P4-005 [process-gap] | spec layer + S-HARDEN-PLAN-PINNING-001 draft | 0/3 |
| 5 | frozen @22f429d0 | NO | NO | 1 HIGH F-CSD-P5-001 (contains_insubquery FuncCall-args recursion gap) + 3 MED (F-CSD-P5-002 POL-24 hint drift / F-CSD-P5-003 stale test-vector row / F-CSD-P5-004 stale position-invariant claim) | — | 0/3 |
| — (fix-burst) | — | — | — | — | test-writer RED @283bbc4b T18/T19+T17-tightened; implementer GREEN @30217403 FuncCall Scalar/Aggregate recursion arms + Window explicit false + byte-exact hint; PO BC-2.11.005 v1.8→v1.9; 1521/1521 prism-query; just check GREEN | 0/3 |
| 6 | frozen @30217403 | NO | YES | 1 LOW F-CSD-P6-001 (`check_expr_insubquery_projection` lacked DML `source_select` defense-in-depth arm; zero current exploitability, S-3.06 forward risk) | implementer fix-burst @3d48b6a9 | 0/3 |
| — (fix-burst) | — | — | — | — | implementer @3d48b6a9 — T20 RED→GREEN (INSERT INTO...SELECT col IN (SELECT...) now fires E-QUERY-043 via new `Ast::Sql(SqlStatement::Dml(dml)) => dml.source_select...check_sql_query` arm; comment corrected); 20/20 defect tests; 15/15 temporal; 1522/1522 prism-query; just check GREEN; non-exhaustive 89/89 | 0/3 |
| 7 | frozen @3d48b6a9 | NO | NO | 1 MED F-CSD-P7-001 (`check_expr_insubquery_projection` `check_sql_query` non-recursive: projection `Expr::InSubquery` nested inside WHERE/HAVING `Predicate::InSubquery` subqueries slipped to -32000; third walker-dimension gap in gate family; NOTE: HAVING `Predicate::InSubquery` IS grammar-reachable — corrects pass-6 contrary determination) + 3 OBS (OBS-001 endpoint-count method ambiguity doc-closed; OBS-002 stale fidelity future-note doc-closed; OBS-004 T20 ordering coupling doc-closed) + OBS-003 no-action (incidents SAP-2 gap correctly anchored to DTU-EXT-001) | — | 0/3 |
| — (fix-burst) | — | — | — | — | test-writer RED @dd51459c (T21-T23 + T24 over-rejection control); implementer STRUCTURAL @38b05bbc — `descend_subquery_expr` + `check_predicate` + extended `check_sql_query` covering WHERE/HAVING/JOIN-ON interiors recursively (mirrors `walk_sql_query` pattern; closes P5/P6/P7 walker-gap lineage structurally); T5 JOIN-ON boundary + T24 predicate-support preserved; OBS-001/002/004 doc closures; 24/24 defect tests; 15/15 temporal; just check 5444/5444 GREEN; non-exhaustive 89/89 | 0/3 |
| 8 | frozen @38b05bbc | NO | NO | 1 MED F-CSD-P8-001 (E-QUERY-043 gate placed below `!any_external_table_registered` early-return in pipeline Step 1d session path; all-zero-batch pipeline path returned Ok(empty) instead of E-QUERY-043; data-dependent error surface) + 1 LOW F-CSD-P8-002 (DML `filter`/`assignments` interiors unwalked; fourth walker-dimension gap in E-QUERY-043 gate family) + 2 OBS (P8-003 dead `variables_produced` TOML entries; P8-004 over-broad gate docstring). Strong invariant confirmations: recursion bounded by PRISM_MAX_NESTING_DEPTH; walker parity verified; POL-24 byte-clean; SAP-1/2 clean. | — | 0/3 |
| — (fix-burst) | — | — | — | — | test-writer RED @0198c88e (T25 pipeline-path bypass RED + T26 populated-path lock GREEN + T27 DML-interior constructed-AST RED); implementer @8b284d67 — DUAL gate placement (pipeline Step 1d after `check_temporal_literals`, data-independent + retained session-entry call); DML arm three-way (`source_select` + `filter` via `check_predicate` + `assignments` via `descend_subquery_expr`); docstring corrected; TOML `variables_produced` dead entries → `[]` with fallback-behavior comments (validator-consumer check clean); 27/27 defect; 96/96 temporal; 1529/1529 prism-query; 765/765 prism-spec-engine; just check GREEN; non-exhaustive 89/89 | 0/3 |

| 9 | frozen @8b284d67 | NO | NO | 1 HIGH F-CSD-P9-001 (`prism-dtu-harness` CrowdStrike clone GET-only on POST `/devices/entities/devices/v2` in BOTH `build_router()` + `build_standalone_router()` builders; INV-HARNESS-ROUTE-PARITY violation; latent 405→silent-0-row in harness-driven scenarios) + 2 OBS (informational, no action) | test-writer @d4a4cb37 + implementer @544acd70 + PO BC-2.16.013 v1.27 | 0/3 |
| — (fix-burst) | — | — | — | — | test-writer RED @d4a4cb37 (4 harness tests targeting `build_router()` + `build_standalone_router()` CrowdStrike POST `/devices/entities/devices/v2`); implementer @544acd70 — `post_host_details` + `host_details_inner` shared-helper refactor in `prism-dtu-harness` CrowdStrike clone; both routers registered; TD-VSDD-060 full CrowdStrike verb-surface sweep = parity everywhere (`/dtu/filter-log` standalone-only by design, confirmed correct); PO BC-2.16.013 v1.26→v1.27 (INV-HARNESS-ROUTE-PARITY block + CrowdStrike example added to §INV-HARNESS-ROUTE-PARITY); harness 140/140; workspace 5451/5451; just check GREEN; non-exhaustive 89/89 | 0/3 |
| 10 | frozen @544acd70 | NO | **YES** (FIRST) | 1 LOW F-CSD-P10-001 (JOIN-ON × FuncCall-wrapped InSubquery matrix cell unverified — DataFusion execution behavior unknown at pass time) + 1 pre-existing OBS-1 (harness `host_detail()` missing `first_seen`) | CLOSED via empirical determination + in-scope fix: test-writer ran empirical DETERMINATION — DataFusion EXECUTES JOIN-ON FuncCall-wrapped InSubquery shape (2-row result → F-CSD-P10-001 closed as documented DataFusion capability); walker asymmetry confirmed correct by design (no production change needed); OBS-1 fixed in-scope by implementer @5a58046f (`first_seen` added to harness `host_detail()`, RFC-3339 deterministic, 6/6 TOML columns covered; harness 141/141) | 0/3 |
| — (closure) | — | — | — | — | EMPIRICAL DETERMINATION: test-writer ran JOIN-ON × FuncCall-wrapped InSubquery shape — DataFusion executes successfully (2-row result); T28/T29 GREEN locks; walker asymmetry (check_predicate JOIN-ON arm fires Expr::InSubquery scan) confirmed correct by design — the gate correctly rejects plain `col IN (SELECT ...)` in JOIN-ON but allows `func(col) IN (SELECT ...)` because DataFusion evaluates it as a FuncCall return value, not a correlated subquery. F-CSD-P10-001 CLOSED as documented DataFusion capability. OBS-1: implementer @5a58046f added `first_seen` to `host_detail()` in `prism-dtu-harness` CrowdStrike clone (RFC-3339, deterministic seed, 6/6 TOML-declared columns now covered; harness 141/141). 29-test prism-query defect suite GREEN; workspace just check GREEN; non-exhaustive 89/89. No production code changes. Streak 0/3. LOCAL pass 11 DISPATCHED on frozen HEAD `5a58046f`. | 0/3 |

Streak reset at pass 3 and pass 4. Streak 0/3 after pass-10 closure. Pass 10 is the FIRST CLEAN(PR-merge) of the cascade. Structural-fix class closure: P5 (FuncCall-args recursion gap), P6 (DML source_select defense-in-depth), P7 (WHERE/HAVING/JOIN-ON check_sql_query non-recursive), P8-MED (gate placement below early-return; DUAL placement fix), P8-LOW (DML filter/assignments interiors), P9-HIGH (harness-clone POST verb surface gap) — all closed, E-QUERY-043 gate family + harness parity complete. Pass-10 closure pattern: empirical DataFusion behavior determination (F-CSD-P10-001 LOW closed as verified capability) + in-scope harness OBS-1 fix (first_seen). LOCAL pass 11 DISPATCHED on frozen HEAD `5a58046f`.

---

## Finding Summary

### Pass 1

| ID | Severity | Description | Resolution |
|----|----------|-------------|------------|
| F-CSD-P1-001 | HIGH | Contract-fidelity: POST conversion spec not propagated to BC-2.06.019 | Spec propagation burst (D-1652 + D-1654) |
| F-CSD-P1-002 | HIGH | Sub-defect 2 empty-MemTable gate missing | Implementer TDD fix-burst |
| F-CSD-P1-003..005 | MED | Spec / test coverage gaps | Closed in fix-burst |
| F-CSD-P1-006 | (wording) | BC-2.06.019 merge-brittle sentence | BC-2.06.019 v1.16→v1.17 (D-1654) |

### Pass 2

Zero findings. CLEAN(strict)=YES. Streak 1/3.

### Pass 3

| ID | Severity | Description | Resolution |
|----|----------|-------------|------------|
| F-CSD-P3-001 | HIGH | Subquery walk: `Expr::InSubquery` in projection position executed rather than plan-time rejected; misaligned with `Expr::InSubquery` WHERE-position path | Architect adjudication D-1656 — Option A: revert COUNT(*) rewrite; E-QUERY-043 plan-time rejection gate for Expr::InSubquery in non-WHERE positions; BC-2.11.003 v1.12→v1.13; code @22f429d0 (revert + gate + 6 position locks) |

Streak reset 1/3→0/3.

### Pass 4

| ID | Severity | Description | Resolution |
|----|----------|-------------|------------|
| F-CSD-P4-001 | HIGH | Unauthorized COUNT(*) rewrite in non-WHERE subquery position introduced in fix-burst for F-CSD-P3-001; produced semantic alteration of count queries | Architect Option A adjudication D-1656: revert rewrite; E-QUERY-043 plan-time gate; code @22f429d0; BC-2.11.003 v1.13 |
| F-CSD-P4-002 | MED | Spec coverage gap in E-QUERY-043 error taxonomy row | error-taxonomy v2.37→v2.38 (E-QUERY-043 row added) |
| F-CSD-P4-003 | MED | DEC-022 position-invariant not fully covered by test vectors in BC-2.11.005 | BC-2.11.005 v1.7→v1.8 (11 test vectors; D-1656) |
| F-CSD-P4-004 | LOW | Pin / documentation finding | Closed in spec layer |
| F-CSD-P4-005 | OBS [process-gap] | `high002_plan_pinning_tests.rs` substring guards insufficient — should be structural SQL-shape assertions | S-HARDEN-PLAN-PINNING-001 draft v0.1 registered (D-1656); codified in lessons.md L25 |

Streak stays 0/3. LOCAL pass 5 NEXT on frozen `22f429d0`.

### Pass 5

| ID | Severity | Description | Resolution |
|----|----------|-------------|------------|
| F-CSD-P5-001 | HIGH | `contains_insubquery` helper does not recurse into `Expr::ScalarFunction` / `Expr::AggregateFunction` args — FuncCall-args recursion gap; `Expr::WindowFunction` explicit-false missing | test-writer RED @283bbc4b T18/T19+T17-tightened; implementer GREEN @30217403 FuncCall Scalar/Aggregate recursion arms + Window explicit false |
| F-CSD-P5-002 | MED | POL-24 hint message for E-QUERY-043 does not match byte-exact spec — drift from BC-2.11.003 v1.13 hint wording | implementer GREEN @30217403 byte-exact hint string aligned |
| F-CSD-P5-003 | MED | Stale test-vector row in BC-2.11.005 cited wrong test name; E-QUERY-043 postcondition reference missing | BC-2.11.005 v1.8→v1.9 (PO fix-burst; D-1657) |
| F-CSD-P5-004 | MED | §Postconditions + §DEC-022 incorrectly listed "SELECT Expr::InSubquery" as a position covered by pre-registration claims | BC-2.11.005 v1.8→v1.9 (PO fix-burst; D-1657) |

Streak stays 0/3. LOCAL pass 6 NEXT on frozen `30217403`.

### Pass 6

| ID | Severity | Description | Resolution |
|----|----------|-------------|------------|
| F-CSD-P6-001 | LOW | `check_expr_insubquery_projection` lacked DML `source_select` defense-in-depth arm vs sibling `check_temporal_literals` F-P4-LOW-1 precedent; zero current exploitability (DML INSERT INTO…SELECT path not reachable via current grammar), S-3.06 forward risk | implementer @3d48b6a9: new `Ast::Sql(SqlStatement::Dml(dml)) => dml.source_select.as_ref().map(|s| check_sql_query(s))` arm; T20 RED→GREEN |

CLEAN(strict): NO. CLEAN(PR-merge): YES. Streak stays 0/3. LOCAL pass 7 NEXT on frozen `3d48b6a9`.

### Pass 7

| ID | Severity | Description | Resolution |
|----|----------|-------------|------------|
| F-CSD-P7-001 | MED | `check_expr_insubquery_projection` calls `check_sql_query` on the subquery body but `check_sql_query` does not recurse into WHERE/HAVING `Predicate::InSubquery` sub-subqueries; projection `Expr::InSubquery` nested one level deeper slipped to -32000. Third walker-dimension gap in the E-QUERY-043 gate family (P5=FuncCall-args, P6=DML source_select, P7=WHERE/HAVING/JOIN-ON interior). NOTE: HAVING `Predicate::InSubquery` IS grammar-reachable (corrects pass-6 determination that it was not). | implementer STRUCTURAL @38b05bbc: `descend_subquery_expr` + `check_predicate` + extended `check_sql_query` (recursive walk of WHERE/HAVING/JOIN-ON interiors); T21-T23 RED→GREEN; T24 over-rejection control |
| OBS-001 | OBS | Endpoint-count comment method ambiguity | doc-closed @38b05bbc |
| OBS-002 | OBS | Stale fidelity future-note in comment | doc-closed @38b05bbc |
| OBS-003 | OBS | Incidents SAP-2 gap anchored to DTU-EXT-001 | no-action (correctly anchored) |
| OBS-004 | OBS | T20 ordering coupling in test | doc-closed @38b05bbc |

CLEAN(strict): NO. CLEAN(PR-merge): NO. Streak stays 0/3. LOCAL pass 8 NEXT on frozen `38b05bbc`.

### Pass 8

| ID | Severity | Description | Resolution |
|----|----------|-------------|------------|
| F-CSD-P8-001 | MED | E-QUERY-043 gate placed below `!any_external_table_registered` early-return in pipeline Step 1d session path; all-zero-batch pipeline path (no registered tables yet) returned Ok(empty) instead of E-QUERY-043; data-dependent error surface — reachable with real empty-batch input | implementer @8b284d67: DUAL gate placement — move gate to pipeline Step 1d after `check_temporal_literals`, data-independent; retain session-entry call at original site; T25 RED→GREEN |
| F-CSD-P8-002 | LOW | DML `filter` and `assignments` interiors not walked by `check_expr_insubquery_projection`; fourth walker-dimension gap in E-QUERY-043 gate family (P5=FuncCall-args, P6=DML source_select, P7=WHERE/HAVING/JOIN-ON, P8=DML filter/assignments) | implementer @8b284d67: DML arm extended to three-way: `source_select` + `filter` via `check_predicate` + `assignments` via `descend_subquery_expr`; T27 RED→GREEN |
| P8-003 | OBS | Dead `variables_produced` TOML entries in crowdstrike.sensor.toml (values declared but no consumer; spec lists them to match raw API response shape) | TOML entries → `[]` with fallback-behavior inline comments; validator-consumer check confirmed clean |
| P8-004 | OBS | Over-broad gate docstring implied broader rejection than implemented | docstring corrected @8b284d67 |

CLEAN(strict): NO. CLEAN(PR-merge): NO. Streak stays 0/3. LOCAL pass 9 NEXT on frozen `8b284d67`.

### Pass 9

| ID | Severity | Description | Resolution |
|----|----------|-------------|------------|
| F-CSD-P9-001 | HIGH | `prism-dtu-harness` CrowdStrike clone registered GET-only handler for `/devices/entities/devices/v2` in both `build_router()` and `build_standalone_router()`; INV-HARNESS-ROUTE-PARITY violated — real CrowdStrike API uses POST; harness-driven scenarios would receive 405 Method Not Allowed, causing silent-0-row materialization in harness test runs | test-writer @d4a4cb37: 4 RED gates (both builders); implementer @544acd70: `post_host_details` + `host_details_inner` shared-helper refactor; both routers now register POST; T28–T31 RED→GREEN |
| P9-OBS-001 | OBS | (informational — no action required) | no-action |
| P9-OBS-002 | OBS | (informational — no action required) | no-action |

CLEAN(strict): NO. CLEAN(PR-merge): NO. Streak stays 0/3. LOCAL pass 10 NEXT on frozen `544acd70`.

### Pass 10

| ID | Severity | Description | Resolution |
|----|----------|-------------|------------|
| F-CSD-P10-001 | LOW | JOIN-ON × FuncCall-wrapped `Expr::InSubquery` matrix cell — walker asymmetry (`check_predicate` JOIN-ON arm fires `Expr::InSubquery` scan, which should fire E-QUERY-043, but DataFusion execution behavior unverified at pass time); unknown whether the shape executes or errors at runtime | CLOSED via empirical determination: test-writer ran the shape; DataFusion EXECUTES it (2-row result, T28/T29 GREEN locks); walker asymmetry confirmed correct by design — FuncCall-wrapped InSubquery in JOIN-ON is treated as a FuncCall return value by DataFusion (not a correlated subquery), so execution is appropriate; NO production code change needed |
| OBS-1 | OBS | `host_detail()` in `prism-dtu-harness` CrowdStrike clone was missing `first_seen` field; 5/6 TOML-declared columns populated (pre-existing gap from prior harness work) | FIXED in-scope: implementer @5a58046f added `first_seen` RFC-3339 deterministic value; 6/6 TOML columns now covered; harness 141/141 |

CLEAN(strict): NO (1 LOW + 1 OBS). CLEAN(PR-merge): **YES** — FIRST CLEAN(PR-merge) of the cascade. Streak stays 0/3. LOCAL pass 11 DISPATCHED on frozen `5a58046f`.

## Evidence at Pass 10 Fix HEAD (5a58046f)

- prism-query defect suite: **29/29** GREEN
- harness: **141/141** tests GREEN
- Full workspace `just check`: GREEN
- Non-exhaustive gate: 89/89

## Spec Layer Modified (pass-10 closure)

No spec changes — harness-only fix (OBS-1 `first_seen` field). No BC or error-taxonomy version bumps. F-CSD-P10-001 closed as documented DataFusion capability (empirical determination), no code change.

---

## Evidence at Pass 9 Fix HEAD (544acd70)

- harness: **140/140** tests GREEN
- Full workspace `just check`: **5451/5451** GREEN
- Non-exhaustive gate: 89/89

## Spec Layer Modified (pass-9 closure)

| Artifact | Version Bump | Change |
|----------|-------------|--------|
| BC-2.16.013 | v1.26→v1.27 | F-CSD-P9-001: INV-HARNESS-ROUTE-PARITY block added for CrowdStrike POST `/devices/entities/devices/v2` in both router builders; CrowdStrike example added to §INV-HARNESS-ROUTE-PARITY explanatory text |
| BC-INDEX | v7.82→v7.83 | BC-2.16.013 inline row updated (D-1661) |

---

## Evidence at Pass 4 Fix HEAD (22f429d0)

- prism-query: **5431 + 6 = 1519** tests GREEN (defect suite 17/17)
- Full workspace `just check`: GREEN
- Non-exhaustive gate: 89/89

## Spec Layer Modified (pass-4 closure)

| Artifact | Version Bump | Change |
|----------|-------------|--------|
| error-taxonomy | v2.37→v2.38 | E-QUERY-043 row (Expr::InSubquery non-WHERE position plan-time rejection) |
| BC-2.11.003 | v1.12→v1.13 | Stale subquery invariant replaced; E-QUERY-001 subquery row superseded; E-QUERY-043 row + vectors |
| BC-2.11.005 | v1.7→v1.8 | DEC-022 position-invariant expansion + 11 test vectors |
| S-HARDEN-PLAN-PINNING-001 | NEW draft v0.1 | F-CSD-P4-005 [process-gap] closure story |

## Evidence at Pass 8 Fix HEAD (8b284d67)

- prism-query: **27/27** defect tests GREEN; **96/96** temporal tests GREEN; **1529/1529** full prism-query suite GREEN
- prism-spec-engine: **765/765** tests GREEN
- Full workspace `just check`: GREEN
- Non-exhaustive gate: 89/89

## Spec Layer Modified (pass-8 closure)

No spec changes — code-only fix. DUAL gate placement in pipeline Step 1d (`check_temporal_literals` → `check_expr_insubquery_projection`) + DML `filter`/`assignments` walk extension + TOML `variables_produced` dead entries → `[]` with comments. No BC or error-taxonomy version bumps.

---

## Evidence at Pass 7 Fix HEAD (38b05bbc)

- prism-query: **24/24** defect tests GREEN; **15/15** temporal tests GREEN
- Full workspace `just check`: **5444/5444** GREEN
- Non-exhaustive gate: 89/89

## Spec Layer Modified (pass-7 closure)

No spec changes — code-only fix. Structural walker fix in `check_expr_insubquery_projection` (`descend_subquery_expr` + `check_predicate` + extended `check_sql_query`). No BC or error-taxonomy version bumps.

---

## Evidence at Pass 6 Fix HEAD (3d48b6a9)

- prism-query: **1522/1522** tests GREEN (defect suite 20/20; T20 RED→GREEN for DML source_select arm)
- Full workspace `just check`: GREEN
- Non-exhaustive gate: 89/89

## Spec Layer Modified (pass-6 closure)

No spec changes — code-only fix. DML `source_select` defense-in-depth arm added to `check_expr_insubquery_projection` in `fix/csdevices-empty-pipeline`. No BC or error-taxonomy version bumps.

---

## Evidence at Pass 5 Fix HEAD (30217403)

- prism-query: **1521/1521** tests GREEN (defect suite 19/19 after T18+T19+T17-tightened)
- Full workspace `just check`: GREEN
- Non-exhaustive gate: 89/89

## Spec Layer Modified (pass-5 closure)

| Artifact | Version Bump | Change |
|----------|-------------|--------|
| BC-2.11.005 | v1.8→v1.9 | F-CSD-P5-003: stale test-vector row corrected + E-QUERY-043 postcondition reference; F-CSD-P5-004: "SELECT Expr::InSubquery" removed from position-invariant pre-registration claims in §Postconditions + §DEC-022 |
| BC-INDEX | v7.81→v7.82 | BC-2.11.005 inline row updated (D-1657) |

## Architect Adjudication Record (F-CSD-P4-001 / F-CSD-P3-001)

**Option A selected (2026-07-10):** Revert COUNT(*) rewrite. Add E-QUERY-043 plan-time
rejection for `Expr::InSubquery` in non-WHERE projection position. WHERE-position preserved
(existing behavior correct). 6 position locks committed @22f429d0.

Source: `.factory/research/defect-csdevices-empty-pipeline-rootcause-2026-07-10.md`
§"Adjudication — Expr::InSubquery projection-position execution (F-CSD-P4-001) — 2026-07-10"

---

## Merge Record

_Pending: LOCAL cascade not yet converged. PR not yet created._

**Status: IN PROGRESS — LOCAL pass 11 IN FLIGHT on frozen `5a58046f` (streak 0/3). Pass-10 closure COMPLETE (D-1662): FIRST CLEAN(PR-merge) of cascade. F-CSD-P10-001 LOW CLOSED via empirical determination — DataFusion executes JOIN-ON FuncCall-wrapped InSubquery shape (T28/T29 GREEN); walker asymmetry confirmed correct by design. OBS-1 fixed in-scope: harness `first_seen` @5a58046f; harness 141/141; workspace just check GREEN; non-exhaustive 89/89. Walker-gap + gate-placement lineage P5/P6/P7/P8 CLOSED; harness parity gap P9 CLOSED; empirical DataFusion capability P10 CLOSED. Severity trajectory: 2HIGH+3MED+LOWs → 0 → 1HIGH → 1HIGH+2MED+1LOW+2OBS → 1HIGH+3MED → 1LOW → 1MED+4OBS → 1MED+1LOW+2OBS → 1HIGH+2OBS → 1LOW+1OBS [CLEAN(PR-merge)] (decaying toward convergence).**
