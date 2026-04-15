---
document_type: adversarial-review
pass: 22
status: complete-fixed
novelty: HIGH
findings: 12
critical: 3
high: 7
low: 2
---

# Pass 22 — data flow integrity, lifecycle handoffs, error code referential integrity

## CRITICAL (3)

- ADV-22-001: DI-025 references E-CASE-001 (not found) for invalid state transitions — should be E-CASE-004
- ADV-22-002: E-SPEC-002/003 collision — BCs use different meanings than taxonomy. BCs not updated after E-SPEC-009 was added
- ADV-22-003: sensor_id validation pattern contradiction — entities/interfaces use lowercase-only `^[a-z]`, SensorSpec entity and BC-2.16.009 allow mixed-case `[a-zA-Z0-9_-]+`

## HIGH (7)

- ADV-22-004: BC-2.16.005 reload_config still uses `*.toml` glob instead of `*.sensor.toml`
- ADV-22-005: Alert `resolved_at` set "when linked case reaches Resolved" but no BC implements this propagation
- ADV-22-006: Severity enum inconsistency — create_rule uses info/low/med/high/critical, create_case missing info, update_case uses "informational"
- ADV-22-007: Auto-case-creation from high-severity alerts declared in CAP-022 but has no BC
- ADV-22-008: EC-14-019 defines MTTR as `closed_at - created_at`, BC-2.14.008 says `resolved_at - created_at`
- ADV-22-009: Silent diff skip prevents correlation window cleanup — detection engine never invoked when data unchanged
- ADV-22-010: BC-2.14.001 uses non-standard `notifications/case/created` instead of MCP-standard `notifications/resources/updated`

## LOW (2)

- ADV-22-011: DEC-023 says "rejected before materialization" but BC says abort happens during fan-out (mid-materialization)
- ADV-22-012: BC-2.04.004 EC-04-008 references removed "active client" context switching concept
