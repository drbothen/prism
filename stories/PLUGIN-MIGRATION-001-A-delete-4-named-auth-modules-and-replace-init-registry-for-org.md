---
document_type: story
story_id: PLUGIN-MIGRATION-001-A
title: "prism-sensors: Delete 4 Named Auth Modules + Re-exports + Replace init_registry_for_org"
wave: 1
epic_id: PLUGIN-MIGRATION-001
priority: P0
status: ready
version: "v1.0"
level: "L4"
producer: story-writer
timestamp: "2026-05-22T00:00:00Z"
modified: "2026-05-22"
tdd_mode: strict
subsystems: [SS-01, SS-16]
# Subsystem anchor justifications:
#   SS-01 (Sensor Adapters, prism-sensors) owns all four auth modules being deleted, the
#   `init_registry_for_org` function, and `auth_type_name()` return values — core of this story.
#   SS-16 (Spec Engine, prism-spec-engine) owns BC-2.16.012 PluginRegistry dispatch; the
#   spec-catalog dispatch path that replaces `init_registry_for_org` lives in the spec engine.
crates_touched: [prism-sensors, prism-bin]
target_module: prism-sensors
capabilities: [CAP-029]
behavioral_contracts:
  - BC-2.01.016  # SensorAuth Open Trait — auth_type_name() return values for all 4 impls
                 #   are the direct target of this story; CyberintAuth/ClarotyAuth/ArmisAuth
                 #   values corrected; CrowdStrikeAuth unchanged; Red Gate test amended
  - BC-2.01.013  # DataSource Trait — spec-driven adapter pattern; confirms deletion of hand-
                 #   written Rust adapters is the correct outcome of TOML-spec parity being proven
  - BC-2.16.012  # PluginRegistry Dispatch — init_registry_for_org replacement dispatches through
                 #   spec-catalog open path; no hardcoded sensor name match arms survive
  - BC-3.2.001   # Per-Org Sensor Data Isolation — init_registry_for_org replacement must preserve
                 #   the OrgId-keyed composite dispatch contract (BC-3.2.001 precondition 4);
                 #   any replacement must NOT regress multi-tenant isolation
verification_properties:
  - VP-148  # VP-PLUGIN-003: DTU parity — verified GREEN by PLUGIN-MIGRATION-001-D (PR #153);
            # this story's deletion is gated on VP-148 being satisfied. No new VP authored here;
            # the existing parity tests continue to exercise the plugin-driven path post-deletion.
depends_on:
  - S-PLUGIN-PREREQ-A  # SensorId newtype: no SensorType closed enum in deleted code
  - S-PLUGIN-PREREQ-B  # PipelineExecutor + AuthProvider: replacement path depends on this
  - S-PLUGIN-PREREQ-C  # TOML grammar: 4 bundled specs must load correctly post-deletion
  - S-PLUGIN-PREREQ-E  # SensorAuth open trait: `auth_type_name()` is on the open trait
  - PLUGIN-MIGRATION-001-D  # INV-PARITY-001: 001-A MUST NOT delete until VP-PLUGIN-003 GREEN
                             # (PR #153 merged; gate satisfied 2026-05-22)
  - PLUGIN-MIGRATION-001-E  # [PARTIAL GATE] CrowdStrike .prx WASM plugin — required before
                             # CrowdStrike auth module deletion (AC-006-E-GATED). 001-E authored
                             # in parallel; this depends_on entry tracks the gated sub-step only.
                             # Scoping decision documented in §Open Questions / AC-006.
blocks:
  - PLUGIN-MIGRATION-001-B  # prism-query dispatch stories use SensorId strings from specs;
                             # must not hold adapter imports that 001-A deletes
  - PLUGIN-MIGRATION-001-C  # SpecDrivenMapper uses the 4 bundled spec schemas; clean up before
points: 3
# Points justification: Three discrete work units, each bounded:
#   - auth_type_name() corrections (3 impls): ~0.5 day (trivial string change + Red Gate amendment)
#   - delete claroty/cyberint/armis auth modules + re-exports: ~0.5 day (deletion + compile clean)
#   - init_registry_for_org replacement stub: ~1 day (remove hardcoded adapter construction,
#     wire spec-catalog dispatch or emit todo!() stub with correct signature, update callers)
#   Total: 3 points (~1.5–2 days). CrowdStrike deletion is GATED on 001-E — excluded from estimate.
estimated_days: 2
risk: MEDIUM
# Risk justification: D10 co-merge contract (ADR-028 §D10) means deploying to production
# before 001-E/001-B are ready creates an E-SPEC-012 regression window. CI/test-only risk is LOW
# (parity tests tagged ignore; no production credentials). Production deployment BLOCKED until
# co-merge contract is satisfied (ADR-028 §D10).
acceptance_criteria_count: 9
red_gate_tests: 5
estimated_passes: "2-3 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "D10 co-merge contract (ADR-028 §D10): 001-A and 001-D MUST be deployed to production
    simultaneously. 001-D is already merged (PR #153, develop@3f2de889). The deploy-gate
    is enforced by requiring both PRs to reach production simultaneously, not by feature flags."
  - "CrowdStrike deletion gate (INV-PARITY-001 extension): AC-006 is explicitly labeled
    [GATED-ON-001-E]. The implementer must NOT delete crates/prism-sensors/src/auth/crowdstrike.rs
    until PLUGIN-MIGRATION-001-E has merged. The remaining 3 deletions proceed unconditionally."
  - "Spec-catalog dispatch correctness: init_registry_for_org replacement must maintain BC-3.2.001
    OrgId-keyed isolation; any regression is caught by existing org_id_binding.rs integration tests."
inputs:
  - "crates/prism-sensors/src/auth/mod.rs"
  - "crates/prism-sensors/src/auth/cyberint.rs"
  - "crates/prism-sensors/src/auth/claroty.rs"
  - "crates/prism-sensors/src/auth/armis.rs"
  - "crates/prism-sensors/src/auth/crowdstrike.rs"
  - "crates/prism-sensors/src/lib.rs"
  - "crates/prism-bin/src/boot.rs"
  - "crates/prism-sensors/tests/org_id_binding.rs"
  - ".factory/specs/behavioral-contracts/BC-2.01.016-sensor-auth-open-trait-contract.md"
  - ".factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.012-plugin-registry-dispatch-migration.md"
  - ".factory/specs/behavioral-contracts/BC-3.2.001-per-org-sensor-data-isolation.md"
  - ".factory/specs/architecture/decisions/ADR-028-toml-spec-grounding-vs-dtu-routes.md"
  - ".factory/stories/PLUGIN-MIGRATION-001-D-author-4-production-toml-sensor-specs.md"
input-hash: null
traces_to: []
cycle: "v1.0.0-greenfield"
phase: 3
---

# PLUGIN-MIGRATION-001-A: prism-sensors — Delete 4 Named Auth Modules + Re-exports + Replace init_registry_for_org

**Story ID:** PLUGIN-MIGRATION-001-A
**Status:** ready
**Version:** v1.0
**Wave:** 1 (co-merge with PLUGIN-MIGRATION-001-D per ADR-028 §D10; 001-D merged via PR #153
develop@3f2de889)

---

## §Origin

Registered in STORY-INDEX at D-334 (2026-05-10) as Wave 1 of the PLUGIN-MIGRATION saga. Gating
condition: VP-PLUGIN-003 parity tests GREEN for all 4 sensors (INV-PARITY-001). Gate satisfied by
PLUGIN-MIGRATION-001-D PR #153, merged 2026-05-22T09:05:47Z.

Dispatch context: ADR-028 v1.10 §D6 (scope expansion per user Path-A adjudication D-747) and
§D10 (co-merge contract adjudicated in FB-IMPL-1 D-FB-IMPL-1-MED-005). This story's scope was
expanded beyond the original deletion-only scope to include auth_type_name() corrections and
Red Gate test amendment.

---

## Story-Level Goal

At merge, three hardcoded auth modules (`claroty`, `cyberint`, `armis`) are deleted from
`crates/prism-sensors/src/auth/`, their re-exports removed from `auth/mod.rs` and `lib.rs`,
and `init_registry_for_org` in `crates/prism-sensors/src/lib.rs` is rewritten to dispatch
via spec-catalog rather than constructing hardcoded adapter instances. The auth_type_name()
return values for the three deleted impls are corrected to DTU-grounded values in the same
burst — immediately before deletion. The Red Gate test `test_BC_2_01_016_003_four_auth_impls_minimal_diff_post_unsealing`
is amended to assert the corrected values. CrowdStrike auth module deletion is gated on
PLUGIN-MIGRATION-001-E merging first (AC-006 [GATED-ON-001-E]).

---

## Narrative

As the Prism platform, I want the four hardcoded Rust auth modules deleted (three immediately, one
gated on 001-E), `init_registry_for_org` replaced with spec-catalog dispatch, and the three
ADR-028 §D2 auth_type_name() label bugs corrected before deletion, so that the prism-sensors
crate no longer contains hardcoded adapter logic and sensors run exclusively from TOML specs
through the plugin runtime as mandated by ADR-023.

---

## Functional Summary

1. **Rewrite `auth_type_name()` for Cyberint, Claroty, Armis** per ADR-028 §D6 (immediately
   before deletion, so the correction is observable in CI and the Red Gate test passes GREEN
   against the new values one commit before the deletion commit):
   - `CyberintAuth::auth_type_name()` → `"cookie_roundtrip"` (corrected from `"bearer_static"`)
   - `ClarotyAuth::auth_type_name()` → `"bearer_static"` (corrected from `"cookie_roundtrip"`)
   - `ArmisAuth::auth_type_name()` → `"bearer_static"` (corrected from `"api_key"`)
   - `CrowdStrikeAuth::auth_type_name()` → unchanged at `"oauth2_client_credentials"`

2. **Amend Red Gate test** `test_BC_2_01_016_003_four_auth_impls_minimal_diff_post_unsealing`
   in `crates/prism-sensors/src/auth/mod.rs` to assert the corrected DTU-grounded values.

3. **Delete three auth modules**: `claroty.rs`, `cyberint.rs`, `armis.rs` from
   `crates/prism-sensors/src/auth/`.

4. **Remove re-exports and `pub mod` declarations** from `auth/mod.rs` and `lib.rs` for the
   three deleted modules. Verify compile-clean.

5. **Rewrite `init_registry_for_org`** in `crates/prism-sensors/src/lib.rs`:
   - Remove the four hardcoded adapter construction calls
   (`CrowdStrikeAdapter::new`, `CyberintAdapter::new`, `ClarotyAdapter::new`, `ArmisAdapter::new`)
   - Wire spec-catalog dispatch (BC-2.16.012): adapter registry is populated from the loaded
   `SensorSpec` catalog at boot time rather than by hardcoded construction per sensor
   - Preserve the `OrgId`-keyed composite dispatch invariant (BC-3.2.001 precondition 4)
   - Update all call sites (`prism-bin/src/boot.rs` comments/stubs and any other callers)

6. **CrowdStrike auth module deletion** [GATED-ON-001-E]: `crowdstrike.rs` is NOT deleted
   in this story. See AC-006 and §Open Questions for the scoping decision.

---

## Behavioral Contracts

| BC ID | Version | Title | Subsystem | Role in This Story |
|-------|---------|-------|-----------|-------------------|
| BC-2.01.016 | 1.11 | SensorAuth Open Trait — Plugin-Implementable Auth Contract (No Sealed Marker) | SS-01 | **Primary** — auth_type_name() returns for three impls corrected per §D2/§D6; Red Gate test amended; INV-AUTH-OPEN-002 satisfied post-amendment |
| BC-2.01.013 | 1.6 | DataSource Trait — Spec-Driven Adapter Pattern | SS-01 | **Completion** — deleting hardcoded Rust adapter modules is the final act of the spec-driven migration ADR-023 mandates for these three sensors |
| BC-2.16.012 | 1.31 | PluginRegistry Dispatch in spec_parser.rs | SS-16 | **Required** — init_registry_for_org replacement uses spec-catalog open dispatch (INV-SPEC-PARSER-OPEN-001); no new hardcoded sensor name match arms |
| BC-3.2.001 | 0.6 | Per-Org Sensor Data Isolation via Composite HashMap Key | SS-01 | **Anti-regression** — init_registry_for_org replacement must preserve OrgId-keyed composite dispatch (precondition 4); org_id_binding.rs integration tests are the gate |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~6,000 |
| BC files (4 BCs, read in full) | ~16,000 |
| ADR-028 v1.10 (§D2/§D6/§D10 sections) | ~3,000 |
| auth module sources (4 files, read in full) | ~10,000 |
| prism-sensors/src/lib.rs (init_registry_for_org site) | ~2,000 |
| prism-bin/src/boot.rs (caller context, partial read) | ~3,000 |
| crates/prism-sensors/tests/org_id_binding.rs | ~3,000 |
| BC-2.16.013 v1.16 (parity gate context) | ~5,000 |
| **Total estimate** | **~48,000** |
| Agent context window (claude-sonnet-4-6) | ~200,000 |
| **% of context window** | **~24%** |

Within the 20–30% target. All source files are readable in full without context pressure.

---

## Acceptance Criteria

### AC-001: `auth_type_name()` corrections applied (traces to BC-2.01.016 postcondition; ADR-028 §D6)

Before deletion (same commit or the commit immediately preceding deletion):

| Impl | Current (LIVE / ADR-026 §D3) | Corrected (ADR-028 §D2/§D6) |
|------|------------------------------|------------------------------|
| `CyberintAuth::auth_type_name()` | `"bearer_static"` | `"cookie_roundtrip"` |
| `ClarotyAuth::auth_type_name()` → | `"cookie_roundtrip"` | `"bearer_static"` |
| `ArmisAuth::auth_type_name()` | `"api_key"` | `"bearer_static"` |
| `CrowdStrikeAuth::auth_type_name()` | `"oauth2_client_credentials"` | unchanged |

Each `auth_type_name()` impl body is a single `&'static str` return per ADR-026 §D1.
No other changes to these impl blocks.

(traces to BC-2.01.016 postcondition — four impls return correct `auth_type_name()` discriminators;
INV-AUTH-OPEN-002 satisfied for the three corrected impls; ADR-028 §D6 Action 1)

### AC-002: Red Gate test amended to assert DTU-grounded values (traces to BC-2.01.016 invariant INV-AUTH-OPEN-002; ADR-028 §D6 Action 2)

`test_BC_2_01_016_003_four_auth_impls_minimal_diff_post_unsealing` in
`crates/prism-sensors/src/auth/mod.rs` is amended:

| Impl | Old assertion | New assertion |
|------|--------------|---------------|
| `CyberintAuth` | `"bearer_static"` | `"cookie_roundtrip"` |
| `ClarotyAuth` | `"cookie_roundtrip"` | `"bearer_static"` |
| `ArmisAuth` | `"api_key"` | `"bearer_static"` |
| `CrowdStrikeAuth` | `"oauth2_client_credentials"` | unchanged |

Test passes GREEN at `cargo nextest run -p prism-sensors -E 'test(test_BC_2_01_016_003)'`
before the deletion commits.

(traces to BC-2.01.016 invariant INV-AUTH-OPEN-002 — four impls minimal diff post-unsealing;
ADR-028 §D6 Action 2)

### AC-003: Three auth modules deleted; compile-clean (traces to BC-2.01.013 postcondition — spec-driven migration complete for Claroty/Cyberint/Armis)

The following files are deleted:
- `crates/prism-sensors/src/auth/claroty.rs`
- `crates/prism-sensors/src/auth/cyberint.rs`
- `crates/prism-sensors/src/auth/armis.rs`

The following declarations are removed from `crates/prism-sensors/src/auth/mod.rs`:
- `pub mod claroty;`
- `pub mod cyberint;`
- `pub mod armis;`
- `pub use claroty::ClarotyAuth;`
- `pub use cyberint::CyberintAuth;`
- `pub use armis::ArmisAuth;`
- The `//!` module-level doc-comment entries for `ClarotyAuth`, `CyberintAuth`, `ArmisAuth`
  (lines referencing the deleted impls)

The following re-exports are removed from `crates/prism-sensors/src/lib.rs`:
- `pub use auth::armis::ArmisAdapter;`
- `pub use auth::claroty::{ClarotyAdapter, ClarotyId};`
- `pub use auth::cyberint::CyberintAdapter;`
- `pub use auth::{ArmisAuth, ClarotyAuth, CyberintAuth, SensorAuth};`
  (replaced with `pub use auth::{CrowdStrikeAuth, SensorAuth};` — only CrowdStrike remains)
- `pub use pagination::{paginate_claroty, OffsetCursor};` — if this re-export is in lib.rs
  (implementer must verify; if `paginate_claroty` originates from a deleted module, remove it)
- Any other `pub use` of symbols originating exclusively from the three deleted modules

`cargo build -p prism-sensors` passes with zero errors and zero unexpected warnings after deletion.

(traces to BC-2.01.013 postcondition — hardcoded adapter code for Claroty/Cyberint/Armis deleted;
spec-driven path is the sole path for these sensors)

### AC-004: `init_registry_for_org` replaced with spec-catalog dispatch (traces to BC-2.16.012 postcondition; BC-3.2.001 precondition 4)

`crates/prism-sensors/src/lib.rs::init_registry_for_org` is rewritten:
- Removes the four hardcoded adapter construction calls
  (`CrowdStrikeAdapter::new`, `CyberintAdapter::new`, `ClarotyAdapter::new`, `ArmisAdapter::new`)
  for the three deleted adapters. CrowdStrikeAdapter construction may remain in a reduced form
  pending AC-006 gating (see §Open Questions and implementer note below).
- Populates `AdapterRegistry` from the loaded `SensorSpec` catalog via `PluginRegistry`
  open dispatch (BC-2.16.012 INV-SPEC-PARSER-OPEN-001) — no hardcoded sensor name match arms.
- Preserves the `org_id: OrgId` parameter and uses it as the composite key for all registry
  entries (BC-3.2.001 precondition 4). The `OrgId(A)` → `OrgId(B)` isolation invariant is
  not weakened by the replacement.
- The function signature remains: `pub fn init_registry_for_org(org_id: OrgId, ...) -> AdapterRegistry`
  where the credential parameters for deleted adapters are removed; implementer MUST update
  all call sites accordingly.

**Note on interim CrowdStrike handling (pending AC-006):** While `crowdstrike.rs` is still
present (not yet deleted per AC-006 gate), `CrowdStrikeAdapter::new` construction in
`init_registry_for_org` MUST still be removed as part of the spec-catalog dispatch rewrite.
The spec-catalog dispatch path handles CrowdStrike via the bundled `crowdstrike.sensor.toml`
spec. If spec-catalog dispatch is not yet fully wired, the implementer may emit a justified
`todo!("PLUGIN-MIGRATION-001-A: CrowdStrike adapter registration pending spec-catalog dispatch wiring")` 
for the CrowdStrike registration path only — but MUST document this as a non-blocking gap in
§Known Gaps and NOT as a tech-debt-register entry (per Canonical Principle Rule 3).

`cargo test -p prism-sensors -- tests::org_id_binding` (full `org_id_binding.rs` suite) passes
GREEN after the replacement.

(traces to BC-2.16.012 postcondition — open dispatch; BC-3.2.001 precondition 4 — OrgId-keyed composite key preserved)

### AC-005: Call sites in prism-bin updated (traces to BC-2.16.012 postcondition — no orphan adapter imports)

`crates/prism-bin/src/boot.rs` references to `init_registry_for_org` are updated to match the
new function signature (removed credential parameters for deleted adapters). The existing
`step7_init_storage` and `step8_init_query_engine` stubs that reference `init_registry_for_org`
in doc-comments are updated to reflect the new dispatch model.

`cargo build -p prism-bin` passes with zero errors after the update.

(traces to BC-2.16.012 postcondition — no call site retains imports from deleted modules)

### AC-006 [GATED-ON-001-E]: CrowdStrike auth module deleted (traces to BC-2.01.013 postcondition — spec-driven migration complete for CrowdStrike)

**This AC is GATED. `crowdstrike.rs` MUST NOT be deleted until PLUGIN-MIGRATION-001-E has merged.**

When PLUGIN-MIGRATION-001-E merges (CrowdStrike OAuth2 WASM `.prx` plugin):
- `crates/prism-sensors/src/auth/crowdstrike.rs` is deleted
- `pub mod crowdstrike;` and `pub use crowdstrike::CrowdStrikeAuth;` are removed from `auth/mod.rs`
- `pub use auth::crowdstrike::CrowdStrikeAdapter;` and `pub use auth::{CrowdStrikeAuth, SensorAuth};`
  (reduced to `pub use auth::SensorAuth;`) are removed from `lib.rs`
- The `#[non_exhaustive]` compile-fail perimeter gate at `tests/external/non_exhaustive_violation/`
  is updated if it references `CrowdStrikeAuth`
- `cargo build -p prism-sensors` passes compile-clean after this final deletion

This AC may be delivered as a follow-on commit to this story's branch (before merge), or as a
separate micro-story dispatched immediately after 001-E merges. See §Open Questions for the
implementer guidance on the recommended delivery pattern.

(traces to BC-2.01.013 postcondition — hardcoded CrowdStrike adapter deleted; spec-driven path is sole path)

### AC-007: No orphan re-exports or dead imports remain (traces to BC-2.01.013 invariant — no hardcoded adapter code survives in module surface)

After all deletions (ACs 003–005 and, when gated, AC-006):

```
grep -rn "CyberintAuth\|ClarotyAuth\|ArmisAuth\|CyberintAdapter\|ClarotyAdapter\|ArmisAdapter\|paginate_claroty" \
  crates/prism-sensors/src/ crates/prism-bin/src/
```

returns ZERO matches in production source files (excluding `#[cfg(test)]` blocks that may
retain test fixtures; implementer must confirm test-only vs production scope). Any match outside
`#[cfg(test)]` is a blocking defect.

(traces to BC-2.01.013 invariant — spec-driven-only sensor surface; no hardcoded adapter symbols exposed)

### AC-008: Workspace-wide `just check` GREEN (traces to BC-2.16.012 invariant INV-SPEC-PARSER-OPEN-001 — no compile regression)

`just check` (fmt + clippy + nextest + doctests + crate-layout) passes workspace-wide with all
pre-existing tests green. No tests that were passing before this story are made to fail.

The existing DTU parity tests under `crates/prism-spec-engine/tests/parity/` (authored in
PLUGIN-MIGRATION-001-D) continue to pass or remain `#[ignore]`-tagged per their pre-existing DTU
availability status — 001-A does not change their ignore-tag policy.

(traces to BC-2.16.012 invariant INV-SPEC-PARSER-OPEN-001 — workspace compile-clean; open dispatch path regression-free)

### AC-009: ADR-028 §D2 correctness observable in `auth/mod.rs` doc-comment (traces to BC-2.01.016 invariant INV-AUTH-OPEN-002)

The `//!` module-level doc-comment in `crates/prism-sensors/src/auth/mod.rs` is updated to
reflect the corrected auth_type_name() values for the surviving impls (CrowdStrike at minimum;
Claroty/Cyberint/Armis entries removed since those modules are deleted). No stale doc-comment
entry asserts a pre-ADR-028-§D2 value.

(traces to BC-2.01.016 invariant INV-AUTH-OPEN-002 — minimal diff post-unsealing; doc-comment coherence per POL-29)

---

## Tasks

Implementer: follow strict TDD discipline — amend the Red Gate test first (Task 2), make it
fail RED for the right reason, then apply the `auth_type_name()` fixes (Task 3) to drive GREEN.
Then proceed with deletion tasks.

### Task 1: Read source files and BCs before writing any code

Read in full:
- `crates/prism-sensors/src/auth/{mod.rs,cyberint.rs,claroty.rs,armis.rs,crowdstrike.rs}`
- `crates/prism-sensors/src/lib.rs` (lines containing `init_registry_for_org` and function body)
- `crates/prism-sensors/tests/org_id_binding.rs`
- `crates/prism-bin/src/boot.rs` (lines containing `init_registry_for_org` references)
- BC-2.01.016 v1.11, BC-2.01.013, BC-2.16.012 v1.31, BC-3.2.001 v0.6
- ADR-028 v1.10 §D2, §D6, §D10

Confirm the exact current `auth_type_name()` return values match the table in AC-001.

### Task 2: Amend Red Gate test to assert DTU-grounded values (RED first)

In `crates/prism-sensors/src/auth/mod.rs::test_BC_2_01_016_003_four_auth_impls_minimal_diff_post_unsealing`:

Change the three assertions per AC-002. Run:
```
cargo nextest run -p prism-sensors -E 'test(test_BC_2_01_016_003)'
```
Confirm test fails RED (wrong assertion fails against current `auth_type_name()` values).

### Task 3: Rewrite `auth_type_name()` for Cyberint, Claroty, Armis (GREEN)

Apply the three string changes per AC-001:
- `crates/prism-sensors/src/auth/cyberint.rs`: `"bearer_static"` → `"cookie_roundtrip"`
- `crates/prism-sensors/src/auth/claroty.rs`: `"cookie_roundtrip"` → `"bearer_static"`
- `crates/prism-sensors/src/auth/armis.rs`: `"api_key"` → `"bearer_static"`

Run Red Gate test again — must pass GREEN.

Run full crate:
```
just iter prism-sensors
```
Confirm all pre-existing tests still pass.

### Task 4: Delete three auth module source files

```
rm crates/prism-sensors/src/auth/claroty.rs
rm crates/prism-sensors/src/auth/cyberint.rs
rm crates/prism-sensors/src/auth/armis.rs
```

Remove `pub mod` and `pub use` declarations per AC-003. Update `//!` module doc-comment.

```
cargo build -p prism-sensors 2>&1 | head -50
```
Resolve all compile errors (missing symbols, orphan imports). Do NOT re-add any deleted symbol;
find callers and migrate them to the spec-catalog path.

### Task 5: Remove re-exports from `crates/prism-sensors/src/lib.rs`

Per AC-003 re-export removal list. Verify each removed symbol is not used elsewhere in lib.rs
or in crates/prism-bin/.

Run:
```
cargo build -p prism-sensors -p prism-bin 2>&1 | head -50
```
Resolve compile errors.

### Task 6: Rewrite `init_registry_for_org` per AC-004

Replace the hardcoded adapter construction block with spec-catalog dispatch. Preserve `org_id`
parameter and composite-key invariant (BC-3.2.001). Update function signature (remove credential
parameters for deleted adapters). Update call sites in `prism-bin/src/boot.rs` per AC-005.

Run:
```
just iter prism-sensors
```
Confirm `org_id_binding.rs` test suite passes GREEN.

### Task 7: Final workspace gate

```
just check
```
Must pass GREEN. Resolve any clippy warnings. Confirm no test regressions.

### Task 8: Verify no orphan symbols per AC-007

Run the grep command from AC-007. Zero matches in production source. Document any test-only
matches as expected (note: they're acceptable in `#[cfg(test)]`).

### Task 9: Mark AC-006 status in story frontmatter and §Known Gaps

Do NOT delete `crowdstrike.rs`. Update `§Known Gaps` to record that AC-006 is pending 001-E
merge. Do not modify STORY-INDEX.md or STATE.md.

---

## Previous Story Intelligence

PLUGIN-MIGRATION-001-D (direct predecessor, merged PR #153 develop@3f2de889):

- The four bundled TOML sensor specs are now at `crates/prism-sensors/specs/`. Their auth_type
  values are already DTU-grounded (ADR-028 §D2 compliant). No TOML files are modified in 001-A.
- The parity tests under `crates/prism-spec-engine/tests/parity/` are live. They were authored
  to verify the spec-driven path; they continue to run after 001-A deletes the legacy adapters.
- The SpecErrorCode::ESpec017 variant was added in 001-D scope (D-737 Decision 3). 001-A does
  not modify the error taxonomy.
- The ADR-028 §D10 co-merge contract was established: 001-A and 001-D must deploy to production
  simultaneously. 001-D is already merged; this story is the other leg of the co-deploy.
- ADR-028 §D2 documents the latent `auth_type_name()` label bugs in the adapters being deleted.
  The correction must happen in the same burst as the deletion (or the commit immediately before).
- ADR-028 §D4 forbids citing `crates/prism-sensors/src/auth/{sensor}.rs` symbols as ground-truth
  for URL paths or auth flows in any future spec artifact. After this story merges, that prohibition
  is permanent for the three deleted files.

---

## Architecture Compliance Rules

Extracted from `architecture/module-decomposition.md`, `ADR-023`, `ADR-028`, `ADR-026`:

| Rule | Source | Enforcement |
|------|--------|-------------|
| No hardcoded sensor name match arms in spec_parser.rs or init_registry_for_org replacement | ADR-023 Rule 2 / BC-2.16.012 INV-SPEC-PARSER-OPEN-001 | Adversary grep check |
| `auth_type_name()` returns must match DTU-enforced behavior for each sensor | ADR-028 §D2 | AC-001/AC-002 Red Gate test |
| OrgId-keyed composite dispatch preserved in init_registry_for_org replacement | BC-3.2.001 precondition 4 | org_id_binding.rs integration test |
| CrowdStrikeAuth `"oauth2_client_credentials"` unchanged | ADR-028 §D6 Action 1 (CrowdStrike unchanged) | AC-001 assertion |
| `crowdstrike.rs` MUST NOT be deleted before PLUGIN-MIGRATION-001-E merges | ADR-028 §D10 / INV-PARITY-001 extension | AC-006 gate label |
| No `unwrap()` / `expect()` in production code added by this story | CLAUDE.md §Conventions | Clippy + adversary |
| No `println!` in production code | CLAUDE.md §Conventions | Clippy `--deny warnings` |
| Credentials MUST NOT appear in `Debug` output | AD-017 | Existing impls already comply; deletion is the fix |

### Forbidden Dependencies

`prism-sensors` MUST NOT gain any new dependency on `prism-spec-engine` through this story.
The existing `prism-sensors → prism-spec-engine` dependency prohibition (from BC-2.16.013 ADR-028
§D3: "Parity tests MUST NOT require a `prism-sensors` dev-dep on `prism-spec-engine` at test
runtime") remains in force. If spec-catalog dispatch wiring requires cross-crate access, the
wiring belongs in `prism-bin` (as a boot-time orchestration concern), not in `prism-sensors`.

---

## Library and Framework Requirements

| Library | Version | Pin Source |
|---------|---------|------------|
| `secrecy` | per workspace `Cargo.toml` | workspace pin |
| `reqwest` | per workspace `Cargo.toml` | workspace pin; note production timeout policy: `.timeout(Duration::from_secs(30))` required (TD-S-PLUGIN-PREREQ-B-005 P2) |
| `tokio` | per workspace `Cargo.toml` | workspace pin |
| `arrow` | per workspace `Cargo.toml` | workspace pin |

Do NOT introduce new crate dependencies in `prism-sensors/Cargo.toml` for this story. The deletion
direction means net-zero or fewer dependencies, not more. If spec-catalog dispatch requires a new
import from a sibling crate already in the workspace, verify the dep is already declared before adding.

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-sensors/src/auth/claroty.rs` | DELETE | After auth_type_name() correction (Task 3) |
| `crates/prism-sensors/src/auth/cyberint.rs` | DELETE | After auth_type_name() correction (Task 3) |
| `crates/prism-sensors/src/auth/armis.rs` | DELETE | After auth_type_name() correction (Task 3) |
| `crates/prism-sensors/src/auth/crowdstrike.rs` | RETAIN (AC-006 GATED) | Delete only after 001-E merges |
| `crates/prism-sensors/src/auth/mod.rs` | MODIFY | Remove pub mod/pub use for deleted modules; amend Red Gate test; update //! doc-comment |
| `crates/prism-sensors/src/lib.rs` | MODIFY | Remove re-exports; rewrite init_registry_for_org |
| `crates/prism-bin/src/boot.rs` | MODIFY (minor) | Update call-site signatures/comments for init_registry_for_org |
| `crates/prism-sensors/specs/` | NO CHANGE | 4 bundled TOML spec files from 001-D; not touched |
| `crates/prism-spec-engine/tests/parity/` | NO CHANGE | 001-D parity tests; not touched |
| `tests/external/non_exhaustive_violation/` | REVIEW ONLY | Check if any entries reference deleted types; remove if so |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Caller retains a reference to `ClarotyAuth`, `CyberintAuth`, or `ArmisAuth` after deletion | Compile error at call site; implementer MUST sweep all callers in the same burst (TD-VSDD-060 sibling-site sweep) |
| EC-002 | `paginate_claroty` is re-exported from lib.rs and imported in prism-bin | Remove the re-export; verify prism-bin does not import it (if it does, migrate prism-bin to spec-catalog path) |
| EC-003 | `org_id_binding.rs` integration tests construct deleted adapter types directly | Those tests become compile errors; implementer rewrites the relevant test sections to use spec-catalog dispatch path or removes the test if the behavior is now covered by parity tests |
| EC-004 | `init_registry_for_org` is called with the old signature from an integration test | Update the test call sites; do not retain the old signature as a compatibility shim |
| EC-005 | CrowdStrikeAdapter construction remains in a `todo!()` stub after spec-catalog dispatch rewrite | Acceptable as a documented gap (§Known Gaps AC-004 note); but the STUB must carry the correct future-story anchor, not an open-ended TODO |
| EC-006 | Non-exhaustive compile-fail gate in `tests/external/non_exhaustive_violation/` references deleted types | Remove those entries; the gate is for live pub-API surface types only |

---

## §Open Questions

### Q1: AC-006 — Delivery pattern for CrowdStrike deletion (DECIDED: two-AC same-branch)

**Decision recorded here (story-writer call per dispatch instructions):**

AC-006 is scoped INTO this story as a gated AC — not split to a separate story. Rationale:
1. The deletion is mechanically identical to AC-003 (delete a file, remove re-exports, verify
   compile-clean). Splitting to a separate story adds process overhead with no engineering benefit.
2. The 001-E gate is short-lived (001-E is being authored in parallel on 2026-05-22). The
   implementer can hold the 001-A branch open until 001-E merges, then execute AC-006 as an
   additional commit before raising the 001-A PR. This is within normal story branch lifecycle.
3. Story points (3) reflect the 001-E-gated work as bounded, not open-ended.

**Implementer guidance:** Implement ACs 001–005 and 007–009 first. After PLUGIN-MIGRATION-001-E
merges to develop, rebase 001-A branch and apply AC-006 (delete crowdstrike.rs, remove re-exports,
verify compile-clean). Then raise the PR for 001-A. Do NOT raise the PR before AC-006 is complete.

### Q2: `init_registry_for_org` spec-catalog dispatch — implementation detail

If spec-catalog dispatch wiring is not yet complete (i.e., the boot sequence that calls
`init_registry_for_org` uses `todo!()` stubs in `step7_init_storage`), the implementer should:
- Remove the hardcoded adapter construction block from `init_registry_for_org`
- Wire the `SpecCatalog` / `PluginRegistry` path as far as the existing infrastructure permits
- If a full wire-up would require changes to `prism-spec-engine` (violating the Forbidden
  Dependencies rule), emit a scoped `todo!()` with the exact future-story anchor
  (`S-WAVE5-PREP-01`/`S-3.02-FOLLOWUP-RUNTIME`) per the existing boot.rs pattern

The function body should NOT retain any hardcoded `ClarotyAdapter::new`, `CyberintAdapter::new`,
or `ArmisAdapter::new` calls after this story merges. Those calls reference deleted types.

---

## §Known Gaps

| Gap ID | Scope | Description | Resolution Target |
|--------|-------|-------------|-------------------|
| GAP-001-A | L3 | AC-006 CrowdStrike auth module deletion gated on PLUGIN-MIGRATION-001-E merge | PLUGIN-MIGRATION-001-E (authored in parallel 2026-05-22; same Wave 1) |
| GAP-002-A | L3 | `CrowdStrikeAdapter::new` construction in init_registry_for_org may persist as a scoped `todo!()` if spec-catalog dispatch wiring is incomplete at boot.rs step7 | S-WAVE5-PREP-01 / S-3.02-FOLLOWUP-RUNTIME (existing boot stubs already carry this anchor) |

---

## Dependencies

### Satisfied (ALL MERGED)

| Dependency | PR | SHA | Notes |
|------------|-----|-----|-------|
| S-PLUGIN-PREREQ-A | #142 | MERGED | SensorId newtype |
| S-PLUGIN-PREREQ-B | #143 | MERGED | PipelineExecutor |
| S-PLUGIN-PREREQ-C | #144 | MERGED | TOML grammar |
| S-PLUGIN-PREREQ-E | #151 | MERGED develop@80ebe794 | SensorAuth open trait |
| PLUGIN-MIGRATION-001-D | #153 | MERGED develop@3f2de889 | VP-PLUGIN-003 GREEN (parity gate) |

### Pending (PARALLEL AUTHORING)

| Dependency | Status | Gate Applied To |
|------------|--------|----------------|
| PLUGIN-MIGRATION-001-E | PLANNED (authored in parallel 2026-05-22) | AC-006 CrowdStrike deletion only |

**Dependency anchor justifications:**
- `depends_on: PLUGIN-MIGRATION-001-D` — because INV-PARITY-001 requires VP-PLUGIN-003 GREEN before
  any adapter deletion; without 001-D's parity tests, there is no proof that the plugin-driven
  path is behaviorally equivalent to the deleted adapters.
- `depends_on: PLUGIN-MIGRATION-001-E` (partial) — because CrowdStrike OAuth2 WASM plugin
  (`crowdstrike.prx`) must be the registered replacement before `crowdstrike.rs` is deleted;
  without 001-E, CrowdStrike auth has no functional replacement.
- `blocks: PLUGIN-MIGRATION-001-B` — because 001-B's prism-query dispatch stories must not retain
  imports from adapter modules this story deletes.
- `blocks: PLUGIN-MIGRATION-001-C` — because 001-C's SpecDrivenMapper presupposes clean sensor
  module boundaries established here.

---

## §Source Citations

| Artifact | Version / SHA | Authoritative Symbols |
|----------|-------------|----------------------|
| `crates/prism-sensors/src/auth/mod.rs` | develop@3f2de889 | `SensorAuth` trait; `test_BC_2_01_016_003_four_auth_impls_minimal_diff_post_unsealing` |
| `crates/prism-sensors/src/auth/cyberint.rs` | develop@3f2de889 | `CyberintAuth::auth_type_name()` → current `"bearer_static"` |
| `crates/prism-sensors/src/auth/claroty.rs` | develop@3f2de889 | `ClarotyAuth::auth_type_name()` → current `"cookie_roundtrip"` |
| `crates/prism-sensors/src/auth/armis.rs` | develop@3f2de889 | `ArmisAuth::auth_type_name()` → current `"api_key"` |
| `crates/prism-sensors/src/lib.rs` | develop@3f2de889 | `init_registry_for_org` (line 166) |
| `crates/prism-bin/src/boot.rs` | develop@3f2de889 | `step7_init_storage` / `step8_init_query_engine` stubs |
| `crates/prism-sensors/tests/org_id_binding.rs` | develop@3f2de889 | `test_AC_001_init_registry_for_org_uses_org_id_in_signature` |
| ADR-028 | v1.10 (2026-05-21) | §D2 auth_type grounding rule; §D6 scope expansion; §D10 co-merge contract |
| BC-2.01.016 | v1.11 (2026-05-22) | postconditions; INV-AUTH-OPEN-002 |
| BC-2.01.013 | v1.6 | postconditions (spec-driven migration completion) |
| BC-2.16.012 | v1.31 (2026-05-22) | postconditions; INV-SPEC-PARSER-OPEN-001 |
| BC-3.2.001 | v0.6 | precondition 4 (OrgId composite key) |

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `auth_type_name()` corrections | `crates/prism-sensors/src/auth/{cyberint,claroty,armis}.rs` | Pure (string constant return) |
| Red Gate test amendment | `crates/prism-sensors/src/auth/mod.rs` | Pure (test-only) |
| Module deletion | `crates/prism-sensors/src/auth/` | Structural (deletion) |
| `init_registry_for_org` rewrite | `crates/prism-sensors/src/lib.rs` | Effectful (registry construction) |
| Call-site update | `crates/prism-bin/src/boot.rs` | Effectful (boot orchestration) |
