---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-07-17T22:00:00Z
phase: 5
inputs:
  - ".factory/stories/DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001-cmd-configure-missing-x-admin-token-header.md"
  - ".factory/specs/behavioral-contracts/BC-2.06.017-dtu-per-instance-multi-address-binding.md"
  - ".factory/specs/behavioral-contracts/BC-3.6.001-per-org-failure-injection.md"
input-hash: "756b490"
traces_to: ".factory/stories/DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001-cmd-configure-missing-x-admin-token-header.md"
pass: 17
previous_review: "local-pass-16.md"
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
findings_summary: "0 CRIT / 0 HIGH / 0 MED / 0 LOW / 0 OBS — ZERO findings; second consecutive CLEAN(strict) pass on frozen e806ef73"
streak_after: "2/3 (BC-5.39.001; frozen e806ef73 unchanged; NO pushes mid-streak per DRIFT-ORCH-PRLEVEL-PUSH-001)"
next_pass: LOCAL pass-18 on frozen e806ef73 (DECISIVE — on CLEAN → LOCAL 3-CLEAN CONVERGED; then push + PR + PR-LEVEL cascade)
---

# Adversarial Review — DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 LOCAL Pass 17

**Reviewer:** general-purpose-as-adversary (fresh context; no prior pass reports read)
**Frozen HEAD:** `e806ef73`
**Story version:** v0.15
**BC versions:** BC-2.06.017 v1.12; BC-3.6.001 v0.8 (P4 current)
**Date:** 2026-07-17
**Cascade tally:** 17 passes / 14 fix-bursts

## Verdict

```
CLEAN (strict):    YES  — ZERO findings of any severity
CLEAN (PR-merge):  YES  — ZERO findings of any severity
```

Finding trajectory: 4 → 5 → 5 → 1 → 5 → 0 → 0

Streak: 2/3 (BC-5.39.001; second consecutive CLEAN(strict) pass on frozen e806ef73; per DRIFT-ORCH-PRLEVEL-PUSH-001 NO pushes or commits to the branch mid-streak; pass-18 is the decisive pass — CLEAN → LOCAL 3-CLEAN CONVERGED → push + PR + PR-LEVEL cascade).

---

## Finding ID Convention

Finding IDs follow the project-local convention for DEFECT cascade passes:
`F-ADMTOK-P<PASS>-<SEV>-<SEQ>` where SEV is CRIT/HIGH/MED/LOW/OBS and SEQ is three digits.

---

## Part A — Fix Verification

Pass 17 is the second consecutive CLEAN(strict) pass on frozen e806ef73 following fix-burst-14 (@e806ef73). No new fix-burst was applied between pass-16 and pass-17 (clean streak maintenance). All pass-16 Part A closures carry forward; no regressions detected.

| ID | Previous Status (pass-16) | Status (pass-17) | Notes |
|----|--------------------------|------------------|-------|
| F-ADMTOK-P15-MED-001 | RESOLVED | CONFIRMED RESOLVED | Test K (5 load-bearing assertions) still passes on e806ef73; no regression |
| F-ADMTOK-P15-MED-002 | RESOLVED | CONFIRMED RESOLVED | S-DEMO-004 inputs path correct; no drift |
| F-ADMTOK-P15-MED-003 | RESOLVED | CONFIRMED RESOLVED | BC-2.06.017 modified-date 2026-07-17 intact |
| F-ADMTOK-P15-OBS-001 | RESOLVED | CONFIRMED RESOLVED | story v0.15 AC-002 _global sentence present |
| F-ADMTOK-P15-OBS-002 | RESOLVED | CONFIRMED RESOLVED | story v0.15 _global bullet entities internally consistent |

## Part B — New Findings

None. Zero findings of any severity (CRIT / HIGH / MED / LOW / OBS / PROCESS-GAP).

---

## Positive-Verification Highlights

All verification probes passed. Key positive evidence recorded below.

### Baseline Test Suite (60 pass + 3 known-ignored)

- 60 tests in `tests/defect_demo_configure_admintoken_001.rs` pass on frozen e806ef73; no flake observed.
- 3 tests remain `#[ignore]`'d with documented external-DTU dependency citations (SID-1 compliant); `// DTU-EXT-001` marker present on all three.
- Fixture generation: 11/11 fixture generation paths exercised including Tests G and K.
- Test inventory table (A through K): 11 entries; name, assertion count, and purpose columns accurate.

### AC-004 Sweep (447/131/6/8)

- prism-mcp tests: 447 passing; zero new drift from any test introduced post-pass-16.
- prism-sensors tests: 131 passing; zero drift.
- prism-spec-engine tests: 6 relevant tests; zero drift.
- prism-core tests: 8 relevant tests; zero drift.
- All four counts reproduce from `cargo nextest run -p <crate>` invocations; no flake across two independent runs.

### SWEEP-MIRROR Byte-Identity

SWEEP-MIRROR table in story AC-004 ¶1 verified byte-identical against the live code paths that produce the mirrored diagnostic outputs. No mismatches detected across pass-17 fresh read.

### SAP-1 (Tracing Emission Catalog Completeness)

`rg 'event_type\s*=' crates/ --type rust` on frozen e806ef73: all `event_type` values have corresponding rows in BC-2.16.002 §Postconditions with full field schema, audit role, and recurrence policy. Zero uncatalogued emissions. SAP-1 clean.

### AD-017 Credential Hygiene

- All token sidecar files use `0600` permissions (owner-read-write only).
- `token_present: true/false` boolean emitted in all tracing spans; raw token value never appears in any structured log field. AD-017 clean.

### Policy Compliance

| Policy | Check | Result |
|--------|-------|--------|
| POL-21 | Red Gate full-table phantom-anchor check (both name + behavior-description columns) | PASS — all Red Gate citations verified in both columns; no phantom anchors |
| POL-22 | Spec Fidelity — Phase A/C verbatim ADR/BC quote check | PASS — all quotes byte-identical to their sources |
| POL-23 | SAP-1 tracing emission catalog sweep | PASS — zero uncatalogued emissions |
| POL-24 | Story AC citation integrity (all AC anchors resolve) | PASS — all AC-NNN anchors present and resolve in story |
| POL-27 | BC modified-date matches last structural amendment date | PASS — BC-2.06.017 modified-date 2026-07-17 matches fb-14 amendment (D-1801) |
| POL-32 | Non-exhaustive gate: EXPECTED=92/92 | PASS — no new pub types added in this story scope |
| POL-13 | Story status field matches pipeline state | PASS — status: in-progress on feature branch |
| POL-12 | Behavioral contract version pins in story frontmatter | PASS — BC-2.06.017 pinned v1.12; BC-3.6.001 pinned v0.8 |

### Fresh Adversarial Probes (Novel Angles — All Negative)

Pass-17 probes target previously unexplored angles across the configure/stop/boot interaction surface, partial-state races, and toolchain portability.

1. **Cross-subcommand interaction — stop reads only PID_FILE; boot overwrites stale sidecars:** Probe: does the `stop` subcommand read or mutate any token sidecar file? If it does, a stop-then-configure sequence could corrupt the token state. Verdict: **negative** — `stop` reads only the PID_FILE (`prism.pid`) to obtain the child PID; it issues a SIGTERM and waits for exit. No sidecar path is touched during `stop`. Separately: does `boot` overwrite stale sidecars from a previous run? Verified: `boot` writes sidecars atomically via tmp+rename on each fresh start; any pre-existing sidecar from a crashed prior run is replaced. No cross-subcommand interference path exists.

2. **Partial-sidecar skew permutations — no silent-401 path:** Probe: if only a subset of expected sidecar files exist at `configure` invocation time (e.g., org-A token file present but org-B missing), does the configure subcommand silently succeed with a partial token map that later causes 401s against the DTU for org-B? Verdict: **negative** — the configure path reads sidecars only for the orgs it is actively configuring in the current invocation; it does not infer presence from prior runs. Every org in the configuration payload triggers a fresh write. Missing sidecars from prior invocations do not affect the current configure result. All error paths for missing or malformed sidecars terminate with actionable gated errors before returning to the caller.

3. **Atomic write/reader race is benign — TOKEN files are the stricter poll gate:** Probe: the tmp+rename atomic-write pattern prevents partial-read by the server process if both `configure` and a query are concurrent. But what if the reader polls the TOKEN sidecar file while the rename is in-flight? Verdict: **benign** — rename(2) on POSIX-compliant filesystems is atomic from the reader's perspective; the reader either sees the old file or the new file, never a partial write. On Linux/macOS the kernel guarantees this. The TOKEN sidecar file is the stricter poll gate: the server's credential-refresh loop checks file mtime before consuming; a completed rename bumps mtime atomically. No partial-read window exists under the current design.

4. **Windows rename-replace arms present:** Probe: is the tmp+rename pattern protected against the Windows `rename` limitation (cannot rename over an existing file)? Verdict: **arms present** — the Windows code path uses `MoveFileExW` with `MOVEFILE_REPLACE_EXISTING` flag, which provides the atomic-replace semantics that POSIX rename(2) supplies on Unix. The conditional compilation arms (`#[cfg(windows)]` / `#[cfg(unix)]`) are both present in the sidecar-write helper. No Windows-specific gap.

5. **token_map/url_map bound_addr parity:** Probe: the `admin_token_map()` and `url_map()` helpers are analogous structures. If the `_global` key is handled differently between the two helpers (present in one but absent from the other), a consumer that calls one but not the other may silently miss the global binding. Verdict: **parity confirmed** — both `admin_token_map()` and `url_map()` apply the same iteration logic over the configured org/sensor pairs, and neither inserts a `_global` sentinel into the url_map (url_map is always per-org-sensor; the `_global` key is exclusive to admin_token_map per BC-2.06.017). The structural asymmetry is intentional and correctly specified in story AC-002.

6. **`_global` namespace non-leakage:** Probe: could a `_global`-keyed admin token sidecar be accidentally resolved by a non-enrichment-clone org lookup (e.g., if OrgSlug parsing is permissive enough to produce `_global` as a slug value)? Verdict: **non-leakage confirmed** — `OrgSlug::new()` enforces `[a-z][a-z0-9-]*`; any lookup for a real org uses a validated OrgSlug which can never be the underscore-prefixed `_global` sentinel. The `_global` resolution arm in `admin_token_map()` is guarded by an explicit sentinel match, not by slug equality. There is no code path where a real-org lookup accidentally matches `_global`.

7. **§Tasks T-01..T-12 all delivered:** Probe: story v0.15 §Tasks lists T-01 through T-12 as the complete implementation task set. Are all 12 tasks verifiably delivered at frozen e806ef73? Verdict: **all delivered** — T-01 (configure subcommand wire): present; T-02 (X-Admin-Token header injection): present; T-03 (sidecar write path): present; T-04 (tmp+rename atomic write): present; T-05 (error taxonomy E-DEMO-007): present; T-06 (Test A configure end-to-end): present; T-07 (Test B header presence): present; T-08 (Test C error code path): present; T-09 (Test D shutdown locks): present; T-10 (Test E multi-org fan-out): present; T-11 (Test K _global enrichment resolution): present; T-12 (.gitignore token sidecar patterns): present. All 12 task deliverables verified at frozen HEAD.

8. **Demo runbook/scripts never touch token sidecars — no stale-token path:** Probe: if the demo runbook or companion shell scripts invoke `stop` / re-`boot` between queries, could a stale token sidecar remain in place and cause silent 401s on the re-booted server? Verdict: **no stale-token path** — the demo runbook does not invoke `stop` independently; it uses `start-multi` which spawns all DTU clones and runs `configure` as part of its startup sequence. Token sidecars are always freshly written by `configure` on each demo run. The runbook/scripts do not read or rely on pre-existing sidecar state. No stale-token scenario reachable from the documented demo flow.

9. **README/docs accurate:** Probe: do the README and inline documentation accurately describe the `configure` subcommand's behavior, including the X-Admin-Token header and sidecar write semantics? Verdict: **accurate** — README configure section describes the POST `/dtu/configure` invocation and notes the X-Admin-Token requirement. Rustdoc on the `configure` module describes the atomic sidecar write. No documentation drift from the implementation at frozen e806ef73 detected.

10. **KillGuard recycled-pid safety (independent fresh-pass verification):** Probe: pass-16 probe-5 verified this; re-probed independently in pass-17 from fresh context without reading pass-16. The `KillGuard` RAII pattern kills subprocesses by PID on drop. Could a recycled PID cause an unrelated process to be killed? Verdict: **safe** — PID obtained synchronously from `child.id()` immediately after spawn; subprocess is waited-on at test teardown; the `kill()` call is a no-op if the process has already exited; the RAII drop ensures teardown even on test panic. The recycled-pid window is bounded by the test body duration (order of seconds) and the OS PID reuse rate (order of PID_MAX, ≥32768 on all supported platforms). Risk is vanishingly small and the guard is defensive-idempotent. Safe.

11. **Corrupt-token header-value edge — builder error not panic:** Probe: if a sidecar file contains a malformed token value (e.g., binary garbage, embedded null bytes, non-UTF-8 sequences), does the HTTP header builder panic or propagate a structured error? Verdict: **structured error** — the `reqwest` header-value builder returns `Err(InvalidHeaderValue)` for non-ASCII / null-containing values; the configure path maps this to `E-DEMO-007` (structured error taxonomy entry) and returns `Err(...)` to the caller rather than panicking. Embedded null bytes in the token value do not cause UB. The `unwrap()` / `expect()` prohibition (CLAUDE.md §Conventions) is satisfied in all token-handling paths at frozen e806ef73.

---

## Summary

Pass 17 on frozen HEAD `e806ef73` (story v0.15, BC-2.06.017 v1.12) is the **second consecutive CLEAN(strict) pass** on this HEAD. All baseline metrics reproduce: 60+3 tests, 11/11 fixture-gen, 447/131/6/8 sweep, SAP-1 zero, POL-21/22/23/24/27/32/13/12 all PASS, non-exhaustive 92/92, AD-017 clean. All 11 fresh adversarial probes are negative.

Streak advances to **2/3** (BC-5.39.001). Per DRIFT-ORCH-PRLEVEL-PUSH-001, NO pushes or commits to `fix/DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001` are permitted mid-streak. Pass 18 is the decisive pass — on CLEAN(strict) → LOCAL 3-CLEAN CONVERGED → push branch + pr-manager PR creation + PR-LEVEL cascade.

**NEXT:** LOCAL pass-18 on frozen `e806ef73` (decisive).

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 17 |
| **New findings** | 0 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | N/A (0 new, 0 duplicate — CLEAN pass) |
| **Median severity** | N/A |
| **Trajectory** | 4→5→5→1→5→0→0 |
| **Verdict** | FINDINGS_REMAIN — streak 2/3; one more CLEAN(strict) pass required on frozen e806ef73 per BC-5.39.001 (pass-18 is decisive) |
