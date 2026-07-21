---
document_type: adr
adr_id: "ADR-053"
title: "Wave-A Sensor Fidelity Remediation — OpenAPI Grounding, Armis Token-Exchange Auth, and Cyberint Dual-Surface Split"
status: proposed
date: "2026-07-20"
modified: "2026-07-20"
version: "0.2"
producer: architect
subsystems_affected: [SS-01, SS-06, SS-16, SS-17]
supersedes:
  - "ADR-028 §D1/§D2/§D5 (grounding order rules: DTU→OpenAPI as canonical; effective on ADR-053 acceptance)"
  - "ADR-028 LOCKED Armis auth_type D-747 (bearer_static→custom_via_plugin token-exchange)"
  - "ADR-028 LOCKED Cyberint auth_type D-747 (combined cookie_roundtrip spec→dual-surface split)"
  - "ADR-031 §D3 (scope-narrowing only: single-surface cookie_roundtrip assumption narrowed to Assets; §D3-a/b Assets static-cookie contract PRESERVED; Alerts becomes separate surface definition)"
superseded_by: null
amends: null
related_adrs: [ADR-026, ADR-028, ADR-031, ADR-032, ADR-050]
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

Proposed 2026-07-20, v0.2 (revised after adversary/spec-reviewer/consistency-validator triad + Cyberint
Alerts auth research confirmation). Awaiting human approval gate before proceeding to spec/BC work.
Authored by architect under D-1889 authorization. Locks three Wave-A architectural corrections:
D1 (OpenAPI grounding order), D2 (Armis token-exchange + `header_scheme` injection), D3 (Cyberint
dual-surface split with both surfaces CONFIRMED cookie auth).

---

<!-- BROWNFIELD: You MUST cite implementation evidence (function names + behavioral anchors from
     crates/ or legacy-design-docs/) before this ADR can be accepted. Omitting evidence is a
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
consequence of this design: eight CRIT findings across all four sensors' spec and DTU artifacts
have drifted from real vendor API behavior, producing production failures in live-client
deployments. The root cause is circular: DTU clones themselves drifted from real APIs over
time, and because specs grounded FROM DTU clones, the drift propagated into every spec
artifact, BC, and test fixture. The triage identifies this as the systemic root cause across
all eight CRIT-severity sensor findings.

The specific fidelity failures exposed by the live audit include:

- **Armis (S-ARMIS-AUTH-FIDELITY-001, CRIT):** `auth_type = "bearer_static"` was grounded
  against the DTU clone's `Authorization: Bearer <token>` enforcement. The real Armis v1 API
  uses token-exchange (`POST /api/v1/access_token/` → short-lived access_token) and a raw-token
  Authorization header with NO "Bearer" prefix. Bearer-prefix injection causes auth failures
  against live Armis tenants; short-lived tokens cause outages when they expire with no
  refresh path.

- **Cyberint (ARCH-CYBERINT-AUTH-READJUDICATION-001, HIGH):** `auth_type = "cookie_roundtrip"`
  with a single `base_url` was grounded against the DTU clone's cookie enforcement. The real
  Cyberint API exposes TWO distinct surfaces — Alerts (`/alert` prefix) and Assets
  (`/asset-configuration` prefix) — with different server prefixes. A single `auth_type` +
  single `base_url` cannot serve both surfaces correctly.

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

**No-OpenAPI governance:** For sensors without a downloadable OpenAPI (Armis, CrowdStrike),
the dtu-validator scores the DTU against the web-corroborated findings document using its
Confirmed/Partial/Unconfirmed confidence tiers as the contract. Only Confirmed-tier claims
(corroborated by two or more independent production connectors) may be spec'd without
live-tenant validation. Partial-tier claims require a DTU-EXT-NNN blocker reference (§D9
of ADR-028) and a Confirmed upgrade path.

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

**Cyberint C2-class mechanical fixes (in scope for remediation story):** OpenAPI grounding
of Cyberint Alerts entails the following C2-class corrections to the spec: POST (not GET) for
`/alert/api/v1/alerts`, response extraction path `$.alerts` (not `$.data`), page/size-capped
pagination (page + size, max 100 per page, not cursor-based), and `/alert` server prefix.
These corrections are in-scope for the Cyberint Alerts remediation story.

**Implementation note:** For CrowdStrike and Claroty, the DTU clones are already well-aligned
with real API behavior per the live audit. The grounding-order correction primarily affects
Armis and Cyberint spec authoring for Wave-A.

**Human authorization:** D-1889 (2026-07-20). Final ADR approval gate pending.

### D2 — Armis Auth Model: Token-Exchange via WASM Plugin + `header_scheme` Field (supersedes ADR-028 LOCKED Armis, D-747)

The real Armis v1 API (`https://<tenant>.armis.com/api/v1/`) uses token-exchange
authentication, not bearer-static. The exact flow is:

1. **Acquire:** POST form-encoded `secret_key=<long-lived credential>` to
   `POST /api/v1/access_token/`. Response shape:
   `{"success":true,"data":{"access_token":"<short-lived token>","expiration_utc":"<UTC timestamp>"}}`.
2. **Use:** Inject `Authorization: <raw_access_token>` with NO "Bearer" prefix on every
   subsequent request. (Bearer-prefix injection causes HTTP 401 against live Armis v1.)
3. **Refresh:** Re-POST the secret_key when `expiration_utc` is reached or on any 401.
   There is no refresh token; the long-lived secret_key is re-used for each exchange.

The credential reference for the secret_key is `secret_key` (per ADR-032 credential-ref
naming convention, bare ref name with sensor segment separate; env var
`PRISM_CLIENTS_{ORG}_SENSORS_ARMIS_SECRET_KEY`). ADR-032 Armis rows that reference
`bearer_token` credential_ref go stale under this decision and must be updated in the
remediation story (see MED-1 note; `related_adrs` carries ADR-032 for tracking).

**Implementation approach: `auth_type = "custom_via_plugin"` with `armis-token-exchange.prx`.**

This follows the CrowdStrike-oauth2 plugin precedent (`crowdstrike-oauth2.prx`). The
CrowdStrike-oauth2 plugin performs HTTP POST → token acquisition → cached token use → re-
exchange on expiry. The Armis token-exchange flow is structurally identical: the differences
are endpoint, form-encoded vs JSON body, and raw-vs-Bearer header injection. The WASM plugin
handles ACQUISITION; the engine handles INJECTION (see `header_scheme` below).

**TOML wiring for Armis (`auth_plugin` field — SR-005):**

```toml
[sensor]
auth_type = "custom_via_plugin"
auth_plugin = "armis-token-exchange"   # names the registered WASM plugin
header_scheme = "raw"                  # Authorization: {token} — no Bearer prefix
credential_ref = "secret_key"          # resolved via BC-2.03.006 three-tier chain
```

The `auth_plugin` field names the registered WASM plugin. If the plugin is not registered at
spec-load time, `E-SPEC-012` is emitted per BC-2.01.016 EC-016-002 (plugin auth requires
registered plugin) and EC-016-005 (unregistered plugin → spec rejected). This matches the
existing CrowdStrike-oauth2 registration contract.

The `armis-token-exchange.prx` plugin's `acquire_token()` implementation:
- Performs `POST /api/v1/access_token/` with form-encoded `secret_key=<credential>`
- Parses `$.data.access_token` from the JSON response
- Records `$.data.expiration_utc` for expiry-based re-exchange
- Returns the raw token to the pipeline

**Header injection selection mechanism: `header_scheme` TOML field.**

The current `PipelineExecutor::build_request()` dispatches on `auth_type` with two arms:
`CookieRoundtrip → Cookie: access_token={token}` and a catch-all `→ Authorization: Bearer
{token}`. This is incorrect for Armis v1 (raw token, no prefix), and it is ambiguous for
`custom_via_plugin` — the host cannot distinguish "CrowdStrike custom_via_plugin (needs Bearer)"
from "Armis custom_via_plugin (needs raw token)" using `auth_type` alone.

The selection key is a new `header_scheme` TOML field on `SensorSpec`. Values:

| `header_scheme` value | Header injected | Use case |
|----------------------|-----------------|----------|
| `"bearer"` (default) | `Authorization: Bearer {token}` | CrowdStrike, Claroty, existing sensors |
| `"raw"` | `Authorization: {token}` (no prefix) | Armis v1 raw-token |
| `"cookie:<name>"` | `Cookie: <name>={token}` | Cyberint (both surfaces, parameterized) |

`build_request()` reads `spec.header_scheme` (defaulting to `"bearer"` for backward
compatibility) and dispatches on it. The existing `auth_type`-based cookie arm is replaced by
the `"cookie:access_token"` value in `header_scheme`. This is a single, backward-compatible
engine change: all existing sensors that do not declare `header_scheme` continue to receive
`Authorization: Bearer {token}` unchanged.

**Why `header_scheme` over a new `token_exchange` `AuthType` closed-enum variant:**
`AuthType` (spec_parser.rs) / the E-SPEC-012 canonical auth_type set governs ACQUISITION —
which AuthProvider is constructed at spec-load time (e.g., `PluginAuthProvider` for
`custom_via_plugin`). `header_scheme` governs INJECTION — how the acquired token is placed in
the HTTP request. These are orthogonal concerns. Adding a `token_exchange` variant to
`AuthType` would conflate both, require BC-2.01.016 enum amendment, E-SPEC-012 new code,
and a new `TokenExchangeAuthProvider` in prism-spec-engine — all without changing the
acquisition pattern (the WASM plugin already handles it). `header_scheme` is the correct
abstraction boundary.

A native `token_exchange` `AuthType` variant remains viable for Wave-B if two or more
additional sensors exhibit the token-exchange acquisition pattern.

**Required engine change:** `SensorSpec` gains the `header_scheme` field (with `#[serde(default)]`
defaulting to `"bearer"`). `PipelineExecutor::build_request()` switches from `auth_type`-based
dispatch to `header_scheme`-based dispatch. This engine change MUST land before the Armis
sensor spec can be authored or validated end-to-end, and is in-scope for the Armis auth story.

**VP assignment (DRIFT-D849-002):** The VP that `acquire_token()` makes no network calls
during spec-load applies to the Armis plugin. The plugin MUST lazy-acquire on first sensor
request, not at spec-parsing or boot time. Implementation evidence must be cited in the
plugin's `acquire_token()` function via behavioral anchor before this ADR is accepted.

**Supersession scope:** ADR-028 LOCKED Armis decision (D-747, `bearer_static`) is superseded.
ADR-028 §D13's consistency table row `bearer_static (Armis, Claroty)` is narrowed — see HIGH-1
annotation in §D13 below. ADR-026 §D3's `ArmisAuth::auth_type_name() → "api_key"` was already
superseded by ADR-028 §D6; the legacy struct was deleted by PLUGIN-MIGRATION-001-A and is moot.

**Armis v1 vs v3 (no conflict):** Armis states v1/v2 are NOT deprecated. Prism's v1 targeting
is valid. v3 (true OAuth2 client-credentials, Bearer prefix, structured `/v3/assets/_search`)
could be adopted in a future wave; if adopted, it would use `auth_type = "oauth2_client_credentials"` (matching CrowdStrike). This ADR covers v1 only.

**Human authorization:** D-1889 (2026-07-20). Final ADR approval gate pending.

### D3 — Cyberint Dual-Surface Schema (supersedes ADR-028 LOCKED Cyberint D-747; ADR-031 §D3 scope-narrowing)

The Cyberint API exposes two distinct surfaces with different server prefixes and different
OpenAPI files:

| Surface | Server prefix | Auth (CONFIRMED) | Use case |
|---------|--------------|-----------------|----------|
| Alerts  | `/alert`     | `Cookie: access_token=<static token>` (no login step) | Alert detection, IOC enrichment |
| Assets  | `/asset-configuration` | `Cookie: access_token=<static token>` (no login step) | ASM inventory |

Both surfaces use the same static-cookie injection mechanism. The split is required because
they have different `base_url` values (server prefixes) and different OpenAPI specs.

A single TOML sensor definition cannot faithfully represent both surfaces because they have
different base URLs. Per-table `base_url` overrides in the prism-spec-engine are not
implemented (see D3-b below).

**D3-a — Two sensor definitions (chosen approach):**

Two separate TOML sensor specs are introduced:
- `cyberint-alerts.sensor.toml` — Alerts surface (`/alert` prefix, `auth_type = "cookie_roundtrip"`, `header_scheme = "cookie:access_token"`, `credential_ref = "access_token"`)
- `cyberint-assets.sensor.toml` — Assets surface (`/asset-configuration` prefix, `auth_type = "cookie_roundtrip"`, `header_scheme = "cookie:access_token"`, `credential_ref = "access_token"`)

The existing `cyberint.sensor.toml` is superseded and deleted as part of the remediation story.

**D3-b — Per-table auth overrides deferred (Wave-B spec-engine capability):**

The prism-spec-engine `SensorSpec` schema does not currently support per-table `auth_type`
or `base_url` overrides. Adding this capability would allow a single sensor definition to
serve multiple server prefixes/auth models via table-level overrides. This is the more elegant
long-term solution but requires spec-engine schema changes, BC amendments, and a new story.
It is deferred to Wave-B. The two-definition approach requires zero spec-engine changes beyond
the `header_scheme` field introduced in D2.

**D3-c — Cyberint Alerts auth CONFIRMED (research-agent, 2026-07-20):**

Research-agent confirmation: Cyberint Alerts API uses static `Cookie: access_token=<token>`
injection on every request. The credential is a portal-generated static API key — there is
NO login round-trip, NO session exchange, and NO token expiry. This is identical to the
Assets surface mechanism and matches the `StaticCookieAuthProvider` pattern (ADR-031 §D3-a/b,
BC-2.01.017 §P2). Confirmed by: Alerts OpenAPI info.description authentication prose (the
securitySchemes omission was a doc-gen artifact), IBM QRadar connector, qmasters integration,
ThreatQ adapter, XSOAR integration, and Axonius connector — all hitting `/alert/api/v1/alerts`
with `Cookie: access_token=<token>`.

The `X-Api-Key` hypothesis (raised in v0.1 D3-c) is REJECTED. The precondition blocking
spec authoring is RESOLVED.

**Sub-question resolved:** Both Cyberint surfaces use the same static-cookie mechanism
(no login roundtrip). This is consistent with BC-2.01.017 §P2's `StaticCookieAuthProvider`
(no-login, static credential, `access_token` cookie) and ADR-031 §D3-a/b. The
`cookie_roundtrip` auth_type label remains correct — it names the cookie-injection behavior,
not a login exchange.

**D3-d — Cyberint Assets auth CONFIRMED (from v0.1, unchanged):**

The Assets OpenAPI (`cyberint_assets_openapi_06.20.2026.json`) declares:
`securitySchemes."Access Token" = {type:apiKey, in:cookie, name:access_token}`.
Static API-key-as-cookie, no login step. This matches ADR-031's `StaticCookieAuthProvider`
pattern.

Cyberint Assets: `auth_type = "cookie_roundtrip"`, `header_scheme = "cookie:access_token"`,
`credential_ref = "access_token"`. Unchanged from existing Assets-facing behavior; the
fix is isolating this to a dedicated `cyberint-assets.sensor.toml` spec file.

**Supersession scope:** ADR-028 LOCKED Cyberint decision (D-747, combined `cookie_roundtrip`
spec) is superseded in the sense that the combined spec is split. Both surfaces continue to
use `cookie_roundtrip` with static-cookie injection. ADR-031 §D3 (scope-narrowed, not
reversed) — §D3's single-surface assumption is narrowed to the Assets definition only. The
§D3-a/b Assets static-cookie contract (`StaticCookieAuthProvider`, `access_token` cookie,
no login step) is PRESERVED and unchanged. The Alerts surface becomes a separate sensor
definition under D3-a above, not governed by ADR-031 §D3.

**Human authorization:** D-1889 (2026-07-20). Final ADR approval gate pending.

### D4 — Wave-C Out-of-Scope (TLS/Transport, F10)

DEFECT-ADAPTER-TLS-XDOME-LIVE-001 (F10, CRIT): live xDome HTTPS connections fail against
the WAF profile (h1-only, no User-Agent). The fix requires transport-level changes. Any
transport fix MUST remain ADR-050-compliant: `native-tls` is forbidden; the `rustls-tls`
backend MUST be used. The allowed path is HTTP/1.1 forced with `User-Agent` injection via
reqwest builder. This is deferred to Wave-C and is explicitly out of scope for this ADR.

No spec, BC, or story for F10 may be authored before a Wave-C transport ADR is accepted.

### D5 — Spec Amendment Manifest

The following BCs require amendment as a direct consequence of D1–D3. Each amendment is
in-scope for the corresponding remediation story.

| BC | Amendment required | Triggered by |
|----|-------------------|--------------|
| BC-2.01.008 (`armis-bearer-aql`) | Title and contract-level auth premise invalidated — Armis auth is no longer `bearer_static`; the BC must be updated to reflect `custom_via_plugin` + token-exchange + `header_scheme = "raw"` | D2 |
| BC-2.01.017 §P2 dispatch table | Currently hardcodes `CustomViaPlugin → Authorization: Bearer {token}`. Must gain the `header_scheme`-based raw arm (or delegate dispatch to `header_scheme` entirely) to eliminate the spec-vs-spec conflict with D2's `header_scheme = "raw"` for Armis | D2 |
| BC-2.01.006 | Split scope: rename/restrict existing Cyberint BC to Assets surface only; author new Cyberint Alerts BC covering the `/alert` surface auth and endpoints | D3 |

Additional artifacts requiring audit (not BC amendments, but must not contain contradicted values):
- Any story, holdout scenario, or test grounded on the old Armis `bearer_static` or
  Cyberint combined-spec values must be audited and updated per the supersession.
- ADR-032 Armis credential rows (`bearer_token` credential_ref) must be updated to `secret_key`.

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

### Why `custom_via_plugin` for Armis token-exchange acquisition?

The WASM plugin approach (D2) follows ADR-023's plugin-only sensor architecture: complex
sensor-specific HTTP logic (token acquisition, credential handling, expiry tracking) belongs
in the WASM sandbox, not in spec-engine code. The CrowdStrike-oauth2 plugin is the proven
production precedent for HTTP-at-acquire_token. The Armis token-exchange flow is structurally
identical; the plugin handles the acquisition without spec-engine changes.

A native `token_exchange` `AuthType` variant (Option C, Alternatives) would conflate
acquisition pattern with injection pattern — adding spec-engine code, E-SPEC-012 amendments,
and BC-2.01.016 changes without changing the acquisition behavior. If two or more additional
sensors exhibit token-exchange acquisition, the cost/benefit shifts and a native auth_type
should be evaluated.

**The real engine cost (corrected from v0.1 rationale):** D2 does require an engine change —
the `header_scheme` TOML field and the `build_request()` dispatch change. This was always
required for raw-token injection; the v0.1 "zero engine change" rationale was false. The
`header_scheme` field is the correct approach because it decouples the header shape (injection)
from the provider type (acquisition), keeps `AuthType` closed, and fixes ALL current and
future non-Bearer sensors in a single backward-compatible change.

### Why two sensor definitions for Cyberint?

The two-definition approach (D3-a) requires zero spec-engine changes beyond `header_scheme`,
delivers clean audit trails per surface, and isolates the less-stable Assets API (v0.1.0,
"FastAPI" title) from the more stable Alerts API (v1.0). The alternative (per-table auth
overrides) requires spec-engine schema changes, which are a separate Wave-B scope. The
two-definition approach is self-consistent with ADR-023's principle that TOML specs are
declarative baselines. Auth is now confirmed identical for both surfaces (static
`access_token` cookie), so the split cost is purely configuration (two sensor definitions
vs one) with zero auth-complexity overhead.

---

## Consequences

### Positive

- All Wave-A spec authoring for Armis and Cyberint is grounded on vendor OpenAPI, not DTU
  circular references. Drift detection becomes dtu-validator's job (OpenAPI vs DTU), not a
  manual audit activity.
- Armis token-exchange plugin follows the established CrowdStrike-oauth2 plugin pattern.
  The `header_scheme = "raw"` field resolves the raw-token injection gap at the engine level
  in a single backward-compatible change that also benefits any future non-Bearer sensor.
- Both Cyberint surfaces now have confirmed auth: static `access_token` cookie, no login step,
  no token-exchange complexity. The split is purely structural (two `base_url` values).
- The phantom Cyberint `incidents` table (DEFECT-CYBERINT-SPEC-FIDELITY-001) is eliminated
  by the two-definition approach — neither `cyberint-alerts.sensor.toml` nor
  `cyberint-assets.sensor.toml` contains an `incidents` table.
- ADR-031 (DTU = True-DTU) is structurally reinforced: dtu-validator now has an OpenAPI
  baseline to score against, not just the spec.

### Negative / Trade-offs

- **Engine change required (D2):** `SensorSpec` must gain `header_scheme` field and
  `PipelineExecutor::build_request()` must be updated to dispatch on it. This MUST land
  before any Armis live-client spec can be validated end-to-end.
- **Atomic DTU re-clone required (Armis — SR-003):** The existing Armis DTU enforces
  `Authorization: Bearer <non-empty>` (HTTP 403 on non-Bearer). As of D2, the spec will
  declare `header_scheme = "raw"`, injecting `Authorization: <raw_token>` (no Bearer prefix).
  This means ALL Armis DTU tests will fail at the transition point if the DTU is not
  simultaneously updated. The Armis remediation story MUST re-clone the Armis DTU to:
  (a) add `POST /api/v1/access_token/` token-exchange endpoint, and (b) accept raw-token
  Authorization headers. The spec flip and DTU re-clone MUST land in the SAME story commit
  sequence. Splitting them across separate stories risks a period where all Armis DTU tests
  403-fail on the new spec.
- Two Cyberint sensor definitions (not one) increase the config surface. Operators running
  Cyberint must configure two TOML entries with separate `base_url` values.
- LOCKED decisions D-747 (Armis bearer_static; Cyberint combined cookie_roundtrip) are
  superseded. Any story, holdout scenario, or test that was authored against the old locked
  values must be audited and amended. See D5 Spec Amendment Manifest.
- ADR-032 Armis credential rows (`bearer_token`) are now stale and must be updated to
  `secret_key` in the Armis remediation story.

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

- **Option C (rejected — native `token_exchange` `AuthType` variant for Armis):** Add a new
  `token_exchange` variant to the `AuthType` closed enum (BC-2.01.016 E-SPEC-012), implement
  a `TokenExchangeAuthProvider` in prism-spec-engine. Rejected for Wave-A because `AuthType`
  governs acquisition pattern (provider dispatch at spec-load), not injection pattern (header
  shape at request time). The acquisition pattern (HTTP POST → token cache → expiry refresh)
  is correctly handled by the WASM plugin, which is already the existing model for
  `custom_via_plugin`. Adding a `token_exchange` enum variant would change the provider
  dispatch without changing the acquisition behavior, requiring BC-2.01.016 + E-SPEC-012
  amendments for no behavioral benefit. Remains viable for Wave-B if additional sensors
  require token-exchange acquisition with a dedicated native provider.

- **Option D (rejected — per-table auth overrides for Cyberint):** Add per-table `auth_type`
  and `base_url` override fields to `SensorSpec`. This is the cleanest long-term schema but
  requires spec-engine changes, BC amendments, and a new story. Deferred to Wave-B.

- **Option E (rejected — single Cyberint definition with cookie_roundtrip for both surfaces):**
  Keep one definition, use the same cookie auth for both surfaces. Now that both surfaces are
  confirmed to use identical static-cookie auth, this is technically feasible — but the
  different server prefixes (`/alert` vs `/asset-configuration`) still require two base URLs.
  A single definition cannot express two base URLs without per-table overrides (Option D).

- **Option F (rejected — `header_scheme` as auth_type variant instead of separate field):**
  Extend `auth_type` to encode both acquisition and injection (e.g., `"custom_via_plugin_raw"`).
  Rejected because this conflates two orthogonal concerns in a single value, exponentially
  expanding the enum as combinations multiply, and invalidates the existing E-SPEC-012
  validation surface.

---

## Source / Origin

- **Triage capture:** `.factory/planning/findings-remediation-2026-07-20/triage-capture.md`
  (D-1889, 2026-07-20) — systemic root cause identified as ADR-028 circular grounding
- **Findings source (Armis):** `/Users/jmagady/Dev/test-soc/demo-soc/findings/prism-armis-endpoint-plan.md`
  (2026-07-20) — auth flow evidence: token-exchange flow, raw-token header, no Bearer prefix;
  corroborated by Google Chronicle + Brinqa + Cortex XSOAR + Sumo Logic + Hunters connectors
- **Findings source (Cyberint):** `/Users/jmagady/Dev/test-soc/demo-soc/findings/prism-cyberint-endpoint-plan.md`
  (2026-07-20) — two-surface OpenAPI analysis; Alerts and Assets both confirmed static-cookie
- **Cyberint Alerts auth confirmation (research-agent, 2026-07-20):** Confirmed `Cookie: access_token=<static token>`, no login step, via Alerts OpenAPI info.description + IBM QRadar + qmasters + ThreatQ + XSOAR + Axonius connectors. X-Api-Key hypothesis rejected.
- **OpenAPI files (Cyberint):** `.factory/reference/api-specs/cyberint_alerts_openapi_06.20.2026.json`
  and `cyberint_assets_openapi_06.20.2026.json` (both dated 2026-06-20)
- **Engine location (Authorization header injection):** `crates/prism-sensors/src/pipeline.rs` —
  `PipelineExecutor::build_request()` function; Bearer prefix hardcoded for non-cookie auth
  (cite function name + behavioral anchor at D2 acceptance time)
- **CrowdStrike-oauth2 plugin precedent:** `crowdstrike-oauth2/` WASM plugin; `acquire_token()`
  HTTP-POST pattern — cite function name + behavioral anchor at D2 acceptance time
- **Human authorization:** D-1889 (2026-07-20); triage-capture.md Open Decision #1/#2/#3 sign-off

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 0.2 | 2026-07-20 | architect | Central design decision: header injection mechanism resolved — `header_scheme` TOML field (`bearer`/`raw`/`cookie:<name>`) replaces `auth_type`-based dispatch in `build_request()`; keeps `AuthType` closed; fixes Armis raw-token + generalizes to future sensors. Armis TOML wiring specified: `auth_plugin = "armis-token-exchange"`, `header_scheme = "raw"`, `credential_ref = "secret_key"` (HIGH-4: was `armis_secret_key` — doubled sensor segment). D3-c CONFIRMED: Cyberint Alerts auth is static `Cookie: access_token` (no login step); X-Api-Key hypothesis rejected; research-agent citation added. HIGH-2 fix: ADR-031 §D3 supersession reframed as scope-narrowing (Assets only), not reversal — §D3-a/b static-cookie contract PRESERVED. D5 Spec Amendment Manifest added (BC-2.01.008, BC-2.01.017 §P2, BC-2.01.006). SR-003 Armis DTU atomic-sequencing consequence added. SR-004: SensorAuth→`AuthType` (`spec_parser.rs`)/E-SPEC-012 correction throughout. SR-005: `auth_plugin` TOML field wiring + BC-2.01.016 EC-016-002/005 cited. SR-006: Spec Amendment Manifest. SR-007: No-OpenAPI governance sentence (Confirmed/Partial tiers). SR-008: C2-class Cyberint mechanical fixes noted in D1. F3: §Source/Origin cite instructions changed from file:line to function+anchor. F4: "EIGHT of the four sensors" → "eight CRIT findings across all four sensors". OBS-5: §Changelog promoted to top-level ## Changelog. MED-1: ADR-032 added to related_adrs. |
| 0.1 | 2026-07-20 | architect | Initial proposal under D-1889 authorization |
