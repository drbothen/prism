---
document_type: behavioral-contract
level: L3
version: "0.7"
status: PROPOSED
producer: product-owner
timestamp: 2026-04-27T00:00:00
phase: 3.A
inputs:
  - ".factory/specs/architecture/decisions/ADR-007-configurable-dtu-mode.md"
  - ".factory/specs/architecture/decisions/ADR-010-customer-config-schema.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "21f7e5a"
traces_to: ".factory/specs/architecture/decisions/ADR-010-customer-config-schema.md"
origin: greenfield
extracted_from: null
subsystem: "SS-06"
capability: "CAP-009"
lifecycle_status: active
introduced: wave-3
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
bc_id: BC-3.3.004
title: Customer Config Validation Rejects Invalid Schema at Startup
wave: 3
phase: 3.A
date: 2026-04-27
authors: [product-owner]
related_decisions: [D-041, D-042, D-046]
related_adrs: [ADR-010]
inherits_from: null
superseded_by: null
---

# BC-3.3.004: Customer Config Validation Rejects Invalid Schema at Startup

## Description

At Prism startup, every `customers/*.toml` file is parsed and structurally validated before the `OrgRegistry` is populated. Any file that fails field-type checks, violates constraint rules, or contains unknown fields causes the process to refuse to start and emit an error message identifying the offending file and the specific rule violation. Validation is multi-error: all problems across all files are collected and reported in a single startup pass rather than stopping at the first failure.

## Preconditions

1. The Prism process is starting (pre-request-acceptance phase).
2. The `customers/` directory exists at the workspace root.
3. The archetype catalog is compiled into the binary (embedded in `prism-dtu-common`).
4. The `DTU_DEFAULT_MODE` registry (ADR-007 §2.3) is available. It DOES include `demo-server` with a `test_only = true` annotation (per ADR-007 D-051). The production config validator absence-checks `[[dtu]].type` against the production-allowed set — registry entries with `test_only = true` are excluded from that set. `demo-server` is therefore rejected in production config not because it is absent from the registry, but because it is marked test-only.
5. Zero or more `*.toml` files may be present in `customers/`; zero files is valid.

## Postconditions

**On any validation failure (process MUST NOT start):**

1. The process exits with code `1` before accepting any MCP connections.
2. All collected validation errors across all files are written to stderr; stdout is empty.
3. Each error line includes: filename, error code, and a human-readable description naming the offending value.
4. `OrgRegistry` contains zero entries (no partial registration occurs).
5. No DTU instances are constructed.

**On successful validation:**

1. `OrgRegistry` contains exactly one entry per `customers/*.toml` file, registered in lexicographic filename order.
2. DTU instance maps are constructed from all `[[dtu]]` blocks.
3. The process proceeds to accept MCP connections.

**Specific rejection rules:**

| Rule | Condition | Error Code | Example Message |
|------|-----------|------------|-----------------|
| R-CUST-001 | Missing required top-level field (`schema_version`, `org_id`, `org_slug`, `display_name`) | `E-CFG-001` | `customers/acme.toml: E-CFG-001: missing required field 'org_id'` |
| R-CUST-002 | `org_slug` does not match filename stem (case-sensitive) | `E-CFG-002` | `customers/acme-corp.toml: E-CFG-002: org_slug 'acme-new' does not match filename stem 'acme-corp'` |
| R-CUST-003 | `org_id` is not a valid UUID v7 (version nibble != 7) | `E-CFG-003` | `customers/acme.toml: E-CFG-003: org_id '550e8400-...' is UUID v4; must be UUID v7` |
| R-CUST-004 | `[[dtu]] type` not in `DTU_DEFAULT_MODE` registry at all (truly unknown type string) | `E-CFG-004` | `customers/acme.toml: E-CFG-004: unknown DTU type 'fake-sensor'` |
| R-CUST-005 | `credential_ref` not matching an allowed opaque scheme | `E-CFG-005` | `customers/acme.toml: E-CFG-005: credential_ref 'bearer-abc' does not match allowed schemes (vault://, env://, file://, keyring://)` |
| R-CUST-006 | `data.archetype` not in archetype catalog | `E-CFG-006` | `customers/acme.toml: E-CFG-006: unknown archetype 'enterprise-ot'; valid: HealthyOtEnvironment, CompromisedEndpoint, AuthOutage, LargeScale, PaginationEdgeCases, SchemaDrift, HighChurn, DormantTenant` |
| R-CUST-007 | `data.seed` is negative or overflows `u64` | `E-CFG-007` | `customers/acme.toml: E-CFG-007: data.seed -1 is not a non-negative integer (u64 range required)` |
| R-CUST-008 | `data.scale` is `<= 0.0`, `NaN`, or infinite | `E-CFG-008` | `customers/acme.toml: E-CFG-008: data.scale 0.0 must be a positive finite float` |
| R-CUST-009 | `mode` not in `{"shared","client"}` | `E-CFG-009` | `customers/acme.toml: E-CFG-009: dtu[0].mode 'dedicated' is invalid; must be 'shared' or 'client'` |
| R-CUST-010 | Unknown field at any level (`deny_unknown_fields`) | `E-CFG-010` | `customers/acme.toml: E-CFG-010: unknown field 'api_url' in [[dtu]] block` |
| R-CUST-011 | Duplicate `org_id` across two files | `E-CFG-011` | `E-CFG-011: org_id '01975e4e-...' declared in both 'customers/acme-corp.toml' and 'customers/acme.toml'` |
| R-CUST-012 | Duplicate `org_slug` across two files | `E-CFG-012` | `E-CFG-012: org_slug 'acme-corp' declared in both 'customers/acme-corp.toml' and 'customers/acme2.toml'` |
| R-CUST-013 | `[[dtu]] type` is in `DTU_DEFAULT_MODE` but has `test_only = true` annotation — type is registry-known but not permitted in production customer config | `E-CFG-013` | `customers/acme.toml: E-CFG-013: DTU type 'demo-server' is test-only and cannot be used in production customer config` |
| R-CUST-014 | `[[dtu]]` block has `mode = "client"` but the `spec` field is absent (client-mode DTU requires a sensor spec path) | `E-CFG-014` | `customers/acme.toml: E-CFG-014: [[dtu]] type 'claroty' has mode='client' but 'spec' field is missing; provide a path to the sensor spec TOML` |
| R-CUST-015 | `[[dtu]]` block has `mode = "client"` and a `spec` field is present, but the referenced TOML file does not exist on disk | `E-CFG-015` | `customers/acme.toml: E-CFG-015: spec path 'sensors/claroty.toml' does not exist on disk` |
| R-CUST-016 | `[[dtu]]` block has `mode = "shared"` and the `spec` field is present — `spec` is only valid for `mode = "client"` | `E-CFG-016` | `customers/acme.toml: E-CFG-016: [[dtu]] type 'slack' has mode='shared' but 'spec' field is present; 'spec' is only valid for mode='client'` |

## Invariants

1. Config validation MUST complete for all files before `OrgRegistry::register` is called for any file (ADR-010 §2.5 step order).
2. A failed validation leaves zero `OrgRegistry` entries — no partial registration.
3. `demo-server` is never a valid `[[dtu]] type` value in production customer config files. It is present in `DTU_DEFAULT_MODE` with `test_only = true` (ADR-007 D-051); the production validator rejects it via absence-check against the production-allowed set. The error code for a test-only type used in production config is `E-CFG-013` (distinct from `E-CFG-004` "unknown type"), because the type IS in the registry but is test-only.
4. Files are processed in lexicographic filename order for deterministic error reporting across runs.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-3.3.004-01 | `customers/` directory contains zero `*.toml` files | Process starts successfully with empty `OrgRegistry`; no error |
| EC-3.3.004-02 | A single file has three distinct validation violations | All three error codes emitted before exit; exit code 1 |
| EC-3.3.004-03 | Two files each have different violations | Errors from both files emitted in lexicographic file order; exit code 1 |
| EC-3.3.004-04 | `data.scale = NaN` (IEEE 754 not-a-number encoded in TOML) | `E-CFG-008` rejection; message states "NaN is not a positive finite float" |
| EC-3.3.004-05 | `data.scale = inf` | `E-CFG-008` rejection; message states "infinite value is not a positive finite float" |
| EC-3.3.004-06 | `org_id` is a valid UUID but version nibble is 4 (UUID v4) | `E-CFG-003`; message states "UUID v4; must be UUID v7" |
| EC-3.3.004-07 | `customers/` contains a non-`.toml` file (e.g., `README.md`) | File is skipped silently; no error |
| EC-3.3.004-08 | A `[[dtu]]` block has `mode = "client"` but `spec` field is absent | `E-CFG-014` — "mode='client' requires 'spec' field; no spec path provided" |
| EC-3.3.004-09 | A `[[dtu]]` block has `mode = "client"` and `spec = "sensors/claroty.toml"` but that file does not exist on disk | `E-CFG-015` — "spec path 'sensors/claroty.toml' does not exist on disk"; multi-error collection continues for remaining files |
| EC-3.3.004-10 | A `[[dtu]]` block has `mode = "shared"` and a `spec` field is present (e.g., `spec = "sensors/slack.toml"`) | `E-CFG-016` — "mode='shared' but 'spec' field is present; 'spec' is only valid for mode='client'"; `deny_unknown_fields` does NOT catch this because `spec` is a known field in the schema — this requires an explicit semantic validation rule (ADR-010 §2.3 rule 5) |

## Canonical Test Vectors

| TV-ID | Input | Expected Output | Category |
|-------|-------|-----------------|----------|
| TV-3.3.004-01 | `customers/acme.toml` with `org_id` field absent | Exit 1; stderr contains `E-CFG-001` naming `'org_id'` | error |
| TV-3.3.004-02 | `customers/acme-corp.toml` with `org_slug = "acme-new"` | Exit 1; stderr contains `E-CFG-002`; names both `'acme-new'` and `'acme-corp'` | error |
| TV-3.3.004-03 | `customers/acme.toml` with `org_id = "550e8400-e29b-41d4-a716-446655440000"` (UUID v4) | Exit 1; stderr contains `E-CFG-003`; states "UUID v4" | error |
| TV-3.3.004-04 | `customers/acme.toml` with `[[dtu]] type = "demo-server"` | Exit 1; stderr contains `E-CFG-013`; message states "test-only" and names `'demo-server'`; `E-CFG-004` is NOT used (demo-server is in the registry with `test_only=true`, not an unknown type) | error |
| TV-3.3.004-05 | `customers/acme.toml` with `credential_ref = "bearer-token-abc123"` | Exit 1; stderr contains `E-CFG-005`; lists allowed schemes | error |
| TV-3.3.004-06 | `customers/acme.toml` with `data.archetype = "enterprise-ot"` | Exit 1; stderr contains `E-CFG-006`; lists valid archetypes | error |
| TV-3.3.004-07 | `customers/acme.toml` with `data.seed = -1` | Exit 1; stderr contains `E-CFG-007` | error |
| TV-3.3.004-08 | `customers/acme.toml` with `data.scale = 0.0` | Exit 1; stderr contains `E-CFG-008`; names `0.0` | error |
| TV-3.3.004-09 | `customers/acme.toml` with `dtu[0].mode = "dedicated"` | Exit 1; stderr contains `E-CFG-009`; names `'dedicated'` | error |
| TV-3.3.004-10 | `customers/acme.toml` with `api_url = "https://example.com"` in `[[dtu]]` | Exit 1; stderr contains `E-CFG-010`; names `'api_url'` | error |
| TV-3.3.004-11 | Valid `customers/acme-corp.toml` (all fields correct, UUID v7, slug matches filename) | Exit 0; `OrgRegistry` contains entry for `org_slug = "acme-corp"` | happy-path |
| TV-3.3.004-12 | Two files both declaring `org_id = "01975e4e-9f00-7abc-8def-000000000001"` | Exit 1; stderr contains `E-CFG-011`; both filenames named | error |
| TV-3.3.004-13 | Single file with violations: missing `org_id`, unknown field, bad seed | Exit 1; stderr contains `E-CFG-001`, `E-CFG-010`, `E-CFG-007` — all three | edge-case |
| TV-3.3.004-14 | `customers/acme.toml` with `[[dtu]] mode = "client"` and `spec = "sensors/nonexistent.toml"` (file absent on disk) | Exit 1; stderr contains `E-CFG-015`; names `'sensors/nonexistent.toml'` | error |
| TV-3.3.004-15 | `customers/acme.toml` with `[[dtu]] type = "slack" mode = "shared" spec = "sensors/slack.toml"` | Exit 1; stderr contains `E-CFG-016`; names `'slack'` and states "mode='shared' but 'spec' field is present" | error |

## Verification Properties

| VP | Property | Proof Method |
|----|----------|--------------|
| VP-105 / VP-3.3.004-A | For all inputs, if exit code is 0 then `OrgRegistry` entry count equals file count | proptest |
| VP-106 / VP-3.3.004-B | For all inputs with any validation error, exit code is always 1 and `OrgRegistry` is empty | proptest |
| VP-107 / VP-3.3.004-C | Validation error output always includes the offending filename | integration test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 ("Client Configuration") per capabilities.md §CAP-009 |
| Capability Anchor Justification | CAP-009 ("Client Configuration") per capabilities.md §CAP-009 — this BC specifies startup validation of per-customer TOML config files, which is exactly what CAP-009 defines: "Load and validate per-client sensor mappings, credential references, and capability overrides from TOML configuration." |
| L2 Domain Invariants | N/A (Wave 3 new capability; DI-NNN assignment pending domain-spec Wave 3 extension) |
| Architecture Module | SS-06 (Client Configuration) per ARCH-INDEX.md |
| Stories | S-3.3.01, S-3.3.02 |

## Related BCs

- BC-3.3.002 — depends on (credential ref opaqueness, evaluated in same validation pass)
- BC-3.3.003 — depends on (schema_version enforcement, evaluated in same validation pass)

## Architecture Anchors

- `crates/prism-core/src/ids.rs:10-42` — UUID v7 constraint; `OrgId` implemented via `uuid_v7_newtype!` macro; `OrgId::try_from` validates version nibble (planned implementation site)
- ADR-010 §2.2 — required top-level fields and their type constraints
- ADR-010 §2.3 — `[[dtu]]` block validation rules 1–10
- ADR-010 §2.5 — loading lifecycle: validation-before-registration ordering invariant

## Story Anchor

S-3.3.01, S-3.3.02

## VP Anchors

- VP-105 (VP-3.3.004-A) — proptest: exit-0 implies OrgRegistry count == file count
- VP-106 (VP-3.3.004-B) — proptest: any validation error implies exit-1 and empty OrgRegistry
- VP-107 (VP-3.3.004-C) — integration_test: error output names offending file

## BC Changelog

| Version | Change |
|---------|--------|
| v0.7 | M-002 (Pass 6): R-CUST-016 / E-CFG-016 added: `[[dtu]] mode='shared'` with `spec` field present → E-CFG-016. `spec` is a known schema field so `deny_unknown_fields` does not catch this — requires explicit semantic rule per ADR-010 §2.3 rule 5. EC-3.3.004-10 and TV-3.3.004-15 added. E-CFG-016 added to error-taxonomy.md v1.10. |
| v0.6 | m-002 (Pass 5): R-CUST-015 row added to rejection rules table: `[[dtu]] mode='client'` with `spec` field present but file absent on disk → `E-CFG-015`. EC-3.3.004-09 and TV-3.3.004-14 added for the file-existence check. Frontmatter `title:` corrected to title-case to match H1. |
| v0.5 | m-002 (Pass 4): Verification Properties table and VP Anchors updated to include flat VP-NNN IDs alongside dotted forms (VP-105/VP-3.3.004-A through VP-107/VP-3.3.004-C). VP-107 proof method corrected manual/integration test → integration test (consistent with VP-INDEX). |
| v0.4 | M-002 fix (2026-04-27): EC-3.3.004-08 error code corrected E-CFG-013 → E-CFG-014 (mode='client' missing spec field). E-CFG-013 remains bound exclusively to R-CUST-013 (test-only type in production config). R-CUST-014 row added to rejection rules table: `[[dtu]] mode='client'` with absent `spec` field → `E-CFG-014`. This eliminates the dual-binding where two distinct conditions mapped to the same error code. |
| v0.3 | C-002/M-006/m-006/m-007 fixes (2026-04-27): Precondition 4 corrected to reflect D-051 — `demo-server` IS in `DTU_DEFAULT_MODE` with `test_only=true`; production validator uses absence-check against production-allowed set, not a denylist. Invariant 3 updated to match. R-CUST-004 clarified: only truly unknown types (not in registry at all) get E-CFG-004. R-CUST-013 added: test-only type in production config → `E-CFG-013`. TV-3.3.004-04 updated: `demo-server` now correctly emits `E-CFG-013` (not E-CFG-004). EC-3.3.004-08: parenthetical hedge removed; E-CFG-013 confirmed as the error for missing spec on client-mode. ADR-007 added to inputs list. Story anchors updated: S-3.3.01 and S-3.3.02. |
| v0.2 | Initial authoring from ADR-010. |
