---
document_type: story
story_id: S-WAVE-A-ARMIS-REMEDIATION-001
title: "Armis Token-Exchange Spec Migration and DTU Reclone — auth_type token_exchange, [auth_acquisition] block, Armis DTU token endpoint"
version: "1.1"
status: draft
producer: story-writer
phase: 3
wave: wave-a
epic_id: E-WAVE-A-SENSOR-REMEDIATION
priority: P1
points: 8
tdd_mode: strict
target_module: prism-dtu-armis
subsystems: ["SS-06 (SensorSpec)", "SS-12 (DTU-Armis)", "SS-03 (AuthProvider)"]
depends_on:
  - S-ADR054-WAVE-A-001    # AuthType::TokenExchange + DeclarativeHttpAuthProvider must exist
                           # before armis.sensor.toml can declare auth_type = "token_exchange"
                           # and before the Armis DTU can implement the token acquisition route
blocks: []
behavioral_contracts:
  - BC-2.01.006
  - BC-2.06.003
verification_properties:
  - VP-153
estimated_days: 3
# BC status: BC-2.01.006 covers Armis behavior; the dispatch table row for TokenExchange on the
# Armis surface must be verified against the BC-2.01.017 amendment landing in S-ADR054-WAVE-A-001.
# BC-2.06.003 v1.3 covers credential refs — Armis credential_refs will change from bearer_static
# pattern to token_exchange pattern. BC amendments (ADR-053 D5) are PO tasks; they block
# status: ready transition, not implementation dispatch.
assumption_validations: []
risk_mitigations: []
---

# S-WAVE-A-ARMIS-REMEDIATION-001: Armis Token-Exchange Spec Migration and DTU Reclone

## Authority

**ADR-053 v0.35 §D-Armis section** and **ADR-054 v0.52** are the co-authorities.
ADR-053 establishes that Armis migrates to `token_exchange` auth type. ADR-054 provides
the `DeclarativeHttpAuthProvider` and `AuthType::TokenExchange` that make it possible.

This story CANNOT be dispatched before `S-ADR054-WAVE-A-001` is complete (`depends_on`
above).

---

## Narrative

As a Prism maintainer, I want the Armis sensor spec migrated from `auth_type =
"bearer_static"` to `auth_type = "token_exchange"` with a declarative `[auth_acquisition]`
block — and the Armis DTU clone updated to serve the token acquisition endpoint — so that
the Armis adapter authenticates via the native `DeclarativeHttpAuthProvider` (built in
S-ADR054-WAVE-A-001) rather than a static bearer token, matching how the real Armis Centrix
API authenticates API clients.

---

## Current State vs Target State

### Current state (pre-S-WAVE-A-ARMIS-REMEDIATION-001)

```toml
# armis.sensor.toml (current)
auth_type = "bearer_static"
# No [auth_acquisition] block
```

The Armis DTU (`prism-dtu-armis`) authenticates via a static bearer token
(`Bearer <token>` header) with no token acquisition step.

### Target state (this story)

```toml
# armis.sensor.toml (target) — all values Confirmed-tier per ADR-054 §D3 Armis wiring
auth_type = "token_exchange"       # native declarative provider (ADR-054 D1)
header_scheme = "raw"              # Authorization: {token} — no Bearer prefix (ADR-053 D2)
# base_url = "${env.ARMIS_INSTANCE_URL}" (per-org resolved; overlays flow to token URL)

[auth_acquisition]
token_path = "/api/v1/access_token/"
# Token URL derived per-org at step9a_populate_adapter_registry:
#   format!("{}{}", resolved_spec.spec.base_url, "/api/v1/access_token/")
# Per-org base_url overlays (DTU clone, multi-region) flow through automatically.
# token_url is NOT a field — derived from base_url + token_path per ADR-054 §D3 invariant.
credential_body_field = "secret_key"        # form body: secret_key={resolved_value}
token_response_path = "data.access_token"   # dotted path (no $. prefix) — ADR-054 §D3
expiry_field = "data.expiration_utc"        # dotted path: data.expiration_utc
expiry_mode = "absolute_utc_string"
# ttl_buffer_secs = 30 (default)

[[credential_refs]]
name = "secret_key"
description = "Armis long-lived secret key for token exchange"
# Resolved via BC-2.06.003 four-tier per-client chain (PRISM_CLIENTS_{ID}_SENSORS_ARMIS_SECRET_KEY)
```

The Armis DTU serves a `POST /api/v1/access_token/` route (matching `[auth_acquisition].token_path`) that:
- Accepts a form POST body with the `secret_key` credential
- Returns `{"success":true,"data":{"access_token":"...","expiration_utc":"..."}}` (Confirmed per ADR-053 D2
  and vendored Armis research at `.factory/reference/api-specs/armis_endpoint_research_07.20.2026.md`)

**All auth-wiring values are Confirmed-tier per ADR-054 §D3, architect-verified against the vendored
Armis endpoint research at `.factory/reference/api-specs/armis_endpoint_research_07.20.2026.md`
(full agreement with ADR-053 D2). No Armis OpenAPI exists (ADR-053 §D1 no-OpenAPI governance names
Armis explicitly; only Confirmed-tier values appear in this spec). This story may advance to
`status: ready` after PO dependencies PO-001 and PO-002 are resolved (BC amendments per ADR-053 §D5)
and `S-ADR054-WAVE-A-001` ships.**

---

## Acceptance Criteria

### AC-001: armis.sensor.toml declares auth_type = "token_exchange" with valid [auth_acquisition]
(traces to BC-2.01.006 postcondition — Armis sensor spec authenticates via token exchange)

`crates/prism-sensors/specs/armis.sensor.toml` is updated to:
- `auth_type = "token_exchange"`
- `header_scheme = "raw"` (no Bearer prefix; Confirmed-tier per ADR-054 §D3, ADR-053 D2)
- No `auth_plugin` field present (E-SPEC-028(b) rejects auth_plugin for token_exchange regardless of [auth_acquisition] presence)
- A valid `[auth_acquisition]` block with all fields matching the ADR-054 §D3 Armis wiring:
  `token_path = "/api/v1/access_token/"`, `credential_body_field = "secret_key"`,
  `token_response_path = "data.access_token"`, `expiry_field = "data.expiration_utc"`,
  `expiry_mode = "absolute_utc_string"`
- `credential_refs` contains exactly one entry with `name = "secret_key"`

After migration, `parse_and_validate_spec_toml()` accepts the updated Armis spec without
any E-SPEC-028 (Rule 10) or E-SPEC-012 (invalid auth_type) errors.

### AC-002: Armis DTU serves the token acquisition endpoint
(traces to BC-2.01.006 postcondition — DTU behavioral clone supports token exchange flow)

The Armis DTU (`crates/prism-dtu-armis/`) serves a route at `POST /api/v1/access_token/`
(matching `[auth_acquisition].token_path`). The route:
- Accepts a form POST body containing the `secret_key` credential
- Returns `{"success":true,"data":{"access_token":"...","expiration_utc":"..."}}` on success
  (fields at `data.access_token` and `data.expiration_utc` per ADR-054 §D3 `token_response_path`
  and `expiry_field` values)
- Returns HTTP 401 for missing or invalid credentials

A test in `crates/prism-dtu-armis/tests/` verifies the token acquisition flow end-to-end:
POST to `/api/v1/access_token/` with valid `secret_key` → receive `access_token` → use token
raw (no Bearer prefix, per `header_scheme = "raw"`) in a subsequent `/api/v1/search` request
→ receive search results.

SAP-2 compliance: the DTU response fields for the token acquisition endpoint must match
the `[auth_acquisition]` config in `armis.sensor.toml` — `token_response_path`, `expiry_field`,
and `expiry_mode` must all have corresponding DTU type fields.

### AC-003: Armis DTU accepts raw acquired token from DeclarativeHttpAuthProvider (not static token)
(traces to BC-2.01.006 postcondition — the acquired token is used for subsequent API calls)

An integration test verifies that a client which:
1. POSTs to `/api/v1/access_token/` with the Armis `secret_key`
2. Receives an `access_token` in the `data.access_token` response field
3. Uses `Authorization: <access_token>` (raw, no Bearer prefix, per `header_scheme = "raw"`) on `/api/v1/search`

...receives a 200 response with Armis device data. This validates the token exchange flow
end-to-end through the DTU.

The DTU must NOT accept a raw string that was NOT issued by the DTU's own token endpoint
(i.e., old-style `bearer_static` static tokens must be rejected after this story; the auth
model has changed).

**DTU reclone implementation note:** The implementer must read `crates/prism-dtu-armis/src/`
during T-03 before writing T-04 to understand the current auth mechanism. The T-04 update must
achieve the behavioral invariant above: only tokens issued via `POST /api/v1/access_token/` are
accepted on data endpoints. The DTU tracks issued tokens in-process (an allowlist); requests
presenting a token not in that allowlist receive HTTP 401. Existing Armis routes (`/api/v1/search`,
etc.) must continue to work. This is the primary DTU reclone work — its exact scope is bounded
by the T-03 source read, not by speculation.

### AC-004: customer overlay specs are NOT broken by the migration
(traces to BC-2.01.006 invariant — overlay specs inherit base spec auth_type)

`crates/prism-sensors/specs/customers/acme/armis.sensor.toml` and
`crates/prism-sensors/specs/customers/contoso/armis.sensor.toml` are overlay specs.
They must NOT define `auth_type` (the comment in both files says "Schema fields are
FORBIDDEN here"). After the base spec migration, verify:
- Both overlay specs still parse without error
- The overlay merge semantics do not accidentally inject an `auth_type` field from the overlay

### AC-005: VP-153 partial — token_exchange arm for Armis surface passes
(traces to VP-153 — SensorAuth Runtime Cross-Composition Prevention)

The VP-153 MERGE-GATE-VP153-FULL (which is in S-ADR054-WAVE-A-001) should already have
the `TokenExchange` arm. Verify that the Armis-specific test case in VP-153 (if one exists
as a separate probe) passes after this story's changes.

If VP-153 has an Armis-specific arm or test fixture, that fixture must be updated to use
`auth_type = "token_exchange"` after this migration. A test run of the VP-153 harness is
REQUIRED before the PR merges.

### AC-006: BC-2.06.003 Armis credential_refs updated in spec and documented
(traces to BC-2.06.003 postcondition — credential_refs match the new auth pattern)

The Armis credential refs in `armis.sensor.toml` are updated from the bearer_static pattern
(a single bearer token ref) to the token_exchange pattern (a secret key ref used for token
acquisition). The BC-2.06.003 Armis rows are identified for PO amendment (see
Product-Owner Dependencies).

### AC-007: Authorization header carries raw token — no Bearer prefix (wire-shape assertion)
(traces to BC-2.16.009 Rule 9 absence path A — header_scheme = "raw" emits raw token with no Bearer prefix; traces to BC-2.01.017 §P2 dispatch table "raw" row — Bearer-prefixed header causes HTTP 401 on Armis v1 API)

A test in `crates/prism-dtu-armis/tests/` acquires a token via `POST /api/v1/access_token/`,
then issues a data request to the DTU. The test captures the raw HTTP `Authorization` header
value emitted on that data request and asserts:

1. The header value equals the acquired token exactly — no prefix, no whitespace
2. The header value does NOT start with `Bearer ` (case-sensitive)
3. The header value does NOT start with `bearer ` (case-insensitive guard)

Per CLAUDE.md §Wire-shape assertion discipline, asserting `header_scheme = "raw"` in the TOML
config alone is NOT sufficient. This AC requires a wire-level assertion on the emitted header
bytes. Per SID-2, the composed `Authorization` header value must be asserted as composed, not
only via its component fields.

**Why this AC is load-bearing:** BC-2.16.009 Rule 9 absence path A silently applies a `"bearer"`
runtime default when `header_scheme` is absent or misspelled, emitting `Authorization: Bearer <token>`.
The Armis v1 API rejects Bearer-prefixed tokens with HTTP 401 (BC-2.01.017 §P2 dispatch table
`"raw"` row explicitly states "Bearer prefix causes HTTP 401"). This test is the only gate that
would catch a missing or wrong `header_scheme` behind a green spec-validation run.

---

## Product-Owner Dependencies

### PO-001: BC-2.06.003 Armis rows amendment (BLOCKS status: ready)

BC-2.06.003 v1.3 §Armis credential table rows must be updated to reflect the new
credential_ref name and token exchange semantics. Per ADR-053 §D5 amendment manifest
"BC-2.06.003 Armis/Cyberint rows." This is a PO task.

### PO-002: BC-2.01.006 Armis behavior contract amendment (BLOCKS status: ready)

BC-2.01.006 §Armis-specific postconditions must be verified against the token exchange
flow. If the BC currently describes bearer_static authentication for Armis, it requires
amendment. Per ADR-053 §D5 amendment manifest.

---

## Architecture Mapping

| Component | File | Pure/Effectful | Change |
|-----------|------|---------------|--------|
| `armis.sensor.toml` | `crates/prism-sensors/specs/` | Pure (config) | auth_type → token_exchange; add [auth_acquisition] |
| Token acquisition route | `crates/prism-dtu-armis/src/routes/` | Effectful (HTTP handler) | NEW — POST token endpoint |
| DTU auth state | `crates/prism-dtu-armis/src/state.rs` | Effectful (auth state) | UPDATE — token exchange model |
| VP-153 harness | locate via `grep -r MERGE-GATE-VP153-FULL crates/` | Pure (test harness) | Verify Armis arm is present |

---

## Behavioral Contracts

| BC | Version | Relevance |
|----|---------|-----------|
| BC-2.01.006 | current | Armis behavior contract — auth_type change requires BC amendment (PO task) |
| BC-2.06.003 | v1.3 | Credential refs resolution chain — Armis rows change from bearer to token_exchange pattern |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Token acquisition fails — Armis secret_key credential is wrong | DTU returns 401 from token endpoint; DeclarativeHttpAuthProvider returns auth error; query fails with E-SENSOR-401 or equivalent |
| EC-002 | Acquired token expires mid-query | DeclarativeHttpAuthProvider's ttl_buffer_secs triggers refresh before expiry; clean acquisition |
| EC-003 | Old bearer_static token submitted to DTU after migration | DTU rejects — 401 (token not issued by this DTU's token endpoint) |
| EC-004 | Customer overlay armis.sensor.toml has no auth_type | AC-004: overlay does not define auth_type; base spec's token_exchange flows correctly |
| EC-005 | [auth_acquisition].token_path = "" in armis spec after migration | BC-2.16.009 Rule 10 D10(d) fires: E-SPEC-028. This is a spec authoring error, not a runtime error. |

---

## Tasks

### T-01: Read armis.sensor.toml fully before modifying
**File:** `crates/prism-sensors/specs/armis.sensor.toml`

Understand the current credential_refs, auth_type, and any Armis-specific pagination or
auth configuration. Document all fields that will change.

### T-02: Verify auth-wiring values against vendored Armis research doc
**File:** `.factory/reference/api-specs/armis_endpoint_research_07.20.2026.md`

All auth-wiring values are pre-resolved in this spec against ADR-054 §D3 (Confirmed-tier,
architect-verified). No Armis OpenAPI exists (ADR-053 §D1 no-OpenAPI governance names Armis
explicitly). The vendored research doc at the path above is the Confirmed-tier grounding artifact.

The implementer MUST read the research doc to confirm understanding of the following values
before writing any code:
- Token endpoint path: `POST /api/v1/access_token/` (form body, not JSON)
- Credential body field: `secret_key`
- Response shape: `{"success":true,"data":{"access_token":"...","expiration_utc":"..."}}`
- Response paths: `data.access_token`, `data.expiration_utc` (dotted paths, no `$.` prefix)
- Expiry format: absolute UTC string (`expiry_mode = "absolute_utc_string"`)
- Header scheme: `"raw"` — emits `Authorization: <token>`, no Bearer prefix

Do NOT invent new values. If the vendored doc contradicts ADR-054 §D3 values in this spec,
STOP and report to the orchestrator — that is an architect adjudication, not an implementer call.

### T-03: Read crates/prism-dtu-armis/src/ structure before recloning
**Scope:** `crates/prism-dtu-armis/src/`

Understand the current auth model (bearer_static or other). Identify what changes are
needed for the token exchange flow:
- Is there a current token-acquisition route? If yes, it may only need updating.
- Is there an access_token allowlist (like the Cyberint model)? If yes, the token exchange
  model adds a /token endpoint that issues tokens to that allowlist.
- Mark exact file paths for T-04 changes.

### T-04: Update Armis DTU for token exchange auth
**Files:** determined by T-03 read; expected targets per File Structure Requirements:
`crates/prism-dtu-armis/src/routes/` (new `POST /api/v1/access_token/` endpoint) and
`crates/prism-dtu-armis/src/state.rs` (auth state update from static-token to issued-token model)

Add `POST /api/v1/access_token/` route returning `{"success":true,"data":{"access_token":"...","expiration_utc":"..."}}`.
Update auth state to track issued tokens in-process (allowlist model per AC-003). Replace static-token
model with issued-token model. Preserve all existing Armis routes (AQL search, alerts, etc.).

### T-05: Update armis.sensor.toml with ADR-054 §D3 auth-wiring values
**File:** `crates/prism-sensors/specs/armis.sensor.toml`

Apply the Confirmed-tier auth wiring from the "Target state" TOML block in this spec (all values
pre-resolved per ADR-054 §D3). Add `header_scheme = "raw"` alongside `auth_type = "token_exchange"`.
Verify:
- `parse_and_validate_spec_toml()` accepts the updated spec
- No E-SPEC-028 errors for any Rule 10 sub-condition
- No E-SPEC-012 errors (token_exchange is now in VALID_AUTH_TYPES after ADR-054 lands)

### T-06: Write AC-002/AC-003/AC-007 integration tests
**File:** `crates/prism-dtu-armis/tests/`

Tests for the full token exchange flow: `POST /api/v1/access_token/` → acquire `access_token`
from `data.access_token` → use raw (no Bearer prefix) in `/api/v1/search`. Wire-shape assertions
per CLAUDE.md §Wire-shape assertion discipline.

**AC-007 wire-shape test (REQUIRED):** One test must capture the raw `Authorization` header
value emitted on the search request and assert: (1) exact value equals the acquired token with
no prefix, (2) does NOT start with `Bearer `, (3) does NOT start with `bearer `. If the DTU
test infrastructure does not currently expose outbound request headers, add a test hook (e.g.,
`last_auth_header: Arc<Mutex<Option<String>>>` on DTU state, populated in test builds only,
gated `#[cfg(test)]`). Asserting `header_scheme = "raw"` in TOML config alone does not satisfy
AC-007 — the assertion must be on emitted bytes per SID-2 composed-output discipline.

### T-07: Run VP-153 harness and verify Armis arm
**Scope:** VP-153 harness files (locate via grep MERGE-GATE-VP153-FULL)

Verify the TokenExchange arm added in S-ADR054-WAVE-A-001 covers the Armis token_exchange
case. If an Armis-specific fixture needs updating, update it in this story.

---

## Token Budget Estimate

| Context source | Estimated tokens |
|----------------|-----------------|
| This story spec | ~2,500 |
| ADR-053 §D-Armis section | ~1,500 |
| ADR-054 §D4 DeclarativeHttpAuthProvider | ~1,500 |
| `crates/prism-sensors/specs/armis.sensor.toml` | ~2,500 |
| `crates/prism-dtu-armis/src/` (structure survey) | ~4,000 |
| Vendored Armis research (`.factory/reference/api-specs/armis_endpoint_research_07.20.2026.md`) | ~2,000 |
| VP-153 harness files | ~1,500 |
| BC-2.01.006 Armis section | ~1,000 |
| Running test output (nextest) | ~1,500 |
| **Total estimate** | **~18,000** |

18,000 tokens is within the 20–30% threshold. No split required unless the DTU reclone
proves more complex than expected.

---

## Previous Story Intelligence

**From S-WAVE-A-CYBERINT-SPEC-001 (sibling story, same wave):**
- Cookie auth DTU reclone pattern (Cyberint): the DTU issues a token (cookie or bearer)
  to an allowlist at configure time; client presents the token on each request. Armis may
  use a similar issued-token model with the token endpoint generating time-bounded tokens.
- SAP-2: every column name in armis.sensor.toml MUST have a corresponding field in
  `crates/prism-dtu-armis/src/types.rs`. After credential_refs change, verify there are
  no SAP-2 column parity violations.

**From S-ADR054-WAVE-A-001 (dependency):**
- `DeclarativeHttpAuthProvider::get_token()` handles caching and refresh automatically.
  The Armis DTU only needs to implement a realistic token endpoint; the provider handles
  all caching logic.
- The `[auth_acquisition]` TOML block fields (`ttl_buffer_secs`, `expiry_field`,
  `expiry_mode`) directly drive the provider's cache behavior. Choose `expiry_mode`
  based on what the real Armis API returns.

**General lessons:**
- DTU reclones take longer than TOML migrations. Budget most of the 8 points for T-03/T-04
  (DTU work) and plan accordingly.
- Read the existing DTU tests BEFORE writing new ones — the Armis DTU has existing tests
  that assert the current bearer_static auth behavior. Those tests will need updating.

---

## Architecture Compliance Rules

1. **ADR-028 §D1 — DTU-grounded spec authoring.** After T-04 adds the token endpoint
   to the Armis DTU, T-05 MUST update `[auth_acquisition].token_path` to match the
   DTU route registration exactly.

2. **ADR-050 — rustls-tls mandatory.** If the Armis DTU adds any new `reqwest` usage
   (for test clients), it must use `default-features = false, features = ["rustls-tls"]`.

3. **CLAUDE.md §Wire-shape assertion discipline.** AC-002/AC-003 integration tests must
   assert on serialized JSON output from the DTU HTTP responses — not only on Rust structs.

4. **VP-153 MERGE-GATE-VP153-FULL was set in S-ADR054-WAVE-A-001.** If VP-153 has an
   Armis-specific probe, it must pass after this story. If not, the general TokenExchange
   arms (added in ADR-054 story) cover it.

5. **ADR-022 §C — Wiring, not redesign.** If the Armis DTU auth state requires updating
   for token exchange, update the existing auth-state wiring rather than replacing the
   DTU's existing test infrastructure.

---

## Library & Framework Requirements

| Library | Version | Source of truth |
|---------|---------|----------------|
| `axum` | workspace pinned | `architecture/dependency-graph.md §External Dependencies` |
| `serde_json` | workspace pinned | same |
| `reqwest` (if new test clients added) | `default-features = false, features = ["rustls-tls"]` | ADR-050 |

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-sensors/specs/armis.sensor.toml` | MODIFY | T-05: auth_type → token_exchange; add [auth_acquisition]; update credential_refs |
| `crates/prism-dtu-armis/src/routes/` | MODIFY/CREATE | T-04: add token acquisition route |
| `crates/prism-dtu-armis/src/state.rs` | MODIFY | T-04: convert auth state from static-token to issued-token model; exact scope determined by T-03 source read |
| `crates/prism-dtu-armis/tests/` | MODIFY/ADD | T-06: update existing auth tests + new token exchange integration tests |
| VP-153 harness files | VERIFY (no change expected) | T-07: confirm TokenExchange arm covers Armis |
| `crates/prism-sensors/specs/customers/acme/armis.sensor.toml` | VERIFY (no change) | AC-004: overlay spec must remain valid |
| `crates/prism-sensors/specs/customers/contoso/armis.sensor.toml` | VERIFY (no change) | AC-004: overlay spec must remain valid |

---

## Verification Properties

| VP | Description | Applicability |
|----|-------------|---------------|
| VP-153 | SensorAuth Runtime Cross-Composition Prevention | T-07: verify Armis token_exchange case passes VP-153 after S-ADR054-WAVE-A-001 adds TokenExchange arm |

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.1 | 2026-07-25 | story-writer | FB49: re-derive against ADR-054 §D3 ratified Armis wiring; add header_scheme = "raw" (F-WASE-P64-CRIT-004); fix token_path, token_response_path, expiry_field; add AC-007 wire-shape assertion (F-WASE-P64-HIGH-006); resolve all 18 unresolved placeholders; re-anchor status: ready gate from nonexistent Armis OpenAPI to ADR-053 §D1 no-OpenAPI governance |
| 1.0 | 2026-07-25 | story-writer | Initial stub; all auth values unresolved (OpenAPI grounding assumed at time of authoring); depends_on S-ADR054-WAVE-A-001; PO dependency encoding |
