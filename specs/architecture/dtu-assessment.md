---
document_type: dtu-assessment
level: L3
section: "dtu-assessment"
version: "1.1"
status: approved
producer: architect
timestamp: 2026-04-20T00:00:00Z
phase: 2-patch
inputs:
  - .factory/specs/prd.md
  - .factory/stories/STORY-INDEX.md
  - .factory/STATE.md
  - .factory/specs/architecture/write-operations.md
  - .factory/specs/architecture/sensor-adapters.md
  - .factory/specs/architecture/security-architecture.md
  - .factory/specs/verification-properties/VP-INDEX.md
input-hash: "3ff257e"
traces_to: ARCH-INDEX.md
dtu_strategy: "Option 2 — DTU-first"
dtu_strategy_decided: "2026-04-20"
---

# DTU Assessment: Prism External-Service Behavioral Clones

<!-- ## [Section Content] — template compliance marker; body sections follow -->

> **Produced:** Phase 3 patch cycle, updated Burst 5.5a. This assessment was required at Phase 1
> (P1-06) and was discovered missing at the Phase 3 → Phase 4 gate. `STATE.md` already records
> `dtu_required: true`. This document provides the formal justification and per-surface scope
> matrix that drives stories S-6.06 through S-6.19 and the `/vsdd-factory:dtu-creation` execution.
>
> **Burst 5.5a update:** Original scope covered only the 4 MSSP sensors. Human review identified
> that Prism has additional external-service integration surfaces — actions, infusions, and log
> forwarding — each requiring DTU or equivalent test infrastructure. Scope expanded to 14 DTU
> crates (5 → 14) plus documented alternatives for 5 surfaces not requiring dedicated crates.

---

## Summary

| Metric | Value |
|--------|-------|
| External dependencies identified | 17 (14 DTU + 3 non-DTU alternatives) |
| DTU clones recommended | 14 |
| Total clone story points | 72 (S-6.06 through S-6.19) |
| Estimated wave capacity needed | Waves 0–3 (distributed; see Section 12) |

---

## Integration Surface Inventory (MANDATORY — all categories required)

### Inbound Data Sources (External → Product)

Four MSSP sensor APIs that Prism polls for security event and device data.

| # | Service | Protocol | Fidelity | DTU? | Justification |
|---|---------|----------|----------|------|---------------|
| 1 | CrowdStrike Falcon API | REST/OAuth2 | L4 (adversarial) | YES — S-6.07 | Two-step fetch pipeline, irreversible containment writes, cursor pagination |
| 2 | Claroty xDome API | REST/Bearer | L4 (adversarial) | YES — S-6.08 | POST-body filtering, group_by semantics, reversible tag writes |
| 3 | Cyberint API | REST/Cookie | L2 (stateful) | YES — S-6.09 | Cookie-roundtrip auth, alert status transitions |
| 4 | Armis Centrix API | REST/Bearer+AQL | L2 (stateful) | YES — S-6.10 | AQL passthrough, timestamp fallback, device tagging |

### Outbound Operations (Product → External)

Three action-delivery channels Prism writes to when delivering alerts and reports.

| # | Service | Protocol | Fidelity | DTU? | Justification |
|---|---------|----------|----------|------|---------------|
| 1 | Slack Webhook API | HTTPS/POST | L2 (stateful) | YES — S-6.11 | Block Kit payload shape, 429+Retry-After, message_ts |
| 2 | PagerDuty Events API v2 | REST | L3 (behavioral) | YES — S-6.12 | Incident state machine: create→ack→resolve, dedup keys |
| 3 | Jira REST API v3 | REST/Basic+OAuth | L3 (behavioral) | YES — S-6.13 | Issue lifecycle transitions, field validation |
| 4 | SMTP | SMTP | N/A | NO | `mailpit` Docker container; no custom clone needed |
| 5 | Syslog (action) | UDP/TCP | N/A | NO | Generic `SyslogReceiver` in prism-dtu-common |
| 6 | Webhook (action) | HTTPS/POST | N/A | NO | Generic `WebhookReceiver` in prism-dtu-common |

### Identity & Access (Bidirectional — auth flow)

Auth is exercised through the sensor and action DTU clones; no dedicated identity-service DTU required.

| # | Service | Protocol | Fidelity | DTU? | Justification |
|---|---------|----------|----------|------|---------------|
| 1 | CrowdStrike OAuth2 token endpoint | OAuth2 | L4 | YES (bundled in S-6.07) | Token issuance + 401 rejection modeled within CrowdStrike DTU |
| 2 | Claroty Bearer / Cyberint Cookie / Armis Bearer | Per-sensor | L2–L4 | YES (bundled in S-6.08–10) | Auth failure paths modeled within each sensor DTU |

AD-017 (AI-opaque credentials) means credential values never reach the DTU — see Section 7.

### Persistence & State (Product ↔ Storage)

Prism uses RocksDB as its sole persistence layer; it is an embedded library with no network interface and requires no DTU. No external databases or message queues are used.

| # | Service | Protocol | Fidelity | DTU? | Justification |
|---|---------|----------|----------|------|---------------|
| — | RocksDB | Embedded | N/A | NO | In-process library; no network; test with real RocksDB in-process |

### Observability & Export (Product → Monitoring)

Four external log-forwarding destinations Prism writes to when the operator configures log export.

| # | Service | Protocol | Fidelity | DTU? | Justification |
|---|---------|----------|----------|------|---------------|
| 1 | Datadog Logs API | HTTPS/POST | L2 (stateful) | YES — S-6.16 | Batched ingestion, DD-API-KEY header, 413/429 handling |
| 2 | Splunk HEC | HTTPS/POST | L2 (stateful) | YES — S-6.17 | Token auth, index routing, HEC ack response shape |
| 3 | Elasticsearch Bulk API | HTTPS/POST | L2 (stateful) | YES — S-6.18 | Bulk API, partial failure responses, multi-version fixture sets |
| 4 | OTLP/HTTP | HTTPS/POST (protobuf) | L2 (stateful) | YES — S-6.19 | Protobuf encoding, 400/429/503 handling |
| 5 | Syslog (log-forward) | UDP/TCP | N/A | NO | Shared `SyslogReceiver` from prism-dtu-common |
| 6 | Webhook (log-forward) | HTTPS/POST | N/A | NO | Shared `WebhookReceiver` from prism-dtu-common |

### Enrichment & Lookup (External → Product, on-demand)

Two external enrichment services Prism queries to augment alert and CVE data.

| # | Service | Protocol | Fidelity | DTU? | Justification |
|---|---------|----------|----------|------|---------------|
| 1 | Threat Intel Aggregator (GreyNoise/VirusTotal/AbuseIPDB) | REST | L2 (stateful) | YES — S-6.14 | IP/domain/hash lookups, rate limiting, multi-source score shape |
| 2 | NVD / NIST CVSS API | REST | L2 (stateful) | YES — S-6.15 | CVE fetch, bulk fetch, cache-miss semantics, API key rate limits |
| 3 | GeoIP (MaxMind GeoLite2) | File read | N/A | NO | Offline .mmdb file; no network; test with shipped fixture .mmdb |

---

## Dependency Summary

| # | Service | Category | Fidelity | DTU? | Points | Justification |
|---|---------|----------|----------|------|--------|---------------|
| 1 | CrowdStrike Falcon API | Inbound sensor | L4 | YES — S-6.07 | 8 | Irreversible writes; two-step pipeline; VP-033/VP-036 anchor |
| 2 | Claroty xDome API | Inbound sensor | L4 | YES — S-6.08 | 8 | Brownfield reference available; L4 for write-safety pipeline |
| 3 | Cyberint API | Inbound sensor | L2 | YES — S-6.09 | 5 | Cookie auth; no public OpenAPI spec |
| 4 | Armis Centrix API | Inbound sensor | L2 | YES — S-6.10 | 5 | AQL passthrough; device tagging |
| 5 | Slack Webhook API | Outbound action | L2 | YES — S-6.11 | 3 | Block Kit shape; 429 retry |
| 6 | PagerDuty Events API v2 | Outbound action | L3 | YES — S-6.12 | 5 | Incident state machine; dedup keys |
| 7 | Jira REST API v3 | Outbound action | L3 | YES — S-6.13 | 5 | Issue lifecycle; Basic+OAuth auth |
| 8 | Threat Intel Aggregator | Enrichment | L2 | YES — S-6.14 | 5 | Multi-source score; rate limiting |
| 9 | NVD / NIST CVSS API | Enrichment | L2 | YES — S-6.15 | 5 | CVE fetch; cache-miss validation |
| 10 | Datadog Logs API | Observability | L2 | YES — S-6.16 | 3 | Batched ingestion; API key |
| 11 | Splunk HEC | Observability | L2 | YES — S-6.17 | 3 | Token auth; HEC ack |
| 12 | Elasticsearch Bulk API | Observability | L2 | YES — S-6.18 | 5 | Partial failure; multi-version |
| 13 | OTLP/HTTP | Observability | L2 | YES — S-6.19 | 5 | Protobuf; schema versioning |
| 14 | DTU Common Infrastructure | Shared harness | N/A | YES — S-6.06 | 7 | BehavioralClone trait; LatencyLayer; FailureLayer; receivers |

---

## Services NOT Requiring DTU

| # | Service | Reason |
|---|---------|--------|
| 1 | SMTP | `mailpit` pre-built binary/Docker; lettre SMTP client connects directly; no custom Rust clone needed |
| 2 | Syslog (action + log-forward) | Generic `SyslogReceiver` in prism-dtu-common; no per-destination clone needed |
| 3 | Generic webhook (action + log-forward) | Generic `WebhookReceiver` in prism-dtu-common; no per-destination clone needed |
| 4 | GeoIP (MaxMind GeoLite2) | Offline `.mmdb` file read by `maxminddb` crate; no network calls; test with shipped fixture file |
| 5 | RocksDB | Embedded in-process library; no network interface; no clone possible or needed |

---

## DTU Architecture

All 14 DTU clones are implemented as in-process Rust crates (Axum HTTP servers bound to random ports on `127.0.0.1`, spawned as `tokio::task`s within the test runtime). No Docker, no external processes, no port conflicts. See Section 4 for full delivery model rationale.

| Clone | Crate | Fidelity | Depends On |
|-------|-------|----------|------------|
| DTU Common | prism-dtu-common | N/A | S-0.02 |
| CrowdStrike | prism-dtu-crowdstrike | L4 | prism-dtu-common |
| Claroty xDome | prism-dtu-claroty | L4 | prism-dtu-common |
| Cyberint | prism-dtu-cyberint | L2 | prism-dtu-common |
| Armis Centrix | prism-dtu-armis | L2 | prism-dtu-common |
| Slack | prism-dtu-slack | L2 | prism-dtu-common |
| PagerDuty | prism-dtu-pagerduty | L3 | prism-dtu-common |
| Jira | prism-dtu-jira | L3 | prism-dtu-common |
| Threat Intel | prism-dtu-threatintel | L2 | prism-dtu-common |
| NVD | prism-dtu-nvd | L2 | prism-dtu-common |
| Datadog | prism-dtu-datadog | L2 | prism-dtu-common |
| Splunk HEC | prism-dtu-splunk-hec | L2 | prism-dtu-common |
| Elasticsearch | prism-dtu-elasticsearch | L2 | prism-dtu-common |
| OTLP | prism-dtu-otlp | L2 | prism-dtu-common |

All clones expose a shared `POST /dtu/configure` protocol (defined in prism-dtu-common) for failure injection and state reset. No Docker Compose required for CI; a `docker-compose.integration.yml` is available as an opt-in hybrid for manual development use.

---

## Clone Development Approach

Each DTU clone is delivered as a VSDD story (S-6.06–S-6.19) following the DTU-first wave schedule (Section 12). Development follows Prism's standard TDD discipline:

- Behavioral contracts derived from vendor API documentation and captured fixtures
- Axum route handlers with `tower` middleware layers (`LatencyLayer`, `FailureLayer` from prism-dtu-common)
- Fixture JSON files in `crates/dtu/{surface}/fixtures/` (one file per endpoint per scenario)
- Schema validation at test time via `jsonschema` or equivalent; fidelity target ≥ 85% per crate
- Reset endpoint (`POST /dtu/reset`) for deterministic test setup
- All crates are `[dev-dependencies]` only — zero impact on the production `prism` binary
- `just integration-test` starts the full 14-crate fleet inline; `just dtu-validate` runs fidelity scoring

Wave scheduling ensures every DTU clone is built before its product consumers. See Section 12 for the full wave distribution and dependency chain verification.

---

## 1. Decision Summary

**DTU_REQUIRED: true** — spans these external-service surface categories: sensors (4), actions (3), infusions (2), log forwarding (4).

Prism has four categories of external-service integration, each requiring deterministic test
infrastructure:

1. **Sensors** (4 MSSP sensors) — behavioral clones for read/write API surfaces
2. **Actions** (Slack, PagerDuty, Jira) — behavioral clones for action delivery destinations
3. **Infusions** (threat-intel aggregator, NVD/CVSS) — behavioral clones for enrichment APIs
4. **Log forwarding** (Datadog, Splunk HEC, Elasticsearch, OTLP) — behavioral clones for log destinations

**Surfaces NOT requiring dedicated DTU crates:**
- **SMTP** — use `mailpit` Docker container (proven test infra; lettree-compatible; no custom clone needed)
- **Syslog (action + log-forward)** — generic UDP/TCP receiver provided in `prism-dtu-common`; no per-destination clone needed
- **Generic webhook (action + log-forward)** — generic HTTP receiver provided in `prism-dtu-common`; no per-destination clone needed
- **GeoIP (MaxMind GeoLite2)** — offline `.mmdb` file read by `maxminddb` crate; no network; no DTU needed; test with shipped fixture `.mmdb`

**Total DTU crates: 14**

| Crate | Category | Fidelity |
|-------|----------|----------|
| `prism-dtu-common` | Shared infrastructure | N/A (shared harness) |
| `prism-dtu-crowdstrike` | Sensor | L4 (adversarial) |
| `prism-dtu-claroty` | Sensor | L4 (adversarial) — re-classified Burst 5.5a |
| `prism-dtu-cyberint` | Sensor | L2 (stateful) |
| `prism-dtu-armis` | Sensor | L2 (stateful) |
| `prism-dtu-slack` | Action | L2 (stateful) |
| `prism-dtu-pagerduty` | Action | L3 (behavioral) |
| `prism-dtu-jira` | Action | L3 (behavioral) |
| `prism-dtu-threatintel` | Infusion | L2 (stateful) |
| `prism-dtu-nvd` | Infusion | L2 (stateful) |
| `prism-dtu-datadog` | Log forwarding | L2 (stateful) |
| `prism-dtu-splunk-hec` | Log forwarding | L2 (stateful) |
| `prism-dtu-elasticsearch` | Log forwarding | L2 (stateful) |
| `prism-dtu-otlp` | Log forwarding | L2 (stateful) |

**Artifact dependencies:**

| Artifact | Dependency on DTU |
|----------|-------------------|
| VP-033 | Audit buffer integration test (prism-audit) requires a sensor adapter to complete an HTTP cycle so the pre/post audit write ordering can be verified |
| VP-036 | SessionContext drop test (prism-operations) requires a full query cycle against a sensor endpoint |
| S-6.06 | Rescoped: DTU common infra story — builds `prism-dtu-common` with syslog/webhook receivers; this assessment is the scope input |
| S-6.07–S-6.10 | Per-sensor DTU stories (CrowdStrike, Claroty, Cyberint, Armis) |
| S-6.11–S-6.13 | Actions DTU stories (Slack, PagerDuty, Jira) |
| S-6.14–S-6.15 | Infusions DTU stories (ThreatIntel, NVD) |
| S-6.16–S-6.19 | Log-forwarding DTU stories (Datadog, Splunk HEC, Elasticsearch, OTLP) |
| S-0.02 | Developer toolchain (`just integration-test`) invokes the full 14-crate DTU fleet |
| Story S-3.06 | Write operations integration — contain/uncontain/acknowledge cycles require DTU CrowdStrike |
| Story S-3.07 | Write safety system — three-gate pipeline exercise requires realistic sensor write responses |

---

## 1a. Fidelity Taxonomy (Canonical L0–L4 Definitions)

All fidelity level references in this document and across the project use the following taxonomy. Any legacy labels ("Full", "Partial", "Shape-only") are deprecated — use L0–L4 exclusively.

| Level | Label | Definition |
|-------|-------|------------|
| **L0** | schema-only | No network, no error simulation. Static fixture data returned for every request. Validates Prism parses the response shape correctly. No state, no error paths. |
| **L1** | happy-path | Schema + success responses for all in-scope endpoints. No error simulation, no state between calls. Sufficient for fire-and-forget or read-only integrations with trivial failure modes. |
| **L2** | stateful | L1 + error simulation (4xx/5xx on configurable triggers) + stateful behavior across calls within a test session (e.g., write then read reflects the write). Sufficient for CRUD APIs and log-forwarding destinations. |
| **L3** | behavioral | L2 + full state machine semantics + deduplication/idempotency invariants. Required when the API has ordered lifecycle transitions (e.g., PagerDuty incident: trigger → acknowledge → resolve) or dedup semantics that Prism must respect. |
| **L4** | adversarial | L3 + irreversible-operation simulation + adversarial failure injection (mid-pagination 5xx, partial write failures, confirmation-token validation). Required when Prism performs writes that cannot be undone or when the integration test must validate the three-gate write-safety pipeline end-to-end. |

**Assignment rationale summary:**
- CrowdStrike and Claroty are L4 (adversarial) because both have irreversible write operations (host containment; device tagging is reversible but Claroty's write-safety pipeline validation requires adversarial-grade error injection per S-6.08).
- PagerDuty and Jira are L3 (behavioral) because their APIs have ordered state machines and dedup semantics but no irreversible operations in Prism v1 scope.
- All other surfaces are L2 (stateful) because they require error simulation and some stateful behavior (rate-limit counters, received-payload capture, tag stores) but have no complex state machines or irreversible operations.

---

## 2. Rationale — Why DTU Clones

### Why real sensor instances are not viable in CI

**Credential availability.** Real API credentials (CrowdStrike client_id/client_secret, Claroty
bearer tokens, Cyberint session cookies, Armis API keys) are MSSP customer-specific. They cannot
be checked in, stored in CI secrets at build time, or shared across developer workstations. AD-017
(AI-opaque credential management) further prohibits credential values from transiting through any
automated AI context. There is no safe path to injecting four sets of real vendor credentials into
CI runners.

**Non-determinism.** Live sensor data changes continuously — hosts appear and disappear, detections
get triaged, device inventory ages. Tests asserting on record counts, field values, or state
transitions would be flaky by construction. Prism's integration tests must be deterministic.

**Rate limits and quota cost.** CrowdStrike's Falcon API enforces per-minute and per-day rate
limits. Claroty's xDome API uses linear-backoff retry on 429 (confirmed by reference implementation
at `.references/mcp-claroty-xdome/src/integrations/claroty/xdome-api-client.ts`). Running a CI
pipeline that fires API calls on every PR would exhaust rate limits or incur per-call costs for
Cyberint and Armis commercial tiers.

**Vendor dependency availability.** Integration tests that depend on an external SaaS endpoint
are subject to that vendor's incident windows, maintenance windows, and network availability from
the CI runner. A CrowdStrike API degradation event should not block a Prism PR merge.

**Vendor ToS and data handling.** MSSP customer data returned by live sensors carries data handling
obligations. Pulling real endpoint data into CI log artifacts likely violates those obligations.
Fixtures and canned responses in the DTU avoid this entirely.

### Why simple mocks are insufficient

A mock (e.g., `mockito` or an in-test `Arc<dyn SensorAdapter>` that returns hardcoded data)
tests Prism's assumptions about the sensor API, not the sensor API itself. Key failure modes that
mocks cannot catch:

- **Schema drift:** The real CrowdStrike `/detects/entities/summaries/GET/v1` endpoint adds or
  renames a field. A mock continues to return the old schema. Prism's TOML spec parses correctly
  against the mock but would fail against the real API. A DTU clone that validates its responses
  against the captured OpenAPI schema catches this.

- **Two-step pipeline correctness:** CrowdStrike's two-step fetch (query IDs then batch-detail)
  requires the DTU to maintain a stateful ID registry across two HTTP calls within the same query
  session. A simple mock returning a flat fixture cannot exercise this pipeline.

- **Error sequence behavior:** When the DTU returns 429 on the second page of a paginated
  response, does Prism's `E-SENSOR-003` propagate correctly, include `retry_after_seconds`,
  and set `partial_results: true`? A mock that always returns 200 cannot validate this path.

- **Write state transitions:** After a `contain` write to the CrowdStrike DTU, subsequent reads
  to `crowdstrike_hosts` for that device_id must return `containment_status: "contained"`. A mock
  has no shared state between the write handler and the read handler.

DTU clones are the middle ground: behaviorally complete enough to catch integration regressions,
controlled enough to run deterministically and without real credentials.

---

## 3. Scope Matrix — Per-Sensor DTU Requirements

### 3.1 CrowdStrike Falcon

**Real API surface used by Prism (read):**

| Endpoint | Method | Purpose | Prism Table |
|----------|--------|---------|-------------|
| `/detects/queries/detects/v1` | GET | Step 1: query detection IDs (filter push-down) | `crowdstrike_alerts` |
| `/detects/entities/summaries/GET/v1` | POST | Step 2: fetch full detection details (batch 100) | `crowdstrike_alerts` |
| `/devices/queries/devices/v1` | GET | Step 1: query host IDs | `crowdstrike_hosts` |
| `/devices/entities/devices/v2` | GET | Step 2: fetch host details | `crowdstrike_hosts` |

**Real API surface used by Prism (write) — from write-operations.md risk table:**

| Endpoint | Method | Purpose | Pipe Verb | Risk Tier |
|----------|--------|---------|-----------|-----------|
| `/devices/entities/devices-actions/v2?action_name=contain` | POST | Host containment | `contain` | Irreversible |
| `/devices/entities/devices-actions/v2?action_name=lift_containment` | POST | Release containment | `uncontain` | Reversible |
| `/detects/entities/detects/v2` | PATCH | Update detection status | `update_status` | Reversible |
| `/detects/entities/detects/v2` | PATCH | Assign detection | `assign` | Reversible |

**In-scope for DTU:**
All 8 endpoints above. CrowdStrike is the highest-priority sensor — it has the most write
endpoints, the most complex two-step read pipeline, and the only irreversible write operation
(host containment) in the initial sensor set.

**Out-of-scope for DTU:**
- CrowdStrike Intelligence, Spotlight, Fusion, Discover — Prism v1 does not use these modules.
- Streaming event bus API — not in v1 scope.
- RTR (Real-Time Response) — not exposed via PrismQL; destructive ops not surfaced in MCP.

**Fidelity level: L4 (adversarial)**
- Response schema validation against CrowdStrike OpenAPI spec for all 8 endpoints.
- Two-step fetch pipeline: DTU must maintain a stateful ID registry per session so Step 2 batch
  calls return records matching the IDs returned in Step 1.
- Stateful writes: after a `contain` POST, GET `/devices/entities/devices/v2` for the same
  device_id must return `status: "contained"` on subsequent calls within the same DTU session.
- Cursor pagination: `/detects/queries/detects/v1` must support `next_token` pagination.

**Error simulation required:**
- HTTP 429 on Step 1 detection query → maps to `E-SENSOR-003`, `retry_after_seconds` populated.
- HTTP 500 on Step 2 entity fetch → maps to `E-SENSOR-002`, degraded severity, retryable.
- HTTP 401 on OAuth2 token endpoint → maps to `E-AUTH-001`, broken severity.
- Mid-pagination 503 (Step 2, batch 2 of 3) → maps to `E-SENSOR-005`, partial results.
- Write `contain` returning 400 "already contained" → maps to `E-QUERY-025` (partial write failure),
  partial success path — some records succeed while this one fails.
- Write returning 429 → maps to `E-QUERY-028`, partial execution acknowledged.
- Confirmation token missing on irreversible write → must return structured error before any API call.

**Stateful behavior requirements:**
- Yes. Contains a write state store (device_id → containment_status map). POST to contain
  transitions the entry; subsequent GET returns updated status. Reset endpoint for test setup.
- Session-scoped detection ID registry for two-step fetch.

**Size estimate:** ~700-900 lines Rust (axum), 4 fixture JSON files (one per read table),
8 route handlers, 1 state store, 1 reset endpoint. Approximately 12 endpoints total.

---

### 3.2 Claroty xDome

**Real API surface used by Prism (read) — confirmed from reference `.references/mcp-claroty-xdome/`:**

| Endpoint | Method | Purpose | Prism Table |
|----------|--------|---------|-------------|
| `/api/v1/devices` | POST | Device inventory with field selection and group_by | `claroty_devices` |
| `/api/v1/alerts` | POST | Alert list with filter params | `claroty_alerts` |
| `/api/v1/alerts/{alert_id}/devices` | POST | Devices associated with a specific alert | `claroty_alerted_devices` |
| `/api/v1/vulnerabilities` | POST | Vulnerability inventory | `claroty_vulnerabilities` |
| `/api/v1/vulnerabilities/{vuln_id}/devices` | POST | Devices affected by a vulnerability | `claroty_vulnerability_devices` |

The reference implementation (`xdome-api-client.ts`) confirms: Bearer token auth, POST-body
filtering, 15s timeout, retry on 429 and 5xx (linear backoff), errors on 401/403/404/422.

**Real API surface used by Prism (write):**

| Endpoint | Method | Purpose | Pipe Verb | Risk Tier |
|----------|--------|---------|-----------|-----------|
| `/api/v1/devices/{device_id}/tags/` | POST | Add device tag | `tag` | Reversible |
| `/api/v1/devices/{device_id}/tags/{tag_key}` | DELETE | Remove device tag | `remove_tag` | Reversible |

**In-scope for DTU:**
All 7 endpoints above. The Claroty reference implementation is the only brownfield reference
with real API surface to bootstrap from — the DTU fixture schemas can be derived directly from
the TypeScript type definitions in `.references/mcp-claroty-xdome/src/types/`.

**Out-of-scope for DTU:**
- Claroty CTD (Continuous Threat Detection) module — separate product, not in v1 scope.
- Communication graph APIs — referenced in PRD as future scope.
- Network segment management — not surfaced in PrismQL v1.

**Fidelity level: L4 (adversarial)**
- Response schema validation against types in the reference implementation.
- `group_by` parameter behavior: when `group_by` is set, the DTU must return only the grouped
  fields (as confirmed by reference implementation logic).
- Stateful device tagging: POST tag followed by GET devices returns updated tag list.
- Retry-after behavior on 429 to validate Prism's `E-SENSOR-003` path.

**Error simulation required:**
- HTTP 401/403 → `E-AUTH-003` (bearer token rejected).
- HTTP 429 → `E-SENSOR-003` with `Retry-After` header.
- HTTP 422 → `E-SENSOR-004` (unexpected format — invalid filter params).
- HTTP 500 → `E-SENSOR-002`.
- Timeout (server holds connection > 15s) → `E-SENSOR-001`.

**Stateful behavior requirements:**
- Yes, for write paths only. Device tag store (device_id → tag set). Reset endpoint for test setup.
- Read endpoints are stateless — fixture data is sufficient for read-path tests.

**Bootstrapping advantage:** The TypeScript type definitions in `.references/mcp-claroty-xdome/src/types/claroty.ts`
provide the canonical response schemas. These should be translated to Rust structs for the DTU
fixture validation layer, avoiding manual schema re-derivation.

**Size estimate:** ~500-700 lines Rust (axum), 5 fixture JSON files, 7 route handlers, 1 tag state
store, 1 reset endpoint. Approximately 9 endpoints total.

---

### 3.3 Cyberint

**Real API surface used by Prism (read):**

| Endpoint | Method | Purpose | Prism Table |
|----------|--------|---------|-------------|
| `POST /login` | POST | Cookie-based authentication (CookieRoundtrip) | — (auth only) |
| `/api/v1/alerts` | GET/POST | Alert list with filter and pagination | `cyberint_alerts` |
| `/api/v1/alerts/{alert_id}` | GET | Alert detail | `cyberint_alerts` |
| `/api/v1/threat-intel` | GET | Threat intelligence feed | `cyberint_threats` |

**Real API surface used by Prism (write):**

| Endpoint | Method | Purpose | Pipe Verb | Risk Tier |
|----------|--------|---------|-----------|-----------|
| `/api/v1/alerts/{alert_id}/status` | PATCH | Acknowledge alert | `acknowledge` | Reversible |
| `/api/v1/alerts/{alert_id}/close` | POST | Close alert permanently | `close_alert` | Irreversible |

**In-scope for DTU:**
All 6 endpoints above, plus the cookie auth flow. Cyberint's `CookieRoundtrip` auth pattern is
unique among the four sensors — the DTU must implement the POST login → session cookie flow so
Prism's `CookieRoundtrip` handler is exercised in integration tests.

**Out-of-scope for DTU:**
- Dark web monitoring export endpoints — read-only, low-frequency, not in v1 PrismQL table scope.
- Historical intelligence archives — pagination-heavy, tested adequately via partial fixture sets.
- Indicator submission endpoints — future scope.

**Fidelity level: L2 (stateful)**
- Response schema + happy path for read endpoints.
- Cookie auth round-trip: POST login returns `Set-Cookie` with a fake session token; subsequent
  requests must validate `Cookie` header presence (any value accepted in DTU — no crypto).
- Stateful alert status: PATCH acknowledge transitions alert `status` field; subsequent GET
  returns updated status. Close alert also transitions (irreversible in DTU — no undo in session).
- Error simulation for auth and rate limit paths only (not full error matrix).

Cyberint has a simpler read-dominated API than CrowdStrike. The irreversible `close_alert` write
is the most complex path — confirmation token + audit intent ordering must be validated.

**Error simulation required:**
- HTTP 401 on any request with missing/invalid cookie → `E-AUTH-004`.
- HTTP 429 on alert list → `E-SENSOR-003`.
- HTTP 500 → `E-SENSOR-002`.
- Multi-format timestamp response (Cyberint uses inconsistent timestamp formats) → must be in fixtures
  to validate Prism's timestamp normalization in the TOML spec.

**Stateful behavior requirements:**
- Yes. Alert status store (alert_id → status). Cookie session registry (session_token → valid).
- Reset endpoint for test isolation.

**Size estimate:** ~450-600 lines Rust (axum), 4 fixture JSON files, 6 route handlers, 1 alert state
store, 1 session store, 1 reset endpoint. Approximately 8 endpoints total.

---

### 3.4 Armis Centrix

**Real API surface used by Prism (read):**

| Endpoint | Method | Purpose | Prism Table |
|----------|--------|---------|-------------|
| `/api/v1/devices` | GET/POST | Device inventory (AQL forwarding) | `armis_devices` |
| `/api/v1/devices/{device_id}/activity` | GET | Device activity log | `armis_device_activity` |
| `/api/v1/alerts` | GET | Alert / policy violation list | `armis_alerts` |
| `/api/v1/devices/{device_id}/risk` | GET | Device risk score | (column on armis_devices) |

Armis uses AQL (Armis Query Language) forwarding via BearerStatic auth. Timestamp fallback
(primary field missing → secondary field) is documented in sensor-adapters.md.

**Real API surface used by Prism (write):**

| Endpoint | Method | Purpose | Pipe Verb | Risk Tier |
|----------|--------|---------|-----------|-----------|
| `/api/v1/devices/{device_id}/tags/` | POST | Add device tag | `tag` | Reversible |
| `/api/v1/devices/{device_id}/tags/{tag_key}` | DELETE | Remove device tag | `remove_tag` | Reversible |

**In-scope for DTU:**
All 6 endpoints above. Armis has the simplest API profile in the set — read-dominated, two
reversible-only write operations, BearerStatic auth (no OAuth2 or cookie flows).

**Out-of-scope for DTU:**
- Armis policy management API — not surfaced in PrismQL v1.
- Armis network segmentation controls — not in v1 scope.
- Armis Vulnerability Management module — future scope, after v1.

**Fidelity level: L2 (stateful)**
- Response schema + happy path for all read endpoints.
- AQL pass-through: DTU accepts any AQL string and returns a fixture dataset (no AQL parsing —
  test coverage of AQL push-down is handled by unit tests on the TOML spec pipeline, not the DTU).
- Timestamp fallback: fixture includes devices where primary timestamp is null and secondary is
  populated, to exercise Prism's fallback logic.
- Stateful tagging (same pattern as Claroty).
- Error simulation for auth and rate limit only.

**Error simulation required:**
- HTTP 403 on invalid bearer token → `E-AUTH-003`.
- HTTP 429 → `E-SENSOR-003`.
- HTTP 500 → `E-SENSOR-002`.
- Timestamp fallback fixture: device where `last_seen` is null, `first_seen` is populated.

**Stateful behavior requirements:**
- Minimal. Tag store only (device_id → tag set). Reset endpoint.
- Read endpoints are fully stateless (fixture-driven).

**Size estimate:** ~350-450 lines Rust (axum), 4 fixture JSON files, 6 route handlers, 1 tag state
store, 1 reset endpoint. Approximately 8 endpoints total.

---

---

## 3.5 Actions DTU Scope

Three action destinations require dedicated DTU crates. Three others use alternative test infrastructure already provided in `prism-dtu-common` or as an external tool.

**Not DTU-covered (rationale):**
- **SMTP:** `mailpit` Docker container. Mailpit provides a full SMTP server + web UI + REST API to inspect received emails. `lettre` in Prism connects to mailpit's SMTP port. No behavioral clone needed — mailpit is the de facto standard for SMTP integration testing.
- **Syslog:** Generic `SyslogReceiver` in `prism-dtu-common` (RFC 5424, UDP + TCP). No per-destination clone needed.
- **Generic webhook:** Generic `WebhookReceiver` in `prism-dtu-common`. No per-destination clone needed.

### 3.5.1 Slack Webhook

**Real API surface used by Prism:**

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `hooks.slack.com/services/{token}` | POST | Deliver Block Kit message payload |

Slack webhooks are one-way: Prism POSTs a JSON payload; Slack returns `200 OK` with body `"ok"` or an error string. The `message_ts` field in successful responses is used by Prism for deduplication state tracking.

**In-scope for DTU:**
- Block Kit JSON payload validation (reject payloads with unknown top-level fields).
- `message_ts` in 200 response body.
- 429 with `Retry-After` header.
- 400 error for malformed payload (missing `blocks` or `text`).

**Fidelity level: L2 (stateful)**
- Stateful only for rate-limit tracking: DTU maintains a per-test request counter; after `N` requests (configurable via `POST /dtu/configure`), returns 429 with `Retry-After: 30`.
- Response shape validation: DTU validates received payload against Block Kit schema before returning 200.

**Error simulation required:**
- HTTP 429 with `Retry-After: 30` → tests Prism action retry logic.
- HTTP 400 with body `"invalid_payload"` → tests Prism `E-ACTION-004` error path.
- HTTP 500 (configurable) → tests retry exhaustion path.

**Stateful behavior requirements:**
- Rate-limit counter (request count → trigger 429 after threshold).
- Received payload capture via `received_payloads()` test API (for assertion in integration tests).
- Reset endpoint for test isolation.

**Size estimate:** ~250-350 lines Rust (axum), 2 fixture JSON files, 2 route handlers, 1 rate-limit state store, 1 payload capture store. Approximately 3 endpoints total.

---

### 3.5.2 PagerDuty Events API v2

**Real API surface used by Prism:**

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `events.pagerduty.com/v2/enqueue` | POST | Create/ack/resolve incident via routing key |

PagerDuty Events API v2 uses a single `/v2/enqueue` endpoint with `event_action` field driving
behavior: `trigger` (create), `acknowledge`, `resolve`. `dedup_key` is used to correlate events
into a single incident lifecycle.

**In-scope for DTU:**
- `trigger` action → creates incident, returns `dedup_key` + `status: "success"`.
- `acknowledge` action → transitions incident to `acknowledged` state.
- `resolve` action → transitions incident to `resolved` state.
- Dedup key tracking: multiple `trigger` events with the same `dedup_key` do not create duplicate incidents.
- Severity mapping validation: Prism sends `critical/error/warning/info`; DTU validates these are the only accepted values.
- 429 with `Retry-After` header.
- 400 for missing `routing_key` or unknown `event_action`.
- 403 for invalid routing key (configurable).

**Fidelity level: L3 (behavioral)**
- Full stateful incident lifecycle: trigger → acknowledge → resolve transitions.
- State is keyed by `dedup_key`: each dedup key has an associated lifecycle state.
- Attempting to `acknowledge` a `resolved` incident returns a structured error.
- Dedup: sending `trigger` with an existing unresolved dedup key returns `status: "success"` without creating a new incident (idempotent per PagerDuty spec).

**Error simulation required:**
- HTTP 429 with `Retry-After: 60` → Prism action retry.
- HTTP 403 with `{"status": "invalid key"}` → `E-ACTION-AUTH-001`.
- HTTP 400 for missing fields → `E-ACTION-SCHEMA-001`.

**Stateful behavior requirements:**
- Incident registry: `dedup_key → (state: triggered|acknowledged|resolved, severity, summary)`.
- Reset endpoint for test isolation.
- `incidents()` test API returning current state for assertion.

**Size estimate:** ~450-600 lines Rust (axum), 3 fixture JSON files (one per lifecycle state), 1 route handler (`/v2/enqueue`), 1 incident state store, 1 reset endpoint. Approximately 3 endpoints total.

---

### 3.5.3 Jira REST API v3

**Real API surface used by Prism:**

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/rest/api/3/issue` | POST | Create issue |
| `/rest/api/3/issue/{issueKey}/comment` | POST | Add comment to issue |
| `/rest/api/3/issue/{issueKey}/transitions` | GET | List available status transitions |
| `/rest/api/3/issue/{issueKey}/transitions` | POST | Execute a status transition |
| `/rest/api/3/issue/{issueKey}` | GET | Fetch issue (for state verification) |

Jira Cloud REST API v3. Auth: Basic (email:api-token, Base64 in `Authorization` header) or OAuth 2.0.
The `prism-dtu-jira` supports Basic auth as primary; OAuth is an optional fixture.

**In-scope for DTU:**
- Create issue: validates required fields (`project.key`, `issuetype.name`, `summary`), returns `id` + `key` (e.g., `"ACME-SEC-1234"`).
- Add comment: associates comment with issue, returns `id`.
- List transitions: returns a fixed transition list (Open → In Progress → Done) per fixture.
- Execute transition: validates `transition.id` is in the available set, updates issue status.
- GET issue: returns current issue state including `status.name`.
- Field validation: reject create requests with unknown issue type or missing project key.
- Auth validation: 401 for missing/invalid `Authorization` header.

**Fidelity level: L3 (behavioral)**
- Full status machine: Open → In Progress → Done. Transitions are validated against the available set.
- Attempting an invalid transition (e.g., transition to a state not reachable from current state) returns 400.
- Comment history: GET issue includes comment count (not full body — just count for test assertion).

**Error simulation required:**
- HTTP 401 for missing/invalid auth → `E-ACTION-AUTH-002`.
- HTTP 400 for field validation errors → `E-ACTION-SCHEMA-002`.
- HTTP 404 for unknown `issueKey` → `E-ACTION-NOT-FOUND-001`.
- HTTP 429 (configurable) → retry path.

**Stateful behavior requirements:**
- Issue registry: `issueKey → (status, summary, comment_count, fields)`.
- Transition registry: per issue, available transitions based on current status.
- Reset endpoint and `issues()` test API.

**Size estimate:** ~600-800 lines Rust (axum), 4 fixture JSON files, 5 route handlers, 1 issue state store, 1 reset endpoint. Approximately 7 endpoints total.

---

## 3.6 Infusions DTU Scope

Two external-API-backed infusion providers require DTU crates. One local-lookup infusion does not.

**Not DTU-covered (rationale):**
- **GeoIP (MaxMind GeoLite2):** `maxminddb` crate reads a local `.mmdb` file — no network, no external service. Integration tests ship a minimal fixture `.mmdb` file containing 5–10 test IP ranges. No behavioral clone needed.

### 3.6.1 Threat Intel Aggregator (prism-dtu-threatintel)

**Provider confirmed from `infusions.md`:** The `threat_intel.infusion.toml` spec enables a
multi-provider aggregator: **GreyNoise + VirusTotal + AbuseIPDB** combined into a unified
`threat_score` (0–100) and `threat_sources` (comma-separated list) output.

The `prism-dtu-threatintel` models the **GreyNoise API as primary** (listed first in the spec,
SOC-oriented, IP-focused — the primary use case for MSSP threat intel lookups). VirusTotal and
AbuseIPDB response shapes are included in the fixture library as secondary providers that
contribute to the aggregated score. The DTU exposes a single unified endpoint that returns the
aggregated response shape the plugin produces — not individual per-provider endpoints.

**Real API surface modeled:**

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/v3/ip/{ip}` | GET | GreyNoise IP context lookup |
| `/vtapi/v3/ip_addresses/{ip}` | GET | VirusTotal IP lookup (secondary fixture) |
| `/api/v2/check` | GET | AbuseIPDB IP check (secondary fixture) |

For integration test purposes, the DTU exposes a simplified unified endpoint that returns the
aggregated plugin output shape (as consumed by Prism's infusion cache), not the raw per-provider
format. Per-provider fixture files are included for reference.

**In-scope for DTU:**
- IP address lookup: returns `threat_score` (0–100), `threat_sources` (list), `threat_is_known_malicious` (bool).
- Domain lookup: returns same shape with domain as input.
- Hash lookup: file hash lookup returns `threat_score` from VirusTotal-style fixture.
- API key auth: `key` query parameter or `Authorization` header; 401 on missing/invalid.
- Rate limiting: configurable 429 with `Retry-After`.

**Fidelity level: L2 (stateful)**
- Stateful for rate-limit counter only.
- Fixture-driven responses: each test IP/domain/hash maps to a fixture response (malicious, benign, unknown).
- `POST /dtu/configure` to add new IP→fixture mappings for specific test scenarios.

**Error simulation required:**
- HTTP 401 → `E-INFUSION-AUTH-001`.
- HTTP 429 with `Retry-After` → tests infusion cache bypass on rate-limit.
- HTTP 404 (IP not found) → returns `threat_score: 0, threat_is_known_malicious: false`.

**Stateful behavior requirements:**
- Rate-limit counter (request count → 429 after threshold).
- Fixture registry: IP/domain/hash → response (pre-loaded from fixture files, overridable via `POST /dtu/configure`).
- Reset endpoint.

**Size estimate:** ~350-450 lines Rust (axum), 3 fixture JSON files (IP malicious, IP benign, hash malicious), 3 route handlers (ip/domain/hash), 1 rate-limit store, 1 fixture registry. Approximately 5 endpoints total.

---

### 3.6.2 NVD / NIST CVSS API (prism-dtu-nvd)

**Real API surface used by Prism:**

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `services.nvd.nist.gov/rest/json/cves/2.0` | GET | CVE lookup by `cveId` parameter |
| `services.nvd.nist.gov/rest/json/cves/2.0` | GET | Bulk CVE fetch (date range, keyword) |

NVD API 2.0. Auth: optional `apiKey` query parameter. Without key: 5 requests/30s. With key: 50 requests/30s.

**In-scope for DTU:**
- Single CVE fetch by `cveId`: returns CVE descriptor with CVSS v3.1 base score, severity, CWE, CISA KEV flag.
- Bulk fetch: returns paginated result set (uses `startIndex`, `resultsPerPage` parameters).
- Cache-miss behavior: first fetch populates the `infusion_cache` RocksDB CF; subsequent fetches return cached value. DTU validates Prism's cache-hit path by tracking whether the same CVE is requested twice (second request should not reach the DTU — integration test asserts request count = 1).
- Rate limiting: unauthenticated 5/30s and authenticated 50/30s buckets, keyed by `apiKey` query parameter.
- `lastModStartDate` / `lastModEndDate` bulk filter (accepts any date range, returns fixture subset).

**Fidelity level: L2 (stateful)**
- Stateful for rate-limit buckets (per `apiKey` value, per 30-second window).
- CVE fixture registry: `cveId → CVE descriptor`. Pre-loaded with 10 fixture CVEs including: CVSS 9.8 (critical, CISA KEV), CVSS 7.5 (high), CVSS 4.3 (medium), CVSS 0.0 (informational), unknown CVE (404).
- Request counter per `cveId` exposed via test API (to assert Prism's cache-hit behavior).

**Error simulation required:**
- HTTP 403 with `"Forbidden. apiKey not verified."` → `E-INFUSION-AUTH-002`.
- HTTP 429 → `E-INFUSION-RATE-001`, tests Prism's rate-limit backoff for NVD.
- HTTP 404 for unknown CVE → `vuln_context` infusion returns empty/null CVSS for that CVE.

**Stateful behavior requirements:**
- Rate-limit buckets per API key per 30-second window.
- CVE fixture registry with request counter.
- Reset endpoint.

**Size estimate:** ~400-500 lines Rust (axum), 1 route handler (`GET /rest/json/cves/2.0`), 10 fixture CVE JSON objects, 1 rate-limit store, 1 request counter store. Approximately 3 endpoints total.

---

## 3.7 Log Forwarding DTU Scope

Four log-forwarding destinations require DTU crates. Two others use existing receivers from `prism-dtu-common`.

**Not DTU-covered (rationale):**
- **Syslog forwarder:** `SyslogReceiver` in `prism-dtu-common` (same receiver used for action syslog).
- **Webhook forwarder:** `WebhookReceiver` in `prism-dtu-common` (same receiver used for webhook action).

### 3.7.1 Datadog Logs API (prism-dtu-datadog)

**Real API surface:**

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `https://http-intake.logs.datadoghq.com/api/v2/logs` | POST | Batched log ingestion |

Auth: `DD-API-KEY` header. Batches are JSON arrays of log objects.

**In-scope for DTU:**
- POST log batch: validates `DD-API-KEY` header, validates batch is a JSON array, returns `{"status":"ok"}`.
- 413 for batch payload exceeding 5 MB.
- 429 with `Retry-After` header.
- 403 for missing or invalid API key.
- Received log capture via `received_logs()` test API.

**Fidelity level: L2 (stateful)**
- Stateful for rate-limit counter and received log capture.
- Validates required log fields: `ddsource`, `service`, `message`.

**Size estimate:** ~250-350 lines Rust (axum), 1 route handler, 2 fixture files (success, 413), 1 log capture store.

---

### 3.7.2 Splunk HTTP Event Collector (prism-dtu-splunk-hec)

**Real API surface:**

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `https://{host}:8088/services/collector/event` | POST | Single event or batch ingestion |
| `https://{host}:8088/services/collector/raw` | POST | Raw text ingestion |

Auth: `Authorization: Splunk {token}` header.

**In-scope for DTU:**
- POST event(s): validates `Authorization: Splunk {token}` header, validates HEC JSON structure (`event` field required), returns `{"text":"Success","code":0}`.
- Index routing: `index` field in payload stored for assertion; DTU does not enforce index existence.
- `sourcetype` stored for assertion.
- Batch: accepts multiple JSON objects in a single POST body (newline-delimited JSON).
- 400 with `{"text":"Invalid data format","code":6}` for malformed payload.
- 403 for missing/invalid token.
- 429 (configurable).

**Fidelity level: L2 (stateful)**
- Stateful for received event capture and rate-limit counter.
- HEC response codes are the key fidelity axis (code 0=success, code 6=invalid format, code 7=mismatched token).

**Size estimate:** ~300-400 lines Rust (axum), 2 route handlers, 3 fixture files, 1 event capture store.

---

### 3.7.3 Elasticsearch Bulk API (prism-dtu-elasticsearch)

**Real API surface:**

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/{index}/_bulk` | POST | Bulk index operation (NDJSON) |
| `/{index}` | HEAD/PUT | Index auto-create check (optional) |

Auth: Basic (`Authorization: Basic {base64}`) or API key (`Authorization: ApiKey {key}`).

**In-scope for DTU:**
- Bulk POST: parses NDJSON (action-metadata + document pairs), returns bulk response with per-item `result` (`created`/`error`).
- Index auto-create semantics: first POST to a new index name succeeds (index is implicitly created); DTU tracks known indices.
- Mapping errors: if a document has a field that conflicts with the index's tracked mapping, the per-item response has `"result":"error"` with a mapping error detail. This exercises Prism's partial failure handling.
- Bulk partial failure: some items succeed, some fail — Prism must handle `errors: true` in the response even when HTTP status is 200.
- 400 for non-NDJSON payload.
- 401/403 for auth failure.
- 429 (configurable).

**Fidelity level: L2 (stateful)**
- Stateful for index registry (known indices + their first-document-implied mapping).
- Bulk partial failure is the key behavioral scenario: `prism-dtu-elasticsearch` can be configured to fail specific items in a batch (e.g., "fail every 10th item") to exercise Prism's error handling.
- Received document capture via `received_documents(index)` test API.

**Size estimate:** ~500-600 lines Rust (axum), 2 route handlers, 4 fixture files, 1 index state store.

---

### 3.7.4 OTLP/HTTP (prism-dtu-otlp)

**Real API surface:**

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/v1/logs` | POST | OTLP/HTTP log ingestion (protobuf or JSON) |

Auth: configurable `Authorization` header (Bearer token or none). gRPC is deferred until needed.

**In-scope for DTU:**
- POST `/v1/logs`: accepts `ExportLogsServiceRequest` protobuf (binary, `Content-Type: application/x-protobuf`) or JSON (`Content-Type: application/json`).
- Returns `ExportLogsServiceResponse` (empty, 200 OK) on success.
- 400 for malformed protobuf (protobuf parse error).
- 429 with `Retry-After` for rate-limit simulation.
- 503 (configurable) for collector unavailable.
- Received request capture via `received_requests()` test API (returns decoded request payloads).

**Fidelity level: L2 (stateful)**
- Stateful for received request capture and rate-limit counter.
- OTLP schema pinned to `opentelemetry-proto` version matching Prism's log forwarder dependency.
- JSON fallback validated against the same OTLP schema.

**Size estimate:** ~350-450 lines Rust (axum + prost), 1 route handler, 2 fixture files, 1 request capture store. Requires `prost`-generated OTLP types (compile-time, from bundled `.proto` files).

---

## 4. Delivery Model

### Recommendation: In-Process Crate (Hybrid Optional)

**Recommended delivery:** 14 Rust crates as dev-dependencies (Burst 5.5a expanded model):

**Shared infrastructure:**
- `prism-dtu-common` — shared test infrastructure; depended on by all 13 per-surface crates

**Sensor crates (4):**
- `prism-dtu-crowdstrike`, `prism-dtu-claroty`, `prism-dtu-cyberint`, `prism-dtu-armis`

**Actions crates (3):**
- `prism-dtu-slack`, `prism-dtu-pagerduty`, `prism-dtu-jira`

**Infusions crates (2):**
- `prism-dtu-threatintel`, `prism-dtu-nvd`

**Log-forwarding crates (4):**
- `prism-dtu-datadog`, `prism-dtu-splunk-hec`, `prism-dtu-elasticsearch`, `prism-dtu-otlp`

**Non-DTU test infrastructure (documented here, not separate crates):**
- **SMTP:** `mailpit` container (pre-built binary / Docker); `lettre` SMTP client connects to mailpit in integration tests. No custom Rust clone needed.
- **Syslog receiver:** Provided in `prism-dtu-common` as `SyslogReceiver` — binds a UDP or TCP socket, collects RFC 5424 messages, exposes via `received_messages()` test API. Reused by both action syslog tests and log-forward syslog tests.
- **Generic webhook receiver:** Provided in `prism-dtu-common` as `WebhookReceiver` — binds an Axum server on a random port, captures all POST bodies, exposes via `received_payloads()` test API. Reused by both webhook action tests and generic webhook forwarder tests.

**`prism-dtu-common` — shared test infrastructure crate (expanded in Burst 5.5a):**

`prism-dtu-common` provides the building blocks shared across all 13 per-surface DTU clones:

- **`BehavioralClone` trait** — a common interface that each per-surface crate implements. Defines `start()`, `reset()`, `configure()`, and `bound_addr()` methods so integration test harnesses can manage the DTU fleet uniformly.
- **`LatencyLayer`** — a tower middleware layer that injects configurable artificial latency into DTU responses.
- **`FailureLayer`** — a tower middleware layer that returns configured HTTP error codes (429, 500, 503, 401) for failure injection testing, driven by `POST /dtu/configure` payloads.
- **`fixture_loader()`** — loads JSON fixture files from `fixtures/` directories inside each per-surface crate.
- **`SyslogReceiver`** — generic RFC 5424 syslog receiver (UDP + TCP). Used by action syslog tests AND log-forward syslog tests. No per-surface crate needed.
- **`WebhookReceiver`** — generic HTTP receiver (Axum, random port). Used by webhook action tests AND generic webhook forwarder tests. No per-surface crate needed.

Each DTU crate starts an `axum` HTTP server bound to a random port on `127.0.0.1` at test
initialization. The bound address is passed to Prism's sensor spec as `base_url` via test harness
configuration. All DTU servers run in the same process as the integration tests (spawned as
`tokio::task`s within the test runtime). No Docker, no external process, no port conflicts.

**Rationale for in-process:**

- Prism is already a Tokio-based Rust workspace. Axum DTU servers are native Tokio tasks — no
  process boundary, no exec permission required during CI.
- Tooling selection (`tooling-selection.md`) establishes `just` as the task runner. The
  `just integration-test` target can start the DTU fleet inline without docker-compose.
- Startup time < 50ms per DTU server (axum binds immediately). Total integration test setup time
  well within the 60-second soft target.
- No Docker daemon dependency in CI runners — removes a significant CI infrastructure requirement.
- DTU crates are `#[cfg(test)]` or `[dev-dependencies]` only — zero binary size impact on the
  production `prism` binary.

**Hybrid option (available but not default):**
A `docker-compose.integration.yml` can be provided alongside the in-process crates for teams
that want process-isolated DTU servers or want to test Prism against the DTU fleet from an
external process (e.g., manual integration testing during development). The docker-compose
configuration uses the same axum binaries compiled to a `dtu-runner` binary. This is explicitly
opt-in and not the CI default.

**Rejected: containerized-first.** Docker-compose as the primary delivery adds CI infrastructure
coupling, slower startup, and requires `docker` permission in sandbox environments. The in-process
model is simpler and sufficient for the integration test scenarios defined.

---

## 5. Fidelity Validation

### Fidelity Definition

A DTU is considered fidelity-valid if, for each in-scope endpoint:
1. The response JSON schema matches the documented vendor API schema (OpenAPI or captured fixture).
2. Error responses use the correct HTTP status codes and response body structure.
3. Stateful transitions behave as documented (contain → contained, tag → tag present in subsequent read).

### Capture Format

- **Read endpoint fixtures:** Captured real API responses, sanitized of real customer data,
  checked in to `crates/dtu/{sensor}/fixtures/`. Format: one JSON file per endpoint per scenario
  (happy path, empty result, paginated page 1, paginated page 2).
- **Schema validation:** Where the vendor provides an OpenAPI spec (CrowdStrike publishes one;
  Claroty schema is derivable from `.references/mcp-claroty-xdome/src/types/`), the DTU response
  generator validates its output against the schema at test time using `jsonschema` or a Rust
  equivalent. Schema validation failures in the DTU itself are CI failures.
- **Fidelity scoring:** Percentage of in-scope endpoints with validated schema + at least one
  passing error scenario. Target ≥ 85% per sensor.

### Per-Crate Fidelity Targets

| Crate | Fidelity | Key validation axes |
|-------|----------|---------------------|
| prism-dtu-crowdstrike | L4 (adversarial) | OAuth2 flow, two-step fetch, stateful writes, failure injection, pagination |
| prism-dtu-claroty | L4 (adversarial) | Bearer token auth, POST-body filtering, group_by, stateful tagging, error matrix |
| prism-dtu-cyberint | L2 (stateful) | Cookie-roundtrip auth, alert status transitions, irreversible close |
| prism-dtu-armis | L2 (stateful) | BearerStatic auth, AQL pass-through, timestamp fallback fixture, tagging |
| prism-dtu-slack | L2 (stateful) | Webhook POST, Block Kit payload shape, 429+Retry-After, message_ts in response |
| prism-dtu-pagerduty | L3 (behavioral) | Events API v2 stateful: create→ack→resolve, dedup keys, severity mapping, 429/auth-fail |
| prism-dtu-jira | L3 (behavioral) | Create issue, add comment, status machine transitions, field validation, Basic+OAuth auth |
| prism-dtu-threatintel | L2 (stateful) | API key auth, rate limits, IP/domain/hash lookup responses, multi-source score shape |
| prism-dtu-nvd | L2 (stateful) | CVE fetch, bulk fetch, cache-miss semantics, API key vs unauthenticated rate limits |
| prism-dtu-datadog | L2 (stateful) | Batched log ingestion, API key header, 413/429 handling |
| prism-dtu-splunk-hec | L2 (stateful) | Token auth, index routing, batched events, HEC ack response shape |
| prism-dtu-elasticsearch | L2 (stateful) | Bulk API, index auto-create, mapping errors, partial failure responses |
| prism-dtu-otlp | L2 (stateful) | OTLP/HTTP protobuf, 400/429/503 handling (gRPC deferred) |

### DTU Validator Process

A `dtu-validator` CI job runs on the post-merge pipeline against captured fixture sets:

1. DTU server started for each surface (all 13 per-surface crates).
2. Validator replays each fixture scenario against the running DTU.
3. DTU response is compared against the fixture schema: field presence, field types, required
   field completeness.
4. Stateful scenarios are replayed in order (write then read) and the state transition is verified.
5. Fidelity score computed: `(endpoints passing schema check) / (total in-scope endpoints) * 100`.
6. Any DTU scoring below 85% blocks Phase 4 progression.

### Maintenance

When a vendor API changes (new field, status code change, pagination format change):
1. Update the fixture JSON files in `crates/dtu/{sensor}/fixtures/`.
2. Update the DTU route handler to match the new schema.
3. DTU validator CI job catches discrepancies before they reach integration tests.
4. Schema version is pinned in the fixture filename: `devices-v1.json`, `devices-v2.json`.

---

## 6. Integration with Test Infrastructure

### VP-033 — Audit Buffer Integration Test

**Property:** RocksDB write completes before sensor delivery attempt.

VP-033 requires a full audit-intent → sensor HTTP call → audit-outcome cycle. The DTU CrowdStrike
server provides the sensor HTTP endpoint. The integration test:
1. Starts DTU CrowdStrike on a random port.
2. Configures Prism with `base_url = "http://127.0.0.1:{port}"` and fake credential reference.
3. Issues a write query (`contain` on a fixture device).
4. Asserts: `WRITE_INTENT` audit record exists in RocksDB before the DTU server receives the HTTP call.
   The DTU's request log (in-memory, accessible via test API) provides the precise arrival timestamp.
5. Asserts: `WRITE_OUTCOME` record exists after DTU returns 200.

This test cannot be implemented against a mock (the audit timing relative to the actual HTTP call
is what is being verified) or against real CrowdStrike (non-deterministic, no real-time audit inspection).

### VP-036 — SessionContext Drop Integration Test

**Property:** SessionContext is dropped before error propagation and on panic.

VP-036 requires a full query cycle that either completes normally or encounters a sensor error.
The DTU provides a configurable error mode (endpoint returns 500 on demand via the reset/configure
API). The integration test:
1. Starts DTU CrowdStrike and configures it to return 500 on Step 2 of the two-step fetch.
2. Executes a query.
3. Asserts that the SessionContext is dropped (via a drop counter or Arc weak reference check)
   before the `E-SENSOR-002` error is returned to the caller.

### Story S-6.06 (Rescoped) — DTU Common Infrastructure

S-6.06 is rescoped to deliver `prism-dtu-common` only (the shared harness), including:
- `BehavioralClone` trait, `LatencyLayer`, `FailureLayer`, `fixture_loader`.
- `SyslogReceiver` — generic RFC 5424 receiver (UDP + TCP). Covers syslog action and log-forward syslog tests.
- `WebhookReceiver` — generic HTTP POST capture server. Covers webhook action and generic forwarder tests.
- `POST /dtu/configure` protocol shared by all per-surface crates.
- `just dtu-validate` and `just integration-test` target stubs.

### Stories S-6.07–S-6.19 — Per-Surface DTU Crates

13 stories (one per per-surface crate), written by story-writer in Burst 5b:

| Story | Crate | Category |
|-------|-------|----------|
| S-6.07 | prism-dtu-crowdstrike | Sensor |
| S-6.08 | prism-dtu-claroty | Sensor |
| S-6.09 | prism-dtu-cyberint | Sensor |
| S-6.10 | prism-dtu-armis | Sensor |
| S-6.11 | prism-dtu-slack | Action |
| S-6.12 | prism-dtu-pagerduty | Action |
| S-6.13 | prism-dtu-jira | Action |
| S-6.14 | prism-dtu-threatintel | Infusion |
| S-6.15 | prism-dtu-nvd | Infusion |
| S-6.16 | prism-dtu-datadog | Log forwarding |
| S-6.17 | prism-dtu-splunk-hec | Log forwarding |
| S-6.18 | prism-dtu-elasticsearch | Log forwarding |
| S-6.19 | prism-dtu-otlp | Log forwarding |

### Story S-0.02 — Developer Toolchain

S-0.02 defines the `just` taskfile. It must include:
- `just integration-test` — starts all 13 per-surface DTU servers (via in-process harness), runs
  `cargo test --test integration` (or a dedicated integration test binary), reports results.
- `just dtu-validate` — runs fidelity validation against full 14-crate DTU fleet.
- `just dtu-start` — starts DTU fleet standalone for manual development use.
- `just mailpit-start` — starts mailpit container for SMTP integration tests.

### Action Delivery Integration Tests

Action framework integration tests require DTU crates and alternative infra:

| Action destination | Test infrastructure | Integration test example |
|-------------------|---------------------|--------------------------|
| Slack webhook | `prism-dtu-slack` | Assert Block Kit payload shape, confirm 429 triggers retry |
| PagerDuty | `prism-dtu-pagerduty` | Assert incident create→ack→resolve lifecycle |
| Jira | `prism-dtu-jira` | Assert issue created, comment added, status transition |
| SMTP | mailpit container | Assert email delivered, HTML template rendered |
| Syslog | `SyslogReceiver` in prism-dtu-common | Assert CEF/LEEF/JSON format, assert correct port |
| Webhook | `WebhookReceiver` in prism-dtu-common | Assert POST body matches rendered template |

### Log Forwarding Integration Tests

Log forwarding integration tests require the 4 log-forwarding DTU crates:

| Destination | DTU crate | VP / integration test |
|------------|-----------|----------------------|
| Datadog Logs API | prism-dtu-datadog | Assert batch structure, API key header, 429 retry |
| Splunk HEC | prism-dtu-splunk-hec | Assert token auth, index/sourcetype routing, HEC ack |
| Elasticsearch Bulk | prism-dtu-elasticsearch | Assert index routing, partial failure handling |
| OTLP/HTTP | prism-dtu-otlp | Assert protobuf encoding, 400/429/503 handling |
| Syslog forwarder | SyslogReceiver (common) | Shared with action syslog test |
| Webhook forwarder | WebhookReceiver (common) | Shared with webhook action test |

---

## 7. Credential Model for DTU

**DTU must not require real credentials. All AD-017 protections apply.**

The DTU credential contract:
- DTU HTTP servers accept any `Authorization` header value (Bearer or Cookie) without cryptographic
  validation. The DTU does not check token authenticity — it accepts any non-empty value as valid.
- To exercise auth failure paths, the DTU exposes a configure endpoint:
  `POST /dtu/configure` with `{ "auth_mode": "reject" }` causes subsequent requests to return 401.
- Integration test fixtures use dummy credential references:

  ```toml
  # test harness prism.toml (not checked in as real config)
  [clients.test_client.sensors.crowdstrike]
  credential = { source = "env", var = "DTU_FAKE_TOKEN" }
  ```

  `DTU_FAKE_TOKEN` is set to any non-empty string (e.g., `"dtu-test-token-not-real"`) in the
  test environment. This value never reaches a real CrowdStrike endpoint.

- Credential references in integration test fixtures must be recognizably fake:
  - Environment variable names must start with `DTU_` prefix.
  - Keyring entries for DTU must use a keyring service name of `prism-dtu-{sensor}`.
  - These naming conventions prevent accidental use of DTU fixtures in a production Prism instance.

- The `prism-credentials` crate resolves the fake reference, passes it to `SensorAuth`, which
  passes it to the DTU server. The DTU accepts it. The full credential resolution path is exercised
  without any real secret value.

---

## 8. Explicit Out-of-Scope

The DTU will NOT:

- **Simulate vendor detection logic.** CrowdStrike's detection engine, Armis's risk scoring
  algorithm, Claroty's OT protocol analysis — these are vendor proprietary logic. The DTU returns
  canned scores and detection data; it does not reimplement vendor ML models.
- **Replicate vendor full datasets.** DTU fixtures contain 10–100 synthetic records per endpoint,
  sufficient for query logic tests. They do not attempt to represent a real customer's asset inventory
  or alert history at production scale.
- **Test performance at real-world scale.** DTU does not simulate thousands of concurrent devices
  or high-frequency alert streams. Performance testing is out of scope for Phase 4; it is
  post-v1 concern.
- **Validate Prism's business logic.** Unit tests (Kani proofs, proptest properties) own that
  surface. The DTU exercises the I/O boundary — HTTP wire format, schema parsing, error propagation —
  not the pure core logic that is already formally verified.
- **Cover vendor features not used by Prism v1.** Endpoint coverage is strictly bounded to the
  tables declared in each sensor's TOML spec file. Any future sensor API capability added to the
  TOML spec requires a corresponding DTU update, but is not in scope for S-6.06.

---

## 9. Fidelity Acceptance Criteria for Phase 4 Gate

Before Phase 4 (Wave 0 implementation) begins, the following must be true:

| Criterion | Definition |
|-----------|------------|
| Full endpoint coverage | Every in-scope endpoint per Section 3 has a route handler in the corresponding DTU crate |
| Schema validation passing | DTU fidelity validator scores ≥ 85% per crate on the post-merge CI job — all 13 per-surface crates |
| Error injection verified | Each listed error scenario (Sections 3.1–3.7) fires the correct Prism error code in integration test |
| VP-033 green | Audit-before-write ordering property passes against DTU CrowdStrike |
| VP-036 green | SessionContext drop property passes against DTU CrowdStrike |
| Action delivery tests green | Slack, PagerDuty, Jira delivery integration tests pass against respective DTU crates |
| Log forwarding tests green | Datadog, Splunk HEC, Elasticsearch, OTLP forwarding integration tests pass against respective DTU crates |
| Enrichment lookup tests green | ThreatIntel and NVD infusion integration tests pass against respective DTU crates |
| SMTP test green | Email action integration test passes against mailpit container |
| Syslog / webhook receivers green | `SyslogReceiver` and `WebhookReceiver` pass both action and log-forward integration tests |
| Integration test runtime | `just integration-test` completes in < 90 seconds (soft target; hard limit 180 seconds — expanded for 14-crate fleet) |
| No real credentials | CI passes with `DTU_FAKE_TOKEN=dtu-not-real` and zero real external-service credentials in environment |
| Write state transitions | Contain/tag/acknowledge state transitions verified in DTU state store after write operations |
| Action state transitions | PagerDuty incident lifecycle (create→ack→resolve) and Jira status machine (Open→In Progress→Done) verified in DTU state store |

---

## 10. Risks and Open Questions

### R-DTU-001: Cyberint API documentation gap

**Risk:** Cyberint does not publish a public OpenAPI specification. The sensor TOML spec was
derived from internal API discovery. The DTU fixture schemas may drift from the real API without
a reference spec to validate against.

**Mitigation:** Use response capture from a real Cyberint instance (if accessible through an MSSP
test tenant) to generate golden fixtures. If no test tenant is available, document the fixture as
"best-effort from API docs" and add a TODO for first-use validation.

**Owner to resolve:** Human architect or MSSP Cyberint account team.

### R-DTU-002: Armis AQL passthrough fidelity

**Risk:** The DTU accepts AQL strings without parsing them. If the TOML spec's push-down filter
serialization to AQL is incorrect, the unit tests may not catch it (the DTU always returns the
same fixture regardless of the AQL string content).

**Mitigation:** Add a DTU AQL capture mode: log all received AQL strings to an in-memory list
accessible via the test API. Integration tests can assert that the expected AQL query was
transmitted, even if the DTU does not parse it.

**Status:** Design decision deferred to S-6.06 story author.

### R-DTU-003: CrowdStrike two-step pagination state complexity

**Risk:** The CrowdStrike two-step fetch pipeline (IDs then details) with cursor pagination creates
non-trivial session state in the DTU. If the DTU's session registry has memory leaks or incorrect
cleanup, it could cause test flakiness under concurrent integration test runs.

**Mitigation:** DTU session state is bounded (maximum 1,000 session entries, LRU eviction). Each
integration test invokes the reset endpoint before use. Sessions are keyed by a test-provided
`X-DTU-Session-Id` header so concurrent tests use isolated state stores.

**Status:** Must be resolved in S-6.06 implementation.

### R-DTU-004: Claroty reference type drift

**Risk:** The type definitions in `.references/mcp-claroty-xdome/src/types/claroty.ts` were
captured at a point in time. The Claroty xDome API may have evolved since the reference was
ingested.

**Mitigation:** Treat the TypeScript types as a starting point only. First real-environment
validation against a Claroty test tenant should compare DTU fixture schemas against actual API
responses and update fixtures accordingly.

**Owner to resolve:** MSSP Claroty account team, during Phase 4 early testing.

### R-DTU-005: Maintenance burden on API evolution

**Risk:** Vendors evolve their APIs. CrowdStrike regularly releases new API versions; Armis
has versioned their API. Without a process for DTU maintenance, the DTU drifts from the real
API and integration tests begin to test an outdated contract.

**Mitigation:** The fidelity validator CI job (Section 5) catches schema drift when fixtures are
updated. The 85% fidelity threshold is the enforcement mechanism. A team policy requiring DTU
fixture updates when sensor TOML specs are updated closes the loop. Document this as a
contribution rule in `CONTRIBUTING.md` for the Prism repository.

### R-DTU-006: Slack Block Kit schema drift

**Risk:** Slack periodically deprecates Block Kit element types or changes required fields in
webhook payloads. The `prism-dtu-slack` fixture schema may diverge from the live Slack API
without triggering a test failure if Slack is lenient about extra fields.

**Mitigation:** Pin the Block Kit schema version used in fixtures. When Slack announces Block
Kit deprecations, update fixtures and the DTU validator rule for that element type.

**Owner to resolve:** Platform engineer on Burst 5b delivery of S-6.11.

### R-DTU-007: PagerDuty Events API v2 version changes

**Risk:** PagerDuty has historically made breaking changes to Events API v2 payload shapes
(severity field naming, dedup key max length). The `prism-dtu-pagerduty` L3 behavioral model
must track the current v2 contract.

**Mitigation:** Capture a reference payload from the PagerDuty Events API v2 documentation
at fixture creation time. Version the fixture file (e.g., `enqueue-v2-2026.json`). Flag DTU
update as required when PagerDuty announces API changes via their developer changelog.

**Owner to resolve:** Platform engineer on S-6.12.

### R-DTU-008: Jira Cloud API tier restrictions

**Risk:** Jira Cloud restricts some REST API v3 endpoints to paid plans. The free tier may
not support OAuth 2.0 (three-legged) or certain issue transition APIs. If the `prd-dtu-jira`
clone is designed against a paid-tier Jira and the MSSP uses a free-tier Jira, fixture
schemas may diverge.

**Mitigation:** Design `prism-dtu-jira` against the published Jira Cloud REST API v3
documentation (not captured from a live instance). Use Basic auth (email + API token) as
the primary auth model — it is available on all Jira Cloud plans. OAuth path is an optional
fixture; flag in the DTU README that OAuth 2.0 requires a paid Jira plan.

**Owner to resolve:** Architect, during S-6.13 scoping.

### R-DTU-009: NVD API rate limits

**Risk:** The NVD API enforces strict rate limits: 5 requests per 30 seconds without an API
key, 50 requests per 30 seconds with a key. Integration tests that hammer the real NVD API
would be throttled. The `prism-dtu-nvd` must accurately simulate 429 behavior including the
`Retry-After` response header with a wait time that matches NVD's documented behavior.

**Mitigation:** `prism-dtu-nvd` implements rate-limit state per virtual API key (keyed by
`apiKey` query parameter). Unauthenticated requests use a separate 5/30s bucket. The `POST
/dtu/configure` endpoint allows tests to pre-exhaust the rate limit bucket to simulate
throttling. No real NVD calls in CI.

**Status:** Must be implemented in S-6.15.

### R-DTU-010: Elasticsearch version incompatibility

**Risk:** The Elasticsearch Bulk API response shape differs between Elasticsearch 7.x, 8.x,
and OpenSearch 1.x/2.x. The `prism-dtu-elasticsearch` may target one version while Prism's
production deployment uses another, causing integration tests to pass against the wrong schema.

**Mitigation:** Design the DTU against Elasticsearch 8.x Bulk API as the primary target.
Provide separate fixture sets for ES 7.x and OpenSearch variants in the `fixtures/` directory.
The `POST /dtu/configure` API accepts a `version_profile` parameter to switch fixture sets.

**Owner to resolve:** Platform engineer on S-6.18.

### R-DTU-011: OTLP protobuf schema versioning

**Risk:** OTLP defines its wire format via protobuf schemas that evolve across OpenTelemetry
specification versions (e.g., 0.9 → 1.0 → 1.3). The `prism-dtu-otlp` must match the
protobuf version that Prism's `otlp` log forwarder uses. If `prost` serializes using a newer
schema than the DTU expects, deserialization in the DTU will fail silently.

**Mitigation:** Pin the OTLP schema version in `prism-dtu-otlp` to match the `opentelemetry-proto`
crate version used by Prism's forwarder. Include the protobuf schema version in the crate's
`CHANGELOG.md`. DTU update is required when the forwarder bumps its proto dependency.

**Status:** Must be co-ordinated with the log-forwarding implementer during S-6.19.

---

## 11. Existing DTU Work

Scanning `.factory/` and `.references/` for existing DTU infrastructure:
- No existing DTU crates found in `.factory/` or the workspace.
- `.references/mcp-claroty-xdome/` contains the Claroty integration but is a production TypeScript
  MCP server, not a test clone. Its type definitions (`src/types/claroty.ts`) and error handling
  (`src/utils/errors.ts`) are valuable bootstrap material for the Claroty DTU fixture schemas and
  error simulation design.
- No existing mock servers, fixture JSON files, or test HTTP servers were found in any of the
  9 reference repositories.

**Conclusion:** S-6.06 is a greenfield implementation. The Claroty TypeScript types are the only
bootstrapping artifact available.

---

## Cross-Reference

| If you need... | Read together with |
|---|---|
| Write endpoint details (verbs, risk tiers, SQL tables) | write-operations.md § Risk Classification Table |
| Sensor auth patterns (OAuth2, Cookie, Bearer) | sensor-adapters.md § Authentication Patterns |
| Credential model detail | security-architecture.md § AD-017 |
| Error codes the DTU must reproduce | prd-supplements/error-taxonomy.md § AUTH, SENSOR, QUERY |
| VP-033 proof harness | verification-properties/vp-033-audit-buffer-write-ordering.md |
| VP-036 proof harness | verification-properties/vp-036-session-context-drop.md |
| Developer toolchain (just targets) | S-0.02 story |
| DTU implementation story | S-6.06 story |
| Wave schedule and depends_on edges | STORY-INDEX.md v1.29 |
| Option 2 change manifest | cycles/phase-2-patch/remediation-step5-option2-dtu.md |

---

## 12. DTU-First Wave Schedule — Option 2 (Decided 2026-04-20)

### 12.1 Decision

**Strategy:** Option 2 — DTU-first.

Product stories that require a DTU clone as a test fixture must list that clone in their
`depends_on` field. DTU clones are built BEFORE their product consumers. Wave assignments for
all 14 DTU stories have been distributed across waves 0–3 to enforce this ordering.

**Decision date:** 2026-04-20
**Authority:** User directive (Step 5 remediation)
**Manifest:** `.factory/cycles/phase-2-patch/remediation-step5-option2-dtu.md`
**STORY-INDEX version capturing this schedule:** v1.29

### 12.2 Alternative Considered and Rejected

**Option 1 — Parallel (no ordering constraint):** Treat DTU clones as test infrastructure that
can be built concurrently with or after product stories. DTU stories are all placed in wave 0
and have no `blocks:` edges on product stories.

**Rejected because:** Prism's TDD discipline requires that integration tests for a product story
are written and pass from the start of that story's implementation sprint. Integration tests for
sensor, action, infusion, and log-forwarding consumers need a running DTU fixture server to be
executable. If the DTU clone does not exist when the product story begins, the integration tests
cannot run, breaking the TDD loop. Option 2 eliminates this gap by guaranteeing fixtures exist
before their consumers are scheduled.

### 12.3 Wave Distribution

All 14 DTU stories are distributed across waves 0–3. S-6.04 (credential CLI) and S-6.05
(migrate-storage) are NOT DTU clones — they remain wave 6.

| DTU Story | Crate | Wave | Blocks (product consumers) | Rationale |
|-----------|-------|------|---------------------------|-----------|
| S-6.06 | prism-dtu-common | 0 | (other DTU stories only) | Shared harness; prerequisite for all 13 per-surface crates |
| S-6.14 | prism-dtu-threatintel | 0 | S-1.14, S-5.06 | Blocks wave-1 S-1.14; must precede it |
| S-6.15 | prism-dtu-nvd | 0 | S-1.14, S-5.06 | Blocks wave-1 S-1.14; must precede it |
| S-6.07 | prism-dtu-crowdstrike | 1 | S-3.06, S-3.07 | Blocks wave-3 consumers |
| S-6.08 | prism-dtu-claroty | 1 | S-3.02 | Blocks wave-3 consumer |
| S-6.09 | prism-dtu-cyberint | 1 | S-3.02 | Blocks wave-3 consumer |
| S-6.10 | prism-dtu-armis | 1 | S-3.02 | Blocks wave-3 consumer |
| S-6.11 | prism-dtu-slack | 2 | S-4.08, S-5.06 | Blocks wave-4 and wave-5 consumers |
| S-6.12 | prism-dtu-pagerduty | 2 | S-4.08, S-5.06 | Blocks wave-4 and wave-5 consumers |
| S-6.13 | prism-dtu-jira | 2 | S-4.08, S-5.06 | Blocks wave-4 and wave-5 consumers |
| S-6.16 | prism-dtu-datadog | 3 | S-5.09 | Blocks wave-5 consumer |
| S-6.17 | prism-dtu-splunk-hec | 3 | S-5.09 | Blocks wave-5 consumer |
| S-6.18 | prism-dtu-elasticsearch | 3 | S-5.09 | Blocks wave-5 consumer |
| S-6.19 | prism-dtu-otlp | 3 | S-5.09 | Blocks wave-5 consumer |

### 12.4 Product Stories Affected

Seven product stories received `depends_on` additions pointing to their required DTU clones.

| Product Story | Wave | DTU Dependencies Added | Purpose |
|---------------|------|------------------------|---------|
| S-1.14 (infusion-specs) | 1 | S-6.14, S-6.15 | Integration tests for ThreatIntel and NVD infusion parsing require fixture servers |
| S-3.02 (query-materialization) | 3 | S-6.08, S-6.09, S-6.10 | Sensor query materialization tests exercise Claroty, Cyberint, and Armis adapters |
| S-3.06 (prismql-write-parser) | 3 | S-6.07 | Write parser tests exercise CrowdStrike write endpoint schema |
| S-3.07 (write-execution) | 3 | S-6.07 | Three-gate write safety pipeline test requires CrowdStrike DTU stateful writes |
| S-4.08 (action-delivery) | 4 | S-6.11, S-6.12, S-6.13 | Action delivery integration tests for Slack, PagerDuty, and Jira channels |
| S-5.06 (action-infusion-tools) | 5 | S-6.11, S-6.12, S-6.13, S-6.14, S-6.15 | MCP tool integration tests for all action and infusion channels |
| S-5.09 (external-log-forwarding) | 5 | S-6.16, S-6.17, S-6.18, S-6.19 | Log forwarder integration tests for all four log-forwarding destinations |

### 12.5 Dependency Chain Invariant

Every DTU → product depends_on edge satisfies: **DTU wave < product consumer wave**.

Full verification table (no cycles):

| DTU Story | DTU Wave | Product Consumer | Consumer Wave | Wave Diff | Status |
|-----------|----------|-----------------|---------------|-----------|--------|
| S-6.14 | 0 | S-1.14 | 1 | +1 | OK |
| S-6.15 | 0 | S-1.14 | 1 | +1 | OK |
| S-6.14 | 0 | S-5.06 | 5 | +5 | OK |
| S-6.15 | 0 | S-5.06 | 5 | +5 | OK |
| S-6.07 | 1 | S-3.06 | 3 | +2 | OK |
| S-6.07 | 1 | S-3.07 | 3 | +2 | OK |
| S-6.08 | 1 | S-3.02 | 3 | +2 | OK |
| S-6.09 | 1 | S-3.02 | 3 | +2 | OK |
| S-6.10 | 1 | S-3.02 | 3 | +2 | OK |
| S-6.11 | 2 | S-4.08 | 4 | +2 | OK |
| S-6.12 | 2 | S-4.08 | 4 | +2 | OK |
| S-6.13 | 2 | S-4.08 | 4 | +2 | OK |
| S-6.11 | 2 | S-5.06 | 5 | +3 | OK |
| S-6.12 | 2 | S-5.06 | 5 | +3 | OK |
| S-6.13 | 2 | S-5.06 | 5 | +3 | OK |
| S-6.16 | 3 | S-5.09 | 5 | +2 | OK |
| S-6.17 | 3 | S-5.09 | 5 | +2 | OK |
| S-6.18 | 3 | S-5.09 | 5 | +2 | OK |
| S-6.19 | 3 | S-5.09 | 5 | +2 | OK |

No cycles detected. DTU stories depend only on S-0.02 (wave 0) or S-6.06 (wave 0). No path
exists from any DTU story back to itself through product story dependencies. The dependency
graph remains a DAG.

### 12.6 VP Anchoring

VP-033 (audit-buffer-write-before-delivery) and VP-036 (session-context-drop) both anchor on
`prism-dtu-crowdstrike` (S-6.07). Both VPs require a full sensor HTTP cycle to execute. The
DTU-first schedule guarantees S-6.07 exists before S-3.06 and S-3.07 (which are the stories
where VP-033 and VP-036 integration harnesses will run). Reference: VP-INDEX v1.5.

---

## Changelog

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-16 | architect | Initial DTU assessment. Scope: 14 clones (sensors 4, actions 3, infusions 2, log-forwarding 4, common 1). Per-surface scope matrices, fidelity taxonomy, delivery model, credential model, risks, VP-033/VP-036 integration. Status: draft. |
| 1.1 | 2026-04-20 | architect | Added Section 12: Option 2 DTU-first wave schedule (decided 2026-04-20). Frontmatter: status draft → approved, version 1.0 → 1.1, inputs expanded to include prd.md / STORY-INDEX.md / STATE.md, dtu_strategy and dtu_strategy_decided fields added. Captures wave distribution (waves 0–3), 7 product story depends_on additions, dependency chain DAG verification, VP-033/VP-036 anchoring. Flips STATE.md dtu_assessment: in_progress → COMPLETE. |
