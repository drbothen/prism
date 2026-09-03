# Changelog

All notable changes to Prism are documented here.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [1.0.0] — 2026-09-03

Prism v1.0.0 is the inaugural production release of an ephemeral federated query
engine for MSSP security operations. It exposes a Model Context Protocol (MCP)
server that enables LLM-native analysis across heterogeneous security sensors
(Claroty xDome, CrowdStrike Falcon, Cyberint, Armis) without persisting raw sensor
data. Analysts issue PrismQL queries over live sensor APIs; Prism handles
authentication, multi-tenant isolation, OCSF normalization, query push-down, and
structured audit logging — all in a single operator-deployable binary.

### Added

#### CI/CD and Release Infrastructure

- Full 5-platform release pipeline (`aarch64-apple-darwin`, `x86_64-apple-darwin`,
  `x86_64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`, `x86_64-pc-windows-msvc`)
  with static musl via `cargo-zigbuild`, build-provenance attestations, and
  `checksums.txt` generation (S-0.01, S-REL-001, #1, #228)
- Developer toolchain bootstrap: `just` recipes, `lefthook` pre-commit/push/tag
  hooks, `cargo-nextest` integration, `cargo deny`, `cargo audit`, `cargo semver-checks`
  (S-0.02, #2)
- GitHub Actions CI with 24 required status checks including compile-fail gates,
  fuzz smoke, Kani proof job, WASM32 compile check, `.prx` build, and
  cross-platform test matrix (#1, #46, #241)
- Workspace crate-layout enforcement via `check-crate-layout.sh` CI gate
  (`src/` convention, ADR-012; S-3.5.01, #82)

#### PrismQL Query Engine

- PrismQL (PQL) ephemeral query engine over sensor APIs: SQL-like pipe syntax,
  WHERE/ORDER BY/LIMIT/GROUP BY, column-type gating, and plan-time error taxonomy
  (`E-QUERY-*`) (S-1.11, S-2.08, #14, #61)
- DataFusion integration for in-process query execution over Arrow `RecordBatch`
  results (S-3.02-FOLLOWUP-RUNTIME, #162)
- Dynamic table availability with `TableRegistry` and `E-QUERY-037` plan-time gate
  preventing queries to unavailable sensor tables (S-3.13, #192)
- LIMIT-aware early-stop pagination: `is_truncated` signaling and adapter-boundary
  `early_stop_limit` wiring (ADR-060 §D8; S-ENGINE-LIMIT-EARLY-STOP-001)
- Case-insensitive operators `IEQ`/`IIN`/`INE` with adapter-boundary OCSF enum-label
  normalization (S-PRISMQL-CASE-INSENSITIVE-001, #217)
- Native temporal typing: lenient-parse + AST-walk + `String` coercion for datetime
  literals (ADR-052 §D4 Option A; S-PRISMQL-NATIVE-TEMPORAL-TYPING-001, #214)
- PrismQL grammar remediation for demo-readiness: SQL pipe syntax fixes, MCP
  surface corrections (S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001, #203)
- `prism_describe` MCP teaching surface — schema introspection, reference prompts,
  ADR-042 reload-aware table listing (S-DEMO-PRISMQL-ONBOARDING-001-A, #198)
- `E-QUERY-038` column gate — Tier-2 source column rejection with pedagogical
  enrichments and `normalized_pql` output (S-DEMO-PRISMQL-ONBOARDING-001-B)
- OCSF column-naming and routing: Stage 1 coercion gap closure, Stage 2 push-down
  (ADR-058; S-ADR058-OCSF-COERCION-001, S-ADR058-OCSF-ROUTING-001, #240)
- Enrichment chain: PrismQL `ENRICH` clause, ThreatIntel/NVD dual-path enrichment
  (HttpLookup + WASM plugin infusion), IOC stamping across Cyberint and CrowdStrike,
  typed UDF output with consistent `ColumnType` coercion
  (S-DEMO-ENRICHMENT-PIVOT-001/002/003, S-DEMO-ENRICHMENT-TYPED-OUTPUT-001)
- Full infusion engine: MMDB/CSV/JSON/HttpLookup sources, 3-tier cache, plugin
  runtime wiring, SEC-001 source-size guard (S-1.14-REDO)
- `E-QUERY-042` gate for `Literal::Timestamp` in GROUP BY/ORDER BY
  (ADR-052 §D4 arms 6+7; DEFECT-EQUERY042-GROUPBY-DEADARM-001, #220)
- `E-QUERY-038` plan-time column gate expanded to 14-position binding-context walk
  with 6 suspension rules (BC-2.11.016; FIX-IEQ-ERRPATH-001, #219)

#### Sensor Adapter Framework

- `SpecDrivenSensorAdapter`: TOML-spec-driven sensor abstraction bridging
  `PipelineExecutor` to `AdapterRegistry`, replacing hardcoded per-sensor adapters
  (S-DEMO-001, #166)
- Config-driven sensor TOML specs for all built-in sensors: CrowdStrike, Cyberint,
  Claroty, Armis ship as TOML specs processed by the spec engine (PLUGIN-MIGRATION-001-D, #153)
- `SpecParser` hot reload: runtime spec reloading with `ArcSwap` config snapshots
  (S-1.12, #24)
- Plugin framework: WASM plugin runtime with host-function security boundary,
  PRX plugin format, CrowdStrike OAuth2 refresh-on-401 plugin
  (S-1.15, PLUGIN-MIGRATION-001-E, S-PLUGIN-PREREQ-D, #22, #149, #154)
- `SensorAuth` open trait + `CustomAdapter` deprecation for extensible plugin
  authentication (S-PLUGIN-PREREQ-E, #151)
- `${env.VAR}` interpolation resolution in sensor-spec string fields (S-SPEC-ENV-VAR-001)
- HTTP method whitelist validation: `E-SPEC-025` + BC-2.16.009 Rule 7 (S-SPEC-HTTP-METHOD-VALIDATION-001, #172)
- Unified sensor type: retire `types::SensorSpec` shadow type, unify on
  `spec_parser::SensorSpec` (ADR-030; S-SPEC-TYPE-UNIFICATION-001, #161)
- `SpecDrivenMapper` in `prism-ocsf` replaces 4 hardcoded OCSF mappers
  (PLUGIN-MIGRATION-001-C, #158)
- `WASMtime` compilation cache with degradable boot (S-PERF-GATE-008, #213)

#### Claroty xDome Sensor Suite (14 tables)

- `devices` table with full column set, SAP-2 parity, live structural tests
  (Wave A baseline, fix #236)
- `alerts` and `device_alert_relations` tables (Wave A/B)
- `vulnerabilities` table: 19-column Tier-1/Tier-2 spec, explicit-nulls wire shape,
  SAP-3 E-QUERY-038 gate (S-CLAROTY-VULNS-001, #245)
- `ot_activity_events` table: 21-column spec, native JSON array for
  `related_alert_ids` (S-CLAROTY-OT-EVENTS-001, #246)
- `device_vulnerability_relations` table: 13-column composite PK spec
  (S-CLAROTY-DEVVULNREL-001, #247)
- `servers` and `server_interfaces` tables: 17-col + 10-col spec, null-passthrough,
  `count=null` empty-page halt (S-CLAROTY-SERVERS-001, #248)
- `zones`, `zone_policies`, `firewall_groups`, `firewall_policies` tables: 4 org
  policy TOML blocks, `entity_management/3004`, URL↔envelope asymmetry
  (S-CLAROTY-ORGPOLICY-001, #249)
- `acl_policies` table: pagination-none, mandatory `filter_by`/`policy_acl_syntax`,
  `applied_models` JSON array (S-CLAROTY-ACLPOLICY-001, #250)
- `audit_logs` time-filter push-down: ADR-033 T1 option with `INDEX` eligibility and
  default `>=` time-guard injection (S-CLAROTY-AUDITLOG-TIMEBOX-001, #239)
- HTTPS transport hardening, deterministic `sort_by` for offset-pagination stability
  across all 7 paginated tables (DEFECT-CLAROTY-SORTBY-DETERMINISM-001, #252)

#### CrowdStrike / Cyberint / Armis Sensor Adapters

- CrowdStrike Falcon: multi-region base URL via env var, CrowdStrike `devices` POST
  fan-out with empty MemTable pre-registration and `E-QUERY-043` gate
  (S-DEMO-CROWDSTRIKE-MULTIREGION-001, DEFECT-CSDEVICES-EMPTY-PIPELINE-001, #170, #221)
- CrowdStrike OAuth2 refresh-on-401 PRX WASM plugin (PLUGIN-MIGRATION-001-E, #154)
- Cyberint DTU: `access_token` auth, `StaticCookieAuthProvider`, sensor-spec fidelity
  bundle (S-DTU-CYBERINT-AUTH-FIDELITY-001, #164)
- Armis AQL search endpoint fidelity: DTU `/api/v1/search` push-down, AQL validator
  (multi-occurrence SELECT + single-quote rejection) (S-DEMO-ARMIS-AQL-001, #168)
- OCSF class migration: all sensor TOMLs migrated
  `security_finding` → `detection_finding` (OCSF v1.1; OCSF-CLASS-MIGRATION-001, #174)
- Query push-down (LIMIT + time-window) into `PipelineExecutor`: ADR-033 T1 + Armis
  AQL full wiring (S-DEMO-QUERY-PUSHDOWN-001, #173)

#### Multi-tenant Architecture

- `OrgId(Uuid v7)` canonical org identity newtype with `OrgRegistry` bijective BiMap
  (S-3.1.01, S-3.1.03, #81, #94)
- `OrgSlug` newtype with redacted `Debug`, `new_unchecked` symbol-keyed audit gate
  (S-3.1.02, #93)
- Per-org overlay loading: `OverlayLoader`, `OrgScopedSpecStore`, resolved-spec-map
  threading through `MaterializationContext` and `QueryEngine`
  (S-CONFIG-MULTI-TENANT-OVERRIDE-001, #155)
- Customer config TOML schema + parser + startup validator (S-3.3.01, #92)
- `OrgRegistry` boot from customer config with validate-before-register pattern
  (S-3.3.02, #97)
- Multi-tenant state segregation in all 4 sensor DTU clones: `(OrgId, String)`
  re-keying across Claroty, CrowdStrike, Armis, Cyberint (S-3.2.01–3.2.04, #85–#88)
- Multi-org smoke test: 4-sensor × N-org isolation with seeded DTU data
  (S-DEMO-004, S-DEMO-MULTI-TENANT-DTU-001, #181)
- Per-org DTU instance: N-address binding for overlay testing
  (S-DEMO-MULTI-TENANT-DTU-001)
- `start-multi` subcommand + N-org demo script consolidation
  (S-DEMO-LAUNCHER-CONSOLIDATION-001)

#### MCP Server

- `PrismServer` via `rmcp` 1.7: tool router, structured error responses,
  prompt injection defense, tri-state client scoping (S-5.01-FOLLOWUP-MCP-BOOT,
  S-5.02, #163)
- MCP Resources + Prompts: per-org DI resources, sensor-health prompts,
  `sanitize_for_log` injection defense (S-5.03)
- Sensor Health Subsystem: live probes, `probe_table` routing, `E-SPEC-026`,
  `HealthSummary` structured content (S-5.04, #202)
- `_meta` envelope in query responses: `sensors_queried`, `has_more`, error-arm
  handling (DEFECT-LIVE-ENVELOPE-OBS-001, #251)
- MCP query row-shape: explicit nulls, structured `message`/`suggestion` split in
  error responses (DEFECT-MCP-ROWSHAPE-NULLS-001, #222)
- Write endpoints with verb uniqueness validation (S-1.13, #20)

#### Digital Twin Universe (DTU) Clones

- DTU common infrastructure: shared harness, `DtuMode` validation, fixture-gen
  feature (S-6.06, S-3.7.01, #4, #76)
- Behavioral DTU clones for all 4 primary sensors: Claroty xDome (53 tests),
  CrowdStrike Falcon, Cyberint, Armis (S-6.07–6.09, #9–#11)
- Fixture generators for all 4 sensors: 8 archetypes each, seeded deterministic
  scenario progression (S-3.7.02–3.7.05, S-DEMO-DTU-LIVE-SCENARIO-001-A/B, #76–#80)
- Harness clone route parity: Armis search + Claroty `audit_log` (S-DEMO-HARNESS-CLONE-PARITY-001, #180)
- Secondary DTU clones: NVD CVE API, ThreatIntel, Slack webhook, Jira REST v3,
  PagerDuty Events v2 (S-6.11–6.15, #55–#57)
- Multi-tenant DTU OrgId ingress tagging across Slack, Jira, PagerDuty
  (S-3.2.05–3.2.07, #89–#91)
- Admin token (`X-Admin-Token`) auth on DTU `/dtu/reset` and `/dtu/configure`
  endpoints (W3-FIX-SEC-001/002/005, #113, #119, #125)
- DTU schema derivation: Armis + CrowdStrike Rust types from Go SDK sources
  (S-3.7.00, #75)
- Unified DTU demo server: `prism-dtu-demo-server` multi-clone harness
  (S-6.20, #29)

#### Audit and Credential Subsystems

- `prism-audit`: audit emitter, redaction, `AuditRiskLevel`, specialized audit events
  (credential, vector, flag, token), `org_id`/`org_slug` fields, SHA-256 `aql_hash`
  (S-2.04, S-2.05, S-3.1.07, #58, #59, #96)
- `prism-credentials`: credential CRUD, secret redaction, audit logging,
  `CredentialStoreOrgId` OrgId-keyed namespace (S-1.07, #27)
- `prism credential set/delete` CLI with stdin credential read (no terminal echo),
  demo setup scripts, operator runbook (S-DEMO-003, #176)
- Canonical `E-CRED-001..010` namespace migration + collision resolution (ADR-035;
  S-MAINT-ECRED-TAXONOMY-SYNC-001)

#### Security

- Four-layer prompt injection defense in `prism-security` and `prism-mcp`
  (S-1.10, #16)
- `prism-security`: `CredentialStore` Argon2id file backend, confirmation tokens,
  `RiskTier` gate, single-use tokens, `FeatureFlagEvaluator`
  (S-1.06, S-1.09, #19, #25)
- `rustls-tls` mandatory workspace-wide: eliminate `native-tls` from all `reqwest`
  dependency entries (ADR-050; DEFECT-ADAPTER-TLS-XDOME-LIVE-001)
- `sanitize_for_log` sibling-sweep across `prism-spec-engine` overlay and error paths
  (SEC-PASS4/5/6)
- Customer-config spec path traversal hardening (`CWE-22`/`E-CFG-018`;
  W3-FIX-SEC-003, #114)
- `SecretString` bearer tokens + Armis AQL validator (W2-FIX-I, #69)
- TOML inline-table redaction + constant-time admin token comparison
  (W3-FIX-SEC-004, #122)
- `X-Org-Id` auth enforcement on all 4 DTU clones (`CWE-287`/`CWE-639`/A01;
  W3-FIX-SEC-001, #113)

#### Formal Verification and Testing Infrastructure

- Kani formal proofs: VP-002/003/004 (capability deny-by-default), VP-007–010
  (confirmation tokens), VP-014/015 (query size/depth limits)
  (S-1.03, S-1.09, S-1.11 proofs, #15, #25)
- Fuzz target `vp021_parse_fuzz` for PrismQL grammar
- Perimeter compile-fail gates: `E0432` (prism-query import boundary),
  `E0639` (`#[non_exhaustive]` external match) (S-PLUGIN-PREREQ-A, S-PLUGIN-PREREQ-C)
- `#[non_exhaustive]` discipline: all public TOML-deserialized and API surface types
  annotated; Layer-1 equality CI gate (S-PLUGIN-PREREQ-C, expanded through Wave C)
- Wire-shape assertion discipline: MCP-visible surfaces assert on serialized JSON
  output, enforcing null-not-absent at wire level (DEFECT-MCP-ROWSHAPE-NULLS-001,
  S-CLAROTY-VULNS-001, S-CLAROTY-OT-EVENTS-001 and subsequent)
- T13 pre-flight live-audit script: 106-check coverage matrix for CI/live parity
  (AUDIT-COVERAGE-001, #226)
- E2E subprocess smoke test: all 4 sensors + multi-org isolation + AQL push-down
  (S-DEMO-002, #171)
- `records-lint.sh` mechanical gate: L1/L7/L9/L10 checks across `.factory/` record
  files (TD-VSDD-092)

#### Storage

- `prism-storage`: RocksDB foundation with 19 column families, audit buffer,
  watchdog, denylist, event buffer, dirty bits (S-2.01–2.03, S-2.08, #43, #52–#53)

#### Observability

- Structured event catalog: all `tracing::*!(event_type=…)` emission sites
  registered in BC-2.16.002 canonical catalog with field schema, audit role,
  recurrence policy (PG-LP11-001)
- 18 diagnostic log targets defined in observability architecture

#### Performance

- `cargo-nextest` + per-platform `PROPTEST_CASES` + `mold` linker: CI wall-clock
  optimization (W3-FIX-CI-001, #112)
- Nextest profile hardening: `wasm-cap` + `http-cap` groups eliminate
  WASMtime/wiremock oversubscription (5.4x faster test runs)
  (S-PERF-GATE-001–008, #204–#213)
- `WASMtime` compilation cache with degradable boot (S-PERF-GATE-008, #213)
- Graceful DTU shutdown for prompt `stop()` completion (S-PERF-GATE-005, #210)

### Fixed

- `_meta` envelope: `sensors_queried` Err-arm + `has_more` invariant
  (DEFECT-LIVE-ENVELOPE-OBS-001, #251)
- Deterministic `sort_by` for 7 Claroty xDome paginated tables: fixes
  offset-pagination instability (DEFECT-CLAROTY-SORTBY-DETERMINISM-001, #252)
- Claroty `audit_logs` time-filter push-down with INDEX eligibility
  (fix #239)
- Claroty HTTPS transport hardening + sensor error/health-status fidelity (fix #237)
- Claroty live xDome API SAP-2 silent-data-loss fixes, column expansion,
  `device_alert_relations` table (fix #236)
- CrowdStrike `devices` pipeline empty results: POST fan-out + empty MemTable
  pre-registration + `E-QUERY-043` gate (#221)
- MCP query row-shape: explicit nulls, `message`/`suggestion` split (#222)
- PrismQL function-call LHS predicate gating: `E-QUERY-038`/`E-QUERY-039`
  plan-time gates, aggregate-in-WHERE enforcement (#223)
- `E-QUERY-042` gate for timestamp literals in GROUP BY/ORDER BY (#220)
- Demo setup scripts: cwd-independence + config-dir-aware guidance
- Per-org overlay resolved-spec-map threading through
  `MaterializationContext` + `QueryEngine` (F-LP2-CRIT-001, F-LP2-HIGH-001)
- Retire `ColumnType` shadow enum; re-export `prism_core` canonical
  (ADR-024, #148)
- `CredentialStoreOrgId` false-positive remediation for BC-3.2.002 regression
  (W3-FIX-CREDS-001, #121)
- `OrgSlug::new_unchecked` guarded by symbol-keyed audit test (not `#[cfg(test)]`
  which doesn't propagate to downstream crates)
- Armis AQL multi-occurrence SELECT + single-quote rejection (W2-FIX-L, #72)
- Audit emitter persistence + `evict_expired` backend scan (W2-FIX-H, #68)
- `SecretString` bearer tokens replacing plain-string credential handling (W2-FIX-I, #69)
- `HarnessBuilder` failure scope + Drop grace period (W3-FIX-CODE-001, #116)

### Changed

- `TenantId` renamed to `OrgSlug` across the full workspace (S-3.1.02, #93)
- Sensor auth: replaced 4 named auth modules with TOML spec-driven auth
  (PLUGIN-MIGRATION-001-A/B/C, #156, #157, #158)
- `crowdstrike_session` field renamed to `org_scoped_session_id` (#126)
- All sensor TOML specs migrated from `ocsf_class: security_finding` to
  `detection_finding` (OCSF v1.1; OCSF-CLASS-MIGRATION-001, #174)
- `prism-spec-engine` compile-fail perimeter: 12 sensor-named tests rewritten to
  TOML fixture loading (PLUGIN-MIGRATION-001-F, #160)
- `reqwest` TLS backend: `native-tls` eliminated workspace-wide, all entries use
  `rustls-tls` (ADR-050)

### Security

- Dependency bump: `lru` 0.17 → 0.18 (RUSTSEC-2026-0253, fix #235)
- Dependency bump: `wasmtime` 44 → 47 (RUSTSEC-2026-0222)
- Dependency bump: `crossbeam-epoch` past RUSTSEC-2026-0204
- `X-Org-Id` auth enforcement on all 4 DTU clones: `CWE-287`/`CWE-639`/A01
  (W3-FIX-SEC-001, #113)
- Admin-token uniformity (`X-Admin-Token`) across 5 DTU clones: `CWE-306`
  (W3-FIX-SEC-002/005, #119, #125)
- Customer-config path traversal hardening: `CWE-22`/`E-CFG-018`
  (W3-FIX-SEC-003, #114)
- TOML inline-table redaction + constant-time admin token comparison
  (W3-FIX-SEC-004, #122)
- `rustls-tls` mandatory workspace-wide: eliminates ~65 s macOS Keychain init
  and corporate MITM proxy interception path for sensor API credentials
  (ADR-050; DEFECT-ADAPTER-TLS-XDOME-LIVE-001)
