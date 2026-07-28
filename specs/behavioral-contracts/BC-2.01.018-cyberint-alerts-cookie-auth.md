---
document_type: behavioral-contract
level: L3
version: "1.7"
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
modified: "2026-07-27"
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
detection and threat intelligence data. Because Cyberint alert responses may include timestamps in varying formats, the spec-driven
adapter employs a 3-format CyberintTime parser — grounded in the canonical Cyberint Alerts
OpenAPI (`cyberint_alerts_openapi_06.20.2026.json`), which declares ALL alert date fields as
`"type: string, format: date-time"` (RFC-3339/ISO-8601); only `whois_created_date` (nested
enrichment) is an integer epoch field; no custom format exists anywhere in the spec:
(1) RFC-3339/ISO-8601 (primary — all API date-time fields), (2) Unix epoch seconds (for
integer epoch fields such as `whois_created_date`), (3) Unix epoch millis (defensive
coverage matching the spec-engine's supported `timestamp_formats` set of
`iso8601 / unix_epoch_seconds / unix_epoch_millis`). Timestamps that cannot be parsed through any of
the 3 formats fall back to the fetch timestamp, with the raw string preserved in
`raw_extensions`. Alert records are fetched using `PageNumber` pagination per ADR-056 §D3 —
`page` (1-based, computed as `offset + 1`) and `size` injected as POST body keys; the
spec-engine `PageNumber` variant manages page counter state with no cursor extraction from
individual records (cursor pagination superseded for the Alerts surface per ADR-056 §D7).
The Assets surface is a separate server with a distinct sensor spec and is covered by BC-2.01.006.

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
- Timestamps in Cyberint Alerts responses are parsed using the CyberintTime 3-format
  parser (RFC-3339/ISO-8601 primary, Unix epoch seconds, Unix epoch millis)
- Multi-page fetch uses `PageNumber` pagination per ADR-056 §D3: `page` (1-based, `offset + 1`) and `size` injected as POST body keys; the page counter advances after each complete page, independent of individual record field values

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
| DEC-015 | Timestamp value that cannot be parsed by any of the 3 CyberintTime formats (unparseable by CyberintTime parser) — e.g., a date in an entirely unrecognized notation | Raw string preserved in `raw_extensions`; OCSF `time` field set to fetch timestamp as fallback; warning logged |
| EC-018-001 | Alert ID field absent from Cyberint Alerts response record | Record emitted with available fields; warning logged for absent AlertID; `PageNumber` page counter advances normally after the full page completes — absent AlertID has no effect on pagination state |
| EC-018-002 | Cyberint Alerts and Cyberint Assets use same `access_token` credential name but different sensor_ids | Both sensors resolve independently via four-tier resolution chain with their respective sensor_id keys; no credential collision |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.01.018-001 | Valid access_token cookie; standard ISO 8601 timestamp in Cyberint alert response | Alert record parsed; `Cookie: access_token={token}` header present in request; POST body contains `"page": 1` and `"size": N` pagination parameters per ADR-056 §D3 |
| TV-BC-2.01.018-002 | Timestamp as Unix epoch seconds integer — e.g., `whois_created_date` field value `1705708800` | Unix-epoch-seconds branch of CyberintTime parser succeeds; OCSF `time` field set to correct RFC-3339 timestamp |
| TV-BC-2.01.018-003 | Timestamp string that fails all 3 CyberintTime formats (DEC-015) — e.g., `"Jan 01 2026 12:00"` | Parse fails on all 3 formats; OCSF `time` falls back to fetch timestamp; raw string preserved in `raw_extensions`; warning logged; record not dropped |
| TV-BC-2.01.018-004 | HTTP 401 cookie rejection | `PrismError::Sensor` with `category: "authentication"` and token refresh suggestion |
| TV-BC-2.01.018-005 | HTTP 429 rate limit | Exponential backoff; partial results with `truncation_reason: "rate_limited"` if retries exhausted |
| TV-BC-2.01.018-006 | Cyberint Alerts and Cyberint Assets queries in same session (shared access_token name) | Both sensors resolve independently; distinct Cookie headers with sensor-specific credential lookups; no cross-contamination |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| (none) | No VP directly verifies this BC — see VP-INDEX.md for full map |

## Related BCs
- BC-2.01.006: Cyberint Assets Cookie-Based Authentication and Multi-Format Timestamp Parsing (sibling BC; same auth mechanism, different surface — Assets API uses page-number pagination (`GetAssetsRequest.page_number`, server-controlled page size, `GetAssetsResponse.total_assets`; no cursor fields in schema); multi-page retrieval pending GAP-ASSETS-PAG-001 (no suitable `PaginationConfig` variant); Alerts surface uses `PageNumber` pagination per ADR-056 §D3)
- BC-2.01.016: SensorAuth Open Trait — Plugin-Implementable Auth Contract (No Sealed Marker) (composes with — StaticCookieAuthProvider implements the AuthProvider trait (the runtime SensorAuth replacement governed by BC-2.01.016))
- BC-2.01.017: StaticCookieAuthProvider Contract — No-Login-Roundtrip Cookie Injection (composes with — header_scheme = "cookie:<name>" dispatch)
- BC-2.06.003: Credential References in Config Resolve to Credential Store Entries (depends on — access_token credential ref resolution)

## Architecture Anchors
- ADR-053 D3: Cyberint dual-surface split rationale
- ADR-053 D2: header_scheme = "cookie:<name>" mechanism
- ADR-054 D1: 6-value canonical auth_type set
- ADR-056 §D3: PageNumber pagination dispatch — POST body injection of `page` and `size` for the Alerts surface
- ADR-056 §D7: CursorToken/CursorPagination mismatch investigation — confirms cursor pagination superseded for the Alerts surface; cursor would silently malfunction against Cyberint's `page`/`size` API contract

## Story Anchor
S-WAVE-A-CYBERINT-SPEC-001 — Cyberint Dual-Surface Spec Migration. This story creates
`cyberint-alerts.sensor.toml` with POST `/alert/api/v1/alerts`, `$.alerts` response path,
and PageNumber pagination (ADR-056 §D3); migrates the Cyberint Alerts DTU route to
POST `{"page": N, "size": 100}` body shape; renames credential ref `api_key` →
`access_token`. BC promotes to `active` on story merge per POL-14 auto-promotion.

**Historical draft rationale (F-WASE-P3-LOW-001 — story anchor resolved to
S-WAVE-A-CYBERINT-SPEC-001 in FB63):** This BC was created by ADR-053 D3 (dual-surface
split) as a spec-before-code artifact. The contracted behaviors — `cyberint-alerts.sensor.toml`,
`SensorSpec::header_scheme = "cookie:access_token"`, credential ref renamed `api_key` →
`access_token`, `PageNumber` pagination per ADR-056 §D3 (cursor pagination superseded in v1.6/FB66), and 3-format timestamp parser (RFC-3339/ISO-8601,
Unix epoch seconds, Unix epoch millis) — are NOT yet shipped. Evidence: only `cyberint.sensor.toml`
(monolithic, sensor_id = "cyberint") exists; `SensorSpec::header_scheme` field absent from spec-engine
source; current TOML uses `name = "api_key"` and `cursor_token`/`$.next_cursor` pagination; the
spec-engine's existing `timestamp_formats` set (`iso8601`, `unix_epoch_seconds`, `unix_epoch_millis` —
3 values) is exactly the 3-format set this BC now contracts, confirming no "Cyberint custom format"
is needed (RU-Q5 REFUTED per `cyberint_alerts_openapi_06.20.2026.json`). BC status set to `draft`
per POL-14. Will promote to `active` when the Wave-A Cyberint Alerts remediation story merges
(POL-14 auto-promotion at merge).

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
| 1.7 | FB70 | 2026-07-27 | product-owner | F-WASE-P66-CRIT-001 twin sweep (POL-29 9a): corrected false cross-reference in §Related BCs — BC-2.01.006 entry previously asserted "Assets surface retains `(Timestamp, AssetID)` cursor pagination" (authored in FB66 same-burst as the cursor removal from BC-2.01.018). Ground truth: `cyberint_assets_openapi_06.20.2026.json` `GetAssetsRequest` has `page_number` (integer, minimum 1) and no cursor fields; `GetAssetsResponse` has `total_assets`, `page_number`, `assets` — no cursor. Updated to accurate description: Assets API uses page-number pagination (server-controlled page size), multi-page retrieval pending GAP-ASSETS-PAG-001. No semantic change to this BC's own contracts (Alerts surface, PageNumber pagination per ADR-056 §D3). POL-29 9b: no section of this BC is a verbatim copy-source for downstream artifacts. POL-29 9c: no new MUST statements. |
| 1.6 | FB66 | 2026-07-27 | product-owner | F-WASE-P65-HIGH-004: BC-2.01.018 re-grounded on ADR-056 `PageNumber` pagination. Removed all `(Timestamp, AlertID)` cursor-pagination language: §Description cursor sentence removed and replaced with `PageNumber` pagination statement (ADR-056 §D3/§D7 grounding); §Postconditions cursor bullet replaced with `PageNumber` dispatch rule; TV-BC-2.01.018-001 cursor reference removed, POST body `"page": 1` / `"size": N` assertion added; TV-BC-2.01.018-002 "cursor timestamp extracted" removed; EC-018-001 re-grounded — cursor fallback replaced with PageNumber-era behavior (absent AlertID → record emitted + warning; page counter unaffected). §Architecture Anchors extended with ADR-056 §D3 and §D7. §Story Anchor historical rationale cursor item updated to `PageNumber` pagination per ADR-056 §D3. BC lifecycle_status unchanged (remains draft — POL-14 auto-promotion at story-PR merge). POL-9/POL-25 sweep: no out-of-scope live cursor pins found referencing this BC's Alerts surface (BC-2.01.006 Assets cursor and other sensor cursors are correct for those surfaces; BC-INDEX pin sync deferred to state-manager). |
| 1.5 | FB63-product-owner | 2026-07-27 | product-owner | Target A (FB63): §Story Anchor replaced — stale "(pending — Wave-A story decomposition, Task #8)" removed; anchor set to S-WAVE-A-CYBERINT-SPEC-001 (verified: §AC-003 of that story owns POST `/alert/api/v1/alerts`, `$.alerts` response path, PageNumber pagination per ADR-056 §D3, and Alerts DTU route migration). Historical draft rationale header updated to note story anchor resolved in FB63. No semantic change to §Preconditions, §Postconditions, §Invariants, §Edge Cases, §Canonical Test Vectors, or §Traceability. |
| 1.4 | wave-a-rmu-amendment-burst-1 | 2026-07-23 | product-owner | RU-Q5 REFUTED: canonical Cyberint OpenAPI (`cyberint_alerts_openapi_06.20.2026.json`, in-repo) confirms ALL alert date fields are `"type: string, format: date-time"` (RFC-3339/ISO-8601); spec intro states "All dates in the API use UTC and are strings in the ISO 8601 format"; only `whois_created_date` (nested enrichment) is an integer epoch field; NO custom format exists anywhere. BC amended from 4-format to 3-format: (1) RFC-3339/ISO-8601 (primary), (2) Unix epoch seconds (`whois_created_date`-class integer fields), (3) Unix epoch millis (defensive, matches spec-engine's `timestamp_formats` supported set). "Cyberint custom format" dropped from §Description (grounding citation added), §Postconditions. DEC-015 reframed: "unexpected 5th format" → "unparseable by all 3 CyberintTime formats". TV-002 rewritten: custom-format test → Unix-epoch-seconds integer test. TV-003 updated: "5th format" → "fails all 3 formats" with concrete example. |
| 1.3 | wave-a-spec-evolution-fix-burst-16 | 2026-07-22 | product-owner | F-WASE-P16-LOW-001: §Related BCs full sweep — three stale labels corrected to canonical H1s (POL-7). BC-2.01.006 label "Cyberint Assets cookie auth" → "Cyberint Assets Cookie-Based Authentication and Multi-Format Timestamp Parsing". BC-2.01.017 label "StaticCookieAuthProvider dispatch table" → "StaticCookieAuthProvider Contract — No-Login-Roundtrip Cookie Injection" (v1.2 burst fixed BC-2.01.016 on line 110 but missed lines 109/111/112). BC-2.06.003 label "Credential reference resolution" → "Credential References in Config Resolve to Credential Store Entries". Pin-sweep: no live-body pins to BC-2.01.018 v1.2 found across .factory/specs/. input-hash updated at commit time. |
| 1.2 | wave-a-spec-evolution-fix-burst-12 | 2026-07-22 | product-owner | F-WASE-P12-MED-002: §Related BCs line for BC-2.01.016 corrected — false claim "StaticCookieAuthProvider implements SensorAuth" removed. `StaticCookieAuthProvider` implements the `AuthProvider` trait (`crates/prism-spec-engine/src/auth_provider.rs`); `SensorAuth` lives in `prism-sensors` and `prism-spec-engine` is forbidden from importing it. Rewording follows the specified correction: "StaticCookieAuthProvider implements the AuthProvider trait (the runtime SensorAuth replacement governed by BC-2.01.016)". BC-2.01.016 label aligned to canonical H1 title "SensorAuth Open Trait — Plugin-Implementable Auth Contract (No Sealed Marker)". Sibling sweep (TD-VSDD-060): BC-2.01.006 — CLEAN (no hit). architecture/sensor-adapters.md — 2 hits but both are factually correct descriptions of the WASM plugin model (architect scope, not product-owner scope). input-hash updated at commit time. |
| 1.1 | wave-a-spec-evolution-burst-3 | 2026-07-22 | product-owner | F-WASE-P3-MED-001: `introduced` corrected from `wave-a-spec-evolution-burst-3` to `"2026-07-22"` (POL-20 pattern `^(cycle-[0-9]+\|[0-9]{4}-[0-9]{2}-[0-9]{2})$`). F-WASE-P3-MED-002: `timestamp` corrected from `2026-07-22T00:00:00` to `2026-07-22T00:00:00Z` (POL-23 Z-suffix required for new BCs). F-WASE-P3-LOW-001: status/lifecycle_status set to `draft` (was `active`). Adjudication verdict: contracted behavior NOT yet shipped — `cyberint-alerts.sensor.toml` does not exist, `SensorSpec::header_scheme` field does not exist in engine, credential ref name `access_token` not yet renamed from `api_key`, `(Timestamp, AlertID)` 2-tuple cursor not yet implemented (current spec uses `cursor_token`/`$.next_cursor`), 4-format CyberintTime parser not implemented (engine supports only `iso8601`/`unix_epoch_seconds`/`unix_epoch_millis`). Wave-A remediation stories (Task #8, pending) will implement these behaviors; BC promotes to active when anchor story merges (POL-14). §Story Anchor updated with interim rationale. |
| 1.0 | wave-a-spec-evolution-burst-3 | 2026-07-22 | product-owner | Initial contract. Created by ADR-053 D3 split of BC-2.01.006 (formerly "Cyberint Cookie-Based Authentication and Multi-Format Timestamp Parsing") into two surface-specific BCs. This BC covers the Cyberint Alerts surface (`cyberint-alerts.sensor.toml`, `/alert` server, `(Timestamp, AlertID)` cursor). BC-2.01.006 covers the Cyberint Assets surface (`cyberint-assets.sensor.toml`, `/asset-configuration` server, `(Timestamp, AssetID)` cursor). Both surfaces: auth_type = "cookie_roundtrip", header_scheme = "cookie:access_token", StaticCookieAuthProvider. DI-012 invariant: 6-value canonical auth_type set per ADR-054 D1. |
