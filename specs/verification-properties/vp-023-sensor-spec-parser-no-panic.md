---
document_type: verification-property
level: L4
version: "1.3"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-1.11-spec-loading.md]
input-hash: "9c3e1e2"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.16.001
module: prism-spec-engine
priority: P0
proof_method: fuzz
verification_method: fuzz
feasibility: high
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-023: Sensor Spec Parser Never Panics on Arbitrary TOML

## Property Statement

For every byte sequence `b`, `SpecParser::parse(b)` returns `Ok(SensorSpec)` or
`Err(SpecParseError)` without panicking. The parser must gracefully handle malformed
TOML, invalid UTF-8, missing required keys, extra unknown keys, circular variable
references, and adversarial inputs designed to trigger recursion or integer overflow.

## Source Contract

- **Anchor Story:** `S-1.11`
- **Source BC:** BC-2.16.001 — Sensor Spec File Loading — Parse TOML, Validate Schema, Register Tables
- **Module:** prism-spec-engine
- **Category:** Safety

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| fuzz | cargo-fuzz (libFuzzer) | No — coverage-guided mutation | Continuous corpus expansion |

## Proof Harness Skeleton

```rust
// [TODO: fuzz target skeleton — author during Phase 5 formal-verify]
// prism-spec-engine/fuzz/fuzz_targets/fuzz_spec_parser.rs
//
// fuzz_target!(|data: &[u8]| {
//     if let Ok(s) = std::str::from_utf8(data) {
//         let _ = SpecParser::parse(s);
//     }
// });
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | No | Coverage-guided |
| Tool support? | Full | libFuzzer + ASan |
| Execution time budget | 30 min initial, continuous in CI | TOML parsers fuzz well |
| Assumptions required | None | Panic = failure |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.3 | pass-88-remediation | 2026-04-21 | architect | F88-012: Anchor Story normalized from slug form (S-1.11-spec-loading.md) to pure ID (S-1.11). |
| 1.2 | pass-87-remediation | 2026-04-21 | architect | F87-006: Source BC label corrected "Sensor Spec File Loading" → "Sensor Spec File Loading — Parse TOML, Validate Schema, Register Tables" (matches BC-2.16.001 H1). |
| 1.1 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
