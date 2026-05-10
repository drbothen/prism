---
document_type: adversarial-review
pass: 2
cycle: wave-4-operations
story_id: S-WAVE5-PREP-01
branch: feature/S-WAVE5-PREP-01-prism-bin-chassis
head: 21d653b2
verdict: BLOCKED-hard
findings_total: 11
findings_critical: 1
findings_high: 3
findings_medium: 3
findings_low: 1
observations: 3
kudos: 3
process_gaps: 2
reviewer: adversary
date: 2026-05-09
streak: "0/3"
prior_pass_closures_verified: 9
prior_pass_closures_incomplete: 4
inputs:
  - .factory/stories/S-WAVE5-PREP-01-prism-bin-chassis.md
  - .factory/specs/architecture/decisions/ADR-022-production-runtime-wiring.md
  - .factory/specs/behavioral-contracts/BC-2.06.011-config-load-on-startup.md
  - .factory/specs/behavioral-contracts/BC-2.21.001-org-registry-init.md
  - .factory/specs/behavioral-contracts/BC-2.03.013-credential-store-init.md
  - .factory/specs/behavioral-contracts/BC-2.05.012-audit-subsystem-init.md
  - .factory/specs/behavioral-contracts/BC-2.22.001-boot-orchestration.md
  - .factory/policies.yaml
  - .factory/cycles/wave-4-operations/adversarial-reviews/s-wave5-prep-01-local-pass-1.md
input-hash: "fb409a7"
---

# Adversarial Review — Pass 2 (LOCAL convergence cascade)

**Story:** S-WAVE5-PREP-01 — prism-bin: Binary Chassis, CLI, and Boot Sequence
**Branch:** `feature/S-WAVE5-PREP-01-prism-bin-chassis` @ `21d653b2`
**Reviewer:** adversary (read-only)
**Date:** 2026-05-09
**Pass type:** LOCAL pass-2 (cycle: wave-4-operations)

## Verdict: BLOCKED-hard

Pass-1 fix-pass closed 9 of 13 prior findings but left 4 INCOMPLETE or REGRESSED. One new CRITICAL defect was introduced by the fix itself (CRIT-1 closure regressed via feature-gate without required-features enforcement), plus two new HIGH findings and additional medium/low findings. Convergence streak resets to 0/3.

## Finding count by severity

- **CRITICAL:** 1
- **HIGH:** 3
- **MEDIUM:** 3
- **LOW:** 1
- **OBS:** 3
- **KUDOs:** 3
- **`[process-gap]` tags:** 2

---

## Pass-1 Closure Verification

| Pass-1 ID | Status | Notes |
|-----------|--------|-------|
| CRIT-1 (test-injection env vars in prod binary) | INCOMPLETE/REGRESSED | Feature-gate added (`#[cfg(feature = "prism_test_injection")]`) but `Cargo.toml` is missing `required-features` enforcement; any crate that depends on prism-bin can enable the feature without audit trail. The env var read is still reachable in a release binary if the feature flag is set by a downstream. |
| HIGH-1 (BC-2.05.012 postconditions unmet — no RocksDB CF, no durable sentinel) | INCOMPLETE | Implementer chose Option A (fully implement step 6), but the implementation uses `prism_storage::audit_buffer` directly rather than constructing `prism_audit::AuditEmitter`. BC-2.05.012 postcondition requires `AuditEmitter` construction (story Tasks step 5.6 verbatim). Additionally, the sentinel write is not fsync'd (see F-PASS2-HIGH-1). |
| HIGH-2 (BC-2.03.013 postconditions unmet — no cred-ref iteration) | INCOMPLETE | The cred-ref iteration loop body is empty because `SensorSpec` v0.1.0 in `prism-core` has no `credential_refs` field. The loop iterates over nothing and the happy-path test passes vacuously (see F-PASS2-HIGH-3). |
| HIGH-3 (EncryptedFile backend always returns CredentialPermissionDenied) | CLOSED | Now fails at step-2 schema validation with `ConfigInvalid` (exit 2), not step-5 PermissionDenied. Exit-code semantics corrected. |
| MED-1 (dead `pub fn step6_init_audit` with `todo!()`) | CLOSED | Dead function deleted. |
| MED-2 (SIGTERM test exercises inline duplicate, not `signals::install_sigterm_handler`) | CLOSED | Test gate now calls into `signals::install_sigterm_handler`. |
| MED-3 (`step4` auto-creates `spec_dir`) | CLOSED | Auto-create replaced with `ConfigInvalid` error. |
| MED-4 (AC-4 stderr field-name not guaranteed) | INCOMPLETE | Test at `tests/bc_2_06_011_config_load.rs:131` still only asserts exit code 2; no stderr content assertion exists. The BC-2.06.011 AC-4 postcondition remains unverified by tests. |
| MED-5 (shared /tmp fixtures race under nextest) | CLOSED | Fixtures switched to `tempfile::TempDir` per-test. |
| LOW-1 (AC-5 first-log-line test missing) | CLOSED | Test added. |
| LOW-2 (duplicate-org error messages don't match BC-2.21.001 canonical format) | CLOSED | Error messages now produce canonical "Duplicate org_id: {uuid}" / "Duplicate org_slug: {slug}" format. |
| LOW-3 (UUID v7 not validated) | CLOSED | `uuid.get_version() == Some(uuid::Version::SortRand)` check added; produces UUID v7 error on mismatch. |
| OBS-1/OBS-2/OBS-3 | CLOSED (2/3) | OBS-2 (BC-2.03.013 OQ-1 non-leak approach) now has a type-level answer; OBS-3 (unwriteable state_dir test) now runs actual chmod. OBS-1 (BC-2.05.012 OQ-2 sentinel schema) remains open — see new F-PASS2-HIGH-2. |

**Summary:** 9 CLOSED, 4 INCOMPLETE/REGRESSED (CRIT-1, HIGH-1 regressed, HIGH-2 incomplete, MED-4 incomplete).

---

## CRITICAL Findings

### F-PASS2-CRIT-1 — BC-2.05.012 postcondition unmet: implementer bypassed `prism_audit::AuditEmitter` entirely; used `prism_storage::audit_buffer` directly

**File:** `crates/prism-bin/src/boot.rs` (step6 implementation, circa lines 547-600)
**BC:** BC-2.05.012 §Postconditions; Story Tasks step 5.6
**Confidence:** HIGH
**Severity:** CRITICAL

Story Tasks step 5.6 states verbatim: *"Call `AuditEmitter::new(storage.clone())` — Open `audit_buffer` RocksDB CF."* The implementer's fix-pass closed HIGH-1 by implementing the RocksDB write path, but wrote directly to `prism_storage::audit_buffer::AuditBuffer` (the raw storage type) rather than constructing `prism_audit::AuditEmitter` from the `prism-audit` crate.

BC-2.05.012 postcondition 1 requires: *"The `audit_buffer` RocksDB column family is opened and confirmed writable via `AuditEmitter`."* (emphasis: "via AuditEmitter"). The `prism_audit` crate contains the canonical audit type with validation, schema enforcement, and the sentinel construction logic. Bypassing it means:

1. The sentinel event is constructed ad hoc without `AuditEmitter`'s schema validation.
2. BC-2.05.012 postcondition 1 is still not satisfied by the letter of the BC.
3. AC-8 traces to BC-2.05.012 postcondition — AC-8 is still technically unmet.
4. The cross-crate dependency (`prism-audit`) is not exercised, so any future change to `AuditEmitter::new()` will silently diverge from the boot path.

**Operational consequence:** Boot step 6 does not actually test the `prism-audit` crate's initialization contract. The sentinel can succeed even if `AuditEmitter::new()` would fail due to a schema mismatch or missing CF configuration — the failure mode is invisible.

**Required fix:** Construct `prism_audit::AuditEmitter::new(storage.clone())` from the `prism-audit` crate (cross-crate call). The `prism-bin` `Cargo.toml` must add `prism-audit` as a dependency. The durable sentinel write must flow through `AuditEmitter::emit()`, not through direct `prism_storage::audit_buffer` manipulation. User has granted cross-crate authorization for this fix (prism-audit, prism-core, prism-storage may be touched).

---

## HIGH Findings

### F-PASS2-HIGH-1 — Sentinel write not fsync'd; BC-2.05.012 invariant violated; SOC 2 compliance defect

**File:** `crates/prism-bin/src/boot.rs` (sentinel write path in step6)
**BC:** BC-2.05.012 §Invariants
**Confidence:** HIGH
**Severity:** HIGH

BC-2.05.012 Postcondition 2 requires: *"The sentinel audit event `boot.audit.initialized` is... written to the `audit_buffer` CF BEFORE step 7 begins — this write is synchronous and confirmed durable (not queued asynchronously)."* The phrase "confirmed durable" is a requirement for `fsync` (or equivalent RocksDB `sync_option`) after the write.

The current implementation calls `db.put_cf(...)` but does not call `db.flush_wal()` or `db.flush_cf()` afterward. On Linux with default RocksDB options, a `put_cf` write goes to the WAL in-process buffer. A crash between the `put_cf` and a WAL flush would result in a lost sentinel — the system could boot step 7 onward without a durable audit record. This violates both BC-2.05.012 and SOC 2 CC7.1 (audit trail completeness).

**Required fix:** After the sentinel `put_cf`, call `db.flush_wal(true)` (synchronous WAL flush) or open the RocksDB column family with `WriteOptions::set_sync(true)`. Document the sync call with the BC-2.05.012 postcondition citation.

---

### F-PASS2-HIGH-2 — Sentinel payload missing required RFC3339 timestamp field per BC-2.05.012 lines 111-120

**File:** `crates/prism-bin/src/boot.rs` (sentinel construction, step6)
**BC:** BC-2.05.012 §Postconditions — sentinel schema
**Confidence:** HIGH
**Severity:** HIGH

BC-2.05.012 §Postconditions (lines 111-120) specifies the sentinel event schema. The sentinel payload must include at minimum:
- `event_type: "boot.audit.initialized"`
- `timestamp: <RFC3339>`
- `prism_version: <semver string>`
- `boot_step: 6`

The current implementation constructs the sentinel as a raw key-value where the value is a JSON blob. Inspection reveals the JSON blob omits the `timestamp` field entirely — the implementer serialized `event_type`, `prism_version`, and `boot_step`, but not `timestamp`. This leaves OBS-1 from pass-1 (BC-2.05.012 OQ-2 sentinel schema) still unresolved: the sentinel is now durable but non-conformant.

The TV-05-012-006 sentinel schema test (if it exists) cannot pass against this payload without a `timestamp` field.

**Required fix:** Add `timestamp: chrono::Utc::now().to_rfc3339()` to the sentinel JSON serialization. If using `AuditEmitter` (per F-PASS2-CRIT-1), the emitter should enforce the schema automatically — this finding is subsumed by CRIT-1's fix if done correctly.

---

### F-PASS2-HIGH-3 — Vacuous credential-ref happy path: `SensorSpec` in `prism-core` has no `credential_refs` field

**File:** `crates/prism-core/src/sensor_spec.rs` (SensorSpec struct); `crates/prism-bin/src/boot.rs` (step5 cred-ref iteration)
**BC:** BC-2.03.013 §Postconditions (Happy path bullet 2)
**Confidence:** HIGH
**Severity:** HIGH

The pass-1 fix for HIGH-2 added a loop in `step5_init_credential_store` that iterates `sensor_specs` and calls `keyring::Entry::new(...)` for each `credential_ref` in each spec. This loop is structurally present but functionally vacuous: `prism_core::SensorSpec` (version 0.1.0) does not have a `credential_refs` field. The loop body iterates over an empty iterator in all cases.

Evidence: `crates/prism-core/src/sensor_spec.rs` SensorSpec struct contains `name: String`, `sensor_type: String`, `version: String`, and `config: HashMap<String, Value>` — no `credential_refs: Vec<CredentialRef>` field. The story Tasks step 5.5 says *"Resolve all credential refs declared in sensor specs."* The loop exists but cannot reach any refs because the data model does not carry them.

The happy-path test in `tests/bc_2_03_013_credential_init.rs` still passes vacuously because the fixture spec directory (even when populated) produces `SensorSpec` instances with no credential refs to validate.

**Required fix:** Extend `prism_core::SensorSpec` with a `credential_refs: Vec<CredentialRef>` field (or `Option<Vec<CredentialRef>>`), and update the fixture(s) to include at least N>0 credential refs so the loop body is actually exercised. This is a cross-crate change (prism-core). User has granted cross-crate authorization.

---

## MEDIUM Findings

### F-PASS2-MED-1 — CRIT-1 fix introduced `just iter` ergonomics regression: `prism_test_injection` feature requires explicit feature flag in every `cargo test` invocation

**File:** `crates/prism-bin/Cargo.toml` (feature definition); `CLAUDE.md` (developer docs)
**Confidence:** MEDIUM
**Severity:** MEDIUM

The `#[cfg(feature = "prism_test_injection")]` fix gates the injection env vars behind a Cargo feature. However, the existing integration tests in `tests/boot_sequence.rs`, `tests/signal_handlers.rs`, and `tests/bc_2_05_012_audit_init.rs` all rely on the injection mechanism. Running `just iter prism-bin` without `--features prism_test_injection` causes these tests to fail silently (the env var reads are gone, injection never fires, tests that expect synthetic failures pass or hang).

This is a DX regression: `just iter prism-bin` was previously sufficient for the TDD inner loop. The developer now needs `just iter prism-bin --features prism_test_injection` to get a useful test run. This is not documented in `CLAUDE.md` or the story spec.

**Required fix:** Either (a) add `prism_test_injection` to the `[[test]]` section's `required-features` list in `Cargo.toml` (so `cargo test` auto-enables it for integration tests), or (b) add the feature to the `just iter` recipe's default arguments for `prism-bin`, and document the requirement in `CLAUDE.md`. Option (a) is preferred — it is self-documenting in the manifest.

---

### F-PASS2-MED-2 — Sibling-update gap from MED-3 closure: `validate-config` subcommand help text still mentions "will create missing directories"

**File:** `crates/prism-bin/src/cli.rs` (validate-config subcommand help string)
**Confidence:** MEDIUM
**Severity:** MEDIUM

MED-3 closure removed the `create_dir_all` call and replaced it with a `ConfigInvalid` error. However, the `validate-config` subcommand's help text (registered via `clap` attribute on the `ValidateConfig` variant) still reads: *"Validate configuration file (creates missing directories if needed)."* This is now false — the command no longer creates missing directories. A user reading the help text will have incorrect expectations and may file a bug when `validate-config` fails on a missing `spec_dir`.

**Required fix:** Update the `clap` help string to: *"Validate configuration file. Exits 2 if any required directory is missing."*

---

### F-PASS2-MED-3 — UUID v7 ordering ambiguity from LOW-3 closure: `get_version()` returns `Some(Version::SortRand)` for any timestamp-based UUID, not strictly v7

**File:** `crates/prism-bin/src/boot.rs` (UUID v7 validation in step3)
**BC:** BC-2.21.001 EC-21-001-008
**Confidence:** MEDIUM
**Severity:** MEDIUM

The LOW-3 fix added `uuid.get_version() == Some(uuid::Version::SortRand)`. In the `uuid` crate, `Version::SortRand` corresponds to UUID version 7. This is semantically correct. However, the error message produced on validation failure is `"org_id must be a UUID"` (generic) rather than `"org_id must be a UUID v7"` as BC-2.21.001 EC-21-001-008 specifies verbatim.

The integration test at `tests/bc_2_21_001_org_registry.rs` checks `combined.contains("v7")` to verify BC-2.21.001 EC-21-001-008, but the current error string does not contain "v7". The test either (a) is not asserting the expected string, or (b) is passing because a different code path produces the "v7" string. Without the test asserting the canonical BC error message, the BC postcondition is unverifiable by automated test.

**Required fix:** Change the error message to `format!("org_id '{}' must be a UUID v7", &entry.org_id)` to match the BC-2.21.001 EC-21-001-008 canonical format exactly. Update the test to assert `combined.contains("must be a UUID v7")`.

---

## LOW Findings

### F-PASS2-LOW-1 — AC-4 stderr field-name assertion still missing (MED-4 carry-forward)

**File:** `crates/prism-bin/tests/bc_2_06_011_config_load.rs`
**AC:** AC-4 (stderr contains line number AND field name of parse error)
**Confidence:** HIGH
**Severity:** LOW

MED-4 from pass-1 was marked INCOMPLETE in the closure verification table above. The test still only asserts exit code 2 without checking stderr content. Downgrading to LOW because the implementation-side fix (parsing the toml error for field name) is non-trivial and may warrant a story-level AC amendment — but the test gap itself is LOW-risk since the exit code is correctly exercised.

**Required fix:** Add a stderr content assertion: `assert!(stderr.contains("field") || stderr.contains("key"), "AC-4: stderr must identify the erroneous field")`. Alternatively, amend AC-4 in the story spec to "stderr contains line number context" and document the limitation.

---

## Observations

### F-PASS2-OBS-1 — `AuditEmitter` cross-crate dependency not declared in `prism-bin/Cargo.toml`

The story spec (Tasks step 5.6) calls for `AuditEmitter::new(storage.clone())`, which lives in `prism-audit`. The current `prism-bin/Cargo.toml` does not list `prism-audit` as a dependency. Any attempt to implement CRIT-1's fix will fail to compile without this dependency addition. Flagging as OBS (not a new finding — the dependency omission is consequential of CRIT-1) to ensure the fix pass adds the dep before attempting to compile.

### F-PASS2-OBS-2 — Sentinel CF name hardcoded as string literal `"audit_buffer"` rather than using the constant from `prism-storage`

**File:** `crates/prism-bin/src/boot.rs` (step6 CF open)
The sentinel write opens the `"audit_buffer"` column family by name string. The `prism-storage` crate exports a `AUDIT_BUFFER_CF_NAME` constant for exactly this purpose (avoiding typo divergence). The hardcoded string creates a maintenance hazard if the CF name is ever refactored in `prism-storage`. Not blocking, but should be addressed in the same fix pass.

### F-PASS2-OBS-3 — Step-11 `todo!("S-WAVE5-PREP-01 step 11 — signal handlers — resolved by S-5.01-FOLLOWUP-MCP-BOOT")` references a story that does not exist in STORY-INDEX v2.30

**File:** `crates/prism-bin/src/boot.rs` (step11 stub)
`S-5.01-FOLLOWUP-MCP-BOOT` does not appear in STORY-INDEX v2.30. The follow-up story referenced in the todo! annotation is either under a different ID or has not been authored yet. This creates a dangling story reference. Not blocking (step 11 is legitimately deferred), but the story reference should be corrected to an existing story ID before merge.

---

## KUDOs

### KUDO-1 — Clean EncryptedFile deferral at step-2 schema validation (HIGH-3 closure)

The HIGH-3 fix correctly moved the failure from step-5 `CredentialPermissionDenied` (exit 5) to step-2 `ConfigInvalid` (exit 2), with a clear error message *"encrypted_file backend not supported in v0.1.0; use keyring backend."* This is architecturally sound: config validation is the right place to reject unsupported config options, and the exit-code semantics are now correct.

### KUDO-2 — `signals::install_sigterm_handler` now properly exercised by test (MED-2 closure)

The MED-2 fix wired the `PRISM_TEST_STOP_AFTER_STEP=6` gate through `signals::install_sigterm_handler` rather than the inline duplicate. The test now exercises the production signal path. This eliminates the coverage decoy found in pass-1.

### KUDO-3 — `tempfile::TempDir` per-test isolation (MED-5 closure)

The MED-5 fix switched all fixtures from shared `/tmp/prism-test-specs` / `/tmp/prism-test-state` to per-test `tempfile::TempDir` instances. This is textbook nextest parallel-safe fixture management and prevents the race condition that would have emerged once step-6 opens RocksDB at `state_dir`.

---

## Process Gaps

### `[process-gap]` PG-1 (carried forward) — POL-12 vs story AC-11 conflict for production-reachable `pub fn` with `todo!()`

Carried forward from pass-1 without change. The orchestrator has not yet codified the resolution. Recommended: add a verification step to POL-12 that defers to AC-11 when the story spec contains an explicit `todo!()` prohibition for numbered steps.

### `[process-gap]` PG-2 — Test-injection feature pairing not codified in `policies.yaml`

The `prism_test_injection` Cargo feature pattern introduced by the CRIT-1 fix is novel to this workspace. No existing policy governs: (a) which crates may define `*_test_injection` features, (b) that such features must use `required-features` in `[[test]]` sections, or (c) that the feature name must not appear in release builds (ensured via `not(feature = "prism_test_injection")` guard in release-mode assertions).

Without a codified policy, the next implementer may introduce a similar feature without the `required-features` guard (reproducing F-PASS2-MED-1 in a different crate), or may enable the feature in a release binary inadvertently.

**Recommended codification:** Add a policy entry to `policies.yaml`:
```yaml
- id: POL-NEW
  name: test-injection-feature-pairing
  rule: >
    Cargo features named `*_test_injection` MUST: (1) be listed as required-features
    in all [[test]] sections that depend on them; (2) be guarded by
    #[cfg(not(feature = "test_injection"))] in any release-only invariant assertion;
    (3) never appear in the default-features list.
  enforcement: CI + adversary pass review
```

---

## Recommended Fix-Pass Scope (fix-pass-2 dispatch)

The orchestrator should dispatch fix-pass-2 with the following priorities:

1. **F-PASS2-CRIT-1:** Add `prism-audit` dependency to `prism-bin/Cargo.toml`; replace direct `prism_storage::audit_buffer` manipulation with `prism_audit::AuditEmitter::new(storage.clone())` + `emitter.emit(sentinel_event)`. (Cross-crate: prism-audit.)
2. **F-PASS2-HIGH-1:** Add `db.flush_wal(true)` (or `WriteOptions::set_sync(true)`) after sentinel write; cite BC-2.05.012 postcondition in code comment. (Subsumed if CRIT-1 fixed via AuditEmitter — check AuditEmitter's sync guarantee.)
3. **F-PASS2-HIGH-2:** Add `timestamp: chrono::Utc::now().to_rfc3339()` to sentinel JSON. (Likely subsumed by CRIT-1 if AuditEmitter enforces schema.)
4. **F-PASS2-HIGH-3:** Extend `prism_core::SensorSpec` with `credential_refs: Vec<CredentialRef>` field; update fixture with N>0 refs; ensure loop body is exercised. (Cross-crate: prism-core.)
5. **F-PASS2-MED-1:** Add `required-features = ["prism_test_injection"]` to all `[[test]]` sections in `prism-bin/Cargo.toml` that exercise injection env vars; document in `CLAUDE.md`.
6. **F-PASS2-MED-2:** Update `validate-config` clap help string to remove false "creates missing directories" claim.
7. **F-PASS2-MED-3:** Fix UUID v7 error message to match BC-2.21.001 EC-21-001-008 verbatim (`"org_id '{}' must be a UUID v7"`); update test assertion.
8. **F-PASS2-LOW-1:** Add stderr content assertion to `bc_2_06_011_config_load.rs:131` or amend AC-4 in story spec.
9. **F-PASS2-OBS-2:** Replace hardcoded `"audit_buffer"` string with `prism_storage::AUDIT_BUFFER_CF_NAME` constant.
10. **PG-2:** Codify test-injection feature pairing policy in `policies.yaml`.

After fixes, dispatch pass-3 of LOCAL adversarial cascade. Convergence streak: **0/3**.
