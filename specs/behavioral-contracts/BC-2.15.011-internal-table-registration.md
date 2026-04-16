---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "Platform Infrastructure"
capability: "CAP-028"
---

# BC-2.15.011: Internal Table Registration — RocksDB Domains as DataFusion Tables

## Preconditions
- Prism is starting up with a valid RocksDB state directory (CAP-019)
- The StorageDomain enum defines domains with queryable data: Alerts, Cases, Schedules, DiffResults, DetectionRules, AuditBuffer, Aliases
- The query engine (CAP-015) is initializing its table catalog

## Postconditions
- Each queryable RocksDB domain is registered as a DataFusion table using underscore-delimited names (dots are not valid in PrismQL identifiers):
  - `prism_alerts` — Alert records (StorageDomain::Alerts)
  - `prism_cases` — Case records (StorageDomain::Cases)
  - `prism_rules` — Detection rule definitions (StorageDomain::DetectionRules)
  - `prism_schedules` — Schedule definitions and state (StorageDomain::Schedules)
  - `prism_diff_results` — Differential result metadata (StorageDomain::DiffResults). Exposes DiffState metadata columns only (`query_hash`, `client_id`, `previous_results_hash`, `epoch`, `counter`, `last_diff_time`). The raw sensor data inside `previous_results` is NOT exposed as queryable columns — use `get_diff_results` MCP tool to inspect diff content. Example: `SELECT query_hash, client_id, epoch, counter, last_diff_time FROM prism_diff_results WHERE client_id = 'acme'`
  - `prism_audit` — Buffered audit log entries (StorageDomain::AuditBuffer)
  - `prism_aliases` — Alias definitions and metadata (StorageDomain::Aliases)
- Each internal table's Arrow schema is derived from the corresponding entity definition (e.g., `prism_alerts` schema matches the Alert entity's key attributes)
- Internal tables implement DataFusion's `TableProvider` trait, with `scan()` reading from the RocksDB domain via prefix-scan and deserializing bincode values into Arrow RecordBatches
- Internal tables are registered at startup and available for the lifetime of the process
- Internal tables are queryable via the same `query` MCP tool (BC-2.11.001) using the same PrismQL syntax as external tables: `SELECT * FROM prism_alerts WHERE severity_id >= 4`
- Virtual fields `_sensor = "prism"` and `_source_table = "{table_name}"` are injected for internal table results (e.g., `_sensor = "prism"`, `_source_table = "prism_alerts"`)
- **Write queries are NOT supported via PrismQL.** Internal tables are read-only in the query engine. Mutations go through dedicated MCP tools. Attempting SQL INSERT/UPDATE/DELETE returns `E-QUERY-010`.
- **Cross-source JOINs supported.** Internal tables can be JOINed with external sensor tables in both SQL and pipe mode (query-engine.md). Example: `SELECT al.alert_id, h.hostname FROM prism_alerts al JOIN crowdstrike_hosts h ON al.device_ip = h.device_ip` enriches internal alerts with sensor host context. Same-type cross-sensor correlation uses composite sources (`FROM EVENTS`); cross-type correlation uses JOINs.
- The `explain_query` tool (BC-2.11.010) includes internal tables in its available sources listing

## Internal Table Query Semantics
- The 10K materialization limit (DI-019) applies only to external table fan-out (records fetched from sensor APIs), not to internal RocksDB reads
- Internal table scans are bounded by a configurable limit (default 50K rows, configurable via `PRISM_MAX_INTERNAL_TABLE_SCAN` environment variable) to prevent unbounded RocksDB iteration. When the limit is hit, the query returns the records collected so far with `_meta.scan_truncated: true` and `_meta.scan_limit: 50000` in the response metadata (unlike external queries which return an error on materialization limit — internal table truncation returns partial results because the data is local and pagination is not needed)
- The `clients` scoping parameter applies to both external and internal tables: `prism_alerts` for `clients: ["acme"]` returns only Acme's alerts, just as `crowdstrike_detections` returns only Acme's CrowdStrike detections
- The `limit` parameter on the `query` tool applies to the final result set
- Each internal table query uses its own DataFusion SessionContext (same as external queries)

## Invariants
- DI-008: Client data separation — internal table queries enforce `client_id` scoping. `prism_alerts` for client "acme" returns only Acme's alerts.
- DI-004: Audit completeness — queries against internal tables are audit-logged identically to external table queries
- Internal table schemas are stable within a Prism release version (schema changes require migration)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-QUERY-010` | SQL write statement (INSERT/UPDATE/DELETE) targets an internal table | Structured error: "Internal tables are read-only via PrismQL. Use the dedicated MCP tool: {tool_name}" |
| `E-QUERY-011` | Query targets `prism_audit` but client lacks `audit.read` capability | Structured error: "Audit table requires audit.read capability. Grant via prism.toml [clients.{id}.capabilities]." |
| `E-STATE-003` | RocksDB domain is corrupted or unreadable during table scan | Structured error with domain name and recovery suggestion (restart, check state_dir) |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-15-011 | Query references `prism_alerts` but no alerts exist | Empty result set with `total_available: 0`, not an error |
| EC-15-012 | Analyst wants to correlate internal alerts with external sensor events | Use a JOIN: `SELECT al.alert_id, al.severity, h.hostname, h.os_version FROM prism_alerts al JOIN crowdstrike_hosts h ON al.device_ip = h.device_ip WHERE al.severity_id >= 4`. Both sides are registered in the same DataFusion SessionContext — internal table reads from RocksDB, external table triggers sensor API fan-out. Or in pipe mode: `FROM prism_alerts | join crowdstrike_hosts on device_ip | where severity_id >= 4` |
| EC-15-013 | `prism_audit` queried — audit table is read-only, requires `audit.read` capability | Returns buffered audit entries only if the querying client has `audit.read = "Allow"` in capabilities. If denied, returns `E-QUERY-011`. The audit table is always read-only (append-only invariant DI-004 maintained). Rationale: `prism_audit` exposes credential source types, operation outcomes, and capability check results — compliance infrastructure that warrants an explicit capability gate beyond the always-visible `query` tool. |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-028 |
| L2 Invariants | DI-004, DI-008 |
| Related BCs | BC-2.11.001 (query tool), BC-2.11.005 (materialization), BC-2.11.012 (virtual fields), BC-2.15.001 (RocksDB init), BC-2.15.002 (domain KV operations) |
| Priority | P0 |
