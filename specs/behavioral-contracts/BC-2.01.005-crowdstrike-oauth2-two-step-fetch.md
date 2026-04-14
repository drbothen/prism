---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Sensor Query Pipeline"
capability: "CAP-001"
---

# BC-2.01.005: CrowdStrike OAuth2 Authentication and Two-Step Fetch

## Preconditions
- CrowdStrike sensor is configured with `client_id` and `client_secret` OAuth2 credentials
- The CrowdStrike API base URL corresponds to the correct region (us-1, us-2, eu-1, ap-1)

## Postconditions
- OAuth2 token is obtained via client credentials grant before any API call
- Alert retrieval follows the two-step pattern: QueryV2 returns alert IDs, then PostEntities returns full alert details
- The response contains fully hydrated alert records (not just IDs)
- Token refresh happens transparently on 401 responses without caller awareness

## Invariants
- DI-012: Sealed auth trait -- CrowdStrike OAuth2 flow cannot be accidentally composed with other sensor auth mechanisms

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Sensor` | OAuth2 token request returns 401 (invalid client credentials) | `category: "authentication"`, suggestion: "Verify CrowdStrike client_id and client_secret in credential store" |
| `PrismError::Sensor` | OAuth2 token request returns 403 (insufficient API scopes) | `category: "authorization"`, suggestion: "Verify CrowdStrike API client has required scopes (alerts:read)" |
| `PrismError::Sensor` | QueryV2 succeeds but PostEntities returns 404 for some IDs | Partial result returned; missing IDs logged as warnings; cursor advances past them |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-002 | OAuth2 token expires between QueryV2 and PostEntities calls | Auth middleware detects 401, auto-refreshes token, retries PostEntities transparently |
| EC-01-007 | QueryV2 returns zero IDs (no matching alerts) | Empty result set returned; cursor still advances to the query timestamp; `has_more: false` |
| EC-01-008 | QueryV2 returns more IDs than PostEntities batch limit | IDs are batched into multiple PostEntities calls (CrowdStrike limit: 1000 per batch) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 |
| L2 Invariants | DI-012 |
| Priority | P0 |
