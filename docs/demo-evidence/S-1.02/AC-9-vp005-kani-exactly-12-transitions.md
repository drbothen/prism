# AC-9 — VP-005 Kani Proof: Exactly 12 Valid Transitions

**Status:** Phase 5 formal verification (Kani not run in standard CI)

## Acceptance Criterion

VP-005 Kani proof passes: exactly 12 CaseStatus transitions are valid.

## Proof Location

`crates/prism-core/src/proofs/case_status.rs`

Proof function: `proof_exactly_12_transitions`

## Reproduction Command

```sh
cd crates/prism-core
cargo kani --harness proof_exactly_12_transitions
```

## What the Proof Verifies

Kani enumerates all 25 symbolic `(current, target)` pairs from the 5×5 cross-product
of `CaseStatus` variants. For each pair it asserts:

```
can_transition_to(current, target) == VALID_TRANSITIONS.contains(&(current, target))
```

This is equivalent to asserting that exactly 12 pairs return `true` and 13 return `false`,
using the same `VALID_TRANSITIONS` const array that the runtime function consults —
no duplication of the transition list.

## Runtime Coverage

The same property is verified at runtime by:

- `test_BC_S_02_001_vp005_exactly_12_valid_transitions` — counts all true pairs and asserts == 12
- `test_BC_S_02_001_vp005_all_12_valid_pairs_return_true` — verifies each pair in VALID_TRANSITIONS
- `test_BC_S_02_001_vp005_invalid_transitions_return_false` — verifies all 13 non-valid pairs

All three pass as part of the 103-test suite at commit 44906b8.

## Phase Gate

This proof runs in Phase 5 (Formal Verification). No GIF/WEBM is produced because
`cargo kani` requires the Kani toolchain (not installed in standard CI).
When Phase 5 is scheduled, the proof will run in a Kani-enabled GitHub Actions runner
and its output will replace this placeholder.
