---
document_type: adversarial-review
pass: 3
cycle: wave-4-operations
story_id: S-WAVE5-PREP-01
branch: feature/S-WAVE5-PREP-01-prism-bin-chassis
head: 5469b3b4
verdict: BLOCKED-soft
findings_total: 5
findings_critical: 0
findings_high: 1
findings_medium: 1
findings_low: 1
observations: 2
kudos: 4
process_gaps: 0
reviewer: adversary
date: 2026-05-09
streak: "0/3"
prior_pass_closures_verified: 11
prior_pass_closures_open: 2
inputs:
  - .factory/stories/S-WAVE5-PREP-01-prism-bin-chassis.md
  - .factory/specs/architecture/decisions/ADR-022-production-runtime-wiring.md
  - .factory/specs/behavioral-contracts/BC-2.06.011-config-load-on-startup.md
  - .factory/specs/behavioral-contracts/BC-2.21.001-org-registry-init.md
  - .factory/specs/behavioral-contracts/BC-2.03.013-credential-store-init.md
  - .factory/specs/behavioral-contracts/BC-2.05.012-audit-subsystem-init.md
  - .factory/specs/behavioral-contracts/BC-2.22.001-boot-orchestration.md
  - .factory/policies.yaml
  - .factory/cycles/wave-4-operations/adversarial-reviews/s-wave5-prep-01-local-pass-2.md
input-hash: "1324b83e712a36a1183a63095e0ffb61"
---

# Adversarial Review — S-WAVE5-PREP-01 LOCAL Pass 3

**Story:** S-WAVE5-PREP-01 — prism-bin chassis
**Branch:** `feature/S-WAVE5-PREP-01-prism-bin-chassis`
**HEAD:** `5469b3b4`
**Verdict:** BLOCKED-soft
**Date:** 2026-05-09
**Streak:** 0/3 (BLOCKED-soft resets streak)

---

## Severity Trend

```
Pass 1: 1C / 3H / 5M / 3L / 3OBS  (BLOCKED-hard)
Pass 2: 1C / 3H / 3M / 1L / 3OBS  (BLOCKED-hard)
Pass 3: 0C / 1H / 1M / 1L / 2OBS  (BLOCKED-soft) ← decisive decrease
```

Trajectory is decisively improving. Critical count has dropped to zero. High count has dropped from 3 to 1. Total substantive findings have dropped from 12 → 8 → 3. No regressions from pass-2.

---

## Pass-2 Closure Verification

| Finding | Severity | Status | Notes |
|---------|----------|--------|-------|
| F-PASS2-CRIT-1 | CRITICAL | OPEN (partial fix) | Data model fixed in `prism-spec-engine`: `SensorSpec` now has `credential_refs: Vec<CredentialRef>`. However no integration test copies the fixture with N>0 credential_refs into `spec_tmp` — the vacuous-loop defect is partially closed at the data model level but the behavioral coverage gap (AC-AC-8 / BC-2.03.013 §Postconditions) is unverified by any test. Elevated to F-PASS3-HIGH-1 (refined carry-forward). |
| F-PASS2-HIGH-1 | HIGH | CLOSED | `fsync` call present after sentinel write in `boot.rs`. Verified at `boot.rs:603-611`. |
| F-PASS2-HIGH-2 | HIGH | CLOSED | RFC3339 timestamp field `sentinel_written_at` present in sentinel JSON payload. Verified at `boot.rs:598`. |
| F-PASS2-HIGH-3 | HIGH | CLOSED (data model) | `SensorSpec.credential_refs` field added in `prism-core`. Integration test coverage gap remains — see F-PASS3-HIGH-1. |
| F-PASS2-MED-1 | MEDIUM | CLOSED | `prism_audit::AuditEmitter` constructor called correctly; dependency added to `prism-bin/Cargo.toml`. |
| F-PASS2-MED-2 | MEDIUM | CLOSED | `required-features = ["prism_test_injection"]` added to `[[test]]` sections in `prism-bin/Cargo.toml`. |
| F-PASS2-MED-3 | MEDIUM | CLOSED | `validate-config` subcommand help text corrected. |
| F-PASS2-LOW-1 | LOW | CLOSED | UUID v7 error message uses canonical format. |
| F-PASS2-OBS-1 | OBS | CLOSED | AC-4 stderr assertion added. |
| F-PASS2-OBS-2 | OBS | CLOSED | `AUDIT_BUFFER_CF_NAME` constant used in place of inline string literal. |
| F-PASS2-OBS-3 | OBS | OPEN-LOW (false-negative in fix-pass-2 comment) | Fix-pass-2 commit message comment claimed "S-5.01-FOLLOWUP-MCP-BOOT story doesn't exist in STORY-INDEX." That story DOES exist at STORY-INDEX line 387. The comment was a false-negative read — the story was present. No code fix required; the comment is inaccurate. |
| PG-1 | PROCESS-GAP | CLOSED | POL-12 vs AC-11 conflict for `pub fn` with `todo!()` resolved via story amendment. |
| PG-2 | PROCESS-GAP | CLOSED | `policies.yaml` codification complete. |

**Summary:** 11 of 13 pass-2 findings CLOSED. 2 OPEN:
- F-PASS2-CRIT-1: OPEN-PARTIAL — data model fixed, integration coverage gap remains → elevated to F-PASS3-HIGH-1
- F-PASS2-OBS-3: OPEN-LOW — false-negative comment; no functional defect

---

## New Findings — Pass 3

### F-PASS3-HIGH-1 — Credential-Ref Behavioral Coverage Gap (Refined Carry-Forward)

**Severity:** HIGH
**Category:** Behavioral coverage / BC compliance
**BC:** BC-2.03.013 §Postconditions
**Files:**
- `crates/prism-bin/src/boot.rs:712-728` — credential ref iteration loop
- `crates/prism-bin/tests/boot_integration.rs:189-234` — credential store init test
- `crates/prism-spec-engine/src/lib.rs:441-459` — `load_all_specs()` fixture path

**Description:**

`prism-core::SensorSpec` now carries `credential_refs: Vec<CredentialRef>` (data model fix from pass-2). The boot sequence at `boot.rs:712-728` iterates `spec.credential_refs` and calls `credential_store.resolve(ref_id)` for each entry. This satisfies the structural requirement of BC-2.03.013.

However, the integration test at `boot_integration.rs:189-234` exercises the credential store init path using the default test fixture loaded from `spec_tmp`. The fixture sensor TOML at `crates/prism-spec-engine/tests/fixtures/test_sensor.sensor.toml` has `credential_refs = []` — zero entries. The iteration loop runs zero times in every test configuration.

**Consequence:** BC-2.03.013 §Postconditions item 3 ("For each sensor spec with at least one credential_ref, the credential store MUST have resolved and cached the referenced credential") is never exercised by any test. A regression in `credential_store.resolve()` or in the iteration logic would not be caught by CI.

**Required fix:** Add an integration test that:
1. Writes a fixture TOML with `credential_refs = [{ id = "test-cred-1", vault_path = "secret/test" }]` to `spec_tmp`
2. Boots the system via the standard boot sequence
3. Asserts that `credential_store.get("test-cred-1")` returns `Ok(Some(_))`

This is a behavioral coverage gap, not a data model gap. The data model is correct. The behavior is unverified.

**Citations:**
- `boot.rs:712` — `for cred_ref in &spec.credential_refs {`
- `boot.rs:715` — `credential_store.resolve(&cred_ref.id)?;`
- `boot.rs:728` — end of loop
- `boot_integration.rs:203` — `let spec_tmp = tempdir()?;` (fixture dir created empty)
- `boot_integration.rs:211` — no TOML written with credential_refs
- `prism-spec-engine/src/lib.rs:448` — `load_all_specs(&spec_dir)` reads from spec_tmp
- `test_sensor.sensor.toml:12` — `credential_refs = []`
- `BC-2.03.013:§Postconditions item 3` — behavioral requirement
- `prism-core/src/sensor_spec.rs:34` — `pub credential_refs: Vec<CredentialRef>`

---

### F-PASS3-MED-1 — BC-2.05.012 Postcondition 4 Drift: BootAuditEmitter Constructed and Dropped

**Severity:** MEDIUM
**Category:** Spec compliance / BC postcondition drift
**BC:** BC-2.05.012 §Postconditions item 4
**Files:**
- `crates/prism-bin/src/boot.rs:786-800` — `BootAuditEmitter` construction and use
- `.factory/specs/behavioral-contracts/BC-2.05.012-audit-subsystem-init.md:§Postconditions`

**Description:**

BC-2.05.012 §Postconditions item 4 states (paraphrased from the current BC text):

> "The `AuditEmitter` instance MUST be propagated to the MCP server context for use throughout the process lifetime. It MUST NOT be dropped at the end of the boot phase."

The current implementation at `boot.rs:786-800`:

```rust
let boot_audit = BootAuditEmitter::new(storage.clone());
boot_audit.emit_boot_start(&config)?;
// ... boot steps ...
boot_audit.emit_boot_complete(&sentinel)?;
// boot_audit dropped here — end of boot() scope
```

`BootAuditEmitter` is constructed, used to emit boot-lifecycle events, and then silently dropped when `boot()` returns. It is NOT propagated to the MCP server context. The `McpServerContext` struct (at `crates/prism-bin/src/mcp_context.rs:18-42`) does not contain an `AuditEmitter` field.

**Why this is MEDIUM and not HIGH:** The boot-lifecycle audit events (start + complete sentinel) are emitted before the drop. No audit events are lost for the boot phase itself. However, the postcondition is architecturally wrong: the BC requires the emitter to be propagated forward, and the current design treats it as a scoped boot-only resource. This creates a spec/impl divergence that will cause confusion for the MCP audit surface (S-5.01-FOLLOWUP-MCP-BOOT story).

**Note:** The research-agent's concurrent analysis (2026-05-09) independently identified this as a BC wording problem rather than an implementation defect. The recommended resolution is to amend BC-2.05.012 to reflect the `BootAuditEmitter` pattern: boot-phase-scoped emitter for boot events, separate `AuditEmitter` for runtime events. The amendment closes this finding at the spec level rather than requiring implementation changes.

**Required resolution (two paths, parallel):**
- Path A (product-owner): Amend BC-2.05.012 §Postconditions item 4 per research-agent recommendation
- Path B (implementer): If BC amendment is not approved, propagate `BootAuditEmitter` into `McpServerContext`

Research-agent recommendation (from concurrent analysis): Path A. See `cycles/wave-4-operations/research/audit-emitter-architecture-2026-05-09.md`.

---

### F-PASS3-LOW-1 — Stale Fix-Pass-2 Comment in boot.rs

**Severity:** LOW
**Category:** Code hygiene / dead comment
**Files:**
- `crates/prism-bin/src/boot.rs:786-788`

**Description:**

Lines 786-788 contain:

```rust
// fix-pass-2: AuditEmitter now wired per BC-2.05.012. BootAuditEmitter
// retained as scoped boot-phase resource per ADR-022 §Boot Sequence
// step 6. S-5.01-FOLLOWUP-MCP-BOOT story doesn't exist in STORY-INDEX.
```

The third line is factually incorrect (S-5.01-FOLLOWUP-MCP-BOOT exists at STORY-INDEX line 387 — this was identified as F-PASS2-OBS-3, a false-negative comment). The comment serves no behavioral purpose and should be removed.

**Required fix:** Delete lines 786-788 (the three-line fix-pass-2 comment block).

---

### F-PASS3-OBS-1 — prism-audit Crate Has No BootAuditEmitter Owning-Crate Unit Test

**Severity:** OBSERVATION
**Category:** Test coverage observation
**Files:**
- `crates/prism-audit/src/lib.rs` — `BootAuditEmitter` implementation
- `crates/prism-audit/tests/` — missing

**Description:**

`BootAuditEmitter` is defined in `crates/prism-audit/src/lib.rs`. The owning crate (`prism-audit`) has no unit test that directly exercises `BootAuditEmitter::new()`, `emit_boot_start()`, or `emit_boot_complete()`. The only test coverage comes from the integration test in `prism-bin`. Per workspace convention (BC-3.7.001), substantive behavior should have owning-crate unit tests before integration-level coverage.

**Not a blocker.** The integration test in prism-bin provides meaningful coverage. This is a coverage observation for the TD register.

---

### F-PASS3-OBS-2 — prism-storage `append_audit_entry_sync` Has No Owning-Crate Unit Test

**Severity:** OBSERVATION
**Category:** Test coverage observation
**Files:**
- `crates/prism-storage/src/audit_buffer.rs` — `append_audit_entry_sync()` implementation
- `crates/prism-storage/tests/` — missing entry for this function

**Description:**

`append_audit_entry_sync()` in `prism-storage` is exercised only through the integration path (prism-bin boot → BootAuditEmitter → storage). No owning-crate unit test directly calls `append_audit_entry_sync()` with known inputs and verifies the written bytes. This is a coverage gap at the storage layer.

**Not a blocker.** Same rationale as F-PASS3-OBS-1.

---

## KUDOs

### KUDO-1 — fsync Implementation Quality

The `fsync` fix (closing F-PASS2-HIGH-1) is correctly placed: `file.flush()?; file.sync_all()?;` using `sync_all()` rather than `sync_data()` — ensures filesystem metadata (directory entry) is flushed in addition to data, which is the correct pattern for durable audit sentinel writes. This exceeds the minimum requirement.

### KUDO-2 — RFC3339 Timestamp Precision

The sentinel payload uses `chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)` — millisecond precision with UTC timezone marker. This is best-practice for audit timestamps and matches the SOC 2 CC7.1 audit trail completeness requirements more precisely than second-level resolution.

### KUDO-3 — required-features Enforcement Completeness

The `required-features = ["prism_test_injection"]` fix applied to all three `[[test]]` sections that reference test-injection behavior. No gaps observed. The implementer also added a comment in `CLAUDE.md` documenting the pattern for future contributors. Proactive documentation of a non-obvious constraint is commendable.

### KUDO-4 — credential_refs Data Model Design

`CredentialRef` is defined as a newtype struct `pub struct CredentialRef { pub id: String, pub vault_path: String }` rather than a bare tuple or a type alias. This is the correct design choice: named fields prevent field-order confusion, and the newtype ensures future extensibility (e.g., adding `rotation_policy`) without breaking existing call sites. The `#[derive(Debug, Clone, Serialize, Deserialize)]` derives are complete.

---

## Anti-Padding Self-Check

Three candidate "findings" were evaluated and rejected:

1. **`BootAuditEmitter` field naming** (`boot_audit` vs `audit_emitter`) — naming preference, no behavioral consequence. Dropped.
2. **`spec_tmp` lifetime** — `tempdir()` lifetime is correctly managed; the fixture dir lives for the duration of the test. No defect. Dropped.
3. **`validate-config` output format** — Output matches story AC wording. No drift detected. Dropped.

---

## Verdict Summary

| Category | Count |
|----------|-------|
| Critical | 0 |
| High | 1 (F-PASS3-HIGH-1: cred-ref integration coverage gap) |
| Medium | 1 (F-PASS3-MED-1: BC-2.05.012 postcondition 4 drift) |
| Low | 1 (F-PASS3-LOW-1: stale comment) |
| Observations | 2 (F-PASS3-OBS-1/2: owning-crate unit test gaps) |
| KUDOs | 4 |
| Process Gaps | 0 |

**Verdict: BLOCKED-soft** — No critical findings. One HIGH (behavioral coverage gap, not data model defect). One MEDIUM (resolvable via BC amendment per research-agent recommendation). Severity trend is decisively decreasing. Pass-4 is achievable with focused work:

- Product owner: amend BC-2.05.012 (closes F-PASS3-MED-1)
- Implementer: add cred-ref integration test with N>0 fixture (closes F-PASS3-HIGH-1)
- Implementer: delete stale comment at boot.rs:786-788 (closes F-PASS3-LOW-1)
- Optional (TD candidates): owning-crate unit tests for BootAuditEmitter + append_audit_entry_sync

Streak reset to 0/3.
