---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: FIX-IEQ-ERRPATH-001
passes: [13]
feature_head_at_review: 13db1a54
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

# PR-LEVEL Adversary Pass 13 — FIX-IEQ-ERRPATH-001

---

## Pass 13 (frozen 13db1a54; fresh-context adversary; PR-LEVEL cascade; streak candidate 2/3 — ADVANCING — 1/3 → 2/3)

**Pass result:** CLEAN(strict)=YES, CLEAN(PR-merge)=YES

**Findings:** 0 total (0 CRIT / 0 HIGH / 0 MED / 0 LOW / 0 OBS / 0 PROCESS-GAP)

**SAP-1:** PASS — `rg 'event_type\s*=' crates/ --type rust` returned no new values; BC-2.16.002 v2.08 catalog row 177 complete, 3-site schema parity (column field: `sanitize_for_log` annotation present; audit role: column_not_found.rejected; recurrence: per_rejected_reference). Unchanged.

**STREAK:** ADVANCING 1/3 → 2/3 — CLEAN(strict) on frozen 13db1a54. No push before pass 13 per DRIFT-ORCH-PRLEVEL-PUSH-001; streak valid. **Next: PR-LEVEL pass 14 on SAME frozen 13db1a54 (streak candidate 3/3 — CONVERGENCE PASS; NO push before pass 14 per DRIFT-ORCH-PRLEVEL-PUSH-001).**

**Code HEAD at review:** 13db1a54 (frozen; PR #219 OPEN base develop@f935edb6; just check 5397/5397 GREEN; non-exhaustive 89/89)

**CLEAN(strict):** YES — 0 findings of any severity

**CLEAN(PR-merge):** YES — 0 findings of CRIT + HIGH + MED severity

---

## Findings

None.

---

## Probe Summary

16 probes executed; all empty-handed.

### Probe 1 — execute_scheduled_inner gate-wiring symmetry

`execute_scheduled_inner` in `crates/prism-query/src/engine.rs` follows the identical gate-ordering path as the non-scheduled `execute_inner`: E-QUERY-037 (temporal literal position gate) → E-QUERY-038 (column-not-found gate via `check_pipe_stage_columns`) → E-QUERY-039 (subquery depth gate) → E-QUERY-011 (concurrent execution semaphore). The discriminating scheduled-path test (`test_scheduled_inner_gate_ordering`) exercises this path explicitly; it is not covered by a generic integration test that could mask gate-bypass on the scheduled branch. Scope-discipline: this PR adds no new scheduled-path logic, so no new gate-wiring is introduced. SOUND.

### Probe 2 — MCP boundary ColumnNotFound(ref d) structured-payload binding + regression guards

`PrismError::ColumnNotFound { ref d }` match arm in `prism-mcp/src/server.rs` destructures `d: ColumnNotFoundDetails` and serializes the payload to the MCP `content` field. The `d.column` field carries the sanitized column name (via `sanitize_for_log` at the `ColumnNotFoundDetails::new` chokepoint). Three `#[tracing_test::traced_test]` emission-path regression locks added in pass-1 fix-burst @39c8b134 (single-tenant, multi-tenant, binding-context paths) are load-bearing: they verify the sanitized value is emitted, not the raw user input. MCP serialization boundary: `to_json()` in `ColumnNotFoundDetails` does not re-expand the sanitized string. No new MCP payload-injection surface introduced in this cascade. SOUND.

### Probe 3 — IIN/INE generic coverage via shared extract_predicate_columns

The `extract_predicate_columns` function in `crates/prism-query/src/engine.rs` handles `IEQ`, `IIN`, and `INE` operators through shared dispatch. The column-not-found gate (`check_pipe_stage_columns`) receives the extracted column references from all three operators via the same code path. The fix is generic-by-construction — `IEQ` receives the same treatment as `IIN` and `INE` through the shared extraction and gate logic. No IEQ-specific special-casing exists that would leave IIN/INE unprotected or vice versa. Verified by inspecting `extract_predicate_columns` dispatch arms for all three operator variants. SOUND.

### Probe 4 — Independent DTU→OcsfEnumMap→IIN G3 chain re-verification (pass-12 closure stability)

Pass-12 established that the G3 chain (CrowdStrike DTU → OcsfEnumMap → IIN lowering) is structurally sound. This pass independently re-verifies the terminal link: `OcsfEnumMap` `status_id[1001]` → `"New"` canonical label; IIN operator lowers both stored value (`"New"` → `"new"`) and IN-list operands (`('new', 'in progress')` → `{'new', 'in progress'}`); `"new"` ∈ `{'new', 'in progress'}` → TRUE. No new enum_map entries have been added to `crowdstrike.sensor.toml` in this cascade that would alter the normalization path. Verification is code-trace based (no live demo run available in context). SOUND; consistent with pass-12 findings.

### Probe 5 — 14-position ⇔ check_pipe_stage_columns arm coverage

`check_pipe_stage_columns` in `crates/prism-query/src/engine.rs` covers 14 structural positions of column references within a PrismQL query pipeline (bare SELECT, qualified SELECT, WHERE predicate, GROUP BY, ORDER BY, JOIN condition, subquery outer scope, etc.). Each position arm is exercised by a corresponding Red Gate test in the `DRIFT-IEQ` test suite. The pass-13 probe verifies that no new pipeline stage has been introduced in the current PR that adds a 15th structural position outside the 14-position walk. Diff surface: `t13-preflight-audit.py` only (no changes to `engine.rs` or related position-walk code). SOUND.

### Probe 6 — A–F audit-section demo-consequence axis

`t13-preflight-audit.py` is partitioned into audit sections A through F, each covering a distinct demo-consequence axis:
- **A** — Sensor connectivity and authentication
- **B** — Schema column availability
- **C** — OCSF normalization fidelity
- **D** — PrismQL operator correctness (IEQ/IIN/INE case-insensitivity)
- **E** — Error taxonomy compliance (E-QUERY-038 payload)
- **F** — Timing and concurrency guards

Each section's PASS/FAIL gate is load-bearing for `demo_ready: YES`. Section G (G1–G8) covers IIN/IEQ behavioral demonstration checks and was the subject of the pass-11 G3 fix. Scope-discipline: pass-13 introduced no new A–F section; existing sections unmodified since pass-11 fix @13db1a54. SOUND.

### Probe 7 — SEC-002 docstring empty-client_id nuance

`SEC-002` in `t13-preflight-audit.py` is documented as a guard against Levenshtein amplification with extremely long column names (CWE-407). The code comment notes that an empty `client_id` string in a multi-tenant context would skip the Levenshtein compute path entirely (returning immediately from `compute_did_you_mean` at the `available_columns.is_empty()` early-exit guard). This is NOT a defect — it is the documented fast-path behavior when no columns are available to compare against. The docstring accurately describes the actual guard, not a hypothetical. Probe confirmed: NOT a defect.

### Probe 8 — CWE-1007 bidi scope

CWE-1007 (Insufficient Visual Distinction of Homoglyphs) applies to UI rendering contexts where visually similar characters (e.g., Latin `a` vs Cyrillic `а`) could deceive a human reader. In this codebase, column name comparison uses byte-level equality after `sanitize_for_log` (which strips control characters but does not normalize homoglyphs). The spec does NOT require homoglyph normalization: `BC-2.11.016 §Postconditions` defines injection safety in terms of CWE-117 control characters, not CWE-1007 homoglyphs. Homoglyph normalization would be a scope extension requiring a new BC and product-owner approval. Confirmed out-of-contract by design. NOT a defect.

### Probe 9 — BC-2.16.002 row-177 field-schema parity

BC-2.16.002 v2.08 §Postconditions Canonical Structured Event Catalog row 177 (`column_not_found.rejected`) declares the field schema: `sensor_id`, `column` (sanitized via `sanitize_for_log`), `available_columns`, `did_you_mean`, `event_type`. The emission site in `prism-core/src/error.rs` `ColumnNotFoundDetails::new` matches this schema: all 5 fields are populated at construction time, `column` is sanitized at the chokepoint. Three emission-path tracing locks (single-tenant, multi-tenant, binding-context) verify the schema at runtime. Row 177 parity: CONFIRMED SOUND.

### Probe 10 — POL-16/12 clean

**POL-16 (BC lifecycle):** No BCs are in `draft` status for this PR; `BC-2.11.016` is `active` v1.25 (promoted post-merge of the feature branch that introduced it). No new BCs authored in this cascade. CLEAN.

**POL-12 (commit attribution):** No AI attribution in commit messages for the PR branch or the factory-artifacts commits in this cascade. CLEAN.

### Probe 11 — SAP-1: Structured Event Catalog completeness

`rg 'event_type\s*=' crates/ --type rust` — no new `event_type` values in any commit since pass-12 (no code commits to the PR branch after @13db1a54). BC-2.16.002 v2.08 catalog complete and unchanged. SAP-1 PASS.

### Probe 12 — SAP-2: Sensor TOML spec modifications

N/A — no sensor TOML spec modifications in this cascade or in the pass-11 fix (@13db1a54 diff is Python-only). SAP-2 not applicable.

### Probe 13 — TD-VSDD-059 paper-fix detection

No fixes in this pass (CLEAN). Prior pass-11 fix (@13db1a54) was structural: query target changed from `cyberint_alerts` → `crowdstrike_detections`, PASS branch assertion added, comment rewritten with behavioral anchors, COVERAGE_MATRIX row updated. Not a doc-comment rename or assert-only paper fix. PASS.

### Probe 14 — TD-VSDD-060 sibling-site sweep

No fixes in this pass. Pass-11 fix touched one site (G3 in `t13-preflight-audit.py`). Sibling sweep was performed in pass-12 (G2 and G6 re-verified SOUND). No new sibling sites introduced. PASS.

### Probe 15 — POL-24 byte-verbatim E-QUERY-038

`E-QUERY-038` error code string in `t13-preflight-audit.py` is unchanged since pass-8 spec-only closure. Byte-verbatim match against `error-taxonomy.md` v2.36 confirmed in pass-12. No code changes since pass-12. PASS.

### Probe 16 — BC-5.39.001 streak computation integrity

Pass history on frozen HEAD 13db1a54:
- Pass 11 (frozen ddf852bc): NOT CLEAN(strict) 1 HIGH → fix pushed @13db1a54 → streak RESET 0/3
- Pass 12 (frozen 13db1a54): CLEAN(strict) → streak 0/3 → 1/3
- Pass 13 (frozen 13db1a54, this pass): CLEAN(strict) → streak 1/3 → 2/3

Streak computation: 2 consecutive CLEAN(strict) passes on the same frozen HEAD 13db1a54 (no pushes between passes 12 and 13; DRIFT-ORCH-PRLEVEL-PUSH-001 satisfied). Streak is VALID at 2/3. One more CLEAN(strict) pass on the same frozen HEAD required for convergence. SOUND.

---

## Version Summary

**No spec/story version changes this pass.** Pass-13 is a CLEAN pass with zero findings. All spec and story versions carry forward from D-1644:
- BC-2.11.016 v1.25 (UNCHANGED)
- BC-2.16.002 v2.08 (UNCHANGED)
- error-taxonomy v2.36 (UNCHANGED)
- S-PRISMQL-CASE-INSENSITIVE-001 v1.55 (UNCHANGED)
- BC-INDEX v7.77 (UNCHANGED)
- STORY-INDEX v2.650 (UNCHANGED)

---

## Convergence Assessment

**Trajectory:** LOCAL 19 passes on frozen 35117a38 (3-CLEAN D-1631) → PR-LEVEL pass 1 on frozen dacb60fa: 3 findings (0/0/2/0/1/0) [NOT CLEAN] → same-burst fix pushed @39c8b134 (streak reset) → **PR-LEVEL pass 2 on frozen 39c8b134: 0 findings (CLEAN; streak 1/3)** → **PR-LEVEL pass 3 on frozen 39c8b134: 3 findings (0/0/0/1/2/0) [NOT CLEAN; streak RESET 0/3]** → same-burst fix pushed @8610ecd0 → **PR-LEVEL pass 4 on frozen 8610ecd0: 1 finding (0/0/1/0/0/0) [NOT CLEAN; streak stays 0/3]** → same-burst spec-only closure (HEAD UNCHANGED) → **PR-LEVEL pass 5 on frozen 8610ecd0: 3 findings (0/0/3/0/0/0) [NOT CLEAN; streak stays 0/3]** → same-burst spec-only closure (HEAD UNCHANGED) → **PR-LEVEL pass 6 on frozen 8610ecd0: 0 findings (CLEAN(strict); streak 0/3 → 1/3)** → **PR-LEVEL pass 7 on frozen 8610ecd0: 1 finding (0/0/1/0/0/0) [NOT CLEAN(strict); streak RESET 1/3 → 0/3]** → same-burst fix pushed @ddf852bc → **PR-LEVEL pass 8 on frozen ddf852bc: 1 finding (0/0/0/1/0/0) [NOT CLEAN(strict); streak stays 0/3]** → same-burst spec-only closure (HEAD ddf852bc UNCHANGED) → **PR-LEVEL pass 9 on frozen ddf852bc: 0 findings (CLEAN(strict); streak 0/3 → 1/3)** → **PR-LEVEL pass 10 on frozen ddf852bc: 0 findings (CLEAN(strict); streak 1/3 → 2/3)** → **PR-LEVEL pass 11 on frozen ddf852bc: 1 finding (0/1/0/0/0/0) [NOT CLEAN(strict); streak RESET 2/3 → 0/3]** → same-burst fix pushed @13db1a54 → **PR-LEVEL pass 12 on frozen 13db1a54: 0 findings (CLEAN(strict); streak 0/3 → 1/3)** → **PR-LEVEL pass 13 on frozen 13db1a54: 0 findings (CLEAN(strict); streak 1/3 → 2/3)**

**Decay signature:** 3→0→3→1→3→0→1→1→0→0→1(high)→0→[0]. Pass-13 confirms continued CLEAN status on frozen 13db1a54 with zero findings across all severity levels.

**Novelty:** NEAR-ZERO — pass-13 probes explore orthogonal fresh angles (scheduled-path gate symmetry, MCP boundary payload binding, IIN/INE generic construction) and re-verify prior pass closure stability (G3 chain, 14-position arm coverage, SEC-002/CWE-1007 nuances). No novel finding axis surfaced; all probe targets returned empty-handed.

**Pattern:** The Rust code and spec/logic surfaces remain clean (zero CRIT/HIGH code-behavior defects across the entire PR-LEVEL cascade). All findings in this cascade are audit-script findings (pass-7: inert assertion A6; pass-8: stale spec pins; pass-11: impossible operand values G3). Passes 12 and 13 together confirm the pass-11 G3 fix is structurally stable and the audit-script surface is durable.

**Streak status:** 2/3 — ADVANCING. CLEAN(strict) on frozen 13db1a54. HEAD UNCHANGED. Per BC-5.39.001 and DRIFT-ORCH-PRLEVEL-PUSH-001, streak is valid. **NEXT: PR-LEVEL adversary pass 14 on SAME frozen HEAD 13db1a54** (streak candidate 3/3 — CONVERGENCE PASS; NO push before pass 14 per DRIFT-ORCH-PRLEVEL-PUSH-001).

---

## Standing Probe Results

**SAP-1 (Structured Event Catalog — BC-2.16.002):** PASS — `rg 'event_type\s*=' crates/ --type rust` finds no new `event_type` values; BC-2.16.002 v2.08 catalog row 177 complete, 3-site schema parity confirmed. Unchanged since pass-12.

**SAP-2:** N/A — No sensor TOML spec modifications in this cascade.

**TD-VSDD-059 (paper-fix detection):** PASS — No fixes in this pass (CLEAN). Prior pass-11 fix verified structural (query redirect + assertion added). No paper-fix pattern.

**TD-VSDD-060 (sibling-site sweep):** PASS — No fixes in this pass. G3 single-site; G2 re-verified SOUND. No uncovered callsites.

**BC-5.39.001 (3-CLEAN streak):** 2/3 — ADVANCING. CLEAN(strict) on frozen 13db1a54. Next pass is streak candidate 3/3 on same frozen HEAD.
