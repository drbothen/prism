---
document_type: behavioral-contract
level: L3
version: "1.7"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-15"
capability: "CAP-026"
lifecycle_status: active
introduced: cycle-1
modified: "2026-07-13"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "76729b7"
traces_to:
  - "CAP-026"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.15.009: Context Decorator Injection — Auto-Inject Metadata into All Results

## Description

Every query result carries two distinct categories of injected metadata. Virtual fields
(`_sensor`, `_client`, `_source_table`, and `_source_type` for sensor tables) are injected as Arrow columns before DataFusion
execution so they are queryable in WHERE/GROUP BY/ORDER BY clauses. Decorator fields
(`client_name`, `sensor_instance`, `analyst_id`, `query_source`, `prism_version`) are
injected into the `_meta` response envelope after DataFusion execution and are not
queryable by PrismQL.

Both categories are deterministic for the same query context and are always present
in every response. Missing context values (e.g., no analyst in automated mode) produce
null fields rather than errors.

## Preconditions
- A query execution has produced OCSF-normalized results (either ad-hoc or scheduled)
- The Prism configuration and session context are available

## Postconditions
- Result records carry two separate categories of metadata:

  **Virtual fields** (injected pre-DataFusion into Arrow RecordBatch — queryable in WHERE/GROUP BY/ORDER BY, underscore-prefixed):
  - `_sensor`: sensor identifier (e.g., "crowdstrike", "armis", "prism" for internal tables)
  - `_client`: client ID (OrgSlug value; formerly TenantId, renamed per ADR-006)
  - `_source_table`: specific table name (e.g., "crowdstrike_detections", "prism_alerts")
  - `_source_type`: data delivery routing indicator — `"live"` (API fetch) or `"buffered"` (EventStream RocksDB buffer); sensor tables only — internal tables do NOT receive this field (see BC-2.11.012 §Invariants)

  **Decorator fields** (injected post-DataFusion into `_meta` response envelope — NOT queryable, no underscore prefix):
  - `client_name`: human-readable client name from TOML config
  - `sensor_instance`: the sensor instance identifier (e.g., "us-1")
  - `analyst_id`: the analyst identifier from the current session context
  - `query_source`: provenance of the query (e.g., "interactive", "schedule:check_alerts", "pack:incident-response.recent_detections")
  - `prism_version`: the running Prism version string

  Response structure:
  ```json
  {
    "rows": [
      { "_sensor": "crowdstrike", "_client": "acme", "_source_table": "crowdstrike_detections", "_source_type": "live", "severity_id": 4, "device_hostname": "DESKTOP-X" }
    ],
    "_meta": {
      "client_name": "Acme Corp",
      "analyst_id": "joshua",
      "query_source": "interactive",
      "prism_version": "0.1.0",
      "sensor_instance": "us-1"
    }
  }
  ```

- Queryable virtual fields (`_sensor`, `_client`, `_source_table`, `_source_type`) are documented in query-engine.md's virtual fields table and BC-2.11.012. Decorator fields (`client_name`, `sensor_instance`, `analyst_id`, `query_source`, `prism_version`) live in the `_meta` envelope and are defined in this BC only.
- Virtual fields are Arrow columns registered in the MemTable schema — they participate in DataFusion execution
- Decorator fields are `_meta` envelope metadata — they are NOT Arrow columns and cannot appear in PrismQL predicates.
- Both categories are deterministic: the same query context always produces the same values
- Both categories are included in audit log entries; only virtual fields are included in differential results

## Invariants
- Every response has all decorator fields present in `_meta` (never partial decoration)
- Decorator fields in `_meta` cannot be referenced in PrismQL predicates — they are injected after DataFusion execution. Virtual fields (`_sensor`, `_client`, `_source_table`, `_source_type`) CAN be referenced in predicates — they are Arrow columns injected before DataFusion execution (note: `_source_type` is sensor-table-only; querying it on an internal table results in `QueryExecutionFailed` per BC-2.11.012 §EC-11-035)
- Decorators never modify the OCSF record itself (they are envelope metadata in `_meta`)

## Error Conditions
| Error | Condition | Behavior |
|-------|-----------|----------|
| (none) | Decorator injection cannot fail | Missing context values (e.g., no analyst_id in automated mode) produce `null` values, not errors |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-15-033 | Cross-client query produces records from 3 clients | Each record has its own `_client` (virtual field in result row, queryable). The `_meta.client_name` shows the name of the queried client context. Per-record client identification uses the `_client` virtual field. |
| EC-15-034 | Scheduled query execution (no analyst session) | `analyst_id` is null; `query_source` is "schedule:{schedule_name}" |
| EC-15-035 | Query returns 0 results | No decoration needed; empty result set |
| EC-15-036 | Client name contains unicode characters | Preserved as-is in `_meta.client_name` |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — interactive query | analyst session, single client | All virtual fields in result rows; all decorator fields in _meta |
| Scheduled query | no analyst session | `analyst_id=null`; `query_source="schedule:check_alerts"` |
| WHERE on virtual field | `WHERE _sensor = 'crowdstrike'` | Filters correctly (virtual field is queryable Arrow column) |
| WHERE on decorator field | `WHERE client_name = 'Acme'` | Parse/plan error — decorator fields not in Arrow schema |
| Cross-client result | query spanning 3 clients | Each result row has its own `_client` value |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (none) | Virtual field queryability requires DataFusion execution context (integration test); decorator-always-present is a struct initialization property (unit test); no pure-function formal invariant. |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-026 |
| L2 Invariants | DI-008 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.7 | DEFECT-MCP-ROWSHAPE-NULLS-001-pass3-rows-terminology | 2026-07-13 | product-owner | **F-MCPNULL-P3-LOW-001 closure — rows terminology sweep (POL-25 full-file).** §Canonical Test Vectors two sites and §Edge Cases one site still used "events" and "event row" as the result noun, inconsistent with the `rows` key rationale from BC-2.11.001 v1.17 and the response example already corrected in v1.6. **Changes (3 sites):** (1) §Canonical Test Vectors happy-path row: "All virtual fields in events" → "All virtual fields in result rows". (2) §Canonical Test Vectors cross-client row: "Each event row has its own `_client` value" → "Each result row has its own `_client` value". (3) §Edge Cases EC-15-033: "virtual field in event row, queryable" → "virtual field in result row, queryable". **POL-25 full-file sweep — all sites checked:** §Description: "Every query result carries two distinct categories" — result noun is "result", not "event" — clean. §Preconditions: "OCSF-normalized results" — clean. §Postconditions: JSON response example uses `"rows": [...]` (already correct from v1.6); bullet text "Result records carry two separate categories" — clean. `_source_type: "live"` example — clean. All `_meta` decorator fields — clean. §Invariants: "Decorators never modify the OCSF record itself" — `record` not `event`, clean. §Error Conditions: clean. §Edge Cases EC-15-034/035/036: clean; EC-15-033 fixed (site 3 above). §Canonical Test Vectors: sites 1 and 2 fixed above; WHERE/schedule rows — clean. §Verification Properties: clean. §Traceability: clean. Virtual-field injection semantics (Arrow column injection pre-DataFusion), decorator injection semantics (post-DataFusion `_meta` envelope), and all invariants UNCHANGED. |
| 1.6 | DEFECT-MCP-ROWSHAPE-NULLS-001-events-to-rows | 2026-07-13 | product-owner | **POL-23 sibling sweep — `events` → `rows` in JSON response example.** §Postconditions response example: `"events": [...]` → `"rows": [...]`. The `query` tool response array key was renamed from `events` to `rows` in S-5.01-FOLLOWUP-MCP-BOOT (PR #163); this BC's example retained the old key. Brought into alignment with BC-2.11.001 v1.17 and shipped behavior per human adjudication 2026-07-13 (DEFECT-MCP-ROWSHAPE-NULLS-001 F-MCPNULL-P1-MED-001). Virtual-field injection semantics and all other postconditions UNCHANGED. |
| 1.5 | DEFECT-CSDEVICES-EMPTY-PIPELINE-001 / F-CSD-P23-001 POL-29 sweep | 2026-07-10 | product-owner | POL-29 exhaustive sweep: virtual-field enumeration updated throughout to include `_source_type` (4th sensor-table virtual field per BC-2.11.012 v1.11). Changes: (1) §Description: `(_sensor, _client, _source_table)` → `(_sensor, _client, _source_table, and _source_type for sensor tables)`. (2) §Postconditions Virtual fields list: added `_source_type` bullet with sensor-table-only qualifier and EC-11-035 cross-reference. (3) JSON response example: added `"_source_type": "live"` field. (4) §Postconditions invariant note: `(_sensor, _client, _source_table)` → `(_sensor, _client, _source_table, _source_type)` + BC-2.11.012 cross-ref added. (5) §Invariants: `(_sensor, _client, _source_table) CAN be referenced` → `(_sensor, _client, _source_table, _source_type) CAN be referenced` + EC-11-035 internal-table note. |
| 1.4 | pass-15-remediation | 2026-04-27 | product-owner | `_client` field description updated TenantId → OrgSlug (ADR-006). |
| 1.3 | pass-74-fix | 2026-04-20 | product-owner | Resolved (placeholder) row in ## Verification Properties per pass-74 VP-TBD decision matrix extension. |
| 1.2 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; renamed Error Cases → Error Conditions; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft |
