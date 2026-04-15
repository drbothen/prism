---
document_type: verification-property-index
level: L4
version: "1.0"
status: draft
producer: architect
timestamp: 2026-04-15T12:00:00
phase: 1b
inputs: [architecture/verification-architecture.md]
traces_to: architecture/ARCH-INDEX.md
---

# Verification Property Index: Prism

> **Context Engineering:** This index lists all verification properties with their
> status and method. Load individual VP files only when working on that specific property.

## Properties

| ID | Property | Module | Method | Priority | Status |
|----|----------|--------|--------|----------|--------|
| VP-001 | TenantId rejects invalid characters | prism-core | kani | P0 | draft |
| VP-002 | Capability resolution: deny-by-default | prism-core | kani | P0 | draft |
| VP-003 | Capability resolution: most-specific-path wins | prism-core | kani | P0 | draft |
| VP-004 | Capability resolution: deny overrides allow at same specificity | prism-core | kani | P0 | draft |
| VP-005 | Case state machine: exactly 12 valid transitions | prism-core | kani | P0 | draft |
| VP-006 | Case state machine: no self-transitions | prism-core | kani | P0 | draft |
| VP-007 | Confirmation token expiry: expired at boundary (inclusive) | prism-security | kani | P0 | draft |
| VP-008 | Confirmation token: single-use enforcement | prism-security | kani | P0 | draft |
| VP-009 | Confirmation token: content hash mismatch rejects | prism-security | kani | P0 | draft |
| VP-010 | Token cap: store rejects at 100 active tokens | prism-security | kani | P0 | draft |
| VP-011 | Credential name sanitization: rejects path traversal | prism-core | kani | P0 | draft |
| VP-012 | Alias depth: rejects composition beyond depth 3 | prism-query | kani | P0 | draft |
| VP-013 | Alias cycles: detects and rejects cyclic references | prism-query | proptest | P0 | draft |
| VP-014 | Query security limits: rejects oversized queries | prism-query | kani | P0 | draft |
| VP-015 | Query security limits: rejects excessive nesting depth | prism-query | kani | P0 | draft |
| VP-016 | OCSF normalization: output is valid protobuf | prism-ocsf | proptest | P0 | draft |
| VP-017 | OCSF normalization: unmapped fields preserved | prism-ocsf | proptest | P0 | draft |
| VP-018 | Detection rule validation: rejects invalid rules | prism-operations | proptest | P0 | draft |
| VP-019 | Diff computation: deterministic | prism-operations | proptest | P0 | draft |
| VP-020 | Feature flag: compile AND runtime must both permit | prism-security | kani | P0 | draft |
| VP-021 | AxiQL parser: never panics on arbitrary input | prism-query | fuzz | P0 | draft |
| VP-022 | OCSF normalizer: never panics on arbitrary input | prism-ocsf | fuzz | P0 | draft |
| VP-023 | Sensor spec parser: never panics on arbitrary TOML | prism-spec-engine | fuzz | P0 | draft |
| VP-024 | Injection scanner: detects known injection patterns | prism-security | proptest | P0 | draft |
| VP-025 | Cache key derivation: deterministic | prism-query | kani | P1 | draft |
| VP-026 | Splay computation: deterministic per (query, client) | prism-operations | kani | P1 | draft |
| VP-027 | Alert dedup key: correct per match mode | prism-operations | proptest | P0 | draft |
| VP-028 | Template interpolation: never panics | prism-operations | fuzz | P0 | draft |
| VP-029 | Cursor cap: rejects at 200 active | prism-core | kani | P1 | draft |
| VP-030 | Schedule/rule count caps: rejects beyond limits | prism-operations | kani | P1 | draft |
| VP-031 | Required column enforcement: rejects unconstrained | prism-query | proptest | P0 | draft |
| VP-032 | Hot reload atomicity: failed validation retains old config | prism-spec-engine | proptest | P1 | draft |

## Summary

| Method | Count | P0 | P1 |
|--------|-------|----|----|
| Kani | 15 | 13 | 2 |
| Proptest | 11 | 9 | 2 |
| Fuzz | 5 | 5 | 0 |
| Cargo-mutants | — | — | — |
| **Total** | **32** | **27** | **5** |
