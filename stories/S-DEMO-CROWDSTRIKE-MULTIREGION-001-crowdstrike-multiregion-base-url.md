---
document_type: story
story_id: S-DEMO-CROWDSTRIKE-MULTIREGION-001
title: "crowdstrike.sensor.toml: Multi-Region base_url Fidelity — Replace Hardcoded us-1 URL with ${env.CROWDSTRIKE_BASE_URL}; Structured Error on Missing Env Var; Region Runbook (ADR-031 §D8-c)"
wave: 5
epic_id: E-DTU-FIDELITY
priority: P2
status: ready
version: "1.5"
level: "L4"
producer: story-writer
timestamp: "2026-05-31T00:00:00Z"
tdd_mode: strict
subsystems: [SS-01, SS-16]
# Subsystem anchor justifications:
#   SS-01 (Sensor Adapters) owns prism-sensors (the TOML spec files) per ARCH-INDEX
#     Subsystem Registry v2.105 row; crowdstrike.sensor.toml is an SS-01 artifact.
#   SS-16 (Spec Engine) owns prism-spec-engine including SensorSpec loading, env-var
#     interpolation (${env.VAR}), and E-SPEC error emission on missing env vars; the
#     behavior change (structured error vs. panic on missing CROWDSTRIKE_BASE_URL) is
#     SS-16 scope if not already handled by the existing interpolation engine.
#   SS-17 (WASM Plugin Runtime) is NOT anchored — no WASM plugin changes.
#   No DTU code change required per ADR-031 §D8-c: "the DTU itself is already
#     region-agnostic — it binds to 127.0.0.1:0 and accepts any valid OAuth2 + Bearer
#     flow regardless of base URL. No DTU code change required."
crates_touched: [prism-sensors, prism-spec-engine]
target_module: prism-sensors
capabilities: [CAP-001]
behavioral_contracts: [BC-2.16.009, BC-2.16.013]
# BC adjudication (2026-05-31 product-owner burst):
# BC-2.16.009 (Spec File Validation — §Validation Rules 6 env-var resolver): ATTACHED.
#   Sensor-agnostic ${env.VAR_NAME} resolver covers ${env.CROWDSTRIKE_BASE_URL} in base_url.
#   E-SPEC-024 error path (missing/empty var → fail-closed) is fully specified. Edge cases
#   EC-009-008 (eu-1 URL happy path) and EC-009-009 (CROWDSTRIKE_BASE_URL unset → E-SPEC-024)
#   added to BC-2.16.009 v1.7 in the same burst. No new BC needed — contract is sensor-agnostic.
# BC-2.16.013 (Bundled Sensor Spec Authoring and DTU-Parity): ATTACHED.
#   §Postconditions §1 CrowdStrike description updated v1.18→v1.19 to reflect
#   base_url = "${env.CROWDSTRIKE_BASE_URL}" (replacing stale hardcoded us-1 pattern).
#   (Historical authoring record; BC-2.16.013 now at v1.22, established D-946.)
#   Story-writer must update body BC table and AC traces per bc_array_changes_propagate_to_body_and_acs.
verification_properties: []
depends_on: [S-SPEC-ENV-VAR-001]
# depends_on justification: HARD gate per D-914.
#   S-SPEC-ENV-VAR-001 must merge before CrowdStrike multi-region can be dispatched.
#   The ${env.CROWDSTRIKE_BASE_URL} token in crowdstrike.sensor.toml requires the
#   env-var resolver to be in place (BC-2.16.009 AC-6, E-SPEC-024 error path).
#   Without the resolver, the literal "${env.CROWDSTRIKE_BASE_URL}" string is
#   transmitted to the DTU instead of the resolved URL — runtime failure.
blocks: []
points: 2
# Points justification:
#   TOML change (crowdstrike.sensor.toml):
#   - base_url = "${env.CROWDSTRIKE_BASE_URL}" (1 line change): ~0.25 pts
#   - TOML comment: document 4 region URLs: ~0.25 pts
#   Verification:
#   - Confirm spec engine already handles missing ${env.VAR} gracefully for Armis/Claroty: ~0.5 pts
#   - If NOT handled: implement E-SPEC structured error in spec engine (SS-16): ~1 pt
#   - Spec-load test with eu-1 URL env var set: ~0.5 pts
#   - DTU demo test (env points to local DTU): ~0.25 pts
#   Total: 2 points (~0.5-1 day, depending on whether env-var error handling already exists)
estimated_days: 1
risk: LOW
# Risk justification:
#   The env-var interpolation pattern (${env.VAR}) already exists for Armis and Claroty.
#   The TOML change is a 1-line edit. The main risk is discovering that the spec engine
#   does NOT gracefully handle missing ${env.VAR} — in that case, E-SPEC error handling
#   must be added (in-scope per production-grade default: "if a thing is worth doing in v1,
#   it is worth doing correctly in v1"). This is production-grade work, not a deferral.
acceptance_criteria_count: 6
red_gate_tests: 3
estimated_passes: "1-2 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "Before implementing, verify that the spec engine's ${env.VAR} interpolation already
    emits a structured E-SPEC error (not panic) when the env var is absent. Run a test:
    load armis.sensor.toml with ARMIS_INSTANCE_URL unset and confirm the error type.
    If it panics, implement E-SPEC error handling in-scope per production-grade default."
  - "DTU demo compatibility: the DTU binds to 127.0.0.1:0 and is region-agnostic.
    When CROWDSTRIKE_BASE_URL=http://127.0.0.1:PORT, the spec engine should resolve
    correctly and the DTU demo path must still work. Verify with AC-004."
  - "auth_plugin and auth_type are preserved: this story changes ONLY base_url.
    auth_type = 'oauth2_client_credentials' and auth_plugin = 'crowdstrike-oauth2' are
    D-747 LOCKED — do NOT change them."
inputs:
  - "crates/prism-sensors/specs/crowdstrike.sensor.toml"
  - "crates/prism-sensors/specs/armis.sensor.toml"
  - "crates/prism-sensors/specs/claroty.sensor.toml"
  - "crates/prism-spec-engine/src/spec_parser.rs"
  - ".factory/specs/architecture/decisions/ADR-031-dtu-equals-true-dtu-fidelity-principle.md"
  - ".factory/proposals/POLLER-DTU-FIDELITY-AUDIT-2026-05-29.md"
  - ".factory/specs/prd-supplements/error-taxonomy.md"
input-hash: null
traces_to: []
cycle: "v1.0.0-brownfield"
phase: 3
---

# S-DEMO-CROWDSTRIKE-MULTIREGION-001 v1.5 — CrowdStrike Multi-Region base_url Fidelity

**Story ID:** S-DEMO-CROWDSTRIKE-MULTIREGION-001
**Status:** ready
**Version:** v1.5
**Wave:** 5
**Priority:** P2
**Points:** 2

---

## Authority

ADR-031 §D8-c is the authoritative design decision for CrowdStrike multi-region base URL fidelity
(Gap-CS-003). Read it before implementing:
`.factory/specs/architecture/decisions/ADR-031-dtu-equals-true-dtu-fidelity-principle.md`.

ADR-031 §D8-c establishes the `${env.CROWDSTRIKE_BASE_URL}` substitution requirement, matching the
Armis and Claroty env-var pattern. ADR-031 §D8-c also confirms the DTU requires no code changes —
the CrowdStrike DTU is already region-agnostic (binds to `127.0.0.1:0`).

ADR-031 `status: accepted`. `superseded_by:` cites ADR-053 §D3 scope-narrowing. The §D3
supersession narrows the single-surface assumption for the Assets surface only; the §D8-c
multi-region provisions governing this story are NOT in the superseded scope.

BC-2.16.009 §Validation Rules 6 governs env-var resolver behavior including the E-SPEC-024 error
path for missing/empty vars (AC-003). BC-2.16.013 §Postconditions §1 governs the CrowdStrike spec
authoring fidelity row (base_url env-var pattern; AC-001, AC-002, AC-005).

ADR-028 §D2 records the D-747 LOCKED constraint: `auth_type = "oauth2_client_credentials"` and
`auth_plugin = "crowdstrike-oauth2"` must not be changed.

---

## Origin

Established by ADR-031 §D8-c (v1.2 amendment, 2026-05-31). The real CrowdStrike Falcon
API is region-routed: MSSP clients may have tenants on multiple regions. Prism's
`crowdstrike.sensor.toml` currently hardcodes `base_url = "https://api.crowdstrike.com"`
(us-1 only). An MSSP with eu-1 tenants cannot use prism as-is.

Per ADR-031 §D8-c, this was previously classified as a non-blocking follow-up (P3). It
is reclassified as REQUIRED fidelity per user directive 2026-05-31 ("all sensors,
best-in-class, no scope compromises").

**Real CrowdStrike API regions (canonical reference: poller-cobra semport):**
| Region | Base URL |
|--------|----------|
| us-1 (default) | `https://api.crowdstrike.com` |
| us-2 | `https://api.us-2.crowdstrike.com` |
| eu-1 | `https://api.eu-1.crowdstrike.com` |
| gov | `https://api.laggar.gcw.crowdstrike.com` |

**Current TOML state (grounded from code):**
```toml
# crowdstrike.sensor.toml
sensor_id = "crowdstrike"
base_url = "https://api.crowdstrike.com"   # ← hardcoded us-1
auth_type = "oauth2_client_credentials"     # D-747 LOCKED — do NOT change
auth_plugin = "crowdstrike-oauth2"          # D-747 LOCKED — do NOT change
```

**Pattern already established by Armis and Claroty:**
```toml
# armis.sensor.toml
base_url = "${env.ARMIS_INSTANCE_URL}"

# claroty.sensor.toml
base_url = "${env.CLAROTY_INSTANCE_URL}"
```

**Required fix per ADR-031 §D8-c:**
```toml
# crowdstrike.sensor.toml (after this story)
base_url = "${env.CROWDSTRIKE_BASE_URL}"
```

**DTU code change:** NONE required. Per ADR-031 §D8-c: "The DTU itself is already
region-agnostic — it binds to `127.0.0.1:0` and accepts any valid OAuth2 + Bearer flow
regardless of base URL. No DTU code changes needed."

---

## Narrative

As the Prism platform team and as MSSP operators, I want `crowdstrike.sensor.toml`'s
`base_url` to be read from `${env.CROWDSTRIKE_BASE_URL}` (matching the Armis and Claroty
env-var pattern), so that prism works with CrowdStrike tenants in any region — not just
us-1 — and so that the spec engine emits a structured E-SPEC error (not a panic) when
the env var is absent.

---

## Story-Level Goal

After this story merges:

1. `crowdstrike.sensor.toml` has `base_url = "${env.CROWDSTRIKE_BASE_URL}"`.
2. A TOML comment documents the 4 CrowdStrike region base URLs for operator reference.
3. When `CROWDSTRIKE_BASE_URL` is unset at spec-load time, the spec engine emits a
   structured `E-SPEC-NNN` error (not a panic). This behavior must be verified — if the
   spec engine already handles missing `${env.VAR}` gracefully for Armis/Claroty, this
   is confirmed by test; if not, it must be implemented in-scope (production-grade default).
4. A spec-load test verifies the sensor spec resolves correctly when `CROWDSTRIKE_BASE_URL`
   is set to the eu-1 URL (`https://api.eu-1.crowdstrike.com`).
5. A DTU demo test verifies the spec loads and the pipeline connects when
   `CROWDSTRIKE_BASE_URL` points to the local DTU address.
6. `auth_type`, `auth_plugin`, and all other TOML fields are UNCHANGED.

---

## Behavioral Contracts

| BC ID | Title | Role in This Story |
|-------|-------|-------------------|
| BC-2.16.009 | Spec File Validation — Schema Validation, Variable Reference Resolution, OCSF Field Validation | §Validation Rules 6 env-var resolver governs `${env.CROWDSTRIKE_BASE_URL}` resolution; E-SPEC-024 on missing/empty var (AC-003); EC-009-008/EC-009-009 are the CrowdStrike test vectors |
| BC-2.16.013 | Bundled Sensor Spec Authoring and DTU-Parity Verification — 4 Initial Sensors | §Postconditions §1 CrowdStrike row governs `base_url = "${env.CROWDSTRIKE_BASE_URL}"` spec authoring fidelity; parity test uses DTU SocketAddr override (DTU region-agnostic per ADR-031 §D8-c) |

---

## New-BC Flags — Resolved

PO adjudication 2026-06-01: BC-2.16.009 (v1.7) and BC-2.16.013 (v1.22) confirmed;
E-SPEC-024 is the missing/empty env-var error code at spec-load time.

---

## Acceptance Criteria

### AC-001: crowdstrike.sensor.toml base_url uses ${env.CROWDSTRIKE_BASE_URL}
`crates/prism-sensors/specs/crowdstrike.sensor.toml` has `base_url = "${env.CROWDSTRIKE_BASE_URL}"`.
The line `base_url = "https://api.crowdstrike.com"` is removed. A TOML comment immediately
below or above the `base_url` line documents the 4 region values:
```toml
# CrowdStrike Falcon API region base URLs (set CROWDSTRIKE_BASE_URL to the tenant's region):
#   us-1 (default):  https://api.crowdstrike.com
#   us-2:            https://api.us-2.crowdstrike.com
#   eu-1:            https://api.eu-1.crowdstrike.com
#   gov:             https://api.laggar.gcw.crowdstrike.com
base_url = "${env.CROWDSTRIKE_BASE_URL}"
```
(traces to BC-2.16.013 §Postconditions §1 (CrowdStrike spec authoring fidelity))

### AC-002: Spec-load test resolves eu-1 URL when env var is set to eu-1
A test sets `CROWDSTRIKE_BASE_URL=https://api.eu-1.crowdstrike.com` and loads
`crowdstrike.sensor.toml`. The resolved `SensorSpec.base_url` equals
`"https://api.eu-1.crowdstrike.com"`. No error is emitted. The spec is otherwise valid
(auth_type, auth_plugin, tables, columns all parse correctly).
(traces to BC-2.16.013 §Postconditions §1 (CrowdStrike spec authoring fidelity))

Red Gate test: `test_BC_2_16_013_crowdstrike_eu1_base_url_env_var_resolves_correctly`

### AC-003: Structured E-SPEC error (not panic) when CROWDSTRIKE_BASE_URL is unset
When `CROWDSTRIKE_BASE_URL` is not set in the environment at spec-load time, loading
`crowdstrike.sensor.toml` returns `Err(SpecEngineError::...)` with an E-SPEC error code
from the error taxonomy. It does NOT panic. The error message includes the variable name
`CROWDSTRIKE_BASE_URL` so operators know what to configure.

**Pre-implementation check:** Before implementing, verify this behavior for Armis:
load `armis.sensor.toml` with `ARMIS_INSTANCE_URL` unset and observe the result. If the
spec engine already returns a structured error (not panic), this AC is confirmed by the
same existing behavior and only requires a test against the CrowdStrike spec. If the spec
engine panics, implement the E-SPEC error in-scope (production-grade default — do not defer).
(traces to BC-2.16.009 §Validation Rules 6 (env-var resolver) + E-SPEC-024; EC-009-009)

Red Gate test: `test_BC_2_16_013_crowdstrike_base_url_env_unset_returns_spec_error_not_panic`

### AC-004: DTU demo path works when env var points to local DTU address
A test sets `CROWDSTRIKE_BASE_URL=http://127.0.0.1:<dtu_port>` and loads
`crowdstrike.sensor.toml`. The spec loads successfully. The pipeline can connect to the
DTU at that address (OAuth2 token exchange and detection fetch complete without error).
This proves the env-var pattern does not break the DTU demo path.
(traces to BC-2.16.013 §Postconditions §2 (DTU parity; DTU region-agnostic per ADR-031 §D8-c))

Red Gate test: `test_BC_2_16_013_crowdstrike_base_url_env_points_to_local_dtu_demo_works`

### AC-005: auth_type and auth_plugin are unchanged
`crowdstrike.sensor.toml` after this story still has:
- `auth_type = "oauth2_client_credentials"` (D-747 LOCKED)
- `auth_plugin = "crowdstrike-oauth2"` (D-747 LOCKED)
These values are NOT changed by this story. A spec-load test confirms the parsed
`SensorSpec.auth_type` and `SensorSpec.auth_plugin` remain correct.
(traces to BC-2.16.013 §Postconditions §1 (CrowdStrike spec authoring fidelity))

### AC-006: No uncatalogued tracing event_type emissions (SAP-1)
If any new `tracing::*!(event_type = ...)` site is introduced in this story's implementation
(e.g., in the env-var resolution error path of spec_parser.rs), it must have a corresponding
row in BC-2.16.002 Structured Event Catalog. Zero uncatalogued `event_type` emissions permitted.
(SAP-1 standing probe: no uncatalogued event_type emissions; ref BC-2.16.002 Structured Event Catalog)

---

## Red Gate Tests

| Test Name | AC | Crate | Description |
|-----------|----|-------|-------------|
| `test_BC_2_16_013_crowdstrike_eu1_base_url_env_var_resolves_correctly` | AC-002 | prism-spec-engine | Set CROWDSTRIKE_BASE_URL=eu-1 URL; load spec; assert base_url resolved to eu-1 |
| `test_BC_2_16_013_crowdstrike_base_url_env_unset_returns_spec_error_not_panic` | AC-003 | prism-spec-engine | Unset CROWDSTRIKE_BASE_URL; load spec; assert Err(SpecEngineError) not panic |
| `test_BC_2_16_013_crowdstrike_base_url_env_points_to_local_dtu_demo_works` | AC-004 | prism-spec-engine | Set CROWDSTRIKE_BASE_URL to DTU addr; spec loads with DTU address as base_url |

---

## Tasks

1. **Read** `crates/prism-sensors/specs/crowdstrike.sensor.toml` — confirm current
   `base_url = "https://api.crowdstrike.com"` (hardcoded us-1).
2. **Read** `crates/prism-sensors/specs/armis.sensor.toml` and `claroty.sensor.toml` —
   confirm the `${env.ARMIS_INSTANCE_URL}` and `${env.CLAROTY_INSTANCE_URL}` patterns.
   These are the template for the CrowdStrike change.
3. **Read** `crates/prism-spec-engine/src/spec_parser.rs` — find the `${env.VAR}`
   interpolation implementation; understand what happens when the env var is absent
   (structured error or panic). This is the critical pre-check for AC-003.
4. **Pre-check (MANDATORY before writing Red Gate tests):** Run a quick test with
   `ARMIS_INSTANCE_URL` unset and load armis.sensor.toml. Observe:
   - Does the spec engine return `Err(SpecEngineError::...)` with an E-SPEC code? → AC-003
     is satisfied by the existing behavior; write a test proving the same for CrowdStrike.
   - Does it panic? → Implement E-SPEC structured error in `spec_parser.rs` first
     (in-scope per production-grade default). Do NOT defer this fix.
5. **Write Red Gate tests** (must ALL FAIL before implementation):
   - `test_BC_2_16_013_crowdstrike_eu1_base_url_env_var_resolves_correctly`
   - `test_BC_2_16_013_crowdstrike_base_url_env_unset_returns_spec_error_not_panic`
   - `test_BC_2_16_013_crowdstrike_base_url_env_points_to_local_dtu_demo_works`
6. **Update** `crates/prism-sensors/specs/crowdstrike.sensor.toml`:
   ```toml
   # CrowdStrike Falcon API region base URLs (set CROWDSTRIKE_BASE_URL to the tenant's region):
   #   us-1 (default):  https://api.crowdstrike.com
   #   us-2:            https://api.us-2.crowdstrike.com
   #   eu-1:            https://api.eu-1.crowdstrike.com
   #   gov:             https://api.laggar.gcw.crowdstrike.com
   base_url = "${env.CROWDSTRIKE_BASE_URL}"
   ```
   Do NOT change `auth_type`, `auth_plugin`, or any other field.
7. **If spec_parser.rs panics on missing env var** (discovered in step 4): implement
   structured E-SPEC error return in `spec_parser.rs` for all three sensors (CrowdStrike,
   Armis, Claroty use the same path). Sibling-site sweep (TD-VSDD-060): fix all three
   sensors' missing-env-var handling in the same commit if the bug exists. Check the error
   taxonomy for the correct E-SPEC code for "required env var missing at spec-load time."
8. **Run** `cargo nextest run -p prism-sensors` — Red Gate tests must now PASS GREEN.
9. **Run** `cargo nextest run -p prism-spec-engine` — ensure spec-engine tests unaffected.
10. **Run** `just check` — final pre-push gate.

---

## File List

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-sensors/specs/crowdstrike.sensor.toml` | MODIFY | base_url → `${env.CROWDSTRIKE_BASE_URL}`; add region runbook comment |
| `crates/prism-spec-engine/src/spec_parser.rs` | CONDITIONALLY MODIFY | Add structured E-SPEC error for missing ${env.VAR} if current behavior is panic (in-scope per production-grade default) |
| `crates/prism-sensors/tests/` or `crates/prism-spec-engine/tests/` | MODIFY or CREATE | Add 3 Red Gate tests for AC-002/AC-003/AC-004 |

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| D-747 LOCKED: auth_type and auth_plugin must NOT change | ADR-028 §D2 D-747 | Adversary checks auth_type and auth_plugin unchanged per SAP-2 |
| Structured E-SPEC error on missing env var (not panic) | CLAUDE.md Canonical Principle — production-grade default | AC-003; panic on missing config is a production defect, not acceptable |
| base_url env-var pattern matches Armis/Claroty convention | ADR-031 §D8-c; convention established by armis.sensor.toml + claroty.sensor.toml | TOML uses ${env.CROWDSTRIKE_BASE_URL} — exact syntax must match existing pattern |
| No DTU code changes | ADR-031 §D8-c: "DTU is already region-agnostic" | No commits to crates/prism-dtu-crowdstrike/* in this story |
| No println! in production code | CLAUDE.md Conventions | Use tracing::*! with structured fields only |
| New event_type emissions require BC-2.16.002 catalog row | SAP-1 + PG-LP11-001 | Adversary greps event_type = on every pass |

### Forbidden Dependencies

`prism-sensors` must NOT gain a dependency on `prism-spec-engine` (the TOML files are
spec artifacts, not code). If spec-load tests are in `prism-spec-engine`, they may import
`prism-sensors` fixture paths but NOT vice versa.

`prism-dtu-crowdstrike` MUST NOT be modified by this story (DTU is region-agnostic per
ADR-031 §D8-c; no code changes required there).

---

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `prism-spec-engine` | workspace path | SensorSpec loading; ${env.VAR} interpolation in spec_parser.rs |
| `prism-sensors` | workspace path | crowdstrike.sensor.toml (modified) |
| `std::env` (stdlib) | — | `std::env::var("CROWDSTRIKE_BASE_URL")` in spec_parser |

No new external dependencies introduced.

---

## Previous Story Intelligence

- **PLUGIN-MIGRATION-001-D** (merged): Authored `crowdstrike.sensor.toml` with the hardcoded
  `base_url = "https://api.crowdstrike.com"`. Read the TOML authorship notes in that file
  before modifying (note D-747 LOCKED constraints, rate_limit_hints, two-step pipeline).

- **S-DTU-CYBERINT-AUTH-FIDELITY-001** (merged PR #164): Demonstrated the production-grade
  default in practice — do not defer observable correctness gaps. The same principle applies
  here: if the spec engine panics on missing env var, fix it in-scope.

- **PLUGIN-MIGRATION-001-E** (merged): Delivered the `crowdstrike-oauth2` WASM plugin
  referenced by `auth_plugin = "crowdstrike-oauth2"`. This value is D-747 LOCKED — this
  story does not touch auth_plugin or auth_type.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | CROWDSTRIKE_BASE_URL set to invalid URL (e.g., "not-a-url") | Spec-load succeeds (env var substitution is syntactic; URL validation is at request time). No error at load time. |
| EC-002 | CROWDSTRIKE_BASE_URL set to empty string | Spec engine returns E-SPEC error: empty base_url is not valid. Same behavior as absent env var (structural: empty URL cannot be used for HTTP requests). |
| EC-003 | CROWDSTRIKE_BASE_URL set to gov URL with additional path segments | Spec loads; base_url is the raw env value; no stripping or normalization of the URL at load time. |
| EC-004 | Multiple CrowdStrike tenants with different regions (multi-org config) | Each org's customer config overrides base_url per per-org sensor endpoint overlay (S-CONFIG-MULTI-TENANT-OVERRIDE-001 story). This story does not implement per-org URL override — it establishes the env-var baseline that per-org config builds upon. |
| EC-005 | DTU demo test: CROWDSTRIKE_BASE_URL=http://127.0.0.1:PORT (http, not https) | Spec loads; DTU operates over plain HTTP (ADR-031 §D2-c permitted divergence: plain HTTP for loopback fixture). |

---

## Notes for Implementer

**D-747 LOCKED constraint is absolute:** `auth_type = "oauth2_client_credentials"` and
`auth_plugin = "crowdstrike-oauth2"` are locked by D-747. This story changes ONLY the
`base_url` line. No other TOML field is touched. Verify this constraint in the diff before
committing.

**Pre-check for E-SPEC error behavior is MANDATORY:** Step 4 requires checking whether
the spec engine already handles missing `${env.VAR}` gracefully. If it panics, the fix
must be applied in-scope for ALL sensors that use `${env.VAR}` (Armis, Claroty, CrowdStrike
— all use the same interpolation path). The fix is a sibling-site sweep per TD-VSDD-060.
This is a production-grade issue: an MCP server that panics on missing configuration is
not production-grade. Fix it; do not defer.

**Error code for missing env var:** Check the error taxonomy
(`.factory/specs/prd-supplements/error-taxonomy.md`) for the E-SPEC code that covers
"required environment variable missing at spec-load time." If none exists, flag to PO
for taxonomy addition (do not invent a code). Likely candidates: E-SPEC-012 (if it exists
per CLAUDE.md hint `E-SPEC-012` in the `auth_plugin` validation note).

**TOML comment format:** The region runbook comment must be immediately adjacent to the
`base_url` line (above it), using the `# key: value` convention consistent with other
comments in the file. Do not add a separate top-level comment block — keep it local to
the field it documents.

---

## Risk Mitigations

| Risk | Mitigation |
|------|-----------|
| Spec engine panics on missing env var (discovered in pre-check) | Fix in-scope per production-grade default; sibling-site sweep for Armis + Claroty too; add E-SPEC error code to taxonomy if missing |
| auth_type or auth_plugin inadvertently changed | Verify D-747 LOCKED values in TOML diff before committing; Red Gate test AC-005 explicitly checks these values |
| New event_type emission uncatalogued | SAP-1 sweep after implementation: `rg 'event_type\s*=' crates/ --type rust`; zero new emissions without catalog rows |
| Multi-org per-tenant URL override not in scope | EC-004 explicitly defers multi-org URL override to S-CONFIG-MULTI-TENANT-OVERRIDE-001; do not attempt per-org base_url override in this story |

---

## Token Budget Estimate

| Context source | Estimated tokens |
|----------------|-----------------|
| This story spec | ~3,500 |
| crowdstrike.sensor.toml | ~1,500 |
| armis.sensor.toml (env-var pattern reference) | ~1,500 |
| claroty.sensor.toml (env-var pattern reference) | ~1,500 |
| crates/prism-spec-engine/src/spec_parser.rs (env-var interpolation section) | ~2,000 |
| ADR-031 §D8-c (relevant section) | ~1,000 |
| POLLER-DTU-FIDELITY-AUDIT-2026-05-29.md §1 (CrowdStrike Gap-CS-003) | ~800 |
| error-taxonomy.md (E-SPEC-NNN section) | ~500 |
| BC files (2 BCs): BC-2.16.009 §Validation Rules 6 + BC-2.16.013 §Postconditions §1 CrowdStrike row | ~700 |
| Tool outputs (cargo nextest) | ~1,500 |
| **Total estimate** | **~14,500 tokens (~6% of 256K context)** |

Well within the 20-30% budget.

---

## References

- ADR-031 v1.2 §D8-c — CrowdStrike Multi-Region Base URL Fidelity (Gap-CS-003)
- POLLER-DTU-FIDELITY-AUDIT-2026-05-29.md §1 — CrowdStrike Gap-CS-003 row
- `crates/prism-sensors/specs/crowdstrike.sensor.toml` — current hardcoded us-1 base_url
- `crates/prism-sensors/specs/armis.sensor.toml` — env-var pattern template
- `crates/prism-sensors/specs/claroty.sensor.toml` — env-var pattern template
- `.factory/specs/prd-supplements/error-taxonomy.md` — E-SPEC error codes
- BC-2.16.009 v1.7 "Spec File Validation — Schema Validation, Variable Reference Resolution, OCSF Field Validation" — §Validation Rules 6 env-var resolver; E-SPEC-024 on missing/empty var; EC-009-008/EC-009-009 CrowdStrike test vectors
- BC-2.16.013 v1.22 "Bundled Sensor Spec Authoring and DTU-Parity Verification — 4 Initial Sensors" — §Postconditions §1 CrowdStrike spec authoring fidelity (base_url env-var pattern); §Postconditions §2 DTU parity (region-agnostic per ADR-031 §D8-c)

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.5 | 2026-08-02 | story-writer | Round 6 DRIFT-STORY-AUTHORITY-ABSENT-CORPUS-001 (D-2084): added §Authority section; bumped body version fields (H1 title + bold Version) from v1.4 to v1.5. |
| 1.4 | 2026-06-02 | product-owner | Adversary fix-burst closing F-PB-HIGH-001 + F-PB-MED-001 + F-PC-LOW-001 (story-side). F-PB-HIGH-001: added §References entries for BC-2.16.009 v1.7 and BC-2.16.013 v1.22 with verbatim H1 titles (POL-7 D-571 verification step). F-PB-MED-001: rewrote AC-006 trace from "(traces to BC-2.16.002...)" to SAP-1 standing-probe guard form, eliminating POL-8 trace-without-frontmatter asymmetry; BC-2.16.002 correctly absent from behavioral_contracts frontmatter. F-PC-LOW-001 (story-side): synced all Red Gate test name citations in §Red Gate Tests table, AC-002/003/004 blocks, and Tasks §5 to verbatim fn names with BC_2_16_013_ infix per bc_2_16_013_crowdstrike_multiregion.rs (test-file doc-table + header version pin v1.1→v1.3 routed to implementer; see worktree fix note). |
| 1.3 | 2026-06-02 | story-writer | Bump-stable cite fix closing F-P2-OBS-001. Frontmatter historical comment re-phrased from volatile "see BC-INDEX v5.74" head-version pointer to burst-stable cite "established D-946" per TD-VSDD-091 spirit and adversary recommendation. No substantive content change. |
| 1.2 | 2026-06-02 | story-writer | Cite-currency fix closing F-P1-OBS-001. §New-BC Flags live confirmation pin BC-2.16.013 (v1.19) → (v1.22) (current active version per BC-INDEX v5.74). Frontmatter historical comment annotated "(BC now at v1.22)" per TD-VSDD-091 (historical record preserved; current state clarified). No substantive content change. |
| 1.1 | 2026-06-01 | story-writer | PO BC-coverage propagation burst. Attached BC-2.16.009 (v1.7) and BC-2.16.013 (v1.19) per PO commit 30111e52. Updated §Behavioral Contracts table (verbatim titles from BC H1s per POL-7). Updated AC-001/AC-002/AC-005 traces → BC-2.16.013 §Postconditions §1; AC-003 → BC-2.16.009 §Validation Rules 6 + E-SPEC-024; AC-004 → BC-2.16.013 §Postconditions §2; AC-006 → BC-2.16.002 §Structured Event Catalog. Resolved New-BC Flags section with PO adjudication (E-SPEC-024 confirmed). Token Budget updated: BC files row added (~700 tokens), total ~14,500. Status: draft → ready (S-7.01 gate cleared). |
| 1.0 | 2026-05-31 | story-writer | Initial materialization from [stub] per ADR-031 §D8-c v1.2 reclassification. 6 ACs, 3 Red Gate tests, 2 pts, wave 5, P2. Grounded against crowdstrike.sensor.toml (hardcoded us-1 base_url), armis.sensor.toml + claroty.sensor.toml (env-var pattern precedent), spec_parser.rs (env-var interpolation path — pre-check required). D-747 LOCKED constraint explicitly enforced. Mandatory pre-check for E-SPEC structured error behavior documented in Tasks step 4 with in-scope fix requirement per production-grade default. New-BC flags provided to PO for env-var resolution BC coverage confirmation. |
