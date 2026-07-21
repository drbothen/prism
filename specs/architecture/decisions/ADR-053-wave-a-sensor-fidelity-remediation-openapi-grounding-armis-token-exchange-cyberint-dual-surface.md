---
document_type: adr
adr_id: "ADR-053"
title: "Wave-A Sensor Fidelity Remediation — OpenAPI Grounding, Armis Token-Exchange Auth, and Cyberint Dual-Surface Split"
status: proposed
date: "2026-07-20"
modified: "2026-07-20"
version: "0.9"
producer: architect
subsystems_affected: [SS-01, SS-06, SS-16, SS-17]
supersedes:
  - "ADR-028 §D1/§D2/§D5 (grounding order rules: DTU→OpenAPI as canonical; effective on ADR-053 acceptance)"
  - "ADR-028 LOCKED Armis auth_type D-747 (bearer_static→token_exchange native declarative per ADR-054)"
  - "ADR-028 LOCKED Cyberint auth_type D-747 (combined cookie_roundtrip spec→dual-surface split)"
  - "ADR-031 §D3 (scope-narrowing only: single-surface assumption narrowed to Assets; §D3-b items 1-2 StaticCookieAuthProvider provider contract PRESERVED; §D3-b item 3 auth_type-keyed dispatch table superseded by header_scheme dispatch — see D2/D5; §D3-a DTU changes unaffected; §D3-b item 4 S-DEMO-001 story-scope note moot/historical — delivered)"
superseded_by: null
amends: null
related_adrs: [ADR-026, ADR-028, ADR-031, ADR-032, ADR-050, ADR-054]
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

Proposed 2026-07-20. v0.9 (2026-07-21): Wave-A cascade re-gate closures — BC-2.16.014 anchoring corrected
(planned declarative-auth BC was mis-anchored to non-existent SS-23; retargeted to SS-16 as BC-2.16.014); D2 header corrected to "Native
DeclarativeHttpAuthProvider"; CrowdStrike Alerts-v2 Alerts:READ scope prerequisite added to D1.
v0.8 (2026-07-20): LOW-3 E-SPEC-027 coherence matrix prose aligned. v0.7 (D-1895): D2 rewritten
— Armis uses `token_exchange` auth_type + `[auth_acquisition]` block per companion ADR-054;
`custom_via_plugin` + plugin approach removed. Coherence matrix updated. ADR-054 added to
`related_adrs`. D5 manifest updated: BC-2.01.008 amendment reflects `token_exchange` native
provider; BC-2.16.014 authoring row and E-SPEC-028 registration row added.
v0.6: pass-5 adversary remediation + full citation audit.
Awaiting human approval gate before proceeding to spec/BC work.
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

**CrowdStrike Alerts-v2 credential scope prerequisite (source: prism-crowdstrike-endpoint-plan.md Gap G9):**
`S-CROWDSTRIKE-ALERTS-V2-MIGRATION-001` requires the CrowdStrike API credential to carry the
`Alerts:READ` scope. A credential provisioned with only `Detects:READ` (the legacy scope for
the retired v1 Detections API) will fail at implementation with a scope/permission error from
the Alerts-v2 endpoints — this is a scope-configuration error, not a code defect. The
implementer must verify the tenant credential scope before proceeding with the migration.
This prerequisite must be captured in the Wave-A Alerts-v2 story acceptance criteria.

**Human authorization:** D-1889 (2026-07-20). Final ADR approval gate pending.

### D2 — Armis Auth Model: Token-Exchange via Native DeclarativeHttpAuthProvider + `header_scheme` Field (supersedes ADR-028 LOCKED Armis, D-747)

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

**Implementation approach: `auth_type = "token_exchange"` with native `DeclarativeHttpAuthProvider` (ADR-054).**

Per human decision D-1895 and companion ADR-054: Armis auth uses the new native `token_exchange`
auth_type backed by `DeclarativeHttpAuthProvider` in `crates/prism-spec-engine/src/auth/`.
No WASM plugin is required. The `token_exchange` variant is added to the `AuthType` closed enum
per ADR-054 D1. `DeclarativeHttpAuthProvider` handles HTTP POST → token extraction → TTL caching
in pure host Rust (reqwest, ADR-050 compliant, 30s timeout, rustls-tls).

**TOML wiring for Armis (`[auth_acquisition]` block — per ADR-054 D3):**

```toml
sensor_id = "armis"
auth_type = "token_exchange"       # native declarative provider (ADR-054 D1)
header_scheme = "raw"              # Authorization: {token} — no Bearer prefix

[auth_acquisition]
token_url = "${env.ARMIS_INSTANCE_URL}/api/v1/access_token/"
credential_body_field = "secret_key"        # form body: secret_key={resolved_value}
token_response_path = "data.access_token"   # $.data.access_token
expiry_field = "data.expiration_utc"        # $.data.expiration_utc (absolute UTC string)
expiry_mode = "absolute_utc_string"
# ttl_buffer_secs = 30 (default)

[[credential_refs]]
name = "secret_key"                # resolved via BC-2.06.003 four-tier per-client chain (PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF})
```

`boot.rs::validate_and_construct_auth_providers` constructs a `DeclarativeHttpAuthProvider(TokenExchange, ...)`
for `auth_type = "token_exchange"`. No `auth_plugin` field; no `BootError::UnknownAuthPlugin` path.
E-SPEC-028 (ADR-054 D10) validates the `[auth_acquisition]` block at spec-load time.

The `DeclarativeHttpAuthProvider::acquire_token()` implementation (per ADR-054 D4):
- Resolves the `secret_key` credential reference via BC-2.06.003 at call time (lazy — never at construction)
- Performs `POST /api/v1/access_token/` with form body `secret_key={resolved_value}` (RFC-3986 percent-encoded)
- Parses `$.data.access_token` from the JSON response (via `token_response_path = "data.access_token"`)
- Computes expiry from `$.data.expiration_utc` as absolute UTC → Unix timestamp, minus `ttl_buffer_secs` (30s)
- Caches token + expiry in in-memory ArcSwap (no plugin KV store)
- Returns the raw token to the pipeline

**Header injection selection mechanism: `header_scheme` TOML field.**

The current `PipelineExecutor::build_request()` (in `crates/prism-spec-engine/src/pipeline.rs`)
dispatches on `auth_type` with two arms: `CookieRoundtrip → Cookie: access_token={token}` and
a catch-all `→ Authorization: Bearer {token}`. This is incorrect for Armis v1 (raw token, no
prefix). The `auth_type`-based dispatch is also insufficient as a general injection key because
multiple auth_types (including `oauth2_client_credentials` and `token_exchange`) may need
different injection schemes. `header_scheme` decouples injection from acquisition.

The selection key is a new `header_scheme` TOML field on `SensorSpec`. Values:

| `header_scheme` value | Header injected | Use case |
|----------------------|-----------------|----------|
| `"bearer"` (default) | `Authorization: Bearer {token}` | CrowdStrike, Claroty, existing sensors |
| `"raw"` | `Authorization: {token}` (no prefix) | Armis v1 raw-token |
| `"cookie:<name>"` | `Cookie: <name>={token}` | Cyberint (both surfaces, parameterized) |

`build_request()` reads `spec.header_scheme` (defaulting to `"bearer"` when field is absent)
and dispatches on it. The existing `auth_type`-based cookie arm is replaced by the
`"cookie:access_token"` `header_scheme` value.

**Backward-compatibility scope and co-land sequencing requirement:**

Sensors that currently inject bearer tokens (`auth_type` ≠ `cookie_roundtrip`) and do not
declare `header_scheme` continue to receive `Authorization: Bearer {token}` unchanged (default
`"bearer"`). The exception is `cyberint.sensor.toml`: it currently uses
`auth_type = "cookie_roundtrip"`, which drives the old `auth_type`-based cookie arm in
`build_request()`. Under `header_scheme` dispatch, without an explicit `header_scheme` field,
it would default to `"bearer"` and break live Cyberint auth.

Because `cyberint.sensor.toml` is superseded and DELETED by the Cyberint spec migration story
(D3-a), this is not a sensor-migration issue — but story-ordering is critical:

The Cyberint spec migration story MUST NOT merge before the standalone Wave-A engine story
has landed (adding `header_scheme` to `SensorSpec` and switching `build_request()` dispatch
to `header_scheme`-based). When the Cyberint spec migration story lands it MUST atomically:
- Delete `cyberint.sensor.toml`
- Create `cyberint-alerts.sensor.toml` (with `header_scheme = "cookie:access_token"`)
- Create `cyberint-assets.sensor.toml` (with `header_scheme = "cookie:access_token"`)

This story-level dependency ordering (engine story → Cyberint spec migration story) replaces
any literal "same commit" constraint. The risk of a deployment window where live Cyberint auth
breaks is closed by the story-dependency gate: once the engine story is merged, deployments
that include it will also include the new Cyberint spec files (both declaring explicit
`header_scheme = "cookie:access_token"`) because both ship in the same story merge window.

**`header_scheme` validation (spec_parser.rs, load time):**

The `header_scheme` field is validated at spec-load time in the same multi-error pass as other
field validations (a new Rule 9, after Rule 8 probe_table, per BC-2.16.009). An unrecognized
`header_scheme` value is a load-time error — no silent fallthrough to `"bearer"` (SOUL.md #4
anti-silent-failure).

Closed value set:

| `header_scheme` value | Valid | Rejection reason if invalid |
|-----------------------|-------|-----------------------------|
| `"bearer"` | Yes | — (default when field absent) |
| `"raw"` | Yes | — |
| `"cookie:<name>"` | Yes | `<name>` must be non-empty; must not contain a colon |
| `"cookie:"` (empty name) | No | Cookie name required (`E-SPEC-027` template a) |
| `"cookie"` (no colon separator) | No | Must be `cookie:<name>` form (`E-SPEC-027` template a) |
| `"cookie:a:b"` (colon in name) | No | Cookie name must not contain a colon (`E-SPEC-027` template a) |
| Any other value | No | Not in closed set (`E-SPEC-027` template a) |

**Error code E-SPEC-027** (new, next free per append_only_numbering DF-030). Two message
templates are required to avoid self-contradiction on well-formed-but-incoherent combinations:

**(a) Unknown or malformed value:**
`"sensor '{sensor_id}' has invalid header_scheme = '{value}'. Valid values: bearer, raw, cookie:<name> (non-empty name required, no colon in name)"`

**(b) Well-formed value, incoherent with auth_type:**
`"sensor '{sensor_id}': auth_type = '{auth_type}' does not permit header_scheme = '{value}'; allowed for this auth_type: {allowed_set}"`

where `{allowed_set}` is derived from the coherence matrix: `bearer_static` → `bearer, raw`;
`oauth2_client_credentials` → `bearer, raw`; `cookie_roundtrip` → `cookie:<name>`;
`custom_via_plugin` → `bearer, raw`; `token_exchange` → `bearer, raw`;
`api_key` → `bearer` (Wave-A scope).

This generalized form produces a correct, actionable message for every incoherent cell:
- `cookie_roundtrip` + `"bearer"` → "...does not permit header_scheme = 'bearer'; allowed: cookie:<name>"
- `bearer_static` + `"cookie:x"` → "...does not permit header_scheme = 'cookie:x'; allowed: bearer, raw"
- `api_key` + `"raw"` → "...does not permit header_scheme = 'raw'; allowed: bearer"
(and so on for all 6 directions — none falsely claims the other direction's constraint)

Template (a) fires when `{value}` is not in the closed value set or is malformed. Template (b)
fires when `{value}` is well-formed but violates the coherence matrix. Both are load-time
errors; spec rejected; boot fails exit code 2; non-retryable. The `{value}`, `{auth_type}`,
and `{allowed_set}` fields are config text (not credentials per AD-017) and safe to echo.
Must be registered in `error-taxonomy.md` with both templates as part of the standalone Wave-A
engine story (same story as `SensorSpec::header_scheme` addition).

**`auth_type × header_scheme` coherence matrix (load-time validation):**

The spec_parser validates that the declared `auth_type` and `header_scheme` are coherent.
Incoherent combinations emit E-SPEC-027 template (b) and reject the spec at load time.

| `auth_type` | Allowed `header_scheme` values | Canonical value |
|-------------|-------------------------------|-----------------|
| `bearer_static` | `"bearer"`, `"raw"` | `"bearer"` |
| `oauth2_client_credentials` | `"bearer"`, `"raw"` | `"bearer"` |
| `cookie_roundtrip` | `"cookie:<name>"` **only** | `"cookie:access_token"` |
| `custom_via_plugin` | `"bearer"`, `"raw"` | `"raw"` for Armis (historical; Armis now uses `token_exchange`) |
| `api_key` | `"bearer"` (Wave-A scope) | `"bearer"` |
| `token_exchange` (new per ADR-054) | `"bearer"`, `"raw"` | `"raw"` for Armis v1 (raw-token, no Bearer prefix) |

**Critical restriction:** `cookie_roundtrip` MUST use a `cookie:<name>` `header_scheme`.
Using `"bearer"` or `"raw"` with `cookie_roundtrip` would inject an Authorization header
instead of a Cookie header, silently breaking auth (SOUL.md #4). This combination is rejected
at load time with E-SPEC-027 template (b).

**`api_key` + non-Bearer injection (Wave-B extension):** `AuthType::ApiKey` could legitimately
support a custom-header (`X-Api-Key`) or query-parameter injection pattern for some sensors.
The `api_key → "bearer"` restriction above is a conservative Wave-A scope bound, not a
permanent exclusion. A Wave-B ADR may extend allowed `header_scheme` values for `api_key`
(e.g., a future `"header:<name>"` or `"query:<name>"` value). No `api_key` sensor is in
Wave-A scope; this restriction does not foreclose future sensors.

**`header_scheme` governs INJECTION; `auth_type` governs ACQUISITION (separation of concerns):**

`header_scheme` governs INJECTION — how the acquired token is placed in the HTTP request.
`auth_type` governs ACQUISITION — how the token is obtained. These two axes are orthogonal:
`token_exchange` (Armis acquisition) + `header_scheme = "raw"` (Armis injection) are independent
declarations. A future Armis v3 that used `oauth2_client_credentials` acquisition but still
needed raw-token injection would express that via `auth_type = "oauth2_client_credentials"` +
`header_scheme = "raw"` — no change to the dispatch logic.

**v0.7 note:** The v0.1–v0.6 text at this section argued against adding `token_exchange` to
the `AuthType` enum, on the grounds that provider construction was `auth_plugin.is_some()`-driven.
That argument is superseded by D-1895 and ADR-054: provider construction for declarative auth
types is now `auth_type`-driven (`DeclarativeHttpAuthProvider` for `token_exchange` and
`oauth2_client_credentials` with `[auth_acquisition]`). The `header_scheme` / `auth_type`
separation-of-concerns principle remains valid and unchanged.

**Required engine change (standalone Wave-A engine story):** `SensorSpec` gains the
`header_scheme` field (with `#[serde(default)]` defaulting to `"bearer"`).
`PipelineExecutor::build_request()` (in `crates/prism-spec-engine/src/pipeline.rs`) switches
from `auth_type`-based dispatch to `header_scheme`-based dispatch. E-SPEC-027 (both message
templates), BC-2.16.009 Rule 9, and `error-taxonomy.md` registration are all in-scope for
this standalone engine story. The Armis auth story and the Cyberint spec migration story both
declare a merge-dependency on this engine story — neither may merge until the engine story
has landed.

**VP assignment (DRIFT-D849-002):** The VP that `acquire_token()` makes no network calls
during spec-load applies to the Armis plugin. The plugin MUST lazy-acquire on first sensor
request, not at spec-parsing or boot time. Implementation evidence must be cited in the
plugin's `acquire_token()` function via behavioral anchor before this ADR is accepted.

**Supersession scope:** ADR-028 LOCKED Armis decision (D-747, `bearer_static`) is superseded.
ADR-028 §D13's consistency table row `bearer_static (Armis, Claroty)` is narrowed — see §D13
supersession blockquote. ADR-026 §D3's `ArmisAuth::auth_type_name() → "api_key"` was already
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
- `cyberint-alerts.sensor.toml` — Alerts surface (`/alert` prefix, `auth_type = "cookie_roundtrip"`, `header_scheme = "cookie:access_token"`, `[[credential_refs]]` entry with `name = "access_token"`)
- `cyberint-assets.sensor.toml` — Assets surface (`/asset-configuration` prefix, `auth_type = "cookie_roundtrip"`, `header_scheme = "cookie:access_token"`, `[[credential_refs]]` entry with `name = "access_token"`)

Note: the `[[credential_refs]]` entry `name` field is changed from the current `api_key` (in
`cyberint.sensor.toml`) to `access_token`. Rationale: `access_token` is the wire cookie name
(per the OpenAPI Assets
securityScheme, ADR-031 §D3-b items 1-2, and ADR-028 §D13 consistency table), and is the
exact value injected by `header_scheme = "cookie:access_token"`. `api_key` was a generic
placeholder that did not match the wire name. Env vars change from
`PRISM_CLIENTS_{ORG}_SENSORS_CYBERINT_API_KEY` to
`PRISM_CLIENTS_{ORG}_SENSORS_CYBERINT_ALERTS_ACCESS_TOKEN` and
`PRISM_CLIENTS_{ORG}_SENSORS_CYBERINT_ASSETS_ACCESS_TOKEN` (per ADR-032 per-client format).

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
Assets surface mechanism and matches the `StaticCookieAuthProvider` pattern (ADR-031
§D3-b items 1-2, BC-2.01.017 §P2). Confirmed by: Alerts OpenAPI info.description
authentication prose (the securitySchemes omission was a doc-gen artifact), IBM QRadar
connector, qmasters integration, ThreatQ adapter, XSOAR integration, and Axonius connector —
all hitting `/alert/api/v1/alerts` with `Cookie: access_token=<token>`.

The `X-Api-Key` hypothesis (raised in v0.1 D3-c) is REJECTED. The precondition blocking
spec authoring is RESOLVED.

**Sub-question resolved:** Both Cyberint surfaces use the same static-cookie mechanism
(no login roundtrip). This is consistent with BC-2.01.017 §P2's `StaticCookieAuthProvider`
(no-login, static credential, `access_token` cookie) and ADR-031 §D3-b items 1-2. The
`cookie_roundtrip` auth_type label remains correct — it names the cookie-injection behavior,
not a login exchange.

**D3-d — Cyberint Assets auth CONFIRMED (from v0.1, unchanged):**

The Assets OpenAPI (`cyberint_assets_openapi_06.20.2026.json`) declares:
`securitySchemes."Access Token" = {type:apiKey, in:cookie, name:access_token}`.
Static API-key-as-cookie, no login step. This matches ADR-031's `StaticCookieAuthProvider`
pattern.

Cyberint Assets: `auth_type = "cookie_roundtrip"`, `header_scheme = "cookie:access_token"`,
`[[credential_refs]]` entry with `name = "access_token"`. Unchanged from existing Assets-facing
behavior; the fix is isolating this to a dedicated `cyberint-assets.sensor.toml` spec file.

**Supersession scope:** ADR-028 LOCKED Cyberint decision (D-747, combined `cookie_roundtrip`
spec) is superseded in the sense that the combined spec is split. Both surfaces continue to
use `cookie_roundtrip` with static-cookie injection. ADR-031 §D3 (scope-narrowed, not
reversed) — §D3's single-surface assumption is narrowed to the Assets definition only.

**What is PRESERVED from ADR-031 §D3-b items 1-2** (the Prism provider contract): the
`StaticCookieAuthProvider` PROVIDER CONTRACT — injects the API key as a static
`Cookie: {name}={token}` header; `acquire_token()` reads the credential from the credential
store at request time (NOT at construction time per AD-017); makes NO HTTP request during
`acquire_token()`; no login step, no session exchange, no token expiry. Items 1 and 2 of
ADR-031 §D3-b define this provider contract and are UNCHANGED by this supersession.

**What CHANGES from ADR-031 §D3-b item 3** (the dispatch mechanism): §D3-b item 3's
`auth_type`-keyed 4-row `build_request()` dispatch table (the `CookieRoundtrip →
Cookie: access_token={token}` arm) is replaced by `header_scheme`-based dispatch (the
`"cookie:access_token"` value). The `cookie_roundtrip` auth_type continues to signal
"cookie injection" semantically, but `build_request()` reads `header_scheme`, not `auth_type`,
for the injection decision after D2 lands. ADR-031 §D3-b item 3's dispatch table is therefore
a D5 Spec Amendment Manifest target (see §D5).

**ADR-031 §D3-a (DTU changes — unaffected):** The DTU-side corrections in §D3-a (remove
`POST /login` route, `extract_access_token()`, static-auth registry, `check_auth` update)
are orthogonal to the Prism-side dispatch change and are unaffected by this supersession.

The Alerts surface becomes a separate sensor definition under D3-a above.

**Human authorization:** D-1889 (2026-07-20). Final ADR approval gate pending.

### D4 — Wave-C Out-of-Scope (TLS/Transport, F10)

DEFECT-ADAPTER-TLS-XDOME-LIVE-001 (F10, CRIT): live xDome HTTPS connections fail against
the WAF profile (h1-only, no User-Agent). The fix requires transport-level changes. Any
transport fix MUST remain ADR-050-compliant: `native-tls` is forbidden; the `rustls-tls`
backend MUST be used. The allowed path is HTTP/1.1 forced with `User-Agent` injection via
reqwest builder. This is deferred to Wave-C and is explicitly out of scope for this ADR.

No spec, BC, or story for F10 may be authored before a Wave-C transport ADR is accepted.

### D5 — Spec Amendment Manifest

The following BCs, ADR sections, and configuration artifacts require amendment as a direct
consequence of D1–D3. Each amendment is in-scope for the corresponding remediation story.

| Artifact | Amendment required | Triggered by |
|----------|-------------------|--------------|
| BC-2.01.008 (`armis-bearer-aql`) | Title and contract-level auth premise invalidated — Armis auth is no longer `bearer_static`; the BC must be updated to reflect `token_exchange` (native `DeclarativeHttpAuthProvider` per ADR-054) + `header_scheme = "raw"` | D2 |
| BC-2.01.017 §P2 dispatch table | Currently hardcodes `CustomViaPlugin → Authorization: Bearer {token}`. Must gain the `header_scheme`-based raw arm (or delegate dispatch to `header_scheme` entirely) to eliminate the spec-vs-spec conflict with D2's `header_scheme = "raw"` for Armis | D2 |
| BC-2.01.017 INV-COOKIE-004 | Cookie injection invariant references `auth_type = "cookie_roundtrip"` as the dispatch trigger; must be re-grounded on `header_scheme = "cookie:<name>"` | D2 |
| BC-2.01.017 TV-BC-2.01.017-008 | Test vector grounded on `auth_type`-based cookie dispatch; must be re-grounded on `header_scheme = "cookie:access_token"` dispatch | D2 |
| BC-2.01.006 | Split scope: rename/restrict existing Cyberint BC to Assets surface only; author new Cyberint Alerts BC covering the `/alert` surface auth and endpoints | D3 |
| ADR-031 §D3-b item 3 (dispatch table) | The `auth_type`-keyed 4-row dispatch table (`CookieRoundtrip → Cookie: access_token={token}` arm) becomes stale under `header_scheme` dispatch; amendment required to reflect `header_scheme = "cookie:<name>"` as the injection dispatch key, replacing the `auth_type = "cookie_roundtrip"` arm. §D3-b items 1-2 (StaticCookieAuthProvider provider contract) are PRESERVED and do not require amendment. | D2 |
| Cyberint `[[credential_refs]]` name rename (`api_key` → `access_token`) | Both new Cyberint spec files (`cyberint-alerts.sensor.toml` and `cyberint-assets.sensor.toml`) MUST use a `[[credential_refs]]` block with `name = "access_token"` (renamed from the `api_key` `[[credential_refs]]` entry in `cyberint.sensor.toml`). Rationale: `access_token` is the wire cookie name (OpenAPI securityScheme, ADR-028 §D13 consistency table, ADR-031 §D3-b items 1-2). Env vars change from `PRISM_CLIENTS_{ORG}_SENSORS_CYBERINT_API_KEY` to `PRISM_CLIENTS_{ORG}_SENSORS_CYBERINT_ALERTS_ACCESS_TOKEN` / `PRISM_CLIENTS_{ORG}_SENSORS_CYBERINT_ASSETS_ACCESS_TOKEN` (per ADR-032 per-client format). | D3 |
| ADR-032 Cyberint credential rows audit | ADR-032 rows referencing `cyberint.sensor.toml` with `[[credential_refs]]` `name = "api_key"` / env var `CYBERINT_API_KEY` are stale post-deletion of `cyberint.sensor.toml`. Must be updated to reflect two new spec files with `[[credential_refs]]` `name = "access_token"` and corresponding per-client env vars. | D3 |
| BC-2.16.009 Rule 9 (new) | Author new Rule 9 — `header_scheme` field validation (unknown/malformed → E-SPEC-027 template a; well-formed-but-incoherent with auth_type → E-SPEC-027 template b); rules numbered after Rule 8 probe_table (E-SPEC-026). Rule 9 is in-scope for the Wave-A engine story that adds `SensorSpec::header_scheme`. | D2 |
| `error-taxonomy.md` | Register E-SPEC-027 with both message templates: (a) unknown/malformed `header_scheme` value; (b) well-formed value incoherent with `auth_type` (generalized form — `sensor '{sensor_id}': auth_type = '{auth_type}' does not permit header_scheme = '{value}'; allowed for this auth_type: {allowed_set}`). Registration is in-scope for the standalone Wave-A engine story. | D2 |
| `error-taxonomy.md` | Register E-SPEC-028 (declarative auth acquisition validation errors — 7 message templates per ADR-054 D10): (a) required block absent; (b) conflicting auth_plugin; (c) unknown expiry_mode; (d) token_exchange missing required fields; (e) credential_body_field undeclared; (f) oauth2_client_credentials missing client_id/client_secret refs; (g) auth_acquisition declared for non-declarative auth_type. Registration is in-scope for the Wave-A CrowdStrike plugin retirement / Armis token-exchange story. | D2 (via ADR-054) |
| New `BC-2.16.014` | Author BC-2.16.014 — Declarative Auth Acquisition Token Lifecycle: `DeclarativeHttpAuthProvider` lazy-acquire, cache-hit, cache-refresh, and AD-017 credential-opacity invariants (ADR-054 D8). In-scope for the Wave-A CrowdStrike plugin retirement / Armis token-exchange story. | D2 (via ADR-054) |

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

### Why native `token_exchange` for Armis (not `custom_via_plugin` + plugin)?

Per human decision D-1895, standard HTTP token-acquisition flows MUST NOT require a WASM
plugin. ADR-054 provides the full design rationale; the summary for this ADR: the Armis
token-exchange flow (HTTP POST → JSON parse → TTL cache) is structurally identical to the
CrowdStrike-oauth2 plugin's behavior. A WASM plugin is unnecessary overhead for a flow that
the engine can express natively via `DeclarativeHttpAuthProvider`. `custom_via_plugin` is
preserved as an escape hatch for genuinely non-standard auth; Armis token-exchange is not
genuinely non-standard — it is a standard HTTP POST for a short-lived credential.

`token_exchange` is the correct semantic label (distinct from `oauth2_client_credentials`
which follows RFC 6749 fixed form body + response format). `boot.rs::validate_and_construct_auth_providers`
constructs `DeclarativeHttpAuthProvider(TokenExchange, ...)` for `auth_type = "token_exchange"`,
driven by `auth_type` (not `auth_plugin.is_some()`). The BC-2.01.016 closed-enum amendment
(E-SPEC-012 + spec_parser `VALID_AUTH_TYPES`) is a small, correct change per ADR-054 D1.

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
  `PipelineExecutor::build_request()` (in `crates/prism-spec-engine/src/pipeline.rs`) must be
  updated to dispatch on it. This MUST land before any Armis live-client spec can be validated
  end-to-end.
- **Atomic co-land sequencing — Armis DTU re-clone (SR-003):** The existing Armis DTU enforces
  `Authorization: Bearer <non-empty>` (HTTP 403 on non-Bearer). As of D2, the spec will
  declare `header_scheme = "raw"`, injecting `Authorization: <raw_token>` (no Bearer prefix).
  This means ALL Armis DTU tests will fail at the transition point if the DTU is not
  simultaneously updated. The Armis remediation story MUST re-clone the Armis DTU to:
  (a) add `POST /api/v1/access_token/` token-exchange endpoint, and (b) accept raw-token
  Authorization headers. The spec flip and DTU re-clone MUST land in the SAME story commit
  sequence. Splitting them across separate stories risks a period where all Armis DTU tests
  403-fail on the new spec.
- **Atomic co-land sequencing — Cyberint dispatch switch:** The `build_request()` dispatch
  switch from `auth_type`-based to `header_scheme`-based MUST land in the same commit as the
  deletion of `cyberint.sensor.toml` and creation of both new Cyberint spec files (each with
  explicit `header_scheme = "cookie:access_token"`). See D2 backward-compat discussion.
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

- **Option C (ACCEPTED — native `token_exchange` `AuthType` variant for Armis, per D-1895):**
  This option was initially deferred in v0.1–v0.6 but was elevated to the chosen approach by
  human architectural decision D-1895. ADR-054 ("Native Declarative HTTP Auth Acquisition")
  provides the full design for the `token_exchange` AuthType variant, `DeclarativeHttpAuthProvider`,
  `[auth_acquisition]` TOML block schema, and `crowdstrike-oauth2.prx` retirement. D2 of this
  ADR (v0.7) is updated to use `token_exchange` + `[auth_acquisition]` for Armis.

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
- **Engine location (Authorization header injection):** `crates/prism-spec-engine/src/pipeline.rs` —
  `PipelineExecutor::build_request()` function; Bearer prefix hardcoded for non-cookie auth
  (cite function name + behavioral anchor at D2 acceptance time)
- **CrowdStrike-oauth2 plugin precedent:** `crowdstrike-oauth2/` WASM plugin; `acquire_token()`
  HTTP-POST pattern — cite function name + behavioral anchor at D2 acceptance time
- **Human authorization:** D-1889 (2026-07-20); triage-capture.md Open Decision #1/#2/#3 sign-off

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 0.9 | 2026-07-21 | architect | HIGH-1: BC-2.16.014 anchoring corrected (prior drafts had mis-anchored the planned declarative-auth BC to non-existent SS-23; retargeted to next-free BC-2.16.014 in SS-16/prism-spec-engine); swept D5 manifest row, Status section, and Changelog v0.7 entry. HIGH-2: D2 section header corrected from "Token-Exchange via WASM Plugin" to "Token-Exchange via Native DeclarativeHttpAuthProvider" (body already described native path; header was stale from pre-D-1895 text). A-6 prerequisite: CrowdStrike Alerts-v2 credential scope prerequisite note added to D1 CrowdStrike implementation note (Alerts:READ scope required; Detects:READ alone fails at implementation with a scope error; source: prism-crowdstrike-endpoint-plan.md Gap G9). |
| 0.8 | 2026-07-20 | architect | LOW-3: E-SPEC-027 template (b) `{allowed_set}` prose enumeration (lines after the template definition) now includes `token_exchange → bearer, raw` — was omitted while the coherence matrix already contained the row, creating an inconsistency between the matrix and the narrative. No behavioral change; prose aligns to matrix. |
| 0.7 | 2026-07-20 | architect | D-1895 Armis native declarative auth (ADR-054): D2 rewritten — Armis uses `token_exchange` auth_type + `[auth_acquisition]` block per ADR-054; `custom_via_plugin` + `armis-token-exchange.prx` approach removed. Coherence matrix updated: `token_exchange → bearer, raw` row added. ADR-054 added to `related_adrs`. D5 manifest: BC-2.01.008 amendment description updated to reflect `token_exchange` native provider; BC-2.16.014 authoring row added; E-SPEC-028 registration row added. Rationale §Why custom_via_plugin updated to §Why native declarative provider per ADR-054. |
| 0.6 | 2026-07-20 | architect | Pass-5 adversary remediation + complete anchor/citation audit. HIGH-1: TOML block credential comment corrected — BC-2.03.006 (query-time resolution) → BC-2.06.003 (config-resolve chain); "three-tier" → "four-tier per-client" matching BC-2.06.003 v1.11. HIGH-2: E-SPEC-027 template (b) generalized to cover all 6 incoherence directions — `sensor '{sensor_id}': auth_type = '{auth_type}' does not permit header_scheme = '{value}'; allowed for this auth_type: {allowed_set}` (allowed_set from coherence matrix). MED-1: three-way story-ownership contradiction resolved — engine change is now a STANDALONE Wave-A engine story; "same-commit" Cyberint constraint replaced with story-level dependency-ordering; D2/D3/D5 consistent. MED-2: BC-2.01.016 EC miscite corrected — EC-016-002 is happy path (PluginRuntime resolution); EC-016-005 is unregistered-plugin rejection; E-SPEC-012 is auth_type validation; real unregistered-plugin error is `BootError::UnknownAuthPlugin`. OBS-2: ADR-031 §D3-b item 4 moot/historical note added to frontmatter supersedes. Citation audit completed — all BC IDs, EC numbers, error codes, symbols, crate paths, TOML grammar, tier counts verified. |
| 0.5 | 2026-07-20 | architect | Pass-4 adversary remediation. HIGH-1: D2 `header_scheme` validation rule-number corrected — "Rule 7 order" → "a new Rule 9 (after Rule 8 probe_table, per BC-2.16.009)"; D5 manifest row added for BC-2.16.009 Rule 9 authorship. MED-1: phantom symbol `construct_plugin_auth_providers` replaced with real symbol `validate_and_construct_auth_providers` in D2 body and Rationale (two sites). MED-2: D2 Armis TOML block rewritten — removed `[sensor]` table header, replaced scalar `credential_ref = "secret_key"` with `[[credential_refs]]` block + `name = "secret_key"` matching real SensorSpec grammar; D3-a/D3-d `credential_ref` prose corrected to reference `[[credential_refs]]` entry; D5 normative Cyberint instruction updated from scalar `credential_ref = "access_token"` to `[[credential_refs]]` entry with `name = "access_token"`. OBS-1: D5 manifest row added for `error-taxonomy.md` E-SPEC-027 dual-template registration (deferred to Wave-A engine story). |
| 0.4 | 2026-07-20 | architect | Pass-3 adversary remediation. HIGH-1 (paper-fix remediation): §D3-a/§D3-b attribution corrected throughout — §D3-a is DTU-only (no StaticCookieAuthProvider); §D3-b items 1-2 are the Prism provider contract (PRESERVED); §D3-b item 3 is the dispatch table (superseded). All sites corrected: frontmatter `supersedes[3]`, D3-c `(§D3-b items 1-2)`, D3 supersession scope `PRESERVED from §D3-b items 1-2 / CHANGES from §D3-b item 3`, explicit §D3-a unaffected callout. ADR-031 `superseded_by` frontmatter synced in ADR-031 v1.5. MED-1: E-SPEC-027 split into two message templates — (a) unknown/malformed value; (b) well-formed-but-incoherent (`auth_type = 'X' requires header_scheme = 'cookie:<name>'; got 'Y'`) — avoids self-contradiction where a valid value is rejected via coherence. MED-2: D5 expanded — Cyberint `credential_ref` rename decision (api_key→access_token; rationale: wire cookie name match + ADR-028 §D13 consistency table + ADR-031 §D3-b items 1-2) + env-var derivation; ADR-032 Cyberint stale-rows audit target. OBS-1: coherence matrix `api_key` row: Wave-B non-Bearer extension note added. OBS-2 addressed in ADR-028 v1.17 (§D13 env-var convention supersession note). |
| 0.3 | 2026-07-20 | architect | Pass-2 adversary remediation. HIGH-5: phantom crate path corrected — `crates/prism-sensors/src/pipeline.rs` → `crates/prism-spec-engine/src/pipeline.rs` in §Source/Origin and D2. HIGH-3: "all existing sensors unchanged" backward-compat claim corrected — `cyberint.sensor.toml` is the one exception (cookie_roundtrip without `header_scheme` would default to bearer, breaking auth); atomic co-land sequencing requirement added for dispatch switch + Cyberint spec files. HIGH-4: `header_scheme` validation spec added — closed value set, E-SPEC-027 (new, next free per DF-030), malformed `cookie:<name>` handling, `auth_type × header_scheme` coherence matrix with `cookie_roundtrip`-must-use-cookie restriction. HIGH-1: D3 supersession scope split into CONTRACT (preserved from §D3-a) vs DISPATCH MECHANISM (changed — §D3-b dispatch table is D5 amendment target); D3-c reference corrected from `§D3-a/b` to `§D3-a`. HIGH-2: D5 Spec Amendment Manifest expanded — added ADR-031 §D3-b dispatch table, BC-2.01.017 INV-COOKIE-004, BC-2.01.017 TV-BC-2.01.017-008. MED-1: D2 `header_scheme` rationale corrected — `auth_plugin.is_some()` (not `auth_type`) drives `PluginAuthProvider` construction at boot; Armis `custom_via_plugin` vs CrowdStrike `oauth2_client_credentials` distinction explained; ADR-031 §D3 supersedes frontmatter updated to reflect §D3-a/b split. |
| 0.2 | 2026-07-20 | architect | Central design decision: header injection mechanism resolved — `header_scheme` TOML field (`bearer`/`raw`/`cookie:<name>`) replaces `auth_type`-based dispatch in `build_request()`; keeps `AuthType` closed; fixes Armis raw-token + generalizes to future sensors. Armis TOML wiring specified: `auth_plugin = "armis-token-exchange"`, `header_scheme = "raw"`, `credential_ref = "secret_key"` (HIGH-4: was `armis_secret_key` — doubled sensor segment). D3-c CONFIRMED: Cyberint Alerts auth is static `Cookie: access_token` (no login step); X-Api-Key hypothesis rejected; research-agent citation added. HIGH-2 fix: ADR-031 §D3 supersession reframed as scope-narrowing (Assets only), not reversal — §D3-a/b static-cookie contract PRESERVED. D5 Spec Amendment Manifest added (BC-2.01.008, BC-2.01.017 §P2, BC-2.01.006). SR-003 Armis DTU atomic-sequencing consequence added. SR-004: SensorAuth→`AuthType` (`spec_parser.rs`)/E-SPEC-012 correction throughout. SR-005: `auth_plugin` TOML field wiring + BC-2.01.016 EC-016-002/005 cited. SR-006: Spec Amendment Manifest. SR-007: No-OpenAPI governance sentence (Confirmed/Partial tiers). SR-008: C2-class Cyberint mechanical fixes noted in D1. F3: §Source/Origin cite instructions changed from file:line to function+anchor. F4: "EIGHT of the four sensors" → "eight CRIT findings across all four sensors". OBS-5: §Changelog promoted to top-level ## Changelog. MED-1: ADR-032 added to related_adrs. |
| 0.1 | 2026-07-20 | architect | Initial proposal under D-1889 authorization |
