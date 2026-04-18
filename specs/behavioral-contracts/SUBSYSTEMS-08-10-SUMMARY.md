---
document_type: behavioral-contract-summary
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
---

# Behavioral Contracts Summary: Subsystems 08-10

## Overview

This document summarizes the behavioral contracts written for Prism subsystems 08 (Sensor Health), 09 (Prompt Injection Defense), and 10 (MCP Interface), plus three PRD supplement documents.

---

## Subsystem 08: Sensor Health (CAP-008) — 7 Contracts

| BC ID | Title | Priority | Key Invariants |
|-------|-------|----------|----------------|
| BC-2.08.001 | On-Demand Connectivity Check Per Sensor Per Client | P1 | DI-004, DI-008 |
| BC-2.08.002 | Auth Validity Check Per Sensor Per Client | P1 | DI-002, DI-008 |
| BC-2.08.003 | Rate Limit State Detection Per Sensor | P1 | DI-008 |
| BC-2.08.004 | Last Successful Query Timestamp Per Sensor Per Client | P1 | DI-008 |
| BC-2.08.005 | Health Check MCP Tool | P1 | DI-004, DI-008 |
| BC-2.08.006 | Health Status MCP Resource | P1 | DI-008 |
| BC-2.08.007 | Partial Health Status (Mixed Sensor Availability) | P1 | DI-004 |

**Design Decisions:**
- Health checks are on-demand, not polled (per CAP-008 spec)
- Each `verify_connectivity()` call exercises the actual sensor API, not a synthetic healthcheck endpoint
- Auth validity distinguishes between "auth invalid" and "sensor unreachable, cannot verify"
- Rate limit state is derived from observed HTTP headers and 429 responses
- Partial health (some sensors up, some down) is a success response, not an error
- Health MCP resource provides cached data; health MCP tool performs fresh checks

---

## Subsystem 09: Prompt Injection Defense (CAP-010) — 8 Contracts

| BC ID | Title | Priority | Key Invariants |
|-------|-------|----------|----------------|
| BC-2.09.001 | Structural Separation of Untrusted Data | P0 | DI-006 |
| BC-2.09.002 | Provenance Framing in Tool Descriptions | P0 | DI-006 |
| BC-2.09.003 | Suspicious Pattern Detection via Regex | P0 | DI-006 |
| BC-2.09.004 | Safety Flag Parallel Fields (Flag, Don't Strip) | P0 | DI-006 |
| BC-2.09.005 | Trust-Level Metadata Per Response | P0 | DI-006 |
| BC-2.09.006 | Tool Description Security Warnings | P0 | DI-006 |
| BC-2.09.007 | OutputSchema for Type-Safe LLM Reasoning | P0 | DI-006 |
| BC-2.09.008 | Response Envelope with Trust Annotations | P0 | DI-004, DI-006 |

**Design Decisions:**
- All 8 contracts are P0 because prompt injection defense is critical for a system that passes attacker-controlled data through LLM context
- Four defense layers operate simultaneously: structural separation, provenance framing, suspicious pattern detection, trust-level metadata
- Flag, don't strip: original data preserved for forensic analysis; `_safety_flag` parallel fields are additive
- Two trust levels only: `"untrusted_external"` (sensor data) and `"internal"` (Prism-generated)
- OutputSchema enables the LLM to reason about response structure before seeing potentially adversarial data
- No sanitization is considered 100% effective; the human analyst is the ultimate safety boundary

---

## Subsystem 10: MCP Interface (Cross-Cutting) — 11 Contracts

| BC ID | Title | Priority | Key Invariants |
|-------|-------|----------|----------------|
| BC-2.10.001 | rmcp ServerHandler Implementation | P0 | DI-003, DI-004 |
| BC-2.10.002 | Tool Registration via #[tool_router] (15 tools) | P0 | DI-003 |
| BC-2.10.003 | Conditional Tool Registration (Feature-Flag Gated) | P0 | DI-003 |
| BC-2.10.004 | Client Scoping on Every Tool (Stateless Model) | P0 | DI-008 |
| BC-2.10.005 | notifications/tools/list_changed on Client Context Switch | P0 | DI-003 |
| BC-2.10.006 | Stdio Transport | P0 | — |
| BC-2.10.007 | Structured Error Responses | P0 | DI-004, DI-006 |
| BC-2.10.008 | MCP Resources for Client List and Sensor Inventory | P0 | DI-002, DI-008 |
| BC-2.10.009 | MCP Prompts for Common Workflows | P1 | DI-006 |
| BC-2.10.010 | Graceful Shutdown on SIGTERM/SIGINT | P0 | — |
| BC-2.10.011 | list_capabilities Meta-Tool | P0 | DI-003 |

**Design Decisions:**
- stdout reserved exclusively for MCP JSON-RPC; all diagnostics to stderr
- Hidden tools pattern: disabled write tools omitted from `tools/list`, not listed as unavailable
- Read tools use `clients` array (via `query`); write tools use scalar `client_id`; no session-level "active client"
- Structured errors include `category`, `retryable`, `retry_after_seconds`, `suggestion`, and `original_params_valid` to enable LLM self-correction
- Graceful shutdown within 5 seconds; force-exit after timeout
- Error responses treat upstream sensor messages as untrusted data (structural separation applies)

---

## PRD Supplements

| Document | Path | Content |
|----------|------|---------|
| Error Taxonomy | `prd-supplements/error-taxonomy.md` | 9 error categories (AUTH, SENSOR, OCSF, CRED, FLAG, STATE, CFG, MCP, SAFETY), 35 error codes with severity, retryability, and message formats |
| NFR Catalog | `prd-supplements/nfr-catalog.md` | 16 non-functional requirements covering performance (query latency, normalization overhead, memory), security (encryption, audit, prompt injection), reliability (cursor durability, graceful shutdown), observability (structured logging, tracing), compatibility (cross-platform, OCSF versions) |
| Interface Definitions | `prd-supplements/interface-definitions.md` | MCP tool schemas (sensor query, health check, capabilities), error response schema, TOML configuration schema with required fields, CLI flags, exit codes |

---

## Cross-Cutting Traceability

| Domain Invariant | Enforced By |
|-----------------|-------------|
| DI-002 (Credential Isolation) | BC-2.08.002, BC-2.10.008 |
| DI-003 (Feature Flag Deny-by-Default) | BC-2.10.001, BC-2.10.002, BC-2.10.003, BC-2.10.005, BC-2.10.011 |
| DI-004 (Audit Completeness) | BC-2.08.001, BC-2.08.005, BC-2.08.007, BC-2.09.008, BC-2.10.001, BC-2.10.007 |
| DI-006 (Prompt Injection Sanitization) | BC-2.09.001 through BC-2.09.008, BC-2.10.007, BC-2.10.009 |
| DI-008 (Client Data Separation) | BC-2.08.001 through BC-2.08.006, BC-2.10.004, BC-2.10.008 |

| Risk | Mitigated By |
|------|-------------|
| R-001 (rmcp breaking changes) | BC-2.10.001 (pinned version, adapter layer) |
| R-005 (Prompt injection) | BC-2.09.001 through BC-2.09.008 (four defense layers) |
| R-006 (Credential exposure) | BC-2.08.002, BC-2.10.008 (no values in responses) |
| R-009 (keyring incompatibility) | BC-2.08.002 (fallback detection in health) |
| R-010 (Rate limit exhaustion) | BC-2.08.003 (rate state in health) |

## Statistics

- **Total behavioral contracts:** 26 (7 + 8 + 11)
- **P0 contracts:** 19
- **P1 contracts:** 7 (all Sensor Health + MCP Prompts)
- **Edge cases covered:** 47
- **Error codes defined:** 35
- **NFRs defined:** 16
