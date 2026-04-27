# W2-FIX-J: Gate MockStorageEngine behind cfg(test, test-utils)

**Epic:** Wave 2 Gate Holdout Remediation
**Mode:** maintenance
**Convergence:** N/A — hygiene fix, no spec convergence cycle required

![Tests](https://img.shields.io/badge/tests-1498%2F1498-brightgreen)
![Coverage](https://img.shields.io/badge/coverage-maintained-brightgreen)
![Mutation](https://img.shields.io/badge/mutation-N%2FA-lightgrey)
![Holdout](https://img.shields.io/badge/holdout-N%2FA-lightgrey)

Wave 2 gate-step-f holdout remediation for gap #2. `prism-storage` exported `pub mod mock;` and `pub use mock::MockStorageEngine;` unconditionally, exposing the test-only type in the production API surface and causing production binaries to link the mock implementation (HS-007-01 anti-pattern). Both declarations are now gated behind `#[cfg(any(test, feature = "test-utils"))]`, matching the existing `InMemoryBackend` gate pattern in the same file. Zero production callers; no API breakage.

---

## Architecture Changes

```mermaid
graph TD
    LIBRS["crates/prism-storage/src/lib.rs"]
    MOCK["mod mock (MockStorageEngine)"]
    PROD_BIN["production binary\n(cargo build --release)"]
    TEST_BIN["test binary\n(cargo test)"]
    TESTUTILS["downstream crate\n(feature = test-utils)"]

    LIBRS -->|"BEFORE: unconditional pub mod"| MOCK
    MOCK -.->|"BEFORE: linked into"| PROD_BIN

    LIBRS -->|"AFTER: cfg(any(test, feature=test-utils))"| MOCK
    MOCK -->|"only available in"| TEST_BIN
    MOCK -->|"only available with"| TESTUTILS
    PROD_BIN -.->|"AFTER: no mock linkage"| PROD_BIN

    style MOCK fill:#90EE90
    style PROD_BIN fill:#90EE90
```

<details>
<summary><strong>Change Details</strong></summary>

**File:** `crates/prism-storage/src/lib.rs`

**Before:**
```rust
pub mod mock;
// ...
pub use mock::MockStorageEngine;
```

**After:**
```rust
#[cfg(any(test, feature = "test-utils"))]
pub mod mock;
// ...
#[cfg(any(test, feature = "test-utils"))]
pub use mock::MockStorageEngine;
```

This matches the existing gate on `InMemoryBackend` in the same file. The `test-utils` feature was already present — no new feature added, no dependent crates affected.

</details>

---

## Story Dependencies

```mermaid
graph LR
    FIXI["W2-FIX-I\nbef2b202 merged"] --> FIXJ["W2-FIX-J\nthis PR"]
    FIXJ --> GATE["Wave 2 gate-step-f\ngap #2 closed"]
    style FIXJ fill:#FFD700
    style FIXI fill:#90EE90
    style GATE fill:#87CEEB
```

---

## Spec Traceability

```mermaid
flowchart LR
    HS007["HS-007-01\nProduction binary\nlinks test mock"]
    INV["Investigation\n0 production callers\nacross workspace"]
    GATE["cfg gate applied\nto pub mod mock\n+ pub use"]
    PROOF["cargo build --release\nclean: mock not linked\ncargo test: 1498 pass"]

    HS007 --> INV
    INV --> GATE
    GATE --> PROOF
```

---

## Test Evidence

### Coverage Summary

| Metric | Value | Threshold | Status |
|--------|-------|-----------|--------|
| Unit tests (prism-storage) | 24/24 pass | 100% | PASS |
| Workspace tests | 1498/1498 pass | 100% | PASS |
| Coverage | maintained (no new code) | baseline | PASS |
| Mutation kill rate | N/A (no new logic) | N/A | N/A |
| Holdout satisfaction | N/A — hygiene fix | N/A | N/A |

### Test Flow

```mermaid
graph LR
    UT["24 Unit Tests\n(prism-storage)"]
    WS["1498 Workspace Tests"]
    Build["cargo build --release\n(no test-utils)"]
    Clippy["clippy --all-targets\n-D warnings"]
    Fmt["cargo fmt\n--check"]
    Doc["cargo doc\nMockStorageEngine absent\nfrom default API surface"]

    UT -->|"24/24 pass"| Pass1["PASS"]
    WS -->|"1498/1498 pass"| Pass2["PASS"]
    Build --> Pass3["PASS — mock not linked"]
    Clippy --> Pass4["PASS — clean"]
    Fmt --> Pass5["PASS — clean"]
    Doc --> Pass6["PASS — 0 type exports"]

    style Pass1 fill:#90EE90
    style Pass2 fill:#90EE90
    style Pass3 fill:#90EE90
    style Pass4 fill:#90EE90
    style Pass5 fill:#90EE90
    style Pass6 fill:#90EE90
```

| Metric | Value |
|--------|-------|
| **Lines changed** | +3 / -1 in `lib.rs` (cfg attributes only) |
| **New tests** | 0 added — no new logic, existing 24 prism-storage tests cover mock via cfg(test) |
| **Total suite** | 1498 tests PASS |
| **Regressions** | 0 |

<details>
<summary><strong>Verification Commands</strong></summary>

```
cargo build -p prism-storage --release           → clean (MockStorageEngine absent)
cargo test -p prism-storage                      → 24 passed
cargo test --workspace                           → 1498 passed (no regression)
cargo clippy --workspace --all-targets -- -D warnings → clean
cargo fmt --all --check                          → clean
cargo doc -p prism-storage --no-deps             → MockStorageEngine: 10 hits → 0 type exports
```

</details>

---

## Demo Evidence

N/A — hygiene fix. This PR makes no behavioral or user-visible changes. `MockStorageEngine` is a test-only type; gating it behind `#[cfg(any(test, feature = "test-utils"))]` produces no observable runtime difference. No acceptance criteria require demo recording. Verification is by `cargo build --release` (mock absent from binary) and `cargo doc` (0 type exports in default API surface), both captured in Test Evidence above.

---

## Holdout Evaluation

N/A — evaluated at wave gate. This is a pure hygiene fix (HS-007-01) with no behavioral changes. No holdout scenarios applicable.

---

## Adversarial Review

N/A — evaluated at Phase 5. This fix was triggered by wave gate holdout evaluation (gate-step-f gap #2). The fix is a 2-line cfg attribute addition with 0 production callers confirmed via workspace-wide search.

---

## Security Review

Post-review verdict: CLEAN. Diff reviewed against OWASP Top 10, CWE-489 (Active Debug Code), injection, auth, and input validation categories.

```mermaid
graph LR
    Critical["Critical: 0"]
    High["High: 0 (HS-007-01 RESOLVED)"]
    Medium["Medium: 0"]
    Low["Low: 0"]

    style Critical fill:#90EE90
    style High fill:#90EE90
    style Medium fill:#90EE90
    style Low fill:#90EE90
```

<details>
<summary><strong>Security Scan Details</strong></summary>

### Finding Resolved

**HS-007-01 (Wave 2 holdout gap #2):** Production API surface exposed `MockStorageEngine` unconditionally. Test mock accessible to any crate depending on `prism-storage` without feature flag. Attack surface: consumers could instantiate mock in production paths, bypassing real storage guarantees.

**Resolution:** `#[cfg(any(test, feature = "test-utils"))]` gate on both `pub mod mock;` and `pub use mock::MockStorageEngine;`.

**Post-fix:** `cargo build -p prism-storage --release` produces a binary with no mock linkage. `MockStorageEngine` absent from default API docs.

### Dependency Audit
- No dependency changes in this PR.

</details>

---

## Risk Assessment & Deployment

### Blast Radius
- **Systems affected:** `prism-storage` crate only (1 file, 4 lines)
- **User impact:** None — zero production callers confirmed via workspace-wide search
- **Data impact:** None — no data paths changed
- **Risk Level:** LOW

### Performance Impact
| Metric | Before | After | Delta | Status |
|--------|--------|-------|-------|--------|
| Binary size (release) | baseline | smaller (mock not linked) | marginal reduction | OK |
| Latency p99 | unchanged | unchanged | 0 | OK |
| Memory | unchanged | unchanged | 0 | OK |

<details>
<summary><strong>Rollback Instructions</strong></summary>

**Immediate rollback (< 2 min):**
```bash
git revert 74979193
git push origin develop
```

**Verification after rollback:**
- `cargo build -p prism-storage --release` — clean
- `cargo test --workspace` — 1498 passed

</details>

### Feature Flags
| Flag | Controls | Default |
|------|----------|---------|
| `test-utils` | Expose `MockStorageEngine` and `InMemoryBackend` to downstream test crates | off (production) |

---

## Traceability

| Requirement | Finding | Fix | Verification | Status |
|-------------|---------|-----|-------------|--------|
| HS-007-01: no mock in prod API | Wave 2 gate-step-f gap #2 | cfg gate on `pub mod mock` + `pub use MockStorageEngine` | `cargo build --release` clean; `cargo doc` 0 type exports | PASS |

<details>
<summary><strong>Investigation Record</strong></summary>

```
HS-007-01 (gate-step-f gap #2)
  → workspace grep for MockStorageEngine callers
  → 0 production callers found
  → 1 caller: crates/prism-storage/src/proofs/storage_batch.rs (inside #[cfg(test)])
  → test-utils feature already gating InMemoryBackend in same file
  → applied same cfg(any(test, feature = "test-utils")) gate
  → cargo build --release: clean
  → cargo test --workspace: 1498 passed (baseline maintained)
```

</details>

---

## AI Pipeline Metadata

<details>
<summary><strong>Pipeline Details</strong></summary>

```yaml
ai-generated: true
pipeline-mode: maintenance
factory-version: "1.0.0"
pipeline-stages:
  spec-crystallization: skipped (hygiene fix)
  story-decomposition: skipped (single-file, 4-line change)
  tdd-implementation: completed (cfg gate + verification)
  holdout-evaluation: "N/A — evaluated at wave gate"
  adversarial-review: "N/A — evaluated at Phase 5"
  formal-verification: skipped (no new logic)
  convergence: achieved
convergence-metrics:
  spec-novelty: "N/A"
  test-kill-rate: "N/A (no new logic)"
  implementation-ci: 1.0
  holdout-satisfaction: "N/A"
adversarial-passes: 0
models-used:
  builder: claude-sonnet-4-6
generated-at: "2026-04-26T00:00:00Z"
```

</details>

---

## Pre-Merge Checklist

- [ ] All CI status checks passing
- [x] Coverage delta: neutral (no new code, cfg attribute only)
- [x] No critical/high security findings unresolved (HS-007-01 resolved by this PR)
- [x] Rollback procedure validated (`git revert 74979193`)
- [x] No feature flag required (test-utils already existed)
- [ ] Human review completed (if autonomy level requires)
- [x] No production-impacting changes (mock/test infrastructure only)
