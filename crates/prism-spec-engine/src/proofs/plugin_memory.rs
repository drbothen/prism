//! VP-041: Plugin Memory Limit Boundary — At-Limit Succeeds, Over-Limit Traps.
//!
//! # Property
//! For any `limit_mb` in `1..=512`, a wasmtime `Store` configured via
//! `create_store_with_limit(limit_mb)` allows WASM linear memory allocation up to
//! exactly `limit_mb * 1024 * 1024` bytes and returns a trap error for any allocation
//! attempt at `limit_mb * 1024 * 1024 + 1` bytes. The boundary is exact.
//!
//! # Method: proptest (1000+ cases).
//!
//! # Source BC: BC-2.17.003 — Plugin Sandbox — Memory Limit Enforced Per Plugin Instance.
//!
//! # Status: Red Gate stub — tests fail, proof not yet written.

#[cfg(test)]
mod tests {
    use prism_core::PluginError;
    use proptest::prelude::*;

    // Import the target under test — will not compile until S-1.15 is implemented.
    use crate::plugin::sandbox::{create_store_with_limit, try_allocate_wasm_memory};

    proptest! {
        /// VP-041: For any limit_mb in 1..=512:
        /// - Allocation at exactly limit_mb MiB must succeed.
        /// - Allocation at limit_mb MiB + 1 byte must return Err(MemoryExceeded).
        ///
        /// Traces to: BC-2.17.003 postcondition "StoreLimits memory guard fires, trap caught,
        /// Err(PluginError::MemoryExceeded) returned"
        #[test]
        fn test_BC_2_17_003_vp041_memory_limit_boundary_exact(
            limit_mb in 1u64..=512u64
        ) {
            let engine = wasmtime::Engine::default();

            // At-limit: allocation of exactly limit_mb MiB must succeed.
            let at_limit_bytes = limit_mb * 1024 * 1024;
            let at_limit_result = try_allocate_wasm_memory(&engine, limit_mb, at_limit_bytes as usize);
            prop_assert!(
                at_limit_result.is_ok(),
                "VP-041: allocation at exactly {} MiB must succeed (limit_mb={})",
                limit_mb,
                limit_mb
            );

            // Over-limit: allocation of limit_mb MiB + 1 byte must trap with MemoryExceeded.
            let over_limit_bytes = limit_mb * 1024 * 1024 + 1;
            let over_limit_result = try_allocate_wasm_memory(&engine, limit_mb, over_limit_bytes as usize);
            prop_assert!(
                over_limit_result.is_err(),
                "VP-041: allocation over {} MiB must trap (limit_mb={})",
                limit_mb,
                limit_mb
            );
            let err = over_limit_result.unwrap_err();
            prop_assert!(
                matches!(err, PluginError::MemoryExceeded { limit_mb: lmb, .. } if lmb == limit_mb),
                "VP-041: error must be MemoryExceeded with limit_mb={}, got: {:?}",
                limit_mb,
                err
            );
        }
    }
}
