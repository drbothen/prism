---
document_type: verification-property
level: L4
version: "1.1"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-4.03-detection-rules.md]
input-hash: "5ce5fe8"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.13.009
module: prism-operations
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

# VP-018: Detection Rule Validation — Rejects Invalid Rules

## Property Statement

For every proposed detection rule `r`, `validate_rule(r)` returns `Ok` only if `r`
satisfies every structural and semantic constraint (known mode, valid query,
bounded window, referenced fields exist, thresholds within range). Any violation
produces a typed `Err(RuleValidationError)` and the rule is not registered.

## Source Contract

- **Anchor Story:** `S-4.03-detection-rules.md`
- **Source BC:** BC-2.13.009 — Detection Rule Schema Validation
- **Module:** prism-operations
- **Category:** Correctness

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest (latest) | No — random rule ASTs | Valid + malformed rules across modes |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_operations::detections::validate_rule
//
// Sketch: generate arbitrary rule structs; assert validator's Ok/Err classification
// matches an independent oracle derived from the validation spec.
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | No | Size-bounded strategies for rule tree |
| Tool support? | Full | proptest |
| Execution time budget | <60 seconds for 10k cases | Validator is fast |
| Assumptions required | Oracle model mirrors spec exactly | Oracle co-maintained with validator |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.1 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
