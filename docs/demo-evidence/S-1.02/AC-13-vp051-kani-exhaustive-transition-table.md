# AC-13 — VP-051 Kani Proof: Exhaustive 5x5 Transition Table

**Status:** Phase 5 formal verification (Kani not run in standard CI)

## Acceptance Criterion

VP-051 Kani proof passes: all 25 `(from, to)` pairs produce the correct `Ok`/`Err`
outcome — exactly 12 `Ok` and 13 `Err`.

Traces to: BC-2.14.002 postconditions.

## Proof Location

`crates/prism-core/src/proofs/case_status_exhaustive.rs`

Proof function: `proof_exhaustive_transition_table`

## Reproduction Command

```sh
cd crates/prism-core
cargo kani --harness proof_exhaustive_transition_table
```

## What the Proof Verifies

Kani enumerates all 25 `(from_state, to_state)` pairs. For each pair it asserts
`advance_case_state` returns:

- `Ok(to_state)` for the 12 valid transitions
- `Err(CaseTransitionError::SelfTransition)` (E-CASE-005) for all 5 self-transitions
- `Err(CaseTransitionError::InvalidTransition)` (E-CASE-004) for the remaining 8 invalid
  non-self pairs

Total: 12 Ok + 5 SelfTransition + 8 InvalidTransition = 25 pairs.

## Runtime Coverage

The same exhaustive check runs at runtime via:

- `test_BC_S_02_001_vp051_exhaustive_25_pairs_correct_outcome` — iterates all 25 pairs
  and asserts the correct variant
- `test_BC_S_02_001_vp051_valid_transition_returns_ok`
- `test_BC_S_02_001_vp051_self_transition_returns_e_case_005`
- `test_BC_S_02_001_vp051_invalid_non_self_returns_e_case_004`

All pass as part of the 103-test suite at commit 44906b8.

## Phase Gate

Phase 5 formal verification. Placeholder replaces GIF/WEBM until Kani toolchain is
available in CI.
