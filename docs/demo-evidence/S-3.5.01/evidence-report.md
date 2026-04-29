# Demo Evidence Report — S-3.5.01

**Story:** S-3.5.01 — Workspace src/ convention sweep  
**Anchor BC:** BC-3.7.001  
**Track:** Platform Engineering  
**Recorded:** 2026-04-29  
**Tool:** VHS 0.10.0 (CLI recordings)  
**Branch:** feature/S-3.5.01  

---

## Coverage Map

| Demo ID | Acceptance Criterion | BC Postcondition | VP | Pass/Fail | Artifacts |
|---------|---------------------|------------------|----|-----------|-----------|
| AC-001 | All 22 workspace crates pass `check-crate-layout.sh` | PC-1 | VP-134 | PASS | [gif](AC-001-conformance-pass.gif) [webm](AC-001-conformance-pass.webm) [tape](AC-001-conformance-pass.tape) |
| AC-002 | Synthetic bad crate triggers non-zero exit + violation lines | PC-2, PC-3 | VP-135 | PASS | [gif](AC-002-violation-detection.gif) [webm](AC-002-violation-detection.webm) [tape](AC-002-violation-detection.tape) |
| AC-003 | `cargo test -p prism-spec-engine` green after fixture migration | PC-6 | VP-134 | PASS | [gif](AC-003-tests-green.gif) [webm](AC-003-tests-green.webm) [tape](AC-003-tests-green.tape) |
| AC-004 | TAP suite 24/24 passing (`tests/crate-layout-gate/run.sh`) | PC-1,2,3 | VP-134, VP-135, VP-136 | PASS | [gif](AC-004-tap-suite-green.gif) [webm](AC-004-tap-suite-green.webm) [tape](AC-004-tap-suite-green.tape) |

**Coverage: 4/4 must-demo criteria recorded (both success and error paths covered)**

---

## AC-001 — Conformance Check on Real Workspace

**Traces to:** BC-3.7.001 postcondition 1, VP-134  
**Command:** `bash scripts/check-crate-layout.sh`  
**Expected:** Exit 0, no violation output for any of the 22 workspace crates  
**Observed:** Exit 0, zero violation lines, message `All crates conform — exit 0`

**Recording:** `AC-001-conformance-pass.gif` / `.webm`

This demo demonstrates the success path: the entire workspace is conformant after the
`prism-spec-engine` fixture migration. The script is read-only and produces no output
on a clean workspace (only the echo confirming exit 0 from the shell).

---

## AC-002 — Violation Detection (Error Path)

**Traces to:** BC-3.7.001 postconditions 2+3, VP-135  
**Command:** `bash docs/demo-evidence/S-3.5.01/ac-002-run.sh`  
**Expected:** Non-zero exit; violation lines in `crates/<name>: <rule>` format  
**Observed:** Exit 1; two violation lines:

```
crates/test-bad-crate: missing src/lib.rs or src/main.rs (Rule 1)
crates/test-bad-crate: loose .rs file at crate root: lib.rs (Rule 2 — move to src/)
check-crate-layout: 2 violation(s) in 1 crate(s) checked.
exit code: 1
```

**Recording:** `AC-002-violation-detection.gif` / `.webm`  
**Helper script:** `ac-002-run.sh` (creates temp dir synthetically, cleans up via trap)

The synthetic bad crate has `lib.rs` at crate root with no `src/` directory. The script
detects both Rule 1 (missing `src/lib.rs` or `src/main.rs`) and Rule 2 (loose `.rs` at
crate root). The temp dir is created and cleaned up within the helper script, making
the tape fully reproducible on any machine.

---

## AC-003 — Cargo Test Suite GREEN

**Traces to:** BC-3.7.001 postcondition 6, VP-134  
**Command:** `cargo test -p prism-spec-engine 2>&1 | tail -20`  
**Expected:** 29 tests pass; 0 failed; fixture paths resolve from `fixtures/` (not `tests/fixtures/`)  
**Observed:** `test result: ok. 29 passed; 0 failed; 0 ignored`

**Recording:** `AC-003-tests-green.gif` / `.webm`

This demo confirms the fixture migration from `crates/prism-spec-engine/tests/fixtures/`
to `crates/prism-spec-engine/fixtures/` is complete and all 29 spec-engine tests pass.
All fixture path references in `crates/prism-spec-engine/tests/` use
`concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/<name>")` for portability.

---

## AC-004 — TAP Suite GREEN (24/24)

**Traces to:** BC-3.7.001 (VP-134, VP-135, VP-136)  
**Command:** `bash tests/crate-layout-gate/run.sh 2>&1 | tail -15`  
**Expected:** 24 tests pass; 0 failed; summary shows "Red Gate lifted"  
**Observed:**

```
# Total:   24
# Passed:  24
# Failed:  0
# Skipped: 0
# WARNING: All tests passed — Red Gate lifted. Implementation complete.
```

**Recording:** `AC-004-tap-suite-green.gif` / `.webm`

The TAP gate covers all VP claims:
- VP-134: all existing crates pass (AC-001, AC-005, AC-006, AC-007, AC-008 tests)
- VP-135: synthetic non-conformant crates trigger non-zero exit (AC-002, EC-002, EC-007 tests)
- VP-136: script is read-only — git status unchanged before/after (AC-008 test)

---

## Artifacts Inventory

| File | Type | Size | Purpose |
|------|------|------|---------|
| `AC-001-conformance-pass.tape` | VHS script | 784 B | Reproducible recording source |
| `AC-001-conformance-pass.gif` | GIF recording | 38 KB | PR embed |
| `AC-001-conformance-pass.webm` | WebM recording | 35 KB | Archival |
| `AC-002-violation-detection.tape` | VHS script | 961 B | Reproducible recording source |
| `AC-002-violation-detection.gif` | GIF recording | 74 KB | PR embed |
| `AC-002-violation-detection.webm` | WebM recording | 76 KB | Archival |
| `ac-002-run.sh` | Helper script | 1.0 KB | Synthetic bad-crate setup/teardown |
| `AC-003-tests-green.tape` | VHS script | 887 B | Reproducible recording source |
| `AC-003-tests-green.gif` | GIF recording | 146 KB | PR embed |
| `AC-003-tests-green.webm` | WebM recording | 387 KB | Archival |
| `AC-004-tap-suite-green.tape` | VHS script | 818 B | Reproducible recording source |
| `AC-004-tap-suite-green.gif` | GIF recording | 90 KB | PR embed |
| `AC-004-tap-suite-green.webm` | WebM recording | 216 KB | Archival |
| `evidence-report.md` | This file | — | Coverage mapping |

---

## Reproducibility Notes

- All tape files use absolute paths to the worktree for setup (hidden with `Hide`/`Show`).
- AC-002 uses `ac-002-run.sh` which creates a `mktemp -d` temp dir and cleans it up
  via `trap 'rm -rf ...' EXIT` — no hard-coded `/tmp` paths that won't survive migration.
- AC-003 records only the last 20 lines of cargo test output to keep the demo under 15s.
- AC-004 records only the last 15 lines of TAP output to show the summary without scrolling.
- Font: `FiraCode Nerd Font Mono` (confirmed installed at `/Users/jmagady/Library/Fonts/`).
- All recordings produced at 1200x600 or 1200x700 with Dracula theme, 20px padding.
