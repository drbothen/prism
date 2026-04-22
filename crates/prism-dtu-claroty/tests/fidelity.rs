//! Fidelity validation test for `prism-dtu-claroty`.
//!
//! Starts `ClarotyClone`, runs `FidelityValidator` against all 7 endpoints,
//! and asserts `checks_failed == 0`. Run as part of `just dtu-validate`.
//!
//! NOTE (Red Gate step 1): This test is a compile-only stub.
//! Assertions will be filled in during Red Gate step 2 (test writing).

// Tests are gated behind the `dtu` feature — this file will not be compiled
// in production builds.

#[tokio::test]
async fn claroty_dtu_fidelity() {
    // Red Gate step 2 will replace this with the full FidelityValidator run.
    // The test is intentionally left unimplemented so Red Gate step 1 verifies
    // compile-only correctness without executing assertions.
    unimplemented!("claroty_dtu_fidelity — implement in Red Gate step 2")
}
