---
document_type: adversarial-review
scope: LOCAL
fix_defect: DEFECT-CSDEVICES-EMPTY-PIPELINE-001
passes: [31]
feature_head_at_review: ed2988cc
date: 2026-07-11
clean_strict: false
clean_pr_merge: false
finding_counts:
  total: 3
  crit: 0
  high: 0
  med: 1
  low: 1
  obs: 1
  process_gap: 0
code_behavior_defects: 2
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: adversary
---

# LOCAL Adversary Pass 31 — DEFECT-CSDEVICES-EMPTY-PIPELINE-001

---

## Pass 31 (frozen ed2988cc; fresh-context adversary; LOCAL cascade; policy rubric + SAP-1 + SAP-2 + POL-22 + POL-24 byte-strict + POL-33 table verification; streak candidate 1/3 — STREAK REMAINS 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO

**Findings:** 3 total (0 CRIT / 0 HIGH / 1 MED / 1 LOW / 1 OBS / 0 PROCESS-GAP)

**Adversary novelty assessment:** LOW-MEDIUM — MED-001 is a type-parity violation (SAP-2 severity INTEGER vs. string) that persisted through P29-006 test rationalization; OBS-001 is a stable-mapping invariant gap that was introduced when `det_index` was threaded at P30; OBS-002 is a pre-existing compile-fail gate gap for `prism_query::ast::VirtualField` (companion to the `prism_core::virtual_fields::VirtualField` gate added in P28) combined with a Layer-2 key-collision regression window.

**SAP-1:** PASS — `rg 'event_type\s*=' crates/ --type rust` across all files changed relative to develop@b9cf3f9b: five SQL emission sites verified (pipe.sql_lowering × 3, pipe.sql_planning_error × 2); all five catalog rows present in BC-2.16.002 §Postconditions (v2.10); no new emission sites introduced without catalog rows.

**SAP-2 (devices):** PASS — 6/6 TOML-declared device columns verified present in DTU generator and fixture. Excess-field gap correctly deferred to DRIFT-SAP2-DEVICES-TOML-SURFACE-001.

**SAP-2 (detections — type parity):** FAIL → F-CSD-P31-MED-001. `crowdstrike.sensor.toml` declares `severity` column with `column_type = "string"`. Standalone DTU generator emits string labels (`"Low"`, `"Medium"`, `"High"`, `"Critical"`). Harness `detection_detail()` at frozen HEAD ed2988cc emits `severity` as INTEGER `50` (numeric). The P29-006 test comment rationalized this divergence as "det_index % 4 maps to integer representation" — surface-and-defer anti-pattern under production-grade rules. Silent "50" rendering in a String Arrow column at query runtime.

**POL-22:** PASS — all cited entities resolve; no dangling cross-references.

**POL-24 (byte-strict):** PASS — E-QUERY-041/042/043 hint strings verified byte-exact against error-taxonomy v2.39; no drift.

**POL-33:** PASS on existing route coverage table (BC-2.16.013 v1.30 9-row table re-verified at frozen ed2988cc).

**Non-exhaustive gate (EXPECTED=90):** PASS — all 90 compile-fail tests verified GREEN on frozen HEAD ed2988cc via two-layer check (count gate scripts/check-non-exhaustive.sh + per-symbol scripts/check-non-exhaustive-per-symbol.py).

**BC-2.16.013 v1.30 clause audit:** PASS on host-pool constraint — `device_id` now correctly computed as `generate_host_ids(org_slug, seed)[det_index % HOST_COUNT]`. OBS-001 finding (below) covers `det_index` value stability across request batch shapes.

**BC-2.11.022 v1.3 gate audit:** PASS — four-assertion CI round-trip gate structure verified load-bearing.

**STREAK:** 0/3 UNCHANGED — CLEAN(strict)=NO (1 MED + 1 LOW + 1 OBS findings). Streak stays 0/3.

**Code HEAD at review:** ed2988cc (frozen; just check FULL WORKSPACE 5479/5479 GREEN; prism-dtu-harness 144/144; non-exhaustive 90/90 two-layer per-symbol; LOCAL-ONLY; develop baseline UNCHANGED @b9cf3f9b)

**CLEAN(strict):** NO — 1 MED + 1 LOW + 1 OBS findings
**CLEAN(PR-merge):** NO — 1 MED finding present

---

## Findings

### F-CSD-P31-MED-001 (MED) — SAP-2 type parity: harness `detection_detail()` emits `severity` as INTEGER 50; TOML declares `column_type = "string"`; standalone DTU emits string labels

**Severity:** MED

**File:** `crates/prism-dtu-harness/src/crowdstrike/detections.rs` (or equivalent detection detail handler)

**Description:** `crowdstrike.sensor.toml` declares:

```toml
[[tables.detections.columns]]
name = "severity"
column_type = "string"
```

The standalone `prism-dtu-crowdstrike` generator emits `severity` as a string label matching the TOML contract: `"Low"`, `"Medium"`, `"High"`, or `"Critical"`. The harness `detection_detail()` at frozen HEAD ed2988cc emits `severity` as the raw integer `50` (a legacy placeholder from before the P29 expansion to 12 fields). When the query engine materializes this response into a `String`-typed Arrow column, the numeric value renders as the string `"50"` — a silent behavioral divergence that violates SAP-2 type parity without any error or warning.

**Additional context — P29-006 test rationalization removed:** The test `test_F_CSD_P29_006_detection_detail_full_toml_field_coverage` contained a comment rationalizing `is_number()` assertion for `severity` as "intentional representation". This is a surface-and-defer anti-pattern (Canonical Principle §Rule 4 + TD-VSDD-059 paper-fix prohibition). The comment and the `is_number()` assertion both need to be corrected to `is_string()` matching the TOML contract and standalone generator behavior.

**BC-2.16.013 v1.30 clause re-read:** The v1.30 clause specifies `detection_detail()` must include `severity` per the `crowdstrike.sensor.toml` detections column list. The clause did not yet specify the type contract for `severity`. This finding drives a v1.31 BC amendment to add an explicit string-type enforcement clause parallel to the existing field-presence requirement.

**BC amendment:** BC-2.16.013 v1.30→v1.31 (product-owner): INV-HARNESS-ROUTE-PARITY `detection_detail()` clause extended — `severity` MUST be a string label matching the standalone generator's emission types (`"Low"` / `"Medium"` / `"High"` / `"Critical"`, cycling on `SEVERITY_LABELS[det_index % 4]`); numeric severity values (e.g., `1`, `2`, `3`, `4`, `50`) are forbidden. Additionally, `det_index` semantic disambiguation: `det_index` MUST be the canonical detection index parsed from the `detection_id` trailing integer (`det-{org_slug}-{seed}-{NNN}` → NNN, parsed via `rsplit('-')` on the last segment); batch-position-derived indices (from `.enumerate()` at the call site) produce a non-stable mapping across request subsets and are forbidden.

**Resolution:**
- test-writer RED @36f0ba9c: `test_F_CSD_P29_006_detection_detail_full_toml_field_coverage` — `is_number()` assertion on `severity` FLIPPED to `is_string()`; rationalization comment deleted; test now asserts string value matching standalone generator labels. New test `test_F_CSD_P31_MED_001_detection_severity_is_string_label_matching_standalone_dtu` explicitly verifies `severity` is one of `["Low", "Medium", "High", "Critical"]`.
- implementer GREEN @072930ee: `severity` changed to `SEVERITY_LABELS[det_index % 4]` where `SEVERITY_LABELS = ["Low", "Medium", "High", "Critical"]`; `det_index` computed INSIDE `detection_detail()` by parsing the trailing integer from `detection_id` via `rsplit('-')` (handles org_slug hyphens; deterministic fallback 0 on malformed detection_id, with comment); `det_index` parameter removed from function signature; `.enumerate()` call removed from sole call site (TD-VSDD-060 sweep confirmed single call site).
- BC-2.16.013 v1.30→v1.31 (product-owner, uncommitted edits ratified in this commit)

**Closure:** @072930ee (implementer), BC-2.16.013 v1.31 (product-owner)

---

### F-CSD-P31-OBS-001 (LOW) — `det_index` was batch enumeration position; same `detection_id` maps to DIFFERENT `device_id` values across request subsets (non-stable mapping)

**Severity:** LOW

**File:** `crates/prism-dtu-harness/src/crowdstrike/detections.rs`

**Description:** The P30 fix threaded `det_index` from `.enumerate()` at the call site. This means `det_index` is the POSITION of a detection within the current batch request's response slice, not a stable property of the detection itself. Consider:

- Full-set request returns detections `[det-org-seed-005, det-org-seed-010, det-org-seed-017]` → `det_index` values `[0, 1, 2]` → `device_id` values `[host[0], host[1], host[2]]`
- Subset request returning only `[det-org-seed-010]` → `det_index=0` → `device_id = host[0]`

The same `detection_id` `det-org-seed-010` maps to `host[1]` in the full request but `host[0]` in the subset request. A `JOIN crowdstrike_detections JOIN crowdstrike_devices ON device_id` that pages through results may yield different device pairings depending on batch composition — violating the stable-mapping contract implied by BC-2.16.013 v1.30.

**Relationship to BC-2.16.013 v1.30:** The v1.30 host-pool clause states `device_id` MUST be `generate_host_ids(org_slug, seed)[det_index % HOST_COUNT]`. The `det_index` semantics were left ambiguous — this finding makes the required semantics explicit: `det_index` must be derived from `detection_id` itself (stable identifier), not batch position (non-stable).

**BC amendment:** Addressed by the same BC-2.16.013 v1.30→v1.31 amendment described in MED-001 above — the `det_index` disambiguation clause explicitly prohibits batch-position-derived indices.

**Resolution:**
- test-writer RED @36f0ba9c: `test_F_CSD_P31_OBS_001_detection_device_id_stable_across_batch_subsets` — verifies that `detection_detail()` called with `detection_id="det-org-seed-005"` returns the same `device_id` whether processing a full set or a singleton subset (identical `detection_id` string → identical `device_id` mapping regardless of call context).
- implementer GREEN @072930ee: `det_index` parsed INSIDE `detection_detail()` from `detection_id` trailing integer via `rsplit('-').last().unwrap_or("0").parse::<usize>().unwrap_or(0)`; deterministic and stable for any given `detection_id`; `.enumerate()` removed from call site (TD-VSDD-060 sweep confirmed single call site).

**Closure:** @072930ee (implementer)

---

### F-CSD-P31-OBS-002 (OBS) — `prism_query::ast::VirtualField` has no compile-fail gate coverage; Layer-2 per-symbol dedup keyed on last-segment name collapses both VirtualField types into one key (silent regression window)

**Severity:** OBS

**File:** `tests/external/non-exhaustive-violation/src/lib.rs` / `scripts/check-non-exhaustive-per-symbol.py`

**Description:** Pass 28 added `#[non_exhaustive]` to `prism_core::virtual_fields::VirtualField` and gate `v90_virtual_field_match` (E0004) to the compile-fail suite. The sibling type `prism_query::ast::VirtualField` (4-variant, `#[non_exhaustive]`) was already compliant per a note in D-1677, but has NO corresponding compile-fail gate coverage in the `non-exhaustive-violation` test crate. This creates a regression window: if `prism_query::ast::VirtualField`'s `#[non_exhaustive]` attribute were accidentally removed, no compile-fail test would catch it.

**Additional sub-issue — Layer-2 per-symbol key collision:** The Python script `scripts/check-non-exhaustive-per-symbol.py` keys its per-symbol deduplication on the LAST path segment (e.g., `VirtualField`). Since both `prism_core::virtual_fields::VirtualField` and `prism_query::ast::VirtualField` share the final segment `VirtualField`, Layer-2 dedup would collapse them into a single key — meaning an addition of the `prism_query::ast::VirtualField` gate would not register as a +1 in the symbol count, and a removal would not show as a -1. This is a silent regression window for the per-symbol verification.

**Resolution:**
- implementer @072930ee: 
  - Added `v91_ast_virtual_field_match` gate (E0004) for `prism_query::ast::VirtualField` to `tests/external/non-exhaustive-violation/src/lib.rs` (or `enum_violations.rs`); compile-fail violation verified against `#[non_exhaustive]` annotation; EXPECTED 90→91 in `ci.yml` (both EXPECTED_COUNT message and EXPECTED_SYMBOLS list), `scripts/check-non-exhaustive.sh`, and worktree `CLAUDE.md` convention sentence.
  - Layer-2 Python script `scripts/check-non-exhaustive-per-symbol.py`: keys for `prism_core::virtual_fields::VirtualField` and `prism_query::ast::VirtualField` entries disambiguated to 2-segment forms (`virtual_fields::VirtualField` vs `ast::VirtualField`) so dedup no longer collapses them (empirically verified against E0004 notes in violation file).
  - Violation-file doc-comment enumerations updated: enum_violations.rs count 22→23 (one entry added for `v91_ast_virtual_field_match`); totals cross-check updated to 91 (23 enum + 68 struct).

**Closure:** @072930ee (implementer)

---

## Verification Summary

The adversary conducted a full-policy pass over frozen HEAD ed2988cc (fix/csdevices-empty-pipeline; LOCAL-ONLY). The following verification work was completed:

1. **SAP-2 type parity audit (detections)** — read `crowdstrike.sensor.toml` detections column definitions; verified `severity` declared `column_type = "string"`; read `prism-dtu-harness` CrowdStrike `detection_detail()` handler; found `severity: serde_json::Value::Number(50)` — numeric, not string; compared against standalone DTU generator which emits string labels; found P29-006 test comment rationalizing numeric severity with `is_number()` assertion; classified as surface-and-defer anti-pattern → F-CSD-P31-MED-001.

2. **`det_index` stability audit** — traced `det_index` parameter threading from P30 `.enumerate()` at call site; constructed counter-example with full-set vs singleton request (same `detection_id` → different batch position → different `device_id`); verified stable-mapping contract violation → F-CSD-P31-OBS-001.

3. **Compile-fail gate completeness audit** — verified `prism_core::virtual_fields::VirtualField` gate `v90_virtual_field_match` present; searched for `prism_query::ast::VirtualField` gate; found absent; verified `prism_query::ast::VirtualField` is `#[non_exhaustive]` 4-variant pub type (D-1677 note confirmed); confirmed CLAUDE.md convention applies; verified Layer-2 Python script last-segment dedup would collapse both `VirtualField` entries → F-CSD-P31-OBS-002.

4. **SAP-1 catalog sweep** — `rg 'event_type\s*=' crates/ --type rust` across changed files; five production SQL emission sites; all five catalog rows present in BC-2.16.002 §Postconditions (v2.10); no new emission sites.

5. **SAP-2 TOML↔DTU parity (devices)** — 6/6 TOML-declared columns verified. Excess-field gap correctly deferred to DRIFT-SAP2-DEVICES-TOML-SURFACE-001.

6. **POL-24 byte-strict** — E-QUERY-041/042/043 hint strings verified byte-exact against error-taxonomy v2.39; no drift.

7. **POL-33 Route Coverage Table** — BC-2.16.013 v1.30 9-row table re-verified at frozen HEAD ed2988cc; all 9 routes confirmed present.

8. **Non-exhaustive gate EXPECTED=90** — all 90 compile-fail tests verified GREEN via two-layer check (count gate + per-symbol Python parse).

9. **BC-2.16.013 v1.30 host-pool clause** — `detection_detail()` `device_id` correctly computed as `generate_host_ids(org_slug, seed)[det_index % HOST_COUNT]`; host-pool constraint satisfied. `det_index` stability gap (F-CSD-P31-OBS-001) is distinct from and additive to this clause.

10. **Load-bearing test verification** — T39/T40/T41/T42, `negative_e043_parity_gate` tests, `test_bc_2_11_022_ci_4tier_gate`, `test_F_CSD_P29_006_detection_detail_full_toml_field_coverage` (P29-006 severity assertion verified — found `is_number()` assertion flagged as surface-and-defer), `test_F_CSD_P30_OBS_003_detection_device_id_is_valid_host_id_not_placeholder`, `test_F_CSD_P30_OBS_003_detection_device_ids_join_devices_nonempty` — all verified GREEN at frozen HEAD ed2988cc.

11. **P29-006 test rationalization audit** — specifically re-read test comment in `test_F_CSD_P29_006_detection_detail_full_toml_field_coverage`; found comment rationalizing numeric `severity` as "det_index % 4 maps to integer representation"; this rationalizes a type contract violation without fixing it; classified as surface-and-defer by TD-VSDD-059 standard; is_number() → is_string() flip required.

---

## Fix Record

**Fix-burst commits:**

1. **@36f0ba9c** (test-writer RED) — F-CSD-P31-MED-001 + F-CSD-P31-OBS-001:
   - `test_F_CSD_P29_006_detection_detail_full_toml_field_coverage`: `severity` assertion flipped from `is_number()` to `is_string()`; rationalization comment "det_index % 4 maps to integer" deleted; test now asserts string value
   - `test_F_CSD_P31_MED_001_detection_severity_is_string_label_matching_standalone_dtu`: new RED test — asserts `severity` field is a string and one of `["Low", "Medium", "High", "Critical"]`
   - `test_F_CSD_P31_OBS_001_detection_device_id_stable_across_batch_subsets`: new RED test — asserts same `detection_id` produces same `device_id` in both full-set and singleton-subset call contexts

2. **BC-2.16.013 v1.30→v1.31** (product-owner, uncommitted edits ratified in this commit) — F-CSD-P31-MED-001 + F-CSD-P31-OBS-001:
   - INV-HARNESS-ROUTE-PARITY `detection_detail()` clause: (1) `det_index` defined as canonical detection index parsed from `detection_id` trailing integer; batch-position-derived indices forbidden; stable mapping guaranteed across all request batch shapes. (2) `severity` MUST be a string label (`"Low"` / `"Medium"` / `"High"` / `"Critical"`) matching standalone DTU generator emission types and `crowdstrike.sensor.toml` `column_type = "string"`; numeric severity values forbidden.
   - Changelog row v1.31

3. **@072930ee** (implementer GREEN) — F-CSD-P31-MED-001 + F-CSD-P31-OBS-001 + F-CSD-P31-OBS-002:
   - `detection_detail()`: `det_index` param removed from signature; `det_index` now computed INSIDE the handler by parsing `detection_id` trailing integer via `rsplit('-').last().unwrap_or("0").parse::<usize>().unwrap_or(0)` (handles org_slug hyphens; deterministic fallback 0 on malformed, with inline comment); `severity` changed to `SEVERITY_LABELS[det_index % 4]` where `SEVERITY_LABELS = ["Low", "Medium", "High", "Critical"]`; `.enumerate()` removed from sole call site (TD-VSDD-060 sweep confirmed single call site; comment added referencing stable-mapping rule)
   - Compile-fail gate: `v91_ast_virtual_field_match` (E0004) added for `prism_query::ast::VirtualField`; EXPECTED 90→91 in ci.yml EXPECTED_COUNT message + EXPECTED_SYMBOLS list + check-non-exhaustive.sh + worktree CLAUDE.md convention sentence (provenance F-CSD-P31-OBS-002); enum_violations.rs doc-comment enumeration updated (22→23 entries; totals 91); Layer-2 Python script per-symbol keys disambiguated to 2-segment forms (`virtual_fields::VirtualField` / `ast::VirtualField`)
   - `just check` FULL WORKSPACE GREEN; non-exhaustive 91/91 two-layer per-symbol (expected count 5481 — 2 new tests over 5479 baseline; exact count pending re-verify at next gate; LOCAL-ONLY)

**New FROZEN HEAD for pass 32:** 072930ee (LOCAL-ONLY). Streak 0/3. Cascade now 31 passes (commits this burst: 36f0ba9c, 072930ee). Develop baseline UNCHANGED @b9cf3f9b.

---

## Streak Status

| Pass | Frozen HEAD | CLEAN(strict) | Streak |
|------|-------------|---------------|--------|
| 27 | 9fe2d016 | YES | 1/3 |
| 28 | 9fe2d016 | NO (1 OBS) | 0/3 RESET |
| 29 | 25b80a81 | NO (5 MED + 1 OBS + 1 PROCESS-GAP) | 0/3 |
| 30 | 7a6f6caa | NO (2 LOW + 2 OBS) | 0/3 |
| 31 (this pass) | ed2988cc | NO (1 MED + 1 LOW + 1 OBS) | **0/3** |

Pass 32 NEXT on NEW frozen HEAD 072930ee. Streak 0/3. If CLEAN(strict), streak advances to 1/3.
