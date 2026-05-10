---
document_type: verification-property
level: L4
version: "1.2"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-1.03-capability-resolution.md]
input-hash: "b5020fb"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.04.003
module: prism-core
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

# VP-004: Capability Resolution — Deny Overrides Allow at Same Specificity

## Property Statement

For every capability path `p` and every capability map `m` that contains two rules
whose paths match `p` at identical specificity levels (same segment count, same
prefix length), if one rule's effect is `Effect::Deny` and the other is
`Effect::Allow`, `evaluate_capability(p, &m)` returns `Effect::Deny`. Deny is the
tiebreaker at any specificity level.

## Source Contract

- **Anchor Story:** `S-1.03`
- **Source BC:** BC-2.04.003 — Hierarchical Capability Resolution (BTreeMap, Most-Specific-Path Wins, Deny Support)
- **Module:** prism-core
- **Category:** Security

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| kani | Kani (latest) | Yes — paths up to 4 segments, 2 conflicting rules | All Deny/Allow same-specificity combinations |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: kani
// Target: prism_core::capabilities::evaluate_capability
//
// Sketch: build a map containing two equally specific rules whose effects are
// Allow and Deny; assert result equals Effect::Deny for any path matching both.
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | Fixed two-rule map, paths bounded to 4 segments |
| Tool support? | Full | Kani handles enum + BTreeMap lookups |
| Execution time budget | <3 minutes | Very small state space |
| Assumptions required | Both rules match path; same specificity | Enforced via kani::assume |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.2 | pass-88-remediation | 2026-04-21 | architect | F88-012: Anchor Story normalized from slug form (S-1.03-capability-resolution.md) to pure ID (S-1.03). |
| 1.1 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
