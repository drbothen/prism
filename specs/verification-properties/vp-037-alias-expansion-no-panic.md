---
document_type: verification-property
level: L4
version: "1.2"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-3.04-alias-system.md]
input-hash: "f29bec4"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.11.008
module: prism-query
priority: P1
proof_method: fuzz
verification_method: fuzz
feasibility: high
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

# VP-037: Alias Expansion Never Panics on Arbitrary Alias Graphs

## Property Statement

For every byte sequence `b` interpreted as an alias-map + query pair, `expand_aliases`
returns `Ok` or `Err` in bounded time without panicking. Cyclic graphs, depth blowups,
malformed references, and adversarial inputs must all produce structured errors —
never stack overflow, panic, or infinite loop.

## Source Contract

- **Anchor Story:** `S-3.04-alias-system.md`
- **Source BC:** BC-2.11.008 — `create_alias` MCP Tool
- **Module:** prism-query
- **Category:** Safety

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| fuzz | cargo-fuzz (libFuzzer) | No — coverage-guided mutation | Continuous corpus expansion |

## Proof Harness Skeleton

```rust
// [TODO: fuzz target skeleton — author during Phase 5 formal-verify]
// prism-query/fuzz/fuzz_targets/fuzz_alias_expand.rs
//
// fuzz_target!(|data: &[u8]| {
//     if let Some((map, query)) = decode_alias_input(data) {
//         let _ = expand_aliases(&map, &query);
//     }
// });
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | No | Coverage-guided |
| Tool support? | Full | libFuzzer + ASan |
| Execution time budget | 30 min initial, continuous in CI | Graph expanders fuzz well |
| Assumptions required | None | Panic = failure |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.2 | pass-87-remediation | 2026-04-21 | architect | F87-006: Source BC label corrected "Alias Expansion with Cycle Detection" → "`create_alias` MCP Tool" (matches BC-2.11.008 H1). |
| 1.1 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
