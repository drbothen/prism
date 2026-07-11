---
document_type: adversarial-review
scope: LOCAL
fix_defect: DEFECT-CSDEVICES-EMPTY-PIPELINE-001
passes: [25]
feature_head_at_review: 0d07be7e
date: 2026-07-11
clean_strict: false
clean_pr_merge: true
finding_counts:
  total: 6
  crit: 0
  high: 0
  med: 0
  low: 5
  obs: 1
  process_gap: 0
code_behavior_defects: 0
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: adversary
---

# LOCAL Adversary Pass 25 — DEFECT-CSDEVICES-EMPTY-PIPELINE-001

---

## Pass 25 (frozen 0d07be7e; fresh-context adversary; LOCAL cascade; policy rubric + SAP-1 + SAP-2 + POL-22; streak candidate 1/3 — NOT ADVANCING — stays 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=YES

**Findings:** 6 total (0 CRIT / 0 HIGH / 0 MED / 5 LOW / 1 OBS / 0 PROCESS-GAP)

**Adversary novelty assessment:** LOW — no new structural classes; findings are spec-prose sync, test-coverage lock, and POL-33 compliance gap.

**SAP-1:** PASS — No new `event_type` values introduced without BC-2.16.002 catalog row; catalog at v2.10 complete and unchanged.

**SAP-2:** PASS — devices table TOML↔DTU parity holds; no new TOML columns vs DTU field divergence introduced.

**POL-24:** PASS — byte-clean; E-QUERY-043 hint string matches BC-2.11.003 v1.13.

**STREAK:** 0/3 — CLEAN(strict)=NO on frozen 0d07be7e (5 LOW + 1 OBS findings). Fix-burst dispatched: test-writer @437dac0e + implementer @99719a7a + PO BC-2.16.013 v1.28. New frozen HEAD for pass 26: **99719a7a** (streak RESET → 0/3 per DRIFT-ORCH-PRLEVEL-PUSH-001 — any commit to branch resets streak).

**Code HEAD at review:** 0d07be7e (frozen; just check FULL WORKSPACE 5472/5472 GREEN; non-exhaustive 89/89; LOCAL-ONLY; develop baseline UNCHANGED @b9cf3f9b)

**CLEAN(strict):** NO — 5 LOW + 1 OBS findings

**CLEAN(PR-merge):** YES — zero CRIT / HIGH / MED findings

---

## Findings

### F-CSD-P25-001 — `defect_csdevices_post_host_details.rs` missing `[[test]] required-features` entry (LOW)

**Finding:** `crates/prism-dtu-crowdstrike/Cargo.toml` had no `[[test]]` entry for `defect_csdevices_post_host_details.rs`. This test file uses the harness DTU server requiring the `test-support` feature; without a `[[test]]` entry declaring `required-features = ["test-support"]`, the test silently skips under `cargo nextest run --no-default-features` and could be omitted from CI under certain feature-flag configurations. The gap was introduced when the test file was created during the pass-9 fix-burst (@544acd70) but the Cargo manifest was not updated to register the test's feature dependency.

**Status:** CLOSED

**Fix:** implementer @99719a7a — added `[[test]] name = "defect_csdevices_post_host_details" required-features = ["test-support"]` entry to `crates/prism-dtu-crowdstrike/Cargo.toml`.

---

### F-CSD-P25-002 — 3 stale "BC-2.11.005 v1.9 DEC-022" cite-pins (LOW)

**Finding:** Three code comment sites contained the cite-pin "BC-2.11.005 v1.9 DEC-022" (TD-VSDD-091 semantic-currency violation). BC-2.11.005 is now at v1.12; the version-pinned cite `v1.9` is stale by 3 versions and could mislead reviewers about which version of DEC-022 governs the behavior. Additionally, a POL-29 sibling sweep revealed 4 more stale cite-pins in adjacent files that should be corrected in the same burst.

**Status:** CLOSED

**Fix:** implementer @99719a7a — swept 3 target sites to "BC-2.11.005 DEC-022 (introduced v1.9)" — retaining the historical introduction note while removing the version-currency claim that decays with every BC bump. POL-29 sibling sweep across 4 additional stale pins (prism-bin/Cargo.toml, adv_p02_e2e_pushdown_pipeline_test.rs, bc_2_11_007_pushdown_test.rs ×2) corrected in the same commit.

---

### F-CSD-P25-003 — LLM quick-reference NegativeE043 example absent — parity gate regression risk (LOW)

**Finding:** The LLM-facing Error Code Quick-Reference in `prism-mcp/src/resources.rs` received E-QUERY-043 rows at pass-24 (F-CSD-P24-001) but contained no load-bearing *negative* example showing a query that fires E-QUERY-043. The CI parity gate extended at pass-24 verified row *presence* but could not detect a future regression restoring DataFusion silent execution (removing the E-QUERY-043 gate behavior), because the positive-example gate only checks for the row existing in the string output — not that the gate actually fires. A `NegativeE043` example in `REFERENCE_EXAMPLES` queried through `build_reference_content` closes this coverage gap by requiring the gate to fire when the example is constructed.

**Status:** CLOSED

**Fix:** test-writer @437dac0e — `crates/prism-mcp/tests/negative_e043_parity_gate.rs` with 2 RED gate tests (`test_negative_e043_example_present_in_reference_content` verifying the new example exists; behavioral gate locking the E-QUERY-043 firing path). implementer @99719a7a — `ExampleKind::NegativeE043` variant added; `REFERENCE_EXAMPLES` entry using generic `sensor_table` per BC-2.10.014 AC-008; E-QUERY-043 negative-examples section in `build_reference_content`; parity gate extended to lock the new variant; prism-mcp 449/449 GREEN.

---

### F-CSD-P25-004 — `check_sql_query` SqlPipe arm walks only `spq.head`, not `spq.stages` — invariant undocumented (LOW)

**Finding:** The `Ast::SqlPipe(spq) => check_sql_query(&spq.head)` arm in the E-QUERY-043 gate's `check_sql_query` dispatcher walked only `spq.head`, not `spq.stages`. The implicit invariant that `PipeStage::Where` predicates in `spq.stages` cannot contain `Expr::InSubquery` / `Predicate::InSubquery` atoms (because `pipe_parser.rs` has no such grammar production) was established in pass-12 but was not documented in any code comment at the dispatch site. A future grammar extension adding InSubquery support to pipe-stage predicates could silently bypass the E-QUERY-043 gate if this invariant is forgotten. The gap was surfaced by the pass-25 adversary walk of the structural InSubquery gate family.

**Status:** CLOSED

**Fix:** implementer @99719a7a — the `Ast::SqlPipe(spq)` arm now additionally walks `spq.stages` via `PipeStage::Where` → `check_predicate` alongside the existing `spq.head` walk; parser-parity invariant documented in doc-comment ("pipe_parser.rs has no InSubquery grammar production — structurally exempt for current grammar; this walk is defensive for future grammar extensions"); T41 constructed-AST RED (SqlPipe stage-walk with WHERE IN-subquery in a stage) → GREEN.

---

### F-CSD-P25-005 — `contains_insubquery` `TimestampArithmetic` Now-base return claim unlocked by test (LOW)

**Finding:** The `contains_insubquery` helper's `Expr::TimestampArithmetic` arm returned `false` (correct — `TimestampArithmetic` structurally cannot embed a subquery at this grammar level). However, no test locked this claim; a refactor collapsing the arm into `_ => false` could silently change behavior if a future `TimestampArithmetic` variant gains a nested expression field. Furthermore, the `ADR-052 §D4` rationale governing the temporal dispatch arms was not cited at this return site, breaking the traceability chain that justifies the arm's existence in the gate.

**Status:** CLOSED

**Fix:** test-writer @437dac0e — T42 `test_contains_insubquery_timestamp_arithmetic_now_base_returns_false` (constructed-AST GREEN lock for the `Expr::TimestampArithmetic { base: NowLiteral, ... }` variant returning `false`; cites ADR-052 §D4). implementer @99719a7a — ADR-052 §D4 citation added in doc-comment at `TimestampArithmetic` arm.

---

### F-CSD-P25-006 — POL-33 Route Coverage Table absent from BC-2.16.013 (OBS)

**Finding:** `policies.yaml` POL-33 (`route_coverage_table_required_for_stagemask_changes`) mandates a Route Coverage Table in the relevant BC for any change touching StageMask-relevant DTU routes. DEFECT-CSDEVICES-EMPTY-PIPELINE-001 modified CrowdStrike DTU route registration (GET→POST + both harness builder functions), which is StageMask-relevant (`containment_status` field). BC-2.16.013 v1.27 added the INV-HARNESS-ROUTE-PARITY block (pass-9 F-CSD-P9-001 closure) but did not include a formal Route Coverage Table per POL-33. The gap left the coverage of 3 registration sites × 3 routes (9 rows total) undocumented in tabular form as required by the policy.

**Status:** CLOSED

**Fix:** product-owner — BC-2.16.013 v1.27→v1.28: added `## Route Coverage Table (POL-33)` section with 9 rows covering `containment_status` across `prism-dtu-crowdstrike::build_router` (standalone), `prism-dtu-harness::build_crowdstrike_router` (in-process), and `prism-dtu-harness::build_crowdstrike_network_router` (network-mode); all 9 rows GUARDED via shared `host_details_inner` (session-registry filter + containment-store merge); write routes via `action_name` guard; Claroty/Cyberint/Armis EXEMPT (no scenario-state-dependent fields in spec-driven parity path); cross-reference to INV-HARNESS-ROUTE-PARITY in section intro. SESSION-HANDOFF.md spec-anchor cite-pin v1.27→v1.28 propagated at 2 sites.

---

## Fix-Burst Summary

| Agent | Commit | Change |
|-------|--------|--------|
| test-writer | @437dac0e | `crates/prism-mcp/tests/negative_e043_parity_gate.rs`: 2 RED gate tests (F-CSD-P25-003 NegativeE043 parity); T41 SqlPipe stage-walk constructed-AST RED (F-CSD-P25-004); T42 `TimestampArithmetic` Now-base GREEN lock (F-CSD-P25-005; cites ADR-052 §D4) |
| implementer | @99719a7a | `ExampleKind::NegativeE043` variant + `REFERENCE_EXAMPLES` entry + E-QUERY-043 section in `build_reference_content` (F-CSD-P25-003); SqlPipe arm walks `spq.stages` PipeStage::Where via `check_predicate` + parser-parity invariant doc-comment (F-CSD-P25-004); ADR-052 §D4 citation at `TimestampArithmetic` arm (F-CSD-P25-005); `[[test]] required-features` entry in Cargo.toml (F-CSD-P25-001); cite-pin sweep: 3 sites "BC-2.11.005 v1.9 DEC-022" → "BC-2.11.005 DEC-022 (introduced v1.9)" + POL-29 sibling sweep 4 additional stale pins (prism-bin/Cargo.toml, adv_p02_e2e_pushdown_pipeline_test.rs, bc_2_11_007_pushdown_test.rs ×2) (F-CSD-P25-002); just check FULL WORKSPACE 5476/5476 GREEN (60 skipped); prism-mcp 449/449; prism-query 1548/1548; non-exhaustive 89/89; SAP-1 zero new `event_type =` emissions |
| product-owner | (uncommitted → D-1674 burst) | BC-2.16.013 v1.27→v1.28: `## Route Coverage Table (POL-33)` section added (9 rows × 3 CrowdStrike registration sites; all GUARDED; Claroty/Cyberint/Armis EXEMPT); SESSION-HANDOFF.md 2 cite-pins v1.27→v1.28 (F-CSD-P25-006 closure) |

**just check FULL WORKSPACE:** 5476/5476 GREEN (60 skipped)
**prism-mcp:** 449/449 GREEN (ExampleKind::NegativeE043 variant added — variant of existing enum, not a new type; non-exhaustive EXPECTED unchanged at 89)
**prism-query:** 1548/1548 GREEN
**Non-exhaustive gate:** 89/89 UNCHANGED
**SAP-1:** PASS — zero new `event_type =` emissions introduced
**SAP-2:** PASS — devices TOML↔DTU parity holds; no new divergence

---

## New FROZEN HEAD for Pass 26

**99719a7a** (LOCAL-ONLY; fix/csdevices-empty-pipeline; streak RESET → 0/3 per DRIFT-ORCH-PRLEVEL-PUSH-001)

Cascade now: 25 passes. Pass 26 NEXT on frozen 99719a7a.
