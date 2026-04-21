---
document_type: verification-property
level: L4
version: "1.3"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-1.05-ocsf-field-mapping.md]
input-hash: "019d284"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.02.007
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

# VP-017: OCSF Normalization — Unmapped Fields Preserved

## Property Statement

For every raw sensor record `r` and every field `f` in `r` that has no mapping to an
OCSF path, the normalized `DynamicMessage` contains `f` and its value under
`raw_extensions` (or equivalent preservation slot). No input field is silently
dropped; the union of mapped and preserved fields covers all input fields.

## Source Contract

- **Anchor Story:** `S-1.05`
- **Source BC:** BC-2.02.007 — Vendor Extension Preservation in raw_extensions
- **Module:** prism-ocsf
- **Category:** Correctness

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest (latest) | No — random records including unknown fields | Random mapped/unmapped mixtures |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_ocsf::mappers::<sensor>::normalize
//
// Sketch: generate raw record with known + unknown fields; normalize; assert
// every unknown field appears in raw_extensions with its original value.
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | No | Size-bounded JSON strategies |
| Tool support? | Full | proptest |
| Execution time budget | <60 seconds for 10k cases | Lightweight field scan |
| Assumptions required | Per-sensor mapping tables stable during test | Load pinned mapping |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.3 | pass-88-remediation | 2026-04-21 | architect | F88-012: Anchor Story normalized from slug form (S-1.05-ocsf-field-mapping.md) to pure ID (S-1.05). |
| 1.2 | pass-87-remediation | 2026-04-21 | architect | F87-006: Source BC label corrected "Unmapped vendor fields preserved in raw_extensions" → "Vendor Extension Preservation in raw_extensions" (matches BC-2.02.007 H1). |
| 1.1 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
