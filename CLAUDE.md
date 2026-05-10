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

## Factory Hook Diagnostics

When `Agent` tool dispatches fail with errors like:

```
PreToolUse:Agent hook error: [...factory-dispatcher]: factory-dispatcher trace=<UUID> event=PreToolUse tool=Agent host_abi=1 matched_tiers=N plugins_run=N total_ms=N block_intent=true exit_code=2
```

— the factory-dispatcher hook chain (52 plugins, see `~/.claude/plugins/cache/claude-mp/vsdd-factory/1.0.0-rc.11/hooks-registry.toml`) blocked the dispatch. The error message itself carries NO human-readable reason — only the trace UUID. To diagnose, follow this procedure.

### Step 1 — Locate the dispatcher log

Internal logs live at:

```
.factory/logs/dispatcher-internal-YYYY-MM-DD.jsonl
```

(One file per day, JSONL format, one event per line.)

### Step 2 — Find the block reason

Search the day's log for the trace UUID:

```bash
grep '<TRACE-UUID>' .factory/logs/dispatcher-internal-$(date +%Y-%m-%d).jsonl
```

Look for `plugin.log` entries with `level: warn` — those carry the human-readable block reason as an embedded multi-line `message` field. Example payload from a real block:

```
"FAIL: MULTI_COMMIT_CHAIN_NOT_ALLOWED — HEAD and HEAD^ both contain 'backfill'.
 The single-commit protocol (TD-VSDD-053) does not use backfill commits.
 ...
 Recover with: git -C .factory reset --soft HEAD~2 then re-author as a single commit"
```

The `plugin_name` field on the same record (e.g., `validate-wave-gate-prerequisite`, `validate-pr-merge-prerequisites`, `regression-gate`) tells you which guard fired.

### Step 3 — Common blockers and recovery procedures

| Blocker | Detection | Recovery |
|---------|-----------|----------|
| **Multi-commit chain (TD-VSDD-053)** | HEAD and HEAD^ both have `backfill` / `Stage 1` / `Stage 2` in their commit messages | `git -C .factory reset --soft HEAD~N` (preserves working tree); re-author as one combined commit; force-push with `--force-with-lease` (requires explicit user approval) |
| **SHA drift** | STATE.md or SESSION-HANDOFF.md cite a develop SHA that doesn't match `git rev-parse origin/develop` | Update narrative via state-manager dispatch; STATE.md `develop_head` and SESSION-HANDOFF cited SHAs must match `c98a38b0` (or current `git -C . log -1 --format=%H develop`) |
| **In-progress narrative** | STATE.md decision log has an open phase without closure | Add closure row via state-manager; bump version |
| **factory-artifacts dirty** | `git -C .factory status --porcelain` is non-empty | Commit/discard pending changes via state-manager |

### Step 4 — Re-run the validator before re-dispatching

```bash
bash .factory/hooks/verify-sha-currency.sh
```

Expected: exit 0 with `PASS` lines and no `FAIL` lines. If it still fails, repeat Step 2 with the new dispatch's trace.

### Step 5 — Going-forward discipline (orchestrator)

To avoid the multi-commit-chain block:

- **Bundle backfills.** When state-manager performs multi-document backfills (e.g., adversary pass-N report + fix-pass-N closure report), stage all files THEN commit ONCE. Never two state-manager dispatches in a row both producing "backfill" commits.
- **Single-commit-per-burst.** Each logical burst (one adversary cascade step, one fix-pass cycle, one phase transition) → one commit in `.factory/`. Multiple consecutive commits with the same theme word (`backfill`, `Stage`) trigger the chain detector.
- **Soft-reset for recovery, never `--hard`.** The working tree state is what we want to preserve.
- **Force-push always needs user approval.** Per project git-safety protocol; orchestrator must request it from the human.

### Hook source locations (read-only reference)

- Dispatcher binary: `~/.claude/plugins/cache/claude-mp/vsdd-factory/<version>/hooks/dispatcher/bin/<platform>/factory-dispatcher`
- Hook registry config: `~/.claude/plugins/cache/claude-mp/vsdd-factory/<version>/hooks-registry.toml`
- Hook plugins (WASM): `~/.claude/plugins/cache/claude-mp/vsdd-factory/<version>/hook-plugins/*.wasm`
- Project-side validator scripts: `.factory/hooks/*.sh` (e.g., `verify-sha-currency.sh`)

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
