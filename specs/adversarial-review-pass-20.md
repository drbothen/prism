---
document_type: adversarial-review
pass: 20
status: complete
novelty: HIGH
findings: 12
critical: 3
high: 6
low: 3
---

# Pass 20 — subsystem 16 integration gaps: sensor ID enum blocks config-driven sensors, missing interface schemas, create_case contradictions

## CRITICAL (3)
- ADV-20-001: create_case interface contradicts BC-2.14.001 on required fields (alert_ids minItems, title required vs optional)
- ADV-20-002: Sensor ID enum in interface definitions is hardcoded to 4 sensors — blocks config-driven sensor addition (CAP-029)
- ADV-20-003: reload_config, add_sensor_spec, list_sensor_specs missing from interface definitions

## HIGH (6)
- ADV-20-004: DI-030/DI-031 atomicity tension — three-tier reload semantics not clearly articulated
- ADV-20-005: E-QUERY-011 and E-SPEC-*/E-RELOAD-* error codes not in taxonomy
- ADV-20-006: PRD says 166 BCs without noting 13 removed (153 active)
- ADV-20-007: set_credential sensor_id enum also blocks config-driven credential setup
- ADV-20-008: Max 16 concurrent schedule executions not in any invariant or NFR
- ADV-20-009: Missing SPEC and RELOAD error code categories in taxonomy

## LOW (3)
- ADV-20-010: Watchdog RSS measurement should not name specific crates
- ADV-20-011: E-CACHE-001 references mutex poisoning but cache may use RwLock (no poison)
- ADV-20-012: query limit 1000 vs materialization 10K — documented, no action needed
