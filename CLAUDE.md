# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

> **Toolchain:** Rust stable (per `rust-toolchain.toml`), edition 2024, resolver 2. Components: rustfmt, clippy, rust-src. Cross-compile targets: aarch64-apple-darwin, x86_64-apple-darwin, x86_64-unknown-linux-gnu, x86_64-unknown-linux-musl, x86_64-pc-windows-msvc. 24-crate workspace.

## Build & Test

```bash
# TDD inner loop — single crate, fast iteration (~10-30 sec warm)
just iter <crate> [test_filter]
# Examples:
just iter prism-query                              # all prism-query tests
just iter prism-query test_BC_2_11_006             # filtered
# PROPTEST_CASES=32 (8× lower than default 256 for speed; full coverage runs in `just check`)

# Pre-push gate — full strict workspace check (5-8 min cold, ~1 min warm)
just check          # fmt + clippy + nextest + doctests + crate-layout
just check-fast     # clippy + layout only (no tests; for refactor sweeps)

# CI-equivalent local run — adds deny + audit + semver-checks
just check-ci

# Diagnostics
just timings        # cargo build --timings HTML report → target/cargo-timings/
just clippy         # workspace clippy with -D warnings
just fmt            # cargo fmt --all
just cov            # coverage via cargo-llvm-cov

# Specialty (require external toolchain installs)
just kani-local     # Kani formal verification proofs
just fuzz-local <crate> <target>   # cargo-fuzz
just mutants        # mutation testing
just udeps          # unused-dep detection (requires nightly)

# Setup (idempotent)
just setup          # install all dev toolchain extensions
```

**DO NOT** use `cargo test --workspace` directly during iteration — `just iter <crate>` is 5-10× faster.

### TDD Inner Loop Discipline

When iterating through a TDD fix-burst (closing multiple findings in sequence), use the cheapest verification that proves what you need. Match the tool to the question:

| Question | Command | Time (warm) |
|---|---|---|
| Did my single fix make its target test pass? | `cargo nextest run -p <crate> -E 'test(<test_name>)'` | < 1s after build |
| Did my fix break anything in this crate? | `just iter <crate>` | 10-30s |
| See ALL failing tests at once (don't stop at first) | `cargo nextest run -p <crate> --no-fail-fast` | 30-60s |
| Final pre-push gate (workspace canonical) | `just check` | 1min warm / 5-8min cold |

**Common anti-pattern:** running `just check` (full workspace) between every TDD fix in a multi-finding burst. For a 10-fix burst this burns 10-50 minutes that adds nothing the per-crate run wouldn't already have caught. Reserve `just check` for ONCE at end of fix-burst before declaring done.

**Auto-iteration:** `cargo watch -x 'nextest run -p <crate> --no-fail-fast'` re-runs on save — useful for tight feedback when iterating on a single module.

**In-process vs subprocess tests:** Integration tests under `crates/<crate>/tests/` that spawn `prism start` as a subprocess each cost 200-800ms (subprocess overhead + RocksDB open). Unit tests inside `src/*.rs` `#[cfg(test)] mod tests` blocks run in-process at ~5ms. For tight inner-loop iteration on logic, prefer unit tests; reserve subprocess integration tests for behavior that genuinely needs the full binary.

**Deep recursion tests** (depth ≥ 50) MUST wrap with `crates/prism-query/src/tests/util.rs::run_with_deep_stack` to avoid SIGBUS on macOS aarch64's 2MB default test thread stack. See SIGBUS triage in `.factory/STATE.md` D-242 / pass-9.

## Formal Verification (Kani)

Verification properties VP-014 (size limit) and VP-015 (depth limit) have Kani proofs in `crates/prism-query/src/proofs/`. Run them locally with:

```bash
just kani-local            # all crate proofs
cargo kani -p prism-query  # prism-query proofs only
```

**Platform support:** Kani is **Linux/macOS only** (upstream Kani uses CBMC as its backend; Windows is not supported by the Kani project). The `kani-verifier` dev-dependency is gated to non-Windows in `crates/prism-query/Cargo.toml`. Windows contributors should rely on concrete unit tests + CI's Linux/macOS proof job — proof validity is platform-agnostic (Rust code is the same on all platforms; one proof = truth for all).

VP coverage layers:
- **Kani proof** (formal, exhaustive within bounds) — Linux/macOS only
- **Concrete unit tests** (specific points, deterministic) — all platforms
- **Fuzz target `vp021_parse_fuzz`** (random exploration) — Linux CI smoke + nightly long-run

## Git Workflow

- **Default branch:** `main` (release branch, infrequent commits)
- **Active development:** `develop` (PRs target `develop`)
- **Feature branches:** `feature/<story-id>` (e.g., `feature/S-3.01`)
- **Maintenance branches:** `maintenance/<scope>` (e.g., `maintenance/rename-crowdstrike-session`)
- **Worktree pattern:** per-story worktrees in `.worktrees/<story-id>/` for parallel work
- **Commit conventions:** Conventional Commits enforced by `lefthook.yml` (`pre-commit`: fmt + clippy + layout; `pre-push`: `just check`; `pre-tag`: semver-checks + audit + deny)
- **No AI attribution in commits** — do not add Claude/Co-Authored-By lines unless explicitly requested

## Project References

| Path | Description |
|------|-------------|
| `.factory/STATE.md` | Live pipeline state (current phase, decisions log, session resume checkpoint) |
| `.factory/SESSION-HANDOFF.md` | Resume-ready handoff for new sessions |
| `.factory/specs/architecture/` | Architecture docs + ADRs + ARCH-INDEX.md (subsystem registry) |
| `.factory/specs/behavioral-contracts/` | BC files + BC-INDEX.md |
| `.factory/specs/verification-properties/` | VP files + VP-INDEX.md (Kani proofs + fuzz targets) |
| `.factory/specs/domain-spec/` | L2 domain spec (entities, invariants, capabilities, edge cases) |
| `.factory/stories/` | Per-story implementation specs + STORY-INDEX.md |
| `.factory/research/` | Cited research artifacts (e.g., build-optimization-2026.md) |
| `.factory/policies.yaml` | Project governance policy registry (10 baseline + project-specific) |
| `docs/dev-setup.md` | Dev environment setup + build performance notes |
| `crates/` | 24-crate Rust workspace (parser, sensors, DTU clones, MCP, etc.) |
| `tests/external/perimeter-violation/` | Compile-fail test crate enforcing prism-query security perimeter |
| `fuzz/` | cargo-fuzz targets (vp021_parse_fuzz, etc.) |
| `Justfile` | Task runner — `just --list` for current recipes |
| `lefthook.yml` | Pre-commit/push/tag git hook config |
| `rust-toolchain.toml` | Pinned Rust toolchain channel + components + targets |
