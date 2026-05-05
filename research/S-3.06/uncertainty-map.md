---
document_type: uncertainty-map
story_id: S-3.06
version: "1.0"
producer: dclaude-uncertainty-scanner
timestamp: "2026-05-04T00:00:00Z"
status: draft
---

# S-3.06 Uncertainty Map — PrismQL Write Parser Extensions

## Summary verdict

**YELLOW** — Story explicitly self-flags Chumsky 0.12 dynamic-dispatch
pattern verification (line 359–361). Plus a cross-crate dependency on
`datafusion-sql` `DmlStatement` AST is mentioned as optional reference
material. Both are realistic concerns; neither is blocking but both warrant
confirmation.

## Findings

| Severity | Category | Finding | Recommended action |
|---|---|---|---|
| Important | api-assumption | Lines 359–361: "Chumsky's dynamic dispatch for write verbs (the verb set is a runtime value, not a static `one_of![]` macro) — verify this pattern against Chumsky 0.12 API." Self-flagged uncertainty. | RESEARCH-NEEDED: confirm Chumsky 0.12 supports a dynamically-built `choice`/`one_of` over `Vec<&str>` of verbs; confirm `Boxed`/`BoxedParser` overhead is acceptable. |
| Important | api-assumption | Line 347: `recover_with` strategies. Chumsky 0.12 reorganized recovery from 0.10's combinators. | RESEARCH-NEEDED: confirm `recover_with(via_parser(...))` and `nested_delimiters` are still the canonical recovery patterns in 0.12. |
| Important | feature-claim | Lines 366–368: claim that `datafusion-sql` exposes a public `DmlStatement` AST that can be reused without coupling Prism's AST to DataFusion. | RESEARCH-NEEDED: confirm `datafusion-sql` 53.x exports `DmlStatement` publicly, or whether it lives inside a private module of `datafusion-sql::parser`. The crate has historically had unstable internal APIs. |
| Suggestion | version-pin | Line 299: `chumsky 0.12.x` matches ARCH-INDEX AD-003 pin. No minor specified. | Pin to specific 0.12.x patch; confirm 0.12.x is still latest stable (Chumsky 0.13 was in alpha mid-2025). |
| Suggestion | architecture-pattern | Cross-story coherence: this story (parser) and S-3.07 (executor) both describe AST nodes. The story shows `WriteNode/DmlNode/WriteArg` enums (line 68). | Verify S-3.07 consumes the same AST without duplicate parser definitions — see W3-batch summary. |

## Cross-references

- AD-003 / ADR (dependency-graph.md) pins Chumsky 0.12.
- BC-2.11.004 (PrismQL Pipe Mode Parsing) active.
- depends_on S-3.01 (parser) and S-1.13/S-6.07 (write endpoint registry / safety).

## RESEARCH-NEEDED queries

1. "Latest stable chumsky crate version as of 2026-05-04. Is 0.12 still current or has 0.13 stabilized? Breaking changes between 0.12 and current."
2. "Chumsky 0.12: how to build a `choice` or `one_of` parser dynamically from a `Vec<&str>` of keywords at runtime. Performance and Boxed overhead."
3. "datafusion-sql 53.x: is `DmlStatement` AST publicly exported? Stability guarantees on the parser AST surface."

