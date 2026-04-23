# AC-6: Kani Formal Proofs — VP-007, VP-008, VP-009, VP-010

**Story:** S-1.09  
**AC:** AC-6  
**Policy:** POL-010  

## Verification Properties

| VP ID | Description | Kani Harness |
|-------|-------------|--------------|
| VP-007 | Token at exactly 300s is expired (boundary condition) | `crates/prism-security/kani/token_proofs.rs` |
| VP-008 | Consumed token cannot be consumed again (single-use invariant) | `crates/prism-security/kani/token_proofs.rs` |
| VP-009 | Modified action params produce different hash → rejection | `crates/prism-security/kani/token_proofs.rs` |
| VP-010 | 101st token generation when 100 active → E-FLAG-007 | `crates/prism-security/kani/token_proofs.rs` |

## Unit Test Counterparts (runnable without Kani)

The following tests directly mirror the Kani proof properties and run in standard
`cargo test`:

```
test test_VP_007_expiry_boundary_suite ... ok
test test_VP_007_error_type_for_expired_token_is_token_expired ... ok
test test_VP_008_single_use_first_consume_succeeds_second_fails ... ok
test test_VP_008_third_consume_also_fails ... ok
test test_VP_008_independent_tokens_are_each_single_use ... ok
test test_VP_009_hash_is_deterministic ... ok
test test_VP_009_key_reordering_does_not_change_hash ... ok
test test_VP_009_tampered_params_hash_mismatch_rejects ... ok
test test_VP_010_token_cap_enforcement_e_flag_007 ... ok
test test_VP_010_consuming_from_full_store_frees_slot_for_generate ... ok
```

All 10 unit-test counterparts pass (`cargo test -p prism-security`).

## Kani Proof Source

Proof harnesses are in `crates/prism-security/kani/token_proofs.rs`.

To run Kani proofs (requires Kani toolchain):
```bash
cargo kani --harness proof_vp007_expiry_boundary
cargo kani --harness proof_vp008_single_use
cargo kani --harness proof_vp009_hash_mismatch
cargo kani --harness proof_vp010_token_cap
```

Kani proofs are not run in CI by default (resource-intensive). The unit test
counterparts above provide runtime evidence of the same properties.
