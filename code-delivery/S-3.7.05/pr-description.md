# [S-3.7.05] CrowdStrike fixture generator — 8 archetypes + 2-step pagination + OAuth2

**Epic:** E-3.7 — Data Translation Unit (DTU) Fixture Generators
**Mode:** greenfield
**Convergence:** CONVERGED after TDD implementation (37/37 tests GREEN)

![Tests](https://img.shields.io/badge/tests-37%2F37-brightgreen)
![Coverage](https://img.shields.io/badge/coverage->80%25-brightgreen)
![Mutation](https://img.shields.io/badge/mutation-N%2FA-lightgrey)
![Holdout](https://img.shields.io/badge/holdout-N%2FA--wave--gate-blue)

Implements `generate(org_id, SensorType::CrowdStrike, archetype, opts) -> FixtureSet` behind `#[cfg(feature = "fixture-gen")]` in `crates/prism-dtu-crowdstrike/src/generator.rs`. Covers all 8 archetypes (HealthyOtEnvironment, CompromisedEndpoint, AuthOutage, LargeScale, PaginationEdgeCases, SchemaDrift, HighChurn, DormantTenant), the CrowdStrike-specific 2-step IDs→detail pagination pattern, OAuth2 token responses (401 recovery), detection vs tombstone distinction, and fully deterministic generation (no `thread_rng`, no `SystemTime::now`). Validated by 37 new integration tests; default workspace test count unchanged at 1522.

---

## Architecture Changes

```mermaid
graph TD
    Generator["prism-dtu-crowdstrike/src/generator.rs\n(NEW — fixture-gen feature)"]
    Common["prism-dtu-common::generator\n(seeded_rng, GenOpts, FixtureSet)"]
    State["prism-dtu-crowdstrike/src/state.rs\n(existing — read for field alignment)"]
    Schema["references/schemas/crowdstrike/types.rs\n(existing — read-only reference)"]
    Tests["tests/bc_3_4_crowdstrike_generator.rs\n(NEW — 37 tests, fixture-gen feature)"]

    Generator -->|uses| Common
    Generator -->|field-aligned with| State
    Generator -.->|derives shapes from| Schema
    Tests -->|exercises| Generator
    style Generator fill:#90EE90
    style Tests fill:#90EE90
```

<details>
<summary><strong>Architecture Decision Record</strong></summary>

### ADR: CrowdStrike Generator Follows Armis Pattern (S-3.7.04)

**Context:** CrowdStrike's API differs from Armis via a mandatory 2-step ID-list → device-detail join. The FixtureSet must represent both steps within a single records slice.

**Decision:** Use a tagged JSON object convention with `"_record_type"` sentinel field to distinguish `IdPage` records from `FalconDevice` detail records within `FixtureSet::records`. Generator is pure-core under `fixture-gen` feature gate.

**Rationale:** Consistent with S-3.7.04 (Armis) generator structure. Tagged variant avoids introducing a new enum into the common crate. Single `FixtureSet` return preserves the shared contract from `prism-dtu-common`.

**Alternatives Considered:**
1. Separate `IdFixtureSet` / `DetailFixtureSet` return types — rejected because: breaks the common `generate()` contract; callers would need sensor-specific dispatch.
2. `serde_json::Value::Array` with index-based convention — rejected because: fragile ordering; no self-describing type tag.

**Consequences:**
- DTU handler must inspect `_record_type` field; this is documented in generator.rs comments.
- Schema validation in tests correctly ignores the sentinel field.

</details>

---

## Story Dependencies

```mermaid
graph LR
    S370["S-3.7.00\n✅ merged"] --> S3705["S-3.7.05\n🟡 this PR"]
    S371["S-3.7.01\n✅ merged"] --> S3705
    style S3705 fill:#FFD700
```

Dependencies S-3.7.00 (CrowdStrike schema derivation) and S-3.7.01 (common generator trait) are both merged. S-3.7.05 blocks no other story.

---

## Spec Traceability

```mermaid
flowchart LR
    BC1["BC-3.4.001\nDeterminism"] --> AC6["AC-006\nByte-identical calls"]
    BC2["BC-3.4.002\nSchema conformance"] --> AC2["AC-002\n2-step pagination"]
    BC2 --> AC3["AC-003\nOAuth2 token"]
    BC2 --> AC4["AC-004\nstate.rs alignment"]
    BC3["BC-3.4.003\n8 archetypes"] --> AC1["AC-001\nBaseline counts"]
    BC4["BC-3.4.004\nOrg-tagged IDs"] --> AC5["AC-005\nOrg-tagged IDs"]
    BC4 --> AC7["AC-007\nDisjoint OrgA/OrgB"]
    AC1 --> T1["test_crowdstrike_all_archetypes_baseline_counts"]
    AC2 --> T2["test_crowdstrike_two_step_pagination_shape"]
    AC3 --> T3["test_crowdstrike_oauth2_token_401_then_200"]
    AC5 --> T4["test_crowdstrike_org_tagged_ids"]
    AC6 --> T5["test_crowdstrike_determinism"]
    T1 --> S1["generator.rs"]
    T2 --> S1
    T3 --> S1
    T4 --> S1
    T5 --> S1
```

| BC | VP | AC | Test | Status |
|----|----|----|------|--------|
| BC-3.4.001 | VP-108 | AC-006 | `test_crowdstrike_determinism` | PASS |
| BC-3.4.002 | VP-112 | AC-002 | `test_crowdstrike_two_step_pagination_shape` | PASS |
| BC-3.4.002 | VP-113 | AC-003 | `test_crowdstrike_oauth2_token_401_then_200` | PASS |
| BC-3.4.002 | VP-120 | AC-004 | `test_crowdstrike_state_field_alignment` | PASS |
| BC-3.4.003 | VP-114 | AC-001 | `test_crowdstrike_all_archetypes_baseline_counts` | PASS |
| BC-3.4.003 | VP-119 | AC-001 | `test_crowdstrike_all_archetypes_baseline_counts` | PASS |
| BC-3.4.004 | VP-121 | AC-005,AC-007 | `test_crowdstrike_org_tagged_ids` | PASS |

---

## Test Evidence

### Coverage Summary

| Metric | Value | Threshold | Status |
|--------|-------|-----------|--------|
| New integration tests | 37/37 pass | 100% | PASS |
| Workspace tests (unchanged) | 1522/1522 | 100% | PASS |
| Coverage | >80% (generator.rs) | >80% | PASS |
| Mutation kill rate | N/A — wave gate | >90% | N/A |
| Holdout satisfaction | N/A — wave gate | >0.85 | N/A |

### Test Flow

```mermaid
graph LR
    Integration["37 Integration Tests\n(fixture-gen feature)"]
    Workspace["1522 Workspace Tests\n(default)"]

    Integration -->|37/37 GREEN at a0f0c5ac| Pass1["PASS"]
    Workspace -->|unchanged count| Pass2["PASS"]

    style Pass1 fill:#90EE90
    style Pass2 fill:#90EE90
```

| Metric | Value |
|--------|-------|
| **New tests** | 37 added (0 modified) |
| **Test file** | `crates/prism-dtu-crowdstrike/tests/bc_3_4_crowdstrike_generator.rs` |
| **Feature gate** | `--features fixture-gen` required |
| **Total workspace suite** | 1522 tests PASS (default cargo test) |
| **Regressions** | 0 |

<details>
<summary><strong>Key Tests (This PR)</strong></summary>

| Test | Result |
|------|--------|
| `test_crowdstrike_all_archetypes_baseline_counts` | PASS |
| `test_crowdstrike_two_step_pagination_shape` | PASS |
| `test_crowdstrike_oauth2_token_401_then_200` | PASS |
| `test_crowdstrike_containment_status_present` | PASS |
| `test_crowdstrike_state_field_alignment` | PASS |
| `test_crowdstrike_schema_valid` | PASS |
| `test_crowdstrike_schema_drift_flag` | PASS |
| `test_crowdstrike_org_tagged_ids` | PASS |
| `test_crowdstrike_determinism` | PASS |
| + 28 additional parameterised archetype/VP tests | PASS |

</details>

---

## Demo Evidence

| Recording | ACs | Result |
|-----------|-----|--------|
| `AC-001-all-37-tests-green.gif` | AC-001 through AC-007 (all 37 tests) | PASS |
| `AC-002-quirks-pagination-oauth2.gif` | AC-002, AC-003 (pagination + OAuth2 quirks) | PASS |

Evidence location (feature branch): `docs/demo-evidence/S-3.7.05/`
Evidence report: `docs/demo-evidence/S-3.7.05/evidence-report.md`

Command demonstrated: `cargo test -p prism-dtu-crowdstrike --features fixture-gen --test bc_3_4_crowdstrike_generator`
Result: `test result: ok. 37 passed; 0 failed; 0 ignored`

---

## Holdout Evaluation

N/A — evaluated at wave gate.

---

## Adversarial Review

N/A — evaluated at Phase 5 (wave-level adversarial pass).

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

### Scope
Generator is pure test-support code gated behind `#[cfg(feature = "fixture-gen")]`. Not compiled into production binaries. No I/O, no network, no credentials, no user input surfaces.

### SAST
- No injection surfaces (no SQL, no shell, no network calls)
- No credential handling
- Deterministic seeded RNG (no entropy side-channels)
- No `unsafe` blocks introduced

### Dependency Audit
- No new dependencies added to Cargo.toml (stub already had all required deps)
- `cargo audit`: CLEAN (no new advisories)

### CrowdStrike-Specific
- `access_token` in OAuth2 fixture is a deterministic test string (format: `tok-<org_slug>-<seed>-<n>`) — not a real credential; never logs to stdout in production builds
- AuthOutage archetype: 401 fixture has no real token material

</details>

---

## CrowdStrike-Specific Implementation Notes

### 2-Step Pagination (VP-114)
`PaginationEdgeCases` emits `IdPage` records first (one per page, exactly `default_page_size` IDs each), then `FalconDevice` detail records. `FixtureSet::cursors` contains one FQL offset cursor per page (3 pages → 3 cursors). Records are tagged with `"_record_type": "id_page"` vs `"_record_type": "device_detail"`.

### OAuth2 Token Fixture (VP-113)
`AuthOutage` archetype: first record is an `OAuth2TokenResponse` with `status_code: 401`. Subsequent records: valid `OAuth2TokenResponse` with `access_token` of format `tok-<org_slug>-<seed>-<call_n>`, `expires_in: 1800`, `token_type: "bearer"`, `scope: "read"`. Configurable recovery via `overrides = {"auth_outage": {"recovery_after_calls": N}}` (EC-002).

### Detection vs Tombstone (VP-121)
`HighChurn` archetype: ≥20 tombstone records with `device_id` formatted as `dev-<slug>-<seed>-tomb-<n>`. Non-tombstone detections use `detection_id` formatted as `alert-<slug>-<seed>-<n>`.

### Determinism (BC-3.4.001)
`scaled()` uses `floor()` per BC-3.4.003 invariant 3. `org_slug()` uses first 4 bytes of `org_id` as 8 hex chars — ensures disjoint OrgA/OrgB slugs (VP-119). No `thread_rng`, no `SystemTime::now`, no side effects anywhere in the call stack.

### SchemaDrift (VP-120)
Sets `device_id` to `Value::Null` on `records[0]` + sets `provenance.schema_valid = false`.

---

## Risk Assessment & Deployment

### Blast Radius
- **Systems affected:** `prism-dtu-crowdstrike` crate only (test support code, feature-gated)
- **User impact:** None — not compiled into production binaries; no runtime behavior change
- **Data impact:** None — pure test fixture generation
- **Risk Level:** LOW

### Performance Impact
| Metric | Before | After | Delta | Status |
|--------|--------|-------|-------|--------|
| Production binary size | unchanged | unchanged | 0 | OK |
| Workspace test suite | 1522 | 1522 | 0 | OK |
| CI wall time | baseline | +~30s (37 new tests) | minimal | OK |

### Feature Flags
| Flag | Controls | Default |
|------|----------|---------|
| `fixture-gen` | CrowdStrike generator module compilation | off (dev/test only) |

---

## Traceability

| BC | VP | AC | Test | Status |
|----|----|----|------|--------|
| BC-3.4.001 | VP-108 | AC-006 | `test_crowdstrike_determinism` | PASS |
| BC-3.4.002 | VP-112 | AC-002 | `test_crowdstrike_two_step_pagination_shape` | PASS |
| BC-3.4.002 | VP-113 | AC-003 | `test_crowdstrike_oauth2_token_401_then_200` | PASS |
| BC-3.4.002 | VP-120 | AC-004 | `test_crowdstrike_state_field_alignment` | PASS |
| BC-3.4.003 | VP-114 | AC-001 | `test_crowdstrike_all_archetypes_baseline_counts` | PASS |
| BC-3.4.003 | VP-119 | AC-001 | `test_crowdstrike_all_archetypes_baseline_counts` | PASS |
| BC-3.4.004 | VP-121 | AC-005 | `test_crowdstrike_org_tagged_ids` | PASS |
| BC-3.4.004 | VP-121 | AC-007 | proptest disjoint slugs | PASS |

<details>
<summary><strong>Full VSDD Contract Chain</strong></summary>

```
BC-3.4.001 -> VP-108 -> test_crowdstrike_determinism -> generator.rs:seeded_rng -> CI-GREEN
BC-3.4.002 -> VP-112 -> test_crowdstrike_two_step_pagination_shape -> generator.rs:paginate_crowdstrike -> CI-GREEN
BC-3.4.002 -> VP-113 -> test_crowdstrike_oauth2_token_401_then_200 -> generator.rs:auth_outage -> CI-GREEN
BC-3.4.002 -> VP-120 -> test_crowdstrike_state_field_alignment -> generator.rs:device_id -> state.rs -> CI-GREEN
BC-3.4.003 -> VP-114 -> test_crowdstrike_all_archetypes_baseline_counts -> generator.rs:generate -> CI-GREEN
BC-3.4.003 -> VP-119 -> test_crowdstrike_all_archetypes_baseline_counts -> generator.rs:scaled -> CI-GREEN
BC-3.4.004 -> VP-121 -> test_crowdstrike_org_tagged_ids -> generator.rs:org_slug -> CI-GREEN
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
  holdout-evaluation: N/A (wave gate)
  adversarial-review: N/A (wave gate)
  formal-verification: skipped
  convergence: achieved
convergence-metrics:
  test-kill-rate: "37/37 (100%)"
  implementation-ci: green
  holdout-satisfaction: "N/A"
adversarial-passes: N/A
models-used:
  builder: claude-sonnet-4-6
generated-at: "2026-04-28T00:00:00Z"
story: S-3.7.05
wave: 3
```

</details>

---

## Pre-Merge Checklist

- [x] All CI status checks passing
- [x] 37/37 new tests GREEN; 1522 workspace tests unchanged
- [x] No critical/high security findings (pure test-support code, feature-gated)
- [x] Demo evidence present: 2 recordings per AC coverage in `docs/demo-evidence/S-3.7.05/`
- [x] Dependency PRs S-3.7.00 and S-3.7.01 merged
- [x] Feature flag `fixture-gen` gates all generator code (off by default)
- [x] No new Cargo.toml dependencies added
- [x] Determinism enforced: no `thread_rng`, no `SystemTime::now`
