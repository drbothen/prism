---
document_type: behavioral-contract
level: L3
version: "1.9"
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
modified: "2026-07-27"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.01.006: Cyberint Assets Cookie-Based Authentication and Multi-Format Timestamp Parsing

## Description

> **Amendment — ADR-053 D3 (Wave-A spec evolution burst 3):** This BC's scope has been
> restricted to the **Cyberint Assets** surface (`/asset-configuration` server, spec file
> `cyberint-assets.sensor.toml`). The Cyberint platform exposes two logically separate API
> servers: the Alerts surface (`/alert` endpoint, `cyberint-alerts.sensor.toml`, covered by
> **BC-2.01.018**) and the Assets surface (`/asset-configuration` endpoint,
> `cyberint-assets.sensor.toml`, covered by this BC). Both surfaces share the same
> `cookie_roundtrip` auth mechanism and `[[credential_refs]] name = "access_token"`, but have
> distinct base URLs and data schemas. The `header_scheme = "cookie:access_token"` TOML field
> (per ADR-053 D2) governs header injection for both surfaces. POL-36: both Cyberint specs
> are general mechanism examples, not hardcoded sensor-specific behaviour.

> **Amendment — ADR-023 (PLUGIN-MIGRATION-001-G):** This BC previously described a
> hardcoded Rust adapter (`CyberintAuth`). That implementation was deleted in
> PLUGIN-MIGRATION-001-A (PR #156). The auth behavior described here is now delivered by the
> Cyberint Assets TOML sensor spec (`.prism/specs/sensors/cyberint-assets.sensor.toml`)
> with `auth_type = "cookie_roundtrip"` and `header_scheme = "cookie:access_token"`
> (declarative TOML; no `.prx` WASM plugin required for this auth mechanism). The
> behavioral contract itself is unchanged — preconditions, postconditions, and invariants
> describe what the system must do, not how. The `SensorAuth` open trait
> (BC-2.01.016) is the runtime interface.

The Cyberint Assets sensor authenticates via an `access_token` cookie declared in the TOML
spec under `auth_type = "cookie_roundtrip"` with `header_scheme = "cookie:access_token"`.
This BC covers the Assets surface only: the `/asset-configuration` API server that provides
asset inventory and configuration data. Because Cyberint asset responses may include timestamps in varying formats, the spec-driven
adapter employs a 3-format CyberintTime parser — same parser profile as the Alerts surface
(BC-2.01.018), grounded in the canonical Cyberint Assets OpenAPI (`cyberint_assets_openapi_06.20.2026.json`,
in-repo) which documents all date fields (`created`, `updated`, `first_seen`, `last_seen`) as
`"type": "string", "format": "date-time"` — no integer epoch fields exist in the assets API schema.
The three formats are:
(1) RFC-3339/ISO-8601 (primary — matches all documented assets API date fields), (2) Unix epoch
seconds (defensive coverage — no documented assets field emits epoch-seconds; the shared
CyberintTime parser tolerates this format as defensive coverage for any undocumented or future
numeric timestamp values), (3) Unix epoch millis (defensive coverage matching the spec-engine's
`timestamp_formats` supported set). Note: `whois_created_date` is a field from the Alerts API
(`cyberint_alerts_openapi_06.20.2026.json`), NOT the Assets API — it does not appear in the
assets schema and is not used as a reference for this BC. The Cyberint Assets API uses
page-number pagination: `GetAssetsRequest` declares `page_number` (integer, minimum 1,
server-controlled page size — no `page_size` parameter exists in the schema) and
`GetAssetsResponse` returns `total_assets`, `page_number`, and `assets`; no cursor fields
exist anywhere in the schema. Multi-page retrieval is pending GAP-ASSETS-PAG-001 (no
suitable `PaginationConfig` variant for server-controlled page size with total-count
termination); until resolved, `cyberint-assets.sensor.toml` carries no
`[tables.steps.pagination]` block (first-page-only per story S-WAVE-A-CYBERINT-SPEC-001
§T-03). Timestamps that cannot be parsed through any of the 3 formats fall back to the fetch
timestamp, with the raw string preserved in `raw_extensions`. The Alerts surface is a
separate server with a distinct sensor spec and is covered by BC-2.01.018.

## Preconditions
- Cyberint Assets sensor spec (`cyberint-assets.sensor.toml`) is configured with
  `auth_type = "cookie_roundtrip"`, `header_scheme = "cookie:access_token"`, and a
  `[[credential_refs]]` block with `name = "access_token"` that resolves to a valid
  Cyberint API access token
- The resolved `access_token` credential is available via the per-client credential store
  (BC-2.06.003 four-tier resolution chain)

## Postconditions
- All Cyberint Assets API requests include the `Cookie: access_token={token}` header,
  injected via `StaticCookieAuthProvider` per `header_scheme = "cookie:access_token"`
- The `Authorization` header is NOT set for `cookie_roundtrip` sensors
- Timestamps in Cyberint Assets responses are parsed using the CyberintTime 3-format
  parser (RFC-3339/ISO-8601 primary, Unix epoch seconds, Unix epoch millis)
- The Cyberint Assets API is page-number paginated (`GetAssetsRequest.page_number`, server-controlled page size, `GetAssetsResponse.total_assets`); no cursor fields exist in the schema. Multi-page retrieval is pending GAP-ASSETS-PAG-001 (no suitable `PaginationConfig` variant for server-controlled page size with total-count termination); until resolved, `cyberint-assets.sensor.toml` carries no `[tables.steps.pagination]` block (first-page-only per story S-WAVE-A-CYBERINT-SPEC-001 §T-03)

## Invariants
- DI-012 (6-value canonical auth_type set per ADR-054 D1): `cookie_roundtrip` is one of
  the six valid auth_type values. Cross-sensor auth composition is prevented at spec-load
  time by `SpecLoader::validate_cross_composition()` per BC-2.01.016 §Invariants.

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Sensor` | Cookie auth rejected (HTTP 401 or 403) | `category: "authentication"`, suggestion: "Verify Cyberint access_token in credential store; token may have expired" |
| `PrismError::Sensor` | Cyberint Assets API returns HTTP 429 (rate limited) | Backoff with exponential retry (2s base, 30s max); if exhausted, return partial results with `truncation_reason: "rate_limited"` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-015 | Timestamp value that cannot be parsed by any of the 3 CyberintTime formats (unparseable by CyberintTime parser) — e.g., a date in an entirely unrecognized notation | Raw string preserved in `raw_extensions`; OCSF `time` field set to fetch timestamp as fallback; warning logged |
| EC-01-009 | Customer ID derived from API URL subdomain changes | Config validation at startup detects mismatch; `customer_id` in `GetAssetsRequest` POST body resolves from the updated config automatically — no cursor state exists (first-page-only retrieval pending GAP-ASSETS-PAG-001) |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.01.006-001 | Valid access_token cookie; standard ISO 8601 timestamp in Cyberint asset response | Asset record parsed; `Cookie: access_token={token}` header present in request; first-page assets returned (page-number pagination, first-page-only per GAP-ASSETS-PAG-001; no cursor extracted from records) |
| TV-BC-2.01.006-002 | DEFENSIVE epoch path — synthetic input: `first_seen` field value as epoch-seconds integer `1705708800` (defensive test — the assets API documents `first_seen` as `"format": "date-time"` string per `cyberint_assets_openapi_06.20.2026.json`; the epoch-seconds branch of the shared CyberintTime parser is defensive tolerance for any undocumented or future numeric timestamp values; no real assets field emits epoch-seconds integers in the documented schema) | Unix-epoch-seconds branch of CyberintTime parser succeeds; OCSF `time` field set to correct RFC-3339 timestamp |
| TV-BC-2.01.006-003 | Timestamp string that fails all 3 CyberintTime formats (DEC-015) — e.g., `"Jan 01 2026 12:00"` | Parse fails on all 3 formats; OCSF `time` falls back to fetch timestamp; raw string preserved in `raw_extensions`; warning logged; record not dropped |
| TV-BC-2.01.006-004 | HTTP 401 cookie rejection | `PrismError::Sensor` with `category: "authentication"` and token refresh suggestion |
| TV-BC-2.01.006-005 | HTTP 429 rate limit | Exponential backoff; partial results with `truncation_reason: "rate_limited"` if retries exhausted |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| (none) | No VP directly verifies this BC — see VP-INDEX.md for full map |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 |
| Capability Anchor Justification | CAP-001 ("Sensor Adapter Layer (Internal)") per capabilities.md §CAP-001. This BC specifies the authentication mechanism and timestamp-parsing behavior for the Cyberint Assets sensor adapter — exactly what CAP-001 defines for the sensor adapter layer (auth, data fetch, cursor management for sensor APIs). |
| L2 Invariants | DI-012 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.9 | FB70 | 2026-07-27 | product-owner | F-WASE-P66-CRIT-001: removed all `(Timestamp, AssetID)` cursor-pagination language. Ground truth: `GetAssetsRequest` in `cyberint_assets_openapi_06.20.2026.json` declares `page_number` (integer, minimum 1) with no cursor fields; `GetAssetsResponse` carries `total_assets`, `page_number`, `assets` — no cursor. §Description cursor sentence replaced with accurate page-number pagination description including GAP-ASSETS-PAG-001 blocker (no suitable `PaginationConfig` variant for server-controlled page size with total-count termination; first-page-only per story S-WAVE-A-CYBERINT-SPEC-001 §T-03). §Postconditions cursor bullet replaced with page-number pagination and first-page-only note. EC-01-009 cursor-state-invalidation removed; replaced with `GetAssetsRequest` POST body customer_id note (no cursor state to invalidate). TV-001 `(Timestamp, AssetID)` cursor-set reference removed. TV-002 `cursor timestamp extracted` removed. POL-29 9a: BC-2.01.018 named twin swept — false cross-reference corrected in same burst. POL-29 9b: story S-WAVE-A-CYBERINT-SPEC-001 §Behavioral Contracts references BC-2.01.006 by version; BC-INDEX pin update deferred to state-manager (outside product-owner write perimeter). POL-29 9c: no new MUST statements added. |
| 1.8 | wave-a-spec-evolution-fix-burst-37 | 2026-07-24 | product-owner | F-WASE-P48-MED-002: §Description mis-grounded on `cyberint_alerts_openapi_06.20.2026.json` (the Alerts spec) and cited `whois_created_date` as the integer-epoch example — that field is alerts-only and does not appear in the assets schema. Read `cyberint_assets_openapi_06.20.2026.json` (in-repo): all date fields (`created`, `updated`, `first_seen`, `last_seen`, `created_from`) are `"type": "string", "format": "date-time"` — no integer epoch fields exist in the assets API. Fix: (a) citation changed to `cyberint_assets_openapi_06.20.2026.json`; (b) `whois_created_date` example removed and a note added that it is an alerts-only field; (c) formats (2) and (3) reframed as DEFENSIVE parser tolerance — the shared CyberintTime parser tolerates epoch-seconds/millis even though documented assets fields emit only RFC-3339/date-time strings; (d) TV-002 rewritten: `whois_created_date: 1705708800` → synthetic defensive vector using `first_seen: 1705708800` labeled as defensive-tolerance to make clear this is shared-parser coverage, not a documented assets field shape. Capability anchor: CAP-001 ("Sensor Adapter Layer (Internal)") per capabilities.md §CAP-001 — this BC specifies authentication and timestamp-parsing behavior for the Cyberint Assets sensor adapter, exactly what CAP-001 defines. POL-29 sweep: no remaining `whois_created_date`, `cyberint_alerts_openapi` reference in live BC body (changelog rows exempt). |
| 1.7 | wave-a-rmu-amendment-burst-1 | 2026-07-23 | product-owner | RU-Q5 REFUTED: parity amendment with BC-2.01.018 v1.4. Canonical Cyberint OpenAPI (`cyberint_alerts_openapi_06.20.2026.json`, in-repo) confirms no custom format exists anywhere. BC amended from 4-format to 3-format throughout: §Description (grounding citation added), §Postconditions. DEC-015 reframed: "unexpected 5th format" → "unparseable by all 3 CyberintTime formats". TV-002 rewritten: custom-format test → Unix-epoch-seconds integer test. TV-003 updated: "5th format" → "fails all 3 formats" with concrete example. |
| 1.6 | wave-a-spec-evolution-burst-3 | 2026-07-22 | product-owner | ADR-053 D3 amendment: BC scope restricted to Cyberint Assets surface only (`cyberint-assets.sensor.toml`, `/asset-configuration` server). H1 title updated (added "Assets"). Amendment note added citing ADR-053 D3 split rationale and BC-2.01.018 as the Alerts-surface sibling BC. Description prose updated to Cyberint Assets sensor with `/asset-configuration` server scope. Preconditions updated to `cyberint-assets.sensor.toml` with `header_scheme = "cookie:access_token"` per ADR-053 D2. Postconditions updated: `StaticCookieAuthProvider` + `header_scheme = "cookie:access_token"` + `(Timestamp, AssetID)` cursor. DI-012 invariant note updated to 6-value canonical auth_type set per ADR-054 D1. Capability Anchor Justification added per adversary policy 5. scheduled_amendment_in cleared (ADR-023 complete in v1.5). modified date 2026-07-22. |
| 1.5 | PLUGIN-MIGRATION-001-G | 2026-05-27 | product-owner | ADR-023 amendment: removed PENDING AMENDMENT banner; added Amendment Note to Description; updated Description prose from deleted `CyberintAuth` Rust adapter to TOML spec `auth_type = "cookie_roundtrip"` declarative language; updated DI-012 invariant from sealed-trait to `SpecLoader::validate_cross_composition()` runtime enforcement per BC-2.01.016; set amendment_lifecycle to null; bumped status draft→active. Behavioral semantics (preconditions, postconditions, error cases, test vectors) unchanged. |
| 1.4 | prereq-f | 2026-05-11 | product-owner | PREREQ-F prefix note: added PENDING AMENDMENT — ADR-023 callout under H1 per ADR-023 L370 wording; added scheduled_amendment_in: ADR-023 and amendment_lifecycle: pending to frontmatter. No semantic change to BC body. Full amendment in Wave 2/G. |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
