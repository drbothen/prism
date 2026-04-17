---
document_type: architecture-section
level: L3
section: "dtu-assessment"
version: "1.0"
status: draft
producer: architect
timestamp: 2026-04-16T20:00:00
phase: 3-patch
inputs: [write-operations.md, sensor-adapters.md, security-architecture.md, verification-properties/VP-INDEX.md, STATE.md]
traces_to: ARCH-INDEX.md
---

# DTU Assessment: Prism Sensor Behavioral Clones

> **Produced:** Phase 3 patch cycle. This assessment was required at Phase 1 (P1-06) and was
> discovered missing at the Phase 3 → Phase 4 gate. `STATE.md` already records
> `dtu_required: true`. This document provides the formal justification and per-sensor
> scope matrix that drives story S-6.06 and the `/vsdd-factory:dtu-creation` execution in Burst 7.

---

## 1. Decision Summary

**DTU_REQUIRED: true**

Prism's integration test suite contains two P0 integration tests (VP-033 and VP-036) whose
correctness depends on real sensor HTTP behavior: response schema fidelity, error codes, and
stateful transitions. Prism's write operations (AD-022) introduce additional test surface —
the audit-intent-before-write ordering and the dry-run/confirmation token lifecycle cannot be
verified end-to-end without a sensor that responds to HTTP calls in a manner structurally
equivalent to the real APIs. Running these tests against real CrowdStrike, Claroty, Cyberint,
or Armis instances is not feasible in CI (see Section 2). DTU (Deterministic Test Utility)
behavioral clones are the explicit architectural solution: HTTP servers that implement the
documented sensor API surface with full response schema, realistic error injection, and
enough stateful behavior to exercise Prism's write pipeline.

**Artifact dependencies:**

| Artifact | Dependency on DTU |
|----------|-------------------|
| VP-033 | Audit buffer integration test (prism-audit) requires a sensor adapter to complete an HTTP cycle so the pre/post audit write ordering can be verified |
| VP-036 | SessionContext drop test (prism-operations) requires a full query cycle against a sensor endpoint |
| S-6.06 | DTU sensor stubs story — builds the four clone servers; this assessment is the scope input |
| S-0.02 | Developer toolchain (`just integration-test`) invokes the DTU fleet |
| Story S-3.06 | Write operations integration — contain/uncontain/acknowledge cycles require DTU CrowdStrike |
| Story S-3.07 | Write safety system — three-gate pipeline exercise requires realistic sensor write responses |

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

**Fidelity level: Full**
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

**Fidelity level: Full**
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

**Fidelity level: Partial**
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

**Fidelity level: Partial**
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

## 4. Delivery Model

### Recommendation: In-Process Crate (Hybrid Optional)

**Recommended delivery:** Five Rust crates as dev-dependencies (Y1 decision — 5-crate split):

- `prism-dtu-common` (`prism-dtu-common/`) — shared test infrastructure; depended on by all four per-sensor crates
- `prism-dtu-crowdstrike` (`prism-dtu-crowdstrike/`) — depends on `prism-dtu-common`
- `prism-dtu-claroty` (`prism-dtu-claroty/`) — depends on `prism-dtu-common`
- `prism-dtu-cyberint` (`prism-dtu-cyberint/`) — depends on `prism-dtu-common`
- `prism-dtu-armis` (`prism-dtu-armis/`) — depends on `prism-dtu-common`

**`prism-dtu-common` — shared test infrastructure crate:**

`prism-dtu-common` provides the building blocks shared across all four per-sensor DTU clones:

- **`BehavioralClone` trait** — a common interface that each per-sensor crate implements (e.g., `impl BehavioralClone for CrowdStrikeDTU`). Defines `start()`, `reset()`, `configure()`, and `bound_addr()` methods so integration test harnesses can manage the DTU fleet uniformly.
- **`LatencyLayer`** — a tower middleware layer that injects configurable artificial latency into DTU responses, enabling timeout and slow-response test scenarios without per-crate reimplementation.
- **`FailureLayer`** — a tower middleware layer that intercepts requests and returns configured HTTP error codes (429, 500, 503, 401) for failure injection testing, driven by per-test `POST /dtu/configure` payloads.
- **`fixture_loader()`** — a utility function that reads JSON fixture files from `fixtures/` directories inside each per-sensor crate and deserializes them into typed fixture structs, eliminating boilerplate fixture loading code in each clone.

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

### DTU Validator Process

A `dtu-validator` CI job runs on the post-merge pipeline against captured fixture sets:

1. DTU server started for each sensor.
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

### Story S-6.06 — DTU Sensor Stubs

S-6.06 is the implementation story for all five DTU crates (Y1 5-crate model). This assessment is its scope input.
Story-writer should use this document's per-sensor endpoint lists and size estimates to scope the
story. S-6.06 delivers:
- `prism-dtu-common` with `BehavioralClone` trait, `LatencyLayer`, `FailureLayer`, and `fixture_loader`.
- Four per-sensor DTU crates with in-scope endpoint coverage, each depending on `prism-dtu-common`.
- Fixture JSON files for each sensor.
- Reset/configure API on each DTU server (using shared `FailureLayer` + `LatencyLayer`).
- `just dtu-validate` target invoking the fidelity validator.
- `just integration-test` target composing DTU startup + integration test execution.

### Story S-0.02 — Developer Toolchain

S-0.02 defines the `just` taskfile. It must include:
- `just integration-test` — starts all four DTU servers (via in-process harness), runs
  `cargo test --test integration` (or a dedicated integration test binary), reports results.
- `just dtu-validate` — runs fidelity validation against DTU fleet.
- `just dtu-start` — starts DTU fleet standalone for manual development use.

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
| Schema validation passing | DTU fidelity validator scores ≥ 85% per sensor on the post-merge CI job |
| Error injection verified | Each listed error scenario (Section 3) fires the correct Prism error code in integration test |
| VP-033 green | Audit-before-write ordering property passes against DTU CrowdStrike |
| VP-036 green | SessionContext drop property passes against DTU CrowdStrike |
| Integration test runtime | `just integration-test` completes in < 60 seconds (soft target; hard limit 120 seconds) |
| No real credentials | CI passes with `DTU_FAKE_TOKEN=dtu-not-real` and zero real sensor credentials in environment |
| Write state transitions | Contain/tag/acknowledge state transitions verified in DTU state store after write operations |

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
