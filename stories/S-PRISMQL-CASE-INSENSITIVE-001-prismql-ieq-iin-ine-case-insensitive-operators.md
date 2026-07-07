---
document_type: story
story_id: S-PRISMQL-CASE-INSENSITIVE-001
title: "PrismQL Case-Insensitive Operators (IEQ/IIN/INE) + Adapter-Boundary OCSF Enum-Label Normalization (ADR-047)"
epic_id: EPIC-DEMO
version: "1.6"
updated: "2026-07-07"
status: draft
producer: story-writer
phase: 3
wave: wave-5
priority: P0
points: 10
tdd_mode: strict
# tdd_mode rationale: this story modifies production Rust code in prism-query
# (filter_parser.rs grammar, ast.rs struct fields, pipe_sql_emitter.rs lowering,
# ast.rs round-trip normalizer) and prism-ocsf (adapter-boundary OCSF enum-label
# normalization — OcsfNormalizer::normalize_with_mappers + OcsfEnumMap in enum_map.rs;
# the definitive DynamicMessage-construction site; prism-spec-engine has zero
# DynamicMessage references). All behavioral changes require Red Gate tests as failing
# todo!() stubs BEFORE production code is modified. Grep-sweep ACs (AC-006, AC-007)
# and VERIFY-only ACs (AC-026) do not have standalone Red Gate stubs but do not
# justify facade mode — production code is modified with new behavioral semantics.
target_module: prism-query
subsystems: [SS-11, SS-02]
# Subsystem anchor justifications:
#   SS-11 (Query Execution) owns prism-query: filter_parser.rs (IEQ/IIN/INE grammar),
#   ast.rs (case_insensitive flag + round-trip normalizer), pipe_sql_emitter.rs
#   (DataFusion lower() lowering). The grammar+AST+emitter is the primary change surface.
#   SS-02 (OCSF Normalization) owns prism-ocsf: enum_map.rs (canonical caption data)
#   and the normalization pipeline that populates OCSF enum-label string fields before
#   DynamicMessage creation. The adapter-boundary fix is the parallel track that ensures
#   stored data is consistently cased so case-sensitive = works correctly across sensors.
crates_touched: [prism-query, prism-ocsf, prism-spec-engine, prism-mcp]
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
# BC array propagation (all 7 BCs cited by ACs in the body below):
#   BC-2.11.024 v1.1 (draft): new — PrismQL IEQ/IIN/INE case-insensitive operators;
#     primary contract for grammar+AST+emitter+round-trip changes. Every parser/emitter AC
#     traces to a BC-2.11.024 postcondition, invariant, or error case.
#   BC-2.02.013 v1.2 (draft): new — adapter-boundary OCSF enum-label canonical-case
#     normalization. Every adapter AC traces to a BC-2.02.013 postcondition, invariant,
#     or error case.
#   BC-2.11.002 v1.5 (active, amended): filter-mode parsing now includes IEQ/IIN/INE in
#     the supported operator table. AC-001/AC-004/AC-012 exercise the filter-mode path.
#   BC-2.11.004 v1.13 (active, amended): pipe-mode | where stage now supports IEQ/IIN/INE
#     via shared filter grammar per ADR-046 D7. AC-013 exercises the pipe-mode path.
#   BC-2.11.018 v1.3 (active, amended): normalized_pql echo now reflects IEQ/IIN/INE
#     predicates in uppercase canonical form (EC-11-057 added). AC-014/AC-015 exercise this.
#   BC-2.02.002 v1.5 (active, amended): normalization applied BEFORE DynamicMessage creation;
#     postconditions updated. AC-016 exercises this ordering invariant.
#   BC-2.02.010 v1.5 (active, amended): enum_map.rs is the sole canonical casing authority
#     at the adapter boundary. AC-016/AC-017 verify this authority chain.
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
red_gate_tests: 27
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
| BC-2.11.024 | v1.1 | PrismQL Case-Insensitive Equality and Membership Operators (IEQ / IIN / INE) | New operator syntax; DataFusion lower() lowering; case-sensitive operators unchanged; normalized_pql round-trip; IEQ superset invariant; IIN non-empty invariant; E-QUERY-001 (non-string RHS, empty list); E-QUERY-002 (non-string column); Mode-Boundary Enforcement (SQL-mode IEQ/IIN/INE rejection, E-QUERY-001) |
| BC-2.02.013 | v1.2 | Adapter-Boundary OCSF Enum-Label Canonical-Case Normalization | Severity + status guaranteed; all OCSF enum-label fields; idempotent; unrecognized values as-received + warning; GROUP BY aggregation consistency; enum_map.rs as sole casing authority |
| BC-2.11.002 | v1.5 | PrismQL Filter Mode Parsing | Amended: IEQ/IIN/INE added to supported filter-mode operator table |
| BC-2.11.004 | v1.13 | PrismQL Pipe Mode | Amended: IEQ/IIN/INE available in \| where stages via shared filter grammar (ADR-046 D7) |
| BC-2.11.018 | v1.3 | normalized_pql Echo | Amended: EC-11-057 added — IEQ/IIN/INE predicates reflected in uppercase canonical form in normalized_pql; round-trip invariant extended |
| BC-2.02.002 | v1.5 | DynamicMessage Creation | Amended: normalization applied BEFORE DynamicMessage creation; postconditions updated to state this explicitly |
| BC-2.02.010 | v1.5 | OCSF Enum Value Map | Amended: enum_map.rs authority extends to adapter-boundary normalization, not only MCP display enrichment |

---

## Token Budget Estimate

| Component | Estimated tokens |
|-----------|-----------------|
| Story spec (this file) | ~18,000 |
| ADR-047 (full) | ~6,000 |
| Design map: prismql-case-insensitive-design-map.md | ~4,500 |
| BC-2.11.024 v1.1 | ~3,000 |
| BC-2.02.013 v1.2 | ~2,500 |
| BC-2.11.002 v1.5 (relevant filter-mode sections) | ~1,500 |
| BC-2.11.004 v1.13 (relevant pipe-mode sections) | ~1,500 |
| BC-2.11.018 v1.3 (normalized_pql section) | ~1,000 |
| BC-2.02.002 v1.5, BC-2.02.010 v1.5 (amended sections) | ~2,000 |
| `crates/prism-query/src/filter_parser.rs` | ~7,000 |
| `crates/prism-query/src/ast.rs` | ~9,000 |
| `crates/prism-query/src/pipe_sql_emitter.rs` | ~8,000 |
| `crates/prism-ocsf/src/enum_map.rs` + normalizer | ~4,000 |
| `crates/prism-spec-engine/src/` (adapter normalization pipeline) | ~6,000 |
| `crates/prism-mcp/src/` (grammar resource, describe examples) | ~4,000 |
| Test files to create / modify | ~5,000 |
| **Total** | **~83,000** |

Estimated at ~42% of a 200K context window. Within the per-story limit. No split required.

---

## Acceptance Criteria

### AC-001 — IEQ parses to Predicate::Compare { case_insensitive: true }
(traces to BC-2.11.024 v1.1 postcondition "New operators and their syntax" — IEQ row;
BC-2.11.002 v1.5 amendment: IEQ added to filter-mode supported operator table)

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
(traces to BC-2.11.024 v1.1 postcondition "New operators and their syntax" — IIN row;
BC-2.11.002 v1.5 amendment)

Given `status IIN ('open', 'new')`,
when parsed,
then the AST contains `Predicate::In { field: FieldPath::new(["status"]), values: vec![Literal::String("open".into()), Literal::String("new".into())], negated: false, case_insensitive: true }`.

CODE-SHAPE NOTE (verified against `ast.rs`): the actual `Predicate::In` variant is
`{ field: FieldPath, values: Vec<Literal>, negated: bool }` — `field` is a `FieldPath` (not a
`String`) and `values` is `Vec<Literal>` (not `Vec<Value>`). The new field `case_insensitive: bool`
is added to the variant.

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_iin_parses_to_in_case_insensitive_true`

### AC-003 — INE parses to Predicate::Compare { op: CompareOp::Ne, case_insensitive: true }
(traces to BC-2.11.024 v1.1 postcondition "New operators and their syntax" — INE row)

Given `severity INE 'informational'`,
when parsed,
then the AST contains `Predicate::Compare { lhs: Box::new(Expr::Field(FieldPath::new(["severity"]))), op: CompareOp::Ne, rhs: Box::new(Expr::Literal(Literal::String("informational".into()))), case_insensitive: true }` (same `lhs`/`rhs` shape as AC-001; `op` is `CompareOp::Ne`).

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_ine_parses_to_compare_ne_case_insensitive_true`

### AC-004 — Keyword parsing is case-insensitive: `ieq`, `IEQ`, `Ieq` produce identical ASTs
(traces to BC-2.11.024 v1.1 postcondition "Operators are parsed case-insensitively in the
Chumsky grammar via the kw(...) combinator: ieq, IEQ, Ieq all parse identically")

Given three queries `severity ieq 'high'`, `severity IEQ 'high'`, and `severity Ieq 'high'`,
when each is parsed,
then all three produce structurally identical ASTs with `case_insensitive: true`.

NOTE: The `kw()` combinator already handles this — no additional implementation needed. This
AC verifies the combinator behavior is preserved.

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_keyword_case_insensitive_parsing`

### AC-005 — IIN parses before IN — no prefix-match collision
(traces to BC-2.11.024 v1.1 invariant: "IIN requires at least one value in the membership
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
(traces to BC-2.11.024 v1.1 postcondition: "Each operator adds a case_insensitive: bool
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
(traces to BC-2.11.024 v1.1 postcondition: "case_insensitive: bool flag" on Predicate::In;
TD-VSDD-060 sibling-site sweep rule)

Same verification as AC-006 for `Predicate::In`:
```bash
rg 'Predicate::In\s*\{' crates/prism-query/src/ --type rust
```
Every construction site must explicitly specify `case_insensitive: false` (or `true` for IIN).

VERIFY-ONLY AC — compilation enforces.

### AC-008 — IEQ lowers to `lower(field) = lower('val')` in DataFusion SQL
(traces to BC-2.11.024 v1.1 postcondition "DataFusion SQL lowering" table — IEQ row:
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

### AC-009 — INE lowers to `lower(field) != lower('val')`
(traces to BC-2.11.024 v1.1 postcondition "DataFusion SQL lowering" table — INE row:
`lower(severity) != lower('low')`)

Given `Predicate::Compare { lhs: Box::new(Expr::Field(FieldPath::new(["severity"]))), op: CompareOp::Ne, rhs: Box::new(Expr::Literal(Literal::String("low".into()))), case_insensitive: true }` for `severity INE 'low'`,
when `predicate_to_datafusion_sql` processes it,
then the emitted string is `lower(severity) != lower('low')`.

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_ine_emits_lower_ne_lower`

### AC-010 — IIN lowers to `lower(field) IN (lower('v1'), lower('v2'), ...)`
(traces to BC-2.11.024 v1.1 postcondition "DataFusion SQL lowering" table — IIN row:
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
(traces to BC-2.11.024 v1.1 postcondition "Relationship to case-sensitive operators":
"The default operators =, !=, and IN are unchanged — they retain case-sensitive exact-match
semantics"; BC-2.11.024 v1.1 invariant: "IEQ is a strict superset of =: for any two values
with identical casing, IEQ matches if and only if = would match")

Given `Predicate::Compare { op: CompareOp::Eq, case_insensitive: false }` for `severity = 'High'`,
when `predicate_to_datafusion_sql` processes it,
then the emitted string is `severity = 'High'` — NO `lower()` wrapping applied.

Same for `!=` (case_insensitive: false → `severity != 'Low'`) and `IN` (case_insensitive: false).

NOTE: This is a regression guard — no behavioral change, verifying that the case_insensitive
flag is respected and the existing paths are NOT modified.

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_case_sensitive_eq_no_lower_wrapping`

### AC-012 — IEQ execution: matches rows regardless of casing
(traces to BC-2.11.024 v1.1 canonical test vector #1: `severity IEQ 'high'` against
`{severity: 'High'}` → Row returned;
test vector #2: `severity IEQ 'HIGH'` against `{severity: 'High'}` → Row returned;
BC-2.11.002 v1.5 amendment: IEQ operational in filter mode)

Given a DataFusion MemTable with one row `{severity: 'High'}`,
when the query `severity IEQ 'high'` is executed,
then the row is returned.

Given the same MemTable,
when `severity IEQ 'HIGH'` is executed,
then the row is also returned (both cases match via `lower()`).

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_execution_case_insensitive_match`

### AC-013 — Case-sensitive `=` returns 0 rows when casing differs (regression guard)
(traces to BC-2.11.024 v1.1 canonical test vector #6: "regression-no-change" —
`severity = 'High'` against `{severity: 'HIGH'}` → Row NOT returned;
BC-2.11.024 v1.1 invariant: "Precision differences appear only when casing differs")

Given a MemTable with `{severity: 'High'}`,
when `severity = 'high'` (case-sensitive) is executed,
then 0 rows are returned.

NOTE: This is the core regression guard. If this test fails after implementation, the
case-sensitive default has been broken — a critical behavioral regression.

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_case_sensitive_eq_returns_zero_on_casing_mismatch`

### AC-013b — IEQ/IIN available in pipe-mode | where stage
(traces to BC-2.11.024 v1.1 invariant: "IEQ/IIN/INE are valid in filter mode and in
pipe-mode | where stages (shared grammar invariant, BC-2.11.023)";
BC-2.11.004 v1.13 amendment)

Given `FROM crowdstrike_detections | where severity IEQ 'high' | head 5`,
when parsed and executed,
then the query succeeds and returns rows where `lower(severity) = lower('high')`.

NOTE: The shared filter grammar (ADR-046 D7) means IEQ/IIN/INE work in pipe-mode `| where`
automatically once the grammar changes land. This AC verifies no parser plumbing is
accidentally missing for the pipe path.

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_in_pipe_where_stage`

### AC-014 — normalized_pql reflects IEQ/IIN/INE in uppercase canonical form
(traces to BC-2.11.024 v1.1 postcondition "normalized_pql round-trip":
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
(traces to BC-2.11.024 v1.1 postcondition "normalized_pql round-trip":
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

### AC-016 — OCSF enum-label fields normalized to canonical Title-case at adapter boundary
(traces to BC-2.02.013 v1.2 postconditions:
"Before the DynamicMessage is populated (BC-2.02.002), every OCSF enum-label string field
in the normalized record is rewritten to its canonical OCSF Title-case casing from enum_map.rs";
"Severity (guaranteed): 'HIGH' → 'High', 'high' → 'High', 'CRITICAL' → 'Critical'";
"Status (guaranteed): normalized to OCSF Title-case";
BC-2.02.002 v1.5 amendment; BC-2.02.010 v1.5 amendment)

Given a sensor record with `severity='CRITICAL'` (all-caps) entering the prism-ocsf
normalization pipeline,
when the adapter-boundary normalization function processes it before DynamicMessage creation,
then the `DynamicMessage` produced has `severity='Critical'` (canonical OCSF Title-case).

Same for `severity='low'` → `'Low'`, `severity='MEDIUM'` → `'Medium'`, etc.
`enum_map.rs` is the sole source for canonical captions (BC-2.02.010 v1.5 invariant).

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_adapter_normalization_critical_to_title_case`
Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_adapter_normalization_low_to_title_case`

OCSF IN-SCOPE FIELDS NOTE (BC-2.02.013 v1.2): The four in-scope enum-label string fields
guaranteed by this story are: `severity`, `status`, `activity_name`, and `disposition`.
The OCSF string label for the activity dimension is `activity_name` (NOT `activity` —
`activity_name` is the OCSF-canonical field name per BC-2.02.013 v1.2). When reading
`enum_map.rs`, sensor TOML specs, or writing test fixtures, always use `activity_name` for
the activity string label column; `activity` refers to a different field (the raw numeric
activity_id context field, not the normalized string label).

### AC-017 — Normalization is idempotent: already-canonical values unchanged
(traces to BC-2.02.013 v1.2 postcondition:
"The normalization function is idempotent: if the field already contains the canonical-case
value (e.g., 'High'), the value is unchanged. Re-normalizing already-canonical data has no
effect"; EC-02-020: CrowdStrike adapter emits severity='High' (already canonical Title-case)
→ value unchanged; no warning)

Given CrowdStrike sensor data with `severity='High'` (already OCSF canonical Title-case),
when the normalization pipeline processes it,
then the `DynamicMessage` has `severity='High'` unchanged, and no warning is emitted.

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_adapter_normalization_idempotent_high`

### AC-018 — Unrecognized vendor values left as-received with warning logged
(traces to BC-2.02.013 v1.2 error cases:
"Warning (non-fatal): An OCSF enum-label field value has no matching caption in enum_map.rs";
EC-02-021: Armis adapter emits severity='UNHANDLED' (vendor-specific value) → value left
as-received, warning logged)

Given a sensor record with `severity='UNHANDLED'` (Armis vendor-specific value not in
OCSF severity captions in `enum_map.rs`),
when the normalization pipeline processes it,
then:
1. The `DynamicMessage` has `severity='UNHANDLED'` (value unchanged)
2. A `tracing::warn!` is emitted with field name, value, and sensor type
3. The normalization does NOT fail or return an error — it is non-fatal

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_adapter_normalization_unrecognized_value_left_as_received`

### AC-019 — GROUP BY severity produces at most 7 buckets after normalization (aggregation consistency)
(traces to BC-2.02.013 v1.2 canonical test vector:
"PrismQL GROUP BY severity across CrowdStrike + Armis after normalization: 'High' appears
as one bucket — not split into 'High' + 'HIGH'";
EC-02-026: Cross-sensor aggregation correct after normalization)

Given simulated sensor records:
- CrowdStrike: `severity='High'` (3 records)
- Armis-like: `severity='HIGH'` (2 records, pre-normalization value — becomes `'High'` after normalization)
- Any sensor: `severity='Critical'` (1 record)

when `GROUP BY severity` is executed after normalization,
then the `'High'` bucket contains 5 rows and the `'Critical'` bucket contains 1 row —
NOT two separate `'High'` (3) and `'HIGH'` (2) buckets.

This verifies that adapter-boundary normalization eliminates GROUP BY fragmentation (ADR-047
§Consequences positive: "GROUP BY severity produces correct aggregation across sensors").

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_group_by_severity_no_case_fragmentation`

### AC-020 — E-QUERY-001: IEQ/INE with non-string literal RHS rejected at parse time
(traces to BC-2.11.024 v1.1 error case: "E-QUERY-001: IEQ/INE with a non-string literal
on the RHS (e.g., severity IEQ 42) — Parse error: 'IEQ/INE require a string literal
as the right-hand side value'")

Given `severity IEQ 42` (integer literal on RHS),
when parsed,
then the result is `Err(E-QUERY-001)` with a message indicating IEQ requires a string literal.

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_non_string_rhs_e_query_001`

### AC-021 — E-QUERY-001: IIN with empty membership list rejected at parse time
(traces to BC-2.11.024 v1.1 error case: "E-QUERY-001: IIN with an empty membership list:
severity IIN () — Parse error: 'IIN requires at least one value in the membership list'";
BC-2.11.024 v1.1 invariant: "IIN requires at least one value in the membership list.
An empty IIN () list is a parse error (E-QUERY-001)")

Given `severity IIN ()`,
when parsed,
then the result is `Err(E-QUERY-001)` with a message indicating IIN requires at least one value.

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_iin_empty_list_e_query_001`

### AC-022 — E-QUERY-002: IEQ/IIN/INE on non-string column returns QueryTypeMismatch
(traces to BC-2.11.024 v1.1 error case:
"E-QUERY-002: IEQ/IIN/INE applied to a non-string column (e.g., severity_id IEQ 'high'
where severity_id is an integer column) — QueryTypeMismatch: not applicable to non-string
types; error includes field name, actual type, operator, and — when the column is a known
OCSF integer-id field — suggests the corresponding string label column";
BC-2.11.024 v1.1 precondition: "The field referenced by IEQ/IIN/INE is a string-type column
in the DataFusion execution schema. Applying lower() to a non-string column results in
E-QUERY-002 (QueryTypeMismatch)")

Given a DataFusion schema where `severity_id` is an integer column,
when `severity_id IEQ 'high'` is executed,
then the result is `Err(E-QUERY-002 QueryTypeMismatch)` with a Display message containing:
- the column name (`severity_id`), its actual type (`Integer`), and the operator (`IEQ`)
- the suggestion: "for label comparison, use the string column 'severity' with IEQ/IIN/INE
  instead" (because `severity_id` is a known OCSF integer-id field; `PrismError::QueryTypeMismatch`
  carries `suggested_column: Some("severity")` per the OCSF sibling lookup contract in
  error-taxonomy.md v2.19 §E-QUERY-002)

The full Display for this case must be:
  "E-QUERY-002: type mismatch — column 'severity_id' in table '<table>' has type 'Integer'
   which does not support operator 'IEQ'; for label comparison, use the string column
   'severity' with IEQ/IIN/INE instead"

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_integer_column_e_query_002`

### AC-023 — SQL-mode IEQ/IIN/INE rejection — structured E-QUERY-001
(traces to BC-2.11.024 v1.1 §Mode-Boundary Enforcement invariant)

Given a PrismQL query in raw SQL mode (begins with `SELECT`) that contains an IEQ, IIN, or INE
operator (e.g., `SELECT * FROM t WHERE severity IEQ 'high'`),
when the parser processes it,
then the parser MUST reject it at PARSE TIME with `E-QUERY-001` BEFORE any DataFusion
planning or execution, with the verbatim message:
`"E-QUERY-001: parse error near '{operator}': case-insensitive operators (IEQ/IIN/INE) are not supported in SQL mode. Use filter mode (e.g., severity IEQ 'high') or a pipe | where stage (e.g., FROM crowdstrike_detections | where severity IEQ 'high') instead."`
where `{operator}` is the encountered keyword in uppercase (IEQ, IIN, or INE).

MCP error mapping: -32602 INVALID_PARAMS.

The rejection MUST NOT produce `E-QUERY-034 QueryExecutionFailed` — that would indicate the
query reached DataFusion execution before rejection. The error code MUST be `E-QUERY-001`.

Regression vector: existing filter-mode AC-001 (`severity IEQ 'high'`) and pipe-mode AC-013b
(`FROM t | where severity IEQ 'high'`) Red Gate tests confirm no regression from the SQL-mode gate.

Red Gate: `test_BC_2_11_024_sql_mode_ieq_rejected`
Red Gate: `test_BC_2_11_024_sql_mode_iin_rejected`
Red Gate: `test_BC_2_11_024_sql_mode_ine_rejected`

### AC-024 — PrismQL grammar reference resource includes IEQ/IIN/INE in operator table
(traces to BC-2.11.024 v1.1 architecture anchor: ADR-047 §D.4 discoverability;
BC-2.11.002 v1.5 amendment: IEQ/IIN/INE in the operator table;
"IEQ/IIN must be reflected in the PrismQL grammar reference resource (governed by
BC-2.11.022/ADR-045 parity gate)")

Given the auto-generated PrismQL grammar reference MCP resource (BC-2.11.022 / ADR-045),
when the grammar resource parity gate is run,
then `IEQ`, `IIN`, and `INE` appear in the operator table, and the ADR-045 parity gate
CI check passes.

NOTE: The implementer must locate the grammar resource generation code and add the three
new operators. The ADR-045 parity gate is a CI check — the PR must not break it.

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_grammar_resource_includes_ieq_iin_ine`

### AC-025 — prism describe output includes IEQ example with OCSF casing note
(traces to ADR-047 §D.4: "The prism describe / tool-schema pedagogical examples (ADR-041
L1/L2 teaching surface)" — "Include the OCSF casing note: 'OCSF severity is stored as
Title-case (High). Use IEQ/IIN to match regardless of the case you type, or = 'High' for
the exact canonical form'")

Given the `prism describe <table>` command output for any sensor table with a `severity`
column,
when the output is inspected,
then it contains at least one example query using `IEQ` with a severity value, AND includes
the OCSF casing note (pedagogical hint that severity is Title-case).

NOTE: This may be a snapshot/golden-file test or a content-inclusion test. The implementer
must decide the appropriate test mechanism for the describe output.

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_describe_output_includes_ieq_example`

### AC-026 — No panic: IEQ/IIN expressions with multiple predicates do not panic (VP-021 regression)
(traces to BC-2.11.024 v1.1 canonical test vector: "severity IEQ 'high' AND severity IEQ 'high'
does not panic (fuzz-seed regression)" — design map T-CASE-009;
VP-021 invariant: parser never panics on arbitrary input)

Given `severity IEQ 'high' AND severity IEQ 'high'` (repeated IEQ predicate),
when parsed and processed,
then the result is either a valid query result or a structured error — NOT a panic.

This is a regression guard targeting the VP-021 fuzz property. The existing `vp021_parse_fuzz`
target in `fuzz/` covers this class of input; this unit test pins the specific fuzz seed.

Red Gate: `test_S_PRISMQL_CASE_INSENSITIVE_001_repeated_ieq_no_panic`

### AC-027 — Non-exhaustive compile-fail gate count UNCHANGED at 89
(traces to BC-2.11.024 v1.1 invariant:
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
| RG-019 | `test_S_PRISMQL_CASE_INSENSITIVE_001_adapter_normalization_critical_to_title_case` | `crates/prism-ocsf/src/tests/` (or prism-spec-engine) | AC-016 | `severity='CRITICAL'` → `'Critical'` in DynamicMessage |
| RG-020 | `test_S_PRISMQL_CASE_INSENSITIVE_001_adapter_normalization_idempotent_high` | `crates/prism-ocsf/src/tests/` (or prism-spec-engine) | AC-017 | `severity='High'` unchanged; no warning emitted |
| RG-021 | `test_S_PRISMQL_CASE_INSENSITIVE_001_adapter_normalization_unrecognized_value_left_as_received` | `crates/prism-ocsf/src/tests/` (or prism-spec-engine) | AC-018 | `severity='UNHANDLED'` unchanged; warning emitted |
| RG-022 | `test_S_PRISMQL_CASE_INSENSITIVE_001_group_by_severity_no_case_fragmentation` | `crates/prism-query/src/tests/` | AC-019 | Multi-sensor GROUP BY produces 1 'High' bucket, not 2 fragmented buckets |
| RG-023 | `test_BC_2_11_024_sql_mode_ieq_rejected` | `crates/prism-query/src/tests/` | AC-023 | SQL-mode `SELECT … WHERE severity IEQ 'high'` → Err(E-QUERY-001); NOT QueryExecutionFailed/E-QUERY-034 |
| RG-024 | `test_BC_2_11_024_sql_mode_iin_rejected` | `crates/prism-query/src/tests/` | AC-023 | SQL-mode `SELECT … WHERE status IIN ('open', 'new')` → Err(E-QUERY-001) |
| RG-025 | `test_BC_2_11_024_sql_mode_ine_rejected` | `crates/prism-query/src/tests/` | AC-023 | SQL-mode `SELECT … WHERE severity INE 'low'` → Err(E-QUERY-001) |

**Additional Red Gate stubs (grammar resource + describe output):**

| RG ID | Test Function Name | Location | AC |
|-------|--------------------|----------|----|
| RG-026 (grammar resource) | `test_S_PRISMQL_CASE_INSENSITIVE_001_grammar_resource_includes_ieq_iin_ine` | `crates/prism-query/src/tests/` or `crates/prism-mcp/src/tests/` | AC-024 |
| RG-027 (describe output) | `test_S_PRISMQL_CASE_INSENSITIVE_001_describe_output_includes_ieq_example` | `crates/prism-mcp/src/tests/` or `crates/prism-query/src/tests/` | AC-025 |

NOTE: RG-026 and RG-027 are implementation-detail-dependent — the implementer must choose
the appropriate test location based on where the grammar resource generation and describe
output code live.

**Total Red Gate tests: 27 (25 core + 2 discoverability)**

The story frontmatter records `red_gate_tests: 27` (total: 25 core + 2 discoverability). The discoverability tests
RG-026/RG-027 may be snapshot or integration tests; include them if they can be written as
failing stubs, otherwise verify the parity gate via `just check` at end of implementation.

---

## Architecture Mapping

| Component | Crate | Files | Pure/Effectful |
|-----------|-------|-------|---------------|
| Grammar (IEQ/IIN/INE keywords) | prism-query | `src/filter_parser.rs` | Pure |
| AST extensions (case_insensitive flag) | prism-query | `src/ast.rs` | Pure |
| DataFusion SQL emitter (lower() lowering) | prism-query | `src/pipe_sql_emitter.rs` | Pure |
| PQL round-trip normalizer | prism-query | `src/ast.rs` — `PqlNormalizer::normalize_predicate` (the `Predicate::Compare` and `Predicate::In` match arms) | Pure |
| OCSF enum-label canonical-case normalization | prism-ocsf | `src/enum_map.rs` (`OcsfEnumMap`) + `src/normalizer.rs` (`OcsfNormalizer::normalize_with_mappers`) and/or `src/mappers/spec_driven.rs` | Pure (lookup-and-rewrite, no I/O) |
| Grammar resource MCP resource | prism-mcp | `src/resources.rs` (or grammar generation code) | Effectful (MCP resource emission) |
| prism describe pedagogical examples | prism-query / prism-mcp | grammar describe output | Effectful (MCP tool response) |

**Architecture section references:**
- `architecture/decisions/ADR-047` §D.2 (IEQ/IIN/INE syntax, lower() lowering rationale)
- `architecture/decisions/ADR-047` §D.3 (adapter normalization, enum_map.rs authority)
- `architecture/module-decomposition.md` §SS-11 (Query Execution — prism-query)
- `architecture/module-decomposition.md` §SS-02 (OCSF Normalization — prism-ocsf)

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
| `crates/prism-ocsf/src/normalizer.rs` and/or `crates/prism-ocsf/src/mappers/spec_driven.rs` | Add the canonical-case rewrite that looks up `OcsfEnumMap` and rewrites enum-label fields BEFORE the `DynamicMessage` field is populated (`OcsfNormalizer::normalize_with_mappers` is the DynamicMessage-creation site). This is the DEFINITIVE location (verified — see Scope Ambiguity 1). |
| `crates/prism-spec-engine/src/` | NOT modified for OCSF normalization (verified: zero `DynamicMessage` references). Retained in `crates_touched` only as a defensive allowance in case SAP-2 fixture/DTU-parity fallout requires a touch; if no prism-spec-engine change is made, note it and consider removing it from `crates_touched` at PR time (flag for product-owner/state-manager). |
| `crates/prism-mcp/src/resources.rs` (or equiv.) | Add IEQ/IIN/INE to grammar reference resource operator table; add OCSF casing note to prism describe examples |
| DTU fixture generators (prism-dtu-*/src/generator.rs or types.rs) | Update any test assertions or fixture values that assert pre-normalization UPPER-case severity/status strings to use canonical Title-case. Per SAP-2, adversary will verify TOML column parity with DTU types.rs |

### Files to CREATE:

| File | Purpose |
|------|---------|
| `crates/prism-query/src/tests/test_case_insensitive_operators.rs` (or equivalent test module) | Red Gate test file for RG-001 through RG-018, RG-023 |
| `crates/prism-ocsf/src/tests/test_adapter_normalization.rs` (or equivalent) | Red Gate tests RG-019, RG-020, RG-021 for adapter normalization |

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

8. **Read** `crates/prism-ocsf/src/` to ground yourself in the normalization pipeline. The
   crate ownership question is ALREADY RESOLVED (see "Scope Ambiguities Resolved → Ambiguity 1"):
   **the OCSF enum-label normalization lives in `prism-ocsf`, NOT prism-spec-engine.** Read:
   - `crates/prism-ocsf/src/enum_map.rs` — `OcsfEnumMap`, the canonical caption authority.
   - `crates/prism-ocsf/src/normalizer.rs` — `OcsfNormalizer::normalize_with_mappers`, which
     creates the `DynamicMessage` (BC-2.02.002) and dispatches to a `SensorMapper`.
   - `crates/prism-ocsf/src/mappers/spec_driven.rs` — the config-driven sensor mapper (the path
     the CrowdStrike/Cyberint/Claroty/Armis TOML specs flow through).
   Insert the canonical-case rewrite so it runs BEFORE the OCSF enum-label field is populated on
   the `DynamicMessage`. Do NOT search prism-spec-engine for the normalizer — it does not create
   `DynamicMessage` (verified: zero `DynamicMessage` references in that crate).

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

14b. **Implement SQL-mode rejection gate (AC-023 / BC-2.11.024 v1.1 §Mode-Boundary Enforcement):**

    In the prism-query parser entry point (the top-level dispatch that determines whether the
    input is a filter-mode expression, a pipe query, or a raw SQL SELECT), add a check:
    if the query begins with `SELECT` AND the query contains any IEQ/IIN/INE keyword, return
    `E-QUERY-001` at PARSE TIME with the verbatim message from BC-2.11.024 v1.1:
    ```
    "E-QUERY-001: parse error near '{operator}': case-insensitive operators (IEQ/IIN/INE)
    are not supported in SQL mode. Use filter mode (e.g., severity IEQ 'high') or a pipe
    | where stage (e.g., FROM crowdstrike_detections | where severity IEQ 'high') instead."
    ```
    where `{operator}` is the first IEQ/IIN/INE keyword encountered (uppercased).

    The rejection MUST occur before DataFusion planning/execution to guarantee `E-QUERY-001`
    is returned and NOT `E-QUERY-034 QueryExecutionFailed`.

    After implementation, verify RG-023, RG-024, RG-025 pass and AC-001/AC-013b do NOT regress:
    ```bash
    cargo nextest run -p prism-query -E 'test(sql_mode)'
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

19. **Implement** the OCSF enum-label canonical-case normalization in the **prism-ocsf**
    normalization pipeline (crate ownership resolved — see Task 8 / Scope Ambiguity 1):

    a. **Read** `crates/prism-ocsf/src/enum_map.rs` to understand the existing caption map
       structure (e.g., `severity_id: 4 → "High"`, `5 → "Critical"`).

    b. **Create or extend** a normalization function:

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
    NOTE: the concrete type is `OcsfEnumMap` (`crates/prism-ocsf/src/enum_map.rs`, re-exported from
    `lib.rs`). `lookup_case_insensitive` is an ILLUSTRATIVE method name — read `enum_map.rs` at
    Task 19a to use (or add, with a Red Gate test) the real lookup API; do not assume this exact
    signature exists.

    c. **Wire** the normalization function into the pipeline step that sets OCSF string fields,
       BEFORE DynamicMessage creation (BC-2.02.002 v1.5 amendment + BC-2.02.013 invariant).

    d. **Emit** `tracing::warn!(event_type = "ocsf.enum_label_unrecognized", field_name = ...,
       value = ..., sensor_type = ...)` for unrecognized values (AC-018, BC-2.02.013 error case).
       Per SAP-1: add a corresponding row to BC-2.16.002 Canonical Structured Event Catalog for
       this new `event_type` in the SAME commit that adds the tracing emission.

20. **Update DTU test vectors** that assert pre-normalization UPPER-case severity/status values.
    Per SAP-2: run `rg 'severity.*HIGH\|status.*OPEN' crates/prism-dtu-*/src/ --type rust`
    and update all assertions to use canonical Title-case form:
    - `severity == "HIGH"` → `severity == "High"`
    - `status == "OPEN"` → `status == "Open"` (if status 'OPEN' maps to OCSF canonical 'Open')
    - `severity == "UNHANDLED"` → remains `"UNHANDLED"` (vendor-specific, not normalized)

21. **Verify** RG-019, RG-020, RG-021 (adapter normalization tests) and RG-022 (GROUP BY no fragmentation) pass.

### Phase 8 — Discoverability (Track F)

22. **Add** IEQ/IIN/INE to the auto-generated PrismQL grammar reference MCP resource.
    Read the grammar resource generation code (BC-2.11.022 / ADR-045) and add the three
    new operators to the operator table. Run the ADR-045 parity gate:
    ```bash
    just check  # includes ADR-045 parity gate
    ```

23. **Add** IEQ example with OCSF casing note to `prism describe` output.
    The example must follow the text specified in design map §F:
    ```
    -- Case-insensitive equality (for OCSF enum-label fields)
    SELECT * FROM crowdstrike_detections WHERE severity IEQ 'high'
    ```
    With note: "OCSF severity is stored as Title-case ('High'). Use IEQ/IIN to match regardless
    of the case you type, or = 'High' for the exact canonical form."

24. **Verify** RG-026, RG-027 (grammar resource and describe tests) pass.

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
    just iter prism-query       # all prism-query tests
    just iter prism-ocsf        # adapter normalization lives in prism-ocsf (resolved at Task 8)
    just check                  # full workspace pre-push gate
    ```
    All 27 Red Gate tests must pass. All existing tests must continue to pass (especially
    existing =, != filter tests — regression guard for AC-011/AC-013).

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

- **crates_touched declaration:** FIDELITY-REMEDIATION established the pattern of declaring
  `crates_touched` explicitly. This story touches prism-query, prism-ocsf, prism-spec-engine
  (verify at Task 8), prism-mcp. Adversary will check for undeclared crate changes.

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

**Ambiguity 1: Adapter normalization in prism-ocsf vs prism-spec-engine — RESOLVED (remove-uncertainty pass-2, 2026-07-06).**
The design map (§E) names `crates/prism-ocsf/src/` as the location. The STORY-INDEX stub named
`crates/prism-spec-engine/` as the adapter-boundary crate.
DEFINITIVE RESOLUTION: **The OCSF enum-label normalization pipeline lives in `prism-ocsf`.**
This was verified directly against the codebase (source of truth): `DynamicMessage` creation
(BC-2.02.002) occurs ONLY in `prism-ocsf` — `crates/prism-ocsf/src/normalizer.rs`
(`OcsfNormalizer::normalize_with_mappers`, which calls `DynamicMessage::new(descriptor)`),
`crates/prism-ocsf/src/mappers/spec_driven.rs` (the config-driven sensor mapper for the
CrowdStrike/Cyberint/Claroty/Armis TOML specs), and `crates/prism-ocsf/src/event.rs`. The
canonical caption authority `OcsfEnumMap` is at `crates/prism-ocsf/src/enum_map.rs` (re-exported
from `lib.rs` as `pub use enum_map::OcsfEnumMap`). A workspace scan found ZERO `DynamicMessage`
references in `prism-spec-engine` — its "normalization" surfaces (`pipeline.rs`, `datetime.rs`,
`column_mapping.rs`) handle spec parsing, datetime coercion, and column mapping, NOT OCSF
protobuf message construction. The STORY-INDEX stub was WRONG.
IMPLEMENTER DIRECTION: implement the canonical-case enum-label rewrite in `prism-ocsf` — insert
it in the `normalizer.rs` normalize path and/or the `spec_driven.rs` mapper so it runs BEFORE the
`DynamicMessage` field is populated (BC-2.02.002 v1.5), using `OcsfEnumMap` from `enum_map.rs`.
`prism-spec-engine` does NOT require modification for the OCSF normalization itself. (`crates_touched`
still lists `prism-spec-engine` — see the note under "File Structure Requirements"; the only
possible prism-spec-engine/DTU touch is fixture/assertion updates per SAP-2, not the normalizer.)
The adversary's SAP-2 probe verifies TOML↔DTU parity regardless.

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
| v1.0 | 2026-07-06 | Initial story decomposition |
| v1.1 | 2026-07-06 | remove-uncertainty pass-1: AST shape verified against ast.rs; CODE-SHAPE NOTEs added to AC-001/AC-002/AC-003/AC-008/AC-010; Scope Ambiguity 1 added (prism-ocsf vs prism-spec-engine pending verification); Ambiguity 2/3 added; File Structure Requirements expanded; IIN-before-IN collision note refined |
| v1.2 | 2026-07-06 | remove-uncertainty pass-2: tdd_mode rationale comment corrected — OCSF adapter-boundary normalization is definitively in prism-ocsf (OcsfNormalizer + OcsfEnumMap), NOT prism-spec-engine (zero DynamicMessage references); Scope Ambiguity 1 closed with DEFINITIVE RESOLUTION; TD-VSDD-091 anchor de-pinning applied (line-number citations removed, replaced with anchor-based references); crates_touched confirmed: prism-query (operators), prism-ocsf (OCSF normalization, primary), prism-spec-engine (defensive allowance, may be removed at PR time if untouched) |
| v1.3 | 2026-07-06 | LOCAL pass-1 fix-burst: BC-2.02.013 v1.0→v1.1 + BC-2.16.002 pin→v1.98 propagation (BC-2.02.013 now concretely specifies in-scope field set severity/status/activity_name/disposition, keying contract, insertion point, warn event; BC-2.16.002 v1.98 added catalog row 91 ocsf.enum_label_unrecognized — note: BC-2.16.002 has no prior version pin in this story, reference sites unchanged per no-AC-text rule) |
| v1.4 | 2026-07-06 | LOCAL pass-2 fix-burst: AC-022 reworded (E-QUERY-002 suggested_column enrichment — PrismError::QueryTypeMismatch carries suggested_column: Some("severity"), full Display format specified, error-taxonomy.md v2.18 §E-QUERY-002 pin added); activity→activity_name correction in File Structure Requirements and v1.3 changelog entry; in-scope OCSF fields note added to AC-016 area (severity/status/activity_name/disposition); BC-2.02.013 v1.1→v1.2 at all 7 pin sites (frontmatter comment, Behavioral Contracts table, Token Budget table, AC-016/017/018/019 traces); BC-2.16.002→v1.99 deferred (no version pins present in story body per v1.3 note); error-taxonomy v2.18 pin added in new AC-022 text only |
| v1.5 | 2026-07-07 | LOCAL pass-3 fix-burst: AC-023 SQL-mode IEQ/IIN/INE rejection (E-QUERY-001) added; old AC-023 (grammar resource) → AC-024; old AC-024 (describe) → AC-025; old AC-025 (no panic) → AC-026; old AC-026 (non-exhaustive) → AC-027; RG-023/024/025 (SQL-mode rejection tests test_BC_2_11_024_sql_mode_*) added to main Red Gate table; old RG-023/024 renumbered to RG-026/027 with AC refs updated to AC-024/025; red_gate_tests 25→27; Behavioral Contracts table BC-2.11.024 v1.0→v1.1 + Mode-Boundary Enforcement added to Key Clauses Used; error-taxonomy v2.18→v2.19 |
| v1.6 | 2026-07-07 | pass-4 fix-burst: RG-015 AC back-ref corrected AC-025→AC-026; full bidirectional RG↔AC traceability sweep — also found and corrected two stale "AC-026" prose refs in Task 25 and Architecture Compliance Rule 5 (non-exhaustive gate is AC-027, not AC-026, after v1.5 renumbering) |
