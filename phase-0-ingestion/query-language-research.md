# Query Language & Alias System Design Research

**Project:** Prism (Rust MCP Server for Security Sensor Integration)
**Research Date:** 2026-04-13
**Research Type:** General (Architecture & Design)
**Status:** Complete
**Inputs:** axiathon-pass-8-deep-synthesis.md, product-brief.md, recovered-architecture.md, ai-consumable-design-research.md

---

## 1. AxiQL Syntax Adaptation for Prism

### 1.1 Axiathon's Three Query Modes

Axiathon's production AxiQL parser (Chumsky 0.10, 1799 LOC, 315 tests) implements three modes:

1. **Filter mode** -- Splunk-style boolean: `severity >= "high" AND src_ip = "10.0.0.1"`
2. **SQL SELECT mode** -- Projection, aggregation, GROUP BY, ORDER BY, LIMIT: `SELECT src_ip, count(*) FROM alerts WHERE severity >= "high" GROUP BY src_ip ORDER BY count DESC LIMIT 10`
3. **Pipe mode** -- KQL-style chaining: `alerts | where severity >= "high" | stats count by src_ip | sort count desc | head 10`

The parser auto-detects the mode based on the first token: `SELECT` triggers SQL mode, a bare expression triggers filter mode, a source name followed by `|` triggers pipe mode.

### 1.2 Recommendation: Adopt All Three Modes, With Modifications

**Filter mode -- adopt as-is.** This is the natural mode for AI agents constructing simple queries. An LLM generating `severity >= "high" AND device.ip = "10.0.0.1"` is straightforward and low-risk for syntax errors. Filter mode should be the default for MCP tool `query` parameters.

**SQL mode -- adopt with modifications.** SQL is the lingua franca that LLMs understand best from training data. However, Prism's architecture diverges from axiathon's in a critical way: Prism is a real-time query server against live sensor APIs, not a SIEM querying stored Parquet/Iceberg tables. This changes what SQL operations are feasible:

| SQL Feature | Axiathon (SIEM) | Prism (Live API) | Recommendation |
|-------------|-----------------|-------------------|----------------|
| SELECT projection | Over stored data | Over API responses | Adopt -- applied post-fetch |
| WHERE filtering | Predicate pushdown to Parquet | Translated to sensor API filters where possible, post-filter remainder | Adopt -- dual execution |
| GROUP BY / aggregation | DataFusion aggregation | In-memory aggregation on fetched results | Adopt -- useful for summarization |
| ORDER BY | DataFusion sorting | In-memory sort on fetched results | Adopt -- useful for prioritization |
| LIMIT | DataFusion limit | Applied post-fetch (or translated to API page_size) | Adopt |
| JOIN | DataFusion cross-table join | See section 1.3 below | Modified -- see below |

**Pipe mode -- adopt with caution.** Pipe mode is powerful for analysts who want to chain operations, but AI agents generally construct queries as parameters to tools, not as multi-stage pipelines. Pipe mode is more valuable for a human TUI than for AI tool calls. Recommendation: implement pipe mode but deprioritize it behind filter and SQL modes. The AI will primarily use filter mode for simple queries and SQL mode for complex ones.

### 1.3 Cross-Sensor JOINs: Implicit Unified Table vs Explicit JOINs

**Strong recommendation: implicit OCSF-unified table, NOT explicit JOINs.**

Reasoning:

1. **Prism's architecture is stateless real-time.** JOINs across live sensor APIs would require: (a) fetching full datasets from both sensors, (b) materializing them in memory, (c) performing the join. This is expensive, latency-sensitive, and fails when one sensor API is slow or down. An explicit `FROM crowdstrike.alerts cs JOIN claroty.events cl ON cs.device.ip = cl.device.ip` implies a relational engine that Prism is not.

2. **OCSF normalization already solves correlation.** The entire point of OCSF normalization is that `device.ip` means the same thing across CrowdStrike and Claroty. A query like `WHERE device.ip = "10.0.0.1"` against the unified OCSF view returns results from ALL sensors that reported events for that IP. This is more intuitive for the AI agent than writing JOINs.

3. **Cross-sensor correlation is a semantic operation, not a syntactic one.** The AI agent asks "show me all security events for device 10.0.0.1 across all sensors." This maps to a single query over the unified OCSF table, not a multi-sensor JOIN. The Prism server handles the fan-out to multiple sensor APIs internally.

4. **AI agents are bad at JOINs.** LLMs frequently generate incorrect JOIN conditions, especially with aliased table names. A single-table query with WHERE filters is dramatically more reliable for AI-generated queries.

**Design: The unified OCSF query model.**

```
-- The AI sees one logical table of OCSF events, optionally scoped by sensor
FROM events WHERE device.ip = "10.0.0.1" AND severity >= "high"

-- If the AI wants sensor-specific data, use a sensor filter
FROM events WHERE sensor = "crowdstrike" AND device.ip = "10.0.0.1"

-- Cross-sensor correlation is just a query without sensor filter
FROM events WHERE device.ip = "10.0.0.1"
-- Returns CrowdStrike alerts + Claroty events + Armis alerts for that IP
```

Under the hood, Prism translates this to parallel API calls to each sensor, normalizes results to OCSF, and merges them into a single result set.

### 1.4 Sensor/Source Scoping in Query Syntax

Sensors should be expressible as filters within the query, not as syntactic constructs:

```sql
-- Good: sensor as a filter predicate
WHERE sensor = "crowdstrike" AND severity >= "high"

-- Good: OCSF event class as a filter  
WHERE class_uid = 2001 AND sensor = "claroty"

-- Bad: sensor as a FROM clause table name (implies JOINs)
FROM crowdstrike.alerts WHERE ...
```

The `sensor` field is a virtual field added by Prism during OCSF normalization (it is not part of OCSF but is essential for provenance). Similarly, `client_id` is a virtual field for multi-client scoping within the query.

However, see section 2 for the interaction between query-level scoping and tool-parameter scoping.

### 1.5 Security Limits -- Adopt from Axiathon

Axiathon's security limits are well-calibrated and CWE-cited. Adopt them with minor adjustments:

| Limit | Axiathon Value | Prism Value | Rationale |
|-------|---------------|-------------|-----------|
| Max query length | 64KB | 64KB | Sufficient for any reasonable query. CWE-400. |
| Max nesting depth | 128 | 64 | AI-generated queries are shallower. Reduce to limit resource consumption. |
| Max pipe stages | 64 | 32 | AI agents rarely chain more than 5-10 stages. |
| Max regex pattern | 1024 bytes | 1024 bytes | Adopt as-is. CWE-1333. |
| Regex engine | Rust `regex` (finite automaton) | Same | Immune to catastrophic backtracking. Non-negotiable. |
| CIDR validation | At parse time | At parse time | CWE-20. |
| Integer overflow | i128 intermediate | Same | CWE-190. |
| **New: Max result set** | N/A | 10,000 events | Prevent memory exhaustion from unbounded queries. |
| **New: Query timeout** | N/A | 30 seconds | Prevent long-running queries from blocking the MCP server. |
| **New: Max concurrent queries** | N/A | 4 per session | Single analyst, multiple parallel tool calls. |

---

## 2. Query Scoping Syntax: Tool Parameters vs Query Language

### 2.1 Analysis of Three Options

**Option A: Scoping in tool parameters only.**
```json
{
  "tool": "query_events",
  "arguments": {
    "clients": ["acme-corp", "globex-inc"],
    "sensors": ["crowdstrike", "claroty"],
    "query": "severity >= \"high\" AND device.ip = \"10.0.0.1\""
  }
}
```

Pros: Clean separation of concerns. Query language stays pure (no Prism-specific constructs). Easy to validate scoping independently from query parsing. Tool parameters are schema-validated via JSON Schema.

Cons: AI must manage two separate "languages" -- the query syntax and the tool parameter syntax. Scoping information is split across two places.

**Option B: Scoping in the query itself.**
```json
{
  "tool": "query_events",
  "arguments": {
    "query": "FROM events WHERE client = \"acme-corp\" AND sensor = \"crowdstrike\" AND severity >= \"high\""
  }
}
```

Pros: Single expression contains all query intent. Familiar SQL-like pattern. No ambiguity about what scope applies.

Cons: More complex parser (must handle client/sensor as pseudo-columns). No schema validation on scoping parameters. Query parsing errors can be confusing when mixing scoping with data filters.

**Option C: Both -- tool parameters set scope, query operates within it.**
```json
{
  "tool": "query_events",
  "arguments": {
    "clients": ["acme-corp"],
    "sensors": ["crowdstrike", "claroty"],
    "query": "severity >= \"high\" AND device.ip = \"10.0.0.1\""
  }
}
```

With the rule that:
- If `clients` is omitted or null, query spans all clients the analyst has access to.
- If `sensors` is omitted or null, query spans all sensors configured for the selected clients.
- If the query also contains `client = "..."` or `sensor = "..."` predicates, they are intersected with the tool parameters (narrowing, never widening).

### 2.2 Recommendation: Option C (Both)

**Option C is best for AI agent ergonomics.** Here is why:

1. **Tool parameters provide guardrails.** The `clients` and `sensors` parameters are JSON Schema validated. The AI cannot make a syntax error on scoping -- it is a typed array of strings. The `outputSchema` documents exactly what values are valid.

2. **Query language stays focused on data filtering.** The query expression deals with OCSF fields (severity, device.ip, process.name), not infrastructure concerns (which client, which sensor). This keeps queries portable and reusable across different scoping contexts.

3. **Graceful defaults for the common case.** Most AI queries will not specify `clients` or `sensors` at all, defaulting to "all clients, all sensors." The AI only narrows scope when the human explicitly asks about a specific client or sensor. This matches the MSSP workflow: "show me critical alerts" (all clients) vs "show me Acme's CrowdStrike alerts" (scoped).

4. **Intersection semantics prevent scope escalation.** If the tool parameters say `clients: ["acme-corp"]` but the query contains `client = "globex-inc"`, the intersection is empty -- no results. The AI cannot accidentally widen scope via the query language.

5. **Backwards-compatible with filter mode.** Simple filter queries like `severity >= "high"` work without any scoping in the query. The tool parameters handle it.

### 2.3 Tool Parameter Schema

```json
{
  "name": "query_events",
  "description": "Query OCSF-normalized security events across sensors and clients. Results are unified across all queried sensors into a single OCSF event stream. Omit clients/sensors to query all available.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "query": {
        "type": "string",
        "description": "AxiQL query expression. Supports filter mode (boolean expressions), SQL mode (SELECT ... FROM events WHERE ...), and pipe mode (events | where ... | stats ...). See explain_query_syntax for full grammar."
      },
      "clients": {
        "type": "array",
        "items": { "type": "string" },
        "description": "Client IDs to query. Omit or null for all clients. Example: [\"acme-corp\", \"globex-inc\"]"
      },
      "sensors": {
        "type": "array",
        "items": { "type": "string" },
        "description": "Sensor types to query. Omit or null for all sensors. Values: crowdstrike, cyberint, claroty, armis"
      },
      "time_range": {
        "type": "object",
        "properties": {
          "last": { "type": "string", "description": "Relative time: 1h, 24h, 7d, 30d" },
          "from": { "type": "string", "format": "date-time" },
          "to": { "type": "string", "format": "date-time" }
        },
        "description": "Time range for the query. Defaults to last 24 hours if omitted."
      },
      "limit": {
        "type": "integer",
        "default": 25,
        "maximum": 1000,
        "description": "Maximum number of results to return"
      },
      "cursor": {
        "type": "string",
        "description": "Pagination cursor from a previous query result"
      }
    },
    "required": ["query"]
  }
}
```

---

## 3. Alias System Design

### 3.1 Storage: Separate Section in Client Config TOML

**Recommendation: aliases in the main Prism TOML config, not a separate file.**

Rationale:
- Prism already uses TOML for all configuration (per the product brief).
- A separate `aliases.toml` creates file management complexity (loading order, error reporting, hot-reload of one file but not the other).
- Aliases are configuration, not data. They belong with the rest of the configuration.
- Per-client aliases naturally nest under the existing `[clients.acme-corp]` TOML sections.

```toml
# prism.toml

# Global aliases (apply to all clients)
[aliases]
recent_critical = "severity = \"critical\" AND time > now()-24h"
ot_devices = "device.category = \"ot\" OR device.category = \"iot\""
open_alerts = "status = \"open\" OR status = \"in_progress\""

# Parameterized aliases
[aliases.recent_alerts]
template = "severity >= \"{severity}\" AND time > now()-{hours}h"
defaults = { severity = "high", hours = "24" }

# Per-client aliases
[clients.acme-corp]
sensors = ["crowdstrike", "claroty"]

[clients.acme-corp.aliases]
ot_risk = "device.category = \"ot\" AND severity >= \"high\" AND sensor = \"claroty\""
vip_hosts = "device.hostname in (\"dc01.acme.com\", \"scada-primary.acme.com\")"

[clients.globex-inc]
sensors = ["crowdstrike", "armis"]

[clients.globex-inc.aliases]
campus_devices = "device.subnet = \"10.20.0.0/16\""
```

### 3.2 Alias Scoping Rules

| Scope | Visibility | Override Behavior |
|-------|-----------|-------------------|
| Global | All clients | Base layer |
| Per-client | Only when querying that client | Overrides global alias of same name |

**Cross-client query with per-client aliases:**

When querying all clients (`clients: null`) and an alias exists for some clients but not others:

**Recommendation: alias resolution fails with a clear error.** Do not silently skip the alias for clients that do not have it. Do not silently use only the clients that have it.

```json
{
  "error": "alias_resolution_failed",
  "alias": "ot_risk",
  "message": "Alias 'ot_risk' is defined for client 'acme-corp' but not for clients 'globex-inc', 'initech'. Either define 'ot_risk' as a global alias, or scope your query to clients that define it.",
  "defined_in": ["acme-corp"],
  "missing_in": ["globex-inc", "initech"],
  "suggestion": "Add clients: [\"acme-corp\"] to scope the query, or define [aliases] ot_risk globally."
}
```

This prevents subtle bugs where the analyst thinks they are querying all clients but actually only getting results from clients that define the alias.

### 3.3 Alias Composability

**Allow aliases to reference other aliases, with a max depth of 3.**

Rationale:
- Depth 1 (no composition) is too restrictive. Analysts want to build on existing aliases: `critical_ot = "ot_devices AND severity = \"critical\""` where `ot_devices` is itself an alias.
- Unlimited depth creates debugging nightmares and potential cycles.
- Depth 3 is sufficient for practical use cases and easy to reason about.

**Cycle detection:** Perform at config load time, not at query time. If `alias_a` references `alias_b` which references `alias_a`, reject the configuration with a clear error. Use a simple visited-set during resolution.

**Resolution order:** Inner-to-outer. Resolve the innermost alias first, then substitute upward. This matches intuitive expectations.

### 3.4 Parameterized Aliases

**Yes, support parameterized aliases.**

This is high-value for AI agents. The AI can call a parameterized alias without knowing the underlying query structure:

```toml
[aliases.recent_by_severity]
template = "severity >= \"{severity}\" AND time > now()-{hours}h"
defaults = { severity = "high", hours = "24" }
description = "Recent alerts filtered by minimum severity and time window"
```

Usage in a query:
```
recent_by_severity(severity="critical", hours=4)
```

Or via MCP tool parameters:
```json
{
  "tool": "query_events",
  "arguments": {
    "query": "recent_by_severity(severity=\"critical\", hours=4)"
  }
}
```

**Parameter validation rules:**
- All parameters must have defaults (so the alias is usable without any parameters).
- Parameter values are string-substituted, then the resulting query is re-parsed. This means the substitution produces a valid query expression or the parse fails.
- Parameters cannot inject arbitrary query syntax (the substituted string is validated by the same parser with the same security limits).

### 3.5 Alias Resolution Timing

**Resolve at query parse time, not query plan time.**

Rationale:
- The parser needs the expanded query to produce a correct AST. If aliases are resolved later, the parser produces an AST with opaque alias nodes that downstream processing cannot optimize or validate.
- Parse-time resolution means the type checker sees the full expanded query and can validate field types, operator compatibility, etc.
- The expanded query should be included in the response for AI transparency (see section 4.4).

Resolution pipeline:
```
Raw query string
  -> Alias detection (identify alias references in the query)
  -> Parameter substitution (if parameterized)
  -> Recursive alias expansion (up to depth 3)
  -> Cycle detection
  -> Security limit check on expanded query (must still fit in 64KB)
  -> Parse expanded query through Chumsky parser
  -> Type checking
  -> Query plan generation
```

### 3.6 MCP Tools for Alias Management

```
create_alias    -- Create or update an alias (global or per-client)
list_aliases    -- List all aliases visible to a client (global + per-client, merged)
delete_alias    -- Delete an alias (requires confirmation token if alias is referenced by other aliases)
explain_alias   -- Show the alias definition, parameter defaults, expanded query, and which clients define it
```

**`create_alias` is a write operation** and should follow Prism's write-operation gating pattern (confirmation token for updates to existing aliases, immediate for new creates). This prevents accidental alias overwrites.

**`list_aliases` output should show resolution context:**
```json
{
  "aliases": [
    {
      "name": "ot_risk",
      "scope": "client:acme-corp",
      "template": "device.category = \"ot\" AND severity >= \"high\" AND sensor = \"claroty\"",
      "is_parameterized": false,
      "referenced_by": ["critical_ot"],
      "references": []
    },
    {
      "name": "recent_alerts",
      "scope": "global",
      "template": "severity >= \"{severity}\" AND time > now()-{hours}h",
      "is_parameterized": true,
      "defaults": { "severity": "high", "hours": "24" },
      "referenced_by": [],
      "references": []
    }
  ]
}
```

---

## 4. AI Agent Ergonomics

### 4.1 Output Schema Design

The `query_events` tool should define an `outputSchema` that the LLM can rely on for consistent field extraction. Following the patterns from Prism's AI-consumable design research:

```json
{
  "outputSchema": {
    "type": "object",
    "properties": {
      "query_context": {
        "type": "object",
        "properties": {
          "original_query": { "type": "string" },
          "expanded_query": { "type": "string", "description": "Query after alias expansion" },
          "clients_queried": { "type": "array", "items": { "type": "string" } },
          "sensors_queried": { "type": "array", "items": { "type": "string" } },
          "time_range_applied": { "type": "object" },
          "total_results": { "type": "integer" },
          "returned_results": { "type": "integer" },
          "is_truncated": { "type": "boolean" },
          "next_cursor": { "type": ["string", "null"] },
          "execution_time_ms": { "type": "integer" }
        }
      },
      "events": {
        "type": "array",
        "items": {
          "type": "object",
          "description": "OCSF-normalized event with provenance metadata"
        }
      },
      "sensor_errors": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "sensor": { "type": "string" },
            "client": { "type": "string" },
            "error": { "type": "string" },
            "is_partial": { "type": "boolean" }
          }
        },
        "description": "Sensors that failed to respond. Results are partial if any sensor errored."
      }
    }
  }
}
```

Key design choices:
- **`query_context` echoes back what was actually queried.** The LLM can verify its query was interpreted correctly.
- **`expanded_query` shows alias resolution.** Transparency for debugging.
- **`sensor_errors` is a first-class field.** Partial results (3 of 4 sensors responded) are common in multi-sensor queries. The AI must know which sensors failed to avoid presenting partial data as complete.
- **`is_truncated` + `next_cursor`** are explicit pagination signals. The AI can decide whether to paginate based on the human's question.

### 4.2 Error Messages with Syntax Help

**Yes, absolutely.** Error messages should include actionable syntax help. This is critical for AI agents that will attempt to self-correct:

```json
{
  "error": "query_parse_error",
  "message": "Unexpected token 'WERE' at position 23. Did you mean 'WHERE'?",
  "position": { "line": 1, "column": 23 },
  "context": "SELECT * FROM events WERE severity = \"high\"",
  "suggestion": "Replace 'WERE' with 'WHERE': SELECT * FROM events WHERE severity = \"high\"",
  "help": "AxiQL SQL mode syntax: SELECT [fields] FROM events WHERE [conditions] [GROUP BY fields] [ORDER BY fields] [LIMIT n]"
}
```

For type errors:
```json
{
  "error": "query_type_error",
  "message": "Cannot compare field 'severity' (string) with operator '>=' against value 42 (integer). Severity values are strings: \"unknown\", \"informational\", \"low\", \"medium\", \"high\", \"critical\", \"fatal\".",
  "suggestion": "Use: severity >= \"high\""
}
```

For unknown fields:
```json
{
  "error": "unknown_field",
  "message": "Field 'source_ip' is not a known OCSF field.",
  "similar_fields": ["src_endpoint.ip", "src.ip"],
  "suggestion": "Did you mean 'src.ip'? (alias for src_endpoint.ip)"
}
```

### 4.3 The `explain_query` Tool

**Yes, implement this.** It is valuable for three reasons:

1. **Debugging.** When results are unexpected, the AI can call `explain_query` to verify its query was parsed and planned correctly.
2. **Learning.** The AI can explore the query language by explaining queries before executing them.
3. **Transparency.** For high-stakes queries (write operations triggered by query results), the analyst can see the query plan before execution.

```json
{
  "name": "explain_query",
  "description": "Parse and explain a query without executing it. Shows alias expansion, parsed AST, field resolution, and which sensors/APIs would be called.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "query": { "type": "string" },
      "clients": { "type": "array", "items": { "type": "string" } },
      "sensors": { "type": "array", "items": { "type": "string" } }
    },
    "required": ["query"]
  }
}
```

Output:
```json
{
  "parsed_mode": "sql",
  "original_query": "SELECT src.ip, count(*) FROM events WHERE recent_critical GROUP BY src.ip",
  "alias_expansion": {
    "recent_critical": "severity = \"critical\" AND time > now()-24h"
  },
  "expanded_query": "SELECT src.ip, count(*) FROM events WHERE severity = \"critical\" AND time > now()-24h GROUP BY src.ip",
  "field_resolution": {
    "src.ip": { "ocsf_path": "src_endpoint.ip", "resolution": "alias" },
    "severity": { "ocsf_path": "severity_id", "resolution": "direct" },
    "time": { "ocsf_path": "time", "resolution": "direct" }
  },
  "execution_plan": {
    "sensors_to_query": ["crowdstrike", "cyberint", "claroty", "armis"],
    "api_filters_pushed": {
      "crowdstrike": "filter=severity:critical+created_on:>1713052800",
      "claroty": "{ \"filters\": [{ \"field\": \"severity\", \"op\": \"gte\", \"value\": 4 }] }",
      "cyberint": "severity=very_high",
      "armis": "AQL: severity:Critical timeFrame:\"1 Day\""
    },
    "post_fetch_operations": ["OCSF normalization", "merge", "GROUP BY src_endpoint.ip", "count aggregation"]
  },
  "estimated_cost": "4 API calls (1 per sensor), estimated 2-5 seconds"
}
```

The `api_filters_pushed` section is particularly valuable: it shows the analyst exactly what each sensor API will receive, which helps debug cases where a sensor returns unexpected results due to filter translation differences.

### 4.4 Alias-Expanded Query Transparency

**Yes, always show the expanded query.** Include it in both `query_events` responses (in `query_context.expanded_query`) and in `explain_query` output. The AI agent needs to verify that alias expansion produced the intended query, especially when debugging with the analyst.

### 4.5 Additional Ergonomic Tool: `query_syntax_help`

Consider a lightweight tool that returns query syntax documentation:

```json
{
  "name": "query_syntax_help",
  "description": "Returns AxiQL query language syntax reference. Use this when unsure about query syntax.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "topic": {
        "type": "string",
        "enum": ["filter", "sql", "pipe", "operators", "functions", "aliases", "fields", "time"],
        "description": "Specific syntax topic. Omit for full reference."
      }
    }
  }
}
```

This is cheaper than the AI guessing at syntax and generating parse errors.

---

## 5. Chumsky 0.10 Parser Design

### 5.1 Reusability of Axiathon's Parser

**The axiathon parser grammar is available as a reference** (axiathon is in the semport directory as a reference repo). However, direct code reuse requires careful consideration:

**What can be reused conceptually (grammar design):**
- Three-mode detection logic (first-token dispatch)
- FilterExpr AST with 11 variants (And, Or, Not, Compare, In, Contains, Matches, Between, IsNull, Exists, Cidr)
- SQL SELECT AST (projections, aggregations, GROUP BY, ORDER BY, LIMIT)
- Pipe stage AST (stats, sort, head, tail, dedup, fields)
- Security limit checking (pre-parse length check, depth counter in recursive parsers, pipe stage counter)
- Type system approach (TypeConstraint, TypeError, FieldWarning)

**What must be modified for Prism:**
- Remove `tenant_id` from the query language entirely (handled by tool parameters as `client_id`)
- Remove `class_uid` routing (Prism queries across all OCSF classes; class filtering is just another WHERE predicate)
- Add `sensor` as a recognized virtual field (filter predicate, not a FROM clause)
- Add alias resolution as a pre-parse phase (axiathon's aliases are resolved inside the parser; Prism should resolve before parsing for cleaner separation)
- Add parameterized alias syntax `alias_name(param=value)` to the lexer
- Remove DataFusion-specific AST nodes (Prism does not use DataFusion)
- Add `time_range` functions (`now()`, relative time expressions like `now()-24h`)

### 5.2 Three-Mode Detection Strategy

The mode detection should happen at the lexer level:

```rust
enum QueryMode {
    Filter,  // Default: bare boolean expression
    Sql,     // Starts with SELECT or FROM
    Pipe,    // Starts with identifier followed by |
}

fn detect_mode(input: &str) -> QueryMode {
    let trimmed = input.trim();
    if trimmed.starts_with_ignore_case("SELECT") || trimmed.starts_with_ignore_case("FROM") {
        QueryMode::Sql
    } else if trimmed.contains('|') && !trimmed.starts_with('"') {
        // Heuristic: pipes in the query suggest pipe mode
        // But pipes inside string literals should not trigger this
        QueryMode::Pipe
    } else {
        QueryMode::Filter
    }
}
```

Note: axiathon's implementation uses Chumsky's `choice()` combinator to try all three parsers and pick the first that succeeds. This is cleaner than heuristic detection but has a performance cost (three parse attempts in the worst case). For Prism's use case (small queries from AI agents), the performance difference is negligible. Recommendation: use `choice()` for correctness, with filter mode last (as the most permissive grammar).

### 5.3 Parser Architecture

```
                    +-----------------+
                    |  Raw Query      |
                    |  (String)       |
                    +--------+--------+
                             |
                    +--------v--------+
                    |  Security Check |
                    |  (length, etc)  |
                    +--------+--------+
                             |
                    +--------v--------+
                    |  Alias Resolve  |
                    |  (pre-parse)    |
                    +--------+--------+
                             |
                    +--------v--------+
                    |  Chumsky Lexer  |
                    |  (tokens)       |
                    +--------+--------+
                             |
              +--------------+--------------+
              |              |              |
     +--------v---+  +------v------+  +----v--------+
     | Filter     |  | SQL         |  | Pipe        |
     | Parser     |  | Parser      |  | Parser      |
     +--------+---+  +------+------+  +----+--------+
              |              |              |
              +--------------+--------------+
                             |
                    +--------v--------+
                    |  Unified AST    |
                    |  (AxiQLStatement)|
                    +--------+--------+
                             |
                    +--------v--------+
                    |  Type Checker   |
                    +--------+--------+
                             |
                    +--------v--------+
                    |  Query Plan     |
                    |  (sensor fan-out)|
                    +-----------------+
```

### 5.4 Key Chumsky 0.10 Considerations

Based on model knowledge (flagged -- verify against current Chumsky docs before implementation):

- Chumsky 0.10 uses a zero-copy architecture with `&str` input by default. This aligns well with Prism's use case (small query strings, no streaming).
- The `recursive()` combinator is the natural choice for nested boolean expressions with depth limiting.
- Error recovery via `recover_with()` is available but axiathon did not implement it (marked as TODO Story 5.2). Prism should implement error recovery to provide better error messages to the AI agent.
- Chumsky 0.10 moved from `Simple<char>` to a richer error type. Use this for structured error messages with position information.
- The `select!` macro simplifies token matching for keywords.

**Important caveat from axiathon synthesis:** The production Chumsky parser is NOT connected to any execution engine. Axiathon's entire working pipeline uses a simpler Pest parser. This means the Chumsky parser's AST has not been battle-tested against real execution. Prism should wire the parser to execution from the first commit (per lesson P3-2).

### 5.5 Licensing

The axiathon synthesis does not explicitly state the license. The semport analysis references ocsf-proto-gen as MIT licensed. Axiathon's license should be verified before any code reuse. Even if the license permits reuse, the recommendation is to rewrite the parser from scratch using axiathon's grammar as a design reference, because:

1. Prism's AST needs are different (no tenant_id in query, sensor as virtual field, parameterized aliases).
2. Prism should have error recovery from the start (axiathon does not).
3. Writing the parser ensures the team understands every production path (per P3-2: wire parser to execution from first commit).

---

## 6. Summary of Recommendations

### Query Language

| Decision | Recommendation | Confidence |
|----------|---------------|------------|
| Query modes | All three (filter, SQL, pipe); filter as default for AI | HIGH |
| Cross-sensor JOINs | No explicit JOINs; implicit unified OCSF table | HIGH |
| Sensor scoping | Virtual `sensor` field as a filter predicate | HIGH |
| Security limits | Adopt axiathon's limits, tighten nesting/pipe limits | HIGH |
| Parser | Chumsky 0.10, rewrite from scratch using axiathon grammar as reference | HIGH |
| Error recovery | Implement from day 1 (axiathon skipped this) | HIGH |

### Query Scoping

| Decision | Recommendation | Confidence |
|----------|---------------|------------|
| Scoping model | Option C: tool params + query predicates, intersection semantics | HIGH |
| Default scope | All clients, all sensors when params omitted | HIGH |
| Scope escalation | Query predicates can only narrow, never widen tool param scope | HIGH |

### Alias System

| Decision | Recommendation | Confidence |
|----------|---------------|------------|
| Storage | In main prism.toml config, per-client sections | HIGH |
| Scopes | Global + per-client; per-client overrides global | HIGH |
| Composability | Max depth 3, cycle detection at config load | HIGH |
| Parameterized | Yes, with required defaults for all parameters | HIGH |
| Resolution timing | Pre-parse (before Chumsky parser) | HIGH |
| Cross-client alias conflicts | Fail with clear error, do not silently skip | HIGH |
| MCP tools | create_alias, list_aliases, delete_alias, explain_alias | HIGH |

### AI Ergonomics

| Decision | Recommendation | Confidence |
|----------|---------------|------------|
| outputSchema | Yes, with query_context echo-back and sensor_errors | HIGH |
| Error messages | Include syntax help and suggestions | HIGH |
| explain_query tool | Yes, shows alias expansion + field resolution + API plan | HIGH |
| Expanded query in response | Always show in query_context | HIGH |
| query_syntax_help tool | Yes, lightweight syntax reference | MEDIUM |

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| Context7 | 0 (denied) | Would have looked up Chumsky 0.10 docs |
| WebSearch | 0 (denied) | Would have searched MCP query tool patterns |
| WebFetch | 0 | N/A |
| Training data | 4 areas | Chumsky 0.10 API patterns, SQL query language design for AI agents, alias system design patterns, TOML configuration patterns |

**Total MCP tool calls:** 0 (both available tools were permission-denied)
**Training data reliance:** HIGH -- All external research tools were denied. Chumsky 0.10 API details, MCP outputSchema patterns, and AI agent query ergonomics are based on model knowledge (May 2025 cutoff). Recommendations for query language design and alias systems are well-grounded in the axiathon synthesis document and Prism's existing architecture documents, which were read directly. Chumsky 0.10 specifics should be verified against current documentation before implementation.
