---
document_type: story
story_id: S-MCP-STREAMABLE-HTTP-TRANSPORT-001
title: "MCP Streamable HTTP Transport — Day-2 Feature Post-Demo"
wave: null
# SCHEDULING NOTE: Day-2 post-demo roadmap item. DO NOT schedule until T13 demo recordings
# complete and the human authorizes. Depends on demo completion (T14 recording), not a
# specific merged story. F1 delta-analysis pass required before TDD dispatch — many open
# design questions in §Open Questions section below.
target_module: prism-mcp
subsystems: [SS-10]
priority: P2
# P2: Day-2 feature — NOT demo-blocking. Post-demo roadmap.
depends_on: []
# Scheduling dependency is T14 demo recording completion (a milestone, not a story ID).
# See scheduling note above. No hard code dependency on any current story —
# rmcp 1.7 already provides Streamable HTTP transport support alongside stdio.
blocks: []
estimated_days: null
# TBD after F1 delta-analysis — session management and auth model are open questions
# that affect implementation scope significantly.
points: null
# TBD after F1 delta-analysis.
level: "L4"
status: draft
# BC status: behavioral_contracts: [] — no BCs authored yet. Status remains draft.
# Spec-First Gate S-7.01: NOT ready for dispatch until BCs are authored.
# # BC status: pending PO authorship
version: "1.0"
updated: "2026-06-24"
producer: story-writer
timestamp: "2026-06-24T00:00:00Z"
input-hash: "TBD"
inputs:
  - ".factory/research/demo-pre-flight-audit-2026-06-24.md"
traces_to: []
cycle: "v1.0.0-greenfield"
epic_id: "E-5"
phase: 2
acceptance_criteria_count: 0
# ACs to be authored during F1 delta-analysis after open questions are resolved.
red_gate_tests: 0
tdd_mode: strict
behavioral_contracts: []
# # BC status: pending PO authorship
# behavioral_contracts must be non-empty before status=ready (Spec-First Gate S-7.01).
verification_properties: []
assumption_validations: []
risk_mitigations: []
crates_touched:
  - prism-mcp
  - prism-bin
  # prism-bin will need a --transport flag or config-driven transport selection.
---

# S-MCP-STREAMABLE-HTTP-TRANSPORT-001: MCP Streamable HTTP Transport (Day-2)

## Narrative

As a Prism operator deploying to a remote or shared environment, I want to connect to the
Prism MCP server over HTTP (Streamable HTTP transport) in addition to the existing stdio
transport, so that remote clients, browser-based tooling, and multi-analyst setups can
connect without requiring a local process fork per client.

## Scope

Add the MCP Streamable HTTP transport path (as defined in the MCP specification) alongside
the existing stdio transport in `prism-mcp`. The stdio transport continues to be the default
and primary transport. HTTP transport is opt-in via configuration or CLI flag.

The rmcp 1.7 crate already provides Streamable HTTP transport support — this story wires it
into `prism-bin`'s server boot path.

**This is a day-2 feature.** It requires a full F1 delta-analysis pass before TDD dispatch
because several design questions (see §Open Questions) must be resolved to produce
production-grade BCs. Do NOT over-spec this story at draft time.

---

## Open Questions for F1 Delta-Analysis

The following design questions MUST be resolved by the F1 delta-analysis (architect +
product-owner) before this story can be promoted to `ready` status. These are not
implementation questions — they require architectural and product decisions.

### OQ-1: Session Management

**Question:** How does the HTTP transport manage session state? The stdio transport has
implicit single-session semantics (one process = one analyst). The HTTP transport may serve
multiple concurrent connections.

Options:
a. **Session-per-connection** (stateless server): Each HTTP connection is an independent
   MCP session. No shared state between connections. Simplest to implement; matches stdio
   semantics per-connection.
b. **Named sessions with explicit lifetime**: Each HTTP session has a UUID; `prism-bin`
   tracks active sessions; sessions expire after a configurable idle timeout.
c. **Single-session HTTP** (reject concurrent connections): Only one HTTP connection
   allowed at a time; second connection receives a 503. Preserves single-session semantics.

The choice affects audit trail design (BC-2.16.001 audit completeness), org-scope
isolation (each session may be a different analyst), and resource limits.

### OQ-2: Authentication Model for Network-Exposed HTTP

**Question:** The stdio transport has implicit trust (the analyst who starts the process
controls it). A network-exposed HTTP endpoint requires explicit authentication.

Options:
a. **API key in header** (simplest): `Authorization: Bearer <key>`. Key configured in
   `prism.toml`. Does not integrate with existing credential store.
b. **mTLS client cert**: Strong but complex to provision.
c. **Same-host-only** (no auth, localhost-only bind): HTTP transport bound to `127.0.0.1`
   only; no auth needed (same threat model as stdio).

The auth model affects whether existing prompt-injection defenses (DI-019, AD-017) are
sufficient or need augmentation for network-sourced requests.

### OQ-3: Supplement vs Replace stdio

**Question:** Can both transports run simultaneously in one `prism` process, or is it
transport-at-startup (one or the other)?

- **Simultaneous transports**: one process serves both stdio and HTTP. Useful for tooling
  that needs both. Requires thread-safe server state (already Arc-DI'd).
- **Transport-at-startup** (exclusive): `prism start --transport http` or
  `prism start --transport stdio` (default). Simpler; no simultaneous-transport complexity.

### OQ-4: Multi-Analyst / Remote Implications

**Question:** Is the HTTP transport intended for multi-analyst use (multiple concurrent
analysts hitting the same `prism` instance) or single-analyst-remote (one analyst,
accessing remotely instead of locally)?

- Multi-analyst use requires per-session org-scope isolation (each connection may have
  different `clients` parameter context).
- Single-analyst-remote is simpler: same semantics as stdio, just over TCP.

The answer affects whether org-scope state is per-connection or global, and whether the
existing `clients` tool parameter model is sufficient.

### OQ-5: Prompt-Injection Surface

**Question:** HTTP-sourced requests have a larger prompt-injection attack surface than
stdio (which is controlled by the local analyst). Does the network-exposed HTTP path
require additional prompt-injection scanning beyond DI-019?

- If HTTP transport is localhost-only (OQ-2 option c), attack surface is equivalent to
  stdio — no additional defenses needed.
- If HTTP transport is network-accessible, the prompt-injection defense layer needs a
  threat model review (AD-017 + DI-019 scope).

---

## Acceptance Criteria

> ACs to be authored during F1 delta-analysis after open questions OQ-1 through OQ-5
> are resolved. Placeholder section — DO NOT implement before ACs are authored and BCs
> are registered.

(TBD at F1 delta-analysis)

---

## Architecture Mapping

| Component | File | Pure/Effectful |
|-----------|------|----------------|
| HTTP transport bind | `crates/prism-bin/src/main.rs` (or `boot.rs`) | Effectful |
| Transport selection config/flag | `crates/prism-bin/src/config.rs` | Pure |
| `PrismServer` — HTTP handler registration | `crates/prism-mcp/src/server.rs` | Effectful |

Note: rmcp 1.7 provides `rmcp::transport::SseServer` or `StreamableHttpServer` — confirm
the exact rmcp API for Streamable HTTP transport during F1 delta-analysis.

---

## Tasks

> Tasks to be authored during F1 delta-analysis. The implementation sequence depends on
> the answers to OQ-1 through OQ-5.

(TBD at F1 delta-analysis)

---

## Token Budget Estimate

> TBD after scope is defined. Expected range: 5–8 story points depending on session
> management and auth model choices.

---

## Previous Story Intelligence

- N/A — this is the first story exploring HTTP transport for Prism MCP. No prior art in
  the codebase beyond the existing stdio transport wiring.

---

## Architecture Compliance Rules

From ADR-022 §C (Arc-DI wiring contract):
- The MCP server is already Arc-DI'd (`PrismServer` holds `Arc<_>` fields). The HTTP
  transport addition MUST NOT bypass existing Arc-DI wiring.

From AD-017 (credential safety):
- HTTP transport implementation MUST NOT allow credentials or auth tokens to be echoed
  in error responses or log output. Existing redacted-Debug discipline applies.

---

## Library & Framework Requirements

| Library | Version | Usage | Notes |
|---------|---------|-------|-------|
| `rmcp` | `1.7` | Streamable HTTP transport | Confirm HTTP transport API in rmcp 1.7 docs during F1 |

---

## File Structure Requirements

> TBD after F1 delta-analysis. Expected changes: `prism-bin/src/main.rs` or `boot.rs`
> for transport selection; `prism-mcp/src/server.rs` for HTTP handler registration.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | demo-readiness-remediation-2026-06-24 | 2026-06-24 | story-writer | Initial draft stub. Day-2 roadmap item per human directive. Open questions captured for F1 delta-analysis. No BCs, no ACs — NOT ready for dispatch. |
