---
document_type: verification-property
level: L4
version: "1.0"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-3.04-alias-system.md]
input-hash: "c67377c"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.11.009
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

# VP-012: Alias Depth — Rejects Composition Beyond Depth 3

## Property Statement

For every alias-expansion call, if the alias reference graph requires traversing more
than 3 nested alias definitions to resolve, `expand_aliases` returns
`Err(AliasError::DepthExceeded)` without producing an expanded query. Depth 3 is the
hard ceiling; depth 4 and beyond is rejected.

## Source Contract

- **Anchor Story:** `S-3.04-alias-system.md`
- **Source BC:** BC-2.11.009 — Alias Composition Depth Limit
- **Module:** prism-query
- **Category:** Security

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| kani | Kani (latest) | Yes — alias graphs up to 5 levels | All depth configurations including boundary |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: kani
// Target: prism_query::alias::expand_aliases
//
// Sketch: construct chain a1 -> a2 -> a3 -> a4 (depth 4); assert expand
// returns Err(DepthExceeded). At depth 3 assert Ok.
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | Alias graph with 5 entries |
| Tool support? | Full | Kani handles bounded recursion with unwind |
| Execution time budget | <5 minutes | Short chains |
| Assumptions required | Acyclic graph (cycle detection in VP-013) | Separation of concerns |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |
