# Evidence Report — S-1.08: Feature Flags (P0 Core)

**Story:** S-1.08  
**Branch:** feature/S-1.08-feature-flags  
**Policy:** POL-010  
**Date:** 2026-04-22  
**Test state:** 71/71 pass with `--no-default-features`

---

## Coverage Summary

| AC | Title | BC | Recording | Error Path |
|----|-------|----|-----------|------------|
| AC-1 | Compile-time gate returns Deny | BC-2.04.001, BC-2.04.004 | `AC-001-compile-time-gate-deny.gif` | Inherent (all paths are Deny by design) |
| AC-2 | Runtime TOML config grants Allow | BC-2.04.002 | `AC-002-runtime-allow.gif` | Covered in AC-1 (absent gate path) |
| AC-3 | Hidden tools pattern — write absent from list | BC-2.04.005 | `AC-003-hidden-tools-write-absent.gif` | Included in BC-2.04.005 test suite |
| AC-4 | list_capabilities returns effective matrix | BC-2.04.006 | `AC-004-list-capabilities.gif` | Unknown client → empty set (EC-004) |
| AC-5 | Audit event emitted for write checks | BC-2.04.013 | `AC-005-audit-event-emission.gif` | Audit failure does not affect gate (EC-003) |
| AC-6 | E-FLAG-001 structured error + resolution_trace | BC-2.04.015 | `AC-006-eflag001-structured-error.gif` | Empty trace minimum fields (EC-005) |
| AC-7 | VP-020 Kani proof | BC-2.04.004 | `AC-007-vp020-kani-harness.md` (Markdown) | N/A — formal proof |
| AC-8 | Hierarchical resolution — specific Deny wins | BC-2.04.003 | `AC-008-hierarchical-deny-override.gif` | Parent Allow + child Deny → Deny |

---

## VP-020 Two-Tier Truth Table

| Recording | Description |
|-----------|-------------|
| `VP-020-two-tier-truth-table.gif` | All 4 combinations of (compile gate) x (runtime flag): unit test counterpart to Kani harness |

---

## Feature-Combination Documentation

| Recording | Feature flags | Notes |
|-----------|--------------|-------|
| `FULL-SUITE-all-features.gif` | `--no-default-features` then `--features all-write` | Shows two-tier gate behavior under both combinations |

### Why Some BC-2.04.001 Tests Intentionally Fail with `--features all-write`

The four tests named `test_BC_2_04_001_real_*_write_gate_absent` verify that the
**real Cargo cfg gate** returns `CompileTimeGate::Absent` when the feature is not
compiled in. By design, they call the gate wrapper functions
(`crowdstrike_write_gate()`, etc.) and assert the result is `Absent`.

When compiled **with** `--features all-write`, those gate functions return `Present`
(the feature IS compiled in), so these specific tests fail — that is correct
behavior proving the two-tier gate actually tracks the compile-time state.

The remaining 67 tests pass under `--features all-write` because they model the
compile gate as a `CompileTimeGate` enum parameter rather than calling the real
cfg wrapper.

**Canonical test invocation:** `cargo test --no-default-features -p prism-security`
(71/71 pass)

---

## File Index

### VHS Recordings

| File | AC | Format |
|------|----|--------|
| `AC-001-compile-time-gate-deny.gif` | AC-1 | GIF (embed) |
| `AC-001-compile-time-gate-deny.webm` | AC-1 | WebM (archival) |
| `AC-001-compile-time-gate-deny.tape` | AC-1 | VHS source |
| `AC-002-runtime-allow.gif` | AC-2 | GIF |
| `AC-002-runtime-allow.webm` | AC-2 | WebM |
| `AC-002-runtime-allow.tape` | AC-2 | VHS source |
| `AC-003-hidden-tools-write-absent.gif` | AC-3 | GIF |
| `AC-003-hidden-tools-write-absent.webm` | AC-3 | WebM |
| `AC-003-hidden-tools-write-absent.tape` | AC-3 | VHS source |
| `AC-004-list-capabilities.gif` | AC-4 | GIF |
| `AC-004-list-capabilities.webm` | AC-4 | WebM |
| `AC-004-list-capabilities.tape` | AC-4 | VHS source |
| `AC-005-audit-event-emission.gif` | AC-5 | GIF |
| `AC-005-audit-event-emission.webm` | AC-5 | WebM |
| `AC-005-audit-event-emission.tape` | AC-5 | VHS source |
| `AC-006-eflag001-structured-error.gif` | AC-6 | GIF |
| `AC-006-eflag001-structured-error.webm` | AC-6 | WebM |
| `AC-006-eflag001-structured-error.tape` | AC-6 | VHS source |
| `AC-008-hierarchical-deny-override.gif` | AC-8 | GIF |
| `AC-008-hierarchical-deny-override.webm` | AC-8 | WebM |
| `AC-008-hierarchical-deny-override.tape` | AC-8 | VHS source |
| `VP-020-two-tier-truth-table.gif` | VP-020 | GIF |
| `VP-020-two-tier-truth-table.webm` | VP-020 | WebM |
| `VP-020-two-tier-truth-table.tape` | VP-020 | VHS source |
| `FULL-SUITE-all-features.gif` | All | GIF |
| `FULL-SUITE-all-features.webm` | All | WebM |
| `FULL-SUITE-all-features.tape` | All | VHS source |

### Markdown Documentation

| File | Purpose |
|------|---------|
| `AC-007-vp020-kani-harness.md` | AC-7: Kani proof structure, instructions, unit test counterpart |
| `evidence-report.md` | This file |

---

## Verification

All VHS recordings produced `.gif` and `.webm` outputs via `vhs 0.10.0`.
Font: `FiraCode Nerd Font Mono`. Theme: `Catppuccin Mocha`.

```
$ cargo test --no-default-features -p prism-security 2>&1 | grep "test result.*ok"
test result: ok. 8 passed ...   # bc_2_04_001_test
test result: ok. 7 passed ...   # bc_2_04_002_test
test result: ok. 10 passed ...  # bc_2_04_003_test
test result: ok. 8 passed ...   # bc_2_04_004_test
test result: ok. 8 passed ...   # bc_2_04_005_test
test result: ok. 8 passed ...   # bc_2_04_006_test
test result: ok. 8 passed ...   # bc_2_04_013_test
test result: ok. 7 passed ...   # bc_2_04_015_test
test result: ok. 7 passed ...   # vp_020_test
```
