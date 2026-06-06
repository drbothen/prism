# Red Gate Log — S-DEMO-003

**Story:** S-DEMO-003 — scripts: Demo Setup Scripts + `prism credential set` CLI + Operator Runbook
**Phase:** 3 (TDD Implementation) — Red Gate Step
**Wave:** wave-5-e-demo-fidelity
**Date:** 2026-06-06
**Author:** test-writer
**Worktree:** .worktrees/S-DEMO-003 (branch: feature/S-DEMO-003, based on develop@9447671f)

---

## Red Gate Status: RED

2 of 2 new tests FAIL. Workspace COMPILES. Red Gate discipline satisfied per BC-5.38.001.

---

## Stub Changes

| File | Change |
|------|--------|
| `crates/prism-bin/src/credential_cli.rs` | NEW — `CredentialCommand`, `CredentialSetArgs`, `CredentialArgs` clap types. `handle_credential_set()` stub (panics with `todo!(S-DEMO-003: ...)`). `generate_demo_prism_toml()` stub (panics with `todo!(S-DEMO-003: ...)`). AD-017: no `--value` field on `CredentialSetArgs`. |
| `crates/prism-bin/src/cli.rs` | Added `Credential(CredentialArgs)` variant to `PrismCommand` enum. Added `pub use crate::credential_cli::CredentialArgs`. |
| `crates/prism-bin/src/lib.rs` | Added `pub mod credential_cli` registration. |
| `crates/prism-bin/src/main.rs` | Added `use prism_bin::credential_cli::{CredentialCommand, handle_credential_set}`. Added dispatch arm `PrismCommand::Credential(credential_args) => match credential_args.command { ... }`. |
| `crates/prism-bin/Cargo.toml` | Added two `[[test]]` entries: `bc_2_06_001_demo_setup_toml` and `bc_2_03_007_credential_set_no_echo`. |

**`cargo check -p prism-bin` result: PASS (exit 0)** — all files compile without error.

---

## Testable Seam Decision (AC-001)

The Red Gate test for AC-001 (`test_BC_2_06_001_demo_setup_generates_valid_prism_toml`) uses
a **Rust helper function** as the testable seam, not the shell script `scripts/demo-setup.sh`.

**Rationale:**
- `demo-setup.sh` does not exist during the Red Gate phase. Running it would produce a
  "file not found" error, not a meaningful assertion about the TOML contract (BC-2.06.001).
- The Rust helper `generate_demo_prism_toml()` gives the implementer a clear, testable unit
  that `demo-setup.sh` can mirror or call when writing `~/.config/prism-demo/prism.toml`.
- The test exercises `toml::from_str::<PrismConfig>()` directly — the exact code path that
  boot step 2 (`step2_load_config`) uses at runtime.

The implementer must implement `generate_demo_prism_toml()` in `credential_cli.rs` to return
a valid TOML string. The `demo-setup.sh` script writes this same content to disk.

---

## Test Files Created

| File | BC | Test Name |
|------|----|-----------|
| `crates/prism-bin/tests/bc_2_06_001_demo_setup_toml.rs` | BC-2.06.001 | `test_BC_2_06_001_demo_setup_generates_valid_prism_toml` |
| `crates/prism-bin/tests/bc_2_03_007_credential_set_no_echo.rs` | BC-2.03.007 | `test_BC_2_03_007_prism_credential_set_does_not_echo_value_to_stdout` |

---

## Red Gate Test Results

```
────────────
 Summary [1.397s] 123 tests run: 121 passed, 2 failed, 13 skipped
    FAIL prism-bin::bc_2_06_001_demo_setup_toml test_BC_2_06_001_demo_setup_generates_valid_prism_toml
    FAIL prism-bin::bc_2_03_007_credential_set_no_echo test_BC_2_03_007_prism_credential_set_does_not_echo_value_to_stdout
```

### test_BC_2_06_001_demo_setup_generates_valid_prism_toml

**Failure reason:** `generate_demo_prism_toml()` panics with `todo!()` — the stub is unimplemented.

```
thread 'test_BC_2_06_001_demo_setup_generates_valid_prism_toml' panicked at crates/prism-bin/src/credential_cli.rs:200:5:
not yet implemented: S-DEMO-003: implement generate_demo_prism_toml — return valid TOML string
for demo prism.toml per BC-2.06.001. Must deserialize via toml::from_str::<PrismConfig>() without error.
```

**Correct failure:** panics in the stub before the assertion can run. Red Gate is correctly established.

### test_BC_2_03_007_prism_credential_set_does_not_echo_value_to_stdout

**Failure reason:** subprocess (`prism credential set`) panics at `handle_credential_set()` `todo!()`.
The panic hook exits with code 1. The test asserts exit code 0 (or exit 1 + "Keyring unavailable"),
which fails because the panic exit code 1 does not satisfy that condition.

```
thread 'test_BC_2_03_007_prism_credential_set_does_not_echo_value_to_stdout' panicked at
crates/prism-bin/tests/bc_2_03_007_credential_set_no_echo.rs:186:5:
prism credential set must exit 0 on success, or exit 1 with 'Keyring unavailable' on headless CI
(EC-001 of S-DEMO-003). Got exit code 1.
```

**Correct failure:** exit code 1 from panic hook, not from "Keyring unavailable". Red Gate correctly established.

**Secondary observation:** the BC-2.03.007 secret-not-on-stdout/stderr assertions would PASS even at Red Gate
(the sentinel `PRISM_DEMO_SECRET_SENTINEL_12345` does not appear in the panic message). The primary failure
is the exit code assertion. This is correct behavior — the exit code gate is the meaningful barrier.

---

## Rpassword Dependency Gap (NEEDS_ARCHITECT_CONFIRM)

`rpassword` is **not** a workspace or prism-bin dependency (verified 2026-06-06).

The story spec (S-DEMO-003 §Library & Framework Requirements) says: `rpassword = "7.*"`.
The open question (story Open Question 2): "Is `rpassword` already in `Cargo.toml` workspace deps?"

**Finding:** It is not. The implementer must add it before the green phase.

**Recommendation for implementer:** Add `rpassword = "7"` to `[dependencies]` in
`crates/prism-bin/Cargo.toml` (not workspace — it is prism-bin only, not a library dep).
Feature-gating is NOT required (the subcommand is in the binary, not a library crate surface).

**Architect routing:** this is a mechanical dependency pin decision, not requiring architect
adjudication per CLAUDE.md §6. The implementer should add it in-scope per the production-grade
default principle.

---

## Keyring Write API for Implementer

The implementer must call `CredentialStore::set()` from `prism-credentials`:

```rust
use prism_credentials::{KeyringBackend, CredentialIndex, trait_::CredentialStore};
use prism_core::{OrgSlug, CredentialName};
use secrecy::SecretString;

let index = CredentialIndex::load_or_create(index_path)?;
let store = KeyringBackend::new("prism", index);
store.set(
    &OrgSlug::new(org_slug),
    &sensor_id,
    &CredentialName::new_from_validated_storage(name),
    SecretString::new(value.into()),
).await?;
```

Namespace key produced: `"{org_slug}/{sensor_id}/{name}"` (BC-2.03.004 OrgSlug-keyed format).

Note: BC-2.03.004 specifies two key formats — OrgSlug-keyed (legacy, used by KeyringBackend::set)
and OrgId-keyed (new, used by KeyringBackend::set_by_org). The CLI subcommand should use the
OrgSlug-keyed path for consistency with how other boot paths resolve credentials, unless the
architect directs otherwise.

---

## Implementer Handoff Instructions

**Next step:** Implement `handle_credential_set()` and `generate_demo_prism_toml()` in
`crates/prism-bin/src/credential_cli.rs`, then make each Red Gate test pass one at a time.

**Micro-commit order:**
1. Implement `generate_demo_prism_toml()` → `test_BC_2_06_001_demo_setup_generates_valid_prism_toml` turns green.
2. Add `rpassword = "7"` to Cargo.toml dependencies.
3. Implement `handle_credential_set()` with stdin read + keyring write → `test_BC_2_03_007_...` turns green.
4. Write `scripts/demo-setup.sh`, `scripts/demo-run.sh`, `scripts/demo-teardown.sh`, `docs/DEMO-RUNBOOK.md`.
5. Run `just check` for final pre-push gate.

**Forbidden patterns (enforce at every step):**
- NEVER add `--value` flag to `CredentialSetArgs` (AD-017 violation).
- NEVER log or print the credential value (BC-2.03.007 postcondition).
- NEVER use `reqwest::Client::new()` without `.timeout(Duration::from_secs(30))` in new HTTP clients.
- NEVER add a `tracing::*!(event_type=...)` emission without a BC-2.16.002 catalog row.
