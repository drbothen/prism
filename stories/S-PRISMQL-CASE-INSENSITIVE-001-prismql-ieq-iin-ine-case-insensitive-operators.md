---
document_type: story
story_id: S-PRISMQL-CASE-INSENSITIVE-001
title: "PrismQL Case-Insensitive Operators (IEQ/IIN/INE) + Adapter-Boundary OCSF Enum-Label Normalization (ADR-047)"
epic_id: EPIC-DEMO
version: "1.48"
updated: "2026-07-09"
status: merged
producer: story-writer
phase: 3
wave: wave-5
priority: P0
points: 10
tdd_mode: strict
# tdd_mode rationale: this story modifies production Rust code in prism-query
# (filter_parser.rs grammar, ast.rs struct fields, pipe_sql_emitter.rs lowering,
# ast.rs round-trip normalizer), prism-bin (PRIMARY OCSF enum-label normalization —
# build_column_array in spec_driven_adapter.rs; architect adjudication F-CRIT-002),
# and prism-ocsf (SECONDARY OCSF enum-label normalization — OcsfNormalizer::
# normalize_with_mappers + OcsfEnumMap in enum_map.rs; the DynamicMessage/protobuf
# path). prism-spec-engine re-added to crates_touched with comment-only scope:
# TD-VSDD-091 anti-volatile-pin sweep only (pass-8 fix-burst 0b2c0983); zero code
# changes in prism-spec-engine; strict TDD does not apply to comment-only files.
# All behavioral changes require Red Gate
# tests as failing todo!() stubs BEFORE production code is modified. Grep-sweep ACs
# (AC-006, AC-007) and VERIFY-only ACs (AC-027) do not have standalone Red Gate stubs
# but do not justify facade mode — production code is modified with new behavioral
# semantics.
target_module: prism-query
subsystems: [SS-11, SS-02, SS-22]
# Subsystem anchor justifications:
#   SS-11 (Query Execution) owns prism-query: filter_parser.rs (IEQ/IIN/INE grammar),
#   ast.rs (case_insensitive flag + round-trip normalizer), pipe_sql_emitter.rs
#   (DataFusion lower() lowering). The grammar+AST+emitter is the primary change surface.
#   SS-02 (OCSF Normalization) owns prism-ocsf: enum_map.rs (canonical caption data)
#   and the normalization pipeline that populates OCSF enum-label string fields before
#   DynamicMessage creation. The adapter-boundary fix is the parallel track that ensures
#   stored data is consistently cased so case-sensitive = works correctly across sensors.
#   SS-22 (Process Lifecycle) owns prism-bin: spec_driven_adapter.rs::build_column_array
#   is the PRIMARY OCSF enum-label normalization insertion point per BC-2.02.013 v1.9
#   (F-CRIT-002 adjudication).
crates_touched: [prism-query, prism-ocsf, prism-mcp, prism-bin, prism-core, prism-spec-engine]
# prism-spec-engine re-added: comment-only TD-VSDD-091 anti-volatile-pin sweep
# (pass-8 fix-burst 0b2c0983: stale 'BC-2.16.002 vN.NN' version pins removed from doc
# comments); zero code changes. Orchestrator adjudicated: legitimate in-scope
# sibling-sweep, kept in diff.
depends_on:
  - S-DEMO-FIDELITY-REMEDIATION-001
  # Dependency anchor: S-DEMO-FIDELITY-REMEDIATION-001 is MERGED on develop@ea714d14.
  # This story builds on the column-availability gate work and sensor TOML spec
  # patterns established there. No blocking runtime dependency, but the TOML-spec
  # adaptation patterns and SAP-2 discipline were established by FIDELITY-REMEDIATION.
  # The develop HEAD is ea714d14 — this story branches from there.
blocks: []
behavioral_contracts:
  - BC-2.11.024
  - BC-2.02.013
  - BC-2.11.002
  - BC-2.11.004
  - BC-2.11.018
  - BC-2.02.002
  - BC-2.02.010
  - BC-2.10.012
# BC array propagation (all 8 BCs cited by ACs in the body below):
#   BC-2.11.024 v1.4 (draft): new — PrismQL IEQ/IIN/INE case-insensitive operators;
#     primary contract for grammar+AST+emitter+round-trip changes. Every parser/emitter AC
#     traces to a BC-2.11.024 postcondition, invariant, or error case.
#   BC-2.02.013 v1.9 (draft): new — adapter-boundary OCSF enum-label canonical-case
#     normalization; PRIMARY insertion point now `build_column_array` in spec_driven_adapter.rs
#     (architect adjudication F-CRIT-002). Every adapter AC traces to a BC-2.02.013
#     postcondition, invariant, or error case.
#   BC-2.11.002 v1.6 (active, amended): filter-mode parsing now includes IEQ/IIN/INE in
#     the supported operator table. AC-001/AC-004/AC-012 exercise the filter-mode path.
#   BC-2.11.004 v1.24 (active, amended): pipe-mode | where stage now supports IEQ/IIN/INE
#     via shared filter grammar per ADR-046 D7. AC-013 exercises the pipe-mode path.
#   BC-2.11.018 v1.3 (active, amended): normalized_pql echo now reflects IEQ/IIN/INE
#     predicates in uppercase canonical form (EC-11-057 added). AC-014/AC-015 exercise this.
#   BC-2.02.002 v1.5 (active, amended): normalization applied BEFORE DynamicMessage creation;
#     postconditions updated. AC-016 exercises this ordering invariant.
#   BC-2.02.010 v1.5 (active, amended): enum_map.rs is the sole canonical casing authority
#     at the adapter boundary. AC-016/AC-017 verify this authority chain.
#   BC-2.10.012 v1.9 (active, amended): prism describe output — pure-parseable example_query
#     invariant + example_note: Option<String> field contract. AC-025 (RG-061/062/063) traces
#     to §example_query (no SQL comment lines; pure PQL) and §example_note (OCSF casing note
#     lives in example_note, not in example_query).
verification_properties: [VP-021, VP-016, VP-022]
# VP-021 (active): PrismQL parser never panics on arbitrary input — fuzz target
#   vp021_parse_fuzz in fuzz/. IEQ/IIN/INE grammar changes must not introduce panics.
#   RG-014 (regression guard) directly targets VP-021 by feeding a combined IEQ expression.
# VP-016 (active): OCSF normalization output is valid protobuf — proptest.
#   Adapter normalization changes must preserve protobuf validity.
# VP-022 (active): OCSF normalizer never panics on arbitrary input — fuzz target.
#   Normalization pipeline changes must not introduce panics.
# VP-TBD (pending authorship): IEQ(field, 'VAL') and IEQ(field, 'val') produce identical
#   DataFusion plans for arbitrary mixed-case string literals (ADR-047 §Verification
#   Obligation 1). Proptest. To be authored by product-owner before formal hardening.
# VP-TBD (pending authorship): normalized_pql for queries containing IEQ/IIN/INE parses
#   back to the same AST (round-trip invariant; ADR-047 §Verification Obligation 2).
#   Proptest. To be authored by product-owner before formal hardening.
assumption_validations: []
risk_mitigations:
  - "IIN must be parsed BEFORE IN in the Chumsky combinator chain (longest-match-first).
     If IN is tried before IIN, the 'IN' prefix consumes 'IIN' and produces a parse error.
     RG-002 (IIN parse test) verifies this ordering: if the combinator order is wrong,
     RG-002 fails with a parse error instead of the expected Predicate::In{case_insensitive:true}.
     Implementer MUST verify the exact combinator ordering in filter_parser.rs before
     declaring IIN grammar work complete. Look at the kw() chain where 'IN' is currently
     defined and insert 'IIN' BEFORE 'IN'."
  - "DTU test vectors asserting UPPER-case severity values (e.g., severity=='HIGH' in
     test fixtures for Armis) will break intentionally after adapter normalization (Track E).
     This is the CORRECT behavior — those tests encoded the pre-normalization wrong behavior.
     The implementer MUST update DTU fixture generators and test assertions to use canonical
     Title-case values after normalization. SAP-2 probe applies: adversary will read
     crates/prism-dtu-*/src/types.rs and generator.rs to verify TOML column type parity
     after any sensor TOML changes made in this story."
  - "Sibling-site sweep obligation (TD-VSDD-060): after adding case_insensitive:bool to
     Predicate::Compare and Predicate::In, the compiler will surface every match arm in
     prism-query that destructures these variants. ALL construction sites (Predicate::Compare{...}
     and Predicate::In{...} struct literals) MUST add case_insensitive: false. The compiler
     will NOT surface construction sites automatically — grep is required. See Task 9."
  - "round-trip normalizer (Track D) renders IEQ/IIN/INE in UPPERCASE canonical form
     (e.g., 'severity ieq 'high'' normalizes to 'severity IEQ 'high''). The kw() combinator
     accepts lowercase but the normalizer must emit uppercase. If the normalizer simply
     passes through the parsed operator string, it will emit lowercase if the query was
     written lowercase. The normalizer must use canonical uppercase operator names."
traces_to: [ADR-047]
red_gate_tests: 83
estimated_days: "3"
---

# S-PRISMQL-CASE-INSENSITIVE-001: PrismQL Case-Insensitive Operators (IEQ/IIN/INE) + Adapter-Boundary OCSF Enum-Label Normalization

Implements ADR-047 (ACCEPTED v1.1, 2026-07-06) decisions D.1–D.4:

- **D.1** — Case-sensitive default for `=`/`!=`/`IN` confirmed (no change to existing semantics)
- **D.2** — Three new opt-in case-insensitive operators: `IEQ`, `IIN`, `INE`
- **D.3** — Adapter-boundary canonical-case normalization for ALL OCSF enum-label string fields
- **D.4** — Discoverability: grammar resource + `prism describe` pedagogical examples

**Demo-critical defect addressed:** LLM agents authoring PrismQL queries consistently write
`WHERE severity IN ('HIGH', 'CRITICAL')` but prism stores OCSF Title-case labels (`'High'`,
`'Critical'`) per `enum_map.rs`. The T13 demo scenario returned zero rows silently. This story
closes the defect via two complementary mechanisms: IEQ/IIN normalize at query time, and
adapter-boundary normalization ensures stored data is consistently Title-cased so case-sensitive
`=` also works for well-formed OCSF enum-label fields.

**OD-4 explicitly excluded:** The zero-rows near-miss pedagogical hint is DEFERRED to a
follow-up story per human sign-off D-1398 (2026-06-27). This story does NOT implement the
hint; a dedicated future story must be created for it.

---

## Narrative

As a SOC analyst or LLM agent authoring PrismQL queries, I want to use
`severity IEQ 'high'` / `status IIN ('open', 'new')` to match OCSF enum-label fields
regardless of case — and have those fields pre-normalized to canonical Title-case at
ingestion — so that `WHERE severity IN ('HIGH', 'CRITICAL')` returns rows from all sensors
consistently, `GROUP BY severity` produces 5 buckets (not 8+ fragmented variants), and the
T13 demo query succeeds without requiring exact case knowledge from the analyst.

---

## Behavioral Contracts

| BC | Version | Title | Key Clauses Used |
|----|---------|-------|-----------------|
| BC-2.11.024 | v1.4 | PrismQL Case-Insensitive Equality and Membership Operators (IEQ / IIN / INE) | New operator syntax; DataFusion lower() lowering; case-sensitive operators unchanged; normalized_pql round-trip; IEQ superset invariant; IIN non-empty invariant; E-QUERY-001 (non-string RHS, empty list, SQL-mode rejection); E-QUERY-002 (non-string column); Mode-Boundary Enforcement (SQL-mode IEQ/IIN/INE rejection for ALL raw-SQL incl. DML WHERE + INSERT...SELECT) |
| BC-2.02.013 | v1.9 | Adapter-Boundary OCSF Enum-Label Canonical-Case Normalization | PRIMARY insertion point: `build_column_array` in `spec_driven_adapter.rs` (architect adjudication F-CRIT-002); SECONDARY: `normalize_with_mappers` (DynamicMessage path); severity + status guaranteed; all OCSF enum-label fields; idempotent; 50-codepoint value+sensor_type cap with warn; empty-string bypass EC-02-028; unrecognized values as-received + warning; GROUP BY aggregation consistency; enum_map.rs as sole casing authority |
| BC-2.11.002 | v1.6 | PrismQL Filter Mode Parsing | Amended: IEQ/IIN/INE added to supported filter-mode operator table |
| BC-2.11.004 | v1.24 | PrismQL Pipe Mode | Amended: IEQ/IIN/INE available in \| where stages via shared filter grammar (ADR-046 D7) |
| BC-2.11.018 | v1.3 | normalized_pql Echo | Amended: EC-11-057 added — IEQ/IIN/INE predicates reflected in uppercase canonical form in normalized_pql; round-trip invariant extended |
| BC-2.02.002 | v1.5 | DynamicMessage Creation | Amended: normalization applied BEFORE DynamicMessage creation; postconditions updated to state this explicitly |
| BC-2.02.010 | v1.5 | OCSF Enum Value Map | Amended: enum_map.rs authority extends to adapter-boundary normalization, not only MCP display enrichment |
| BC-2.10.012 | v1.9 | prism describe Output — `example_query` Pure-PQL Invariant and `example_note` Field Contract | pure-parseable `example_query` invariant + `example_note: Option<String>` field contract (§example_query + §example_note) |

---

## Token Budget Estimate

| Component | Estimated tokens |
|-----------|-----------------|
| Story spec (this file) | ~18,000 |
| ADR-047 (full) | ~6,000 |
| Design map: prismql-case-insensitive-design-map.md | ~4,500 |
| BC-2.11.024 v1.4 | ~3,000 |
| BC-2.02.013 v1.9 | ~2,500 |
| BC-2.11.002 v1.6 (relevant filter-mode sections) | ~1,500 |
| BC-2.11.004 v1.24 (relevant pipe-mode sections) | ~1,500 |
| BC-2.11.018 v1.3 (normalized_pql section) | ~1,000 |
| BC-2.02.002 v1.5, BC-2.02.010 v1.5 (amended sections) | ~2,000 |
| BC-2.10.012 v1.9 (example_query + example_note contract) | ~1,000 |
| `crates/prism-query/src/filter_parser.rs` | ~7,000 |
| `crates/prism-query/src/ast.rs` | ~9,000 |
| `crates/prism-query/src/pipe_sql_emitter.rs` | ~8,000 |
| `crates/prism-bin/src/spec_driven_adapter.rs` (PRIMARY: `build_column_array`) | ~5,000 |
| `crates/prism-ocsf/src/enum_map.rs` + normalizer (SECONDARY: `normalize_with_mappers`) | ~4,000 |
| `crates/prism-mcp/src/` (grammar resource, describe examples) | ~4,000 |
| Test files to create / modify | ~5,000 |
| **Total** | **~83,000** |

Estimated at ~42% of a 200K context window. Within the per-story limit. No split required.

---

## Acceptance Criteria

### AC-001 — IEQ parses to Predicate::Compare { case_insensitive: true }
(traces to BC-2.11.024 v1.4 postcondition "New operators and their syntax" — IEQ row;
BC-2.11.002 v1.6 amendment: IEQ added to filter-mode supported operator table)

Given a PrismQL filter-mode query string `severity IEQ 'high'`,
when the Chumsky parser in `filter_parser.rs` parses it,
then the resulting AST contains `Predicate::Compare { lhs: Box::new(Expr::Field(FieldPath::new(["severity"]))), op: CompareOp::Eq, rhs: Box::new(Expr::Literal(Literal::String("high".into()))), case_insensitive: true }`.

CODE-SHAPE NOTE (verified against `ast.rs` `pub enum Predicate`): the actual `Predicate::Compare`
variant is `{ lhs: Box<Expr>, op: CompareOp, rhs: Box<Expr> }` — NOT `{ field, op, value }`, and
there is NO `Value` type in the query AST (string literals are `Literal::String`, wrapped in
`Expr::Literal`). The IEQ parse site sets `lhs = field_path_to_expr(fp)` (→ `Expr::Field` for a
normal column) and `rhs = Expr::Literal(Literal::String(..))`. The new field `case_insensitive: bool`
is added to the variant.

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_parses_to_compare_case_insensitive_true`

### AC-002 — IIN parses to Predicate::In { case_insensitive: true }
(traces to BC-2.11.024 v1.4 postcondition "New operators and their syntax" — IIN row;
BC-2.11.002 v1.6 amendment)

Given `status IIN ('open', 'new')`,
when parsed,
then the AST contains `Predicate::In { field: FieldPath::new(["status"]), values: vec![Literal::String("open".into()), Literal::String("new".into())], negated: false, case_insensitive: true }`.

CODE-SHAPE NOTE (verified against `ast.rs`): the actual `Predicate::In` variant is
`{ field: FieldPath, values: Vec<Literal>, negated: bool }` — `field` is a `FieldPath` (not a
`String`) and `values` is `Vec<Literal>` (not `Vec<Value>`). The new field `case_insensitive: bool`
is added to the variant.

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_iin_parses_to_in_case_insensitive_true`

### AC-003 — INE parses to Predicate::Compare { op: CompareOp::Ne, case_insensitive: true }
(traces to BC-2.11.024 v1.4 postcondition "New operators and their syntax" — INE row)

Given `severity INE 'informational'`,
when parsed,
then the AST contains `Predicate::Compare { lhs: Box::new(Expr::Field(FieldPath::new(["severity"]))), op: CompareOp::Ne, rhs: Box::new(Expr::Literal(Literal::String("informational".into()))), case_insensitive: true }` (same `lhs`/`rhs` shape as AC-001; `op` is `CompareOp::Ne`).

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_ine_parses_to_compare_ne_case_insensitive_true`

### AC-004 — Keyword parsing is case-insensitive: `ieq`, `IEQ`, `Ieq` produce identical ASTs
(traces to BC-2.11.024 v1.4 postcondition "Operators are parsed case-insensitively in the
Chumsky grammar via the kw(...) combinator: ieq, IEQ, Ieq all parse identically")

Given three queries `severity ieq 'high'`, `severity IEQ 'high'`, and `severity Ieq 'high'`,
when each is parsed,
then all three produce structurally identical ASTs with `case_insensitive: true`.

NOTE: The `kw()` combinator already handles this — no additional implementation needed. This
AC verifies the combinator behavior is preserved.

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_keyword_case_insensitive_parsing`

### AC-005 — IIN parses before IN — no prefix-match collision
(traces to BC-2.11.024 v1.4 invariant: "IIN requires at least one value in the membership
list" — implicitly, IIN must parse at all, meaning the grammar combinator ordering must
not swallow IIN as a malformed IN; design map §A §Collision check)

Given `status IIN ('open')` (single-element IIN),
when parsed,
then the result is `Predicate::In { field: FieldPath::new(["status"]), values: vec![Literal::String("open".into())], negated: false, case_insensitive: true }`
— NOT a parse error and NOT `Predicate::In { .., case_insensitive: false }` (which would indicate
`IIN` was partially consumed as `IN` + stray `I`).

COLLISION NOTE (verified against `filter_parser.rs`): the project-local `kw()` helper matches a
full `[A-Za-z_]+` run via `eq_ignore_ascii_case`, so `kw("IN")` will NOT match the token `IIN`
(the full run `"IIN"` ≠ `"IN"`). Ordering `IIN` before `IN` in the `choice((...))` chain is still
the correct, defensive discipline (Context7-confirmed: `choice` tries alternatives in order,
most-specific-first), but the full-run `kw()` semantics provide a second layer of protection
against prefix-consumption. AC-005 remains a required guard.

IMPL NOTE: `IIN` must appear BEFORE `IN` in the Chumsky alternative chain. The longest
keyword must be tried first. Verify by reading the keyword parser section in filter_parser.rs.

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_iin_before_in_no_collision`

### AC-006 — Sibling-site sweep: all Predicate::Compare construction sites add case_insensitive: false
(traces to BC-2.11.024 v1.4 postcondition: "Each operator adds a case_insensitive: bool
flag (set to true) on the corresponding AST variant" — implies existing sites default to false;
TD-VSDD-060 sibling-site sweep rule)

After adding `case_insensitive: bool` to `Predicate::Compare`, the following grep runs
against `crates/prism-query/src/` returns ZERO construction sites lacking the new field:
```bash
rg 'Predicate::Compare\s*\{' crates/prism-query/src/ --type rust
```
Every match must show `case_insensitive: false` (or `case_insensitive: true` for IEQ/INE
parse sites). No construction site uses struct-update syntax (`..`) that would silently
drop the new field without specifying it.

VERIFY-ONLY AC — no standalone Red Gate test stub needed; compilation failure enforces this
once the field is added to the enum variant.

### AC-007 — Sibling-site sweep: all Predicate::In construction sites add case_insensitive: false
(traces to BC-2.11.024 v1.4 postcondition: "case_insensitive: bool flag" on Predicate::In;
TD-VSDD-060 sibling-site sweep rule)

Same verification as AC-006 for `Predicate::In`:
```bash
rg 'Predicate::In\s*\{' crates/prism-query/src/ --type rust
```
Every construction site must explicitly specify `case_insensitive: false` (or `true` for IIN).

VERIFY-ONLY AC — compilation enforces.

### AC-008 — IEQ lowers to `lower(field) = lower('val')` in DataFusion SQL
(traces to BC-2.11.024 v1.4 postcondition "DataFusion SQL lowering" table — IEQ row:
`lower(severity) = lower('high')`)

Given `Predicate::Compare { lhs: Box::new(Expr::Field(FieldPath::new(["severity"]))), op: CompareOp::Eq, rhs: Box::new(Expr::Literal(Literal::String("high".into()))), case_insensitive: true }`,
when `predicate_to_datafusion_sql` in `pipe_sql_emitter.rs` processes it,
then the emitted string is `lower(severity) = lower('high')`.

EMITTER NOTE (verified against `pipe_sql_emitter.rs`): the `Predicate::Compare` match arm
currently destructures `{ lhs, op, rhs }` and computes `expr_to_sql(lhs)` / `expr_to_sql(rhs)`.
The `case_insensitive: true` branch wraps BOTH sides: `lower({expr_to_sql(lhs)}) = lower({expr_to_sql(rhs)})`.
`expr_to_sql(Expr::Field(["severity"]))` → `severity`; `expr_to_sql(Expr::Literal(Literal::String("high")))` → `'high'`.

The `ILIKE` function is NOT used — it is a pattern operator (ADR-047 §Alternatives Alt-2).

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_emits_lower_equals_lower`
Red Gate (RG-045 / pass-9): `test_BC_2_11_024_ieq_predicate_excluded_from_equality_pushdown` — `crates/prism-query/src/pushdown.rs` (inline test module) — BC-2.11.024 invariant guard: IEQ predicates excluded from case-sensitive equality push-down (pass-9 F-P9-LOW-1)

### AC-009 — INE lowers to `lower(field) != lower('val')`
(traces to BC-2.11.024 v1.4 postcondition "DataFusion SQL lowering" table — INE row:
`lower(severity) != lower('low')`)

Given `Predicate::Compare { lhs: Box::new(Expr::Field(FieldPath::new(["severity"]))), op: CompareOp::Ne, rhs: Box::new(Expr::Literal(Literal::String("low".into()))), case_insensitive: true }` for `severity INE 'low'`,
when `predicate_to_datafusion_sql` processes it,
then the emitted string is `lower(severity) != lower('low')`.

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_ine_emits_lower_ne_lower`

### AC-010 — IIN lowers to `lower(field) IN (lower('v1'), lower('v2'), ...)`
(traces to BC-2.11.024 v1.4 postcondition "DataFusion SQL lowering" table — IIN row:
`lower(severity) IN (lower('high'), lower('critical'))`)

Given `Predicate::In { field: FieldPath::new(["severity"]), values: vec![Literal::String("high".into()), Literal::String("critical".into())], negated: false, case_insensitive: true }`,
when `predicate_to_datafusion_sql` processes it,
then the emitted string is `lower(severity) IN (lower('high'), lower('critical'))`.

`lower()` is applied to BOTH the field reference AND each literal value in the list. EMITTER NOTE:
the existing `Predicate::In` match arm destructures `{ field, values, negated }` and builds the
value list via `literal_to_sql`; the `case_insensitive: true` branch wraps `field_path_to_sql(field)`
in `lower(..)` and each `literal_to_sql(v)` in `lower(..)`.

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_iin_emits_lower_in_lower_list`

### AC-011 — Case-sensitive `=`, `!=`, `IN` emit unchanged (no lower() wrapping)
(traces to BC-2.11.024 v1.4 postcondition "Relationship to case-sensitive operators":
"The default operators =, !=, and IN are unchanged — they retain case-sensitive exact-match
semantics"; BC-2.11.024 v1.4 invariant: "IEQ is a strict superset of =: for any two values
with identical casing, IEQ matches if and only if = would match")

Given `Predicate::Compare { op: CompareOp::Eq, case_insensitive: false }` for `severity = 'High'`,
when `predicate_to_datafusion_sql` processes it,
then the emitted string is `severity = 'High'` — NO `lower()` wrapping applied.

Same for `!=` (case_insensitive: false → `severity != 'Low'`) and `IN` (case_insensitive: false).

NOTE: This is a regression guard — no behavioral change, verifying that the case_insensitive
flag is respected and the existing paths are NOT modified.

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_case_sensitive_eq_no_lower_wrapping`

### AC-012 — IEQ execution: matches rows regardless of casing
(traces to BC-2.11.024 v1.4 canonical test vector #1: `severity IEQ 'high'` against
`{severity: 'High'}` → Row returned;
test vector #2: `severity IEQ 'HIGH'` against `{severity: 'High'}` → Row returned;
BC-2.11.002 v1.6 amendment: IEQ operational in filter mode)

Given a DataFusion MemTable with one row `{severity: 'High'}`,
when the query `severity IEQ 'high'` is executed,
then the row is returned.

Given the same MemTable,
when `severity IEQ 'HIGH'` is executed,
then the row is also returned (both cases match via `lower()`).

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_execution_case_insensitive_match`

### AC-013 — Case-sensitive `=` returns 0 rows when casing differs (regression guard)
(traces to BC-2.11.024 v1.4 canonical test vector #6: "regression-no-change" —
`severity = 'High'` against `{severity: 'HIGH'}` → Row NOT returned;
BC-2.11.024 v1.4 invariant: "Precision differences appear only when casing differs")

Given a MemTable with `{severity: 'High'}`,
when `severity = 'high'` (case-sensitive) is executed,
then 0 rows are returned.

NOTE: This is the core regression guard. If this test fails after implementation, the
case-sensitive default has been broken — a critical behavioral regression.

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_case_sensitive_eq_returns_zero_on_casing_mismatch`

### AC-013b — IEQ/IIN available in pipe-mode | where stage
(traces to BC-2.11.024 v1.4 invariant: "IEQ/IIN/INE are valid in filter mode and in
pipe-mode | where stages (shared grammar invariant, BC-2.11.023)";
BC-2.11.004 v1.24 amendment)

Given `FROM crowdstrike_detections | where severity IEQ 'high' | head 5`,
when parsed and executed,
then the query succeeds and returns rows where `lower(severity) = lower('high')`.

NOTE: The shared filter grammar (ADR-046 D7) means IEQ/IIN/INE work in pipe-mode `| where`
automatically once the grammar changes land. This AC verifies no parser plumbing is
accidentally missing for the pipe path.

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_in_pipe_where_stage`

### AC-014 — normalized_pql reflects IEQ/IIN/INE in uppercase canonical form
(traces to BC-2.11.024 v1.4 postcondition "normalized_pql round-trip":
"IEQ, IIN, and INE predicates are reflected in the normalized_pql field";
"the Chumsky normalizer emits operator keywords in uppercase canonical form
(e.g., `severity ieq 'high'` normalizes to `severity IEQ 'high'`)";
BC-2.11.018 v1.3 amendment EC-11-057)

Given a query `severity ieq 'high'` (lowercase operator keyword),
when executed successfully,
then the `normalized_pql` field in the response contains `severity IEQ 'high'`
(uppercase canonical operator).

Given a query `status IIN ('open', 'new')`,
when executed,
then `normalized_pql` contains `status IIN ('open', 'new')` (IIN uppercase retained).

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_normalized_pql_reflects_ieq_uppercase`

### AC-015 — normalized_pql round-trip: parse → normalize → re-parse → same AST
(traces to BC-2.11.024 v1.4 postcondition "normalized_pql round-trip":
"The round-trip guarantee from BC-2.11.018 applies: the value in normalized_pql parses
back to the same AST";
BC-2.11.018 v1.3 amendment: round-trip invariant extended to cover IEQ/IIN/INE)

Given the original query `severity IEQ 'high'`,
step 1: parse to `ast_original`,
step 2: normalize `ast_original` to PQL string `normalized_str`,
step 3: parse `normalized_str` to `ast_reparsed`,
then `ast_original == ast_reparsed` (AST structural equality).

The `normalized_str` must contain `IEQ` (uppercase) per AC-014.

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_normalized_pql_round_trip_ast_equality`

### AC-016 — OCSF enum-label fields normalized to canonical Title-case via build_column_array (PRIMARY path)
(traces to BC-2.02.013 v1.9 postconditions:
"Before the Arrow StringArray cell is materialized in `build_column_array` (`spec_driven_adapter.rs`),
every OCSF enum-label string column value is rewritten to its canonical OCSF Title-case casing
from `enum_map.rs`";
"Severity (guaranteed): 'HIGH' → 'High', 'high' → 'High', 'CRITICAL' → 'Critical'";
"Status (guaranteed): normalized to OCSF Title-case";
BC-2.02.002 v1.5 amendment; BC-2.02.010 v1.5 amendment; architect adjudication F-CRIT-002)

Given a sensor JSON record delivered through `build_column_array` in `spec_driven_adapter.rs`,
when the column type is `ColumnType::String` and the column name is `severity` or `status`,
then the materialized Arrow `StringArray` cell contains the canonical OCSF Title-case value
(e.g., `'HIGH'` → `'High'`, `'CRITICAL'` → `'Critical'`).
`enum_map.rs` is the sole source for canonical captions (BC-2.02.010 v1.5 invariant).

PRIMARY Red Gate: `test_BC_2_02_013_build_column_array_normalizes_severity_to_title_case`
SECONDARY Red Gate (`normalize_with_mappers` DynamicMessage path): `test_S_PRISMQL_CASE_INSENSITIVE_001_adapter_normalization_critical_to_title_case`

OCSF IN-SCOPE FIELDS NOTE (BC-2.02.013 v1.9): The four in-scope enum-label string fields
guaranteed by this story are: `severity`, `status`, `activity_name`, and `disposition`.
The OCSF string label for the activity dimension is `activity_name` (NOT `activity` —
`activity_name` is the OCSF-canonical field name per BC-2.02.013 v1.9). When reading
`enum_map.rs`, sensor TOML specs, or writing test fixtures, always use `activity_name` for
the activity string label column; `activity` refers to a different field (the raw numeric
activity_id context field, not the normalized string label).

### AC-017 — Normalization via build_column_array covers activity_name and disposition; idempotent (PRIMARY path)
(traces to BC-2.02.013 v1.9 postcondition:
"The normalization function is idempotent: if the field already contains the canonical-case
value (e.g., 'High'), the value is unchanged. Re-normalizing already-canonical data has no
effect"; EC-02-020: CrowdStrike adapter emits severity='High' (already canonical Title-case)
→ value unchanged; no warning)

Given a sensor JSON record delivered through `build_column_array` in `spec_driven_adapter.rs`,
when the column type is `ColumnType::String` and the column name is `activity_name` or `disposition`,
then the materialized Arrow `StringArray` cell contains the canonical OCSF Title-case value;
and when the field value is already canonical (e.g., `severity='High'`), it is unchanged with
no warning emitted. Non-enum String columns (e.g., `hostname`) and non-String columns must NOT
be touched by normalization.

PRIMARY Red Gate: `test_BC_2_02_013_build_column_array_normalizes_status_and_disposition`
PRIMARY Guard: `test_BC_2_02_013_build_column_array_non_enum_string_column_untouched`
PRIMARY Guard: `test_BC_2_02_013_build_column_array_non_string_column_untouched`
SECONDARY Red Gate (`normalize_with_mappers` DynamicMessage path): `test_S_PRISMQL_CASE_INSENSITIVE_001_adapter_normalization_idempotent_high`

### AC-018 — Unrecognized vendor values left as-received with warning logged (PRIMARY path: build_column_array)
(traces to BC-2.02.013 v1.9 error cases:
"Warning (non-fatal): An OCSF enum-label field value has no matching caption in enum_map.rs";
EC-02-021: Armis adapter emits severity='UNHANDLED' (vendor-specific value) → value left
as-received, warning logged)

When `OcsfEnumMap::normalize_enum_label` returns `None` for a String enum-label column in
`build_column_array`, the raw value is materialized as-received into the Arrow `StringArray`
AND `tracing::warn!(event_type = "ocsf.enum_label_unrecognized", ...)` is emitted with the
schema from BC-2.02.013 v1.9 §Postconditions. This is the PRIMARY site; the SECONDARY site
in `normalize_with_mappers` must independently satisfy the same contract for the DynamicMessage path.

Given a sensor JSON record with `severity='UNHANDLED'` (Armis vendor-specific value not in
OCSF severity captions in `enum_map.rs`),
when `build_column_array` processes it,
then:
1. The Arrow `StringArray` cell has `severity='UNHANDLED'` (value unchanged)
2. `tracing::warn!(event_type = "ocsf.enum_label_unrecognized", field_name = ..., value = ..., sensor_type = ...)` is emitted
3. The normalization does NOT fail or return an error — it is non-fatal
4. Both the `value` and `sensor_type` fields in the warning payload MUST be capped at 50
   codepoints: if either field exceeds 50 codepoints, it MUST be truncated to the first
   50 codepoints with no ellipsis sentinel appended (BC-2.02.013 v1.9 / BC-2.16.002 v2.07
   specify a plain 50-codepoint cap on both fields — SEC-002; no `…` suffix). This applies
   to BOTH the PRIMARY (`build_column_array`) and SECONDARY (`normalize_with_mappers`) emission sites.

PRIMARY Red Gate: `test_BC_2_02_013_build_column_array_unrecognized_left_as_received_with_warn`
SECONDARY Red Gate (`normalize_with_mappers` DynamicMessage path): `test_S_PRISMQL_CASE_INSENSITIVE_001_adapter_normalization_unrecognized_value_left_as_received`
Guard (RG-047 / pass-10): `test_BC_2_02_013_build_column_array_empty_string_enum_value_no_warn` — `crates/prism-bin/src/spec_driven_adapter.rs` (test module) — empty-string enum values bypass normalization AND emit NO warn (PRIMARY↔SECONDARY behavioral parity)
Guard (RG-054 / pass-12): `test_BC_2_02_013_normalizer_secondary_empty_string_enum_value_no_warn` — `crates/prism-ocsf/src/tests/test_adapter_normalization.rs` — AC-018 guard — SECONDARY-path empty-string enum value passes through unchanged with NO warn (PRIMARY↔SECONDARY parity mirror of RG-047; pass-12 OBS-1)

### AC-019 — GROUP BY severity produces at most 7 buckets after normalization via build_column_array (PRIMARY path)
(traces to BC-2.02.013 v1.9 canonical test vector:
"PrismQL GROUP BY severity across CrowdStrike + Armis after normalization: 'High' appears
as one bucket — not split into 'High' + 'HIGH'";
EC-02-026: Cross-sensor aggregation correct after normalization)

After normalization via `build_column_array` on the spec-driven adapter path, a `GROUP BY severity`
query across CrowdStrike (emits `'High'`) and Armis (originally emits `'HIGH'`, normalized to
`'High'` by `build_column_array`) produces at most 7 distinct buckets — no fragmentation between
`'High'` and `'HIGH'`. This verifies the PRIMARY insertion point satisfies the BC-2.02.013
cross-sensor consistency postcondition.

Given simulated sensor records:
- CrowdStrike: `severity='High'` (3 records, already canonical)
- Armis-like: `severity='HIGH'` (2 records, normalized to `'High'` by `build_column_array`)
- Any sensor: `severity='Critical'` (1 record)

when `GROUP BY severity` is executed after normalization,
then the `'High'` bucket contains 5 rows and the `'Critical'` bucket contains 1 row —
NOT two separate `'High'` (3) and `'HIGH'` (2) buckets.

This verifies that the PRIMARY adapter-boundary normalization in `build_column_array` eliminates
GROUP BY fragmentation (ADR-047 §Consequences positive: "GROUP BY severity produces correct
aggregation across sensors").

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_group_by_severity_no_case_fragmentation`
Red Gate (RG-044 / pass-9): `test_BC_2_02_013_triage_alerts_prompt_no_stale_vendor_casing` — `crates/prism-mcp/tests/bc_2_02_013_prompt_casing_test.rs` — triage prompt armis leg uses IN ('High','Critical') (case-sensitive IN with Title-case literals per pass-10 F-CRIT-1 correction); `status = 'UNHANDLED'` intentionally retained (vendor value not in OCSF status map, passes through unnormalized; adjudicated pass-9)

### AC-020 — E-QUERY-001: IEQ/INE with non-string literal RHS rejected at parse time
(traces to BC-2.11.024 v1.4 error case: "E-QUERY-001: IEQ/INE with a non-string literal
on the RHS (e.g., severity IEQ 42) — Parse error: 'IEQ/INE require a string literal
as the right-hand side value'")

Given `severity IEQ 42` (integer literal on RHS),
when parsed,
then the result is `Err(E-QUERY-001)` with a message indicating IEQ requires a string literal.

CONTRACT REFINEMENT (pass-14 F-LOW-001): ADR-052 introduced `Literal::RawTemporalLiteral`
for bare date-like tokens. However, QUOTED date-like strings (e.g., `'2026-06-01'`) are
always a `Literal::String` — the quoted form is never reclassified as temporal. Therefore,
`severity IEQ '2026-06-01'` is VALID: the RHS `'2026-06-01'` is a string literal, accepted
unchanged. Only BARE non-string literals (unquoted integers like `42`) are rejected. This
contract applies equally to INE (single RHS) and IIN (list elements). The AC title
"non-string literal RHS" is precise: a quoted string is always a string literal.

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_non_string_rhs_e_query_001`
Red Gate (RG-055 / pass-14): `test_BC_2_11_024_ieq_date_like_rhs_accepted_as_string` — `crates/prism-query/src/tests/test_case_insensitive_operators.rs` — AC-020 refinement — quoted date-like RHS ('2026-06-01') accepted as string for IEQ
Red Gate (RG-056 / pass-14): `test_BC_2_11_024_ine_date_like_rhs_accepted_as_string` — `crates/prism-query/src/tests/test_case_insensitive_operators.rs` — AC-020 refinement — same for INE: quoted date-like RHS accepted as string
Red Gate (RG-057 / pass-14): `test_BC_2_11_024_iin_date_like_elements_accepted_as_string` — `crates/prism-query/src/tests/test_case_insensitive_operators.rs` — AC-020 refinement — same for IIN list elements: quoted date-like elements accepted as strings

### AC-021 — E-QUERY-001: IIN with empty membership list rejected at parse time
(traces to BC-2.11.024 v1.4 error case: "E-QUERY-001: IIN with an empty membership list:
severity IIN () — Parse error: 'IIN requires at least one value in the membership list'";
BC-2.11.024 v1.4 invariant: "IIN requires at least one value in the membership list.
An empty IIN () list is a parse error (E-QUERY-001)")

Given `severity IIN ()`,
when parsed,
then the result is `Err(E-QUERY-001)` with a message indicating IIN requires at least one
value in the membership list; empty () is not valid. Use at least one quoted string:
`severity IIN ('value')`.

NOTE (BC-2.11.024 v1.4 EC-11-024-012 extension): the same parse-time E-QUERY-001 error
applies when IIN list elements are non-string literals (integers, booleans) — not only
when the list is empty. RG-059 and RG-060 cover those cases.

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_iin_empty_list_e_query_001`

### AC-022 — E-QUERY-002: IEQ/IIN/INE on non-string column returns QueryTypeMismatch
(traces to BC-2.11.024 v1.4 error case:
"E-QUERY-002: IEQ/IIN/INE applied to a non-string column (e.g., severity_id IEQ 'high'
where severity_id is an integer column) — QueryTypeMismatch: not applicable to non-string
types; error includes field name, actual type, operator, and — when the column is a known
OCSF integer-id field — suggests the corresponding string label column";
BC-2.11.024 v1.4 precondition: "The field referenced by IEQ/IIN/INE is a string-type column
in the DataFusion execution schema. Applying lower() to a non-string column results in
E-QUERY-002 (QueryTypeMismatch)")

Given a DataFusion schema where `severity_id` is an integer column,
when `severity_id IEQ 'high'` is executed,
then the result is `Err(E-QUERY-002 QueryTypeMismatch)` with a Display message containing:
- the column name (`severity_id`), its actual type (`Integer`), and the operator (`IEQ`)
- the suggestion: "for label comparison, use the string column 'severity' with IEQ/IIN/INE
  instead" (because `severity_id` is a known OCSF integer-id field; `PrismError::QueryTypeMismatch`
  carries `suggested_column: Some("severity")` per the OCSF sibling lookup contract in
  error-taxonomy.md v2.32 §E-QUERY-002)

The full Display for this case must be:
  "E-QUERY-002: type mismatch — column 'severity_id' in table '<table>' has type 'Integer'
   which does not support operator 'IEQ'; for label comparison, use the string column
   'severity' with IEQ/IIN/INE instead"

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_integer_column_e_query_002`
Red Gate (RG-041 / pass-8): `test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_integer_column_sqlpipe_pipe_stage_e_query_002` — SqlPipe pipe-stage `| where severity_id IEQ 'high'` on Int64 column → structured E-QUERY-002 with operator IEQ + suggested_column 'severity' (pass-8 F-MED-1: SqlPipe arm CI pre-flight; Filter/Pipe/SqlPipe all guarded)
Red Gate (RG-042 / pass-8): `test_S_PRISMQL_CASE_INSENSITIVE_001_iin_integer_column_sqlpipe_pipe_stage_e_query_002` — IIN sibling on the same path

### AC-023 — SQL-mode IEQ/IIN/INE rejection — structured E-QUERY-001
(traces to BC-2.11.024 v1.4 §Mode-Boundary Enforcement invariant)

Given a PrismQL query in raw SQL mode — any of: `SELECT` statements, `DELETE`/`UPDATE` statements
containing a WHERE clause, or `INSERT ... SELECT` statements — that contains an IEQ, IIN, or INE
operator (e.g., `SELECT * FROM t WHERE severity IEQ 'high'`,
`DELETE FROM t WHERE severity IEQ 'high'`,
`UPDATE t SET x=1 WHERE status IIN ('open', 'new')`),
when the parser processes it,
then the parser MUST reject it at PARSE TIME with `E-QUERY-001` BEFORE any DataFusion
planning or execution, with the verbatim message:
`"E-QUERY-001: parse error near '{operator}': case-insensitive operators (IEQ/IIN/INE) are not supported in SQL mode. Use filter mode (e.g., severity IEQ 'high') or a pipe | where stage (e.g., FROM crowdstrike_detections | where severity IEQ 'high') instead."`
where `{operator}` is the encountered keyword in uppercase (IEQ, IIN, or INE).

**DML rejection requirement (BC-2.11.024 v1.4 scope extension):** The detection MUST also run
in `parse_sql_dml_with_limits` before DML dispatch — the same keyword scan that guards SELECT
MUST guard DELETE/UPDATE/INSERT. A code comment near `parse_sql_dml_with_limits` must acknowledge
this guard explicitly (e.g.,
`// IEQ/IIN/INE SQL-mode guard: reject before DML dispatch per BC-2.11.024 §Mode-Boundary Enforcement`).

MCP error mapping: -32602 INVALID_PARAMS.

The rejection MUST NOT produce `E-QUERY-034 QueryExecutionFailed` — that would indicate the
query reached DataFusion execution before rejection. The error code MUST be `E-QUERY-001`.

Regression vector: existing filter-mode AC-001 (`severity IEQ 'high'`) and pipe-mode AC-013b
(`FROM t | where severity IEQ 'high'`) Red Gate tests confirm no regression from the SQL-mode gate.

Red Gate: `test_BC_2_11_024_sql_mode_ieq_rejected`
Red Gate: `test_BC_2_11_024_sql_mode_iin_rejected`
Red Gate: `test_BC_2_11_024_sql_mode_ine_rejected`
Red Gate: `test_BC_2_11_024_dml_delete_where_ieq_rejected`
Red Gate: `test_BC_2_11_024_dml_update_where_iin_rejected`
Red Gate: `test_BC_2_11_024_dml_insert_select_where_ine_rejected`
Red Gate (RG-046 / pass-10): `test_BC_2_11_024_all_prompt_embedded_queries_parse` — `crates/prism-mcp/tests/bc_2_11_024_test.rs` — AC-023 corollary (prompt-content mode-boundary coherence) — every MCP-prompt-embedded PrismQL query parses Ok (pass-10 F-CRIT-1: armis leg IIN-in-raw-SQL regression caught; fixed to `IN ('High','Critical')`)
Red Gate (RG-058 / pass-14): `test_BC_2_11_024_sqlpipe_head_where_ieq_rejected` — `crates/prism-query/src/tests/test_case_insensitive_operators.rs` — AC-023 guard — SqlPipe head WHERE IEQ mode-boundary rejection (green lock, symmetric with HAVING; pass-14)

**Compound violations precedence:** When a query violates both the string-literal RHS rule and
the SQL-mode boundary (e.g., `SELECT ... WHERE severity IEQ 42`), the string-literal E-QUERY-001
template takes precedence — the parse-stage `try_map` fires at the literal span before the
post-parse mode-boundary walk runs. Both templates carry E-QUERY-001 and MCP -32602; either
error satisfies the parse-time-rejection requirement of this AC.

### AC-024 — PrismQL grammar reference resource includes IEQ/IIN/INE in operator table
(traces to BC-2.11.024 v1.4 architecture anchor: ADR-047 §D.4 discoverability;
BC-2.11.002 v1.6 amendment: IEQ/IIN/INE in the operator table;
"IEQ/IIN must be reflected in the PrismQL grammar reference resource (governed by
BC-2.11.022/ADR-045 parity gate)")

Given the auto-generated PrismQL grammar reference MCP resource (BC-2.11.022 / ADR-045),
when the grammar resource parity gate is run,
then `IEQ`, `IIN`, and `INE` appear in the operator table, and the ADR-045 parity gate
CI check passes.

NOTE: The implementer must locate the grammar resource generation code and add the three
new operators. The ADR-045 parity gate is a CI check — the PR must not break it.

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_grammar_resource_includes_ieq_iin_ine`
Red Gate (RG-043 / pass-9): `test_BC_2_11_024_reference_content_no_stale_vendor_cased_enum_examples` — `crates/prism-mcp/tests/reference_content.rs` — no vendor-cased enum-equality examples in prismql://reference (post-normalization 0-row guard; pass-9 F-P9-MED-2)

### AC-025 — prism describe output includes IEQ example with OCSF casing note in example_note field
(traces to ADR-047 §D.4: "The prism describe / tool-schema pedagogical examples (ADR-041
L1/L2 teaching surface)" — "Include the OCSF casing note: 'OCSF severity is stored as
Title-case (High). Use IEQ/IIN to match regardless of the case you type, or = 'High' for
the exact canonical form'")

BC-2.10.012 v1.9 INTERACTION NOTE: BC-2.10.012 v1.9 governs the describe output surface
touched by this AC. Under v1.9, the `example_query` field must be pure parseable PQL — it
MUST NOT contain SQL comment lines (`--` or `/* */`). The OCSF casing note lives in the
separate `example_note` field (`Option<String>`). For severity tables the `example_note` is
`Some("OCSF severity is stored as Title-case ('High'). Use IEQ/IIN to match regardless of
the case you type, or = 'High' for the exact canonical form.")`. For tables without a
`severity` column the `example_note` for non-severity examples is `None`.

Given the `prism describe <table>` command output for any sensor table with a `severity`
column,
when the output is inspected,
then:
1. The `example_query` field contains a pure parseable PQL query using `IEQ` with a
   severity value — e.g., `FROM <table> | where severity IEQ 'high' | limit 50` — with NO
   embedded SQL comment lines (BC-2.10.012 v1.9 pure-PQL invariant).
2. The `example_note` field is `Some("OCSF severity is stored as Title-case ('High').
   Use IEQ/IIN to match regardless of the case you type, or = 'High' for the exact
   canonical form.")` — the pedagogical hint lives here, not in `example_query`.
3. The `example_query` value parses successfully without stripping any characters.

SUPPRESSION GUARD (F-MED-1 anchor): The IEQ example and OCSF casing note MUST NOT be
suppressed by other example variants generated for the same table. When a table has a
`severity` column AND other column types that trigger additional example variants (e.g.,
numeric columns that produce aggregate example variants), the example generator MUST emit
the IEQ example for EVERY such table regardless of which other variants are also present.
The presence of an aggregate or other variant does NOT justify omitting the IEQ example.

NOTE: This may be a snapshot/golden-file test or a content-inclusion test. The implementer
must decide the appropriate test mechanism for the describe output.

Red Gate (authoritative): `test_BC_2_11_024_describe_output_includes_ieq_example_and_ocsf_casing_note`
(in `prism_describe.rs` test module in prism-mcp — this is the real test from commit 26325423)
Red Gate (supplementary): `test_S_PRISMQL_CASE_INSENSITIVE_001_describe_output_includes_ieq_example`
(prism-query substitute test; may remain as supplementary coverage)
Red Gate (SUPPRESSION GUARD / F-MED-1 anchor): `test_f_med1_severity_vocabulary_table_ieq_not_suppressed_by_integer_column`
(in `prism_describe.rs` test module in prism-mcp — severity-vocabulary table with an Integer column still emits the IEQ example + casing note; aggregate variant does not suppress)

### AC-026 — No panic: IEQ/IIN expressions with multiple predicates do not panic (VP-021 regression)
(traces to BC-2.11.024 v1.4 canonical test vector: "severity IEQ 'high' AND severity IEQ 'high'
does not panic (fuzz-seed regression)" — design map T-CASE-009;
VP-021 invariant: parser never panics on arbitrary input)

Given `severity IEQ 'high' AND severity IEQ 'high'` (repeated IEQ predicate),
when parsed and processed,
then the result is either a valid query result or a structured error — NOT a panic.

This is a regression guard targeting the VP-021 fuzz property. The existing `vp021_parse_fuzz`
target in `fuzz/` covers this class of input; this unit test pins the specific fuzz seed.

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_repeated_ieq_no_panic`

### AC-027 — Non-exhaustive compile-fail gate count UNCHANGED at 89
(traces to BC-2.11.024 v1.4 invariant:
"No EXPECTED non-exhaustive gate count change: case_insensitive: bool is a new field added
to existing #[non_exhaustive] structs (Predicate::Compare, Predicate::In). The non-exhaustive
compile-fail gate counts annotated types, not field additions within existing types";
CLAUDE.md §Conventions: `ci.yml EXPECTED=89`)

Given the `scripts/check-non-exhaustive.sh` script after this story is implemented,
when the following command is run:
```bash
grep EXPECTED= scripts/check-non-exhaustive.sh
```
then the result shows `EXPECTED=89` — unchanged from the pre-story value.

If any new `#[non_exhaustive]`-annotated public type is introduced by this story
(not expected per design map §Non-Exhaustive Gate Impact Assessment), both `ci.yml` and
this AC must be amended before the PR merges.

VERIFY-ONLY AC — no standalone Red Gate test; verified by running `just check` and confirming
the non-exhaustive gate passes.

---

## Red Gate Test Inventory

All tests must be written as FAILING stubs (`todo!()`) BEFORE any production code is modified.
Verify ALL 25 core stubs (RG-001 through RG-025) fail before proceeding to Task 10 (production code implementation).

| RG ID | Test Function Name | Location | AC | Assertion |
|-------|--------------------|----------|----|-----------|
| RG-001 | `test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_parses_to_compare_case_insensitive_true` | `crates/prism-query/src/tests/` | AC-001 | AST equality: case_insensitive=true on Predicate::Compare{op:Eq} |
| RG-002 | `test_S_PRISMQL_CASE_INSENSITIVE_001_iin_parses_to_in_case_insensitive_true` | `crates/prism-query/src/tests/` | AC-002 | AST equality: case_insensitive=true on Predicate::In |
| RG-003 | `test_S_PRISMQL_CASE_INSENSITIVE_001_ine_parses_to_compare_ne_case_insensitive_true` | `crates/prism-query/src/tests/` | AC-003 | AST equality: CompareOp::Ne + case_insensitive=true |
| RG-004 | `test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_keyword_case_insensitive_parsing` | `crates/prism-query/src/tests/` | AC-004 | All 3 casing variants (ieq/IEQ/Ieq) produce identical AST |
| RG-005 | `test_S_PRISMQL_CASE_INSENSITIVE_001_iin_before_in_no_collision` | `crates/prism-query/src/tests/` | AC-005 | `status IIN ('open')` parses without error; case_insensitive=true |
| RG-006 | `test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_emits_lower_equals_lower` | `crates/prism-query/src/` (emitter test) | AC-008 | Emitter output string = `lower(severity) = lower('high')` |
| RG-007 | `test_S_PRISMQL_CASE_INSENSITIVE_001_ine_emits_lower_ne_lower` | `crates/prism-query/src/` (emitter test) | AC-009 | Emitter output = `lower(severity) != lower('low')` |
| RG-008 | `test_S_PRISMQL_CASE_INSENSITIVE_001_iin_emits_lower_in_lower_list` | `crates/prism-query/src/` (emitter test) | AC-010 | Emitter output = `lower(severity) IN (lower('high'), lower('critical'))` |
| RG-009 | `test_S_PRISMQL_CASE_INSENSITIVE_001_case_sensitive_eq_no_lower_wrapping` | `crates/prism-query/src/` (emitter test) | AC-011 | case_insensitive=false path emits `severity = 'High'` (no lower) |
| RG-010 | `test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_execution_case_insensitive_match` | `crates/prism-query/src/tests/` | AC-012 | Row returned for `severity IEQ 'high'` against `{severity: 'High'}` |
| RG-011 | `test_S_PRISMQL_CASE_INSENSITIVE_001_case_sensitive_eq_returns_zero_on_casing_mismatch` | `crates/prism-query/src/tests/` | AC-013 | `severity = 'high'` returns 0 rows against `{severity: 'High'}` |
| RG-012 | `test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_in_pipe_where_stage` | `crates/prism-query/src/tests/` | AC-013b | Pipe-mode `\| where severity IEQ 'high'` executes successfully |
| RG-013 | `test_S_PRISMQL_CASE_INSENSITIVE_001_normalized_pql_reflects_ieq_uppercase` | `crates/prism-query/src/tests/` | AC-014 | normalized_pql contains `IEQ` uppercase for `severity ieq 'high'` |
| RG-014 | `test_S_PRISMQL_CASE_INSENSITIVE_001_normalized_pql_round_trip_ast_equality` | `crates/prism-query/src/tests/` | AC-015 | Parse→normalize→reparse produces identical AST |
| RG-015 | `test_S_PRISMQL_CASE_INSENSITIVE_001_repeated_ieq_no_panic` | `crates/prism-query/src/tests/` | AC-026 | `IEQ x2 AND` does not panic — VP-021 regression guard |
| RG-016 | `test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_non_string_rhs_e_query_001` | `crates/prism-query/src/tests/` | AC-020 | `severity IEQ 42` → Err(E-QUERY-001) |
| RG-017 | `test_S_PRISMQL_CASE_INSENSITIVE_001_iin_empty_list_e_query_001` | `crates/prism-query/src/tests/` | AC-021 | `severity IIN ()` → Err(E-QUERY-001) |
| RG-018 | `test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_integer_column_e_query_002` | `crates/prism-query/src/tests/` | AC-022 | `severity_id IEQ 'high'` vs integer column → Err(E-QUERY-002) |
| RG-019 | `test_S_PRISMQL_CASE_INSENSITIVE_001_adapter_normalization_critical_to_title_case` | `crates/prism-ocsf/src/tests/` (SECONDARY: DynamicMessage path) | AC-016 | `severity='CRITICAL'` → `'Critical'` in DynamicMessage (`normalize_with_mappers` path) |
| RG-020 | `test_S_PRISMQL_CASE_INSENSITIVE_001_adapter_normalization_idempotent_high` | `crates/prism-ocsf/src/tests/` (SECONDARY: DynamicMessage path) | AC-017 | `severity='High'` unchanged; no warning emitted (`normalize_with_mappers` path) |
| RG-021 | `test_S_PRISMQL_CASE_INSENSITIVE_001_adapter_normalization_unrecognized_value_left_as_received` | `crates/prism-ocsf/src/tests/` (SECONDARY: DynamicMessage path) | AC-018 | `severity='UNHANDLED'` unchanged; warning emitted (`normalize_with_mappers` path) |
| RG-022 | `test_S_PRISMQL_CASE_INSENSITIVE_001_group_by_severity_no_case_fragmentation` | `crates/prism-query/src/tests/` | AC-019 | Multi-sensor GROUP BY produces 1 'High' bucket, not 2 fragmented buckets |
| RG-023 | `test_BC_2_11_024_sql_mode_ieq_rejected` | `crates/prism-query/src/tests/` | AC-023 | SQL-mode `SELECT … WHERE severity IEQ 'high'` → Err(E-QUERY-001); NOT QueryExecutionFailed/E-QUERY-034 |
| RG-024 | `test_BC_2_11_024_sql_mode_iin_rejected` | `crates/prism-query/src/tests/` | AC-023 | SQL-mode `SELECT … WHERE status IIN ('open', 'new')` → Err(E-QUERY-001) |
| RG-025 | `test_BC_2_11_024_sql_mode_ine_rejected` | `crates/prism-query/src/tests/` | AC-023 | SQL-mode `SELECT … WHERE severity INE 'low'` → Err(E-QUERY-001) |

**Additional Red Gate stubs (grammar resource + describe output):**

| RG ID | Test Function Name | Location | AC |
|-------|--------------------|----------|----|
| RG-026 (grammar resource) | `test_S_PRISMQL_CASE_INSENSITIVE_001_grammar_resource_includes_ieq_iin_ine` | `crates/prism-query/src/tests/` or `crates/prism-mcp/src/tests/` | AC-024 |
| RG-027 (describe output, supplementary) | `test_S_PRISMQL_CASE_INSENSITIVE_001_describe_output_includes_ieq_example` | `crates/prism-mcp/src/tests/` or `crates/prism-query/src/tests/` | AC-025 (supplementary — authoritative test is RG-028) |

NOTE: RG-026 is implementation-detail-dependent — the implementer must choose the appropriate
test location based on where the grammar resource generation code lives. RG-027 is now
supplementary; the authoritative describe test is RG-028 (the real prism-mcp test from
commit 26325423).

**Pass-5 new tests (from commit 26325423 and architect adjudication F-CRIT-002):**

| RG ID | Test Function Name | Location | AC | Assertion |
|-------|--------------------|----------|----|-----------|
| RG-028 | `test_BC_2_11_024_describe_output_includes_ieq_example_and_ocsf_casing_note` | `crates/prism-mcp/src/prism_describe.rs` test module | AC-025 | Authoritative: describe output contains IEQ example + OCSF casing note |
| RG-029 | `test_BC_2_11_024_query_type_mismatch_display_with_suggestion_exact` | `crates/prism-core/tests/bc_2_11_024_query_type_mismatch_display.rs` | AC-022 | Full-string byte-exact assertion of E-QUERY-002 QueryTypeMismatch Display WITH suggestion (sub-form b1, taxonomy v2.19); POL-24 byte-for-byte; F-CRIT-001 closure |
| RG-030 | `test_BC_2_11_024_query_type_mismatch_display_without_suggestion_exact` | `crates/prism-core/tests/bc_2_11_024_query_type_mismatch_display.rs` | AC-022 | Byte-exact Display WITHOUT suggestion (sub-form b2, taxonomy v2.19); POL-24 byte-for-byte; F-CRIT-001 closure |
| RG-031 | `test_BC_2_11_024_negated_case_insensitive_in_returns_query_plan_failed` | `crates/prism-query/src/tests/test_case_insensitive_operators.rs` | BC-2.11.024 invariant guard | Emitter returns Err(QueryPlanFailed) for hand-built Predicate::In { negated: true, case_insensitive: true }; IIN grammar is positive-only (not a parse test) |
| RG-032 | `test_BC_2_02_013_build_column_array_normalizes_severity_to_title_case` | `crates/prism-bin/src/` (spec_driven_adapter tests) | AC-016 | PRIMARY: Arrow StringArray cell contains canonical Title-case for severity column |
| RG-033 | `test_BC_2_02_013_build_column_array_normalizes_status_and_disposition` | `crates/prism-bin/src/` (spec_driven_adapter tests) | AC-017 | PRIMARY: status and disposition columns normalized to Title-case via build_column_array |
| RG-034 | `test_BC_2_02_013_build_column_array_unrecognized_left_as_received_with_warn` | `crates/prism-bin/src/` (spec_driven_adapter tests) | AC-018 | PRIMARY: unrecognized value left as-received + warn!(event_type="ocsf.enum_label_unrecognized") emitted |
| RG-035 | `test_BC_2_02_013_build_column_array_non_enum_string_column_untouched` | `crates/prism-bin/src/` (spec_driven_adapter tests) | AC-017 guard | Guard: non-enum String column (e.g., hostname) NOT modified by normalization in build_column_array |
| RG-036 | `test_BC_2_02_013_build_column_array_non_string_column_untouched` | `crates/prism-bin/src/` (spec_driven_adapter tests) | AC-017 guard | Guard: non-String column (e.g., Integer) NOT modified by normalization in build_column_array |
| RG-037 | `test_BC_2_11_024_dml_delete_where_ieq_rejected` | `crates/prism-query/src/tests/` | AC-023 | DML `DELETE FROM t WHERE severity IEQ 'high'` → Err(E-QUERY-001); detector invoked in `parse_sql_dml_with_limits` before DML dispatch |
| RG-038 | `test_BC_2_11_024_dml_update_where_iin_rejected` | `crates/prism-query/src/tests/` | AC-023 | DML `UPDATE t SET x=1 WHERE status IIN ('open', 'new')` → Err(E-QUERY-001); detector invoked in `parse_sql_dml_with_limits` before DML dispatch |
| RG-039 | `test_BC_2_11_024_dml_insert_select_where_ine_rejected` | `crates/prism-query/src/tests/test_case_insensitive_operators.rs` | AC-023 | INSERT...SELECT WHERE with INE rejected at parse time with verbatim E-QUERY-001 mode-boundary message (operator INE) |
| RG-040 | `test_f_med1_severity_vocabulary_table_ieq_not_suppressed_by_integer_column` | `crates/prism-mcp/src/tools/prism_describe.rs` (test module) | AC-025 | Severity-vocabulary table with an Integer column still emits the IEQ example + casing note (aggregate variant does not suppress) |

**Pass-8 new tests (SqlPipe pipe-stage E-QUERY-002 CI pre-flight):**

| RG ID | Test Function Name | Location | AC | Assertion |
|-------|--------------------|----------|----|-----------|
| RG-041 | `test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_integer_column_sqlpipe_pipe_stage_e_query_002` | `crates/prism-query/src/tests/test_case_insensitive_operators.rs` | AC-022 | SqlPipe pipe-stage `\| where severity_id IEQ 'high'` on Int64 column → E-QUERY-002 with operator IEQ + suggested_column 'severity' (Filter/Pipe/SqlPipe all guarded) |
| RG-042 | `test_S_PRISMQL_CASE_INSENSITIVE_001_iin_integer_column_sqlpipe_pipe_stage_e_query_002` | `crates/prism-query/src/tests/test_case_insensitive_operators.rs` | AC-022 | IIN sibling on the same path — `\| where severity_id IIN ('high', 'critical')` on Int64 column → E-QUERY-002 |

**Pass-9 new tests (reference-content casing, triage-prompt casing, IEQ pushdown-exclusion):**

| RG ID | Test Function Name | Location | AC | Assertion |
|-------|--------------------|----------|----|-----------|
| RG-043 | `test_BC_2_11_024_reference_content_no_stale_vendor_cased_enum_examples` | `crates/prism-mcp/tests/reference_content.rs` | AC-024 | prismql://reference content has no vendor-cased enum-equality examples post-normalization (0-row guard; pass-9 F-P9-MED-2) |
| RG-044 | `test_BC_2_02_013_triage_alerts_prompt_no_stale_vendor_casing` | `crates/prism-mcp/tests/bc_2_02_013_prompt_casing_test.rs` | AC-019 | triage prompt armis leg uses IN ('High','Critical') (case-sensitive IN with Title-case literals per pass-10 F-CRIT-1 correction); `status = 'UNHANDLED'` intentionally retained (vendor value passes through unnormalized per pass-9 adjudication) |
| RG-045 | `test_BC_2_11_024_ieq_predicate_excluded_from_equality_pushdown` | `crates/prism-query/src/pushdown.rs` (inline test module) | BC-2.11.024 invariant guard | IEQ predicates excluded from case-sensitive equality push-down (pass-9 F-P9-LOW-1) |

**Pass-10 new tests (prompt-parseability + empty-string no-warn):**

| RG ID | Test Function Name | Location | AC | Assertion |
|-------|--------------------|----------|----|-----------|
| RG-046 | `test_BC_2_11_024_all_prompt_embedded_queries_parse` | `crates/prism-mcp/tests/bc_2_11_024_test.rs` | AC-023 corollary | Every MCP-prompt-embedded PrismQL query parses Ok (pass-10 F-CRIT-1: armis leg IIN-in-raw-SQL regression caught; fixed to `IN ('High','Critical')`) |
| RG-047 | `test_BC_2_02_013_build_column_array_empty_string_enum_value_no_warn` | `crates/prism-bin/src/spec_driven_adapter.rs` (test module) | AC-018 guard | Empty-string enum values bypass normalization AND emit NO warn (PRIMARY↔SECONDARY behavioral parity; pass-10 OBS-3) |

**Pass-11 new tests (negated-IIN marker, detector real-op, IN-list AST lock, describe parse-lock, index collision guards):**

| RG ID | Test Function Name | Location | AC | Assertion |
|-------|--------------------|----------|----|-----------|
| RG-048 | `test_low1_negated_iin_emits_invalid_marker_not_plain_positive_iin` | `crates/prism-query/src/ast.rs` (module low1_negated_iin_invalid_marker_tests) | BC-2.11.024 invariant guard | Normalizer emits `<invalid: negated IIN...>` marker, never silent positive IIN (pass-11 F-LOW-1) |
| RG-049 | `test_low2_detect_ci_operator_gt_mentions_real_op_not_ieq` | `crates/prism-query/src/sql_parser.rs` (inline tests) | BC-2.11.024 invariant guard | Detector reports real op (`I[Gt]`) not hardcoded IEQ (pass-11 F-LOW-2) |
| RG-050 | `test_BC_2_11_003_sql_in_literal_list_parses_to_predicate_in` | `crates/prism-query/src/tests/parser_tests.rs` | AC-011 regression guard | SQL IN literal-list parses to Predicate::In, not RecoveryError (locks base.or(in_subquery) ordering; pass-11 F-OBS-1) |
| RG-051 | `test_obs2_all_build_example_query_outputs_parse_ok` | `crates/prism-mcp/src/tools/prism_describe.rs` | AC-025 guard | All describe example outputs parse Ok (pass-11 F-OBS-2) |
| RG-052 | `test_obs4_ocsf_enum_map_new_no_collisions_in_production_data` | `crates/prism-ocsf/src/tests/bc_2_02_010_enum_map.rs` | BC-2.02.013 guard | normalized_index collision determinism — OcsfEnumMap::new() produces no duplicate normalized keys across production data (pass-11 F-OBS-4) |
| RG-053 | `test_obs4_collision_detection_panics_on_ambiguous_captions` | `crates/prism-ocsf/src/tests/bc_2_02_010_enum_map.rs` | BC-2.02.013 guard | normalized_index collision detection panics on deliberately ambiguous captions (debug-mode guard; pass-11 F-OBS-4) |

**Pass-12 new tests (SECONDARY empty-string parity mirror):**

| RG ID | Test Function Name | Location | AC | Assertion |
|-------|--------------------|----------|----|-----------|
| RG-054 | `test_BC_2_02_013_normalizer_secondary_empty_string_enum_value_no_warn` | `crates/prism-ocsf/src/tests/test_adapter_normalization.rs` | AC-018 guard | SECONDARY-path empty-string enum value passes through unchanged with NO warn (PRIMARY↔SECONDARY parity mirror of RG-047; pass-12 OBS-1) |

**Pass-14 new tests (date-like RHS string acceptance + SqlPipe head WHERE IEQ lock):**

| RG ID | Test Function Name | Location | AC | Assertion |
|-------|--------------------|----------|----|-----------|
| RG-055 | `test_BC_2_11_024_ieq_date_like_rhs_accepted_as_string` | `crates/prism-query/src/tests/test_case_insensitive_operators.rs` | AC-020 refinement | Quoted date-like RHS ('2026-06-01') accepted as string for IEQ (ADR-052 reclassification unwrapped; pass-14 F-LOW-001) |
| RG-056 | `test_BC_2_11_024_ine_date_like_rhs_accepted_as_string` | `crates/prism-query/src/tests/test_case_insensitive_operators.rs` | AC-020 refinement | Same for INE: quoted date-like RHS accepted as string |
| RG-057 | `test_BC_2_11_024_iin_date_like_elements_accepted_as_string` | `crates/prism-query/src/tests/test_case_insensitive_operators.rs` | AC-020 refinement | Same for IIN list elements: quoted date-like elements accepted as strings |
| RG-058 | `test_BC_2_11_024_sqlpipe_head_where_ieq_rejected` | `crates/prism-query/src/tests/test_case_insensitive_operators.rs` | AC-023 guard | SqlPipe head WHERE IEQ mode-boundary rejection (green lock, symmetric with HAVING; pass-14) |

**Pass-15 new tests (IIN non-string parse-time ×2, example_query purity ×2, example_note contract, precedence lock):**

| RG ID | Test Function Name | Location | AC | Assertion |
|-------|--------------------|----------|----|-----------|
| RG-059 | `test_BC_2_11_024_iin_integer_elements_rejected_e_query_001` | `crates/prism-query/src/tests/test_case_insensitive_operators.rs` | AC-021 extension | `severity IIN (42)` → Err(E-QUERY-001) parse-time — IIN list element is integer, not string (BC-2.11.024 v1.4 EC-11-024-012) |
| RG-060 | `test_BC_2_11_024_iin_boolean_elements_rejected_e_query_001` | `crates/prism-query/src/tests/test_case_insensitive_operators.rs` | AC-021 extension | `severity IIN (true)` → Err(E-QUERY-001) parse-time — IIN list element is boolean, not string (BC-2.11.024 v1.4 EC-11-024-012) |
| RG-061 | `test_BC_2_10_012_example_query_contains_no_comment_lines` | `crates/prism-mcp/src/tools/prism_describe.rs` (test module) | AC-025 | example_query field for every generated describe example contains no SQL comment lines (`--` or `/* */`) — pure parseable PQL invariant (BC-2.10.012 v1.9) |
| RG-062 | `test_BC_2_10_012_build_example_note_query_parses_without_stripping` | `crates/prism-mcp/src/tools/prism_describe.rs` (build_example_query_tests module) | AC-025 | example_query field passes through the PrismQL parser without stripping — confirms the field is valid standalone PQL (BC-2.10.012 v1.9 pure-PQL contract) |
| RG-063 | `test_BC_2_10_012_example_note_some_for_severity_tables_none_otherwise` | `crates/prism-mcp/src/tools/prism_describe.rs` (test module) | AC-025 | example_note is `Some("OCSF severity is stored as Title-case...")` for tables with a severity column; `None` for tables without a severity column — example_note field contract (BC-2.10.012 v1.9) |
| RG-064 | `test_BC_2_11_024_compound_violation_string_literal_precedence` | `crates/prism-query/src/tests/test_case_insensitive_operators.rs` | AC-023 | `SELECT * FROM t WHERE severity IEQ 42` — compound violation (non-string RHS + SQL-mode); string-literal `try_map` fires at literal span before mode-boundary walk; E-QUERY-001 string-literal template returned, not mode-boundary template (precedence lock per pass-13 OBS-2) |

**Pass-16 new tests (CI pre-flight guard tests — zero-row unregistered-table skip + empty CI-field list):**

| RG ID | Test Function Name | Location | AC | Assertion |
|-------|--------------------|----------|----|-----------|
| RG-065 | `test_check_ci_column_types_unregistered_table_ok` | `crates/prism-query/src/materialization.rs` (mod `check_ci_column_types_guard_tests`) | AC-022 guard | CI pre-flight returns Ok(()) for unregistered tables — zero-row fetches legitimately skip MemTable registration; documented skip, not fail-closed (pass-16 F-P16-OBS-002 adjudication) |
| RG-066 | `test_check_ci_column_types_empty_fields_ok` | `crates/prism-query/src/materialization.rs` (mod `check_ci_column_types_guard_tests`) | AC-022 guard | CI pre-flight returns Ok(()) for empty CI-field list |

**Pass-17 new tests (query-tool description post-normalization ENUM CASING CONTRACT guard):**

| RG ID | Test Function Name | Location | AC | Assertion |
|-------|--------------------|----------|----|-----------|
| RG-067 | `test_BC_2_10_009_query_tool_description_no_vendor_casing_teaches_ieq` | `crates/prism-mcp/tests/mcp_prism_describe.rs` | AC-025 | Query tool description contains no vendor-casing teaching patterns and mentions IEQ — post-normalization ENUM CASING CONTRACT (pass-17 F-HIGH-1) |

**Pass-18 new tests (PIPE MODE IEQ skeleton guard + severity-column-presence gate replaces allowlist):**

| RG ID | Test Function Name | Location | AC | Assertion |
|-------|--------------------|----------|----|-----------|
| RG-068 | `test_RG_067_query_tool_pipe_mode_example_uses_ieq_not_equals` | `crates/prism-mcp/src/server.rs` (adr_042_tests module) | AC-025/description guard | PIPE MODE skeleton uses IEQ not case-sensitive `=` (pass-18 OBS-1) |
| RG-069 | `test_obs2_sentinel_alerts_with_severity_gets_ieq_example` | `crates/prism-mcp/src/tools/prism_describe.rs` | AC-025 | UNKNOWN sensor prefix with severity String column gets IEQ example + note (allowlist gate replaced by column-presence rule; pass-18 OBS-2) |
| RG-070 | `test_obs2_claroty_devices_no_severity_column_still_no_ieq_note` | `crates/prism-mcp/src/tools/prism_describe.rs` | AC-025 guard | No-severity tables unaffected — IEQ example and casing note not emitted when no severity column present (pass-18 OBS-2) |

**Pass-19 fix-burst (eb7256b7) — RG-043/044 strengthened in-place (no new RGT names):**

RG-043/044 guards updated in-place: caption sets expanded to all four OCSF enum-label caption
sets (severity/status/activity_name/disposition); 3 flagship prompts corrected to
`status = 'New'`; crowdstrike SENSOR_COLUMN_VOCABULARIES `'new'`→`'New'`; reference IN
example re-pointed to `client_id IN ('acme','globex')`. No new RGT names added; RG-043/044
strengthened in place. (D-1589 carry-forward)

**Pass-20 new tests (PRIMARY-path GROUP BY cross-sensor guard + Integer-type IEQ suppression gate):**

| RG ID | Test Function Name | Location | AC | Assertion |
|-------|--------------------|----------|----|-----------|
| RG-071 | `test_BC_2_02_013_build_column_array_group_by_severity_cross_sensor_no_fragmentation` | `crates/prism-bin/src/spec_driven_adapter.rs` (test module) | AC-019 | PRIMARY-path cross-sensor GROUP BY invariant: 5 mixed-casing records (CrowdStrike 'High'×3 + Armis 'HIGH'×2) through build_column_array → DataFusion GROUP BY → exactly 1 bucket, value "High", count 5 (F-P20-HIGH-001; commit 345d4154) |
| RG-072 | `test_f_p20_low001_severity_integer_type_does_not_get_ieq` | `crates/prism-mcp/src/tools/prism_describe.rs` (test module) | AC-025 guard | Defensive gate: severity column with ColumnType::Integer must NOT receive the IEQ pedagogical example (F-P20-LOW-001; commit 257074af; gate tightened to name+ColumnType::String) |

**Pass-21 new tests (explain.rs predicate_to_exprs case_insensitive short-circuit guard):**

| RG ID | Test Function Name | Location | AC | Assertion |
|-------|--------------------|----------|----|-----------|
| RG-073 | `test_BC_2_11_024_f_p21_obs001_explain_ieq_iin_not_classified_pushdownable` | `crates/prism-query/src/explain.rs` (mod `predicate_explain_classification_tests`) | BC-2.11.024 invariant guard | `predicates_from_ast::predicate_to_exprs` short-circuits `case_insensitive: true` Compare/In predicates (emits no push-down-classifiable Expr), mirroring runtime `collect_equality_exprs` exclusion semantics so EXPLAIN can never report IEQ/IIN as push-downable when ColumnSpec is later wired into classification. Closes F-P21-OBS-001. Commit 2de85b18. Traces to BC-2.11.024 v1.4 (IEQ/IIN/INE never push down; DataFusion-local evaluation only). |

**Pass-24 new tests (valid_operators_for_type IEQ/IIN/INE coverage lock):**

| RG ID | Test Function Name | Location | AC | Assertion |
|-------|--------------------|----------|----|-----------|
| RG-074 | `test_BC_2_11_024_f_p24_med001_valid_operators_string_includes_ci_operators` | `crates/prism-query/src/engine.rs` (`#[cfg(test)] mod tests`) | BC-2.11.024 v1.4 | `valid_operators_for_type(ColumnType::String)` returns the full 8-operator set (=, !=, LIKE, IN, NOT IN, IEQ, IIN, INE); range operators excluded; negated IIN intentionally absent (not grammatically representable per ast.rs marker). Commit 633c5fab. RED→GREEN evidence: old 5-operator array. Sibling updates: `crates/prism-query/tests/e_query_pedagogical.rs` (required_string 5→8) and `crates/prism-mcp/tests/normalized_pql.rs` (comments); `error_mapping.rs` auto-tracks (derives dynamically). |

**Pre-PR-LEVEL fix-burst new tests (CR-002/CR-003/CR-004/SEC-001, commit 54c89898):**

| RG ID | Test Function Name | Location | AC | Assertion |
|-------|--------------------|----------|----|-----------|
| RG-075 | `test_cr002_normalize_enum_label_already_canonical_returns_some` | `crates/prism-ocsf/src/normalizer.rs` | AC-017 | CR-002 no-op write guard: `normalize_enum_label` returns `Some(value)` unchanged when the label already matches canonical casing — prevents unnecessary writes and double-normalization drift (idempotence BC-2.02.013 invariant). Commit 54c89898. |
| RG-076 | `test_cr003_normalize_predicate_invalid_ci_op_emits_warn` | `crates/prism-query/src/tests/test_case_insensitive_operators.rs` | AC-008 | CR-003 tracing::warn emitted before IEQ placeholder fallback on invalid `case_insensitive: true` + non-Eq/Ne operator combination — warn is observable and precedes the fallback path. Commit 54c89898. |
| RG-077 | `test_cr004_build_column_array_enum_label_warn_strips_control_chars` | `crates/prism-bin/src/spec_driven_adapter.rs` | AC-018 | CR-004/SEC-001 CWE-117 PRIMARY site: `sanitize_for_log` strips control characters (including `\n`, `\r`, `\t`, `\x00`) from the `value` field in the `ocsf.enum_label_unrecognized` warn payload before logging — prevents log injection. Commit 54c89898. |
| RG-078 | `test_cr004_sanitize_for_log_strips_control_chars_for_secondary_site` | `crates/prism-ocsf/src/normalizer.rs` | AC-018 | CR-004/SEC-001 CWE-117: asserts basic control-char stripping via direct call to `sanitize_enum_label_for_log` helper (no longer the SECONDARY-parity proof — that role superseded by RG-079; MED-002 re-pointed). Commit 54c89898. |

**Pass-24 fix-burst narrative (F-P24-LOW-001, no new RGT):**

F-P24-LOW-001: `crates/prism-mcp/tests/bc_2_10_016_audit_004_test.rs` — dead ALL-CAPS vocabulary entries (`'HIGH'`, `'CRITICAL'`, `'MEDIUM'`, `'LOW'`) removed from the armis_alerts.severity vocabulary table (dead data written before adapter normalization landed); stale `// prompt uses IIN` comment corrected to describe the current case-sensitive `IN ('High','Critical')` Title-case state. No new Red Gate test added — this is a test-file cleanup (dead data removal + comment correction).

**Pass-28 fix-burst narrative (669080f5, F-P28-MED-002 + F-P28-LOW-001, no new RGT):**

F-P28-MED-002: 217 versioned BC pins stripped from code comments across 25 delta files per TD-VSDD-091 / Architecture Compliance Rule 9 (including 47 stale BC-2.10.012 pins in `prism_describe.rs`); no behavioral changes. F-P28-LOW-001: `sql_parser.rs` §DML-Mode-Boundary citation corrected to §Mode-Boundary Enforcement (DML scope). Comment-only changes; 1407/1407 prism-query + 447/447 prism-mcp tests pass. No Red Gate test added or removed.

**Pass-32 fix-burst narrative (de89b557, F-P32-MED-001 + F-P32-OBS-001 + F-P32-OBS-002, no new RGT):**

Pass-32 fix-burst: F-P32-MED-001 BC-2.16.002 v2.04 (SqlPipe emission arm enumerated in pipe.sql_lowering/pipe.sql_planning_error rows); F-P32-OBS-001 numeric catalog-row comment pins → event-type-name citations (materialization.rs ×2, @de89b557); F-P32-OBS-002 IEQ/INE non-string RHS message enumeration dropped (not spec-pinned; IIN pinned message untouched). 1407/1407 prism-query.

**PR-LEVEL pass-1 fix-burst tests (ADV-PR-P1-MED-001/MED-002/LOW-001/OBS-002, commits 56fb83d8+f9be96fa):**

| RG ID | Test Function Name | Location | AC | Assertion |
|-------|--------------------|-----------|----|-----------|
| RG-079 | `test_rg079_secondary_sanitize_enum_label_order_spec_wins` | `crates/prism-ocsf/src/normalizer.rs` | AC-018 | sanitize-before-truncate order at SECONDARY site via load-bearing `sanitize_enum_label_for_log` helper — SECONDARY-parity proof (supersedes RG-078's parity role; ADV-PR-P1-MED-002 closure). Commits 56fb83d8+f9be96fa. |
| RG-080 | `test_rg080_low001_build_column_array_enum_label_warn_order_of_operations` | `crates/prism-bin/src/spec_driven_adapter.rs` | AC-018 | warn-capture order-of-operations at PRIMARY site — extended with `sensor_type` field mirror (WarnFieldVisitor/WarnFieldCapture rename): 65-codepoint ESC "B" control-char vector confirms sanitize fires BEFORE the 50-codepoint truncate cap for BOTH `value` and `sensor_type` fields, with 3 mirrored assertions (BC-2.16.002 v2.06 row 91 symmetry). Closures: ADV-PR-P1-LOW-001 + ADV-PR-P3-OBS-001. Commits 56fb83d8+f9be96fa+fab7df00. |
| RG-081 | `test_rg081_obs002_suggested_suffix_display_some_and_none` | `crates/prism-core/src/error.rs` | AC-022 | SuggestedSuffix Display direct lock — asserts both `Some("use column 'X' instead")` and `None` (no-suffix) arms produce the exact expected Display strings (ADV-PR-P1-OBS-002 closure). Commits 56fb83d8+f9be96fa. |

**PR-LEVEL pass-5 fix-burst tests (ADV-PR-P5-OBS-001→MED/OBS-002, commit 36a094d6):**

| RG ID | Test Function Name | Location | AC | Assertion |
|-------|--------------------|-----------|----|-----------|
| RG-082 | `test_sanitize_for_log_strips_unicode_cc_and_line_separators` | `crates/prism-core/src/error.rs` | AC-018 | Unicode Cc (U+0085 NEL, U+009B CSI) + U+2028/U+2029 stripped; "HÍGH" preserved (CWE-117 scope widening per BC-2.16.002 v2.06) |
| RG-083 | `test_check_ci_column_types_schema_provider_err_propagates` | `crates/prism-query/src/materialization.rs` (check_ci_column_types_guard_tests) | AC-022 | mock ErrSchemaProvider: schema lookup Err propagates as E-QUERY-034 QueryExecutionFailed instead of silently bypassing the AC-022 pre-flight |

RG-028 through RG-083 names are authoritative per verified ground truth.

**Total Red Gate tests: 83 (25 core + 2 discoverability + 9 pass-5 + 4 pass-7 + 2 pass-8 SqlPipe + 3 pass-9 + 2 pass-10 + 6 pass-11 + 1 pass-12 + 4 pass-14 + 6 pass-15 + 2 pass-16 + 1 pass-17 + 3 pass-18 + 2 pass-20 + 1 pass-21 + 1 pass-24 + 4 pre-PR-LEVEL-fix-burst + 3 PR-LEVEL-pass-1-fix-burst + 2 PR-LEVEL-pass-5-fix-burst)**

The story frontmatter records `red_gate_tests: 83` (updated from 81 at v1.36 for RG-082..083). RG-026/RG-027 discoverability tests may be
snapshot or integration tests; include them if they can be written as failing stubs, otherwise
verify the parity gate via `just check`. RG-028 is the authoritative describe test. RG-032
through RG-036 are the PRIMARY build_column_array tests in prism-bin.

---

## Architecture Mapping

| Component | Crate | Files | Pure/Effectful |
|-----------|-------|-------|---------------|
| Grammar (IEQ/IIN/INE keywords) | prism-query | `src/filter_parser.rs` | Pure |
| AST extensions (case_insensitive flag) | prism-query | `src/ast.rs` | Pure |
| DataFusion SQL emitter (lower() lowering) | prism-query | `src/pipe_sql_emitter.rs` | Pure |
| PQL round-trip normalizer | prism-query | `src/ast.rs` — `PqlNormalizer::normalize_predicate` (the `Predicate::Compare` and `Predicate::In` match arms) | Pure |
| OCSF enum-label canonical-case normalization (PRIMARY) | prism-bin | `src/spec_driven_adapter.rs` (`build_column_array`) | Effectful (materialization into Arrow StringArray) |
| OCSF enum-label canonical-case normalization (SECONDARY) | prism-ocsf | `src/enum_map.rs` (`OcsfEnumMap`) + `src/normalizer.rs` (`OcsfNormalizer::normalize_with_mappers`) and/or `src/mappers/spec_driven.rs` | Pure (lookup-and-rewrite, no I/O) |
| Grammar resource MCP resource | prism-mcp | `src/resources.rs` (or grammar generation code) | Effectful (MCP resource emission) |
| prism describe pedagogical examples | prism-query / prism-mcp | grammar describe output | Effectful (MCP tool response) |

**Architecture section references:**
- `architecture/decisions/ADR-047` §D.2 (IEQ/IIN/INE syntax, lower() lowering rationale)
- `architecture/decisions/ADR-047` §D.3 (adapter normalization, enum_map.rs authority)
- `architecture/module-decomposition.md` §SS-11 (Query Execution — prism-query)
- `architecture/module-decomposition.md` §SS-02 (OCSF Normalization — prism-ocsf)

**Pass-11 single-source constant:** `OCSF_ENUM_LABEL_FIELDS` is now declared as `pub const` in `prism-ocsf` (pass-11 F-OBS-3 drift-proof). `prism-bin/src/spec_driven_adapter.rs` imports it from `prism-ocsf` rather than defining it locally — eliminates enum-label field list divergence between the PRIMARY and SECONDARY normalization sites.

**Pass-16 single-access-point:** `shared_enum_map()` is now the single `OcsfEnumMap` access point (`pub fn` in `prism-ocsf`, used by `prism-bin`; duplicate statics removed — pass-16 F-P16-OBS-001).

---

## Edge Case Catalog

| EC ID | Source | Description | Expected Behavior |
|-------|--------|-------------|-------------------|
| EC-001 | BC-2.11.024 EC-11-024-001 | `severity IEQ 'HIGH'` against stored `'High'` | Match — `lower('HIGH') = lower('High')` → both `'high'` |
| EC-002 | BC-2.11.024 EC-11-024-002 | `severity IEQ 'high'` against stored `'High'` | Match — same |
| EC-003 | BC-2.11.024 EC-11-024-003 | `status IIN ('open', 'NEW')` against `'Open'` and `'New'` | Both match |
| EC-004 | BC-2.11.024 EC-11-024-004 | `hostname IEQ 'server-01.corp'` (non-enum free-form field) | Valid — IEQ works on any string column |
| EC-005 | BC-2.11.024 EC-11-024-005 | `severity = 'High'` AND `severity IEQ 'high'` in same query | Both valid; = requires exact `'High'`, IEQ matches any casing |
| EC-006 | BC-2.11.024 EC-11-024-006 | Operator spelled lowercase: `severity ieq 'high'` | Valid — kw() combinator parses case-insensitively |
| EC-007 | BC-2.11.024 EC-11-024-007 | `severity IIN ('high')` (single-value IIN) | Valid — equivalent to `severity IEQ 'high'` |
| EC-008 | BC-2.11.024 EC-11-024-008 | `severity IEQ` on null-valued column | No match — `lower(null)` → null; null comparisons require IS NULL/IS NOT NULL |
| EC-009 | BC-2.11.024 EC-11-024-009 | `severity IEQ ''` (empty string literal) | Valid parse; matches records where `lower(severity) = ''` — typically zero rows |
| EC-010 | BC-2.02.013 EC-02-020 | CrowdStrike `severity='High'` (already canonical) | Idempotent — unchanged; no warning |
| EC-011 | BC-2.02.013 EC-02-021 | Armis `severity='UNHANDLED'` (vendor-specific, not OCSF) | Left as-received; warning logged; queryable via `IEQ` |
| EC-012 | BC-2.02.013 EC-02-022 | Claroty `status='Unresolved'` (vendor-specific, not OCSF caption) | Left as-received; warning logged |
| EC-013 | BC-2.02.013 EC-02-023 | Sensor emits `severity='high'` (all-lowercase) | Normalized to `'High'` (canonical Title-case) |
| EC-014 | BC-2.02.013 EC-02-024 | Sensor emits `severity='CRITICAL'` (all-caps) | Normalized to `'Critical'` |
| EC-015 | BC-2.02.013 EC-02-025 | Field value is null | Null passes through unchanged — not an enum label |
| EC-016 | BC-2.02.013 EC-02-026 | GROUP BY severity across CrowdStrike (Title-case) + Armis (UPPER, pre-norm) after normalization | Both sensors produce `'High'` bucket — no fragmentation |
| EC-017 | ADR-047 §D.1 | Existing query `severity = 'High'` after adapter normalization | Returns rows from ALL sensors since all now store `'High'` |
| EC-018 | Design map §A | `IIN` appearing after `IN` in parser alternatives | If implemented with IN-before-IIN, `status IIN ('x')` fails at parse time — see AC-005 and risk_mitigations |

---

## File Structure Requirements

### Files to MODIFY (existing):

| File | Change Summary |
|------|---------------|
| `crates/prism-query/src/filter_parser.rs` | Add `IEQ`, `IIN`, `INE` keyword alternatives; `IIN` must appear BEFORE `IN` in alternative chain |
| `crates/prism-query/src/ast.rs` | Add `case_insensitive: bool` to `Predicate::Compare` and `Predicate::In`; extend round-trip normalizer to emit IEQ/IIN/INE |
| `crates/prism-query/src/pipe_sql_emitter.rs` | Add `case_insensitive: true` branches emitting `lower(field) OP lower('val')` in `predicate_to_datafusion_sql` |
| `crates/prism-ocsf/src/enum_map.rs` | Verify `OcsfEnumMap` canonical caption map covers severity, status, activity_name, disposition, category; extend if missing entries. This is the sole casing authority (BC-2.02.010 v1.5). NOTE: the OCSF string label for activity is `activity_name` (not `activity`). |
| `crates/prism-bin/src/spec_driven_adapter.rs` | PRIMARY insertion point (architect adjudication F-CRIT-002): add the canonical-case rewrite in `build_column_array` for `ColumnType::String` enum-label columns (severity, status, activity_name, disposition) BEFORE the Arrow `StringArray` cell is materialized. Call `OcsfEnumMap::normalize_enum_label` (or equivalent); emit `tracing::warn!(event_type = "ocsf.enum_label_unrecognized", ...)` for unrecognized values per BC-2.02.013 v1.9 §Postconditions. |
| `crates/prism-ocsf/src/normalizer.rs` and/or `crates/prism-ocsf/src/mappers/spec_driven.rs` | SECONDARY insertion point: add the canonical-case rewrite for the `OcsfNormalizer::normalize_with_mappers` DynamicMessage path (protobuf/future). Same contract as PRIMARY — both insertion sites must satisfy BC-2.02.013 v1.9 independently. |
| `crates/prism-spec-engine/src/` | RE-ADDED to `crates_touched` (pass-20; F-P20-MEDIUM-002): comment-only TD-VSDD-091 anti-volatile-pin sweep (pass-8 fix-burst 0b2c0983: stale 'BC-2.16.002 vN.NN' version pins removed from doc comments); zero code changes. Orchestrator adjudicated: legitimate in-scope sibling-sweep, kept in diff. |
| `crates/prism-mcp/src/resources.rs` (or equiv.) | Add IEQ/IIN/INE to grammar reference resource operator table; add OCSF casing note to prism describe examples |
| `crates/prism-core/src/error.rs` | Add `SuggestedSuffix` helper + `suggested_column: Option<&'static str>` field to `PrismError::QueryTypeMismatch`; update Display impl to emit the "for label comparison, use the string column '...' with IEQ/IIN/INE instead" suffix when `suggested_column` is `Some`. Required by AC-022 E-QUERY-002 contract; RG-029/030 test this file directly. |
| DTU fixture generators (prism-dtu-*/src/generator.rs or types.rs) | Update any test assertions or fixture values that assert pre-normalization UPPER-case severity/status strings to use canonical Title-case. Per SAP-2, adversary will verify TOML column parity with DTU types.rs |

### Files to CREATE:

| File | Purpose |
|------|---------|
| `crates/prism-query/src/tests/test_case_insensitive_operators.rs` (or equivalent test module) | Red Gate test file for RG-001 through RG-018, RG-023 |
| `crates/prism-ocsf/src/tests/test_adapter_normalization.rs` (or equivalent) | Red Gate tests RG-019, RG-020, RG-021 for adapter normalization |
| `crates/prism-core/tests/bc_2_11_024_query_type_mismatch_display.rs` | Red Gate tests RG-029 (`test_BC_2_11_024_query_type_mismatch_display_with_suggestion_exact`) and RG-030 (`test_BC_2_11_024_query_type_mismatch_display_without_suggestion_exact`) — byte-exact POL-24 Display assertions for `PrismError::QueryTypeMismatch` WITH suggested_column (sub-form b1) and WITHOUT (sub-form b2); referenced by AC-022 and cited in the pass-5 RG inventory rows. |

NOTE: The implementer must verify the exact test file locations by reading the existing test
organization in prism-query and prism-ocsf before creating new files.

---

## Library & Framework Requirements

All versions are locked to the workspace `Cargo.lock`. Do NOT upgrade any dependencies as
part of this story.

| Library | Version | Usage |
|---------|---------|-------|
| `chumsky` | 0.12.0 (Cargo.lock) | Chumsky parser combinators — `kw()` (project-local helper in `filter_parser.rs`, not a chumsky-public API) for keyword parsing, combinator ordering for IIN-before-IN |
| `datafusion` | 53.1.0 (Cargo.lock) | `lower()` SQL scalar string function — already used in the `ICONTAINS` path (`pipe_sql_emitter.rs` `lower(field) LIKE lower('%pat%')`); no new DataFusion API surface needed |
| `arrow` | 58.2.0 (Cargo.lock) | Array types for any new test fixtures. NOTE: DataFusion 53.1 pulls arrow 58 transitively (see `prism-query/Cargo.toml` — `arrow = "58"` pinned for explicit awareness); arrow does NOT track DataFusion's 53.x line |
| `chrono` | 0.4.44 (Cargo.lock) | Utility in prism-ocsf if date parsing is needed for enum_map lookup helpers |

**DataFusion lowering rationale (from ADR-047 §Alternatives Alt-2/Alt-3):**
- `lower(field) OP lower('val')` is the chosen mechanism — already proven for ICONTAINS
- `ILIKE` is rejected — pattern-matching metacharacters (`%`, `_`) are semantically wrong for exact equality
- `COLLATE` is rejected — DataFusion does not implement ANSI COLLATE for case-insensitive matching
- No new DataFusion version dependency introduced by this story

**rustls-tls (ADR-050):** This story does not add new HTTP clients. No new `reqwest` dependencies.

---

## Tasks

TDD implementation order follows the dependency graph from design map §Dependency Order:
B (AST) → A (Grammar) → C (Emitter) → D (Normalizer) → F (Discoverability)
E (Adapter normalization) is parallel to A-D.

### Phase 0 — Read grounding artifacts

1. **Read** ADR-047 in full: `.factory/specs/architecture/decisions/ADR-047-prismql-case-sensitivity-policy-ieq-iin-and-adapter-boundary-normalization.md`
   Focus: §D.1 (case-sensitive default), §D.2 (IEQ/IIN/INE + lower() lowering), §D.3 (adapter normalization), §Consequences, §Alternatives (Alt-2 ILIKE rejected, Alt-4 =~ collision).

2. **Read** the design map: `.factory/specs/architecture/prismql-case-insensitive-design-map.md`
   Focus: §A (grammar changes), §B (AST changes), §C (emitter changes), §D (round-trip normalizer),
   §E (adapter normalization), §G (Red Gate tests T-CASE-001 through T-CASE-010).

3. **Read** BC-2.11.024: `.factory/specs/behavioral-contracts/BC-2.11.024-prismql-ieq-iin-ine-case-insensitive-operators.md` — all postconditions, invariants, and error cases.

4. **Read** BC-2.02.013: `.factory/specs/behavioral-contracts/BC-2.02.013-ocsf-enum-label-canonical-case-normalization.md` — all postconditions, invariants, and edge cases.

5. **Read** `crates/prism-query/src/filter_parser.rs` focusing on:
   - Lines 985-993: existing ICONTAINS/ISTARTSWITH/IENDSWITH `kw()` block (model to follow for IEQ/IIN/INE)
   - The `IN` keyword parser location (find where `IN` is defined to insert `IIN` BEFORE it)
   - The `=` and `!=` comparison operator parsers

6. **Read** `crates/prism-query/src/ast.rs` focusing on (cite by anchor, not line number —
   the prior line numbers in this story were stale; TD-VSDD-091):
   - `pub enum Predicate` — the `Compare { lhs, op, rhs }` variant and the `In { field, values, negated }` variant (the enum is `#[non_exhaustive]`)
   - The `Expr` enum (`Expr::Field`, `Expr::Literal`, `Expr::VirtualField`) — the `lhs`/`rhs` payload types
   - `PqlNormalizer::normalize_predicate` — the round-trip normalizer's `Predicate::Compare` / `Predicate::In` arms (models the existing `StringOp` ICONTAINS→"ICONTAINS" uppercase-canonical emission)

7. **Read** `crates/prism-query/src/pipe_sql_emitter.rs` focusing on:
   - `predicate_to_datafusion_sql` (~line 506)
   - Existing CI lowering for ICONTAINS (~lines 541-564, the model to follow)
   - The op table mapping CompareOp variants to SQL strings (~lines 743-749)

8. **Read** `crates/prism-bin/src/spec_driven_adapter.rs` and `crates/prism-ocsf/src/` to ground
   yourself in both insertion sites (architect adjudication F-CRIT-002 — see "Scope Ambiguities
   Resolved → Ambiguity 1"):
   - `crates/prism-bin/src/spec_driven_adapter.rs` — the PRIMARY insertion point. Read
     `build_column_array`: this is where sensor JSON records are materialized into Arrow arrays.
     For `ColumnType::String` columns matching OCSF enum-label field names (severity, status,
     activity_name, disposition), add the canonical-case lookup BEFORE the `StringArray` cell
     is materialized. This is the main production path.
   - `crates/prism-ocsf/src/enum_map.rs` — `OcsfEnumMap`, the canonical caption authority used
     by BOTH insertion sites.
   - `crates/prism-ocsf/src/normalizer.rs` — `OcsfNormalizer::normalize_with_mappers`, which
     creates the `DynamicMessage` (BC-2.02.002); this is the SECONDARY insertion site.
   - `crates/prism-ocsf/src/mappers/spec_driven.rs` — the config-driven sensor mapper.
   Do NOT search prism-spec-engine for the normalizer — it does not create `DynamicMessage`
   (verified: zero `DynamicMessage` references; removed from `crates_touched`).

### Phase 1 — Write all Red Gate test stubs (FAILING first)

9. **Write Red Gate stubs RG-001 through RG-027** as `todo!()` stubs — all must FAIL before any
   production code is written. Organize into:

   a. **Parser/AST tests** (RG-001..005): add to a new test module in `crates/prism-query/src/tests/`
      or an existing test file if a pattern exists. Test functions per the Red Gate Inventory above.

   b. **Emitter tests** (RG-006..009): add to the emitter test module in `crates/prism-query/src/`
      or `crates/prism-query/src/tests/`. Call `predicate_to_datafusion_sql` directly.

   c. **Execution tests** (RG-010..015): use DataFusion MemTable setup. RG-010 and RG-011 require
      a simple table with a `severity` string column.

   d. **Error case tests** (RG-016..018): verify parse errors (RG-016, RG-017) and type-mismatch
      error (RG-018).

   e. **Adapter normalization stubs** (RG-019..021): in `crates/prism-ocsf/src/tests/` or the
      adapter pipeline test module — determined at step 8.

   f. **Aggregation test** (RG-022): MemTable with multi-sensor simulated data + GROUP BY.

   g. **SQL-mode rejection stubs** (RG-023..025): in `crates/prism-query/src/tests/` — three stubs
      asserting Err(E-QUERY-001) for IEQ/IIN/INE in SELECT-prefixed queries.

   h. **Grammar resource / describe stubs** (RG-026..027): placeholder stubs in appropriate test modules.

   **After writing all stubs:** Run `cargo nextest run -p prism-query --no-fail-fast` and
   `cargo nextest run -p prism-ocsf --no-fail-fast` to confirm all new test functions are
   present and FAILING (todo!() panic or compile error). Do NOT proceed to Phase 2 until
   all stubs fail.

### Phase 2 — AST changes (Track B)

10. **Modify** `crates/prism-query/src/ast.rs` — add `case_insensitive: bool` to `Predicate::Compare`
    and `Predicate::In`:

    VERIFIED ACTUAL SHAPE (from `ast.rs` `pub enum Predicate`, which is `#[non_exhaustive]`):
    the existing variants use `lhs`/`rhs` (`Box<Expr>`) for `Compare` and `field: FieldPath` +
    `values: Vec<Literal>` for `In`. Add ONLY the new `case_insensitive: bool` field to each —
    do NOT rename or restructure the existing fields.

    ```rust
    // In Predicate::Compare variant (EXISTING fields lhs/op/rhs unchanged; add case_insensitive):
    Predicate::Compare {
        lhs: Box<Expr>,
        op: CompareOp,
        rhs: Box<Expr>,
        case_insensitive: bool,  // NEW: default false for existing callers
    }

    // In Predicate::In variant (EXISTING fields field/values/negated unchanged; add case_insensitive):
    Predicate::In {
        field: FieldPath,
        values: Vec<Literal>,
        negated: bool,
        case_insensitive: bool,  // NEW: default false for existing callers
    }
    ```

    After this change, all construction sites (`Predicate::Compare { ... }` and `Predicate::In { ... }`)
    will produce compiler errors. DO NOT FIX THEM YET — let the compiler surface all sites.

11. **Sibling-site sweep (TD-VSDD-060):**

    Run in `crates/prism-query/src/`:
    ```bash
    rg 'Predicate::Compare\s*\{' crates/prism-query/src/ --type rust
    rg 'Predicate::In\s*\{' crates/prism-query/src/ --type rust
    ```

    For EVERY construction site found (excluding the new IEQ/INE/IIN parse sites), add
    `case_insensitive: false`. This ensures all existing behavior is preserved with the
    explicit false default. Run in the broader workspace too:
    ```bash
    rg 'Predicate::Compare\s*\{' crates/ --type rust
    rg 'Predicate::In\s*\{' crates/ --type rust
    ```
    Fix all sites across ALL crates that use the `prism-query` AST types.

12. **Add `case_insensitive` handling to the round-trip normalizer** in `ast.rs` —
    `PqlNormalizer::normalize_predicate`, `Predicate::Compare` and `Predicate::In` arms.
    The normalizer must emit `IEQ`, `IIN`, `INE` when `case_insensitive: true`:

    | AST state | Normalized PQL output |
    |-----------|----------------------|
    | `Compare { op: Eq, case_insensitive: false }` | `field = 'value'` (unchanged) |
    | `Compare { op: Eq, case_insensitive: true }` | `field IEQ 'value'` |
    | `Compare { op: Ne, case_insensitive: false }` | `field != 'value'` (unchanged) |
    | `Compare { op: Ne, case_insensitive: true }` | `field INE 'value'` |
    | `In { case_insensitive: false }` | `field IN ('v1', 'v2')` (unchanged) |
    | `In { case_insensitive: true }` | `field IIN ('v1', 'v2')` |

    IMPORTANT: The normalizer must emit UPPERCASE canonical operator names (IEQ, not ieq).
    This is required by BC-2.11.024 postcondition (normalized_pql reflects uppercase canonical form)
    and BC-2.11.018 v1.3 amendment EC-11-057.

### Phase 3 — Grammar changes (Track A)

13. **Modify** `crates/prism-query/src/filter_parser.rs` — add IEQ/IIN/INE keyword alternatives.

    In the comparison operator block:

    ```
    // Near the existing kw("IN") combinator:
    // ADD IIN BEFORE IN (longest match first — critical for collision avoidance):
    kw("IIN") => ... (case_insensitive: true on Predicate::In)
    kw("IN") => ... (existing, unchanged)

    // Near = and != operators, add IEQ and INE:
    kw("IEQ") => ... (case_insensitive: true on Predicate::Compare, op: Eq)
    kw("INE") => ... (case_insensitive: true on Predicate::Compare, op: Ne)
    ```

    The `kw()` combinator already handles case-insensitive keyword matching — `ieq`, `IEQ`, `Ieq`
    all parse via the same `kw("IEQ")` combinator. No additional case-folding needed.

    **Verify IIN-before-IN ordering** by reading the parser combinator chain. If the parser
    uses a different mechanism (e.g., `text::keyword` with exact-match followed by whitespace),
    verify the ordering works. Run RG-005 to confirm IIN parses correctly.

14. **After grammar changes:** Run `cargo nextest run -p prism-query -E 'test(ieq_parses)'`
    (or the RG-001 test name). If RG-001 through RG-005 now pass, grammar changes are correct.
    If any parser tests fail with unexpected parse errors, debug the combinator ordering.

14b. **Implement SQL-mode rejection gate (AC-023 / BC-2.11.024 v1.4 §Mode-Boundary Enforcement):**

    In the prism-query parser entry point (the top-level dispatch that determines whether the
    input is a filter-mode expression, a pipe query, or a raw SQL SELECT), add a check:
    if the query begins with `SELECT` AND the query contains any IEQ/IIN/INE keyword, return
    `E-QUERY-001` at PARSE TIME with the verbatim message from BC-2.11.024 v1.4:
    ```
    "E-QUERY-001: parse error near '{operator}': case-insensitive operators (IEQ/IIN/INE)
    are not supported in SQL mode. Use filter mode (e.g., severity IEQ 'high') or a pipe
    | where stage (e.g., FROM crowdstrike_detections | where severity IEQ 'high') instead."
    ```
    where `{operator}` is the first IEQ/IIN/INE keyword encountered (uppercased).

    **DML rejection (BC-2.11.024 v1.4 scope extension):** The same keyword scan MUST also
    run inside `parse_sql_dml_with_limits` BEFORE the DML statement is dispatched to DataFusion.
    Add the scan at the top of that function and add a code comment:
    ```rust
    // IEQ/IIN/INE SQL-mode guard: reject before DML dispatch per BC-2.11.024 §Mode-Boundary Enforcement
    ```
    This ensures that `DELETE`/`UPDATE` WHERE clauses and `INSERT ... SELECT` containing
    IEQ/IIN/INE are rejected at parse time, not at DataFusion planning time.

    The rejection MUST occur before DataFusion planning/execution to guarantee `E-QUERY-001`
    is returned and NOT `E-QUERY-034 QueryExecutionFailed`.

    After implementation, verify SELECT RG-023/024/025 AND DML RG-037/038 pass; confirm
    AC-001/AC-013b do NOT regress:
    ```bash
    cargo nextest run -p prism-query -E 'test(sql_mode)'
    cargo nextest run -p prism-query -E 'test(dml_delete_where)'
    cargo nextest run -p prism-query -E 'test(dml_update_where)'
    cargo nextest run -p prism-query -E 'test(ieq_parses)'
    ```

### Phase 4 — Emitter changes (Track C)

15. **Modify** `crates/prism-query/src/pipe_sql_emitter.rs` in `predicate_to_datafusion_sql`:

    Add branches for `case_insensitive: true` before the existing case-sensitive branches:

    The `Predicate::Compare` arm already destructures `{ lhs, op, rhs }` and computes
    `lhs_sql = expr_to_sql(lhs)?` and `rhs_sql = expr_to_sql(rhs)?`. The `Predicate::In` arm
    destructures `{ field, values, negated }` and computes `field_sql = field_path_to_sql(field)`
    and a value list via `literal_to_sql`. Add `case_insensitive` to each destructure and branch
    on it — reuse the EXISTING `lhs_sql`/`rhs_sql`/`field_sql` bindings; do NOT invent `field` /
    `value` / `escape_value` (no such variables/fn on these paths — literal escaping is handled
    inside `literal_to_sql` / `expr_to_sql`).

    ```rust
    // For Predicate::Compare { lhs, op, rhs, case_insensitive }:
    let lhs_sql = expr_to_sql(lhs)?;
    let rhs_sql = expr_to_sql(rhs)?;
    if *case_insensitive {
        match op {
            CompareOp::Eq => Ok(format!("lower({lhs_sql}) = lower({rhs_sql})")),
            CompareOp::Ne => Ok(format!("lower({lhs_sql}) != lower({rhs_sql})")),
            _ => { /* Other ops (LT, GT, etc.) — case_insensitive not applicable;
                     grammar only produces IEQ→Eq and INE→Ne, so this arm is unreachable
                     for parser-produced ASTs. Fall through to the existing op_str path. */ }
        }
    } else {
        // Existing case-sensitive path (unchanged): `{lhs_sql} {op_str} {rhs_sql}`
    }

    // For Predicate::In { field, values, negated, case_insensitive }:
    let field_sql = field_path_to_sql(field);
    if *case_insensitive {
        let lowered_values: Vec<String> = values.iter()
            .map(|v| Ok(format!("lower({})", literal_to_sql(v)?)))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(format!("lower({field_sql}) IN ({})", lowered_values.join(", ")))
        // NOTE: negated + case_insensitive (IIN has no negated form in the grammar) is not
        // parser-producible; keep the existing NOT IN handling for negated case-sensitive IN.
    } else {
        // Existing case-sensitive path (unchanged)
    }
    ```

    NOTE: The existing `predicate_to_datafusion_sql` for ICONTAINS uses `lower(field) LIKE lower('%pat%')`.
    The IEQ/IIN pattern is `lower(field) = lower('val')`. Same idiom, different operators.

    **Verify the op table at ~lines 743-749** is not inadvertently bypassed for the CI path.
    If the emitter uses the op table to produce `=`/`!=` strings and then applies wrapping,
    ensure the CI branches wrap the complete expression correctly.

16. **After emitter changes:** Run `cargo nextest run -p prism-query -E 'test(emits_lower)'`
    to verify RG-006, RG-007, RG-008, RG-009 pass.

### Phase 5 — DataFusion execution tests (Track C integration)

17. **Verify** RG-010 (IEQ execution), RG-011 (= regression), RG-012 (pipe-mode IEQ), RG-015 (no panic).
    These require a DataFusion MemTable with a `severity: Utf8` column.

    If any execution test fails: read the DataFusion plan output to verify `lower()` is being
    applied correctly. Common issues: value escaping, column quoting, DataFusion version of `lower()`.

### Phase 6 — Round-trip normalizer tests (Track D)

18. **Verify** RG-013 (normalized_pql contains IEQ uppercase) and RG-014 (round-trip AST equality).
    Both should pass after Task 12 (normalizer update).

### Phase 7 — Adapter normalization (Track E)

19. **Implement** the OCSF enum-label canonical-case normalization in BOTH insertion sites
    (architect adjudication F-CRIT-002 — see Task 8 / Scope Ambiguity 1):

    **Step 19a — PRIMARY insertion site: `build_column_array` in prism-bin**

    a. **Read** `crates/prism-bin/src/spec_driven_adapter.rs` — locate `build_column_array`.
       Identify where `ColumnType::String` values are extracted from the JSON sensor record and
       materialized into the Arrow `StringArray`.

    b. **Read** `crates/prism-ocsf/src/enum_map.rs` to understand the caption map structure
       (e.g., `severity_id: 4 → "High"`, `5 → "Critical"`). This is used by both sites.

    c. **Implement** the normalization in `build_column_array`: for each String column whose name
       matches an OCSF enum-label field (severity, status, activity_name, disposition):

    ```rust
    /// Normalizes an OCSF enum-label string field value to its canonical Title-case
    /// OCSF casing, as defined by enum_map.rs. Idempotent: already-canonical values
    /// are returned unchanged.
    ///
    /// Returns:
    ///   - Some(canonical_value) if the value is found in enum_map.rs (case-insensitive lookup)
    ///   - None if the value is not recognized (caller should leave as-received + warn)
    fn normalize_ocsf_enum_label(field_name: &str, value: &str, enum_map: &OcsfEnumMap) -> Option<String> {
        // Build a case-folded lookup from enum_map captions
        // e.g., "high" → "High", "HIGH" → "High", "CRITICAL" → "Critical"
        enum_map.lookup_case_insensitive(field_name, value)  // method name ILLUSTRATIVE
    }
    ```
    NOTE: `lookup_case_insensitive` is an ILLUSTRATIVE method name — read `enum_map.rs` at
    Task 19b to use (or add, with a Red Gate test) the real lookup API; do not assume this exact
    signature exists.

    d. **Emit** `tracing::warn!(event_type = "ocsf.enum_label_unrecognized", field_name = ...,
       value = ..., sensor_type = ...)` when the lookup returns None (AC-018, BC-2.02.013 v1.9
       error case). Per SAP-1: add a corresponding row to BC-2.16.002 Canonical Structured Event
       Catalog for this `event_type` in the SAME commit that adds the tracing emission.

    e. **Verify** RG-032, RG-033, RG-034, RG-035, RG-036 (PRIMARY build_column_array tests) pass.

    **Step 19b — SECONDARY insertion site: `normalize_with_mappers` in prism-ocsf**

    f. **Wire** the same normalization function into `crates/prism-ocsf/src/normalizer.rs`
       (`OcsfNormalizer::normalize_with_mappers`) BEFORE the `DynamicMessage` field is populated
       (BC-2.02.002 v1.5 amendment + BC-2.02.013 v1.9 invariant). This satisfies the
       DynamicMessage/protobuf path independently.

    g. **Verify** RG-019, RG-020, RG-021 (SECONDARY DynamicMessage path tests) pass.

20. **Update DTU test vectors** that assert pre-normalization UPPER-case severity/status values.
    Per SAP-2: run `rg 'severity.*HIGH\|status.*OPEN' crates/prism-dtu-*/src/ --type rust`
    and update all assertions to use canonical Title-case form:
    - `severity == "HIGH"` → `severity == "High"`
    - `status == "OPEN"` → `status == "Open"` (if status 'OPEN' maps to OCSF canonical 'Open')
    - `severity == "UNHANDLED"` → remains `"UNHANDLED"` (vendor-specific, not normalized)

21. **Verify** RG-022 (GROUP BY no fragmentation) passes — this verifies the PRIMARY insertion
    point's cross-sensor aggregation consistency (AC-019).

### Phase 8 — Discoverability (Track F)

22. **Add** IEQ/IIN/INE to the auto-generated PrismQL grammar reference MCP resource.
    Read the grammar resource generation code (BC-2.11.022 / ADR-045) and add the three
    new operators to the operator table. Run the ADR-045 parity gate:
    ```bash
    just check  # includes ADR-045 parity gate
    ```

23. **Add** IEQ example with OCSF casing note to `prism describe` output.
    Per BC-2.10.012 v1.9, `example_query` MUST be pure parseable PQL — no `--` comment lines
    or any other non-PQL content. The pedagogical OCSF casing note belongs in the separate
    `example_note` field (`Option<String>`). The correct output structure is:

    ```
    example_query: "FROM crowdstrike_detections | where severity IEQ 'high' | limit 50"
    example_note:  "OCSF severity is stored as Title-case ('High'). Use IEQ/IIN to match
                    regardless of the case you type, or = 'High' for the exact canonical form."
    ```

    Do NOT embed the OCSF casing text as a SQL comment inside `example_query` — that would
    violate the BC-2.10.012 v1.9 pure-PQL invariant and cause RG-061/RG-062 to fail.

24. **Verify** RG-026 (grammar resource), RG-027 (describe supplementary), and RG-028
    (describe authoritative: `test_BC_2_11_024_describe_output_includes_ieq_example_and_ocsf_casing_note`
    in `prism_describe.rs`) pass.

### Phase 9 — Final verification

25. **Non-exhaustive gate check:**
    ```bash
    grep EXPECTED= scripts/check-non-exhaustive.sh
    ```
    Must show `EXPECTED=89`. If any new `#[non_exhaustive]` type was introduced, update
    both `scripts/check-non-exhaustive.sh` (or `ci.yml`) AND this AC-027 with the new count.
    Per design map §Non-Exhaustive Gate Impact Assessment: only field additions, not new
    annotated types — count should remain 89.

26. **Sibling-site sweep final verification (TD-VSDD-060):**
    ```bash
    rg 'Predicate::Compare\s*\{' crates/ --type rust
    rg 'Predicate::In\s*\{' crates/ --type rust
    ```
    Every construction site must have `case_insensitive: false` (or `true` for the new IEQ/INE/IIN parsers).
    Zero unexplained omissions.

27. **SAP-1 standing probe:** Verify any new `tracing::warn!(event_type = ...)` emission added
    by the adapter normalization (step 19d) has a corresponding row in BC-2.16.002 Canonical
    Structured Event Catalog. Run: `rg 'event_type.*ocsf.enum' crates/ --type rust` to find
    the emission and cross-reference the BC.

28. **Run full test suite:**
    ```bash
    just iter prism-query       # IEQ/IIN/INE grammar, AST, emitter, execution tests
    just iter prism-ocsf        # SECONDARY adapter normalization (normalize_with_mappers path)
    just iter prism-bin         # PRIMARY adapter normalization (build_column_array path)
    just iter prism-mcp         # describe authoritative test (RG-028)
    just check                  # full workspace pre-push gate
    ```
    All 83 Red Gate tests must pass (25 core + 2 discoverability + 9 pass-5 + 4 pass-7 + 2 pass-8 SqlPipe + 3 pass-9 + 2 pass-10 + 6 pass-11 + 1 pass-12 + 4 pass-14 + 6 pass-15 + 2 pass-16 + 1 pass-17 + 3 pass-18 + 2 pass-20 + 1 pass-21 + 1 pass-24 + 4 pre-PR-LEVEL-fix-burst + 3 PR-LEVEL-pass-1-fix-burst + 2 PR-LEVEL-pass-5-fix-burst).
    All existing tests must continue to pass (especially existing =, != filter tests — regression
    guard for AC-011/AC-013).

---

## Previous Story Intelligence

### From S-DEMO-FIDELITY-REMEDIATION-001 (MERGED develop@ea714d14, base for this story)

- **SAP-2 probe is ACTIVE:** The adversary will run the SAP-2 probe on any TOML sensor spec
  changes. This story's adapter normalization will break DTU assertions that check raw
  UPPER-case severity values. Those failures are INTENTIONAL — update the test vectors
  to expect canonical Title-case. Before declaring done, grep DTU test fixtures for
  any uppercase severity/status assertions.

- **E-QUERY error codes already defined:** This story uses E-QUERY-001 (parse error) and
  E-QUERY-002 (QueryTypeMismatch) — both already in the error taxonomy. No new E-QUERY codes
  are introduced. Do not invent codes outside the taxonomy.

- **crates_touched declaration (UPDATED pass-20):** FIDELITY-REMEDIATION established the pattern of
  declaring `crates_touched` explicitly. This story touches prism-query, prism-ocsf, prism-mcp,
  prism-bin (PRIMARY OCSF normalization insertion point), prism-core, and prism-spec-engine
  (comment-only TD-VSDD-091 anti-volatile-pin sweep; pass-8 fix-burst 0b2c0983; RE-ADDED at
  pass-20 F-P20-MEDIUM-002). prism-spec-engine was originally REMOVED at pass-5 (adversary OBS:
  zero DynamicMessage references, no code changes required) and re-added for comment-only sweep.
  Adversary will check for undeclared crate changes.

### From S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 (MERGED)

- **Predicate field additions follow the same pattern as Literal::RawTemporalLiteral:**
  The temporal typing story added a new variant to the `Literal` enum; this story adds new
  fields to existing `Predicate` variants. The sibling-site sweep obligation (TD-VSDD-060)
  is the same — the compiler surfaces match arms but NOT construction sites. GREP for
  construction sites explicitly (Task 11).

- **Round-trip normalizer location (`PqlNormalizer::normalize_predicate` in ast.rs):** The
  temporal typing story confirmed the normalizer is in `ast.rs` and must emit canonical uppercase
  operator names. Follow the same uppercase-canonical pattern (see how `StringOp` already emits
  `ICONTAINS`/`ISTARTSWITH`/`IENDSWITH` in uppercase from the `(op, case_insensitive)` match).

- **pipe_sql_emitter.rs ICONTAINS pattern is the model:** The ICONTAINS lowering at ~lines
  541-564 is the exact template for IEQ/IIN/INE lowering. Read that code first before
  implementing the new branches.

### From S-DEMO-ENRICHMENT-TYPED-OUTPUT-001 (MERGED)

- **BC-2.16.002 SAP-1 obligation applies to any new tracing::warn! emission:**
  The adapter normalization warning (step 19d) is a `tracing::warn!` with `event_type`.
  This MUST have a BC-2.16.002 Canonical Structured Event Catalog row in the SAME commit.
  The adversary WILL run the SAP-1 grep probe.

- **ARC-DI plumbing:** If the normalization function needs access to `enum_map.rs` data at
  runtime, ensure it is wired via the same dependency injection pattern used by the existing
  adapter (not a lazy static or global). See ADR-022 wiring patterns.

---

## Architecture Compliance Rules

Extracted from ADR-047 and related architectural decisions. Violations are P1 findings in
adversarial review.

1. **No ILIKE for exact equality (ADR-047 §Alternatives Alt-2).**
   The `lower(field) OP lower('val')` idiom is mandatory for IEQ/INE/IIN. `ILIKE` must NOT
   be used because its `%` and `_` metacharacters are semantically wrong for non-pattern matching.
   A query `status ILIKE 'in_progress'` would match `'in progress'` — incorrect behavior.
   The ILIKE function remains appropriate for the existing ICONTAINS/ISTARTSWITH/IENDSWITH paths.

2. **Case-sensitive default is inviolable (ADR-047 §D.1).**
   `=`, `!=`, and `IN` operators MUST NOT emit `lower()` wrapping. The `case_insensitive: false`
   path in `predicate_to_datafusion_sql` must be byte-identical to the pre-story behavior.
   AC-011 and AC-013 enforce this. Any regression in this area is CRITICAL.

3. **enum_map.rs is the sole canonical casing authority (BC-2.02.010 v1.5, BC-2.02.013).**
   No sensor-specific adapter may produce a different casing for an OCSF enum-label field
   without explicit justification against the OCSF schema AND a corresponding enum_map.rs
   amendment. The normalization function must ONLY use enum_map.rs — no hardcoded caption strings.

4. **Normalization applies BEFORE DynamicMessage creation (BC-2.02.002 v1.5).**
   The canonical-case normalization must run in the pipeline stage BEFORE `DynamicMessage`
   construction. Post-creation mutation of DynamicMessage fields is forbidden.

5. **`#[non_exhaustive]` discipline — field additions do not require count increment.**
   Adding `case_insensitive: bool` to existing `#[non_exhaustive]` enum variants is NOT the
   same as adding a new type with `#[non_exhaustive]`. The EXPECTED=89 gate counts annotated
   types, not fields within existing types. If a new PUBLIC TYPE is introduced (not expected),
   increment EXPECTED to 90 in `scripts/check-non-exhaustive.sh` AND update this AC-027.

6. **IIN must appear BEFORE IN in the grammar combinator chain (design map §A Collision check).**
   Chumsky combinators try alternatives in order. If `kw("IN")` is tried before `kw("IIN")`,
   the `"IN"` prefix of `"IIN"` would be consumed, leaving `"N"` as an unrecognized identifier.
   This is a hard ordering constraint. The implementer must verify the combinator order by
   reading `filter_parser.rs` and confirming `IIN` is listed first.

7. **No new E-QUERY error codes introduced by this story.**
   E-QUERY-001 (parse error) and E-QUERY-002 (QueryTypeMismatch) are existing codes used by
   the IEQ/IIN/INE error cases. No new codes are needed. If during implementation a new error
   condition is discovered that requires a new code, surface it to the orchestrator with the
   proposed new code ID and message format — do NOT invent codes silently.

8. **OD-4 near-miss hint is EXPLICITLY EXCLUDED from this story.**
   Do not implement the zero-rows near-miss pedagogical hint (the feature where `=` returning
   zero rows triggers a hint about `IEQ`). This was deferred per ADR-047 OD-4 and human
   sign-off D-1398. Any implementation of OD-4 features in this story is out of scope.

9. **Code comments cite UNVERSIONED BC IDs (TD-VSDD-091, pass-8 F-LOW-1 sweep).**
   A pass-8 F-LOW-1 finding swept 182 inline code comment sites across the codebase that
   previously cited versioned BC IDs (e.g., `// BC-2.11.024 v1.4`). All 182 sites were
   updated to use unversioned BC IDs (e.g., `// BC-2.11.024`) per TD-VSDD-091 (anti-volatile-pin
   rule — narrative spec content must cite function names + behavioral anchors, not version
   numbers that decay on every spec amendment). New code comments added by this story and any
   future story MUST use unversioned BC IDs in inline comments. Versioned pins are only
   acceptable in story frontmatter, changelog entries, and AC traceability prose.

---

## Forbidden Dependencies

The following modules/packages must NOT appear as new imports in the files modified by this story:

| Module | Reason |
|--------|--------|
| `native-tls` (via reqwest or otherwise) | ADR-050: rustls-tls mandatory; native-tls causes ~65s macOS Keychain init + MITM risk |
| `regex` crate (in filter_parser.rs) | IEQ/IIN/INE use simple `kw()` combinator matching, not regex; adding regex for keyword matching would be over-engineering |
| Any new `tokio::*` imports in the pure-function emitter path | `predicate_to_datafusion_sql` is a pure function; async I/O is not appropriate here |

---

## Deferred Items

| Item | Reason Deferred | Future Story Target |
|------|-----------------|---------------------|
| Zero-rows near-miss pedagogical hint (OD-4) | Human sign-off D-1398 explicitly deferred per ADR-047 §Open Decisions OD-4 resolution | Future story (TBD; to be registered post-T13 demo) |
| Proptest VP for IEQ plan-equality (VP-TBD from BC-2.11.024) | VP authoring requires formal verifier agent; not blocking for demo | Phase 6 (Formal Hardening) — to be authored by product-owner before VP authoring pass |
| Proptest VP for normalized_pql round-trip invariant (VP-TBD from BC-2.11.024) | Same as above | Phase 6 (Formal Hardening) |
| Cross-sensor aggregation VP (VP-TBD from BC-2.02.013) | Same as above | Phase 6 (Formal Hardening) |

---

## Scope Ambiguities Resolved

The following ambiguities were present in the input artifacts and resolved by the story-writer
per the production-grade default:

**Ambiguity 1: Adapter normalization PRIMARY site — RESOLVED WITH AMENDMENT (architect adjudication F-CRIT-002, pass-5, 2026-07-07).**

Pass-2 resolution (2026-07-06): confirmed prism-ocsf as the OCSF normalization crate; prism-spec-engine
has zero DynamicMessage references and was excluded.

Pass-5 architect adjudication F-CRIT-002 (2026-07-07): further refines the insertion point within the
codebase. `OcsfNormalizer::normalize_with_mappers` is the DynamicMessage-construction site (BC-2.02.002)
and is the SECONDARY insertion site (for the future protobuf/DynamicMessage path). The PRIMARY
production insertion point is `build_column_array` in `crates/prism-bin/src/spec_driven_adapter.rs` —
the spec-driven adapter materialization path where Arrow arrays are built from JSON sensor records.
This is the path all CrowdStrike/Cyberint/Claroty/Armis TOML-driven sensors flow through at query
time; normalization here ensures consistent casing before the data enters DataFusion execution.

UPDATED IMPLEMENTER DIRECTION:
1. PRIMARY: implement the canonical-case enum-label rewrite in `build_column_array`
   (`crates/prism-bin/src/spec_driven_adapter.rs`). For each column with `ColumnType::String` and
   a name matching an OCSF enum-label field (severity, status, activity_name, disposition),
   call `OcsfEnumMap::normalize_enum_label` and materialize the canonical value (or raw + warn for
   unrecognized). This satisfies BC-2.02.013 v1.9 §Postconditions PRIMARY clause.
2. SECONDARY: also implement in `prism-ocsf/src/normalizer.rs` (`normalize_with_mappers` path) to
   satisfy the DynamicMessage/protobuf path independently. BC-2.02.013 v1.9 §Postconditions states
   both sites must independently satisfy the contract.
3. prism-spec-engine was REMOVED from `crates_touched` (pass-5 adversary OBS) — no production code
   changes required there. RE-ADDED at pass-20 (F-P20-MEDIUM-002) with comment-only scope: the
   TD-VSDD-091 anti-volatile-pin sweep (pass-8 fix-burst 0b2c0983) touched doc comments only.
   Zero code changes in prism-spec-engine.

`OcsfEnumMap` remains in `prism-ocsf/src/enum_map.rs`; both insertion sites depend on it. The
canonical caption authority and sole casing reference (BC-2.02.010 v1.5) is unchanged.
The adversary's SAP-2 probe verifies TOML↔DTU parity regardless of insertion site.

**Ambiguity 2: Non-exhaustive gate count (87 vs 89).**
ADR-047 and the design map cite `EXPECTED=87`. CLAUDE.md currently shows `EXPECTED=89`
(incremented by S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 and S-DEMO-ENRICHMENT-TYPED-OUTPUT-001).
RESOLUTION: The current correct count is 89 (CLAUDE.md is the source of truth for live project
state). This story does NOT increment the count (no new annotated types, only field additions
to existing types). AC-026 checks for 89.

**Ambiguity 3: Are INE and IIN/IIN parity with INE in the grammar mandatory?**
The design map title mentions "IEQ/IIN/INE" but some sections focus only on IEQ and IIN.
RESOLUTION: INE is explicitly confirmed in ADR-047 §D.2 ("The INE operator is included for
completeness of the family (additive, no extra cost)") and BC-2.11.024 postconditions. All
three operators are in scope. INE is implemented as `Predicate::Compare{op: Ne, case_insensitive: true}`.

---

## Story Changelog

| Version | Date | Change Summary |
|---------|------|----------------|
| v1.48 | 2026-07-09 | **pin-sync BC-2.11.016 v1.19 / BC-2.11.017 v1.7 / BC-2.11.020 v1.12 / BC-2.11.004 v1.24 / error-taxonomy v2.32 (STAGE-JOIN SUSPENSION RULE, ADV-FIX-P14-OBS-001) + semantic-cell currency (FIX-IEQ-ERRPATH-001 pass-14 story pin round). BC-2.11.004 v1.23→v1.24: four live sites updated — (1) frontmatter BC status comment; (2) §Behavioral Contracts body table version cell; (3) §Token Budget row; (4) AC-013b trace. error-taxonomy v2.31→v2.32: one live site (AC-022 body). Semantic-cell currency: BC-2.11.004 Key Clauses cell ("Amended: IEQ/IIN/INE available in \| where stages via shared filter grammar (ADR-046 D7)") verified CURRENT — STAGE-JOIN SUSPENSION RULE is a BC-2.11.016 stage-walk binding clause (PipeStage::Join in pipe-stage walk → suspended:=true for remainder), not a pipe-mode IEQ amendment; no Key Clauses extension needed. BC-2.11.016, BC-2.11.017, and BC-2.11.020 not cited in this story. AC semantics UNCHANGED. Frontmatter version 1.47→1.48; updated 2026-07-09 (POL-23).** |
| v1.47 | 2026-07-09 | **pin-sync BC-2.11.016 v1.18 / BC-2.11.017 v1.6 / BC-2.11.020 v1.11 / BC-2.11.004 v1.23 / error-taxonomy v2.31 (STAR-WITH-JOIN SUSPENSION RULE, ADV-FIX-P12-OBS-002) + semantic-cell currency (FIX-IEQ-ERRPATH-001 pass-12 story pin round). BC-2.11.004 v1.22→v1.23: four live sites updated — (1) frontmatter BC status comment; (2) §Behavioral Contracts body table version cell; (3) §Token Budget row; (4) AC-013b trace. error-taxonomy v2.30→v2.31: one live site (AC-022 body). Semantic-cell currency: BC-2.11.004 Key Clauses cell ("Amended: IEQ/IIN/INE available in \| where stages via shared filter grammar (ADR-046 D7)") verified CURRENT — STAR-WITH-JOIN SUSPENSION RULE is a BC-2.11.016 head-projection binding rule, not a pipe-mode IEQ amendment; no Key Clauses extension needed. BC-2.11.016, BC-2.11.017, and BC-2.11.020 not cited in this story. AC semantics UNCHANGED. Frontmatter version 1.46→1.47; updated 2026-07-09 (POL-23).** |
| v1.46 | 2026-07-09 | **pin-sync BC-2.11.016 v1.17 / BC-2.11.017 v1.5 / BC-2.11.020 v1.10 / BC-2.11.004 v1.22 / error-taxonomy v2.30 (LAST-SEGMENT rule, ADV-FIX-P10-OBS-001) + semantic-cell currency (FIX-IEQ-ERRPATH-001 pass-10 story pin round). BC-2.11.004 v1.21→v1.22: four live sites updated — (1) frontmatter BC status comment; (2) §Behavioral Contracts body table version cell; (3) §Token Budget row; (4) AC-013b trace. error-taxonomy v2.29→v2.30: one live site (AC-022 body). Semantic-cell currency: BC-2.11.004 Key Clauses cell ("Amended: IEQ/IIN/INE available in \| where stages via shared filter grammar (ADR-046 D7)") verified CURRENT — LAST-SEGMENT OUTPUT-NAME RULE is a BC-2.11.016 binding rule, not a pipe-mode IEQ amendment; no Key Clauses extension needed. BC-2.11.016, BC-2.11.017, and BC-2.11.020 not cited in this story. AC semantics UNCHANGED. Frontmatter version 1.45→1.46; updated 2026-07-09 (POL-23).** |
| v1.45 | 2026-07-09 | **pin-sync to BC-2.11.016 v1.15 / BC-2.11.017 v1.4 / BC-2.11.020 v1.9 / BC-2.11.004 v1.21 / error-taxonomy v2.29 + semantic-cell currency (FIX-IEQ-ERRPATH-001 pass-7 fix round). BC-2.11.004 v1.20→v1.21: four live sites updated — (1) frontmatter BC status comment; (2) §Behavioral Contracts body table version cell; (3) §Token Budget row; (4) AC-013b trace. error-taxonomy v2.28→v2.29: one live site (AC-022 body). BC-2.11.004 Key Clauses cell ("Amended: IEQ/IIN/INE available in \| where stages via shared filter grammar (ADR-046 D7)") verified CURRENT (v1.21 is pin-only companion to BC-2.11.016 v1.15; no behavioral change). BC-2.11.016, BC-2.11.017, and BC-2.11.020 not cited in this story. AC semantics UNCHANGED. Frontmatter version 1.44→1.45; updated 2026-07-09 (POL-23).** |
| v1.44 | 2026-07-09 | **pin-sync to BC-2.11.016 v1.14 / BC-2.11.004 v1.20 / BC-2.11.020 v1.8 / error-taxonomy v2.28 + Key Clauses semantic refresh (closes ADV-FIX-P6-MED-001/OBS-001). BC-2.11.004 v1.19→v1.20: four live sites updated — (1) frontmatter BC status comment; (2) §Behavioral Contracts body table version cell; (3) §Token Budget row; (4) AC-013b trace. error-taxonomy v2.27→v2.28: one live site (AC-022 body). BC-2.11.004 Key Clauses cell ("Amended: IEQ/IIN/INE available in \| where stages via shared filter grammar (ADR-046 D7)") verified CURRENT (v1.20 was a pin-only update; no behavioral change). BC-2.11.016 and BC-2.11.020 not cited in this story. AC semantics UNCHANGED. Frontmatter version 1.43→1.44; updated 2026-07-09 (POL-23).** |
| v1.43 | 2026-07-09 | **pin-sync to BC-2.11.016 v1.13 / BC-2.11.004 v1.19 / BC-2.11.020 v1.7 / error-taxonomy v2.27 (FIX-IEQ-ERRPATH-001 D-1615 frozen pin round). BC-2.11.004 v1.18→v1.19: four live sites (frontmatter BC status comment; §Behavioral Contracts body table — pre-existing drift from v1.42 corrected from v1.17 to v1.19; §Token Budget row; AC-013b trace). error-taxonomy v2.26→v2.27: one live site (AC-022 body). BC-2.11.016 and BC-2.11.020 not cited in this story — no changes. AC semantics UNCHANGED. Frontmatter version 1.42→1.43; updated 2026-07-09 (POL-23).** |
| v1.42 | 2026-07-08 | **Reconciling pin round (pass-4 closures): BC-2.11.004 v1.17→v1.18 (3 live sites: frontmatter BC status comment; §Behavioral Contracts body table version cell; §Token Budget row; AC-013b trace) + error-taxonomy.md v2.25→v2.26 (1 live site: AC-022 body). Historical changelog rows left unchanged per POL-29. AC semantics UNCHANGED. Frontmatter version 1.41→1.42; updated 2026-07-08 (POL-23).** |
| v1.41 | 2026-07-08 | **Reconciling pin round (pass-3 closures): BC-2.16.002 v2.06→v2.07 (1 live site: AC-018 body) + error-taxonomy.md v2.20→v2.25 (1 live site: AC-022 body). Historical changelog rows and Red Gate table BC-2.16.002 v2.06 citations left unchanged per POL-29. AC semantics UNCHANGED. Frontmatter version 1.40→1.41; updated 2026-07-08 (POL-23).** |
| v1.40 | 2026-07-08 | **BC-2.11.004 v1.16→v1.17 version-pin propagation (sort-grammar fix round, POL-29/POL-23).** PO bumped BC-2.11.004 v1.16→v1.17 (sort-grammar micro-fix). Four live version-pin cites updated: (a) frontmatter BC status comment; (b) §Behavioral Contracts body table version cell; (c) §Token Budget row; (d) AC-013b trace. Historical changelog rows left unchanged per POL-29. AC semantics UNCHANGED. Frontmatter version 1.39→1.40; updated 2026-07-08 (POL-23). |
| v1.39 | 2026-07-08 | **BC-2.11.004 v1.15→v1.16 version-pin propagation (pass-2 CRIT closure burst, POL-29/POL-23).** PO bumped BC-2.11.004 v1.15→v1.16 (14-position gate + derived-column binding rule). Four live cites updated: (a) frontmatter BC status comment; (b) §Behavioral Contracts body table version cell; (c) §Token Budget row; (d) AC-013b trace. Historical changelog rows left unchanged per POL-29. AC semantics UNCHANGED. Frontmatter version 1.38→1.39; updated 2026-07-08 (POL-23). |
| v1.38 | 2026-07-08 | **BC-2.11.004 v1.14→v1.15 + BC-2.11.024 v1.3→v1.4 + BC-2.02.013 v1.8→v1.9 version-pin propagation (POL-29/POL-23).** (1) **BC-2.11.004 v1.14→v1.15 (`\| project`→`\| fields` keyword sweep):** Four live cites updated — (a) frontmatter BC status comment; (b) §Behavioral Contracts body table version cell; (c) §Token Budget row; (d) AC-013b trace. (2) **BC-2.11.024 v1.3→v1.4 (POL-14 draft→active promotion at merge, D-1607 changelog-row-only bump):** All live body cites updated via replace_all (~40 sites: frontmatter comment, §Behavioral Contracts body table version cell, §Token Budget row, all AC-trace/edge-case/Red-Gate-table/Tasks-section inline cites). Historical changelog rows (v1.30 "per BC-2.11.024 v1.3 §Mode-Boundary", v1.28 "contradicting BC-2.11.024 v1.3") left unchanged per POL-29. (3) **BC-2.02.013 v1.8→v1.9 (POL-14 draft→active promotion at merge, D-1607 changelog-row-only bump):** All live body cites updated via replace_all (~18 sites: frontmatter comments ×2, §Behavioral Contracts body table version cell, §Token Budget row, all AC-trace/Tasks-section inline cites). Historical changelog row (v1.31 "BC-2.02.013 v1.7→v1.8 pin sweep") not affected (→v1.8 not matched by pattern). Convention established: live-pin propagation applies even post-merge, including POL-14 lifecycle-only version bumps — evidenced by S-DEMO-ENRICHMENT-TYPED-OUTPUT-001 (merged, BC-2.16.002 v1.96→v1.97 propagated post-merge) and S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 (merged, BC-2.11.020 v1.1→v1.2 propagated post-merge). AC semantics UNCHANGED. Frontmatter version 1.37→1.38; updated 2026-07-08 (POL-23). |
| v1.37 | 2026-07-08 | **BC-2.11.002 v1.5→v1.6 + BC-2.11.004 v1.13→v1.14 version-pin propagation (ADV-FIX-P1-HIGH-001, POL-29/POL-23).** Product-owner amended BC-2.11.002 v1.5→v1.6 and BC-2.11.004 v1.13→v1.14 (sibling BCs bumped in the same burst as BC-2.11.016 v1.6). Live version-pin cites updated: BC-2.11.002: (1) frontmatter BC status comment; (2) §Behavioral Contracts body table row version cell; (3) §Token Budget row; (4) AC-001 trace; (5) AC-002 trace; (6) AC-012 trace; (7) AC-024 trace — 7 sites. BC-2.11.004: (1) frontmatter BC status comment; (2) §Behavioral Contracts body table row version cell; (3) §Token Budget row; (4) AC-013b trace — 4 sites. AC semantics UNCHANGED — IEQ/IIN/INE amendment content is preserved; only the version number cited is updated to the canonical current version. Frontmatter version 1.36→1.37; updated unchanged at 2026-07-08 (POL-23). |
| v1.36 | 2026-07-08 | PR-LEVEL pass-5 fix-burst @36a094d6 — ADV-PR-P5-OBS-001 security-upgraded to MED (CWE-117 C1 control gap): sanitize_for_log widened to `!is_control() && != U+2028/U+2029` (BC-2.16.002 v2.06 + error-taxonomy v2.20 + 9-site comment sweep); RG-082 (`test_sanitize_for_log_strips_unicode_cc_and_line_separators`, prism-core/src/error.rs). ADV-PR-P5-OBS-002: check_ci_column_types Err-swallow fixed (explicit 3-arm match; Err → QueryExecutionFailed redacted-detail; server-side tracing::error! bare, no event_type); RG-083 (`test_check_ci_column_types_schema_provider_err_propagates`, prism-query/src/materialization.rs). ADV-PR-P5-OBS-003: academic ASCII/Unicode casefold asymmetry → lessons.md note only. RGT 81→83. BC-2.16.002 v2.05→v2.06 pin propagation (2 live sites: AC-018, RG-080 row). error-taxonomy v2.19→v2.20 pin propagation (1 live site: AC-022). |
| v1.35 | 2026-07-08 | PR-LEVEL pass-3 OBS closure @fab7df00 — OBS-001: RG-080 extended with `sensor_type` order mirror (WarnFieldVisitor/WarnFieldCapture rename; 65-codepoint ESC "B" vector; 3 mirrored assertions; BC-2.16.002 v2.05 row 91 symmetry); OBS-002: evidence report provenance made durable (code-behavior HEAD phrasing). RGT count unchanged at 81. No BC changes. |
| v1.34 | 2026-07-08 | PR-LEVEL pass-1 fix-burst @56fb83d8+f9be96fa+1172b15a — ADV-PR-P1-MED-001: sanitize-before-truncate order corrected at 5 `ocsf.enum_label_unrecognized` warn sites + 2 comment sites (code brought TO BC-2.16.002 v2.05 spec; `infusion_udf.rs` truncate-first verified INTENTIONAL per its own catalog row — no change there); ADV-PR-P1-MED-002: RG-078 paper-fix superseded by RG-079 (`test_rg079_secondary_sanitize_enum_label_order_spec_wins` — load-bearing `sanitize_enum_label_for_log` helper call at SECONDARY site); ADV-PR-P1-LOW-001: RG-080 (`test_rg080_low001_build_column_array_enum_label_warn_order_of_operations` — PRIMARY site order-of-ops, 65-codepoint control-char vector); ADV-PR-P1-LOW-002: evidence report synced to f9be96fa; ADV-PR-P1-OBS-002: RG-081 (`test_rg081_obs002_suggested_suffix_display_some_and_none` — SuggestedSuffix Display lock Some/None); ADV-PR-P1-OBS-001 informational, no artifact change. RGT 78→81. No BC version changes. |
| v1.33 | 2026-07-08 | Pre-PR-LEVEL fix-burst @54c89898: CR-002 canonical-label write guard (`normalize_enum_label` returns `Some` when already canonical; RG-075); CR-003 tracing::warn before IEQ placeholder fallback on invalid case_insensitive+non-Eq/Ne combination (RG-076); CR-004/SEC-001 CWE-117 `sanitize_for_log` at PRIMARY `ocsf.enum_label_unrecognized` site in `build_column_array` (RG-077) and SECONDARY `normalize_with_mappers` site (RG-078); CR-005 `build_example_note`→`build_example_with_note` rename (96 refs); CR-001 refuted (segments.last() correct for flat Arrow schemas — no change). RGT 74→78. BC-2.16.002 v2.04→v2.05 at live AC-018 site (1 pin site; catalog row 91 value/sensor_type descriptions updated for CWE-117 stripping). |
| v1.32 | 2026-07-08 | Pass-32 closures: F-P32-MED-001 BC-2.16.002 v2.03→v2.04 pin update (live AC-018 site: pipe.sql_lowering/pipe.sql_planning_error rows now enumerate BOTH emission arms — Ast::Pipe and Ast::SqlPipe, @de89b557); F-P32-OBS-001 numeric catalog-row comment pins → event-type-name citations (materialization.rs ×2, @de89b557); F-P32-OBS-002 IEQ/INE non-string RHS message enumeration dropped (not spec-pinned; IIN pinned message untouched). No RGT changes (74 unchanged). |
| v1.31 | 2026-07-08 | Pass-28 closures: F-P28-MED-001 BC-2.02.013 v1.7→v1.8 pin sweep (18 live sites: 17 body + 1 BC-table row); F-P28-MED-002 TD-VSDD-091 code-comment pin strip @669080f5 (217 sites, 25 files, incl. 47 stale BC-2.10.012 pins in prism_describe.rs); F-P28-LOW-001 sql_parser.rs §DML-Mode-Boundary citation corrected to §Mode-Boundary Enforcement (DML scope). No RGT changes (74 unchanged). |
| v1.30 | 2026-07-08 | F-P27-HIGH-001 (pass-27): AC-025 assertion (1) + Task 23 illustrative `example_query` corrected SQL-mode SELECT→pipe-mode canonical form (`FROM <table> | where severity IEQ 'high' | limit 50`) per BC-2.11.024 v1.3 §Mode-Boundary Enforcement + BC-2.10.012 v1.9; POL-25 sweep: 8 `SELECT.*IEQ/IIN/INE` sites audited — 2 fixed (AC-025 body + Task 23 code block), 6 left unchanged (AC-023 intentional rejection inputs + RG-023/024/025/039/064 rejection-test descriptions). No code change; no RGT change. |
| v1.29 | 2026-07-08 | F-P25-MED-001 (pass-25): corrected F-P24-LOW-001 file citation prism-query→prism-mcp (`crates/prism-mcp/tests/bc_2_10_016_audit_004_test.rs`); vocabulary-entry list corrected from (`'HIGH'`, `'CRITICAL'`, `'OPEN'`, etc.) to (`'HIGH'`, `'CRITICAL'`, `'MEDIUM'`, `'LOW'`). Story-side fix only; no code changes, no new RGT. |
| v1.28 | 2026-07-07 | LOCAL pass-22 CLEAN(strict) — streak 1/3 (frozen 2de85b18). Pass-23 CLEAN(strict) — streak 2/3 (frozen 2de85b18). Pass-24 NOT CLEAN — F-P24-MED-001 (`engine.rs` `valid_operators_for_type(ColumnType::String)` omitted IEQ/IIN/INE from machine-readable E-QUERY-002 `valid_operators_for_type` contract, contradicting BC-2.11.024 v1.3 and the Display-prose SuggestedSuffix) + F-P24-LOW-001 (stale "prompt uses IIN" comment + dead ALL-CAPS vocabulary entries in `bc_2_10_016_audit_004_test.rs`). Both findings closed fix-in-scope; commit 633c5fab. RG-074 added (`test_BC_2_11_024_f_p24_med001_valid_operators_string_includes_ci_operators`, `engine.rs` `#[cfg(test)] mod tests`, 8-operator set: =, !=, LIKE, IN, NOT IN, IEQ, IIN, INE); sibling updates in `e_query_pedagogical.rs` (required_string 5→8) and `normalized_pql.rs` (comments); `error_mapping.rs` auto-tracks (derives dynamically). F-P24-LOW-001: dead ALL-CAPS entries removed, comment corrected to Title-case state (no new RGT). `red_gate_tests` 73→74; authoritative range "RG-028 through RG-073"→"RG-028 through RG-074"; Total/Task-28 counts updated 73→74 + 1 pass-24. Streak reset 0/3; next pass-25 on frozen 633c5fab. |
| v1.27 | 2026-07-07 | LOCAL pass-21: 1 OBS finding F-P21-OBS-001 (`explain.rs` `predicate_to_exprs` dropped `case_insensitive` flag — latent EXPLAIN push-down misreport once ColumnSpec is wired) closed fix-in-scope per production-grade default; RG-073 added (`test_BC_2_11_024_f_p21_obs001_explain_ieq_iin_not_classified_pushdownable`, `crates/prism-query/src/explain.rs`, mod `predicate_explain_classification_tests`); `red_gate_tests` 72→73; "RG-028 through RG-072"→"RG-028 through RG-073"; Total/Task-28 counts updated to 73 + 1 pass-21. Commit 2de85b18. CLEAN(PR-merge)=yes on pass-21. |
| v1.26 | 2026-07-07 | LOCAL pass-20 findings F-P20-HIGH-001/MEDIUM-001/MEDIUM-002/LOW-001/LOW-002 closed. (1) F-P20-MEDIUM-001 — subsystems [SS-11, SS-02]→[SS-11, SS-02, SS-22]; SS-22 anchor justification added (prism-bin::build_column_array is PRIMARY normalization insertion point per BC-2.02.013 v1.7 F-CRIT-002 adjudication). (2) F-P20-MEDIUM-002 — prism-spec-engine re-added to crates_touched (comment-only TD-VSDD-091 anti-volatile-pin sweep, pass-8 fix-burst 0b2c0983; zero code changes; File Structure Requirements row updated from REMOVED→RE-ADDED with comment-only scope; tdd_mode rationale comment updated). (3) BC-2.02.013 pin v1.6→v1.7 at all 16 live sites (additive subsystems_multi: ["SS-22", "SS-02"] frontmatter field; no semantic contract change). (4) F-P20-HIGH-001: RG-071 (`test_BC_2_02_013_build_column_array_group_by_severity_cross_sensor_no_fragmentation`, prism-bin; commit 345d4154) + F-P20-LOW-001: RG-072 (`test_f_p20_low001_severity_integer_type_does_not_get_ieq`, prism-mcp; commit 257074af); red_gate_tests 70→72; "RG-028 through RG-070"→"RG-028 through RG-072"; Total/Task-28 counts updated to 72 + 2 pass-20. (5) F-P20-LOW-002 / Pass-19 note (D-1589 carry-forward, commit eb7256b7): RG-043/044 strengthened in-place — caption sets derived from all four OCSF enum-label caption sets; 3 flagship prompts corrected to status='New'; crowdstrike SENSOR_COLUMN_VOCABULARIES 'new'→'New'; reference IN example re-pointed to client_id IN ('acme','globex'); no new RGT names. Changelog table reordered monotonic descending per POL-32. |
| v1.25 | 2026-07-07 | pass-18 OBS closures: RG-068 PIPE MODE IEQ skeleton guard (`test_RG_067_query_tool_pipe_mode_example_uses_ieq_not_equals`, server.rs adr_042_tests, OBS-1); RG-069/070 severity-column-presence gate replaces SENSOR_SEVERITY_VOCABULARY allowlist (`test_obs2_sentinel_alerts_with_severity_gets_ieq_example` + `test_obs2_claroty_devices_no_severity_column_still_no_ieq_note`, prism_describe.rs, OBS-2); SENSOR_SEVERITY_VOCABULARY const REMOVED (gate now has_severity column presence — universally correct post-normalization); `test_f_l2_crit001_unknown_sensor_with_severity_falls_back_to_count_recent` renamed to `..._gets_ieq`; red_gate_tests 67→70; "RG-028 through RG-067"→"RG-028 through RG-070"; Total/Task-28 counts updated to 70 + 3 pass-18 |
| v1.24 | 2026-07-07 | pass-17: F-HIGH-1 query-tool description post-normalization rewrite (RG-067 guard — `test_BC_2_10_009_query_tool_description_no_vendor_casing_teaches_ieq` in `crates/prism-mcp/tests/mcp_prism_describe.rs`); RG-062 re-anchored to `test_BC_2_10_012_build_example_note_query_parses_without_stripping` (build_example_query_tests module, verified in worktree); BC-2.10.012 pin v1.8→v1.9 at all live sites (frontmatter comment, BC table, Token Budget, AC-025 interaction note ×3, AC-025 body, RG-061/062/063 rows, Task-23 ×2) per ADR-047 §D.4 precedence; red_gate_tests 66→67; "RG-028 through RG-066"→"RG-028 through RG-067"; Total/Task-28 counts updated to 67 + 1 pass-17 |
| v1.23 | 2026-07-07 | pass-16: RG-065/066 CI pre-flight guard tests added (zero-row unregistered-table skip adjudicated F-P16-OBS-002; empty CI-field list Ok); shared_enum_map() single-access-point note added to Architecture Mapping (F-P16-OBS-001; duplicate statics removed); red_gate_tests 64→66; "RG-028 through RG-064"→"RG-028 through RG-066"; Total/Task-28 counts updated to 66 + 2 pass-16 |
| v1.22 | 2026-07-07 | pass-16: BC-2.10.012 v1.8 added to behavioral_contracts frontmatter array and body Behavioral Contracts table (F-P16-MED-001; AC-025/RG-061..063 material dependency on pure-PQL invariant + example_note field contract); BC-2.02.013 pin v1.5→v1.6 at all 9 live sites (frontmatter comment, Behavioral Contracts table, Token Budget, AC-016 ×2, AC-017, AC-018, AC-019, File Structure Requirements, Task 19d, Scope Ambiguities — empty-string bypass EC-02-028 + sensor_type cap documented in v1.6); BC-2.16.002 pin v2.01→v2.03 in AC-018 cap language; AC-018 extended to note both `value` and `sensor_type` fields are 50-codepoint-capped per BC-2.02.013 v1.6; Token Budget BC count 7→8 + total ~82k→~83k |
| v1.21 | 2026-07-07 | pass-15: RG-059..064 (IIN non-string parse-time ×2, example_query purity ×2, example_note contract, precedence lock); BC-2.11.024 pin v1.2→v1.3; BC-2.10.012 v1.8 interaction noted at AC-025 (example_note field governs OCSF casing note, example_query is pure PQL); AC-021 template updated (richer empty-IIN message + EC-11-024-012 extension note for non-string elements); AC-025 example_note reframing (pure-PQL invariant, example_note field); Task 23 updated; red_gate_tests 58→64 |
| v1.20 | 2026-07-07 | pass-14: RG-055..058 (date-like RHS string acceptance ×3 + SqlPipe head WHERE IEQ lock); BC-2.02.013 pin v1.4→v1.5 (dual-subsystem traceability); red_gate_tests 54→58; AC-020 refined — quoted date-like RHS always accepted as Literal::String on CI operators regardless of date-like shape (ADR-052 reclassification does not affect quoted strings) |
| v1.19 | 2026-07-07 | pass-13: AC-016 stale `low_to_title_case` SECONDARY Red Gate citation removed — test does not exist, `critical_to_title_case` (RG-019) is the correct SECONDARY (F-LOW-1); AC-023 compound-violation precedence note added — string-literal try_map fires before mode-boundary walk when both constraints violated (F-OBS-2) |
| v1.18 | 2026-07-07 | pass-12: RG-054 SECONDARY empty-string parity mirror; 8 stale v1.0 test-comment pins stripped (code-side); red_gate_tests 53→54 |
| v1.17 | 2026-07-07 | pass-12 F-P11-LOW-1: RG-044 description updated IIN→IN ('High','Critical') per pass-10 F-CRIT-1 correction (2 sites: AC-019 body citation + pass-9 RG inventory row) |
| v1.16 | 2026-07-07 | pass-11 polish: RG-048..053 added (negated-IIN marker, detector real-op, IN-list AST lock, describe parse-lock, index collision guards x2); OCSF_ENUM_LABEL_FIELDS single-sourced as `pub const` in prism-ocsf imported by prism-bin (F-OBS-3 drift-proof); red_gate_tests 47→53; "RG-028 through RG-047"→"RG-028 through RG-053"; Total/Task-28 counts updated to 53 + 6 pass-11 |
| v1.15 | 2026-07-07 | pass-10: RG-046 (prompt-parseability — `test_BC_2_11_024_all_prompt_embedded_queries_parse`, AC-023 corollary) + RG-047 (empty-string no-warn — `test_BC_2_02_013_build_column_array_empty_string_enum_value_no_warn`, AC-018 guard); AC-023 + AC-018 body citations added; red_gate_tests 45→47; "RG-028 through RG-045"→"RG-028 through RG-047"; Total/Task-28 counts updated |
| v1.14 | 2026-07-07 | pass-10 F-HIGH-2: AC-018 ellipsis-sentinel claim removed (BC-2.02.013 v1.5 / BC-2.16.002 v2.01 specify plain 50-codepoint cap — SEC-002; no `…` suffix; spec precedence over story text) |
| v1.13 | 2026-07-07 | pass-9: RG-043/044/045 added (reference-content casing, triage-prompt casing, IEQ pushdown-exclusion); red_gate_tests 42→45; RG-028 through RG-045 authoritative |
| v1.12 | 2026-07-07 | pass-9 F-P9-MED-1: prism-core added to crates_touched (error.rs QueryTypeMismatch.suggested_column + SuggestedSuffix helper + RG-029/030 test file); File Structure Requirements rows added — crates/prism-core/src/error.rs (MODIFY) + crates/prism-core/tests/bc_2_11_024_query_type_mismatch_display.rs (CREATE) |
| v1.11 | 2026-07-07 | pass-8: RG-041/042 SqlPipe E-QUERY-002 rows added (AC-022 Red Gate citations + pass-8 inventory section); red_gate_tests 40→42; count strings updated "40 (25 core + 2 discoverability + 9 pass-5 + 4 pass-7)" → "42 (... + 2 pass-8 SqlPipe)" in inventory summary and Task 28; TD-VSDD-091 unversioned-pin sweep noted in Architecture Compliance Rule 9 (pass-8 F-LOW-1, 182 sites) |
| v1.10 | 2026-07-07 | RG-039 (DML INSERT INE) + RG-040 (F-MED-1 suppression guard) rows added; AC-023 + AC-025 Red Gate citation lists updated; red_gate_tests 38→40 |
| v1.9 | 2026-07-07 | pass-7 fix-burst: BC-2.11.024 v1.2 DML mode-boundary extension — AC-023 extended to cover DELETE/UPDATE WHERE + INSERT…SELECT; `parse_sql_dml_with_limits` guard requirement + code-comment obligation added to Task 14b; 2 new Red Gate tests RG-037/RG-038 added; AC-018 50-codepoint cap contract language added (BC-2.02.013 v1.5 / BC-2.16.002 v2.01); AC-025 no-suppression clause added (F-MED-1 anchor); all version pins bumped: BC-2.11.024 v1.1→v1.2, BC-2.02.013 v1.3→v1.4, BC-2.16.002 v2.01 introduced at new AC-018 cite; red_gate_tests 36→38 |
| v1.8 | 2026-07-07 | RG-029/030/031 rows corrected to verified test names (were inferred); inferred-name NOTE removed |
| v1.7 | 2026-07-07 | pass-5 fix-burst: AC-016..019 reframed to PRIMARY `build_column_array` insertion point in prism-bin (BC-2.02.013 v1.2→v1.3, architect adjudication F-CRIT-002); prism-bin added to crates_touched / prism-spec-engine REMOVED (pass-5 adversary OBS resolved); AC-025 authoritative Red Gate updated to `test_BC_2_11_024_describe_output_includes_ieq_example_and_ocsf_casing_note` in prism_describe.rs (RG-027 now supplementary); 9 new Red Gate rows added (RG-028..036): 4 pass-5 tests (describe IEQ, 2 Display byte-exact, negated-IIN) + 5 PRIMARY build_column_array tests; red_gate_tests 27→36; BC-2.02.013 v1.2→v1.3 at all 8 pin sites; BC-2.16.002 v1.99→v2.00 (catalog row 91 now lists both emission sites — no in-body version pins to update); Scope Ambiguity 1 amended with PRIMARY/SECONDARY distinction; Tasks 8 + 19 updated for two-insertion-site workflow; Token Budget updated (prism-spec-engine row→prism-bin row) |
| v1.6 | 2026-07-07 | pass-4 fix-burst: RG-015 AC back-ref corrected AC-025→AC-026; full bidirectional RG↔AC traceability sweep — also found and corrected two stale "AC-026" prose refs in Task 25 and Architecture Compliance Rule 5 (non-exhaustive gate is AC-027, not AC-026, after v1.5 renumbering) |
| v1.5 | 2026-07-07 | LOCAL pass-3 fix-burst: AC-023 SQL-mode IEQ/IIN/INE rejection (E-QUERY-001) added; old AC-023 (grammar resource) → AC-024; old AC-024 (describe) → AC-025; old AC-025 (no panic) → AC-026; old AC-026 (non-exhaustive) → AC-027; RG-023/024/025 (SQL-mode rejection tests test_BC_2_11_024_sql_mode_*) added to main Red Gate table; old RG-023/024 renumbered to RG-026/027 with AC refs updated to AC-024/025; red_gate_tests 25→27; Behavioral Contracts table BC-2.11.024 v1.0→v1.1 + Mode-Boundary Enforcement added to Key Clauses Used; error-taxonomy v2.18→v2.19 |
| v1.4 | 2026-07-06 | LOCAL pass-2 fix-burst: AC-022 reworded (E-QUERY-002 suggested_column enrichment — PrismError::QueryTypeMismatch carries suggested_column: Some("severity"), full Display format specified, error-taxonomy.md v2.18 §E-QUERY-002 pin added); activity→activity_name correction in File Structure Requirements and v1.3 changelog entry; in-scope OCSF fields note added to AC-016 area (severity/status/activity_name/disposition); BC-2.02.013 v1.1→v1.2 at all 7 pin sites (frontmatter comment, Behavioral Contracts table, Token Budget table, AC-016/017/018/019 traces); BC-2.16.002→v1.99 deferred (no version pins present in story body per v1.3 note); error-taxonomy v2.18 pin added in new AC-022 text only |
| v1.3 | 2026-07-06 | LOCAL pass-1 fix-burst: BC-2.02.013 v1.0→v1.1 + BC-2.16.002 pin→v1.98 propagation (BC-2.02.013 now concretely specifies in-scope field set severity/status/activity_name/disposition, keying contract, insertion point, warn event; BC-2.16.002 v1.98 added catalog row 91 ocsf.enum_label_unrecognized — note: BC-2.16.002 has no prior version pin in this story, reference sites unchanged per no-AC-text rule) |
| v1.2 | 2026-07-06 | remove-uncertainty pass-2: tdd_mode rationale comment corrected — OCSF adapter-boundary normalization is definitively in prism-ocsf (OcsfNormalizer + OcsfEnumMap), NOT prism-spec-engine (zero DynamicMessage references); Scope Ambiguity 1 closed with DEFINITIVE RESOLUTION; TD-VSDD-091 anchor de-pinning applied (line-number citations removed, replaced with anchor-based references); crates_touched confirmed: prism-query (operators), prism-ocsf (OCSF normalization, primary), prism-spec-engine (defensive allowance, may be removed at PR time if untouched) |
| v1.1 | 2026-07-06 | remove-uncertainty pass-1: AST shape verified against ast.rs; CODE-SHAPE NOTEs added to AC-001/AC-002/AC-003/AC-008/AC-010; Scope Ambiguity 1 added (prism-ocsf vs prism-spec-engine pending verification); Ambiguity 2/3 added; File Structure Requirements expanded; IIN-before-IN collision note refined |
| v1.0 | 2026-07-06 | Initial story decomposition |
