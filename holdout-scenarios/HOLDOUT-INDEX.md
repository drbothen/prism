---
document_type: holdout-scenario-index
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-05-04T00:00:00Z
phase: 1b
inputs: []
input-hash: null
traces_to: prd.md
total_scenarios: 75
---

# Holdout Scenario Index -- Prism

**Date:** 2026-05-04
**Phase:** 0 (Multi-Repo Synthesis -- Step 5) / Phase 4.B (Wave 4 Holdout Coverage)
**Total Scenarios:** 75
**Total Groups:** 12
**Input Sources:** 9 pass-8 deep synthesis files, cross-repo-dependencies.md, unified-security-posture.md; Wave 4 stories S-4.01–S-4.08, BC-INDEX v4.32, ADR-013 §2.1, D-209, ADR-016 §2.5, ADR-008

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

```yaml
document: holdout-index
phase: 0_and_4b
step: 5_and_wave4
status: complete
total_scenarios: 75
total_groups: 12
p0_scenarios: 59
p1_scenarios: 16
repos_covered: 9/9_brownfield_plus_3_greenfield
critical_bugs_verified: 14
wave4_groups_added: 4
wave4_scenarios_added: 23
wave4_must_pass_groups: 3
wave4_conditional_pass_groups: 1
d216_closure: true
timestamp: 2026-05-04T00:00:00Z
```

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | wave4-holdout-authoring | 2026-05-04 | product-owner | D-216 closure (Phase 4.B wave gate unblock, D-219 first-wave-with-proper-holdouts): authored HS-009 (6 sub-scenarios, Scheduler Operations, must_pass: true), HS-010 (6 sub-scenarios, Detection & Alert Pipeline, must_pass: true), HS-011 (5 sub-scenarios, Case Management, must_pass: false), HS-012 (6 sub-scenarios, Action Delivery, must_pass: true). total_scenarios 52 → 75 (+23). total_groups 8 → 12. p0_scenarios 36 → 59. BC anchors drawn from BC-INDEX v4.32 (BC-2.12.001–010, BC-2.13.001–013, BC-2.14.001–012, BC-2.18.001–009). Repo Coverage Matrix extended with prism-operations, prism-storage, prism-audit columns. |
| 1.2 | pass-81-remediation | 2026-04-21 | product-owner | F81-006: Synced body "Total Scenarios" (53 → 52) and state checkpoint (total_scenarios: 53 → 52, p0_scenarios: 37 → 36). HS-001-05 was P0; body/checkpoint were stale vs frontmatter. |
| 1.1 | pass-80-remediation | 2026-04-21 | product-owner | F80-006: HS-001-05 marked REMOVED — CAP-013 (xMP Envelope Delivery) is out of scope (REMOVED from capabilities.md). total_scenarios decremented 53 → 52. |
