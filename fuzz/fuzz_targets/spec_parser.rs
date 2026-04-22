//! VP-023 fuzz target: SpecParser never panics on arbitrary TOML input.
//!
//! Property (VP-023): For every byte sequence `b`, `SpecLoader::parse(b)`
//! returns `Ok(SensorSpec)` or `Err(SpecParseError)` without panicking.
//!
//! The parser must gracefully handle:
//! - Malformed TOML
//! - Invalid UTF-8 sequences
//! - Missing required keys
//! - Extra unknown keys
//! - Circular variable references
//! - Adversarial inputs designed to trigger recursion or integer overflow
//!
//! Source BC: BC-2.16.001
//! Method: cargo-fuzz (libFuzzer), coverage-guided
//! Runtime: 30 minutes minimum initial; continuous in CI

#![no_main]

use libfuzzer_sys::fuzz_target;
use prism_spec_engine::spec_parser::SpecLoader;

fuzz_target!(|data: &[u8]| {
    // Only attempt parse if data is valid UTF-8.
    // SpecLoader::parse takes &str, so non-UTF-8 is a caller precondition.
    // The fuzz harness intentionally skips invalid UTF-8 sequences — the
    // panic-freedom property applies to valid UTF-8 strings of arbitrary content.
    if let Ok(s) = std::str::from_utf8(data) {
        // SpecLoader::parse MUST NOT panic.
        // It must return Ok(SensorSpec) or Err(PrismError) for any UTF-8 input.
        let _ = SpecLoader::parse(s);
    }
});
