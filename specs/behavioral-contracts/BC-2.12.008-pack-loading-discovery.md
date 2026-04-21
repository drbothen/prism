---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-12"
capability: "CAP-023"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "e5de7f9"
traces_to: ["CAP-023"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.12.008: Pack Loading and Discovery — Load Packs from Config, Run Discovery Queries, Conditional Execution

## Description

Packs are named collections of scheduled queries loaded from `packs.toml` at startup. Each pack may have an optional discovery query (executed at startup and cached for `pack_refresh_interval`) that gates pack activation — if the discovery query returns zero rows, the pack is inactive. Shard-based activation (`SHA256(client_id) % 100 < shard`) deterministically controls which clients execute a pack. Active pack queries are registered as scheduled queries prefixed with the pack name. Discovery results are re-evaluated periodically for dynamic activation/deactivation.

## Preconditions
- Pack definitions exist in `packs.toml` (separate from `prism.toml`), loaded at startup alongside main config
- Each pack has: `name`, `description`, optional `discovery` (PrismQL query that must return >= 1 row for the pack to be active), optional `sensor_filter` (restrict to specific sensor types), optional `shard` (0-100, percentage of clients that execute this pack)
- Each pack contains one or more named queries, each with its own `interval`, `snapshot`, `removed` settings

## Postconditions
- At startup, `packs.toml` is parsed and validated; all queries within packs pass PrismQL parsing and security limit validation
- For each pack with a `discovery` query: the discovery query is executed at startup and cached for `pack_refresh_interval` (default 3600 seconds); packs with failing discovery queries are marked inactive
- For each pack with a `shard` value: `SHA256(client_id) % 100 < shard` determines whether a client executes the pack (deterministic, consistent across restarts)
- Active pack queries are registered as scheduled queries (BC-2.12.001) with the pack name as a prefix: `{pack_name}.{query_name}`
- Discovery query results are re-evaluated every `pack_refresh_interval` seconds; packs may become active or inactive dynamically
- Queries within a pack are sorted by an optional `priority` field (lower = first); high-priority queries execute before low-priority ones within the same tick

## Invariants
- Pack queries inherit the same security limits as standalone scheduled queries (DI-019)
- Pack names must be unique; query names must be unique within a pack
- Discovery query failures are logged but do not prevent other packs from loading

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-PACK-001` | `packs.toml` parse error | Fatal startup error with line/column position |
| `E-PACK-002` | Pack contains a query that fails PrismQL parsing | Pack is rejected entirely; other packs continue loading |
| `E-PACK-003` | Discovery query exceeds security limits | Pack marked inactive with structured warning |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-12-022 | Discovery query returns 0 rows | Pack is inactive; re-checked after `pack_refresh_interval` |
| EC-12-023 | `shard: 50` with 3 clients | Deterministic: specific clients execute based on hash; may be 1 or 2 of 3 |
| EC-12-024 | Pack references sensor type not configured for any client | Pack loads successfully; all query executions produce empty results |
| EC-12-025 | `packs.toml` does not exist | No packs loaded; not an error; log info message |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| `packs.toml` with 1 pack, no discovery query | Pack loads; queries registered as scheduled queries | happy-path |
| Pack with `discovery` query returning 0 rows | Pack inactive; not registered; re-checked after interval | edge-case |
| `packs.toml` with syntax error | Fatal startup error with position | error |
| `packs.toml` does not exist | No packs; INFO log; server starts normally | edge-case |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| — | Covered by schedule cap and query security limits | — |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-023 |
| L2 Invariants | DI-019 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial contract |
