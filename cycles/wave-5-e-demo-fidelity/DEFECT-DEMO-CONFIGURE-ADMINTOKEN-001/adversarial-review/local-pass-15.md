---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-07-17T00:00:00Z
phase: 5
inputs:
  - ".factory/stories/DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001-cmd-configure-missing-x-admin-token-header.md"
  - ".factory/specs/behavioral-contracts/BC-2.06.017-dtu-per-instance-multi-address-binding.md"
  - ".factory/specs/behavioral-contracts/BC-3.6.001-dtu-admin-token-authentication.md"
input-hash: "8b30b8f"
traces_to: ".factory/stories/DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001-cmd-configure-missing-x-admin-token-header.md"
pass: 15
previous_review: "local-pass-14.md"
story: DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001
scope: LOCAL
reviewer: general-purpose-as-adversary
frozen_head: "803db300"
story_version: v0.14
bc_versions:
  BC-2.06.017: v1.12
date: 2026-07-17
clean_strict: false
clean_pr_merge: false
findings_summary: "0 CRIT / 0 HIGH / 2 MED / 0 LOW / 2 OBS — _global arm zero coverage + POL-23 sweep miss + POL-27 date mismatch + 2 stale inventory cells + EC-007 collision"
streak_after: "0/3 (new HEAD e806ef73 after fb-14; DRIFT-ORCH-PRLEVEL-PUSH-001 streak reset)"
next_pass: LOCAL pass-16 on frozen e806ef73
---

# Adversarial Review — DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 LOCAL Pass 15

**Reviewer:** general-purpose-as-adversary (fresh context; no prior pass reports read)
**Frozen HEAD:** `803db300`
**Story version:** v0.14
**BC versions:** BC-2.06.017 v1.12; BC-3.6.001 (current)
**Date:** 2026-07-17
**Cascade tally:** 15 passes / 13 fix-bursts (pass-15 on story v0.14 + BC-2.06.017 v1.12)

## Verdict

```
CLEAN (strict):    NO   — 2 MED + 2 OBS present
CLEAN (PR-merge):  NO   — 2 MED present
```

Finding trajectory: 4 → 5 → 5 → 1 → 5

Streak: 0/3 (reset by fb-14 push at e806ef73 per DRIFT-ORCH-PRLEVEL-PUSH-001).

---

## Finding ID Convention

Finding IDs follow the project-local convention for DEFECT cascade passes:
`F-ADMTOK-P<PASS>-<SEV>-<SEQ>` where SEV is CRIT/HIGH/MED/LOW/OBS and SEQ is three digits.

---

## Positive Verifications (highlights)

- **Sweep counts reproduce at HEAD:** `write_multi_admin_token_sidecar_to_path` 447 grep hits, `write_token_sidecar_to_path` 131 hits, `token_map()` 6 hits, `TOKEN_MULTI_FILE` 8 hits — all four commands reproduce the reported values at `803db300`.
- **SWEEP-MIRROR byte-identity:** Disposition table in story AC-004 matches the code across all three artifacts (story, code, pass-14 report) — byte-identical counts confirmed.
- **Defect suite 10/10 GREEN:** `cargo nextest run -p prism-dtu-demo-server -E 'test(defect)'` — all 10 targeted defect tests pass on frozen 803db300. Fixture-gen Test G also GREEN.
- **Determinism sweep — zero unsorted user-facing lists:** All `collect()` hits in `harness.rs` are either `.sorted()` before use or are not user-facing.
- **Non-exhaustive gate 92/92 PASS:** `scripts/check-non-exhaustive.sh EXPECTED=92` exits 0.
- **POL-21/22/24/13/32/12 all CLEAN.**
- **AD-017 credential-opacity CLEAN:** No token values in story text; diagnostics emit key-names-only.
- **SAP-1 tracing-emission catalog CLEAN:** No new `event_type=` emissions in the fix/DEFECT branch relative to develop.

---

## Part A — Fix Verification (pass >= 2)

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| F-ADMTOK-P14-OBS-001 | OBS | RESOLVED | Test J row added to `## Test inventory` table in `tests/defect_demo_configure_admintoken_001.rs` at fb-13 @803db300. Inventory now 10/10 rows present. |

All pass-14 findings resolved. No partial resolutions or regressions detected from pass-14 closures.

---

## Part B — New Findings (or all findings for pass 1)

### CRITICAL

_None._

### HIGH

_None._

### MEDIUM

#### F-ADMTOK-P15-MED-001: `_global` enrichment-token arm of `write_multi_admin_token_sidecar_to_path` has ZERO test coverage despite being contract-mandated

- **Severity:** MED
- **Category:** coverage-gap
- **Location:** `crates/prism-dtu-demo-server/src/harness.rs` — `write_multi_admin_token_sidecar_to_path()` `_global` enrichment-token arm; BC-2.06.017 v1.12 §Postconditions PC-3 / ENRICH-3
- **Description:** The `_global` org-slug path inside `write_multi_admin_token_sidecar_to_path` is contract-mandated as part of the URL_MULTI_FILE mirror invariant (both the URL sidecar and the enrichment-token sidecar must emit a `_global` key when the enrichment token is present). The URL twin has a dedicated lock test (`test_enrich3_sidecar_emits_global_key_for_enrichment`) that verifies this path. The enrichment-token sidecar equivalent has ZERO load-bearing assertions for the `_global` arm: no test verifies that `_global` is written, no test verifies that `_global` is resolved by the harness bootstrap, and no test exercises the silent-skip path (the arm exists but could silently no-op with no observable failure).
- **Evidence:** `grep -n "_global" crates/prism-dtu-demo-server/src/` shows the arm present in implementation; `grep -n "global" crates/prism-dtu-demo-server/tests/defect_demo_configure_admintoken_001.rs` returns zero assertions on the `_global` enrichment-token write path. The URL mirror test (`test_enrich3_sidecar_emits_global_key_for_enrichment`) is the precise analog that is absent for the enrichment-token sidecar.
- **Proposed Fix:** Add Test K covering: (1) `_global` key written to disk, (2) key resolvable by `resolve_admin_token_from_sidecar`, (3) fail-loud probe that the silent-skip path does NOT trigger when enrichment-token present, (4) fail-loud global-key probe that `_global` is the EXACT key emitted (not an alias), (5) defect suite GREEN post-Test-K. Add Test K inventory row to story.

#### F-ADMTOK-P15-MED-002 [process-gap]: POL-23 sweep miss — S-DEMO-004 `(ACTIVE vX.Y)` formatting evades policy grep pattern

- **Severity:** MED (process-gap)
- **Category:** spec-fidelity
- **Location:** `.factory/stories/S-DEMO-004-multi-org-sensor-isolation-smoke-test.md` — live BC-2.06.017 pin formatted as `BC-2.06.017 (ACTIVE v1.10)` instead of `BC-2.06.017 v1.12`
- **Description:** S-DEMO-004 contains a live pin for BC-2.06.017 formatted as `BC-2.06.017 (ACTIVE v1.10)`. The POL-23 sweep targets the bare-string pattern `BC-2.06.017 vX.Y`; the `(ACTIVE v...)` wrapper makes the pin invisible to the policy lint. The result: 2 prior fix-bursts and 14 adversary passes have missed this stale pin. Current BC-2.06.017 version is v1.12; story pin is v1.10 — 2 versions stale.
- **Evidence:** `grep "BC-2.06.017" .factory/stories/S-DEMO-004-multi-org-sensor-isolation-smoke-test.md` returns `BC-2.06.017 (ACTIVE v1.10)`. `grep "BC-2.06.017 v1.10" .factory/stories/` returns nothing (the bare-string pattern misses it). POL-23 sweep on this story therefore gives false-negative.
- **Proposed Fix:** Story-writer: update S-DEMO-004 pin from `(ACTIVE v1.10)` to `v1.12`; remove `(ACTIVE ...)` wrapper format from all BC-version-pin sites in S-DEMO-004 to ensure future POL-23 sweeps are effective.

### LOW

_None._

### Observations

#### F-ADMTOK-P15-OBS-001: Test inventory "Finding closed" cells stale for Tests D and F

- **Severity:** OBS
- **Location:** `.factory/stories/DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001-cmd-configure-missing-x-admin-token-header.md` — Test Inventory table, Tests D and F
- **Description:** The Test Inventory table rows for Test D (format lock assertion) and Test F (SWEEP-MIRROR load-bearing assertion) retain stale "Finding closed by" cell values that reference earlier fix-burst numbers and do not reflect their final closure state after fb-12.
- **Proposed Fix:** PO: update Test D and Test F inventory cells to reflect correct final closure status.

#### F-ADMTOK-P15-OBS-002: within-story EC-007 identifier collision in AC-002

- **Severity:** OBS
- **Location:** `.factory/stories/DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001-cmd-configure-missing-x-admin-token-header.md` — AC-002 §Error Conditions
- **Description:** AC-002 §Error Conditions contains two distinct EC-007 rows with different semantics, creating an ambiguous error catalog within the story.
- **Proposed Fix:** PO: renumber second EC-007 to EC-008; update cross-references.

---

## Part C — Process-Gap Finding (not a code/spec defect)

#### F-ADMTOK-P15-MED-003 [process-gap]: POL-27 date-mismatch — BC-2.06.017 frontmatter `modified: 2026-07-16` vs changelog row 1.12 dated `2026-07-17`

- **Severity:** MED (process-gap)
- **Policy anchor:** POL-27 — BC frontmatter `modified` must match latest changelog row date
- **Location:** `.factory/specs/behavioral-contracts/BC-2.06.017-dtu-per-instance-multi-address-binding.md` — frontmatter `modified: "2026-07-16"`; §Changelog v1.12 row: `2026-07-17`
- **Description:** The BC frontmatter `modified` field was not advanced when the v1.12 changelog row was authored on 2026-07-17. POL-27 requires frontmatter date to match the latest changelog row date. Metadata sync only — no version bump needed.
- **Proposed Fix:** State-manager: update frontmatter `modified: "2026-07-16"` → `modified: "2026-07-17"`. Note in pass-15 report resolution and D-1801 row.

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 2 (+ 1 process-gap = 3 total MED-class) |
| LOW | 0 |
| OBS | 2 |

**Overall Assessment:** block — 2 MED + 1 MED[process-gap] findings require fix-burst before next pass
**Convergence:** FINDINGS_REMAIN — iterate
**Readiness:** requires fix-burst fb-14

## Resolution Summary

| Finding | Severity | Closed By | Commit/Version |
|---------|----------|-----------|----------------|
| F-ADMTOK-P15-MED-001 | MED | implementer | @e806ef73 (Test K + 5 assertions; defect suite 10/10) |
| F-ADMTOK-P15-OBS-001 | OBS | PO | story v0.15 (inventory cells synced) |
| F-ADMTOK-P15-OBS-002 | OBS | PO | story v0.15 (EC-007 renumbered to EC-008) |
| F-ADMTOK-P15-MED-002 | MED [process-gap] | story-writer | S-DEMO-004 v1.15 (pin v1.10→v1.12; (ACTIVE) wrapper removed) |
| F-ADMTOK-P15-MED-003 | MED [process-gap] | state-manager | BC-2.06.017 frontmatter modified-date sync (this burst) |

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 15 |
| **New findings** | 5 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 5 / (5 + 0) = 1.00 |
| **Median severity** | MED |
| **Trajectory** | 4→5→5→1→5 |
| **Verdict** | FINDINGS_REMAIN |
