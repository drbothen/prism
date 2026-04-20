---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-01"
capability: "CAP-001"
lifecycle_status: active
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "365fb25"
traces_to: ["CAP-001"]
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

# BC-2.01.005: CrowdStrike OAuth2 Authentication and Two-Step Fetch

## Description

The CrowdStrike adapter authenticates using OAuth2 client credentials grant, then follows a mandatory two-step fetch pattern: a QueryV2 call returns alert IDs, and PostEntities batches (up to 1000 IDs per batch) return fully-hydrated alert records. Token refresh occurs transparently on 401 responses. This two-step pattern means each paginated page requires at least two HTTP calls, which must be accounted for in per-page latency budgets.

## Preconditions
- CrowdStrike sensor is configured with `client_id` and `client_secret` OAuth2 credentials
- The CrowdStrike API base URL corresponds to the correct region (us-1, us-2, eu-1, ap-1)

## Postconditions
- OAuth2 token is obtained via client credentials grant before any API call
- Alert retrieval follows the two-step pattern: QueryV2 returns alert IDs, then PostEntities returns full alert details — this means each page requires 2+ HTTP calls (one QueryV2 + one or more PostEntities batches), which must be accounted for in per-page latency budgets (see NFR-001)
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

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.01.005-001 | Valid OAuth2 credentials; QueryV2 returns 50 IDs | PostEntities fetches all 50 in one batch; 50 hydrated alerts returned |
| TV-BC-2.01.005-002 | QueryV2 returns 0 IDs | Empty result; cursor advances; `has_more: false` |
| TV-BC-2.01.005-003 | OAuth2 401 on token request | `PrismError::Sensor` with `category: "authentication"` and credential store suggestion |
| TV-BC-2.01.005-004 | OAuth2 token expires mid-fetch (401 on PostEntities) | Token refreshed transparently; PostEntities retried; caller unaware of refresh |
| TV-BC-2.01.005-005 | QueryV2 returns 1500 IDs (exceeds 1000 batch limit) | Two PostEntities calls (1000 + 500); all 1500 records returned |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| (none) | No VP directly verifies this BC — see VP-INDEX.md for full map |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 |
| L2 Invariants | DI-012 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Changes |
|---------|-------|------|--------|---------|
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
