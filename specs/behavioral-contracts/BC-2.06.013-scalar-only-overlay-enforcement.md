---
document_type: behavioral-contract
level: L3
bc_id: "BC-2.06.013"
version: "1.0"
status: draft
lifecycle_status: draft
producer: product-owner
timestamp: 2026-05-23T00:00:00Z
phase: 1a
origin: greenfield
subsystem: "SS-06"
capability: "CAP-009"
introduced: "2026-05-23"
modified: "2026-05-23"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
anchored_stories: [S-CONFIG-MULTI-TENANT-OVERRIDE-001]
verifying_vps: []
crates: [prism-spec-engine, prism-bin]
inputs:
  - ".factory/specs/architecture/decisions/ADR-029-multi-tenant-sensor-endpoint-overrides.md"
  - ".factory/research/multi-tenant-sensor-endpoint-overrides-2026-05-23.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: ""
traces_to:
  - "CAP-009"
  - "ADR-029"
extracted_from: null
---

# BC-2.06.013: Scalar-Only Overlay Enforcement — Boot-Time Rejection of Schema Fields in Overlay Files

## Description

During boot step 4 (`step4_load_sensor_specs`), the overlay validator inspects every
`customers/<org_slug>/<sensor_id>.sensor.toml` file and hard-rejects any file that contains
schema-structural keys. The enumerated list of allowed overlay fields is: `extends`,
`instance_id`, `base_url`, `timeout_secs`, `[rate_limit_hints]` (and its scalar sub-fields:
`requests_per_second`, `burst_size`). Any other key — including `[[tables]]`, any `[columns]`
block, `auth_type`, `version`, or any unrecognized field — triggers a structured boot hard
error (exit code 2). This enforcement converts the TOML array-replace footgun (where a
`[[tables]]` block in an overlay would silently replace the entire TYPE schema) into an
unmistakable fail-fast boot error. Validation runs BEFORE the merge step — a file that
fails validation is never merged.

## Preconditions

- Boot step 4 has been invoked and the `customers/` directory walk is in progress.
- Each overlay file has been found on disk and read into memory as raw TOML.
- TYPE specs are loaded (precondition for step 4; TYPE spec knowledge is not required for
  structural rejection — rejection happens on TOML key inspection before cross-referencing
  TYPE spec content).

## Postconditions

### Success path (valid overlay file)

- The overlay file contains ONLY fields from the allowed set: `extends`, `instance_id`,
  `base_url`, `timeout_secs`, `[rate_limit_hints]` (and its scalar sub-fields).
- The file is accepted for further validation (proceeds to BC-2.06.012 merge logic and
  BC-2.06.015 org-slug cross-validation).
- No error is emitted.

### Failure path — `[[tables]]` present in overlay

- The overlay file contains one or more `[[tables]]` array-of-tables blocks.
- `E-SPEC-021` is emitted with: file path, org_slug, sensor_id, and message
  `"Per-org overlay '{file}' for instance '{instance_id}' contains [[tables]] blocks; schema overrides are forbidden in overlay files (ADR-029). Remove [[tables]] and declare schema in the TYPE spec only."`.
- Boot exits with code 2 (`BootError::ConfigInvalid`).
- No query can be served before this error is resolved.

### Failure path — unrecognized field in overlay

- The overlay file contains a field not in the allowed set (e.g., `auth_type`, `name`,
  or a custom field).
- `E-SPEC-023` is emitted with: file path, org_slug, sensor_id, unrecognized field name,
  and message `"Per-org overlay '{file}' contains unrecognized field '{field}'; allowed fields are: extends, instance_id, base_url, timeout_secs, rate_limit_hints"`.
- Boot exits with code 2.

### Failure path — `instance_id` convention mismatch

- The overlay's `instance_id` value does not match the pattern `{sensor_id}@{org_slug}`
  where `sensor_id` is derived from the filename stem and `org_slug` is derived from the
  parent directory name.
- `E-SPEC-020` is emitted with: file path, declared `instance_id`, expected value
  `"{sensor_id}@{org_slug}"`.
- Boot exits with code 2.

## Invariants

- INV-SCALAR-001: The allowed scalar field list is closed and enumerated. No field can be
  added to the allowed set without a corresponding BC amendment and ADR-029 revision.
- INV-SCALAR-002: Rejection happens before merge. A file that fails structural validation
  is never partially merged onto the TYPE spec.
- INV-SCALAR-003: A single invalid overlay file fails the entire boot. There is no partial
  boot where some overlays succeed and a structurally invalid overlay is skipped.
- INV-SCALAR-004: The `[[tables]]` footgun produces exit code 2, not a warning. Operators
  cannot accidentally bypass schema protection via the `[[tables]]`-in-overlay path.

## Allowed vs Forbidden Overlay Fields

| Field | Allowed | Notes |
|-------|---------|-------|
| `extends` | Yes | Required; names the TYPE spec |
| `instance_id` | Yes | Required; must match `{sensor_id}@{org_slug}` |
| `base_url` | Yes | Scalar URL override |
| `timeout_secs` | Yes | Scalar per-org request timeout |
| `[rate_limit_hints]` | Yes (table) | Rate limit override sub-table |
| `rate_limit_hints.requests_per_second` | Yes | Scalar within rate_limit_hints |
| `rate_limit_hints.burst_size` | Yes | Scalar within rate_limit_hints |
| `[[tables]]` | **Forbidden** | E-SPEC-021; schema lives at TYPE level |
| `auth_type` | **Forbidden** | E-SPEC-023; auth mechanism is immutable per org |
| `version` | **Forbidden** | E-SPEC-023; version is TYPE-level |
| `sensor_id` | **Forbidden** | E-SPEC-023; identity comes from filename + `extends` |
| `name` | **Forbidden** | E-SPEC-023; display name is TYPE-level |
| Any other key | **Forbidden** | E-SPEC-023 |

## Error Conditions

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-SPEC-021` | Overlay contains `[[tables]]` block | Boot exits 2; error identifies file, instance, forbidden key |
| `E-SPEC-023` | Overlay contains unrecognized scalar field | Boot exits 2; error identifies file, instance, unrecognized field name |
| `E-SPEC-020` | `instance_id` does not match `{sensor_id}@{org_slug}` | Boot exits 2; error shows declared vs expected instance_id |
| `E-SPEC-001` | TOML parse error in overlay file | Boot exits 2; TOML error with line number |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-013-001 | Overlay contains `[[tables]]` AND an unknown field | Both `E-SPEC-021` and `E-SPEC-023` are collected; both are reported in the boot error; boot exits 2 |
| EC-013-002 | Operator accidentally copies a TYPE spec as an overlay | All TYPE-level keys (`[[tables]]`, `auth_type`, `name`) trigger `E-SPEC-021` / `E-SPEC-023`; operator receives actionable guidance to remove schema keys |
| EC-013-003 | Overlay with only `extends` and `instance_id` (no scalar overrides) | Valid minimal overlay; accepted and merged (no-op merge — all fields from TYPE spec) |
| EC-013-004 | `extends` key absent from overlay | `E-SPEC-001` (parse/validation) — `extends` is a required field; overlay is rejected |
| EC-013-005 | `instance_id` key absent from overlay | Same as EC-013-004; `instance_id` is required |
| EC-013-006 | `rate_limit_hints` present as a table but contains an invalid sub-field | `E-SPEC-023` for the unrecognized sub-field within `rate_limit_hints` |

## Canonical Test Vectors

| Scenario | Overlay Content | Expected Result |
|----------|-----------------|-----------------|
| Happy path — scalars only | `extends="armis"`, `instance_id="armis@acme"`, `base_url="https://armis.acme.io"` | Accepted; no error |
| `[[tables]]` present | Full sensor TYPE spec pasted as overlay | `E-SPEC-021`; boot fails exit 2 |
| Unknown field | `extends`, `instance_id`, `base_url`, `secret_key="x"` | `E-SPEC-023` for `secret_key`; boot fails exit 2 |
| `auth_type` present | `extends`, `instance_id`, `auth_type="bearer_static"` | `E-SPEC-023` for `auth_type`; boot fails exit 2 |
| Wrong `instance_id` | `instance_id="armis@wrongorg"` in `customers/acme/armis.sensor.toml` | `E-SPEC-020`; boot fails exit 2 |
| Minimal valid overlay | `extends` + `instance_id` only | Accepted; merged spec has all fields from TYPE |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (none yet) | Red Gate tests: `test_BC_2_06_013_tables_in_overlay_fails_boot` and `test_BC_2_06_013_unrecognized_field_fails_boot` — both must emit the correct E-SPEC code and exit 2 |

## Related BCs

- BC-2.06.012 — Per-Tenant Overlay Loading and Merge Semantics (this BC gates entry to the merge path)
- BC-2.06.015 — OrgRegistry Cross-Validation at Boot (parallel boot-time validation)
- BC-2.06.016 — Error Taxonomy for Override Violations (defines the E-SPEC-NNN codes emitted here)
- BC-2.16.001 — Sensor Spec File Loading (TYPE spec loading; `[[tables]]` lives here only)

## Architecture Anchors

- ADR-029 §Instance Overlay Schema: "FORBIDDEN in overlay files (boot-time hard error E-SPEC-020)"
- ADR-029 §Boot-Time Validation: step 3d "Reject any overlay containing `[[tables]]` blocks (E-SPEC-020)"
- ADR-029 §Decision Driver: "TOML array REPLACE semantics — `[[tables]]` arrays are replaced on merge, never merged element-wise"
- Research artifact §2.10: "TOML array-replace footgun — the central risk that drove Helm/Kustomize complexity"
- `prism-spec-engine/src/spec_parser.rs` — overlay structural validator

## Story Anchor

S-CONFIG-MULTI-TENANT-OVERRIDE-001 (to-be-created)

## VP Anchors

(None yet — VP to be authored by test-writer alongside S-CONFIG-MULTI-TENANT-OVERRIDE-001)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 |
| Capability Anchor Justification | CAP-009 ("Client Configuration") per capabilities.md §CAP-009 — this BC specifies boot-time configuration validation: "Configuration Validation Reports All Errors in One Pass" (BC-2.06.005 pattern); the overlay schema-field rejection is a structural config validation that fires during `step4_load_sensor_specs` as part of the client configuration loading lifecycle. |
| L2 Invariants | DI-008 (client data separation — schema stability across orgs prevents one org's overlay from corrupting schema for another org's queries) |
| L2 Entities | SensorInstanceOverlay, BootError |
| Priority | P0 |
| ADR | ADR-029 (Multi-Tenant Sensor Endpoint Overrides) |
| Source-of-Truth Precedence | ADR-029 §Instance Overlay Schema (forbidden field list) is authoritative. This BC supersedes any earlier ambiguity. |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | D-803 burst-3 | 2026-05-23 | product-owner | Initial draft per ADR-029 Burst 3 handoff |
