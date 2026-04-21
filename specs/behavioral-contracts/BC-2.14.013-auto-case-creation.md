---
document_type: behavioral-contract
level: L3
version: "1.4"
status: draft
producer: product-owner
timestamp: 2026-04-16T12:00:00
phase: 2-patch
origin: greenfield
subsystem: "SS-14"
capability: "CAP-022"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "2d476ce"
traces_to:
  - "CAP-022"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.14.013: Auto-Case-Creation from High-Severity Detection Rules

## Description

When a detection rule fires and generates an alert at CRITICAL severity (configurable
threshold, default: `severity_id >= 4` corresponding to OCSF severity High/Critical),
a case is automatically created and linked to the triggering alert. Deduplication
prevents a new case from being created if an open case already references an alert
from the same rule for the same client within the dedup window. The case inherits
metadata from the triggering alert. Auto-case-creation is idempotent when a rule
re-fires: the same alert pattern within the dedup window does not create a second case.

Note: As recorded in CAP-022, the authority on this behavior, key design questions
were tracked for story decomposition: severity threshold (now configurable),
deduplication strategy (same rule + client + open case within window), capability
gate bypass (system-generated cases bypass the confirmation token flow), and
auto-title generation (template from rule meta).

## Preconditions

- A detection rule with `auto_case: true` in its meta block (or the global
  `[detection.auto_case_threshold]` is configured and the alert severity meets the threshold)
- The detection rule fires and generates an alert via BC-2.13.005
- The alert's `severity_id` meets or exceeds the configured auto-case threshold
  (default: `severity_id >= 4`, i.e., High or Critical)
- The `cases` RocksDB column family is initialized (BC-2.15.001)

## Postconditions

- **No existing open case (dedup miss):**
  - A new case is created automatically with:
    - `title`: interpolated from rule meta template or default:
      `"[Auto] {rule_name} — {severity_label} alert on {client_id}"`
    - `description`: alert description or empty string
    - `status`: `New`
    - `severity`: inherited from the triggering alert's severity
    - `client_id`: from the triggering alert
    - `source_alert_ids`: `[alert.alert_id]` (the triggering alert)
    - `rule_id`: from the triggering rule (stored in case metadata for dedup lookups)
    - `created_at`: current UTC timestamp
    - `auto_created: true` flag in case metadata (distinguishes from analyst-created cases)
  - Case is persisted to RocksDB `cases` CF atomically with the alert record
  - A timeline annotation is added: `{ type: "note", body: "Auto-created from detection rule '{rule_id}'", author: "prism-system" }`
  - An audit event is emitted: `case_auto_created` with `rule_id`, `alert_id`, `client_id`, `case_id`
- **Existing open case found (dedup hit):**
  - The triggering alert is linked to the existing open case (added to `source_alert_ids`)
  - The existing case's timeline receives an annotation: `{ type: "alert_link", alert_id, body: "Alert auto-linked by rule re-fire" }`
  - NO new case is created
  - An audit event is emitted: `case_auto_alert_linked` with `rule_id`, `alert_id`, `client_id`, `case_id`
- Auto-case-creation does NOT require a confirmation token (system-generated, not analyst-initiated)
- Auto-case-creation is NOT subject to the `case.write` capability gate for system-generated events;
  it is treated as an internal platform action (separate from analyst MCP tool writes)

## Invariants

- Auto-case-creation is idempotent: the same triggering pattern within the dedup window
  produces exactly one case
- Dedup key: `(rule_id, client_id)` + open case status check. If any case with status
  in `{New, Acknowledged, Investigating}` already references the same `rule_id` for the same
  `client_id`, the dedup fires and the new alert is linked to the existing case
- The dedup check is performed atomically with case creation (RocksDB WriteBatch)
- Alert persistence (BC-2.13.005) MUST complete before auto-case-creation begins
  (alert-before-case ordering)
- `auto_created: true` cases are visible in `list_cases` and `get_case` with no
  filtering; they are fully manageable by analysts

## Error Conditions

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-STORE-002` | RocksDB write failure during case creation | Case creation fails; alert is still persisted; `ERROR` log: "Auto-case-creation failed for alert '{alert_id}': {error}"; the alert does NOT re-trigger case creation on its own |
| — | Dedup query fails (RocksDB read error) | Log `ERROR`; fall through to case creation (fail-open on dedup — prefer creating a duplicate case over silently dropping) |
| — | `rule_id` not found in active rules (rule deleted between alert generation and case creation) | Case still created; `rule_id` recorded in metadata; case title uses `"[Auto] {alert.title}"` |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-14-046 | Same rule fires 5 times within 1 minute for the same client | First fire: case created. Fires 2-5: alerts linked to existing open case. 1 case, 5 linked alerts. |
| EC-14-047 | Open case transitions to `Resolved`; rule fires again | Resolved case is NOT in the open set `{New, Acknowledged, Investigating}`; new case created. |
| EC-14-048 | Alert severity exactly at threshold (`severity_id = 4`, threshold = 4) | Auto-case-creation triggers (threshold is `>=`, inclusive) |
| EC-14-049 | Alert severity below threshold (`severity_id = 3`, threshold = 4) | No auto-case-creation; alert persisted normally; no case action |
| EC-14-050 | Detection rule has `auto_case: false` explicitly set | No auto-case-creation even if severity is CRITICAL |
| EC-14-051 | 100 concurrent CRITICAL alerts from same rule for same client | First to acquire RocksDB WriteBatch lock creates the case; remaining 99 are dedup-hit and linked to the created case |
| EC-14-052 | Auto-case-creation configured globally (`[detection.auto_case_threshold]`) with per-rule override (`auto_case: false`) | Per-rule override takes precedence; rule with `auto_case: false` does not auto-create even at CRITICAL severity |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — first fire | rule fires at severity_id=4, no open case | Case created with auto_created=true, alert linked |
| Dedup hit — re-fire | same rule fires again; open case exists | Alert linked to existing case; no new case |
| Resolved case then re-fire | existing case is Resolved; rule fires | New case created (Resolved not in open set) |
| Below threshold | severity_id=3, threshold=4 | No case action; alert persisted only |
| Per-rule override | rule has `auto_case: false`, severity=CRITICAL | No auto-case-creation |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| VP-060 | Pure decision function decide_dedup_action() returns Link iff existing case within window, Create otherwise. Proptest. P0. Effectful wrapper CaseDedupRegistry::check_and_create() integration-tested via S-4.06 AC-12/AC-12b. |

## Related BCs

- BC-2.13.005 — Alert Generation (auto-case-creation is triggered after alert persistence)
- BC-2.14.001 — `create_case` MCP Tool (shares same case creation code path; `auto_created` flag distinguishes)
- BC-2.14.009 — Case Persistence (RocksDB storage for auto-created cases)
- BC-2.04.008 — Dry-Run Default (does NOT apply to system-generated auto-case creation)
- BC-2.05.001 — Audit Entry per Tool Invocation (auto-case-creation emits audit event outside MCP tool context via internal audit path)

## Architecture Anchors

- AD-004: RocksDB WriteBatch for atomic alert + case creation
- AD-021: Actions — case triggers (auto-case-creation is a system-level trigger, distinct from action triggers)
- `specs/architecture/operational-pipeline.md` — detection → alert → auto-case flow
- CAP-022: Case Management (auto-case-creation documented in capability definition)

## Story Anchor

S-4.06 — prism-operations: Case Management

## VP Anchors

- VP-060: Pure decision function decide_dedup_action() — proptest verifies Link iff existing case within window, Create otherwise. Effectful wrapper CaseDedupRegistry::check_and_create() integration-tested via S-4.06 AC-12/AC-12b.

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-022 |
| L2 Invariants | DI-004, DI-008 |
| ADR | AD-004 |
| Story | S-4.06 |
| Priority | P1 |
| Notes | Tracked in CAP-022: "Auto-case-creation requires a dedicated BC during story decomposition" — this BC fulfills that tracking note |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.4 | pass-74-defer-close | 2026-04-20 | product-owner | Closed DEFER: VP-060 created (Proptest P0) verifies pure decide_dedup_action() function; effectful wrapper integration-tested. |
| 1.3 | pass-74-fix | 2026-04-20 | product-owner | Resolved (placeholder) row in ## Verification Properties per pass-74 VP-TBD decision matrix extension. |
| 1.2 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; renamed Error Cases → Error Conditions; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-16 | product-owner | Initial phase-2-patch BC |
