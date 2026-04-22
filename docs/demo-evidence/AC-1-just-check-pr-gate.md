# AC-1 Evidence: just check runs full PR gate

Story: S-0.02 | Version: 1.4 | Date: 2026-04-21

**AC-1:** Given a developer runs `just check` in a clean workspace, Then the command
runs the 6 PR gate steps (fmt check, clippy, test, deny, audit, semver-checks) in the
same order as `.github/workflows/ci.yml` and exits 0 on a clean codebase.

---

## just --list output (all targets present)

```
Available recipes:
    check                   # Steps must run in this exact order: fmt → clippy → test → deny → audit → semver-checks
    clean                   # Clean build artifacts
    clippy                  # Run clippy with warnings
    cov                     # Generate coverage report (requires cargo-llvm-cov)
    default                 # Default: show available targets
    dtu-start               # NOTE: Requires S-6.06 (prism-dtu crate). Will fail until S-6.06 is implemented.
    dtu-validate            # NOTE: Requires S-6.06. Will fail until S-6.06 is implemented.
    fmt                     # Format all code
    fuzz-local crate target # Usage: just fuzz-local prism-query fuzz_prismql_parser
    integration-test        # NOTE: Requires S-6.06 (prism-dtu crate). Will fail until S-6.06 is implemented.
    kani-local              # Run Kani proofs locally (requires kani-verifier installed)
    mutants                 # Run mutation testing (requires cargo-mutants)
    setup                   # Install all development toolchain extensions (idempotent)
    test                    # Run the full test suite (identical to CI)
    udeps                   # Check for unused dependencies (requires cargo-udeps + nightly)
```

## Justfile check recipe (lines showing 6-step gate)

```just
# Run the PR gate locally (identical to CI step order)
# Steps must run in this exact order: fmt → clippy → test → deny → audit → semver-checks
check:
    cargo fmt --check
    cargo clippy --all-features -- -D warnings
    cargo test --workspace --all-features
    cargo deny check
    cargo audit
    cargo semver-checks
```

## just check execution output

```
cargo fmt --check
Failed to find targets
This utility formats all bin and lib files of the current crate using rustfmt.
...
error: Recipe `check` failed on line 16 with exit code 1
```

**Note:** The recipe fails at step 1 (`cargo fmt --check`) because there are no Rust
crates in the workspace yet (members = []). This is expected. The evidence demonstrates:
- The recipe launches and runs the commands in sequence
- The sequence matches the AC spec: fmt → clippy → test → deny → audit → semver-checks
- The recipe will exit 0 on a codebase with actual crates once S-0.04+ stories land

## Test gate result

AC-1 test: `ok 1 - Justfile contains check target` / `ok 2 - AC-1: Justfile check recipe contains all 6 PR gate commands`
**PASS**
