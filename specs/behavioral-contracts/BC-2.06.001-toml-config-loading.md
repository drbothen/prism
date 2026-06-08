---
document_type: behavioral-contract
level: L3
version: "1.3"
status: active
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-06"
capability: "CAP-009"
lifecycle_status: active
introduced: cycle-1
modified: "2026-06-08"  # v1.3 POL-14 promotion draft→active (S-DEMO-003 merged PR #176 a42e3eaf)
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "c36ec87"
traces_to: ["CAP-009"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.06.001: TOML Configuration Loads and Deserializes at Startup

## Description

At startup, Prism loads and deserializes its TOML configuration file from the path specified
by CLI argument or `PRISM_CONFIG_PATH` environment variable. All `[clients.{id}]` sections
are deserialized into `ClientConfig` structs with validated `client_id` values. The
`[defaults]` section (if present) is loaded for capability merging. Configuration is
immutable after load — there is no hot-reload mechanism in the stdio per-analyst model, and
the loaded config is available to all subsystems throughout the session lifetime.

Fatal errors at load time (file not found, TOML syntax error, permission denied) terminate
startup immediately with a descriptive error message.

## Preconditions
- A TOML configuration file exists at the path specified by CLI argument or `PRISM_CONFIG_PATH` environment variable
- The file is readable by the Prism process

## Postconditions
- All `[clients.{id}]` sections are deserialized into `ClientConfig` structs
- Each `ClientConfig` has a validated `client_id` matching `[a-zA-Z0-9_-]+`
- The `[defaults]` section (if present) is loaded and available for capability merging
- Configuration is immutable after load -- no hot-reload mechanism exists in the stdio per-analyst model
- The loaded config is available to all subsystems (MCP tools, sensor adapters, credential resolution)

## Invariants
- DI-008: Client data separation -- each `ClientConfig` is independently loaded and scoped

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Config` | TOML file not found at specified path | Fatal error: "Configuration file not found at '{path}'" with suggestion to check the path |
| `PrismError::Config` | TOML syntax error (invalid TOML) | Fatal error with the TOML parser error message including line and column |
| `PrismError::Config` | TOML file is not readable (permission denied) | Fatal error: "Cannot read configuration file at '{path}': permission denied" |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-06-001 | Config file exists but contains zero `[clients.*]` sections | Prism starts successfully with an empty client list; queries return "no clients configured" |
| DEC-006 | Operator changes TOML while Prism is running | Running session continues with startup-time config; no hot-reload |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.06.001.

| Scenario | Input | Expected Output |
|----------|-------|----------------|
| Valid config, 2 clients | Well-formed TOML with `[clients.acme]` and `[clients.beta]` | Prism starts; 2 `ClientConfig` structs loaded |
| File not found | Path does not exist | Fatal: "Configuration file not found at '{path}'" |
| TOML syntax error | `api_base = invalid url` (no quotes) | Fatal with TOML parser error including line/column |
| Zero clients | Only `[defaults]` section, no `[clients.*]` | Prism starts; empty client list; no error |

## Verification Properties

No VPs in VP-INDEX v1.5 directly verify TOML config loading. Placeholder for future VP.

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 |
| L2 Invariants | DI-008 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | S-DEMO-003-merged-PR-176 | 2026-06-08 | state-manager | **POL-14 auto-promotion draft→active (D-1055).** S-DEMO-003 squash-merged PR #176 into develop@a42e3eaf. S-DEMO-003 is in `behavioral_contracts:` for this BC (per story frontmatter). The generated `prism.toml` produced by `demo-setup.sh` is schema-valid and accepted at startup — TOML loading is exercised by the full story test suite (4,000+ tests passing, CI 43/43 GREEN). `status: draft → active`; `lifecycle_status: draft → active`. Both fields now consistently `active`. BC v1.2 → v1.3. |
| 1.2 | Wave-5-Phase-B-gate-F-004 | 2026-06-03 | product-owner | F-004 field desync fix (D-989): corrected `lifecycle_status: active` → `lifecycle_status: draft` to match `status: draft`. POL-14 requires at least one merged story citing this BC in its `behavioral_contracts:` array. All four citing stories (S-5.05, S-5.07, S-6.02, S-DEMO-003) are `status: draft` — no POL-14 promotion trigger exists. The config loading behavior is delivered by the running system (boot sequence produces valid TOML-loaded config), but no formal spec-delivery story with this BC in `behavioral_contracts:` has merged. Resolution: `lifecycle_status: draft` pending S-5.05 merge (minimal unblock path). Both fields now consistently `draft`. BC v1.1 → v1.2. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
