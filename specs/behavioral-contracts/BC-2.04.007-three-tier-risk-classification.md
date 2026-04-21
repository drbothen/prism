---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-04"
capability: "CAP-006"
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
input-hash: "365fb25"
traces_to: ["CAP-006"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.04.007: Three-Tier Risk Classification for Operations

## Description

Every MCP tool in Prism is assigned exactly one of three risk tiers at registration time:
Read (no gate), Reversible Write (dry-run default), or Irreversible Write (confirmation token
required). The tier cannot change at runtime and determines the gating mechanism applied at
invocation. Destructive operations (delete sensor, wipe endpoint) are not exposed via MCP at
all. Risk classification is conservative: ambiguous operations are classified as irreversible.

The risk table below is the authoritative classification for all management tools.

## Preconditions
- An MCP tool is being registered or invoked
- Each tool has a defined risk classification

## Postconditions
- All tools are classified into exactly one of three risk tiers:
  - **Read** (no gate): List alerts, get device info, query sensors -- no confirmation required
  - **Reversible Write** (dry-run default): Acknowledge alert, add tag, update alert status -- `dry_run: true` by default
  - **Irreversible Write** (confirmation token): Contain host, quarantine file, network isolation -- two-step confirmation required
- Destructive operations (delete sensor, wipe endpoint) are not exposed via MCP at all
- Risk classification is defined per tool at registration time and cannot change at runtime

## Invariants
- Every tool has exactly one risk tier
- Risk tier determines the gating mechanism; there is no way to bypass the tier's gate

## Management Tool Risk Classification

| Tool | Risk Tier | Gate Mechanism | Notes |
|------|-----------|----------------|-------|
| `create_schedule` | Reversible Write | Dry-run default (BC-2.04.008) | Preview-then-activate pattern |
| `delete_schedule` | Irreversible Write | Confirmation token (BC-2.04.009) | Deletes schedule and associated diff state |
| `create_rule` (analyst/client) | Reversible Write | Dry-run default (BC-2.04.008) | Client-scoped; can be deleted |
| `create_rule` (global) | Irreversible Write | Confirmation token (BC-2.04.009) | Affects all clients |
| `delete_rule` | Irreversible Write | Confirmation token (BC-2.04.009) | Removes rule and stops future detections |
| `create_case` | Immediate (low risk) | No gate | Case creation is additive; no destructive effect |
| `update_case` | Immediate | No gate | Status transitions are audited and reversible (reopen) |
| `case_metrics` | Read | No gate | Read-only aggregation |
| `acknowledge_alert` | Immediate | No gate | Idempotent; no destructive effect |
| `create_pack` | Reversible Write | Dry-run default (BC-2.04.008) | Creates pack config; can be deleted |
| `delete_pack` | Irreversible Write | Confirmation token (BC-2.04.009) | Removes pack and deregisters queries |
| `create_alias` | Reversible Write | Dry-run default (BC-2.04.008) | Creates alias; can be deleted |
| `delete_alias` | Irreversible Write | Confirmation token (BC-2.04.009) | Removes alias permanently |
| `configure_credential_source` | Irreversible Write | Confirmation token (BC-2.04.009) | Updates to existing credential source references require confirmation; new credential source references are immediate |
| `delete_credential` | Irreversible Write | Confirmation token (BC-2.04.009) | Removes credential permanently |
| `add_sensor_spec` | Reversible Write | Dry-run default (BC-2.04.008) | New specs can be removed; replacement of existing spec requires confirmation |
| `reload_config` | Immediate | No gate | Read-only config reload; supports `dry_run` but not gated |
| `crowdstrike_contain_host` | Irreversible Write | Confirmation token (BC-2.04.009) | Representative sensor write action; network isolation |

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| N/A | Agent calls a reversible write without setting `dry_run: false` | Tool executes in dry-run mode (default); returns preview of what would happen |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-04-014 | A tool's risk classification is ambiguous (e.g., could be reversible or irreversible) | Classification is conservative: if uncertain, classify as irreversible (requires confirmation token) |
| EC-04-015 | New sensor write operation added during development | Must be classified before registration; unclassified tools cannot be registered (enforced by type system) |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.04.007.

| Scenario | Tool Invoked | Expected Gate |
|----------|-------------|--------------|
| Read tool | `query_crowdstrike_alerts` | No gate; executes immediately |
| Reversible write, default | `create_schedule` (no `dry_run` param) | `dry_run: true`; returns preview |
| Reversible write, explicit execute | `create_schedule` with `dry_run: false` | Executes immediately |
| Irreversible write | `crowdstrike_contain_host` | Returns `ConfirmationToken`; does not execute |

## Verification Properties

No VPs in VP-INDEX v1.5 directly verify the risk classification tier assignment. Placeholder for future VP covering type-system enforcement that unclassified tools cannot be registered.

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 |
| L2 Invariants | DI-003 |
| Priority | P1 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-72-fix | 2026-04-20 | product-owner | Reordered changelog rows to fully descending (CRIT-001 class scope expansion from pass-71 MED-002 fix). |
| 1.2 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; appended ## Changelog row. |
| 1.1 | Burst 43 | 2026-04-19 | product-owner | P3P41-A-HIGH-001: renamed `set_credential` → `configure_credential_source` in risk-tier table; updated Notes column to reflect source-type reference semantics |
| 1.0 | Phase 1 | 2026-04-14 | product-owner | Initial contract |
