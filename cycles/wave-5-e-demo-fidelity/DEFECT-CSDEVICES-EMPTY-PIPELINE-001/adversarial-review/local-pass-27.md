---
document_type: adversarial-review
scope: LOCAL
fix_defect: DEFECT-CSDEVICES-EMPTY-PIPELINE-001
passes: [27]
feature_head_at_review: 9fe2d016
date: 2026-07-11
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
code_behavior_defects: 0
streak_after: 1/3
convergence: IN_PROGRESS
authored_by: adversary
---

# LOCAL Adversary Pass 27 — DEFECT-CSDEVICES-EMPTY-PIPELINE-001

---

## Pass 27 (frozen 9fe2d016; fresh-context adversary; LOCAL cascade; policy rubric + SAP-1 + SAP-2 + POL-22 + POL-24 byte-strict + POL-33 table verification; streak candidate 1/3 — ADVANCING — streak 0/3 → 1/3)

**Pass result:** CLEAN(strict)=YES, CLEAN(PR-merge)=YES

**Findings:** 0 total (0 CRIT / 0 HIGH / 0 MED / 0 LOW / 0 OBS / 0 PROCESS-GAP)

**Adversary novelty assessment:** N/A — zero findings.

**SAP-1:** PASS — All `event_type =` emission sites in changed files verified against BC-2.16.002 Canonical Structured Event Catalog (v2.10); no new emissions introduced without a catalog row; catalog complete.

**SAP-2:** PASS — devices TOML spec: all 6 TOML-declared columns (hostname, device_id, first_seen, last_seen, status, platform_name) present in DTU generator and fixture. Excess-field gap (os_version, containment_status, external_ip, local_ip, agent_version, cid/agent_id) correctly deferred to DRIFT-SAP2-DEVICES-TOML-SURFACE-001 (human-directed; D-1666 decision 4). No new divergence introduced.

**POL-22:** PASS — All cited entities resolve; no dangling cross-references.

**POL-24 (byte-strict):** PASS — E-QUERY-041/042/043 hint strings verified byte-exact against error-taxonomy v2.39 and production code; no drift.

**POL-33:** PASS — BC-2.16.013 v1.28 Route Coverage Table (9 rows) verified against all relevant routes in code.

**Non-exhaustive gate:** PASS — ExampleKind 4-arm match in `non-exhaustive-violation` crate verified (PositiveE043 / NegativeE043 / generic / wildcard); EXPECTED=89 preserved; no enum arms added or removed by this branch.

**Architect-ratified invariant (Ast::Pipe / Ast::SqlPipe):** RE-VERIFIED EMPIRICALLY — `pipe_sql_emitter::predicate_to_datafusion_sql` Err arm (`Predicate::InSubquery => Err(...)`) confirmed present; blocks `Ast::Pipe`-mode InSubquery before DataFusion; `Ast::SqlPipe` stage-walk correct by design (D-1675 Option B); T39 lock consistent.

**SqlPipe stage-walk structural completeness:** CONFIRMED — `spq.stages` walk covers `PipeStage::Where → check_predicate`; other stage variants (Select, Extend, As, Aggregate) carry `FieldPath` or projection-only forms, not `Expr`-bearing predicates; no InSubquery surface present in unwalked stage arms.

**BC-2.16.013 v1.28 Route Coverage Table:** VERIFIED — 9 rows cover all sensor adapter routes enumerated in the spec; all routes verified present in DTU harness and adapter code; no gaps.

**STREAK:** 0/3 → **1/3** — CLEAN(strict)=YES on frozen 9fe2d016. Frozen-HEAD rule (DRIFT-ORCH-PRLEVEL-PUSH-001): HEAD unchanged since pass start; streak advances. Pass 28 NEXT on same frozen 9fe2d016.

**Code HEAD at review:** 9fe2d016 (frozen; just check FULL WORKSPACE 5476/5476 GREEN, 60 skipped; non-exhaustive 89/89; LOCAL-ONLY; develop baseline UNCHANGED @b9cf3f9b)

**CLEAN(strict):** YES — ZERO findings of any severity

**CLEAN(PR-merge):** YES — zero CRIT / HIGH / MED findings

---

## Findings

None. Zero findings at any severity level.

---

## Verification Summary

The adversary conducted a full-policy pass over frozen HEAD 9fe2d016 (fix/csdevices-empty-pipeline; LOCAL-ONLY). The following verification work was completed:

1. **SAP-1 catalog sweep** — grep `event_type\s*=` across all files changed relative to develop@b9cf3f9b; zero new emission sites found without BC-2.16.002 catalog rows.

2. **SAP-2 TOML↔DTU parity** — devices table: 6 TOML-declared columns verified against `prism-dtu-crowdstrike` response generator and fixture JSON. Excess-field gap (6 DTU fields not in TOML) pre-dates this branch and is correctly anchored to DRIFT-SAP2-DEVICES-TOML-SURFACE-001; no new divergence.

3. **BC-2.16.013 v1.28 Route Coverage Table** — 9 rows enumerated and verified against route registration in `prism-dtu-harness` CrowdStrike clone (both `build_router()` and `build_standalone_router()`) and corresponding adapter code. All 9 routes present and correctly mapped.

4. **POL-24 byte-strict E-QUERY-041/042/043 templates** — hint strings in error_taxonomy.md v2.39 compared byte-for-byte against `crates/prism-query/src/error.rs` and `crates/prism-query/src/plan_pinning.rs` production emission sites; no drift.

5. **Non-exhaustive ExampleKind 4-arm match** — `tests/external/non-exhaustive-violation/src/lib.rs` inspected; 4 arms present (PositiveE043, NegativeE043, and remaining variants); EXPECTED=89 in `ci.yml` UNCHANGED.

6. **Architect-ratified Ast::Pipe / Ast::SqlPipe wildcard invariant** — `pipe_sql_emitter::predicate_to_datafusion_sql` Err arm re-verified at source; T39 lock assertion verified consistent with adjudication (D-1675 Option B); no regression.

7. **SqlPipe stage-walk structural completeness** — `check_expr_insubquery_projection` SqlPipe arm: `spq.stages` walk covers `PipeStage::Where → check_predicate`; remaining `PipeStage` variants (Select, Extend, As, Aggregate) carry `FieldPath` list or non-predicate forms; no `Expr::InSubquery` surface reachable from unwalked arms.

8. **4-tier ExampleKind gate prose** — `build_reference_content` in `crates/prism-mcp/src/resources.rs`: "4-tier design" note present; no stale "3-tier" prose; `test_bc_2_11_022_ci_4tier_gate` test name confirmed correct.

All verification items passed. No findings to report.

---

## Streak Status

| Pass | Frozen HEAD | CLEAN(strict) | Streak |
|------|-------------|---------------|--------|
| 27 (this pass) | 9fe2d016 | YES | **1/3** |

Pass 28 NEXT on frozen 9fe2d016. If CLEAN(strict), streak advances to 2/3. If NOT CLEAN(strict), streak resets to 0/3.
