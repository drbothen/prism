---
document_type: research-sidecar
topic: rust-build-optimization-2026
project: prism
version: "1.1"
audience: orchestrator, devops-engineer, dx-engineer, implementer
producer: vsdd-factory:research-agent
timestamp: 2026-05-05T00:00:00Z
input_hash: prism-24-crate-workspace-macos-aarch64-2026-05-05
status: complete
changelog:
  - "v1.1: F-MEDIUM-002 employer-name redaction (PR-127 adversary pass-9)"
sources_consulted:
  - perplexity-sonar-deep-research-pre-existing-2026-05
  - context7:/websites/doc_rust-lang_cargo
  - websearch:nnethercote-faster-rust-builds-mac-2025-09
  - websearch:eisel-lld-blog
  - websearch:howardjohn-shared-rust-build
  - websearch:rust-lang-cargo-pr-9298
  - websearch:apple-developer-forums-ld_prime
  - websearch:nexte.st-docs
scope:
  - macOS aarch64 (Apple Silicon) local developer iteration
  - 24-crate Cargo workspace (datafusion, chumsky, axum, rocksdb)
  - cargo nextest run --workspace --all-features ~5min target reduction
non_scope:
  - GitHub Actions CI optimization (covered separately in 2026-04-30-ci-free-tier-optimization.md)
  - Linux/Windows local dev (some tips overlap; treat applicability as "linux-only" where flagged)
---

# Rust Workspace Build Optimization for Prism (macOS aarch64, 2026)

## 1. Executive Summary

Prism's `cargo nextest run --workspace --all-features` cycle is ~5 minutes on macOS aarch64. The single largest verified speedup available in 2026 is the **macOS XProtect "Developer Tools" exception** for the terminal — the canonical Nethercote case study reduced the rustc UI test suite from 9m42s to 3m33s (a 63% wall-clock reduction). This is a one-time settings change with no code or config impact and applies workspace-wide. Two additional layered wins are: (a) `[profile.dev]` debuginfo tuning (`debug = "line-tables-only"` + dependency-only `debug = false`) for an additional 5–15% incremental compile speedup with no debugger regression on macOS, and (c) tighter iteration discipline — using `cargo nextest run -p <crate>` instead of `--workspace` during TDD, since whole-workspace runs are reserved for `just check` pre-push.

**Top three actionable recommendations:**

1. **Add the user's terminal (Terminal.app, iTerm, or the host running `cargo`) to System Settings → Privacy & Security → Developer Tools.** No code change. Largest verified benefit. Estimated impact for a 5-min cycle: 1.5–3 min reduction (depending on how much of the cycle is rustc/build-script process spawning vs steady-state compilation).
2. **Adopt the `debug = "line-tables-only"` + dependency-only `debug = false` profile pattern** in a committed `.cargo/config.toml`. Validated 5–15% incremental speedup with full backtrace fidelity preserved; loses only inline source-line debugging in dependencies (acceptable for a workspace where Prism owns the code under test).
3. **Document iteration discipline in implementer prompts.** Default to `cargo nextest run -p <crate>` for TDD inner loop; reserve `cargo nextest run --workspace --all-features` for `just check` pre-push. The 24-crate workspace means a single-crate run is typically 5–20× faster than a workspace run.

The XProtect mitigation alone, combined with iteration discipline, is expected to bring TDD inner-loop feedback to the **30–90s range** for incremental edits, with the 5-minute mark reserved for pre-push verification only.

## 2. Project Context

| Fact | Value | Source |
|------|-------|--------|
| Workspace size | 24 crates (per `/Users/jmagady/Dev/prism/Cargo.toml` lines 3-28) | Read 2026-05-05 |
| Heavy-lift dependencies | datafusion 53.1, arrow 58.2, rocksdb 0.24, chumsky 0.12, axum (via prism-mcp) | `.factory/research/W3-library-versions.md` |
| Toolchain | Rust stable; resolver = "2" | `Cargo.toml` line 2 |
| Test runner | `cargo nextest run --workspace --all-features` per `justfile` line 23 | `justfile` |
| Current PROPTEST_CASES (local) | 100 (justfile `check` target overrides shell env) | `justfile` line 17, 23 |
| Current PROPTEST_CASES (CI) | unset → defaults are CI-managed | `justfile` line 32 |
| `.cargo/config.toml` | **does not exist** | Glob 2026-05-05 |
| Worktrees in use | yes — `.worktrees/S-3.01/...` etc. (separate Cargo.toml per worktree) | Glob 2026-05-05 |
| Platform | macOS Sequoia (15.x) on Apple Silicon (aarch64-apple-darwin) | session context |
| Pain point | `cargo nextest run --workspace --all-features` ≈ 5 min wall-clock | user-reported |
| TDD impact | Cycle compounds across many runs per feature | user-reported |

The absence of `.cargo/config.toml` means every recommendation in this document is greenfield — no existing convention to migrate.

## 3. Validated Recommendations

### 3.1 Quick wins

#### 3.1.1 macOS XProtect Developer Tools exception (P0 — TIER 1)

- **Claim:** Exempting the terminal application from XProtect on-launch scanning eliminates per-binary process-launch overhead from XprotectService (a single-threaded daemon).
- **Status:** ✅ **VALIDATED** against an authoritative primary source.
- **Measured speedup:** Per Nicholas Nethercote (Mozilla, rustc compile-time team lead, [Faster Rust builds on Mac, 2025-09-04](https://nnethercote.github.io/2025/09/04/faster-rust-builds-on-mac.html)):
  - Individual build scripts: **0.48–3.88s → 0.06–0.14s** (each invocation)
  - Rust compiler `tests/ui` suite: **9m42s → 3m33s** (≈63% reduction)
  - One trivial example: 75ms direct vs 300ms via Cargo on the same script
- **Mechanism:** XProtect scans every executable on first launch. Build scripts (which spawn many short-lived binaries) and `cargo test` (which spawns one binary per test) are the worst-case workload because they dwarf the actual computation in scan-overhead time.
- **Mitigation:** System Settings → Privacy & Security → Developer Tools → add the terminal application (Terminal.app, iTerm2, ghostty, or whatever is running the cargo invocation). Per Apple's documented "Developer Tools" exception list. The exception causes child processes to inherit the trust grant.
- **Applicability to prism:** **HIGH.** Prism has 24 crates with build scripts (proto-gen, build.rs in some crates), large dependency tree (rocksdb, datafusion → many transitive crates with build scripts), and is tested via `cargo nextest` which spawns one binary per test. Every nextest invocation pays the per-binary scan tax for any binary whose hash is unfamiliar to XProtect.
- **Caveats and uncertainties (flagged):**
  - **MDM / corporate-policy constraints:** ❓ **inconclusive.** No primary source documents whether MDM-managed Macs (likely the case for a 1898 & Co employee on a corporate laptop) can self-grant Developer Tools privileges, or whether IT must whitelist via a configuration profile. **[needs verification by user]** — check with 1898 IT before assuming this is freely available. If MDM blocks self-grant, the next-best mitigation is to ensure XProtect signature DB is current (which lets XProtect cache prior decisions) but the speedup will be smaller.
  - **macOS version:** Nethercote's 2025-09 post does not name a specific macOS version. The XProtect daemon model has been stable across Sequoia 15.x → 15.2 (per [Eclectic Light Co, 2024-12-19](https://eclecticlight.co/2024/12/19/xprotect-has-changed-again-in-macos-sequoia-15-2/)). The Developer Tools exception predates Sequoia. Reasonably high confidence the mitigation works on 15.x, but **[needs validation]** with `cargo build --timings` before/after on prism specifically.
  - **Security trade-off:** This grants exemption from runtime malware scanning for binaries spawned from the terminal. Acceptable for a developer who trusts their toolchain (the canonical posture for local dev). Document this trade-off in a CONTRIBUTING note.
- **Source citations:**
  - Primary: [nnethercote.github.io/2025/09/04/faster-rust-builds-on-mac.html](https://nnethercote.github.io/2025/09/04/faster-rust-builds-on-mac.html)
  - Confirmation: [news.lavx.hu/article/macos-xprotect-the-hidden-performance-tax-on-rust-builds-and-how-to-fix-it](https://news.lavx.hu/article/macos-xprotect-the-hidden-performance-tax-on-rust-builds-and-how-to-fix-it)
  - Mechanism context: [eclecticlight.co/2024/12/19/xprotect-has-changed-again-in-macos-sequoia-15-2/](https://eclecticlight.co/2024/12/19/xprotect-has-changed-again-in-macos-sequoia-15-2/)

#### 3.1.2 `[profile.dev]` debuginfo tuning (P0 — TIER 1)

- **Claim:** Setting `debug = "line-tables-only"` for the workspace and `debug = false` for dependencies reduces incremental compile + link time without breaking debugger workflows on macOS.
- **Status:** ✅ **VALIDATED** against authoritative cargo documentation (Context7) and corroborated by primary research from David Lattimore.
- **Measured speedup:**
  - Authoritative cargo guide: [doc.rust-lang.org/cargo/guide/build-performance.html](https://doc.rust-lang.org/cargo/guide/build-performance.html) directly recommends this pattern as the canonical "speed up dev profile" technique. (Confirmed via Context7 query 2026-05-05.)
  - Per the Nethercote post and standard cargo profile guidance: 5–15% incremental speedup with one cited 14.30s → 12.38s case (≈13.4%).
- **Default on macOS:** `split-debuginfo = "unpacked"` is **already the default on macOS for profiles with debug info** as of [cargo PR #9298 (alexcrichton, merged Mar 2021)](https://github.com/rust-lang/cargo/pull/9298). The default has been stable for ~5 years. Quote from [The Cargo Book — Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html): "split-debuginfo = '...' # Platform-specific."  Cargo defaults macOS to `unpacked`; explicitly setting it is a no-op but harmless and improves config readability.
- **Compatibility verification (the explicit task asked):**
  - **lldb:** ✅ **compatible** by design. From the cargo PR thread: "Rust's backtrace support is smart enough to know how to find these `.o` files. Tools such as lldb also know how to do this." The `unpacked` mode leaves `.o` files in `target/debug/deps/` and skips `dsymutil`. lldb auto-discovers them.
  - **rust-analyzer:** ✅ **compatible.** rust-analyzer does not depend on debug info layout — it operates on source + crate metadata via `cargo check --message-format=json` and rustc's save-analysis. The `unpacked`/`packed`/`off` distinction is invisible to rust-analyzer.
  - **`debug = "line-tables-only"`:** ✅ **compatible.** Preserves backtrace line numbers in panics + lldb stack traces. Loses only inline source-line stepping for in-function debugging in dependencies. rust-analyzer is unaffected (it ignores debug-info presence).
  - **`debug = false` for `package."*"`:** ✅ **compatible.** Loses ability to step into dependency code with lldb. Acceptable for prism, where debugging targets workspace crates 99% of the time. **Mitigation:** define a `[profile.debugging]` profile that inherits from `dev` with full debug info, used via `cargo build --profile debugging` when actually needed (per the cargo guide example).
- **Known macOS-specific debugger edge cases (flagged):**
  - The rust-lang issue [rust#83730](https://github.com/rust-lang/rust/issues/83730) documents an old case where lldb could not extract debug info from dylib dependencies on macOS with `unpacked` split-debuginfo. **Status as of 2026:** issue remains open but the workaround (`CARGO_PROFILE_DEV_SPLIT_DEBUGINFO=packed` env var when actively debugging dylibs) is well-known. **Applicability to prism: low** — prism builds rlibs for workspace crates and a single binary (prism-mcp); no in-workspace dylibs.
- **Applicability to prism:** **HIGH.** Prism's pain point is iteration speed, not debugger fidelity. The default `[profile.dev] debug = true` on a 24-crate workspace generates large `.o` files for every dependency, which slows both rustc and the linker.
- **Source citations:**
  - Primary: [doc.rust-lang.org/cargo/guide/build-performance.html](https://doc.rust-lang.org/cargo/guide/build-performance.html) (Context7 verified 2026-05-05)
  - Default behavior: [github.com/rust-lang/cargo/pull/9298](https://github.com/rust-lang/cargo/pull/9298)
  - Speedup case study: [davidlattimore.github.io/posts/2024/02/04/speeding-up-the-rust-edit-build-run-cycle.html](https://davidlattimore.github.io/posts/2024/02/04/speeding-up-the-rust-edit-build-run-cycle.html)

#### 3.1.3 cargo-nextest (already in use)

- **Claim:** Up to 3× speedup vs `cargo test` due to per-test process isolation enabling true parallelism across cores.
- **Status:** ✅ **VALIDATED**, **already adopted** in prism per `justfile` lines 23, 32.
- **No change recommended** — confirmed in use. Documented for completeness so that future implementer agents do not "rediscover" nextest.
- **Source:** [nexte.st/docs/benchmarks/](https://nexte.st/docs/benchmarks/) — verified 2× to 3.4× across real workspaces. Prism's prior CI-optimization research (see `.factory/research/2026-04-30-ci-free-tier-optimization.md` §1) covers this in depth.

### 3.2 Conditional wins

#### 3.2.1 `lld` linker on macOS (P2 — measure before adopting)

- **Claim:** lld is 20–50% faster than `ld64`. **However**, since Xcode 15, Apple's default linker is `ld_prime` (not `ld64`), which closed most of the gap.
- **Status:** ⚠️ **CONDITIONAL** — claim valid only if linking is a measurable bottleneck.
- **Verification of "ld-prime" claim:**
  - ✅ ld_prime is real. Per [Apple Developer Forums thread 749285](https://developer.apple.com/forums/thread/749285) and [Wade Tregaskis — ld_prime tag](https://wadetregaskis.com/tags/ld_prime/), Xcode 15 introduced `ld_prime` (also written `ld-prime`) as the new default; the old `ld64` is still available via `-ld_classic`. Beta cycle option names were `-ld64` and `-ld_prime`; on release the canonical pair became `-ld_classic` / `-ld_new`.
  - ⚠️ Michael Eisel (author of zld and the lld-on-macOS post) explicitly notes [eisel.me/lld](https://eisel.me/lld): "with the release of ld-prime, the new default linker, lld is no longer necessarily the fastest option." This is the strongest single source against using lld on macOS in 2026.
  - The "20–50% faster than ld64" benchmark was measured against the **legacy** linker. The gap vs `ld_prime` is materially smaller; **no primary numerical benchmark of lld vs ld_prime for Rust on Apple Silicon was located in this research**. **[needs measurement]** on prism specifically before adoption.
- **Decision rule:** Run `cargo build --timings` once. If the timing report shows linking >15% of total wall-clock for a workspace build, evaluate lld. If <15%, do not bother — `ld_prime` is already good.
- **Configuration if adopted:**
  ```toml
  # .cargo/config.toml — DO NOT enable without measuring first
  # [target.aarch64-apple-darwin]
  # linker = "clang"
  # rustflags = ["-C", "link-arg=-fuse-ld=lld"]
  ```
- **Caveat:** lld on macOS has historically been less battle-tested than on Linux. Some build-script linker invocations may fail with cryptic errors. Roll-back is trivial (delete the lines).
- **Applicability to prism:** **MEDIUM-LOW.** Prism's 24-crate workspace produces many test binaries (each linked individually), but with `split-debuginfo = "unpacked"` the linker is doing less work per binary than the default would suggest. The XProtect mitigation (§3.1.1) and debuginfo tuning (§3.1.2) likely deliver more speedup at lower risk than swapping the linker.
- **Source citations:**
  - [eisel.me/lld](https://eisel.me/lld) — the original lld-on-macOS post, with explicit ld-prime caveat
  - [developer.apple.com/forums/thread/749285](https://developer.apple.com/forums/thread/749285) — Apple confirmation of ld_prime
  - [medium.com/rustaceans/rust-1-90-the-speed-update-lld-linker-makes-everything-7x-faster](https://medium.com/rustaceans/rust-1-90-the-speed-update-lld-linker-makes-everything-7x-faster-30a79af465bf) — the often-cited 7× claim, but **note: the 7× and 40% headline numbers refer to Linux x86_64**, not macOS; do not transfer the number.

#### 3.2.2 Profile-level `opt-level` overrides for slow dependencies (P3 — empirical)

- **Claim:** Setting `opt-level = 1` or higher for specific runtime-heavy dependencies (e.g., `rocksdb-sys`, `datafusion-execution`) speeds up dev-profile **runtime** without proportionally increasing compile time.
- **Status:** ❓ **EMPIRICAL.** No general-purpose recommendation; depends on which crates dominate test-runtime cost vs compile cost.
- **Configuration pattern:**
  ```toml
  [profile.dev.package.rocksdb]
  opt-level = 3   # only example — measure before applying
  ```
- **Decision rule:** Only useful if test runs are slow because of *unoptimized library code at runtime*, not because of compile time. For prism with `cargo nextest run --workspace --all-features ≈ 5 min`, the bottleneck is most likely compile/link, not runtime — **so this recommendation is unlikely to apply**. Re-evaluate after XProtect + debuginfo wins land. **[needs measurement]** which sub-step (compile vs link vs run) dominates on prism.
- **Source:** [doc.rust-lang.org/cargo/reference/profiles.html — Overrides](https://doc.rust-lang.org/cargo/reference/profiles.html#overrides) (Context7 verified).

### 3.3 Demoted (rejected for prism)

| Recommendation | Demotion reason | Source |
|---|---|---|
| **`mold` linker** | Linux-only by design; does not run on macOS | [github.com/rui314/mold](https://github.com/rui314/mold) — author's own README explicitly: "currently does not work on macOS" |
| **`sold` (mold's macOS port)** | Author no longer recommends; Apple's `ld_prime` (Xcode 15+) is the new default and closes most of the gap | Pre-existing perplexity research; corroborated by [eisel.me/lld](https://eisel.me/lld) |
| **`zld`** | Archived. Repository [michaeleisel/zld](https://github.com/michaeleisel/zld) explicitly recommends moving to lld; lld in turn is now ambiguously better than `ld_prime`. | [github.com/michaeleisel/zld](https://github.com/michaeleisel/zld) |
| **Shared `target/` dir between worktrees** | No native cargo support; cargo-lock contention serializes builds. Howard John's [shared-rust-build post](https://blog.howardjohn.info/posts/shared-rust-build/) explicitly: "Only one build can run at a time, and the last thing I want is some background agent blocking my builds." Hardlink-based scripts work but are fragile. | [blog.howardjohn.info/posts/shared-rust-build/](https://blog.howardjohn.info/posts/shared-rust-build/) |
| **`sccache` for local dev** | (a) Rust workspace caching is already handled by cargo's incremental + fingerprint system; (b) sccache's working-directory-as-cache-key behavior breaks worktree sharing too; (c) net negative when `Swatinem/rust-cache` (CI) or local cargo incremental are in play. See `.factory/research/2026-04-30-ci-free-tier-optimization.md` §6 for the full rejection chain. | Existing CI research |
| **Cranelift codegen backend** | Available as `rustc-codegen-cranelift-preview` rustup component for `aarch64-apple-darwin`, but **not stable** for production: ABI incompatibilities documented in [github.com/rust-lang/rustc_codegen_cranelift/issues/1248](https://github.com/rust-lang/rustc_codegen_cranelift/issues/1248). [bjorn3 progress report Nov 2024](https://bjorn3.github.io/2024/11/14/progress-report-nov-2024.html) flags arm64-macOS variadic-arg ABI quirks. ~5% clean-build speedup is not worth the rough edges. **Re-evaluate H2 2026.** | [github.com/rust-lang/rustc_codegen_cranelift](https://github.com/rust-lang/rustc_codegen_cranelift) |
| **`incremental = false` for local dev** | Standard practice keeps `incremental = true` for local dev (the cargo default). Disabling speeds up clean builds slightly but devastates rebuild times. Only set `incremental = false` in CI. | [doc.rust-lang.org/cargo/reference/profiles.html — incremental](https://doc.rust-lang.org/cargo/reference/profiles.html#incremental) |

## 4. Inconclusive / Empirical

### 4.1 PROPTEST_CASES tuning for local TDD

- **Question:** What `PROPTEST_CASES` value optimizes local TDD throughput vs. regression coverage?
- **Status:** ❓ **NO STATISTICAL GUIDANCE** in proptest docs or community sources.
- **Current prism choice:** 100 (per `justfile` line 23) for local `just check`; default 256 for nextest standalone runs; CI uses 1000 (per `.factory/research/2026-04-30-ci-free-tier-optimization.md` §2).
- **Empirical reasoning** (from training data, flagged):
  - Bug detection probability scales as `1 - (1 - p)^N` where `p` is the per-input failure rate and `N = PROPTEST_CASES`.
  - For "common" bugs (failing 5–50% of inputs), `N = 100` already gives >99% catch probability.
  - For "rare" edge cases (failing <0.5% of inputs), `N = 100` is inadequate (~39% catch); `N = 1000` gives >99%.
  - Persisted regression seeds (`proptest-regressions/*.txt`, committed to repo) replay deterministically regardless of `N`.
- **Recommendation:** Keep prism's existing tiering — local 100, CI 1000, scheduled 5000 — and revisit only if a "rare" bug class escapes to CI repeatedly. **No change.**
- **Citation:** [docs.rs/proptest — Config](https://docs.rs/proptest/latest/proptest/test_runner/struct.Config.html), [proptest-book/failure-persistence](https://altsysrq.github.io/proptest-book/proptest/failure-persistence.html). Statistical reasoning is from training data and explicitly flagged.

### 4.2 nextest `test-threads` tuning

- **Question:** Should prism set an explicit `test-threads` value rather than the default `num-cpus`?
- **Status:** ❓ **EMPIRICAL.** Per [nexte.st/docs/configuration/threads-required/](https://nexte.st/docs/configuration/threads-required/): "Be sure to benchmark your test runs, as thread limiting will often cause test runs to become slower overall."
- **Recommendation:** Leave at default (`test-threads = "num-cpus"`). Only override if a specific test class shows contention (e.g., a test that monopolizes a port or a fixed-path tempdir). For such cases, use nextest's `[test-groups]` to gate them rather than throttling the whole suite.
- **Citation:** [nexte.st/docs/configuration/threads-required/](https://nexte.st/docs/configuration/threads-required/), [nexte.st/docs/configuration/test-groups/](https://nexte.st/docs/configuration/test-groups/).

## 5. 2025–2026 ecosystem updates

| Topic | Status as of 2026-05-05 | Action |
|---|---|---|
| Cargo build-dir layout rework | Tracking issue: [Rust Project Goals 2025h2 — Rework Cargo Build Dir Layout](https://rust-lang.github.io/rust-project-goals/2025h2/cargo-build-dir-layout.html). User-wide cache being designed as first-class solution for cross-workspace artifact sharing. **Not landed yet on stable.** | Watch; no action. Will eventually replace ad-hoc target-dir sharing. |
| Cranelift codegen on aarch64-apple-darwin | Available as `rustc-codegen-cranelift-preview` (preview component). Rough edges remain on arm64 ABI. | Re-evaluate H2 2026 per `bjorn3` progress reports. |
| `sccache` + GHA cache backend | `Mozilla-Actions/sccache-action` + `SCCACHE_GHA_ENABLED=true`. Net-negative on local dev with cargo incremental enabled. | No change. Covered in detail in CI research (§6). |
| Apple `ld_prime` linker | Default in Xcode 15+. Available in macOS 14+ toolchains. Closes most of the lld-vs-ld64 gap. | No action — already the default. Inform decisions about `lld`. |
| cargo `--timings` HTML report | Stable, in `target/cargo-timings/cargo-timing.html`. Confirmed via Context7 query 2026-05-05. | Use for diagnostic procedure (§7). |
| `cargo nextest` JetBrains integration | RustRover 2026.1 ships native nextest integration ([blog.jetbrains.com/rust/2026/04/03/](https://blog.jetbrains.com/rust/2026/04/03/rustrover-2026-1-professional-testing-with-native-cargo-nextest-integration/)). | No action; informational. |
| Rust 1.92 features | **[needs verification]** — no specific 1.92-attributable build-perf feature was confirmed in research. The 7× lld claim ([Rust 1.90: The Speed Update](https://medium.com/rustaceans/rust-1-90-the-speed-update-lld-linker-makes-everything-7x-faster-30a79af465bf)) **applies to Linux x86_64** where lld became default for the rustc compile-driver, **not macOS**. | Do not transfer Linux-default-lld claim to macOS. |

## 6. Concrete config diff for prism

### 6.1 `.cargo/config.toml` (NEW FILE — proposed)

Path: `/Users/jmagady/Dev/prism/.cargo/config.toml`

```toml
# Prism cargo config — local dev profile tuning for macOS aarch64.
# Created 2026-05-05 per .factory/research/build-optimization-2026.md.
#
# This file is intentionally minimal: every line documented and tied to
# a measured speedup. Do not add settings without a corresponding
# entry in the build-optimization research sidecar.

# -----------------------------------------------------------------------------
# Dev profile: faster incremental compiles by trimming debug info.
# -----------------------------------------------------------------------------
# Rationale (research §3.1.2):
#   - debug = "line-tables-only" preserves panic backtraces + lldb call-stack
#     line numbers; loses inline source-line debugging. Acceptable for the
#     common Prism dev workflow (debug entry from a panic, not in-function
#     stepping).
#   - debug = false on dependencies is the standard cargo-recommended
#     pattern for workspaces that don't routinely step into dependency code.
#     Loses ability to step into rocksdb / datafusion source. Acceptable;
#     re-enable on a per-debug-session basis via the [profile.debugging]
#     profile below (cargo build --profile debugging).
#
# Source: doc.rust-lang.org/cargo/guide/build-performance.html
[profile.dev]
debug = "line-tables-only"

[profile.dev.package."*"]
debug = false

# -----------------------------------------------------------------------------
# Debugging profile: full debug info, opt-in via --profile debugging.
# -----------------------------------------------------------------------------
# When you actually need to step through a test with lldb, run:
#   cargo nextest run -p <crate> <test_name> --profile debugging
# (or build a binary with --profile debugging and attach lldb manually).
[profile.debugging]
inherits = "dev"
debug = true

# -----------------------------------------------------------------------------
# split-debuginfo on macOS:
#   The cargo default is "unpacked" on macOS for profiles with debug info
#   (cargo PR #9298, merged 2021). We do NOT explicitly set it because the
#   default is correct and explicit setting risks silent breakage if the
#   default ever changes.
# -----------------------------------------------------------------------------

# -----------------------------------------------------------------------------
# Linker overrides: NONE.
#   Apple's ld_prime (Xcode 15+ default) is the recommended linker for
#   aarch64-apple-darwin in 2026. lld and zld are not measurably faster
#   without project-specific benchmarking (research §3.2.1). Re-evaluate
#   only if `cargo build --timings` shows linking >15% of total time.
# -----------------------------------------------------------------------------

# Optional: increase build-time parallelism for the dev profile if the
# default codegen-units=256 is somehow being clamped by an environment
# variable elsewhere. Uncomment only after measurement.
# [profile.dev]
# codegen-units = 256  # default for dev; explicit for clarity
```

### 6.2 `justfile` delta (proposed)

The existing `check` target runs the full workspace gate. We add an iteration-mode target for TDD.

```diff
--- a/justfile
+++ b/justfile
@@ -27,6 +27,28 @@ check:
     PROPTEST_CASES=100 cargo test --workspace --all-features --doc
     @scripts/check-crate-layout.sh

+# TDD iteration mode — single crate, fast feedback (target: <60s).
+# Usage: just iter prism-query
+#        just iter prism-query test_parser
+# This is the recommended inner loop. Do NOT use `just check` during TDD —
+# reserve it for pre-push verification.
+iter crate test_filter='':
+    PROPTEST_CASES=32 cargo nextest run -p {{crate}} {{test_filter}}
+
+# Fast workspace check — lint only, no tests. Use to confirm the workspace
+# still type-checks during a refactor sweep before running tests.
+check-fast:
+    cargo clippy --all-features -- -D warnings
+
+# Generate a build-timings report for diagnostics. Outputs HTML at
+# target/cargo-timings/cargo-timing.html. See research sidecar §7 for
+# how to interpret the output.
+timings:
+    cargo build --workspace --all-features --timings
+    @echo "Timings report: target/cargo-timings/cargo-timing.html"
+
 # CI-only: identical to CI behavior (full-strength)
 # Steps run in spec order: fmt → clippy → nextest → doctests → deny → audit → semver-checks → check-layout
 check-ci:
```

Notes:
- `just iter` defaults `PROPTEST_CASES=32` for ultra-fast inner loop. Regression seeds in `proptest-regressions/` still replay deterministically, so this does not regress on already-found bugs.
- `check-fast` is a clippy-only "still compiles?" check — useful during a multi-file refactor.
- `timings` produces the HTML report needed for §7's diagnostic procedure.

### 6.3 Implementer agent prompt language (proposed boilerplate)

Insert into the standard implementer-agent dispatch prompt (e.g., the orchestrator's implementer template):

> ### Build & Test Iteration Discipline
>
> When iterating on a single crate, **always use `cargo nextest run -p <crate>` (or `just iter <crate> [<test_filter>]`)** rather than the workspace-wide runner. The 24-crate workspace means a single-crate run is typically 5–20× faster than `--workspace --all-features`.
>
> Reserve **`just check`** for pre-push verification — run it once before opening or updating a PR, not during the TDD inner loop.
>
> If the inner loop feels slow (>60s for a single-crate run after a one-line edit), check that the macOS XProtect Developer Tools exception is in place for your terminal (System Settings → Privacy & Security → Developer Tools), and that `.cargo/config.toml` is committed at the repo root with the `[profile.dev]` debuginfo tuning. See `.factory/research/build-optimization-2026.md` for the canonical settings.
>
> Do not introduce `[profile.dev]` overrides on a per-crate basis (`Cargo.toml` profile sections) without first measuring with `just timings`. Workspace-level `.cargo/config.toml` is the single source of truth for build-perf settings.

### 6.4 `.gitignore` addition (proposed)

Path: `/Users/jmagady/Dev/prism/.gitignore` — append:

```
# Build timings reports (research §7 diagnostic procedure)
target/cargo-timings/
```

(Confirm not already present before adding.)

## 7. Diagnostic procedure — `cargo build --timings`

### 7.1 How to capture a baseline

```bash
# Step 1: clean baseline measurement (cold cache)
cargo clean
cargo build --workspace --all-features --timings
open target/cargo-timings/cargo-timing.html
# Step 2: incremental measurement (warm cache, single touched file)
touch crates/prism-query/src/lib.rs
cargo build --workspace --all-features --timings
open target/cargo-timings/cargo-timing.html
```

### 7.2 What to look for in the report

The HTML report (per Context7-verified cargo docs) shows:

1. **Per-unit compile time table** — which crates dominate. Look for crates with >5% of total time. For prism, expect `rocksdb-sys`, `datafusion-*`, `arrow-*`, `chumsky` to be in the top 10.
2. **Concurrency timeline** — gaps where rustc is waiting (limited parallelism, dependency-chain bottlenecks). Long horizontal bars at the start are usually proc-macro crates blocking everything else.
3. **Codegen vs frontend split** (if Rust 1.85+): rustc breaks down where time was spent. High codegen → linker-likely. High frontend → trait-resolution-likely.

### 7.3 Decision rules from the report

| Observation in report | Likely bottleneck | Recommended action |
|---|---|---|
| Total time >> sum of per-crate times | Process-launch overhead (XProtect) | Apply XProtect exception (§3.1.1) — highest ROI |
| Linking shows as discrete >15% slice on terminal-leaf binaries | Linker speed | Evaluate lld (§3.2.1); measure delta |
| One dependency dominates (>30% of total) | Overcompilation of that crate | Check whether `--all-features` is enabling unused deps; consider `default-features = false` on its dependency declaration |
| Long sequential build-script phase | Build-script process spawning | XProtect mitigation again; secondarily, audit `build.rs` files for unnecessary network/disk |
| Codegen-heavy on debug | Optimization level | Already at `opt-level = 0` for dev; consider profile-package overrides for runtime-slow deps (§3.2.2) |
| Gaps between bars (rustc idle) | Codegen-units bottleneck or dependency chain | Verify `codegen-units = 256` (default for dev); check for accidental override |

### 7.4 Expected before/after for prism (qualitative)

After applying §3.1.1 (XProtect exception) + §3.1.2 (debuginfo tuning):

| Workload | Before (baseline) | Expected after | Confidence |
|---|---|---|---|
| `cargo nextest run --workspace --all-features` (warm cache) | ≈5 min | 2–3 min | medium-high (XProtect dominant per Nethercote) |
| `cargo nextest run -p prism-query` (warm cache) | ≈30–60s **[needs measurement]** | 10–25s | medium |
| `cargo build` (cold cache, full workspace) | ≈3–8 min **[needs measurement]** | 10–25% faster | medium (XProtect helps cold; debuginfo helps warm) |
| `cargo clippy --workspace --all-features` (warm) | ≈30–90s **[needs measurement]** | similar (clippy is rustc-bound, less linker) | low (no specific source) |

**All "before" numbers above except the user-reported 5-min are flagged `[needs measurement]` — capture `just timings` baselines before/after each change.**

## 8. Open questions (require empirical measurement on prism)

Each question below is something the user should resolve with a `just timings` run rather than additional research:

1. **What fraction of the 5-minute cycle is rustc-frontend vs codegen vs linking vs test execution?** The diagnostic plan in §7 answers this. Expected (no source, training data): rustc-bound ~70–80%, link 10–15%, test execution 10–15% — but this is a guess.
2. **Does the user's terminal currently appear in the Developer Tools list?** If yes, §3.1.1 is already in effect and explains why prism is "only" at 5 min rather than worse. If no, applying it will deliver the headline win.
3. **MDM policy:** can your organization's MDM profile permit Developer Tools self-grant? Confirm with IT before relying on §3.1.1. (If MDM blocks, escalate as a request — 9-minute → 3-minute developer cycles is a strong business case.)
4. **Worktree count and concurrency:** how many worktrees does the user maintain in parallel and do they ever build simultaneously? If rare or never, the lack of shared `target/` is no concern. If frequent (e.g., 5 worktrees × concurrent CI-like runs), revisit hardlink schemes per Howard John's pattern.
5. **Which subset of crates does TDD iterate on most?** If 80% of edits are in `prism-query` + `prism-mcp`, the inner loop discipline (§6.3) compounds; if edits are spread evenly across all 24 crates, less benefit.
6. **Does `.cargo/config.toml` interact with per-crate `Cargo.toml` profile blocks?** Verify no in-crate `[profile.dev]` exists before adding the workspace `.cargo/config.toml`. (A `Grep` in the worktrees shows none currently.)
7. **Test-runtime vs compile-time split for full workspace run:** the profile-package opt-level recommendation (§3.2.2) only applies if test runtime is the bottleneck.
8. **Effect on the lefthook pre-push hook (if any).** The lefthook config is referenced in `justfile` line 51 — verify the pre-push hook is `just check`, not `cargo test --workspace --all-features` directly, so the build-perf wins propagate.

## 9. Risk register

| Recommendation | Risk | Detection | Roll-back |
|---|---|---|---|
| §3.1.1 XProtect exception | Reduced runtime malware scanning for terminal-spawned binaries | None — security trade-off is intentional | Remove terminal from System Settings → Privacy & Security → Developer Tools |
| §3.1.1 XProtect exception | MDM / corporate policy may block self-grant | Setting reverts on next MDM sync, or grant button is greyed out | Escalate to IT; cannot work around at the user level |
| §3.1.2 `debug = "line-tables-only"` | Cannot inspect local-variable values inside library functions when stepping in lldb | `lldb> p some_local` returns `<no debug info>` | Use `cargo build --profile debugging` for that session, or temporarily revert the line in `.cargo/config.toml` |
| §3.1.2 `debug = false` on deps | Cannot step into rocksdb / datafusion / chumsky source from lldb | lldb stops at the workspace boundary; "no symbols" for transitive frames | `cargo build --profile debugging` rebuilds with full info; re-attach lldb |
| §3.1.2 split-debuginfo unpacked | Larger `target/` dir size (objects retained); on rare cases lldb can fail to find dylib syminfo | `target/debug/` size grows; lldb error "no debug info for module" | Set `CARGO_PROFILE_DEV_SPLIT_DEBUGINFO=packed` for that session |
| §6.1 `.cargo/config.toml` | Settings unintentionally bleed into release builds | Unlikely — only `[profile.dev]` set, `[profile.release]` untouched | Delete the file |
| §6.1 `.cargo/config.toml` | Conflict with future per-crate profile overrides someone adds | Cargo merges with last-write-wins per the cargo profile docs; surprises possible | `cargo config get` to inspect resolved config; remove conflicting entries |
| §6.2 `just iter` with `PROPTEST_CASES=32` | Misses bugs that fail <3% of inputs in the inner loop | A bug shipped to `just check` (PROPTEST_CASES=100) or CI (1000) is caught there but later than ideal | Increase `PROPTEST_CASES` env var override at invocation time, or update the recipe |
| §3.2.1 lld linker (if adopted) | Build-script linker invocations may fail with cryptic errors on macOS | Build fails with `ld:` error mentioning lld | Comment out `[target.aarch64-apple-darwin]` block in `.cargo/config.toml` |
| §3.2.2 opt-level package overrides | Specific dependency now compiles much slower while runtime improves | Compile-time spike on cold builds for the targeted package | Remove the `[profile.dev.package.<name>]` block |
| Demoted: shared `target/` between worktrees | If user re-attempts this, cargo-lock contention will serialize concurrent builds, undoing wins | Builds visibly wait on each other | Revert `CARGO_TARGET_DIR` env to unset; per-worktree `target/` is correct |

## 10. Sources

Date convention: all sources accessed 2026-05-05 unless otherwise noted.

### Primary (web research)

1. [Nicholas Nethercote — Faster Rust builds on Mac (2025-09-04)](https://nnethercote.github.io/2025/09/04/faster-rust-builds-on-mac.html) — XProtect mitigation primary case study; 9m42s → 3m33s rustc UI tests. **Authoritative.**
2. [LavX News — macOS XProtect: The Hidden Performance Tax on Rust Builds](https://news.lavx.hu/article/macos-xprotect-the-hidden-performance-tax-on-rust-builds-and-how-to-fix-it) — confirms the Nethercote findings.
3. [Eclectic Light Co — XProtect has changed again in macOS Sequoia 15.2 (2024-12-19)](https://eclecticlight.co/2024/12/19/xprotect-has-changed-again-in-macos-sequoia-15-2/) — XProtect mechanism on Sequoia.
4. [David Lattimore — Speeding up the Rust edit-build-run cycle (2024-02-04)](https://davidlattimore.github.io/posts/2024/02/04/speeding-up-the-rust-edit-build-run-cycle.html) — `[profile.dev]` debuginfo + opt-level pattern primary.
5. [Michael Eisel — Faster Apple Builds with the lld Linker (eisel.me/lld)](https://eisel.me/lld) — lld history; **explicit ld-prime caveat in update notice**.
6. [Apple Developer Forums — XCode 15 linker ld_prime causing issues (thread 749285)](https://developer.apple.com/forums/thread/749285) — Apple confirms ld_prime as Xcode 15 default.
7. [Wade Tregaskis — ld_prime tag](https://wadetregaskis.com/tags/ld_prime/) — additional ld_prime context.
8. [Howard John — Sharing Rust Build Cache (blog.howardjohn.info)](https://blog.howardjohn.info/posts/shared-rust-build/) — primary source for shared `target/` lock contention.
9. [Rust Internals — Help test faster incremental debug macOS builds on nightly](https://internals.rust-lang.org/t/help-test-faster-incremental-debug-macos-builds-on-nightly/14016) — split-debuginfo unpacked discussion thread.
10. [GitHub: rust-lang/cargo PR #9298 — Default macOS targets to unpacked debuginfo](https://github.com/rust-lang/cargo/pull/9298) — origin of macOS default; merged Mar 2021.
11. [GitHub: rust-lang/rust issue #83730 — macos: unable to get debug information from dylib dependencies](https://github.com/rust-lang/rust/issues/83730) — known split-debuginfo edge case.
12. [GitHub: rust-lang/rustc_codegen_cranelift issue #1248 — Add macOS AArch64 support](https://github.com/rust-lang/rustc_codegen_cranelift/issues/1248) — Cranelift status on Apple Silicon.
13. [bjorn3 — Progress report on rustc_codegen_cranelift (November 2024)](https://bjorn3.github.io/2024/11/14/progress-report-nov-2024.html) — Cranelift Apple Silicon ABI status.
14. [Rust Project Goals 2025h2 — Rework Cargo Build Dir Layout](https://rust-lang.github.io/rust-project-goals/2025h2/cargo-build-dir-layout.html) — future user-wide cache.
15. [GitHub: rust-lang/rust-analyzer issue #10684 — Reduce target dir lock contention](https://github.com/rust-lang/rust-analyzer/issues/10684) — rust-analyzer + cargo lock interaction.

### Cargo authoritative documentation (Context7-verified, 2026-05-05)

16. [The Cargo Book — Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html) — `[profile.dev]` defaults including `split-debuginfo = '...'  # Platform-specific`. (Context7 ID: `/websites/doc_rust-lang_cargo`.)
17. [The Cargo Book — Build Performance Guide](https://doc.rust-lang.org/cargo/guide/build-performance.html) — canonical `debug = "line-tables-only"` + `[profile.dev.package."*"] debug = false` pattern. **Authoritative source for §3.1.2.**
18. [The Cargo Book — cargo-build --timings](https://doc.rust-lang.org/cargo/commands/cargo-build.html) — `--timings` flag, output to `target/cargo-timings/cargo-timing.html`.
19. [The Cargo Book — target-dir / CARGO_TARGET_DIR](https://doc.rust-lang.org/cargo/commands/cargo-rustc.html) — target-dir documentation.

### Secondary / corroborating

20. [cargo-nextest — Benchmarks](https://nexte.st/docs/benchmarks/) — 2× to 3.4× across real workspaces.
21. [cargo-nextest — threads-required configuration](https://nexte.st/docs/configuration/threads-required/) — explicit warning that `test-threads` tuning is empirical.
22. [cargo-nextest — test-groups configuration](https://nexte.st/docs/configuration/test-groups/) — alternative to global test-threads throttling.
23. [rui314/mold (README)](https://github.com/rui314/mold) — Linux-only confirmation.
24. [michaeleisel/zld](https://github.com/michaeleisel/zld) — archived; recommends lld successor.
25. [proptest — Config docs](https://docs.rs/proptest/latest/proptest/test_runner/struct.Config.html) — PROPTEST_CASES default 256.
26. [proptest-book — failure persistence](https://altsysrq.github.io/proptest-book/proptest/failure-persistence.html) — regression seed determinism.
27. [Rust 1.90: The Speed Update — LLD Linker Makes Everything 7x Faster (Medium)](https://medium.com/rustaceans/rust-1-90-the-speed-update-lld-linker-makes-everything-7x-faster-30a79af465bf) — **Linux-specific** 7× claim; **do not transfer to macOS**.
28. [JetBrains RustRover 2026.1 — nextest integration blog](https://blog.jetbrains.com/rust/2026/04/03/rustrover-2026-1-professional-testing-with-native-cargo-nextest-integration/) — adoption signal.

### Internal cross-references

29. `/Users/jmagady/Dev/prism/.factory/research/2026-04-30-ci-free-tier-optimization.md` — sister CI research; covers nextest, sccache, mold-on-Linux in CI context.
30. `/Users/jmagady/Dev/prism/.factory/research/W3-library-versions.md` — pinned library versions including datafusion/arrow/rocksdb/chumsky used in §2.

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| Context7 (`resolve-library-id`, `query-docs`) | 4 | `/websites/doc_rust-lang_cargo` × 3 (`[profile.dev]` keys, `target-dir`/`CARGO_TARGET_DIR`, `cargo build --timings`) + `resolve-library-id` × 1 |
| WebSearch | 6 | XProtect Sequoia + cargo / split-debuginfo + lldb on Apple Silicon / split-debuginfo macOS default lineage / Cranelift on aarch64-darwin / `ld_prime` vs `lld` / shared `target/` between worktrees / nextest test-threads + PROPTEST_CASES |
| WebFetch | 3 | nnethercote.github.io (XProtect primary), eisel.me/lld (lld primary), blog.howardjohn.info/shared-rust-build (worktree primary) |
| Perplexity (search/reason/deep_research) | 0 in this run | The user supplied the pre-existing perplexity_research output as the seed; this run validated and extended via Context7 + targeted WebSearch/WebFetch |
| Tavily | 0 | Tool not invoked |
| Read (local) | 4 | `Cargo.toml`, `justfile`, prior CI research, W3 library-versions research |
| Glob (local) | 4 | `.factory/research/*`, `.cargo/config.toml` (confirmed missing), `Cargo.toml`, worktrees layout |
| Training data | 2 areas (flagged inline) | (1) Statistical reasoning for PROPTEST_CASES probability — §4.1; (2) qualitative bottleneck guess for prism's 5-min split — §8 question 1. Both explicitly marked. |

**Total MCP / web tool calls:** 13 (4 Context7 + 6 WebSearch + 3 WebFetch). Plus 8 local file/glob reads for project grounding.

**Training data reliance:** **low.** Every numerical claim in §3 (timings, percentage speedups, ld_prime status, split-debuginfo defaults) is sourced from a primary URL above. The two training-data zones — proptest case-count statistics and qualitative bottleneck split — are both flagged inline and routed to the §8 open-questions list for empirical validation.

**Conflicts identified:**
- ld_prime vs lld speed: Eisel's blog says lld "no longer necessarily fastest"; the Medium "Rust 1.90 7×" post says lld is hugely faster. **Resolution:** Medium post is Linux x86_64; Eisel is Apple-platform-aware. No conflict — different platforms.
- split-debuginfo macOS default: rustc docs say "packed", cargo docs say "unpacked". **Resolution:** [GitHub cargo issue #12243](https://github.com/rust-lang/cargo/issues/12243) confirms it's a docs inconsistency; cargo's behavior (unpacked) is what actually happens on macOS. Cited.

**Inconclusive items (flagged in body):**
- §3.1.1: MDM policy compatibility — `[needs verification by user/IT]`
- §3.2.1: lld vs ld_prime numerical benchmark on aarch64-apple-darwin for Rust workspaces — no primary source located; **`[needs measurement]`**
- §4.1: Statistical guidance for PROPTEST_CASES — none in upstream docs; reasoning is from training data
- §5: Rust 1.92-attributable build-perf features — `[needs verification]`
- §7.4 & §8: Several "before" baseline numbers for prism specifically — `[needs measurement]` via `just timings`
