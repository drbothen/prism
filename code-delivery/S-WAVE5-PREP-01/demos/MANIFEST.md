# S-WAVE5-PREP-01 Demo Manifest

Story: S-WAVE5-PREP-01 — prism-bin: Binary Chassis, CLI, and Boot Sequence  
Worktree HEAD: d5dc4210  
Date: 2026-05-09  
Binary: `target/debug/prism` (debug; built with `--features test-injection` for fault-injection ACs)  
VHS version: 0.10.0  
Font: FiraCode Nerd Font Mono  
Theme: Dracula  

## Per-AC Demos

| AC | Description | Evidence files | Exit code | Verified |
|----|-------------|----------------|-----------|----------|
| AC-1 | `prism --help` lists all 4 subcommands with exit-code docs | `AC-002-help.gif`, `AC-002-help.webm`, `AC-002-help.txt` | 0 | PASS |
| AC-2 | `prism version` / `prism --version` prints `prism X.Y.Z` and exits 0 | `AC-001-version.gif`, `AC-001-version.webm`, `AC-001-version.txt` | 0 | PASS |
| AC-3 | `validate-config` with valid fixtures exits 0, shows step-1-6 success log | `AC-003-validate-config-valid.gif`, `AC-003-validate-config-valid.webm`, `AC-003-validate-config-valid.txt` | 0 | PASS |
| AC-4 | Invalid TOML exits 2 with line number and field context in stderr | `AC-004-invalid-toml.gif`, `AC-004-invalid-toml.webm`, `AC-004-invalid-toml.txt` | 2 | PASS |
| AC-5 | First log line with `PRISM_LOG_FORMAT=json` is `{"level":"INFO","fields":{"message":"Prism v0.1.0"},...}` | `AC-005-json-log.gif`, `AC-005-json-log.webm`, `AC-005-json-log.txt` | 0 | PASS |
| AC-6 | SIGTERM during boot emits "Received SIGTERM — shutting down" and exits 0 | `AC-006-sigterm.gif`, `AC-006-sigterm.webm`, `AC-006-sigterm.txt` | 0 | PASS |
| AC-7 | Credential PermissionDenied (injected) exits 5 (not 1 or 4) | `AC-007-cred-permission-denied.gif`, `AC-007-cred-permission-denied.webm`, `AC-007-cred-permission-denied.txt` | 5 | PASS |
| AC-8 | Audit init failure (injected) exits 4 | `AC-008-audit-init-fail.gif`, `AC-008-audit-init-fail.webm`, `AC-008-audit-init-fail.txt` | 4 | PASS |
| AC-9 | Zero orgs in config exits 2 with "Config must declare at least one org" | `AC-009-empty-orgs.gif`, `AC-009-empty-orgs.webm`, `AC-009-empty-orgs.txt` | 2 | PASS |
| AC-10 | MCP traffic gate blocks traffic until step 8 completes | `AC-010-DEFERRED.md` | N/A | DEFERRED to S-3.02-FOLLOWUP-RUNTIME |
| AC-11 | POL-12: no `todo!()`/`unimplemented!()` in steps 1-6 production code paths | `AC-011-no-stubs-steps1-6.gif`, `AC-011-no-stubs-steps1-6.webm`, `AC-011-no-stubs-steps1-6.txt` | N/A | PASS |
| AC-12 | Injected panic fires tracing::error! hook and exits 1 | `AC-012-panic-hook.gif`, `AC-012-panic-hook.webm`, `AC-012-panic-hook.txt` | 1 | PASS |

## Coverage Summary

- **11 of 12 ACs recorded** (AC-10 deferred per story — requires S-3.02-FOLLOWUP-RUNTIME)
- **11 VHS GIF recordings** produced
- **11 VHS webm recordings** produced
- **11 plain text captures** produced (reproducible, no timestamps)
- **1 deferral markdown** (`AC-010-DEFERRED.md`) explaining why AC-10 has no live demo

## Fault Injection Notes (AC-7, AC-8, AC-12)

ACs 7, 8, and 12 use the `test-injection` feature flag (compiled into debug binary).
Injection is triggered by env vars that are zero-cost no-ops in release builds
(`#[cfg(feature = "test-injection")]` gates all PRISM_TEST_* reads).

| AC | Env var | Injected failure |
|----|---------|-----------------|
| AC-7 | `PRISM_TEST_INJECT_FAIL_STEP=5_permission` | `CredentialPermissionDenied` → exit 5 |
| AC-8 | `PRISM_TEST_INJECT_FAIL_STEP=6_audit_failure` | `AuditInitFailed` → exit 4 |
| AC-12 | `PRISM_TEST_INJECT_PANIC=true` | raw `panic!()` in boot step → custom hook → exit 1 |

## SIGTERM Test Mechanism (AC-6)

Uses `PRISM_TEST_STOP_AFTER_STEP=6` + `PRISM_TEST_READY_FILE=/tmp/sentinel` to hold the
process at step-6 state after writing the sentinel file. The test sends SIGTERM after
detecting the sentinel, verifying the `signals::create_sigterm_future` handler fires
and the process exits cleanly (code 0).

## Files in This Directory

```
AC-001-version.gif            AC-001-version.webm           AC-001-version.txt
AC-001-version.tape
AC-002-help.gif               AC-002-help.webm              AC-002-help.txt
AC-002-help.tape
AC-003-validate-config-valid.gif  AC-003-validate-config-valid.webm  AC-003-validate-config-valid.txt
AC-003-validate-config-valid.tape
AC-004-invalid-toml.gif       AC-004-invalid-toml.webm      AC-004-invalid-toml.txt
AC-004-invalid-toml.tape
AC-005-json-log.gif           AC-005-json-log.webm          AC-005-json-log.txt
AC-005-json-log.tape
AC-006-sigterm.gif            AC-006-sigterm.webm           AC-006-sigterm.txt
AC-006-sigterm.tape
AC-007-cred-permission-denied.gif  AC-007-cred-permission-denied.webm  AC-007-cred-permission-denied.txt
AC-007-cred-permission-denied.tape
AC-008-audit-init-fail.gif    AC-008-audit-init-fail.webm   AC-008-audit-init-fail.txt
AC-008-audit-init-fail.tape
AC-009-empty-orgs.gif         AC-009-empty-orgs.webm        AC-009-empty-orgs.txt
AC-009-empty-orgs.tape
AC-010-DEFERRED.md
AC-011-no-stubs-steps1-6.gif  AC-011-no-stubs-steps1-6.webm AC-011-no-stubs-steps1-6.txt
AC-011-no-stubs-steps1-6.tape
AC-012-panic-hook.gif         AC-012-panic-hook.webm        AC-012-panic-hook.txt
AC-012-panic-hook.tape
MANIFEST.md
```
