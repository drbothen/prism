---
document_type: verification-property
level: L4
version: "1.5"
status: verified
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-3.01-prismql-parser.md]
input-hash: "9443ed5"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.11.006
module: prism-query
priority: P0
proof_method: kani
verification_method: kani
feasibility: medium
verification_lock: true
proof_completed_date: 2026-05-05
proof_file_hash: "b6b6184a5ce605d612187294c45e162e4614374a0bdeff729333d4203408a06f"
lifecycle_status: active
introduced: cycle-1
modified: 2026-05-05
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-014: Query Security Limits — Rejects Oversized Queries

## Property Statement

For every query input byte sequence `b`, if `b.len() > 65536` (64 KiB), then
`PrismQlParser::parse(b)` returns `Err(ParseError::QueryTooLarge)` before performing
any lexical or syntactic work. The byte-length check is the first gate in the parse
pipeline.

## Source Contract

- **Anchor Story:** `S-3.01`
- **Source BC:** BC-2.11.006 — Query Security Limits Enforcement
- **Module:** prism-query
- **Category:** Security

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| kani | Kani (latest) | Yes — scaled limit (e.g. 64 bytes) | Boundary above/below scaled limit |

## Proof Harness Skeleton

```rust
// VERIFIED — real harness at crates/prism-query/src/proofs/vp014_size_limit.rs
// Commit: f5212641 (PR #127)
// Harness: proof_check_query_size_rejects_oversize
//
// Three-layer verification strategy:
//   1. Gate-level: check_query_size() returns Err(QueryTooLarge) for len > MAX_QUERY_BYTES.
//   2. Structural composition: PrismQlParser::parse() calls check_query_size() before any
//      lexical or syntactic work; proven by verifying the call ordering at parse entry point.
//   3. Dynamic boundary test: inputs at exactly MAX_QUERY_BYTES-1, MAX_QUERY_BYTES,
//      and MAX_QUERY_BYTES+1 exercise the boundary condition.
//
// Note: requires --no-unwinding-checks due to std::env::var memchr_naive loop.
// Result: 0 of 4371 steps failed; 285 steps unreachable.
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | Scaled MAX_BYTES constant |
| Tool support? | Full | Kani handles length checks |
| Execution time budget | <3 minutes | Small scaled bound |
| Assumptions required | Scaling argument generalizes to 64 KiB | Documented in harness |

## Verification Record

| Field | Value |
|-------|-------|
| Proof file | `crates/prism-query/src/proofs/vp014_size_limit.rs` |
| Commit | `f5212641` (PR #127) |
| Harness | `proof_check_query_size_rejects_oversize` |
| Verification date | 2026-05-05 |
| Result | 0 of 4371 steps failed; 285 steps unreachable |
| Kani flags | `--no-unwinding-checks` (required: `std::env::var` `memchr_naive` loop) |
| Proof file SHA256 | `b6b6184a5ce605d612187294c45e162e4614374a0bdeff729333d4203408a06f` |
| Caveat | Scaling argument: MAX_QUERY_BYTES scaled down for bounded model checking; generalizes to 64 KiB production limit by monotonicity of the gate predicate |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |
| verified | 2026-05-05 | formal-verifier |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.5 | pr-127-formal-verify | 2026-05-05 | architect | VP-014 promoted to verified. Real Kani harness `proof_check_query_size_rejects_oversize` lands in commit f5212641 (PR #127). Replaces prior `assert(true)` stub. Verification successful: 0 of 4371 failed (285 unreachable). Property: gate-level (`check_query_size`) + structural composition (parser calls gate first) + dynamic boundary test. Cross-ref: VP-INDEX v1.28 VP-014 row promoted to verified; verification-architecture.md v1.29 Provable Properties Catalog updated. |
| 1.4 | pass-88-remediation | 2026-04-21 | architect | F88-012: Anchor Story normalized from slug form (S-3.01-prismql-parser.md) to pure ID (S-3.01). |
| 1.3 | pass-61-fix | 2026-04-20 | architect | Renumbered duplicate pre-build-sweep Changelog row for version monotonicity (MED-001 VP scope extension). |
| 1.2 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
| 1.1 | B-52 | 2026-04-19 | state-manager | Renamed `AxiqlParser` → `PrismQlParser` in Property Statement and harness target comments/imports (PrismQL rename propagation gap). Closes P3P55-A-MED-001. |
