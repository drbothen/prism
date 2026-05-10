---
document_type: verification-property
level: L4
version: "1.1"
status: draft
producer: architect
timestamp: 2026-04-20T00:00:00Z
phase: 2-patch
inputs:
  - specs/prd.md
  - specs/verification-properties/VP-INDEX.md
  - specs/behavioral-contracts/BC-2.17.003
input-hash: "76729b7"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.17.003
module: prism-spec-engine
priority: P1
proof_method: proptest
verification_method: proptest
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: cycle-2-patch
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-041: Plugin Memory Limit Boundary — At-Limit Succeeds, Over-Limit Traps

## Property Statement

For any `limit_mb` in `1..=512`, a wasmtime `Store` configured via
`create_store_with_limit(limit_mb)` allows WASM linear memory allocation up to exactly
`limit_mb * 1024 * 1024` bytes and returns a trap error for any allocation attempt at
`limit_mb * 1024 * 1024 + 1` bytes. The boundary is exact and enforced per-plugin-instance.

## Source Contract

- **Anchor Story:** `S-1.15`
- **Source BC:** BC-2.17.003 — Plugin Sandbox — Memory Limit Enforced Per Plugin Instance (default 64MB)
- **Module:** prism-spec-engine
- **Category:** Resource Safety / Correctness

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest (latest) | No — parameterized over limit_mb 1..=512 | At-limit and over-limit boundary for all valid limit values |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_spec_engine::plugin::create_store_with_limit
//
// Sketch:
// proptest!(|(limit_mb in 1u64..=512u64)| {
//     let engine = Engine::new(&Config::default()).unwrap();
//     let store = create_store_with_limit(&engine, limit_mb);
//
//     // At-limit: allocate exactly limit_mb MiB — must succeed
//     let at_limit_result = try_allocate_wasm_memory(&store, limit_mb * 1024 * 1024);
//     prop_assert!(at_limit_result.is_ok(),
//         "allocation at exactly limit_mb={} MiB must succeed", limit_mb);
//
//     // Over-limit: allocate limit_mb MiB + 1 byte — must trap
//     let over_limit_result = try_allocate_wasm_memory(&store, limit_mb * 1024 * 1024 + 1);
//     prop_assert!(over_limit_result.is_err(),
//         "allocation over limit_mb={} MiB must trap", limit_mb);
// });
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | No | limit_mb range 1..=512; proptest shrinks to minimal failing case |
| Tool support? | Full | proptest + wasmtime StoreLimits; well-known boundary test pattern |
| Execution time budget | <120 seconds for 1000 cases | Store construction is cheap; memory allocation WAT is trivial |
| Assumptions required | At-limit allocation uses a synthetic WAT binary that requests exactly N bytes | Test helper produces WAT with matching memory declaration |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-20 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.1 | pass-87-remediation | 2026-04-21 | architect | F87-006: Source BC label corrected "Plugin Memory Limit" → "Plugin Sandbox — Memory Limit Enforced Per Plugin Instance (default 64MB)" (matches BC-2.17.003 H1). |
| 1.0 | pass-69-housekeeping | 2026-04-20 | architect | Initial draft. Resolves VP-TBD in BC-2.17.003 (at-limit/over-limit boundary). Host-process-isolation half remains integration test per decision matrix. |
