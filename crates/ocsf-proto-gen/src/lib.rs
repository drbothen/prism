//! Generate Protocol Buffer definitions from OCSF JSON schema.
//!
//! `ocsf-proto-gen` reads the [OCSF](https://schema.ocsf.io/) (Open Cybersecurity
//! Schema Framework) schema export and generates deterministic `.proto` files
//! suitable for compilation with `protoc` or `prost-build`.
//!
//! # Features
//!
//! - Generates proto3 messages for OCSF event classes and shared objects
//! - Generates per-class and shared-object enum definitions
//! - Resolves transitive object dependencies automatically
//! - Skips deprecated attributes
//! - Maps `json_t` to `string` (avoids `google.protobuf.Struct` compatibility issues)
//! - Handles extension-prefixed objects (e.g., `win/win_service`)
//! - Deterministic output: byte-identical across runs
//!
//! # Usage
//!
//! ```no_run
//! use std::path::Path;
//!
//! let schema = ocsf_proto_gen::schema::load_schema(Path::new("schema.json"))?;
//! let stats = ocsf_proto_gen::codegen::generate(
//!     &schema,
//!     &["authentication".to_string(), "security_finding".to_string()],
//!     Path::new("output/"),
//! )?;
//! eprintln!("Generated {} classes, {} objects", stats.classes_generated, stats.objects_generated);
//! # Ok::<(), ocsf_proto_gen::error::Error>(())
//! ```

pub mod codegen;
pub mod error;
pub mod schema;
pub mod type_map;
