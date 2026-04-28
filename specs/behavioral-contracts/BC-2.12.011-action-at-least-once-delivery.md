---
document_type: behavioral-contract
level: L3
version: "1.5"
status: retired
lifecycle_status: retired
producer: product-owner
timestamp: 2026-04-16T22:00:00
phase: 2-patch
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "412c872"
traces_to: []
extracted_from: "[tombstone]"
origin: greenfield
subsystem: "SS-12"
capability: "CAP-021"
introduced: cycle-1
modified: [phase-2-patch-burst-51]
deprecated: "2026-04-16"
deprecated_by: "v3-patch-burst-4b"
retired: "2026-04-16"
removed: null
removal_reason: null
replacement: "BC-2.18.001"
---

> **RETIRED (2026-04-16):** Superseded by BC-2.18.001 (Action Delivery Engine subsystem, INV-ACTION-001).
> BC-2.12.011 was a high-level cross-subsystem summary written before subsystem 18 was established.
> BC-2.18.001 is the normative specification. In all conflicts, BC-2.18.001 wins.
> This file is retained for historical traceability only.

# BC-2.12.011: Action At-Least-Once Delivery with Retry

## Description

RETIRED. This contract specified that action delivery (webhook, email, script) must be
attempted at least once per trigger event, with exponential-backoff retry on transient
failure and dead-letter persistence on permanent failure. It was written as a
cross-subsystem summary before SS-18 (Action Delivery Engine) was established.
The normative successor is BC-2.18.001.

## Preconditions
- Tombstone — no preconditions apply. See BC-2.18.001 for the active specification.

## Postconditions
- Tombstone — no postconditions apply. See BC-2.18.001 for the active specification.

## Invariants
- Tombstone — no invariants apply. See BC-2.18.001 for the active specification.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | RETIRED — see BC-2.18.001 | n/a |

## Canonical Test Vectors

> RETIRED — see BC-2.18.001 for active test vectors.

| Input | Expected Output | Category |
|-------|----------------|----------|
| RETIRED | see BC-2.18.001 | n/a |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | RETIRED — see BC-2.18.001 | n/a |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-021 |
| L2 Domain Invariants | DI-002, DI-004 |
| Architecture Module | SS-12 (historical); normative owner: SS-18 Action Delivery Engine |
| Stories | — (retired before story assignment) |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.5 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.4 | pass-63-fix | — | P3P63-A-MED-001 | Aligned Changelog rows 1.2 and 1.3 to canonical 4-column schema (Version \| Burst \| Finding \| Change). Row 1.2 Burst/Date swap and stale 1.0→1.1 version-bump text corrected. Row 1.3 collapsed from 5-value to 4-column schema with proper Finding citation. |
| 1.3 | pass-62-fix | — | P3P62-A-MED-001 | Renumbered duplicate 1.0 Changelog rows for monotonicity (pass-62 MED-001; status=retired BC scope gap from pass-61 Track B's removed-only filter). |
| 1.2 | pre-build-sweep | — | — | Template-compliance sweep: standardized inputs/input-hash/traces_to/extracted_from frontmatter to Wave 4 convention. |
| 1.1 | Burst 51 | — | P3P50-A-MED-001 | Frontmatter `status: removed` corrected to `status: retired` — 3-way consistency fix (lifecycle_status, body RETIRED prose, and BC-INDEX Status col all canonical as `retired`); no semantic change. Template conformance fields added. |
| 1.0 | cycle-1 / Burst 4b | — | — | Created as cross-subsystem summary for Action Delivery; retired 2026-04-16 when SS-18 (Action Delivery Engine) established; superseded by BC-2.18.001 |
