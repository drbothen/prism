---
document_type: gate-step-report
gate_step: c
gate_step_name: code-review
cycle: phase-3-dtu-wave-2
gate: wave-2-integration-gate
scope: e45159b9..c239dd0b (full Wave 2 + Pass 1-5 fix-PRs + 7 CI hotfixes)
reviewer: vsdd-factory:code-reviewer
date: 2026-04-26
verdict: FINDINGS_OPEN
total_findings: 14
high: 2
medium: 6
low: 6
---

# Wave 2 Integration Gate — Gate Step C: Code Review

**Scope:** e45159b9..c239dd0b (full Wave 2 + Pass 1-5 fix-PRs + 7 CI hotfixes)
**Reviewer:** vsdd-factory:code-reviewer
**Date:** 2026-04-26
**Verdict:** FINDINGS_OPEN — 14 findings (2 HIGH, 6 MEDIUM, 6 LOW)

---

## HIGH Findings

### WGC-W2-001 (HIGH): S-2.05 Audit Emitters Do Not Persist to Storage — Silently Non-Functional

**File:line:** `crates/prism-audit/src/credential_events.rs:123–155`, `crates/prism-audit/src/flag_events.rs:106–128`, `crates/prism-audit/src/token_events.rs:110–142` (and `emit_token_consumed`, `emit_token_expired` similarly)

**Issue:** The three S-2.05 emitter functions (`emit_credential_event`, `emit_flag_eval`, `emit_token_generated`/`consumed`/`expired`) all claim in their doc comments to "call `AuditEmitter::emit()` to persist the entry to the `audit_buffer` CF" and declare `Err(AuditPersistenceFailed)` as a return path. In practice, every function body constructs the `parameters` JSON blob, logs it via `tracing::info!`, then returns `Ok(())`. No storage backend is accepted as a parameter. No `append_audit_entry` call is present. `AuditPersistenceFailed` is never returned.

**Evidence:** `emit_credential_event` signature is `fn emit_credential_event(name, sensor_id, access_type, result, ctx) -> Result<(), PrismError>` — no backend parameter. Body ends with `tracing::info!(...); Ok(())`. Same pattern in `flag_events` and `token_events`.

**Recommendation:** Either (a) add a `backend: &dyn RocksStorageBackend` parameter to each emitter and call `append_audit_entry` (matching the stated contract), or (b) explicitly document that S-2.05 emitters are log-only stubs deferred to S-3.02 and adjust doc comments to not claim RocksDB persistence. The current state is dangerous because callers expect durable compliance records and will not receive them; the doc contract is a lie that will mislead future developers.

---

### WGC-W2-002 (HIGH): `evict_expired` Only Scans In-Memory Cache — Backend Keys Never Evicted After Restart

**File:line:** `crates/prism-sensors/src/event_buffer.rs:316–340`

**Issue:** `evict_expired()` collects keys to delete by scanning `self.write_cache` (the in-memory `BTreeMap`). It then deletes those keys from both the cache and the RocksDB backend. However, it never calls `self.backend.scan(...)` to find keys that exist in RocksDB from a previous process run. After a process restart, the `write_cache` starts empty. Any records written to RocksDB before the crash will not be discovered by `evict_expired` and will remain in RocksDB permanently, violating the TTL eviction contract (AC-4).

**Evidence:** Lines 316–340 filter only `cache_guard.keys()`. Contrast with `has_data()` (line 408) and `buffer_size_bytes()` (line 451) which both fall back to `self.backend.scan(...)` when the cache is empty.

**Recommendation:** Add a backend scan fallback inside `evict_expired` parallel to how `has_data` handles cross-restart data. Specifically, if `cache_guard` is empty or the scope has no matching entries in cache, scan the backend with `self.backend.scan(StorageDomain::EventBuffer, scope_bytes)` and decode timestamps to identify expired keys.

---

## MEDIUM Findings

### WGC-W2-003 (MEDIUM): Hardcoded `SensorType::CrowdStrike` in Task Panic Handler

**File:line:** `crates/prism-sensors/src/fanout.rs:354–362`

**Issue:** In the `fan_out` function's `join_all` loop, when a spawned task panics (the `Err(join_err)` arm), a `FanOutError` is constructed with `sensor_type: prism_core::types::SensorType::CrowdStrike` hardcoded. Since the original target is consumed by the `tokio::spawn` closure, its `sensor_type` is unavailable here. The workaround pins the error to CrowdStrike regardless of which sensor actually panicked, creating misleading telemetry.

**Recommendation:** Move the `client_id` and `sensor_type` values into the task result to preserve them across the `JoinHandle` boundary.

---

### WGC-W2-004 (MEDIUM): `CrowdStrikeAdapter::new` Silently Falls Back on HTTP Client Build Failure

**File:line:** `crates/prism-sensors/src/auth/crowdstrike.rs:128–131`

**Issue:** `Client::builder().cookie_store(false).build().unwrap_or_default()` silently swallows a TLS/platform failure. `cookie_store(false)` (and any future security-critical options) could be silently dropped if the build fails.

**Recommendation:** Change to `.build().expect("failed to build reqwest Client — TLS init failed")` or propagate via `try_new -> Result`.

---

### WGC-W2-005 (MEDIUM): `event_key` Uses Two Separate `SystemTime::now()` Calls for Timestamp and Nanos Suffix

**File:line:** `crates/prism-sensors/src/event_buffer.rs:84–99`

**Issue:** The `event_key` function calls `SystemTime::now()` once for the microsecond-precision prefix and a second time for the nanos suffix. These two calls are not guaranteed to produce consistent values and introduce a subtle temporal inconsistency in the key construction.

**Recommendation:** Derive both the microsecond prefix and nanos suffix from the same `record.ingested_at` value.

---

### WGC-W2-006 (MEDIUM): `CredentialAccessType::Rotate` Variant Has Wrong Doc Comment ("List")

**File:line:** `crates/prism-audit/src/credential_events.rs:29–30`

**Issue:** The `Rotate` variant is documented as "List credentials for a sensor" — this describes the `List` operation, not rotation. This is a copy-paste documentation error that will mislead callers.

**Recommendation:** Either rename the variant to `List` (matching the doc) or fix the doc to describe `Rotate` semantics.

---

### WGC-W2-007 (MEDIUM): Duplicate `CapabilityCheckResult` Type — One Not Re-exported, Naming Collision

**File:line:** `crates/prism-audit/src/audit_entry.rs:87`, `crates/prism-audit/src/write_audit.rs:17`

**Issue:** `CapabilityCheckResult` is defined in two locations in `prism-audit`. One is used internally and not re-exported; the other is public. This creates a naming collision and confuses downstream consumers about which type to use.

**Recommendation:** Consolidate to one `CapabilityCheckResult` or rename one to `WriteCapabilityCheckResult`.

---

### WGC-W2-008 (MEDIUM): Token Cache in `CrowdStrikeAdapter` Has TOCTOU Race

**File:line:** `crates/prism-sensors/src/auth/crowdstrike.rs:208–219`

**Issue:** The token cache uses a read-lock check (`if token is expired`) followed by a write-lock replacement. Between the read-lock drop and write-lock acquisition, another thread may have already refreshed the token, leading to redundant refresh calls and potential brief window of using a stale token.

**Recommendation:** Double-checked locking with write lock + re-check, or use `tokio::sync::OnceCell` for single-flight initialization.

---

## LOW Findings

### WGC-W2-009 (LOW): `fanout.rs` Contains Dead Duplicate of Fan-Out Logic (`execute_target`)

Dead code: the `execute_target` function in `fanout.rs` duplicates logic already present in the main `fan_out` function body. The function is never called.

### WGC-W2-010 (LOW): `DateTime::from_timestamp` Is Deprecated in `chrono` 0.4.23+

Multiple files use the deprecated `DateTime::from_timestamp(secs, 0)` form. The preferred API is `DateTime::from_timestamp(secs, 0).expect(...)` or the `chrono::DateTime::from_timestamp_opt` variant.

### WGC-W2-011 (LOW): `retry_forward_entry` in `audit_buffer.rs` Is Permanently Stubbed and Always Errors

`retry_forward_entry` returns `Err(PrismError::AuditPersistenceFailed)` unconditionally. The stub comment says "TODO: implement retry logic" but there is no tracking issue and no corresponding story.

### WGC-W2-012 (LOW): `DecorationStore` Fields and Helper Are Suppressed With `#[allow(dead_code)]`

Three fields and one helper in `decoration_store.rs` are suppressed with `#[allow(dead_code)]`. These are either genuine dead code that should be removed, or forward-declarations that should be moved to the story that uses them.

### WGC-W2-013 (LOW): `paginate_claroty` Silently Aborts Stream on `total_count == 0` Without Error

When the Claroty API returns `total_count == 0` on the first page, `paginate_claroty` returns `Ok(vec![])` without logging or surfacing this as even an informational event. A legitimate empty result and a misconfigured query are indistinguishable.

### WGC-W2-014 (LOW): `AuditEmitterService::call` Reconstructs `AuditedResponse` Redundantly on Inner Error

The `call` method reconstructs an `AuditedResponse` twice in some error paths — once in the happy path and once in the error arm — where the second construction is immediately overwritten by an `Err(...)` return.

---

## Positive Observations on Wave 2 Code Quality

- **Sealed-trait pattern** for `SensorAuth` correctly implemented with private `Sealed` marker trait — downstream crates cannot implement the auth trait, preventing unvalidated adapter bypass.
- **`TableType` canonicalization** in `prism-core` with `pub use prism_core::TableType` in `table_dispatch.rs` — single source of truth across crates, D-026 decision fully realized.
- **Dual-semaphore design** (`fanout-10` + `HTTP-200`) in `fanout.rs` is clean and correctly implements the DoS-resistance contract from BC-2.01.012.
- **`redact()` recursive walk** in `redaction.rs` handles nested objects at arbitrary depth — not just top-level keys.
- **`OffsetCursor::advance` with `saturating_add`** prevents offset regression under overflow (DI-001 mitigation pattern).
- **All three DTU state structs** follow the same clean pattern with `Arc<Mutex<FailureMode>>` — consistent and auditable.
- **`AuditEntry::new`** carries all required SOC 2 / ISO 27001 fields with `static_assertions::assert_fields!` compile-time check — prevents field omission regressions.
- **`ResourceWatchdog`** with `Arc<dyn MemoryProbe>` test seam and `DashMap` for concurrent token cancellation is well-structured; easy to test in isolation.
