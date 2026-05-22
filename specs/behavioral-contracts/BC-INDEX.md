---
document_type: behavioral-contract-index
level: L3
version: "5.37"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 3.A
total_contracts: 240
active_contracts: 228
draft_contracts: 3
deprecated_contracts: 0
removed_contracts: 7
retired_contracts: 2
---

# Behavioral Contract Index

Flat index of all 240 behavioral contracts for Prism (240 total files, 228 active, 3 draft, 0 deprecated, 7 removed, 2 retired), organized by BC ID. Note: 5 prior index-only reserved entries (BC-2.07.007/008/009/010, BC-2.14.011) were dropped — they never had corresponding files.

**Note on `total_contracts`:** This count represents unique BC identifiers ever filed
(active + draft + deprecated + removed + retired = 228 + 3 + 0 + 7 + 2 = 240). Five prior index-only reserved entries
(BC-2.07.007/008/009/010, BC-2.14.011) were dropped in v4.8 because they never had
corresponding files — they are NOT counted in `total_contracts` and remain only in the
historical references section below. Counts are derived from workspace enumeration of
individual BC file `lifecycle_status` frontmatter fields (ground truth per VSDD). `draft_contracts: 3` covers BC-2.06.011, BC-2.21.001, and BC-2.16.013 (added D-731 PLUGIN-MIGRATION-001-D PO authoring). BC-2.01.016, BC-2.16.011, and BC-2.16.012 promoted draft→active at D-726 per POL-14 (PR #151 merge). BC-2.16.004 lifecycle_status was already `removed`; `status` field aligned to `removed` at D-726 (was draft, inconsistent). `deprecated_contracts: 0` — BC-2.16.004 fully removed.

Phase 3-patch additions (2026-04-16): 22 new BCs added in Burst 1 to close traceability gaps for AD-019 (WASM plugins), AD-020 (infusions), AD-021 (actions), CAP-022 (auto-case-creation), and BC-2.14.012 stub completion. Burst 2.5: 4 additional BCs closing remaining gaps flagged by story-writer: BC-2.08.008/009 (diagnostics tool + resources, S-5.08), BC-2.05.011 (audit forwarding at-least-once, S-5.10), BC-2.13.014 (IOC file loading, S-4.03).

| BC ID | Title | Subsystem | CAP | Priority | Status |
|-------|-------|-----------|-----|----------|--------|
| BC-2.01.001 | ~~Single-Client Sensor Query Returns Scoped Results~~ | 01 - Sensor Adapters | CAP-001 | P0 | removed |
| BC-2.01.002 | Cross-Client Fan-Out — Query Engine Orchestrates Parallel Sensor Fetches | 01 - Sensor Adapters | CAP-002 | P0 | draft |
| BC-2.01.003 | ~~Cursor-Based Forward-Only Pagination (MCP-Exposed)~~ | 01 - Sensor Adapters | CAP-001 | P0 | removed |
| BC-2.01.004 | Offset-Based Hybrid Pagination for Claroty Audit Logs | 01 - Sensor Adapters | CAP-001 | P0 | draft |
| BC-2.01.005 | CrowdStrike OAuth2 Authentication and Two-Step Fetch | 01 - Sensor Adapters | CAP-001 | P0 | draft (amendment_lifecycle: pending — ADR-023) |
| BC-2.01.006 | Cyberint Cookie-Based Authentication and Multi-Format Timestamp Parsing | 01 - Sensor Adapters | CAP-001 | P0 | draft (amendment_lifecycle: pending — ADR-023) |
| BC-2.01.007 | Claroty Bearer Token Auth with Polymorphic ID Handling | 01 - Sensor Adapters | CAP-001 | P0 | draft (amendment_lifecycle: pending — ADR-023) |
| BC-2.01.008 | Armis Bearer Token Auth with AQL Query Forwarding and Timestamp Fallback | 01 - Sensor Adapters | CAP-001 | P0 | draft (amendment_lifecycle: pending — ADR-023) |
| BC-2.01.009 | ~~Query Filtering and Sorting Parameters~~ | 01 - Sensor Adapters | CAP-001 | P0 | removed |
| BC-2.01.010 | Partial Failure Handling for Paginated and Cross-Client Queries | 01 - Sensor Adapters | CAP-001, CAP-002 | P0 | draft |
| BC-2.01.011 | ~~Cross-Sensor Correlation via OCSF Field Alignment~~ | 01 - Sensor Adapters | CAP-012 | P1 | removed |
| BC-2.01.012 | ~~Query Fingerprint Validation at Startup~~ | 01 - Sensor Adapters | CAP-001 | P0 | removed |
| BC-2.01.013 | DataSource Trait Eliminates Per-Sensor Code Duplication | 01 - Sensor Adapters | CAP-001 | P0 | active (promoted draft→active D-398 per POL-14; anchor story S-PLUGIN-PREREQ-A merged PR #142 develop@90d7c80f) |
| BC-2.01.014 | Exponential Backoff and Retry for Transient Sensor API Errors | 01 - Sensor Adapters | CAP-001 | P0 | draft |
| BC-2.01.015 | ~~MCP Tool Response Envelope Structure~~ | 01 - Sensor Adapters | CAP-001 | P0 | removed |
| BC-2.01.016 | SensorAuth Open Trait — Plugin-Implementable Auth Contract (No Sealed Marker) | 01 - Sensor Adapters | CAP-001 | P0 | active (promoted draft→active D-726 per POL-14; anchor story S-PLUGIN-PREREQ-E merged PR #151 develop@80ebe794 2026-05-19) — v1.10 |
| BC-2.02.001 | OCSF Schema Loading at Build Time via ocsf-proto-gen | 02 - OCSF Normalization | CAP-003 | P0 | draft |
| BC-2.02.002 | DynamicMessage Creation from Sensor Records | 02 - OCSF Normalization | CAP-003 | P0 | draft |
| BC-2.02.003 | CrowdStrike Alert Field Mapping to OCSF | 02 - OCSF Normalization | CAP-003 | P0 | draft (amendment_lifecycle: pending — ADR-023) |
| BC-2.02.004 | Cyberint Alert Field Mapping to OCSF | 02 - OCSF Normalization | CAP-003 | P0 | draft (amendment_lifecycle: pending — ADR-023) |
| BC-2.02.005 | Claroty xDome Field Mapping to OCSF (9 Data Sources) | 02 - OCSF Normalization | CAP-003 | P0 | draft (amendment_lifecycle: pending — ADR-023) |
| BC-2.02.006 | Armis Centrix Field Mapping to OCSF (7 Data Sources) | 02 - OCSF Normalization | CAP-003 | P0 | draft (amendment_lifecycle: pending — ADR-023) |
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
| BC-2.03.013 | CredentialStore Initialization — Reference Validation Only, No Values in Memory at Process Start | 03 - Credential Management | CAP-004 | P0 | active |
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
| BC-2.05.012 | AuditEmitter Initialization — audit_buffer CF Open and boot.audit.initialized Emitted at Process Start | 05 - Audit Trail | CAP-007 | P0 | active |
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
| BC-2.06.011 | ConfigManager Initialization — prism.toml Schema Validation at Process Start | 06 - Client Configuration | CAP-009 | P0 | draft |
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
| BC-2.09.001 | Structural Separation of Untrusted Data | 09 - Prompt Injection Defense | CAP-010 | P0 | active |
| BC-2.09.002 | Provenance Framing in Tool Descriptions | 09 - Prompt Injection Defense | CAP-010 | P0 | active |
| BC-2.09.003 | Suspicious Pattern Detection via Regex with NFKC Normalization | 09 - Prompt Injection Defense | CAP-010 | P0 | active |
| BC-2.09.004 | Safety Flags via _meta.safety_flags Array (Centralized, Not Per-Field) | 09 - Prompt Injection Defense | CAP-010 | P0 | active |
| BC-2.09.005 | Trust-Level Metadata Per Response | 09 - Prompt Injection Defense | CAP-010 | P0 | active |
| BC-2.09.006 | Tool Description Security Warnings | 09 - Prompt Injection Defense | CAP-010 | P0 | active |
| BC-2.09.007 | OutputSchema for Type-Safe LLM Reasoning | 09 - Prompt Injection Defense | CAP-010 | P0 | active |
| BC-2.09.008 | Response Envelope with Trust Annotations | 09 - Prompt Injection Defense | CAP-010 | P0 | active |
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
| BC-2.11.004 | PrismQL Pipe Mode Parsing | 11 - Query Execution | CAP-015 | P0 | active |
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
| BC-2.16.001 | Sensor Spec File Loading — Parse TOML, Validate Schema, Register Tables | 16 - Spec Engine | CAP-029 | P0 | draft — v1.6 (FB-IMPL-1-PO 2026-05-21: §Known Gaps added — KG-006-001 DEC-036 DataFusion-level unavailability marking is prism-query S-3.02 scope, not exercisable in prism-spec-engine; AC-006 parse-time PASS criterion scoped accordingly) |
| BC-2.16.002 | Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation | 16 - Spec Engine | CAP-029 | P0 | active (promoted draft→active D-427 per POL-14; anchor story S-PLUGIN-PREREQ-B merged PR #143 develop@ae7e26c8 2026-05-12) — v1.36 |
| BC-2.16.003 | Column-to-OCSF Mapping at Query Time — Map Sensor Columns to OCSF Fields Per Spec | 16 - Spec Engine | CAP-029 | P0 | draft |
| BC-2.16.004 | ~~Rust Escape Hatch for Custom Adapters — Trait-Based Override When Config Is Insufficient~~ | 16 - Spec Engine | CAP-029 | P0 | removed (lifecycle_status: removed since PREREQ-E impl; status aligned at D-726 per POL-14 PR #151 merge) — v1.5 |
| BC-2.16.005 | `reload_config` MCP Tool — Re-Read All Config Files, Validate, Atomic Swap, Notify | 16 - Spec Engine | CAP-030 | P1 | draft |
| BC-2.16.006 | Arc-Swap Config Access on Hot Path — Lock-Free Reads for Query-Time Config Access | 16 - Spec Engine | CAP-030 | P1 | draft |
| BC-2.16.007 | Sensor Spec Hot Reload — Add/Remove/Update Sensor Tables Without Restart | 16 - Spec Engine | CAP-030 | P1 | draft |
| BC-2.16.008 | `add_sensor_spec` MCP Tool — Upload a New Sensor Spec at Runtime | 16 - Spec Engine | CAP-029, CAP-030 | P0 | draft |
| BC-2.16.009 | Spec File Validation — Schema Validation, Variable Reference Resolution, OCSF Field Validation | 16 - Spec Engine | CAP-029 | P0 | draft |
| BC-2.16.010 | `list_sensor_specs` MCP Tool — List Loaded Sensor Specs with Table Schemas and Status | 16 - Spec Engine | CAP-029 | P0 | draft |
| BC-2.16.011 | CustomAdapter Rust Trait Retirement — Removal of Trait, Registry, and All Call Sites | 16 - Spec Engine | CAP-029 | P0 | active (promoted draft→active D-726 per POL-14; anchor story S-PLUGIN-PREREQ-E merged PR #151 develop@80ebe794 2026-05-19) — v1.12 |
| BC-2.16.012 | PluginRegistry Dispatch in spec_parser.rs — Hardcoded Sensor Names Replaced with Registry Lookup | 16 - Spec Engine | CAP-029 | P0 | active (promoted draft→active D-726 per POL-14; anchor story S-PLUGIN-PREREQ-E merged PR #151 develop@80ebe794 2026-05-19) — v1.30 |
| BC-2.16.013 | Bundled Sensor Spec Authoring and DTU-Parity Verification — 4 Initial Sensors | 16 - Spec Engine | CAP-029 | P0 | draft (v1.13 FB-IMPL-2 2026-05-21 — architect adjudication: Armis fallback chain corrected to `["first_seen"]`; DTU-EXT-005 page_size; ADR-028 v1.10) — v1.13 |
| BC-2.17.001 | Plugin Panic Isolation — Crashed Plugin Does Not Terminate Host Process | 17 - WASM Plugin Runtime | CAP-032 | P0 | active (POL-14 auto-promotion D-568 S-PLUGIN-PREREQ-D merge PR #149 ec90fe8f 2026-05-15) |
| BC-2.17.002 | Plugin Sandbox — No Direct Filesystem or Network Access | 17 - WASM Plugin Runtime | CAP-032 | P0 | active (POL-14 auto-promotion D-568 S-PLUGIN-PREREQ-D merge PR #149 ec90fe8f 2026-05-15) |
| BC-2.17.003 | Plugin Sandbox — Memory Limit Enforced Per Plugin Instance (default 64MB) | 17 - WASM Plugin Runtime | CAP-032 | P0 | active (POL-14 auto-promotion D-568 S-PLUGIN-PREREQ-D merge PR #149 ec90fe8f 2026-05-15) |
| BC-2.17.004 | Plugin Sandbox — CPU Time Limit Enforced via Epoch Interruption (default 5s) | 17 - WASM Plugin Runtime | CAP-032 | P0 | active (POL-14 auto-promotion D-568 S-PLUGIN-PREREQ-D merge PR #149 ec90fe8f 2026-05-15) |
| BC-2.17.005 | Plugin Hot Reload — Atomic Module Swap, In-Flight Calls Complete Against Old Version | 17 - WASM Plugin Runtime | CAP-030, CAP-032 | P0 | draft |
| BC-2.17.006 | WIT Interface Validation Before Plugin Registration | 17 - WASM Plugin Runtime | CAP-032 | P0 | active (POL-14 auto-promotion D-568 S-PLUGIN-PREREQ-D merge PR #149 ec90fe8f 2026-05-15) |
| BC-2.17.007 | Plugin Manifest Schema Validation Before WIT Validation | 17 - WASM Plugin Runtime | CAP-032 | P0 | active (POL-14 auto-promotion D-568 S-PLUGIN-PREREQ-D merge PR #149 ec90fe8f 2026-05-15) |
| BC-2.18.001 | Alert and Case Action Triggers — At-Least-Once Delivery with Exponential Backoff Retry | 18 - Action Delivery Engine | CAP-033 | P0 | draft |
| BC-2.18.002 | Schedule Action Triggers — Best-Effort, Retry on Next Cron Tick | 18 - Action Delivery Engine | CAP-033 | P0 | draft |
| BC-2.18.003 | Manual Action Triggers — Fire-and-Forget, Result Returned Immediately to AI Caller | 18 - Action Delivery Engine | CAP-033 | P0 | draft |
| BC-2.18.004 | Action Delivery Semaphore — 8-Permit Independent Pool, try_acquire() Skip-If-Unavailable | 18 - Action Delivery Engine | CAP-033 | P0 | draft |
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
| BC-2.21.001 | OrgRegistry Initialization — Bijective Resolution Verified at Process Start | 21 - Identity & Core Types | CAP-038 | P0 | draft |
| BC-2.22.001 | Boot Orchestration — Sequencing, Exit-Code Map, and Pre-Traffic Gate | 22 - Binary Entrypoint | CAP-034 | P0 | active |

## Wave 3 — Phase 3.A Behavioral Contracts (2026-04-27)

22 new BCs registered in v4.15 (Wave 3 Phase 3.A spec burst). All BCs at v0.2 PROPOSED.

**Wave 3 BC Family 3.1 — Multi-Tenant Identity (ADR-006)**

| BC ID | Title | Subsystem | CAP | Priority | Status |
|-------|-------|-----------|-----|----------|--------|
| BC-3.1.001 | OrgRegistry Bijective Slug/UUID Resolution | SS-21 (Identity & Core Types) | CAP-038 | P0 | draft |
| BC-3.1.002 | Audit Entry Carries Both org_id and org_slug at Construction Time | SS-05 (Audit Trail) | CAP-007 | P0 | draft |
| BC-3.1.003 | OrgRegistry Maintains Strict Bijectivity at All Times | SS-21 (Identity & Core Types) | CAP-038 | P0 | draft |
| BC-3.1.004 | OrgRegistry Rejects Duplicate Slugs and UUIDs at Registration | SS-21 (Identity & Core Types) | CAP-038 | P0 | draft |

**Wave 3 BC Family 3.2 — Per-Org Data and Credential Isolation (ADR-006)**

| BC ID | Title | Subsystem | CAP | Priority | Status |
|-------|-------|-----------|-----|----------|--------|
| BC-3.2.001 | Per-Org Sensor Data Isolation via Composite HashMap Key | SS-01 (Sensor Adapters) | CAP-001 | P0 | draft |
| BC-3.2.002 | Per-Org Credential Isolation via OrgId-Keyed Namespace | SS-03 (Credential Management) | CAP-004 | P0 | draft |
| BC-3.2.003 | Per-Org Session Token Isolation via (OrgId, token) Composite Key | SS-03 (Credential Management) | CAP-004 | P0 | draft |
| BC-3.2.004 | Shared-Mode DTU Tags OrgId in Payload Body Not in Routing Headers | SS-01 (Sensor Adapters) | CAP-040 | P0 | draft |
| BC-3.2.005 | DTU Mode is Deployment-Time Config — No Runtime API to Change It | SS-06 (Client Configuration) | CAP-040 | P0 | draft |

**Wave 3 BC Family 3.3 — Customer Config Validation (ADR-007, ADR-010)**

| BC ID | Title | Subsystem | CAP | Priority | Status |
|-------|-------|-----------|-----|----------|--------|
| BC-3.3.001 | Startup Rejects Security Telemetry DTU Type Declared with Shared Mode | SS-06 (Client Configuration) | CAP-009 | P0 | draft |
| BC-3.3.002 | No Credential Values in Customer Config Files | SS-06 (Client Configuration) | CAP-009 | P0 | draft |
| BC-3.3.003 | Schema Version Enforcement Rejects Unknown or Missing schema_version | SS-06 (Client Configuration) | CAP-009 | P0 | draft |
| BC-3.3.004 | Customer Config Validation Rejects Invalid Schema at Startup | SS-06 (Client Configuration) | CAP-009 | P0 | draft |

**Wave 3 BC Family 3.4 — Multi-Tenant Data Generator (ADR-009)**

| BC ID | Title | Subsystem | CAP | Priority | Status |
|-------|-------|-----------|-----|----------|--------|
| BC-3.4.001 | Generator Determinism — Identical Inputs Produce Byte-Identical FixtureSet | SS-01 (Sensor Adapters) | CAP-039 | P0 | draft |
| BC-3.4.002 | Generator Output Schema-Validates Against Canonical Vendor API Spec | SS-01 (Sensor Adapters) | CAP-039 | P0 | draft |
| BC-3.4.003 | Archetype Catalog Enumeration — 8 Archetypes with Defined Baselines | SS-01 (Sensor Adapters) | CAP-039 | P0 | draft |
| BC-3.4.004 | Org-Tagged Record IDs — Every Generated Record Carries an Org-Derived ID Prefix | SS-01 (Sensor Adapters) | CAP-039 | P0 | draft |

**Wave 3 BC Family 3.5 — DTU Test Harness Isolation (ADR-011)**

| BC ID | Title | Subsystem | CAP | Priority | Status |
|-------|-------|-----------|-----|----------|--------|
| BC-3.5.001 | Harness Logical Isolation Invariants | SS-01 (Sensor Adapters) | CAP-036 | P0 | draft |
| BC-3.5.002 | Harness Network Isolation Invariants | SS-01 (Sensor Adapters) | CAP-036 | P0 | draft |

**Wave 3 BC Family 3.6 — Harness Fault Injection (ADR-011)**

| BC ID | Title | Subsystem | CAP | Priority | Status |
|-------|-------|-----------|-----|----------|--------|
| BC-3.6.001 | Per-Org Failure Injection | SS-01 (Sensor Adapters) | CAP-036 | P0 | draft |
| BC-3.6.002 | Harness Crash Detection | SS-01 (Sensor Adapters) | CAP-036 | P0 | draft |

**Wave 3 BC Family 3.7 — Workspace Conventions (ADR-012)**

| BC ID | Title | Subsystem | CAP | Priority | Status |
|-------|-------|-----------|-----|----------|--------|
| BC-3.7.001 | Workspace src/ Convention Lint Enforcement | SS-01 (Sensor Adapters) | CAP-037 | P1 | draft |

## Summary

| Subsystem | BC Count | P0 | P1 | Removed | Retired |
|-----------|----------|----|----|---------|---------|
| 01 - Sensor Adapters | 9 | 9 | 0 | 6 | 0 |
| 02 - OCSF Normalization | 12 | 12 | 0 | 0 | 0 |
| 03 - Credential Management | 13 | 13 | 0 | 0 | 0 |
| 04 - Feature Flags | 15 | 9 | 6 | 0 | 0 |
| 05 - Audit Trail | 12 | 12 | 0 | 0 | 0 |
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
| 16 - Spec Engine | 9 | 6 | 3 | 0 | 0 |
| 17 - WASM Plugin Runtime | 7 | 7 | 0 | 0 | 0 |
| 18 - Action Delivery Engine | 9 | 9 | 0 | 0 | 0 |
| 19 - Infusion Enrichment Framework | 5 | 5 | 0 | 0 | 0 |
| 20 - Observability / Log Forwarding | 5 | 5 | 0 | 0 | 0 |
| Wave 3 BC Family: 3.1 - Multi-Tenant Identity | 4 | 4 | 0 | 0 | 0 |
| Wave 3 BC Family: 3.2 - Per-Org Data & Credential Isolation | 5 | 5 | 0 | 0 | 0 |
| Wave 3 BC Family: 3.3 - Customer Config Validation | 4 | 4 | 0 | 0 | 0 |
| Wave 3 BC Family: 3.4 - Multi-Tenant Data Generator | 4 | 4 | 0 | 0 | 0 |
| Wave 3 BC Family: 3.5 - DTU Test Harness Isolation | 2 | 2 | 0 | 0 | 0 |
| Wave 3 BC Family: 3.6 - Harness Fault Injection | 2 | 2 | 0 | 0 | 0 |
| Wave 3 BC Family: 3.7 - Workspace Conventions | 1 | 0 | 1 | 0 | 0 |
| 22 - Process Lifecycle | 1 | 1 | 0 | 0 | 0 |
| **Total** | **225** | **195** | **30** | **6** | **2** |

**Note (v4.51):** 5 BCs promoted draft→active per D-319 (S-WAVE5-PREP-01 merged at develop@53b87961 2026-05-10T00:55:49Z): BC-2.03.013, BC-2.05.012, BC-2.06.011, BC-2.21.001, BC-2.22.001. active_contracts 222→227. total_contracts=235 (227 active + 6 removed + 2 retired).

**Note (v5.08 reconciliation):** The "222" in the v4.51 note refers to the Summary table Total BC Count at that moment (set at v4.19 when Wave 3 BCs were added). The Summary table was never updated after v4.19 promotions. v4.81 D-572 corrected the frontmatter `active_contracts` to 225 via empirical enumeration; the Summary table Total row was corrected to 225 in v5.08. BC-2.06.011 and BC-2.21.001 were listed as v4.51 promotions but file frontmatter shows `lifecycle_status: draft` for both (confirmed v4.81 + v5.08 enumeration) — they were NOT actually promoted; the v4.51 list was aspirational. POL-14 promotion for BC-2.06.011 + BC-2.21.001 requires a future merge event.

### Phase 3-Patch Additions (2026-04-16)

**26 new BCs added (22 Burst 1 + 4 Burst 2.5):**

**Burst 1:**
- BC-2.14.012: Acknowledge Alert MCP Tool (stub completed — was placeholder since phase 1a)
- BC-2.14.013: Auto-Case-Creation from High-Severity Detection Rules (CAP-022 tracking note fulfilled)
- BC-2.17.001 through BC-2.17.007: WASM Plugin Runtime (AD-019) — 7 BCs from INV-PLUGIN-001 through INV-PLUGIN-006 + BC-2.17.007 manifest schema validation (closes F-LP1-HIGH-004)
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

**v5.37 (2026-05-21, FB-IMPL-2):** architect | BC-2.16.013 v1.12→v1.13 (F-LP2-HIGH-004 Option a: Armis `timestamp_fallback_chain` corrected from `["last_seen","first_seen"]` to `["first_seen"]` — self-referential primary column in chain is semantic no-op; false doc-comment fixed; implementer skip-guard required in pipeline.rs. F-LP2-MEDIUM-001 Option b: DTU-EXT-005 added to §Known Gaps — `page_size` removed from cyberint.sensor.toml pagination block; `AlertListParams` struct has no `page_size` field. ADR-028 v1.9→v1.10 §D8-B + §D9 amended). BC-INDEX row 221 updated to v1.13. BC-INDEX v5.36→v5.37.

**v5.36 (2026-05-21, FB-IMPL-1 PO):** product-owner | BC-2.16.002 in-line row 210 v1.35→v1.36 (FB-IMPL-1 routing gap closure: `timestamp.fallback_to_now` WARN event_type catalog row 35 added per ADR-028 v1.9 §D8-B Option A grammar contract; catalog count 34→35; catalog bullet label `(v1.22)` → `(v1.23)` per POL-30 Fork B; architect bumped BC-2.16.013 v1.11→v1.12 in same FB-IMPL-1 burst specifying the tracing emission but did not propagate to BC-2.16.002 catalog). POL-30 Fork B sibling-sweep: error-taxonomy v1.43→v1.44 (2 cite-pin sites: E-PLUGIN-020 + E-PIPELINE-001) + BC-2.16.012 in-line row 220 v1.29→v1.30 (3 cite-pin sites: §Postconditions line 84 ×2 + EC-016-012-005 line 110). BC-INDEX v5.35→v5.36.

**v5.35 (2026-05-21, FB-IMPL-1-PO):** product-owner | BC-2.16.001 v1.5→v1.6 (F-LP1-HIGH-005 closure Option a): §Known Gaps KG-006-001 added — DEC-036 DataFusion-level unavailability marking not exercisable in prism-spec-engine per AD-015; AC-006 PASS criterion scoped to parse-time only; gap closes in S-3.02. BC-INDEX row 209 updated to v1.6. BC-INDEX v5.34→v5.35.

**v5.34 (2026-05-21, FB-IMPL-1):** architect | (D-FB-IMPL-1-OPT-A) BC-2.16.013 v1.11→v1.12 (§O-001 LOCKED Option A; E-SPEC-018 TimestampParseFailure registered in error-taxonomy.md v1.42→v1.43; documented-gap exception ADR-028 §D9; co-merge contract ADR-028 §D10; ADR-028 v1.8→v1.9 cite-pin sweep across 6 Architecture Anchors sites + Traceability row; Cyberint + Armis §Postconditions §1 updated from WASM-plugin language to Option A grammar). BC-INDEX row 221 updated to v1.12. BC-INDEX v5.33→v5.34.

**v5.33 (2026-05-21, FB-IMPL-P22-PO):** product-owner | BC-2.16.013 v1.10→v1.11 (F-LP22-MED-001 closure: error-taxonomy.md v1.41→v1.42 cite-pin sweep at §Error Conditions E-SPEC-017 row; chain propagation of BC-2.16.013 v1.10→v1.11 into story cite-pins — 8 sites). BC-INDEX row 221 updated to v1.11. BC-INDEX v5.32→v5.33.

**v5.32 (2026-05-20, FB-IMPL-P17-PO):** product-owner | BC-2.16.013 v1.9→v1.10 (F-LP17-HIGH-002 propagation closure: ADR-028 v1.7→v1.8 cite-pin sweep — 6 active-prose sites in §Architecture Anchors lines 375-379 and §Traceability ADR anchors line 403; POL-29 fixed-point per F-LP16-OBS-001 — architect FB-IMPL-P17-ARCH reverted ADR-028 §Changelog to descending + bumped v1.7→v1.8 + added §D7; cites bump only). BC-INDEX row 221 updated to v1.10. BC-INDEX v5.31→v5.32.

**v5.31 (2026-05-20, FB-IMPL-P16-PO):** product-owner | BC-2.16.013 v1.8→v1.9 (F-LP16-MED-001 propagation closure: ADR-028 v1.6→v1.7 cite-pin sweep — 6 active-prose sites in §Architecture Anchors lines 375-379 and §Traceability ADR anchors line 403; POL-29 fixed-point per F-LP16-OBS-001 codification — same-burst sweep of architect bump's own stale-cite class; workspace grep confirmed clean). BC-INDEX row 221 updated to v1.9. BC-INDEX v5.30→v5.31.

**v5.30 (2026-05-20, FB-IMPL-P15-PO):** product-owner | BC-2.16.013 v1.7→v1.8 (F-LP15-MED-001 closure: ADR-028 v1.5→v1.6 cite-pin sweep — 6 active-prose sites in §Architecture Anchors lines 375-379 and §Traceability ADR anchors line 403; POL-29 cross-file sweep closes F-LP15-OBS-001 process-gap; workspace grep confirmed no other active-prose stale cites outside BC-2.16.013). BC-INDEX row 221 updated to v1.8. BC-INDEX v5.29→v5.30.

**v5.29 (2026-05-20, FB-IMPL-P13-PO):** product-owner | BC-2.16.013 v1.6→v1.7 (F-LP13-MED-002 ADR-028 v1.5 pin propagation): §Architecture Anchors updated to versioned ADR-028 v1.5 citations; §D6 anchor added (PLUGIN-MIGRATION-001-A auth migration scope); Claroty/Cyberint/Armis §Postconditions auth rows updated with ADR-028 §D2 supersession of ADR-026 §D3 (D-747) context. BC-INDEX row updated to v1.7. BC-INDEX v5.28→v5.29.

**v5.28 (2026-05-20, FB-IMPL-P7 D-741):** state-manager | BC-2.16.013 row 221 in-line text bumped v1.5 → v1.6 (FB-IMPL-P6-PO content: Armis auth-grounding cite swept to module-level `//!` doc-comment anchor per TD-VSDD-091 + POL-25 sibling-anti-pattern sweep). Closes F-LP7-MED-001 POL-29 BC-INDEX in-line row drift. BC-INDEX v5.27→v5.28.

**v5.27 (2026-05-20, FB-IMPL-P6-PO fix-burst-6):** product-owner | BC-2.16.013 v1.5→v1.6 (pass-6 F-LP6-LOW-001 TD-VSDD-091 sibling-asymmetric): Armis auth-grounding cite replaced — `lib.rs:16-17` → `crates/prism-dtu-armis/src/lib.rs module documentation` in §Postconditions §1 Armis auth-grounding sentence. POL-25 sweep: HS-016 v1.1→v1.2 updated in same burst. BC-INDEX v5.26→v5.27.

**v5.26 (2026-05-20, FB-IMPL-P5-PO fix-burst-5):** product-owner | BC-2.16.013 v1.4→v1.5 (pass-5 F-LP5-LOW-001 TD-VSDD-091): Cyberint auth-grounding cite replaced — `alerts.rs:43-46` → `alerts.rs::extract_session_token()` (symbol anchor per TD-VSDD-091 anti-volatile-pin). POL-25 sweep: HS-015 v1.1→v1.2 updated in same burst. BC-2.16.001 `modified: null` → `"2026-05-20"` (POL-27 F-LP5-MED-002 closure — frontmatter-only, no version bump). BC-2.16.009 `modified: null` → `"2026-05-20"` (POL-27 sibling-sweep — v1.4 had 2026-05-20 Changelog entry). BC-INDEX v5.25→v5.26.

**v5.25 (2026-05-20, FB-IMPL-P4-PO fix-burst-4):** product-owner | BC-2.16.013 v1.3→v1.4 (pass-4 F-LP4-HIGH-001/F-LP4-HIGH-002/F-LP4-HIGH-004/F-LP4-MED-003): URL re-grounding against DTU clone routes (ADR-028 §D1) — CrowdStrike detections `/detects/queries/detects/v1`+`/detects/entities/summaries/GET/v1`; devices `/devices/queries/devices/v1`+`/devices/entities/devices/v2`; Cyberint alerts `/api/v1/alerts`; Claroty alerts `/api/v1/alerts`. auth_type DTU-grounded (ADR-028 §D2) — claroty=`bearer_static`, cyberint=`cookie_roundtrip`, armis=`bearer_static`, crowdstrike=`oauth2_client_credentials` (unchanged). Fixture-JSON parity mechanism (ADR-028 §D3) — reference OCSF loaded from `crates/prism-dtu-{sensor}/fixtures/parity/reference-ocsf/<table>.json`; no prism-sensors dev-dep. §Known Gaps DTU-EXT-001..004 added. request_count relaxed to >=2. ADR-028 §D1/D2/D3/D4/D5 cited in §inputs + §Architecture Anchors. BC-2.16.001 v1.4→v1.5 (F-LP4-HIGH-003 + F-LP4-MED-002): E-SPEC-017 enforcement contract expanded with SpecErrorCode::ESpec017 variant in prism-core; load_all()/parse_spec_directory() emits, parse(toml_input) does NOT; RG-09/HS-018 must use load_all() driver. BC-INDEX v5.24→v5.25.

**v5.24 (2026-05-20, FB-IMPL-P3-PO fix-burst-3):** product-owner | BC-2.16.013 v1.2→v1.3 (pass-3 F-LP3-CRIT-001/F-LP3-CRIT-002/F-LP3-CRIT-003/F-LP3-HIGH-001/F-LP3-HIGH-002): F-LP3-CRIT-001: phantom `spec_parser::parse_spec_file()` replaced with `SpecLoader::parse(toml_input: &str)` (CODE-GROUNDED spec_parser.rs:655) in §Postconditions §2 step 2 and §Canonical Test Vectors. F-LP3-CRIT-002: CrowdStrike URL phantoms corrected — `/detects/queries/detects/v1` et al. replaced with actual format patterns from crowdstrike.rs:262,315 (`/queries/{resource_type}` QueryV2, `/entities/{resource_type}/GET` PostEntities); incidents table corrected from single-step GET to two-step pattern. F-LP3-CRIT-003: Claroty `/xdome` prefix stripped — actual pattern is `/api/v1/{resource}s` per claroty.rs:244; NO `/xdome`. F-LP3-HIGH-001: Cyberint `/v1` segment removed — actual pattern is `/api/{resource}s` per cyberint.rs:251; NO `/v1`. F-LP3-HIGH-002: Armis endpoint corrected — single `/api/v1/search` (armis.rs:517, no trailing slash) for ALL tables; `devices` and `alerts` discriminated by AQL expression (`DEFAULT_AQL_TEMPLATE = "in:{table}"` per armis.rs:72) not by separate endpoint paths. HS-013 scenario description corrected to use actual CrowdStrike URL pattern. HS-014 scenario description `/xdome` prefix stripped. HS-017 `parse_spec_file` phantom replaced with `SpecLoader::parse` in sub-scenarios and known-good corpus. BC-INDEX v5.22→v5.24 (v5.23 frontmatter bump was missed in prior burst; correcting now). Note: F-LP3-MED-001 (OrgSlug::new_unchecked comment in story) is story-writer scope; see story-writer handoff below.

**v5.23 (2026-05-20, FB-IMPL-P2-PO fix-burst-2):** product-owner | BC-2.16.013 v1.1→v1.2 (pass-2 F-001/F-002/F-003/F-004/F-005): auth_type strings corrected [claroty→cookie_roundtrip, cyberint→bearer_static] per CODE-GROUNDED verification; E-SPEC-017 introduced for filename-stem mismatch (distinct from E-SPEC-009 duplicate-sensor_id); CrowdStrikeAdapter::fetch_page() phantom replaced with SensorAdapter::fetch() (actual symbol); ${query.aql}→${query.filter.aql} in test vectors; TD-VSDD-091 line-number citations removed. BC-2.16.001 v1.3→v1.4 (E-SPEC-017 added to Error Conditions). BC-2.16.009 v1.3→v1.4 (E-SPEC-002/E-SPEC-003 added to Error Conditions; auth_type 4→5 value set adding custom_via_plugin per CODE-GROUNDED VALID_AUTH_TYPES). error-taxonomy.md v1.40→v1.41 (E-SPEC-017 registered; E-SPEC-015/E-SPEC-016 tombstone rows added). HS-013..HS-018 epic_id aligned to PLUGIN-MIGRATION-001. BC-INDEX v5.22→v5.23.

**v5.22 (2026-05-20, FB-IMPL-P1-PO fix-burst-1):** product-owner | BC-2.16.013 v1.0→v1.1 — closes pass-1 adversarial findings F-001 (DTU API signature corrected: `BehavioralClone::start_on(bind, shutdown, tls)`), F-002 (PipelineExecutor::execute 5-arg signature corrected), F-004 (E-SPEC-015 retired as test verdict; E-SPEC-016 replaced with existing E-SPEC-009), F-006 (ADR-023 §Decision Rules — Rule 1/3 anchor fix), F-007 (ADR-023 §Architectural Constraints — C2 anchor fix replacing phantom ADR-022 §C2), O-001 (grammar verification table added: `fan_out_batch_size` confirmed supported; `${query.filter.aql}` corrected from non-existent `${query.aql}`; `timestamp_format`/`timestamp_fallback_chain` confirmed NOT supported in current grammar — prerequisite documented). BC-INDEX v5.21→v5.22.

**v5.21 (2026-05-20, D-731 PLUGIN-MIGRATION-001-D PO authoring):** product-owner | BC-2.16.013 registered — new draft BC "Bundled Sensor Spec Authoring and DTU-Parity Verification — 4 Initial Sensors" (SS-16/CAP-029/P0; origin: brownfield; anchor story PLUGIN-MIGRATION-001-D; VP-148/VP-PLUGIN-003 primary contract). total_contracts 239→240, draft_contracts 2→3. BC-INDEX v5.20→v5.21.

**v5.20 (2026-05-19, D-726-post-merge):** state-manager | POL-14 BC auto-promotion at PR #151 merge (develop@80ebe794 2026-05-19T18:06:44Z): BC-2.01.016 v1.9→v1.10 status draft→active + BC-2.16.011 v1.11→v1.12 status draft→active + BC-2.16.012 v1.28→v1.29 status draft→active + BC-2.16.004 v1.4→v1.5 status draft→removed (lifecycle_status alignment). BC-INDEX active_contracts 225→228, draft_contracts 5→2, deprecated_contracts 1→0, removed_contracts 6→7. BC-INDEX v5.19→v5.20.

**v5.19 (2026-05-19, FB-PR-1):** product-owner | BC-2.16.011 v1.10→v1.11 (FB-PR-1 AC-11 relocation: §Error Cases E-SPEC-008 row updated to two-layer enforcement model per architect adjudication FB-PR-1-error-taxonomy-test-relocation.md Option 1; code-side Rust test gate + spec-side hook gate). BC-INDEX v5.18→v5.19.

**v5.18 (2026-05-18, pass-11-spec-hygiene):** product-owner | BC-2.16.002 in-line row 212 v1.34→v1.35 (F-LP-IMPL-P11-HIGH-001: frontmatter `deprecated/deprecated_by` YAML concatenation defect fixed). BC-INDEX v5.17→v5.18.

**v5.17 (2026-05-18, pass-10-spec-hygiene):** product-owner | BC-2.16.002 in-line row 212 v1.33→v1.34 (F-LP-IMPL-P10-SUG-001 Option B: catalog bullet label `(v1.21)` → `(v1.22)`) + BC-2.16.012 in-line row 222 v1.27→v1.28 (F-LP-IMPL-P10-IMP-002: E-PLUGIN-021 §Error Cases row + EC-016-012-006 §Edge Cases row; catalog cite-pin `(v1.21)` → `(v1.22)` at 2 sites). BC-INDEX v5.16→v5.17.

**v5.16 (2026-05-18, FB-IMPL-4 D-707):** state-manager | BC-2.01.016 in-line row 49 v1.8→v1.9 (E-SPEC-014 backend qualification appended; D-706 architect adjudication applied) + BC-2.16.002 in-line row 212 v1.32→v1.33 (F-LP-IMPL-P5-003 intro catalog count 33→34 sync). BC-INDEX v5.15→v5.16.

**v5.15 (2026-05-18, FB-IMPL-3 D-705):** state-manager | BC-2.16.002 in-line row 212 v1.31→v1.32 (F-LP-IMPL-P4-002 Route A: implementer added `plugin_registration_rolled_back` catalog row 34; state-manager frontmatter sync + INDEX row bump) + BC-2.16.012 in-line row 222 v1.26→v1.27 (F-LP-IMPL-P4-002 Route A: implementer clarified EC-016-012-004 fail-closed semantics; state-manager frontmatter sync + INDEX row bump). BC-INDEX v5.14→v5.15.

**v5.14 (2026-05-17, FB74 D-696):** state-manager | BC-2.16.002 in-line row 212 v1.30→v1.31 (F-LP86-HIGH-001 closure: step 8f sibling-INDEX gap — BC-INDEX in-line table row summary cell was stale at v1.30; §Changelog v5.13 declared v1.30→v1.31 but did NOT update in-line row 212) + §Changelog v5.13 narrative correction (F-LP86-MED-001 closure: catalog bullet factual error — "POL-30 Fork B catalog bullet (v1.22) UNCHANGED" corrected to "(v1.21) UNCHANGED" per BC-2.16.002 line 74). BC-INDEX v5.13→v5.14.

**v5.13 (2026-05-17, FB73 D-695):** state-manager | BC-2.16.011 v1.9→v1.10 (PO F-LP85-HIGH-001 closure: ADR-026 D7 v1.22→v1.23 sweep — cross-value-class side-effect bump; step 8g first-application) + BC-2.16.012 v1.25→v1.26 (PO F-LP85-HIGH-001 closure: ADR-026 D7 v1.22→v1.23 sweep at 4 live-narrative sites — same cross-value-class META-class) + BC-2.16.002 v1.30→v1.31 (PO F-LP85-HIGH-001 closure: ADR-026 D7 v1.22→v1.23 sweep at catalog-row live-narrative site; POL-30 Fork B catalog bullet (v1.21) UNCHANGED per POL-30). BC-INDEX v5.12→v5.13.

**v5.12 (2026-05-17, FB70 D-692):** state-manager | BC-2.16.002 v1.29→v1.30 (F-LP82-HIGH-001 closure — line 110 row 33 body cite-pin advanced ADR-026 D7 v1.21→v1.22; FB69 misapplied POL-30 Fork B framing retracted per FB55/FB56b/FB62 3-burst precedent; line 74 catalog bullet (v1.21) correctly preserved per Fork B canonical rule). BC-INDEX v5.11→v5.12.

**v5.11 (2026-05-17, FB69 D-691):** state-manager | BC-2.16.011 v1.8→v1.9 (PO F-LP81-HIGH-001 closure: INV-ADAPTER-RETIRE-003 + Precondition rewrite preserving CustomAdapter-cleanup intent while reflecting BC-2.16.012 sibling-scope 1-line insertion truth — 37-pass-surviving BC↔story semantic contradiction) + BC-2.16.012 v1.24→v1.25 (PO F-LP81-HIGH-002 closure: ADR-026 D7 v1.21→v1.22 sweep at 4 live-narrative sites — META-META step 8b self-induced bump gap) + BC-2.16.002 v1.28→v1.29 (PO F-LP81-HIGH-002 closure: frontmatter-only bump — POL-30 Fork B catalog bullet (v1.21) UNCHANGED per POL-30). BC-INDEX v5.10→v5.11.

**v5.10 (2026-05-17, FB64 D-686):** BC-2.16.012 v1.23→v1.24 + BC-2.16.002 v1.27→v1.28 (PO burst-label sweep closure F-LP76-HIGH-001 §Changelog row cells corrected FB74→FB62; POL-30 Fork B catalog bullet (v1.21) UNCHANGED).

**v5.09 (2026-05-17):** state-manager | FB62 D-684 | F-LP74-HIGH-001 closure (state-manager scope): BC-2.16.012 v1.22→v1.23 (PO cascade: ADR-026 D7 v1.19→v1.21 sweep at 4 live-narrative sites) + BC-2.16.002 v1.26→v1.27 (PO cascade: ADR-026 D7 v1.19→v1.21 sweep at 1 live-narrative site; POL-30 Fork B catalog bullet (v1.21) UNCHANGED). BC-INDEX v5.08→v5.09.

**v5.08 (2026-05-17):** state-manager | FB61 D-683 | F-LP73-MED-001 closure (state-manager scope): BC count reconciliation — empirical lifecycle_status enumeration of all 239 BC files yielded active=225, draft=5, deprecated=1, removed=6, retired=2, total=239; frontmatter `active_contracts` was 225 (correct per v4.81 D-572 enumeration); Summary table §Total BC Count was 222 (stale from v4.19, never updated after D-319 v4.51 promotions + D-398 v4.55 + D-427 v4.60 + D-568 v4.80 + D-572 v4.81 corrections); per-row reconciliation: SS-03 12→13 (BC-2.03.013 promoted active at v4.55), SS-05 11→12 (BC-2.05.012 promoted active at v4.51), SS-16 10→9 (BC-2.16.004 deprecated at v4.54 + BC-2.16.011/012 added as draft not active), SS-17 6→7 (BC-2.17.007 added at v4.64 as 7th active P0), SS-22 (new row) 0→1 (BC-2.22.001 active); Summary table Total row 222→225 BC Count, P0 192→195; SS-22 row added (1 BC Count / 1 P0); Note v4.51 "active_contracts 222→227" is historically accurate (Summary table value at that moment was 222; v4.81 corrected the frontmatter discrepancy; BC-2.06.011 + BC-2.21.001 remained draft per file frontmatter despite being listed in v4.51 promotions). POL-11 + POL-14 + POL-26 consistency restored. BC-INDEX v5.07→v5.08.

**v5.07 (2026-05-17):** state-manager | FB60 D-682 | F-LP72-HIGH-002 closure (state-manager scope; production-grade sibling-CLASS sweep per CLAUDE.md Canonical Principle Rule 4): trailing `| v1.x |` cells removed from 10 catalog rows (PREREQ-E targets BC-2.01.016 + BC-2.16.011 + BC-2.16.012 + sibling-class catches BC-2.17.001 + BC-2.17.002 + BC-2.17.003 + BC-2.17.004 + BC-2.17.006 + BC-2.17.007 + BC-2.22.001). FB54 v1.57 canonical precedent ("Version-tracking lives in §Changelog rows per existing convention for all 154 catalog rows") propagated to BC-INDEX after 18-pass-surviving gap. POL-26 schema_integrity + POL-4 semantic_anchoring_integrity + POL-29 sibling-CLASS sweep restored across BC-INDEX. BC-INDEX v5.06→v5.07.

**v5.06 (2026-05-17):** state-manager | FB56b D-678 combined | BC-2.16.012 v1.21→v1.22 (FB56b architect/PO cascade: ADR-026 D7 v1.18→v1.19 sweep at 4 live-narrative sites) + BC-2.16.002 v1.25→v1.26 (FB56b PO cascade: ADR-026 D7 v1.18→v1.19 sweep at 1 catalog-row live-narrative site). POL-29 v1.17 step 8a FIRST APPLICATION META-cascade catch. BC-INDEX v5.05→v5.06.

**v5.05 (2026-05-17):** state-manager | FB55 D-677 | BC-2.16.012 v1.20→v1.21 (F-LP67-HIGH-001 closure: ADR-026 D7 pin v1.17→v1.18 at lines 84, 109, 124, 138; POL-29 v1.16 class (b) recurrence #18) + BC-2.16.002 v1.24→v1.25 (F-LP67-HIGH-001 sibling-sweep extension: catalog row line 110 D7 v1.17→v1.18; correct-agent-pattern PREREQ-B PG-LP11-001 precedent)

**v5.04 (2026-05-17):** state-manager | FB51 | BC-2.01.016 v1.7→v1.8 (F-LP63-MED-001 split provenance closure; POL-23 sibling sweep identified BC-2.16.011 same pattern) + BC-2.16.011 v1.7→v1.8 (POL-23 sibling-sweep: same mis-anchored PLUGIN-AUDIT-001 HIGH-3 cite pattern; both closed in single FB51 burst)

**v5.03 (2026-05-17):** state-manager | FB50 | BC-2.16.012 v1.19→v1.20 + BC-2.16.002 v1.23→v1.24 (POL-29 sibling-sweep for OBS-LP62-002 D7 v1.16/v1.10 → v1.17 across architect-domain; 17-site total sweep Interpretation #2 per production-grade default Rule 4)

**v5.02 (2026-05-17):** state-manager | FB48 | BC-2.16.012 v1.18→v1.19 (F-LP60-HIGH-001 §Changelog row position bookkeeping repair: v1.16/v1.17/v1.18 to descending top per D-611/D-628/D-635/D-659 precedent)

**v5.01 (2026-05-16):** state-manager | FB47 | BC-2.16.011 v1.6→v1.7 (F-LP59-MED-001 §Architecture Anchors line 178 ADR-027 framing label update per ADR-027 v1.8 title "Same-Burst Removal — Perimeter Enforcement in Wave 1/A")

**v5.00 (2026-05-16):** state-manager | FB45 | BC-2.16.012 v1.17→v1.18 (POL-23 sibling-sweep: 4 ADR-026 D7 live-narrative pins v1.15→v1.16)

**v4.99 (2026-05-16):** state-manager | FB44 | BC-2.16.012 v1.16→v1.17 (F-LP56-HIGH-001 EC-016-012-005 production call-site designation; ADR-026 D7 pin v1.10→v1.15 sibling-sweep)

**v4.98 (2026-05-16):** state-manager | FB43 D-663: F-LP54-HIGH-001 sibling-sweep — BC-2.16.002 row bumped v1.22→v1.23 reflecting Fork B canonical rule clarification (POL-30 established FB42 D-662). v4.97 row's "synced with frontmatter v1.21; 9th POL-23 catalog-bullet-label sub-class manifestation" framing was Fork-A-aligned; Fork B canonical rule recognizes bullet-version-label tracks catalog-content-version INDEPENDENT of BC frontmatter. Under Fork B, FB41 fix remains correct (bullet (v1.21) reflects catalog state after FB37 row 33 addition); the "9-recurrence catalog-bullet sub-class" is retrospectively closed as not-a-real-defect-class under Fork B. v4.97 row preserved per POL-26 immutability; v4.98 corrective row supersedes.

**v4.97 (2026-05-16):** state-manager | FB41 D-661: BC-2.16.002 row v1.21→v1.22 (F-LP52-HIGH-001 in-body §Postconditions Canonical Structured Event Catalog bullet header `(v1.20)` → `(v1.21)` synced with frontmatter v1.21; 9th POL-23 catalog-bullet-label sub-class manifestation; PO single-line fix; BC-2.16.002 v1.22).

**v4.96 (2026-05-16):** state-manager | FB37 D-656: BC-2.16.002 row v1.20→v1.21 (F-LP47-HIGH-001 AtomicBool set-time semantic temporal contradiction closed; BC-2.16.002 row 33 corrected); BC-2.16.012 row v1.15→v1.16 (F-LP47-HIGH-001 EC-016-012-005 AtomicBool set-time corrected + F-LP47-MED-002 §Architecture Anchors expanded with ADR-026 §D7 + ADR-027 §D5); POL-23 BC-2.16.002 v1.20→v1.21 cascade propagated across all citing artifacts (story 3 sites + BC-2.16.012 2 sites + error-taxonomy 2 sites = 7 live-narrative sites updated).

**v4.95 (2026-05-16):** state-manager | FB34 D-653: BC-2.01.016 row bumped v1.6→v1.7 (within-FB sibling-sweep: EC-016-003 "impl block is unchanged" corrected to explicit "ONE new method body per ADR-026 §D2 Path B"; resolves internal §Postconditions/AC-2/INV-AUTH-OPEN-002 contradiction).

**v4.94 (2026-05-16):** state-manager | D-649 FB31 single-commit closure (F-LP40-MED-001): BC-2.01.016 row bumped v1.5→v1.6 (fabricated CAP-001 quoted-attribution at §Traceability "Capability Anchor Justification" corrected to verbatim title `"Sensor Adapter Layer (Internal)"` per POL-22 Phase A + POL-7; 39-pass-surviving defect surfaced by lateral attack vector; PO stage complete).

**v4.93 (2026-05-16):** state-manager | D-629 PREREQ-E pass-22+FB20 closure (F-LP22-MED-001): BC-2.01.016 `modified:` field follow-up sync 2026-05-15 → 2026-05-16 (POL-27 cross-check with v1.5 §Changelog row); FB19 within-burst sibling-sweep asymmetry at `modified:` field closed (BC-2.16.011 was correctly synced FB19; BC-2.01.016 was missed; now both consistent).

**v4.92 (2026-05-16):** state-manager | D-628 PREREQ-E fix-burst-19 closure (F-LP21-HIGH-001): BC-2.01.016 row v1.3→v1.5 + BC-2.16.011 row v1.4→v1.6 (D-611-equivalent renumber-repair-redo applied to 2 sibling BCs missed in FB14 D-611 BC-2.16.012 closure; POL-26 monotonic-ordering violations pre-existing FB1 now resolved across all 3 PREREQ-E NEW BCs).

**v4.91 (2026-05-16):** state-manager | D-621 PREREQ-E fix-burst-17 closure (F-LP18-HIGH-001 9TH MANIFESTATION BC-2.16.002 citation defect family at NEW close-paren placement sub-dimension): BC-2.16.012 row v1.14→v1.15 (PO D-620 close-paren placement fix at EC-016-012-005 line 109 + COMPREHENSIVE 5-sub-dimension workspace POL-25 sweep verifying ALL PASS).

**v4.90 (2026-05-16):** state-manager | D-611 PREREQ-E fix-burst-14 closure: BC-2.16.002 row v1.19→v1.20 (D-610 PO bullet label sync F-LP15-HIGH-001 + D-611 state-manager BC-2.16.002 internal §Postconditions Canonical Structured Event Catalog bullet label `(v1.19)`→`(v1.20)` sync — actual F-LP15-HIGH-001 closure completing the work D-610 initiated); BC-2.16.012 row v1.11→v1.14 (D-610 PO v1.11→v1.12 sibling-sweep on BC-2.16.002 v1.20 + D-611 state-manager §Changelog renumber-repair-redo F-LP15-MED-001: state-manager catch row v1.2→v1.3 + cascade shift v1.3→v1.4 through v1.12→v1.13 + new v1.14 repair row; frontmatter v1.12→v1.14 because v1.13 consumed by shifted PO row).

**v4.89 (2026-05-16):** state-manager | D-608 PREREQ-E fix-burst-13 closure (F-LP14-HIGH-001): BC-2.16.012 row v1.10→v1.11 (architect D-607 §Verification Properties VP-156 row sibling-sweep pin v1.9→v1.10; 5th RECURRENCE of POL-23 within-FB sibling-sweep asymmetry; single-bump discipline applied — ADR-026 stays v1.10).

**v4.88 (2026-05-16):** state-manager | D-605 PREREQ-E fix-burst-12 closure (F-LP13-HIGH-001/002/003): BC-2.16.002 row v1.18→v1.19 (architect D-603 Option A row 33 source spec clarification: plugin_name sourced from entry.plugin_name = PluginRuntime-set manifest name); BC-2.16.012 row v1.9→v1.10 (PO D-604 POL-21 sweep 3 sites + Option A §Postconditions field-source citation).

**v4.87 (2026-05-16):** state-manager | D-601 PREREQ-E fix-burst-11 closure (F-LP12-MED-001): BC-2.16.002 row v1.17→v1.18 (Canonical Structured Event Catalog new row 33 `write_tool_registration_after_boot` WARN); BC-2.16.012 row v1.8→v1.9 (§Postconditions cross-reference to BC-2.16.002 v1.18 catalog + EC-016-012-005 explicit event name). PG-LP11-001 discipline enforcement; cycle scope expanded to BC-2.16.002 per Canonical Principle Rule 4.

**v4.86 (2026-05-16):** state-manager | D-595 PREREQ-E fix-burst-9 closure (F-LP10-LOW-001 production-grade Intent B): BC-2.01.016 row sibling-sweep — added `v1.3` trailing version cell to match BC-2.16.011 (v1.4) + BC-2.16.012 (v1.8) row format. Three PREREQ-E NEW BCs created in same burst (v4.82, D-574) now consistent. Sibling-asymmetry was a sibling-sweep gap surfaced by pass-10 fresh-context review per BC-5.39.001 3-CLEAN protocol. BC-INDEX v4.85→v4.86.

**v4.85 (2026-05-16):** architect | prereq-e-fix-burst-8 — BC-2.16.012 v1.7→v1.8: §Verification Properties VP-156 row pin advanced ADR-026 D7 v1.8 → v1.9 (F-LP8-HIGH-002 within-FB7 sibling-sweep asymmetry final close). BC-INDEX BC-2.16.012 row tag added v1.8. BC-INDEX v4.84→v4.85.

**v4.84 (2026-05-16):** state-manager | D-588 — BC-2.16.011 v1.3→v1.4 (PO fix-burst-7 F-LP7-HIGH-002 + F-LP7-MED-004: §Postconditions removal_reason advanced + all 4 BC-2.16.004 frontmatter mutations enumerated; §Architecture Anchors VP-154 anchor corrected ADR-027 §D5 → §Verification Property Anchors). BC-INDEX BC-2.16.011 row tag updated v1.3→v1.4. BC-INDEX v4.83→v4.84. F-LP7-MED-001 modified-date drift: BC-2.16.011 + BC-2.16.012 `modified:` updated 2026-05-15 → 2026-05-16 (POL-27 sibling-sweep close).

**v4.83 (2026-05-16):** architect | prereq-e-fix-burst-6 — BC-2.16.011 v1.2→v1.3: EC-016-011-005 `deprecated_by` adjudicated from ADR-023 → ADR-027 (ADR-027 §Decision is the operational deletion mandate; ADR-023 Rule 5 is the deprecation philosophy ADR-027 operationalizes); `removed: "<PREREQ-E merge date>"`, `removal_reason: "PREREQ-E retirement per ADR-027 §Decision + ADR-023 Rule 5"`, `lifecycle_status: deprecated → removed` added per F-LP6-MED-004 architect adjudication. BC-2.16.012 v1.6→v1.7: §Verification Properties VP-156 row ADR-026 D7 pin updated v1.7→v1.8 per POL-23 sibling sweep (ADR-026 bumped to v1.8 in this burst). BC-INDEX v4.82→v4.83.

**v4.82 (2026-05-15):** state-manager | D-574 — 3 new draft BCs registered (BC-2.01.016 + BC-2.16.011 + BC-2.16.012) for S-PLUGIN-PREREQ-E spec draft package. draft_contracts 2→5; total_contracts 236→239. Subsystem / CAP confirmed from BC file frontmatter: BC-2.01.016 (SS-01/CAP-001/P0), BC-2.16.011 (SS-16/CAP-029/P0), BC-2.16.012 (SS-16/CAP-029/P0). Prose H1 description + total_contracts Note updated. active_contracts: 225 unchanged.

**v4.81 (2026-05-15):** product-owner | D-572 OBS-LP36-002 closure — workspace enumeration reconciled prose-vs-frontmatter count drift; D-571 cycle-close. Authoritative counts derived from `lifecycle_status:` frontmatter of all 236 individual BC files. Corrections: `active_contracts` 235→225 (enumeration shows 225 files with `lifecycle_status: active`); `retired_contracts` 3→2 (v4.54 changelog erroneously incremented retired_contracts when BC-2.16.004 was deprecated, not retired — actual retired files: BC-2.12.011 + BC-2.12.012 only); added `draft_contracts: 2` (BC-2.06.011 + BC-2.21.001 have `lifecycle_status: draft` in files despite BC-INDEX table showing active for BC-2.06.011 — file is ground truth); added `deprecated_contracts: 1` (BC-2.16.004). Prose H1 description and total_contracts Note updated to match enumerated truth. Sibling-sweep (TD-VSDD-060): STATE.md + SESSION-HANDOFF.md count references are historical decision-log entries (immutable records); ADR-025:133 "all 235 active BCs" is frozen rationale (architectural record); no live prose sister-sites required update in this burst. SESSION-HANDOFF.md `active_contracts: 235` at line 182 is state-manager domain — flagged for state-manager update in D-572 burst. total_contracts=236 unchanged.
  - Additionally: BC-2.06.011 table-row status corrected from `active` to `draft` to match source file frontmatter (`lifecycle_status: draft` confirmed in BC-2.06.011-config-load-on-startup.md v1.4). Table-vs-file consistency gap surfaced during OBS-LP36-002 enumeration; bundled into D-572 burst per Canonical Principle Rule 4 (AI-built defect fixed in-scope). TD-VSDD-060 sibling-sweep: all other `.factory/` references to BC-2.06.011 + "active" are historical decision-log entries (SESSION-HANDOFF D-319, STORY-INDEX v2.31, ADR-025 analysis table) — immutable records of the D-319 promotion event, not live status assertions. No additional live prose corrections required.

**v4.80 (2026-05-15):** state-manager | D-568 POL-14 BC promotions for S-PLUGIN-PREREQ-D merge (PR #149, squash ec90fe8f, 2026-05-15T19:08:45Z): BC-2.17.001 v1.3→v1.4 (draft→active); BC-2.17.002 v1.7→v1.8 (draft→active); BC-2.17.003 v1.4→v1.5 (draft→active); BC-2.17.004 v1.4→v1.5 (draft→active); BC-2.17.006 v1.4→v1.5 (draft→active); BC-2.17.007 v1.4→v1.5 (draft→active). 6 BCs promoted draft→active. active_contracts 229→235. total_contracts=236 unchanged.

**v4.79 (2026-05-14):** implementer | F-PASS3-HIGH-001 closure: BC-2.16.002 v1.16→v1.17 — add `plugin_log_level_unrecognized` catalog row (row 32; emitted by host::log callback on unrecognized WIT enum log-level name; fields: plugin_id, received_name; audit role: operational observability; forward-compat safe-default to Info after emission). Catalog intro updated v1.16→v1.17; count 31→32. BC-INDEX row annotation v1.16→v1.17. Closes F-PASS3-HIGH-001 (SOUL.md #4 observability; PG-LP11-001 SOP). | D-TBD
**v4.78 (2026-05-14):** implementer | F-PASS2-HIGH-001 closure: BC-2.16.002 v1.15→v1.16 (prose intro catalog version label and count corrected: v1.12→v1.16, 25→31 events — TD-VSDD-060 sibling-sweep gap from 3 prior amendments missed the intro line). BC-INDEX `timestamp:` Z suffix added (POL-20 ISO-8601 compliance, MED-004 closure). BC-INDEX row v1.15→v1.16 annotation sync. | D-549
**v4.77 (2026-05-14):** implementer | impl-pass-1 fix-burst — BC-2.16.002 v1.13→v1.15 (MED-001: add `message` field to `plugin_load_failed_compilation` row; MED-002: rename `plugin_id`→`sensor_id` in `pipeline_max_requests_exceeded` row + emission site; HIGH-003/005/006: add 3 new catalog rows for E-PLUGIN-017/018/019 new error variants; catalog total 28→31). BC-INDEX row v1.13→v1.15 annotation sync. | D-TBD
**v4.76 (2026-05-14):** state-manager | BC-2.16.002 v1.12→v1.13 (fix-burst-37: F-LP40-MED-001 frontmatter sync — `modified` updated null→2026-05-14; `timestamp` updated 2026-04-13T12:00:00→2026-05-14T00:00:00Z; sibling-sweep gap from F-LP36-MED-001 / OBS-LP36-001 fix-burst-34 not propagated to BC-2.16.002 despite 12 prior amendments; pure metadata sync, no body change) | D-541

**v4.75 (2026-05-14):** state-manager | BC-2.17.007 v1.3→v1.4 (fix-burst-34: F-LP36-MED-001 frontmatter modified+timestamp sync to 2026-05-14 + F-LP36-LOW-001 VP-PLUGIN-007 description line 138+161 sibling-catch rewrite from "per AC-7 default-deny" to "per AC-5 manifest gate; default-deny consumer is AC-7" — canonical anchor restoration per BC §Story Anchor line 157) | D-537

**v4.74 (2026-05-14):** state-manager | BC-2.17.007 v1.2→v1.3 (fix-burst-33: VP-PLUGIN-007 description sweep — lines 138+161 from pre-AC-7 "allowed_urls = None"/"allowlist not-None" Option-semantics to post-AC-7 "explicit Vec<String>"/"explicit list under AC-7 default-deny" framing; sibling-doc propagation gap from F-LP34-LOW-001 D-533 closure); error-taxonomy.md v1.21→v1.22 same-burst per POL-9 (line 464 §Canonical Structured Event Catalog → §Postconditions (Canonical Structured Event Catalog bullet, v1.12)) | D-535

**v4.73 (2026-05-14):** fix-burst-30 stage-1 product-owner — BC-2.17.002 v1.6→v1.7: EC-17-007 phantom variant removal (F-LP32-CRIT-001 closure, Path A: existing-semantics-alignment). Fabricated `PluginError::AllowlistRejected` introduced in v1.6 fix-burst-29 removed — variant does not exist in crates/prism-core/src/error.rs PluginError enum (8 real variants: Trapped/Timeout/MemoryExceeded/NotLoaded/InvalidInterface/SandboxViolation/CompilationFailed/EmptyPluginId), not in error-taxonomy.md, not in story §Error Taxonomy Additions, not in AC-7 prescription. Replaced with existing E-PLUGIN-005 SandboxViolation semantics: `host_http_request` returns `HttpResponse { status: 403, ... }` synchronously per AC-7 prescription and host_functions.rs:64-68. Audit-log mechanism documented via `tracing::warn!(event_type = "plugin_http_request_blocked", ...)`. Path A: zero new error variant, zero signature change, zero new scope. BC-INDEX row v1.6→v1.7. total_contracts=236 unchanged; active_contracts=229 unchanged.

**v4.72 (2026-05-14):** fix-burst-29 stage-1 product-owner — BC-2.17.002 v1.5→v1.6: EC-17-007 default-deny alignment (F-LP31-HIGH-002). Pre-AC-7 "Request allowed to any URL (open by default)" semantics replaced with post-AC-7 default-deny semantics: "Request denied; `PluginError::AllowlistRejected` returned; audit log entry created; empty allowlist → no host matches → deny" per S-PLUGIN-PREREQ-D AC-7 + AC-17 `Vec<String>` field-type contract. BC-INDEX row v1.5→v1.6. total_contracts=236 unchanged; active_contracts=229 unchanged.

**v4.71 (2026-05-13):** state(D-498/D-499) fix-burst-17 stage-3 — BC-INDEX frontmatter v4.70→v4.71. BC-2.16.002 v1.11→v1.12 (product-owner stage 1A @ 84f58565): 2 new structured event catalog rows added — `plugin_load_failed_manifest_name_missing` (E-PLUGIN-015; WARN level; audit role: plugin load failure audit; recurrence: once per manifest name validation failure) + `plugin_load_failed_manifest_version_malformed` (E-PLUGIN-016; WARN level; audit role: plugin load failure audit; recurrence: once per manifest version validation failure). Catalog total 23→25 rows. Closes F-LP18-MED-001 BC portion. Story-writer stage 1B (4b28d5d6) closes F-LP18-MED-001 story portion + F-LP18-LOW-001 + F-LP18-LOW-002 (STORY-INDEX v2.83→v2.84 in parallel). F-LP18-OBS-001 reinforces existing process-gap codification candidate 5 (4th lexical-vs-semantic-sweep recurrence); no new deferral. 9th consecutive single-commit-with-TBD-pin (F-LP10-OBS-001 decisively stable). total_contracts=236 unchanged; active_contracts=229 unchanged.

**v4.70 (2026-05-13):** state(D-480/D-481) fix-burst-8 stage-3 — BC-INDEX frontmatter v4.69→v4.70. fix-burst-8 closed both pass-9 actionable findings in-scope (zero MVP-deferrals per CLAUDE.md Canonical Principle Rule 3). PO stage 1 (4ed96e06): BC-2.16.002 v1.10→v1.11 Path B adjudication — scope broadened from "PipelineExecutor and pipeline.rs helpers" to canonical universal catalog (all `prism-spec-engine` + `prism-bin` boot-step plugin-load emissions); catalog header renamed "Canonical Structured Event Catalog (v1.11)"; 7 new rows: plugin_load_unsigned, plugin_load_disabled_via_envvar, plugin_load_failed_manifest_no_allowed_urls, plugin_load_failed_format_version_exceeded, plugin_load_failed_wit_invalid, plugin_http_request_blocked, pipeline_max_requests_exceeded; total 16→23 rows. Story-writer stage 2 (0f126bbe): story v1.7→v1.8 Catalog Additions preamble Path B sync + 5 metadata corrections + F-LP9-LOW-001 AC-9 line 373 Form A fix. F-LP9-OBS-001 [process-gap] routed to cycle-closing checklist (codification candidate). total_contracts=236 unchanged; active_contracts=229 unchanged.

**v4.69 (2026-05-13):** state(D-478/D-479) fix-burst-7 stage-3 — (1) 6 plugin BCs lifecycle_status Path B sweep (BC-2.17.001/002/003/004/006/007): `lifecycle_status: active` → `lifecycle_status: draft` per BC-INDEX draft status and absence of POL-14 merge event (stale value from Wave-6-pre-build-sweep at v1.1; corrected fix-burst-7 stage 1A by product-owner); versions bumped: BC-2.17.001 v1.2→v1.3, BC-2.17.002 v1.4→v1.5, BC-2.17.003 v1.3→v1.4, BC-2.17.004 v1.3→v1.4, BC-2.17.006 v1.3→v1.4, BC-2.17.007 v1.1→v1.2; (2) BC-2.22.001 v1.4→v1.5: `plugin_load_unsigned` level adjudicated Option A (WARN canonical tracing level + orthogonal audit-channel routing via `event_type` field; clarifying sentence added to §Postconditions plugin-load happy-path block to remove WARN/AUDIT ambiguity for implementer). Index rows updated to reflect new versions. total_contracts=236 unchanged; active_contracts=229 unchanged (6 plugin BCs remain draft; BC-2.22.001 remains active).

**v4.69 (2026-05-13):** S-PLUGIN-PREREQ-D-fix-burst-8-stage-1 — BC-2.16.002 v1.10→v1.11 (F-LP9-MEDIUM-001 closure, Path B). Scope broadened from "PipelineExecutor and helpers / pipeline.rs" to canonical universal catalog covering all prism-spec-engine + prism-bin plugin-load event_type sites. Catalog header renamed "Canonical Structured Event Catalog (v1.11)". +7 new rows: plugin_load_unsigned, plugin_load_disabled_via_envvar, plugin_load_failed_manifest_no_allowed_urls, plugin_load_failed_format_version_exceeded, plugin_load_failed_wit_invalid, plugin_http_request_blocked, pipeline_max_requests_exceeded. Total catalog rows 16→23. BC-2.22.001 unchanged (delegation "per BC-2.16.002" correct under broadened scope). BC-INDEX row v1.10→v1.11 annotation. total_contracts=236 unchanged; active_contracts=229 unchanged.

**v4.68 (2026-05-13):** state(D-476/D-477) fix-burst-6 stage-3 — (1) BC-2.22.001 v1.3→v1.4 (fix-burst-6 stage-1 by product-owner: plugin-load step 7.5 added to §Sequencing Invariant; new postconditions for happy-path / PRISM_DISABLE_PLUGIN_LOAD escape valve / manifest n-1 survivor / fatal exit(4); §Pre-Traffic Gate Invariant condition 6 added; §Exit-Code Map updated; cross-refs to ADR-023 §C4 + BC-2.17.007 added); (2) BC-2.22.001 lifecycle_status adjudicated **Path A** (promoted draft→active per D-319 S-WAVE5-PREP-01 merge at develop@53b87961 2026-05-10; BC file frontmatter `status: draft` + `lifecycle_status: draft` were stale sibling-sweep gap from ADR-025 sweep at v4.62 — corrected to `status: active` + `lifecycle_status: active`); (3) BC-2.17.002 v1.3→v1.4 (fix-burst-6 stage-1 by product-owner: E-PLUGIN-005 timeout corrected 10s → 30s per ADR-023 §C4). Row updates: BC-2.22.001 row status draft→active; BC-2.17.002 row annotated v1.4. total_contracts=236 unchanged; active_contracts=229 unchanged (BC-2.22.001 was already counted active in BC-INDEX since v4.51 D-319; this corrects the BC file frontmatter to match the index).

**v4.67 (2026-05-13):** state(D-469/470/471) — POL-20 actual 100% workspace sweep: 8 BCs with compound-suffix or opaque burst-ID `introduced:` fields migrated to canonical format. (1) BC-2.20.001/002/003/004/005 v1.3→v1.4: `introduced: cycle-1-pass-80` → `introduced: cycle-1` (drop compound pass suffix; pass-80 is pass-metadata not a cycle boundary); input-hash 335606b→3a0a478. (2) BC-2.06.011 v1.3→v1.4 + BC-2.21.001 v1.2→v1.3 + BC-2.22.001 v1.2→v1.3: `introduced: "bundle-B-phase-B-1b-ss22-bcs-2026-05-08"` / `"redirect-option-d-2026-05-08"` → `introduced: "2026-05-08"` (extract embedded ISO date; opaque burst-ID prohibited). input-hash d852024 unchanged (inputs not modified). Closes F-LP4-MED-001. total_contracts=236 unchanged; active_contracts=229 unchanged. POL-20 workspace compliance: 100% (anchored regex `^(cycle-[0-9]+|[0-9]{4}-[0-9]{2}-[0-9]{2})$` returns zero violations).

**v4.66 (2026-05-13):** D-468 POL-20 sweep 100% + TD-VSDD-091 cleanup — 8 BC violations unblocked: BC-3.2.001/002/003/004 + BC-3.3.002 + BC-3.3.004 + BC-3.4.001 + BC-3.4.004 had pre-existing TD-031 line-number anchors blocking Edit-tool POL-20 fix at D-466. TD-VSDD-091 anti-volatile-pin cleanup (line-number → symbol-name) bundled with POL-20 migration (wave-3/v3.0.0 → cycle-3) via Write tool. POL-20 workspace compliance: 100% per prior unanchored grep (NOTE: anchored verification at D-469 found 8 additional violations — see v4.67). total_contracts=236 unchanged; active_contracts=229 unchanged.

**v4.65 (2026-05-13):** D-466/D-467 POL-20 workspace sweep (16 of 24 violations) — 16 BC violations canonicalized: 14 (wave-3/v3.0.0 cluster) → cycle-3; BC-2.03.013 (v1.0.0-greenfield) → cycle-1; BC-2.05.012 (bundle-B-phase-B-1b) → cycle-3. 8 blocked by pre-existing TD-031 violations (validate-stable-anchors). New TD filed. total_contracts=236 unchanged; active_contracts=229 unchanged.

**v4.64 (2026-05-13):** wave-4-fix-burst-F-LP1-HIGH-004 — New BC-2.17.007 "Plugin Manifest Schema Validation Before WIT Validation" (CAP-032, SS-17, P0, draft). Closes F-LP1-HIGH-004 from S-PLUGIN-PREREQ-D pass-1. Authors E-PLUGIN-013/014/015/016 in error-taxonomy.md v1.19. total_contracts=235→236; active_contracts=228→229.

**v4.63 (2026-05-13):** D-457 ADR-025 BC-2.03.013 sweep — removed `lifecycle: active` per ADR-025; added template frontmatter fields (lifecycle_status, introduced, modified, deprecated, deprecated_by, replacement, retired, removed, removal_reason, extracted_from); fixed duplicate v1.0 changelog rows (second renamed v1.0.1); BC-INDEX title synced to H1 source of truth ("Reference Validation Only, No Values in Memory at Process Start"). BC-2.03.013 v1.1→v1.2. total_contracts=235 unchanged; active_contracts=228 unchanged.

**v4.62 (2026-05-12):** D-454 ADR-025 BC lifecycle sweep — BC-2.22.001 v1.1→v1.2 (status:accepted→draft, lifecycle:active removed, template fields added, changelog reordered newest-first, BC-INDEX title synced to H1 adding "and"); BC-2.06.011 v1.2→v1.3 (lifecycle:active removed, template fields added, input-hash computed); BC-2.21.001 v1.1→v1.2 (lifecycle:active removed, template fields added, input-hash computed). Per ADR-025: `status:` is sole canonical lifecycle field; `lifecycle:` field retired. total_contracts=235 unchanged; active_contracts=228 unchanged.

**v4.61 (2026-05-12):** S-PLUGIN-PREREQ-C-fix-burst-1 — BC-2.16.002 v1.9 → v1.10 (PREREQ-C fix-burst-1 catalog amendment: +2 event_type rows for jsonpath_extraction_failed and jsonpath_size_cap_exceeded). total_contracts=235 unchanged; active_contracts=228 unchanged.

**v4.60 (2026-05-12):** D-427 S-PLUGIN-PREREQ-B PR #143 merge — BC-2.16.002 v1.8→v1.9 status promoted draft→active per POL-14 (anchor story S-PLUGIN-PREREQ-B merged at develop@ae7e26c8 2026-05-12T06:58:48Z). active_contracts 227→228. total_contracts=235 unchanged.

**v4.59 (2026-05-11):** D-419 S-PLUGIN-PREREQ-B fix-burst-11 — BC-2.16.002 v1.7→v1.8 amendment (F-LP11-MED-001 BC event-catalog drift + F-LP11-MED-002 field-schema drift). New "Structured Event Catalog (v1.8)" postcondition with 14-row table enumerating ALL event_type values emitted by PipelineExecutor: auth_initial_acquired/_empty/_failed for both execute() (no step_name) and execute_step() (with step_name field); auth_refresh_triggered/succeeded/failed/double_401 × issue_request_with_retry; pipeline_truncated; pagination_cursor_unsupported_type; fanout_invalid_source_type; fanout_ambiguous_multi_array. Product-owner field-name discipline correction: actual tracing macro field is `detail` not `error`. Two superseded audit-signal bullets redirect to catalog (narrative+VP-PLUGIN-005 cites preserved). PG-LP11-001 SOP codified in D-419. Factory commit (this burst). total_contracts=235 unchanged; active_contracts=227 unchanged (BC-2.16.002 remains draft).

**v4.58 (2026-05-11):** D-415 S-PLUGIN-PREREQ-B fix-burst-9 — BC-2.16.002 v1.6→v1.7 amendment (F-LP9-MED-001 audit-signal enumeration). Postcondition "Auth initial acquisition audit signal" rewritten to enumerate THREE events: (1) non-empty token Ok → tracing::info! auth_initial_acquired; (2) empty token Ok → tracing::debug! auth_initial_acquired_empty; (3) Err → tracing::error! auth_initial_failed. Factory commit (this burst). total_contracts=235 unchanged; active_contracts=227 unchanged (BC-2.16.002 remains draft).

**v4.57 (2026-05-11):** D-411 S-PLUGIN-PREREQ-B fix-burst-7 — BC-2.16.002 v1.5→v1.6 amendment (F-LP7-MED-003 partial-record discard). New postcondition: "On mid-pipeline failure, execute returns Err; records accumulated from prior steps are discarded (all-or-nothing semantics)." Factory commit d11dbf0d. total_contracts=235 unchanged; active_contracts=227 unchanged (BC-2.16.002 remains draft).

**v4.56 (2026-05-11):** D-408 S-PLUGIN-PREREQ-B fix-burst-5 — BC-2.16.002 v1.4→v1.5 amendment. Precondition lifecycle changed lazy→eager: token acquired unconditionally at pipeline start (AuthType has no Null variant). New postconditions: (1) request_count counts HTTP pipeline requests only, excluding acquire_token transport; (2) auth_initial_acquired (info) + auth_initial_failed (error) audit events added; (3) auth_refresh_* family fully enumerated (triggered/succeeded/failed/double_401). Factory commit 82fd868c. total_contracts=235 unchanged; active_contracts=227 unchanged (BC-2.16.002 remains draft).

**v4.55 (2026-05-11):** D-398 post-merge POL-14 promotion — BC-2.01.013 status draft→active (anchor story S-PLUGIN-PREREQ-A merged via PR #142 at develop@90d7c80f, 2026-05-11T16:37:14Z). active_contracts 226→227. total_contracts=235 unchanged.

**v4.54 (2026-05-11):** PREREQ-F — ADR-023 v1.17 BC catalog amendments. (1) BC-2.16.004 lifecycle_status active→deprecated (deprecated_by: ADR-023); index row strikethrough + status updated to "deprecated (ADR-023 PREREQ-F)"; active_contracts 227→226, retired_contracts 2→3. (2) BC-2.01.013 amendment_lifecycle: pending — ADR-023 v1.4 amendment removes sealed-trait language, adds spec-driven runtime validation rules; index status updated. (3) Eight sensor-named BCs (BC-2.01.005/006/007/008, BC-2.02.003/004/005/006) amendment_lifecycle: pending — ADR-023 prefix notes added; index status updated. VP-PLUGIN-001..007 registered in VP-INDEX.md. TS-PLUGIN-PARITY-001 authored.

**v4.53 (2026-05-09):** PR-139-pr-level-pass-3-fix-pass — Sync BC-2.06.011 row title to H1 per POL-7 — closes F-P3-MED-1 from PR #139 PR-LEVEL adversary pass-3 (POL-7 violation, pre-existing drift surfaced during v4.52 amendment). Title updated from "ConfigManager Initialization Validation — Config Loaded and Validated Before Serving" to "ConfigManager Initialization — prism.toml Schema Validation at Process Start". No count changes (total_contracts=235 unchanged).

**v4.52 (2026-05-09):** PR-139-pr-level-pass-2-fix-pass — BC-2.06.011 v1.1→v1.2: replaced obsolete `~/.prism/` default-path references with platform-aware `dirs::config_dir().join("prism")` resolution (F-P2-MED-1 closure). Three sites updated: Description, Preconditions, and TV-06-011-001 test vector. No count changes (total_contracts=235 unchanged).

**v4.51 (2026-05-10):** D-319 S-WAVE5-PREP-01 chassis SHIPPED — 5 BCs promoted draft→active per ADR-021 POL-14: BC-2.06.011 v1.0→v1.1 (ConfigManager init, SS-06), BC-2.21.001 v1.0→v1.1 (OrgRegistry init — first BC under SS-21), BC-2.03.013 v1.0→v1.1 (CredentialStore init), BC-2.05.012 v1.2→v1.3 (BootAuditEmitter audit init), BC-2.22.001 v1.0→v1.1 (boot orchestration — first BC under SS-22). active_contracts 222→227. No changes to total_contracts (235 unchanged).

**v4.50 (2026-05-09):** D-315 S-WAVE5-PREP-01 fix-pass-4 spec track — BC-2.05.012 v1.1→v1.2 amendment, F-PASS4-LOW-2 closure: §Failure paths and Error Cases table updated to describe RocksDbBackend::open failure (the actually-fallible step) instead of phantom AuditEmitter::new failure (BootAuditEmitter::new is infallible). 4 edits, 247 lines. No count changes (total_contracts=235 unchanged).

**v4.49 (2026-05-09):** D-312 S-WAVE5-PREP-01 fix-pass-3 spec track — BC-2.05.012 v1.0→v1.1 amendment per research-agent recommendation + adversary F-PASS3-MED-1 closure. Description lines 31-32 clarify BootAuditEmitter is the boot-time specialization distinct from request-time AuditEmitterLayer; Postcondition bullets 1+4 reflect the two-phase emitter design; OQ-2 marked resolved. Research artifact: audit-emitter-architecture-2026-05-09.md. No count changes (total_contracts=235 unchanged).

**v4.48 (2026-05-08):** D-307 Bundle B Phase B-1b Option (d) — 5 new BCs registered (lifecycle: draft, all anchored to S-WAVE5-PREP-01): BC-2.03.013 (CredentialStore init, SS-03, CAP-004), BC-2.05.012 (AuditEmitter init, SS-05, CAP-007), BC-2.06.011 (ConfigManager init, SS-06, CAP-009), BC-2.21.001 (OrgRegistry init — first BC under SS-21, CAP-038), BC-2.22.001 (Boot orchestration — first BC under SS-22, CAP-034). total_contracts 230→235; active_contracts unchanged at 222 (5 new BCs are draft; promote to active per ADR-021/POL-14 when S-WAVE5-PREP-01 merges). Count-propagation sweep: updated flat index header line, frontmatter total_contracts.

**v4.47 (2026-05-08):** Bundle A.2.2 — POL-14 BC promotion: 9 BCs promoted draft → active. Anchor stories: S-1.10 (BC-2.09.001..008) + S-3.06 (BC-2.11.004). Per ADR-021 + D-304. Status column updated for all 9 rows in flat index. BC file frontmatter status + version bumped for each.

**v4.42 (2026-05-07):** S-3.04 adversary local pass-2 remediation — BC-2.11.006 v1.16→v1.17: F-LOCAL-P2-CRIT-001 + F-LOCAL-P2-HIGH-005 closure: added 5 S-3.04 alias-system symbols to `restricted_symbols` (`alias_tools::create_alias`, `alias_tools::create_alias_with_clients`, `alias_tools::create_alias_with_clients_gated_inner`, `alias_tools::delete_alias`, `alias_store::AliasStore::create_or_update`). Layer-5 (S-3.04): 0→5 symbols. Total perimeter list: 26→31. Documented `alias-write` Cargo feature as runtime-advisory gate (F-LOCAL-P2-HIGH-004, option b). Updated lib.rs perimeter docstring with layer-5 block.

**v4.41 (2026-05-07):** S-3.03 adversary local pass-4 remediation — BC-2.11.010 v1.5→v1.6: (C-2) Canonical test vector corrected — "push-down shown as FQL" replaced with "push-down shown as PrismQL-native predicate strings (sensor-native translation deferred — see INV-PUSH-001 / TD-S303-PUSH-DOWN-TRANSLATION-001)". (I-2) DI-PUSH-001 renamed to INV-PUSH-001 (BC-local prefix) across all occurrences to avoid orphan-DI detector false positives. (I-3) v1.5 changelog note added acknowledging incomplete softening propagation.

**v4.40 (2026-05-07):** S-3.03 adversary local pass-3 remediation — BC-2.11.010 v1.4→v1.5: (I-LOCAL-PASS3-2) restructured `execution_plan` postcondition tree — `api_filters_pushed` moved from sibling-of-`sensors_to_query` to nested inside each `ExplainSource` entry, matching actual struct layout; added `source_ref`, `sensor_type`, `post_filter_predicates`, `estimated_row_count` sub-fields. (I-LOCAL-PASS3-3) softened "sensor-native syntax" claim in Description and postcondition to "predicate strings (sensor-native translation deferred via TD-S303-PUSH-DOWN-TRANSLATION-001)"; added DI-PUSH-001 invariant explicitly documenting the deferral. TD-S303-PUSH-DOWN-TRANSLATION-001 filed.

**v4.39 (2026-05-07):** S-3.03 adversary local pass-2 remediation — BC-2.11.010 v1.3→v1.4: added `clients_to_query: Vec<OrgSlug>` to ExecutionPlan postcondition (C-LOCAL-002 / AC-5); added Preconditions section documenting I-LOCAL-002 sensor-scope filter (external-only; intentional); added DataFusion plan elision invariant (I-LOCAL-001 / TD-S303-DATAFUSION-PLAN-001). Set `modified: 2026-05-07` in frontmatter (was null despite v1.4 changelog).

**v4.38 (2026-05-05):** Adversary pass-8 remediation — BC-2.11.006 v1.9→v1.10: F-HIGH-001 — added `ParseLimits::snapshot` to `restricted_symbols` frontmatter (16→17 entries; 14 unique parent paths unchanged — snapshot collapses to `ParseLimits` like the other methods). Symbol was already `pub(crate)` and enumerated in lib.rs perimeter docstring; missing from BC frontmatter caused docstring↔spec drift. Body note expanded to explain snapshot's constructor role and defence-in-depth rationale. PR-127 adversary pass-8 remediation.

**v4.37 (2026-05-05):** Adversary pass-7 remediation — BC-2.11.006 v1.8→v1.9: F-LOW-004 — added 3 `*_with_limits` functions to `restricted_symbols` frontmatter (`parse_filter_with_limits`, `parse_sql_with_limits`, `parse_pipe_with_limits`; 13→16 entries, 11→14 unique parent paths after normalize_to_use_path). Body note added explaining de-facto-private rationale (`ParseLimits` fields are `pub(crate)`) and future-proofing intent. PR-127 adversary pass-7 remediation.

**v4.36 (2026-05-05):** Adversary pass-6 remediation — BC-2.11.006 v1.7→v1.8: F-MEDIUM-001 — added 4th enforcement layer (CI gate `perimeter-compile-fail` in `.github/workflows/ci.yml`, now implemented; drops stale "devops dispatch in flight" wording). F-LOW-001 — footnote added distinguishing private `build_filter_parser`/`build_sql_parser` (fn-private, inaccessible regardless of perimeter) from the seven pub(crate) builder factories. OBS-001 part — added structured `restricted_symbols:` frontmatter block (13 entries: 3 parse_* entry points, 7 build_*_parser factories, 3 thread-local API symbols) as machine-checkable source-of-truth for perimeter validation against `tests/external/perimeter-violation/src/main.rs`. PR-127 adversary pass-6 remediation.

**v4.35 (2026-05-05):** Adversary pass-5 remediation — BC-2.11.006 v1.6→v1.7: F-MEDIUM-001 corrected inaccurate clippy.toml enforcement claim (per-crate scope only; `cargo build` does not run clippy); layered enforcement now accurately describes Rust visibility (primary), clippy intra-crate defence-in-depth, and api_surface integration test. F-MEDIUM-002: INV-SEC-PERIMETER-001 now cross-references DI-034 (prism-query security perimeter domain invariant lifted by business-analyst in parallel); L2 Invariants traceability updated to DI-019, DI-034. PR-127 adversary pass-5 remediation.

**v4.34 (2026-05-05):** Adversary pass-4 OBS-002 remediation — BC-2.11.006 v1.5→v1.6: added Security Perimeter postcondition (pub(crate) enforcement of sub-parsers and builder factories), INV-SEC-PERIMETER-001 invariant, and two compile-failure test vectors for api_surface.rs. Codifies that prism-query exposes only PrismQlParser::parse as its public security boundary; perimeter is enforced via Rust visibility and clippy.toml disallowed-methods lint. Refs PR-127.

**v4.33 (2026-05-05):** PR-127 review remediation — BC-2.11.003 v1.3→v1.4: added canonical Denied SQL Statement Prefixes section (~40 keywords across DML/DDL/TCL/DCL/procedural/utility/vendor categories per SQL:2016); updated Invariants to use E-QUERY-002 for denylist hits; updated Error Cases table; expanded Canonical Test Vectors with 9 new denylist vectors. Addresses Adv OBS-002 [process-gap]. Implementer follow-up required: extend filter_parser.rs denylist from 7 to ~40 keywords.

**v4.32 (2026-05-03):** Wave 4 Phase 4.A Pre-Pass-21 Broad-Scope Sweep — F-PreP21-H-002: BC-2.18.003 v1.3→v1.4 (ActionEngine→ActionDeliveryEngine 1 site; modified→2026-05-03); BC-2.18.008 v1.3→v1.4 (ActionEngine→ActionDeliveryEngine 2 sites; modified→2026-05-03). Sister BCs in SS-18 with stale type name cleaned; D-209 + ADR-016 §1.1 propagated.

**v4.31 (2026-05-03):** Wave 4 Phase 4.A Pass 20 remediation — BC-2.18.001 v1.7→v1.8 (F-P20-L-002: ActionEngine→ActionDeliveryEngine 1 site); BC-2.18.002 v1.4→v1.5 (F-P20-L-002: ActionEngine→ActionDeliveryEngine 2 sites); BC-2.18.004 v1.4→v1.5 (F-P20-L-002: ActionEngine→ActionDeliveryEngine 10 sites).

**v4.30 (2026-05-03):** Wave 4 Phase 4.A Pass 14 remediation — BC-2.12.004 v1.7→v1.8 (F-P14-H-002: frontmatter `modified` 2026-05-04→2026-05-03 corrected; v1.7 changelog row date corrected; v1.6 row date also corrected).

**v4.29 (2026-05-03):** Wave 4 Phase 4.A Pass 13 remediation — BC-2.12.004 v1.6→v1.7 (VP-137 row added to Verification Properties table; closes F-P13-M-003 POL-4 reverse traceability gap).

**v4.28 (2026-05-03):** Wave 4 Phase 4.A Pass 10 remediation — BC-2.18.001 v1.6→v1.7 (line 58 + EC-18-005/a case-trigger action-dispatch analog added; closes F-P10-M-002).

**v4.27 (2026-05-01):** Wave 3 BC status sweep — all 22 BC-3.* files updated PROPOSED → draft post-Wave-3 implementation closure. BC-3.1.001–004, BC-3.2.001–005, BC-3.3.001–004, BC-3.4.001–004, BC-3.5.001–002, BC-3.6.001–002, BC-3.7.001. Closes F-48-M-001 (MEDIUM) + WGCV-W3-006 carry.

**v4.26 (2026-04-27):** M-23-001 (pass-23-remediation): BC Family 3.4 Subsystem column corrected SS-06 (Client Configuration) → SS-01 (Sensor Adapters) for BC-3.4.001–004. CAP-039 implementation lives in `crates/prism-dtu-common` (SS-01) per D-056, not in Client Configuration (SS-06).

**v4.25 (2026-04-27):** M-20-001 (pass-20-remediation): BC-3.7.001 anchor remains SS-01 per D-060 (v4.23 row claimed an SS-21 change that contradicted D-060 and was never applied; this is documented for clarity). M-20-002 (pass-20-remediation): Wave 3 BC Family 3.6 header corrected from "(ADR-008, ADR-011)" to "(ADR-011)" — BC-3.6.001/002 trace only to ADR-011, not ADR-008.

**v4.24 (2026-04-27):** M-19-004 (pass-19-remediation): Wave 3 section headers relabeled — "Subsystem 3.X" → "Wave 3 BC Family 3.X" for all 7 families (3.1–3.7) to match Summary table relabel from v4.22. Family 3.7 ADR reference corrected from "(ADR-006)" to "(ADR-012)" — workspace src/ convention BCs are scoped by ADR-012, not ADR-006.

**v4.23 (2026-04-27):** M-18-002 (pass-18-remediation): BC-3.2.005 subsystem corrected SS-01 → SS-06 in index table (DTU mode is Client Configuration scope, not Sensor Adapters); BC-3.7.001 subsystem corrected SS-01 → SS-21 in index table (workspace convention lint is Identity & Core Types / prism-core scope per ADR-012 §2.1). BC file frontmatter `subsystem:` fields updated in same pass.

**v4.22 (2026-04-27):** m-003 (pass-10-remediation): Summary table rows 3.1–3.7 relabeled "Wave 3 BC Family: 3.X" to clarify these are BC prefix groupings, not ARCH-INDEX subsystems (SS-01..SS-20). Naive readers previously concluded 27 subsystems existed; label now disambiguates from real SS-NN entries above. Counts and totals unchanged.

**v4.21 (2026-04-27):** M-004 (pass-8-remediation): Title Case sweep for BC-3.1.001–004 and BC-3.2.001–005 — 9 titles updated from sentence-case to Title Case to match BC-3.3.*/BC-3.4.* siblings (POL 7 H1 source-of-truth). BC file frontmatter `title:` and H1 headings updated in same pass. Titles: BC-3.1.001 "OrgRegistry Bijective Slug/UUID Resolution"; BC-3.1.002 "Audit Entry Carries Both org_id and org_slug at Construction Time"; BC-3.1.003 "OrgRegistry Maintains Strict Bijectivity at All Times"; BC-3.1.004 "OrgRegistry Rejects Duplicate Slugs and UUIDs at Registration"; BC-3.2.001 "Per-Org Sensor Data Isolation via Composite HashMap Key"; BC-3.2.002 "Per-Org Credential Isolation via OrgId-Keyed Namespace"; BC-3.2.003 "Per-Org Session Token Isolation via (OrgId, token) Composite Key"; BC-3.2.004 "Shared-Mode DTU Tags OrgId in Payload Body Not in Routing Headers"; BC-3.2.005 "DTU Mode is Deployment-Time Config — No Runtime API to Change It".

**v4.20 (2026-04-27):** m-004 (pass-7-remediation): BC-3.3.001 title corrected to Title Case "Startup Rejects Security Telemetry DTU Type Declared with Shared Mode" (was sentence-case) — POL 7 H1 source-of-truth; siblings BC-3.3.002/003/004 were already Title Case. BC file frontmatter and H1 updated in same pass.

**v4.19 (2026-04-27):** C-003: Summary table updated to include 7 Wave 3 subsystem rows (3.1–3.7) with their 22 BCs. Total row updated from 200 active to 222 active; P0 updated 171→192; P1 updated 29→30 (BC-3.7.001 is P1). Frontmatter active_contracts=222 now matches Summary table total.

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

### Version 4.46 (2026-05-07, S-3.04-fix-pass-30 — BC-2.07.002 v4.8 broken anchor fix)

**BC body update:**
- BC-2.07.002 v4.7→v4.8: Fixed broken anchor in E-STORE-020 Error Cases row — `§Concurrent Fetch Limits (MCP-exposed surface)` → `§Concurrent Fetch Limits` (actual heading has no parenthetical suffix). Resolves F-PASS11-HIGH-001. TD-080 class recurrence #5.

### Version 4.45 (2026-05-07, S-3.04-fix-pass-29 — BC-2.07.002 v4.7 E-STORE-020 Error Cases row)

**BC body update:**
- BC-2.07.002 v4.6→v4.7: Added `Err(PrismError::CursorCapExceeded)` (E-STORE-020) row to Error Cases table. Code was cited in §Cursor Lifecycle (MCP-exposed surface) — Cap but absent from Error Cases table (internal inconsistency). Resolves F-PASS10-MED-001.

### Version 4.44 (2026-05-07, S-3.04-fix-pass-28 — BC-2.07.002 v4.6 §Cursor Lifecycle section added)

**BC body update:**
- BC-2.07.002 v4.5→v4.6: Added `## Cursor Lifecycle (MCP-exposed surface)` section covering TTL (60s), cap (200 cross-client), creation/advancement/expiry/cross-client-allocation semantics. Fixed Note anchor from broken §Cursor Lifecycle to `## Cursor Lifecycle (MCP-exposed surface)`. Resolves F-PASS9-HIGH-001.

### Version 4.43 (2026-05-07, S-3.04-fix-pass-27 — BC-2.07.002 v4.5 Note reconciliation)

**BC body update:**
- BC-2.07.002 v4.4→v4.5: Note rewritten to acknowledge S-3.05 MCP-cursor surface layered on internal pagination machinery. Resolves F-PASS8-CRIT-002 internal contradiction in prior v4.4 Note which stated pagination was "entirely internal" while v4.4 Error Cases already exposed MCP-facing cursor errors.

### Version 4.42 (2026-05-07, S-3.05-fix-pass-16-sub-burst — BC-2.07.002 error code taxonomy update)

**BC body update:**
- BC-2.07.002 v4.3→v4.4: Error Cases table updated with cursor lifecycle error codes per S-3.05 fix-pass-16 renumber (D-272). Added PrismError::CursorExpired (E-QUERY-012), PrismError::CursorPageSizeInvalid (E-QUERY-013), PrismError::CursorTokenUnknown (E-QUERY-014). E-QUERY-014 unknown-token case newly distinguished from E-QUERY-012 expired-token case (pass-8 IMP-004 finding: unknown tokens previously returned E-QUERY-004 misleadingly). Cites F-PASS9-CRIT-001/002/003.
