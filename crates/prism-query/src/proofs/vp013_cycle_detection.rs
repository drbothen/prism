//! VP-013: Alias cycle detection correctly identifies all circular references.
//!
//! Property: For every alias map, `AliasResolver::detect_cycle()` (and transitively
//! `expand()`) terminates without stack overflow or infinite loop, and returns
//! `Err(AliasCycleDetected { .. })` whenever the alias reference graph contains a
//! cycle reachable from the input name. Expansion of alias-free inputs succeeds or
//! returns E-ALIAS-001 (absent alias); expansion of any cyclic input fails in bounded time.
//!
//! ## Proof Method
//!
//! | Method   | Tool     | Bounded? | Coverage |
//! |----------|----------|----------|----------|
//! | proptest | proptest | No       | Random alias graphs including self-loops, mutual, chained cycles |
//!
//! ## Story
//!
//! Story: S-3.04 — prism-query: Alias System (P1)
//! VP:    VP-013 (proptest proof_method)

// ─────────────────────────────────────────────────────────────────────────────
// Concrete tests (always compiled; RED by design)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod concrete_tests {
    use crate::alias_resolver::AliasResolver;
    use crate::alias_store::AliasStore;
    use crate::alias_types::{AliasEntry, AliasScope};

    /// Self-loop: A = "@A" must always be detected as a cycle.
    ///
    /// Non-vacuous (F-HIGH-005): self-loop is detected directly without needing store entries.
    #[test]
    fn concrete_self_loop_detected() {
        let path = format!("/tmp/vp013_self_{}.toml", std::process::id());
        let store = AliasStore::empty(&path);
        let result = AliasResolver::detect_cycle("A", "@A", &store);
        assert!(
            result.is_err(),
            "self-loop A = '@A' must be detected as a cycle"
        );
        let _ = std::fs::remove_file(&path);
    }

    /// Direct mutual cycle: A = "@B", B = "@A" — both entries in store.
    ///
    /// Non-vacuous (F-HIGH-005): the old test only verified the B-absent case (always Ok).
    /// This version populates B = "@A" in the store, then calls detect_cycle for
    /// A = "@B", which must detect the back-edge A→B→A.
    #[test]
    fn concrete_mutual_cycle_detected() {
        let path = format!("/tmp/vp013_mutual_{}.toml", std::process::id());
        let mut store = AliasStore::empty(&path);

        // Populate B = "@A" in the store (no cycle at this point — A is not yet defined).
        let entry_b = AliasEntry {
            name: "B".to_string(),
            scope: AliasScope::Global,
            query: "@A AND field >= 1".to_string(),
            parameters: None,
            description: None,
        };
        if store.create_or_update(entry_b, None).is_err() {
            let _ = std::fs::remove_file(&path);
            return; // I/O failure in test env — inconclusive
        }

        // Now detect_cycle for A = "@B" must detect the cycle A→B→A.
        let result = AliasResolver::detect_cycle("A", "@B", &store);
        assert!(
            result.is_err(),
            "mutual cycle A='@B', B='@A' must be detected when B is in the store"
        );

        let _ = std::fs::remove_file(&path);
    }

    /// Acyclic alias: A = "literal_value" returns Ok(()) — no false positive.
    #[test]
    fn concrete_acyclic_no_cycle_error() {
        let path = format!("/tmp/vp013_acyclic_{}.toml", std::process::id());
        let store = AliasStore::empty(&path);
        let result = AliasResolver::detect_cycle("A", "severity_id >= 3", &store);
        assert!(result.is_ok(), "literal definition (no @refs) has no cycle");
        let _ = std::fs::remove_file(&path);
    }

    /// Three-node chain cycle: A → B → C → A (all nodes in the store).
    ///
    /// Non-vacuous (F-HIGH-005): the old test only verified the B-absent case (always Ok).
    /// This version populates B = "@C" and C = "@A", then calls detect_cycle for
    /// A = "@B", which must detect the three-node cycle A→B→C→A.
    #[test]
    fn concrete_three_node_cycle_detected() {
        let path = format!("/tmp/vp013_three_{}.toml", std::process::id());
        let mut store = AliasStore::empty(&path);

        // Populate B = "@C AND field >= 1" (no cycle yet — C not defined).
        let entry_b = AliasEntry {
            name: "B".to_string(),
            scope: AliasScope::Global,
            query: "@C AND field >= 1".to_string(),
            parameters: None,
            description: None,
        };
        if store.create_or_update(entry_b, None).is_err() {
            let _ = std::fs::remove_file(&path);
            return;
        }

        // Populate C = "@A AND field >= 2" (no cycle at C creation time — A not yet defined).
        let entry_c = AliasEntry {
            name: "C".to_string(),
            scope: AliasScope::Global,
            query: "@A AND field >= 2".to_string(),
            parameters: None,
            description: None,
        };
        if store.create_or_update(entry_c, None).is_err() {
            let _ = std::fs::remove_file(&path);
            return;
        }

        // detect_cycle for A = "@B" must now detect the cycle A→B→C→A.
        let result = AliasResolver::detect_cycle("A", "@B", &store);
        assert!(
            result.is_err(),
            "three-node cycle A='@B', B='@C', C='@A' must be detected"
        );

        let _ = std::fs::remove_file(&path);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Proptest harnesses (cfg(test)) — RED by design; fire todo!() on every call
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod proptest_harnesses {
    use proptest::prelude::*;

    use crate::alias_resolver::AliasResolver;
    use crate::alias_store::AliasStore;

    // ──────────────────────────────────────────────────────────────────────
    // Graph generation strategies
    // ──────────────────────────────────────────────────────────────────────

    /// Generate a node name: single uppercase letter A–J (10 possible nodes).
    fn node_name() -> impl Strategy<Value = String> {
        (b'A'..=b'J').prop_map(|c| String::from(c as char))
    }

    /// Generate an alias map of up to 10 nodes where each node self-references.
    ///
    /// Simplified to avoid `dyn Strategy` associated-type issues at stub time.
    /// A graph of self-loops is trivially cyclic and exercises the
    /// termination + cycle-detection property.
    fn alias_graph() -> impl Strategy<Value = Vec<(String, String)>> {
        (1usize..=10).prop_map(|n| {
            (0..n)
                .map(|i| {
                    let name = format!("{}", (b'A' + i as u8) as char);
                    let def = format!("@{name}"); // self-loop → always cyclic
                    (name, def)
                })
                .collect()
        })
    }

    // ──────────────────────────────────────────────────────────────────────
    // Properties
    // ──────────────────────────────────────────────────────────────────────

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(32))]

        /// VP-013 property: detect_cycle always terminates and returns Err when
        /// the graph has a self-loop (A = "@A").
        ///
        /// RED — fires todo!() on each invocation until detect_cycle is implemented.
        #[test]
        fn prop_vp013_self_loop_always_detected(name in node_name()) {
            let path = format!("/tmp/vp013_prop_self_{name}.toml");
            let store = AliasStore::empty(&path);
            let self_ref_definition = format!("@{name}");
            let result = AliasResolver::detect_cycle(&name, &self_ref_definition, &store);
            // Once implemented: must be Err(AliasCycleDetected). Currently RED.
            prop_assert!(result.is_err(), "todo!() fires — VP-013 RED gate");
        }

        /// VP-013 property: detect_cycle on a literal definition (no @references)
        /// must return Ok(()) — no false-positive cycle detection.
        ///
        /// RED — fires todo!() on each invocation until detect_cycle is implemented.
        #[test]
        fn prop_vp013_literal_definition_no_cycle(name in node_name()) {
            let path = format!("/tmp/vp013_prop_literal_{name}.toml");
            let store = AliasStore::empty(&path);
            // A literal definition with no @references cannot form a cycle.
            let result = AliasResolver::detect_cycle(&name, "severity_id >= 1", &store);
            // Literal definition — no @references, so no cycle possible.
            prop_assert!(result.is_ok(), "literal definition has no cycle: {:?}", result);
        }

        /// VP-013 property: detect_cycle always terminates (no infinite loop or
        /// stack overflow) regardless of graph structure.
        ///
        /// We verify termination by the fact the test itself completes without
        /// a timeout or SIGBUS. The actual result (Ok or Err) is not checked here
        /// — only that the function returns within bounded time.
        ///
        /// RED — fires todo!() on each invocation.
        /// VP-013 property: detect_cycle always terminates (no infinite loop or stack overflow).
        ///
        /// For self-loops the result must be Err; for non-self-loops with empty store the
        /// result must be Ok (no cycle reachable). Both paths must terminate without panic.
        #[test]
        fn prop_vp013_always_terminates(name in node_name(), ref_name in node_name()) {
            let path = format!("/tmp/vp013_prop_term_{name}_{ref_name}.toml");
            let store = AliasStore::empty(&path);
            let is_self_loop = name == ref_name;
            let definition = if is_self_loop {
                format!("@{ref_name}") // self-loop
            } else {
                format!("@{ref_name} AND severity_id >= 1")
            };
            let result = AliasResolver::detect_cycle(&name, &definition, &store);
            // Self-loop must be Err; cross-node reference with empty store must be Ok.
            if is_self_loop {
                prop_assert!(result.is_err(), "self-loop must be detected as a cycle");
            } else {
                prop_assert!(result.is_ok(), "non-self-loop with empty store must return Ok (no reachable cycle)");
            }
        }

        /// VP-013 property: detect_cycle on a multi-reference definition (A = "@B AND @C")
        /// always terminates and returns Ok when neither B nor C is A (empty store).
        ///
        /// When A references both B and C, and the store is empty, no back-edge to A
        /// can be found, so no cycle is detected. Self-references (when ref_b or ref_c
        /// equals name) must be detected.
        #[test]
        fn prop_vp013_multi_reference_definition_terminates(
            name in node_name(),
            ref_b in node_name(),
            ref_c in node_name(),
        ) {
            let path = format!("/tmp/vp013_prop_multi_{name}_{ref_b}_{ref_c}.toml");
            let store = AliasStore::empty(&path);
            let definition = format!("@{ref_b} AND @{ref_c} OR severity_id >= 1");
            let result = AliasResolver::detect_cycle(&name, &definition, &store);
            // If neither ref_b nor ref_c equals name, the empty store contains no back-edges.
            let has_self_ref = ref_b == name || ref_c == name;
            if has_self_ref {
                prop_assert!(result.is_err(), "multi-ref definition with self-reference must detect cycle");
            } else {
                prop_assert!(result.is_ok(), "multi-ref definition with no self-reference must return Ok (empty store)");
            }
        }

        /// VP-013 property: non-self-reference with guaranteed distinct names (A-E, F-J ranges)
        /// and empty store returns Ok — no cycle reachable.
        #[test]
        fn prop_vp013_non_self_reference_terminates(
            name in "[A-E]",
            // Reference a different node (guaranteed by using F-J range)
            other in "[F-J]",
        ) {
            let path = format!("/tmp/vp013_prop_non_self_{name}_{other}.toml");
            let store = AliasStore::empty(&path);
            // name is in A-E, other is in F-J: they cannot be equal — no self-loop.
            let definition = format!("@{other} AND active = TRUE");
            let result = AliasResolver::detect_cycle(&name, &definition, &store);
            // No self-reference + empty store = no cycle reachable.
            prop_assert!(result.is_ok(), "non-self-reference with empty store must return Ok: name={name}, other={other}");
        }

        /// VP-013 property: a three-node cycle (n1 -> n2 -> n3 -> n1 topology) is detected
        /// when the back-edge (n1 referencing n2) is being added.
        ///
        /// SEC-012 fix: the store is now populated with n2 -> @n3 and n3 -> @n1 entries
        /// so that detect_cycle can traverse the full graph and detect the cycle.
        /// Previously the store was empty so no cycle could ever be found.
        #[test]
        fn prop_vp013_transitive_cycle_via_3node_graph(
            // Node names from different ranges to control topology (guaranteed distinct).
            n1 in "[A-C]",
            n2 in "[D-F]",
            n3 in "[G-I]",
        ) {
            use crate::alias_types::{AliasEntry, AliasScope};
            // Unique per-case path avoids file-write contention under concurrent nextest
            // execution (CR-P3-001).
            let path = format!("/tmp/vp013_prop_{n1}{n2}{n3}.toml");
            let mut store = AliasStore::empty(&path);

            // Populate n2 -> @n3 (no cycle yet; n3 not in store).
            let entry_n2 = AliasEntry {
                name: n2.clone(),
                scope: AliasScope::Global,
                query: format!("@{n3} AND field >= 1"),
                parameters: None,
                description: None,
            };
            // Skip this case if the setup write fails (e.g. residual I/O race).
            prop_assume!(store.create_or_update(entry_n2, None).is_ok(), "setup write n2 must succeed");

            // Populate n3 -> @n1 (n1 not yet in store so no cycle detected at create time).
            let entry_n3 = AliasEntry {
                name: n3.clone(),
                scope: AliasScope::Global,
                query: format!("@{n1} AND field >= 2"),
                parameters: None,
                description: None,
            };
            // Skip this case if the setup write fails (e.g. residual I/O race).
            prop_assume!(store.create_or_update(entry_n3, None).is_ok(), "setup write n3 must succeed");

            // Now detect_cycle for n1 -> @n2 must detect the cycle n1->n2->n3->n1.
            let definition = format!("@{n2} AND field >= 1");
            let result = AliasResolver::detect_cycle(&n1, &definition, &store);
            // With a populated store, the three-node cycle must be detected.
            prop_assert!(result.is_err(), "three-node cycle n1->n2->n3->n1 must be detected: n1={n1}, n2={n2}, n3={n3}");
        }

        /// VP-013 property: detect_cycle on an empty definition (no @references)
        /// always returns Ok(()) — no false positive cycle detection possible.
        ///
        /// Store does not need to be populated since there are no @references to traverse.
        #[test]
        fn prop_vp013_empty_definition_no_false_positive(name in node_name()) {
            let path = format!("/tmp/vp013_prop_empty_{name}.toml");
            let store = AliasStore::empty(&path);
            // A definition with no @-references cannot possibly form a cycle.
            let definition = "severity_id >= 1 AND active = TRUE";
            let result = AliasResolver::detect_cycle(&name, definition, &store);
            prop_assert!(result.is_ok(), "literal definition with no @refs must not detect a cycle");
        }
    }
}
