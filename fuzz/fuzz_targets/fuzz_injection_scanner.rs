//! VP-038: InjectionScanner::scan() never panics on arbitrary input.
//!
//! Fuzz target: coverage-guided mutation via libFuzzer.
//! InjectionScanner must handle:
//! - Empty input
//! - Invalid UTF-8
//! - Extreme length (>10KB)
//! - Unicode edge cases (surrogates, combining marks, homoglyphs)
//! - Adversarial regex-bomb inputs
//!
//! This target must be run via: `cargo fuzz run fuzz_injection_scanner`
//!
//! Verification property: VP-038
//! Source BC: BC-2.09.003

#![no_main]

use libfuzzer_sys::fuzz_target;
use prism_security::injection_scanner::InjectionScanner;

fuzz_target!(|data: &[u8]| {
    // VP-038: InjectionScanner::scan_bytes() must never panic on arbitrary input.
    // The result is discarded; we only care that no panic occurs.
    let scanner = InjectionScanner::global();
    let _ = scanner.scan_bytes("fuzz_field", 0, data);
});
