---
document_type: adversarial-review
scope: LOCAL
fix_defect: DEFECT-CSDEVICES-EMPTY-PIPELINE-001
passes: [30]
feature_head_at_review: 7a6f6caa
date: 2026-07-11
clean_strict: false
clean_pr_merge: true
finding_counts:
  total: 4
  crit: 0
  high: 0
  med: 0
  low: 2
  obs: 2
  process_gap: 0
code_behavior_defects: 1
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: adversary
---

# LOCAL Adversary Pass 30 — DEFECT-CSDEVICES-EMPTY-PIPELINE-001

---

## Pass 30 (frozen 7a6f6caa; fresh-context adversary; LOCAL cascade; policy rubric + SAP-1 + SAP-2 + POL-22 + POL-24 byte-strict + POL-33 table verification; streak candidate 1/3 — STREAK REMAINS 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=YES

**Findings:** 4 total (0 CRIT / 0 HIGH / 0 MED / 2 LOW / 2 OBS / 0 PROCESS-GAP)

**Adversary novelty assessment:** LOW — fresh-context sweep of newly-introduced per-symbol Python script files (pass-29 added `scripts/check-non-exhaustive-per-symbol.py`); doc-comment enumeration gaps in that script; OBS-003 is the same failure class as the original defect (silent-0-row via mismatched JOIN key).

**SAP-1:** PASS — `rg 'event_type\s*=' crates/ --type rust` across all files changed relative to develop@b9cf3f9b: five SQL emission sites verified (pipe.sql_lowering × 3, pipe.sql_planning_error × 2); all five catalog rows present in BC-2.16.002 §Postconditions (v2.10); no new emission sites introduced without catalog rows.

**SAP-2 (devices):** PASS — 6/6 TOML-declared device columns verified present in DTU generator and fixture. Excess-field gap correctly deferred to DRIFT-SAP2-DEVICES-TOML-SURFACE-001.

**SAP-2 (detections — post-pass-29-fix):** PASS on field completeness — `detection_detail()` handler now exposes all 12 TOML-declared detections columns as required by BC-2.16.013 v1.29. OBS-003 finding below covers a further constraint violation (device_id value correctness vs host pool membership).

**POL-22:** PASS — All cited entities resolve; no dangling cross-references.

**POL-24 (byte-strict):** PASS — E-QUERY-041/042/043 hint strings verified byte-exact against error-taxonomy v2.39 and production code; no drift.

**POL-33:** PASS on existing route coverage table (BC-2.16.013 v1.28 9-row table re-verified at frozen 7a6f6caa).

**Per-symbol gate design audit:** PASS — adversary verified the two-layer `check-non-exhaustive.sh` + `check-non-exhaustive-per-symbol.py` design: Layer 1 (count gate, ci.yml) feeds Layer 2 (per-symbol Python parse); both layers fail-closed; ci.yml replacement with `bash scripts/check-non-exhaustive.sh` is correct (python3 present in CI environment); E0639-via-spans-text + E0004-via-defined-here-note parsing is correct and covers both error classes.

**BC-2.11.022 v1.3 gate audit:** PASS — four-assertion CI round-trip gate structure verified load-bearing: (1) NegativeE043 example present in `build_reference_content` output; (2) behavioral gate fires E-QUERY-043 for InSubquery in projection position; (3) NegativeE040 gate still fires; (4) new `test_bc_2_11_022_ci_4tier_gate` exhaustiveness-stub arm added for NegativeE043.

**detection_detail() 12-field audit (BC-2.16.013 v1.29):** PASS on field count and schema — all 12 TOML-declared detections columns present at correct paths per BC-2.16.013 v1.29 clause. OBS-003 covers device_id value constraint (below).

**Non-exhaustive gate (EXPECTED=90):** PASS — all 90 gates verified GREEN on frozen HEAD 7a6f6caa.

**STREAK:** 0/3 UNCHANGED — CLEAN(strict)=NO (2 LOW + 2 OBS findings). Streak stays 0/3.

**Code HEAD at review:** 7a6f6caa (frozen; just check FULL WORKSPACE 5477/5477 GREEN, 60 skipped; non-exhaustive 90/90 two-layer per-symbol; LOCAL-ONLY; develop baseline UNCHANGED @b9cf3f9b)

**CLEAN(strict):** NO — 2 LOW + 2 OBS findings
**CLEAN(PR-merge):** YES — zero CRIT/HIGH/MED findings

---

## Findings

### F-CSD-P30-OBS-001 (LOW) — `enum_violations.rs` module doc-comment enumeration omitted v79; claimed 21, truth is 22

**Severity:** LOW

**File:** `scripts/check-non-exhaustive-per-symbol.py` (or `tests/external/non-exhaustive-violation/src/enum_violations.rs`)

**Description:** The `enum_violations.rs` section of the per-symbol Python script (or its module doc-comment) enumerates the expected E0004-type `#[non_exhaustive]` enum violations used as test anchors and claims 21 entries. The actual count is 22 — one entry (v79 or the most recently added `VirtualField` E0004 gate added in pass-28) was omitted from the enumeration comment/list. The per-symbol Python script's `len==90` import guard is correct (90 total symbols), but the module-doc enumeration of the E0004 subset is stale by one entry.

**Risk:** Documentation accuracy gap in a newly-introduced script. A future auditor counting the doc-enumeration against actual symbols would encounter a discrepancy and could doubt the script's completeness. No functional gate gap — the Python script's load-path guard is structurally correct.

**Resolution:**
- Implementer @3a9ec741: Rebuilt enum_violations module doc-comment enumeration from the actual function list in the file; verified 22 entries match reality; 22 + 68 = 90 cross-checked against the Python script's `len==90` guard.

**Closure:** @3a9ec741

---

### F-CSD-P30-OBS-002 (LOW) — `struct_violations.rs` doc-comment enumeration missing v73-v76 + v88; claimed 60, truth is 68

**Severity:** LOW

**File:** `scripts/check-non-exhaustive-per-symbol.py` (or `tests/external/non-exhaustive-violation/src/struct_violations.rs`)

**Description:** The `struct_violations.rs` section doc-comment enumeration claims 60 E0639-type `#[non_exhaustive]` struct violations. The actual count is 68 — entries v73-v76 and v88 (recently added struct violations from passes 18-29 cascade) were omitted from the enumeration comment. The Python script `len==90` total guard is correct (90 total), but the E0639 subset doc-comment count is stale.

**Risk:** Same as OBS-001 — documentation accuracy gap. No functional gate failure. A future engineer cross-referencing the enumeration comment against actual symbols would find the 60 ≠ 68 discrepancy.

**Resolution:**
- Implementer @3a9ec741: Rebuilt struct_violations module doc-comment enumeration from actual functions; verified 68 entries; 68 + 22 = 90 cross-checked against `len==90` guard.

**Closure:** @3a9ec741

---

### F-CSD-P30-OBS-003 (OBS → ARCHITECT ADJUDICATION Option A) — harness `detection_detail()` `device_id="placeholder-device-id"` not in host pool; harness-mode detections⋈devices JOIN silently returns 0 rows

**Severity:** OBS (adjudicated to IN-SCOPE-FIX by architect, 2026-07-11)

**File:** `crates/prism-dtu-harness/src/crowdstrike/detections.rs` (or equivalent detection detail handler)

**Description:** The `detection_detail()` handler in the `prism-dtu-harness` CrowdStrike clone was expanded in pass-29 to include 12 TOML-declared fields per BC-2.16.013 v1.29. However, the `device_id` field in the response was set to a literal placeholder string (e.g., `"placeholder-device-id"`) that does not exist in the host pool generated by `generate_host_ids(org_slug, seed)`. This means that a harness-mode query:

```sql
SELECT * FROM crowdstrike_detections
JOIN crowdstrike_devices ON crowdstrike_detections.device_id = crowdstrike_devices.device_id
```

...would silently return 0 rows — even though both tables have data — because the detection's `device_id` value does not match any `device_id` value in the devices table. This is the exact same failure class as the original DEFECT-CSDEVICES-EMPTY-PIPELINE-001 defect (silent 0-row materialization due to JOIN key mismatch).

**BC-2.16.013 v1.29 clause re-read:** The v1.29 clause requires that `device_id` be a top-level field at the correct path and that it be present. It did NOT explicitly require the value to come from the host pool. However, this constraint is implied by the invariant: a harness that produces structurally correct but JOIN-incompatible data defeats the purpose of the parity test.

**Architect adjudication (Option A, 2026-07-11):** Thread real host IDs. Key evidence: (1) call site already holds `State` with `org_slug` and `seed` — no refactor required; (2) standalone generator's `det_index % HOST_COUNT` modulo pattern is the established design (mirrors `host_detail()` precedent); (3) BC-2.16.013 clause requires key presence; Option A strengthens additively without conflicting; (4) Option B (doc-only closure, paper-fix) rejected — TD-VSDD-059 prohibits paper-fix when no concrete future dependency mandates the deferral. The silent-0-row path is not currently exercised in harness scenarios but represents a latent correctness gap that the cascade is designed to eliminate.

**BC amendment:** BC-2.16.013 v1.29→v1.30 (product-owner): `device_id` host-pool constraint clause added to INV-HARNESS-ROUTE-PARITY `detection_detail()` block. `device_id` MUST be a valid host ID from `generate_host_ids(org_slug, seed)`, computed as `generate_host_ids(org_slug, seed)[det_index % HOST_COUNT]`. Literal placeholder strings that do not appear in the harness host pool are forbidden. A harness-mode JOIN `crowdstrike_detections JOIN crowdstrike_devices ON device_id = device_id` MUST return non-empty rows when both tables have data.

**Resolution:**
- test-writer RED @c26a74ef: `test_F_CSD_P30_OBS_003_detection_device_id_is_valid_host_id_not_placeholder` (asserts `device_id` matches a value from `generate_host_ids`) + `test_F_CSD_P30_OBS_003_detection_device_ids_join_devices_nonempty` (JOIN-fidelity lock: intersection non-empty)
- implementer GREEN @ed2988cc: `detection_detail(detection_id, det_index, org_slug, seed)` — new `org_slug` + `seed` parameters; `generate_host_ids(org_slug, seed)[det_index % HOST_COUNT]` at root-level `device_id`; same modulo value at nested `device.device_id` for backward compatibility; sole call site threaded with `.enumerate()` for `det_index`; P29-006 test `test_F_CSD_P29_006_detection_detail_full_toml_field_coverage` still GREEN.
- BC-2.16.013 v1.29→v1.30 (product-owner, included in this commit)

**Closure:** @ed2988cc (implementer), BC-2.16.013 v1.30 (product-owner)

---

### F-CSD-P30-OBS-004 (OBS cosmetic) — per-symbol Python script has duplicate numeric comment labels for v70 and v86

**Severity:** OBS (cosmetic)

**File:** `scripts/check-non-exhaustive-per-symbol.py`

**Description:** The per-symbol Python script introduced in pass-29 uses sequential numeric comment labels (`# v1`, `# v2`, ...) to track which symbol each entry corresponds to. The labels v70 and v86 each appear twice — once for a struct_violations entry and once for an enum_violations entry. The duplicate labels cause confusion when cross-referencing the script against the non-exhaustive gate test files (a reader counting entries and finding two `# v70` entries would be uncertain which is which).

**Risk:** Pure documentation cosmetic. The script's functional logic (the `len==90` guard, the symbol-name list) is correct. No CI gate gap.

**Resolution:**
- Implementer @3a9ec741: Added file-type suffixes to disambiguate duplicate labels — entries in the struct_violations section use `# v70-struct` / `# v86-struct` and entries in the enum_violations section use `# v70-enum` / `# v86-enum`; all labels now unique; `len==90` guard unchanged.

**Closure:** @3a9ec741

---

## Verification Summary

The adversary conducted a full-policy pass over frozen HEAD 7a6f6caa (fix/csdevices-empty-pipeline; LOCAL-ONLY). The following verification work was completed:

1. **Per-symbol gate design audit** — read `scripts/check-non-exhaustive-per-symbol.py` + `scripts/check-non-exhaustive.sh` + ci.yml replacement; two-layer architecture confirmed (Layer 1 count gate + Layer 2 per-symbol parse); ci.yml `bash scripts/check-non-exhaustive.sh` call confirmed correct; python3 available in CI environment; E0639 + E0004 parsing logic confirmed; 0 UNKNOWN at frozen HEAD.

2. **Doc-comment enumeration audit (enum_violations.rs / struct_violations.rs)** — cross-referenced Python script module-doc enumeration against actual function count; enum_violations: 21 claimed, 22 actual (F-CSD-P30-OBS-001); struct_violations: 60 claimed, 68 actual (F-CSD-P30-OBS-002); total 90 matches `len==90` guard (correct).

3. **detection_detail() BC-2.16.013 v1.29 clause audit** — verified 12 TOML-declared detections columns present at correct paths; `device_id` top-level present; `behaviors` array with IOC keys present; `ioc_value` nullable. Clause satisfied. Additional constraint: `device_id` value was `"placeholder-device-id"`, not a real host ID → F-CSD-P30-OBS-003.

4. **SAP-1 catalog sweep** — `rg 'event_type\s*=' crates/ --type rust` across changed files; five production SQL emission sites (pipe.sql_lowering × 3, pipe.sql_planning_error × 2); all five catalog rows present in BC-2.16.002 §Postconditions (v2.10); no new emission sites.

5. **SAP-2 TOML↔DTU parity (devices)** — 6/6 TOML-declared columns verified. Excess-field gap correctly deferred to DRIFT-SAP2-DEVICES-TOML-SURFACE-001.

6. **BC-2.11.022 v1.3 gate load-bearing audit** — four CI assertions confirmed: NegativeE043 example present, behavioral gate fires E-QUERY-043, NegativeE040 gate still fires, exhaustiveness-stub arm present for NegativeE043.

7. **POL-24 byte-strict** — E-QUERY-041/042/043 hint strings verified byte-exact against error-taxonomy v2.39; no drift found.

8. **POL-33 Route Coverage Table** — BC-2.16.013 v1.28 9-row table re-verified at frozen HEAD 7a6f6caa; all 9 routes confirmed present.

9. **Non-exhaustive gate EXPECTED=90** — all 90 compile-fail tests verified GREEN.

10. **Per-symbol script duplicate label audit** — examined label sequence in Python script; found two duplicate numeric labels at v70 and v86 (one each in struct_violations and enum_violations sections) → F-CSD-P30-OBS-004.

11. **Load-bearing test verification** — T39 (Pipe wildcard boundary), T40 (SqlPipe head InSubquery lock), T41 (SqlPipe stages walk), T42 (TimestampArithmetic lock), `negative_e043_parity_gate` tests, `test_bc_2_11_022_ci_4tier_gate`, `test_F_CSD_P29_006_detection_detail_full_toml_field_coverage` — all verified GREEN at frozen HEAD 7a6f6caa.

---

## Architect Memo Summary

**F-CSD-P30-OBS-003 Option A adjudication (2026-07-11):**

Architect evaluated `detection_detail()` `device_id` placeholder string against Canonical-Principle deferral criteria.

**Evidence supporting Option A (thread real host IDs):**

1. **Call site already holds `State` with `org_slug` and `seed`** — no refactor required; the signature change is additive (new parameters passed through).
2. **Established design pattern** — `host_detail()` uses `generate_host_ids(org_slug, seed)[index % HOST_COUNT]` for device IDs. The detection-level `device_id` must match the same pool for JOIN to return non-empty rows. This is not new architecture; it is consistent application of the existing pattern.
3. **Option A strengthens BC-2.16.013 v1.29 additively** — v1.29 requires the `device_id` field at the correct path; v1.30 adds the value constraint. No conflict.
4. **Option B rejected (TD-VSDD-059 paper-fix)** — a doc-comment or BC-note closure that does not enforce the JOIN-non-empty constraint would leave the same silent-0-row hazard in place with only documentation as the guard. The Canonical Principle §Rule 4 and TD-VSDD-059 both prohibit this pattern when no concrete future dependency mandates the deferral.

**Ruling:** IN-SCOPE-FIX. `detection_detail()` receives `det_index`, `org_slug`, and `seed` as new parameters; root-level `device_id` is `generate_host_ids(org_slug, seed)[det_index % HOST_COUNT]`; nested `device.device_id` receives the same value for backward compatibility.

---

## Fix Record

**Fix-burst commits:**

1. **@3a9ec741** (implementer) — F-CSD-P30-OBS-001/002/004:
   - `scripts/check-non-exhaustive-per-symbol.py`: `enum_violations.rs` module-doc enumeration rebuilt from actual functions (22 entries); `struct_violations.rs` module-doc enumeration rebuilt from actual functions (68 entries); 22 + 68 = 90 verified against `len==90` import guard
   - Duplicate labels v70/v86: added file-type suffixes (`-struct` / `-enum`) to disambiguate; all 90 labels now unique
   - Both per-symbol and count gates verified PASS at frozen HEAD post-fix

2. **BC-2.16.013 v1.29→v1.30** (product-owner, uncommitted edits ratified in this commit) — F-CSD-P30-OBS-003 spec-note:
   - INV-HARNESS-ROUTE-PARITY `detection_detail()` clause: added `device_id` host-pool constraint after the `ioc_description` sentence
   - `device_id` MUST be a valid host ID from `generate_host_ids(org_slug, seed)`, computed as `generate_host_ids(org_slug, seed)[det_index % HOST_COUNT]`
   - Literal placeholder strings not in harness host pool are forbidden
   - Harness-mode JOIN `crowdstrike_detections JOIN crowdstrike_devices ON device_id = device_id` MUST return non-empty rows when both tables have data
   - Governs F-CSD-P30-OBS-003 (architect Option A ruling 2026-07-11)
   - Changelog row v1.30

3. **@c26a74ef** (test-writer RED) — F-CSD-P30-OBS-003:
   - `test_F_CSD_P30_OBS_003_detection_device_id_is_valid_host_id_not_placeholder`: asserts `device_id` in detection response is a member of `generate_host_ids(org_slug, seed)` (not a placeholder string)
   - `test_F_CSD_P30_OBS_003_detection_device_ids_join_devices_nonempty`: JOIN-fidelity lock — intersection of detection `device_id` values and device `device_id` values is non-empty when both tables have data

4. **@ed2988cc** (implementer GREEN) — F-CSD-P30-OBS-003:
   - `detection_detail(detection_id, det_index, org_slug, seed)`: new `org_slug` + `seed` parameters
   - Root-level `device_id`: `generate_host_ids(org_slug, seed)[det_index % HOST_COUNT]`
   - Nested `device.device_id`: same value for backward compatibility
   - Sole call site: `.enumerate()` threaded to supply `det_index`
   - P29-006 test `test_F_CSD_P29_006_detection_detail_full_toml_field_coverage` still GREEN
   - `just check` FULL WORKSPACE 5479/5479 GREEN; `prism-dtu-harness` 144/144; non-exhaustive 90/90 two-layer per-symbol

**New FROZEN HEAD for pass 31:** ed2988cc (LOCAL-ONLY). Streak 0/3. Cascade now 30 passes (commits this burst: 3a9ec741, c26a74ef, ed2988cc). Develop baseline UNCHANGED @b9cf3f9b.

---

## Streak Status

| Pass | Frozen HEAD | CLEAN(strict) | Streak |
|------|-------------|---------------|--------|
| 27 | 9fe2d016 | YES | 1/3 |
| 28 | 9fe2d016 | NO (1 OBS) | 0/3 RESET |
| 29 | 25b80a81 | NO (5 MED + 1 OBS + 1 PROCESS-GAP) | 0/3 |
| 30 (this pass) | 7a6f6caa | NO (2 LOW + 2 OBS) | **0/3** |

Pass 31 NEXT on NEW frozen HEAD ed2988cc. Streak 0/3. If CLEAN(strict), streak advances to 1/3.
