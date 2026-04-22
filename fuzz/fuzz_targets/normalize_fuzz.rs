//! VP-022 fuzz target — OCSF normalizer never panics on arbitrary input.
//!
//! # Property (VP-022)
//!
//! For every byte sequence `b`:
//!   `OcsfNormalizer::normalize(b)` returns `Ok(DynamicMessage)` or `Err(NormalizerError)`
//!   WITHOUT panicking.
//!
//! The normalizer must gracefully handle:
//! - Empty payloads
//! - Invalid UTF-8
//! - Invalid JSON
//! - Unexpected field types
//! - Deeply nested structures
//! - Adversarial records designed to trigger numeric or string panics
//!
//! # Run Requirements (VP-022 feasibility assessment)
//!
//! - Minimum: 30 minutes initial run with no panics found.
//! - Continuous: run in CI after every merge to main.
//! - Tool: cargo-fuzz (libFuzzer) + AddressSanitizer.
//!
//! # How to run
//!
//! ```sh
//! # Install cargo-fuzz if not present:
//! cargo install cargo-fuzz
//!
//! # Run the fuzz target (requires nightly toolchain):
//! cargo +nightly fuzz run normalize_fuzz -- -max_total_time=1800
//! ```
//!
//! # Fuzz corpus seeds
//!
//! Seed corpus entries live in `fuzz/corpus/normalize_fuzz/`. Pre-populate with:
//! - `{}` (empty JSON object — EC-02-003)
//! - `{"detection_id": "abc123", "severity": "High"}` (well-formed CrowdStrike)
//! - `{"severity_id": "not_a_number"}` (type mismatch — EC-02-004)
//! - A deeply nested object (stack overflow probe)
//! - A 1MB+ string value (allocation probe)

#![no_main]

use libfuzzer_sys::fuzz_target;
use prism_ocsf::OcsfNormalizer;

fuzz_target!(|data: &[u8]| {
    // VP-022: No panic on arbitrary bytes.
    //
    // Strategy:
    // 1. Attempt to parse bytes as UTF-8 and then as JSON.
    // 2. If parsing succeeds, call normalize() with each of the known sensor types.
    // 3. Assert: no panic. All error paths must return Err, never unwrap()/expect()/panic!().

    let normalizer = OcsfNormalizer::new();

    // Attempt UTF-8 decode — arbitrary bytes may not be valid UTF-8.
    let json_value = match std::str::from_utf8(data) {
        Ok(s) => match serde_json::from_str(s) {
            Ok(v) => v,
            Err(_) => serde_json::Value::Null,
        },
        Err(_) => serde_json::Value::Null,
    };

    // Exercise all sensor/record_type combinations that have OCSF mappings.
    // VP-022 requires no panic on ANY input regardless of sensor+type.
    let _ = normalizer.normalize("crowdstrike", "detection", json_value.clone());
    let _ = normalizer.normalize("crowdstrike", "incident", json_value.clone());
    let _ = normalizer.normalize("cyberint", "alert", json_value.clone());
    let _ = normalizer.normalize("claroty", "alert", json_value.clone());
    let _ = normalizer.normalize("claroty", "device", json_value.clone());
    let _ = normalizer.normalize("claroty", "vulnerability", json_value.clone());
    let _ = normalizer.normalize("armis", "device", json_value.clone());
    let _ = normalizer.normalize("armis", "alert", json_value.clone());

    // Also exercise the unknown-sensor path to verify Err is returned cleanly.
    let _ = normalizer.normalize("unknown_fuzz_sensor", "unknown_type", json_value);
});
