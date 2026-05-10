---
document_type: epic-registry
level: L3
version: "1.4"
status: draft
producer: product-owner
timestamp: 2026-05-04T00:00:00Z
inputs:
  - ".factory/specs/prd.md"
  - ".factory/stories/STORY-INDEX.md"
input-hash: "76729b7"
traces_to: [".factory/specs/prd.md"]
---

# Epic Registry — Prism

Epics are wave-aligned implementation groupings that organize stories into coherent delivery phases. Each epic corresponds to one implementation wave and collects all stories whose `epic_id` frontmatter matches the epic's ID. Wave ordering reflects technical dependency: each wave's outputs are preconditions for the next wave's work.

Story counts and membership are derived from `epic_id:` frontmatter values across `.factory/stories/S-*.md`.

## Epic Table

| Epic ID | Title | Wave | Description | Story Count | Stories |
|---------|-------|------|-------------|-------------|---------|
| E-0 | Foundation (CI/CD + Toolchain) | 0 | Repository scaffolding, CI/CD pipeline, developer toolchain, and build infrastructure. Precondition for all subsequent waves. | 2 | S-0.01, S-0.02 |
| E-1 | Core Primitives & Domain Types | 1 | Foundational Rust types, OCSF schema, credential store, feature flags, prompt-injection defense, hot-reload, sensor specs, and WASM runtime. | 15 | S-1.01, S-1.02, S-1.03, S-1.04, S-1.05, S-1.06, S-1.07, S-1.08, S-1.09, S-1.10, S-1.11, S-1.12, S-1.13, S-1.14, S-1.15 |
| E-2 | Storage & Sensor Foundation | 2 | RocksDB initialization, audit buffer, decorators, audit construction and events, datasource trait, per-sensor auth, and event tables. | 8 | S-2.01, S-2.02, S-2.03, S-2.04, S-2.05, S-2.06, S-2.07, S-2.08 |
| E-3 | Query Engine & PrismQL (core) | 3 | PrismQL parser, query materialization, EXPLAIN, alias system, pagination/caching, write parser, write execution, hidden columns, query profiling, cost estimation, in-query caching, column pruning, and dynamic table availability. | 13 | S-3.01, S-3.02, S-3.03, S-3.04, S-3.05, S-3.06, S-3.07, S-3.08, S-3.09, S-3.10, S-3.11, S-3.12, S-3.13 |
| E-3.0 | Query Engine Infrastructure Fixes | 3 | Pre-W3 infrastructure fixes: lefthook fmt-hook repair and DTU default-mode metadata registry (ADR-007 §2.3). | 2 | S-3.0.01, S-3.0.02 |
| E-3.1 | Multi-Tenant OrgId Foundation | 3 | OrgId(Uuid v7) newtype, tenant-id → org-slug rename, OrgRegistry, credentials/spec-engine/sensors OrgId boundary enforcement, audit OrgId fields, and ImplPhase adapter OrgId binding follow-on. | 8 | S-3.1.01, S-3.1.02, S-3.1.03, S-3.1.04, S-3.1.05, S-3.1.06, S-3.1.06-ImplPhase, S-3.1.07 |
| E-3.2 | Multi-Tenant Sensor State Segregation | 3 | Per-org state re-keying for Claroty, Armis, CrowdStrike, and Cyberint DTU crates; OrgId tagging for Slack, PagerDuty, and Jira DTU crates; CrowdStrike pagination session-ID org-scoping. | 8 | S-3.2.01, S-3.2.02, S-3.2.03, S-3.2.04, S-3.2.05, S-3.2.06, S-3.2.07, S-3.2.08 |
| E-3.3 | Multi-Tenant Config & Harness Isolation | 3 | Customer config TOML schema and parser, OrgRegistry boot integration, harness logical and network isolation, harness builder ergonomics, and reload-config mode-change rejection; includes 1 W3-FIX-* remediation story. | 7 | S-3.3.01, S-3.3.02, S-3.3.03, S-3.3.04, S-3.3.05, S-3.3.06, W3-FIX-SEC-004 |
| E-3.4 | DTU Harness Test Migration | 3 | Migrate existing per-sensor DTU test suites (Claroty, Armis, CrowdStrike, Cyberint, Slack/PagerDuty/Jira) into the shared prism-dtu-harness framework. | 5 | S-3.4.01, S-3.4.02, S-3.4.03, S-3.4.04, S-3.4.05 |
| E-3.5 | Workspace Convention Sweep | 3 | Workspace-wide src/ layout convention enforcement: check-crate-layout.sh script, CI gate, and CRATE-LAYOUT.md documentation; includes 14 W3-FIX-* remediation stories from W3 carry-forward debt. | 15 | S-3.5.01, W3-FIX-CI-001, W3-FIX-CODE-001, W3-FIX-CODE-002, W3-FIX-CODE-003, W3-FIX-CODE-004, W3-FIX-CODE-005, W3-FIX-CODE-006, W3-FIX-CREDS-001, W3-FIX-LEFTHOOK-001, W3-FIX-SEC-001, W3-FIX-SEC-002, W3-FIX-SEC-003, W3-FIX-SEC-005, W3-FIX-WIN-001 |
| E-3.6 | Holdout Scenario Refresh | 3 | Re-anchor Wave 3 holdout scenarios HS-006 (multi-tenant state recovery) and HS-007 (multi-tenant cross-repo failure) to current Wave 3 BCs. | 2 | S-3.6.01, S-3.6.02 |
| E-3.7 | Schema Derivation & Fixture Generators | 3 | Derive Rust types from Armis (armis-sdk-go) and CrowdStrike (gofalcon) schemas; archetype catalog and generator options; per-sensor fixture generators for Claroty, Cyberint, Armis, and CrowdStrike. | 6 | S-3.7.00, S-3.7.01, S-3.7.02, S-3.7.03, S-3.7.04, S-3.7.05 |
| E-4 | Scheduler, Detection, Cases | 4 | Schedule CRUD, diff/result packs, detection rules, detection evaluation, alert generation, case management, case metrics, and action delivery. | 8 | S-4.01, S-4.02, S-4.03, S-4.04, S-4.05, S-4.06, S-4.07, S-4.08 |
| E-5 | MCP Interface & Tool Surface | 5 | MCP bootstrap, tool routing, resources/prompts, sensor health, config loading, action infusion tools, multi-repo git config, diagnostics/logs CLI, external log forwarding, and audit trail forwarding. | 10 | S-5.01, S-5.02, S-5.03, S-5.04, S-5.05, S-5.06, S-5.07, S-5.08, S-5.09, S-5.10 |
| E-6 | DTU, Packaging, Integration Tests | 0–3, 6 [^e6] | CLI startup, E2E smoke tests, installation, credential CLI, storage migration, DTU common harness, and DTU clones for CrowdStrike, Claroty, Cyberint, Armis, Slack, PagerDuty, Jira, ThreatIntel, NVD, Datadog, Splunk HEC, Elasticsearch, OTLP, and Unified Multi-Clone Demo Harness (S-6.20). | 20 | S-6.01, S-6.02, S-6.03, S-6.04, S-6.05, S-6.06, S-6.07, S-6.08, S-6.09, S-6.10, S-6.11, S-6.12, S-6.13, S-6.14, S-6.15, S-6.16, S-6.17, S-6.18, S-6.19, S-6.20 |

**Total stories: 129**

[^e6]: Epic E-6 spans two delivery tracks per the Option 2 DTU-first strategy. DTU clone stories (S-6.06–S-6.19) are distributed across waves 0–3 to precede their product consumers; packaging and ops stories (S-6.01–S-6.05) land in wave 6. S-6.20 (Unified Multi-Clone DTU Demo Harness) was merged in Wave 1 (PR #29, db550cec). Wave values on individual story frontmatter are authoritative. See STORY-INDEX.md and dtu-assessment.md Section 12.

## Changelog

| Version | Burst | Date | Author | Changes |
|---------|-------|------|--------|---------|
| 1.4 | wave-4-phase-4a-converged | 2026-05-04 | product-owner | W3-FIX-* story family added: 14 stories assigned to E-3.5 (W3-FIX-CI-001, W3-FIX-CODE-001/002/003/004/005/006, W3-FIX-CREDS-001, W3-FIX-LEFTHOOK-001, W3-FIX-SEC-001/002/003/005, W3-FIX-WIN-001) and 1 story assigned to E-3.3 (W3-FIX-SEC-004). E-3.3 Story Count 6→7. E-3.5 Story Count 1→15. Total stories 114→129. Prior v1.3 scan missed W3-FIX-* family because it scanned only S-N.NN pattern files; filesystem grep confirmed 15 W3-FIX-* stories with valid epic_id frontmatter. |
| 1.3 | wave-4-phase-4a-converged | 2026-05-04 | product-owner | Ground-truth refresh from filesystem: E-3 expanded from 13 to 51 stories during W3 execution via eight new sub-epics (E-3.0 through E-3.7) covering multi-tenant OrgId foundation, sensor state segregation, config/harness isolation, DTU harness test migration, workspace convention sweep, holdout scenario refresh, and schema derivation/fixture generators. Anomaly: two files share the S-3.1.06 slug (original S-3.1.06 sensor boundary story + follow-on S-3.1.06-ImplPhase adapter binding story); both counted under E-3.1. Total stories 76 → 114. SESSION-HANDOFF cited 129 — actual filesystem count is 114. |
| 1.2 | pre-wave-2-audit | 2026-04-24 | state-manager | Add S-6.20 to E-6 row; Story Count 19 → 20; Total stories 75 → 76; update E-6 description to include Unified Multi-Clone Demo Harness; update footnote [^e6] with S-6.20 Wave 1 merge info. Pre-Wave-2 audit M-002 remediation (ebf7c63c). |
| 1.1 | pass-59-fix | 2026-04-20 | product-owner | Fixed MED-001: E-6 Wave column corrected from "6" to "0–3, 6" to reflect Option 2 DTU-first distribution (DTU clones S-6.06–S-6.19 in waves 0–3; packaging/ops S-6.01–S-6.05 in wave 6). Added footnote [^e6] explaining mixed-wave nature. |
| 1.0 | pre-build-sweep | 2026-04-20 | product-owner | Initial epic registry created; backfills epic_id references across 75 stories (IMP-005). |
