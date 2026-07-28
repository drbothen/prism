---
document_type: behavioral-contract
level: L3
version: "1.30"
status: active
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-16"
capability: "CAP-029"
lifecycle_status: active
introduced: cycle-1
modified: "2026-07-27"
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
ten categories of checks in a single pass: schema validation (field types, regex
patterns, enumerations), variable reference resolution (ensuring `${step.field}`
references point to steps that exist and have already executed), OCSF field validation
(against the compiled protobuf schema), pagination configuration consistency, rate
limit hint validity, env-var token resolution (substituting `${env.VAR_NAME}` tokens
in string fields before subsequent validation rules run), HTTP method whitelist
validation (rejecting `step.method` values not in the 7-element allowed set),
probe table reference validation (verifying `probe_table` matches a declared
`table_name` in `[[tables]]` when set),
`header_scheme` validation (syntactic check against the three-form set plus auth_type
coherence matrix per ADR-053 D2) [PLANNED — Wave-A engine story per ADR-053 D2 / ADR-054 D10], and `[auth_acquisition]` coherence validation
(block presence requirements, required fields, and type-scoped field restrictions
per ADR-054 D10) [PLANNED — Wave-A engine story per ADR-053 D2 / ADR-054 D10].

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
- If pagination type is `page_number`, `page_size` must be > 0; a TOML spec declaring `page_size = 0` for a `PageNumber` step is rejected at spec-load time by `validate_sensor_spec` §Category 4 with `SpecErrorCode::ESpec001` and message `"page_number pagination in step '{step_name}' requires page_size > 0"`. Mirrors the `offset_limit` §Category 4 rejection. Grounding: ADR-056 §D10 CE-2; §D3 spec-load layer. Anchored: `S-WAVE-A-CYBERINT-SPEC-001` RG-017.
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

**Implementation anchor:** `SpecLoader::parse()` in `crates/prism-spec-engine/src/spec_parser.rs`; Rule 8 executes at the end of `parse()`, after the timestamp_formats, timestamp_fallback_chain, and source_path validation gates, immediately before `Ok(spec)` is returned. `validate_step_methods()` (Rule 7) is NOT called from within `parse()` — it is invoked by callers of `parse()` (e.g., `parse_and_validate_spec_toml()`, `load_all()`). Rule 8 therefore does NOT run "after the Rule 7 `validate_step_methods` call" — it runs inside `parse()` independent of Rule 7. `SpecErrorCode::ESpec026` variant added to `prism_core::SpecErrorCode` (`#[non_exhaustive]` enum — additive, no semver break, no new compile-fail gate entry; EXPECTED is ci.yml EXPECTED authority — S-5.04 probe_table adds +0 because ESpec026 is a new variant on a #[non_exhaustive] enum, not a new pub struct).

**TOML path:** `sensor.probe_table`.

**Ordering:** Rule 8 MUST execute after Rule 7. Rule 8 is independent of Rules 1–7 and does not interact with env-var resolution or HTTP method validation.

### 9. `header_scheme` Validation (E-SPEC-027) — Wave-A ADR-053 D2 [PLANNED — Wave-A engine story per ADR-053 D2 / ADR-054 D10]

This validation rule runs AFTER Rule 8 (probe table reference validation). It validates the `header_scheme` TOML field, which is represented as `Option<String>` with bare `#[serde(default)]` — `None` when absent (per ADR-053 v0.30 §D2; the `default_header_scheme()` function does NOT exist; absence is `None`, not `"bearer"` at deserialization).

**Absence handling — two branches (ADR-053 v0.30 §D2):**

- **Absence path A — `header_scheme` absent (`None`) AND `auth_type` is NOT `cookie_roundtrip`:** Rule 9 passes silently. The runtime default `"bearer"` is applied in `build_request()` via `header_scheme.as_deref()` matching `None | Some("bearer")` at execution time — NOT at deserialization. Existing sensor specs that omit `header_scheme` remain backward-compatible.

- **Absence path B — `header_scheme` absent (`None`) AND `auth_type = "cookie_roundtrip"`:** E-SPEC-027 template (c) load-time error. `cookie_roundtrip` sensors MUST provide an explicit `header_scheme = "cookie:<name>"` value; there is no sensible cookie-name default. Spec rejected; boot fails exit code 2; non-retryable without correcting the TOML.

**Syntactic check:**
`header_scheme` must be one of exactly three forms:
- `"bearer"` — inject `Authorization: Bearer {token}` on data-fetch requests
- `"raw"` — inject `Authorization: {token}` with NO "Bearer" prefix (used by sensors whose API rejects the prefix, e.g., Armis Centrix)
- `"cookie:<name>"` — inject `Cookie: <name>={token}`, where `<name>` is the non-empty cookie name substring after `cookie:` (`<name>` must be non-empty, must be ≤128 codepoints, and must consist entirely of RFC 6265 `cookie-name` / `token` characters (tchar per RFC 9110 §5.6.2: letters, digits, and ``! # $ % & ' * + - . ^ _ ` | ~``)). **Length bound (MED-005 / CWE-390):** a cookie name >128 codepoints passes the tchar character-class check but is injected verbatim into the `Cookie: <name>={token}` header at `build_request()` call time, producing an opaque HTTP 431 Request Header Too Large response from the sensor API — re-introducing the CWE-390 deferred-opaque-failure mode that SEC-001 was credited with eliminating, by a different door. Rationale for 128: the longest production cookie name in the canonical sensor suite is `"access_token"` (12 chars); 128 is ≥10× that; proportionally smaller than E-AUTH-006's 4096-byte cookie value bound (cookie names are short identifiers; values are tokens). Cookie name >128 codepoints → E-SPEC-027 template (a).

Any value that does not match one of these three forms (including `"cookie:"` with empty name, `"cookie:foo:bar"` where `:` is not a tchar character, `"cookie:sid=x; admin"` where `;` and `=` are not tchar characters, or an arbitrary string) → `E-SPEC-027` (syntactic variant).

**Error message (syntactic):** ``"sensor '{sensor_id}' has invalid header_scheme = '{value}'. Valid values: bearer, raw, cookie:<name> (non-empty name, ≤128 codepoints, RFC 6265 token characters only: A-Z a-z 0-9 ! # $ % & ' * + - . ^ _ ` | ~)"``

**64-codepoint echo cap for `{value}` in template (a) (CWE-400 mitigation):** The `{value}` substitution in template (a) is the full `header_scheme` string as declared in the TOML. A 256 KiB TOML permits `header_scheme` values up to that bound; echoing verbatim produces an unbounded message up to 256 KiB in the MCP error response (CWE-400). Cap precedents: Rule 7's `method_value` is capped at 32 codepoints; `base_url` echoes are capped at 200 codepoints. `header_scheme` valid forms are `"bearer"` (6 chars), `"raw"` (3 chars), and `"cookie:<name>"` (7 + cookie name); the longest production value in the canonical sensor suite is `"cookie:access_token"` (19 chars). **Cap:** before substituting `{value}` in template (a), apply `truncate_at_char_boundary(&header_scheme_value, 64)` — the same helper used by Rule 7's method echo cap. 64 codepoints is ≥2.4× the longest realistic production value; for values ≤64 codepoints the truncated string equals the original (POL-24 preserved for all inputs that reach a human reader in practice). Rule 7's 32-codepoint cap is unsuitable here because a 34-char invalid value such as `"cookie:xxxxxxxxxxxxxxxxxxxxxxxxxxx"` would be truncated by a 32-char cap even though the value is informative; 64 avoids that problem.

**CTL-character escaping for `{value}` in template (a) (CWE-117 log-injection mitigation):** EC-009-046 guarantees that CTL characters (bytes 0x00–0x1F, 0x7F) in a cookie name always trigger template (a). Template (a) embeds `{value}` in both the MCP error-response `content[].text` and the audit log line. A raw LF (0x0A) in `{value}` — e.g., `header_scheme = "cookie:a\nFORGED LINE"` — injects a newline into the log, enabling log injection (CWE-117). **Escaping:** after applying the 64-codepoint cap, iterate over the UTF-8 byte sequence of the capped string and replace each byte b where `(b as u8) <= 0x1F || (b as u8) == 0x7F` with the four-character ASCII sequence `\xNN` (literal backslash, literal `x`, two uppercase hex digits for b); all other bytes are emitted as-is. Examples: LF (0x0A) → `\x0A`, CR (0x0D) → `\x0D`, NUL (0x00) → `\x00`. This rule is fully deterministic: two independent implementers who apply it produce byte-identical output. For values with no CTL bytes the escaping is a no-op; the emitted message is byte-identical to the original substitution (POL-24 preserved).

**Coherence matrix (auth_type ↔ allowed header_scheme):**

| `auth_type` | Allowed `header_scheme` values |
|-------------|-------------------------------|
| `bearer_static` | `"bearer"` or `"raw"` |
| `oauth2_client_credentials` | `"bearer"` or `"raw"` |
| `cookie_roundtrip` | `"cookie:<name>"` ONLY. **`header_scheme` is REQUIRED — absent `header_scheme` fires E-SPEC-027 template (c) (absence path B); absence path A's silent `"bearer"` carve-out does NOT apply for `cookie_roundtrip`.** |
| `custom_via_plugin` | `"bearer"` or `"raw"` |
| `api_key` | `"bearer"` only |
| `token_exchange` | `"bearer"` or `"raw"` |

A syntactically valid `header_scheme` that is incoherent with the declared `auth_type` → `E-SPEC-027` (coherence variant). Example: `auth_type = "cookie_roundtrip"` with `header_scheme = "bearer"` → E-SPEC-027 coherence.

**Error message (coherence):** `"sensor '{sensor_id}': auth_type = '{auth_type}' does not permit header_scheme = '{value}'; allowed for this auth_type: {allowed_set}"`

**Entry points and function coverage (normative):** Four functions participate in the validation pipeline; they cover distinct rule sets and are composed by callers — they do NOT call each other:

- **`validate_sensor_spec()`** (`crates/prism-spec-engine/src/validation.rs`) — covers Rules 1–5 (schema validation, variable reference resolution, OCSF field validation, pagination configuration, rate limit hints). A standalone pure function. VP-059 is explicitly scoped to `validate_sensor_spec()` and does NOT cover Rules 6–10. `SpecLoader::parse()` does NOT call `validate_sensor_spec()` — they are independent entry points invoked separately by callers.
- **`SpecLoader::parse()`** (`crates/prism-spec-engine/src/spec_parser.rs`) — performs TOML deserialization, credential_refs cross-composition Rules A+B, timestamp_formats and timestamp_fallback_chain validation gates, source_path validation, and Rule 8 (probe_table reference validation). Returns `Ok(SensorSpec)` before Rules 6 and 7 run. Does NOT call `validate_sensor_spec()`, `resolve_env_var_tokens()`, or `validate_step_methods()`.
- **`resolve_env_var_tokens()`** (`crates/prism-spec-engine/src/env_resolver.rs`) — covers Rule 6 (env-var token resolution). Called by callers of `SpecLoader::parse()` as a separate post-parse step.
- **`validate_step_methods()`** (`crates/prism-spec-engine/src/validation.rs`) — covers Rule 7 (HTTP method whitelist validation). Called by callers after `resolve_env_var_tokens()` completes.

**Integration function:** `parse_and_validate_spec_toml()` (`crates/prism-spec-engine/src/add_sensor_spec.rs`) composes the pipeline for the `add_sensor_spec` path: calls `SpecLoader::parse()` first (covers TOML deserialization + Rule B + timestamp gates + Rule 8), then `resolve_env_var_tokens()` (Rule 6), then `validate_step_methods()` (Rule 7). The S-WAVE-A-ENGINE-001 implementation adds Rule 9 inside `SpecLoader::parse()` — not inside `validate_sensor_spec()` — ensuring Rule 9 executes on every path that calls `parse()`. The S-ADR054-WAVE-A-001 implementation adds Rule 10 inside `SpecLoader::parse()` by the same rationale, per ADR-054 §D10 — not inside `validate_sensor_spec()` — ensuring Rule 10 also executes on every path that calls `parse()`.

**Security requirement (Rule 9):** security-triage-rule9-cookie-name-charset.md §1.3 identifies exactly one active injection vector: a client holding `sensor_spec.write` calling the `add_sensor_spec` MCP tool with crafted `toml_content` (the `toml_content` wire parameter per BC-2.16.008 §Tool Schema). For Rule 9 to close this vector, `add_sensor_spec` MUST reach `SpecLoader::parse()` — an implementation that calls only `validate_sensor_spec()` in the `add_sensor_spec` handler bypasses Rules 8/9/10 (none of Rules 8–10 are in `validate_sensor_spec()`) and leaves the injection vector open. As-built: `parse_and_validate_spec_toml()` calls `SpecLoader::parse()` as its first act, so Rule 9 is automatically covered once S-WAVE-A-ENGINE-001 adds it inside `parse()`. No changes to the `add_sensor_spec` integration path are required for Rule 9 coverage.

**Ordering:** Rule 9 runs after Rule 8 and before Rule 10. It is independent of Rules 1–8.

### 10. `[auth_acquisition]` Coherence Validation (E-SPEC-028) — Wave-A ADR-054 D10 [PLANNED — Wave-A engine story per ADR-053 D2 / ADR-054 D10]

This validation rule runs AFTER Rule 9 (`header_scheme` validation). It validates the optional `[auth_acquisition]` TOML sub-table when present, and checks required-block obligations for declarative auth types. All sub-conditions are checked in a single pass (no fail-fast); all errors are collected.

**Sub-conditions (8 checks):**

**(a) Declarative auth_types require `[auth_acquisition]` with `token_path`:** If `auth_type ∈ {oauth2_client_credentials, token_exchange}` and either (i) no `[auth_acquisition]` block is present, or (ii) `[auth_acquisition]` is present but `token_path` is absent → `E-SPEC-028`. Both declarative auth types require `token_path` to derive the per-org token URL at boot step 9A (ADR-054 §D10(a)).

**(b) `auth_plugin` on declarative `auth_type` (Definition 1, ratified in ADR-054 v0.35 §D10(b)):** If `auth_type ∈ {oauth2_client_credentials, token_exchange}` and `auth_plugin` is present in the sensor spec → `E-SPEC-028`, regardless of whether `[auth_acquisition]` is declared. These auth_types use the native `DeclarativeHttpAuthProvider`; `auth_plugin` serves no role in declarative auth flows and its presence is rejected to prevent silent misconfiguration (ADR-054 §D5/D10(b)). Message template: `"sensor '{sensor_id}': auth_type = '{auth_type}' uses native declarative provider and does not accept auth_plugin. Remove auth_plugin or change auth_type to custom_via_plugin."` **Disjointness with (g):** (b) covers `auth_type ∈ {oauth2_client_credentials, token_exchange}` (declarative types); (g) covers `auth_type ∈ {bearer_static, cookie_roundtrip, api_key, custom_via_plugin}` (non-declarative types). These sets are disjoint — a given sensor spec triggers at most one of (b) or (g), never both.

**(c) Invalid `expiry_mode`:** If `auth_type = "token_exchange"` AND `[auth_acquisition]` is present AND `expiry_mode` is set, its value must be one of: `"absolute_utc_string"` or `"relative_seconds"`. Any other value → `E-SPEC-028` citing the invalid `expiry_mode` value. Note: when `expiry_mode` appears in an `[auth_acquisition]` block whose `auth_type` is not `token_exchange`, sub-condition (h) handles the position-validity check; sub-condition (c) does NOT additionally fire for value-validity of a wrong-position field.

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
- Within each validation function, all errors and warnings are collected before returning (see §Invariants for the fail-fast boundary between Rule 9 and Rule 10 within `SpecLoader::parse()`)
- Errors are grouped by spec file, then by table, then by field
- Each error includes the exact TOML path (e.g., `sensor.tables[0].steps[1].path_template`) for actionable correction
- Warnings do not prevent the spec from loading; errors do

## Invariants
- Collect-all semantics are per validation function: `validate_sensor_spec()` (Rules 1–5) collects all errors before returning (VP-059); Rule 10 within `SpecLoader::parse()` checks all 8 sub-conditions in a single pass (no fail-fast within Rule 10). At the `SpecLoader::parse()` rule boundary, execution is fail-fast: Rule 9 returning `Err` prevents Rule 10 from executing — a spec with both a Rule 9 violation and a Rule 10 violation reports only the Rule 9 error from `parse()`.
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
| `E-SPEC-027` | Three templates: (a) `header_scheme` value is syntactically invalid (not `"bearer"`, `"raw"`, or `"cookie:<name>"` with non-empty name consisting entirely of RFC 6265 tchar characters — any non-tchar character including `;`, `=`, SP, CTL, high bytes, and RFC 9110 §5.6.2 delimiters triggers this template; message: ``"sensor '{sensor_id}' has invalid header_scheme = '{value}'. Valid values: bearer, raw, cookie:<name> (non-empty name, ≤128 codepoints, RFC 6265 token characters only: A-Z a-z 0-9 ! # $ % & ' * + - . ^ _ ` | ~)"``); (b) `header_scheme` value is syntactically valid but incoherent with the declared `auth_type` (e.g., `auth_type = "cookie_roundtrip"` with `header_scheme = "bearer"`); (c) `header_scheme` is absent (`None`) AND `auth_type = "cookie_roundtrip"` (absence path B — no sensible cookie-name default; message: `"sensor '{sensor_id}': auth_type = 'cookie_roundtrip' requires an explicit header_scheme = 'cookie:<name>' value; absent header_scheme is not valid for cookie_roundtrip auth (cookie name unknown)"`). Absent `header_scheme` IS an error for `cookie_roundtrip` (template (c)); absent `header_scheme` is NOT an error for all other auth_types (absence path A — silent `"bearer"` default at runtime in `build_request()`) | [PLANNED — Wave-A engine story per ADR-053 D2 / ADR-054 D10] Spec rejected; syntactic error (template a) includes `sensor_id`, `header_scheme` value (capped at 64 codepoints via `truncate_at_char_boundary`, CTL bytes 0x00–0x1F/0x7F replaced with `\xNN` — CWE-400/CWE-117; EC-009-047..048); template (a) also fires when cookie `<name>` exceeds 128 codepoints (MED-005 / CWE-390; EC-009-051) or contains high bytes (0x80–0xFF, not tchar; EC-009-050) or TAB (0x09, CTL, not tchar; EC-009-049); template (a) includes corrective guidance citing the RFC 6265 tchar constraint (SEC-001 / CWE-20 fix per security-triage-rule9-cookie-name-charset.md); coherence error (template b) includes `sensor_id`, `auth_type`, `header_scheme` value, and allowed set for that auth_type; absence error (template c) includes `sensor_id`; ADR-053 v0.30 §D2 |
| `E-SPEC-028` | `[auth_acquisition]` block coherence violation — any of 8 sub-conditions: (a) `auth_type ∈ {oauth2_client_credentials, token_exchange}` with block absent OR `token_path` absent; (b) `auth_type ∈ {oauth2_client_credentials, token_exchange}` AND `auth_plugin` present (block presence not required — Definition 1, ADR-054 v0.35 §D10(b)); (c) invalid `expiry_mode`; (d) `token_exchange` missing any of `{credential_body_field, token_response_path, expiry_field, expiry_mode}` — one error per absent field; `ttl_buffer_secs` is optional; (e) `credential_body_field` not in `[[credential_refs]]`; (f) `oauth2_client_credentials` missing `client_id` or `client_secret` credential refs (one or both absent); (g) block on non-declarative auth_type; (h) `token_exchange`-only fields on non-`token_exchange` auth_type — single aggregated `{field_list}` emission | [PLANNED — Wave-A engine story per ADR-053 D2 / ADR-054 D10] Spec rejected; error message includes `sensor_id`, specific sub-condition identifier, field name(s), and corrective guidance; all sub-condition errors collected in single multi-error pass |

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
| EC-009-030 | `header_scheme = "cookie:access_token"` with `auth_type = "cookie_roundtrip"` (e.g., Cyberint pattern) | Rule 9 passes: `"cookie:access_token"` is syntactically valid (non-empty name, all characters are RFC 6265 tchar); coherence check: `cookie_roundtrip` allows `"cookie:<name>"` → passes. Spec loads. |
| EC-009-031 | `header_scheme = "bearer"` with `auth_type = "cookie_roundtrip"` — coherence violation | Rule 9 syntactic check passes (`"bearer"` is valid form); coherence check: `cookie_roundtrip` requires `"cookie:<name>"` ONLY; `"bearer"` is not allowed → `E-SPEC-027` coherence variant; spec rejected. |
| EC-009-032 | `header_scheme = "cookie:"` — empty cookie name | Rule 9 syntactic check fails: `<name>` after `cookie:` is empty → `E-SPEC-027` syntactic variant; spec rejected. |
| EC-009-033 | `header_scheme = "cookie:foo:bar"` — colon in cookie name | Rule 9 syntactic check fails: `':'` is not a tchar character → `E-SPEC-027` syntactic variant; spec rejected. (The old rationale "contains colon" is retired — it was a subset of the correct constraint; `;`, `=`, SP, and other non-tchar characters are now equally rejected per SEC-001.) |
| EC-009-034 | `auth_type = "token_exchange"` with no `[auth_acquisition]` block | Rule 10(a) fires `E-SPEC-028`; spec rejected. |
| EC-009-035 | `auth_type = "token_exchange"` with `[auth_acquisition]` present but `token_path` absent | Rule 10(a) fires `E-SPEC-028` citing missing `token_path` (token_path absence is a 10(a) condition, not 10(d)); spec rejected. |
| EC-009-036 | `auth_type = "oauth2_client_credentials"`, `auth_plugin = "crowdstrike-oauth2"` present, with or without `[auth_acquisition]` (Definition 1 — ADR-054 v0.35 §D10(b)) | **Branch A — WITH `[auth_acquisition]` (and `token_path` present):** Rule 10(b) fires only — one `E-SPEC-028`; spec rejected. Message: `"sensor '<id>': auth_type = 'oauth2_client_credentials' uses native declarative provider and does not accept auth_plugin..."` **Branch B — WITHOUT `[auth_acquisition]`:** Rule 10(a) AND Rule 10(b) both fire — two `E-SPEC-028` errors collected in the same multi-error pass: (a) fires for missing `[auth_acquisition]` block on declarative auth_type; (b) fires for `auth_plugin` coexisting with declarative auth_type. This is a (a)∩(b) co-fire. If `auth_type` were `custom_via_plugin` instead, neither (a) nor (b) would fire — `custom_via_plugin` is non-declarative, does not require an `[auth_acquisition]` block, and legitimately uses `auth_plugin`; in Branch B (no block) the spec is valid with zero E-SPEC-028 errors (in Branch A, with a block present, (g) fires). |
| EC-009-037 | `[auth_acquisition]` present with `auth_type = "bearer_static"` | Rule 10(g) fires `E-SPEC-028`; spec rejected. |
| EC-009-038 | `auth_type = "token_exchange"`, `expiry_mode = "absolute_utc_string"` in `[auth_acquisition]` | Rule 10(c) passes: `"absolute_utc_string"` IS in `{absolute_utc_string, relative_seconds}` (valid set per ADR-054 v0.33 D10(c)); spec loads (subject to other Rule 10 checks passing). Consistency conflict with BC-2.01.008 resolved by ADR-054 v0.33 D10(c) adjudication. |
| EC-009-039 | `auth_type = "oauth2_client_credentials"` with no `[auth_acquisition]` block | Rule 10(a) fires `E-SPEC-028` — `oauth2_client_credentials` requires `[auth_acquisition]` with `token_path` (same condition as token_exchange); spec rejected. |
| EC-009-040 | `auth_type = "token_exchange"` with `[auth_acquisition]` present including `token_path` but missing `token_response_path` | Rule 10(d) fires `E-SPEC-028` citing missing `token_response_path`; spec rejected. Verifies the four-field check in 10(d): a TOML omitting any of `{credential_body_field, token_response_path, expiry_field, expiry_mode}` is rejected even when `token_path` is present (which only satisfies 10(a)). |
| EC-009-041 | `auth_type = "oauth2_client_credentials"`, `[[credential_refs]]` declares `client_id` only — no `client_secret` entry | Rule 10(f) fires `E-SPEC-028`; `{field_list}` = `"client_secret"`; spec rejected. Demonstrates that a partially-declared credential set is rejected — both `client_id` AND `client_secret` must appear in `[[credential_refs]]`. A spec with zero `[[credential_refs]]` blocks would produce `{field_list}` = `"client_id, client_secret"`. |
| EC-009-042 | `auth_type = "cookie_roundtrip"` with `header_scheme` absent (field not present in TOML) — absence path B | Rule 9 absence path B fires: `header_scheme` is `None` AND `auth_type = "cookie_roundtrip"` → E-SPEC-027 template (c); spec rejected; boot fails exit code 2. Message: `"sensor '{sensor_id}': auth_type = 'cookie_roundtrip' requires an explicit header_scheme = 'cookie:<name>' value; absent header_scheme is not valid for cookie_roundtrip auth (cookie name unknown)"`. This was previously documented as "passes silently" — ADR-053 v0.30 §D2 corrects this: the silent default `"bearer"` would cause wrong-header injection for cookie_roundtrip sensors. `{sensor_id}` is config text, safe to echo (AD-017). |
| EC-009-043 | `header_scheme = "cookie:sid=x; admin"` — semicolon and equals in cookie name (SEC-001 security-motivating case, CWE-20/CWE-74) | Rule 9 syntactic check fails: `;` and `=` are not tchar characters → `E-SPEC-027` template (a); spec rejected at load time. Without this check, the synthesized `Cookie` header value would be `"sid=x; admin={token}"`, remapping the auth credential to the attacker-controlled key `admin` rather than the spec-intended name. The token is still sent to the correct sensor API server (not exfiltrated — `base_url` is validated independently), but the auth request fails because the expected cookie name is absent. Requires `sensor_spec.write` capability to exploit (multi-tenant MSSP vector per security-triage-rule9-cookie-name-charset.md §1.3). |
| EC-009-044 | `header_scheme = "cookie:a=b"` — bare `=` in cookie name | Rule 9 syntactic check fails: `=` is not a tchar character → `E-SPEC-027` template (a); spec rejected. `=` in the cookie name corrupts the `name=value` boundary in the Cookie header, mapping the token to an unintended key. |
| EC-009-045 | `header_scheme = "cookie:a b"` — SPACE in cookie name (TAB U+0009 is equally invalid) | Rule 9 syntactic check fails: SP (0x20) is not a tchar character → `E-SPEC-027` template (a); spec rejected. Neither SP nor TAB appears in the RFC 9110 §5.6.2 tchar set. |
| EC-009-046 | `header_scheme = "cookie:a\nb"` — CTL character (LF, CR, NUL, etc.) in cookie name (SEC-002 side effect) | Rule 9 syntactic check fails: CTL characters are not tchar characters → `E-SPEC-027` template (a); spec rejected at load time. Previously (SEC-002 / CWE-390): CTL characters passed Rule 9's old no-colon check and loaded without error; every subsequent query then failed with the opaque error `"builder error"` from reqwest (deferred `HeaderValue::InvalidHeaderValue` at `.send()` time with no indication of the root cause). The RFC 6265 tchar fix fully eliminates the SEC-002 deferred-error path as a side effect — CTL characters are now rejected at spec-load time with a specific E-SPEC-027 template (a) message. |
| EC-009-047 | `header_scheme` = 65-codepoint all-`X` string — overlong echo-cap trigger (HIGH-002 CWE-400); mirrors EC-009-021 for Rule 7 method echo cap | Rule 9 syntactic check fires: a 65-char string does not match `"bearer"`, `"raw"`, or `"cookie:<name>"` → `E-SPEC-027` template (a); spec rejected. The `{value}` field in the error message is truncated at 64 codepoints via `truncate_at_char_boundary` before embedding (CWE-400 mitigation); the emitted `{value}` is the first 64 `X` characters. For values ≤64 codepoints the error message `{value}` is byte-identical to the original (POL-24 preserved for the common case). |
| EC-009-048 | `header_scheme = "cookie:a\x0Ab"` — LF byte (0x0A) in cookie name; CTL-escaping in error message (HIGH-002 CWE-117 mitigation; complement to EC-009-046, which documents the rejection trigger) | Rule 9 syntactic check fires: 0x0A is not a tchar byte → `E-SPEC-027` template (a); spec rejected. The `{value}` substitution MUST replace the 0x0A byte with the four-character ASCII sequence `\x0A` (literal backslash-x-0-A) before embedding in the error message and log line. The emitted message contains the literal text `cookie:a\x0Ab` — NOT a raw newline byte — preventing CWE-117 log-line injection. For values with no CTL bytes the escaping is a no-op; the emitted message is byte-identical to the original substitution (POL-24 preserved). |
| EC-009-049 | `header_scheme = "cookie:\t"` — TAB (byte 0x09) as the sole cookie name character (MED-010; independent EC for TAB; EC-009-045 documents SPACE with a parenthetical note about TAB, which is insufficient for a standalone test vector) | Rule 9 syntactic check fails: TAB (0x09) is a CTL byte and is not a tchar character (RFC 9110 §5.6.2 tchar set does not include TAB; TAB is 0x09 = HT, a control character per ASCII) → `E-SPEC-027` template (a); spec rejected. The `{value}` substitution MUST replace the 0x09 byte with the four-character ASCII sequence `\x09` (literal backslash-x-0-9) before embedding in the error message and log line. The emitted message contains the literal text `cookie:\x09` — NOT a raw TAB byte — an eleven-character string (`cookie:` = 7 chars + `\x09` = 4 chars); TAB (0x09) is in the CTL escape class (`(b as u8) <= 0x1F`) by the same escaping rule that applies to LF (EC-009-048, which emits `cookie:a\x0Ab`). Note: SP (0x20) is non-tchar and triggers template (a) rejection, but SP (0x20 > 0x1F) is NOT in the CTL escape class and is therefore NOT escaped in the `{value}` substitution — EC-009-049 is the authoritative case that tests TAB's dual role as both non-tchar (rejection trigger) AND CTL (escape obligation). Implementation correctness probe: an implementation using `char.is_whitespace()` to reject whitespace would also reject TAB — but the escaping obligation arises from the CTL class, not the whitespace class; an implementation that tests `char.is_whitespace()` to gate escaping but uses `char::is_ascii_control()` (which excludes 0x7F) would miss DEL escaping. The correct implementation uses a byte-level tchar predicate (`is_tchar_byte(b)`) for rejection and a byte-range predicate (`b <= 0x1F || b == 0x7F`) for escaping, independently. |
| EC-009-050 | `header_scheme = "cookie:café"` — UTF-8 high byte (U+00E9 = 'é', encoded as 0xC3 0xA9) in cookie name (MED-010; implementation correctness probe for `chars()` vs `bytes()` tchar check) | Rule 9 syntactic check fails: 'é' (U+00E9) is NOT in the RFC 9110 §5.6.2 tchar set (which includes only ASCII letters A-Z and a-z — not Unicode letters) → `E-SPEC-027` template (a); spec rejected. **Implementation correctness probe:** an implementation using `char.is_alphanumeric()` would ACCEPT `"café"` because `'é'.is_alphanumeric()` returns `true` in Rust (Unicode-aware); the correct implementation uses the byte-level tchar predicate (`name.bytes().all(is_tchar_byte)`) which rejects bytes 0xC3 and 0xA9 (both ≥ 0x80 and not in the tchar byte set). This test specifically catches the `char.is_alphanumeric()` vs `is_tchar_byte` implementation divergence. |
| EC-009-051 | `header_scheme = "cookie:" + "a" * 129` — cookie name with 129 codepoints, one over the 128-codepoint bound (MED-005 / CWE-390) | Rule 9 length check fires: `<name>` is 129 codepoints, exceeding the 128-codepoint maximum → `E-SPEC-027` template (a); spec rejected. The tchar check would PASS for all-`a` input (every byte is a valid tchar); only the length bound triggers the rejection. Prevents a >128-codepoint cookie name from being injected verbatim into the `Cookie: <name>={token}` header, which would produce an opaque HTTP 431 from the sensor API (CWE-390 deferred-opaque-failure mode). |

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
| header_scheme — absent + cookie_roundtrip (EC-009-042, absence path B) | `auth_type = "cookie_roundtrip"`, `header_scheme` field absent | `E-SPEC-027` template (c) absence path B; spec rejected; message: `"sensor '...': auth_type = 'cookie_roundtrip' requires an explicit header_scheme = 'cookie:<name>' value; absent header_scheme is not valid for cookie_roundtrip auth (cookie name unknown)"` |
| header_scheme — semicolon injection (EC-009-043, SEC-001 CWE-20/CWE-74) | `header_scheme = "cookie:sid=x; admin"` | `E-SPEC-027` template (a); spec rejected at load time (`;` and `=` are not tchar characters; synthesized Cookie would have been `sid=x; admin={token}` — cookie-pair injection) |
| header_scheme — equals sign in name (EC-009-044) | `header_scheme = "cookie:a=b"` | `E-SPEC-027` template (a); spec rejected (`=` not a tchar character) |
| header_scheme — space in name (EC-009-045) | `header_scheme = "cookie:a b"` | `E-SPEC-027` template (a); spec rejected (SP not a tchar character) |
| header_scheme — CTL character in name (EC-009-046, SEC-002 side effect) | `header_scheme = "cookie:a\nb"` (LF in name) | `E-SPEC-027` template (a); spec rejected at load time; load-time rejection replaces the previous deferred opaque `"builder error"` from reqwest (SEC-002 / CWE-390 eliminated as side effect of SEC-001 fix) |
| header_scheme — overlong value, echo cap (EC-009-047, HIGH-002 CWE-400) | `header_scheme` = 65-character string of `X`s | `E-SPEC-027` template (a); spec rejected; `{value}` in error message is the first 64 `X` characters (truncated by `truncate_at_char_boundary`); mirrors EC-009-021 for Rule 7 method echo cap |
| header_scheme — CTL in value, error-message escaping (EC-009-048, HIGH-002 CWE-117) | `header_scheme = "cookie:a\x0Ab"` (LF byte 0x0A in cookie name) | `E-SPEC-027` template (a); spec rejected; emitted error message contains the literal text `cookie:a\x0Ab` (four-char sequence `\x0A`), NOT a raw newline byte; prevents log-injection |
| header_scheme — TAB in cookie name (EC-009-049, MED-010) | `header_scheme = "cookie:\t"` (TAB = 0x09 as sole cookie name char) | `E-SPEC-027` template (a); spec rejected; TAB (0x09) is a CTL byte (0x09 ≤ 0x1F) and not a tchar byte; emitted error message `{value}` is `cookie:\x09` (eleven-char literal: `cookie:` [7 chars] + CTL-escape `\x09` [4 chars]) — NOT a raw TAB byte; same CTL-escape rule as EC-009-048 (LF 0x0A → `\x0A`); SP (0x20) is also non-tchar but is NOT in the CTL escape class (0x20 > 0x1F) and is not escaped — TAB is the boundary case that tests both rejection AND escaping; byte-level tchar predicate is the correct implementation |
| header_scheme — high byte in cookie name (EC-009-050, MED-010) | `header_scheme = "cookie:café"` (é = U+00E9, UTF-8 bytes 0xC3 0xA9) | `E-SPEC-027` template (a); spec rejected; 0xC3 and 0xA9 are both ≥ 0x80 and not in the tchar byte set; an implementation using `char.is_alphanumeric()` would ACCEPT this (Unicode-aware — 'é' is alphanumeric); the byte-level tchar predicate correctly rejects it |
| header_scheme — overlong cookie name (EC-009-051, MED-005 / CWE-390) | `header_scheme = "cookie:" + 129 `a` characters` | `E-SPEC-027` template (a); spec rejected; tchar check passes (all `a`); length bound triggers (129 > 128); prevents opaque HTTP 431 from sensor API |
| auth_acquisition — token_exchange missing block (EC-009-034) | `auth_type = "token_exchange"`, no `[auth_acquisition]` | `E-SPEC-028(a)`; spec rejected |
| auth_acquisition — bearer_static with block (EC-009-037) | `auth_type = "bearer_static"`, `[auth_acquisition]` present | `E-SPEC-028(g)`; spec rejected |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| VP-059 | Spec validator: all errors collected (no fail-fast); warning-only specs return Ok — for any `SensorSpec` with N distinct validation errors (N >= 1), `validate_sensor_spec()` returns `Err(errors)` where `errors.len() == N`; for a spec with only warnings and no errors, returns `Ok(warnings)` (spec accepted); the function never returns early on the first error. Method: Proptest. Priority: P1. |
| VP-160 | Rule 9 cookie-name charset totality: `is_valid_cookie_name_tchar` accepts exactly the 77-character RFC 9110 §5.6.2 tchar set (26 uppercase + 26 lowercase + 10 digits + 15 specials: ``! # $ % & ' * + - . ^ _ ` | ~``); semicolons, bare equals, spaces, TAB (0x09), CTL bytes (0x00–0x1F, 0x7F), non-ASCII bytes (0x80–0xFF), and RFC 9110 §5.6.2 delimiters are all rejected — exhaustive proof across all 128 ASCII byte values; non-ASCII bytes (0x80–0xFF) structurally excluded per VP-160 §Feasibility Assessment (valid `&str` bytes cannot include standalone 0x80–0xFF). Crate: prism-spec-engine. Method: Kani. Priority: P0. Status: draft. |

## Traceability
| Field | Value |
|-------|-------|
| Stories | S-1.11, S-1.13, PLUGIN-MIGRATION-001-F, S-SPEC-ENV-VAR-001, S-DEMO-CROWDSTRIKE-MULTIREGION-001, S-SPEC-HTTP-METHOD-VALIDATION-001, S-5.04, S-WAVE-A-ENGINE-001, S-ADR054-WAVE-A-001 |
| L2 Capability | CAP-029 |
| Capability Anchor Justification | CAP-029 ("Config-Driven Sensor Adapters") per capabilities.md §CAP-029. This BC specifies spec-file validation — exactly what CAP-029 mandates: "Every spec file is validated at load time and reload time (DI-030). Variable references in step templates are resolved against the step dependency graph — forward references and undefined variables are validation errors (DEC-038)." Env-var token resolution (AC-6) is a prerequisite of that load-time validation: a spec whose `base_url` contains an unresolved `${env.VAR}` token cannot pass URL-format validation, so resolution must occur in the same spec-load pass. |
| L2 Invariants | DI-030 |
| L2 Entities | SensorSpec, TableSpec, ColumnSpec |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.30 | FB71 | 2026-07-27 | product-owner | F-CVA-HIGH-002: added `page_number` row to §Validation Rule 4. New row mirrors `offset_limit` rejection: `page_size == 0` in `PaginationConfig::PageNumber` is rejected at spec-load time by `validate_sensor_spec` §Category 4 with `SpecErrorCode::ESpec001` and message `"page_number pagination in step '{step_name}' requires page_size > 0"`. Grounding: ADR-056 §D10 CE-2; §D3 spec-load layer. Anchored: `S-WAVE-A-CYBERINT-SPEC-001` RG-017. POL-29 9a: no named twin for BC-2.16.009 (BC-2.16.002 is a distinct capability, not a split-event twin); 9b: BC-2.16.009 §Validation Rule 4 is a downstream copy target of ADR-056 §D8 §page_number row spec — both swept in this burst; 9c: `S-WAVE-A-CYBERINT-SPEC-001` RG-017 is the load-bearing anchor. |
| 1.29 | FB66 | 2026-07-27 | product-owner | F-WASE-P65-MED-005 (PO half): EC-009-049 expected-behavior column now states the escaped `{value}` form explicitly — TAB (0x09) is in the CTL escape class (`b <= 0x1F`), so the emitted error message `{value}` is `cookie:\x09` (eleven-character literal: `cookie:` + four-char ASCII sequence `\x09`), NOT a raw TAB byte. This mirrors EC-009-048 (LF 0x0A → `cookie:a\x0Ab` explicit statement) and makes the escaping obligation self-documenting in the EC. SP (0x20 > 0x1F) is NOT in the CTL escape class and is not escaped — EC-009-049 is the boundary case that tests both non-tchar rejection AND CTL escaping independently. Canonical test vector row for EC-009-049 updated in parallel with the same escaped-value statement. No new ECs added; no POL-38 obligation. BC-INDEX pin sync (1.28→1.29) deferred to state-manager. |
| 1.28 | FB56 | 2026-07-26 | product-owner | F-WASE-P64-MED-001: §Verification Properties VP-160 row corrected — "exhaustive proof across all 256 byte values" → "exhaustive proof across all 128 ASCII byte values; non-ASCII bytes (0x80–0xFF) structurally excluded per VP-160 §Feasibility Assessment". VP-160 proof harness bounds to a 128-point ASCII exhaustive space (`kani::assume(b <= 0x7F)` per VP-160 §Proof Harness Skeleton); non-ASCII rejection is by structural argument per VP-160 §Feasibility Assessment; the BC row was the lone outlier vs. the VP and VP-INDEX. F-WASE-P64-MED-003: E-SPEC-027 template (a) error message updated — "non-empty name required" → "non-empty name, ≤128 codepoints". Template (a) fires for both tchar charset violations AND the 128-codepoint length violation (EC-009-051); the previous message named only the charset constraint, giving actively misleading guidance for a 129-char all-tchar cookie name whose every character satisfies the only constraint named. Decision: length clause added to template (a) rather than a distinct template (d) — both failure modes share the same code path; a separate template adds spec complexity without operational benefit. Three POL-24 sites updated in this BC: §Rule 9 §Error message (syntactic) and §Error Conditions E-SPEC-027 template (a) Condition column (both changed). The `"cookie:<name>"` form definition bullet (site 3) already stated "must be ≤128 codepoints" from v1.26 and needed no change. Companion: error-taxonomy.md v2.70 (same template (a) change). F-WASE-P64-MED-004: §Invariants first bullet scoped — unqualified "always no-fail-fast" claim replaced with explicit per-function collect-all semantics: `validate_sensor_spec()` (Rules 1–5) and Rule 10's sub-conditions are collect-all; `SpecLoader::parse()` is fail-fast at rule boundaries (Rule 9 `Err` prevents Rule 10 executing). §Multi-Error Reporting first bullet updated for consistency. No new ECs added (EC-009-051 already existed); no POL-38 obligation created. |
| 1.27 | FB51b | 2026-07-25 | product-owner | F-WASE-P64-HIGH-008: §Integration function attribution corrected. The original sentence "S-WAVE-A-ENGINE-001 adds Rules 9 and 10 inside `SpecLoader::parse()`" was false — S-WAVE-A-ENGINE-001 scopes to Rule 9 only; Rule 10 is owned by S-ADR054-WAVE-A-001. Sentence split into two: (1) S-WAVE-A-ENGINE-001 adds Rule 9 inside `SpecLoader::parse()` — not inside `validate_sensor_spec()` — ensuring Rule 9 executes on every path that calls `parse()`; (2) S-ADR054-WAVE-A-001 adds Rule 10 inside `SpecLoader::parse()` by the same rationale, per ADR-054 §D10 — not inside `validate_sensor_spec()` — ensuring Rule 10 also executes on every path that calls `parse()`. Placement fact (both rules in `SpecLoader::parse()`, not `validate_sensor_spec()`) and coverage rationale preserved — both are correct and load-bearing. §Traceability Stories row: S-ADR054-WAVE-A-001 added (Rule 10 owning story was absent from the contract that defines Rule 10). TD-VSDD-060 sibling sweep: two live-body ENGINE-001 references in §Integration function and §Security requirement examined; §Security requirement paragraph is correctly scoped to Rule 9 only and is unchanged. |
| 1.26 | FB46 | 2026-07-25 | product-owner | F-WASE-P62-HIGH-001: "Entry points and function coverage (normative)" sub-section in Rule 9 rewritten against the verified call graph. False claim removed: `SpecLoader::parse()` does NOT call `validate_sensor_spec()`; the two functions are independent entry points. Corrected model: `validate_sensor_spec()` covers Rules 1–5 only; `SpecLoader::parse()` covers TOML deserialization + credential_refs Rule B + timestamp gates + Rule 8 — calls neither `validate_sensor_spec()` nor `resolve_env_var_tokens()` nor `validate_step_methods()`; `resolve_env_var_tokens()` covers Rule 6; `validate_step_methods()` covers Rule 7. Integration function `parse_and_validate_spec_toml()` documented. Security conclusion preserved: `add_sensor_spec` reaches `SpecLoader::parse()` as its first act (verified per `add_sensor_spec.rs` §parse_and_validate_spec_toml). Security requirement paragraph clarified: bypassing with only `validate_sensor_spec()` would miss Rules 8–10. §Validation Rule 8 implementation anchor corrected: Rule 8 runs at the end of `SpecLoader::parse()` before `Ok(spec)`; `validate_step_methods()` (Rule 7) is NOT called from within `parse()`. F-WASE-P62-HIGH-002 (security — CWE-400 / CWE-117): E-SPEC-027 template (a) `{value}` echo bounded at 64 codepoints via `truncate_at_char_boundary` (CWE-400) and CTL bytes (0x00–0x1F, 0x7F) replaced with `\xNN` uppercase-hex (CWE-117 log-injection). Cap 64: ≥2.4× longest realistic production value. EC-009-047 (overlong echo cap) and EC-009-048 (CTL escaping in error message) added with test vectors. POL-24: template (a) format string unchanged; companion error-taxonomy.md v2.69. F-WASE-P62-MED-005 (CWE-390): cookie `<name>` in `cookie:<name>` bounded at ≤128 codepoints. A 200 KiB all-tchar cookie name passes the character-class check but is injected verbatim into the `Cookie` header, producing opaque HTTP 431 (CWE-390). Rationale 128: ≥10× longest production cookie name (`"access_token"` = 12 chars); proportionally smaller than E-AUTH-006's 4096-byte cookie value bound. EC-009-051 added with test vector. §Error Conditions E-SPEC-027 updated to cite cookie name length bound. F-WASE-P62-MED-010: independent edge cases for TAB (0x09) and high bytes (0x80–0xFF) in cookie name added. EC-009-045 mentioned TAB parenthetically with no independent test vector. EC-009-049 (TAB) and EC-009-050 (high byte — U+00E9 'é' = 0xC3 0xA9) added as independent ECs with canonical test vectors. EC-009-050 is an implementation correctness probe: `char.is_alphanumeric()` would ACCEPT 'é' (Unicode-aware); the byte-level `is_tchar_byte` predicate correctly rejects it. F-WASE-P62-MED-004 (VP-160 row added): §Verification Properties extended with VP-160 — Kani exhaustive-proof that `is_valid_cookie_name_tchar` accepts exactly the 77-character RFC 9110 §5.6.2 tchar set and rejects all other byte values (CWE-20/CWE-74 security control). Priority P0, status draft. 77-character count consistent with §Validation Rule 9 enumeration (26+26+10+15 specials = 77; arithmetic verified against security-triage-rule9-cookie-name-charset.md §5). VP-059 row unchanged — its error-collection-semantics description does not reference rule coverage and remains accurate after the HIGH-001 entry-points correction. |
| 1.25 | FB45 | 2026-07-24 | product-owner | F-WASE-P61-HIGH-001: backtick (U+0060) restored to the RFC 9110 §5.6.2 tchar enumeration and tail ordering corrected to `^ _ ` \| ~` (15 special chars, RFC-ordered). The v1.24 message listed 14 special chars with wrong tail `~ \|`, omitting the backtick. Three authoritative sources agree against v1.24: (1) RFC 9110 §5.6.2 tchar includes `"^" / "_" / "\`" / "\|" / "~"` in that order; (2) security-triage-rule9-cookie-name-charset.md §5 item 2 explicitly mandates the 15-char set with ordering `` ^ _ ` \| ~ `` and states the permitted set is 77 characters (26+26+10+**15** = 77; the 14-char list yields 76); (3) S-WAVE-A-ENGINE-001 `is_valid_cookie_name_tchar` match arm includes `b'\`'`. Three live-body sites fixed: §Syntactic check `"cookie:<name>"` bullet, §Error message (syntactic), §Error Conditions E-SPEC-027 template (a) message (all use double-backtick code span to accommodate the literal backtick character). POL-24 byte-identity re-verified across all four sites (3 in this BC + 1 in error-taxonomy.md); all four now agree. Note: the v1.24 POL-24 claim was valid (BC↔taxonomy matched) but neither site matched RFC 9110 or the triage document — the missing backtick was introduced in v1.24 itself. TD-VSDD-060 sibling sweep: 4 live-body sites in `.factory/specs/` found and fixed; 2 changelog rows carry the old charset (this v1.24 row and taxonomy v2.67 row) — exempt per TD-VSDD-091. F-WASE-P61-HIGH-002: added `**Entry points and function coverage (normative):**` sub-section to Rule 9 — names the two-function split (`validate_sensor_spec()` covers Rules 1–7 only per VP-059 scope; `SpecLoader::parse()` covers Rules 1–10), asserts that `add_sensor_spec` MUST reach `SpecLoader::parse()` for Rule 9 coverage on the sole exploitable injection surface (security-triage-rule9-cookie-name-charset.md §1.3), and binds implementation verification to S-WAVE-A-ENGINE-001. Production-grade default per CLAUDE.md §6: no "pending architect review" placeholder; normative requirement stated inline. F-WASE-P61-MED-001: §Description count corrected "nine categories" → "ten categories" and probe table reference validation (Rule 8 — E-SPEC-026) added to the category list in rule order (after HTTP method whitelist, before header_scheme). Drift origin: v1.11 added Rule 8 without §Description update; v1.12 corrected 7→9 from a list that already omitted Rule 8 (so the count went 7→9 not 7→10). |
| 1.24 | sec-001-spec-amendment | 2026-07-24 | product-owner | SEC-001 (CWE-20/CWE-74, OWASP A03:2021) spec amendment — human-authorized reopening of Wave-A spec perimeter per security-triage-rule9-cookie-name-charset.md. Rule 9 cookie-name constraint strengthened from "no colon" (strict subset) to full RFC 6265 `cookie-name` / `token` tchar requirement (RFC 9110 §5.6.2): letters, digits, and `! # $ % & ' * + - . ^ _ ~ |`. The old constraint admitted `;`, `=`, SP, TAB, high bytes, and all RFC 9110 §5.6.2 delimiters; a `;` in `<name>` synthesized extra Cookie pairs (e.g., `cookie:sid=x; admin` → `Cookie: sid=x; admin={token}`), remapping the auth credential to an attacker-chosen key. Requires `sensor_spec.write` capability to exploit (multi-tenant MSSP active injection vector). Changes: (1) `"cookie:<name>"` constraint bullet updated to cite RFC 6265 / RFC 9110 §5.6.2 tchar; (2) examples sentence in Rule 9 updated to cite `"cookie:foo:bar"` as non-tchar rejection and add `"cookie:sid=x; admin"` as SEC-001 motivating example; (3) §Error message (syntactic) updated — removes `(non-empty name required, no colon in name)`, now reads `(non-empty name required, RFC 6265 token characters only: A-Z a-z 0-9 ! # $ % & ' * + - . ^ _ ~ |)`; (4) §Error Conditions E-SPEC-027 template (a) condition and behavior updated to cite tchar and SEC-001; (5) EC-009-030 rationale updated from "no colon in name" to "all characters are RFC 6265 tchar"; (6) EC-009-033 rationale updated from "contains colon" to "`:` is not a tchar character" with retirement note; (7) EC-009-043..046 added (`;=` injection, bare `=`, SP, CTL — SEC-002 side effect for CTL); (8) canonical test vectors for EC-009-043..046 added. POL-24 byte-identity verified: template (a) message is byte-identical across BC-2.16.009 §Error message (syntactic), BC-2.16.009 §Error Conditions E-SPEC-027 template (a), and error-taxonomy.md E-SPEC-027 template (a). POL-36 / Q3 non-implication: this is a character-legality check (RFC standard), NOT a sensor-specific cookie-name allowlist; any RFC 6265-compliant name (`access_token`, `session_id`, `api_key_1`) remains valid. SEC-002 (CWE-390 deferred `"builder error"` for CTL chars) eliminated as side effect — load-time tchar rejection replaces deferred runtime failure. ADR-053 not edited per architect ruling (§D2 delegates charset detail to BC-2.16.009); three ADR-053 sites echo the old constraint and require architect adjudication. BC-5.39.001 streak resets 3/3 → 0/3. |
| 1.23 | wave-a-spec-evolution-fix-burst-37 | 2026-07-24 | product-owner | F-WASE-P48-MED-003: BC-side alignment with ADR-053 v0.30 §D2 architect adjudication. `SensorSpec::header_scheme` is `Option<String>` with bare `#[serde(default)]` (NOT non-Option with `default_header_scheme()` → `"bearer"`). Absence is `None` post-deserialization; runtime default `"bearer"` is applied in `build_request()` via `as_deref()` matching `None | Some("bearer")` at execution time. ADR-053 v0.30 adds E-SPEC-027 template (c) for the `None` + `cookie_roundtrip` case (absent `header_scheme` for cookie_roundtrip was a wrong-header injection hazard). Changes: (1) Rule 9 opening paragraph replaced — single "passes silently for absent" sentence expanded to two-branch absence handling: absence path A (`None` + non-cookie_roundtrip auth_type → silent runtime `"bearer"` default, backward-compatible) and absence path B (`None` + `cookie_roundtrip` → E-SPEC-027 template (c) load-time error, message byte-identical to error-taxonomy.md per POL-24); (2) coherence matrix `cookie_roundtrip` row annotated — `header_scheme` REQUIRED, absence path A carve-out does NOT apply; (3) §Error Conditions E-SPEC-027 row updated — Condition now names three templates (a)/(b)/(c) with template (c) trigger and exact message; Behavior updated — absent IS an error for cookie_roundtrip (template c), NOT an error for all other auth_types (absence path A); (4) EC-009-042 added (absence path B edge case); (5) test vector for EC-009-042 added. POL-25 sweep: ADR-053 D5 manifest row (line 570) says "both message templates" — ADR cannot be edited; reported out-of-scope. No stories or VP files reference old absence wording. ADR-054 references to E-SPEC-027 describe it in template-count-agnostic terms; no out-of-scope fixes needed. |
| 1.22 | wave-a-spec-evolution-fix-burst-34 | 2026-07-23 | product-owner | F-WASE-P40-MED-001: Rule 10(c) trigger scope narrowed per ADR-054 v0.49 §D10(c) adjudication (token_exchange-GATED, narrow scope). Added `auth_type = "token_exchange"` predicate to (c)'s trigger condition — was "If `[auth_acquisition]` is present and `expiry_mode` is set"; now "If `auth_type = "token_exchange"` AND `[auth_acquisition]` is present AND `expiry_mode` is set". Added Note explaining that when `expiry_mode` appears on a non-`token_exchange` block, sub-condition (h) handles position-validity and (c) does NOT additionally fire for value-validity of a wrong-position field (impossible (c)∩(h) and (c)∩(g) co-fires confirmed). EC-009-038 description updated to include `auth_type = "token_exchange"` context ("`expiry_mode = "absolute_utc_string"` in `[auth_acquisition]`" → "`auth_type = "token_exchange"`, `expiry_mode = "absolute_utc_string"` in `[auth_acquisition]`"); expected-behavior cell unchanged. Companion: error-taxonomy.md v2.65. |
| 1.21 | wave-a-spec-evolution-fix-burst-24 | 2026-07-23 | product-owner | F-WASE-P26-MED-001: EC-009-036 counterfactual sentence corrected. Previous text "If auth_type were custom_via_plugin instead, (b) would NOT fire (non-declarative type), and in Branch B only (a) would fire for the missing block" was false: Rule 10(a) is gated on auth_type ∈ {oauth2_client_credentials, token_exchange}; custom_via_plugin fails that predicate so (a) cannot fire in either branch. Verified by 8-condition × 2-branch walk: Branch B (no block) → 0 sub-conditions fire — spec is valid with zero E-SPEC-028 errors; Branch A (block present) → only (g) fires (block on non-declarative auth_type). Replaced with: "If auth_type were custom_via_plugin instead, neither (a) nor (b) would fire — custom_via_plugin is non-declarative, does not require an [auth_acquisition] block, and legitimately uses auth_plugin; in Branch B (no block) the spec is valid with zero E-SPEC-028 errors (in Branch A, with a block present, (g) fires)." input-hash updated at commit time. |
| 1.20 | wave-a-spec-evolution-fix-burst-23 | 2026-07-23 | product-owner | F-WASE-P25-MED-001: EC-009-036 updated to document both branches of the oauth2+auth_plugin case. Branch A (WITH `[auth_acquisition]` and `token_path` present) emits only Rule 10(b) `E-SPEC-028`. Branch B (WITHOUT `[auth_acquisition]`) emits both Rule 10(a) `E-SPEC-028` (block absent on declarative type) AND Rule 10(b) `E-SPEC-028` (auth_plugin on declarative type) in the same multi-error pass — two distinct errors. Sweep result: Rule 10(b) body already says "a given sensor spec triggers at most one of (b) or (g), never both" — correctly scoped to (b)-(g) mutual exclusion only, no change needed. §Error Conditions E-SPEC-028 row and other ECs (EC-009-034..041) carry no false universal-disjointness claim. Companion: error-taxonomy.md v2.63. input-hash updated at commit time. |
| 1.19 | wave-a-spec-evolution-fix-burst-21 | 2026-07-23 | product-owner | F-WASE-P22-MED-001: §Description "nine categories" sentence amended — `header_scheme` validation and `[auth_acquisition]` coherence validation entries both annotated [PLANNED — Wave-A engine story per ADR-053 D2 / ADR-054 D10] (mirroring the Rule 1 `token_exchange` [PLANNED] precedent). Rule 9 header and Rule 10 header each annotated [PLANNED — Wave-A engine story per ADR-053 D2 / ADR-054 D10]. Sweep: §Error Conditions E-SPEC-027 and E-SPEC-028 Behavior columns each annotated [PLANNED — Wave-A engine story per ADR-053 D2 / ADR-054 D10] (these rows make present-tense claims about errors firing; without the annotation they implied the rules are already built). No other live-body present-tense claims that Rules 9/10 are implemented were found; Postconditions/Invariants/Multi-Error sections are generic and not Rule-9/10-specific. input-hash updated at commit time. |
| 1.18 | wave-a-spec-evolution-fix-burst-19 | 2026-07-23 | product-owner | F-WASE-P20-MED-001: Two misanchored ADR-054 D1 citations corrected to D10. (1) §Description line "per ADR-054 D1" — the `[auth_acquisition]` coherence validation rules are governed by ADR-054 D10 (not D1; D1 = token_exchange enum addition); changed to "per ADR-054 D10". (2) §Validation Rule 10 section header "Wave-A ADR-054 D1" → "Wave-A ADR-054 D10". D1 sweep verdict: all other ADR-054 D1 citations in .factory/specs/ correctly refer to the token_exchange enum addition (D1 = Add token_exchange to AuthType closed enum); no other mis-anchors found. input-hash updated at commit time. |
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
