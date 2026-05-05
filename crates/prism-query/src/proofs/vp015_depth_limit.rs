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
//! Run: `cargo kani --harness proof_expr_depth_limit`
//! Run: `cargo kani --harness proof_predicate_depth_limit`
//!
//! BC: BC-2.11.006 / DI-019 / EC-002
//! Story: S-3.01

#[cfg(kani)]
mod kani_proofs {
    use crate::ast::{CompareOp, Expr, FieldPath, Literal, Predicate, Span};
    use crate::security::{
        check_expr_nesting_depth, check_nesting_depth, check_predicate_nesting_depth,
        PRISM_MAX_NESTING_DEPTH,
    };

    /// VP-015 — `check_nesting_depth(Expr)` at depth > 64 always returns Err.
    ///
    /// The canonical limit is 64. Any depth value > 64 passed to
    /// `check_nesting_depth` MUST return `Err`.
    #[kani::proof]
    fn proof_nesting_depth_limit() {
        let extra_depth: u32 = kani::any();
        kani::assume(extra_depth > 0);
        kani::assume(extra_depth <= 8);

        let depth_to_check = PRISM_MAX_NESTING_DEPTH + extra_depth;
        let leaf = Expr::Literal(Literal::Integer(1));
        let result = check_nesting_depth(&leaf, depth_to_check);
        kani::assert(result.is_err(), "VP-015: depth > 64 must return Err");
    }

    /// VP-015 — `check_expr_nesting_depth(Expr::Not chain)` at depth 65 returns Err.
    ///
    /// Constructs a concrete `Expr::Not` chain of the specified depth and
    /// asserts that `check_expr_nesting_depth` returns `Err`.
    #[kani::proof]
    fn proof_expr_depth_limit() {
        // Kani symbolic depth: any value in (64, 72].
        let depth: u32 = kani::any();
        kani::assume(depth > PRISM_MAX_NESTING_DEPTH);
        kani::assume(depth <= PRISM_MAX_NESTING_DEPTH + 8);

        // Build a depth-`depth` NOT chain.
        let mut expr = Expr::Literal(Literal::Integer(0));
        let mut i = 0u32;
        while i < depth {
            expr = Expr::Not(Box::new(expr));
            i += 1;
        }
        let result = check_expr_nesting_depth(&expr, 0);
        kani::assert(
            result.is_err(),
            "VP-015: Expr::Not chain of depth > 64 must return Err",
        );
    }

    /// VP-015 — `check_predicate_nesting_depth(Predicate::Not chain)` at depth 65 returns Err.
    ///
    /// Exercises the visitor-based depth check path that was missing from
    /// the original `check_nesting_depth` implementation.
    #[kani::proof]
    fn proof_predicate_depth_limit() {
        let depth: u32 = kani::any();
        kani::assume(depth > PRISM_MAX_NESTING_DEPTH);
        kani::assume(depth <= PRISM_MAX_NESTING_DEPTH + 8);

        // Build a depth-`depth` Predicate::Not chain wrapping a leaf Compare.
        let leaf = Predicate::Compare {
            lhs: Box::new(Expr::Field(FieldPath {
                segments: vec!["x".to_string()],
                span: Span::ZERO,
            })),
            op: CompareOp::Eq,
            rhs: Box::new(Expr::Literal(Literal::Integer(0))),
        };
        let mut pred = leaf;
        let mut i = 0u32;
        while i < depth {
            pred = Predicate::Not(Box::new(pred));
            i += 1;
        }
        let result = check_predicate_nesting_depth(&pred, 0);
        kani::assert(
            result.is_err(),
            "VP-015: Predicate::Not chain of depth > 64 must return Err",
        );
    }
}
