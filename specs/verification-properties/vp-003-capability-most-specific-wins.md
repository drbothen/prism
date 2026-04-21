---
document_type: verification-property
level: L4
version: "1.1"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-1.03-capability-resolution.md]
input-hash: "49b3c02"
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

# VP-003: Capability Resolution — Most-Specific-Path Wins

## Property Statement

For every capability path `p` and every capability map `m` that contains both a
general rule (shorter prefix) and a more specific rule (longer prefix that matches
`p`), `evaluate_capability(p, &m)` returns the effect from the most specific matching
rule. When rules exist at multiple specificity levels, the rule whose path has the
greatest number of matching segments determines the resulting `Effect`.

## Source Contract

- **Anchor Story:** `S-1.03-capability-resolution.md`
- **Source BC:** BC-2.04.003 — Hierarchical Capability Resolution (BTreeMap, Most-Specific-Path Wins, Deny Support)
- **Module:** prism-core
- **Category:** Security

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| kani | Kani (latest) | Yes — paths up to 4 segments, maps up to 3 rules | All specificity orderings within bound |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: kani
// Target: prism_core::capabilities::evaluate_capability
//
// Sketch: construct a bounded capability map with a broad rule (e.g. "sensor.*")
// and a narrower rule (e.g. "sensor.crowdstrike.read"); assert that the narrower
// rule's effect dominates the broader rule's effect when both match.
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | Paths bounded to 4 segments, map to 3 rules |
| Tool support? | Full | Kani handles BTreeMap and string prefix comparisons |
| Execution time budget | <5 minutes | String compares dominate symbolic cost |
| Assumptions required | Path segment count bound; ASCII-only segments | Keeps state space tractable |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.1 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
