# Demo Evidence Report — W3-FIX-CI-001
## CI Wall-Clock Optimization (CAP-DEV-SPEED)

**Story:** W3-FIX-CI-001  
**Date:** 2026-04-30  
**Branch:** fix/W3-FIX-CI-001  
**Implementation Commit:** f027503b

---

## Coverage Summary

| AC | Description | Type | Evidence |
|----|-------------|------|----------|
| AC-001 | cargo nextest: 2363 tests pass | VHS | [AC-001-nextest-2363-pass.gif](AC-001-nextest-2363-pass.gif) / [.webm](AC-001-nextest-2363-pass.webm) |
| AC-002 | Doctest split on linux-gnu only | Doc | YAML snippet below |
| AC-003 | Per-platform PROPTEST_CASES matrix | Doc | YAML snippet below |
| AC-004 | mold linker step (Linux only) | Doc | YAML snippet below |
| AC-005 | Justfile parity: both targets use nextest | VHS | [AC-005-justfile-nextest-parity.gif](AC-005-justfile-nextest-parity.gif) / [.webm](AC-005-justfile-nextest-parity.webm) |

---

## AC-001 — cargo nextest: 2363 tests pass locally

**Recording:** `AC-001-nextest-2363-pass.gif` / `AC-001-nextest-2363-pass.webm`  
**Tape:** `AC-001-nextest-2363-pass.tape`

Demonstrates `cargo nextest run --workspace --all-features --no-fail-fast` producing the
summary line `2363 tests run: 2363 passed, 6 skipped` on the local workstation.
The `.config/nextest.toml` `[profile.ci]` profile is NOT used for local runs (omit `--profile ci`);
CI adds `--profile ci` to enable JUnit output and the slow-timeout flag.

---

## AC-002 — Doctest split runs on linux-gnu only

**Type:** Doc-only (CI YAML structural proof)

`cargo nextest` does not run doctests. The CI matrix gates the separate `cargo test --doc` step
behind `if: matrix.run_doctests`, which is only set to `true` on the `x86_64-unknown-linux-gnu`
leg. All other targets omit the key (defaults to falsy in GitHub Actions).

### Matrix definition (`.github/workflows/ci.yml` lines 53–72)

```yaml
strategy:
  fail-fast: false
  matrix:
    include:
      - runner: macos-latest
        target: aarch64-apple-darwin
        proptest_cases: 256
      - runner: macos-15-intel
        target: x86_64-apple-darwin
        proptest_cases: 256
      - runner: ubuntu-latest
        target: x86_64-unknown-linux-gnu
        proptest_cases: 1000
        run_doctests: true          # <-- only this leg gets doctests
      - runner: ubuntu-latest
        target: x86_64-unknown-linux-musl
        install_musl: true
        proptest_cases: 256
      - runner: windows-latest
        target: x86_64-pc-windows-msvc
        proptest_cases: 256
```

### Doctest step (`.github/workflows/ci.yml` lines 109–111)

```yaml
      - name: Run doctests (linux-gnu only — nextest skips doctests)
        if: matrix.run_doctests
        run: cargo test --workspace --all-features --doc
```

The condition `if: matrix.run_doctests` is falsy when the key is absent, so doctests
run on exactly one matrix leg: `x86_64-unknown-linux-gnu`.

---

## AC-003 — Per-platform PROPTEST_CASES matrix

**Type:** Doc-only (CI YAML structural proof)

The matrix assigns tiered `proptest_cases` values: **1000** for the linux-gnu leg
(full-strength fuzzing) and **256** for all other platforms (proptest's own default).
The value is injected via the `PROPTEST_CASES` environment variable on the `Run tests (nextest)` step.

### Matrix entries with proptest_cases (`.github/workflows/ci.yml` lines 54–71)

| target | runner | proptest_cases |
|--------|--------|----------------|
| aarch64-apple-darwin | macos-latest | 256 |
| x86_64-apple-darwin | macos-15-intel | 256 |
| x86_64-unknown-linux-gnu | ubuntu-latest | **1000** |
| x86_64-unknown-linux-musl | ubuntu-latest | 256 |
| x86_64-pc-windows-msvc | windows-latest | 256 |

### Environment injection (`.github/workflows/ci.yml` lines 104–108)

```yaml
      - name: Run tests (nextest)
        env:
          PROPTEST_CASES: ${{ matrix.proptest_cases }}
          RUSTFLAGS: ${{ runner.os == 'Linux' && '-C link-arg=-fuse-ld=mold' || '' }}
        run: cargo nextest run --workspace --all-features --profile ci
```

The `PROPTEST_CASES` env var flows from the matrix into every proptest invocation
without changes to test source code.

---

## AC-004 — mold linker step (Linux only)

**Type:** Doc-only (CI YAML structural proof)

The mold linker is installed via `rui314/setup-mold@v1` and applied with
`RUSTFLAGS: -C link-arg=-fuse-ld=mold`. The install step guards with
`if: runner.os == 'Linux'`; macOS and Windows runners skip it entirely.
`make-default: false` is intentional — RUSTFLAGS is set explicitly rather than
mutating the global linker, preventing conflicts with other RUSTFLAGS settings.

### Install step (`.github/workflows/ci.yml` lines 89–95)

```yaml
      # Optimization: mold linker — faster linking on Linux (saves ~5 min per Linux leg)
      # make-default: false — we set RUSTFLAGS explicitly for robustness (see rui314/setup-mold#3)
      - name: Install mold linker (Linux only)
        if: runner.os == 'Linux'
        uses: rui314/setup-mold@9c9c13bf4c3f1adef0cc596abc155580bcb04444 # v1
        with:
          make-default: false
```

### RUSTFLAGS injection (`.github/workflows/ci.yml` lines 104–108)

```yaml
      - name: Run tests (nextest)
        env:
          PROPTEST_CASES: ${{ matrix.proptest_cases }}
          RUSTFLAGS: ${{ runner.os == 'Linux' && '-C link-arg=-fuse-ld=mold' || '' }}
        run: cargo nextest run --workspace --all-features --profile ci
```

On macOS and Windows, `runner.os != 'Linux'` evaluates `RUSTFLAGS` to an empty string —
no mold flag is emitted. The `test-no-default-features` job (Ubuntu-only) hardcodes
`RUSTFLAGS: -C link-arg=-fuse-ld=mold` directly since it runs exclusively on Linux.

---

## AC-005 — Justfile parity: both check targets use nextest

**Recording:** `AC-005-justfile-nextest-parity.gif` / `AC-005-justfile-nextest-parity.webm`  
**Tape:** `AC-005-justfile-nextest-parity.tape`

Demonstrates `just --show check` and `just --show check-ci` — both recipes use
`cargo nextest run` (not `cargo test`) plus a separate `cargo test --doc` step,
mirroring the CI pipeline structure.

### `check` recipe (Justfile lines 20–25)

```makefile
check:
    cargo fmt --check
    cargo clippy --all-features -- -D warnings
    PROPTEST_CASES=100 cargo nextest run --workspace --all-features --no-fail-fast
    PROPTEST_CASES=100 cargo test --workspace --all-features --doc
    @scripts/check-crate-layout.sh
```

### `check-ci` recipe (Justfile lines 29–37)

```makefile
check-ci:
    cargo fmt --check
    cargo clippy --all-features -- -D warnings
    cargo nextest run --workspace --all-features --no-fail-fast
    cargo test --workspace --all-features --doc
    cargo deny check
    cargo audit
    cargo semver-checks --workspace --baseline-rev origin/develop
    @scripts/check-crate-layout.sh
```

Both targets use `cargo nextest run` (not legacy `cargo test`) and include a
separate `cargo test --doc` step (because nextest skips doctests by design).
The `check` recipe sets `PROPTEST_CASES=100` for fast local iteration;
`check-ci` inherits the shell environment (CI sets 1000 or 256 via the matrix).

---

## nextest.toml — CI Profile

`.config/nextest.toml` introduces the `[profile.ci]` profile used by all CI test steps:

```toml
[profile.ci]
fail-fast = false
slow-timeout = { period = "60s" }
final-status-level = "slow"
failure-output = "immediate-final"

[profile.ci.junit]
path = "junit.xml"
store-success-output = false
store-failure-output = true
```

JUnit XML is uploaded as `junit-<target>` artifacts after every CI run
(retained 7 days), enabling test result visualization in GitHub Actions.

---

_Evidence generated by Demo Recorder agent per VSDD factory protocol._
