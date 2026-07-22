---
document_type: behavioral-contract
level: L3
version: "1.7"
status: active
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
input-hash: "fc9d874"
traces_to: ["CAP-001"]
extracted_from: ".factory/specs/prd.md"
scheduled_amendment_in: null
amendment_lifecycle: null
introduced: cycle-1
modified: "2026-07-22"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.01.008: Armis Token Exchange Auth with AQL Query Forwarding and Timestamp Fallback

## Description

> **Amendment — ADR-053 D2 + ADR-054 D1 (Wave-A spec evolution burst 3):** This BC's auth
> premise has been updated from `bearer_static` to `token_exchange`. The Armis Centrix API
> uses token-exchange: a long-lived `secret_key` credential is exchanged via
> `POST /api/v1/access_token/` for a short-lived access token; the access token is injected
> as a raw Authorization header with NO "Bearer" prefix (`header_scheme = "raw"`).
> Auth acquisition is handled natively by `DeclarativeHttpAuthProvider(TokenExchange)`
> (per ADR-054 D2; no WASM plugin required). Bearer-prefix injection against live Armis v1
> tenants causes HTTP 401; short-lived tokens require a refresh path via re-exchange on expiry.
> The `SensorAuth` open trait (BC-2.01.016) is the runtime interface; the 6-value canonical
> auth_type set now includes `token_exchange` (ADR-054 D1 + DI-012 amended).

> **Amendment — ADR-023 (PLUGIN-MIGRATION-001-G):** This BC previously described a
> hardcoded Rust adapter (`ArmisAuth`). That implementation was deleted in
> PLUGIN-MIGRATION-001-A (PR #156). The auth behavior described here is now delivered by the
> Armis TOML sensor spec (`.prism/specs/sensors/armis.sensor.toml`)
> with `auth_type = "token_exchange"`, `header_scheme = "raw"`, and an `[auth_acquisition]`
> block (declarative TOML; no `.prx` WASM plugin required for this auth mechanism). The
> behavioral contract itself is unchanged — preconditions, postconditions, and invariants
> describe what the system must do, not how. The `SensorAuth` open trait
> (BC-2.01.016) is the runtime interface.

The Armis Centrix sensor authenticates via token exchange declared in the TOML spec under
`auth_type = "token_exchange"`. A long-lived `secret_key` credential (declared as
`[[credential_refs]] name = "secret_key"`) is exchanged via `POST /api/v1/access_token/`
for a short-lived access token; the token is injected as a raw Authorization header
(`header_scheme = "raw"`, no "Bearer" prefix) on every subsequent API request. The spec-driven
adapter uses `DeclarativeHttpAuthProvider(TokenExchange)` for lazy token acquisition, in-memory
ArcSwap caching, and automatic refresh on expiry or 401. Because Armis records use inconsistent
timestamp and ID field names across its 7 data sources, the spec-driven adapter employs
per-source fallback chains (1-3 candidate timestamp fields, 2-4 candidate ID fields) to reliably
construct a `(Timestamp, TypeSpecificID)` cursor. Records with no valid timestamp in any fallback
field are included in results but do not advance the cursor.

## Preconditions
- Armis Centrix sensor is configured for token exchange via a `secret_key` credential declared
  in the TOML spec under `auth_type = "token_exchange"` with `[[credential_refs]] name = "secret_key"`
  and an `[auth_acquisition]` block specifying `token_path = "/api/v1/access_token/"`,
  `credential_body_field = "secret_key"`, `token_response_path = "data.access_token"`,
  `expiry_field = "data.expiration_utc"`, and `expiry_mode = "absolute_utc_string"`
- The target data source is one of the 7 Armis sources (alerts, activities, audit_logs,
  risk_factors, connections, devices, vulnerabilities)

## Postconditions
- Token acquisition: `DeclarativeHttpAuthProvider(TokenExchange)` resolves the `secret_key`
  credential at first request (lazy — zero network calls at construction per BC-2.16.014 P1),
  POSTs to `{base_url}/api/v1/access_token/` with form body `secret_key={resolved_value}`,
  and extracts the short-lived access token from `$.data.access_token`
- All Armis API requests use the raw access token via `header_scheme = "raw"`:
  `Authorization: {raw_access_token}` (NO "Bearer" prefix)
- Token is cached in-memory via ArcSwap; re-acquired before expiry (`data.expiration_utc`
  minus 30s TTL buffer) or on any HTTP 401 response
- Queries are expressed in AQL (Armis Query Language) and forwarded to the GetSearch API
- Timestamp extraction uses the per-source fallback chain (1-3 candidate fields)
- ID extraction uses the per-source fallback chain (2-4 candidate fields)
- Cursor is a `(Timestamp, TypeSpecificID)` 2-tuple

## Invariants
- DI-012 (6-value canonical auth_type set per ADR-054 D1): `token_exchange` is the 6th variant
  of the closed auth_type enum; cross-composition with other sensor auth mechanisms is prevented
  at spec-load time by `SpecLoader::validate_cross_composition()` per BC-2.01.016 §Invariants.
- DI-001: Cursor forward progress

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Sensor` | Token-exchange POST fails (HTTP 401 — invalid secret_key) | `category: "authentication"`, suggestion: "Verify Armis secret_key in credential store" |
| `PrismError::Sensor` | AQL syntax error (HTTP 400) | `category: "api_contract"`, include the AQL query and Armis error message in the structured error |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-013 | Record has no valid timestamp in any of the fallback fields | Warning logged identifying the record; record included in response but with null cursor contribution; does not advance cursor |
| EC-01-011 | All records in a page lack valid timestamps | Page treated as having no cursor advancement; pagination halts to prevent infinite loops |
| EC-01-012 | ID fallback chain exhausted (no valid ID field found) | Record logged as warning and skipped; cursor does not account for this record |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.01.008-001 | Armis alerts source with valid secret_key; token exchange succeeds; all records have primary timestamp field | Records fetched; cursor advanced with `(Timestamp, AlertID)` 2-tuple |
| TV-BC-2.01.008-002 | Record missing primary timestamp; secondary fallback field present | Secondary timestamp used; cursor correctly set; warning logged |
| TV-BC-2.01.008-003 | Record has no timestamp in any fallback field (DEC-013) | Record included; cursor not advanced for this record; warning logged |
| TV-BC-2.01.008-004 | Token-exchange POST returns HTTP 401 (invalid secret_key) | `PrismError::Sensor` with `category: "authentication"` |
| TV-BC-2.01.008-005 | AQL syntax error (HTTP 400) | `PrismError::Sensor` with `category: "api_contract"` including AQL query text |
| TV-BC-2.01.008-006 | AQL contains disallowed construct (e.g. sub-query `in:devices (select ...)`); rejected by `validate_aql()` before HTTP call | `SensorError::ConfigValidation { sensor: "armis", detail: "<rejected AQL> — reason: sub-query construct not permitted" }`; no HTTP call issued; HIGH-severity audit event emitted with `aql_hash` + 64-char preview + `validation_outcome: "reject"` |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| (none) | No VP directly verifies this BC — see VP-INDEX.md for full map |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 |
| L2 Invariants | DI-001, DI-012 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.7 | wave-a-spec-evolution-burst-3 | 2026-07-22 | product-owner | ADR-053 D2 + ADR-054 D1 amendment: auth_type bearer_static → token_exchange; credential ref bearer_token → secret_key; H1 title updated (Bearer Token Auth → Token Exchange Auth); Description Amendment Note updated (BearerStaticCredentialAuthProvider → DeclarativeHttpAuthProvider(TokenExchange) + header_scheme = "raw" + lazy acquire per BC-2.16.014 P1); Description prose updated to token exchange with ArcSwap cache; Preconditions updated to token_exchange TOML wiring + [auth_acquisition] block; Postconditions updated to DeclarativeHttpAuthProvider token acquisition flow + raw Authorization header injection; DI-012 invariant note updated to 6-value canonical auth_type set per ADR-054 D1; Error Cases auth suggestion updated (API key → secret_key); Test Vectors TV-001/004 auth language updated; modified date 2026-07-22; scheduled_amendment_in cleared (ADR-023 complete in v1.6). |
| 1.6 | PLUGIN-MIGRATION-001-G | 2026-05-27 | product-owner | ADR-023 amendment: removed PENDING AMENDMENT banner; added Amendment Note to Description; updated Description prose from deleted `ArmisAuth` Rust adapter to TOML spec `auth_type = "bearer_static"` + AQL query forwarding config declarative language; updated DI-012 invariant from sealed-trait to `SpecLoader::validate_cross_composition()` runtime enforcement per BC-2.01.016; set amendment_lifecycle to null; bumped status draft→active. Behavioral semantics (preconditions, postconditions, error cases, test vectors) unchanged. |
| 1.5 | prereq-f | 2026-05-11 | product-owner | PREREQ-F prefix note: added PENDING AMENDMENT — ADR-023 callout under H1 per ADR-023 L370 wording; added scheduled_amendment_in: ADR-023 and amendment_lifecycle: pending to frontmatter. No semantic change to BC body. Full amendment in Wave 2/G. |
| 1.4 | W2-FIX-I-PO | 2026-04-26 | product-owner | Added TV-BC-2.01.008-006: pre-wire `ConfigValidation` rejection case per ADR-005 Q3 decision. |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
