# Changelog

All notable changes to Prism are documented here.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [1.0.0-beta.2] - 2026-09-09

### Highlights

> **Supported sensor scope for the 1.0.0 beta line: Claroty xDome only.** No product feature changes in this release — beta.2 is a release-infrastructure stabilization. Adapter code for CrowdStrike Falcon, Cyberint, and Armis is present but not supported in this line.

- Nightly pre-release channel: `develop` builds are now published automatically as `vX.Y.Z-nightly.YYYYMMDD` pre-releases on a scheduled cadence; the `edge-nightly` convenience pointer tag tracks the latest nightly for tooling integrations
- Nightly release notes are now categorized (Added / Fixed / etc.) via git-cliff rather than a canned body, matching the format used for stable and beta releases
- Sensor specs ship as a dedicated release asset: each release now publishes `prism-specs-<tag>.tar.gz` containing the versioned Claroty xDome sensor TOML spec alongside the platform binaries
- Install scripts now handle spec placement: `install.sh --spec-dir <dir>` and `install.ps1 -SpecDir <dir>` download, checksum-verify, and place the Claroty spec directly into the configured `spec_dir` — a fresh install boots with the sensor spec discovered without a manual copy or repository clone

### Upgrade Notes

- Existing beta.1 installs are unaffected in behavior; the nightly channel and specs tarball are additive release-infrastructure improvements, not replacements.
- On new or re-installs, use `install.sh --spec-dir <dir>` (Linux/macOS) or `install.ps1 -SpecDir <dir>` (Windows) to place the Claroty xDome sensor spec automatically. Previously, the spec had to be manually copied from the `specs/` directory inside the binary archive.
- No configuration migration is required. There are no breaking changes.

### Added

- git-cliff categorized notes for nightly tags (S-REL-NIGHTLY-NOTES-001) ([#277](https://github.com/drbothen/prism/pull/277))

- ship specs tarball + install-script spec placement (S-REL-SPECS-TARBALL-001) ([#279](https://github.com/drbothen/prism/pull/279))


### Fixed

- use git cliff --latest for nightly notes (AC-008, S-REL-NIGHTLY-NOTES-001) ([#278](https://github.com/drbothen/prism/pull/278))

## [1.0.0-beta.1] - 2026-09-07

### Highlights

> **Supported sensor scope for beta.1: Claroty xDome only.** Adapter code for CrowdStrike Falcon, Cyberint, and Armis is present in the workspace but is **not supported** in beta.1 and returns in a future release.

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

- prism-security + prism-mcp — four-layer prompt injection defense (SS-09) ([#16](https://github.com/drbothen/prism/pull/16))

- sensor-spec-engine write endpoints with verb uniqueness ([#20](https://github.com/drbothen/prism/pull/20))

- prism-security credential store trait + Argon2id file backend ([#19](https://github.com/drbothen/prism/pull/19))

- prism-feature-flags P0 core — runtime gating + analyst audit ([#23](https://github.com/drbothen/prism/pull/23))

- prism-wasm plugin runtime + host-function security boundary ([#22](https://github.com/drbothen/prism/pull/22))

- prism-security confirmation tokens — RiskTier gate, single-use tokens, Kani VP-007..010 ([#25](https://github.com/drbothen/prism/pull/25))

- prism-spec-engine hot reload + runtime management ([#24](https://github.com/drbothen/prism/pull/24))

- prism-credentials CRUD, resolution, secret redaction, and audit logging ([#27](https://github.com/drbothen/prism/pull/27))

- prism-audit — audit emitter + redaction + AuditRiskLevel (Wave 2) ([#58](https://github.com/drbothen/prism/pull/58))

- claroty multi-tenant state segregation — (OrgId, String) re-keying (BC-3.2.001/003) ([#86](https://github.com/drbothen/prism/pull/86))

- prism-customer-config TOML schema + parser + startup validator (BC-3.3.001/002/003/004) ([#92](https://github.com/drbothen/prism/pull/92))

- reload_config detects + warns on DTU mode changes — invariant-preserving rejection (BC-3.2.005) ([#100](https://github.com/drbothen/prism/pull/100))

- S-3.01 PrismQL parser — filter/SQL/pipe modes via Chumsky 0.12 ([#127](https://github.com/drbothen/prism/pull/127))

- PrismQL write parser extensions — BC-2.11.004 v1.4 + perimeter +10 symbols ([#130](https://github.com/drbothen/prism/pull/130))

- PrismQL query materialization — BC-2.11.001/005/006/007/011/012 ([#129](https://github.com/drbothen/prism/pull/129))

- pagination + caching (cursor lifecycle, moka LRU, write invalidation)

- alias system — alias-write feature flag + capability/resolver/store/tools/types modules

- explain query diagnostics — BC-2.11.010

- write execution pipeline — BC-2.04.001/005, BC-2.05.009 (Phase 2 safety + Phase 5 dispatch + structured error taxonomy)

- prism-bin chassis — boot sequence + CLI + signal handlers ([#138](https://github.com/drbothen/prism/pull/138))

- S-PLUGIN-PREREQ-B — Real PipelineExecutor with multi-step fetch, auth refresh, and Structured Event Catalog ([#143](https://github.com/drbothen/prism/pull/143))

- S-PLUGIN-PREREQ-C — TOML grammar extensions (page_size + JSONPath brackets/wildcards + Interpolator escape) + pub-API hardening (30 #[non_exhaustive] types + symbol-keyed allowlist + SensorIdValidationError re-export) ([#144](https://github.com/drbothen/prism/pull/144))

- plugin runtime boot wiring (18 ACs; 25 Red Gate tests; 3-CLEAN converged) ([#149](https://github.com/drbothen/prism/pull/149))

- un-seal SensorAuth + deprecate CustomAdapter + WriteToolInvalidationMap runtime extensibility ([#151](https://github.com/drbothen/prism/pull/151))

- implement per-org overlay loading (BC-2.06.012..016)

- per-org sensor endpoint overlay loading (ADR-029) ([#155](https://github.com/drbothen/prism/pull/155))

- prism-mcp PrismServer — rmcp 1.7, tool router, injection defense ([#163](https://github.com/drbothen/prism/pull/163))

- ${env.VAR} interpolation resolution in sensor-spec string fields

- SpecDrivenSensorAdapter + boot step 9A — bridge PipelineExecutor to AdapterRegistry (closes GAP-002-A) ([#166](https://github.com/drbothen/prism/pull/166))

- HTTP method whitelist validation in sensor spec — E-SPEC-025 + BC-2.16.009 v1.10 Rule 7 (DRIFT-D926-001) ([#172](https://github.com/drbothen/prism/pull/172))

- migrate sensor TOMLs ocsf_class security_finding → detection_finding (OCSF v1.1) ([#174](https://github.com/drbothen/prism/pull/174))

- demo setup scripts + prism credential set/delete CLI + operator runbook ([#176](https://github.com/drbothen/prism/pull/176))

- NormalizePathLayer trailing-slash route fidelity — ADR-031 §D8-b Gap-CL-001 (BC-2.16.013 v1.25)

- OffsetLimit POST-body pagination for Claroty (closes Gap-CL-004)

- start-multi subcommand + N-org demo script consolidation

- enrichment pivot query support for prism-query

- MCP tool routing, structured error responses, tri-state client scoping

- dynamic table availability — TableRegistry + E-QUERY-037 plan-time gate

- full infusion engine (MMDB/CSV/JSON/HttpLookup sources, 3-tier cache, plugin runtime, SEC-001 source-size guard)

- MCP Resources & Prompts — per-org DI resources, sensor-health, prompt sanitization

- ThreatIntel/NVD dual-path enrichment (HttpLookup + WASM plugin infusion)

- MCP teaching surface — prism_describe + reference prompts + ADR-042 reload-aware schema

- PrismQL Query Engine L4 — E-QUERY-038 column gate + pedagogical enrichments + normalized_pql ([#198](https://github.com/drbothen/prism/pull/198))

- ENRICH-1/2/3/4-B integration — PrismQL enrichment chain end-to-end

- prism-mcp Sensor Health Subsystem — live probes, probe_table routing, E-SPEC-026, HealthSummary ([#202](https://github.com/drbothen/prism/pull/202))

- ADR-052 §D4 Option A temporal typing — lenient-parse + AST-walk + String coercion ([#214](https://github.com/drbothen/prism/pull/214))

- typed enrichment-UDF output with consistent ColumnType coercion

- PrismQL case-insensitive operators IEQ/IIN/INE with adapter-boundary OCSF enum-label normalization ([#217](https://github.com/drbothen/prism/pull/217))

- release.yml repair — 5-platform release pipeline (DEF-REL-001..004, prerelease, static musl, pinned toolchain) ([#228](https://github.com/drbothen/prism/pull/228))

- ADR-058 Stage 1 — column coercion gap closure (EC-016-013-007/008/009/030) ([#240](https://github.com/drbothen/prism/pull/240))

- OCSF column-naming/routing Stage 2 (§J1–J5, query-surface resolution, push-down)

- LIMIT-aware early-stop pagination + is_truncated signaling (ADR-060 §D8)

- Claroty xDome vulnerabilities table — TOML block, 19-column Tier-1/Tier-2 spec, live structural tests (Wave A G1) ([#245](https://github.com/drbothen/prism/pull/245))

- Claroty xDome OT activity events table — TOML block, 21-column Tier-1/Tier-2 spec, live structural tests (Wave A G2) ([#246](https://github.com/drbothen/prism/pull/246))

- Claroty xDome device_vulnerability_relations table — 13-column Tier-1/Tier-2 spec, composite PK, live structural tests (Wave B G3) ([#247](https://github.com/drbothen/prism/pull/247))

- Claroty xDome servers + server_interfaces tables — 17-col + 10-col TOML spec, live structural tests (Wave C G4) ([#248](https://github.com/drbothen/prism/pull/248))

- Claroty xDome org policy tables — 4 TOML blocks, entity_management/3004, 8 Json cols, fw URL↔envelope asymmetry, live structural tests (Wave C G5) ([#249](https://github.com/drbothen/prism/pull/249))

- Claroty xDome ACL policies table — 14th Claroty table, pagination-none, mandatory filter_by/policy_acl_syntax, Wave C G6 ([#250](https://github.com/drbothen/prism/pull/250))

- install.sh + install.ps1 — checksum-verified 5-platform consumer install scripts ([#254](https://github.com/drbothen/prism/pull/254))

- version identity — 1.0.0-dev reset + PRISM_VERSION injection + version-agnostic docs ([#262](https://github.com/drbothen/prism/pull/262))

- agent-facing version identity — prism-mcp serverInfo + prism-spec-engine UA (S-REL-AGENT-VERSION-001) ([#263](https://github.com/drbothen/prism/pull/263))


### Fixed

- query-core — E-QUERY taxonomy splits, watchdog grace period, cache/hot-reload fixes

- mcp-boot lane — fail-closed write audit, reload_config WriteTool, capability fields, security polish ([#184](https://github.com/drbothen/prism/pull/184))

- E-QUERY-038 plan-time column gate — 14-position binding-context walk + 6 suspension rules (BC-2.11.016 v1.25) ([#219](https://github.com/drbothen/prism/pull/219))

- E-QUERY-042 gate for Literal::Timestamp in GROUP BY/ORDER BY (ADR-052 §D4 v1.11 arms 6+7) ([#220](https://github.com/drbothen/prism/pull/220))

- PrismQL function-call LHS predicate gating — seven-position E-QUERY-038/039 plan-time gates, aggregate-in-WHERE enforcement, LOW-006 reserved-keyword exclusion, BC-2.11.019 injection-safety ([#223](https://github.com/drbothen/prism/pull/223))

- MCP query row-shape defects — explicit nulls + error message/suggestion split ([#222](https://github.com/drbothen/prism/pull/222))

- cmd_configure missing X-Admin-Token header — POST /dtu/configure returns 401 ([#225](https://github.com/drbothen/prism/pull/225))

- live xDome API fidelity — SAP-2 silent-data-loss fixes, column expansion, device_alert_relations table ([#236](https://github.com/drbothen/prism/pull/236))

- xDome HTTPS transport hardening + sensor error/health-status fidelity (F10 + F9) ([#237](https://github.com/drbothen/prism/pull/237))

- audit_logs time-filter push-down + INDEX eligibility (S-CLAROTY-AUDITLOG-TIMEBOX-001) ([#239](https://github.com/drbothen/prism/pull/239))

- _meta envelope — sensors_queried Err-arm (OBS-1) + has_more invariant (OBS-2) ([#251](https://github.com/drbothen/prism/pull/251))

- deterministic sort_by for 7 Claroty xDome tables — fixes offset-pagination instability (D-001..D-007) ([#252](https://github.com/drbothen/prism/pull/252))

- claroty_vulnerabilities sort_by — published_date replaces adjusted_vulnerability_score ([#256](https://github.com/drbothen/prism/pull/256))

- musl rustup race + CHANGELOG beta.1 section + Release-notes wiring (DEFECT-REL001-MUSL-RUSTUP-COMPONENT-RACE-001) ([#261](https://github.com/drbothen/prism/pull/261))


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


### Performance

- WASMtime compilation cache with degradable boot ([#213](https://github.com/drbothen/prism/pull/213))


### Changed

- canonical E-CRED-001..010 namespace migration + collision resolution (ADR-035)


[Unreleased]: https://github.com/drbothen/prism/compare/v1.0.0-beta.2...HEAD
[1.0.0-beta.2]: https://github.com/drbothen/prism/compare/v1.0.0-beta.1...v1.0.0-beta.2
[1.0.0-beta.1]: https://github.com/drbothen/prism/releases/tag/v1.0.0-beta.1
