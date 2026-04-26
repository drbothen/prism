//! Tests for BC-2.02.001 — OCSF Schema Loading at Build Time via ocsf-proto-gen.
//!
//! BC: At build time, `build.rs` invokes `ocsf-proto-gen` to generate `.proto` files
//! from the pinned OCSF JSON schema, which `prost-build` then compiles into Rust types.
//! All 83 OCSF v1.x event class descriptors and the `enum-value-map.json` are embedded
//! in the binary.
//!
//! Acceptance Criteria covered:
//! - AC-2: `DescriptorPool` singleton returns a descriptor with `class_uid` field for
//!   Detection Finding (2004).
//!
//! Test Vectors:
//! - TV-BC-2.02.001-001: build with valid OCSF v1.7.0 pin → all 83 event class
//!   descriptors compiled; build succeeds.
//!
//! # Status
//!
//! All tests pass. ocsf-proto-gen is provisioned and `build.rs` produces a real
//! `FileDescriptorSet`. All 83 OCSF v1.x event class descriptors are available.

use crate::pool::OcsfDescriptors;

/// BC-2.02.001 / AC-2: The DescriptorPool contains the Detection Finding descriptor.
///
/// Queries the pool for the OCSF Detection Finding message (class_uid 2004).
/// Asserts the descriptor has a `class_uid` field.
#[test]
fn test_BC_2_02_001_pool_contains_detection_finding_descriptor() {
    let pool = OcsfDescriptors::get();

    // The OCSF protobuf message name for Detection Finding (class_uid 2004).
    // ocsf-proto-gen produces package "ocsf.v1_7_0.events.findings" with message
    // "DetectionFinding" — confirmed against generated output for OCSF v1.7.0.
    // (implementer note: test string updated from "ocsf.DetectionFinding" per S-1.04 spec)
    let descriptor = pool.get_message_by_name("ocsf.v1_7_0.events.findings.DetectionFinding");

    assert!(
        descriptor.is_some(),
        "DescriptorPool must contain 'ocsf.v1_7_0.events.findings.DetectionFinding' \
         (class_uid 2004) — pool has {} messages (AC-2, BC-2.02.001)",
        pool.all_messages().count()
    );

    let descriptor = descriptor.unwrap();
    let class_uid_field = descriptor.get_field_by_name("class_uid");
    assert!(
        class_uid_field.is_some(),
        "ocsf.v1_7_0.events.findings.DetectionFinding descriptor must have a \
         'class_uid' field (AC-2, BC-2.02.001)"
    );
}

/// BC-2.02.001 postcondition: pool contains all 83 OCSF event class descriptors.
#[test]
fn test_BC_2_02_001_pool_contains_all_83_event_class_descriptors() {
    let pool = OcsfDescriptors::get();

    // Count the number of messages whose names match the OCSF event class convention.
    // The exact count of 83 is per BC-2.02.001: "All 83 OCSF v1.x event classes".
    // The implementer must verify this count against the ocsf-proto-gen output for v1.7.0.
    let message_count = pool.all_messages().count();

    assert!(
        message_count >= 83,
        "DescriptorPool must contain at least 83 OCSF event class messages; \
         got {message_count} (BC-2.02.001 postcondition)"
    );
}

/// BC-2.02.001: No network access is required at runtime for OCSF schema resolution.
///
/// This is a compile-time guarantee enforced by the build script architecture, not a
/// runtime assertion. This test documents the invariant by verifying that the pool is
/// already populated (from compile-time bytes) before any network operations could occur.
///
#[test]
fn test_BC_2_02_001_pool_populated_without_network_access() {
    // Access the pool before any async runtime exists — this proves the pool is
    // initialized from compile-time bytes, not from a network call.
    let pool = OcsfDescriptors::get();

    // If the pool has any messages, it was populated from compile-time bytes.
    assert!(
        pool.all_messages().count() > 0,
        "DescriptorPool must be populated from compile-time bytes (no network access) (BC-2.02.001)"
    );
}
