---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-14T07:00:00
phase: 1a
origin: greenfield
subsystem: "SS-11"
capability: "CAP-015"
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
input-hash: "365fb25"
traces_to: ["CAP-015"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.11.011: Cross-Client Query Scoping

## Description

Cross-client queries fan out to all configured clients simultaneously, with each result event tagged with `client_id` for provenance (DI-008). Scope resolution follows strict intersection semantics: tool parameters define the maximum boundary, and query predicates can only narrow within that boundary — they can never widen it. A query predicate naming a client outside the tool-parameter scope produces an empty result set, not an error. Partial failures (some clients succeed, some fail) return available results plus `sensor_errors` for failures. Clients that lack a queried sensor are silently skipped.

## Preconditions
- A `query` tool call specifies scoping via tool parameters (`clients`, `sensors`) and/or query predicates (`client_id = "..."`, `sensor = "..."`)

## Postconditions
- Scope resolution follows intersection semantics:
  1. **Tool parameters** define the outer boundary: `clients: null` means all configured clients; `clients: ["acme", "globex"]` means only those two
  2. **Query predicates** can only narrow within the tool parameter scope: if `clients: ["acme", "globex"]` and query contains `client_id = "acme"`, the effective scope is `["acme"]` only
  3. **Query predicates cannot widen**: if `clients: ["acme"]` and query contains `client_id = "globex"`, the intersection is empty -- zero results, not an error
- Sensor scoping follows the same intersection logic: `sensors` tool parameter intersected with `sensor = "..."` query predicates
- For cross-client queries (`clients: null` or multiple clients):
  - Fan-out occurs to all matching (client, sensor) combinations
  - Each result event includes `client_id` as a virtual field for provenance
  - Partial failures (some clients succeed, some fail) return results from successful clients plus `sensor_errors` for failures
  - Clients that lack a queried sensor are silently skipped (not a failure), listed in metadata as `clients_skipped`
- Per-client alias handling: if the query uses an alias defined per-client, all queried clients must define it or the query fails (DEC-025)

## Invariants
- DI-008: Client data separation -- each event in cross-client results has explicit `client_id`
- Tool parameters are always the maximum scope; query predicates only narrow

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-ALIAS-001` | Per-client alias not defined for all queried clients | Error listing defined_in and missing_in clients (DEC-025) |
| `E-CFG-001` | No clients match the intersection of tool params and query predicates | Empty result set with metadata explaining the empty intersection |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-025 | Per-client alias missing for some clients | Error with client lists |
| EC-11-027 | `clients: null` with 50 configured clients | Fan-out to all 50; 10K materialization limit still applies across all clients combined |
| EC-11-028 | `clients: ["acme"]` but query has `client_id = "acme" OR client_id = "globex"` | Intersection: only `acme` (tool param limits to `acme`; `globex` from query is outside scope) |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| `query(query="severity='critical'", clients=null)` with 3 configured clients | Fan-out to all 3; each result has `client_id` | happy-path |
| `query(query="client_id='globex'", clients=["acme"])` | Empty result set; intersection is empty | edge-case |
| `query(query="high_sev", clients=null)` where `high_sev` is per-client alias not defined for client "beta" | `Err(E-ALIAS-001)` with defined_in/missing_in | error |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-001 | TenantId rejects invalid characters | kani |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 |
| L2 Invariants | DI-008 |
| L2 Edge Cases | DEC-025 |
| Priority | P0 |

## Changelog
| Version | Date | Burst | Change |
|---------|------|-------|--------|
| 1.0 | 2026-04-14 | cycle-1 | Initial contract |
| 1.1 | 2026-04-20 | pre-build-sweep | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
