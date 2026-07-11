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
date_pass22: 2026-07-10
streak_at_pass22: 0
date_pass23: 2026-07-10
streak_at_pass23: 0
date_pass24: 2026-07-10
streak_at_pass24: 0
date_pass25: 2026-07-11
streak_at_pass25: 0
date_pass26: 2026-07-11
streak_at_pass26: 0
date_pass27: 2026-07-11
streak_at_pass27: 1
date_pass28: 2026-07-11
streak_at_pass28: 0
date_pass29: 2026-07-11
streak_at_pass29: 0
date_pass30: 2026-07-11
streak_at_pass30: 0
date_pass31: 2026-07-11
streak_at_pass31: 0
date_pass32: 2026-07-11
streak_at_pass32: 0
total_passes_to_date: 32
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

## Cascade Table (28 passes to date)

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
| 24 | frozen @4f084a31 | NO | NO | 3 MED + 2 OBS (F-CSD-P24-001/002/003/OBS-002/OBS-003; quick-reference rows, docstrings, SqlPipe gate lock, store_step_vars locks) | implementer @6a913680 + test-writer @0d07be7e | 0/3 |
| — (fix-burst) | — | — | — | — | implementer @6a913680 (E-QUERY-041/042/043 quick-reference 3 rows added + CI-parity gate extended; parser_tests.rs five→four docstring ×2; overlay_loading_tests.rs EXEMPT); test-writer @0d07be7e (T40 SqlPipe head InSubquery E-QUERY-043 lock; 3 store_step_vars unit tests; T39 re-anchored to T2+T40); just check FULL WORKSPACE 5472/5472 GREEN; prism-mcp 447/447; non-exhaustive 89/89. New FROZEN HEAD for pass 25: 0d07be7e (LOCAL-ONLY). Streak RESET (stays 0/3). | 0/3 |
| 25 | frozen @0d07be7e | NO | YES | 5 LOW + 1 OBS (F-CSD-P25-001 LOW [[test]] required-features; F-CSD-P25-002 LOW 3 stale v1.9 DEC-022 cite-pins; F-CSD-P25-003 LOW NegativeE043 parity gate gap; F-CSD-P25-004 LOW SqlPipe stages walk undocumented; F-CSD-P25-005 LOW TimestampArithmetic unlocked; F-CSD-P25-006 OBS POL-33 Route Coverage Table absent from BC-2.16.013). Adversary novelty: LOW. SAP-1/SAP-2/POL-24 clean. | test-writer @437dac0e + implementer @99719a7a + PO BC-2.16.013 v1.28 | 0/3 |
| — (fix-burst) | — | — | — | — | test-writer @437dac0e (negative_e043_parity_gate.rs 2 RED + T41 SqlPipe stage-walk RED + T42 TimestampArithmetic GREEN lock ADR-052 §D4); implementer @99719a7a (ExampleKind::NegativeE043 + REFERENCE_EXAMPLES + build_reference_content + SqlPipe spq.stages walk + [[test]] required-features + cite-pin sweep 3+4 sites; just check 5476/5476 GREEN, 60 skipped; prism-mcp 449/449; prism-query 1548/1548; non-exhaustive 89/89; SAP-1 zero new emissions); PO BC-2.16.013 v1.27→v1.28 Route Coverage Table (9 rows POL-33) + SESSION-HANDOFF.md v1.28 cite-pin propagation. New FROZEN HEAD 99719a7a (LOCAL-ONLY). Streak RESET → 0/3. LOCAL pass 26 NEXT on frozen 99719a7a. | 0/3 |
| 26 | frozen @99719a7a | NO | YES | 3 OBS (F-CSD-P26-OBS-001 "3-tier" gate docstring stale — catalog now 4 tiers; F-CSD-P26-OBS-002 Ast::Pipe vs Ast::SqlPipe stage-walk asymmetry — ARCHITECT ADJUDICATION Option B; F-CSD-P26-OBS-003 SAP-2 devices DTU-fields-not-in-TOML gap — CLOSED-BY-EXISTING-DEFERRAL DRIFT-SAP2-DEVICES-TOML-SURFACE-001). Adversary novelty: LOW. SAP-1 PASS (rows 179–183 verified). SAP-2 PASS (no new TOML↔DTU divergence). POL-22 PASS. POL-33 PASS. | test-writer @3202d80f + implementer @9fe2d016 (architect Option B adjudication D-1675) | 0/3 |
| — (fix-burst) | — | — | — | — | test-writer @3202d80f (test_bc_2_11_022_ci_3tier_gate → test_bc_2_11_022_ci_4tier_gate; has_negative_e043 assertion added; exhaustiveness-stub NegativeE043 arm extended; T39 Pipe sub-assertion emitter-boundary comment added; TD-VSDD-060 sibling sweep in negative_e043_parity_gate.rs); implementer @9fe2d016 (build_reference_content "3-tier" stale prose → "4-tier" design note; materialization.rs wildcard-arm comment → architect-ratified emitter-boundary rationale citing pipe_sql_emitter::predicate_to_datafusion_sql; emitter claim re-verified; just check FULL WORKSPACE GREEN 5476 @9fe2d016; non-exhaustive 89/89; SAP-1 zero new emissions). OBS-003 CLOSED-BY-EXISTING-DEFERRAL (no code change; DRIFT-SAP2-DEVICES-TOML-SURFACE-001 D-1666 decision 4). New FROZEN HEAD 9fe2d016 (LOCAL-ONLY). Streak RESET → 0/3. LOCAL pass 27 NEXT on frozen 9fe2d016. | 0/3 |
| 27 | frozen @9fe2d016 | YES | YES | 0 | — | **1/3** |
| 28 | frozen @9fe2d016 | NO | YES | 1 OBS F-CSD-P28-OBS-001 (`prism_core::virtual_fields::VirtualField` missing `#[non_exhaustive]`; 3-variant pub+Deserialize; CLAUDE.md convention unambiguous; sibling prism_query::ast::VirtualField already compliant) | implementer @25b80a81 (annotation added; TD-VSDD-060 sweep; gate 89→90; just check 5476/5476 GREEN @25b80a81; non-exhaustive 90/90 ON BRANCH) | 0/3 |
| — (fix-burst) | — | — | — | — | implementer @25b80a81 (`#[non_exhaustive]` added to `prism_core::virtual_fields::VirtualField`; TD-VSDD-060 sweep confirms only match site is in defining crate; `prism-storage` uses method calls only; all other refs are `prism_query::ast::VirtualField` — separate type; gate `v90_virtual_field_match` E0004 added to `tests/external/non-exhaustive-violation/src/lib.rs`; EXPECTED 89→90 in `ci.yml` both message lists + changelog + `scripts/check-non-exhaustive.sh`; CLAUDE.md convention sentence 89→90 + provenance entry; just check FULL WORKSPACE GREEN; non-exhaustive 90/90 ON BRANCH). New FROZEN HEAD 25b80a81 (LOCAL-ONLY). Streak RESET → 0/3. LOCAL pass 29 NEXT on frozen 25b80a81. | 0/3 |
| 29 | frozen @25b80a81 | NO | NO | 5 MED + 1 OBS + 1 PROCESS-GAP (F-CSD-P29-001 MED ci.yml FAIL message omits TemporalLiteralPosition; F-CSD-P29-002 MED resources.rs 4 stale "3-tier" doc sites + NegativeE043 absent from contract enumeration; F-CSD-P29-003 MED BC-2.11.022 3-variant ExampleKind spec-code drift from F-CSD-P25-003; F-CSD-P29-004 MED ci.yml "ADR-045 3-tier gate" stale; F-CSD-P29-005 MED ci.yml "47+17=64" stale; F-CSD-P29-006 MED harness detection_detail() 4-field thin shape vs 12 TOML columns SAP-2 — architect IN-SCOPE-FIX; F-CSD-P29-OBS-001 PROCESS-GAP count-only CI gate undetectable net-zero regressions; F-CSD-P29-OBS-002 OBS fetch_incidents POST shape zero DTU coverage until DTU-EXT-001 no-action) | implementer @3dd1fd96 + test-writer @74c578c3 + implementer @7a6f6caa + PO BC-2.11.022 v1.3 + PO BC-2.16.013 v1.29 | 0/3 |
| — (fix-burst) | — | — | — | — | implementer @3dd1fd96 (P29-001/004/005 + OBS-001: 106-line inline ci.yml step → single `bash scripts/check-non-exhaustive.sh` call; new `scripts/check-non-exhaustive-per-symbol.py` explicit 90-symbol list `len==90` import guard, E0639+E0004 parse, 0 UNKNOWN; `check-non-exhaustive.sh` two-layer Layer 1 count + Layer 2 per-symbol; P29-002 resources.rs 4 sites 4-tier + NegativeE043; POL-25 worktree grep clean); PO BC-2.11.022 v1.2→v1.3 (P29-003 spec-code drift: ExampleKind 4-variant; Tier 3 NegativeE043 gate item; EC-11-022-007; test vector); PO BC-2.16.013 v1.28→v1.29 (P29-006 spec-note: INV-HARNESS-ROUTE-PARITY detection_detail() response-shape clause); SESSION-HANDOFF.md v1.28→v1.29 cite-pin; test-writer RED @74c578c3 (test_F_CSD_P29_006_detection_detail_full_toml_field_coverage); implementer GREEN @7a6f6caa (detection_detail() 4→12 fields; ioc_value explicit null; nested device retained; TD-VSDD-060 sweep clean); just check FULL WORKSPACE 5477/5477 GREEN (60 skipped); prism-dtu-harness 142/142; non-exhaustive 90/90 two-layer per-symbol. New FROZEN HEAD 7a6f6caa (LOCAL-ONLY). Streak 0/3. Cascade 29 passes (commits: 3dd1fd96, 74c578c3, 7a6f6caa). | 0/3 |
| 30 | frozen @7a6f6caa | NO | **YES** | 4 total (0 CRIT / 0 HIGH / 0 MED / 2 LOW / 2 OBS): F-CSD-P30-OBS-001 LOW enum_violations.rs doc-comment claimed 21 (truth 22); F-CSD-P30-OBS-002 LOW struct_violations.rs claimed 60 (truth 68); F-CSD-P30-OBS-003 OBS device_id="placeholder" not in host pool → harness-mode detections⋈devices JOIN silently 0 rows (ARCHITECT Option A D-1679 2026-07-11); F-CSD-P30-OBS-004 OBS duplicate numeric comment labels v70/v86 in per-symbol script. Adversary verified: per-symbol gate design sound; two-layer wiring fail-closed; ci.yml replacement correct (python3 present); detection_detail 12-field BC-2.16.013 v1.29 clause exact; BC-2.11.022 v1.3 four-assertion gate load-bearing; SAP-1/SAP-2/POL-22/POL-33 all clean. Novelty LOW. | implementer @3a9ec741 + test-writer @c26a74ef + implementer @ed2988cc + PO BC-2.16.013 v1.29→v1.30 | 0/3 |
| — (fix-burst) | — | — | — | — | implementer @3a9ec741 (OBS-001: enum_violations.rs doc-comment rebuilt from actual fns 22 entries; OBS-002: struct_violations.rs rebuilt 68 entries; 22+68=90 cross-checked against len==90 guard; OBS-004: duplicate labels v70/v86 → v70-struct/v70-enum + v86-struct/v86-enum suffixes; gate both layers PASS); PO BC-2.16.013 v1.29→v1.30 (OBS-003 host-pool constraint clause: device_id MUST be generate_host_ids(org_slug,seed)[det_index%HOST_COUNT]; placeholder strings forbidden; harness-mode JOIN non-empty requirement; SESSION-HANDOFF v1.29→v1.30 cite-pin); test-writer @c26a74ef RED (test_F_CSD_P30_OBS_003_detection_device_id_is_valid_host_id_not_placeholder + test_F_CSD_P30_OBS_003_detection_device_ids_join_devices_nonempty JOIN-fidelity lock); implementer @ed2988cc GREEN (detection_detail(detection_id, det_index, org_slug, seed); generate_host_ids modulo at root + nested device_id; sole call site .enumerate() threaded; P29-006 test still GREEN); just check FULL WORKSPACE 5479/5479 GREEN; prism-dtu-harness 144/144; non-exhaustive 90/90 two-layer per-symbol. New FROZEN HEAD ed2988cc (LOCAL-ONLY). Streak 0/3. Cascade 30 passes (commits: 3a9ec741, c26a74ef, ed2988cc). Develop baseline UNCHANGED @b9cf3f9b. | 0/3 |
| 31 | frozen @ed2988cc | NO | NO | 1 MED + 1 LOW + 1 OBS (F-CSD-P31-MED-001 MED SAP-2 severity INTEGER 50 vs TOML string + standalone string labels; F-CSD-P31-OBS-001 LOW det_index batch-position non-stable mapping; F-CSD-P31-OBS-002 OBS ast::VirtualField no compile-fail gate + Layer-2 key collision). SAP-1 PASS; SAP-2 devices PASS; SAP-2 detections FAIL → MED-001. POL-22/24/33 PASS. EXPECTED=90 PASS. BC-2.16.013 v1.30 host-pool clause PASS. Adversary novelty: LOW-MEDIUM. | test-writer @36f0ba9c + BC-2.16.013 v1.30→v1.31 (PO) + implementer @072930ee | 0/3 |
| — (fix-burst) | — | — | — | — | test-writer RED @36f0ba9c (P29-006 is_number()→is_string() flip + rationalization comment deleted; test_F_CSD_P31_MED_001_detection_severity_is_string_label_matching_standalone_dtu; test_F_CSD_P31_OBS_001_detection_device_id_stable_across_batch_subsets); BC-2.16.013 v1.30→v1.31 (det_index canonical semantics — batch-position indices forbidden; severity MUST be string label ["Low","Medium","High","Critical"]; changelog row); implementer @072930ee GREEN (SEVERITY_LABELS[det_index % 4]; det_index parsed INSIDE detection_detail() from detection_id trailing NNN via rsplit('-'); handles org_slug hyphens; deterministic fallback 0; det_index param removed; .enumerate() removed from sole call site; TD-VSDD-060 single-site sweep; v91_ast_virtual_field_match E0004 gate added; EXPECTED 90→91 ci.yml+sh+py+CLAUDE.md; Layer-2 keys disambiguated; enum doc 22→23 entries; totals 91; just check FULL WORKSPACE GREEN; non-exhaustive 91/91 two-layer per-symbol ON BRANCH; expected count 5481 pending exact re-verify). BC-INDEX v7.93→v7.94. New FROZEN HEAD 072930ee (LOCAL-ONLY). Streak 0/3. Cascade 31 passes (commits: 36f0ba9c, 072930ee). Develop baseline UNCHANGED @b9cf3f9b. | 0/3 |
| 32 | frozen @072930ee | NO | **YES** | 3 LOW (F-CSD-P32-OBS-001 LOW docstring contradiction E0004 note shape v90/v91 VirtualField; F-CSD-P32-OBS-002 LOW py module comment "90 entries" stale (truth 91); F-CSD-P32-OBS-003 LOW ci.yml comment ">= 90" stale + CLAUDE.md fix-branch authority pointer misidentifies ci.yml as EXPECTED owner). SAP-1 PASS (232 sites; pre_register bare debug! D-765 precedent exempt). SAP-2 PASS. POL-22/24/33 PASS. All adjudicated invariants re-verified intact. Adversary novelty: LOW-MEDIUM. | implementer @a6f86fa3 (docs-only) | 0/3 |
| — (fix-burst) | — | — | — | — | implementer @a6f86fa3 (docs-only): extract_e0004_symbol worked-example annotated for v90 re-export (2-part note → `virtual_fields::VirtualField`) vs v91 direct-path (3-part note → `ast::VirtualField`) distinction; Layer-2 failure canary added (prints full sorted symbol set to stderr on mismatch); py module header "90 entries" → "91 entries"; ci.yml comment `>= 90` → `>= EXPECTED (owned by scripts/check-non-exhaustive.sh)`; CLAUDE.md fix-branch authority pointer corrected to scripts/check-non-exhaustive.sh; TD-VSDD-060 sweep clean; scripts/check-non-exhaustive.sh 91/91 both layers; just check-fast clean (docs-only; no code logic altered). New FROZEN HEAD a6f86fa3 (LOCAL-ONLY, NOT pushed to origin — flag priority push after 3-CLEAN). Streak 0/3. Cascade 32 passes. Develop baseline UNCHANGED @b9cf3f9b. | 0/3 |

Streak reset at passes 3, 4, 12, 14, 16, 18, 19, 20, and 28. Pass 10 is the FIRST CLEAN(PR-merge) of the cascade. Pass 11 is the FIRST CLEAN(strict) of the cascade (streak 1/3); pass 12 LOW F-CSD-P12-001 reset to 0/3. Pass 13 (frozen @421ce222): CLEAN(strict)=YES — streak 1/3 (SECOND CLEAN(strict) of the cascade). Pass 14 (frozen @421ce222): MED F-CSD-P14-001 virtual-field schema parity gap — streak RESET 1/3→0/3. Pass 15 (frozen @87e8ff10): CLEAN(strict)=YES — streak 1/3 (THIRD CLEAN(strict) of the cascade). Pass 16 (frozen @87e8ff10): MED F-CSD-P16-001 virtual-field spec-anchor gap (BC-2.11.005 DEC-022 + BC-2.11.012 missing test-specific invariant) + OBS F-CSD-P16-002 comment fragmentation — streak RESET 1/3→0/3. Pass 17 (frozen @819beeda): CLEAN(strict)=YES CLEAN(PR-merge)=YES — ZERO findings. Streak 1/3 (per D-1666). Pass 18 (frozen @819beeda): NOT CLEAN(strict) CLEAN(PR-merge) — 7 OBS findings F-CSD-P18-001..007. Streak RESET 1/3→0/3. Pass 19 (frozen @962f2ffb): NOT CLEAN(strict) CLEAN(PR-merge) — 4 LOW findings F-CSD-P19-001..004. Streak RESET (stays 0/3). Mode-dimension survey complete: SQL+SqlPipe subquery atoms present and pre_register_empty_tables wired (both virtual-field and registration); Filter/Pipe parsers have no subquery atoms — structurally exempt. Structural-fix class closure: P5 (FuncCall-args recursion gap), P6 (DML source_select defense-in-depth), P7 (WHERE/HAVING/JOIN-ON check_sql_query non-recursive), P8-MED (gate placement below early-return; DUAL placement fix), P8-LOW (DML filter/assignments interiors), P9-HIGH (harness-clone POST verb surface gap), P10 (empirical DataFusion capability + harness first_seen OBS), P12 (SqlPipe pre_register_empty_tables), P14-001 (empty-MemTable virtual-field schema), P16-001 (virtual-field spec-anchor gap — spec layer closed via BC-2.11.005 v1.10 + BC-2.11.012 v1.5), P16-002 (comment block re-flowed), P18-001..007 (nullable parity T33b; volatile pins sweep; stale allow(dead_code); DTU-EXT-001 POST constraint; BC-2.11.012 lifecycle promote; T1 backward-ref; T3 body matcher), P19-001 (volatile pins test doc comments swept), P19-002 (Compare-arm InSubquery gate gap closed), P19-003 (PO split adjudication: _source_type implemented as 4th virtual field; _safety_flags retired; BC-2.11.012 v1.7), P19-004 (CWE-117 five tracing sites sanitize_for_log) — all closed. PENDING-HUMAN: DRIFT-SAP2-DEVICES-TOML-SURFACE-001 (P14-010 SAP-2 §4-class). SCRUTINY ITEM E-QUERY-038 second-emission-source CLOSED by pass-20 ARCHITECT Option A (D-1669): fallback REMOVED @e8f7dc8b; execute_against_session production-reachable ONLY via execute_inner plan-time gate first; runtime FieldNotFound = internal anomaly → QueryExecutionFailed; BC-2.11.016 v1.26 §Design Constraints codifies THREE emission sites. Pass 20 (frozen @7347bb16): CLEAN(strict)=NO CLEAN(PR-merge)=NO — 15 findings: 3 CRIT + 5 HIGH + 4 MED + 1 LOW + 2 OBS. Fix-burst: architect Option A fallback removal @e8f7dc8b; BC-2.11.012 v1.8 + BC-2.16.002 v2.09 + BC-2.11.016 v1.26; 4-commit fix chain @0d198d6d→e8f7dc8b→848ef359→14bffa1c; defect suite 41/41; just check FULL WORKSPACE 5466/5466 GREEN; non-exhaustive 89/89; prism-sensors 180/180. Streak RESET (stays 0/3). PENDING-HUMAN: DRIFT-SAP2-DEVICES-TOML-SURFACE-001 (P14-010 SAP-2 §4-class). NEW QUEUE: TD-S302-005 buffer-serving story (D-1669). Pass 21 (frozen @14bffa1c): CLEAN(strict)=NO CLEAN(PR-merge)=YES — 3 findings: 2 LOW + 1 OBS (F-CSD-P21-OBS-001 LOW four-field test coverage gap; F-CSD-P21-OBS-002 LOW CWE-117 sibling sites in materialization.rs; F-CSD-P21-OBS-003 OBS VirtualField variant count vs BC-2.11.012 v1.8 — TWO-STAGE PO ADJUDICATION → BC-2.11.012 v1.10 + DRIFT-INTERNAL-TABLE-COLUMN-GATE-001 queued). Fix-burst: test-writer @635bc31c (T32/T33/T37 four-field), implementer @b5caea48 (CWE-117 complete sweep 12 sites), implementer @7e528956 (docstring internal-table scope, TD-VSDD-060), BC-2.11.012 v1.9→v1.10 (POL-22 spec-code mismatch corrected), test-writer @e1a00fa3 (EC-11-035 lock → QueryExecutionFailed + negative ColumnNotFound assert). defect suite 42/42; just check FULL WORKSPACE 5467/5467 GREEN (60 skipped); non-exhaustive 89/89. Streak RESET (stays 0/3). NEW FROZEN HEAD e1a00fa3. NEW DRIFT ITEM: DRIFT-INTERNAL-TABLE-COLUMN-GATE-001 (extend plan-time column gate to internal tables; story-queued after CSDEVICES merge). Pass 22 (frozen @e1a00fa3): CLEAN(strict)=NO CLEAN(PR-merge)=NO — 5 findings: 1 MED + 3 LOW + 1 OBS. Fix-burst: implementer @86b65ffb (7-site SQL sanitize_for_log sweep); test-writer @4f084a31 (pin sweep + EC-11-035b storage-backed lock). BC-2.11.005 v1.11 + BC-2.11.012 v1.11 + BC-2.16.002 v2.10. Streak RESET (stays 0/3). NEW FROZEN HEAD 4f084a31. Severity trajectory note: P20 spike (3C/5H/4M) → P21 (2L/1O) → P22 (1M/3L/1O) → P23 (1M) — structural classes closed; remaining findings are spec-prose sync + test-hardening tail. Pass 23 (frozen @4f084a31): CLEAN(strict)=NO CLEAN(PR-merge)=NO — 1 finding: 1 MED (F-CSD-P23-001 POL-29 within-file sibling-sweep gap; §Edge Cases T32/T33 sibling rows missed by P22 fix). Fix-burst: PO exhaustive per-variant sweep across all of .factory/specs/ — 7 artifacts bumped (spec-layer-only; code HEAD UNCHANGED 4f084a31). Streak RESET (stays 0/3). FROZEN HEAD UNCHANGED 4f084a31. NEW LESSON: L38 [process-gap] — POL-29 partial sweeps generate follow-on passes; exhaustive per-variant grep across all .factory/specs/ is cheaper than another adversary round-trip. Pass 24 (frozen @4f084a31): CLEAN(strict)=NO CLEAN(PR-merge)=NO — 3 MED + 2 OBS. Fix-burst: implementer @6a913680 (quick-reference 3 rows + parity gate + docstring sweep); test-writer @0d07be7e (T40 SqlPipe lock + store_step_vars 3 unit tests + T39 re-anchor). just check FULL WORKSPACE 5472/5472 GREEN; prism-mcp 447/447; non-exhaustive 89/89. Streak RESET (stays 0/3). NEW FROZEN HEAD 0d07be7e. Pass 25 (frozen @0d07be7e): CLEAN(strict)=NO CLEAN(PR-merge)=YES — 5 LOW + 1 OBS. Adversary novelty: LOW. SAP-1/SAP-2/POL-24 clean. Fix-burst: test-writer @437dac0e (negative_e043_parity_gate.rs 2 RED + T41 + T42); implementer @99719a7a (ExampleKind::NegativeE043 + SqlPipe stages walk + [[test]] required-features + cite-pin sweep 7 sites; just check 5476/5476 GREEN, 60 skipped; prism-mcp 449/449; prism-query 1548/1548; non-exhaustive 89/89; SAP-1 zero new emissions); PO BC-2.16.013 v1.27→v1.28 Route Coverage Table (9 rows POL-33; F-CSD-P25-006 OBS CLOSED). NEW FROZEN HEAD 99719a7a (LOCAL-ONLY). Streak RESET → 0/3. LOCAL pass 26 NEXT on frozen 99719a7a (streak 0/3). Pass 26 (frozen @99719a7a): CLEAN(strict)=NO CLEAN(PR-merge)=YES — 3 OBS. Adversary novelty: LOW. SAP-1 PASS (rows 179–183 verified). SAP-2 PASS. POL-22 PASS. POL-33 PASS. F-CSD-P26-OBS-001: "3-tier" gate docstring stale (catalog now 4 tiers; ExampleKind::NegativeE043 added at pass-25 not reflected in prose/assertions/stub). F-CSD-P26-OBS-002: Ast::Pipe vs Ast::SqlPipe stage-walk asymmetry — ARCHITECT ADJUDICATION Option B (D-1675): asymmetry STANDS; key correction: pipe_sql_emitter::predicate_to_datafusion_sql has explicit Predicate::InSubquery => Err arm blocking before DataFusion; SqlPipe lacks this emitter block hence stage-walk necessary; T39 lock UNCHANGED; two-step future-gate-extension condition documented. F-CSD-P26-OBS-003: SAP-2 devices DTU-fields-not-in-TOML gap (os_version, containment_status, external_ip, local_ip, agent_version, cid/agent_id) — PRE-EXISTING; CLOSED-BY-EXISTING-DEFERRAL DRIFT-SAP2-DEVICES-TOML-SURFACE-001 (D-1666 decision 4; human-authorized; queued post-CSDEVICES-merge). Fix-burst: test-writer @3202d80f (4-tier rename + has_negative_e043 assertion + exhaustiveness-stub arm + T39 emitter-boundary comment); implementer @9fe2d016 (build_reference_content "4-tier" design note + materialization.rs architect-ratified emitter-boundary rationale; just check FULL WORKSPACE GREEN 5476 @9fe2d016; non-exhaustive 89/89; renames only, no new tests). NEW FROZEN HEAD 9fe2d016 (LOCAL-ONLY). Streak RESET → 0/3. LOCAL pass 27 NEXT on frozen 9fe2d016. Pass 27 (frozen @9fe2d016): CLEAN(strict)=YES CLEAN(PR-merge)=YES — ZERO findings. Adversary novelty: N/A (zero findings). SAP-1 PASS. SAP-2 PASS (devices 6/6; excess-field gap correctly deferred to DRIFT-SAP2-DEVICES-TOML-SURFACE-001). POL-22 PASS. POL-24 PASS (byte-strict). POL-33 PASS. Non-exhaustive gate EXPECTED=89 UNCHANGED. Architect-ratified Pipe/Filter wildcard invariant re-verified empirically. SqlPipe stage-walk structural completeness confirmed. Streak 0/3 → 1/3 on frozen 9fe2d016 (frozen-HEAD rule: HEAD unchanged since pass start). LOCAL pass 28 NEXT on same frozen 9fe2d016.

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
| 22 | frozen @e1a00fa3 | NO | NO | 5 findings: 1 MED + 3 LOW + 1 OBS (F-CSD-P22-001 MED spec-vs-spec co-mutation drift — BC-2.11.005 v1.10 DEC-022 enumerated THREE empty-path virtual fields while BC-2.11.012 v1.10 + code mandate FOUR; F-CSD-P22-002 LOW volatile "engine.rs execute_inner line ~978" pin in EC-11-035 test comment; F-CSD-P22-003 LOW EC-11-035 test only drove storage=None branch — real production path unverified; F-CSD-P22-004 LOW BC-2.11.012 _source_table internal enumeration stale short names; F-CSD-P22-005 OBS BC-2.16.002 5 SQL-event catalog rows falsely claimed control-char-only data while lowered SQL embeds user WHERE literals — CWE-117 sanitize_for_log gap) | PO BC-2.11.005 v1.11 + BC-2.11.012 v1.11; implementer @86b65ffb + test-writer @4f084a31; PO BC-2.16.002 v2.10 | 0/3 |
| — (fix-burst) | — | — | — | — | F-CSD-P22-001 MED: PO BC-2.11.005 v1.10→v1.11 (DEC-022 §Postconditions bullet + §Edge Cases row corrected to FOUR virtual fields incl _source_type; code-verified against append_virtual_fields_to_schema). F-CSD-P22-002 LOW + F-CSD-P22-003 LOW: test-writer @4f084a31 — volatile "engine.rs execute_inner line ~978" pin in EC-11-035 test comment → symbolic anchor (storage-conditional expression; file-wide pin grep clean); NEW test EC-11-035b `test_BC_2_11_012_EC_11_035_source_type_on_internal_table_storage_backed` — InMemoryBackend via pub(crate) field injection + alias_store pattern, prism_alerts registered under alerts_schema(), DataFusion FIELD-not-found → QueryExecutionFailed + negative ColumnNotFound assertion (locks against Option-A fallback re-introduction on real production path). F-CSD-P22-004 LOW: PO BC-2.11.012 v1.10→v1.11 (internal table enumeration corrected to full prism_* names: prism_rules, prism_alerts, prism_threats, prism_enrichments, prism_raw, prism_cache, prism_meta — 7 values; code-verified against inject_internal_virtual_fields/INTERNAL_TABLE_SPECS; NOTE POL-22 win: adversary's claim that prism_rules was missing was partially wrong — prism_rules IS in INTERNAL_TABLE_SPECS; real defect was short names not full prism_* names). F-CSD-P22-005 OBS BOTH-SIDES FIX: implementer @86b65ffb — sanitize_for_log at ALL SEVEN physical emission sites (5 logical events; pipe.sql_lowering + pipe.sql_planning_error each emit from both Ast::Pipe and Ast::SqlPipe arms; complete-set classification table produced); PO BC-2.16.002 v2.09→v2.10 (catalog label v1.60→v1.61; 5 rows' field descriptions + SECURITY sentences corrected: carries lowered SQL incl user WHERE literals, control-char-sanitized via sanitize_for_log, CWE-117). defect suite 43/43; just check FULL WORKSPACE 5468/5468 GREEN (60 skipped); non-exhaustive 89/89. New FROZEN HEAD 4f084a31. Streak 0/3. | 0/3 |
| 23 | frozen @4f084a31 | NO | NO | 1 MED F-CSD-P23-001 (POL-29 within-file sibling-sweep gap: BC-2.11.005 v1.11 §Edge Cases T32/T33 vectors still said "three virtual fields" / omitted `_source_type`; P22-001 fix corrected DEC-022 bullet + one §Edge Cases row but missed two sibling rows T32/T33 in the same section; exhaustive workspace sweep found same stale-enumeration class in 6 additional artifacts) | PO exhaustive POL-29 per-variant sweep across ALL of `.factory/specs/` → 7 artifacts fixed | 0/3 |
| — (fix-burst) | — | — | — | — | PO exhaustive per-variant sweep (FIX/EXEMPT/EXEMPT-historical classification table produced): BC-2.11.005 v1.11→v1.12 (§Edge Cases T32/T33 two sibling rows corrected to "four virtual fields incl `_source_type`"); BC-2.15.009 v1.4→v1.5 (stale `_source_type`/`_safety_flags` enumeration corrected); BC-2.11.022 v1.1→v1.2 (RETIRED `_safety_flags` listed as 4th VirtualField enum member at line 55 → `_source_type` with retirement note); prd.md v1.12→v1.13 (VirtualField enumeration corrected); test-vectors.md v2.9→v2.10 (VirtualField enumeration corrected); query-engine.md v1.3→v1.4 (VirtualField enumeration corrected); operational-pipeline.md v1.2→v1.3 (VirtualField enumeration corrected). Code HEAD UNCHANGED 4f084a31 (frozen-HEAD rule; spec-layer-only fix). defect suite 43/43; workspace 5468/5468 GREEN; non-exhaustive 89/89 ALL UNCHANGED. | 0/3 |

---

### Pass 21

| ID | Severity | Description | Resolution |
|----|----------|-------------|------------|
| F-CSD-P21-OBS-001 | LOW | T32 and T33 (DataFusion-path integration tests) locked only 3 of 4 virtual fields; `_source_type` added by @7347bb16 was not included in the SELECT assertion list or the expected-row array. T37 docstring still read "three-field" after the four-field promotion in BC-2.11.012 v1.8. | test-writer @635bc31c: T32 SELECT + NULL-assert extended to all 4 virtual fields (`_sensor`, `_client`, `_source_table`, `_source_type`); T33 expected array extended to 4 fields; T37 docstring refreshed to four-field set. |
| F-CSD-P21-OBS-002 | LOW | CWE-117 sibling sites in `materialization.rs` unsanitized after the P20 sweep: `check_ci_column_types` assertion detail strings and `source_table`/`source_name` emission sites passed raw user-controlled values to `tracing::*!` without `sanitize_for_log`. | implementer @b5caea48: complete-set sweep across materialization.rs — 12 sites wrapped with `sanitize_for_log`; 9 sites explicitly classified exempt (SQL-string indirect category; validated newtypes `SensorId`/`OrgSlug`/`OrgId` already redact on `Debug`); 1543/1543 GREEN. |
| F-CSD-P21-OBS-003 | OBS | `prism_core::VirtualField` enum had 3 variants (`Sensor`, `Client`, `SourceTable`) while BC-2.11.012 v1.8 claimed a four-field workspace. TWO-STAGE PO ADJUDICATION: Stage 1 — PO investigation: `_source_type` is sensor-table-only; internal tables (`prism_*`) retain 3 virtual fields + `_meta_scan_truncated`. BC-2.11.012 v1.8→v1.9: internal-table exclusion clause + EC-11-035 + invariant split sensor/internal. Implementer @7e528956: VirtualField docstring scoped to internal-table context (TD-VSDD-060 doc sweep: 1 site fixed, 3 exempt). Stage 2 — SPEC-CODE MISMATCH (POL-22): v1.9 EC-11-035 claimed `SELECT _source_type FROM prism_alerts → E-QUERY-038` based on unverified PO claim about `check_query_column_availability`. Test-writer REFUSED to lock: verified actual behavior = `QueryExecutionFailed` (two independent mechanisms: three-mode `starts_with("prism_")` fail-open in `check_query_column_availability`; `Expr::VirtualField` always-valid skip in `extract_field_paths_from_expr`). PO corrected same-burst: BC-2.11.012 v1.9→v1.10 — EC-11-035 + test vector re-locked to `QueryExecutionFailed` consistent with BC-2.11.016 v1.26 §Design Constraints; v1.9 mis-citation noted in changelog; Option B (extend gate to return E-QUERY-038) rejected as requiring architectural design (partial-gate inconsistency risk → DRIFT-INTERNAL-TABLE-COLUMN-GATE-001 queued). Test-writer @e1a00fa3: `test_BC_2_11_012_EC_11_035_source_type_on_internal_table_returns_query_execution_failed` with `is_err()` assertion + explicit negative `ColumnNotFound` assert. |

CLEAN(strict): NO (2 LOW + 1 OBS). CLEAN(PR-merge): YES. Streak stays 0/3. Fix-burst chain: @b5caea48 → @7e528956 → @635bc31c → @e1a00fa3. New FROZEN HEAD for pass 22: `e1a00fa3`. LOCAL pass 22 NEXT on frozen `e1a00fa3` (streak 0/3).

### Pass 22

| ID | Severity | Description | Resolution |
|----|----------|-------------|------------|
| F-CSD-P22-001 | MED | Spec-vs-spec co-mutation drift: BC-2.11.005 v1.10 DEC-022 §Postconditions bullet + §Edge Cases row enumerated THREE empty-path virtual fields (`_sensor`, `_client`, `_source_table`) while BC-2.11.012 v1.10 + code (`append_virtual_fields_to_schema`) mandate FOUR (incl. `_source_type`); any reader trusting BC-2.11.005 DEC-022 alone would produce a schema-parity gap on the empty-table registration path. | PO: BC-2.11.005 v1.10→v1.11 — DEC-022 §Postconditions bullet + §Edge Cases row corrected to enumerate all FOUR virtual fields incl `_source_type`; code-verified against `append_virtual_fields_to_schema`. |
| F-CSD-P22-002 | LOW | Volatile "engine.rs execute_inner line ~978" pin in EC-11-035 test comment (test_BC_2_11_012_EC_11_035_source_type_on_internal_table_returns_query_execution_failed) — line number decays on diffs; violates TD-VSDD-091. | test-writer @4f084a31: volatile pin replaced with symbolic anchor referencing storage-conditional expression; file-wide pin grep confirmed clean. |
| F-CSD-P22-003 | LOW | EC-11-035 test (`test_BC_2_11_012_EC_11_035_source_type_on_internal_table_returns_query_execution_failed`) only exercised the storage=None (no-backend) engine path; the storage-backed engine path (InMemoryBackend or RocksDB) uses a different code branch — the assertion did NOT verify that the REAL production QueryExecutionFailed path (BC-2.11.016 v1.26 §Design Constraints) fires under realistic engine conditions; leaves open the Option-A fallback re-introduction risk on the real path. | test-writer @4f084a31: NEW test EC-11-035b `test_BC_2_11_012_EC_11_035_source_type_on_internal_table_storage_backed` — constructs InMemoryBackend via pub(crate) field injection using alias_store pattern; registers prism_alerts under alerts_schema(); executes `SELECT _source_type FROM prism_alerts`; asserts DataFusion FieldNotFound → QueryExecutionFailed; explicit negative ColumnNotFound assertion (locks against Option-A fallback re-introduction on the REAL production path). |
| F-CSD-P22-004 | LOW | BC-2.11.012 `_source_table` internal table enumeration (§Description / §Postconditions) used stale SHORT names (e.g., `rules`, `alerts`, `threats`) instead of full `prism_*` qualified names as implemented in INTERNAL_TABLE_SPECS; any reader following the spec for table enumeration would produce incorrect table name checks. NOTE POL-22 win: adversary's claim that `prism_rules` was entirely absent from the enumeration was partially incorrect — `prism_rules` IS present in INTERNAL_TABLE_SPECS (7-entry array); real defect was short names vs full prism_* names in the spec text; PO code-verification caught the adversary's overstatement during the fix. | PO: BC-2.11.012 v1.10→v1.11 — internal table enumeration corrected to full prism_* names: `prism_rules`, `prism_alerts`, `prism_threats`, `prism_enrichments`, `prism_raw`, `prism_cache`, `prism_meta` — 7 values; code-verified against `inject_internal_virtual_fields`/INTERNAL_TABLE_SPECS. |
| F-CSD-P22-005 | OBS | BC-2.16.002 catalog rows for the 5 SQL-string events (`pipe.sql_lowering`, `pipe.sql_planning_error`, and 3 related events) falsely claimed the SQL field carried "column names and operator keywords only, not sensor data" — the lowered SQL string actually embeds user WHERE clause literals (string values, timestamps, etc.); `sql.sql_planning_error` inherited the same false claim pattern. CWE-117: all 7 physical emission sites lacked `sanitize_for_log` guard before the fix-burst. | BOTH-SIDES FIX: implementer @86b65ffb — `sanitize_for_log` added at ALL SEVEN physical emission sites (5 logical events; `pipe.sql_lowering` + `pipe.sql_planning_error` each emit from BOTH `Ast::Pipe` and `Ast::SqlPipe` arms; complete-set classification table produced); PO — BC-2.16.002 v2.09→v2.10 (catalog label v1.60→v1.61; 5 rows' field descriptions + SECURITY sentences corrected: SQL field carries lowered SQL incl user WHERE literals, control-char-sanitized via `sanitize_for_log`, CWE-117). |

CLEAN(strict): NO (1 MED + 3 LOW + 1 OBS). CLEAN(PR-merge): NO (MED finding present). Streak stays 0/3. Fix-burst chain: @86b65ffb → @4f084a31. New FROZEN HEAD for pass 23: `4f084a31`. LOCAL pass 23 NEXT on frozen `4f084a31` (streak 0/3).

### Pass 23

| ID | Severity | Description | Resolution |
|----|----------|-------------|------------|
| F-CSD-P23-001 | MED | POL-29 within-file sibling-sweep gap: BC-2.11.005 v1.11 §Edge Cases T32/T33 vectors still said "three virtual fields" / omitted `_source_type`; pass-22 P22-001 fix corrected the DEC-022 §Postconditions bullet + one §Edge Cases row but missed two sibling rows (T32/T33) in the same section of BC-2.11.005; exhaustive workspace-wide sweep found the same stale-enumeration class in 6 additional artifacts. Additional catch: BC-2.11.022 line 55 still listed RETIRED `_safety_flags` as 4th VirtualField enum member — corrected to `_source_type` with retirement note. Root cause: POL-29 partial sweep (fix-the-first-occurrence pattern); exhaustive per-variant grep across all `.factory/specs/` is the required discipline. | PO exhaustive per-variant sweep: BC-2.11.005 v1.11→v1.12; BC-2.15.009 v1.4→v1.5; BC-2.11.022 v1.1→v1.2; prd.md v1.12→v1.13; test-vectors.md v2.9→v2.10; query-engine.md v1.3→v1.4; operational-pipeline.md v1.2→v1.3. FIX/EXEMPT/EXEMPT-historical classification table produced. Code HEAD UNCHANGED 4f084a31. |

CLEAN(strict): NO (1 MED). CLEAN(PR-merge): NO (MED present). Streak stays 0/3. Code HEAD UNCHANGED (frozen-HEAD rule; spec-layer-only fix). FROZEN HEAD UNCHANGED `4f084a31`. LOCAL pass 24 NEXT on frozen `4f084a31` (streak 0/3).

---

## Evidence at Pass 23 Fix HEAD (4f084a31 — UNCHANGED)

Code HEAD `4f084a31` UNCHANGED (frozen-HEAD rule; spec-layer-only fix). Evidence unchanged from pass 22:

- prism-query defect suite: **43/43** GREEN
- Full workspace `just check`: **5468/5468** GREEN (60 skipped)
- Non-exhaustive gate: 89/89

## Spec Layer Modified (pass-23 closure)

| Artifact | Version Bump | Change |
|----------|-------------|--------|
| BC-2.11.005 | v1.11→v1.12 | F-CSD-P23-001: §Edge Cases T32/T33 vectors corrected to enumerate all FOUR virtual fields incl `_source_type` (was "three virtual fields"); two sibling rows in same section missed by pass-22 P22-001 fix (POL-29 partial-sweep root cause) |
| BC-2.15.009 | v1.4→v1.5 | F-CSD-P23-001 exhaustive sweep: stale `_source_type`/`_safety_flags` enumeration corrected |
| BC-2.11.022 | v1.1→v1.2 | F-CSD-P23-001 exhaustive sweep: RETIRED `_safety_flags` listed as 4th VirtualField enum member (line 55) corrected to `_source_type` with retirement note |
| prd.md | v1.12→v1.13 | F-CSD-P23-001 exhaustive sweep: VirtualField enumeration corrected |
| test-vectors.md | v2.9→v2.10 | F-CSD-P23-001 exhaustive sweep: VirtualField enumeration corrected |
| query-engine.md | v1.3→v1.4 | F-CSD-P23-001 exhaustive sweep: VirtualField enumeration corrected |
| operational-pipeline.md | v1.2→v1.3 | F-CSD-P23-001 exhaustive sweep: VirtualField enumeration corrected |
| BC-INDEX | v7.89→v7.90 | BC-2.11.005 row v1.11→v1.12; BC-2.15.009 row v1.4→v1.5; BC-2.11.022 row v1.1→v1.2; changelog entry D-1672 |
| ARCH-INDEX | v2.175→v2.176 | query-engine.md v1.3→v1.4; operational-pipeline.md v1.2→v1.3; changelog entry D-1672 |

---

## Evidence at Pass 22 Fix HEAD (4f084a31)

- prism-query defect suite: **43/43** GREEN
- Full workspace `just check`: **5468/5468** GREEN (60 skipped)
- Non-exhaustive gate: 89/89
- SAP-1 sweep: zero new `event_type =` emissions without BC-2.16.002 catalog row
- SAP-2 sweep: CLEAN (no new TOML column vs DTU field divergence introduced)

## Spec Layer Modified (pass-22 closure)

| Artifact | Version Bump | Change |
|----------|-------------|--------|
| BC-2.11.005 | v1.10→v1.11 | F-CSD-P22-001: DEC-022 §Postconditions bullet + §Edge Cases row corrected to enumerate FOUR virtual fields incl `_source_type` (was THREE); code-verified against `append_virtual_fields_to_schema` |
| BC-2.11.012 | v1.10→v1.11 | F-CSD-P22-004: internal table enumeration corrected to full prism_* names (7 values: prism_rules, prism_alerts, prism_threats, prism_enrichments, prism_raw, prism_cache, prism_meta); code-verified against INTERNAL_TABLE_SPECS; NOTE POL-22: adversary partially wrong on prism_rules existence — PO verification caught overstatement |
| BC-2.16.002 | v2.09→v2.10 | F-CSD-P22-005: catalog label v1.60→v1.61; 5 SQL-event rows' field descriptions + SECURITY sentences corrected — lowered SQL carries user WHERE literals, control-char-sanitized via sanitize_for_log, CWE-117; implementer @86b65ffb confirmed 7-site sanitize_for_log sweep |
| BC-INDEX | v7.88→v7.89 | BC-2.11.005 row v1.10→v1.11; BC-2.11.012 row v1.10→v1.11; BC-2.16.002 row v2.09→v2.10; changelog entry D-1671 |

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

### Pass 24

| ID | Severity | Description | Resolution |
|----|----------|-------------|------------|
| F-CSD-P24-001 | MED | LLM-facing Error Code Quick-Reference in `prism-mcp` `resources.rs` missing E-QUERY-041/E-QUERY-042/E-QUERY-043 rows (agent self-correction surface; AD-017) | implementer @6a913680: 3 rows added sourced from error-taxonomy v2.39; BC-2.11.022 CI-parity gate test extended to lock the rows (`reference_content.rs`) |
| F-CSD-P24-002 | MED | `parser_tests.rs` docstrings said "five canonical" virtual fields (stale post-`_safety_flags` retirement; set is FOUR) | implementer @6a913680: both affected docstrings corrected; workspace sweep found third hit ("five canonical error codes E-SPEC-019..023" in `overlay_loading_tests.rs`) correctly EXEMPT (different value class) |
| F-CSD-P24-003 | MED | `Ast::SqlPipe(spq) => check_sql_query(&spq.head)` arm of E-QUERY-043 gate had no end-to-end regression lock | test-writer @0d07be7e: NEW T40 `test_BC_2_11_003_F_CSD_P24_003_T40_sqlpipe_head_insubquery_projection_fires_e_query_043` (parses SqlPipe head with InSubquery projection, asserts `Ast::SqlPipe` then `ExprInSubqueryProjectionNotSupported`) |
| F-CSD-P24-OBS-002 | OBS | `store_step_vars` last-segment fallback (which all four `variables_produced = []` TOML steps depend on) had no contract lock | test-writer @0d07be7e: 3 unit tests in new `store_step_vars_tests` module in `prism-spec-engine` `pipeline.rs` (fallback key population, `or_insert_with` guard, nested-path last segment) |
| F-CSD-P24-OBS-003 | OBS | T39 docstring cited non-existent "T9 / T18" test names | test-writer @0d07be7e: re-anchored to actual test names (T2 + new T40) |

CLEAN(strict): NO (3 MED). CLEAN(PR-merge): NO. Streak RESET (stays 0/3). New FROZEN HEAD for pass 25: `0d07be7e` (LOCAL-ONLY).

## Fix-Burst (pass-24 closure)

**Commit 6a913680** (implementer): E-QUERY-041/042/043 rows added to LLM quick-reference in `prism-mcp/src/resources.rs`; CI-parity gate test extended in `reference_content.rs` to lock the 3 new rows; parser_tests.rs docstrings corrected five→four (2 sites); workspace sweep confirmed third hit in `overlay_loading_tests.rs` is EXEMPT (different value class — "five canonical error codes E-SPEC-019..023").

**Commit 0d07be7e** (test-writer): NEW T40 `test_BC_2_11_003_F_CSD_P24_003_T40_sqlpipe_head_insubquery_projection_fires_e_query_043` in `prism-query`; NEW `store_step_vars_tests` module (3 unit tests) in `prism-spec-engine/src/pipeline.rs`; T39 docstring re-anchored to T2 + T40 actual test names.

## Evidence at Pass-24 Fix HEAD (0d07be7e)

- Full workspace `just check`: **5472/5472** GREEN
- Non-exhaustive gate: **89/89**
- `prism-mcp`: **447/447** GREEN (extended parity gate incl. 3 new quick-reference rows)

## Spec Layer Modified (pass-24 closure)

No spec-layer version bumps — code-only fix (quick-reference rows in MCP resource, docstring corrections, T40 regression lock, store_step_vars unit tests). No BC or error-taxonomy version bumps.

---

---

### Pass 25

| ID | Severity | Description | Resolution |
|----|----------|-------------|------------|
| F-CSD-P25-001 | LOW | `crates/prism-dtu-crowdstrike/Cargo.toml` missing `[[test]] required-features = ["test-support"]` for `defect_csdevices_post_host_details.rs` — silent skip under `--no-default-features` | implementer @99719a7a: `[[test]] name = "defect_csdevices_post_host_details" required-features = ["test-support"]` added |
| F-CSD-P25-002 | LOW | 3 code comment sites contain stale cite-pin "BC-2.11.005 v1.9 DEC-022" (BC-2.11.005 now v1.12; TD-VSDD-091 semantic-currency violation); POL-29 sibling sweep found 4 additional stale pins | implementer @99719a7a: 3 sites → "BC-2.11.005 DEC-022 (introduced v1.9)"; POL-29 sibling sweep 4 additional sites corrected |
| F-CSD-P25-003 | LOW | LLM quick-reference had no load-bearing NegativeE043 example — parity gate couldn't catch E-QUERY-043 gate regression (positive-row gate only verified row presence, not gate firing) | test-writer @437dac0e: `negative_e043_parity_gate.rs` 2 RED; implementer @99719a7a: `ExampleKind::NegativeE043` + `REFERENCE_EXAMPLES` entry + E-QUERY-043 section in `build_reference_content` |
| F-CSD-P25-004 | LOW | `check_sql_query` `Ast::SqlPipe(spq)` arm walked only `spq.head`, not `spq.stages` — implicit parser-parity invariant (PipeStage::Where has no InSubquery grammar production) undocumented in code; defensive walk missing for future grammar extensions | test-writer @437dac0e: T41 SqlPipe stage-walk constructed-AST RED; implementer @99719a7a: `spq.stages` walk + parser-parity invariant doc-comment |
| F-CSD-P25-005 | LOW | `contains_insubquery` `Expr::TimestampArithmetic` Now-base return claim unlocked by test; ADR-052 §D4 rationale citation absent at arm | test-writer @437dac0e: T42 GREEN lock; implementer @99719a7a: ADR-052 §D4 citation in doc-comment |
| F-CSD-P25-006 | OBS | POL-33 (`route_coverage_table_required_for_stagemask_changes`) compliance: BC-2.16.013 lacked a formal Route Coverage Table for CrowdStrike DTU route changes that affect `containment_status` (StageMask-relevant) | PO BC-2.16.013 v1.27→v1.28: `## Route Coverage Table (POL-33)` section added (9 rows × 3 registration sites; all GUARDED; Claroty/Cyberint/Armis EXEMPT) |

CLEAN(strict): NO (5 LOW + 1 OBS). CLEAN(PR-merge): YES — zero CRIT/HIGH/MED. Adversary novelty: LOW. SAP-1 PASS. SAP-2 PASS. POL-24 PASS (byte-clean). Streak RESET → 0/3 (per DRIFT-ORCH-PRLEVEL-PUSH-001 frozen-HEAD rule — fix-burst commits 437dac0e + 99719a7a pushed to branch). New FROZEN HEAD for pass 26: `99719a7a` (LOCAL-ONLY).

## Fix-Burst (pass-25 closure)

**Commit 437dac0e** (test-writer): NEW `crates/prism-mcp/tests/negative_e043_parity_gate.rs` — 2 RED gate tests (`test_negative_e043_example_present_in_reference_content` locking NegativeE043 variant presence; behavioral gate locking E-QUERY-043 firing path); T41 `test_BC_2_11_003_F_CSD_P25_004_T41_sqlpipe_stage_walk_insubquery_fires` (SqlPipe stage-walk constructed-AST RED — InSubquery in a PipeStage::Where); T42 `test_contains_insubquery_timestamp_arithmetic_now_base_returns_false` (TimestampArithmetic GREEN lock; cites ADR-052 §D4).

**Commit 99719a7a** (implementer): `ExampleKind::NegativeE043` enum variant added; `REFERENCE_EXAMPLES` entry using generic `sensor_table` per BC-2.10.014 AC-008; E-QUERY-043 negative-examples section in `build_reference_content`; `Ast::SqlPipe(spq)` arm now walks `spq.stages` `PipeStage::Where` via `check_predicate` + parser-parity invariant doc-comment; `[[test]] required-features = ["test-support"]` added for `defect_csdevices_post_host_details`; cite-pin sweep 3 sites (v1.9 DEC-022 → "DEC-022 (introduced v1.9)"); POL-29 sibling sweep 4 additional stale pins (prism-bin/Cargo.toml, adv_p02_e2e_pushdown_pipeline_test.rs, bc_2_11_007_pushdown_test.rs ×2); ADR-052 §D4 citation at `TimestampArithmetic` arm.

**PO edits (committed as part of D-1674 burst):** BC-2.16.013 v1.27→v1.28 — `## Route Coverage Table (POL-33)` section added (9 rows covering `containment_status` across `prism-dtu-crowdstrike::build_router` standalone, `prism-dtu-harness::build_crowdstrike_router` in-process, `prism-dtu-harness::build_crowdstrike_network_router` network-mode; all 9 rows GUARDED via shared `host_details_inner`; write routes via `action_name` guard; Claroty/Cyberint/Armis EXEMPT — no scenario-state-dependent fields in spec-driven parity path); cross-reference to INV-HARNESS-ROUTE-PARITY in section intro; changelog row prepended; modified: 2026-07-11. SESSION-HANDOFF.md 2 cite-pins v1.27→v1.28.

## Evidence at Pass-25 Fix HEAD (99719a7a)

- Full workspace `just check`: **5476/5476** GREEN (60 skipped)
- Non-exhaustive gate: **89/89** UNCHANGED (ExampleKind::NegativeE043 is a new variant on an existing `#[non_exhaustive]` enum — gate already counts this crate's non_exhaustive types)
- `prism-mcp`: **449/449** GREEN (+2 new test file negative_e043_parity_gate.rs)
- `prism-query`: **1548/1548** GREEN (T41 + T42 added)
- SAP-1: PASS — zero new `event_type =` emissions

## Spec Layer Modified (pass-25 closure)

| Artifact | Version Bump | Change |
|----------|-------------|--------|
| BC-2.16.013 | v1.27→v1.28 | F-CSD-P25-006: `## Route Coverage Table (POL-33)` section added (9 rows × 3 CrowdStrike registration sites; all GUARDED; Claroty/Cyberint/Armis EXEMPT); modified: 2026-07-11 |

No other BC version bumps. No error-taxonomy version bumps. Code changes: ExampleKind::NegativeE043 + REFERENCE_EXAMPLES + build_reference_content (prism-mcp); SqlPipe spq.stages walk + doc-comment (prism-query); [[test]] required-features (prism-dtu-crowdstrike Cargo.toml); cite-pin sweep 7 sites (code comments).

---

### Pass 29

| ID | Severity | Description | Resolution |
|----|----------|-------------|------------|
| F-CSD-P29-001 | MED | ci.yml FAIL message omits TemporalLiteralPosition; claims 90 but lists 89 | Absorbed by OBS-001 hardening: 106-line inline ci.yml step replaced by `bash scripts/check-non-exhaustive.sh`; new per-symbol Python script @3dd1fd96 |
| F-CSD-P29-002 | MED | `resources.rs` 4 stale "3-tier" doc sites + NegativeE043 absent from contract enumeration | @3dd1fd96: 4 sites swept to 4-tier + NegativeE043 |
| F-CSD-P29-003 | MED | BC-2.11.022 v1.2 declared 3-variant ExampleKind; code shipped 4-variant (spec-code drift from F-CSD-P25-003) | PO BC-2.11.022 v1.2→v1.3: 4-variant; Tier 3 NegativeE043 gate; EC-11-022-007; test vector |
| F-CSD-P29-004 | MED | ci.yml history comment "ADR-045 3-tier gate" stale | Absorbed by OBS-001 hardening @3dd1fd96 |
| F-CSD-P29-005 | MED | ci.yml count comment "47+17=64" stale (truth 68+22=90) | Absorbed by OBS-001 hardening @3dd1fd96 |
| F-CSD-P29-006 | MED | harness `detection_detail()` 4-field thin shape vs 12 TOML detections columns; `device_id` nested not top-level; key fields absent — SAP-2 violation (architect IN-SCOPE-FIX) | test-writer @74c578c3 RED + implementer @7a6f6caa GREEN (12 fields; ioc_value null); BC-2.16.013 v1.29 spec-note clause |
| F-CSD-P29-OBS-001 | OBS [process-gap] | CI non-exhaustive gate count-only; net-zero regression undetectable | @3dd1fd96: `scripts/check-non-exhaustive-per-symbol.py` explicit 90-symbol list with `len==90` guard; per-symbol verification across all 90 symbols |
| F-CSD-P29-OBS-002 | OBS | `fetch_incidents` POST shape zero DTU coverage until DTU-EXT-001 | No action; noted for DTU-EXT-001 owner (must be POST `{"ids":[...]}`) |

CLEAN(strict): NO (5 MED + 1 OBS + 1 PROCESS-GAP). CLEAN(PR-merge): NO — 5 MED present. Adversary novelty: MEDIUM. SAP-1 PASS. SAP-2 (devices) PASS. SAP-2 (detections) FAIL → F-CSD-P29-006. Streak 0/3 (unchanged). New FROZEN HEAD for pass 30: `7a6f6caa` (LOCAL-ONLY).

---

### Pass 30

| ID | Severity | Description | Resolution |
|----|----------|-------------|------------|
| F-CSD-P30-OBS-001 | LOW | `enum_violations.rs` module doc-comment enumeration omitted v79; claimed 21 (truth 22) | @3a9ec741: rebuilt enumeration from actual fns; 22 entries; 22+68=90 cross-checked |
| F-CSD-P30-OBS-002 | LOW | `struct_violations.rs` doc-comment enumeration missing v73-v76 + v88; claimed 60 (truth 68) | @3a9ec741: rebuilt enumeration from actual fns; 68 entries; 68+22=90 cross-checked |
| F-CSD-P30-OBS-003 | OBS | harness `detection_detail()` `device_id="placeholder"` not in host pool → harness-mode detections⋈devices JOIN silently 0 rows; same failure class as original defect (ARCHITECT Option A D-1679 2026-07-11) | BC-2.16.013 v1.29→v1.30 (host-pool constraint + JOIN non-empty); test-writer @c26a74ef RED + implementer @ed2988cc GREEN (generate_host_ids modulo) |
| F-CSD-P30-OBS-004 | OBS | per-symbol Python script duplicate numeric comment labels at v70 and v86 (struct vs enum file entries) | @3a9ec741: added -struct/-enum suffixes to all duplicate labels; all 90 labels now unique |

CLEAN(strict): NO (2 LOW + 2 OBS). CLEAN(PR-merge): YES — zero CRIT/HIGH/MED. Adversary novelty: LOW. SAP-1 PASS. SAP-2 PASS. POL-22 PASS. POL-33 PASS. Per-symbol gate design VERIFIED SOUND. Streak 0/3 (unchanged). New FROZEN HEAD for pass 31: `ed2988cc` (LOCAL-ONLY).

---

### Pass 31

| ID | Severity | Description | Resolution |
|----|----------|-------------|------------|
| F-CSD-P31-MED-001 | MED | SAP-2 type parity: harness `detection_detail()` emits `severity` as INTEGER `50`; TOML declares `column_type = "string"`; standalone DTU emits string labels; P29-006 test rationalization comment had deferred the fix (surface-and-defer anti-pattern) | test-writer @36f0ba9c RED (P29-006 `is_number()`→`is_string()` flip + rationalization comment deleted; `test_F_CSD_P31_MED_001_detection_severity_is_string_label_matching_standalone_dtu`); implementer @072930ee GREEN (`SEVERITY_LABELS[det_index % 4]` where LABELS = ["Low","Medium","High","Critical"]); BC-2.16.013 v1.31 severity string-type clause |
| F-CSD-P31-OBS-001 | LOW | `det_index` was batch enumeration position (`.enumerate()` at call site) — same `detection_id` mapped to DIFFERENT `device_id` across request subsets (non-stable mapping; violates BC-2.16.013 v1.30 stable-mapping intent) | test-writer @36f0ba9c RED (`test_F_CSD_P31_OBS_001_detection_device_id_stable_across_batch_subsets` — full-set vs singleton same-mapping lock); implementer @072930ee GREEN (`det_index` parsed INSIDE `detection_detail()` from `detection_id` trailing NNN via `rsplit('-')`; deterministic fallback 0; `.enumerate()` removed from call site; TD-VSDD-060 sweep confirmed single site); BC-2.16.013 v1.31 `det_index` disambiguation clause |
| F-CSD-P31-OBS-002 | OBS | `prism_query::ast::VirtualField` (4-variant, `#[non_exhaustive]`) had no compile-fail gate coverage; Layer-2 per-symbol dedup keyed on last-segment `VirtualField` would collapse both VirtualField types into one key (silent regression window) | implementer @072930ee: `v91_ast_virtual_field_match` E0004 gate added; EXPECTED 90→91 in ci.yml + check-non-exhaustive.sh + worktree CLAUDE.md + py EXPECTED_COUNT/EXPECTED_SYMBOLS; Layer-2 keys disambiguated to 2-segment forms (`virtual_fields::VirtualField` / `ast::VirtualField`); enum_violations.rs doc enumerations updated (22→23 entries; totals 91) |

CLEAN(strict): NO (1 MED + 1 LOW + 1 OBS). CLEAN(PR-merge): NO — 1 MED present. Adversary novelty: LOW-MEDIUM. SAP-1 PASS. SAP-2 devices PASS. SAP-2 detections FAIL → F-CSD-P31-MED-001. POL-22 PASS. POL-24 PASS. POL-33 PASS. Streak 0/3 (unchanged). New FROZEN HEAD for pass 32: `072930ee` (LOCAL-ONLY).

**Fix-burst commits:** test-writer @36f0ba9c (P29-006 severity flip + 2 new RED tests); BC-2.16.013 v1.30→v1.31 (product-owner); implementer @072930ee (severity string fix + det_index stable parse + gate 91 + Layer-2 key disambiguation). Fix-branch @072930ee: just check FULL WORKSPACE GREEN; non-exhaustive 91/91 two-layer per-symbol; expected workspace count 5481 (5479 + 2 new tests — pending exact re-verify at next gate). SAP-1 zero new emissions.

---

### Pass 32

| ID | Severity | Description | Resolution |
|----|----------|-------------|------------|
| F-CSD-P32-OBS-001 | LOW | `extract_e0004_symbol` worked-example in `scripts/check-non-exhaustive-per-symbol.py` shows only v90 re-export E0004 note shape; v91 direct-path shape not annotated; enum_violations.rs v91 entry description also reflects v90 re-export rather than v91 direct-path. Gate passing correctly — prose-only contradiction. | implementer @a6f86fa3: worked-example annotated for v90 re-export (2-part note → 2-seg key `virtual_fields::VirtualField`) vs v91 direct-path (3-part note → 2-seg key `ast::VirtualField`); Layer-2 failure canary added (prints full sorted symbol set to stderr on mismatch); TD-VSDD-060 sweep clean |
| F-CSD-P32-OBS-002 | LOW | `scripts/check-non-exhaustive-per-symbol.py` module header comment claims "90 entries"; pass-31 added v91 entry → count is 91 | implementer @a6f86fa3: module header "90 entries" → "91 entries"; aligned with `EXPECTED_COUNT = 91` |
| F-CSD-P32-OBS-003 | LOW | ci.yml non-exhaustive step comment cites `>= 90` (stale; now 91); CLAUDE.md fix-branch convention sentence attributes EXPECTED authority to `ci.yml EXPECTED=NN` — incorrect since pass-29 moved EXPECTED ownership to `scripts/check-non-exhaustive.sh` (ci.yml now only calls the script) | implementer @a6f86fa3: ci.yml comment → `>= EXPECTED (owned by scripts/check-non-exhaustive.sh)`; CLAUDE.md fix-branch sentence authority corrected to `scripts/check-non-exhaustive.sh`; TD-VSDD-060 sweep clean |

CLEAN(strict): NO (3 LOW). CLEAN(PR-merge): YES — zero CRIT/HIGH/MED. Adversary novelty: LOW-MEDIUM. SAP-1 PASS (232 sites; new pre_register sites are bare debug! without event_type — D-765 precedent). SAP-2 PASS. POL-22 PASS. POL-24 PASS. POL-33 PASS. EXPECTED=91 verified consistent at all load-bearing sites. Streak 0/3 (unchanged). New FROZEN HEAD for pass 33: `a6f86fa3` (LOCAL-ONLY, NOT pushed to origin).

**Fix-burst commit: @a6f86fa3** (implementer, docs-only): worked-example v90/v91 distinction; Layer-2 failure canary; module header count; ci.yml comment; CLAUDE.md authority pointer. just check-fast clean. New FROZEN HEAD a6f86fa3. Streak 0/3. Cascade 32 passes. Develop baseline UNCHANGED @b9cf3f9b.

---

## Merge Record

_Pending: LOCAL cascade not yet converged. PR not yet created._

**Status: IN PROGRESS — LOCAL pass 33 NEXT on frozen `a6f86fa3` (streak 0/3). Pass-31 on frozen `ed2988cc`: CLEAN(strict)=NO CLEAN(PR-merge)=NO — 1 MED + 1 LOW + 1 OBS. Novelty LOW-MEDIUM. F-CSD-P31-MED-001: severity INTEGER vs string (SAP-2 type parity; P29-006 rationalization removed). F-CSD-P31-OBS-001: det_index batch-position non-stable mapping across request subsets. F-CSD-P31-OBS-002: ast::VirtualField no compile-fail gate + Layer-2 key collision. Fix-burst: test-writer @36f0ba9c RED + implementer @072930ee GREEN (SEVERITY_LABELS[det_index%4]; det_index stable parse from detection_id; gate 90â91; Layer-2 keys disambiguated); BC-2.16.013 v1.30âv1.31. just check FULL WORKSPACE GREEN @072930ee; non-exhaustive 91/91 two-layer; expected count 5481 (pending exact re-verify). New FROZEN HEAD 072930ee. Pass-30 on frozen `7a6f6caa`: 2 LOW + 2 OBS CLEAN(PR-merge). Fix @3a9ec741 + @c26a74ef + @ed2988cc; BC-2.16.013 v1.30. Pass-29 on frozen `25b80a81`: 5 MED + 1 OBS + 1 PG. Pass-28: 1 OBS VirtualField gate 89â90. Pass-27: CLEAN(strict) ZERO streak 0/3â1/3. PENDING-HUMAN: DRIFT-SAP2-DEVICES-TOML-SURFACE-001; DRIFT-INTERNAL-TABLE-COLUMN-GATE-001 (story-queued after merge). Walker-gap P5-P8 CLOSED; harness-parity P9 CLOSED; DataFusion P10 CLOSED; SqlPipe-mode P12 CLOSED; virtual-field-schema P14-001 CLOSED; spec-anchor P16-001 CLOSED; nullable+volatile+lifecycle P18 CLOSED; volatile-pins+CWE-117 P19 CLOSED; E-QUERY-038+FieldNotFound P20 CLOSED; four-field+EC-11-035 P21 CLOSED; spec-code-drift+internal-names+CWE-117-SQL P22 CLOSED; quick-ref+E043-lock P24 CLOSED; NegativeE043+POL-33 P25 CLOSED; 3-tier-docstring+Pipe-asymmetry P26 CLOSED; VirtualField-non-exhaustive P28 CLOSED; ci.yml-per-symbol+detection_detail-shape P29 CLOSED; doc-enumerations+device_id-host-pool P30 CLOSED; severity-string-type+det_index-stable+ast::VirtualField-gate+Layer-2-key P31 CLOSED. Severity trajectory: 0[P27 streak-1] â 1OBS[P28] â 5MED+1OBS+1PG[P29] â 2LOW+2OBS CLEAN(PR-merge)[P30] â 1MED+1LOW+1OBS[P31] (structural defect classes closed; remaining: type-contract + stable-index + gate-completeness in harness generators).**
