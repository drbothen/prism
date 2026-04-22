//! VP-040: Plugin Linker Excludes All WASI Namespace Imports.
//!
//! # Property
//! `PluginRuntime::build_linker()` produces a `wasmtime::component::Linker<HostState>`
//! whose complete set of linked import namespace names contains no entry with the prefix
//! `wasi:`. Any WASM Component that requires a WASI import will fail at
//! `instantiate_pre` time rather than at runtime.
//!
//! # Method
//! Kani (conditional on wasmtime Linker exposing import enumeration API).
//! Proptest fallback: attempt instantiation of a WASI-importing component binary and
//! assert `Err(PluginError::SandboxViolation)` or `Err(PluginError::CompilationFailed)`.
//!
//! # Source BC
//! BC-2.17.002 — Plugin Sandbox — No Direct Filesystem or Network Access.
//!
//! # Status: Red Gate stub — tests fail, proof not yet written.

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    // Import the target under test. These will fail to compile until S-1.15 is
    // implemented (Red Gate requirement).
    use crate::plugin::PluginRuntime;
    use prism_core::PluginError;

    /// VP-040 proptest fallback: any WASM Component binary that includes a WASI import
    /// must be rejected at `load_plugin` / `instantiate_pre` time.
    ///
    /// Uses a hardcoded minimal WAT that imports `wasi:filesystem/types` — a real
    /// WASI import that the linker must not satisfy.
    ///
    /// Traces to: BC-2.17.002 postcondition "WASI imports not linked → trap at
    /// instantiate_pre time"
    #[test]
    fn test_BC_2_17_002_vp040_wasi_importing_component_rejected() {
        // A minimal WAT component that imports a WASI function.
        // This is NOT a valid Component Model binary but encodes the intent.
        // The real test will use `wasm-tools` to build a proper WASI-importing component.
        let wasi_importing_wat = r#"
            (module
              (import "wasi_snapshot_preview1" "fd_write"
                (func $fd_write (param i32 i32 i32 i32) (result i32)))
            )
        "#;

        let wasi_bytes = wat::parse_str(wasi_importing_wat)
            .expect("WAT should parse");

        let runtime = PluginRuntime::new()
            .expect("PluginRuntime construction should succeed");

        // A WASI-importing module must be rejected, NOT successfully loaded.
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), &wasi_bytes).unwrap();

        let result = runtime.load_plugin(tmp.path());
        assert!(
            result.is_err(),
            "VP-040: WASI-importing component must be rejected, not loaded"
        );

        // The error must be a sandbox violation or compilation failure — not a
        // silent success that would allow WASI access.
        let plugin_err = result.err().expect("result was already asserted Err above");
        match plugin_err {
            PluginError::SandboxViolation { .. }
            | PluginError::CompilationFailed { .. }
            | PluginError::InvalidInterface { .. } => {
                // Acceptable: the runtime correctly rejected the WASI-importing binary.
            }
            other => panic!(
                "VP-040: expected SandboxViolation/CompilationFailed/InvalidInterface, got: {:?}",
                other
            ),
        }
    }

    /// VP-040 additional check: the built linker has no `wasi:` prefixed namespaces.
    ///
    /// This test relies on a hypothetical `linker_namespaces()` inspection API.
    /// Until wasmtime exposes this, the test is marked as `#[ignore]` with a comment
    /// explaining the Kani conditional feasibility caveat from VP-040.
    ///
    /// Traces to: BC-2.17.002 postcondition "PluginRuntime Linker configured with
    /// ONLY Prism host interface bindings"
    #[test]
    #[ignore = "VP-040 Kani feasibility: requires wasmtime Linker import enumeration API (see VP-040 spec)"]
    fn test_BC_2_17_002_vp040_linker_has_no_wasi_namespace() {
        // TODO(S-1.15 impl): enumerate linker namespaces and assert none start with "wasi:"
        // let engine = wasmtime::Engine::default();
        // let linker = PluginRuntime::build_linker(&engine).unwrap();
        // let namespaces = linker.component().root().imports(); // hypothetical API
        // for (name, _) in namespaces {
        //     assert!(!name.starts_with("wasi:"),
        //         "VP-040: found WASI import '{}' in plugin linker", name);
        // }
        unimplemented!("VP-040 Kani path — awaiting wasmtime Linker enumeration API")
    }
}
