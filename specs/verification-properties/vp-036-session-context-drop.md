---
document_type: verification-property
level: L4
version: "1.1"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-6.07-dtu-crowdstrike.md]
input-hash: "e8e33c1"
traces_to: architecture/verification-architecture.md
source_bc: "BC-2.15.007"
module: prism-dtu-crowdstrike
priority: P0
proof_method: integration_test
verification_method: integration_test
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

# VP-036: SessionContext Dropped Before Error Propagation and on Panic

## Property Statement

For every code path in which an operation using a `SessionContext` returns an error
or panics, the `SessionContext` is dropped (RAII cleanup runs) strictly before the
error is observed by the caller or the panic propagates past the owning frame. No
error or unwind path escapes while a `SessionContext` is still live.

## Source Contract

- **Anchor Story:** `S-6.07-dtu-crowdstrike.md`
- **Source BC:** `BC-2.15.007` — Watchdog Query Termination (invariant: no leaked SessionContext/RecordBatches/HTTP connections on termination)
- **Module:** prism-dtu-crowdstrike
- **Category:** Safety

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| integration_test | cargo test + DTU clone with instrumented Drop | No — scenario-driven | Error and panic paths exercised |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Location: crates/prism-dtu-crowdstrike/tests/session_context_drop.rs
//
// Sketch: instrument SessionContext with drop counter; run scenarios that
// return Err and scenarios that panic (std::panic::catch_unwind); assert the
// drop counter increments before the Err is returned / before the unwind
// escapes the scope.
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | No | Scenario-driven |
| Tool support? | Full | cargo test + catch_unwind |
| Execution time budget | <5 minutes | Integration scale |
| Assumptions required | Drop is infallible (panic-in-drop excluded) | Standard Rust guidance |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.1 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
