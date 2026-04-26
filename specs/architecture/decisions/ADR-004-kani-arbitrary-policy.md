---
document_type: adr
adr_id: ADR-004
status: proposed
date: 2026-04-26
version: "0.1"
subsystems_affected: [SS-07]
supersedes: null
superseded_by: null
inputs:
  - PR #45 (7903da15) — CI hotfix that added kani::Arbitrary to CaseStatus
  - W2-P2-A-003 finding — adversary flagged product-code change in CI hotfix scope
  - Architect decision 2026-04-26 — KEEP the change; retroactively document policy
traces_to: specs/architecture/verification-architecture.md
---

# ADR-004: Kani Arbitrary Policy — Which Types Carry kani::Arbitrary

## Context

Kani harnesses use `kani::any::<T>()` to generate symbolic inputs for formal proofs. For `kani::any::<T>()` to compile, the type `T` must implement `kani::Arbitrary`. Kani provides a derive macro (`#[derive(kani::Arbitrary)]`) gated behind the `kani` cfg feature, written as `#[cfg_attr(kani, derive(kani::Arbitrary))]` so it is a no-op in non-Kani builds.

PR #45 (`7903da15`) was submitted as a CI hotfix scoped to `.github/workflows/post-merge.yml`. The diff also included a one-line change to `crates/prism-core/src/case.rs:50`:

```rust
#[cfg_attr(kani, derive(kani::Arbitrary))]
pub enum CaseStatus {
```

This addition was required to fix a Kani harness compilation failure: harnesses for VP-005 (`CaseStatus` state machine reachability), VP-006 (case lifecycle invariants), and VP-051 (case-closed postcondition) all call `kani::any::<CaseStatus>()`. Without `kani::Arbitrary` on `CaseStatus`, the Kani build fails.

The Wave 2 gate Pass 2 adversary (finding W2-P2-A-003) flagged this as a CI hotfix scope-creep: a product-code change landed via a CI hotfix PR without a story, Red Gate stub, or story-level review. The Architect reviewed the concern and decided: **KEEP the change** — it is load-bearing for VP-005/006/051 proofs and behaviorally inert outside Kani builds. This ADR retroactively documents the policy so future harness authors add the derive in their own product PRs rather than requiring a CI hotfix.

## Decision

Any enum or struct used as input to a `kani::any::<T>()` call in a harness targeting a VP-NNN property MUST carry `#[cfg_attr(kani, derive(kani::Arbitrary))]` on its definition. This attribute MUST be added in the same product PR that authors the harness, not in a subsequent CI hotfix.

CI hotfix PRs are scope-limited to `.github/workflows/**`, `fuzz/Cargo.toml`, and test-fixture files only. Any product-code change — even a one-line attribute macro — requires a full story/feature PR with the standard Red Gate and review process.

## Rationale

The `kani::Arbitrary` derive is gated behind `#[cfg_attr(kani, ...)]` and adds zero runtime cost, no public API surface change, and no semantic change to the type outside Kani builds. It is as low-risk as a doc comment. The correct process is to co-locate the derive with the harness in one product PR, making the dependency explicit and reviewable as a unit.

The CI hotfix path is dangerous because it bypasses the story/Red Gate review cycle. Even if a given change is trivially safe (as this one is), establishing a pattern of "product-code can sneak into CI hotfixes" creates a precedent that could admit larger unreviewed changes in the future. The cost of one additional product PR per new provable type is negligible; the benefit of enforcing the scope boundary is permanent.

The process gap is recorded as TD-W2-CICD-SCOPE-001. ADR-004 closes the policy gap; TD-W2-CICD-SCOPE-001 tracks the checklist enforcement work.

## Consequences

### Positive

- Future Kani harness authors will not encounter a compile error that appears to require a CI hotfix detour. The derive is part of the same story/PR as the harness.
- CI hotfix PRs remain unambiguously scoped to workflow/fuzz/fixture files; drift is caught by pr-manager review.
- The ADR creates a searchable policy reference for future architects and implementers adding Kani proofs.

### Negative / Trade-offs

- One additional diff hunk per new provable type (the `cfg_attr` line). Negligible in practice.
- Harness authors must be aware of this policy. Enforcement is manual until a lint hook is authored (tracked as potential TD-VSDD-007 if recurrence warrants it).

### Status as of 2026-04-26 (v0.1)

Proposed. The CaseStatus derive (PR #45, `7903da15`) is in effect and KEPT per Architect decision. Policy is not yet enforced via automated hook; manual checklist enforcement at pr-manager review.

## Alternatives Considered

- **Option A — Revert and re-land in product PR:** Rejected. The change is load-bearing for VP-005/006/051; reverting would break three Kani proofs. The process cost of a second PR is not justified when the change is already merged and correct.
- **Option B — Accept CI hotfix scope-creep as policy:** Rejected. This would normalize sneaking product-code changes into CI hotfix PRs, creating an uncontrolled bypass of the Red Gate and story review process.
- **Option C — Add lint to block cfg_attr in hotfix diffs:** Considered for future (TD-VSDD-007 if needed). Not implemented as part of this ADR; enforcement is by convention until recurrence frequency justifies the lint investment.

## Source / Origin

- **Code as-built:** `crates/prism-core/src/case.rs:50` — `#[cfg_attr(kani, derive(kani::Arbitrary))]` on `CaseStatus`; shipped in PR #45 (`7903da15`, 2026-04-25)
- **Adversarial finding:** Wave 2 integration gate Pass 2, finding W2-P2-A-003 (scope-creep in CI hotfix)
- **Architect decision:** 2026-04-26 — KEEP; retroactive ADR stub requested
- **Verification properties:** VP-005 (state machine reachability), VP-006 (case lifecycle), VP-051 (case-closed postcondition) — all depend on `CaseStatus: kani::Arbitrary`
- **Tech debt:** TD-W2-CICD-SCOPE-001 (CI hotfix scope discipline)

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 0.1 | 2026-04-26 | architect (via state-manager burst) | Initial stub — retroactive documentation of PR #45 kani::Arbitrary addition + W2-P2-A-003 architect KEEP decision |
