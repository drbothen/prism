---
document_type: adversarial-review
scope: LOCAL
fix_defect: DEFECT-CSDEVICES-EMPTY-PIPELINE-001
passes: [29]
feature_head_at_review: 25b80a81
date: 2026-07-11
clean_strict: false
clean_pr_merge: false
finding_counts:
  total: 7
  crit: 0
  high: 0
  med: 5
  low: 0
  obs: 2
  process_gap: 1
code_behavior_defects: 1
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: adversary
---

# LOCAL Adversary Pass 29 — DEFECT-CSDEVICES-EMPTY-PIPELINE-001

---

## Pass 29 (frozen 25b80a81; fresh-context adversary; LOCAL cascade; policy rubric + SAP-1 + SAP-2 + POL-22 + POL-24 byte-strict + POL-33 table verification; streak candidate 1/3 — STREAK RESET — streak 0/3 → 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO

**Findings:** 7 total (0 CRIT / 0 HIGH / 5 MED / 0 LOW / 1 OBS / 1 PROCESS-GAP)

**Adversary novelty assessment:** MEDIUM — 3-tier→4-tier propagation sweep gap (F-CSD-P25-003 added NegativeE043 tier; BC never amended until this pass; F-CSD-P29-003 spec-code drift from that prior fix) + new SAP-2 detections-shape finding (F-CSD-P29-006, in-scope via devices-table precedent).

**SAP-1:** PASS — `rg 'event_type\s*=' crates/ --type rust` across all files changed relative to develop@b9cf3f9b: five SQL emission sites verified (pipe.sql_lowering × 3, pipe.sql_planning_error × 2); all five catalog rows present in BC-2.16.002 §Postconditions (v2.10); no new emission sites introduced without catalog rows.

**SAP-2 (devices):** PASS for declared columns (6/6 TOML columns present in DTU generator and fixture). Excess-field gap correctly deferred to DRIFT-SAP2-DEVICES-TOML-SURFACE-001.

**SAP-2 (detections — F-CSD-P29-006):** FAIL — `detection_detail()` handler in `prism-dtu-harness` CrowdStrike clone exposes only a 4-field thin shape (`detection_id`, `status`, `severity`, `device`) vs 12 TOML-declared detections columns. See F-CSD-P29-006 below.

**POL-22:** PASS — All cited entities resolve; no dangling cross-references.

**POL-24 (byte-strict):** PASS — E-QUERY-041/042/043 hint strings verified byte-exact against error-taxonomy v2.39 and production code; no drift.

**POL-33:** PASS on existing route coverage table (BC-2.16.013 v1.28 9-row table verified).

**Non-exhaustive gate (EXPECTED=90):** PASS — all 90 gates verified GREEN on frozen HEAD 25b80a81.

**STREAK:** 0/3 UNCHANGED — CLEAN(strict)=NO (5 MED + 1 OBS + 1 PROCESS-GAP findings). Streak stays 0/3.

**Code HEAD at review:** 25b80a81 (frozen; just check FULL WORKSPACE 5477/5477 GREEN, 60 skipped; non-exhaustive 90/90 two-layer per-symbol; LOCAL-ONLY; develop baseline UNCHANGED @b9cf3f9b)

**CLEAN(strict):** NO — 5 MED + 1 OBS + 1 PROCESS-GAP findings
**CLEAN(PR-merge):** NO — 5 MED findings present

---

## Findings

### F-CSD-P29-001 (MED) — ci.yml FAIL message omits TemporalLiteralPosition; claims 90 but lists 89

**Severity:** MED

**File:** `ci.yml`

**Description:** The non-exhaustive CI step's FAIL message text enumerates the expected 89 type names in a FAIL message string but does not include `TemporalLiteralPosition` (added at pass 25 / PR #214 for the temporal typing story). The EXPECTED counter says 90, but the human-readable FAIL message listing only has 89 names. This creates a situation where a count mismatch produces a FAIL message that cannot distinguish which symbol was added or removed — the "expected 90 but got N" message is not accompanied by the correct 90-symbol list to diff against.

**Risk:** CI gating is count-only; net-zero regression (one symbol added, one removed) is undetectable. If a pub type is removed and a different pub type is added in the same PR, the count stays at 90 but the wrong types are in scope.

**Resolution:** Absorbed by OBS-001 process-gap remediation: implementer replaced the 106-line inline ci.yml step with a single `bash scripts/check-non-exhaustive.sh` call; `check-non-exhaustive.sh` delegates to new `scripts/check-non-exhaustive-per-symbol.py` which maintains an explicit 90-symbol list with a `len==90` import guard, E0639-via-spans-text parse, and E0004-via-defined-here note parse. Validated 0 UNKNOWN across all 90 symbols on frozen HEAD.

**Closure:** @3dd1fd96

---

### F-CSD-P29-002 (MED) — `resources.rs` 4 stale "3-tier" doc sites; contract enumeration missing NegativeE043

**Severity:** MED

**File:** `crates/prism-mcp/src/resources.rs`

**Description:** Four documentation sites in `resources.rs` still say "3-tier gate" or enumerate only three `ExampleKind` variants after F-CSD-P25-003 added the NegativeE043 tier. Additionally, the contract enumeration within these sites does not mention `NegativeE043` as a valid `ExampleKind` member. A POL-25 worktree grep found these four sites to be the complete set; no spillover to other files detected.

**Risk:** Documentation accuracy for callers of `build_reference_content`; audit trail of the 4-tier design is incomplete in the source; contributes to spec-code drift surface for future passes.

**Resolution:** Implementer swept 4 sites → 4-tier + NegativeE043 in contract enumeration; POL-25 worktree grep confirms clean.

**Closure:** @3dd1fd96

---

### F-CSD-P29-003 (MED) — BC-2.11.022 declared 3-variant ExampleKind; code shipped 4-variant (spec-code drift from F-CSD-P25-003)

**Severity:** MED

**File:** `.factory/specs/behavioral-contracts/BC-2.11.022-auto-generated-prismql-reference-content-contract-and-ci-parity-gate.md`

**Description:** BC-2.11.022 at version v1.2 described `ExampleKind` as a 3-variant enum (`Positive`, `NegativeE040`, `NegativeOther`). F-CSD-P25-003 (pass 25 fix-burst) added `ExampleKind::NegativeE043` as a 4th variant plus a corresponding CI gate in `crates/prism-mcp/tests/negative_e043_parity_gate.rs`. The BC was never amended to reflect the 4th variant. This is a spec-code drift: the code is correct (the NegativeE043 tier is load-bearing and correct per the DEFECT-CSDEVICES cascade), but the spec lagged.

**Risk:** Future adversary passes, consistency validators, and story-writers reading BC-2.11.022 will see a 3-variant description that does not match the code. If a future implementer adds a new `ExampleKind` variant, they have no spec anchor requiring the CI gate be updated.

**Resolution:** Product-owner amended BC-2.11.022 v1.2→v1.3: `ExampleKind` enum corrected to 4 variants; §CI round-trip gate updated to "four complementary assertions" with new Tier 3 item for `NegativeE043` load-bearing E-QUERY-043 plan-time rejection gate; EC-11-022-007 edge case added; NegativeE043 canonical test vector added; changelog row v1.3 authored.

**Closure:** BC-2.11.022 v1.3 (product-owner, included in this commit)

---

### F-CSD-P29-004 (MED) — ci.yml history comment "ADR-045 3-tier gate" stale

**Severity:** MED

**File:** `ci.yml`

**Description:** A history/changelog comment in the non-exhaustive CI step references "ADR-045 3-tier gate" — the correct characterization is now "4-tier gate" after F-CSD-P25-003 added the NegativeE043 tier. This is a documentation-accuracy finding on the ci.yml inline comment, not a functional gap.

**Resolution:** Absorbed by OBS-001 hardening: the 106-line inline step was replaced entirely by `bash scripts/check-non-exhaustive.sh`; the stale comment was removed as part of that refactor.

**Closure:** @3dd1fd96

---

### F-CSD-P29-005 (MED) — ci.yml count comment "47+17=64" stale; truth is 68+22=90

**Severity:** MED

**File:** `ci.yml`

**Description:** The ci.yml non-exhaustive step contained a count annotation comment summing the expected symbol counts as "47+17=64" — a stale figure from an earlier state of the gate. The correct current split is 68+22=90 (68 E0639 symbols + 22 E0004 symbols = 90 total). The count mismatch between the comment and the actual EXPECTED=90 could confuse future engineers auditing the gate breakdown.

**Resolution:** Absorbed by OBS-001 hardening: the 106-line inline step was replaced by `bash scripts/check-non-exhaustive.sh`; all stale count comments were removed; the per-symbol Python script is now the authoritative source for the split.

**Closure:** @3dd1fd96

---

### F-CSD-P29-006 (MED) — harness `detection_detail()` 4-field thin shape vs 12 TOML detections columns; `device_id` nested not top-level; key fields absent

**Severity:** MED (SAP-2 severe direction; pre-existing prior to this branch)

**File:** `crates/prism-dtu-harness/src/crowdstrike/detections.rs` (or equivalent)

**Description:** The `prism-dtu-harness` CrowdStrike clone's `detection_detail()` handler returns a thin shape with approximately 4 fields (`detection_id`, `status`, `severity`, `device` nested sub-object) vs the 12 TOML-declared detections columns (`detection_id`, `status`, `severity`, `created_timestamp`, `tactic`, `technique`, `behaviors_ioc_type`, `behaviors_ioc_value`, `behaviors_ioc_source`, `behaviors_ioc_description`, `device_id` as a top-level field, plus nested device sub-object for context). `device_id` is declared as a top-level detections column but is only present nested inside a `device` sub-object. `created_timestamp`, `tactic`, `technique`, and `behaviors_ioc_*` are entirely absent from the harness response.

This is a SAP-2 violation (TOML-declared column present, DTU equivalent absent at matching path). The devices-table precedent (`host_detail()` 6/6 field-completeness, F-CSD-P10-OBS-1) applies: harness handler must cover all TOML-declared columns at the correct paths.

**Architect adjudication (IN-SCOPE-FIX, 2026-07-11):** EXEMPT path evaluated and failed all three Canonical-Principle deferral criteria (no human direction to defer, no concrete future dependency requiring deferral, no specific future story anchor). The silent-NULL path is unreachable in current suites but the fix is additive and does not require redesign. Devices-table `host_detail()` 6/6 precedent is binding.

**Resolution:**
- test-writer RED @74c578c3: `test_F_CSD_P29_006_detection_detail_full_toml_field_coverage` in new harness `[[test]]` entry
- implementer GREEN @7a6f6caa: `detection_detail()` expanded 4→12 fields; `ioc_value` explicit null (nullable per TOML type); nested `device` sub-object retained for backward compat; TD-VSDD-060 sweep clean; BC-2.16.013 v1.29 spec-note clause added by product-owner (INV-HARNESS-ROUTE-PARITY detection_detail response-shape clause)

**Closure:** @7a6f6caa (implementer), BC-2.16.013 v1.29 (product-owner, included in this commit)

---

### F-CSD-P29-OBS-001 [process-gap] — non-exhaustive CI assertion count-only; net-zero regression undetectable

**Severity:** OBS [process-gap]

**Description:** The CI non-exhaustive gate asserts a single integer count (EXPECTED=90). A net-zero regression — one symbol removed AND one new symbol added in the same PR — produces count=90 and passes the gate even though the wrong types are being enforced. This was surfaced as a process-gap by F-CSD-P29-001 analysis.

**Resolution:** Implementer replaced the 106-line inline ci.yml step with `bash scripts/check-non-exhaustive.sh`; created new `scripts/check-non-exhaustive-per-symbol.py` with an explicit 90-symbol list and `len==90` import guard. The script parses E0639 (via `spans-text`) and E0004 (via `defined-here` note text) compiler error patterns from `cargo check` output to verify each of the 90 expected symbols individually. Validated 0 UNKNOWN across all 90 symbols at frozen HEAD. Net-zero regression is now detectable: if a symbol is removed from the list but a new one passes compilation, the grep check will report the missing symbol as absent.

**Closure:** @3dd1fd96 (scripts/check-non-exhaustive.sh + scripts/check-non-exhaustive-per-symbol.py + ci.yml refactor to single bash call)

---

### F-CSD-P29-OBS-002 (OBS) — `fetch_incidents` POST shape has zero DTU coverage until DTU-EXT-001

**Severity:** OBS (informational; no action required in this burst)

**Description:** The `crowdstrike.sensor.toml` `fetch_incidents` step uses `method = "POST"` with `body_template` `query_incident_ids`. The `prism-dtu-harness` CrowdStrike clone does not yet implement a POST `/incidents/entities/incidents/GET/v1` route (this is the CrowdStrike POST-with-GET-verb convention, which is distinct from standard POST). The DTU clone's incidents route is GET-only. This gap is correctly anchored to DTU-EXT-001 (external-service integration test dependency) and is pre-existing.

**Action:** No code change required in this burst. DTU-EXT-001 owner must implement the incidents POST route as `POST /incidents/entities/incidents/GET/v1` with body `{"ids": [...]}` (NOT a plain GET). The `GET` in the URL path is the CrowdStrike convention for bulk-fetch endpoints. Noted here for DTU-EXT-001 owner's reference.

**Closure:** No action. Noted for DTU-EXT-001 owner.

---

## Verification Summary

The adversary conducted a full-policy pass over frozen HEAD 25b80a81 (fix/csdevices-empty-pipeline; LOCAL-ONLY). The following verification work was completed:

1. **SAP-1 catalog sweep** — grep `event_type\s*=` across all files changed relative to develop@b9cf3f9b; five production SQL emission sites verified (pipe.sql_lowering × 3, pipe.sql_planning_error × 2); all five catalog rows present in BC-2.16.002 §Postconditions (v2.10); no new emission sites.

2. **SAP-2 TOML↔DTU parity (devices)** — 6 TOML-declared columns verified present in DTU generator and fixture. Excess-field gap (os_version, containment_status, external_ip, local_ip, agent_version, cid/agent_id) correctly deferred to DRIFT-SAP2-DEVICES-TOML-SURFACE-001.

3. **SAP-2 TOML↔DTU parity (detections)** — `crowdstrike.sensor.toml` detections columns enumerated (12 columns); cross-referenced against `prism-dtu-harness` CrowdStrike `detection_detail()` response shape; identified thin-shape gap (F-CSD-P29-006).

4. **BC-2.16.013 v1.28 Route Coverage Table** — 9 rows verified against route registration in both harness builders. All 9 routes confirmed present.

5. **BC-2.11.022 version audit** — BC-2.11.022 v1.2 read; confirmed 3-variant ExampleKind declaration vs 4-variant code (F-CSD-P29-003); spec-code drift confirmed.

6. **ci.yml non-exhaustive gate inspection** — FAIL message symbol list inspected; TemporalLiteralPosition absent from list despite EXPECTED=90 (F-CSD-P29-001); count comment "47+17=64" stale vs truth 68+22=90 (F-CSD-P29-005); history comment "ADR-045 3-tier gate" stale (F-CSD-P29-004); OBS-001 process-gap: count-only gate cannot detect net-zero regressions.

7. **resources.rs 4-tier audit** — `build_reference_content` and adjacent doc sites enumerated; 4 stale "3-tier" / 3-variant references found (F-CSD-P29-002).

8. **POL-24 byte-strict E-QUERY-041/042/043 templates** — hint strings compared byte-for-byte against error-taxonomy v2.39 and production emission sites; no drift.

9. **Non-exhaustive gate EXPECTED=90** — verified current `ci.yml` EXPECTED=90; gate `v90_virtual_field_match` (E0004) confirmed present; all 90 compile-fail tests pass at frozen HEAD.

10. **Load-bearing test verification** — T39 (Pipe wildcard boundary), T40 (SqlPipe head InSubquery E-QUERY-043 lock), T41 (SqlPipe stages walk), T42 (TimestampArithmetic lock), `negative_e043_parity_gate` tests, `test_bc_2_11_022_ci_4tier_gate` — all verified GREEN at frozen HEAD 25b80a81.

---

## Architect Memo Summary

**F-CSD-P29-006 IN-SCOPE-FIX adjudication (2026-07-11):**

Architect evaluated the `detection_detail()` thin-shape gap against the three Canonical-Principle deferral criteria. All three criteria failed:

1. **No human direction to defer** — no prior decision records an explicit human authorization to defer the detections harness shape completeness.
2. **No concrete future dependency** — the devices-table `host_detail()` 6/6 precedent (F-CSD-P10-OBS-1, 2026-07-10) binds this case: harness shape completeness for TOML-declared columns is in-scope when the DTU path is already in use for scenario testing.
3. **No specific future story anchor** — DRIFT-SAP2-DEVICES-TOML-SURFACE-001 covers *TOML spec* expansion for devices; it does not cover harness shape completeness for detections. They are distinct issues.

The fix is additive: `detection_detail()` gains 8 new fields + `ioc_value` explicit null. The nested `device` sub-object is retained for backward compatibility. The silent-NULL path (TOML-declared columns missing from harness response) is currently unreachable because no scenario exercises both a 0-batch and a populated detections path in the same JOIN, but the fix is still correct and non-breaking.

Ruling: **IN-SCOPE-FIX**. Test-writer dispatched for RED gate; implementer dispatched for GREEN.

---

## Fix Record

**Fix-burst commits:**

1. **@3dd1fd96** (implementer) — F-CSD-P29-001/004/005 + F-CSD-P29-OBS-001 hardening:
   - Replaced 106-line inline ci.yml non-exhaustive step with single `bash scripts/check-non-exhaustive.sh` call
   - New `scripts/check-non-exhaustive-per-symbol.py`: explicit 90-symbol list, `len==90` import guard, E0639-via-spans-text + E0004-via-defined-here-note parse, 0 UNKNOWN across all 90 symbols verified
   - `check-non-exhaustive.sh`: two-layer (Layer 1 count + Layer 2 per-symbol via python script)
   - `resources.rs` 4 stale "3-tier" doc sites → "4-tier" + NegativeE043 in contract enumeration (F-CSD-P29-002)
   - POL-25 worktree grep: no remaining "3-tier" in perimeter files

2. **BC-2.11.022 v1.2→v1.3** (product-owner, uncommitted edit ratified in this commit) — F-CSD-P29-003 spec-code drift closure:
   - `ExampleKind` enum: 3-variant → 4-variant (adds `NegativeE043`)
   - §CI round-trip gate: "three complementary assertions" → "four complementary assertions"; Tier 3 `NegativeE043` gate item added
   - EC-11-022-007 edge case added (NegativeE043 gate regression)
   - NegativeE043 canonical test vector added
   - Changelog row v1.3

3. **BC-2.16.013 v1.28→v1.29** (product-owner, uncommitted edit ratified in this commit) — F-CSD-P29-006 spec-note:
   - INV-HARNESS-ROUTE-PARITY: added explicit CrowdStrike `detection_detail()` response-shape clause
   - Requires 12 TOML-declared detections columns at correct paths; `device_id` top-level; `behaviors` array with IOC keys; `ioc_value` nullable
   - Changelog row v1.29
   - SESSION-HANDOFF.md line-137 anchor: v1.28→v1.29 (also ratified in this commit)

4. **@74c578c3** (test-writer RED) — F-CSD-P29-006:
   - `test_F_CSD_P29_006_detection_detail_full_toml_field_coverage` in new harness `[[test]]` entry
   - Asserts all 12 TOML-declared detections columns present at correct paths
   - Asserts `ioc_value` nullable (null is valid, key must be present)
   - RED gate confirmed before implementer dispatch

5. **@7a6f6caa** (implementer GREEN) — F-CSD-P29-006:
   - `detection_detail()` expanded 4→12 fields
   - `ioc_value` explicit null
   - Nested `device` sub-object retained for backward compatibility
   - TD-VSDD-060 sweep: no unintended callsite regressions
   - `just check` FULL WORKSPACE 5477/5477 GREEN (60 skipped); `prism-dtu-harness` 142/142; non-exhaustive 90/90 two-layer per-symbol

**New FROZEN HEAD for pass 30:** 7a6f6caa (LOCAL-ONLY). Streak 0/3. Cascade now 29 passes (commits this burst: 3dd1fd96, 74c578c3, 7a6f6caa). Develop baseline UNCHANGED @b9cf3f9b.

---

## Streak Status

| Pass | Frozen HEAD | CLEAN(strict) | Streak |
|------|-------------|---------------|--------|
| 27 | 9fe2d016 | YES | 1/3 |
| 28 | 9fe2d016 | NO (1 OBS) | 0/3 RESET |
| 29 (this pass) | 25b80a81 | NO (5 MED + 1 OBS + 1 PROCESS-GAP) | **0/3** |

Pass 30 NEXT on NEW frozen HEAD 7a6f6caa. Streak 0/3. If CLEAN(strict), streak advances to 1/3.
