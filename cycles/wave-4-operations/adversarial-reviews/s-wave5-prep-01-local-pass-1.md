---
document_type: adversarial-review
pass: 1
cycle: wave-4-operations
story_id: S-WAVE5-PREP-01
branch: feature/S-WAVE5-PREP-01-prism-bin-chassis
head: c8900f56
verdict: BLOCKED-hard
findings_total: 12
findings_critical: 1
findings_high: 3
findings_medium: 5
findings_low: 3
observations: 3
kudos: 2
process_gaps: 1
reviewer: adversary
date: 2026-05-08
streak: "0/3"
inputs:
  - .factory/stories/S-WAVE5-PREP-01-prism-bin-chassis.md
  - .factory/specs/architecture/decisions/ADR-022-production-runtime-wiring.md
  - .factory/specs/behavioral-contracts/BC-2.06.011-config-load-on-startup.md
  - .factory/specs/behavioral-contracts/BC-2.21.001-org-registry-init.md
  - .factory/specs/behavioral-contracts/BC-2.03.013-credential-store-init.md
  - .factory/specs/behavioral-contracts/BC-2.05.012-audit-subsystem-init.md
  - .factory/specs/behavioral-contracts/BC-2.22.001-boot-orchestration.md
  - .factory/policies.yaml
input-hash: "afbee5b"
---

# Adversarial Review — Pass 1 (LOCAL convergence cascade)

**Story:** S-WAVE5-PREP-01 — prism-bin: Binary Chassis, CLI, and Boot Sequence
**Branch:** `feature/S-WAVE5-PREP-01-prism-bin-chassis` @ `c8900f56`
**Reviewer:** adversary (read-only)
**Date:** 2026-05-08
**Pass type:** LOCAL pass-1 (cycle: wave-4-operations)

## Verdict: BLOCKED-hard

Two anchor BC postconditions are demonstrably unmet by the production code, plus a critical test-injection gate is not actually gated and ships in the production binary. These are not nits — they are spec-conformance and security gaps. Convergence streak resets to 0/3.

## Finding count by severity

- **CRITICAL:** 1
- **HIGH:** 3
- **MEDIUM:** 5
- **LOW:** 3
- **OBS:** 3
- **KUDOs:** 2
- **`[process-gap]` tags:** 1

---

## CRITICAL Findings

### CRIT-1 — Test-injection env vars are NOT `#[cfg(test)]`-gated; they ship in production binary

**File:** `crates/prism-bin/src/boot.rs:161-163, 181-228, 244-274`
**Confidence:** HIGH
**Severity:** CRITICAL

Three test-injection environment variables are read in the production code path and trigger crashes / state-machine deviations:

- Line 161-163: `PRISM_TEST_INJECT_PANIC=true` causes `panic!()` unconditionally in any binary (debug or release).
- Line 181: `PRISM_TEST_INJECT_FAIL_STEP` triggers fake `CredentialPermissionDenied` / `CredentialRefInvalid` / `AuditInitFailed` returns based on the env var value.
- Line 244-274: `PRISM_TEST_STOP_AFTER_STEP=6` causes the production binary to halt at step 6 and install an inline signal handler.

Lines 155-158 do gate one copy of the panic-injection behind `#[cfg(test)]`, but the implementer added a duplicate at 161-163 that is unconditional, and the comment at line 159 acknowledges this: *"Also fires in integration tests run as separate process (binary compiled in test mode)."*

Integration tests run the binary built by `cargo test` (not in `cfg(test)` for the binary crate itself), so the implementer needed an unconditional path. The actual fix is to use a feature flag or build script — not to leak the gate into production. BC-2.22.001 §Test Strategy explicitly says: *"Failure injection uses environment variables ... gated behind `#[cfg(test)]` in the boot sequence."* This invariant is violated.

**Operational consequence:** A deployed prism-bin can be crashed by setting `PRISM_TEST_INJECT_PANIC=true` in its environment. Forced exit 5 / 4 / 2 is also achievable by anyone with environment-variable control, evading SOC 2 audit assumptions about boot integrity.

**Required fix:** Move all `PRISM_TEST_*` reads behind `#[cfg(prism_test_injection)]` (a custom cfg flag) or compile-time `#[cfg(debug_assertions)]` and ensure the cfg is set only by integration test invocations. Document the gate in the story.

---

## HIGH Findings

### HIGH-1 — BC-2.05.012 postconditions NOT satisfied: no RocksDB CF open, no durable sentinel write

**File:** `crates/prism-bin/src/boot.rs:547-573`
**BC:** BC-2.05.012 §Postconditions
**Confidence:** HIGH
**Severity:** HIGH

`step6_init_audit_lightweight` only:
1. Calls `std::fs::create_dir_all(&config.state_dir)` (line 549)
2. Emits a tracing log with the sentinel fields (line 562-570)

BC-2.05.012 postconditions require, verbatim:
- *"The `audit_buffer` RocksDB column family is opened and confirmed writable"* — NOT done.
- *"The sentinel audit event `boot.audit.initialized` is constructed and written to the `audit_buffer` CF BEFORE step 7 begins — this write is synchronous and confirmed durable (not queued asynchronously)"* — NOT done.

Implementation comment at line 540-541 acknowledges: *"For the MVP chassis story, we perform a lightweight audit init that emits the boot.audit.initialized sentinel via tracing without requiring a full RocksDB open (which is deferred to step 7 via S-3.02-FOLLOWUP-RUNTIME)."*

Story narrative line 75: *"Wire steps 1–6 fully (tracing, config, org-registry, sensor TOML, credentials, audit)."* "Fully" is the word. Story Tasks step 5.6: Call `AuditEmitter::new(storage.clone())` — NOT done. Open `audit_buffer` RocksDB CF — NOT done.

BC-2.05.012 is in the story's `behavioral_contracts:` and `anchor_bcs:` arrays. AC-8 traces to BC-2.05.012 postcondition. The BC's invariants section explicitly forbids the chassis path: *"Audit initialization is non-optional ... no `--skip-audit` flag or degraded-mode path."* The lightweight chassis IS a degraded-mode path.

**Required fix:** Either (a) actually implement step 6 with RocksDB CF open + durable write before merge, or (b) re-anchor the story so BC-2.05.012 is NOT claimed satisfied (move it to a follow-up like S-3.02-FOLLOWUP-RUNTIME), and update AC-8.

### HIGH-2 — BC-2.03.013 postconditions NOT satisfied: no credential-ref iteration over sensor specs

**File:** `crates/prism-bin/src/boot.rs:490-534`
**BC:** BC-2.03.013 §Postconditions (Happy path bullet 2)
**Confidence:** HIGH
**Severity:** HIGH

`step5_init_credential_store` constructs a `KeyringBackend` for the `Keyring` config branch but does NOT iterate sensor spec credential refs. BC-2.03.013 happy-path postcondition 2 requires: *"All credential references declared in sensor specs are validated as resolvable: For each credential ref: the reference target EXISTS and the process has READ permission."*

Implementation comment at line 497-498 acknowledges the deferral: *"Real credential ref validation (checking each sensor spec's refs) is deferred to S-1.06/S-1.07 story implementations; here we just verify the store opens."*

This deferral is not authorized by the story. Tasks step 5.5: *"Resolve all credential refs declared in sensor specs (verify access only; no inline values per AD-017)."*

BC-2.03.013 is anchored in the story. AC-7 traces to BC-2.03.013. The BC's failure case table (Error Cases) explicitly lists "Credential reference declared in sensor spec but not found in backend" → exit 2 — this code path does not exist in the implementation.

The chassis test passes vacuously because the `valid` fixture has empty `spec_dir = "/tmp/prism-test-specs"` (auto-created on line 456) with zero sensor specs, so there are no refs to validate.

**Required fix:** Implement ref iteration: load sensor specs from step 4's `ConfigManager`, walk each spec's credential refs, call `keyring::Entry::new(...)` (or equivalent) for existence-only probe, error-map to `CredentialRefInvalid` (exit 2) or `CredentialPermissionDenied` (exit 5).

### HIGH-3 — `EncryptedFile` credential backend ALWAYS returns `CredentialPermissionDenied` even for valid configs

**File:** `crates/prism-bin/src/boot.rs:509-532`
**Confidence:** HIGH
**Severity:** HIGH

The `EncryptedFile { path }` branch:
1. Validates the file exists (line 511-516).
2. Logs a `tracing::warn!` (line 521-525).
3. Unconditionally returns `BootError::CredentialPermissionDenied(...)` with the message *"EncryptedFile backend requires passphrase resolution (deferred to S-1.07-FOLLOWUP)"* (line 527-531).

This means: any user who configures their `prism.toml` with `[credential_backend]` `type = "encrypted_file"` cannot start prism — boot exits 5. There is no opt-in to defer; the path is always taken.

This is in conflict with:
- BC-2.03.013 happy-path postconditions: a valid encrypted-file config should construct the store successfully.
- ADR-022 §A: encrypted-file backend is a documented option in `prism.toml`.
- The story's "Wire steps 1-6 fully" mandate.

Mapping a valid config to permission-denied is also a misuse of exit code 5. BC-2.03.013 reserves exit 5 for *actual* permission denial (keyring locked, file chmod), not for "feature not implemented in this story". A more honest failure would be `BootError::ConfigInvalid("encrypted_file backend not supported in v0.1.0")` mapped to exit 2, with the message clearly indicating unsupported configuration.

**Required fix:** Either implement encrypted-file passphrase resolution per AD-017, or fail-closed at config schema validation (step 2) with `ConfigInvalid` so users see a deterministic config error before step 5.

---

## MEDIUM Findings

### MED-1 — `pub async fn step6_init_audit` is dead `todo!()` code in step-6 path; AC-11 ambiguity

**File:** `crates/prism-bin/src/boot.rs:586-596`
**Confidence:** HIGH
**Severity:** MEDIUM

Two distinct functions both named `step6_init_audit*`:
- `step6_init_audit_lightweight(&PrismConfig) -> Result<(), BootError>` — actually called.
- `pub async fn step6_init_audit(_storage: Arc<RocksDbBackend>) -> Result<Arc<AuditEmitterLayer<RocksDbBackend>>, BootError>` with `todo!()` body — never called from anywhere in the workspace.

This violates story AC-11: *"No `todo!()`, `unimplemented!()`, or `panic!('stub')` may remain in the steps 1–6 production code paths before merge."* The function is in `boot.rs`, named `step6_init_audit`, has `todo!()`, and is `pub`. POL-12 verification step 1 finds this hit.

**Required fix:** Either delete `step6_init_audit` entirely (it's not called anywhere), or relocate the `todo!()` signature to a comment / doc placeholder. Do not ship a `pub fn` named `step6_*` with `todo!()` while the story claims AC-11 satisfaction.

### MED-2 — AC-6 SIGTERM test does NOT exercise `signals::install_sigterm_handler`

**File:** `crates/prism-bin/tests/signal_handlers.rs:54-95`; `crates/prism-bin/src/boot.rs:251-273`; `crates/prism-bin/src/signals.rs:40-95`
**Confidence:** HIGH
**Severity:** MEDIUM

`test_BC_2_10_010_sigterm_causes_graceful_exit_zero` sets `PRISM_TEST_STOP_AFTER_STEP=6`, which triggers boot.rs lines 244-274 — an INLINE signal handler installed directly inside `boot_to_step_6`. This handler does NOT invoke `signals::install_sigterm_handler`. It duplicates a subset of its logic.

The actual `signals::install_sigterm_handler` (`signals.rs:40-95`) is reachable only from `step11_install_signal_handlers` (boot.rs:648-653), which is a `todo!()`. So `signals.rs` is never executed in production OR test.

This means BC-2.10.010 (anchor BC for the production SIGTERM path) has zero coverage. AC-6 traces to BC-2.10.010 but exercises a different (inline duplicate) implementation. The test is a coverage decoy.

**Required fix:** Wire `signals::install_sigterm_handler` from the test gate at boot.rs:244-274 (call into signals.rs instead of duplicating the select! arm), or expose a test-only `_run_sigterm_loop` API on `signals` and use it from the test gate.

### MED-3 — `step4_load_sensor_specs` auto-creates `spec_dir` if missing — masks invalid config

**File:** `crates/prism-bin/src/boot.rs:454-462`
**Confidence:** HIGH
**Severity:** MEDIUM

When `config.spec_dir` does not exist, the implementation creates it via `std::fs::create_dir_all(spec_dir)`. The intent (per inline comment line 453) is *"empty is valid for validate-config"*. But:

1. The implementer is silently making writes to user-supplied filesystem locations during `validate-config`. A user testing config with `spec_dir = "/some/wrong/path"` would discover the path was created when they didn't want it.
2. BC-2.06.011 §Invariants requires strict validation. An invalid `spec_dir` (typo, missing path) should surface as a config error, not be papered over.

**Required fix:** Replace `create_dir_all` with `Err(BootError::ConfigInvalid(format!("spec_dir does not exist: {}", spec_dir.display())))`. The `validate-config` subcommand should NOT have side effects on the filesystem.

### MED-4 — AC-4 stderr field-name requirement is not guaranteed

**File:** `crates/prism-bin/src/boot.rs:363-364`
**AC:** AC-4: *"stderr contains the line number and field name of the parse error"*
**Confidence:** MEDIUM
**Severity:** MEDIUM

The implementation formats parse errors as `"Failed to parse prism.toml: {e}"` where `e` is the `toml::de::Error`. The `toml` crate's error includes line/column context but the field name may or may not appear depending on the error variant. AC-4 promises both line number AND field name; only line number is reliable.

The integration test at `tests/bc_2_06_011_config_load.rs:131` only checks exit code 2 — does not validate stderr content. So AC-4 is unverified by tests AND under-implemented.

**Required fix:** Either (a) post-process the toml error to extract field name, or (b) weaken AC-4 to "line number context" in a follow-up edit to the story.

### MED-5 — Test fixtures use shared `/tmp/` paths — parallel test races

**File:** `crates/prism-bin/fixtures/config/*/prism.toml` (all fixtures)
**Confidence:** HIGH
**Severity:** MEDIUM

All fixtures use `spec_dir = "/tmp/prism-test-specs"` and `state_dir = "/tmp/prism-test-state"`. When `cargo nextest` (or any parallel test runner) runs the integration suite, multiple subprocesses race on the same filesystem paths.

Today the chassis is lightweight enough that this hasn't surfaced. Once step 6 actually opens RocksDB at `state_dir`, parallel tests will fail with LOCK contention non-deterministically.

**Required fix:** Switch fixtures to use `${TMPDIR}/prism-{test-name}-{pid}/` pattern via test setup helper, or use `tempfile::TempDir` per test and synthesize the prism.toml dynamically.

---

## LOW Findings

### LOW-1 — AC-5 has no integration test

**File:** `crates/prism-bin/tests/` (no test exists)
**AC:** AC-5: *"first structured log line emitted is `{"level":"INFO","message":"Prism vX.Y.Z",...}`"*
**Confidence:** HIGH
**Severity:** LOW

Grep'd test files for "Prism v" and AC-5 — no test. The implementation in `step1_init_tracing` (boot.rs:315) emits the log line, but no test asserts:
- It is the FIRST log line.
- It is in the documented JSON format (when `PRISM_LOG_FORMAT=json`).

**Required fix:** Add `test_AC_5_first_log_line_is_prism_version` that runs `prism start` with `PRISM_LOG_FORMAT=json`, captures stdout, parses first JSON line, asserts `message == "Prism v0.1.0"`.

### LOW-2 — AC-9 + duplicate-org error messages don't match BC-2.21.001 canonical format

**File:** `crates/prism-bin/src/boot.rs:425-428`
**BC:** BC-2.21.001 Error Cases table
**Confidence:** MEDIUM
**Severity:** LOW

BC-2.21.001 specifies error messages "Duplicate org_id: {uuid}" and "Duplicate org_slug: {slug}". The implementation wraps the inner `RegistrationError` (which displays as "slug 'X' is already bound to org Y; cannot rebind to Z") inside *"Duplicate org entry: {e}"*. Tests pass because they only require `combined.contains("duplicate") && combined.contains("org")`, but the canonical message in the BC is not produced.

**Required fix:** Branch on the `RegistrationError` variant (`SlugConflict` vs `IdConflict`) and produce the canonical messages.

### LOW-3 — UUID v7 format is not strictly validated; any UUID accepted

**File:** `crates/prism-bin/src/boot.rs:415-420`
**BC:** BC-2.21.001 EC-21-001-008 (`org_id` not a UUID v7 → exit 2 with "must be a UUID v7")
**Confidence:** HIGH
**Severity:** LOW

`uuid::Uuid::parse_str(&entry.org_id)` accepts UUIDs of any version. BC-2.21.001 EC-21-001-008 expects rejection of non-v7 UUIDs. Implementer doesn't check the version field.

**Required fix:** After parse, assert `org_uuid.get_version() == Some(uuid::Version::SortRand)` (UUID v7) and return `OrgRegistryFailed` if not.

---

## Observations

### OBS-1 — BC-2.05.012 OQ-2 (sentinel schema confirmation) not addressed

The BC's open question requires the implementer to confirm whether `AuditEntry` covers `prism_version` and `boot_step` fields, or whether a `BootSentinelEntry` struct is needed. Implementation does neither — just emits via `tracing::info!` with structured fields, bypassing the type system. TV-05-012-006 (sentinel schema test) cannot be machine-verified.

### OBS-2 — BC-2.03.013 OQ-1 (non-leak invariant test approach) not addressed

The BC asks the implementer to confirm Approach A or B for non-leak verification. The test in `tests/bc_2_03_013_credential_init.rs:164-191` only checks `BootError::CredentialPermissionDenied` *display string*; it does not inspect the `CredentialStore` struct's fields for secret values. Approach A (type-level) and Approach B (runtime field count) are both unimplemented.

### OBS-3 — BC-2.05.012 unwriteable state_dir test (TV-05-012-002 with `chmod 444`) not actually performed

`tests/bc_2_05_012_audit_init.rs:56-74` claims to test "TV-05-012-002: Unwriteable state_dir → exit 4" but uses `PRISM_TEST_INJECT_FAIL_STEP=6_audit_failure` which is a synthetic injection. The actual `chmod 444` filesystem behavior is never exercised. Cross-platform safety is the stated reason but the BC's authority is the canonical test vector.

---

## KUDOs

### KUDO-1 — Disciplined exit-code constants and `BootError::exit_code()` mapping

`exit_codes.rs` is exemplary: every constant has a clear ADR-022 §A citation, the inline tests document the canonical table by both value and distinctness, and `BootError::exit_code()` (boot.rs:81-90) cleanly fans into the constants. This is the kind of belt-and-suspenders that catches regressions when the exit-code contract is touched.

### KUDO-2 — Honest "Wire steps 7-11 as todo!() with story IDs" annotations

The step 7-11 stubs (boot.rs:602-653) follow the story's exact format: each `todo!("S-WAVE5-PREP-01 step N — ... — resolved by S-X-FOLLOWUP-Y")` makes the deferral graph traceable.

---

## Process Gaps

### `[process-gap]` PG-1 — POL-12 vs story AC-11 conflict for "production-reachable pub fn with todo!()"

POL-12 verification step 4 says: *"If `status: ready` is permitted as a transient Red Gate test."* Story AC-11 says: *"No `todo!()` ... may remain in the steps 1–6 production code paths before merge."* These are different scopes:
- POL-12 is about story status, not step.
- AC-11 is per-step.

When an implementer ships a `pub async fn step6_init_audit` with `todo!()` while `status: ready`, the policies disagree. The orchestrator should clarify which authority wins for step-1-6 stubs that are "Red Gate transient" but anchored to ACs that forbid them.

**Recommended codification:** Add a verification step to POL-12 referencing AC-11-class story-level invariants — when the story spec contains an AC that explicitly forbids `todo!()` in a numbered step, that AC overrides the transient-Red-Gate exemption.

---

## Novelty Assessment

This is pass 1 — all findings are novel. Highest-impact issues: CRIT-1 (production injection footgun), HIGH-1/2/3 (anchor-BC postconditions unmet, encrypted-file backend always denies). MED findings cluster around test coverage decoys (MED-2, MED-4) and config validation laxity (MED-3).

The story committed to "fully wire steps 1-6" but the implementation actually wires 1-4 fully and 5-6 partially with a lightweight chassis. This pattern (claiming "fully" while shipping "partial") is the single biggest correctness gap; it is invisible to test coverage because tests use empty fixtures that bypass the unimplemented validation paths.

---

## Recommended fix-pass scope

The orchestrator should dispatch a fix pass with the following priorities:

1. CRIT-1: Gate test-injection env vars behind a build-script `cfg` flag, not unconditional reads.
2. HIGH-1: Either implement step 6 RocksDB CF open + durable sentinel (requires storage backend wiring), OR re-anchor BC-2.05.012 off this story (move to S-3.02-FOLLOWUP-RUNTIME) and amend AC-8.
3. HIGH-2: Implement credential-ref iteration over sensor specs, OR re-anchor BC-2.03.013 off this story.
4. HIGH-3: Either implement encrypted-file passphrase resolution OR fail at step-2 schema validation with ConfigInvalid (exit 2), not step-5 PermissionDenied (exit 5).
5. MED-1: Delete dead `pub fn step6_init_audit` with `todo!()` body.
6. MED-2: Wire SIGTERM test through `signals::install_sigterm_handler`.
7. MED-3, MED-5: Stop auto-creating `spec_dir`; switch fixtures off shared /tmp paths.
8. LOW-1, LOW-3, OBS-1, OBS-2: Add missing tests / address open questions.

After fixes, dispatch pass-2 of LOCAL adversarial cascade. Convergence streak: **0/3**.
