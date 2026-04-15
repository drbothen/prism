---
document_type: grammar-reference
version: "1.0"
status: draft
timestamp: 2026-04-13
source: axiathon crate axiathon-query (production workspace)
parser: Chumsky 0.10 (PEG-style parser combinators)
---

# AxiQL Grammar Reference

This document defines the complete AxiQL grammar as extracted from the axiathon production parser (`crates/axiathon-query/src/parser.rs`, `ast.rs`, `axiathon-core/src/query_types.rs`). It serves as the normative reference for Prism's query language implementation.

---

## 1. Notation

This grammar uses EBNF notation with the following conventions:

| Symbol | Meaning |
|--------|---------|
| `=` | Definition |
| `;` | End of rule |
| `\|` | Alternation |
| `[ ... ]` | Optional (zero or one) |
| `{ ... }` | Repetition (zero or more) |
| `( ... )` | Grouping |
| `"..."` | Terminal string (case-insensitive for keywords) |
| `'...'` | Terminal character |
| `/* ... */` | Grammar comment |

All keywords are **case-insensitive**. The parser uses `text::ident()` with case-insensitive comparison (not `text::keyword()`), per Chumsky community convention (zesterer/chumsky#699).

---

## 2. Top-Level Grammar

```ebnf
axiql_query = [ whitespace ] , ( sql_statement | pipe_or_filter ) , [ whitespace ] , EOF ;

pipe_or_filter = filter_expr , { "|" , pipe_stage } ;
/* If pipe stages are present -> Pipe mode; otherwise -> Filter mode */

sql_statement = "SELECT" , select_list , "FROM" , source
              , [ "WHERE" , filter_expr ]
              , [ "GROUP" , "BY" , field_list ]
              , [ "ORDER" , "BY" , order_by_list ]
              , [ "LIMIT" , positive_integer ] ;
```

### 2.1 Mode Detection

The parser attempts SQL first (leading `SELECT` keyword), then falls back to pipe-or-filter. A query with a filter expression followed by one or more `|` pipe stages is Pipe mode; without pipes it is Filter mode. There is no explicit mode marker -- the grammar is unambiguous.

```
axiql_parser = choice( sql_statement , pipe_or_filter ) ;
```

---

## 3. Token Definitions

### 3.1 Identifiers

```ebnf
identifier = ( letter | "_" ) , { letter | digit | "_" } ;
/* First character must be alphabetic or underscore. Digits not allowed at start.
   This prevents ambiguity with integer literals and aligns with OCSF naming,
   SQL standard, and all SIEM query languages (Splunk, Elastic, KQL). */
```

### 3.2 Field References

```ebnf
field_ref = vendor_extension | dotted_path ;

dotted_path = segment , { "." , segment } ;
segment = identifier , [ "[" , bracket_content , "]" ] ;
bracket_content = { any_char_except_close_bracket } ;

vendor_extension = "UNMAPPED" , "[" , "'" , { any_char_except_single_quote } , "'" , "]" ;
/* Case-insensitive UNMAPPED keyword. Preserves dots inside brackets.
   Example: unmapped['claroty.alert_type'] */
```

**Examples:**
- `severity` -- single segment
- `src_endpoint.ip` -- dotted path
- `answers[0].value` -- array indexing
- `unmapped['vendor.field_name']` -- vendor extension bracket notation

### 3.3 Literals

```ebnf
value = string_literal | boolean_literal | duration_literal
      | float_literal | integer_literal ;
/* Parse order matters: duration (most specific), float, integer (least specific) */

string_literal = '"' , { string_char } , '"' ;
string_char = escape_sequence | any_char_except_quote_or_backslash ;
escape_sequence = known_escape | unknown_escape ;
known_escape = "\" , ( '"' | '\' | 'n' | 'r' | 't' ) ;
/* Known escapes map to control characters: \" -> ", \\ -> \, \n -> newline, etc. */
unknown_escape = "\" , any_char_except_quote ;
/* Unknown escapes pass through literally: \. -> \. (preserves regex patterns) */

boolean_literal = "TRUE" | "FALSE" ;
/* Case-insensitive */

integer_literal = [ "-" ] , digit , { digit } ;
/* Signed via optional minus prefix. No binary minus operator in AxiQL,
   so -100 is unambiguously a negative number.
   Uses i128 intermediate to correctly handle i64::MIN.
   Range: -9223372036854775808 to 9223372036854775807 (i64) */

float_literal = [ "-" ] , digit , { digit } , "." , digit , { digit } ;
/* Must have digits on both sides of the decimal point.
   Range: f64. */

duration_literal = digit , { digit } , duration_unit ;
duration_unit = "s" | "m" | "h" | "d" ;
/* Always positive (no negative durations).
   s = seconds, m = minutes (x60), h = hours (x3600), d = days (x86400).
   Overflow detected via checked_mul. */

positive_integer = digit , { digit } ;
/* Used for LIMIT, head, tail. Range: u64. */
```

### 3.4 Comments

```ebnf
line_comment = ( "//" | "#" ) , { any_char_except_newline } , ( newline | EOF ) ;
/* Stripped in a preprocessing pass before parsing.
   Comment content replaced with spaces to preserve byte offsets for error spans.
   Comment markers inside quoted strings are NOT comments. */
```

---

## 4. Filter Expressions

Filter expressions form a boolean tree of field predicates with standard precedence: NOT binds tightest, then AND, then OR.

```ebnf
filter_expr = or_expr ;

or_expr = and_expr , { ( "OR" | "||" ) , and_expr } ;

and_expr = not_expr , { ( "AND" | "&&" ) , not_expr } ;

not_expr = ( "NOT" | "!" ) , atom
         | atom ;

atom = comparison | parenthesized ;

parenthesized = "(" , filter_expr , ")" ;
/* Nesting depth tracked and limited to MAX_NESTING_DEPTH (128). */

comparison = has_check
           | missing_check
           | regex_match
           | cidr_match
           | not_in_list
           | in_list
           | string_op_match
           | field_comparison ;
/* Parse order matters for disambiguation. */

has_check = "HAS" , field_ref ;
missing_check = "MISSING" , field_ref ;

regex_match = field_ref , ( "=~" | "MATCHES" ) , string_literal ;
/* Regex pattern validated at parse time via regex::Regex::new().
   Uses finite automaton engine (immune to catastrophic backtracking / ReDoS).
   Max pattern length: 1024 bytes (CWE-1333). */

cidr_match = field_ref , "IN" , "CIDR" , string_literal ;
/* CIDR string validated at parse time (CWE-20).
   Supports IPv4 (/0-/32) and IPv6 (/0-/128). */

not_in_list = field_ref , "NOT" , "IN" , "(" , value_list , ")" ;
in_list = field_ref , "IN" , "(" , value_list , ")" ;
value_list = value , { "," , value } ;

string_op_match = field_ref , string_op , string_literal ;
string_op = "CONTAINS" | "STARTSWITH" | "ENDSWITH"
          | "ICONTAINS" | "ISTARTSWITH" | "IENDSWITH" ;
/* Case-insensitive variants prefixed with I. */

field_comparison = field_ref , compare_op , value ;
/* If value is a string containing * or ?, auto-promotes to wildcard match:
   - With = -> Wildcard (negated: false)
   - With != -> Wildcard (negated: true)
   - With >, <, >=, <= -> parse error (ordering ops meaningless on wildcards) */
```

### 4.1 Comparison Operators

```ebnf
compare_op = "!=" | ">=" | "<=" | "==" | "=" | ">" | "<" ;
/* Parse order: multi-char operators first to prevent partial match.
   == and = are both accepted as equality. */
```

| Operator | AST Variant | Notes |
|----------|-------------|-------|
| `=` | `CompareOp::Eq` | Single equals |
| `==` | `CompareOp::Eq` | Double equals (also equality) |
| `!=` | `CompareOp::Ne` | Not equal |
| `>` | `CompareOp::Gt` | Greater than |
| `<` | `CompareOp::Lt` | Less than |
| `>=` | `CompareOp::GtEq` | Greater than or equal |
| `<=` | `CompareOp::LtEq` | Less than or equal |

### 4.2 Boolean Connectives

| Syntax | AST Variant | Precedence |
|--------|-------------|------------|
| `NOT` / `!` | `FilterExpr::Not` | Highest (unary) |
| `AND` / `&&` | `FilterExpr::And` | Middle (left-associative) |
| `OR` / `\|\|` | `FilterExpr::Or` | Lowest (left-associative) |

Note: `||` is parsed as OR, not as pipe-then-pipe. The parser resolves this by consuming `||` as the OR operator within filter expression context.

---

## 5. SQL Mode

```ebnf
sql_statement = "SELECT" , select_list , "FROM" , source
              , [ "WHERE" , filter_expr ]
              , [ "GROUP" , "BY" , field_list ]
              , [ "ORDER" , "BY" , order_by_list ]
              , [ "LIMIT" , positive_integer ] ;

select_list = select_item , { "," , select_item } ;
select_item = ( aggregation_expr | "*" | field_ref ) , [ "AS" , identifier ] ;

aggregation_expr = percentile_agg | standard_agg ;

standard_agg = agg_function , "(" , ( "*" | field_ref ) , ")" ;
agg_function = "COUNT" | "SUM" | "AVG" | "MIN" | "MAX" | "DISTINCT_COUNT" ;

percentile_agg = "PERCENTILE" , "(" , field_ref , "," , percentile_value , ")" ;
percentile_value = float_literal | integer_literal ;
/* Must be between 0 and 100 inclusive. PERCENTILE(*, 95) is invalid. */

source = "EVENTS" | "ALERTS" | "SESSIONS" | "ASSETS" | identifier ;
/* Built-in sources matched case-insensitively. Any other identifier
   falls through to Source::Custom (custom views, saved queries). */

field_list = field_ref , { "," , field_ref } ;

order_by_list = order_by_item , { "," , order_by_item } ;
order_by_item = field_ref , [ sort_direction ] ;
sort_direction = "ASC" | "DESC" ;
/* Default is ASC when omitted. */
```

### 5.1 Aggregation Functions

| Function | Syntax | Notes |
|----------|--------|-------|
| `COUNT` | `COUNT(*)` or `COUNT(field)` | Count all or count non-null |
| `SUM` | `SUM(field)` | Numeric sum |
| `AVG` | `AVG(field)` | Numeric average |
| `MIN` | `MIN(field)` | Minimum value |
| `MAX` | `MAX(field)` | Maximum value |
| `DISTINCT_COUNT` | `DISTINCT_COUNT(field)` | Count of unique values |
| `PERCENTILE` | `PERCENTILE(field, N)` | N-th percentile (0-100, int or float) |

---

## 6. Pipe Mode

```ebnf
pipe_query = filter_expr , { "|" , pipe_stage } ;
/* At least one pipe stage required to be Pipe mode (vs. Filter mode). */

pipe_stage = head_stage | tail_stage | stats_stage
           | sort_stage | dedup_stage | fields_stage ;

head_stage = "HEAD" , positive_integer ;
tail_stage = "TAIL" , positive_integer ;

sort_stage = "SORT" , sort_item , { "," , sort_item } ;
sort_item = field_ref , [ sort_direction ] ;

dedup_stage = "DEDUP" , field_ref , { "," , field_ref } ;

fields_stage = "FIELDS" , [ fields_mode ] , field_ref , { "," , field_ref } ;
fields_mode = "+" | "-" ;
/* Default is Include (+) when omitted. At least one field required. */

stats_stage = "STATS" , stat_function_list , [ "BY" , field_list ] ;
stat_function_list = stat_function , { "," , stat_function } ;
stat_function = ( percentile_agg | parenthesized_agg | bare_count ) , [ "AS" , identifier ] ;

parenthesized_agg = agg_function , "(" , [ "*" | field_ref ] , ")" ;
/* Empty parens count() = count all (per KQL/ES|QL/LogScale consensus). */

bare_count = "COUNT" ;
/* Bare count without parens = count all. Only COUNT is allowed bare;
   bare SUM, AVG, MIN, MAX are rejected ("sum of what?"). */
```

### 6.1 Pipe Stage Catalog

| Stage | Syntax | Description |
|-------|--------|-------------|
| `head` | `head N` | Return first N rows |
| `tail` | `tail N` | Return last N rows |
| `sort` | `sort field [ASC\|DESC], ...` | Sort by fields; default ASC |
| `dedup` | `dedup field, ...` | Deduplicate by field combination |
| `fields` | `fields [+\|-] field, ...` | Include (+) or exclude (-) fields; default include |
| `stats` | `stats agg [AS alias], ... [BY field, ...]` | Aggregation with optional grouping |

---

## 7. AST Type Summary

The parser produces `AxiQLStatement`, which is one of three variants:

```
AxiQLStatement
  |-- Filter(FilterExpr)
  |-- Select { projection, from, filter, group_by, order_by, limit }
  |-- Pipe { filter: FilterExpr, stages: Vec<PipeStage> }

FilterExpr
  |-- Comparison { field, op, value }
  |-- And(left, right)
  |-- Or(left, right)
  |-- Not(inner)
  |-- Has(field)
  |-- Missing(field)
  |-- Regex { field, pattern }
  |-- StringMatch { field, op, value }
  |-- InList { field, values, negated }
  |-- CidrMatch { field, cidr }
  |-- Wildcard { field, pattern, negated }

SelectItem
  |-- Unaliased(SelectExpr)
  |-- Aliased { expr, alias }

SelectExpr
  |-- Star
  |-- Field(FieldRef)
  |-- Aggregation(AggregationExpr)

AggregationExpr { function: AggFunction, field: Option<FieldRef> }

AggFunction = Count | Sum | Avg | Min | Max | DistinctCount | Percentile(f64)

Source = Events | Alerts | Sessions | Assets | Custom(String)

PipeStage
  |-- Stats { functions: Vec<StatFunction>, group_by }
  |-- Sort(Vec<OrderByExpr>)
  |-- Head(u64)
  |-- Tail(u64)
  |-- Dedup(Vec<FieldRef>)
  |-- Fields { mode: Include|Exclude, fields }

StatFunction { agg: AggregationExpr, alias: Option<String> }

Value = String | Integer(i64) | Float(f64) | Boolean | Regex { pattern, flags } | Duration

FieldRef { segments: Vec<FieldSegment>, has_array: bool }
FieldSegment = Named(String) | Index(name, index)

CompareOp = Eq | Ne | Gt | Lt | GtEq | LtEq
StringOp = Contains | StartsWith | EndsWith | IContains | IStartsWith | IEndsWith
```

### 7.1 Type System

The parser output feeds into a post-parse type checker (partially implemented as of axiathon Story 5.1). The type system defines:

| AxiQL Type | Valid for Equality | Valid for Ordering |
|------------|-------------------|--------------------|
| String | Yes | Yes |
| Integer | Yes | Yes |
| Float | Yes | Yes |
| Boolean | Yes | No |
| IpAddress | Yes | No |
| Timestamp | Yes | Yes |

---

## 8. Security Limits

All limits are compile-time constants in the parser, cited against specific CWEs.

| Limit | Value | CWE | Enforcement Point |
|-------|-------|-----|-------------------|
| Maximum query length | 65,536 bytes (64 KB) | CWE-400 | Pre-parse check |
| Maximum nesting depth | 128 levels | CWE-674 | `Rc<Cell<usize>>` counter in paren parser |
| Maximum pipe stages | 64 | CWE-400 | Post-parse validation |
| Maximum regex pattern length | 1,024 bytes | CWE-1333 | In regex_match combinator |
| Integer overflow | i64 range | CWE-190 | `try_map` with structured error |
| Invalid CIDR | IP + prefix validated | CWE-20 | `validate_cidr()` at parse time |
| Regex engine | Finite automaton (no backtracking) | CWE-1333 | `regex::Regex::new()` validation |

### 8.1 Query Execution Limits (QueryConfig)

These are separate from parser limits and are hot-reloadable configuration:

| Limit | Default Value |
|-------|---------------|
| Default query timeout | 30 seconds |
| Maximum query timeout | 300 seconds |
| Maximum result rows | 10,000 |
| Maximum concurrent queries | 50 |
| Maximum memory per query | 512 MB |

---

## 9. Field Alias Resolution

The parser operates on raw field references. Alias resolution is a separate layer (pre-parse or post-parse depending on alias type).

### 9.1 Three-Tier Resolution

```
Analyst Shortcut -> AxiQL Canonical -> OCSF Canonical
     src_ip     ->     src.ip       -> src_endpoint.ip
```

Resolution is a single HashMap lookup with provenance tracking (`OcsfDirect`, `AliasResolved`, `Unknown`).

### 9.2 Default Alias Table

| Shortcut | AxiQL Canonical | OCSF Canonical |
|----------|----------------|----------------|
| `src_ip` | `src.ip` | `src_endpoint.ip` |
| `dst_ip` | `dst.ip` | `dst_endpoint.ip` |
| `src_port` | `src.port` | `src_endpoint.port` |
| `dst_port` | `dst.port` | `dst_endpoint.port` |
| `user` | `user.name` | `actor.user.name` |
| `hostname` | `device.hostname` | `device.hostname` |
| `action` | `activity_name` | `activity_name` |

### 9.3 Version-Conditional Aliases

`OcsfVersionAliasMap` provides version-specific resolution for fields that moved between OCSF versions. Maps `(alias, ocsf_version)` to the resolved `FieldRef` for that version.

---

## 10. Examples

### 10.1 Filter Mode

```axiql
// Simple comparison
severity = "high"

// Boolean logic with precedence
(severity = "high" OR severity = "critical") AND src_endpoint.ip = "10.0.0.1"

// Field existence checks
HAS src_endpoint.ip
MISSING user.name

// Regex match (validated at parse time)
src_endpoint.ip =~ "10\.0\..*"

// CIDR match
src_endpoint.ip IN CIDR "10.0.0.0/8"

// IN list and NOT IN
severity IN ("high", "critical")
severity NOT IN ("low", "info")

// String operations (case-sensitive and case-insensitive)
user.name CONTAINS "admin"
file.path IENDSWITH ".exe"

// Wildcard patterns
src_endpoint.ip = "10.0.*"
src_endpoint.ip != "192.168.*"

// Duration and numeric comparisons
response.time > 30s
uptime > 24h
cert.expiry < 7d
dst_endpoint.port > 1024
confidence >= 0.95

// Negative numbers
traffic.bytes_in > -100
temperature < -2.75

// Boolean values
is_remote = TRUE

// Vendor extension fields
unmapped['claroty.alert_type'] = "critical"

// Alternative operators
severity == "high"       // double equals
!severity = "low"        // bang prefix for NOT
a = "x" && b = "y"      // double ampersand for AND
a = "x" || b = "y"      // double pipe for OR

// Comments
src_endpoint.ip = "10.0.0.1" // find this IP
src_endpoint.ip = "10.0.0.1" # also a comment
```

### 10.2 SQL Mode

```axiql
// Select all
SELECT * FROM EVENTS

// Projection with alias
SELECT src_endpoint.ip AS source_ip, dst_endpoint.ip FROM EVENTS

// Full query with all clauses
SELECT src_endpoint.ip, COUNT(*) AS total
FROM EVENTS
WHERE severity = "high"
GROUP BY src_endpoint.ip
ORDER BY total DESC
LIMIT 20

// Aggregation functions
SELECT COUNT(*) FROM EVENTS
SELECT SUM(traffic.bytes_in) FROM EVENTS
SELECT AVG(confidence_score) FROM EVENTS
SELECT MIN(severity_id), MAX(severity_id) FROM EVENTS
SELECT DISTINCT_COUNT(src_endpoint.ip) FROM EVENTS
SELECT PERCENTILE(response.latency, 95) AS p95 FROM EVENTS
SELECT PERCENTILE(response.latency, 99.9) FROM EVENTS

// Custom source (saved query / view)
SELECT * FROM my_saved_query

// Duration in WHERE
SELECT * FROM EVENTS WHERE response_time > 30s

// Case-insensitive keywords
select * from events where severity = "high"
Select * From Events
```

### 10.3 Pipe Mode

```axiql
// Basic pipe
severity = "high" | head 10

// Stats with grouping
severity = "high" | stats count by src_endpoint.ip

// Multiple aggregations with aliases
severity = "high" | stats count(*) AS total, sum(traffic.bytes_in) AS total_bytes BY src_endpoint.ip

// Chained stages
severity = "high" | stats count by src_endpoint.ip | sort count desc | head 10

// All pipe stages
severity = "high" | tail 5
severity = "high" | dedup src_endpoint.ip
severity = "high" | fields + src_endpoint.ip, dst_endpoint.ip
severity = "high" | fields - raw_data, metadata
severity = "high" | fields src_endpoint.ip, user.name   // bare = include

// Stats variations
severity = "high" | stats count                         // bare count (no parens)
severity = "high" | stats count()                       // empty parens = count all
severity = "high" | stats count(*) by src_endpoint.ip   // explicit star
severity = "high" | stats count(*) AS total by src_endpoint.ip

// Percentile in pipe
severity = "high" | stats percentile(response.latency, 95) by src_endpoint.ip
severity = "high" | stats percentile(response.latency, 99) AS p99 by src_endpoint.ip
```

---

## 11. Prism Adaptations

The following sections document where Prism's grammar differs from axiathon's or extends it. These are changes Prism needs, not things present in axiathon.

### 11.1 Adopted As-Is from Axiathon

| Feature | Status |
|---------|--------|
| All three modes (filter, SQL, pipe) | Adopted |
| All filter expression types (comparison, boolean, regex, CIDR, IN, string ops, wildcard, HAS/MISSING) | Adopted |
| All pipe stages (stats, sort, head, tail, dedup, fields) | Adopted |
| All aggregation functions (COUNT, SUM, AVG, MIN, MAX, DISTINCT_COUNT, PERCENTILE) | Adopted |
| SQL SELECT with WHERE, GROUP BY, ORDER BY, LIMIT | Adopted |
| Value types (string, integer, float, boolean, duration) | Adopted |
| Field reference syntax (dotted paths, array indexing, vendor extension brackets) | Adopted |
| Case-insensitive keywords | Adopted |
| Comment syntax (// and #) | Adopted |
| All security limits (64KB, 128 depth, 64 stages, 1024-byte regex) | Adopted |
| Three-tier alias resolution architecture | Adopted |
| Escape sequence handling in strings | Adopted |

### 11.2 Modified for Prism

| Feature | Axiathon | Prism Adaptation |
|---------|----------|------------------|
| **Sensor/client virtual fields** | Not present | Prism adds virtual field `_sensor` and `_client` that are transparently injected by the query engine based on tool parameters. These are not parseable as user-specified fields -- they are implicit context. |
| **Alias parameter substitution** | Not implemented (design only) | Prism implements parameterized aliases with function-call syntax: `alias_name(param="value")`. Parameter values must parse as a single AxiQL literal token (see BC-2.11.009). |
| **Alias scope resolution** | Single global registry | Prism adds per-client alias scope: per-client aliases override global aliases of the same name when querying a specific client. |
| **Alias composition** | Not implemented | Prism allows aliases to reference other aliases (up to depth 3, cycles detected at config load). |
| **Source variants** | Events, Alerts, Sessions, Assets, Custom | Prism may add Findings, Incidents as built-in sources depending on domain model finalization. |
| **OCSF version filter syntax** | Placeholder (`OcsfVersionFilter`) | Prism will define explicit syntax for version-scoped queries if cross-version querying is implemented. |
| **Error recovery** | Stops at first error (TODO Story 5.2) | Prism should implement Chumsky error recovery from the start for multi-error reporting. |

### 11.3 Prism-Specific Token: Alias Parameter Values

Per BC-2.11.009, alias parameter values are validated as AxiQL literal tokens:

```ebnf
alias_param_value = string_literal | integer_literal | float_literal
                  | boolean_literal | duration_literal | identifier ;
/* Values that parse as expressions, operators, or compound constructs
   are rejected (E-ALIAS-004). This prevents query injection through
   parameterized alias values. */
```

Valid examples: `"critical"`, `4`, `24h`, `src_endpoint.ip`, `true`
Invalid examples: `"x" OR "y"`, `field = value`, `1 + 2`

---

## 12. Formal Grammar Summary (Condensed EBNF)

```ebnf
(* AxiQL Complete Grammar *)

query           = ws , ( sql | pipe_or_filter ) , ws , EOF ;
pipe_or_filter  = filter , { "|" , stage } ;
sql             = "SELECT" , select_list , "FROM" , source
                  , [ "WHERE" , filter ]
                  , [ "GROUP" , "BY" , fields ]
                  , [ "ORDER" , "BY" , sorts ]
                  , [ "LIMIT" , pos_int ] ;

(* Filter expressions -- precedence: NOT > AND > OR *)
filter          = or ;
or              = and , { ( "OR" | "||" ) , and } ;
and             = not , { ( "AND" | "&&" ) , not } ;
not             = ( "NOT" | "!" ) , atom | atom ;
atom            = comparison | "(" , filter , ")" ;

comparison      = "HAS" , field
                | "MISSING" , field
                | field , ( "=~" | "MATCHES" ) , string
                | field , "IN" , "CIDR" , string
                | field , "NOT" , "IN" , "(" , vals , ")"
                | field , "IN" , "(" , vals , ")"
                | field , str_op , string
                | field , cmp_op , value ;

cmp_op          = "!=" | ">=" | "<=" | "==" | "=" | ">" | "<" ;
str_op          = "CONTAINS" | "STARTSWITH" | "ENDSWITH"
                | "ICONTAINS" | "ISTARTSWITH" | "IENDSWITH" ;

(* Values *)
value           = string | bool | duration | float | integer ;
string          = '"' , { esc | char } , '"' ;
esc             = '\' , ( '"' | '\' | 'n' | 'r' | 't' | any ) ;
bool            = "TRUE" | "FALSE" ;
integer         = [ "-" ] , digits ;
float           = [ "-" ] , digits , "." , digits ;
duration        = digits , ( "s" | "m" | "h" | "d" ) ;
pos_int         = digits ;
digits          = digit , { digit } ;

(* Fields *)
field           = "UNMAPPED" , "['" , dotted , "']"
                | ident , { "." , ident } ;
ident           = ( letter | "_" ) , { letter | digit | "_" } ;

(* SQL SELECT *)
select_list     = sel_item , { "," , sel_item } ;
sel_item        = ( agg | "*" | field ) , [ "AS" , ident ] ;
agg             = "PERCENTILE" , "(" , field , "," , num , ")"
                | func , "(" , ( "*" | field ) , ")" ;
func            = "COUNT" | "SUM" | "AVG" | "MIN" | "MAX" | "DISTINCT_COUNT" ;
source          = "EVENTS" | "ALERTS" | "SESSIONS" | "ASSETS" | ident ;
fields          = field , { "," , field } ;
sorts           = sort_item , { "," , sort_item } ;
sort_item       = field , [ "ASC" | "DESC" ] ;

(* Pipe stages *)
stage           = "HEAD" , pos_int
                | "TAIL" , pos_int
                | "SORT" , sort_item , { "," , sort_item }
                | "DEDUP" , field , { "," , field }
                | "FIELDS" , [ "+" | "-" ] , field , { "," , field }
                | "STATS" , stat_fn , { "," , stat_fn } , [ "BY" , fields ] ;
stat_fn         = ( "PERCENTILE" , "(" , field , "," , num , ")"
                  | func , "(" , [ "*" | field ] , ")"
                  | "COUNT" ) , [ "AS" , ident ] ;
vals            = value , { "," , value } ;
num             = float | integer ;
ws              = { whitespace_char | line_comment } ;
line_comment    = ( "//" | "#" ) , { not_newline } , newline ;
```
