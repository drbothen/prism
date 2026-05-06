//! VP-021 fuzz target: PrismQlParser never panics on arbitrary byte input.
//!
//! Property (VP-021): For every byte sequence `data`,
//! `PrismQlParser::parse(data)` returns `Ok(Ast)` or `Err(Vec<ParseError>)`
//! without panicking. `unwrap()` and `expect()` inside the parser are
//! forbidden (S-3.01 §Architecture Compliance Rules).
//!
//! The parser must gracefully handle:
//! - Arbitrary byte sequences (including non-UTF-8)
//! - Extremely long inputs (up to and beyond 64KB)
//! - Inputs with deeply nested parentheses (depth bomb)
//! - Inputs designed to trigger Chumsky backtracking explosions
//! - SQL injection probes
//! - Path traversal characters in source refs (`/`, `\`, `..`)
//!
//! Source BC: BC-2.11.002 / BC-2.11.003 / BC-2.11.004 / BC-2.11.006
//! VP: VP-021
//! Method: cargo-fuzz (libFuzzer), coverage-guided
//! Runtime: 30 minutes minimum initial; continuous in CI
//!
//! Story: S-3.01

#![no_main]

use libfuzzer_sys::fuzz_target;
use prism_query::PrismQlParser;

fuzz_target!(|data: &[u8]| {
    // Only attempt parse if data is valid UTF-8.
    // PrismQlParser::parse takes &str; non-UTF-8 is a caller precondition.
    // The panic-freedom property (VP-021) applies to valid UTF-8 strings of
    // arbitrary content — invalid UTF-8 bytes are skipped by this harness
    // (consistent with the spec_parser.rs fuzz target pattern in this repo).
    if let Ok(s) = std::str::from_utf8(data) {
        // PrismQlParser::parse MUST NOT panic.
        // It must return Ok(Ast) or Err(Vec<ParseError>) for any UTF-8 input.
        let _ = PrismQlParser::parse(s);
    }
});
