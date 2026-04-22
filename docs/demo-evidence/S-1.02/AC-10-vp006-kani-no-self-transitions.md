# AC-10 — VP-006 Kani Proof: No Self-Transitions

**Status:** Phase 5 formal verification (Kani not run in standard CI)

## Acceptance Criterion

VP-006 Kani proof passes: no CaseStatus self-transitions exist.

## Proof Location

`crates/prism-core/src/proofs/case_status.rs`

Proof function: `proof_no_self_transitions`

## Reproduction Command

```sh
cd crates/prism-core
cargo kani --harness proof_no_self_transitions
```

## What the Proof Verifies

For every `CaseStatus` variant, asserts:

```
!status.can_transition_to(status)
```

This exhaustively proves that the state machine contains no self-loops across all 5
variants (New, Acknowledged, Investigating, Resolved, Closed).

## Runtime Coverage

The same property is verified at runtime by:

- `test_BC_S_02_001_vp006_no_self_transitions` — iterates all 5 variants and asserts
  each self-transition returns false.

Passes as part of the 103-test suite at commit 44906b8.

## Phase Gate

Phase 5 formal verification. Placeholder replaces GIF/WEBM until Kani toolchain is
available in CI.
