---
document_type: verification-property
level: L4
version: "0.1"
status: draft
producer: architect
timestamp: 2026-05-02T20:30:00Z
phase: 4-W4-Phase2-ADR
inputs:
  - .factory/specs/architecture/decisions/ADR-018-differential-result-pack-format.md
  - .factory/STATE.md
input-hash: "6d6fbfb"
traces_to: .factory/specs/architecture/decisions/ADR-018-differential-result-pack-format.md
source_bc: null
source_adr: ADR-018
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

# VP-142: Pack Expansion Idempotence and Collision Semantics

> **NOTE:** This VP traces to an ADR, not a BC. `source_bc: null` and
> `source_adr: ADR-018` are used because the primary constraint originates
> in the architecture decision record. A BC anchor will be added when
> ADR-018 transitions to Accepted and produces a concrete BC in Phase 4.B.

> **[STUB — full VP authoring deferred to Phase 4.B BC authoring]**

## Property Statement

ADR-018 §3 and §6 specify pack expansion idempotence and collision semantics
for differential result packs. This VP verifies that: (1) expanding a pack
twice produces the same result as expanding it once (idempotence), and (2)
when two packs share a key, the collision resolution semantics defined in §6
are applied deterministically — the same pair of colliding packs always
resolves to the same winner. A proptest round-trip across randomized pack
contents and collision scenarios must demonstrate both properties hold.

## Source Contract

> **ADR-sourced stub — BC not yet assigned.**

- **ADR:** ADR-018 — Differential Result Pack Format, §3 Pack Expansion + §6 Collision Semantics
- **Decision Reference:** pack expansion idempotence + collision semantics
- **Postcondition/Invariant:** expand(expand(pack)) == expand(pack); collision resolution is deterministic per (pack_a, pack_b) pair.
- **BC:** To be assigned when ADR-018 is Accepted and BC authoring completes in Wave 4 Phase 4.B.
- **Module:** prism-operations
- **Category:** Correctness / Idempotence

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest 1.x | Yes — randomized pack contents + collision pairs | Expansion idempotence + deterministic collision resolution |

**Feasibility:** Pack contents are bounded maps; proptest can generate
randomized pack pairs and verify both idempotence and collision determinism
exhaustively within a bounded key space.

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_operations::diff::ResultPack
//
// proptest! {
//     #[test]
//     fn pack_expansion_is_idempotent(pack in arb_result_pack()) {
//         let once = pack.expand();
//         let twice = once.clone().expand();
//         prop_assert_eq!(once, twice);
//     }
//
//     #[test]
//     fn pack_collision_resolution_is_deterministic(
//         pack_a in arb_result_pack(),
//         pack_b in arb_result_pack(),
//     ) {
//         let result_ab = resolve_collision(&pack_a, &pack_b);
//         let result_ab2 = resolve_collision(&pack_a, &pack_b);
//         prop_assert_eq!(result_ab, result_ab2);
//     }
// }
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Bounded | Pack key space is finite and small; proptest covers representative combinations |
| Proof complexity | Low | Two independent assertions: idempotence + determinism |
| Tool support | Full | proptest 1.x; pack structs are Clone + PartialEq |
| Estimated proof time | <30 seconds | Small state space; pure data transformation, no I/O |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-05-02 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 0.1 | W4-Phase2-ADR-commit | 2026-05-02 | architect | Initial stub. Traces to ADR-018 §3 + §6. source_bc null pending ADR acceptance + BC authoring. Full harness deferred to Wave 4 Phase 4.B. |
