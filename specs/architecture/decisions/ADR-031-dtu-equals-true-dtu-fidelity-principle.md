---
document_type: adr
adr_id: "ADR-031"
title: "DTU = True DTU — Fidelity Principle for All Clone Implementations"
status: Proposed
date: "2026-05-29"
modified: "2026-05-29"
version: "1.0"
producer: architect
subsystems_affected: [SS-01, SS-07, SS-16, SS-17]
supersedes: ["ADR-028 §D12 (Cyberint cookie auth real-API vs DTU model divergence — DTU-shortcut acceptance SUPERSEDED)"]
superseded_by: null
amends: null
anchor_stories: [S-DTU-CYBERINT-AUTH-FIDELITY-001]
related_adrs: [ADR-003, ADR-023, ADR-028]
related_bcs: [BC-2.16.013]
locked_decisions: []
user_directive_date: "2026-05-29"
---

# ADR-031: DTU = True DTU — Fidelity Principle for All Clone Implementations

## Status

Proposed 2026-05-29, v1.0. Established by explicit user direction 2026-05-29. Supersedes
ADR-028 §D12, which accepted the Cyberint `access_token` vs `cyberint_session` divergence as
deliberate and deferred real-API faithfulness to a post-demo follow-up story. That decision
is reversed here.

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

3. **`PipelineExecutor::build_request` dispatch for `CookieRoundtrip`.** Must inject
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

### D6 — Cross-Sensor Applicability

The DTU=True-DTU principle applies to ALL four sensor DTU clones. The cross-sensor
applicability assessment for each remaining sensor is in the POLLER-DTU-FIDELITY-AUDIT
revision (v1.1, same burst). Summary per sensor:

| Sensor | DTU conformance status | Correction required pre-demo |
|--------|------------------------|------------------------------|
| CrowdStrike | Auth (OAuth2 token endpoint): CORRECT. Response field names: `detection_id` fix applied (develop@72baf413). | No additional DTU code changes required (TOML-only fixes already applied). |
| Claroty xDome | Auth (Bearer static): CORRECT. Devices route: CORRECT. Audit log route gap: DTU-EXT gap documented. | Audit log DTU route (`/api/v1/audit_log/get`) to be added via `S-DEMO-CLAROTY-AUDIT-DTU-001`. |
| Armis Centrix | Auth (Bearer static): CORRECT. Direct REST endpoints (not AQL): documented gap DTU-EXT-003/004. | AQL endpoint divergence acceptable per D2 exception process; documented in `S-DEMO-ARMIS-AQL-001`. |
| Cyberint Argos | Auth: WRONG — `cyberint_session` cookie must become `access_token` cookie. | BLOCKING: `S-DTU-CYBERINT-AUTH-FIDELITY-001` must ship pre-demo. |

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
| 1.0 | 2026-05-29 | architect | Initial version — establishes DTU=True-DTU as binding architectural principle per user directive 2026-05-29. Supersedes ADR-028 §D12. Defines D1 (six fidelity requirements), D2 (exhaustive list of permitted divergences), D3 (Cyberint DTU correction), D4 (§D12 supersession), D5 (validation discipline), D6 (cross-sensor applicability). Anchor story: S-DTU-CYBERINT-AUTH-FIDELITY-001 (reclassified from P2-post-demo to P0-pre-demo-BLOCKING). |
