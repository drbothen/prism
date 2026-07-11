---
document_type: story
story_id: "S-DRIFT-SAP2-DEVICES-TOML-SURFACE-001"
title: "CrowdStrike devices TOML column surface expansion + DTU harness constant-time token comparison"
wave: maintenance
epic_id: maintenance
priority: P1
status: draft
version: "0.2"
spec_version: "v0.2"
level: ops
producer: story-writer
timestamp: "2026-07-11"
modified: "2026-07-11"
input-hash: ""
inputs:
  - crates/prism-sensors/specs/crowdstrike.sensor.toml
  - crates/prism-dtu-crowdstrike/src/generator.rs
  - crates/prism-dtu-harness/src/builder.rs
  - crates/prism-dtu-harness/src/clone_server.rs
  - .factory/specs/behavioral-contracts/BC-2.16.013-bundled-sensor-spec-authoring-and-dtu-parity.md
  - .factory/specs/behavioral-contracts/BC-2.11.016-e-query-038-column-not-found.md
origin_finding: "DRIFT-SAP2-DEVICES-TOML-SURFACE-001 + DRIFT-HARNESS-ADMIN-TOKEN-CT-001"
origin_cascade: "D-1666 human decision (2026-07-10); unblocked by PR #221 develop@5f1b5771 (2026-07-11)"
cycle: "v1.0.0-greenfield"
phase: 3
tdd_mode: strict
track: "Platform Engineering"
subsystems: [SS-16]
crates_touched:
  - prism-sensors
  - prism-dtu-harness
target_module: "crates/prism-sensors/specs/crowdstrike.sensor.toml"
behavioral_contracts:
  - BC-2.16.013
  - BC-2.11.016
# BC status: BC-2.16.013 v1.32 and BC-2.11.016 v1.27 are both active.
# BC-2.16.013 (INV-HARNESS-ROUTE-PARITY) governs SAP-2 DTU-TOML field parity.
# v1.32 adds explicit constant-time admin-token comparison clause (CWE-208, OQ-001 closed).
# BC-2.11.016 (E-QUERY-038) governs the column-gate available_columns set that
# expands when new TOML columns are added.
# OQ-001 RESOLVED: BC-2.16.013 amended to v1.32 with constant-time comparison clause.
# AC↔BC bidirectional traces required before status=ready (S-7.01).
verification_properties: []
depends_on: []
blocks: []
points: 5
estimated_days: 1.5
risk: MEDIUM
acceptance_criteria_count: 5
red_gate_tests: 5
estimated_passes: "3-4"
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
---

# S-DRIFT-SAP2-DEVICES-TOML-SURFACE-001: CrowdStrike devices TOML column surface expansion + DTU harness constant-time token comparison

## §Origin — [drift] DRIFT-SAP2-DEVICES-TOML-SURFACE-001 + DRIFT-HARNESS-ADMIN-TOKEN-CT-001

**Cascade:** DEFECT-CSDEVICES-EMPTY-PIPELINE-001, unblocked by PR #221 develop@5f1b5771 (2026-07-11)
**Human decision:** D-1666 (2026-07-10): DRIFT-SAP2-DEVICES-TOML-SURFACE-001 EXPAND-NOW post-merge; DRIFT-HARNESS-ADMIN-TOKEN-CT-001 new drift item registered same session

During the DEFECT-CSDEVICES-EMPTY-PIPELINE-001 cascade, the `devices` table in
`crates/prism-sensors/specs/crowdstrike.sensor.toml` was kept intentionally minimal
to unblock the pipeline fix. The DTU generator (`make_device()` in
`crates/prism-dtu-crowdstrike/src/generator.rs`) emits five fields absent from the TOML
spec: `os_version`, `agent_version`, `external_ip`, `local_ip`, `containment_status`.
SAP-2 requires TOML↔DTU column parity; this story closes the gap.

Additionally, the DTU harness admin-token bearer comparison across all clone
implementations uses Rust `!=` / `==` string equality — a non-constant-time comparison
vulnerable to timing side-channel information leakage (CWE-208). This story remediates
all harness comparison sites.

The anchor comments in `crowdstrike.sensor.toml` around the `devices` table
(lines starting "Column surface intentionally minimal…" and the `fan_out_batch_size`
tuning note) reference this story; they must be removed or updated as part of delivery.

## Behavioral Contracts

| BC | Title | Version | Relevance |
|----|-------|---------|-----------|
| BC-2.16.013 | Bundled Sensor Spec Authoring and DTU-Parity Verification — 4 Initial Sensors | v1.32 | INV-HARNESS-ROUTE-PARITY: SAP-2 mandates that every TOML-declared column exists as a field in the DTU generator response struct for that table. Adding five TOML columns creates five new SAP-2 parity obligations. v1.32 adds explicit constant-time admin-token bearer comparison requirement (CWE-208, AC-004 anchor). |
| BC-2.11.016 | E-QUERY-038 Column-Not-Found Plan-Time Gate (L4) | v1.27 | The gate sources `available_columns` from `columns_for_table()` which reads the TOML spec. Expanding the `devices` column set allows analysts to reference `os_version`, `agent_version`, etc. without false E-QUERY-038 rejections. |

## Acceptance Criteria

### AC-001 — Five columns added to `crowdstrike.sensor.toml` devices table with verified SAP-2 parity
(traces to BC-2.16.013 v1.32 INV-HARNESS-ROUTE-PARITY postcondition: "every TOML-declared column has a matching field in the DTU generator response for that table")

The `[[tables]]` block for `table_name = "devices"` in
`crates/prism-sensors/specs/crowdstrike.sensor.toml` gains five new `[[tables.columns]]`
entries verified against `make_device()` in
`crates/prism-dtu-crowdstrike/src/generator.rs`:

| TOML column name | `column_type` | OCSF field | DTU `make_device()` field | Type match |
|------------------|--------------|------------|--------------------------|------------|
| `os_version` | `"string"` | `"device.os.version"` | `"os_version": "Ubuntu 22.04"` | string |
| `agent_version` | `"string"` | `"device.agent.version"` | `"agent_version": "7.10.0.0"` | string |
| `external_ip` | `"string"` | `"device.ip"` | `"external_ip": "203.0.113.1"` | string |
| `local_ip` | `"string"` | `"src_endpoint.ip"` | `"local_ip": "10.0.0.1"` | string |
| `containment_status` | `"string"` | `"status_detail"` | `"containment_status": "normal"` / `"contained"` | string |

A Red Gate test `test_BC_2_16_013_SAP2_devices_columns_parity_crowdstrike` asserts that
each of the five new column names appears in both the TOML-declared column list (via the
`TableRegistry` or parsed spec) AND in the generator's fixture output for the `devices`
table. Failure means SAP-2 gap has been reintroduced.

### AC-002 — Anchor comments removed from `crowdstrike.sensor.toml` devices block
(traces to BC-2.16.013 v1.32 INV-HARNESS-ROUTE-PARITY postcondition: "spec is complete; no deferred-expansion placeholders for shipped columns")

The two placeholder anchor comments in `crates/prism-sensors/specs/crowdstrike.sensor.toml`
that reference DRIFT-SAP2-DEVICES-TOML-SURFACE-001 are removed:
1. The block comment above `[[tables]]` for `devices`: "Column surface intentionally minimal...queued as a dedicated story after this PR merges."
2. The inline comment on the `fetch_devices` step's `fan_out_batch_size = 100`: the deferred-tuning parenthetical referencing this story.

The `fan_out_batch_size` comment itself is replaced with a permanent rationale reflecting
the outcome of AC-003.

### AC-003 — `fan_out_batch_size` for `fetch_devices` step documented with benchmark-informed rationale
(traces to BC-2.16.013 v1.32 §Postconditions: "step parameters are grounded in the DTU-verified API contract")

The `fetch_devices` POST step's `fan_out_batch_size` field is reviewed against the
CrowdStrike POST `/devices/entities/devices/v2` documented limit (5000 IDs per call,
enforced by `MAX_IDS_PER_BATCH = 5_000` in `crates/prism-dtu-crowdstrike/src/routes/hosts.rs`).
The implementer benchmarks or reasons through memory-pressure impact of larger batches
(a single 5000-element batch at ~500 bytes/device = ~2.5 MB raw JSON per step;
within the 200 MB per-query budget from BC-2.11.006) and documents their finding in
an inline comment adjacent to the `fan_out_batch_size` field. The final value MUST be:
- At least the current 100 if no benchmark evidence supports an increase, OR
- A higher value (up to 5000) with the benchmark result cited in the adjacent comment

An unconditional bump to 5000 without a comment is a failing AC. The decision
artifact need not be a separate file — the TOML comment is the authoritative record.

### AC-004 — All DTU harness admin-token bearer comparisons replaced with constant-time comparison
(traces to BC-2.16.013 v1.32 INV-HARNESS-ROUTE-PARITY postcondition: "harness security controls do not leak timing information about valid tokens")

**Scope:** every `&str` / `String` token comparison site in `crates/prism-dtu-harness/src/`
that accepts an HTTP `Authorization: Bearer <token>` header and compares the extracted
token value against the stored `admin_token`. Identified sites (grep
`!= admin_token\|== admin_token\|!= .*admin_token\|provided != Some` in
`crates/prism-dtu-harness/src/`):

| File | Line vicinity | Current pattern |
|------|--------------|----------------|
| `builder.rs` | `check_bearer()` fn | `if token != admin_token` |
| `clone_server.rs` | bearer middleware | `if provided != Some(state.admin_token.as_str())` |
| `clones/armis.rs` | bearer middleware | `if provided != Some(state.admin_token.as_str())` |
| `clones/armis.rs` | `check_bearer_token` | `Some(token) if token == expected_token` |
| `clones/claroty.rs` | bearer middleware | `if provided != Some(state.admin_token.as_str())` |
| `clones/claroty.rs` | inner check | `if token != admin_token` |
| `clones/cyberint.rs` | bearer middleware | `if provided != Some(state.clone_state.admin_token.as_str())` |
| `clones/cyberint.rs` | inner check | `if token == admin_token` |
| `clones/crowdstrike.rs` | bearer middleware | `if provided != Some(state.admin_token.as_str())` |
| `clones/crowdstrike.rs` | inner check | `if provided_token != admin_token` |
| `clones/jira.rs` | bearer middleware | `if provided != Some(ctx.clone_state.admin_token.as_str())` |
| `clones/pagerduty.rs` | bearer middleware | `if provided != Some(ctx.clone_state.admin_token.as_str())` |
| `clones/slack.rs` | bearer middleware | `if provided != Some(ctx.clone_state.admin_token.as_str())` |

Each comparison is replaced with a constant-time byte comparison using the
`subtle` crate's `ConstantTimeEq` trait or an equivalent `ct_eq` function that
compares the token bytes in constant time regardless of where the first differing byte
appears (CWE-208 timing side-channel mitigation). A short helper function
`ct_compare_tokens(provided: &str, expected: &str) -> bool` is introduced in
a shared location within `crates/prism-dtu-harness/src/` and called from all sites.

A Red Gate test `test_DRIFT_HARNESS_ADMIN_TOKEN_CT_001_ct_compare_tokens_constant_time`
asserts that `ct_compare_tokens` returns `false` for a single-byte-differing pair and
`true` for identical pairs, confirming the helper compiles and is callable.

**Note:** the admin token is a UUID-v4 string (not a cryptographic secret in production),
but since the harness is also used in integration test scenarios where the token is
asserted against, constant-time comparison is the correct default to prevent future
promotion of the harness into security-sensitive contexts without regression.

### AC-005 — SAP-2 column parity audit gate passes with zero new gaps
(traces to BC-2.16.013 v1.32 INV-HARNESS-ROUTE-PARITY postcondition: "every TOML column has a matching DTU generator field")

After adding the five columns, the SAP-2 probe (read `make_device()` fields, compare
against `devices` table `[[tables.columns]]` names in the TOML) finds zero column-in-TOML-with-no-DTU-equivalent gaps. The existing columns `device_id`, `hostname`,
`platform_name`, `status`, `first_seen`, `last_seen` all remain present in
`make_device()`'s output. The probe also confirms `containment_status` variant values
(`"normal"`, `"contained"`) are consistent with how the generator sets the field in
scenario-state-dependent paths.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| CrowdStrike sensor spec | `crates/prism-sensors/specs/crowdstrike.sensor.toml` | Pure data (TOML config) |
| DTU CrowdStrike generator | `crates/prism-dtu-crowdstrike/src/generator.rs` | Pure (JSON value construction) |
| DTU harness bearer auth | `crates/prism-dtu-harness/src/builder.rs` | Effectful (Axum middleware) |
| DTU harness clone auth (per-clone) | `crates/prism-dtu-harness/src/clones/*.rs` | Effectful (Axum middleware / route handler) |
| `ct_compare_tokens` helper | `crates/prism-dtu-harness/src/auth.rs` (new) or `src/util.rs` (existing) | Pure (byte comparison) |

Architecture section references:
- `architecture/module-decomposition.md` §SS-16 Spec Engine + DTU Clones
- `architecture/decisions/ADR-028-dtu-sensor-spec-authoring-and-parity-protocol.md` (DTU-TOML parity contract)

**Anchor justifications (POL-4/POL-5):**
- SS-16 owns this story's scope because both `prism-sensors/specs/` (TOML authoring) and `crates/prism-dtu-harness/` (DTU clone infrastructure) are SS-16 subsystem artifacts per ARCH-INDEX Subsystem Registry.
- No `depends_on` dependencies: this story has no product story prerequisites; it expands a shipped TOML spec and patches harness security — both are self-contained.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | A `containment_status` value neither `"normal"` nor `"contained"` appears in the generator | The TOML spec declares `column_type = "string"` with no enumeration constraint; the value passes through normalized. SAP-2 parity test uses the known generator values only. |
| EC-002 | `fan_out_batch_size` set to 5000 in TOML but harness enforces `MAX_IDS_PER_BATCH = 5_000` | The harness guard fires before the limit is exceeded; the TOML value and the harness cap must agree. If the implementer bumps the TOML to 5000, verify the harness guard is still `5_000` (not `100`). |
| EC-003 | `ct_compare_tokens` called with an empty provided token | Returns `false`; constant-time compare of length-0 vs non-zero strings is well-defined in `subtle`. |
| EC-004 | Provided token is the correct admin token with trailing whitespace | Returns `false`; the harness should NOT strip whitespace — exact byte match is required. |
| EC-005 | SAP-2 probe run against a fixture snapshot that predates this story | Probe must be run against the live TOML after the five-column addition; stale fixture snapshots are not authoritative. |

## Token Budget Estimate

| Item | Lines | Tokens (est.) |
|------|-------|--------------|
| Story spec (this file) | ~150 | ~2,100 |
| BC-2.16.013 (DTU parity BC) | ~250 | ~3,500 |
| BC-2.11.016 (E-QUERY-038 BC) | ~350 | ~5,000 |
| `crowdstrike.sensor.toml` (target TOML, ~400 lines) | ~400 | ~5,600 |
| `crates/prism-dtu-crowdstrike/src/generator.rs` (`make_device` + relevant context, ~100 lines) | ~100 | ~1,400 |
| `crates/prism-dtu-harness/src/builder.rs` (`check_bearer` + auth middleware, ~150 lines context) | ~150 | ~2,100 |
| `crates/prism-dtu-harness/src/clone_server.rs` + 6 clone files (auth sites only, ~30 lines each) | ~250 | ~3,500 |
| BC files (2 BCs) | — | ~8,500 |
| **Total estimate** | | **~31,700 tokens** |

Fits within a 100k-token agent context window (~32%). No split required.

## Tasks

- [ ] Read `crates/prism-dtu-crowdstrike/src/generator.rs` `make_device()` function and confirm exact field names and types for all five columns.
- [ ] Read `crates/prism-dtu-crowdstrike/src/routes/hosts.rs` to confirm `MAX_IDS_PER_BATCH = 5_000` constant is in place on both POST handlers.
- [ ] Add five `[[tables.columns]]` entries to the `devices` table block in `crates/prism-sensors/specs/crowdstrike.sensor.toml` (AC-001).
- [ ] Remove the two anchor comments from the `devices` block (AC-002); update/replace the `fan_out_batch_size` comment with the benchmark rationale (AC-003).
- [ ] Decide and document `fan_out_batch_size` value for `fetch_devices` (AC-003): benchmark impact of 100 vs 1000 vs 5000 using the 200 MB per-query budget; record in the TOML comment.
- [ ] Introduce `ct_compare_tokens(provided: &str, expected: &str) -> bool` helper in `crates/prism-dtu-harness/src/` using `subtle::ConstantTimeEq` (add `subtle` to `Cargo.toml` if absent — confirm workspace pin).
- [ ] Replace all 13 identified non-constant-time token comparison sites with calls to `ct_compare_tokens` (AC-004).
- [ ] Add `subtle` to `crates/prism-dtu-harness/Cargo.toml` with `default-features = false` if not already present in workspace.
- [ ] Write Red Gate tests AC-001 and AC-004 before making any source changes (TDD strict).
- [ ] Run `just iter prism-sensors` and `just iter prism-dtu-harness` to verify all tests GREEN.
- [ ] Run `just check` (full workspace) before declaring done.

## Previous Story Intelligence

**Prior cascade context:**
- `DEFECT-CSDEVICES-EMPTY-PIPELINE-001` (PR #221, merged develop@5f1b5771 2026-07-11) delivered the minimal `devices` table column set to unblock the pipeline fix. The anchor comments now pointing at this story were placed deliberately by the implementer to mark the deferral boundary.
- The five absent columns (`os_version`, `agent_version`, `external_ip`, `local_ip`, `containment_status`) were present in the DTU generator's `make_device()` from the start of the cascade; the TOML was intentionally kept minimal.
- `DRIFT-HARNESS-ADMIN-TOKEN-CT-001` was registered at D-1666 (2026-07-10) after the CWE-208 gap was spotted in the harness across all clone auth sites.
- BC-2.16.013 v1.32 contains `INV-HARNESS-ROUTE-PARITY` with the CrowdStrike `host_detail()` 6/6 field-completeness precedent from passes 29–31 of the CSDEVICES cascade. This story follows the same precedent for the `devices` table column set.

## Architecture Compliance Rules

- **ADR-028 §D1 (DTU-TOML parity):** Every `[[tables.columns]]` entry in a sensor TOML spec MUST correspond to a field emitted by the DTU generator for that table. Column names must match exactly (no snake_case vs camelCase mismatch). Adding a column with no DTU equivalent is a P1 finding per SAP-2.
- **ADR-028 §D3 (fixture-JSON parity mechanism):** The parity test must read the live generator output (or fixtures), not just the TOML. Stale fixtures or hand-crafted JSON blobs are NOT authoritative.
- **TD-VSDD-091:** Cite function names (`make_device`, `check_bearer`, `ct_compare_tokens`), not file/line numbers.
- **No `println!`:** Use `tracing::*!` with structured fields for any diagnostic logging in harness middleware.
- **`subtle` dep constraint:** Add `default-features = false` per ADR-050 pattern (though `subtle` is not `reqwest`; confirm workspace has no existing `subtle` pin before adding).
- **reqwest TLS (ADR-050):** No reqwest changes in this story; rule is not triggered.

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `subtle` | workspace-pinned (or add `^2` if absent) | `ConstantTimeEq` for constant-time byte comparison (CWE-208); `default-features = false` |
| `nextest` | workspace-pinned | `just iter prism-dtu-harness` for fast inner loop |

The `subtle` crate provides the `ConstantTimeEq` trait. If not already in the workspace
`Cargo.toml`, add `subtle = { version = "2", default-features = false }` to
`crates/prism-dtu-harness/Cargo.toml` (non-workspace dep) or to the root
`[workspace.dependencies]` table.

**Forbidden dependencies (build-time enforcement):** None added by this story. The
existing perimeter that forbids `prism-dtu-harness` from importing `prism-sensors`
(INV-PERIMETER-001, BC-2.06.017) remains in force.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-sensors/specs/crowdstrike.sensor.toml` | Modify | Add 5 `[[tables.columns]]` entries to `devices` block; remove anchor comments; update `fan_out_batch_size` comment |
| `crates/prism-dtu-harness/src/builder.rs` | Modify | Replace `check_bearer()` `token != admin_token` with `ct_compare_tokens` |
| `crates/prism-dtu-harness/src/clone_server.rs` | Modify | Replace bearer comparison site |
| `crates/prism-dtu-harness/src/clones/armis.rs` | Modify | Replace 2 bearer comparison sites |
| `crates/prism-dtu-harness/src/clones/claroty.rs` | Modify | Replace 2 bearer comparison sites |
| `crates/prism-dtu-harness/src/clones/cyberint.rs` | Modify | Replace 2 bearer comparison sites |
| `crates/prism-dtu-harness/src/clones/crowdstrike.rs` | Modify | Replace 2 bearer comparison sites |
| `crates/prism-dtu-harness/src/clones/jira.rs` | Modify | Replace 1 bearer comparison site |
| `crates/prism-dtu-harness/src/clones/pagerduty.rs` | Modify | Replace 1 bearer comparison site |
| `crates/prism-dtu-harness/src/clones/slack.rs` | Modify | Replace 1 bearer comparison site |
| `crates/prism-dtu-harness/src/auth.rs` | Create | `ct_compare_tokens` helper + unit test `test_DRIFT_HARNESS_ADMIN_TOKEN_CT_001_ct_compare_tokens_constant_time` |
| `crates/prism-dtu-harness/Cargo.toml` | Modify (if needed) | Add `subtle` dependency if not workspace-provided |
| `crates/prism-dtu-crowdstrike/tests/` | Modify or create | Red Gate test `test_BC_2_16_013_SAP2_devices_columns_parity_crowdstrike` (AC-001) |

## Open Questions

### OQ-001 — BC for constant-time harness auth comparison — RESOLVED (2026-07-11)

**Adjudication:** BC-2.16.013 amended to v1.32 (product-owner burst 2026-07-11). Option (a) selected: INV-HARNESS-ROUTE-PARITY now contains an explicit **Admin-token bearer comparison MUST use constant-time equality (`ct_compare_tokens`)** clause covering all 13 comparison sites in `prism-dtu-harness`. No new BC required — the constant-time requirement is a harness security-control invariant that belongs in INV-HARNESS-ROUTE-PARITY alongside the existing auth-model and route-surface parity obligations.

**Amended BC:** BC-2.16.013 v1.32 §Invariants INV-HARNESS-ROUTE-PARITY — added clause: every `Authorization: Bearer <token>` comparison against `admin_token` MUST use `ct_compare_tokens(provided: &str, expected: &str) -> bool` (implemented with `subtle::ConstantTimeEq`); CWE-208 mitigation; applies to all 13 sites across `src/builder.rs` (`check_bearer`), `src/clone_server.rs`, and 7 per-clone modules.

**AC-004 trace is now formal:** The AC-004 trace `(traces to BC-2.16.013 v1.32 INV-HARNESS-ROUTE-PARITY postcondition: "harness security controls do not leak timing information about valid tokens")` is now a verified postcondition in the amended BC, not an informal trace. The story may be dispatched.

### OQ-002 — `subtle` crate workspace pin decision
If `subtle` is not already in the workspace `Cargo.toml`, the implementer must decide
whether to add it at the workspace level (allowing future use by other crates) or only
as a direct dep in `prism-dtu-harness`. This is a minor workspace governance question;
the implementer may decide inline — no ADR required for a well-established `no_std`
utility crate.
