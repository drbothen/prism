---
document_type: verification-property
level: L4
version: "1.1"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-4.05-alert-generation.md]
input-hash: "5ddf1df"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.13.005
module: prism-operations
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

# VP-028: Template Interpolation Never Panics

## Property Statement

For every alert template string `t` and every event snapshot `e`, the call
`interpolate(t, e)` returns `Ok(String)` or `Err(TemplateError)` without panicking.
Missing variables, malformed placeholders, cycles, and adversarial inputs must all
be handled as structured errors — never panics, stack overflow, or infinite loop.

## Source Contract

- **Anchor Story:** `S-4.05-alert-generation.md`
- **Source BC:** BC-2.13.005 — Alert Generation with Template Interpolation
- **Module:** prism-operations
- **Category:** Safety

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| fuzz | cargo-fuzz (libFuzzer) | No — coverage-guided mutation | Continuous corpus expansion |

## Proof Harness Skeleton

```rust
// [TODO: fuzz target skeleton — author during Phase 5 formal-verify]
// prism-operations/fuzz/fuzz_targets/fuzz_template_interpolate.rs
//
// fuzz_target!(|data: &[u8]| {
//     if let Ok(s) = std::str::from_utf8(data) {
//         let (tmpl, event) = split_input(s);
//         let _ = interpolate(tmpl, &parse_event(event));
//     }
// });
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | No | Coverage-guided |
| Tool support? | Full | libFuzzer + ASan |
| Execution time budget | 30 min initial, continuous in CI | Template engines fuzz well |
| Assumptions required | None | Panic = failure |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.1 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
