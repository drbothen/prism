---
document_type: behavioral-contract-summary
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
subsystems: ["01-Sensor Query Pipeline", "02-OCSF Normalization", "03-Credential Management", "04-Feature Flag System"]
---

# Behavioral Contracts Summary: Subsystems 01-04

## Subsystem 01: Sensor Query Pipeline (15 BCs)

Capabilities: CAP-001, CAP-002, CAP-012

| ID | Title | Capability | Priority |
|----|-------|-----------|----------|
| BC-2.01.001 | Single-Client Sensor Query Returns Scoped Results | CAP-001 | P0 |
| BC-2.01.002 | Cross-Client Fan-Out Query Aggregates Results with Per-Client Attribution | CAP-002 | P0 |
| BC-2.01.003 | Cursor-Based Forward-Only Pagination | CAP-001 | P0 |
| BC-2.01.004 | Offset-Based Hybrid Pagination for Claroty Audit Logs | CAP-001 | P0 |
| BC-2.01.005 | CrowdStrike OAuth2 Authentication and Two-Step Fetch | CAP-001 | P0 |
| BC-2.01.006 | Cyberint Cookie-Based Authentication and Multi-Format Timestamp Parsing | CAP-001 | P0 |
| BC-2.01.007 | Claroty Bearer Token Auth with Polymorphic ID Handling | CAP-001 | P0 |
| BC-2.01.008 | Armis Bearer Token Auth with AQL Query Forwarding and Timestamp Fallback | CAP-001 | P0 |
| BC-2.01.009 | Query Filtering and Sorting Parameters | CAP-001 | P0 |
| BC-2.01.010 | Partial Failure Handling for Paginated and Cross-Client Queries | CAP-001, CAP-002 | P0 |
| BC-2.01.011 | Cross-Sensor Correlation via OCSF Field Alignment | CAP-012 | P1 |
| BC-2.01.012 | Query Fingerprint Validation at Startup | CAP-001 | P0 |
| BC-2.01.013 | DataSource Trait Eliminates Per-Sensor Code Duplication | CAP-001 | P0 |
| BC-2.01.014 | Exponential Backoff and Retry for Transient Sensor API Errors | CAP-001 | P0 |
| BC-2.01.015 | MCP Tool Response Envelope Structure | CAP-001 | P0 |

## Subsystem 02: OCSF Normalization (12 BCs)

Capability: CAP-003

| ID | Title | Capability | Priority |
|----|-------|-----------|----------|
| BC-2.02.001 | OCSF Schema Loading at Build Time via ocsf-proto-gen | CAP-003 | P0 |
| BC-2.02.002 | DynamicMessage Creation from Sensor Records | CAP-003 | P0 |
| BC-2.02.003 | CrowdStrike Alert Field Mapping to OCSF | CAP-003 | P0 |
| BC-2.02.004 | Cyberint Alert Field Mapping to OCSF | CAP-003 | P0 |
| BC-2.02.005 | Claroty xDome Field Mapping to OCSF (9 Data Sources) | CAP-003 | P0 |
| BC-2.02.006 | Armis Centrix Field Mapping to OCSF (7 Data Sources) | CAP-003 | P0 |
| BC-2.02.007 | Vendor Extension Preservation in raw_extensions | CAP-003 | P0 |
| BC-2.02.008 | Three-Tier Field Alias Resolution | CAP-003 | P0 |
| BC-2.02.009 | OCSF Version Pinning Per Release | CAP-003 | P0 |
| BC-2.02.010 | OCSF Enum Value Map for Runtime Display Names | CAP-003 | P0 |
| BC-2.02.011 | Graceful Normalization Error Handling (No Silent Data Loss) | CAP-003 | P0 |
| BC-2.02.012 | OCSF Event Class Selection Per Sensor Record Type | CAP-003 | P0 |

## Subsystem 03: Credential Management (12 BCs)

Capability: CAP-004

| ID | Title | Capability | Priority |
|----|-------|-----------|----------|
| BC-2.03.001 | CredentialStore Trait with Tenant-Scoped Operations | CAP-004 | P0 |
| BC-2.03.002 | OS Keyring Backend via keyring-rs | CAP-004 | P0 |
| BC-2.03.003 | AES-256-GCM Encrypted File Backend Fallback | CAP-004 | P0 |
| BC-2.03.004 | Credential Namespace Isolation by (client_id, sensor_id, credential_name) | CAP-004 | P0 |
| BC-2.03.005 | Credential CRUD Operations via MCP Tools | CAP-004 | P0 |
| BC-2.03.006 | Credential Resolution at Sensor Query Time | CAP-004 | P0 |
| BC-2.03.007 | Secret Redaction in Logs, Errors, and MCP Responses | CAP-004 | P0 |
| BC-2.03.008 | Credential Name Sanitization Against Path Traversal | CAP-004 | P0 |
| BC-2.03.009 | resolve_secret() for _FILE Env Var and K8s Secret Mount Compatibility | CAP-004 | P0 |
| BC-2.03.010 | Credential Access Audit Logging | CAP-004 | P0 |
| BC-2.03.011 | Keyring Startup Probe for Permission Pre-Authorization | CAP-004 | P0 |
| BC-2.03.012 | Credential Backend Selection and Fallback | CAP-004 | P0 |

## Subsystem 04: Feature Flag System (15 BCs)

Capabilities: CAP-005, CAP-006

| ID | Title | Capability | Priority |
|----|-------|-----------|----------|
| BC-2.04.001 | Compile-Time Cargo Features Gate Write Code Families | CAP-005 | P0 |
| BC-2.04.002 | Runtime Per-Client TOML Feature Flag Configuration | CAP-005 | P0 |
| BC-2.04.003 | Hierarchical Capability Resolution (Most-Specific to Least-Specific with Deny Fallback) | CAP-005 | P0 |
| BC-2.04.004 | Two-Tier Gate -- Both Compile-Time and Runtime Must Permit Operation | CAP-005 | P0 |
| BC-2.04.005 | Hidden Tools Pattern -- Disabled Write Tools Omitted from tools/list | CAP-005 | P0 |
| BC-2.04.006 | list_capabilities Meta-Tool for Capability Discovery | CAP-005 | P0 |
| BC-2.04.007 | Three-Tier Risk Classification for Operations | CAP-006 | P1 |
| BC-2.04.008 | Dry-Run Default for Reversible Write Operations | CAP-006 | P1 |
| BC-2.04.009 | Confirmation Token Generation for Irreversible Write Operations | CAP-006 | P1 |
| BC-2.04.010 | Confirmation Token Consumption via confirm_action | CAP-006 | P1 |
| BC-2.04.011 | Token Expiry at 300 Seconds with Structured Error Recovery | CAP-006 | P1 |
| BC-2.04.012 | Token Content Hash Verification Prevents Action Tampering | CAP-006 | P1 |
| BC-2.04.013 | Feature Flag Evaluation Audit Logging for Write Operations | CAP-005 | P0 |
| BC-2.04.014 | notifications/tools/list_changed on Client Context Switch | CAP-005 | P0 |
| BC-2.04.015 | Structured Error When Write Capability Is Denied | CAP-005 | P0 |

## Totals

| Subsystem | BC Count | P0 | P1 |
|-----------|----------|----|----|
| 01 - Sensor Query Pipeline | 15 | 14 | 1 |
| 02 - OCSF Normalization | 12 | 12 | 0 |
| 03 - Credential Management | 12 | 12 | 0 |
| 04 - Feature Flag System | 15 | 9 | 6 |
| **Total** | **54** | **47** | **7** |

## Invariant Coverage

| Invariant | BCs Referencing |
|-----------|----------------|
| DI-001 (Cursor Forward Progress) | BC-2.01.003, BC-2.01.004, BC-2.01.007, BC-2.01.008, BC-2.01.010 |
| DI-002 (Credential Isolation) | BC-2.03.001 through BC-2.03.012 |
| DI-003 (Feature Flag Deny-by-Default) | BC-2.04.001 through BC-2.04.015 |
| DI-004 (Audit Completeness) | BC-2.01.001, BC-2.01.002, BC-2.01.014, BC-2.01.015, BC-2.03.005, BC-2.03.010, BC-2.04.013 |
| DI-005 (OCSF Schema Validity) | BC-2.02.001 through BC-2.02.012 |
| DI-006 (Prompt Injection Sanitization) | BC-2.01.015 |
| DI-007 (Confirmation Token Expiry) | BC-2.04.009 through BC-2.04.012 |
| DI-008 (Client Data Separation) | BC-2.01.001, BC-2.01.002, BC-2.01.011 |
| DI-009 (Persistence Before State Update) | BC-2.01.003, BC-2.01.004, BC-2.01.010 |
| DI-010 (Query Fingerprint Consistency) | BC-2.01.009, BC-2.01.012 |
| DI-012 (Sealed Auth Trait) | BC-2.01.005, BC-2.01.006, BC-2.01.007, BC-2.01.008, BC-2.01.013 |
| DI-013 (Atomic State Writes) | BC-2.01.003 |
| DI-014 (Credential Name Sanitization) | BC-2.03.008 |

## Edge Case Coverage from Domain Spec

| Domain Edge Case | BC Reference |
|-----------------|-------------|
| DEC-001 (HTTP 503 mid-pagination) | BC-2.01.010, BC-2.01.014 |
| DEC-002 (CrowdStrike token expires mid-query) | BC-2.01.005 |
| DEC-003 (Cross-client partial credential failure) | BC-2.01.002 |
| DEC-004 (Client with zero sensors) | BC-2.01.001 |
| DEC-005 (Cross-client sensor not configured) | BC-2.01.002 |
| DEC-006 (Config change while running) | BC-2.04.002 |
| DEC-007 (Unmapped CrowdStrike field) | BC-2.02.003, BC-2.02.007 |
| DEC-008 (Prompt injection in hostname) | BC-2.01.015 |
| DEC-009 (Expired confirmation token) | BC-2.04.010 |
| DEC-010 (Claroty polymorphic IDs) | BC-2.01.007 |
| DEC-011 (OS keyring locked) | BC-2.03.002, BC-2.03.011 |
| DEC-012 (Query fingerprint mismatch) | BC-2.01.009, BC-2.01.012 |
| DEC-013 (Armis missing timestamps) | BC-2.01.008, BC-2.02.006 |
| DEC-015 (Cyberint 5th timestamp format) | BC-2.01.006, BC-2.02.004 |
