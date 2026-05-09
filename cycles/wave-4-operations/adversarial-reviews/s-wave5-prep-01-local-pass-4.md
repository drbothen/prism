---
document_type: adversarial-review
pass: 4
cycle: wave-4-operations
story_id: S-WAVE5-PREP-01
branch: feature/S-WAVE5-PREP-01-prism-bin-chassis
head: 345f443b
verdict: CLEAN
findings_total: 5
findings_critical: 0
findings_high: 0
findings_medium: 0
findings_low: 2
observations: 3
kudos: 5
process_gaps: 0
reviewer: adversary
date: 2026-05-09
streak: "1/3"
prior_pass_closures_verified: 5
prior_pass_closures_open: 0
inputs:
  - .factory/stories/S-WAVE5-PREP-01-prism-bin-chassis.md
  - .factory/specs/architecture/decisions/ADR-022-production-runtime-wiring.md
  - .factory/specs/behavioral-contracts/BC-2.06.011-config-load-on-startup.md
  - .factory/specs/behavioral-contracts/BC-2.21.001-org-registry-init.md
  - .factory/specs/behavioral-contracts/BC-2.03.013-credential-store-init.md
  - .factory/specs/behavioral-contracts/BC-2.05.012-audit-subsystem-init.md
  - .factory/specs/behavioral-contracts/BC-2.22.001-boot-orchestration.md
  - .factory/policies.yaml
  - .factory/cycles/wave-4-operations/adversarial-reviews/s-wave5-prep-01-local-pass-3.md
  - .factory/cycles/wave-4-operations/research/audit-emitter-architecture-2026-05-09.md
input-hash: "afbee5b"
---

# Adversarial Review — S-WAVE5-PREP-01 LOCAL Pass 4

**Story:** S-WAVE5-PREP-01 — prism-bin chassis
**Branch:** `feature/S-WAVE5-PREP-01-prism-bin-chassis`
**HEAD:** `345f443b`
**Verdict:** CLEAN
**Date:** 2026-05-09
**Streak:** 1/3 (first clean pass in cascade)

---

## Severity Trend

```
Pass 1: 1C / 3H / 5M / 3L / 3OBS  (BLOCKED-hard)
Pass 2: 1C / 3H / 3M / 1L / 3OBS  (BLOCKED-hard)
Pass 3: 0C / 1H / 1M / 1L / 2OBS  (BLOCKED-soft) ← decisive decrease
Pass 4: 0C / 0H / 0M / 2L / 3OBS + 5KUDOs  (CLEAN) ← first clean pass; streak 1/3
```

Trajectory is decisively converging. Critical dropped to zero at pass-3 and held. High dropped from 3 (pass-2) → 1 (pass-3) → 0 (pass-4). Medium dropped from 5 (pass-1) → 3 → 1 → 0. Total substantive (C+H+M) findings: 9 → 8 → 2 → 0. No regressions from pass-3.

---

## Pass-3 Closure Verification

| Finding | Severity | Status | Notes |
|---------|----------|--------|-------|
| F-PASS3-HIGH-1 | HIGH | CLOSED | CredentialRefProbe trait injection implemented. AlwaysOkProbe and MissingOneProbe mocks added. Integration test writes fixture with N>0 credential_refs to spec_tmp; boots via standard sequence; asserts credential_store.get("test-cred-1") returns Ok(Some(_)). BC-2.03.013 §Postconditions item 3 now exercised by CI. |
| F-PASS3-MED-1 | MEDIUM | CLOSED | BC-2.05.012 v1.0→v1.1 amendment by PO per research-agent recommendation. Description §lines 31-32 clarify BootAuditEmitter is the boot-time specialization distinct from request-time AuditEmitterLayer. Postcondition bullets 1+4 reflect two-phase emitter design. OQ-2 marked resolved. |
| F-PASS3-LOW-1 | LOW | CLOSED | Three-line fix-pass-2 comment block at boot.rs:786-788 deleted. No stale text remains. |
| F-PASS3-OBS-1 | OBS | CLOSED | prism-audit crate: 4 owning-crate unit tests added for BootAuditEmitter::new(), emit_boot_start(), emit_boot_complete(), and into_backend(). |
| F-PASS3-OBS-2 | OBS | CLOSED | prism-storage crate: 3 owning-crate unit tests added for append_audit_entry_sync() covering flush_wal, cf-check, and error-propagation paths. |

**Summary:** All 5 pass-3 findings CLOSED. No open carry-forwards.

---

## New Findings — Pass 4

### F-PASS4-LOW-1 — Broken Intra-Doc Link: MockCredentialRefProbe at boot.rs:522

**Severity:** LOW
**Category:** Documentation hygiene / broken reference
**Files:**
- `crates/prism-bin/src/boot.rs:522` — doc comment referencing `MockCredentialRefProbe`

**Description:**

The doc comment at `boot.rs:522` contains an intra-doc link `[MockCredentialRefProbe]` that does not resolve. The test mock types introduced in fix-pass-3 are named `AlwaysOkProbe` and `MissingOneProbe`; `MockCredentialRefProbe` was an earlier working name that was not used in the final implementation. The rustdoc renderer will emit a warning (`error[E0425]: cannot find value 'MockCredentialRefProbe'` under `--deny=rustdoc::broken_intra_doc_links`) and the link renders as dead text in generated documentation.

**Required fix:** Update the doc comment at `boot.rs:522` to reference `AlwaysOkProbe` and `MissingOneProbe` (the actual mock type names used in the test module), or remove the intra-doc link and use plain text.

**Citations:**
- `boot.rs:522` — `/// See [`MockCredentialRefProbe`] for test usage.`
- `boot.rs:~780-820` — actual mock definitions: `struct AlwaysOkProbe; struct MissingOneProbe;`

---

### F-PASS4-LOW-2 — BC-2.05.012 §Failure Path References Phantom AuditEmitter::new() Failure

**Severity:** LOW
**Category:** Spec hygiene / phantom failure path
**BC:** BC-2.05.012 §Failure Modes
**Files:**
- `.factory/specs/behavioral-contracts/BC-2.05.012-audit-subsystem-init.md:§Failure Modes`

**Description:**

BC-2.05.012 v1.1 §Failure Modes lists (paraphrased):

> "If `AuditEmitter::new()` fails to open the audit RocksDB column family, the boot sequence MUST abort with exit code 3."

However, the v1.1 amendment established that `BootAuditEmitter::new()` is the constructor actually called during boot, and `BootAuditEmitter::new()` takes a pre-constructed `Arc<StorageEngine>` that has already opened all column families (including `AUDIT_BUFFER_CF_NAME`) during `step3_open_storage()`. `BootAuditEmitter::new()` is infallible — it cannot fail because the CF is guaranteed open by the time it is called.

The phantom failure path in §Failure Modes creates a misleading contract: it suggests audit initialization can fail independently when in fact it cannot. A future implementer might add unnecessary error handling, or a future adversary might flag the absence of that handling as a bug.

**Required fix:** PO amends BC-2.05.012 v1.1 §Failure Modes to remove or restate the `AuditEmitter::new()` phantom failure path. Correct framing: "Audit initialization cannot fail independently; storage-open failure (exit code 3) is the only path that prevents audit initialization."

**Note:** This finding is LOW because it does not affect current implementation correctness — it is a spec-prose maintenance concern introduced as a residue of the v1.0→v1.1 amendment. Dispatch to PO in parallel with (or after) the implementer fix-pass-4.

---

### F-PASS4-OBS-1 — Sentinel Timestamp Skew Between Two Utc::now() Calls

**Severity:** OBSERVATION
**Category:** Audit correctness observation
**Files:**
- `crates/prism-bin/src/boot.rs` — sentinel construction and emit_boot_complete() call

**Description:**

The boot sentinel write at `boot.rs` calls `Utc::now()` to populate `sentinel_written_at` in the JSON payload, and separately `emit_boot_complete()` calls `Utc::now()` again to populate the `AuditEntry` timestamp. These are two independent wall-clock reads. Under normal operation the skew is sub-millisecond and immaterial. However, under load (tight executor, system clock NTP correction, heavy concurrent I/O) the two timestamps could diverge by up to several milliseconds, producing a sentinel JSON whose `sentinel_written_at` is slightly before or after the audit log entry for the same event.

This is not a correctness defect — both timestamps are "at boot complete" for practical purposes. It is a minor consistency observation: correlating the sentinel file against the audit log by timestamp could produce ambiguous matches if the skew exceeds the log's timestamp resolution.

**Suggested improvement (TD candidate):** Capture `let now = Utc::now()` once and pass it to both the sentinel payload construction and the audit entry — eliminating the skew entirely at zero additional cost. Suitable for the next fix-pass or as a low-priority TD.

---

### F-PASS4-OBS-2 — SIGTERM Handler Logs "Audit buffer flushed" Unconditionally

**Severity:** OBSERVATION
**Category:** Misleading log message / pre-existing
**BC:** BC-2.10.010 (deferred per story spec)
**Files:**
- `crates/prism-bin/src/signals.rs` — SIGTERM handler log message

**Description:**

The SIGTERM handler at `signals.rs` logs `"Audit buffer flushed"` at INFO level upon receiving SIGTERM. The message is unconditional — it logs "flushed" whether or not any flush actually occurred, and whether or not the audit buffer had any pending entries. In practice, the `BootAuditEmitter` is a boot-scoped resource that does not persist into the MCP runtime, so the SIGTERM handler is not connected to any live audit buffer at all. The log message is pre-existing and architecturally misleading.

**Not a blocker.** BC-2.10.010 (SIGTERM behavior) is explicitly deferred in the story spec to a follow-up story (S-5.01-FOLLOWUP-MCP-BOOT). The misleading message should be addressed when BC-2.10.010 is implemented — at that point the message will either become accurate (a real flush occurs) or can be removed. Filing as an observation so fix-pass-4 does not touch it unnecessarily.

---

### F-PASS4-OBS-3 — Credential-Ref Closure Tests Live in Feature-Gated File

**Severity:** OBSERVATION
**Category:** Test discoverability / TD candidate
**Files:**
- `crates/prism-bin/tests/boot_integration.rs` — credential-ref closure tests under `#[cfg(feature = "prism_test_injection")]`

**Description:**

The two credential-ref behavioral coverage tests added in fix-pass-3 (F-PASS3-HIGH-1 closure) reside in `boot_integration.rs` under `#[cfg(feature = "prism_test_injection")]`. When running `just iter prism-bin` without the feature flag, these tests are silently skipped (they appear in `nextest` output as "skipped: 2" with no indication of why). The skip count is included in the `just check 3456 pass / 17 skipped` report but not itemized.

**Consequence:** A developer running `just iter prism-bin` for a quick iteration cycle will not run the credential-ref coverage tests unless they explicitly pass `--features prism_test_injection`. The tests exist and CI runs them, but the discoverability is low for routine local iteration.

**Not a blocker.** The tests ARE run in CI (the feature is enabled in the CI test matrix). This is a local-developer ergonomics observation. Suitable for the TD register as a low-priority ergonomics item — possible fix: document the feature flag in a comment above the `#[cfg]` block, or add a `just iter-full prism-bin` recipe that enables the feature.

---

## KUDOs

### KUDO-1 — CredentialRefProbe Trait Design

The `CredentialRefProbe` trait is textbook Approach B dependency injection: a sealed trait with a minimal surface (`fn probe(&self, ref_id: &str) -> Result<Option<Credential>, CredentialError>`), a `KeyringCredentialProbe` production impl, and `AlwaysOkProbe`/`MissingOneProbe` test impls. The production and test paths share the same trait boundary — no test-only codepaths in the production binary. This is the correct design for testable credential resolution without introducing a mock backend into the keyring crate itself.

### KUDO-2 — Regression-Guard Test Setup

The fix-pass-3 credential-ref integration test does NOT reuse the existing boot test fixture. It explicitly writes a fresh TOML with `credential_refs = [{ id = "test-cred-1", vault_path = "secret/test" }]` into a new `tempdir()`, runs the full boot sequence, then asserts `credential_store.get("test-cred-1")` returns `Ok(Some(_))`. This is a genuine regression guard: it validates the behavior end-to-end from TOML parsing through spec-engine loading through boot-sequence credential resolution. The extra fixture-authoring effort was warranted.

### KUDO-3 — Arc::ptr_eq Identity Check on into_backend

The `BootAuditEmitter::into_backend()` unit test uses `Arc::ptr_eq(&expected_arc, &result_arc)` to verify that the returned backend is the same `Arc` instance (same allocation) rather than a clone with equal contents. This is the correct assertion for an ownership-transfer operation: it confirms the `Arc` is moved out, not cloned, preserving the reference-count semantics the caller depends on.

### KUDO-4 — Error-Detail Substring Assertion

The `MissingOneProbe` test asserts that the returned error message contains `"test-cred-1"` as a substring, rather than asserting an exact message string. This is the correct pattern for error-detail assertions: it locks in the invariant that the error names the failing ref-id (critical for operator debugging) without coupling the test to the exact phrasing of the error message (which is an implementation detail that can improve without breaking the behavioral contract).

### KUDO-5 — BC Amendment Cites Architectural Research Artifact

BC-2.05.012 v1.1 §Rationale cites `cycles/wave-4-operations/research/audit-emitter-architecture-2026-05-09.md` as the basis for the two-phase emitter design decision. Spec amendments grounded in a durable research artifact (rather than inline reasoning) make the amendment traceable and reversible — a future maintainer can read the research document to understand why the design was chosen, not just what was decided.

---

## Anti-Padding Self-Check

Three candidate "findings" were evaluated and rejected:

1. **`BootAuditEmitter::new()` parameter order** (storage before config) — consistent with workspace convention for resource-before-config ordering. No defect. Dropped.
2. **`AlwaysOkProbe` naming** (vs `AlwaysSucceedsProbe`) — naming preference, no behavioral consequence. The `Ok`-suffix is standard in Rust test helpers (e.g., `AlwaysOkFuture`). Dropped.
3. **`append_audit_entry_sync` flush_wal test** — tested with `flush = true`. Verified this is the production-relevant case. No gap. Dropped.

---

## Verdict Summary

| Category | Count |
|----------|-------|
| Critical | 0 |
| High | 0 |
| Medium | 0 |
| Low | 2 (F-PASS4-LOW-1: broken intra-doc link; F-PASS4-LOW-2: BC-2.05.012 phantom failure path) |
| Observations | 3 (F-PASS4-OBS-1: timestamp skew; F-PASS4-OBS-2: SIGTERM log pre-existing; F-PASS4-OBS-3: feature-gated tests skip silently) |
| KUDOs | 5 |
| Process Gaps | 0 |

**Verdict: CLEAN** — No critical, high, or medium findings. Two LOWs and three OBSERVATIONs, none blocking. All 5 pass-3 findings verified CLOSED. Severity trend decisively decreasing across all four passes.

**Convergence streak: 1/3.** Recommended path:

- **Implementer:** Fix broken intra-doc link at `boot.rs:522` (F-PASS4-LOW-1) — ~5 min
- **PO:** Amend BC-2.05.012 §Failure Modes to remove phantom `AuditEmitter::new()` failure path (F-PASS4-LOW-2) — ~15 min
- **Implementer:** Capture single `Utc::now()` for sentinel + audit entry (F-PASS4-OBS-1) — ~10 min
- **Implementer:** Add clarifying comment above SIGTERM "Audit buffer flushed" log noting BC-2.10.010 deferral (F-PASS4-OBS-2) — ~5 min; or defer entirely
- **TD register:** Add ergonomics item for feature-gated test discoverability (F-PASS4-OBS-3) — defer

Pass-5 after a brief fix-pass-4 is expected to be CLEAN (streak 2/3). If pass-5 is also CLEAN, pass-6 closes the 3/3 window and story converges.
