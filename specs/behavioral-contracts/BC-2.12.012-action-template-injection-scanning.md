---
document_type: behavioral-contract
level: L3
version: "1.3"
status: retired
lifecycle_status: retired
producer: product-owner
timestamp: 2026-04-16T22:00:00
phase: 2-patch
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "c36ec87"
traces_to: []
extracted_from: "[tombstone]"
origin: greenfield
subsystem: "SS-12"
capability: "CAP-021"
introduced: cycle-1
modified: null
deprecated: "2026-04-16"
deprecated_by: "v3-patch-burst-4b"
retired: "2026-04-16"
removed: null
removal_reason: null
replacement: "BC-2.18.006"
---

> **RETIRED (2026-04-16):** Superseded by BC-2.18.006 (Action Delivery Engine subsystem, INV-ACTION-006).
> BC-2.12.012 was a high-level cross-subsystem summary written before subsystem 18 was established.
> BC-2.18.006 is the normative specification. In all conflicts, BC-2.18.006 wins.
> This file is retained for historical traceability only.

# BC-2.12.012: Action Template Injection Scanning

## Description

RETIRED. This contract specified that action template variables containing untrusted data from sensor events or alert fields must be scanned by InjectionScanner (BC-2.09.003) before interpolation, with safety flags included in delivery payload metadata. It was written as a cross-subsystem summary before SS-18 (Action Delivery Engine) was established. The normative successor is BC-2.18.006.

## Preconditions
- Tombstone — no preconditions apply. See BC-2.18.006 for the active specification.

## Postconditions
- Tombstone — no postconditions apply. See BC-2.18.006 for the active specification.

## Invariants
- Tombstone — no invariants apply. See BC-2.18.006 for the active specification.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | RETIRED — see BC-2.18.006 | n/a |

## Canonical Test Vectors

> RETIRED — see BC-2.18.006 for active test vectors.

| Input | Expected Output | Category |
|-------|----------------|----------|
| RETIRED | see BC-2.18.006 | n/a |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | RETIRED — see BC-2.18.006 | n/a |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-021 |
| L2 Domain Invariants | DI-006 |
| Architecture Module | SS-12 (historical); normative owner: SS-18 Action Delivery Engine |
| Stories | — (retired before story assignment) |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.2 | pass-64-fix | — | P3P64-A-LOW-001 | Realigned row 1.1 columns to match 4-col schema (Burst/Finding columns were swapped). Pass-63 verification of BC-2.12.012 had been incorrect. |
| 1.1 | pre-build-sweep | — | — | Template-compliance sweep (Wave 4): full tombstone treatment — added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description, ## Preconditions, ## Postconditions, ## Invariants, ## Edge Cases, ## Canonical Test Vectors, ## Verification Properties, ## Traceability, ## Changelog stub sections; version 1.0 → 1.1. |
| 1.0 | cycle-1 / Burst 4b | — | — | Created as cross-subsystem summary for Action Template Injection Scanning; retired 2026-04-16 when SS-18 established; superseded by BC-2.18.006 |
