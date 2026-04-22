//! build.rs — OCSF schema compilation entry point.
//!
//! # Stub (S-1.04 Red Gate)
//!
//! This build script is a stub. It documents the build-time contract required by
//! BC-2.02.001 and BC-2.02.009 and emits the `ocsf_version.txt` file that
//! `src/version.rs` reads via `include_str!()`.
//!
//! ## What the real implementation must do (implementer checklist):
//!
//! 1. Invoke `ocsf_proto_gen::compile_ocsf_schema(OCSF_PINNED_VERSION)` to generate
//!    `.proto` files in `OUT_DIR` from the bundled OCSF JSON schema files.
//!    (BC-2.02.001 postcondition 1: all 83 OCSF v1.x event class descriptors compiled)
//!
//! 2. Invoke `prost_build::Config::new().compile_protos(...)` on the generated files
//!    to produce Rust types + a `FileDescriptorSet` binary blob.
//!
//! 3. Write the pinned OCSF version string to `$OUT_DIR/ocsf_version.txt` so that
//!    `src/version.rs` can `include_str!()` it at compile time.
//!    (BC-2.02.009 invariant: version baked in, immutable at runtime)
//!
//! 4. Emit `cargo:rerun-if-changed=ocsf-schema/` so that the build reruns when
//!    schema files change (EC-02-001).
//!
//! ## Pinned OCSF version
//!
//! The story spec (S-1.04 task 2) and BC-2.02.009 canonical test vector TV-BC-2.02.009-001
//! call for pinning to v1.7.0. The implementer MUST verify each class_uid in
//! EventClassSelector against the compiled descriptors for this version. Note that
//! Security Finding (class_uid 2001) is DEPRECATED since OCSF v1.1.0 and MUST NOT be used.

use std::env;
use std::fs;
use std::path::PathBuf;

/// Pinned OCSF schema version. Must match the version passed to ocsf-proto-gen.
/// BC-2.02.009: upgrading this value requires a new Prism release.
const OCSF_PINNED_VERSION: &str = "1.7.0";

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR must be set by cargo"));

    // Write the pinned version string for compile-time inclusion by src/version.rs.
    // BC-2.02.009 postcondition: ocsf_version() returns the compile-time pinned value.
    let version_path = out_dir.join("ocsf_version.txt");
    fs::write(&version_path, OCSF_PINNED_VERSION)
        .expect("build.rs: failed to write ocsf_version.txt to OUT_DIR");

    // Rerun if any OCSF schema file changes (EC-02-001).
    println!("cargo:rerun-if-changed=ocsf-schema/");
    println!("cargo:rerun-if-changed=build.rs");

    // -------------------------------------------------------------------------
    // STUB: ocsf-proto-gen invocation
    //
    // The real implementation must replace this block with:
    //
    //   ocsf_proto_gen::compile_ocsf_schema(OCSF_PINNED_VERSION, &out_dir)
    //       .expect("ocsf-proto-gen: failed to compile OCSF schema");
    //
    //   prost_build::Config::new()
    //       .file_descriptor_set_path(out_dir.join("ocsf_descriptor.bin"))
    //       .compile_protos(
    //           &[out_dir.join("ocsf_all.proto")],
    //           &[&out_dir],
    //       )
    //       .expect("prost-build: failed to compile generated .proto files");
    //
    // Until ocsf-proto-gen is available, we write a minimal descriptor placeholder
    // so that src/pool.rs can compile as a stub. The tests will fail because the
    // placeholder does not contain real OCSF descriptors.
    // -------------------------------------------------------------------------

    // Write a zero-byte placeholder for the descriptor set binary.
    // src/pool.rs reads this path at compile time via include_bytes!().
    // The empty descriptor will cause DescriptorPool initialization to return
    // an empty pool, which will cause test assertions to fail — as required for
    // Red Gate.
    let descriptor_path = out_dir.join("ocsf_descriptor.bin");
    fs::write(&descriptor_path, b"")
        .expect("build.rs: failed to write ocsf_descriptor.bin placeholder");
}
