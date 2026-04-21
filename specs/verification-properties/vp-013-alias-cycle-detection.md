---
document_type: verification-property
level: L4
version: "1.3"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-3.04-alias-system.md]
input-hash: "bb78cbb"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.11.008
module: prism-query
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

# VP-013: Alias Cycles — Detects and Rejects Cyclic References

## Property Statement

For every alias map, `expand_aliases` terminates without stack overflow or infinite
loop, and returns `Err(AliasError::CycleDetected)` whenever the alias reference graph
contains a cycle reachable from the input query. Expansion of alias-free inputs
succeeds; expansion of any cyclic input fails in bounded time.

## Source Contract

- **Anchor Story:** `S-3.04`
- **Source BC:** BC-2.11.008 — `create_alias` MCP Tool
- **Module:** prism-query
- **Category:** Safety

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest (latest) | No — random alias graphs | Millions of random maps including self-loops, mutual, chained cycles |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_query::alias::expand_aliases
//
// Sketch: generate arbitrary alias maps with possible cycles; assert expand
// always terminates; if cycle present assert Err(CycleDetected); else Ok or
// depth/other legitimate error.
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | No | Random graphs, size bound per-strategy |
| Tool support? | Full | proptest + custom graph strategy |
| Execution time budget | <30 seconds per 10k cases | Lightweight graph ops |
| Assumptions required | None | Runtime check |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.3 | pass-88-remediation | 2026-04-21 | architect | F88-012: Anchor Story normalized from slug form (S-3.04-alias-system.md) to pure ID (S-3.04). |
| 1.2 | pass-87-remediation | 2026-04-21 | architect | F87-006: Source BC label corrected "Alias Expansion with Cycle Detection" → "`create_alias` MCP Tool" (matches BC-2.11.008 H1). |
| 1.1 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
