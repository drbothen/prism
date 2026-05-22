# AC-007..010 — DTU-EXT-001..004 incident table parity gaps (legitimate deferred tests)

## Acceptance Criteria

- **AC-007 (DTU-EXT-001):** CrowdStrike incidents table parity — blocked on `prism-dtu-crowdstrike` DTU clone (S-6.07).
- **AC-008 (DTU-EXT-002):** Claroty incidents table parity — blocked on `prism-dtu-claroty` DTU clone (S-6.08).
- **AC-009 (DTU-EXT-003):** Cyberint incidents table parity — blocked on `prism-dtu-cyberint` DTU clone (S-6.09); incidents sub-test runs as explicit SKIP-assertion (NOT `#[ignore]`).
- **AC-010 (DTU-EXT-004):** Armis incidents table parity — blocked on `prism-dtu-armis` DTU clone (S-6.10).

## Evidence Type

These ACs have legitimate `#[ignore]` tests. The `#[ignore]` attribute is the correct deferral mechanism per TD-VSDD-059 (implementation discipline note in story). The BC-2.16.013 §Known Gaps section documents DTU-EXT-001..004 as planned but blocked on Wave 2 DTU clone stories.

## Source Evidence — `#[ignore]` Attributes

### CrowdStrike (AC-007 / DTU-EXT-001)

File: `crates/prism-spec-engine/tests/parity/crowdstrike.rs:151`

```rust
/// DTU-EXT-001 gap: incidents table parity is NOT exercised in v1 cycle.
/// Only detections + devices are tested here per story §Red Gate Test Set RG-04.
///
/// Tagged #[ignore] until S-6.07 merges per EC-016-013-006 / EC-016-013-001.
#[ignore = "requires prism-dtu-crowdstrike DTU clone (S-6.07 not yet merged; \
DTU-EXT-001..004 routes not yet implemented; tracking under PLUGIN-MIGRATION-Wave-2)"]
#[tokio::test]
async fn test_BC_2_16_013_dtu_parity_crowdstrike() {
```

### Claroty (AC-008 / DTU-EXT-002)

File: `crates/prism-spec-engine/tests/parity/claroty.rs:113`

```rust
/// Note: assets table deferred — DTU-EXT-002: Claroty DTU has /api/v1/devices, not /api/v1/assets.
/// Parity test pivots to alerts which has a DTU route at /api/v1/alerts per clone.rs build_router().
///
/// Tagged #[ignore] until S-6.08 merges.
#[ignore = "requires prism-dtu-claroty DTU clone (S-6.08 not yet merged; \
DTU-EXT-001..004 routes not yet implemented; tracking under PLUGIN-MIGRATION-Wave-2)"]
#[tokio::test]
async fn test_BC_2_16_013_dtu_parity_claroty() {
```

### Cyberint alerts (AC-009 / DTU-EXT-003)

File: `crates/prism-spec-engine/tests/parity/cyberint.rs:111`

```rust
/// Tagged #[ignore] until S-6.09 merges per EC-016-013-006 / EC-016-013-001.
#[ignore = "requires prism-dtu-cyberint DTU clone (S-6.09 not yet merged; \
DTU-EXT-001..004 routes not yet implemented; tracking under PLUGIN-MIGRATION-Wave-2)"]
#[tokio::test]
async fn test_BC_2_16_013_dtu_parity_cyberint() {
```

### Cyberint incidents SKIP-assertion (AC-009 special case — NOT `#[ignore]`, runs in CI)

File: `crates/prism-spec-engine/tests/parity/cyberint.rs:205`

```rust
/// This test is NOT `#[ignore]`'d — it runs in CI and passes by asserting the SKIP
/// verdict is returned for the incidents table (EC-016-013-002).
```

Test passes (explicit SKIP-assertion for the incidents table):

```
cargo nextest run -p prism-spec-engine \
  -E 'test(test_BC_2_16_013_dtu_parity_cyberint_incidents_skip)' --no-fail-fast

    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.30s
────────────
 Nextest run ID 3dad5d8a-b331-459d-8be0-b7da127d450f with nextest profile: default
    Starting 1 test across 32 binaries (425 tests skipped)
        PASS [   0.010s] (1/1) prism-spec-engine::parity_cyberint test_BC_2_16_013_dtu_parity_cyberint_incidents_skip
────────────
     Summary [   0.011s] 1 test run: 1 passed, 425 skipped
EXIT:0
```

### Armis (AC-010 / DTU-EXT-004)

File: `crates/prism-spec-engine/tests/parity/armis.rs:111`

```rust
/// Tagged #[ignore] until S-6.10 merges per EC-016-013-006 / EC-016-013-001.
#[ignore = "requires prism-dtu-armis DTU clone (S-6.10 not yet merged; \
DTU-EXT-001..004 routes not yet implemented; tracking under PLUGIN-MIGRATION-Wave-2)"]
#[tokio::test]
async fn test_BC_2_16_013_dtu_parity_armis() {
```

## BC-2.16.013 §Known Gaps Reference

The BC documents DTU-EXT-001..004 as planned gaps. Each is tracked via a specific future story:

| Gap ID | Sensor | Future Story |
|--------|--------|--------------|
| DTU-EXT-001 | CrowdStrike | S-6.07 |
| DTU-EXT-002 | Claroty | S-6.08 |
| DTU-EXT-003 | Cyberint | S-6.09 |
| DTU-EXT-004 | Armis | S-6.10 |

## Verdict

| AC | Status |
|----|--------|
| AC-007 (DTU-EXT-001 / CrowdStrike incidents) | DEFERRED — `#[ignore]` pending S-6.07; source evidence captured |
| AC-008 (DTU-EXT-002 / Claroty incidents) | DEFERRED — `#[ignore]` pending S-6.08; source evidence captured |
| AC-009 (DTU-EXT-003 / Cyberint incidents) | DEFERRED — alerts test `#[ignore]` pending S-6.09; incidents SKIP-assertion PASS |
| AC-010 (DTU-EXT-004 / Armis incidents) | DEFERRED — `#[ignore]` pending S-6.10; source evidence captured |

All deferrals are human-directed, attached to specific future stories, and blocked on a concrete external dependency (DTU clone availability). This meets Canonical Principle Rule 3 criteria.

## Metadata

| Field | Value |
|-------|-------|
| Captured at | 2026-05-22T08:05:20Z |
| Worktree HEAD SHA | 55b4f72daf3514599a87cd31866bc361e43fc1d6 |
| Branch | feature/PLUGIN-MIGRATION-001-D |
