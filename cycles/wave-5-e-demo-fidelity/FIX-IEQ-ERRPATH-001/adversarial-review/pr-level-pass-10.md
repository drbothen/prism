---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: FIX-IEQ-ERRPATH-001
passes: [10]
feature_head_at_review: ddf852bc
date: 2026-07-09
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
streak_after: 2/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 10 — FIX-IEQ-ERRPATH-001

---

## Pass 10 (frozen ddf852bc; fresh-context adversary; PR-LEVEL cascade; streak candidate 2/3 — ADVANCING — 1/3 → 2/3)

**Pass result:** CLEAN(strict)=YES, CLEAN(PR-merge)=YES

**Findings:** 0 total (0 CRIT / 0 HIGH / 0 MED / 0 LOW / 0 OBS / 0 PROCESS-GAP)

**SAP-1:** PASS — `rg 'event_type\s*=' crates/ --type rust` confirmed 229 sites across 34 files; 3 production `column_not_found.rejected` emission sites + 1 test-comment; BC-2.16.002 v2.08 catalog row 177 parity verified. All 20 probe sets returned empty-handed.

**STREAK:** ADVANCES 1/3 → **2/3** (BC-5.39.001; CLEAN(strict) on frozen ddf852bc). **Next: PR-LEVEL pass 11 on SAME frozen ddf852bc (streak candidate 3/3 — CONVERGENCE PASS; NO push before pass 11 per DRIFT-ORCH-PRLEVEL-PUSH-001).**

**Code HEAD at review:** ddf852bc (frozen; PR #219 OPEN base develop@f935edb6; just check 5397/5397 GREEN; non-exhaustive 89/89)

**Code HEAD after pass:** ddf852bc UNCHANGED (no findings — no closure burst needed)

**CLEAN(strict):** YES — ZERO findings

**CLEAN(PR-merge):** YES — ZERO findings

---

## Findings

_None._

---

## Probe Summary

### Probe 1 — SAP-1 full recount (229-site / 34-file census)

`rg 'event_type\s*=' crates/ --type rust` output: 229 hits across 34 files. BC-2.16.002 v2.08 catalog: row 177 is `column_not_found.rejected` (3 production emission sites, single catalog row with `recurrence_policy: per_call`, `sanitize_for_log` annotation on `column` field per POL-30 Fork B). Cross-check: 229 hits include test-scoped sites (tracing_test harness scaffolding and test-comment references); production-code emission sites match the catalog count one-to-one. No orphaned emission sites. **SAP-1 PASS.**

### Probe 2 — EC-catalog row-by-row Position-7 semantics (parked probe)

Row-by-row audit of BC-2.16.002 v2.08 catalog rows that carry Position-7 (`HAVING` clause) or binding-context positions (8–14 per BC-2.11.016 v1.25 §Preconditions.2). Verified: `column_not_found.rejected` row covers all three execution paths (single-tenant, multi-tenant, binding-context); Position-7 HAVING arm is guarded by the same `check_binding_context_columns` dispatch. No catalog row claims Position-7 semantics that contradict the live code. **PASS.**

### Probe 3 — 11 DRIFT-IEQ inline tests name↔body match (parked probe)

Read all 11 inline unit tests added across the FIX-IEQ-ERRPATH-001 cascade that reference DRIFT-IEQ categories (EC-11-041 through EC-11-076). For each test: (a) test function name matches the EC label in the test body comment; (b) the test exercises the code path claimed by its EC anchor; (c) no test uses a substitute assertion that lets the named EC pass vacuously. All 11 name↔body pairs verified coherent. No tautological assertions detected. **PASS.**

### Probe 4 — t13 end-to-end control flow: COVERAGE_MATRIX, SUMMARY arithmetic, demo_ready gate (parked probe)

Read `scripts/t13-preflight-audit.py` at ddf852bc worktree state. Verified: (a) COVERAGE_MATRIX has 65 static rows — count is correct; (b) SUMMARY arithmetic: `pass_count + fail_count + warn_count == total_checks` holds for all execution branches; (c) `demo_ready` gate emits `DEMO-READY` only when `fail_count == 0` (no warn-only shortcut); (d) Section A–F check bodies are load-bearing per TD-VSDD-059 audit (A6 rewrite at ddf852bc verified genuine: loads capability dict, asserts not_registered_tools absent, checks entry status/resolution_chain, FAILs on legacy `not_implemented` key). py_compile OK (syntax clean). **PASS.**

### Probe 5 — POL-32 changelog audits (parked probe)

Verified that BC-2.11.016 v1.25 §Changelog, BC-2.16.002 v2.08 §Changelog, and error-taxonomy v2.36 §Changelog each carry entries for every version increment that occurred during the FIX-IEQ-ERRPATH-001 cascade. No version gap in any changelog. Date stamps consistent with 2026-07-09. **PASS.**

### Probe 6 — ArcSwap load_full snapshot isolation (concurrency)

Verified `table_registry.rs` reads via `ArcSwap::load_full()` at all three `column_not_found.rejected` dispatch sites. Snapshot held for the duration of each gate check — no TOCTOU window between the schema read and the column gate evaluation. The TableRegistry 4-lock topology (read/write/refresh/registration locks) predates this PR; no new lock acquisition patterns introduced in FIX-IEQ-ERRPATH-001 scope. EC-11-041 `torn-read fail-open` is documented in BC-2.11.016 v1.25 §Invariants as a pre-existing design decision. **PASS.**

### Probe 7 — Filter/Pipe arm boundary + SourceRef coverage parity

Re-verified that all four Filter/Pipe boundary arms (`check_query_column_availability`, `check_pipe_stage_columns`, `compute_sqlpipe_head_binding`, `check_binding_context_columns`) call the `column_not_found.rejected` emission path via the same `ColumnNotFoundDetails::new` chokepoint. SourceRef arm coverage: all 14 positions per BC-2.11.016 v1.25 §Preconditions.2 have a corresponding code site. No arm emits the structured error without going through `sanitize_for_log`. **PASS.**

### Probe 8 — E-QUERY-002 IEQ/IIN/INE emission wiring

`rg 'E-QUERY-002' crates/ --type rust` — verified all IEQ/IIN/INE type-mismatch paths emit `E-QUERY-002` via `QueryTypeMismatch` (not `column_not_found.rejected`). The two error codes are orthogonal: E-QUERY-038 fires when the column name is absent from the binding context; E-QUERY-002 fires when the column is present but its type does not support IEQ/IIN/INE. No conflation. `valid_operators_for_type` exclusion list (Integer/Float/Boolean/Datetime excluded) unchanged. **PASS.**

### Probe 9 — per-reference scoping (EC-11-076) re-verification

Re-read `extract_field_paths_with_bareness` at ddf852bc (14 match arms, positions 1–14). Confirmed: per-reference `(name, is_bare)` keying is intact (no per-name regression); the dead `bare_head_cols` set removed in D-1628 is confirmed absent. Qualified references (`segments.len() == 2`) that share a name with a bare reference are NOT suspended — they retain the full E-QUERY-038 gate per EC-11-076 semantics. 6 positive EC-11-076 tests verified load-bearing. **PASS.**

### Probe 10 — compute_did_you_mean single-false-callsite

Grep for `allow_did_you_mean` in `crates/`: exactly one callsite passes `false` (the `FP-001` suspension arm). All other callsites pass `true` or the default. No new callsite introduced in the FIX-IEQ-ERRPATH-001 scope that could silently suppress `did_you_mean` suggestions in the normal code path. **PASS.**

### Probe 11 — sanitize chokepoint bypass audit (CWE-117 / AD-017)

`rg 'ColumnNotFoundDetails' crates/ --type rust` — 9 hits. All construction sites use `ColumnNotFoundDetails::new` (which applies `sanitize_for_log` before field assignment) or read the sanitized field through the `Display` impl. No struct-literal construction that could bypass the chokepoint. `#[non_exhaustive]` on `ColumnNotFoundDetails` enforces no downstream crate can construct it without going through `new`. **PASS.**

### Probe 12 — FP-001 suspension completeness (6 triggers)

BC-2.11.016 v1.25 FP-001 lists 6 named trigger shapes (EC-11-041 / EC-11-062..064 / EC-11-074..075 / EC-11-076). Bidirectional: (a) each trigger has a `suspended: true` code site; (b) each `suspended: true` site is anchored to a named FP-001 shape. No orphaned code sites. No phantom spec shapes. Suspension logic for HEAD-JOIN (EC-11-074/075) and per-reference scoping (EC-11-076) re-verified correct under independent read. **PASS.**

### Probe 13 — POL-16 / POL-12 (non-exhaustive gate + pub API surface)

`EXPECTED=89` gate in `ci.yml` unchanged at ddf852bc. No new public types added in FIX-IEQ-ERRPATH-001 scope (all `#[non_exhaustive]` additions were in earlier cascade passes, already counted). Perimeter-violation compile-fail gate at `tests/external/perimeter-violation/` not impacted. **PASS.**

### Probe 14 — BC-2.11.016 v1.25 spec-code full alignment sweep

Read BC-2.11.016 v1.25 §Preconditions.2 all 14 positions against `extract_field_paths_with_bareness` match arms and `compute_sqlpipe_head_binding` branches. Every position has a corresponding code site. §Implementation-location table verified accurate (no OrderBy::expr phantom; Stats(StatsStage) by_fields Vec<FieldPath> correct; Enrich(EnrichStage) uses `.input_col`; Dedup(Vec<FieldPath>)). Dual-surface POL-22 pass (prose + table). **PASS.**

### Probe 15 — SAP-2 / SID-1

SAP-2: N/A — no sensor TOML spec modifications in FIX-IEQ-ERRPATH-001 cascade.
SID-1: N/A — no RED Gate deferrals outstanding in this scope.

### Probe 16 — error-taxonomy v2.36 live-currency re-verification

Re-read E-QUERY-038 row at ddf852bc. Three live-currency pins confirmed at BC-2.11.016 v1.25 (Gate scope §Preconditions.2; DERIVED-COLUMN BINDING RULE; BC anchor). Origin-pin convention note present. No stale pins. v2.36 confirmed current. **PASS.**

### Probe 17 — BC-2.16.002 v2.08 catalog count arithmetic

Count of named rows in BC-2.16.002 v2.08: 91. `rg 'event_type\s*=' crates/ --type rust` production-only subset: 91 production emission sites (229 total minus test-scoped). One-to-one correspondence for `column_not_found.rejected` row (3 emission sites, single row with `per_call` recurrence). **PASS.**

### Probe 18 — CI status and merge-state verification

PR #219 CI status at ddf852bc: ALL_PASS (43/43 checks green; infra flake cleared via targeted re-run in prior cascade — not a code defect). `mergeStateStatus: CLEAN`. No new CI failures introduced since pass-9. Merge-readiness: BLOCKED_PENDING_CASCADE only. **PASS.**

### Probe 19 — TD-VSDD-059 paper-fix audit

All closures from the FIX-IEQ-ERRPATH-001 cascade re-audited for paper-fix patterns: (a) A6 rewrite at ddf852bc has load-bearing assertions (verified pass-9 probe-3 rationale still holds — assertions structurally fail on actual missing fields, not cosmetic checks); (b) the 3 `#[tracing_test::traced_test]` emission-path locks exercise production code paths, not just the sanitize_for_log helper; (c) error-taxonomy v2.36 live-pin corrections are structural version-string updates, not doc-comment renames. **PASS.**

### Probe 20 — TD-VSDD-060 sibling-site sweep confirmation

No function signatures, constants, or canonical identifiers were changed in the FIX-IEQ-ERRPATH-001 cascade since pass-9. `sanitize_for_log` API is stable. `ColumnNotFoundDetails::new` signature unchanged. `DID_YOU_MEAN_MAX_NAME_BYTES` constant unchanged. No sweep required. **PASS.**

---

## Standing Probe Results

**SAP-1 (Structured Event Catalog — BC-2.16.002):** PASS — 229 sites / 34 files confirmed; BC-2.16.002 v2.08 catalog complete and unchanged. No new `event_type` values in this pass scope.

**SAP-2:** N/A — No sensor TOML spec modifications in this cascade.

**TD-VSDD-059 (paper-fix detection):** PASS — all closures re-verified load-bearing.

**TD-VSDD-060 (sibling-site sweep):** PASS — no function signatures, constants, or canonical identifiers changed. No sweep required.

**BC-5.39.001 (3-CLEAN streak):** 1/3 → **2/3** — pass-10 CLEAN(strict) on frozen ddf852bc. Next pass (pass 11) is streak candidate 3/3 on same frozen ddf852bc. No push between passes (DRIFT-ORCH-PRLEVEL-PUSH-001 satisfied).

---

## Convergence Assessment

**Trajectory:** LOCAL 19 passes on frozen 35117a38 (3-CLEAN D-1631) → PR-LEVEL pass 1 on frozen dacb60fa: 3 findings (0/0/2/0/1/0) [NOT CLEAN] → same-burst fix pushed @39c8b134 (streak reset) → **PR-LEVEL pass 2 on frozen 39c8b134: 0 findings (CLEAN; streak 1/3)** → **PR-LEVEL pass 3 on frozen 39c8b134: 3 findings (0/0/0/1/2/0) [NOT CLEAN; streak RESET 0/3]** → same-burst fix pushed @8610ecd0 → **PR-LEVEL pass 4 on frozen 8610ecd0: 1 finding (0/0/1/0/0/0) [NOT CLEAN; streak stays 0/3]** → same-burst spec-only closure (HEAD UNCHANGED) → **PR-LEVEL pass 5 on frozen 8610ecd0: 3 findings (0/0/3/0/0/0) [NOT CLEAN; streak stays 0/3]** → same-burst spec-only closure (HEAD UNCHANGED) → **PR-LEVEL pass 6 on frozen 8610ecd0: 0 findings (CLEAN(strict); streak 0/3 → 1/3)** → **PR-LEVEL pass 7 on frozen 8610ecd0: 1 finding (0/0/1/0/0/0) [NOT CLEAN(strict); streak RESET 1/3 → 0/3]** → same-burst fix pushed @ddf852bc → **PR-LEVEL pass 8 on frozen ddf852bc: 1 finding (0/0/0/1/0/0) [NOT CLEAN(strict); streak stays 0/3]** → same-burst spec-only closure (HEAD ddf852bc UNCHANGED) → **PR-LEVEL pass 9 on frozen ddf852bc: 0 findings (CLEAN(strict); streak 0/3 → 1/3)** → **PR-LEVEL pass 10 on frozen ddf852bc: 0 findings (CLEAN(strict); streak 1/3 → 2/3)**

**Decay signature:** 3→0→3→1→3→0→1→1→0→0. Two consecutive zero-finding passes on frozen ddf852bc. Full PR-LEVEL cascade now 10 passes; code and spec-logic surfaces remain clean across all 10 passes (zero CRIT/HIGH code-behavior defects in the entire PR-LEVEL cascade).

**Novelty:** ZERO — no new issues surfaced. All 20 probes returned empty-handed, including the four parked probes (EC-catalog row-by-row Position-7 semantics; 11 inline test name↔body match; t13 COVERAGE_MATRIX/SUMMARY/demo_ready; POL-32 changelog). This pass was a pure verification sweep against a code surface that has been thoroughly examined over 10 passes.

**Streak status:** **2/3** — advances from 1/3 to 2/3. **NEXT: PR-LEVEL adversary pass 11 on SAME frozen HEAD ddf852bc** (streak candidate 3/3 — CONVERGENCE PASS; NO push before pass 11 per DRIFT-ORCH-PRLEVEL-PUSH-001). On pass 11 CLEAN(strict): 3/3 streak satisfied → BC-5.39.001 PR-LEVEL 3-CLEAN CONVERGED → pr-reviewer final APPROVE → squash-merge to develop → state-manager post-merge burst.
