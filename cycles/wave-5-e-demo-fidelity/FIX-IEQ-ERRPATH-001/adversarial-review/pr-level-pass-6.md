---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: FIX-IEQ-ERRPATH-001
passes: [6]
feature_head_at_review: 8610ecd0
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

# PR-LEVEL Adversary Pass 6 — FIX-IEQ-ERRPATH-001

---

## Pass 6 (frozen 8610ecd0; fresh-context adversary; PR-LEVEL cascade; streak candidate 1/3 — ADVANCING — 0/3 → 1/3)

**Pass result:** CLEAN(strict)=YES, CLEAN(PR-merge)=YES

**Findings:** 0 total (0 CRIT / 0 HIGH / 0 MED / 0 LOW / 0 OBS / 0 PROCESS-GAP)

**SAP-1:** PASS — `rg 'event_type\s*=' crates/ --type rust` returned no new values; BC-2.16.002 v2.08 catalog complete and unchanged.

**STREAK:** 1/3 — CLEAN(strict)=YES on frozen 8610ecd0 (zero findings). PR HEAD UNCHANGED. Per BC-5.39.001, streak advances 0/3 → 1/3. **Next: PR-LEVEL pass 7 on SAME frozen 8610ecd0 (streak candidate 2/3; NO push before pass 7 per DRIFT-ORCH-PRLEVEL-PUSH-001).**

**Code HEAD at review:** 8610ecd0 (frozen; PR #219 OPEN base develop@f935edb6; just check 5397/5397 GREEN; non-exhaustive 89/89)

**CLEAN(strict):** YES — 0 findings

**CLEAN(PR-merge):** YES — 0 findings

---

## Findings

None.

---

## Probe Summary

### Probe 1 — Newest spec edits vs ast.rs/engine.rs code alignment

BC-2.11.016 v1.25 §Preconditions.2 positions 8/11/4 prose corrections re-verified against `crates/prism-query/src/ast.rs`:
- Position-8: `PipeStage::Where(Predicate)` — confirmed; prose now reads "embedded `Predicate` tree" — CORRECT
- Position-11: `PipeStage::Stats(StatsStage)` with `StatsStage.by_fields: Vec<FieldPath>` — confirmed against `ast.rs` tuple-variant definition and `StatsStage` struct — CORRECT
- Position-4: `OrderExpr::expr` (not `OrderBy::expr`; `OrderBy` does not exist in `ast.rs`) — confirmed — CORRECT

BC-2.11.016 v1.25 §Implementation location table re-verified: all six suspension rules land in code at their documented call-sites in `engine.rs`. Position-11 `AggFunc` 7-variant coverage confirmed present. All 14 positions clean.

BC-2.11.004 v1.30 §Error Cases re-verified against `ast.rs`: four v1.29→v1.30 fixes confirmed load-bearing — `Where(Predicate)` tree, `Stats(StatsStage)` by_fields, `Enrich(EnrichStage).input_col` (not `.field`), `Dedup(Vec<FieldPath>)`. All four AST shapes present verbatim in `ast.rs`.

### Probe 2 — S-DEMO-FIDELITY-REMEDIATION-001 v2.44 AC-M2 sub-arm attribution

AC-M2 two-sub-arm split re-verified against `crates/prism-query/src/engine.rs`:
- Multi-segment `FieldPath` sub-arm: recurses via `extract_field_paths_with_bareness` — CONFIRMED present in code
- Single-segment bare `Field` sub-arm: calls `extract_column_name_from_field_path` directly, classifying with `is_bare: true` — CONFIRMED distinct code path, NOT a recursive call to `extract_field_paths_with_bareness`

The architectural boundary between the recursive path and the direct extraction path is correctly described in v2.44. No "both cases recurse" residual found.

### Probe 3 — FP-001 trigger-list ↔ code suspension-site bidirectional completeness

FP-001 trigger list in BC-2.11.016 v1.25 contains 6 shapes. Code scan via `grep -n 'suspended.*true\|suspended: true' crates/prism-query/src/engine.rs`:
- 7 `suspended: true` sites found (6 FP-001 shapes + 1 per-reference HEAD-JOIN discard arm)
- All 7 sites trace to a documented FP-001 shape or the HEAD-JOIN suspension rule per BC-2.11.016 v1.25 §Suspension rules
- No orphaned `suspended: true` sites found outside the spec-anchored shapes

Bidirectional coverage complete: every spec shape has a code site; every code site has a spec anchor.

### Probe 4 — 10 novel query-shape traces (not in EC catalog)

Ten query shapes exercised via mental trace through `engine.rs` binding-context walk (E-QUERY-038 gate):
1. `SELECT a, b FROM t | where a = 'x'` — positions 1+8 walked; `a` and `b` both in HEAD; no suspension; CORRECT gate
2. `SELECT a FROM t | sort a DESC` — position-10 walked; `a` in HEAD; CORRECT
3. `SELECT a FROM t | stats count() by a` — position-11 walked; `a` in HEAD (by_fields); CORRECT
4. `SELECT a, b FROM t JOIN s ON t.a = s.a | where b = 1` — HEAD-JOIN FP-001 suspension for bare `a`/`b` refs in position-8 walk; CORRECT
5. `SELECT a AS x FROM t | sort x` — position-10; `x` seeded from HEAD alias; CORRECT
6. `SELECT count(*) AS cnt FROM t | sort cnt` — position-10; `cnt` alias seeded from HEAD; CORRECT
7. `SELECT * FROM t | dedup a, b` — positions 1+14; STAR seeds all columns; CORRECT
8. `SELECT a FROM t | where a IN (1, 2, 3)` — IIN operator; position-8; CORRECT
9. `SELECT a FROM t | where a IEQ 'FOO'` — IEQ operator; position-8; CORRECT
10. `SELECT a FROM t1, t2 | where a = 1` — multi-source FROM; HEAD seeding; CORRECT

All 10 shapes: spec-code agreement confirmed. No false E-QUERY-038 triggers and no missed gates found.

### Probe 5 — POL-24 byte-verbatim E-QUERY-038 error message incl. did_you_mean suffix

`ColumnNotFoundInTable` error formatting verified: the display string matches the POL-24 byte-verbatim contract in error-taxonomy v2.35. The `did_you_mean` suffix path (Levenshtein ≤3 candidate) emits the sanitized suggestion via `sanitize_for_log`. The suspension arm correctly passes `compute_did_you_mean: false` so no Levenshtein computation occurs for suspended references.

### Probe 6 — Chokepoint + 3 log sites sanitization dual-layer

`ColumnNotFoundDetails::new` chokepoint (from `@39c8b134` fix): `sanitize_for_log` applied at construction time — CONFIRMED present. Three `column_not_found.rejected` emission sites in engine.rs use the pre-sanitized `ColumnNotFoundDetails` — no raw column string reaches tracing. CWE-117 dual-layer (construction-time sanitize + MCP-payload sanitize) verified structurally intact.

### Probe 7 — Test-fixture soundness

Sampled 8 test fixtures in `crates/prism-query/tests/`:
- All use canonical constructors (no struct literal bypass of `#[non_exhaustive]`)
- Multi-tenant fixtures contain realistic `OrgSlug` and sensor shape
- `OrgSlug::new_unchecked` callsites verified against allowlist in `new_unchecked_audit.rs` — no new unauthorized sites

### Probe 8 — compute_did_you_mean dual-path short-circuit

`compute_did_you_mean: bool` parameter verified in all 9 callsites (TD-VSDD-060 sweep from D-1635). The suspension arm passes `false` at all relevant sites. The live gate arm passes `true`. No site passes `true` under a suspension condition. Short-circuit semantics correct.

### Probe 9 — POL-13 spot-check + audit-script exit-1 anti-pattern

`t13-preflight-audit.py` reviewed for exit-1 anti-pattern (from ADV-PR-P3-OBS-002 history): G4 now uses canonical `check_query_column_availability` function anchor (not volatile `engine.rs:NNN` line number). Audit-script `fail_count == 0` gate confirmed WILL exit with non-zero status on gate failure — not exhibiting the exit-1 anti-pattern. G2/G3/G6/G7/G8 FAIL-on-error confirmed (NB-2 closure from D-1632 intact).

### Probe 10 — POL-16/POL-12 clean

Test suite spot-checked for tautological assertions (POL-16) and test-naming compliance (POL-12). 10 sampled tests across `crates/prism-query/tests/`:
- All assertions exercise behavior with non-trivially-true conditions
- Test names follow `test_BC_N_NN_NNN_*` or `test_EC_NN_NNN_*` naming conventions
- No `assert!(true)` or trivially-passing assertion patterns found

---

## Version Summary

**No spec/story version changes this pass.** Pass-6 is a CLEAN pass with zero findings. All spec and story versions carry forward from D-1637:
- BC-2.11.016 v1.25 (UNCHANGED)
- BC-2.11.017 v1.13 (UNCHANGED)
- BC-2.11.020 v1.18 (UNCHANGED)
- BC-2.11.004 v1.30 (UNCHANGED)
- S-DEMO-FIDELITY-REMEDIATION-001 v2.44 (UNCHANGED)
- S-DEMO-PRISMQL-ONBOARDING-001-B v2.20 (UNCHANGED)
- S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 v1.29 (UNCHANGED)
- S-PRISMQL-CASE-INSENSITIVE-001 v1.54 (UNCHANGED)
- error-taxonomy v2.35 (UNCHANGED)
- BC-2.16.002 v2.08 (UNCHANGED)

---

## Convergence Assessment

**Trajectory:** LOCAL 19 passes on frozen 35117a38 (3-CLEAN D-1631) → PR-LEVEL pass 1 on frozen dacb60fa: 3 findings (0/0/2/0/1/0) [NOT CLEAN] → same-burst fix pushed @39c8b134 (streak reset) → **PR-LEVEL pass 2 on frozen 39c8b134: 0 findings (CLEAN; streak 1/3)** → **PR-LEVEL pass 3 on frozen 39c8b134: 3 findings (0/0/0/1/2/0) [NOT CLEAN; streak RESET 0/3]** → same-burst fix pushed @8610ecd0 → **PR-LEVEL pass 4 on frozen 8610ecd0: 1 finding (0/0/1/0/0/0) [NOT CLEAN; streak stays 0/3]** → same-burst spec-only closure (HEAD UNCHANGED) → **PR-LEVEL pass 5 on frozen 8610ecd0: 3 findings (0/0/3/0/0/0) [NOT CLEAN; streak stays 0/3]** → same-burst spec-only closure (HEAD UNCHANGED) → **PR-LEVEL pass 6 on frozen 8610ecd0: 0 findings (CLEAN(strict); streak 0/3 → 1/3)**

**Novelty:** LOW — Pass-6 is a rotation-of-angles pass after five rounds of spec/story prose-citation corrections. All 10 probe sets returned empty-handed. The D-1637 dual-surface class-closure sweep was comprehensive; no residual dual-surface gaps found. The injection-safety and code-behavior surfaces remain fully clean across all passes (zero CRIT/HIGH code-behavior defects in the entire PR-LEVEL cascade). Decay signature: 3→0→3→1→3→[0].

**Pattern:** Zero findings on all categories. All prior spec/story citation surfaces verified clean on pass-6 rotation. Code-correctness, injection-safety, and test-integrity surfaces have been clean since pass-1. The cascade is converging normally via spec-citation cleanup across passes 3–5.

**Streak status:** 1/3 — CLEAN(strict)=YES on frozen 8610ecd0 (first CLEAN(strict) on this HEAD after the D-1635 push). **PR HEAD 8610ecd0 UNCHANGED.** Per BC-5.39.001 and DRIFT-ORCH-PRLEVEL-PUSH-001, next pass re-gates on the same 8610ecd0. **NEXT: PR-LEVEL adversary pass 7 on SAME frozen HEAD 8610ecd0** (streak candidate 2/3; NO push before pass 7).

---

## Standing Probe Results

**SAP-1 (Structured Event Catalog — BC-2.16.002):** PASS — `rg 'event_type\s*=' crates/ --type rust` finds no new `event_type` values in PR-LEVEL pass-6 scope; BC-2.16.002 v2.08 catalog complete and unchanged.

**SAP-2:** N/A — No sensor TOML spec modifications in this cascade.

**TD-VSDD-059 (paper-fix detection):** PASS — No fixes in this pass (CLEAN pass). Prior fix closures (D-1632 through D-1637) re-confirmed load-bearing from rotation-of-angles probe (emission-path locks, structural AST corrections, AC sub-arm split).

**TD-VSDD-060 (sibling-site sweep):** PASS — No value changes in this pass. Prior sibling sweeps from D-1632/D-1635 confirmed complete; no new propagation sites identified.

**POL-22 (citation accuracy):** PASS — All 14 BC-2.11.016 §Preconditions.2 positions + all 14 §Implementation location table cells + all BC-2.11.004 §Error Cases corrections verified against `ast.rs`. No residual citation inaccuracies found.

**POL-4 (story AC correctness):** PASS — S-DEMO-FIDELITY-REMEDIATION-001 v2.44 AC-M2 two-sub-arm description verified against `engine.rs` code paths. No residual "both cases recurse" claim.

**BC-5.39.001 (3-CLEAN streak):** 1/3 — pass-6 result CLEAN(strict)=YES on frozen 8610ecd0. Streak advances 0/3 → 1/3. Next cascade gate on same frozen 8610ecd0.
