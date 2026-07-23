---
document_type: behavioral-contract
level: L3
version: "1.17"
status: active
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-16"
capability: "CAP-029"
lifecycle_status: active
introduced: cycle-1
modified: "2026-07-22"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "fc9d874"
traces_to:
  - "CAP-029"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.16.009: Spec File Validation — Schema Validation, Variable Reference Resolution, OCSF Field Validation

## Description

Spec file validation runs at startup, on reload, and on `add_sensor_spec`. It performs
nine categories of checks in a single pass: schema validation (field types, regex
patterns, enumerations), variable reference resolution (ensuring `${step.field}`
references point to steps that exist and have already executed), OCSF field validation
(against the compiled protobuf schema), pagination configuration consistency, rate
limit hint validity, env-var token resolution (substituting `${env.VAR_NAME}` tokens
in string fields before subsequent validation rules run), HTTP method whitelist
validation (rejecting `step.method` values not in the 7-element allowed set),
`header_scheme` validation (syntactic check against the three-form set plus auth_type
coherence matrix per ADR-053 D2), and `[auth_acquisition]` coherence validation
(block presence requirements, required fields, and type-scoped field restrictions
per ADR-054 D1).

All errors and warnings are collected in a single pass and reported together in a
multi-error format grouped by file, table, and field, including exact TOML paths for
actionable correction. Warnings do not prevent loading; errors do.

## Preconditions
- A sensor spec file has been parsed from TOML into the `SensorSpec` struct (BC-2.16.001)
- Validation runs at startup and on reload (BC-2.16.005) and on `add_sensor_spec` (BC-2.16.008)

## Validation Rules

### 1. Schema Validation
- `sensor_id` must match `^[a-z][a-z0-9_-]*$` — same character set as client_id (BC-2.06.010)
- `name` must be non-empty
- `auth_type` must be one of: `oauth2_client_credentials`, `bearer_static`, `cookie_roundtrip`, `api_key`, `custom_via_plugin`, `token_exchange` — 6-value canonical TARGET set per ADR-054 D1 (`custom_via_plugin` permits external plugin-registered auth strategies per S-PLUGIN-PREREQ-E / ADR-026; `token_exchange` is the 6th variant [PLANNED — engine story]; the as-built `VALID_AUTH_TYPES` in `spec_parser.rs::validate_cross_composition` is currently 5-value and is extended to 6 by the ADR-054 engine story)
- `base_url` must be a valid URL (parsed by `url::Url`)
- `version` must be a valid semver string
- Each table must have a non-empty `table_name` matching `[a-zA-Z0-9_]+`
- Each table must have at least one column
- Each table must have at least one step
- Column names must be unique within a table
- Column types must be one of: `string`, `integer`, `float`, `boolean`, `datetime`, `json`
- Column options must be one of: `REQUIRED`, `INDEX`, `ADDITIONAL`, `HIDDEN`, `OPTIMIZED` (or empty for no options)

### 2. Variable Reference Resolution (DEC-038)
- All `${step_name.field}` references in `path_template` and `body_template` are resolved against the step dependency graph
- A variable reference to a step that does not exist in the same table's steps array: **validation error** `E-SPEC-001` with message "Step '{step_name}' referenced in template but not defined. Available steps: [...]"
- A variable reference to a step that appears AFTER the referencing step (forward reference): **validation error** `E-SPEC-001` with message "Step '{referencing_step}' references '{referenced_step}' which has not executed yet. Steps execute in order."
- Self-references (`${this_step.field}`): **validation error** unless the step explicitly declares the variable in `variables_produced` from a prior execution context

### 3. OCSF Field Validation
- Each `ocsf_field` value is checked against the compiled OCSF protobuf schema
- Invalid OCSF field paths: **warning** (not error) — logged with the column name and invalid path
- Valid OCSF field paths with incompatible types (e.g., mapping a `string` column to an OCSF `int32` field): **warning** — coercion will be attempted at runtime (BC-2.16.003)

### 4. Pagination Configuration Validation
- If pagination type is `cursor_token`, `cursor_response_path` must be a valid JSONPath expression
- If pagination type is `offset_limit`, `page_size` must be > 0
- If pagination type is `none`, no pagination fields should be set (warning if they are)

### 5. Rate Limit Hint Validation
- `requests_per_second` must be > 0 if specified
- `burst_size` must be >= 1 if specified

### 6. Env Var Token Resolution (AC-6)
Post-TOML-parse, before URL-format validation, the resolver scans all string fields for `${env.VAR_NAME}` tokens and resolves them against `std::env::var`.

**Sibling-sweep (TD-VSDD-060):** All four canonical sensor specs (`crates/prism-sensors/specs/`) were audited. As of S-SPEC-ENV-VAR-001, the `${env.VAR}` pattern was used in `base_url` of `armis.sensor.toml`, `claroty.sensor.toml`, `cyberint.sensor.toml`. As of S-DEMO-CROWDSTRIKE-MULTIREGION-001, `crowdstrike.sensor.toml` also uses `${env.CROWDSTRIKE_BASE_URL}` for `base_url` — all four canonical sensor specs now use the env-var pattern for `base_url`. No other string fields (`path_template`, `body_template`, `response_path`, `ocsf_class`, etc.) in the four canonical sensor specs currently use env tokens. The resolver MUST scan `base_url` at minimum; the implementation SHOULD scan all `String` fields in `SensorSpec` to remain correct for future specs. Per-org overlay `base_url` is also in scope (overlays are merged before validation runs).

**Partial interpolation** is supported within `base_url`: the pattern `"https://${env.CYBERINT_ENVIRONMENT}.cyberint.io"` replaces only the `${env.VAR_NAME}` token, preserving the surrounding literal string. After resolution, the full URL is subject to the `starts_with("http://")` / `starts_with("https://")` URL-format validation rule (Validation Rule 1).

**Token format:** `${env.VAR_NAME}` where `VAR_NAME` matches `[A-Z0-9_]+` (uppercase letters, digits, underscores — standard POSIX env var names). Tokens with a different namespace (e.g., `${step.field}`) are NOT resolved by this pass; those belong to the runtime interpolation engine (BC-2.16.002).

**Error path (E-SPEC-024):**
- If `VAR_NAME` is absent from the environment (`std::env::var` returns `VarError::NotPresent`) → validation error `E-SPEC-024`
- If `VAR_NAME` is present but the value is empty string (`""`), → validation error `E-SPEC-024` (empty value is treated as missing)
- The error message MUST include the variable NAME (`VAR_NAME`) and the TOML path (`toml_path`) of the failing field
- The error message MUST NOT include the variable VALUE — per AD-017 / AI-opaque-credentials discipline; credential or instance endpoint values must not appear in logs or MCP error responses
- Multiple unresolvable tokens produce multiple E-SPEC-024 errors, one per token, collected in the same multi-error pass (no fail-fast)
- Fail-closed: a spec with any unresolved env tokens is REJECTED ENTIRELY; it does not load in a degraded state

**Success path:**
- Every `${env.VAR_NAME}` token in every string field is replaced with the resolved value
- The resulting string (with tokens replaced) is passed to subsequent validation rules (URL-format check, etc.)
- If a field contained only the token (e.g., `base_url = "${env.ARMIS_INSTANCE_URL}"`), the resolved value must itself pass URL-format validation

### 7. HTTP Method Whitelist Validation (AC-7) — S-SPEC-HTTP-METHOD-VALIDATION-001

This validation rule runs AFTER the env-var token resolution pass (Rule 6) — environment-
variable-resolved method values (e.g., `${env.SENSOR_STEP_METHOD}` resolving to `"CONNECT"`)
are validated against the whitelist on their RESOLVED string, not on the raw token.

**Whitelist constant:** The following 7 HTTP methods are the complete allowed set:
`GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `HEAD`, `OPTIONS`. This set is expressed as a
compile-time constant `ALLOWED_HTTP_METHODS: &[&str]` in
`crates/prism-spec-engine/src/validation.rs`. The constant is never runtime-configurable.

**Implementation function:** `validate_step_methods(spec: &SensorSpec) -> Vec<(usize, usize, SpecEngineError)>`
is added to `validation.rs` and called post-env-resolver pass in the validation pipeline.
Each tuple entry carries `(table_index, step_index, error)` where `table_index` and `step_index`
are the 0-based `enumerate` indices into `spec.tables` and `table.steps` respectively. Callers
use these indices to construct the canonical `sensor.tables[{table_index}].steps[{step_index}].method`
TOML path in the E-SPEC-025 error without a fragile step-name reverse-lookup (step-name uniqueness
is not enforced by the spec, making name-based lookup unsafe — F-LOCAL-P4-MED-001). Callers that
only need the error values (not the path indices) unwrap with `.map(|(_, _, e)| e)`.

**Validation semantics:**
- If `step.method` is **absent / `None`**: no validation error — absent method defaults to
  GET at the pipeline level (`_=>GET` fallback in `PipelineExecutor`); absence is not invalid.
- If `step.method` is present and its resolved value appears in `ALLOWED_HTTP_METHODS`
  (case-sensitive match): validation passes; no error.
- If `step.method` is present and its resolved value does NOT appear in
  `ALLOWED_HTTP_METHODS` (including wrong case: `"get"` is invalid; empty string `""` is
  invalid; typos like `"GETT"` are invalid; unsupported methods `"CONNECT"` and `"TRACE"`
  are invalid): validation error `E-SPEC-025` is emitted.
- **Case sensitivity:** The whitelist is case-sensitive and upper-case only. `"get"` is
  NOT equivalent to `"GET"`. The implementation MUST NOT silently normalize to upper-case
  before comparison — invalid case is a spec authoring error that should be caught and
  reported explicitly. This matches industry convention for HTTP client implementations.
- **Multi-error collection:** All `step.method` validation errors across all tables and
  all steps in the spec are collected before returning, consistent with INV-ERR-003
  (no fail-fast, same pass as other validation rules).
- **Belt-and-suspenders:** The `_=>GET` fallback in `PipelineExecutor` is NOT removed.
  This rule adds EARLY validation so invalid methods are caught at spec-load time. The
  fallback remains as a safety net for code paths added in future that bypass validation.
- **Error message format (E-SPEC-025):** `"Step '<step_name>' in '<sensor_id>.<table_name>' declares method '<method_value>' which is not a supported HTTP method. Supported: GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS"`
  The method value is safe to echo (it is config text, not a credential per AD-017).
- **32-codepoint echo cap (CWE-400 mitigation, SEC-001):** The `<method_value>` embedded
  in the E-SPEC-025 error message is truncated to a maximum of 32 codepoints via
  `truncate_at_char_boundary(&step.method, 32)` in `validate_step_methods`. A 256 KiB
  TOML file can contain a method string up to that size; echoing it verbatim would produce
  an unbounded (~256 KiB) error response returned to the MCP caller (CWE-400). HTTP
  methods are at most 7 ASCII characters; 32 codepoints is a generous cap that preserves
  full legibility for any reasonable method value. For method values of 32 codepoints or
  fewer the truncated slice equals the original string — Display output is byte-identical
  (POL-24 preserved). This follows the `base_url` 200-codepoint truncation precedent
  (F-LP10-MED-001). Verified by load-bearing test
  `test_BC_2_16_009_sec_001_overlong_method_truncated_in_error`.

**Ordering:** Rule 7 MUST execute after Rule 6 (env-var resolution). Specifically:
1. Rule 6 resolves all `${env.VAR}` tokens; if any token is unresolvable → `E-SPEC-024`
   is emitted for that field and the field's resolved value is undefined. Rule 7 MUST
   skip `step.method` fields that had unresolvable tokens (their method value is undefined
   after a failed Rule 6 resolution). The correct implementation runs Rule 6 first; if
   `E-SPEC-024` fires for a `step.method` field, that field is excluded from Rule 7
   processing (double-reporting the same field is noise, not signal).
2. Rule 7 validates all remaining (successfully-resolved or non-env-token) `step.method`
   field values against the whitelist. Malformed env pseudo-tokens (e.g., `${env.lower}`,
   `${env.foo-bar}`, `${env.}`) — which contain a lowercase or hyphenated `VAR_NAME` that
   does not match the `ENV_TOKEN_REGEX` (`[A-Z0-9_]+`) — are never resolved or errored by
   Rule 6. They fall through to Rule 7's whitelist check and produce `E-SPEC-025` because
   the literal string (e.g., `"${env.lower}"`) is not in `ALLOWED_HTTP_METHODS`. This
   behavior is tested via F-LOCAL-P3-MED-002.

3. **Full-match skip-guard clause (F-PR1-OBS-001):** Rule 7's env-token skip fires ONLY
   when the ENTIRE `step.method` value — after TOML parse, before env resolution — is
   exactly one well-formed `${env.VAR_NAME}` token with `VAR_NAME` matching
   `ENV_TOKEN_REGEX` (`[A-Z0-9_]+`), anchored at both start and end of the string
   (full-string match, not a substring search). A `step.method` value that merely CONTAINS
   a well-formed token but is not exclusively that token does NOT satisfy the skip condition:

   - **Literal prefix + token** (`"GET${env.X}"`) — the method string is not a single token;
     it does not match the full-string `ENV_TOKEN_REGEX` anchor. Rule 6 does not resolve it
     (the leading literal `"GET"` breaks the whole-string token shape). Rule 7 receives the
     raw string `"GET${env.X}"` as its input value; that string is not in
     `ALLOWED_HTTP_METHODS`; `E-SPEC-025` is emitted.
   - **Token + literal suffix** (`"${env.X}GET"`) — same reasoning; partial-token shape
     is not resolved by Rule 6 and not skipped by Rule 7; `E-SPEC-025` is emitted.
   - **Two concatenated tokens** (`"${env.A}${env.B}"`) — the value contains two token
     patterns; neither the combined string nor any sub-string equals a single full-match
     `ENV_TOKEN_REGEX` token. Rule 6 does not resolve it; Rule 7 produces `E-SPEC-025`.

   **Rationale:** When a `step.method` is exactly one well-formed env token AND that token
   is unresolvable, Rule 6 already emitted `E-SPEC-024` for that field — the field's
   effective method value is undefined. Rule 7 skipping it avoids double-reporting the same
   unresolvable field (noise, not signal). For any other `step.method` shape — including
   partial embeddings, concatenated tokens, and malformed pseudo-tokens — the string IS the
   literal method value (Rule 6 never transformed it), and it must be validated against the
   whitelist exactly as written. There is no ambiguity: if the string is not in
   `ALLOWED_HTTP_METHODS`, `E-SPEC-025` is the correct outcome regardless of whether the
   string happens to contain token-like substrings.

   **Tested by:** `test_BC_2_16_009_f_pr1_obs_001_partial_token_embedding_not_skipped`
   (literal prefix `"GET${env.X}"`), `test_BC_2_16_009_f_pr1_obs_001_token_prefix_not_skipped`
   (token + literal suffix `"${env.X}GET"`),
   `test_BC_2_16_009_f_pr1_obs_001_two_tokens_concatenated_not_skipped`
   (`"${env.A}${env.B}"`), and the non-regression
   `test_BC_2_16_009_f_pr1_obs_001_exact_single_token_still_skipped`
   (exact single-token `"${env.SENSOR_METHOD}"` with unset var → Rule 6 fires E-SPEC-024;
   Rule 7 skips — confirming the skip path is not inadvertently broken).

### 8. Probe Table Reference Validation (AC-2 of S-5.04)

This validation rule runs AFTER Rule 7 (HTTP method whitelist), immediately before `Ok(spec)` is returned. It validates the `probe_table` field (introduced per probe-table-field-design.md §1 / D-1260).

**Trigger conditions:**
- If `spec.probe_table` is `None` (absent or not set): no validation is performed; Rule 8 passes silently (back-compat default).
- If `spec.probe_table` is `Some(name)` AND `spec.tables` is non-empty: `name` MUST case-sensitively match the `table_name` of exactly one `[[tables]]` block. Any mismatch → `E-SPEC-026`.
- If `spec.probe_table` is `Some(name)` AND `spec.tables` is empty: unconditionally `E-SPEC-026` — a probe_table reference with no tables to match against is invalid.

**Error code:** `E-SPEC-026` (broken, validation). See §Error Conditions and error-taxonomy.md §E-SPEC-026.

**Error message format (E-SPEC-026):**
```
"sensor '{sensor_id}' declares probe_table = '{name}' but no [[tables]] block
 has table_name = '{name}'. Declared tables: [{table_list}]. Remove probe_table
 or add a matching [[tables]] block."
```
`{table_list}` is the sorted, comma-separated list of `table_name` values from `spec.tables` (empty string `""` when `spec.tables` is empty). `{sensor_id}` and `{name}` are config values, not credentials — safe to echo per AD-017.

**Implementation anchor:** `SpecLoader::parse()` in `crates/prism-spec-engine/src/spec_parser.rs`, Rule 8 block added after the Rule 7 `validate_step_methods` call. `SpecErrorCode::ESpec026` variant added to `prism_core::SpecErrorCode` (`#[non_exhaustive]` enum — additive, no semver break, no new compile-fail gate entry; EXPECTED is ci.yml EXPECTED authority — S-5.04 probe_table adds +0 because ESpec026 is a new variant on a #[non_exhaustive] enum, not a new pub struct).

**TOML path:** `sensor.probe_table`.

**Ordering:** Rule 8 MUST execute after Rule 7. Rule 8 is independent of Rules 1–7 and does not interact with env-var resolution or HTTP method validation.

### 9. `header_scheme` Validation (E-SPEC-027) — Wave-A ADR-053 D2

This validation rule runs AFTER Rule 8 (probe table reference validation). It validates the optional `header_scheme` TOML field. If `header_scheme` is absent from the sensor spec, Rule 9 passes silently (backward-compatible default applies at runtime).

**Syntactic check:**
`header_scheme` must be one of exactly three forms:
- `"bearer"` — inject `Authorization: Bearer {token}` on data-fetch requests
- `"raw"` — inject `Authorization: {token}` with NO "Bearer" prefix (used by sensors whose API rejects the prefix, e.g., Armis Centrix)
- `"cookie:<name>"` — inject `Cookie: <name>={token}`, where `<name>` is the non-empty cookie name substring after `cookie:` (`<name>` must be non-empty and must NOT contain a colon)

Any value that does not match one of these three forms (including `"cookie:"` with empty name, `"cookie:foo:bar"` with a colon in the name, or an arbitrary string) → `E-SPEC-027` (syntactic variant).

**Error message (syntactic):** `"sensor '{sensor_id}' has invalid header_scheme = '{value}'. Valid values: bearer, raw, cookie:<name> (non-empty name required, no colon in name)"`

**Coherence matrix (auth_type ↔ allowed header_scheme):**

| `auth_type` | Allowed `header_scheme` values |
|-------------|-------------------------------|
| `bearer_static` | `"bearer"` or `"raw"` |
| `oauth2_client_credentials` | `"bearer"` or `"raw"` |
| `cookie_roundtrip` | `"cookie:<name>"` ONLY |
| `custom_via_plugin` | `"bearer"` or `"raw"` |
| `api_key` | `"bearer"` only |
| `token_exchange` | `"bearer"` or `"raw"` |

A syntactically valid `header_scheme` that is incoherent with the declared `auth_type` → `E-SPEC-027` (coherence variant). Example: `auth_type = "cookie_roundtrip"` with `header_scheme = "bearer"` → E-SPEC-027 coherence.

**Error message (coherence):** `"sensor '{sensor_id}': auth_type = '{auth_type}' does not permit header_scheme = '{value}'; allowed for this auth_type: {allowed_set}"`

**Ordering:** Rule 9 runs after Rule 8 and before Rule 10. It is independent of Rules 1–8.

### 10. `[auth_acquisition]` Coherence Validation (E-SPEC-028) — Wave-A ADR-054 D1

This validation rule runs AFTER Rule 9 (`header_scheme` validation). It validates the optional `[auth_acquisition]` TOML sub-table when present, and checks required-block obligations for declarative auth types. All sub-conditions are checked in a single pass (no fail-fast); all errors are collected.

**Sub-conditions (8 checks):**

**(a) Declarative auth_types require `[auth_acquisition]` with `token_path`:** If `auth_type ∈ {oauth2_client_credentials, token_exchange}` and either (i) no `[auth_acquisition]` block is present, or (ii) `[auth_acquisition]` is present but `token_path` is absent → `E-SPEC-028`. Both declarative auth types require `token_path` to derive the per-org token URL at boot step 9A (ADR-054 §D10(a)).

**(b) `auth_plugin` on declarative `auth_type` (Definition 1, ratified in ADR-054 v0.35 §D10(b)):** If `auth_type ∈ {oauth2_client_credentials, token_exchange}` and `auth_plugin` is present in the sensor spec → `E-SPEC-028`, regardless of whether `[auth_acquisition]` is declared. These auth_types use the native `DeclarativeHttpAuthProvider`; `auth_plugin` serves no role in declarative auth flows and its presence is rejected to prevent silent misconfiguration (ADR-054 §D5/D10(b)). Message template: `"sensor '{sensor_id}': auth_type = '{auth_type}' uses native declarative provider and does not accept auth_plugin. Remove auth_plugin or change auth_type to custom_via_plugin."` **Disjointness with (g):** (b) covers `auth_type ∈ {oauth2_client_credentials, token_exchange}` (declarative types); (g) covers `auth_type ∈ {bearer_static, cookie_roundtrip, api_key, custom_via_plugin}` (non-declarative types). These sets are disjoint — a given sensor spec triggers at most one of (b) or (g), never both.

**(c) Invalid `expiry_mode`:** If `[auth_acquisition]` is present and `expiry_mode` is set, its value must be one of: `"absolute_utc_string"` or `"relative_seconds"`. Any other value → `E-SPEC-028` citing the invalid `expiry_mode` value.

**(d) `token_exchange` missing required `[auth_acquisition]` fields:** If `auth_type = "token_exchange"` and `[auth_acquisition]` is present, each of `credential_body_field`, `token_response_path`, `expiry_field`, and `expiry_mode` MUST be present. One `E-SPEC-028` is emitted per absent field, citing the field name (ADR-054 §D10(d)). `ttl_buffer_secs` is OPTIONAL with a default of 30 seconds (ADR-054 §D3); a `token_exchange` spec that omits `ttl_buffer_secs` is VALID and passes Rule 10(d) — the ADR-054 §D3 Armis wiring example omits it.

**(e) `credential_body_field` not in `[[credential_refs]]`:** If `[auth_acquisition]` declares `credential_body_field = "<name>"` and no `[[credential_refs]]` block has `name = "<name>"` → `E-SPEC-028`. The credential body field must reference a declared credential ref.

**(f) `oauth2_client_credentials` missing `client_id` or `client_secret` credential refs:** If `auth_type = "oauth2_client_credentials"` and one or both of `client_id`, `client_secret` entries are absent from `[[credential_refs]]` → `E-SPEC-028`. A spec with no `[[credential_refs]]` at all, or one with only `client_id` declared but missing `client_secret` (or vice versa), is rejected. `{field_list}` = comma-separated list of the absent ref names (e.g., `"client_secret"` when only `client_id` is declared, `"client_id, client_secret"` when neither is declared). ADR-054 §D10(f).

**(g) `[auth_acquisition]` on non-declarative auth_type:** If `[auth_acquisition]` is present and `auth_type` is one of `bearer_static`, `cookie_roundtrip`, `api_key`, or `custom_via_plugin` → `E-SPEC-028`. Only `token_exchange` and `oauth2_client_credentials` (native `DeclarativeHttpAuthProvider` variants) support `[auth_acquisition]`; the other auth_types do not use it.

**(h) `token_exchange`-only fields on non-`token_exchange` auth_type:** If `[auth_acquisition]` is present and any of the `token_exchange`-specific fields (`credential_body_field`, `token_response_path`, `expiry_field`, `expiry_mode`) are set, but `auth_type != "token_exchange"` → a single `E-SPEC-028` citing all offending field names as `{field_list}` (comma-separated aggregated emission — not one error per field; cardinality ratified in ADR-054 v0.35 §D10(h)).

**Ordering:** Rule 10 MUST execute after Rule 9. Rule 10 is independent of Rules 1–9 (it checks block-level coherence, not intra-field syntax).

## Postconditions
- If any errors are found: the spec is rejected and the error list is returned
- If only warnings are found: the spec loads successfully and warnings are logged at startup and included in reload results
- If no issues: the spec loads cleanly

## Multi-Error Reporting
- All validation errors and warnings are collected in a single pass
- Errors are grouped by spec file, then by table, then by field
- Each error includes the exact TOML path (e.g., `sensor.tables[0].steps[1].path_template`) for actionable correction
- Warnings do not prevent the spec from loading; errors do

## Invariants
- Validation is always a single-pass, all-errors-collected operation (no fail-fast on first error)
- A spec with any errors is never loaded or written to disk
- Warnings are reported but never block loading

## Error Conditions
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-SPEC-001` | Schema or variable reference validation error (with TOML path and corrective guidance) | Spec rejected; all errors reported together |
| `E-SPEC-001` | TOML parse error (syntax error in the file) | Spec rejected; parse error with line number |
| `E-SPEC-002` | Invalid column type for a column in a sensor table (type not in: string, integer, float, boolean, datetime, json) | Spec rejected; error message includes sensor_id, table_name, column_name, and the invalid type value |
| `E-SPEC-003` | Undefined variable reference `${step.field}` — step name does not exist, or forward reference to a step that has not yet executed | Spec rejected; error message names both the referencing step and the undefined step; forward-reference message distinguishes "not defined" from "not yet executed" |
| `E-SPEC-009` | Duplicate `sensor_id` across spec files | Second file rejected; first wins |
| `E-SPEC-004` | Duplicate table_name within a sensor | Spec file rejected entirely |
| `E-SPEC-024` | `${env.VAR_NAME}` token in a string field (e.g., `base_url`) references an env var that is absent or empty at spec-load time | Spec rejected; error message includes var NAME and TOML path; var VALUE never included (AD-017); multiple tokens → multiple errors in same multi-error pass |
| `E-SPEC-025` | `step.method` value (after env-var resolution) is not in the allowed HTTP method set (`GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `HEAD`, `OPTIONS`) | Spec rejected; error message includes step_name, sensor_id, table_name, and the invalid method value (method value is config text, not a credential per AD-017); multiple invalid steps → multiple E-SPEC-025 errors in same multi-error pass; absent `step.method` is NOT an error (defaults to GET at pipeline level) |
| `E-SPEC-026` | `probe_table` is set (`Some(name)`) but `name` does not case-sensitively match any `table_name` in `[[tables]]`, OR `probe_table` is set but `spec.tables` is empty | Spec rejected; error message includes `sensor_id`, `probe_table` value, and sorted list of declared table names (empty list `""` when no tables); `{sensor_id}` and `{name}` are config values, not credentials (AD-017); emitted by `SpecLoader::parse()` Rule 8; `probe_table = None` (absent) is NOT an error — Rule 8 passes silently |
| `E-SPEC-027` | `header_scheme` value is syntactically invalid (not `"bearer"`, `"raw"`, or `"cookie:<name>"` with non-empty name and no colon in name) OR is syntactically valid but incoherent with the declared `auth_type` (e.g., `auth_type = "cookie_roundtrip"` with `header_scheme = "bearer"`) | Spec rejected; syntactic error message includes `sensor_id`, `header_scheme` value, and corrective guidance; coherence error message includes `sensor_id`, `auth_type`, `header_scheme` value, and allowed set for that auth_type; `header_scheme` absent is NOT an error (backward-compatible default) |
| `E-SPEC-028` | `[auth_acquisition]` block coherence violation — any of 8 sub-conditions: (a) `auth_type ∈ {oauth2_client_credentials, token_exchange}` with block absent OR `token_path` absent; (b) `auth_type ∈ {oauth2_client_credentials, token_exchange}` AND `auth_plugin` present (block presence not required — Definition 1, ADR-054 v0.35 §D10(b)); (c) invalid `expiry_mode`; (d) `token_exchange` missing any of `{credential_body_field, token_response_path, expiry_field, expiry_mode}` — one error per absent field; `ttl_buffer_secs` is optional; (e) `credential_body_field` not in `[[credential_refs]]`; (f) `oauth2_client_credentials` missing `client_id` or `client_secret` credential refs (one or both absent); (g) block on non-declarative auth_type; (h) `token_exchange`-only fields on non-`token_exchange` auth_type — single aggregated `{field_list}` emission | Spec rejected; error message includes `sensor_id`, specific sub-condition identifier, field name(s), and corrective guidance; all sub-condition errors collected in single multi-error pass |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-038 | Forward variable reference | `E-SPEC-001` with message identifying referencing and referenced steps |
| Warning-only | invalid ocsf_field paths (not in compiled schema) | Spec loads; warnings logged; runtime falls back to raw_extensions |
| Multiple errors | 3 schema errors + 2 variable errors in one file | All 5 reported in single response; spec rejected |
| Empty table | table with no columns | `E-SPEC-001`: "Table must have at least one column" |
| EC-009-001 | `base_url = "${env.ARMIS_INSTANCE_URL}"` with `ARMIS_INSTANCE_URL` not set | `E-SPEC-024` with message citing `ARMIS_INSTANCE_URL` (name only, no value); TOML path `sensor.base_url`; spec rejected |
| EC-009-002 | `base_url = "${env.ARMIS_INSTANCE_URL}"` with `ARMIS_INSTANCE_URL=""` (empty string) | `E-SPEC-024` (empty value treated as missing); spec rejected |
| EC-009-003 | `base_url = "https://${env.CYBERINT_ENVIRONMENT}.cyberint.io"` with `CYBERINT_ENVIRONMENT=us1` | Resolves to `"https://us1.cyberint.io"`; URL-format validation passes; spec loads |
| EC-009-004 | `base_url = "https://${env.CYBERINT_ENVIRONMENT}.cyberint.io"` with `CYBERINT_ENVIRONMENT` not set | `E-SPEC-024` with message citing `CYBERINT_ENVIRONMENT`; partial URL not constructed; spec rejected |
| EC-009-005 | `base_url = "${env.ARMIS_INSTANCE_URL}"` with `ARMIS_INSTANCE_URL="not-a-url"` (no http/https prefix) | Env var resolves successfully; then URL-format validation (`E-SPEC-001`: "base_url must start with http:// or https://") fires on the resolved value |
| EC-009-006 | Spec has two fields with unresolvable env tokens | Two `E-SPEC-024` errors emitted (one per token); both included in the multi-error response; spec rejected |
| EC-009-007 | Per-org overlay has `base_url = "${env.ARMIS_INSTANCE_URL}"` and var is not set | `E-SPEC-024` emitted with TOML path `base_url` and file path identifying the overlay file; overlay rejected; TYPE spec's `base_url` is NOT substituted as fallback — fail-closed |
| EC-009-008 | `crowdstrike.sensor.toml` `base_url = "${env.CROWDSTRIKE_BASE_URL}"` with `CROWDSTRIKE_BASE_URL` set to eu-1 URL (`https://api.eu-1.crowdstrike.com`) | Env var resolves; `base_url` = `"https://api.eu-1.crowdstrike.com"`; URL-format validation passes; spec loads. Demonstrates sensor-agnosticism: same resolver handles any CrowdStrike region URL (us-1, us-2, eu-1, gov) — S-DEMO-CROWDSTRIKE-MULTIREGION-001 |
| EC-009-009 | `crowdstrike.sensor.toml` `base_url = "${env.CROWDSTRIKE_BASE_URL}"` with `CROWDSTRIKE_BASE_URL` not set | `E-SPEC-024` with message citing `CROWDSTRIKE_BASE_URL` (name only, no value); TOML path `sensor.base_url`; spec rejected; fail-closed — S-DEMO-CROWDSTRIKE-MULTIREGION-001 |
| EC-009-010 | `step.method = "GET"` — valid uppercase | Passes Rule 7; no error; spec loads |
| EC-009-011 | `step.method = "POST"` — valid, common for POST-for-read sensors (Claroty, Armis) | Passes Rule 7; no error |
| EC-009-012 | `step.method = "CONNECT"` — unsupported | `E-SPEC-025` with step_name, sensor_id, table_name, and `"CONNECT"` in message |
| EC-009-013 | `step.method = "TRACE"` — unsupported | `E-SPEC-025` with `"TRACE"` in message |
| EC-009-014 | `step.method = "GETT"` — typo | `E-SPEC-025` with `"GETT"` in message |
| EC-009-015 | `step.method = "get"` — wrong case (lowercase) | `E-SPEC-025` (case-sensitive; `"get"` is not in the whitelist) |
| EC-009-016 | `step.method = ""` — empty string | `E-SPEC-025` (empty string is not in the whitelist) |
| EC-009-017 | `step.method` absent (field not present in TOML) | No error from Rule 7; absent defaults to GET at pipeline level; this is valid and expected |
| EC-009-018 | Two steps in same spec with invalid methods (`"CONNECT"` + `"TRACE"`) | Two `E-SPEC-025` errors collected; both in same multi-error response; spec rejected |
| EC-009-019 | `step.method = "${env.SENSOR_METHOD}"` with `SENSOR_METHOD="CONNECT"` | Rule 6 resolves to `"CONNECT"`; Rule 7 fires `E-SPEC-025` on resolved value `"CONNECT"` |
| EC-009-020 | `step.method = "${env.SENSOR_METHOD}"` with `SENSOR_METHOD` unset | Rule 6 fires `E-SPEC-024` for the unresolved token; Rule 7 SKIPS this step (method value undefined after Rule 6 failure; double-reporting is noise) |
| EC-009-021 | `step.method` set to a string longer than 32 codepoints (e.g., a 256-char garbage value in a malformed TOML) | `E-SPEC-025` is emitted; the `method_value` field in the error message is truncated at 32 codepoints via `truncate_at_char_boundary` (CWE-400 unbounded-echo mitigation, SEC-001). For inputs ≤32 codepoints the echoed value is byte-identical to the original. Tested by `test_BC_2_16_009_sec_001_overlong_method_truncated_in_error`. |
| EC-009-022 | `step.method = "GET${env.X}"` — literal prefix concatenated with a well-formed env token (partial embedding, F-PR1-OBS-001) | The full-match skip-guard clause does NOT apply: `"GET${env.X}"` is not exclusively one well-formed token (anchored full-string match fails). Rule 6 does not resolve it. Rule 7 receives `"GET${env.X}"` as the raw method value; it is not in `ALLOWED_HTTP_METHODS`; `E-SPEC-025` is emitted. Spec rejected. Tested by `test_BC_2_16_009_f_pr1_obs_001_partial_token_embedding_not_skipped`. |
| EC-009-023 | `step.method = "${env.X}GET"` — well-formed env token concatenated with a literal suffix (partial embedding, F-PR1-OBS-001) | Same reasoning as EC-009-022: full-string anchor fails; Rule 6 does not resolve; Rule 7 produces `E-SPEC-025` on the raw string `"${env.X}GET"`. Spec rejected. Tested by `test_BC_2_16_009_f_pr1_obs_001_token_prefix_not_skipped`. |
| EC-009-024 | `step.method = "${env.A}${env.B}"` — two concatenated well-formed env tokens (F-PR1-OBS-001) | The combined string is not a single full-string `ENV_TOKEN_REGEX` match. Rule 6 does not resolve the concatenated form. Rule 7 produces `E-SPEC-025` on `"${env.A}${env.B}"`. Spec rejected. Tested by `test_BC_2_16_009_f_pr1_obs_001_two_tokens_concatenated_not_skipped`. |
| EC-009-025 | `step.method = "${env.SENSOR_METHOD}"` (exact single well-formed token) with `SENSOR_METHOD` unset — non-regression for skip path (F-PR1-OBS-001) | Full-string match succeeds; the value IS exactly one well-formed token. Rule 6 fires `E-SPEC-024` (var unset). Rule 7 SKIPS this step (double-reporting avoidance). Only one error emitted: `E-SPEC-024`. Spec rejected. Tested by `test_BC_2_16_009_f_pr1_obs_001_exact_single_token_still_skipped`. |
| EC-009-026 | `probe_table = "alerts"` and spec declares `[[tables]]` with `table_name = "alerts"` (valid match, Rule 8 happy path) | Rule 8 passes; no E-SPEC-026 emitted; spec loads. |
| EC-009-027 | `probe_table = "devices"` and spec declares `[[tables]]` with `table_name = "alerts"` only (name-not-found) | Rule 8 fires `E-SPEC-026`; error message includes `sensor_id`, `probe_table = "devices"`, and `Declared tables: [alerts]`; spec rejected. |
| EC-009-028 | `probe_table = "devices"` and spec has no `[[tables]]` blocks (empty table list) | Rule 8 fires `E-SPEC-026`; error message includes `sensor_id`, `probe_table = "devices"`, and `Declared tables: []`; spec rejected. |
| EC-009-029 | `header_scheme = "raw"` with `auth_type = "token_exchange"` (e.g., Armis pattern) | Rule 9 passes: `"raw"` is syntactically valid; coherence check: `token_exchange` allows `"bearer"` or `"raw"` → passes. Spec loads. |
| EC-009-030 | `header_scheme = "cookie:access_token"` with `auth_type = "cookie_roundtrip"` (e.g., Cyberint pattern) | Rule 9 passes: `"cookie:access_token"` is syntactically valid (non-empty name, no colon in name); coherence check: `cookie_roundtrip` allows `"cookie:<name>"` → passes. Spec loads. |
| EC-009-031 | `header_scheme = "bearer"` with `auth_type = "cookie_roundtrip"` — coherence violation | Rule 9 syntactic check passes (`"bearer"` is valid form); coherence check: `cookie_roundtrip` requires `"cookie:<name>"` ONLY; `"bearer"` is not allowed → `E-SPEC-027` coherence variant; spec rejected. |
| EC-009-032 | `header_scheme = "cookie:"` — empty cookie name | Rule 9 syntactic check fails: `<name>` after `cookie:` is empty → `E-SPEC-027` syntactic variant; spec rejected. |
| EC-009-033 | `header_scheme = "cookie:foo:bar"` — colon in cookie name | Rule 9 syntactic check fails: `<name>` contains colon → `E-SPEC-027` syntactic variant; spec rejected. |
| EC-009-034 | `auth_type = "token_exchange"` with no `[auth_acquisition]` block | Rule 10(a) fires `E-SPEC-028`; spec rejected. |
| EC-009-035 | `auth_type = "token_exchange"` with `[auth_acquisition]` present but `token_path` absent | Rule 10(a) fires `E-SPEC-028` citing missing `token_path` (token_path absence is a 10(a) condition, not 10(d)); spec rejected. |
| EC-009-036 | `auth_type = "oauth2_client_credentials"`, `auth_plugin = "crowdstrike-oauth2"` present, with or without `[auth_acquisition]` (Definition 1 — ADR-054 v0.35 §D10(b)) | Rule 10(b) fires `E-SPEC-028` — `auth_type` is a declarative type and `auth_plugin` coexists; spec rejected. Message: `"sensor '<id>': auth_type = 'oauth2_client_credentials' uses native declarative provider and does not accept auth_plugin..."` If `auth_type` were `custom_via_plugin` instead, Rule 10(b) would NOT fire (non-declarative type). |
| EC-009-037 | `[auth_acquisition]` present with `auth_type = "bearer_static"` | Rule 10(g) fires `E-SPEC-028`; spec rejected. |
| EC-009-038 | `expiry_mode = "absolute_utc_string"` in `[auth_acquisition]` | Rule 10(c) passes: `"absolute_utc_string"` IS in `{absolute_utc_string, relative_seconds}` (valid set per ADR-054 v0.33 D10(c)); spec loads (subject to other Rule 10 checks passing). Consistency conflict with BC-2.01.008 resolved by ADR-054 v0.33 D10(c) adjudication. |
| EC-009-039 | `auth_type = "oauth2_client_credentials"` with no `[auth_acquisition]` block | Rule 10(a) fires `E-SPEC-028` — `oauth2_client_credentials` requires `[auth_acquisition]` with `token_path` (same condition as token_exchange); spec rejected. |
| EC-009-040 | `auth_type = "token_exchange"` with `[auth_acquisition]` present including `token_path` but missing `token_response_path` | Rule 10(d) fires `E-SPEC-028` citing missing `token_response_path`; spec rejected. Verifies the four-field check in 10(d): a TOML omitting any of `{credential_body_field, token_response_path, expiry_field, expiry_mode}` is rejected even when `token_path` is present (which only satisfies 10(a)). |
| EC-009-041 | `auth_type = "oauth2_client_credentials"`, `[[credential_refs]]` declares `client_id` only — no `client_secret` entry | Rule 10(f) fires `E-SPEC-028`; `{field_list}` = `"client_secret"`; spec rejected. Demonstrates that a partially-declared credential set is rejected — both `client_id` AND `client_secret` must appear in `[[credential_refs]]`. A spec with zero `[[credential_refs]]` blocks would produce `{field_list}` = `"client_id, client_secret"`. |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — valid spec | well-formed TOML; all fields valid | Spec loads; no errors; no warnings |
| Schema error | `sensor_id: "123-invalid"` (starts with digit) | `E-SPEC-001` with TOML path `sensor.sensor_id` |
| Forward variable reference | step 2 references step 3 | `E-SPEC-001` with message identifying forward reference |
| Invalid OCSF field | `ocsf_field: "nonexistent.field"` | Warning logged; spec loads; field goes to raw_extensions at runtime |
| Multiple errors | invalid sensor_id + forward reference | Both errors reported together; spec rejected |
| Env var — missing var | `base_url = "${env.ARMIS_INSTANCE_URL}"`, `ARMIS_INSTANCE_URL` unset | `E-SPEC-024` message includes `ARMIS_INSTANCE_URL` (name); does NOT include any resolved value; TOML path is `sensor.base_url` |
| Env var — empty var | `base_url = "${env.ARMIS_INSTANCE_URL}"`, `ARMIS_INSTANCE_URL=""` | `E-SPEC-024` (empty treated as missing) |
| Env var — partial interpolation success | `base_url = "https://${env.CYBERINT_ENVIRONMENT}.cyberint.io"`, `CYBERINT_ENVIRONMENT=us1` | Resolved: `"https://us1.cyberint.io"`; spec loads (URL-format valid) |
| Env var — value set but invalid URL | `base_url = "${env.MY_URL}"`, `MY_URL="ftp://example.com"` | Env var resolves; then `E-SPEC-001` (base_url not http/https); value `ftp://example.com` may appear in E-SPEC-001 message (not a credential; plain URL); spec rejected |
| HTTP method — valid GET | `step.method = "GET"` | No E-SPEC-025; spec loads |
| HTTP method — valid POST (Claroty pattern) | `step.method = "POST"` | No E-SPEC-025; spec loads |
| HTTP method — CONNECT rejected | `step.method = "CONNECT"` | `E-SPEC-025` citing step_name, sensor_id, table_name, `"CONNECT"`; spec rejected |
| HTTP method — lowercase rejected | `step.method = "get"` | `E-SPEC-025` (case-sensitive whitelist); spec rejected |
| HTTP method — env-resolved invalid | `step.method = "${env.M}"`, `M="TRACE"` | Rule 6 resolves to `"TRACE"`; Rule 7 fires `E-SPEC-025` on resolved value; spec rejected |
| HTTP method — absent (no step.method) | step has no `method` field | No E-SPEC-025; spec loads; pipeline defaults to GET |
| HTTP method — overlong (>32 codepoints) | `step.method` = 33-character garbage string | `E-SPEC-025` emitted; `method_value` in error message is truncated to first 32 codepoints (SEC-001 / CWE-400 mitigation); spec rejected |
| HTTP method — partial embedding, literal prefix (F-PR1-OBS-001, EC-009-022) | `step.method = "GET${env.X}"` | Full-match skip-guard does NOT apply; Rule 6 does not resolve partial embedding; Rule 7 receives raw string `"GET${env.X}"`; `E-SPEC-025` emitted; spec rejected |
| HTTP method — partial embedding, literal suffix (F-PR1-OBS-001, EC-009-023) | `step.method = "${env.X}GET"` | Full-match skip-guard does NOT apply; Rule 7 receives raw string `"${env.X}GET"`; `E-SPEC-025` emitted; spec rejected |
| HTTP method — two concatenated tokens (F-PR1-OBS-001, EC-009-024) | `step.method = "${env.A}${env.B}"` | Full-match skip-guard does NOT apply; Rule 7 receives raw string `"${env.A}${env.B}"`; `E-SPEC-025` emitted; spec rejected |
| HTTP method — exact single token, var unset, skip path non-regression (F-PR1-OBS-001, EC-009-025) | `step.method = "${env.SENSOR_METHOD}"`, `SENSOR_METHOD` unset | Full-match skip-guard applies (exclusive single token); Rule 6 fires `E-SPEC-024`; Rule 7 SKIPS; only `E-SPEC-024` emitted; spec rejected |
| probe_table — valid match (EC-009-026) | `probe_table = "alerts"`, spec has `table_name = "alerts"` | Rule 8 passes; no E-SPEC-026; spec loads |
| probe_table — name not found (EC-009-027) | `probe_table = "devices"`, spec has `table_name = "alerts"` only | `E-SPEC-026` with `sensor_id`, `probe_table = "devices"`, `Declared tables: [alerts]`; spec rejected |
| probe_table — empty table list (EC-009-028) | `probe_table = "devices"`, spec has no `[[tables]]` | `E-SPEC-026` with `sensor_id`, `probe_table = "devices"`, `Declared tables: []`; spec rejected |
| header_scheme — raw + token_exchange (EC-009-029) | `auth_type = "token_exchange"`, `header_scheme = "raw"` | Rule 9 passes; spec loads |
| header_scheme — cookie + cookie_roundtrip (EC-009-030) | `auth_type = "cookie_roundtrip"`, `header_scheme = "cookie:access_token"` | Rule 9 passes; spec loads |
| header_scheme — coherence violation (EC-009-031) | `auth_type = "cookie_roundtrip"`, `header_scheme = "bearer"` | `E-SPEC-027` coherence variant; spec rejected |
| header_scheme — empty cookie name (EC-009-032) | `header_scheme = "cookie:"` | `E-SPEC-027` syntactic variant; spec rejected |
| auth_acquisition — token_exchange missing block (EC-009-034) | `auth_type = "token_exchange"`, no `[auth_acquisition]` | `E-SPEC-028(a)`; spec rejected |
| auth_acquisition — bearer_static with block (EC-009-037) | `auth_type = "bearer_static"`, `[auth_acquisition]` present | `E-SPEC-028(g)`; spec rejected |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| VP-059 | Spec validator: all errors collected (no fail-fast); warning-only specs return Ok — for any `SensorSpec` with N distinct validation errors (N >= 1), `validate_sensor_spec()` returns `Err(errors)` where `errors.len() == N`; for a spec with only warnings and no errors, returns `Ok(warnings)` (spec accepted); the function never returns early on the first error. Method: Proptest. Priority: P1. |

## Traceability
| Field | Value |
|-------|-------|
| Stories | S-1.11, S-1.13, PLUGIN-MIGRATION-001-F, S-SPEC-ENV-VAR-001, S-DEMO-CROWDSTRIKE-MULTIREGION-001, S-SPEC-HTTP-METHOD-VALIDATION-001, S-5.04 |
| L2 Capability | CAP-029 |
| Capability Anchor Justification | CAP-029 ("Config-Driven Sensor Adapters") per capabilities.md §CAP-029. This BC specifies spec-file validation — exactly what CAP-029 mandates: "Every spec file is validated at load time and reload time (DI-030). Variable references in step templates are resolved against the step dependency graph — forward references and undefined variables are validation errors (DEC-038)." Env-var token resolution (AC-6) is a prerequisite of that load-time validation: a spec whose `base_url` contains an unresolved `${env.VAR}` token cannot pass URL-format validation, so resolution must occur in the same spec-load pass. |
| L2 Invariants | DI-030 |
| L2 Entities | SensorSpec, TableSpec, ColumnSpec |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.17 | wave-a-spec-evolution-fix-burst-9 | 2026-07-22 | product-owner | F-WASE-P9-MED-001: Rule 1 auth_type parenthetical reworded from "(6-value canonical set; ... token_exchange is the 6th variant per ADR-054 D1)" to honest spec-first form: 6-value canonical TARGET set per ADR-054 D1; the as-built `VALID_AUTH_TYPES` in `spec_parser.rs::validate_cross_composition` is currently 5-value (confirmed) and is extended to 6 by the ADR-054 engine story; `token_exchange` annotated [PLANNED — engine story]. Sweep: no other live-body claim that `token_exchange` is already in code found beyond this parenthetical (Rules 9/10 references are target-spec definitions, not as-built code state assertions; changelog rows exempt). input-hash updated at commit time. |
| 1.16 | wave-a-spec-evolution-fix-burst-7 | 2026-07-22 | product-owner | F-WASE-P7-LOW-002: Rule 10(b) heading cite reworded from "(Definition 1 — ADR-054 v0.35 §D10(b))" to "(Definition 1, ratified in ADR-054 v0.35 §D10(b))"; Rule 10(h) trailing cite reworded from "ADR-054 v0.35 §D10(h)." to "(cardinality ratified in ADR-054 v0.35 §D10(h))". Both changes use ratification-provenance form so the cites cannot be read as stale current-version pins. Version numbers unchanged — they are historically correct ratification points. No version-sweep of other ADR-054 v0.35 occurrences per orchestrator adjudication. |
| 1.15 | wave-a-spec-evolution-fix-burst-2 | 2026-07-22 | product-owner | F-WASE-P2-HIGH-001: Rule 10(b) rewritten to ADR-054 v0.35 §D10(b) Definition 1 — fires when `auth_type ∈ {oauth2_client_credentials, token_exchange}` AND `auth_plugin` is present (regardless of `[auth_acquisition]` presence); removed UNCONDITIONAL framing; message template updated to `"sensor '{sensor_id}': auth_type = '{auth_type}' uses native declarative provider and does not accept auth_plugin. Remove auth_plugin or change auth_type to custom_via_plugin."` Disjointness note with (g) added (b=declarative auth_types, g=non-declarative). F-WASE-P2-HIGH-002: Rule 10(f) trigger corrected from "no [[credential_refs]] blocks declared" → "one or both of client_id, client_secret entries absent from [[credential_refs]]"; EC-009-041 added (client_secret-missing case). Rule 10(h) cardinality corrected to single aggregated emission with `{field_list}` (was "citing the mismatched field name" singular). Error Conditions E-SPEC-028 row updated: (b) and (f) summaries corrected. EC-009-036 updated to Definition 1 trigger. Companion: error-taxonomy.md v2.60. |
| 1.14 | wave-a-fix-burst-1 | 2026-07-22 | product-owner | F-WASE-P1-CRIT-001: Rule 10(a) expanded to fire for `auth_type ∈ {oauth2_client_credentials, token_exchange}` when `[auth_acquisition]` block is absent OR `token_path` is absent (previously: token_exchange block-absence only). Rule 10(d) corrected to check the four token_exchange-specific required fields (`credential_body_field`, `token_response_path`, `expiry_field`, `expiry_mode`); `ttl_buffer_secs` downgraded to OPTIONAL (default 30, ADR-054 §D3) — was wrongly required, which would have rejected the ADR-054 §D3 Armis wiring example. Error Conditions E-SPEC-028(a)/(d) rows updated to match. EC-009-035 rule reference corrected from Rule 10(d) → Rule 10(a) (`token_path` absent is a 10(a) condition, not 10(d)). EC-009-039 (oauth2_client_credentials block absent → Rule 10(a)) and EC-009-040 (token_exchange `token_response_path` absent → Rule 10(d)) added. |
| 1.13 | wave-a-spec-evolution-burst-3-correction | 2026-07-22 | product-owner | ADR-054 v0.33 D10(c) adjudication applied: §Validation Rule 10(c) valid set corrected from `{absolute_epoch_secs, ttl_secs}` → `{absolute_utc_string, relative_seconds}`; EC-009-038 updated from error-case (fires E-SPEC-028) to pass-case (`"absolute_utc_string"` IS in valid set) with CONSISTENCY FLAG note removed — conflict resolved by ADR-054 v0.33 D10(c). Cite: ADR-054 v0.33 D10(c) ratification (D11 follow-up rows in architect adjudication burst). |
| 1.12 | wave-a-spec-evolution-burst-3 | 2026-07-22 | product-owner | ADR-053 D2 + ADR-054 D1 amendment: §Description "seven categories" → "nine categories" (adds header_scheme + [auth_acquisition] to description list); §Validation Rule 1 auth_type enumeration adds `token_exchange` (6-value canonical set per ADR-054 D1; "(5-value canonical set; ...)" → "(6-value canonical set; ...)" with token_exchange annotation); added §Validation Rule 9 — `header_scheme` Validation (E-SPEC-027): syntactic three-form check (bearer/raw/cookie:<name>), auth_type coherence matrix, absent = pass-silently backward compat; added §Validation Rule 10 — `[auth_acquisition]` Coherence Validation (E-SPEC-028): 8 sub-conditions (a) token_exchange requires block; (b) auth_plugin+block coexist UNCONDITIONAL; (c) invalid expiry_mode; (d) token_exchange missing required fields; (e) credential_body_field not in credential_refs; (f) oauth2_client_credentials missing cred refs; (g) block on non-declarative auth_type; (h) token_exchange-only fields on non-token_exchange; §Error Conditions: added E-SPEC-027 row + E-SPEC-028 row; §Edge Cases: added EC-009-029..038 for Rules 9/10 coverage; §Canonical Test Vectors: added Rule 9/10 test vectors; input-hash "76729b7"→"fc9d874"; modified date 2026-07-22. NOTE: EC-009-038 flags CONSISTENCY ISSUE between BC-2.01.008 `expiry_mode = "absolute_utc_string"` and E-SPEC-028c valid set {absolute_epoch_secs, ttl_secs} — requires ADR-054 D10 adjudication. |
| 1.11 | S-5.04-spec-prep | 2026-06-22 | product-owner | Rule 8 (E-SPEC-026 probe_table-must-reference-declared-table): added §Validation Rules 8 with full trigger conditions, error message format (EXACT template from probe-table-field-design.md §1), implementation anchor (`SpecLoader::parse()` post-Rule-7, `SpecErrorCode::ESpec026`); added E-SPEC-026 to §Error Conditions; added EC-009-026..028 (valid match / name-not-found / empty-table-list); added 3 canonical test vectors for Rule 8; added S-5.04 to Stories traceability (D-1262 fold — probe_table is part of S-5.04, no separate S-5.04-PROBE-TABLE story exists). |
| 1.10 | FB-PR4 | 2026-06-04 | state-manager | v1.9→v1.10 2026-06-04 — §VR7 §Ordering Point 3 full-match skip-guard clause added (F-PR1-OBS-001 / F-PR6-HIGH-001): Rule 7 env-token skip fires only on exact full-string single-token match; partial embeddings fall through to whitelist → E-SPEC-025; added EC-009-022..025 + canonical test vectors. Test rename: `test_BC_2_16_009_e_spec_025_display_matches_error_taxonomy_v1_59_template_byte_for_byte` → `..._template_byte_for_byte` (OBS-PR6-001 / TD-VSDD-091). |
| 1.9 | FB-PR2 | 2026-06-04 | product-owner | §VR7 implementation-function signature corrected to `Vec<(usize, usize, SpecEngineError)>` (F-PR1-MED-002 closure); 32-codepoint `method_value` truncation invariant documented (F-PR4-MED-002 / SEC-001 / CWE-400), with load-bearing test `test_BC_2_16_009_sec_001_overlong_method_truncated_in_error`; §Description category count five→seven correcting stale description; malformed-pseudo-token behavior (e.g., `${env.lower}`) documented in §VR7 §Ordering with E-SPEC-025 fallback + F-LOCAL-P3-MED-002 test cite; EC-009-021 (overlong method echo cap) added to §Edge Cases; overlong-method canonical test vector added to §Canonical Test Vectors; §Validation Rules 6 env-var content restored (was reordered in v1.8 edit — corrected to Rules 6 then 7 ascending order). §VR6 full text re-added to maintain ascending rule order (6 before 7). OBS-PR4-001/002/003 folded. |
| 1.8 | Wave-5-Phase-A-PO-burst | 2026-06-03 | product-owner | S-SPEC-HTTP-METHOD-VALIDATION-001 (DRIFT-D926-001 anchor): Added §Validation Rules 7 — HTTP Method Whitelist Validation (AC-7). Whitelist: `GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `HEAD`, `OPTIONS` (7 values; compile-time `ALLOWED_HTTP_METHODS: &[&str]` constant). Validation runs POST Rule 6 env-var resolution; absent `step.method` (None) is NOT an error; wrong-case methods (e.g., `"get"`) are invalid; unsupported methods (`CONNECT`, `TRACE`) are invalid; empty string is invalid. Multi-error collection per INV-ERR-003. Rule 7 skips step.method fields that already failed Rule 6 (E-SPEC-024) to prevent double-reporting. Added E-SPEC-025 to §Error Conditions with full message template. Added edge cases EC-009-010..EC-009-020 and 6 canonical test vectors. Added S-SPEC-HTTP-METHOD-VALIDATION-001 to Stories traceability. No semantic change to existing rules. BC v1.7 → v1.8. |
| 1.7 | S-DEMO-CROWDSTRIKE-MULTIREGION-001 BC attachment burst | 2026-05-31 | product-owner | Updated §Validation Rules 6 sibling-sweep note: `crowdstrike.sensor.toml` now also uses `${env.CROWDSTRIKE_BASE_URL}` for `base_url` (S-DEMO-CROWDSTRIKE-MULTIREGION-001), making all four canonical sensor specs env-var-based for `base_url`. Added EC-009-008 (CrowdStrike eu-1 URL resolution — happy path) and EC-009-009 (CrowdStrike missing CROWDSTRIKE_BASE_URL → E-SPEC-024) for explicit CrowdStrike multi-region coverage. Added S-DEMO-CROWDSTRIKE-MULTIREGION-001 to Stories traceability. No semantic behavior change — the resolver was already sensor-agnostic; this adds CrowdStrike-specific test vectors to the existing contract. |
| 1.6 | S-SPEC-ENV-VAR-001 spec burst | 2026-05-31 | product-owner | Added §Validation Rules 6: Env Var Token Resolution (AC-6). Covers `${env.VAR_NAME}` token resolution in sensor spec string fields at spec-load time (post-TOML-parse, pre-URL-format-validation). Sibling-sweep (TD-VSDD-060): `${env.VAR}` pattern confirmed in `base_url` of `armis.sensor.toml`, `claroty.sensor.toml`, `cyberint.sensor.toml`; no other string fields in the four canonical sensor specs currently use env tokens. Added E-SPEC-024 to §Error Conditions. Added 7 edge cases EC-009-001..EC-009-007 covering: missing var, empty var, partial interpolation (cyberint pattern), failed partial interpolation, invalid URL after resolution, multiple failing tokens, overlay file failing. Added 4 canonical test vectors for env-var scenarios. AD-017 no-value-leak constraint documented explicitly in the rule (var NAME acceptable, var VALUE forbidden). error-taxonomy.md bumped to v1.56 in same burst. |
| 1.5 | D-776-post-merge | 2026-05-22 | state-manager | POL-14 auto-promotion at merge: PR #153 (PLUGIN-MIGRATION-001-D) squash-merged to develop@3f2de889 at 2026-05-22T09:05:47Z; status draft→active (lifecycle_status was already active). |
| 1.4 | FB-IMPL-P2-PO fix-burst-2 | 2026-05-20 | product-owner | F-007 closure (pass-2 adversarial): Added E-SPEC-002 (invalid column type) and E-SPEC-003 (undefined variable reference) to §Error Conditions — both codes were present in error-taxonomy.md and exercised by HS-017 sub-scenarios but absent from this BC's error table (AI-built defect fixed in-scope per CLAUDE.md Canonical Principle Rule 4). F-008 closure: §Validation Rules 1 `auth_type` enumeration expanded from 4-value to 5-value set — added `custom_via_plugin` per `VALID_AUTH_TYPES` constant in `spec_parser.rs::validate_cross_composition` (CODE-GROUNDED: 5 values confirmed in source). |
| 1.3 | pass-74-fix | 2026-04-20 | product-owner | Resolved (placeholder) row in ## Verification Properties per pass-74 VP-TBD decision matrix extension. |
| 1.2 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description; added ## Invariants; added ## Error Conditions (normalized from inline Error Codes section); added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft |
