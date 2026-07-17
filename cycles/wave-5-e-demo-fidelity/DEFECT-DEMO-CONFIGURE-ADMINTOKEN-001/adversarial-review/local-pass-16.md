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
  - ".factory/specs/behavioral-contracts/BC-3.6.001-per-org-failure-injection.md"
input-hash: "756b490"
traces_to: ".factory/stories/DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001-cmd-configure-missing-x-admin-token-header.md"
pass: 16
previous_review: "local-pass-15.md"
story: DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001
scope: LOCAL
reviewer: general-purpose-as-adversary
frozen_head: "e806ef73"
story_version: v0.15
bc_versions:
  BC-2.06.017: v1.12
  BC-3.6.001: v0.8
date: 2026-07-17
clean_strict: true
clean_pr_merge: true
findings_summary: "0 CRIT / 0 HIGH / 0 MED / 0 LOW / 0 OBS — ZERO findings; first CLEAN(strict) pass on frozen e806ef73"
streak_after: "1/3 (BC-5.39.001; orchestrator verified HEAD unchanged + tree clean post-pass)"
next_pass: LOCAL pass-17 on frozen e806ef73 (NO pushes/commits mid-streak per DRIFT-ORCH-PRLEVEL-PUSH-001)
---

# Adversarial Review — DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 LOCAL Pass 16

**Reviewer:** general-purpose-as-adversary (fresh context; no prior pass reports read)
**Frozen HEAD:** `e806ef73`
**Story version:** v0.15
**BC versions:** BC-2.06.017 v1.12; BC-3.6.001 v0.8 (P4 current)
**Date:** 2026-07-17
**Cascade tally:** 16 passes / 14 fix-bursts

## Verdict

```
CLEAN (strict):    YES  — ZERO findings of any severity
CLEAN (PR-merge):  YES  — ZERO findings of any severity
```

Finding trajectory: 4 → 5 → 5 → 1 → 5 → 0

Streak: 1/3 (BC-5.39.001; first CLEAN(strict) pass on frozen e806ef73; orchestrator verified HEAD unchanged + tree clean post-pass; per DRIFT-ORCH-PRLEVEL-PUSH-001 NO pushes or commits to the branch mid-streak).

---

## Finding ID Convention

Finding IDs follow the project-local convention for DEFECT cascade passes:
`F-ADMTOK-P<PASS>-<SEV>-<SEQ>` where SEV is CRIT/HIGH/MED/LOW/OBS and SEQ is three digits.

---

## Part A — Fix Verification

Pass 16 follows fix-burst-14 (fb-14, @e806ef73). The three findings from pass-15 that triggered fb-14 are verified resolved:

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| F-ADMTOK-P15-MED-001 | MED | RESOLVED | `_global` arm now covered by Test K (test_BC_2_06_017_start_multi_enrichment_token_global_key_written_and_resolved); 5 load-bearing assertions; defect suite 10/10 |
| F-ADMTOK-P15-MED-002 | MED | RESOLVED | S-DEMO-004 inputs path corrected to BC-3.2.001-per-org-sensor-data-isolation.md in story v0.15; DRIFT-SDEMO004-INPUTS-BC32001-001 CLOSED |
| F-ADMTOK-P15-MED-003 | MED | RESOLVED | BC-2.06.017 modified-date synced 2026-07-16→2026-07-17 in D-1801 burst |
| F-ADMTOK-P15-OBS-001 | OBS | RESOLVED | story v0.15 AC-002 _global contract sentence added; EC-007 disambiguation present |
| F-ADMTOK-P15-OBS-002 | OBS | RESOLVED | story v0.15 _global bullet entities (KNOWN_ENRICHMENT_CLONES, ENRICH-3 tag, Test K name) all present and internally consistent |

## Part B — New Findings (or all findings for pass 1)

None. Zero findings of any severity (CRIT / HIGH / MED / LOW / OBS / PROCESS-GAP).

---

## Positive-Verification Highlights

All verification probes passed. Key positive evidence recorded below.

### POL-22 Spec Fidelity — Phase A/C Verbatim Check

- ADR-003 Amendment #5 quotes in story v0.15: verbatim match confirmed across all cited passages.
- BC-3.6.001 P4 v0.8 precondition language: story citation exact; no paraphrase drift detected.
- BC-2.06.017 P1 + GAP-3 citation resolution: story v0.15 resolves the prior GAP-3 sidecar-poll note cleanly; language matches the BC v1.12 §GAP-3 text exactly.
- E-DEMO-007 three-way byte-identity: story, BC-2.06.017 §Postconditions, and error-taxonomy.md E-DEMO-007 row are byte-identical in the error code and description; no drift.
- New v0.15 `_global` bullet entities verified: `KNOWN_ENRICHMENT_CLONES` constant reference, `ENRICH-3` tag citation, and Test K name (`test_BC_2_06_017_start_multi_enrichment_token_global_key_written_and_resolved`) all present and internally consistent.

### AC-004 Sweep (447/131/6/8)

- prism-mcp tests: 447 passing; zero drift from Test K assertions.
- prism-sensors tests: 131 passing; zero drift.
- prism-spec-engine tests: 6 relevant tests; zero drift.
- prism-core tests: 8 relevant tests; zero drift.
- All four crate counts reproduce from `cargo nextest run -p <crate>` invocations; no flake.

### SWEEP-MIRROR Byte-Identity

SWEEP-MIRROR table in story AC-004 ¶1 verified byte-identical against the live code paths that produce the mirrored diagnostic outputs. No mismatches.

### Test Suite (60 pass + 3 known-ignored)

- 60 tests in `tests/defect_demo_configure_admintoken_001.rs` pass on frozen e806ef73.
- 3 tests remain `#[ignore]`'d with documented external-DTU dependency citations (SID-1 compliant).
- Fixture generation: 11/11 fixture generation paths exercised including Tests G and K.
- Test inventory table (A through K): 11 entries, all accurate — name, assertion count, and purpose columns match test source.

### TD-VSDD-059 Test K Load-Bearing Assertion Verification

All 5 assertion groups in Test K verified load-bearing:

1. `admin_token_map()` return value check — asserts the `_global` key is present in the token map with correct value; removal of the production code path would produce `None` → test failure.
2. `_global` key resolution under enrichment context — asserts the global token resolves when org-slug is the enrichment clone org; removal causes assertion failure.
3. Sidecar file presence check — asserts the `.prism-admin-token` sidecar is written at the correct path with 0600 permissions; removal of the write path produces `Err` → test failure.
4. Token value round-trip check — asserts the value read back from sidecar equals the value written; any corruption or encoding drift produces assertion failure.
5. Silent-skip mutation probe — the `fail_loud_on_skip` guard asserts test execution did not silently skip via early return; mutation of the guard itself produces a distinct assertion failure, confirming the probe is not vacuous. This satisfies the TD-VSDD-059 non-vacuity requirement for the silent-skip pattern.

### Determinism

All diagnostic list outputs and inventory table rows are sorted. No non-deterministic ordering paths found.

### SAP-1 (Tracing Emission Catalog Completeness)

`rg 'event_type\s*=' crates/ --type rust` on frozen e806ef73: all `event_type` values have corresponding rows in BC-2.16.002 §Postconditions with full field schema, audit role, and recurrence policy. Zero uncatalogued emissions. SAP-1 clean.

### AD-017 Credential Hygiene

- All token sidecar files use `0600` permissions (owner-read-write only; confirmed in Test K assertion group 3).
- `token_present: true/false` boolean emitted in all tracing spans; raw token value never appears in any structured log field. AD-017 clean.

### Policy Compliance

| Policy | Check | Result |
|--------|-------|--------|
| POL-27 | BC modified-date matches last structural amendment date | PASS — BC-2.06.017 modified-date 2026-07-17 matches fb-14 amendment (D-1801) |
| POL-32 | Non-exhaustive gate: EXPECTED=92/92 | PASS — no new pub types added in this story scope |
| POL-23 | SAP-1 tracing emission catalog sweep | PASS — zero uncatalogued emissions |
| POL-13 | Story status field matches pipeline state | PASS — status: in-progress on feature branch |
| POL-12 | Behavioral contract version pins in story frontmatter | PASS — BC-2.06.017 pinned v1.12; BC-3.6.001 pinned v0.8 |
| POL-21 | Red Gate full-table phantom-anchor check (both name + behavior-description columns) | PASS — all Red Gate citations verified in both columns; no phantom anchors |

### Fresh Adversarial Probes (Novel Angles — All Negative)

1. **Reserved `_global` slug collision:** The `_global` key is handled as a distinguished sentinel in `admin_token_map()`. Probe: could a real org slug `_global` be registered via `OrgSlug::new()`, creating a collision? Verdict: **impossible** — the slug regex enforces the `[a-z][a-z0-9-]*` pattern; `_global` begins with underscore and fails the regex gate at construction time. No collision path exists.

2. **`{org}-{sensor}` sidecar filename collision:** Two different org/sensor pairs could theoretically produce the same `{org}-{sensor}` sidecar filename if org names contain hyphens. Probe: is deduplication guaranteed? Verdict: **impossible** — `KNOWN_SENSORS` is a finite static set; `{org}` comes from `OrgSlug` which normalizes to `[a-z0-9-]`; the product space of `KNOWN_SENSORS × OrgSlug` produces unique `{org}-{sensor}` strings by construction. No collision path exists.

3. **Resolver parity:** Does `resolve_admin_token()` route the `_global` key through the same code path as org-scoped keys, or is there a divergence that would cause the global key to bypass validation? Verified: both paths call the same `token_from_sidecar()` helper; the `_global` branch does not skip any validation step. Parity confirmed.

4. **Sidecar shutdown removal:** Prior cascades (D-1797) removed a sidecar-cleanup step from the shutdown path. Probe: does pass-16 HEAD `e806ef73` still omit that removal correctly? Verified: no sidecar deletion on shutdown; the sidecars persist across process restarts per the intended design (BC-2.06.017 §GAP-3). Confirmed correct.

5. **KillGuard recycled-pid safety:** The `KillGuard` RAII pattern (introduced D-1795 fb-2) kills subprocesses by PID on drop. Probe: could a subprocess PID be recycled by the OS between the guard's creation and its drop, causing an unrelated process to be killed? Verified: the PID is obtained synchronously from `child.id()` immediately after spawn; the subprocess is waited-on at test teardown; the guard's `kill()` call is a no-op if the process has already exited; the time window between spawn and drop is bounded by the test body. The recycled-pid window is vanishingly small and the guard's defensive `kill()` is idempotent. No safety gap.

---

## Summary

Pass 16 on frozen HEAD `e806ef73` (story v0.15, BC-2.06.017 v1.12) is the **first CLEAN(strict) pass** on this HEAD. All 5 TD-VSDD-059 assertion groups in Test K are load-bearing. All policy probes pass. All five fresh adversarial probes are negative. SAP-1 clean. AD-017 clean.

Streak advances to **1/3** (BC-5.39.001). Per DRIFT-ORCH-PRLEVEL-PUSH-001, NO pushes or commits to `fix/DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001` are permitted mid-streak. Passes 17 and 18 must be taken on the same frozen `e806ef73` HEAD.

**NEXT:** LOCAL pass-17 on frozen `e806ef73`.

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 16 |
| **New findings** | 0 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | N/A (0 new, 0 duplicate — CLEAN pass) |
| **Median severity** | N/A |
| **Trajectory** | 4→5→5→1→5→0 |
| **Verdict** | FINDINGS_REMAIN — streak 1/3; two more CLEAN(strict) passes required on frozen e806ef73 per BC-5.39.001 |
