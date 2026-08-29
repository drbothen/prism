---
title: "Claroty xDome — Full Endpoint Expansion Plan"
document_type: objectives-plan
version: "1.1"
created: 2026-08-24
governing_directive: "operator, 2026-08-24"
---

# Claroty xDome — Full Endpoint Expansion Plan

## Scope Status (D-2357, human-directed 2026-08-29)

> **v1-BLOCKING (amended 2026-08-29):** Per D-2357 (human-directed 2026-08-29), all near-term stories (G2–G6) in this plan are now **v1-BLOCKING**, not merely near-term. Wave order: LIMIT → VULNS/G1 → Wave-A (G2) → Wave-B (G3) → Wave-C (G4/G5/G6). The 4 pre-work spikes are also v1-blocking (must complete before their respective story). The deferred DTU-creation stories remain post-v1.

## Governing Directive (operator, 2026-08-24)

SKIP DTU creation for now. Add DTU-build stories as DEFERRED drafts (execute later with the rest of the DTU work). NEAR-TERM: add ALL new tables (TOML `[[tables]]` blocks in `crates/prism-sensors/specs/claroty.sensor.toml`) + tests AGAINST THE LIVE SENSOR (monroe), NOT DTU.

## Current Coverage

4 tables: alerts / audit_logs / devices / device_alert_relations.

Full xDome surface = 64 paths (ref: `.factory/reference/api-specs/xdome_openapi_06.20.2026.json`). Approximately 10 new queryable tables. NO new OCSF `class_selector` arms required (pragmatic mappings).

## Gap Table

| Gap | Table | Path | OCSF class / class_uid | DTU exists? | Size | Notes |
|-----|-------|------|------------------------|-------------|------|-------|
| G1 | claroty_vulnerabilities | POST /api/v1/vulnerabilities/ | vulnerability_finding / 2002 | YES | M (≈spec-only) | Pre-work: vuln `id` vs `name` primary-key spike |
| G2 | claroty_ot_activity_events | POST /api/v1/ot_activity_events/ | detection_finding / 2004 (Option B) | NO | M | Option A network_activity(4001) = new class arm → ADR decision |
| G3 | claroty_device_vulnerability_relations | POST /api/v1/device_vulnerability_relations/ (envelope `devices_vulnerabilities`) | vulnerability_finding / 2002 | NO | L | 214-field join; first-cut ~13 cols |
| G4 | claroty_servers (+server_interfaces) | POST /api/v1/servers/ | inventory_info / 5001 | NO | S-M | Scalar fields only |
| G5 | claroty_organization_zones +3 policy tables | 4× organization_* | entity_management / 3004 | NO | S each | Pre-work: nested-field types spike |
| G6 | claroty_organization_acl_policies | POST /api/v1/organization_acl_policies/ | entity_management / 3004 | NO | S+spike | Atypical: NO pagination fields |

## Near-Term Stories (TOML table + LIVE tests; DTU deferred)

| Story ID | Gap | Notes |
|----------|-----|-------|
| S-CLAROTY-VULNS-001 | G1 | Nearly spec-only since DTU + OCSF class already exist |
| S-CLAROTY-OT-EVENTS-001 | G2 | ADR decision on Option A vs B required first |
| S-CLAROTY-DEVVULNREL-001 | G3 | Large join table; first-cut ~13 cols |
| S-CLAROTY-SERVERS-001 | G4 | Scalar fields, straightforward |
| S-CLAROTY-ORGPOLICY-001 | G5 | Nested field types spike required first |
| S-CLAROTY-ACLPOLICY-001 | G6 | ACL pagination anomaly spike required first |

Wave order: A (G1, G2) → B (G3) → C (G4, G5, G6).

## Deferred DTU-Creation Stories

One per endpoint lacking a DTU (G2, G3, G4, G5, G6). Anchor to the future DTU-parity epic. Add as DRAFT status now; execute with the rest of the DTU work.

## Pre-Work Spikes (run before authoring the affected story)

1. Vuln `id` vs `name` primary key (blocks S-CLAROTY-VULNS-001)
2. ACL pagination anomaly (blocks S-CLAROTY-ACLPOLICY-001)
3. Org-policy nested field types (blocks S-CLAROTY-ORGPOLICY-001)
4. network_activity(4001) ADR — Option A vs B for OT events (blocks S-CLAROTY-OT-EVENTS-001)

## Per-Story Pipeline (no DTU; LIVE tests)

F2 spec (TOML block + BC + remove-uncertainty pass) → F3 story (SAC-1 RG list, `tdd_mode: strict`, live holdout scenarios) → F4 (test-writer LIVE structural tests + implementer TOML block) → LOCAL adversary 3-CLEAN → live validation on monroe (Variant-1 structural + optional Variant-2 agent).

SAP-2 DTU-parity probe is N/A until the deferred DTU stories run.

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.1 | 2026-08-29 | state-manager | D-2357: §Scope Status added — near-term G2–G6 stories + 4 pre-work spikes are now v1-BLOCKING (human-directed 2026-08-29). |
| 1.0 | 2026-08-24 | state-manager | Initial plan — gap table, near-term stories, deferred DTU-creation, pre-work spikes, per-story pipeline. |
