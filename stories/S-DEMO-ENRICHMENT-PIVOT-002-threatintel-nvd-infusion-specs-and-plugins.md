---
document_type: story
story_id: S-DEMO-ENRICHMENT-PIVOT-002
title: "ThreatIntel/NVD Infusion Specs and .prx Plugins — Grounded Against DTU Clone Routes"
wave: 5
epic_id: E-DEMO
priority: P2
status: draft
version: "1.2"
level: "L4"
producer: story-writer
timestamp: "2026-06-12T00:00:00Z"
created: "2026-06-12"
modified: "2026-06-17T00:00:00Z"
tdd_mode: strict
subsystems: [SS-19, SS-17, SS-01]
# Subsystem anchor justifications:
#   SS-19 (Infusion Enrichment Framework) owns .infusion.toml specs and the InfusionLoader
#   pipeline they feed into per ARCH-INDEX Subsystem Registry.
#   SS-17 (Plugin Runtime / WASM) owns .prx WASM plugin authorship (cargo-component or
#   equivalent) and the plugin ABI they implement.
#   SS-01 (Sensor Adapters) is included because this story grounds the infusion specs
#   against prism-dtu-threatintel and prism-dtu-nvd route surfaces (DTU-backed enrichment
#   per ADR-028/ADR-031 DTU grounding requirement, WO-D1109 §Q1).
target_module: prism-spec-engine
crates_touched: [prism-spec-engine, prism-dtu-threatintel, prism-dtu-nvd]
# new crates (out-of-workspace, standalone [workspace] in each Cargo.toml, excluded from root
#   Cargo.toml via exclude = [...] — same pattern as crates/plugins/ocsf-complex-transforms):
#   crates/plugins/prism-threatintel-infusion — WASM guest cdylib (wasm32-wasip1) calling
#     prism-dtu-threatintel HTTP API via host WIT import host.http-request
#   crates/plugins/prism-nvd-infusion — same pattern for NVD
# BC status: pending PO authorship
# BC-2.19.001 governs infusion spec loading and UDF registration (nearest anchor).
# No dedicated BC exists yet for plugin-backed DTU-grounded infusion specs.
# PO should anchor new BCs or confirm BC-2.19.001 + BC-2.06.020 cover this story at
# materialization time. BC-2.06.020 is relevant because this story exercises the
# ScenarioEntityCatalog IOC/CVE injection that BC-2.06.020 governs from the consumer side.
behavioral_contracts: [BC-2.19.001]
# BC array propagation note: BC-2.19.001 is cited by AC-001 and AC-002 (bidirectional
# trace satisfied for the existing BC). BC-2.06.020 would be the second anchor if PO
# confirms scope overlap — not listed until confirmed.
verification_properties: []
depends_on:
  - S-DEMO-ENRICHMENT-PIVOT-001
  # Dependency anchor: 001 delivers plugin_bridge::enrich_via_plugin operational,
  # InfusionLoader for type="plugin" specs, and DataFusion UDF registration wiring.
  # Without these, the .infusion.toml specs authored in this story cannot be loaded
  # or executed — the integration tests in this story would fail to build.
blocks:
  - S-DEMO-ENRICHMENT-PIVOT-003
  # Blocks anchor: 003 requires threat_intel and nvd enrichment UDFs to be operational
  # so the canonical pivot queries (| enrich threat_intel(ioc.value),
  # | enrich nvd(device_cves_first)) can be validated against the demo server.
points: 8
# Points justification:
#   1. threatintel.infusion.toml grounded against prism-dtu-threatintel route surface: 1 pt
#   2. nvd.infusion.toml grounded against prism-dtu-nvd route surface: 1 pt
#   3. prism-threatintel-infusion .prx plugin (WASM or cargo-component):
#      HTTP call to DTU endpoint + response parsing: 3 pts
#   4. prism-nvd-infusion .prx plugin (same pattern for NVD): 2 pts
#   5. Integration test with demo server: 1 pt
#   Total: 8 pts
estimated_days: 3
risk: HIGH
# Risk justification: two new WASM plugin crates (out-of-workspace) introduce
# wasm32-wasip1 cross-compilation complexity via Justfile pipeline
# (wasm-tools 1.248.0, wit-bindgen 0.51, wasmtime 44 component-encoding alignment
# must be verified as a pre-flight step). DTU grounding requires SAP-2 verification
# against actual prism-dtu-threatintel and prism-dtu-nvd types.rs and routes/ before
# writing any TOML spec. The WIT interface for the .prx ABI (CAP-032, AD-019) must be
# read before implementing. WASM guests cannot use reqwest/tokio — HTTP through host WIT.
red_gate_tests: 15
estimated_passes: "3-4 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "DTU grounding (ADR-028/ADR-031, SAP-2, U10/U12/U13): BEFORE writing any TOML spec, read
     crates/prism-dtu-threatintel/src/routes/lookup.rs (confirmed 2026-06-12: three separate
     routes — GET /v3/ip/:ip, GET /v3/domain/:domain, GET /v3/hash/:hash — NO unified
     /threatintel/lookup/{v} endpoint). Plugin must dispatch on input-type to the correct path.
     Read crates/prism-dtu-nvd/src/routes/cves.rs (confirmed 2026-06-12: single route
     GET /rest/json/cves/2.0 with ?cveId= query param). Every TOML column MUST correspond to
     a DTU response field. Column in TOML with no DTU equivalent = P1 CRITICAL."
  - "ThreatIntel response shape (U11, confirmed 2026-06-12 lookup.rs): response contains
     threat_score (integer), threat_is_known_malicious (bool), threat_sources (JSON ARRAY
     of source strings — NOT threat_source singular string). Declare threat_sources as
     Json-typed column (faithful array representation). Also present: greynoise_classification
     (String), abuseipdb_confidence_score (Integer), virustotal_detections (Integer)."
  - "ThreatIntel auth (U15, confirmed 2026-06-12 lookup.rs lines 20-53): API key via
     ?key= query param OR Authorization: Bearer <token> header — NOT X-Admin-Token
     (X-Admin-Token is admin-surface only). The .infusion.toml credential reference must
     wire to the correct auth mechanism. Use keyring reference path (AD-017); no inline values."
  - "NVD auth (U15): apiKey query param; NVD DTU uses ?apiKey=<key> (CveQueryParams,
     cves.rs). Wire .infusion.toml credential to the correct NVD apiKey param."
  - "NVD CVSS wire names (U12/U13, confirmed 2026-06-12 types.rs — all serde rename_all=camelCase):
     envelope: vulnerabilities[].cve; CveRecord: id (not cve_id), metrics.cvssMetricV31[0].cvssData.baseScore (f64),
     .baseSeverity (String), .vectorString (String). Rust fields: id/metrics/cvss_metric_v31/cvss_data/
     base_score/base_severity/vector_string — camelCase wire names via serde(rename_all=camelCase)."
  - "WASM guest HTTP (U9): WASM guests CANNOT use reqwest or tokio (no sockets in sandbox —
     research-confirmed). HTTP goes through the host WIT import host.http-request
     (prism-infusion-plugin.wit; host_functions.rs:59, allowlist + HOST reqwest client).
     The 30s timeout convention applies to the HOST client, not the guest. Mirror
     crates/plugins/ocsf-complex-transforms/Cargo.toml for guest dependency constraints —
     no reqwest, no tokio in guest Cargo.toml."
  - "Toolchain (U14/Ruling 4): canonical build pipeline is Justfile —
     cargo build --target wasm32-wasip1 --release → wasm-tools component new --adapt
     wasi_snapshot_preview1.wasm → .prx (wasm-tools 1.248.0, wit-bindgen 0.51).
     Do NOT use cargo-component (not the repo pattern). Add Justfile recipe per
     build-plugin-crowdstrike-oauth2 template. Pre-flight: verify wit-bindgen 0.51 /
     wasm-tools 1.248.0 / wasmtime 44 component-encoding alignment against wasmtime 44
     changelog before implementing."
  - "Plugin crate location (U16/Ruling 3): crates/plugins/prism-threatintel-infusion/ and
     crates/plugins/prism-nvd-infusion/ — each with standalone [workspace] in Cargo.toml,
     excluded from root Cargo.toml exclude list (NOT workspace members — same pattern as
     crates/plugins/ocsf-complex-transforms). Justfile recipe required per pattern."
  - "Infusion spec location (U16, confirmed loader.rs:45): {config_dir}/infusions/*.infusion.toml.
     Match the canonical TOML table layout from
     crates/prism-spec-engine/fixtures/threat_intel_plugin.infusion.toml and InfusionSpec
     deserialization at infusion/mod.rs:295-319 — verify [source]/[plugin] table names and
     key names before writing."
  - "NVD scope note (U17/Ruling 1b): Armis DTU fixture generator projects first CVE ID into
     scalar column device_cves_first (String). The enrich path is enrich nvd(device_cves_first).
     Armis sensor TOML declares the column. This story (002) handles TOML + grounding;
     the generator projection task belongs to S-DEMO-ENRICHMENT-PIVOT-003 per ruling."
traces_to: [D-1109, WO-D1109]
supersedes: []
---

# S-DEMO-ENRICHMENT-PIVOT-002: ThreatIntel/NVD Infusion Specs and .prx Plugins

Author the `threatintel.infusion.toml` and `nvd.infusion.toml` infusion specs — grounded
against the actual DTU clone route surfaces per ADR-028/ADR-031 (DTU=True-DTU fidelity
principle) — and implement the corresponding `.prx` WASM plugins that call the DTU HTTP
endpoints.

**Sequencing context (D-1109, WO-D1109):** Slots AFTER S-DEMO-ENRICHMENT-PIVOT-001
(plugin bridge operational) and BEFORE S-DEMO-ENRICHMENT-PIVOT-003 (IOC stamping + pivot query).

**DTU grounding requirement (WO-D1109 §Story 2):** The infusion TOML specs MUST be
grounded against the actual DTU clone route surface, not assumed production API URLs.
Use the DTU clone routes as the source of truth for endpoint paths and response schemas.
SAP-2 applies: adversary will read `prism-dtu-threatintel/src/routes/` and
`prism-dtu-nvd/src/routes/` to verify column parity.

---

## Narrative

As an analyst running the capstone demo (T13), I want `| enrich threat_intel(ioc.value)`
and `| enrich nvd(device_cves_first)` to resolve against live DTU HTTP enrichment services —
not static files — so that the demo faithfully represents the production enrichment
architecture where analyst queries resolve IOCs against a threat intelligence backend and
CVEs against the NVD CVSS database.

NOTE (U17/Ruling 1b): the NVD enrich field is `device_cves_first` (a scalar String column
projected from Armis device records by the fixture generator in STORY-003), NOT
`device_cves[0]` (bracket-index not supported by FieldPath). This story declares the
`device_cves_first` column in the Armis sensor TOML; STORY-003 implements the generator
projection.

---

## Behavioral Contracts

| BC | Title | Key Clauses |
|----|-------|-------------|
| BC-2.19.001 v? | Infusion Spec Loading — Each Field Registers Exactly One DataFusion Scalar UDF | Postcondition: each field in `[[infusion.fields]]` produces exactly one `InfusionUdfDescriptor` registered in `SessionContext` |

---

## Acceptance Criteria

### AC-001 — threatintel.infusion.toml parses and loads as plugin-type infusion spec
(traces to BC-2.19.001 postcondition — each field registers exactly one UDF descriptor)

Given `{config_dir}/infusions/threatintel.infusion.toml` (U16: location per InfusionLoader
loader.rs:45 — NOT repo-root specs/infusions/) with:
- `source.type = "plugin"`
- `plugin_ref = "threatintel-lookup.prx"`
- `[[infusion.fields]]` declaring `threat_is_known_malicious` (Boolean), `threat_score` (Integer),
  `threat_sources` (Json — array of source strings; confirmed 2026-06-12 lookup.rs response shape)
- TOML table layout matching crates/prism-spec-engine/fixtures/threat_intel_plugin.infusion.toml
  and InfusionSpec deserialization at infusion/mod.rs:295-319

when `InfusionLoader::load_all` runs against the infusions directory,
then the returned `InfusionRegistry` contains 3 `InfusionUdfDescriptor` entries with the
declared field names (threat_is_known_malicious, threat_score, threat_sources), and
`registry.is_api_backed("threat_score")` returns `true`.

Red Gate: `test_enrichment_pivot_002_threatintel_toml_loads_and_registers_3_udfs`

### AC-002 — nvd.infusion.toml parses and loads as plugin-type infusion spec
(traces to BC-2.19.001 postcondition — each field registers exactly one UDF descriptor)

Given `specs/infusions/nvd.infusion.toml` with:
- `source.type = "plugin"`
- `plugin_ref = "nvd-lookup.prx"`
- `[[infusion.fields]]` declaring `cvss_base_score` (Float), `cvss_severity` (String),
  `cvss_vector` (String)

grounded against `prism-dtu-nvd` route `GET /rest/json/cves/2.0?cveId=<id>` response shape
(confirmed 2026-06-12 from cves.rs + types.rs, all serde rename_all=camelCase):
envelope `vulnerabilities[0].cve`; CveRecord wire field `id` (NOT cve_id);
`metrics.cvssMetricV31[0].cvssData.baseScore` (f64 → maps to cvss_base_score Float),
`metrics.cvssMetricV31[0].cvssData.baseSeverity` (String → cvss_severity),
`metrics.cvssMetricV31[0].cvssData.vectorString` (String → cvss_vector).
NOTE (U17/Ruling 1b): the enrich input field is `device_cves_first` (scalar projected by
STORY-003 Armis generator), NOT `device_cves[0]`.

when `InfusionLoader::load_all` runs,
then the returned `InfusionRegistry` contains 3 `InfusionUdfDescriptor` entries for NVD fields
and `registry.is_api_backed("cvss_base_score")` returns `true`.

Red Gate: `test_enrichment_pivot_002_nvd_toml_loads_and_registers_3_udfs`

### AC-003 — threatintel-lookup.prx plugin calls DTU lookup endpoint and returns enrichment fields
(traces to BC-2.19.001 postcondition — plugin-type source executes via plugin bridge)

Given the `prism-threatintel-infusion` plugin compiled and loaded,
when `PluginInfusionSource::enrich_single` is called with an IOC value from
`ScenarioEntityCatalog.ioc_ips[0]` (a known-Malicious scenario IOC pre-populated in the
DTU `fixture_registry`),
then the plugin dispatches on input-type to the correct DTU route — for an IP: `GET /v3/ip/:ip`
(confirmed 2026-06-12 lookup.rs:162); for a domain: `GET /v3/domain/:domain` (lookup.rs:187);
for a hash: `GET /v3/hash/:hash` (lookup.rs:214) — and returns a response containing
`threat_is_known_malicious = true` and `threat_score >= 75`.
Auth: `?key=<api_key>` query param OR `Authorization: Bearer <token>` header (NOT X-Admin-Token).
Response field `threat_sources` is a JSON array (NOT `threat_source` singular string).

Integration test requires the demo server running with `scenario.enabled = true`.

Red Gate: `test_enrichment_pivot_002_threatintel_plugin_resolves_scenario_ioc_as_malicious`

### AC-004 — nvd-lookup.prx plugin calls DTU CVE endpoint and returns CVSS fields
(traces to BC-2.19.001 postcondition — plugin-type source executes via plugin bridge)

Given the `prism-nvd-infusion` plugin compiled and loaded,
when `PluginInfusionSource::enrich_single` is called with the CVE ID from
`ScenarioEntityCatalog.device_cves[0]` (pre-populated in NVD `cve_registry` with
`cvss_base_score = 8.1`, `cvss_severity = "HIGH"`),
then the plugin calls `GET /rest/json/cves/2.0?cveId=<cve_id>` on the DTU HTTP endpoint
(confirmed 2026-06-12 cves.rs — NOT /nvd/cves/{id}; auth via ?apiKey= query param)
and parses `vulnerabilities[0].cve.metrics.cvssMetricV31[0].cvssData.baseScore` (f64)
and `baseSeverity` (String) and `vectorString` (String) from the camelCase response envelope,
returning `cvss_base_score >= 7.0` and `cvss_severity = "HIGH"`.

Integration test requires the demo server running with `scenario.enabled = true`.

Red Gate: `test_enrichment_pivot_002_nvd_plugin_resolves_scenario_cve_high_cvss`

### AC-005 — | enrich threat_intel(ioc_value) returns Malicious for scenario IOCs
(traces to BC-2.19.001 postcondition — pipe stage enrich produces declared output columns)

Given a Cyberint alerts query result (from demo server at stage >= 3) containing alert records
with an `ioc_value`-compatible field (interim field name; real field `ioc.value` is added by
S-DEMO-ENRICHMENT-PIVOT-003),
when `| enrich threat_intel(ioc_value)` pipe stage executes against the demo server,
then the result set includes `threat_is_known_malicious`, `threat_score`, `threat_sources`
columns and scenario IOCs show `threat_is_known_malicious = true`.

NOTE: the output column is `threat_sources` (Json array), NOT `threat_source` (String).
This AC uses an intermediate field name; STORY-003 updates the field reference to `ioc.value`.

Red Gate: `test_enrichment_pivot_002_enrich_threatintel_pipe_stage_returns_malicious_for_scenario_iocs`

### AC-007 — UDF name validation rejects non-identifier characters at parse time (DRIFT-PIVOT-UDFNAME-VALIDATION-001 — SEC-001 CWE-20)
(traces to BC-2.19.001 precondition — InfusionSpec is structurally valid before UDF registration)

Given an `[[infusion.fields]]` entry in any `.infusion.toml` with a `name` containing
a character outside the `[a-zA-Z][a-zA-Z0-9_]*` identifier pattern — e.g.,
`name = "threat; DROP TABLE"`, `name = " leading_space"`, `name = "has-hyphen"`,
`name = "1starts_with_digit"`, `name = ""` (empty) — when `InfusionLoader::parse`
processes that spec, then it returns `Err(InfusionError::InvalidFieldSpec { field: <name>,
spec_path: <path>, message: "field name must match [a-zA-Z][a-zA-Z0-9_]* ..." })`
and no `InfusionUdfDescriptor` is registered for any field in that spec.

Validation is applied at parse time (in `InfusionLoader::parse`, before `SessionContext`
UDF registration) so malformed names never reach DataFusion.

Valid names accepted: `threat_is_known_malicious`, `cvss_base_score`, `field1`, `THREAT_SCORE`.

Red Gate (unit, prism-spec-engine):
`test_enrichment_pivot_002_sec001_udf_name_rejects_sql_injection_chars`
`test_enrichment_pivot_002_sec001_udf_name_rejects_leading_digit`
`test_enrichment_pivot_002_sec001_udf_name_accepts_valid_identifiers`

### AC-008 — PluginInfusionSource.config is not publicly readable (DRIFT-PIVOT-PLUGINCONFIG-PUB-FIELD-001 — SEC-002 CWE-200)
(traces to BC-2.19.001 invariant — credential data does not leak through public API surface)

Given `PluginInfusionSource` in `crates/prism-spec-engine/src/infusion/plugin_bridge.rs`,
the `config` field (which will be populated with resolved credentials in this story)
MUST NOT be `pub`. It must be narrowed to `pub(crate)` or have visibility limited via
an explicit accessor before credentials are populated in PIVOT-002.

At the time PIVOT-002 wires real credentials into `PluginInfusionSource::new(...)`,
the `config` field visibility MUST be `pub(crate)` so that external crates cannot read
resolved credential values through a struct field access.

Additionally: `plugin_id` and `runtime` fields on `PluginInfusionSource` are currently
`pub`. As they do not contain sensitive data, they MAY remain public; however this AC
REQUIRES `config: pub(crate)` before any credential is passed in.

Red Gate (compile-fail or unit, prism-spec-engine):
`test_enrichment_pivot_002_sec002_plugin_infusion_source_config_not_pub`

NOTE: a compile-fail test in `tests/external/perimeter-violation/` is the canonical
enforcement pattern (ADR per S-PLUGIN-PREREQ-A). If a compile-fail test is not feasible
for `pub(crate)` visibility (it only applies to external crate access), an in-module
unit test asserting the field is inaccessible via `std::mem::offset_of!` or a doc-comment
audit is acceptable as a compensating control; document the rationale.

### AC-009 — SandboxViolation URL is not logged at WARN in analyst-visible output (DRIFT-PIVOT-SANDBOXVIOLATION-URL-LOG-001 — SEC-003 CWE-209)
(traces to BC-2.19.001 invariant — plugin errors do not disclose server internals to analysts)

Given `PluginInfusionSource::enrich_single` in `plugin_bridge.rs`, when the underlying
`PluginRuntime::enrich_single` call returns `Err(PluginError::SandboxViolation { url, .. })`,
the `url` field (which may contain the resolved DTU endpoint address when the DTU URL is
missing from `allowed_urls`) MUST NOT be included in WARN-level tracing output that
surfaces in analyst-visible MCP error responses.

Required fix: in `map_plugin_error_to_infusion_error` (or before the WARN call in
`enrich_single`), either:
  (a) Redact the URL field: emit it at DEBUG level only, or replace with `<redacted>` in
      the WARN log, OR
  (b) Do not include the `url` string in the `InfusionError::MissingRequiredField { field }`
      message constructed by `map_plugin_error_to_infusion_error`.

The `plugin_id` MAY appear in WARN logs (it identifies the plugin configuration, not
a network address). The `url` field MUST be redacted or demoted to DEBUG before PIVOT-002
wires real DTU endpoint addresses into the allowlist.

Red Gate (unit, prism-spec-engine):
`test_enrichment_pivot_002_sec003_sandbox_violation_url_not_in_warn_log`

NOTE: use `tracing_test` or capture WARN spans and assert the `url` value does not appear
in the formatted WARN output. The DEBUG-level emission (for operator diagnostics) is
acceptable and NOT gated.

### AC-010 — spawn_blocking gate: WASM plugin call does not block the async runtime (DRIFT-PIVOT-PLUGINID-INFUSIONID-001 SEC-001 sync-WASM gate — CWE-400)
(traces to BC-2.19.001 postcondition — plugin-type source executes without blocking tokio runtime)

Given `PluginInfusionSource::enrich_single` dispatches synchronously into the wasmtime
WASM runtime via `PluginRuntime::enrich_single`, and `InfusionAsyncUdf::invoke_with_args`
(in `prism-query`) calls this synchronous operation from an async DataFusion context:

The WASM call MUST be wrapped in `tokio::task::spawn_blocking` (or migrate
`InfusionSource::enrich_single` to an async trait) before any PIVOT-002 or PIVOT-003
plugin-bridge wiring merges to the production boot path.

Rationale: synchronous WASM execution on a tokio worker thread blocks the thread for the
duration of the WASM call (up to host HTTP timeout, 30s). Under concurrent query load this
exhausts the tokio thread pool (CWE-400). The security review of PR #189 (D-1179)
elevated this to a MANDATORY gate.

Acceptable implementations:
  (a) `spawn_blocking`: wrap the synchronous `runtime.enrich_single(...)` call in a
      `tokio::task::spawn_blocking(|| runtime.enrich_single(...)).await?` in the
      DataFusion UDF's async `invoke_with_args` (prism-query side), OR
  (b) Async trait: migrate `InfusionSource::enrich_single` to `async fn enrich_single`
      (requires updating all InfusionSource implementations — MMDB/CSV/etc. gain
      trivial async wrapping; WASM source gains `spawn_blocking` internally).

Implementation (a) is preferred for minimum-scope PIVOT-002 delivery; (b) is the
S-1.14-REDO full-engine approach and may be deferred if (a) is implemented here.

Red Gate (integration or unit, prism-spec-engine or prism-query):
`test_enrichment_pivot_002_sec001_wasm_enrich_wraps_spawn_blocking`

NOTE: if `InfusionAsyncUdf` already uses `spawn_blocking` from PIVOT-001, verify and
cite the location; close this gate with a code-pointer in the PR. If not present,
implement it in this story.

### AC-011 — plugin_ref path is canonicalized and restricted to the plugin directory (DRIFT-PIVOT-PLUGINPATH-TRAVERSAL-001 — SEC-003 CWE-22)
(traces to BC-2.19.001 precondition — plugin_ref paths are confined to the designated plugin directory)

Given an `InfusionSpec` with `plugin_ref = "../../etc/passwd.prx"` or any path
containing `..` components, when `InfusionLoader` or the PIVOT-002 plugin loading
code processes the `plugin_ref` value to resolve the `.prx` file path, then:

  1. `std::fs::canonicalize` is called on the resolved path BEFORE any file I/O.
  2. The canonicalized path is verified to share the same `starts_with(plugin_dir)` prefix
     as the configured plugin directory. If not, `InfusionError::InvalidFieldSpec` (or a
     new `InfusionError::PluginPathViolation`) is returned and no file I/O is performed.
  3. A relative path that stays within the plugin directory (e.g., `subdir/plugin.prx`) is
     accepted after canonicalization confirms it is within bounds.

Red Gate (unit, prism-spec-engine):
`test_enrichment_pivot_002_sec003_path_traversal_rejected_for_dotdot_plugin_ref`
`test_enrichment_pivot_002_sec003_path_within_plugin_dir_accepted`

### AC-012 — load_all errors do not disclose absolute filesystem paths in MCP responses (DRIFT-PIVOT-LOADALL-PATH-DISCLOSURE-001 — SEC-002 CWE-209)
(traces to BC-2.19.001 invariant — error messages surfaced to callers do not leak server paths)

Given `InfusionLoader::load_all` processes a directory of `.infusion.toml` files and
encounters a parse error on one file, when the resulting `InfusionError` is propagated
through to an MCP tool response (e.g., as part of a PrismError message), then the
`spec_path` field in the error MUST NOT contain the absolute filesystem path of the server
(e.g., `/home/analyst/.prism/infusions/bad.infusion.toml`).

Required sanitization: strip absolute path prefix from `InfusionError` messages before
they surface in MCP JSON. Acceptable forms:
  - Filename only: `bad.infusion.toml`
  - Relative path from config dir: `infusions/bad.infusion.toml`
  - Redacted path: `<infusions-dir>/bad.infusion.toml`

Internal error formatting (tracing at DEBUG/INFO for operator diagnostics) MAY retain
the full absolute path. Only the MCP-surfaced error string must be sanitized.

Red Gate (unit, prism-spec-engine):
`test_enrichment_pivot_002_sec002_load_all_error_does_not_leak_absolute_path`

### AC-006 — | enrich nvd(device_cves_first) returns HIGH CVSS for scenario CVEs
(traces to BC-2.19.001 postcondition — pipe stage enrich produces declared output columns)

Given an Armis devices query result (from demo server at stage >= 4 Containment, when
`device_cves = true` per BC-2.06.019 PC-2) containing device records with the scalar column
`device_cves_first` (projected by the Armis fixture generator in STORY-003; this story
declares the column in the Armis sensor TOML),
when `| enrich nvd(device_cves_first)` pipe stage executes against the demo server,
then the result set includes `cvss_base_score`, `cvss_severity`, `cvss_vector` columns
and scenario CVEs show `cvss_base_score >= 7.0` and `cvss_severity = "HIGH"`.

NOTE (U17/Ruling 1b): `device_cves[0]` bracket-index is NOT supported by FieldPath.
The correct field is `device_cves_first` (scalar String). The generator projection is
scoped to STORY-003; this story scopes the TOML column declaration and grounding.

Red Gate: `test_enrichment_pivot_002_enrich_nvd_pipe_stage_returns_high_cvss_for_scenario_cves`

---

## Red Gate Test Plan

| # | Test Name | Crate | BC Clause | Type | DRIFT Item |
|---|-----------|-------|-----------|------|------------|
| 1 | `test_enrichment_pivot_002_threatintel_toml_loads_and_registers_3_udfs` | prism-spec-engine | BC-2.19.001 postcondition | unit | — |
| 2 | `test_enrichment_pivot_002_nvd_toml_loads_and_registers_3_udfs` | prism-spec-engine | BC-2.19.001 postcondition | unit | — |
| 3 | `test_enrichment_pivot_002_threatintel_plugin_resolves_scenario_ioc_as_malicious` | prism-spec-engine integration | BC-2.19.001 postcondition | integration (demo server) | — |
| 4 | `test_enrichment_pivot_002_nvd_plugin_resolves_scenario_cve_high_cvss` | prism-spec-engine integration | BC-2.19.001 postcondition | integration (demo server) | — |
| 5 | `test_enrichment_pivot_002_enrich_threatintel_pipe_stage_returns_malicious_for_scenario_iocs` | prism-spec-engine or prism-query integration | BC-2.19.001 postcondition | integration (demo server) | — |
| 6 | `test_enrichment_pivot_002_enrich_nvd_pipe_stage_returns_high_cvss_for_scenario_cves` | prism-spec-engine or prism-query integration | BC-2.19.001 postcondition | integration (demo server) | — |
| 7 | `test_enrichment_pivot_002_sec001_udf_name_rejects_sql_injection_chars` | prism-spec-engine | BC-2.19.001 precondition | unit | DRIFT-PIVOT-UDFNAME-VALIDATION-001 |
| 8 | `test_enrichment_pivot_002_sec001_udf_name_rejects_leading_digit` | prism-spec-engine | BC-2.19.001 precondition | unit | DRIFT-PIVOT-UDFNAME-VALIDATION-001 |
| 9 | `test_enrichment_pivot_002_sec001_udf_name_accepts_valid_identifiers` | prism-spec-engine | BC-2.19.001 precondition | unit | DRIFT-PIVOT-UDFNAME-VALIDATION-001 |
| 10 | `test_enrichment_pivot_002_sec002_plugin_infusion_source_config_not_pub` | prism-spec-engine | BC-2.19.001 invariant | unit/compile-fail | DRIFT-PIVOT-PLUGINCONFIG-PUB-FIELD-001 |
| 11 | `test_enrichment_pivot_002_sec003_sandbox_violation_url_not_in_warn_log` | prism-spec-engine | BC-2.19.001 invariant | unit | DRIFT-PIVOT-SANDBOXVIOLATION-URL-LOG-001 |
| 12 | `test_enrichment_pivot_002_sec001_wasm_enrich_wraps_spawn_blocking` | prism-spec-engine or prism-query | BC-2.19.001 postcondition | unit/integration | DRIFT-PIVOT-PLUGINID-INFUSIONID-001 SEC-001 |
| 13 | `test_enrichment_pivot_002_sec003_path_traversal_rejected_for_dotdot_plugin_ref` | prism-spec-engine | BC-2.19.001 precondition | unit | DRIFT-PIVOT-PLUGINPATH-TRAVERSAL-001 |
| 14 | `test_enrichment_pivot_002_sec003_path_within_plugin_dir_accepted` | prism-spec-engine | BC-2.19.001 precondition | unit | DRIFT-PIVOT-PLUGINPATH-TRAVERSAL-001 |
| 15 | `test_enrichment_pivot_002_sec002_load_all_error_does_not_leak_absolute_path` | prism-spec-engine | BC-2.19.001 invariant | unit | DRIFT-PIVOT-LOADALL-PATH-DISCLOSURE-001 |

Integration tests (tests 3-6) require demo server running with scenario.enabled = true.
Per SID-1, these tests are NOT `#[ignore]`'d unless blocking on a live external service.
An in-process demo server harness is sufficient and should NOT be `#[ignore]`'d.

Security gate tests 7-15 are unit tests. They MUST pass before any PIVOT-002 code merges
(they are Red Gate tests, not advisory). Tests 7-9 validate the new identifier-regex
validation in `InfusionLoader::parse`. Test 10 validates `pub(crate)` visibility. Test 11
uses `tracing_test` or equivalent span capture to assert URL redaction. Test 12 verifies
`spawn_blocking` wrapping. Tests 13-14 verify path canonicalization. Test 15 verifies
path stripping from MCP-surfaced errors.

---

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~4,000 |
| S-DEMO-ENRICHMENT-PIVOT-001 spec (context for plugin bridge) | ~1,500 |
| `prism-dtu-threatintel/src/types.rs` (response struct definitions) | ~800 |
| `prism-dtu-threatintel/src/routes/lookup.rs` (route shape) | ~600 |
| `prism-dtu-nvd/src/types.rs` (CveRecord struct + nested types) | ~1,000 |
| `prism-dtu-nvd/src/routes/cves.rs` (route shape) | ~600 |
| `threatintel.infusion.toml` (authored spec) | ~300 |
| `nvd.infusion.toml` (authored spec) | ~300 |
| `prism-threatintel-infusion/src/lib.rs` (WASM plugin) | ~1,500 |
| `prism-nvd-infusion/src/lib.rs` (WASM plugin) | ~1,200 |
| BC-2.19.001 (full) | ~1,500 |
| BC files (BC-2.06.020 for scenario IOC/CVE correlation context) | ~1,000 |
| Test files (15 stubs × ~50 lines each — 6 original + 9 security gate) | ~2,250 |
| BC files (BC-2.19.001, security drift item references) | ~1,500 |
| `plugin_bridge.rs` + `loader.rs` (security gate context reads) | ~1,200 |
| Tool outputs (nextest, clippy, demo server integration, tracing_test) | ~1,800 |
| **Total estimate** | **~20,350** |

At ~200k context window, this is ~10.2% — within the 20-30% ceiling.

---

## Tasks

**MANDATORY SECURITY GATES (D-1205) — must be addressed FIRST, before any TOML/plugin work**

These gates address security findings deferred from PIVOT-001 (3 LOW findings from
PIVOT-001 PR #189 adversary cascade + 2 latent findings that become live in PIVOT-002 +
1 SEC-001 sync-WASM MANDATORY gate). All 6 must have passing Red Gate tests before
PIVOT-002 code merges.

**Security Gate 1 — DRIFT-PIVOT-UDFNAME-VALIDATION-001 (SEC-001 CWE-20): UDF name identifier validation**

- [ ] Read `InfusionLoader::parse` in `crates/prism-spec-engine/src/infusion/loader.rs`
  to locate where `InfusionField { name, .. }` is constructed from TOML (currently line ~257-268)
- [ ] Add identifier regex validation on each `InfusionField.name` during parse:
  `^[a-zA-Z][a-zA-Z0-9_]*$` (must start with letter, followed by alphanumerics/underscore).
  Return `Err(InfusionError::InvalidFieldSpec { field: <name>, spec_path: <path>,
  message: "field name must match [a-zA-Z][a-zA-Z0-9_]* ..." })` on violation.
  Empty string also rejected.
- [ ] Write failing tests 7, 8, 9 FIRST (FAIL first — TDD Iron Law):
  `test_enrichment_pivot_002_sec001_udf_name_rejects_sql_injection_chars`
  `test_enrichment_pivot_002_sec001_udf_name_rejects_leading_digit`
  `test_enrichment_pivot_002_sec001_udf_name_accepts_valid_identifiers`
- [ ] Verify all three pass

**Security Gate 2 — DRIFT-PIVOT-PLUGINCONFIG-PUB-FIELD-001 (SEC-002 CWE-200): PluginInfusionSource.config encapsulation**

- [ ] In `crates/prism-spec-engine/src/infusion/plugin_bridge.rs`, change
  `pub config: Arc<PluginConfigMap>` to `pub(crate) config: Arc<PluginConfigMap>`
- [ ] Verify workspace builds: `just check-fast` — any external crate that accessed
  `.config` directly will produce E0616; fix callsites within `prism-spec-engine` to use
  the field (pub(crate) allows within-crate access)
- [ ] Write failing test 10 FIRST:
  `test_enrichment_pivot_002_sec002_plugin_infusion_source_config_not_pub`
  (unit test or compile-fail gate per ADR perimeter-violation pattern; document rationale
  in the test if using a unit test compensating control instead of compile-fail)
- [ ] Verify test 10 passes

**Security Gate 3 — DRIFT-PIVOT-SANDBOXVIOLATION-URL-LOG-001 (SEC-003 CWE-209): SandboxViolation URL redaction**

- [ ] In `plugin_bridge.rs` `map_plugin_error_to_infusion_error`, for the
  `PluginError::SandboxViolation { plugin_id, url }` arm: do NOT include `url` in the
  `InfusionError::MissingRequiredField { field }` message string. Include only
  `plugin_id` and a generic "sandbox policy violation" description.
- [ ] The `url` value MUST be emitted at DEBUG level only (not WARN) — add a
  `tracing::debug!(plugin_id = %pid, sandbox_url = %url, "sandbox violation URL (debug only)")`.
- [ ] Verify the `enrich_single` WARN log at line ~131-136 of plugin_bridge.rs does NOT
  contain the URL via the InfusionError Display (since the error no longer embeds it).
- [ ] Write failing test 11 FIRST:
  `test_enrichment_pivot_002_sec003_sandbox_violation_url_not_in_warn_log`
  (use `tracing_test` crate or a custom span recorder; assert formatted WARN output does
  not contain the URL string for a SandboxViolation error)
- [ ] Verify test 11 passes

**Security Gate 4 — DRIFT-PIVOT-PLUGINID-INFUSIONID-001 SEC-001 sync-WASM spawn_blocking gate (CWE-400)**

- [ ] Read `crates/prism-query/src/` for `InfusionAsyncUdf` or equivalent DataFusion UDF
  wrapper that calls `InfusionSource::enrich_single` from an async context
- [ ] Verify whether `spawn_blocking` wraps the synchronous WASM call. If YES: cite the
  location and close this gate with a code-pointer in the PR description.
  If NO: implement `tokio::task::spawn_blocking(|| runtime.enrich_single(...)).await`
  at the DataFusion UDF invoke_with_args boundary (preferred for minimum scope)
- [ ] Write failing test 12 FIRST:
  `test_enrichment_pivot_002_sec001_wasm_enrich_wraps_spawn_blocking`
  (if already implemented in PIVOT-001: write a test that confirms the async UDF call
  does NOT block the tokio runtime — a short timeout-based test is acceptable)
- [ ] Verify test 12 passes

**Security Gate 5 — DRIFT-PIVOT-PLUGINPATH-TRAVERSAL-001 (SEC-003 CWE-22): plugin_ref path canonicalization**

- [ ] Locate where `plugin_ref` from `InfusionSpec` is resolved to a filesystem path in
  the PIVOT-002 plugin loading code (likely in `InfusionLoader` or `InfusionRegistry::load_spec_with_runtime`)
- [ ] Before any `std::fs::read` or `File::open` on the `.prx` path:
  a) Resolve the path relative to the configured plugin directory
  b) Call `std::fs::canonicalize(resolved_path)` — this rejects `..` escapes by following
     symlinks and producing a real absolute path
  c) Assert `canonicalized_path.starts_with(&plugin_dir_canonical)` — if not, return
     `Err(InfusionError::InvalidFieldSpec { field: "plugin_ref", ... })` with a message
     like "plugin path escapes plugin directory" (do NOT include the attempted path in the
     error message surfaced to callers — see Security Gate 6)
- [ ] Write failing tests 13, 14 FIRST:
  `test_enrichment_pivot_002_sec003_path_traversal_rejected_for_dotdot_plugin_ref`
  `test_enrichment_pivot_002_sec003_path_within_plugin_dir_accepted`
- [ ] Verify both pass

**Security Gate 6 — DRIFT-PIVOT-LOADALL-PATH-DISCLOSURE-001 (SEC-002 CWE-209): path sanitization in MCP errors**

- [ ] Identify where `InfusionError` messages containing `spec_path` (absolute path) are
  converted into `PrismError` or MCP tool response strings. This is likely in the
  error chain from `InfusionLoader::load_all` → `InfusionError` → `SpecEngineError` →
  `PrismError` → MCP JSON response.
- [ ] At the point where `InfusionError` is serialized for MCP response (NOT internal tracing),
  strip the absolute path prefix from `spec_path`. Use `Path::file_name()` to extract
  just the filename, or `path.strip_prefix(config_dir)` to produce a relative path.
  Internal tracing (DEBUG/INFO for operator diagnostics) MAY retain the full path.
- [ ] Write failing test 15 FIRST:
  `test_enrichment_pivot_002_sec002_load_all_error_does_not_leak_absolute_path`
  (feed a deliberately bad TOML at an absolute path; capture the resulting error string
  that would surface in an MCP response; assert it does not contain the absolute prefix)
- [ ] Verify test 15 passes

---

**Pre-flight: SAP-2 DTU grounding verification (MANDATORY before writing any TOML)**

- [ ] Read `crates/prism-dtu-threatintel/src/types.rs` — confirm lookup response struct field names and types
- [ ] Read `crates/prism-dtu-threatintel/src/routes/lookup.rs` — confirm three separate routes:
  GET /v3/ip/:ip (line 162), GET /v3/domain/:domain (line 187), GET /v3/hash/:hash (line 214);
  confirm auth: ?key= param OR Authorization: Bearer <token>; response field `threat_sources`
  (JSON array) — NOT `threat_source` (string)
- [ ] Read `crates/prism-dtu-nvd/src/types.rs` — confirm CveRecord, CveMetrics, CvssMetricV31,
  CvssData structs and serde(rename_all=camelCase) annotations; wire names for JSON response:
  `vulnerabilities[].cve.id`, `metrics.cvssMetricV31[0].cvssData.baseScore/baseSeverity/vectorString`
- [ ] Read `crates/prism-dtu-nvd/src/routes/cves.rs` — confirm route: GET /rest/json/cves/2.0
  with ?cveId= query param (NOT /nvd/cves/{id}); auth: ?apiKey= query param
- [ ] For each TOML field, verify a corresponding field exists in the DTU response struct (SAP-2 check)

**Phase 1: threatintel.infusion.toml**

- [ ] Write failing test 1 (FAIL first): `test_enrichment_pivot_002_threatintel_toml_loads_and_registers_3_udfs`
- [ ] Author `{config_dir}/infusions/threatintel.infusion.toml` (location per loader.rs:45;
  match table layout from crates/prism-spec-engine/fixtures/threat_intel_plugin.infusion.toml
  and InfusionSpec deserialization at infusion/mod.rs:295-319):
  - `infusion_id = "threat_intel"`
  - `[source] type = "plugin"`, `plugin_ref = "threatintel-lookup.prx"`
  - `[source.credential] ref = "<keyring-reference-per-ADR-032>"` — auth via ?key= or Bearer token
    (NOT X-Admin-Token; that is admin surface only per lookup.rs lines 20-53)
  - `[[infusion.fields]] name = "threat_is_known_malicious" type = "Boolean"`
  - `[[infusion.fields]] name = "threat_score" type = "Integer"`
  - `[[infusion.fields]] name = "threat_sources" type = "Json"` — array per lookup.rs response
  - `[pipe_stage] adds_columns = ["threat_is_known_malicious", "threat_score", "threat_sources"]`
  - DTU clone routes: GET /v3/ip/:ip, GET /v3/domain/:domain, GET /v3/hash/:hash
    (NOT a unified endpoint); plugin dispatches on input IOC type to correct route
  - No production API URL in TOML (ADR-028)
- [ ] Verify test 1 passes

**Phase 2: nvd.infusion.toml**

- [ ] Write failing test 2 (FAIL first): `test_enrichment_pivot_002_nvd_toml_loads_and_registers_3_udfs`
- [ ] Author `{config_dir}/infusions/nvd.infusion.toml` (location per loader.rs:45;
  match table layout from InfusionSpec deserialization at infusion/mod.rs:295-319):
  - `infusion_id = "nvd"`
  - `[source] type = "plugin"`, `plugin_ref = "nvd-lookup.prx"`
  - `[source.credential] ref = "<keyring-reference-per-ADR-032>"` — auth via ?apiKey= query param
  - `[[infusion.fields]] name = "cvss_base_score" type = "Float"` — maps to cvssData.baseScore
  - `[[infusion.fields]] name = "cvss_severity" type = "String"` — maps to cvssData.baseSeverity
  - `[[infusion.fields]] name = "cvss_vector" type = "String"` — maps to cvssData.vectorString
  - `[pipe_stage] adds_columns = ["cvss_base_score", "cvss_severity", "cvss_vector"]`
  - DTU route: GET /rest/json/cves/2.0?cveId=<id> (NOT /nvd/cves/{id}; confirmed cves.rs)
  - ALSO declare in Armis sensor TOML: `device_cves_first` column (String) per Ruling 1b;
    generator projection task is in STORY-003 but TOML declaration is in this story
- [ ] Verify test 2 passes

**Phase 3: prism-threatintel-infusion WASM plugin**

- [ ] Create `crates/plugins/prism-threatintel-infusion/` crate (U18/Ruling 3) with
  standalone `Cargo.toml` (own `[workspace]` table, wasm32-wasip1 cdylib target; mirror
  crates/plugins/ocsf-complex-transforms/ structure; NO reqwest/tokio — HTTP via host WIT
  import host.http-request per prism-infusion-plugin.wit) and `src/lib.rs`
- [ ] Pre-flight: verify wit-bindgen 0.51 / wasm-tools 1.248.0 / wasmtime 44
  component-encoding alignment against wasmtime 44 changelog (U14/Ruling 4)
- [ ] Add exclusion to root `Cargo.toml` exclude list (NOT workspace members)
- [ ] Add Justfile recipe per build-plugin-crowdstrike-oauth2 template:
  `cargo build --target wasm32-wasip1 --release → wasm-tools component new --adapt
  wasi_snapshot_preview1.wasm → .prx`
- [ ] Write failing test 3 (FAIL first) in integration test
- [ ] Implement `src/lib.rs`: receive IOC value via WIT ABI; dispatch on IOC type to
  correct DTU route via host.http-request WIT import —
  IP: GET /v3/ip/:ip, domain: GET /v3/domain/:domain, hash: GET /v3/hash/:hash;
  auth: ?key= query param OR Authorization: Bearer <token>;
  deserialize response into `{ threat_is_known_malicious: bool, threat_score: i64, threat_sources: Vec<String> }`;
  return as plugin ABI output per WIT interface (AD-019/CAP-032)
- [ ] Verify test 3 passes (with demo server running)

**Phase 4: prism-nvd-infusion WASM plugin**

- [ ] Create `crates/plugins/prism-nvd-infusion/` crate (U18/Ruling 3) with standalone
  `Cargo.toml` (own `[workspace]` table, wasm32-wasip1 cdylib; NO reqwest/tokio) and `src/lib.rs`
- [ ] Add exclusion to root `Cargo.toml` exclude list; add Justfile recipe (same pattern as ThreatIntel)
- [ ] Write failing test 4 (FAIL first) in integration test
- [ ] Implement `src/lib.rs`: receive CVE ID via WIT ABI; call DTU route
  `GET /rest/json/cves/2.0?cveId=<cve_id>` via host.http-request WIT import;
  auth: ?apiKey= query param;
  parse response envelope: `vulnerabilities[0].cve.metrics.cvssMetricV31[0].cvssData.baseScore` (f64),
  `baseSeverity` (String), `vectorString` (String) (all camelCase wire names per serde rename_all);
  return as plugin ABI output
- [ ] Verify test 4 passes (with demo server running)

**Phase 5: pipe stage integration tests**

- [ ] Write failing tests 5, 6 (FAIL first): enrich pipe stage integration tests
- [ ] Verify both tests pass with demo server at stage >= 3

**Phase 6: Final gates**

- [ ] SAP-2 post-implementation sweep: re-read DTU types.rs and routes — confirm each TOML
  column has a matching DTU struct field; document parity table in PR description
- [ ] SAP-1 probe: `rg 'event_type\s*=' crates/ --type rust` — any new emissions need catalog rows
- [ ] Run `just check` — all Red Gate tests pass; zero clippy warnings; fmt clean
- [ ] Verify perimeter: new plugin crates must NOT import `prism-spec-engine`, `prism-sensors`,
  or `prism-query` (plugin binary ABI is a separate compilation unit)

---

## Previous Story Intelligence

**S-DEMO-ENRICHMENT-PIVOT-001 (direct predecessor):**
- `InfusionLoader::parse` for `source.type = "plugin"` is operational
- `plugin_bridge::enrich_via_plugin` is wired (or has annotated todo! for S-1.15)
- DataFusion UDF registration is wired in prism-query
- `is_api_backed()` is implemented for plugin-type UDFs

**S-DEMO-DTU-LIVE-SCENARIO-001-B (substrate):**
- `ThreatIntelClone::new_with_scenario(entities)` pre-populates `fixture_registry` with
  scenario IOCs as `FixtureKey::Malicious` at construction time
- `NvdClone::new_with_scenario(entities)` pre-populates `cve_registry` with scenario CVEs
  at `cvss_base_score = 8.1`, `cvss_severity = "HIGH"`
- At stage >= 3 (Exfil), `ioc_hashes`, `ioc_ips`, `ioc_domains` are visible in StageMask

**From PLUGIN-MIGRATION-001-D/E lessons:**
- SAP-2: adversary reads DTU types.rs and routes/ — do not rely on story description alone
- SID-1: integration tests driving in-process demo server are NOT `#[ignore]`'d

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| TOML infusion specs MUST be grounded against DTU clone route surfaces — NOT assumed production API URLs | ADR-028/ADR-031 + WO-D1109 §Story 2 DTU grounding requirement | SAP-2 adversary probe (read DTU types.rs + routes/ before review) |
| Every TOML `[[infusion.fields]]` column MUST have a matching field in the corresponding DTU route response struct | SAP-2 (CLAUDE.md §SAP-2) | Adversary: read prism-dtu-threatintel/src/types.rs + routes/ and prism-dtu-nvd/src/types.rs + routes/ |
| Plugin crates (`prism-threatintel-infusion`, `prism-nvd-infusion`) MUST NOT depend on `prism-spec-engine`, `prism-sensors`, or `prism-query` | INV-PERIMETER-COMPLIANCE-001 (BC-2.06.020) + ADR-036 v2.2 §2.5 | Build check; compile-fail gate |
| Credential references in TOML specs MUST use reference-based model (no inline credential values) | AD-017 + ADR-032 | Adversary AD-017 probe |
| All `event_type =` tracing emissions require BC-2.16.002 catalog rows | SAP-1 / CLAUDE.md §SAP-1 | Adversary SAP-1 probe |
| HOST HTTP client: `.timeout(Duration::from_secs(30))` mandatory | CLAUDE.md §Conventions (TD-S-PLUGIN-PREREQ-B-005 precedent) — applies to HOST reqwest client in host_functions.rs, NOT to WASM guest crates |  Adversary |
| WASM guest crates MUST NOT use reqwest or tokio (no sockets in sandbox) | U9 research-confirmed 2026-06-12; HTTP via host WIT import host.http-request (host_functions.rs:59) | Adversary: grep for reqwest/tokio in crates/plugins/prism-*-infusion/Cargo.toml |

| `InfusionField.name` MUST match `^[a-zA-Z][a-zA-Z0-9_]*$` — validated at parse time before UDF registration | DRIFT-PIVOT-UDFNAME-VALIDATION-001 (D-1190 SEC-001 CWE-20) | AC-007; test_enrichment_pivot_002_sec001_* |
| `PluginInfusionSource.config` field MUST be `pub(crate)` (not `pub`) before PIVOT-002 populates credentials | DRIFT-PIVOT-PLUGINCONFIG-PUB-FIELD-001 (D-1190 SEC-002 CWE-200) | AC-008; test_enrichment_pivot_002_sec002_plugin_infusion_source_config_not_pub |
| `SandboxViolation.url` MUST NOT appear in WARN-level tracing or MCP error strings | DRIFT-PIVOT-SANDBOXVIOLATION-URL-LOG-001 (D-1190 SEC-003 CWE-209) | AC-009; test_enrichment_pivot_002_sec003_sandbox_violation_url_not_in_warn_log |
| Sync WASM call in async UDF MUST be wrapped in `spawn_blocking` | DRIFT-PIVOT-PLUGINID-INFUSIONID-001 SEC-001 MANDATORY gate (D-1179) | AC-010; test_enrichment_pivot_002_sec001_wasm_enrich_wraps_spawn_blocking |
| `plugin_ref` path MUST be canonicalized and verified against plugin_dir before file I/O | DRIFT-PIVOT-PLUGINPATH-TRAVERSAL-001 (D-1179 SEC-003 CWE-22) | AC-011; test_enrichment_pivot_002_sec003_path_traversal_rejected_* |
| `InfusionLoader::load_all` errors MUST NOT surface absolute filesystem paths in MCP responses | DRIFT-PIVOT-LOADALL-PATH-DISCLOSURE-001 (D-1179 SEC-002 CWE-209) | AC-012; test_enrichment_pivot_002_sec002_load_all_error_does_not_leak_absolute_path |

**Forbidden Dependencies:**
- `crates/plugins/prism-threatintel-infusion` and `crates/plugins/prism-nvd-infusion` MUST NOT
  depend on `prism-spec-engine`, `prism-sensors`, or `prism-query` (plugin binary ABI is a
  separate compilation unit; out-of-workspace crates have no workspace dependency access)
- WASM guest crates MUST NOT depend on `reqwest` or `tokio` (no sockets in WASM sandbox — U9);
  HTTP goes through host WIT import `host.http-request`

---

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| `serde` / `serde_json` | 1.x (workspace — host side); guest: check ocsf-complex-transforms deps | DTU response deserialization |
| `wit-bindgen` | 0.51 (confirmed 2026-06-12) | WIT guest bindings for host.http-request WIT import in plugin crates |
| `wasm-tools` | 1.248.0 (confirmed 2026-06-12) | Justfile build pipeline: `wasm-tools component new --adapt wasi_snapshot_preview1.wasm` |
| `wasmtime` | 44.x (host, already in prism-spec-engine) | Plugin loading and execution (host side) |
| `toml` | 0.8.x (workspace) | Infusion spec TOML parsing (in prism-spec-engine, already present) |
| `reqwest` | HOST ONLY (project-pinned; `.timeout(30s)` mandatory) | HTTP client in HOST functions (host_functions.rs) — NOT in WASM guest crates (U9: no sockets in WASM sandbox) |
| `tokio` | HOST ONLY (1.x workspace) | Async runtime for HOST reqwest client — NOT in WASM guest crates |
| `tracing-test` | 0.2.6 (already in workspace Cargo.lock; used in prism-query) | Dev-dep for AC-009 WARN log assertion test. Add `tracing-test = "0.2"` to `[dev-dependencies]` in `prism-spec-engine/Cargo.toml` if not already present. |
| `regex` or `once_cell` + `Lazy<Regex>` | workspace | Identifier validation regex for AC-007. Use `once_cell::sync::Lazy` (already workspace dep) or inline `Regex::new(r"^[a-zA-Z][a-zA-Z0-9_]*$")`. Do NOT add regex as a new production dep if inline validation with `str::chars()` suffices (char-by-char is simpler and zero-dep). |

**MSRV:** Rust stable per `rust-toolchain.toml` (host crates); wasm32-wasip1 target for plugin guest crates (out-of-workspace — standalone [workspace] per Ruling 3).

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `{config_dir}/infusions/threatintel.infusion.toml` | CREATE | ThreatIntel plugin-type infusion spec grounded against prism-dtu-threatintel routes (U16: location per InfusionLoader loader.rs:45) |
| `{config_dir}/infusions/nvd.infusion.toml` | CREATE | NVD plugin-type infusion spec grounded against prism-dtu-nvd routes; includes device_cves_first column declaration in Armis TOML (U17/Ruling 1b) |
| `crates/plugins/prism-threatintel-infusion/Cargo.toml` | CREATE | Out-of-workspace standalone crate (U18/Ruling 3); own [workspace]; excluded from root Cargo.toml; wasm32-wasip1 cdylib; NO reqwest/tokio |
| `crates/plugins/prism-threatintel-infusion/src/lib.rs` | CREATE | WASM guest plugin: dispatches IOC type → correct DTU route via host.http-request WIT; parses threat_sources (Json array) |
| `crates/plugins/prism-nvd-infusion/Cargo.toml` | CREATE | Out-of-workspace standalone crate (U18/Ruling 3); same pattern |
| `crates/plugins/prism-nvd-infusion/src/lib.rs` | CREATE | WASM guest plugin: calls GET /rest/json/cves/2.0?cveId=; parses camelCase CVSS fields |
| `Cargo.toml` (workspace root) | MODIFY | Add `crates/plugins/prism-threatintel-infusion` and `crates/plugins/prism-nvd-infusion` to `exclude` list (NOT members) |
| `Justfile` | MODIFY | Add build-plugin-threatintel-infusion + build-plugin-nvd-infusion recipes per build-plugin-crowdstrike-oauth2 pattern |
| Armis sensor TOML spec | MODIFY | Add device_cves_first column declaration (String; generator projection in STORY-003) |
| Integration test file (TBD location) | CREATE | Tests 3-6: pipe stage integration against demo server |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Lookup for an IOC value NOT in `fixture_registry` (non-scenario IOC) | DTU returns non-malicious result; `threat_is_known_malicious = false`; no error |
| EC-002 | Lookup for a CVE ID NOT in `cve_registry` (non-scenario CVE) | DTU returns 404 or empty; plugin returns `None` or zero-scored result; no panic |
| EC-003 | Demo server NOT running during integration test | HTTP error; plugin returns `Err(...)` propagated to pipe stage; test marked for demo server prerequisite |
| EC-004 | Stage < 3 (IOCs not yet visible) | DTU alerts route withholds IOC-bearing records; enrich pipe stage has no matching records to enrich (empty result, not error) |
| EC-005 | TOML spec references a `plugin_ref` file that does not exist on disk | `InfusionLoader::load_all` returns `SpecEngineError::PluginNotFound` with the plugin_ref name |
| EC-006 | Plugin HTTP call times out (DTU slow or not running) | Host WIT import host.http-request applies 30s timeout on HOST side (reqwest client in host_functions.rs, not in WASM guest); host returns error to guest; plugin returns `Err(...)`; no infinite hang (U9: WASM guests have no socket access — timeout is host-side only) |
| EC-007 | `[[infusion.fields]]` entry with `name = "threat; DROP TABLE"` (SQL injection attempt in field name) | `InfusionLoader::parse` returns `InfusionError::InvalidFieldSpec`; no UDF registered; no DataFusion crash (DRIFT-PIVOT-UDFNAME-VALIDATION-001; AC-007) |
| EC-008 | `plugin_ref = "../../etc/passwd.prx"` (path traversal in TOML spec) | Canonicalization gate rejects the path; returns `InfusionError::InvalidFieldSpec`; no file I/O on the traversal target (DRIFT-PIVOT-PLUGINPATH-TRAVERSAL-001; AC-011) |
| EC-009 | `PluginError::SandboxViolation { url: "http://dtu-host:8080/v3/ip/1.2.3.4" }` surfaces from plugin call | WARN log does not contain the URL string; DEBUG log may contain it; `InfusionError` message does not embed the URL (DRIFT-PIVOT-SANDBOXVIOLATION-URL-LOG-001; AC-009) |
| EC-010 | `InfusionLoader::load_all` fails on `/home/analyst/.prism/infusions/bad.toml` | MCP-surfaced error message contains only `bad.toml` (or `infusions/bad.toml`), NOT the absolute path `/home/analyst/.prism/infusions/bad.toml` (DRIFT-PIVOT-LOADALL-PATH-DISCLOSURE-001; AC-012) |

---

## Architecture Mapping

| Component | Module | Pure/Effectful | Anchor |
|-----------|--------|---------------|--------|
| `specs/infusions/threatintel.infusion.toml` | `specs/infusions/` (config artifact) | Pure (declarative config) | WO-D1109 §Story 2 |
| `specs/infusions/nvd.infusion.toml` | `specs/infusions/` (config artifact) | Pure (declarative config) | WO-D1109 §Story 2 |
| `prism-threatintel-infusion` plugin | New crate (SS-17) | Effectful (HTTP call to DTU) | AD-019 / CAP-032 |
| `prism-nvd-infusion` plugin | New crate (SS-17) | Effectful (HTTP call to DTU) | AD-019 / CAP-032 |
| `InfusionLoader` (loading these specs) | `prism-spec-engine` (SS-19) | Mixed (provided by 001) | BC-2.19.001 |

---

## SAP-2 Compliance Note

Per CLAUDE.md §SAP-2, the adversary for this story MUST:
1. Read `crates/prism-dtu-threatintel/src/routes/lookup.rs` — confirm three routes: GET /v3/ip/:ip,
   GET /v3/domain/:domain, GET /v3/hash/:hash (NOT a unified /threatintel/lookup/{v} endpoint);
   confirm auth: ?key= or Authorization: Bearer; confirm `threat_sources` is JSON array (NOT `threat_source` string)
2. Read `crates/prism-dtu-nvd/src/types.rs` and `routes/cves.rs` — confirm route: GET /rest/json/cves/2.0?cveId=;
   confirm wire field names are camelCase (baseScore/baseSeverity/vectorString — NOT snake_case in JSON);
   NVD auth: ?apiKey= param
3. For every column in `threatintel.infusion.toml` and `nvd.infusion.toml`:
   - Verify the column name matches a field in the DTU response struct (adversary reads actual source)
   - Verify the column type matches the DTU Rust type (threat_sources: Json array; cvss_base_score: Float)
4. Verify WASM guest Cargo.toml files do NOT list reqwest or tokio as dependencies (U9 gate)
5. Verify plugin crates are in root Cargo.toml `exclude` list, NOT `members` (U18/Ruling 3 gate)

Column in TOML with no DTU equivalent = **P1 CRITICAL** finding. `threat_source` (singular) as a declared column = **P1 CRITICAL** (wrong; JSON response has array field `threat_sources`). reqwest/tokio in WASM guest = **P1 CRITICAL** (U9).

---

## Story Changelog

| Version | Date | Change |
|---------|------|--------|
| v1.2 | 2026-06-17 | D-1205 MANDATORY security gate fold-in (story-writer pre-TDD pass). Added AC-007 through AC-012 covering all 6 DRIFT items: DRIFT-PIVOT-UDFNAME-VALIDATION-001 (UDF name identifier validation, AC-007), DRIFT-PIVOT-PLUGINCONFIG-PUB-FIELD-001 (PluginInfusionSource.config encapsulation, AC-008), DRIFT-PIVOT-SANDBOXVIOLATION-URL-LOG-001 (SandboxViolation URL redaction, AC-009), DRIFT-PIVOT-PLUGINID-INFUSIONID-001 SEC-001 sync-WASM spawn_blocking gate (AC-010), DRIFT-PIVOT-PLUGINPATH-TRAVERSAL-001 (path traversal rejection, AC-011), DRIFT-PIVOT-LOADALL-PATH-DISCLOSURE-001 (path stripping in MCP errors, AC-012). Red Gate tests expanded from 6 to 15 (9 new security gate tests added). Tasks section expanded with MANDATORY SECURITY GATES section (6 security gate tasks, each with FAIL-first Red Gate test discipline). Architecture Compliance Rules updated with 6 security gate enforcement rows. Edge Cases EC-007 through EC-010 added. Token budget updated (~20,350 tokens, ~10.2% of 200k). Points remain 8 (security gates are implementation of existing code, not new functional scope). remove-uncertainty validate: wasmtime=44 (confirmed in prism-spec-engine/Cargo.toml), wasm-tools=1.248.0, wit-bindgen=0.51 — all confirmed correct in workspace. No new technology uncertainties found. |
| v1.1 | 2026-06-12 | D-1109 remove-uncertainty closure: U1/U9/U10/U11/U12/U13/U14/U15/U16/U17/U18 applied (scanner + research-agent + architect rulings 1-4, WO-D1109 v1.1). enrich syntax → function-call form. reqwest/tokio removed from WASM guest crates (host WIT import host.http-request). ThreatIntel endpoints corrected: three separate routes GET /v3/{ip,domain,hash}/:value (NOT unified). threat_sources declared as Json array (NOT string). NVD route corrected: GET /rest/json/cves/2.0?cveId= (confirmed cves.rs). NVD wire names corrected to camelCase. cargo-component removed; Justfile wasm-tools pipeline (Ruling 4). ThreatIntel auth corrected to ?key=/Bearer (NOT X-Admin-Token). NVD auth: ?apiKey=. Spec location: config_dir/infusions/ (loader.rs:45). Plugin crates relocated to crates/plugins/ out-of-workspace (Ruling 3). Ruling 1b: NVD enrich field device_cves_first; Armis TOML column scope in this story, generator projection in STORY-003. |
| v1.0 | 2026-06-12 | Initial draft per WO-D1109 §Story 2. Grounded against DTU clone route surfaces. Depends on 001; blocks 003. Two new plugin crates. SAP-2 compliance note included. BC-2.19.001 as primary anchor; PO to confirm at materialization. |
