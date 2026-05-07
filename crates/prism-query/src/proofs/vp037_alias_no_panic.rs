//! VP-037: Alias expansion never panics on arbitrary alias graphs.
//!
//! Property: For every byte sequence interpreted as an alias-map + query pair,
//! `expand_aliases` returns `Ok` or `Err` in bounded time without panicking.
//! Cyclic graphs, depth blowups, malformed references, and adversarial inputs
//! must all produce structured errors — never stack overflow, panic, or infinite loop.
//!
//! ## Proof Method
//!
//! | Method | Tool          | Bounded? | Coverage                                        |
//! |--------|---------------|----------|-------------------------------------------------|
//! | fuzz   | cargo-fuzz    | No       | Coverage-guided mutation; arbitrary byte inputs |
//! | proptest | proptest    | No       | Structured arbitrary string inputs              |
//!
//! The proptest harnesses in this module provide structured coverage of the
//! no-panic property during TDD (unit-test phase). The cargo-fuzz target
//! (fuzz/fuzz_targets/fuzz_alias_expand.rs) provides exhaustive coverage during
//! Phase 6 hardening.
//!
//! ## Invariant tested
//!
//! `AliasResolver::expand()` is safe to call with ANY string input and MUST
//! NOT panic. It may return `Ok(expanded)` or `Err(structured_error)` but never
//! cause a stack overflow, memory abort, or unwind through a non-`#[test]` frame.
//!
//! Story: S-3.04 — prism-query: Alias System (P1)
//! VP:    VP-037 (fuzz + proptest coverage layers)
//! BCs:   BC-2.11.008, BC-2.11.009

// ─────────────────────────────────────────────────────────────────────────────
// Proptest harnesses (cfg(test)) — always compiled and run; RED by design.
//
// These tests use catch_unwind to verify that todo!() panics are bounded. Once
// `expand()` is implemented, the catch_unwind will see Ok/Err (not Err from panic)
// and the test bodies should be updated to assert !result.is_err().
//
// Test naming: test_VP_037_* (VP-based prefix for traceability)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod proptest_harnesses {
    use std::collections::HashMap;

    use proptest::prelude::*;

    use crate::alias_resolver::AliasResolver;
    use crate::alias_store::AliasStore;
    use crate::alias_types::AliasScope;

    // NOTE: catch_unwind is intentionally ABSENT (F-CRIT-003). proptest converts panics
    // to test failures by default. Using catch_unwind would mask VP-037 violations rather
    // than surface them. All calls below must return Ok or Err — never panic.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// VP-037 property: `expand()` never panics on arbitrary printable ASCII queries.
        ///
        /// Covers the entire printable ASCII space (0x20–0x7E), which includes:
        /// - Operator characters: = != > < >= <= ( ) | AND OR NOT
        /// - String literals: "quoted", 'single'
        /// - Identifier characters: a-z A-Z _ 0-9
        /// - Injection attempts: ; DROP TABLE, OR 1=1, etc.
        #[test]
        fn prop_vp037_expand_arbitrary_printable_ascii(
            query in "[\\x20-\\x7E]{0,512}"
        ) {
            let store = AliasStore::empty("/tmp/vp037_proof.toml");
            let scope = AliasScope::Global;
            let args = HashMap::new();
            // Must return Ok or Err — must not abort, SIGBUS, or panic.
            let _ = AliasResolver::expand(&query, &store, &scope, &args, 0);
        }

        /// VP-037 property: `expand()` never panics when the query contains @-references
        /// with arbitrary valid identifier names.
        ///
        /// Empty store means all references produce E-ALIAS-001 — never panic.
        #[test]
        fn prop_vp037_expand_with_alias_references_no_panic(
            ref_name in "[a-zA-Z_][a-zA-Z0-9_]{0,63}",
            suffix in "[a-zA-Z0-9 _=><'\"]{0,64}"
        ) {
            let query = format!("@{ref_name} {suffix}");
            let store = AliasStore::empty("/tmp/vp037_proof.toml");
            let scope = AliasScope::Global;
            let args = HashMap::new();
            let _ = AliasResolver::expand(&query, &store, &scope, &args, 0);
        }

        /// VP-037 property: `expand()` called with depth at or beyond MAX_ALIAS_DEPTH
        /// never panics — always returns Err(AliasDepthExceeded).
        ///
        /// This exercises the depth-limit pre-check path for arbitrary query inputs.
        #[test]
        fn prop_vp037_expand_at_or_beyond_max_depth_no_panic(
            query in "[\\x20-\\x7E]{0,128}",
            depth_offset in 0u32..=5u32
        ) {
            use crate::alias_resolver::MAX_ALIAS_DEPTH;
            let depth = MAX_ALIAS_DEPTH + depth_offset;
            let store = AliasStore::empty("/tmp/vp037_proof.toml");
            let scope = AliasScope::Global;
            let args = HashMap::new();
            // At depth >= MAX_ALIAS_DEPTH, must return Err(AliasDepthExceeded) — never panic.
            let result = AliasResolver::expand(&query, &store, &scope, &args, depth);
            prop_assert!(
                result.is_err(),
                "expand at depth >= MAX_ALIAS_DEPTH must return Err(AliasDepthExceeded)"
            );
        }

        /// VP-037 property: `validate_atomic_literal()` never panics on arbitrary inputs.
        ///
        /// The injection guard must handle ALL inputs without panicking — including
        /// compound expressions, SQL injections, empty strings, and binary-looking data.
        #[test]
        fn prop_vp037_validate_atomic_literal_arbitrary_input(
            value in "[\\x20-\\x7E]{0,256}",
            param in "[a-zA-Z_][a-zA-Z0-9_]{0,31}",
            alias in "[a-zA-Z_][a-zA-Z0-9_]{0,31}"
        ) {
            // Must return Ok or Err — must not panic.
            let _ = AliasResolver::validate_atomic_literal(&value, &param, &alias);
        }

        /// VP-037 property: `detect_alias_tokens()` never panics on arbitrary query bodies.
        ///
        /// The regex engine used for @-token detection must handle all string inputs
        /// including those with control characters, non-standard whitespace, and
        /// embedded null bytes (as lossy-decoded strings).
        #[test]
        fn prop_vp037_detect_alias_tokens_arbitrary_input(
            query in "[\\x00-\\x7F]{0,512}"
        ) {
            // Must return Vec<String> — must not panic.
            let _ = AliasResolver::detect_alias_tokens(&query);
        }

        /// VP-037 property: `substitute_params()` never panics on arbitrary template
        /// and parameter combinations.
        ///
        /// Template placeholders may be malformed ({{}} without name, nested braces, etc.)
        /// — the substitution function must handle all without panicking.
        #[test]
        fn prop_vp037_substitute_params_arbitrary_template(
            template in "[\\x20-\\x7E]{0,256}",
            value in "[\\x20-\\x7E]{0,128}"
        ) {
            use crate::alias_types::{AliasEntry, AliasScope, ParamDefault};
            use std::collections::HashMap as StdHashMap;

            let mut parameters = StdHashMap::new();
            parameters.insert(
                "p".to_string(),
                ParamDefault { value: "default".to_string() },
            );
            let entry = AliasEntry {
                name: "test_alias".to_string(),
                scope: AliasScope::Global,
                query: template.clone(),
                parameters: Some(parameters),
                description: None,
            };
            let mut args = StdHashMap::new();
            args.insert("p".to_string(), value);

            // Must return Ok or Err — must not panic.
            let _ = AliasResolver::substitute_params(&template, &entry, &args);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Fuzz target stub note
//
// The cargo-fuzz target for VP-037 lives at:
//   fuzz/fuzz_targets/fuzz_alias_expand.rs
//
// See VP-037 proof harness skeleton in:
//   .factory/specs/verification-properties/vp-037-alias-expansion-no-panic.md
//
// The fuzz target is authored separately during Phase 6 formal hardening.
// ─────────────────────────────────────────────────────────────────────────────
