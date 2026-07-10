---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: FIX-IEQ-ERRPATH-001
passes: [12]
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
streak_after: 1/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 12 — FIX-IEQ-ERRPATH-001

---

## Pass 12 (frozen 13db1a54; fresh-context adversary; PR-LEVEL cascade; streak candidate 1/3 — ADVANCING — 0/3 → 1/3)

**Pass result:** CLEAN(strict)=YES, CLEAN(PR-merge)=YES

**Findings:** 0 total (0 CRIT / 0 HIGH / 0 MED / 0 LOW / 0 OBS / 0 PROCESS-GAP)

**SAP-1:** PASS — `rg 'event_type\s*=' crates/ --type rust` returned no new values; BC-2.16.002 v2.08 catalog complete and unchanged.

**STREAK:** ADVANCING 0/3 → 1/3 — CLEAN(strict) on frozen 13db1a54. No push before pass 12 per DRIFT-ORCH-PRLEVEL-PUSH-001; streak valid. **Next: PR-LEVEL pass 13 on SAME frozen 13db1a54 (streak candidate 2/3; NO push before pass 13 per DRIFT-ORCH-PRLEVEL-PUSH-001).**

**Code HEAD at review:** 13db1a54 (frozen; PR #219 OPEN base develop@f935edb6; just check 5397/5397 GREEN; non-exhaustive 89/89)

**CLEAN(strict):** YES — 0 findings of any severity

**CLEAN(PR-merge):** YES — 0 findings of CRIT + HIGH + MED severity

---

## Findings

None.

---

## Probe Summary

11 probes executed; all empty-handed.

### Probe 1 — G3 redirect full-chain re-trace (pass-11 closure validation)

Pass-11 redirected G3 from `cyberint_alerts` to `crowdstrike_detections`. This pass re-traces the full code chain to confirm the fix is sound:

1. **CS DTU generator emission:** `crates/prism-dtu-crowdstrike/src/routes/detections.rs` — `make_detection_with_ioc` sets `status: "new"` unconditionally for all generated detection records. The raw value `"new"` is always emitted by the DTU generator path exercised in demo scenarios.

2. **Tombstones are device records, not detections:** CrowdStrike DTU also emits device tombstone records, but those flow through the `devices` table, not `crowdstrike_detections`. G3 queries `crowdstrike_detections`; tombstone records are unreachable via that query. The `"new"` status is exclusive to the detection row path.

3. **No enum_map in crowdstrike.sensor.toml for detections status:** `specs/sensors/crowdstrike.sensor.toml` has no `[[enum_maps]]` block that translates `status` for the detections table. The OCSF `normalize_enum_label` path applies the shared `OcsfEnumMap` instead.

4. **OcsfEnumMap status_id[1001] → "New":** The shared `OcsfEnumMap` maps OCSF `status_id` value 1001 to canonical label `"New"`. The CrowdStrike detection normalization path maps raw status `"new"` → `status_id` 1001 → canonical `"New"` (Title-case). This is the stored value.

5. **IIN lowering both sides:** IIN operator lowers both the column value and the IN-list operands before comparison. `"New"` (stored canonical) lowers to `"new"`; `('new', 'in progress')` lowers to `{'new', 'in progress'}`. `"new"` ∈ `{'new', 'in progress'}` → TRUE. The PASS branch is reachable.

6. **demo.toml org-c Stage-4 guarantees non-zero detections:** The multi-tenant demo config `demo.toml` org-c scenario Stage-4 includes CrowdStrike detections with status `"new"` for at least one detection. The G3 PASS branch is exercised on real demo data.

**Verdict:** G3 fix is structurally correct. The PASS branch is reachable with real DTU data. The full chain (DTU generator → OCSF mapping → IIN lowering) is verified end-to-end.

### Probe 2 — Title-case PASS-assertion semantics

G3 PASS branch asserts that returned rows carry Title-case canonical status values (`"New"` or `"In Progress"`). This is the correct stored canonical form after OcsfEnumMap normalization. The assertion is load-bearing: a row with lowercase `"new"` or `"in_progress"` stored would fail the assertion, catching a regression in the OcsfEnumMap normalization path. Semantics SOUND.

### Probe 3 — G2/G6 sibling spot-traces

**G2 (cyberint_alerts severity IIN):** cyberint DTU emits raw `high`/`critical` → `normalize_enum_label` → `"High"`/`"Critical"` (Title-case canonical). IIN lowers both: `"high"` ∈ `{'high', 'critical'}` → TRUE. PASS branch reachable. SOUND.

**G6 (claroty_alerts severity IIN):** claroty DTU emits raw `high`/`critical` → `normalize_enum_label` → `"High"`/`"Critical"`. Same lowering path as G2. SOUND.

Both sibling G-checks remain SOUND after the G3 redirect. No collateral effects.

### Probe 4 — POL-24 byte-verbatim E-QUERY-038

`E-QUERY-038` error text in `t13-preflight-audit.py` was verified byte-verbatim against `error-taxonomy.md` v2.36. The `did_you_mean` suffix format matches the canonical template. POL-24 PASS.

### Probe 5 — HEAD-JOIN per-reference FP-001 trigger completeness

FP-001 trigger list in `t13-preflight-audit.py` was re-traced against the per-reference suspension sites in `crates/prism-query/src/engine.rs`. Every `suspended: true` site for HEAD-JOIN qualified references has a corresponding FP-001 trigger entry. No new suspension sites added in this cascade (pass-11 fix was Python-only). Bidirectional completeness SOUND.

### Probe 6 — COVERAGE_MATRIX arithmetic (65+5=70)

`t13-preflight-audit.py` COVERAGE_MATRIX has 65 G-checks plus 5 F-checks = 70 total rows. SUMMARY block cites `70` as the total. Arithmetic correct. Updated G3 row (crowdstrike_detections status IIN) is present and correctly reflects the fix. 70/70 rows account for the declared coverage.

### Probe 7 — G3 matrix-row and comment TD-VSDD-091 compliance

G3 comment in `t13-preflight-audit.py` was verified: cites `sensor=crowdstrike_detections`, `DTU emits lowercase raw → OcsfEnumMap → Title-case canonical`, `IIN lowering → match`. No `file.rs:NNN` line-number citations. TD-VSDD-091 COMPLIANT (behavioral anchors, no volatile line pins).

G3 COVERAGE_MATRIX row text reads `crowdstrike_detections status IIN (OcsfEnumMap path)`. Accurate, non-volatile anchor. COMPLIANT.

### Probe 8 — Cyberint counterfactual

Confirmed that the cyberint status vocabulary (`open`, `acknowledged`, `closed`) has no intersection with the former G3 operands (`new`, `in progress`). The pass-11 finding conclusion stands: the original G3 was a structural false-alarm, not a test-data absence problem. No new IIN operand set could make the original G3 PASS against cyberint_alerts DTU data. Counterfactual CONFIRMED.

### Probe 9 — Sanitize chokepoint consistency

`sanitize_for_log` at `ColumnNotFoundDetails::new` chokepoint in `prism-core/src/error.rs` is unchanged since pass-1 fix-burst @39c8b134. Three production emission paths (single-tenant, multi-tenant, binding-context) verified by the `#[tracing_test::traced_test]` locks added in pass-1. No new `ColumnNotFound`-class construction sites visible in the pass-11 diff. Chokepoint discipline CONSISTENT.

### Probe 10 — SAP-1: Structured Event Catalog completeness

`rg 'event_type\s*=' crates/ --type rust` — no new `event_type` values in the pass-11 fix diff (Python-only change to `t13-preflight-audit.py`). BC-2.16.002 v2.08 catalog complete and unchanged. SAP-1 PASS.

### Probe 11 — TD-VSDD-059 + TD-VSDD-060 standing checks

**TD-VSDD-059 (paper-fix detection):** G3 fix in pass-11 was structural (query target changed from cyberint_alerts → crowdstrike_detections; PASS branch assertion added). Not a doc-comment rename. The PASS branch now exercises the OcsfEnumMap→IIN path end-to-end. No paper-fix pattern detected. PASS.

**TD-VSDD-060 (sibling-site sweep):** G3 is a single function body in `t13-preflight-audit.py`. G2 was the only sibling with a related pattern (both use IIN on cyberint/crowdstrike columns); G2 was re-verified SOUND in pass-11. COVERAGE_MATRIX has one G3 row (updated). No uncovered callsites. PASS.

---

## Version Summary

**No spec/story version changes this pass.** Pass-12 is a CLEAN pass with zero findings. All spec and story versions carry forward from D-1643:
- BC-2.11.016 v1.25 (UNCHANGED)
- BC-2.16.002 v2.08 (UNCHANGED)
- error-taxonomy v2.36 (UNCHANGED)
- S-PRISMQL-CASE-INSENSITIVE-001 v1.55 (UNCHANGED)
- BC-INDEX v7.77 (UNCHANGED)
- STORY-INDEX v2.650 (UNCHANGED)

---

## Convergence Assessment

**Trajectory:** LOCAL 19 passes on frozen 35117a38 (3-CLEAN D-1631) → PR-LEVEL pass 1 on frozen dacb60fa: 3 findings (0/0/2/0/1/0) [NOT CLEAN] → same-burst fix pushed @39c8b134 (streak reset) → **PR-LEVEL pass 2 on frozen 39c8b134: 0 findings (CLEAN; streak 1/3)** → **PR-LEVEL pass 3 on frozen 39c8b134: 3 findings (0/0/0/1/2/0) [NOT CLEAN; streak RESET 0/3]** → same-burst fix pushed @8610ecd0 → **PR-LEVEL pass 4 on frozen 8610ecd0: 1 finding (0/0/1/0/0/0) [NOT CLEAN; streak stays 0/3]** → same-burst spec-only closure (HEAD UNCHANGED) → **PR-LEVEL pass 5 on frozen 8610ecd0: 3 findings (0/0/3/0/0/0) [NOT CLEAN; streak stays 0/3]** → same-burst spec-only closure (HEAD UNCHANGED) → **PR-LEVEL pass 6 on frozen 8610ecd0: 0 findings (CLEAN(strict); streak 0/3 → 1/3)** → **PR-LEVEL pass 7 on frozen 8610ecd0: 1 finding (0/0/1/0/0/0) [NOT CLEAN(strict); streak RESET 1/3 → 0/3]** → same-burst fix pushed @ddf852bc → **PR-LEVEL pass 8 on frozen ddf852bc: 1 finding (0/0/0/1/0/0) [NOT CLEAN(strict); streak stays 0/3]** → same-burst spec-only closure (HEAD ddf852bc UNCHANGED) → **PR-LEVEL pass 9 on frozen ddf852bc: 0 findings (CLEAN(strict); streak 0/3 → 1/3)** → **PR-LEVEL pass 10 on frozen ddf852bc: 0 findings (CLEAN(strict); streak 1/3 → 2/3)** → **PR-LEVEL pass 11 on frozen ddf852bc: 1 finding (0/1/0/0/0/0) [NOT CLEAN(strict); streak RESET 2/3 → 0/3]** → same-burst fix pushed @13db1a54 → **PR-LEVEL pass 12 on frozen 13db1a54: 0 findings (CLEAN(strict); streak 0/3 → 1/3)**

**Decay signature:** 3→0→3→1→3→0→1→1→0→0→1(high)→0. Pass-12 is CLEAN with zero findings across all severity levels.

**Novelty:** LOW — pass-12 probes are validation sweeps of the pass-11 fix (G3 redirect re-trace, sibling soundness, POL-24 verbatim, FP-001 trigger completeness, COVERAGE_MATRIX arithmetic). No novel finding axis required; all probe targets were pre-identified from the pass-11 finding.

**Pattern:** The Rust code and spec/logic surfaces remain clean (zero CRIT/HIGH code-behavior defects across the entire PR-LEVEL cascade). The three findings in this cascade are all audit-script findings (pass-7: inert assertion A6; pass-8: stale spec pins; pass-11: impossible operand values G3). Pass-12 confirms the pass-11 G3 fix is structurally sound and the audit-script surface is now stable.

**Streak status:** 1/3 — ADVANCING. CLEAN(strict) on frozen 13db1a54. HEAD UNCHANGED. Per BC-5.39.001 and DRIFT-ORCH-PRLEVEL-PUSH-001, streak is valid. **NEXT: PR-LEVEL adversary pass 13 on SAME frozen HEAD 13db1a54** (streak candidate 2/3; NO push before pass 13 per DRIFT-ORCH-PRLEVEL-PUSH-001).

---

## Standing Probe Results

**SAP-1 (Structured Event Catalog — BC-2.16.002):** PASS — `rg 'event_type\s*=' crates/ --type rust` finds no new `event_type` values in this cascade step; BC-2.16.002 v2.08 catalog complete and unchanged.

**SAP-2:** N/A — No sensor TOML spec modifications in this cascade.

**TD-VSDD-059 (paper-fix detection):** PASS — No fixes in this pass (CLEAN). Prior pass-11 fix verified structural (query redirect + assertion added). No paper-fix pattern.

**TD-VSDD-060 (sibling-site sweep):** PASS — No fixes in this pass. G3 single-site; G2 re-verified SOUND. No uncovered callsites.

**BC-5.39.001 (3-CLEAN streak):** 1/3 — ADVANCING. CLEAN(strict) on frozen 13db1a54. Next pass is streak candidate 2/3 on same frozen HEAD.
