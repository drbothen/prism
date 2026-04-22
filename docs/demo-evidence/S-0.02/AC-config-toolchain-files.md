# AC-config Evidence: static toolchain configuration files

Story: S-0.02 | Version: 1.4 | Date: 2026-04-21

Covers: rust-toolchain.toml, rustfmt.toml, clippy.toml, kani.toml

---

## rust-toolchain.toml

```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy", "rust-src"]
targets = [
  "aarch64-apple-darwin",
  "x86_64-apple-darwin",
  "x86_64-unknown-linux-gnu",
  "x86_64-unknown-linux-musl",
  "x86_64-pc-windows-msvc",
]
```

TOML parse: `{'toolchain': {'channel': 'stable', 'components': ['rustfmt', 'clippy', 'rust-src'], 'targets': [...]}}`  
Status: **OK**

## rustfmt.toml

```toml
edition = "2021"
max_width = 100
use_small_heuristics = "Default"
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
```

TOML parse: `{'edition': '2021', 'max_width': 100, 'use_small_heuristics': 'Default', 'imports_granularity': 'Crate', 'group_imports': 'StdExternalCrate'}`  
Status: **OK**

## clippy.toml

```toml
cognitive-complexity-threshold = 30
too-many-arguments-threshold = 8
```

TOML parse: `{'cognitive-complexity-threshold': 30, 'too-many-arguments-threshold': 8}`  
Status: **OK**

## kani.toml

```toml
[general]
default-unwind = 10

[verification]
timeout = 300
memory-limit = 8192
```

TOML parse: `{'general': {'default-unwind': 10}, 'verification': {'timeout': 300, 'memory-limit': 8192}}`  
Status: **OK**

## Architecture compliance checks

| Requirement | Value | Status |
|-------------|-------|--------|
| channel = "stable" (not nightly) | stable | PASS |
| rust-src component present (required for Kani) | yes | PASS |
| cognitive-complexity-threshold = 30 | 30 | PASS |
| too-many-arguments-threshold = 8 | 8 | PASS |
| kani timeout = 300 | 300 | PASS |
| kani memory-limit = 8192 | 8192 | PASS |
| kani default-unwind = 10 | 10 | PASS |

## Cargo.toml workspace lints

```toml
[workspace.lints.clippy]
await_holding_lock = "deny"
unwrap_used = "warn"
```

## Test gate result

```
ok 1 - rust-toolchain.toml exists
ok 2 - rust-toolchain.toml pins channel = "stable"
ok 3 - rust-toolchain.toml includes rust-src component (required for Kani)
ok 4 - rust-toolchain.toml does not pin nightly (architecture compliance)
ok 5 - rustfmt.toml exists
ok 6 - rustfmt.toml: group_imports correct
ok 7 - rustfmt.toml: imports_granularity correct
ok 8 - rustfmt.toml: edition correct
ok 9 - rustfmt.toml: use_small_heuristics correct
ok 10 - rustfmt.toml: max_width correct
ok 11 - clippy.toml exists
ok 12 - clippy.toml: cognitive-complexity-threshold = 30
ok 13 - clippy.toml: too-many-arguments-threshold = 8
ok 14 - kani.toml exists
ok 15 - kani.toml: default-unwind = 10
ok 16 - kani.toml: timeout = 300
ok 17 - kani.toml: memory-limit = 8192
ok 18 - Cargo.toml: await_holding_lock = "deny" in [workspace.lints.clippy]
ok 19 - Cargo.toml: unwrap_used = "warn" in [workspace.lints.clippy]
ok 20 - Justfile: integration-test target exists
ok 21 - Justfile: dtu-start target exists
ok 22 - Justfile: dtu-validate target exists
ok 23 - rustc --version succeeds (rust-toolchain.toml pin is valid)
```
**PASS (23/23)**
