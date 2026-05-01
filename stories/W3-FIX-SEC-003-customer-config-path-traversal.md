---
story_id: W3-FIX-SEC-003
title: "prism-customer-config: path canonicalization + E-CFG-018 SpecPathTraversal rejection"
wave: 3.1
level: "L4"
target_module: prism-customer-config
subsystems: [SS-06]
priority: P0
depends_on: []
blocks: []
estimated_days: 1
points: 3
status: draft
document_type: story
version: "1.0"
producer: story-writer
timestamp: "2026-05-01T00:00:00Z"
input-hash: ""
inputs:
  - .factory/cycles/wave-3-multi-tenant/gate-step-d-security-review.md
  - .factory/specs/behavioral-contracts/BC-3.3.001-startup-rejects-st-shared-mode.md
  - .factory/specs/behavioral-contracts/BC-3.3.004-customer-config-startup-validation.md
  - .factory/specs/architecture/decisions/ADR-010-customer-config-schema.md
traces_to: []
cycle: "v1.0.0-greenfield"
epic_id: "E-3.5"
phase: 3
behavioral_contracts:
  - BC-3.3.001
  - BC-3.3.004
verification_properties: [VP-105, VP-106]
assumption_validations: []
risk_mitigations: []
anchor_bcs: [BC-3.3.001, BC-3.3.004]
anchor_capabilities: [CAP-009]
anchor_subsystem: ["SS-06"]
tdd_mode: strict
---

# W3-FIX-SEC-003: prism-customer-config — path canonicalization and E-CFG-018 SpecPathTraversal rejection

## Narrative

As a Prism security reviewer, I want the customer-config validator to reject any
`[[dtu]].spec` path that escapes the customer-config directory via `..` traversal,
so that an operator who loads a malicious customer TOML file cannot cause Prism to
read arbitrary files on the filesystem at startup.

## Objective

Gate Step D identified SEC-003 (HIGH, CWE-22, OWASP A01): the validator at
`crates/prism-customer-config/src/validator.rs:539-548` resolves `spec_path` via
`parent.join(spec_path)` with no canonicalization or boundary check. A path like
`../../../../etc/passwd` resolves to a valid filesystem path; if the traversal target
exists, the `resolved.exists()` check passes silently.

Two complementary controls are required:

1. **Pre-join rejection:** Check `spec_path` for `..` components (using
   `Path::components()` or a simple string check for `..`) before `parent.join()`.
   Return `E-CFG-018: SpecPathTraversal` immediately if any `..` component is found.
2. **Post-join boundary check:** After `parent.join(spec_path)`, call
   `resolved.canonicalize()` (which resolves symlinks) and verify the canonical path
   starts with the canonical customers directory prefix. Return `E-CFG-018` if not.

The new error code `E-CFG-018: SpecPathTraversal` must be added to the `ConfigError`
enum. Absolute paths in `spec_path` (paths starting with `/`) must also be rejected.

## Behavioral Contracts

| BC ID | Title | Relevant Clause |
|-------|-------|-----------------|
| BC-3.3.001 | Startup Rejects Security Telemetry DTU Type Declared with Shared Mode | General startup rejection posture — config validation must refuse dangerous inputs |
| BC-3.3.004 | Customer Config Validation Rejects Invalid Schema at Startup | R-CUST-015 (spec path not found) is extended; new rule for traversal. Postcondition "process exits code 1" and "all errors collected" apply to E-CFG-018 as to all other error codes |

## Acceptance Criteria

### AC-001: `..` traversal rejected with E-CFG-018 (traces to BC-3.3.004 postcondition on failure)
A customer TOML with `spec = "../../../../etc/passwd"` causes the validator to emit
`E-CFG-018: spec path '../../../../etc/passwd' traverses outside the customers directory`
and the process exits with code 1 before accepting any connections.

### AC-002: Absolute paths rejected with E-CFG-018 (traces to BC-3.3.004 postcondition on failure)
A customer TOML with `spec = "/etc/passwd"` (absolute path) causes `E-CFG-018` rejection.
Absolute spec paths are never valid — specs must be relative to the config file's parent.

### AC-003: Relative paths within the tree pass (traces to BC-3.3.004 postcondition on success)
A customer TOML with `spec = "sensors/claroty.toml"` (relative, no `..`, within tree) passes
the boundary check when the referenced file exists. Existing R-CUST-015 behavior for
non-existent files is unchanged.

### AC-004: Symlink escaping rejected (traces to BC-3.3.004 postcondition on failure)
A spec path that resolves to a valid file but whose canonical path (after symlink resolution)
escapes the customers directory is rejected with `E-CFG-018`. The `canonicalize()` call
catches symlinks that indirectly traverse the boundary.

### AC-005: E-CFG-018 is part of multi-error collection (traces to BC-3.3.004 invariant 1)
`E-CFG-018` errors are collected alongside other errors across all customer TOML files and
all are reported in a single startup pass. The process does not stop at the first
traversal finding.

### AC-006: New regression tests in `tests/path_traversal.rs` (traces to BC-3.3.004 postcondition on failure)
A new test file `crates/prism-customer-config/tests/path_traversal.rs` contains at least:
- `test_dotdot_traversal_rejected` — verifies AC-001
- `test_absolute_path_rejected` — verifies AC-002
- `test_relative_within_tree_passes` — verifies AC-003

### AC-007: Process does not start when traversal is detected — startup rejection posture (traces to BC-3.3.001 postcondition — startup rejects dangerous config)
A customer TOML containing a traversal `spec` path causes the process to exit with code 1
at startup, consistent with BC-3.3.001's broader startup-rejection posture for any config
that violates safety constraints. The startup rejection behavior is verified by AC-001.

## Tasks

1. Read `crates/prism-customer-config/src/validator.rs` lines 530-555 to understand
   the current spec path resolution code.
2. Read the `ConfigError` enum definition (likely in `crates/prism-customer-config/src/error.rs`
   or inline in `validator.rs`) to understand the existing error code structure.
3. Add `E-CFG-018` variant to `ConfigError`:
   ```rust
   SpecPathTraversal {
       file: PathBuf,
       spec_path: String,
       message: String,
   }
   ```
   with `Display` impl: `"{file}: E-CFG-018: spec path '{spec_path}' {message}"`.
4. Before `parent.join(spec_path)`, add a pre-join check:
   - If `spec_path` starts with `/` → push `E-CFG-018("absolute paths are not permitted")`.
   - If any component of `Path::new(spec_path)` is `..` → push `E-CFG-018("traverses outside the customers directory via '..' component")`.
   - Continue to next validation rule (multi-error collection).
5. After `parent.join(spec_path)` (in the `mode == "client"` path, only when pre-join
   check passed), add a post-join boundary check:
   - `let canonical_spec = resolved.canonicalize()` — if `Err` (file not found), fall
     through to existing R-CUST-015 handler.
   - `let canonical_base = config_path.parent().unwrap_or(Path::new(".")).canonicalize()`
   - If `!canonical_spec.starts_with(&canonical_base)` → push `E-CFG-018("resolves to a path outside the customers directory")`.
6. Create `crates/prism-customer-config/tests/path_traversal.rs` with the three tests
   from AC-006 plus an EC-003 symlink test (where feasible on the CI platforms — see Edge Cases).
7. Run `cargo test -p prism-customer-config --all-features` — all tests pass.
8. Open PR to `develop`.

## Architecture Mapping

| Component | Module | File(s) | Pure/Effectful |
|-----------|--------|---------|----------------|
| `validate_spec_path` (new helper) | prism-customer-config | `crates/prism-customer-config/src/validator.rs` | Effectful (filesystem `canonicalize`) |
| `ConfigError::SpecPathTraversal` | prism-customer-config | error definition file | Pure (data type) |
| Path traversal tests | prism-customer-config | `crates/prism-customer-config/tests/path_traversal.rs` | Effectful (creates temp dirs) |

**Subsystem anchor justification:** SS-06 (Client Configuration) owns this story's scope
because `prism-customer-config` is the config validation crate and this fix is entirely
within its validator module, per the ARCH-INDEX Subsystem Registry definition of SS-06.

**Dependency anchor justification:** `depends_on: []` — this fix is self-contained;
it touches only `prism-customer-config` and requires no other W3-FIX story to land
first. `blocks: []` — no other W3-FIX story depends on path traversal protection.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `spec = "../../../../etc/passwd"` where `/etc/passwd` exists | Pre-join `..` check fires before filesystem access; `E-CFG-018` emitted without touching `/etc/passwd` |
| EC-002 | `spec = "./sensors/claroty.toml"` (leading `./`, no `..`) | Passes pre-join check; `parent.join("./sensors/claroty.toml")` resolves correctly; boundary check passes if within tree |
| EC-003 | Symlink within `customers/` dir pointing to `/etc/passwd` | Pre-join check passes (no `..` in spec_path); post-join `canonicalize()` resolves the symlink target; `starts_with` check on canonical path fails; `E-CFG-018` emitted |
| EC-004 | `spec = "../../customers/other/sensors/claroty.toml"` (traverses up but stays in repo) | Pre-join `..` check fires; `E-CFG-018` regardless of whether canonical path is within customers — the `..` component itself is sufficient for rejection |
| EC-005 | `canonicalize()` fails because file does not exist (Windows, before R-CUST-015 check) | `canonicalize()` returns `Err`; skip the starts_with check; fall through to existing R-CUST-015 `!resolved.exists()` check which emits `E-CFG-015` (not `E-CFG-018`) |
| EC-006 | Config file is in the filesystem root (`/`), so `parent()` returns `None` → `Path::new(".")` | `parent.join(spec_path)` resolves relative to `.`; boundary check uses canonical `.` as base — still blocks traversal correctly |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| Pre-join `..` component check | pure-core | Pure: inspects `Path::components()` with no I/O |
| Post-join `canonicalize()` boundary check | effectful-shell | Performs filesystem stat + symlink resolution |
| `ConfigError::SpecPathTraversal` variant | pure-core | Data type definition |
| Test helpers (temp dir creation) | effectful-shell | Creates temporary directories with `tempfile` or `std::fs::create_dir_all` |

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~3 000 |
| BC files (2 BCs) | ~3 500 |
| `validator.rs` full file (~600 lines) | ~4 500 |
| `ConfigError` enum (~100 lines) | ~800 |
| New test file (create) | ~1 000 |
| Cargo output | ~500 |
| **Total** | **~13 300** |

Well within a single agent context window.

## Previous Story Intelligence

- **S-3.3.01** (PR delivering `prism-customer-config`) established the validator's
  multi-error collection pattern and the E-CFG-NNN code taxonomy. The E-CFG-018 code
  extends that taxonomy; verify that E-CFG-017 is the current highest code to avoid
  collisions (the BC lists codes up to E-CFG-016).
- **BC-3.3.004 R-CUST-014 / R-CUST-015** are the existing spec-path rules. E-CFG-018
  is a new rule that fires BEFORE R-CUST-015 — if traversal is detected, R-CUST-015
  does not also fire (do not double-report for the same spec path).

## Architecture Compliance Rules

- E-CFG-018 MUST be part of the multi-error collector — do not use `return Err(...)`;
  use `errors.push(ConfigError::SpecPathTraversal { ... })` and continue.
- The pre-join `..` check MUST fire before any filesystem I/O (`canonicalize`,
  `exists`). Do not read files you are about to reject based on their path structure.
- E-CFG-016 is the current highest spec-path-related error code; E-CFG-018 must not
  collide with E-CFG-017 if E-CFG-017 was reserved. Verify the enum before assigning
  E-CFG-018.
- Do NOT use `Path::canonicalize` on the spec path unless the file is expected to exist
  (i.e., after the pre-join check passes). On non-existent paths `canonicalize` returns
  an error on all platforms; handle this case gracefully.

## Library & Framework Requirements

| Library | Version (workspace pin) | Purpose |
|---------|------------------------|---------|
| std::path::{Path, PathBuf, Component} | std | Path component inspection, `canonicalize` |
| tempfile | workspace pin (dev-dep) | Create temp dirs for traversal tests |

No new runtime Cargo dependencies. `tempfile` may already be a dev-dependency of
`prism-customer-config`; confirm before adding.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-customer-config/src/validator.rs` | Modify | Add pre-join and post-join checks (lines ~530-555) |
| `crates/prism-customer-config/src/error.rs` (or inline) | Modify | Add `SpecPathTraversal` variant with E-CFG-018 code |
| `crates/prism-customer-config/tests/path_traversal.rs` | Create | Three required tests + optional symlink test |

## Forbidden Dependencies

- Do NOT introduce `regex` or any new parsing crate for path validation. `std::path`
  component inspection is sufficient.
- Do NOT use `std::fs::read` or any file-reading call in the path validation helper —
  only `canonicalize` (stat-only on most platforms).
