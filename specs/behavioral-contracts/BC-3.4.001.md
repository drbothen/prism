---
document_type: behavioral-contract
level: L3
version: "0.3"
status: PROPOSED
producer: product-owner
timestamp: 2026-04-27T00:00:00
phase: 3.A
inputs:
  - ".factory/specs/architecture/decisions/ADR-009-multi-tenant-data-generator.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "802850d"
traces_to: ["CAP-009"]
origin: greenfield
extracted_from: null
subsystem: "SS-06"
capability: "CAP-039"
lifecycle_status: active
introduced: wave-3
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
bc_id: BC-3.4.001
title: Generator determinism — identical inputs produce byte-identical FixtureSet
wave: 3
phase: 3.A
date: 2026-04-27
authors: [product-owner]
related_decisions: [D-043, D-045]
related_adrs: [ADR-009]
inherits_from: null
superseded_by: null
---

# BC-3.4.001: Generator Determinism — Identical Inputs Produce Byte-Identical FixtureSet

## Description

The `generate(org_id, sensor_type, archetype, GenOpts { seed, scale, time_anchor, overrides })` function is a pure function with no I/O, no global state, and no non-deterministic entropy sources. Calling it twice on the same binary with identical arguments produces byte-identical `FixtureSet::records`. This determinism is guaranteed by a seeded `ChaCha20Rng` initialized as `ChaCha20Rng::seed_from_u64(seed ^ org_id_hash)` where `org_id_hash` is the first 8 bytes of the `OrgId` UUID interpreted as little-endian `u64`, ensuring org-namespace separation without requiring separate seed management per org.

## Preconditions

1. The `generate` function is called from within the same compiled binary (determinism is per-binary, not cross-toolchain).
2. `GenOpts::seed` is a `u64` value provided by the caller (default `42`).
3. `GenOpts::scale` is a positive finite `f64` (validated by BC-3.3.001 if sourced from customer config).
4. `GenOpts::time_anchor` is a `DateTime<Utc>` value; default is `DateTime::UNIX_EPOCH` for tests.
5. `GenOpts::overrides` is a `serde_json::Value` (default `Null` = no overrides).
6. `org_id` is a valid `OrgId` (UUID v7).
7. No `rand::thread_rng()` or timestamp-seeded entropy is used anywhere in the generator call stack.

## Postconditions

1. `generate(org_id, sensor, archetype, opts)` called twice returns `FixtureSet` values where `records` are byte-identical (JSON serialization of the `Vec<serde_json::Value>` is identical).
2. `generate` with the same `(org_id, seed, archetype, scale)` but different `time_anchor` MAY produce different timestamp field values — timestamps are anchored to `time_anchor`; this is expected and not a violation of the determinism contract.
3. `generate` with different `seed` values MUST produce different `records` (distinct RNG streams).
4. `generate` with different `org_id` values and the same `seed` MUST produce different `records` (XOR-based org-namespace separation).
5. `generate` called twice within the same process (without restart) returns identical results.
6. `generate` called in two separate process invocations with identical arguments returns identical results (no process-level global state leaks into the RNG).
7. If `overrides` is non-null, the JSON Merge Patch (RFC 7396) is applied after generation; the patched result is what becomes `FixtureSet::records`. The same patch applied to the same base generation always produces the same patched output.

## Invariants

1. The generator MUST NOT call `rand::thread_rng()`, `SystemTime::now()`, or any other non-deterministic entropy source (ADR-009 §3.2 threat mitigation).
2. The RNG initialization formula is exactly `ChaCha20Rng::seed_from_u64(seed ^ org_id_hash)` where `org_id_hash = u64::from_le_bytes(org_id.as_bytes()[0..8])` (ADR-009 §2.4).
3. `FixtureSet::provenance` is excluded from the byte-identity comparison (it is metadata, not canonical output).
4. The `#[cfg(any(test, feature = "dtu"))]` gate ensures the generator never links into production binaries.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-3.4.001-01 | Same call, same binary, sequential invocations | `records` are byte-identical |
| EC-3.4.001-02 | Same call, same binary, parallel invocations (two threads) | Each invocation independently produces byte-identical `records`; no shared mutable state between calls |
| EC-3.4.001-03 | `seed = 0` | Valid; produces a deterministic stream from XOR with `org_id_hash` |
| EC-3.4.001-04 | `seed = u64::MAX` | Valid; no overflow or panic |
| EC-3.4.001-05 | `org_id` where first 8 UUID bytes are all zeros | `org_id_hash = 0`; `seed ^ 0 = seed`; still deterministic, but same as if no XOR were applied — org-differentiation requires non-zero org bytes (noted as degenerate case) |
| EC-3.4.001-06 | `overrides = {"alerts": []}` (non-null patch) | Post-patch result is deterministic: same base generation + same patch = same output |
| EC-3.4.001-07 | `scale = 0.1` (minimal) | Produces fewer records than default but still deterministically |
| EC-3.4.001-08 | `scale = 10.0` (stress) | Produces more records; deterministic at any scale |

## Canonical Test Vectors

| TV-ID | Input | Expected Output | Category |
|-------|-------|-----------------|----------|
| TV-3.4.001-01 | `generate(orgA, claroty, HealthyOtEnvironment, GenOpts::default())` called twice | Both calls return identical `records` (byte-for-byte JSON match) | happy-path |
| TV-3.4.001-02 | `generate(orgA, claroty, HealthyOtEnvironment, opts{seed=1})` vs `generate(orgA, claroty, HealthyOtEnvironment, opts{seed=2})` | Records are different (different seed → different RNG stream) | edge-case |
| TV-3.4.001-03 | `generate(orgA, claroty, HealthyOtEnvironment, GenOpts::default())` vs `generate(orgB, claroty, HealthyOtEnvironment, GenOpts::default())` | Records are different (different org_id XOR → different RNG stream) | edge-case |
| TV-3.4.001-04 | `generate(orgA, claroty, HealthyOtEnvironment, GenOpts::default())` called in two separate process invocations | Both invocations return identical `records` (no process-level non-determinism) | happy-path |
| TV-3.4.001-05 | `generate` with `overrides = {"device_count": 5}` called twice | Both patched results are byte-identical | happy-path |
| TV-3.4.001-06 | `generate(orgA, crowdstrike, CompromisedEndpoint, GenOpts::default())` called twice | Identical `records` for a different sensor type and archetype | happy-path |
| TV-3.4.001-07 | `generate` with `seed = u64::MAX` called twice | No panic; results are identical | edge-case |

## Verification Properties

| VP | Property | Proof Method |
|----|----------|--------------|
| VP-3.4.001-A | `∀ inputs: generate(inputs) == generate(inputs)` (idempotent) | kani model check / proptest |
| VP-3.4.001-B | `∀ seed1 ≠ seed2: generate(..., seed1) ≠ generate(..., seed2)` with overwhelming probability | proptest (statistical, not formal) |
| VP-3.4.001-C | `∀ org1 ≠ org2: generate(org1, ..., seed) ≠ generate(org2, ..., seed)` with overwhelming probability | proptest |
| VP-3.4.001-D | No call to `thread_rng` or `SystemTime::now` anywhere in generator call stack | static analysis / `grep`-based CI check |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-039 ("Multi-Tenant Fixture Generation") per capabilities.md §CAP-039 |
| Capability Anchor Justification | CAP-039 ("Multi-Tenant Fixture Generation") per capabilities.md §CAP-039 — this BC specifies the determinism guarantee that CAP-039 defines as a core property: "Generator is a pure function seeded by `ChaCha20Rng::seed_from_u64(seed ^ org_id_hash)`; no global entropy, no timestamp-seeded RNG, no I/O." Determinism is not a config loading property (CAP-009); it is a generator behavioral property (CAP-039). |
| L2 Domain Invariants | N/A (Wave 3 new capability; DI-NNN assignment pending domain-spec Wave 3 extension) |
| Architecture Module | SS-06 (Client Configuration) per ARCH-INDEX.md; generator module lives in `crates/prism-dtu-common/src/generator/` |
| Stories | S-TBD (Phase 3.A implementation) |

## Related BCs

- BC-3.4.002 — depends on (schema conformance assumes generator produces well-formed records)
- BC-3.4.003 — depends on (archetype behavioral coverage assumes deterministic baseline counts)
- BC-3.4.004 — depends on (org-tagged IDs require deterministic ID derivation from org + seed)

## Architecture Anchors

- ADR-009 §2.4 — Determinism Contract; XOR-seed construction formula
- ADR-009 §2.3 — Generator API: `generate()` signature, `GenOpts`, `FixtureSet`
- ADR-009 §3.2 — Threat: non-determinism contamination; mitigation via `seeded_rng` convention
- `crates/prism-dtu-common/src/seed.rs:9` — `seeded_rng(seed: u64) -> ChaCha20Rng`; generator extends this convention

## Story Anchor

S-TBD (Phase 3.A implementation)

## VP Anchors

- VP-3.4.001-A — kani/proptest: generate is idempotent
- VP-3.4.001-B — proptest: distinct seeds produce distinct records
- VP-3.4.001-C — proptest: distinct org_ids produce distinct records
- VP-3.4.001-D — CI static check: no thread_rng in generator module

## Open Questions

None. All open questions resolved.

- Archetype catalog code placement and feature gate: **Resolved via D-056** — Generator lives in `crates/prism-dtu-common/src/generator/` behind `#[cfg(feature = "fixture-gen")]`; no separate `prism-dtu-fixture-gen` crate for Wave 3.

## BC Changelog

| Version | Change |
|---------|--------|
| v0.3 | C-5 re-anchoring (2026-04-27): capability CAP-009 → CAP-039; Capability Anchor Justification updated to cite CAP-039 ("Multi-Tenant Fixture Generation") verbatim. Open Questions marked resolved. |
| v0.2 | Initial authoring from ADR-009. |
