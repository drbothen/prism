---
document_type: story
story_id: S-ADR054-WAVE-A-001
title: "Declarative HTTP Auth Acquisition — DeclarativeHttpAuthProvider, TokenExchange, Rule 10, CrowdStrike TOML Migration, crowdstrike-oauth2.prx Retirement"
version: "1.0"
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
# BC status: BC-2.01.017 §dispatch table must be amended to add TokenExchange arm (PO task per ADR-053 D5
# amendment manifest). BC-2.16.009 needs Rule 10 description added for [auth_acquisition] validation
# (PO task). BC-2.06.003 covers credential refs for the new token_exchange flow. All three BCs are
# currently active. BC amendments are required before status: ready transition.
assumption_validations: []
risk_mitigations: []
---

# S-ADR054-WAVE-A-001: Declarative HTTP Auth Acquisition

## Authority

**ADR-054 v0.52** (accepted 2026-07-22) is the authoritative design document.
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
| D10 | Rule 10 added to `validate_sensor_spec()`; E-SPEC-028 (8 sub-conditions for `[auth_acquisition]` validation) |

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

`validate_sensor_spec()` implements Rule 10 with E-SPEC-028. The 8 sub-conditions from
ADR-054 §D10 are enforced:

D10(a): `[auth_acquisition]` present + auth_type ∉ {oauth2_client_credentials, token_exchange} → E-SPEC-028
D10(b): auth_type ∈ {oauth2_client_credentials, token_exchange} AND `auth_plugin` present → E-SPEC-028 (always rejected regardless of `[auth_acquisition]`)
D10(c): `token_exchange` auth_type WITHOUT `[auth_acquisition]` block → E-SPEC-028
D10(d): `[auth_acquisition]` present but `token_path` absent/empty → E-SPEC-028
D10(e): `expiry_mode = "absolute_utc_string"` without `expiry_field` → E-SPEC-028
D10(f): `expiry_mode = "relative_seconds"` without `expiry_field` → E-SPEC-028
D10(g): `token_response_path` malformed (not starting with `$.`) → E-SPEC-028
D10(h): `ttl_buffer_secs = 0` → E-SPEC-028

A test for each sub-condition verifies rejection via `parse_and_validate_spec_toml()`
(SAP-3: reachable from integration surface, not just from `validate_sensor_spec()` directly).

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
without E-SPEC-028 errors and without E-SPEC-009 (auth_plugin with oauth2_client_credentials
is rejected by D10(b)).

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

---

## Product-Owner Dependencies

### PO-001: BC-2.01.017 amendment for TokenExchange dispatch table row (BLOCKS status: ready)

BC-2.01.017 §Adapter Registration dispatch table must gain a `TokenExchange` row describing
how `step9a_populate_adapter_registry()` handles `auth_type = "token_exchange"`. Per ADR-053
§D5 amendment manifest, this is a PO task.

### PO-002: BC-2.16.009 Rule 10 documentation (BLOCKS status: ready)

BC-2.16.009 §Validation Rules section must add Rule 10 description covering E-SPEC-028 and
its 8 sub-conditions from ADR-054 §D10. The rule exists in the ADR but is not yet in the BC.

---

## Architecture Mapping

| Component | File | Pure/Effectful | Change |
|-----------|------|---------------|--------|
| `AuthType` enum | `crates/prism-spec-engine/src/types.rs` (TBD — locate via grep) | Pure (data) | Add `TokenExchange` variant (D1) |
| `AuthAcquisitionConfig` | `crates/prism-spec-engine/src/types.rs` (TBD) | Pure (data) | New struct (D3) |
| `CachedAuthToken` | `crates/prism-spec-engine/src/auth/` (TBD) | Pure (data) | New struct (D4) |
| `ExpiryMode` | `crates/prism-spec-engine/src/types.rs` or `auth/` (TBD) | Pure (data) | New enum (D3) |
| `DeclarativeHttpAuthProvider` | `crates/prism-spec-engine/src/auth/` | Effectful (HTTP client) | New struct (D4) |
| `validate_sensor_spec()` | `crates/prism-spec-engine/src/validation.rs` | Pure (validation) | Add Rule 10 / E-SPEC-028 (D10) |
| `step9a_populate_adapter_registry()` | `crates/prism-bin/src/spec_driven_adapter.rs` | Effectful (DI wiring) | Add TokenExchange arm; rewrite Oauth2 arm (D7) |
| `crowdstrike.sensor.toml` | `crates/prism-sensors/specs/` | Pure (config) | Migrate auth_plugin → [auth_acquisition] (D2) |
| `crowdstrike-oauth2.prx` | workspace root (TBD path) | Effectful (plugin binary) | DELETE (D5) |

---

## Behavioral Contracts

| BC | Version | Relevance |
|----|---------|-----------|
| BC-2.16.009 | current | Rule 10 / E-SPEC-028 — [auth_acquisition] validation |
| BC-2.01.017 | current | Adapter dispatch table — TokenExchange arm (PO amendment needed) |
| BC-2.06.003 | v1.3 | Credential refs for the token_exchange flow |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | [auth_acquisition] block present with auth_type = "bearer_static" | D10(a): E-SPEC-028 — bearer_static does not use declarative acquisition |
| EC-002 | auth_type = "oauth2_client_credentials" with auth_plugin present | D10(b): E-SPEC-028 — auth_plugin is always rejected for this auth_type regardless of [auth_acquisition] |
| EC-003 | auth_type = "token_exchange" without [auth_acquisition] | D10(c): E-SPEC-028 — block is required |
| EC-004 | [auth_acquisition] block with empty token_path = "" | D10(d): E-SPEC-028 |
| EC-005 | ttl_buffer_secs = 0 | D10(h): E-SPEC-028 |
| EC-006 | VP-159: token acquisition HTTP endpoint returns 401 | DeclarativeHttpAuthProvider returns E-SENSOR-401 (or equivalent) rather than panicking or returning stale token |
| EC-007 | Clock injection: get_token() called at expiry boundary (exactly at expiry - ttl_buffer_secs) | Token is refreshed (boundary is inclusive for refresh) |
| EC-008 | CrowdStrike TOML without [auth_acquisition] after this story merges | D10(c) would fire for oauth2_client_credentials without [auth_acquisition] — wait, D10(c) is for token_exchange only. For oauth2_client_credentials, [auth_acquisition] is optional (D10(a) only fires if [auth_acquisition] is present with wrong auth_type). Verify this against ADR-054 §D10 — the TBD here is whether oauth2_client_credentials REQUIRES [auth_acquisition] after D5 (plugin retirement). If yes, existing crowdstrike TOML without the block would break. |

---

## Tasks

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

### T-04: Add Rule 10 / E-SPEC-028 to validate_sensor_spec()
Per ADR-054 §D10. All 8 sub-conditions. Collect-all semantics (VP-059) — Rule 10 errors
are collected with Rules 1–5 errors, not fail-fast. SAP-3: each sub-condition must be
reachable via `parse_and_validate_spec_toml()` (integration surface), not just via
`validate_sensor_spec()` directly.

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
| 1.0 | 2026-07-25 | story-writer | Initial stub from ADR-054 §D1/D2/D3/D4/D5/D7/D10; VP-153/VP-159 gates; split dispatch guidance; PO dependency encoding |
