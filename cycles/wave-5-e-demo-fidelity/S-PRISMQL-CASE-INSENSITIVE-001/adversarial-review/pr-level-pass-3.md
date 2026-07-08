---
document_type: adversarial-review
scope: PR-LEVEL
passes: [3]
story: S-PRISMQL-CASE-INSENSITIVE-001
pr: 217
feature_head_at_review: dcb37099
base_develop_head: 7b1f6c51
closure_head: fab7df00
date: 2026-07-08
clean_strict: false
clean_pr_merge: true
finding_counts:
  OBS: 2
  total: 2
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay from adversary pass-3 output
---
# PR-LEVEL Adversarial Review — Pass 3
## S-PRISMQL-CASE-INSENSITIVE-001

**Frozen HEAD:** dcb37099 (feature/S-PRISMQL-CASE-INSENSITIVE-001)
**Base:** develop@7b1f6c51
**Date:** 2026-07-08
**Authored by:** orchestrator-relay from adversary pass-3 output

---

## Verdict

| Criterion | Result |
|-----------|--------|
| CLEAN (strict) | **no** |
| CLEAN (PR-merge) | **yes** |

**Finding summary:** 2 OBS, 0 CRIT, 0 HIGH, 0 MED, 0 LOW, 0 PROCESS-GAP.

**Novelty:** LOW — same documentation-accuracy and metadata-treadmill classes as prior OBS findings. No new defect classes introduced.

**Streak status:** 0/3 — this pass is not CLEAN(strict); push of fab7df00 resets streak per DRIFT-ORCH-PRLEVEL-PUSH-001 regardless.

---

## Findings

### ADV-PR-P3-OBS-001 — RG-080 covered `value` field only; `sensor_type` order-symmetry untested

**Severity:** OBS
**Category:** BC-2.16.002 v2.05 row 91 — `ocsf.enum_label_unrecognized` warn event field symmetry
**File:** `crates/prism-bin/src/spec_driven_adapter.rs`
**Test:** `test_rg080_low001_build_column_array_enum_label_warn_order_of_operations`

**Description:** RG-080 (the pass-1 ADV-PR-P1-LOW-001 closure) asserted the sanitize-before-truncate order-of-operations for the `value` field in the `ocsf.enum_label_unrecognized` warn event at the PRIMARY site (`build_column_array`). However, the BC-2.16.002 v2.05 catalog row 91 specifies that BOTH `value` and `sensor_type` must be sanitized (CWE-117) before the 50-codepoint truncation cap. RG-080 did not verify the `sensor_type` field's sanitize-before-truncate order, leaving a gap in the row-91 symmetry guarantee.

No behavioral regression — the production code was correct at both fields. The defect was a coverage asymmetry in the existing load-bearing test.

**Anchor:** `test_rg080_low001_build_column_array_enum_label_warn_order_of_operations` — asserted `value` field order only; `sensor_type` mirror assertions absent.

**Closure note:** @fab7df00 (implementer, single commit). RG-080 extended in place: `WarnFieldVisitor`/`WarnFieldCapture` types renamed to support multi-field capture; 65-codepoint ESC `"B"` control-character vector injected into `sensor_id` (feeds `sensor_type` label); 3 mirrored `sensor_type` assertions added confirming sanitize-before-truncate for both fields. `just check` 5317/5317 GREEN; non-exhaustive 89/89. Test is extended, not added — RGT count unchanged at 81.

Story updated: story-writer amended RG-080 row description in the Red Gate Test table (v1.34→v1.35; RGT count 81 unchanged; no BC version changes).

---

### ADV-PR-P3-OBS-002 — Evidence report HEAD SHA `f9be96fa` stale vs frozen `dcb37099`

**Severity:** OBS
**Category:** Metadata treadmill — evidence report cites an older feature HEAD after subsequent docs-only commits
**File:** `docs/demo-evidence/S-PRISMQL-CASE-INSENSITIVE-001/evidence-report.md`

**Description:** The evidence report cited `f9be96fa` as the code-behavior HEAD. After the pass-1 fix-burst committed behavioral code at `f9be96fa`, two subsequent commits landed: `dcb37099` (pass-2 docstring scrub, docs-only) and the frozen HEAD for this pass-3 review. The SHA citation in the evidence report therefore lagged behind the actual frozen HEAD by two non-behavioral commits.

This is a metadata-treadmill class finding: the code behavior being evidenced has not changed since `f9be96fa`, but the literal HEAD SHA reference was stale. No correctness impact; the docs-only nature of intervening commits means the evidence is valid for `dcb37099` as well.

**Anchor:** `docs/demo-evidence/S-PRISMQL-CASE-INSENSITIVE-001/evidence-report.md` — SHA citation `f9be96fa`.

**Closure note:** @fab7df00 (implementer). Evidence report updated with durable provenance phrasing: "Code-behavior HEAD: f9be96fa (ADV-PR-P1-MED-001/LOW-001/OBS-002 fix-burst); subsequent commits non-behavioral: dcb37099 (docstring scrub), fab7df00 (RG-080 sensor_type mirror). Evidence valid for all three." This phrasing is immune to future docs-only commit staleness because it explicitly enumerates non-behavioral subsequent commits rather than claiming a single HEAD.

---

## Probe Results

### SAP-1 — Tracing emission catalog completeness

**Result: CLEAN**

No new `event_type =` sites introduced in this pass. Row-91 `ocsf.enum_label_unrecognized` sites (PRIMARY: `crates/prism-spec-engine/src/spec_driven_adapter.rs`; SECONDARY: `crates/prism-ocsf/src/normalizer.rs`) re-verified at frozen dcb37099: both carry `event_type = "ocsf.enum_label_unrecognized"` and both match BC-2.16.002 §Postconditions catalog row 91. No new catalog rows required.

### SAP-2 — DTU↔TOML schema parity

**Result: N/A** — this story does not modify `.prism/specs/sensors/*.toml` or DTU clone route/type files.

### POL-22 — Phase A+C gates

**Result: CLEAN** — Phase A (story frontmatter completeness) and Phase C (BC traceability) both verified clean.

### CWE-117 — Log injection order at PRIMARY+SECONDARY

**Result: CLEAN both sites** — RG-079 (SECONDARY load-bearing helper test) and RG-080 (PRIMARY order-of-operations vector test, now extended with sensor_type mirror) both GREEN at frozen dcb37099. Pass-1 fix-burst @f9be96fa reordered all 5 warn sites; confirmed at this pass-3 review. RG-080 closure @fab7df00 extends the symmetry guarantee to both `value` and `sensor_type` fields per BC-2.16.002 v2.05 row 91.

### Paper-fix audit

**Result: none detected** — ADV-PR-P3-OBS-001 closure @fab7df00 is load-bearing (test extended with three new field-level assertions, not merely renamed or doc-commented). No paper-fix pattern.

---

## Convergence Trajectory (PR-LEVEL)

| Pass | Frozen HEAD | CLEAN(strict) | CLEAN(PR-merge) | Findings | Streak |
|------|------------|---------------|-----------------|----------|--------|
| 1    | a2fc8940   | no            | no              | 2 MED + 2 LOW + 2 OBS (total 6) | 0/3 reset |
| 2    | 1172b15a   | no            | yes             | 1 LOW (total 1)                 | 0/3 (push resets) |
| 3    | dcb37099   | no            | yes             | 2 OBS (total 2)                 | 0/3 (push resets) |

---

## Post-Pass Action

Implementer @fab7df00 closed ADV-PR-P3-OBS-001 (RG-080 sensor_type mirror extension) and ADV-PR-P3-OBS-002 (evidence report durable provenance). `just check` 5317/5317 GREEN; non-exhaustive 89/89. Story v1.34→v1.35 (RGT count 81 unchanged).

**VERY NEXT ACTION:** Push feature HEAD fab7df00 to origin → new frozen HEAD → PR-LEVEL adversary pass-4 on new frozen HEAD. Per DRIFT-ORCH-PRLEVEL-PUSH-001, the push resets the streak to 0/3. If passes 4, 5, 6 are all CLEAN(strict) on unchanged HEAD → 3-CLEAN CONVERGED → pr-manager squash-merge + post-merge burst (POL-14: BC-2.11.024 + BC-2.02.013 draft→active).
