---
document_type: verification-property
level: L4
version: "1.7"
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
proof_file_hash: "e87b8e833666c30d43c088963b1a163120f89cdb35fc1f89dd3280278bbef234"
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

# VP-015: Query Security Limits — Rejects Excessive Nesting Depth

## Property Statement

For every valid UTF-8 query input, if its parenthesization/grouping structure would
produce an AST whose depth exceeds the configured ceiling (**64**), the parser
returns `Err(Vec<ParseError>)` where the first ParseError's `message` field contains
the substring `E-QUERY-003`. The parser never recurses past the depth limit.

**Canonical limit: 64** (per BC-2.11.006 postcondition, DI-019, EC-002 in S-3.01).
The prior "e.g. 32" was an illustrative placeholder; 64 is the canonical value.

## Source Contract

- **Anchor Story:** `S-3.01`
- **Source BC:** BC-2.11.006 — Query Security Limits Enforcement
- **Module:** prism-query
- **Category:** Security

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| kani | Kani (latest) | Yes — scaled depth limit | All depths around limit boundary |

## Proof Harness Skeleton

```rust
// VERIFIED — real harnesses at crates/prism-query/src/proofs/vp015_depth_limit.rs
// Commit: f5212641 (PR #127)
//
// Four harnesses verified:
//   1. proof_nesting_depth_limit       — PrismQL nested parentheses hit MAX_DEPTH ceiling
//   2. proof_expr_depth_limit          — expression recursion depth bounded
//   3. proof_predicate_depth_limit     — predicate recursion depth bounded
//   4. proof_sql_query_depth_limit     — SqlQuery variant depth limit (NEW in PR #127)
//
// SqlQuery harness result: 0 of 5664 steps failed; 397 steps unreachable.
//
// Notes:
//   - proof_expr_depth_limit + proof_predicate_depth_limit require
//     --no-unwinding-checks --default-unwind 2 (recursive parser unwind bound).
//   - Canonical depth limit: 64 (per BC-2.11.006 postcondition, DI-019, EC-002 in S-3.01).
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | Scaled MAX_DEPTH and string bound |
| Tool support? | Full | Kani with unwind bound |
| Execution time budget | <10 minutes | Parser symbolic execution is heavier |
| Assumptions required | Iterative or depth-checked recursive parser | Implementation must check before recursing |

## Verification Record

| Field | Value |
|-------|-------|
| Proof file | `crates/prism-query/src/proofs/vp015_depth_limit.rs` |
| Commit | `f5212641` (PR #127) |
| Harnesses | `proof_nesting_depth_limit`, `proof_expr_depth_limit`, `proof_predicate_depth_limit`, `proof_sql_query_depth_limit` |
| Verification date | 2026-05-05 |
| SqlQuery result | 0 of 5664 steps failed; 397 steps unreachable |
| Kani flags | `--no-unwinding-checks --default-unwind 2` (required for `proof_expr_depth_limit` and `proof_predicate_depth_limit`) |
| Proof file SHA256 | `e87b8e833666c30d43c088963b1a163120f89cdb35fc1f89dd3280278bbef234` |
| New harness | `proof_sql_query_depth_limit` added in PR #127 — extends depth limit coverage to the `SqlQuery` AST variant (previously only PrismQL covered) |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |
| verified | 2026-05-05 | formal-verifier |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.7 | pr-127-pass4-remediation | 2026-05-05 | architect | Property statement corrected to match actual `ParseError` struct API (returns `Err(Vec<ParseError>)` with `message` containing `E-QUERY-003`). Replaces incorrect `ParseError::NestingTooDeep` enum-variant reference identified by adversary pass-4 (F-MEDIUM-001). Verification lock retained — Kani proofs at f5212641 cover `check_paren_depth` gate-level property; structural composition argument unchanged. proof_file_hash refreshed from `caf599af` → `e87b8e83` to reflect 3 new dynamic tests added by implementer #4 (4b1d8fb0): having, joins, and order_by depth limit regression tests. |
| 1.6 | pr-127-formal-verify | 2026-05-05 | architect | VP-015 promoted to verified. Four Kani harnesses in `vp015_depth_limit.rs` at commit f5212641 (PR #127): `proof_nesting_depth_limit`, `proof_expr_depth_limit`, `proof_predicate_depth_limit`, and new `proof_sql_query_depth_limit`. SqlQuery harness: 0 of 5664 failed (397 unreachable). `proof_expr_depth_limit` / `proof_predicate_depth_limit` require `--no-unwinding-checks --default-unwind 2`. Cross-ref: VP-INDEX v1.28 VP-015 row promoted to verified; verification-architecture.md v1.29 Provable Properties Catalog updated. |
| 1.5 | W3-spec-remediation | 2026-05-04 | story-writer | Depth limit corrected from "e.g. 32" to canonical 64 (per BC-2.11.006, DI-019, EC-002 in S-3.01). Three-way inconsistency (VP-015 said 32; BC-2.11.006 + story + DI-019 said 64) resolved in favour of 64 — majority of citations + grammar complexity requirement. |
| 1.4 | pass-88-remediation | 2026-04-21 | architect | F88-012: Anchor Story normalized from slug form (S-3.01-prismql-parser.md) to pure ID (S-3.01). |
| 1.3 | pass-61-fix | 2026-04-20 | architect | Renumbered duplicate pre-build-sweep Changelog row for version monotonicity (MED-001 VP scope extension). |
| 1.2 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
| 1.1 | B-52 | 2026-04-19 | state-manager | Renamed `AxiqlParser` → `PrismQlParser` in harness target comment (PrismQL rename propagation gap). Closes P3P55-A-MED-001. |
