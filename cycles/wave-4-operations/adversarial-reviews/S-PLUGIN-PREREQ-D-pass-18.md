---
document_type: adversarial-pass-report
target_artifact: S-PLUGIN-PREREQ-D
pass_N: 18
target_sha: 4b28d5d6
story_content_sha: 1cf0a905
base_sha: 95d46be2
verdict: BLOCKED-soft
streak: "0/3 → 0/3 (HOLD — streak does NOT advance)"
finding_summary: {CRITICAL: 0, HIGH: 0, MEDIUM: 1, LOW: 2, OBS: 1}
prior_passes: [pass-1..pass-17]
prior_fix_bursts: [fix-burst-1..fix-burst-17]
trajectory: "16 → 8 → 6 → 4 → 0 → 4 → 7 → 4 → 2 → 2 → 2 → 1 → 1 → 1 → 3 → 6 → 4 → 4"
idempotency_check: false
producer: adversary (vsdd-factory; reified by state-manager due to read-only tool profile)
---

# S-PLUGIN-PREREQ-D Adversarial Pass 18 Report

## §1 Executive Summary

Fresh-context audit of S-PLUGIN-PREREQ-D story v1.16 at factory HEAD 4b28d5d6 (story-writer stage 1B commit). Audit conducted against:

- S-PLUGIN-PREREQ-D story file (v1.16 post-fix-burst-17 story-writer stages 1A+1B)
- BC-2.16.002 v1.12 (canonical structured event catalog — 25 rows post-fix-burst-17 PO stage 1A)
- BC-2.22.001 v1.5 (plugin-load warning invariant)
- BC-2.17.001..004 + BC-2.17.006 + BC-2.17.007 (plugin behavior contracts)
- ADR-022 v1.3 (boot sequence wiring)
- ADR-023 v1.18 (plugin runtime specification)
- VP-PLUGIN-004 + VP-PLUGIN-007

**Verdict: BLOCKED-soft.** 1 MEDIUM + 2 LOW + 1 OBS. Streak HOLD 0/3 — 8th consecutive advance failure. Trajectory plateau: 6→4→4 (flat; declining novelty signature continues). All findings are precision/completeness-level; severity ceiling MEDIUM. No new CRITICAL or HIGH findings.

**Trajectory observation:** The 4→4 plateau with severity ceiling MEDIUM (down from the 6-finding rebound at pass-16) indicates declining novelty is being preserved despite the plateau count. The remaining finding space is narrowing toward the AC-5 validation table / catalog symmetric coverage axis — a lexical-vs-semantic gap class that the adversary has flagged as a recurring surface.

**Forecast uplift:** Pass-18 parallel PO+story-writer dispatch (fix-burst-17) closed 3/3 in-perimeter findings. Pass-19 forecast ~60% CLEAN (highest forecast yet). Pass-20 ~70%. Pass-21 ~85% — 3-CLEAN window opens pass-19..21.

---

## §2 External-Anchor Verification

27 external anchors verified in this pass. All PASS.

| # | Anchor | Location | Verdict |
|---|--------|----------|---------|
| 1 | BC-2.16.002 §Structured Event Catalog (v1.12) catalog row count = 25 | BC-2.16.002 v1.12 | PASS |
| 2 | AC-5 four-field validation: name / version / format_version / allowed_urls | Story §AC-5 | PASS |
| 3 | E-PLUGIN-015 PluginError::ManifestNameMissing exists | error-taxonomy.md §E-PLUGIN-015 | PASS |
| 4 | E-PLUGIN-016 PluginError::ManifestVersionMalformed exists | error-taxonomy.md §E-PLUGIN-016 | PASS |
| 5 | BC-2.17.007 §Preconditions manifest validation before WIT validation | BC-2.17.007 | PASS |
| 6 | ADR-023 §C4 plugin manifest validation ordering invariant | ADR-023 v1.18 §C4 | PASS |
| 7 | VP-PLUGIN-004 property: manifest validation gate | VP-PLUGIN-004 | PASS |
| 8 | VP-PLUGIN-007 property: manifest name + version field presence | VP-PLUGIN-007 | PASS |
| 9 | ADR-022 §C (wiring not redesign principle) | ADR-022 v1.3 §C | PASS |
| 10 | BC-2.22.001 §Sequencing Invariant step 7.5 plugin-load | BC-2.22.001 v1.5 | PASS |
| 11 | EC-D-012 appended for E-PLUGIN-015 (fix-burst-16 closure) | Story §Error Cases | PASS |
| 12 | EC-D-013 appended for E-PLUGIN-016 (fix-burst-16 closure) | Story §Error Cases | PASS |
| 13 | assumption_validations array 5 items (fix-burst-16 closure) | Story frontmatter | PASS |
| 14 | risk_mitigations array 8 items (fix-burst-16 closure) | Story frontmatter | PASS |
| 15 | Task 5 explicit `[dependencies]` placement for zeroize + url (fix-burst-16 closure) | Story §Task 5 | PASS |
| 16 | E-PLUGIN-013 PluginError::ManifestParseError exists | error-taxonomy.md §E-PLUGIN-013 | PASS |
| 17 | E-PLUGIN-014 PluginError::ManifestSchemaViolation exists | error-taxonomy.md §E-PLUGIN-014 | PASS |
| 18 | BC-2.16.002 §Structured Event Catalog row: plugin_load_failed_wit_invalid | BC-2.16.002 v1.12 | PASS |
| 19 | BC-2.16.002 §Structured Event Catalog row: plugin_load_failed_format_version_exceeded | BC-2.16.002 v1.12 | PASS |
| 20 | BC-2.16.002 §Structured Event Catalog row: plugin_load_failed_manifest_no_allowed_urls | BC-2.16.002 v1.12 | PASS |
| 21 | AC-5 "empty list [] accepted" correction (fix-burst-17 story-writer stage 1B) | Story §AC-5 Task 1 line | PASS |
| 22 | Story §Structured Event Catalog Additions table 2 new rows (fix-burst-17 stage 1B) | Story §Catalog Additions | PASS |
| 23 | Task 10 deferred-to-§RG pattern matching Task 11 (fix-burst-17 stage 1B) | Story §Task 10 | PASS |
| 24 | 5 count narrative sites updated 7→9 (fix-burst-17 stage 1B) | Story frontmatter + body | PASS |
| 25 | BC-2.16.002 v1.12 2 new catalog rows: plugin_load_failed_manifest_name_missing + plugin_load_failed_manifest_version_malformed | BC-2.16.002 §Catalog | PASS |
| 26 | Token budget arithmetic: ~40,200 / 15.7% stable | Story §Token Budget | PASS |
| 27 | F-LP10-OBS-001 single-commit-with-TBD-pin preserved (9th consecutive) | fix-burst-17 closure + STATE.md | PASS |

---

## §3 Fix-Burst-17 Closure Verification

Fix-burst-17 claimed closure of 3 findings across parallel PO + story-writer dispatch. Verification: 4/4 PASS (3 closed findings verified + TBD-pin discipline preserved).

| Finding | Claimed Closure | Verified? |
|---------|-----------------|-----------|
| F-LP18-MED-001 BC portion (BC-2.16.002 §Catalog missing rows for name-missing + version-malformed) | PO @ 84f58565: 2 new catalog rows added; total 23→25 | PASS |
| F-LP18-MED-001 story portion (story §Catalog Additions table missing symmetric coverage) | story-writer @ 4b28d5d6: 2 parallel catalog rows added | PASS |
| F-LP18-LOW-001 (Task 1 line validation "allowed_urls non-empty" → "empty list [] accepted") | story-writer @ 4b28d5d6: line rewritten | PASS |
| F-LP18-LOW-002 (Task 10 body deferred to §RG Tests pattern) | story-writer @ 4b28d5d6: Task 10 rewritten | PASS |

**Note on finding count:** The adversary categorized F-LP18-MED-001 as ONE finding with two sub-parts (BC portion + story portion). Fix-burst-17 closed both sub-parts in a parallel dispatch (PO for BC, story-writer for story). The parallel-dispatch coherence was verified: both agents used canonical event names `plugin_load_failed_manifest_name_missing` + `plugin_load_failed_manifest_version_malformed`; commits touched different files (BC vs story); no cross-dependency conflict.

---

## §4 Idempotency Check

Not applicable this pass. Prior passes 1-17 have been verified for idempotency at their respective closure points.

---

## §5 Critical Findings

ZERO critical findings.

---

## §6 High Findings

ZERO high findings.

---

## §7 Medium Findings

### F-LP18-MED-001 — Lexical-vs-Semantic Gap: AC-5 Validation Table Symmetric Coverage vs §Catalog Additions

**Severity:** MEDIUM
**Surface:** Story §AC-5 validation table + §Structured Event Catalog Additions table

**Finding:**

Fix-burst-16 closed AC-5 ↔ EC table propagation (EC-D-012/013 added for E-PLUGIN-015/016). Fix-burst-17 addressed the §Catalog Additions table's missing rows for the two new error codes. However, close reading of §AC-5's validation table reveals a lexical coverage gap:

The AC-5 validation table enumerates four manifest field checks: `name`, `version`, `format_version`, `allowed_urls`. The story §Catalog Additions now correctly adds `plugin_load_failed_manifest_name_missing` (name check failure) and `plugin_load_failed_manifest_version_malformed` (version check failure). However, the AC-5 validation table's **error event column** does not enumerate the corresponding catalog event names for the name-missing and version-malformed cases — the table rows describe the validation predicate but do not cross-reference the specific `event_type` values from the catalog.

This is a **lexical gap** (the event names appear in §Catalog Additions but not in the AC-5 table's event column) rather than a **semantic gap** (the behaviors are correctly specified). The adversary classifies this MEDIUM because implementers reading only the AC-5 table would not discover the correct event_type strings without cross-referencing §Catalog Additions — a non-obvious cross-section read.

**Prescriptive Fix:**
In the AC-5 validation table, add an "Error Event" column (or annotate the relevant rows) that cites `plugin_load_failed_manifest_name_missing` for the name-check failure row and `plugin_load_failed_manifest_version_malformed` for the version-check failure row. This makes the AC-5 table self-contained for implementers without requiring cross-referencing §Catalog Additions.

**Note to fix-burst-18:** This finding is within the existing lexical-vs-semantic-sweep pattern (F-LP15-MED-002 adversary-must-verify-external-anchors / 4th recurrence). The BC-2.16.002 v1.12 catalog rows already exist and are correctly named. The fix is story-level only: annotate AC-5 table rows with the corresponding catalog event_type values.

---

## §8 Low Findings

### F-LP18-LOW-001 — Task 1 Validation Coverage: "allowed_urls non-empty" Claim After Fix-Burst-17

**Severity:** LOW
**Surface:** Story §Task 1 — AC-5 validation description

**Finding:**

Fix-burst-17 stage 1B corrected the "allowed_urls non-empty" claim to "empty list [] accepted." However, the Task 1 narrative now reads: validation checks `name`, `version`, `format_version`, `allowed_urls` — but does not explicitly state that the `allowed_urls` field presence check (is the key present?) is distinct from the value check (must the list be non-empty?). An implementer reading this could reasonably implement a "field must exist AND be non-empty" gate, which contradicts the "empty list [] accepted" correction.

**Prescriptive Fix:**
Add a parenthetical to the Task 1 `allowed_urls` validation row: "`allowed_urls` field presence required; empty list `[]` is valid (no url allowlist enforced if field is present but empty)." This makes the distinction explicit without requiring the implementer to hold the correction in memory from an earlier reading.

**Classification:** This is a precision/completeness finding within an already-corrected surface. Severity LOW because the "empty list [] accepted" language in the same paragraph does convey the intent to an attentive reader.

---

### F-LP18-LOW-002 — Task 10 Deferred-to-§RG Pattern: Missing Explicit §RG Section Reference

**Severity:** LOW
**Surface:** Story §Task 10 body after fix-burst-17 rewrite

**Finding:**

Fix-burst-17 stage 1B rewrote Task 10 to "defer to §Red Gate Tests matching Task 11 pattern." The Task 10 body now says the validation is covered in §Red Gate Tests. However, the body does not include a markdown link or explicit section identifier (`§Red Gate Tests`) — it uses informal prose that references the concept without the canonical section anchor. Task 11 (which Task 10 is meant to mirror) uses the explicit `§Red Gate Tests` anchor form in its deferral prose.

**Prescriptive Fix:**
Update Task 10's deferral prose to include the explicit `§Red Gate Tests` anchor matching Task 11's form: "See §Red Gate Tests for the name-missing and version-malformed test cases." The current informal deferral is sufficient for human readers but inconsistent with the Task 11 canonical form.

**Classification:** LOW. No semantic gap — the §Red Gate Tests section exists and contains the relevant test cases. This is a prose consistency issue between Task 10 and Task 11's deferral patterns.

---

## §9 Observations

### F-LP18-OBS-001 — Process-Gap: 4th Recurrence of Lexical-vs-Semantic-Sweep Pattern

**Severity:** OBS [process-gap]
**Surface:** Process discipline — adversary pass methodology

**Observation:**

This pass found F-LP18-MED-001, which is the 4th instance of a lexical-vs-semantic cross-section gap in this cascade. The pattern: a finding is semantically correct in one artifact (BC-2.16.002 §Catalog has the right event_type values) but the story's cross-referencing table (AC-5 validation table) does not cite those values explicitly, requiring an implementer to perform a non-obvious cross-section read.

Prior instances:
1. F-LP9-OBS-001 (version-pin-sweep-burst-vs-version-prose distinction)
2. F-LP15-MED-002 (adversary-must-verify-external-anchors — first explicit lexical-vs-semantic coding)
3. F-LP16-MED-001 / F-LP16-MED-002 (citation + hedging gaps requiring cross-section reads)
4. F-LP18-MED-001 (this finding — AC-5 table missing event_type cross-reference)

**Recommendation:** This is the 4th recurrence of the lexical-vs-semantic-sweep pattern within process-gap candidate 5 (`adversary-must-verify-external-anchors`). The codification proposal in that candidate stands. The adversary should include a specific lexical-cross-section sweep step as part of the standard pass protocol: after verifying semantic correctness of all artifacts, verify that any artifact whose content references another artifact's named constants (error codes, event_type strings, variant names) actually cites those constants by name — not just by concept.

**Note:** This is a process-gap observation, not a spec finding. It reinforces existing codification candidate 5 and does not open a new candidate.

---

## §10 Carry-Forward Closure Verification (Passes 1–17 Sampled)

Representative sample of prior findings verified as still closed after fix-burst-17:

| Finding | Sampled At | Verdict |
|---------|------------|---------|
| F-LP1-HIGH-004 (BC-2.17.007 manifest validation ordering) | BC-2.17.007 v1.2 | STILL CLOSED |
| F-LP5-CRIT-001 (AC-5 four-field validation structure) | Story §AC-5 | STILL CLOSED |
| F-LP8-OBS-002 (lifecycle_status drift pattern) | BC-2.17.001..007 lifecycle_status | STILL CLOSED |
| F-LP9-MED-001 (BC-2.16.002 universal catalog scope) | BC-2.16.002 v1.12 header | STILL CLOSED |
| F-LP10-OBS-001 (single-commit-with-TBD-pin pattern) | fix-burst-17 meta | STILL CLOSED (9th consecutive) |
| F-LP15-MED-002 (adversary-must-verify-external-anchors) | §2 External-Anchor Verification | STILL CLOSED — 27 anchors verified |
| F-LP16-HIGH-001 (PrismError::Internal E-INT-001 non-existent variant) | Story §AC-9 code sample | STILL CLOSED |
| F-LP17-LOW-003 (EC-D-012/013 for E-PLUGIN-015/016) | Story §Error Cases | STILL CLOSED |

---

## §11 Novelty Assessment

**Novelty: DECLINING.** Trajectory plateau 6→4→4. Severity ceiling has dropped from HIGH (pass-9) to MEDIUM (pass-18). The remaining finding space is dominated by precision/completeness cross-referencing gaps within already-correctly-specified semantics. The signal pattern is consistent with a cascade approaching convergence:

- Pass-15: 3 findings (1H/2L)
- Pass-16: 6 findings (1H/2M/2L/1OBS) — rebound from prescription gap
- Pass-17: 4 findings (3L/1OBS) — severity floor at LOW
- Pass-18: 4 findings (1M/2L/1OBS) — severity floor at LOW/MED; plateau

The F-LP18-MED-001 finding (MEDIUM) is a cross-referencing gap rather than a substantive semantic error — the event names exist correctly in BC-2.16.002 v1.12 and are used correctly by the story. The finding is about citation completeness within the AC-5 table. This class of finding is amenable to a single-pass fix (add cross-reference column to AC-5 table).

**Convergence forecast:**
- Pass-19: ~60% CLEAN (highest forecast yet; fix-burst-18 closes 3/4 in-perimeter; F-LP18-OBS-001 reinforces existing candidate only)
- Pass-20: ~70% CLEAN
- Pass-21: ~85% CLEAN → 3-CLEAN window opens pass-19..21

---

## §12 Recommended Next Dispatch

**Action:** Dispatch fix-burst-18 immediately. Scope: story-writer only (all 3 actionable findings are story-level; BC-2.16.002 v1.12 is correct as-is).

**Fix-burst-18 scope:**
1. F-LP18-MED-001: In AC-5 validation table, add "Error Event" column or annotation for name-missing row (`plugin_load_failed_manifest_name_missing`) and version-malformed row (`plugin_load_failed_manifest_version_malformed`). Story-writer scoped fix.
2. F-LP18-LOW-001: Task 1 `allowed_urls` validation row — add parenthetical distinguishing field-presence check from value-non-empty check.
3. F-LP18-LOW-002: Task 10 deferral prose — add explicit `§Red Gate Tests` anchor matching Task 11 form.
4. F-LP18-OBS-001: No story fix required. Routes to existing process-gap codification candidate 5 tracking (4th recurrence confirmation). Not a new deferral.

**Post-fix-burst-18:** Dispatch adversary pass-19 against story v1.17 + BC-2.16.002 v1.12. Target: streak 0/3 → 1/3 if CLEAN.
