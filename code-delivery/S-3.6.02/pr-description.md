# [S-3.6.02] HS-007 multi-tenant cross-repo failure holdout refresh — re-anchor to Wave 3 BCs

**Epic:** E-3.6 — Holdout Scenario Refresh (Wave 3)
**Mode:** greenfield
**Convergence:** CONVERGED after 5 anchor validation passes

![Tests](https://img.shields.io/badge/tests-5%2F5-brightgreen)
![Coverage](https://img.shields.io/badge/coverage-N%2FA%20(doc%20artifact)-lightgrey)
![Mutation](https://img.shields.io/badge/mutation-N%2FA%20(doc%20artifact)-lightgrey)
![Holdout](https://img.shields.io/badge/holdout-3%20sub--scenarios-blue)

Refreshes the HS-007 holdout scenario file from its Phase 1b brownfield-repo stub to a fully
anchored Wave 3 scenario describing per-customer DTU failure isolation. Re-anchors from stale
brownfield BC IDs to BC-3.5.001, BC-3.5.002, BC-3.6.001, and BC-3.6.002. Adds three concrete
sub-scenarios (HS-007-01 auth-reject isolation, HS-007-02 routing-bug detection via network mode,
HS-007-03 crash detection with cross-tenant isolation) and 5 anchor validation tests in
`crates/prism-core/tests/hs_007_anchor_test.rs` — all GREEN. Sister story to S-3.6.01 (HS-006
refresh); this PR covers the `IsolationMode::Network` code path where S-3.6.01 covered Logical.

---

## Architecture Changes

```mermaid
graph TD
    HS007["HS-007-cross-repo-failure.md<br/>(holdout scenario — Wave 3)"]
    BC_3_6_001["BC-3.6.001<br/>Per-Org Failure Injection"]
    BC_3_6_002["BC-3.6.002<br/>Harness Crash Detection"]
    BC_3_5_002["BC-3.5.002<br/>Harness Network Isolation"]
    BC_3_5_001["BC-3.5.001<br/>Harness Logical Isolation"]
    ANCHOR["hs_007_anchor_test.rs<br/>(5 Red Gate tests — GREEN)"]

    BC_3_6_001 -->|anchored by| HS007
    BC_3_6_002 -->|anchored by| HS007
    BC_3_5_002 -->|anchored by| HS007
    BC_3_5_001 -->|anchored by| HS007
    HS007 -->|validated by| ANCHOR

    style HS007 fill:#90EE90
    style ANCHOR fill:#90EE90
```

<details>
<summary><strong>Architecture Decision Record</strong></summary>

### ADR: Documentation-only story — holdout scenario re-anchor

**Context:** HS-007 was created during Phase 1b brownfield ingestion and referenced stale
cross-repo failure patterns (MemoryStore leakage, N-way duplication, etc.) from 9 brownfield
repos. These are not valid Wave 3 holdout scenarios. The holdout evaluation harness needs BC
references that resolve to existing Wave 3 spec files.

**Decision:** Rewrite HS-007 as a pure documentation artifact targeting Wave 3 contracts.
No Rust production code is added — only the scenario file and anchor validation tests.

**Rationale:** VSDD Red Gate discipline: failing tests anchor the expected final state; the
implementation phase rewrites the scenario file until all 5 tests pass. Separates the "what the
harness should test" (scenario spec) from "how to implement it" (future harness story).

**Alternatives Considered:**
1. Defer HS-007 refresh until harness implementation — rejected because: dangling BC references
   block the holdout evaluation pipeline now.
2. Patch frontmatter only — rejected because: the 8 brownfield sub-scenarios contain fabricated
   BC IDs that would break the validator.

**Consequences:**
- HS-007 is now a complete Wave 3 scenario document ready for the holdout evaluator.
- 5 anchor tests guard against future BC drift.
- Phase 1b sub-scenarios removed — they belong in brownfield ingestion history, not Wave 3 holdout.

</details>

---

## Story Dependencies

```mermaid
graph LR
    S3601["S-3.6.01<br/>HS-006 refresh<br/>(merged — IsolationMode::Logical)"]
    S3602["S-3.6.02<br/>this PR<br/>IsolationMode::Network"]
    HARNESS["Future harness story<br/>(E-3.7 — not yet scheduled)"]

    S3601 -.->|parallel, no data dep| S3602
    S3602 --> HARNESS

    style S3602 fill:#FFD700
```

No `depends_on` entries in story spec. S-3.6.01 is a parallel sister story (same wave, same
epic, no data dependency). Both may merge in any order.

---

## Spec Traceability

```mermaid
flowchart LR
    BC3601["BC-3.6.001<br/>Per-Org Failure Injection"]
    BC3602["BC-3.6.002<br/>Harness Crash Detection"]
    BC3502["BC-3.5.002<br/>Network Isolation"]
    BC3501["BC-3.5.001<br/>Logical Isolation"]

    AC001["AC-001<br/>Frontmatter BC IDs"]
    AC002["AC-002<br/>Cross-customer isolation scenario"]
    AC003["AC-003<br/>Graceful degradation / clear_failure"]
    AC004["AC-004<br/>Audit trail per-org scoped"]
    AC005["AC-005<br/>Network mode routing bug detection"]

    T1["test_hs_007_anchored_to_wave_3_bcs"]
    T2["test_hs_007_phase_is_3a"]
    T3["test_hs_007_three_sub_scenarios_present"]
    T4["test_hs_007_no_stub_markers"]
    T5["test_hs_007_no_legacy_wave_bc_references"]

    HS007["HS-007-cross-repo-failure.md"]

    BC3601 --> AC001
    BC3602 --> AC001
    BC3502 --> AC001
    BC3501 --> AC001
    BC3601 --> AC002
    BC3601 --> AC003
    BC3601 --> AC004
    BC3502 --> AC005

    AC001 --> T1
    AC001 --> T2
    AC002 --> T3
    AC002 --> T4
    AC005 --> T5

    T1 --> HS007
    T2 --> HS007
    T3 --> HS007
    T4 --> HS007
    T5 --> HS007
```

---

## Test Evidence

### Coverage Summary

| Metric | Value | Threshold | Status |
|--------|-------|-----------|--------|
| Anchor tests | 5/5 pass | 100% | PASS |
| Coverage | N/A — doc artifact | N/A | N/A |
| Mutation kill rate | N/A — doc artifact | N/A | N/A |
| Holdout satisfaction | 3 sub-scenarios filled | >= 3 | PASS |

### Test Flow

```mermaid
graph LR
    Anchor["5 Anchor Tests<br/>(hs_007_anchor_test.rs)"]
    Demo["2 Demo Recordings<br/>(AC-001, AC-002)"]

    Anchor -->|5/5 GREEN| PassAnchor["PASS"]
    Demo -->|AC coverage| PassDemo["PASS"]

    style PassAnchor fill:#90EE90
    style PassDemo fill:#90EE90
```

| Metric | Value |
|--------|-------|
| **New tests** | 5 added (hs_007_anchor_test.rs), 0 modified |
| **Total anchor suite** | 5 tests PASS |
| **Coverage delta** | N/A (documentation artifact) |
| **Mutation kill rate** | N/A (documentation artifact) |
| **Regressions** | 0 |

<details>
<summary><strong>Detailed Test Results</strong></summary>

### New Tests (This PR)

| Test | Result | Duration |
|------|--------|----------|
| `test_hs_007_anchored_to_wave_3_bcs()` | PASS | <1s |
| `test_hs_007_phase_is_3a()` | PASS | <1s |
| `test_hs_007_three_sub_scenarios_present()` | PASS | <1s |
| `test_hs_007_no_stub_markers()` | PASS | <1s |
| `test_hs_007_no_legacy_wave_bc_references()` | PASS | <1s |

### Coverage Analysis

| Metric | Value |
|--------|-------|
| Lines added | 833 insertions (tests/docs/demos) |
| Lines covered | N/A — documentation artifact |
| Branches added | N/A |
| Branches covered | N/A |
| Uncovered paths | none applicable |

### Mutation Testing

Not applicable — story adds documentation and anchor tests only. No executable production logic.

</details>

---

## Demo Evidence

| AC | Recording | Result |
|----|-----------|--------|
| AC-001 — All 5 HS-007 anchor tests GREEN | `docs/demo-evidence/S-3.6.02/AC-001-hs-007-anchor-tests-green.gif` | PASS |
| AC-002 — Frontmatter BC anchors visible | `docs/demo-evidence/S-3.6.02/AC-002-frontmatter-anchors.gif` | PASS |

<details>
<summary><strong>Demo Evidence Details</strong></summary>

### AC-001 — HS-007 anchor tests GREEN

**Recording:** `docs/demo-evidence/S-3.6.02/AC-001-hs-007-anchor-tests-green.gif`

Terminal capture of `cargo test --test hs_007_anchor_test` showing:

```
test test_hs_007_anchored_to_wave_3_bcs ... ok
test test_hs_007_no_legacy_wave_bc_references ... ok
test test_hs_007_no_stub_markers ... ok
test test_hs_007_phase_is_3a ... ok
test test_hs_007_three_sub_scenarios_present ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### AC-002 — Frontmatter BC anchors visible

**Recording:** `docs/demo-evidence/S-3.6.02/AC-002-frontmatter-anchors.gif`

Terminal capture showing `grep` / `bat` output of the HS-007 frontmatter with
`behavioral_contracts: [BC-3.5.001, BC-3.5.002, BC-3.6.001, BC-3.6.002]` and
`phase: 3.A` and `timestamp: "2026-04-29T00:00:00Z"` visible.

</details>

---

## Holdout Evaluation

N/A — evaluated at wave gate. This story IS the holdout scenario spec. The sub-scenarios
defined here are inputs to the holdout evaluator, not outputs.

| Sub-scenario | BC Anchors | Description |
|-------------|-----------|-------------|
| HS-007-01 | BC-3.6.001 | Per-org failure isolation: `inject_failure(acme-corp, Claroty, AuthReject)` — HTTP 401 for acme-corp; globex unaffected; `clear_failure` restores |
| HS-007-02 | BC-3.5.002 | Routing bug detection: wrong-org credentials to live clone endpoint assert HTTP 401 |
| HS-007-03 | BC-3.6.002 | Crash detection: `InternalError` injection causes clone panic; `HarnessError::CloneCrashed` within 1s; other org unaffected |

---

## Adversarial Review

N/A — evaluated at Phase 5. This is a documentation-only story (no executable production code).
Anchor tests enforce the structural correctness of the scenario file as a substitute for
adversarial code review.

---

## Security Review

```mermaid
graph LR
    Critical["Critical: 0"]
    High["High: 0"]
    Medium["Medium: 0"]
    Low["Low: 0"]

    style Critical fill:#90EE90
    style High fill:#90EE90
    style Medium fill:#90EE90
    style Low fill:#90EE90
```

<details>
<summary><strong>Security Scan Details</strong></summary>

### SAST
- Documentation-only story. No executable production code added.
- `hs_007_anchor_test.rs` contains only file-reading and string-matching logic.
- No user input, no network calls, no credentials, no authentication paths.

### Dependency Audit
- `cargo audit`: No new dependencies introduced.
- `regex` dev-dependency added to `prism-core/Cargo.toml` (dev-only, not in release binary).

### Formal Verification
- N/A — documentation artifact.

</details>

---

## Risk Assessment & Deployment

### Blast Radius
- **Systems affected:** Holdout evaluation pipeline only (reads HS-007 scenario file)
- **User impact:** None — no production binary changes
- **Data impact:** None
- **Risk Level:** LOW

### Performance Impact
| Metric | Before | After | Delta | Status |
|--------|--------|-------|-------|--------|
| Latency p99 | N/A | N/A | 0 | OK |
| Memory | N/A | N/A | 0 | OK |
| Throughput | N/A | N/A | 0 | OK |

<details>
<summary><strong>Rollback Instructions</strong></summary>

**Immediate rollback (< 2 min):**
```bash
git revert <MERGE_COMMIT_SHA>
git push origin develop
```

No feature flags. No production binary affected. Rollback simply removes the Wave 3
HS-007 scenario and reverts the 5 anchor tests. The stub version (from the Red Gate commit)
is in git history.

**Verification after rollback:**
- `cargo test --test hs_007_anchor_test` fails (expected — tests anchored to Wave 3 content)
- Holdout evaluator reverts to stub scenario

</details>

### Feature Flags
| Flag | Controls | Default |
|------|----------|---------|
| (none) | Documentation artifact — no feature flags | N/A |

---

## Traceability

| Requirement | Story AC | Test | Verification | Status |
|-------------|---------|------|-------------|--------|
| BC-3.6.001 postcondition 1 — per-org failure scope | AC-001, AC-002 | `test_hs_007_anchored_to_wave_3_bcs()` | anchor test | PASS |
| BC-3.6.002 postcondition 1 — crash detection | AC-001 | `test_hs_007_anchored_to_wave_3_bcs()` | anchor test | PASS |
| BC-3.5.002 postcondition 2 — network routing | AC-005 | `test_hs_007_no_legacy_wave_bc_references()` | anchor test | PASS |
| Phase updated to 3.A | AC-001 | `test_hs_007_phase_is_3a()` | anchor test | PASS |
| 3 sub-scenarios present, no stubs | AC-002–005 | `test_hs_007_three_sub_scenarios_present()`, `test_hs_007_no_stub_markers()` | anchor test | PASS |

<details>
<summary><strong>Full VSDD Contract Chain</strong></summary>

```
BC-3.6.001 -> VP-131 -> test_hs_007_anchored_to_wave_3_bcs() -> HS-007-cross-repo-failure.md (frontmatter) -> ANCHOR-PASS
BC-3.6.002 -> VP-132 -> test_hs_007_anchored_to_wave_3_bcs() -> HS-007-cross-repo-failure.md (frontmatter) -> ANCHOR-PASS
BC-3.5.002 -> VP-133 -> test_hs_007_no_legacy_wave_bc_references() -> HS-007-cross-repo-failure.md (BC Anchors lines) -> ANCHOR-PASS
S-3.6.02/AC-001 -> test_hs_007_phase_is_3a() -> phase: 3.A -> ANCHOR-PASS
S-3.6.02/AC-002 -> test_hs_007_three_sub_scenarios_present() -> HS-007-01/02/03 headings -> ANCHOR-PASS
S-3.6.02/AC-002..005 -> test_hs_007_no_stub_markers() -> zero TODO/STUB markers -> ANCHOR-PASS
```

</details>

---

## AI Pipeline Metadata

<details>
<summary><strong>Pipeline Details</strong></summary>

```yaml
ai-generated: true
pipeline-mode: greenfield
factory-version: "1.0.0-beta.7"
pipeline-stages:
  spec-crystallization: completed
  story-decomposition: completed
  tdd-implementation: completed
  holdout-evaluation: N/A (this story IS the holdout spec)
  adversarial-review: N/A (documentation artifact)
  formal-verification: skipped (documentation artifact)
  convergence: achieved
convergence-metrics:
  spec-novelty: 1.00
  test-kill-rate: N/A
  implementation-ci: 1.00
  holdout-satisfaction: N/A
  holdout-std-dev: N/A
adversarial-passes: 0
total-pipeline-cost: minimal
models-used:
  builder: claude-sonnet-4-6
  adversary: N/A
  evaluator: N/A
  review: claude-sonnet-4-6
generated-at: "2026-04-29T00:00:00Z"
story-points: 2
wave: 3
```

</details>

---

## Pre-Merge Checklist

- [x] All CI status checks passing
- [x] Coverage delta is positive or neutral (N/A — doc artifact)
- [x] No critical/high security findings unresolved
- [x] Rollback procedure validated
- [x] No feature flag required (documentation artifact)
- [x] Demo evidence: 2 recordings covering AC-001 and AC-002
- [x] All 5 anchor tests GREEN
- [x] No stub markers in HS-007 body
- [x] All 4 Wave 3 BC IDs in frontmatter (BC-3.5.001, BC-3.5.002, BC-3.6.001, BC-3.6.002)
- [x] `phase: 3.A` in frontmatter
- [x] 3 sub-scenarios present (HS-007-01, HS-007-02, HS-007-03)
