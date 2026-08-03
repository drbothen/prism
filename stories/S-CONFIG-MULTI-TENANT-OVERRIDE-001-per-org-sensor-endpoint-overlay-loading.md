---
document_type: story
story_id: S-CONFIG-MULTI-TENANT-OVERRIDE-001
title: "Per-Org Sensor Endpoint Overlay Loading — ADR-029 Hybrid Sensor Instance with Per-Org Composition Directory"
wave: 0
epic_id: wave-0-plugin-prereqs
priority: P0
status: merged
version: "v1.4"
level: "L4"
producer: story-writer
timestamp: "2026-05-23T00:00:00Z"
created: "2026-05-23"
modified: "2026-05-31 D-916 status-correction: stale ready→merged; merged PR #155 develop@3e822522 2026-05-26"
merged_via_pr: 155
merged_via_sha: "3e822522"
merged_at: "2026-05-26T19:01:58Z"
tdd_mode: strict
# BC status: All 5 BCs (BC-2.06.012–016) are status: active + lifecycle_status: active
# per BC-INDEX v5.52. Auto-promoted via POL-14 at S-CONFIG merge develop@3e822522
# (2026-05-26). PO confirmed active status at commit b8cf19e1 (2026-05-29).
# status=merged: D-916 state-correction burst (2026-05-31). Story was shipped with
# status=ready at merge; POL-14 promoted BCs correctly but story-status→done/merged
# transition was not paired with the merge event. Root cause: POL-14 auto-promotion
# covers BC draft→active but has no paired story-status→merged trigger. [process-gap]
subsystems: [SS-06, SS-16]
# Subsystem anchor justifications:
#   SS-06 (Client Configuration, prism-spec-engine config subsystem) owns the entire
#   overlay loading, merge semantics, and boot validation behavior specified in
#   BC-2.06.012–016. This is the primary subsystem.
#   SS-16 (Sensor Adapter Engine, prism-spec-engine spec_parser.rs and SpecLoader) owns
#   the `SpecLoader::load_all` extension and the new `SensorInstanceOverlay` /
#   `ResolvedSensorSpec` types. `spec_parser.rs` is the canonical module for sensor
#   spec loading per BC-2.16.001 §Architecture Anchors; extending it is an SS-16
#   responsibility.
crates_touched: [prism-spec-engine, prism-bin, prism-core, prism-sensors]
target_module: prism-spec-engine
behavioral_contracts:
  - BC-2.06.012  # Per-Tenant Overlay Loading and Merge Semantics — overlay file
                 # discovery, SensorInstanceOverlay struct, ResolvedSensorSpec map.
                 # AC-001 + AC-006 trace here.
  - BC-2.06.013  # Scalar-Only Overlay Enforcement — boot-time rejection of [[tables]],
                 # auth_type, and unrecognized fields in overlay files (E-SPEC-021,
                 # E-SPEC-020, E-SPEC-023). AC-002 traces here.
  - BC-2.06.014  # Instance Identity Resolution at Fanout — (org_id, sensor_id) →
                 # ResolvedSensorSpec O(1) hot-path lookup; FanOutTarget uses overlay
                 # base_url. AC-003 traces here.
  - BC-2.06.015  # OrgRegistry Cross-Validation at Boot — unknown customers/<slug>/
                 # directory triggers E-SPEC-022 and aborts boot. AC-004 traces here.
  - BC-2.06.016  # Error Taxonomy for Override Violations — E-SPEC-019..E-SPEC-023
                 # canonical message templates, severity, and suggestions land in
                 # error-taxonomy.md and prism-core SpecErrorCode enum. AC-005 traces here.
verification_properties: []
# VP status: None yet — test-writer will author VPs alongside Red Gate test stubs.
# Expected VPs: 5 unit/integration VPs (one per BC; see Verification Properties section).
depends_on:
  - S-WAVE5-PREP-01   # Boot orchestration chassis exists; step4_load_sensor_specs is
                      # the extension point. S-WAVE5-PREP-01 merged 2026-05-10 (PR #138).
  # Human approval gate: ADR-029 must be accepted before this story dispatches.
  # ADR-029 status is currently Proposed; it auto-promotes to Accepted when this story
  # reaches LOCAL adversarial 3-CLEAN convergence per ADR-029 §Status + ADR-021
  # promotion lifecycle. Do not dispatch to implementer while ADR-029 is Proposed.
blocks:
  - PLUGIN-MIGRATION-001-F  # Multi-tenant prism deployments that route per-org Armis/
                            # Claroty queries to per-instance endpoints depend on this
                            # story's ResolvedSensorSpec fanout wiring.
  - S-DEMO-001              # S-DEMO-001 boot step 9A iterates the ResolvedSensorSpec map
                            # (produced by this story) to construct one SpecDrivenSensorAdapter
                            # per (org, sensor) pair. S-DEMO-001 cannot start boot step 9A
                            # without the map this story delivers. See S-DEMO-001 v1.3
                            # depends_on field.
# Dependency anchor justifications:
#   depends_on S-WAVE5-PREP-01: step4_load_sensor_specs is defined in prism-bin/src/boot.rs
#   (authored in S-WAVE5-PREP-01). This story extends that step. Without the boot chassis,
#   the extension point does not exist.
#   blocks PLUGIN-MIGRATION-001-F: Wave 1 plugin migration stories that test multi-tenant
#   sensor dispatch need a working ResolvedSensorSpec fanout to exercise per-org base_url
#   routing end-to-end. This story delivers that infrastructure.
#   blocks S-DEMO-001: boot step 9A depends on ResolvedSensorSpec map from this story.
#   Parallel to S-PLUGIN-CI-001 and S-DTU-CYBERINT-AUTH-FIDELITY-001: both are S-DEMO-001
#   prerequisites but have no hard ordering between each other.
#   Parallel to S-DTU-CYBERINT-AUTH-FIDELITY-001: both wave-5 prereqs for S-DEMO-001;
#   no hard ordering between them. Dispatch sequence: S-CONFIG + S-DTU-CYBERINT in
#   parallel → S-DEMO-001 after both merge.
points: 8
# Points justification:
#   - New types: SensorInstanceOverlay, ResolvedSensorSpec (prism-spec-engine + prism-core):
#     ~1.5 days
#   - SpecLoader::load_all extension (customers/ walk + overlay merge): ~1.5 days
#   - Boot step 4 validation (5 E-SPEC codes, multi-error aggregation): ~1 day
#   - Fanout resolution wiring (FanOutTarget → ResolvedSensorSpec): ~1 day
#   - error-taxonomy.md + SpecErrorCode enum additions (5 new codes): ~0.5 day
#   - Test fixtures (customers/acme/armis.sensor.toml, customers/contoso/armis.sensor.toml,
#     customers/.gitkeep scaffold): ~0.5 day
#   - 7 Red Gate tests + backwards compat verification: included in above
#   Total: ~6 days = 8 points. Below 13-point cap.
estimated_days: 6
risk: MEDIUM
# Risk justification: The primary risk is fanout wiring — FanOutTarget currently does not
# carry a ResolvedSensorSpec reference; adding it may require threading changes through
# prism-sensors. Boot validation multi-error aggregation (BC-2.06.016 INV-ERR-003) requires
# collecting ALL overlay errors before aborting — must not short-circuit.
acceptance_criteria_count: 9
red_gate_tests: 9
estimated_passes: "2-4 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "Fanout wiring threads ResolvedSensorSpec through FanOutTarget; verify no Arc<Mutex>
    hotpath contention introduced. The map is read-only after boot (INV-OVL-006)."
  - "Boot multi-error aggregation: collect ALL overlay validation errors into a Vec
    before returning BootError::ConfigInvalid — do not short-circuit at first error
    (INV-ERR-003 requirement)."
inputs:
  - ".factory/specs/architecture/decisions/ADR-029-multi-tenant-sensor-endpoint-overrides.md"
  - ".factory/research/multi-tenant-sensor-endpoint-overrides-2026-05-23.md"
  - ".factory/specs/behavioral-contracts/BC-2.06.012-per-tenant-overlay-loading-and-merge-semantics.md"
  - ".factory/specs/behavioral-contracts/BC-2.06.013-scalar-only-overlay-enforcement.md"
  - ".factory/specs/behavioral-contracts/BC-2.06.014-instance-identity-resolution-at-fanout.md"
  - ".factory/specs/behavioral-contracts/BC-2.06.015-org-registry-cross-validation-at-boot.md"
  - ".factory/specs/behavioral-contracts/BC-2.06.016-error-taxonomy-for-override-violations.md"
  - "crates/prism-spec-engine/src/spec_parser.rs"
  - "crates/prism-bin/src/boot.rs"
  - "crates/prism-core/src/error.rs"
  - "crates/prism-sensors/src/fanout.rs"
  - ".factory/specs/prd-supplements/error-taxonomy.md"
input-hash: "[initial-stub]"
traces_to:
  - "CAP-009"
  - "ADR-029"
cycle: "v1.0.0-greenfield"
phase: 3
---

# S-CONFIG-MULTI-TENANT-OVERRIDE-001: Per-Org Sensor Endpoint Overlay Loading

**Story ID:** S-CONFIG-MULTI-TENANT-OVERRIDE-001
**Status:** ready
**Version:** v1.4
**Wave:** 0 (prereq; parallel to S-PLUGIN-CI-001; both unblock multi-tenant deployments)

---

## Authority

ADR-029 is the authoritative design document for this story. It defines the hybrid Sensor
Instance with Per-Org Composition Directory pattern: SensorInstanceOverlay struct,
ResolvedSensorSpec map, overlay merge semantics, scalar-only enforcement, and FanOutTarget
routing contract. Read it before implementing:
`.factory/specs/architecture/decisions/ADR-029-multi-tenant-sensor-endpoint-overrides.md`.

ADR-022 §B defines the boot step ordering. Boot step 4 (step4_load_sensor_specs) is the
extension point this story uses; the ResolvedSensorSpec map must be complete before step 8
(pre-traffic gate). Read the §B sequencing table before amending boot.rs:
`.factory/specs/architecture/decisions/ADR-022-production-runtime-wiring.md`.

ADR-007 §D1 defines the per-type DTU mode default registry. Per-org overlays compose with
these defaults; the overlay must not reassign a sensor's DTU mode. Read §D1 before
implementing overlay merge:
`.factory/specs/architecture/decisions/ADR-007-configurable-dtu-mode.md`.

---

## Summary

This story implements ADR-029 (Proposed, 2026-05-23): the hybrid Sensor Instance with
Per-Org Composition Directory approach to multi-tenant endpoint parameterization. The global
`<sensor>.sensor.toml` file (TYPE spec) declares sensor schema, auth_type, and a default
`base_url`. Per-org `customers/<org>/<sensor>.sensor.toml` INSTANCE overlay files declare
scalar-only tunables (primarily `base_url`) for sensors that vary per MSSP client
(Armis Centrix, Claroty on-prem). At boot step 4, overlays are discovered, validated, and
merged onto TYPE specs to produce a `ResolvedSensorSpec` per `(org_slug, sensor_id)` pair.
The fanout engine resolves `(org_id, sensor_id)` to `ResolvedSensorSpec` in O(1) at dispatch
time, routing each org's query to the correct sensor instance endpoint.

This story closes the per-tenant endpoint parameterization gap surfaced during
PLUGIN-MIGRATION-001-E architecture-clarification (D-803, 2026-05-23). It is a Wave 0
prereq for multi-tenant prism deployments where MSSP clients run Armis or Claroty on
distinct instances. SaaS sensors (CrowdStrike, Cyberint) require zero overlay files.

**Human approval gate:** ADR-029 must be accepted by the architect before this story
is dispatched to the implementer. ADR-029 auto-promotes from Proposed → Accepted when
this story reaches LOCAL adversarial 3-CLEAN convergence per ADR-029 §Status.

---

## Behavioral Contracts

| BC ID | Version | Title | Role in This Story |
|-------|---------|-------|-------------------|
| BC-2.06.012 | 1.0 | Per-Tenant Overlay Loading and Merge Semantics | Core overlay discovery, `SensorInstanceOverlay` + `ResolvedSensorSpec` types, scalar-merge logic. AC-001 + AC-006 implement it. |
| BC-2.06.013 | 1.0 | Scalar-Only Overlay Enforcement — Boot-Time Rejection of Schema Fields in Overlay Files | `[[tables]]` / auth_type / unrecognized-field rejection (E-SPEC-020/021/023). AC-002 implements it. |
| BC-2.06.014 | 1.0 | Instance Identity Resolution at Fanout — (org_id, sensor_id) Tuple Resolves to ResolvedSensorSpec | O(1) hot-path lookup in fanout; FanOutTarget wiring; per-org `base_url` used at dispatch. AC-003 implements it. |
| BC-2.06.015 | 1.0 | OrgRegistry Cross-Validation at Boot — Unknown Overlay Directory Triggers E-SPEC-022 | Boot rejection of `customers/<slug>/` dirs not in OrgRegistry. AC-004 implements it. |
| BC-2.06.016 | 1.0 | Error Taxonomy for Per-Org Overlay Override Violations (E-SPEC-019 through E-SPEC-023) | Canonical error codes, message templates, and `SpecErrorCode` enum variants. AC-005 implements it. |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~4,000 |
| BC files (5 BCs: BC-2.06.012–016, full reads) | ~10,000 |
| ADR-029 (full read, merge semantics + consequences) | ~4,000 |
| prism-spec-engine/src/spec_parser.rs (full read) | ~4,000 |
| prism-bin/src/boot.rs (boot step 4 extension point) | ~2,500 |
| prism-core/src/error.rs (SpecErrorCode additions) | ~1,500 |
| prism-sensors/src/fanout.rs (FanOutTarget wiring) | ~2,000 |
| error-taxonomy.md (E-SPEC section, partial read) | ~1,500 |
| Test fixture files (customers/ overlay TOMLs) | ~500 |
| **Total estimate** | **~30,000** |
| Agent context window (claude-sonnet-4-6) | ~200,000 |
| **% of context window** | **~15%** |

Well within the 20-30% target.

---

## Acceptance Criteria

Each AC traces to its BC clause and includes the Red Gate test name per SID-1 §5.

### AC-001: Overlay file discovery and scalar merge (traces to BC-2.06.012 postconditions 1–7 — overlay parsed into SensorInstanceOverlay; ResolvedSensorSpec produced by merging overlay scalars onto TYPE spec; indexed by (org_slug, sensor_id))

At boot step 4, after all TYPE specs are loaded from the root `sensor_specs_dir`,
`SpecLoader::load_all` walks `customers/<org_slug>/` subdirectories and discovers
`<sensor_id>.sensor.toml` files. For each file, a `SensorInstanceOverlay` struct is parsed
with fields `extends: String`, `instance_id: String`, `base_url: Option<String>`, and
`rate_limit_hints: Option<RateLimitHints>`. A `ResolvedSensorSpec` is produced by copying
the TYPE spec identified by `extends` and overlaying: `base_url` (if present in overlay),
`rate_limit_hints.requests_per_second` (if present), `rate_limit_hints.burst_size` (if
present). Fields `[[tables]]`, `auth_type`, `version`, and `sensor_id` are NEVER overridden
— they are always from the TYPE spec. The `ResolvedSensorSpec` carries provenance metadata
indicating which scalar fields came from the overlay. The resolved spec is indexed in memory
by `(org_slug, sensor_id)` for O(1) lookup. Each loaded overlay is logged at `info` level
with `event_type = "overlay.loaded"`, `org_slug`, `sensor_id`, `instance_id`.

**Red Gate Test:** `test_BC_2_06_012_overlay_discovered_and_merged`

This test: creates a temp sensor_specs_dir with `armis.sensor.toml` (TYPE spec) and
`customers/acme/armis.sensor.toml` (overlay with `base_url = "https://armis.acme-corp.io"`);
asserts the ResolvedSensorSpec for `(acme, armis)` has `base_url = "https://armis.acme-corp.io"`
and TYPE spec `[[tables]]` unchanged.

### AC-002: Scalar-only enforcement — [[tables]] and unrecognized fields fail boot (traces to BC-2.06.013 postconditions failure paths — [[tables]] triggers E-SPEC-021; unrecognized field triggers E-SPEC-023; instance_id mismatch triggers E-SPEC-020; all exit code 2)

Any per-org overlay file that contains:
- `[[tables]]` blocks → `E-SPEC-021` (boot hard error, exit 2) with message matching the
  canonical template from BC-2.06.016: `"Per-org overlay '{file}' for instance '{instance_id}'
  contains [[tables]] blocks. Schema overrides are forbidden in overlay files (ADR-029)."`
- An unrecognized scalar field (e.g., `auth_type`, `name`, `secret_key`) → `E-SPEC-023`
  (boot hard error, exit 2) with the unrecognized field name in the error context.
- An `instance_id` value that does not match `{sensor_id}@{org_slug}` (derived from the
  filename stem and parent directory name) → `E-SPEC-020` (boot hard error, exit 2).

Validation runs BEFORE the merge step (INV-SCALAR-002). A single invalid overlay file
fails the entire boot (INV-SCALAR-003). Multiple errors in the same overlay file are
collected and reported together (multi-error pattern per BC-2.06.016 INV-ERR-003).

**Red Gate Test:** `test_BC_2_06_013_tables_in_overlay_rejects_with_e_spec_021`

This test: creates overlay file containing `[[tables]]`; asserts `SpecLoader::load_all`
returns `Err` carrying `E-SPEC-021` code; process-level test asserts exit code 2.

Additional Red Gate tests (same family):
- `test_BC_2_06_013_unrecognized_field_rejects_with_e_spec_023`
- `test_BC_2_06_013_wrong_instance_id_rejects_with_e_spec_020`

### AC-003: Instance identity resolution at fanout uses overlay base_url (traces to BC-2.06.014 postcondition Case A — ResolvedSensorSpec map lookup at (org_slug, sensor_id); overlay base_url used at HTTP dispatch; Case B — SaaS sensor falls back to TYPE spec base_url)

When the query engine dispatches a `FanOutTarget` for `(org_id, sensor_id)`:

**Case A (overlay exists):** `OrgRegistry.slug_for(org_id)` resolves to `OrgSlug`; the
`ResolvedSensorSpec` map is looked up at `(org_slug, sensor_id)` in O(1); the
`FanOutTarget`'s effective `SensorSpec` uses the overlay `base_url`. The sensor adapter's
HTTP client dispatches to the per-org endpoint. Instance identity is logged as
`instance_id = "{sensor_id}@{org_slug}"` in the fanout span.

**Case B (no overlay):** Map lookup returns `None`; the fanout target uses the TYPE spec
`base_url`. Instance identity logged as bare `{sensor_id}`. This is the backwards-compatible
path for SaaS sensors and single-tenant deployments.

The `CredentialResolver` is NOT changed — credential lookup continues by `(org_id,
sensor_id)` independently of endpoint resolution (INV-FANOUT-001). Resolution is O(1) with
no filesystem I/O on the hot path (INV-FANOUT-002).

**Red Gate Test:** `test_BC_2_06_014_resolved_spec_overlays_base_url`

This test: sets up ResolvedSensorSpec map with `(acme, armis)` → overlay `base_url`;
dispatches a FanOutTarget for `(acme_org_id, armis)`; asserts the HTTP mock receives a
request to `https://armis.acme-corp.io` (not the TYPE spec default URL).

### AC-004: OrgRegistry cross-validation — unknown customers/<slug>/ directory aborts boot with E-SPEC-022 (traces to BC-2.06.015 postcondition failure path — OrgRegistry::slug_exists returns false; E-SPEC-022 emitted; exit code 2)

During boot step 4, every `customers/<slug>/` directory entry is cross-checked against
`OrgRegistry` via `OrgRegistry::slug_exists(slug)`. If the directory name does not match
any registered org slug, boot fails immediately with `E-SPEC-022` (exit code 2), message:
`"Per-org overlay directory 'customers/{slug}/' references org slug '{slug}' which is not
registered in OrgRegistry. Check for typos or register the org in prism.toml [[orgs]]."`.

The check fires even for empty directories (INV-COMPAT-003). Only subdirectory entries
trigger the slug lookup — `customers/.gitkeep` (a plain file) is not treated as a slug
(INV-COMPAT-004). Boot step 3 (`OrgRegistry` init) always completes before step 4, so the
registry is fully populated when this validation runs (INV-COMPAT-002).

Multiple unregistered directories have all `E-SPEC-022` errors collected and reported
together (multi-error pattern).

**Red Gate Test:** `test_BC_2_06_015_unknown_org_dir_aborts_boot_with_e_spec_022`

This test: creates `customers/unknown-org/armis.sensor.toml` with OrgRegistry containing
only `acme`; asserts `SpecLoader::load_all` returns `Err` carrying `E-SPEC-022` for
slug `unknown-org`.

### AC-005: Error taxonomy and SpecErrorCode enum additions (traces to BC-2.06.016 postconditions — error-taxonomy.md gains rows for E-SPEC-019..E-SPEC-023; prism-core SpecErrorCode gains ESpec019..ESpec023 variants)

Five new entries are appended to `.factory/specs/prd-supplements/error-taxonomy.md` in the
SPEC namespace, E-SPEC-019 through E-SPEC-023, matching the canonical definitions in
BC-2.06.016 §Error Catalog:

| Code | Condition | Severity |
|------|-----------|----------|
| E-SPEC-019 | Overlay `extends` references unknown sensor TYPE | broken/validation |
| E-SPEC-020 | `instance_id` does not match `{sensor_id}@{org_slug}` | broken/validation |
| E-SPEC-021 | Overlay contains `[[tables]]` blocks | broken/validation |
| E-SPEC-022 | `customers/<slug>/` dir references unknown org slug | broken/validation |
| E-SPEC-023 | Overlay contains unrecognized scalar field | broken/validation |

Each row includes: code, category, severity, message template, suggestion, and BC
enforcement reference (matching BC-2.06.016 §Error Catalog canonical form).

`prism-core/src/error.rs` gains `SpecErrorCode::ESpec019` through `SpecErrorCode::ESpec023`
variants (parallel to the existing `SpecErrorCode::ESpec017` variant introduced in
S-PLUGIN-PREREQ-D). The error messages emitted at boot match the canonical templates in
BC-2.06.016 exactly (INV-ERR-002 — no credential values in error messages).

**Red Gate Test:** `test_BC_2_06_016_error_messages_match_canonical_templates`

This test: triggers each of the 5 error conditions; captures the structured error output
(JSON `BootError::ConfigInvalid` block); asserts each error carries the correct `code`
field (`E-SPEC-019`..`E-SPEC-023`) and that the `message` matches the canonical template
from error-taxonomy.md (test reads the taxonomy file and validates against it, test-driven
validation).

### AC-006: Backwards compatibility — single-tenant deployments with no customers/ directory are unaffected (traces to BC-2.06.012 postcondition — absent customers/ dir produces zero ResolvedSensorSpec entries; boot continues normally; EC-012-001 and EC-012-002 coverage)

Existing single-tenant prism deployments that have no `customers/` directory (or have only
`customers/.gitkeep` and no subdirectories) must continue to work unchanged:
- `SpecLoader::load_all` produces zero `ResolvedSensorSpec` entries.
- Boot step 4 completes successfully with the zero-entry overlay map.
- All fanout targets fall back to TYPE spec `base_url` (Case B, BC-2.06.014).
- No error is emitted.
- All existing single-tenant integration tests pass without modification.

**Red Gate Test:** `test_BC_2_06_012_backcompat_no_customers_dir_uses_type_spec_only`

This test: creates a sensor_specs_dir with NO `customers/` subdirectory; asserts
`SpecLoader::load_all` returns no ResolvedSensorSpec entries and no error; asserts
the fanout for any (org_id, sensor_id) uses the TYPE spec base_url.

### AC-007: Example Armis overlay and two-org fixture (traces to BC-2.06.012 test vector — two-org same-sensor produces two independent ResolvedSensorSpec entries; schemas identical from TYPE)

Two example overlay fixture files are added to the repository:
- `crates/prism-sensors/specs/customers/acme/armis.sensor.toml` — overlay with
  `extends = "armis"`, `instance_id = "armis@acme"`,
  `base_url = "https://armis.acme-corp.io"`
- `crates/prism-sensors/specs/customers/contoso/armis.sensor.toml` — overlay with
  `extends = "armis"`, `instance_id = "armis@contoso"`,
  `base_url = "https://armis.contoso.com"`
- `crates/prism-sensors/specs/customers/.gitkeep` — ensures the directory is tracked
  by git even when no prod overlays are present.

A test asserts that loading both fixture overlays produces two independent
`ResolvedSensorSpec` entries, each with their own `base_url`, and identical `[[tables]]`
schemas from the TYPE spec.

**Red Gate Test:** `test_S_CONFIG_MULTI_TENANT_OVERRIDE_001_007_two_org_overlays_produce_distinct_resolved_specs`

### AC-008: Paper-fix resistance — injected base_url is actually consumed by the HTTP dispatch layer (D-823 / SAP-3)

Given org A has overlay `base_url = "https://armis.acme-corp.io"` and org B has overlay
`base_url = "https://armis.contoso.com"`, when `FanOutTarget` dispatches for `(org_A, armis)`,
the HTTP request goes to `https://armis.acme-corp.io` (not the TYPE spec default). When
`FanOutTarget` dispatches for `(org_B, armis)`, the HTTP request goes to
`https://armis.contoso.com`.

This AC explicitly verifies that the `base_url` value is NOT merely stored in the
`ResolvedSensorSpec` map — it must actually reach the HTTP client's `reqwest::Client`
(or whatever the production HTTP dispatch layer is). A test that only asserts the map
CONTAINS the overlay value without verifying the HTTP request destination is a paper-fix
(D-823 paper-fix detection; SAP-3-candidate for future adversary standing probe).

This AC also covers the negative: org B's HTTP dispatch MUST NOT use org A's `base_url`
(no cross-tenant URL leakage).

(traces to BC-2.06.014 postcondition Case A — fanout uses overlay base_url at HTTP dispatch,
not just at spec construction time)

**Red Gate Test:** `test_S_CONFIG_PROD_CONSUMER_READS_INJECTED_BASE_URL`

This test: sets up two mock HTTP servers at distinct addresses; configures org A → server A,
org B → server B via overlays; dispatches fanout for each org; asserts server A received
exactly org A's requests and server B received exactly org B's requests. Zero requests from
org A to server B and vice versa.

### AC-009: DTU multi-tenant emulation — per-tenant routing testable against DTU clones (ADR-031 DTU=true-DTU)

Under the DTU=true-DTU principle (ADR-031), per-tenant overlay changes must be exercisable
against DTU clones, not only against mock HTTP servers. Specifically:

**Case A (single DTU, per-org base_url pointing to same DTU):** If both `org_A` and `org_B`
overlay the `base_url` to the same DTU clone address (the typical demo scenario), both
orgs should receive valid data. This proves the overlay plumbing works end-to-end with
a real DTU.

**Case B (gap acknowledged):** The `prism-dtu-demo-server` or individual DTU clones
(`prism-dtu-armis`, `prism-dtu-claroty`) do NOT currently support binding multiple
network addresses to simulate different tenant instances. This means the full "org A to
instance A, org B to instance B" routing cannot be tested against separate DTU processes
in this story. This gap is documented here and deferred to
`S-DEMO-MULTI-TENANT-DTU-001` (new story stub — add to STORY-INDEX after this story
dispatches; P2).

**Current AC scope:** Add a comment to `test_BC_2_06_014_resolved_spec_overlays_base_url`
(AC-003 Red Gate test) that documents the DTU limitation: "Full per-org DTU routing
tested in S-DEMO-MULTI-TENANT-DTU-001; this test verifies the overlay plumbing using
a mock HTTP server per AC-008." This ensures the limitation is visible to implementers
and reviewers without blocking this story's dispatch.

(traces to BC-2.16.013 postcondition — DTU parity discipline; ADR-031 §D5 validation:
parity tests should exercise real DTU paths where possible)

**Red Gate Test:** `test_S_CONFIG_DTU_BASE_URL_OVERLAY_ROUTES_TO_CORRECT_DTU_INSTANCE`

This test: starts one `prism-dtu-armis` clone; configures org A overlay `base_url` to
the clone address; dispatches fanout for `(org_A, armis)` via PipelineExecutor with the
ResolvedSensorSpec; asserts non-empty data returned from the DTU. Also asserts that
org B (no overlay for armis) falls back to TYPE spec base_url (which will NOT match the
DTU address, so the fallback path returns an HTTP error — verifying the isolation).

---

## Tasks

High-level TDD order. Full task breakdown happens during ready-for-implementation refinement.

1. **Read source files first** — `spec_parser.rs` (SpecLoader::load_all current implementation),
   `boot.rs` (step4_load_sensor_specs extension point), `error.rs` (SpecErrorCode enum),
   `fanout.rs` (FanOutTarget struct), `error-taxonomy.md` (SPEC section, last E-SPEC-NNN),
   `customers/.gitkeep` current state. Establish exact function signatures and struct layouts
   before writing any new code.

2. **Write Red Gate tests (stub phase — ALL must FAIL before implementation):**
   - `test_BC_2_06_012_overlay_discovered_and_merged` (AC-001)
   - `test_BC_2_06_013_tables_in_overlay_rejects_with_e_spec_021` (AC-002)
   - `test_BC_2_06_013_unrecognized_field_rejects_with_e_spec_023` (AC-002)
   - `test_BC_2_06_013_wrong_instance_id_rejects_with_e_spec_020` (AC-002)
   - `test_BC_2_06_014_resolved_spec_overlays_base_url` (AC-003)
   - `test_BC_2_06_015_unknown_org_dir_aborts_boot_with_e_spec_022` (AC-004)
   - `test_BC_2_06_016_error_messages_match_canonical_templates` (AC-005)
   - `test_BC_2_06_012_backcompat_no_customers_dir_uses_type_spec_only` (AC-006)
   - `test_S_CONFIG_MULTI_TENANT_OVERRIDE_001_007_two_org_overlays_produce_distinct_resolved_specs` (AC-007)
   - `test_S_CONFIG_PROD_CONSUMER_READS_INJECTED_BASE_URL` (AC-008 — paper-fix resistance)
   - `test_S_CONFIG_DTU_BASE_URL_OVERLAY_ROUTES_TO_CORRECT_DTU_INSTANCE` (AC-009 — DTU emulation)

3. **Add `SensorInstanceOverlay` and `ResolvedSensorSpec` types** to
   `prism-spec-engine/src/spec_parser.rs` (or a new `overlay.rs` module). Both types must
   have `#[non_exhaustive]` per the compile-fail gate (`EXPECTED` count in `ci.yml` must
   be incremented accordingly). `SensorInstanceOverlay` fields: `extends: String`,
   `instance_id: String`, `base_url: Option<String>`, `rate_limit_hints: Option<RateLimitHints>`.
   `ResolvedSensorSpec` wraps `SensorSpec` with provenance metadata (which fields came from
   overlay vs TYPE spec).

4. **Add `SpecErrorCode::ESpec019`..`ESpec023` variants** to `prism-core/src/error.rs`
   (parallel to `ESpec017` from S-PLUGIN-PREREQ-D). Update error-taxonomy.md with 5 new
   rows (append-only per POL-1/DF-030). **Verify E-SPEC-018 is already allocated to
   `TimestampParseFailure` (BC-2.16.013) — do NOT reuse or overwrite it.**

5. **Implement overlay structural validator** — the allowed scalar field enumeration
   (extends, instance_id, base_url, timeout_secs, rate_limit_hints and its sub-fields).
   Rejection before merge (INV-SCALAR-002). Collect ALL errors per overlay file before
   returning (multi-error aggregation per BC-2.06.016 INV-ERR-003).

6. **Extend `SpecLoader::load_all`** — after loading all TYPE specs (root flat scan),
   walk `customers/` subdirectory recursively. For each `<org_slug>/<sensor_id>.sensor.toml`:
   - Cross-check `<org_slug>` against OrgRegistry (E-SPEC-022).
   - Run structural validator (E-SPEC-019, E-SPEC-020, E-SPEC-021, E-SPEC-023).
   - If valid, merge overlay scalars onto TYPE spec to produce `ResolvedSensorSpec`.
   - Index by `(org_slug, sensor_id)`.
   - Log at info level with `event_type = "overlay.loaded"` per BC-2.06.012 postcondition.
   **Add BC-2.16.002 Structured Event Catalog row for `overlay.loaded` event
   (SAP-1 standing adversary probe — any new `event_type = ...` site requires a catalog row).**

7. **Wire fanout resolution** — in `prism-sensors/src/fanout.rs` (or the equivalent
   fanout engine entry point), at dispatch time: call `OrgRegistry.slug_for(org_id)` →
   `OrgSlug`; lookup `(org_slug, sensor_id)` in the boot-time `ResolvedSensorSpec` map;
   provide the resolved spec (or TYPE spec fallback) to the HTTP client. This is O(1),
   no filesystem I/O on the hot path (INV-FANOUT-002). `CredentialResolver` NOT changed.

8. **Add fixture files** —
   `crates/prism-sensors/specs/customers/.gitkeep`,
   `crates/prism-sensors/specs/customers/acme/armis.sensor.toml`,
   `crates/prism-sensors/specs/customers/contoso/armis.sensor.toml`.
   Update `armis.sensor.toml` TYPE spec `base_url` comment to document per-org override path.

9. **Pre-push gate** — `just check` GREEN workspace-wide. Verify `EXPECTED` count in
   `ci.yml` for `#[non_exhaustive]` compile-fail gate is updated for new public types.
   No `--no-verify`. No `println!` in production code.

---

## Previous Story Intelligence

N/A for this stub — first story in the multi-tenant endpoint override epic. Full previous
story intelligence populated during ready-for-implementation refinement.

Key lessons from adjacent stories that apply:

- **BC-2.16.002 Structured Event Catalog discipline (SAP-1, PG-LP11-001):** Every new
  `event_type = "..."` emission site in `crates/**/*.rs` must have a corresponding row in
  BC-2.16.002 §Postconditions (Canonical Structured Event Catalog). The `overlay.loaded`
  event (AC-001) requires a new catalog row. Do not ship without it — adversary SAP-1 sweep
  catches every uncatalogued emission site.

- **`#[non_exhaustive]` discipline:** All new public TOML-deserialized types
  (`SensorInstanceOverlay`, `ResolvedSensorSpec`) require `#[non_exhaustive]`. The
  `ci.yml` `EXPECTED=35` constant must be bumped by the number of new types. Verify the
  compile-fail gate crate at `tests/external/non-exhaustive-violation/`.

- **Multi-error aggregation pattern (BC-2.06.005 precedent):** Boot-time config validation
  collects ALL errors before failing. Do not return at the first invalid overlay — scan
  all overlay files, collect all errors, then emit the full error report and exit 2.

- **OrgRegistry boot sequencing (ADR-022 §B):** Boot step 3 (OrgRegistry init) always
  precedes step 4 (sensor spec loading). OrgRegistry is fully populated when the overlay
  validator runs. Do not add any lazy-init logic.

- **E-SPEC-018 is already allocated:** E-SPEC-018 = `TimestampParseFailure`
  (ADR-028/BC-2.16.013). The implementer MUST use E-SPEC-019..E-SPEC-023 for the five
  overlay violation codes. This is enforced by BC-2.06.016 INV-ERR-005 and the
  Source-of-Truth Precedence Rule #3 in CLAUDE.md.

- **reqwest::Client timeout:** Any new HTTP client construction in this story must use
  `.timeout(Duration::from_secs(30))` per CLAUDE.md conventions. (This story doesn't
  add HTTP clients, but the fanout wiring change touches the HTTP dispatch path — verify
  the existing client has the timeout set.)

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| `[[tables]]` in overlay files must be rejected at boot with E-SPEC-021 (exit 2) | BC-2.06.013 INV-SCALAR-004 + ADR-029 §Decision Drivers (TOML array-replace footgun) | AC-002 Red Gate test |
| Root sensor_specs_dir scan remains flat (non-recursive); only `customers/` is walked recursively | BC-2.06.012 INV-OVL-004 | Unit test verifying flat scan of root dir |
| `[[tables]]` schema is immutable per overlay — all tenants see identical schema | BC-2.06.012 INV-OVL-001 | AC-007 Red Gate test (two orgs, same TYPE tables) |
| `auth_type` is immutable per overlay — cannot be changed per org | BC-2.06.012 INV-OVL-002 | E-SPEC-023 rejection path (auth_type is a forbidden field) |
| `OrgRegistry` boot step 3 precedes overlay walk (boot step 4) | BC-2.06.015 INV-COMPAT-002 + ADR-022 §B | Boot sequence ordering in boot.rs |
| `ResolvedSensorSpec` map is read-only after boot — no mutation on hot path | BC-2.06.012 INV-OVL-006 | Arc<HashMap> (no Mutex on read path); or ArcSwap per ADR-007 if hot-reload is wired |
| All five E-SPEC codes are FATAL/broken/validation — no downgrade to warning | BC-2.06.016 INV-ERR-001 | Every error path asserts exit code 2 |
| Error messages MUST NOT include credential values | BC-2.06.016 INV-ERR-002 + AD-017 | Code review + security-reviewer pass |
| New `tracing::*!(event_type=...)` sites require BC-2.16.002 catalog row | SAP-1 + PG-LP11-001 | Adversary SAP-1 sweep on every pass |
| New public TOML-deserialized types require `#[non_exhaustive]` | CLAUDE.md + S-PLUGIN-PREREQ-C compile-fail gate | `ci.yml EXPECTED=35` count bump; compile-fail gate test |
| E-SPEC-018 is ALREADY ALLOCATED to TimestampParseFailure (ADR-028/BC-2.16.013) | BC-2.06.016 INV-ERR-005; Source-of-Truth Precedence Rule #3 | Do not define `SpecErrorCode::ESpec018` for overlay violations |

### Forbidden Dependencies

`prism-spec-engine` overlay validation MUST NOT gain a dependency on `prism-sensors` to
resolve org slugs — the `OrgRegistry` reference must be passed in from the caller
(dependency-injection, not import). If `prism-spec-engine` gains a direct dep on
`prism-sensors`, the build MUST fail (perimeter violation — same pattern as the
`prism-query` security perimeter in `tests/external/perimeter-violation/`).

`prism-bin/src/boot.rs` already imports both `prism-spec-engine` and `prism-core` (via
S-WAVE5-PREP-01). The OrgRegistry instance from step 3 must be threaded into step 4 as a
parameter — not re-initialized.

---

## Library and Framework Requirements

| Library | Version | Justification |
|---------|---------|---------------|
| `toml` | per `prism-spec-engine/Cargo.toml` workspace pin | Overlay file parsing; do not add separate toml dep |
| `serde` | per workspace pin | Overlay struct deserialization (`#[derive(Deserialize)]`) |
| `tracing` | per workspace pin | `event_type = "overlay.loaded"` structured event |

Do NOT pin new Rust library versions. Use workspace-inherited versions. Do NOT use
`config-rs` or `figment` for overlay merging — the explicit scalar-field allowlist is
implemented as a custom deserializer or post-parse validator, not as a layered config
source (ADR-029 §Decision: the TOML array-replace footgun is the reason for NOT using
config-rs layering for overlays).

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-spec-engine/src/spec_parser.rs` | MODIFY | Add `SensorInstanceOverlay`, `ResolvedSensorSpec` types; extend `SpecLoader::load_all` for `customers/` walk + overlay merge |
| `crates/prism-spec-engine/src/overlay.rs` | CREATE (optional) | Isolate overlay types + validator here if spec_parser.rs becomes too large |
| `crates/prism-bin/src/boot.rs` | MODIFY | Extend `step4_load_sensor_specs` to pass OrgRegistry into SpecLoader and handle overlay validation errors |
| `crates/prism-core/src/error.rs` | MODIFY | Add `SpecErrorCode::ESpec019`..`ESpec023` variants |
| `crates/prism-sensors/src/fanout.rs` | MODIFY | Wire `(org_slug, sensor_id)` → `ResolvedSensorSpec` lookup; pass resolved spec to HTTP client |
| `.factory/specs/prd-supplements/error-taxonomy.md` | MODIFY | Append E-SPEC-019..E-SPEC-023 rows (append-only per POL-1/DF-030) |
| `.factory/specs/behavioral-contracts/BC-2.16.002-*.md` | MODIFY | Add `overlay.loaded` event row to Structured Event Catalog (SAP-1) |
| `crates/prism-sensors/specs/customers/.gitkeep` | CREATE | Ensures `customers/` directory is git-tracked; absent dir = zero overlays (EC-012-001) |
| `crates/prism-sensors/specs/customers/acme/armis.sensor.toml` | CREATE | Example overlay fixture: `base_url = "https://armis.acme-corp.io"` |
| `crates/prism-sensors/specs/customers/contoso/armis.sensor.toml` | CREATE | Example overlay fixture: `base_url = "https://armis.contoso.com"` |
| `crates/prism-sensors/specs/armis.sensor.toml` | MODIFY | Update `base_url` comment to document per-org override path per ADR-029 §Consequences |
| `crates/prism-sensors/specs/claroty.sensor.toml` | MODIFY | Same base_url comment update |
| `crates/prism-spec-engine/tests/overlay_loading_tests.rs` | CREATE | Red Gate tests for AC-001 through AC-009 (includes AC-008 paper-fix resistance + AC-009 DTU emulation) |
| `tests/external/non-exhaustive-violation/src/lib.rs` | MODIFY | Assert new `SensorInstanceOverlay` / `ResolvedSensorSpec` types are non_exhaustive (if needed); bump EXPECTED count in `ci.yml` |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `customers/` directory absent entirely | No overlays loaded; zero `ResolvedSensorSpec` entries; boot succeeds (EC-012-001) |
| EC-002 | `customers/.gitkeep` only (no subdirectories) | Same as EC-001; zero overlays; boot succeeds (EC-012-002) |
| EC-003 | Overlay `extends` references a sensor_id that has no TYPE spec | E-SPEC-019 boot hard error; message includes `extends` value and suggestion to add TYPE spec |
| EC-004 | Overlay file contains syntactically invalid TOML | E-SPEC-001 parse error (existing error code); boot fails exit 2 with file path and line number |
| EC-005 | Overlay `instance_id` set to identity fields (e.g., `sensor_id = "armis"`) | `sensor_id` is a forbidden field → E-SPEC-023 |
| EC-006 | Two orgs use overlays for the same sensor (`acme/armis.sensor.toml` + `contoso/armis.sensor.toml`) | Two independent `ResolvedSensorSpec` entries; schemas identical from TYPE; no interference (EC-012-006) |
| EC-007 | Overlay specifies same `base_url` as TYPE spec (no actual change) | Boot succeeds; overlay merged (no-op merge); no warning required |
| EC-008 | Stale `customers/oldcorp/` directory after org removed from `prism.toml` | E-SPEC-022 at boot; operator must remove or re-add org (EC-015-001) |
| EC-009 | SaaS sensor (CrowdStrike) for org `acme` — no overlay file | Fanout uses TYPE spec `base_url = "https://api.crowdstrike.com"`; instance_id logged as bare `crowdstrike` (EC-014-002) |
| EC-010 | Config reload (`reload_config`) adds new `globex/armis.sensor.toml` overlay | After reload, `ResolvedSensorSpec` map rebuilt atomically; subsequent fanout for `(globex, armis)` uses new overlay `base_url` (EC-012-007, EC-014-005) |
| EC-011 | E-SPEC-021 and E-SPEC-023 both fire on same overlay file | Both errors collected; multi-error boot failure report; exit 2 (EC-016-001) |
| EC-012 | Empty `customers/<slug>/` registered directory (no .sensor.toml files) | OrgRegistry cross-validation passes (slug is registered); zero overlays from this dir; boot succeeds (EC-015-003 / EC-015-005) |

---

## Cross-References

**Implements:** ADR-029 v1.1 (Proposed — gated on human approval before implementation dispatch)

**Implements BCs:** BC-2.06.012, BC-2.06.013, BC-2.06.014, BC-2.06.015, BC-2.06.016 (all status: draft → active on this story's PR merge per POL-14)

**Surfaced by:** PLUGIN-MIGRATION-001-E architecture-clarification (D-803, 2026-05-23) — the gap between per-tenant credential resolution (existing) and per-tenant endpoint resolution (missing) was identified during the PLUGIN-MIGRATION-001-E cascade.

**Parallel to:** S-PLUGIN-CI-001 and S-DTU-CYBERINT-AUTH-FIDELITY-001 (all three are S-DEMO-001
prerequisites; no hard ordering between them — dispatch in parallel where possible).

**Critical-path dispatch order:** S-CONFIG-MULTI-TENANT-OVERRIDE-001 + S-DTU-CYBERINT-AUTH-FIDELITY-001
(parallel) → S-DEMO-001 (after both merge) → S-DEMO-002.

**Depends on:** S-WAVE5-PREP-01 (merged 2026-05-10, PR #138) — boot chassis with `step4_load_sensor_specs` extension point

**Follow-up story (deferred):** `S-CONFIG-MULTI-TENANT-OVERRIDE-002` — `prism config show --sensor <instance_id>` provenance-aware rendering (per ADR-029 §Follow-Up Actions; not in scope here)

**New error codes introduced:** E-SPEC-019, E-SPEC-020, E-SPEC-021, E-SPEC-022, E-SPEC-023

**New event type introduced:** `overlay.loaded` (requires BC-2.16.002 catalog row — SAP-1)

**OrgRegistry dependency:** BC-2.21.001 (OrgRegistry Initialization — boot step 3; must complete before this story's step 4 extension runs; OrgRegistry is passed as a parameter into SpecLoader::load_all)

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| v1.4 | 2026-08-02 | story-writer | Added ## Authority section (DRIFT-STORY-AUTHORITY-ABSENT-CORPUS-001 Round 6, D-2084). Synced stale `**Version:**` pseudo-field from v1.2 to v1.4 to match frontmatter (TD-VSDD-060 sibling-sweep correction, orchestrator-authorized; v1.3 row still absent from changelog — pre-existing drift registered for spec-steward). |
| v0.1 | 2026-05-23 | story-writer | Initial stub — D-803 Burst 4/4; anchored to ADR-029 v1.1 + 5 new BCs BC-2.06.012–016; 7 ACs with Red Gate test names per SID-1 §5; subsystems SS-06 + SS-16; wave-0 prereq parallel to S-PLUGIN-CI-001. |
| v0.2 | 2026-05-24 | story-writer | F-LP4-MED-004 closure — swept stale `ci.yml EXPECTED=32` → `EXPECTED=35` at Architecture Compliance Rules (§#[non_exhaustive] discipline) and Architecture Compliance Rules table row (compile-fail gate enforcement column). Fix-burst-3 bumped ci.yml but missed story body citations. POL-29 sibling-sweep: no other EXPECTED=32 citations in this story file. |
| v1.1 | 2026-05-29 | story-writer | Pre-dispatch refinement per orchestrator direction 2026-05-29: (1) Added AC-008 (paper-fix resistance — injected base_url actually consumed at HTTP dispatch layer, D-823 / SAP-3-candidate; Red Gate: test_S_CONFIG_PROD_CONSUMER_READS_INJECTED_BASE_URL); (2) Added AC-009 (DTU emulation gap documented under ADR-031 DTU=true-DTU principle — single-DTU emulation described; full multi-instance DTU gap surfaced as S-DEMO-MULTI-TENANT-DTU-001 stub needed; Red Gate: test_S_CONFIG_DTU_BASE_URL_OVERLAY_ROUTES_TO_CORRECT_DTU_INSTANCE); (3) Updated blocks: to include S-DEMO-001 (boot step 9A depends on ResolvedSensorSpec map from this story per S-DEMO-001 v1.3 depends_on); (4) Added dispatch order note: S-CONFIG + S-DTU-CYBERINT parallel → S-DEMO-001 after both merge; (5) acceptance_criteria_count 7→9, red_gate_tests 7→9. Status remains draft: BC-2.06.012–016 are draft status; Spec-First Gate S-7.01 requires non-empty behavioral_contracts with active (not draft) BCs before status=ready. |
| v1.2 | 2026-05-29 | story-writer | D-849-prep: status flipped `draft → ready` per BC-2.06.012–016 confirmed active in BC-INDEX v5.52 (PO finding commit b8cf19e1 2026-05-29). All 5 anchor BCs are status: active + lifecycle_status: active (auto-promoted at S-CONFIG merge develop@3e822522 2026-05-26 per POL-14). S-7.01 gate cleared: behavioral_contracts non-empty, all BC IDs match BC-\d+\.\d{2}\.\d{3} pattern, all BCs active, bidirectional AC↔BC traces verified (all ACs cite specific BC clauses). Body Version v0.1→v1.2, Status draft→ready. BC status frontmatter comment updated to reflect confirmed-active status. |
