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

Prism is a Rust MCP server that unifies multi-client security sensor management for MSSP analysts, exposing CrowdStrike, Cyberint, Claroty, and Armis data through AI-consumable tools with OCSF normalization, feature-flagged write operations, and prompt injection defense. The domain model encompasses sensor adapters with per-client credential isolation, cursor-based state management with forward-progress invariants, a two-tier feature flag system gating dangerous operations, and a four-layer sanitization pipeline for attacker-controlled content flowing through LLM context.

## Document Map

| Section | File | Est. Tokens | Primary Consumer | Purpose |
|---------|------|-------------|-----------------|---------|
| Capabilities | capabilities.md | ~1100 | PRD Author, Architect | Enumerates all domain capabilities (CAP-001 through CAP-013) with business rules and priority |
| Entities | entities.md | ~1200 | Architect, Implementer | Defines 14 domain entities with key attributes and invariants |
| Invariants | invariants.md | ~1100 | Architect, Test Writer | Specifies 14 domain rules (DI-001 through DI-014) that must always hold with violation behavior |
| Events | events.md | ~1100 | Architect, Implementer | Documents 10 processing stages from tool invocation through audit emission |
| Edge Cases | edge-cases.md | ~1200 | Test Writer, Implementer | Specifies expected behavior for 15 boundary scenarios (DEC-001 through DEC-015) |
| Assumptions | assumptions.md | ~1000 | Product Owner, Architect | Lists 10 assumptions (ASM-001 through ASM-010) requiring validation with impact analysis |
| Risks | risks.md | ~1100 | Product Owner, Architect | Risk register with 12 entries (R-001 through R-012) including mitigations |
| Failure Modes | failure-modes.md | ~1100 | Implementer, SRE | Documents 12 runtime failure modes (FM-001 through FM-012) with detection and recovery |
| Differentiators | differentiators.md | ~1000 | Product Owner, Stakeholders | Maps 8 competitive differentiators to supporting capabilities |
| Index | L2-INDEX.md | ~500 | All consumers | Navigation, cross-references, and ID registry |

## Cross-References

| Source ID | Target IDs | Relationship |
|-----------|-----------|-------------|
| CAP-001 | DI-001, DI-008, DEC-001, DEC-010, DEC-013, FM-001, FM-006, R-003 | Sensor query capability constrained by cursor invariant, client separation; edge cases for failures; risks from API changes |
| CAP-002 | DEC-003, DEC-005, DI-008, R-007 | Cross-client query edge cases for partial failures and missing sensors; data mixing risk |
| CAP-003 | DI-005, DEC-007, DEC-015, ASM-002, ASM-005, ASM-010, R-004, FM-005 | OCSF normalization constrained by schema validity; edge cases for unmappable fields; risks from schema instability |
| CAP-004 | DI-002, DI-014, DEC-011, ASM-003, R-006, FM-004 | Credential management constrained by isolation and sanitization invariants; keyring availability edge case |
| CAP-005 | DI-003, DEC-006, ASM-001, R-001 | Feature flags constrained by deny-by-default; edge case for config change during session |
| CAP-006 | DI-007, DEC-009, R-012, FM-007 | Write gating constrained by token expiry; edge case for expired tokens; replay risk |
| CAP-007 | DI-004, DEC-014, R-005 | Audit logging constrained by completeness invariant; edge case for logging failure |
| CAP-010 | DI-006, DEC-008, R-005 | Prompt injection defense constrained by sanitization invariant; edge case for hostile hostnames |
| CAP-008 | DEC-001, FM-001, FM-002, FM-010, R-003, R-010 | Sensor health monitoring detects unreachable sensors, expired auth, rate limiting; risks from API changes |
| CAP-009 | DI-002, DI-003, DI-008, DEC-004, DEC-006, FM-010, R-007 | Client configuration constrained by credential isolation, deny-by-default flags, client separation; edge cases for zero-sensor and config changes |
| CAP-011 | DI-001, DI-009, DI-010, DI-011, DI-013, DEC-012, R-008, FM-003, FM-009 | Cursor state management constrained by forward progress, persistence ordering, fingerprint consistency, MemoryStore ban, atomic writes |
| CAP-012 | CAP-003, DI-005, DEC-003, DEC-005, ASM-002 | Cross-sensor correlation depends on OCSF normalization; edge cases for cross-client partial failures and mixed sensor availability |
| CAP-013 | DI-001, DI-009, DEC-001, FM-001, FM-006 | xMP delivery depends on cursor state and sensor query; failure modes for unreachable sinks and malformed responses |

## ID Registry Summary

| ID Format | Range | Count | Section |
|-----------|-------|-------|---------|
| CAP-NNN | CAP-001 to CAP-013 | 13 | capabilities.md |
| DI-NNN | DI-001 to DI-014 | 14 | invariants.md |
| DEC-NNN | DEC-001 to DEC-015 | 15 | edge-cases.md |
| ASM-NNN | ASM-001 to ASM-010 | 10 | assumptions.md |
| R-NNN | R-001 to R-012 | 12 | risks.md |
| FM-NNN | FM-001 to FM-012 | 12 | failure-modes.md |
| **Total** | | **76** | |

## Priority Distribution

| Priority | Capabilities | Description |
|----------|-------------|-------------|
| P0 | CAP-001, CAP-002, CAP-003, CAP-004, CAP-005, CAP-007, CAP-009, CAP-010, CAP-011 | Core query pipeline, credential management, feature flags, audit, config, prompt injection defense, cursor state -- required for MVP |
| P1 | CAP-006, CAP-008, CAP-012, CAP-013 | Write operation gating, sensor health, cross-sensor correlation, xMP delivery -- required for full launch |
| P2 | (none defined) | Post-launch enhancements will be identified during PRD phase |

**P0 count:** 9 capabilities (69%)
**P1 count:** 4 capabilities (31%)
**P2 count:** 0 capabilities
