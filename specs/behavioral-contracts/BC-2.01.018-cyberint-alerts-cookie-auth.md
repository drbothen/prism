---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-07-22T00:00:00Z
phase: 1a
origin: greenfield
subsystem: "SS-01"
capability: "CAP-001"
lifecycle_status: draft
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "fc9d874"
traces_to: ["CAP-001"]
extracted_from: ".factory/specs/prd.md"
scheduled_amendment_in: null
amendment_lifecycle: null
introduced: "2026-07-22"
modified: "2026-07-22"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.01.018: Cyberint Alerts Cookie-Based Authentication and Multi-Format Timestamp Parsing

## Description

> **Origin — ADR-053 D3 (Wave-A spec evolution burst 3):** This BC was created when the
> previously monolithic Cyberint BC (BC-2.01.006) was split into two surface-specific
> contracts. The Cyberint platform exposes two logically separate API servers: the Alerts
> surface (`/alert` endpoint, `cyberint-alerts.sensor.toml`, covered by **this BC**) and
> the Assets surface (`/asset-configuration` endpoint, `cyberint-assets.sensor.toml`,
> covered by **BC-2.01.006**). Both surfaces share the same `cookie_roundtrip` auth
> mechanism and `[[credential_refs]] name = "access_token"`, but have distinct base URLs
> and data schemas. The `header_scheme = "cookie:access_token"` TOML field (per ADR-053
> D2) governs header injection for both surfaces. POL-36: both Cyberint specs are general
> mechanism examples, not hardcoded sensor-specific behaviour.

The Cyberint Alerts sensor authenticates via an `access_token` cookie declared in the
TOML spec under `auth_type = "cookie_roundtrip"` with `header_scheme = "cookie:access_token"`.
This BC covers the Alerts surface only: the `/alert` API server that provides alert
detection and threat intelligence data. Because Cyberint alert responses use inconsistent
timestamp formats across API versions, the spec-driven adapter employs a 4-format
CyberintTime parser (ISO 8601, RFC 3339, Unix epoch seconds, Cyberint custom format)
and maintains a `(Timestamp, AlertID)` 2-tuple cursor. Timestamps that cannot be parsed
through any format fall back to the fetch timestamp, with the raw string preserved in
`raw_extensions`. The Assets surface is a separate server with a distinct sensor spec
and is covered by BC-2.01.006.

## Preconditions
- Cyberint Alerts sensor spec (`cyberint-alerts.sensor.toml`) is configured with
  `auth_type = "cookie_roundtrip"`, `header_scheme = "cookie:access_token"`, and a
  `[[credential_refs]]` block with `name = "access_token"` that resolves to a valid
  Cyberint API access token
- The resolved `access_token` credential is available via the per-client credential store
  (BC-2.06.003 four-tier resolution chain)

## Postconditions
- All Cyberint Alerts API requests include the `Cookie: access_token={token}` header,
  injected via `StaticCookieAuthProvider` per `header_scheme = "cookie:access_token"`
- The `Authorization` header is NOT set for `cookie_roundtrip` sensors
- Timestamps in Cyberint Alerts responses are parsed using the CyberintTime 4-format
  parser (ISO 8601, RFC 3339, Unix epoch seconds, Cyberint custom format)
- Cursor is a `(Timestamp, AlertID)` 2-tuple extracted from each alert record

## Invariants
- DI-012 (6-value canonical auth_type set per ADR-054 D1): `cookie_roundtrip` is one of
  the six valid auth_type values. Cross-sensor auth composition is prevented at spec-load
  time by `SpecLoader::validate_cross_composition()` per BC-2.01.016 §Invariants.

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Sensor` | Cookie auth rejected (HTTP 401 or 403) | `category: "authentication"`, suggestion: "Verify Cyberint access_token in credential store; token may have expired" |
| `PrismError::Sensor` | Cyberint Alerts API returns HTTP 429 (rate limited) | Backoff with exponential retry (2s base, 30s max); if exhausted, return partial results with `truncation_reason: "rate_limited"` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-015 | Timestamp in unexpected 5th format not covered by CyberintTime parser | Raw string preserved in `raw_extensions`; OCSF `time` field set to fetch timestamp as fallback; warning logged |
| EC-018-001 | Alert ID field absent from Cyberint Alerts response record | Cursor falls back to timestamp-only with warning; record is still emitted with available fields |
| EC-018-002 | Cyberint Alerts and Cyberint Assets use same `access_token` credential name but different sensor_ids | Both sensors resolve independently via four-tier resolution chain with their respective sensor_id keys; no credential collision |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.01.018-001 | Valid access_token cookie; standard ISO 8601 timestamp in Cyberint alert response | Alert record parsed; `(Timestamp, AlertID)` cursor set; `Cookie: access_token={token}` header present in request |
| TV-BC-2.01.018-002 | Timestamp in Cyberint custom format (4th format) | CyberintTime parser succeeds on 4th attempt; timestamp correctly extracted |
| TV-BC-2.01.018-003 | Timestamp in unknown 5th format (DEC-015) | Parse fails; fallback to fetch timestamp; raw string in `raw_extensions`; warning logged |
| TV-BC-2.01.018-004 | HTTP 401 cookie rejection | `PrismError::Sensor` with `category: "authentication"` and token refresh suggestion |
| TV-BC-2.01.018-005 | HTTP 429 rate limit | Exponential backoff; partial results with `truncation_reason: "rate_limited"` if retries exhausted |
| TV-BC-2.01.018-006 | Cyberint Alerts and Cyberint Assets queries in same session (shared access_token name) | Both sensors resolve independently; distinct Cookie headers with sensor-specific credential lookups; no cross-contamination |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| (none) | No VP directly verifies this BC — see VP-INDEX.md for full map |

## Related BCs
- BC-2.01.006: Cyberint Assets cookie auth (sibling BC; same auth mechanism, different surface and cursor key)
- BC-2.01.016: SensorAuth Open Trait — Plugin-Implementable Auth Contract (No Sealed Marker) (composes with — StaticCookieAuthProvider implements the AuthProvider trait (the runtime SensorAuth replacement governed by BC-2.01.016))
- BC-2.01.017: StaticCookieAuthProvider dispatch table (composes with — header_scheme = "cookie:<name>" dispatch)
- BC-2.06.003: Credential reference resolution (depends on — access_token credential ref resolution)

## Architecture Anchors
- ADR-053 D3: Cyberint dual-surface split rationale
- ADR-053 D2: header_scheme = "cookie:<name>" mechanism
- ADR-054 D1: 6-value canonical auth_type set

## Story Anchor
(pending — Wave-A story decomposition, Task #8)

**Interim draft rationale (F-WASE-P3-LOW-001):** This BC was created by ADR-053 D3 (dual-surface
split) as a spec-before-code artifact. The contracted behaviors — `cyberint-alerts.sensor.toml`,
`SensorSpec::header_scheme = "cookie:access_token"`, credential ref renamed `api_key` →
`access_token`, `(Timestamp, AlertID)` 2-tuple cursor, and 4-format timestamp parser — are NOT
yet shipped. Evidence: only `cyberint.sensor.toml` (monolithic, sensor_id = "cyberint") exists;
`SensorSpec::header_scheme` field absent from spec-engine source; current TOML uses `name =
"api_key"` and `cursor_token`/`$.next_cursor` pagination; spec-engine `timestamp_formats`
supports only `iso8601`, `unix_epoch_seconds`, `unix_epoch_millis` (3 values, no Cyberint custom
format). BC status set to `draft` per POL-14. Will promote to `active` when the Wave-A Cyberint
Alerts remediation story merges (POL-14 auto-promotion at merge).

## VP Anchors
(filled after VP creation — see VP-INDEX.md for current coverage)

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 |
| Capability Anchor Justification | CAP-001 ("Sensor Adapter Layer (Internal)") per capabilities.md §CAP-001. This BC specifies the authentication mechanism and timestamp-parsing behavior for the Cyberint Alerts sensor adapter — exactly what CAP-001 defines for the sensor adapter layer (auth, data fetch, cursor management for sensor APIs). The Alerts surface was split from BC-2.01.006 (Assets surface) per ADR-053 D3 to give each server its own traceable contract. |
| L2 Invariants | DI-012 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.2 | wave-a-spec-evolution-fix-burst-12 | 2026-07-22 | product-owner | F-WASE-P12-MED-002: §Related BCs line for BC-2.01.016 corrected — false claim "StaticCookieAuthProvider implements SensorAuth" removed. `StaticCookieAuthProvider` implements the `AuthProvider` trait (`crates/prism-spec-engine/src/auth_provider.rs`); `SensorAuth` lives in `prism-sensors` and `prism-spec-engine` is forbidden from importing it. Rewording follows the specified correction: "StaticCookieAuthProvider implements the AuthProvider trait (the runtime SensorAuth replacement governed by BC-2.01.016)". BC-2.01.016 label aligned to canonical H1 title "SensorAuth Open Trait — Plugin-Implementable Auth Contract (No Sealed Marker)". Sibling sweep (TD-VSDD-060): BC-2.01.006 — CLEAN (no hit). architecture/sensor-adapters.md — 2 hits but both are factually correct descriptions of the WASM plugin model (architect scope, not product-owner scope). input-hash updated at commit time. |
| 1.1 | wave-a-spec-evolution-burst-3 | 2026-07-22 | product-owner | F-WASE-P3-MED-001: `introduced` corrected from `wave-a-spec-evolution-burst-3` to `"2026-07-22"` (POL-20 pattern `^(cycle-[0-9]+\|[0-9]{4}-[0-9]{2}-[0-9]{2})$`). F-WASE-P3-MED-002: `timestamp` corrected from `2026-07-22T00:00:00` to `2026-07-22T00:00:00Z` (POL-23 Z-suffix required for new BCs). F-WASE-P3-LOW-001: status/lifecycle_status set to `draft` (was `active`). Adjudication verdict: contracted behavior NOT yet shipped — `cyberint-alerts.sensor.toml` does not exist, `SensorSpec::header_scheme` field does not exist in engine, credential ref name `access_token` not yet renamed from `api_key`, `(Timestamp, AlertID)` 2-tuple cursor not yet implemented (current spec uses `cursor_token`/`$.next_cursor`), 4-format CyberintTime parser not implemented (engine supports only `iso8601`/`unix_epoch_seconds`/`unix_epoch_millis`). Wave-A remediation stories (Task #8, pending) will implement these behaviors; BC promotes to active when anchor story merges (POL-14). §Story Anchor updated with interim rationale. |
| 1.0 | wave-a-spec-evolution-burst-3 | 2026-07-22 | product-owner | Initial contract. Created by ADR-053 D3 split of BC-2.01.006 (formerly "Cyberint Cookie-Based Authentication and Multi-Format Timestamp Parsing") into two surface-specific BCs. This BC covers the Cyberint Alerts surface (`cyberint-alerts.sensor.toml`, `/alert` server, `(Timestamp, AlertID)` cursor). BC-2.01.006 covers the Cyberint Assets surface (`cyberint-assets.sensor.toml`, `/asset-configuration` server, `(Timestamp, AssetID)` cursor). Both surfaces: auth_type = "cookie_roundtrip", header_scheme = "cookie:access_token", StaticCookieAuthProvider. DI-012 invariant: 6-value canonical auth_type set per ADR-054 D1. |
