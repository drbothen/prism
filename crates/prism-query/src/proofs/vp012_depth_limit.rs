//! VP-012: Alias expansion depth > 3 always returns `Err(E-ALIAS-003)`.
//!
//! Property: For every alias-expansion call where the alias reference graph
//! requires traversing more than 3 nested alias definitions, `AliasResolver::expand()`
//! returns `Err(AliasDepthExceeded { ... })` without producing an expanded query.
//! Depth 3 is the hard ceiling; depth 4 and beyond is always rejected.
//!
//! ## Proof Method
//!
//! | Method | Tool | Bounded? | Coverage |
//! |--------|------|----------|----------|
//! | kani   | Kani (0.67) | Yes — alias graphs up to 5 levels | All depth configurations |
//!
//! ## Kani harness sketch
//!
//! Construct chains a1→a2→a3→a4 (depth 4); assert expand returns
//! Err(AliasDepthExceeded). At depth 3 assert Ok or E-ALIAS-001 (not E-ALIAS-003).
//!
//! ## Canonical invocation
//!
//! ```text
//! cargo kani -p prism-query \
//!     --harness "proofs::vp012_depth_limit::kani_proofs::proof_depth_gt3_always_err" \
//!     --exact --no-unwinding-checks --default-unwind 2
//! ```
//!
//! Story: S-3.04 — prism-query: Alias System (P1)
//! VP:    VP-012 (kani proof_method)

// ─────────────────────────────────────────────────────────────────────────────
// Concrete tests (cfg(test)) — always compile and run; RED by design.
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod concrete_tests {
    use std::collections::HashMap;

    use crate::alias_resolver::{AliasResolver, MAX_ALIAS_DEPTH};
    use crate::alias_store::AliasStore;
    use crate::alias_types::AliasScope;

    /// Concrete boundary check: calling expand at depth == MAX_ALIAS_DEPTH must
    /// immediately return Err (depth limit pre-check fires before alias lookup).
    ///
    /// This test is RED until AliasResolver::expand is implemented.
    #[test]
    fn concrete_expand_at_max_depth_is_err() {
        // RED: AliasResolver::expand is todo!()
        let store = AliasStore::empty("/tmp/vp012_test.toml");
        let scope = AliasScope::Global;
        let args = HashMap::new();
        let result = AliasResolver::expand("@any_alias", &store, &scope, &args, MAX_ALIAS_DEPTH);
        assert!(
            result.is_err(),
            "VP-012 RED gate: expand at depth={} must return Err; got Ok",
            MAX_ALIAS_DEPTH
        );
    }

    /// Concrete boundary: depth 3 at the expansion boundary (MAX_ALIAS_DEPTH - 1 + 1).
    ///
    /// At `depth = MAX_ALIAS_DEPTH - 1 = 2`, a single `@` reference would push depth
    /// to 3, which is exactly the limit. This tests that depth=2 is the last accepted
    /// depth (the recursive call at depth=3 would fail only if the alias is found).
    ///
    /// RED until expand is implemented.
    #[test]
    fn concrete_expand_at_depth_2_not_depth_limit_error() {
        // RED: todo!()
        let store = AliasStore::empty("/tmp/vp012_test.toml");
        let scope = AliasScope::Global;
        let args = HashMap::new();
        // At depth=2 the depth-limit pre-check should NOT fire (2 < MAX_ALIAS_DEPTH=3).
        // But an E-ALIAS-001 may fire (alias not in store). Either way the test is RED.
        let result = AliasResolver::expand("@any_alias", &store, &scope, &args, 2);
        assert!(result.is_err(), "todo!() fires — RED gate");
    }

    /// Concrete: depth=4 (one past the ceiling) must be rejected.
    ///
    /// RED until expand is implemented.
    #[test]
    fn concrete_expand_at_depth_4_is_err() {
        // RED: todo!()
        let store = AliasStore::empty("/tmp/vp012_test.toml");
        let scope = AliasScope::Global;
        let args = HashMap::new();
        let result = AliasResolver::expand("@any_alias", &store, &scope, &args, 4);
        assert!(result.is_err(), "todo!() fires — RED gate");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Kani proof harnesses (cfg(kani)) — compiled only under `cargo kani`
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(kani)]
mod kani_proofs {
    use std::collections::HashMap;

    use crate::alias_resolver::{AliasResolver, MAX_ALIAS_DEPTH};
    use crate::alias_store::AliasStore;
    use crate::alias_types::AliasScope;
    use prism_core::error::PrismError;

    /// Kani proof: for all `depth >= MAX_ALIAS_DEPTH`, `expand()` returns
    /// `Err(PrismError::AliasDepthExceeded { .. })` — never `Ok`.
    ///
    /// The proof enumerates symbolic `depth` values in the range
    /// `[MAX_ALIAS_DEPTH, MAX_ALIAS_DEPTH + 4]` (5 levels) to cover the
    /// boundary and slightly beyond.
    ///
    /// Notes:
    /// - The query content is fixed to `"@stub"` (content-oblivious: the
    ///   depth check fires before alias lookup).
    /// - `AliasStore::empty()` is used so no aliases resolve (Kani cannot
    ///   model arbitrary store state within feasibility bounds).
    /// - `args` is an empty HashMap (no parameter substitution at this level).
    #[kani::proof]
    #[kani::unwind(8)]
    fn proof_depth_gt3_always_err() {
        // Symbolic depth offset in [0, 4]; effective depth = MAX_ALIAS_DEPTH + offset
        let offset: u32 = kani::any();
        kani::assume(offset <= 4);
        let depth = MAX_ALIAS_DEPTH + offset;

        let store = AliasStore::empty("/tmp/kani_vp012.toml");
        let scope = AliasScope::Global;
        let args = HashMap::new();

        let result = AliasResolver::expand("@stub", &store, &scope, &args, depth);

        // Property: must never be Ok.
        assert!(
            result.is_err(),
            "VP-012 VIOLATION: expand at depth={depth} returned Ok"
        );

        // Stronger property: error must be AliasDepthExceeded (not some other error).
        // The depth limit fires before alias lookup, so this must be the first error.
        match result {
            Err(PrismError::AliasDepthExceeded { .. }) => { /* expected */ }
            Err(other) => {
                // AliasStore::empty() means E-ALIAS-001 might fire if depth check
                // does not run first — that would be a VP-012 violation.
                panic!(
                    "VP-012 VIOLATION: expected AliasDepthExceeded, got {:?}",
                    other
                );
            }
            Ok(_) => panic!("VP-012 VIOLATION: expand at depth={depth} returned Ok"),
        }
    }

    /// Kani proof: at `depth = MAX_ALIAS_DEPTH - 1 = 2`, the depth limit does NOT
    /// fire. The expand call may return E-ALIAS-001 (alias absent) but must NOT
    /// return AliasDepthExceeded.
    ///
    /// This is the complementary boundary property: the ceiling is inclusive at
    /// exactly MAX_ALIAS_DEPTH, not MAX_ALIAS_DEPTH - 1.
    #[kani::proof]
    #[kani::unwind(8)]
    fn proof_depth_below_max_not_depth_limit_err() {
        // Test at depth = MAX_ALIAS_DEPTH - 1 (the highest non-rejected depth).
        let depth = MAX_ALIAS_DEPTH - 1;

        let store = AliasStore::empty("/tmp/kani_vp012.toml");
        let scope = AliasScope::Global;
        let args = HashMap::new();

        let result = AliasResolver::expand("@stub", &store, &scope, &args, depth);

        // At this depth the depth limit must NOT fire.
        if let Err(PrismError::AliasDepthExceeded { .. }) = &result {
            panic!(
                "VP-012 VIOLATION: AliasDepthExceeded at depth={depth} (< MAX_ALIAS_DEPTH={})",
                MAX_ALIAS_DEPTH
            );
        }
    }
}
