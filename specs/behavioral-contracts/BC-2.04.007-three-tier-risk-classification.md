---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Feature Flag System"
capability: "CAP-006"
---

# BC-2.04.007: Three-Tier Risk Classification for Operations

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

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| N/A | Agent calls a reversible write without setting `dry_run: false` | Tool executes in dry-run mode (default); returns preview of what would happen |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-04-014 | A tool's risk classification is ambiguous (e.g., could be reversible or irreversible) | Classification is conservative: if uncertain, classify as irreversible (requires confirmation token) |
| EC-04-015 | New sensor write operation added during development | Must be classified before registration; unclassified tools cannot be registered (enforced by type system) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 |
| L2 Invariants | DI-003 |
| Priority | P1 |
