# Adversarial Review — S-PLUGIN-PREREQ-C — LOCAL Pass 1

**HEAD**: 0e7d8bca
**Reviewer**: adversary (fresh-context)
**Date**: 2026-05-12
**Subject**: TOML Grammar Extensions + Pub-API Hardening — 7 ACs across prism-spec-engine + prism-core

---

## Critical Findings

### F-LP1-CRITICAL-001 — AC-5 Red Gate compile-fail test is functionally INERT (no CI invocation)
- **Severity**: CRITICAL
- **Category**: spec-compliance / process-gap / policy-11 (positive-coverage assertion) `[process-gap]`
- **Subject**: `tests/external/non-exhaustive-violation/` exists with a deliberate struct-literal violation crate, but NO CI workflow, Justfile recipe, or `just check` step invokes `cargo check -p non-exhaustive-violation` and asserts non-zero exit.
- **Evidence**:
  - File: `tests/external/non-exhaustive-violation/Cargo.toml` — `[workspace]` block is empty, opting out of parent workspace; comment says "CI pattern: assert non-zero exit code from `cargo check -p non-exhaustive-violation`" — but no such CI step exists.
  - File: `.github/workflows/ci.yml` — grep for "non-exhaustive" returns ZERO matches. Compare to the sibling `perimeter-violation` pattern which has a full CI job with `cargo check ... > /tmp/...log` then per-symbol assertion.
  - File: `Cargo.toml` — workspace `exclude` lists `tests/external/perimeter-violation` but NOT `tests/external/non-exhaustive-violation`. The non-exhaustive crate is "orphaned".
- **Why it's CRITICAL**: This is exactly the POL-11 / CI-as-Code policy violation. AC-5's entire forward-compat enforcement rests on the compile-fail test demonstrating that external struct-literal construction breaks. If that test never runs, regressions land unnoticed. Identical class of META-GAP as prism PR #127 pass-13 F-PG-001 (perimeter-compile-fail was inert for 12 passes).
- **Recommended fix**: Add a CI step to `.github/workflows/ci.yml` (modeled on the perimeter-violation job) that runs `cargo check --manifest-path tests/external/non-exhaustive-violation/Cargo.toml`, asserts exit code != 0, AND emits a positive-coverage log line ("Check passed: 8 types correctly reject external struct-literal construction"). The exit-1 result alone is insufficient; per POL-11 the log must positively assert N items validated.
- `[process-gap]` — applies to the CI workflow regression-detection discipline; consider lifting to a project rule: "any tests/external/<name>/ crate must be paired with a CI workflow invocation."

### F-LP1-CRITICAL-002 — AC-5 positional `::new()` constructors silently defeat the forward-compat purpose of `#[non_exhaustive]`
- **Severity**: CRITICAL
- **Category**: spec-compliance / backward-compat
- **Subject**: `SensorSpec::new`, `FetchStep::new`, `ColumnSpec::new`, `CredentialRef::new`, `SensorTableDescriptor::new`, `TableSpec::new` are all defined as POSITIONAL constructors that accept every field as a positional argument. The point of `#[non_exhaustive]` is to allow adding a field later without breaking external callers; positional `::new()` constructors break that property — adding a field requires changing the `::new()` signature, which IS a breaking change for every external caller.
- **Evidence**:
  - File: `crates/prism-spec-engine/src/spec_parser.rs` — `FetchStep::new` with 9 positional args, `ColumnSpec::new` with 4 positional args, `TableSpec::new` with 7 positional args, plus `CredentialRef::new`, `SensorSpec::new`, `SensorTableDescriptor::new`.
  - Story prescribes "use the `Default` impl or builder pattern."
  - Grep for `impl Default for` in `spec_parser.rs` returns ZERO matches. No Default impls exist. The story's prescribed alternative was not implemented.
- **Why it's CRITICAL**: This converts `#[non_exhaustive]` into a paper protection. External callers using `FetchStep::new(a, b, c, d, e, f, g, h, i)` will break the moment a 10th field is added — the SAME outcome they would have suffered from struct-literal construction. The forward-compat property the AC was designed to establish is structurally absent.
- **Recommended fix**: Either (a) add `impl Default` to all 8 types and document `..Default::default()` as the forward-compat external construction pattern, OR (b) replace positional constructors with a typestate builder (`SensorSpec::builder().sensor_id(...).name(...).build()`). Document in each type's rustdoc why struct-literal-with-`..Default::default()` is the canonical forward-compat construction. Then revise the compile-fail test to also exercise that the Default-based construction WORKS while struct-literal FAILS.

### F-LP1-CRITICAL-003 — AC-1(c) "page_size: None" test is a paper-fix (does NOT call build_paged_url)
- **Severity**: CRITICAL
- **Category**: paper-fix / test-sufficiency
- **Subject**: `test_BC_2_16_002_cursor_pagination_page_size_none_omitted` asserts that the BASE URL does not contain `page_size=` — but it constructs `let current_output = base_url.to_string();` (i.e., literally clones the input string `"https://api.example.com/v1/devices"`). The test does NOT invoke `build_paged_url_for_test()` at all in the None branch. Compare to AC-1(a)/(b) which DO call it.
- **Evidence**:
  - File: `crates/prism-spec-engine/tests/ac_1_cursor_page_size_test.rs` — `let current_output = base_url.to_string(); assert_url_omits_page_size(&current_output);` — the assertion runs on the raw `base_url` parameter passed in, not on the function output.
- **Why CRITICAL**: This is a textbook paper-fix per the PREREQ-A pass-3 / PREREQ-B pass-8 precedents. If you revert `build_paged_url_impl` to its pre-AC-1 state, this test still passes. The test does NOT exercise the production code path it claims to cover.
- **Recommended fix**: Rewrite the test to actually invoke `build_paged_url_for_test(base_url, &step_with_pagination_none, &None, 0)` and `build_paged_url_for_test(base_url, &step_with_pagination_none, &Some("cursor_xyz".to_string()), 0)`, asserting that NEITHER produced URL contains `page_size=`. The current implementation is tautological.

---

## Important Findings

### F-LP1-HIGH-001 — AC-2 bounds-check error detail is DISCARDED at call sites (observability silence)
- **Severity**: HIGH
- **Category**: spec-compliance / silent-failure
- **Subject**: `extract_at_path` (in pipeline.rs) returns `Err(String)` with detailed messages such as `"index 99 out of bounds: array has 3 elements in path '$.x[99]'"` — but every call site in pipeline.rs maps via `.map_err(|_| SpecEngineError::JsonPathExtractionFailed { sensor_id, step_name, path })`, DISCARDING the descriptive detail. The operator sees only `"response_path '$.x[99]' did not match response"` with no out-of-bounds explanation.
- **Evidence**: pipeline.rs `extract_at_path(...).map_err(|_| SpecEngineError::JsonPathExtractionFailed{...})` pattern at multiple call sites. `error.rs` `JsonPathExtractionFailed` has NO `detail` field. Story AC-2(d): "Bounds-checked: `$.x[99]` on a 3-element array returns a structured error (event_type emission — must update BC-2.16.002 catalog if new)". The implementer chose plain `Err(String)` and not `tracing::*!(event_type = ...)`, AND the structured error variant loses the detail. Operator gets neither structured event nor descriptive error.
- **Why HIGH**: A TOML-spec authoring bug (declared `$.devices[99]` against an API that returns 3 devices) will surface only as "response_path did not match" in production logs — operator must reproduce locally to discover the actual cause. This is a production diagnostic gap.
- **Recommended fix**: Add `detail: String` field to `SpecEngineError::JsonPathExtractionFailed` and populate from the `Err(String)` of `extract_at_path`: `.map_err(|e| SpecEngineError::JsonPathExtractionFailed { ..., detail: e })`. Alternatively (and preferred per PG-LP11-001 SOP), emit a structured `tracing::warn!(event_type = "jsonpath_extraction_failed", path = %path, detail = %e)` event at the call site AND add a 15th row to BC-2.16.002 catalog. Either path closes the silence; the catalog row approach is more consistent with BC v1.9 discipline.

### F-LP1-HIGH-002 — AC-3(c) proptest body is a hardcoded JSON value (not "ANY JSON string")
- **Severity**: HIGH
- **Category**: test-sufficiency / spec-compliance
- **Subject**: AC-3(c) requires `extract_at_path` totality "for any JSON string and any path string". The actual proptest in pipeline.rs (function `proptest_extract_at_path_totality`) fixes the body to a hardcoded 5-key JSON value and only varies the path. An adversarial body (deeply nested arrays, empty arrays, recursive shapes, special characters, RFC 6901 tilde escapes) is never exercised.
- **Evidence**: The `body` is the same `serde_json::json!({"devices": [...], "total": 2, "nested": {...}})` for every proptest iteration. Additionally, the path regex excludes `~` characters — RFC 6901 tilde escaping (a documented feature of `extract_at_path` per its doc-comment) is never exercised by the totality proptest.
- **Why HIGH**: Totality property is partial — the proptest cannot catch panics triggered by adversarial JSON shapes (e.g., recursive cycles via serde_json's `Map` types, or near-stack-limit nesting). The AC promised whole-input totality.
- **Recommended fix**: Add a `body in any::<serde_json::Value>().prop_map(...)` strategy (with depth limit to keep proptest fast) and expand the path regex to include `~` escapes. Alternatively, add a separate proptest for "ANY body" with depth-limited nested values.

### F-LP1-HIGH-003 — AC-4 escape grammar deviates from spec (doc-comment contradicts implementation)
- **Severity**: HIGH
- **Category**: spec-compliance / spec-drift
- **Subject**: AC-4 story says `$${var}` (double-dollar + brace) is the escape sequence; the doc-comment in interpolation.rs says "`$$` immediately before `{...}` is consumed as a single literal `$`." But `replace_double_dollar_escapes` consumes EVERY `$$` pair regardless of what follows. Input `"$$abc"` (no brace) produces `"$abc"` per the implementation, but per the doc-comment should produce `"$$abc"` (escape not anchored before `{`). The escape is context-free in the implementation but anchored in the spec.
- **Evidence**: `replace_double_dollar_escapes` is context-free `$$` consumption. Compare to doc-comment ("immediately before `{...}`") and story AC-4 escape semantics (which treat `$${...}` as the escape unit). Tests cover `$${var}` and `$$${var}` but NOT `$$<non-brace>`. The deviation is invisible to the AC-4 tests.
- **Why HIGH**: TOML spec authors writing prose with `$$1.99` (literal dollar amounts in a body_template) will see the double-dollar collapsed to a single-dollar unexpectedly. This is a real grammar surprise. If the implementation IS context-free (deliberately), the doc-comment must be corrected. If the doc-comment is the intent, the implementation is wrong.
- **Recommended fix**: Decide which is the canonical grammar:
  - (a) Keep context-free `$$` consumption (the implementation): then correct the doc-comment AND amend the story AC-4 narrative to remove the "immediately before `{...}`" anchoring claim.
  - (b) Anchor escape on `$${` only (the doc-comment + story): then rewrite `replace_double_dollar_escapes` to scan for `$${` (3-char prefix) and only consume the leading `$$` in that context.
  Add a test `test_AC4_escape_only_before_brace` that exercises `$$abc` and `$$1.99` to lock the chosen semantics.

### F-LP1-HIGH-004 — AC-5 audit scope is too narrow: 3+ duplicate type families NOT covered, RateLimitHints + types.rs siblings missed
- **Severity**: HIGH
- **Category**: spec-compliance / sibling-sweep (TD-VSDD-060 BROAD)
- **Subject**: AC-5 audited only 8 named types in `spec_parser.rs`, but the workspace has additional pub TOML-deserialized types that are NOT marked `#[non_exhaustive]`:
  1. `RateLimitHints` (spec_parser.rs) — pub, TOML-deserialized via `SensorSpec.rate_limit_hints`. Field expansion expected (request bucket policy, jitter). Not in AC-5 audit.
  2. `types::SensorTableDescriptor`, `types::CredentialRef`, `types::SensorSpec` — DUPLICATE definitions in `prism-spec-engine/src/types.rs`. All pub, TOML-deserialized, none marked `#[non_exhaustive]`.
  3. `infusion::CredentialRef` (infusion/mod.rs) — THIRD pub CredentialRef, also TOML-deserialized.
  4. `prism_core::ColumnType` and `prism_core::ColumnOptions` (column.rs) — used as fields of `ColumnSpec`, declared in prism-core; pub, TOML-deserialized (`#[serde(rename_all)]`); neither marked `#[non_exhaustive]`. Adding a new column type variant breaks external matchers.
- **Why HIGH**: The AC-5 audit promise was "ALL pub TOML-deserialized types in prism-spec-engine." Three sibling sweeps were missed inside that same crate. Adding "ColumnType::Binary" to support binary columns would break every external matcher today. The blast radius the AC was designed to close is still open.
- **Recommended fix**: Either (a) explicitly DOCUMENT in the story that the AC-5 audit scope is the 8 types in the table only (and file new TD items for the missed types), or (b) expand the audit to cover the 5+ missed types listed above. Triage decision: types.rs SensorTableDescriptor duplicate is genuinely confusing — investigate whether it should be deleted entirely (dead code) or whether it's a separate IPC type. Cross-newtype duplication of `CredentialRef` is a design smell worth a separate TD.

### F-LP1-HIGH-005 — AC-6 audit allowlist is file-suffix-keyed, allowing silent allowlisting of FUTURE new_unchecked entries in tenant.rs
- **Severity**: HIGH
- **Category**: security / spec-compliance
- **Subject**: The `new_unchecked_audit.rs` test allowlist (`GATED_OR_ALLOWLISTED_UNCHECKED`) keys on file SUFFIX (`"tenant.rs"`) and uses `relative.ends_with(allowed)`. Any future `fn new_unchecked` added to `tenant.rs` for ANY type (e.g., `Tenant::new_unchecked`, `OrgRef::new_unchecked`) is automatically allowlisted with no review.
- **Why HIGH**: This is a too-broad regression detector. If `tenant.rs` ever grows a second pub newtype with a `new_unchecked` constructor, that new validation-bypass constructor will be allowlisted silently — no code review trigger. This contradicts the AC-6 intent of "explicit inventory + justification per site."
- **Recommended fix**: Change the allowlist to be SYMBOL-keyed: store `(file_suffix, type_name_prefix)` tuples like `("tenant.rs", "OrgSlug")`. Add an exact-symbol parser that extracts the type name from the line context (e.g., the `impl Foo {` block above the `fn new_unchecked` line). Any future addition will require updating the allowlist explicitly.

### F-LP1-HIGH-006 — AC-6 production caller justification rests on undocumented OrgId Display invariant
- **Severity**: HIGH
- **Category**: backward-compat / security-precondition
- **Subject**: The AC-6 audit allowlist comment justifies `OrgSlug::new_unchecked(&format!("org-{}", &org_id.to_string()[..8]))` in `prism-query/src/materialization.rs` by asserting "UUIDs contain only hex digits and hyphens, and the 8-char prefix is always valid." That's TRUE only because `OrgId::Display` emits the standard hyphenated lowercase UUID format AND the first 8 chars of a UUID v7 are timestamp hex (always `[0-9a-f]`). If `OrgId::Display` ever changes (e.g., to Braced/Simple or with a new UUID version that puts a hyphen at position ≤ 7), the precondition breaks silently.
- **Why HIGH**: A latent security precondition that depends on a transitive crate's formatting behavior is exactly the class of invariant that drifts. If `OrgSlug` is ever derived from a string with hyphen at position ≤ 7 (e.g., a new UUID encoding or a buggy `&str[..8]` slice on multi-byte input), the slug enters the `Valid` state with INVALID content — silent invariant violation. The doc comment does not enforce; it asserts.
- **Recommended fix**: One of:
  - (a) Use `OrgSlug::new(&format!(...))` instead of `new_unchecked` and add explicit error handling (the validation cost is microseconds; the call site is a once-per-fan-out-target fallback, not a hot loop). This eliminates the precondition entirely.
  - (b) Add a `#[test]` in tenant.rs that asserts `OrgSlug::new(&format!("org-{}", &OrgId::new().to_string()[..8])).is_ok()` for at least 1000 random UUIDs — locks the invariant against transitive crate drift.
  - (c) Replace the synthetic-slug fallback with a hardcoded sentinel `OrgSlug::new("synthetic-unmapped").expect(...)` that is provably valid at compile time.

### F-LP1-HIGH-007 — AC-2 nested-wildcard memory amplification is unbounded (security)
- **Severity**: HIGH
- **Category**: security / performance
- **Subject**: `extract_with_tokens` (pipeline.rs) recurses on each array element when a `Wildcard` token is encountered, and PUSHES the recursive result into a `Vec`. For paths like `$.a[*].b[*].c[*]` against attacker-controlled JSON where a, b, c are large arrays, the result is a nested `Array(Array(Array([...])))` whose total size is O(|a| * |b| * |c|). No depth limit, no total-element limit. The `MAX_PIPELINE_RECORDS=10_000` cap applies only after extraction.
- **Why HIGH**: A TOML spec declaring `$.events[*].artifacts[*].iocs[*].hashes[*]` against a hostile API that returns 1000-element arrays at each level produces a 10^12-element result internally. Even if downstream truncation fires, the memory has already been allocated. Process OOM is the production outcome.
- **Recommended fix**: Add a `MAX_JSONPATH_RESULT_SIZE` constant (e.g., 100_000) and a `current_size: &mut usize` accumulator threaded through `extract_with_tokens`. On every push, increment and check against the cap; return `Err("JSONPath result exceeded N elements")` on overflow. Also add a `MAX_JSONPATH_DEPTH` (e.g., 32) to bound recursion depth. Add a test exercising `$.a[*].b[*]` on a 100x100 array and asserting the cap fires.

### F-LP1-HIGH-008 — AC-7 doctest is tautological (does NOT exercise `match` on the error type as the AC required)
- **Severity**: HIGH
- **Category**: paper-fix / test-sufficiency / spec-compliance
- **Subject**: AC-7 story prescribes a doctest "that demonstrates `use prism_core::SensorIdValidationError;` compiles and the error type can be matched." The actual doctest at `prism-core/src/lib.rs` is `let _: Option<SensorIdValidationError> = None;` — a type annotation with `None`. There is NO `match` expression, no enum variant access, no exercise of the error type's behavior. The doctest passes iff the `pub use` line compiles, which is tautological with the re-export's existence.
- **Why HIGH**: Classic paper-fix — revert `pub use sensor_id::SensorIdValidationError;` and the doctest fails because the type isn't in scope. So the doctest tests the re-export existing, which is what the SAME line declares. Circular. If `SensorIdValidationError` is renamed, the doctest fails to compile, which would catch some regressions — but it does NOT exercise the type's behavior contract, which was the AC promise.
- **Recommended fix**: Rewrite the doctest to construct + match on a specific variant: e.g., `let err = SensorIdValidationError::TooShort; match err { SensorIdValidationError::TooShort => {}, _ => panic!() }`. This exercises the type's pub variants AND validates that they remain matchable from outside the module.

---

## Observations (Medium/Low/OBS)

### F-LP1-OBS-001 — TD-VSDD-091 volatile-line-number citations in pipeline.rs comments (PREREQ-B inheritance)
- **Severity**: MEDIUM
- **Category**: process-gap / policy-violation `[process-gap]`
- **Subject**: pipeline.rs has 7 doc-comment citations using volatile line-numbers (PREREQ-B-era). PREREQ-C did not author them but touched the file (AC-1, AC-2, AC-3 production changes) which would shift line numbers, invalidating the citations.
- **Recommended fix**: Replace each citation with a stable anchor (function name) OR defer to a separate cleanup story.

### F-LP1-OBS-002 — Story EC-001 (page_size=0) explicitly unaddressed; implementer chose silent verbatim append, inconsistent with OffsetLimit's documented ">0" precondition
- **Severity**: MEDIUM
- **Category**: edge-case / spec-compliance
- **Subject**: Story EC-001 says page_size=0 is implementer's choice. The implementation silently appends `page_size=0` to the URL. `OffsetLimit::page_size` doc-comment says "must be > 0" — but CursorToken::page_size accepts 0 with no validation. Inconsistent treatment across the same enum's variants.
- **Recommended fix**: Either add `if n == 0 { return Err(...) }` at the top of `build_paged_url_impl`'s CursorToken arm, or explicitly document in CursorToken::page_size's doc-comment that `Some(0)` is accepted and forwarded.

### F-LP1-OBS-003 — AC-3(d/e) proptest variable strategy is too narrow (no malformed templates)
- **Severity**: MEDIUM
- **Category**: test-sufficiency
- **Subject**: `template_strategy()` only generates well-formed `${step.field}` references. AC-3(d) says "for any template string." Malformed templates (`${unbalanced`, `${.empty_step}`, `$`, empty string, templates with `$$` escapes) are NOT exercised.
- **Recommended fix**: Add an `arbitrary_template_strategy()` using `"\\PC*"` (any printable char sequence) and assert no panic.

### F-LP1-OBS-004 — `proptest_extract_at_path_totality_sentinel` is a no-op stub passed off as a test (fragile design)
- **Severity**: LOW
- **Category**: test-sufficiency
- **Subject**: tests/proptest_AC_3.rs declares `proptest_extract_at_path_totality_sentinel` with an empty body. The comment delegates the real proptest to pipeline.rs's in-module proptest. If the in-module proptest is removed/renamed during a refactor, the sentinel still passes — no compile error, no test failure.
- **Recommended fix**: Replace the empty body with a compile-time existence check OR remove the sentinel.

### F-LP1-OBS-005 — AC-5 `pub fn new_point_in_time()` constructor exists on TableSpec but NOT on sibling types (consistency)
- **Severity**: LOW
- **Category**: docs / design-consistency
- **Subject**: TableSpec has two constructors (`new` + `new_point_in_time`); FetchStep, SensorSpec, ColumnSpec, CredentialRef, SensorTableDescriptor have only `new`. The asymmetry is undocumented.

### F-LP1-OBS-006 — AC-3 in-module proptest is `#[cfg(test)]` only; not accessible via test-helpers feature for crate-external invocation
- **Severity**: LOW
- **Category**: test-design

### F-LP1-OBS-007 — Self-referential dev-dep `prism-spec-engine = { path = ".", features = ["test-helpers"] }` may break `cargo publish` (forward-looking)
- **Severity**: LOW
- **Category**: backward-compat

---

## Novelty Assessment

This is pass 1. All findings are NEW (no priors). Novelty: HIGH — multiple substantive defects across CRIT/HIGH bands. The most surprising find is F-LP1-CRITICAL-001 (AC-5 CI-inert Red Gate) — this is the same META-GAP class as the prism PR #127 perimeter-compile-fail incident, recurring here for a different AC.

The AC-5 positional-constructor anti-pattern (F-LP1-CRITICAL-002) is a real design defect — the `#[non_exhaustive]` annotations are present and the compile-fail crate exists, but the public ergonomic constructors silently defeat the forward-compat property.

The paper-fix in AC-1(c) (F-LP1-CRITICAL-003) is the exact precedent flagged in the user's brief — confirms the value of the paper-fix-detection rubric.

---

## Total Findings by Severity

| Severity | Count |
|----------|-------|
| CRITICAL | 3 (F-001, F-002, F-003) |
| HIGH | 8 (F-001 through F-008 in HIGH band) |
| MEDIUM (OBS) | 3 (OBS-001, OBS-002, OBS-003) |
| LOW (OBS) | 4 (OBS-004, OBS-005, OBS-006, OBS-007) |
| **Total** | **18** |

## 3-CLEAN Streak Status

0/3 (this is pass 1; 3 CRIT findings + 8 HIGH findings prevent CLEAN status).

## Trajectory

PREREQ-A: 14→12→6→4→2→6→4→0(false)→4→0→0→0 (12 passes)
PREREQ-B: 20→10→4→7→10→9→8→4→4→2→3→3→2→0→0→0 (16 passes)
**PREREQ-C: 18 at pass-1**

## Verdict

**BLOCKED-soft** — 3 CRITICAL + 8 HIGH findings require fix-burst. AC-5 CI gap (F-LP1-CRITICAL-001) and positional `::new()` design defect (F-LP1-CRITICAL-002) are the two highest-impact items.
