//! build.rs — OCSF schema compilation entry point.
//!
//! Invokes `ocsf-proto-gen` to generate `.proto` files from the bundled OCSF
//! JSON schema, then compiles them with `prost-build` to produce a
//! `FileDescriptorSet` binary consumed by `src/pool.rs` at runtime.
//!
//! # Contract
//!
//! - BC-2.02.001: all 83 OCSF v1.x event class descriptors compiled at build time.
//! - BC-2.02.009: OCSF schema version pinned at compile time; written to
//!   `$OUT_DIR/ocsf_version.txt` for `include_str!()` in `src/version.rs`.
//! - `cargo:rerun-if-changed=ocsf-schema/` triggers rebuild on schema updates.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Pinned OCSF schema version. Must match the schema directory name under
/// `ocsf-schema/`. BC-2.02.009: upgrading requires a new Prism release.
const OCSF_PINNED_VERSION: &str = "1.7.0";

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR must be set by cargo"));
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set"));

    // BC-2.02.009: bake the pinned version string into the binary.
    let version_path = out_dir.join("ocsf_version.txt");
    fs::write(&version_path, OCSF_PINNED_VERSION)
        .expect("build.rs: failed to write ocsf_version.txt to OUT_DIR");

    // Rerun triggers.
    println!("cargo:rerun-if-changed=ocsf-schema/");
    println!("cargo:rerun-if-changed=build.rs");

    // ── Step 1: Load the bundled OCSF JSON schema. ──────────────────────────
    let schema_path = manifest_dir
        .join("ocsf-schema")
        .join(OCSF_PINNED_VERSION)
        .join("schema.json");

    let schema = ocsf_proto_gen::schema::load_schema(&schema_path)
        .expect("build.rs: failed to load bundled OCSF schema JSON");

    // ── Step 2: Generate .proto files for ALL event classes. ────────────────
    let proto_out_dir = out_dir.join("ocsf_protos");
    fs::create_dir_all(&proto_out_dir).expect("build.rs: failed to create proto output directory");

    let all_class_names: Vec<String> = schema.classes.keys().cloned().collect();

    ocsf_proto_gen::codegen::generate(&schema, &all_class_names, &proto_out_dir)
        .expect("build.rs: ocsf-proto-gen failed to generate .proto files");

    // ── Step 3: Collect all generated .proto files. ──────────────────────────
    let proto_files = collect_proto_files(&proto_out_dir);
    assert!(
        !proto_files.is_empty(),
        "build.rs: ocsf-proto-gen produced no .proto files"
    );

    // ── Step 4: Compile with prost-build to produce FileDescriptorSet. ───────
    let descriptor_path = out_dir.join("ocsf_descriptor.bin");

    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor_path)
        .compile_protos(&proto_files, &[&proto_out_dir])
        .expect("build.rs: prost-build failed to compile OCSF .proto files");
}

/// Recursively collect all `.proto` file paths under `dir`.
fn collect_proto_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_proto_files_inner(dir, &mut files);
    files.sort(); // deterministic order
    files
}

fn collect_proto_files_inner(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_proto_files_inner(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("proto") {
            out.push(path);
        }
    }
}
