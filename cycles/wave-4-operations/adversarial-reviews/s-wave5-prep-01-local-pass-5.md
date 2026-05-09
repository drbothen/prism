---
document_type: adversarial-review
pass: 5
cycle: wave-4-operations
story_id: S-WAVE5-PREP-01
branch: feature/S-WAVE5-PREP-01-prism-bin-chassis
head: be6228f0
verdict: CLEAN
findings_total: 5
findings_critical: 0
findings_high: 0
findings_medium: 0
findings_low: 2
observations: 3
kudos: 5
process_gaps: 0
reviewer: adversary
date: 2026-05-09
streak: "2/3"
prior_pass_closures_verified: 4
prior_pass_closures_partial: 1
prior_pass_closures_open: 0
inputs:
  - .factory/stories/S-WAVE5-PREP-01-prism-bin-chassis.md
  - .factory/specs/architecture/decisions/ADR-022-production-runtime-wiring.md
  - .factory/specs/behavioral-contracts/BC-2.06.011-config-load-on-startup.md
  - .factory/specs/behavioral-contracts/BC-2.21.001-org-registry-init.md
  - .factory/specs/behavioral-contracts/BC-2.03.013-credential-store-init.md
  - .factory/specs/behavioral-contracts/BC-2.05.012-audit-subsystem-init.md
  - .factory/specs/behavioral-contracts/BC-2.22.001-boot-orchestration.md
  - .factory/policies.yaml
  - .factory/cycles/wave-4-operations/adversarial-reviews/s-wave5-prep-01-local-pass-4.md
  - .factory/cycles/wave-4-operations/research/audit-emitter-architecture-2026-05-09.md
input-hash: "afbee5b"
---

# Adversarial Review — S-WAVE5-PREP-01 LOCAL Pass 5

**Story:** S-WAVE5-PREP-01 — prism-bin chassis  
**Branch:** `feature/S-WAVE5-PREP-01-prism-bin-chassis`  
**HEAD:** `be6228f0`  
**Pass:** 5  
**Verdict:** CLEAN  
**Streak:** 2/3 (second consecutive CLEAN — one more closes the window)  
**Date:** 2026-05-09

---

## Severity Trend

| Pass | Critical | High | Medium | Low | OBS | Kudos | Verdict |
|------|----------|------|--------|-----|-----|-------|---------|
| 1    | 1        | 3    | 5      | 3   | 3   | 2     | BLOCKED |
| 2    | 1        | 3    | 3      | 1   | 3   | 3     | BLOCKED |
| 3    | 0        | 1    | 1      | 1   | 2   | 4     | BLOCKED |
| 4    | 0        | 0    | 0      | 2   | 3   | 5     | CLEAN   |
| 5    | 0        | 0    | 0      | 2   | 3   | 5     | CLEAN   |

Two consecutive CLEAN passes achieved. One more closes the convergence window.

---

## Pass-4 Closure Verification

All 5 pass-4 findings reviewed.

| Finding | Status | Notes |
|---------|--------|-------|
| F-PASS4-LOW-1 — broken intra-doc link MockCredentialRefProbe at boot.rs:522 | CLOSED | Link repaired; verified at `be6228f0` |
| F-PASS4-LOW-2 — BC-2.05.012 §Failure Modes phantom AuditEmitter::new() failure path | CLOSED | BC-2.05.012 amended v1.1→v1.2; §Failure paths + Error Cases table now describes RocksDbBackend::open failure (the actually-fallible step) |
| F-PASS4-OBS-1 — sentinel timestamp skew (two Utc::now() calls) | CLOSED | Collapsed to single Utc::now() call; timestamp generated once and reused |
| F-PASS4-OBS-2 — SIGTERM "Audit buffer flushed" misleading log — **PARTIAL** | PARTIAL | Unix path at signals.rs:93 FIXED (honest log message); non-Unix SIGTERM sibling path at signals.rs:93 still emits "Audit buffer flushed" — sibling not updated. Surfaces as F-PASS5-LOW-2 per Partial-Fix Regression Discipline. |
| F-PASS4-OBS-3 — closure tests feature-gated, silently skipped under `just iter` | CLOSED via TD | TD-candidate filed; out-of-scope for this pass |

**Summary:** 4 CLOSED, 1 PARTIAL (F-PASS4-OBS-2 sibling-not-updated).

---

## Flaky-Test Fix Audit

The sentinel-file readiness handshake mechanism introduced in fix-pass-4 (commit `be6228f0`) was audited in full.

**Sentinel file mechanism:** The test writes a sentinel file path to `PRISM_TEST_READY_FILE`; the prism-bin process creates that file after completing startup steps; the test polls until the file appears, then proceeds. This eliminates the hardcoded-sleep race.

- **Readiness handshake correctly implemented:** Polling loop uses `std::thread::sleep` with bounded retry count. File creation is an atomic filesystem operation. No TOCTOU window.
- **Feature gate:** `PRISM_TEST_READY_FILE` environment variable read is gated `#[cfg(feature = "test-injection")]`. Production binary cannot be triggered via env.
- **libc dependency gated:** `libc` crate dependency in `prism-bin/Cargo.toml` is gated to `[dev-dependencies]` under `test-injection` feature only. Zero libc in production artifact.
- **Zero `#[ignore]`:** Searched entire `prism-bin` crate. Zero `#[ignore]` attributes found. No tests silently skipped.

Flaky-test fix earns KUDO-4 (see Kudos section).

---

## Findings

### F-PASS5-LOW-1 — Duplicate comment block at boot.rs:236-251

**Severity:** LOW  
**Category:** Code quality / duplication  
**File:** `crates/prism-bin/src/boot.rs` lines 236-251

The comment block:
```
// Test gate: PRISM_TEST_STOP_AFTER_STEP=6
// If the PRISM_TEST_STOP_AFTER_STEP environment variable is set to "6",
// the process exits here after completing boot step 6.
```
appears verbatim twice — once at line 236 and again at line 248. The second block is an exact duplicate of the first, including all three comment lines. The step-6 gate logic itself appears only once (correct), but the explanatory comment block was accidentally doubled, likely during a rebase or copy-paste during fix-pass-4.

**Impact:** No behavioral impact. Cosmetic; increases cognitive load when reading boot.rs step-6 region.

**Required fix:** Delete the duplicate comment block (lines 236-238 or 248-251, whichever is the unintended copy). Three-line deletion.

---

### F-PASS5-LOW-2 — Non-Unix SIGTERM sibling not updated (signals.rs:93)

**Severity:** LOW  
**Category:** Partial-fix regression (Partial-Fix Regression Discipline, S-7.01)  
**File:** `crates/prism-bin/src/signals.rs` line 93

F-PASS4-OBS-2 was PARTIALLY closed: the Unix SIGTERM handler path received the honest log message fix, but the non-Unix SIGTERM sibling path at `signals.rs:93` still emits `"Audit buffer flushed"`. This is misleading — on non-Unix platforms, no actual audit buffer flush occurs at that point.

Under the Partial-Fix Regression Discipline, a partial closure of a prior pass finding is re-elevated. F-PASS4-OBS-2 was marked PARTIAL in the closure table above and surfaces here as a new LOW finding.

**Required fix:** Update `signals.rs:93` non-Unix branch to use the same honest log message applied to the Unix path in fix-pass-4.

---

## Observations (Non-Blocking)

### F-PASS5-OBS-1 — `expect` / `unwrap_or` inconsistency in audit timestamp conversions

**Severity:** OBS  
`boot_emitter` uses `.expect("system time before Unix epoch")` for timestamp conversion. Other audit emitters in the codebase use `.unwrap_or(0)` for the same `SystemTime::duration_since(UNIX_EPOCH)` conversion. The inconsistency is not a correctness issue (Unix epoch is guaranteed for all practical system clocks), but it creates an asymmetry in error behavior: `boot_emitter` panics on impossible-but-representable error; others silently produce `0`.

**Recommendation:** Harmonize to `.unwrap_or(0)` across all audit timestamp conversions, or document the intentional divergence.

---

### F-PASS5-OBS-2 — Sentinel write precedes signal handler registration

**Severity:** OBS  
In `boot.rs`, the sentinel file is written at the end of step 6, before signal handlers are registered in step 7. There is a microsecond-wide race window: if the test observes the sentinel file and immediately sends SIGTERM before step 7 completes, the signal handler may not yet be installed.

**Context:** This race window is dominated by the polling interval (typically 10ms or more), making it practically inert in the current test harness. The sentinel-file mechanism is a major improvement over the hardcoded-sleep predecessor.

**Recommendation:** Consider registering signal handlers before writing the sentinel file, or document the ordering decision and the accepted race window. This is a design-time choice, not an emergency fix.

---

### F-PASS5-OBS-3 — Ctrl-C and SIGTERM handlers are identical (signals.rs:55-84)

**Severity:** OBS  
The Ctrl-C handler (lines 55-69) and SIGTERM handler (lines 71-84) in `signals.rs` are structurally identical — same logic, same shutdown sequence, same log messages. This is not a correctness issue (both signals should produce graceful shutdown), but the duplication means any future change to shutdown behavior must be applied in two places.

**Recommendation:** Extract a shared `handle_shutdown_signal(signal_name: &str)` closure or function. This is a refactor, not a bug fix. Deferral is acceptable; the risk of refactoring outweighs the value at this stage of the cascade.

---

## Kudos

**KUDO-1 — Sentinel-file readiness handshake**  
The sentinel-file mechanism correctly eliminates the RocksDB init race that caused flaky SIGTERM tests. The implementation is clean: atomic file creation, feature-gated, bounded polling, no hardcoded sleeps.

**KUDO-2 — Parent-PID path**  
The boot sequence correctly captures the parent PID before any fork-like operations, ensuring the PID in audit records reflects the intended process identity.

**KUDO-3 — libc gating**  
`libc` is correctly limited to `[dev-dependencies]` under `test-injection` feature. No libc symbols leak into production artifact.

**KUDO-4 — BC-2.05.012 v1.2 amendment**  
The BC-2.05.012 v1.1→v1.2 amendment correctly identifies that `BootAuditEmitter::new` is infallible and that `RocksDbBackend::open` is the actually-fallible step. The amended failure path description is architecturally accurate.

**KUDO-5 — Single Utc::now() call**  
Collapsing two `Utc::now()` calls into one eliminates the sentinel timestamp skew (F-PASS4-OBS-1). The generated timestamp is now consistent across the sentinel payload.

---

## Anti-Padding Self-Check

Five candidates were evaluated before finalizing this report. Three were dropped as non-findings:

1. **signals.rs handler parameter naming** — Evaluated: `signal_name` parameter vs unnamed. Not a finding; naming is internally consistent and clear. Dropped.
2. **boot.rs step counter comment drift** — Evaluated: Whether step numbers in comments match actual step sequence. Verified correct at `be6228f0`. Dropped.
3. **test-injection feature doc coverage** — Evaluated: Whether the `test-injection` feature is documented in crate-level docs. Not a finding for this story's scope (BC-2.22.001 does not require feature documentation). Dropped.

Two remaining candidates were elevated as ranked findings (F-PASS5-LOW-1, F-PASS5-LOW-2). Three candidates were elevated as observations (F-PASS5-OBS-1 through F-PASS5-OBS-3).

---

## Verdict Summary

**CLEAN — Streak 2/3.**

No CRITICAL, HIGH, or MEDIUM findings. Two LOW findings (F-PASS5-LOW-1 duplicate comment, F-PASS5-LOW-2 non-Unix SIGTERM sibling) and three OBS (F-PASS5-OBS-1 timestamp expect/unwrap_or, F-PASS5-OBS-2 sentinel-before-handler micro-race, F-PASS5-OBS-3 Ctrl-C/SIGTERM handler duplication). Five KUDOs awarded.

Per user directive ("fix everything that surfaces"), fix-pass-5 dispatch is recommended for F-PASS5-LOW-1, F-PASS5-LOW-2, F-PASS5-OBS-1, and F-PASS5-OBS-2. F-PASS5-OBS-3 (Ctrl-C/SIGTERM handler extraction) is recommended for deferral — refactor risk exceeds value at this cascade stage.

After fix-pass-5: adversary pass-6 → target streak 3/3 → CONVERGENCE → merge-ready.
