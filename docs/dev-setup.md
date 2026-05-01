# Prism Developer Setup

## Prerequisites

Install the Prism developer toolchain by running:

```bash
just setup
```

This script is idempotent and installs all required toolchain extensions (cargo-audit,
cargo-deny, cargo-semver-checks, cargo-llvm-cov, etc.).

---

## Pre-push gate (fast local check)

The pre-push gate (`just check`) runs in ~5-8 min on a typical workstation. It executes:

- `cargo fmt --check`
- `cargo clippy --all-features -- -D warnings`
- `cargo test --workspace --all-features` (with `PROPTEST_CASES=100`)
- `scripts/check-crate-layout.sh`

The `PROPTEST_CASES=100` setting overrides any value in your shell environment for the
duration of the `cargo test` invocation only. CI uses the default (1000 cases) for full
coverage.

CI runs the full-strength `just check-ci` which includes:

- All of the above with default 1000 proptest cases
- `cargo audit` (RustSec supply-chain advisories)
- `cargo deny check` (license + advisory + duplicate detection)
- `cargo semver-checks` (API compatibility against `origin/develop`)

### Standalone targets (run ad-hoc)

- `just audit` — run cargo-audit alone
- `just deny` — run cargo-deny alone
- `just semver-checks` — run cargo-semver-checks alone (use before tagging releases)

### Pre-tag hook (release prep)

The `pre-tag` lefthook hook (requires lefthook >= 1.6) runs `just semver-checks`,
`just audit`, and `just deny` automatically before every `git tag` invocation.

Verify your lefthook version with: `lefthook --version`

If your lefthook version is < 1.6 (pre-tag hook not supported), run these manually
before tagging a release:

```bash
just semver-checks
just audit
just deny
```

---

## Faster cross-worktree builds (optional)

Each `.worktrees/<story>/` has its own `target/` directory by default, which can
consume ~8GB or more per worktree. To share a single build cache across all worktrees:

```bash
# Add to ~/.zshrc or ~/.bashrc
export CARGO_TARGET_DIR=$HOME/.cargo-target-shared/prism
```

Caveats:

- **Lock contention:** Cargo serializes concurrent invocations via `target/.cargo-lock`.
  Two `cargo` processes from different worktrees sharing the same directory will wait for
  each other rather than run in parallel. There is no corruption risk — just serialization.
- **Disk space:** The shared cache grows with each unique build configuration. Requires
  at least 50GB free on the target volume. Not recommended on HDDs.
- **First build:** Cold-build savings are substantial (~10-15 min first build per config).
  Subsequent incremental builds benefit less.
- **EC-003:** Cargo creates the `$CARGO_TARGET_DIR` directory automatically on first use;
  no manual `mkdir` is required.
- **CI does NOT use this setting** — set only in your local shell config.
