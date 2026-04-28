---
document_type: verification-property
level: L4
version: "1.3"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-1.04-ocsf-schema-loading.md]
input-hash: "5d1a873"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.02.002
module: prism-ocsf
priority: P0
proof_method: proptest
verification_method: proptest
feasibility: medium
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-016: OCSF Normalization — Output Is Valid Protobuf

## Property Statement

For every raw sensor record successfully processed by `OcsfNormalizer::normalize`,
the resulting `DynamicMessage` serializes to a byte sequence that round-trips
through `prost`/`protox` decode into an equivalent `DynamicMessage` (same fields,
same values). The normalizer never produces ill-formed protobuf.

## Source Contract

- **Anchor Story:** `S-1.04`
- **Source BC:** BC-2.02.002 — DynamicMessage Creation from Sensor Records
- **Module:** prism-ocsf
- **Category:** Correctness

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest (latest) | No — generated raw records | Random valid records across event classes |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_ocsf::OcsfNormalizer::normalize
//
// Sketch: generate arbitrary sensor records; normalize to DynamicMessage;
// encode -> bytes -> decode; assert decoded message equals original.
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | No | Size-bounded record strategies |
| Tool support? | Full | proptest + prost round-trip |
| Execution time budget | <60 seconds for 10k cases | Serialization overhead moderate |
| Assumptions required | None | Pure property test |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.3 | pass-88-remediation | 2026-04-21 | architect | F88-012: Anchor Story normalized from slug form (S-1.04-ocsf-schema-loading.md) to pure ID (S-1.04). |
| 1.2 | pass-87-remediation | 2026-04-21 | architect | F87-006: Source BC label corrected "OcsfNormalizer produces valid DynamicMessage" → "DynamicMessage Creation from Sensor Records" (matches BC-2.02.002 H1). |
| 1.1 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
