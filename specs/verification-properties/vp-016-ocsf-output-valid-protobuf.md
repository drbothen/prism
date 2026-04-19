---
document_type: verification-property
level: L4
version: "1.0"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-1.04-ocsf-schema-loading.md]
input-hash: "b05d905"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.02.002
module: prism-ocsf
proof_method: proptest
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

- **Anchor Story:** `S-1.04-ocsf-schema-loading.md`
- **Source BC:** BC-2.02.002 — OcsfNormalizer produces valid DynamicMessage
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
