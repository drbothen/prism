# PrismQL (PQL) Reference

PrismQL (PQL) is the custom DSL used by Prism to query federated security sensor data.
Use `prism_describe` to discover tables and columns before writing queries.

## What is PrismQL

PrismQL (PQL) is a custom domain-specific language for querying Prism's ephemeral federated
security sensor data lake. Each query runs against one or more live sensor APIs and returns
normalized OCSF records. PQL is declarative: you specify what data you want, and Prism
handles routing, fan-out, and normalization.

Key properties:
- **Ephemeral**: no data is stored between queries; each query fetches from live sensors.
- **Federated**: a single query can fan out to multiple sensors (CrowdStrike, Claroty, Armis, etc.).
- **Schema-typed**: every column has a declared type (String, Integer, Float, Boolean, Datetime, Json).
- **Client-scoped**: queries run within a client_id scope (DI-008 isolation).

## Clause Grammar (BNF)

```
query      ::= select_clause from_clause [where_clause] [groupby_clause]
               [orderby_clause] [limit_clause]
             | pipeline

select_clause  ::= "SELECT" col_list
col_list       ::= "*" | col_expr ("," col_expr)*
col_expr       ::= col_name | aggregate_fn "(" col_name ")" ["AS" alias]

from_clause    ::= "FROM" table_ref
table_ref      ::= table_name

where_clause   ::= "WHERE" condition
condition      ::= predicate ("AND" | "OR" predicate)*
predicate      ::= col_name operator value
                 | col_name "IN" "(" value_list ")"
                 | col_name "IS" ["NOT"] "NULL"

groupby_clause ::= "GROUP BY" col_name ("," col_name)*
orderby_clause ::= "ORDER BY" col_name ["ASC" | "DESC"]
limit_clause   ::= "LIMIT" integer

pipeline       ::= query "|" query_stage ("|" query_stage)*
query_stage    ::= where_clause | orderby_clause | limit_clause | select_clause

operator       ::= "=" | "!=" | "<" | "<=" | ">" | ">=" | "LIKE" | "NOT LIKE"
aggregate_fn   ::= "COUNT" | "SUM" | "AVG" | "MIN" | "MAX"
value          ::= string_literal | integer | float | boolean | datetime_expr
```

Examples:
```
SELECT * FROM <sensor_table> LIMIT 25
SELECT * FROM <sensor_table> WHERE severity = 'HIGH' LIMIT 25
SELECT col1, COUNT(*) FROM <sensor_table> GROUP BY col1 LIMIT 25
SELECT * FROM <sensor_table> | WHERE status = 'open' | LIMIT 10
```

## Operators and Types

| Column Type | Supported Operators | Notes |
|-------------|---------------------|-------|
| String | =, !=, LIKE, NOT LIKE, IN, IS NULL, IS NOT NULL | LIKE uses % wildcard |
| Integer | =, !=, <, <=, >, >=, IN, IS NULL, IS NOT NULL | Numeric comparison |
| Float | =, !=, <, <=, >, >=, IS NULL, IS NOT NULL | Numeric comparison |
| Boolean | =, !=, IS NULL, IS NOT NULL | Values: true, false |
| Datetime | =, !=, <, <=, >, >=, IS NULL, IS NOT NULL | Use NOW() and INTERVAL |
| Json | IS NULL, IS NOT NULL | Structural equality only |

Aggregate functions (GROUP BY required):
- COUNT(*), COUNT(col) — row count
- SUM(col) — sum of Integer/Float column
- AVG(col) — average of Integer/Float column
- MIN(col), MAX(col) — min/max of Integer/Float/Datetime column

## Datetime Arithmetic

PQL supports relative datetime expressions using `NOW()` and `INTERVAL`:

```
NOW()                     -- current UTC timestamp
NOW() - INTERVAL '24h'    -- 24 hours ago
NOW() - INTERVAL '7d'     -- 7 days ago
NOW() - INTERVAL '30m'    -- 30 minutes ago
NOW() - INTERVAL '1w'     -- 1 week ago
```

Interval units: `s` (seconds), `m` (minutes), `h` (hours), `d` (days), `w` (weeks).

OCSF timestamp fields are typically named `time`, `created_time`, `modified_time`,
`start_time`, or `end_time` — use `prism_describe` to confirm column names for each table.

Example:
```
SELECT * FROM <sensor_table> WHERE time > NOW() - INTERVAL '24h' LIMIT 25
```

## Error Code Quick-Reference

| Error Code | Trigger | Recovery |
|------------|---------|----------|
| E-QUERY-001 | Unknown table name in FROM clause | Use `prism_describe` to list valid table names for the client; check spelling |
| E-QUERY-002 | Unknown column name in SELECT or WHERE | Use `prism_describe` to list valid columns; check `available_columns` in error |
| E-QUERY-003 | Invalid operator for column type (e.g., LIKE on Integer) | Check `valid_operators_for_type` in error; use a compatible operator |
| E-QUERY-037 | Query syntax error — unexpected token | Read `near_text` and `how_to_fix` in error; consult Clause Grammar section above |
| E-QUERY-038 | Normalized PQL validation failure after parse | Read `did_you_mean` and `how_to_fix`; verify column types match operator expectations |

All error responses include: `near_text`, `available_columns`, `did_you_mean`,
`valid_operators_for_type`, `how_to_fix`. Use these fields to self-correct.

## Query Examples

All examples use `<sensor_table>` placeholder — replace with real table names from `prism_describe`.

**Retrieve recent records:**
```
SELECT * FROM <sensor_table> LIMIT 25
```

**Filter by severity:**
```
SELECT * FROM <sensor_table> WHERE severity = 'HIGH' LIMIT 25
```

**Filter by time range (last 24 hours):**
```
SELECT * FROM <sensor_table> WHERE time > NOW() - INTERVAL '24h' LIMIT 50
```

**Count records grouped by a field:**
```
SELECT status, COUNT(*) FROM <sensor_table> GROUP BY status LIMIT 10
```

**Multi-stage pipeline:**
```
SELECT * FROM <sensor_table> | WHERE severity = 'CRITICAL' | ORDER BY time DESC | LIMIT 10
```

**Null check:**
```
SELECT * FROM <sensor_table> WHERE resolved IS NULL LIMIT 25
```

## Self-Correction Workflow

When a query fails with an E-QUERY error:

1. **Read the error fields:**
   - `near_text`: the token or expression where parsing failed
   - `available_columns`: valid column names for the table you queried
   - `did_you_mean`: suggested correction for misspelled column or operator
   - `valid_operators_for_type`: operators valid for the column type you used
   - `how_to_fix`: step-by-step remedy for the specific error

2. **Consult this reference:** find the error code in the quick-reference table above.

3. **Rewrite and retry:** apply the correction. Retry up to 3 times.

4. **If still failing after 3 retries:** call `prism_describe` again to re-read the current
   schema (hot-reload may have changed table/column names), then retry once more.
