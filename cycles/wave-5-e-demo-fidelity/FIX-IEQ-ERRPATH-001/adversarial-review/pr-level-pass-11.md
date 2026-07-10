---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: FIX-IEQ-ERRPATH-001
passes: [11]
feature_head_at_review: ddf852bc
date: 2026-07-09
clean_strict: false
clean_pr_merge: false
finding_counts:
  total: 1
  crit: 0
  high: 1
  med: 0
  low: 0
  obs: 0
  process_gap: 0
code_behavior_defects: 0
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 11 — FIX-IEQ-ERRPATH-001

---

## Pass 11 (frozen ddf852bc; fresh-context adversary; PR-LEVEL cascade; streak candidate 3/3 — NOT ADVANCING — RESET 2/3 → 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO

**Findings:** 1 total (0 CRIT / 1 HIGH / 0 MED / 0 LOW / 0 OBS / 0 PROCESS-GAP)

**SAP-1:** PASS — `rg 'event_type\s*=' crates/ --type rust` returned no new values; BC-2.16.002 v2.08 catalog complete and unchanged.

**STREAK:** RESET 2/3 → 0/3 — NOT CLEAN(strict) on frozen ddf852bc (1 HIGH finding). Per BC-5.39.001, streak resets to 0/3. Same-burst fix pushed @13db1a54. Per DRIFT-ORCH-PRLEVEL-PUSH-001, streak restarts on new frozen HEAD 13db1a54. **Next: PR-LEVEL pass 12 on SAME frozen 13db1a54 (streak candidate 1/3; NO push before pass 12 per DRIFT-ORCH-PRLEVEL-PUSH-001).**

**Code HEAD at review:** ddf852bc (frozen; PR #219 OPEN base develop@f935edb6; just check 5397/5397 GREEN; non-exhaustive 89/89)

**Code HEAD after fix-burst:** 13db1a54 (pushed; streak restarts on new frozen HEAD; py_compile clean; just check GREEN; non-exhaustive 89/89)

**CLEAN(strict):** NO — 1 HIGH finding

**CLEAN(PR-merge):** NO — 1 HIGH finding (>= MED severity)

---

## Findings

### ADV-PR-P11-HIGH-001 — G3 IIN assertion uses impossible cyberint status values (impossible-assertion / false-alarm demo-blocker)

**Severity:** HIGH
**Confidence:** HIGH
**Novelty:** Genuinely novel — first pass to trace G-check assertions against ACTUAL DTU generator emissions via code-trace chain (DTU source types.rs → TOML spec → OCSF enum mapping → IIN lowering)

**Finding:** `scripts/t13-preflight-audit.py` check G3 ("IIN case-insensitive on status") asserted `status IIN ('new','in progress')` against `cyberint_alerts`. This assertion is IMPOSSIBLE to satisfy:

1. **cyberint DTU generator emits only `{open, acknowledged, closed}`.** Reading `crates/prism-dtu-cyberint/src/types.rs` (alert status field) and `crates/prism-dtu-cyberint/src/routes/alerts.rs` (normalization path): the three raw values emitted are `open`, `acknowledged`, and `closed`. Neither `new` nor `in progress` appears in the cyberint DTU source.

2. **No OcsfEnumMap for cyberint status.** `specs/sensors/cyberint.sensor.toml` has no `[[enum_maps]]` block for the `status` column. The OCSF `normalize_enum_label` function is a pass-through when no mapping is defined — it lowercases but does NOT translate `open`→`new` or `acknowledged`→`in progress`. The canonical stored values remain `open`, `acknowledged`, `closed`.

3. **IIN lowering cannot bridge the gap.** IIN lowers both operands before comparison (`open` IIN `('new','in progress')` → `open` in `{'new','in_progress'}` → FALSE). No amount of case-folding converts `open` to `new` or `in_progress`. The PASS branch of G3 can NEVER be reached against a real cyberint_alerts response.

4. **Demo-blocking consequence.** G3 was introduced in the same PR-level cascade pass that hardened WARN→FAIL semantics (D-1632 same-burst). With WARN→FAIL active, G3 emits FAIL on every cyberint_alerts IIN run. `demo_ready` gate outputs `DEMO-READY: NO`. This is a false-alarm authored in this PR that would block the multi-client SOC demo on a cyberint status assertion that can never pass — the check author confused cyberint status vocabulary with CrowdStrike detection status vocabulary.

**Root cause:** G3 was written with CrowdStrike detection status values (`new`, `in progress`) applied to a cyberint_alerts query. The two sensors have entirely different status vocabularies. The error was masked until this pass because prior passes did not trace G-check assertion operands back to the actual DTU generator outputs.

**Status:** CLOSED (same-burst fix @13db1a54)

---

## Closure — ADV-PR-P11-HIGH-001

**Implementer:** @13db1a54 (same-burst fix; pushed to PR #219)

**Fix applied — Option B (redirect G3 to crowdstrike_detections):**

G3 was redirected from `cyberint_alerts` to `crowdstrike_detections` with corrected status values. The fix:

1. **G3 query changed to `crowdstrike_detections`:** CrowdStrike detection `status` field maps through OcsfEnumMap (`new` → OCSF canonical `"New"`; `in_progress` → `"In Progress"`). The canonical stored values are Title-case (`"New"`, `"In Progress"`). IIN lowers both sides (`"New"` IIN `('new','in progress')` → `'new'` in `{'new','in progress'}` → TRUE). G3 now exercises a genuine case-folding scenario — both the IIN operator AND the OCSF enum_map normalization path are exercised in a single check.

2. **PASS branch assertions updated:** The PASS branch now asserts that the returned rows carry Title-case canonical status values (`"New"` or `"In Progress"`), confirming the OcsfEnumMap→IIN path is end-to-end correct.

3. **Comment rewritten with behavioral anchors:** G3 comment now cites: sensor=crowdstrike_detections; DTU emits lowercase raw → OcsfEnumMap → Title-case canonical; IIN lowering → match. No file/line-number citations (TD-VSDD-091 compliant).

4. **COVERAGE_MATRIX row text updated:** G3 row updated from `cyberint_alerts status IIN` to `crowdstrike_detections status IIN (OcsfEnumMap path)`.

**Sibling sweep — G2 verified SOUND:**

G2 (`cyberint_alerts severity IIN ('high','critical')`) was re-verified: cyberint DTU emits raw `high` and `critical` → OCSF `normalize_enum_label` maps to `"High"` and `"Critical"` (canonical Title-case) → IIN lowers both sides → `'high'` in `{'high','critical'}` → TRUE. G2 is SOUND.

**Full per-check value-set-vs-generator verdict table (G1–G8 + F1–F6):**

| Check | Sensor | Column | DTU emits | OCSF mapping | IIN/IEQ lowers | SOUND? |
|-------|--------|--------|-----------|--------------|----------------|--------|
| G1 | crowdstrike_detections | tactic | `"Credential Access"` (Title-case) | pass-through | lowers → `"credential access"` IEQ `'credential access'` | SOUND |
| G2 | cyberint_alerts | severity | `high`/`critical` → `"High"`/`"Critical"` | normalize_enum_label | `"high"` IIN `('high','critical')` | SOUND |
| G3 | crowdstrike_detections | status | `new`/`in_progress` → `"New"`/`"In Progress"` | OcsfEnumMap | `"new"` IIN `('new','in progress')` | SOUND (fixed @13db1a54) |
| G4 | claroty_alerts | status | `open`/`closed` → `"Active"`/`"Closed"` | OcsfEnumMap canonical | IIN lowering | SOUND |
| G5 | armis_devices | category | `"IT"` (pass-through) | pass-through | IEQ lowers | SOUND |
| G6 | claroty_alerts | severity | `high`/`critical` → `"High"`/`"Critical"` | normalize_enum_label | IIN lowers | SOUND |
| G7 | crowdstrike_detections | tactic INE | `"Credential Access"` | pass-through | lowers → not `'lateral movement'` | SOUND |
| G8 | cyberint_alerts | status IEQ | `open` → `"open"` (pass-through) | no mapping | `"open"` IEQ `'open'` | SOUND |
| F1–F6 | all sensors | various | per-sensor DTU types.rs | per-TOML spec | standard | SOUND |

**Verification level:** code-trace chain (DTU generator types.rs → TOML spec enum_maps → OCSF normalize_enum_label → IIN lowering). Live demo run not possible in context (DTU clones not running). py_compile clean. `just check` pre-push GREEN; non-exhaustive 89/89.

---

## Probe Summary

### Probe 1 — G-check value-set-vs-generator trace (novel probe; first pass to execute this sweep)

Systematic trace of every G-check assertion operand back to the actual DTU generator source for that sensor's column:

For each G-check: (a) identify the sensor and column from the check body; (b) read the corresponding DTU `crates/prism-dtu-<sensor>/src/types.rs` for the field's raw values; (c) read the TOML `specs/sensors/<sensor>.sensor.toml` for any `[[enum_maps]]` or `normalize_enum_label` wiring; (d) trace through OCSF canonical storage; (e) verify IIN/IEQ operand matches the canonical stored values after lowering.

**G3 finding:** cyberint DTU types.rs alert status: raw values `open`, `acknowledged`, `closed`. cyberint.sensor.toml: no enum_map for status column. normalize_enum_label: pass-through (no mapping → lowercase). Canonical stored: `open`, `acknowledged`, `closed`. G3 asserts `IIN ('new','in progress')` → NO intersection with `{open, acknowledged, closed}` → FALSE on every real row → FAIL emitted → DEMO-READY: NO (false alarm).

**All other G-checks (G1, G2, G4–G8):** traced and verified SOUND per verdict table above.

### Probe 2 — SAP-1: Structured Event Catalog completeness

`rg 'event_type\s*=' crates/ --type rust` — no new `event_type` values in PR-LEVEL pass-11 scope. BC-2.16.002 v2.08 catalog complete and unchanged. **SAP-1 PASS.**

### Probe 3 — TD-VSDD-059 (paper-fix detection on G3 closure)

G3 fix verified load-bearing: PASS branch now asserts Title-case canonical values (`"New"` or `"In Progress"`), confirming the OcsfEnumMap→IIN path is end-to-end correct. The fix is structural (query target changed + assertion added), not a doc-comment rename. **TD-VSDD-059 PASS.**

### Probe 4 — TD-VSDD-060 (sibling-site sweep on G3 changes)

G2 re-verified SOUND (cyberint severity path unaffected). COVERAGE_MATRIX row updated (single location). Comment behavioral anchors added (no callsite propagation). **TD-VSDD-060 PASS.**

---

## Version Summary

**No spec/story version changes this pass.** Pass-11 finding is confined to `scripts/t13-preflight-audit.py` (Python audit script). All spec and story versions carry forward from D-1642:
- BC-2.11.016 v1.25 (UNCHANGED)
- BC-2.16.002 v2.08 (UNCHANGED)
- error-taxonomy v2.36 (UNCHANGED)
- S-PRISMQL-CASE-INSENSITIVE-001 v1.55 (UNCHANGED)
- BC-INDEX v7.77 (UNCHANGED)
- STORY-INDEX v2.650 (UNCHANGED)

---

## Convergence Assessment

**Trajectory:** LOCAL 19 passes on frozen 35117a38 (3-CLEAN D-1631) → PR-LEVEL pass 1 on frozen dacb60fa: 3 findings (0/0/2/0/1/0) [NOT CLEAN] → same-burst fix pushed @39c8b134 (streak reset) → **PR-LEVEL pass 2 on frozen 39c8b134: 0 findings (CLEAN; streak 1/3)** → **PR-LEVEL pass 3 on frozen 39c8b134: 3 findings (0/0/0/1/2/0) [NOT CLEAN; streak RESET 0/3]** → same-burst fix pushed @8610ecd0 → **PR-LEVEL pass 4 on frozen 8610ecd0: 1 finding (0/0/1/0/0/0) [NOT CLEAN; streak stays 0/3]** → same-burst spec-only closure (HEAD UNCHANGED) → **PR-LEVEL pass 5 on frozen 8610ecd0: 3 findings (0/0/3/0/0/0) [NOT CLEAN; streak stays 0/3]** → same-burst spec-only closure (HEAD UNCHANGED) → **PR-LEVEL pass 6 on frozen 8610ecd0: 0 findings (CLEAN(strict); streak 0/3 → 1/3)** → **PR-LEVEL pass 7 on frozen 8610ecd0: 1 finding (0/0/1/0/0/0) [NOT CLEAN(strict); streak RESET 1/3 → 0/3]** → same-burst fix pushed @ddf852bc → **PR-LEVEL pass 8 on frozen ddf852bc: 1 finding (0/0/0/1/0/0) [NOT CLEAN(strict); streak stays 0/3]** → same-burst spec-only closure (HEAD ddf852bc UNCHANGED) → **PR-LEVEL pass 9 on frozen ddf852bc: 0 findings (CLEAN(strict); streak 0/3 → 1/3)** → **PR-LEVEL pass 10 on frozen ddf852bc: 0 findings (CLEAN(strict); streak 1/3 → 2/3)** → **PR-LEVEL pass 11 on frozen ddf852bc: 1 finding (0/1/0/0/0/0) [NOT CLEAN(strict); streak RESET 2/3 → 0/3]** → same-burst fix pushed @13db1a54

**Decay signature:** 3→0→3→1→3→0→1→1→0→0→1(high). Pass-11 finding is a genuinely novel axis (G-check value-set-vs-generator trace); no prior pass deployed this probe. ADV-PR-P11-HIGH-001 is the first HIGH severity finding in the PR-LEVEL cascade, and the first G-check defect.

**Novelty:** HIGH — first pass to trace G-check assertion operands back to DTU generator emissions. All prior passes focused on Rust code correctness, spec citations, and Python audit script assertion soundness. The G-check trace probe category is new and uncovered a demo-blocking false-alarm.

**Pattern:** The Rust code and spec/logic surfaces remain clean (zero CRIT/HIGH code-behavior defects in the entire PR-LEVEL cascade across all 11 passes). The finding is confined to the audit script surface (Python preflight check). This represents a third distinct audit-script finding class (pass-7: inert assertion A6; pass-8: stale spec pins; pass-11: impossible operand values in G3).

**Streak status:** 0/3 — RESET by pass-11 finding. Fix pushed @13db1a54 (new frozen HEAD). Per DRIFT-ORCH-PRLEVEL-PUSH-001, streak restarts on new frozen HEAD 13db1a54. **NEXT: PR-LEVEL adversary pass 12 on SAME frozen HEAD 13db1a54** (streak candidate 1/3; NO push before pass 12 per DRIFT-ORCH-PRLEVEL-PUSH-001).

---

## Standing Probe Results

**SAP-1 (Structured Event Catalog — BC-2.16.002):** PASS — `rg 'event_type\s*=' crates/ --type rust` finds no new `event_type` values in PR-LEVEL pass-11 scope; BC-2.16.002 v2.08 catalog complete and unchanged.

**SAP-2:** N/A — No sensor TOML spec modifications in this cascade.

**TD-VSDD-059 (paper-fix detection):** PASS — G3 fix is structural (query target redirected + assertion added); not a doc-comment rename. PASS branch asserts canonical Title-case values, confirming OcsfEnumMap→IIN end-to-end correctness.

**TD-VSDD-060 (sibling-site sweep):** PASS — G2 re-verified SOUND (cyberint severity path). COVERAGE_MATRIX row updated at single location. No callsite propagation needed.

**BC-5.39.001 (3-CLEAN streak):** 0/3 — RESET by pass-11 finding (1 HIGH). Fix-burst pushed @13db1a54 (new frozen HEAD). Next pass re-gates on 13db1a54 (streak candidate 1/3).
