---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "365fb25"
traces_to: ["CAP-034"]
extracted_from: ".factory/specs/prd.md"
origin: greenfield
subsystem: "SS-10"
capability: "CAP-034"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.10.006: Stdio Transport

## Description

Prism uses MCP JSON-RPC 2.0 over stdio, initialized via `rmcp::ServiceExt::serve(stdio())`. stdout is reserved exclusively for MCP JSON-RPC protocol messages — all logging, diagnostics, and metrics write to stderr via `tracing_subscriber`. One stdio session equals one analyst; there is no multiplexing. A broken stdin or stdout pipe triggers graceful shutdown per BC-2.10.010. Pagination keeps individual responses bounded; no chunking at the MCP level.

## Preconditions
- Prism is launched as a child process by Claude Code (or another MCP client)
- stdin and stdout are connected to the MCP client via pipes

## Postconditions
- MCP JSON-RPC 2.0 messages are read from stdin and responses written to stdout
- stdout is reserved exclusively for MCP JSON-RPC protocol messages
- All logging, diagnostics, and metrics are written to stderr (via `tracing_subscriber`)
- The transport is initialized via `rmcp::ServiceExt::serve(stdio())`
- One stdio session corresponds to one analyst; there is no multiplexing
- The server processes one request at a time per the JSON-RPC 2.0 over stdio convention

## Invariants
- stdout purity: no non-MCP content ever written to stdout

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Transport error | stdin pipe broken (client process died) | Prism detects broken pipe, initiates graceful shutdown (BC-2.10.010) |
| Transport error | stdout pipe broken (cannot send response) | Prism detects broken pipe, initiates graceful shutdown |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| FM-011 | Client disconnects mid-query | In-flight HTTP requests continue; responses cannot be delivered; graceful shutdown initiates |
| EC-10-010 | Prism binary launched without stdin connected (e.g., double-click on macOS) | Immediate stdin read error; Prism exits with error message to stderr |
| EC-10-011 | Very large MCP response (>1MB of sensor data) | Response is written as a single JSON-RPC message; no chunking at the MCP level; pagination keeps individual responses bounded |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| MCP client connects via stdio | JSON-RPC messages over stdin/stdout; no non-MCP content on stdout | happy-path |
| stdin pipe broken mid-session | Graceful shutdown sequence initiated | error |
| Very large query response (>1MB) | Single JSON-RPC message; no mid-message chunking | edge-case |

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vector tables.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (no matching VP) | stdout contains only valid JSON-RPC messages; no log lines | integration test |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-034 |
| L2 Invariants | DI-017 |
| L2 Failure Modes | FM-011 |
| Priority | P0 |

## Changelog
| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial draft |
