# AC-7: VP-020 Kani Proof Harness Documentation

**Story:** S-1.08 — prism-security: Feature Flags (P0 Core)
**AC:** AC-7 — VP-020 Kani proof passes
**BC:** BC-2.04.004 (Two-Tier Gate), BC-2.04.001 (Compile-Time Gate)

## Why No VHS Recording

AC-7 requires the Kani model checker, which is a separate Rust toolchain
(`cargo +kani kani`) not installed in the CI/dev environment. Per VP-020's
feasibility assessment:

> "Compile-time gate modeled as runtime bool in test; separate build-matrix
> test covers the real cfg gate."

The unit test equivalent (VP-020 truth table) is recorded in
`VP-020-two-tier-truth-table.gif/.webm` and passes in the standard test run.

## Harness Location

```
crates/prism-security/kani/feature_flag_proof.rs
```

## Proof Structure

The Kani harness contains two proofs:

### `proof_vp020_two_tier_gate_truth_table`

Symbolically checks all four combinations of `(compile_ok: bool, runtime_allow: bool)`:

| compile_ok | runtime_allow | Expected result       |
|-----------|---------------|-----------------------|
| false     | false         | DeniedCompileTime     |
| false     | true          | DeniedCompileTime     |
| true      | false         | DeniedRuntime         |
| true      | true          | Allowed               |

Kani assertion:
```rust
kani::assert(
    allowed == (compile_ok && runtime_allow),
    "VP-020: check_permission must return Allowed iff BOTH gates pass",
);
```

### `proof_vp020_compile_absent_runtime_allow_still_denies`

Critical security property: runtime configuration CANNOT override a missing
compile-time feature (BC-2.04.001, BC-2.04.004 invariant).

```rust
// Runtime config explicitly allows — compile gate is absent.
let result = evaluator.check_permission(
    CompileTimeGate::Absent, // feature NOT compiled in
    client_id,
    capability,
);
kani::assert(
    matches!(result, CapabilityCheckResult::DeniedCompileTime { .. }),
    "VP-020: compile-absent with runtime Allow must still return DeniedCompileTime",
);
```

## Unit Test Counterpart (Runs in CI)

The unit test file `tests/vp_020_test.rs` implements the same truth table
exhaustively using concrete inputs (not symbolic). All 7 tests pass with
`--no-default-features`:

```
test test_VP_020_truth_table_absent_deny_is_denied ... ok
test test_VP_020_truth_table_absent_allow_is_denied_compile_time ... ok
test test_BC_2_04_004_vp020_unit_assertion_counterpart_to_kani_proof ... ok
test test_VP_020_result_equals_logical_and_of_both_gates ... ok
test test_VP_020_truth_table_present_allow_is_allowed ... ok
test test_VP_020_truth_table_present_deny_is_denied_runtime ... ok
test test_VP_020_applies_to_all_sensor_write_families ... ok

test result: ok. 7 passed; 0 failed
```

## Running the Kani Proof (when toolchain available)

```bash
# Install Kani toolchain (one-time)
cargo install --locked kani-verifier
cargo kani setup

# Run the VP-020 proof
cd crates/prism-security
cargo kani --harness proof_vp020_two_tier_gate_truth_table
cargo kani --harness proof_vp020_compile_absent_runtime_allow_still_denies
```

Expected output (when Kani is available):
```
VERIFICATION:- SUCCESSFUL
Status: SATISFIED
```
