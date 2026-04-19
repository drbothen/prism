---
document_type: verification-property-index
level: L4
version: "1.3"
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

## Summary

| Method | Count | P0 | P1 |
|--------|-------|----|----|
| Kani | 20 | 16 | 4 |
| Proptest | 11 | 9 | 2 |
| Fuzz | 6 | 5 | 1 |
| Integration test | 2 | 2 | 0 |
| **Total** | **39** | **32** | **7** |

### Phase 3-Patch Addition (2026-04-16, Burst 2.5)

**VP-039** proposed by BC-2.05.011 (Audit Forwarding At-Least-Once). Kani harness proves the per-destination forward watermark is monotonically non-decreasing across all event sequences: ACK, transient network failure, permanent destination failure, and process restart with RocksDB watermark recovery. Story anchor: S-5.10.

### Phase 3-Patch Reassignment (2026-04-16, Burst 6b)

**VP-033 and VP-036** reassigned to `prism-dtu-crowdstrike` (anchor story S-6.07):

- **VP-033** (Audit buffer RocksDB-write-before-delivery ordering): module `prism-audit` → `prism-dtu-crowdstrike`; anchor S-2.04 → S-6.07
- **VP-036** (SessionContext drop on error): module `prism-operations` → `prism-dtu-crowdstrike`; anchor S-4.04 → S-6.07

Both VPs remain `integration_test` method. VP-033 and VP-036 are integration tests that exercise the CrowdStrike behavioral clone. The test code lives in `crates/prism-dtu-crowdstrike/tests/`. The VPs verify cross-crate interaction behavior (prism-audit ordering / prism-operations SessionContext drop) but the execution vehicle is the DTU crate. Since the DTU crate (`prism-dtu-crowdstrike`, story S-6.07) provides the behavioral clone against which these tests run, S-6.07 is the authoritative anchor story.
