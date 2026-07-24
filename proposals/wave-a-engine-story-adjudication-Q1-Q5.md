---
document_type: architect-proposal
title: "S-WAVE-A-ENGINE-001 — Six Uncertainty-Scan Adjudications (Q1–Q6)"
author: architect
date: "2026-07-24"
status: ADJUDICATED-Q1-Q5-ALL-A-Q6-C
version: "1.1"
story_id: S-WAVE-A-ENGINE-001
blocking_cascade: false
perimeter_impact: NONE
decision: "Q1–Q5 resolved code-side (A). Q6 resolved (C): no spec amendment, story-level code-comment constraint required at the expiry parsing site. Spec perimeter remains frozen. No human reopen decision required."
---

# S-WAVE-A-ENGINE-001 — Architect Adjudication of Six Uncertainty-Scan Questions (Q1–Q6)

Produced by architect agent for orchestrator dispatch.
No spec artifacts were modified. No code was modified.
The Wave-A spec perimeter remains FROZEN (BC-5.39.001 strict 3/3, D-2011).

---

## Context

The story `S-WAVE-A-ENGINE-001` (v1.1) is the first story in the ADR-054 §D7 sequencing
chain. It adds `SensorSpec::header_scheme`, Rule 9 / E-SPEC-027 validation, the `build_request()`
header-injection dispatch switch, and `AuthProvider::get_token()`. An uncertainty scan produced
five open questions that could not be resolved by the test-writer or implementer without
architect adjudication.

**Critical constraint applied:** the spec perimeter is FROZEN. Every question was evaluated
against the (A) / (B) classification framework. All five resolved as (A) — resolvable
code-side without amending any `.factory/specs/` artifact.

---

## Q1 (was U-03) — Which error mechanism owns Rule 9 / E-SPEC-027?

### Evidence reviewed

- `crates/prism-spec-engine/src/spec_parser.rs` `SpecLoader::parse()` returns `Result<SensorSpec, PrismError>`, not `Result<SensorSpec, SpecEngineError>`.
- Rule 7 / E-SPEC-025 pattern: `ValidationError` (= `SpecError`) collected in `validation.rs::validate_sensor_spec`.
- Rule 8 / E-SPEC-026 pattern: `PrismError::Spec(SpecError { code: SpecErrorCode::ESpec026, message: format!(...), toml_path: Some("sensor.probe_table") })` constructed inline in `parse()`, fail-fast.
- cross-composition pattern: `PrismError::Internal { detail: format!("cross-composition validation failed for sensor '{}': {}", ..) }` — wraps and PREFIXES a `SpecEngineError`'s display. **This prefix breaks verbatim message checks.**
- Story's proposed mechanism: bare `SpecEngineError` variants returned from `parse()`. **Not viable**: `parse()` returns `PrismError`, not `SpecEngineError`. A bare `SpecEngineError` cannot be returned without conversion.
- ADR-053 §D2: "validated at spec-load time in the same multi-error pass as other field validations (a new Rule 9, after Rule 8 probe_table, per BC-2.16.009)."
- `prism-core/src/error.rs` `SpecErrorCode` enum: currently ends at `ESpec026`. Adding `ESpec027` is one additive `#[non_exhaustive]` enum variant — no semver break.

### Ruling

**Rule 9 / E-SPEC-027 follows the Rule 8 mechanism: inline validation in `SpecLoader::parse()`, returning `Err(PrismError::Spec(SpecError { code: SpecErrorCode::ESpec027, message: <verbatim template text>, toml_path: Some("sensor.header_scheme"), file_path: None, line_number: None }))`.** No `PrismError::Internal` wrapping and no message prefix.

The story's proposed `SpecEngineError` variants (`InvalidHeaderScheme`, `HeaderSchemeCoherenceViolation`, `HeaderSchemeRequiredForCookieRoundtrip`) are NOT added to `error.rs`. They are architecturally incompatible with `parse()`'s return type. The Rule 8 pattern constructs `SpecError` directly — this is the only existing pattern in `parse()` that (a) works with `PrismError`'s return type, (b) preserves verbatim message content, and (c) does not add a message prefix that would break AC-005..010 verbatim checks.

The story's T-A01 is therefore revised:
- **REMOVE:** three `SpecEngineError` variants (`InvalidHeaderScheme`, `HeaderSchemeCoherenceViolation`, `HeaderSchemeRequiredForCookieRoundtrip`) from `error.rs`.
- **ADD INSTEAD:** `ESpec027` variant to `SpecErrorCode` in `crates/prism-core/src/error.rs`. This is one additive enum variant on a `#[non_exhaustive]` enum; no non-exhaustive gate bump (ESpec027 is an enum variant, not a new pub struct — the EXPECTED=92 gate counts pub types, not enum variants).

The story's T-B02 is revised:
- **CHANGE:** `validate_header_scheme(sensor_id: &str, header_scheme: Option<&str>, auth_type: &AuthType) -> Result<(), PrismError>` (return type `PrismError`, not `SpecEngineError`; auth_type parameter type follows Q4 ruling below).
- The function constructs `PrismError::Spec(SpecError { code: SpecErrorCode::ESpec027, message: verbatim_template_text, ... })` directly.

The story's T-B03 is unchanged in intent: call `validate_header_scheme` from `parse()` after Rule 8.

AC-005..010 assert "verbatim" — this works against `spec_err.message` extracted from the returned `PrismError::Spec(spec_err)`. Test pattern:

```rust
let err = SpecLoader::parse(bad_toml).unwrap_err();
let prism_core::PrismError::Spec(spec_err) = err else { panic!(...) };
assert_eq!(spec_err.message, expected_verbatim_message);
// NOT: assert_eq!(err.to_string(), ...) — PrismError::Spec Display wraps the message
```

`crates_touched` must be expanded to include `prism-core` (for `ESpec027` in `SpecErrorCode`).

### Classification: **(A) — code-side / story-side**

**Instruction to story-writer / implementer:**
1. Strike T-A01's three `SpecEngineError` variants. Add `ESpec027` to `SpecErrorCode` in `crates/prism-core/src/error.rs` instead.
2. Change T-B02 signature to `-> Result<(), PrismError>` with `auth_type: &AuthType` parameter.
3. Expand `crates_touched` frontmatter to `[prism-spec-engine, prism-core]`.
4. Expand "Files to MODIFY" table to include `crates/prism-core/src/error.rs`.
5. Write AC-005..010 tests against `spec_err.message` (extracted from `PrismError::Spec`).

---

## Q2 (was U-09) — Fail-fast path or all-errors-collected path for Rule 9?

### Evidence reviewed

- `crates/prism-spec-engine/src/validation.rs` module-level doc: "Validation is ALWAYS a single-pass, all-errors-collected operation. It NEVER returns early on the first error." This describes `validate_sensor_spec`, NOT `SpecLoader::parse()`.
- VP-059 property statement: "For any `SensorSpec` with N distinct validation errors, `validate_sensor_spec(spec)` returns `Err(errors)` where `errors.len() == N`." VP-059 explicitly targets `validate_sensor_spec`.
- Rule 8 already lives in `parse()` as fail-fast. VP-059 does NOT count Rule 8 (ESpec026) errors in its N-count. This is not a gap — it is the correct scoping: VP-059 governs `validate_sensor_spec` (Rules 1–7); `parse()` has separate fail-fast behavior.
- ADR-053 §D2: "in the same multi-error pass as other field validations" — this is aspirational language in the ADR context description, but the as-built architecture already deviates for Rule 8 (fail-fast in `parse()`). Following Rule 8's precedent for Rule 9 is the pragmatic correct decision.
- Rule 9 governs a single field (`header_scheme`). A spec can produce at most ONE E-SPEC-027 error. The fail-fast vs collect-all distinction has no observable difference for a single-error rule.

### Ruling

**Rule 9 is on the fail-fast path in `parse()`, following Rule 8 precedent.** VP-059 governs `validate_sensor_spec` (Rules 1–7 only) and is NOT affected by Rule 9 in `parse()`. The story's `verification_properties: [VP-153]` frontmatter is correct — VP-059 is not in scope for this story.

No VP-059 harness update is required. No new injector cases need to be added to the VP-059 skeleton for E-SPEC-027.

### Classification: **(A) — code-side**

**Instruction to story-writer / implementer:** No changes to `verification_properties` frontmatter. VP-059 is correctly excluded.

---

## Q3 (was U-11) — Does ADR-053 §D2 supersede ADR-031 §D4's cookie-name mandate?

### Evidence reviewed

- ADR-053 frontmatter `supersedes`: "ADR-031 §D3 (scope-narrowing only: … §D3-b item 3 auth_type-keyed dispatch table superseded by header_scheme dispatch per ADR-053 D2/D5 …)"
- ADR-031 §D4 specifies the `build_request` dispatch table: `CookieRoundtrip → Cookie: access_token={token}`. §D4 is the operationalization of §D3-b item 3.
- `pipeline.rs` line ~1000: hardcoded `req.header("Cookie", format!("access_token={}", token.as_str()))` with comment: `"Cookie name MUST be 'access_token' — NOT 'cyberint_session' (permanently superseded per ADR-031 §D3 and §D4)."`
- ADR-053 §D2: "Both new Cyberint spec files declare explicit `header_scheme = 'cookie:access_token'` and therefore pass Rule 9." — This demonstrates the fidelity constraint is preserved; it is now declared in TOML rather than hardcoded in dispatch.
- ADR-031 §D1-a (NOT superseded): "DTU MUST use exactly the field names, header names, and cookie names the real API uses." The real Cyberint API uses `access_token`. This principle is strengthened by ADR-053 §D1, not weakened.
- POL-36 / INV-014-001: no sensor-name-conditional logic in the engine. A Rule 9 allowlist restricting cookie names to `"access_token"` would encode Cyberint-specific knowledge in the engine — a direct POL-36 violation.

### Ruling

**ADR-031 §D4's cookie-name mandate (`access_token`) SURVIVES as a DTU fidelity principle.** ADR-053 supersedes the enforcement MECHANISM (hardcoded auth_type-keyed dispatch → spec-declared `header_scheme`), not the fidelity requirement. The real Cyberint API still requires `Cookie: access_token={token}`, so the correct `header_scheme` for Cyberint is `"cookie:access_token"`. This fidelity is now encoded in the TOML spec rather than in the engine's dispatch logic.

Three code-side consequences the implementer must handle:

**1. Stale in-code comment (TD-VSDD-091):** When T-C01 rewrites `build_request()` dispatch from auth_type-keyed to header_scheme-keyed, the comment block at the `CookieRoundtrip` arm (`"Cookie name MUST be 'access_token' — NOT 'cyberint_session' (permanently superseded per ADR-031 §D3 and §D4)"`) and the function-level doc table MUST be removed (they reference a dispatch mechanism that no longer exists). Replace with a behavioral anchor tied to the new header_scheme mechanism:

```rust
// ADR-053 §D2: cookie name is declared in the sensor spec's header_scheme field
// (e.g., "cookie:access_token" for Cyberint per ADR-031 DTU fidelity principle).
// No hardcoded cookie name — the name is extracted from header_scheme[7..].
```

**2. No Rule 9 allowlist:** Rule 9 validates the SYNTAX and AUTH_TYPE COHERENCE of `header_scheme`. It does NOT restrict cookie names to an allowlist. `"cookie:cyberint_session"` would pass Rule 9's syntactic check. The fidelity constraint (Cyberint MUST use `access_token`) is enforced by the Cyberint spec file (`S-WAVE-A-CYBERINT-SPEC-001`), not by the engine. Introducing an engine-level allowlist would be a POL-36 / INV-014-001 violation.

**3. Compensating control is S-WAVE-A-CYBERINT-SPEC-001:** ADR-053 §D2 explicitly states that both new Cyberint spec files must declare `header_scheme = "cookie:access_token"`. The story's existing `blocks` array and `MERGE-GATE-CYBERINT` already encode this co-land constraint. No additional mechanism is needed.

### Classification: **(A) — code-side**

**Instruction to implementer:** When rewriting `build_request()` per T-C01, remove the stale hardcoded-name comments per TD-VSDD-091. Replace with the behavioral anchor comment above. Do NOT add a cookie-name allowlist to Rule 9. The fidelity requirement is enforced by S-WAVE-A-CYBERINT-SPEC-001.

---

## Q4 (was U-12) — Should `validate_header_scheme` take `&AuthType` instead of `&str`?

### Evidence reviewed

- `crates/prism-spec-engine/src/spec_parser.rs` `AuthType` is a typed enum with `as_str()`. `SensorSpec.auth_type: AuthType`.
- ADR-054 §D7 coherence matrix scope boundary: the 5-variant coherence matrix in this story must NOT include `token_exchange` until S-ADR054-WAVE-A-001 adds `AuthType::TokenExchange`.
- If `validate_header_scheme` takes `&str` (as the story proposes): when S-ADR054-WAVE-A-001 adds `AuthType::TokenExchange` and calls `validate_header_scheme("token_exchange", ...)`, the coherence match inside would fall through to a wildcard arm (or compile without error) — silently missing the coherence matrix row for `token_exchange`. This is exactly the forward-reference bug class ADR-054 §D7 exists to prevent.
- If `validate_header_scheme` takes `&AuthType`: when S-ADR054-WAVE-A-001 adds `AuthType::TokenExchange`, any `match auth_type { AuthType::BearerStatic => ..., ..., }` inside `validate_header_scheme` would produce a compiler error E0004 (non-exhaustive patterns) — the story-writer for S-ADR054-WAVE-A-001 CANNOT compile without adding the `TokenExchange` arm and its coherence matrix row.
- `AuthType` is `#[non_exhaustive]` (workspace convention). A wildcard `_ => { ... }` arm is required for external match sites, but `validate_header_scheme` in `spec_parser.rs` is in the SAME crate as `AuthType` — internal match does NOT require a wildcard. This means the compile-error enforcement is fully load-bearing for internal callers like `validate_header_scheme`.
- SAP-3 (CLAUDE.md standing probe: spec-arm reachability): Rule A / E-SPEC-012 (`validate_cross_composition`, `VALID_AUTH_TYPES` check) is only reachable by direct invocation. `AuthType` deserialization via `toml::from_str` rejects unknown auth_type strings at the serde boundary (E-SPEC-001), before `validate_cross_composition` can check Rule A. Any `auth_type` that reaches Rule A from the TOML parser surface is ALREADY a valid `AuthType` variant — Rule A from `parse()` is structurally unreachable from the public TOML surface.

### Ruling

**`validate_header_scheme` MUST take `&AuthType` (or `AuthType`) as its `auth_type` parameter, not `&str`.** This makes the coherence matrix exhaustiveness compiler-enforced for the S-ADR054-WAVE-A-001 story, directly implementing the ADR-054 §D7 scope boundary as a compile-time guarantee rather than a documentation reminder.

The story's T-B02 signature (see also Q1 ruling) becomes:

```rust
fn validate_header_scheme(
    sensor_id: &str,
    header_scheme: Option<&str>,
    auth_type: &AuthType,
) -> Result<(), PrismError>
```

The coherence matrix `match auth_type` inside this function has exactly 5 arms (one per existing variant). When `AuthType::TokenExchange` is added by S-ADR054-WAVE-A-001, this match produces E0004 — the implementer of that story MUST add the `TokenExchange` arm with its coherence matrix row (`bearer` or `raw` allowed, `cookie:<name>` disallowed) before the crate compiles.

**SAP-3 compliance for Rule A defense-in-depth:** Any Red Gate test for AC-005 that constructs a `validate_header_scheme` call path from `SpecLoader::parse()` is reachable from the public surface. However, `validate_cross_composition`'s Rule A check (E-SPEC-012) is NOT reachable from a TOML parse of `auth_type = "garbage"` — serde would reject that string with E-SPEC-001 before Rule A runs. Per SAP-3, any test that exercises Rule A by directly calling `validate_cross_composition` with an invalid auth_type string MUST carry the following comment:

```rust
// SAP-3: Rule A / E-SPEC-012 in validate_cross_composition is defense-in-depth by
// construction. A TOML spec with an unrecognized auth_type string is rejected by serde
// at deserialization time (E-SPEC-001) before validate_cross_composition is reached.
// This test validates the defense-in-depth internal path only, not the parser surface.
// Spec-arm reachability from SpecLoader::parse() is covered by RG-005..010 for Rule 9.
```

### Classification: **(A) — code-side**

**Instruction to story-writer / implementer:**
1. Change T-B02 `auth_type` parameter from `&str` to `&AuthType`.
2. The coherence matrix `match auth_type` inside `validate_header_scheme` must be exhaustive over the 5 existing variants with NO wildcard arm that would let `TokenExchange` fall through silently.
3. Add the SAP-3 rationale comment to any test that exercises Rule A via direct `validate_cross_composition` invocation.

---

## Q5 (was U-08) — Ship the self-contradictory "Valid values: token_exchange" message or defer?

### Evidence reviewed

- `error-taxonomy.md` v2.57 E-SPEC-012 "Valid values:" clause (executed Wave-A spec burst 3, 2026-07-22): includes `token_exchange` as the 6th value. This is the POL-24 source of truth. **Already executed.**
- VP-153 v0.21 (executed Wave-A spec burst 3, 2026-07-22): `prop_rule_a_invalid_auth_type_rejected_with_e_spec_012` asserts the E-SPEC-012 Display includes `token_exchange` in "Valid values:". **Already executed.**
- Story's MERGE-GATE-VP153-PARTIAL requires `prop_rule_a_invalid_auth_type_rejected_with_e_spec_012` to PASS.
- `VALID_AUTH_TYPES` constant in `spec_parser.rs::validate_cross_composition` currently has 5 values; ORCHESTRATOR RULING requires it to stay at 5 until S-ADR054-WAVE-A-001.
- ADR-054 §D11: E-SPEC-012 code-alignment row ("PENDING — engine story") requires the Display to be updated to match the taxonomy template verbatim, including `token_exchange`.
- **Option A (ship with `token_exchange` in "Valid values:"):**
  - VP-153 proptest PASSES
  - MERGE-GATE-VP153-PARTIAL satisfied
  - For one story cycle: `auth_type = "token_exchange"` is rejected by E-SPEC-012, and the error message says "token_exchange" is a valid value. Self-contradictory UX but by construction (taxonomy-first design per POL-24).
- **Option B (defer "Valid values: token_exchange" to S-ADR054-WAVE-A-001):**
  - VP-153's proptest expects `token_exchange` in the message (per v0.21). If the Display does NOT include it, the proptest FAILS.
  - MERGE-GATE-VP153-PARTIAL is NOT satisfiable under Option B.
  - This would require VP-153 v0.21 to be rolled back — a spec amendment. **Not available (perimeter frozen).**

### Ruling

**Ship the E-SPEC-012 Display WITH `token_exchange` in "Valid values:" (Option A).** Option B is ruled out because it would fail MERGE-GATE-VP153-PARTIAL: VP-153's proptest (already executed at v0.21) expects the message to include `token_exchange`. Choosing Option B would require amending VP-153 — a (B) classification that reopens the perimeter. Option A is fully consistent with POL-24 (taxonomy is the source of truth; code catches up), VP-153 v0.21, and the ORCHESTRATOR RULING (which restricts `VALID_AUTH_TYPES` from gaining `"token_exchange"`, not the Display string).

The self-contradiction lasts exactly one story cycle. This is the intentional design of the taxonomy-first approach: the taxonomy, VP-153, and error-taxonomy.md were updated in Wave-A spec burst 3 to include the complete 6-value set. The Display now matches the spec. The VALID_AUTH_TYPES runtime constant catches up in S-ADR054-WAVE-A-001.

**Implementation note:** The implementer MUST add the following comment at the E-SPEC-012 Display site in T-A02 (the `#[error(...)]` rewrite):

```rust
// POL-24: error-taxonomy.md v2.57 (Wave-A burst 3) and VP-153 v0.21 list
// token_exchange as the 6th valid value. The Display includes it now per the
// taxonomy source-of-truth contract. VALID_AUTH_TYPES will include "token_exchange"
// in S-ADR054-WAVE-A-001 — for one cycle, the message is intentionally ahead of
// the runtime constant by design (taxonomy-first, per ADR-054 D11 row).
```

### Classification: **(A) — code-side**

**Instruction to implementer:** Apply T-A02 as specified: rewrite `AuthTypeCrossComposition` `#[error(…)]` to the taxonomy v2.57 template VERBATIM, including `token_exchange` as the 6th value. Add the comment above. Do NOT defer the `token_exchange` entry from the Display string.

---

## Q6 — ADR-052 §D5 single-parser mandate vs Wave-A inline expiry parsing

### Sub-determinations

1. ADR-052 §D5 precise scope — data-plane universal mandate or targeted adapter-boundary rule?
2. `prism-spec-engine::datetime` module doc carve-out reconciliation
3. Auth-token-expiry parsing site — inside or outside D5's governed domain?
4. Relaxed-vs-strict divergence defensibility and future-maintainer trap
5. ADR-052 ↔ ADR-054/BC-2.16.014 mutual cross-reference check
6. Deprecated chrono API sub-finding (separate classification)

### Evidence reviewed

- ADR-052 `§D5` titled "Pushdown / adapter-boundary semantics (explicit no-change statement)": operative text governs the data-plane sensor column normalization boundary — inbound ISO-8601 strings from sensor API responses parsed to `i64` microseconds-since-epoch for Arrow `Timestamp(Microsecond, UTC)` column storage. `related_adrs: [ADR-024, ADR-033, ADR-040, ADR-043, ADR-044, ADR-051]` — ADR-054 absent.
- ADR-052 `§D4` ("Chrono strictness invariant, AC-013, preserved"): "The sensor-boundary datetime parsing path (`spec_driven_adapter.rs`, ISO-8601 string → `i64` microseconds-since-epoch) continues to use `chrono::DateTime::parse_from_rfc3339`."
- `prism-spec-engine::datetime` module doc (as built): "# ADR-052 D5 — identical chrono strictness. Uses `chrono::DateTime::parse_from_rfc3339` exclusively. The normaliser's lenient-IN behaviour in the ingestion path is intentionally outside D5 scope." Mandate: "**Do NOT introduce a second RFC-3339 parser.** Any new datetime-parsing site that needs **`Timestamp(µs,UTC)` output** MUST call this function (ADR-052 D5)." The output-type qualifier is load-bearing.
- `prism-spec-engine::datetime::parse_datetime_to_micros`: returns `Result<i64, SpecEngineError>` where `i64` is microseconds-since-epoch for Arrow column storage.
- ADR-054 `§D4 step 4` (`absolute_utc_string` expiry mode): `lenient s.parse::<DateTime<FixedOffset>>()` (chrono relaxed `FromStr` — accepts both T-separator ISO-8601 `"2099-01-01T00:00:00Z"` AND space-separated `"YYYY-MM-DD HH:MM:SS.ffffff+HH:MM"`; rejects only if even relaxed parse fails → E-AUTH-001). Rationale (RU-Q1): Armis Python backend likely emits space-separated UTC strings that strict `parse_from_rfc3339` rejects.
- BC-2.16.014 `§P2` (`absolute_utc_string` mode): formula `expiry_str.parse::<DateTime<FixedOffset>>().map(|dt| dt.timestamp() as u64).saturating_sub(ttl_buffer_secs)` — output is `u64` Unix seconds for `CachedAuthToken::expires_at`.
- BC-2.16.014 `TV-2`: `"2099-01-01 00:00:00+00:00"` (space-separated) yields same `expires_at` (epoch 4_070_908_800 Unix seconds) as `"2099-01-01T00:00:00Z"`.
- VP-159 `§AC-6c`: asserts `"2099-01-01 00:00:00.000000+00:00"` parses to the same epoch as `"2099-01-01T00:00:00Z"` — kill condition is strict `parse_from_rfc3339`, which rejects space-separated form.
- VP-159 proof harness (full read): all AC-6/AC-7 variants use `expiry_str.parse::<DateTime<FixedOffset>>().map(|dt| dt.timestamp() as u64)`. No invocation of `parse_datetime_to_micros` anywhere in the harness. Clock seam is `Arc<AtomicU64>`; HTTP interception is wiremock. No deprecated chrono constructors found.
- ADR-054 `related_adrs: [ADR-023, ADR-026, ADR-028, ADR-031, ADR-032, ADR-050, ADR-053]` — ADR-052 absent.
- `wave-a-engine-story-uncertainty-research.md` RQ-3 findings: `DateTime<FixedOffset>` `FromStr` delegates to `parse_rfc3339_relaxed` — offset mandatory, space OR T separator accepted. Deprecated API family: `NaiveDateTime::from_timestamp*` (since 0.4.23/0.4.35 → build failure under `-D warnings`). NOT deprecated: `DateTime::timestamp()` (method on an existing `DateTime` returning `i64` seconds).

### Sub-determination 1 — ADR-052 §D5 precise scope

ADR-052 is titled "PrismQL Native Temporal Typing — Datetime Columns and Literals from Arrow Utf8 to Timestamp(Microsecond, UTC)." §D5 is titled "Pushdown / adapter-boundary semantics (explicit no-change statement)." Its operative text addresses:
- ADR-033 T1 push-down extractor: `Literal::Timestamp.instant` → `.to_rfc3339()` for sensor API filter strings
- Sensor timestamp parsing addition: `spec_driven_adapter.rs` parsing inbound sensor API ISO-8601 strings to `i64` microseconds-since-epoch

**§D5 is a data-plane mandate** scoped to the OCSF sensor column ingestion path. It does not declare itself a universal single-parser rule for all datetime handling in the codebase. The mandate wording in the `datetime.rs` module doc is explicitly qualified: "Any new datetime-parsing site that needs **`Timestamp(µs,UTC)` output**." This qualifier is not decorative — it is the scope boundary.

### Sub-determination 2 — Module doc carve-out reconciliation

The phrase "The normaliser's lenient-IN behaviour in the ingestion path is intentionally outside D5 scope" in the module doc refers to the error-handling behavior: when `parse_from_rfc3339` fails on a sensor data cell, the normaliser handles this gracefully (producing a NULL Arrow cell) rather than aborting the pipeline. This is an error-handling carve-out for the ingestion path, not a general permission for lenient parsers elsewhere.

The output-type scope qualifier ("needs `Timestamp(µs,UTC)` output") is the operative gate for all new parsing sites. The Wave-A expiry parsing site does not produce `Timestamp(µs,UTC)` output — it produces `u64` Unix seconds. The carve-out and the scope qualifier are independent clauses; both favor the same conclusion.

### Sub-determination 3 — Auth-token-expiry site: outside D5's governed domain

The Wave-A expiry parsing site is outside D5's governed domain on three independent principled grounds:

**Ground 1 — Output type.** `parse_datetime_to_micros` returns `i64` microseconds-since-epoch for Arrow column storage. The Wave-A site returns `u64` Unix seconds for `CachedAuthToken::expires_at`. D5's mandate requires "Timestamp(µs,UTC) output." The Wave-A site produces a TTL integer — a categorically different output type.

**Ground 2 — Domain boundary.** ADR-052 governs the PrismQL data-plane: sensor column normalization, query execution over typed Arrow batches, and predicate pushdown to sensor APIs. ADR-054 governs the control-plane: HTTP auth acquisition, token caching, and TTL management. `DeclarativeHttpAuthProvider::acquire_token()` is a control-plane operation with no Arrow column semantics.

**Ground 3 — Strictness rationale is principled.** Strict `parse_from_rfc3339` for sensor data is required because the OCSF UTC normalization contract demands it. Lenient `DateTime<FixedOffset>::FromStr` for auth expiry is required because Armis Python backend emits space-separated UTC strings that `parse_from_rfc3339` rejects (RU-Q1, ADR-054 §D4 step 4). These are different technical requirements driven by different interface contracts.

### Sub-determination 4 — Relaxed-vs-strict divergence defensibility and future-maintainer trap

The divergence is principled and defensible. However, it creates a **future-maintainer trap**. A developer who:
1. Reads `datetime.rs` module doc: "Do NOT introduce a second RFC-3339 parser... MUST call this function (ADR-052 D5)"
2. Finds `expiry_str.parse::<DateTime<FixedOffset>>()` in `acquire_token()`
3. Concludes this is a second parser site violating D5

...may attempt to unify by calling `parse_datetime_to_micros`. This would produce a catastrophic regression:
- `parse_datetime_to_micros` uses strict `parse_from_rfc3339`, which rejects space-separated Armis forms — breaking RU-Q1 and failing VP-159 AC-6c
- `parse_datetime_to_micros` returns `i64` microseconds (e.g., `4_070_908_800_000_000` for 2099-01-01); using this value as `u64` Unix seconds produces a TTL of approximately 4×10¹⁵ seconds

The module doc's "MUST call this function" language does not include an explicit carve-out naming the auth expiry use case. The output-type qualifier ("needs Timestamp(µs,UTC) output") is precise but requires careful reading. Without an explicit scope-exclusion comment at the expiry parsing site, the module doc's first clause reads as a global prohibition.

This is the signal that (C), not (A), is the correct classification. The ADRs are compatible, but the implementation site needs an explicit annotation to prevent maintainer confusion.

### Sub-determination 5 — Cross-reference gap

ADR-052 `related_adrs` does not list ADR-054. ADR-054 `related_adrs` does not list ADR-052. BC-2.16.014 `§Architecture Anchors` does not reference ADR-052. VP-159 `traces_to` does not reference ADR-052. The two ADRs governing datetime parsing mechanisms within the same crate (`prism-spec-engine`) have mutual silence on each other.

This is a **traceability gap**: a reviewer auditing "which ADRs govern datetime parsing in prism-spec-engine?" finds ADR-052 but is not directed to ADR-054 §D4's lenient-parse decision, and vice versa.

Since the perimeter is frozen, no `related_adrs` amendment is possible in this cycle. The gap does not elevate the classification to (B) — the ADRs are compatible as written; the gap is a maintenance issue, not a semantic conflict. The post-convergence maintenance note in the instruction block below tracks remediation.

### Sub-determination 6 — Deprecated chrono API check

The deprecated chrono API family relevant to Wave-A spec text: `NaiveDateTime::from_timestamp*` (deprecated since 0.4.23/0.4.35, build failure under `-D warnings`) and `TimeZone::timestamp(secs, nsecs)` (deprecated since 0.4.23). `DateTime::timestamp()` (a method on an existing `DateTime` returning `i64` seconds since epoch) is NOT deprecated — it is the inverse direction from the deprecated constructors.

Audit result across all converged Wave-A spec artifacts touching chrono:
- ADR-054 `§D4 step 4`: `s.parse::<DateTime<FixedOffset>>().map(|dt| dt.timestamp() as u64)` — uses `DateTime<FixedOffset>::timestamp()` — **NOT deprecated**
- BC-2.16.014 `§P2` formula: identical pattern — **NOT deprecated**
- VP-159 AC-6/AC-6c/AC-6b and all harness variants: same pattern — **NOT deprecated**
- VP-159 AC-7 (relative seconds): `unix_now() + expires_in.saturating_sub(ttl_buffer_secs)` — no chrono API
- VP-159 full harness (lines 1–1226): wiremock + `Arc<AtomicU64>` clock seam; zero occurrences of `NaiveDateTime::from_timestamp*` or `TimeZone::timestamp`

No deprecated chrono constructor appears in any converged Wave-A spec text. Implementing the spec as written will not produce a build failure under `-D warnings`.

**Sub-finding classification: (A) NO CONFLICT.**

### Primary classification

**(C) — NO CONFLICT BUT STORY-LEVEL CONSTRAINT NEEDED**

ADR-052 §D5 and ADR-054 §D4 are compatible as written. §D5's mandate is scoped to datetime-parsing sites that need `Timestamp(µs,UTC)` Arrow output. The Wave-A auth expiry site produces `u64` Unix-seconds TTL and operates in the control-plane — it is outside D5's governed domain on all three principled grounds above.

The spec perimeter remains FROZEN. No spec artifact requires amendment.

The story-level constraint: the implementer of `DeclarativeHttpAuthProvider::acquire_token()` MUST add the following code comment at the `expiry_str.parse::<DateTime<FixedOffset>>()` site:

```rust
// ADR-052 §D5 SCOPE EXCLUSION: this site is outside the D5 single-parser mandate.
// D5 governs sites that produce Timestamp(µs,UTC) Arrow output (data-plane sensor
// column normalization via parse_datetime_to_micros). This site produces u64
// Unix-seconds TTL for CachedAuthToken::expires_at — a control-plane auth-cache
// computation with no Arrow column semantics.
// Do NOT replace with parse_datetime_to_micros: (a) it uses strict parse_from_rfc3339,
// which rejects space-separated Armis backend expiry strings, breaking RU-Q1 and
// failing VP-159 AC-6c; (b) it returns i64 microseconds, not u64 seconds —
// a catastrophically wrong TTL (~4e15 seconds for a 2099 expiry).
// Lenient parse rationale: ADR-054 §D4 step 4; RU-Q1.
```

Post-convergence maintenance note (non-blocking, no spec action in this cycle): ADR-052 `related_adrs` does not list ADR-054, and ADR-054 `related_adrs` does not list ADR-052. The next non-frozen spec cycle touching either ADR should add the reciprocal cross-reference.

---

## Summary Table

| Question | Ruling (one line) | Classification | Spec artifact that would need amendment if (B) |
|----------|-------------------|----------------|------------------------------------------------|
| Q1 — Rule 9 error mechanism | Rule 9 follows Rule 8: `ESpec027` in `SpecErrorCode` (prism-core), inline in `parse()`, no `SpecEngineError` variants | **(A)** | error-taxonomy.md, BC-2.16.009, error.rs (not needed) |
| Q2 — Fail-fast or all-errors-collected? | Fail-fast in `parse()` per Rule 8 precedent; VP-059 is out of scope for Rule 9 | **(A)** | VP-059 (not needed) |
| Q3 — ADR-031 §D4 cookie-name mandate | Fidelity principle survives; mechanism changes to TOML-declared `header_scheme`; stale comment updated per TD-VSDD-091; no engine allowlist | **(A)** | ADR-031 (not needed) |
| Q4 — `&AuthType` vs `&str` | Take `&AuthType`; exhaustiveness is compiler-enforced for ADR-054 §D7 scope boundary; SAP-3 comment on defense-in-depth test | **(A)** | story T-B02 signature (code-only) |
| Q5 — Ship self-contradictory "Valid values:" | Ship with `token_exchange` now; VP-153 proptest gate requires it; one-cycle UX quirk by design per POL-24 taxonomy-first | **(A)** | VP-153 v0.21, error-taxonomy.md v2.57 (not needed — already executed) |
| Q6 — ADR-052 §D5 vs Wave-A inline expiry parsing (primary) | D5 is data-plane scoped (Timestamp(µs,UTC) output); Wave-A expiry site produces u64 Unix-seconds control-plane TTL — outside D5 domain; story-level comment required to prevent maintainer trap | **(C)** | `datetime.rs` module doc, ADR-052, ADR-054 (not needed — perimeter frozen) |
| Q6 — Deprecated chrono constructors sub-finding | No `NaiveDateTime::from_timestamp*` or `TimeZone::timestamp` in VP-159/BC-2.16.014/ADR-054 §D4; `DateTime::timestamp()` used is NOT deprecated | **(A)** | None |

---

## Consolidated Story-Writer / Implementer Instruction Block

The following changes are required on the CODE-SIDE (story or implementation) without
touching any `.factory/specs/` artifact.

### Changes to story frontmatter
- `crates_touched`: change from `[prism-spec-engine]` to `[prism-spec-engine, prism-core]`

### Changes to "Files to MODIFY" table
- Add row: `crates/prism-core/src/error.rs` — Add `ESpec027` variant to `SpecErrorCode` enum

### Changes to Task T-A01
- **REMOVE:** Add three `SpecEngineError` variants for E-SPEC-027 to `error.rs`
- **REPLACE WITH:** Add one `ESpec027` variant to `SpecErrorCode` enum in `crates/prism-core/src/error.rs` with doc-comment pointing to BC-2.16.009 Rule 9 and the three E-SPEC-027 message templates from error-taxonomy.md v2.66

### Changes to Task T-B02
- Change signature: `validate_header_scheme(sensor_id: &str, header_scheme: Option<&str>, auth_type: &AuthType) -> Result<(), PrismError>`
- The function body constructs `PrismError::Spec(SpecError { code: SpecErrorCode::ESpec027, message: verbatim_template_text, toml_path: Some("sensor.header_scheme".to_string()), file_path: None, line_number: None })` — no `SpecEngineError` intermediary
- The coherence matrix `match auth_type` has exactly 5 arms (one per existing `AuthType` variant) with NO wildcard arm that would silently pass `TokenExchange`

### Changes to Task T-C01
- When rewriting `build_request()`, remove the stale ADR-031 §D4 comment block referencing hardcoded `access_token` name (TD-VSDD-091 anti-volatile-pin). Replace with behavioral anchor referencing header_scheme mechanism and ADR-053 §D2.

### Changes to Task T-A02
- Add the POL-24 / taxonomy-first comment at the `AuthTypeCrossComposition` `#[error(…)]` rewrite site explaining why `token_exchange` appears in "Valid values:" before `VALID_AUTH_TYPES` includes it.

### Changes to test infrastructure (RG-005..010)
- Tests assert `spec_err.message == verbatim_template_text` where `spec_err` is extracted from `PrismError::Spec(spec_err)` — NOT `err.to_string()` (the outer `PrismError::Spec` Display wraps the message)
- Any test exercising Rule A via direct `validate_cross_composition` invocation must carry the SAP-3 rationale comment (defense-in-depth by construction)

### Story-level constraint from Q6 (ADR-052 §D5 scope exclusion)

At the `expiry_str.parse::<DateTime<FixedOffset>>()` site in `DeclarativeHttpAuthProvider::acquire_token()`, add the following code comment verbatim:

```rust
// ADR-052 §D5 SCOPE EXCLUSION: this site is outside the D5 single-parser mandate.
// D5 governs sites that produce Timestamp(µs,UTC) Arrow output (data-plane sensor
// column normalization via parse_datetime_to_micros). This site produces u64
// Unix-seconds TTL for CachedAuthToken::expires_at — a control-plane auth-cache
// computation with no Arrow column semantics.
// Do NOT replace with parse_datetime_to_micros: (a) it uses strict parse_from_rfc3339,
// which rejects space-separated Armis backend expiry strings, breaking RU-Q1 and
// failing VP-159 AC-6c; (b) it returns i64 microseconds, not u64 seconds —
// a catastrophically wrong TTL (~4e15 seconds for a 2099 expiry).
// Lenient parse rationale: ADR-054 §D4 step 4; RU-Q1.
```

This comment is the load-bearing deliverable for the (C) classification. Without it, a future maintainer seeing two `DateTime<FixedOffset>` parse sites in the same crate could attempt to unify them by routing this site through `parse_datetime_to_micros`, producing both a space-separated-form regression (VP-159 AC-6c kill condition) and a wrong-TTL catastrophic failure.

No `.factory/specs/` files are touched. No story frontmatter changes required for Q6.

Post-convergence maintenance note (non-blocking, deferred to next non-frozen spec cycle): add `ADR-054` to ADR-052 `related_adrs`, and add `ADR-052` to ADR-054 `related_adrs`, to close the cross-reference gap surfaced in Q6 sub-determination 5.

### No changes required
- `verification_properties: [VP-153]` frontmatter is correct; VP-059 is not in scope
- `crates/prism-bin/tests/vp153_rule_c_shaped_probe.rs` stays untouched (per ORCHESTRATOR RULING)
- `scripts/check-non-exhaustive.sh` EXPECTED stays at 92 (`ESpec027` is an enum variant, not a new pub struct)
- No spec artifacts under `.factory/specs/` are touched
- VP-159 harness requires no changes; AC-6c already covers the space-separated parse kill condition

---

## Adjudicator Notes

**On the fail-fast vs multi-error question (Q1/Q2):** ADR-053 §D2's phrase "same multi-error pass" is aspirational language in the context of Rule 8 already being fail-fast in `parse()`. The as-built architecture has a known split: `validate_sensor_spec` (multi-error, Rules 1–7) and `parse()` (fail-fast, Rule 8). Rule 9 following Rule 8 (fail-fast) is consistent with the established pattern and is pragmatically correct because Rule 9 governs a single field (at most ONE E-SPEC-027 error per spec) — the multi-error vs fail-fast distinction has no observable impact. Following the multi-error path would require changing the test surface (calling `validate_sensor_spec` instead of `parse()`), contradicting the story's ACs.

**On the SpecEngineError question (Q1):** The story's proposed `SpecEngineError` variants would require a fourth error-plumbing pattern unique to Rule 9. Both existing patterns in `parse()` (cross-composition: `PrismError::Internal` with prefix; Rule 8: `PrismError::Spec` inline) fail the verbatim AC requirement if used as-is. The Rule 8 inline pattern (with `PrismError::Spec` constructed directly) is the only approach that satisfies verbatim message checks without a new pattern. Using `SpecEngineError` for E-SPEC-027 and wrapping it via `PrismError::Internal` would produce messages like "cross-composition validation failed for sensor 'X': sensor 'X' has invalid header_scheme..." — failing verbatim checks.

**On the self-contradiction (Q5):** This is intentional by design. The taxonomy-first discipline (POL-24) means the taxonomy is updated first, then code catches up. VP-153 v0.21 was authored to match the taxonomy, creating a forward state. The code-alignment happens in S-WAVE-A-ENGINE-001; the VALID_AUTH_TYPES runtime constant catches up in S-ADR054-WAVE-A-001. One-cycle contradiction is the expected transient state.

**On the D5 mandate scope qualification (Q6):** The module doc's "Do NOT introduce a second RFC-3339 parser" reads as a global prohibition at first glance, but the qualifying clause — "that needs `Timestamp(µs,UTC)` output" — is load-bearing. Dropping the qualifier would mean that any `DateTime::parse_from_rfc3339` or `FromStr` usage anywhere in the codebase violates D5, which would be an absurd over-reading. The qualifier exists precisely because ADR-052's authors scoped the mandate to the OCSF normalization path, not all chrono usage. The (C) classification preserves this scoping by requiring an explicit comment at the expiry site, rather than silently relying on readers to parse the full mandate clause correctly every time.

**On the (C) vs (A) threshold for Q6:** A finding is (A) if the ADRs are compatible and no annotation is needed for a careful reader. Q6 would be (A) if the module doc's output-type qualifier were obvious to any reader. It is not: the first clause is a flat prohibition; the qualifying clause is in the same sentence but architecturally significant. Given that the wrong path (call `parse_datetime_to_micros`) produces a catastrophic TTL regression, the (A) → (C) uplift is warranted. The conservative path is (C) with a comment; the optimistic path is (A) without one. Under the production-grade default, (C) is correct.

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.1 | 2026-07-24 | architect | Q6 added: ADR-052 §D5 vs Wave-A inline expiry parsing. Primary classification (C) — no spec amendment required; story-level scope-exclusion comment required at `expiry_str.parse` site in `acquire_token()`. Deprecated-chrono sub-finding (A) — no deprecated constructors found in VP-159/BC-2.16.014/ADR-054 §D4. Cross-reference gap noted (mutual silence between ADR-052 and ADR-054 `related_adrs`) — non-blocking maintenance note for next non-frozen cycle. Summary Table, Consolidated Block, Adjudicator Notes, and frontmatter updated. |
| 1.0 | 2026-07-24 | architect | Initial adjudication of Q1–Q5. All (A). Spec perimeter remains frozen. |
