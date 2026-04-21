---
document_type: verification-property
level: L4
version: "1.3"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-3.01-prismql-parser.md]
input-hash: "e0d4832"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.11.006
module: prism-query
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

# VP-015: Query Security Limits — Rejects Excessive Nesting Depth

## Property Statement

For every valid UTF-8 query input, if its parenthesization/grouping structure would
produce an AST whose depth exceeds the configured ceiling (e.g. 32), the parser
returns `Err(ParseError::NestingTooDeep)` without constructing the fully nested AST.
The parser never recurses past the depth limit.

## Source Contract

- **Anchor Story:** `S-3.01-prismql-parser.md`
- **Source BC:** BC-2.11.006 — Query Security Limits Enforcement
- **Module:** prism-query
- **Category:** Security

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| kani | Kani (latest) | Yes — scaled depth limit | All depths around limit boundary |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: kani
// Target: prism_query::parser::PrismQlParser::parse
//
// Sketch: construct symbolic depth d up to MAX_DEPTH+2 with scaled MAX_DEPTH;
// build parenthesized input of that depth; assert d > MAX_DEPTH => Err(NestingTooDeep).
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | Scaled MAX_DEPTH and string bound |
| Tool support? | Full | Kani with unwind bound |
| Execution time budget | <10 minutes | Parser symbolic execution is heavier |
| Assumptions required | Iterative or depth-checked recursive parser | Implementation must check before recursing |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.3 | pass-61-fix | 2026-04-20 | architect | Renumbered duplicate pre-build-sweep Changelog row for version monotonicity (MED-001 VP scope extension). |
| 1.2 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
| 1.1 | B-52 | 2026-04-19 | state-manager | Renamed `AxiqlParser` → `PrismQlParser` in harness target comment (PrismQL rename propagation gap). Closes P3P55-A-MED-001. |
