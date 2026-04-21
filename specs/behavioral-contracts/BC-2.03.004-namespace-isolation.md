---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-03"
capability: "CAP-004"
lifecycle_status: active
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "572c2a9"
traces_to: ["CAP-004"]
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

# BC-2.03.004: Credential Namespace Isolation by (client_id, sensor_id, credential_name)

## Description

Credentials are uniquely identified by the 3-tuple `(client_id, sensor_id, credential_name)`. The `TenantId` type parameter enforces at compile time that credential operations cannot cross client boundaries — accessing Client A's credentials while in Client B's context is a type error, not a runtime check. Each fan-out leg in cross-client queries independently resolves its own client's credentials, with no cross-tenant access path.

## Preconditions
- A credential operation (get, set, delete, list) is invoked

## Postconditions
- The credential is uniquely identified by the 3-tuple `(client_id, sensor_id, credential_name)`
- Two clients with the same sensor type have completely independent credential storage
- The namespace prevents Client A's CrowdStrike `client_secret` from being accessible when operating as Client B

## Invariants
- DI-002: Credential isolation per client -- accessing Client A's credentials while operating in Client B's context is a type error
- The `TenantId` type parameter enforces isolation at compile time

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Compile error | Code attempts credential access without providing `TenantId` | Rust type system rejects the call |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-03-009 | Same `credential_name` used across different sensors for the same client | Independent storage; `sensor_id` differentiates them |
| EC-03-010 | Client ID contains valid but unusual characters (e.g., `client-with-dashes`) | Valid per `[a-zA-Z0-9_-]` pattern; stored correctly in both keyring and file backends |
| EC-03-011 | Cross-client credential query during `client_id: null` fan-out | Each fan-out leg resolves credentials for its own `client_id`; no cross-tenant credential access |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.03.004-001 | Same credential_name for client A and client B, same sensor | Two independent entries; Client A retrieval returns Client A's value only |
| TV-BC-2.03.004-002 | Cross-client fan-out (client_id: null) | Each fan-out leg uses its own TenantId; no cross-tenant credential access |
| TV-BC-2.03.004-003 | Client ID with dashes (`client-with-dashes`) | Valid; stored and retrieved with correct namespace |
| TV-BC-2.03.004-004 | Compile-time: call credential_get without TenantId | Rust type error; does not compile |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| VP-011 | Credential name sanitization: rejects path traversal (kani) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 |
| L2 Invariants | DI-002 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
