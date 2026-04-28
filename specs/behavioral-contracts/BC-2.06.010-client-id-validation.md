---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-06"
capability: "CAP-009"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "dc078d2"
traces_to: ["CAP-009"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.06.010: Client ID Validation Enforces Allowed Character Set

## Description

The `client_id` value (derived from the TOML key under `[clients.*]`) is validated against
`[a-zA-Z0-9_-]+` and must be non-empty and unique across all configured clients. Valid IDs
are wrapped in the `TenantId` newtype (pre-Wave 3) / `OrgSlug` newtype (Wave 3+, per ADR-006),
which enforces the invariant at the type level throughout the codebase. The same validation
applies to `client_id` / `org_slug` values supplied in MCP tool call parameters at runtime.

> **Wave 3 note (ADR-006):** `TenantId` is renamed to `OrgSlug`; `client_id` fields become
> `org_slug`. The validation regex `[a-zA-Z0-9_-]+` is preserved unchanged. This BC
> describes Wave 1-2 baseline behavior; Wave 3 implementation uses `OrgSlug` with identical
> semantics. Enforced by DI-033 (OrgRegistry Bijectivity) which supersedes the DI-008
> uniqueness constraint for identity validation.

The identifier `__global__` is reserved for internal use (global-scope confirmation tokens)
and cannot be used as a client name in configuration.

## Preconditions
- A `client_id` value is provided in TOML configuration (as a TOML key under `[clients.*]`)
- The config loader is constructing `TenantId` newtype instances

## Postconditions
- The `client_id` is validated against the pattern `[a-zA-Z0-9_-]+`
- The `client_id` is non-empty
- The `client_id` is unique across all configured clients
- Valid IDs are wrapped in the `TenantId` newtype, which enforces the invariant at the type level
- The same validation applies to `client_id` values in MCP tool call parameters at runtime

## Invariants
- DI-008: Client data separation -- validated `TenantId` prevents path traversal or injection via client IDs

## Reserved Identifiers
- `__global__` is a reserved identifier that cannot be used as a client name in configuration. It is used internally as a sentinel value for global-scope operations (aliases, schedules, packs, global-scope rules) in ConfirmationToken `client_id` fields.

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::InvalidInput` | `client_id` contains spaces, dots, slashes, or other disallowed characters | "Invalid client_id '{value}': must match [a-zA-Z0-9_-]+" |
| `PrismError::InvalidInput` | `client_id` is empty | "Invalid client_id: must be non-empty" |
| `PrismError::InvalidInput` | `client_id` is `__global__` (reserved) | "Invalid client_id '__global__': reserved identifier" |
| `PrismError::Config` | Duplicate `client_id` in TOML | "Duplicate client_id '{value}' in configuration" |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-06-018 | `client_id` is a single character (e.g., `a`) | Valid; no minimum length beyond non-empty |
| EC-06-019 | `client_id` contains only hyphens and underscores (e.g., `--__`) | Valid per the pattern; unusual but not prohibited |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.06.010.

| Scenario | Input `client_id` | Expected Result |
|----------|-----------------|----------------|
| Valid ID | `"acme-corp"` | `TenantId("acme-corp")` created |
| Valid, minimal | `"a"` | `TenantId("a")` created |
| Invalid char | `"acme.corp"` | `PrismError::InvalidInput`: must match pattern |
| Reserved | `"__global__"` | `PrismError::InvalidInput`: reserved identifier |
| Empty | `""` | `PrismError::InvalidInput`: must be non-empty |
| Duplicate | Two `[clients.acme]` sections | `PrismError::Config`: duplicate client_id |

## Verification Properties

- **VP-001** (OrgSlug rejects invalid characters) — Kani proof that the `OrgSlug` newtype constructor (formerly `TenantId`) rejects any input containing characters outside `[a-zA-Z0-9_-]`. Anchored to DI-033.

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 |
| L2 Invariants | DI-008, DI-033 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.2 | pass-15-remediation | 2026-04-27 | product-owner | Wave 3 supplement: Description updated with ADR-006 TenantId → OrgSlug note; VP-001 label updated to "OrgSlug rejects invalid characters"; DI-033 added to L2 Invariants (alongside DI-008). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
