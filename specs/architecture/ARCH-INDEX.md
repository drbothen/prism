---
document_type: architecture-index
level: L3
version: "2.39"
status: draft
producer: architect
timestamp: 2026-05-04T00:00:00
phase: 1b
inputs: [domain-spec/L2-INDEX.md, prd.md, prd-supplements/interface-definitions.md, prd-supplements/nfr-catalog.md, prd-supplements/error-taxonomy.md]
traces_to: prd.md
deployment_topology: single-service  # planned — no service binary exists yet (prism-bin is a future-wave crate)
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
| Verification Architecture | verification-architecture.md | ~1,000 | formal-verifier, architect | Provable Properties Catalog, proof strategy — v1.30 (pass4 sister-line bump: VP-014/VP-015 property-text corrected to actual ParseError struct API) |
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
| ADR-001 | DTU Rate Limit Pattern | ACCEPTED v1.1 | 2026-04-22 | decisions/ADR-001-dtu-rate-limit-pattern.md |
| ADR-002 | L2 DTU Clone Template | ACCEPTED v1.1 | 2026-04-22 | decisions/ADR-002-l2-dtu-clone-template.md |
| ADR-003 | DTU Reset Lookup and Fidelity Auth | ACCEPTED v1.4 | 2026-04-22 | decisions/ADR-003-dtu-reset-lookup-and-fidelity-auth.md |
| ADR-004 | Kani Arbitrary Policy — Which Types Carry kani::Arbitrary | PROPOSED | 2026-04-26 | decisions/ADR-004-kani-arbitrary-policy.md |
| ADR-005 | AQL Injection Mitigation — Armis Adapter Query Trust Model | ACCEPTED v0.3 | 2026-04-26 | decisions/ADR-005-aql-injection-mitigation.md |
| ADR-006 | Multi-Tenant DTU Topology — OrgId/OrgSlug Identity, OrgRegistry, Configurable Shared/Client Mode | ACCEPTED v0.15 | 2026-04-27 | decisions/ADR-006-multi-tenant-dtu-topology.md |
| ADR-007 | Configurable Shared/Client DTU Mode — Per-Type Default Registry, Config Schema, and Isolation Semantics | ACCEPTED v0.15 | 2026-04-27 | decisions/ADR-007-configurable-dtu-mode.md |
| ADR-008 | DTU State Segregation — HashMap<(OrgId, String), V> Keying Pattern, Per-Tenant Lock Granularity, and Reset Semantics | ACCEPTED v0.14 | 2026-04-27 | decisions/ADR-008-dtu-state-segregation.md |
| ADR-009 | Multi-Tenant Data Generator — Hybrid Archetype Catalog + Deterministic Generator | ACCEPTED v0.15 | 2026-04-27 | decisions/ADR-009-multi-tenant-data-generator.md |
| ADR-010 | Customer Config Schema — customers/{org_slug}.toml Structure, Validation Rules, Loading Lifecycle, and Schema Versioning | ACCEPTED v0.17 | 2026-04-27 | decisions/ADR-010-customer-config-schema.md |
| ADR-011 | DTU Harness Isolation Modes — Logical (In-Process) and Network (Per-Port) | ACCEPTED v0.15 | 2026-04-27 | decisions/ADR-011-harness-isolation-modes.md |
| ADR-012 | Workspace src/ Convention Normalization — Canonical Crate Layout | ACCEPTED v0.18 | 2026-04-27 | decisions/ADR-012-src-convention.md |
| ADR-013 | Schedule Execution Semantics | PROPOSED v0.7 | 2026-05-03 | decisions/ADR-013-schedule-execution-semantics.md |
| ADR-014 | Local Pre-Push vs CI Gate Asymmetry | ACCEPTED v1.1 | 2026-04-30 | decisions/ADR-014-local-pre-push-ci-gate-asymmetry.md |
| ADR-015 | Detection Rule Language | PROPOSED v0.6 | 2026-05-03 | decisions/ADR-015-detection-rule-language.md |
| ADR-016 | Action Delivery Framework | PROPOSED v0.14 | 2026-05-02 | decisions/ADR-016-action-delivery-framework.md |
| ADR-017 | Case Lifecycle Invariants | PROPOSED v0.7 | 2026-05-03 | decisions/ADR-017-case-lifecycle-invariants.md |
| ADR-018 | Differential Result Pack Format | PROPOSED v0.6 | 2026-05-03 | decisions/ADR-018-differential-result-pack-format.md |
| ADR-019 | SIEM Output Formats | PROPOSED v0.4 | 2026-05-03 | decisions/ADR-019-siem-output-formats.md |
| ADR-020 | Story Status Taxonomy Reform — Closed Enum, Partial-Merge Semantics, and Graduation Contract | ACCEPTED v1.1 | 2026-05-08 | decisions/ADR-020-story-status-taxonomy-reform.md |
| ADR-021 | BC/VP Promotion Lifecycle — Draft → Active → Verified Transitions, Audit Cadence, and BC-INDEX Count Authority | ACCEPTED v1.1 | 2026-05-08 | decisions/ADR-021-bc-vp-promotion-lifecycle.md |
| ADR-022 | Production Runtime Wiring — prism-bin Chassis, Boot Sequence, Wiring Contracts, Infusion Fate, Hot-Reload Watcher, MCP Topology | ACCEPTED v1.1 | 2026-05-08 | decisions/ADR-022-production-runtime-wiring.md |
| ADR-023 | Plugin-Only Sensor Architecture — TOML Specs as Declarative Baseline, .prx WASM for Non-Declarative Cases, Retired CustomAdapter Rust Trait | COMMITTED v1.17 | 2026-05-10 | decisions/ADR-023-plugin-only-sensor-architecture.md |

## Architecture Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| AD-001 | Modular monolith via Cargo workspace; current workspace has 22 member crates (11 non-DTU production/build-helper crates: prism-core, prism-credentials, prism-mcp, prism-ocsf, prism-security, prism-spec-engine, prism-sensors, prism-storage, prism-audit, prism-query, ocsf-proto-gen; 11 DTU test-only crates: prism-dtu-common plus 10 per-surface clones). Remaining Phase-1 production crates (prism-bin, prism-operations) are targeted for future waves. Plus prism-dtu-harness planned in Wave 3 per ADR-011, bringing total to 23 crates at end of Wave 3. | Single binary deployment matches per-analyst stdio model; crate boundaries enforce module isolation without network overhead |
| AD-002 | DataFusion as SQL execution engine | Provides Arrow-native SQL with UDF extensibility; ephemeral SessionContext per query aligns with data-in-flight model |
| AD-003 | Chumsky 0.12 for PrismQL parsing | Zero-copy parser combinators with error recovery; axiathon reference proves pattern viability |
| AD-004 | RocksDB with 17 column families | Domain-isolated persistence for operational state; osquery-proven pattern; single-process LOCK fits stdio model. CFs: default, schedules, diff_results, detection_rules, detection_state, alerts, cases, audit_buffer, dirty_bits, watchdog, aliases, decorators, action_state, infusion_cache, plugin_state, event_buffer, case_dedup_idx. |
| AD-005 | rmcp 1.4 as MCP SDK | Official Anthropic SDK; #[tool_router] macro for 35+ tool registration; native tokio async. [NOT IMPLEMENTED — prism-mcp is a 10-line stub (verified 2026-05-08); no rmcp dep in Cargo.toml; covered by ADR-022 §F; tracked by S-5.01-FOLLOWUP-MCP-BOOT] |
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
| AD-022 | PrismQL Write Operations | Pipe mode terminal action verbs + SQL DML (INSERT/UPDATE/DELETE) targeting sensor write endpoints. All writes route through feature flags, risk-tier gates, dry-run/confirmation system, and intent-log audit. Filter mode remains read-only. Internal tables write-protected via PrismQL. [PARTIAL — safety gates landed via S-3.07 PR #135 (commit 2ae7185b); Phase 3 fetch hardcoded empty (`write_pipeline::execute_write_phase3`); no concrete adapter write overrides (`adapter::SensorAdapter::write`); SQL DML verbs NotImplemented (`write_table_registration` INSERT/UPDATE/DELETE handlers); covered by ADR-022 §C + S-3.02-FOLLOWUP-RUNTIME / W3-FIX-S307-001/002/003] |

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
| SS-10 | MCP Interface | api-surface.md | prism-mcp, prism-bin *(planned — S-WAVE5-PREP-01)* | Phase 1 [NOT IMPLEMENTED — crate is 10-line stub; PrismServer struct absent; no tool router; no rmcp dep; covered by ADR-022 §F + S-5.01-FOLLOWUP-MCP-BOOT] |
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
| SS-21 | Identity & Core Types | system-overview.md, module-decomposition.md | prism-core | Phase 3 — First BC anchored 2026-05-08: BC-2.21.001 (OrgRegistry init) per Option (d) decomposition — SS-21 transitions from 0-BC placeholder to active subsystem |
| SS-22 | Process Lifecycle | module-decomposition.md | prism-bin *(S-WAVE5-PREP-01 — NOT IMPLEMENTED)* | Phase 5 — Scope: boot orchestration contract only (sequencing of ADR-022 §B 11-step boot, startup failure exit-code map, traffic gate signal handlers); per-subsystem init contracts (config/org/cred/audit init) live in their respective subsystems (SS-06/SS-21/SS-03/SS-05) per Option (d) decomposition |

## Changelog

| Version | Pass | Date | Author | Change |
|---------|------|------|--------|--------|
| 2.39 | ADR-023-fix-burst-18-pass-23 | 2026-05-10 | state-manager | ADR-023 row version sync v1.16→v1.17 per F-PASS23-HIGH-001 closure (12th S-7.01 sibling-site recurrence; fix-burst-17 bumped ADR-023 to v1.17 but did not update ARCH-INDEX sibling-site). Version-sync sibling-site sweep applied. ARCH-INDEX v2.38→v2.39. |
| 2.38 | ADR-023-fix-burst-16-pass-21 | 2026-05-10 | state-manager | ADR-023 row added to ADR Registry (F-PASS21-MED-002 closure — sibling-file partial-fix gap closed; ADR-023 COMMITTED v1.16 since 2026-05-10). Pre-existing TD-031 volatile line citations fixed in AD-022 row (write_pipeline/adapter/write_table_registration line numbers replaced with function-name anchors). Pre-existing table cell-count violations fixed: AD-005, AD-022 (Architecture Decisions table extra annotation cell merged into Rationale), SS-10, SS-21, SS-22 (Subsystem Registry extra annotation cell merged into Phase Introduced). ARCH-INDEX v2.37→v2.38. |
| 2.37 | PR-139-pr-level-pass-6-fix-F-P6-MED-1 | 2026-05-09 | product-owner | ADR-022 v1.0→v1.1: §B step 2 stale `~/.prism/` literal replaced with platform-aware default (dirs::config_dir().join("prism")) to match BC-2.06.011 v1.2. Closes F-P6-MED-1 from PR #139 PR-LEVEL adversary pass-6. ARCH-INDEX v2.36→v2.37. |
| 2.36 | bundle-B-1b-option-d-decomposition-2026-05-08 | 2026-05-08 | architect | Bundle B Phase B-1b Option (d) correction: SS-22 scope narrowed to boot orchestration contract only (BC-2.22.001); per-subsystem init BCs distributed to natural subsystems. SS-21 row annotated — first BC (BC-2.21.001 OrgRegistry init) now anchored as of 2026-05-08; SS-21 is no longer a 0-BC placeholder. Subsystem Registry SS-21 and SS-22 rows updated with scope annotations. Mirrors module-decomposition.md v1.16 amendment. |
| 2.35 | bundle-B-1b-ss22-process-lifecycle-2026-05-08 | 2026-05-08 | architect | Bundle B Phase B-1b: SS-22 (Process Lifecycle) added to Subsystem Registry. Scope: ordered 11-step boot sequence orchestrated by prism-bin (ADR-022 §B), startup failure exit-code contract, signal handler lifecycle. Boot-sequence BCs assigned namespace BC-2.22.001..004 (pending PO authorship). BC-2.BOOT.001..004 placeholders in S-WAVE5-PREP-01 resolve to BC-2.22.001..004. |
| 2.34 | bundle-B-0-adr-022-production-runtime-wiring-2026-05-08 | 2026-05-08 | architect | Bundle B Phase B-0: ADR-022 (Production Runtime Wiring) registered. AD-005 row annotated [NOT IMPLEMENTED — prism-mcp 10-line stub; S-5.01-FOLLOWUP-MCP-BOOT]. AD-022 row annotated [PARTIAL — S-3.07 gates landed; Phase 3 fetch/adapter write/SQL DML gaps; W3-FIX-S307-001/002/003]. SS-10 row annotated [NOT IMPLEMENTED — no rmcp server; S-5.01-FOLLOWUP-MCP-BOOT]. prism-bin crate noted as SS-10 dependency (planned). POL-15 runtime_wiring_required_for_accepted_adrs satisfied for: AD-005, AD-018, AD-022. Six story seeds seeded in ADR-022 §G. ARCH-INDEX v2.33→v2.34. |
| 2.33 | bundle-A-2-3-adr-frontmatter-backfill-2026-05-08 | 2026-05-08 | architect | Bundle A.2.3 ADR frontmatter backfill: added `runtime_deliverables` and `wiring_deferred_to` fields to all 14 accepted ADRs (ADR-001 through ADR-012, ADR-014, ADR-020, ADR-021) enabling POL-15 (runtime_wiring_required_for_accepted_adrs) enforcement by audit-runtime-wiring skill. Version bumped +0.1 on each ADR. ADRs with code deliverables fully wired: ADR-003 (FidelityCheck/admin_token), ADR-005 (validate_aql), ADR-006 (OrgId/OrgRegistry), ADR-007 (DtuMode/DTU_DEFAULT_MODE), ADR-008 (state re-keying), ADR-009 (data generator), ADR-010 (customer config), ADR-011 (Harness), ADR-012 (layout script), ADR-014 (just check/check-ci). Pure methodology ADRs: ADR-001, ADR-002, ADR-020, ADR-021 (runtime_deliverables: []). |
| 2.32 | bundle-A-cleanup-2026-05-08 | 2026-05-08 | architect | Bundle A status-taxonomy reform: ADR-020 (story status taxonomy — closed enum, partial-merge semantics, graduation contract) and ADR-021 (BC/VP promotion lifecycle — draft→active→verified, audit cadence, BC-INDEX count authority) added to ADR Registry. Companion policies POL-12..16 added to policies.yaml v1.6. Hook specifications document added (hook-specs-bundle-a.md). No code changes; no story-file changes; schema + policy layer only. |
| 2.31 | pr-127-pass4-remediation | 2026-05-05 | architect | Adversary pass-4 (F-MEDIUM-001) property-text correction cascade: VP-014 v1.6 + VP-015 v1.7 replace non-existent `ParseError::QueryTooLarge` / `ParseError::NestingTooDeep` enum-variant references with correct `Err(Vec<ParseError>)` API (message contains `E-QUERY-003`). Document Map verification-architecture.md row updated v1.29→v1.30. VP-INDEX v1.29. ARCH-INDEX v2.30→v2.31. Note: proof_file_hash values retained — hashes confirmed unchanged at commit 8feb4cf2. Hash may need re-computation after implementer #4 push lands changes to vp015_depth_limit.rs. |
| 2.30 | pr-127-formal-verify | 2026-05-05 | architect | VP-014 and VP-015 promoted to verified following Kani proof runs at commit f5212641 (PR #127). Document Map verification-architecture.md row updated v1.28→v1.29. Cross-ref: VP-INDEX v1.28, vp-014 v1.5, vp-015 v1.6, verification-architecture.md v1.29. ARCH-INDEX v2.29→v2.30. |
| 2.29 | TD-W4-CV-LOW-002-closure | 2026-05-04 | architect | TD-W4-CV-LOW-002 closure: ADR-016 date sync. ADR-016 row date corrected 2026-05-04 → 2026-05-02 to match ADR-016 frontmatter `timestamp: 2026-05-02T00:00:00Z`. Version/status/title confirmed clean (PROPOSED v0.14, "Action Delivery Framework"). ARCH-INDEX v2.28→v2.29. |
| 2.28 | W4-Phase4A-Pass28-fix | 2026-05-04 | state-manager | Pass 28 BLOCKED→REMEDIATED. F-P28-H-001: vp-045 spec v1.3→v1.4 (H1 heading "Schedule Semaphore" → "Action Delivery Semaphore" per VP-INDEX line 66 canonical + BC-2.18.004 H1; Pass 26 body-rewrite sister-line gap; 7th orchestrator-prompt-introduced defect — H1-axis; SUBSTANTIVE). 12 cross-cuts verified CLEAN. Window stays 0/3; Pass 29 next. |
| 2.27 | W4-Phase4A-Pass27-fix | 2026-05-04 | state-manager | Pass 27 BLOCKED→REMEDIATED. F-P27-H-001: ADR-016 §5.4 footer (line 533) + v0.12 changelog entry (line 579) contained wrong VP-047 rationale "action delivery dedup correctness" → canonical "template variable UUID v7 validation" per VP-INDEX line 68 + BC-2.18.009; SUBSTANTIVE; architect v0.14. Comprehensive grep across all 6 W4 ADRs confirmed sole VP-INDEX mis-anchor site. META-INSIGHT: 6th orchestrator-prompt-introduced defect this session — semantic mis-anchor in rationale text (NEW class beyond stale module names). TD-VSDD-052 codified (pre-dispatch VP scope verification). ARCH-INDEX v2.26→v2.27. ADR-016 v0.14 in registry. Window stays 0/3; Pass 28 next. |
| 2.26 | W4-Phase4A-Pass26-fix | 2026-05-04 | state-manager | Pass 26 BLOCKED→REMEDIATED. F-P26-H-001: ADR-016 v0.12→v0.13 (lines 552+568 orphan `action_dispatcher` token → `action_delivery`; sibling-file regression of F-P25-H-001 PRD fix; SUBSTANTIVE). F-PreP27-H-001: vp-045 spec v1.2→v1.3 (lines 37/44/68 same orphan token; 3 sites; caught proactively before Pass 27; SUBSTANTIVE). META-INSIGHT: all 5 orphan sites across 3 docs (PRD, ADR-016, vp-045 spec) introduced by orchestrator-authored fix-burst prompt text; TD-VSDD-051 codified (orchestrator-prompt verification + sibling-ADR prose sweep). ARCH-INDEX version 2.25→2.26. ADR-016 v0.13 in registry. Window stays 0/3; Pass 27 next. |
| 2.25 | W4-Phase4A-Pass25-fix | 2026-05-04 | state-manager | F-P25-H-001: prd.md v1.9→v1.10 (PRD §2 line 382 stale `action_dispatcher` token in subsystem-introduction prose → `action_delivery` per concurrency-architecture v1.1 + module-decomposition v1.13 canonicals; orphan introduced by orchestrator-authored pre-Pass-24 fix-burst prompt; SUBSTANTIVE). TD-VSDD-050 filed (PRD §2 SUBSYSTEM PROSE sync check — sibling to TD-VSDD-049 BC-table sync). ARCH-INDEX version 2.24→2.25. Window stays 0/3; Pass 26 next. |
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
