---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-15"
capability: "CAP-028"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "47125c0"
traces_to:
  - "CAP-028"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.15.011: Internal Table Registration — RocksDB Domains as DataFusion Tables

## Description

At startup, Prism registers seven RocksDB domains as DataFusion table providers so
analysts can query internal state (alerts, cases, rules, schedules, diff results, audit
log, aliases) using the same PrismQL syntax as external sensor tables. Internal tables
implement `TableProvider` with `scan()` reading from RocksDB via prefix-scan and
deserializing bincode values into Arrow RecordBatches.

Internal table scans use a separate 50K-row soft limit (not the external 10K hard
limit) and return partial results with `_meta.scan_truncated: true` when the limit is
hit. Write operations (INSERT/UPDATE/DELETE) are rejected — mutations must go through
dedicated MCP tools. The `prism_audit` table requires the `audit.read` capability.

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
  - `prism_diff_results` — Differential result metadata (StorageDomain::DiffResults). Exposes DiffState metadata columns only (`query_hash`, `client_id`, `previous_results_hash`, `epoch`, `counter`, `last_diff_time`). The raw sensor data inside `previous_results` is NOT exposed as queryable columns.
  - `prism_audit` — Buffered audit log entries (StorageDomain::AuditBuffer)
  - `prism_aliases` — Alias definitions and metadata (StorageDomain::Aliases)
- Each internal table's Arrow schema is derived from the corresponding entity definition
- Internal tables implement DataFusion's `TableProvider` trait, with `scan()` reading from the RocksDB domain via prefix-scan and deserializing bincode values into Arrow RecordBatches
- Internal tables are registered at startup and available for the lifetime of the process
- Internal tables are queryable via the same `query` MCP tool (BC-2.11.001) using the same PrismQL syntax as external tables
- Virtual fields `_sensor = "prism"` and `_source_table = "{table_name}"` are injected for internal table results
- **Write queries are NOT supported via PrismQL.** Internal tables are read-only in the query engine. Mutations go through dedicated MCP tools. Attempting SQL INSERT/UPDATE/DELETE returns `E-QUERY-010`.
- **Cross-source JOINs supported.** Internal tables can be JOINed with external sensor tables in both SQL and pipe mode.
- Internal table scans are bounded by a configurable limit (default 50K rows, configurable via `PRISM_MAX_INTERNAL_TABLE_SCAN`). When the limit is hit, partial results are returned with `_meta.scan_truncated: true`.

## Invariants
- DI-008: Client data separation — internal table queries enforce `client_id` scoping
- DI-004: Audit completeness — queries against internal tables are audit-logged identically to external table queries
- **Capability gate:** `prism_audit` requires `audit.read` capability (E-QUERY-011 if denied)
- Internal table schemas are stable within a Prism release version (schema changes require migration)

## Error Conditions
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-QUERY-010` | SQL write statement (INSERT/UPDATE/DELETE) targets an internal table | Structured error: "Internal tables are read-only via PrismQL. Use the dedicated MCP tool: {tool_name}" |
| `E-QUERY-011` | Query targets `prism_audit` but client lacks `audit.read` capability | Structured error: "Audit table requires audit.read capability. Grant via prism.toml [clients.{id}.capabilities]." |
| `E-STATE-003` | RocksDB domain is corrupted or unreadable during table scan | Structured error with domain name and recovery suggestion (restart, check state_dir) |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-15-042 | Query references `prism_alerts` but no alerts exist | Empty result set with `total_available: 0`, not an error |
| EC-15-012 | Analyst wants to correlate internal alerts with external sensor events | Use a JOIN: `SELECT al.alert_id, al.severity, h.hostname FROM prism_alerts al JOIN crowdstrike_hosts h ON al.device_ip = h.device_ip` |
| EC-15-013 | `prism_audit` queried — audit table is read-only, requires `audit.read` capability | Returns buffered audit entries only if the querying client has `audit.read = "Allow"`. If denied, returns `E-QUERY-011`. |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — prism_alerts | `SELECT * FROM prism_alerts WHERE client_id='acme'` | Acme alerts only; `_sensor="prism"` virtual field injected |
| Cross-source JOIN | `SELECT al.alert_id, h.hostname FROM prism_alerts al JOIN crowdstrike_hosts h ON al.device_ip = h.device_ip` | Internal + external data joined correctly |
| Write rejected | `INSERT INTO prism_alerts VALUES (...)` | `E-QUERY-010` |
| audit.read denied | query `prism_audit` without capability | `E-QUERY-011` |
| Scan truncation | 60K alerts in DB | First 50K returned; `_meta.scan_truncated: true` |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (none) | audit.read gate covered transitively by VP-002 (deny-by-default capability); client_id scoping on internal table scans is a DataFusion execution integration test; no additional formal VP. |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-028 |
| L2 Invariants | DI-004, DI-008 |
| Related BCs | BC-2.11.001 (query tool), BC-2.11.005 (materialization), BC-2.11.012 (virtual fields), BC-2.15.001 (RocksDB init), BC-2.15.002 (domain KV operations) |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-74-fix | 2026-04-20 | product-owner | Resolved (placeholder) row in ## Verification Properties per pass-74 VP-TBD decision matrix extension. |
| 1.2 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; renamed Error Cases → Error Conditions; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft |
