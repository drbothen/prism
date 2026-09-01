---
document_type: holdout-scenario-index
level: L3
version: "1.34"
status: draft
producer: product-owner
timestamp: 2026-09-01T19:35:20Z
phase: 3
inputs: []
input-hash: null
traces_to: prd.md
total_scenarios: 118
---

# Holdout Scenario Index -- Prism

**Date:** 2026-08-30 (updated)
**Phase:** 0 (Multi-Repo Synthesis -- Step 5) / Phase 4.B (Wave 4 Holdout Coverage) / Phase 3 Wave 0 Plugin Migration / Phase 3 DRIFT-CLAROTY-AUDITLOG-TIMEOUT-001 / Phase 3 S-ADR058-OCSF-ROUTING-001 A+W re-gate / Phase 3 Wave A xDome Expansion (S-CLAROTY-VULNS-001 + S-CLAROTY-OT-EVENTS-001) / Phase 3 Wave B xDome Expansion (S-CLAROTY-DEVVULNREL-001) / Phase 3 Wave C xDome Expansion (S-CLAROTY-SERVERS-001 + S-CLAROTY-ORGPOLICY-001) / Phase 3 S-ENGINE-LIMIT-EARLY-STOP-001 holdout backfill
**Total Scenarios:** 118 (115 prior + 3 new HS-030 for S-ENGINE-LIMIT-EARLY-STOP-001)
**Total Groups:** 24
**Input Sources:** 9 pass-8 deep synthesis files, cross-repo-dependencies.md, unified-security-posture.md; Wave 4 stories S-4.01–S-4.08, BC-INDEX v4.32, ADR-013 §2.1, D-209, ADR-016 §2.5, ADR-008; FB-IMPL-P1-PO fix-burst-1 2026-05-20 (HS-013..HS-018 authored)

---

## Scenario Group Summary

| Group | File | Title | Scenarios | Priority | Key Risk |
|-------|------|-------|-----------|----------|----------|
| HS-001 | [HS-001-happy-path.md](HS-001-happy-path.md) | Happy Path | 6 | P0 | Basic MCP-to-sensor-to-OCSF pipeline |
| HS-002 | [HS-002-multi-sensor.md](HS-002-multi-sensor.md) | Multi-Sensor | 5 | P0 | Cross-sensor consistency and independence |
| HS-003 | [HS-003-multi-tenant.md](HS-003-multi-tenant.md) | Multi-Tenant | 7 | P0 | MSSP client data isolation |
| HS-004 | [HS-004-credential-lifecycle.md](HS-004-credential-lifecycle.md) | Credential Lifecycle | 6 | P0 | Per-tenant credential CRUD and rotation |
| HS-005 | [HS-005-failure-scenarios.md](HS-005-failure-scenarios.md) | Failure Scenarios | 7 | P0 | Sensor down, auth expired, rate limited, timeout |
| HS-006 | [HS-006-state-recovery.md](HS-006-state-recovery.md) | State Recovery | 6 | P0 | Restart resilience, cursor forward progress |
| HS-007 | [HS-007-cross-repo-failure.md](HS-007-cross-repo-failure.md) | Cross-Repo Failure | 8 | P1 | Patterns from one repo failing in unified context |
| HS-008 | [HS-008-contract-violation.md](HS-008-contract-violation.md) | Contract Violation | 8 | P1 | OCSF/proto/API schema mismatches |
| HS-009 | [HS-009-scheduler-operations.md](HS-009-scheduler-operations.md) | Scheduler Operations | 6 | P0 | 60s tick regression (ADR-013 §2.1), ScheduleFireMissed, multi-tenant CF key isolation |
| HS-010 | [HS-010-detection-alert-pipeline.md](HS-010-detection-alert-pipeline.md) | Detection & Alert Pipeline | 6 | P0 | alert_id UUID v7 idempotency, dedup correctness, three-scope rule isolation |
| HS-011 | [HS-011-case-management.md](HS-011-case-management.md) | Case Management | 5 | P0 | 5-state machine enforcement, timeline_entry_id idempotency, MTTR accuracy |
| HS-012 | [HS-012-action-delivery.md](HS-012-action-delivery.md) | Action Delivery | 6 | P0 | D-209 semaphore independence, VP-045 non-blocking, ADR-016 §2.5 discriminator FSM |
| HS-013 | [HS-013-crowdstrike-dtu-parity.md](HS-013-crowdstrike-dtu-parity.md) | CrowdStrike DTU Parity | 2 | P0 | Two-step pipeline spec parity; batch cap at CROWDSTRIKE_BATCH_SIZE (100) |
| HS-014 | [HS-014-claroty-post-for-read-parity.md](HS-014-claroty-post-for-read-parity.md) | Claroty POST-for-Read Parity | 2 | P0 | POST-for-read pattern; polymorphic ID (integer vs UUID string) normalization |
| HS-015 | [HS-015-cyberint-alerts-cursor-parity.md](HS-015-cyberint-alerts-cursor-parity.md) | Cyberint Alerts Cursor Parity | 3 | P0 | Multi-format timestamp parsing; SKIP verdict for incidents table; cookie_roundtrip auth; GET /api/v1/alerts |
| HS-016 | [HS-016-armis-aql-timestamp-fallback-parity.md](HS-016-armis-aql-timestamp-fallback-parity.md) | Armis AQL + Timestamp Fallback Parity | 3 | P0 | AQL forwarding via `${query.filter.aql}`; fallback chain; WARN audit signal; bearer_static auth; DTU gaps DTU-EXT-003/004 noted |
| HS-017 | [HS-017-bundled-spec-validation-gate.md](HS-017-bundled-spec-validation-gate.md) | Bundled Spec Validation CI Gate | 2 | P0 | Negative: malformed specs rejected by BC-2.16.009 (E-SPEC-002, E-SPEC-003) |
| HS-018 | [HS-018-spec-id-filename-mismatch-rejection.md](HS-018-spec-id-filename-mismatch-rejection.md) | Spec sensor_id / Filename Mismatch | 3 | P0 | Negative: sensor_id ≠ filename stem rejected at load time (E-SPEC-017); v1.4 sweeps error-taxonomy.md v1.42→v1.44 at 3 active-prose sites (FB-IMPL-9 transitive cite-pin chain) |
| HS-019 | [S-CLAROTY-AUDITLOG-TIMEBOX-001-HS-001-count-star-no-timeout.md](S-CLAROTY-AUDITLOG-TIMEBOX-001-HS-001-count-star-no-timeout.md), [S-CLAROTY-AUDITLOG-TIMEBOX-001-HS-002-bounded-default-window.md](S-CLAROTY-AUDITLOG-TIMEBOX-001-HS-002-bounded-default-window.md), [S-CLAROTY-AUDITLOG-TIMEBOX-001-HS-003-explicit-time-filter-not-truncated.md](S-CLAROTY-AUDITLOG-TIMEBOX-001-HS-003-explicit-time-filter-not-truncated.md), [S-CLAROTY-AUDITLOG-TIMEBOX-001-HS-004-time-range-both-bounds.md](S-CLAROTY-AUDITLOG-TIMEBOX-001-HS-004-time-range-both-bounds.md) | Claroty audit_logs Push-Down Fix — Single Story (S-CLAROTY-AUDITLOG-TIMEBOX-001) **[CONSUMED 2026-08-15 — D-2195 holdout gate PASS; all 4 scenarios single-use; must NOT be reused]** | 4 | P0 | Push-down fix: COUNT(*) no timeout; unbounded SELECT bounded to 7d default; explicit older-than-7d filter honored (no silent truncation); BETWEEN a AND b pushes both bounds; ASM-CLAROTY-AUDITLOG-001 field-name validation |
| ~~HS-020~~ | ~~Story B scenarios~~ | ~~Claroty audit_logs Layer 2 — Dynamic Push-Down~~ | ~~2~~ | ~~P0~~ | **RETIRED 2026-08-15 before shipping:** Design reworked to single story. All 4 scenarios consolidated into HS-019 under S-CLAROTY-AUDITLOG-TIMEBOX-001. ID reserved per append_only_numbering (DF-030). |
| HS-021 | [S-ADR058-OCSF-COERCION-001-HS-001-string-column-object-input-null-cell.md](S-ADR058-OCSF-COERCION-001-HS-001-string-column-object-input-null-cell.md), [S-ADR058-OCSF-COERCION-001-HS-002-integer-column-parseable-string-no-data-loss.md](S-ADR058-OCSF-COERCION-001-HS-002-integer-column-parseable-string-no-data-loss.md), [S-ADR058-OCSF-COERCION-001-HS-003-integer-column-non-parseable-string-null-warn.md](S-ADR058-OCSF-COERCION-001-HS-003-integer-column-non-parseable-string-null-warn.md), [S-ADR058-OCSF-COERCION-001-HS-004-enrich1-array-column-json-list-preserved.md](S-ADR058-OCSF-COERCION-001-HS-004-enrich1-array-column-json-list-preserved.md) | Column Coercion Gap Closure — Single Story (S-ADR058-OCSF-COERCION-001) | 4 | P0 | String+Object null-demotion (AC-005); Integer+parseable-string no-loss (AC-007 happy); Integer+non-parseable-string null+warn (AC-007 fail); ENRICH-1 Array non-regression after Object arm insertion |
| HS-022 | [S-ADR058-OCSF-ROUTING-001-HS-001-claroty-alerts-ocsf-field-name-wire-shape.md](S-ADR058-OCSF-ROUTING-001-HS-001-claroty-alerts-ocsf-field-name-wire-shape.md), [S-ADR058-OCSF-ROUTING-001-HS-002-prism-describe-raw-extensions-tier2-prohibition.md](S-ADR058-OCSF-ROUTING-001-HS-002-prism-describe-raw-extensions-tier2-prohibition.md), [S-ADR058-OCSF-ROUTING-001-HS-003-claroty-audit-logs-class-uid-3004-wire-shape.md](S-ADR058-OCSF-ROUTING-001-HS-003-claroty-audit-logs-class-uid-3004-wire-shape.md), [S-ADR058-OCSF-ROUTING-001-HS-004-claroty-devices-device-type-label-filter.md](S-ADR058-OCSF-ROUTING-001-HS-004-claroty-devices-device-type-label-filter.md) | OCSF Field-Name Routing — Single Story (S-ADR058-OCSF-ROUTING-001) **[CONSUMED — D-2270; A+W amendment (2026-08-23) post-dates HS-022; re-gate required via HS-023; must NOT be reused]** | 4 | P0 | Claroty alerts ocsf_field Arrow name wire shape (AC-003); prism_describe raw_extensions Tier-2 prohibition (AC-006/AC-007b); claroty audit_logs class_uid 3004 wire shape (AC-009); claroty devices device_type_label filter (AC-005/AC-010) |
| HS-023 | [S-ADR058-OCSF-ROUTING-001-B-HS-001-zero-tier1-aw-warning-and-available-set.md](S-ADR058-OCSF-ROUTING-001-B-HS-001-zero-tier1-aw-warning-and-available-set.md), [S-ADR058-OCSF-ROUTING-001-B-HS-002-spec-load-j4-collision-e-spec-030-rejection.md](S-ADR058-OCSF-ROUTING-001-B-HS-002-spec-load-j4-collision-e-spec-030-rejection.md), [S-ADR058-OCSF-ROUTING-001-B-HS-003-audit-logs-metadata-uid-wire-shape-and-e-query-038-available-columns.md](S-ADR058-OCSF-ROUTING-001-B-HS-003-audit-logs-metadata-uid-wire-shape-and-e-query-038-available-columns.md) | OCSF Field-Name Routing A+W Re-Gate — Single Story (S-ADR058-OCSF-ROUTING-001) — HIDDEN, SINGLE-USE | 3 | P0 | Zero-Tier-1 A+W warning + raw_extensions available-set (EC-11-080 A+W sub-case); §J4 intra-table collision E-SPEC-030 rejection at spec-load (ADR-058 §J7); audit_logs metadata_uid OQ-005 wire shape + E-QUERY-038 available_columns (EC-11-079) |
| HS-024 | [S-CLAROTY-VULNS-001-HS-001-vulnerability-wire-shape-class-uid.md](S-CLAROTY-VULNS-001-HS-001-vulnerability-wire-shape-class-uid.md), [S-CLAROTY-VULNS-001-HS-002-finding-info-title-queryable.md](S-CLAROTY-VULNS-001-HS-002-finding-info-title-queryable.md), [S-CLAROTY-VULNS-001-HS-003-tier2-raw-extensions-content.md](S-CLAROTY-VULNS-001-HS-003-tier2-raw-extensions-content.md) | Claroty xDome Vulnerabilities Table — Single Story (S-CLAROTY-VULNS-001) — HIDDEN, SINGLE-USE | 3 | P0 | class_uid=2002 wire shape and finding_info_title Tier-1 present (BC-2.16.015 §PC1/2); Tier-1 OCSF column queryable; no E-QUERY-038 on finding_info_title; Tier-2 raw_extensions non-empty with cve_ids + severity_score keys |
| HS-025 | [S-CLAROTY-OT-EVENTS-001-HS-001-ot-event-wire-shape-class-uid.md](S-CLAROTY-OT-EVENTS-001-HS-001-ot-event-wire-shape-class-uid.md), [S-CLAROTY-OT-EVENTS-001-HS-002-tier2-source-ip-not-standalone.md](S-CLAROTY-OT-EVENTS-001-HS-002-tier2-source-ip-not-standalone.md), [S-CLAROTY-OT-EVENTS-001-HS-003-detection-time-time-column.md](S-CLAROTY-OT-EVENTS-001-HS-003-detection-time-time-column.md) | Claroty xDome OT Activity Events Table — Single Story (S-CLAROTY-OT-EVENTS-001) — HIDDEN, SINGLE-USE, NO DTU **[PARTIALLY CONSUMED 2026-08-31 — D-2399: HS-002 PASS 1.00 (consumed), HS-003 PASS 1.00 (consumed); HS-001 SETUP-FAILURE (quiescent OT network, 0 events — NOT consumed; kept for future re-run when monroe OT network has ≥1 event). Human ACCEPTED (Option-1): gate treated as PASSED on 2/3-at-ceiling + structural corroboration]** | 3 | P0 | class_uid=2004 Option B wire shape + finding_info_uid REQUIRED Tier-1 present (BC-2.16.016 §PC1/2/3); Tier-2 network field (source_ip) raises E-QUERY-038 + raw_extensions queryable; Tier-1 `time` OCSF rename accepts; `detection_time` raw name rejected |
| HS-026 | [S-CLAROTY-DEVVULNREL-001-HS-001-wire-shape-class-uid-finding-info-title.md](S-CLAROTY-DEVVULNREL-001-HS-001-wire-shape-class-uid-finding-info-title.md), [S-CLAROTY-DEVVULNREL-001-HS-002-join-key-raw-extensions-content.md](S-CLAROTY-DEVVULNREL-001-HS-002-join-key-raw-extensions-content.md), [S-CLAROTY-DEVVULNREL-001-HS-003-detection-date-time-tier1-rename.md](S-CLAROTY-DEVVULNREL-001-HS-003-detection-date-time-tier1-rename.md) | Claroty xDome Device-Vulnerability Relations Table — Single Story (S-CLAROTY-DEVVULNREL-001) — HIDDEN, SINGLE-USE, NO DTU **[CONSUMED 2026-08-31 — D-2403: HS-001 PASS 1.00 (consumed; class_uid=2002 + finding_info_title CVE string), HS-002 PASS 0.75 at-threshold (consumed; VERDICT-A over-specified — vulnerability_name is Tier-1 finding_info_title NOT in raw_extensions; HS-002 v1.0→v1.1 product-owner amendment), HS-003 PASS 1.00 (consumed; SELECT time accepted; SELECT device_vulnerability_detection_date E-QUERY-038). Mean satisfaction 0.92. GATE PASS. Must NOT be reused.]** | 3 | P0 | class_uid=2002 wire shape + finding_info_title REQUIRED Tier-1 present (BC-2.16.017 §PC1/2); vulnerability_name join-key accessible as Tier-1 finding_info_title (NOT in raw_extensions — ADR-058 §B2); device_uid in raw_extensions Tier-2; SELECT device_uid raises E-QUERY-038; Tier-1 rename: SELECT time accepts; SELECT device_vulnerability_detection_date rejected |
| HS-027 | [S-CLAROTY-SERVERS-001-HS-001-servers-wire-shape-class-uid-5001.md](S-CLAROTY-SERVERS-001-HS-001-servers-wire-shape-class-uid-5001.md), [S-CLAROTY-SERVERS-001-HS-002-servers-tier1-device-name-rename.md](S-CLAROTY-SERVERS-001-HS-002-servers-tier1-device-name-rename.md), [S-CLAROTY-SERVERS-001-HS-003-server-interfaces-separate-endpoint-tier2-plan-gate.md](S-CLAROTY-SERVERS-001-HS-003-server-interfaces-separate-endpoint-tier2-plan-gate.md) | Claroty xDome Servers + Server Interfaces Tables — Single Story (S-CLAROTY-SERVERS-001) — HIDDEN, SINGLE-USE, NO DTU **[CONSUMED 2026-09-01 — D-2406: HS-001 PASS 1.00 (consumed; class_uid=5001 + device_name Tier-1 + raw_extensions JSON-serialized string with inventory keys; HS-001 v1.0→v1.1 raw_extensions wire-encoding corrected by product-owner), HS-002 PASS 1.00 (consumed; server_name rejected E-QUERY-038 with device_name in available_columns; SELECT device_name and status_code accepted), HS-003 PASS 1.00 (consumed; server_interfaces separate endpoint queryable; interface_status rejected E-QUERY-038 with status_code in available_columns; raw_extensions JSON-serialized string with interface Tier-2 keys; HS-003 v1.0→v1.1 raw_extensions wire-encoding corrected by product-owner). Mean satisfaction 1.00 all 3 must_pass. GATE PASS. Must NOT be reused.]** | 3 | P0 | claroty_servers SELECT * wire shape: class_uid=5001 + device_name Tier-1 REQUIRED + raw_extensions JSON-serialized string (Arrow Utf8; ADR-058 §I2) with inventory keys (BC-2.16.018 §PC1/2); claroty_servers Tier-1 rename: server_name rejected E-QUERY-038 + device_name and status_code accepted (BC-2.16.018 §PC2 + §Invariants); claroty_server_interfaces: separate /api/v1/server_interfaces/ endpoint queryable + class_uid=5001 + interface_status rejected E-QUERY-038 + raw_extensions JSON-serialized string with interface keys (BC-2.16.019 §PC1/2/3) |
| HS-028 | [S-CLAROTY-ORGPOLICY-001-HS-001-zone-wire-shape-class-uid-3004.md](S-CLAROTY-ORGPOLICY-001-HS-001-zone-wire-shape-class-uid-3004.md), [S-CLAROTY-ORGPOLICY-001-HS-002-zone-policy-json-columns-raw-extensions.md](S-CLAROTY-ORGPOLICY-001-HS-002-zone-policy-json-columns-raw-extensions.md), [S-CLAROTY-ORGPOLICY-001-HS-003-firewall-group-wire-shape-class-uid-3004.md](S-CLAROTY-ORGPOLICY-001-HS-003-firewall-group-wire-shape-class-uid-3004.md), [S-CLAROTY-ORGPOLICY-001-HS-004-firewall-policy-json-tier2-plan-gate.md](S-CLAROTY-ORGPOLICY-001-HS-004-firewall-policy-json-tier2-plan-gate.md) | Claroty xDome Organization Policy Tables — Single Story (S-CLAROTY-ORGPOLICY-001) — HIDDEN, SINGLE-USE, NO DTU **[CONSUMED 2026-09-01 — D-2409: HS-001 PASS 1.00 (consumed; class_uid=3004 + name Tier-1 REQUIRED + raw_extensions JSON-serialized string per ADR-058 §I2; HS-001 v1.0→v1.1 product-owner raw_extensions wire-encoding correction pre-consumption), HS-002 PASS 1.00 (consumed; zone_policies Json columns communication_conditions/related_alerts_ids/applied_zone_pairs as JSON array values; SELECT communication_conditions E-QUERY-038 confirmed), HS-003 PASS 1.00 (consumed; firewall_groups class_uid=3004 + name Tier-1 REQUIRED + URL/envelope asymmetry verified; raw_extensions JSON-serialized string per ADR-058 §I2; HS-003 v1.0→v1.1 product-owner amendment pre-consumption), HS-004 PASS 1.00 (consumed; applied_group_pairs in raw_extensions as JSON array; SELECT communication_conditions E-QUERY-038; SELECT activity_name accepted). Mean satisfaction 1.00. GATE PASS. Must NOT be reused.]** | 4 | P0 | claroty_organization_zones SELECT *: class_uid=3004 + name Tier-1 REQUIRED (zone_name→entity_management name) + raw_extensions JSON-serialized string (Arrow Utf8; ADR-058 §I2) with zone Tier-2 keys; zone_name NOT standalone (BC-2.16.020 §PC1/3); zone_policies Json columns in raw_extensions: communication_conditions/related_alerts_ids/applied_zone_pairs as JSON array values (not quoted strings); SELECT communication_conditions → E-QUERY-038 (BC-2.16.020 §PC4 + §Invariants); firewall_groups SELECT *: class_uid=3004 + name Tier-1 REQUIRED + URL /api/v1/organization_fw_groups/ vs envelope $.organization_firewall_groups asymmetry verified via non-empty result (BC-2.16.021 §PC1/3); firewall_policies: applied_group_pairs (not applied_zone_pairs) in raw_extensions as JSON array + Tier-2 plan-gate E-QUERY-038 + activity_name (policy_action→activity_name) Tier-1 accepted (BC-2.16.021 §PC3/4 + §Invariants) |
| HS-029 | [S-CLAROTY-ACLPOLICY-001-HS-001-acl-wire-shape-class-uid-3004-metadata-uid.md](S-CLAROTY-ACLPOLICY-001-HS-001-acl-wire-shape-class-uid-3004-metadata-uid.md), [S-CLAROTY-ACLPOLICY-001-HS-002-acl-applied-models-json-in-raw-extensions.md](S-CLAROTY-ACLPOLICY-001-HS-002-acl-applied-models-json-in-raw-extensions.md), [S-CLAROTY-ACLPOLICY-001-HS-003-acl-pagination-none-single-page-no-offset-injection.md](S-CLAROTY-ACLPOLICY-001-HS-003-acl-pagination-none-single-page-no-offset-injection.md) | Claroty xDome Organization ACL Policies Table — Single Story (S-CLAROTY-ACLPOLICY-001) — HIDDEN, SINGLE-USE, NO DTU **[LIVE HOLDOUT GATE (2026-09-01): (1) FAILED HTTP 422 — body_template omitted mandatory filter_by.policy_id selector; FIX-A applied (code @29695e0b9; BC-2.16.022 v1.3 + EC-016-022-011); (2) RE-RUN post-FIX-A SETUP-FAILURE — monroe tenant has ZERO organization ACL policies; API HTTP 404 "No matching policies found" is LEGITIMATE QUIESCENT DATA; FIX-A verified correct (HTTP 422 GONE); binary SHA256 7bb5994f1006fc1ebfe2e10c5d1629e56761f132c2698952d7a46609a57cd2c8 @29695e0b9; structural/plan-gate/column-resolution assertions ALL PASS; HS-001/HS-002/HS-003 SETUP-FAILURE — row-content P0 assertions unexercisable on empty tenant; scenarios PRESERVED SINGLE-USE for re-run when tenant has ≥1 org ACL policy; HUMAN-ACCEPTED 2026-09-01 D-2414]** | 3 | P0 | claroty_organization_acl_policies SELECT *: class_uid=3004 + metadata_uid Tier-1 REQUIRED (policy_id→metadata.uid via ocsf_field_to_arrow_name) + name Tier-1 (policy_name→entity_management name) + raw_extensions with ACL Tier-2 keys; policy_id NOT standalone (BC-2.16.022 §PC1/2); applied_models Json column in raw_extensions: JSON array value (not quoted string); SELECT applied_models → E-QUERY-038 (BC-2.16.022 §PC5 + §Invariants); KEY NOVELTY: PaginationConfig::None single-page fetch — unbounded SELECT succeeds without 422 (no offset/limit injection); no count column in wire output; SELECT policy_id → E-QUERY-038 with metadata_uid in available_columns; SELECT metadata_uid → success (BC-2.16.022 §PC4 + §Invariants EC-016-022-004/007) |
| HS-030 | [S-ENGINE-LIMIT-EARLY-STOP-001-HS-001-partial-final-page-no-false-truncation.md](S-ENGINE-LIMIT-EARLY-STOP-001-HS-001-partial-final-page-no-false-truncation.md), [S-ENGINE-LIMIT-EARLY-STOP-001-HS-002-limit-1-early-stop-end-to-end.md](S-ENGINE-LIMIT-EARLY-STOP-001-HS-002-limit-1-early-stop-end-to-end.md), [S-ENGINE-LIMIT-EARLY-STOP-001-HS-003-count-star-aggregate-plan-gate.md](S-ENGINE-LIMIT-EARLY-STOP-001-HS-003-count-star-aggregate-plan-gate.md) | LIMIT-Aware Early-Stop Pagination — Single Story (S-ENGINE-LIMIT-EARLY-STOP-001) — HIDDEN, SINGLE-USE **[CONSUMED 2026-08-30 — D-2370 holdout gate PASS mean 1.00; all 3 P0 scenarios single-use; must NOT be reused]** | 3 | P0 | HS-001 partial-final-page discriminator: LIMIT=dataset_size on Claroty DTU (10 alerts, page_size=1000) → is_truncated=false (broken discriminator gives is_truncated=true; catches EC-11-094 violation F-P31-LENSA-OBS-001); HS-002 LIMIT 1 end-to-end wiring: SQL LIMIT 1 → FetchContext.early_stop_limit=Some(1) → execute_impl check → DataFusion LIMIT 1 → 1 row returned, is_truncated=false (partial page); HS-003 plan-shape gate: SELECT COUNT(*) on claroty.alerts returns count=10 (full aggregate, no early-stop interference); ADR-060 §D8.2/§D8.3 discriminator + §D8.7 Condition A gate + BC-2.11.001 EC-11-094 |

---

## Full Scenario Registry

### HS-001: Happy Path (P0)

| ID | Title | Repos Tested |
|----|-------|-------------|
| HS-001-01 | Single Sensor Alert Query via MCP | poller-cobra, tally, ocsf-proto-gen, axiathon |
| HS-001-02 | Cyberint Alert Query with Cookie Authentication | poller-express, tally |
| HS-001-03 | Claroty xDome Multi-Source Query | poller-bear, mcp-claroty-xdome |
| HS-001-04 | Armis AQL Query Forwarding | poller-coaster, ocsf-proto-gen |
| ~~HS-001-05~~ | ~~xMP Envelope Backward Compatibility~~ REMOVED (CAP-013 out of scope) | — |
| HS-001-06 | Health Probes and Readiness Tracking | all 4 pollers |

### HS-002: Multi-Sensor (P0)

| ID | Title | Repos Tested |
|----|-------|-------------|
| HS-002-01 | Cross-Sensor Alert Correlation for Single Tenant | all 4 pollers, tally, ocsf-proto-gen, axiathon |
| HS-002-02 | Mixed Data Source Types Across Sensors | poller-bear, poller-coaster |
| HS-002-03 | Concurrent Sensor Polling with Independent Backoff | poller-cobra, poller-express, all pollers |
| HS-002-04 | Batch Sink Delivery Across Multiple Sensors | poller-cobra, poller-bear, poller-coaster |
| HS-002-05 | OCSF Schema Consistency Across Sensors | ocsf-proto-gen, axiathon |

### HS-003: Multi-Tenant (P0)

| ID | Title | Repos Tested |
|----|-------|-------------|
| HS-003-01 | Tenant Data Isolation Under Normal Operation | axiathon, all pollers |
| HS-003-02 | Tenant ID Spoofing Prevention | axiathon, mcp-claroty-xdome |
| HS-003-03 | Cache Isolation Between Tenants | mcp-claroty-xdome, axiathon |
| HS-003-04 | Cursor State Isolation Between Tenants | poller-bear, poller-coaster |
| HS-003-05 | Error Message Tenant Isolation | poller-express, axiathon, tally |
| HS-003-06 | Per-Tenant Rate Limiting Toward Sensor APIs | mcp-claroty-xdome, all pollers |
| HS-003-07 | Log Field Isolation and Filtering | axiathon, tally, all pollers |

### HS-004: Credential Lifecycle (P0)

| ID | Title | Repos Tested |
|----|-------|-------------|
| HS-004-01 | Credential Storage and Retrieval Per Tenant Per Sensor | serveMyAPI, axiathon |
| HS-004-02 | File-Backed Secret Resolution with Env Var Fallback | all 4 pollers |
| HS-004-03 | OAuth2 Token Lifecycle for CrowdStrike | poller-cobra |
| HS-004-04 | Credential Rotation Without Restart | all pollers, serveMyAPI |
| HS-004-05 | Startup Credential Validation (Fail-Fast) | poller-cobra, poller-express |
| HS-004-06 | Credential Audit Trail | serveMyAPI, axiathon, tally |

### HS-005: Failure Scenarios (P0)

| ID | Title | Repos Tested |
|----|-------|-------------|
| HS-005-01 | Sensor API Unreachable (Connection Refused) | all pollers |
| HS-005-02 | Authentication Expired Mid-Session | poller-cobra |
| HS-005-03 | Sensor API Rate Limited (HTTP 429) | mcp-claroty-xdome, all pollers |
| HS-005-04 | Request Timeout | poller-bear, poller-cobra, mcp-claroty-xdome |
| HS-005-05 | Partial Batch Failure in Sink Delivery | poller-cobra, poller-bear, poller-coaster |
| HS-005-06 | Malformed Sensor API Response | poller-express, poller-bear, axiathon |
| HS-005-07 | MCP Client Disconnection During Long Query | tally, mcp-claroty-xdome |

### HS-006: State Recovery (P0)

| ID | Title | Repos Tested |
|----|-------|-------------|
| HS-006-01 | Clean Restart with Cursor Resume | poller-bear, poller-coaster |
| HS-006-02 | Crash Recovery with Atomic State Files | poller-bear, poller-coaster, poller-cobra |
| HS-006-03 | Config Change Detection via Query Fingerprint | all 4 pollers |
| HS-006-04 | Forward Progress Invariant Prevents Cursor Regression | poller-cobra, poller-coaster, poller-bear |
| HS-006-05 | Batch Receipt Audit Trail Survives Restart | poller-bear, poller-coaster |
| HS-006-06 | Multi-Tenant State Recovery After System-Wide Restart | all pollers |

### HS-007: Cross-Repo Failure (P1)

| ID | Title | Repos Tested |
|----|-------|-------------|
| HS-007-01 | MemoryStore Pattern Leaks Into Production Code | poller-cobra, poller-express |
| HS-007-02 | N-Way Collector Duplication Eliminated by Generic Trait | poller-bear, poller-coaster, poller-express |
| HS-007-03 | Cobra's State-Before-Persistence Bug Cannot Recur | poller-cobra |
| HS-007-04 | Express's Strict JSON Decoding Replaced with Lenient Parsing | poller-express, ocsf-proto-gen |
| HS-007-05 | Bear's Polymorphic JSON IDs Handled in Typed Rust Context | poller-bear, poller-express |
| HS-007-06 | ServeMyAPI's Path Traversal Prevented in Credential Store | serveMyAPI |
| HS-007-07 | Tally's Error Code Mapping Unified Across All Tools | tally, mcp-claroty-xdome |
| HS-007-08 | Axiathon's Unbounded Caches Bounded in Prism | mcp-claroty-xdome, poller-express, poller-coaster, axiathon |

### HS-008: Contract Violation (P1)

| ID | Title | Repos Tested |
|----|-------|-------------|
| HS-008-01 | OCSF Schema Version Mismatch | ocsf-proto-gen, axiathon |
| HS-008-02 | Proto Field Number Instability Across OCSF Versions | ocsf-proto-gen |
| HS-008-03 | Protobuf Version Conflict Between Dependencies | axiathon, ocsf-proto-gen |
| HS-008-04 | xMP Envelope Format Drift | all 4 pollers |
| HS-008-05 | Sensor API Contract Change (Breaking) | poller-bear, mcp-claroty-xdome |
| HS-008-06 | CrowdStrike Two-Step Fetch Contract Violation | poller-cobra |
| HS-008-07 | Armis AQL Query Syntax Rejected | poller-coaster |
| HS-008-08 | MCP Protocol Version Mismatch | tally, mcp-claroty-xdome |

### HS-009: Scheduler Operations (P0)

| ID | Title | Repos Tested |
|----|-------|-------------|
| HS-009-01 | Schedule CRUD Lifecycle with Org-Prefixed CF Keys | prism-operations, prism-storage |
| HS-009-02 | 60-Second Default Tick Fires Correctly per ADR-013 §2.1 | prism-operations, prism-storage |
| HS-009-03 | ScheduleFireMissed Audit Event Emitted on Tick Overrun | prism-operations, prism-audit |
| HS-009-04 | Schedule Pack Rotation Invalidates Pending Fires for Retired Packs | prism-operations, prism-storage |
| HS-009-05 | Multi-Tenant Schedule Isolation | prism-operations, prism-storage |
| HS-009-06 | Schedule Disable/Re-Enable Preserves schedule_id and Pauses Tick Fires | prism-operations, prism-storage |

### HS-010: Detection & Alert Pipeline (P0)

| ID | Title | Repos Tested |
|----|-------|-------------|
| HS-010-01 | Detection Rule Registration and Matching Against Query Result Packs | prism-operations, prism-storage |
| HS-010-02 | Diff Pack Flow — Detect Changes Between Consecutive Query Result Packs | prism-operations, prism-storage |
| HS-010-03 | Alert Generation with alert_id UUID v7 as Idempotency Key — Replay-Safe | prism-operations, prism-storage |
| HS-010-04 | Detection Rule Evaluation Under Multi-Tenant Isolation | prism-operations, prism-storage |
| HS-010-05 | Alert Deduplication via Idempotency Key (Same alert_id = No Duplicate Emission) | prism-operations, prism-storage |
| HS-010-06 | Detection Rule Disable Retains Existing Alerts but Stops New Generation | prism-operations, prism-storage |

### HS-011: Case Management (P0)

| ID | Title | Repos Tested |
|----|-------|-------------|
| HS-011-01 | Case Lifecycle — Open, Update, and Close with State Machine Enforcement | prism-operations, prism-storage |
| HS-011-02 | Timeline Entry Idempotency via timeline_entry_id UUID v7 | prism-operations, prism-storage |
| HS-011-03 | Case Metrics Aggregation — Open Count, MTTR, Severity Distribution per Org | prism-operations, prism-storage |
| HS-011-04 | Multi-Tenant Case Isolation — case_dedup_idx CF Org-Prefixed | prism-operations, prism-storage |
| HS-011-05 | Case Linking to Alerts — One Case References Multiple alert_ids | prism-operations, prism-storage |

### HS-012: Action Delivery (P0)

| ID | Title | Repos Tested |
|----|-------|-------------|
| HS-012-01 | Action Delivery Uses Independent 8-Permit Semaphore — NOT Shared with Scheduler (D-209) | prism-operations |
| HS-012-02 | VP-045 try_acquire Non-Blocking — Tick Aborts Within 10ms if Permit Unavailable | prism-operations |
| HS-012-03 | SemaphoreExhausted Audit Event Emitted When Action Delivery Semaphore Saturated | prism-operations, prism-audit |
| HS-012-04 | action_state CF Discriminator Transitions — Success and Failure Paths per ADR-016 §2.5 | prism-operations, prism-storage |
| HS-012-05 | DELIVERY_TERMINAL State — No Further Transitions | prism-operations, prism-storage |
| HS-012-06 | Multi-Tenant Action Delivery Isolation — Org A's Actions Invisible to Org B | prism-operations, prism-storage |

### HS-013: CrowdStrike DTU Parity (P0) — PLUGIN-MIGRATION-001-D

| ID | Title | Crates Tested |
|----|-------|--------------|
| HS-013-01 | CrowdStrike Detections Happy Path — 3 Detection Records, >=2 HTTP Calls (DTU routes: /detects/queries/detects/v1 + /detects/entities/summaries/GET/v1) | prism-spec-engine, prism-dtu-crowdstrike |
| HS-013-02 | CrowdStrike Batch Cap at CROWDSTRIKE_BATCH_SIZE (100 IDs, 1 PostEntities Batch; DTU route: POST /detects/entities/summaries/GET/v1) | prism-spec-engine, prism-dtu-crowdstrike |

### HS-014: Claroty POST-for-Read Parity (P0) — PLUGIN-MIGRATION-001-D

| ID | Title | Crates Tested |
|----|-------|--------------|
| HS-014-01 | Claroty Alerts POST-for-Read — Integer ID Polymorphic Normalization (bearer_static auth; POST /api/v1/alerts) | prism-spec-engine, prism-dtu-claroty |
| HS-014-02 | Claroty Alerts POST-for-Read — UUID String ID Polymorphic Normalization (bearer_static auth; POST /api/v1/alerts) | prism-spec-engine, prism-dtu-claroty |

### HS-015: Cyberint Alerts Cursor Parity (P0) — PLUGIN-MIGRATION-001-D

| ID | Title | Crates Tested |
|----|-------|--------------|
| HS-015-01 | Cyberint Alerts Happy Path — ISO-8601 Timestamps, Cursor Pagination | prism-spec-engine, prism-dtu-cyberint |
| HS-015-02 | Cyberint Alerts — Multi-Format Timestamp Edge Cases (RFC3339, no-TZ, microseconds, null) | prism-spec-engine, prism-dtu-cyberint |
| HS-015-03 | Cyberint Incidents Table — SKIP Verdict per TS-PLUGIN-PARITY-001 Cyberint DTU Gap Note | prism-spec-engine, prism-dtu-cyberint |

### HS-016: Armis AQL + Timestamp Fallback Parity (P0) — PLUGIN-MIGRATION-001-D

| ID | Title | Crates Tested |
|----|-------|--------------|
| HS-016-01 | Armis Devices — AQL Expression Forwarding via ${query.filter.aql} Verified at DTU | prism-spec-engine, prism-dtu-armis |
| HS-016-02 | Armis Devices — Timestamp Resolved from firstSeen (No Fallback) | prism-spec-engine, prism-dtu-armis |
| HS-016-03 | Armis Devices — Timestamp Fallback to now() with tracing::warn! Audit Signal | prism-spec-engine, prism-dtu-armis |

### HS-017: Bundled Spec Validation CI Gate (P0) — PLUGIN-MIGRATION-001-D

| ID | Title | Crates Tested |
|----|-------|--------------|
| HS-017-01 | Malformed Spec — Invalid Column Type Rejected with E-SPEC-002 | prism-spec-engine |
| HS-017-02 | Malformed Spec — Undefined Variable Reference Rejected with E-SPEC-003 | prism-spec-engine |

### HS-018: Spec sensor_id / Filename Mismatch Rejection (P0) — PLUGIN-MIGRATION-001-D

| ID | Title | Crates Tested |
|----|-------|--------------|
| HS-018-01 | Filename Stem Mismatch (crowdstrike.sensor.toml + sensor_id: "falcon") — E-SPEC-017 | prism-spec-engine |
| HS-018-02 | Case Mismatch (crowdstrike.sensor.toml + sensor_id: "CrowdStrike") — E-SPEC-017 | prism-spec-engine |
| HS-018-03 | Valid Convention (crowdstrike.sensor.toml + sensor_id: "crowdstrike") — Loads OK | prism-spec-engine |

### HS-019: Claroty audit_logs Push-Down Fix — Single Story (P0) — DRIFT-CLAROTY-AUDITLOG-TIMEOUT-001 **[CONSUMED 2026-08-15 — D-2195]**

Story-level holdout gate for S-CLAROTY-AUDITLOG-TIMEBOX-001 (consolidated single push-down story). HIDDEN from test-writer and implementer. SINGLE-USE. Design reworked from two-layer to one story; all 4 scenarios under this group. **All 4 scenarios CONSUMED 2026-08-15 (D-2195 holdout gate PASS; must NOT be reused).**

| ID | Title | Crates Tested |
|----|-------|--------------|
| HS-AUDITLOG-001-A-001 | COUNT(*) on claroty.audit_logs completes in < 5s with no E-QUERY-004; ASM-CLAROTY-AUDITLOG-001 validation: `after_seconds_ago`-invalid or DTU HTTP 400 → SETUP-FAILURE (not behavioral FAIL) | prism-spec-engine, prism-dtu-claroty |
| HS-AUDITLOG-001-A-002 | SELECT * FROM claroty.audit_logs without time filter returns only records from last 7 days (old group excluded); confirms bounded default scope via push-down `greater_or_equal now−604800s` | prism-spec-engine, prism-dtu-claroty |
| HS-AUDITLOG-001-A-003 | WHERE timestamp > 45d ago returns records from middle group (8–44d) AND recent group (0–7d) — explicit filter honored; no silent truncation to 7-day default | prism-spec-engine, prism-dtu-claroty, prism-sensors |
| HS-AUDITLOG-001-A-004 | WHERE timestamp BETWEEN a AND b pushes BOTH bounds — result scoped to [a, b]; upper bound (`less_or_equal`) and lower bound (`greater_or_equal`) both injected via compound `and` filter | prism-spec-engine, prism-dtu-claroty, prism-sensors |

### HS-021: Column Coercion Gap Closure (P0) — S-ADR058-OCSF-COERCION-001

Story-level holdout gate for S-ADR058-OCSF-COERCION-001 (ADR-058 Stage 1 coercion fixes). HIDDEN from test-writer and implementer. SINGLE-USE.

| ID | Title | Crates Tested |
|----|-------|--------------|
| HS-COERCION-001-A-001 | String column `description` (claroty.alerts) receives JSON Object — null cell present in wire output (not stringified); `column_coercion_failure` warn with `column_type = "string"` and `actual_json_kind = "object"` emitted (AC-005 Path A fix) | prism-bin, prism-spec-engine |
| HS-COERCION-001-A-002 | Integer column `devices_count` (claroty.alerts) receives JSON string `"42"` — wire output is integer 42 (not null); no `column_coercion_failure` event emitted (AC-007 happy path: parse success) | prism-bin, prism-spec-engine |
| HS-COERCION-001-A-003 | Integer column `devices_count` (claroty.alerts) receives non-parseable JSON string — null key present in wire output; `column_coercion_failure` warn with `column_type = "integer"` and `actual_json_kind = "string"` emitted; record not dropped (AC-007 failure path) | prism-bin, prism-spec-engine |
| HS-COERCION-001-A-004 | String column `ip_list` (claroty.devices) receives JSON Array — value serializes as compact JSON-list string (not null); no `column_coercion_failure` event; ENRICH-1 Array arm preserved after AC-005 Object arm insertion (EC-016-013-026 non-regression) | prism-bin, prism-spec-engine, prism-dtu-claroty |

### HS-022: OCSF Field-Name Routing (P0) — S-ADR058-OCSF-ROUTING-001 **[CONSUMED — D-2270; A+W amendment post-dates HS-022; superseded by HS-023]**

Story-level holdout gate for S-ADR058-OCSF-ROUTING-001 (ADR-058 Stage 2 — ocsf_column_naming flag, underscore-flattened Arrow names, Claroty activation). HIDDEN from test-writer and implementer. SINGLE-USE. **All 4 scenarios CONSUMED (D-2270; A+W amendment human-decision 2026-08-23 post-dates these scenarios; re-gate required; must NOT be reused). Fresh re-gate scenarios registered as HS-023.**

| ID | Title | Crates Tested |
|----|-------|--------------|
| HS-ROUTING-001-A-001 | Claroty alerts OCSF field Arrow name wire shape — `finding_info_uid` column present in RecordBatch (not `id`); `ocsf_column_naming = true` activates flattened names per ADR-058 §B2 / §I2 | prism-bin, prism-spec-engine, prism-dtu-claroty |
| HS-ROUTING-001-A-002 | `prism_describe` Tier-2 prohibition — no individual ColumnDescriptor emitted for `ocsf_field == None` columns; exactly one `raw_extensions` ColumnDescriptor present with source-key enumeration (BC-2.16.003 EC-016-013-027 / ADR-058 §G) | prism-mcp, prism-spec-engine, prism-dtu-claroty |
| HS-ROUTING-001-A-003 | Claroty audit_logs class_uid 3004 wire shape — Arrow column `class_uid` value is 3004 (entity_management, not 3001 account_change); `ocsf.unknown_class_name` WARN absent on happy path (AC-009 / BC-2.16.003 EC-016-013-023) | prism-bin, prism-ocsf, prism-dtu-claroty |
| HS-ROUTING-001-A-004 | Claroty devices device_type_label filter — `WHERE device_type_label = 'PLC'` returns rows where KF-06 correction applies; `device_type_label` Arrow field present in schema (not `device_type`); AC-005 / AC-010 assertion 6 | prism-bin, prism-spec-engine, prism-dtu-claroty |

### HS-023: OCSF Field-Name Routing A+W Re-Gate (P0) — S-ADR058-OCSF-ROUTING-001

Story-level holdout re-gate for S-ADR058-OCSF-ROUTING-001 covering the A+W amendment (human decision 2026-08-23: zero-Tier-1 table warning + raw_extensions available-set) and new surfaces not covered by consumed HS-022. HIDDEN from test-writer and implementer. SINGLE-USE. Supersedes HS-022 for the re-gate surface.

| ID | Title | Crates Tested |
|----|-------|--------------|
| HS-ROUTING-001-B-001 | Zero-Tier-1 OCSF table: synthetic sensor with `ocsf_column_naming = true` + 2 Tier-2-only columns → `ocsf.zero_tier1_table` WARN emitted ONCE at registration (not per-query); `raw_extensions` absent from E-QUERY-038; Tier-2 raw col.name rejected with available_columns = {raw_extensions, class_uid, _sensor} (BC-2.11.016 EC-11-080 A+W sub-case; BC-2.16.002 catalog row; ADR-058 §J6) | prism-bin, prism-spec-engine |
| HS-ROUTING-001-B-002 | §J4 intra-table collision: synthetic sensor TOML with two columns whose `ocsf_field` values both flatten to the same Arrow name (`src.ip` + `src_ip` → `src_ip`) → spec REJECTED at load with E-SPEC-030 [§J4] in stderr; boot exits non-zero (exit 2 ConfigInvalid); no MCP I/O (ADR-058 §J7 Validation Rule 8) | prism-bin, prism-spec-engine |
| HS-ROUTING-001-B-003 | Claroty audit_logs OQ-005 fix — `metadata_uid` (ocsf_field `metadata.uid` → Arrow `metadata_uid`) present in wire RecordBatch (not `id`); SELECT `id` triggers E-QUERY-038 with `available_columns` containing "metadata_uid" (not "id"); BC-2.11.016 EC-11-079 sub-cases (a)+(b) | prism-bin, prism-spec-engine, prism-dtu-claroty |

### HS-024: Claroty xDome Vulnerabilities Table (P0) — S-CLAROTY-VULNS-001

Story-level holdout gate for S-CLAROTY-VULNS-001 (Wave A G1 — claroty_vulnerabilities table, OCSF vulnerability_finding/2002). HIDDEN from test-writer and implementer. SINGLE-USE. Live monroe sensor — DTU exists (SAP-2 applicable).

| ID | Title | Crates Tested |
|----|-------|--------------|
| HS-VULNS-001-001 | claroty_vulnerabilities SELECT *: class_uid=2002 in wire output; finding_info_title Tier-1 REQUIRED column present and non-null string (BC-2.16.015 §PC1/2 Tier-1) | prism-bin, prism-spec-engine, claroty-live |
| HS-VULNS-001-002 | SELECT finding_info_title succeeds (no E-QUERY-038); SELECT name (raw col.name) raises E-QUERY-038 with finding_info_title in available_columns (BC-2.16.015 §PC2 Tier-1 plan-gate + BC-2.11.016) | prism-bin, prism-spec-engine, claroty-live |
| HS-VULNS-001-003 | SELECT raw_extensions returns non-empty JSON object with cve_ids + severity_score keys (BC-2.16.015 §PC2 Tier-2 aggregation + ADR-058 Tier-2 path) | prism-bin, prism-spec-engine, claroty-live |

### HS-025: Claroty xDome OT Activity Events Table (P0) — S-CLAROTY-OT-EVENTS-001

Story-level holdout gate for S-CLAROTY-OT-EVENTS-001 (Wave A G2 — claroty_ot_activity_events table, OCSF detection_finding/2004). HIDDEN from test-writer and implementer. SINGLE-USE. Live monroe sensor — NO DTU (SAP-2 N/A per D-2200 deferred DTU).

**EVALUATION RESULT (D-2399, 2026-08-31):** HS-002 PASS 1.00 (CONSUMED). HS-003 PASS 1.00 (CONSUMED). HS-001 SETUP-FAILURE — quiescent OT network, 0 events returned via force_refresh; NOT a behavioral FAIL; scenario §Edge Conditions applies. Structural corroboration: table registered, class_uid non-null-int mapped to detection_finding, finding_info_uid integer Tier-1 column present. Human ACCEPTED (Option-1): gate treated as PASSED on 2/3-at-ceiling + structural corroboration. HS-001 kept unconsumed for future re-run when monroe OT network has ≥1 event.

| ID | Title | Crates Tested | Result |
|----|-------|--------------|--------|
| HS-OTEVTS-001-001 | claroty_ot_activity_events SELECT *: class_uid=2004 (Option B) in wire output; finding_info_uid REQUIRED Tier-1 column present as non-null integer (BC-2.16.016 §PC1/2/3) | prism-bin, prism-spec-engine, claroty-live | **SETUP-FAILURE 2026-08-31** (quiescent OT network — 0 events; NOT consumed; re-run pending) |
| HS-OTEVTS-001-002 | SELECT source_ip raises E-QUERY-038 with raw_extensions in available_columns but NOT source_ip; SELECT raw_extensions succeeds (no E-QUERY-038) — Tier-2 network 5-tuple plan-gate rejection (BC-2.16.016 §Invariants + EC-016-016-006) | prism-bin, prism-spec-engine, claroty-live | **PASS 1.00 CONSUMED 2026-08-31** |
| HS-OTEVTS-001-003 | SELECT time (OCSF Arrow name) succeeds (no E-QUERY-038); SELECT detection_time (raw col.name) raises E-QUERY-038 with time in available_columns (BC-2.16.016 §PC2 Tier-1 datetime rename) | prism-bin, prism-spec-engine, claroty-live | **PASS 1.00 CONSUMED 2026-08-31** |

### HS-026: Claroty xDome Device-Vulnerability Relations Table (P0) — S-CLAROTY-DEVVULNREL-001

Story-level holdout gate for S-CLAROTY-DEVVULNREL-001 (Wave B G3 — claroty_device_vulnerability_relations table, OCSF vulnerability_finding/2002, 214-field join table, first cut 13 columns). HIDDEN from test-writer and implementer. SINGLE-USE. Live monroe sensor — NO DTU (SAP-2 N/A per D-2200 deferred DTU).

**EVALUATION RESULT (D-2403, 2026-08-31):** HS-001 PASS 1.00 (CONSUMED — class_uid=2002 confirmed, finding_info_title CVE string non-null). HS-003 PASS 1.00 (CONSUMED — SELECT time accepted with data; SELECT device_vulnerability_detection_date E-QUERY-038 with time in available_columns). HS-002 PASS 0.75 at-threshold (CONSUMED — VERDICT-A: over-specified scenario; implementation CORRECT; vulnerability_name is Tier-1 mapped to finding_info_title per ADR-058 §B2, NOT in raw_extensions; HS-002 v1.0 dimension-2 "vulnerability_name in raw_extensions" was wrong; product-owner amended to v1.1 pre-consumption [finding_info_title Tier-1 access + device_uid in raw_extensions]; satisfaction 0.75 = three of four dimensions at 1.00, one dimension re-scored 0.00 [old v1.0 dimension for vulnerability_name in raw_extensions] was adjudicated as incorrect scenario, not implementation gap). Mean gate satisfaction 0.92. GATE PASS. All 3 scenarios single-use; must NOT be reused.

| ID | Title | Crates Tested | Result |
|----|-------|--------------|--------|
| HS-DEVVULNREL-001-001 | claroty_device_vulnerability_relations SELECT *: class_uid=2002 in wire output; finding_info_title Tier-1 REQUIRED column (vulnerability_name→finding_info.title) present and non-null string (BC-2.16.017 §PC1/2) | prism-bin, prism-spec-engine, claroty-live | **PASS 1.00 CONSUMED 2026-08-31** |
| HS-DEVVULNREL-001-002 | vulnerability_name accessible as Tier-1 OCSF column finding_info_title (NOT in raw_extensions — ADR-058 §B2; tiers mutually exclusive); device_uid in raw_extensions (Tier-2); SELECT device_uid raises E-QUERY-038 (BC-2.16.017 §PC2 Tier-1+Tier-2 + §PC3 composite PK + §Invariants). HS-002 v1.1 (v1.0 dimension "vulnerability_name in raw_extensions" was over-specified; corrected by product-owner amendment VERDICT-A) | prism-bin, prism-spec-engine, claroty-live | **PASS 0.75 CONSUMED 2026-08-31 (VERDICT-A)** |
| HS-DEVVULNREL-001-003 | SELECT time (OCSF Arrow name for device_vulnerability_detection_date) succeeds (no E-QUERY-038); SELECT device_vulnerability_detection_date (raw col.name) raises E-QUERY-038 with time in available_columns (BC-2.16.017 §PC2 Tier-1 datetime rename + §Invariants) | prism-bin, prism-spec-engine, claroty-live | **PASS 1.00 CONSUMED 2026-08-31** |

### HS-027: Claroty xDome Servers + Server Interfaces Tables (P0) — S-CLAROTY-SERVERS-001

Story-level holdout gate for S-CLAROTY-SERVERS-001 (Wave C G4 — claroty_servers and claroty_server_interfaces tables, OCSF inventory_info/5001, separate endpoints, first Tier-1 device_name mapping across both tables). HIDDEN from test-writer and implementer. SINGLE-USE. Live monroe sensor — NO DTU (SAP-2 N/A per D-2200 deferred DTU).

| ID | Title | Crates Tested | Result |
|----|-------|--------------|--------|
| HS-SERVERS-001-001 | claroty_servers SELECT *: class_uid=5001 in wire output; device_name Tier-1 REQUIRED column (server_name→device.name via ocsf_field_to_arrow_name) present as non-null string; raw_extensions JSON-serialized string (Arrow Utf8; ADR-058 §I2) whose parsed object contains at least one server inventory Tier-2 key (BC-2.16.018 §Postconditions 1 and 2) | prism-bin, prism-spec-engine, claroty-live | **PASS 1.00 CONSUMED 2026-09-01** |
| HS-SERVERS-001-002 | claroty_servers Tier-1 rename enforcement: SELECT server_name (raw col.name) raises E-QUERY-038 with device_name in available_columns; SELECT device_name (Arrow field name) succeeds with non-null value; SELECT status_code (second Tier-1 from server_status) succeeds with non-null value (BC-2.16.018 §Postconditions 2 Tier-1 plan-gate + §Invariants) | prism-bin, prism-spec-engine, claroty-live | **PASS 1.00 CONSUMED 2026-09-01** |
| HS-SERVERS-001-003 | claroty_server_interfaces: separate POST /api/v1/server_interfaces/ endpoint independently queryable (not routed via servers endpoint); class_uid=5001 in wire output; SELECT interface_status (raw col.name) raises E-QUERY-038 with status_code in available_columns; SELECT raw_extensions returns JSON-serialized string (Arrow Utf8; ADR-058 §I2) whose parsed object contains at least one interface Tier-2 key (BC-2.16.019 §Postconditions 1/2/3) | prism-bin, prism-spec-engine, claroty-live | **PASS 1.00 CONSUMED 2026-09-01** |

### HS-028: Claroty xDome Organization Policy Tables (P0) — S-CLAROTY-ORGPOLICY-001

Story-level holdout gate for S-CLAROTY-ORGPOLICY-001 (Wave C G5 — 4 organization policy tables across 2 domain pairs: claroty_organization_zones + claroty_organization_zone_policies [BC-2.16.020] and claroty_organization_firewall_groups + claroty_organization_firewall_policies [BC-2.16.021]; OCSF entity_management/3004; offset_limit/1000 pagination; 8 Json columns across 4 tables; critical URL vs envelope key asymmetry for firewall group endpoint). HIDDEN from test-writer and implementer. SINGLE-USE. Live monroe sensor — NO DTU (SAP-2 N/A per D-2200 deferred DTU).

**EVALUATION RESULT (D-2409, 2026-09-01):** HS-001 PASS 1.00 (CONSUMED — class_uid=3004 confirmed, name Tier-1 REQUIRED non-null string present, zone_name NOT standalone, raw_extensions JSON-serialized string per ADR-058 §I2 with zone Tier-2 keys; HS-001 v1.0→v1.1 product-owner amendment corrected raw_extensions encoding expectation pre-consumption). HS-002 PASS 1.00 (CONSUMED — zone_policies communication_conditions/related_alerts_ids/applied_zone_pairs present as JSON array values in raw_extensions, not quoted strings; SELECT communication_conditions E-QUERY-038 confirmed). HS-003 PASS 1.00 (CONSUMED — firewall_groups non-empty result; class_uid=3004; name Tier-1 REQUIRED non-null; firewall_group_name NOT standalone; URL/envelope asymmetry verified; raw_extensions JSON-serialized string per ADR-058 §I2; HS-003 v1.0→v1.1 product-owner amendment corrected raw_extensions encoding expectation pre-consumption). HS-004 PASS 1.00 (CONSUMED — applied_group_pairs as JSON array in raw_extensions; SELECT communication_conditions E-QUERY-038 confirmed; SELECT activity_name accepted with Allow/Deny values). Mean satisfaction 1.00 all 4 must_pass. GATE PASS. OBSERVATION adjudicated VERDICT-A per D-2406 (same architect ruling): raw_extensions is Arrow Utf8 column emitted as JSON-serialized STRING per ADR-058 §I2 — INTENDED, consistent across all merged G1–G4; D-2381 native-JSON rule governs only values INSIDE the raw_extensions container. All 4 HS-028 scenarios CONSUMED (lifecycle_status: consumed; used: true; last_evaluated: 2026-09-01; last_eval_satisfaction: 1.00). HS-002 input-hash corrected f660fcc→a91185c (BC-2.16.020 update drift; same inputs as HS-001). HS-004 input-hash corrected 967aa52→f0cbf03 (BC-2.16.021 update drift; same inputs as HS-003). All 4 scenarios single-use; must NOT be reused.

| ID | Title | Crates Tested | Result |
|----|-------|--------------|--------|
| HS-ORGPOL-001-001 | claroty_organization_zones SELECT *: class_uid=3004 in wire output; name Tier-1 REQUIRED column (zone_name→entity_management name via ocsf_field_to_arrow_name) present as non-null string; zone_name NOT a standalone Arrow column; raw_extensions JSON-serialized string (Arrow Utf8; ADR-058 §I2) with at least one zone Tier-2 key (BC-2.16.020 §Postconditions 1 and 3) | prism-bin, prism-spec-engine, claroty-live | **PASS 1.00 CONSUMED 2026-09-01** |
| HS-ORGPOL-001-002 | claroty_organization_zone_policies: raw_extensions contains communication_conditions, related_alerts_ids, and applied_zone_pairs as JSON array values (not quoted strings — confirms column_type="Json" not "String"); SELECT communication_conditions (raw col.name) raises E-QUERY-038 confirming Tier-2 plan-gate active (BC-2.16.020 §Postconditions 4 + §Invariants) | prism-bin, prism-spec-engine, claroty-live | **PASS 1.00 CONSUMED 2026-09-01** |
| HS-ORGPOL-001-003 | claroty_organization_firewall_groups SELECT *: non-empty result set (URL /api/v1/organization_fw_groups/ with response_path $.organization_firewall_groups — URL vs envelope key asymmetry verified; empty result = SUSPICIOUS-FAIL for possible response_path bug); class_uid=3004; name Tier-1 REQUIRED (firewall_group_name→entity_management name) present non-null; firewall_group_name NOT standalone; raw_extensions JSON-serialized string (Arrow Utf8; ADR-058 §I2) (BC-2.16.021 §Postconditions 1 and 3) | prism-bin, prism-spec-engine, claroty-live | **PASS 1.00 CONSUMED 2026-09-01** |
| HS-ORGPOL-001-004 | claroty_organization_firewall_policies: applied_group_pairs (not applied_zone_pairs — firewall-specific Json column) in raw_extensions as JSON array; SELECT communication_conditions raises E-QUERY-038 (Tier-2 plan-gate); SELECT activity_name (policy_action→activity_name Tier-1 mapping) returns non-error result with Allow/Deny/null values (BC-2.16.021 §Postconditions 3 and 4 + §Invariants) | prism-bin, prism-spec-engine, claroty-live | **PASS 1.00 CONSUMED 2026-09-01** |

### HS-029: Claroty xDome Organization ACL Policies Table (P0) — S-CLAROTY-ACLPOLICY-001

Story-level holdout gate for S-CLAROTY-ACLPOLICY-001 (Wave C G6 — claroty_organization_acl_policies table; OCSF entity_management/3004; KEY NOVELTY: PaginationConfig::None — non-paginated single-page POST with mandatory `policy_acl_syntax` request body field; 4 Tier-1 columns including metadata_uid from policy_id via metadata.uid OCSF field; 7 Tier-2 columns including applied_models Json array column). HIDDEN from test-writer and implementer. SINGLE-USE. Live monroe sensor — NO DTU (SAP-2 N/A per D-2200 deferred DTU). **LIVE HOLDOUT GATE (2026-09-01): (1) FAILED HTTP 422 — FIX-A applied (code @29695e0b9; BC-2.16.022 v1.3 + EC-016-022-011). (2) RE-RUN post-FIX-A SETUP-FAILURE — monroe tenant has ZERO organization ACL policies (confirmed via Claroty xDome UI: Network > Enforcement > ACL Policies > Organization ACL Policies tab showed "0 Organization Policies"); API HTTP 404 "No matching policies found" is LEGITIMATE QUIESCENT DATA; FIX-A verified correct (HTTP 422 GONE across all query shapes); binary SHA256 7bb5994f1006fc1ebfe2e10c5d1629e56761f132c2698952d7a46609a57cd2c8 @29695e0b9; structural/plan-gate/column-resolution assertions ALL PASS (class_uid=3004 recognized; OCSF Tier-1 renames present — metadata_uid, name, actor_user_name, comment; SELECT policy_id → E-QUERY-038 with metadata_uid in available_columns; SELECT applied_models → E-QUERY-038 Tier-2 gate enforced; no offset/limit injection → no 422; single round-trip; no standalone count column); HS-001/HS-002/HS-003 SETUP-FAILURE (not consumed) — row-content P0 assertions unexercisable on empty tenant; scenarios PRESERVED SINGLE-USE for re-run when tenant has ≥1 org ACL policy. HUMAN-ACCEPTED 2026-09-01 D-2414.**

| ID | Title | Crates Tested | Result (2026-09-01) |
|----|-------|--------------|---------------------|
| HS-ACLPOLICY-001-001 | claroty_organization_acl_policies SELECT *: class_uid=3004 in wire output; metadata_uid Tier-1 REQUIRED column (policy_id→metadata.uid via ocsf_field_to_arrow_name) present as non-null string; name Tier-1 (policy_name→entity_management name) present; raw_extensions JSON object with at least one ACL Tier-2 key (HS-001 v1.0→v1.1: raw_extensions is JSON-serialized STRING per ADR-058 §I2, consistent G1–G5 VERDICT-A — corrected pre-run); policy_id NOT a standalone Arrow column (BC-2.16.022 §Postconditions 1 and 2) | prism-bin, prism-spec-engine, claroty-live | **SETUP-FAILURE — zero org ACL policies in monroe tenant; API HTTP 404 "No matching policies found" legitimate quiescent data; FIX-A HTTP 422 GONE; NOT consumed; PRESERVED single-use for re-run when tenant ≥1 org ACL policy (D-2414 HUMAN-ACCEPTED)** |
| HS-ACLPOLICY-001-002 | claroty_organization_acl_policies applied_models Json column in raw_extensions: value is a JSON array (either [] or ["model_a", ...]) NOT a quoted string; SELECT applied_models raises E-QUERY-038 confirming Tier-2 plan-gate active; policy_acl String Tier-2 column also present as string key in raw_extensions (BC-2.16.022 §Postconditions 5 + §Invariants) | prism-bin, prism-spec-engine, claroty-live | **SETUP-FAILURE — zero org ACL policies in monroe tenant; HTTP 404 legitimate quiescent data; NOT consumed; preserved single-use (D-2414)** |
| HS-ACLPOLICY-001-003 | KEY NOVELTY: PaginationConfig::None contract — unbounded SELECT * succeeds without 422 (no offset/limit injection in HTTP POST body); no count column in wire schema; SELECT policy_id (raw TOML col.name) raises E-QUERY-038 with metadata_uid listed in available_columns; SELECT metadata_uid (Arrow field name from ocsf_field_to_arrow_name) returns non-error result (BC-2.16.022 §Postconditions 4 + §Invariants EC-016-022-004/007) | prism-bin, prism-spec-engine, claroty-live | **SETUP-FAILURE — zero org ACL policies in monroe tenant; HTTP 404 legitimate quiescent data; NOT consumed; preserved single-use (D-2414)** |

### HS-030: LIMIT-Aware Early-Stop Pagination (P0) — S-ENGINE-LIMIT-EARLY-STOP-001 **[CONSUMED 2026-08-30 — D-2370]**

Story-level holdout gate for S-ENGINE-LIMIT-EARLY-STOP-001 (Wave A xDome Expansion — LIMIT-Aware Early-Stop Pagination; ADR-060 §D8; adds `FetchContext.early_stop_limit: Option<usize>` and partial-final-page discriminator). HIDDEN from test-writer and implementer. SINGLE-USE. All scenarios use Claroty DTU (prism-dtu-claroty alerts fixture: 10 records, page_size=1000 — all pages are PARTIAL because fixture count < page_size). **All 3 scenarios CONSUMED 2026-08-30 (D-2370 holdout gate PASS mean 1.00; must NOT be reused).**

| ID | Title | Crates Tested |
|----|-------|--------------|
| HS-EARLY-STOP-001-001 | Partial-final-page discriminator: SQL `LIMIT 10` on claroty.alerts (DTU has exactly 10 records, page_size=1000 → PARTIAL page, early_stopped=false) → 10 rows returned, `is_truncated: false`; broken discriminator (unconditional `early_stopped=true`) would yield `is_truncated: true` — catches EC-11-094 violation from F-P31-LENSA-OBS-001 (ADR-060 §D8.2/§D8.3; BC-2.11.001 EC-11-094) | prism-bin, prism-spec-engine, prism-sensors, prism-dtu-claroty |
| HS-EARLY-STOP-001-002 | LIMIT 1 end-to-end wiring: SQL `LIMIT 1` on claroty.alerts → 1 row returned, `is_truncated: false`; verifies full chain: SQL LIMIT → params.limit → FetchContext.early_stop_limit=Some(1) → execute_impl early-stop check → FetchOutput.any_early_stopped propagation → engine Step 6 is_truncated formula; completes in ≤5s (ADR-060 §D8.1; BC-2.16.002 §Postconditions LIMIT-Aware Early-Stop) | prism-bin, prism-spec-engine, prism-sensors, prism-dtu-claroty |
| HS-EARLY-STOP-001-003 | Plan-Shape Gate Condition A: `SELECT COUNT(*) FROM claroty.alerts` (no SQL LIMIT; Part B adds `LIMIT 1`) → 1 row, count field = 10, `is_truncated: false` in both parts; verifies `ast_is_reducing_plan()` gate in materialization.rs suppresses early-stop for aggregate queries so full dataset is fetched (ADR-060 §D8.7 Condition A; BC-2.16.002 §Postconditions Plan-Shape Gate) | prism-bin, prism-spec-engine, prism-sensors, prism-dtu-claroty |

### ~~HS-020~~: ~~Claroty audit_logs Layer 2 — Dynamic Push-Down~~ — RETIRED before shipping

**RETIRED 2026-08-15:** Single-story design rework collapsed Story B into Story A. HS-020 scenarios HS-AUDITLOG-002-B-001/002 re-keyed to HS-AUDITLOG-001-A-003/004 and moved to HS-019. HS-020 ID reserved per append_only_numbering (DF-030).

---

## Repo Coverage Matrix

Shows which repos are tested by which scenario groups.

| Repo | HS-001 | HS-002 | HS-003 | HS-004 | HS-005 | HS-006 | HS-007 | HS-008 | HS-009 | HS-010 | HS-011 | HS-012 | Total Groups |
|------|--------|--------|--------|--------|--------|--------|--------|--------|--------|--------|--------|--------|-------------|
| poller-cobra | X | X | X | X | X | X | X | X | - | - | - | - | 8/12 |
| poller-express | X | X | X | X | X | X | X | - | - | - | - | - | 7/12 |
| poller-bear | X | X | X | X | X | X | X | X | - | - | - | - | 8/12 |
| poller-coaster | X | X | X | X | X | X | X | X | - | - | - | - | 8/12 |
| tally | X | X | X | X | X | - | X | X | - | - | - | - | 7/12 |
| ocsf-proto-gen | X | X | - | - | - | - | X | X | - | - | - | - | 4/12 |
| axiathon | X | X | X | X | X | - | X | X | - | - | - | - | 7/12 |
| mcp-claroty-xdome | X | X | X | - | X | - | X | X | - | - | - | - | 6/12 |
| serveMyAPI | - | - | - | X | - | - | X | - | - | - | - | - | 2/12 |
| prism-operations | - | - | - | - | - | - | - | - | X | X | X | X | 4/12 |
| prism-storage | - | - | - | - | - | - | - | - | X | X | X | X | 4/12 |
| prism-audit | - | - | - | - | - | - | - | - | X | - | - | X | 2/12 |

---

## Critical Bugs Being Verified as Fixed

These scenarios specifically validate that known bugs from reference repos are NOT inherited by Prism:

| Bug | Source Repo | Severity | Verified By |
|-----|-----------|----------|-------------|
| MemoryStore hardcoded despite FileStore config | poller-cobra | HIGH | HS-007-01 |
| State updated before persistence | poller-cobra | HIGH | HS-007-03, HS-006-02 |
| DisallowUnknownFields breaks forward compat | poller-express | MEDIUM | HS-007-04 |
| No signal handling | poller-express | MEDIUM | HS-006-06 |
| String comparison of numeric IDs | poller-express | MEDIUM | HS-007-05 |
| Inconsistent forward progress error handling | poller-coaster | MEDIUM | HS-006-04 |
| Path traversal in credential store | serveMyAPI | CRITICAL | HS-007-06 |
| Plaintext credential storage | serveMyAPI | CRITICAL | HS-004-01 |
| Session ID collision (Date.now) | serveMyAPI | HIGH | HS-005-07 |
| Hardcoded vault passphrase | axiathon | CRITICAL | HS-004-01 |
| Static Argon2 salt | axiathon | HIGH | HS-004-01 |
| Unbounded caches/sessions | mcp-claroty-xdome | HIGH | HS-007-08 |
| ErrorCode(-1) for all errors | tally | MEDIUM | HS-007-07 |
| Health server shutdown never called | poller-cobra | MEDIUM | HS-006-06 |

---

## Evaluation Criteria

Each scenario is evaluated as:
- **PASS**: Expected outcome fully achieved
- **PARTIAL**: Some expected outcomes achieved, non-critical gaps
- **FAIL**: Expected outcome not achieved or critical regression

Minimum acceptance: All P0 scenarios PASS. P1 scenarios at least PARTIAL.

---

## State Checkpoint

> **Note:** This State Checkpoint reflects the cumulative HOLDOUT-INDEX state as of the most recent version bump. Frontmatter fields (`total_scenarios`, `total_groups`, `timestamp`) are the canonical source of truth; this block is a point-in-time snapshot for downstream tooling consumption. `total_scenarios` counts top-level HS-NNN groups (each HS file = one scenario unit) plus prior sub-scenario accumulation per the v1.3 baseline accounting (52 sub-scenarios → 75 at v1.3, +6 HS files at v1.4 = 81). Verify by frontmatter `total_scenarios:` for the authoritative count.

```yaml
document: holdout-index
phase: 0_and_4b_and_plugin_migration_and_drift_claroty_and_ocsf_routing_regate_and_wave_a_xdome_and_wave_b_xdome_and_wave_c_xdome_and_engine_limit_backfill
step: 5_and_wave4_and_prereq_and_drift_claroty_auditlog_and_hs023_and_hs024_hs025_and_hs026_and_hs027_and_hs028_and_hs029_and_hs030
status: complete
total_scenarios: 118
total_groups: 24
p0_scenarios: 102
p1_scenarios: 16
repos_covered: 9/9_brownfield_plus_3_greenfield
critical_bugs_verified: 14
wave4_groups_added: 4
wave4_scenarios_added: 23
plugin_migration_groups_added: 1
plugin_migration_scenarios_added: 6
drift_claroty_auditlog_groups_added: 2
drift_claroty_auditlog_scenarios_added: 4
ocsf_coercion_groups_added: 1
ocsf_coercion_scenarios_added: 4
ocsf_routing_groups_added: 1
ocsf_routing_scenarios_added: 4
ocsf_routing_regate_groups_added: 1
ocsf_routing_regate_scenarios_added: 3
wave_a_xdome_groups_added: 2
wave_a_xdome_scenarios_added: 6
wave_b_xdome_groups_added: 1
wave_b_xdome_scenarios_added: 3
wave_c_xdome_groups_added: 3
wave_c_xdome_scenarios_added: 10
wave_c_xdome_g5_groups_added: 1
wave_c_xdome_g5_scenarios_added: 4
wave_c_xdome_g6_groups_added: 1
wave_c_xdome_g6_scenarios_added: 3
engine_limit_backfill_groups_added: 1
engine_limit_backfill_scenarios_added: 3
wave4_must_pass_groups: 3
wave4_conditional_pass_groups: 1
d216_closure: true
hs022_consumed: true
hs022_consumed_ref: D-2270
hs023_aw_amendment_coverage: true
hs024_claroty_vulns_001: true
hs025_claroty_ot_events_001: true
hs026_claroty_devvulnrel_001: true
hs027_claroty_servers_001: true
hs028_claroty_orgpolicy_001: true
hs028_consumed: true
hs028_consumed_ref: D-2409
hs029_claroty_aclpolicy_001: true
hs029_live_holdout_human_accepted: true
hs029_human_accepted_ref: D-2414
hs030_engine_limit_early_stop_001: true
hs030_consumed: true
hs030_consumed_ref: D-2370
timestamp: 2026-09-01T19:35:20Z
```

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.34 | G6-live-holdout-acceptance | 2026-09-01 | state-manager | D-2414: G6 (S-CLAROTY-ACLPOLICY-001) live holdout RE-RUN post-FIX-A — SETUP-FAILURE (quiescent data). monroe tenant has ZERO organization ACL policies (confirmed via Claroty xDome UI: Network > Enforcement > ACL Policies > Organization ACL Policies tab showed "0 Organization Policies / No Organization Policies to Show"). API HTTP 404 "No matching policies found" is LEGITIMATE QUIESCENT DATA, not an over-filter defect. FIX-A verified correct: HTTP 422 GONE across all query shapes; binary SHA256 7bb5994f1006fc1ebfe2e10c5d1629e56761f132c2698952d7a46609a57cd2c8 @29695e0b9. Structural/plan-gate/column-resolution assertions ALL PASS (class_uid=3004 recognized; OCSF Tier-1 renames metadata_uid/name/actor_user_name/comment present; SELECT policy_id → E-QUERY-038 with metadata_uid in available_columns; SELECT applied_models → E-QUERY-038; no offset/limit injection → no 422; single round-trip; no standalone count column). HS-001 v1.1 / HS-002 / HS-003 classified SETUP-FAILURE — row-content P0 assertions unexercisable on empty tenant; scenarios NOT consumed (used: false); preserved single-use for future re-run when tenant has ≥1 org ACL policy. HUMAN-ACCEPTED 2026-09-01. hs029_live_holdout_human_accepted: true / hs029_human_accepted_ref: D-2414. HS-029 summary row and detail section updated. HOLDOUT-INDEX v1.33→v1.34. |
| 1.33 | G6-FIX-A-session-wrap | 2026-09-01 | state-manager | D-2413: G6 (S-CLAROTY-ACLPOLICY-001) holdout gate ATTEMPTED on monroe — FAILED HTTP 422 (body_template omitted mandatory filter_by.policy_id selector required by organization_acl_policies API; real API-contract defect NOT a behavioral assertion failure). FIX-A applied: code @29695e0b9 LOCAL-ONLY (filter_by selector + regression test); BC-2.16.022 v1.2→v1.3 (EC-016-022-011 filter_by contract + AC-003 + RG-013); story v1.5→v1.6. HS-001 v1.0→v1.1 (product-owner raw_extensions verdict-A pre-run: raw_extensions is JSON-serialized STRING per ADR-058 §I2, consistent G1–G5 VERDICT-A; corrected before the gate attempt). All 3 HS-029 scenarios NOT consumed (used: false; gate must re-run after FIX-A binary deploy to test-soc). HS-029 summary row updated (GATE FAILED annotation). HS-029 detail section updated (Result column + gate-failed annotation + HS-001 description updated for v1.1 raw_extensions verdict-A). HOLDOUT-INDEX v1.32→v1.33. |
| 1.32 | G5-live-holdout-gate | 2026-09-01 | state-manager | D-2409: G5 (S-CLAROTY-ORGPOLICY-001) live holdout gate EXECUTED on monroe (Variant-2 read-only; setup healthy; real live data). HS-001 PASS 1.00 (consumed; class_uid=3004 + name Tier-1 REQUIRED non-null string + raw_extensions JSON-serialized string per ADR-058 §I2 with zone Tier-2 keys; HS-001 v1.0→v1.1 product-owner raw_extensions wire-encoding correction pre-consumption). HS-002 PASS 1.00 (consumed; zone_policies Json columns communication_conditions/related_alerts_ids/applied_zone_pairs as JSON array values; SELECT communication_conditions E-QUERY-038 confirmed). HS-003 PASS 1.00 (consumed; firewall_groups class_uid=3004 + name Tier-1 REQUIRED + URL/envelope asymmetry verified; raw_extensions JSON-serialized string per ADR-058 §I2; HS-003 v1.0→v1.1 product-owner amendment pre-consumption). HS-004 PASS 1.00 (consumed; applied_group_pairs in raw_extensions as JSON array; SELECT communication_conditions E-QUERY-038; SELECT activity_name accepted). Mean satisfaction 1.00 all 4 must_pass. GATE PASS. OBSERVATION adjudicated VERDICT-A per D-2406 (same ruling as G4): raw_extensions is Arrow Utf8 column emitted as JSON-serialized STRING per ADR-058 §I2 — INTENDED. HS-028 summary row updated (consumed annotation + description corrections). HS-028 detail section updated (EVALUATION RESULT + Result column). Scenario files: used→true / lifecycle_status→consumed / last_evaluated 2026-09-01 / last_eval_satisfaction 1.00. HS-002 input-hash corrected f660fcc→a91185c (BC-2.16.020 update drift; same inputs as HS-001). HS-004 input-hash corrected 967aa52→f0cbf03 (BC-2.16.021 update drift; same inputs as HS-003). HOLDOUT-INDEX v1.31→v1.32. |
| 1.31 | G4-live-holdout-gate | 2026-09-01 | state-manager | D-2406: G4 (S-CLAROTY-SERVERS-001) live holdout gate EXECUTED on monroe (Variant-2 read-only; setup healthy; real live data _source_type:live; both tables populated). HS-001 PASS 1.00 (consumed: class_uid=5001 + device_name Tier-1 REQUIRED present as non-null string + raw_extensions JSON-serialized string with inventory keys; HS-001 v1.0→v1.1 product-owner amendment pre-consumption). HS-002 PASS 1.00 (consumed: server_name rejected E-QUERY-038 with device_name in available_columns; SELECT device_name and status_code accepted). HS-003 PASS 1.00 (consumed: server_interfaces separate endpoint queryable; interface_status rejected E-QUERY-038 with status_code in available_columns; raw_extensions JSON-serialized string with interface Tier-2 keys; HS-003 v1.0→v1.1 product-owner amendment pre-consumption). Mean satisfaction 1.00 all 3 must_pass. GATE PASS. OBSERVATION adjudicated VERDICT-A (architect): raw_extensions is Arrow Utf8 column emitted as JSON-serialized STRING per ADR-058 §I2 — INTENDED, not a defect; consistent across G1/G2/G3. HS-027 summary row updated (consumed annotation + description corrections). HS-027 detail section updated (Result column added to header + 3 result cells + raw_extensions description corrected to JSON-serialized string). Scenario files: used→true / lifecycle_status→consumed / last_evaluated 2026-09-01 / last_eval_satisfaction 1.00. HS-002 input-hash updated 9311257→6ca4e11 (drift from BC-2.16.018 update; same inputs as HS-001 which already carried 6ca4e11). HOLDOUT-INDEX v1.30→v1.31. |
| 1.30 | G3-live-holdout-gate | 2026-08-31 | state-manager | D-2403: G3 (S-CLAROTY-DEVVULNREL-001) live holdout gate EXECUTED on monroe (Variant-2 read-only; live data _source_type:live). HS-001 PASS 1.00 (consumed: class_uid=2002 + finding_info_title CVE string non-null). HS-003 PASS 1.00 (consumed: SELECT time accepted; SELECT device_vulnerability_detection_date E-QUERY-038 with time in available_columns). HS-002 PASS 0.75 at-threshold (consumed; VERDICT-A: over-specified — vulnerability_name is Tier-1 finding_info_title NOT in raw_extensions per ADR-058 §B2; product-owner amended HS-002 v1.0→v1.1 pre-consumption; scenario files used→true / lifecycle_status→consumed / last_eval_satisfaction set). Mean satisfaction 0.92. GATE PASS. HS-026 summary row updated (consumed annotation + corrected HS-002 description). HS-026 detail section updated (EVALUATION RESULT + Result column). HS-001/HS-003 input-hash fdccb7c→3ad5d86 (drift from BC-2.16.017 update by product-owner; same inputs as HS-002 which already carried 3ad5d86). HOLDOUT-INDEX v1.29→v1.30. |
| 1.29 | G2-live-holdout-gate | 2026-08-31 | state-manager | D-2399: G2 (S-CLAROTY-OT-EVENTS-001) live holdout gate EXECUTED on monroe (Variant-2 read-only). HS-002 PASS 1.00 (consumed: source_ip E-QUERY-038 + raw_extensions queryable). HS-003 PASS 1.00 (consumed: SELECT time succeeded; SELECT detection_time E-QUERY-038 with time in available_columns). HS-001 SETUP-FAILURE (quiescent OT network — 0 events; NOT consumed; kept for future re-run). Human ACCEPTED (Option-1): gate treated as PASSED on 2/3-at-ceiling + structural corroboration. HS-025 group summary row + detail section updated. HOLDOUT-INDEX v1.28→v1.29. |
| 1.28 | S-ENGINE-LIMIT-EARLY-STOP-001-holdout-gate-pass | 2026-08-30 | state-manager | HS-030 all 3 scenarios CONSUMED (D-2370 story-level holdout gate PASS mean 1.00; S-ENGINE-LIMIT-EARLY-STOP-001 VERDICT=PASS 3/3 P0 on frozen HEAD @1c1159c68; real MCP stdio + Claroty DTU; wire-level assertions; negative control confirmed is_truncated:true under tool limit=3). Scenario files used→true; last_evaluated 2026-08-30; last_eval_satisfaction 1.0. HS-030 table row and detail section annotated CONSUMED. State checkpoint hs030_consumed: true / hs030_consumed_ref: D-2370. HOLDOUT-INDEX v1.27→v1.28. |
| 1.27 | S-ENGINE-LIMIT-EARLY-STOP-001-holdout-backfill | 2026-08-30 | product-owner | Registered HS-030 (3 scenarios for S-ENGINE-LIMIT-EARLY-STOP-001 holdout backfill: HS-EARLY-STOP-001-001 partial-final-page discriminator [SQL LIMIT=10 on claroty.alerts DTU 10-record fixture, page_size=1000 → PARTIAL page, early_stopped=false → is_truncated=false; broken discriminator gives is_truncated=true; catches EC-11-094 violation from F-P31-LENSA-OBS-001; ADR-060 §D8.2/§D8.3; BC-2.11.001 EC-11-094], HS-EARLY-STOP-001-002 LIMIT 1 end-to-end wiring [SQL LIMIT 1 → params.limit → FetchContext.early_stop_limit=Some(1) → execute_impl check → FetchOutput propagation → engine Step 6; 1 row returned, is_truncated=false; ADR-060 §D8.1; BC-2.16.002 §Postconditions LIMIT-Aware Early-Stop], HS-EARLY-STOP-001-003 plan-shape gate Condition A [SELECT COUNT(*) → count=10, is_truncated=false; SELECT COUNT(*) LIMIT 1 also count=10; ast_is_reducing_plan() gate in materialization.rs suppresses early-stop; ADR-060 §D8.7 Condition A]). All 3 P0 against Claroty DTU (10-record alerts fixture). Story materialized after last holdout-authoring burst — process-gap backfill per story-level holdout gate protocol. total_scenarios 115→118; total_groups 23→24; p0_scenarios 99→102. |
| 1.26 | xdome-wave-c-f2-spec-evolution-g6 | 2026-08-24 | product-owner | Registered HS-029 (3 scenarios for S-CLAROTY-ACLPOLICY-001: HS-ACLPOLICY-001-001 claroty_organization_acl_policies SELECT * wire shape [class_uid=3004; metadata_uid Tier-1 REQUIRED from policy_id→metadata.uid via ocsf_field_to_arrow_name; name Tier-1 from policy_name; raw_extensions with ACL Tier-2 keys; policy_id NOT standalone], HS-ACLPOLICY-001-002 applied_models Json column enforcement [raw_extensions.applied_models is JSON array not quoted string; SELECT applied_models → E-QUERY-038; policy_acl String Tier-2 also in raw_extensions], HS-ACLPOLICY-001-003 KEY NOVELTY PaginationConfig::None single-page fetch [unbounded SELECT succeeds without 422 — no offset/limit injection; no count column in wire output; SELECT policy_id raw name → E-QUERY-038 with metadata_uid in available_columns; SELECT metadata_uid accepted]). All 3 P0 against live monroe sensor (no DTU per D-2200; SAP-2 N/A). BC-2.16.022 Wave C G6. total_scenarios 112→115; total_groups 22→23; p0_scenarios 96→99. |
| 1.25 | xdome-wave-c-f2-spec-evolution | 2026-08-24 | product-owner | Registered HS-028 (4 scenarios for S-CLAROTY-ORGPOLICY-001: HS-ORGPOL-001-001 claroty_organization_zones SELECT * wire shape [class_uid=3004, name Tier-1 REQUIRED from zone_name→entity_management name, raw_extensions with zone Tier-2 keys; zone_name NOT standalone], HS-ORGPOL-001-002 claroty_organization_zone_policies Json columns in raw_extensions [communication_conditions/related_alerts_ids/applied_zone_pairs as JSON arrays not quoted strings; SELECT communication_conditions → E-QUERY-038], HS-ORGPOL-001-003 claroty_organization_firewall_groups SELECT * [class_uid=3004, name Tier-1 REQUIRED, URL /api/v1/organization_fw_groups/ vs envelope $.organization_firewall_groups asymmetry verified via non-empty result], HS-ORGPOL-001-004 claroty_organization_firewall_policies [applied_group_pairs Json in raw_extensions; SELECT communication_conditions → E-QUERY-038; SELECT activity_name policy_action→activity_name Tier-1 accepted]). All 4 against live monroe sensor (no DTU per D-2200). BC-2.16.020 Wave C G5 (zones+zone_policies domain pair) + BC-2.16.021 Wave C G5 (firewall_groups+firewall_policies domain pair). total_scenarios 108→112; total_groups 21→22; p0_scenarios 92→96. |
| 1.24 | xdome-wave-c-f2-spec-evolution | 2026-08-24 | product-owner | Registered HS-027 (3 scenarios for S-CLAROTY-SERVERS-001: HS-SERVERS-001-001 claroty_servers SELECT * wire shape [class_uid=5001, device_name Tier-1 REQUIRED from server_name→device.name, raw_extensions with inventory keys], HS-SERVERS-001-002 claroty_servers Tier-1 rename enforcement [SELECT server_name→E-QUERY-038 with device_name in available_columns; SELECT device_name and status_code accepted], HS-SERVERS-001-003 claroty_server_interfaces separate /api/v1/server_interfaces/ endpoint [class_uid=5001; SELECT interface_status→E-QUERY-038 with status_code in available_columns; SELECT raw_extensions with interface Tier-2 keys]). All 3 against live monroe sensor (no DTU per D-2200). BC-2.16.018 Wave C G4 (17-field Servers, POST /api/v1/servers/) + BC-2.16.019 Wave C G4 (10-field ServerInterfaces, SEPARATE endpoint POST /api/v1/server_interfaces/, composite PK server_name+interface_name). total_scenarios 105→108; total_groups 20→21; p0_scenarios 89→92. |
| 1.23 | xdome-wave-b-f2-spec-evolution | 2026-08-24 | product-owner | Registered HS-026 (3 scenarios for S-CLAROTY-DEVVULNREL-001: HS-DEVVULNREL-001-001 class_uid=2002 wire shape + finding_info_title Tier-1 REQUIRED [vulnerability_name→finding_info.title], HS-DEVVULNREL-001-002 composite join-key correctness [vulnerability_name + device_uid in raw_extensions] + Tier-2 plan-gate [SELECT device_uid → E-QUERY-038], HS-DEVVULNREL-001-003 Tier-1 detection-date→time rename [SELECT time accepts; SELECT device_vulnerability_detection_date → E-QUERY-038]). All 3 against live monroe sensor (no DTU per D-2200). BC-2.16.017 Wave B G3 (214-field DeviceVulnerability; first cut 13 cols; composite PK). total_scenarios 102→105; total_groups 19→20; p0_scenarios 86→89. |
| 1.22 | xdome-wave-a-f2-spec-evolution | 2026-08-24 | product-owner | Registered HS-024 (3 scenarios for S-CLAROTY-VULNS-001: HS-VULNS-001-001 class_uid=2002 wire shape + finding_info_title Tier-1, HS-VULNS-001-002 Tier-1 OCSF column plan-gate acceptance + raw col.name rejection, HS-VULNS-001-003 raw_extensions Tier-2 aggregation content with cve_ids + severity_score keys) and HS-025 (3 scenarios for S-CLAROTY-OT-EVENTS-001: HS-OTEVTS-001-001 class_uid=2004 Option B wire shape + finding_info_uid REQUIRED Tier-1, HS-OTEVTS-001-002 Tier-2 network 5-tuple source_ip E-QUERY-038 rejection + raw_extensions plan-gate acceptance, HS-OTEVTS-001-003 Tier-1 detection_time→time OCSF rename). All 6 against live monroe sensor (no DTU for G2 per D-2200). total_scenarios 96→102; total_groups 17→19; p0_scenarios 80→86. |
| 1.20 | S-ADR058-OCSF-ROUTING-001-aw-regate-holdout-authoring | 2026-08-23 | product-owner | Registered HS-023 (3 scenarios for S-ADR058-OCSF-ROUTING-001 A+W re-gate: HS-ROUTING-001-B-001 zero-Tier-1 A+W warning + raw_extensions available-set, HS-ROUTING-001-B-002 §J4 intra-table collision E-SPEC-030 rejection, HS-ROUTING-001-B-003 audit_logs metadata_uid OQ-005 wire shape + E-QUERY-038 available_columns). HS-022 annotated CONSUMED (D-2270; A+W amendment 2026-08-23 post-dates original scenarios; re-gate required). total_scenarios 93→96; total_groups 16→17; p0_scenarios 77→80. State Checkpoint updated. |
| 1.19 | S-ADR058-OCSF-ROUTING-001-pre-delivery | 2026-08-21 | story-writer | Registered HS-022 (4 scenarios for S-ADR058-OCSF-ROUTING-001: HS-ROUTING-001-A-001 claroty alerts ocsf_field Arrow name wire shape, HS-ROUTING-001-A-002 prism_describe raw_extensions Tier-2 prohibition, HS-ROUTING-001-A-003 claroty audit_logs class_uid 3004 wire shape, HS-ROUTING-001-A-004 claroty devices device_type_label filter). total_scenarios 89→93; total_groups 15→16. State Checkpoint updated. |
| 1.18 | S-ADR058-OCSF-COERCION-001-holdout-authoring | 2026-08-19 | product-owner | Authored HS-021 (4 scenarios for S-ADR058-OCSF-COERCION-001: HS-COERCION-001-A-001..004 covering String+Object null-demotion, Integer+parseable-string no-loss, Integer+non-parseable-string null+warn, ENRICH-1 Array non-regression). total_scenarios 85→89; total_groups 14→15. |
| 1.17 | S-CLAROTY-AUDITLOG-TIMEBOX-001-holdout-gate-pass | 2026-08-15 | state-manager | HS-019 all 4 scenarios CONSUMED (D-2195 story-level holdout gate PASS; S-CLAROTY-AUDITLOG-TIMEBOX-001 VERDICT=PASS 4/4 on frozen HEAD f867a234b). Scenario files status→consumed; input-hashes populated. HS-019 table row and detail section annotated CONSUMED. HOLDOUT-INDEX v1.16→v1.17. |
| 1.16 | DRIFT-CLAROTY-AUDITLOG-TIMEOUT-001-registration-burst | 2026-08-15 | state-manager | File renames: S-CLAROTY-AUDITLOG-TIMEBOX-002-HS-001-explicit-time-filter-not-truncated.md → S-CLAROTY-AUDITLOG-TIMEBOX-001-HS-003-explicit-time-filter-not-truncated.md; S-CLAROTY-AUDITLOG-TIMEBOX-002-HS-002-time-range-both-bounds.md → S-CLAROTY-AUDITLOG-TIMEBOX-001-HS-004-time-range-both-bounds.md. HS-019 table links updated to reflect new TIMEBOX-001-HS-003/004 names. HOLDOUT-INDEX v1.15→v1.16. |
| 1.15 | DRIFT-CLAROTY-AUDITLOG-TIMEOUT-001-po-bc-amendments | 2026-08-15 | product-owner | Design rework: two-layer design collapsed to single story S-CLAROTY-AUDITLOG-TIMEBOX-001. HS-020 retired before shipping (append_only_numbering preserved). All 4 scenarios consolidated under HS-019: HS-AUDITLOG-002-B-001/002 re-keyed to HS-AUDITLOG-001-A-003/004 with `story_source: S-CLAROTY-AUDITLOG-TIMEBOX-001`; "explicit-filter-not-truncated" scenario body updated — asserts CORRECT behavior (older window IS returned, no truncation); Layer-1/2 language removed. total_groups 15→14. |
| 1.14 | DRIFT-CLAROTY-AUDITLOG-TIMEOUT-001-po-bc-amendments | 2026-08-15 | product-owner | Initial DRIFT-CLAROTY-AUDITLOG-TIMEOUT-001 holdout authoring: HS-019 (2 for Story A) + HS-020 (2 for Story B). total_scenarios 81→85; total_groups 13→15. Superseded by v1.15 design rework in same burst. |
| 1.13 | FB-IMPL-9 | 2026-05-21 | state-manager | FB-IMPL-9 transitive cite-pin sweep: HS-018 v1.3→v1.4 — swept `error-taxonomy.md v1.42` → `v1.44` at 3 active-prose sites (frontmatter notes, HS-018-01 §Expected Outcome, HS-018-02 §Expected Outcome). Additional discovery beyond pass-10 adversary enumeration. HOLDOUT-INDEX v1.12→v1.13. |
| 1.12 | FB-IMPL-P22-PO | 2026-05-21 | product-owner | F-LP22-MED-001 closure (16th coherence-axis: same-line dual-format cite-pin escape): HS-018 v1.2→v1.3 — swept `error-taxonomy.md v1.41` → `v1.42` at 3 active-prose sites (frontmatter notes, HS-018-01 §Expected Outcome, HS-018-02 §Expected Outcome). Summary row updated. HOLDOUT-INDEX v1.11→v1.12. |
| 1.11 | FB-IMPL-P21-PO | 2026-05-21 | product-owner | F-LP21-MED-001 closure (15th coherence-axis: section-versioned cite-pin format): HS-018 v1.1→v1.2 — stripped `v1.2` section-version pin from §Expected Outcome BC-2.16.013 cite per Option A. Summary row updated. HOLDOUT-INDEX v1.10→v1.11. |
| 1.10 | FB-IMPL-P13-PO | 2026-05-20 | product-owner | F-LP13-MED-002 closure: HS-018 v1.0→v1.1 — added §Evaluation Criteria with coverage mapping for HS-018-01/02/03; applied Option A confirming HS-018-02 (case-mismatch) covered by RG-09 case-sensitive string equality enforcement; §Changelog added per POL-26. Summary row updated to note v1.1 coverage clarification. HOLDOUT-INDEX v1.9→v1.10. |
| 1.9 | FB-IMPL-P12-PO | 2026-05-20 | product-owner | F-LP12-MED-002 closure: Backfilled missing v1.4 changelog row (HS-013..018 authoring, 75→81); corrected disambiguating prose at line 292 from "+6 HS files at v1.7" → "+6 HS files at v1.4" (HS-013..018 entered at v1.4, not v1.7). POL-26 changelog continuity discipline applied. HOLDOUT-INDEX v1.8→v1.9. |
| 1.8 | FB-IMPL-P11-PO | 2026-05-20 | product-owner | F-LP11-MED-001 closure + proactive embedded-state-block sweep per S-7.02 lesson preview: State Checkpoint yaml block refreshed (total_scenarios 75→81, total_groups 12→13, p0_scenarios 59→65, timestamp 2026-05-04→2026-05-20, phase/step/plugin_migration fields added). Disambiguating prose block added before yaml. HOLDOUT-INDEX v1.7→v1.8. |
| 1.7 | FB-IMPL-P6-PO fix-burst-6 | 2026-05-20 | product-owner | F-LP6-LOW-001 closure (TD-VSDD-091 POL-25 sweep): HS-016 v1.1→v1.2 — replaced line-pinned cite `lib.rs:16-17` with module-doc anchor `crates/prism-dtu-armis/src/lib.rs module documentation` in §Scenario auth note. HOLDOUT-INDEX v1.6→v1.7. |
| 1.6 | FB-IMPL-P5-PO fix-burst-5 | 2026-05-20 | product-owner | F-LP5-LOW-001 closure (TD-VSDD-091 POL-25 sweep): HS-015 v1.1→v1.2 — replaced line-pinned cites `alerts.rs:43-46` with symbol anchor `alerts.rs::extract_session_token()` at two locations (§Scenario auth note, HS-015-01 Step 2). HOLDOUT-INDEX v1.5→v1.6. |
| 1.5 | FB-IMPL-P4-PO fix-burst-4 | 2026-05-20 | product-owner | F-LP4-HIGH-001/F-LP4-HIGH-002/F-LP4-HIGH-004 closure: HS-013 v1.0→v1.1 (URL re-grounded to `/detects/queries/detects/v1`+`/detects/entities/summaries/GET/v1` per ADR-028 §D1; fixture reference at `prism-dtu-crowdstrike/fixtures/parity/reference-ocsf/detections.json` per ADR-028 §D3; request_count relaxed to >=2). HS-014 v1.0→v1.1 (auth corrected to `bearer_static` per ADR-028 §D2; scenario pivoted to `alerts` table at `POST /api/v1/alerts` per ADU gap note DTU-EXT-002; fixture reference added). HS-015 v1.0→v1.1 (auth corrected to `cookie_roundtrip` per ADR-028 §D2; URL corrected from `/api/alerts` to `/api/v1/alerts` per ADR-028 §D1; fixture reference added). HS-016 v1.0→v1.1 (auth corrected to `bearer_static` per ADR-028 §D2; DTU gap noted for AQL routes DTU-EXT-003/004; fixture reference added; bearer auth step added). |
| 1.4 | FB-IMPL-P1-PO | 2026-05-20 | product-owner | Authored HS-013..HS-018 (6 new holdout scenarios for PLUGIN-MIGRATION-001-D: HS-013 CrowdStrike DTU parity, HS-014 Claroty POST-for-read parity, HS-015 Cyberint cookie cursor parity, HS-016 Armis AQL timestamp fallback parity, HS-017 bundled spec validation gate, HS-018 spec_id/filename mismatch rejection). total_scenarios 75 → 81; total_groups 12 → 13. |
| 1.3 | wave4-holdout-authoring | 2026-05-04 | product-owner | D-216 closure (Phase 4.B wave gate unblock, D-219 first-wave-with-proper-holdouts): authored HS-009 (6 sub-scenarios, Scheduler Operations, must_pass: true), HS-010 (6 sub-scenarios, Detection & Alert Pipeline, must_pass: true), HS-011 (5 sub-scenarios, Case Management, must_pass: false), HS-012 (6 sub-scenarios, Action Delivery, must_pass: true). total_scenarios 52 → 75 (+23). total_groups 8 → 12. p0_scenarios 36 → 59. BC anchors drawn from BC-INDEX v4.32 (BC-2.12.001–010, BC-2.13.001–013, BC-2.14.001–012, BC-2.18.001–009). Repo Coverage Matrix extended with prism-operations, prism-storage, prism-audit columns. |
| 1.2 | pass-81-remediation | 2026-04-21 | product-owner | F81-006: Synced body "Total Scenarios" (53 → 52) and state checkpoint (total_scenarios: 53 → 52, p0_scenarios: 37 → 36). HS-001-05 was P0; body/checkpoint were stale vs frontmatter. |
| 1.1 | pass-80-remediation | 2026-04-21 | product-owner | F80-006: HS-001-05 marked REMOVED — CAP-013 (xMP Envelope Delivery) is out of scope (REMOVED from capabilities.md). total_scenarios decremented 53 → 52. |
