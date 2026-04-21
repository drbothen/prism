---
document_type: verification-property-index
level: L4
version: "1.8"
status: draft
producer: product-owner
timestamp: 2026-04-16T14:00:00
phase: 2-patch
inputs: [architecture/verification-architecture.md]
traces_to: architecture/ARCH-INDEX.md
---

# Verification Property Index: Prism

> **Context Engineering:** This index lists all verification properties with their
> status and method. Load individual VP files only when working on that specific property.

## Properties

| ID | Property | Module | Method | Priority | Status | Anchor Story |
|----|----------|--------|--------|----------|--------|--------------|
| VP-001 | TenantId rejects invalid characters | prism-core | kani | P0 | draft | S-1.01 |
| VP-002 | Capability resolution: deny-by-default | prism-core | kani | P0 | draft | S-1.03 |
| VP-003 | Capability resolution: most-specific-path wins | prism-core | kani | P0 | draft | S-1.03 |
| VP-004 | Capability resolution: deny overrides allow at same specificity | prism-core | kani | P0 | draft | S-1.03 |
| VP-005 | Case state machine: exactly 12 valid transitions | prism-core | kani | P0 | draft | S-1.02 |
| VP-006 | Case state machine: no self-transitions | prism-core | kani | P0 | draft | S-1.02 |
| VP-007 | Confirmation token expiry: expired at boundary (inclusive) | prism-security | kani | P0 | draft | S-1.09 |
| VP-008 | Confirmation token: single-use enforcement | prism-security | kani | P0 | draft | S-1.09 |
| VP-009 | Confirmation token: content hash mismatch rejects | prism-security | kani | P0 | draft | S-1.09 |
| VP-010 | Token cap: store rejects at 100 active tokens | prism-security | kani | P0 | draft | S-1.09 |
| VP-011 | Credential name sanitization: rejects path traversal | prism-core | kani | P0 | draft | S-1.02 |
| VP-012 | Alias depth: rejects composition beyond depth 3 | prism-query | kani | P0 | draft | S-3.04 |
| VP-013 | Alias cycles: detects and rejects cyclic references | prism-query | proptest | P0 | draft | S-3.04 |
| VP-014 | Query security limits: rejects oversized queries | prism-query | kani | P0 | draft | S-3.01 |
| VP-015 | Query security limits: rejects excessive nesting depth | prism-query | kani | P0 | draft | S-3.01 |
| VP-016 | OCSF normalization: output is valid protobuf | prism-ocsf | proptest | P0 | draft | S-1.04 |
| VP-017 | OCSF normalization: unmapped fields preserved | prism-ocsf | proptest | P0 | draft | S-1.05 |
| VP-018 | Detection rule validation: rejects invalid rules | prism-operations | proptest | P0 | draft | S-4.03 |
| VP-019 | Diff computation: deterministic | prism-operations | proptest | P0 | draft | S-4.02 |
| VP-020 | Feature flag: compile AND runtime must both permit | prism-security | kani | P0 | draft | S-1.08 |
| VP-021 | PrismQL parser: never panics on arbitrary input | prism-query | fuzz | P0 | draft | S-3.01 |
| VP-022 | OCSF normalizer: never panics on arbitrary input | prism-ocsf | fuzz | P0 | draft | S-1.04 |
| VP-023 | Sensor spec parser: never panics on arbitrary TOML | prism-spec-engine | fuzz | P0 | draft | S-1.11 |
| VP-024 | Injection scanner: detects known injection patterns | prism-security | proptest | P0 | draft | S-1.10 |
| VP-025 | Cache key derivation: deterministic | prism-query | kani | P1 | draft | S-3.04 |
| VP-026 | Splay computation: deterministic per (query, client) | prism-operations | kani | P1 | draft | S-4.01 |
| VP-027 | Alert dedup key: correct per match mode | prism-operations | proptest | P0 | draft | S-4.04 |
| VP-028 | Template interpolation: never panics | prism-operations | fuzz | P0 | draft | S-4.05 |
| VP-029 | Cursor cap: rejects at 200 active | prism-core | kani | P1 | draft | S-1.02 |
| VP-030 | Schedule/rule count caps: rejects beyond limits | prism-operations | kani | P1 | draft | S-4.01 |
| VP-031 | Required column enforcement: rejects unconstrained | prism-query | proptest | P0 | draft | S-3.02 |
| VP-032 | Hot reload atomicity: failed validation retains old config | prism-spec-engine | proptest | P1 | draft | S-1.12 |
| VP-033 | Audit buffer: RocksDB write completes before delivery attempt | prism-dtu-crowdstrike | integration_test | P0 | draft | S-6.07 |
| VP-034 | Encryption round-trip: encrypt then decrypt returns plaintext | prism-credentials | proptest | P0 | draft | S-1.06 |
| VP-035 | Key derivation: same inputs produce same key | prism-credentials | proptest | P1 | draft | S-1.06 |
| VP-036 | SessionContext dropped before error propagation and on panic | prism-dtu-crowdstrike | integration_test | P0 | draft | S-6.07 |
| VP-037 | Alias expansion: never panics on arbitrary alias graphs | prism-query | fuzz | P1 | draft | S-3.04 |
| VP-038 | Injection scanner: never panics on arbitrary input strings | prism-security | fuzz | P0 | draft | S-1.10 |
| VP-039 | Audit forward watermark: monotonically non-decreasing per destination across ACK, failure, and restart sequences | prism-audit | kani | P0 | draft | S-5.10 |
| VP-040 | Plugin Linker excludes all WASI namespace imports | prism-spec-engine | kani | P1 | draft | S-1.15 |
| VP-041 | Plugin memory limit boundary: at-limit succeeds, over-limit traps | prism-spec-engine | proptest | P1 | draft | S-1.15 |
| VP-042 | Plugin hot reload: failed compile retains old InstancePre | prism-spec-engine | proptest | P1 | draft | S-1.15 |
| VP-043 | WIT validation rejects component missing required exports | prism-spec-engine | proptest | P1 | draft | S-1.15 |
| VP-044 | Action retry state machine: bounded by 5 attempts, dead-letter terminal | prism-operations | kani | P0 | draft | S-4.08 |
| VP-045 | Schedule semaphore: try_acquire used (non-blocking), never acquire | prism-operations | proptest | P0 | draft | S-4.08 |
| VP-046 | Action inline credential rejected at load time; value not in error message | prism-operations | proptest | P0 | draft | S-4.08 |
| VP-047 | UUID v7 validation: non-v7 always rejected, v7 always accepted, order preserved | prism-operations | proptest | P0 | draft | S-4.08 |
| VP-048 | Infusion spec: N fields produces exactly N UDF descriptors; duplicates error | prism-spec-engine | kani | P1 | draft | S-1.14 |
| VP-049 | Infusion per-query dedup: source calls = unique value count | prism-spec-engine | proptest | P1 | draft | S-1.14 |
| VP-050 | MCP sensor resource response redacts credentials and full API URLs | prism-mcp | proptest | P0 | draft | S-5.03 |
| VP-051 | Case state machine: exhaustive 5×5 transition table — 12 accept, 13 reject | prism-core | kani | P0 | draft | S-1.02 |
| VP-052 | update_case: disposition applied before status transition in single-call update | prism-core | proptest | P0 | draft | S-4.06 |
| VP-053 | Resolved case always has non-null disposition; transition rejects without disposition | prism-core | kani | P0 | draft | S-4.06 |
| VP-054 | TTR uses first resolution timestamp across reopen cycles; null aggregate when no resolved cases | prism-core | proptest | P1 | draft | S-4.06 |
| VP-055 | StorageEngine put_batch atomicity and domain isolation (MockStorageEngine) | prism-persistence | proptest | P1 | draft | S-1.02 |
| VP-056 | Audit buffer overflow purge: oldest entries deleted, newest preserved, purge-event produced | prism-audit | proptest | P1 | draft | S-5.10 |
| VP-057 | Crash recovery: denylist triggered at consecutive_crashes >= 3; exact threshold | prism-persistence | kani | P0 | draft | S-1.02 |
| VP-058 | Watchdog memory grace period: single check does not terminate; two consecutive checks do | prism-persistence | proptest | P0 | draft | S-2.02 |
| VP-059 | Spec validator: all errors collected (no fail-fast); warning-only specs return Ok | prism-spec-engine | proptest | P1 | draft | S-1.11 |
| VP-060 | Dedup decision: Link(c.id) iff existing case within window; Create otherwise | prism-operations | proptest | P0 | draft | S-4.06 |

## Summary

| Method | Count | P0 | P1 |
|--------|-------|----|----|
| Kani | 26 | 20 | 6 |
| Proptest | 26 | 16 | 10 |
| Fuzz | 6 | 5 | 1 |
| Integration test | 2 | 2 | 0 |
| **Total** | **60** | **43** | **17** |

### Phase 3-Patch Addition (2026-04-16, Burst 2.5)

**VP-039** proposed by BC-2.05.011 (Audit Forwarding At-Least-Once). Kani harness proves the per-destination forward watermark is monotonically non-decreasing across all event sequences: ACK, transient network failure, permanent destination failure, and process restart with RocksDB watermark recovery. Story anchor: S-5.10.

### Phase 3-Patch Reassignment (2026-04-16, Burst 6b)

**VP-033 and VP-036** reassigned to `prism-dtu-crowdstrike` (anchor story S-6.07):

- **VP-033** (Audit buffer RocksDB-write-before-delivery ordering): module `prism-audit` → `prism-dtu-crowdstrike`; anchor S-2.04 → S-6.07
- **VP-036** (SessionContext drop on error): module `prism-operations` → `prism-dtu-crowdstrike`; anchor S-4.04 → S-6.07

Both VPs remain `integration_test` method. VP-033 and VP-036 are integration tests that exercise the CrowdStrike behavioral clone. The test code lives in `crates/prism-dtu-crowdstrike/tests/`. The VPs verify cross-crate interaction behavior (prism-audit ordering / prism-operations SessionContext drop) but the execution vehicle is the DTU crate. Since the DTU crate (`prism-dtu-crowdstrike`, story S-6.07) provides the behavioral clone against which these tests run, S-6.07 is the authoritative anchor story.

### VP-029 Anchor Justification (2026-04-19, P3P41-A-OBS-001 — updated to Option B)

**VP-029** (Cursor cap: rejects at 200 active) is anchored to S-1.02 and module `prism-core`. The cap invariant has **joint ownership** across two subsystems:

- **Enforcement vehicle:** S-1.02 / `prism-core` — The 200-cursor cap is enforced at the `CursorRegistry::allocate()` boundary inside `crates/prism-core/src/cursor.rs`. The `CursorId` newtype and `CursorRegistry` struct are foundational prism-core entities; the invariant (reject when `active.len() >= 200`) is a type-level allocation boundary. S-1.02 delivers `CursorId`, `CursorRegistry`, and the VP-029 Kani proof at `crates/prism-core/src/proofs/cursor.rs`.

- **Policy owner:** SS-07 (Adapter Pagination & Response Cache, owned by `prism-query`) — SS-07 owns the semantic cap requirement: concurrent pagination must be bounded to 200 active cursors to enforce memory safety and prevent unbounded allocation across all pagination consumers. SS-07 calls `allocate()` and `release()` to drive pagination semantics; the cap value itself is SS-07's policy, enforced at the allocation site in prism-core.

S-1.02 frontmatter has been updated to `subsystems: [SS-03, SS-07, SS-11, SS-12, SS-14]`, making the cross-subsystem contribution explicit. SS-07 is named because S-1.02's `CursorRegistry` directly enforces SS-07's cap policy — not merely because SS-07 consumes the type.

**Conclusion (Option B):** VP-029 anchor to S-1.02/prism-core is correct as the enforcement vehicle. SS-07 is additionally named in S-1.02's subsystem list as the cap policy owner. Joint ownership is now explicit in both artifacts. Supersedes Option C justification-only resolution from v1.4. Closes P3P41-A-OBS-001.
