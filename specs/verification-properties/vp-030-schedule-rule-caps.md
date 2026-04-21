---
document_type: verification-property
level: L4
version: "1.3"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-4.01-schedule-crud.md]
input-hash: "2f6d557"
traces_to: architecture/verification-architecture.md
source_bc: [BC-2.12.001, BC-2.13.006]
module: prism-operations
priority: P1
proof_method: kani
verification_method: kani
feasibility: medium
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: cycle-1
modified: 2026-04-19
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-030: Schedule/Rule Count Caps — Rejects Beyond Limits

## Property Statement

For every schedule store and rule store, if the count of registered items meets or
exceeds the configured ceiling, subsequent `create_schedule()` / `create_rule()`
calls return `Err(*::CapacityExhausted)`. Both stores strictly enforce their
configured caps.

## Source Contract

- **Anchor Story:** `S-4.01-schedule-crud.md`
- **Source BC:** BC-2.12.001 — `create_schedule` MCP Tool — Create a Scheduled Query; BC-2.13.006 — `create_rule` MCP Tool — Create Detection Rule with Scope
- **Module:** prism-operations
- **Category:** Security

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| kani | Kani (latest) | Yes — scaled caps | Boundary above/below scaled caps |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: kani
// Target: prism_operations::schedules::create_schedule / prism_operations::detections::create_rule
//
// Sketch: with scaled CAP, fill to CAP then assert next create returns
// CapacityExhausted; run for both schedule and rule stores.
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | Scaled caps |
| Tool support? | Full | Kani |
| Execution time budget | <5 minutes | Two harnesses, small scale |
| Assumptions required | Linear generalization to production caps | Documented |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |
| modified | 2026-04-19 | product-owner — Burst 41 / P3P39-A-HIGH-005: corrected source_bc from invented BC-2.12.010 to [BC-2.12.001, BC-2.13.006] (actual DI-028 enforcers per invariants.md); rewrote Source BC body section with canonical BC-INDEX titles; version bump to 1.1 |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.3 | pass-61-fix | 2026-04-20 | architect | Renumbered duplicate pre-build-sweep Changelog row for version monotonicity (MED-001 VP scope extension). |
| 1.2 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat); created Changelog section (previously only Lifecycle existed). |
| 1.1 | Burst-41 | 2026-04-19 | product-owner | Corrected source_bc from invented BC-2.12.010 to [BC-2.12.001, BC-2.13.006]; rewrote Source BC body section; version bump to 1.1. Closes P3P39-A-HIGH-005. |
