---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: FIX-IEQ-ERRPATH-001
passes: [9]
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
streak_after: 1/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 9 — FIX-IEQ-ERRPATH-001

---

## Pass 9 (frozen ddf852bc; fresh-context adversary; PR-LEVEL cascade; streak candidate 1/3 — ADVANCING — 0/3 → 1/3)

**Pass result:** CLEAN(strict)=YES, CLEAN(PR-merge)=YES

**Findings:** 0 total (0 CRIT / 0 HIGH / 0 MED / 0 LOW / 0 OBS / 0 PROCESS-GAP)

**SAP-1:** PASS — `rg 'event_type\s*=' crates/ --type rust` confirmed 91 emission sites; BC-2.16.002 v2.08 catalog arithmetic verified. All 15 probe sets returned empty-handed.

**STREAK:** ADVANCES 0/3 → **1/3** (BC-5.39.001; CLEAN(strict) on frozen ddf852bc). **Next: PR-LEVEL pass 10 on SAME frozen ddf852bc (streak candidate 2/3; NO push before pass 10 per DRIFT-ORCH-PRLEVEL-PUSH-001).**

**Code HEAD at review:** ddf852bc (frozen; PR #219 OPEN base develop@f935edb6; just check 5397/5397 GREEN; non-exhaustive 89/89)

**Code HEAD after pass:** ddf852bc UNCHANGED (no findings — no closure burst needed)

**CLEAN(strict):** YES — ZERO findings

**CLEAN(PR-merge):** YES — ZERO findings

---

## Findings

_None._

---

## Probe Summary

### Probe 1 — error-taxonomy v2.36 live-currency verification (pass-8 closure re-verification)

Direct read of error-taxonomy.md E-QUERY-038 row at ddf852bc worktree state. Verified: three live-currency pins now read "BC-2.11.016 v1.25 §Preconditions.2", "BC-2.11.016 v1.25" (DERIVED-COLUMN BINDING RULE), and "BC-2.11.016 v1.25" (BC anchor) — all correctly at v1.25. Origin-pin convention note present and accurate: eight additional anchors in §Changelog and §Origin correctly classified as origin pins (record the BC version that introduced each scope extension; not bumped on increments). Pass-8 closure verified correct under independent fresh-context read. **PASS.**

### Probe 2 — Chokepoint + per-site sanitization (CWE-117 / AD-017)

Verified `sanitize_for_log` applied at all 3 `column_not_found.rejected` emission sites (single-tenant path, multi-tenant path, binding-context path). Chokepoint at `ColumnNotFoundDetails::new` in `prism-core/src/error.rs` — sanitization applied before field assignment, preventing bypass via struct construction. MCP payload path independently sanitized via `ColumnNotFoundDetails::column` Display impl. Both layers independently load-bearing (TD-VSDD-059). No additional emission sites found by `rg 'ColumnNotFoundDetails'` sweep. **PASS.**

### Probe 3 — compute_did_you_mean short-circuit correctness + defensive arm safety

Verified dual-path logic: (a) suspension arm passes `allow_did_you_mean: false` — Levenshtein computation skipped entirely, `None` returned directly; (b) normal path passes `allow_did_you_mean: true`, candidates computed and filtered by `DID_YOU_MEAN_MAX_NAME_BYTES = 128`. Defensive guard at suspension boundary is present and structurally correct — no reachable code path bypasses the suspension gate. Short-circuit on byte length guard (SEC-002) applies correctly. **PASS.**

### Probe 4 — per-reference is_bare uniform across all 11 extractor match arms

Read `extract_field_paths_with_bareness` at ddf852bc (positions 1–14 per BC-2.11.016 v1.25 §Preconditions.2). All 11 match arms that push column references use `(name, is_bare)` pairs — no arm uses name-only keying. Per-reference scoping is correct (EC-11-076 D-1628): qualified references with `segments.len() == 2` are not suspended even when the unqualified name appears bare elsewhere in positions 1–6. Dead `bare_head_cols` set confirmed absent (removed in D-1628 implementer burst @35117a38). **PASS.**

### Probe 5 — cap_name_for_levenshtein CWE-407 guard at 3 sites

Verified `DID_YOU_MEAN_MAX_NAME_BYTES = 128` constant applied at all 3 `column_not_found.rejected` emission sites before computing did_you_mean candidates. No site omits the cap. Consistent with SEC-002 description in error-taxonomy.md v2.36. **PASS.**

### Probe 6 — MED-002 traced_tests load-bearing (control-char flip verified)

Re-verified the 3 `#[tracing_test::traced_test]` emission-path locks authored at `@7e23a2c2` (D-1633 test-writer burst):
- `test_column_not_found_single_tenant_traces_column_rejected` — reaches single-tenant execution path
- `test_column_not_found_multi_tenant_traces_column_rejected` — reaches multi-tenant execution path
- `test_column_not_found_binding_context_traces_column_rejected` — reaches binding-context execution path

Each test exercised the actual `tracing::warn!(event_type = "column_not_found.rejected", ...)` emission site, not the `sanitize_for_log` helper directly. Control-character injection test (`\x01test\ninjected`) confirmed to produce sanitized output (SOH and LF stripped). TD-VSDD-059 PASS — these tests are not paper-fixes. **PASS.**

### Probe 7 — BC-2.16.002 v2.08 row-177 3-site enumeration

Direct read of BC-2.16.002 `column_not_found.rejected` catalog entry (row added D-1633 product-owner burst). Verified: `sanitize_for_log` annotation present on `column` field (mirrors pattern from `infusion.coercion_failed` row per POL-30 Fork B). Three emission sites enumerated (single-tenant, multi-tenant, binding-context) — all match the code sites found by `rg`. Catalog arithmetic: 91 rows in BC-2.16.002 v2.08 = 91 `rg 'event_type\s*='` hits in `crates/`. **PASS.**

### Probe 8 — FP-001 trigger↔code completeness (bidirectional)

BC-2.11.016 v1.25 FP-001 trigger list has 6 named shapes (EC-11-041 / EC-11-062..064 / EC-11-074..075 / EC-11-076). Bidirectional check: (a) each of the 6 trigger shapes has a corresponding `suspended: true` code site in `check_binding_context_columns` or `compute_sqlpipe_head_binding`; (b) each `suspended: true` code site anchors to a named FP-001 shape in the spec. No orphaned code sites. No phantom spec shapes. **PASS.**

### Probe 9 — POL-16 / POL-12 (non-exhaustive gate + pub API surface)

`EXPECTED=89` gate in `ci.yml` unchanged. No new public types added to `prism-core`, `prism-spec-engine`, or `prism-query` in the FIX-IEQ-ERRPATH-001 cascade. All types added earlier in this cascade already carry `#[non_exhaustive]`. Perimeter-violation compile-fail gate at `tests/external/perimeter-violation/` not impacted. **PASS.**

### Probe 10 — #[non_exhaustive] discipline on PR diff surface

Direct review of PR diff scope: `ColumnNotFoundDetails` (existing type, carries `#[non_exhaustive]` since S-5.02). No new struct or enum types introduced in FIX-IEQ-ERRPATH-001 scope. **PASS.**

### Probe 11 — 91-event catalog arithmetic (SAP-1 full recount)

`rg 'event_type\s*=' crates/ --type rust` output: 91 hits. BC-2.16.002 v2.08 catalog: 91 named rows. One-to-one correspondence verified for the `column_not_found.rejected` row (3 emission sites, single catalog row with recurrence-policy `per_call`). **SAP-1 PASS.**

### Probe 12 — binding-context available_columns derived-view alignment

`compute_sqlpipe_head_binding` branches verified: (a) branch with non-empty JOIN — head binding columns derived from FROM-present columns only; bare references to JOIN columns are suspended (EC-11-074/075); (b) branch with empty JOIN — standard FROM-column binding with full E-QUERY-038 gate. No derived-view column leakage into the binding set. Alignment with BC-2.11.016 v1.25 §Preconditions.2 positions 1–4 confirmed. **PASS.**

### Probe 13 — grammar keyword pins (IEQ / IIN / INE operators)

`prism-parser/src/grammar.rs` keyword registration for `IEQ`, `IIN`, `INE`, and `IEQ_NOT` (alias for INE): all present at correct parser precedence positions. No stale grammar keyword references. Case-insensitive operator keyword table consistent with `valid_operators_for_type` exclusion logic in the query engine (String-typed columns only; Integer/Float/Boolean/Datetime excluded). **PASS.**

### Probe 14 — SAP-2 / SID-1

SAP-2: N/A — no sensor TOML spec modifications in FIX-IEQ-ERRPATH-001 cascade.
SID-1: N/A — no RED Gate deferrals outstanding in this scope.

### Probe 15 — CI status verification (pr-manager report)

pr-manager confirmed: CI ALL_PASS on ddf852bc (all 43/43 checks green; one infra flake cleared via targeted re-run — not a code defect). `mergeStateStatus: CLEAN`. PR body updated through pass-7 closures. Merge-readiness: BLOCKED_PENDING_CASCADE only (no review/CI/merge-conflict blocks). **PASS.**

---

## Standing Probe Results

**SAP-1 (Structured Event Catalog — BC-2.16.002):** PASS — 91 emission sites confirmed; BC-2.16.002 v2.08 catalog complete and unchanged. No new event_type values in this pass scope.

**SAP-2:** N/A — No sensor TOML spec modifications in this cascade.

**TD-VSDD-059 (paper-fix detection):** PASS — MED-002 traced_tests independently re-verified as load-bearing at actual emission paths. Error-taxonomy v2.36 live-pin corrections are structural version-string updates, not doc-comment renames.

**TD-VSDD-060 (sibling-site sweep):** PASS — no function signatures, constants, or canonical identifiers changed. No sweep required.

**BC-5.39.001 (3-CLEAN streak):** 0/3 → **1/3** — pass-9 CLEAN(strict) on frozen ddf852bc. Next pass (pass 10) is streak candidate 2/3 on same frozen ddf852bc. No push between passes (DRIFT-ORCH-PRLEVEL-PUSH-001 satisfied).

---

## Convergence Assessment

**Trajectory:** LOCAL 19 passes on frozen 35117a38 (3-CLEAN D-1631) → PR-LEVEL pass 1 on frozen dacb60fa: 3 findings (0/0/2/0/1/0) [NOT CLEAN] → same-burst fix pushed @39c8b134 (streak reset) → **PR-LEVEL pass 2 on frozen 39c8b134: 0 findings (CLEAN; streak 1/3)** → **PR-LEVEL pass 3 on frozen 39c8b134: 3 findings (0/0/0/1/2/0) [NOT CLEAN; streak RESET 0/3]** → same-burst fix pushed @8610ecd0 → **PR-LEVEL pass 4 on frozen 8610ecd0: 1 finding (0/0/1/0/0/0) [NOT CLEAN; streak stays 0/3]** → same-burst spec-only closure (HEAD UNCHANGED) → **PR-LEVEL pass 5 on frozen 8610ecd0: 3 findings (0/0/3/0/0/0) [NOT CLEAN; streak stays 0/3]** → same-burst spec-only closure (HEAD UNCHANGED) → **PR-LEVEL pass 6 on frozen 8610ecd0: 0 findings (CLEAN(strict); streak 0/3 → 1/3)** → **PR-LEVEL pass 7 on frozen 8610ecd0: 1 finding (0/0/1/0/0/0) [NOT CLEAN(strict); streak RESET 1/3 → 0/3]** → same-burst fix pushed @ddf852bc → **PR-LEVEL pass 8 on frozen ddf852bc: 1 finding (0/0/0/1/0/0) [NOT CLEAN(strict); streak stays 0/3]** → same-burst spec-only closure (HEAD ddf852bc UNCHANGED) → **PR-LEVEL pass 9 on frozen ddf852bc: 0 findings (CLEAN(strict); streak 0/3 → 1/3)**

**Decay signature:** 3→0→3→1→3→0→1→1→**[0]**. Zero-finding pass on first attempt against frozen ddf852bc after spec-only closure in pass-8. Code and spec-logic surfaces remain clean across all 9 PR-LEVEL passes (zero CRIT/HIGH code-behavior defects in the entire PR-LEVEL cascade).

**Novelty:** ZERO — no new issues surfaced. All 15 probes returned empty-handed, including independent re-verification of pass-8 spec-only closures (taxonomy v2.36 live pins, per-reference is_bare scoping).

**Streak status:** **1/3** — advances from 0/3 to 1/3. **NEXT: PR-LEVEL adversary pass 10 on SAME frozen HEAD ddf852bc** (streak candidate 2/3; NO push before pass 10 per DRIFT-ORCH-PRLEVEL-PUSH-001).
