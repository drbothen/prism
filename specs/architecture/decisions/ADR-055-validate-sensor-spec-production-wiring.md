---
document_type: adr
adr_id: "ADR-055"
title: "Sensor Spec Semantic Validation — Wire validate_sensor_spec() into Production Spec-Loading Pipeline"
status: proposed
date: "2026-07-25"
modified: "2026-07-25"
version: "1.0"
producer: architect
subsystems_affected: [SS-06]
supersedes: null
superseded_by: null
amends: null
anchor_stories: []
related_adrs: [ADR-030]
related_bcs: [BC-2.16.009, BC-2.16.001, BC-2.16.007, BC-2.16.008]
locked_decisions: []
inputs:
  - crates/prism-spec-engine/src/validation.rs
  - crates/prism-spec-engine/src/spec_parser.rs
  - crates/prism-spec-engine/src/add_sensor_spec.rs
  - crates/prism-spec-engine/src/config_manager.rs
  - crates/prism-spec-engine/src/hot_reload.rs
  - .factory/specs/behavioral-contracts/BC-2.16.009-spec-file-validation.md
input-hash: ""
---

# ADR-055: Sensor Spec Semantic Validation — Wire validate_sensor_spec() into Production Spec-Loading Pipeline

## Status

Proposed 2026-07-25, v1.0. Ready for human review and story-writer dispatch.

---

## Context

### Finding (F-WASE-P63-HIGH-003): validate_sensor_spec() has zero production callers

A workspace-wide grep for `validate_sensor_spec(` returns callers only in non-production
code:
- `examples/demo_spec_loading.rs` — demo binary, not production
- `src/proofs/spec_validator.rs` — Kani proof harness
- `tests/bc_2_16_009_test.rs` — unit tests
- `tests/env_var_resolution_tests.rs` — unit tests
- `tests/pipeline_http_integration.rs` — integration tests

**Zero production callers.** No production path invokes `validate_sensor_spec()`.

### Production spec-loading paths verified

Three production surfaces load sensor specs; all were read and verified:

**Surface 1 — startup loading via `SpecLoader::load_all()` (`boot.rs`):**
Calls `SpecLoader::parse()` (TOML deserialization + credential_refs Rule B + timestamp gates
+ Rule 8), then `resolve_env_var_tokens()` (Rule 6), then `validate_step_methods()` (Rule 7).
Does NOT call `validate_sensor_spec()`.

**Surface 2 — hot reload via `process_spec_changes()` (`hot_reload.rs`):**
Calls `parse_and_validate_spec_toml()` for both Added and Modified events. Does NOT call
`validate_sensor_spec()`.

**Surface 3 — add_sensor_spec MCP tool via `add_sensor_spec()` (`add_sensor_spec.rs`):**
Calls `parse_and_validate_spec_toml()`. Does NOT call `validate_sensor_spec()`.

`parse_spec_directory()` in `config_manager.rs` also calls `parse_and_validate_spec_toml()`.
All three non-boot surfaces funnel through `parse_and_validate_spec_toml()`.

### Validations unenforced in production

`validate_sensor_spec()` implements BC-2.16.009 Rules 1–5. Every one of these checks is
unenforced in production today:

| Rule | Check | Example of accepted (wrong) input |
|------|-------|-----------------------------------|
| Rule 1 | `base_url` must start with `http://` or `https://` | `base_url = "ftp://x"` accepted |
| Rule 1 | `version` must be semver-like (N.N.N) | `version = "not-a-version"` accepted |
| Rule 1 | `table_name` must not be empty | `table_name = ""` accepted |
| Rule 1 | Each table must have ≥1 column | Empty `columns` array accepted |
| Rule 1 | Each table must have ≥1 step | Empty `steps` array accepted |
| Rule 1 | Column names must be unique within a table | Duplicate column names accepted |
| Rule 2 | Variable references must resolve to a prior step | `${nonexistent.field}` accepted |
| Rule 2 | No forward references allowed | `${later_step.field}` accepted silently |
| Rule 2b | Multi-array fan-out ambiguity rejected | Two prior-step array sources referenced accepted |
| Rule 3a | `response_path` must be `$.<key>` | `response_path = "$."` accepted |
| Rule 3b | `fan_out_batch_size` must be > 0 | `fan_out_batch_size = 0` accepted until panic |
| Rule 4 | Pagination `page_size` must be > 0 | `page_size = 0` accepted until panic |
| Rule 4 | Cursor pagination `cursor_response_path` must not be empty | `cursor_response_path = ""` accepted |
| Rule 5 | `rate_limit_hints.requests_per_second` must be > 0 | `requests_per_second = -1.0` accepted |

The most severe unenforced gaps are:
- **Rule 1 `base_url` scheme gate**: `ftp://` or bare hostnames are silently accepted.
  EC-009-005 (`base_url` resolves to `not-a-url` → E-SPEC-001) is unreachable from any public
  surface, violating SAP-3.
- **Rule 2 variable reference resolution (E-SPEC-001 / E-SPEC-003)**: a dangling
  `${nonexistent.field}` reference passes spec load and only fails at query runtime —
  producing a confusing runtime error rather than a clear load-time rejection.
- **Rule 3b fan_out_batch_size = 0**: the existing code comment notes that
  `slice::chunks(0)` panics; the guard exists but is never exercised from production load.

### The collect-all invariant vs. parse() single-error signature

BC-2.16.009 §Invariants requires: *"a single-pass, all-errors-collected operation (no
fail-fast on first error)"*. VP-059 scopes this invariant to `validate_sensor_spec()`.

`SpecLoader::parse()` returns `Result<SensorSpec, PrismError>` with fail-fast semantics —
each internal gate does `return Err(...)` on the first violation. This is appropriate for
TOML structural checking (malformed TOML is an unrecoverable format error, not a domain
validation error).

`validate_sensor_spec()` returns `ValidatorOutput = Result<Vec<ValidationWarning>, Vec<SpecError>>`.
The `Vec<SpecError>` carries ALL errors from a single non-fail-fast pass.

`parse_and_validate_spec_toml()` returns `Result<SensorSpec, Vec<types::ValidationError>>`.
`types::ValidationError` has `errors: Vec<String>` — it CAN carry multiple error strings.

The resolution: when wiring `validate_sensor_spec()` into `parse_and_validate_spec_toml()`,
convert the full `Vec<SpecError>` into a single `types::ValidationError` with all error
strings collected. This preserves the collect-all invariant at the surface that matters —
the caller of `parse_and_validate_spec_toml()` receives ALL semantic validation errors at
once, not just the first one.

The `parse()` fail-fast behavior is orthogonal to the collect-all invariant: `parse()`
handles TOML structural concerns; `validate_sensor_spec()` handles domain semantic concerns.
The BC's "single-pass, all-errors-collected" requirement explicitly refers to the semantic
validation pass within `validate_sensor_spec()`, which already implements this correctly.

### Ordering constraint with env-var resolution

`validate_sensor_spec()` checks `base_url` for `http://`/`https://` scheme. Bundled
production specs use env-var tokens in `base_url`:

| Spec | `base_url` field |
|------|-----------------|
| armis | `${env.ARMIS_INSTANCE_URL}` |
| claroty | `${env.CLAROTY_INSTANCE_URL}` |
| crowdstrike | `${env.CROWDSTRIKE_BASE_URL}` |
| cyberint | `https://${env.CYBERINT_ENVIRONMENT}.cyberint.io` |
| customers/acme/armis | `https://armis.acme-corp.io` (hardcoded) |
| customers/contoso/armis | `https://armis.contoso.com` (hardcoded) |

`validate_sensor_spec()` MUST run AFTER `resolve_env_var_tokens()`. If called before env-var
resolution, `${env.ARMIS_INSTANCE_URL}` does not start with `http://` or `https://` and
all three env-var-backed specs would incorrectly fail.

The current `parse_and_validate_spec_toml()` ordering (parse → env-var → HTTP methods) must
therefore be extended as: parse → env-var → HTTP methods → validate_sensor_spec.

### Blast radius — bundled specs

All bundled specs will pass `validate_sensor_spec()` after env-var resolution because:
- All have `version = "1.0.0"` (valid semver)
- `base_url` resolves to `https://...` URLs or the spec is already rejected by env-var
  resolution for a missing environment variable
- Customer specs (`acme/`, `contoso/`) have hardcoded `https://` base URLs
- All have at least one table, one column, and one step per table

The story implementing this MUST verify all bundled specs pass by running the bundled-spec
load tests (`tests/bc_2_16_001_bundled_spec_load.rs`) after wiring.

### BC-2.16.009 intent confirmed

BC-2.16.009 v1.26 explicitly describes `validate_sensor_spec()` and `SpecLoader::parse()` as
"independent entry points invoked separately by callers," implying the caller is responsible
for invoking both. The BC describes `parse_and_validate_spec_toml()` as the integration
function that composes Rules 6 and 7, but does not yet list Rules 1–5 as composed there —
confirming the gap is a production-code omission, not a spec gap.

---

## Decision Drivers

| Driver | Constraint |
|--------|------------|
| BC-2.16.009 §Invariants | All 10 validation rules must run on every spec-load surface |
| SAP-3 (spec-arm reachability) | EC-009-005 (`base_url` scheme rejection) is unreachable from any public surface — must be fixed |
| Production-grade default (CLAUDE.md) | A validator that exists but is never called in production is as bad as no validator |
| Collect-all invariant (VP-059) | All errors from Rules 1–5 must be surfaced together, not one at a time |
| Ordering with Rule 6 | Rules 1–5 must execute after env-var token resolution |
| Blast radius | No bundled specs must newly fail |

---

## Decision

### §D1 — Wire `validate_sensor_spec()` into `parse_and_validate_spec_toml()`

Insert a call to `validate_sensor_spec()` in `parse_and_validate_spec_toml()`, after
`validate_step_methods()` (Rule 7) and before the required-fields non-empty block.

The resulting ordering:
1. `SpecLoader::parse()` — TOML deserialization + credential_refs Rule B + timestamp gates + Rule 8
2. `resolve_env_var_tokens()` — Rule 6 (env-var token resolution; fail-closed per AD-017)
3. `validate_step_methods()` — Rule 7 (HTTP method whitelist)
4. **`validate_sensor_spec()` [NEW]** — Rules 1–5 (schema, variable refs, OCSF warnings,
   pagination, rate limit hints)
5. Required-fields non-empty block — belt-and-suspenders on `sensor_id`, `name`, `version`,
   `base_url` (retaining SEC-001 SENSOR_ID_RE as defense-in-depth for CWE-22)

When `validate_sensor_spec()` returns `Err(spec_errors)`, convert all errors to a single
`types::ValidationError` carrying the full collected list:

```rust
if let Err(spec_errors) = validate_sensor_spec(&spec) {
    return Err(vec![ValidationError {
        sensor_id: Some(spec.sensor_id.clone()),
        source_path: source_path.to_string(),
        errors: spec_errors.into_iter().map(|e| format!("{e}")).collect(),
    }]);
}
```

This preserves BC-2.16.009's collect-all invariant: all Rule 1–5 errors surface to the
caller in a single `ValidationError.errors` vec — not just the first error.

### §D2 — Wire `validate_sensor_spec()` into `SpecLoader::load_all()`

`SpecLoader::load_all()` is called from `boot.rs` (production startup path). It currently
runs Rule 6 and Rule 7 but not Rules 1–5. After `validate_step_methods()` in `load_all()`,
add:

```rust
// BC-2.16.009 Rules 1–5 — semantic validation (post env-var resolution).
// Runs AFTER Rule 6 (env-var) and Rule 7 (HTTP methods). Must run on the
// resolved spec so base_url ${env.VAR} tokens are already substituted.
let semantic_errors = crate::validation::validate_sensor_spec(&spec);
if let Err(sem_errs) = semantic_errors {
    for e in sem_errs {
        errors.push(PrismError::Spec(SpecError {
            code: e.code,
            message: e.message,
            toml_path: e.toml_path,
            file_path: Some(file_name.clone()),
            line_number: None,
        }));
    }
    // DI-030: reject this spec, continue loading others.
    continue;
}
```

The multi-error loop is consistent with how `load_all()` already handles Rule 7 findings.

### §D3 — Rationale: wire into callers, not into `SpecLoader::parse()`

Three alternatives were considered:

**Alternative: fold Rules 1–5 into `SpecLoader::parse()`**
`parse()` returns `Result<SensorSpec, PrismError>` (single error). Threading
`Vec<SpecError>` through this signature would require an API change. More importantly,
`parse()` runs BEFORE env-var resolution, so `validate_sensor_spec()` inside `parse()`
would receive unresolved `${env.VAR}` tokens and incorrectly reject the bundled specs.
`parse()` is also the authoritative TOML deserialization path — merging semantic validation
here conflates two distinct concerns.

**Alternative: fold Rules 1–5 into `validate_step_methods()`**
`validate_step_methods()` is scoped to HTTP method whitelist validation per BC-2.16.009 Rule 7.
Merging Rules 1–5 there conflates five rule categories into one function and violates SRP.

**Chosen: wire into the two call-site integration functions (`parse_and_validate_spec_toml()`
and `load_all()`)**
These are the points where the full resolution pipeline (TOML parse + env-var + domain
validation) is assembled. Adding `validate_sensor_spec()` here respects the BC's documented
architecture of "independent entry points invoked separately by callers." All production
surfaces are covered because:
- All three non-boot surfaces funnel through `parse_and_validate_spec_toml()`
- The boot surface goes through `load_all()` in `boot.rs`

### §D4 — Rationale: ordering after env-var resolution

`validate_sensor_spec()` checks the `base_url` scheme. Bundled production specs use
`${env.ARMIS_INSTANCE_URL}` etc. in `base_url`. If called before env-var resolution, the
unresolved token does not start with `http://` or `https://` and the spec is rejected.
The correct ordering is: env-var resolution (Rule 6) → semantic validation (Rules 1–5).

Calling after Rule 7 (HTTP methods) is also correct — there is no ordering dependency
between Rule 7 and Rules 1–5, and placing it after Rule 7 minimizes disruption to the
existing error-reporting flow.

### §D5 — Rationale: preserve the SEC-001 SENSOR_ID_RE check

`validate_sensor_spec()` calls `validate_sensor_id()`, which checks the same character
constraints as `SENSOR_ID_RE`. However, the explicit SEC-001 SENSOR_ID_RE check in
`parse_and_validate_spec_toml()` carries a documented CWE-22 rationale and is defense-in-depth
Layer 1 against path traversal. It MUST be retained even though `validate_sensor_spec()` now
covers the same character constraint. Removing it would require a security review that is
out of scope for this story.

### §D6 — Warnings from `validate_sensor_spec()`

`validate_sensor_spec()` returns `Ok(warnings)` for specs that have warnings but no errors.
Currently no production caller surfaces these warnings. The implementation should:
- Log each warning at `tracing::warn!` level with `event_type = "spec.validation_warning"`,
  `sensor_id`, `toml_path`, and the warning message.
- A new `event_type = "spec.validation_warning"` must be registered in BC-2.16.002
  §Postconditions (Canonical Structured Event Catalog) per the structured event catalog
  discipline (SAP-1 / PG-LP11-001). This is a BC-2.16.002 amendment obligation.

---

## Story Scope for Story-Writer

This ADR describes a focused production-wiring story. The implementer makes surgical changes
to two functions in one crate.

### Crates affected
- `crates/prism-spec-engine` (sole crate)

### Files changed
- `src/add_sensor_spec.rs` — add `validate_sensor_spec()` call per §D1
- `src/spec_parser.rs` — add `validate_sensor_spec()` call in `load_all()` per §D2

### BCs requiring amendment
- **BC-2.16.009** — §Description "Integration function" paragraph must be updated to reflect
  that `parse_and_validate_spec_toml()` now composes Rules 1–7 (not only Rules 6 and 7);
  `load_all()` paragraph updated similarly. This is a documentation correction, not a
  behavioral change.
- **BC-2.16.002** — Canonical Structured Event Catalog §Postconditions must gain a new
  `event_type = "spec.validation_warning"` row (SAP-1 / §D6 warning surfacing).

### Acceptance shape
1. A sensor spec with `base_url = "ftp://evil.example.com"` submitted via `add_sensor_spec`
   returns `ValidationFailed` with an E-SPEC-001 error citing `sensor.base_url`.
2. EC-009-005 (`base_url` resolves to `not-a-url`) is reachable from the `add_sensor_spec`
   MCP tool and produces the specified E-SPEC-001 error (SAP-3 compliance).
3. A spec with a forward variable reference (`${later_step.field}`) is rejected at load time
   on all three surfaces (startup, hot-reload, add_sensor_spec) with E-SPEC-001 or E-SPEC-003.
4. All bundled specs in `crates/prism-sensors/specs/` load without error (the existing
   `bc_2_16_001_bundled_spec_load.rs` test suite must remain green).
5. Multi-error collect-all behavior: a spec with both a bad `base_url` and a dangling
   variable reference produces TWO errors in the same response, not one.
6. BC-2.16.009 `validate_sensor_spec()` integration test (`bc_2_16_009_test.rs`) exercises
   each check via the `parse_and_validate_spec_toml()` public surface, not by calling
   `validate_sensor_spec()` directly (SAP-3: spec-arm reachability from public surface).

### Co-landing requirement
None. This is a self-contained wiring story with no prerequisite merges. It may be
sequenced independently of Wave-A engine stories.

### Out of scope
- Changing `SpecLoader::parse()`'s signature or internal ordering
- Removing the SEC-001 SENSOR_ID_RE check
- Moving warnings into the MCP response payload (deferred to a separate story if needed)
- Adding new validation rules to `validate_sensor_spec()` beyond what already exists

---

## Consequences

### Architecture Impact

`validate_sensor_spec()` transitions from a tested-but-bypassed function to an active
gatekeeper on every production spec-loading surface. BC-2.16.009 Rules 1–5 become
enforced in production.

`parse_and_validate_spec_toml()` and `load_all()` each gain one new step in their
validation pipelines.

### Behavioral Change: Specs Previously Accepted Will Be Rejected

Any currently-deployed spec that violates Rules 1–5 will be newly rejected after this story
merges. The known categories:

- Specs with non-HTTP/HTTPS `base_url` values (e.g., test fixtures using `localhost:8080`
  without scheme) — must be fixed to `http://localhost:8080`
- Specs with empty table names, missing columns, or missing steps — these indicate corrupt
  specs; rejection is correct
- Specs with dangling variable references — these would have failed at query runtime anyway

Test fixture specs in `tests/` that deliberately construct invalid inputs (e.g., for
testing error cases) may need to be updated if they relied on `parse_and_validate_spec_toml()`
not running Rules 1–5. The implementer must audit all fixtures in
`crates/prism-spec-engine/tests/` before declaring the story complete.

### Existing Test Coverage

`bc_2_16_009_test.rs` directly calls `validate_sensor_spec()`. After this story, those tests
still pass (the function is unchanged). New tests must verify the wiring — i.e., that the
same error conditions trigger correctly via `parse_and_validate_spec_toml()` and
`load_all()`, not only via direct `validate_sensor_spec()` calls.

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-07-25 | architect | Initial proposal from F-WASE-P63-HIGH-003 investigation. |
