---
document_type: story
story_id: "S-MAINT-EAUTH-COLLISION-001"
title: "E-AUTH-001/002 Collision Resolution — Reallocate SpecEngineError OAuth2 Variants to E-AUTH-008/009"
wave: post-demo-maintenance
epic_id: maintenance
priority: P2
status: draft
version: "1.0"
spec_version: "v1.0"
level: ops
producer: story-writer
timestamp: "2026-06-16"
modified: "2026-06-16"
input-hash: ""
inputs:
  - .factory/specs/prd-supplements/error-taxonomy.md
  - .factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md
  - .factory/specs/behavioral-contracts/BC-2.01.016-sensor-auth-open-trait-contract.md
  - crates/prism-spec-engine/src/error.rs
  - crates/prism-core/src/error.rs
  - crates/prism-mcp/src/error_mapping.rs
  - crates/prism-bin/src/spec_driven_adapter.rs
  - crates/prism-bin/tests/bc_2_01_013_spec_driven_adapter.rs
  - crates/prism-spec-engine/tests/crowdstrike_oauth2_plugin_tests.rs
traces_to: "DRIFT-EAUTH-CODE-COLLISION-001"
anchors: "DRIFT-EAUTH-CODE-COLLISION-001"
closes_finding: "DRIFT-EAUTH-CODE-COLLISION-001"
drift_anchor: "DRIFT-EAUTH-CODE-COLLISION-001"
cycle: "v1.0.0-greenfield"
phase: 3
tdd_mode: strict
track: "Platform Engineering"
subsystems:
  - SS-01
  - SS-16
crates_touched:
  - prism-spec-engine
  - prism-mcp
  - prism-bin
  - prism-core
target_module: "crates/prism-spec-engine/src/error.rs, crates/prism-mcp/src/error_mapping.rs, crates/prism-bin/src/spec_driven_adapter.rs"
behavioral_contracts:
  - BC-2.01.013
  - BC-2.01.016
# BC status: pending PO authorship for any net-new BCs; BC-2.01.013 and BC-2.01.016
# own the error-code surface for sensor adapters and auth; status can advance to ready
# once PO confirms AC↔BC traces below are sufficient or authors supplemental BCs.
verification_properties: []
depends_on:
  # Sequenced AFTER the live-demo capstone (T13 multi-client SOC-analyst narrative story
  # + T14 demo recording). This story must NOT be scheduled on the demo critical path.
  # The demo capstone story IDs are not yet finalized; update this field with the
  # canonical T13 story ID once it is materialized.
  - "demo-capstone-T13-T14"  # placeholder — replace with canonical story ID at scheduling time
blocks: []
points: 5
estimated_days: 1.0
risk: MEDIUM
acceptance_criteria_count: 6
red_gate_tests: 4
estimated_passes: "2"
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
design_source: "error-taxonomy.md v1.84 §IMPLEMENTER FOLLOW-UP"
---

# S-MAINT-EAUTH-COLLISION-001: E-AUTH-001/002 Collision Resolution

## Narrative

As a Prism operator and maintainer, I want `SpecEngineError::AuthAcquisitionFailed`
and `SpecEngineError::AuthRefreshFailed` to emit the dedicated codes E-AUTH-008 and
E-AUTH-009 respectively, so that E-AUTH-001 and E-AUTH-002 are exclusively owned by
`PrismError::InvalidOrgSlug` and `PrismError::InvalidAnalystId` (identity-validation)
— eliminating the live code collision documented in DRIFT-EAUTH-CODE-COLLISION-001
and enabling monitoring rules, alerts, and operator runbooks to rely on a single
unambiguous code per condition.

## Scheduling Note

**SCHEDULED POST-LIVE-DEMO (user directive 2026-06-16).** This story belongs to the
post-demo maintenance wave. It is sequenced AFTER the live-demo capstone (T13
multi-client SOC-analyst narrative story + T14 demo recording). It is NOT on the demo
critical path. The `depends_on` field carries a placeholder `demo-capstone-T13-T14`
that must be replaced with the canonical T13 story ID when that story is materialized.

Do NOT advance this story's `status` to `ready` until the demo capstone has shipped.

## Background

DRIFT-EAUTH-CODE-COLLISION-001 was registered at D-1192 (2026-06-16, taxonomy v1.84).
Two semantically distinct conditions share the same error code:

**E-AUTH-001 collision:**
- `PrismError::InvalidOrgSlug` in `prism-core` — identity-validation: the org/tenant
  slug failed format validation. Display: `"E-AUTH-001: invalid tenant ID: {reason}"`.
  MCP routes to `-32602 INVALID_PARAMS`.
- `SpecEngineError::AuthAcquisitionFailed` in `prism-spec-engine` — sensor OAuth2: the
  `AuthProvider::acquire_token` implementation could not complete the OAuth2 flow.
  Display: `"E-AUTH-001: auth token acquisition failed for sensor '{sensor_id}', client
  '{client_id}': {detail}"`. Used by the pipeline executor and prism-bin
  spec-driven adapter.

**E-AUTH-002 collision:**
- `PrismError::InvalidAnalystId` in `prism-core` — identity-validation: the analyst
  identifier failed format validation. Display: `"E-AUTH-002: invalid analyst ID: {reason}"`.
  MCP routes to `-32602 INVALID_PARAMS`.
- `SpecEngineError::AuthRefreshFailed` in `prism-spec-engine` — sensor OAuth2: double-401
  after token re-acquisition; pipeline aborts. Display: `"E-AUTH-002: auth refresh failed
  for sensor '{sensor_id}', client '{client_id}': HTTP 401 persisted after token
  re-acquisition on step '{step_name}'"`. Pinned by `bc_2_01_013_spec_driven_adapter.rs`
  test `F-004-R` (~12 assertions).

**Resolution direction (per D-1192):** Reallocate the SpecEngineError OAuth2 variants
to new codes at the E-AUTH tail. Proposed: `AuthAcquisitionFailed → E-AUTH-008`,
`AuthRefreshFailed → E-AUTH-009`. The implementer MUST verify these slots are free in
error-taxonomy.md v1.84 before allocating (the taxonomy as of v1.84 shows E-AUTH-004..007
and E-AUTH-010/011/020 allocated; E-AUTH-008/009 appear free — verify current state
at implementation time).

The `PrismError::InvalidOrgSlug` / `InvalidAnalystId` identity-validation variants
retain E-AUTH-001/002 unchanged. They are more widely referenced in MCP tooling tests
and have shorter blast radius. S-5.02 `ec_code_override` pins for E-AUTH-001/002/003
remain CORRECT for the identity-validation variants and are unaffected.

## Pre-Allocation Verification (implementer obligation)

Before writing any Red Gate tests or code, the implementer MUST run:

```bash
grep -n "E-AUTH-008\|E-AUTH-009" .factory/specs/prd-supplements/error-taxonomy.md
```

Expected: zero hits. If either code appears (another story allocated it), STOP and
escalate to the orchestrator before proceeding. Do not allocate a collided code.

## Behavioral Contracts

| BC ID | Title | Relevant Clause |
|-------|-------|-----------------|
| BC-2.01.013 | DataSource Trait Eliminates Per-Sensor Code Duplication | §Error Cases: per-sensor auth failure codes must be unambiguous; E-AUTH-008/009 replace E-AUTH-001/002 for OAuth2 sensor-auth paths |
| BC-2.01.016 | SensorAuth Open Trait — Plugin-Implementable Auth Contract (No Sealed Marker) | §Error Cases: `acquire_token` and token-refresh failure paths must emit canonical, non-collided error codes |

**Anchor justification per POL-5:**

- **BC-2.01.013** anchors AC-001, AC-002, AC-005, and AC-006. This BC owns the
  DataSource trait's error surface, including the sensor auth failure conditions
  tested in `bc_2_01_013_spec_driven_adapter.rs`. Renumbering the OAuth2 codes
  updates the error-code contract this BC governs.
- **BC-2.01.016** anchors AC-003 and AC-004. This BC defines the `SensorAuth` open
  trait's `acquire_token` and refresh-failure contract. It is the primary contract
  for `SpecEngineError::AuthAcquisitionFailed` (token acquisition) and
  `SpecEngineError::AuthRefreshFailed` (refresh failure).

## Acceptance Criteria

### AC-001: SpecEngineError::AuthAcquisitionFailed emits E-AUTH-008 (traces to BC-2.01.016 §Error Cases postcondition — acquire_token failure code)

`SpecEngineError::AuthAcquisitionFailed` in `crates/prism-spec-engine/src/error.rs`:

- The `#[error(...)]` attribute reads `"E-AUTH-008: auth token acquisition failed for
  sensor '{sensor_id}', client '{client_id}': {detail}"`.
- The doc comment reads `/// E-AUTH-008: ...` (no longer `E-AUTH-001`).
- Running `rg 'E-AUTH-001.*acquisition' crates/prism-spec-engine/src/error.rs` returns
  **zero hits**.
- Running `rg 'E-AUTH-008.*acquisition' crates/prism-spec-engine/src/error.rs` returns
  exactly **one hit**.

### AC-002: SpecEngineError::AuthRefreshFailed emits E-AUTH-009 (traces to BC-2.01.016 §Error Cases postcondition — refresh-failure code)

`SpecEngineError::AuthRefreshFailed` in `crates/prism-spec-engine/src/error.rs`:

- The `#[error(...)]` attribute reads `"E-AUTH-009: auth refresh failed for sensor
  '{sensor_id}', client '{client_id}': HTTP 401 persisted after token re-acquisition
  on step '{step_name}'"`.
- The doc comment reads `/// E-AUTH-009: ...` (no longer `E-AUTH-002`).
- Running `rg 'E-AUTH-002.*refresh' crates/prism-spec-engine/src/error.rs` returns
  **zero hits**.
- Running `rg 'E-AUTH-009.*refresh' crates/prism-spec-engine/src/error.rs` returns
  exactly **one hit**.

### AC-003: error-taxonomy.md gains canonical E-AUTH-008 and E-AUTH-009 rows (traces to BC-2.01.016 §Error Cases postcondition — taxonomy completeness)

`.factory/specs/prd-supplements/error-taxonomy.md` is updated (append-only per POL-1):

- A row for **E-AUTH-008** is added: `SpecEngineError::AuthAcquisitionFailed` — Display:
  `"E-AUTH-008: auth token acquisition failed for sensor '{sensor_id}', client
  '{client_id}': {detail}"`. Category: `authentication`. Retryable: No. Description:
  sensor OAuth2 token acquisition failed; `AuthProvider::acquire_token` could not
  complete the OAuth2 flow (credential resolution failure, HTTP error from token
  endpoint, or provider configuration error). Emitter: `prism-spec-engine`.
- A row for **E-AUTH-009** is added: `SpecEngineError::AuthRefreshFailed` — Display:
  `"E-AUTH-009: auth refresh failed for sensor '{sensor_id}', client '{client_id}':
  HTTP 401 persisted after token re-acquisition on step '{step_name}'"`. Category:
  `authentication`. Retryable: No. Description: sensor OAuth2 double-401 after
  token re-acquisition; pipeline aborts with no further retries. Applicable to
  `Oauth2ClientCredentials` auth type. Emitter: `prism-spec-engine`.
- The E-AUTH-001 and E-AUTH-002 rows in the taxonomy are amended (append-only: the
  old collision-notice text is superseded) to reflect that the collision is resolved:
  each row now documents a single unambiguous emitter (E-AUTH-001: `PrismError::InvalidOrgSlug`
  only; E-AUTH-002: `PrismError::InvalidAnalystId` only) with the collision-notice
  annotation replaced by a resolved-reference noting E-AUTH-008/009.
- The taxonomy version is bumped from v1.84 to v1.85 (or the next available version
  at implementation time).

### AC-004: TD-VSDD-060 sibling-sweep — all blast-radius sites updated (traces to BC-2.01.013 §Error Cases postcondition — no remaining cross-crate collision)

Every site identified in the DRIFT-EAUTH-CODE-COLLISION-001 blast-radius inventory is
updated from E-AUTH-001/E-AUTH-002 (OAuth2 meaning) to E-AUTH-008/E-AUTH-009.
Specifically:

1. **`crates/prism-spec-engine/src/error.rs`** — `AuthAcquisitionFailed` and
   `AuthRefreshFailed` `#[error]` attributes and doc comments updated (covered by
   AC-001 and AC-002).

2. **`crates/prism-mcp/src/error_mapping.rs`** — Any arm comments or routing notes
   that cite the old OAuth2 codes E-AUTH-001/E-AUTH-002 for the SpecEngineError
   variants are updated to E-AUTH-008/E-AUTH-009. The `PrismError::InvalidOrgSlug`
   and `PrismError::InvalidAnalystId` arms (identity-validation, E-AUTH-001/002)
   remain UNCHANGED.

3. **`crates/prism-bin/src/spec_driven_adapter.rs`** — Doc comments or inline comments
   that cite E-AUTH-002 in the context of auth-refresh failure are updated to E-AUTH-009.
   S-5.02 `ec_code_override` pins for identity-validation errors (E-AUTH-001/002/003 on
   `InvalidOrgSlug`/`InvalidAnalystId`/`InvalidClientId`) remain UNCHANGED — they are
   not OAuth2 paths.

4. **`crates/prism-bin/tests/bc_2_01_013_spec_driven_adapter.rs`** — The ~12 test
   assertions in test `F-004-R` (and any related functions) that pin the string
   `"E-AUTH-002"` for `AuthRefreshFailed` display output are updated to `"E-AUTH-009"`.
   Similarly any assertions pinning `"E-AUTH-001"` for `AuthAcquisitionFailed` are
   updated to `"E-AUTH-008"`. Assertions pinning E-AUTH-001/002 for
   identity-validation paths (`InvalidOrgSlug`/`InvalidAnalystId`) are UNCHANGED.

5. **`crates/prism-spec-engine/tests/crowdstrike_oauth2_plugin_tests.rs`** — Any
   assertions pinning `"E-AUTH-001"` or `"E-AUTH-002"` in the OAuth2 test cases are
   updated to `"E-AUTH-008"` and `"E-AUTH-009"` respectively.

6. **`crates/prism-core/tests/ac_5_prism_error_display.rs`** (if affected) — Assertions
   for `PrismError::InvalidOrgSlug` and `PrismError::InvalidAnalystId` that pin
   E-AUTH-001/002 for identity-validation are UNCHANGED.

7. **BC-2.01.013 §Error Cases** — The OAuth2 auth-failure error code citations in the
   spec are updated from E-AUTH-001/002 to E-AUTH-008/009. BC version is bumped.

8. **BC-2.01.016 §Error Cases** — The `acquire_token` and refresh-failure error code
   citations in the spec are updated from E-AUTH-001/002 to E-AUTH-008/009. BC version
   is bumped.

### AC-005: No-collision invariant — grep proof (traces to BC-2.01.013 §Error Cases postcondition — single-meaning per code)

After all code and spec changes:

```bash
rg '"E-AUTH-001' crates/ --type rust
```

Returns hits ONLY in:
- `prism-core/src/error.rs` (the `InvalidOrgSlug` `#[error]` attribute)
- `prism-mcp/src/error_mapping.rs` (the identity-validation arm comment/routing, if any)
- `prism-bin/src/spec_driven_adapter.rs` (identity-validation `ec_code_override`)
- Test files that test `PrismError::InvalidOrgSlug` specifically

Returns **zero hits** in:
- `prism-spec-engine/src/error.rs` (AuthAcquisitionFailed must now show E-AUTH-008)
- `prism-spec-engine/tests/crowdstrike_oauth2_plugin_tests.rs`
- `bc_2_01_013_spec_driven_adapter.rs` test `F-004-R`

Equivalent check for E-AUTH-002: `rg '"E-AUTH-002' crates/ --type rust` returns
hits ONLY for the `InvalidAnalystId` identity-validation path; zero hits for OAuth2
refresh paths.

### AC-006: full `just check` green (traces to BC-2.01.013 — no regressions)

`just check` exits 0 across the entire workspace after all code, test, and spec
changes. No compilation error, no clippy warning, no test failure.

## Red Gate Tests

These tests must be written as **failing** (Red Gate) before any implementation
begins, per TDD Iron Law. They drive the Display-string changes in prism-spec-engine
before the `#[error]` attributes are updated.

### RG-EAUTH-001: AuthAcquisitionFailed emits E-AUTH-008

**File:** `crates/prism-spec-engine/tests/` — new file
`bc_2_01_016_auth_acquisition_renumber.rs` OR inline in
`crates/prism-spec-engine/tests/crowdstrike_oauth2_plugin_tests.rs`.

**Test name:** `test_auth_acquisition_failed_emits_e_auth_008`

**Behavior:** Construct a `SpecEngineError::AuthAcquisitionFailed { sensor_id: ...,
client_id: ..., detail: ... }` and assert the Display output starts with `"E-AUTH-008"`.

**Fails before:** `#[error]` still reads `"E-AUTH-001: ..."`.
**Passes after:** `#[error]` updated to `"E-AUTH-008: ..."`.

### RG-EAUTH-002: AuthRefreshFailed emits E-AUTH-009

**File:** Same file as RG-EAUTH-001.

**Test name:** `test_auth_refresh_failed_emits_e_auth_009`

**Behavior:** Construct a `SpecEngineError::AuthRefreshFailed { sensor_id: ...,
client_id: ..., step_name: ... }` and assert the Display output starts with `"E-AUTH-009"`.

**Fails before:** `#[error]` still reads `"E-AUTH-002: ..."`.
**Passes after:** `#[error]` updated to `"E-AUTH-009: ..."`.

### RG-EAUTH-003: bc_2_01_013_spec_driven_adapter.rs F-004-R pins E-AUTH-009 for refresh failure

**File:** `crates/prism-bin/tests/bc_2_01_013_spec_driven_adapter.rs`.

**Test name:** (update existing assertions in `F-004-R` / `test_f004_r_auth_refresh_failed`
or equivalent — the ~12 assertions that currently pin `"E-AUTH-002"` for the
`AuthRefreshFailed` path must be updated to `"E-AUTH-009"`).

**Behavior:** After the update, each assertion checks that the display string contains
`"E-AUTH-009"` (not `"E-AUTH-002"`).

**Fails before:** Display still emits `E-AUTH-002`; the UPDATED test assertions
checking E-AUTH-009 would fail.
**Passes after:** `prism-spec-engine` `AuthRefreshFailed` emits `E-AUTH-009`; updated
assertions pass.

### RG-EAUTH-004: No-collision workspace scan guard

**File:** New test in `crates/prism-spec-engine/tests/` or
`crates/prism-core/tests/e_auth_namespace_invariants.rs`.

**Test name:** `test_e_auth_001_002_not_emitted_by_spec_engine_oauth2_paths`

**Behavior:** Using `std::process::Command`, run:
```rust
let output = std::process::Command::new("git")
    .args(["grep", "E-AUTH-001", "--", "crates/prism-spec-engine/"])
    .output()
    .expect("git grep failed");
let hits = String::from_utf8_lossy(&output.stdout);
assert!(
    !hits.contains("AuthAcquisitionFailed"),
    "E-AUTH-001 must not appear alongside AuthAcquisitionFailed in prism-spec-engine \
     after renumber to E-AUTH-008"
);
```
Apply same pattern for `E-AUTH-002` / `AuthRefreshFailed` → E-AUTH-009.

**Fails before:** `prism-spec-engine/src/error.rs` still contains `E-AUTH-001`/`E-AUTH-002`.
**Passes after:** Both variants updated to E-AUTH-008/009.

## Tasks

### Implementer tasks (develop-based worktree)

1. **Pre-allocation verification (MANDATORY FIRST STEP):**

   ```bash
   grep -n "E-AUTH-008\|E-AUTH-009" .factory/specs/prd-supplements/error-taxonomy.md
   ```

   Expected: zero hits. If non-zero, STOP and escalate to the orchestrator.

2. **Write the 4 Red Gate tests** (RG-EAUTH-001 through RG-EAUTH-004). Verify each
   fails with the CURRENT code before proceeding.

3. **Update `crates/prism-spec-engine/src/error.rs`:**
   - `AuthAcquisitionFailed`: change doc comment and `#[error]` from `E-AUTH-001:` to
     `E-AUTH-008:`.
   - `AuthRefreshFailed`: change doc comment and `#[error]` from `E-AUTH-002:` to
     `E-AUTH-009:`.

4. **Sibling-sweep via TD-VSDD-060:**

   ```bash
   rg '"E-AUTH-001' crates/prism-spec-engine/ --type rust
   rg '"E-AUTH-002' crates/prism-spec-engine/ --type rust
   rg '"E-AUTH-001' crates/prism-bin/ --type rust
   rg '"E-AUTH-002' crates/prism-bin/ --type rust
   rg '"E-AUTH-001' crates/prism-mcp/ --type rust
   rg '"E-AUTH-002' crates/prism-mcp/ --type rust
   ```

   For EVERY hit that refers to an OAuth2 sensor-auth meaning, update from
   E-AUTH-001/002 to E-AUTH-008/009. Hits that refer to identity-validation
   (`InvalidOrgSlug`/`InvalidAnalystId`) are UNCHANGED.

5. **Update `bc_2_01_013_spec_driven_adapter.rs` test `F-004-R` assertions (~12):**
   Change every `assert!(display.contains("E-AUTH-002"))` (or equivalent) in the auth-
   refresh-failure test path to `"E-AUTH-009"`. Run:
   ```bash
   cargo nextest run -p prism-bin -E 'test(F-004-R)' --no-fail-fast
   ```
   Must exit 0.

6. **Update `crowdstrike_oauth2_plugin_tests.rs` assertions:**
   Replace `"E-AUTH-001"` / `"E-AUTH-002"` OAuth2 assertions with `"E-AUTH-008"` /
   `"E-AUTH-009"`. Run:
   ```bash
   cargo nextest run -p prism-spec-engine --no-fail-fast
   ```
   Must exit 0.

7. **Confirm identity-validation tests are UNCHANGED:**

   ```bash
   cargo nextest run -p prism-core -E 'test(e_auth_001)' --no-fail-fast
   cargo nextest run -p prism-core -E 'test(e_auth_002)' --no-fail-fast
   ```

   These MUST still pass without modification. Any failure here means the sweep
   incorrectly modified an identity-validation site — revert and re-scope.

8. **Confirm S-5.02 `ec_code_override` pins are UNCHANGED:**
   The `ec_code_override: Some("E-AUTH-001")` / `Some("E-AUTH-002")` / `Some("E-AUTH-003")`
   pins in `spec_driven_adapter.rs` are for the identity-validation variants
   (`InvalidOrgSlug`/`InvalidAnalystId`/`InvalidClientId`). They must NOT be changed.
   Verify via:
   ```bash
   rg 'ec_code_override.*E-AUTH-00[123]' crates/prism-bin/src/
   ```
   If these hits are absent after the sweep, the sweep overreached — revert and re-scope.

9. **Run per-crate gate:**

   ```bash
   just iter prism-spec-engine
   just iter prism-bin
   just iter prism-mcp
   just iter prism-core
   ```

   All must exit 0.

10. **Run no-collision grep proof (AC-005 verification):**

    ```bash
    rg '"E-AUTH-001' crates/ --type rust
    rg '"E-AUTH-002' crates/ --type rust
    ```

    Inspect output: confirm zero OAuth2-auth hits for E-AUTH-001/002.

11. **Run full pre-push gate:** `just check` — exits 0 across all crates.

12. **Commit** with message citing `DRIFT-EAUTH-CODE-COLLISION-001`,
    `S-MAINT-EAUTH-COLLISION-001`. No AI attribution per project git conventions.

### Spec tasks (product-owner / architect — .factory/ artifacts)

13. **Update error-taxonomy.md:** Add E-AUTH-008 and E-AUTH-009 rows per AC-003.
    Amend E-AUTH-001/002 rows to reflect collision-resolved status (single emitter
    per code). Bump taxonomy version. Single atomic commit per TD-VSDD-053.

14. **Update BC-2.01.013 §Error Cases:** Change OAuth2 auth-failure code citations
    from E-AUTH-001/002 to E-AUTH-008/009. Bump BC version.

15. **Update BC-2.01.016 §Error Cases:** Change `acquire_token` and refresh-failure
    code citations from E-AUTH-001/002 to E-AUTH-008/009. Bump BC version.

16. **Commit** `.factory/` artifacts in a single atomic commit per TD-VSDD-053.

## Previous Story Intelligence

- **D-1192 (2026-06-16, taxonomy v1.83→v1.84):** DRIFT-EAUTH-CODE-COLLISION-001
  registered. E-AUTH-001/002 live collision confirmed across both `develop` and
  `.worktrees/S-5.02` — this is NOT an S-5.02 branch artifact, it exists on develop.
  Proposed renumber: `AuthAcquisitionFailed → E-AUTH-008`, `AuthRefreshFailed → E-AUTH-009`.
  E-AUTH-010/011/020 ratified in same burst. Taxonomy IMPLEMENTER FOLLOW-UP section
  added inline.

- **S-MAINT-ECRED-TAXONOMY-SYNC-001 (2026-06-07, MERGED PR #175):** Established the
  canonical pattern for error-code collision resolution in this codebase: (1) write Red
  Gate tests first, (2) update `#[error]` attributes and doc comments, (3) sibling-sweep
  via `rg`, (4) tighten test assertions, (5) rewrite taxonomy rows, (6) update BC
  §Error Cases citations. Follow the same sequence here.

- **Key lesson from ECRED story:** When an error code appears in test assertions as a
  string literal (e.g., `assert!(display.contains("E-AUTH-002"))`), the test is a
  source-of-truth anchor for the code. The Red Gate for this story must UPDATE those
  assertions to the new code BEFORE the `#[error]` attribute changes — otherwise the
  existing passing tests would continue to pass when they should fail (masking the
  renumber). Write the new-code assertion tests AND update the old-code assertions in
  the Red Gate step, then drive both to green with the `#[error]` change.

- **Key lesson (SID-1):** The `#[ignore]`'d integration tests in
  `crowdstrike_oauth2_plugin_tests.rs` may depend on a live Crowdstrike DTU. Use the
  display-string unit test pattern (RG-EAUTH-001/002) to drive the `#[error]` change
  without DTU dependency. Verify `#[ignore]`'d integration test expectations are updated
  in the same commit even if they remain `#[ignore]`'d.

## Architecture Compliance Rules

(Derived from CLAUDE.md §Conventions, error-taxonomy.md v1.84 §IMPLEMENTER FOLLOW-UP,
and the ECRED story precedent.)

1. **Verify free slots before allocating.** E-AUTH-008 and E-AUTH-009 MUST be
   confirmed free in error-taxonomy.md at implementation time. Do not rely on this
   story's authorship-time check alone — another burst may allocate them between now
   and delivery.

2. **`#[error]` is the sole source of truth for Display output** (per thiserror crate).
   Doc comments are informational. Update BOTH the `#[error]` attribute and the doc
   comment in the same edit to keep them in sync.

3. **Identity-validation variants are off-limits.** `PrismError::InvalidOrgSlug` and
   `PrismError::InvalidAnalystId` in `prism-core/src/error.rs` must NOT be modified.
   Their E-AUTH-001/002 codes are intentional and correct for identity-validation.
   S-5.02 `ec_code_override: Some("E-AUTH-001/002/003")` pins in `spec_driven_adapter.rs`
   must NOT be modified. If the sweep touches these, escalate immediately.

4. **TD-VSDD-060 sibling-site sweep is mandatory.** grep ALL of `crates/prism-spec-engine/`,
   `crates/prism-bin/`, and `crates/prism-mcp/` for `"E-AUTH-001"` and `"E-AUTH-002"`
   before committing. Any OAuth2-meaning hit that still shows the old code is an
   incomplete sweep.

5. **error-taxonomy.md is append-only per POL-1.** The E-AUTH-001/002 rows are amended
   (superseded text retained, collision-resolved annotation added) rather than deleted.
   The E-AUTH-008/009 rows are new appended rows.

6. **BC §Error Cases updates are mandatory.** AC-004 items 7 and 8 require that
   BC-2.01.013 and BC-2.01.016 §Error Cases are updated. These spec changes are
   in-scope for this story's spec-task step. If the implementer merges code without
   the BC updates, the spec-vs-code invariant is violated (Standing Rule for VSDD:
   spec wins; code brought into alignment).

7. **No AI attribution in commits** per project git conventions (CLAUDE.md).

8. **`just check` must exit 0** before the PR is opened.

9. **No scope expansion.** If grep reveals additional E-AUTH-001/002 OAuth2-meaning
   sites beyond the blast-radius inventory above, update them — that is TD-VSDD-060
   obligation. If grep reveals an unexpected collision with a third emitter, record a
   new DRIFT item; do NOT attempt to resolve three-way collisions in this story.

## Library & Framework Requirements

No new dependencies. All changes are to existing Rust source files and `.factory/`
Markdown.

| Tool | Purpose | Version |
|------|---------|---------|
| thiserror | `#[error(...)]` attribute macro on `SpecEngineError` variants | existing workspace pin |
| ripgrep (`rg`) | Site discovery for sibling-site sweep (TD-VSDD-060) | system |
| `just check` | Final pre-push gate | workspace Justfile |
| `cargo nextest` | Per-crate TDD inner loop | existing workspace pin |

## File Structure Requirements (§FSR)

| File | Action | Crate/Location |
|------|--------|----------------|
| `crates/prism-spec-engine/src/error.rs` | Modify — update `#[error]` + doc comment for `AuthAcquisitionFailed` (E-AUTH-001→E-AUTH-008) and `AuthRefreshFailed` (E-AUTH-002→E-AUTH-009) | prism-spec-engine |
| `crates/prism-mcp/src/error_mapping.rs` | Modify — update arm comments/routing notes that cite OAuth2 codes E-AUTH-001/002 to E-AUTH-008/009; identity-validation arms UNCHANGED | prism-mcp |
| `crates/prism-bin/src/spec_driven_adapter.rs` | Modify — update doc comments citing E-AUTH-002 for auth-refresh failure to E-AUTH-009; `ec_code_override` identity-validation pins UNCHANGED | prism-bin |
| `crates/prism-bin/tests/bc_2_01_013_spec_driven_adapter.rs` | Modify — update ~12 assertions in `F-004-R` from `"E-AUTH-002"` to `"E-AUTH-009"`; any `"E-AUTH-001"` OAuth2-path assertions to `"E-AUTH-008"` | prism-bin |
| `crates/prism-spec-engine/tests/crowdstrike_oauth2_plugin_tests.rs` | Modify — update OAuth2 assertions pinning `"E-AUTH-001"`/`"E-AUTH-002"` to `"E-AUTH-008"`/`"E-AUTH-009"` | prism-spec-engine |
| `crates/prism-spec-engine/tests/bc_2_01_016_auth_acquisition_renumber.rs` OR existing test file | Add — RG-EAUTH-001, RG-EAUTH-002 (display-string unit tests for new codes) | prism-spec-engine |
| `crates/prism-spec-engine/tests/` OR `crates/prism-core/tests/e_auth_namespace_invariants.rs` | Add — RG-EAUTH-003 (F-004-R assertion update) + RG-EAUTH-004 (no-collision workspace scan guard) | prism-spec-engine or prism-core |
| `crates/prism-core/tests/ac_5_prism_error_display.rs` | Verify UNCHANGED — identity-validation E-AUTH-001/002 assertions must still pass | prism-core (no-edit verify) |
| `.factory/specs/prd-supplements/error-taxonomy.md` | Modify — add E-AUTH-008 + E-AUTH-009 rows; amend E-AUTH-001/002 collision-notice to resolved-reference; bump version | .factory spec |
| `.factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md` | Modify — update §Error Cases OAuth2 auth-failure code cites E-AUTH-001/002→E-AUTH-008/009; bump version | .factory spec |
| `.factory/specs/behavioral-contracts/BC-2.01.016-sensor-auth-open-trait-contract.md` | Modify — update §Error Cases acquire_token and refresh-failure code cites E-AUTH-001/002→E-AUTH-008/009; bump version | .factory spec |

**Subsystem anchor justification:**

- **SS-01 (Sensor Adapters)** owns this story's primary scope because `SpecEngineError`
  (`prism-spec-engine`) is the error type for the sensor adapter pipeline (OAuth2
  token acquisition and refresh). The BC anchors (BC-2.01.013 and BC-2.01.016) are
  both in the SS-01 subsystem per the ARCH-INDEX Subsystem Registry.
- **SS-16 (Spec Engine)** is co-owner because `prism-spec-engine` implements the
  spec-driven adapter tier (Tier 1), and the `SpecEngineError` variants live in that
  crate. SS-16 and SS-01 share ownership of `prism-spec-engine` per ARCH-INDEX row 155.
- `prism-mcp` (SS-10) and `prism-bin` (SS-22) are touched only for sibling-sweep
  compliance (arm comments, doc comments, test assertions); they do not own the
  renumber decision.

**Dependency anchor justification:**

- `depends_on: [demo-capstone-T13-T14]` — this story is explicitly scheduled in the
  post-demo maintenance wave per user directive 2026-06-16. The demo capstone must
  ship before this story is dispatched. The placeholder ID must be replaced with the
  canonical T13 story ID at scheduling time.
- `blocks: []` — no currently-defined story is blocked by this maintenance work.
  S-5.02 is independent (its E-AUTH-001/002 identity-validation pins are unaffected).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Pre-allocation check finds E-AUTH-008 or E-AUTH-009 already allocated in taxonomy at implementation time | STOP — another burst allocated the code. Escalate to orchestrator for alternative slot allocation (e.g., E-AUTH-012/013). Do NOT proceed. |
| EC-002 | The sibling-sweep finds additional files citing `"E-AUTH-001"` or `"E-AUTH-002"` for OAuth2-meaning beyond the blast-radius inventory | Update all hits found — this is TD-VSDD-060 obligation in-scope for this story. |
| EC-003 | A `#[cfg(test)]` block in `prism-spec-engine/src/error.rs` itself asserts the old E-AUTH-001/002 codes | Update those assertions in-scope per TD-VSDD-060. |
| EC-004 | `just check` fails after updating `AuthRefreshFailed` because a match arm in prism-mcp or prism-bin exhaustively matches on it and references the old Display in a comment that causes a doc-test failure | Fix the doc-test or inline-comment reference. This is a TD-VSDD-060 sibling site, not out-of-scope. |
| EC-005 | The adversary questions whether BC-2.01.013 §Error Cases actually references E-AUTH-001/002 and whether an update is required | Read the current BC-2.01.013 file. If no explicit E-AUTH-001/002 code citation is present in §Error Cases, the BC update is limited to a normative cross-reference to the new taxonomy rows. Do not invent citations. |
| EC-006 | S-5.02 `ec_code_override` pins are accidentally modified during sweep | Revert those changes immediately. The `ec_code_override: Some("E-AUTH-001/002/003")` pins are correct for identity-validation; only the `SpecEngineError` OAuth2 variants change. |
| EC-007 | `crowdstrike_oauth2_plugin_tests.rs` has `#[ignore]`'d tests that pin the old codes but cannot be run to verify the fix without a live Crowdstrike DTU | Update the assertion strings in the `#[ignore]`'d tests anyway. Add a code comment: `// E-AUTH-008 after S-MAINT-EAUTH-COLLISION-001 renumber; was E-AUTH-001`. The test remains `#[ignore]`'d; the string update ensures no drift when the DTU is available per SID-1. |

## Architecture Mapping

| Component | Module | File | Pure/Effectful |
|-----------|--------|------|----------------|
| `SpecEngineError::AuthAcquisitionFailed` (renumber) | prism-spec-engine | `src/error.rs` | Pure (enum variant, no I/O) |
| `SpecEngineError::AuthRefreshFailed` (renumber) | prism-spec-engine | `src/error.rs` | Pure (enum variant, no I/O) |
| MCP error routing arm comments | prism-mcp | `src/error_mapping.rs` | Pure (match routing) |
| Spec-driven adapter doc comments | prism-bin | `src/spec_driven_adapter.rs` | Effectful (HTTP pipeline entry) |
| BC-2.01.013 spec-driven adapter tests | prism-bin | `tests/bc_2_01_013_spec_driven_adapter.rs` | Effectful (test exercises HTTP pipeline) |
| Crowdstrike OAuth2 plugin tests | prism-spec-engine | `tests/crowdstrike_oauth2_plugin_tests.rs` | Effectful (plugin test) |
| Display-string unit tests (RG-EAUTH-001/002) | prism-spec-engine | `tests/bc_2_01_016_auth_acquisition_renumber.rs` | Pure (construct enum variant, no I/O) |
| No-collision workspace scan (RG-EAUTH-004) | prism-spec-engine or prism-core | `tests/e_auth_namespace_invariants.rs` | Pure (invokes git grep as subprocess) |
| error-taxonomy.md | .factory/specs | `prd-supplements/error-taxonomy.md` | N/A — spec |
| BC-2.01.013 | .factory/specs | `behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md` | N/A — spec |
| BC-2.01.016 | .factory/specs | `behavioral-contracts/BC-2.01.016-sensor-auth-open-trait-contract.md` | N/A — spec |

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~7 000 |
| `prism-spec-engine/src/error.rs` (full SpecEngineError enum) | ~2 000 |
| `prism-mcp/src/error_mapping.rs` | ~1 500 |
| `prism-bin/src/spec_driven_adapter.rs` (relevant sections) | ~1 500 |
| `bc_2_01_013_spec_driven_adapter.rs` (F-004-R test block) | ~2 000 |
| `crowdstrike_oauth2_plugin_tests.rs` (relevant assertions) | ~1 200 |
| `prism-core/src/error.rs` (verify identity-validation variants are correct) | ~2 000 |
| `error-taxonomy.md` E-AUTH section (read before amending) | ~1 000 |
| BC-2.01.013 §Error Cases (read before amending) | ~1 000 |
| BC-2.01.016 §Error Cases (read before amending) | ~800 |
| `just check` output | ~500 |
| `rg` sibling-site sweep output | ~400 |
| **Total** | **~20 900** |

Context window headroom: ~21k tokens is ~6% of a 350k context window.
No splitting required. Implementer (code + test) and product-owner (spec changes)
can operate independently without context conflict.

## §References

Per POL-7 (verbatim BC H1 titles):

- BC-2.01.013 — *DataSource Trait Eliminates Per-Sensor Code Duplication*
- BC-2.01.016 — *SensorAuth Open Trait — Plugin-Implementable Auth Contract (No Sealed Marker)*
- error-taxonomy.md v1.84 §IMPLEMENTER FOLLOW-UP (DRIFT-EAUTH-CODE-COLLISION-001 registration)
- DRIFT-EAUTH-CODE-COLLISION-001 registered D-1192 2026-06-16

## §Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-06-16 | story-writer | Initial materialization from DRIFT-EAUTH-CODE-COLLISION-001 (D-1192, taxonomy v1.84) per user directive 2026-06-16 |
