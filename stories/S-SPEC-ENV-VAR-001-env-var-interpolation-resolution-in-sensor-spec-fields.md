---
document_type: story
story_id: S-SPEC-ENV-VAR-001
title: "${env.VAR} interpolation resolution in sensor-spec string fields"
wave: wave-5-e-demo-fidelity
epic_id: E-SPEC-ENGINE
priority: P0
status: draft
version: "1.0"
level: "L4"
producer: story-writer
timestamp: "2026-05-31T00:00:00Z"
tdd_mode: strict
subsystems: [SS-16]
# Subsystem anchor justification:
#   SS-16 (Spec Engine) owns prism-spec-engine, spec_parser, and all spec-load-time
#   validation logic per ARCH-INDEX Subsystem Registry. The ${env.VAR_NAME} token
#   resolver is a post-TOML-parse, pre-URL-format-validation pass that lives entirely
#   within the spec_parser validation pipeline (BC-2.16.009 §Validation Rules 6 AC-6).
#   No other subsystem is touched by this story.
crates_touched: [prism-spec-engine]
target_module: prism-spec-engine
capabilities: [CAP-029]
behavioral_contracts: [BC-2.16.009]
verification_properties: [VP-059]
depends_on: []
# depends_on is empty: this is a leaf prerequisite. The ${env.VAR} resolver has no
# upstream story dependencies — it adds a new validation pass (AC-6) to the
# existing spec_parser pipeline that was already delivered in S-1.11-spec-loading.
blocks: [S-DEMO-ARMIS-AQL-001, S-DEMO-CLAROTY-PAGINATION-001, S-DEMO-CROWDSTRIKE-MULTIREGION-001]
# Dependency anchor justifications:
#   S-DEMO-ARMIS-AQL-001 depends on S-SPEC-ENV-VAR-001 because armis.sensor.toml
#     uses ${env.ARMIS_INSTANCE_URL} in base_url; the parity ACs of ARMIS-AQL-001
#     require the spec engine to load that TOML cleanly, which requires the resolver.
#   S-DEMO-CLAROTY-PAGINATION-001 depends on S-SPEC-ENV-VAR-001 because
#     claroty.sensor.toml uses ${env.CYBERINT_ENVIRONMENT} (partial interpolation)
#     and ${env.CLAROTY_BASE_URL} in base_url; the Claroty fidelity lane is gated on
#     this story delivering the partial-token resolution path.
#   S-DEMO-CROWDSTRIKE-MULTIREGION-001 is hard-gated: the multi-region base_url
#     (${env.CROWDSTRIKE_BASE_URL}) cannot be exercised until the resolver is in place.
points: 5
# Points justification:
#   New validation pass in spec_parser — scanner + regex + std::env::var loop: ~1 pt
#   Multi-error collection (no fail-fast), error struct per token: ~0.5 pt
#   Ordering: insert pass post-TOML-parse, pre-URL-format-validation: ~0.5 pt
#   AD-017 no-value-leak: error construction discipline + tests: ~0.5 pt
#   8 ACs with Red Gate tests (one-per-AC TDD discipline): ~2 pt
#   Sibling-sweep (TD-VSDD-060) — audit 3 canonical TOML specs: ~0.5 pt
#   Total: 5 points (~1.5-2 days)
estimated_days: 2
risk: LOW
# Risk justification:
#   Pure new code path — no existing logic is changed. The resolver is inserted as
#   a new step in an existing pipeline; it does not alter the schema validation pass.
#   The only risk is incorrect ordering (resolver must run before URL-format check)
#   — addressed explicitly in AC-007 and the architecture compliance rules.
acceptance_criteria_count: 8
red_gate_tests: 8
estimated_passes: "2 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "Resolution ordering: AC-007 red gate test asserts that URL-format validation runs
    on the fully-resolved value, not the ${env.VAR} token string. Adversary probes for
    this ordering on every pass."
  - "AD-017 no-value-leak: AC-008 red gate test asserts the E-SPEC-024 error message
    contains the var NAME but NOT the resolved value (or any string that could be the
    value). Adversary probes for value leaks in error messages."
  - "Sibling-sweep: the resolver must scan base_url across all three canonical sensor
    specs that use ${env.VAR} tokens (armis, claroty, cyberint). AC-001 and AC-003 use
    armis as the representative case; the sibling-sweep note in §Architecture Compliance
    Rules requires the scanner to cover all String fields in SensorSpec, not just base_url."
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.009-spec-file-validation.md"
  - ".factory/specs/prd-supplements/error-taxonomy.md"
  - "crates/prism-spec-engine/src/spec_parser.rs"
  - "crates/prism-spec-engine/src/validation.rs"
  - "crates/prism-sensors/specs/armis.sensor.toml"
  - "crates/prism-sensors/specs/claroty.sensor.toml"
  - "crates/prism-sensors/specs/cyberint.sensor.toml"
input-hash: null
traces_to: [BC-2.16.009]
cycle: "v1.0.0-brownfield"
phase: 3
---

# S-SPEC-ENV-VAR-001 v1.0 — `${env.VAR}` Interpolation Resolution in Sensor-Spec String Fields

**Story ID:** S-SPEC-ENV-VAR-001
**Status:** draft
**Version:** v1.0
**Wave:** wave-5-e-demo-fidelity
**Priority:** P0
**Points:** 5

---

## Origin

This story implements AC-6 of BC-2.16.009 v1.6 (§Validation Rules 6: Env Var Token
Resolution), which was added in the same spec burst that authored this story.

Three canonical sensor specs use `${env.VAR_NAME}` tokens in their `base_url` field:
- `armis.sensor.toml`: `base_url = "${env.ARMIS_INSTANCE_URL}"` (full-token)
- `claroty.sensor.toml`: `base_url = "https://${env.CYBERINT_ENVIRONMENT}.cyberint.io"` (partial-token)
- `cyberint.sensor.toml`: `base_url = "${env.CYBERINT_BASE_URL}"` (full-token)

Without this resolver, specs with env-var tokens fail URL-format validation with a
misleading `E-SPEC-001` ("base_url must start with http://") instead of the actionable
`E-SPEC-024` ("env var not set"). The resolver closes this gap.

**Unblocking impact:** This story is a leaf prerequisite that unblocks three demo-fidelity
lanes: S-DEMO-ARMIS-AQL-001 (parity ACs require clean spec load), the Claroty fidelity
lane, and S-DEMO-CROWDSTRIKE-MULTIREGION-001 (hard-gated on this story). It is NOT on
the S-CONFIG → S-DEMO-001 keystone spine.

**Sibling-sweep note (TD-VSDD-060):** Per BC-2.16.009 v1.6 §Validation Rules 6,
`${env.VAR}` patterns are currently present ONLY in `base_url` across the four canonical
sensor specs (armis, claroty, cyberint, crowdstrike). However, the resolver implementation
MUST scan all `String` fields in `SensorSpec` (not just `base_url`) to remain correct for
future specs. The adversary will verify at every pass that the scanner is not limited to
`base_url` only.

---

## Narrative

As the spec engine, I want to scan all string fields in a parsed `SensorSpec` for
`${env.VAR_NAME}` tokens and resolve them against `std::env::var` before any downstream
validation runs, so that operators can put environment-specific values (base URLs,
instance identifiers) in sensor specs without hardcoding them, and so that missing or
empty vars produce an actionable `E-SPEC-024` error instead of a confusing URL-format
error.

---

## Story-Level Goal

After this story merges:

1. The spec_parser validation pipeline includes a new resolution pass that runs after
   TOML deserialization and before URL-format validation.
2. Every `${env.VAR_NAME}` token (where `VAR_NAME` matches `[A-Z0-9_]+`) in every
   `String` field of `SensorSpec` is replaced with `std::env::var(VAR_NAME)`.
3. If the var is absent (`VarError::NotPresent`) or empty (`""`), the resolver emits
   `E-SPEC-024` with the var NAME and TOML path — but NEVER the var VALUE.
4. Multiple unresolvable tokens produce multiple `E-SPEC-024` errors collected in the
   same multi-error pass (no fail-fast). A spec with any unresolved token is rejected
   entirely (fail-closed).
5. Partial interpolation is supported: `"https://${env.CYBERINT_ENVIRONMENT}.cyberint.io"`
   with `CYBERINT_ENVIRONMENT=us1` resolves to `"https://us1.cyberint.io"`, which then
   passes URL-format validation.
6. Tokens using other namespaces (`${step.field}`, `${query.filter.aql}`) are NOT
   touched by this pass — they belong to the runtime interpolation engine (BC-2.16.002).
7. All three canonical sensor specs (armis, claroty, cyberint) load cleanly when their
   respective env vars are set.

---

## Behavioral Contracts

| BC ID | Title | Role in This Story |
|-------|-------|-------------------|
| BC-2.16.009 v1.6 | Spec File Validation — Schema Validation, Variable Reference Resolution, OCSF Field Validation | §Validation Rules 6 (AC-6) is the authoritative contract for `${env.VAR_NAME}` resolution. Defines: token format, error path (E-SPEC-024), success path, multi-error collection, fail-closed semantics, AD-017 no-value-leak, and 7 edge cases (EC-009-001..007). Every AC in this story traces to a clause in this BC. |

---

## Acceptance Criteria

### AC-001: Full-token resolution — single token, var set
`base_url = "${env.ARMIS_INSTANCE_URL}"` with `ARMIS_INSTANCE_URL="https://example.armis.io"`
resolves to `base_url = "https://example.armis.io"`. The resolved URL passes URL-format
validation; the spec loads with no errors.
(traces to BC-2.16.009 §Validation Rules 6 postcondition — "Every `${env.VAR_NAME}` token
in every string field is replaced with the resolved value"; EC-009-003 partial-token
analogue for full-token case)

Red Gate test: `test_env_var_full_token_resolves_to_value`

### AC-002: Partial-token interpolation — token within URL prefix/suffix
`base_url = "https://${env.CYBERINT_ENVIRONMENT}.cyberint.io"` with
`CYBERINT_ENVIRONMENT=us1` resolves to `"https://us1.cyberint.io"`. The literal prefix
`https://` and suffix `.cyberint.io` are preserved exactly. The resolved URL passes URL-format
validation; the spec loads with no errors.
(traces to BC-2.16.009 §Validation Rules 6 — "Partial interpolation is supported within
`base_url`: the pattern ... replaces only the `${env.VAR_NAME}` token, preserving the
surrounding literal string"; EC-009-003)

Red Gate test: `test_env_var_partial_token_resolves_preserving_surrounding_literals`

### AC-003: Multi-token field — two `${env.VAR}` tokens in one field, both set
`base_url = "https://${env.REGION}.${env.SENSOR_HOST}.io"` with `REGION=us1` and
`SENSOR_HOST=example` resolves to `"https://us1.example.io"`. Both tokens are replaced;
the resulting URL passes URL-format validation; spec loads.
(traces to BC-2.16.009 §Validation Rules 6 — resolver scans ALL `${env.VAR_NAME}` tokens
in a field; postcondition: "Every `${env.VAR_NAME}` token in every string field is replaced")

Red Gate test: `test_env_var_multi_token_single_field_both_resolve`

### AC-004: Missing var → E-SPEC-024, spec rejected
`base_url = "${env.ARMIS_INSTANCE_URL}"` with `ARMIS_INSTANCE_URL` not set in the
environment. The resolver emits `E-SPEC-024` with:
- `var_name = "ARMIS_INSTANCE_URL"` (NAME included)
- `toml_path = "sensor.base_url"` (TOML path included)
- No resolved value anywhere in the error (VALUE excluded per AD-017)
The spec is rejected; the error is collected in the multi-error pass (not a panic).
(traces to BC-2.16.009 §Validation Rules 6 error path — "If `VAR_NAME` is absent from
the environment ... → validation error `E-SPEC-024`"; EC-009-001)

Red Gate test: `test_env_var_missing_var_produces_e_spec_024`

### AC-005: Empty var → E-SPEC-024, empty value treated as missing
`base_url = "${env.ARMIS_INSTANCE_URL}"` with `ARMIS_INSTANCE_URL=""`. The resolver treats
the empty string as missing and emits `E-SPEC-024` with `var_name = "ARMIS_INSTANCE_URL"`.
The spec is rejected; no degraded-load state.
(traces to BC-2.16.009 §Validation Rules 6 — "If `VAR_NAME` is present but the value is
empty string (`""`), → validation error `E-SPEC-024` (empty value is treated as missing)";
EC-009-002)

Red Gate test: `test_env_var_empty_var_produces_e_spec_024`

### AC-006: Multi-error collection — multiple unresolvable tokens produce multiple E-SPEC-024
A spec with two fields each containing an unresolvable `${env.VAR}` token (e.g., two
different vars both missing) produces two `E-SPEC-024` errors — one per token — in the same
`ValidationErrors` collection. The resolver does NOT stop at the first failure; it continues
scanning all fields. The spec is rejected after all errors are collected.
(traces to BC-2.16.009 §Validation Rules 6 — "Multiple unresolvable tokens produce multiple
E-SPEC-024 errors, one per token, collected in the same multi-error pass (no fail-fast)";
BC-2.16.009 §Invariants — "Validation is always a single-pass, all-errors-collected operation";
EC-009-006)

Red Gate test: `test_env_var_multi_missing_tokens_collect_multiple_errors`

### AC-007: Resolution ordering — resolver runs post-TOML-parse, pre-URL-format-validation
Given `base_url = "${env.ARMIS_INSTANCE_URL}"` with `ARMIS_INSTANCE_URL` set to a
valid HTTPS URL:
- The raw `${env.ARMIS_INSTANCE_URL}` string is NEVER passed to `url::Url::parse` or the
  `starts_with("http://") / starts_with("https://")` URL-format check.
- The resolved URL (the env var value) IS passed to URL-format validation.
Given `base_url = "${env.ARMIS_INSTANCE_URL}"` with `ARMIS_INSTANCE_URL` NOT set:
- `E-SPEC-024` is emitted (not `E-SPEC-001` "base_url must start with http://").
  The URL-format validation pass never fires on an unresolved token.
(traces to BC-2.16.009 §Validation Rules 6 — "Post-TOML-parse, before URL-format
validation, the resolver scans all string fields"; success path postcondition — "the
resulting string ... is passed to subsequent validation rules"; EC-009-004 analogue)

Red Gate test: `test_env_var_resolution_runs_before_url_format_validation`

### AC-008: AD-017 no-value-leak — E-SPEC-024 message contains var NAME, never var VALUE
Given `base_url = "${env.SECRET_ENDPOINT}"` with `SECRET_ENDPOINT` not set, the `E-SPEC-024`
error message:
- CONTAINS the string `"SECRET_ENDPOINT"` (var name)
- CONTAINS the string `"sensor.base_url"` (TOML path)
- Does NOT contain the value of `SECRET_ENDPOINT` (even if the var is set to a test value
  before the unset scenario, e.g., set to `"https://secret.internal.io"` and then unset —
  the message must not contain `"https://secret.internal.io"`)
Additionally: if the var IS set (happy path), the resolved value does not appear in any
error or warning log emitted during validation — the value is only written into the
`SensorSpec.base_url` field for downstream processing.
(traces to BC-2.16.009 §Validation Rules 6 — "The error message MUST NOT include the
variable VALUE — per AD-017 / AI-opaque-credentials discipline"; E-SPEC-024 taxonomy entry
— "The env var VALUE is NEVER included in the error message")

Red Gate test: `test_env_var_error_contains_name_not_value`

---

## Red Gate Tests

| Test Name | AC | Crate | Description |
|-----------|----|-------|-------------|
| `test_env_var_full_token_resolves_to_value` | AC-001 | prism-spec-engine | Full token ${env.X} with X set → resolved URL; spec loads |
| `test_env_var_partial_token_resolves_preserving_surrounding_literals` | AC-002 | prism-spec-engine | Partial token in URL → only token replaced; literals preserved; spec loads |
| `test_env_var_multi_token_single_field_both_resolve` | AC-003 | prism-spec-engine | Two tokens in one field, both set → both replaced; spec loads |
| `test_env_var_missing_var_produces_e_spec_024` | AC-004 | prism-spec-engine | Missing var → E-SPEC-024 with var name + TOML path; spec rejected |
| `test_env_var_empty_var_produces_e_spec_024` | AC-005 | prism-spec-engine | Empty var → E-SPEC-024; not a URL-format error |
| `test_env_var_multi_missing_tokens_collect_multiple_errors` | AC-006 | prism-spec-engine | Two missing vars → two E-SPEC-024 errors in same pass; no fail-fast |
| `test_env_var_resolution_runs_before_url_format_validation` | AC-007 | prism-spec-engine | Unresolved token → E-SPEC-024, NOT E-SPEC-001; ordering enforced |
| `test_env_var_error_contains_name_not_value` | AC-008 | prism-spec-engine | Error message has var NAME; does not contain the VALUE; AD-017 enforced |

---

## Tasks

### Step 1 — Read context (before writing any code)

1. **Read** `crates/prism-spec-engine/src/spec_parser.rs` — understand `SensorSpec` struct
   field list (all `String` fields that could contain `${env.VAR}` tokens); understand the
   current call sequence: TOML parse → schema validation → URL-format check.
2. **Read** `crates/prism-spec-engine/src/validation.rs` — understand `validate_sensor_spec()`
   function signature; find where URL-format validation fires (the `starts_with("http://")` check
   referenced in E-SPEC-024 taxonomy as `validation.rs:135`); this is the insertion point
   — resolver runs immediately before this check.
3. **Read** `crates/prism-sensors/specs/armis.sensor.toml` — confirm `${env.ARMIS_INSTANCE_URL}`
   in `base_url`; note TOML path is `sensor.base_url`.
4. **Read** `crates/prism-sensors/specs/claroty.sensor.toml` — confirm partial-token pattern
   `"https://${env.CYBERINT_ENVIRONMENT}.cyberint.io"`.
5. **Read** `crates/prism-sensors/specs/cyberint.sensor.toml` — confirm full-token `${env.VAR}`
   in `base_url`.

### Step 2 — Stub the resolver (for Red Gate)

6. **Add** to `spec_parser.rs` (or a new `env_resolver.rs` submodule):
   ```rust
   /// Post-TOML-parse env var token resolver (BC-2.16.009 §Validation Rules 6 / AC-6).
   /// Scans all String fields in SensorSpec for `${env.VAR_NAME}` tokens and resolves them.
   /// Runs after TOML deserialization, before URL-format validation.
   pub fn resolve_env_var_tokens(spec: &mut SensorSpec, file_path: &str)
       -> Vec<SpecEngineError>
   {
       todo!()
   }
   ```
7. **Insert** a `resolve_env_var_tokens` call into `validate_sensor_spec()` (or the spec-load
   path) at the correct ordering point — after `SensorSpec` is deserialized from TOML, before
   the URL-format validation check fires.

### Step 3 — Write Red Gate tests (ALL must FAIL before Step 4)

8. **Write** all 8 Red Gate tests in `crates/prism-spec-engine/tests/` or
   `crates/prism-spec-engine/src/spec_parser.rs::tests` — use inline TOML strings as input;
   set/unset env vars with `std::env::set_var` / `std::env::remove_var` in test setup and
   tear down (use `serial_test` or env-var scoping if needed to avoid test pollution).
9. **Verify RED gate:** `cargo nextest run -p prism-spec-engine --no-fail-fast` — all 8 Red
   Gate tests must FAIL. Do not proceed to Step 4 until RED is confirmed.

### Step 4 — Implement the resolver

10. **Implement** `resolve_env_var_tokens`:
    - Use a regex matching `\$\{env\.([A-Z0-9_]+)\}` to find all tokens in each `String` field
      of `SensorSpec`. The regex may be compiled once via `once_cell::sync::Lazy`.
    - For each match: call `std::env::var(var_name)`.
      - `Ok(value)` and value is non-empty → `str.replace("${env.VAR_NAME}", &value)`.
      - `Ok(value)` and value is empty → push `E-SPEC-024` error for this token (empty = missing).
      - `Err(VarError::NotPresent)` → push `E-SPEC-024` error for this token.
    - Collect all errors; do NOT return early on the first error.
    - `E-SPEC-024` construction: include `var_name` (the NAME) and `toml_path` (the TOML path
      of the field, e.g., `"sensor.base_url"`). Do NOT include `std::env::var(var_name)` or
      any resolved value in the error message.
    - Return the error vec. If non-empty, the caller rejects the spec.
11. **Non-env tokens:** tokens matching `${step.field}`, `${query.*}`, etc. must be left
    untouched. The regex `\$\{env\.([A-Z0-9_]+)\}` is the exact namespace boundary — only
    `${env.VAR}` tokens are resolved by this pass.
12. **String fields to scan:** at minimum, scan `base_url` in `SensorSpec` top-level and in
    per-org overlay `base_url`. Per the sibling-sweep note, the scanner SHOULD iterate all
    `String` fields to remain future-proof. Use reflection (derive macro or explicit field list).

### Step 5 — Verify GREEN

13. **Run** `cargo nextest run -p prism-spec-engine --no-fail-fast` — all 8 Red Gate tests
    and existing tests must be GREEN.
14. **Sibling-sweep (TD-VSDD-060):** run `rg '\$\{env\.' crates/prism-sensors/specs/` to find
    all `${env.VAR}` usages across all TOML specs. Verify each var mentioned maps to a test
    scenario or is covered by existing coverage. No ${env.VAR} usage should be undocumented.
15. **Run** `just check` — final pre-push gate.

---

## File List

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-spec-engine/src/spec_parser.rs` (or `src/env_resolver.rs`) | MODIFY or CREATE | `resolve_env_var_tokens()` function; call site insertion |
| `crates/prism-spec-engine/src/validation.rs` | MODIFY | Insert resolver call at correct ordering point (post-parse, pre-URL-format) |
| `crates/prism-spec-engine/tests/env_var_resolution.rs` (or inline tests) | CREATE or MODIFY | 8 Red Gate tests |

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Resolver runs AFTER TOML deserialization, BEFORE URL-format validation | BC-2.16.009 §Validation Rules 6 AC-6 | AC-007 Red Gate test asserts ordering; adversary probes call sequence on every pass |
| Resolver scans ALL String fields in SensorSpec, not just base_url | BC-2.16.009 §Validation Rules 6 sibling-sweep note (TD-VSDD-060) | Adversary reads resolver implementation and confirms no hardcoded field list limited to base_url |
| Only `${env.VAR_NAME}` namespace resolved; `${step.*}` and `${query.*}` left untouched | BC-2.16.009 §Validation Rules 6 — "Tokens with a different namespace ... are NOT resolved by this pass" | Adversary writes a test with `${step.field}` in a template and verifies it is not modified |
| E-SPEC-024 message contains var NAME + TOML path; never var VALUE | BC-2.16.009 §Validation Rules 6 error path; E-SPEC-024 taxonomy; AD-017 | AC-008 Red Gate test; adversary probes for value leaks on every pass |
| Fail-closed: spec with any unresolved token is REJECTED ENTIRELY | BC-2.16.009 §Validation Rules 6 — "Fail-closed: a spec with any unresolved env tokens is REJECTED ENTIRELY" | ACs 004 and 005; AC-006 multi-error test |
| No fail-fast: all errors collected in single pass | BC-2.16.009 §Invariants — "Validation is always a single-pass, all-errors-collected operation" | AC-006 Red Gate test asserts errors.len() == 2 for two missing vars |
| No `println!` in production code; use `tracing::*!` with structured fields | CLAUDE.md Conventions | New tracing emissions (if any) require BC-2.16.002 catalog row (SAP-1) |
| `SpecEngineError::EnvVarNotSet` (or equivalent E-SPEC-024 variant): var VALUE never in the variant | error-taxonomy.md E-SPEC-024; AD-017 | Adversary inspects SpecEngineError enum definition and error construction sites |

### Forbidden Dependencies

`prism-spec-engine` must NOT gain a new dependency on any `prism-dtu-*` crate. The env
var resolver is pure std + regex — it must not pull in HTTP client, sensor adapter, or DTU
clone crates. If a `once_cell` or `regex` dependency is not already in `prism-spec-engine`'s
`Cargo.toml`, check the workspace manifest — these are workspace-level deps and should not
be added independently.

---

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `regex` | workspace version | `${env.VAR_NAME}` token extraction (pattern `\$\{env\.([A-Z0-9_]+)\}`) |
| `once_cell` | workspace version | `Lazy<Regex>` for compiled-once regex |
| `std::env` | stdlib | `std::env::var(name)` for resolution |

Version source: workspace `Cargo.toml`. Do not pin independently. If `regex` is not already
a dependency of `prism-spec-engine`, verify it is available at the workspace level before
adding it; prefer reusing an existing parsing utility if one exists in the crate.

---

## Previous Story Intelligence

This is the first story in the env-var interpolation feature area. Adjacent stories:

- **S-1.11-spec-loading** (delivered): implemented the core `validate_sensor_spec()` pipeline
  including TOML parse → schema validation → URL-format check. This story inserts a new pass
  into that pipeline. Read `validation.rs` to find the exact insertion point.

- **S-CONFIG-MULTI-TENANT-OVERRIDE-001** (merged PR, recent): added per-org overlay loading.
  EC-009-007 in BC-2.16.009 specifies that overlay `base_url` fields are also subject to env
  var resolution. The resolver must cover overlay `base_url` as well as the TYPE spec `base_url`.
  Read the overlay merge logic to understand when overlays are applied — resolution must run
  after overlay merge, not before.

- **PLUGIN-MIGRATION-001-D** (merged): authored `armis.sensor.toml`, `claroty.sensor.toml`,
  `cyberint.sensor.toml` with `${env.VAR}` tokens in `base_url`. These are the live production
  files that will exercise this story's implementation via the spec-load path in wave 5.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-009-001 | `base_url = "${env.ARMIS_INSTANCE_URL}"` with var not set | E-SPEC-024 with var_name=ARMIS_INSTANCE_URL, toml_path=sensor.base_url; spec rejected |
| EC-009-002 | `base_url = "${env.ARMIS_INSTANCE_URL}"` with var empty (`""`) | E-SPEC-024 (empty treated as missing); spec rejected; not a URL-format error |
| EC-009-003 | `base_url = "https://${env.CYBERINT_ENVIRONMENT}.cyberint.io"` with `CYBERINT_ENVIRONMENT=us1` | Resolves to `"https://us1.cyberint.io"`; spec loads cleanly; URL-format validation passes |
| EC-009-004 | `base_url = "https://${env.CYBERINT_ENVIRONMENT}.cyberint.io"` with var not set | E-SPEC-024 (CYBERINT_ENVIRONMENT); partial URL not constructed; spec rejected |
| EC-009-005 | `base_url = "${env.MY_URL}"` with `MY_URL="ftp://example.com"` | Env var resolves; then E-SPEC-001 (base_url must be http/https) fires on resolved value; spec rejected (E-SPEC-001, not E-SPEC-024) |
| EC-009-006 | Spec has two fields with two different unresolvable env tokens | Two E-SPEC-024 errors (one per token); both in multi-error response; spec rejected |
| EC-009-007 | Per-org overlay has `base_url = "${env.ARMIS_INSTANCE_URL}"` and var not set | E-SPEC-024 with TOML path identifying the overlay field; overlay rejected; TYPE spec base_url NOT substituted as fallback — fail-closed |
| Non-env token | `path_template = "${step.auth.token}"` with no matching env var | Token left untouched by resolver; runtime interpolation engine handles it at fetch time |
| Token namespace collision | Field contains both `${env.HOST}` and `${step.auth.token}` | Only `${env.HOST}` is resolved; `${step.auth.token}` is left verbatim |

---

## Token Budget Estimate

| Context source | Estimated tokens |
|----------------|-----------------|
| This story spec | ~4,500 |
| BC-2.16.009 v1.6 (§Validation Rules 6 + edge cases) | ~2,000 |
| error-taxonomy.md E-SPEC-024 entry | ~500 |
| crates/prism-spec-engine/src/spec_parser.rs | ~4,000 |
| crates/prism-spec-engine/src/validation.rs | ~3,000 |
| crates/prism-sensors/specs/armis.sensor.toml | ~1,500 |
| crates/prism-sensors/specs/claroty.sensor.toml | ~1,000 |
| crates/prism-sensors/specs/cyberint.sensor.toml | ~800 |
| Test files (existing prism-spec-engine/tests/) | ~2,000 |
| Tool outputs (cargo nextest) | ~1,500 |
| **Total estimate** | **~20,800 tokens (~8% of 256K context)** |

Well within the 20-30% budget.

---

## References

- BC-2.16.009 v1.6 §Validation Rules 6 — authoritative contract for this story
- error-taxonomy.md §E-SPEC-024 — error code, message format, AD-017 constraint
- AD-017 — AI-opaque-credentials discipline (var VALUE never in logs/MCP responses)
- TD-VSDD-060 — sibling-site sweep obligation
- SAP-1 — standing adversary probe for tracing emission catalog completeness

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.0 | 2026-05-31 | story-writer | Initial materialization. 8 ACs (all TDD-ready Red Gate), 5 pts, wave-5-e-demo-fidelity, P0, depends_on: []. Grounded against BC-2.16.009 v1.6 §Validation Rules 6 (AC-6 postconditions, error path, EC-009-001..007) and error-taxonomy.md E-SPEC-024 (broken, configuration, message format, AD-017 no-value-leak). Unblocks S-DEMO-ARMIS-AQL-001, Claroty fidelity lane, and S-DEMO-CROWDSTRIKE-MULTIREGION-001 (hard-gated). NOT on S-CONFIG → S-DEMO-001 keystone spine. |
