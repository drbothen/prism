---
document_type: verification-property
level: L4
version: "1.1"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-1.10-prompt-injection-defense.md]
input-hash: "a898cd7"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.09.003
module: prism-security
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

# VP-038: Injection Scanner Never Panics on Arbitrary Input Strings

## Property Statement

For every byte sequence `b`, `InjectionScanner::scan(b)` returns a `ScanResult`
without panicking. The scanner must gracefully handle empty input, invalid UTF-8,
extreme length, Unicode edge cases (surrogates, combining marks), and adversarial
regex-bomb inputs — never panic, stall, or infinite-loop.

## Source Contract

- **Anchor Story:** `S-1.10-prompt-injection-defense.md`
- **Source BC:** BC-2.09.003 — Suspicious Pattern Detection via Regex
- **Module:** prism-security
- **Category:** Safety

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| fuzz | cargo-fuzz (libFuzzer) | No — coverage-guided mutation | Continuous corpus expansion |

## Proof Harness Skeleton

```rust
// [TODO: fuzz target skeleton — author during Phase 5 formal-verify]
// prism-security/fuzz/fuzz_targets/fuzz_injection_scanner.rs
//
// fuzz_target!(|data: &[u8]| {
//     let _ = InjectionScanner::scan(data);
// });
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | No | Coverage-guided |
| Tool support? | Full | libFuzzer + ASan |
| Execution time budget | 30 min initial, continuous in CI | Regex scanners fuzz well; use regex crate for ReDoS resistance |
| Assumptions required | Regex engine is bounded-time (not PCRE-style backtracking) | regex crate meets this |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.1 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
