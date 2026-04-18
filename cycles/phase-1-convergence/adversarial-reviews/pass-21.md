---
document_type: adversarial-review
pass: 21
status: complete-fixed
novelty: HIGH
findings: 13
critical: 3
high: 7
low: 3
---

# Pass 21 — config-driven sensor consistency, subsystem 16 integration, error taxonomy collisions

## CRITICAL (3) — ALL FIXED

- ADV-21-001: `add_sensor_spec` parameter name mismatch between interface-definitions.md (`spec_toml`) and BC-2.16.008 (`sensor_id` + `spec_content`). **FIX:** Aligned BC to match interface definition (spec_toml, sensor_id extracted from parsed spec).
- ADV-21-002: `E-CAP-001` referenced in 7 BCs but no CAP category in error taxonomy. **FIX:** Updated all 7 BCs to use `E-FLAG-001` (existing, semantically identical).
- ADV-21-003: BC-2.14.001 `create_case` uses E-CASE-001/002/003 for alert validation, but taxonomy uses those codes for case operations. **FIX:** Changed to E-ALERT-001 (existing), E-CASE-014 (new), E-CASE-015 (new).

## HIGH (7) — ALL FIXED

- ADV-21-004: E-SPEC-006 collision (circular step dependency vs custom adapter panic). **FIX:** Added E-SPEC-008 for adapter panics. Updated BC-2.16.004.
- ADV-21-005: E-SPEC-007 collision (OCSF warning vs sensor_id mismatch). **FIX:** Mismatch scenario removed from BC (sensor_id now extracted from spec, not a separate param). Added E-SPEC-009 for duplicate sensor_id.
- ADV-21-006: E-RELOAD-002 semantic collision (BC says "validation failed", taxonomy says "no changes"). **FIX:** Renumbered: E-RELOAD-001=file read error, E-RELOAD-002=validation failed (broken), E-RELOAD-003=partial spec reload, E-RELOAD-004=no changes (cosmetic). Updated BC-2.16.005.
- ADV-21-007: `update_case` missing `client_id` in interface required array (security gap). **FIX:** Added client_id to required array and properties.
- ADV-21-008: BC-2.16.005 labels DI-031 as "all-or-nothing per reload" but DI-031 is three-tier. **FIX:** Updated BC-2.16.005 invariant reference to describe three-tier model.
- ADV-21-009: `sensor_spec.write` capability path referenced but missing from TOML config example. **FIX:** Added to capabilities example.
- ADV-21-010: `update_case` annotation is plain string in interface but structured object in BC. **FIX:** Changed to structured object with type+content. Added missing fields: severity, assignee, link_alert_ids.

## LOW (3) — ALL FIXED

- ADV-21-011: BC-2.16.001 file glob `*.toml` inconsistent with `*.sensor.toml` convention. **FIX:** Changed to `*.sensor.toml`.
- ADV-21-012: get_case timeline event_type enum missing disposition_set, priority_changed, assignee_changed. **FIX:** Extended enum to 7 values.
- ADV-21-013: BC-2.14.003 uses E-CASE-001 for alert-not-found in link_alert_ids. **FIX:** Changed to E-ALERT-001.
