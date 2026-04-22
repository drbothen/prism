---
document_type: remediation-manifest
version: "1.0"
date: "2026-04-20"
author: story-writer
directive: "User Option 2 — DTU-first: product stories that need to test against a sensor/service MUST have the DTU clone built first because tests cannot run without fixtures."
tracks_reversed: ["Track A IMP-001-B (removed blocks edges from 13 DTU stories)"]
---

# Remediation Manifest: Step 5 Option 2 — DTU-First Wave Rework

## Summary

This manifest documents the reversal of Track A's IMP-001-B remediation (which removed
DTU → product `blocks:` edges) and applies user directive Option 2: DTU-first scheduling
where product stories that need DTU clones as test fixtures explicitly depend on them.

**Files changed: 20 total (13 DTU stories + 7 product stories + STORY-INDEX)**

---

## Action 1 — Restore `blocks:` on 13 DTU stories

| Story File | Old blocks | New blocks | Wave Change | Version |
|------------|------------|------------|-------------|---------|
| S-6.07-dtu-crowdstrike.md | [] | [S-3.06, S-3.07] | 0 → 1 | 1.2 → 1.3 |
| S-6.08-dtu-claroty.md | [] | [S-3.02] | 0 → 1 | 1.2 → 1.3 |
| S-6.09-dtu-cyberint.md | [] | [S-3.02] | 0 → 1 | 1.2 → 1.3 |
| S-6.10-dtu-armis.md | [] | [S-3.02] | 0 → 1 | 1.2 → 1.3 |
| S-6.11-dtu-slack.md | [] | [S-4.08, S-5.06] | 0 → 2 | 1.3 → 1.4 |
| S-6.12-dtu-pagerduty.md | [] | [S-4.08, S-5.06] | 0 → 2 | 1.3 → 1.4 |
| S-6.13-dtu-jira.md | [] | [S-4.08, S-5.06] | 0 → 2 | 1.3 → 1.4 |
| S-6.14-dtu-threatintel.md | [] | [S-1.14, S-5.06] | 0 (unchanged) | 1.2 → 1.3 |
| S-6.15-dtu-nvd.md | [] | [S-1.14, S-5.06] | 0 (unchanged) | 1.2 → 1.3 |
| S-6.16-dtu-datadog.md | [] | [S-5.09] | 0 → 3 | 1.2 → 1.3 |
| S-6.17-dtu-splunk-hec.md | [] | [S-5.09] | 0 → 3 | 1.2 → 1.3 |
| S-6.18-dtu-elasticsearch.md | [] | [S-5.09] | 0 → 3 | 1.2 → 1.3 |
| S-6.19-dtu-otlp.md | [] | [S-5.09] | 0 → 3 | 1.2 → 1.3 |

Note: S-6.06 (DTU common infrastructure) is unchanged — wave 0, no product `blocks:` edges.
Note: S-6.14 and S-6.15 remain wave 0 because they block wave-1 S-1.14. Moving them to
wave 1 would violate the wave ordering invariant (must precede consumer wave).

---

## Action 2 — Add reciprocal `depends_on:` on 7 product stories

| Story File | DTU deps added | Old depends_on | New depends_on (DTU portion) | Version |
|------------|----------------|----------------|------------------------------|---------|
| S-1.14-infusion-specs.md | S-6.14, S-6.15 | [S-1.11] | [S-1.11, S-6.14, S-6.15] | 1.1 → 1.2 |
| S-3.02-query-materialization.md | S-6.08, S-6.09, S-6.10 | [S-3.01, S-2.06, S-1.04, S-2.01, S-2.03] | [...+S-6.08, S-6.09, S-6.10] | 1.2 → 1.3 |
| S-3.06-prismql-write-parser.md | S-6.07 | [S-3.01, S-1.13] | [S-3.01, S-1.13, S-6.07] | 1.2 → 1.3 |
| S-3.07-write-execution.md | S-6.07 | [S-3.06, S-3.02, S-1.08, S-1.09, S-2.04] | [...+S-6.07] | 1.2 → 1.3 |
| S-4.08-action-delivery.md | S-6.11, S-6.12, S-6.13 | [S-4.05, S-4.06, S-4.01, S-1.15] | [...+S-6.11, S-6.12, S-6.13] | 1.1 → 1.2 |
| S-5.06-action-infusion-tools.md | S-6.11, S-6.12, S-6.13, S-6.14, S-6.15 | [S-5.01, S-4.08, S-1.14] | [...+S-6.11, S-6.12, S-6.13, S-6.14, S-6.15] | 1.5 → 1.6 |
| S-5.09-external-log-forwarding.md | S-6.16, S-6.17, S-6.18, S-6.19 | [S-5.08, S-1.15] | [S-5.08, S-1.15, S-6.16, S-6.17, S-6.18, S-6.19] | 1.1 → 1.2 |

---

## Action 3 — Wave Schedule Rework

### New DTU Wave Distribution

| Wave | DTU Stories | Rationale |
|------|-------------|-----------|
| 0 | S-6.06 (DTU common), S-6.14 (threat intel), S-6.15 (NVD) | S-6.14/15 block wave-1 S-1.14; must precede it |
| 1 | S-6.07 (CrowdStrike), S-6.08 (Claroty), S-6.09 (Cyberint), S-6.10 (Armis) | Block wave-3 S-3.02, S-3.06, S-3.07 |
| 2 | S-6.11 (Slack), S-6.12 (PagerDuty), S-6.13 (Jira) | Block wave-4 S-4.08 and wave-5 S-5.06 |
| 3 | S-6.16 (Datadog), S-6.17 (Splunk HEC), S-6.18 (Elasticsearch), S-6.19 (OTLP) | Block wave-5 S-5.09 |
| 6 | S-6.04 (credential CLI), S-6.05 (migrate-storage) | Independent product stories; unchanged |

### Dependency Chain Verification (no cycles)

All DTU → product edges checked: DTU wave < product consumer wave in every case.

| DTU Story | DTU Wave | Product Consumer | Consumer Wave | Diff | Status |
|-----------|----------|-----------------|---------------|------|--------|
| S-6.14 | 0 | S-1.14 | 1 | +1 | OK |
| S-6.15 | 0 | S-1.14 | 1 | +1 | OK |
| S-6.14 | 0 | S-5.06 | 5 | +5 | OK |
| S-6.15 | 0 | S-5.06 | 5 | +5 | OK |
| S-6.07 | 1 | S-3.06 | 3 | +2 | OK |
| S-6.07 | 1 | S-3.07 | 3 | +2 | OK |
| S-6.08 | 1 | S-3.02 | 3 | +2 | OK |
| S-6.09 | 1 | S-3.02 | 3 | +2 | OK |
| S-6.10 | 1 | S-3.02 | 3 | +2 | OK |
| S-6.11 | 2 | S-4.08 | 4 | +2 | OK |
| S-6.12 | 2 | S-4.08 | 4 | +2 | OK |
| S-6.13 | 2 | S-4.08 | 4 | +2 | OK |
| S-6.11 | 2 | S-5.06 | 5 | +3 | OK |
| S-6.12 | 2 | S-5.06 | 5 | +3 | OK |
| S-6.13 | 2 | S-5.06 | 5 | +3 | OK |
| S-6.16 | 3 | S-5.09 | 5 | +2 | OK |
| S-6.17 | 3 | S-5.09 | 5 | +2 | OK |
| S-6.18 | 3 | S-5.09 | 5 | +2 | OK |
| S-6.19 | 3 | S-5.09 | 5 | +2 | OK |

**Result: No cycles detected. All depends_on edges satisfied by earlier-wave predecessors.**

Cycle check rationale: DTU stories depend only on S-0.02 (wave 0) or S-6.06 (wave 0).
Product stories depend on DTU stories only — there is no path from any DTU story back to
itself through product story dependencies. The dependency graph remains a DAG.

---

## Action 4 — STORY-INDEX.md (v1.28 → v1.29)

### Changes made to STORY-INDEX.md

1. **Frontmatter:** version v1.28 → v1.29, timestamp updated to 2026-04-20
2. **Wave Summary table:** Updated to show distributed DTU wave placement across waves 0-3
3. **Wave explanation paragraphs:** Replaced "Wave 0 = all DTU" with per-wave DTU grouping and rationale
4. **Full Story List:** DTU stories reordered with [W0]/[W1]/[W2]/[W3] wave labels; 7 product story Depends On cells updated
5. **DTU blocks note:** Updated from "Test-Track Layer 1" model to Option 2 distributed model with full dependency chain verification
6. **Topological Order block:** Updated with (*) markers on stories now depending on DTU; TT-Layers updated to 4 tiers
7. **S-6.* namespace note:** Updated to list per-story wave assignments
8. **Scope Expansions table:** Added step5-option2 row documenting the change
9. **Changelog:** Added v1.29 row

---

## Final Wave Schedule — All 75 Stories

| Wave | Story | Crate | Title |
|------|-------|-------|-------|
| 0 | S-0.01 | devops | CI/CD Pipeline and Release Workflow |
| 0 | S-0.02 | devops | Developer Toolchain Bootstrap |
| 0 | S-6.06 | prism-dtu-common | DTU Common Infrastructure |
| 0 | S-6.14 | prism-dtu-threatintel | DTU for Threat Intel Aggregator — L2 (stateful) |
| 0 | S-6.15 | prism-dtu-nvd | DTU for NVD/NIST CVSS API — L2 (stateful) |
| 1 | S-1.01 | prism-core | Foundational Types |
| 1 | S-1.02 | prism-core | Entity Types and State Machines |
| 1 | S-1.03 | prism-core | Capability Resolution Engine |
| 1 | S-1.04 | prism-ocsf | OCSF Schema Loading and DynamicMessage |
| 1 | S-1.05 | prism-ocsf | OCSF Field Mapping and Normalization |
| 1 | S-1.06 | prism-credentials | Credential Store Trait and Backends |
| 1 | S-1.07 | prism-credentials | Credential CRUD, Resolution, and Security |
| 1 | S-1.08 | prism-security | Feature Flags (P0 Core) |
| 1 | S-1.09 | prism-security | Confirmation Tokens (P1) |
| 1 | S-1.10 | prism-security | Prompt Injection Defense |
| 1 | S-1.11 | prism-spec-engine | Spec Loading and Pipeline Execution |
| 1 | S-1.12 | prism-spec-engine | Hot Reload and Runtime Management |
| 1 | S-1.13 | prism-spec-engine | Sensor Spec Write Endpoints |
| 1 | S-1.14 | prism-spec-engine | Infusion Spec Loading and UDF Registration |
| 1 | S-1.15 | prism-spec-engine | WASM Plugin Runtime |
| 1 | S-6.07 | prism-dtu-crowdstrike | DTU for CrowdStrike Falcon API — L4 (adversarial) |
| 1 | S-6.08 | prism-dtu-claroty | DTU for Claroty xDome API — L4 (adversarial) |
| 1 | S-6.09 | prism-dtu-cyberint | DTU for Cyberint API — L2 (stateful) |
| 1 | S-6.10 | prism-dtu-armis | DTU for Armis Centrix API — L2 (stateful) |
| 2 | S-2.01 | prism-storage | RocksDB Initialization and Domain Operations |
| 2 | S-2.02 | prism-storage | Audit Buffer and Watchdog |
| 2 | S-2.03 | prism-storage | Decorators and Internal Tables |
| 2 | S-2.04 | prism-audit | Audit Entry Construction and Compliance |
| 2 | S-2.05 | prism-audit | Specialized Audit Events |
| 2 | S-2.06 | prism-sensors | DataSource Trait and Auth Patterns |
| 2 | S-2.07 | prism-sensors | Per-Sensor Auth and Pagination |
| 2 | S-2.08 | prism-sensors | Event Table Abstraction and Local Buffering |
| 2 | S-6.11 | prism-dtu-slack | DTU for Slack Webhook API — L2 (stateful) |
| 2 | S-6.12 | prism-dtu-pagerduty | DTU for PagerDuty Events API v2 — L3 (behavioral) |
| 2 | S-6.13 | prism-dtu-jira | DTU for Jira REST API v3 — L3 (behavioral) |
| 3 | S-3.01 | prism-query | PrismQL Parser (Filter + SQL + Pipe) |
| 3 | S-3.02 | prism-query | Query Tool and Materialization |
| 3 | S-3.03 | prism-query | Explain and Query Diagnostics |
| 3 | S-3.04 | prism-query | Alias System (P1) |
| 3 | S-3.05 | prism-query | Pagination and Caching |
| 3 | S-3.06 | prism-query | PrismQL Write Parser Extensions |
| 3 | S-3.07 | prism-query | Write Execution Pipeline |
| 3 | S-3.08 | prism-query | Hidden Columns |
| 3 | S-3.09 | prism-query | Query Performance Profiling |
| 3 | S-3.10 | prism-query | Cost Estimation (API Latency-Aware Planner) |
| 3 | S-3.11 | prism-query | In-Query Dedup Caching |
| 3 | S-3.12 | prism-query | Column Pruning and Field Selection Push-Down |
| 3 | S-3.13 | prism-query | Dynamic Table Availability |
| 3 | S-6.16 | prism-dtu-datadog | DTU for Datadog Logs API — L2 (stateful) |
| 3 | S-6.17 | prism-dtu-splunk-hec | DTU for Splunk HTTP Event Collector — L2 (stateful) |
| 3 | S-6.18 | prism-dtu-elasticsearch | DTU for Elasticsearch Bulk API — L2 (stateful) |
| 3 | S-6.19 | prism-dtu-otlp | DTU for OTLP/HTTP Log Ingestion — L2 (stateful) |
| 4 | S-4.01 | prism-operations | Schedule CRUD and Execution Loop |
| 4 | S-4.02 | prism-operations | Differential Results and Packs |
| 4 | S-4.03 | prism-operations | Detection Rule Loading and Compilation |
| 4 | S-4.04 | prism-operations | Detection Evaluation (Single/Correlation/Sequence) |
| 4 | S-4.05 | prism-operations | Alert Generation |
| 4 | S-4.06 | prism-operations | Case Management |
| 4 | S-4.07 | prism-operations | Case Metrics and Acknowledge Alert |
| 4 | S-4.08 | prism-operations | Action Delivery Framework |
| 5 | S-5.01 | prism-mcp | Server Bootstrap and Tool Registration |
| 5 | S-5.02 | prism-mcp | Tool Routing, Errors, and Client Scoping |
| 5 | S-5.03 | prism-mcp | Resources and Prompts |
| 5 | S-5.04 | prism-mcp | Sensor Health Subsystem |
| 5 | S-5.05 | prism-mcp | Config Loading and Validation |
| 5 | S-5.06 | prism-mcp | Action and Infusion MCP Tools |
| 5 | S-5.07 | prism-mcp | Multi-Repo Git Config Subscriptions |
| 5 | S-5.08 | prism-mcp | Diagnostics: prism logs CLI + get_diagnostics + Trace IDs |
| 5 | S-5.09 | prism-mcp | External Log Forwarding Subsystem |
| 5 | S-5.10 | prism-audit | Audit Trail External Forwarding |
| 6 | S-6.01 | prism-bin | CLI, Startup, and Initialization |
| 6 | S-6.02 | prism-bin | End-to-End Integration Smoke Tests |
| 6 | S-6.03 | prism-bin | Installation and Distribution |
| 6 | S-6.04 | prism-bin | prism credential CLI Subcommand Group |
| 6 | S-6.05 | prism-bin | prism migrate-storage CLI Command |

**Total: 75 stories across 7 waves (0-6)**

Wave counts: Wave 0: 5, Wave 1: 19, Wave 2: 11, Wave 3: 17, Wave 4: 8, Wave 5: 10, Wave 6: 5 = 75.

---

## Notes

- input-hash fields: NOT modified (left as null or [pending-recompute] per directive).
- No commits made per directive.
- S-6.06 (DTU common): wave unchanged at 0; no blocks changes (S-6.06 only blocks other DTU stories, not product stories).
