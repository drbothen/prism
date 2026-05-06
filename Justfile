# Justfile — S-0.02 Developer Toolchain Bootstrap
# TODO: S-0.02 — all targets are stubs; real logic added in implementation pass.

# Default: show available targets
default:
    @just --list

# Run the full test suite (identical to CI)
test:
    @echo "TODO: S-0.02 target test"
    @exit 1

# Run the full PR gate locally — fast feedback (5-8 min target)
# Steps: fmt → clippy → nextest (PROPTEST_CASES=100) → doctests → check-layout
# Skipped on local pre-push (run on CI only): cargo audit, cargo deny, cargo semver-checks
# Use 'just check-ci' to run identical to CI, or invoke 'just audit', 'just deny', 'just semver-checks' ad-hoc.
# NOTE: PROPTEST_CASES=100 in the recipe overrides any value set in your shell environment
# for the duration of the cargo nextest invocation.
# NOTE: cargo-nextest skips doctests by default; the separate --doc step covers them.
check:
    cargo fmt --check
    cargo clippy --all-features -- -D warnings
    PROPTEST_CASES=100 cargo nextest run --workspace --all-features --no-fail-fast
    PROPTEST_CASES=100 cargo test --workspace --all-features --doc
    @scripts/check-crate-layout.sh

# iter <crate>: TDD inner loop. PROPTEST_CASES=32 (8x less than default 256) for speed.
# WARNING: property-test failures during `iter` may not reproduce at full strength.
# Always run `just check` before pushing to verify with default cases.
#
# TDD iteration mode — single crate, fast feedback (target: <60s).
# Usage: just iter prism-query
#        just iter prism-query test_parser
# This is the recommended inner loop. Do NOT use `just check` during TDD —
# reserve it for pre-push verification.
iter crate test_filter='':
    PROPTEST_CASES=32 cargo nextest run -p {{crate}} {{test_filter}}

# Fast workspace check — lint only, no tests. Use to confirm the workspace
# still type-checks during a refactor sweep before running tests.
check-fast:
    cargo clippy --all-features -- -D warnings

# Generate a build-timings report for diagnostics. Outputs HTML at
# target/cargo-timings/cargo-timing.html. See research sidecar §7 for
# how to interpret the output.
timings:
    cargo build --workspace --all-features --timings
    @echo "Timings report: target/cargo-timings/cargo-timing.html"

# CI-only: identical to CI behavior (full-strength)
# Steps run in spec order: fmt → clippy → nextest → doctests → deny → audit → semver-checks → check-layout
check-ci:
    cargo fmt --check
    cargo clippy --all-features -- -D warnings
    cargo nextest run --workspace --all-features --no-fail-fast
    cargo test --workspace --all-features --doc
    cargo deny check
    cargo audit
    cargo semver-checks --workspace --baseline-rev origin/develop
    @scripts/check-crate-layout.sh

# Standalone: cargo audit (supply-chain advisories)
# Run manually ad-hoc or invoked by check-ci / CI pipeline.
audit:
    cargo audit

# Standalone: cargo deny (license + advisory + duplicates)
# Run manually ad-hoc or invoked by check-ci / CI pipeline.
deny:
    cargo deny check

# Standalone: cargo semver-checks (use before tagging a release)
# Also invoked by the lefthook pre-tag hook (lefthook >= 1.6).
semver-checks:
    cargo semver-checks --workspace --baseline-rev origin/develop

# Format all code
fmt:
    @echo "TODO: S-0.02 target fmt"
    @exit 1

# Run clippy with warnings
clippy:
    @echo "TODO: S-0.02 target clippy"
    @exit 1

# Generate coverage report (requires cargo-llvm-cov)
cov:
    @echo "TODO: S-0.02 target cov"
    @exit 1

# Run mutation testing (requires cargo-mutants)
mutants:
    @echo "TODO: S-0.02 target mutants"
    @exit 1

# Run a specific fuzz target locally (requires cargo-fuzz)
# Usage: just fuzz-local prism-query fuzz_prismql_parser
fuzz-local crate target:
    @echo "TODO: S-0.02 target fuzz-local"
    @exit 1

# Run Kani proofs locally (requires kani-verifier installed)
kani-local:
    @echo "TODO: S-0.02 target kani-local"
    @exit 1

# Clean build artifacts
clean:
    @echo "TODO: S-0.02 target clean"
    @exit 1

# Check for unused dependencies (requires cargo-udeps + nightly)
udeps:
    @echo "TODO: S-0.02 target udeps"
    @exit 1

# Run integration tests against DTU sensor stubs
# NOTE: Requires S-6.06 (prism-dtu crate). Will fail until S-6.06 is implemented.
integration-test:
    @echo "TODO: S-0.02 target integration-test"
    @exit 1

# Start DTU sensor stubs standalone for manual development use
# NOTE: Requires S-6.06 (prism-dtu crate). Will fail until S-6.06 is implemented.
dtu-start:
    @echo "TODO: S-0.02 target dtu-start"
    @exit 1

# Run DTU fidelity validation against the DTU fleet
# NOTE: Requires S-6.06. Will fail until S-6.06 is implemented.
dtu-validate:
    @echo "TODO: S-0.02 target dtu-validate"
    @exit 1

# Validate canonical src/ layout for all workspace crates (ADR-012 §2.4, BC-3.7.001).
# TODO(S-3.5.01 implementer): scripts/check-crate-layout.sh is a Red Gate stub that
# exits 1 until the real validation logic is written. Make the script pass the tests
# in tests/check_crate_layout_test.rs one rule at a time.
check-layout:
    @scripts/check-crate-layout.sh

# Generate markdown conformance table for docs/CRATE-LAYOUT.md §5 (ADR-012 §2.4).
# TODO(S-3.5.01 implementer): implement --markdown flag in check-crate-layout.sh.
layout-report:
    @scripts/check-crate-layout.sh --markdown

# Install all development toolchain extensions (idempotent)
setup:
    @bash scripts/dev-setup.sh
