---
document_type: verification-property
level: L4
version: "1.0"
status: draft
producer: architect
timestamp: 2026-04-20T00:00:00Z
phase: 2-patch
inputs:
  - specs/prd.md
  - specs/verification-properties/VP-INDEX.md
  - specs/behavioral-contracts/BC-2.15.005
input-hash: "643fc81"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.15.005
module: prism-persistence
priority: P0
proof_method: kani
verification_method: kani
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

# VP-057: Crash Recovery — Denylist Triggered at Three Consecutive Crashes

## Property Statement

`advance_crash_counter(entry: DirtyBitEntry) -> RecoveryAction` returns
`RecoveryAction::Denylist { expiry_seconds: 86400 }` if and only if
`entry.consecutive_crashes + 1 >= 3`. For all other values it returns
`RecoveryAction::Warn`. The denylist threshold is exactly 3 (not 2 or 4).
The result is idempotent: processing the same dirty bit entry twice produces the
same recovery action.

## Source Contract

- **Anchor Story:** `S-1.02`
- **Source BC:** BC-2.15.005 — Crash Recovery Dirty Bits
- **Module:** prism-persistence
- **Category:** Safety-Critical / Recovery State Machine

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| kani | Kani (latest) | Yes — symbolic u32 for consecutive_crashes | All possible counter values including boundary |

**Feasibility:** This is structurally identical to the retry state machine proven by VP-044.
The function `advance_crash_counter` takes a `DirtyBitEntry` (a struct with a `consecutive_crashes: u32`
field) and returns a `RecoveryAction` enum. Kani can prove the exact threshold condition across
all symbolic u32 values in seconds. The 86400-second expiry is a constant that Kani can verify
is always present in the Denylist variant.

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: kani
// Target: prism_persistence::recovery::advance_crash_counter
//
// #[kani::proof]
// fn verify_denylist_at_three_consecutive_crashes() {
//     let consecutive_crashes: u32 = kani::any();
//
//     let entry = DirtyBitEntry { consecutive_crashes };
//     let action = advance_crash_counter(entry);
//
//     // Threshold is exactly 3: denylist iff (consecutive_crashes + 1) >= 3,
//     // i.e., consecutive_crashes >= 2
//     if consecutive_crashes >= 2 {
//         kani::assert!(
//             matches!(action, RecoveryAction::Denylist { expiry_seconds: 86400 }),
//             "must denylist with 86400s expiry when consecutive_crashes >= 2"
//         );
//     } else {
//         kani::assert!(
//             matches!(action, RecoveryAction::Warn),
//             "must warn (not denylist) when consecutive_crashes < 2"
//         );
//     }
//
//     // Idempotency: calling again with same entry produces same action
//     let action2 = advance_crash_counter(entry);
//     kani::assert!(action == action2, "advance_crash_counter must be idempotent");
//
//     // Exact threshold: boundary at consecutive_crashes == 2
//     if consecutive_crashes == 1 {
//         kani::assert!(matches!(action, RecoveryAction::Warn),
//             "exactly 2 crashes (0-indexed 1) must not trigger denylist");
//     }
//     if consecutive_crashes == 2 {
//         kani::assert!(matches!(action, RecoveryAction::Denylist { .. }),
//             "exactly 3 crashes (0-indexed 2) must trigger denylist");
//     }
// }
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | consecutive_crashes is a u32; Kani uses symbolic values across full range |
| Tool support? | Full | Trivially bounded state machine identical in structure to VP-044 |
| Execution time budget | <30 seconds | Single function call with symbolic integer; very fast |
| Assumptions required | `advance_crash_counter` is a pure function with no side effects | BC-2.15.005 design requires this for Kani feasibility |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-20 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.0 | pass-74-vp-additions | 2026-04-20 | architect | Initial draft. Resolves VP-TBD in BC-2.15.005. Proves exact denylist threshold (>=3 consecutive crashes) with 86400s expiry and idempotency. Method: Kani. P0. |
