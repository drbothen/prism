# toolchain-gate — S-0.02 Validation Tests

Red Gate step 2 tests for story S-0.02 Developer Toolchain Bootstrap.

## Purpose

These tests assert acceptance criteria (AC-1 through AC-6) and configuration
semantic requirements from the story. Every test MUST FAIL against the stub
scaffold and MUST PASS after full implementation.

## How to Run

```bash
# From the worktree root:
bash tests/toolchain-gate/run.sh

# Or run a single test:
bash tests/toolchain-gate/test_AC-1_just-check-runs-pr-gate.sh
```

## Test Files

| File | AC | Assertion |
|------|----|-----------|
| `test_AC-1_just-check-runs-pr-gate.sh` | AC-1 | `just check` exists, fails on stub, Justfile contains all 6 PR gate commands |
| `test_AC-2_lefthook-precommit-hooks.sh` | AC-2 | lefthook.yml has fmt+clippy commands, stage_fixed, glob *.rs |
| `test_AC-3_dev-setup-installs-tools.sh` | AC-3 | dev-setup.sh exists, executable, references all 9 tools, runs lefthook install |
| `test_AC-4_dev-setup-idempotent.sh` | AC-4 | dev-setup.sh contains existence checks; stub fails twice (Red Gate) |
| `test_AC-5_deny-toml-license-check.sh` | AC-5 | deny.toml has correct license allowlist, vulnerability=deny, wildcards=deny |
| `test_AC-6_semgrep-credential-rule.sh` | AC-6 | Semgrep rule prism-no-string-credentials declared, stub pattern does not fire |
| `test_AC-config_toolchain-files.sh` | Tasks 1-4, Arch Compliance | rust-toolchain.toml, rustfmt.toml, clippy.toml, kani.toml exact values; Cargo.toml lints |

## Tool Dependencies

The following tools are required for full test coverage. Tests SKIP gracefully
(print `ok - SKIP <reason>`) when a tool is absent — they never print `ok` for
unverified behavior.

| Tool | Used by | Install via |
|------|---------|-------------|
| `just` | AC-1, AC-config | `scripts/dev-setup.sh` (after implementation) |
| `lefthook` | AC-2 | `scripts/dev-setup.sh` |
| `cargo-deny` | AC-5 | `scripts/dev-setup.sh` |
| `cargo-audit` | AC-1 (via just check) | `scripts/dev-setup.sh` |
| `cargo-semver-checks` | AC-1 (via just check) | `scripts/dev-setup.sh` |
| `semgrep` | AC-6 | `pip install semgrep` or `brew install semgrep` |
| `shellcheck` | (not required, bash -n used instead) | `brew install shellcheck` |
| `rustc` / `cargo` | AC-config | `rustup` |

## Red Gate Verdict

Expected state before implementation:
- All test files exit non-zero.
- `run.sh` exits 1.
- AC-3 fails because `scripts/dev-setup.sh` exits 1.
- AC-2 fails because lefthook.yml has no fmt/clippy commands.
- AC-1 fails because `just check` exits 1.
- AC-6 fails because semgrep stub patterns do not fire on real Rust code.
