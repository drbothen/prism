---
document_type: story
story_id: S-WAVE-A-ENGINE-001
title: "prism-spec-engine: SensorSpec header_scheme field — Rule 9 validation, build_request header-injection dispatch, and AuthProvider::get_token trait method (Wave-A ADR-053 standalone engine prerequisite)"
wave: wave-a
epic_id: E-SPEC-ENGINE
priority: P1
status: draft
version: "3.2"
updated: "2026-08-13"
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
verification_properties: [VP-153, VP-160]
# VP-153: SensorAuth Runtime Cross-Composition Prevention.
# Engine-story gate (PARTIAL per ORCHESTRATOR RULING 2026-07-24): run prism-spec-engine
# VP-153 suite WITHOUT adding token_exchange strategy arms; verify existing active proptests
# pass including Rule A with updated E-SPEC-012 Display. Token_exchange arm ADDITION
# (MERGE-GATE-VP153-FULL) belongs to S-ADR054-WAVE-A-001.
# See §Tasks MERGE-GATE-VP153-PARTIAL and Architecture Compliance Rule 9.
# VP-160: Rule 9 Cookie-Name Charset Totality and Injection Rejection (Kani proof).
# Anchor justification (POL-5): §Tasks T-B02 step 3 authors the full body of
# `is_valid_cookie_name_tchar` — the exact symbol VP-160 targets as its proof vehicle.
# §Architecture Mapping assigns `SpecLoader::validate_header_scheme` (Rule 9) to
# `crates/prism-spec-engine/src/spec_parser.rs`. Under the VSDD anchor-story rule,
# the anchor story is the one that builds the test vehicle; this story builds it.
depends_on: []
# This story is the FIRST in the ADR-054 §D7 sequencing chain.
# No product-story hard dependencies — can enter Wave 1 of wave-a scheduling.
blocks:
  - S-ADR054-WAVE-A-001             # TBD — ADR-054 declarative auth implementation story (DeclarativeHttpAuthProvider + TokenExchange + Rule 10)
  - S-WAVE-A-CYBERINT-SPEC-001      # Cyberint dual-surface spec migration; implementation ordering only (needs header_scheme grammar from this story)
  - S-WAVE-A-ARMIS-REMEDIATION-001  # TBD — Armis token-exchange remediation story
# blocks anchor justifications:
#   S-ADR054-WAVE-A-001: ADR-054 §D7 explicit merge dependency — "implementation stories
#     MUST merge AFTER the ADR-053 standalone Wave-A engine story." DeclarativeHttpAuthProvider
#     overrides get_token() added here; Rule 10 builds on Rule 9 registered here.
#   S-WAVE-A-CYBERINT-SPEC-001: The dual-surface migration adds new sensor spec files that
#     declare header_scheme; it needs the SensorSpec::header_scheme grammar and Rule 9
#     that this story introduces. This is an implementation ordering dependency only —
#     CYBERINT-SPEC-001 explicitly does NOT need to co-land atomically with ENGINE-001
#     (per S-WAVE-A-CYBERINT-SPEC-001 §Scheduling Note: "This story does NOT need to
#     co-land atomically with S-WAVE-A-ENGINE-001"). The boot-failure hazard from the
#     existing cyberint.sensor.toml (absence path B → E-SPEC-027(c)) is handled atomically
#     by S-WAVE-A-CYBERINT-PATCH-001, which co-lands with ENGINE-001 per its own
#     MERGE-GATE-ENGINE-001. CYBERINT-SPEC-001 follows on its own schedule after
#     ENGINE-001 + PATCH-001 are both live.
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
#   - TDD test coverage across all 27 ACs (unit + integration + MCP surface): 1.5 pt
#   Total: 8 points
estimated_days: 2.0
risk: MEDIUM
# Risk justification:
#   MEDIUM because this story touches four production files across the
#   spec-load/execution boundary (spec_parser, pipeline, auth_provider, error)
#   plus two test files in two crates, all coordinated as a single atomic
#   wave-a delivery. The individual changes are well-specified but the
#   CYBERINT-PATCH-001 co-land (boot-hazard prevention) requires scheduling discipline.
#   No novel algorithm design — risk is coordination and test coverage breadth.
assumption_validations: []
risk_mitigations: []
---

# S-WAVE-A-ENGINE-001: Wave-A ADR-053 Standalone Engine Story — SensorSpec `header_scheme` Field, Rule 9 Validation, `build_request` Header-Injection Dispatch, and `AuthProvider::get_token` Trait Method

## Authority

**ADR-053 §D2** (accepted 2026-07-22, D-1943) is the authoritative design decision for
this story. ADR-053 §D2 establishes the `header_scheme` TOML field on `SensorSpec`, the
`header_scheme`-keyed dispatch in `build_request()`, and the `auth_type × header_scheme`
coherence matrix validated at spec-load time by Rule 9. ADR-053 §D5 manifest formally
confirms BC-2.01.017 §P2 dispatch mechanism as a spec amendment target implemented by
this story.

Read ADR-053 §D2 and §D5 in full before implementing:
`.factory/specs/architecture/decisions/ADR-053-wave-a-sensor-fidelity-remediation-openapi-grounding-armis-token-exchange-cyberint-dual-surface.md`

This story is the ADR-053 §D2 standalone engine prerequisite: all `header_scheme`-related
scope (field addition, Rule 9 validation, `build_request()` dispatch switch, and the
`AuthProvider::get_token()` prerequisite method for S-ADR054-WAVE-A-001) is governed
by ADR-053 §D2.

---

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
| BC-2.16.009 | Spec File Validation — Schema Validation, Variable Reference Resolution, OCSF Field Validation | v1.29 | Rule 9 only — `header_scheme` value validation, 3 E-SPEC-027 templates, coherence matrix (5 existing variants), absence paths A/B. Cookie name constraint: RFC 6265 tchar per SEC-001 CWE-20/CWE-74 fix (15-char set with backtick, RFC 9110 §5.6.2 order). §Entry points and function coverage sub-section asserts `add_sensor_spec` MUST reach `SpecLoader::parse()` for Rule 9 coverage on the sole active injection vector. New in v1.26: EC-009-047..EC-009-051 (overlong value echo cap CWE-400, CTL-byte `\xNN` hex escaping CWE-117, TAB in cookie name, high-byte byte-level predicate divergence probe, cookie name > 128 codepoints CWE-390). v1.27 (attribution-only): §Integration function sentence split — Rule 9 scoped to this story; Rule 10 scoped to S-ADR054-WAVE-A-001; §Traceability Stories row added S-ADR054-WAVE-A-001. v1.28 (FB56 MED-003): template (a) `≤128 codepoints` clause added to E-SPEC-027 template (a); §Invariants scoped to per-function collect-all semantics (`validate_sensor_spec` collect-all; `parse()` fail-fast boundary stated). No Rule 9 behavioral change. Rule 10 (`[auth_acquisition]`) is OUT OF SCOPE. |
| BC-2.01.017 | StaticCookieAuthProvider Contract — No-Login-Roundtrip Cookie Injection | v1.10 | P2 dispatch table: build_request() switches from auth_type-keyed to header_scheme-keyed dispatch per ADR-053 D2. INV-COOKIE-004: no Authorization header for cookie injection. |
| BC-2.16.014 | Declarative Auth Acquisition Token Lifecycle | v1.21 | P9 only — `get_token()` default method added to `AuthProvider` trait (default body delegates to `acquire_token()`); execute_impl and execute_step call sites change from `acquire_token()` to `get_token()`. issue_request_with_retry 401 path remains `acquire_token()`. v1.20 adds INV-014-007 ADR-050 §D5/§D6 note (DeclarativeHttpAuthProvider inherits UA + http2 via build_http_client_with_custom_timeout delegation — implemented by DEFECT-ADAPTER-TLS-XDOME-LIVE-001); no scope change for this story. v1.21 (D-2119): INV-014-007 §Invariants body corrected — ADR-050 v2.2 pin per POL-23 (was v2.0); no scope change for this story. |
| BC-2.01.016 | SensorAuth Open Trait — Plugin-Implementable Auth Contract (No Sealed Marker) | v1.15 | Rule A/B E-SPEC-012/013 Display alignment (AC-017/AC-018); foundation for VP-153 re-verification gate. |

## Acceptance Criteria

### Tier 1 — SensorSpec::header_scheme Field Deserialization

**AC-001 — absent field deserializes to None (absence path A, ALL four non-cookie auth_types)**
A sensor TOML spec that contains no `header_scheme` key deserializes `SensorSpec::header_scheme`
to `None`. For ALL `auth_type` values OTHER than `cookie_roundtrip` — specifically
`bearer_static`, `oauth2_client_credentials`, `api_key`, and `custom_via_plugin` — `None`
passes spec-load validation silently (absence path A — runtime `build_request()` injects
`Authorization: Bearer` by default). Spec load returns `Ok` for all four variants.
The universal quantifier "for ALL non-cookie_roundtrip auth_types" is exercised across
all four arms by: RG-001 (`bearer_static`), RG-037 (`oauth2_client_credentials` — also
EC-014), RG-038 (`api_key`), RG-039 (`custom_via_plugin`). This matches the coherence
matrix's five-arm exhaustive `match auth_type` with no wildcard (Q4 ruling) — leaving
any arm's absence path untested would undercut that design.
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
matches E-SPEC-027 template (a) VERBATIM per error-taxonomy.md v2.70:
``"sensor '{sensor_id}' has invalid header_scheme = '{value}'. Valid values: bearer, raw, cookie:<name> (non-empty name, ≤128 codepoints, RFC 6265 token characters only: A-Z a-z 0-9 ! # $ % & ' * + - . ^ _ ` | ~)"``
where `{sensor_id}` and `{value}` are substituted with the spec's values.
(traces to BC-2.16.009 Rule 9 postcondition 2a: syntactically invalid value → E-SPEC-027(a))

**AC-006 — "cookie:" with empty name → E-SPEC-027 template (a)**
A spec with `header_scheme = "cookie:"` (the `cookie:` prefix present but cookie name
is the empty string) is rejected at spec-load time. `spec_err.message` (from
`PrismError::Spec`) matches E-SPEC-027 template (a). The "non-empty name, ≤128 codepoints"
clause covers the empty-name case: `is_valid_cookie_name_tchar` fails the `!name.is_empty()`
predicate for an empty string. The `bytes().all(...)` loop is vacuously true on an empty
sequence, so the non-empty predicate is the operative rejection, not the charset check.
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

**SAP-3 reachability note — RG-011..RG-014:** These four unit tests exercise `build_request()`
directly from within `pipeline.rs`'s inline `#[cfg(test)] mod tests`. Per SAP-3 rule 2,
unit tests of private functions are defense-in-depth only — they establish internal
correctness but do not demonstrate the arm is reachable from the product surface.
AC-027 / RG-040 (Tier 7 below) provides the primary SAP-3 end-to-end reachability test:
it drives `PipelineExecutor::execute` with `header_scheme = "bearer"` from the public
surface and asserts the `Authorization: Bearer` header at the wire level.

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

**AC-017 — E-SPEC-012 Display matches error-taxonomy.md v2.70 verbatim (AuthTypeCrossComposition clause)**
The `SpecEngineError::AuthTypeCrossComposition` variant's `Display` output (emitted
via `thiserror` `#[error(…)]`) matches the FIRST OR-clause of the E-SPEC-012
`message_template` in error-taxonomy.md v2.70 BYTE-FOR-BYTE (the first clause covers
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

**AC-018 — E-SPEC-013 Display matches error-taxonomy.md v2.70 verbatim**
The `SpecEngineError::MultipleCredentialRefs` variant's `Display` output matches the
E-SPEC-013 `message_template` in error-taxonomy.md v2.70:
`"auth method for sensor '{sensor_id}' declares {count} credential_refs; exactly {expected} required for auth_type '{auth_type}'"`
The variant struct gains (or renames to) fields `sensor_id`, `count`, `expected`,
`auth_type` to satisfy this template. The as-built "hardcoded 1 with no {auth_type}
or {expected} parameters" divergence is corrected atomically with the struct-field
rename. A unit test asserts the emitted string contains `{auth_type}` substituted
with the actual auth_type value.
(traces to BC-2.01.016 Rule B postcondition: E-SPEC-013 emitted per POL-24;
F-WASE-P31-MED-001 engine-story D11 row)

### Tier 6 — Security Injection Vector Coverage (SEC-001 / SAP-3)

**AC-019 — non-tchar characters in cookie name (`;`, `=`, SP, TAB, CTL) → E-SPEC-027 template (a) (SEC-001 CWE-20/CWE-74 closure)**
A spec with a `header_scheme` value whose cookie name contains any non-tchar character
is rejected at spec-load time by Rule 9 tchar validation. The security-motivating cases
are `;` (semicolon), `=` (equals), SP (space, 0x20), TAB (0x09), and CTL bytes
(0x00–0x1F including LF/CR/NUL).
`spec_err.message` (extracted from `PrismError::Spec`) matches E-SPEC-027 template (a)
VERBATIM for each. The `;` case — `header_scheme = "cookie:sid=x; admin"` — is the
SEC-001 primary injection vector: without Rule 9, the synthesized Cookie header would
be `"sid=x; admin={token}"`, remapping the auth credential to the attacker-controlled
key `admin`. Load-time tchar rejection at spec-load eliminates this vector. The CTL case
eliminates the SEC-002 / CWE-390 deferred `"builder error"` from reqwest as a side
effect — load-time rejection replaces the prior opaque runtime failure. TAB (0x09) is
a CTL byte and is equally rejected; RG-030 provides the dedicated TAB test vector
serving as the EC-009-049 implementation-correctness probe (a byte-level tchar predicate
that uses `char.is_whitespace()` would also reject TAB but for the wrong reason;
`char.is_alphanumeric()` would NOT reject TAB — the byte-level `matches!` predicate is
the distinguishing mechanism).
RG-020..RG-023 drive `;`, `=`, SP, and CTL (LF) injection-class inputs through
`SpecLoader::parse()` and assert the exact E-SPEC-027 template (a) message. RG-030
drives the TAB (0x09) case as the dedicated EC-009-049 test vector.
(traces to BC-2.16.009 Rule 9 postcondition 2a: non-tchar character in cookie name →
E-SPEC-027(a); EC-009-043, EC-009-044, EC-009-045, EC-009-046, EC-009-049)

**AC-020 — `add_sensor_spec` call path reaches `SpecLoader::parse()`, applying Rule 9 (SAP-3 end-to-end, prism-spec-engine API boundary)**
Calling `add_sensor_spec(org_slug, toml_content)` in `prism-spec-engine` with `toml_content`
containing `header_scheme = "cookie:bad;name"` (`;` is not a tchar character) returns
`Ok(AddSensorSpecResult::ValidationFailed { errors })` where `errors` contains at least one
entry whose text matches the E-SPEC-027 template (a) VERBATIM string. RG-024 drives this
at the `prism-spec-engine` public API boundary — NOT a direct `SpecLoader::parse()` call,
and NOT a `prism-mcp` integration test. This verifies that the `add_sensor_spec` handler
reaches `SpecLoader::parse()` (per BC-2.16.009 Rule 9 §Entry points and function coverage),
closing the CWE-20/CWE-74 injection vector on the handler path. An implementation that
calls only `validate_sensor_spec()` in the handler bypasses Rules 8/9/10 per BC-2.16.009
§Security requirement and leaves the injection vector open. **S-WAVE-A-MCP-001 carries
the original wire-level intent** (BC-2.10.007 envelope mapping, `structuredContent.error.code`
assertion at the MCP stdio surface); that story depends on this one. Per ADR-053 §D6
(Option B), the current multi-error prose-string shape from `add_sensor_spec` is a gap,
not an intended contract — do NOT assert `structuredContent.error.code` in this story.
`prism-mcp` remains outside `crates_touched` and Files to MODIFY.
(traces to BC-2.16.009 Rule 9 §Entry points and function coverage: `add_sensor_spec`
MUST reach `SpecLoader::parse()` for Rule 9 to close the CWE-20/CWE-74 injection vector;
SAP-3: end-to-end coverage at prism-spec-engine public API, not synthetic internal
invocation; BC-2.16.008 v1.6: `add_sensor_spec` uses same validation pipeline as startup)

**AC-021 — cookie name with all 15 RFC 6265 tchar special characters is ACCEPTED (HIGH-003 tchar positive coverage)**
A spec with ``header_scheme = "cookie:a!#$%&'*+-.^_`|~9Z"`` (cookie name
``a!#$%&'*+-.^_`|~9Z`` exercises all 15 tchar special characters: `! # $ % & ' * + - . ^ _`
plus backtick, `| ~`, plus a digit `9` and uppercase letter `Z`) and
`auth_type = "cookie_roundtrip"` passes Rule 9 validation. `SpecLoader::parse()` returns
`Ok`. This test makes the permissive half of the tchar character set load-bearing in the
test suite: an implementation of `is_valid_cookie_name_tchar` that used only `[A-Za-z0-9_]`
(omitting all 15 special tchar characters) would FAIL this test. RG-025 lives in
`spec_parser.rs`'s inline `#[cfg(test)] mod tests`.
(traces to BC-2.16.009 Rule 9 postcondition: syntactically valid cookie name with all 15
RFC 6265 tchar special characters is accepted; HIGH-003 tchar coverage gap)

**AC-022 — high byte (U+00E9 'é' = 0xC3 0xA9) in cookie name is REJECTED (EC-009-050: byte-level predicate divergence probe)**
A spec with `header_scheme = "cookie:caf\xc3\xa9"` (cookie name contains U+00E9 'é'
as multi-byte UTF-8 bytes 0xC3 0xA9) is rejected at spec-load time. `spec_err.message`
(extracted from `PrismError::Spec`) matches E-SPEC-027 template (a). This test distinguishes
a correct byte-level tchar predicate (`name.bytes().all(|b| matches!(...))`) from a buggy
unicode-aware implementation using `chars()` + `char::is_alphanumeric()`, which would treat
'é' as alphabetic and accept it incorrectly. High bytes (0x80–0xFF) are outside the
byte-level tchar set and must be rejected. RG-026 lives in `spec_parser.rs`'s inline
`#[cfg(test)] mod tests`.
(traces to BC-2.16.009 Rule 9 postcondition 2a: high byte fails byte-level tchar check;
EC-009-050)

**AC-023 — cookie name exceeding 128 codepoints is REJECTED by the Rule 9 length guard (EC-009-051: CWE-390 opaque-HTTP-431 prevention)**
A spec with `header_scheme = "cookie:" + "a" × 129` (cookie name of 129 `a` characters,
all of which pass the tchar character-class check) is rejected at spec-load time.
`spec_err.message` (extracted from `PrismError::Spec`) matches E-SPEC-027 template (a).
This test verifies the 128-codepoint length bound is a SEPARATE condition from the tchar
character-class check: the 129-`a` input would PASS a tchar-only implementation; only the
`name.chars().count() > 128` (or equivalent) length guard triggers the rejection. Without
this guard, a >128-codepoint all-tchar cookie name would be injected verbatim into
`Cookie: <name>={token}`, producing an opaque HTTP 431 Request Header Too Large from the
sensor API (CWE-390 deferred-opaque-failure mode). RG-027 lives in `spec_parser.rs`'s
inline `#[cfg(test)] mod tests`. SAP-3: test drives `SpecLoader::parse()` with raw TOML
string — parser surface, not synthetic-struct invocation.
(traces to BC-2.16.009 Rule 9 postcondition: cookie name >128 codepoints → E-SPEC-027(a);
EC-009-051)

**AC-024 — overlong `header_scheme` value is capped at 64 codepoints in the E-SPEC-027 template (a) message (EC-009-047: CWE-400 unbounded-echo mitigation)**
A spec with a `header_scheme` value consisting of 65 `X` characters (not a valid
`"bearer"`, `"raw"`, or `"cookie:<name>"` form, so template (a) fires) is rejected at
spec-load time. The FULL COMPOSED `spec_err.message` string (extracted from
`PrismError::Spec` and asserted via `assert_eq!(spec_err.message, expected_verbatim)`)
contains the `{value}` substitution equal to the first 64 `X` characters — NOT 65. The
assertion is on the complete message string as emitted, not merely on a `value` component
field (SID-2 composed-output assertion discipline). For `header_scheme` values ≤64
codepoints, the error message `{value}` substitution is byte-identical to the original
(POL-24 preserved for common-case inputs). The cap is applied via
`truncate_at_char_boundary(&header_scheme_value, 64)` before `{value}` substitution in
the template (a) format string. RG-028 lives in `spec_parser.rs`'s inline
`#[cfg(test)] mod tests`. SAP-3: test drives `SpecLoader::parse()` with raw TOML — parser
surface entry point.
(traces to BC-2.16.009 Rule 9 postcondition: overlong {value} capped at 64 codepoints in
template (a) error message via `truncate_at_char_boundary`; EC-009-047)

**AC-025 — CTL bytes in `header_scheme` are `\xNN`-escaped in the E-SPEC-027 template (a) message (EC-009-048: CWE-117 log-injection mitigation)**
A spec with `header_scheme = "cookie:a\x0Ab"` (cookie name contains LF byte 0x0A, which
is not a tchar byte, triggering template (a)) is rejected at spec-load time.
The FULL COMPOSED `spec_err.message` string (extracted from `PrismError::Spec`) MUST
satisfy both assertions (SID-2 composed-output assertion discipline):
(a) `spec_err.message.contains("\\x0A")` — the message contains the literal four-character
    ASCII sequence backslash-x-0-A (NOT a raw 0x0A byte), confirming CWE-117 log-injection
    escaping is applied to the `{value}` substitution;
(b) `!spec_err.message.as_bytes().contains(&0x0A)` — the message bytes contain no raw LF
    byte, confirming the raw control character was not embedded verbatim.
The escaping rule: after applying the 64-codepoint cap, iterate over the UTF-8 byte
sequence and replace each byte `b` where `(b as u8) <= 0x1F || (b as u8) == 0x7F` with
the four-character ASCII sequence `\xNN` (literal backslash, lowercase `x`, two
uppercase hex digits for `b`; examples: LF 0x0A → `\x0A`, CR 0x0D → `\x0D`,
TAB 0x09 → `\x09`). For values with no CTL bytes the escaping is a no-op (POL-24
preserved). RG-029 lives in `spec_parser.rs`'s inline `#[cfg(test)] mod tests`.
SAP-3: test drives `SpecLoader::parse()` with raw TOML string — parser surface entry point.
(traces to BC-2.16.009 Rule 9 postcondition: CTL bytes in {value} replaced with `\xNN`
in template (a) error message; EC-009-048)

**AC-026 — Rule 9 fail-fast boundary: spec with Rule 9 violation + `[auth_acquisition]` block produces exactly ONE error from `SpecLoader::parse()` (BC-2.16.009 §Invariants observable consequence)**
A TOML spec containing BOTH (1) an invalid `header_scheme` value that triggers Rule 9
(e.g., `header_scheme = "garbage"`) AND (2) an `[auth_acquisition]` block present in the
TOML body is submitted to `SpecLoader::parse()`. The call returns exactly ONE error —
the Rule 9 `ESpec027` error — confirming that Rule 9's `Err` return prevents execution
from reaching Rule 10. Assertion pattern:

    let err = SpecLoader::parse(spec_with_rule9_violation_and_auth_acquisition_block).unwrap_err();
    let prism_core::PrismError::Spec(ref spec_err) = err else {
        panic!("expected Spec error, got {:?}", err)
    };
    assert_eq!(spec_err.code, SpecErrorCode::ESpec027);
    // Exactly one error returned: Rule 9 fired (fail-fast); Rule 10 did not execute.

This test is forward-compatible: after S-ADR054-WAVE-A-001 implements Rule 10, a spec
with an invalid `header_scheme` still causes Rule 9 to return `Err` from `parse()` first,
preventing Rule 10 from executing. The `[auth_acquisition]` block is present in the TOML
but ignored at this story's scope; it triggers no separate error because Rule 10 has not
yet been implemented. After Rule 10 lands, the test continues to hold because the
fail-fast semantics ensure Rule 10 is never reached when Rule 9 fails.
RG-031 lives in `spec_parser.rs`'s inline `#[cfg(test)] mod tests`. SAP-3: drives
`SpecLoader::parse()` with raw TOML string — parser surface, not synthetic-struct invocation.
(traces to BC-2.16.009 §Invariants — per-function collect-all semantics with cross-rule
fail-fast boundary at the `parse()`/`validate_sensor_spec()` function boundary;
F-WASE-P64-MED-004 fail-fast boundary observable consequence)

### Tier 7 — SAP-3 End-to-End Pipeline Integration Test for Header Dispatch (F-CVB-P67-OBS-001)

**AC-027 — SAP-3: `PipelineExecutor::execute` with `header_scheme = "bearer"` injects `Authorization: Bearer <token>` at wire level (end-to-end reachability)**
A full integration test constructs a `SensorSpec` with `header_scheme = "bearer"` and
`auth_type = "bearer_static"`, starts a wiremock server that asserts on the incoming
`Authorization` header value `Bearer test-token-abc` (the known token returned by
`MockAuthProvider`), calls
`PipelineExecutor::execute(&spec, table, &context, &http_client, &auth_provider)`, and
asserts the wiremock expectation was satisfied (confirming the outbound HTTP request
contained `Authorization: Bearer test-token-abc`). The wiremock `header()` matcher
asserts on the bytes the HTTP server receives — this is the wire-level assertion required
by the 2026-07-13 wire-shape discipline: it tests the exact bytes that the sensor API
server sees, not pre-send Rust structures.

RG-011..RG-014 in `pipeline.rs`'s inline test module are defense-in-depth (SAP-3 rule 2);
this test is the PRIMARY end-to-end reachability test for the bearer dispatch arm. Test file:
`crates/prism-spec-engine/tests/bc_2_01_017_static_cookie_auth_provider.rs` (add to existing
file — same BC, same executor surface, following the cookie-dispatch test at AC-007 in that
file). If file length is a concern the test-writer may create
`crates/prism-spec-engine/tests/bc_2_01_017_header_scheme_bearer_e2e.rs` instead.
(traces to BC-2.01.017 P2 postcondition: `header_scheme = "bearer"` → `Authorization: Bearer <token>`;
SAP-3 end-to-end reachability from `PipelineExecutor::execute` surface; closes F-CVB-P67-OBS-001)

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
gate requires no new symbol registration (no new pub types; only a new field on SensorSpec,
new variants on the existing SpecEngineError enum). The registration bump for
AuthAcquisitionConfig/CachedAuthToken/ExpiryMode belongs to S-ADR054-WAVE-A-001.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `header_scheme = ""` (empty string) | E-SPEC-027(a) — empty string is neither bearer, raw, nor valid cookie:<name> |
| EC-002 | `header_scheme = "BEARER"` (wrong case) | E-SPEC-027(a) — only lowercase "bearer" is valid |
| EC-009-033 | `header_scheme = "cookie:a:b"` (`:` in name) | E-SPEC-027(a) — `:` is not an RFC 6265 tchar character (tchar excludes all RFC 9110 §5.6.2 delimiters) |
| EC-004 | `header_scheme = " bearer"` (leading space) | E-SPEC-027(a) — SP (0x20) is not a tchar character; whitespace-prefixed values are invalid |
| EC-009-031 | `header_scheme = "bearer"` + `auth_type = "cookie_roundtrip"` | E-SPEC-027(b) — coherence violation |
| EC-006 | `header_scheme = "raw"` + `auth_type = "api_key"` | E-SPEC-027(b) — api_key only permits bearer |
| EC-007 | `header_scheme = "cookie:tok"` + `auth_type = "bearer_static"` | E-SPEC-027(b) — bearer_static only permits bearer/raw |
| EC-009-042 | `auth_type = "cookie_roundtrip"` + no `header_scheme` key | E-SPEC-027(c) — absence path B; cookie name unknown |
| EC-014 | `auth_type = "oauth2_client_credentials"` + no `header_scheme` key | loads OK (absence path A); bearer default at runtime |
| EC-010 | `header_scheme = "cookie:name"` + `auth_type = "oauth2_client_credentials"` | E-SPEC-027(b) — coherence violation; oauth2 does not permit cookie injection |
| EC-011 | E-SPEC-027(c) message text has no credential value substituted | Verified by AC-010; `{sensor_id}` is config text (safe per AD-017) |
| EC-012 | `build_request()` with `header_scheme = None` (after absence path A spec-load) | Silent bearer default applied; no error returned |
| EC-013 | `execute_step` called with custom AuthProvider tracking get_token vs acquire_token | get_token() invoked; acquire_token() NOT invoked on the normal path |
| EC-009-043 | `header_scheme = "cookie:sid=x; admin"` (`;` and `=` in name — SEC-001 CWE-20/CWE-74 case) | E-SPEC-027(a) — `;` and `=` are not tchar; spec rejected at load time (without fix: synthesizes `Cookie: sid=x; admin={token}`, injecting extra cookie pair) |
| EC-009-044 | `header_scheme = "cookie:a=b"` (`=` in name) | E-SPEC-027(a) — `=` is not tchar; corrupts `name=value` boundary in Cookie header |
| EC-009-045 | `header_scheme = "cookie:a b"` (SP in name) | E-SPEC-027(a) — SP is not tchar; malformed cookie name on wire |
| EC-009-046 | `header_scheme = "cookie:a\nb"` (LF/CTL in name — SEC-002 CWE-390 side effect) | E-SPEC-027(a) — CTL is not tchar; load-time rejection replaces prior deferred `"builder error"` from reqwest at `.send()` time |
| EC-009-047 | `header_scheme` value > 64 codepoints (CWE-400 echo cap) | E-SPEC-027(a) — `{value}` substitution in the error message is capped at 64 codepoints via `truncate_at_char_boundary` (plain slice, no appended marker; identical helper to Rule 7's method echo cap) to prevent log flooding from untrusted input |
| EC-009-048 | CTL byte (e.g., 0x01) in cookie name — message escaping (CWE-117) | E-SPEC-027(a) — CTL bytes in the echoed `{value}` are `\xNN`-escaped (uppercase hex) in the error message, preventing log injection via raw control characters |
| EC-009-049 | TAB (0x09) in cookie name | E-SPEC-027(a) — TAB is a CTL byte (0x09), not tchar; spec rejected at load time (same mechanism as EC-009-046 LF case; TAB escapes as `\x09` per EC-009-048) |
| EC-009-050 | High byte (U+00E9 'é' = 0xC3 0xA9) in cookie name | E-SPEC-027(a) — high bytes (0x80–0xFF) are outside the byte-level tchar set; the byte-level predicate (`name.bytes().all(...)`) rejects them; a buggy `char::is_alphanumeric()` implementation would wrongly accept 'é' |
| EC-009-051 | Cookie name length > 128 codepoints (CWE-390) | E-SPEC-027(a) — cookie names exceeding 128 codepoints are rejected by Rule 9 length guard; the echoed value is additionally capped per EC-009-047 |

## Token Budget Estimate

| Artifact | Estimated Tokens | Notes |
|----------|-----------------|-------|
| This story file | ~10,000 | |
| BC-2.16.009 v1.29 (Rule 9 full text incl. §Entry points sub-section + EC-009-047..051) | ~25,000 | Primary authoring source; grew in v1.24–v1.25 (tchar amendment + Entry points sub-section), v1.26 (new ECs); v1.27 attribution-only; v1.28 template (a) length clause + §Invariants scoping; v1.29 EC-009-049 escaped-value statement explicit |
| BC-2.01.017 v1.10 (P2 dispatch table) | ~8,000 | Dispatch switch spec |
| BC-2.16.014 v1.21 (P9 get_token callers) | ~18,000 | Call-site change spec; v1.20 adds ADR-050 §D5/§D6 INV note; v1.21 corrects INV-014-007 ADR-050 v2.2 pin per POL-23 (no scope change for this story) |
| BC-2.01.016 v1.15 (AuthProvider trait) | ~12,000 | |
| `spec_parser.rs` (field + Rule 9 validation + FetchStep doc) | ~20,000 | Large file |
| `pipeline.rs` (build_request dispatch + call-sites) | ~30,000 | Large file |
| `auth_provider.rs` (get_token addition) | ~8,000 | |
| `error.rs` (E-SPEC-027 variants + E-SPEC-012/013 rewrites) | ~10,000 | |
| ADR-053 (header_scheme field spec, coherence matrix) | ~18,000 | Reference |
| ADR-054 §D4/D11 (get_token wiring, D11 rows) | ~15,000 | Reference |
| VP-153 (proof harness skeleton) | ~10,000 | Re-verification gate |
| VP-160 (cookie-name charset Kani proof) | ~8,000 | Proof vehicle harness; `is_valid_cookie_name_tchar` is the target symbol |
| Test files (vp153 × 2) | ~12,000 | Harness amendment |
| error-taxonomy.md E-SPEC-027 + E-SPEC-012 + E-SPEC-013 rows | ~8,000 | POL-24 source |
| **Total estimated** | **~209,000** | Approaches one context window |

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
- [ ] **RG-024**: `test_add_sensor_spec_api_rejects_nontchar_cookie_name_rule9_path` — AC-020
  (SAP-3 end-to-end at prism-spec-engine public API: `add_sensor_spec(org_slug, toml_content)`
  with `header_scheme = "cookie:bad;name"`; assert `Ok(AddSensorSpecResult::ValidationFailed
  { errors })` with an entry matching E-SPEC-027(a) verbatim text; verifies handler reaches
  `SpecLoader::parse()` per BC-2.16.009 Rule 9 §Entry points; does NOT assert
  `structuredContent.error.code` — S-WAVE-A-MCP-001 carries the wire-level MCP assertion
  deferred by ADR-053 §D6 Option B)
- [ ] **RG-025**: `test_rule9_all_tchar_special_chars_cookie_name_accepted` — AC-021
  (HIGH-003: cookie name ``a!#$%&'*+-.^_`|~9Z`` exercises all 15 RFC 6265 tchar special
  chars; any `is_valid_cookie_name_tchar` using only `[A-Za-z0-9_]` would fail this test)
- [ ] **RG-026**: `test_rule9_high_byte_in_cookie_name_e_spec_027_template_a` — AC-022
  (EC-009-050: cookie name `"caf\xc3\xa9"` — U+00E9 high-byte bytes fail the byte-level
  tchar check; catches buggy `chars()`/`is_alphanumeric()` implementations that accept 'é')
- [ ] **RG-027**: `test_rule9_overlong_cookie_name_129_codepoints_rejected` — AC-023 / EC-009-051
  (cookie name of 129 `a` characters — all tchar-valid; only the `chars().count() > 128`
  length guard triggers E-SPEC-027(a) rejection; CWE-390 bound verification;
  `spec_parser.rs` inline test module; SAP-3: drives `SpecLoader::parse()` with raw TOML)
- [ ] **RG-028**: `test_rule9_overlong_value_echo_cap_64_codepoints_in_error_message` — AC-024 / EC-009-047
  (65-char all-`X` `header_scheme` — not a valid form, template (a) fires; asserts FULL
  composed `spec_err.message == expected_verbatim` where `{value}` = first 64 `X`s, NOT
  65; SID-2 composed-output assertion on the complete message string; verifies
  `truncate_at_char_boundary` is applied before `{value}` substitution; `spec_parser.rs`
  inline test module; SAP-3: drives `SpecLoader::parse()` with raw TOML)
- [ ] **RG-029**: `test_rule9_ctl_byte_in_value_hex_escaped_in_error_message` — AC-025 / EC-009-048
  (`header_scheme = "cookie:a\x0Ab"` — LF byte 0x0A in cookie name triggers template (a);
  SID-2 composed-output assertions on the FULL message bytes:
  (a) `assert!(spec_err.message.contains("\\x0A"))` — literal four-char sequence
      backslash-x-0-A present in the message (escaped form);
  (b) `assert!(!spec_err.message.as_bytes().contains(&0x0A))` — raw LF byte absent from
      message bytes (no log injection);
  `spec_parser.rs` inline test module; SAP-3: drives `SpecLoader::parse()` with raw TOML)
- [ ] **RG-030**: `test_rule9_tab_in_cookie_name_e_spec_027_template_a` — extended AC-019 / EC-009-049
  (`header_scheme = "cookie:\t"` — TAB (0x09) as sole cookie name character;
  SID-2 composed-output assertions on the FULL message bytes (derived from BC-2.16.009 v1.29
  EC-009-049 §Canonical Test Vectors — TAB (0x09) is CTL class 0x09 ≤ 0x1F, so the emitted
  `{value}` is the eleven-character string `cookie:\x09` — `cookie:` [7 chars] + four-char
  ASCII sequence `\x09` [4 chars] — NOT a raw TAB byte; same CTL-escape rule as EC-009-048
  which emits `cookie:a\x0Ab` for LF):
  (a) `assert!(spec_err.message.contains("cookie:\\x09"))` — the full `header_scheme` value
      `"cookie:\t"` after CTL-escaping produces `cookie:\x09` embedded in the message; the
      literal four-char ASCII sequence backslash-x-0-9 must be present in the message;
  (b) `assert!(!spec_err.message.as_bytes().contains(&0x09))` — raw TAB byte (0x09) absent
      from message bytes, confirming no log injection;
  implementation-correctness probe: `char.is_whitespace()` would also reject TAB (for the
  wrong reason), but `char.is_alphanumeric()` would NOT reject it; only the byte-level
  `matches!` predicate correctly rejects 0x09 as a non-tchar CTL byte AND triggers the
  CTL-escape obligation; SP (0x20) is also non-tchar but 0x20 > 0x1F so it is NOT escaped
  — TAB is the boundary case that exercises both rejection AND escaping independently;
  `spec_parser.rs` inline test module; SAP-3: drives `SpecLoader::parse()` with raw TOML;
  DEFENSE-IN-DEPTH note: NOT applicable here — TAB is reachable via the parser surface and
  must be rejected by Rule 9; this is a primary coverage test, not a defense-in-depth test)
- [ ] **RG-031**: `test_rule9_rule10_fail_fast_boundary_single_error_from_parse` — AC-026
  (Spec with `header_scheme = "garbage"` [Rule 9 violation] AND an `[auth_acquisition]`
  block present in the TOML; assert `parse()` returns exactly ONE `ESpec027` error; verifies
  that Rule 9's `Err` return prevents Rule 10 from executing; forward-compatible: after
  S-ADR054-WAVE-A-001 implements Rule 10 the same spec still produces exactly ONE error
  because Rule 9 fires first; `spec_parser.rs` inline test module; SAP-3: drives
  `SpecLoader::parse()` with raw TOML string — parser surface, not synthetic-struct invocation)
- [ ] **RG-032**: `test_rule9_empty_string_header_scheme_e_spec_027_template_a` — EC-001
  (`header_scheme = ""` — empty string is neither `bearer`, `raw`, nor valid `cookie:<name>`;
  `is_valid_cookie_name_tchar` is not reached (the `bearer`/`raw`/`cookie:` prefix check
  fails first); E-SPEC-027(a) fired; asserts E-SPEC-027(a) message VERBATIM from
  error-taxonomy.md v2.70; `spec_parser.rs` inline test module; SAP-3: drives
  `SpecLoader::parse()` with raw TOML)
- [ ] **RG-033**: `test_rule9_uppercase_bearer_rejected_e_spec_027_template_a` — EC-002
  (`header_scheme = "BEARER"` — only lowercase `"bearer"` is valid; case-sensitive string
  match rejects uppercase; E-SPEC-027(a); asserts E-SPEC-027(a) message VERBATIM;
  `spec_parser.rs` inline test module; SAP-3: drives `SpecLoader::parse()` with raw TOML)
- [ ] **RG-034**: `test_rule9_leading_space_header_scheme_rejected_e_spec_027_template_a` — EC-004
  (`header_scheme = " bearer"` — leading SP (0x20); the value is not `"bearer"`, `"raw"`,
  or `"cookie:..."` so E-SPEC-027(a) fires; distinct from RG-022 which tests SP inside a
  cookie name; here the entire value is non-matching; asserts E-SPEC-027(a) message VERBATIM;
  `spec_parser.rs` inline test module; SAP-3: drives `SpecLoader::parse()` with raw TOML)
- [ ] **RG-035**: `test_rule9_cookie_scheme_bearer_static_coherence_e_spec_027_template_b` — EC-007
  (`header_scheme = "cookie:tok"` + `auth_type = "bearer_static"` — coherence violation;
  `bearer_static` permits only `bearer` and `raw`; `cookie:<name>` is disallowed; E-SPEC-027(b)
  with `{allowed_set}` = "bearer, raw"; asserts E-SPEC-027(b) message VERBATIM;
  `spec_parser.rs` inline test module; SAP-3: drives `SpecLoader::parse()` with raw TOML)
- [ ] **RG-036**: `test_rule9_cookie_scheme_oauth2_coherence_e_spec_027_template_b` — EC-010
  (`header_scheme = "cookie:name"` + `auth_type = "oauth2_client_credentials"` — coherence
  violation; `oauth2_client_credentials` permits only `bearer` and `raw`; E-SPEC-027(b)
  with `{allowed_set}` = "bearer, raw"; asserts E-SPEC-027(b) message VERBATIM;
  `spec_parser.rs` inline test module; SAP-3: drives `SpecLoader::parse()` with raw TOML)
- [ ] **RG-037**: `test_rule9_absent_header_scheme_oauth2_passes_absence_path_a` — EC-014 / AC-001 (oauth2_client_credentials arm)
  (`auth_type = "oauth2_client_credentials"` + no `header_scheme` key → spec loads OK
  (absence path A); `SensorSpec::header_scheme = None` is silence-permitted; bearer default
  at runtime; verifies the `oauth2_client_credentials` arm of AC-001's universal quantifier;
  `spec_parser.rs` inline test module; SAP-3: drives `SpecLoader::parse()` with raw TOML)
- [ ] **RG-038**: `test_rule9_absent_header_scheme_api_key_passes_absence_path_a` — AC-001 (api_key arm)
  (`auth_type = "api_key"` + no `header_scheme` key → spec loads OK (absence path A);
  bearer default at runtime; verifies the `api_key` arm of AC-001's universal quantifier;
  `spec_parser.rs` inline test module; SAP-3: drives `SpecLoader::parse()` with raw TOML)
- [ ] **RG-039**: `test_rule9_absent_header_scheme_custom_via_plugin_passes_absence_path_a` — AC-001 (custom_via_plugin arm)
  (`auth_type = "custom_via_plugin"` + no `header_scheme` key → spec loads OK (absence
  path A); bearer default at runtime; verifies the `custom_via_plugin` arm of AC-001's
  universal quantifier; `spec_parser.rs` inline test module; SAP-3: drives
  `SpecLoader::parse()` with raw TOML)
- [ ] **RG-040**: `test_header_scheme_bearer_pipeline_execute_injects_authorization_bearer_e2e` — AC-027
  _(SAP-3 primary end-to-end reachability: drives `PipelineExecutor::execute` with a
  `SensorSpec` that has `header_scheme = "bearer"` and `auth_type = "bearer_static"` + a
  `MockAuthProvider` returning "test-token-abc" against a wiremock server; asserts wiremock
  `Authorization: Bearer test-token-abc` header matcher was satisfied — wire-level assertion
  per 2026-07-13 discipline (header bytes the HTTP server receives, not pre-send Rust struct);
  lives in `crates/prism-spec-engine/tests/bc_2_01_017_static_cookie_auth_provider.rs` or
  `bc_2_01_017_header_scheme_bearer_e2e.rs`; test-writer chooses; RG-011..RG-014 are
  defense-in-depth companions per SAP-3 rule 2)_

**Red Gate density check** (BC-5.38.001): **40 failing tests** before implementation begins.
(19 original + RG-020..RG-023 for EC-009-043..046 per SEC-001 spec amendment + RG-024 for
add_sensor_spec prism-spec-engine API per SAP-3/AC-020 + RG-025 for AC-021 tchar positive
coverage + RG-026 for AC-022 EC-009-050 high-byte predicate probe + RG-027 for AC-023
EC-009-051 cookie-name 128-codepoint length bound + RG-028 for AC-024 EC-009-047 echo cap
SID-2 composed-message assertion + RG-029 for AC-025 EC-009-048 CTL escaping SID-2
composed-message assertion + RG-030 for extended AC-019 EC-009-049 TAB implementation
probe + RG-031 for AC-026 Rule 9/Rule 10 fail-fast boundary observable consequence +
RG-032 for EC-001 empty-string rejection + RG-033 for EC-002 uppercase BEARER rejection +
RG-034 for EC-004 leading-space rejection + RG-035 for EC-007 cookie+bearer_static
coherence + RG-036 for EC-010 cookie+oauth2 coherence + RG-037 for EC-014/AC-001
oauth2_client_credentials absence path A + RG-038 for AC-001 api_key absence path A +
RG-039 for AC-001 custom_via_plugin absence path A + RG-040 for AC-027 SAP-3
end-to-end bearer dispatch integration test.)
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
  no gate bump required (`EXPECTED_SYMBOLS` in `scripts/check-non-exhaustive-per-symbol.py` tracks
  pub struct types, not enum variants):
  ```rust
  /// E-SPEC-027: `header_scheme` field validation failure (Rule 9, BC-2.16.009 v1.29).
  /// Three templates: (a) syntactically invalid value (not bearer/raw/cookie:<tchar-name>);
  /// (b) coherence violation with auth_type; (c) absent header_scheme when
  /// auth_type = "cookie_roundtrip" (absence path B).
  /// See error-taxonomy.md v2.70 E-SPEC-027 for verbatim message templates.
  ESpec027,
  ```
- [ ] **T-A02**: Rewrite `AuthTypeCrossComposition` `#[error(…)]` attribute to match the
  FIRST OR-clause of E-SPEC-012 taxonomy template verbatim per error-taxonomy.md v2.70
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
  taxonomy template verbatim per error-taxonomy.md v2.70. Add/rename struct fields: `sensor_id`,
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

  - **Template (a) `{value}` construction — cap then escape (CWE-400/CWE-117):** When
    building the template (a) message, the raw `header_scheme` string (the value as
    declared in TOML) must be processed in two sequential steps before substituting for
    `{value}`:
    1. **Cap (CWE-400, EC-009-047):** apply `truncate_at_char_boundary(&raw_value, 64)`.
       For values ≤64 codepoints this is a no-op; the capped string equals the original.
    2. **Escape (CWE-117, EC-009-048):** iterate over the UTF-8 byte sequence of the
       capped string. Replace each byte `b` where `(b as u8) <= 0x1F || (b as u8) == 0x7F`
       with the four-character ASCII sequence `\xNN` — literal backslash, lowercase `x`,
       two uppercase hex digits (e.g., LF 0x0A → `\x0A`, CR 0x0D → `\x0D`,
       TAB 0x09 → `\x09`). All other bytes are emitted as-is. For values with no CTL
       bytes this step is a no-op; the emitted message is byte-identical to the capped
       string (POL-24 preserved for common-case inputs).
    Only after both steps is the resulting string substituted for `{value}` in the
    template (a) format string. Templates (b) and (c) substitute `{auth_type}` and
    `{sensor_id}` (config text validated safe for echoing per AD-017) — cap and escape
    are NOT required for those templates.

  Logic:
  1. If `None` AND `*auth_type == AuthType::CookieRoundtrip` →
     Err(PrismError::Spec + ESpec027 + template (c) message verbatim)
  2. If `None` → Ok (absence path A)
  3. Parse value: accept `"bearer"`, `"raw"`, `"cookie:<name>"` where `<name>` is
     non-empty and every byte is RFC 6265 tchar (RFC 9110 §5.6.2):
     ```rust
     // RFC 6265 §4.1.1 cookie-name = token; RFC 9110 §5.6.2 tchar
     // CWE-390: cookie name >128 codepoints rejected at spec-load (EC-009-051).
     fn is_valid_cookie_name_tchar(name: &str) -> bool {
         !name.is_empty()
             && name.chars().count() <= 128  // CWE-390: ≤128-codepoint bound (EC-009-051)
             && name.bytes().all(|b| matches!(b,
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
     template (a) message verbatim from error-taxonomy.md v2.70)
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
  correct; VP-059 is intentionally excluded and must NOT be added to this story.
  The boundary between `parse()`-resident rules and `validate_sensor_spec` rules (including
  any fail-fast vs collect-all semantics at that boundary) is governed by BC-2.16.009
  §Invariants — that contract is the authority (CLAUDE.md Source-of-Truth Precedence rule 1:
  the BC supersedes the story on contract semantics). This story must not re-assert contract
  semantics unilaterally. AC-026 documents the observable consequence of the rule-boundary
  semantics at the `parse()` level.
- [ ] **T-B03**: Call `validate_header_scheme` from `SpecLoader::parse()` after all other
  field validations, citing BC-2.16.009 §Validation Rules 9 §Security requirement. Order:
  Rule 9 runs after auth_type validation succeeds (fail-fast: if Rule 8 fails, Rule 9 is
  not reached). NOTE: `SpecLoader::parse()` is the unconditional call point — calling
  `validate_header_scheme` only from `validate_sensor_spec()` bypasses Rules 8/9/10 and
  leaves the CWE-20/CWE-74 injection vector open per BC-2.16.009 §Security requirement.
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

- [ ] **MERGE-GATE-CYBERINT**: Verify cyberint.sensor.toml co-land with CYBERINT-PATCH-001.
  Once this story lands, `cyberint.sensor.toml` (which has `auth_type = "cookie_roundtrip"`
  with absent `header_scheme`) will be REJECTED at spec-load by E-SPEC-027(c). The minimal
  co-land fix (`S-WAVE-A-CYBERINT-PATCH-001`) MUST be merged in the SAME release batch as
  this story. `S-WAVE-A-CYBERINT-SPEC-001` (the full dual-surface migration) does NOT need
  to co-land — it follows on its own schedule after ENGINE-001 + PATCH-001 are both live
  (per S-WAVE-A-CYBERINT-SPEC-001 §Scheduling Note). Confirm with orchestrator before merging.

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

7. **`#[non_exhaustive]` gate — no new symbol registration**: No new pub types are
   introduced by this story (only a new field on SensorSpec, new variants on SpecEngineError,
   a new method on AuthProvider). `scripts/check-non-exhaustive.sh` must pass;
   `EXPECTED_SYMBOLS` in `scripts/check-non-exhaustive-per-symbol.py` is the single source
   of truth from which the count derives automatically — no new entry is required for this
   story. The registration bump for AuthAcquisitionConfig/CachedAuthToken/ExpiryMode
   belongs to S-ADR054-WAVE-A-001.

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

| File | Purpose |
|------|---------|
| `crates/prism-spec-engine/tests/bc_2_01_017_header_scheme_bearer_e2e.rs` (or add to `bc_2_01_017_static_cookie_auth_provider.rs`) | RG-040 (AC-027): SAP-3 end-to-end integration test; `PipelineExecutor::execute` with `header_scheme = "bearer"` against wiremock. Test-writer chooses whether to add to the existing BC-2.01.017 test file or create a companion file. |

### Files to MODIFY

| File | Change Summary |
|------|----------------|
| `crates/prism-core/src/error.rs` | Add `ESpec027` variant to the `SpecErrorCode` enum (T-A01). DO NOT add `InvalidHeaderScheme`, `HeaderSchemeCoherenceViolation`, or `HeaderSchemeRequiredForCookieRoundtrip` to `SpecEngineError` — those variants are architecturally incompatible with `SpecLoader::parse()`'s `PrismError` return type (Q1 ruling). `SpecErrorCode` is `#[non_exhaustive]`; adding a variant is additive and does NOT require a new entry in `EXPECTED_SYMBOLS` in `scripts/check-non-exhaustive-per-symbol.py` (enum variants are not tracked as pub struct symbols). |
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
| `.factory/specs/prd-supplements/error-taxonomy.md` | Frozen at v2.70 for this story. E-SPEC-027 row (including backtick charset correction) completed in v2.68 by product-owner leg of FB45; EC-009-047..051 added in v2.69; template (a) `≤128 codepoints` clause added in v2.70 (FB56 MED-003 fix). E-SPEC-012 row already executed in v2.57. Spec is CONVERGED — do NOT amend. |
| `.factory/specs/behavioral-contracts/BC-2.16.009-spec-file-validation.md` | Frozen. Rule 9 fully specified. |
| `.factory/specs/behavioral-contracts/BC-2.16.014-*.md` | Frozen. P9 call-site change is code, not spec amendment. |
| `.factory/specs/architecture/decisions/ADR-054-*.md` | Frozen. D11 rows document what code to change; reading only. |
| `crates/prism-sensors/specs/crowdstrike.sensor.toml` | CrowdStrike TOML migration belongs to S-ADR054-WAVE-A-001. |
| `crates/prism-sensors/specs/cyberint.sensor.toml` | The one-line `header_scheme` patch belongs to S-WAVE-A-CYBERINT-PATCH-001 (co-lands with ENGINE-001). The full dual-surface migration belongs to S-WAVE-A-CYBERINT-SPEC-001 (implementation ordering only — no co-land requirement per CYBERINT-SPEC-001 §Scheduling Note). |
| `crates/prism-bin/tests/vp153_rule_c_shaped_probe.rs` | Token_exchange strategy arm ADDITION for `arb_matching_auth_type()` and `arb_mismatched_auth_type_pair()` ROUTED TO S-ADR054-WAVE-A-001 per ORCHESTRATOR RULING (2026-07-24); no changes from this story. |
| `scripts/check-non-exhaustive.sh` | Count is derived automatically from `scripts/check-non-exhaustive-per-symbol.py`; no change needed this story. |

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
    → S-WAVE-A-CYBERINT-SPEC-001  (dual-surface spec migration; implementation ordering only — needs header_scheme grammar; NOT a co-land per CYBERINT-SPEC-001 §Scheduling Note)
    → S-WAVE-A-ARMIS-REMEDIATION-001  (Armis spec uses token_exchange; blocked by ADR-054 story)
```

**Sequencing constraint (ADR-054 §D7 verbatim):** The ADR-054 implementation stories
MUST merge AFTER this story. The story-writer for S-ADR054-WAVE-A-001 MUST encode
`depends_on: [S-WAVE-A-ENGINE-001]` in that story's frontmatter.

**Co-land constraint (Cyberint):** `S-WAVE-A-CYBERINT-PATCH-001` (the minimal one-line
fix) MUST enter the same develop batch as this story. After this story lands,
`cyberint.sensor.toml` is rejected by E-SPEC-027(c) until PATCH-001 adds
`header_scheme = "cookie:access_token"`. `S-WAVE-A-CYBERINT-SPEC-001` (the full
dual-surface migration) does NOT need to co-land — it follows after ENGINE-001 and
PATCH-001 are both live (per CYBERINT-SPEC-001 §Scheduling Note).

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
| 3.2 | 2026-08-13 | POL-23 BC-2.16.014 pin propagation: v1.20 → v1.21 in §Behavioral Contracts table and §Token Budget Estimate table. v1.21 (D-2119): INV-014-007 §Invariants body corrected — ADR-050 v2.2 pin per consistency-validator sweep (was v2.0). No scope, AC, or Red Gate changes for this story. |
| 3.1 | 2026-08-12 | BC-2.16.014 version pin propagation: v1.19 → v1.20 in §Behavioral Contracts table and §Token Budget Estimate table. v1.20 adds INV-014-007 ADR-050 §D5/§D6 note (DeclarativeHttpAuthProvider inherits UA + http2 via build_http_client_with_custom_timeout delegation; implemented by DEFECT-ADAPTER-TLS-XDOME-LIVE-001). No scope or AC changes for this story. |
| 3.0 | 2026-07-27 | FB72 story-writer leg 2: (1) Add `## Authority` section citing ADR-053 §D2 and §D5 — closes MED-002 residual per SAC-2 bidirectional traceability requirement. (2) Add AC-027 (Tier 7) + RG-040 SAP-3 end-to-end bearer dispatch integration test via `PipelineExecutor::execute` + wiremock — closes F-CVB-P67-OBS-001; RG-011..RG-014 annotated as SAP-3 defense-in-depth. (3) Remove five forbidden count restatements (`EXPECTED=92` / `EXPECTED stays at 92` / `check-non-exhaustive.sh EXPECTED=92`) from §Architecture Mapping, §T-A01, §Architecture Compliance Rule 7, §Files to MODIFY (prism-core row), §Files to MODIFY (check-non-exhaustive.sh row) — closes F-CVC-OBS-001; replaced with `EXPECTED_SYMBOLS in scripts/check-non-exhaustive-per-symbol.py is the single source of truth` per CLAUDE.md §Conventions amendment. (4) Add new test file row to §Files to CREATE. RGT COUNT: 39 → 40. AC COUNT: 27 → 28 (AC-027 added). |
| 2.9 | 2026-07-27 | FB67 Obligation 2 (F-WASE-P65-MED-005 story-writer half): fix RG-030 — add SID-2 composed-output assertions specifying the escaped TAB form. TAB (0x09) is CTL class 0x09 ≤ 0x1F; the emitted `{value}` is the eleven-character string `cookie:\x09` per BC-2.16.009 v1.29 EC-009-049 §Canonical Test Vectors; two assertions added: (a) `spec_err.message.contains("cookie:\\x09")` — four-char ASCII backslash-x-0-9 present; (b) `!spec_err.message.as_bytes().contains(&0x09)` — raw TAB byte absent. Obligation 4: BC-2.16.009 pin v1.28 → v1.29 in §Behavioral Contracts table, §Token Budget, and T-A01 §ESpec027 doc-comment (POL-25 sweep: doc comment inside code block also carried stale v1.28 cite). AC count: 27 (unchanged). RGT count: 39 (unchanged). |
| 2.8 | 2026-07-27 | FB63 MED-004: fix §Behavioral Contracts Title column to verbatim BC H1 titles — BC-2.16.009: "Spec File Validation — Schema Validation, Variable Reference Resolution, OCSF Field Validation"; BC-2.01.017: "StaticCookieAuthProvider Contract — No-Login-Roundtrip Cookie Injection"; BC-2.01.016: "SensorAuth Open Trait — Plugin-Implementable Auth Contract (No Sealed Marker)"; BC-2.16.014 "Declarative Auth Acquisition Token Lifecycle" already matched exactly. |
| 2.7 | 2026-07-26 | FB57 story-writer leg — closes six F-WASE-P64-MED findings (MED-002, MED-003 propagation, MED-004 follow-on, MED-007, MED-010, MED-017), all targeting this story. FIX-1 MED-002: EC-009-047 ellipsis marker "excess replaced with `…`" struck; replaced with plain-truncation description via `truncate_at_char_boundary` (no appended marker; POL-24 byte-identity preserved). FIX-2 MED-003 propagation: E-SPEC-027 template (a) string updated across ALL live sites to new form — "non-empty name required" → "non-empty name, ≤128 codepoints" per FB56 product-owner fix; error-taxonomy pin v2.69 → v2.70 at all live sites; BC-2.16.009 pin v1.27 → v1.28 at all live sites. FIX-3 MED-004 follow-on: Removed unilateral Q2 VP-059 assertion "Rule 9 is fail-fast in parse() following the Rule 8 precedent" from T-B02; replaced with cite to BC-2.16.009 §Invariants as the contract authority; added AC-026 (fail-fast boundary observable consequence: spec with Rule 9 violation + `[auth_acquisition]` block → exactly ONE ESpec027 error from `SpecLoader::parse()`; forward-compatible with S-ADR054-WAVE-A-001) + RG-031. FIX-4 MED-007: AC count reconciled 26 → 27 and RGT count 30 → 39 at all live sites (frontmatter TDD-coverage comment, §Red Gate density check heading and explanation). FIX-5 MED-010: CYBERINT-SPEC-001 `blocks:` inline comment and anchor justification corrected — removed stale "stories must co-land" claim and ambiguous "or vice versa"; correct description is implementation-ordering dependency only (CYBERINT-SPEC-001 §Scheduling Note explicitly disclaims co-land); boot-hazard handled atomically by CYBERINT-PATCH-001; updated MERGE-GATE-CYBERINT, §Co-land constraint, §Dependency Graph Edges, Files NOT to MODIFY, and risk note to match. FIX-6 MED-017: Added nine Red Gate tests — RG-031 (AC-026 fail-fast boundary); RG-032 (EC-001 empty string), RG-033 (EC-002 BEARER uppercase), RG-034 (EC-004 leading space), RG-035 (EC-007 cookie+bearer_static coherence), RG-036 (EC-010 cookie+oauth2 coherence), RG-037 (EC-014 / AC-001 oauth2 absence path A), RG-038 (AC-001 api_key absence path A), RG-039 (AC-001 custom_via_plugin absence path A); extended AC-001 text to explicitly name all four non-cookie auth_types and map each to its covering RGT. AC COUNT: 26 → 27. RGT COUNT: 30 → 39. |
| 2.6 | 2026-07-26 | FB55c story-writer leg — closes F-WASE-P64-HIGH-004 (story-owned half). Added VP-160 to `verification_properties:` frontmatter array; added POL-5 anchor justification comment: §Tasks T-B02 step 3 authors `is_valid_cookie_name_tchar` (VP-160's proof target symbol) and §Architecture Mapping assigns `SpecLoader::validate_header_scheme` (Rule 9) to `crates/prism-spec-engine/src/spec_parser.rs`. Added VP-160 row to §Token Budget Estimate table; total updated ~201,000 → ~209,000. No AC changes, no Red Gate test changes. AC COUNT: 26 (unchanged). RGT COUNT: 30 (unchanged). |
| 2.5 | 2026-07-26 | FB55a HIGH-001: remove semantically-inverted `blocks: S-WAVE-A-CYBERINT-PATCH-001` edge. Both stories listing each other in `blocks:` created an unschedulable DAG cycle. The correct edge is PATCH-001 → ENGINE-001 (PATCH gates ENGINE), expressed as `S-WAVE-A-ENGINE-001` in CYBERINT-PATCH-001's `blocks:` array. ENGINE-001's reciprocal entry asserted the opposite direction and is dropped. The prose MERGE-GATE-ENGINE-001 co-land constraint remains in CYBERINT-PATCH-001 — single directional edge plus prose marker is the canonical co-land expression per the wave-a dependency graph convention. |
| 2.4 | 2026-07-26 | FB54 story-writer leg — closes F-WASE-P64-CRIT-005 (POL-38: BC amendment added EC-009-047..049/051 without corresponding ACs or Red Gate tests in this story). Added AC-023 (EC-009-051 CWE-390: cookie-name 128-codepoint length bound — `is_valid_cookie_name_tchar` length guard; RG-027), AC-024 (EC-009-047 CWE-400: 64-codepoint echo cap on template-(a) `{value}` via `truncate_at_char_boundary`; RG-028 SID-2 composed-message assertion), AC-025 (EC-009-048 CWE-117: CTL-byte `\xNN` escaping in template-(a) message; RG-029 dual-assertion on escaped form present + raw byte absent). Extended AC-019 to explicitly enumerate TAB (0x09) alongside `;`, `=`, SP, and CTL bytes; added RG-030 as the EC-009-049 dedicated TAB implementation-correctness probe. Fixed T-B02: `is_valid_cookie_name_tchar` snippet gains `&& name.chars().count() <= 128` length guard (CWE-390); error-construction bullet gains template-(a) two-step cap-then-escape specification. BC-2.16.009 version pin updated v1.26 → v1.27 at all live-pin sites (v1.27 is attribution-only per F-WASE-P64-HIGH-008; ECs and behavioral content unchanged). AC COUNT: 23 → 26 (AC-001..AC-025 plus AC-014b). RGT COUNT: 26 → 30. |
| 2.3 | 2026-07-25 | FB47b records-tier correction — closes F-WASE-P63-HIGH-008. `blocks:` frontmatter array updated by FB47a: added `S-WAVE-A-CYBERINT-PATCH-001` with MERGE-GATE-ENGINE-001 co-land rationale (cyberint.sensor.toml declares `auth_type=cookie_roundtrip` with no `header_scheme`; Rule 9 absence path B rejects this at spec-load producing boot exit 2; ENGINE-001 must co-land with CYBERINT-PATCH-001 to prevent boot failure). FB47a made the `blocks:` array edit without appending a changelog row; this v2.3 row is the missing records-tier correction. No AC changes, no Red Gate test changes, no task changes, no behavioral-contract version propagation. AC COUNT and RGT COUNT unchanged: 23 ACs / 26 RGTs. |
| 2.2 | 2026-07-25 | FB46 story-writer leg — closes 7 pass-62 findings. F-WASE-P62-CRIT-001: AC-020 + RG-024 restated from MCP wire-level assertion to prism-spec-engine API boundary; assert `Ok(AddSensorSpecResult::ValidationFailed { errors })` with E-SPEC-027(a) text match; `structuredContent.error.code` assertion removed (unsatisfiable per ADR-053 §D6 Option B); S-WAVE-A-MCP-001 noted as carrying wire-level intent; `prism-mcp` confirmed absent from `crates_touched` / Files to MODIFY. F-WASE-P62-HIGH-003: AC-021 (all 15 RFC 6265 tchar special chars accepted — ``"cookie:a!#$%&'*+-.^_`|~9Z"``) + RG-025 (`test_rule9_all_tchar_special_chars_cookie_name_accepted`) added; AC-022 (EC-009-050 high-byte U+00E9 predicate divergence probe rejection) + RG-026 (`test_rule9_high_byte_in_cookie_name_e_spec_027_template_a`) added; Red Gate density 24 → 26. F-WASE-P62-HIGH-004: T-B03 parenthetical "(or the validation pass within spec_parser.rs)" deleted; `SpecLoader::parse()` stated unconditionally with BC-2.16.009 §Security requirement cite; failure path note added. F-WASE-P62-MED-008: BC-2.01.016 §Behavioral Contracts row first sentence corrected — "AuthProvider trait extension: `get_token()` default method added" removed (belongs to BC-2.16.014 §P9); BC-2.01.016 scope now reads Rule A/B E-SPEC-012/013 Display alignment only; BC-2.16.014 scope updated to include get_token() trait method addition. F-WASE-P62-LOW-002: story-local EC-009 (oauth2 absent header_scheme, loads OK) renumbered to EC-014, eliminating prefix collision with BC-canonical EC-009-043..EC-009-051 range. F-WASE-P62-LOW-003: EC-003/EC-005/EC-008 canonicalized to BC-canonical IDs EC-009-033/EC-009-031/EC-009-042 (consistent with EC-043..046 treatment; decision: normalize all BC-mapped rows to BC-canonical IDs). F-WASE-P62-LOW-004: Phase F and Phase G task sections reordered to alphabetical (F before G); no task cross-references broken. VERSION PROPAGATION: BC-2.16.009 v1.25 → v1.26 (§Behavioral Contracts, §Token Budget); error-taxonomy v2.68 → v2.69 (AC-005, AC-017, AC-018, T-A01 comment, Files NOT to Modify); token budget BC-2.16.009 row updated with v1.26 EC additions. NEW EDGE CASES: EC-009-047..EC-009-051 added to §Edge Cases table (overlong value echo cap CWE-400, CTL-byte `\xNN` hex escaping CWE-117, TAB in cookie name, high-byte predicate divergence probe, cookie name > 128 codepoints CWE-390). AC COUNT: 21 → 23 (AC-001..AC-022 plus AC-014b). RGT COUNT: 24 → 26. |
| 2.1 | 2026-07-24 | FB45 story-writer leg — closes F-WASE-P61-HIGH-001 (charset + version propagation), F-WASE-P61-HIGH-002 (add_sensor_spec injection vector coverage), F-WASE-P61-MED-002 (SEC-001 Red Gate tests gain AC coverage), F-WASE-P61-MED-003 (stale frozen-perimeter claims), F-WASE-P61-LOW-003 (discontinuous EC ID scheme), F-WASE-P61-LOW-004 (AC-015 compile-time postcondition). CHARSET: AC-005 + AC-007 tchar set corrected to 15-char RFC 9110 §5.6.2 order (backtick U+0060 restored, tail corrected to `^ _ ` \| ~`); double-backtick code spans per upstream convention; error-taxonomy.md pinned at v2.68. BC VERSIONS: BC-2.16.009 v1.24→v1.25, BC-2.16.014 v1.18→v1.19, BC-2.01.016 v1.14→v1.15 in §Behavioral Contracts table and §Token Budget Estimate table; BC-2.16.009 Token Budget row updated v1.23→v1.25 with token estimate bumped to ~24,000 (reflects v1.24–v1.25 Rule 9 growth). NEW ACs: AC-019 (non-tchar char rejection for SEC-001 injection classes; RG-020..023 re-anchored to AC-019 in addition to EC-009-043..046); AC-020 (add_sensor_spec MCP wire-level coverage per SAP-3 §Entry points). NEW RGT: RG-024 (add_sensor_spec MCP surface, wire-shape assertion on `structuredContent.error.code`). Red Gate density updated 23→24. AC COUNT: 19→21. AC-015 rewritten to specify delegation-counter runtime assertion; 8-implementor compile claim moved to build-gate note. EDGE CASES: EC-043..046 renamed to BC-canonical IDs EC-009-043..EC-009-046 to resolve anchor ambiguity (option a — BC references, not story-local). ARCH RULE 6: stale "FROZEN (3/3 CLEAN at passes 58/59/60)" premise replaced by accurate description of spec perimeter status post-FB45. FILES NOT TO MODIFY: error-taxonomy.md version reference updated to v2.68. Frontmatter points note updated 19→21 ACs. |
| 2.0 | 2026-07-24 | Q1–Q6 architect rulings applied (wave-a-engine-story-adjudication-Q1-Q5): Q1 — ESpec027 added to SpecErrorCode in prism-core (NOT three SpecEngineError variants); error construction uses inline PrismError::Spec following Rule 8 precedent; crates_touched expanded to include prism-core. Q2 — Rule 9 is fail-fast in parse() per Rule 8 precedent; VP-059 explicitly excluded (stated in T-B02). Q3 — ADR-031 fidelity survives via spec-declared header_scheme; stale hardcoded-name comment removed from build_request() per TD-VSDD-091; ADR-031 cite added to T-C01. Q4 — validate_header_scheme parameter changed to `&AuthType` (NOT `&str`); SAP-3 defense-in-depth comment requirement added for Rule A path. Q5 — E-SPEC-012 Display includes `token_exchange` in Valid values clause now; POL-24 comment added at rewrite site (AC-017). Q6(C) — ADR-052 §D5 scope-exclusion forward constraint added as Architecture Compliance Rule 10. SEC-001 spec amendment (BC-2.16.009 v1.24): Rule 9 tchar charset replaces old no-colon rule; EC-043..046 added to Edge Cases; RG-020..023 added to Red Gate tests; Red Gate density updated 19→23. BLOCKER resolutions: BLOCKER 1 — MERGE-GATE-VP153-PARTIAL nextest filter corrected to binary(vp153_sensorauth_cross_composition); BLOCKER 2 — build_request return type corrected to reqwest::RequestBuilder throughout; BLOCKER 3 — three stale SpecEngineError variants replaced by ESpec027 in prism-core; BLOCKER 4 — test files bc_2_01_016_test and crowdstrike_oauth2_plugin_tests added to Files to MODIFY; BLOCKER 5 — ORCHESTRATOR RULING text corrected (token_exchange arm ADDITION, not marker removal; no [PLANNED] markers exist in harness code). Phase G added (T-G01..T-G05): five exhaustive SensorSpec literal sites broken by header_scheme field addition. T-E01 RQ-2 dyn-compatibility confirmation note added. T-E02 corrected to grep `impl .*AuthProvider for` and count 8 implementors (BearerStaticAuthProvider in prism-bin). T-E02 MERGE-GATE-JUST-CHECK reworded: ZERO new warnings relative to merge base (baseline has pre-existing unused-import warning). Files to MODIFY table rewritten: stale three-variant error.rs entry replaced with ESpec027-in-prism-core entry; three additional rows added (bc_2_01_016_test, crowdstrike_oauth2_plugin_tests, spec_validator). Files NOT to MODIFY vp153_rule_c_shaped_probe.rs entry corrected to say arm ADDITION. Architecture Mapping [PLANNED] language corrected. |
| 1.1 | 2026-07-24 | Encode ORCHESTRATOR RULING (2026-07-24): token_exchange `VALID_AUTH_TYPES` addition and VP-153 token_exchange arm activation (former T-F02/T-F03/T-F04) ROUTED to S-ADR054-WAVE-A-001. Dual-path conditional in Phase F removed — story states ONE path: `VALID_AUTH_TYPES` is NOT touched here. `MERGE-GATE-VP153` narrowed to `MERGE-GATE-VP153-PARTIAL` (prism-spec-engine only, existing active arms, no token_exchange activation). Proof re-run split: satisfiable portion (Rule A/B/C active arms + E-SPEC-012 Display validation) stays here; full 8-proptest run with token_exchange arms is `MERGE-GATE-VP153-FULL` in S-ADR054-WAVE-A-001. Architecture Compliance Rule 9 added. `crates_touched` updated (prism-bin removed). Files to Modify table updated (vp153_rule_c_shaped_probe.rs moved to Files NOT to modify). D11 exclusions list expanded with three new routed items. AC count unchanged (19 ACs). D11 row count unchanged (9 rows, VP-153 row scoped to partial). |
| 1.0 | 2026-07-24 | Initial story creation. |
