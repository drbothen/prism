---
document_type: story
story_id: S-WAVE-A-ARMIS-ACTIVITY-001
title: "Armis Device Activity TOML Surface — Add armis_device_activity Table to Spec"
version: "1.0"
status: draft
producer: story-writer
phase: 3
wave: wave-a
epic_id: E-WAVE-A-SENSOR-REMEDIATION
priority: P2
points: 3
tdd_mode: strict
# tdd_mode NOTE (SAC-1 / BC-8.30.001): tdd_mode: strict is the required default.
# Red Gate list and BC-5.38.001 density check are PENDING — implementation cannot begin
# until the architect confirms variable injection grammar (see §Blocking Dependency below)
# and the product-owner authors BCs. The story will be dispatched only after that gate clears.
target_module: prism-sensors
subsystems: ["SS-06 (SensorSpec)", "SS-12 (DTU-Armis)"]
depends_on: []
blocks: []
behavioral_contracts: []
# BC status: pending PO authorship. Implementation is blocked on architect confirmation
# of variable injection pattern `${variable.device_id}` in path_template. Once architect
# confirms support (or authors a grammar extension ADR), the product-owner can author
# BCs and this story can transition to status: ready. Per S-7.01 Spec-First Gate: status
# MUST NOT be set to `ready` while behavioral_contracts: [] — frontmatter comment required.
verification_properties: []
assumption_validations: []
risk_mitigations: []
estimated_days: 1
---

# S-WAVE-A-ARMIS-ACTIVITY-001: Armis Device Activity TOML Surface

## Narrative

As a Prism maintainer, I want `armis.sensor.toml` to declare an `armis_device_activity`
table so that the `get_device_activity` DTU route (which already exists in
`prism-dtu-armis::routes::devices`) becomes reachable from spec-driven queries, enabling
per-device activity timeline queries for the Armis sensor surface.

---

## Blocking Dependency (ARCHITECT-CONFIRMATION-REQUIRED)

**Status: BLOCKED — implementation cannot begin.**

The `armis_device_activity` surface requires a parameterized per-device fetch:
```
path_template = "/api/v1/devices/${variable.device_id}/activity"
```

This uses a `${variable.*}` interpolation token that injects a value from a parent
query result set (the device ID from a prior `armis_devices` query row). Before this
TOML table can be authored, the architect must confirm:

1. **Is `${variable.*}` injection currently supported** in `spec_parser.rs` and
   `pipeline.rs`? (The existing flat-table AQL queries `in:devices` and `in:alerts`
   via `GET /api/v1/search` do not exercise this pattern.)

2. **If NOT currently supported**, what ADR and story anchor defines the grammar
   extension for variable injection in `path_template`?

This deferral is documented in **BC-2.02.006 EC-02-014** (product-owner, FB68d).
Story `S-WAVE-A-ARMIS-ACTIVITY-001` is the resolution target for that deferral per
Canonical Principle Rule 3: a deferral must attach to a real story ID so it cannot
get lost.

**Until the architect confirms the variable injection pattern, the TOML table cannot
be authored correctly, and the product-owner cannot write the behavioral contract.**

---

## Ground-Truth DTU State (confirmed from code)

The following are confirmed from direct code reading before authoring this story:

| Item | Source | State |
|------|--------|-------|
| `ActivityRecord` struct | `crates/prism-dtu-armis/src/types.rs` | Present: `activity_id: String`, `device_id: String`, `activity_type: String`, `timestamp: String`, `details: serde_json::Value` |
| `ActivityData` struct | `crates/prism-dtu-armis/src/types.rs` | Present: `activities: Vec<ActivityRecord>`, `total: u32` |
| `ActivityResponse` struct | `crates/prism-dtu-armis/src/types.rs` | Present: `data: ActivityData` |
| `get_device_activity` handler | `crates/prism-dtu-armis/src/routes/devices.rs` | Fully implemented, auth-checked, filters by `device_id`, returns `ActivityResponse` |
| Route registration | `crates/prism-dtu-armis/src/clone.rs` | Registered at `GET /api/v1/devices/:device_id/activity` in `build_router()` |
| `armis.sensor.toml` activity surface | `crates/prism-sensors/specs/armis.sensor.toml` | No `armis_device_activity` or `device_activity` table declared — surface unreachable from spec-driven queries |

---

## Acceptance Criteria

**NOTE: Acceptance criteria are PENDING — cannot be authored until the architect confirms
variable injection grammar.** Placeholder criteria are recorded below to sketch intent.
These MUST be replaced with BC-traced criteria before this story transitions to `status: ready`.

### AC-001 (PLACEHOLDER — pending architect confirmation and BC authorship)
`armis.sensor.toml` declares an `armis_device_activity` table with:
- `path_template` containing the confirmed variable injection syntax for device_id
- `response_path = "$.data.activities"` (matching `ActivityResponse.data.activities`)
- All 5 `ActivityRecord` fields mapped as TOML columns with SAP-2 parity

### AC-002 (PLACEHOLDER — pending BC authorship)
Every column declared in the `armis_device_activity` table maps to a field in
`ActivityRecord` (SAP-2: column with no DTU struct field = P1 CRITICAL).

| TOML column | ActivityRecord field | Rust type | column_type |
|-------------|---------------------|-----------|-------------|
| `activity_id` | `activity_id` | `String` | `String` |
| `device_id` | `device_id` | `String` | `String` |
| `activity_type` | `activity_type` | `String` | `String` |
| `timestamp` | `timestamp` | `String` | `Datetime` (with timestamp_formats) |
| `details` | `details` | `serde_json::Value` | `Json` |

### AC-003 (PLACEHOLDER — pending BC authorship)
At least one test asserts the `armis_device_activity` table appears in the registered
sensor spec and its columns match the `ActivityRecord` struct fields enumerated above.

---

## Architecture Mapping

| Component | Module | Pure/Effectful | Architecture Section |
|-----------|--------|---------------|----------------------|
| `armis.sensor.toml` (activity table) | `crates/prism-sensors/specs/` | Pure (config data) | `architecture/module-decomposition.md §SS-06 SensorSpec` |
| `get_device_activity` handler | `crates/prism-dtu-armis/src/routes/devices.rs` | Effectful (HTTP handler) | `architecture/module-decomposition.md §SS-12 DTU-Armis` |
| `ActivityRecord` / `ActivityData` / `ActivityResponse` | `crates/prism-dtu-armis/src/types.rs` | Pure (data types) | `architecture/module-decomposition.md §SS-12 DTU-Armis` |

---

## Behavioral Contracts

| BC | Version | Relevance |
|----|---------|-----------|
| BC-2.02.006 | v1.7 | Armis surfaces contract; EC-02-014 records this gap and its deferral to S-WAVE-A-ARMIS-ACTIVITY-001 |

No additional BCs until architect confirms variable injection support. Product-owner
must author new BC(s) covering the `armis_device_activity` surface after that gate.

---

## UX / Operator Surfaces

None — this story produces no user-facing UI changes. The only surface change is
the addition of a new queryable table in `armis.sensor.toml`.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `device_id` variable not injected (grammar not supported) | Spec load fails or pipeline returns E-SPEC error — NOT silent empty result |
| EC-002 | Device has no activity records | `armis_device_activity` query returns empty result set — not an error |
| EC-003 | `details` field contains nested JSON object | Serialized as JSON string per `column_type = "Json"` |

---

## Tasks

**All tasks are BLOCKED pending architect confirmation of variable injection grammar.**

### T-01 (BLOCKED): Author `armis_device_activity` table in `armis.sensor.toml`
**Files:** `crates/prism-sensors/specs/armis.sensor.toml` (MODIFY)

Add `[[tables]]` block for `armis_device_activity` surface. Specifics (endpoint path,
pagination treatment, variable injection syntax) depend on architect confirmation.

**Blocker:** variable injection pattern `path_template = "/api/v1/devices/${variable.device_id}/activity"`
requires architect confirmation before this TOML block can be authored. Filing the
task now so the story is a concrete implementation anchor — not so the work can proceed.

### T-02 (BLOCKED): SAP-2 column parity verification
After T-01, verify every column in the `armis_device_activity` table matches an
`ActivityRecord` field. Missing-column-in-DTU = P1 CRITICAL per SAP-2 protocol.

---

## Token Budget Estimate

| Context source | Estimated tokens |
|----------------|-----------------|
| This story spec | ~1,500 |
| `armis.sensor.toml` (current state, reference) | ~2,000 |
| `crates/prism-dtu-armis/src/types.rs` (ActivityRecord types) | ~500 |
| Architect confirmation ADR (TBD) | ~1,000 |
| BC files (pending authorship) | ~1,000 |
| Running test output (nextest per-crate) | ~1,000 |
| **Total estimate** | **~7,000** |

Small story (3 points); single TOML table addition once unblocked.

---

## Previous Story Intelligence

**From F-SAP2-MED-006 probe (wave-a-sap2-probe-pass-65):**
- `ActivityRecord`, `ActivityResponse`, `ActivityData` confirmed present in
  `prism-dtu-armis::types`; `get_device_activity` handler confirmed fully implemented
  and registered at `GET /api/v1/devices/:device_id/activity` in `clone.rs`
- The DTU is ready; only the TOML spec and variable injection grammar are missing

**From S-WAVE-A-ARMIS-REMEDIATION-001 (predecessor wave-a story):**
- Armis uses bearer static auth; `check_bearer_auth` pattern already established
- Activity route is auth-checked; no additional auth work needed in this story

---

## Architecture Compliance Rules

1. **SAP-2 mandatory:** every TOML column in `armis_device_activity` MUST have a
   corresponding field in `ActivityRecord` (confirmed in `types.rs`). Do not add columns
   that have no backing struct field.

2. **Wire-shape assertion discipline (CLAUDE.md):** any test covering the DTU HTTP
   surface must assert on serialized JSON output, not only on Rust struct state. At
   least one test must assert on the serialized `ActivityResponse` shape from the DTU.

3. **ADR-028 §D1 — DTU-grounded spec authoring:** path_template must match the DTU
   route exactly. The registered route is `GET /api/v1/devices/:device_id/activity`.

4. **Variable injection grammar:** do NOT use `${variable.*}` syntax unless the
   architect has confirmed support. Using unsupported syntax silently produces an
   uninterpolated literal path, causing 404 on every request.

---

## Library & Framework Requirements

| Library | Version | Source of truth |
|---------|---------|----------------|
| `axum` | pinned in workspace `Cargo.toml` | `architecture/dependency-graph.md §External Dependencies` |
| `serde` / `serde_json` | pinned in workspace `Cargo.toml` | same |

No new external dependencies are introduced by this story.

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-sensors/specs/armis.sensor.toml` | MODIFY | Task T-01 (BLOCKED); add `[[tables]]` block for `armis_device_activity` surface |

No new files created by this story — the DTU handler and types already exist.

---

## Verification Properties

None assigned yet — pending BC authorship.

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-07-27 | story-writer | Initial authoring (FB69). Created to satisfy Canonical Principle Rule 3: F-SAP2-MED-006 deferred gap `armis_device_activity` surface required a real story anchor. Ground-truth DTU state confirmed from code. Blocked on architect confirmation of variable injection grammar. Deferral cross-referenced to BC-2.02.006 EC-02-014 (FB68d). |
