---
document_type: behavioral-contract-summary
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystems: ["Audit & Compliance", "Client Configuration", "Cursor State Management"]
capabilities: ["CAP-007", "CAP-009", "CAP-011"]
---

# Behavioral Contracts Summary: Subsystems 05-07

## Overview

This document summarizes 30 behavioral contracts across three subsystems. Each BC specifies a single testable behavior with preconditions, postconditions, invariants, error cases, and edge cases.

---

## Subsystem 05: Audit & Compliance (CAP-007) -- 10 BCs

| BC ID | Title | Key Invariants | Priority |
|-------|-------|----------------|----------|
| BC-2.05.001 | Every MCP Tool Invocation Produces Exactly One Audit Entry | DI-004 | P0 |
| BC-2.05.002 | Audit Entries Use Structured JSON Format with Complete Fields | DI-004 | P0 |
| BC-2.05.003 | Credential Values Are Never Present in Audit Entries | DI-002 | P0 |
| BC-2.05.004 | Write Operations Log Capability Check and Execution Outcome | DI-003, DI-004 | P0 |
| BC-2.05.005 | Credential Access Events Are Audit-Logged with Context | DI-002, DI-004 | P0 |
| BC-2.05.006 | Audit Entries Are Append-Only and Immutable | DI-004 | P0 |
| BC-2.05.007 | Audit Entries Are Compatible with the Vector Pipeline | DI-004 | P0 |
| BC-2.05.008 | Audit Entries Satisfy SOC 2 Type II and ISO 27001 Requirements | DI-003, DI-004 | P0 |
| BC-2.05.009 | Feature Flag Evaluations for Write Operations Are Audit-Logged | DI-003, DI-004 | P0 |
| BC-2.05.010 | Confirmation Token Lifecycle Events Are Audit-Logged | DI-004, DI-007 | P0 |

**Coverage notes:** BC-2.05.001 through BC-2.05.003 cover the foundational audit mechanics (one entry per invocation, structured format, secret redaction). BC-2.05.004 and BC-2.05.005 cover the two specialized audit domains (write operations and credential access). BC-2.05.006 through BC-2.05.008 cover compliance properties (immutability, Vector compatibility, SOC 2/ISO 27001 field requirements). BC-2.05.009 and BC-2.05.010 cover feature flag evaluation and confirmation token lifecycle audit trails.

---

## Subsystem 06: Client Configuration (CAP-009) -- 10 BCs

| BC ID | Title | Key Invariants | Priority |
|-------|-------|----------------|----------|
| BC-2.06.001 | TOML Configuration Loads and Deserializes at Startup | DI-008 | P0 |
| BC-2.06.002 | Per-Client Sensor Mapping from TOML Configuration | DI-008 | P0 |
| BC-2.06.003 | Credential References in Config Resolve to Credential Store Entries | DI-002, DI-014 | P0 |
| BC-2.06.004 | Capability Overrides Merge with Defaults Using More-Specific-Wins | DI-003 | P0 |
| BC-2.06.005 | Configuration Validation Reports All Errors in One Pass | -- | P0 |
| BC-2.06.006 | --dry-run Flag Validates Config and Prints Redacted Summary | DI-002 | P0 |
| BC-2.06.007 | Missing Required Fields Produce Actionable Error Messages | -- | P0 |
| BC-2.06.008 | Default Values Apply and Environment Variables Override TOML | -- | P0 |
| BC-2.06.009 | Client Context Switch Triggers notifications/tools/list_changed | DI-003 | P0 |
| BC-2.06.010 | Client ID Validation Enforces Allowed Character Set | DI-008 | P0 |

**Coverage notes:** BC-2.06.001 and BC-2.06.002 cover TOML loading and per-client sensor mapping. BC-2.06.003 covers the three-tier credential resolution chain (\_FILE env > env var > credential store). BC-2.06.004 covers capability merging semantics. BC-2.06.005 through BC-2.06.007 cover the configuration validation UX (multi-error, dry-run, actionable messages). BC-2.06.008 covers layered config precedence. BC-2.06.009 covers MCP tool list notification on context switch. BC-2.06.010 covers TenantId validation.

---

## Subsystem 07: Cursor State Management (CAP-011) -- 10 BCs

| BC ID | Title | Key Invariants | Priority |
|-------|-------|----------------|----------|
| BC-2.07.001 | Cursor Is a Composite of Timestamp and RecordID | DI-001 | P0 |
| BC-2.07.002 | Cursor Regression Is Detected and Produces a Fatal Error | DI-001 | P0 |
| BC-2.07.003 | State Files Use Atomic Write Pattern (temp + fsync + rename) | DI-009, DI-013 | P0 |
| BC-2.07.004 | Cursor State Is Persisted AFTER Successful Delivery | DI-009 | P0 |
| BC-2.07.005 | Query Fingerprint Is SHA-256 of Sorted Config Fields | DI-010 | P0 |
| BC-2.07.006 | Fingerprint Mismatch at Startup Is a Fatal Error | DI-010 | P0 |
| BC-2.07.007 | State Is Isolated Per-Client, Per-Sensor, Per-Source | DI-001, DI-008 | P0 |
| BC-2.07.008 | MemoryStore Is Test-Only and Panics in Production | DI-011 | P0 |
| BC-2.07.009 | FileStore Is the Default and Only Production CursorStore | DI-011, DI-013 | P0 |
| BC-2.07.010 | State File Directory Follows {client}/{sensor}/{source}.json | DI-008, DI-014 | P0 |

**Coverage notes:** BC-2.07.001 and BC-2.07.002 cover cursor structure and the forward-only invariant. BC-2.07.003 and BC-2.07.004 cover crash safety (atomic writes) and the delivery-before-persistence ordering that fixes the poller-cobra bug. BC-2.07.005 and BC-2.07.006 cover query fingerprinting and mismatch detection. BC-2.07.007 covers state isolation. BC-2.07.008 and BC-2.07.009 cover the MemoryStore/FileStore split. BC-2.07.010 covers the directory structure convention.

---

## Invariant Coverage Matrix

| Invariant | Subsystem 05 BCs | Subsystem 06 BCs | Subsystem 07 BCs |
|-----------|-------------------|-------------------|-------------------|
| DI-001 (Cursor Forward Progress) | -- | -- | 001, 002, 007 |
| DI-002 (Credential Isolation) | 003, 005 | 003 | -- |
| DI-003 (Feature Flag Deny-by-Default) | 004, 008, 009 | 004, 009 | -- |
| DI-004 (Audit Completeness) | 001-010 (all) | -- | -- |
| DI-007 (Confirmation Token Expiry) | 010 | -- | -- |
| DI-008 (Client Data Separation) | -- | 001, 002, 010 | 007, 010 |
| DI-009 (Persistence Before State Update) | -- | -- | 003, 004 |
| DI-010 (Query Fingerprint Consistency) | -- | -- | 005, 006 |
| DI-011 (MemoryStore Production Ban) | -- | -- | 008, 009 |
| DI-013 (Atomic State Writes) | -- | -- | 003, 009 |
| DI-014 (Credential Name Sanitization) | -- | 003 | 010 |

## Domain Edge Case Coverage

| DEC ID | Description | Covered By |
|--------|-------------|------------|
| DEC-004 | Client with zero sensors | BC-2.06.002 |
| DEC-006 | Config change while running (no hot-reload) | BC-2.06.001 |
| DEC-009 | Expired confirmation token | BC-2.05.010 |
| DEC-010 | Claroty polymorphic IDs | BC-2.07.001 |
| DEC-011 | OS keyring locked | BC-2.06.003 |
| DEC-012 | Query fingerprint mismatch after config change | BC-2.07.006 |
| DEC-013 | Armis record with no valid timestamp | BC-2.07.001 |
| DEC-014 | Tracing subscriber error during audit | BC-2.05.001 |
