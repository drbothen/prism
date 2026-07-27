---
document_type: story
story_id: S-ADR054-WAVE-A-001
title: "Declarative HTTP Auth Acquisition — DeclarativeHttpAuthProvider, TokenExchange, Rule 10, CrowdStrike TOML Migration, crowdstrike-oauth2.prx Retirement"
version: "1.5"
status: draft
producer: story-writer
phase: 3
wave: wave-a
epic_id: E-WAVE-A-SENSOR-REMEDIATION
priority: P0
points: 13
tdd_mode: strict
target_module: prism-spec-engine
subsystems: ["SS-06 (SensorSpec)", "SS-03 (AuthProvider)", "SS-02 (AdapterRegistry)"]
depends_on:
  - S-WAVE-A-ENGINE-001    # ADR-054 §D7: implementation stories MUST merge AFTER ENGINE-001;
                           # engine-001 adds header_scheme grammar that this story's CrowdStrike
                           # migration touches (crowdstrike uses bearer_static — absence path A;
                           # no header_scheme required — but the grammar must exist before migration)
blocks:
  - S-WAVE-A-ARMIS-REMEDIATION-001    # Armis TOML migration needs AuthType::TokenExchange +
                                      # DeclarativeHttpAuthProvider in place before it can be
                                      # implemented; S-WAVE-A-ARMIS-REMEDIATION-001 is gated on
                                      # this story's completion
behavioral_contracts:
  - BC-2.16.009
  - BC-2.01.017
  - BC-2.06.003
verification_properties:
  - VP-153
  - VP-159
estimated_days: 5
# BC status: PO-001 retired — the section originally cited in PO-001 does not exist in
# BC-2.01.017 v1.10 (POL-21 phantom section anchor); the 5→6-value canonical auth_type set
# amendments (§Preconditions, §P3, §Related BCs) are EXECUTED per ADR-054 D11 Wave-A spec
# evolution burst 3; the TokenExchange dispatch arm is covered by AC-008/D7 in this story.
# PO-002 retired — BC-2.16.009 Rule 10 is fully specified in v1.12+ with all 8 sub-conditions
# (EXECUTED per ADR-054 D11). BC-2.06.003 covers credential refs for the token_exchange flow.
# No BC amendments are required before status: ready transition.
assumption_validations: []
risk_mitigations: []
---

# S-ADR054-WAVE-A-001: Declarative HTTP Auth Acquisition

## Authority

**ADR-054 v0.55** (accepted 2026-07-22) is the authoritative design document.
Read it in full before implementing:
`.factory/specs/architecture/decisions/ADR-054-native-declarative-http-auth-acquisition.md`

This story is the single implementation story for ADR-054. All five ADR-054 decision
points (D1–D5, D7, D10) are in scope. Stories that split these across multiple PRs are
not permitted — the CrowdStrike migration, plugin retirement, and adapter registry
update must land atomically.

---

## Narrative

As a Prism maintainer, I want the `oauth2_client_credentials` auth flow rewritten as a
native Rust `DeclarativeHttpAuthProvider` — replacing the `crowdstrike-oauth2.prx` plugin
binary — and a new `token_exchange` auth type added, so that (a) the CrowdStrike adapter
no longer depends on an external plugin binary at runtime, (b) the Armis token-exchange
flow can be declared in TOML without a new plugin, (c) BC-2.16.009 Rule 10 validates
`[auth_acquisition]` blocks at spec load time, and (d) VP-153 and VP-159 pass the full
verification suite.

---

## Scope Summary (from ADR-054)

| Decision | Change |
|----------|--------|
| D1 | `AuthType::TokenExchange` added to closed enum; `"token_exchange"` added to `VALID_AUTH_TYPES`; E-SPEC-012 updated |
| D2 | CrowdStrike TOML migrated from `auth_plugin = "crowdstrike-oauth2"` to `[auth_acquisition]` block |
| D3 | `AuthAcquisitionConfig` struct with `token_path` (required), `ttl_buffer_secs` (default 30), plus TokenExchange-only fields |
| D4 | `DeclarativeHttpAuthProvider` struct in `crates/prism-spec-engine/src/auth/`; implements `AuthProvider`; lazy acquisition + ArcSwap cache |
| D5 | `crowdstrike-oauth2.prx` crate deleted; removed from workspace `Cargo.toml` `members` |
| D7 | `step9a_populate_adapter_registry` in `spec_driven_adapter.rs` gains `TokenExchange` arm; `Oauth2ClientCredentials` arm rewritten |
| D10 | Rule 10 added inside `SpecLoader::parse()` (not in `validate_sensor_spec()`); E-SPEC-028 (8 sub-conditions for `[auth_acquisition]` validation) |

---

## Acceptance Criteria

### AC-001: AuthType::TokenExchange variant in closed enum
(traces to BC-2.16.009 Rule 10 precondition — auth_type = "token_exchange" is parseable)

`AuthType` enum in `prism-spec-engine` has a `TokenExchange` variant. `VALID_AUTH_TYPES`
array includes `"token_exchange"`. E-SPEC-012 error message (for invalid auth_type) is
updated to list `"token_exchange"` as a valid value.

A unit test verifies that a TOML with `auth_type = "token_exchange"` is successfully
parsed by `SpecLoader::parse()` without E-SPEC-012 error.

### AC-002: AuthAcquisitionConfig struct is #[non_exhaustive] and correct
(traces to BC-2.16.009 Rule 10 postcondition — [auth_acquisition] is deserialized and validated)

`AuthAcquisitionConfig` is defined in `prism-spec-engine` with:
- `token_path: String` — required field
- `ttl_buffer_secs: u64` — default 30 (via `#[serde(default = "default_ttl_buffer")]`)
- Token-exchange-only fields (per ADR-054 D3): `credential_body_field: Option<String>`,
  `token_response_path: Option<String>`, `expiry_field: Option<String>`,
  `expiry_mode: Option<ExpiryMode>`
- `ExpiryMode` enum: `AbsoluteUtcString` / `RelativeSeconds` (both `#[non_exhaustive]`)

`AuthAcquisitionConfig`, `CachedAuthToken`, and `ExpiryMode` are all `#[non_exhaustive]`.
The non-exhaustive gate bumps from 92 to 95. All three of `scripts/check-non-exhaustive.sh`
(EXPECTED=95), `CLAUDE.md` (`EXPECTED=92` sentence updated to 95), and
`scripts/check-non-exhaustive-per-symbol.py` (EXPECTED_COUNT + EXPECTED_SYMBOLS) are updated
in the same commit.

### AC-003: Rule 10 / E-SPEC-028 validates [auth_acquisition] blocks
(traces to BC-2.16.009 Rule 10 postcondition — 8 sub-conditions enforced)

Rule 10 executes inside `SpecLoader::parse()` as the final gate before `Ok(spec)` is
returned — the same execution site as Rules 8 and 9 (probe_table and header_scheme
validation). Rule 10 is NOT placed inside `validate_sensor_spec()`. That function covers
Rules 1–5 only, has zero production callers, and placement there makes E-SPEC-028
unreachable from the `add_sensor_spec` MCP tool and from hot-reload (ADR-054 §D10
rationale; BC-2.16.009 §Integration function).

`SpecLoader::parse()` implements Rule 10 with `E-SPEC-028`. The 8 sub-conditions from
ADR-054 §D10 are enforced (BC-2.16.009 Rule 10 is the normative text; ADR-054 §D10 is
authoritative for trigger logic; error-taxonomy.md `E-SPEC-028` is authoritative for
message wording):

D10(a): `auth_type ∈ {oauth2_client_credentials, token_exchange}` AND (`[auth_acquisition]`
absent OR `token_path` absent) → E-SPEC-028. Both declarative auth types require
`[auth_acquisition]` with `token_path` to derive the per-org token URL at boot step 9A.

D10(b): `auth_type ∈ {oauth2_client_credentials, token_exchange}` AND `auth_plugin` present
— regardless of whether `[auth_acquisition]` is declared → E-SPEC-028. These auth types use
the native `DeclarativeHttpAuthProvider`; `auth_plugin` is unconditionally rejected
(Definition 1, ADR-054 §D10(b)).

D10(c): `auth_type = "token_exchange"` AND `[auth_acquisition]` present AND `expiry_mode` is
set but not in `{absolute_utc_string, relative_seconds}` → E-SPEC-028. Sub-condition (c) does
NOT fire when `expiry_mode` appears on a non-`token_exchange` block; sub-condition (h) handles
that case (clean partition: (c) = value-validity for valid-position use; (h) = position-validity
for wrong-position use).

D10(d): `auth_type = "token_exchange"` AND `[auth_acquisition]` present AND any of
`{credential_body_field, token_response_path, expiry_field, expiry_mode}` is absent
(`field.is_none()`) → one E-SPEC-028 per absent field. Absence predicate: `field.is_none()`
means the TOML key is omitted entirely. An explicitly empty string (e.g.,
`credential_body_field = ""`) resolves to `Some("")` — passes D10(d) but may fail D10(e)
if `""` does not match any `[[credential_refs]]` name. `ttl_buffer_secs` is OPTIONAL
(default 30); omitting it is VALID and does not trigger D10(d).

D10(e): `credential_body_field` is present (`Some(name)`) but no `[[credential_refs]]`
block has `name = "{name}"` → E-SPEC-028.

D10(f): `auth_type = "oauth2_client_credentials"` AND one or both of `client_id`,
`client_secret` absent from `[[credential_refs]]` → E-SPEC-028 citing the missing ref names.

D10(g): `[auth_acquisition]` present AND `auth_type ∈ {bearer_static, cookie_roundtrip,
api_key, custom_via_plugin}` → E-SPEC-028. Only declarative auth types support
`[auth_acquisition]`.

D10(h): `[auth_acquisition]` present AND any of `{credential_body_field, token_response_path,
expiry_field, expiry_mode}` is present (`field.is_some()`) AND `auth_type != "token_exchange"`
→ single aggregated E-SPEC-028 citing all offending field names as `{field_list}`. Prevents
token_exchange-only fields from being silently ignored on an `oauth2_client_credentials` block
(SOUL.md #4 violation class).

A test for each sub-condition verifies rejection via `parse_and_validate_spec_toml()`
(SAP-3: Rule 10 is inside `SpecLoader::parse()`, which `parse_and_validate_spec_toml()` calls
as its first act; tests reach Rule 10 through this integration surface — not via
`validate_sensor_spec()`, which is NOT the Rule 10 execution site and has zero production
callers).

### AC-004: DeclarativeHttpAuthProvider implements AuthProvider with lazy acquisition
(traces to BC-2.01.017 postcondition — auth provider acquires token on first use, caches it)

`DeclarativeHttpAuthProvider` in `crates/prism-spec-engine/src/auth/`:
- `get_token()`: returns cached token if present and not within `ttl_buffer_secs` of expiry;
  otherwise calls `acquire_token()` and updates the ArcSwap cache
- `acquire_token()`: force-refresh; makes HTTP POST to `token_url` with credential body;
  parses response at `token_response_path`; returns `CachedAuthToken`
- Clock injection: `now_fn: Arc<dyn Fn() -> u64 + Send + Sync>` for testability
- `new_for_test(now_fn)`: test-only constructor that accepts clock injection
- Production constructor wires `reqwest::Client` with `default-features = false, features = ["rustls-tls"]` and `.timeout(Duration::from_secs(30))` per ADR-050 and CLAUDE.md §HTTP client timeout

VP-159 wiremock integration tests verify:
- Lazy acquisition (VP-159-a): `get_token()` calls `acquire_token()` on first invocation
- Cache hit (VP-159-b): second `get_token()` does NOT call the token endpoint again
- Cache refresh on expiry (VP-159-c): after clock advances past expiry - ttl_buffer_secs, `get_token()` re-calls `acquire_token()`

### AC-005: VP-153 MERGE-GATE-VP153-FULL passes
(traces to VP-153 — SensorAuth Runtime Cross-Composition Prevention with TokenExchange arms)

The VP-153 harness files (locate via `grep -r "MERGE-GATE-VP153-FULL" crates/`) have
`TokenExchange` arms added in the same commit as the `TokenExchange` variant is added to
the `AuthType` enum. The MERGE-GATE-VP153-FULL assertion must be verified before the PR
merges.

A test run of the VP-153 harness confirms it passes with no runtime panics or test failures.

### AC-006: CrowdStrike TOML migrated from auth_plugin to [auth_acquisition]
(traces to BC-2.01.017 postcondition — CrowdStrike authenticates via DeclarativeHttpAuthProvider)

`crates/prism-sensors/specs/crowdstrike.sensor.toml` is updated:
- `auth_plugin = "crowdstrike-oauth2"` line is DELETED
- `[auth_acquisition]` block added per ADR-054 D2:
  ```toml
  [auth_acquisition]
  token_path = "/oauth2/token"
  ttl_buffer_secs = 30
  ```
- `auth_type = "oauth2_client_credentials"` is RETAINED (unchanged)

After migration, `parse_and_validate_spec_toml()` accepts the updated CrowdStrike spec
without E-SPEC-028 errors. (The removed `auth_plugin = "crowdstrike-oauth2"` line would
have triggered E-SPEC-028 sub-condition (b) had it remained — auth_plugin on a declarative
auth_type is unconditionally rejected by D10(b) regardless of `[auth_acquisition]` presence.)

### AC-007: crowdstrike-oauth2.prx crate deleted from workspace
(traces to BC-2.01.017 postcondition — no plugin binary dependency at runtime)

The `crowdstrike-oauth2.prx` crate source directory is deleted. The workspace `Cargo.toml`
`members` array no longer includes the crate. `just check` passes after deletion (no orphaned
references in other crates).

A pre-deletion grep confirms no production code path in other crates imports or references
`crowdstrike-oauth2` — only the deletion commit removes it.

### AC-008: step9a_populate_adapter_registry updated for TokenExchange and Oauth2ClientCredentials
(traces to BC-2.01.017 postcondition — adapter registry correctly wires auth providers)

`spec_driven_adapter.rs::step9a_populate_adapter_registry()` gains:
- A `TokenExchange` arm that constructs `DeclarativeHttpAuthProvider` with the
  `auth_acquisition` config from the sensor spec
- The `Oauth2ClientCredentials` arm is rewritten to also use `DeclarativeHttpAuthProvider`
  (removing the `crowdstrike-oauth2.prx` plugin invocation)

A unit test verifies that a sensor spec with `auth_type = "oauth2_client_credentials"` and
a valid `[auth_acquisition]` block produces a `DeclarativeHttpAuthProvider` in the adapter
registry (not a plugin-backed provider).

### AC-009: reqwest client in DeclarativeHttpAuthProvider uses rustls-tls
(traces to ADR-050 — rustls-tls mandatory; CLAUDE.md §reqwest TLS backend)

The `reqwest::Client` constructed inside `DeclarativeHttpAuthProvider::new()` uses:
```rust
reqwest::Client::builder()
    .default_headers(...)
    .timeout(Duration::from_secs(30))
    .build()
    .expect("DeclarativeHttpAuthProvider HTTP client build failed")
```

The `Cargo.toml` for `prism-spec-engine` (or the crate that owns `DeclarativeHttpAuthProvider`)
has `reqwest = { version = "...", default-features = false, features = ["rustls-tls"] }`.
The native-tls feature is NOT present in any form.

### AC-010: Rule 9 → Rule 10 fail-fast boundary verified with Rule 10 live
(traces to BC-2.16.009 §Invariants v1.28 — fail-fast boundary at the `SpecLoader::parse()` rule
boundary: Rule 9 returning `Err` prevents Rule 10 from executing)

Contract authority: BC-2.16.009 §Invariants v1.28: "At the `SpecLoader::parse()` rule boundary,
execution is fail-fast: Rule 9 returning `Err` prevents Rule 10 from executing — a spec with both
a Rule 9 violation and a Rule 10 violation reports only the Rule 9 error from `parse()`."

With Rule 10 implemented by this story, a TOML spec carrying BOTH:
1. A Rule 9 violation: `header_scheme = "garbage"` (syntactically invalid — triggers E-SPEC-027
   template (a): value is neither `"bearer"`, `"raw"`, nor `"cookie:<name>"`)
2. A genuine Rule 10 sub-condition (d) violation: `auth_type = "token_exchange"`, `[auth_acquisition]`
   block present with `token_path`, `credential_body_field`, `expiry_field`, and
   `expiry_mode = "absolute_utc_string"` declared but `token_response_path` ABSENT — the field
   is omitted entirely from the TOML (`token_response_path.is_none()` → triggers E-SPEC-028
   sub-condition (d) when Rule 10 executes per BC-2.16.009 Rule 10 and EC-009-040)

yields exactly ONE error from `SpecLoader::parse()` — the Rule 9 `ESpec027` error. Exactly one
error confirms Rule 10 was reached-but-short-circuited by the Rule 9 `Err` return, not absent.

**Why sub-condition (d) is independently a genuine Rule 10 rejection:** D10(d) fires when
`auth_type = "token_exchange"` AND `[auth_acquisition]` is present AND any of
`{credential_body_field, token_response_path, expiry_field, expiry_mode}` is absent
(`field.is_none()`). An `[auth_acquisition]` block for `token_exchange` with all fields present
except `token_response_path` is a real D10(d) violation per BC-2.16.009 Rule 10 and EC-009-040.
The `ttl_buffer_secs` field is explicitly optional (default 30) and its absence is NOT an error —
the omission of `token_response_path` is what triggers D10(d). Its rejection is confirmed by the
control assertion (RGT-010-B).

**Control assertion (RGT-010-B):** The same `[auth_acquisition]` configuration — missing
`token_response_path` — submitted with `header_scheme = "raw"` (valid for `token_exchange` per
BC-2.16.009 EC-009-029; Rule 9 passes) yields exactly ONE error — E-SPEC-028 from sub-condition
(d). The control proves the fixture is a genuine Rule 10(d) rejection, not an inert block that
Rule 10 ignores. Without the control, the boundary assertion (RGT-010-A) would be vacuous: a spec
with Rule 10 absent or not-yet-implemented also produces exactly one E-SPEC-027 error when
submitted with `header_scheme = "garbage"` — the boundary case and the absent-Rule-10 case are
indistinguishable without a separate proof that the `[auth_acquisition]` block is a real violation.

RGT-010-A: `test_rule9_rule10_d_boundary_rule9_err_prevents_rule10_single_e_spec027`
  Drives `SpecLoader::parse()` with raw TOML containing both violations; asserts exactly one
  `ESpec027` error returned. `spec_parser.rs` inline `#[cfg(test)] mod tests`. SAP-3: parser
  surface entry point via raw TOML — NOT via `validate_sensor_spec()`, NOT via
  synthetic-struct invocation.

RGT-010-B: `test_rule10_d_missing_token_response_path_control_no_rule9_violation_e_spec028`
  Drives `SpecLoader::parse()` with the same `[auth_acquisition]` block missing
  `token_response_path` but with `header_scheme = "raw"` (valid for `token_exchange`); asserts
  exactly one `ESpec028` error returned. `spec_parser.rs` inline `#[cfg(test)] mod tests`.
  SAP-3: parser surface.

Cross-reference: ENGINE-001 AC-026/RG-031 is the sending side — it covers the boundary observable
consequence with Rule 10 not yet implemented (RG-031's `[auth_acquisition]` block is inert at
ENGINE-001's scope because Rule 10 does not exist there). This story's AC-010 / RGT-010-A +
RGT-010-B is the receiving side — it verifies the same boundary holds with Rule 10 live, using a
fixture that is a genuine Rule 10(d) rejection confirmed by RGT-010-B. The hand-off is
bidirectional: ENGINE-001 AC-026 forward-references "S-ADR054-WAVE-A-001"; this AC
back-references "ENGINE-001 AC-026/RG-031".

---

## Product-Owner Dependencies

### PO-001: RETIRED — The section originally cited does not exist in BC-2.01.017 (POL-21 phantom anchor)

The section referenced by PO-001 does not exist in BC-2.01.017 v1.10 (POL-21 phantom section
anchor). Verified actual sections: §Description, §Preconditions,
§Postconditions (§P1 Token Acquisition, §P2 Request Header Injection, §P3 Auth Type Dispatch,
§P4 Zero Login-Shaped Requests), §Invariants, §Error Cases, §Edge Cases, §Canonical Test
Vectors, §Verification Properties, §Related BCs, §Architecture Anchors, §Story Anchor,
§VP Anchors, §Traceability, §Notes for Implementers, §Changelog.

The 5→6-value canonical auth_type set amendments to BC-2.01.017 (§Preconditions, §P3 Auth
Type Dispatch, §Related BCs) are already EXECUTED per ADR-054 D11 Wave-A spec evolution
burst 3. The §P2 dispatch table is `header_scheme`-keyed per ADR-053 D2 and already includes
a `"raw"` row covering `token_exchange` (Armis). The `TokenExchange` adapter arm in
`step9a_populate_adapter_registry()` is implemented by AC-008 of this story (ADR-054 D7) and
does not require a separate BC amendment. PO-001 is retired as moot.

### PO-002: RETIRED — BC-2.16.009 Rule 10 is already fully specified

BC-2.16.009 §Validation Rules 10 is fully specified with all 8 sub-conditions as of v1.12
(current version v1.28). The EXECUTED annotation in ADR-054 D11 confirms:
`BC-2.16.009 Rule set: [EXECUTED — Wave-A spec evolution burst 3, 2026-07-22] Add
[auth_acquisition] coherence validation as Rule 10; E-SPEC-028 error suite per D10`.
PO-002 is retired as already-satisfied.

---

## Architecture Mapping

| Component | File | Pure/Effectful | Change |
|-----------|------|---------------|--------|
| `AuthType` enum | `crates/prism-spec-engine/src/types.rs` (TBD — locate via grep) | Pure (data) | Add `TokenExchange` variant (D1) |
| `AuthAcquisitionConfig` | `crates/prism-spec-engine/src/types.rs` (TBD) | Pure (data) | New struct (D3) |
| `CachedAuthToken` | `crates/prism-spec-engine/src/auth/` (TBD) | Pure (data) | New struct (D4) |
| `ExpiryMode` | `crates/prism-spec-engine/src/types.rs` or `auth/` (TBD) | Pure (data) | New enum (D3) |
| `DeclarativeHttpAuthProvider` | `crates/prism-spec-engine/src/auth/` | Effectful (HTTP client) | New struct (D4) |
| `SpecLoader::parse()` | `crates/prism-spec-engine/src/spec_parser.rs` | Pure (validation) | Add Rule 10 / E-SPEC-028 inside `SpecLoader::parse()` as final gate before `Ok(spec)`, after Rules 8 and 9 (D10). Rule 10 is NOT added to `validate_sensor_spec()`. |
| `step9a_populate_adapter_registry()` | `crates/prism-bin/src/spec_driven_adapter.rs` | Effectful (DI wiring) | Add TokenExchange arm; rewrite Oauth2 arm (D7) |
| `crowdstrike.sensor.toml` | `crates/prism-sensors/specs/` | Pure (config) | Migrate auth_plugin → [auth_acquisition] (D2) |
| `crowdstrike-oauth2.prx` | workspace root (TBD path) | Effectful (plugin binary) | DELETE (D5) |

---

## Behavioral Contracts

| BC | Version | Relevance |
|----|---------|-----------|
| BC-2.16.009 | v1.28 | Rule 10 / E-SPEC-028 — [auth_acquisition] validation |
| BC-2.01.017 | v1.10 | §P2 `"raw"` dispatch row: `Authorization: {token}` for `token_exchange` (Bearer prefix → HTTP 401 per §P2 note); §P3 6-value canonical auth_type set includes `token_exchange` per ADR-054 D11; AC-008 wires `step9a_populate_adapter_registry` `TokenExchange` + `Oauth2ClientCredentials` arms to `DeclarativeHttpAuthProvider` |
| BC-2.06.003 | v1.3 | Credential refs for the token_exchange flow |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | [auth_acquisition] block present with auth_type = "bearer_static" | D10(g): E-SPEC-028 — bearer_static does not use declarative acquisition; only oauth2_client_credentials and token_exchange support [auth_acquisition] |
| EC-002 | auth_type = "oauth2_client_credentials" with auth_plugin present (with or without [auth_acquisition]) | D10(b): E-SPEC-028 — auth_plugin is unconditionally rejected for declarative auth types regardless of [auth_acquisition] presence (Definition 1, ADR-054 §D10(b)) |
| EC-003 | auth_type = "token_exchange" without [auth_acquisition] block | D10(a): E-SPEC-028 — [auth_acquisition] with token_path is required for both declarative auth types |
| EC-004 | auth_type = "token_exchange", [auth_acquisition] present but token_path absent (TOML key omitted) | D10(a): E-SPEC-028 — token_path is required in [auth_acquisition]; its absence triggers sub-condition (a) even when the block itself is present |
| EC-005 | auth_type = "token_exchange", [auth_acquisition] present with token_path, but token_response_path absent (TOML key omitted) | D10(d): E-SPEC-028 — token_response_path.is_none() triggers sub-condition (d); one error per absent required field; ttl_buffer_secs is optional and its absence is NOT an error |
| EC-006 | VP-159: token acquisition HTTP endpoint returns 401 | DeclarativeHttpAuthProvider returns E-SENSOR-401 (or equivalent) rather than panicking or returning stale token |
| EC-007 | Clock injection: get_token() called at expiry boundary (exactly at expiry - ttl_buffer_secs) | Token is refreshed (boundary is inclusive for refresh) |
| EC-008 | oauth2_client_credentials TOML without [auth_acquisition] block after plugin retirement | D10(a) fires E-SPEC-028 — oauth2_client_credentials requires [auth_acquisition] with token_path (same sub-condition as token_exchange; per BC-2.16.009 Rule 10(a) and EC-009-039). This is why AC-006 adds the [auth_acquisition] block to crowdstrike.sensor.toml as part of the D5 migration: without it, D10(a) would reject the CrowdStrike spec at spec-load time. The migration satisfies D10(a) for the CrowdStrike spec. |

---

## Tasks

### Red Gate tests (to be written by test-writer BEFORE implementation)

- [ ] **RG-001**: `test_authtype_token_exchange_variant_parses_without_e_spec_012` — AC-001
  _(SAP-3: submits TOML with `auth_type = "token_exchange"` via `parse_and_validate_spec_toml()`; asserts no E-SPEC-012 error (unknown auth_type) is returned; confirms `token_exchange` is in the canonical 6-value auth_type set after AC-001 landing)_

- [ ] **RG-002**: `test_auth_acquisition_config_non_exhaustive_gate_includes_auth_acquisition_config` — AC-002
  _(Compile-fail test or symbol-count test confirming `AuthAcquisitionConfig` carries `#[non_exhaustive]` and the gate script `check-non-exhaustive.sh` EXPECTED count is updated; links to the non-exhaustive perimeter gate pattern from `tests/external/perimeter-violation/`)_

- [ ] **RG-003**: `test_auth_acquisition_config_ttl_buffer_default_is_30` — AC-002
  _(Unit test in `spec_parser.rs` or `auth_provider.rs`: constructs `AuthAcquisitionConfig` with no explicit `ttl_buffer_seconds`; asserts `ttl_buffer_seconds == 30`; covers the default value clause in AC-002)_

- [ ] **RG-004**: `test_rule10_d10a_missing_auth_acquisition_for_oauth2_returns_e_spec_028` — AC-003
  _(SAP-3: drives `SpecLoader::parse()` via `parse_and_validate_spec_toml()` with `auth_type = "oauth2_client_credentials"` and no `[auth_acquisition]` block; asserts E-SPEC-028 sub-condition (a); confirm reachable from public surface)_

- [ ] **RG-005**: `test_rule10_d10b_auth_plugin_on_declarative_auth_type_returns_e_spec_028` — AC-003
  _(Drives `SpecLoader::parse()` with `auth_type = "token_exchange"` AND `auth_plugin = "crowdstrike-oauth2"` present; asserts E-SPEC-028 sub-condition (b) — auth_plugin field forbidden for declarative types)_

- [ ] **RG-006**: `test_rule10_d10c_invalid_expiry_mode_for_token_exchange_returns_e_spec_028` — AC-003
  _(Drives `SpecLoader::parse()` with valid `[auth_acquisition]` except `expiry_mode = "bogus"`; asserts E-SPEC-028 sub-condition (c))_

- [ ] **RG-007**: `test_rule10_d10d_absent_token_response_path_for_token_exchange_returns_e_spec_028` — AC-003
  _(Drives `SpecLoader::parse()` with `token_exchange` + valid `[auth_acquisition]` but `token_response_path` absent; asserts E-SPEC-028 sub-condition (d))_

- [ ] **RG-008**: `test_rule10_d10e_credential_body_field_not_in_credential_refs_returns_e_spec_028` — AC-003
  _(Drives `SpecLoader::parse()` with `credential_body_field = "client_secret"` but no matching `credential_refs` entry; asserts E-SPEC-028 sub-condition (e))_

- [ ] **RG-009**: `test_rule10_d10f_oauth2_missing_client_id_in_credential_refs_returns_e_spec_028` — AC-003
  _(Drives `SpecLoader::parse()` with `auth_type = "oauth2_client_credentials"` + `[auth_acquisition]` but no `client_id` field in `credential_refs`; asserts E-SPEC-028 sub-condition (f))_

- [ ] **RG-010**: `test_rule10_d10g_auth_acquisition_on_bearer_static_returns_e_spec_028` — AC-003
  _(Drives `SpecLoader::parse()` with `auth_type = "bearer_static"` + `[auth_acquisition]` block present (forbidden for static types); asserts E-SPEC-028 sub-condition (g))_

- [ ] **RG-011**: `test_rule10_d10h_token_exchange_only_fields_on_oauth2_returns_e_spec_028` — AC-003
  _(Drives `SpecLoader::parse()` with `auth_type = "oauth2_client_credentials"` + `token_response_path` (a token_exchange-only field); asserts E-SPEC-028 sub-condition (h) — token_exchange-only fields forbidden on oauth2)_

- [ ] **RG-012**: `test_declarative_http_auth_provider_lazy_acquire_on_first_get_token` — AC-004
  _(VP-159 arm (a): constructs `DeclarativeHttpAuthProvider` with wiremock token endpoint; calls `get_token()` once; asserts exactly one POST to the token endpoint was made; validates lazy-acquire behavior)_

- [ ] **RG-013**: `test_declarative_http_auth_provider_cache_hit_no_second_acquire` — AC-004
  _(VP-159 arm (b): calls `get_token()` twice within TTL window; asserts only one POST to token endpoint total; validates cache-hit path — no re-acquisition within TTL)_

- [ ] **RG-014**: `test_declarative_http_auth_provider_refresh_on_expiry` — AC-004
  _(VP-159 arm (c): uses `now_fn` clock injection to advance past expiry; calls `get_token()` again; asserts second POST to token endpoint was made; validates refresh-on-expiry path)_

- [ ] **RG-015**: `test_vp_153_merge_gate_full_with_token_exchange_arm` — AC-005
  _(VP-153 MERGE-GATE-VP153-FULL: runs the full VP-153 harness after `TokenExchange` arm is added to both harness files; asserts harness passes with no cross-composition violations; must be in same commit as enum variant addition)_

- [ ] **RG-016**: `test_crowdstrike_toml_no_auth_plugin_field` — AC-006
  _(Parses `crates/prism-sensors/specs/crowdstrike.sensor.toml`; asserts no `auth_plugin` field is present at any level; verifies the plugin field removal per ADR-054 D2)_

- [ ] **RG-017**: `test_crowdstrike_toml_auth_acquisition_block_present_with_token_path` — AC-006
  _(Parses `crowdstrike.sensor.toml`; asserts `[auth_acquisition]` block is present with a non-empty `token_response_path`; verifies TOML migration)_

- [ ] **RG-018**: `test_crowdstrike_oauth2_prx_crate_not_in_workspace_members` — AC-007
  _(Reads `Cargo.toml`; asserts no member path matching `crowdstrike-oauth2` is present; also asserts `just check` passes after deletion — no dangling dependency in workspace)_

- [ ] **RG-019**: `test_step9a_token_exchange_spec_produces_declarative_http_auth_provider` — AC-008
  _(Unit test in `step9a_populate_adapter_registry` or its test module: constructs a `SensorSpec` with `auth_type = TokenExchange` and full `AuthAcquisitionConfig`; calls `step9a_populate_adapter_registry()`; asserts the returned provider is a `DeclarativeHttpAuthProvider` (via downcasting or type assertion))_

- [ ] **RG-020**: `test_declarative_http_auth_provider_cargo_toml_uses_rustls_tls_not_native_tls` — AC-009
  _(Reads `Cargo.toml` for the crate containing `DeclarativeHttpAuthProvider`; asserts `reqwest` dependency has `default-features = false` and `features = ["rustls-tls"]`; asserts no `native-tls` or `default-tls` feature present; ADR-050 compliance)_

- [ ] **RG-021**: `test_rule9_rule10_d_boundary_rule9_err_prevents_rule10_single_e_spec027` — AC-010
  _(Boundary: TOML with `header_scheme = "garbage"` [Rule 9 violation → E-SPEC-027(a)] AND `[auth_acquisition]` for `token_exchange` with valid fields except `token_response_path` absent [Rule 10(d) violation → E-SPEC-028 when Rule 10 is live]; assert exactly one `ESpec027` error; proves Rule 10 reached-but-short-circuited, not absent; SAP-3: drives `SpecLoader::parse()` via raw TOML — NOT via `validate_sensor_spec()`, NOT via synthetic-struct invocation)_

- [ ] **RG-022**: `test_rule10_d_missing_token_response_path_control_no_rule9_violation_e_spec028` — AC-010 (control)
  _(Control: same `[auth_acquisition]` block missing `token_response_path` but with `header_scheme = "raw"` [valid for `token_exchange` per BC-2.16.009 EC-009-029; Rule 9 passes]; assert exactly one `ESpec028` error; proves fixture is a genuine Rule 10(d) rejection; SAP-3: drives `SpecLoader::parse()` via raw TOML)_

- [ ] **RG-023**: `test_rule10_d10a_token_exchange_missing_token_path_returns_e_spec_028` — AC-003/EC-004
  _(EC-004 coverage: `token_exchange` with `[auth_acquisition]` block but entirely missing `token_path` field; asserts E-SPEC-028 sub-condition (d); verifies EC-004 edge case from story §Edge Cases)_

- [ ] **RG-024**: `test_declarative_http_auth_provider_token_endpoint_401_returns_auth_error` — AC-004/EC-006
  _(EC-006 coverage: wiremock returns HTTP 401 on token endpoint; calls `get_token()`; asserts a structured auth error (not a panic or silent empty token) is returned; validates EC-006 expected behavior)_

**Red Gate density check** (BC-5.38.001): **24 failing tests** before implementation begins. RG-001 covers AC-001; RG-002/RG-003 cover AC-002; RG-004..RG-011 cover AC-003 (all 8 sub-conditions D10(a)–D10(h)); RG-012/RG-013/RG-014 cover AC-004 (VP-159 arms a/b/c); RG-015 covers AC-005 (VP-153 gate); RG-016/RG-017 cover AC-006 (CrowdStrike TOML); RG-018 covers AC-007 (crate deletion); RG-019 covers AC-008 (step9a adapter wiring); RG-020 covers AC-009 (rustls-tls); RG-021/RG-022 cover AC-010 (Rule 9/Rule 10 boundary); RG-023/RG-024 cover EC-004/EC-006. RED_RATIO is computed by the orchestrator at Step 3.5 per per-story-delivery.md from actual Red Gate results; BC-5.38.002 and BC-5.38.003 define the exempt test classes (green-by-design and wiring-exempt) that reduce the denominator.

### Implementation tasks

### T-01: Locate AuthType enum and VALID_AUTH_TYPES before modifying
**Scope:** grep `crates/prism-spec-engine/src/` for `AuthType` and `VALID_AUTH_TYPES`

Read the actual file and enum before adding `TokenExchange`. Do NOT assume file location.
Verify the VP-153 harness file locations (grep `MERGE-GATE-VP153-FULL`) before implementing
AC-005 — the harness files need `TokenExchange` arms in the same commit.

### T-02: Add AuthAcquisitionConfig, CachedAuthToken, ExpiryMode structs
Per ADR-054 §D3/D4. Ensure all three are `#[non_exhaustive]`. Update the non-exhaustive
gate (92→95) in `scripts/check-non-exhaustive.sh`, `CLAUDE.md`, and
`scripts/check-non-exhaustive-per-symbol.py` in the same commit.

### T-03: Implement DeclarativeHttpAuthProvider
Per ADR-054 §D4. Must implement `AuthProvider` trait. Requires clock injection
(`now_fn: Arc<dyn Fn() -> u64 + Send + Sync>`) for testability. Production constructor
uses `reqwest::Client` with rustls-tls (AC-009). Test-only `new_for_test(now_fn)`.

### T-04: Add Rule 10 / E-SPEC-028 inside SpecLoader::parse()
Per ADR-054 §D10 and BC-2.16.009 §Integration function. Rule 10 is added inside
`SpecLoader::parse()` as the final gate before `Ok(spec)` is returned — after Rule 8
(probe_table) and Rule 9 (header_scheme). Rule 10 is NOT added to `validate_sensor_spec()`;
that function covers Rules 1–5 only, has zero production callers, and placement there makes
E-SPEC-028 unreachable from `add_sensor_spec` and hot-reload. All 8 sub-conditions per AC-003.
Collect-all semantics — all E-SPEC-028 errors are collected in a single multi-error pass inside
`SpecLoader::parse()` (no fail-fast). SAP-3: each sub-condition must be reachable via
`parse_and_validate_spec_toml()`, which calls `SpecLoader::parse()` as its first act.

### T-05: Migrate crowdstrike.sensor.toml
Delete `auth_plugin = "crowdstrike-oauth2"` line. Add `[auth_acquisition]` block per
ADR-054 D2. Verify that `parse_and_validate_spec_toml()` accepts the updated spec.

### T-06: Delete crowdstrike-oauth2.prx crate
Locate the crate directory. Run grep to confirm no production callers remain. Delete
the source directory. Remove from workspace `Cargo.toml` members. Run `just check`.

### T-07: Update step9a_populate_adapter_registry
Per ADR-054 §D7. Add `TokenExchange` arm. Rewrite `Oauth2ClientCredentials` arm to
use `DeclarativeHttpAuthProvider`. Verify the adapter is constructed with full Arc-DI
wiring per ADR-022 (not a placeholder construct).

### T-08: Write VP-159 wiremock integration tests
Three test cases: lazy acquisition, cache hit, cache refresh on expiry. Use the
`now_fn` clock injection to control expiry timing without sleeping.

### T-09: Verify VP-153 MERGE-GATE-VP153-FULL passes
Add `TokenExchange` arms to both VP-153 harness files in the same commit as the
`TokenExchange` variant addition. Run the VP-153 harness and confirm it passes.

---

## Token Budget Estimate

| Context source | Estimated tokens |
|----------------|-----------------|
| This story spec | ~3,000 |
| ADR-054 (full — 598+ lines) | ~8,000 |
| `crates/prism-spec-engine/src/validation.rs` | ~3,000 |
| `crates/prism-spec-engine/src/types.rs` (AuthType, new structs) | ~2,500 |
| `crates/prism-bin/src/spec_driven_adapter.rs` | ~3,000 |
| `crates/prism-sensors/specs/crowdstrike.sensor.toml` | ~1,500 |
| VP-153 harness files (2 files) | ~2,000 |
| VP-159 wiremock test setup | ~2,000 |
| BC-2.16.009 (Rule 10 context) | ~2,000 |
| Running test output (nextest) | ~2,000 |
| **Total estimate** | **~29,000** |

29,000 tokens is above the 20–30% threshold. This story SHOULD be split if context pressure
is encountered. Recommended split points:
- **Dispatch A (data structures + validation):** T-01, T-02, T-04 (AuthType + AuthAcquisitionConfig + Rule 10)
- **Dispatch B (provider implementation + tests):** T-03, T-08 (DeclarativeHttpAuthProvider + VP-159)
- **Dispatch C (integration + retirement):** T-05, T-06, T-07, T-09 (CrowdStrike migration + plugin deletion + adapter registry + VP-153)

The test-writer can write failing stubs for all three dispatches in one pass; implementer dispatches B and C after A is green.

---

## Previous Story Intelligence

**From S-WAVE-A-ENGINE-001 (dependency):**
- `header_scheme` grammar is live after ENGINE-001 merges. CrowdStrike TOML has
  `auth_type = "oauth2_client_credentials"` and no `header_scheme` — it is on absence
  path A (no header_scheme required). No Rule 9 interaction for CrowdStrike.
- The ADR-022 Arc-DI wiring rule: `DeclarativeHttpAuthProvider::new()` must receive real
  Arc dependencies (not placeholder constructs). The production constructor should accept
  `credential_resolver: Arc<dyn CredentialResolver>` as a parameter.

**General lessons:**
- ADR-050 rustls-tls: every new reqwest dependency MUST have
  `default-features = false, features = ["rustls-tls"]`. Missing this caused ~65s macOS
  Keychain init overhead in past PRs. Check Cargo.toml of the crate that owns
  DeclarativeHttpAuthProvider before writing any reqwest usage.
- VP-153: "add TokenExchange arm to both harness files in same commit." The "both" is
  important — the VP harness has TWO files. Grep for MERGE-GATE-VP153-FULL to find both.

---

## Architecture Compliance Rules

1. **ADR-022 §C — Wiring, not redesign.** Adding `Arc<dyn CredentialResolver>` to
   `DeclarativeHttpAuthProvider::new()` is wiring. Do not construct a placeholder
   `CredentialResolver` stub — wire the real one.

2. **ADR-050 — rustls-tls mandatory.** All new reqwest Cargo.toml entries in any crate
   touched by this story MUST use `default-features = false, features = ["rustls-tls"]`.
   Forbidden: `native-tls`, `default-tls`, `native-tls-alpn`, `native-tls-vendored`.

3. **CLAUDE.md §Non-exhaustive gate.** All three new types (AuthAcquisitionConfig,
   CachedAuthToken, ExpiryMode) must be `#[non_exhaustive]`. Gate EXPECTED bumps from
   92 to 95; all three counter files update in same commit.

4. **BC-2.16.009 collect-all (VP-059).** Rule 10 errors are collected in the same
   `Vec<SpecError>` as Rules 1–5 errors. No separate fail-fast gate for Rule 10.

5. **ADR-054 §D7 — Must merge AFTER S-WAVE-A-ENGINE-001.** Do not attempt to merge
   this story before ENGINE-001 is merged.

6. **VP-153 MERGE-GATE-VP153-FULL — same commit.** `TokenExchange` arms in VP-153
   harness files must be in the SAME commit as the `TokenExchange` variant in the
   `AuthType` enum. The gate is designed to catch split-commit drift.

---

## Library & Framework Requirements

| Library | Version | Source of truth |
|---------|---------|----------------|
| `reqwest` | workspace pinned, `default-features = false, features = ["rustls-tls"]` | ADR-050; CLAUDE.md §reqwest TLS backend |
| `wiremock` | workspace pinned (dev-dependency) | `architecture/dependency-graph.md §External Dependencies` |
| `arc-swap` | workspace pinned | same |

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-spec-engine/src/types.rs` (TBD path) | MODIFY | Add AuthType::TokenExchange, AuthAcquisitionConfig, ExpiryMode |
| `crates/prism-spec-engine/src/auth/declarative_http.rs` (new) | CREATE | DeclarativeHttpAuthProvider |
| `crates/prism-spec-engine/src/auth/mod.rs` | MODIFY | re-export new module |
| `crates/prism-spec-engine/src/validation.rs` | MODIFY | Add Rule 10 / E-SPEC-028 |
| `crates/prism-sensors/specs/crowdstrike.sensor.toml` | MODIFY | auth_plugin → [auth_acquisition] |
| `crates/crowdstrike-oauth2.prx/` (TBD exact name) | DELETE | Entire crate directory |
| `Cargo.toml` (workspace root) | MODIFY | Remove crowdstrike-oauth2.prx from members |
| `crates/prism-bin/src/spec_driven_adapter.rs` | MODIFY | TokenExchange arm + Oauth2 rewrite |
| VP-153 harness files (2 — grep to locate) | MODIFY | Add TokenExchange arms |
| `scripts/check-non-exhaustive.sh` | MODIFY | EXPECTED 92→95 |
| `scripts/check-non-exhaustive-per-symbol.py` | MODIFY | EXPECTED_COUNT + EXPECTED_SYMBOLS updated |
| `CLAUDE.md` | MODIFY | "EXPECTED=92" sentence updated to 95 |

---

## Verification Properties

| VP | Description | Applicability |
|----|-------------|---------------|
| VP-153 | SensorAuth Runtime Cross-Composition Prevention | MERGE-GATE-VP153-FULL: TokenExchange arms required in same commit as enum variant |
| VP-159 | Lazy acquisition + refresh-on-expiry invariants | Wiremock-based integration tests for DeclarativeHttpAuthProvider |

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.5 | 2026-07-26 | story-writer | FB61 gate-review DEFECT-1: remove fabricated RED_RATIO formula (Density = 24/10 ACs = 2.4) from §Red Gate density check; replace with orchestrator-computation note per per-story-delivery.md §Step 3.5, citing BC-5.38.002/BC-5.38.003 |
| 1.4 | 2026-07-26 | story-writer | FB61 MED-016: expand §Red Gate tests from 2 to 24 RGTs (RG-001..RG-024) covering all 10 ACs and EC-004/EC-006; §Tasks reordered — Red Gate section precedes T-01..T-09 implementation tasks; BC-5.38.001 density updated 2→24; BC-2.01.017 relevance cell updated — stale "(PO amendment needed)" replaced with accurate v1.10 content description (§P2 `"raw"` dispatch row, §P3 6-value canonical auth_type set, AC-008 adapter wiring) |
| 1.3 | 2026-07-26 | story-writer | FB60 MED-008: pin BC-2.01.017 from `current` to v1.10 in §Behavioral Contracts table |
| 1.2 | 2026-07-26 | story-writer | FB58 close OBLIG-MED004-RULE10-BOUNDARY-001: adds AC-010 (Rule 9→Rule 10 fail-fast boundary with Rule 10 live; BC-2.16.009 §Invariants v1.28 authority; sub-condition D10(d) missing `token_response_path` as genuine violation; RGT-010-B control proves fixture is a real Rule 10(d) rejection) plus §Red Gate Tests section with RGT-010-A (boundary) and RGT-010-B (control); updates BC-2.16.009 version pin current→v1.28 in §Behavioral Contracts table and PO-002 body; cross-references ENGINE-001 AC-026/RG-031 as sending side. AC count: 9→10. RGT count: 0→2. |
| 1.1 | 2026-07-25 | story-writer | FB52b re-derive against ADR-054 v0.55 and BC-2.16.009 v1.27: (CRIT-001) AC-003 rewritten with all 8 ratified §D10 sub-conditions (a)–(h); removed 4 invented conditions that contradicted §D3 (dotted path not `$.`-prefixed) and §D3 optional default semantics; corrected sub-condition labels (a)–(h) throughout AC-003, EC-001..EC-005, and T-04. (CRIT-002) Rule 10 execution site re-anchored from `validate_sensor_spec()` to `SpecLoader::parse()` across all 6 occurrences (§Scope Summary D10 row, AC-003 body, §Architecture Mapping file/function row, T-04 title and body); all surviving `validate_sensor_spec` references are now negative constructions. (HIGH-005) PO-001 retired — the cited BC section does not exist (POL-21 phantom anchor) and the relevant BC amendments are already EXECUTED per ADR-054 D11; PO-002 retired — BC-2.16.009 Rule 10 fully specified since v1.12. (MED-013) AC-006 wrong-error-code corrected: duplicate-sensor-id code replaced with E-SPEC-028 sub-condition (b). EC-008 TBD resolved per BC-2.16.009 Rule 10(a)/EC-009-039. §Authority updated to v0.55. |
| 1.0 | 2026-07-25 | story-writer | Initial stub from ADR-054 §D1/D2/D3/D4/D5/D7/D10; VP-153/VP-159 gates; split dispatch guidance; PO dependency encoding |
