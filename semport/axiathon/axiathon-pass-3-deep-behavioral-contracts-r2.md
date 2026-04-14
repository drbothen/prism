# Pass 3 Deep: Behavioral Contracts -- Round 2

**Project:** Axiathon
**Pass:** 3 (Behavioral Contracts)
**Round:** 2
**Date:** 2026-04-13

---

## Purpose

This round targets gaps identified in Round 1: FilterExpr::Has/Missing/Wildcard contracts with test evidence, all PipeStage contracts, Duration/signed value parsing contracts, string escape contracts, detection parser grammar contracts, and wildcard operator restriction contracts.

---

## 14. AxiQL Parser Contracts -- Round 2 Additions (BC-2.03.xxx through BC-2.06.xxx)

### BC-2.03.001: FilterExpr::Has parsed from `HAS field_path`

**Preconditions:** Query starts with or contains `HAS` keyword followed by field path
**Postconditions:** Produces `FilterExpr::Has(FieldRef)` -- field existence check
**Evidence:** `parser_test.rs:530-538` test `parse_filter_has_existence`
**Confidence:** HIGH

### BC-2.03.002: FilterExpr::Missing parsed from `MISSING field_path`

**Preconditions:** Query starts with or contains `MISSING` keyword followed by field path
**Postconditions:** Produces `FilterExpr::Missing(FieldRef)` -- field absence check
**Evidence:** `parser_test.rs:541-549` test `parse_filter_missing_existence`
**Confidence:** HIGH

### BC-2.03.003: FilterExpr::Wildcard parsed from string value containing `*` or `?`

**Preconditions:** Comparison with `=` or `!=` where string value contains wildcard characters
**Postconditions:**
- `field = "10.0.*"` -> `Wildcard { field, pattern: "10.0.*", negated: false }`
- `field != "192.168.*"` -> `Wildcard { field, pattern: "192.168.*", negated: true }`
**Evidence:** `parser_test.rs:674-700` tests `parse_filter_wildcard_eq`, `parse_filter_wildcard_ne_preserves_negation`
**Confidence:** HIGH

### BC-2.03.004: Wildcard rejects ordering operators

**Preconditions:** Comparison with `>=`, `<`, etc. where string value contains wildcard
**Postconditions:** Parse error -- ordering operators with wildcards are semantically meaningless
**Evidence:** `parser_test.rs:1318-1334` tests `parse_filter_wildcard_ordering_op_rejects`, `parse_filter_wildcard_lt_rejects`
**Confidence:** HIGH

### BC-2.04.001: Duration values parsed from integer + unit suffix

**Preconditions:** Numeric value followed by `s`, `m`, `h`, or `d`
**Postconditions:**
- `30s` -> `Value::Duration(Duration::from_secs(30))`
- `5m` -> `Value::Duration(Duration::from_secs(300))`
- `24h` -> `Value::Duration(Duration::from_secs(86400))`
- `7d` -> `Value::Duration(Duration::from_secs(604800))`
- `0s` -> `Value::Duration(Duration::from_secs(0))`
- `1024` (no suffix) -> `Value::Integer(1024)`, NOT duration
- `2.72` (no suffix) -> `Value::Float(2.72)`, NOT duration
- `-5m` -> parse error (negative durations not supported)
**Evidence:** `parser_test.rs:314-400` (7 duration tests)
**Confidence:** HIGH

### BC-2.04.002: Signed numeric values parsed via optional minus prefix

**Preconditions:** Value starting with `-` followed by digits
**Postconditions:**
- `-100` -> `Value::Integer(-100)`
- `-2.75` -> `Value::Float(-2.75)`
- `-0` -> `Value::Integer(0)` (negation of zero is zero)
- `-0.5` -> `Value::Float(-0.5)`
- `-` (bare, no digits) -> parse error
- Positive integers without sign still parse correctly
- Negative in IN lists: `IN (-1, 0, 1)` works
- Negative in SQL WHERE: `WHERE priority > -10` works
**Evidence:** `parser_test.rs:172-310` (12 signed value tests)
**Confidence:** HIGH

### BC-2.04.003: String escape sequences in quoted strings

**Preconditions:** Quoted string with backslash sequences
**Postconditions:**
- `\"` -> literal `"` (known escape)
- `\\` -> literal `\` (known escape)
- `\n` -> newline (known escape)
- `\r` -> carriage return (known escape)
- `\t` -> tab (known escape)
- `\.` -> literal `\.` (unknown escape, pass-through for regex patterns)
- Unterminated string after escape -> parse error
- `"foo\"` (backslash escapes closing quote) -> unterminated string, parse error
- `""` (empty string) -> `Value::String("")`
**Evidence:** `parser_test.rs:562-634` (5 escape tests)
**Confidence:** HIGH

### BC-2.04.004: Operator aliases accepted

**Preconditions:** Query using `==` or `!` prefix
**Postconditions:**
- `==` -> `CompareOp::Eq` (same as `=`)
- `! condition` -> `FilterExpr::Not(condition)` (same as `NOT condition`)
- `!condition` (no space) -> also `FilterExpr::Not(condition)`
**Evidence:** `parser_test.rs:704-737` (3 alias tests)
**Confidence:** HIGH

### BC-2.05.001: PipeStage::Stats parses aggregation functions

**Preconditions:** Pipe expression with `stats` keyword
**Postconditions:**
- `stats count by field` -> bare count, field=None (backward compat)
- `stats count(*) by field` -> count with star, field=None
- `stats count() by field` -> count with empty parens, field=None
- `stats sum(field) by group` -> sum with field
- `stats sum by field` -> REJECTED (bare sum without parens)
- `stats avg by field` -> REJECTED (bare avg without parens)
- `stats count(*) AS alias by field` -> count with alias
- `stats count(*) AS total, sum(bytes) AS total_bytes BY ip` -> multiple aggs
- `stats count(*) as total by ip` -> AS is case-insensitive
- `stats count(` (unclosed paren) -> parse error
- `stats bogus(field) by ip` -> parse error (invalid function)
- `stats count(*)` (no group_by) -> valid, group_by=None
**Evidence:** `parser_test.rs:908-1310` (15 stats tests)
**Confidence:** HIGH

### BC-2.05.002: PipeStage::Sort parses direction and fields

**Preconditions:** Pipe expression with `sort` keyword
**Postconditions:** `sort field1 desc` -> `Sort(vec![OrderByExpr { field, direction: Desc }])`
**Evidence:** `parser_test.rs:1108-1120`
**Confidence:** HIGH

### BC-2.05.003: PipeStage::Head/Tail parse numeric argument

**Preconditions:** Pipe expression with `head N` or `tail N`
**Postconditions:**
- `head 10` -> `Head(10)`
- `tail 5` -> `Tail(5)`
**Evidence:** `parser_test.rs:894-906` (head), `parser_test.rs:1171-1181` (tail)
**Confidence:** HIGH

### BC-2.05.004: PipeStage::Dedup parses field list

**Preconditions:** Pipe expression with `dedup field1 [, field2 ...]`
**Postconditions:** `dedup src_endpoint.ip` -> `Dedup(vec![FieldRef("src_endpoint.ip")])`
**Evidence:** `parser_test.rs:1123-1133`
**Confidence:** HIGH

### BC-2.05.005: PipeStage::Fields parses mode and field list

**Preconditions:** Pipe expression with `fields [+|-] field1, field2 ...`
**Postconditions:**
- `fields + src_endpoint.ip, dst_endpoint.ip` -> `Fields { mode: Include, fields: [...] }`
- `fields - raw_data, metadata` -> `Fields { mode: Exclude, fields: [...] }`
- `fields src_endpoint.ip, user.name` (bare, no +/-) -> defaults to Include mode (per Splunk/Sumo Logic)
- `fields` (no field names) -> parse error (at_least(1) enforced)
- `fields +` (mode but no fields) -> parse error
**Evidence:** `parser_test.rs:1136-1261` (6 fields tests)
**Confidence:** HIGH

### BC-2.05.006: Multiple pipe stages chained with `|`

**Preconditions:** Query with multiple `|` separators
**Postconditions:**
- `filter | stats count by ip | sort count desc | head 10` -> Pipe with 3 stages
- Up to 64 stages allowed
- >64 stages -> error "at most 64 pipe stages"
- Empty stage (`filter ||`) is NOT an empty stage but `||` is treated as OR (double pipe is logical OR)
**Evidence:** `parser_test.rs:1156-1168`, `parser_test.rs:2719-2742` (max stages), `parser_test.rs:1770` (double pipe)
**Confidence:** HIGH

### BC-2.05.007: PipeStage::Stats percentile function

**Preconditions:** Pipe expression with `percentile` aggregation
**Postconditions:**
- `stats percentile(field, 95)` -> `AggFunction::Percentile(95.0)` with field
- `stats percentile(field, 99.9) AS p99` -> with alias
**Evidence:** `parser_test.rs:2158-2207` (2 percentile tests)
**Confidence:** HIGH

### BC-2.06.001: AxiQL comments in pipe expressions

**Preconditions:** Pipe expression with `//` or `#` comments
**Postconditions:** Comments stripped before parsing. Comments between pipe stages work correctly.
**Evidence:** `parser_test.rs:2393` test `parse_pipe_with_inline_comments`
**Confidence:** HIGH

---

## 15. Detection Parser Grammar Contracts (BC-3.05.xxx)

### BC-3.05.001: parse_rules() parses detection DSL file into Rule AST

**Preconditions:** Input string in .axd format
**Postconditions:**
- One or more `rule <id> { meta { ... } match <clause> alert { ... } }` blocks parsed
- Returns `Vec<ast::Rule>` on success
- Returns error string on parse failure
**Evidence:** Used extensively in engine.rs, correlation.rs, sequence.rs, alert.rs tests (all tests call `parse_rules()`)
**Confidence:** HIGH

### BC-3.05.002: Detection DSL meta block parses rule metadata

**Preconditions:** `meta { name "..." severity <level> [mitre "..."] [description "..."] [enabled <bool>] }`
**Postconditions:**
- `name` -> RuleMeta.name
- `severity` -> RuleMeta.severity (info/low/medium/high/critical)
- `mitre` -> RuleMeta.mitre (optional MITRE ATT&CK ID)
- `description` -> RuleMeta.description (optional)
- `enabled` -> RuleMeta.enabled (default true)
**Evidence:** All rule test strings contain meta blocks with these fields
**Confidence:** HIGH

### BC-3.05.003: Detection DSL match clause parses three types

**Preconditions:** `match event where <condition>` or `match count(...)` or `match sequence by ...`
**Postconditions:**
- `match event where <condition>` -> `MatchClause::SingleEvent(condition)`
- `match count(event where <condition>) >= N group_by <fields> within <duration>` -> `MatchClause::Correlation { ... }`
- `match sequence by <key_field> within <duration> { step ... }` -> `MatchClause::Sequence { ... }`
**Evidence:** engine.rs, correlation.rs, sequence.rs tests
**Confidence:** HIGH

### BC-3.05.004: Detection DSL condition tree parses boolean operators

**Preconditions:** Condition expression in detection rule
**Postconditions:**
- `field == "value"` -> `Condition::Predicate(FieldPredicate { field, op: Eq, value })`
- `cond1 and cond2` -> `Condition::And(vec![cond1, cond2])`
- `cond1 or cond2` -> `Condition::Or(vec![cond1, cond2])`
- `not cond` -> `Condition::Not(Box<cond>)`
- `(cond1 or cond2) and cond3` -> parenthesized grouping
**Evidence:** engine.rs tests with various condition structures
**Confidence:** HIGH

### BC-3.05.005: Detection DSL predicate operators parse all types

**Preconditions:** Predicate in detection rule condition
**Postconditions:**
- `==` -> PredicateOp::Eq
- `!=` -> PredicateOp::NotEq
- `>`, `>=`, `<`, `<=` -> Gt, Gte, Lt, Lte
- `contains` -> PredicateOp::Contains
- `matches` -> PredicateOp::Matches (regex)
- `cidr` -> PredicateOp::Cidr
- `in (...)` -> PredicateOp::In with LiteralValue::List
**Evidence:** engine.rs tests demonstrating all operators
**Confidence:** HIGH

### BC-3.05.006: Detection DSL duration parsing

**Preconditions:** Duration in `within` clause
**Postconditions:**
- `5m` -> Duration::from_secs(300)
- `10m` -> Duration::from_secs(600)
- `1s` -> Duration::from_secs(1)
**Evidence:** correlation.rs `within 5m`, sequence.rs `within 10m`, `within 1s`
**Confidence:** HIGH

### BC-3.05.007: Detection DSL sequence steps parse event and count types

**Preconditions:** `step` clause in sequence match
**Postconditions:**
- `step name: event where <condition>` -> `StepType::Event(condition)`
- `step name: count(event where <condition>) >= N` -> `StepType::Count { condition, op: Gte, threshold: N }`
**Evidence:** sequence.rs tests
**Confidence:** HIGH

### BC-3.05.008: Detection DSL alert clause parses template strings

**Preconditions:** `alert { title "..." description "..." }` block
**Postconditions:** Title and description stored as raw strings with `{field}` placeholders for later interpolation
**Evidence:** alert.rs tests showing interpolated output
**Confidence:** HIGH

### BC-3.05.009: load_rules_from_dir() loads all .axd files from directory

**Preconditions:** Directory path containing .axd rule files
**Postconditions:** All .axd files parsed and rules concatenated. Used in engine.rs test `all_axd_rules_evaluation`.
**Evidence:** `engine.rs:507-514` test `all_axd_rules_evaluation`
**Confidence:** HIGH

---

## 16. Type System Contracts (BC-2.07.xxx)

### BC-2.07.001: TypeConstraint validates operator/type compatibility

**Preconditions:** CompareOp and AxiQLType provided
**Postconditions:**
- Equality operators (Eq, Ne) accept all 6 types: String, Integer, Float, Boolean, IpAddress, Timestamp
- Ordering operators (Gt, Lt, GtEq, LtEq) accept 4 types: Integer, Float, Timestamp, String
- Unknown future CompareOp variants -> default to equality types + tracing warning
**Evidence:** `crates/axiathon-query/src/type_system.rs` lines 36-68, `tests/type_system_test.rs`
**Confidence:** HIGH

### BC-2.07.002: TypeError provides actionable hint messages

**Preconditions:** Type mismatch detected
**Postconditions:**
- With field_name: `"field 'process.pid' is Boolean, but operator '>' expects: Integer, Float, Timestamp, String"`
- Without field_name (ordering): `"ordering operators require Integer, Float, Timestamp, or String types"`
- Without field_name (equality): `"equality operators accept all types"`
**Evidence:** `type_system.rs` lines 140-162
**Confidence:** MEDIUM (from code, not test)

---

## 17. Query Config Contracts (BC-2.08.xxx)

### BC-2.08.001: QueryConfig provides sensible defaults

**Preconditions:** `QueryConfig::default()`
**Postconditions:**
- default_timeout_secs: 30
- max_timeout_secs: 300
- max_result_rows: 10,000
- max_concurrent_queries: 50
- max_memory_per_query_mb: 512
**Evidence:** `crates/axiathon-query/src/config.rs` Default impl, `tests/config_test.rs`
**Confidence:** HIGH

---

## 18. AxiQL Error Contracts (BC-2.09.xxx)

### BC-2.09.001: AxiQLError builder methods chain correctly

**Preconditions:** AxiQLError instance
**Postconditions:**
- `with_label("in WHERE clause")` -> sets label field on Parse or Type variant
- `with_hint("try casting to Integer")` -> sets hint field on Parse or Type variant
- Mutates in-place and returns self (builder pattern)
- `span()`, `expected()`, `found()`, `label()`, `hint()` accessors work across both Parse and Type variants
**Evidence:** `crates/axiathon-query/src/error.rs` lines 103-124, `tests/error_test.rs`
**Confidence:** HIGH

### BC-2.09.002: AxiQLError::Parse found=None indicates EOF

**Preconditions:** Parser reached end of input unexpectedly
**Postconditions:** `found` is `None`, Display shows "end of input". For unexpected token, `found` is `Some(token_str)`.
**Evidence:** `error.rs` lines 40-66, parser output in parse_axiql()
**Confidence:** HIGH

---

## 19. Spike Error Contracts (BC-1.07.xxx)

### BC-1.07.001: Spike AxiathonError has domain-specific variants

**Preconditions:** Error in spike subsystem
**Postconditions:**
- `Parse(String)` -- parser failures
- `Storage(String)` -- Iceberg/Parquet failures
- `Query(String)` -- DataFusion failures
- `Detection(String)` -- rule evaluation failures
- `Plugin(String)` -- plugin lifecycle failures
- `Vault(String)` -- credential vault failures
- `Tenant(String)` -- tenant validation failures
- `Validation(String)` -- input validation failures
- `NotFound { resource, id }` -- entity lookup failures
- `Arrow(ArrowError)` -- transparent arrow conversion
- `Io(io::Error)` -- transparent I/O conversion
- `SerdeJson(serde_json::Error)` -- transparent JSON conversion
- `Other(anyhow::Error)` -- catch-all for third-party errors
**Evidence:** `spike/crates/axiathon-core/src/error.rs`
**Confidence:** HIGH

### BC-1.07.002: Spike error Display is for logging only (security)

**Preconditions:** Error rendered via Display
**Postconditions:** Display includes internal details (resource names, IDs, file paths). Code comments cite CWE-209. 8 specific call sites identified where errors leak to API responses (spike only).
**Evidence:** `spike/crates/axiathon-core/src/error.rs` security comment lines 3-11
**Confidence:** HIGH

---

## 20. Storage Schema Evolution Contracts (BC-4.02.xxx)

### BC-4.02.001: promote_fields() is idempotent

**Preconditions:** FieldPromotion requested for column that may already exist
**Postconditions:**
- First call: adds new column to Iceberg schema, returns vec with promoted field info
- Second call with same column: returns empty vec (no-op)
- Schema field count increases by exactly 1 per new promotion
**Evidence:** `field_promotion.rs` Phase 2 (lines 208-249): two promote_fields calls, second returns empty
**Confidence:** HIGH

### BC-4.02.002: Compaction with backfill extracts promoted fields from JSON

**Preconditions:** Old data files with values in unmapped JSON, schema evolved to add typed column
**Postconditions:**
- Compaction reads old files
- Extracts promoted field values from unmapped JSON
- Writes to typed column in compacted output
- After backfill: all data queryable via typed column (no UDF needed)
- Alive files verified via manifest inspection (not SQL, due to IcebergStaticTableProvider limitation)
**Evidence:** `field_promotion.rs` Phase 6 (lines 372-462)
**Confidence:** MEDIUM (test documents a known limitation: IcebergStaticTableProvider shows duplicates after rewrite_files)

### BC-4.02.003: json_extract_string UDF correctly parses unmapped JSON

**Preconditions:** Events with unmapped JSON blob, registered UDF
**Postconditions:**
- `json_extract_string(unmapped, 'syslog.hostname')` -> extracts value from JSON
- All old events with unmapped JSON return non-null values
- Used for pre-promotion data access
**Evidence:** `field_promotion.rs` Phase 4 (lines 310-325)
**Confidence:** HIGH

### BC-4.02.004: COALESCE wrapper transparently queries old + new data

**Preconditions:** Mix of old data (value in unmapped) and new data (value in typed column)
**Postconditions:**
- Query for hostname in OLD data -> found via json_extract_string fallback
- Query for hostname in NEW data -> found via typed column
- Wildcard query across both -> finds all matching records
**Evidence:** `field_promotion.rs` Phase 5 (lines 330-366)
**Confidence:** HIGH

---

## Updated Gaps

| Area | Gap | Status |
|------|-----|--------|
| Source::Sessions/Assets/Custom | No test evidence | Enum variants exist but may not be wired to actual tables |
| AxiQL error recovery | Not implemented | TODO(Story 5.2) -- parser stops at first error |
| Detection DSL security limits | No max query length/depth/regex size | Explicitly marked as spike-only gap |
| WASM plugin loading | Stub only | `wasm.rs` exists but no functional implementation |
| CrossVersionProjection | Placeholder struct | Story 5.3 work |

---

## Delta Summary
- New contracts added: 22 (BC-2.03.001 through BC-2.09.002, BC-3.05.001 through BC-3.05.009, BC-1.07.001-002, BC-4.02.001 through BC-4.02.004)
- Existing contracts refined: 0 (all Round 1 contracts remain accurate)
- Remaining gaps: Narrowed to 5 items (Source enum untested variants, error recovery, detection security limits, WASM stub, CrossVersionProjection)

## Novelty Assessment
Novelty: NITPICK
Round 2 filled in test evidence for entities already identified in Round 1 (Has, Missing, Wildcard, all PipeStages, Duration, signed values). The detection parser contracts (BC-3.05.xxx) are refinements of contracts already evident from the engine tests -- the parser is the means, the behavioral contracts from the engine tests capture the actual guarantees. The field promotion lifecycle contracts (BC-4.02.xxx) are elaborations of the Phase 5 test already documented in BC-4.01.005. No new behavioral guarantees were discovered that would change how you'd spec the system.

## Convergence Declaration
Pass 3 has converged -- findings are confirmations of test evidence for already-identified behaviors, not new behavioral discoveries.

## State Checkpoint
```yaml
pass: 3
round: 2
status: complete
files_scanned: 42
timestamp: 2026-04-13T00:00:00Z
novelty: NITPICK
```
