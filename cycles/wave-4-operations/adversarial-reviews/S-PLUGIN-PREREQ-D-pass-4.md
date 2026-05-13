---
document_type: adversarial-pass-report
target_artifact: S-PLUGIN-PREREQ-D
pass_N: 4
target_sha: 9d6289ad
base_sha: 95d46be2
verdict: BLOCKED-soft
streak: "0/3 → 0/3"
finding_summary:
  CRITICAL: 0
  HIGH: 0
  MEDIUM: 2
  LOW: 1
  OBS: 1
prior_passes: [pass-1, pass-2, pass-3]
prior_fix_bursts: [fix-burst-1, fix-burst-2, fix-burst-3]
trajectory_note: "16 → 8 → 6 → 4 (descending; closure regression on F-LP3-MED-002)"
producer: adversary (orchestrator-backfilled)
timestamp: 2026-05-13T10:30:00Z
---

# S-PLUGIN-PREREQ-D Adversary Pass 4 — BLOCKED-soft

**Target artifact:** S-PLUGIN-PREREQ-D story v1.3 at SHA 9d6289ad
**Base SHA:** 95d46be2
**Verdict:** BLOCKED-soft (2 MEDIUM findings; streak does not advance)
**Streak:** 0/3 → 0/3 (resets — 2 MED findings present)

## Pass-3 Closure Verification

| Finding | Verdict | Evidence |
|---|---|---|
| F-LP3-MED-001 | CONFIRMED CLEAN | Story v1.3 changelog v1.3 row present; Task 11 test names corrected (test_BC_2_17_007_* prefix). |
| F-LP3-MED-002 | PAPER-FIX — CLOSURE REGRESSION | D-468 claimed POL-20 sweep 100% complete but verification used UNANCHORED grep. Anchored regex `^(cycle-[0-9]+\|[0-9]{4}-[0-9]{2}-[0-9]{2})$` reveals 8 BCs still non-compliant. See F-LP4-MED-001. |
| F-LP3-LOW-003 | CONFIRMED CLEAN | AC-7 None-branch note added in story v1.3; AC-17 schema tightening covers Some() case; None-branch resolved. |
| F-LP3-LOW-004 | CONFIRMED CLEAN | TODO(S-4.08) tags disambiguated; closed tag renamed RESOLVED(S-PLUGIN-PREREQ-D). |
| F-LP3-OBS-005 | CONFIRMED CLEAN | Changelog v1.3 row "6/8 in-scope" note updated with accurate accounting note. |
| F-LP3-OBS-006 | CONFIRMED CLEAN | Architecture Compliance Rules row citing BC-2.17.005 removed; BC-2.17.007 substituted. |

**Summary:** 5/6 CONFIRMED CLEAN + 1 PAPER-FIX (F-LP3-MED-002 POL-20 workspace sweep — closure
claimed but verification was unanchored, missing 8 BCs with compound-suffix or opaque burst-ID
`introduced:` values).

## Findings (4)

### F-LP4-MED-001 — POL-20 Closure Regression: 8 BCs Non-Compliant After Claimed 100% Sweep (MEDIUM)

**Severity:** MEDIUM
**Routing:** state-manager (in-scope POL-20 migration + policies.yaml amendment)
**Policy violation:** POL-20 (bc_introduced_field_canonical_format)

The D-468 state-burst declared POL-20 workspace sweep "100% COMPLETE — ZERO remaining violations"
but the verification used an UNANCHORED grep:

```bash
grep -vE 'cycle-[0-9]+|[0-9]{4}-[0-9]{2}-[0-9]{2}'
```

This pattern substring-matches and false-greens on values that contain the pattern as a
non-anchored substring:

| BC | Current `introduced:` | Why unanchored grep missed it |
|----|----------------------|-------------------------------|
| BC-2.20.001 | `cycle-1-pass-80` | `cycle-1` substring matches `cycle-[0-9]+` |
| BC-2.20.002 | `cycle-1-pass-80` | Same |
| BC-2.20.003 | `cycle-1-pass-80` | Same |
| BC-2.20.004 | `cycle-1-pass-80` | Same |
| BC-2.20.005 | `cycle-1-pass-80` | Same |
| BC-2.06.011 | `"bundle-B-phase-B-1b-ss22-bcs-2026-05-08"` | Embedded date `2026-05-08` matches `[0-9]{4}-[0-9]{2}-[0-9]{2}` |
| BC-2.21.001 | `"bundle-B-phase-B-1b-ss22-bcs-2026-05-08"` | Same |
| BC-2.22.001 | `"redirect-option-d-2026-05-08"` | Embedded date `2026-05-08` |

**Required fix:** Anchored regex verification + migration of all 8:
- `cycle-1-pass-80` → `cycle-1` (pass suffix not part of cycle identifier)
- `"bundle-B-phase-B-1b-ss22-bcs-2026-05-08"` → `"2026-05-08"` (extract embedded ISO date)
- `"redirect-option-d-2026-05-08"` → `"2026-05-08"` (extract embedded ISO date)

### F-LP4-MED-002 — Story v1.3 Changelog Row POL-20 Accounting Inaccurate (MEDIUM)

**Severity:** MEDIUM
**Routing:** story-writer (in-story changelog amendment)
**Policy violation:** POL-7 (BC H1 is title source of truth — by extension, changelog entries must be factually accurate)

The v1.3 changelog row states "6 of 8 BCs in-scope migrated; 2 deferred (sibling story BCs)"
but the true accounting is:

- The 8 violations were NOT split 6/2 by scope; all 8 were missed by unanchored grep.
- BC-2.06.011, BC-2.21.001, BC-2.22.001 are not "sibling story BCs" in the sense of being
  out-of-scope — they are in-scope for any POL-20 sweep regardless of owning story.
- The 5 BC-2.20.x files were listed as closed but remained in violation (paper-fix).

The changelog entry misleads auditors reconstructing sweep history.

**Required fix:** Story-writer to amend changelog v1.3 row to accurately state: "0/8 BC
violations actually closed at D-466/D-467; all 8 closed at D-469/470/471 by state-manager
via anchored verification."

### F-LP4-LOW-003 — AC-7 None-Branch Under-Specified at BC Level (LOW)

**Severity:** LOW
**Routing:** story-writer (AC-7 prose clarification)

AC-7 specifies a None-branch when no plugins are registered, resolving to "platform-native
sensor behavior." BC-2.17.004 (Plugin Dispatch Fallthrough) does not specify the schema
contract for the None case — is it an empty list, null, or an error? AC-17 schema tightening
(fix-burst-3) covers the Some() case but the None-branch BC-level postcondition remains
under-specified. Low risk; implementation ambiguity only.

### F-LP4-OBS-004 — POL-20 verification_steps Does Not Require Anchored Regex (OBSERVATION)

**Severity:** OBS
**Routing:** state-manager (policies.yaml amendment)

POL-20's `verification_steps` do not explicitly require ANCHORED regex. The steps read
"if the value matches `cycle-N`" — ambiguous between substring match and exact match.
This ambiguity allowed the unanchored grep to appear compliant while missing 8 violations.
The policy should explicitly FORBID unanchored grep and specify the anchored regex form
`^(cycle-[0-9]+|[0-9]{4}-[0-9]{2}-[0-9]{2})$`.

## KUDOs (4)

1. **Trajectory efficiency:** 16→8→6→4 across 4 passes — consistent quality improvement;
   no CRIT/HIGH since pass-1 fix-burst.
2. **Fix-burst-3 quality (5/6):** 5 genuine closures with correct mechanism; only the POL-20
   paper-fix (unanchored grep) was a regression. Overall fix-burst quality above baseline.
3. **BC-2.17.007 content:** New BC authored in fix-burst-1 has solid postconditions,
   canonical test vectors, and correct error taxonomy anchoring — no drift found in 4 passes.
4. **Story AC coverage (AC-1..AC-16):** Comprehensive; AC-17 schema tightening is good
   defensive spec. The None-branch gap (F-LP4-LOW-003) is a refinement, not a failure.

## Process-Gaps

### PG-LP4-001 — POL-20 Verification Regex Unanchored (CLOSED by fix-burst-4)

POL-20 verification_steps did not require anchored regex, enabling the false-green.
Closed by policies.yaml v1.9→v1.10: verification_steps amended to explicitly require
anchored regex `^(cycle-[0-9]+|[0-9]{4}-[0-9]{2}-[0-9]{2})$` and FORBID unanchored
substring grep. Provides the shell one-liner for bulk verification.

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 4 |
| **New findings** | 4 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 4 / (4 + 0) = 1.00 |
| **Median severity** | MED (2 MED + 1 LOW + 1 OBS; median MED) |
| **Trajectory** | 16 → 8 → 6 → 4 |
| **Verdict** | FINDINGS_REMAIN |

All 4 findings are genuinely novel: F-LP4-MED-001/OBS-004 are a new detection class
(unanchored-grep false-green) not present in prior passes; F-LP4-MED-002 is the downstream
changelog inaccuracy cascade from MED-001; F-LP4-LOW-003 is a BC-level precision gap surfaced
by the AC-17 schema tightening in fix-burst-3 which raised the bar for the None-branch spec.
Trajectory 16→8→6→4: healthy descent but streak resets (2 MED present).

## Convergence Position

**Verdict:** BLOCKED-soft (2 MEDIUM findings; streak cannot advance)
**Streak:** 0/3 → 0/3

**Fix-burst-4 routing (parallel):**

- **State-manager track:** F-LP4-MED-001 (8 BC migrations + anchored verification) + F-LP4-OBS-004 (policies.yaml v1.10 amendment)
- **Story-writer track:** F-LP4-MED-002 (changelog v1.3 row correction) + F-LP4-LOW-003 (AC-7 None-branch clarification)

If both tracks close cleanly: pass-5 targets CLEAN → streak 0/3 → 1/3.
Two additional CLEAN passes (pass-6, pass-7) required for 3/3 CONVERGED under BC-5.39.001.
