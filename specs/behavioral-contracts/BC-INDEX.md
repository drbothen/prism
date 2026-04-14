---
document_type: behavioral-contract-index
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T06:00:00
phase: 1a
total_contracts: 102
---

# Behavioral Contract Index

Flat index of all 102 behavioral contracts for Prism, organized by BC ID.

| BC ID | Title | Subsystem | CAP | Priority | Status |
|-------|-------|-----------|-----|----------|--------|
| BC-2.01.001 | Single-Client Sensor Query Returns Scoped Results | 01 - Sensor Query Pipeline | CAP-001 | P0 | draft |
| BC-2.01.002 | Cross-Client Fan-Out Query Aggregates Results with Per-Client Attribution | 01 - Sensor Query Pipeline | CAP-002 | P0 | draft |
| BC-2.01.003 | Cursor-Based Forward-Only Pagination | 01 - Sensor Query Pipeline | CAP-001 | P0 | draft |
| BC-2.01.004 | Offset-Based Hybrid Pagination for Claroty Audit Logs | 01 - Sensor Query Pipeline | CAP-001 | P0 | draft |
| BC-2.01.005 | CrowdStrike OAuth2 Authentication and Two-Step Fetch | 01 - Sensor Query Pipeline | CAP-001 | P0 | draft |
| BC-2.01.006 | Cyberint Cookie-Based Authentication and Multi-Format Timestamp Parsing | 01 - Sensor Query Pipeline | CAP-001 | P0 | draft |
| BC-2.01.007 | Claroty Bearer Token Auth with Polymorphic ID Handling | 01 - Sensor Query Pipeline | CAP-001 | P0 | draft |
| BC-2.01.008 | Armis Bearer Token Auth with AQL Query Forwarding and Timestamp Fallback | 01 - Sensor Query Pipeline | CAP-001 | P0 | draft |
| BC-2.01.009 | Query Filtering and Sorting Parameters | 01 - Sensor Query Pipeline | CAP-001 | P0 | draft |
| BC-2.01.010 | Partial Failure Handling for Paginated and Cross-Client Queries | 01 - Sensor Query Pipeline | CAP-001, CAP-002 | P0 | draft |
| BC-2.01.011 | Cross-Sensor Correlation via OCSF Field Alignment | 01 - Sensor Query Pipeline | CAP-012 | P1 | draft |
| BC-2.01.012 | ~~Query Fingerprint Validation at Startup~~ | 01 - Sensor Query Pipeline | CAP-001 | P0 | removed |
| BC-2.01.013 | DataSource Trait Eliminates Per-Sensor Code Duplication | 01 - Sensor Query Pipeline | CAP-001 | P0 | draft |
| BC-2.01.014 | Exponential Backoff and Retry for Transient Sensor API Errors | 01 - Sensor Query Pipeline | CAP-001 | P0 | draft |
| BC-2.01.015 | MCP Tool Response Envelope Structure | 01 - Sensor Query Pipeline | CAP-001 | P0 | draft |
| BC-2.02.001 | OCSF Schema Loading at Build Time via ocsf-proto-gen | 02 - OCSF Normalization | CAP-003 | P0 | draft |
| BC-2.02.002 | DynamicMessage Creation from Sensor Records | 02 - OCSF Normalization | CAP-003 | P0 | draft |
| BC-2.02.003 | CrowdStrike Alert Field Mapping to OCSF | 02 - OCSF Normalization | CAP-003 | P0 | draft |
| BC-2.02.004 | Cyberint Alert Field Mapping to OCSF | 02 - OCSF Normalization | CAP-003 | P0 | draft |
| BC-2.02.005 | Claroty xDome Field Mapping to OCSF (9 Data Sources) | 02 - OCSF Normalization | CAP-003 | P0 | draft |
| BC-2.02.006 | Armis Centrix Field Mapping to OCSF (7 Data Sources) | 02 - OCSF Normalization | CAP-003 | P0 | draft |
| BC-2.02.007 | Vendor Extension Preservation in raw_extensions | 02 - OCSF Normalization | CAP-003 | P0 | draft |
| BC-2.02.008 | Three-Tier Field Alias Resolution | 02 - OCSF Normalization | CAP-003 | P0 | draft |
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
| BC-2.04.001 | Compile-Time Cargo Features Gate Write Code Families | 04 - Feature Flag System | CAP-005 | P0 | draft |
| BC-2.04.002 | Runtime Per-Client TOML Feature Flag Configuration | 04 - Feature Flag System | CAP-005 | P0 | draft |
| BC-2.04.003 | Hierarchical Capability Resolution (BTreeMap, Most-Specific-Path Wins, Deny Support) | 04 - Feature Flag System | CAP-005 | P0 | draft |
| BC-2.04.004 | Two-Tier Gate -- Both Compile-Time and Runtime Must Permit Operation | 04 - Feature Flag System | CAP-005 | P0 | draft |
| BC-2.04.005 | Hidden Tools Pattern -- Disabled Write Tools Omitted from tools/list | 04 - Feature Flag System | CAP-005 | P0 | draft |
| BC-2.04.006 | list_capabilities Meta-Tool for Capability Discovery | 04 - Feature Flag System | CAP-005 | P0 | draft |
| BC-2.04.007 | Three-Tier Risk Classification for Operations | 04 - Feature Flag System | CAP-006 | P1 | draft |
| BC-2.04.008 | Dry-Run Default for Reversible Write Operations | 04 - Feature Flag System | CAP-006 | P1 | draft |
| BC-2.04.009 | Confirmation Token Generation with 100-Token Active Cap | 04 - Feature Flag System | CAP-006 | P1 | draft |
| BC-2.04.010 | Confirmation Token Consumption via confirm_action | 04 - Feature Flag System | CAP-006 | P1 | draft |
| BC-2.04.011 | Token Expiry at 300 Seconds with Structured Error Recovery | 04 - Feature Flag System | CAP-006 | P1 | draft |
| BC-2.04.012 | Token Content Hash Verification Prevents Action Tampering | 04 - Feature Flag System | CAP-006 | P1 | draft |
| BC-2.04.013 | Feature Flag Evaluation Audit Logging for Write Operations | 04 - Feature Flag System | CAP-005 | P0 | draft |
| BC-2.04.014 | ~~notifications/tools/list_changed on Client Context Switch~~ | 04 - Feature Flag System | CAP-005 | P0 | removed |
| BC-2.04.015 | Structured Error When Write Capability Is Denied | 04 - Feature Flag System | CAP-005 | P0 | draft |
| BC-2.05.001 | Every MCP Tool Invocation Produces Exactly One Audit Entry (Fail-Closed for Writes) | 05 - Audit & Compliance | CAP-007 | P0 | draft |
| BC-2.05.002 | Audit Entries Use Structured JSON Format with Complete Fields | 05 - Audit & Compliance | CAP-007 | P0 | draft |
| BC-2.05.003 | Credential Values Are Never Present in Audit Entries | 05 - Audit & Compliance | CAP-007 | P0 | draft |
| BC-2.05.004 | Write Operations Log Capability Check and Execution Outcome | 05 - Audit & Compliance | CAP-007 | P0 | draft |
| BC-2.05.005 | Credential Access Events Are Audit-Logged with Context | 05 - Audit & Compliance | CAP-007 | P0 | draft |
| BC-2.05.006 | Audit Entries Are Append-Only and Immutable | 05 - Audit & Compliance | CAP-007 | P0 | draft |
| BC-2.05.007 | Audit Entries Are Compatible with the Vector Pipeline | 05 - Audit & Compliance | CAP-007 | P0 | draft |
| BC-2.05.008 | Audit Entries Satisfy SOC 2 Type II and ISO 27001 Requirements | 05 - Audit & Compliance | CAP-007 | P0 | draft |
| BC-2.05.009 | Feature Flag Evaluations for Write Operations Are Audit-Logged | 05 - Audit & Compliance | CAP-007 | P0 | draft |
| BC-2.05.010 | Confirmation Token Lifecycle Events Are Audit-Logged | 05 - Audit & Compliance | CAP-007 | P0 | draft |
| BC-2.06.001 | TOML Configuration Loads and Deserializes at Startup | 06 - Client Configuration | CAP-009 | P0 | draft |
| BC-2.06.002 | Per-Client Sensor Mapping from TOML Configuration | 06 - Client Configuration | CAP-009 | P0 | draft |
| BC-2.06.003 | Credential References in Config Resolve to Credential Store Entries | 06 - Client Configuration | CAP-009 | P0 | draft |
| BC-2.06.004 | Capability Overrides Merge with Defaults Using More-Specific-Wins | 06 - Client Configuration | CAP-009 | P0 | draft |
| BC-2.06.005 | Configuration Validation Reports All Errors in One Pass | 06 - Client Configuration | CAP-009 | P0 | draft |
| BC-2.06.006 | --dry-run Flag Validates Config and Prints Redacted Summary | 06 - Client Configuration | CAP-009 | P0 | draft |
| BC-2.06.007 | Missing Required Fields Produce Actionable Error Messages | 06 - Client Configuration | CAP-009 | P0 | draft |
| BC-2.06.008 | Default Values Apply and Environment Variables Override TOML | 06 - Client Configuration | CAP-009 | P0 | draft |
| BC-2.06.009 | ~~Client Context Switch Triggers notifications/tools/list_changed~~ | 06 - Client Configuration | CAP-009 | P0 | removed |
| BC-2.06.010 | Client ID Validation Enforces Allowed Character Set | 06 - Client Configuration | CAP-009 | P0 | draft |
| BC-2.07.001 | Ephemeral Cursor-Based Pagination (No Persistent State) | 07 - Pagination & Caching | CAP-011 | P0 | draft |
| BC-2.07.002 | Pagination Token Expiry and Cleanup | 07 - Pagination & Caching | CAP-011 | P0 | draft |
| BC-2.07.003 | Response Cache with Configurable TTL | 07 - Pagination & Caching | CAP-014 | P0 | draft |
| BC-2.07.004 | Cache Invalidation on Write Operations | 07 - Pagination & Caching | CAP-014 | P0 | draft |
| BC-2.07.005 | Cache Key Derivation from Query Parameters | 07 - Pagination & Caching | CAP-014 | P0 | draft |
| BC-2.07.006 | Cache Memory Bounds and Eviction Policy | 07 - Pagination & Caching | CAP-014 | P0 | draft |
| BC-2.07.007 | ~~State Is Isolated Per-Client, Per-Sensor, Per-Source~~ | 07 - Pagination & Caching | CAP-011 | P0 | removed |
| BC-2.07.008 | ~~MemoryStore Is Test-Only and Panics in Production~~ | 07 - Pagination & Caching | CAP-011 | P0 | removed |
| BC-2.07.009 | ~~FileStore Is the Default and Only Production CursorStore~~ | 07 - Pagination & Caching | CAP-011 | P0 | removed |
| BC-2.07.010 | ~~State File Directory Follows {client}/{sensor}/{source}.json~~ | 07 - Pagination & Caching | CAP-011 | P0 | removed |
| BC-2.08.001 | On-Demand Connectivity Check Per Sensor Per Client | 08 - Sensor Health | CAP-008 | P1 | draft |
| BC-2.08.002 | Auth Validity Check Per Sensor Per Client | 08 - Sensor Health | CAP-008 | P1 | draft |
| BC-2.08.003 | Rate Limit State Detection Per Sensor | 08 - Sensor Health | CAP-008 | P1 | draft |
| BC-2.08.004 | Last Successful Query Timestamp Per Sensor Per Client | 08 - Sensor Health | CAP-008 | P1 | draft |
| BC-2.08.005 | Health Check MCP Tool | 08 - Sensor Health | CAP-008 | P1 | draft |
| BC-2.08.006 | Health Status MCP Resource | 08 - Sensor Health | CAP-008 | P1 | draft |
| BC-2.08.007 | Partial Health Status (Mixed Sensor Availability) | 08 - Sensor Health | CAP-008 | P1 | draft |
| BC-2.09.001 | Structural Separation of Untrusted Data | 09 - Prompt Injection Defense | CAP-010 | P0 | draft |
| BC-2.09.002 | Provenance Framing in Tool Descriptions | 09 - Prompt Injection Defense | CAP-010 | P0 | draft |
| BC-2.09.003 | Suspicious Pattern Detection via Regex | 09 - Prompt Injection Defense | CAP-010 | P0 | draft |
| BC-2.09.004 | Safety Flag Parallel Fields (Flag, Don't Strip) | 09 - Prompt Injection Defense | CAP-010 | P0 | draft |
| BC-2.09.005 | Trust-Level Metadata Per Response | 09 - Prompt Injection Defense | CAP-010 | P0 | draft |
| BC-2.09.006 | Tool Description Security Warnings | 09 - Prompt Injection Defense | CAP-010 | P0 | draft |
| BC-2.09.007 | OutputSchema for Type-Safe LLM Reasoning | 09 - Prompt Injection Defense | CAP-010 | P0 | draft |
| BC-2.09.008 | Response Envelope with Trust Annotations | 09 - Prompt Injection Defense | CAP-010 | P0 | draft |
| BC-2.10.001 | rmcp ServerHandler Implementation | 10 - MCP Server & Transport | CAP-005 | P0 | draft |
| BC-2.10.002 | Tool Registration via #[tool_router] | 10 - MCP Server & Transport | CAP-005 | P0 | draft |
| BC-2.10.003 | Conditional Tool Registration (Feature-Flag Gated) | 10 - MCP Server & Transport | CAP-005 | P0 | draft |
| BC-2.10.004 | client_id Parameter on Every Tool (Stateless Model) | 10 - MCP Server & Transport | CAP-009 | P0 | draft |
| BC-2.10.005 | ~~notifications/tools/list_changed on Client Context Switch~~ | 10 - MCP Server & Transport | CAP-005 | P0 | removed |
| BC-2.10.006 | Stdio Transport | 10 - MCP Server & Transport | -- | P0 | draft |
| BC-2.10.007 | Structured Error Responses | 10 - MCP Server & Transport | CAP-007 | P0 | draft |
| BC-2.10.008 | MCP Resources for Client List and Sensor Inventory | 10 - MCP Server & Transport | CAP-009 | P0 | draft |
| BC-2.10.009 | MCP Prompts for Common Workflows | 10 - MCP Server & Transport | CAP-010 | P1 | draft |
| BC-2.10.010 | Graceful Shutdown on SIGTERM/SIGINT | 10 - MCP Server & Transport | -- | P0 | draft |
| BC-2.10.011 | list_capabilities Meta-Tool | 10 - MCP Server & Transport | CAP-005 | P0 | draft |

## Summary

| Subsystem | BC Count | P0 | P1 | Removed |
|-----------|----------|----|----|---------|
| 01 - Sensor Query Pipeline | 14 | 13 | 1 | 1 |
| 02 - OCSF Normalization | 12 | 12 | 0 | 0 |
| 03 - Credential Management | 12 | 12 | 0 | 0 |
| 04 - Feature Flag System | 14 | 8 | 6 | 1 |
| 05 - Audit & Compliance | 10 | 10 | 0 | 0 |
| 06 - Client Configuration | 9 | 9 | 0 | 1 |
| 07 - Pagination & Caching | 6 | 6 | 0 | 4 |
| 08 - Sensor Health | 7 | 0 | 7 | 0 |
| 09 - Prompt Injection Defense | 8 | 8 | 0 | 0 |
| 10 - MCP Server & Transport | 10 | 9 | 1 | 1 |
| **Total** | **102** | **87** | **15** | **8** |

### Change Log (Adversarial Review Fixes)

**Removed BCs (8):**
- BC-2.01.012: Query Fingerprint Validation at Startup -- persistent cursor fingerprints eliminated with ephemeral pagination model
- BC-2.04.014: notifications/tools/list_changed on Client Context Switch -- no session-level active client in stateless model
- BC-2.06.009: Client Context Switch Triggers notifications/tools/list_changed -- no session-level active client in stateless model
- BC-2.07.007: State Is Isolated Per-Client, Per-Sensor, Per-Source -- persistent state eliminated
- BC-2.07.008: MemoryStore Is Test-Only and Panics in Production -- FileStore/MemoryStore removed with ephemeral model
- BC-2.07.009: FileStore Is the Default and Only Production CursorStore -- FileStore removed with ephemeral model
- BC-2.07.010: State File Directory Follows {client}/{sensor}/{source}.json -- persistent state directories eliminated
- BC-2.10.005: notifications/tools/list_changed on Client Context Switch -- no session-level active client in stateless model

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
