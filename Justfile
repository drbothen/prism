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
# NOTE: RUSTFLAGS="" is set explicitly on the three cargo-compilation steps (clippy, nextest, doctest)
# so they share the same fingerprint cache. Without alignment, a RUSTFLAGS drift (e.g. a
# shell export) forces a full recompile between steps (see RUSTFLAGS alignment on ci.yml's
# nextest and doctest steps in the test job — mold-linker fingerprint-cache rationale).
# S-PERF-GATE-006: RUSTFLAGS="" aligns clippy's build fingerprint with the
# nextest/doctest steps for shared-cache reuse. See story S-PERF-GATE-006
# for rationale + measured savings.
check:
    cargo fmt --check
    RUSTFLAGS="" cargo clippy --all-features -- -D warnings
    RUSTFLAGS="" PROPTEST_CASES=100 cargo nextest run --workspace --all-features --profile prepush
    RUSTFLAGS="" PROPTEST_CASES=100 cargo test --workspace --all-features --doc
    @scripts/check-crate-layout.sh
    @scripts/check-non-exhaustive.sh

# iter <crate>: TDD inner loop. PROPTEST_CASES=32 (8x less than default 256) for speed.
# WARNING: property-test failures during `iter` may not reproduce at full strength.
# Always run `just check` before pushing to verify with default cases.
#
# TDD iteration mode — single crate, fast feedback (target: <60s).
# Usage: just iter prism-query
#        just iter prism-query test_parser
# This is the recommended inner loop. Do NOT use `just check` during TDD —
# reserve it for pre-push verification.
# NOTE: RUSTFLAGS="" keeps iter's nextest builds in the same RUSTFLAGS bucket as check,
# so `just iter` does not invalidate check's shared dependency cache on the RUSTFLAGS axis.
# (iter is single-crate/default-features, so this does not make iter → check rebuild-free.)
# See story S-PERF-GATE-006 for full rationale.
iter crate test_filter='':
    RUSTFLAGS="" PROPTEST_CASES=32 cargo nextest run -p {{crate}} {{test_filter}}

# Fast workspace check — lint only, no tests. Use to confirm the workspace
# still type-checks during a refactor sweep before running tests.
# NOTE: RUSTFLAGS="" keeps this clippy fingerprint aligned with `check`, so the
# edit → `just check-fast` → `just check` dev loop reuses clippy artifacts instead
# of re-checking. See story S-PERF-GATE-006 for full rationale.
check-fast:
    RUSTFLAGS="" cargo clippy --all-features -- -D warnings

# Generate a build-timings report for diagnostics. Outputs HTML at
# target/cargo-timings/cargo-timing.html. See research sidecar §7 for
# how to interpret the output.
timings:
    cargo build --workspace --all-features --timings
    @echo "Timings report: target/cargo-timings/cargo-timing.html"

# CI-equivalent: mirrors CI nextest behavior (full-strength, including all serial-group overrides).
# Steps run in spec order: fmt → clippy → nextest → doctests → deny → audit → semver-checks → shellcheck → check-layout
# --profile ci applies all [[profile.ci.overrides]] serial-group assignments (signal_handlers,
# adv_p02_e2e_pushdown_pipeline_test, bc_2_01_013_spec_driven_adapter). Without --profile ci,
# nextest runs under the default profile and silently skips those overrides.
check-ci:
    cargo fmt --check
    cargo clippy --all-features -- -D warnings
    cargo nextest run --workspace --all-features --no-fail-fast --profile ci
    cargo test --workspace --all-features --doc
    cargo deny check
    cargo audit
    cargo semver-checks --workspace --baseline-rev origin/develop
    @scripts/check-non-exhaustive.sh
    @scripts/check-crate-layout.sh
    # S-DEMO-003 AC-008: shellcheck gate for demo scripts.
    shellcheck scripts/demo-setup.sh scripts/demo-run.sh scripts/demo-teardown.sh

# S-DEMO-003 AC-008: standalone shellcheck for demo scripts.
# Run: just shellcheck-demo
shellcheck-demo:
    shellcheck scripts/demo-setup.sh scripts/demo-run.sh scripts/demo-teardown.sh

# Run the S-REL-001 release-gate TAP test suite (S-REL-001 AC-012, F-REL001-P2-001).
# Fails closed when actionlint is absent from PATH — the AC-011 test exits non-zero
# rather than skipping. Install actionlint locally via: brew install actionlint
# (NOT cargo install actionlint — actionlint is Go, not Rust; research U4).
# Usage: just test-release-gate
test-release-gate:
    @bash tests/release-gate/run.sh

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

# Verify #[non_exhaustive] types reject external struct-literal or exhaustive-match construction.
# Mirrors the CI `non-exhaustive-violation-compile-fail` job for local pre-push parity.
# Violations are split across src/enum_violations.rs and src/struct_violations.rs;
# the cargo --message-format=json path ensures all 46 violations are counted (not capped
# by rustc's per-file error limit). Update EXPECTED when adding/removing violations.
# (BC-2.01.013 AC-5 / F-LP2-OBS-001 S-PLUGIN-PREREQ-C)
check-non-exhaustive:
    @scripts/check-non-exhaustive.sh

# Verify that crates/prism-spec-engine/fixtures/error-taxonomy-snapshot.md matches
# the canonical E-SPEC-019..023 rows in .factory/specs/prd-supplements/error-taxonomy.md.
# No-ops with a warning when .factory/ is not mounted (CI default).
# PRR-008 fix: S-CONFIG-MULTI-TENANT-OVERRIDE-001 PR #155.
check-taxonomy-snapshot:
    @scripts/check-error-taxonomy-snapshot.sh

# Install all development toolchain extensions (idempotent)
setup:
    @bash scripts/dev-setup.sh

# Build the component_model_dispatch.prx fixture from source (F-PASS6-MED-001).
# Requires wasm-tools 1.248.0 (`cargo install wasm-tools --version 1.248.0`).
# Source files:
#   tests/fixtures/src/component_model_dispatch.wit
#   tests/fixtures/src/component_model_dispatch.core.wat
# Output:
#   tests/fixtures/component_model_dispatch.prx
# Full recipe documented in: tests/fixtures/src/component_model_dispatch.README.md
build-fixture-component_model_dispatch:
    wasm-tools component embed \
        --world dispatch-test \
        tests/fixtures/src/component_model_dispatch.wit \
        tests/fixtures/src/component_model_dispatch.core.wat \
        -o /tmp/component_model_dispatch.embedded.wasm
    wasm-tools component new \
        /tmp/component_model_dispatch.embedded.wasm \
        -o tests/fixtures/component_model_dispatch.prx
    @echo "Built tests/fixtures/component_model_dispatch.prx"
    @wasm-tools component wit tests/fixtures/component_model_dispatch.prx

# Build the crowdstrike-oauth2 .prx WASM plugin from Rust source (PLUGIN-MIGRATION-001-E).
#
# This recipe compiles the plugin Rust crate to wasm32-wasip1 cdylib, then wraps it
# with wasm-tools into a valid WASM Component binary (.prx).
#
# Prerequisites:
#   - Rust wasm32-wasip1 target: `rustup target add wasm32-wasip1`
#   - wasm-tools 1.248.0+: `cargo install wasm-tools --version 1.248.0`
#   - wasm-opt (optional, for size reduction): `brew install binaryen` or apt install binaryen
#
# Output:
#   crates/prism-spec-engine/plugins/crowdstrike-oauth2/crowdstrike-oauth2.prx
#
# The output .prx file is loaded by PluginRuntime::load_all_plugins at boot step 7.5.
# In CI, the pre-built .prx is checked into the repository for reproducible builds.
# Rebuild when changing src/lib.rs, wit/sensor-auth.wit, or manifest plugin.toml.
#
# Validation:
#   wasm-tools validate --features=component-model crowdstrike-oauth2.prx
#
# Story: PLUGIN-MIGRATION-001-E (F-LP1-MED-017 closure)
build-plugin-crowdstrike-oauth2:
    @echo "Building crowdstrike-oauth2 plugin (wasm32-wasip1 → Component)"
    # F-LP2-HIGH-003: fail-fast if wasi_snapshot_preview1.wasm is missing (required for --adapt).
    test -f tests/fixtures/wasi_snapshot_preview1.wasm || \
        (echo "ERROR: tests/fixtures/wasi_snapshot_preview1.wasm is missing (required for --adapt)"; exit 1)
    cargo build \
        -p crowdstrike-oauth2-plugin \
        --target wasm32-wasip1 \
        --release
    @echo "Lifting to WASM Component via wasm-tools..."
    # F-LP2-HIGH-003 / F-LPCI3-MED-001: --adapt path is required for WASI reactor → Component
    # lifting. The bare-wrap fallback produces a non-functional artifact (core module, not
    # component) so it is removed. Fail-fast on any error; stderr is preserved to the terminal.
    wasm-tools component new \
        target/wasm32-wasip1/release/crowdstrike_oauth2_plugin.wasm \
        --adapt wasi_snapshot_preview1=tests/fixtures/wasi_snapshot_preview1.wasm \
        -o crates/prism-spec-engine/plugins/crowdstrike-oauth2/crowdstrike-oauth2.prx || \
        (echo "ERROR: wasm-tools component new --adapt failed — build aborted"; exit 1)
    @echo "Validating Component Model binary..."
    # F-LP2-HIGH-003: validate exits non-zero on failure; positive assertion checks '(component'
    # in wasm-tools print output. Both gates exit 1 on failure (no silent fallthrough).
    wasm-tools validate \
        --features=component-model \
        crates/prism-spec-engine/plugins/crowdstrike-oauth2/crowdstrike-oauth2.prx || \
        (echo "ERROR: crowdstrike-oauth2.prx failed Component Model validation"; exit 1)
    wasm-tools print \
        crates/prism-spec-engine/plugins/crowdstrike-oauth2/crowdstrike-oauth2.prx | \
        grep -q '(component' || \
        (echo "ERROR: crowdstrike-oauth2.prx is a core WASM module, not a Component"; exit 1)
    @echo "PASS: crowdstrike-oauth2.prx is a valid WASM Component"
    # F-LP3-HIGH-001: verify all 3 required WIT sensor-auth exports are present in the component.
    # wit-bindgen's export!(Component) must emit auth-type-name, acquire-token, get-token
    # as kebab-case WIT export names (validate_wit_interface requires these; discovery.rs:26).
    wasm-tools print \
        crates/prism-spec-engine/plugins/crowdstrike-oauth2/crowdstrike-oauth2.prx | \
        grep -E '(auth-type-name|acquire-token|get-token)' | \
        grep -qE 'auth-type-name' || \
        (echo "ERROR: auth-type-name export absent from crowdstrike-oauth2.prx"; exit 1)
    wasm-tools print \
        crates/prism-spec-engine/plugins/crowdstrike-oauth2/crowdstrike-oauth2.prx | \
        grep -qE 'acquire-token' || \
        (echo "ERROR: acquire-token export absent from crowdstrike-oauth2.prx"; exit 1)
    wasm-tools print \
        crates/prism-spec-engine/plugins/crowdstrike-oauth2/crowdstrike-oauth2.prx | \
        grep -qE 'get-token' || \
        (echo "ERROR: get-token export absent from crowdstrike-oauth2.prx"; exit 1)
    @echo "PASS: crowdstrike-oauth2.prx has all 3 required sensor-auth WIT exports"
    @echo "Done: crates/prism-spec-engine/plugins/crowdstrike-oauth2/crowdstrike-oauth2.prx"

# S-DEMO-ENRICHMENT-PIVOT-002: Build prism-threatintel-infusion WASM plugin → .prx artifact.
# Pattern mirrors build-plugin-crowdstrike-oauth2 (U14/Ruling 4).
# Output: crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.prx
#         crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.manifest.toml
#
# HIGH-1 fix (S-DEMO-ENRICHMENT-PIVOT-002): the companion manifest is deployed alongside
# the .prx binary. PluginRuntime::load_all_plugins reads the manifest `name` field as
# the plugin_id — so the manifest `name = "threat_intel"` ensures the plugin registers
# under "threat_intel", matching infusion_id in specs/infusions/threatintel.infusion.toml.
build-plugin-threatintel-infusion:
    @echo "Building prism-threatintel-infusion plugin (wasm32-wasip1 → Component)"
    test -f tests/fixtures/wasi_snapshot_preview1.wasm || \
        (echo "ERROR: tests/fixtures/wasi_snapshot_preview1.wasm is missing (required for --adapt)"; exit 1)
    cargo build \
        --manifest-path crates/plugins/prism-threatintel-infusion/Cargo.toml \
        --target wasm32-wasip1 \
        --release
    @echo "Lifting to WASM Component via wasm-tools..."
    mkdir -p crates/prism-spec-engine/plugins/threatintel-lookup
    wasm-tools component new \
        crates/plugins/prism-threatintel-infusion/target/wasm32-wasip1/release/prism_threatintel_infusion.wasm \
        --adapt wasi_snapshot_preview1=tests/fixtures/wasi_snapshot_preview1.wasm \
        -o crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.prx || \
        (echo "ERROR: wasm-tools component new --adapt failed — build aborted"; exit 1)
    @echo "Validating Component Model binary..."
    wasm-tools validate \
        --features=component-model \
        crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.prx || \
        (echo "ERROR: threatintel-lookup.prx failed Component Model validation"; exit 1)
    wasm-tools print \
        crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.prx | \
        grep -q '(component' || \
        (echo "ERROR: threatintel-lookup.prx is a core WASM module, not a Component"; exit 1)
    @echo "Deploying companion manifest (HIGH-1: name = threat_intel matches infusion_id)..."
    cp crates/plugins/prism-threatintel-infusion/threatintel-lookup.manifest.toml \
        crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.manifest.toml
    @echo "Recording source tree hash for CI staleness gate (F-MCPNULL-P2-OBS-002)..."
    python3 scripts/hash-plugin-source.py crates/plugins/prism-threatintel-infusion \
        > crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.prx.src-tree-hash
    @echo "Source tree hash recorded."
    @echo "Done: crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.prx"
    @echo "Done: crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.manifest.toml"
    @echo "Done: crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.prx.src-tree-hash"

# S-DEMO-ENRICHMENT-PIVOT-002 v1.3: build-plugin-nvd-infusion REMOVED (ADR-040 D9).
# NVD enrichment is now served by InfusionType::HttpLookup (permanent built-in).
# The prism-nvd-infusion WASM plugin crate has been deleted.

