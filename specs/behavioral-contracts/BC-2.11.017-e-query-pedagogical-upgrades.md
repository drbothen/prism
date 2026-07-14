---
document_type: behavioral-contract
level: L3
version: "1.14"
status: active
producer: product-owner
timestamp: 2026-06-19T00:00:00Z
phase: 1a
inputs: [".factory/specs/domain-spec/capabilities.md", ".factory/specs/domain-spec/invariants.md", ".factory/specs/architecture/decisions/ADR-041-prismql-llm-auto-onboarding-4-layer-teaching-surface-for-automatic-agent-query-authoring.md"]
input-hash: "TBD"
traces_to: ["CAP-015"]
extracted_from: null
origin: greenfield
subsystem: "SS-11"
capability: "CAP-015"
lifecycle_status: active
introduced: 2026-06-19
modified: 2026-07-14
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.11.017: E-QUERY Pedagogical Enrichments (L4 — Codes 001, 002, 003, 037)

## Description

Four existing E-QUERY error codes are enriched with additional actionable fields to complete the L4 pedagogical self-correction loop defined in ADR-041. Each enrichment adds structured data that an LLM agent can read to self-correct its PQL query without human intervention: `near_text` and a reference pointer for E-QUERY-001 (parse errors), `valid_operators_for_type` for E-QUERY-002 (type errors), `how_to_fix` for E-QUERY-003 (security limit violations), and `did_you_mean` for E-QUERY-037 (table unavailable — already pedagogically rich; this adds the fuzzy-match suggestion field that was specified in ADR-041 but whose implementation requirement is explicitly captured here). These are ADDITIVE enrichments to existing error codes — no existing fields are removed or renamed; no new error codes are allocated.

## Preconditions

1. A PQL query has been submitted to the `query` or `explain_query` tool.
2. The query triggers one of the four target error codes during parsing or planning.
3. The relevant `PrismError` variants already exist in `prism-core/src/error.rs` with display strings that include their respective `E-QUERY-NNN` prefixes.

## Postconditions

### E-QUERY-001 (parse error) — additive fields

**Current state:** `"E-QUERY-001: query parse error at offset {offset}: {detail}"` (emitted by `PrismError::QueryParseFailed` in prism-core/src/error.rs) — message only.

**New postcondition additions:**

The E-QUERY-001 structured error response (BC-2.10.007 format) MUST include:

1. `near_text: String` — the offending token or short substring (≤ 50 characters) near the parse failure position. This is the text that caused the parse error. Sourced from the Chumsky parser's error context at the byte offset `{offset}`. If the parser cannot provide a token (e.g., unexpected end of input), `near_text` is the empty string `""`.

2. `reference_pointer: "prismql://reference"` — a literal string in the error response pointing to the grammar reference resource. This allows the LLM agent to fetch the full grammar when it cannot self-diagnose from the error message alone. The field value is the static string `"prismql://reference"` (the MCP resource URI registered in BC-2.10.014).

**Implementation note:** The `PrismError::QueryParseFailed` variant does not need new Rust fields — the `near_text` and `reference_pointer` additions are extracted at MCP error-map time from the variant's existing `query` field by `error_mapping.rs` (BC-2.11.017 AC-003). The display string `"E-QUERY-001: query parse error at offset {offset}: {detail}"` is UNCHANGED — the new fields are additional structured metadata in the MCP structured error response, not changes to the Display.

**Injection-safety:** `near_text` is a substring of the model's own PQL query string (the input it submitted). It does not contain sensor data. The `near_text` field MUST be truncated to ≤ 50 characters to prevent log bloat and to discourage relay of raw model PQL text as error context.

### E-QUERY-002 (type error) — additive field

**Current state (as shipped, S-DEMO-PRISMQL-ONBOARDING-001-B):** E-QUERY-002 is now emitted by `PrismError::QueryTypeMismatch { column, table, actual_type, operator }` (inline variant, `prism-core/src/error.rs`) with Display:

`"E-QUERY-002: type mismatch — column '{column}' in table '{table}' has type '{actual_type:?}' which does not support operator '{operator}'"`

This variant was introduced by S-DEMO-PRISMQL-ONBOARDING-001-B as part of the ADR-041 L4 pedagogical enrichment (CORRECTION-2 adjudication, architect decision). It is an inline variant on the already-`#[non_exhaustive]` `PrismError` enum — the compile-fail gate count does not change (+0 gate) because `PrismError` is already gated.

**Note on pre-existing Display format:** The pre-v1.92 error-taxonomy row described `"Type error: field '{field}' is {actual_type}, cannot use {operator}"` for E-QUERY-002. That format matched neither the shipped `QueryTypeMismatch` Display nor the pre-existing `PrismError::QueryPlanFailed` Display (`"E-QUERY-002: query planning failed: {detail}"`). The taxonomy row was stale spec-only prose and was superseded by the v1.92 dual-Display collision row in error-taxonomy.md. See error-taxonomy.md v1.94 §E-QUERY-002 collision row for the complete live-emitter audit (both `QueryPlanFailed` and `QueryTypeMismatch` emit E-QUERY-002; blast-radius renumbering to separate them is deferred to a future maintenance story).

**New postcondition addition:**

The E-QUERY-002 structured error response (for the `QueryTypeMismatch` condition) MUST include:

`valid_operators_for_type: [String]` — an array of operator strings that ARE valid for `{actual_type}`. For example: if `actual_type` is `"String"`, then `valid_operators_for_type` includes `["=", "!=", "LIKE", "IN", "NOT IN"]`; if `actual_type` is `"Integer"`, then `["=", "!=", "<", ">", "<=", ">=", "BETWEEN", "IN", "NOT IN"]`; etc.

The complete per-type operator table is fixed at compile time (it reflects PQL semantics, not per-client config):

| ColumnType | Valid operators |
|------------|----------------|
| `String` | `=`, `!=`, `LIKE`, `IN`, `NOT IN` |
| `Integer` | `=`, `!=`, `<`, `>`, `<=`, `>=`, `BETWEEN`, `IN`, `NOT IN` |
| `Float` | `=`, `!=`, `<`, `>`, `<=`, `>=`, `BETWEEN` |
| `Boolean` | `=`, `!=` |
| `Datetime` | `=`, `!=`, `<`, `>`, `<=`, `>=`, `BETWEEN` |
| `Json` | `=`, `!=` (top-level), path-access (implementation-defined) |

**Implementation note:** A `fn valid_operators_for_type(t: ColumnType) -> &'static [&'static str]` helper in `prism-query` (or `prism-core`) is sufficient. The Display string for `QueryTypeMismatch` is the shipped format above — it is the authoritative contract for the `QueryTypeMismatch` path and must not be changed without a corresponding taxonomy amendment.

### E-QUERY-003 (security limits) — additive field

**Current state:** `"E-QUERY-003: {limit_detail}"` — single string describing the limit violation.

**New postcondition addition:**

The E-QUERY-003 structured error response MUST include:

`how_to_fix: String` — a human/model-readable actionable suggestion for resolving the specific limit that was violated. The suggestion is determined by the `{limit_detail}` category:

| Limit violated | `how_to_fix` value |
|----------------|--------------------|
| Query size > 64KB | `"Shorten the query. Remove large IN (...) lists or break into multiple queries."` |
| Nesting depth > 64 | `"Flatten nested conditions. Use AND/OR instead of deeply nested parentheses."` |
| Pipe stage count > 32 | `"Reduce the number of pipe stages. Combine adjacent filter conditions."` |
| Regex pattern > 1024 bytes | `"Use a shorter regex pattern. Consider using LIKE instead of regex for simple pattern matching."` |
| Expanded query > 64KB (after alias expansion) | `"The alias expansion produced a query over 64KB. Simplify the aliased query or use a narrower alias."` |

If the limit category is not recognized (implementation catch-all), `how_to_fix` is `"Simplify or shorten the query."`.

**Implementation note:** The `PrismError::QuerySecurityLimitExceeded { detail }` variant currently carries a single `detail: String`. The `how_to_fix` field can be computed at error-map time from the `detail` string (pattern-matching the known limit prefixes) and added to the structured response without modifying the `PrismError` variant. The display string is UNCHANGED.

### E-QUERY-037 (table unavailable) — additive field (did_you_mean)

**Note:** E-QUERY-037 already carries `did_you_mean` as part of its implementation per ADR-039 and BC-2.11.001 v1.9. This postcondition REAFFIRMS the `did_you_mean` requirement as an explicit L4 pedagogical contract element and adds the reference to `prismql://reference` in the error suggestion.

The E-QUERY-037 structured error response ALREADY includes (per BC-2.11.001):
- `available_sensors` (org-scoped)
- `available_tables` (org-scoped)
- `did_you_mean` (optional, Levenshtein ≤ 3 over org-visible tables)

**New postcondition addition (additive only):**

The E-QUERY-037 structured error `suggestion` field MUST include a reference to `prism_describe` as the discovery path:

`suggestion: "Call prism_describe('<client_id>') to see available tables and columns. If you meant '<did_you_mean_value>', retry with that table name."` (when `did_you_mean` is present)

OR:

`suggestion: "Call prism_describe('<client_id>') to see available tables and columns for this client."` (when `did_you_mean` is absent)

This adds the `prism_describe` discovery pointer to the E-QUERY-037 self-correction path, closing the loop: table-not-found error → model reads suggestion → model calls `prism_describe` → model discovers real tables → model retries.

**Implementation note:** The `suggestion` field is already part of the BC-2.10.007 structured error envelope. The E-QUERY-037 error handler in `prism-query/src/engine.rs` (or wherever `PrismError::TableNotAvailable` is constructed) already populates this field; it should be updated to include the `prism_describe` pointer text.

### PrismError variant impact summary

Three of the four enrichments are additive to existing error handling without new variants:
- E-QUERY-001: new fields in structured response builder (not new variant)
- E-QUERY-003: new `how_to_fix` field in structured response builder (computed from `detail`)
- E-QUERY-037: updated `suggestion` string text (existing field, new content)

**E-QUERY-002: ONE new `PrismError` variant WAS added** — `PrismError::QueryTypeMismatch { column, table, actual_type, operator }` (inline variant, `prism-core/src/error.rs`). This was introduced by S-DEMO-PRISMQL-ONBOARDING-001-B per the CORRECTION-2 adjudication (architect decision): the initial BC body assumed the `valid_operators_for_type` field could be computed at error-map time from an existing `QueryPlanFailed { detail }` catch-all, but the implementation determined that a dedicated structured variant was required to carry the four typed fields (`column`, `table`, `actual_type`, `operator`) needed to populate both the Display and the `valid_operators_for_type` array correctly. The variant is inline on the already-`#[non_exhaustive]` `PrismError` enum — the compile-fail gate count is unchanged (+0 gate).

No new E-QUERY codes are allocated. The error taxonomy rows for 001, 002, 003, and 037 are updated to document the new fields. The E-QUERY-002 row in particular was updated at v1.92 to document the dual-Display collision between `QueryPlanFailed` and `QueryTypeMismatch` (see error-taxonomy.md v1.94 §E-QUERY-002).

## Invariants

- DI-004: All four error codes are emitted during `query` or `explain_query` tool calls; the `AuditEntry` for the tool call already captures the rejection.
- DI-019: The security limits themselves (E-QUERY-003) are not relaxed by this enrichment. The `how_to_fix` field is guidance for the model, not a new limit or exception path.
- DI-006: `near_text` (E-QUERY-001) is a substring of the model's own PQL input — not sensor data. Truncated to ≤ 50 characters. No sensor API response data flows into any of these enrichment fields.
- DI-002: None of these enrichment fields (`near_text`, `valid_operators_for_type`, `how_to_fix`, updated `suggestion`) contain credential values or internal API URLs.
- **SIBLING-GATE CONSISTENCY cross-reference (BC-2.11.016 v1.25):** The E-QUERY-002 type-compat gate (`check_operator_type_compatibility`) runs as a sibling gate inside the same `check_pipe_stage_columns` binding-context walk as E-QUERY-038. When a column name is bound with DERIVED provenance in the running binding context (i.e., it is a stats output alias, enrich output column, SqlPipe head alias, any name that shadows a raw-schema column under a derived context, or a JOIN-qualified bare-`Field` SELECT item seeded via LAST-SEGMENT OUTPUT-NAME RULE (BC-2.11.016 v1.17) — per BC-2.11.016 §Preconditions DERIVED-COLUMN BINDING RULE SIBLING-GATE CONSISTENCY and LAST-SEGMENT OUTPUT-NAME RULE clauses), the E-QUERY-002 gate MUST fail open for that name (skip type-compat checking). Applying the raw-schema type to a DERIVED name produces a false E-QUERY-002 on a query that would succeed at execution (FP-001 violation). This constraint is co-defined in BC-2.11.016 v1.25 SIBLING-GATE CONSISTENCY clause and cannot be tested in isolation against this BC — it is a cross-gate constraint enforced by the shared binding-context walk. **STAR-WITH-JOIN SUSPENSION RULE extension (BC-2.11.016 v1.18):** when the initial binding context for a SqlPipe pipe-stage walk is SUSPENDED at the outset via the STAR-WITH-JOIN SUSPENSION RULE (branches (a) and (c) with a non-empty JOIN list and at least one `Star`/`TableStar`), the E-QUERY-002 gate is also suspended for ALL subsequent pipe stages — `suspended = true` disables both the E-QUERY-038 existence gate and the E-QUERY-002 type-compat gate simultaneously. This is not a new exception but an application of the existing suspension-propagation rule to a new suspension trigger; it is noted here for implementer clarity. **STAGE-JOIN SUSPENSION RULE extension (BC-2.11.016 v1.19):** when a `PipeStage::Join` stage is encountered during the pipe-stage walk, `suspended := true` engages via the STAGE-JOIN SUSPENSION RULE — the same suspension-propagation rule applies; E-QUERY-002 is also suspended for all remaining stages, not just E-QUERY-038. This is a parallel application of the same suspension-propagation principle as the STAR-WITH-JOIN extension above, applied to the stage-level join trigger instead of the head-level trigger; it is noted here for implementer clarity.

## Error Cases

These postconditions describe enrichments to existing error codes, not new error codes. No new error cases are introduced.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-11-046 | E-QUERY-001 parse error at end-of-input (model submitted an incomplete query) | `near_text: ""` (empty string — no offending token to show); `reference_pointer: "prismql://reference"` still present |
| EC-11-047 | E-QUERY-002 for ColumnType `Json` (operator semantics are implementation-defined) | `valid_operators_for_type` includes at minimum `["=", "!="]`; additional operators listed if implemented |
| EC-11-048 | E-QUERY-003 with unrecognized limit category (future limit added without updating this table) | `how_to_fix: "Simplify or shorten the query."` (catch-all) |
| EC-11-049 | E-QUERY-037 with `did_you_mean` present | `suggestion` includes both the `did_you_mean` retry hint AND the `prism_describe` discovery pointer |
| EC-11-050 | E-QUERY-037 with `did_you_mean` absent | `suggestion` includes only the `prism_describe` discovery pointer |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `query("SELECT * FROM events WHERE sevrity > 5")` with `severity` as a String column (type mismatch + near-typo) | E-QUERY-002 with `valid_operators_for_type: ["=", "!=", "LIKE", "IN", "NOT IN"]` (String operators); E-QUERY-002, not E-QUERY-038, because the field exists but the operator is wrong | type-error-enriched |
| `query("SELECT * FROM (((((((... deeply nested ...")` | E-QUERY-003 with `how_to_fix: "Flatten nested conditions. Use AND/OR instead of deeply nested parentheses."` | security-limit-how-to-fix |
| `query("SELCT * FROM crowdstrike_alerts")` (typo in SELECT keyword) | E-QUERY-001 with `near_text: "SELCT"` (the offending token), `reference_pointer: "prismql://reference"` | parse-error-enriched |
| `query("SELECT * FROM crowdstrike_alert")` when `crowdstrike_alerts` is registered | E-QUERY-037 with `did_you_mean: "crowdstrike_alerts"` AND `suggestion` containing "Call prism_describe" AND "crowdstrike_alerts" retry hint | table-not-found-suggestion |
| `query("SELECT * FROM crowdstrike_bogustable")` when no close match exists | E-QUERY-037 with `did_you_mean` absent AND `suggestion` containing "Call prism_describe('...')" | table-not-found-no-suggestion |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (VP-TBD) | E-QUERY-001 structured response always contains `near_text` (may be empty string) and `reference_pointer: "prismql://reference"` | unit test |
| (VP-TBD) | E-QUERY-002 structured response always contains `valid_operators_for_type` as a non-null array | unit test |
| (VP-TBD) | E-QUERY-003 structured response always contains `how_to_fix` as a non-empty string | unit test |
| (VP-TBD) | E-QUERY-037 `suggestion` always contains the substring `"prism_describe"` | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 |
| Capability Anchor Justification | CAP-015 ("Ephemeral OCSF Query Engine") per capabilities.md §CAP-015 — this BC specifies enrichments to existing E-QUERY-NNN parse-time and plan-time error responses emitted by the PQL query engine. CAP-015 governs the query engine including its structured error responses ("Violations return structured errors with actionable suggestions"). Pedagogical field enrichments to E-QUERY-001/002/003/037 are improvements to the error actionability within the query engine surface that CAP-015 owns. |
| L2 Invariants | DI-002, DI-004, DI-006, DI-019 |
| ADR | ADR-041 v1.1 §L4 — "Codes getting the full pedagogical treatment (new or upgraded)" table; §Consequences — "The E-QUERY-038 column-not-found error with available_columns + did_you_mean gives the model everything it needs to self-correct" |
| Architecture Module | SS-11 (Query Execution) |
| Priority | P1 |

## Related BCs

- BC-2.11.001 — depends on: all four enriched error codes are error cases of the `query` MCP tool (BC-2.11.001 Error Cases table); story-writer must propagate E-QUERY-001/002/003/037 enrichment details to BC-2.11.001 body (under `bc_array_changes_propagate_to_body_and_acs` policy)
- BC-2.11.016 — composes with: E-QUERY-038 (new code) is part of the same L4 pedagogical suite; together, BCs 016 and 017 constitute the complete L4 error enrichment
- BC-2.10.014 — composes with: E-QUERY-001 `reference_pointer: "prismql://reference"` points to the grammar resource; E-QUERY-037 `suggestion` references `prism_describe` which in turn references `prismql://reference`
- BC-2.11.010 — depends on: `explain_query` calls also go through the plan-time validation pipeline and trigger E-QUERY-001/002/003/037/038; these enrichments apply to explain responses as well

## Architecture Anchors

- `architecture/decisions/ADR-041` §L4 — "Pedagogical E-QUERY-NNN Self-Correction Loop": error-shape contract, per-code upgrade table
- `architecture/decisions/ADR-039` — E-QUERY-037 `did_you_mean` is already implemented per ADR-039; this BC reaffirms and adds the `prism_describe` suggestion text

## Story Anchor

S-DEMO-PRISMQL-ONBOARDING-001-B — the BC's behavior (pedagogical enrichments to E-QUERY-001/002/003/037) is implemented by this story per its `anchor_bcs: [BC-2.11.017, ...]` frontmatter (POL-4/POL-5).

## VP Anchors

VP assignments TBD — assigned after VP authoring pass.

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.14 | DEFECT-PQL-FNCALL-LHS-001-FB19-F-PQLFN-P24-OBS-003 | 2026-07-14 | product-owner | **F-PQLFN-P24-OBS-003 POL-23 companion — §Postconditions E-QUERY-001 "current state" and Implementation note corrected.** Three stale normative claims referenced `"Query parse error at position {pos}: {message}"` (never a live Display form) and `PrismError::ParseError` (wrong variant name). Actual emitter is `PrismError::QueryParseFailed` in prism-core/src/error.rs with Display `"E-QUERY-001: query parse error at offset {offset}: {detail}"`. **(1)** §Postconditions `**Current state:**` (line 43): stale form → `"E-QUERY-001: query parse error at offset {offset}: {detail}"` with correct emitter cite `PrismError::QueryParseFailed`. **(2)** §Postconditions `near_text` sourcing note (line 49): `at the position {pos}` → `at the byte offset {offset}`. **(3)** §Postconditions `**Implementation note:**` (line 53): `PrismError::ParseError (or equivalent)` → `PrismError::QueryParseFailed`; `"Query parse error at position {pos}: {message}"` → `"E-QUERY-001: query parse error at offset {offset}: {detail}"`; note clarified: `near_text`/`reference_pointer` are extracted at MCP error-map time from the existing `query` field (AC-003), not added as new Rust struct fields. Semantic content of BC UNCHANGED — additive MCP-layer fields, no Display change. Companion: error-taxonomy.md v2.47→v2.48. |
| 1.13 | FIX-IEQ-ERRPATH-001-ADV-PR-P5 | 2026-07-09 | product-owner | **ADV-PR-P5 companion to BC-2.11.016 v1.25 — SIBLING-GATE CONSISTENCY cross-reference pin update (pin-only; no content change).** BC-2.11.016 v1.25 closes ADV-PR-P5-MED-001/002 and fixes a proactive-class inaccuracy in §Implementation table row 4 (`OrderBy::expr` → `OrderExpr::expr`). These corrections do not affect the SIBLING-GATE CONSISTENCY invariant or the `check_pipe_stage_columns` binding-context walk semantics. **Amendment (pin-only):** heading pin `(BC-2.11.016 v1.24):` → `(BC-2.11.016 v1.25):`; internal `co-defined in BC-2.11.016 v1.24` → `co-defined in BC-2.11.016 v1.25`. Origin anchors `(BC-2.11.016 v1.18)` and `(BC-2.11.016 v1.19)` UNCHANGED. Frontmatter v1.12→v1.13; modified: 2026-07-09. |
| 1.12 | FIX-IEQ-ERRPATH-001-ADV-PR-P4-PROACTIVE | 2026-07-09 | product-owner | **ADV-PR-P4 proactive companion to BC-2.11.016 v1.24 — SIBLING-GATE CONSISTENCY cross-reference pin update (pin-only; no content change).** BC-2.11.016 v1.24 corrects three AST type-name inaccuracies in §Implementation location table: position 7 `And`/`Or`/`Not` → `Predicate::Logical`/`Predicate::Not`; position 8 `PipeStage::Where(FilterExpr)` → `PipeStage::Where(Predicate)`; position 10 `SortEntry` → `SortExpr`. These corrections do not affect the SIBLING-GATE CONSISTENCY invariant or the `check_pipe_stage_columns` binding-context walk semantics. **Amendment (pin-only):** heading pin `(BC-2.11.016 v1.23):` → `(BC-2.11.016 v1.24):`; internal `co-defined in BC-2.11.016 v1.23` → `co-defined in BC-2.11.016 v1.24`. Origin anchors `(BC-2.11.016 v1.18)` and `(BC-2.11.016 v1.19)` UNCHANGED. Frontmatter v1.11→v1.12; modified: 2026-07-09. |
| 1.11 | FIX-IEQ-ERRPATH-001-ADV-PR-P3-LOW-001 | 2026-07-09 | product-owner | **ADV-PR-P3-LOW-001 POL-25 companion to BC-2.11.016 v1.23 — SIBLING-GATE CONSISTENCY cross-reference pin update (pin-only; no content change).** BC-2.11.016 v1.23 corrects §Implementation location table function names (POL-22 Phase C): positions 1/3/4/5 → `extract_field_paths_with_bareness`; positions 2/6 → `extract_predicate_columns_with_bareness`; positions 10–14 → `extract_column_name_from_field_path`. This does not affect the SIBLING-GATE CONSISTENCY invariant or the `check_pipe_stage_columns` binding-context walk semantics. **Amendment (pin-only):** §Invariants SIBLING-GATE CONSISTENCY cross-reference heading pin updated: `(BC-2.11.016 v1.22):` → `(BC-2.11.016 v1.23):`; internal `co-defined in BC-2.11.016 v1.22` → `co-defined in BC-2.11.016 v1.23`. Origin anchors `(BC-2.11.016 v1.18)` and `(BC-2.11.016 v1.19)` within the same bullet UNCHANGED. No postcondition content changed. Frontmatter v1.10→v1.11; modified: 2026-07-09. |
| 1.10 | FIX-IEQ-ERRPATH-001-ADV-PR-P1-OBS-001 | 2026-07-09 | product-owner | **ADV-PR-P1-OBS-001 POL-25 companion to BC-2.11.016 v1.22 — SIBLING-GATE CONSISTENCY cross-reference pin update (pin-only; no content change).** BC-2.11.016 v1.22 adds `### Injection-safety of \`column\` (MCP-facing payload)` — a postcondition note documenting `sanitize_for_log` control-char sanitization applied to the `ColumnNotFoundDetails.column` MCP-facing field (CWE-116/CWE-117; ADV-PR-P1-OBS-001). This is a payload postcondition concern; it does not affect the `check_pipe_stage_columns` binding-context walk that the SIBLING-GATE CONSISTENCY invariant (this BC) covers. No new SIBLING-GATE CONSISTENCY extension note is needed. **Amendment (pin-only):** §Invariants SIBLING-GATE CONSISTENCY cross-reference heading pin updated: `(BC-2.11.016 v1.21):` → `(BC-2.11.016 v1.22):`; internal `co-defined in BC-2.11.016 v1.21` → `co-defined in BC-2.11.016 v1.22`. The STAR-WITH-JOIN SUSPENSION RULE extension note `(BC-2.11.016 v1.18)` and STAGE-JOIN SUSPENSION RULE extension note `(BC-2.11.016 v1.19)` within the same bullet are UNCHANGED — they accurately cite the versions that introduced those rules. No postcondition content changed. Frontmatter v1.9→v1.10; modified: 2026-07-09. |
| 1.9 | FIX-IEQ-ERRPATH-001-ADV-FIX-P16-MED-001 | 2026-07-09 | product-owner | **ADV-FIX-P16-MED-001 POL-25 companion to BC-2.11.016 v1.21 — SIBLING-GATE CONSISTENCY cross-reference pin update (pin-only; no content change).** BC-2.11.016 v1.21 adds PER-REFERENCE SCOPING clarification to the HEAD-JOIN SUSPENSION RULE: suspension applies per individual column reference, not per column name; a qualified reference to a name that also appears bare elsewhere in positions 1–6 is NOT suspended and retains full E-QUERY-038 checking. The HEAD-JOIN SUSPENSION RULE (and its PER-REFERENCE SCOPING clarification) is a positions-1–6 rule that applies OUTSIDE the `check_pipe_stage_columns` binding-context walk — it does not affect the pipe-stage walk that the SIBLING-GATE CONSISTENCY invariant (this BC) covers. Therefore, no new SIBLING-GATE CONSISTENCY extension note is needed. **Amendment (pin-only):** §Invariants SIBLING-GATE CONSISTENCY cross-reference heading pin updated: `(BC-2.11.016 v1.20):` → `(BC-2.11.016 v1.21):`; internal `co-defined in BC-2.11.016 v1.20` → `co-defined in BC-2.11.016 v1.21`. The STAR-WITH-JOIN SUSPENSION RULE extension note `(BC-2.11.016 v1.18)` and STAGE-JOIN SUSPENSION RULE extension note `(BC-2.11.016 v1.19)` within the same bullet are UNCHANGED — they accurately cite the versions that introduced those rules. No postcondition content changed. Frontmatter v1.8→v1.9; modified: 2026-07-09. |
| 1.8 | FIX-IEQ-ERRPATH-001-ADV-FIX-P15-MED-001 | 2026-07-09 | product-owner | **ADV-FIX-P15-MED-001 POL-25 companion to BC-2.11.016 v1.20 — SIBLING-GATE CONSISTENCY cross-reference pin update (pin-only; no content change).** BC-2.11.016 v1.20 introduces the HEAD-JOIN SUSPENSION RULE: when the head query's JOIN list is non-empty AND a bare unqualified column reference at positions 1–6 is absent from `schema_columns(table, OrgId)`, E-QUERY-038 MUST NOT fire (fail-open). The HEAD-JOIN SUSPENSION RULE is a position-1–6 rule that applies OUTSIDE the `check_pipe_stage_columns` binding-context walk — it does not affect the pipe-stage walk that the SIBLING-GATE CONSISTENCY invariant (this BC) covers. Therefore, no new SIBLING-GATE CONSISTENCY extension note is needed. **Amendment (pin-only):** §Invariants SIBLING-GATE CONSISTENCY cross-reference heading pin updated: `(BC-2.11.016 v1.19):` → `(BC-2.11.016 v1.20):`; internal `co-defined in BC-2.11.016 v1.19` → `co-defined in BC-2.11.016 v1.20`. The STAGE-JOIN SUSPENSION RULE extension note `(BC-2.11.016 v1.19)` within the same bullet is UNCHANGED — it accurately cites the version that introduced the STAGE-JOIN rule (v1.19). No postcondition content changed. Frontmatter v1.7→v1.8; modified: 2026-07-09. |
| 1.7 | FIX-IEQ-ERRPATH-001-ADV-FIX-P14-OBS-001-002-003 | 2026-07-09 | product-owner | **ADV-FIX-P14-OBS-001-003 POL-25 companion to BC-2.11.016 v1.19 — SIBLING-GATE CONSISTENCY invariant updated for STAGE-JOIN SUSPENSION RULE.** BC-2.11.016 v1.19 introduces the STAGE-JOIN SUSPENSION RULE: when the pipe-stage walk encounters `PipeStage::Join`, `suspended := true` engages. The SIBLING-GATE CONSISTENCY invariant (this BC) owns the E-QUERY-002 fail-open obligation for ALL suspension triggers. **Amendment:** §Invariants SIBLING-GATE CONSISTENCY cross-reference: (1) BC pin updated v1.18 → v1.19; (2) STAGE-JOIN SUSPENSION RULE extension note added — when suspension is set by the STAGE-JOIN SUSPENSION RULE during the pipe-stage walk, E-QUERY-002 is also suspended for all remaining stages (parallel to the STAR-WITH-JOIN extension in v1.6; same suspension-propagation principle applied to the stage-level join trigger). No postcondition content changed. Frontmatter v1.6→v1.7; modified: 2026-07-09. |
| 1.6 | FIX-IEQ-ERRPATH-001-ADV-FIX-P12-OBS-002 | 2026-07-09 | product-owner | **ADV-FIX-P12-OBS-002 POL-25 companion to BC-2.11.016 v1.18 — SIBLING-GATE CONSISTENCY invariant updated for STAR-WITH-JOIN SUSPENSION RULE.** BC-2.11.016 v1.18 introduces the STAR-WITH-JOIN SUSPENSION RULE: in branches (a) and (c), when the SqlPipe head has a non-empty JOIN list and at least one `Star`/`TableStar`, the initial binding context is SUSPENDED (`suspended := true`). The SIBLING-GATE CONSISTENCY invariant (this BC) owns the E-QUERY-002 fail-open obligation. **Amendment:** §Invariants SIBLING-GATE CONSISTENCY cross-reference: (1) BC pin updated v1.17 → v1.18; (2) STAR-WITH-JOIN SUSPENSION RULE extension note added — when initial suspension is set via this rule at the start of the pipe-stage walk, E-QUERY-002 is also suspended for ALL subsequent pipe stages (not a new exception; application of existing suspension-propagation to a new suspension trigger). No postcondition content changed (the `valid_operators_for_type` field requirement for RAW-bound columns is unaffected). Frontmatter v1.5→v1.6; modified: 2026-07-09. |
| 1.5 | FIX-IEQ-ERRPATH-001-ADV-FIX-P10-OBS-001 | 2026-07-09 | product-owner | **ADV-FIX-P10-OBS-001 POL-25 companion to BC-2.11.016 v1.17 — SIBLING-GATE CONSISTENCY invariant DERIVED provenance examples updated.** BC-2.11.016 v1.17 introduces the LAST-SEGMENT OUTPUT-NAME RULE: un-aliased bare-`Field` SELECT items in branches (b)/(c) whose qualifier matches a JOIN alias seed their last segment with DERIVED provenance. The SIBLING-GATE CONSISTENCY invariant (this BC) owns the E-QUERY-002 fail-open obligation for all DERIVED-provenance names. **Amendment:** §Invariants SIBLING-GATE CONSISTENCY cross-reference: DERIVED provenance examples expanded from "(stats output alias, enrich output column, SqlPipe head alias, or any name that shadows a raw-schema column under a derived context)" to also include "JOIN-qualified bare-`Field` SELECT item seeded via LAST-SEGMENT OUTPUT-NAME RULE (BC-2.11.016 v1.17)". BC version pin updated from v1.15 → v1.17 in the invariant body. No postcondition content changed (the `valid_operators_for_type` field requirement for RAW-bound columns is unaffected). Frontmatter v1.4→v1.5; modified: 2026-07-09. |
| 1.4 | FIX-IEQ-ERRPATH-001-ADV-FIX-P7-MED-001-OBS-001-OBS-002 | 2026-07-09 | product-owner | **ADV-FIX-P7-MED-001 POL-25 cross-reference: SIBLING-GATE CONSISTENCY invariant added.** BC-2.11.016 v1.15 introduces the SIBLING-GATE CONSISTENCY clause establishing that the E-QUERY-002 type-compat gate MUST honor the same per-name provenance (RAW vs DERIVED) as the E-QUERY-038 existence gate in `check_pipe_stage_columns`. This BC owns E-QUERY-002 (§E-QUERY-002 postconditions). **New invariant:** SIBLING-GATE CONSISTENCY cross-reference — E-QUERY-002 gate MUST fail open for names with DERIVED provenance (stats alias, enrich output, SqlPipe head alias, shadow of raw column); applying raw-schema type to a DERIVED name fires a false E-QUERY-002 (FP-001 violation per BC-2.11.016). Cross-reference to BC-2.11.016 v1.15 §Preconditions DERIVED-COLUMN BINDING RULE SIBLING-GATE CONSISTENCY clause. No postcondition content changed (the `valid_operators_for_type` field requirement for RAW-bound columns is unaffected). Frontmatter v1.3→v1.4; modified: 2026-07-09. |
| 1.3 | F-001B-SCFRESH-MED-001-story-anchor-fix | 2026-06-22 | product-owner | F-001B-SCFRESH-MED-001 closure (POL-4 story-anchor mis-anchoring): `## Story Anchor` corrected from placeholder `S-5.04 (or dedicated ADR-041 teaching story — to be assigned by story-writer)` to the actual implementing story `S-DEMO-PRISMQL-ONBOARDING-001-B`. Exhaustive BC metadata audit: all other surfaces clean. |
| 1.2 | F-001B-FRESH2-MED-001-pol20-normalization | 2026-06-22 | product-owner | POL-20 normalization: `introduced: ADR-041-teaching-burst-2026-06-19` → `introduced: 2026-06-19` (opaque burst-ID format prohibited by POL-20 anchored-regex; ISO date extracted). No body semantics changed. |
| 1.1 | S-DEMO-PRISMQL-ONBOARDING-001-B | 2026-06-22 | product-owner | **F-001B-FRESH-MED-001 closure** — propagate shipped implementation reality to BC body. (1) §E-QUERY-002 "Current state": replaced stale Display `"Type error: field '{field}' is {actual_type}, cannot use {operator}"` (never a live format) with the ratified `PrismError::QueryTypeMismatch` variant and its shipped Display `"E-QUERY-002: type mismatch — column '{column}' in table '{table}' has type '{actual_type:?}' which does not support operator '{operator}'"`. Added cross-reference to error-taxonomy.md v1.94 §E-QUERY-002 dual-Display collision row. (2) §"No new PrismError variants required": corrected — `QueryTypeMismatch { column, table, actual_type, operator }` WAS added (CORRECTION-2 adjudication; +0 non_exhaustive gate). The `valid_operators_for_type` additive field requirement is unchanged. H1 title and all other postconditions/ACs are preserved verbatim. |
| 1.0 | ADR-041-teaching-burst-2026-06-19 | 2026-06-19 | product-owner | Initial draft — ADR-041 L4 pedagogical enrichments to E-QUERY-001/002/003/037 |
