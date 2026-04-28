---
document_type: behavioral-contract-summary
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystems: ["05-Audit Trail", "06-Client Configuration", "07-Adapter Pagination & Response Cache"]
capabilities: ["CAP-007", "CAP-009", "CAP-011"]
---

# Behavioral Contracts Summary: Subsystems 05-07

## Overview

This document summarizes 27 active behavioral contracts across three subsystems (31 historical; 4 removed across SS-07). Each BC specifies a single testable behavior with preconditions, postconditions, invariants, error cases, and edge cases.

---

## Subsystem 05: Audit Trail (CAP-007) -- 11 BCs

| BC ID | Title | Key Invariants | Priority |
|-------|-------|----------------|----------|
| BC-2.05.001 | Every MCP Tool Invocation Produces Exactly One Audit Entry (Fail-Closed for Writes) | DI-004 | P0 |
| BC-2.05.002 | Audit Entries Use Structured JSON Format with Complete Fields | DI-004 | P0 |
| BC-2.05.003 | Credential Values Are Never Present in Audit Entries | DI-002 | P0 |
| BC-2.05.004 | Write Operations Log Capability Check and Execution Outcome | DI-003, DI-004 | P0 |
| BC-2.05.005 | Credential Access Events Are Audit-Logged with Context | DI-002, DI-004 | P0 |
| BC-2.05.006 | Audit Entries Are Append-Only and Immutable | DI-004 | P0 |
| BC-2.05.007 | Audit Entries Are Compatible with the Vector Pipeline | DI-004 | P0 |
| BC-2.05.008 | Audit Entries Satisfy SOC 2 Type II and ISO 27001 Requirements | DI-003, DI-004 | P0 |
| BC-2.05.009 | Feature Flag Evaluations for Write Operations Are Audit-Logged | DI-003, DI-004 | P0 |
| BC-2.05.010 | Confirmation Token Lifecycle Events Are Audit-Logged | DI-004, DI-007 | P0 |
| BC-2.05.011 | Audit Forwarding — At-Least-Once Delivery to External Destinations (VP-039 monotonic watermark) | DI-026 | P0 |

**Coverage notes:** BC-2.05.001 through BC-2.05.003 cover the foundational audit mechanics (one entry per invocation, structured format, secret redaction). BC-2.05.004 and BC-2.05.005 cover the two specialized audit domains (write operations and credential access). BC-2.05.006 through BC-2.05.008 cover compliance properties (immutability, Vector compatibility, SOC 2/ISO 27001 field requirements). BC-2.05.009 and BC-2.05.010 cover feature flag evaluation and confirmation token lifecycle audit trails.

---

## Subsystem 06: Client Configuration (CAP-009) -- 10 active BCs, 0 removed

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
| BC-2.06.009 | Config Reload Triggers notifications/tools/list_changed | DI-003 | P0 |
| BC-2.06.010 | Client ID Validation Enforces Allowed Character Set | DI-008, DI-033 | P0 |

**Coverage notes:** BC-2.06.001 and BC-2.06.002 cover TOML loading and per-client sensor mapping. BC-2.06.003 covers the three-tier credential resolution chain (\_FILE env > env var > credential store). BC-2.06.004 covers capability merging semantics. BC-2.06.005 through BC-2.06.007 cover the configuration validation UX (multi-error, dry-run, actionable messages). BC-2.06.008 covers layered config precedence. BC-2.06.009 covers config-reload-triggered `notifications/tools/list_changed` notification (un-retired 2026-04-17, Burst 21 — new Config-Reload semantics). BC-2.06.010 covers OrgSlug validation (formerly TenantId; renamed per ADR-006).

---

## Subsystem 07: Adapter Pagination & Response Cache (CAP-011, CAP-014) -- 6 active BCs, 4 removed

Pagination is now entirely internal to the query engine's sensor fetch layer. No pagination tokens are exposed to the MCP agent. The agent uses `limit` and `total_available` on the `query` tool. Only one cache type exists: the query engine's sensor-fetch cache (no "direct tool cache").

| BC ID | Title | Key Invariants | Priority | Status |
|-------|-------|----------------|----------|--------|
| BC-2.07.001 | Internal Ephemeral Pagination Token Structure | DI-001 | P0 | draft |
| BC-2.07.002 | Internal Pagination Token Lifecycle — Forward Progress, Timeout, and Cleanup | DI-001 | P0 | draft |
| BC-2.07.003 | Query Engine Sensor-Fetch Cache with Configurable TTL | DI-018 | P1 | draft |
| BC-2.07.004 | Cache Invalidation on Write Operations | DI-018 | P1 | draft |
| BC-2.07.005 | Cache Key Derivation from Push-Down Parameters | DI-018 | P1 | draft |
| BC-2.07.006 | Cache Memory Bounds and Eviction Policy | DI-018 | P1 | draft |

**Coverage notes:** BC-2.07.001 and BC-2.07.002 cover internal pagination token structure and lifecycle (forward-only progress, fetch timeout, concurrent fetch limits). These are internal to the query engine -- no tokens are exposed to the MCP agent. BC-2.07.003 through BC-2.07.006 cover the query engine's sensor-fetch cache (TTL, invalidation, key derivation, memory bounds). Cache keys use push-down parameter hashes only -- there is no separate "tool query hash".

---

## Invariant Coverage Matrix

| Invariant | Subsystem 05 BCs | Subsystem 06 BCs | Subsystem 07 BCs |
|-----------|-------------------|-------------------|-------------------|
| DI-001 (Cursor Forward Progress) | -- | -- | 001, 002 |
| DI-002 (Credential Isolation) | 003, 005 | 003 | -- |
| DI-003 (Feature Flag Deny-by-Default) | 004, 008, 009 | 004 | -- |
| DI-004 (Audit Completeness) | 001-010 (all) | -- | -- |
| DI-026 (Audit Buffer Durability) | 011 | -- | -- |
| DI-007 (Confirmation Token Expiry) | 010 | -- | -- |
| DI-008 (Client Data Separation) | -- | 001, 002, 010 | -- |
| DI-018 (Cache Bounds) | -- | -- | 003, 004, 006 |
| DI-014 (Credential Name Sanitization) | -- | 003 | -- |
| DI-029 (Correlation Window >= Schedule Interval) | -- | BC-2.06.005 | -- |

## Domain Edge Case Coverage

| DEC ID | Description | Covered By |
|--------|-------------|------------|
| DEC-004 | Client with zero sensors | BC-2.06.002 |
| DEC-006 | Config change while running (no hot-reload) | BC-2.06.001 |
| DEC-009 | Expired confirmation token | BC-2.05.010 |
| DEC-010 | Claroty polymorphic IDs | BC-2.07.001 |
| DEC-011 | OS keyring locked | BC-2.06.003 |
| DEC-013 | Armis record with no valid timestamp | BC-2.07.001 |
| DEC-014 | Tracing subscriber error during audit | BC-2.05.001 |
| DEC-020 | Cross-client fetch ordering fairness | BC-2.07.002 |

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.1 | 2026-04-27 | product-owner | Pass 15 sweep: BC-2.06.010 coverage note updated TenantId → OrgSlug (ADR-006); DI-033 added to BC-2.06.010 traceability row. |
