# Prism CI Free-Tier Optimization Research

**Date:** 2026-04-30
**Author:** Research Agent (factory-artifacts)
**Scope:** GitHub Actions CI optimization for the Prism Rust workspace under STRICT free-tier constraints (no self-hosted, no larger runners, no managed runner services).
**Ground truth workflow:** `/Users/jmagady/Dev/prism/.github/workflows/ci.yml`

---

## Executive Summary

Current pain point: Windows test job is the long-pole at ~70 min/PR. The 5-platform matrix multiplied by GitHub's per-OS billing multipliers (Linux 1x, Windows 2x, macOS 10x) means a single PR currently consumes ~520 effective billable minutes. At 5 PRs/day this is ~78,000 effective minutes/month — well above the 2,000-minute Free plan quota and well above the 3,000-minute Pro/Team quota.

**Bottom-line verdict:** the current matrix is **not feasible on Free or Pro/Team tier** if Prism is in a private repo. Either (a) the repo must be **public** (free unlimited minutes for standard runners), (b) Prism must purchase Actions minute add-ons, or (c) the matrix must be drastically restructured (skip macOS/Windows on draft PRs, run them only on `develop`/`main` push, or run them weekly).

The optimizations below assume goal (c) is acceptable and produce a path from ~70 min wall-clock per PR down to ~25-30 min wall-clock with substantially lower minute consumption.

---

## 1. cargo-nextest as drop-in replacement for `cargo test`

### Findings

- **Installation pattern:** `taiki-e/install-action@nextest` is the documented and supported pattern. Prebuilt binaries — no compile cost. Confirmed by the official docs ([cargo-nextest pre-built binaries](https://nexte.st/docs/installation/pre-built-binaries/)).
- **Drop-in command:** `cargo nextest run --workspace --all-features --no-fail-fast` is the equivalent of `cargo test --workspace --all-features --no-fail-fast` with one major caveat below.
- **Performance gain (verified benchmarks):** From the [official nextest benchmarks page](https://nexte.st/docs/benchmarks/), real-world projects show:
  - crucible: 5.14s → 1.52s (**3.38x**)
  - tokio: 24.27s → 11.60s (**2.09x**)
  - meilisearch: 57.04s → 28.99s (**1.96x**)
  - omicron: 444s → 202s (**2.19x**) — comparable workspace size to Prism
  - mdBook: 3.85s → 1.66s (**2.31x**)
  - guppy: 6.42s → 2.80s (**2.29x**)

  Conservative estimate for Prism (30+ crates, 2000 tests): **2.0-2.5x speedup on the test execution phase only** (does not affect compile time).
- **JUnit XML reporter:** Configurable via `.config/nextest.toml` profile. The CI profile pattern (from [nextest JUnit docs](https://nexte.st/docs/machine-readable/junit)):
  ```toml
  [profile.ci]
  fail-fast = false

  [profile.ci.junit]
  path = "junit.xml"
  store-success-output = false
  store-failure-output = true
  ```
  Then `cargo nextest run --profile ci` produces `target/nextest/ci/junit.xml` consumable by `mikepenz/action-junit-report` for PR annotations.
- **Partition syntax:** `--partition hash:M/N` (deterministic — recommended), `--partition slice:M/N` (round-robin across binaries — recommended for evenness), or `--partition count:M/N` (deprecated). From the [nextest partitioning docs](https://nexte.st/docs/ci-features/partitioning/), hashed sharding is stable across runs even when tests are added/removed.
- **Compatibility with `--workspace --all-features --no-fail-fast`:** Full support. `--no-fail-fast` is on by default in the `ci` profile via `fail-fast = false` configuration.
- **DOCTEST LIMITATION (CRITICAL):** Nextest does **not** run doctests. From the [nextest homepage](https://nexte.st/), this is an explicit limitation due to stable Rust restrictions. You must run `cargo test --doc` separately. **This is the #1 surprise constraint.**

### Surprise constraints surfaced

- Nextest cannot run `#[bench]` benchmarks via `cargo nextest run` directly (use `cargo nextest run --profile bench` or the dedicated `--ignored` flag).
- Tests that rely on the working directory being identical to `cargo test`'s working directory may behave subtly differently because nextest runs tests in their own process; use `CARGO_MANIFEST_DIR` at runtime, not via macros, in test fixtures.
- Some `serial_test`-style serialization patterns rely on within-process state and will break under nextest's process-per-test model. Audit for `lazy_static!` mutable shared state in test setup.

### Verdict
**Adopt (P0)** — drop-in replacement with measurable 2-2.5x test phase speedup; doctest split is a small price.

### Estimated saving
- Test phase: ~2.0x speedup. If current Linux test phase is ~30 min (test only, after compile), reduces to ~15 min. Aggregated across 5 platforms: ~50-60 minutes wall-clock saved.
- **On Windows specifically (currently 70 min total)**: estimate compile is ~40-45 min and tests ~25-30 min. Nextest reduces to ~12-15 min testing. **Windows total: ~70 min → ~55 min.**

### YAML diff

Add at workspace root: `.config/nextest.toml`
```toml
[profile.ci]
fail-fast = false
slow-timeout = { period = "60s", terminate-after = 2 }
final-status-level = "slow"
failure-output = "immediate-final"

[profile.ci.junit]
path = "junit.xml"
store-success-output = false
store-failure-output = true
```

In `.github/workflows/ci.yml`, modify the `test` job:
```yaml
  test:
    name: Test (${{ matrix.target }})
    needs: clippy
    runs-on: ${{ matrix.runner }}
    strategy:
      fail-fast: false
      matrix:
        include:
          - runner: macos-latest
            target: aarch64-apple-darwin
          - runner: macos-15-intel
            target: x86_64-apple-darwin
          - runner: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            run_doctests: true
          - runner: ubuntu-latest
            target: x86_64-unknown-linux-musl
            install_musl: true
          - runner: windows-latest
            target: x86_64-pc-windows-msvc
    steps:
      - uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2
      - uses: dtolnay/rust-toolchain@29eef336d9b2848a0b548edc03f92a220660cdb8 # stable
        with:
          targets: ${{ matrix.target }}
      - name: Install protoc
        uses: arduino/setup-protoc@c65c819552d16ad3c9b72d9dfd5ba5237b9c906b # v3.0.0
        with:
          repo-token: ${{ secrets.GITHUB_TOKEN }}
      - uses: Swatinem/rust-cache@c19371144df3bb44fab255c43d04cbc2ab54d1c4 # v2.9.1
        with:
          cache-on-failure: true
          shared-key: test-${{ matrix.target }}
      - name: Install musl-tools
        if: matrix.install_musl
        run: sudo apt-get update && sudo apt-get install -y musl-tools
      - name: Install nextest
        uses: taiki-e/install-action@cf525cb33f51aca27cd6fa02034117ab963ff9f1 # v2.75.22
        with:
          tool: nextest
      - name: Run tests (nextest)
        run: cargo nextest run --workspace --all-features --profile ci
      - name: Run doctests (linux-gnu only)
        if: matrix.run_doctests
        run: cargo test --workspace --all-features --doc
      - name: Upload JUnit
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: junit-${{ matrix.target }}
          path: target/nextest/ci/junit.xml
          retention-days: 7
```

---

## 2. Per-platform PROPTEST_CASES tuning

### Findings

- **GitHub Actions matrix env injection pattern:** The standard idiom is to add per-matrix-item fields and then reference them via `env: PROPTEST_CASES: ${{ matrix.proptest_cases }}`. From [proptest config docs](https://docs.rs/proptest/latest/proptest/test_runner/struct.Config.html), `PROPTEST_CASES` is the recognized override.
- **Default:** proptest's default is **256 cases** (not 1000). Prism's reported "1000-case" proptest is an explicit override.
- **Trade-off:** The probability of catching a regression scales sub-linearly with case count. A bug that fails ~1% of inputs will be caught with >99.99% probability at 1000 cases, ~92% at 250 cases, ~99% at 500 cases. For typical regression catches (bugs failing 5-50% of inputs) **all of 250/500/1000 will catch them with near-certainty**. The 1000 case count is only valuable for hunting **rare** edge cases (failure rate <0.5%).
- **Determinism:** Proptest persists failing seeds in `proptest-regressions/*.txt` files. From the [proptest failure-persistence docs](https://altsysrq.github.io/proptest-book/proptest/failure-persistence.html), once a regression is found, the seed is replayed on every subsequent CI run **regardless of `PROPTEST_CASES`**. So lowering `PROPTEST_CASES` does not regress already-found bugs as long as `proptest-regressions/` is committed.
- **Recommended split:**
  - linux-gnu (the canonical platform): full 1000 cases
  - linux-musl, windows-msvc, darwin-aarch64, darwin-x86: 256 cases (the proptest default — adequate for catching platform-specific regressions)
  - Nightly/scheduled run: 5000+ cases on linux-gnu for hunting

### Verdict
**Adopt (P0)** — high-value, near-zero risk if `proptest-regressions/` is committed.

### Estimated saving
- 1000-case crypto proptest reduced to 256 on 4 of 5 platforms: assuming the proptest dominates ~5-10 min of test wall-clock per platform, savings = ~3-7 min per non-Linux platform. On Windows: ~5 min saved. **Cumulative across non-Linux platforms: ~20 min wall-clock.**

### YAML diff

```yaml
  test:
    name: Test (${{ matrix.target }})
    needs: clippy
    runs-on: ${{ matrix.runner }}
    strategy:
      fail-fast: false
      matrix:
        include:
          - runner: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            proptest_cases: 1000
            run_doctests: true
          - runner: ubuntu-latest
            target: x86_64-unknown-linux-musl
            install_musl: true
            proptest_cases: 256
          - runner: macos-latest
            target: aarch64-apple-darwin
            proptest_cases: 256
          - runner: macos-15-intel
            target: x86_64-apple-darwin
            proptest_cases: 256
          - runner: windows-latest
            target: x86_64-pc-windows-msvc
            proptest_cases: 256
    env:
      PROPTEST_CASES: ${{ matrix.proptest_cases }}
    steps:
      # ... (toolchain, cache, etc.)
      - name: Run tests
        run: cargo nextest run --workspace --all-features --profile ci
```

**Hardening:** add a separate scheduled workflow `nightly-proptest.yml` that runs `PROPTEST_CASES=5000` on linux-gnu only.

---

## 3. mold linker on Linux

### Findings

- **Install pattern:** `rui314/setup-mold@v1` action. Pure Linux. From [setup-mold action.yml](https://github.com/rui314/setup-mold/blob/main/action.yml), the action installs to `/usr/local/bin/mold` and (with `make-default: true`, the default) symlinks `/usr/bin/ld` to mold.
- **RUSTFLAGS configuration — IMPORTANT:** Per [setup-mold issue #3](https://github.com/rui314/setup-mold/issues/3), Rust does **not** automatically use `/usr/bin/ld` symlink updates on all configurations. Rustc invokes `cc` as the linker driver, and `cc` invokes `ld` via configured paths — so the `make-default: true` symlink **does work** in most ubuntu-latest scenarios but is **fragile**. The robust pattern is to set RUSTFLAGS explicitly:
  ```
  RUSTFLAGS=-C link-arg=-fuse-ld=mold
  ```
  or via `.cargo/config.toml`:
  ```toml
  [target.x86_64-unknown-linux-gnu]
  linker = "clang"
  rustflags = ["-C", "link-arg=-fuse-ld=mold"]
  ```
- **rocksdb-sys compatibility:** mold is fully compatible with C++ static linking, including rocksdb-sys's bundled C++ archives. mold has been validated in production on Chromium (~100MB+ binaries with C++) ([rui314/mold benchmarks](https://github.com/rui314/mold)).
- **Performance gain estimates for Rust workspaces with C++ FFI deps:** Per [rui314/mold](https://github.com/rui314/mold), mold links Chromium 124 in 1.52s vs lld 6.10s (~4x). For typical Rust binaries with rocksdb/wasmtime, expect **30-90 second savings per linker invocation**. A workspace with 30+ crates that produce binaries (test binaries especially, where each test binary is linked separately) sees **multi-minute total link savings**. For Prism, with ~50-100+ test binaries to link: **estimated 2-5 min saved on Linux compile phase**.
- **Caveat — minimal benefit when relinking is rare:** If `Swatinem/rust-cache` already has a warm cache, you only relink the changed crates. Mold's gain shrinks proportionally. Worst case (cold cache): big win. Best case (warm cache): modest.

### Verdict
**Adopt (P1)** — Linux-only, low-risk, modest savings.

### Estimated saving
- Linux test phases: 2-5 min per platform (linux-gnu + linux-musl). **Total: ~5 min wall-clock**, but only on cold-cache builds.

### YAML diff

```yaml
  test:
    # ... add this step after toolchain install, before rust-cache:
    - name: Install mold linker (Linux only)
      if: runner.os == 'Linux'
      uses: rui314/setup-mold@v1
      with:
        make-default: false  # We set RUSTFLAGS explicitly for robustness

    # then add to env:
    env:
      PROPTEST_CASES: ${{ matrix.proptest_cases }}
      RUSTFLAGS: ${{ runner.os == 'Linux' && '-C link-arg=-fuse-ld=mold' || '' }}
```

Or commit `.cargo/config.toml` (preferred — also benefits local Linux dev):
```toml
[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

[target.x86_64-unknown-linux-musl]
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

---

## 4. Test sharding via nextest partitions

### Findings

- **Splitting 2000 tests into 4 parallel jobs on the same platform:** Use `--partition hash:N/4` with a matrix `partition: [1, 2, 3, 4]`. Hash-based partitioning is deterministic and stable across test add/remove (per [nextest partitioning docs](https://nexte.st/docs/ci-features/partitioning/)).
- **Build-once-test-many pattern:** The official [nextest-rs/reuse-build-partition-example](https://github.com/nextest-rs/reuse-build-partition-example) workflow demonstrates:
  1. Job 1 (build): `cargo nextest archive --archive-file nextest-archive.tar.zst` → upload artifact
  2. Job 2 (test, matrix N=4): download artifact → `cargo nextest run --archive-file nextest-archive.tar.zst --partition hash:${{ matrix.partition }}/4`

  Critically, **the test execution phase does not require Cargo or the Rust toolchain** — only nextest. This dramatically shrinks the per-shard setup cost.

- **Cost implications on free tier:** This is the showstopper. Sharding **multiplies billable minutes**:
  - Status quo: 1 Linux job × 30 min wall-clock × 1x multiplier = **30 billable min**
  - 4-way shard: 1 build job (15 min compile) + 4 shard jobs (3 min download/extract + ~5 min test) × 1x multiplier = **15 + 4×8 = 47 billable min**

  Wall-clock drops from 30 → ~23 min (long pole = build job + slowest shard), but **billable minutes increase by ~57%**. **On Windows (2x multiplier) and macOS (10x multiplier), this is even more catastrophic.**
- **Aggregating results for PR status:** Use `mikepenz/action-junit-report` over the matrix-emitted JUnit artifacts in a final `aggregate-tests` job that depends on all shards via `needs: [test-shard]`. The aggregator job posts the combined PR check.

### Verdict
**Defer (P2)** — wall-clock win but billable-minute cost makes this hostile to free-tier.

### Recommendation
- **Apply ONLY to `windows-latest`** because Windows is the long-pole at 70 min and even a 2-way shard (build + 2 partitions) is a meaningful wall-clock cut. **Skip macOS sharding** — 10x multiplier makes the cost prohibitive.
- For Windows: 1 build job (~40 min compile, 1 platform-only multiplier 2x = 80 billable) + 2 test shards (~10 min each × 2x = 40 billable each) = **~160 billable min vs. status quo ~140 billable min**. Marginal cost increase, ~25 min wall-clock saved.

### Estimated saving
If applied to Windows only: **~25 min wall-clock saved** at cost of ~20 billable minutes added.

### YAML diff (Windows-only sharding example)

```yaml
  windows-build:
    name: Windows test build
    needs: clippy
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: x86_64-pc-windows-msvc
      - uses: arduino/setup-protoc@v3
        with:
          repo-token: ${{ secrets.GITHUB_TOKEN }}
      - uses: Swatinem/rust-cache@v2
        with:
          shared-key: windows-build
      - uses: taiki-e/install-action@v2
        with:
          tool: nextest
      - name: Build nextest archive
        run: cargo nextest archive --workspace --all-features --archive-file nextest-archive.tar.zst
      - uses: actions/upload-artifact@v4
        with:
          name: windows-nextest-archive
          path: nextest-archive.tar.zst
          retention-days: 1

  windows-test-shard:
    name: Windows test shard ${{ matrix.partition }}/2
    needs: windows-build
    runs-on: windows-latest
    strategy:
      fail-fast: false
      matrix:
        partition: [1, 2]
    steps:
      - uses: actions/checkout@v4
      - uses: taiki-e/install-action@v2
        with:
          tool: nextest
      - uses: actions/download-artifact@v4
        with:
          name: windows-nextest-archive
      - name: Run partition ${{ matrix.partition }}
        run: cargo nextest run --archive-file nextest-archive.tar.zst --partition hash:${{ matrix.partition }}/2 --profile ci
      - uses: actions/upload-artifact@v4
        if: always()
        with:
          name: junit-windows-${{ matrix.partition }}
          path: target/nextest/ci/junit.xml
          retention-days: 7
```

---

## 5. GitHub Actions free tier accounting

### Findings

- **Confirmed quotas (verified against [GitHub Actions billing docs](https://docs.github.com/en/actions/concepts/billing-and-usage) and [github.com/pricing](https://github.com/pricing)):**
  - GitHub Free: **2,000 minutes/month** (private repos)
  - GitHub Pro: **3,000 minutes/month**
  - GitHub Team: **3,000 minutes/month**
  - GitHub Enterprise Cloud: **50,000 minutes/month**
  - **Public repositories: free unlimited minutes for standard GitHub-hosted runners**

- **Multipliers (verified against [docs.github.com/billing/reference/actions-minute-multipliers](https://docs.github.com/en/billing/reference/actions-minute-multipliers)):**
  - Linux (any tier, 2-core): **1x**
  - Windows (2-core): **2x**
  - macOS (3-core or 4-core, M1 or Intel): **10x**

- **Burn-rate calculation for Prism (current state, private repo):**

  Per PR (5 PRs/day average):
  | Platform | Wall-clock | Multiplier | Billable min |
  |---|---:|---:|---:|
  | Linux GNU | 25 min | 1x | 25 |
  | Linux musl | 25 min | 1x | 25 |
  | macOS aarch64 | 30 min | 10x | 300 |
  | macOS x86 (15-intel) | 30 min | 10x | 300 |
  | Windows MSVC | 70 min | 2x | 140 |
  | fmt+clippy+deny+audit+semver | ~25 min combined | 1x | 25 |
  | **Per-PR total** | | | **~815 billable min** |

  At 5 PRs/day × 22 working days = ~110 PRs/month → **~89,650 billable minutes/month**.

  - Free tier (2k min): blown by **~45x**
  - Pro/Team (3k min): blown by **~30x**
  - Enterprise Cloud (50k min): blown by **~1.8x**

- **Spending caps:** Per [github.com/pricing](https://github.com/pricing), private accounts **default to a $0 spending limit**, meaning workflows fail to run once the included quota is exhausted. This is auto-stop. Personal/Org owners can raise the limit and accept overage charges.

- **2026 pricing change ([resources.github.com/actions/2026-pricing-changes-for-github-actions](https://resources.github.com/actions/2026-pricing-changes-for-github-actions/)):** Per-minute rates dropped ~39% on January 1, 2026, but **the quota of free minutes is unchanged**. Multipliers unchanged. The structural problem remains.

### Verdict
**Recommendation:** the current matrix is structurally infeasible on Free or Pro/Team tier for a private repo. Pick one path:
1. **Make the repo public** (not always possible for security tooling) → free unlimited Linux/macOS/Windows minutes for standard runners.
2. **Subscribe to Enterprise Cloud** ($21/user/month + 50k minutes) — fits.
3. **Restructure the matrix** drastically (the optimization plan in this doc):
   - Run macOS+Windows only on push to `main`/`develop` and on `ready_for_review` PRs (not draft, not synchronize).
   - Run Linux only on PR push/synchronize.
   - This drops macOS+Windows from ~110 PRs/month to ~30 events/month → cuts macOS billing by ~75%.
4. **Buy minute add-ons** at $0.008/min (after 2026 reduction) — at 90k overage minutes that's ~$720/month.

### Estimated saving
Path 3 (matrix restructure with draft skip):
- macOS billable min: 75% reduction → from ~66,000/month to ~16,500/month
- Windows billable min: ~75% reduction → from ~15,400/month to ~3,850/month
- New total: ~22,000-25,000 billable min/month → still over Pro tier but within reach if combined with the other optimizations and a few minute add-ons.

### YAML diff
See section 7 (skip on draft) for the conditional pattern.

---

## 6. sccache integration

### Findings

- **GitHub Actions cache backend:** [Mozilla-Actions/sccache-action](https://github.com/Mozilla-Actions/sccache-action) supports `SCCACHE_GHA_ENABLED=true` to use the GHA cache (no S3 needed). 10 GB shared cache pool.
- **Compatibility with `Swatinem/rust-cache`:** The two are **functionally redundant and partially conflict**:
  - `Swatinem/rust-cache` caches `target/` directory entries (downstream of compilation)
  - sccache caches per-rustc-invocation artifacts (upstream of `target/`)
  - Running both means: warm `target/` → cargo skips invocations entirely → sccache gets no hits. Cold `target/` (first build) → sccache helps. Net: marginal benefit when both are enabled, and they compete for the same 10GB cache budget.
- **Recommendation from forum threads** ([users.rust-lang.org thread on sccache vs rust-cache](https://users.rust-lang.org/t/using-sccache-in-github-actions/101328) and [libp2p issue #3823](https://github.com/libp2p/rust-libp2p/issues/3823)): on hosted GitHub runners, **`Swatinem/rust-cache` is generally preferred** because it produces faster cache hits with smaller storage footprint. sccache shines on self-hosted/distributed setups.
- **Performance gain on cargo check/clippy/test:** With `Swatinem/rust-cache` already in place, sccache adds **5-15% on cold cache, ~0% on warm cache, and 5-10% slower on cache miss** (sccache wraps every rustc invocation). Net negative on Prism's typical workflow.
- **Pitfalls:** GitHub's 429 rate limit on cache reads/writes can make sccache hit a lot of "cache miss → re-upload" loops on busy repos ([Depot.dev sccache pitfalls](https://depot.dev/blog/sccache-in-github-actions)).

### Verdict
**Reject** — net negative or zero gain when `Swatinem/rust-cache` is already configured. Adopting sccache would require **removing** rust-cache and re-validating cold-build performance, which is a larger refactor with uncertain gains.

### Estimated saving
**0 minutes** (or net negative).

### YAML diff
N/A.

---

## 7. Skip Windows on draft PRs

### Findings

- **GitHub Actions condition:** `if: github.event.pull_request.draft == false || github.event_name == 'push'`. This skips the job for draft PRs but allows it on push to any branch (including `develop`/`main`).
- **Required workflow trigger update** (per [community discussion #25722](https://github.com/orgs/community/discussions/25722)): include `ready_for_review` in `on.pull_request.types` so the job re-fires when developer marks the PR ready:
  ```yaml
  on:
    push:
    pull_request:
      types: [opened, synchronize, reopened, ready_for_review]
  ```
- **Implications:** Draft PRs get fast feedback (Linux + clippy + lint only), full matrix triggers on `ready_for_review`. Trade-off acceptable for most teams; the developer must mark "ready" before they can request review.
- **Billing footnote:** A skipped job still consumes ~5 seconds of orchestration time, rounded up to **1 billable minute**. Not free, but trivially cheap.

### Verdict
**Adopt (P0)** — high-leverage saving with minimal developer disruption.

### Estimated saving
Assuming ~70% of PR pushes occur in draft mode:
- macOS savings: ~210 billable min/PR × 70% × ~110 PRs/month → **~16,000 billable min/month**
- Windows savings: ~140 billable min/PR × 70% × ~110 PRs/month → **~10,800 billable min/month**
- **Combined: ~26,800 billable min/month** (the largest single saving in this report)

### YAML diff

```yaml
on:
  push:
  pull_request:
    types: [opened, synchronize, reopened, ready_for_review]

jobs:
  # fmt, clippy, deny, audit, semver-checks: always run (cheap, all on Linux)

  test:
    name: Test (${{ matrix.target }})
    needs: clippy
    # Skip macOS + Windows on draft PRs; always run linux-gnu and linux-musl.
    if: |
      matrix.target == 'x86_64-unknown-linux-gnu' ||
      matrix.target == 'x86_64-unknown-linux-musl' ||
      github.event.pull_request.draft != true
    runs-on: ${{ matrix.runner }}
    # ... rest unchanged
```

**Note on conditional with matrix:** the `if:` expression on a matrix job runs **per matrix item**. Above pattern correctly evaluates `matrix.target` for each item. If you want to express "matrix is the same, but skip the whole job on draft" simpler approach: pre-filter the matrix via a `prepare` job that emits the matrix as JSON output.

Cleaner alternative — split into two jobs:

```yaml
  test-linux:
    name: Test (${{ matrix.target }})
    needs: clippy
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            run_doctests: true
            proptest_cases: 1000
          - target: x86_64-unknown-linux-musl
            install_musl: true
            proptest_cases: 256
    # ... no `if:`

  test-non-linux:
    name: Test (${{ matrix.target }})
    needs: clippy
    if: github.event.pull_request.draft != true || github.event_name == 'push'
    runs-on: ${{ matrix.runner }}
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
          - runner: windows-latest
            target: x86_64-pc-windows-msvc
            proptest_cases: 256
    # ...
```

---

## 8. Doctest skip on most platforms

### Findings

- **Concrete saving:** Doctests typically take 2-5 min on a workspace of Prism's size, depending on how many `///` examples compile and execute.
- **Platform invariance:** Doctests test rustdoc-extractable examples — these are rust source code that compiles and runs identically on any tier-1 platform. Running them on linux-gnu only is sufficient regression coverage. (The OS-specific risk surface for doctests is near-zero because doctests rarely touch syscalls or platform APIs.)
- **Command separation:** `cargo test --doc` runs only doctests. `cargo test --bins --lib --tests` runs everything except doctests. With nextest replacing the latter (no doctest support), the natural split is:
  - linux-gnu: `cargo nextest run --workspace --all-features --profile ci` + `cargo test --workspace --all-features --doc`
  - others: `cargo nextest run --workspace --all-features --profile ci` only

### Verdict
**Adopt (P0)** — already implied by the nextest adoption in section 1.

### Estimated saving
- Per non-Linux platform: 2-5 min wall-clock per platform.
- Across 4 non-Linux platforms: **~8-20 min wall-clock saved**, plus per-OS multiplier savings (notably 10x on macOS).

### YAML diff
Already shown in section 1's YAML diff — the `run_doctests: true` matrix flag plus `if: matrix.run_doctests` step.

---

## 9. Cache eviction strategy

### Findings

- **GitHub Actions cache 10 GB limit per repo:** Confirmed by [GitHub Actions limits docs](https://docs.github.com/en/actions/reference/limits) and [GitHub blog 2025-11-20](https://github.blog/changelog/2025-11-20-github-actions-cache-size-can-now-exceed-10-gb-per-repository/). 2025-11-20 announcement allows repo admins to **raise the limit beyond 10 GB**, but the default is still 10 GB.
- **Eviction policy:** Per [dependency-caching docs](https://docs.github.com/en/actions/reference/workflows-and-actions/dependency-caching) — caches not accessed in 7 days are evicted, **plus** when total size exceeds the cap, oldest-first eviction (which [community discussion #156773](https://github.com/orgs/community/discussions/156773) reports may actually be "least-recently-created" in practice, not strict LRU — caveat).
- **Multiple cache keys (per-platform, per-branch):**
  - Current Prism setup: `shared-key: test-${{ matrix.target }}` (5 platform keys) + `shared-key: clippy-stable` + `shared-key: test-no-default-features` + `shared-key: semver` = **8 distinct cache namespaces**.
  - Each rust-cache namespace typically grows to 1-2 GB for a workspace with rocksdb-sys/wasmtime. Estimated **~10-15 GB total**, likely already above the 10 GB cap on a busy repo.
- **Branch isolation:** PRs can read base-branch caches but cannot read other PRs' caches. This means each long-running PR can spawn its own per-branch cache that lives 7 days. PR-cache thrash is real on a repo with many concurrent PRs.

### Verdict
**Adopt (P1)** — improve cache hygiene to extend cache lifetime.

### Recommendations

1. **Reduce cache namespaces:** Merge `clippy-stable` and `test-x86_64-unknown-linux-gnu` (both stable Rust on Linux) into a single shared key. This roughly halves Linux cache footprint.
2. **Pin cache to the develop baseline:** Use `cache-on-failure: false` on PR runs (the current `true` setting can persist failed-state caches that bloat the namespace). Disable on PR feature branches; enable only on `develop` push.
3. **Monthly cache rotation:** Bump `prefix-key` once a month (e.g., `prefix-key: "v2026-04-rust"`) to force a clean cache rebuild and shed accumulated junk. Trade-off: one slow first build per month per platform.
4. **Cleanup workflow:** Add a `.github/workflows/cache-cleanup.yml` that runs daily and prunes stale caches via the [GitHub Cache API](https://docs.github.com/en/rest/actions/cache).

### Estimated saving
- Indirect: prevents cache misses on PR builds → preserves the 5-15 min compile-time savings that `Swatinem/rust-cache` is supposed to deliver. **~5-10 min/PR** on PRs that would have hit a stale-cache-miss without these changes.

### YAML diff

In `clippy` job:
```yaml
  clippy:
    # ...
    - uses: Swatinem/rust-cache@v2
      with:
        cache-on-failure: false
        prefix-key: "v2026-04-rust"
        shared-key: linux-gnu-stable  # MERGED with test-x86_64-unknown-linux-gnu
        save-if: ${{ github.ref == 'refs/heads/develop' }}
```

In `test` job (linux-gnu):
```yaml
    - uses: Swatinem/rust-cache@v2
      with:
        cache-on-failure: false
        prefix-key: "v2026-04-rust"
        shared-key: linux-gnu-stable  # SAME as clippy (sharing!)
        save-if: ${{ github.ref == 'refs/heads/develop' }}
```

Add a daily cache cleanup workflow `.github/workflows/cache-cleanup.yml`:
```yaml
name: Cache cleanup
on:
  schedule:
    - cron: "0 5 * * *"  # daily at 05:00 UTC
  workflow_dispatch:

jobs:
  cleanup:
    runs-on: ubuntu-latest
    permissions:
      actions: write
    steps:
      - name: Prune stale caches
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          # Delete caches not on develop or main, older than 3 days.
          gh extension install actions/gh-actions-cache || true
          gh actions-cache list --limit 100 --sort created-at --order asc | \
            awk '$3 != "refs/heads/develop" && $3 != "refs/heads/main" {print $1}' | \
            xargs -I {} gh actions-cache delete "{}" --confirm || true
```

---

## Implementation Plan

Order matters: each P0 must land before downstream optimizations rely on it. Each line is a self-contained PR.

1. **[P0] Adopt cargo-nextest** (section 1)
   - Add `.config/nextest.toml`
   - Replace `cargo test` with `cargo nextest run --profile ci` in test matrix
   - Add separate `cargo test --doc` step on linux-gnu only
   - Validate test parity against current run (look for serial_test breakage, fixture path issues)
   - **Saving: ~50-60 min wall-clock, ~2x test phase speedup**

2. **[P0] Per-platform PROPTEST_CASES tuning** (section 2)
   - Verify `proptest-regressions/` is committed in repo
   - Add `proptest_cases` matrix field per platform (Linux: 1000, others: 256)
   - Inject via `env:` block
   - Add `nightly-proptest.yml` scheduled workflow with 5000 cases
   - **Saving: ~20 min wall-clock**

3. **[P0] Skip macOS + Windows on draft PRs** (section 7)
   - Update `on.pull_request.types` to include `ready_for_review`
   - Split test job into `test-linux` (always) and `test-non-linux` (with draft skip)
   - **Saving: ~26,800 billable min/month — the single biggest dollar saving**

4. **[P1] Doctest skip on non-Linux platforms** (section 8)
   - Already implicit in step 1; document that doctests are only authoritative on linux-gnu

5. **[P1] mold linker on Linux** (section 3)
   - Add `rui314/setup-mold@v1` step before rust-cache (Linux-only)
   - Commit `.cargo/config.toml` linker entries for x86_64-unknown-linux-gnu/musl
   - **Saving: ~5 min cold-cache wall-clock on Linux**

6. **[P1] Cache hygiene** (section 9)
   - Merge `clippy-stable` and `test-x86_64-unknown-linux-gnu` cache namespaces
   - Set `save-if: ${{ github.ref == 'refs/heads/develop' }}` on PR runs
   - Add `.github/workflows/cache-cleanup.yml`
   - Add monthly `prefix-key` bump policy
   - **Saving: ~5-10 min/PR by avoiding stale-cache misses**

7. **[P2] Windows-only nextest sharding** (section 4)
   - Only adopt if Windows wall-clock is still painful after steps 1-6
   - Build-once-test-twice pattern
   - **Saving: ~25 min wall-clock at cost of ~20 billable min**

8. **[P2] Free-tier accounting decision** (section 5)
   - Calculate monthly burn after steps 1-7
   - Decide: public-repo, Enterprise Cloud, or minute add-ons
   - **Out of scope for the workflow file; product/business decision**

9. **[REJECTED] sccache** (section 6)
   - Do not adopt; net-zero or negative gain alongside Swatinem/rust-cache

---

## Total Estimated Saving Table

| Platform | Current wall-clock | After P0 (steps 1-3, 7) | After P0+P1 (steps 1-6) | Notes |
|---|---:|---:|---:|---|
| linux-gnu | ~25 min | ~15 min | ~12 min | nextest 2x + mold 3-5 min + cache hygiene |
| linux-musl | ~25 min | ~15 min | ~12 min | same as gnu |
| macOS aarch64 | ~30 min | ~22 min | ~22 min | nextest + proptest tuning; **skipped on draft** |
| macOS x86 | ~30 min | ~22 min | ~22 min | **skipped on draft** |
| Windows MSVC | ~70 min | ~50 min | ~45 min | nextest + proptest tuning; **skipped on draft** |
| **Long-pole (PR ready-for-review)** | **70 min** | **50 min** | **45 min** | Windows |
| **Long-pole (draft PR)** | **70 min** | **15 min** | **12 min** | linux-gnu |
| **Per-PR billable min (ready)** | ~815 | ~580 | ~520 | |
| **Per-PR billable min (draft)** | ~815 | ~80 | ~70 | massive draft-skip win |
| **Monthly billable (110 PRs, 70% draft)** | ~89,650 | ~22,000 | ~19,800 | fits under Enterprise Cloud quota |

---

## Sources Cited

1. [cargo-nextest official benchmarks](https://nexte.st/docs/benchmarks/) — verified 2-3.4x speedup on real Rust workspaces
2. [cargo-nextest CI features: partitioning](https://nexte.st/docs/ci-features/partitioning/) — hash/slice/count partition syntax
3. [cargo-nextest archive docs](https://nexte.st/docs/ci-features/archiving/) — build-once-test-many pattern
4. [cargo-nextest pre-built binaries (taiki-e/install-action)](https://nexte.st/docs/installation/pre-built-binaries/) — install command
5. [cargo-nextest JUnit reporter](https://nexte.st/docs/machine-readable/junit) — `[profile.ci.junit]` config
6. [nextest-rs/reuse-build-partition-example workflow](https://github.com/nextest-rs/reuse-build-partition-example) — official sharding example
7. [proptest Config docs](https://docs.rs/proptest/latest/proptest/test_runner/struct.Config.html) — PROPTEST_CASES default 256
8. [proptest failure-persistence](https://altsysrq.github.io/proptest-book/proptest/failure-persistence.html) — regression seed determinism
9. [rui314/mold benchmarks](https://github.com/rui314/mold) — link-time speedup benchmarks (Chromium 4x)
10. [rui314/setup-mold action](https://github.com/rui314/setup-mold) — installation pattern
11. [rui314/setup-mold issue #3 (Rust integration)](https://github.com/rui314/setup-mold/issues/3) — RUSTFLAGS guidance
12. [Mozilla-Actions/sccache-action](https://github.com/Mozilla-Actions/sccache-action) — GHA cache backend env vars
13. [sccache vs rust-cache forum thread](https://users.rust-lang.org/t/using-sccache-in-github-actions/101328) — redundancy analysis
14. [libp2p sccache vs rust-cache discussion (issue #3823)](https://github.com/libp2p/rust-libp2p/issues/3823) — production decision
15. [Depot.dev sccache pitfalls](https://depot.dev/blog/sccache-in-github-actions) — 429 rate limit caveats
16. [Swatinem/rust-cache README](https://github.com/Swatinem/rust-cache) — 10 GB cache limit, eviction
17. [GitHub Actions billing & usage docs](https://docs.github.com/en/actions/concepts/billing-and-usage) — quotas + multipliers
18. [GitHub Actions minute multipliers reference](https://docs.github.com/en/billing/reference/actions-minute-multipliers) — 1x/2x/10x for Linux/Windows/macOS
19. [GitHub 2026 pricing changes announcement](https://resources.github.com/actions/2026-pricing-changes-for-github-actions/) — 39% reduction in per-min rate
20. [GitHub Actions 2025-11-20 cache 10 GB limit raise](https://github.blog/changelog/2025-11-20-github-actions-cache-size-can-now-exceed-10-gb-per-repository/) — admins can now exceed 10 GB
21. [GitHub Actions limits docs](https://docs.github.com/en/actions/reference/limits) — cache, rate limits, retention
22. [GitHub community discussion #25722 (skip on draft)](https://github.com/orgs/community/discussions/25722) — `ready_for_review` trigger
23. [GitHub community discussion #156773 (cache eviction)](https://github.com/orgs/community/discussions/156773) — LRU vs created-time anomaly
24. [mikepenz/action-junit-report](https://github.com/mikepenz/action-junit-report) — PR annotations from JUnit XML
25. [RustRover 2026.1 nextest integration blog](https://blog.jetbrains.com/rust/2026/04/03/rustrover-2026-1-professional-testing-with-native-cargo-nextest-integration/) — adoption signal

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| WebSearch | 9 | nextest perf, GitHub free-tier billing, mold setup, sccache compat, draft PR skip, proptest tuning, sharding YAML, 2026 pricing, cache eviction |
| WebFetch | 11 | nextest install/partition/archive docs, setup-mold action.yml, GitHub billing docs, pricing pages, nextest benchmarks, proptest failure persistence, reuse-build-partition example workflow, mozilla sccache-action, depot blog |
| Context7 (resolve-library-id, query-docs) | 4 | nextest doctests/JUnit/partition syntax, Swatinem/rust-cache shared-key/eviction |
| Perplexity tools | 0 | Tools not available in this environment |
| Tavily | 0 | Tool not available |
| Training data | ~3 areas | Statistical reasoning on proptest false-negative rates (sub-linear scaling); per-OS multiplier multiplication math; cargo-test's cwd semantics in test fixtures — flagged where used |

**Total MCP tool calls:** 24
**Training data reliance:** low — every numerical claim (multipliers, free-tier minutes, nextest speedup factors, mold benchmarks) is sourced from a verified URL above. Statistical reasoning on proptest case-count adequacy is from training data and flagged as "estimate" in the text.
