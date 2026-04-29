# Prism Workspace Crate Layout

> **Status:** Enforced — `scripts/check-crate-layout.sh` runs on every pre-commit
> (via `lefthook.yml`) and on every push/PR (via `.github/workflows/crate-layout.yml`).
> See `ADR-012-src-convention.md` for design rationale. Traces to BC-3.7.001, S-3.5.01.

---

## 1. Canonical Crate Shape

Every workspace crate must follow this layout:

```
crates/<name>/
  Cargo.toml              # required — workspace crate manifest
  build.rs                # optional — Cargo-mandated build script location (see §3.2)
  fixtures/               # optional — test fixture data (NOT tests/fixtures/)
  src/
    lib.rs                # required for library crates (OR src/main.rs for binaries)
    main.rs               # required for binary crates; allowed alongside lib.rs
    tests/                # optional — dedicated unit-test submodule (src/tests/)
  tests/                  # optional — integration test files (e.g., tests/bc_*.rs)
  examples/               # optional — example binaries
  benches/                # optional — benchmark binaries
```

**Key invariants:**
- `src/lib.rs` or `src/main.rs` must exist (Rule 1).
- No loose `.rs` files at the crate root, except `build.rs` (Rule 2).
- Fixture data lives in `fixtures/` at the crate root, never in `tests/fixtures/` (Rule 3).

---

## 2. Layout Rules

| Rule | Description | Rationale | Checked By Script |
|------|-------------|-----------|-------------------|
| Rule 1 | `src/lib.rs` OR `src/main.rs` must exist | Canonical Rust crate entry point; prevents stray source trees | Yes |
| Rule 2 | No `.rs` files at crate root except `build.rs` | Prevents accidental module shadowing and confusion about code location | Yes |
| Rule 3 | Fixture data in `fixtures/` not `tests/fixtures/` | Fixtures are shared across unit and integration tests; `tests/` is for test binaries, not data | Yes |
| Rule 4 | Inline `#[cfg(test)]` in `src/` is permitted | Standard Rust pattern for unit tests adjacent to implementation | Not checked |
| Rule 5 | `src/tests/` submodule is permitted | Alternative to inline `#[cfg(test)]` for larger test suites | Not checked |
| Rule 6 | `tests/` directory is optional | Some crates (e.g., `prism-ocsf`) have no integration tests | Not checked |

**Enforcement:** `scripts/check-crate-layout.sh` checks Rules 1, 2, and 3.
Rules 4, 5, and 6 are structural conventions checked by code review only.

---

## 3. Exceptions

### 3.1 `prism-ocsf` — No `tests/` Directory

`prism-ocsf` does not have a `tests/` directory. This is conformant — `tests/` is
optional per ADR-012 §2.1 Rule 6. The crate generates Rust types from the OCSF schema
at build time (via `build.rs`) and its correctness is validated by the schema compiler
rather than integration tests.

`scripts/check-crate-layout.sh` does **not** flag the absence of `tests/` as a
violation. No crate is required to have a `tests/` directory.

This exception is documented explicitly to prevent false violation reports.
Traces to: AC-005, BC-3.7.001 postcondition 5, edge case EC-003.

### 3.2 `build.rs` at Crate Root

`build.rs` at any crate root is explicitly permitted and is excluded from Rule 2
("no loose `.rs` at crate root"). `build.rs` is the Cargo-mandated build script
location — it cannot be placed in `src/` or elsewhere.

Crates that currently have `build.rs`: `prism-ocsf` (schema code generation).

Traces to: AC-007, BC-3.7.001 postcondition 7, edge case EC-004, ADR-012 §7 OQ-1.

---

## 4. FAQ

**Q: Where do fixture files go?**

A: Fixture data (test input files: JSON, CSV, TOML, WASM, WAT, etc.) must live in
`crates/<name>/fixtures/` at the crate root. Do **not** put them in `tests/fixtures/`.
Path references in test code should use `concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/", <name>)`
for portability.

**Q: Where do integration tests go?**

A: Integration test files (`.rs` files that test the public API) go in
`crates/<name>/tests/`. Each file becomes a separate integration test binary.
Name them after the behavioral contract they validate (e.g., `tests/bc_2_16_001_test.rs`).

**Q: Inline unit tests or `src/tests/` submodule?**

A: Both are permitted:
- Inline `#[cfg(test)]` blocks in `src/` are standard Rust and are preferred for
  small unit tests that are tightly coupled to the implementation.
- A `src/tests/` submodule (e.g., `src/tests/mod.rs` with sub-files per BC) is
  preferred when the test suite is large (>200 lines) or when test-only helpers need
  to be shared across multiple test modules.

See `prism-core/src/tests/` for an example of the submodule pattern.

**Q: Is `tests/` required?**

A: No. `tests/` is optional. `prism-ocsf` is a conformant crate without a `tests/`
directory (see §3.1). The layout validator does not require its presence.

**Q: Where do example binaries go?**

A: In `crates/<name>/examples/`. Example files at the crate root or in `src/` are
not permitted — they shadow library modules. See `prism-spec-engine/examples/`
for a conformant example.

**Q: What happens if I put a `.rs` file at the crate root?**

A: The pre-commit hook will block your commit with a violation like:
```
crates/<name>: loose .rs file at crate root: helpers.rs (Rule 2 — move to src/)
```
Move the file to `src/` or remove it. `build.rs` is the only permitted exception.

---

## 5. 22-Crate Conformance Table

Generated by `scripts/check-crate-layout.sh --markdown` against the workspace
at S-3.5.01 implementation (all 22 crates pass).

| Crate | src/lib.rs | No loose .rs | fixtures/ clean | Status |
|-------|-----------|--------------|-----------------|--------|
| ocsf-proto-gen | OK | OK | OK | PASS |
| prism-audit | OK | OK | OK | PASS |
| prism-core | OK | OK | OK | PASS |
| prism-credentials | OK | OK | OK | PASS |
| prism-dtu-armis | OK | OK | OK | PASS |
| prism-dtu-claroty | OK | OK | OK | PASS |
| prism-dtu-common | OK | OK | OK | PASS |
| prism-dtu-crowdstrike | OK | OK | OK | PASS |
| prism-dtu-cyberint | OK | OK | OK | PASS |
| prism-dtu-demo-server | OK | OK | OK | PASS |
| prism-dtu-jira | OK | OK | OK | PASS |
| prism-dtu-nvd | OK | OK | OK | PASS |
| prism-dtu-pagerduty | OK | OK | OK | PASS |
| prism-dtu-slack | OK | OK | OK | PASS |
| prism-dtu-threatintel | OK | OK | OK | PASS |
| prism-mcp | OK | OK | OK | PASS |
| prism-ocsf | OK | OK | OK | PASS |
| prism-query | OK | OK | OK | PASS |
| prism-security | OK | OK | OK | PASS |
| prism-sensors | OK | OK | OK | PASS |
| prism-spec-engine | OK | OK | OK | PASS |
| prism-storage | OK | OK | OK | PASS |

To regenerate this table after adding a new crate:
```bash
just layout-report
```

---

## 6. Usage

### Running the Layout Validator

```bash
# Plain output (exit 0 = all conformant, non-zero = violations listed)
just check-layout

# Markdown table output for updating §5 of this document
just layout-report

# Run directly (WORKSPACE_ROOT override for testing)
WORKSPACE_ROOT=/path/to/fixture bash scripts/check-crate-layout.sh
```

### Pre-Commit Hook

```yaml
# From lefthook.yml — runs automatically when crates/** files are staged:
layout:
  glob: "crates/**"
  run: scripts/check-crate-layout.sh
```

The hook runs the full 22-crate workspace scan once per commit (not once per staged
file). Runtime is under 1 second per ADR-012 §5 analysis.

---

## 7. Adding a New Crate

Follow this checklist when creating a new crate to ensure it is conformant on first
commit:

1. Create `crates/<name>/Cargo.toml` and register in the workspace root `Cargo.toml`.
2. Create `crates/<name>/src/lib.rs` (or `src/main.rs` for a binary).
3. If you have test fixture data, create `crates/<name>/fixtures/` — do **not** use
   `tests/fixtures/`.
4. If you have a build script, it must be named `build.rs` at the crate root.
5. Verify conformance before committing:
   ```bash
   bash scripts/check-crate-layout.sh
   ```
   Or stage the files and let the pre-commit hook run automatically.

---

*Traces to: BC-3.7.001, ADR-012, VP-134, VP-135, VP-136.*
*Generated by S-3.5.01 implementation phase.*
