---
document_type: behavioral-contract
level: L3
version: "2.2"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-01"
capability: "CAP-002"
lifecycle_status: active
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "7f46c63b987acec8c89bfe1dbe3ae382"
traces_to: ["CAP-002"]
extracted_from: ".factory/specs/prd.md"
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.01.002: Cross-Client Fan-Out — Query Engine Orchestrates Parallel Sensor Fetches

**Note:** This file replaces BC-2.01.002 v1.0 "Cross-Client Fan-Out Query Aggregates Results with Per-Client Attribution". The behavior is fundamentally the same but is now invoked by the query engine (subsystem 11) rather than by a per-sensor MCP tool. The MCP-facing interface is `query(clients: null, ...)` or `query(clients: ["acme", "globex"], ...)`.

## Description

When the `query` tool is invoked with `clients: null` (all clients) or `clients: ["a", "b", ...]` (multiple clients), the query engine fans out sensor API fetches to all matching `(client_id, sensor_id, source_id)` tuples in parallel. Results from all clients are OCSF-normalized and materialized into a unified Arrow RecordBatch, with each row attributed to its source client via the `client_id` virtual field. Partial failures from individual clients are surfaced in the `sensor_errors` array rather than aborting the whole query.

## Preconditions
- The query engine (BC-2.11.001) receives a query with `clients: null` (all clients) or `clients: ["a", "b", ...]` (multiple clients)
- At least one client is configured with sensors matching the query's `sensors`/`sources` scope
- The query engine has planned the sensor fetch (BC-2.11.007 push-down)

## Postconditions
- The query engine fans out sensor API fetches to all matching `(client_id, sensor_id, source_id)` tuples in parallel
- Each fetch uses the client's credentials (BC-2.01.005-008) and respects the adapter's pagination and retry logic (BC-2.01.014)
- Results from all clients are collected, OCSF-normalized (subsystem 02), and materialized into a single Arrow RecordBatch (BC-2.11.005)
- Each materialized row includes the `client_id` virtual field (BC-2.11.012) identifying its source client
- The query engine applies the PrismQL query across the unified materialized table -- cross-client correlation is a natural consequence of materialization
- Partial failures (some clients succeed, others fail) are reported in the `sensor_errors` array of the query response
- The `query_context.clients_queried` field lists all clients that were actually queried

## Invariants
- DI-008: Per-result `client_id` attribution is never absent or incorrect (enforced via virtual field injection at materialization)
- DI-004: Exactly one AuditEntry emitted for the query invocation, recording the full client list

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Config` | No clients are configured for any sensor matching the query scope | Structured error: "No clients configured for the requested sensor/source scope" |
| Partial failure | Some clients fail (expired credentials, sensor timeout) | Successful client results are returned; failed clients appear in `sensor_errors` with error category and suggestion |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-003 | 3 of 5 clients configured but 1 has expired credentials | Return results from 2 successful clients; `sensor_errors` lists the failed client |
| DEC-005 | Cross-client query but Client B only has Armis (query targets CrowdStrike) | Client B silently excluded -- no `(client_b, crowdstrike, *)` tuple exists to fan out to |
| EC-01-002 | All clients fail (e.g., all credentials expired) | Return empty `events` array with all clients listed in `sensor_errors`; this is not a tool-level error |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.01.002-001 | `query(clients: ["acme", "globex"], sensors: ["crowdstrike"])` — both clients healthy | Unified RecordBatch with rows attributed to `acme` and `globex`; `sensor_errors: []` |
| TV-BC-2.01.002-002 | `query(clients: null)` with 3 configured clients | Fan-out to all 3; `query_context.clients_queried` lists all 3 |
| TV-BC-2.01.002-003 | `query(clients: ["acme", "globex"])` — `globex` credentials expired | Results from `acme`; `sensor_errors` lists `globex` with `category: "authentication"` |
| TV-BC-2.01.002-004 | All clients fail authentication | `events: []`; all clients in `sensor_errors`; not a tool-level error |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| (none) | No VP directly verifies this BC — see VP-INDEX.md for full map |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-002 |
| L2 Invariants | DI-004, DI-008 |
| Replaces | BC-2.01.002 v1.0 (MCP tool-level cross-client fan-out) |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Changes |
|---------|-------|------|--------|---------|
| 2.0 | cycle-1 | 2026-04-14 | product-owner | Rewrite: cross-client fan-out now orchestrated by query engine, not per-sensor MCP tool. |
| 2.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 2.2 | pass-61-fix | 2026-04-20 | product-owner | Renumbered duplicate pre-build-sweep Changelog row for version monotonicity (MED-001 BC scope extension). |
