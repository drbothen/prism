---
document_type: holdout-scenario-index
level: L3
version: "1.17"
status: draft
producer: product-owner
timestamp: 2026-08-15T00:00:00Z
phase: 3
inputs: []
input-hash: null
traces_to: prd.md
total_scenarios: 85
---

# Holdout Scenario Index -- Prism

**Date:** 2026-08-15 (updated)
**Phase:** 0 (Multi-Repo Synthesis -- Step 5) / Phase 4.B (Wave 4 Holdout Coverage) / Phase 3 Wave 0 Plugin Migration / Phase 3 DRIFT-CLAROTY-AUDITLOG-TIMEOUT-001
**Total Scenarios:** 85 (81 prior + 4 new HS-019 for DRIFT-CLAROTY-AUDITLOG-TIMEOUT-001; HS-020 retired before shipping)
**Total Groups:** 14
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
phase: 0_and_4b_and_plugin_migration_and_drift_claroty
step: 5_and_wave4_and_prereq_and_drift_claroty_auditlog
status: complete
total_scenarios: 85
total_groups: 14
p0_scenarios: 69
p1_scenarios: 16
repos_covered: 9/9_brownfield_plus_3_greenfield
critical_bugs_verified: 14
wave4_groups_added: 4
wave4_scenarios_added: 23
plugin_migration_groups_added: 1
plugin_migration_scenarios_added: 6
drift_claroty_auditlog_groups_added: 2
drift_claroty_auditlog_scenarios_added: 4
wave4_must_pass_groups: 3
wave4_conditional_pass_groups: 1
d216_closure: true
timestamp: 2026-08-15T00:00:00Z
```

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
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
