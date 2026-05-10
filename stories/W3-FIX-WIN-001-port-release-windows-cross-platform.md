---
story_id: W3-FIX-WIN-001
title: "prism-dtu-harness: cross-platform fix for drop_releases_ports test (Windows winsock)"
wave: 3
level: "L4"
target_module: prism-dtu-harness
subsystems: [SS-01]
priority: P0
depends_on: []
blocks: [WAVE-3-CLOSE]
estimated_days: 0.5
points: 2
status: merged
document_type: story
version: "1.0"
producer: story-writer
timestamp: "2026-04-30T17:00:00Z"
input-hash: "b909751"
inputs:
  - .factory/specs/behavioral-contracts/BC-3.5.001-harness-logical-isolation.md
  - crates/prism-dtu-harness/tests/logical_isolation_test.rs
traces_to: []
cycle: "v1.0.0-greenfield"
epic_id: "E-3.5"
phase: 3
behavioral_contracts:
  - BC-3.5.001
verification_properties:
  - VP-124
anchor_bcs: [BC-3.5.001]
anchor_capabilities: [CAP-036]
anchor_subsystem: ["SS-01"]
tdd_mode: strict
assumption_validations: []
risk_mitigations: []
---

# W3-FIX-WIN-001: prism-dtu-harness — cross-platform fix for drop_releases_ports test (Windows winsock)

## Narrative

As a Prism CI maintainer, I want `test_BC_3_5_001_drop_releases_ports` to pass on all 6 supported CI platforms (linux-gnu, linux-musl, darwin-x86_64, darwin-arm64, windows-msvc, no-default-features) so that BC-3.5.001 postcondition 4 is verified as a true cross-platform invariant rather than a Linux/macOS-only assertion.

## Problem Statement

Since S-3.3.03 (PR #101) merged, `test_BC_3_5_001_drop_releases_ports` in `crates/prism-dtu-harness/tests/logical_isolation_test.rs` (line 241) fails every Windows CI run deterministically.

**Root cause:** Windows winsock semantics differ from POSIX. When there is no listener on a port, `TcpStream::connect` does NOT return `ConnectionRefused` promptly. The OS either silently drops the SYN packet or returns a different error after a multi-second delay. The existing 1s `tokio::time::timeout` fires before the connect resolves, panicking with:

```
TCP connect to 127.0.0.1:<port> did not resolve within 1s after drop (TV-6; VP-124)
```

**Evidence:**
- 6 consecutive Windows CI runs, all failing at the same line (line 241), same panic message
- Random port assignment rules out port contention as a factor
- Linux/macOS/no-default-features matrix all pass
- This verifies BC-3.5.001 postcondition 4 / TV-6 / VP-124 ("drop releases ports"), which is a stated cross-platform invariant

**Wave gate impact:** Wave 3 close is blocked until this story merges.

## Behavioral Contracts

| BC ID | Title | Relevant Clause |
|-------|-------|-----------------|
| BC-3.5.001 | Harness Logical Isolation | Postcondition 4: drop releases ports (TV-6) |

## Acceptance Criteria

### AC-001: macOS pass (traces to BC-3.5.001 postcondition 4)
`test_BC_3_5_001_drop_releases_ports` passes locally on macOS after the fix is applied:
```
cargo test -p prism-dtu-harness --features dtu --test logical_isolation_test test_BC_3_5_001_drop_releases_ports
```
No test failures. No regressions in adjacent tests in `logical_isolation_test.rs`.

### AC-002: Windows CI pass (traces to BC-3.5.001 postcondition 4)
After merging to `develop`, the Windows CI runner (`windows-msvc` matrix leg) produces a green result for `test_BC_3_5_001_drop_releases_ports`. This is verified post-merge by inspecting the CI run.

### AC-003: No regressions on linux-gnu, linux-musl, darwin-arm64 (traces to BC-3.5.001 postcondition 4)
All three matrix legs that previously passed continue to pass after the fix. CI results for those legs remain green.

### AC-004: BC-3.5.001 postcondition 4 semantics preserved (traces to BC-3.5.001 postcondition 4)
The new assertion verifies the same semantic invariant — "after `drop(harness)` the port is no longer held" — using a platform-agnostic mechanism: attempting to bind a new `std::net::TcpListener` to the identical `SocketAddr`. If `bind` succeeds, the port was released. If `bind` fails with `AddrInUse`, the port is still held. The BC-3.5.001 / TV-6 / VP-124 traceability comments are preserved in the test body.

## Fix Design

### Why the Existing Pattern Fails on Windows

The test currently asserts:

```rust
TcpStream::connect(addr) -> ConnectionRefused  // passes on Linux/macOS
```

POSIX stacks return `ECONNREFUSED` (mapped to `ConnectionRefused`) immediately when the port has no listener. Windows winsock does not — it may timeout, silently drop, or return `WSAECONNREFUSED` only after a multi-second delay that exceeds the 1s timeout.

### Replacement Pattern: bind-after-drop

```rust
// CROSS-PLATFORM: instead of connect (winsock semantics break this),
// attempt to re-bind the same port. If bind succeeds the OS released
// the port on drop (TV-6; BC-3.5.001 postcondition 4; VP-124).
// If bind fails with AddrInUse, the port is still held — test panics.
let rebind = std::net::TcpListener::bind(addr);
match rebind {
    Ok(_listener) => {
        // port was released — test passes
    }
    Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
        panic!("port {addr} still bound after drop(harness) — drop did not release port (TV-6; BC-3.5.001 postcondition 4; VP-124)");
    }
    Err(e) => {
        panic!("unexpected error rebinding {addr} after drop(harness): {e} (TV-6; VP-124)");
    }
}
```

This pattern is semantically equivalent (proves the OS released the port) and works identically on all 6 platforms because `TcpListener::bind` returns `AddrInUse` consistently across POSIX and winsock when the port is still in use.

## Tasks (MANDATORY)

1. Open `crates/prism-dtu-harness/tests/logical_isolation_test.rs` and locate `test_BC_3_5_001_drop_releases_ports` (starts at line 202).
2. Replace the `tokio::time::timeout` + `TcpStream::connect` + `match connect_result { ... }` block (lines 223-243) with the `std::net::TcpListener::bind(addr)` rebind pattern described above.
3. Remove the now-unnecessary `tokio::time::sleep(100ms)` delay (line 221) — the rebind is synchronous and needs no wait. (Optional: keep a brief sleep as defensive padding; not required.)
4. Add a multi-line doc comment above the replacement block explaining the winsock incompatibility and why bind-after-drop is the canonical cross-platform assertion.
5. Preserve all existing BC/VP/TV traceability comments (`TV-6`, `BC-3.5.001 postcondition 4`, `VP-124`).
6. Run `cargo test -p prism-dtu-harness --features dtu --test logical_isolation_test test_BC_3_5_001_drop_releases_ports` locally on macOS — must pass (AC-001).
7. Open a PR to `develop` with the fix. CI must go green on all matrix legs (AC-002, AC-003).

## Architecture Mapping

| Component | Module | File | Pure/Effectful |
|-----------|--------|------|---------------|
| logical_isolation_test | prism-dtu-harness | `crates/prism-dtu-harness/tests/logical_isolation_test.rs` | Effectful (OS I/O) |

**Architecture Compliance:** SS-01 (DTU harness) owns this test. No module boundary changes required. This is a test-only change — no production code paths are modified.

**Subsystem anchor justification:** SS-01 owns this story's scope because `prism-dtu-harness` is the DTU harness subsystem and this fix is scoped entirely to its integration test file per the ARCH-INDEX Subsystem Registry.

**Dependency anchor justification:** `depends_on: []` — this fix has no blocking predecessors; it is a self-contained test correction. `blocks: [WAVE-3-CLOSE]` — Wave 3 cannot close until all CI matrix legs are green.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Port is still bound (drop failed silently) | `TcpListener::bind` returns `AddrInUse`; test panics with clear message citing TV-6 / BC-3.5.001 |
| EC-002 | OS assigns a different ephemeral port to a new process between drop and rebind | Bind succeeds on the original addr — test passes correctly (this is the happy path; the port is free) |
| EC-003 | `addr` was on `0.0.0.0` rather than `127.0.0.1` | Bind still succeeds/fails correctly; no special handling needed |
| EC-004 | Windows SO_REUSEADDR semantics allow bind before winsock fully frees the socket | `TcpListener::bind` does NOT set `SO_REUSEADDR` by default in std on Windows — rebind will fail with `AddrInUse` if port still held, correctly catching the error |

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~2 500 |
| BC files (1 BC: BC-3.5.001) | ~800 |
| Test file context (logical_isolation_test.rs ~300 lines) | ~2 000 |
| Cargo.toml for prism-dtu-harness | ~500 |
| Tool output (cargo test) | ~500 |
| **Total** | **~6 300** |

Well within the 20-30% context window limit for a 200k-token agent.

## Previous Story Intelligence

The stories in Epic E-3.3 that introduced this regression:
- **S-3.3.03 (PR #101):** Built harness logical isolation — introduced `test_BC_3_5_001_drop_releases_ports` using the connect-based pattern.
- **S-3.3.04 (PR #103), S-3.3.05 (PR #104):** Merged with the Windows regression undetected.

**Lesson:** The connect-based pattern is not cross-platform for port-release assertions. Always use bind-after-drop for this class of test going forward.

## Architecture Compliance Rules

- Do NOT modify any production source files — this fix is test-only.
- Do NOT add new Cargo dependencies. `std::net::TcpListener` is in `std`.
- Do NOT remove the `#[tokio::test]` attribute — the test must remain async-compatible even though the rebind assertion is synchronous.
- Keep the `--features dtu` gate intact; the test requires the `dtu` feature flag.

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| prism-dtu-harness (test) | effectful-shell | Integration test performs OS-level TCP bind; not pure |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| tokio | (workspace pin) | `#[tokio::test]` runtime; no new tokio APIs required by this fix |
| std::net::TcpListener | std (no external crate) | Synchronous rebind assertion — stdlib only |

No new external dependencies are introduced by this fix.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-dtu-harness/tests/logical_isolation_test.rs` | Modify | Replace connect-based assertion (lines 223-243) with bind-after-drop assertion |

No new files created. No other files modified.

---

## DRIFT NOTE — Windows Failures Undetected for 4 PRs

**PRs affected:** S-3.3.03 (#101), S-3.2.08 (#102), S-3.3.04 (#103), S-3.3.05 (#104)

**Root cause (process):** The `develop` branch has no required-checks gate for the `windows-msvc` CI matrix leg. `pr-manager` dispatches during those merges waved off Windows failures as "non-required", allowing the regression to accumulate silently across 4 PRs and 6 deterministic Windows failures.

**Corrective action (going forward):** `pr-manager` dispatches MUST flag any Windows CI test failure as **blocking** unless the failing test is explicitly marked `#[cfg(not(target_os = "windows"))]` or the story frontmatter contains an explicit `platform_exclusions: [windows]` field. Waving off Windows failures without an explicit platform-gate justification is not acceptable. This policy applies to all matrix legs (linux-gnu, linux-musl, darwin-x86_64, darwin-arm64, windows-msvc, no-default-features).

**Recommended follow-up:** Add a branch protection rule or CI gate that treats the `windows-msvc` leg as required-to-pass before merge on `develop`. Track as tech debt if branch protection is not immediately configurable.
