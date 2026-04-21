---
document_type: verification-property
level: L4
version: "1.3"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-1.10-prompt-injection-defense.md]
input-hash: "5dc7ee8"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.09.003
module: prism-security
priority: P0
proof_method: proptest
verification_method: proptest
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

# VP-024: Injection Scanner — Detects Known Injection Patterns

## Property Statement

For every string `s` that contains any known prompt-injection signature from the
canonical pattern catalogue (e.g. "ignore previous instructions", tool-invocation
markup, role-hijack phrasing), `InjectionScanner::scan(s)` produces a result whose
`suspicious_flags` field is non-empty and identifies the matching category. Patterns
embedded in noise, mixed case, or Unicode variants are still detected.

## Source Contract

- **Anchor Story:** `S-1.10`
- **Source BC:** BC-2.09.003 — Suspicious Pattern Detection via Regex with NFKC Normalization
- **Module:** prism-security
- **Category:** Security

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest (latest) | No — random wrappers around pattern catalogue | Every catalogue entry + fuzz wrappers |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_security::injection::InjectionScanner::scan
//
// Sketch: for each pattern in catalogue, embed in randomly generated
// prefix/suffix noise; assert scanner flags the pattern category.
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | No | Random noise, size-bounded |
| Tool support? | Full | proptest |
| Execution time budget | <60 seconds for 10k cases | Regex scanning is fast |
| Assumptions required | Canonical pattern catalogue versioned | Co-located test fixture |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.3 | pass-88-remediation | 2026-04-21 | architect | F88-012: Anchor Story normalized from slug form (S-1.10-prompt-injection-defense.md) to pure ID (S-1.10). |
| 1.2 | pass-87-remediation | 2026-04-21 | architect | F87-006: Source BC label corrected "Suspicious Pattern Detection via Regex" → "Suspicious Pattern Detection via Regex with NFKC Normalization" (matches BC-2.09.003 H1). |
| 1.1 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
