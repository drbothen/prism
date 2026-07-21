---
document_type: adr
adr_id: "ADR-053"
title: "Wave-A Sensor Fidelity Remediation — OpenAPI Grounding, Armis Token-Exchange Auth, and Cyberint Dual-Surface Split"
status: proposed
date: "2026-07-20"
version: "0.1"
producer: architect
subsystems_affected: [SS-01, SS-06, SS-16, SS-17]
supersedes:
  - "ADR-028 §D1/§D2/§D5 (grounding order rules: DTU→OpenAPI as canonical; effective on ADR-053 acceptance)"
  - "ADR-028 LOCKED Armis auth_type D-747 (bearer_static→custom_via_plugin token-exchange)"
  - "ADR-028 LOCKED Cyberint auth_type D-747 (cookie_roundtrip→dual-surface split)"
  - "ADR-031 §D3-a (partial: Cyberint Alerts cookie_roundtrip model for Alerts surface only)"
superseded_by: null
amends: null
related_adrs: [ADR-026, ADR-028, ADR-031, ADR-050]
related_bcs: [BC-2.01.006, BC-2.01.008, BC-2.01.016, BC-2.01.017, BC-2.06.003]
human_authorization: "D-1889 (2026-07-20) — 'Authorize full correction'; final ADR approval gate pending before any spec/BC work begins"
wave_scope: "Wave-A only (grounding order + sensor auth models); transport/TLS (F10) is Wave-C"
---

# ADR-053: Wave-A Sensor Fidelity Remediation — OpenAPI Grounding, Armis Token-Exchange Auth, and Cyberint Dual-Surface Split

> **Human Authorization:** D-1889 (2026-07-20) — "Authorize full correction." This ADR supersedes
> multiple LOCKED architectural decisions. The supersession is executed under explicit human
> authorization, not by unilateral AI adjudication. Final ADR approval gate (human sign-off on
> this proposed ADR) is required before any spec authoring, BC amendment, or story decomposition
> begins on Wave-A sensor remediation items.

## Status

Proposed 2026-07-20, v0.1. Awaiting human approval gate before proceeding to spec/BC work.
Authored by architect under D-1889 authorization. Locks three Wave-A architectural corrections:
D1 (OpenAPI grounding order), D2 (Armis token-exchange), D3 (Cyberint dual-surface split).

---

<!-- BROWNFIELD: You MUST cite implementation evidence (file:line from crates/ or
     legacy-design-docs/) before this ADR can be accepted. Omitting evidence is a
     template-compliance failure. -->

## Context

### The Root Cause: Circular Grounding (ADR-028 §D1/§D2/§D5)

ADR-028 established the grounding rule that TOML sensor spec URLs and `auth_type` values
MUST be derived from DTU clone route registrations (`crates/prism-dtu-{sensor}/src/routes/`)
because the legacy Rust adapters contained latent URL and auth-label bugs. At the time, the
DTU clones were the most accurate available reference for real API behavior. ADR-028 §D5
further required that DTU clones MUST precede spec — a new spec entry for a URL path required
a corresponding DTU route registration first.

The live-audit findings of 2026-07-20 (D-1889; triage-capture.md) revealed the systemic
consequence of this design: EIGHT of the four sensors' spec and DTU artifacts have drifted
from real vendor API behavior, producing production failures in live-client deployments. The
root cause is circular: DTU clones themselves drifted from real APIs over time, and because
specs grounded FROM DTU clones, the drift propagated into every spec artifact, BC, and test
fixture. The triage identifies this as the systemic root cause across all eight CRIT-severity
sensor findings.

The specific fidelity failures exposed by the live audit include:

- **Armis (S-ARMIS-AUTH-FIDELITY-001, CRIT):** `auth_type = "bearer_static"` was grounded
  against the DTU clone's `Authorization: Bearer <token>` enforcement. The real Armis v1 API
  uses token-exchange (`POST /api/v1/access_token/` → short-lived access_token) and a raw-token
  Authorization header with NO "Bearer" prefix. Bearer-prefix injection causes auth failures
  against live Armis tenants; short-lived tokens cause outages when they expire with no
  refresh path.

- **Cyberint (ARCH-CYBERINT-AUTH-READJUDICATION-001, HIGH):** `auth_type = "cookie_roundtrip"`
  with a single `base_url` was grounded against the DTU clone's cookie enforcement. The real
  Cyberint API exposes TWO distinct surfaces — Alerts (`/alert` prefix, no securityScheme in
  OpenAPI) and Assets (`/asset-configuration` prefix, `access_token` apiKey cookie) — with
  different auth models and different server prefixes. A single `auth_type` + single `base_url`
  cannot serve both surfaces.

- **Cyberint Alerts phantom table (DEFECT-CYBERINT-SPEC-FIDELITY-001, CRIT):** The current
  spec declares an `incidents` table mapping `GET /api/v1/incidents` — an endpoint that does
  not exist in the Cyberint OpenAPI. This was derived from the DTU clone which fabricated the
  endpoint. Real Cyberint has no incidents object; everything is an alert.

### ADR-031 Established DTU = True-DTU, But the Circular Problem Remained

ADR-031 (DTU = True-DTU Fidelity Principle, 2026-05-29) established that DTU clones MUST
mirror real API field names, auth flows, cookie names, endpoints, and response shapes. This
was the correct direction. However, ADR-031 addressed DTU correctness going forward — it did
NOT reverse the grounding ORDER established by ADR-028 (spec grounds FROM DTU). The
consequence is that if DTU clones drift from real APIs (as they have), ADR-031 requires
fixing the DTU, but ADR-028 keeps the spec grounded on the (now-drifted) DTU. The combination
preserves the circular dependency: spec→DTU→(hoped-to-be-correct)→real-API.

The fix requires inverting the grounding direction: spec grounds FROM vendor OpenAPI; DTU is
validated AGAINST the spec/OpenAPI by the dtu-validator.

### Canonical OpenAPI Evidence

Two authoritative Cyberint OpenAPI files are available (dated 2026-06-20):
- `cyberint_alerts_openapi_06.20.2026.json` (title "Alerts", v1.0, server `/alert`, 11 paths)
- `cyberint_assets_openapi_06.20.2026.json` (title "FastAPI", v0.1.0, server `/asset-configuration`, 5 paths)

For Armis, no downloadable OpenAPI exists. Claims are web-corroborated from multiple
independent production connectors (Google Chronicle, Cortex XSOAR, Swimlane, Brinqa, Sumo
Logic, Hunters) per `demo-soc/findings/prism-armis-endpoint-plan.md` (2026-07-20).
Source file: `/Users/jmagady/Dev/test-soc/demo-soc/findings/prism-armis-endpoint-plan.md`.

For Claroty xDome, the canonical OpenAPI reference is `xdome_openapi_06.20.2026.json`.

### Human Authorization for LOCKED Decision Supersession

ADR-028 frontmatter records `locked_decisions: ["D-737 Decision 1", "D-737 Decision 4"]`.
The Armis bearer_static and Cyberint cookie_roundtrip auth values were locked by D-747
(Path-A adjudication in the PLUGIN-MIGRATION-001-D cascade). D-1889 (2026-07-20) explicitly
authorizes full correction and supersession of those locked decisions.

---

## Decision

### D1 — OpenAPI Grounding Order (supersedes ADR-028 §D1/§D2/§D5)

TOML sensor spec URL paths (`base_url`, per-table `path` fields) and `auth_type` values MUST
be derived from the canonical vendor OpenAPI specification for that sensor API. Where no
downloadable OpenAPI exists, web-corroborated documentation from multiple independent
production connectors serves as the canonical reference, with confidence flags declared inline.

The DTU clone is NOT the authoritative grounding reference for new spec authoring. The DTU
FOLLOWS the canonical spec/OpenAPI: after a spec is authored from the OpenAPI, the DTU clone
is re-cloned to match the spec's declared endpoints, auth flows, response shapes, and field
names. The dtu-validator scores DTU fidelity against the OpenAPI/spec.

**Supersession scope:** ADR-028 §D1 (URL grounding — DTU routes), §D2 (auth_type grounding —
DTU enforcement middleware), and §D5 (DTU must precede spec) are superseded for all new spec
authoring. ADR-028 clauses §D3 (OCSF parity grounding — fixture JSON), §D4 (adapter-as-
reference forbidden), §D6 (scope expansion — executed and now moot), §D7 (per-file changelog
convention), §D8 through §D13 are NOT superseded; they remain authoritative.

**Preserved by ADR-031:** ADR-031's DTU = True-DTU fidelity principle is STRENGTHENED, not
weakened, by this grounding correction. The direction for DTU correctness (DTU must mirror
real API) is now backed by a correct grounding authority (OpenAPI → spec → DTU). ADR-031
§D8-a (Armis AQL search endpoint), §D8-b (Claroty trailing slash), §D8-c (CrowdStrike multi-
region base_url) remain authoritative as permitted-divergence classifications.

**Implementation note:** For CrowdStrike and Claroty, the DTU clones are already well-aligned
with real API behavior per the live audit. The grounding-order correction primarily affects
Armis and Cyberint spec authoring for Wave-A.

**Human authorization:** D-1889 (2026-07-20). Final ADR approval gate pending.

### D2 — Armis Auth Model: token_exchange via WASM Plugin (supersedes ADR-028 LOCKED Armis, D-747)

The real Armis v1 API (`https://<tenant>.armis.com/api/v1/`) uses token-exchange
authentication, not bearer-static. The exact flow is:

1. **Acquire:** POST form-encoded `secret_key=<long-lived credential>` to
   `POST /api/v1/access_token/`. Response shape:
   `{"success":true,"data":{"access_token":"<short-lived token>","expiration_utc":"<UTC timestamp>"}}`.
2. **Use:** Inject `Authorization: <raw_access_token>` with NO "Bearer" prefix on every
   subsequent request. (Bearer-prefix injection causes HTTP 401 against live Armis v1.)
3. **Refresh:** Re-POST the secret_key when `expiration_utc` is reached or on any 401.
   There is no refresh token; the long-lived secret_key is re-used for each exchange.

The credential reference for the secret_key is `armis_secret_key` (per ADR-032 credential-ref
naming convention; env var `PRISM_CLIENTS_{ORG}_SENSORS_ARMIS_ARMIS_SECRET_KEY`).

**Implementation approach: `auth_type = "custom_via_plugin"` with `armis-token-exchange.prx`.**

This follows the CrowdStrike-oauth2 plugin precedent (crowdstrike-oauth2.prx). The
CrowdStrike-oauth2 plugin performs HTTP POST → token acquisition → cached token use → re-
exchange on expiry. The Armis token-exchange flow is structurally identical: the differences
are endpoint, form encoding (vs JSON body), and raw-vs-Bearer header injection.

A new `armis-token-exchange.prx` WASM plugin is introduced. The plugin's `acquire_token()`
implementation:
- Performs `POST /api/v1/access_token/` with form-encoded `secret_key=<credential>`
- Parses `$.data.access_token` from the JSON response
- Records `$.data.expiration_utc` for expiry-based re-exchange
- Returns the raw token to the pipeline

**Required engine change — raw-token Authorization header arm:** The current
`pipeline.rs::build_request()` injects `Authorization: Bearer <token>` for all non-cookie
auth paths. This is incorrect for Armis v1 (raw token, no prefix) and potentially for other
sensors. The engine MUST be amended to support a `raw_token` auth strategy option that injects
`Authorization: <token>` verbatim. This engine change is in-scope for the Armis auth story and
MUST land before the Armis sensor spec can be authored or validated.

**Why custom_via_plugin over a new `token_exchange` auth_type closed-enum entry:**
The `custom_via_plugin` approach (Option A, chosen) avoids expanding the `SensorAuth` closed
enum at this time. The CrowdStrike precedent demonstrates the pattern works for HTTP-at-
acquire_token scenarios. The WASM plugin sandboxes the token-exchange logic without requiring
spec-engine code changes. A native `token_exchange` auth_type (Option B) would be appropriate
if three or more sensors exhibit this pattern, but at Wave-A only Armis requires it, making
the spec-engine investment premature.

**VP assignment (DRIFT-D849-002):** The VP that `acquire_token()` makes no network calls
during spec-load applies to the Armis plugin. The plugin MUST lazy-acquire on first sensor
request, not at spec-parsing or boot time. Implementation evidence must be cited in the
plugin's `acquire_token()` before this ADR is accepted.

**Supersession scope:** ADR-028 LOCKED Armis decision (D-747, `bearer_static`) is superseded.
ADR-026 §D3's `ArmisAuth::auth_type_name() → "api_key"` (the original, pre-ADR-028 value)
was already superseded by ADR-028 §D6 to `"bearer_static"`; this ADR further supersedes the
Armis auth classification to `"custom_via_plugin"`. The distinction is moot in production code
because the legacy `ArmisAuth` struct was deleted by PLUGIN-MIGRATION-001-A; this ADR governs
the TOML spec and new WASM plugin, not a deleted Rust struct.

**Armis v1 vs v3 (no conflict):** Armis states v1/v2 are NOT deprecated. Prism's v1 targeting
is valid. v3 (true OAuth2 client-credentials, Bearer prefix, structured `/v3/assets/_search`)
could be adopted in a future wave; if adopted, it would use `auth_type = "oauth2_client_credentials"` (matching CrowdStrike). This ADR covers v1 only.

**Human authorization:** D-1889 (2026-07-20). Final ADR approval gate pending.

### D3 — Cyberint Dual-Surface Schema (supersedes ADR-028 LOCKED Cyberint D-747; ADR-031 §D3-a)

The Cyberint API exposes two distinct surfaces with different server prefixes, different auth
models, and different OpenAPI files:

| Surface | Server prefix | Auth (OpenAPI declared) | Use case |
|---------|--------------|-------------------------|----------|
| Alerts  | `/alert`     | NOT declared (no securitySchemes in OpenAPI) | Alert detection, IOC enrichment |
| Assets  | `/asset-configuration` | apiKey in cookie `access_token` | ASM inventory |

A single TOML sensor definition cannot faithfully represent both surfaces because they have
different base URLs and different auth models. Per-table `base_url`/auth overrides in the
prism-spec-engine are not implemented (see D3-b below).

**D3-a — Two sensor definitions (chosen approach):**

Two separate TOML sensor specs are introduced:
- `cyberint-alerts.sensor.toml` — Alerts surface (`/alert` prefix)
- `cyberint-assets.sensor.toml` — Assets surface (`/asset-configuration` prefix)

The existing `cyberint.sensor.toml` is superseded and deleted as part of the remediation story.

**D3-b — Per-table auth overrides deferred (Wave-B spec-engine capability):**

The prism-spec-engine `SensorSpec` schema does not currently support per-table `auth_type`
or `base_url` overrides. Adding this capability would allow a single sensor definition to
serve multiple server prefixes/auth models via table-level overrides. This is the more elegant
long-term solution but requires spec-engine schema changes, BC amendments, and a new story.
It is deferred to Wave-B. The two-definition approach requires zero spec-engine changes.

**D3-c — Cyberint Alerts auth UNCONFIRMED (precondition for story authoring):**

The Cyberint Alerts OpenAPI declares NO `securitySchemes`. External Cyberint product
documentation suggests `X-Api-Key` header authentication for the Alerts REST API, but this
is NOT confirmed in the OpenAPI file itself. Applying the Assets `cookie_roundtrip` model to
Alerts is likely wrong (different server prefix, no cookie scheme declared in Alerts OpenAPI).

**Precondition (BLOCKING):** A research-agent pass confirming the Cyberint Alerts API
authentication mechanism is required before the `cyberint-alerts.sensor.toml` spec can be
authored or before any BC amendments to BC-2.01.006 are finalized. The research-agent should:
1. Confirm the header name (expected `X-Api-Key`; alternatives: `Authorization`, `X-Token`)
2. Confirm whether the credential is a static API key (no exchange, no expiry)
3. Confirm whether both surfaces share one hostname or use separate hostnames

If confirmed as static `X-Api-Key`, auth_type = `bearer_static` (using `X-Api-Key` instead
of `Authorization` header, if the spec engine supports custom header name — if not, a
`custom_via_plugin` approach may be needed). This decision is deferred pending research.

**D3-d — Cyberint Assets auth CONFIRMED (cookie_roundtrip preserved):**

The Assets OpenAPI (`cyberint_assets_openapi_06.20.2026.json`) declares:
`securitySchemes."Access Token" = {type:apiKey, in:cookie, name:access_token}`.
This is a static API-key-as-cookie: the API key is injected as `Cookie: access_token={key}`
on every request with no login step. This matches ADR-031's `StaticCookieAuthProvider` pattern
(from the poller-express reference implementation: `cookieTransport.RoundTrip` injects
`access_token` cookie statically).

Cyberint Assets: `auth_type = "cookie_roundtrip"`, credential_ref `api_key`,
cookie name `access_token`. This is unchanged from the existing Assets-facing behavior. The
fix is isolating this to a dedicated `cyberint-assets.sensor.toml` spec file.

**Supersession scope:** ADR-028 LOCKED Cyberint decision (D-747, `cookie_roundtrip` for the
combined spec) is superseded in the sense that the combined spec is split. The Assets surface
continues to use `cookie_roundtrip`. The Alerts surface auth is PENDING research confirmation.
ADR-031 §D3-a (Cyberint Alerts cookie_roundtrip — the portion of the old §D12 that required
the DTU login flow, corrected by ADR-031) is superseded by this split.

**Human authorization:** D-1889 (2026-07-20). Final ADR approval gate pending.

### D4 — Wave-C Out-of-Scope (TLS/Transport, F10)

DEFECT-ADAPTER-TLS-XDOME-LIVE-001 (F10, CRIT): live xDome HTTPS connections fail against
the WAF profile (h1-only, no User-Agent). The fix requires transport-level changes. Any
transport fix MUST remain ADR-050-compliant: `native-tls` is forbidden; the `rustls-tls`
backend MUST be used. The allowed path is HTTP/1.1 forced with `User-Agent` injection via
reqwest builder. This is deferred to Wave-C and is explicitly out of scope for this ADR.

No spec, BC, or story for F10 may be authored before a Wave-C transport ADR is accepted.

---

## Rationale

### Why invert the grounding order?

The live-audit findings confirm ADR-028's original motivation (DTU clones were more accurate
than legacy adapters) no longer applies after plugin-migration. The legacy adapters are
deleted; the DTU clones are the only reference. But DTU clones drift because they are
maintained code, not static contracts. The vendor OpenAPI files are issued by the vendor and
represent the real API surface at a point in time — they drift only when the vendor changes
the API. The grounding order should reflect the most stable, authoritative source: vendor
OpenAPI → spec → DTU (DTU scored against OpenAPI by dtu-validator). This mirrors ADR-031's
direction (DTU must match real API) but makes it structural rather than aspirational.

### Why custom_via_plugin for Armis token-exchange?

The WASM plugin approach (D2) avoids expanding the spec-engine closed `auth_type` enum for a
single sensor. The token-exchange pattern (HTTP POST → cached token → expiry re-exchange) is
already exercised in production by the CrowdStrike-oauth2 plugin. Adding a dedicated
`token_exchange` auth_type would require: (a) spec-engine code change, (b) new enum variant
in BC-2.01.016 and E-SPEC-012, (c) new AuthProvider implementation, (d) BC amendments. The
plugin approach delivers the same behavior in Wave-A scope without those cascading changes. If
two or more additional sensors require token-exchange, the cost/benefit shifts and a native
auth_type should be evaluated.

### Why two sensor definitions for Cyberint?

The two-definition approach (D3-a) requires zero spec-engine changes, delivers clean audit
trails per surface, and isolates the less-stable Assets API (v0.1.0, "FastAPI" title) from
the more stable Alerts API (v1.0). The alternative (per-table auth overrides) requires
spec-engine schema changes, which are a separate Wave-B scope. The two-definition approach is
self-consistent with ADR-023's principle that TOML specs are declarative baselines.

---

## Consequences

### Positive

- All Wave-A spec authoring for Armis and Cyberint is grounded on vendor OpenAPI, not DTU
  circular references. Drift detection becomes dtu-validator's job (OpenAPI vs DTU), not a
  manual audit activity.
- Armis token-exchange plugin follows the established CrowdStrike-oauth2 plugin pattern.
  Once the raw-token Authorization header arm is added to pipeline.rs, Armis live-client
  auth will work correctly with short-lived token refresh.
- Cyberint split eliminates the phantom `incidents` table (DEFECT-CYBERINT-SPEC-FIDELITY-001)
  and correctly scopes the Assets auth to its own spec definition.
- ADR-031 (DTU = True-DTU) is structurally reinforced: dtu-validator now has an OpenAPI
  baseline to score against, not just the spec.

### Negative / Trade-offs

- Engine change required: pipeline.rs `build_request()` raw-token Authorization header arm
  must land before any Armis live-client spec can be validated end-to-end.
- Cyberint Alerts auth is UNCONFIRMED. A research-agent pass is a hard precondition for
  cyberint-alerts spec authoring. If the auth mechanism turns out to require a new auth_type
  (e.g., custom header-name support not in current spec-engine), additional spec-engine work
  will be required.
- Two Cyberint sensor definitions (not one) increase the config surface. Operators running
  Cyberint must configure two TOML entries with potentially different credentials.
- LOCKED decisions D-747 (Armis bearer_static; Cyberint cookie_roundtrip combined) are
  superseded. Any story, holdout scenario, or test that was authored against the old locked
  values must be audited and amended.

### Status as of 2026-07-20

Proposed. Not in effect. Awaiting human approval gate before any spec/BC work, story
decomposition, or implementation begins. The three locked decisions (ADR-028 §D1/§D2/§D5,
Armis D-747, Cyberint D-747) are still the operative constraints until this ADR is accepted.

---

## Alternatives Considered

- **Option A (chosen — grounding inversion):** Vendor OpenAPI as ground truth; DTU follows
  spec; dtu-validator scores DTU vs OpenAPI. Breaks the circular dependency permanently.

- **Option B (rejected — fix DTU then re-ground from DTU):** Keep ADR-028 §D1/§D2/§D5 intact.
  Rebuild all four DTU clones to match real APIs, then re-author specs from the corrected DTUs.
  Rejected because this preserves the circular dependency: future DTU drift will again
  propagate into specs. The live audit is evidence this happens in practice.

- **Option C (rejected — native `token_exchange` auth_type for Armis):** Add a new
  `token_exchange` variant to the closed `auth_type` enum (BC-2.01.016, E-SPEC-012), implement
  a `TokenExchangeAuthProvider` in prism-spec-engine. Rejected for Wave-A because only one
  sensor needs it and the plugin approach delivers identical behavior without spec-engine
  changes. Remains viable for Wave-B if additional sensors require token-exchange.

- **Option D (rejected — per-table auth overrides for Cyberint):** Add per-table `auth_type`
  and `base_url` override fields to `SensorSpec`. This is the cleanest long-term schema but
  requires spec-engine changes, BC amendments, and a new story. Deferred to Wave-B.

- **Option E (rejected — single Cyberint definition with cookie_roundtrip for both surfaces):**
  Keep one definition, use the Assets cookie auth for both surfaces. Rejected because the
  Alerts OpenAPI declares NO securitySchemes; applying cookie auth to the Alerts API is not
  justified by the OpenAPI evidence and would likely fail against live Cyberint tenants.

---

## Source / Origin

- **Triage capture:** `.factory/planning/findings-remediation-2026-07-20/triage-capture.md`
  (D-1889, 2026-07-20) — systemic root cause identified as ADR-028 circular grounding
- **Findings source (Armis):** `/Users/jmagady/Dev/test-soc/demo-soc/findings/prism-armis-endpoint-plan.md`
  (2026-07-20) — auth flow evidence: token-exchange flow, raw-token header, no Bearer prefix;
  corroborated by Google Chronicle + Brinqa + Cortex XSOAR + Sumo Logic + Hunters connectors
- **Findings source (Cyberint):** `/Users/jmagady/Dev/test-soc/demo-soc/findings/prism-cyberint-endpoint-plan.md`
  (2026-07-20) — two-surface OpenAPI analysis; Alerts no securitySchemes; Assets cookie apiKey
- **OpenAPI files (Cyberint):** `.factory/reference/api-specs/cyberint_alerts_openapi_06.20.2026.json`
  and `cyberint_assets_openapi_06.20.2026.json` (both dated 2026-06-20)
- **Engine location (Authorization header):** `crates/prism-sensors/src/pipeline.rs` —
  `build_request()` function; Bearer prefix hardcoded for non-cookie auth (cite at D2
  acceptance time)
- **CrowdStrike-oauth2 plugin precedent:** `crowdstrike-oauth2/` WASM plugin; acquire_token
  HTTP-POST pattern — cite specific file:line at D2 acceptance time
- **Human authorization:** D-1889 (2026-07-20); triage-capture.md Open Decision #1/#2/#3 sign-off

### §Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 0.1 | 2026-07-20 | architect | Initial proposal under D-1889 authorization |
