---
document_type: verification-property
level: L4
version: "1.4"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-3.05-pagination-caching.md]
input-hash: "e94f963"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.07.005
module: prism-query
priority: P1
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

# VP-025: Cache Key Derivation — Deterministic

## Property Statement

For every expanded query `q` and every client scope `c`, `cache_key(q, c)` produces
the same output byte sequence across repeated invocations and across semantically-
equivalent but syntactically-permuted inputs (field order normalized). Equal inputs
yield equal keys; unequal inputs (differing in any material way) yield unequal keys.

## Source Contract

- **Anchor Story:** `S-3.05`
- **Source BC:** BC-2.07.005 — Cache Key Derivation from Push-Down Parameters
- **Module:** prism-query
- **Category:** Correctness

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| kani | Kani (latest) | Yes — bounded query ASTs | Key stability under canonicalization |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: kani
// Target: prism_query::cache::cache_key
//
// Sketch: for symbolic query inputs, assert cache_key(q, c) = cache_key(q, c)
// and that permuted-but-equivalent inputs produce the same key.
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | Bounded AST size |
| Tool support? | Partial | Hash modeled as uninterpreted function |
| Execution time budget | <10 minutes | Hash abstraction required |
| Assumptions required | Hash function modeled as UF; canonicalization is injective | Standard modeling |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.4 | pass-88-remediation | 2026-04-21 | architect | F88-008: inputs field corrected S-3.04-alias-system.md → S-3.05-pagination-caching.md. |
| 1.3 | pass-87-remediation | 2026-04-21 | architect | F87-002: Anchor Story corrected S-3.04 → S-3.05; cache_key property belongs to pagination cache story, not alias story. |
| 1.2 | pass-86-remediation | 2026-04-21 | architect | F86-002: re-anchor source_bc BC-2.11.013 → BC-2.07.005; updated body Source BC label to canonical title. |
| 1.1 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
