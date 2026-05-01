---
document_type: behavioral-contract
level: L3
version: "0.3"
status: draft
producer: product-owner
timestamp: 2026-04-27T00:00:00
phase: 3.A
inputs:
  - ".factory/specs/architecture/decisions/ADR-010-customer-config-schema.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "900c3d4"
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
bc_id: BC-3.3.003
title: Schema Version Enforcement Rejects Unknown or Missing schema_version
wave: 3
phase: 3.A
date: 2026-04-27
authors: [product-owner]
related_decisions: [D-041]
related_adrs: [ADR-010]
inherits_from: null
superseded_by: null
---

# BC-3.3.003: Schema Version Enforcement Rejects Unknown or Missing schema_version

## Description

Every `customers/*.toml` file must contain a top-level `schema_version` integer field equal to `1`. Wave 3 supports only schema version 1; no other value is accepted. A missing `schema_version` is not silently treated as version 0 — it is an explicit error identical in severity to an unknown version. This single-version rule eliminates version-dispatch logic from the loader and makes migration explicit via the `prism config migrate` CLI command when breaking schema changes are introduced in future waves.

## Preconditions

1. The Prism startup validator is processing a `customers/*.toml` file (same pass as BC-3.3.001 and BC-3.3.002).
2. The current binary supports exactly one schema version: `1`.
3. The supported version constant is compiled into the binary and is not runtime-configurable.

## Postconditions

**On `schema_version` absent:**

1. The process exits with code `1`.
2. Stderr contains `E-CFG-030` with the offending filename and message: `"missing required field 'schema_version'"`.
3. `OrgRegistry` contains zero entries.

**On `schema_version` present but not equal to `1`:**

1. The process exits with code `1`.
2. Stderr contains `E-CFG-031` with the offending filename, the actual value found, and a message indicating the only supported version.
3. If the value is greater than `1` (future schema), the message additionally states: `"Run 'prism config migrate customers/' to upgrade to the current schema version."` — providing a migration path hint.
4. `OrgRegistry` contains zero entries.

**On `schema_version = 1`:**

1. Schema version check passes; validation continues with remaining BC-3.3.001 rules.
2. No error is emitted for this field.

**Rejection rule table:**

| Condition | Error Code | Example Message |
|-----------|------------|-----------------|
| `schema_version` field absent | `E-CFG-030` | `customers/acme.toml: E-CFG-030: missing required field 'schema_version'` |
| `schema_version = 0` | `E-CFG-031` | `customers/acme.toml: E-CFG-031: schema_version 0 is not supported; only schema_version 1 is supported in this binary` |
| `schema_version = 2` | `E-CFG-031` | `customers/acme.toml: E-CFG-031: schema_version 2 is not supported; only schema_version 1 is supported. Run 'prism config migrate customers/' to upgrade.` |
| `schema_version = -1` | `E-CFG-031` | `customers/acme.toml: E-CFG-031: schema_version -1 is not supported; only schema_version 1 is supported in this binary` |
| `schema_version` is a string (e.g., `"1"`) | `E-CFG-000` (TOML type error) | `customers/acme.toml: E-CFG-000: TOML parse error: expected integer for schema_version, got string` |

## Invariants

1. At most one schema version is supported by any given binary release — there is no version-dispatch logic (ADR-010 §2.6).
2. A missing `schema_version` MUST produce an error; it is never silently defaulted to any version value.
3. The migration hint message ("Run 'prism config migrate'") MUST appear in the error for `schema_version > 1` to give operators an actionable path forward.
4. `schema_version` check runs as the first field validation (before UUID checks, slug checks, etc.) to give operators the clearest possible signal when files are from a different schema generation.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-3.3.003-01 | `schema_version = 1` | Passes; validation continues normally |
| EC-3.3.003-02 | `schema_version = 0` | `E-CFG-031`; no migration hint (version 0 is not a future version) |
| EC-3.3.003-03 | `schema_version = 2` (future) | `E-CFG-031` with migration hint message |
| EC-3.3.003-04 | `schema_version` field completely absent | `E-CFG-030`; distinct from unknown version error |
| EC-3.3.003-05 | `schema_version = 1` but file has other validation errors | `E-CFG-030`/`E-CFG-031` NOT emitted; other errors from BC-3.3.001 are collected and reported |
| EC-3.3.003-06 | `schema_version = "1"` (string instead of integer) | `E-CFG-000` TOML parse error; serde rejects wrong type |
| EC-3.3.003-07 | `schema_version = 1.0` (float instead of integer) | `E-CFG-000` TOML parse error; serde rejects wrong type |
| EC-3.3.003-08 | Large future version: `schema_version = 999` | `E-CFG-031` with migration hint; no special handling for large values |

## Canonical Test Vectors

| TV-ID | Input | Expected Output | Category |
|-------|-------|-----------------|----------|
| TV-3.3.003-01 | `customers/acme.toml` with `schema_version = 1` (all other fields valid) | Exit 0; schema version check passes | happy-path |
| TV-3.3.003-02 | `customers/acme.toml` with `schema_version = 0` | Exit 1; stderr contains `E-CFG-031`; states only version 1 is supported | error |
| TV-3.3.003-03 | `customers/acme.toml` with `schema_version = 2` | Exit 1; stderr contains `E-CFG-031`; contains migration hint `'prism config migrate'` | error |
| TV-3.3.003-04 | `customers/acme.toml` with `schema_version` field absent | Exit 1; stderr contains `E-CFG-030`; distinguishable from `E-CFG-031` | error |
| TV-3.3.003-05 | `customers/acme.toml` with `schema_version = "1"` (string) | Exit 1; stderr contains `E-CFG-000` TOML type mismatch | error |
| TV-3.3.003-06 | `customers/acme.toml` with `schema_version = -1` | Exit 1; stderr contains `E-CFG-031` | error |
| TV-3.3.003-07 | Two files: one with `schema_version = 1` (valid), one with `schema_version = 2` (invalid) | Exit 1; `E-CFG-031` for the invalid file only; valid file does not produce a schema_version error | edge-case |

## Verification Properties

| VP | Property | Proof Method |
|----|----------|--------------|
| VP-3.3.003-A | For all integer `schema_version` values not equal to 1, exit code is 1 | proptest with integer generator excluding 1 |
| VP-3.3.003-B | Absent `schema_version` produces `E-CFG-030`, not `E-CFG-031` (distinct error codes for absent vs. wrong value) | unit test |
| VP-3.3.003-C | `schema_version = 1` never produces a schema-version error regardless of other field values | proptest |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 ("Client Configuration") per capabilities.md §CAP-009 |
| Capability Anchor Justification | CAP-009 ("Client Configuration") per capabilities.md §CAP-009 — this BC specifies the schema version gate on customer config loading, which is part of the "Load and validate per-client sensor mappings, credential references, and capability overrides from TOML configuration" lifecycle defined in CAP-009. |
| L2 Domain Invariants | N/A (Wave 3 new capability; DI-NNN assignment pending domain-spec Wave 3 extension) |
| Architecture Module | SS-06 (Client Configuration) per ARCH-INDEX.md |
| Stories | S-3.3.01 |

## Related BCs

- BC-3.3.001 — composes with (schema_version check runs first within the same validation pass)
- BC-3.3.002 — related to (same validation pass)

## Architecture Anchors

- ADR-010 §2.2 — `schema_version` as required integer field; MUST equal `1` for Wave 3
- ADR-010 §2.6 — Schema Versioning design: single-version rule, migrator CLI pattern
- ADR-010 §3.3 — Threat model: schema version forgery / downgrade attack mitigation

## Story Anchor

S-3.3.01

## VP Anchors

- VP-3.3.003-A — proptest: all non-1 integers produce exit 1
- VP-3.3.003-B — unit: absent vs. wrong value produce distinct error codes
- VP-3.3.003-C — proptest: schema_version=1 never produces schema-version error

## BC Changelog

| Version | Change |
|---------|--------|
| v0.3 | M-004 (Pass 5): Frontmatter `title:` corrected to title-case to match H1 heading. |
| v0.2 | Initial authoring from ADR-010. |
