---
document_type: adversarial-review
scope: LOCAL
fix_defect: DEFECT-CSDEVICES-EMPTY-PIPELINE-001
passes: [28]
feature_head_at_review: 9fe2d016
date: 2026-07-11
clean_strict: false
clean_pr_merge: true
finding_counts:
  total: 1
  crit: 0
  high: 0
  med: 0
  low: 0
  obs: 1
  process_gap: 0
code_behavior_defects: 0
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: adversary
---

# LOCAL Adversary Pass 28 — DEFECT-CSDEVICES-EMPTY-PIPELINE-001

---

## Pass 28 (frozen 9fe2d016; fresh-context adversary; LOCAL cascade; policy rubric + SAP-1 + SAP-2 + POL-22 + POL-24 byte-strict + POL-33 table verification; streak candidate 2/3 — STREAK RESET — streak 1/3 → 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=YES

**Findings:** 1 total (0 CRIT / 0 HIGH / 0 MED / 0 LOW / 1 OBS / 0 PROCESS-GAP)

**Adversary novelty assessment:** LOW — single OBS on a pre-existing annotation gap surfaced by perimeter exposure of the file; all structural and behavioral probes returned empty-handed.

**SAP-1:** PASS — `rg 'event_type\s*=' crates/ --type rust` across all files changed relative to develop@b9cf3f9b: five SQL emission sites verified (pipe.sql_lowering × 3, pipe.sql_planning_error × 2); all five catalog rows present in BC-2.16.002 §Postconditions (v2.10); no new emission sites introduced without catalog rows.

**SAP-2:** PASS — devices TOML spec: all 6 TOML-declared columns (hostname, device_id, first_seen, last_seen, status, platform_name) present in DTU generator and fixture. Excess-field gap (os_version, containment_status, external_ip, local_ip, agent_version, cid/agent_id) correctly deferred to DRIFT-SAP2-DEVICES-TOML-SURFACE-001 (human-directed; D-1666 decision 4). No new divergence introduced by this branch.

**POL-22:** PASS — All cited entities resolve; no dangling cross-references.

**POL-24 (byte-strict):** PASS — E-QUERY-041/042/043 hint strings verified byte-exact against error-taxonomy v2.39 and production code; no drift.

**POL-33:** PASS — BC-2.16.013 v1.28 Route Coverage Table (9 rows) verified against all relevant routes in code.

**Non-exhaustive gate (EXPECTED=89):** PASS for all previously-gated types; see F-CSD-P28-OBS-001 below for the one gap found.

**STREAK:** 1/3 → **0/3** — CLEAN(strict)=NO (1 OBS finding). Streak RESET.

**Code HEAD at review:** 9fe2d016 (frozen; just check FULL WORKSPACE 5476/5476 GREEN, 60 skipped; non-exhaustive 89/89; LOCAL-ONLY; develop baseline UNCHANGED @b9cf3f9b)

**CLEAN(strict):** NO — 1 OBS finding
**CLEAN(PR-merge):** YES — zero CRIT / HIGH / MED findings

---

## Findings

### F-CSD-P28-OBS-001 — OBS/LOW — `prism_core::virtual_fields::VirtualField` missing `#[non_exhaustive]`

**Severity:** OBS (LOW per CLAUDE.md non_exhaustive discipline — non_exhaustive is a correctness requirement, not style)

**File:** `crates/prism-core/src/virtual_fields.rs`

**Description:** `pub enum prism_core::virtual_fields::VirtualField` (derives `Serialize` + `Deserialize`; 3 variants: `Sensor`, `Client`, `SourceTable`) lacks `#[non_exhaustive]`. This type is in the branch perimeter (file touched by an earlier pass's doc-only change). The sibling enum `prism_query::ast::VirtualField` (4-variant) is already `#[non_exhaustive]` compliant. CLAUDE.md convention is unambiguous: `pub` + `Deserialize` ⇒ `#[non_exhaustive]`. The 3-variant vs 4-variant layer split is documented (BC-2.11.012 scope; internal-table exclusion) and does NOT exempt the type from the annotation requirement — the documented split and the `#[non_exhaustive]` annotation are independent obligations.

**Risk:** External crates that exhaustively match on `VirtualField` will fail to compile when a new variant is added (e.g., when DRIFT-INTERNAL-TABLE-COLUMN-GATE-001 queues a new variant), with no forward-compatibility guarantee in the API surface.

**Resolution (in-scope):** Implementer adds `#[non_exhaustive]` to `prism_core::virtual_fields::VirtualField`. TD-VSDD-060 sweep required: confirm the only match site is in the defining crate (stays exhaustive there); all external refs use method-call or structural-access patterns (no external match arms affected). Gate `v90_virtual_field_match` (E0004) added to `tests/external/non-exhaustive-violation/src/lib.rs`; `EXPECTED` counter bumped 89→90 in `ci.yml` (both job matrices + changelog comment) and `scripts/check-non-exhaustive.sh`.

**Adjudication:** In-scope. No architect input required — CLAUDE.md convention unambiguous; TD-VSDD-060 sweep confirmed single defining-crate match site; docs-only file touch brought file into perimeter. Fix dispatched to implementer; implementer commit @25b80a81.

---

## Verification Summary

The adversary conducted a full-policy pass over frozen HEAD 9fe2d016 (fix/csdevices-empty-pipeline; LOCAL-ONLY). The following verification work was completed:

1. **SAP-1 catalog sweep** — grep `event_type\s*=` across all files changed relative to develop@b9cf3f9b; five production SQL emission sites sanitize_for_log verified (pipe.sql_lowering × 3, pipe.sql_planning_error × 2); all five catalog rows present in BC-2.16.002 §Postconditions (v2.10); no new emission sites.

2. **SAP-2 TOML↔DTU parity** — devices table: 6 TOML-declared columns verified against `prism-dtu-crowdstrike` response generator and fixture JSON. Excess-field gap (6 DTU fields not in TOML) pre-dates this branch and is correctly anchored to DRIFT-SAP2-DEVICES-TOML-SURFACE-001; no new divergence.

3. **BC-2.16.013 v1.28 Route Coverage Table** — 9 rows enumerated and verified against route registration in `prism-dtu-harness` CrowdStrike clone (both `build_router()` and `build_standalone_router()`) and corresponding adapter code. All 9 routes present and correctly mapped.

4. **POL-24 byte-strict E-QUERY-041/042/043 templates** — hint strings in error_taxonomy.md v2.39 compared byte-for-byte against `crates/prism-query/src/error.rs` and `crates/prism-query/src/plan_pinning.rs` production emission sites; no drift.

5. **Non-exhaustive gate sweep** — enumerated all pub types added or modified by the branch; found `prism_core::virtual_fields::VirtualField` missing annotation (F-CSD-P28-OBS-001); all other branch-introduced pub types correctly annotated; EXPECTED=89 is current for the pre-fix state.

6. **Architect-ratified Ast::Pipe / Ast::SqlPipe wildcard invariant** — `pipe_sql_emitter::predicate_to_datafusion_sql` Err arm re-verified; T39 lock consistent with D-1675 Option B adjudication; no regression.

7. **SqlPipe stage-walk structural completeness** — `check_expr_insubquery_projection` SqlPipe arm: `spq.stages` walk covers `PipeStage::Where → check_predicate`; remaining `PipeStage` variants carry `FieldPath` or non-predicate forms; no `Expr::InSubquery` surface reachable from unwalked arms.

8. **Load-bearing test verification** — T39 (Pipe wildcard boundary) / T40 (SqlPipe head InSubquery E-QUERY-043 lock) / T41 (SqlPipe stages walk RED) verified GREEN; `negative_e043_parity_gate` (passes 25-27 locked) verified intact; ExampleKind 4-arm match in `non-exhaustive-violation` crate verified (EXPECTED=89 for current HEAD).

---

## Fix Record

**Fix-burst:** implementer @25b80a81

- `#[non_exhaustive]` added to `prism_core::virtual_fields::VirtualField`
- TD-VSDD-060 sweep: only match site is in defining crate (`virtual_fields.rs` itself, stays exhaustive); `prism-storage` uses method calls only; all other refs are the `prism_query::ast::VirtualField` enum (separate type, already compliant)
- Gate `v90_virtual_field_match` (E0004) added to `tests/external/non-exhaustive-violation/src/lib.rs`
- `EXPECTED` counter 89→90 in `ci.yml` (both message lists + changelog comment) and `scripts/check-non-exhaustive.sh`
- `.worktrees/FIX-CSDEVICES-EMPTY-PIPELINE/CLAUDE.md` convention sentence 89→90 + provenance entry appended
- `just check` FULL WORKSPACE GREEN at @25b80a81; non-exhaustive gate 90/90

**New FROZEN HEAD for pass 29:** 25b80a81 (LOCAL-ONLY). Streak 0/3.

---

## Streak Status

| Pass | Frozen HEAD | CLEAN(strict) | Streak |
|------|-------------|---------------|--------|
| 27 (prev) | 9fe2d016 | YES | 1/3 |
| 28 (this pass) | 9fe2d016 | NO (1 OBS) | **0/3 RESET** |

Pass 29 NEXT on NEW frozen HEAD 25b80a81. Streak restarts from 0/3. If CLEAN(strict), streak advances to 1/3. If NOT CLEAN(strict), streak stays 0/3.
