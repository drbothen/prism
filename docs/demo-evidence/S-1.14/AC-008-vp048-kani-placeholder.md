# VP-048 Kani Proof — Placeholder

**AC-9 / VP-048:** `InfusionRegistry::load_spec` with N distinct fields produces exactly N
`InfusionUdfDescriptor` values; any duplicate field name returns `Err(E-INFUSE-002)`.

**Method:** Kani bounded model checking (cfg-gated `#[cfg(kani)]`)

**Proof harness location:** `crates/prism-spec-engine/src/proofs/infusion_spec.rs`

**Traces to:** BC-2.19.001 postconditions

## Why No Recording

Kani proofs require the Kani toolchain (`cargo kani`) which runs formal symbolic
verification — not a terminal demo that can be VHS-recorded. The proof harness exists
and is authored at the location above. It will be executed during Phase 5 formal-verify.

## Harnesses Defined

| Harness | Property | Bound |
|---------|----------|-------|
| `verify_n_fields_n_descriptors` | N distinct fields → exactly N descriptors | N in 1..=16 |
| `verify_duplicate_udf_name_errors` | Duplicate field name → `Err(DuplicateUdfName)` | Fixed (2 identical fields) |

## Compile-Time Verification

A compile-check unit test (`test_BC_2_19_001_proof_types_compile`) verifies that all
types referenced in the Kani harnesses compile correctly under the normal Rust test
runner. This test passes as part of the 220/220 green suite.

The proptest equivalent (VP-049, AC-10) — which runs 1000 stochastic cases covering
the same dedup invariant — is recorded in `AC-009-vp049-proptest.gif/.webm`.
