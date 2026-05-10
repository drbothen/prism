---
document_type: verification-property
level: L4
version: "1.2"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-1.08-feature-flags.md]
input-hash: "0434206"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.04.004
module: prism-security
priority: P0
proof_method: kani
verification_method: kani
feasibility: medium
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

# VP-020: Feature Flag — Compile AND Runtime Must Both Permit

## Property Statement

For every capability path `p`, `is_allowed(p, &ctx)` returns `true` if and only if
both gates pass: the compile-time Cargo feature for the enclosing code family is
enabled, AND the runtime capability evaluation for the tenant returns `Allow`.
Either gate alone denying — whether compile-time absent or runtime `Deny` — forces
the combined result to `false`.

## Source Contract

- **Anchor Story:** `S-1.08`
- **Source BC:** BC-2.04.004 — Two-Tier Gate — Both Compile-Time and Runtime Must Permit Operation
- **Module:** prism-security
- **Category:** Security

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| kani | Kani (latest) | Yes — 2x2 truth table | All four gate combinations |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: kani
// Target: prism_security::flags::is_allowed
//
// Sketch: symbolic bools compile_ok, runtime_allow; assert result == (compile_ok && runtime_allow).
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | 4-combination truth table |
| Tool support? | Full | Kani trivially handles bool logic |
| Execution time budget | <30 seconds | Tiny proof |
| Assumptions required | Compile-time gate modeled as runtime bool in test | Separate build-matrix test covers the real cfg gate |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.2 | pass-88-remediation | 2026-04-21 | architect | F88-012: Anchor Story normalized from slug form (S-1.08-feature-flags.md) to pure ID (S-1.08). |
| 1.1 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
