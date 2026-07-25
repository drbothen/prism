---
document_type: story
story_id: S-WAVE-A-ENGINE-001
title: "prism-spec-engine: SensorSpec header_scheme field — Rule 9 validation, build_request header-injection dispatch, and AuthProvider::get_token trait method (Wave-A ADR-053 standalone engine prerequisite)"
wave: wave-a
epic_id: E-SPEC-ENGINE
priority: P1
status: draft
version: "2.1"
updated: "2026-07-24"
level: "L3"
producer: story-writer
timestamp: "2026-07-24T00:00:00Z"
tdd_mode: strict
subsystems: [SS-16]
# Subsystem anchor justification:
#   SS-16 (Spec Engine) owns all of spec_parser.rs, pipeline.rs, auth_provider.rs,
#   and error.rs in prism-spec-engine. Every file touched by this story lives
#   within SS-16's module boundary. No other subsystem is crossed.
crates_touched: [prism-spec-engine, prism-core]
# prism-core: ESpec027 variant added to SpecErrorCode enum per Q1 architect ruling
#   (wave-a-engine-story-adjudication-Q1-Q5.md). Rule 9 returns PrismError::Spec
#   following the Rule 8 pattern (SpecLoader::parse returns PrismError, not SpecEngineError).
# prism-bin is NOT touched by this story.  The vp153_rule_c_shaped_probe.rs
# token_exchange arm addition (formerly T-F03) is ROUTED TO S-ADR054-WAVE-A-001
# per ORCHESTRATOR RULING (2026-07-24): token_exchange arm addition requires
# VALID_AUTH_TYPES + AuthType::TokenExchange, both of which land in ADR-054.
target_module: prism-spec-engine
capabilities: []
behavioral_contracts:
  - BC-2.16.009
  - BC-2.01.017
  - BC-2.16.014
  - BC-2.01.016
# BC-2.16.009: Rule 9 — header_scheme validation (template a/b/c) + absence paths A/B
# BC-2.01.017: StaticCookieAuthProvider; P2 dispatch table is header_scheme-keyed per ADR-053 D2
# BC-2.16.014: P9 — execute_impl / execute_step call sites switch to get_token()
# BC-2.01.016: SensorAuth open trait; Rule A foundation for VP-153 re-verification gate
verification_properties: [VP-153]
# VP-153: SensorAuth Runtime Cross-Composition Prevention.
# Engine-story gate (PARTIAL per ORCHESTRATOR RULING 2026-07-24): run prism-spec-engine
# VP-153 suite WITHOUT adding token_exchange strategy arms; verify existing active proptests
# pass including Rule A with updated E-SPEC-012 Display. Token_exchange arm ADDITION
# (MERGE-GATE-VP153-FULL) belongs to S-ADR054-WAVE-A-001.
# See §Tasks MERGE-GATE-VP153-PARTIAL and Architecture Compliance Rule 9.
depends_on: []
# This story is the FIRST in the ADR-054 §D7 sequencing chain.
# No product-story hard dependencies — can enter Wave 1 of wave-a scheduling.
blocks:
  - S-ADR054-WAVE-A-001          # TBD — ADR-054 declarative auth implementation story (DeclarativeHttpAuthProvider + TokenExchange + Rule 10)
  - S-WAVE-A-CYBERINT-SPEC-001   # TBD — Cyberint spec migration (cyberint.sensor.toml must gain header_scheme before engine story lands, or vice versa)
  - S-WAVE-A-ARMIS-REMEDIATION-001  # TBD — Armis token-exchange remediation story
# blocks anchor justifications:
#   S-ADR054-WAVE-A-001: ADR-054 §D7 explicit merge dependency — "implementation stories
#     MUST merge AFTER the ADR-053 standalone Wave-A engine story." DeclarativeHttpAuthProvider
#     overrides get_token() added here; Rule 10 builds on Rule 9 registered here.
#   S-WAVE-A-CYBERINT-SPEC-001: Once this story lands, cyberint.sensor.toml
#     (which uses auth_type = "cookie_roundtrip" with absent header_scheme) is REJECTED
#     at spec-load time by E-SPEC-027 template (c). The Cyberint spec migration story MUST
#     add header_scheme = "cookie:<name>" to that file before or in the same batch as
#     this story merging — the stories must co-land.
#   S-WAVE-A-ARMIS-REMEDIATION-001: Armis spec uses token_exchange auth_type (ADR-053 §D1).
#     The token_exchange coherence matrix row (out of scope for this story) belongs to
#     S-ADR054-WAVE-A-001. Armis spec migration depends on the TokenExchange variant existing.
points: 8
# Points justification:
#   - SensorSpec::header_scheme Option<String> field + #[serde(default)] in spec_parser.rs: 0.5 pt
#   - Rule 9 validation (parse + 3 E-SPEC-027 templates + coherence matrix, 5 variants): 2.5 pt
#   - build_request() header-injection dispatch switch (bearer/raw/cookie:*/absent paths): 1.5 pt
#   - AuthProvider::get_token() default method + execute_impl/execute_step call-site changes: 1.0 pt
#   - E-SPEC-012 + E-SPEC-013 Display rewrites (POL-24 code alignment): 0.5 pt
#   - VP-153 test-inventory fn-name correction (T-F01) + partial proof re-run
#     (MERGE-GATE-VP153-PARTIAL, existing active arms only): 0.3 pt
#     [T-F02/T-F03/T-F04 token_exchange arm activation ROUTED TO S-ADR054-WAVE-A-001
#      per ORCHESTRATOR RULING (2026-07-24) — see Architecture Compliance Rule 9]
#   - TDD test coverage across all 21 ACs (unit + integration + MCP surface): 1.5 pt
#   Total: 8 points
estimated_days: 2.0
risk: MEDIUM
# Risk justification:
#   MEDIUM because this story touches four production files across the
#   spec-load/execution boundary (spec_parser, pipeline, auth_provider, error)
#   plus two test files in two crates, all coordinated as a single atomic
#   wave-a delivery. The individual changes are well-specified but the
#   co-land ordering (Cyberint spec migration) requires scheduling discipline.
#   No novel algorithm design — risk is coordination and test coverage breadth.
assumption_validations: []
risk_mitigations: []
---

# S-WAVE-A-ENGINE-001: Wave-A ADR-053 Standalone Engine Story — SensorSpec `header_scheme` Field, Rule 9 Validation, `build_request` Header-Injection Dispatch, and `AuthProvider::get_token` Trait Method

## Narrative

As a spec-engine maintainer, I want the sensor spec to carry an explicit `header_scheme`
field that decouples token injection style (bearer/raw/cookie) from acquisition method
(`auth_type`), so that sensor specs can declare any supported token-injection style
independent of how the token was acquired — enabling the Wave-A Cyberint dual-surface
and Armis token-exchange use-cases without hardcoding sensor-name-conditional dispatch
logic (POL-36 INV-014-001).

This story is the **first story in the ADR-054 §D7 sequencing chain**. The ADR-054
implementation story (DeclarativeHttpAuthProvider, AuthType::TokenExchange, Rule 10)
MUST NOT merge before this story. See §Architecture Compliance Rules for the explicit
merge-dependency encoding.

## Behavioral Contracts

| BC | Title | Version | Scope in This Story |
|----|-------|---------|---------------------|
| BC-2.16.009 | Spec File Validation | v1.25 | Rule 9 only — `header_scheme` value validation, 3 E-SPEC-027 templates, coherence matrix (5 existing variants), absence paths A/B. Cookie name constraint: RFC 6265 tchar per SEC-001 CWE-20/CWE-74 fix (15-char set with backtick, RFC 9110 §5.6.2 order). New in v1.25: §Entry points and function coverage sub-section asserts `add_sensor_spec` MUST reach `SpecLoader::parse()` for Rule 9 coverage on the sole active injection vector. Rule 10 (`[auth_acquisition]`) is OUT OF SCOPE. |
| BC-2.01.017 | StaticCookieAuthProvider — No Login Roundtrip | v1.10 | P2 dispatch table: build_request() switches from auth_type-keyed to header_scheme-keyed dispatch per ADR-053 D2. INV-COOKIE-004: no Authorization header for cookie injection. |
| BC-2.16.014 | Declarative Auth Acquisition Token Lifecycle | v1.19 | P9 only — execute_impl and execute_step call sites change from `acquire_token()` to `get_token()`. issue_request_with_retry 401 path remains `acquire_token()`. |
| BC-2.01.016 | SensorAuth Open Trait Contract | v1.15 | AuthProvider trait extension: `get_token()` default method added. Foundation for VP-153 re-verification gate (Rule A E-SPEC-012 Display alignment). |

## Acceptance Criteria

### Tier 1 — SensorSpec::header_scheme Field Deserialization

**AC-001 — absent field deserializes to None (absence path A)**
A sensor TOML spec that contains no `header_scheme` key deserializes `SensorSpec::header_scheme`
to `None`. For all `auth_type` values OTHER than `cookie_roundtrip`, `None` passes
spec-load validation silently (absence path A — runtime `build_request()` injects
`Authorization: Bearer` by default). Spec load returns `Ok`.
(traces to BC-2.16.009 Rule 9 precondition: "absent header_scheme passes silently for
all auth_types except cookie_roundtrip")

**AC-002 — "bearer" is a valid value**
A spec with `header_scheme = "bearer"` (any supported `auth_type` except `cookie_roundtrip`)
deserializes without error. `SpecLoader::parse()` returns `Ok`.
(traces to BC-2.16.009 Rule 9 postcondition: syntactically valid value with compatible auth_type accepted)

**AC-003 — "raw" is a valid value**
A spec with `header_scheme = "raw"` and a `raw`-compatible `auth_type` (e.g.,
`bearer_static`, `oauth2_client_credentials`, `custom_via_plugin`) deserializes without
error. `SpecLoader::parse()` returns `Ok`.
(traces to BC-2.16.009 Rule 9 postcondition: syntactically valid value with compatible auth_type accepted)

**AC-004 — "cookie:\<name\>" with non-empty RFC 6265 tchar name is valid**
A spec with `header_scheme = "cookie:access_token"` (non-empty cookie name consisting
entirely of RFC 6265 tchar characters — `access_token` uses only letters, digits, and
`_`, all of which are tchar per RFC 9110 §5.6.2) and `auth_type = "cookie_roundtrip"`
deserializes without error. `SpecLoader::parse()` returns `Ok`.
(traces to BC-2.16.009 Rule 9 postcondition: syntactically valid cookie value with compatible auth_type accepted)

### Tier 2 — Rule 9 Validation Errors (E-SPEC-027)

**Test assertion pattern (all Tier 2 ACs):** Rule 9 returns `Err(PrismError::Spec(spec_err))`
from `SpecLoader::parse()`. Assert `spec_err.message == expected_verbatim_string`. Do NOT
use `err.to_string()` — the outer `PrismError::Spec` Display wraps the message with additional
text. Pattern:
```rust
let err = SpecLoader::parse(bad_toml).unwrap_err();
let prism_core::PrismError::Spec(ref spec_err) = err else { panic!("expected Spec error, got {:?}", err) };
assert_eq!(spec_err.message, expected_verbatim_string);
```

**AC-005 — syntactically invalid value → E-SPEC-027 template (a)**
A spec with `header_scheme = "garbage"` (not one of `bearer`, `raw`, `cookie:<name>`)
is rejected at spec-load time. `spec_err.message` (extracted from `PrismError::Spec`)
matches E-SPEC-027 template (a) VERBATIM per error-taxonomy.md v2.68:
``"sensor '{sensor_id}' has invalid header_scheme = '{value}'. Valid values: bearer, raw, cookie:<name> (non-empty name required, RFC 6265 token characters only: A-Z a-z 0-9 ! # $ % & ' * + - . ^ _ ` | ~)"``
where `{sensor_id}` and `{value}` are substituted with the spec's values.
(traces to BC-2.16.009 Rule 9 postcondition 2a: syntactically invalid value → E-SPEC-027(a))

**AC-006 — "cookie:" with empty name → E-SPEC-027 template (a)**
A spec with `header_scheme = "cookie:"` (the `cookie:` prefix present but cookie name
is the empty string) is rejected at spec-load time. `spec_err.message` (from
`PrismError::Spec`) matches E-SPEC-027 template (a). The "non-empty name required"
clause is the applicable reason — the tchar check vacuously fails on an empty string.
(traces to BC-2.16.009 Rule 9 postcondition 2a: empty cookie name is syntactically invalid)

**AC-007 — "cookie:has:extra:colon" → E-SPEC-027 template (a)**
A spec with `header_scheme = "cookie:has:extra:colon"` (`:` present in the name
portion, after the first `:` separator) is rejected at spec-load time. `spec_err.message`
(from `PrismError::Spec`) matches E-SPEC-027 template (a). The applicable reason is that
`:` is NOT an RFC 6265 tchar character — the tchar set (``A-Z a-z 0-9 ! # $ % & ' * + - . ^ _ ` | ~``)
excludes all RFC 9110 §5.6.2 delimiters including `:`.
(traces to BC-2.16.009 Rule 9 postcondition 2a: `:` fails tchar check; EC-009-033)

**AC-008 — coherence violation (cookie_roundtrip + bearer) → E-SPEC-027 template (b)**
A spec with `auth_type = "cookie_roundtrip"` and `header_scheme = "bearer"` is rejected
at spec-load time. `spec_err.message` (from `PrismError::Spec`) matches E-SPEC-027
template (b) VERBATIM:
`"sensor '{sensor_id}': auth_type = '{auth_type}' does not permit header_scheme = '{value}'; allowed for this auth_type: {allowed_set}"`
where `{allowed_set}` for `cookie_roundtrip` is `cookie:<name>` (only).
(traces to BC-2.16.009 Rule 9 postcondition 2b: coherence violation → E-SPEC-027(b))

**AC-009 — api_key + raw → E-SPEC-027 template (b) (api_key permits bearer only)**
A spec with `auth_type = "api_key"` and `header_scheme = "raw"` is rejected at
spec-load time. `spec_err.message` (from `PrismError::Spec`) matches E-SPEC-027
template (b). `api_key` permits `bearer` ONLY; `raw` is not in the allowed set.
The `{allowed_set}` substitution is `bearer`.
(traces to BC-2.16.009 Rule 9 postcondition 2b: api_key coherence — raw not permitted)

**AC-010 — absent header_scheme + cookie_roundtrip → E-SPEC-027 template (c) (absence path B)**
A spec with `auth_type = "cookie_roundtrip"` and NO `header_scheme` key (i.e.,
`SensorSpec::header_scheme = None`) is rejected at spec-load time. `spec_err.message`
(from `PrismError::Spec`) matches E-SPEC-027 template (c) VERBATIM:
`"sensor '{sensor_id}': auth_type = 'cookie_roundtrip' requires an explicit header_scheme = 'cookie:<name>' value; absent header_scheme is not valid for cookie_roundtrip auth (cookie name unknown)"`
(traces to BC-2.16.009 Rule 9 absence path B postcondition: absent + cookie_roundtrip is an error)

**AC-011 — absent header_scheme + bearer_static → spec loads OK (absence path A)**
A spec with `auth_type = "bearer_static"` and NO `header_scheme` key loads without
error. `SensorSpec::header_scheme = None` is silence-permitted for all non-cookie
auth_types (absence path A). `build_request()` subsequently applies the silent
`"bearer"` runtime default.
(traces to BC-2.16.009 Rule 9 absence path A postcondition: absent + non-cookie → no error)

### Tier 3 — build_request() Header-Injection Dispatch

**AC-012 — header_scheme "bearer" injects Authorization: Bearer**
A call to the `build_request()` free function with `header_scheme = Some("bearer".to_string())`
produces a `reqwest::RequestBuilder`; calling `.build()?` on it yields a `reqwest::Request`
whose `Authorization` header value (accessed via `reqwest::Request::headers()` and
`reqwest::header::AUTHORIZATION`) is `Bearer <token>`.
The exact prefix is `"Bearer "` (capital B, space before token). No other token-injection
header is present. RG-011 lives in `pipeline.rs`'s inline `#[cfg(test)] mod tests`
(not in `tests/`) because `build_request` is private; header inspection uses `.build()?`.
(traces to BC-2.01.017 P2 postcondition: header_scheme "bearer" → Authorization: Bearer <token>)

**AC-013 — header_scheme "raw" injects Authorization without "Bearer" prefix**
A call to `build_request()` with `header_scheme = Some("raw".to_string())` produces
a `reqwest::RequestBuilder`; calling `.build()?` yields a request whose `Authorization`
header (via `reqwest::header::AUTHORIZATION`) contains the raw token string without the
`"Bearer "` prefix. The exact header value equals the token string.
RG-012 lives in `pipeline.rs`'s inline `#[cfg(test)] mod tests`.
(traces to BC-2.01.017 P2 postcondition: header_scheme "raw" → Authorization: <token> raw)

**AC-014 — header_scheme "cookie:\<name\>" injects Cookie header; no Authorization header**
A call to `build_request()` with `header_scheme = Some("cookie:access_token".to_string())`
produces a `reqwest::RequestBuilder`; calling `.build()?` yields a request whose `Cookie`
header contains `access_token=<token>`. The request carries NO `Authorization` header.
(INV-COOKIE-004 enforcement per BC-2.01.017.) RG-013 lives in `pipeline.rs`'s inline
`#[cfg(test)] mod tests`; asserts `Cookie` header bytes (wire-shape assertion discipline
2026-07-13) and absence of `Authorization` header.
(traces to BC-2.01.017 P2 postcondition: header_scheme "cookie:<name>" → Cookie: <name>=<token>;
INV-COOKIE-004: no Authorization header for cookie_roundtrip sensors)

**AC-014b — absent header_scheme (None) uses silent bearer default in build_request()**
A call to `build_request()` with `header_scheme = None` produces a `reqwest::RequestBuilder`;
calling `.build()?` yields a request whose `Authorization` header (via
`reqwest::header::AUTHORIZATION`) has the value `Bearer <token>` — identical to AC-012's
output. No error is returned; absent `header_scheme` silently defaults to bearer
injection in the execution path (distinct from spec-load: absence path A permits
this, but absence path B already rejected the cookie_roundtrip + None combination
at spec-load time, so this path is only reached for non-cookie auth_types).
RG-014 lives in `pipeline.rs`'s inline `#[cfg(test)] mod tests`.
(traces to BC-2.16.009 Rule 9 absence path A postcondition: silent bearer default in runtime;
BC-2.01.017 P2 postcondition: None → bearer fallback)

### Tier 4 — AuthProvider::get_token Trait Method (F-WASE-P7-HIGH-001)

**AC-015 — get_token() default impl delegates to acquire_token() at runtime**
After adding `get_token()` to the `AuthProvider` trait with a default implementation
body that delegates to `self.acquire_token(spec, client_id)`, calling `get_token()` on
an implementor that has not overridden `get_token()` invokes `acquire_token()` exactly
once per call. A test creates an `AuthProvider` implementor that returns a known
distinguishable token from `acquire_token()` and asserts that calling `get_token()`
returns that same known token — confirming that the default body delegates to
`acquire_token()`. Behavioral change for all 5-value as-built auth-type implementors
is zero: `get_token()` produces identical output to `acquire_token()` on any implementor
that does not override `get_token()`.

**Build-gate note (not an AC):** All 8 existing implementors (`NullAuthProvider`,
`MockAuthProvider`, `StaticCookieAuthProvider`, `BearerStaticAuthProvider`,
`BearerStaticCredentialAuthProvider`, `PluginAuthProvider`, `FailingAuthProvider`,
`ChainAuthProvider`) compile without code modification because they inherit the default
`get_token()` body. This is verified by `just check` passing (build gate), not by a
test assertion. `BearerStaticAuthProvider` lives in
`crates/prism-bin/src/spec_driven_adapter.rs` and is distinct from
`BearerStaticCredentialAuthProvider`. Do NOT add a separate compilation test for this
claim; `just check` passing is the gate.
(traces to BC-2.16.014 P9 precondition: existing providers inherit default get_token()
delegating to acquire_token() — zero behavioral change until DeclarativeHttpAuthProvider
overrides it)

**AC-016 — execute_impl and execute_step call get_token(); issue_request_with_retry 401 path calls acquire_token() unchanged**
After this story lands:
- `execute_impl` eager-acquisition block (before the `'steps:` loop): calls
  `auth_provider.get_token(spec, &context.client_id)` instead of `acquire_token()`.
- `execute_step` eager-acquisition block (top of function): calls
  `auth_provider.get_token(spec, &context.client_id)` instead of `acquire_token()`.
- `issue_request_with_retry` 401-refresh arm: STILL calls `auth_provider.acquire_token(spec, client_id)`
  unchanged (force-refresh semantics; BC-2.16.014 P5/P6 require acquire_token() here).
A test using a custom AuthProvider that tracks get_token() vs acquire_token() invocations
separately verifies that execute_impl invokes get_token() (not acquire_token()) and
that issue_request_with_retry's 401 arm invokes acquire_token() (not get_token()).
(SAP-3 spec-arm reachability: each call-site arm verified from the production executor
surface, not via synthetic AST injection. traces to BC-2.16.014 P9 postcondition:
execute_impl and execute_step use get_token(); 401 arm uses acquire_token())

### Tier 5 — E-SPEC-012 / E-SPEC-013 Display Alignment (POL-24 Code Alignment)

**AC-017 — E-SPEC-012 Display matches error-taxonomy.md v2.68 verbatim (AuthTypeCrossComposition clause)**
The `SpecEngineError::AuthTypeCrossComposition` variant's `Display` output (emitted
via `thiserror` `#[error(…)]`) matches the FIRST OR-clause of the E-SPEC-012
`message_template` in error-taxonomy.md v2.68 BYTE-FOR-BYTE (the first clause covers
`AuthTypeCrossComposition`; the second clause covers `UnknownAuthPlugin` and is NOT
changed by this story):
`"auth_type for sensor '{sensor_id}' must be a single value; got: {value}. Valid values: oauth2_client_credentials, bearer_static, cookie_roundtrip, api_key, custom_via_plugin, token_exchange"`
A unit test asserts the emitted string contains `"token_exchange"` in the "Valid values:"
clause and that the field substitution uses `{value}` (not `{provided_value}` — the
current divergent field name). The VP-153 proptest harness implicitly validates this
via the E-SPEC-012 assertion in `prop_rule_a_invalid_auth_type_rejected_with_e_spec_012`.
Add the POL-24 comment at the `#[error(...)]` rewrite site per Q5 ruling:
```rust
// POL-24: error-taxonomy.md v2.57 (Wave-A burst 3) and VP-153 v0.21 list
// token_exchange as the 6th valid value. The Display includes it now per the
// taxonomy source-of-truth contract. VALID_AUTH_TYPES will include "token_exchange"
// in S-ADR054-WAVE-A-001 — for one cycle, the message is intentionally ahead of
// the runtime constant by design (taxonomy-first, per ADR-054 D11 row).
```
(traces to BC-2.01.016 Rule A postcondition: E-SPEC-012 emitted per POL-24 taxonomy
source-of-truth; F-WASE-P16-OBS-003 engine-story D11 row)

**AC-018 — E-SPEC-013 Display matches error-taxonomy.md v2.68 verbatim**
The `SpecEngineError::MultipleCredentialRefs` variant's `Display` output matches the
E-SPEC-013 `message_template` in error-taxonomy.md v2.68:
`"auth method for sensor '{sensor_id}' declares {count} credential_refs; exactly {expected} required for auth_type '{auth_type}'"`
The variant struct gains (or renames to) fields `sensor_id`, `count`, `expected`,
`auth_type` to satisfy this template. The as-built "hardcoded 1 with no {auth_type}
or {expected} parameters" divergence is corrected atomically with the struct-field
rename. A unit test asserts the emitted string contains `{auth_type}` substituted
with the actual auth_type value.
(traces to BC-2.01.016 Rule B postcondition: E-SPEC-013 emitted per POL-24;
F-WASE-P31-MED-001 engine-story D11 row)

### Tier 6 — Security Injection Vector Coverage (SEC-001 / SAP-3)

**AC-019 — non-tchar characters in cookie name (`;`, `=`, SP, CTL) → E-SPEC-027 template (a) (SEC-001 CWE-20/CWE-74 closure)**
A spec with a `header_scheme` value whose cookie name contains any non-tchar character
is rejected at spec-load time by Rule 9 tchar validation. The security-motivating cases
are `;` (semicolon), `=` (equals), SP (space, 0x20), and CTL bytes (e.g., LF/CR).
`spec_err.message` (extracted from `PrismError::Spec`) matches E-SPEC-027 template (a)
VERBATIM for each. The `;` case — `header_scheme = "cookie:sid=x; admin"` — is the
SEC-001 primary injection vector: without Rule 9, the synthesized Cookie header would
be `"sid=x; admin={token}"`, remapping the auth credential to the attacker-controlled
key `admin`. Load-time tchar rejection at spec-load eliminates this vector. The CTL case
eliminates the SEC-002 / CWE-390 deferred `"builder error"` from reqwest as a side
effect — load-time rejection replaces the prior opaque runtime failure.
RG-020..RG-023 each drive one of the four injection-class inputs through
`SpecLoader::parse()` and assert the exact E-SPEC-027 template (a) message.
(traces to BC-2.16.009 Rule 9 postcondition 2a: non-tchar character in cookie name →
E-SPEC-027(a); EC-009-043, EC-009-044, EC-009-045, EC-009-046)

**AC-020 — non-tchar cookie name rejected via `add_sensor_spec` MCP tool surface (SAP-3 end-to-end wire-level coverage)**
Calling the `add_sensor_spec` MCP tool with `toml_content` containing
`header_scheme = "cookie:bad;name"` (`;` is not a tchar character) produces an MCP
error response at the wire level. The serialized JSON envelope consumed by the LLM
agent contains `structuredContent.error.code` matching `E-SPEC-027`. RG-024 drives this
through the actual MCP stdio surface (an integration test that invokes the MCP binary
endpoint, NOT a direct `SpecLoader::parse()` call) and asserts on the serialized JSON
output per CLAUDE.md wire-shape assertion discipline (2026-07-13). This test verifies
that the `add_sensor_spec` handler reaches `SpecLoader::parse()` so Rule 9 applies on
the sole exploitable injection surface — an implementation that calls only
`validate_sensor_spec()` in the handler bypasses Rules 8/9/10 and leaves the injection
vector open. Per BC-2.16.008, `add_sensor_spec` uses "the same validation pipeline as
startup loading"; this AC operationalizes that assertion at the wire level.
(traces to BC-2.16.009 Rule 9 §Entry points and function coverage: `add_sensor_spec`
MUST reach `SpecLoader::parse()` for Rule 9 to close the CWE-20/CWE-74 injection vector;
SAP-3: end-to-end coverage from MCP tool surface, not synthetic internal invocation)

## `auth_type × header_scheme` Coherence Matrix (Rule 9 — 5 Existing Variants)

The following 5-variant coherence matrix is fully in scope for this story.
The `token_exchange` row is EXPLICITLY OUT OF SCOPE — it ships atomically
with `AuthType::TokenExchange` in the ADR-054 story (S-ADR054-WAVE-A-001).

| auth_type | Allowed header_scheme values | Disallowed (→ E-SPEC-027(b)) |
|-----------|------------------------------|-------------------------------|
| `bearer_static` | `bearer`, `raw` | `cookie:<name>` |
| `oauth2_client_credentials` | `bearer`, `raw` | `cookie:<name>` |
| `cookie_roundtrip` | `cookie:<name>` only | `bearer`, `raw` |
| `custom_via_plugin` | `bearer`, `raw` | `cookie:<name>` |
| `api_key` | `bearer` only | `raw`, `cookie:<name>` |

Absent `header_scheme` (None):
- All 5 variants EXCEPT `cookie_roundtrip`: absence path A — silent bearer default, spec loads OK
- `cookie_roundtrip` with absent `header_scheme`: absence path B — E-SPEC-027 template (c) at spec-load

## Architecture Mapping

| Component | File | Pure/Effectful |
|-----------|------|---------------|
| `SensorSpec::header_scheme` field | `crates/prism-spec-engine/src/spec_parser.rs` | Pure (struct field, serde deserialization) |
| `SpecLoader::validate_header_scheme` (Rule 9) | `crates/prism-spec-engine/src/spec_parser.rs` | Pure (validates against coherence matrix; no I/O) |
| E-SPEC-027 error variants | `crates/prism-spec-engine/src/error.rs` | Pure (error type definitions) |
| E-SPEC-012/E-SPEC-013 Display rewrite | `crates/prism-spec-engine/src/error.rs` | Pure (Display trait only) |
| `build_request()` dispatch switch | `crates/prism-spec-engine/src/pipeline.rs` | Pure (module-level free function, 8 params, no `&self`; confirmed location per ADR-053 D2 and ADR-054 §D4) |
| `execute_impl` call-site (get_token) | `crates/prism-spec-engine/src/pipeline.rs` | Effectful (async, calls AuthProvider trait) |
| `execute_step` call-site (get_token) | `crates/prism-spec-engine/src/pipeline.rs` | Effectful (async, calls AuthProvider trait) |
| `issue_request_with_retry` (no change) | `crates/prism-spec-engine/src/pipeline.rs` | Effectful (no change; acquire_token stays on 401 path) |
| `AuthProvider::get_token()` default method | `crates/prism-spec-engine/src/auth_provider.rs` | Effectful (async, delegates to acquire_token) |
| VP-153 harness fn-name correction (T-F01; doc-fix only) | `crates/prism-spec-engine/tests/vp153_sensorauth_cross_composition.rs` | Pure (proptest harness doc-fix only; token_exchange strategy arm ADDITION is OUT OF SCOPE — ROUTED TO S-ADR054-WAVE-A-001 per ORCHESTRATOR RULING; no arm additions of any kind in this story) |
| `FetchStep` doc-comment correction | `crates/prism-spec-engine/src/spec_parser.rs` | Pure (doc-comment only) |

**Architecture Compliance:** This story does NOT introduce DeclarativeHttpAuthProvider,
AuthType::TokenExchange, or any new pub types, and does NOT add `"token_exchange"` to
the `VALID_AUTH_TYPES: &[&str]` constant in `spec_parser.rs` (ORCHESTRATOR RULING —
belongs to S-ADR054-WAVE-A-001; see Architecture Compliance Rule 9). The non-exhaustive
gate EXPECTED value stays at 92 (no new pub types, only a new field on SensorSpec and
new variants on the existing SpecEngineError enum). Bump to 95 belongs to S-ADR054-WAVE-A-001.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `header_scheme = ""` (empty string) | E-SPEC-027(a) — empty string is neither bearer, raw, nor valid cookie:<name> |
| EC-002 | `header_scheme = "BEARER"` (wrong case) | E-SPEC-027(a) — only lowercase "bearer" is valid |
| EC-003 | `header_scheme = "cookie:a:b"` (`:` in name) | E-SPEC-027(a) — `:` is not an RFC 6265 tchar character (tchar excludes all RFC 9110 §5.6.2 delimiters) |
| EC-004 | `header_scheme = " bearer"` (leading space) | E-SPEC-027(a) — SP (0x20) is not a tchar character; whitespace-prefixed values are invalid |
| EC-005 | `header_scheme = "bearer"` + `auth_type = "cookie_roundtrip"` | E-SPEC-027(b) — coherence violation |
| EC-006 | `header_scheme = "raw"` + `auth_type = "api_key"` | E-SPEC-027(b) — api_key only permits bearer |
| EC-007 | `header_scheme = "cookie:tok"` + `auth_type = "bearer_static"` | E-SPEC-027(b) — bearer_static only permits bearer/raw |
| EC-008 | `auth_type = "cookie_roundtrip"` + no `header_scheme` key | E-SPEC-027(c) — absence path B; cookie name unknown |
| EC-009 | `auth_type = "oauth2_client_credentials"` + no `header_scheme` key | loads OK (absence path A); bearer default at runtime |
| EC-010 | `header_scheme = "cookie:name"` + `auth_type = "oauth2_client_credentials"` | E-SPEC-027(b) — coherence violation; oauth2 does not permit cookie injection |
| EC-011 | E-SPEC-027(c) message text has no credential value substituted | Verified by AC-010; `{sensor_id}` is config text (safe per AD-017) |
| EC-012 | `build_request()` with `header_scheme = None` (after absence path A spec-load) | Silent bearer default applied; no error returned |
| EC-013 | `execute_step` called with custom AuthProvider tracking get_token vs acquire_token | get_token() invoked; acquire_token() NOT invoked on the normal path |
| EC-009-043 | `header_scheme = "cookie:sid=x; admin"` (`;` and `=` in name — SEC-001 CWE-20/CWE-74 case) | E-SPEC-027(a) — `;` and `=` are not tchar; spec rejected at load time (without fix: synthesizes `Cookie: sid=x; admin={token}`, injecting extra cookie pair) |
| EC-009-044 | `header_scheme = "cookie:a=b"` (`=` in name) | E-SPEC-027(a) — `=` is not tchar; corrupts `name=value` boundary in Cookie header |
| EC-009-045 | `header_scheme = "cookie:a b"` (SP in name) | E-SPEC-027(a) — SP is not tchar; malformed cookie name on wire |
| EC-009-046 | `header_scheme = "cookie:a\nb"` (LF/CTL in name — SEC-002 CWE-390 side effect) | E-SPEC-027(a) — CTL is not tchar; load-time rejection replaces prior deferred `"builder error"` from reqwest at `.send()` time |

## Token Budget Estimate

| Artifact | Estimated Tokens | Notes |
|----------|-----------------|-------|
| This story file | ~10,000 | |
| BC-2.16.009 v1.25 (Rule 9 full text incl. §Entry points sub-section) | ~24,000 | Primary authoring source; grew in v1.24–v1.25 (tchar amendment + Entry points sub-section) |
| BC-2.01.017 v1.10 (P2 dispatch table) | ~8,000 | Dispatch switch spec |
| BC-2.16.014 v1.19 (P9 get_token callers) | ~18,000 | Call-site change spec |
| BC-2.01.016 v1.15 (AuthProvider trait) | ~12,000 | |
| `spec_parser.rs` (field + Rule 9 validation + FetchStep doc) | ~20,000 | Large file |
| `pipeline.rs` (build_request dispatch + call-sites) | ~30,000 | Large file |
| `auth_provider.rs` (get_token addition) | ~8,000 | |
| `error.rs` (E-SPEC-027 variants + E-SPEC-012/013 rewrites) | ~10,000 | |
| ADR-053 (header_scheme field spec, coherence matrix) | ~18,000 | Reference |
| ADR-054 §D4/D11 (get_token wiring, D11 rows) | ~15,000 | Reference |
| VP-153 (proof harness skeleton) | ~10,000 | Re-verification gate |
| Test files (vp153 × 2) | ~12,000 | Harness amendment |
| error-taxonomy.md E-SPEC-027 + E-SPEC-012 + E-SPEC-013 rows | ~8,000 | POL-24 source |
| **Total estimated** | **~201,000** | Approaches one context window |

**Context management guidance:** Load spec_parser.rs + auth_provider.rs + error.rs
in the FIRST sub-burst (field/trait/error changes). Load pipeline.rs in a SECOND
sub-burst (dispatch switch + call-site changes). The Red Gate tests for each tier
can be written before the implementation for that tier using only the BC + story
context, deferring source-file reads to the implementation phase.

## Tasks

### Red Gate tests (to be written by test-writer BEFORE implementation)

- [ ] **RG-001**: `test_rule9_absent_header_scheme_bearer_static_passes` — AC-001 / AC-011
- [ ] **RG-002**: `test_rule9_valid_bearer_accepted` — AC-002
- [ ] **RG-003**: `test_rule9_valid_raw_accepted` — AC-003
- [ ] **RG-004**: `test_rule9_valid_cookie_name_accepted` — AC-004
- [ ] **RG-005**: `test_rule9_invalid_syntax_e_spec_027_template_a` — AC-005
- [ ] **RG-006**: `test_rule9_empty_cookie_name_e_spec_027_template_a` — AC-006
- [ ] **RG-007**: `test_rule9_colon_in_cookie_name_e_spec_027_template_a` — AC-007
- [ ] **RG-008**: `test_rule9_coherence_violation_cookie_roundtrip_bearer_e_spec_027_template_b` — AC-008
- [ ] **RG-009**: `test_rule9_coherence_violation_api_key_raw_e_spec_027_template_b` — AC-009
- [ ] **RG-010**: `test_rule9_absent_cookie_roundtrip_e_spec_027_template_c` — AC-010
- [ ] **RG-011**: `test_build_request_bearer_injects_authorization_bearer_header` — AC-012
- [ ] **RG-012**: `test_build_request_raw_injects_authorization_no_prefix` — AC-013
- [ ] **RG-013**: `test_build_request_cookie_name_injects_cookie_no_authorization` — AC-014
- [ ] **RG-014**: `test_build_request_absent_header_scheme_uses_bearer_default` — AC-014b
- [ ] **RG-015**: `test_auth_provider_get_token_default_delegates_to_acquire_token` — AC-015
- [ ] **RG-016**: `test_execute_impl_calls_get_token_not_acquire_token_on_normal_path` — AC-016 (SAP-3)
- [ ] **RG-017**: `test_issue_request_with_retry_401_path_calls_acquire_token` — AC-016 (401 path)
- [ ] **RG-018**: `test_e_spec_012_display_matches_taxonomy_verbatim` — AC-017
- [ ] **RG-019**: `test_e_spec_013_display_matches_taxonomy_verbatim` — AC-018
- [ ] **RG-020**: `test_rule9_semicolon_injection_in_cookie_name_e_spec_027_template_a` — AC-019 / EC-009-043
  (SEC-001 CWE-20/CWE-74: `"cookie:sid=x; admin"` — `;` fails tchar; spec rejected at load time)
- [ ] **RG-021**: `test_rule9_equals_in_cookie_name_e_spec_027_template_a` — AC-019 / EC-009-044
  (`"cookie:a=b"` — `=` fails tchar; corrupts `name=value` boundary in Cookie header)
- [ ] **RG-022**: `test_rule9_space_in_cookie_name_e_spec_027_template_a` — AC-019 / EC-009-045
  (`"cookie:a b"` — SP fails tchar; malformed cookie name on wire)
- [ ] **RG-023**: `test_rule9_ctl_in_cookie_name_e_spec_027_template_a` — AC-019 / EC-009-046
  (`"cookie:a\nb"` — LF/CTL fails tchar; load-time rejection replaces prior deferred
  reqwest `"builder error"` at `.send()` time — SEC-002 CWE-390 side effect)
- [ ] **RG-024**: `test_add_sensor_spec_mcp_tool_rejects_nontchar_cookie_name_wire_level` — AC-020
  (SAP-3 end-to-end from MCP stdio surface: `add_sensor_spec` with `header_scheme = "cookie:bad;name"`;
  assert `structuredContent.error.code == "E-SPEC-027"` on the serialized JSON envelope;
  wire-shape assertion discipline per CLAUDE.md 2026-07-13; verifies `add_sensor_spec` reaches
  `SpecLoader::parse()` per BC-2.16.009 Rule 9 §Entry points and function coverage)

**Red Gate density check** (BC-5.38.001): **24 failing tests** before implementation begins.
(19 original + RG-020..RG-023 for EC-009-043..046 per SEC-001 spec amendment + RG-024 for
add_sensor_spec MCP surface per SAP-3 / AC-020.)
All non-trivial function bodies use `todo!()` stubs.

### Implementation tasks (to be executed by implementer after Red Gate)

#### Phase A — Error variants and Display rewrites (error.rs)

- [ ] **T-A01** (revised per Q1 architect ruling — wave-a-engine-story-adjudication-Q1-Q5.md):
  Add `ESpec027` variant to the `SpecErrorCode` enum in `crates/prism-core/src/error.rs`.
  **DO NOT** add `InvalidHeaderScheme`, `HeaderSchemeCoherenceViolation`, or
  `HeaderSchemeRequiredForCookieRoundtrip` to `SpecEngineError` in `error.rs` — those
  variants are architecturally incompatible with `SpecLoader::parse()`'s `PrismError`
  return type (the Rule 8 inline-`SpecError` pattern is the correct mechanism).
  `SpecErrorCode` is `#[non_exhaustive]`; adding one variant is additive — no semver break,
  no `EXPECTED=92` gate bump (EXPECTED counts pub struct types, not enum variants):
  ```rust
  /// E-SPEC-027: `header_scheme` field validation failure (Rule 9, BC-2.16.009 v1.25).
  /// Three templates: (a) syntactically invalid value (not bearer/raw/cookie:<tchar-name>);
  /// (b) coherence violation with auth_type; (c) absent header_scheme when
  /// auth_type = "cookie_roundtrip" (absence path B).
  /// See error-taxonomy.md v2.68 E-SPEC-027 for verbatim message templates.
  ESpec027,
  ```
- [ ] **T-A02**: Rewrite `AuthTypeCrossComposition` `#[error(…)]` attribute to match the
  FIRST OR-clause of E-SPEC-012 taxonomy template verbatim per error-taxonomy.md v2.68
  (rename field `provided_value` → `value` or adjust the `#[error(…)]` substitution token).
  Verify template includes `token_exchange` in the Valid values clause. POL-24 atomicity:
  taxonomy (already executed v2.57) + VP-153 prose (already executed v0.21) + this code
  site = the third and final POL-24 copy. Add the Q5 POL-24 comment at the rewrite site
  (see AC-017 for exact comment text). **Note on existing test assertions:** the E-SPEC-012
  Display rewrite DELETES the `E-SPEC-0NN:` prefix currently in the variant's Display.
  Two test files assert the pre-rewrite format and MUST be updated in the same commit:
  `crates/prism-spec-engine/tests/bc_2_01_016_test.rs` (two assertion sites — one for
  `AuthTypeCrossComposition`, one for E-SPEC-013) and
  `crates/prism-spec-engine/tests/crowdstrike_oauth2_plugin_tests.rs`
  (`UnknownAuthPlugin` path — verify which OR-clause applies and update accordingly).
  See Files to MODIFY table.
- [ ] **T-A03**: Rewrite `MultipleCredentialRefs` `#[error(…)]` attribute to match E-SPEC-013
  taxonomy template verbatim per error-taxonomy.md v2.68. Add/rename struct fields: `sensor_id`,
  `count`, `expected`, `auth_type`. Verify `Display` output has parameterized `{expected}`
  and `{auth_type}` (not hardcoded `1`). Update the construction site for this variant:
  there is exactly **one** construction site (in `validate_cross_composition`), which
  already has `expected_ref_count` and `auth_type` in scope — do NOT search for a
  "sweep of all construction sites"; it is a 1-site edit.

#### Phase B — SensorSpec field + Rule 9 validation (spec_parser.rs)

- [ ] **T-B01**: Add `header_scheme: Option<String>` field to `SensorSpec` struct with
  bare `#[serde(default)]` (no `default_header_scheme()` fn — rejected per F-WASE-P48-MED-003).
  `#[serde(default)]` is redundant for a plain `Option<T>` at serde 1.0.228 (confirmed via
  RQ-4 research) but MATCHES the `probe_table` precedent (3 sibling fields use the same
  pattern) and becomes mandatory if `#[serde(deserialize_with = ...)]` is ever added — so
  keep it and add a one-line trap comment:
  ```rust
  /// Cookie name for header injection; e.g. `"cookie:access_token"`. See BC-2.16.009 Rule 9.
  /// TRAP: if `#[serde(deserialize_with = ...)]` is ever added to this field, `#[serde(default)]`
  /// becomes mandatory (absent field → hard error without it). See RQ-4 serde 1.0.228 research.
  #[serde(default)]
  pub header_scheme: Option<String>,
  ```
  Verify `SensorSpec` already carries `#[non_exhaustive]`; add it if absent.
  **Field addition breaks 5 exhaustive `SensorSpec` literal sites — see Phase G (T-G01..G05).**
- [ ] **T-B02** (revised per Q1/Q4 architect rulings): Implement
  `validate_header_scheme(sensor_id: &str, header_scheme: Option<&str>, auth_type: &AuthType) -> Result<(), PrismError>`
  as a pure function in `spec_parser.rs`. Changes from v1.1 spec:
  - Parameter `auth_type: &AuthType` (NOT `&str`) — compiler-enforces coherence matrix
    exhaustiveness for S-ADR054-WAVE-A-001 (Q4 ruling)
  - Return type `Result<(), PrismError>` (NOT `Result<(), SpecEngineError>`) — Rule 8
    precedent; `parse()` returns `PrismError` (Q1 ruling)
  - Errors constructed as: `PrismError::Spec(SpecError { code: SpecErrorCode::ESpec027,
    message: verbatim_template_text, toml_path: Some("sensor.header_scheme".to_string()),
    file_path: None, line_number: None })`

  Logic:
  1. If `None` AND `*auth_type == AuthType::CookieRoundtrip` →
     Err(PrismError::Spec + ESpec027 + template (c) message verbatim)
  2. If `None` → Ok (absence path A)
  3. Parse value: accept `"bearer"`, `"raw"`, `"cookie:<name>"` where `<name>` is
     non-empty and every byte is RFC 6265 tchar (RFC 9110 §5.6.2):
     ```rust
     // RFC 6265 §4.1.1 cookie-name = token; RFC 9110 §5.6.2 tchar
     fn is_valid_cookie_name_tchar(name: &str) -> bool {
         !name.is_empty() && name.bytes().all(|b| matches!(b,
             b'!' | b'#' | b'$' | b'%' | b'&' | b'\'' | b'*' | b'+' |
             b'-' | b'.' | b'^' | b'_' | b'`' | b'|' | b'~' |
             b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z'
         ))
     }
     ```
     This tchar check **subsumes** the old "no colon in name" constraint (`:` is not
     tchar) and additionally rejects `;`, `=`, SP, TAB, CTL chars, high bytes, and
     all RFC 9110 §5.6.2 delimiters (SEC-001 CWE-20/CWE-74 fix — closes SEC-002
     CWE-390 as side effect). Invalid syntax → Err(PrismError::Spec + ESpec027 +
     template (a) message verbatim from error-taxonomy.md v2.68)
  4. Coherence matrix `match auth_type` with exactly **5 exhaustive arms** and **NO
     wildcard** (Q4 rationale: forces `TokenExchange` arm addition in S-ADR054-WAVE-A-001):
     ```rust
     match auth_type {
         AuthType::BearerStatic => { /* bearer, raw allowed; cookie:* disallowed */ }
         AuthType::Oauth2ClientCredentials => { /* bearer, raw allowed; cookie:* disallowed */ }
         AuthType::CookieRoundtrip => { /* only cookie:<name> allowed */ }
         AuthType::CustomViaPlugin => { /* bearer, raw allowed; cookie:* disallowed */ }
         AuthType::ApiKey => { /* only bearer allowed */ }
     }
     ```
     Violation → Err(PrismError::Spec + ESpec027 + template (b) message verbatim)
  5. Otherwise → Ok

  **Q4 SAP-3 defense-in-depth comment:** Any Red Gate test that exercises Rule A
  (`validate_cross_composition`) via direct invocation must carry:
  ```rust
  // SAP-3: Rule A / E-SPEC-012 in validate_cross_composition is defense-in-depth.
  // A TOML spec with an unrecognized auth_type string is rejected by serde (E-SPEC-001)
  // before validate_cross_composition is reached. This test validates the internal
  // defense-in-depth path, not the parser surface.
  ```

  **Q2 VP-059 explicit exclusion note:** VP-059 governs `validate_sensor_spec` (Rules 1–7)
  and is NOT affected by Rule 9 in `parse()`. `verification_properties: [VP-153]` is
  correct; VP-059 is intentionally excluded and must NOT be added to this story. Rule 9
  is fail-fast in `parse()` following the Rule 8 precedent; VP-059's multi-error collection
  semantics apply only to `validate_sensor_spec`. This is stated explicitly so reviewers
  do not re-raise it.
- [ ] **T-B03**: Call `validate_header_scheme` from `SpecLoader::parse()` (or the validation
  pass within spec_parser.rs) after all other field validations. Order: Rule 9 runs after
  auth_type validation succeeds.
- [ ] **T-B04**: Correct `FetchStep` struct doc-comment: replace "use the `Default` impl or
  builder pattern for external construction" with guidance to use `FetchStep::new(...)`.
  Correct `Default for FetchStep` impl doc-comment: remove the struct-literal +
  `..Default::default()` example; replace with FetchStep::new(...) guidance.
  (F-WASE-P9-OBS-003 D11 engine-story row)

#### Phase C — build_request dispatch switch (pipeline.rs)

- [ ] **T-C01**: Switch `build_request()` free function (8 params, no `&self`) from
  auth_type-keyed dispatch to header_scheme-keyed dispatch. `build_request` returns
  `reqwest::RequestBuilder` (NOT `http::Request`; `http` is NOT a dependency of
  `prism-spec-engine`). New dispatch logic (legal Rust — `header_scheme: Option<&str>`):
  ```rust
  match header_scheme {
      Some("bearer") | None => {
          // "bearer" = explicit bearer injection
          // None = absence path A (non-cookie_roundtrip spec loads with silent bearer default)
          builder.header(AUTHORIZATION, format!("Bearer {}", token.as_str()))
      }
      Some("raw") => {
          builder.header(AUTHORIZATION, token.as_str())
      }
      Some(s) if s.starts_with("cookie:") => {
          // ADR-053 §D2: cookie name is declared in the sensor spec's header_scheme field
          // (e.g., "cookie:access_token" for Cyberint per ADR-031 DTU fidelity principle).
          // No hardcoded cookie name — the name is extracted from header_scheme[7..].
          let name = &s[7..];
          builder.header("Cookie", format!("{}={}", name, token.as_str()))
      }
      Some(_) => {
          // Rule 9 rejects all non-bearer/raw/cookie:<name> header_scheme values at
          // spec-load time; this arm is defense-in-depth (unreachable in valid specs).
          // Fallback to bearer to produce a sensible (not panicking) request.
          builder.header(AUTHORIZATION, format!("Bearer {}", token.as_str()))
      }
  }
  ```
  Remove the old `match auth_type` switch entirely. Remove the stale ADR-031 §D4
  comment block referencing hardcoded `access_token` name per TD-VSDD-091
  (anti-volatile-pin — the dispatch mechanism no longer exists).

  **Q3 (ADR-031 cite):** ADR-031 §D4's `access_token` cookie-name fidelity SURVIVES
  as a DTU fidelity principle; enforcement moves to the TOML spec's `header_scheme`
  field. ADR-053 supersedes the enforcement MECHANISM (hardcoded dispatch → spec-declared
  `header_scheme`), not the fidelity requirement. The correct `header_scheme` for
  Cyberint remains `"cookie:access_token"`, enforced by `S-WAVE-A-CYBERINT-SPEC-001`
  (co-land constraint). No engine-side cookie-name allowlist (POL-36 / INV-014-001).
- [ ] **T-C02**: Update `build_request()` call sites — pass `spec.header_scheme.as_deref()`
  instead of `&spec.auth_type` wherever dispatch was previously keyed on auth_type.
  TD-VSDD-060 sibling-site sweep: `grep -rn "build_request(" crates/prism-spec-engine/`
  to find all call sites.

#### Phase D — execute_impl + execute_step call-site changes (pipeline.rs)

- [ ] **T-D01**: In `execute_impl`, change the eager-acquisition call in the
  `let mut bearer_token = match auth_provider.` block (before the `'steps:` loop) from
  `auth_provider.acquire_token(spec, &context.client_id)` to
  `auth_provider.get_token(spec, &context.client_id)`.
- [ ] **T-D02**: In `execute_step`, change the eager-acquisition call in the
  `let bearer_token = match auth_provider.` block (top of function) from
  `auth_provider.acquire_token(spec, &context.client_id)` to
  `auth_provider.get_token(spec, &context.client_id)`.
- [ ] **T-D03**: Verify `issue_request_with_retry` 401 arm STILL calls `acquire_token()`
  (no change — intentional force-refresh per BC-2.16.014 P5/P6). Add an inline comment
  at the call site: `// BC-2.16.014 P5/P6: 401-retry path always force-refreshes; get_token() would return stale cached token`

#### Phase E — AuthProvider trait get_token() method (auth_provider.rs)

- [ ] **T-E01**: Add `get_token` as a default method to the `AuthProvider` trait:
  ```rust
  fn get_token<'a>(
      &'a self,
      spec: &'a SensorSpec,
      client_id: &'a OrgSlug,
  ) -> Pin<Box<dyn Future<Output = Result<AuthToken, SpecEngineError>> + Send + 'a>> {
      self.acquire_token(spec, client_id)  // default: delegates to force-refresh
  }
  ```
  The method signature must be identical to `acquire_token()` for object-safety.
  **RQ-2 confirmation:** Fully dyn-compatible at edition 2024. Defaulted bodies are
  irrelevant to dyn-compatibility (Rust Reference). Edition 2024 RFC 3498 is RPIT-only —
  `Pin<Box<dyn Future>>` is a concrete named type, not `impl Trait`; it is unaffected.
  The object-safety→dyn-compatibility rename (Rust 1.83) was terminology-only.
  **DO NOT add `where Self: Sized`** — that would make the method explicitly non-dispatchable
  via `Arc<dyn AuthProvider>`, which is the primary usage site (RQ-2).
- [ ] **T-E02**: Verify all 8 existing AuthProvider implementors compile without code
  changes (they inherit the default). TD-VSDD-060 sibling-site sweep:
  `grep -rn "impl .*AuthProvider for" crates/` — the fully-qualified form
  `impl prism_spec_engine::AuthProvider for` (used at the `BearerStaticAuthProvider`
  site in `crates/prism-bin/src/spec_driven_adapter.rs`) would be missed by
  `grep -rn "impl AuthProvider" crates/`.

#### Phase G — SensorSpec exhaustive literal fixups (broken by header_scheme field addition)

Adding `header_scheme: Option<String>` to `SensorSpec` breaks every exhaustive struct
literal in the codebase. `SensorSpec` carries `#[non_exhaustive]` for external users, but
internal `..` spread syntax requires `Default::default()` to be explicit — internal test
literals that enumerate all fields WILL fail to compile. Fix the 5 known sites atomically
with the field addition (T-B01). **DO NOT extend `SensorSpec::new()` arity** — the
`probe_table` precedent (see S-SPEC-ENV-VAR-001 merge notes) keeps `new()` minimal;
add the field via `..Default::default()` spread or direct assignment.

- [ ] **T-G01**: `SensorSpec::default()` explicit-field construction site in
  `crates/prism-spec-engine/src/spec_parser.rs` (inline test module) — add
  `header_scheme: None` or switch to `..Default::default()` spread.
- [ ] **T-G02**: `SensorSpec::new(...)` constructor body in
  `crates/prism-spec-engine/src/spec_parser.rs` — add `header_scheme: None` field.
  **Do NOT add `header_scheme` as a parameter** — `new()` uses positional minimal
  construction per `probe_table` precedent.
- [ ] **T-G03**: First inline test `SensorSpec { ... }` literal in
  `crates/prism-spec-engine/src/pipeline.rs` (search for exhaustive literal by all-field
  enumeration) — add `header_scheme: None`.
- [ ] **T-G04**: Second inline test `SensorSpec { ... }` literal in
  `crates/prism-spec-engine/src/pipeline.rs` — add `header_scheme: None`.
- [ ] **T-G05**: `SensorSpec { ... }` literal in
  `crates/prism-spec-engine/src/proofs/spec_validator.rs` — add `header_scheme: None`.

TD-VSDD-060 sibling-site sweep: `grep -rn "SensorSpec {" crates/` to confirm these 5
are the complete set and no sixth site was missed.

#### Phase F — VP-153 harness amendments

- [ ] **T-F01**: In `crates/prism-spec-engine/tests/vp153_sensorauth_cross_composition.rs`,
  correct test-inventory `//!` doc table row 4: rename
  `prop_rule_b_single_or_zero_credential_refs_accepted` →
  `prop_rule_b_single_credential_ref_accepted` in the docstring table.
  (F-WASE-P29-OBS-001 D11 row)
---

> **ORCHESTRATOR RULING (2026-07-24) — T-F02, T-F03, T-F04 ROUTED TO S-ADR054-WAVE-A-001**
>
> Adding `"token_exchange"` to the as-built `VALID_AUTH_TYPES: &[&str]` constant in
> `crates/prism-spec-engine/src/spec_parser.rs`, and adding the token_exchange strategy
> arms to the VP-153 proptest harness files, are **OUT OF SCOPE for S-WAVE-A-ENGINE-001**
> and belong to the ADR-054 implementation story (S-ADR054-WAVE-A-001).
> (Note: the harness files have no existing `[PLANNED]` marker comments in their source;
> this is arm ADDITION work, not marker removal.)
>
> Grounding:
> 1. ADR-054 §D11 groups BOTH the `AuthType::TokenExchange` enum variant AND the
>    `"token_exchange"` addition to `VALID_AUTH_TYPES` in a SINGLE manifest row whose
>    "Triggered by" cell is **D1** — and D1 is ADR-054's decision, not ADR-053's.
>    The manifest itself assigns the work.
> 2. ADR-054 §D7's "Coherence matrix scope boundary" exists specifically to prevent a
>    forward reference in which validation state predates its enum variant. Accepting
>    `"token_exchange"` in `validate_cross_composition` while `AuthType::TokenExchange`
>    does not yet exist is exactly that forward reference.
>
> This is a **scope-boundary routing**, NOT a tech-debt deferral. Do NOT add a
> tech-debt-register entry. The `[PLANNED — ADR-054 D1 engine story]` markers in both
> harness files MUST REMAIN as-is until S-ADR054-WAVE-A-001 lands. Do NOT activate them
> in this story.
>
> **Former T-F02** (add `Just("token_exchange")` arm to `arb_valid_auth_type()` in FILE 1
> (`vp153_sensorauth_cross_composition.rs`) — the harness currently has 5 `Just(...)` arms
> for the 5 as-built auth types; activating the 6th arm is work ADDITION, not un-commenting
> existing code) → assigned to S-ADR054-WAVE-A-001.
>
> **Former T-F03** (add `"token_exchange"` arms to `arb_matching_auth_type()` and
> `arb_mismatched_auth_type_pair()` in `crates/prism-bin/tests/vp153_rule_c_shaped_probe.rs`
> — again, this is arm ADDITION to existing match/strategy expressions, not removing markers)
> → assigned to S-ADR054-WAVE-A-001.
>
> **Former T-F04** (add `"token_exchange"` to `VALID_AUTH_TYPES` constant in both harness
> files and in `spec_parser.rs`) → assigned to S-ADR054-WAVE-A-001.

### Merge gate (required before PR can merge)

- [ ] **MERGE-GATE-VP153-PARTIAL (this story)**: Run the VP-153 proptest suite against
  `prism-spec-engine` ONLY, WITHOUT activating the token_exchange strategy arms
  (T-F02/T-F03/T-F04 are not done here):
  ```
  cargo nextest run -p prism-spec-engine -E 'binary(vp153_sensorauth_cross_composition)'
  ```
  **NOTE on nextest filter:** `-E 'test(vp153)'` selects ZERO tests because no VP-153
  test function name contains the string "vp153". The correct filter is
  `-E 'binary(vp153_sensorauth_cross_composition)'` which selects the binary by name
  (the test binary is named after the harness file). All proptest function names in that
  binary will run.

  **What this gate verifies for this story:**
  - All currently-active VP-153 proptests in FILE 1 (`vp153_sensorauth_cross_composition.rs`)
    pass after T-A02's E-SPEC-012 Display rewrite — in particular,
    `prop_rule_a_invalid_auth_type_rejected_with_e_spec_012` MUST pass with the updated
    Display string that now includes `"token_exchange"` in the "Valid values:" clause
    (the proptest implicitly validates this per AC-017).
  - T-A03's E-SPEC-013 Display rewrite does not regress any Rule B proptests.
  - T-F01's fn-name doc fix does not regress the test harness.
  - No baseline regression from any Phase A–E implementation tasks.
  Confirm `lifecycle_status: active` (already active; confirm no regression).

  **What this gate does NOT verify (ROUTED to S-ADR054-WAVE-A-001 as MERGE-GATE-VP153-FULL):**
  - The token_exchange arms in `arb_valid_auth_type()` (FILE 1) are NOT added here
    (former T-F02; arm ADDITION, not marker removal, belongs to S-ADR054-WAVE-A-001).
  - The token_exchange arms in `arb_matching_auth_type()` and
    `arb_mismatched_auth_type_pair()` in `vp153_rule_c_shaped_probe.rs` (FILE 2) are NOT
    added here (former T-F03; arm ADDITION belongs to S-ADR054-WAVE-A-001).
  - `VALID_AUTH_TYPES` does NOT include `"token_exchange"` after this story.
  - Adding token_exchange arms increases the number of active strategy arms, not the
    number of proptest functions — `prism-bin` VP-153 probe is NOT run here.

  (F-WASE-P4-OBS-002: partial scope per ORCHESTRATOR RULING (2026-07-24); full VP-153
  re-run with token_exchange strategy arms active in both harness files is
  MERGE-GATE-VP153-FULL in S-ADR054-WAVE-A-001)

- [ ] **MERGE-GATE-CYBERINT**: Verify cyberint.sensor.toml co-land. Once this story lands,
  `cyberint.sensor.toml` (which has `auth_type = "cookie_roundtrip"` with absent
  `header_scheme`) will be REJECTED at spec-load by E-SPEC-027(c). The Cyberint spec
  migration story (S-WAVE-A-CYBERINT-SPEC-001) MUST be merged in the SAME release batch
  as this story, or this story merges AFTER the Cyberint spec migration. Confirm with
  orchestrator before merging.

- [ ] **MERGE-GATE-JUST-CHECK**: `just check` passes with ZERO new warnings **relative
  to merge base** and all existing tests continue to pass. The baseline is NOT
  warning-free (known pre-existing: `unused import: PluginAuthProvider` in
  `crowdstrike_oauth2_plugin_tests.rs`); only warnings INTRODUCED by this story's
  changes are blocking. Run `just check` on the merge-base branch first if uncertain
  which warnings are pre-existing.

## Previous Story Intelligence

N/A — this is the first story in the Wave-A engine prerequisite chain. No predecessor
story in this epic exists.

Prior art from closely related stories:
- `S-SPEC-ENV-VAR-001` (merged PR #165): pattern for adding a new validated field to
  `SensorSpec` with spec-load rejection. Follow the same integration-point pattern:
  field addition in spec_parser.rs, validation call in the existing validation pass,
  error variant in error.rs, unit tests in spec_parser's inline `#[cfg(test)] mod tests`.
- `S-SPEC-HTTP-METHOD-VALIDATION-001` (merged PR #172): pattern for adding a pure
  validation function alongside an existing validation pass. The E-SPEC-025 error
  variant structure is a close analog for the E-SPEC-027 variant structure.
- `PLUGIN-MIGRATION-001-D`: pattern for changing call sites in pipeline.rs (acquire_token →
  alternatives). Lessons: use `just iter prism-spec-engine` for inner-loop iteration;
  full `just check` only once at end of fix-burst per TDD inner-loop discipline.

## Architecture Compliance Rules

These rules are extracted from the authoritative architecture section files and ADRs.
Violations are P1 findings in adversarial review.

1. **POL-36 / INV-014-001 — No sensor-name-conditional logic in the engine**: The new
   `validate_header_scheme` function and the updated `build_request()` dispatch MUST NOT
   contain any `if sensor_id == "cyberint"` or equivalent sensor-name conditions. The
   dispatch is keyed on `header_scheme` value only.

2. **ADR-050 / rustls-tls**: No new `reqwest::Client` is introduced in this story. The
   existing clients in `pipeline.rs` and `auth_provider.rs` already comply with
   `default-features = false, features = ["rustls-tls"]`. Do not regress.

3. **ADR-022 / Arc-DI**: No new stub-construct pattern (`Arc::new(Something::placeholder())`)
   in any production boot path. This story adds only pure validation functions, trait methods,
   and call-site changes — no new Arc-injected types.

4. **TD-VSDD-091 / Anti-volatile-pin**: Cite function names and behavioral anchors in
   comments, NOT `file.rs:NNN` line numbers. The D11 rows use behavioral anchors
   ("`let mut bearer_token = match auth_provider.` block in execute_impl") — follow
   this convention in all inline comments and story references.

5. **AD-017 / AI-opaque credentials**: E-SPEC-027 template substitutions use `{sensor_id}`
   (config text) and `{value}` / `{auth_type}` (config text) — never credential values.
   The `build_request()` call receives the token opaquely (does not log or echo it).

6. **BC-5.39.001 / Spec-first gate**: Do NOT amend any spec artifact (BC, ADR,
   error-taxonomy) from the story. The spec perimeter was reopened under human
   authorization for FB45; product-owner and architect completed their spec-amendment
   legs before this story was dispatched. The BC-5.39.001 streak is 0/3 post-amendment.
   If a spec defect is discovered during implementation, STOP and report to orchestrator
   per CLAUDE.md Companion Principle rule 2 — do not self-amend.

7. **`#[non_exhaustive]` gate — EXPECTED stays at 92**: No new pub types are introduced
   by this story (only a new field on SensorSpec, new variants on SpecEngineError, a
   new method on AuthProvider). `scripts/check-non-exhaustive.sh EXPECTED=92` must pass.
   The bump to 95 belongs to S-ADR054-WAVE-A-001 (AuthAcquisitionConfig/CachedAuthToken/
   ExpiryMode introduced by DeclarativeHttpAuthProvider).

8. **S-7.01 gate (merge dependency encoding)**: This story's `blocks` array lists the
   ADR-054 story and the two co-land stories. The ADR-054 story writer MUST encode
   `depends_on: [S-WAVE-A-ENGINE-001]` in its frontmatter. This is the explicit
   merge-dependency encoding required by ADR-054 §D7.

9. **ORCHESTRATOR RULING (2026-07-24) — token_exchange scope boundary**: The
   `VALID_AUTH_TYPES: &[&str]` constant in `crates/prism-spec-engine/src/spec_parser.rs`
   MUST NOT include `"token_exchange"` after this story merges. The token_exchange
   strategy arms in BOTH VP-153 harness files MUST NOT be added here — those arm
   additions (formerly T-F02 for `arb_valid_auth_type()` in FILE 1, and T-F03 for
   `arb_matching_auth_type()` / `arb_mismatched_auth_type_pair()` in FILE 2) belong
   to S-ADR054-WAVE-A-001. Grounding: ADR-054 §D11 groups the `AuthType::TokenExchange`
   enum variant AND the `VALID_AUTH_TYPES` string update in a single manifest row triggered
   by ADR-054's D1 decision; ADR-054 §D7's coherence matrix scope boundary forbids
   accepting `"token_exchange"` in validation before the enum variant exists. This is a
   **scope-boundary routing** — implementer must not override it. Do NOT add a
   tech-debt-register entry. The work is assigned to S-ADR054-WAVE-A-001.

10. **Q6 forward constraint (ADR-052 §D5 scope exclusion)**: The implementer of
    `S-ADR054-WAVE-A-001` MUST add the following verbatim scope-exclusion comment at
    the `expiry_str.parse::<DateTime<FixedOffset>>()` site in
    `DeclarativeHttpAuthProvider::acquire_token()`:
    ```rust
    // ADR-052 §D5 SCOPE EXCLUSION: D5 governs sites that produce Arrow Timestamp(µs,UTC)
    // output for the data-plane (query engine). Auth-token expiry parsing produces a u64
    // Unix-seconds TTL for the control-plane — outside D5's domain. Do NOT replace this
    // call with parse_datetime_to_micros(): that function (a) outputs the wrong type for
    // TTL arithmetic and (b) does not handle Armis's space-separated datetime strings
    // (RFC 3339 strict parse fails on "2026-07-24 14:30:00 UTC").
    ```
    This forward constraint is encoded here (in S-WAVE-A-ENGINE-001) so it survives
    story decomposition and reaches S-ADR054-WAVE-A-001's implementer. The `acquire_token`
    site is the S-ADR054-WAVE-A-001 implementer's responsibility, not this story's
    (this story does not touch `DeclarativeHttpAuthProvider`).

11. **TD-DECOMP-EPIC-001 / pipeline.rs growth gate**: `pipeline.rs` is at approximately
    4,578 production lines (plus ~12,100 inline test lines), already registered under
    TD-DECOMP-EPIC-001. Per CLAUDE.md §File size — any PR that grows `pipeline.rs` past
    1,500 net production lines MUST include a decomposition rationale in the PR description
    citing a TD-DECOMP-EPIC-001 anchor story. This story's changes (dispatch switch +
    two call-site edits + one doc-comment) are modest net-additions; no growth gate
    trigger is expected. But the test-writer and implementer MUST verify final line count
    before declaring done — if inline tests in `#[cfg(test)] mod tests` push the file past
    1,500 NEW net production lines, include the rationale.

## Library and Framework Requirements

| Library | Version / Source | Purpose |
|---------|-----------------|---------|
| `thiserror` | as pinned in workspace `Cargo.toml` | `#[derive(Error)]` for SpecEngineError new variants |
| `serde` | as pinned in workspace `Cargo.toml` | `#[serde(default)]` on `SensorSpec::header_scheme` |
| `proptest` | `1.x` as pinned in workspace `Cargo.toml` | VP-153 harness amendment (existing dep) |
| `reqwest` | `default-features = false, features = ["rustls-tls"]` | No new client; existing clients must remain ADR-050 compliant |
| `tokio` | as pinned in workspace `Cargo.toml` | Async trait method (`get_token` is `async fn` via Pin<Box>) |

**No new dependencies added by this story.** All libraries above are existing
workspace deps. Do not introduce new crate dependencies.

## File Structure Requirements

### Files to CREATE

None. This story modifies existing files only.

### Files to MODIFY

| File | Change Summary |
|------|----------------|
| `crates/prism-core/src/error.rs` | Add `ESpec027` variant to the `SpecErrorCode` enum (T-A01). DO NOT add `InvalidHeaderScheme`, `HeaderSchemeCoherenceViolation`, or `HeaderSchemeRequiredForCookieRoundtrip` to `SpecEngineError` — those variants are architecturally incompatible with `SpecLoader::parse()`'s `PrismError` return type (Q1 ruling). `SpecErrorCode` is `#[non_exhaustive]`; adding a variant is additive and does NOT bump the `EXPECTED=92` gate. |
| `crates/prism-spec-engine/src/spec_parser.rs` | Add `header_scheme: Option<String>` field to `SensorSpec` (with `#[serde(default)]`). Add `validate_header_scheme()` pure function. Call it from the existing validation pass. Correct `FetchStep` struct + `Default` impl doc-comments. |
| `crates/prism-spec-engine/src/error.rs` | Rewrite `AuthTypeCrossComposition` `#[error(…)]` to match E-SPEC-012 taxonomy template verbatim (T-A02; rename field `provided_value` → `value` or adjust token). Add Q5 POL-24 comment. Rewrite `MultipleCredentialRefs` struct fields + `#[error(…)]` to match E-SPEC-013 taxonomy template verbatim (T-A03; rename/add fields `sensor_id`, `count`, `expected`, `auth_type`). Update the single construction site in `validate_cross_composition`. |
| `crates/prism-spec-engine/src/pipeline.rs` | Switch `build_request()` dispatch from auth_type-keyed to header_scheme-keyed. Change `execute_impl` and `execute_step` call sites from `acquire_token()` to `get_token()`. Add doc comment at `issue_request_with_retry` 401 path noting intentional `acquire_token()` retention. Note: file is ~4,578 production lines (registered TD-DECOMP-EPIC-001); any PR growing it past 1,500 net new production lines requires decomposition rationale in PR description per Architecture Compliance Rule 11. |
| `crates/prism-spec-engine/src/auth_provider.rs` | Add `get_token()` as a default method on the `AuthProvider` trait. Signature matches `acquire_token()` for object-safety. Default body: `self.acquire_token(spec, client_id)`. |
| `crates/prism-spec-engine/tests/vp153_sensorauth_cross_composition.rs` | Correct fn-name in `//!` doc table row 4 only (T-F01). Token_exchange strategy arm ADDITION is OUT OF SCOPE — ROUTED TO S-ADR054-WAVE-A-001 per ORCHESTRATOR RULING (2026-07-24). Do NOT add token_exchange arms here. |
| `crates/prism-spec-engine/tests/bc_2_01_016_test.rs` | Update Display string assertions for `AuthTypeCrossComposition` (E-SPEC-012 Display rewrite deletes the `E-SPEC-0NN:` prefix; two assertion sites require updating) and for `MultipleCredentialRefs` (E-SPEC-013 Display rewrite adds `{auth_type}` and `{expected}` fields). Both sites must be updated atomically with T-A02/T-A03. |
| `crates/prism-spec-engine/tests/crowdstrike_oauth2_plugin_tests.rs` | Update the `UnknownAuthPlugin` path assertion for the E-SPEC-012 Display rewrite (the second OR-clause — `UnknownAuthPlugin` clause — must also be verified and updated if the `E-SPEC-0NN:` prefix is present there). Atomic with T-A02. |
| `crates/prism-spec-engine/src/proofs/spec_validator.rs` | Update exhaustive `SensorSpec { ... }` literal to include `header_scheme: None` (T-G05). Compilation fails on this file the moment `header_scheme` is added to `SensorSpec` (if the literal enumerates all fields). |

### Files NOT to modify

| File | Reason |
|------|--------|
| `.factory/specs/prd-supplements/error-taxonomy.md` | Frozen at v2.68 for this story. E-SPEC-027 row (including backtick charset correction) completed in v2.68 by product-owner leg of FB45. E-SPEC-012 row already executed in v2.57. Spec is CONVERGED — do NOT amend. |
| `.factory/specs/behavioral-contracts/BC-2.16.009-spec-file-validation.md` | Frozen. Rule 9 fully specified. |
| `.factory/specs/behavioral-contracts/BC-2.16.014-*.md` | Frozen. P9 call-site change is code, not spec amendment. |
| `.factory/specs/architecture/decisions/ADR-054-*.md` | Frozen. D11 rows document what code to change; reading only. |
| `crates/prism-sensors/specs/crowdstrike.sensor.toml` | CrowdStrike TOML migration belongs to S-ADR054-WAVE-A-001. |
| `crates/prism-sensors/specs/cyberint.sensor.toml` | Cyberint TOML migration belongs to S-WAVE-A-CYBERINT-SPEC-001 (must co-land). |
| `crates/prism-bin/tests/vp153_rule_c_shaped_probe.rs` | Token_exchange strategy arm ADDITION for `arb_matching_auth_type()` and `arb_mismatched_auth_type_pair()` ROUTED TO S-ADR054-WAVE-A-001 per ORCHESTRATOR RULING (2026-07-24); no changes from this story. |
| `scripts/check-non-exhaustive.sh` | EXPECTED stays at 92; no change needed this story. |

## Forbidden Dependencies

The following modules/packages MUST NOT appear as new dependencies in
`crates/prism-spec-engine/Cargo.toml` after this story:

- `prism-query` (build-time enforcement: `tests/external/perimeter-violation/` compile-fail gate)
- `prism-mcp` (spec-engine must not depend on MCP layer)
- `native-tls` / `default-tls` / `native-tls-alpn` / `native-tls-vendored` (ADR-050; macOS Keychain overhead + MITM risk)

## UX Screen References

N/A — this story has no UX surface. All changes are internal to the spec-load/execution
pipeline. The only user-visible output is structured error messages (E-SPEC-027) which
are surfaced via MCP tool error responses when `prism start` rejects a malformed TOML
spec.

## Dependency Graph Edges

```
S-WAVE-A-ENGINE-001 (this story)
  depends_on: []
  blocks:
    → S-ADR054-WAVE-A-001      (DeclarativeHttpAuthProvider + TokenExchange + Rule 10)
    → S-WAVE-A-CYBERINT-SPEC-001  (cyberint.sensor.toml header_scheme = "cookie:..." migration)
    → S-WAVE-A-ARMIS-REMEDIATION-001  (Armis spec uses token_exchange; blocked by ADR-054 story)
```

**Sequencing constraint (ADR-054 §D7 verbatim):** The ADR-054 implementation stories
MUST merge AFTER this story. The story-writer for S-ADR054-WAVE-A-001 MUST encode
`depends_on: [S-WAVE-A-ENGINE-001]` in that story's frontmatter.

**Co-land constraint (Cyberint):** S-WAVE-A-CYBERINT-SPEC-001 and this story must
enter the same develop batch. After this story lands, cyberint.sensor.toml is rejected
by E-SPEC-027(c) until the Cyberint spec migration adds `header_scheme = "cookie:<name>"`.

## ADR-054 §D11 Engine-Story Rows Pulled Into This Story

The following §D11 amendment manifest rows are anchored to "engine story" and are
explicitly in scope for this story's implementation tasks:

| D11 Row | Finding | Task | Status |
|---------|---------|------|--------|
| validate_cross_composition fn-name docstring row-4 | F-WASE-P29-OBS-001 | T-F01 | PENDING |
| VP-153 proof re-run gate — partial | F-WASE-P4-OBS-002 | MERGE-GATE-VP153-PARTIAL (this story: prism-spec-engine only, existing active arms, no token_exchange activation; full 8-proptest run → MERGE-GATE-VP153-FULL in S-ADR054-WAVE-A-001 per ORCHESTRATOR RULING) | PENDING |
| AuthProvider trait `get_token()` addition | F-WASE-P7-HIGH-001 | T-E01/E02 | PENDING |
| execute_impl eager-path call-site (acquire→get_token) | F-WASE-P7-HIGH-001 | T-D01 | PENDING |
| execute_step eager-path call-site (acquire→get_token) | F-WASE-P7-HIGH-001 | T-D02 | PENDING |
| issue_request_with_retry 401 arm no-change note | F-WASE-P7-HIGH-001 | T-D03 | PENDING |
| `AuthTypeCrossComposition` Display rewrite (E-SPEC-012 POL-24) | F-WASE-P16-OBS-003 | T-A02 | PENDING |
| `MultipleCredentialRefs` Display rewrite (E-SPEC-013 POL-24) | F-WASE-P31-MED-001 | T-A03 | PENDING |
| `FetchStep` struct + `Default` impl doc-comment correction | F-WASE-P9-OBS-003 | T-B04 | PENDING |

**Count of §D11 engine-story rows pulled in: 9 rows** (6 code changes, 2 doc-hygiene,
1 merge gate — the VP-153 proof re-run row is scoped to partial/existing-arms only per
ORCHESTRATOR RULING; full proof re-run is MERGE-GATE-VP153-FULL in S-ADR054-WAVE-A-001).

Rows explicitly NOT in scope for this story (belong to S-ADR054-WAVE-A-001):
- Non-exhaustive gate EXPECTED bump 92→95 (AuthAcquisitionConfig/CachedAuthToken/ExpiryMode)
- AuthType::TokenExchange enum variant addition to spec_parser.rs
- auth_acquisition: Option<AuthAcquisitionConfig> field in SensorSpec
- DeclarativeHttpAuthProvider implementation (auth/declarative.rs)
- CrowdStrike TOML spec migration + crowdstrike-oauth2 plugin retirement
- Rule 10 validation + E-SPEC-028 (all 8 templates)
- Token_exchange coherence matrix row (blocked on AuthType::TokenExchange variant)
- `"token_exchange"` addition to VALID_AUTH_TYPES constant in spec_parser.rs
  (ORCHESTRATOR RULING 2026-07-24 — former T-F04; ADR-054 §D11 D1-triggered row)
- VP-153 token_exchange strategy arm ADDITION in vp153_sensorauth_cross_composition.rs
  (ORCHESTRATOR RULING 2026-07-24 — former T-F02; arm addition to `arb_valid_auth_type()`;
  no [PLANNED] markers exist in that file; blocked on VALID_AUTH_TYPES update)
- VP-153 token_exchange strategy arm ADDITION in vp153_rule_c_shaped_probe.rs
  (ORCHESTRATOR RULING 2026-07-24 — former T-F03; arm addition to `arb_matching_auth_type()`
  and `arb_mismatched_auth_type_pair()`; blocked on VALID_AUTH_TYPES update)
- Full VP-153 8-proptest pass (MERGE-GATE-VP153-FULL — all arms active, both harness files)
- Doc-hygiene sweep of crowdstrike-oauth2 references in preserved infrastructure files

## Version History

| Version | Date | Change Summary |
|---------|------|----------------|
| 2.1 | 2026-07-24 | FB45 story-writer leg — closes F-WASE-P61-HIGH-001 (charset + version propagation), F-WASE-P61-HIGH-002 (add_sensor_spec injection vector coverage), F-WASE-P61-MED-002 (SEC-001 Red Gate tests gain AC coverage), F-WASE-P61-MED-003 (stale frozen-perimeter claims), F-WASE-P61-LOW-003 (discontinuous EC ID scheme), F-WASE-P61-LOW-004 (AC-015 compile-time postcondition). CHARSET: AC-005 + AC-007 tchar set corrected to 15-char RFC 9110 §5.6.2 order (backtick U+0060 restored, tail corrected to `^ _ ` \| ~`); double-backtick code spans per upstream convention; error-taxonomy.md pinned at v2.68. BC VERSIONS: BC-2.16.009 v1.24→v1.25, BC-2.16.014 v1.18→v1.19, BC-2.01.016 v1.14→v1.15 in §Behavioral Contracts table and §Token Budget Estimate table; BC-2.16.009 Token Budget row updated v1.23→v1.25 with token estimate bumped to ~24,000 (reflects v1.24–v1.25 Rule 9 growth). NEW ACs: AC-019 (non-tchar char rejection for SEC-001 injection classes; RG-020..023 re-anchored to AC-019 in addition to EC-009-043..046); AC-020 (add_sensor_spec MCP wire-level coverage per SAP-3 §Entry points). NEW RGT: RG-024 (add_sensor_spec MCP surface, wire-shape assertion on `structuredContent.error.code`). Red Gate density updated 23→24. AC COUNT: 19→21. AC-015 rewritten to specify delegation-counter runtime assertion; 8-implementor compile claim moved to build-gate note. EDGE CASES: EC-043..046 renamed to BC-canonical IDs EC-009-043..EC-009-046 to resolve anchor ambiguity (option a — BC references, not story-local). ARCH RULE 6: stale "FROZEN (3/3 CLEAN at passes 58/59/60)" premise replaced by accurate description of spec perimeter status post-FB45. FILES NOT TO MODIFY: error-taxonomy.md version reference updated to v2.68. Frontmatter points note updated 19→21 ACs. |
| 2.0 | 2026-07-24 | Q1–Q6 architect rulings applied (wave-a-engine-story-adjudication-Q1-Q5): Q1 — ESpec027 added to SpecErrorCode in prism-core (NOT three SpecEngineError variants); error construction uses inline PrismError::Spec following Rule 8 precedent; crates_touched expanded to include prism-core. Q2 — Rule 9 is fail-fast in parse() per Rule 8 precedent; VP-059 explicitly excluded (stated in T-B02). Q3 — ADR-031 fidelity survives via spec-declared header_scheme; stale hardcoded-name comment removed from build_request() per TD-VSDD-091; ADR-031 cite added to T-C01. Q4 — validate_header_scheme parameter changed to `&AuthType` (NOT `&str`); SAP-3 defense-in-depth comment requirement added for Rule A path. Q5 — E-SPEC-012 Display includes `token_exchange` in Valid values clause now; POL-24 comment added at rewrite site (AC-017). Q6(C) — ADR-052 §D5 scope-exclusion forward constraint added as Architecture Compliance Rule 10. SEC-001 spec amendment (BC-2.16.009 v1.24): Rule 9 tchar charset replaces old no-colon rule; EC-043..046 added to Edge Cases; RG-020..023 added to Red Gate tests; Red Gate density updated 19→23. BLOCKER resolutions: BLOCKER 1 — MERGE-GATE-VP153-PARTIAL nextest filter corrected to binary(vp153_sensorauth_cross_composition); BLOCKER 2 — build_request return type corrected to reqwest::RequestBuilder throughout; BLOCKER 3 — three stale SpecEngineError variants replaced by ESpec027 in prism-core; BLOCKER 4 — test files bc_2_01_016_test and crowdstrike_oauth2_plugin_tests added to Files to MODIFY; BLOCKER 5 — ORCHESTRATOR RULING text corrected (token_exchange arm ADDITION, not marker removal; no [PLANNED] markers exist in harness code). Phase G added (T-G01..T-G05): five exhaustive SensorSpec literal sites broken by header_scheme field addition. T-E01 RQ-2 dyn-compatibility confirmation note added. T-E02 corrected to grep `impl .*AuthProvider for` and count 8 implementors (BearerStaticAuthProvider in prism-bin). T-E02 MERGE-GATE-JUST-CHECK reworded: ZERO new warnings relative to merge base (baseline has pre-existing unused-import warning). Files to MODIFY table rewritten: stale three-variant error.rs entry replaced with ESpec027-in-prism-core entry; three additional rows added (bc_2_01_016_test, crowdstrike_oauth2_plugin_tests, spec_validator). Files NOT to MODIFY vp153_rule_c_shaped_probe.rs entry corrected to say arm ADDITION. Architecture Mapping [PLANNED] language corrected. |
| 1.1 | 2026-07-24 | Encode ORCHESTRATOR RULING (2026-07-24): token_exchange `VALID_AUTH_TYPES` addition and VP-153 token_exchange arm activation (former T-F02/T-F03/T-F04) ROUTED to S-ADR054-WAVE-A-001. Dual-path conditional in Phase F removed — story states ONE path: `VALID_AUTH_TYPES` is NOT touched here. `MERGE-GATE-VP153` narrowed to `MERGE-GATE-VP153-PARTIAL` (prism-spec-engine only, existing active arms, no token_exchange activation). Proof re-run split: satisfiable portion (Rule A/B/C active arms + E-SPEC-012 Display validation) stays here; full 8-proptest run with token_exchange arms is `MERGE-GATE-VP153-FULL` in S-ADR054-WAVE-A-001. Architecture Compliance Rule 9 added. `crates_touched` updated (prism-bin removed). Files to Modify table updated (vp153_rule_c_shaped_probe.rs moved to Files NOT to modify). D11 exclusions list expanded with three new routed items. AC count unchanged (19 ACs). D11 row count unchanged (9 rows, VP-153 row scoped to partial). |
| 1.0 | 2026-07-24 | Initial story creation. |
