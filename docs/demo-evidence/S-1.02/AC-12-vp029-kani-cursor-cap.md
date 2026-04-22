# AC-12 — VP-029 Kani Proof: CursorRegistry Enforces 200-Cap

**Status:** Phase 5 formal verification (Kani not run in standard CI)

## Acceptance Criterion

VP-029 Kani proof passes: `CursorRegistry` enforces a cap of 200 active cursors.

## Proof Location

`crates/prism-core/src/proofs/cursor.rs`

Proof function: `proof_cursor_cap_200`

## Reproduction Command

```sh
cd crates/prism-core
cargo kani --harness proof_cursor_cap_200
```

## What the Proof Verifies

Three-phase assertion:

1. Allocate exactly 200 cursors — all must return `Ok`.
2. Attempt 201st allocation — must return `Err(CursorCapExceeded)`.
3. Release one cursor; attempt allocation again — must return `Ok` (cap counts active
   cursors, not lifetime-allocated cursors).

## Runtime Coverage

The same boundary behavior is verified at runtime by:

- `test_BC_S_02_004_ac6_201st_allocation_fails` — phase 2 above
- `test_BC_S_02_004_ac7_release_then_allocate_succeeds` — phase 3 above
- `test_BC_S_02_004_vp029_active_count_at_cap` — phase 1 above
- `test_BC_S_02_004_vp029_199_allocations_succeed` — boundary at 199

All pass as part of the 103-test suite at commit 44906b8.

## Phase Gate

Phase 5 formal verification. Placeholder replaces GIF/WEBM until Kani toolchain is
available in CI.
