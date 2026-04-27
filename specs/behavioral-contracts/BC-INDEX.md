---
document_type: behavioral-contract-index
level: L3
version: "4.18"
status: draft
producer: product-owner
timestamp: 2026-04-27T00:00:00
phase: 3.A
total_contracts: 230
active_contracts: 222
removed_contracts: 6
retired_contracts: 2
---

# Behavioral Contract Index

Flat index of all 230 behavioral contracts for Prism (230 total files, 222 active, 6 removed, 2 retired), organized by BC ID. Note: 5 prior index-only reserved entries (BC-2.07.007/008/009/010, BC-2.14.011) were dropped — they never had corresponding files.

**Note on `total_contracts`:** This count represents unique BC identifiers ever filed
(active + removed + retired = 208). Five prior index-only reserved entries
(BC-2.07.007/008/009/010, BC-2.14.011) were dropped in v4.8 because they never had
corresponding files — they are NOT counted in `total_contracts` and remain only in the
historical references section below.

Phase 3-patch additions (2026-04-16): 22 new BCs added in Burst 1 to close traceability gaps for AD-019 (WASM plugins), AD-020 (infusions), AD-021 (actions), CAP-022 (auto-case-creation), and BC-2.14.012 stub completion. Burst 2.5: 4 additional BCs closing remaining gaps flagged by story-writer: BC-2.08.008/009 (diagnostics tool + resources, S-5.08), BC-2.05.011 (audit forwarding at-least-once, S-5.10), BC-2.13.014 (IOC file loading, S-4.03).

| BC ID | Title | Subsystem | CAP | Priority | Status |
|-------|-------|-----------|-----|----------|--------|
| BC-2.01.001 | ~~Single-Client Sensor Query Returns Scoped Results~~ | 01 - Sensor Adapters | CAP-001 | P0 | removed |
| BC-2.01.002 | Cross-Client Fan-Out — Query Engine Orchestrates Parallel Sensor Fetches | 01 - Sensor Adapters | CAP-002 | P0 | draft |
| BC-2.01.003 | ~~Cursor-Based Forward-Only Pagination (MCP-Exposed)~~ | 01 - Sensor Adapters | CAP-001 | P0 | removed |
| BC-2.01.004 | Offset-Based Hybrid Pagination for Claroty Audit Logs | 01 - Sensor Adapters | CAP-001 | P0 | draft |
| BC-2.01.005 | CrowdStrike OAuth2 Authentication and Two-Step Fetch | 01 - Sensor Adapters | CAP-001 | P0 | draft |
| BC-2.01.006 | Cyberint Cookie-Based Authentication and Multi-Format Timestamp Parsing | 01 - Sensor Adapters | CAP-001 | P0 | draft |
| BC-2.01.007 | Claroty Bearer Token Auth with Polymorphic ID Handling | 01 - Sensor Adapters | CAP-001 | P0 | draft |
| BC-2.01.008 | Armis Bearer Token Auth with AQL Query Forwarding and Timestamp Fallback | 01 - Sensor Adapters | CAP-001 | P0 | draft |
| BC-2.01.009 | ~~Query Filtering and Sorting Parameters~~ | 01 - Sensor Adapters | CAP-001 | P0 | removed |
| BC-2.01.010 | Partial Failure Handling for Paginated and Cross-Client Queries | 01 - Sensor Adapters | CAP-001, CAP-002 | P0 | draft |
| BC-2.01.011 | ~~Cross-Sensor Correlation via OCSF Field Alignment~~ | 01 - Sensor Adapters | CAP-012 | P1 | removed |
| BC-2.01.012 | ~~Query Fingerprint Validation at Startup~~ | 01 - Sensor Adapters | CAP-001 | P0 | removed |
| BC-2.01.013 | DataSource Trait Eliminates Per-Sensor Code Duplication | 01 - Sensor Adapters | CAP-001 | P0 | draft |
| BC-2.01.014 | Exponential Backoff and Retry for Transient Sensor API Errors | 01 - Sensor Adapters | CAP-001 | P0 | draft |
| BC-2.01.015 | ~~MCP Tool Response Envelope Structure~~ | 01 - Sensor Adapters | CAP-001 | P0 | removed |
| BC-2.02.001 | OCSF Schema Loading at Build Time via ocsf-proto-gen | 02 - OCSF Normalization | CAP-003 | P0 | draft |
| BC-2.02.002 | DynamicMessage Creation from Sensor Records | 02 - OCSF Normalization | CAP-003 | P0 | draft |
| BC-2.02.003 | CrowdStrike Alert Field Mapping to OCSF | 02 - OCSF Normalization | CAP-003 | P0 | draft |
| BC-2.02.004 | Cyberint Alert Field Mapping to OCSF | 02 - OCSF Normalization | CAP-003 | P0 | draft |
| BC-2.02.005 | Claroty xDome Field Mapping to OCSF (9 Data Sources) | 02 - OCSF Normalization | CAP-003 | P0 | draft |
| BC-2.02.006 | Armis Centrix Field Mapping to OCSF (7 Data Sources) | 02 - OCSF Normalization | CAP-003 | P0 | draft |
| BC-2.02.007 | Vendor Extension Preservation in raw_extensions | 02 - OCSF Normalization | CAP-003 | P0 | draft |
| BC-2.02.008 | Four-Tier Field Alias Resolution | 02 - OCSF Normalization | CAP-003 | P0 | draft |
| BC-2.02.009 | OCSF Version Pinning Per Release | 02 - OCSF Normalization | CAP-003 | P0 | draft |
| BC-2.02.010 | OCSF Enum Value Map for Runtime Display Names | 02 - OCSF Normalization | CAP-003 | P0 | draft |
| BC-2.02.011 | Graceful Normalization Error Handling (No Silent Data Loss) | 02 - OCSF Normalization | CAP-003 | P0 | draft |
| BC-2.02.012 | OCSF Event Class Selection Per Sensor Record Type | 02 - OCSF Normalization | CAP-003 | P0 | draft |
| BC-2.03.001 | CredentialStore Trait with Tenant-Scoped Operations | 03 - Credential Management | CAP-004 | P0 | draft |
| BC-2.03.002 | OS Keyring Backend via keyring-rs | 03 - Credential Management | CAP-004 | P0 | draft |
| BC-2.03.003 | AES-256-GCM Encrypted File Backend Fallback | 03 - Credential Management | CAP-004 | P0 | draft |
| BC-2.03.004 | Credential Namespace Isolation by (client_id, sensor_id, credential_name) | 03 - Credential Management | CAP-004 | P0 | draft |
| BC-2.03.005 | Credential CRUD Operations via MCP Tools (Mutations Require Confirmation Token) | 03 - Credential Management | CAP-004 | P0 | draft |
| BC-2.03.006 | Credential Resolution at Sensor Query Time | 03 - Credential Management | CAP-004 | P0 | draft |
| BC-2.03.007 | Secret Redaction in Logs, Errors, and MCP Responses | 03 - Credential Management | CAP-004 | P0 | draft |
| BC-2.03.008 | Credential Name Sanitization Against Path Traversal | 03 - Credential Management | CAP-004 | P0 | draft |
| BC-2.03.009 | resolve_secret() for _FILE Env Var and K8s Secret Mount Compatibility | 03 - Credential Management | CAP-004 | P0 | draft |
| BC-2.03.010 | Credential Access Audit Logging | 03 - Credential Management | CAP-004 | P0 | draft |
| BC-2.03.011 | Keyring Startup Probe for Permission Pre-Authorization | 03 - Credential Management | CAP-004 | P0 | draft |
| BC-2.03.012 | Credential Backend Selection and Fallback | 03 - Credential Management | CAP-004 | P0 | draft |
| BC-2.04.001 | Compile-Time Cargo Features Gate Write Code Families | 04 - Feature Flags | CAP-005 | P0 | draft |
| BC-2.04.002 | Runtime Per-Client TOML Feature Flag Configuration | 04 - Feature Flags | CAP-005 | P0 | draft |
| BC-2.04.003 | Hierarchical Capability Resolution (BTreeMap, Most-Specific-Path Wins, Deny Support) | 04 - Feature Flags | CAP-005 | P0 | draft |
| BC-2.04.004 | Two-Tier Gate -- Both Compile-Time and Runtime Must Permit Operation | 04 - Feature Flags | CAP-005 | P0 | draft |
| BC-2.04.005 | Hidden Tools Pattern -- Stateless Tool List Based on Configured Capabilities | 04 - Feature Flags | CAP-005 | P0 | draft |
| BC-2.04.006 | list_capabilities Meta-Tool for Capability Discovery | 04 - Feature Flags | CAP-005 | P0 | draft |
| BC-2.04.007 | Three-Tier Risk Classification for Operations | 04 - Feature Flags | CAP-006 | P1 | draft |
| BC-2.04.008 | Dry-Run Default for Reversible Write Operations | 04 - Feature Flags | CAP-006 | P1 | draft |
| BC-2.04.009 | Confirmation Token Generation for Irreversible Write Operations (100-Token Active Cap) | 04 - Feature Flags | CAP-006 | P1 | draft |
| BC-2.04.010 | Confirmation Token Consumption via confirm_action | 04 - Feature Flags | CAP-006 | P1 | draft |
| BC-2.04.011 | Token Expiry at 300 Seconds with Structured Error Recovery | 04 - Feature Flags | CAP-006 | P1 | draft |
| BC-2.04.012 | Token Content Hash Verification Prevents Action Tampering | 04 - Feature Flags | CAP-006 | P1 | draft |
| BC-2.04.013 | Feature Flag Evaluation Audit Logging for Write Operations | 04 - Feature Flags | CAP-005 | P0 | draft |
| BC-2.04.014 | notifications/tools/list_changed on Config Reload or Server Startup | 04 - Feature Flags | CAP-005 | P0 | draft |
| BC-2.04.015 | Structured Error When Write Capability Is Denied | 04 - Feature Flags | CAP-005 | P0 | draft |
| BC-2.05.001 | Every MCP Tool Invocation Produces Exactly One Audit Entry (Fail-Closed for Writes) | 05 - Audit Trail | CAP-007 | P0 | draft |
| BC-2.05.002 | Audit Entries Use Structured JSON Format with Complete Fields | 05 - Audit Trail | CAP-007 | P0 | draft |
| BC-2.05.003 | Credential Values Are Never Present in Audit Entries | 05 - Audit Trail | CAP-007 | P0 | draft |
| BC-2.05.004 | Write Operations Log Capability Check and Execution Outcome | 05 - Audit Trail | CAP-007 | P0 | draft |
| BC-2.05.005 | Credential Access Events Are Audit-Logged with Context | 05 - Audit Trail | CAP-007 | P0 | draft |
| BC-2.05.006 | Audit Entries Are Append-Only and Immutable | 05 - Audit Trail | CAP-007 | P0 | draft |
| BC-2.05.007 | Audit Entries Are Compatible with the Vector Pipeline | 05 - Audit Trail | CAP-007 | P0 | draft |
| BC-2.05.008 | Audit Entries Satisfy SOC 2 Type II and ISO 27001 Requirements | 05 - Audit Trail | CAP-007 | P0 | draft |
| BC-2.05.009 | Feature Flag Evaluations for Write Operations Are Audit-Logged | 05 - Audit Trail | CAP-007 | P0 | draft |
| BC-2.05.010 | Confirmation Token Lifecycle Events Are Audit-Logged | 05 - Audit Trail | CAP-007 | P0 | draft |
| BC-2.05.011 | Audit Forwarding — At-Least-Once Delivery to External Destinations (VP-039 monotonic watermark) | 05 - Audit Trail | CAP-007 | P0 | draft |
| BC-2.06.001 | TOML Configuration Loads and Deserializes at Startup | 06 - Client Configuration | CAP-009 | P0 | draft |
| BC-2.06.002 | Per-Client Sensor Mapping from TOML Configuration | 06 - Client Configuration | CAP-009 | P0 | draft |
| BC-2.06.003 | Credential References in Config Resolve to Credential Store Entries | 06 - Client Configuration | CAP-009 | P0 | draft |
| BC-2.06.004 | Capability Overrides Merge with Defaults Using More-Specific-Wins | 06 - Client Configuration | CAP-009 | P0 | draft |
| BC-2.06.005 | Configuration Validation Reports All Errors in One Pass | 06 - Client Configuration | CAP-009 | P0 | draft |
| BC-2.06.006 | --dry-run Flag Validates Config and Prints Redacted Summary | 06 - Client Configuration | CAP-009 | P0 | draft |
| BC-2.06.007 | Missing Required Fields Produce Actionable Error Messages | 06 - Client Configuration | CAP-009 | P0 | draft |
| BC-2.06.008 | Default Values Apply and Environment Variables Override TOML | 06 - Client Configuration | CAP-009 | P0 | draft |
| BC-2.06.009 | Config Reload Triggers notifications/tools/list_changed | 06 - Client Configuration | CAP-009 | P0 | draft |
| BC-2.06.010 | Client ID Validation Enforces Allowed Character Set | 06 - Client Configuration | CAP-009 | P0 | draft |
| BC-2.07.001 | Internal Ephemeral Pagination Token Structure | 07 - Adapter Pagination & Response Cache | CAP-011 | P0 | draft |
| BC-2.07.002 | Internal Pagination Token Lifecycle — Forward Progress, Timeout, and Cleanup | 07 - Adapter Pagination & Response Cache | CAP-011 | P0 | draft |
| BC-2.07.003 | Query Engine Sensor-Fetch Cache with Configurable TTL | 07 - Adapter Pagination & Response Cache | CAP-014 | P1 | draft |
| BC-2.07.004 | Cache Invalidation on Write Operations | 07 - Adapter Pagination & Response Cache | CAP-014 | P1 | draft |
| BC-2.07.005 | Cache Key Derivation from Push-Down Parameters | 07 - Adapter Pagination & Response Cache | CAP-014 | P1 | draft |
| BC-2.07.006 | Cache Memory Bounds and Eviction Policy | 07 - Adapter Pagination & Response Cache | CAP-014 | P1 | draft |
| BC-2.08.001 | On-Demand Connectivity Check Per Sensor Per Client | 08 - Sensor Health | CAP-008 | P1 | draft |
| BC-2.08.002 | Auth Validity Check Per Sensor Per Client | 08 - Sensor Health | CAP-008 | P1 | draft |
| BC-2.08.003 | Rate Limit State Detection Per Sensor | 08 - Sensor Health | CAP-008 | P1 | draft |
| BC-2.08.004 | Last Successful Query Timestamp Per Sensor Per Client | 08 - Sensor Health | CAP-008 | P1 | draft |
| BC-2.08.005 | Health Check MCP Tool | 08 - Sensor Health | CAP-008 | P1 | draft |
| BC-2.08.006 | Health Status MCP Resource | 08 - Sensor Health | CAP-008 | P1 | draft |
| BC-2.08.007 | Partial Health Status (Mixed Sensor Availability) | 08 - Sensor Health | CAP-008 | P1 | draft |
| BC-2.08.008 | `get_diagnostics` MCP Tool — Subsystem Diagnostic Query with Injection Defense | 08 - Sensor Health | CAP-008 | P1 | draft |
| BC-2.08.009 | Diagnostic Resource Templates — `prism://diagnostics/*` MCP Resources | 08 - Sensor Health | CAP-008 | P1 | draft |
| BC-2.09.001 | Structural Separation of Untrusted Data | 09 - Prompt Injection Defense | CAP-010 | P0 | draft |
| BC-2.09.002 | Provenance Framing in Tool Descriptions | 09 - Prompt Injection Defense | CAP-010 | P0 | draft |
| BC-2.09.003 | Suspicious Pattern Detection via Regex with NFKC Normalization | 09 - Prompt Injection Defense | CAP-010 | P0 | draft |
| BC-2.09.004 | Safety Flags via _meta.safety_flags Array (Centralized, Not Per-Field) | 09 - Prompt Injection Defense | CAP-010 | P0 | draft |
| BC-2.09.005 | Trust-Level Metadata Per Response | 09 - Prompt Injection Defense | CAP-010 | P0 | draft |
| BC-2.09.006 | Tool Description Security Warnings | 09 - Prompt Injection Defense | CAP-010 | P0 | draft |
| BC-2.09.007 | OutputSchema for Type-Safe LLM Reasoning | 09 - Prompt Injection Defense | CAP-010 | P0 | draft |
| BC-2.09.008 | Response Envelope with Trust Annotations | 09 - Prompt Injection Defense | CAP-010 | P0 | draft |
| BC-2.10.001 | rmcp ServerHandler Implementation | 10 - MCP Interface | CAP-034 | P0 | draft |
| BC-2.10.002 | Tool Registration via #[tool_router] | 10 - MCP Interface | CAP-005, CAP-015 | P0 | draft |
| BC-2.10.003 | Conditional Tool Registration (Feature-Flag Gated) | 10 - MCP Interface | CAP-005 | P0 | draft |
| BC-2.10.004 | Client Scoping on Every Tool (Stateless Model) | 10 - MCP Interface | CAP-009 | P0 | draft |
| BC-2.10.005 | notifications/tools/list_changed on Config Reload | 10 - MCP Interface | CAP-005, CAP-009 | P0 | draft |
| BC-2.10.006 | Stdio Transport | 10 - MCP Interface | CAP-034 | P0 | draft |
| BC-2.10.007 | Structured Error Responses | 10 - MCP Interface | CAP-034 | P0 | draft |
| BC-2.10.008 | MCP Resources for Client List and Sensor Inventory | 10 - MCP Interface | CAP-008, CAP-009 | P0 | draft |
| BC-2.10.009 | MCP Prompts for Common Workflows | 10 - MCP Interface | CAP-034 | P1 | draft |
| BC-2.10.010 | Graceful Shutdown on SIGTERM/SIGINT | 10 - MCP Interface | CAP-034 | P0 | draft |
| BC-2.10.011 | list_capabilities Meta-Tool | 10 - MCP Interface | CAP-005 | P0 | draft |
| BC-2.11.001 | `query` MCP Tool Accepts Scoping + PrismQL Query String | 11 - Query Execution | CAP-015 | P0 | draft |
| BC-2.11.002 | PrismQL Filter Mode Parsing | 11 - Query Execution | CAP-015 | P0 | draft |
| BC-2.11.003 | PrismQL SQL Mode Parsing | 11 - Query Execution | CAP-015 | P0 | draft |
| BC-2.11.004 | PrismQL Pipe Mode Parsing | 11 - Query Execution | CAP-015 | P0 | draft |
| BC-2.11.005 | Ephemeral Materialization — Fan-Out, Normalize, Arrow RecordBatch, DataFusion MemTable | 11 - Query Execution | CAP-015 | P0 | draft |
| BC-2.11.006 | Query Security Limits Enforcement | 11 - Query Execution | CAP-015 | P0 | draft |
| BC-2.11.007 | Sensor Filter Push-Down | 11 - Query Execution | CAP-015 | P0 | draft |
| BC-2.11.008 | `create_alias` MCP Tool | 11 - Query Execution | CAP-016 | P1 | draft |
| BC-2.11.009 | Alias Resolution — Pre-Parse Expansion, Composition, Cycle Detection | 11 - Query Execution | CAP-016 | P1 | draft |
| BC-2.11.010 | `explain_query` MCP Tool | 11 - Query Execution | CAP-015 | P0 | draft |
| BC-2.11.011 | Cross-Client Query Scoping | 11 - Query Execution | CAP-015 | P0 | draft |
| BC-2.11.012 | Virtual Fields in Queries — `_sensor`, `_client`, `_source_table` | 11 - Query Execution | CAP-015 | P0 | draft |
| BC-2.11.013 | `list_aliases` MCP Tool | 11 - Query Execution | CAP-016 | P1 | draft |
| BC-2.11.014 | `delete_alias` MCP Tool | 11 - Query Execution | CAP-016 | P1 | draft |
| BC-2.11.015 | `explain_alias` MCP Tool | 11 - Query Execution | CAP-016 | P1 | draft |
| BC-2.12.001 | `create_schedule` MCP Tool — Create a Scheduled Query | 12 - Scheduler | CAP-017 | P0 | draft |
| BC-2.12.002 | `list_schedules` MCP Tool — List Active Schedules with Next Run Times | 12 - Scheduler | CAP-017 | P0 | draft |
| BC-2.12.003 | `delete_schedule` MCP Tool — Remove a Schedule (Confirmation Required) | 12 - Scheduler | CAP-017 | P0 | draft |
| BC-2.12.004 | Schedule Execution Loop — Tick-Based with Splay and In-Flight Skip | 12 - Scheduler | CAP-017 | P0 | draft |
| BC-2.12.005 | Differential Result Computation — Hash Previous Results, Return Added/Removed | 12 - Scheduler | CAP-018 | P0 | draft |
| BC-2.12.006 | Epoch/Counter Tracking — Exactly-Once Semantics, Persist to Storage After Each Run | 12 - Scheduler | CAP-018 | P0 | draft |
| BC-2.12.007 | `get_diff_results` MCP Tool — Retrieve Differential Results for a Scheduled Query | 12 - Scheduler | CAP-018 | P0 | draft |
| BC-2.12.008 | Pack Loading and Discovery — Load Packs from Config, Run Discovery Queries, Conditional Execution | 12 - Scheduler | CAP-023 | P0 | draft |
| BC-2.12.009 | Pack CRUD MCP Tools — `create_pack`, `list_packs`, `delete_pack` | 12 - Scheduler | CAP-023 | P0 | draft |
| BC-2.12.010 | Schedule State Persistence — RocksDB Domain for Scheduling Metadata | 12 - Scheduler | CAP-017 | P0 | draft |
| BC-2.12.011 | Action At-Least-Once Delivery with Retry | 12 - Scheduler | CAP-021 | P0 | retired |
| BC-2.12.012 | Action Template Injection Scanning | 12 - Scheduler | CAP-021 | P0 | retired |
| BC-2.13.001 | Detection Rule Loading — Parse PrismQL Predicate, Validate at Load Time, Reject Invalid Rules | 13 - Detection Engine | CAP-020 | P0 | draft |
| BC-2.13.002 | Single-Event Detection — Evaluate Rule Predicate Against Each Differential Record | 13 - Detection Engine | CAP-020 | P0 | draft |
| BC-2.13.003 | Correlation Detection — Threshold Over Sliding Window with Group-By, Reset-After-Fire | 13 - Detection Engine | CAP-020 | P0 | draft |
| BC-2.13.004 | Sequence Detection — Ordered Multi-Event Pattern Matching Within Time Window | 13 - Detection Engine | CAP-020 | P0 | draft |
| BC-2.13.005 | Alert Generation — Interpolate Template, Persist Alert, Broadcast via MCP Notification | 13 - Detection Engine | CAP-020 | P0 | draft |
| BC-2.13.006 | `create_rule` MCP Tool — Create Detection Rule with Scope | 13 - Detection Engine | CAP-020 | P0 | draft |
| BC-2.13.007 | `list_rules` MCP Tool — List Active Rules by Scope | 13 - Detection Engine | CAP-020 | P0 | draft |
| BC-2.13.008 | `delete_rule` MCP Tool — Remove Rule (Confirmation for Global Rules) | 13 - Detection Engine | CAP-020 | P0 | draft |
| BC-2.13.009 | Rule-to-SQL Compilation — Translate Detection Predicates to DataFusion WHERE Clauses | 13 - Detection Engine | CAP-027 | P0 | draft |
| BC-2.13.010 | Security UDF Registration — Register Domain-Specific Functions with DataFusion | 13 - Detection Engine | CAP-027 | P0 | draft |
| BC-2.13.011 | Three-Scope Rule Resolution — Global Baseline + Per-Client Overrides + Analyst Ad-Hoc | 13 - Detection Engine | CAP-020 | P0 | draft |
| BC-2.13.012 | Detection State Persistence — RocksDB Domain for Correlation Windows, Sequence State, Alert History | 13 - Detection Engine | CAP-020 | P0 | draft |
| BC-2.13.013 | Alert Deduplication — Per-Match-Mode Dedup Keys Prevent Duplicate Alerts | 13 - Detection Engine | CAP-021 | P0 | draft |
| BC-2.13.014 | IOC File Loading and Pattern Store — At-Startup Load with Hot Reload and Bounded Memory | 13 - Detection Engine | CAP-020 | P0 | draft |
| BC-2.14.001 | `create_case` MCP Tool — Create Case from One or More Alerts | 14 - Alert & Case Management | CAP-022 | P0 | draft |
| BC-2.14.002 | Case State Transitions — 5-State Machine with 12 Valid Transitions | 14 - Alert & Case Management | CAP-022 | P0 | draft |
| BC-2.14.003 | `update_case` MCP Tool — Transition State, Set Disposition, Add Annotation | 14 - Alert & Case Management | CAP-022 | P0 | draft |
| BC-2.14.004 | `list_cases` MCP Tool — Filter by Status, Client, Severity, Assignee | 14 - Alert & Case Management | CAP-022 | P0 | draft |
| BC-2.14.005 | `get_case` MCP Tool — Full Case Detail with Timeline and Linked Alerts | 14 - Alert & Case Management | CAP-022 | P0 | draft |
| BC-2.14.006 | Disposition Assignment — Required on Resolved Transition | 14 - Alert & Case Management | CAP-022 | P0 | draft |
| BC-2.14.007 | Timeline Annotations — 5 Types: Note, StatusChange, AlertLink, EvidenceLink, OtImpact | 14 - Alert & Case Management | CAP-022 | P0 | draft |
| BC-2.14.008 | TTD/TTI/TTR Per-Case and Aggregate MTTD/MTTI/MTTR Computation — From Event Timestamps to Case State Transitions | 14 - Alert & Case Management | CAP-022 | P0 | draft |
| BC-2.14.009 | Case Persistence — RocksDB Domain for Case State, Timeline, Disposition, Metrics | 14 - Alert & Case Management | CAP-022 | P0 | draft |
| BC-2.14.010 | `case_metrics` MCP Tool — Aggregate MTTD/MTTR and Case Status Counts | 14 - Alert & Case Management | CAP-022 | P0 | draft |
| BC-2.14.012 | `acknowledge_alert` MCP Tool — Mark Alert as Acknowledged (Idempotent) | 14 - Alert & Case Management | CAP-022 | P0 | draft |
| BC-2.14.013 | Auto-Case-Creation from High-Severity Detection Rules | 14 - Alert & Case Management | CAP-022 | P1 | draft |
| BC-2.15.001 | RocksDB Initialization — Create/Open Database, Initialize Column Families for All Domains | 15 - Storage Layer | CAP-019 | P0 | draft |
| BC-2.15.002 | Domain-Based Key-Value Operations — get/put/putBatch/remove/removeRange/scan per Domain | 15 - Storage Layer | CAP-019 | P0 | draft |
| BC-2.15.003 | Buffered Audit Log Persistence — Write to RocksDB Before stderr/Vector, Exponential Backoff on Forward Failure | 15 - Storage Layer | CAP-025 | P0 | draft |
| BC-2.15.004 | Audit Buffer Overflow — Purge Oldest Entries When Exceeding 100K, Log Warning | 15 - Storage Layer | CAP-025 | P0 | draft |
| BC-2.15.005 | Crash Recovery Dirty Bits — Set Before Operation, Clear After, Detect on Restart | 15 - Storage Layer | CAP-024 | P0 | draft |
| BC-2.15.006 | Resource Watchdog Initialization — Set Memory/CPU/Timeout Limits Based on Graduated Level | 15 - Storage Layer | CAP-024 | P0 | draft |
| BC-2.15.007 | Watchdog Query Termination — Kill Query Exceeding Limits, Return Structured Error | 15 - Storage Layer | CAP-024 | P0 | draft |
| BC-2.15.008 | Query Denylisting — After N Consecutive Failures, Denylist with Manual Override | 15 - Storage Layer | CAP-024 | P0 | draft |
| BC-2.15.009 | Context Decorator Injection — Auto-Inject Metadata into All Results | 15 - Storage Layer | CAP-026 | P0 | draft |
| BC-2.15.010 | Decorator Three-Phase Model — Config-Time, Query-Time, Periodic | 15 - Storage Layer | CAP-026 | P0 | draft |
| BC-2.15.011 | Internal Table Registration — RocksDB Domains as DataFusion Tables | 15 - Storage Layer | CAP-028 | P0 | draft |
| BC-2.16.001 | Sensor Spec File Loading — Parse TOML, Validate Schema, Register Tables | 16 - Spec Engine | CAP-029 | P0 | draft |
| BC-2.16.002 | Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation | 16 - Spec Engine | CAP-029 | P0 | draft |
| BC-2.16.003 | Column-to-OCSF Mapping at Query Time — Map Sensor Columns to OCSF Fields Per Spec | 16 - Spec Engine | CAP-029 | P0 | draft |
| BC-2.16.004 | Rust Escape Hatch for Custom Adapters — Trait-Based Override When Config Is Insufficient | 16 - Spec Engine | CAP-029 | P0 | draft |
| BC-2.16.005 | `reload_config` MCP Tool — Re-Read All Config Files, Validate, Atomic Swap, Notify | 16 - Spec Engine | CAP-030 | P1 | draft |
| BC-2.16.006 | Arc-Swap Config Access on Hot Path — Lock-Free Reads for Query-Time Config Access | 16 - Spec Engine | CAP-030 | P1 | draft |
| BC-2.16.007 | Sensor Spec Hot Reload — Add/Remove/Update Sensor Tables Without Restart | 16 - Spec Engine | CAP-030 | P1 | draft |
| BC-2.16.008 | `add_sensor_spec` MCP Tool — Upload a New Sensor Spec at Runtime | 16 - Spec Engine | CAP-029, CAP-030 | P0 | draft |
| BC-2.16.009 | Spec File Validation — Schema Validation, Variable Reference Resolution, OCSF Field Validation | 16 - Spec Engine | CAP-029 | P0 | draft |
| BC-2.16.010 | `list_sensor_specs` MCP Tool — List Loaded Sensor Specs with Table Schemas and Status | 16 - Spec Engine | CAP-029 | P0 | draft |
| BC-2.17.001 | Plugin Panic Isolation — Crashed Plugin Does Not Terminate Host Process | 17 - WASM Plugin Runtime | CAP-032 | P0 | draft |
| BC-2.17.002 | Plugin Sandbox — No Direct Filesystem or Network Access | 17 - WASM Plugin Runtime | CAP-032 | P0 | draft |
| BC-2.17.003 | Plugin Sandbox — Memory Limit Enforced Per Plugin Instance (default 64MB) | 17 - WASM Plugin Runtime | CAP-032 | P0 | draft |
| BC-2.17.004 | Plugin Sandbox — CPU Time Limit Enforced via Epoch Interruption (default 5s) | 17 - WASM Plugin Runtime | CAP-032 | P0 | draft |
| BC-2.17.005 | Plugin Hot Reload — Atomic Module Swap, In-Flight Calls Complete Against Old Version | 17 - WASM Plugin Runtime | CAP-030, CAP-032 | P0 | draft |
| BC-2.17.006 | WIT Interface Validation Before Plugin Registration | 17 - WASM Plugin Runtime | CAP-032 | P0 | draft |
| BC-2.18.001 | Alert and Case Action Triggers — At-Least-Once Delivery with Exponential Backoff Retry | 18 - Action Delivery Engine | CAP-033 | P0 | draft |
| BC-2.18.002 | Schedule Action Triggers — Best-Effort, Retry on Next Cron Tick | 18 - Action Delivery Engine | CAP-033 | P0 | draft |
| BC-2.18.003 | Manual Action Triggers — Fire-and-Forget, Result Returned Immediately to AI Caller | 18 - Action Delivery Engine | CAP-033 | P0 | draft |
| BC-2.18.004 | Scheduled Report Queries — try_acquire() on 16-Permit Semaphore, Skip If Unavailable | 18 - Action Delivery Engine | CAP-033 | P0 | draft |
| BC-2.18.005 | Partial Report Failure — Failed Sections Include Error Note, Others Delivered | 18 - Action Delivery Engine | CAP-033 | P0 | draft |
| BC-2.18.006 | Action Template Variables from Sensor/Alert Data — Injection-Scanned Before Interpolation | 18 - Action Delivery Engine | CAP-033 | P0 | draft |
| BC-2.18.007 | Action Credentials Must Use AI-Opaque Reference Model — No Inline Values (E-ACTION-001) | 18 - Action Delivery Engine | CAP-033 | P0 | draft |
| BC-2.18.008 | All Action Executions Are Audit-Logged — Success, Failure, and Suppression | 18 - Action Delivery Engine | CAP-033 | P0 | draft |
| BC-2.18.009 | `${case.alert_ids_quoted}` Values Validated as UUID v7 Before Interpolation | 18 - Action Delivery Engine | CAP-033 | P0 | draft |
| BC-2.19.001 | Infusion Spec Loading — Each Field Registers Exactly One DataFusion Scalar UDF | 19 - Infusion Enrichment Framework | CAP-031 | P0 | draft |
| BC-2.19.002 | Per-Query Dedup Cache — Unique Input Values Only, Not Per-Row | 19 - Infusion Enrichment Framework | CAP-031 | P0 | draft |
| BC-2.19.003 | API-Backed Infusion UDFs Rejected in Detection Rule Filters — E-RULE-012 | 19 - Infusion Enrichment Framework | CAP-031 | P0 | draft |
| BC-2.19.004 | Infusion Hot Reload — Failed Validation Retains Previous Registration (CI-002) | 19 - Infusion Enrichment Framework | CAP-030, CAP-031 | P0 | draft |
| BC-2.19.005 | Infusion Credentials Are Never Logged or Included in Error Messages | 19 - Infusion Enrichment Framework | CAP-031 | P0 | draft |
| BC-2.20.001 | Log Forwarder Recursive Prevention — Plugin host.log() Writes to Local Sink Only | 20 - Observability / Log Forwarding | CAP-035 | P0 | draft |
| BC-2.20.002 | Log Forwarder Min-Level Filter — Per-Destination min_level Applied Before Enqueue | 20 - Observability / Log Forwarding | CAP-035 | P0 | draft |
| BC-2.20.003 | Log Forwarder Queue Cap — Drop-Oldest on Overflow with Metric Emission | 20 - Observability / Log Forwarding | CAP-035 | P0 | draft |
| BC-2.20.004 | Log Forwarder Credential Resolution — AD-017 Opaque Reference Model at Forward Time | 20 - Observability / Log Forwarding | CAP-035 | P0 | draft |
| BC-2.20.005 | Log Forwarder Destination Isolation — Single Failed Destination Must Not Block Others | 20 - Observability / Log Forwarding | CAP-035 | P0 | draft |

## Wave 3 — Phase 3.A Behavioral Contracts (2026-04-27)

22 new BCs registered in v4.15 (Wave 3 Phase 3.A spec burst). All BCs at v0.2 PROPOSED.

**Subsystem 3.1 — Multi-Tenant Identity (ADR-006)**

| BC ID | Title | Subsystem | CAP | Priority | Status |
|-------|-------|-----------|-----|----------|--------|
| BC-3.1.001 | OrgRegistry bijective slug/uuid resolution | SS-06 (Client Configuration) | CAP-038 | P0 | PROPOSED |
| BC-3.1.002 | Audit entry carries both org_id and org_slug at construction time | SS-05 (Audit Trail) | CAP-007 | P0 | PROPOSED |
| BC-3.1.003 | OrgRegistry maintains strict bijectivity at all times | SS-06 (Client Configuration) | CAP-038 | P0 | PROPOSED |
| BC-3.1.004 | OrgRegistry rejects duplicate slugs and UUIDs at registration | SS-06 (Client Configuration) | CAP-038 | P0 | PROPOSED |

**Subsystem 3.2 — Per-Org Data and Credential Isolation (ADR-006)**

| BC ID | Title | Subsystem | CAP | Priority | Status |
|-------|-------|-----------|-----|----------|--------|
| BC-3.2.001 | Per-org sensor data isolation via composite HashMap key | SS-01 (Sensor Adapters) | CAP-001 | P0 | PROPOSED |
| BC-3.2.002 | Per-org credential isolation via OrgId-keyed namespace | SS-03 (Credential Management) | CAP-004 | P0 | PROPOSED |
| BC-3.2.003 | Per-org session token isolation via (OrgId, token) composite key | SS-03 (Credential Management) | CAP-004 | P0 | PROPOSED |
| BC-3.2.004 | Shared-mode DTU tags OrgId in payload body not in routing headers | SS-01 (Sensor Adapters) | CAP-040 | P0 | PROPOSED |
| BC-3.2.005 | DTU mode is deployment-time config — no runtime API to change it | SS-06 (Client Configuration) | CAP-040 | P0 | PROPOSED |

**Subsystem 3.3 — Customer Config Validation (ADR-007, ADR-010)**

| BC ID | Title | Subsystem | CAP | Priority | Status |
|-------|-------|-----------|-----|----------|--------|
| BC-3.3.001 | Startup rejects Security Telemetry DTU type declared with shared mode | SS-06 (Client Configuration) | CAP-009 | P0 | PROPOSED |
| BC-3.3.002 | No Credential Values in Customer Config Files | SS-06 (Client Configuration) | CAP-009 | P0 | PROPOSED |
| BC-3.3.003 | Schema Version Enforcement Rejects Unknown or Missing schema_version | SS-06 (Client Configuration) | CAP-009 | P0 | PROPOSED |
| BC-3.3.004 | Customer Config Validation Rejects Invalid Schema at Startup | SS-06 (Client Configuration) | CAP-009 | P0 | PROPOSED |

**Subsystem 3.4 — Multi-Tenant Data Generator (ADR-009)**

| BC ID | Title | Subsystem | CAP | Priority | Status |
|-------|-------|-----------|-----|----------|--------|
| BC-3.4.001 | Generator Determinism — Identical Inputs Produce Byte-Identical FixtureSet | SS-06 (Client Configuration) | CAP-039 | P0 | PROPOSED |
| BC-3.4.002 | Generator Output Schema-Validates Against Canonical Vendor API Spec | SS-06 (Client Configuration) | CAP-039 | P0 | PROPOSED |
| BC-3.4.003 | Archetype Catalog Enumeration — 8 Archetypes with Defined Baselines | SS-06 (Client Configuration) | CAP-039 | P0 | PROPOSED |
| BC-3.4.004 | Org-Tagged Record IDs — Every Generated Record Carries an Org-Derived ID Prefix | SS-06 (Client Configuration) | CAP-039 | P0 | PROPOSED |

**Subsystem 3.5 — DTU Test Harness Isolation (ADR-011)**

| BC ID | Title | Subsystem | CAP | Priority | Status |
|-------|-------|-----------|-----|----------|--------|
| BC-3.5.001 | Harness Logical Isolation Invariants | SS-01 (Sensor Adapters) | CAP-036 | P0 | PROPOSED |
| BC-3.5.002 | Harness Network Isolation Invariants | SS-01 (Sensor Adapters) | CAP-036 | P0 | PROPOSED |

**Subsystem 3.6 — Harness Fault Injection (ADR-008, ADR-011)**

| BC ID | Title | Subsystem | CAP | Priority | Status |
|-------|-------|-----------|-----|----------|--------|
| BC-3.6.001 | Per-Org Failure Injection | SS-01 (Sensor Adapters) | CAP-036 | P0 | PROPOSED |
| BC-3.6.002 | Harness Crash Detection | SS-01 (Sensor Adapters) | CAP-036 | P0 | PROPOSED |

**Subsystem 3.7 — Workspace Conventions (ADR-006)**

| BC ID | Title | Subsystem | CAP | Priority | Status |
|-------|-------|-----------|-----|----------|--------|
| BC-3.7.001 | Workspace src/ Convention Lint Enforcement | SS-01 (Sensor Adapters) | CAP-037 | P1 | PROPOSED |

## Summary

| Subsystem | BC Count | P0 | P1 | Removed | Retired |
|-----------|----------|----|----|---------|---------|
| 01 - Sensor Adapters | 9 | 9 | 0 | 6 | 0 |
| 02 - OCSF Normalization | 12 | 12 | 0 | 0 | 0 |
| 03 - Credential Management | 12 | 12 | 0 | 0 | 0 |
| 04 - Feature Flags | 15 | 9 | 6 | 0 | 0 |
| 05 - Audit Trail | 11 | 11 | 0 | 0 | 0 |
| 06 - Client Configuration | 10 | 10 | 0 | 0 | 0 |
| 07 - Adapter Pagination & Response Cache | 6 | 2 | 4 | 0 | 0 |
| 08 - Sensor Health | 9 | 0 | 9 | 0 | 0 |
| 09 - Prompt Injection Defense | 8 | 8 | 0 | 0 | 0 |
| 10 - MCP Interface | 11 | 10 | 1 | 0 | 0 |
| 11 - Query Execution | 15 | 10 | 5 | 0 | 0 |
| 12 - Scheduler | 10 | 10 | 0 | 0 | 2 |
| 13 - Detection Engine | 14 | 14 | 0 | 0 | 0 |
| 14 - Alert & Case Management | 12 | 11 | 1 | 0 | 0 |
| 15 - Storage Layer | 11 | 11 | 0 | 0 | 0 |
| 16 - Spec Engine | 10 | 7 | 3 | 0 | 0 |
| 17 - WASM Plugin Runtime | 6 | 6 | 0 | 0 | 0 |
| 18 - Action Delivery Engine | 9 | 9 | 0 | 0 | 0 |
| 19 - Infusion Enrichment Framework | 5 | 5 | 0 | 0 | 0 |
| 20 - Observability / Log Forwarding | 5 | 5 | 0 | 0 | 0 |
| **Total** | **200** | **171** | **29** | **6** | **2** |

### Phase 3-Patch Additions (2026-04-16)

**26 new BCs added (22 Burst 1 + 4 Burst 2.5):**

**Burst 1:**
- BC-2.14.012: Acknowledge Alert MCP Tool (stub completed — was placeholder since phase 1a)
- BC-2.14.013: Auto-Case-Creation from High-Severity Detection Rules (CAP-022 tracking note fulfilled)
- BC-2.17.001 through BC-2.17.006: WASM Plugin Runtime (AD-019) — 6 BCs from INV-PLUGIN-001 through INV-PLUGIN-006
- BC-2.18.001 through BC-2.18.009: Action Delivery Engine (AD-021) — 9 BCs from INV-ACTION-001 through INV-ACTION-009
- BC-2.19.001 through BC-2.19.005: Infusion Enrichment Framework (AD-020) — 5 BCs from INV-INFUSE-001 through INV-INFUSE-005

**Burst 2.5 (follow-up BCs from story-writer traceability gaps):**
- BC-2.08.008: `get_diagnostics` MCP Tool — Subsystem Diagnostic Query with Injection Defense (S-5.08)
- BC-2.08.009: Diagnostic Resource Templates — `prism://diagnostics/*` MCP Resources (S-5.08)
- BC-2.05.011: Audit Forwarding — At-Least-Once Delivery to External Destinations (S-5.10; proposes VP-039 Kani monotonic watermark)
- BC-2.13.014: IOC File Loading and Pattern Store — At-Startup Load with Hot Reload and Bounded Memory (S-4.03)

**New subsystems introduced (Burst 1):**
- Subsystem 17: WASM Plugin Runtime (AD-019, CAP-032, CAP-030)
- Subsystem 18: Action Delivery Engine (AD-021, CAP-033)
- Subsystem 19: Infusion Enrichment Framework (AD-020, CAP-031)

### Change Log (Adversarial Review Fixes)

**v4.18 (2026-04-27):** Pass 1 adversarial convergence fixes — C-002: BC-3.3.004 Precondition 4 corrected per D-051 (demo-server IS in DTU_DEFAULT_MODE with test_only=true; production validator uses absence-check); R-CUST-004 clarified (truly-unknown types only); R-CUST-013/E-CFG-013 added (test-only type in production config); TV-3.3.004-04 updated (demo-server → E-CFG-013); EC-3.3.004-08 parenthetical hedge removed. m-007: BC-3.3.001 and BC-3.3.004 story anchors updated from TBD to S-3.3.01 / S-3.3.02. M-005: BC-3.5.001 TV-1/TV-2/TV-3/TV-4 device ID prefix format corrected to D-059 canonical `dev-{org_slug}-{seed}-{index}`. M-009: v4.16 changelog note added for BC-3.2.004 CAP-009 → CAP-040 two-step transition. Arithmetic unchanged.

**v4.17 (2026-04-27):** C-5 capability re-anchoring — 10 Wave 3 BCs moved from CAP-009 to semantically correct capabilities: BC-3.1.001/003/004 → CAP-038 ("Multi-Tenant Identity Model"); BC-3.2.004/005 → CAP-040 ("Multi-Tenant Adapter Dispatch Mode"); BC-3.4.001/002/003/004 → CAP-039 ("Multi-Tenant Fixture Generation"). BC-3.3.001/002/003/004 remain CAP-009 (config validation is the correct anchor). CAP-038/039/040 added to capabilities.md v1.7. ADR-006/007/009 updated with `anchored_capabilities` frontmatter. Arithmetic unchanged (total_contracts=230, active=222).

**v4.16 (2026-04-27):** NEW-1 fix — corrected subsystem/capability columns for 10 Wave 3 BC rows whose index entries incorrectly listed SS-06/CAP-009 instead of the actual frontmatter values: BC-3.1.002 → SS-05/CAP-007; BC-3.2.001 → SS-01/CAP-001; BC-3.2.002 → SS-03/CAP-004; BC-3.2.003 → SS-03/CAP-004; BC-3.2.004 → SS-01/CAP-009 *(note: this v4.16 entry recorded CAP-009 as an intermediate value; v4.17 re-anchored BC-3.2.004 → CAP-040 as the semantically correct capability)*; BC-3.5.001 → SS-01/CAP-036; BC-3.5.002 → SS-01/CAP-036; BC-3.6.001 → SS-01/CAP-036; BC-3.6.002 → SS-01/CAP-036; BC-3.7.001 → SS-01/CAP-037. Minor fix: Wave 3 intro text corrected from "21 new BCs" to "22 new BCs". Arithmetic unchanged (total_contracts=230, active=222).

**v4.15 (2026-04-27):** Wave 3 Phase 3.A registration — 22 new BCs (BC-3.1.001–004, BC-3.2.001–005, BC-3.3.001–004, BC-3.4.001–004, BC-3.5.001–002, BC-3.6.001–002, BC-3.7.001) added to Wave 3 section. BLOCK-1 fix: old BC-3.3.001.md (ADR-010 variant) renamed to BC-3.3.004-customer-config-startup-validation.md; bc_id, H1, EC/TV/VP references updated to BC-3.3.004; traces_to corrected from `["CAP-009"]` array to ADR-010 file path string. DRIFT-1 fix: ADR-006 `related_bcs_planned` updated to include BC-3.2.003 and BC-3.2.004. DRIFT-3 fix: all 22 Wave 3 BC files bumped from v0.1 to v0.2. total_contracts: 208 → 230 (22 new BC-3.x IDs); active_contracts: 200 → 222.

**v4.9 (2026-04-19):** Burst 27 — Subsystem Summary table split Removed/Retired into two columns (eliminates SS-12 conflation); total_contracts clarifying note added; 7 L2-Invariants citations added by architect (DI-016/.025/.027/.028/.029/.030/.031); 4 SS-16 BC files (BC-2.16.001/.005/.007/.009) migrated from non-standard `## Traces` H2 format to canonical `## Traceability` table. arithmetic: removed_contracts 13 → 8 (v4.8 dropped 5 reserved-never-created) → 6 (v4.9 reclassified 2 as retired).

**v4.8 (2026-04-19):** Dropped 5 reserved-but-never-created entries from flat index table (BC-2.07.007/008/009/010, BC-2.14.011); moved to historical traceability section. Status-column hygiene for BC-2.12.011/.012 (removed→retired) per Pass-25 Burst 26 H-002. Frontmatter arithmetic: total=203, active=195, removed=6, retired=2.

**v4.7 (prior):** Phase 3-patch Burst 2.5 additions and un-retirement of BC-2.04.014, BC-2.06.009, BC-2.10.005.

**Removed BCs (16 historical decisions; 8 currently tombstoned as files — 6 removed + 2 retired):**

*Note:* Of the 16 entries below, 3 were un-retired (BC-2.04.014, BC-2.06.009, BC-2.10.005) and 5 were index-only reserved entries never backed by files (BC-2.07.007/008/009/010, BC-2.14.011) — these 5 have been dropped from the flat index table in v4.8 but are retained here for historical traceability. The remaining 8 BCs (6 removed + 2 retired) are the physical tombstone files present on disk.

- BC-2.01.001: Single-Client Sensor Query Returns Scoped Results -- replaced by `query(clients: ["acme"], ...)` (BC-2.11.001)
- BC-2.01.003: Cursor-Based Forward-Only Pagination (MCP-Exposed) -- query engine handles pagination internally; agent uses `limit`/`total_available`
- BC-2.01.009: Query Filtering and Sorting Parameters -- replaced by PrismQL query language (BC-2.11.002/003/004) and sensor filter push-down (BC-2.11.007)
- BC-2.01.011: Cross-Sensor Correlation via OCSF Field Alignment -- cross-sensor correlation IS the query engine (BC-2.11.005, BC-2.11.012)
- BC-2.01.012: Query Fingerprint Validation at Startup -- persistent cursor fingerprints eliminated with ephemeral pagination model
- BC-2.01.015: MCP Tool Response Envelope Structure -- replaced by query engine response format (BC-2.11.001)
- BC-2.04.014: UN-RETIRED (2026-04-17, Burst 21) -- new Config-Reload semantics; `notifications/tools/list_changed` fires on SIGHUP/config reload (not client context switch)
- BC-2.06.009: UN-RETIRED (2026-04-17, Burst 21) -- new Config-Reload semantics; Config Reload Triggers `notifications/tools/list_changed`
- BC-2.07.007: State Is Isolated Per-Client, Per-Sensor, Per-Source -- persistent state eliminated
- BC-2.07.008: MemoryStore Is Test-Only and Panics in Production -- FileStore/MemoryStore removed with ephemeral model
- BC-2.07.009: FileStore Is the Default and Only Production CursorStore -- FileStore removed with ephemeral model
- BC-2.07.010: State File Directory Follows {client}/{sensor}/{source}.json -- persistent state directories eliminated
- BC-2.10.005: UN-RETIRED (2026-04-17, Burst 21) -- new Config-Reload semantics; `notifications/tools/list_changed on Config Reload` (dual-anchor CAP-005, CAP-009)
- BC-2.14.011: Reserved -- ID slot reserved, never used
- BC-2.12.011: Action At-Least-Once Delivery with Retry -- RETIRED (2026-04-16, Burst 4b); superseded by BC-2.18.001 (Action Delivery Engine, INV-ACTION-001). BC-2.12.011 was a cross-subsystem summary written before subsystem 18 was established. BC-2.18.001 is the normative specification.
- BC-2.12.012: Action Template Injection Scanning -- RETIRED (2026-04-16, Burst 4b); superseded by BC-2.18.006 (Action Delivery Engine, INV-ACTION-006). BC-2.12.012 was a cross-subsystem summary written before subsystem 18 was established. BC-2.18.006 is the normative specification.

> **Note (P3P3-L-004, 2026-04-16):** Retired BCs' `capability` field is historical — do not include it in active capability coverage counts. BC-2.12.011 (`CAP-021`) and BC-2.12.012 (`CAP-021`) are retired; their capability attribution is preserved for traceability only. Active CAP-021 coverage is provided by the BC-2.18.xxx subsystem (Action Delivery Engine).

**Subsystem 01 Rename:** "Sensor Query Pipeline" renamed to "Sensor Adapters" (ARCH-INDEX canonical; formerly "Sensor Adapter Layer") -- per-sensor MCP read tools removed; subsystem now provides internal adapter behaviors (auth, pagination, retry) called by the query engine (subsystem 11).

### Version 4.3 (2026-04-16, Burst 5b — Adversary Pass 2 Fixes)

**Arithmetic corrections (P3P2-C-001):**
- `total_contracts`: 207 → 208 (SS-12 had 10 active BCs, not 8; enumerated row count now matches)
- `active_contracts`: 191 → 192 (SS-12 correction: +2; SS-14 correction: -1; net +1)
- SS-12 summary row: `8 | 8 | 0 | 2` → `10 | 10 | 0 | 2` (BC-2.12.001–010 are all 10 active; BC-2.12.011/012 are the 2 removed — removed count was already correct)
- SS-14 summary row: `13 | 12 | 1 | 1` → `12 | 11 | 1 | 1` (BC-2.14.011 removed means 12 active total: 11 P0 + 1 P1; previous row incorrectly counted 13 active)
- Total P0 count: 162 → 163 (reflects +2 from SS-12 and -1 from SS-14 P0)

**Attribution fix (P3P2-H-003):**
- BC-2.15.001 `event_buffer` column family attribution corrected: was `(BC-2.13.003)`, now `(S-2.08; osquery event publisher pattern)`

**CAP-022 body mention (P3P2-H-007):**
- BC-2.14.012 Description section updated to explicitly reference CAP-022

**Rewritten BCs (query engine refactor):**
- BC-2.01.002: Cross-client fan-out now orchestrated by query engine, not MCP tool handler
- BC-2.07.001: Pagination tokens now internal to query engine fetch layer (never exposed to MCP agent)
- BC-2.07.002: Pagination lifecycle reframed as internal resource management (fetch timeout, concurrent fetch limits)
- BC-2.07.003: Cache simplified -- only query engine sensor-fetch cache exists (no "direct tool cache")
- BC-2.07.005: Cache keys simplified -- only push-down parameter hashes (no "tool query hash")
- BC-2.10.002: Tool inventory updated to 15 tools (7 read + 8 write per-sensor)
- BC-2.10.004: Client scoping simplified -- read tools use `clients` array via `query`; write tools use scalar `client_id`

**Replaced BCs (subsystem 07 rewrite):**
- BC-2.07.001: Composite Cursor Structure -> Ephemeral Cursor-Based Pagination (No Persistent State)
- BC-2.07.002: Forward-Only Progress Invariant -> Pagination Token Expiry and Cleanup
- BC-2.07.003: Atomic File Writes -> Response Cache with Configurable TTL (CAP-014)
- BC-2.07.004: Persistence After Delivery -> Cache Invalidation on Write Operations (CAP-014)
- BC-2.07.005: Query Fingerprint Computation -> Cache Key Derivation from Query Parameters (CAP-014)
- BC-2.07.006: Fingerprint Mismatch Detection -> Cache Memory Bounds and Eviction Policy (CAP-014)

**Updated BC Titles:**
- BC-2.04.003: Added "BTreeMap, Most-Specific-Path Wins, Deny Support" to reflect hierarchical override model
- BC-2.04.009: Added "100-Token Active Cap" to reflect token cap constraint
- BC-2.05.001: Added "Fail-Closed for Writes" to reflect audit fail-closed policy
- BC-2.10.004: Added "Stateless Model" to reflect no session-level active client

**New Capability:**
- CAP-014: Response Caching (4 BCs: BC-2.07.003 through BC-2.07.006)

**New Capabilities (Query Engine & Aliases):**
- CAP-015: Ephemeral OCSF Query Engine (10 BCs: BC-2.11.001 through BC-2.11.007, BC-2.11.010, BC-2.11.011, BC-2.11.012)
- CAP-016: Query Aliases (5 BCs: BC-2.11.008, BC-2.11.009, BC-2.11.013, BC-2.11.014, BC-2.11.015)

### Version 4.4 (2026-04-16, Burst 11 PO — Adversary Pass 8/9 Fix P3P8-O-001)

**CAP taxonomy correction (P3P8-O-001 / P3P9 concur):**
- SS-19 BCs (BC-2.19.001-005) were anchored to CAP-020 "Detection Rules" — a semantic mismatch. SS-19 is the Infusion Enrichment Framework (AD-020), not detection rules.
- Created CAP-031 "Infusion Enrichment" in `domain-spec/capabilities.md` as a dedicated capability for the enrichment framework.
- Re-anchored BC-2.19.001, BC-2.19.002, BC-2.19.003, BC-2.19.005 from CAP-020 → CAP-031. (BC-2.19.004 was already correctly anchored to CAP-030 for hot reload.)
- BC-INDEX flat table: 4 CAP-020 rows in SS-19 changed to CAP-031.
- PRD §7 traceability matrix: 4 rows updated. PRD §7 Capability Coverage Summary: CAP-020 count 14 → 10; CAP-031 added with 4 BCs.
- PRD §2 SS-19 capability reference: CAP-020 → CAP-031.
- BC-INDEX "New subsystems introduced" note for Subsystem 19: CAP-020 → CAP-031.

**Bundled fix (P3P9-L-001):**
- "Removed BCs (14)" header corrected to "Removed BCs (16)" — frontmatter already showed 16; the section header was stale.

### Version 4.5 (2026-04-17, Burst 13 Part B — P3P12-A4-001 Fix)

**Root cause:** PRD §7 Capability Coverage Summary CAP titles had been hand-edited to match mis-anchored BCs (CAP-024 and CAP-025 swapped vs. canonical capabilities.md). BC file frontmatter `capability:` fields are the single source of truth. Part A (Burst 13 PO-A) fixed the BC frontmatters; Part B regenerates all indexes from those BC files.

**BC-INDEX CAP column regenerated from BC file source of truth (P3P12-A4-001):**

All CAP column values in the flat index table have been verified against each BC file's frontmatter `capability:` field. Changes applied:

- BC-2.10.001: `CAP-005` → `CAP-034` (MCP Server & Transport — rmcp ServerHandler is the transport layer)
- BC-2.10.006: `--` → `CAP-034` (Stdio Transport belongs to MCP Server & Transport)
- BC-2.10.007: `CAP-007` → `CAP-034` (Structured error responses are MCP transport behavior)
- BC-2.10.008: `CAP-009` → `CAP-008, CAP-009` (dual-anchor: health resources + client config)
- BC-2.10.009: `CAP-010` → `CAP-034` (MCP prompts belong to MCP Server & Transport)
- BC-2.10.010: `--` → `CAP-034` (Graceful shutdown belongs to MCP Server & Transport)
- BC-2.13.004: `CAP-021` → `CAP-020` (Sequence Detection is detection-rule logic, not alert generation)
- BC-2.15.001: `CAP-024` → `CAP-019` (RocksDB init belongs to Persistent Storage, not Resource Watchdog)
- BC-2.15.002: `CAP-024` → `CAP-019` (Domain KV ops belong to Persistent Storage)
- BC-2.15.003: `CAP-019` → `CAP-025` (Buffered audit log persistence belongs to Buffered Audit Logging)
- BC-2.15.004: `CAP-019` → `CAP-025` (Audit buffer overflow belongs to Buffered Audit Logging)
- BC-2.15.008: `CAP-025` → `CAP-024` (Query denylisting belongs to Resource Watchdog)
- BC-2.17.001–004, BC-2.17.006: `CAP-029` → `CAP-032` (WASM Plugin Runtime, not Config-Driven Sensor Adapters)
- BC-2.18.001–009: `CAP-021` → `CAP-033` (Action Delivery Engine, not Alert Generation)
- BC-2.19.004: `CAP-030` → `CAP-030, CAP-031` (dual-anchor: hot reload + infusion enrichment)

**"New subsystems introduced" note updated:**
- Subsystem 17: `CAP-029, CAP-030` → `CAP-032, CAP-030`
- Subsystem 18: `CAP-021` → `CAP-033`

### Version 4.7 (2026-04-17, Burst 21 Task A — Un-Retire 3 BCs with Config-Reload Semantics)

**Un-retired BCs (active_contracts 192 → 195; removed_contracts 16 → 13):**

- BC-2.04.014: Status `removed` → `draft`. New semantics: `notifications/tools/list_changed on Config Reload or Server Startup` (fires on SIGHUP/config reload, not client context switch). CAP-005.
- BC-2.06.009: Status `removed` → `draft`. New semantics: `Config Reload Triggers notifications/tools/list_changed`. CAP-009.
- BC-2.10.005: Status `removed` → `draft`. New semantics: `notifications/tools/list_changed on Config Reload`. Dual-anchor [CAP-005, CAP-009]. Active dual-anchor count 5 → 6.

**Summary table changes:**
- SS-04: 14 active / 1 removed → 15 active / 0 removed; P0 count 8 → 9
- SS-06: 9 active / 1 removed → 10 active / 0 removed; P0 count 9 → 10
- SS-10: 10 active / 1 removed → 11 active / 0 removed; P0 count 9 → 10
- Total: 192 → 195 active; 16 → 13 removed; P0 163 → 166

**BC-INDEX title column (bc_h1_is_title_source_of_truth):** All 3 un-retired rows updated to match current BC H1.

### Version 4.6 (2026-04-17, Burst 19 Part B — Systematic BC Title Reconciliation)

**Policy enforced:** `bc_h1_is_title_source_of_truth` — BC file H1 is the canonical title. BC-INDEX Title column and PRD §2 table title column must match the BC file H1 exactly.

**BC file H1 updates (enrichment moved into H1 from BC-INDEX, or H1 corrected):**
- BC-2.03.005: Added "(Mutations Require Confirmation Token)" to H1
- BC-2.04.009: Added "(100-Token Active Cap)" to H1; also clarified "for Irreversible Write Operations"
- BC-2.05.001: Added "(Fail-Closed for Writes)" to H1
- BC-2.05.011: Added "(VP-039 monotonic watermark)" to H1
- BC-2.14.012: Added "(Idempotent)" to H1
- BC-2.17.003: Added "(default 64MB)" to H1
- BC-2.17.004: Added "(default 5s)" to H1
- BC-2.18.001: Added "Exponential Backoff" to H1 delivery guarantee description
- BC-2.18.003: Added "to AI Caller" to H1
- BC-2.18.004: Added ", Skip If Unavailable" to H1
- BC-2.18.007: Added "(E-ACTION-001)" to H1
- BC-2.19.004: Added "(CI-002)" to H1

**BC-INDEX Title column corrections (synced to authoritative H1):**
- BC-2.02.008: "Three-Tier" → "Four-Tier" (BC body confirmed 4 tiers: Prism metadata, Proto fields, raw_extensions, None)
- BC-2.04.005: "Disabled Write Tools Omitted from tools/list" → "Stateless Tool List Based on Configured Capabilities"
- BC-2.04.009: "with 100-Token Active Cap" → "for Irreversible Write Operations (100-Token Active Cap)"
- BC-2.07.002: Added "Internal" prefix; "Expiry" → "Timeout"
- BC-2.09.003: Added "with NFKC Normalization"
- BC-2.09.004: "Safety Flag Parallel Fields (Flag, Don't Strip)" → "Safety Flags via _meta.safety_flags Array (Centralized, Not Per-Field)" (BC body unambiguous: centralized array, no per-field parallel fields; old BC-INDEX title was factually wrong)
- BC-2.12.007: Added full subtitle "for a Scheduled Query"
- BC-2.12.008: Added full subtitle "Load Packs from Config, Run Discovery Queries, Conditional Execution"
- BC-2.12.010: Added "for Scheduling Metadata"
- BC-2.13.001–005: Restored full subtitles truncated in BC-INDEX
- BC-2.13.008: "Confirmation for Global" → "Confirmation for Global Rules"
- BC-2.13.010: "subnet_contains, ioc_match, time_window" → "Register Domain-Specific Functions with DataFusion"
- BC-2.13.011: "Global + Client + Analyst Merge" → "Global Baseline + Per-Client Overrides + Analyst Ad-Hoc"
- BC-2.13.013: Added "Prevent Duplicate Alerts"
- BC-2.14.002: "5-State Machine, 12 Valid Transitions" → "5-State Machine with 12 Valid Transitions"
- BC-2.14.003–005: Restored full subtitles ("Transition State, Set Disposition, Add Annotation"; "Assignee" added; "Case" and "Linked" added)
- BC-2.14.007: Snake_case type names → CamelCase to match H1 (Note, StatusChange, AlertLink, EvidenceLink, OtImpact)
- BC-2.14.008: "MTTD/MTTR Auto-Computation" → "TTD/TTI/TTR Per-Case and Aggregate MTTD/MTTI/MTTR Computation — From Event Timestamps to Case State Transitions"
- BC-2.15.001: Added "Database" and "Initialize"
- BC-2.15.002: Added "removeRange" (was dropped from BC-INDEX); added "per Domain"
- BC-2.15.003: Restored full subtitle with write sequence and backoff detail
- BC-2.15.006–008: Restored full subtitles
- BC-2.19.001: Removed spurious "Entry" word
- BC-2.19.003: Changed parenthetical to em-dash format to match H1

### Version 4.14 (2026-04-22, BLOCK-WV1-04 — BC-2.02.003 severity format fix)

**BC-2.02.003 severity format corrected (BLOCK-WV1-04):** CrowdStrike severity field was incorrectly specified as integer (1-5); corrected to string (e.g., `"High"`) with OCSF v1.x name-to-id mapping per S-1.05 Task 2 and AC-1. `severity_name` preservation in `raw_extensions["crowdstrike_severity_name"]` documented. Postconditions expanded to full field list. Test vectors updated to use string severity. This unblocks S-1.05 implementer dispatch. Arithmetic unchanged (total_contracts=208, active=200, removed=6, retired=2).

### Version 4.13 (2026-04-21, pass-93-F93-002 — BC-2.17.005 dual-anchor CAP-030, CAP-032)

**BC-2.17.005 capability dual-anchor (F93-002):** Capability column updated CAP-030 → CAP-030, CAP-032. BC-2.17.005 (Plugin Hot Reload — Atomic Module Swap) is the SS-17 hot reload contract; sibling BCs BC-2.17.001/002/003/004/006 all anchor to CAP-032. Parallel precedent: BC-2.19.004 (Infusion Hot Reload) dual-anchors CAP-030, CAP-031 per pass-92. Arithmetic unchanged (total_contracts=208, active=200, removed=6, retired=2).

### Version 4.12 (2026-04-21, pass-80 follow-on — CAP-035 re-anchor for SS-20)

**CAP re-anchor for all 5 SS-20 BCs (BC-2.20.001–005):** Capability column updated CAP-025 → CAP-035 (Diagnostic Log Forwarding), following business-analyst creation of CAP-035 post-hoc per pass-80 F80-002 follow-on. Arithmetic unchanged (total_contracts=208, active=200, removed=6, retired=2).

### Version 4.11 (2026-04-21, pass-80 remediation — F80-002: SS-20 BC authoring)

**5 new BCs added for SS-20 (Observability / Log Forwarding):**

- BC-2.20.001: Log Forwarder Recursive Prevention — Plugin host.log() Writes to Local Sink Only
- BC-2.20.002: Log Forwarder Min-Level Filter — Per-Destination min_level Applied Before Enqueue
- BC-2.20.003: Log Forwarder Queue Cap — Drop-Oldest on Overflow with Metric Emission
- BC-2.20.004: Log Forwarder Credential Resolution — AD-017 Opaque Reference Model at Forward Time
- BC-2.20.005: Log Forwarder Destination Isolation — Single Failed Destination Must Not Block Others

**Capability anchor:** CAP-025 (Buffered Audit Logging) used as closest semantic match per F80-002
instructions. No existing CAP covers diagnostic log forwarding to external systems.
A dedicated CAP-035 (Diagnostic Log Forwarding) would be the semantically correct anchor
and is recommended for a future capabilities.md update. (Note: superseded by v4.12 — CAP-035 was created post-hoc and is now the canonical anchor.)

**Arithmetic:**
- total_contracts: 203 → 208
- active_contracts: 195 → 200
- SS-20 summary row: 0/0/0/0/0 → 5/5/0/0/0
- Total P0 count: 166 → 171

### Version 4.10 (2026-04-19, Burst 28 — DI-017 dual citation + SS-16 Priority coherence)

**BC body fixes:**
- BC-2.10.006: L2 Invariants now cites DI-017 (stdio transport as primary enforcer of single-process invariant; BC body postcondition line 35 states "one stdio session corresponds to one analyst"). BC-2.15.001 retains independent DI-017 citation (RocksDB LOCK is complementary storage-layer enforcement). Two enforcer BCs now citing DI-017.
- BC-2.16.001: Body Priority P1 → P0 (sync to BC-INDEX entry; v1.0-blocking — no sensor spec can load without this BC enforced).
- BC-2.16.009: Body Priority P1 → P0 (sync to BC-INDEX entry; v1.0-blocking — gates BC-2.16.001 rejection path).

**SS-16 priority pattern now coherent:**
- BC-2.16.001 (sensor spec file loading) P0: blocks v1.0 loading path
- BC-2.16.009 (spec file validation) P0: gates loading rejection path
- BC-2.16.005 (hot reload on SIGHUP) P1: post-v1.0 hot-reload convenience
- BC-2.16.007 (hot reload on file-watcher event) P1: post-v1.0 hot-reload convenience
