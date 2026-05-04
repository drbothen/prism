---
document_type: architecture-index
level: L3
version: "2.24"
status: draft
producer: architect
timestamp: 2026-05-04T00:00:00
phase: 1b
inputs: [domain-spec/L2-INDEX.md, prd.md, prd-supplements/interface-definitions.md, prd-supplements/nfr-catalog.md, prd-supplements/error-taxonomy.md]
traces_to: prd.md
deployment_topology: single-service
---

# Architecture Index: Prism

> **Context Engineering:** This is a lightweight index (~400 tokens). Agents load
> ONLY the section files they need, not the full architecture. See the Document Map
> for per-section consumer guidance.

## Document Map

| Section | File | Tokens | Primary Consumer | Purpose |
|---------|------|--------|-----------------|---------|
| System Overview | system-overview.md | ~1,000 | orchestrator, all agents | Architecture vision, principles, deployment model, constraints |
| Module Decomposition | module-decomposition.md | ~1,200 | story-writer, implementer | Crate catalog with boundaries, responsibilities, public APIs — v1.13 (3 ActionEngine sites cleaned per F-PreP21-H-001) |
| Dependency Graph | dependency-graph.md | ~800 | story-writer, consistency-validator | Inter-crate dependencies, topological build order |
| API Surface | api-surface.md | ~1,000 | test-writer, implementer | MCP tool registry, error contract, resource/prompt surface — v1.6 (ActionEngine→ActionDeliveryEngine per F-PreP21-H-001) |
| Data Layer | data-layer.md | ~1,000 | implementer, test-writer | RocksDB domains, Arrow materialization, caching strategy — v1.3 (17 CFs, D-209 concurrency, ADR-016 §2.5 retry key) |
| Query Engine | query-engine.md | ~1,200 | implementer, test-writer | PrismQL parser, DataFusion integration, fan-out pipeline — v1.2 (D-209 8/8 + memory math corrected per F-PreP24-H-002) |
| Sensor Adapters | sensor-adapters.md | ~1,000 | implementer, test-writer | Config-driven TOML specs, CustomAdapter escape hatch |
| Security Architecture | security-architecture.md | ~1,000 | security-reviewer, implementer | Credentials, feature flags, audit, prompt injection defense |
| Operational Pipeline | operational-pipeline.md | ~1,000 | implementer, test-writer | Scheduler, differential results, detection, alerts, cases — v1.2 (D-209 8/8 + 60s tick + ActionDeliveryEngine per F-P23-H-001) |
| Concurrency Architecture | concurrency-architecture.md | ~800 | implementer, formal-verifier | Tokio runtime, arc-swap, shared state protection — v1.1 (Mermaid diagram + 6 edits; 16-permit→8/8 split per D-209) |
| Purity Boundary Map | purity-boundary-map.md | ~800 | implementer, formal-verifier | Pure core / effectful shell classification per crate |
| Verification Architecture | verification-architecture.md | ~1,000 | formal-verifier, architect | Provable Properties Catalog, proof strategy — v1.28 (Mermaid P13 + 145 VPs per F-PreP21-H-001 + Pass 13 F-P13-H-002) |
| Tooling Selection | tooling-selection.md | ~400 | formal-verifier, devops-engineer | Kani, proptest, fuzz tool versions and config |
| Detection Rule Format | detection-rule-format.md | ~1,200 | implementer, test-writer | .detect rule structure, condition modes, rule-to-SQL compilation |
| Infusions | infusions.md | ~1,500 | implementer, test-writer | Enrichment framework — GeoIP, threat intel, asset inventory, CVSS. TOML specs + .prx plugins. |
| Actions | actions.md | ~1,500 | implementer, test-writer | Alert delivery + scheduled reports — Slack, PagerDuty, Jira, email, syslog. TOML specs + .prx plugins. — v1.3 (Mermaid display labels Action Engine→ActionDeliveryEngine per F-P23-H-002) |
| Installation | installation.md | ~1,500 | devops-engineer, implementer | Distribution channels, CLI commands, secops-factory integration, first-run UX |
| Config Schema | config-schema.md | ~1,500 | implementer, devops-engineer | Full prism.toml schema, aliases.toml, env var overrides, validation tiers, config diff tool |
| Observability | observability.md | ~2,000 | implementer, devops-engineer | 18 diagnostic log targets, per-subsystem levels, trace IDs, `prism logs` CLI, `get_diagnostics` tool, external log forwarding (Datadog/Splunk/Elastic/OTLP/plugin) — v1.1 (debug log + JSON examples updated with 8/8 split per D-209) |
| Verification Coverage | verification-coverage-matrix.md | ~600 | consistency-validator | VP-to-module coverage mapping |
| Write Operations | write-operations.md | ~2,000 | implementer, test-writer, security-reviewer | AD-022: PrismQL write extensions — pipe verbs, SQL DML, safety integration, sensor spec schema, error codes |
| DTU Assessment | dtu-assessment.md | ~2,000 | story-writer, test-writer, devops-engineer | Behavioral clone assessment: per-sensor scope matrix, fidelity levels, delivery model, VP-033/VP-036 integration |

## Cross-References

| If you need... | Read these together |
|----------------|-------------------|
| Implementation plan for a module | module-decomposition.md + dependency-graph.md + api-surface.md |
| Verification plan for a module | verification-architecture.md + purity-boundary-map.md + tooling-selection.md |
| Full module picture | module-decomposition.md + purity-boundary-map.md + verification-coverage-matrix.md |
| Story decomposition input | module-decomposition.md + dependency-graph.md |
| Query pipeline understanding | query-engine.md + sensor-adapters.md + data-layer.md |
| Security review | security-architecture.md + purity-boundary-map.md |
| Operational features | operational-pipeline.md + data-layer.md |
| Write operation design | write-operations.md + security-architecture.md + sensor-adapters.md |
| Integration test infrastructure | dtu-assessment.md + verification-architecture.md + tooling-selection.md |

## ADR Registry

> **Wave 3 namespace note (2026-04-27):** ADR-006 through ADR-012 were authored during
> Wave 3 Phase 3.A and are registered here. ADR-005 was authored in Wave 2 and
> retroactively added in the same pass.

| ID | Title | Status | Date | File |
|----|-------|--------|------|------|
| ADR-001 | DTU Rate Limit Pattern | ACCEPTED | 2026-04-22 | decisions/ADR-001-dtu-rate-limit-pattern.md |
| ADR-002 | L2 DTU Clone Template | ACCEPTED | 2026-04-22 | decisions/ADR-002-l2-dtu-clone-template.md |
| ADR-003 | DTU Reset Lookup and Fidelity Auth | ACCEPTED | 2026-04-22 | decisions/ADR-003-dtu-reset-lookup-and-fidelity-auth.md |
| ADR-004 | Kani Arbitrary Policy — Which Types Carry kani::Arbitrary | PROPOSED | 2026-04-26 | decisions/ADR-004-kani-arbitrary-policy.md |
| ADR-005 | AQL Injection Mitigation — Armis Adapter Query Trust Model | ACCEPTED | 2026-04-26 | decisions/ADR-005-aql-injection-mitigation.md |
| ADR-006 | Multi-Tenant DTU Topology — OrgId/OrgSlug Identity, OrgRegistry, Configurable Shared/Client Mode | ACCEPTED | 2026-04-27 | decisions/ADR-006-multi-tenant-dtu-topology.md |
| ADR-007 | Configurable Shared/Client DTU Mode — Per-Type Default Registry, Config Schema, and Isolation Semantics | ACCEPTED | 2026-04-27 | decisions/ADR-007-configurable-dtu-mode.md |
| ADR-008 | DTU State Segregation — HashMap<(OrgId, String), V> Keying Pattern, Per-Tenant Lock Granularity, and Reset Semantics | ACCEPTED | 2026-04-27 | decisions/ADR-008-dtu-state-segregation.md |
| ADR-009 | Multi-Tenant Data Generator — Hybrid Archetype Catalog + Deterministic Generator | ACCEPTED | 2026-04-27 | decisions/ADR-009-multi-tenant-data-generator.md |
| ADR-010 | Customer Config Schema — customers/{org_slug}.toml Structure, Validation Rules, Loading Lifecycle, and Schema Versioning | ACCEPTED | 2026-04-27 | decisions/ADR-010-customer-config-schema.md |
| ADR-011 | DTU Harness Isolation Modes — Logical (In-Process) and Network (Per-Port) | ACCEPTED | 2026-04-27 | decisions/ADR-011-harness-isolation-modes.md |
| ADR-012 | Workspace src/ Convention Normalization — Canonical Crate Layout | ACCEPTED | 2026-04-27 | decisions/ADR-012-src-convention.md |
| ADR-013 | Schedule Execution Semantics | PROPOSED v0.7 | 2026-05-03 | decisions/ADR-013-schedule-execution-semantics.md |
| ADR-015 | Detection Rule Language | PROPOSED v0.6 | 2026-05-03 | decisions/ADR-015-detection-rule-language.md |
| ADR-016 | Action Delivery Framework | PROPOSED v0.12 | 2026-05-03 | decisions/ADR-016-action-delivery-framework.md |
| ADR-017 | Case Lifecycle Invariants | PROPOSED v0.7 | 2026-05-03 | decisions/ADR-017-case-lifecycle-invariants.md |
| ADR-018 | Differential Result Pack Format | PROPOSED v0.6 | 2026-05-03 | decisions/ADR-018-differential-result-pack-format.md |
| ADR-019 | SIEM Output Formats | PROPOSED v0.4 | 2026-05-03 | decisions/ADR-019-siem-output-formats.md |

## Architecture Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| AD-001 | Modular monolith via Cargo workspace; current workspace has 22 member crates (11 non-DTU production/build-helper crates: prism-core, prism-credentials, prism-mcp, prism-ocsf, prism-security, prism-spec-engine, prism-sensors, prism-storage, prism-audit, prism-query, ocsf-proto-gen; 11 DTU test-only crates: prism-dtu-common plus 10 per-surface clones). Remaining Phase-1 production crates (prism-bin, prism-operations) are targeted for future waves. Plus prism-dtu-harness planned in Wave 3 per ADR-011, bringing total to 23 crates at end of Wave 3. | Single binary deployment matches per-analyst stdio model; crate boundaries enforce module isolation without network overhead |
| AD-002 | DataFusion as SQL execution engine | Provides Arrow-native SQL with UDF extensibility; ephemeral SessionContext per query aligns with data-in-flight model |
| AD-003 | Chumsky 0.12 for PrismQL parsing | Zero-copy parser combinators with error recovery; axiathon reference proves pattern viability |
| AD-004 | RocksDB with 17 column families | Domain-isolated persistence for operational state; osquery-proven pattern; single-process LOCK fits stdio model. CFs: default, schedules, diff_results, detection_rules, detection_state, alerts, cases, audit_buffer, dirty_bits, watchdog, aliases, decorators, action_state, infusion_cache, plugin_state, event_buffer, case_dedup_idx. |
| AD-005 | rmcp 1.4 as MCP SDK | Official Anthropic SDK; #[tool_router] macro for 35+ tool registration; native tokio async |
| AD-006 | Config-driven sensor adapters via TOML spec files | 80% of sensors need zero Rust code; eat-our-own-dog-food principle for built-in sensors |
| AD-007 | arc-swap for hot config reload | Lock-free reads on query hot path; atomic snapshot swap; in-flight queries unaffected |
| AD-008 | Pure core / effectful shell separation | Maximizes formal verification surface; domain logic testable without I/O mocking |
| AD-009 | Sealed trait pattern for SensorAuth | Prevents cross-sensor auth composition at compile time; type-level safety |
| AD-010 | ~~TenantId newtype for client isolation~~ → **superseded by ADR-006**: OrgId (UUID v7 newtype) + OrgSlug (kebab-case string) replace the single TenantId concept; OrgRegistry provides bijective resolution; compile-time enforcement via OrgId newtype prevents accidental cross-org leakage | See ADR-006 for the canonical OrgId/OrgSlug split decision |
| AD-011 | Two-tier feature flag system (compile-time + runtime) | Compile-time gates remove code from binary; runtime gates enable per-client control; defense in depth |
| AD-012 | Bincode for RocksDB value serialization | Compact binary encoding; schema evolution via versioned keys; faster than JSON for structured data |
| AD-013 | tokio multi-threaded runtime | Required for concurrent sensor fan-out; DataFusion uses tokio internally; rmcp requires tokio |
| AD-014 | Process-level RSS watchdog with self-SIGTERM | Last-resort memory protection; graceful shutdown path preserves state integrity |
| AD-015 | DynamicMessage protobuf for OCSF normalization | Runtime-flexible field mapping without per-class codegen; axiathon-proven pattern |
| AD-016 | Write-audit ordering (intent-log pattern) | Durable audit trace for every write operation, even across crashes |
| AD-017 | AI-opaque credential management | Credential values never transit through AI context; reference-based model |
| AD-018 | Automatic filesystem watching for config reload | `notify` crate monitors config/spec/IOC/plugin directories; debounced 500ms; same validation as manual reload |
| AD-019 | WASM plugins for custom sensor adapters and infusions | Polyglot (Rust/Go/Python/JS/C#), sandboxed, hot-reloadable via `.prx` files. `wasmtime` runtime with WIT interface. Augments TOML specs, doesn't replace them. |
| AD-020 | Infusions — composable enrichment framework | GeoIP, threat intel, asset inventory, CVSS as TOML specs + `.prx` plugins. Register as DataFusion UDFs and `enrich` pipe stages. Same two-tier pattern as sensors. |
| AD-021 | Actions — config-driven alert delivery and reporting | Slack, PagerDuty, Jira, email, syslog, custom webhooks as TOML specs + `.prx` plugins. Three triggers: alert, schedule, manual. At-least-once delivery with retry. |
| AD-022 | PrismQL Write Operations | Pipe mode terminal action verbs + SQL DML (INSERT/UPDATE/DELETE) targeting sensor write endpoints. All writes route through feature flags, risk-tier gates, dry-run/confirmation system, and intent-log audit. Filter mode remains read-only. Internal tables write-protected via PrismQL. |

## Subsystem Registry

| SS ID | Name | Architecture Doc | Crate(s) | Phase Introduced |
|-------|------|-----------------|----------|-----------------|
| SS-01 | Sensor Adapters | sensor-adapters.md | prism-sensors, prism-spec-engine, prism-dtu-common, prism-dtu-claroty, prism-dtu-armis, prism-dtu-crowdstrike, prism-dtu-cyberint, prism-dtu-slack, prism-dtu-pagerduty, prism-dtu-jira, prism-dtu-nvd, prism-dtu-threatintel, prism-dtu-demo-server, prism-dtu-harness *(planned per ADR-011)* | Phase 1 |
| SS-02 | OCSF Normalization | system-overview.md | prism-ocsf | Phase 1 |
| SS-03 | Credential Management | security-architecture.md | prism-credentials | Phase 1 |
| SS-04 | Feature Flags | security-architecture.md | prism-security | Phase 1 |
| SS-05 | Audit Trail | operational-pipeline.md | prism-audit | Phase 1 |
| SS-06 | Client Configuration | config-schema.md | prism-spec-engine, prism-mcp | Phase 1 |
| SS-07 | Adapter Pagination & Response Cache | query-engine.md | prism-query | Phase 1 |
| SS-08 | Sensor Health | api-surface.md § Sensor Health, operational-pipeline.md | prism-mcp, prism-sensors | Phase 1 |
| SS-09 | Prompt Injection Defense | security-architecture.md | prism-security | Phase 1 |
| SS-10 | MCP Interface | api-surface.md | prism-mcp | Phase 1 |
| SS-11 | Query Execution | query-engine.md | prism-query | Phase 1 |
| SS-12 | Scheduler | operational-pipeline.md | prism-operations | Phase 1 |
| SS-13 | Detection Engine | operational-pipeline.md, detection-rule-format.md | prism-operations | Phase 1 |
| SS-14 | Alert & Case Management | operational-pipeline.md | prism-operations | Phase 1 |
| SS-15 | Storage Layer | data-layer.md | prism-storage | Phase 1 |
| SS-16 | Spec Engine | sensor-adapters.md §Tier 1, query-engine.md | prism-spec-engine | Phase 1 |
| SS-17 | WASM Plugin Runtime | sensor-adapters.md §Tier 2 (AD-019) | prism-spec-engine | Phase 3 |
| SS-18 | Action Delivery Engine | actions.md (AD-021) | prism-operations, prism-siem-formats | Phase 3 |
| SS-19 | Infusion Enrichment Framework | infusions.md (AD-020) | prism-spec-engine | Phase 3 |
| SS-20 | Observability / Log Forwarding | observability.md | prism-mcp | Phase 3 |
| SS-21 | Identity & Core Types | system-overview.md, module-decomposition.md | prism-core | Phase 3 |

## Changelog

| Version | Pass | Date | Author | Change |
|---------|------|------|--------|--------|
| 2.24 | W4-Phase4A-Pass24-fix | 2026-05-04 | state-manager | F-P24-CRIT-001: prd.md v1.8→v1.9 (PRD §2 line 389 BC-2.18.004 cell title sync to BC H1 — "Scheduled Report Queries — try_acquire() on 16-Permit Semaphore" → "Action Delivery Semaphore — 8-Permit Independent Pool"; comprehensive TD-VSDD-049 sweep across 200 PRD §2 BC rows found ONLY this drift; SUBSTANTIVE). TD-VSDD-049 filed. ARCH-INDEX version 2.23→2.24. Pass 25 next (window stays 0/3). |
| 2.23 | W4-Phase4A-PrePass24-Sweep | 2026-05-04 | state-manager | F-PreP24-CRIT-001: prd.md v1.7→v1.8 (INV-ACTION-004 root contract "shared 16-permit semaphore" contradicts D-209 LOCKED — PRD root contract corrected). F-PreP24-H-001: interface-definitions.md v2.5→v2.6 (6 sites Subsystem 18 "Action Engine" label corrected to "Action Delivery Engine"). F-PreP24-H-002: query-engine.md v1.1→v1.2 (16 concurrent→8 concurrent + 3.2 GB→1.6 GB memory math). Document Map rows updated: query-engine v1.2; module-decomposition v1.13 (missing annotation added); api-surface v1.6 (missing annotation added); verification-architecture v1.28 (missing annotation added). ARCH-INDEX version 2.22→2.23. TD-VSDD-048 methodology applied. |
| 2.22 | W4-Phase4A-Pass23-fix | 2026-05-04 | state-manager | F-P23-H-001: operational-pipeline.md v1.1→v1.2 (3 stale refs fixed: 16-permit + Action Engine + 60s tick; missed by Pre-Pass-21 hand-curated sweep target list). F-P23-H-002: actions.md v1.2→v1.3 (Mermaid participant display labels Action Engine→ActionDeliveryEngine claim-vs-reality drift). Document Map rows updated. ARCH-INDEX version 2.21→2.22. TD-VSDD-048 filed. |
| 2.21 | W4-Phase4A-Pass22-fix | 2026-05-03 | state-manager | F-P22-L-001: Document Map line 39 actions.md annotation updated to v1.2 (D-209 8/8 split + 60s tick + ActionDeliveryEngine + ADR-016 §2.5 CF table per F-P22-H-001). ARCH-INDEX version 2.20→2.21. |
| 2.20 | W4-Phase4A-PrePass22-BroadSweep | 2026-05-03 | state-manager | F-PreP22-H-001: concurrency-architecture.md v1.0→v1.1 (Mermaid + 6 edits; 16-permit→8/8 split per D-209). F-PreP22-H-002: observability.md v1.0→v1.1 (debug log + JSON user-facing examples updated to 8/8 per D-209). F-PreP22-H-003: interface-definitions.md v2.4→v2.5 (ActionEngine→ActionDeliveryEngine). F-PreP22-H-004: vp-045-schedule-semaphore-try-acquire-nonblocking.md v1.1→v1.2 (full body rewrite + slug-preservation banner per POL-1). Document Map rows updated: concurrency-architecture v1.1, observability v1.1. Window stays 0/3; Pass 22 dispatch ready. |
| 2.19 | W4-Phase4A-Pass21-fix | 2026-05-03 | state-manager | Pass 21 BLOCKED → REMEDIATED: data-layer.md v1.2→v1.3 (F-P21-H-001 concurrency claim "16 scheduled" stale → D-209 8/8+2 ad-hoc per-subsystem; F-P21-H-002 CF count 16→17 + case_dedup_idx row added per P5-XADR-A-M-006; F-P21-M-001 retry CF key canonical `{org_id}:\x04:{action_id}:{idempotency_key}` per ADR-016 §2.5). All 3 findings SUBSTANTIVE. Window stays 0/3; Pass 22 next. |
| 2.18 | W4-Phase4A-PrePass21-BroadSweep | 2026-05-03 | state-manager | F-PreP21-H-001: actions.md v1.0→v1.1 (16-permit→8-permit per D-209; 1-second→60s default per ADR-013 §2.1; ActionEngine→ActionDeliveryEngine); module-decomposition.md v1.12→v1.13 (3 ActionEngine sites); api-surface.md v1.5→v1.6 (1 site); data-layer.md v1.1→v1.2 (1 site); verification-architecture.md v1.27→v1.28 (P13 Mermaid node label). Foundation arch docs cleaned; D-209 + ADR-013 §2.1 + ADR-016 §1.1 propagated. F-PreP21-H-002: BC-2.18.003/008 v1.4 ActionEngine→ActionDeliveryEngine. |
| 2.17 | W4-Phase4A-Pass20-fix | 2026-05-03 | state-manager | F-P20-H-002 capture: ADR-016 v0.11→v0.12 (VP-045+VP-047 priority P1→P0 sync to VP-INDEX SoT per POL-9; architect burst). |
| 2.16 | W4-Phase4A-Pass18-burst | 2026-05-03 | state-manager | Pass 18 CLEAN (window 1/3 OPEN; FINDINGS_REMAIN): ADR-016 v0.10→v0.11 (F-P18-M-001/M-002 remediation-notes table header + stale-narrative fixes); ADR-017 v0.6→v0.7 (same). |
| 2.15 | W4-Phase4A-PrePass18-Sweep-2 | 2026-05-03 | state-manager | F-PreP18-H-001 architect-burst capture: ADR-016 v0.9→v0.10 (Status H2 synced v0.8→v0.10; sister-line regression class per F-P16-H-002); ADR-017 v0.5→v0.6 (Status H2 synced v0.4→v0.6; same class). Ready for Pass 18. |
| 2.14 | W4-Phase4A-Pass17-fix | 2026-05-03 | state-manager | Pass 17 BLOCKED → REMEDIATED: ADR-016 v0.8→v0.9 (F-P17-M-001 frontmatter date 2026-05-02→2026-05-03; COSMETIC); ADR-017 v0.4→v0.5 (F-P17-M-001 frontmatter date 2026-05-02→2026-05-03; COSMETIC). |
| 2.13 | W4-Phase4A-Pass16-fix | 2026-05-03 | state-manager | Pass 16 BLOCKED → REMEDIATED: ADR-015 v0.5→v0.6 (F-P16-H-002 Status H2 sync); ADR-016 v0.7→v0.8 (F-P16-M-001 §5.5 VP-143 anchor corrected to S-4.08 only); ADR-018 v0.5→v0.6 (F-P16-H-002 Status H2 sync). |
| 2.12 | W4-Phase4A-Pass14-fix | 2026-05-03 | state-manager | Pass 14 BLOCKED → REMEDIATED: ADR-013 v0.6→v0.7 (F-P14-M-001 enum tuple form + DashMap key-resolution paragraph; F-P14-M-002 producer attribution; F-P14-L-002 Status H2 sync). ADR-015 v0.4→v0.5 (F-P14-M-001 cascade: 5 enum variant sites). ADR-018 v0.4→v0.5 (F-P14-M-001 cascade: 3 enum variant sites). |
| 2.11 | W4-PrePass14-Sweep | 2026-05-03 | state-manager | Pre-Pass-14 sweep (TD-VSDD-039 codified methodology): F-PreP14-H-003 — ADR-017 v0.3→v0.4 (sister-section partial-fix regression: stale `case:{org_id}:` body prose at lines ~230 §3.5 and ~282 Rationale updated to canonical `{org_id}:case:{client_id}:{case_id}` per §3.4). |
| 2.10 | F-P13-M-002 | 2026-05-03 | state-manager | Pass 13 MEDIUM: ADR-013 row updated v0.5 → v0.6 (date sync to 2026-05-03 per architect F-P13-L-001 burst; ARCH-INDEX date was already 2026-05-03). |
| 2.9 | W4-Phase4A-D214-Sweep | 2026-05-03 | state-manager | D-214 Component 1 proactive structural sweep: ADR-019 v0.3→v0.4 (F-PSweep-H-001: missing ## Status H2 section added). |
| 2.8 | W4-Phase4A-Pass10-fix | 2026-05-03 | state-manager | Wave 4 Phase 4.A Pass 10 fix-burst: ADR-016 ADR Registry version propagated v0.5→v0.7 (catches both Pass 9 and Pass 10 ADR-016 bumps; F-P10-H-001 partial-fix regression). |
| 2.7 | W4-Phase4A-Pass9-fix | 2026-05-03 | state-manager | P9 fix-burst: changelog row order normalized (rows were non-monotonic: 2.1, 2.6, 2.5, 2.3, 2.2, 2.0 → reordered descending 2.6, 2.5, 2.3, 2.2, 2.1, 2.0). |
| 2.6 | W4-Phase4A-Pass8-fix | 2026-05-03 | state-manager | Pass 8 remediation: ADR-013 upgraded PROPOSED v0.4→v0.5 (§2.1 croner 2.0→2.1 per research R-2); ADR-016 upgraded PROPOSED v0.4→v0.5 (§5.5 120s→60s retry scanner tick; retry-state row \x04 + dead-letter row \x03 discriminators per BC-2.18.001 H-002 alignment). |
| 2.5 | W4-Phase4A-Pass5-fix | 2026-05-03 | state-manager | P5-XADR-A-M-006: AD-004 amended — 16→17 column families; added case_dedup_idx (per S-4.06 Task 9b auto-case-dedup secondary index). |
| 2.3 | W4-Phase4A-Pass3-fix | 2026-05-02 | state-manager | Pass 3 remediation: ADR-013/015/016/018 upgraded PROPOSED v0.3 → v0.4 in ADR Registry (CF key prefix order, global rule detection_state key, next_run_at lag annotation, manual-trigger dedup contradiction); ADR-019 upgraded v0.2 → v0.3 (§10→§2.10 mis-anchor corrected). |
| 2.2 | W4-Phase4A-Pass2-fix | 2026-05-02 | state-manager | Pass 2 remediation: ADR-013/015/016/017/018 upgraded PROPOSED v0.2 → v0.3 in ADR Registry (idempotency_key canon, timeline_entry_id, splay best-effort, Created invalidation scope, auth order, WIT note, pack scope, cron 6-field, case_dedup race). ADR-019 unchanged at v0.2. |
| 2.1 | W4-Phase4A-Pass1-fix | 2026-05-02 | state-manager | Pass 1 remediation: all 6 Wave 4 ADRs upgraded PROPOSED v0.1 → v0.2 in ADR Registry (subsystem corrections, UNION merge model, UDF Volatility=Immutable, retry reconcile, manual trigger fire-and-forget, INV-CASE-006). SS-18 crate column updated to add prism-siem-formats (per ADR-019 §9 task). |
| 2.0 | W4-ADR-Phase3-burst | 2026-05-02 | state-manager | Wave 4 Phase 3 ADR burst: ADR-016 (Action Delivery Framework) + ADR-019 (SIEM Output Formats) PROPOSED v0.1 registered in ADR Registry table. ALL 6 Wave 4 ADRs now PROPOSED v0.1. |
| 1.9 | W4-ADR-burst | 2026-05-02 | state-manager | Wave 4 Phase 2 ADR burst: ADR-015 (Detection Rule Language) + ADR-018 (Differential Result Pack Format) PROPOSED v0.1 registered in ADR Registry table. |
| 1.8 | pass-18-remediation | 2026-04-27 | product-owner | M-18-002: SS-01 crates column — annotated `prism-dtu-harness` as *(planned per ADR-011)*. AD-001 narrative updated to clarify harness is planned, total becomes 23 crates at end of Wave 3. m-18-001: ADR Registry Status column uppercased to match canonical ADR frontmatter (PROPOSED/ACCEPTED per POL 7 source-of-truth). |
| 1.7 | pass-17-remediation | 2026-04-27 | product-owner | m-17-003: SS-21 Phase Introduced corrected "Phase 1" → "Phase 3". SS-21 was created in Wave 3 (Phase 3.A) per D-095, consistent with SS-17/18/19/20 which all say "Phase 3". |
| 1.6 | pass-13-remediation | 2026-04-27 | product-owner | M-001: AD-001 updated — crate count corrected to 22 (11 non-DTU production/build-helper + 11 DTU test-only), stale "16 member crates (8+8)" replaced. M-002: SS-21 "Identity & Core Types" added for prism-core (OrgId/OrgRegistry implementation site per D-047); CAP-038 anchored to SS-21 in capabilities.md. M-003+m-001+m-002: all 7 Wave 3 ADR Status blocks and §6/§7 preambles updated (BCs authored; OQ RESOLVED annotations added). |
| 1.5 | pass-11-remediation | 2026-04-27 | product-owner | M-003/M-004 fix: SS-01 Crate(s) column actually updated to include all prism-dtu-* crates (pass-10 v1.4 changelog claimed this change but never applied it to the table row). v1.4 changelog entry retained for historical record. Frontmatter version bumped 1.3→1.5 (skipping 1.4 since v1.4 body change never landed). |
| 1.4 | pass-10-remediation | 2026-04-27 | product-owner | OBS-001: SS-01 Crate(s) column expanded to include all prism-dtu-* crates per CAP-036 (DTU test harness) and CAP-037 (workspace conventions) ownership. Added: prism-dtu-common, prism-dtu-claroty, prism-dtu-armis, prism-dtu-crowdstrike, prism-dtu-cyberint, prism-dtu-slack, prism-dtu-pagerduty, prism-dtu-jira, prism-dtu-nvd, prism-dtu-threatintel, prism-dtu-demo-server, prism-dtu-harness. CAP-036 §SS-01 was already the declared owner; ARCH-INDEX row was simply incomplete. |
| 1.3 | Pass1-W3 | 2026-04-27 | product-owner | C-004 fix: ADR Registry expanded — added ADR-005 (Wave 2 retroactive), ADR-006 through ADR-012 (Wave 3 Phase 3.A). Added Wave 3 namespace note. AD-010 entry updated: TenantId concept superseded by ADR-006 OrgId/OrgSlug split. |
| 1.2 | W2-P2 | 2026-04-26 | state-manager (via architect decision) | Added ADR Registry table; registered ADR-004 stub (Kani Arbitrary Policy — retroactive documentation of PR #45 + W2-P2-A-003 architect KEEP decision). |
| 1.1 | pass-82 | 2026-04-21 | architect | OBS-082-003: corrected SS-20 Phase Introduced label Phase 1 → Phase 3 (SS-20 authored pass-80 alongside CAP-035 Phase 3 capability; consistent with SS-17/18/19). |
| 1.0 | pass-15 | 2026-04-15 | architect | Initial version |
