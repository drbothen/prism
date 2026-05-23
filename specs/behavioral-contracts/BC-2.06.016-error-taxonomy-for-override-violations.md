---
document_type: behavioral-contract
level: L3
bc_id: "BC-2.06.016"
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
crates: [prism-spec-engine, prism-bin, prism-core]
inputs:
  - ".factory/specs/architecture/decisions/ADR-029-multi-tenant-sensor-endpoint-overrides.md"
  - ".factory/research/multi-tenant-sensor-endpoint-overrides-2026-05-23.md"
  - ".factory/specs/prd-supplements/error-taxonomy.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: ""
traces_to:
  - "CAP-009"
  - "ADR-029"
extracted_from: null
---

# BC-2.06.016: Error Taxonomy for Per-Org Overlay Override Violations (E-SPEC-019 through E-SPEC-023)

## Description

This BC defines the operator-facing error contract for the five classes of per-org overlay
violations introduced by ADR-029. Each violation is classified with a unique error code, a
canonical message template, a severity level, and an actionable operator suggestion. All five
errors fire during boot step 4 (`step4_load_sensor_specs`) and cause boot to fail with exit
code 2 (`BootError::ConfigInvalid`). They are classified as `broken/validation` — the prism
process cannot serve any queries until the configuration is corrected.

**Error code allocation note:** ADR-029 originally proposed codes E-SPEC-018 through E-SPEC-022.
However, E-SPEC-018 is already allocated in error-taxonomy.md v1.43 to the timestamp-parse
failure from ADR-028/BC-2.16.013 (`TimestampParseFailure`). To preserve append-only ID
integrity (DF-030 / POL-1), ADR-029 override violations are allocated E-SPEC-019 through
E-SPEC-023, shifting each code up by one from the ADR's draft numbering.

### ADR-029 Draft → Final Code Mapping

| ADR-029 Draft Code | Violation | Final Code |
|--------------------|-----------|------------|
| E-SPEC-018 | `extends` references unknown sensor TYPE | **E-SPEC-019** |
| E-SPEC-019 | `instance_id` does not match `{sensor}@{org}` | **E-SPEC-020** |
| E-SPEC-020 | Overlay contains `[[tables]]` blocks | **E-SPEC-021** |
| E-SPEC-021 | Overlay directory references unknown org slug | **E-SPEC-022** |
| E-SPEC-022 | Overlay contains unrecognized scalar field | **E-SPEC-023** |

## Preconditions

- Error taxonomy rows for E-SPEC-019 through E-SPEC-023 have been added to
  `.factory/specs/prd-supplements/error-taxonomy.md` (implementation story responsibility).
- `prism-core` exports `SpecErrorCode::ESpec019` through `SpecErrorCode::ESpec023` variants
  (parallel to the existing `SpecErrorCode::ESpec017` variant introduced in S-PLUGIN-PREREQ-D;
  implementer responsibility in S-CONFIG-MULTI-TENANT-OVERRIDE-001).
- Boot step 4 validation is in progress.

## Postconditions

When an override violation occurs, the operator observes:

1. A structured error written to stderr (JSON-formatted `BootError::ConfigInvalid` block)
   containing: `code`, `category`, `severity`, `file_path` (the overlay file or directory),
   `org_slug`, `sensor_id` (where applicable), `instance_id` (where applicable), `message`
   (canonical template rendered with context), and `suggestion` (actionable corrective guidance).
2. Process exits with code 2.
3. No MCP connection is established; no sensor queries are dispatched.

## Error Catalog (Canonical Definitions)

### E-SPEC-019: Unknown Sensor TYPE in Overlay `extends` Field

| Field | Value |
|-------|-------|
| Code | `E-SPEC-019` |
| Severity | broken |
| Category | validation |
| Exit code | 2 |
| Retryable | No |
| Message template | `"Per-org overlay '{file}' declares extends='{extends_value}' but no sensor TYPE named '{extends_value}' is loaded. Check spelling or add a TYPE spec file named '{extends_value}.sensor.toml'."` |
| Suggestion | `"Ensure a TYPE spec file named '{extends_value}.sensor.toml' exists in the root sensor_specs_dir and is valid before adding an overlay that extends it."` |
| BC enforcement | BC-2.06.012 §Error Conditions; BC-2.06.013 §Error Conditions |

### E-SPEC-020: Malformed `instance_id` Convention

| Field | Value |
|-------|-------|
| Code | `E-SPEC-020` |
| Severity | broken |
| Category | validation |
| Exit code | 2 |
| Retryable | No |
| Message template | `"Per-org overlay '{file}' declares instance_id='{actual}' but expected '{sensor_id}@{org_slug}' (derived from filename and parent directory). Rename or correct the instance_id field."` |
| Suggestion | `"Set instance_id to '{sensor_id}@{org_slug}' where '{sensor_id}' matches the filename stem and '{org_slug}' matches the parent directory name."` |
| BC enforcement | BC-2.06.012 §Error Conditions; BC-2.06.013 §Error Conditions |

### E-SPEC-021: Schema Override Forbidden — `[[tables]]` in Overlay

| Field | Value |
|-------|-------|
| Code | `E-SPEC-021` |
| Severity | broken |
| Category | validation |
| Exit code | 2 |
| Retryable | No |
| Message template | `"Per-org overlay '{file}' for instance '{instance_id}' contains [[tables]] blocks. Schema overrides are forbidden in overlay files (ADR-029). Table schema must be declared in the TYPE spec '{extends_value}.sensor.toml' only."` |
| Suggestion | `"Remove [[tables]] from the overlay file. If you need to add or modify a sensor table's schema, edit the TYPE spec '{extends_value}.sensor.toml' directly. Per-org overlays may only set: extends, instance_id, base_url, timeout_secs, rate_limit_hints."` |
| BC enforcement | BC-2.06.013 §Error Conditions (primary); BC-2.06.012 §Error Conditions |
| Risk | This is the TOML array-replace footgun from ADR-029 §Decision Drivers. The error message must be maximally actionable because operators unfamiliar with TOML array semantics will naturally try to add per-org tables. |

### E-SPEC-022: Unknown Org Slug in Overlay Directory

| Field | Value |
|-------|-------|
| Code | `E-SPEC-022` |
| Severity | broken |
| Category | validation |
| Exit code | 2 |
| Retryable | No |
| Message template | `"Per-org overlay directory 'customers/{slug}/' references org slug '{slug}' which is not registered in OrgRegistry. Check for typos or register the org in prism.toml [[orgs]]."` |
| Suggestion | `"Either: (a) add an [[orgs]] entry for '{slug}' in prism.toml, or (b) remove or rename the stale directory 'customers/{slug}/'."` |
| BC enforcement | BC-2.06.015 §Error Conditions (primary) |

### E-SPEC-023: Unrecognized Field in Overlay File

| Field | Value |
|-------|-------|
| Code | `E-SPEC-023` |
| Severity | broken |
| Category | validation |
| Exit code | 2 |
| Retryable | No |
| Message template | `"Per-org overlay '{file}' contains unrecognized field '{field_name}'. Allowed overlay fields are: extends, instance_id, base_url, timeout_secs, rate_limit_hints (with sub-fields: requests_per_second, burst_size)."` |
| Suggestion | `"Remove '{field_name}' from the overlay file. If this field controls sensor behavior, it belongs in the TYPE spec '{extends_value}.sensor.toml', not in a per-org overlay."` |
| BC enforcement | BC-2.06.013 §Allowed vs Forbidden Overlay Fields (primary) |

## Invariants

- INV-ERR-001: All five codes are FATAL/broken/validation. No downgrade to `degraded` or
  `cosmetic` is permitted. The process must not serve queries with any unresolved override
  violation.
- INV-ERR-002: Error messages MUST NOT include credential values. `base_url` values from
  the overlay may appear in error messages (they are endpoint URLs, not credentials per
  AD-017). `extends` values, `instance_id` values, and field names may appear.
- INV-ERR-003: Multiple violations in the same boot are reported together (multi-error
  aggregation pattern per BC-2.06.005). Boot does not stop at the first error — all
  overlay files are scanned and all errors collected before emitting the final error report.
- INV-ERR-004: Error code IDs are append-only (DF-030). E-SPEC-019 through E-SPEC-023
  are permanently allocated to these violation classes and are never reused for other purposes.
- INV-ERR-005: The error code shift (ADR-029 draft → final) is recorded in the Changelog
  of this BC and in the error-taxonomy.md row descriptions. The ADR-029 text citing
  E-SPEC-018 through E-SPEC-022 is superseded by this BC's E-SPEC-019 through E-SPEC-023
  allocation per source-of-truth precedence (this BC supersedes the ADR draft).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-016-001 | E-SPEC-021 and E-SPEC-023 both fire on same overlay | Both errors collected; multi-error report shows both codes; boot fails |
| EC-016-002 | E-SPEC-022 fires for dir AND E-SPEC-021 fires for a file within it | E-SPEC-022 is a directory-level check; E-SPEC-021 is a file-level check; both collected |
| EC-016-003 | All five error codes fire in the same boot | Multi-error boot failure report lists all five; each with file path and suggestion |
| EC-016-004 | E-SPEC-019: `extends` value is a valid TYPE but with wrong case | `E-SPEC-019` fires (sensor_id matching is case-sensitive per BC-2.16.001 §Error Conditions E-SPEC-017 precedent) |

## Canonical Test Vectors

| Scenario | Error Expected | Message Includes |
|----------|---------------|------------------|
| Overlay `extends="armis"` but no `armis.sensor.toml` TYPE spec exists | `E-SPEC-019` | `"extends='armis'"`, suggestion to add TYPE spec |
| `instance_id="armis@wrongorg"` in `customers/acme/armis.sensor.toml` | `E-SPEC-020` | `"expected 'armis@acme'"`, `"actual 'armis@wrongorg'"` |
| Overlay file contains `[[tables]]` | `E-SPEC-021` | overlay file path, `instance_id`, allowed fields list |
| `customers/stale-org/` dir not in OrgRegistry | `E-SPEC-022` | `"slug 'stale-org'"`, suggestion to register or remove |
| Overlay contains `auth_type = "bearer"` | `E-SPEC-023` | `"field 'auth_type'"`, allowed fields list |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (none yet) | Five Red Gate tests: one per E-SPEC code. Each test drives the exact violation scenario and asserts on the error code emitted, the presence of key message fields, and exit code 2. |

## Related BCs

- BC-2.06.012 — Per-Tenant Overlay Loading and Merge Semantics (emits E-SPEC-019, E-SPEC-020)
- BC-2.06.013 — Scalar-Only Overlay Enforcement (emits E-SPEC-021, E-SPEC-023)
- BC-2.06.015 — OrgRegistry Cross-Validation at Boot (emits E-SPEC-022)
- BC-2.16.001 — Sensor Spec File Loading (E-SPEC-017 precedent for case-sensitive sensor_id matching)

## Architecture Anchors

- ADR-029 §New Error Codes: original E-SPEC-018–022 table (superseded by this BC's E-SPEC-019–023 allocation)
- error-taxonomy.md v1.43: E-SPEC-018 already allocated to `TimestampParseFailure` (ADR-028/BC-2.16.013)
- error-taxonomy.md §SPEC: append-only POL-1 compliance — E-SPEC-019 through E-SPEC-023 are new rows
- `prism-core/src/error.rs` — `SpecErrorCode` enum (implementer adds variants ESpec019–ESpec023)
- `prism-spec-engine/src/spec_parser.rs` — overlay validator emits these codes

## Story Anchor

S-CONFIG-MULTI-TENANT-OVERRIDE-001 (to-be-created)

## VP Anchors

(None yet — VP to be authored by test-writer alongside S-CONFIG-MULTI-TENANT-OVERRIDE-001)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 |
| Capability Anchor Justification | CAP-009 ("Client Configuration") per capabilities.md §CAP-009 — this BC defines the error taxonomy for config validation failures in the per-org overlay loading lifecycle: "Missing Required Fields Produce Actionable Error Messages" (BC-2.06.007 pattern) applied to overlay violation detection. |
| L2 Invariants | DI-008 (client data separation — error messages are org-scoped; no cross-org data in error output) |
| L2 Entities | BootError, SpecErrorCode |
| Priority | P0 |
| ADR | ADR-029 (Multi-Tenant Sensor Endpoint Overrides) |
| Source-of-Truth Precedence | This BC supersedes ADR-029's draft E-SPEC-018–022 table for the final error code allocation. The implementer must use E-SPEC-019–023 (not E-SPEC-018–022) as the canonical codes. |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | D-803 burst-3 | 2026-05-23 | product-owner | Initial draft per ADR-029 Burst 3 handoff. Resolved E-SPEC-018 collision with ADR-028 TimestampParseFailure (error-taxonomy.md v1.43); allocated E-SPEC-019–023 instead of ADR-029 draft E-SPEC-018–022. All sibling BCs (BC-2.06.012, BC-2.06.013, BC-2.06.015) updated to cite the final codes. |
