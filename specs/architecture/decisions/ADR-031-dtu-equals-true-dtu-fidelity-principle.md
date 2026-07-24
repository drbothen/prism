---
document_type: adr
adr_id: "ADR-031"
title: "DTU = True DTU — Fidelity Principle for All Clone Implementations"
status: accepted
date: "2026-05-29"
modified: "2026-07-24"
version: "1.9"
producer: architect
subsystems_affected: [SS-01, SS-07, SS-16, SS-17]
supersedes: ["ADR-028 §D12 (Cyberint cookie auth real-API vs DTU model divergence — DTU-shortcut acceptance SUPERSEDED)"]
superseded_by: "ADR-053 §D3 (scope-narrowing only: §D3 single-surface assumption narrowed to Assets; §D3-b items 1-2 StaticCookieAuthProvider provider contract PRESERVED — no-HTTP acquire_token, static-credential-read, no login step; §D3-b item 3 auth_type-keyed dispatch table superseded by header_scheme dispatch per ADR-053 D2/D5; §D3-a DTU changes unaffected; Alerts becomes separate surface; Alerts auth CONFIRMED static-cookie per research-agent 2026-07-20; authorized D-1889 2026-07-20; final ADR approval gate PASSED 2026-07-22 (D-1943))"
amends: null
anchor_stories: [S-DTU-CYBERINT-AUTH-FIDELITY-001]
related_adrs: [ADR-003, ADR-023, ADR-028, ADR-053, ADR-054]
related_bcs: [BC-2.16.013]
locked_decisions: []
user_directive_date: "2026-05-29"
---

# ADR-031: DTU = True DTU — Fidelity Principle for All Clone Implementations

## Status

Accepted (retroactive; frontmatter flipped v1.4 2026-07-20 per DRIFT-ADR031-STATUS-001;
§Status body synced v1.8, F-WASE-P51-MED-001). Scope-narrowed by accepted ADR-053 §D3 gate
PASSED D-1943. Originally proposed 2026-05-29, v1.0. Established by explicit user direction
2026-05-29. Supersedes ADR-028 §D12, which accepted the Cyberint `access_token` vs
`cyberint_session` divergence as deliberate and deferred real-API faithfulness to a post-demo
follow-up story. That decision is reversed here.

ADR-028 §D12 is annotated `[SUPERSEDED by ADR-031 2026-05-29 — DTU=true-DTU principle adoption]`.
The §D12 body text is preserved for traceability (per POL-1 append-only). ADR-028 frontmatter
`superseded_by: null` field for §D12 is not a full-ADR supersession — §D12 specifically is
superseded; the rest of ADR-028 remains authoritative.

---

## Context

### The DTU Value Proposition

DTU clones (`crates/prism-dtu-{sensor}/`) exist to serve two purposes:

1. **Live demo fidelity.** Engineers and customers should be able to observe prism's sensor
   data pipeline working against realistic API behavior without real cloud credentials.
   If the DTU diverges from real API behavior, the demo proves the wrong thing.

2. **Post-deployment regression testing.** When prism ships a spec change, the DTU enables
   regression tests that verify the change against real-API-shaped responses. If the DTU
   returns a different cookie name than the real API, a passing test does not prove the
   real API will work.

DTU value is FIDELITY. A DTU that accepts `cyberint_session` when the real API issues
`access_token` is a convenient test fixture, not a fidelity model. That distinction matters:
a convenient test fixture proves the plumbing compiles; a fidelity model proves the plumbing
works against the real API.

### ADR-028 §D12 — The Decision Being Superseded

ADR-028 §D12 (added 2026-05-29, architect, S-DEMO-001 v1.1 gap analysis) accepted the
divergence between:
- **Real Cyberint API (poller-express):** `Cookie: access_token={apiKey}` injected statically
  on every request; NO login step; cookie name is `access_token`.
- **DTU clone (`prism-dtu-cyberint`):** `POST /login` returns `Set-Cookie: cyberint_session={uuid}`;
  subsequent routes validate `cyberint_session` cookie; cookie name is `cyberint_session`.

§D12 ruled: "For the live demo (which runs against the DTU clone), the DTU model governs."
It locked `CookieLoginAuthProvider` to perform `POST /login → cyberint_session` and deferred
real-API faithfulness to follow-up story `S-DEMO-CYBERINT-LIVE-AUTH-001` (P2-post-demo).

### Why §D12 Was Wrong Under the DTU=True-DTU Rule

The §D12 reasoning inverted the DTU-value chain: it treated the DTU's existing implementation
as the ground truth and forced prism's auth implementation to conform to the DTU. The correct
direction is the inverse — the DTU must conform to the real API, and prism's auth
implementation must conform to the real API.

By locking `cyberint_session` as the cookie name because "the DTU uses that name," §D12
created a demo that proves prism can talk to its own DTU, not prism can talk to Cyberint.
The user has explicitly rejected this: **"the cyberint fix needs to happen pre-demo."**

### Canonical Reference for Cyberint Auth

Poller-express (`.factory/semport/poller-express/poller-express-broad-sweep.md §2.1`):

```go
type cookieTransport struct {
    apiKey string
}

func (t *cookieTransport) RoundTrip(req *http.Request) (*http.Response, error) {
    req.AddCookie(&http.Cookie{Name: "access_token", Value: t.apiKey})
    return http.DefaultTransport.RoundTrip(req)
}
```

The real Cyberint API:
- Uses `Cookie: access_token={API_KEY}` on every request.
- Does NOT require a login step.
- The API key itself IS the session token; it is static across all requests.
- Cookie name: `access_token`. Not `cyberint_session`. Not `session`. `access_token`.

The DTU must emit `access_token`. The DTU's current `cyberint_session` model is wrong.

---

## Decision

### D1 — The DTU = True DTU Principle (Binding)

DTU clone implementations (`crates/prism-dtu-{sensor}/`) MUST model real third-party API
behavior with high fidelity. Specifically:

**D1-a — Cookie and header names.** DTU MUST use exactly the field names, header names, and
cookie names the real API uses. A DTU that issues `cyberint_session` when the real API uses
`access_token` violates this rule. The canonical reference for field names is the production
poller semport ingest document (`.factory/semport/poller-{sensor}/`).

**D1-b — Auth flow.** DTU MUST implement exactly the auth flow the real API requires:
- If the real API requires OAuth2 client credentials → DTU implements OAuth2 token endpoint.
- If the real API requires static Bearer → DTU enforces Bearer header.
- If the real API requires static cookie injection (no login step) → DTU MUST also accept
  static cookie injection. A DTU that requires a login step when the real API does not is
  a fidelity violation.

**D1-c — Endpoint structure.** DTU MUST register exactly the endpoints the real API exposes
(per D5 of ADR-028, which is not superseded). No extra routes, no missing routes.

**D1-d — Response field names.** DTU MUST return response fields with the same names the
real API uses. Where prism's TOML spec uses an alias (e.g., `alert_id` instead of `ref_id`),
the DTU must return the real field name and the TOML spec must have a column mapping from the
real name to prism's internal name.

**D1-e — Request validation.** DTU auth validation MUST be strong enough to reject requests
that would fail against the real API. A DTU that accepts any non-empty Bearer token for a
sensor that requires a specific format provides weaker coverage than the real API.

**D1-f — Fixture data.** DTU MAY use synthetic (non-real) customer data. The data SHAPE
must match the real API: same field names, same types, same nullable semantics. Pagination
behavior must match real exactly.

### D2 — Permitted Divergences (Enumerated, Exhaustive)

The following divergences from real API behavior are explicitly permitted with justification:

**D2-a — Rate limit cooldowns.** If the production API has a rate limit of N requests per
minute with a documented cooldown, the DTU MAY use a shorter cooldown (e.g., 100ms instead
of 60s) to allow test suites to run in reasonable time. The production rate limit value MUST
be documented in the DTU source as a comment citing the real API limit.

**D2-b — Credential format validation.** The DTU MAY accept any syntactically valid credential
value (e.g., any UUID as a Bearer token) rather than requiring real cloud-issued credentials.
The DTU MUST NOT accept structurally invalid values — for example, if the real API requires
a specific format (like the CrowdStrike `access_token` JWT), the DTU must enforce that format
in its token validation.

**D2-c — TLS/HTTPS.** DTU clones bind to `127.0.0.1` over plain HTTP for local use.
This is acceptable because the DTU is never exposed to a network; it is a loopback fixture.

**D2-d — Persistence semantics.** DTU state (e.g., registered sessions, mutation history)
resets on server restart. Real APIs have persistent state. This divergence is acceptable
because DTU use cases are per-test-run.

All other DTU behavior divergences require explicit architectural justification in an ADR
§D section with the following structure:
- Gap ID (e.g., DTU-DIV-NNN)
- Real API behavior (cited to poller semport or real API documentation)
- DTU behavior
- Justification for why this specific divergence is acceptable
- Follow-up story ID to close the divergence (if applicable)

**v1.2 Amendment — Sensor-specific "acceptable divergences" removed.** Three sensor-specific
divergences previously listed in §D6 as "acceptable per D2 exception" are NOT D2-permitted.
They were incorrectly classified as acceptable in §D6 v1.0/v1.1 while awaiting Wave 5
capacity. Per user directive 2026-05-31 ("all sensors, best-in-class, no scope compromises"),
these are reclassified as REQUIRED fidelity tracked in Wave 5:
- Armis AQL endpoint divergence (Gap-AR-001) — required via `S-DEMO-ARMIS-AQL-001`
- Claroty trailing-slash route fidelity (Gap-CL-001) — required via `S-DEMO-CLAROTY-TRAILING-SLASH-001`
- CrowdStrike multi-region `base_url` (Gap-CS-003) — required via `S-DEMO-CROWDSTRIKE-MULTIREGION-001`

D2-a through D2-d remain permitted. No other changes to permitted divergences.

### D3 — Cyberint DTU Correction (Pre-Demo CRITICAL)

Per D1 and the reversal of ADR-028 §D12, the Cyberint DTU (`prism-dtu-cyberint`) MUST be
corrected to emit `access_token` (matching poller-express real-API behavior) before the
live demo ships.

#### D3-a — DTU Changes Required

The DTU auth model changes from stateful session (login step → `cyberint_session`) to
stateless static-cookie (no login step; `access_token` validated on every request):

1. **Remove (or repurpose) `POST /login` route.** The real Cyberint API has no login step.
   The `POST /login` route (`routes/auth.rs::post_login`) must be REMOVED or demoted to a
   no-op stub that returns 200 without issuing any cookie. This route does not correspond
   to a real Cyberint API endpoint.

2. **Change `extract_session_token()` in `routes/alerts.rs`.** Currently extracts
   `cyberint_session` cookie. Must be renamed/rewritten to `extract_access_token()` that
   extracts the `access_token` cookie value:
   ```rust
   pub fn extract_access_token(headers: &HeaderMap) -> Option<String> {
       let cookie_header = headers.get("cookie")?.to_str().ok()?;
       for pair in cookie_header.split(';') {
           let pair = pair.trim();
           if let Some(val) = pair.strip_prefix("access_token=") {
               return Some(val.to_owned());
           }
       }
       None
   }
   ```

3. **Session store becomes static-auth registry.** The `CyberintState.session_store` no
   longer tracks login-issued tokens. It becomes an `access_token` allowlist: at startup,
   the demo server registers the demo `access_token` value (from its config) as an allowed
   token. Auth validation checks `access_token` cookie against the allowlist. This still
   validates auth without requiring a real login exchange.

4. **`check_auth` updated.** Must call `extract_access_token()` instead of
   `extract_session_token()`, and validate against the static-token store.

#### D3-b — Prism Changes Required

With the DTU corrected to accept `access_token` static cookie, prism's auth implementation
for Cyberint MUST also change:

1. **`auth_type = "cookie_roundtrip"` label is now misleading.** The label is preserved
   (per D-747 LOCKED in ADR-028) because changing `auth_type` values has blast-radius
   across the plugin runtime contract. However, the **behavior** of `CookieRoundtrip` for
   Cyberint changes: instead of performing a login step, the implementation injects the
   API key as a static `Cookie: access_token={api_key}` header.

2. **`CookieLoginAuthProvider` is NOT the correct implementation for the DTU-correct path.**
   The implementation for Cyberint is a new `StaticCookieAuthProvider` (or the existing type
   renamed) that:
   - Reads the API key from the credential store at acquire-token time (NOT at construction
     time per AD-017).
   - Returns the raw API key as the "token" value.
   - Does NOT perform any HTTP request during `acquire_token`.

3. **`build_request` (module-level free function in `crates/prism-spec-engine/src/pipeline.rs`) dispatch for `CookieRoundtrip`.** Must inject
   `Cookie: access_token={token}` (NOT `Cookie: cyberint_session={token}`). The cookie name
   changes from `cyberint_session` to `access_token`.

   Updated dispatch table:

   | AuthType | Header injected |
   |----------|----------------|
   | `CookieRoundtrip` | `Cookie: access_token={token}` |
   | `BearerStatic` | `Authorization: Bearer {token}` |
   | `Oauth2ClientCredentials` | `Authorization: Bearer {token}` |
   | `CustomViaPlugin` | `Authorization: Bearer {token}` |

4. **S-DEMO-001 story scope expands.** The Cyberint auth implementation in S-DEMO-001 must
   implement the correct pattern from the start. `CookieLoginAuthProvider` (which performs
   a login step) is not the right type — the production type for the DTU-correct Cyberint
   auth path is a stateless provider that holds NO session state.

#### D3-c — Story That Delivers This Correction

**`S-DTU-CYBERINT-AUTH-FIDELITY-001`** (renaming and reclassification of
`S-DEMO-CYBERINT-LIVE-AUTH-001` from P2-post-demo to P0-pre-demo-BLOCKING).

This story is NOT a "live auth follow-up" — it is the canonical Cyberint auth
implementation. It MUST merge before or simultaneously with S-DEMO-001. Any demo against
the Cyberint DTU that uses the `cyberint_session` model is demonstrating incorrect behavior.

### D4 — ADR-028 §D12 Supersession

ADR-028 §D12 is superseded by this ADR-031 D3. Specifically:

- The `build_request` dispatch table in §D12 (`CookieRoundtrip → Cookie: cyberint_session={token}`)
  is REPLACED by the dispatch table in §D3-b above (`CookieRoundtrip → Cookie: access_token={token}`).
- The deferred story `S-DEMO-CYBERINT-LIVE-AUTH-001` (P2-post-demo) is RECLASSIFIED to
  `S-DTU-CYBERINT-AUTH-FIDELITY-001` (P0-pre-demo-BLOCKING).
- `CookieLoginAuthProvider` (login-step implementation) is NOT the correct type for the
  DTU-correct Cyberint auth path. It is a wrong-by-construction implementation if written
  per §D12's contract.

ADR-028 §D12 retains its body text (POL-1 append-only) but is annotated:
`[SUPERSEDED by ADR-031 2026-05-29 — DTU=true-DTU principle adoption — D3 is the correct contract]`

### D5 — Validation Discipline

Per ADR-003 §D3 (DTU Fidelity Auth), the dtu-validator agent validates DTU behavior
against real API semport documentation before any story that modifies DTU routes can merge.

Per this ADR, parity tests for every sensor MUST:
1. Assert cookie/header NAMES match real API (not just that auth passes).
2. Assert response field names match real API field names (per SAP-2 standing probe).
3. Assert endpoint paths match real API paths where paths are known.

A parity test that passes with `cyberint_session` does NOT constitute evidence of Cyberint
fidelity — it is evidence of internal consistency within a wrong model. The adversary MUST
probe for this distinction per SAP-2.

### D7 — Scope Expansion: Harness Clones are In-Scope for DTU=True-DTU (v1.1 Amendment)

**Trigger:** F-LP1-OBS-001 [process-gap] from S-DTU-CYBERINT-AUTH-FIDELITY-001 Pass 1
LOCAL adversary cascade (2026-05-30). The adversary found that
`crates/prism-dtu-harness/src/clones/cyberint.rs` still uses `cyberint_session` cookie
+ `POST /login` auth model, violating ADR-031 §D1. This file was NOT in scope of the
prior architect audit (POLLER-DTU-FIDELITY-AUDIT-2026-05-29 v1.1) because the audit
enumerated `crates/prism-dtu-{sensor}/src/` paths but not the harness-clones path.

**Amendment to D1 (scope extension):**

The following sentence is added to §D1 to make the enumeration of in-scope paths explicit:

> DTU clone implementations governed by this ADR include ALL behavioral clones of sensor
> APIs in the workspace, regardless of crate path. Explicitly enumerated:
>
> - `crates/prism-dtu-{sensor}/src/clone.rs` and `crates/prism-dtu-{sensor}/src/routes/`
>   (canonical standalone DTU clone crates)
> - `crates/prism-dtu-harness/src/clones/{sensor}.rs` (harness-embedded behavioral clones)
> - Any future harness, fixture, or integration-test clone added under any `crates/prism-dtu-*/`
>   or `crates/*-harness/` path

**Audit outcome (HARNESS-DTU-FIDELITY-AUDIT-2026-05-30):**

Full audit results are in `.factory/proposals/HARNESS-DTU-FIDELITY-AUDIT-2026-05-30.md`.
Summary per sensor:

| Sensor | Harness clone ADR-031 D1 violations | Severity | Harness change required |
|--------|-------------------------------------|----------|------------------------|
| Cyberint | 4 CRITICAL (cookie name `cyberint_session` vs `access_token`; login step; auth validator; session store) + 1 HIGH (POST /login route) | CRITICAL | YES — simultaneous with canonical DTU fix in S-DTU-CYBERINT-AUTH-FIDELITY-001 |
| CrowdStrike | None | — | NO |
| Claroty | 1 HIGH — missing `/api/v1/audit_log/get` route (same as canonical Gap-CL-006) | HIGH | YES — simultaneous with canonical DTU fix in S-DEMO-CLAROTY-AUDIT-DTU-001 |
| Armis | None | — | NO |

**Remediation pattern (Pattern B — in-place rewrite, adopted):**

Harness clones are rewritten in-place to implement the correct fidelity model. Harness
clones do NOT delegate to canonical DTU crates at runtime (maintaining the existing
architectural choice to avoid circular dev-dependency chains). Fixture data sharing via
`include_str!` is preserved for Claroty and Armis. Cyberint harness clone rewrite is
in-scope for S-DTU-CYBERINT-AUTH-FIDELITY-001 (expanded scope).

**Scope decision: Scope-1 (in-current-story) for Cyberint:**

The Cyberint harness clone fix is added to S-DTU-CYBERINT-AUTH-FIDELITY-001 scope.
Both `prism-dtu-cyberint` (canonical DTU) and `prism-dtu-harness/src/clones/cyberint.rs`
(harness clone) share the same wrong auth model and MUST be corrected in the same PR.
The Claroty audit_log harness gap is co-scoped with S-DEMO-CLAROTY-AUDIT-DTU-001.

**Process-gap codification (F-LP1-OBS-001):**

Architect audits that scope DTU clone paths MUST explicitly enumerate ALL paths where
behavioral clones live, not just the `crates/prism-dtu-{sensor}/src/` prefix. The correct
audit enumeration for DTU fidelity is:

```
sources_read:
  - crates/prism-dtu-{sensor}/src/clone.rs
  - crates/prism-dtu-{sensor}/src/routes/{relevant_routes}.rs
  - crates/prism-dtu-harness/src/clones/{sensor}.rs       # REQUIRED — do not omit
  - crates/prism-dtu-{sensor}/src/state.rs
```

See lessons.md entry 54 for the process-gap codification.

### D6 — Cross-Sensor Applicability

The DTU=True-DTU principle applies to ALL four sensor DTU clones. The cross-sensor
applicability assessment for each remaining sensor is in the POLLER-DTU-FIDELITY-AUDIT
revision (v1.1, same burst). Summary per sensor:

| Sensor | DTU conformance status | Correction required |
|--------|------------------------|---------------------|
| CrowdStrike | Auth (OAuth2 token endpoint): CORRECT. Response field names: `detection_id` fix applied (develop@72baf413). Base URL hardcoded to us-1 (Gap-CS-003). | Multi-region `base_url` routing REQUIRED — tracked in `S-DEMO-CROWDSTRIKE-MULTIREGION-001` (Wave 5). Previously incorrectly classified as D2-permitted; reclassified 2026-05-31 per user directive. |
| Claroty xDome | Auth (Bearer static): CORRECT. Devices route: CORRECT. Audit log route gap: DTU-EXT gap tracked. Trailing slash on endpoint paths (Gap-CL-001). | Audit log DTU route (`/api/v1/audit_log/get`) required via `S-DEMO-CLAROTY-AUDIT-DTU-001`. Trailing-slash route fidelity REQUIRED — tracked in `S-DEMO-CLAROTY-TRAILING-SLASH-001` (Wave 5). Previously incorrectly classified as D2-permitted; reclassified 2026-05-31 per user directive. |
| Armis Centrix | Auth (Bearer static): CORRECT. Current DTU exposes direct REST endpoints (`GET /api/v1/devices`, `GET /api/v1/alerts`); production uses AQL search endpoint (`GET /api/v1/search?aql=<query>`) for all data (Gap-AR-001, DTU-EXT-003/004). | Full AQL endpoint fidelity REQUIRED — tracked in `S-DEMO-ARMIS-AQL-001` (Wave 5). Previously incorrectly classified as D2-permitted; reclassified 2026-05-31 per user directive. |
| Cyberint Argos | Auth: CORRECTED — `access_token` cookie static injection, no login step (`S-DTU-CYBERINT-AUTH-FIDELITY-001` merged). | No remaining REQUIRED fidelity gaps for DTU path. |

### D8 — Wave 5 Fidelity Reclassification (v1.2 Amendment)

**Trigger:** User directive 2026-05-31: "all sensors, best-in-class, no scope compromises."
Three sensor-specific gaps previously recorded as post-demo acceptable divergences in §D6
are reclassified from "permitted/deferred" to REQUIRED fidelity, each anchored to an existing
Wave 5 story. D2-a through D2-d (generic local-DTU constraints) remain permitted and unchanged.

#### D8-a — Armis AQL Endpoint Fidelity (S-DEMO-ARMIS-AQL-001)

**Gap ID:** Gap-AR-001 (DTU-EXT-003/004)

**Prior classification (WRONG):** "AQL endpoint divergence acceptable per D2 exception process;
documented in `S-DEMO-ARMIS-AQL-001`." The v1.0/v1.1 §D2 exception was granted on the grounds
that the real Armis API also exposes direct REST endpoints (`GET /api/v1/devices`,
`GET /api/v1/alerts`) alongside the AQL endpoint — making it a "valid call pattern."

**Reclassification (v1.2):** REQUIRED fidelity. The production poller-coaster uses AQL
(`GET /api/v1/search?aql=<query>`) for ALL data sources. Prism's direct-endpoint DTU path
proves only that prism can talk to a non-production Armis call pattern; it does NOT prove
prism can query Armis the way the production poller does. Full AQL fidelity requires:
1. `prism-dtu-armis`: add `GET /api/v1/search` route that accepts an `aql=` query parameter,
   applies the AQL string as a filter against the fixture data, logs the AQL string to the
   AQL log (existing `aql_log` mechanism), and returns the filtered device or alert records
   in the same envelope as the current direct-endpoint routes.
2. `armis.sensor.toml`: update `devices` and `alerts` table steps to use
   `path_template = "/api/v1/search"` with `aql` query parameter forwarding (replacing or
   supplementing the current `GET /api/v1/devices` / `GET /api/v1/alerts` paths).
3. Parity tests: assert that the AQL string received by the DTU matches the AQL string prism
   constructs from the sensor spec (validates AQL push-down per R-DTU-002).

**Current DTU state (grounded):** `crates/prism-dtu-armis/src/routes/devices.rs` — handles
`GET /api/v1/devices` and `POST /api/v1/devices` with `aql` parameter capture but no
`/api/v1/search` route. `crates/prism-dtu-armis/src/routes/alerts.rs` — handles
`GET /api/v1/alerts`. AQL log mechanism exists (`state.capture_aql()`, `GET /dtu/aql-log`).
The infrastructure for AQL capture is in place; the missing piece is the search route itself.

**TOML impact:** YES — `devices` and `alerts` table steps change from
`path_template = "/api/v1/devices"` / `path_template = "/api/v1/alerts"` to
`path_template = "/api/v1/search"` with AQL parameter forwarding. Existing columns are
preserved (field names are already grounded against DTU fixture data; the search route will
return the same fields).

**Wave 5 story:** `S-DEMO-ARMIS-AQL-001` (existing stub, elevated from P2-post-demo to
REQUIRED Wave 5 fidelity work).

**New BCs needed:** No new behavioral contracts required. The AQL endpoint is a pipeline
implementation change. The query-filter AQL push-down is covered by existing R-DTU-002
and the `armis_devices` / `armis_alerts` DataFusion table contracts. If AQL syntax
validation is added to the DTU, BC-2.01.NNN may need a new EC row — flag to product-owner
if and when the story writer discovers AQL validation scope.

#### D8-b — Claroty Trailing-Slash Route Fidelity (S-DEMO-CLAROTY-TRAILING-SLASH-001)

**Gap ID:** Gap-CL-001

**Prior classification (WRONG):** "MEDIUM — trailing slash matters for live Claroty, not for
DTU. DTU axum routes match with/without trailing slash via Axum normalization. Follow-up
S-DEMO-CLAROTY-TRAILING-SLASH-001 (P2)."

**Reclassification (v1.2):** REQUIRED fidelity. The real Claroty xDome API uses trailing
slashes on all POST-for-read endpoints (`/api/v1/alerts/`, `/api/v1/devices/`,
`/api/v1/audit_log/get/`). Prism's TOML spec currently uses paths WITHOUT trailing slash.
This means:
- When prism talks to a real Claroty instance, the request paths will lack the required
  trailing slash, which may cause 301 redirects or 404s depending on Claroty's server
  configuration.
- The DTU's Axum route normalization masks this gap at demo time.

Full trailing-slash fidelity requires:
1. `claroty.sensor.toml`: update all `path_template` values to include trailing slashes
   matching the real Claroty API (`/api/v1/alerts/`, `/api/v1/devices/`,
   `/api/v1/audit_log/get/`). Grounded from poller-bear semport §API table.
2. `prism-dtu-claroty`: verify Axum normalizes trailing-slash vs non-trailing-slash
   consistently, or explicitly register both route forms. Current Axum behavior handles
   this but must be verified by a parity test that sends trailing-slash paths.
3. Parity tests: assert that the DTU accepts the trailing-slash paths that prism will
   send after the TOML update.

**Current DTU state (grounded):** `crates/prism-dtu-claroty/src/routes/alerts.rs` —
route registered as `POST /api/v1/alerts` (no trailing slash). Axum's default behavior
does NOT automatically redirect or accept trailing-slash variants unless
`axum::middleware::normalize_path()` is in the layer stack. Verify whether the Claroty DTU
router includes `normalize_path` before assuming Axum handles it transparently.

**TOML impact:** YES — all three `path_template` values change:
- `alerts` table: `/api/v1/alerts` → `/api/v1/alerts/`
- `devices` table: `/api/v1/devices` → `/api/v1/devices/`
- `audit_logs` table: `/api/v1/audit_log/get` → `/api/v1/audit_log/get/`

**Wave 5 story:** `S-DEMO-CLAROTY-TRAILING-SLASH-001` (existing stub, elevated from
P2-post-demo to REQUIRED Wave 5 fidelity work).

**New BCs needed:** No new behavioral contracts. The trailing-slash change is a request-path
fidelity fix with no behavioral semantics change. The parity test addition (AC for the story)
is sufficient.

#### D8-c — CrowdStrike Multi-Region Base URL Fidelity (S-DEMO-CROWDSTRIKE-MULTIREGION-001)

**Gap ID:** Gap-CS-003

**Prior classification (WRONG):** "Multi-region routing not in TOML. Demo environment is
us-1. Not a demo blocker. Follow-up S-DEMO-CROWDSTRIKE-MULTIREGION-001 (P3)."

**Reclassification (v1.2):** REQUIRED fidelity. The real CrowdStrike Falcon API is
region-routed: MSSP clients may have tenants on us-1 (`api.crowdstrike.com`), us-2
(`api.us-2.crowdstrike.com`), eu-1 (`api.eu-1.crowdstrike.com`), and gov (`api.laggar.gcw.crowdstrike.com`).
Prism's `crowdstrike.sensor.toml` currently hardcodes `base_url = "https://api.crowdstrike.com"`
(us-1 only). An MSSP with eu-1 tenants cannot use prism as-is.

Full multi-region fidelity requires:
1. `crowdstrike.sensor.toml`: change `base_url` from the hardcoded `https://api.crowdstrike.com`
   to a per-instance env variable reference: `base_url = "${env.CROWDSTRIKE_BASE_URL}"`.
   The TOML environment-variable substitution mechanism (`${env.VAR}`) already exists for
   Armis (`${env.ARMIS_INSTANCE_URL}`) and Claroty (`${env.CLAROTY_INSTANCE_URL}`).
2. Documentation/runbook: document the base URLs for each supported CrowdStrike region so
   MSSP operators can configure per-tenant `CROWDSTRIKE_BASE_URL`.
3. `prism-dtu-crowdstrike`: the DTU itself is already region-agnostic — it binds to
   `127.0.0.1:0` and accepts any valid OAuth2 + Bearer flow regardless of base URL.
   No DTU code change required. Verify DTU config tests still pass when TOML `base_url`
   is set from env var (the DTU test harness already overrides `base_url` via harness config).
4. Parity tests: add a test that verifies the sensor spec loads correctly with a non-us-1
   `CROWDSTRIKE_BASE_URL` env var (e.g., `api.eu-1.crowdstrike.com`).

**Current DTU state (grounded):** `crates/prism-dtu-crowdstrike/src/state.rs` — no
`base_url` field in `CrowdstrikeState`; DTU is URL-agnostic. `crates/prism-dtu-crowdstrike/src/routes/oauth.rs` —
issues `access_token: "dtu-fake-cs-token"` regardless of URL. No DTU code changes needed.

**TOML impact:** YES — `base_url` changes from `"https://api.crowdstrike.com"` to
`"${env.CROWDSTRIKE_BASE_URL}"` (matching the env-var pattern used by Armis and Claroty).
Existing `auth_type = "oauth2_client_credentials"` and `auth_plugin = "crowdstrike-oauth2"`
are unchanged.

**Wave 5 story:** `S-DEMO-CROWDSTRIKE-MULTIREGION-001` (existing stub, elevated from
P3-post-demo to REQUIRED Wave 5 fidelity work).

**New BCs needed:** Potentially. If the sensor config loading behavior for `${env.VAR}`
substitution does not already have a BC covering multi-sensor env-var resolution, the
product-owner should evaluate whether BC-2.01.NNN (spec loading) needs a new AC for
`CROWDSTRIKE_BASE_URL` env-var substitution. Flag to product-owner. If the env-var
substitution is already covered by existing BCs (Armis/Claroty use the same pattern),
no new BC is needed.

---

## Rationale

The DTU = True-DTU principle rests on a single claim: a test that passes against a DTU with
the wrong auth model or wrong cookie name proves nothing about real-API compatibility. The
Cyberint `cyberint_session` vs `access_token` discrepancy (the immediate trigger for this
ADR) is exactly this failure: every parity test that passed gave false confidence because the
DTU enforced a different cookie name than the real API. A demo that "works" by talking to its
own fabricated DTU does not constitute evidence that Prism works against the real sensor.

The user directive (2026-05-29) that establishes this ADR is unambiguous: real-API fidelity
is not a post-demo enhancement; it is the definition of correctness. The demo's evidentiary
value — its ability to demonstrate to customers and engineers that Prism correctly connects
to real sensors — depends entirely on the DTU being a faithful representation. Approximate
fidelity and deliberate divergence are indistinguishable from bugs to a downstream observer.

The alternative (defer real-API fidelity to follow-up stories) was explicitly evaluated and
rejected. Deferral creates a class of tests that pass and a class of runtime behaviors that
fail — exactly the failure mode that caused S-DEMO-001's auth outage. The only acceptable
approach is that DTU fidelity is the invariant, not the goal.

---

## Consequences

### Positive

- **Demo proves real-world compatibility.** When prism talks to the Cyberint DTU using
  `access_token` cookie, the demo proves the same pipeline will work against a real
  Cyberint instance. The demo has EVIDENTIARY VALUE.

- **No "fake pass" in parity tests.** Parity tests that pass against a DTU using the wrong
  cookie name gave false confidence. With DTU correction, a passing parity test means the
  real API would also pass.

- **Eliminates the "live-auth follow-up" pattern.** There is no "we'll fix this for real API
  later" — the real API behavior IS the demo behavior. The follow-up is eliminated, not
  deferred.

- **Establishes the validation discipline across all 4 sensors.** The cross-sensor audit
  and per-sensor D6 assessment ensures Claroty, Armis, and CrowdStrike are evaluated under
  the same lens.

### Negative

- **S-DEMO-001 scope expands.** The Cyberint auth implementation changes from
  `CookieLoginAuthProvider` (HTTP login step + session cookie) to `StaticCookieAuthProvider`
  (no HTTP call + static API key as cookie). This is a different implementation pattern;
  the S-DEMO-001 test-writer and implementer must work from the revised AC-003/AC-009.

- **DTU `prism-dtu-cyberint` requires code changes.** The `POST /login` route becomes
  unnecessary; `extract_session_token()` becomes `extract_access_token()`. The DTU state
  model changes. This is a code change that must go through the per-story-delivery pipeline
  (`S-DTU-CYBERINT-AUTH-FIDELITY-001`).

- **Existing tests in `prism-dtu-cyberint` must be updated.** Any test that sends
  `cyberint_session` cookie to the DTU must be rewritten to send `access_token` cookie.
  The DTU test suite is the correct place to verify fidelity.

---

## Alternatives Considered

### Alt 1: Extend ADR-028 §D13 instead of new ADR-031

Rejected. The DTU=True-DTU principle is a binding FOUNDATIONAL rule that applies across
all 4 sensors and all future sensor integrations. It is not an amendment to a specific
decision axis (URL grounding). ADR-028 covers TOML spec URL and auth_type grounding against
DTU routes. This ADR covers DTU route grounding against real API behavior — a different
and higher-level principle. A standalone ADR makes the principle citable across stories
without requiring readers to parse ADR-028's 12-section body.

### Alt 2: Accept §D12 and ship the demo with `cyberint_session`

Rejected by user direction 2026-05-29: "the cyberint fix needs to happen pre-demo."
The user explicitly rejected the pragmatic convergence deferral. No further justification
needed — this is a direct user directive.

### Alt 3: Add `auth_cookie_name` TOML field to parameterize cookie name

Considered as a mechanism for prism-side, not for the DTU. The DTU must change its cookie
name regardless — the prism side cookie name is a consequence of the DTU correction, not
the cause. The `auth_cookie_name` TOML field may still be useful later for sensors with
non-standard cookie names, but it does not resolve the DTU fidelity problem.

---

## Source / Origin

- **User directive (2026-05-29):** Explicit direction to fix the Cyberint auth fidelity
  pre-demo. Establishes the DTU=True-DTU principle as binding across all four sensors.
  Recorded in STATE.md and confirmed in orchestrator adjudication session leading to this ADR.
- **Behavioral contracts:** BC-2.16.013 (sensor spec correctness contract), BC-2.01.006
  (Cyberint auth contract), BC-2.16.001 (parity test contract).
- **Reference implementation:** `.factory/semport/poller-express/poller-express-broad-sweep.md §2.1`
  — Go `cookieTransport` implementation confirming `access_token` cookie injection without
  login step. This is the canonical real-API behavioral evidence.
- **Prior art:** ADR-028 §D12 (the superseded decision) and ADR-003 (DTU Reset Lookup and
  Fidelity Auth) — the predecessor DTU fidelity framework that this ADR extends.

---

## Related Decisions

- **Supersedes ADR-028 §D12** (Cyberint cookie auth real-API vs DTU model divergence — the
  DTU-shortcut acceptance is reversed).
- **Extends ADR-003** (DTU Reset Lookup and Fidelity Auth) by specifying that fidelity
  means real API field names, auth flows, and cookie names — not just structural similarity.
- **Constrains ADR-028 §D2** (auth_type grounding rule) — the auth_type grounding rule
  now requires that the DTU enforcement behavior itself be faithful to the real API, so
  grounding against DTU is only sufficient if the DTU has already been verified against
  real API behavior.
- **References poller semport ingest documents** as the canonical source of truth for
  real API behavior: `.factory/semport/poller-{sensor}/poller-{sensor}-broad-sweep.md`.

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.9 | 2026-07-24 | architect | F-WASE-P52-LOW-001 POL-29 class sweep: live-body `PipelineExecutor::build_request` citation at §D3 item 3 corrected to accurate free-function form — first mention in document expanded to `` `build_request` (module-level free function in `crates/prism-spec-engine/src/pipeline.rs`) ``. `PipelineExecutor::build_request` does not resolve; `build_request` is a module-level free function at `pipeline.rs:975` (8 params, no `&self`). `version` bumped 1.8→1.9. |
| 1.8 | 2026-07-24 | architect | F-WASE-P51-MED-001: §Status body synced to accepted state — frontmatter was flipped to `accepted` at v1.4 (DRIFT-ADR031-STATUS-001 2026-07-20) but §Status body opening still read "Proposed 2026-05-29, v1.0". Retroactive-acceptance pattern applied (mirrors ADR-026 v1.41 / ADR-028 v1.27). Scope-narrowed-by ADR-053 §D3 gate PASSED D-1943 noted in §Status opening. Class-closing audit confirmed ADR-031 is the only accepted ADR with this drift pattern. |
| 1.7 | 2026-07-24 | architect | F-WASE-P50-MED-001: `superseded_by` ADR-053 §D3 annotation — "final ADR approval gate pending" → "final ADR approval gate PASSED 2026-07-22 (D-1943)". |
| 1.6 | 2026-07-21 | architect | OBS-1: ADR-054 added to `related_adrs` — soft symmetry with ADR-054's `related_adrs: [..., ADR-031]`; ADR-054 §D4 implementation contract references `AuthProvider` (the trait `StaticCookieAuthProvider` defined in ADR-031 §D3-b also implements). |
| 1.5 | 2026-07-20 | architect | HIGH-1 (ADR-053 pass-3 paper-fix): `superseded_by` frontmatter corrected — §D3-a (DTU changes) is UNAFFECTED by ADR-053 dispatch change; §D3-b items 1-2 (StaticCookieAuthProvider provider contract) are PRESERVED; §D3-b item 3 (auth_type-keyed dispatch table) is superseded by header_scheme dispatch per ADR-053 D2/D5. Previous framing incorrectly cited `§D3-a static-cookie CONTRACT PRESERVED` — §D3-a is DTU-only and contains no StaticCookieAuthProvider. Correct attribution anchors the provider contract to §D3-b items 1-2 where it actually lives. Closes ADR-053 v0.4 HIGH-1. |
| 1.4 | 2026-07-20 | architect | OBS-1 / DRIFT-ADR031-STATUS-001: `status: Proposed` → `status: accepted`. §D3-b (Cyberint static-cookie `build_request()` dispatch correction, `CookieRoundtrip → Cookie: access_token={token}`) is shipped and realized in `crates/prism-spec-engine/src/pipeline.rs` via `S-DTU-CYBERINT-AUTH-FIDELITY-001`. An accepted ADR can be partially superseded. `superseded_by` updated to reflect §D3-a/b split per ADR-053 v0.3: §D3-a static-cookie CONTRACT preserved; §D3-b auth_type-keyed dispatch table superseded by `header_scheme` dispatch (ADR-053 D2/D5). Closes DRIFT-ADR031-STATUS-001. |
| 1.3 | 2026-07-20 | architect | ADR-053 §D3 supersession linkage applied: `superseded_by:` corrected from §D3-a framing (HIGH-2 fix — §D3-a is the DTU-correction direction, not the login-flow portion; the old framing risked an implementer reintroducing `POST /login`). Correct framing: §D3 scope-narrowed to Assets surface only; §D3-a/b static-cookie contract PRESERVED; Alerts becomes a separate surface under ADR-053 D3-a. `related_adrs` updated to include ADR-053 (previous session). Cyberint Alerts auth CONFIRMED: static `Cookie: access_token` (research-agent 2026-07-20), no login step — consistent with `StaticCookieAuthProvider` / BC-2.01.017. |
| 1.2 | 2026-05-31 | architect | Wave 5 fidelity reclassification — per user directive 2026-05-31 ("all sensors, best-in-class, no scope compromises"), three sensor-specific divergences incorrectly classified as D2-permitted in §D6 v1.0/v1.1 are reclassified as REQUIRED fidelity: (1) Armis AQL endpoint (Gap-AR-001/DTU-EXT-003/004) → `S-DEMO-ARMIS-AQL-001` Wave 5 required; (2) Claroty trailing-slash route paths (Gap-CL-001) → `S-DEMO-CLAROTY-TRAILING-SLASH-001` Wave 5 required; (3) CrowdStrike multi-region `base_url` (Gap-CS-003) → `S-DEMO-CROWDSTRIKE-MULTIREGION-001` Wave 5 required. §D2 amended to close the three sensor-specific loopholes (D2-a..d unchanged). §D6 Cross-Sensor Applicability table updated to reflect REQUIRED status. New §D8 Wave 5 Fidelity Reclassification section documents precise scope, current DTU gap, TOML impact, and per-story requirements for each of the three reclassified stories. |
| 1.1 | 2026-05-30 | architect | Scope expansion amendment — adds §D7 extending DTU=true-DTU binding to all harness-clone paths (`crates/prism-dtu-harness/src/clones/{sensor}.rs`) per F-LP1-OBS-001 [process-gap] from S-DTU-CYBERINT-AUTH-FIDELITY-001 Pass 1 LOCAL adversary cascade. Harness audit (`HARNESS-DTU-FIDELITY-AUDIT-2026-05-30.md`): Cyberint CRITICAL violations (4 CRIT + 1 HIGH; fixed in S-DTU-CYBERINT-AUTH-FIDELITY-001 expanded scope); Claroty HIGH gap (audit_log route, co-scoped with S-DEMO-CLAROTY-AUDIT-DTU-001); CrowdStrike and Armis CLEAN. Remediation pattern: Pattern B (in-place rewrite). Scope decision: Scope-1 for Cyberint. Process-gap lesson 54 codified. |
| 1.0 | 2026-05-29 | architect | Initial version — establishes DTU=True-DTU as binding architectural principle per user directive 2026-05-29. Supersedes ADR-028 §D12. Defines D1 (six fidelity requirements), D2 (exhaustive list of permitted divergences), D3 (Cyberint DTU correction), D4 (§D12 supersession), D5 (validation discipline), D6 (cross-sensor applicability). Anchor story: S-DTU-CYBERINT-AUTH-FIDELITY-001 (reclassified from P2-post-demo to P0-pre-demo-BLOCKING). |
