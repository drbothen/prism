---
document_type: verification-property
level: L4
version: "0.1"
status: draft
producer: architect
timestamp: 2026-05-02T20:30:00Z
phase: 4-W4-Phase2-ADR
inputs:
  - .factory/specs/architecture/decisions/ADR-015-detection-rule-language.md
  - .factory/STATE.md
input-hash: "1360731"
traces_to: .factory/specs/architecture/decisions/ADR-015-detection-rule-language.md
source_bc: null
source_adr: ADR-015
module: prism-operations
priority: P1
proof_method: proptest
verification_method: proptest
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: wave-4-phase-2-adr
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-139: IOC Matching Layered Correctness

> **NOTE:** This VP traces to an ADR, not a BC. `source_bc: null` and
> `source_adr: ADR-015` are used because the primary constraint originates
> in the architecture decision record. A BC anchor will be added when
> ADR-015 transitions to Accepted and produces a concrete BC in Phase 4.B.

> **[STUB — full VP authoring deferred to Phase 4.B BC authoring]**

## Property Statement

The IOC pattern matching architecture described in ADR-015 §4 mandates a
layered split: fast path via aho-corasick multi-pattern string search, slow
path via RegexSet for complex patterns. This VP verifies that the split
implementation produces results equivalent to a monolithic baseline (single
RegexSet over all patterns) for all valid IOC inputs. A proptest round-trip
across randomized IOC corpora and rule sets must demonstrate that aho-corasick
plus RegexSet combined matches equal monolithic RegexSet matches, with no false
positives or false negatives introduced by the routing logic.

## Source Contract

> **ADR-sourced stub — BC not yet assigned.**

- **ADR:** ADR-015 — Detection Rule Language, §4 IOC Pattern Matching Architecture
- **Decision Reference:** aho-corasick + RegexSet split equivalence with monolithic baseline
- **Postcondition/Invariant:** For all valid IOC inputs, split-path match results == monolithic-path match results.
- **BC:** To be assigned when ADR-015 is Accepted and BC authoring completes in Wave 4 Phase 4.B.
- **Module:** prism-operations
- **Category:** Correctness / Detection

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest 1.x | Yes — randomized IOC corpora + rule sets | aho-corasick + RegexSet split vs monolithic equivalence |

**Feasibility:** IOC inputs are bounded strings; proptest can generate randomized
pattern sets and verify that the split routing produces identical match sets to
the monolithic baseline.

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_operations::detection::ioc_matcher
//
// proptest! {
//     #[test]
//     fn ioc_matching_split_equals_monolithic(
//         patterns in arb_ioc_pattern_set(),
//         input in arb_ioc_input(),
//     ) {
//         let split_result = split_matcher(&patterns, &input);
//         let monolithic_result = monolithic_matcher(&patterns, &input);
//         prop_assert_eq!(split_result, monolithic_result);
//     }
// }
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Bounded | IOC patterns are bounded strings; proptest can enumerate representative corpora |
| Proof complexity | Low | Single equivalence assertion: split-path == monolithic-path per input |
| Tool support | Full | proptest 1.x handles randomized string generation |
| Estimated proof time | <60 seconds | Small state space; regex compilation dominates, not execution |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-05-02 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 0.1 | W4-Phase2-ADR-commit | 2026-05-02 | architect | Initial stub. Traces to ADR-015 §4. source_bc null pending ADR acceptance + BC authoring. Full harness deferred to Wave 4 Phase 4.B. |
