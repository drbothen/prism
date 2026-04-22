# VP-038 — Fuzz Harness: InjectionScanner Never Panics

**Verification Property:** VP-038  
**Source:** `fuzz/fuzz_targets/fuzz_injection_scanner.rs`  
**BC:** BC-2.09.003  
**Phase:** Scheduled for Phase 5 (coverage-guided fuzzing campaign)

## What It Tests

`InjectionScanner::scan_bytes()` must never panic on arbitrary byte input. The fuzzer
exercises:

- Empty input
- Invalid UTF-8 sequences
- Extreme-length fields (above the 10KB scan limit)
- Unicode edge cases: combining marks, surrogates, homoglyphs
- Adversarial regex-bomb shaped inputs

## Fuzz Target Source

```
fuzz/fuzz_targets/fuzz_injection_scanner.rs
```

```rust
fuzz_target!(|data: &[u8]| {
    let scanner = InjectionScanner::global();
    let _ = scanner.scan_bytes("fuzz_field", 0, data);
});
```

## How to Run

**Prerequisites:** nightly Rust toolchain + `cargo-fuzz`

```bash
# Install cargo-fuzz (once)
cargo install cargo-fuzz

# Run from worktree root — campaign runs until Ctrl-C or a panic is found
cargo +nightly fuzz run fuzz_injection_scanner

# Run with a corpus of known injection payloads
cargo +nightly fuzz run fuzz_injection_scanner fuzz/corpus/injection_scanner/

# Run for a fixed duration (e.g., 60 seconds) in CI
cargo +nightly fuzz run fuzz_injection_scanner -- -max_total_time=60

# Check for existing crash artifacts
ls fuzz/artifacts/fuzz_injection_scanner/ 2>/dev/null || echo "No crashes found"
```

## Expected Outcome

No panics. All byte sequences produce a `ScanResult` with either empty flags or one or
more `SafetyFlag` entries. The `TruncatedScan` flag is emitted for inputs exceeding 10KB.

## Status

| Status | Details |
|--------|---------|
| Target implemented | `fuzz/fuzz_targets/fuzz_injection_scanner.rs` — compiled, not yet run for campaign |
| Scheduled run | Phase 5 adversarial hardening campaign |
| CI integration | Pending Phase 5 — `cargo +nightly fuzz run fuzz_injection_scanner -- -max_total_time=60` |

This document is the demo evidence for VP-038. Full recording is deferred to Phase 5
per story S-1.10 spec: "VP-038 fuzz target — document only, runs in Phase 5."
