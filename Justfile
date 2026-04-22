# Justfile — S-0.02 Developer Toolchain Bootstrap
# TODO: S-0.02 — all targets are stubs; real logic added in implementation pass.

# Default: show available targets
default:
    @just --list

# Run the full test suite (identical to CI)
test:
    @echo "TODO: S-0.02 target test"
    @exit 1

# Run the PR gate locally (identical to CI step order)
check:
    @echo "TODO: S-0.02 target check"
    @exit 1

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

# Install all development toolchain extensions (idempotent)
setup:
    @echo "TODO: S-0.02 target setup"
    @exit 1
