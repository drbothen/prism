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
- Each queryable RocksDB domain is registered as a DataFusion table in the query engine's catalog under the `prism` namespace:
  - `prism.alerts` — Alert records (StorageDomain::Alerts)
  - `prism.cases` — Case records (StorageDomain::Cases)
  - `prism.rules` — Detection rule definitions (StorageDomain::DetectionRules)
  - `prism.schedules` — Schedule definitions and state (StorageDomain::Schedules)
  - `prism.diff_results` — Differential result history (StorageDomain::DiffResults)
  - `prism.audit` — Buffered audit log entries (StorageDomain::AuditBuffer)
  - `prism.aliases` — Alias definitions and metadata (StorageDomain::Aliases)
- Each internal table's Arrow schema is derived from the corresponding entity definition (e.g., `prism.alerts` schema matches the Alert entity's key attributes)
- Internal tables implement DataFusion's `TableProvider` trait, with `scan()` reading from the RocksDB domain via prefix-scan and deserializing bincode values into Arrow RecordBatches
- Internal tables are registered at startup and available for the lifetime of the process
- Internal tables are queryable via the same `query` MCP tool (BC-2.11.001) using the same AxiQL syntax as external tables
- Virtual fields `sensor = "prism"` and `source = "{table_name}"` are injected for internal table results (e.g., `sensor = "prism"`, `source = "alerts"`)
- **Write queries are NOT supported via AxiQL.** Internal tables are read-only in the query engine. Mutations to alerts, cases, rules, schedules, and aliases go through their dedicated MCP tools (`acknowledge_alert`, `update_case`, `create_rule`, `create_schedule`, `create_alias`, etc.). Attempting a SQL INSERT/UPDATE/DELETE against an internal table returns `E-QUERY-010: "Internal tables are read-only. Use the dedicated MCP tool for mutations."`
- Cross-source queries are supported: an AxiQL query can reference both external sensor tables and internal Prism tables in the same query (e.g., `FROM prism.alerts a, crowdstrike.alerts cs WHERE a.matched_event_ids CONTAINS cs.event_uid`)
- The `explain_query` tool (BC-2.11.010) includes internal tables in its available sources listing

## Invariants
- DI-008: Client data separation — internal table queries enforce `client_id` scoping. `prism.alerts` for client "acme" returns only Acme's alerts.
- DI-004: Audit completeness — queries against internal tables are audit-logged identically to external table queries
- Internal table schemas are stable within a Prism release version (schema changes require migration)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-QUERY-010` | SQL write statement (INSERT/UPDATE/DELETE) targets an internal table | Structured error: "Internal tables are read-only via AxiQL. Use the dedicated MCP tool: {tool_name}" |
| `E-STATE-003` | RocksDB domain is corrupted or unreadable during table scan | Structured error with domain name and recovery suggestion (restart, check state_dir) |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-15-011 | Query references `prism.alerts` but no alerts exist | Empty result set with `total_available: 0`, not an error |
| EC-15-012 | Cross-source query joins `prism.alerts` with `crowdstrike.alerts` | Both tables are registered in the same SessionContext; DataFusion handles the join. External table triggers API fan-out; internal table reads from RocksDB. |
| EC-15-013 | `prism.audit` queried — audit table is read-only | Returns buffered audit entries. The audit table is always read-only (append-only invariant DI-004 maintained). |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-028 |
| L2 Invariants | DI-004, DI-008 |
| Related BCs | BC-2.11.001 (query tool), BC-2.11.005 (materialization), BC-2.11.012 (virtual fields), BC-2.15.001 (RocksDB init), BC-2.15.002 (domain KV operations) |
| Priority | P0 |
