# Changelog

All notable changes to Prism are documented here.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [1.0.0-beta.1] - 2026-09-07

### Highlights

- MCP stdio server exposing sensor data via the Model Context Protocol — designed for AI agent-driven security analyst workflows
- PrismQL query engine — SQL-pipe queries over live API data with OCSF normalization; federation-capable architecture shipping one sensor in beta.1
- OCSF + protobuf normalization at the sensor adapter boundary — events are normalized to Open Cybersecurity Schema Framework on ingestion
- Multi-tenant MSSP sensor management with per-org credential isolation and hot-reload configuration
- Built-in Claroty xDome sensor — TOML-spec-driven adapter with full table coverage over the live xDome API, normalized to OCSF
- Agent-facing version identity — `prism-mcp` reports `serverInfo.version` via MCP protocol; `prism-spec-engine` sets a version-pinned User-Agent on all outbound sensor requests

### Upgrade Notes

- Intel-macOS (`x86_64-apple-darwin`) is not a published binary target for this release. Published targets are: `aarch64-apple-darwin` (Apple Silicon macOS), `x86_64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`, and `x86_64-pc-windows-msvc`. Intel-Mac users must build from source.

### Added

- CI/CD pipeline and release workflow ([#1](https://github.com/drbothen/prism/pull/1))

- developer toolchain bootstrap ([#2](https://github.com/drbothen/prism/pull/2))

- prism-dtu-common DTU common infrastructure ([#4](https://github.com/drbothen/prism/pull/4))

- prism-dtu-threatintel L2 ThreatIntel DTU clone ([#6](https://github.com/drbothen/prism/pull/6))

- prism-dtu-nvd L2 NVD CVE API DTU clone ([#7](https://github.com/drbothen/prism/pull/7))

- prism-dtu-cyberint — L2 Cyberint API behavioral clone ([#10](https://github.com/drbothen/prism/pull/10))

- prism-dtu-crowdstrike — CrowdStrike Falcon API L4 behavioral clone ([#9](https://github.com/drbothen/prism/pull/9))

- prism-dtu-claroty — Claroty xDome API L4 adversarial DTU (53 tests) ([#11](https://github.com/drbothen/prism/pull/11))

- prism-core foundational types — TenantId, PrismError, StorageDomain ([#13](https://github.com/drbothen/prism/pull/13))

- prism-core capability resolution engine — deny-by-default, most-specific-wins, VP-002/003/004 Kani proofs ([#15](https://github.com/drbothen/prism/pull/15))

- prism-security + prism-mcp — four-layer prompt injection defense (SS-09) ([#16](https://github.com/drbothen/prism/pull/16))

- prism-core entity types and state machines ([#17](https://github.com/drbothen/prism/pull/17))

- prism-spec-engine — spec loading, pipeline execution, validation ([#14](https://github.com/drbothen/prism/pull/14))

- prism-ocsf OCSF schema loading and DynamicMessage infrastructure ([#18](https://github.com/drbothen/prism/pull/18))

- sensor-spec-engine write endpoints with verb uniqueness ([#20](https://github.com/drbothen/prism/pull/20))

- prism-security credential store trait + Argon2id file backend ([#19](https://github.com/drbothen/prism/pull/19))

- prism-spec-engine infusion specs + UDF registration ([#21](https://github.com/drbothen/prism/pull/21))

- prism-feature-flags P0 core — runtime gating + analyst audit ([#23](https://github.com/drbothen/prism/pull/23))

- prism-wasm plugin runtime + host-function security boundary ([#22](https://github.com/drbothen/prism/pull/22))

- prism-security confirmation tokens — RiskTier gate, single-use tokens, Kani VP-007..010 ([#25](https://github.com/drbothen/prism/pull/25))

- prism-spec-engine hot reload + runtime management ([#24](https://github.com/drbothen/prism/pull/24))

- prism-ocsf field mapping and normalization ([#26](https://github.com/drbothen/prism/pull/26))

- prism-credentials CRUD, resolution, secret redaction, and audit logging ([#27](https://github.com/drbothen/prism/pull/27))

- DTU demo server — unified multi-clone harness (closes Wave 1) ([#29](https://github.com/drbothen/prism/pull/29))

- arch-decided + auth — 3 TD items (TD-WV1-01, TD-WV1-02, TD-WV0-07) + ADR-003 Amendments #3/#4/#5 ([#40](https://github.com/drbothen/prism/pull/40))

- prism-storage — RocksDB foundation, 16 CFs, dirty bits (Wave 2 keystone) ([#43](https://github.com/drbothen/prism/pull/43))

- prism-storage — audit buffer + watchdog + denylist (Wave 2) ([#52](https://github.com/drbothen/prism/pull/52))

- prism-storage — decorators + internal tables (Wave 2) ([#53](https://github.com/drbothen/prism/pull/53))

- prism-dtu-pagerduty — Events API v2 DTU (Wave 2) ([#55](https://github.com/drbothen/prism/pull/55))

- prism-audit — audit emitter + redaction + AuditRiskLevel (Wave 2) ([#58](https://github.com/drbothen/prism/pull/58))

- prism-dtu-jira — Jira REST API v3 DTU (Wave 2) ([#56](https://github.com/drbothen/prism/pull/56))

- prism-dtu-slack — Slack webhook DTU + FailureLayer 429 body fix (Wave 2) ([#57](https://github.com/drbothen/prism/pull/57))

- prism-sensors — datasource trait + adapter registry + fanout + retry (Wave 2) ([#54](https://github.com/drbothen/prism/pull/54))

- prism-audit — specialized audit events (credential, vector, flag, token) (Wave 2) ([#59](https://github.com/drbothen/prism/pull/59))

- prism-sensors — per-sensor auth + pagination + timestamp parsing (Wave 2) ([#60](https://github.com/drbothen/prism/pull/60))

- event-tables — TableType + EventBufferStore + EventPoller + prism-query crate (Wave 2 finale) ([#61](https://github.com/drbothen/prism/pull/61))

- DTU_DEFAULT_MODE registry — 10-entry slice per ADR-007 §2.3 (S-3.0.02) ([#74](https://github.com/drbothen/prism/pull/74))

- schema derivation — Armis + CrowdStrike Rust types from Go SDK sources ([#75](https://github.com/drbothen/prism/pull/75))

- Archetype catalog + GenOpts API behind fixture-gen feature (S-3.7.01) ([#76](https://github.com/drbothen/prism/pull/76))

- CrowdStrike fixture generator — 8 archetypes + 2-step pagination + OAuth2 (S-3.7.05) ([#80](https://github.com/drbothen/prism/pull/80))

- Cyberint fixture generator — 8 archetypes × 4 endpoints (S-3.7.03) ([#77](https://github.com/drbothen/prism/pull/77))

- Armis fixture generator — 8 archetypes + AQL envelope + polymorphic IDs (S-3.7.04) ([#78](https://github.com/drbothen/prism/pull/78))

- Claroty fixture generator — all 8 archetypes (S-3.7.02) ([#79](https://github.com/drbothen/prism/pull/79))

- OrgId(Uuid v7) newtype — prism-core canonical org identity (BC-3.1.001) ([#81](https://github.com/drbothen/prism/pull/81))

- HS-007 multi-tenant cross-repo failure holdout refresh — re-anchor to Wave 3 BCs ([#84](https://github.com/drbothen/prism/pull/84))

- HS-006 multi-tenant state recovery holdout refresh — re-anchor to Wave 3 BCs (closes TD-HOLDOUT-W2-002) ([#83](https://github.com/drbothen/prism/pull/83))

- workspace src/ convention sweep — check-crate-layout.sh + CI gate + CRATE-LAYOUT.md (BC-3.7.001) ([#82](https://github.com/drbothen/prism/pull/82))

- cyberint multi-tenant state segregation — alert_store + session_store re-keying (BC-3.2.001/003) ([#87](https://github.com/drbothen/prism/pull/87))

- claroty multi-tenant state segregation — (OrgId, String) re-keying (BC-3.2.001/003) ([#86](https://github.com/drbothen/prism/pull/86))

- crowdstrike multi-tenant state segregation — containment + detection store re-keying (BC-3.2.001/003, D-048) ([#85](https://github.com/drbothen/prism/pull/85))

- prism-dtu-armis multi-tenant state segregation — (OrgId, String) re-keying (BC-3.2.001) ([#88](https://github.com/drbothen/prism/pull/88))

- prism-dtu-slack shared-mode OrgId ingress tagging + DtuMode validation (BC-3.2.004/005) ([#89](https://github.com/drbothen/prism/pull/89))

- prism-dtu-jira shared-mode OrgId ingress tagging (BC-3.2.004/005) ([#91](https://github.com/drbothen/prism/pull/91))

- prism-dtu-pagerduty shared-mode OrgId ingress tagging (BC-3.2.004/005) ([#90](https://github.com/drbothen/prism/pull/90))

- prism-customer-config TOML schema + parser + startup validator (BC-3.3.001/002/003/004) ([#92](https://github.com/drbothen/prism/pull/92))

- workspace TenantId → OrgSlug rename across all crates (BC-3.1.001) ([#93](https://github.com/drbothen/prism/pull/93))

- prism-core OrgRegistry — bijective BiMap with conflict + idempotent semantics (BC-3.1.001/003/004) ([#94](https://github.com/drbothen/prism/pull/94))

- prism-spec-engine OrgId-scoped sensor specs — OrgScopedSpecStore + get_spec resolution (BC-3.1.001) ([#98](https://github.com/drbothen/prism/pull/98))

- prism-audit org_id + org_slug fields + SHA-256 aql_hash (BC-3.1.001/002) ([#96](https://github.com/drbothen/prism/pull/96))

- OrgRegistry boot from customer config — validate-before-register pattern (BC-3.1.003/004, BC-3.3.004) ([#97](https://github.com/drbothen/prism/pull/97))

- prism-credentials OrgId-keyed namespace — CredentialStoreOrgId trait + impls (BC-3.2.002) ([#95](https://github.com/drbothen/prism/pull/95))

- prism-sensors OrgId-keyed adapter dispatch — BC-3.2.001/004 (v0.2.0) ([#99](https://github.com/drbothen/prism/pull/99))

- prism-dtu-harness logical isolation + crash detection + failure injection (BC-3.5.001/3.6.001/3.6.002 GREEN) ([#101](https://github.com/drbothen/prism/pull/101))

- reload_config detects + warns on DTU mode changes — invariant-preserving rejection (BC-3.2.005) ([#100](https://github.com/drbothen/prism/pull/100))

- prism-query org-scoped CrowdStrike session IDs — XOR UUID v7 embedding (BC-3.2.003 / D-048) ([#102](https://github.com/drbothen/prism/pull/102))

- prism-dtu-harness network isolation mode — atomic per-port TCP + HTTP 401 routing (BC-3.5.002 / ADR-011 D-058) ([#103](https://github.com/drbothen/prism/pull/103))

- prism-dtu-harness builder ergonomics, per-test overrides, and documentation (BC-3.5.001/002, BC-3.6.001) ([#104](https://github.com/drbothen/prism/pull/104))

- migrate slack/pagerduty/jira tests to prism-dtu-harness shared-mode (BC-3.2.004, BC-3.5.001) ([#110](https://github.com/drbothen/prism/pull/110))

- migrate prism-dtu-cyberint tests to prism-dtu-harness (BC-3.5.001/002, BC-3.6.001) ([#111](https://github.com/drbothen/prism/pull/111))

- migrate prism-dtu-claroty tests to prism-dtu-harness (BC-3.5.001/002) ([#107](https://github.com/drbothen/prism/pull/107))

- migrate prism-dtu-crowdstrike tests to prism-dtu-harness (BC-3.5.001/002, BC-3.2.003) ([#109](https://github.com/drbothen/prism/pull/109))

- migrate prism-dtu-armis tests to prism-dtu-harness (BC-3.5.001/002) ([#108](https://github.com/drbothen/prism/pull/108))

- S-3.01 PrismQL parser — filter/SQL/pipe modes via Chumsky 0.12 ([#127](https://github.com/drbothen/prism/pull/127))

- PrismQL write parser extensions — BC-2.11.004 v1.4 + perimeter +10 symbols ([#130](https://github.com/drbothen/prism/pull/130))

- PrismQL query materialization — BC-2.11.001/005/006/007/011/012 ([#129](https://github.com/drbothen/prism/pull/129))

- pagination + caching (cursor lifecycle, moka LRU, write invalidation)

- alias system — alias-write feature flag + capability/resolver/store/tools/types modules

- explain query diagnostics — BC-2.11.010

- write execution pipeline — BC-2.04.001/005, BC-2.05.009 (Phase 2 safety + Phase 5 dispatch + structured error taxonomy)

- prism-bin chassis — boot sequence + CLI + signal handlers ([#138](https://github.com/drbothen/prism/pull/138))

- QueryEngine execution pipeline — fill 9 todo!() sites (S-3.02-FOLLOWUP-RUNTIME) ([#141](https://github.com/drbothen/prism/pull/141))

- S-PLUGIN-PREREQ-A — SensorId(Arc<str>) open newtype replaces SensorType closed enum ([#142](https://github.com/drbothen/prism/pull/142))

- S-PLUGIN-PREREQ-B — Real PipelineExecutor with multi-step fetch, auth refresh, and Structured Event Catalog ([#143](https://github.com/drbothen/prism/pull/143))

- S-PLUGIN-PREREQ-C — TOML grammar extensions (page_size + JSONPath brackets/wildcards + Interpolator escape) + pub-API hardening (30 #[non_exhaustive] types + symbol-keyed allowlist + SensorIdValidationError re-export) ([#144](https://github.com/drbothen/prism/pull/144))

- plugin runtime boot wiring (18 ACs; 25 Red Gate tests; 3-CLEAN converged) ([#149](https://github.com/drbothen/prism/pull/149))

- un-seal SensorAuth + deprecate CustomAdapter + WriteToolInvalidationMap runtime extensibility ([#151](https://github.com/drbothen/prism/pull/151))

- author 4 production TOML sensor specs + DTU-parity tests (Wave 1) ([#153](https://github.com/drbothen/prism/pull/153))

- scaffold stubs for per-org overlay loading (Red Gate)

- implement per-org overlay loading (BC-2.06.012..016)

- CrowdStrike OAuth2 refresh-on-401 PRX wasm plugin ([#154](https://github.com/drbothen/prism/pull/154))

- per-org sensor endpoint overlay loading (ADR-029) ([#155](https://github.com/drbothen/prism/pull/155))

- delete 4 named auth modules + replace init_registry_for_org ([#156](https://github.com/drbothen/prism/pull/156))

- prism-query — convert 3 sensor-name dispatch sites to spec-catalog lookup ([#157](https://github.com/drbothen/prism/pull/157))

- prism-ocsf — SpecDrivenMapper replaces 4 hardcoded OCSF mappers ([#158](https://github.com/drbothen/prism/pull/158))

- WASM plugin CI toolchain + 3 deferred test closures ([#159](https://github.com/drbothen/prism/pull/159))

- rewrite 12 sensor-named tests to TOML fixture loading + compile-fail perimeter ([#160](https://github.com/drbothen/prism/pull/160))

- retire types::SensorSpec — unify on spec_parser::SensorSpec (ADR-030) ([#161](https://github.com/drbothen/prism/pull/161))

- prism-query QueryEngine execution pipeline — fill 9 todo!() sites ([#162](https://github.com/drbothen/prism/pull/162))

- prism-mcp PrismServer — rmcp 1.7, tool router, injection defense ([#163](https://github.com/drbothen/prism/pull/163))

- Cyberint DTU access_token auth + StaticCookieAuthProvider + sensor-spec fidelity bundle ([#164](https://github.com/drbothen/prism/pull/164))

- ${env.VAR} interpolation resolution in sensor-spec string fields

- SpecDrivenSensorAdapter + boot step 9A — bridge PipelineExecutor to AdapterRegistry (closes GAP-002-A) ([#166](https://github.com/drbothen/prism/pull/166))

- prism-dtu-claroty audit_log DTU route — Gap-CL-006 ([#167](https://github.com/drbothen/prism/pull/167))

- Armis AQL search endpoint fidelity — DTU /api/v1/search push-down ([#168](https://github.com/drbothen/prism/pull/168))

- crowdstrike multi-region base_url via env var ([#170](https://github.com/drbothen/prism/pull/170))

- e2e subprocess smoke test — all 4 sensors + multi-org isolation + AQL push-down + e2e CI gate ([#171](https://github.com/drbothen/prism/pull/171))

- HTTP method whitelist validation in sensor spec — E-SPEC-025 + BC-2.16.009 v1.10 Rule 7 (DRIFT-D926-001) ([#172](https://github.com/drbothen/prism/pull/172))

- query-param push-down (limit + time-window) into PipelineExecutor — ADR-033 T1 + Armis AQL full wiring ([#173](https://github.com/drbothen/prism/pull/173))

- migrate sensor TOMLs ocsf_class security_finding → detection_finding (OCSF v1.1) ([#174](https://github.com/drbothen/prism/pull/174))

- demo setup scripts + prism credential set/delete CLI + operator runbook ([#176](https://github.com/drbothen/prism/pull/176))

- NormalizePathLayer trailing-slash route fidelity — ADR-031 §D8-b Gap-CL-001 (BC-2.16.013 v1.25)

- Gap-CL-006 CLOSED audit_logs closure comment + spec-prose fidelity test (BC-2.16.013 v1.25)

- OffsetLimit POST-body pagination for Claroty (closes Gap-CL-004)

- harness clone route parity — Armis search + Claroty audit_log (closes F-P6-DEFER-001 / F-P10-LOW-001) ([#180](https://github.com/drbothen/prism/pull/180))

- baseline seeding retrofit — per-client distinct seeded DTU data (BC-2.06.018) ([#181](https://github.com/drbothen/prism/pull/181))

- deterministic scenario progression + enrichment correlation for multi-client SOC live demo

- per-DTU-instance multi-address binding for multi-tenant overlay testing

- multi-org sensor isolation smoke test

- start-multi subcommand + N-org demo script consolidation

- enrichment pivot query support for prism-query

- MCP tool routing, structured error responses, tri-state client scoping

- dynamic table availability — TableRegistry + E-QUERY-037 plan-time gate

- full infusion engine (MMDB/CSV/JSON/HttpLookup sources, 3-tier cache, plugin runtime, SEC-001 source-size guard)

- MCP Resources & Prompts — per-org DI resources, sensor-health, prompt sanitization

- ThreatIntel/NVD dual-path enrichment (HttpLookup + WASM plugin infusion)

- IOC stamping (Cyberint + CrowdStrike) + canonical pivot query validation

- MCP teaching surface — prism_describe + reference prompts + ADR-042 reload-aware schema

- PrismQL Query Engine L4 — E-QUERY-038 column gate + pedagogical enrichments + normalized_pql ([#198](https://github.com/drbothen/prism/pull/198))

- ENRICH-1/2/3/4-B integration — PrismQL enrichment chain end-to-end

- prism-mcp Sensor Health Subsystem — live probes, probe_table routing, E-SPEC-026, HealthSummary ([#202](https://github.com/drbothen/prism/pull/202))

- demo-readiness grammar + MCP fixes ([#203](https://github.com/drbothen/prism/pull/203))

- ADR-052 §D4 Option A temporal typing — lenient-parse + AST-walk + String coercion ([#214](https://github.com/drbothen/prism/pull/214))

- typed enrichment-UDF output with consistent ColumnType coercion

- PrismQL case-insensitive operators IEQ/IIN/INE with adapter-boundary OCSF enum-label normalization ([#217](https://github.com/drbothen/prism/pull/217))

- T13 pre-flight live-audit script (106-check coverage matrix) + live-audit triage codifications (holdout gate, wire-shape, SAP-3, SID-2) ([#226](https://github.com/drbothen/prism/pull/226))

- release.yml repair — 5-platform release pipeline (DEF-REL-001..004, prerelease, static musl, pinned toolchain) ([#228](https://github.com/drbothen/prism/pull/228))

- ADR-058 Stage 1 — column coercion gap closure (EC-016-013-007/008/009/030) ([#240](https://github.com/drbothen/prism/pull/240))

- OCSF column-naming/routing Stage 2 (§J1–J5, query-surface resolution, push-down)

- LIMIT-aware early-stop pagination + is_truncated signaling (ADR-060 §D8)

- Claroty xDome vulnerabilities table — TOML block, 19-column Tier-1/Tier-2 spec, live structural tests (Wave A G1) ([#245](https://github.com/drbothen/prism/pull/245))

- Claroty xDome OT activity events table — TOML block, 21-column Tier-1/Tier-2 spec, live structural tests (Wave A G2) ([#246](https://github.com/drbothen/prism/pull/246))

- Claroty xDome device_vulnerability_relations table — 13-column Tier-1/Tier-2 spec, composite PK, live structural tests (Wave B G3) ([#247](https://github.com/drbothen/prism/pull/247))

- Claroty xDome servers + server_interfaces tables — 17-col + 10-col TOML spec, live structural tests (Wave C G4) ([#248](https://github.com/drbothen/prism/pull/248))

- Claroty xDome org policy tables — 4 TOML blocks, entity_management/3004, 8 Json cols, fw URL↔envelope asymmetry, live structural tests (Wave C G5) ([#249](https://github.com/drbothen/prism/pull/249))

- Claroty xDome ACL policies table — 14th Claroty table, pagination-none, mandatory filter_by/policy_acl_syntax, Wave C G6 ([#250](https://github.com/drbothen/prism/pull/250)) ([#250](https://github.com/drbothen/prism/pull/250))

- install.sh + install.ps1 — checksum-verified 5-platform consumer install scripts ([#254](https://github.com/drbothen/prism/pull/254))

- version identity — 1.0.0-dev reset + PRISM_VERSION injection + version-agnostic docs ([#262](https://github.com/drbothen/prism/pull/262))

- agent-facing version identity — prism-mcp serverInfo + prism-spec-engine UA (S-REL-AGENT-VERSION-001) ([#263](https://github.com/drbothen/prism/pull/263))

- git-cliff CHANGELOG tooling + drop Intel mac (4-target matrix) — S-REL-CLIFF-001, S-REL-WRITER-001, S-REL-DROP-INTEL-MAC-001 ([#264](https://github.com/drbothen/prism/pull/264))


### Fixed

- mount missing DTU introspection routes on nvd + threatintel ([#28](https://github.com/drbothen/prism/pull/28))

- remediate adversarial Pass 1 blocking findings (C-001 + H-001/002 + M-002/003 + L-002) ([#30](https://github.com/drbothen/prism/pull/30))

- remediate Pass 2 code findings (H-001, M-001/003/004) ([#31](https://github.com/drbothen/prism/pull/31))

- wire TLS from --tls CLI flag through harness to all 6 DTU clones ([#32](https://github.com/drbothen/prism/pull/32))

- CI hardening — 6 TD items (TD-WV0-01, 02, 09, 10, 11, 12) ([#33](https://github.com/drbothen/prism/pull/33))

- CI hardening followups — 4 PR-A review items (TD-WV05-PR33-001/002/003/004) ([#34](https://github.com/drbothen/prism/pull/34))

- config/workspace hardening — 3 TD items (TD-WV0-03, 04, 06) ([#35](https://github.com/drbothen/prism/pull/35))

- small code fixes — 2 TD items (TD-WV0-08, TD-WV1-03) ([#36](https://github.com/drbothen/prism/pull/36))

- add DEMO_FAKE_* env var exports to start-demo.sh (IMPORTANT-001 closure) ([#38](https://github.com/drbothen/prism/pull/38))

- TD-WV1-04 follow-ups — 3 items (FU-001, FU-002, FU-003) ([#39](https://github.com/drbothen/prism/pull/39))

- gate Pass 1 remediation — H-001 workspace lint + M-001/M-004 + L-001/003/004/005

- gate Pass 2 remediation — H-001 (9 files) + M-004 (crowdstrike workspace lints) ([#42](https://github.com/drbothen/prism/pull/42))

- post-merge workflow — toolchain: nightly + drop deprecated Kani flags ([#44](https://github.com/drbothen/prism/pull/44))

- post-merge — RUSTUP_TOOLCHAIN=nightly + CaseStatus kani::Arbitrary ([#45](https://github.com/drbothen/prism/pull/45))

- hotfix #3 — align fuzz targets with reality + scope Kani to prism-core et al ([#47](https://github.com/drbothen/prism/pull/47))

- hotfix #4 — explicit --target x86_64-unknown-linux-gnu for cargo fuzz ([#48](https://github.com/drbothen/prism/pull/48))

- hotfix #5 — relocate misplaced dependency lines in fuzz/Cargo.toml ([#49](https://github.com/drbothen/prism/pull/49))

- disable post-merge workflow pending redesign (closes 7-layer cascade) ([#50](https://github.com/drbothen/prism/pull/50))

- default-enable \`dtu\` feature so workspace test runs (OBS-001) ([#51](https://github.com/drbothen/prism/pull/51))

- propagate EventBufferStore backend errors + align ULID docs (W2-P1-A-001/004/005) ([#62](https://github.com/drbothen/prism/pull/62))

- S-2.08 AC-5 deferred reclassification + coverage theater fix (W2-P1-A-002, W2-P1-A-015) ([#64](https://github.com/drbothen/prism/pull/64))

- audit emitter persistence + evict_expired backend scan (WGC-W2-001/002 HIGH) ([#68](https://github.com/drbothen/prism/pull/68))

- SecretString bearer tokens + Armis AQL validator (WGS-W2-001/002 HIGH) ([#69](https://github.com/drbothen/prism/pull/69))

- gate MockStorageEngine behind cfg(any(test, feature = "test-utils")) ([#70](https://github.com/drbothen/prism/pull/70))

- strip token_id from generated/expired audit entries + replace tautology test (P7 HIGH-001/003) ([#71](https://github.com/drbothen/prism/pull/71))

- Armis AQL validator — multi-occurrence select + single-quote rejection (P7 HIGH-002) ([#72](https://github.com/drbothen/prism/pull/72))

- lefthook fmt hook — cargo fmt --all --check (TD-W2-FIX-H-001) ([#73](https://github.com/drbothen/prism/pull/73))

- pre-push gate tuning — proptest 100 cases, audit/deny CI-only, semver-checks pre-tag (CAP-DEV-SPEED) ([#106](https://github.com/drbothen/prism/pull/106))

- CI wall-clock optimization — cargo-nextest, per-platform PROPTEST_CASES, mold linker ([#112](https://github.com/drbothen/prism/pull/112))

- KeyringBackend CredentialStoreOrgId — SEC-004 false-positive remediation + regression tests ([#115](https://github.com/drbothen/prism/pull/115))

- customer-config spec path traversal hardening (CWE-22/E-CFG-018) ([#114](https://github.com/drbothen/prism/pull/114))

- HarnessBuilder failure scope + Drop grace (CR-001/CR-002 HIGH) ([#116](https://github.com/drbothen/prism/pull/116))

- X-Org-Id auth enforcement on 4 DTU clones (CWE-287/CWE-639/A01 — gate-step-d SEC-001) ([#113](https://github.com/drbothen/prism/pull/113))

- pass-49 cleanup bundle — CR-010..015, SEC-P2-002/006, BC-3.5.002 timing (lands first in W3.2) ([#118](https://github.com/drbothen/prism/pull/118))

- /dtu/reset admin token auth on 4 DTU clones (CWE-306/A07 — closes SEC-NEW-001) ([#119](https://github.com/drbothen/prism/pull/119))

- BC-3.2.002 regression coverage — CredentialStoreOrgId false-positive remediation ([#121](https://github.com/drbothen/prism/pull/121))

- config validation + dispatch hygiene bundle — CR-003/004/005/006, SEC-006/007 ([#120](https://github.com/drbothen/prism/pull/120))

- TOML inline-table redaction + pipe-finder anchor + constant-time admin token (SEC-P3-001/002/003) ([#122](https://github.com/drbothen/prism/pull/122))

- pass-50 sibling endpoint coverage — CR-016/017/018 poll cadence + org-id guards + doc deviation ([#123](https://github.com/drbothen/prism/pull/123))

- admin-token uniformity across 5 DTU clones (P7 CR-021/022) ([#125](https://github.com/drbothen/prism/pull/125))

- retire ColumnType shadow enum, re-export prism_core canonical + semver override (ADR-024)

- maintenance D-572 — F-LP16 edition 2021→2024 + F-LP22 PluginError #[non_exhaustive] ([#150](https://github.com/drbothen/prism/pull/150))

- F-LP1-LOW-001+LOW-002+MED-001+OBS-001+HIGH-001 — remove dead _overlay_file param + delete raw_toml_contains_tables_header + fix SpecLoader doc refs + replace .expect() with defensive match + remove E-SPEC-022 early-return guard to aggregate all errors

- F-LP1-MED-001 — rename SpecLoader::load_all_with_overlays doc ref to OverlayLoader::load_overlays

- F-LP1-CRIT-002 — implement resolve_spec_for_fanout + fan_out_with_overlay_map

- F-LP1-CRIT-001 — implement step4_load_sensor_specs_with_overlays + wire into boot

- F-LP2-MED-001 — remove trailing 'Instance:' from E-SPEC-023 canonical-template drift (POL-24 verbatim)

- F-LP2-CRIT-001 + F-LP2-HIGH-001 — thread resolved_spec_map through MaterializationContext + QueryEngine + RunningServer + end-to-end overlay-dispatch test

- F-LP5-LOW-001 — replace paraphrased doc comments with canonical taxonomy references (option b)

- F-LP6-LOW-001 — apply forward-pointer style to 2 sibling canonical-error-template doc-comments (E-SPEC-022, E-SPEC-023) — Standing Rule 3 §1b sibling-sweep completion

- F-LP14-CI-001 — AC-005 test uses include_str!() fixture instead of .factory/ walk-up (PR #155 CI-portability)

- PR #155 fix-burst — 24-finding consolidation

- pass-2 fix-burst — sanitize extends_value + hard-abort boot test

- PR #155 pass-3 carry-forward findings

- SEC-PASS4-002 sanitize expected_sensor_id/org_slug in E-SPEC-021 error path

- SEC-PASS5-001/002 sanitize_for_log sibling-sweep gaps in overlay.rs

- SEC-PASS6-001 sanitize dir_display at derivation in E-SPEC-022 path

- correct BC-3.5.002 precond-3 org-guard mis-cite → W3-FIX-SEC-001 (DRIFT-D943-001)

- query-core — E-QUERY taxonomy splits, watchdog grace period, cache/hot-reload fixes

- mcp-boot lane — fail-closed write audit, reload_config WriteTool, capability fields, security polish ([#184](https://github.com/drbothen/prism/pull/184))

- dtu-fleet lane — DTU schema parity, Postcondition-5 propagation, Armis isolation, CrowdStrike parity ([#182](https://github.com/drbothen/prism/pull/182))

- demo-fidelity code fixes + rustls-tls test-dep standardization ([#208](https://github.com/drbothen/prism/pull/208))

- bump crossbeam-epoch past RUSTSEC-2026-0204

- E-QUERY-038 plan-time column gate — 14-position binding-context walk + 6 suspension rules (BC-2.11.016 v1.25) ([#219](https://github.com/drbothen/prism/pull/219))

- E-QUERY-042 gate for Literal::Timestamp in GROUP BY/ORDER BY (ADR-052 §D4 v1.11 arms 6+7) ([#220](https://github.com/drbothen/prism/pull/220))

- CrowdStrike devices pipeline empty results — POST fan-out + empty-MemTable pre-registration + E-QUERY-043 gate ([#221](https://github.com/drbothen/prism/pull/221))

- PrismQL function-call LHS predicate gating — seven-position E-QUERY-038/039 plan-time gates, aggregate-in-WHERE enforcement, LOW-006 reserved-keyword exclusion, BC-2.11.019 injection-safety ([#223](https://github.com/drbothen/prism/pull/223))

- MCP query row-shape defects — explicit nulls + error message/suggestion split ([#222](https://github.com/drbothen/prism/pull/222))

- cmd_configure missing X-Admin-Token header — POST /dtu/configure returns 401 ([#225](https://github.com/drbothen/prism/pull/225))

- T13 pre-flight audit reads structuredContent.error.code as authoritative ([G4]/[H8] false FAILs) + SAP-3 wire-level coverage ([#227](https://github.com/drbothen/prism/pull/227))

- exempt script check names (L10, L11+) from ARM5 false-positive ([#231](https://github.com/drbothen/prism/pull/231))

- demo scripts cwd-independence + config-dir-aware guidance

- bump wasmtime 44→47 to clear RUSTSEC-2026-0222

- bump lru 0.17→0.18 to clear RUSTSEC-2026-0253 ([#235](https://github.com/drbothen/prism/pull/235))

- retire stale D-747 LOCKED comments in armis + cyberint sensor TOMLs ([#234](https://github.com/drbothen/prism/pull/234))

- live xDome API fidelity — SAP-2 silent-data-loss fixes, column expansion, device_alert_relations table ([#236](https://github.com/drbothen/prism/pull/236))

- xDome HTTPS transport hardening + sensor error/health-status fidelity (F10 + F9) ([#237](https://github.com/drbothen/prism/pull/237))

- audit_logs time-filter push-down + INDEX eligibility (S-CLAROTY-AUDITLOG-TIMEBOX-001) ([#239](https://github.com/drbothen/prism/pull/239))

- _meta envelope — sensors_queried Err-arm (OBS-1) + has_more invariant (OBS-2) ([#251](https://github.com/drbothen/prism/pull/251))

- deterministic sort_by for 7 Claroty xDome tables — fixes offset-pagination instability (D-001..D-007) ([#252](https://github.com/drbothen/prism/pull/252))

- claroty_vulnerabilities sort_by — published_date replaces adjusted_vulnerability_score ([#256](https://github.com/drbothen/prism/pull/256))

- musl rustup race + CHANGELOG beta.1 section + Release-notes wiring (DEFECT-REL001-MUSL-RUSTUP-COMPONENT-RACE-001) ([#261](https://github.com/drbothen/prism/pull/261))


### Performance

- bundle 7 optimizations + bump all actions to latest SHAs ([#46](https://github.com/drbothen/prism/pull/46))

- nextest profile hardening + build_http_client timeout fix (S-PERF-GATE-001) ([#204](https://github.com/drbothen/prism/pull/204))

- serialize adv_p02 test binary (nextest max-threads=1) to eliminate oversubscription blowup ([#206](https://github.com/drbothen/prism/pull/206))

- serialize bc_2_01_013 test binary + check-ci --profile ci alignment ([#207](https://github.com/drbothen/prism/pull/207))

- cap DTU test-binary concurrency to eliminate nextest oversubscription ([#209](https://github.com/drbothen/prism/pull/209))

- wire DTU clone graceful shutdown so stop() completes promptly ([#210](https://github.com/drbothen/prism/pull/210))

- add nextest wasm-cap + http-cap groups to eliminate WASMtime/wiremock oversubscription (5.4x faster) ([#211](https://github.com/drbothen/prism/pull/211))

- align check, check-fast, iter RUSTFLAGS for fingerprint consistency ([#212](https://github.com/drbothen/prism/pull/212))

- WASMtime compilation cache with degradable boot ([#213](https://github.com/drbothen/prism/pull/213))


### Changed

- rename crowdstrike_session to org_scoped_session_id ([#126](https://github.com/drbothen/prism/pull/126))

- consolidate CredentialRef → types::CredentialRef (close TD-S-PLUGIN-PREREQ-C-001-A sub-fix 1)

- canonical E-CRED-001..010 namespace migration + collision resolution (ADR-035)


