---
document_type: verification-property
level: L4
version: "1.0"
status: draft
producer: architect
timestamp: 2026-04-15T12:00:00
phase: 1b
inputs: [prd.md, architecture/query-engine.md]
traces_to: prd.md
source_bc: BC-2.11.006
module: prism-query
proof_method: fuzz
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
removal_reason: null
removed: null
withdrawal_reason: null
---

# VP-021: PrismQL Parser Never Panics on Arbitrary Input

## Property Statement

For all byte sequences `b`, if `b` is valid UTF-8, then `AxiqlParser::parse(b)` returns `Ok(Ast)` or `Err(Vec<ParseError>)` without panicking. The parser must gracefully handle all possible inputs including empty strings, maximum-length strings (64KB), deeply nested expressions, malformed unicode, and adversarial inputs designed to trigger stack overflow.

## Source Contract

- **BC:** BC-2.11.006 — Query Security Limits Enforcement
- **Invariant:** DI-019 — Query Security Limits

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| fuzz | cargo-fuzz (libFuzzer) | No — coverage-guided mutation | Continuous corpus expansion |

## Fuzz Target Skeleton

```rust
// prism-query/fuzz/fuzz_targets/fuzz_prismql_parser.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use prism_query::parser::AxiqlParser;

fuzz_target!(|data: &[u8]| {
    if let Ok(input) = std::str::from_utf8(data) {
        // Enforce the 64KB security limit
        if input.len() <= 65536 {
            // Must not panic — Ok or Err are both acceptable
            let _ = AxiqlParser::parse(input);
        }
    }
});
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Unbounded | Coverage-guided mutation explores effectively |
| Proof complexity | N/A (fuzz, not proof) | Panics detected by sanitizer |
| Tool support | Full | libFuzzer with AddressSanitizer |
| Estimated corpus time | 30 min initial, continuous thereafter | Parser parsers are excellent fuzz targets |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-04-15 | architect |
