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
date_pass11: 2026-07-10
streak_at_pass11: 1
date_pass12: 2026-07-10
streak_at_pass12: 0
date_pass13: 2026-07-10
streak_at_pass13: 1
date_pass14: 2026-07-10
streak_at_pass14: 0
date_pass15: 2026-07-10
streak_at_pass15: 1
date_pass16: 2026-07-10
streak_at_pass16: 0
date_pass17: 2026-07-10
streak_at_pass17: 1
date_pass18: 2026-07-10
streak_at_pass18: 0
date_pass19: 2026-07-10
streak_at_pass19: 0
date_pass20: 2026-07-10
streak_at_pass20: 0
date_pass21: 2026-07-10
streak_at_pass21: 0
total_passes_to_date: 21
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

## Cascade Table (21 passes to date)

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

| 11 | frozen @5a58046f | YES | YES | 0 | — | 1/3 |
| 12 | frozen @5a58046f | NO | YES | 1 LOW F-CSD-P12-001 (Ast::SqlPipe arm lacked pre_register_empty_tables call; SqlPipe head WHERE IN-subquery on 0-batch table → table-not-found → -32000; violates BC-2.11.005 v1.9 position-invariant DEC-022 in SqlPipe mode) | — | 0/3 |
| — (fix-burst) | — | — | — | — | test-writer @eaefee94 (T30/T31 RED + Filter/Pipe structural survey: no subquery atoms in filter_parser.rs/pipe_parser.rs — modes structurally exempt); implementer @421ce222 (one-line pre_register_empty_tables call in SqlPipe arm before plan_pinned_head_sql; design comment enumerates all 4 modes accurately); 31/31 defect suite; 96/96 temporal; 1533/1533 prism-query; just check 5456/5456 GREEN; non-exhaustive 89/89 | 0/3 |
| 13 | frozen @421ce222 | YES | YES | 0 | — | 1/3 |
| 14 | frozen @421ce222 | NO | YES | 1 MED F-CSD-P14-001 (empty MemTable virtual-field columns missing: `_sensor`/`_client`/`_source_table` not appended to schema before DataFusion registration; SELECT of virtual field from 0-batch side of JOIN → data-dependent -32000; violates BC-2.11.005 v1.9 DEC-022 in all 4 pre_register_empty_tables call sites) + 9 LOW/OBS (P14-002 umbrella-term note; P14-003 PipeStage::Join structurally moot; P14-004 T5 doc; P14-005 0-batch cache TTL no-action; P14-006 DTU-EXT-001 anchor no-action; P14-007 PostHostDetailsBody pub→pub(crate); P14-008 per-table debug lines no-action; P14-009 gate-precedence note; P14-010 SAP-2 §4-class TOML projection surface PENDING-HUMAN) | — | 0/3 |
| — (fix-burst) | — | — | — | — | test-writer @7f6db987 (T32/T33 RED: virtual-field SELECT from 0-batch side + T34 GREEN lock: populated side unaffected); implementer @87e8ff10 — `virtual_fields::append_virtual_fields_to_schema` single-definition helper with spoofed-column guard applied to all 4 `pre_register_empty_tables` call sites; taxonomy v2.38→v2.39 (P14-002/009 spec-closed); P14-007 pub→pub(crate) harness; 34/34 defect suite; 96/96 temporal; 1536/1536 prism-query; just check 5459/5459 GREEN; non-exhaustive 89/89 | 0/3 |
| 15 | frozen @87e8ff10 | YES | YES | 0 | — | 1/3 |
| 16 | frozen @87e8ff10 | NO | YES | 1 MED F-CSD-P16-001 (virtual-field behavior in BC-2.11.005 DEC-022 + BC-2.11.012 lacked test-specific spec anchors; tests T32/T33/T34 named against BC-2.11.012 but no spec invariant confirmed `_sensor`/`_client`/`_source_table` must appear in empty MemTable schema before registration — data-dependent contract gap) + 1 OBS F-CSD-P16-002 (fragmented comment block around VariantMeta insertion in `error_mapping.rs`; rustfmt long-match threshold anomaly documented) | — | 0/3 |
| — (fix-burst) | — | — | — | — | PO BC-2.11.005 v1.9→v1.10 (virtual-field append documented in DEC-022 §Postconditions; 3 test-vector rows cross-referenced to BC-2.11.012 v1.5; F-CSD-P16-001 spec-layer closure); BC-2.11.012 v1.4→v1.5 (Empty MemTable schema parity invariant added; 3 test-vector rows; POL-27 modified fix annotation; cross-referenced BC-2.11.005 v1.10 bidirectionally); implementer @819beeda — comment block re-flowed around VariantMeta insertion in `error_mapping.rs` + rustfmt long-match threshold anomaly corrected; prism-mcp 447/447; just check GREEN; non-exhaustive 89/89 | 0/3 |

| 17 | frozen @819beeda | YES | YES | 0 | — | 1/3 |
| 18 | frozen @819beeda | NO | YES | 7 OBS (F-CSD-P18-001..007; NOTE: adversary transcript labeled F-CSD-P17-NNN — canonical IDs are F-CSD-P18-001..007; pass 17 was CLEAN/zero-findings so no collision) | test-writer @b7a1fd93 + implementer @962f2ffb + BC-2.11.012 v1.5→v1.6 (PO, in dirty tree) + DTU-EXT-001 tech-debt-register amendment (state-manager) | 0/3 |
| — (fix-burst) | — | — | — | — | test-writer RED @b7a1fd93 (F-CSD-P18-001: T33b 6 nullable parity assertions; F-CSD-P18-006: devices T1 query_device_ids backward-ref assertion; F-CSD-P18-007: Test 3 wiremock POST body_partial_json pin); implementer @962f2ffb (F-CSD-P18-002: ALL FOUR branch-introduced volatile line-pins in materialization.rs swept to symbolic anchors TD-VSDD-091+TD-VSDD-060; F-CSD-P18-003: stale `#![allow(dead_code)]` on virtual_fields.rs removed); BC-2.11.012 v1.5→v1.6 PO adjudication (F-CSD-P18-005: status draft→active; changelog row; no semantic contract change); DTU-EXT-001 tech-debt-register amendment (F-CSD-P18-004: POST-method constraint added — DTU-EXT-001 future incidents route MUST land as POST; anchor: F-CSD-P18-004 + crowdstrike.sensor.toml [[tables]] incidents fetch_incidents method=POST body_template query_incident_ids); prism-query 1537/1537; just check FULL WORKSPACE 5460/5460 GREEN; non-exhaustive 89/89 | 0/3 |

Streak reset at passes 3, 4, 12, 14, 16, 18, 19, and 20. Pass 10 is the FIRST CLEAN(PR-merge) of the cascade. Pass 11 is the FIRST CLEAN(strict) of the cascade (streak 1/3); pass 12 LOW F-CSD-P12-001 reset to 0/3. Pass 13 (frozen @421ce222): CLEAN(strict)=YES — streak 1/3 (SECOND CLEAN(strict) of the cascade). Pass 14 (frozen @421ce222): MED F-CSD-P14-001 virtual-field schema parity gap — streak RESET 1/3→0/3. Pass 15 (frozen @87e8ff10): CLEAN(strict)=YES — streak 1/3 (THIRD CLEAN(strict) of the cascade). Pass 16 (frozen @87e8ff10): MED F-CSD-P16-001 virtual-field spec-anchor gap (BC-2.11.005 DEC-022 + BC-2.11.012 missing test-specific invariant) + OBS F-CSD-P16-002 comment fragmentation — streak RESET 1/3→0/3. Pass 17 (frozen @819beeda): CLEAN(strict)=YES CLEAN(PR-merge)=YES — ZERO findings. Streak 1/3 (per D-1666). Pass 18 (frozen @819beeda): NOT CLEAN(strict) CLEAN(PR-merge) — 7 OBS findings F-CSD-P18-001..007. Streak RESET 1/3→0/3. Pass 19 (frozen @962f2ffb): NOT CLEAN(strict) CLEAN(PR-merge) — 4 LOW findings F-CSD-P19-001..004. Streak RESET (stays 0/3). Mode-dimension survey complete: SQL+SqlPipe subquery atoms present and pre_register_empty_tables wired (both virtual-field and registration); Filter/Pipe parsers have no subquery atoms — structurally exempt. Structural-fix class closure: P5 (FuncCall-args recursion gap), P6 (DML source_select defense-in-depth), P7 (WHERE/HAVING/JOIN-ON check_sql_query non-recursive), P8-MED (gate placement below early-return; DUAL placement fix), P8-LOW (DML filter/assignments interiors), P9-HIGH (harness-clone POST verb surface gap), P10 (empirical DataFusion capability + harness first_seen OBS), P12 (SqlPipe pre_register_empty_tables), P14-001 (empty-MemTable virtual-field schema), P16-001 (virtual-field spec-anchor gap — spec layer closed via BC-2.11.005 v1.10 + BC-2.11.012 v1.5), P16-002 (comment block re-flowed), P18-001..007 (nullable parity T33b; volatile pins sweep; stale allow(dead_code); DTU-EXT-001 POST constraint; BC-2.11.012 lifecycle promote; T1 backward-ref; T3 body matcher), P19-001 (volatile pins test doc comments swept), P19-002 (Compare-arm InSubquery gate gap closed), P19-003 (PO split adjudication: _source_type implemented as 4th virtual field; _safety_flags retired; BC-2.11.012 v1.7), P19-004 (CWE-117 five tracing sites sanitize_for_log) — all closed. PENDING-HUMAN: DRIFT-SAP2-DEVICES-TOML-SURFACE-001 (P14-010 SAP-2 §4-class). SCRUTINY ITEM E-QUERY-038 second-emission-source CLOSED by pass-20 ARCHITECT Option A (D-1669): fallback REMOVED @e8f7dc8b; execute_against_session production-reachable ONLY via execute_inner plan-time gate first; runtime FieldNotFound = internal anomaly → QueryExecutionFailed; BC-2.11.016 v1.26 §Design Constraints codifies THREE emission sites. Pass 20 (frozen @7347bb16): CLEAN(strict)=NO CLEAN(PR-merge)=NO — 15 findings: 3 CRIT + 5 HIGH + 4 MED + 1 LOW + 2 OBS. Fix-burst: architect Option A fallback removal @e8f7dc8b; BC-2.11.012 v1.8 + BC-2.16.002 v2.09 + BC-2.11.016 v1.26; 4-commit fix chain @0d198d6d→e8f7dc8b→848ef359→14bffa1c; defect suite 41/41; just check FULL WORKSPACE 5466/5466 GREEN; non-exhaustive 89/89; prism-sensors 180/180. Streak RESET (stays 0/3). PENDING-HUMAN: DRIFT-SAP2-DEVICES-TOML-SURFACE-001 (P14-010 SAP-2 §4-class). NEW QUEUE: TD-S302-005 buffer-serving story (D-1669). Pass 21 (frozen @14bffa1c): CLEAN(strict)=NO CLEAN(PR-merge)=YES — 3 findings: 2 LOW + 1 OBS (F-CSD-P21-OBS-001 LOW four-field test coverage gap; F-CSD-P21-OBS-002 LOW CWE-117 sibling sites in materialization.rs; F-CSD-P21-OBS-003 OBS VirtualField variant count vs BC-2.11.012 v1.8 — TWO-STAGE PO ADJUDICATION → BC-2.11.012 v1.10 + DRIFT-INTERNAL-TABLE-COLUMN-GATE-001 queued). Fix-burst: test-writer @635bc31c (T32/T33/T37 four-field), implementer @b5caea48 (CWE-117 complete sweep 12 sites), implementer @7e528956 (docstring internal-table scope, TD-VSDD-060), BC-2.11.012 v1.9→v1.10 (POL-22 spec-code mismatch corrected), test-writer @e1a00fa3 (EC-11-035 lock → QueryExecutionFailed + negative ColumnNotFound assert). defect suite 42/42; just check FULL WORKSPACE 5467/5467 GREEN (60 skipped); non-exhaustive 89/89. Streak RESET (stays 0/3). NEW FROZEN HEAD e1a00fa3. NEW DRIFT ITEM: DRIFT-INTERNAL-TABLE-COLUMN-GATE-001 (extend plan-time column gate to internal tables; story-queued after CSDEVICES merge). LOCAL pass 22 NEXT on frozen HEAD `e1a00fa3` (streak 0/3).

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

### Pass 11

Zero findings. CLEAN(strict)=YES. CLEAN(PR-merge)=YES. Streak advances: 0/3→**1/3** — FIRST CLEAN(strict) of the cascade. LOCAL pass 12 DISPATCHED on frozen `5a58046f`.

### Pass 12

| ID | Severity | Description | Resolution |
|----|----------|-------------|------------|
| F-CSD-P12-001 | LOW | `Ast::SqlPipe` arm in `plan_pipeline` lacked `pre_register_empty_tables` call before `plan_pinned_head_sql`; SqlPipe head containing a WHERE IN-subquery on a 0-batch table returned table-not-found → -32000 instead of E-QUERY-043; violates BC-2.11.005 v1.9 position-invariant DEC-022 in SqlPipe mode | test-writer @eaefee94: T30 RED (SqlPipe head WHERE IN-subquery 0-batch path) + T31 RED (SqlPipe-specific lock); Filter/Pipe structural survey: no `Expr::InSubquery` / `Predicate::InSubquery` atoms reachable through filter_parser.rs / pipe_parser.rs — both modes structurally exempt; implementer @421ce222: one-line `pre_register_empty_tables(session, &spec)` call in SqlPipe arm before `plan_pinned_head_sql`; design comment updated to enumerate all 4 modes (SQL: pre_register_empty_tables present; SqlPipe: now present; Filter: structurally exempt; Pipe: structurally exempt); T30/T31 RED→GREEN; 31/31 defect suite GREEN |

CLEAN(strict): NO (1 LOW). CLEAN(PR-merge): YES. Streak RESET 1/3→0/3. LOCAL pass 13 DISPATCHED on frozen `421ce222`.

### Pass 17

Zero findings. CLEAN(strict)=YES. CLEAN(PR-merge)=YES. Streak advances: 0/3→**1/3** (FOURTH CLEAN(strict) of the cascade; FIRST on frozen `819beeda`). LOCAL pass 18 DISPATCHED on frozen `819beeda`.

### Pass 18

NOTE: adversary transcript labeled findings F-CSD-P17-NNN. Canonical IDs are F-CSD-P18-001..007 (pass 17 was CLEAN/zero-findings so no collision; this alias note is the authoritative record).

| ID | Severity | Description | Resolution |
|----|----------|-------------|------------|
| F-CSD-P18-001 | OBS | T33 lacked nullable=true/false per-path parity assertions claimed by BC-2.11.005 v1.10 + BC-2.11.012 T33 vector — test verified schema parity but did not assert per-field nullable values | test-writer @b7a1fd93: added sibling test `test_BC_2_11_012_F_CSD_P14_001_T33b_virtual_field_nullable_parity_per_path` (6 assertions: empty path nullable=true ×3 via append_virtual_fields_to_schema; populated path nullable=false ×3 via inject_virtual_fields) |
| F-CSD-P18-002 | OBS | Volatile `~line NNNN` pins in `check_expr_insubquery_projection` comments in `materialization.rs` — all four branch-introduced line pins violated TD-VSDD-091 (volatile-pin prohibition) | implementer @962f2ffb: swept ALL FOUR branch-introduced line pins to symbolic anchors; TD-VSDD-060 complete-set sweep via `git diff develop..HEAD` confirmed no residual pins |
| F-CSD-P18-003 | OBS | Stale module-wide `#![allow(dead_code)]` on `virtual_fields.rs` — all items have production callers; allow was from initial file creation and was never removed | implementer @962f2ffb: removed the module-level allow entirely; no per-item allows needed |
| F-CSD-P18-004 | OBS | `crowdstrike.sensor.toml` `fetch_incidents` now POST — DTU-EXT-001 forward constraint needed: future DTU incidents route MUST land as POST or the silent-empty defect class recurs | state-manager: amended tech-debt-register DTU-EXT-001 entry (formalization of pre-existing human-ratified notation) adding POST-method constraint with anchor "F-CSD-P18-004, crowdstrike.sensor.toml [[tables]] incidents fetch_incidents method=POST body_template query_incident_ids" |
| F-CSD-P18-005 | OBS | BC-2.11.012 `status: draft` contradicted `lifecycle_status: active` + sibling BC-2.11.005 `status: active` — draft misrepresents a contract whose implementation is on develop today | product-owner adjudicated PROMOTE: BC-2.11.012 v1.5→v1.6 (status draft→active; changelog row added; no semantic contract change); state-manager synced BC-INDEX row + v7.84→v7.85 |
| F-CSD-P18-006 | OBS | Devices Test 1 missing `query_device_ids` backward-ref assertion — asymmetric vs incidents Test 2 which asserts the request body backward-ref; incidents but not devices side of the test verified the request body shape | test-writer @b7a1fd93: added `query_device_ids` backward-ref assertion to Test 1 |
| F-CSD-P18-007 | OBS | Test 3 wiremock POST mock lacked body matcher — could not detect the exact sub-defect-1 regression class (TD-VSDD-059 paper-fix guard); a POST with a wrong or empty body would silently pass the test | test-writer @b7a1fd93: added `body_partial_json` ids pin to Test 3 POST mock |

CLEAN(strict): NO (7 OBS). CLEAN(PR-merge): YES. Streak RESET 1/3→0/3. New frozen HEAD for pass 19: `962f2ffb` (fix branch remains LOCAL-ONLY; no push). LOCAL pass 19 NEXT on frozen `962f2ffb`.

| 19 | frozen @962f2ffb | NO | YES | 4 LOW (F-CSD-P19-001 [process-gap] ≥11 volatile line pins in defect_csdevices_empty_memtable_tests.rs test doc comments — TD-VSDD-091 spirit (measurable ~80-line decay); F-CSD-P19-002 E-QUERY-043 scope hole — Expr::InSubquery at top level of Predicate::Compare LHS/RHS DESCENDED not REJECTED; constructed-AST T35 proved DataFusion SILENTLY EXECUTES returning WRONG RESULTS Ok/row_count 1; F-CSD-P19-003 parser maps `_source_type`/`_safety_flags` to VirtualField variants but runtime materializes only 3 columns → opaque failure; PO split adjudication: `_source_type` IMPLEMENT (S-3.02 delivery gap) / `_safety_flags` RETIRE (BC-2.09.004 §Invariants: safety flags centralized in _meta.safety_flags; AST variant speculative; SPEC WINS); BC-2.11.012 v1.6→v1.7; F-CSD-P19-004 CWE-117 five `%table_name` tracing sites in do_register_empty_mem_table + pre_register_empty_tables lacking sanitize_for_log) | — | 0/3 |
| — (fix-burst) | — | — | — | — | test-writer @7e8df858 T35 (F-CSD-P19-002: constructed-AST Predicate::Compare LHS/RHS E-QUERY-043 gate via contains_insubquery; T27 pattern); test-writer @d4c239f3 T36/T37/T38 RED (F-CSD-P19-003: T36 _source_type runtime materialization RED; T37 _source_type JOIN-side RED; T38 execute_against_session E-QUERY-038 mapping RED); implementer @80fd37fe (F-CSD-P19-002: check_predicate Compare arm → descend_subquery_expr → contains_insubquery; T35 RED→GREEN); implementer @7347bb16 (F-CSD-P19-001: ALL volatile pins in defect_csdevices_empty_memtable_tests.rs doc comments swept to symbolic anchors; F-CSD-P19-003: _source_type wired as 4th virtual field via inject_virtual_fields + append_virtual_fields_to_schema nullable=true empty path + spoof guard + runtime enum; VirtualField::SafetyFlags retired across ast.rs/pushdown.rs/pipe_sql_emitter.rs/explain.rs; parser test inverted — _safety_flags now parses as ordinary Expr::Field, not VirtualField; prism-mcp resources.rs doc listing updated; BC-2.11.012 v1.6→v1.7 H1 four-field title + §Description/§Preconditions/§Postconditions/§Invariants/§Edge Cases EC-11-032/033/034/§Canonical Test Vectors; F-CSD-P19-004: all five %table_name tracing sites wrapped with prism_core::sanitize_for_log); prism-query 1541/1541 (defect suite 39/39); prism-mcp 447/447; workspace 5464/5464 GREEN; non-exhaustive 89/89; SAP-1 zero new emissions | 0/3 |
| 20 | frozen @7347bb16 | NO | NO | 15 findings: 3 CRIT + 5 HIGH + 4 MED + 1 LOW + 2 OBS (F-CSD-P20-001 CRIT fallback payload spec-violations: client_id="", no Levenshtein, wrong JOIN table attribution, unsorted columns; F-CSD-P20-002 CRIT fallback missing column_not_found.rejected audit event; F-CSD-P20-003 CRIT undocumented 2nd E-QUERY-038 emission source / over-broad FieldNotFound→ColumnNotFound conversion at execute_against_session error catch; F-CSD-P20-004 HIGH T38 asserted variant-tag only — TD-VSDD-059; F-CSD-P20-005 HIGH inject_source_type zero production callers + BC v1.7 "buffered" dead-lettered; F-CSD-P20-006 HIGH TD-VSDD-060 detections.query_detection_ids retained variables_produced=["detection_ids"]; F-CSD-P20-007 HIGH CWE-117 fallback detail strings echo raw table_name in do_register_empty_mem_table; F-CSD-P20-008 HIGH CWE-117 sibling sweep register_mem_table 3 tracing sites + 2 detail strings; F-CSD-P20-009 MED EC-11-032/033 no code-path exercisers; F-CSD-P20-010 MED SAP-1 Ast::Sql(Select) planning-error catch unstructured; F-CSD-P20-011 MED T33b locked stale 3-field invariant; F-CSD-P20-012 MED T17 docstring embedded decay-prone historical RED hint snapshot; F-CSD-P20-013 LOW fallback available_columns org-scoping/CWE-200; F-CSD-P20-014 OBS inject_source_type doc claims production role; F-CSD-P20-015 OBS wildcard-arm "no E-QUERY-043 for Pipe/Filter" unlocked) | — | 0/3 |
| — (fix-burst) | — | — | — | — | ARCHITECT Option A (2026-07-10): call-graph proof — execute_against_session is production-reachable ONLY via engine.rs execute_inner which runs plan-time gate first; fallback could only mislabel internal anomalies as user errors; fallback REMOVED @e8f7dc8b; BC-2.11.016 v1.25→v1.26 §Design Constraints (E-QUERY-038 exclusively plan-time, three emission sites; runtime FieldNotFound = internal anomaly → QueryExecutionFailed + sql.sql_planning_error); adjudication record appended to rootcause research artifact. F-CSD-P20-001/002/013 MOOTED by Option A. PO (F-CSD-P20-005): EventStream buffer-serving path does NOT exist in production; inject_source_type fencing CORRECT; BC-2.11.012 v1.7→v1.8 "buffered" gated behind TD-S302-005; EC-11-032/033 annotated untestable-end-to-end pending story. PO (F-CSD-P20-010): BC-2.16.002 v2.08→v2.09 catalog row 92 sql.sql_planning_error (count 91→92). test-writer @0d198d6d: T33b 4-field + nullable both-path parity (P20-011); T39 constructed-AST wildcard lock (P20-015). implementer @e8f7dc8b: fallback removal + sql.sql_planning_error structured emission + pub(crate) gate + TOML detections variables_produced→[] + CWE-117 safe_table_name in do_register_empty_mem_table + sanitize_for_log sweep register_mem_table + T17 docstring pruned (P20-003/006/007/008/010/012). implementer @848ef359: inject_source_type module header + fn docstring + inject_virtual_fields docstring fenced (P20-014). test-writer @14bffa1c: T38 re-pointed to pub(crate) check_query_column_availability with FULL payload assertions (column/table/client_id/sorted available_columns/did_you_mean None) + T38b runtime lock (execute_against_session → QueryExecutionFailed, NOT ColumnNotFound — Option A boundary locked from both sides) (P20-004). F-CSD-P20-009 RESOLVED-BY-FENCING (BC-2.11.012 v1.8 annotates untestable pending TD-S302-005 story). New FROZEN HEAD 14bffa1c. defect suite 41/41; just check FULL WORKSPACE 5466/5466 GREEN; non-exhaustive 89/89; prism-sensors 180/180. | 0/3 |
| 21 | frozen @14bffa1c | NO | YES | 3 findings: 2 LOW + 1 OBS (F-CSD-P21-OBS-001 LOW: T32/T33 DataFusion-path integration tests locked only 3 of 4 virtual fields; T37 stale "three-field" docstring; F-CSD-P21-OBS-002 LOW: CWE-117 sibling sites in materialization.rs unsanitized — check_ci_column_types + source_table/source_name emission sites; F-CSD-P21-OBS-003 OBS: prism_core::VirtualField 3 variants vs BC-2.11.012 v1.8 four-field workspace claim → TWO-STAGE PO ADJUDICATION; all core contracts verified PASS: Option A boundary, catalog row 92 parity, E-QUERY-043 byte-lock, four-field set, SafetyFlags retirement zero live refs, CWE-117 at swept sites, SAP-1/SAP-2 clean) | test-writer @635bc31c + implementer @b5caea48 + implementer @7e528956 + PO BC-2.11.012 v1.9→v1.10 + test-writer @e1a00fa3 | 0/3 |
| — (fix-burst) | — | — | — | — | F-CSD-P21-OBS-001: test-writer @635bc31c — T32 SELECT+NULL-assert extended to all 4 virtual fields; T33 expected array extended to 4 fields; T37 docstring refreshed to four-field set. F-CSD-P21-OBS-002: implementer @b5caea48 — complete-set sweep, 12 sites wrapped (incl. 4 client-facing detail strings), 9 exempt with per-site classification (SQL-string indirect category + validated newtypes SensorId/OrgSlug/OrgId); 1543/1543 GREEN. F-CSD-P21-OBS-003 TWO-STAGE PO ADJUDICATION: Stage 1 — FENCE: `_source_type` is sensor-table-only; internal tables keep 3 fields + _meta_scan_truncated; BC-2.11.012 v1.8→v1.9 (internal-table exclusion, EC-11-035 scope clarification, invariants split sensor/internal); implementer @7e528956: prism-core VirtualField docstring scoped to internal tables (TD-VSDD-060 doc sweep: 1 fixed, 3 exempt). Stage 2 — SPEC-CODE MISMATCH CAUGHT (POL-22): v1.9 EC-11-035 claimed `SELECT _source_type FROM prism_alerts → E-QUERY-038` based on unverified PO investigation claim. Test-writer REFUSED to lock it: verified real behavior is QueryExecutionFailed (two mechanisms: intentional three-mode `starts_with("prism_")` fail-open in check_query_column_availability; Expr::VirtualField always-valid skip in extract_field_paths_from_expr). PO corrected same-burst: BC-2.11.012 v1.9→v1.10 — EC-11-035 + test vector re-locked to QueryExecutionFailed (consistent with BC-2.11.016 v1.26 §Design Constraints); v1.9 mis-citation noted in changelog per POL-22; Option B (extend gate to return E-QUERY-038) rejected as requiring architectural design (partial-gate inconsistency risk). test-writer @e1a00fa3: `test_BC_2_11_012_EC_11_035_source_type_on_internal_table_returns_query_execution_failed` with negative ColumnNotFound assertion. defect suite 42/42; just check FULL WORKSPACE 5467/5467 GREEN (60 skipped); non-exhaustive 89/89. New FROZEN HEAD e1a00fa3. Streak 0/3. | 0/3 |

---

### Pass 21

| ID | Severity | Description | Resolution |
|----|----------|-------------|------------|
| F-CSD-P21-OBS-001 | LOW | T32 and T33 (DataFusion-path integration tests) locked only 3 of 4 virtual fields; `_source_type` added by @7347bb16 was not included in the SELECT assertion list or the expected-row array. T37 docstring still read "three-field" after the four-field promotion in BC-2.11.012 v1.8. | test-writer @635bc31c: T32 SELECT + NULL-assert extended to all 4 virtual fields (`_sensor`, `_client`, `_source_table`, `_source_type`); T33 expected array extended to 4 fields; T37 docstring refreshed to four-field set. |
| F-CSD-P21-OBS-002 | LOW | CWE-117 sibling sites in `materialization.rs` unsanitized after the P20 sweep: `check_ci_column_types` assertion detail strings and `source_table`/`source_name` emission sites passed raw user-controlled values to `tracing::*!` without `sanitize_for_log`. | implementer @b5caea48: complete-set sweep across materialization.rs — 12 sites wrapped with `sanitize_for_log`; 9 sites explicitly classified exempt (SQL-string indirect category; validated newtypes `SensorId`/`OrgSlug`/`OrgId` already redact on `Debug`); 1543/1543 GREEN. |
| F-CSD-P21-OBS-003 | OBS | `prism_core::VirtualField` enum had 3 variants (`Sensor`, `Client`, `SourceTable`) while BC-2.11.012 v1.8 claimed a four-field workspace. TWO-STAGE PO ADJUDICATION: Stage 1 — PO investigation: `_source_type` is sensor-table-only; internal tables (`prism_*`) retain 3 virtual fields + `_meta_scan_truncated`. BC-2.11.012 v1.8→v1.9: internal-table exclusion clause + EC-11-035 + invariant split sensor/internal. Implementer @7e528956: VirtualField docstring scoped to internal-table context (TD-VSDD-060 doc sweep: 1 site fixed, 3 exempt). Stage 2 — SPEC-CODE MISMATCH (POL-22): v1.9 EC-11-035 claimed `SELECT _source_type FROM prism_alerts → E-QUERY-038` based on unverified PO claim about `check_query_column_availability`. Test-writer REFUSED to lock: verified actual behavior = `QueryExecutionFailed` (two independent mechanisms: three-mode `starts_with("prism_")` fail-open in `check_query_column_availability`; `Expr::VirtualField` always-valid skip in `extract_field_paths_from_expr`). PO corrected same-burst: BC-2.11.012 v1.9→v1.10 — EC-11-035 + test vector re-locked to `QueryExecutionFailed` consistent with BC-2.11.016 v1.26 §Design Constraints; v1.9 mis-citation noted in changelog; Option B (extend gate to return E-QUERY-038) rejected as requiring architectural design (partial-gate inconsistency risk → DRIFT-INTERNAL-TABLE-COLUMN-GATE-001 queued). Test-writer @e1a00fa3: `test_BC_2_11_012_EC_11_035_source_type_on_internal_table_returns_query_execution_failed` with `is_err()` assertion + explicit negative `ColumnNotFound` assert. |

CLEAN(strict): NO (2 LOW + 1 OBS). CLEAN(PR-merge): YES. Streak stays 0/3. Fix-burst chain: @b5caea48 → @7e528956 → @635bc31c → @e1a00fa3. New FROZEN HEAD for pass 22: `e1a00fa3`. LOCAL pass 22 NEXT on frozen `e1a00fa3` (streak 0/3).

---

## Evidence at Pass 21 Fix HEAD (e1a00fa3)

- prism-query defect suite: **42/42** GREEN
- Full workspace `just check`: **5467/5467** GREEN (60 skipped)
- Non-exhaustive gate: 89/89
- SAP-1 sweep: zero new `event_type =` emissions
- SAP-2 sweep: CLEAN (no new TOML column vs DTU field divergence introduced)

## Spec Layer Modified (pass-21 closure)

| Artifact | Version Bump | Change |
|----------|-------------|--------|
| BC-2.11.012 | v1.8→v1.10 | TWO-STAGE adjudication: v1.9 (F-CSD-P21-OBS-003 Stage 1 — internal-table exclusion clause; `_source_type` sensor-table-only; invariants split sensor/internal; EC-11-035 draft); v1.10 (F-CSD-P21-OBS-003 Stage 2 — POL-22 spec-code mismatch correction; EC-11-035 + test vector re-locked to `QueryExecutionFailed`; v1.9 mis-citation noted per POL-22 changelog discipline; Option B rejected) |
| BC-INDEX | v7.87→v7.88 | BC-2.11.012 row v1.8→v1.10; changelog entry D-1670 |

---

## Evidence at Pass 20 Fix HEAD (14bffa1c)

- prism-query defect suite: **41/41** GREEN
- Full workspace `just check`: **5466/5466** GREEN
- Non-exhaustive gate: 89/89
- prism-sensors: **180/180** GREEN
- SAP-1 sweep: `sql.sql_planning_error` structured emission added @e8f7dc8b; catalog row 92 registered in BC-2.16.002 v2.09

## Spec Layer Modified (pass-20 closure)

| Artifact | Version Bump | Change |
|----------|-------------|--------|
| BC-2.11.016 | v1.25→v1.26 | F-CSD-P20-003 architect Option A: §Design Constraints added — E-QUERY-038 exclusively plan-time; THREE emission sites documented; runtime FieldNotFound = internal anomaly → QueryExecutionFailed + sql.sql_planning_error; fallback removed @e8f7dc8b |
| BC-2.11.012 | v1.7→v1.8 | F-CSD-P20-005 PO investigation: EventStream buffer-serving path not in production; inject_source_type fencing CORRECT; "buffered" condition gated behind TD-S302-005 story; EC-11-032/033 annotated untestable-end-to-end pending TD-S302-005 delivery |
| BC-2.16.002 | v2.08→v2.09 | F-CSD-P20-010 SAP-1 gap: catalog row 92 `sql.sql_planning_error` added (count 91→92) |
| BC-INDEX | v7.86→v7.87 | BC-2.11.016 row v1.25→v1.26; BC-2.11.012 row v1.7→v1.8; BC-2.16.002 row v2.08→v2.09; changelog entry D-1669 |
| research/defect-csdevices-empty-pipeline-rootcause-2026-07-10.md | — | §F-CSD-P20-003 architect adjudication section appended (Option A call-graph proof) |

---

## Evidence at Pass 19 Fix HEAD (7347bb16)

- prism-query defect suite: **39/39** GREEN (1541/1541 total prism-query)
- prism-mcp: **447/447** GREEN
- Full workspace `just check`: **5464/5464** GREEN
- Non-exhaustive gate: 89/89
- SAP-1 sweep: zero new `event_type =` emissions

## Spec Layer Modified (pass-19 closure)

| Artifact | Version Bump | Change |
|----------|-------------|--------|
| BC-2.11.012 | v1.6→v1.7 | F-CSD-P19-003 PO split adjudication: H1 title updated to four fields; `_source_type` wired as 4th virtual field (S-3.02 delivery gap); `_safety_flags` retired (BC-2.09.004 §Invariants; spec wins); §Description/§Preconditions/§Postconditions/§Invariants/§Edge Cases EC-11-032/033/034/§Canonical Test Vectors updated |
| BC-INDEX | v7.85→v7.86 | BC-2.11.012 inline row: version v1.6→v1.7, title updated to four-field set; changelog entry D-1668 |

---

## Evidence at pass 18 Fix HEAD (962f2ffb)

- prism-query defect suite: **1537/1537** GREEN
- Full workspace `just check`: **5460/5460** GREEN
- Non-exhaustive gate: 89/89

## Spec Layer Modified (pass-18 closure)

| Artifact | Version Bump | Change |
|----------|-------------|--------|
| BC-2.11.012 | v1.5→v1.6 | F-CSD-P18-005: `status: draft → active`; changelog row added; no semantic contract change; production-grade default requires removing the false-draft label (inject_virtual_fields already on develop) |
| tech-debt-register.md | v2.23→v2.24 (amendment) | F-CSD-P18-004: DTU-EXT-001 formalized as a tech-debt row; POST-method constraint added — DTU-EXT-001 future CrowdStrike incidents route MUST land as POST handler for `/incidents/entities/incidents/GET/v1`; silent GET would reproduce the empty-pipeline defect class; anchor: crowdstrike.sensor.toml [[tables]] incidents fetch_incidents method=POST body_template query_incident_ids |
| BC-INDEX | v7.84→v7.85 | BC-2.11.012 inline row: version v1.5→v1.6, status draft→active; changelog entry D-1667 |

---

## Evidence at Pass 12 Fix HEAD (421ce222)

- prism-query defect suite: **31/31** GREEN
- temporal tests: **96/96** GREEN
- full prism-query suite: **1533/1533** GREEN
- Full workspace `just check`: **5456/5456** GREEN
- Non-exhaustive gate: 89/89

## Spec Layer Modified (pass-12 closure)

No spec changes — code-only fix. One-line `pre_register_empty_tables` call added to SqlPipe arm in `plan_pipeline`; design comment updated to document all 4 mode exemptions. No BC or error-taxonomy version bumps (F-CSD-P12-001 closed entirely within existing BC-2.11.005 v1.9 DEC-022 contract).

---

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

### Pass 13

Zero findings. CLEAN(strict)=YES. CLEAN(PR-merge)=YES. Streak advances: 0/3→**1/3** — SECOND CLEAN(strict) of the cascade; FIRST CLEAN(strict) on frozen `421ce222`. LOCAL pass 14 DISPATCHED on frozen `421ce222`.

### Pass 14

| ID | Severity | Description | Resolution |
|----|----------|-------------|------------|
| F-CSD-P14-001 | MED | Empty MemTable registrations lacked virtual-field columns `_sensor`/`_client`/`_source_table`; schema-only MemTable registered with base spec columns only; SELECT of a virtual field from 0-batch side of a JOIN → column not found in DataFusion → data-dependent -32000; violates BC-2.11.005 v1.9 DEC-022 position-invariant guarantee in all 4 `pre_register_empty_tables` call sites | test-writer @7f6db987: T32 RED (virtual-field SELECT on 0-batch side) + T33 RED (multi-table 0-batch path) + T34 GREEN lock (populated side unaffected); implementer @87e8ff10: `virtual_fields::append_virtual_fields_to_schema` single-definition helper with spoofed-column guard; applied to all 4 call sites; 34/34 defect suite GREEN |
| F-CSD-P14-002 | LOW | E-QUERY-043 description missing umbrella-term note clarifying "projection position" covers SELECT/GROUP BY/ORDER BY | CLOSED at spec: taxonomy v2.38→v2.39 — umbrella-term note appended to Gate scope clause |
| F-CSD-P14-003 | LOW | PipeStage::Join path in `plan_pipeline` lacks `pre_register_empty_tables` call | Structurally moot: PipeStage::Join errors pre-registration until ENRICH-4-C (ADR-044 §D3); design comment added at call site by implementer |
| F-CSD-P14-004 | LOW | T5 doc-comment claimed "SELECT * FROM sensors.devices" but actual test uses a specific column list | Doc corrected in-scope by implementer |
| F-CSD-P14-005 | OBS | 0-batch cache TTL policy lacks explicit WARN-level log | No-action adjudicated: pre-existing TTL policy; fix improved observability in-scope; no BC change required |
| F-CSD-P14-006 | OBS | DTU-EXT-001 anchor missing from incidents SAP-2 note | No-action: DTU-EXT-001 anchor already present in cascade-summary; pre-existing anchor adequate |
| F-CSD-P14-007 | LOW | `PostHostDetailsBody` and its handler pub visibility unnecessarily broad in harness | Fixed in-scope: `PostHostDetailsBody` + handler `pub` → `pub(crate)` in harness (harness-twin already `pub(crate)`) |
| F-CSD-P14-008 | OBS | Per-table debug log lines in `pre_register_empty_tables` are voluminous under high batch counts | No-action adjudicated: per-table debug lines are intentional diagnosability; log level correct (debug, not info) |
| F-CSD-P14-009 | OBS | E-QUERY-043 description missing gate-precedence note (E-QUERY-042 fires before E-QUERY-043) | CLOSED at spec: taxonomy v2.38→v2.39 — precedence sentence appended to Gate ordering clause |
| F-CSD-P14-010 | OBS | SAP-2 §4-class: DTU `PostHostDetailsBody` fields without matching TOML sensor-spec column (pre-existing develop gap; not a pass-14 regression) | PENDING-HUMAN: registered as DRIFT-SAP2-DEVICES-TOML-SURFACE-001 in STATE.md Drift Items; product-owner adjudicates whether to expand devices TOML projection surface |

CLEAN(strict): NO (1 MED). CLEAN(PR-merge): YES (9 LOW/OBS only below MED threshold is incorrect — F-CSD-P14-001 is MED). Actually: NOT CLEAN(PR-merge) either since F-CSD-P14-001 is MED. Streak RESET 1/3→0/3. LOCAL pass 15 DISPATCHED on frozen `87e8ff10`.

## Evidence at Pass 14 Fix HEAD (87e8ff10)

- prism-query defect suite: **34/34** GREEN
- temporal tests: **96/96** GREEN
- full prism-query suite: **1536/1536** GREEN
- Full workspace `just check`: **5459/5459** GREEN
- Non-exhaustive gate: 89/89

## Spec Layer Modified (pass-14 closure)

| Artifact | Version Bump | Change |
|----------|-------------|--------|
| error-taxonomy.md | v2.38→v2.39 | F-CSD-P14-002: umbrella-term note appended to E-QUERY-043 Gate scope clause; F-CSD-P14-009: gate-precedence sentence appended to Gate ordering clause; Message Format byte-lock UNCHANGED |

No BC version bumps. Code-only fix for F-CSD-P14-001 (virtual_fields::append_virtual_fields_to_schema helper + 4 call-site applications). Harness pub-visibility fix for F-CSD-P14-007.

---

## Merge Record

_Pending: LOCAL cascade not yet converged. PR not yet created._

**Status: IN PROGRESS — LOCAL pass 22 NEXT on frozen `e1a00fa3` (streak 0/3). Pass-21 on frozen `14bffa1c`: CLEAN(strict)=NO CLEAN(PR-merge)=YES — 3 findings: 2 LOW + 1 OBS. Fix-burst: test-writer @635bc31c (T32/T33/T37 four-field); implementer @b5caea48 (CWE-117 complete sweep 12 sites); implementer @7e528956 (docstring internal-table scope; TD-VSDD-060 doc sweep); BC-2.11.012 v1.8→v1.10 TWO-STAGE PO ADJUDICATION (POL-22 spec-code mismatch corrected); test-writer @e1a00fa3 (EC-11-035 lock → QueryExecutionFailed + negative ColumnNotFound). defect suite 42/42; just check FULL WORKSPACE 5467/5467 GREEN (60 skipped); non-exhaustive 89/89. Streak RESET (stays 0/3). New FROZEN HEAD for pass 22: e1a00fa3 (LOCAL-ONLY). PENDING-HUMAN: DRIFT-SAP2-DEVICES-TOML-SURFACE-001 (P14-010 SAP-2 §4-class). NEW DRIFT ITEM: DRIFT-INTERNAL-TABLE-COLUMN-GATE-001 (story-queued after CSDEVICES merge). Walker-gap + gate-placement lineage P5/P6/P7/P8 CLOSED; harness parity gap P9 CLOSED; empirical DataFusion capability P10 CLOSED; SqlPipe mode gap P12 CLOSED; virtual-field schema gap P14-001 CLOSED; virtual-field spec-anchor gap P16-001 CLOSED; nullable parity + volatile-pins + lifecycle + DTU-constraint + backward-refs + body-matcher P18 CLOSED; volatile test-doc-pins + Compare-arm InSubquery gate + _source_type/safety_flags PO-split + CWE-117 tracing P19 CLOSED; E-QUERY-038 2nd-source + over-broad FieldNotFound conversion + TD-VSDD-059/060 + CWE-117 sibling + SAP-1 sql.sql_planning_error + Option-A BC amendments P20 CLOSED; four-field test coverage + CWE-117 sibling sweep + BC-2.11.012 v1.10 POL-22 spec-mismatch + EC-11-035 lock P21 CLOSED. Severity trajectory: 2HIGH+3MED+LOWs → 0 → 1HIGH → 1HIGH+2MED+1LOW+2OBS → 1HIGH+3MED → 1LOW → 1MED+4OBS → 1MED+1LOW+2OBS → 1HIGH+2OBS → 1LOW+1OBS [CLEAN(PR-merge)] → 0 [streak-1] → 1LOW [reset] → 0 [streak-1] → 1MED+9LOW/OBS [reset] → 0 [streak-1] → 1MED+1OBS [reset] → 0 [streak-1] → 7OBS [reset] → 4LOW [reset] → 3CRIT+5HIGH+4MED+1LOW+2OBS [reset] → 2LOW+1OBS [reset] (converging toward 3-CLEAN).**
