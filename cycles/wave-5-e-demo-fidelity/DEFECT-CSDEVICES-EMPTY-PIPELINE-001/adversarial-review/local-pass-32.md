---
document_type: adversarial-review
scope: LOCAL
fix_defect: DEFECT-CSDEVICES-EMPTY-PIPELINE-001
passes: [32]
feature_head_at_review: 072930ee
date: 2026-07-11
clean_strict: false
clean_pr_merge: true
finding_counts:
  total: 3
  crit: 0
  high: 0
  med: 0
  low: 3
  obs: 0
  process_gap: 0
code_behavior_defects: 0
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: adversary
---

# LOCAL Adversary Pass 32 — DEFECT-CSDEVICES-EMPTY-PIPELINE-001

---

## Pass 32 (frozen 072930ee; fresh-context adversary; LOCAL cascade; policy rubric + SAP-1 + SAP-2 + POL-22 + POL-24 byte-strict + POL-33 table verification; streak candidate 1/3 — STREAK REMAINS 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=YES

**Findings:** 3 total (0 CRIT / 0 HIGH / 0 MED / 3 LOW / 0 OBS / 0 PROCESS-GAP)

**Adversary novelty assessment:** LOW-MEDIUM — all three findings are documentation-accuracy defects in the non-exhaustive gate tooling layer: a docstring contradiction on the E0004 note shape for the v90/v91 VirtualField types, a stale entry count in the Python script module comment, and a stale CI comment combined with an incorrect CLAUDE.md authority pointer.

**SAP-1:** PASS — `rg 'event_type\s*=' crates/ --type rust` across the full codebase: 232 emission sites cataloged. New `pre_register` call sites use bare `debug!` macros without `event_type` key — D-765 precedent applies (bare debug emissions without `event_type` do not require BC-2.16.002 catalog rows). No new `event_type =` emission sites introduced without catalog rows.

**SAP-2 (devices):** PASS — 6/6 TOML-declared device columns verified present in DTU generator and fixture. Excess-field gap correctly deferred to DRIFT-SAP2-DEVICES-TOML-SURFACE-001.

**SAP-2 (detections):** PASS — `detection_detail()` 12-field shape verified at frozen 072930ee; `severity` emits string labels matching TOML `column_type = "string"` and BC-2.16.013 v1.31; `det_index` parsed from `detection_id` trailing NNN via `rsplit('-')` (stable mapping, batch-position `.enumerate()` removed); `device_id` computed as `generate_host_ids(org_slug, seed)[det_index % HOST_COUNT]` (host-pool constraint satisfied; harness-mode JOIN non-empty verified).

**POL-22:** PASS — all cited entities resolve; no dangling cross-references.

**POL-24 (byte-strict):** PASS — E-QUERY-041/042/043 hint strings verified byte-exact against error-taxonomy v2.39; no drift.

**POL-33:** PASS — BC-2.16.013 v1.31 9-row Route Coverage Table re-verified at frozen 072930ee; all 9 routes confirmed present.

**Non-exhaustive gate (EXPECTED=91):** PASS — all 91 compile-fail tests verified GREEN on frozen HEAD 072930ee via two-layer check (`scripts/check-non-exhaustive.sh` count gate + `scripts/check-non-exhaustive-per-symbol.py` per-symbol gate). Two-segment Layer-2 keys (`virtual_fields::VirtualField` / `ast::VirtualField`) verified correctly disambiguated.

**BC-2.16.013 v1.31 clause audit:** PASS — `det_index` disambiguation clause verified (stable parse from `detection_id` trailing NNN); severity string-label clause verified (SEVERITY_LABELS cycling); `device_id` host-pool modulo verified. No regression from pass-31 fixes.

**BC-2.11.022 v1.3 gate audit:** PASS — four-assertion CI round-trip gate verified load-bearing; 4-tier ExampleKind (Standard, Custom, Negative, NegativeE043) verified in spec and code.

**Adjudicated invariants re-verified:**
- `detection_detail` `det_index` stable-mapping (parsed from `detection_id` trailing NNN, not batch position) — INTACT
- `detection_detail` `severity` string labels (`"Low"/"Medium"/"High"/"Critical"`; BC-2.16.013 v1.31) — INTACT
- Non-exhaustive gate two-layer EXPECTED=91 with 2-seg key disambiguation + v91 canary — INTACT
- E0004 note shapes empirically pinned (v90 2-part `prism_core::VirtualField` re-export; v91 3-part `prism_query::ast::VirtualField` direct) — gate passing correctly; docstring contradiction noted below (F-CSD-P32-OBS-001)
- Architect Option B Pipe/SqlPipe asymmetry rationale — INTACT
- `det_index % HOST_COUNT` modulo in `detection_detail()` — INTACT

**STREAK:** 0/3 UNCHANGED — CLEAN(strict)=NO (3 LOW findings). Streak stays 0/3.

**Code HEAD at review:** 072930ee (frozen; expected just check FULL WORKSPACE ~5481 GREEN; non-exhaustive 91/91 two-layer per-symbol ON BRANCH; LOCAL-ONLY; develop baseline UNCHANGED @b9cf3f9b)

**CLEAN(strict):** NO — 3 LOW findings
**CLEAN(PR-merge):** YES — zero CRIT/HIGH/MED findings

---

## Findings

### F-CSD-P32-OBS-001 (LOW) — Docstring contradiction on E0004 note shape for v90/v91 VirtualField types

**Severity:** LOW

**File:** `scripts/check-non-exhaustive-per-symbol.py` (worked-example comment) / `tests/external/non-exhaustive-violation/src/enum_violations.rs` (v91 entry description)

**Description:** The Python script `scripts/check-non-exhaustive-per-symbol.py` contains a worked-example in its module docstring (the `extract_e0004_symbol` illustration) that shows the expected E0004 note shape for a `match` on a non-exhaustive type. The pass-31 fix added `v91_ast_virtual_field_match` for `prism_query::ast::VirtualField` — a type emitted by rustc as a 3-part path since it is declared directly in `prism_query::ast`, not via a re-export alias. However the worked-example was not updated to reflect the 3-part note shape vs the 2-part re-export shape.

Additionally, the `enum_violations.rs` v91 entry doc-comment describes the E0004 note shape in terms that imply the v90 re-export rendering, rather than the v91 direct-crate rendering. The empirical verification in the pass-31 implementer burst (`@072930ee`) confirmed the gate passes correctly for both shapes — the contradiction is prose-only and does not affect gate execution.

**Root cause:** v90 gate (`v90_virtual_field_match`) matches `prism_core::VirtualField`, a re-exported alias — rustc E0004 notes emit a 2-part path; 2-seg Layer-2 key is `virtual_fields::VirtualField`. v91 gate (`v91_ast_virtual_field_match`) matches `prism_query::ast::VirtualField`, a direct 3-part path — rustc E0004 notes emit the 3-part path; 2-seg Layer-2 key is `ast::VirtualField`. The pass-31 fix correctly disambiguated the Layer-2 keys but left the prose description of the worked example unreflective of this distinction.

**Resolution:**
- implementer @a6f86fa3: `extract_e0004_symbol` worked-example updated to annotate the v90 re-export (2-part note → 2-seg key `virtual_fields::VirtualField`) vs v91 direct (3-part note → 2-seg key `ast::VirtualField`) distinction; canary diff comment added to the Layer-2 failure path (on mismatch, failure now prints full sorted actually-extracted symbol set to stderr for diagnostics); TD-VSDD-060 sweep found no other stale description sites.

**Closure:** @a6f86fa3 (implementer)

---

### F-CSD-P32-OBS-002 (LOW) — Python script module comment claims "90 entries" (truth: 91)

**Severity:** LOW

**File:** `scripts/check-non-exhaustive-per-symbol.py`

**Description:** The Python script `scripts/check-non-exhaustive-per-symbol.py` contains a module-level prose comment stating the expected symbol count as "90 entries" (or equivalent). The pass-31 implementer burst (`@072930ee`) added `v91_ast_virtual_field_match` to `EXPECTED_SYMBOLS` and updated `EXPECTED_COUNT = 91`, but the module header prose comment was not updated to match. This creates a silent count discrepancy between human-readable prose and the authoritative `EXPECTED_COUNT` constant.

**Resolution:**
- implementer @a6f86fa3: Module header comment updated "90 entries" → "91 entries"; now aligned with `EXPECTED_COUNT = 91` constant and the actual `EXPECTED_SYMBOLS` list length.

**Closure:** @a6f86fa3 (implementer)

---

### F-CSD-P32-OBS-003 (LOW) — ci.yml comment ">= 90" stale; CLAUDE.md authority pointer misidentifies ci.yml as EXPECTED owner

**Severity:** LOW

**File:** `.github/workflows/ci.yml` (comment) / `CLAUDE.md` fix-branch copy (convention sentence)

**Description:** Two related but distinct issues:

1. **ci.yml:** The non-exhaustive check step (which now calls `bash scripts/check-non-exhaustive.sh`) contains a comment citing `>= 90` as the floor count. After pass-28 (89→90) and pass-31 (90→91) gate bumps, the comment still reads `>= 90` — stale by one count.

2. **CLAUDE.md (fix-branch copy):** The convention sentence describing the non-exhaustive gate states `ci.yml EXPECTED=NN` is the authority for `EXPECTED`. This is incorrect: the pass-29 fix replaced the inline 106-line ci.yml non-exhaustive step with a single call to `bash scripts/check-non-exhaustive.sh`, making `scripts/check-non-exhaustive.sh` (and `scripts/check-non-exhaustive-per-symbol.py`) the authoritative home for `EXPECTED`. ci.yml now simply invokes the script; the script owns the count. The CLAUDE.md sentence should point authority at `scripts/check-non-exhaustive.sh` (called by ci.yml), not at ci.yml directly.

**Resolution:**
- implementer @a6f86fa3: ci.yml comment updated from `>= 90` to `>= EXPECTED (count owned by scripts/check-non-exhaustive.sh)`; CLAUDE.md fix-branch convention sentence corrected to point authority at `scripts/check-non-exhaustive.sh` instead of `ci.yml EXPECTED=NN`; TD-VSDD-060 sweep found no other stale `>= 90` references or misattributed authority-pointer sites.

**Closure:** @a6f86fa3 (implementer)

---

## Verification Summary

The adversary conducted a full-policy pass over frozen HEAD 072930ee (fix/csdevices-empty-pipeline; LOCAL-ONLY). The following verification work was completed:

1. **SAP-1 catalog sweep** — `rg 'event_type\s*=' crates/ --type rust` full codebase; 232 sites cataloged; new `pre_register` sites use bare `debug!` without `event_type` key (D-765 precedent: bare emissions exempt from catalog registration requirement); no new `event_type =` sites without catalog rows.

2. **SAP-2 TOML↔DTU parity (devices)** — 6/6 TOML-declared columns verified. Excess-field gap correctly deferred to DRIFT-SAP2-DEVICES-TOML-SURFACE-001.

3. **SAP-2 TOML↔DTU parity (detections)** — `detection_detail()` 12-field shape verified; `severity` string-label type verified (BC-2.16.013 v1.31 clause exact); `det_index` stable-mapping from `detection_id` trailing NNN verified; `device_id` host-pool modulo verified. No type regressions from pass-31 fix.

4. **Non-exhaustive gate EXPECTED=91** — two-layer check: `scripts/check-non-exhaustive.sh` count gate PASS (91/91); `scripts/check-non-exhaustive-per-symbol.py` per-symbol gate PASS (all 91 symbols recognized); 2-seg Layer-2 keys `virtual_fields::VirtualField` / `ast::VirtualField` correctly disambiguated.

5. **Gate tooling documentation audit** — read `scripts/check-non-exhaustive-per-symbol.py` module docstring; found `extract_e0004_symbol` worked-example describing only the v90 re-export note shape, not the v91 direct-path shape; module header comment citing "90 entries" stale after pass-31 v91 addition → F-CSD-P32-OBS-001 + F-CSD-P32-OBS-002.

6. **ci.yml and CLAUDE.md authority audit** — read ci.yml non-exhaustive step; comment cites `>= 90` (stale, now 91); read CLAUDE.md fix-branch convention sentence; authority pointer references `ci.yml EXPECTED=NN` as authoritative — incorrect since pass-29 moved EXPECTED ownership into `scripts/check-non-exhaustive.sh` → F-CSD-P32-OBS-003.

7. **Empirical E0004 note shape re-verification** — v90 `prism_core::VirtualField` is a re-export alias; rustc emits 2-part E0004 note → 2-seg key `virtual_fields::VirtualField` CORRECT. v91 `prism_query::ast::VirtualField` is a direct 3-part path; rustc emits 3-part E0004 note → 2-seg key `ast::VirtualField` CORRECT. Gate passing correctly at frozen 072930ee; discrepancy is prose-only.

8. **Adjudicated invariants re-verified** — all pass-31 closures (det_index stable-mapping, severity string, VirtualField gates, Layer-2 key disambiguation) verified INTACT at frozen 072930ee.

9. **POL-24 byte-strict** — E-QUERY-041/042/043 hint strings verified byte-exact against error-taxonomy v2.39; no drift.

10. **POL-33 Route Coverage Table** — BC-2.16.013 v1.31 9-row table re-verified at frozen 072930ee; all 9 routes confirmed present.

11. **BC-2.11.022 v1.3 gate audit** — four-assertion CI round-trip gate verified load-bearing; 4-tier ExampleKind (Standard, Custom, Negative, NegativeE043) verified in spec and code.

---

## Fix Record

**Fix-burst commit: @a6f86fa3** (implementer) — F-CSD-P32-OBS-001 + F-CSD-P32-OBS-002 + F-CSD-P32-OBS-003:
- `scripts/check-non-exhaustive-per-symbol.py`: `extract_e0004_symbol` worked-example updated to annotate v90 re-export (2-part note → `virtual_fields::VirtualField`) vs v91 direct-path (3-part note → `ast::VirtualField`) distinction; Layer-2 failure path canary added (failure now prints full sorted actually-extracted symbol set to stderr on mismatch); module header comment "90 entries" → "91 entries".
- `.github/workflows/ci.yml`: non-exhaustive step comment updated `>= 90` → `>= EXPECTED (count owned by scripts/check-non-exhaustive.sh)`.
- `CLAUDE.md` (fix-branch): convention sentence authority pointer corrected from `ci.yml EXPECTED=NN` to `scripts/check-non-exhaustive.sh` (called by ci.yml).
- TD-VSDD-060 sweep: no other stale `>= 90` or misattributed authority-pointer sites found.
- Verification: `scripts/check-non-exhaustive.sh` 91/91 both layers PASS; `just check-fast` clean (docs-only change; no code logic altered).

**New FROZEN HEAD for pass 33:** a6f86fa3 (LOCAL-ONLY, NOT pushed to origin — flag priority push after 3-CLEAN). Streak 0/3. Cascade now 32 passes (docs-only fix over 072930ee). Develop baseline UNCHANGED @b9cf3f9b.

---

## Streak Status

| Pass | Frozen HEAD | CLEAN(strict) | Streak |
|------|-------------|---------------|--------|
| 27 | 9fe2d016 | YES | 1/3 |
| 28 | 9fe2d016 | NO (1 OBS) | 0/3 RESET |
| 29 | 25b80a81 | NO (5 MED + 1 OBS + 1 PROCESS-GAP) | 0/3 |
| 30 | 7a6f6caa | NO (2 LOW + 2 OBS) | 0/3 |
| 31 | ed2988cc | NO (1 MED + 1 LOW + 1 OBS) | 0/3 |
| 32 (this pass) | 072930ee | NO (3 LOW) | **0/3** |

Pass 33 NEXT on NEW frozen HEAD a6f86fa3. Streak 0/3. If CLEAN(strict), streak advances to 1/3.
