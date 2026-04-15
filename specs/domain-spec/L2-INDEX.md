---
document_type: domain-spec-section
level: L2
section: "index"
version: "1.0"
status: draft
producer: business-analyst
timestamp: 2026-04-14T04:00:00
phase: 1a
inputs: [product-brief.md]
input-hash: "be246a0"
traces_to: L2-INDEX.md
---

# L2 Domain Specification Index -- Prism

## Domain Summary

Prism is a Rust MCP server that unifies multi-client security sensor management for MSSP analysts. The query engine (CAP-015) is the sole data access interface -- all sensor data is accessed via AxiQL queries, not per-sensor read tools. The query engine orchestrates live API calls through an internal sensor adapter layer (CAP-001), normalizes all responses to OCSF, materializes ephemeral Arrow tables, and executes queries via DataFusion. Per-sensor MCP tools exist only for write operations (containment, acknowledgment, device actions), gated behind a two-tier feature flag system. The domain model encompasses the query engine with filter/SQL/pipe modes (Chumsky + DataFusion), an internal sensor adapter layer with per-client credential isolation and ephemeral cursor-based pagination (never exposed to MCP clients), response caching with configurable TTL, bounded confirmation tokens, a four-layer sanitization pipeline for attacker-controlled content, and a composable query alias system. The MCP tool surface is approximately 15 tools.

## Document Map

| Section | File | Est. Tokens | Primary Consumer | Purpose |
|---------|------|-------------|-----------------|---------|
| Architecture Concept | architecture-concept.md | ~2500 | All consumers, New contributors | Explains the core architectural concept (ephemeral federated query engine), query flow, and comparisons with SIEM/Trino/direct API access |
| Capabilities | capabilities.md | ~1800 | PRD Author, Architect | Enumerates all domain capabilities (CAP-001 through CAP-016, CAP-013 removed); CAP-001/002/011/012 are internal capabilities consumed by the query engine, not MCP-facing |
| Entities | entities.md | ~1900 | Architect, Implementer | Defines 17 domain entities (QueryFingerprint removed; CacheEntry, QueryPlan, MaterializedTable, Alias added) with key attributes and invariants |
| Invariants | invariants.md | ~1700 | Architect, Test Writer | Specifies 21 domain rules (DI-001 through DI-021; DI-009, DI-010, DI-011, DI-013 removed) that must always hold with violation behavior |
| Events | events.md | ~1100 | Architect, Implementer | Documents 10 processing stages from tool invocation through audit emission |
| Edge Cases | edge-cases.md | ~2000 | Test Writer, Implementer | Specifies expected behavior for 26 boundary scenarios (DEC-001 through DEC-027; DEC-012 removed) |
| Assumptions | assumptions.md | ~1000 | Product Owner, Architect | Lists 10 assumptions (ASM-001 through ASM-010) requiring validation with impact analysis |
| Risks | risks.md | ~1100 | Product Owner, Architect | Risk register with 12 entries (R-001 through R-012) including mitigations |
| Failure Modes | failure-modes.md | ~1100 | Implementer, SRE | Documents 12 runtime failure modes (FM-001 through FM-012) with detection and recovery |
| Differentiators | differentiators.md | ~1000 | Product Owner, Stakeholders | Maps 8 competitive differentiators to supporting capabilities |
| Index | L2-INDEX.md | ~500 | All consumers | Navigation, cross-references, and ID registry |


## Cross-References

| Source ID | Target IDs | Relationship |
|-----------|-----------|-------------|
| CAP-001 | DI-001, DI-008, DEC-001, DEC-010, DEC-013, FM-001, FM-006, R-003 | Sensor adapter layer (internal) constrained by cursor invariant, client separation; edge cases for failures; risks from API changes. Consumed by query engine (CAP-015), not exposed as MCP tools. |
| CAP-002 | DEC-003, DEC-005, DEC-020, DI-008, R-007 | Cross-client fan-out (internal to query engine) edge cases for partial failures, missing sensors, internal cursor cap pressure; data mixing risk |
| CAP-003 | DI-005, DEC-007, DEC-015, ASM-002, ASM-005, ASM-010, R-004, FM-005 | OCSF normalization constrained by schema validity; edge cases for unmappable fields; risks from schema instability |
| CAP-004 | DI-002, DI-014, DEC-011, ASM-003, R-006, FM-004 | Credential management constrained by isolation and sanitization invariants; keyring availability edge case |
| CAP-005 | DI-003, DEC-006, ASM-001, R-001 | Feature flags constrained by deny-by-default with deny override; edge case for config change during session |
| CAP-006 | DI-007, DI-015, DEC-009, DEC-016, DEC-017, DEC-019, DEC-021, R-012, FM-007 | Write gating constrained by token expiry and token cap; edge cases for expired/lost/capped tokens, concurrency, and combined cursor+token pressure; replay risk |
| CAP-007 | DI-004, DI-016, DEC-014, R-005 | Audit logging constrained by completeness invariant (split: reads proceed, writes fail-closed); edge case for logging failure |
| CAP-010 | DI-006, DEC-008, R-005 | Prompt injection defense constrained by sanitization invariant; edge case for hostile hostnames |
| CAP-008 | DEC-001, FM-001, FM-002, FM-010, R-003, R-010 | Sensor health monitoring detects unreachable sensors, expired auth, rate limiting; risks from API changes |
| CAP-009 | DI-002, DI-003, DI-008, DEC-004, DEC-006, FM-010, R-007 | Client configuration constrained by credential isolation, deny-by-default flags, client separation; edge cases for zero-sensor and config changes |
| CAP-011 | DI-001, DI-017, DEC-020, DEC-021, R-008 | Internal adapter pagination constrained by pagination validity and single-process invariant; edge cases for internal cursor cap pressure and combined cursor+token pressure. Cursors never exposed to MCP clients. |
| CAP-012 | CAP-003, CAP-015, DI-005, DEC-003, DEC-005, ASM-002 | Cross-sensor correlation is a natural consequence of the query engine's unified OCSF table; depends on OCSF normalization; edge cases for cross-client partial failures and mixed sensor availability |
| ~~CAP-013~~ | — | **REMOVED** — xMP backward compatibility not required |
| CAP-014 | DI-018, DEC-018, DEC-019 | Response caching constrained by cache bounds; edge cases for stale data after writes and concurrent access |
| CAP-015 | DI-019, DEC-022, DEC-023, DEC-026, CAP-003, CAP-014 | Ephemeral OCSF query engine constrained by security limits; edge cases for empty results, scope too broad, and timeout; depends on OCSF normalization and response cache for sensor fetch layer |
| CAP-016 | DI-020, DEC-024, DEC-025, CAP-015, BC-2.11.008, BC-2.11.009, BC-2.11.013, BC-2.11.014, BC-2.11.015 | Query aliases constrained by composition depth and cycle detection; edge cases for undefined alias references and cross-client alias gaps; aliases feed into the query engine; MCP tools for create, list, delete, and explain alias operations |

## ID Registry Summary

| ID Format | Range | Count | Section |
|-----------|-------|-------|---------|
| CAP-NNN | CAP-001 to CAP-016 (CAP-013 removed) | 15 | capabilities.md |
| DI-NNN | DI-001 to DI-021 (DI-009, DI-010, DI-011, DI-013 removed) | 17 | invariants.md |
| DEC-NNN | DEC-001 to DEC-027 (DEC-012 removed) | 26 | edge-cases.md |
| ASM-NNN | ASM-001 to ASM-010 | 10 | assumptions.md |
| R-NNN | R-001 to R-012 | 12 | risks.md |
| FM-NNN | FM-001 to FM-012 | 12 | failure-modes.md |
| **Total** | | **92** | |

## Priority Distribution

| Priority | Capabilities | Description |
|----------|-------------|-------------|
| P0 | CAP-001, CAP-002, CAP-003, CAP-004, CAP-005, CAP-007, CAP-009, CAP-010, CAP-011, CAP-015 | Query engine (sole data access), internal sensor adapter layer, internal adapter pagination, OCSF normalization, credential management, feature flags, audit, config, prompt injection defense -- required for MVP |
| P1 | CAP-006, CAP-008, CAP-012, CAP-014, CAP-016 | Write operation gating, sensor health, cross-sensor correlation (via query engine), response caching, query aliases -- required for full launch |
| P2 | (none defined) | Post-launch enhancements will be identified during PRD phase |

**P0 count:** 10 capabilities (67%)
**P1 count:** 5 capabilities (33%)
**P2 count:** 0 capabilities
