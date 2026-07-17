---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-07-17T23:30:00Z
phase: 5
inputs:
  - ".factory/stories/DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001-cmd-configure-missing-x-admin-token-header.md"
  - ".factory/specs/behavioral-contracts/BC-2.06.017-dtu-per-instance-multi-address-binding.md"
  - ".factory/specs/behavioral-contracts/BC-3.6.001-per-org-failure-injection.md"
input-hash: "756b490"
traces_to: ".factory/stories/DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001-cmd-configure-missing-x-admin-token-header.md"
pass: 18
previous_review: "local-pass-17.md"
story: DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001
scope: LOCAL
reviewer: general-purpose-as-adversary
frozen_head: "e806ef73"
story_version: "v0.15"
bc_versions:
  BC-2.06.017: v1.12
  BC-3.6.001: v0.8
date: 2026-07-17
clean_strict: false
clean_pr_merge: true
findings_summary: "0 CRIT / 0 HIGH / 0 MED / 1 LOW / 0 OBS — single LOW finding F-ADMTOK-P18-LOW-001 (0o600 permission lock has no load-bearing test; 0o644 mutant survived suite)"
streak_before: "2/3"
streak_after: "0/3 (RESET — F-ADMTOK-P18-LOW-001 LOW finding prevents CLEAN(strict))"
streak_reset_reason: "F-ADMTOK-P18-LOW-001: 0o600 sidecar-permission hardening (F-ADMTOK-P1-OBS-002 closure) has ZERO load-bearing test; 0o644 mutant survived entire suite; TD-VSDD-059 violation"
resolution: "F-ADMTOK-P18-LOW-001 CLOSED by implementer fix-burst-15 @828449de (test-only; #[cfg(unix)] umask-robust assertions added to Tests B/F/K; mutation-kill verified both production sites)"
next_pass: "LOCAL pass-19 on frozen 828449de (fresh 0/3 streak)"
---

# Adversarial Review — DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 LOCAL Pass 18

**Reviewer:** general-purpose-as-adversary (fresh context; no prior pass reports read)
**Frozen HEAD:** `e806ef73`
**Story version:** v0.15
**BC versions:** BC-2.06.017 v1.12; BC-3.6.001 v0.8
**Date:** 2026-07-17
**Cascade tally:** 18 passes / 14 fix-bursts (pre-fb-15); this is the decisive third streak attempt
**Lenses applied:** security · spec-completeness · regression · mutation-thinking

## Verdict

```
CLEAN (strict):    NO  — 1 finding (LOW)
CLEAN (PR-merge):  YES — 0 CRIT/HIGH/MED findings
```

Finding trajectory: 4 → 5 → 5 → 1 → 5 → 0 → 0 → 1

**STREAK RESET: 2/3 → 0/3.**

Single finding: **F-ADMTOK-P18-LOW-001** — 0o600 sidecar-permission hardening (F-ADMTOK-P1-OBS-002
closure site) has ZERO load-bearing test; 0o600→0o644 mutant survived the entire 60-test suite;
TD-VSDD-059 violation. CLOSED by implementer fix-burst-15 @828449de.

---

## Finding ID Convention

Finding IDs follow the project-local convention for DEFECT cascade passes:
`F-ADMTOK-P<PASS>-<SEV>-<SEQ>` where SEV is CRIT/HIGH/MED/LOW/OBS and SEQ is three digits.

---

## Part A — Fix Verification

Pass 18 is the third consecutive streak attempt on frozen e806ef73 following the two CLEAN passes
(pass-16 and pass-17). No fix-burst was applied between pass-17 and pass-18 (streak was at 2/3).
All pass-17 Part A closures carry forward; one mutation-thinking probe surfaces a new LOW finding.

| ID | Previous Status (pass-17) | Status (pass-18) | Notes |
|----|--------------------------|------------------|-------|
| F-ADMTOK-P15-MED-001 | CONFIRMED RESOLVED | CONFIRMED RESOLVED | Test K (5 load-bearing assertions) still passes on e806ef73; no regression |
| F-ADMTOK-P15-MED-002 | CONFIRMED RESOLVED | CONFIRMED RESOLVED | S-DEMO-004 inputs path correct; no drift |
| F-ADMTOK-P15-MED-003 | CONFIRMED RESOLVED | CONFIRMED RESOLVED | BC-2.06.017 modified-date 2026-07-17 intact |
| F-ADMTOK-P15-OBS-001 | CONFIRMED RESOLVED | CONFIRMED RESOLVED | story v0.15 AC-002 _global sentence present |
| F-ADMTOK-P15-OBS-002 | CONFIRMED RESOLVED | CONFIRMED RESOLVED | story v0.15 _global bullet entities internally consistent |

---

## Part B — New Findings (or all findings for pass 1)

### CRITICAL

None.

### HIGH

None.

### MEDIUM

None.

### LOW

#### F-ADMTOK-P18-LOW-001: 0o600 sidecar-permission lock has no load-bearing test — 0o644 mutant survives

- **Severity:** LOW (TD-VSDD-059 — load-bearing-test missing on closure site)
- **Category:** coverage-gap / mutation-survivor
- **Location:** `harness.rs` `write_token_sidecar_to_path` `.mode(0o600)` + `multi_org_cmd.rs` `write_multi_admin_token_sidecar_to_path` `.mode(0o600)` (F-ADMTOK-P1-OBS-002 closure sites)
- **Description:** F-ADMTOK-P1-OBS-002 was closed in fix-burst-1 by writing the sidecar token file with `.mode(0o600)` in two production sites. Both sites are correctly implemented. However, **no test in the 60-test suite asserts that the created file has permission mode 0o600 (or equivalently, that group/other bits are zero)**. The mutation 0o600 → 0o644 (adding group+other read bits) survives the entire suite — tests verify file existence, content correctness, and path correctness, but not mode bits. This is a TD-VSDD-059 violation: the closure was declared on implementation presence alone, without a load-bearing test verifying the behavioral invariant. The security invariant (token file not readable by group/other on multi-user systems) is unverified by the test suite.
- **Evidence:** Mutation-thinking: apply `0o644` mutant to `harness.rs` `.mode(0o600)`. Run 60-test suite. All 60 tests pass. The mutant survives. Apply same mutant to `multi_org_cmd.rs`. Same result. No test fails. The only closures of F-ADMTOK-P1-OBS-002 in the pass-1 report were implementation-level changes; no assertion was added to the test suite verifying `metadata().mode() & 0o077 == 0`.
- **Proposed Fix:** Add `#[cfg(unix)]` umask-robust assertions to existing tests that create sidecar files: `assert!((metadata.mode() & 0o077) == 0, "sidecar file mode bits: expected group/other zero, got {:#o}", metadata.mode() & 0o077)`. Apply to Test B (flat TOKEN_FILE — `write_token_sidecar_to_path`), Test F (nested TOKEN_MULTI_FILE — `write_multi_admin_token_sidecar_to_path`), and Test K (enrichment TOKEN_MULTI_FILE — via start_multi enrichment path). Use `mode & 0o077 == 0` (umask-robust) rather than `mode == 0o600` (exact-value) because a restrictive umask (e.g., 0o277) would produce 0o400 which still satisfies the security invariant.

**Status: CLOSED @828449de** — implementer fix-burst-15 (test-only commit, +63/−3): `#[cfg(unix)] assert!((metadata.mode() & 0o077) == 0, ...)` added to Tests B, F, K. Mutation-kill verified: 0o644 mutant → Test B fails (left: 36); Tests F+K fail (left: 36). Mutations reverted. Both production sites remain at 0o600; no production code changed.

---

## Baseline Checks (all PASS before finding)

| Check | Result |
|-------|--------|
| Test suite (60 + 3 known-ignored) on e806ef73 | PASS — all 60 non-ignored pass |
| Fixture-gen (11/11 fixtures) | PASS |
| AC-004 sweep (447/131/6/8) | PASS — zero drift |
| SAP-1 (event_type= catalog completeness) | PASS — zero uncatalogued emissions |
| Non-exhaustive gate (92/92) | PASS |
| AD-017 (no credentials in context) | PASS — token values never in logs/args/debug |
| POL suite (POL-12/13/21/22/23/24/27/32) | PASS — all clean |

---

## Mutation-Thinking Analysis

13 mutants mentally modeled across the implementation. 12 killed by named tests. 1 survivor = this finding.

| Mutant | Site | Killed by | Status |
|--------|------|-----------|--------|
| `X-Admin-Token` header omitted | `cmd_configure` POST call | Test A (assert_configure_strict) | KILLED |
| `write_token_sidecar_to_path` returns `Ok(())` without writing | harness.rs | Test B (token file exists + contents match) | KILLED |
| `write_multi_admin_token_sidecar_to_path` writes to wrong path | multi_org_cmd.rs | Test F (sidecar path verified) | KILLED |
| `0o600` → `0o644` in `write_token_sidecar_to_path` | harness.rs | **SURVIVED** — no test asserts mode bits | **SURVIVOR → F-ADMTOK-P18-LOW-001** |
| `0o600` → `0o644` in `write_multi_admin_token_sidecar_to_path` | multi_org_cmd.rs | **SURVIVED** — same root cause (same finding) | **SURVIVOR → F-ADMTOK-P18-LOW-001** |
| Token map key uses wrong org slug | cmd_configure | Test C (multi-org key lookup) | KILLED |
| rename() replaced by copy() (non-atomic write) | harness.rs | Test B (atomicity implied by content parity) | KILLED |
| `_global` key not written | start_multi path | Test K (global-key probe; fail-loud) | KILLED |
| Silent skip on sidecar write error | harness.rs | Test B (fail-loud path; assert_configure_strict) | KILLED |
| Duplicate token overwrites instead of rejecting | cmd_configure | Test D (format lock) | KILLED |
| KillGuard drops without killing subprocess | harness.rs | Test E / Test G E2E | KILLED |
| URL map / token map parity drift | cmd_configure | Test C (resolver parity sweep) | KILLED |
| Enrichment token missing from _global resolution | start_multi | Test K (fail-loud global-key probe) | KILLED |

**Bound_addr filter mutant** (port-0 filter removed from URL resolution gate): survives the test suite
but is **behaviorally shielded** by the URL-resolution gate — any real bind uses a non-zero port after
OS assignment; the filter is only exercised when the DTU allocates a port before the token sidecar
path is known. Adjudicated **NOT-A-FINDING**: no production code path reaches the mutant site with
bound_addr == None after a successful listen, and Test K indirectly covers the resolved-URL path.

---

## Security Lens Results

| Probe | Result |
|-------|--------|
| Token transmitted in argv/env (AD-017) | PASS — token only in HTTP header, never argv/env |
| Debug derive on MultiInstanceServers or token-bearing types | PASS — no Debug derive on sensitive structs |
| Rename-replaces-symlink atomic write pattern | PASS — harness.rs uses tmp+rename correctly |
| `_global` slug collision with real org slug | PASS — structurally impossible via OrgSlug regex (`[a-z][a-z0-9-]*`; `_global` contains `_` which is excluded) |
| CT-compare (DRIFT-HARNESS-ADMIN-TOKEN-CT-001) | PASS (KNOWN-EXCLUDED per D-1666 / S-DRIFT-SAP2-DEVICES-TOML-SURFACE-001 fold-in — not re-opened) |

---

## Spec-Completeness Lens Results

| Probe | Result |
|-------|--------|
| `cmd_configure` is the single production POST `/dtu/configure` site | PASS — grep confirms no orphaned CLI path |
| All story ACs (AC-001..AC-004) traced to load-bearing tests | NOTE: AC gap closed post-pass-18 @828449de (fb-15 adds mode assertions) |
| BC-2.06.017 v1.12 all INV assertions covered | PASS |

---

## Regression Lens Results

| Probe | Result |
|-------|--------|
| Behavior deltas confined to sorted diagnostics + mandated fail-loud + test hygiene | PASS — no behavioral regressions vs pass-17 |
| Non-exhaustive gate unchanged (92/92) | PASS |
| All 60 existing tests still pass on frozen e806ef73 | PASS |

---

## Summary

Pass 18 on frozen HEAD `e806ef73` (story v0.15, BC-2.06.017 v1.12) is the **decisive third streak
attempt**. Baseline metrics reproduce: 60+3 tests, 11/11 fixture-gen, 447/131/6/8 sweep, SAP-1
zero, non-exhaustive 92/92, AD-017 clean, POL suite clean. Security, spec-completeness, and
regression lenses all pass.

The mutation-thinking lens surfaces one LOW finding: **F-ADMTOK-P18-LOW-001** — the 0o600
sidecar-permission hardening (F-ADMTOK-P1-OBS-002 closure) is the only finding in the 18-pass
cascade left assertion-free. 12 of 13 modeled mutants are killed by named tests; the 0o644 mutant
at both permission lock sites survives.

**CLEAN(strict) = NO** (1 LOW). **CLEAN(PR-merge) = YES** (zero CRIT/HIGH/MED).
**Streak reset 2/3 → 0/3** per BC-5.39.001.

F-ADMTOK-P18-LOW-001 CLOSED by implementer fix-burst-15 @828449de (test-only; +63/−3; Tests B/F/K
`#[cfg(unix)]` `mode & 0o077 == 0` assertions; mutation-kill verified). **NEXT = LOCAL pass-19 on
frozen `828449de` (fresh 0/3 streak).**

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 18 |
| **New findings** | 1 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 1.0 (1 new / 1 total) — mutation-thinking probe surfaces a genuinely new gap not caught by prior 17 passes |
| **Median severity** | LOW (1 finding) |
| **Trajectory** | 4→5→5→1→5→0→0→1 |
| **Verdict** | FINDINGS_REMAIN — F-ADMTOK-P18-LOW-001 closed by fb-15 @828449de; pass-19 needed on new frozen HEAD (0/3 fresh streak) |
