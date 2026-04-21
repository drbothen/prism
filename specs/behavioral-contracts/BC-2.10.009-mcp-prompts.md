---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "8bd996e"
traces_to: ["CAP-034"]
extracted_from: ".factory/specs/prd.md"
origin: greenfield
subsystem: "SS-10"
capability: "CAP-034"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.10.009: MCP Prompts for Common Workflows

## Description

Prism registers at least four static MCP prompts covering common analyst workflows: `triage_alerts`, `investigate_host`, `client_overview`, and `cross_client_status`. Each prompt has a snake_case name, one-line description, and parameterized arguments (e.g., `client_id`, `hostname`, `time_range`). Prompt messages include security reminders about untrusted sensor data per DI-006. Prompts are static (build-time defined, not dynamically generated). An invalid prompt name returns a standard MCP error.

## Preconditions
- MCP prompts are registered in `prompts/list`
- Prompts provide pre-built conversation starters for common analyst workflows

## Postconditions
- The following prompts are registered (at minimum):
  - `triage_alerts`: "Triage open alerts for a client" -- guides the agent through checking all sensors for open high/critical alerts
  - `investigate_host`: "Investigate a specific host across all sensors" -- guides cross-sensor correlation by hostname or IP
  - `client_overview`: "Security posture overview for a client" -- guides pulling alert counts, health status, and recent activity
  - `cross_client_status`: "Cross-client security status" -- guides checking all clients for critical alerts
- Each prompt includes:
  - `name`: snake_case identifier
  - `description`: one-line summary of the workflow
  - `arguments`: parameterized inputs (e.g., `client_id`, `hostname`, `time_range`)
- Prompt messages include security reminders about untrusted sensor data
- Prompts are static (defined at build time, not generated dynamically)

## Invariants
- DI-006: Prompts include reminders to treat sensor data as untrusted

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Prompt not found | Invalid prompt name | MCP error: "Prompt '{name}' not found" |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-10-016 | Prompt references a sensor not configured for the specified client | The prompt generates tool calls; the tool handles the "sensor not configured" case normally |
| EC-10-017 | Prompt argument `client_id` is null | Prompt operates in cross-client mode where applicable |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `prompts/list` | At least 4 entries: triage_alerts, investigate_host, client_overview, cross_client_status | happy-path |
| Invoke `triage_alerts` with valid `client_id` | Prompt messages include security reminder about untrusted sensor data | happy-path |
| Invoke with unknown prompt name | MCP error: "Prompt '{name}' not found" | error |

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vector tables.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (no matching VP) | All prompt messages include DI-006 security reminder | integration test |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-034 |
| L2 Invariants | DI-006 |
| Priority | P1 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial draft |
