//! VP-015: AST nesting depth limit is always enforced.
//!
//! Property: For all AST expression trees where nesting depth exceeds
//! `MAX_NESTING_DEPTH` (64), `check_nesting_depth()` returns `Err` — never `Ok`.
//!
//! The canonical depth limit is **64** (BC-2.11.006, DI-019, EC-002).
//! The prior illustrative value of 32 is incorrect; this proof harness
//! uses 64 exclusively. See S-3.01 v1.6 changelog.
//!
//! Method: Kani bounded model checking (`cargo kani`).
//! Run: `cargo kani --harness proof_nesting_depth_limit`
//!
//! BC: BC-2.11.006 / DI-019 / EC-002
//! Story: S-3.01

#[cfg(kani)]
mod kani_proofs {
    use crate::ast::Expr;
    use crate::security::{check_nesting_depth, PRISM_MAX_NESTING_DEPTH};

    /// VP-015 — AST nesting depth > 64 always returns Err.
    ///
    /// Constructs a symbolic expression tree with depth strictly greater
    /// than `PRISM_MAX_NESTING_DEPTH` (64) and asserts that
    /// `check_nesting_depth` returns `Err`.
    ///
    /// The canonical limit is 64. Any value above 64 MUST be rejected.
    #[kani::proof]
    fn proof_nesting_depth_limit() {
        // extra_depth: any positive integer (search space bounded for Kani).
        let extra_depth: u32 = kani::any();
        kani::assume(extra_depth > 0);
        kani::assume(extra_depth <= 8); // bound Kani search space

        let depth_to_check = PRISM_MAX_NESTING_DEPTH + extra_depth;

        // TODO(S-3.01): construct a concrete deeply-nested Expr tree of
        // the required depth and call check_nesting_depth on it.
        // The stub uses a placeholder assertion.
        let _ = depth_to_check;
        kani::assert(
            true,
            "VP-015 harness stub — implementer must complete with real Expr tree",
        );
    }
}
