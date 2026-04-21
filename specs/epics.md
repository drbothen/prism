---
document_type: epic-registry
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-20T00:00:00Z
inputs:
  - ".factory/specs/prd.md"
  - ".factory/stories/STORY-INDEX.md"
input-hash: "572c2a9"
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
| E-3 | Query Engine & PrismQL | 3 | PrismQL parser, query materialization, EXPLAIN, alias system, pagination/caching, write parser, write execution, hidden columns, query profiling, cost estimation, in-query caching, column pruning, and dynamic table availability. | 13 | S-3.01, S-3.02, S-3.03, S-3.04, S-3.05, S-3.06, S-3.07, S-3.08, S-3.09, S-3.10, S-3.11, S-3.12, S-3.13 |
| E-4 | Scheduler, Detection, Cases | 4 | Schedule CRUD, diff/result packs, detection rules, detection evaluation, alert generation, case management, case metrics, and action delivery. | 8 | S-4.01, S-4.02, S-4.03, S-4.04, S-4.05, S-4.06, S-4.07, S-4.08 |
| E-5 | MCP Interface & Tool Surface | 5 | MCP bootstrap, tool routing, resources/prompts, sensor health, config loading, action infusion tools, multi-repo git config, diagnostics/logs CLI, external log forwarding, and audit trail forwarding. | 10 | S-5.01, S-5.02, S-5.03, S-5.04, S-5.05, S-5.06, S-5.07, S-5.08, S-5.09, S-5.10 |
| E-6 | DTU, Packaging, Integration Tests | 0–3, 6 [^e6] | CLI startup, E2E smoke tests, installation, credential CLI, storage migration, DTU common harness, and DTU clones for CrowdStrike, Claroty, Cyberint, Armis, Slack, PagerDuty, Jira, ThreatIntel, NVD, Datadog, Splunk HEC, Elasticsearch, and OTLP. | 19 | S-6.01, S-6.02, S-6.03, S-6.04, S-6.05, S-6.06, S-6.07, S-6.08, S-6.09, S-6.10, S-6.11, S-6.12, S-6.13, S-6.14, S-6.15, S-6.16, S-6.17, S-6.18, S-6.19 |

**Total stories: 75**

[^e6]: Epic E-6 spans two delivery tracks per the Option 2 DTU-first strategy. DTU clone stories (S-6.06–S-6.19) are distributed across waves 0–3 to precede their product consumers; packaging and ops stories (S-6.01–S-6.05) land in wave 6. Wave values on individual story frontmatter are authoritative. See STORY-INDEX.md and dtu-assessment.md Section 12.

## Changelog

| Version | Burst | Date | Author | Changes |
|---------|-------|------|--------|---------|
| 1.0 | pre-build-sweep | 2026-04-20 | product-owner | Initial epic registry created; backfills epic_id references across 75 stories (IMP-005). |
| 1.1 | pass-59-fix | 2026-04-20 | product-owner | Fixed MED-001: E-6 Wave column corrected from "6" to "0–3, 6" to reflect Option 2 DTU-first distribution (DTU clones S-6.06–S-6.19 in waves 0–3; packaging/ops S-6.01–S-6.05 in wave 6). Added footnote [^e6] explaining mixed-wave nature. |
