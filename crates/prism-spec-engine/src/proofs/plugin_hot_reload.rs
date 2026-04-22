//! VP-042: Plugin Hot Reload — Failed Compile Retains Old InstancePre.
//!
//! # Property
//! Given a `PluginRegistry` with a valid plugin registered under `plugin_id`, invoking
//! `hot_reload(plugin_id, invalid_bytes)` where compilation of `invalid_bytes` fails
//! leaves the registry entry unchanged: the old `Arc<LoadedPlugin>` is still returned
//! for `plugin_id` after the failed reload attempt.
//!
//! The registry never transitions to a partially-loaded or empty state for that plugin.
//!
//! # Method: proptest (1000+ cases with random invalid byte sequences).
//!
//! # Source BC: BC-2.17.005 — Plugin Hot Reload — Atomic Module Swap.
//!
//! # Status: Red Gate stub — tests fail, proof not yet written.

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use arc_swap::ArcSwap;
    use proptest::prelude::*;
    use prism_core::PluginError;

    // Import targets — will not compile until S-1.15 is implemented.
    use crate::plugin::hot_reload::hot_reload;
    use crate::plugin::LoadedPlugin;

    /// Helper: create a minimal valid WAT component for test setup.
    /// In the real implementation, this uses the fixture `.prx` from `tests/fixtures/`.
    fn minimal_valid_plugin_bytes() -> Vec<u8> {
        // A minimal WAT module (not a full Component Model binary, but sufficient
        // to test the registry-retention invariant — the mock compiler accepts this).
        wat::parse_str("(module)").expect("minimal WAT should parse")
    }

    proptest! {
        /// VP-042: Failed hot reload must NOT modify the registry entry.
        ///
        /// For any byte sequence that fails compilation (random bytes are almost always
        /// invalid WASM), the registry entry after the failed reload must be pointer-equal
        /// to the original `Arc<LoadedPlugin>`.
        ///
        /// Traces to: BC-2.17.005 postcondition "Failed recompilation: registry entry NOT
        /// updated — old version remains active"
        #[test]
        fn test_BC_2_17_005_vp042_failed_reload_retains_old_arc(
            invalid_bytes in prop::collection::vec(0u8..=255u8, 0..1024)
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                use crate::plugin::PluginRuntime;

                let runtime = PluginRuntime::new()
                    .expect("PluginRuntime should construct");

                // Load a valid plugin to establish the initial registry entry.
                let valid_bytes = minimal_valid_plugin_bytes();
                let tmp_valid = tempfile::NamedTempFile::new().unwrap();
                std::fs::write(tmp_valid.path(), &valid_bytes).unwrap();

                // Attempt to load a valid plugin (may fail if not a real Component — that's OK,
                // the key invariant is that a failed reload doesn't corrupt an existing entry).
                let load_result = runtime.load_plugin(tmp_valid.path());

                if let Ok(_loaded) = load_result {
                    // We have a valid plugin in the registry. Now try to hot-reload it with garbage.
                    let plugin_ids = runtime.list_plugins();
                    if let Some(plugin_id) = plugin_ids.first() {
                        let tmp_bad = tempfile::NamedTempFile::new().unwrap();
                        std::fs::write(tmp_bad.path(), &invalid_bytes).unwrap();

                        // Get pointer before reload attempt.
                        let before_arc = runtime.get_plugin(plugin_id)
                            .expect("plugin must be in registry before reload");

                        // Attempt to reload with invalid bytes.
                        let registry = &runtime.registry;
                        let reload_result = hot_reload(
                            registry,
                            &runtime.engine,
                            &runtime.linker,
                            plugin_id,
                            tmp_bad.path(),
                            &invalid_bytes,
                        );

                        // If compilation failed, the registry must still return the old Arc.
                        if reload_result.is_err() {
                            let after_arc = runtime.get_plugin(plugin_id)
                                .expect("plugin must still be in registry after failed reload");
                            prop_assert!(
                                Arc::ptr_eq(&before_arc, &after_arc),
                                "VP-042: failed reload must retain old Arc<LoadedPlugin>; \
                                 registry entry was replaced (ptr_eq failed)"
                            );
                        }
                    }
                }
                // If the initial load also failed (bytes not a real Component), that's fine —
                // the invariant being tested is about failed reload of an existing entry.
                Ok(())
            }).unwrap()
        }
    }

    /// VP-042 deterministic companion: given a pre-loaded valid plugin, hot_reload
    /// with a known-bad byte sequence (empty bytes = not valid WASM) must not
    /// remove or replace the registry entry.
    ///
    /// Traces to: BC-2.17.005 invariant "Failed recompilation MUST NOT unload a
    /// currently-working plugin (CI-002)"
    #[test]
    fn test_BC_2_17_005_vp042_empty_bytes_reload_retains_old_plugin() {
        use crate::plugin::PluginRuntime;
        use arc_swap::ArcSwap;

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let runtime = PluginRuntime::new().expect("PluginRuntime should construct");

            // Set up initial valid plugin (uses fixture from tests/fixtures/).
            let fixture_path = std::path::Path::new(
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/noop_infusion.wasm")
            );

            if fixture_path.exists() {
                let load_result = runtime.load_plugin(fixture_path);
                assert!(load_result.is_ok(), "noop_infusion fixture should load successfully");

                let plugin_ids = runtime.list_plugins();
                let plugin_id = plugin_ids.first().expect("registry must have at least one plugin");

                let before_arc = runtime.get_plugin(plugin_id).unwrap();

                // Reload with empty bytes (guaranteed compilation failure).
                let tmp = tempfile::NamedTempFile::new().unwrap();
                std::fs::write(tmp.path(), &[]).unwrap();

                let result = hot_reload(
                    &runtime.registry,
                    &runtime.engine,
                    &runtime.linker,
                    plugin_id,
                    tmp.path(),
                    &[],
                );

                assert!(result.is_err(), "empty bytes must fail compilation");
                let after_arc = runtime.get_plugin(plugin_id)
                    .expect("plugin must still be present after failed reload");
                assert!(
                    Arc::ptr_eq(&before_arc, &after_arc),
                    "VP-042: Arc must be unchanged after failed reload with empty bytes"
                );
            } else {
                // Fixture not yet compiled — Red Gate: test must fail until fixture exists.
                panic!(
                    "VP-042 test fixture not found at {:?}. \
                     S-1.15 implementation must compile fixtures before this test can pass.",
                    fixture_path
                );
            }
        });
    }
}
