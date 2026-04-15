---
document_type: architecture-section
level: L3
section: "detection-rule-format"
version: "1.0"
status: draft
producer: architect
timestamp: 2026-04-15T18:00:00
phase: 1b
inputs: [prd.md, domain-spec/scheduled-detection-concept.md, operational-pipeline.md]
traces_to: ARCH-INDEX.md
---

# Detection Rule Format (.axd)

## Overview

Detection rules are defined in `.axd` (AxiQL Detection) format — a structured TOML document with AxiQL condition expressions. The `.axd` format is used for:
- File-based rules shipped with query packs (global baseline rules)
- Per-client rules stored in TOML config
- Runtime-created rules via `create_rule` MCP tool (serialized to the same format in RocksDB)

The `.axd` format is **not** a standalone DSL with its own parser. It reuses AxiQL filter expressions (axiql-grammar.md section 4) as the condition language, embedded within a TOML structure that declares the rule's metadata, match mode, and alert template.

## Rule Structure

```toml
[meta]
rule_id = "brute_force_detection"      # Unique within scope
name = "Brute Force Login Attempts"     # Human-readable name (REQUIRED)
severity = "high"                       # REQUIRED: critical, high, medium, low, info
description = "Detects multiple failed login attempts from the same source IP"
tags = ["authentication", "credential-access"]
mitre = ["T1110.001"]                   # ATT&CK technique IDs (optional)
enabled = true                          # Default: true

[condition]
mode = "correlation"                    # REQUIRED: "single", "correlation", "sequence"
source = "EVENTS"                       # AxiQL FROM source

# --- Single-event mode ---
# filter = 'activity_name = "Authentication" AND status = "Failure"'

# --- Correlation mode ---
filter = 'activity_name = "Authentication" AND status = "Failure"'
threshold = 5                           # Fire when >= threshold events match
window = "5m"                           # Sliding window duration (AxiQL duration literal)
group_by = ["src_endpoint.ip"]          # Group correlation by these fields

# --- Sequence mode ---
# [[condition.steps]]
# name = "recon"
# filter = 'activity_name = "PortScan"'
#
# [[condition.steps]]
# name = "exploit"
# filter = 'activity_name = "Exploitation" AND src_endpoint.ip = "${recon.src_endpoint.ip}"'
#
# window = "15m"                        # All steps must complete within this window
# group_by = ["src_endpoint.ip"]

[alert]
title = "Brute Force: ${threshold}+ failed logins from ${group.src_endpoint.ip}"   # REQUIRED
description = """Multiple failed authentication attempts detected from \
${group.src_endpoint.ip} within ${window}. Total: ${count} attempts."""             # REQUIRED
severity_override = ""                  # Optional: override meta.severity for this alert
additional_fields = ["src_endpoint.ip", "user.name", "device_hostname"]
```

## Condition Modes

### Single-Event Mode

Stateless per-record evaluation. Each new record from the differential results is tested against the `filter` AxiQL predicate. Fires immediately on match.

```toml
[condition]
mode = "single"
source = "EVENTS"
filter = 'severity_id >= 4 AND _sensor = "crowdstrike"'
```

**Compilation:** The `filter` string is parsed by the AxiQL parser (Chumsky) as a `FilterExpr`. It is then compiled to a DataFusion `WHERE` clause.

**Detection evaluation context:** Detection rule evaluation is serialized — one rule at a time per scheduler tick, sharing a single ephemeral `SessionContext` per tick. The differential RecordBatch is registered as a MemTable named `"events"` — matching the same table name used in ad-hoc queries so that rule filter expressions written against `FROM EVENTS` work identically in both contexts. The detection engine passes only the differential (added records) to the MemTable, not the full query result.

**Detection memory budget:** Detection evaluation reuses the scheduled query's existing `SessionContext` and `GreedyMemoryPool`. The query phase materializes records and drops its DataFusion plan (freeing pool memory), then the detection phase registers the differential RecordBatch as a new MemTable in the same context and evaluates rules sequentially. Since the pool tracks live allocations only, the query phase's freed memory becomes available for detection evaluation. No separate pool is created — the same per-query pool (200 MB cap at normal watchdog level) serves both phases sequentially.

Rules are evaluated one at a time within each schedule-execution task. Each rule's DataFusion plan is created, executed, and dropped before the next rule begins. Peak detection memory per rule is typically under 5 MB (the differential is a subset of the 10K-record materialization). With sequential evaluation, the detection phase adds minimal memory on top of the already-freed query phase allocations.

**Detection does not re-enter the query engine for sensor fan-out.** Detection rules evaluate against the already-materialized differential RecordBatch (registered as a MemTable in the reused SessionContext). The detection evaluation never triggers new sensor API calls — it only runs DataFusion SQL against in-memory data. This means detection rules do not consume HTTP semaphore permits or trigger adapter fan-out. The per-query and global HTTP semaphores are only consumed during the scheduled query's initial materialization phase, not during subsequent detection evaluation.

This design means the "Scheduled query overhead: ~50 MB" line in system-overview.md covers both the query materialization phase and the subsequent detection evaluation phase — they do not run simultaneously within a single execution task.

### Correlation Mode

Threshold over sliding time window with group-by. Requires `threshold`, `window`, and `group_by`.

```toml
[condition]
mode = "correlation"
source = "EVENTS"
filter = 'activity_name = "Authentication" AND status = "Failure"'
threshold = 5
window = "5m"
group_by = ["src_endpoint.ip"]
```

**Compilation:** The `filter` is compiled to DataFusion SQL. The correlation logic wraps it:
```sql
-- window_start is computed from the stored correlation state's window anchor, not wall-clock now()
-- Per DEC-035: event_time (the OCSF `time` field) is used for window math, not fetch time
SELECT src_endpoint.ip, COUNT(*) AS match_count
FROM events
WHERE <compiled_filter> AND time >= CAST('{window_start}' AS TIMESTAMP)
GROUP BY src_endpoint.ip
HAVING COUNT(*) >= 5
-- Note: window_start = max(stored_window_start, current_time - window_duration)
-- Events with time < window_start are pre-expired per DEC-035
```

State (per `(rule_id, group_key)`) is persisted to RocksDB `detection_state` domain.

**Important: Correlation operates on persisted state, not just the current differential.** Each differential produces new matching records that are added to the persisted sliding window in `detection_state`. The full window (accumulated across multiple schedule ticks) is evaluated for threshold. This means:
- Tick 1: 2 matching events → stored in window → count=2 (below threshold 5)
- Tick 2: 3 matching events → added to window → count=5 (threshold met → alert fires, window resets)

This is why DI-029 warns that `window >= interval`: if the window is shorter than the interval, events from the previous tick expire before the next tick evaluates, making threshold-based rules ineffective. With `window=60s` and `interval=300s`, each tick starts with an empty window (all previous events expired). The detection can only fire if a single tick's differential contains enough events. This is documented behavior, not a bug — but analysts must be warned via DI-029's config validation warning.

### Sequence Mode

Ordered multi-event pattern matching. Steps must match in order within the time window.

```toml
[condition]
mode = "sequence"
source = "EVENTS"
window = "15m"
group_by = ["src_endpoint.ip"]

[[condition.steps]]
name = "recon"
filter = 'activity_name = "PortScan"'

[[condition.steps]]
name = "exploit"
filter = 'activity_name = "Exploitation" AND src_endpoint.ip = "${recon.src_endpoint.ip}"'

[[condition.steps]]
name = "exfil"
filter = 'activity_name = "DataExfiltration"'
```

**Step variable interpolation:** `${step_name.field}` references bind to the first matched record's field value from a previous step. Variables are resolved at evaluation time, not parse time.

**Compilation:** Each step's `filter` is compiled to a DataFusion SQL template with named parameters for step variable references. At evaluation time:
1. Steps without variable references use cached compiled SQL (static — no re-compilation)
2. Steps with `${prev_step.field}` references use parameterized execution: the variable reference is compiled to a SQL parameter placeholder (`$1`, `$2`, ...), and the runtime value from the previous step's match is bound at evaluation time
3. If DataFusion's prepared statement API does not support the required parameter binding pattern, the step filter falls back to interpretive evaluation: the `FilterExpr` AST is walked directly against each record using Rust pattern matching on the typed `Value` enum. The step variable value (attacker-controlled sensor data) is always compared as a typed `Value::String` against the record's field value — never string-interpolated into SQL or any other query string. This is the same typed-comparison pattern used by the sensor adapter's PipelineExecutor variable interpolation (sensor-adapters.md). No SQL is generated in the fallback path.

The sequence tracker maintains progress per `(rule_id, group_key)` in RocksDB `detection_state`.

## Alert Template Variables

Alert `title` and `description` support variable interpolation with four resolution levels:

| Variable Pattern | Resolution |
|-----------------|------------|
| `${field_name}` | Value from the triggering record (single-event) or first record in group |
| `${group.field}` | Group key value (correlation/sequence) |
| `${count}` | Number of matching events (correlation) |
| `${threshold}` | Rule's configured threshold |
| `${window}` | Rule's configured window duration |
| `${step_name.field}` | Field from a specific sequence step's matched record |

Missing variables render as `<undefined:variable_name>` — never silently empty.

**Injection safety:** All variable values resolved from sensor data are passed through `InjectionScanner::scan()` before interpolation. If suspicious patterns are detected, the alert's `safety_flags` array is populated (same pattern as MCP response safety flags). The interpolated value is included as-is (flag, don't strip — analysts need forensic data), but the resulting alert title/description is served with `trust_level: "untrusted_external"` in MCP responses from `get_alert` and `list_alerts`. This ensures alert templates with sensor-provided values follow the same injection defense layering as all other MCP responses containing sensor data.

## Validation Rules (DI-024)

All rules are validated before activation:

1. `meta.name` is present and non-empty
2. `meta.severity` is a valid severity level
3. `alert.title` is present and non-empty
4. `alert.description` is present and non-empty
5. `condition.mode` is one of `single`, `correlation`, `sequence`
6. `condition.filter` parses as a valid AxiQL `FilterExpr`
7. Correlation mode requires `threshold` (>0), `window` (valid duration), `group_by` (non-empty)
8. Sequence mode requires at least one step, each step has `name` and valid `filter`
9. Step variable references (`${step_name.field}`) reference previously-declared steps only (no forward refs)
10. Regex patterns in filters compile successfully with 1 MB size limit
11. CIDR literals in filters parse as valid CIDR ranges
12. Total rule size does not exceed 16 KB
13. Condition nesting depth does not exceed 16 levels
14. `condition.source` must be an external composite source (`EVENTS`, `ALERTS`, `DEVICES`, `ASSETS`) or a specific sensor source (e.g., `crowdstrike_detections`). Prohibited sources: internal tables (`prism_alerts`, `prism_cases`, `prism_rules`) to prevent feedback loops, and `SESSIONS` (reserved, no sensor mapping — would silently fail with E-QUERY-015 on every tick). Validation rejects prohibited sources at rule creation time with `E-RULE-001`.

Invalid rules are rejected with a multi-error report (same pattern as config validation).

## Rule-to-SQL Compilation (BC-2.13.009)

The compilation pipeline:

1. Parse `condition.filter` as AxiQL `FilterExpr` via Chumsky parser
2. Walk the AST and classify each predicate as push-down or post-filter (same as query engine)
3. Generate DataFusion SQL `WHERE` clause from the `FilterExpr`
4. For correlation: wrap in `GROUP BY ... HAVING COUNT(*) >= threshold`
5. Register security UDFs (`subnet_contains`, `ioc_match`, `time_window`) for use in filter expressions
6. Static filter SQL is cached per rule — recompilation only on rule update. Sequence steps with `${step.field}` variable references use parameterized templates (cached) with runtime value binding (per evaluation)

## File Organization

```
config/
  rules/                          # Global baseline rules
    brute_force.axd
    port_scan.axd
    malware_detection.axd
  packs/
    incident-response/
      rules/                      # Pack-specific rules
        lateral_movement.axd
clients/
  {client_id}/
    rules/                        # Per-client rule overrides
      custom_rule.axd
```

Rules in `clients/{client_id}/rules/` with the same `rule_id` as a global rule override the global version for that client (three-scope resolution, BC-2.13.011).

## Rule Evaluation Order

Rules are evaluated sequentially within each scheduler tick, ordered by `rule_id` (deterministic lexicographic order). This ordering ensures:
1. **Rate limit fairness:** No rule is systematically starved by alphabetically-earlier rules consuming the global rate limit
2. **Deterministic behavior:** Same rules + same diff input = same alerts (important for testing)
3. **No write races:** Sequential evaluation prevents concurrent writes to `detection_state` for the same group key across different rules

The global alert rate limit (1,000/hr default) is checked atomically before each alert is persisted — under the same lock used for the per-rule rate limit check. If the global limit is reached mid-evaluation, remaining rules in the tick are still evaluated (for state updates) but their alerts are suppressed.

## Rate Limiting

To prevent runaway alert generation from detection loops (A-11 from adversarial review):

- **Per-rule alert rate limit:** Max 100 alerts per rule per hour (configurable via `meta.max_alerts_per_hour`, default 100)
- **Global alert rate limit:** Max 1,000 alerts per hour across all rules (configurable via `[defaults.limits]`)
- When a rule hits its rate limit, subsequent matches are counted but not alerted. The suppressed count is reported in the next alert when the rate limit window resets.
- **Per-rule active group key cap:** Max 10,000 distinct group keys per rule (configurable via `meta.max_group_keys`, default 10,000). When the cap is reached for a rule, the oldest group key is evicted (LRU) to make room for the new one. This prevents a single rule with a high-cardinality `group_by` (e.g., attacker rotating source IPs) from monopolizing the detection_state column family's 100 MB budget. The cap is evaluated during detection state writes, not at rule creation time.
- Rate limit state is persisted to RocksDB `detection_state` column family using the length-prefix encoding with type tag (see operational-pipeline.md): `[rule_id_len: u16][rule_id bytes][\x01][rate_limit]` — type tag `\x01` distinguishes rate limit entries from group correlation entries (type `\x00`) regardless of group_key content. RocksDB write happens after the in-memory Mutex is released (CI-004 compliance). A crash between Mutex release and RocksDB write can lose at most one increment per concurrent task — this bounded under-count is acceptable because the rate limit is a best-effort mechanism backed by the global 1,000/hr hard limit.
