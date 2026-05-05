---
document_type: behavioral-contract
level: L3
version: "1.8"
status: draft
producer: product-owner
timestamp: 2026-04-14T07:00:00
phase: 1a
origin: greenfield
subsystem: "SS-11"
capability: "CAP-015"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "412c872"
traces_to: ["CAP-015"]
extracted_from: ".factory/specs/prd.md"
restricted_symbols:
  description: "Symbols that MUST be pub(crate) or private. External crates referencing any of these MUST fail to compile. Validated by tests/external/perimeter-violation/src/main.rs."
  symbols:
    - prism_query::filter_parser::parse_filter
    - prism_query::sql_parser::parse_sql
    - prism_query::pipe_parser::parse_pipe
    - prism_query::filter_parser::build_predicate_parser
    - prism_query::filter_parser::build_source_ref_parser
    - prism_query::filter_parser::build_string_parser
    - prism_query::filter_parser::build_literal_parser
    - prism_query::filter_parser::build_expr_parser
    - prism_query::filter_parser::build_pipe_mode_parser
    - prism_query::pipe_parser::build_pipe_parser
    - prism_query::security::ParseLimits::install_thread_local
    - prism_query::security::ParseLimits::clear_thread_local
    - prism_query::security::ParseLimits::current_regex_limit
---

# BC-2.11.006: Query Security Limits Enforcement

## Description

This BC defines the complete set of security limits that constitute DI-019. Seven limits are enforced at different stages of the query lifecycle: query length (64KB, pre-parse), nesting depth (64, parse-time recursive counter), pipe stages (32, post-parse), materialized records (10K, streaming counter during fan-out), execution timeout (30s, tokio::time::timeout wrapping the full lifecycle), regex pattern length (1024 bytes, parse-time), and integer overflow prevention (i128 intermediates). All limits are configurable via TOML with the listed values as defaults. Every limit violation returns a structured error with the specific limit name, actual value, maximum, and actionable suggestion.

## Preconditions
- An PrismQL query string has been received via the `query` MCP tool

## Postconditions
- The following security limits are enforced at the specified stages:
  1. **Query length (64KB max)**: Checked before any parsing begins. Applies to the expanded query (after alias resolution).
  2. **Nesting depth (64 max)**: Tracked via a depth counter in the recursive Chumsky parser. Each nested parenthesis or boolean combinator increments the counter.
  3. **Pipe stages (32 max)**: Counted after pipe mode parsing completes.
  4. **Materialized records (10K max)**: Enforced during sensor fan-out collection, before Arrow RecordBatch conversion begins.
  5. **Query timeout (30s)**: Enforced via `tokio::time::timeout` wrapping the entire query execution lifecycle (alias resolution through result serialization).
  6. **Regex pattern length (1024 bytes max)**: Checked at parse time for `matches` predicates. Regex engine is Rust `regex` (finite automaton, CWE-1333 immune).
  7. **Integer overflow prevention**: Arithmetic uses i128 intermediate representation (CWE-190).
- Each limit violation returns a structured error with: the specific limit name, the actual value, the maximum allowed value, and an actionable suggestion
- All limits are configurable via TOML config with the above values as defaults
- **Security Perimeter (pub(crate) enforcement):** The `prism-query` crate exposes a single public security entry point: `PrismQlParser::parse(input: &str) -> Result<Ast, Vec<ParseError>>`. All sub-parsers (`parse_filter`, `parse_pipe`, `parse_sql`) and parser-builder factories (`build_predicate_parser`, `build_source_ref_parser`, `build_string_parser`, `build_literal_parser`, `build_expr_parser`, `build_pipe_mode_parser`, `build_pipe_parser`, etc.) are crate-private (`pub(crate)`). External callers MUST NOT be able to bypass the seven security guards (`query_size`, `paren_depth`, `predicate_nesting_depth`, `expr_nesting_depth`, `sql_query_nesting_depth`, `list_size`, `pipe_stage_count`) by calling sub-components directly.

  > **Note:** `build_filter_parser` (filter_parser.rs:366) and `build_sql_parser` (sql_parser.rs:158) are NOT listed above because they are private (`fn`, not `pub(crate)` or `pub`). They are inaccessible by Rust visibility regardless of perimeter enforcement. If a future story promotes either to `pub(crate)`, BC-2.11.006 must be amended to add it to the `restricted_symbols` list.

  **Enforcement layers:**
  1. **Rust visibility (primary):** All sub-parsers and builder factories are `pub(crate)`. External crates referencing them produce a Rust visibility error during `cargo build`.
  2. **Clippy lint (defence-in-depth, intra-crate only):** The `crates/prism-query/clippy.toml` `disallowed-methods` list flags accidental intra-crate misuse during `cargo clippy`. Note: this lint is per-crate by Cargo design and does NOT propagate to downstream crates.
  3. **API surface integration test:** `crates/prism-query/tests/api_surface.rs` exercises the public API and confirms only `PrismQlParser::parse` is callable.
  4. **CI gate (negative test):** `.github/workflows/ci.yml` job `perimeter-compile-fail` builds the `tests/external/perimeter-violation/` external test crate using `cargo check`. The crate imports forbidden symbols (sub-parsers, builder factories, thread-local API). The job ASSERTS `cargo check` fails with `error[E0603]: ... is private`; if the build succeeds, the job fails the PR with a security regression error.

  **Pre-merge regression check:** `cargo build --workspace` must compile cleanly; any external crate adding a forbidden import (e.g., `prism_query::filter_parser::parse_filter`) will produce a hard visibility error and fail the build. The CI gate `perimeter-compile-fail` (`.github/workflows/ci.yml`) builds `tests/external/perimeter-violation/` with `cargo check` and asserts the build fails — if it compiles successfully, the job fails the PR as a security regression.

## Invariants
- DI-019: All limits defined in this BC constitute the DI-019 invariant
- **INV-SEC-PERIMETER-001 (lifts DI-034):** `prism-query` exposes only `PrismQlParser::parse` as a public security boundary. No sub-parser or parser-builder factory is accessible from outside the crate. Violation of this invariant allows callers to bypass one or more of the seven security guards, constituting a security defect. Cross-reference: `domain-spec/invariants.md#di-034-prism-query-security-perimeter`.

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-QUERY-003` | Query length exceeds 64KB | `"Query is {N} bytes (max 65536). Simplify the query or use aliases to reduce length."` |
| `E-QUERY-003` | Nesting depth exceeds 64 | `"Query nesting depth is {N} (max 64). Reduce nested parentheses or boolean expressions."` |
| `E-QUERY-003` | Pipe stages exceed 32 | `"Query has {N} pipe stages (max 32). Combine operations or simplify the pipeline."` |
| `E-QUERY-005` | Materialized records exceed 10K (streaming counter) | `"Fetched {N} records (max 10000). Narrow by: time range, client, sensor, or severity."` |
| `E-QUERY-004` | 30s timeout exceeded | `"Query timed out after 30s. Narrow scope to reduce execution time."` |
| `E-QUERY-003` | Regex pattern exceeds 1024 bytes | `"Regex pattern is {N} bytes (max 1024). Simplify the pattern."` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-11-015 | Query is exactly 64KB | Valid (limit is strictly greater than) |
| EC-11-016 | Alias expansion pushes query over 64KB | Error after expansion, before parsing; error message mentions alias expansion as the cause |
| EC-11-017 | Timeout fires during sensor API fan-out (before DataFusion execution) | Same timeout error; the timeout covers the entire lifecycle |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| Query of exactly 65536 bytes | Valid; no error | edge-case |
| Query of 65537 bytes | `Err(E-QUERY-003)` length exceeded | error |
| Query with 65 levels of nesting | `Err(E-QUERY-003)` depth exceeded | error |
| Query with 33 pipe stages | `Err(E-QUERY-003)` pipe stage limit | error |
| `matches` predicate with 1025-byte pattern | `Err(E-QUERY-003)` regex length exceeded | error |
| External test crate `tests/api_surface.rs` calls `prism_query::filter_parser::parse_filter(input)` | Compile error: `function \`parse_filter\` is private` (or equivalent Rust visibility error) — `cargo build --workspace` fails | security-perimeter |
| External test crate calls `prism_query::filter_parser::build_predicate_parser()` | Compile error: `function \`build_predicate_parser\` is private` (or equivalent Rust visibility error) — `cargo build --workspace` fails | security-perimeter |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-014 | Query security limits: rejects oversized queries | kani |
| VP-015 | Query security limits: rejects excessive nesting depth | kani |
| VP-021 | PrismQL parser never panics on arbitrary input | fuzz |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 |
| L2 Invariants | DI-019, DI-034 |
| L2 Edge Cases | DEC-023, DEC-026 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.8 | pass-6-remediation | 2026-05-05 | product-owner | F-MEDIUM-001 — added 4th enforcement layer (CI gate perimeter-compile-fail, now implemented). F-LOW-001 — footnote distinguishing private build_*_parser from pub(crate) ones. OBS-001 part — added structured `restricted_symbols:` frontmatter for machine-checkable perimeter validation. PR-127 adversary pass-6 remediation. |
| 1.7 | pass-5-remediation | 2026-05-05 | product-owner | F-MEDIUM-001 — corrected clippy.toml enforcement claim (per-crate scope, not workspace-wide; cargo build does not run clippy). Layered enforcement now accurately described: Rust visibility (primary), clippy intra-crate (defence), api_surface test (CI). F-MEDIUM-002 — INV-SEC-PERIMETER-001 now cross-references DI-034 (lifted by business-analyst). L2 Invariants traceability updated: DI-019, DI-034. PR-127 adversary pass-5 remediation. |
| 1.6 | pass-4-obs-002 | 2026-05-05 | product-owner | Add Security Perimeter postcondition (per adversary pass-4 OBS-002 process-gap). Codifies that prism-query exposes only PrismQlParser::parse; sub-parsers and builders are pub(crate) and lint-denied via clippy.toml disallowed-methods. Adds INV-SEC-PERIMETER-001 invariant and two compile-failure test vectors for api_surface.rs. Refs PR-127. |
| 1.5 | pass-87-remediation | 2026-04-21 | architect | F87-001: VP-021 Proof Method corrected proptest → fuzz (matches VP-INDEX, VP-021 frontmatter, verification-architecture, coverage-matrix). |
| 1.4 | pass-86-remediation | 2026-04-21 | architect | F86-007: added VP-021 row to Verification Properties table. |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract |
