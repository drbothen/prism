---
document_type: verification-property
level: L4
version: "1.0"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-3.01-prismql-parser.md]
input-hash: "f9c43ae"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.11.006
module: prism-query
proof_method: kani
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

# VP-014: Query Security Limits — Rejects Oversized Queries

## Property Statement

For every query input byte sequence `b`, if `b.len() > 65536` (64 KiB), then
`AxiqlParser::parse(b)` returns `Err(ParseError::QueryTooLarge)` before performing
any lexical or syntactic work. The byte-length check is the first gate in the parse
pipeline.

## Source Contract

- **Anchor Story:** `S-3.01-prismql-parser.md`
- **Source BC:** BC-2.11.006 — Query Security Limits Enforcement
- **Module:** prism-query
- **Category:** Security

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| kani | Kani (latest) | Yes — scaled limit (e.g. 64 bytes) | Boundary above/below scaled limit |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: kani
// Target: prism_query::parser::AxiqlParser::parse
//
// Sketch: with MAX_BYTES scaled down for proof, generate inputs of length
// MAX_BYTES-1, MAX_BYTES, MAX_BYTES+1; assert only the first two may succeed
// and the third returns QueryTooLarge.
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | Scaled MAX_BYTES constant |
| Tool support? | Full | Kani handles length checks |
| Execution time budget | <3 minutes | Small scaled bound |
| Assumptions required | Scaling argument generalizes to 64 KiB | Documented in harness |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |
