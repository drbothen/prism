---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "MCP Interface"
capability: "CAP-034"
---

# BC-2.10.006: Stdio Transport

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

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-034 |
| L2 Failure Modes | FM-011 |
| Priority | P0 |
