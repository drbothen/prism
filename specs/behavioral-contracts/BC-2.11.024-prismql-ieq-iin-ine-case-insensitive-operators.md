---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-07-06T00:00:00Z
phase: 1a
origin: greenfield
subsystem: "SS-11"
capability: "CAP-015"
lifecycle_status: draft
introduced: 2026-07-06
modified: "2026-07-07"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/domain-spec/capabilities.md"
  - ".factory/specs/domain-spec/invariants.md"
  - ".factory/specs/architecture/decisions/ADR-047-prismql-case-sensitivity-policy-ieq-iin-and-adapter-boundary-normalization.md"
input-hash: "TBD"
traces_to: ["CAP-015"]
extracted_from: null
---

# BC-2.11.024: PrismQL Case-Insensitive Equality and Membership Operators (IEQ / IIN / INE)

## Description

PrismQL extends its existing `I`-prefix case-insensitive operator family (`ICONTAINS`, `ISTARTSWITH`, `IENDSWITH`) with three new operators for equality and membership predicates: `IEQ` (case-insensitive equality), `IIN` (case-insensitive membership), and `INE` (case-insensitive inequality). Each operator is lowered to a `lower(field) OP lower('val')` DataFusion SQL expression, reusing the idiom already established for `ICONTAINS`. The default `=`, `!=`, and `IN` operators remain case-sensitive (unchanged per ADR-047 D.1). `IEQ`/`IIN`/`INE` are opt-in operators providing an ergonomic safety net for fields where agent-generated casing may differ from stored data — the primary use case is OCSF enum-label fields such as `severity` and `status`.

## Preconditions

- The query string has been auto-detected as filter mode (BC-2.11.002) or contains a `| where` stage in pipe mode (BC-2.11.004). `IEQ`/`IIN`/`INE` are available in both these modes because filter predicates use a shared grammar per the ADR-046 D7 invariant (BC-2.11.023). **These operators are NOT available in raw SQL mode (BC-2.11.003)** — a SQL-mode query containing `IEQ`, `IIN`, or `INE` MUST be rejected at parse time; see the Mode-Boundary Enforcement invariant.
- The field referenced by `IEQ`/`IIN`/`INE` is a string-type column in the DataFusion execution schema. Applying `lower()` to a non-string column (e.g., `severity_id IEQ 'high'` where `severity_id` is integer) results in `E-QUERY-002 (QueryTypeMismatch)`.
- The query string has passed the 64KB length check.

## Postconditions

### New operators and their syntax

| Operator | Syntax | Semantics |
|----------|--------|-----------|
| `IEQ` | `field IEQ 'value'` | Case-insensitive equality: matches when `lower(field) = lower('value')` |
| `IIN` | `field IIN ('val1', 'val2', ...)` | Case-insensitive membership: matches when `lower(field) IN (lower('val1'), lower('val2'), ...)` |
| `INE` | `field INE 'value'` | Case-insensitive inequality: matches when `lower(field) != lower('value')` |

- Operators are parsed **case-insensitively** in the Chumsky grammar via the `kw(...)` combinator: `ieq`, `IEQ`, `Ieq` all parse identically.
- Each operator adds a `case_insensitive: bool` flag (set to `true`) on the corresponding AST variant: `Predicate::Compare` (for `IEQ`/`INE`) and `Predicate::In` (for `IIN`).

### DataFusion SQL lowering

`predicate_to_datafusion_sql` in `pipe_sql_emitter.rs` lowers each case-insensitive predicate using the `lower()` idiom already established for `ICONTAINS`/`ISTARTSWITH`/`IENDSWITH`:

| PrismQL Predicate | DataFusion SQL Lowering |
|-------------------|------------------------|
| `severity IEQ 'high'` | `lower(severity) = lower('high')` |
| `severity IIN ('high', 'critical')` | `lower(severity) IN (lower('high'), lower('critical'))` |
| `severity INE 'low'` | `lower(severity) != lower('low')` |

`lower()` is applied to **both sides**: field reference and each literal value. The `ILIKE` function is NOT used — it is a pattern-matching operator whose `%`/`_` metacharacters are semantically wrong for exact equality (see ADR-047 §Alternatives Alt-2).

### Relationship to case-sensitive operators

The default operators `=`, `!=`, and `IN` are **unchanged** — they retain case-sensitive exact-match semantics consistent with DataFusion's default and SIEM execution-language consensus (KQL/EQL). `IEQ`/`IIN`/`INE` are additive, opt-in operators only. A query using `severity = 'High'` behaves identically to before this BC was authored.

### I-prefix family membership

`IEQ`/`IIN`/`INE` complete the case-insensitive operator family alongside `ICONTAINS`, `ISTARTSWITH`, `IENDSWITH`. All six I-prefix operators share the `lower(...)` lowering idiom and the semantic guarantee: they match regardless of the casing difference between stored data and query literals.

### normalized_pql round-trip (ADR-047 D.4 / BC-2.11.018)

`IEQ`, `IIN`, and `INE` predicates are reflected in the `normalized_pql` field (BC-2.11.018) on successful query responses. The Chumsky normalizer emits operator keywords in uppercase canonical form (e.g., `severity ieq 'high'` normalizes to `severity IEQ 'high'`). The round-trip guarantee from BC-2.11.018 applies: the value in `normalized_pql` parses back to the same AST.

## Invariants

- `IEQ` is a strict superset of `=`: for any two values with identical casing, `IEQ` matches if and only if `=` would match. Precision differences appear only when casing differs between field value and query literal.
- The `lower(field) OP lower('val')` lowering sacrifices index sargability (the full column is lowercased before comparison). This is acceptable given Prism's MemTable query size is bounded at 10,000 rows (BC-2.11.006). Analysts using `IEQ` on large-scale cold-query engines outside Prism cannot assume the same performance profile.
- `IEQ`/`IIN`/`INE` are valid in filter mode and in pipe-mode `| where` stages (shared grammar invariant, BC-2.11.023).
- `IIN` requires at least one value in the membership list. An empty `IIN ()` list is a parse error (`E-QUERY-001`).
- **Mode-Boundary Enforcement — scope-closed to filter and pipe modes:** `IEQ`/`IIN`/`INE` are PrismQL-level extensions that have no equivalent in standard SQL. **ALL** raw-SQL mode statements — `SELECT`, `DELETE`, `UPDATE`, and `INSERT...SELECT` — that contain `IEQ`, `IIN`, or `INE` MUST be rejected at **parse time** before reaching DataFusion plan or execution stages. The scope covers ALL raw-SQL statements, not only SELECT, because `build_predicate_parser` (which parses WHERE-clause predicates, including IEQ/IIN/INE) is shared by the DML sub-parsers (`build_delete_parser`, `build_update_parser`, `build_insert_parser`). Without this gate, `DELETE FROM foo WHERE severity IEQ 'high'` reaches DataFusion as an unknown identifier and produces the same opaque internal planning failure (`E-QUERY-034: query execution error: {detail}` with `detail` redacted to `<redacted; see server logs>`) as a SELECT query — an analyst-facing trap providing zero corrective guidance, equally harmful on the DML path. **Implementation obligation:** the CI-operator detector function MUST be invoked by both (a) the SQL-mode SELECT parse path (BC-2.11.003) and (b) `parse_sql_dml_with_limits` before dispatching to the DML sub-parsers — a code comment near `parse_sql_dml_with_limits` MUST acknowledge this guard. The parse-time rejection emits `E-QUERY-001` with the structured message: `"E-QUERY-001: parse error near '{operator}': case-insensitive operators (IEQ/IIN/INE) are not supported in SQL mode. Use filter mode (e.g., severity IEQ 'high') or a pipe | where stage (e.g., FROM crowdstrike_detections | where severity IEQ 'high') instead."` where `{operator}` is the specific keyword encountered (IEQ, IIN, or INE, normalized to uppercase in the message). This is a **scope-boundary closure** — it eliminates the analyst-facing trap across ALL raw-SQL statement types. It is **NOT** an implementation of SQL-mode case-insensitive operators. Full SQL-mode case-insensitive support, if ever desired, is a separate future feature requiring its own ADR and BC.
- **No `EXPECTED` non-exhaustive gate count change**: `case_insensitive: bool` is a new field added to existing `#[non_exhaustive]` structs (`Predicate::Compare`, `Predicate::In`). The non-exhaustive compile-fail gate counts annotated *types*, not field additions within existing types. Verify at story implementation time per ADR-047 §Consequences.

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-QUERY-001` | `IEQ`/`INE` with a non-string literal on the RHS (e.g., `severity IEQ 42`) | Parse error: "IEQ/INE require a string literal as the right-hand side value" |
| `E-QUERY-001` | `IIN` with an empty membership list: `severity IIN ()` | Parse error: "IIN requires at least one value in the membership list" |
| `E-QUERY-002` | `IEQ`/`IIN`/`INE` applied to a non-string column (e.g., `severity_id IEQ 'high'` where `severity_id` is an integer column) | `QueryTypeMismatch`: `lower()` is not applicable to non-string types; error includes field type info and suggests using the corresponding string column |
| `E-QUERY-001` | Unknown field name used with `IEQ`/`IIN`/`INE` | Error with `similar_fields` suggestions (same behavior as `=`) |
| `E-QUERY-001` | `IEQ`, `IIN`, or `INE` used in any raw-SQL mode statement (BC-2.11.003) — the operator keyword appears in a `SELECT … FROM …` query, a `DELETE FROM … WHERE …` statement, an `UPDATE … SET … WHERE …` statement, or an `INSERT INTO … SELECT … WHERE …` statement; not in filter mode or a `| where` stage | **Parse-time rejection** (before DataFusion planning): `"E-QUERY-001: parse error near '{operator}': case-insensitive operators (IEQ/IIN/INE) are not supported in SQL mode. Use filter mode (e.g., severity IEQ 'high') or a pipe | where stage (e.g., FROM crowdstrike_detections | where severity IEQ 'high') instead."` MCP mapping: `-32602 INVALID_PARAMS` (caller-resolvable). This error replaces the prior opaque `E-QUERY-034` that the operator produced when reaching DataFusion as an unknown identifier. This is scope-boundary enforcement — applies to ALL raw-SQL statement types. No SQL-mode case-insensitive support is added. |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-11-024-001 | `severity IEQ 'HIGH'` where stored value is `'High'` (OCSF canonical Title-case) | Match — `lower('HIGH') = lower('High')` → both `'high'` |
| EC-11-024-002 | `severity IEQ 'high'` where stored value is `'High'` | Match — `lower('high') = lower('High')` → both `'high'` |
| EC-11-024-003 | `status IIN ('open', 'NEW')` where stored values include `'Open'` and `'New'` | Both match — `lower('open')` = `lower('Open')`, `lower('NEW')` = `lower('New')` |
| EC-11-024-004 | `hostname IEQ 'server-01.corp'` (non-enum free-form string field) | Valid — `IEQ` works on any string column, not only OCSF enum fields |
| EC-11-024-005 | `severity = 'High'` (case-sensitive) AND `severity IEQ 'high'` in the same query | Both valid in a single query; `=` requires exact `'High'`; `IEQ` matches any casing equivalent |
| EC-11-024-006 | Operator spelled lowercase: `severity ieq 'high'` | Valid — `kw()` combinator parses keywords case-insensitively; produces same AST as `severity IEQ 'high'` |
| EC-11-024-007 | `IIN` with a single value: `severity IIN ('high')` | Valid — single-element case-insensitive membership check; equivalent to `severity IEQ 'high'` |
| EC-11-024-008 | `IEQ` on a column whose value is `null` | No match — `lower(null)` evaluates to `null`; null comparisons require `IS NULL`/`IS NOT NULL` |
| EC-11-024-009 | `severity IEQ ''` (empty string literal) | Valid parse; matches records where `lower(severity) = ''` — typically zero rows for well-formed OCSF data |
| EC-11-024-010 | `SELECT severity, count(*) FROM crowdstrike_detections WHERE severity IEQ 'high' GROUP BY severity` — raw SQL SELECT statement containing `IEQ` in a SQL `WHERE` clause | **Parse-time `E-QUERY-001` rejection** with message: `"E-QUERY-001: parse error near 'IEQ': case-insensitive operators (IEQ/IIN/INE) are not supported in SQL mode. Use filter mode (e.g., severity IEQ 'high') or a pipe | where stage (e.g., FROM crowdstrike_detections | where severity IEQ 'high') instead."` — NOT an opaque `E-QUERY-034`. The same rejection applies to `IIN` and `INE` appearing in SQL context. |
| EC-11-024-011 | `DELETE FROM crowdstrike_detections WHERE severity IEQ 'high'` — raw SQL DML statement containing `IEQ` in a `WHERE` clause; invoked via `parse_sql_dml_with_limits` | **Parse-time `E-QUERY-001` rejection** with message: `"E-QUERY-001: parse error near 'IEQ': case-insensitive operators (IEQ/IIN/INE) are not supported in SQL mode. Use filter mode (e.g., severity IEQ 'high') or a pipe | where stage (e.g., FROM crowdstrike_detections | where severity IEQ 'high') instead."` — NOT an opaque `E-QUERY-034`. `parse_sql_dml_with_limits` MUST invoke the same CI-operator detector as the SELECT path before dispatching to DML sub-parsers. The same rejection applies to UPDATE and INSERT...SELECT containing IEQ, IIN, or INE. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `severity IEQ 'high'` against data row `{severity: 'High'}` | Row returned | happy-path |
| `severity IEQ 'HIGH'` against data row `{severity: 'High'}` | Row returned — case-insensitive | happy-path |
| `severity IEQ 'high'` against data row `{severity: 'HIGH'}` | Row returned — `lower('HIGH')` = `lower('high')` | happy-path |
| `status IIN ('open', 'in_progress')` against `{status: 'Open'}` | Row returned | happy-path |
| `severity INE 'informational'` against `{severity: 'Informational'}` | Row NOT returned — `lower('Informational') != lower('informational')` is false | happy-path |
| `severity = 'High'` against `{severity: 'HIGH'}` | Row NOT returned — `=` is case-sensitive, unchanged behavior | regression-no-change |
| `severity IEQ 42` | `Err(E-QUERY-001)` — IEQ requires string literal | error |
| `severity IIN ()` | `Err(E-QUERY-001)` — IIN requires at least one value | error |
| `severity_id IEQ 'high'` where `severity_id` is integer column | `Err(E-QUERY-002)` — `lower()` not applicable to integer type | error |
| `severity ieq 'critical'` (lowercase operator) | Parsed identically to `severity IEQ 'critical'` — `kw()` is case-insensitive | happy-path |
| `FROM crowdstrike_detections \| where severity IEQ 'high' \| head 10` | Returns rows matching `lower(severity)='high'`; `normalized_pql` contains `severity IEQ 'high'` (uppercase canonical) | pipe-mode + normalized_pql |
| `SELECT * FROM crowdstrike_detections WHERE severity IEQ 'high'` (SQL mode) | `Err(E-QUERY-001)` — parse-time rejection: `"E-QUERY-001: parse error near 'IEQ': case-insensitive operators (IEQ/IIN/INE) are not supported in SQL mode. Use filter mode (e.g., severity IEQ 'high') or a pipe | where stage (e.g., FROM crowdstrike_detections | where severity IEQ 'high') instead."` MCP: `-32602 INVALID_PARAMS`. NOT an opaque `E-QUERY-034`. | error (mode-boundary) |
| `SELECT * FROM crowdstrike_detections WHERE status IIN ('open', 'new')` (SQL mode with IIN) | `Err(E-QUERY-001)` — parse-time rejection with `{operator}` = `IIN` in the message (same structured mode-boundary error pattern) | error (mode-boundary) |
| `DELETE FROM crowdstrike_detections WHERE severity IEQ 'high'` (SQL DML mode — `parse_sql_dml_with_limits` path) | `Err(E-QUERY-001)` — parse-time rejection with `{operator}` = `IEQ` in the message (same structured mode-boundary error pattern as SELECT; NOT `E-QUERY-034`). DML path MUST invoke the CI-operator detector. | error (mode-boundary DML) |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-021 | PrismQL parser: never panics on arbitrary input (including IEQ/IIN/INE predicates) | fuzz |
| (VP-TBD) | `IEQ(field, 'VAL')` and `IEQ(field, 'val')` produce identical DataFusion plans for arbitrary mixed-case string literals (ADR-047 §Verification Obligation 1) | proptest |
| (VP-TBD) | `normalized_pql` for queries containing `IEQ`/`IIN`/`INE` parses back to the same AST (round-trip invariant per ADR-047 §Verification Obligation 2 + BC-2.11.018) | proptest |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 |
| Capability Anchor Justification | CAP-015 ("Ephemeral OCSF Query Engine") per capabilities.md §CAP-015 — this BC adds three case-insensitive predicate operators (`IEQ`/`IIN`/`INE`) to the PrismQL grammar. CAP-015 is the authoritative capability for the PrismQL query engine and its full grammar: it defines the three query modes (filter, SQL, pipe), the supported operators in each mode, and the DataFusion execution layer. Extending the operator set is a direct extension of CAP-015's grammar responsibility. |
| L2 Invariants | DI-019 |
| ADR | ADR-047 §D.1 (case-sensitive default unchanged), §D.2 (IEQ/IIN/INE opt-in operators): ACCEPTED per human sign-off D-1398, 2026-06-27 (OD-2: case-sensitive default confirmed; OD-3: IEQ/IIN/INE spelling confirmed) |
| Architecture Module | SS-11 (Query Execution) — `prism-query` Chumsky grammar + `pipe_sql_emitter.rs` DataFusion lowering |
| Priority | P1 |

## Related BCs

- BC-2.11.002 (composes with): amended (v1.5) to list `IEQ`/`IIN`/`INE` in the filter-mode supported operator table; filter-mode and pipe-mode share predicate grammar per ADR-046 D7
- BC-2.11.004 (composes with): amended (v1.13) to note `IEQ`/`IIN`/`INE` available in `| where` stages via shared filter grammar
- BC-2.11.018 (composes with): amended (v1.3) — EC-11-057 added; `normalized_pql` reflects `IEQ`/`IIN` predicates in uppercase canonical form; round-trip guarantee applies
- BC-2.02.013 (composes with): adapter-boundary normalization (companion contract) ensures canonical casing in stored data; `IEQ`/`IIN`/`INE` are the ergonomic safety net for any remaining case mismatches and for free-form non-enum fields

## Architecture Anchors

- `architecture/decisions/ADR-047` §D.1 — Case-Sensitive Default (`=`/`!=`/`IN` unchanged)
- `architecture/decisions/ADR-047` §D.2 — Opt-In Case-Insensitive Operators (IEQ/IIN/INE syntax, `lower()` lowering, `I`-prefix rationale)
- `architecture/decisions/ADR-047` §Alternatives Alt-2 (ILIKE rejected for exact equality), Alt-4 (`=~` rejected — grammar collision with regex), Alt-5 (Sigma `|ci` rejected)
- `architecture/decisions/ADR-047` §Consequences — sargability tradeoff; backward compatibility guarantee

## Story Anchor

S-PRISMQL-CASE-INSENSITIVE-001 — implements IEQ/IIN/INE Chumsky grammar rules, AST `case_insensitive` flag, and `predicate_to_datafusion_sql` lowering.

## VP Anchors

VP-021 (existing). VP for IEQ/IIN proptest plan-equality and round-trip verification to be assigned after VP authoring pass.

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | S-PRISMQL-CASE-INSENSITIVE-001-bc-burst | 2026-07-06 | product-owner | Initial draft. IEQ/IIN/INE case-insensitive operator family: grammar, AST `case_insensitive` flag, DataFusion `lower()` lowering. Resolves ADR-047 OD-2 (case-sensitive default) + OD-3 (IEQ/IIN/INE spelling) per human sign-off D-1398 2026-06-27. |
| 1.2 | S-PRISMQL-CASE-INSENSITIVE-001-adversary-pass-7-dml-scope-extension | 2026-07-07 | product-owner | **LOCAL adversary pass-7 F-LOW-2 closure: DML scope adjudication — EXTEND to ALL raw-SQL statements.** v1.1 Mode-Boundary Enforcement scoped the CI-operator rejection to "queries beginning with SELECT". F-LOW-2 found that `build_predicate_parser` is shared by DML sub-parsers (`build_delete_parser`, `build_update_parser`, `build_insert_parser`), so `DELETE FROM foo WHERE severity IEQ 'high'` also produces the opaque E-QUERY-034 trap via `parse_sql_dml_with_limits`. Adjudication: EXTEND — the trap applies equally to DML; extension is trivially cheap; production-grade default requires closure. Changes: (1) Mode-Boundary Enforcement invariant: scope changed from "queries beginning with `SELECT`" to "ALL raw-SQL mode statements (SELECT, DELETE, UPDATE, INSERT...SELECT)"; added implementation obligation that `parse_sql_dml_with_limits` MUST invoke the same CI-operator detector before dispatching to DML sub-parsers; added code-comment requirement near `parse_sql_dml_with_limits`. (2) Error Cases table: "SELECT … FROM …" example broadened to "any raw-SQL mode statement" with all four SQL forms listed. (3) EC-11-024-011 added: DML mode-boundary edge case (DELETE containing IEQ). (4) Test vector added: DML DELETE with IEQ → E-QUERY-001 (mode-boundary DML category). Implementer directive: extend `detect_ci_operator_in_sql_query` (to be written) to also cover the DML parse path via `parse_sql_dml_with_limits`; add code comment near `parse_sql_dml_with_limits` acknowledging the CI-operator guard. Test-writer directive: add DML RED Gate tests (DELETE/UPDATE containing IEQ → E-QUERY-001 with mode-boundary message; NOT E-QUERY-034). |
| 1.1 | S-PRISMQL-CASE-INSENSITIVE-001-adversary-pass-3-fix | 2026-07-07 | product-owner | Mode-Boundary Enforcement invariant: IEQ/IIN/INE are scope-closed to filter and pipe `\| where` modes. SQL-mode queries using these operators MUST be rejected at parse time with structured `E-QUERY-001` (not the opaque `E-QUERY-034` DataFusion planning failure). Added: precondition clarification, Mode-Boundary Enforcement invariant with verbatim message template, SQL-mode rejection error case, EC-11-024-010, two mode-boundary test vectors. Scope-boundary enforcement only — no SQL-mode case-insensitive support is added (future feature). |
