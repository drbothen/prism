---
document_type: verification-property
level: L4
version: "1.3"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-1.02-entity-types.md]
input-hash: "7f066a0"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.14.002
module: prism-core
priority: P0
proof_method: kani
verification_method: kani
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

# VP-006: Case State Machine — No Self-Transitions

## Property Statement

For every `CaseStatus` value `s`, `s.can_transition_to(s)` returns `false`. The
state machine permits no reflexive transitions; a case cannot "transition" to the
status it is already in.

## Source Contract

- **Anchor Story:** `S-1.02`
- **Source BC:** BC-2.14.002 — Case State Transitions — 5-State Machine with 12 Valid Transitions
- **Module:** prism-core
- **Category:** Correctness

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| kani | Kani (latest) | No — exhaustive enum (5 values) | 100% of enum domain |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: kani
// Target: prism_core::case::CaseStatus::can_transition_to
//
// Sketch: for any symbolic CaseStatus s, assert !s.can_transition_to(s).
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | 5-variant enum, tiny domain |
| Tool support? | Full | Kani handles enum nondeterminism natively |
| Execution time budget | <10 seconds | Trivial enumeration |
| Assumptions required | None | Complete enumeration |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.3 | pass-88-remediation | 2026-04-21 | architect | F88-012: Anchor Story normalized from slug form (S-1.02-entity-types.md) to pure ID (S-1.02). |
| 1.2 | pass-87-remediation | 2026-04-21 | architect | F87-006: Source BC label corrected "5-State Machine, 12 Valid Transitions" → "5-State Machine with 12 Valid Transitions" (matches BC-2.14.002 H1). |
| 1.1 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
