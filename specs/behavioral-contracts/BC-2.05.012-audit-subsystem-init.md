---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-08T00:00:00Z
phase: 3
origin: greenfield
subsystem: "SS-05"
capability: "CAP-007"
lifecycle: draft
anchored_stories: [S-WAVE5-PREP-01]
verifying_vps: []
crates: [prism-bin, prism-audit]
inputs:
  - .factory/specs/architecture/decisions/ADR-022-production-runtime-wiring.md
  - .factory/specs/architecture/module-decomposition.md
input-hash: "[md5]"
traces_to: ["CAP-007"]
---

# BC-2.05.012: AuditEmitter Initialization — audit_buffer CF Open and boot.audit.initialized Emitted at Process Start

## Description

This BC is the Audit Trail subsystem's (SS-05) startup-time contract. It specifies how
`prism-bin` constructs the `AuditEmitter` and opens the `audit_buffer` RocksDB column family at
boot step 6 (per ADR-022 §B). The orchestration of this and the other 3 subsystem init contracts
in §B order is specified separately in BC-2.22.001.

`prism-bin` constructs the `AuditEmitter` handle (from `prism-audit`) and opens the `audit_buffer`
RocksDB column family (per AD-004). The audit subsystem MUST be operational before any step that
could log audit events — steps 7+ in ADR-022 §B all produce audit-loggable events and must not
begin until this step completes. Upon successful init, the audit subsystem emits the sentinel event
`"boot.audit.initialized"` which is durably persisted to the `audit_buffer` CF before step 7
begins. On any failure (RocksDB CF open failure, WAL unwriteable, write-ahead log inaccessible),
the process exits with code 4 (internal-error).

The audit subsystem is non-negotiable for SOC 2 compliance: there is no degraded-mode path that
skips or defers audit initialization.

No `todo!()`, `unimplemented!()`, or `panic!("stub...")` may appear in the production code
path for this step at or after story S-WAVE5-PREP-01 merges (POL-12 enforcement).

## Preconditions

- BC-2.06.011 is satisfied: valid `PrismConfig` handle available (provides `state_dir` path)
- BC-2.21.001 is satisfied: valid `OrgRegistry` handle available
- BC-2.03.013 is satisfied: valid `CredentialStore` handle available
- Boot steps 2, 3, 4, and 5 have all completed without error (ADR-022 §B ordering)
- The filesystem at `state_dir` is writable by the process user
- RocksDB can be opened at the configured `state_dir` path

## Postconditions

**Happy path:**
- `AuditEmitter` handle is constructed from the `prism-audit` crate
- The `audit_buffer` RocksDB column family is opened and confirmed writable
- The sentinel audit event `"boot.audit.initialized"` is constructed and written to the
  `audit_buffer` CF BEFORE step 7 begins — this write is synchronous and confirmed durable
  (not queued asynchronously)
- The `AuditEmitter` handle (wrapped in `Arc`) is available to all subsequent boot steps
  and the MCP tool dispatch middleware
- Boot continues to step 7 (storage + internal-tables provider init) per ADR-022 §B ordering
- Log line: `tracing::info!("Audit subsystem initialized; boot.audit.initialized persisted")`

**Failure path — RocksDB CF open failure:**
- The `audit_buffer` column family cannot be opened (RocksDB LOCK conflict, corrupted SST
  files, insufficient disk space for WAL)
- The process emits a `tracing::error!("Audit subsystem init failed: {err}")` log
- The process exits with code **4** (internal-error) per ADR-022 §A
- Step 7 never begins

**Failure path — WAL unwriteable:**
- The RocksDB write-ahead log cannot be written to (disk full, filesystem read-only)
- The process emits a `tracing::error!` describing the WAL failure
- The process exits with code **4**
- Step 7 never begins

**Failure path — AuditEmitter construction failure:**
- The `AuditEmitter::new()` constructor returns an error (internal initialization failure)
- The process emits a `tracing::error!` describing the failure
- The process exits with code **4**
- Step 7 never begins

**Failure path — boot.audit.initialized persistence failure:**
- The `AuditEmitter` is constructed but the sentinel write fails
- The process treats this as equivalent to CF-open failure: exit code **4**
- Step 7 never begins — partial audit initialization with no confirmed persistence is not
  an acceptable runtime state

## Invariants

- Boot step 6 is blocking: no concurrent execution with step 7 (ADR-022 §B "Traffic gate")
- Exit code on any audit failure is exactly 4, never 1, 2, 3, or 5 (ADR-022 §A canonical table)
- The `audit_buffer` CF MUST be open and confirmed writable before any audit event can be
  produced by steps 7+ (SOC 2 requirement: audit must be operational before any auditable event)
- The `"boot.audit.initialized"` sentinel write is SYNCHRONOUS and DURABLE — it must complete
  with an `fsync`-equivalent guarantee before step 7 begins. Async queuing is insufficient.
- Audit initialization is non-optional: there is no `--skip-audit` flag or degraded-mode
  path. If audit fails, the process fails. This is a SOC 2 hard requirement.
- The audit subsystem must remain operational for the process lifetime; the `AuditEmitter`
  handle must not be dropped until after the graceful shutdown flush (SIGTERM handler)

## boot.audit.initialized Sentinel Event Schema

The sentinel event written at the end of this step MUST conform to the AuditEntry structure
defined in BC-2.05.002 (structured JSON format). Minimum required fields:

```json
{
  "event_type": "boot.audit.initialized",
  "timestamp": "<RFC 3339>",
  "prism_version": "<semver from Cargo.toml>",
  "config_dir": "<redacted path or hash>",
  "org_count": <integer>,
  "boot_step": 6
}
```

The `config_dir` field MUST be redacted (only a hash or basename, not the full path) to
avoid leaking filesystem layout in audit logs that may be forwarded externally.

## Error Cases

| Error Code | Condition | Behavior |
|------------|-----------|----------|
| Exit 4 | RocksDB cannot open `audit_buffer` CF | "Audit subsystem init failed: RocksDB CF open error: {detail}"; exit 4 |
| Exit 4 | RocksDB WAL directory not writable | "Audit subsystem init failed: WAL unwriteable: {path}"; exit 4 |
| Exit 4 | `AuditEmitter::new()` returns Err | "Audit subsystem init failed: {err}"; exit 4 |
| Exit 4 | `boot.audit.initialized` sentinel write fails | "Audit subsystem init failed: sentinel persistence error"; exit 4 |
| Exit 4 | Disk full during CF open | "Audit subsystem init failed: no space left on device"; exit 4 |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-05-012-001 | `audit_buffer` CF was partially written before crash (previous run) | RocksDB WAL recovery applies; CF opens normally; `boot.audit.initialized` is appended (duplicate sentinel events are acceptable — idempotent append-only log) |
| EC-05-012-002 | State directory (`state_dir`) does not exist | RocksDB cannot create it without explicit `create_if_missing` flag; exit 4 OR step 2 validation catches missing dir — if step 2 validates `state_dir` exists, this cannot occur at step 6 |
| EC-05-012-003 | `audit_buffer` CF is present but has corrupted SST files | RocksDB returns an error on open; exit 4 with "RocksDB CF open error" |
| EC-05-012-004 | Disk fills exactly between CF open success and sentinel write | CF is open but sentinel write fails; exit 4 — partial init is not acceptable |
| EC-05-012-005 | Process is killed (SIGKILL) after step 6 but before step 7 | RocksDB WAL recovery on next start preserves the `boot.audit.initialized` sentinel from the previous run; new sentinel is appended on next successful init |
| EC-05-012-006 | RocksDB LOCK file exists from a zombie Prism process | RocksDB returns "Lock file held by another process"; exit 4 with actionable message: "Another Prism process may be running. Check {state_dir}/LOCK" |

## Canonical Test Vectors

| ID | Scenario | Setup | Expected Exit Code | Expected Log Output |
|----|----------|-------|-------------------|---------------------|
| TV-05-012-001 | Valid state_dir, writable | Clean state_dir with write permission | Boot continues | `tracing::info!("Audit subsystem initialized; boot.audit.initialized persisted")` |
| TV-05-012-002 | State_dir not writable | `chmod 444 state_dir` | 4 | "Audit subsystem init failed: ..." |
| TV-05-012-003 | RocksDB LOCK held | Touch `state_dir/LOCK` before start | 4 | "Another Prism process may be running" |
| TV-05-012-004 | Disk full simulation | Preallocate disk, no space for WAL | 4 | "no space left on device" |
| TV-05-012-005 | Sentinel is durably written | Inspect `audit_buffer` CF after step 6 completes | N/A (unit test) | CF contains exactly one `boot.audit.initialized` record |
| TV-05-012-006 | Sentinel schema compliance | Read sentinel from CF; parse as JSON | N/A (unit test) | All required fields present per sentinel schema |

## Test Strategy

Integration tests in `crates/prism-bin/tests/boot_tests.rs`:
- `test_BC_2_05_012_valid_state_dir` — clean state_dir; assert boot continues past step 6
- `test_BC_2_05_012_unwriteable_dir` — chmod 444; assert exit code 4 + error message
- `test_BC_2_05_012_rocksdb_lock_held` — pre-touch LOCK; assert exit code 4 + "LOCK" message
- `test_BC_2_05_012_sentinel_persisted` — after step 6, open the `audit_buffer` CF directly
  and read the first entry; assert `event_type == "boot.audit.initialized"`

Unit tests in `crates/prism-audit/tests/`:
- `test_BC_2_05_012_sentinel_schema` — construct a sentinel record, serialize to JSON, assert
  all required fields are present and non-null

The `AuditEmitter` in `prism-audit` is the pure side of audit construction; the effectful
boot step (file I/O, RocksDB open) lives in `prism-bin`. Unit tests can mock the storage
layer using the `StorageBackend` trait's BTreeMap implementation for the sentinel write test.

## Verification Properties

No formal VP is proposed at this time. The audit durability guarantee (sentinel is written
before step 7) is verified by integration test TV-05-012-005. A future Kani proof could
verify the ordering property if `AuditEmitter` exposes a pure-core state machine, but the
current architecture mixes RocksDB I/O with emission logic, limiting formal verification scope.

VP-033 (existing — "Audit buffer: RocksDB write completes before delivery attempt") covers
the audit buffer write ordering for the CrowdStrike DTU clone path; the boot sentinel is
a related but distinct guarantee. A new VP proposal (VP-NNN) to cover the boot sentinel
persistence ordering is deferred to the S-WAVE5-PREP-01 implementer.

## Related BCs

- BC-2.22.001 — Boot Orchestration (orchestrates: this BC is one of 4 subsystem init contracts
  whose ordering and exit-code mapping are specified in BC-2.22.001)
- BC-2.06.011 — Config load (depends on: this BC requires all preceding boot steps)
- BC-2.21.001 — OrgRegistry init (depends on: this BC requires BC-2.21.001)
- BC-2.03.013 — Credential store init (depends on: this BC requires BC-2.03.013)
- BC-2.05.001 — Every MCP Tool Invocation Produces Exactly One Audit Entry (composes with:
  this BC is the prerequisite that ensures AuditEmitter is ready before BC-2.05.001 applies)
- BC-2.05.006 — Audit Entries Are Append-Only and Immutable (enforced by: the sentinel write
  uses the same append-only CF that BC-2.05.006 specifies; duplicate sentinels from crash
  recovery are acceptable because append-only semantics tolerate duplicates)
- BC-2.05.008 — Audit Entries Satisfy SOC 2 Type II and ISO 27001 Requirements (traces to:
  the SOC-2 requirement that audit be operational before any auditable event is the root
  motivation for this BC's strict exit-code-4 on failure)
- BC-2.05.011 — Audit Forwarding At-Least-Once Delivery (composes with: BC-2.05.011 handles
  forwarding; this BC handles the boot-time opening of the buffer that BC-2.05.011 reads from)

## Architecture Anchors

- `specs/architecture/decisions/ADR-022-production-runtime-wiring.md` §B step 6 (boot step spec)
- `specs/architecture/decisions/ADR-022-production-runtime-wiring.md` §A exit-code contract
- Architecture decision AD-004 (RocksDB with 17 column families; `audit_buffer` CF spec)
- `specs/architecture/module-decomposition.md` COMP-001 `prism-bin` (SS-22), COMP-011 `prism-audit` (SS-05)
- `specs/architecture/data-layer.md` (RocksDB CF catalog, WAL configuration)

## Story Anchor

S-WAVE5-PREP-01 — prism-bin: Binary Chassis, CLI, and Boot Sequence

## VP Anchors

VP-033 (partially related — audit buffer write ordering for DTU path; not a direct VP for
this boot step; see Verification Properties)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 |
| Capability Anchor Justification | CAP-007 ("Audit Logging") per capabilities.md §CAP-007 — this BC specifies the startup-time initialization of the `AuditEmitter` and `audit_buffer` CF, which is the foundational setup for the "Log every MCP tool invocation" behavior that CAP-007 defines. The SOC 2 non-negotiability ("Audit entries are append-only and include sufficient detail for SOC 2 Type II and ISO 27001 compliance") motivates this BC's hard exit-4-on-failure design. |
| L2 Invariants | DI-004 (Audit Completeness — every MCP tool invocation produces exactly one AuditEntry): this BC establishes the prerequisite that AuditEmitter is ready before any MCP traffic begins (step 9), satisfying DI-004's foundation. |
| ADR Source | ADR-022 §B step 6, §A exit-code table; AD-004 (RocksDB 17 CFs, audit_buffer) |
| Priority | P0 |
| SOC 2 Note | Audit initialization is non-optional. Exit code 4 on failure is a hard failure by design — there is no degraded mode that skips audit. This is a SOC 2 Type II control requirement. |
| POL-12 Note | The production code path satisfying this BC MUST contain no `todo!()`, `unimplemented!()`, or `panic!("stub...")` before S-WAVE5-PREP-01 transitions to `merged`. |

## Open Questions

**OQ-2 (boot.audit.initialized sentinel schema confirmation):** The sentinel JSON schema
defined in this BC specifies a minimum required field set. The implementer of S-WAVE5-PREP-01
should confirm whether `prism-audit`'s `AuditEntry` type already covers these fields or
whether a dedicated `BootSentinelEntry` struct is needed. If `AuditEntry` does not include
`prism_version` and `boot_step`, a thin extension struct or additional fields must be added
before the TV-05-012-006 schema compliance test can pass.

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | bundle-B-phase-B-1b-ss22-bcs-2026-05-08 | 2026-05-08 | product-owner | Initial authorship — Bundle B Phase B-1b SS-22 boot-sequence BCs |
| 1.0 | redirect-option-d-2026-05-08 | 2026-05-08 | product-owner | Relocated from BC-2.22.004 (SS-22) to BC-2.05.012 (SS-05 Audit Trail) per Option (d) decomposition. Capability anchor updated CAP-034 → CAP-007. EC/TV IDs renumbered to EC-05-012-NNN / TV-05-012-NNN. OQ-2 preserved (sentinel schema confirmation). |
