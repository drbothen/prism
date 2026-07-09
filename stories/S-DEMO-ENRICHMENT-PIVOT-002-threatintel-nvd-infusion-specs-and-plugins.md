---
document_type: story
story_id: S-DEMO-ENRICHMENT-PIVOT-002
title: "ThreatIntel/NVD Infusion — Dual-Path: ThreatIntel WASM Plugin + NVD HttpLookup Built-in"
wave: 5
epic_id: E-DEMO
priority: P2
status: merged
version: "1.6"
level: "L4"
producer: story-writer
timestamp: "2026-06-12T00:00:00Z"
created: "2026-06-12"
modified: "2026-06-17T00:00:00Z"
tdd_mode: strict
subsystems: [SS-19, SS-17, SS-01]
# Subsystem anchor justifications:
#   SS-19 (Infusion Enrichment Framework) owns .infusion.toml specs, InfusionLoader,
#   InfusionRegistry, and the new HttpLookupSource built-in per ARCH-INDEX Subsystem Registry.
#   SS-17 (Plugin Runtime / WASM) owns the ThreatIntel .prx WASM plugin crate, the WIT
#   guest bindings (wit_bindgen::generate!), and the Val-lift fix in PluginRuntime::enrich_single.
#   SS-01 (Sensor Adapters) is included because this story grounds the infusion specs
#   against prism-dtu-threatintel and prism-dtu-nvd route surfaces (DTU-backed enrichment
#   per ADR-028/ADR-031 DTU grounding requirement, WO-D1109 §Q1).
target_module: prism-spec-engine
crates_touched: [prism-spec-engine, prism-core, prism-dtu-threatintel, prism-dtu-nvd]
# new crate (out-of-workspace, standalone [workspace] in Cargo.toml, excluded from root
#   Cargo.toml via exclude = [...] — same pattern as crates/plugins/ocsf-complex-transforms):
#   crates/plugins/prism-threatintel-infusion — WASM guest cdylib (wasm32-wasip1) calling
#     prism-dtu-threatintel HTTP API via host WIT import host.http-request
#   NOTE (ADR-040 v2.0 D9): crates/plugins/prism-nvd-infusion is NOT built and NOT needed.
#   NVD uses the new InfusionType::HttpLookup permanent built-in (no WASM toolchain).
#   prism-core is now in crates_touched because new InfusionError + PluginError variants
#   are added in this story (ADR-040 D5 + D8.5).
# BC status: BC-2.19.001 v1.9 is the current authoritative version (PO amendment complete).
# BC-2.19.001 governs infusion spec loading and UDF registration (nearest anchor).
# PO amendment complete (v1.8, 2026-06-17): BC-2.19.001 E-INFUSE-004 valid-types list
#   was amended to add "http_lookup" — the error message now reads:
#   "Unknown source type '...'. Valid types: maxmind_mmdb, csv, json_lookup, plugin, http_lookup."
#   No further PO routing required for BC-2.19.001 http_lookup scope.
# E-INFUSE-004 error.rs sync obligation (architect ruling S-1.14-REDO Q3, 2026-06-18):
#   When PIVOT-002 adds InfusionType::HttpLookup to infusion/mod.rs, the implementer MUST
#   also update InfusionError::UnknownSourceType Display in error.rs to include http_lookup
#   in the valid-types string in the SAME COMMIT. Until PIVOT-002 lands, error.rs
#   intentionally omits http_lookup. See body Behavioral Contracts section + Phase 1 tasks.
#   BC-2.06.020 scope: not anchored here until PO confirms overlap.
behavioral_contracts: [BC-2.19.001]
# BC array propagation note: BC-2.19.001 is cited by AC-001, AC-002, AC-003, AC-004, AC-013
#   through AC-018 (bidirectional trace satisfied for all ACs in this story).
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
points: 13
# Points justification (ADR-040 v2.0 dual-path pivot):
#   1. InfusionType::HttpLookup variant + HttpLookupConfig/HttpLookupAuthType/
#      HttpLookupCredentialConfig types (new public non-exhaustive types, D8.2): 2 pts
#   2. HttpLookupSource implementation (~150-200 lines, D8.4): reuses Interpolator +
#      extract_at_path + build_http_client_with_timeout from pipeline.rs; SSRF validation;
#      credential AD-017 handling; error codes E-INFUSE-009/010/011: 3 pts
#   3. InfusionLoader::parse "http_lookup" arm + registry wiring + is_api_backed (D8.3/D8.6): 1 pt
#   4. nvd.infusion.toml → type="http_lookup" schema per D8.1: 0.5 pt
#   5. threatintel.infusion.toml grounded against prism-dtu-threatintel route surface: 0.5 pt
#   6. prism-threatintel-infusion .prx plugin (WASM + wit_bindgen::generate! + Val-lift fix D2/D3):
#      IOC dispatch + host WIT HTTP + WIT file copy: 3 pts
#   7. PluginRuntime::enrich_single Val-lift fix + enrich_batch fix (D2) + error variants
#      E-PLUGIN-023 / E-INFUSE-008 (D5): 2 pts
#   8. prism-nvd-infusion crate removal (D9): 0.5 pt
#   9. BC-2.16.002 catalog rows for 4 new event_types (SAP-1 obligation): 0.5 pt
#   Total: 13 pts (scope expanded significantly vs v1.2 8 pts due to HttpLookup built-in
#   and Val-lift CRIT fix being in-scope; NVD WASM plugin removal saves ~2 pts but
#   HttpLookup infrastructure + CRIT fix adds ~7 pts net)
estimated_days: 3
risk: HIGH
# Risk justification (ADR-040 v2.0 dual-path pivot):
#   ThreatIntel WASM path: val-lift CRIT fix (D2) requires wasmtime Component Model
#   internals understanding; wit_bindgen::generate! in out-of-workspace crate requires
#   WIT file copy + path discipline (D3); wasm32-wasip1 cross-compile complexity
#   (wasm-tools 1.248.0, wit-bindgen 0.51, wasmtime 44 alignment).
#   HttpLookup path: new InfusionType built-in requires SSRF validation at construction
#   time (DNS lookup risk if DNS unavailable — must fail closed); Interpolator/extract_at_path
#   reuse requires reading pipeline.rs implementation before integration; four new public
#   non-exhaustive types (EXPECTED count update in ci.yml).
#   Both paths: SAP-2 DTU grounding before any TOML; SAP-1 BC-2.16.002 catalog rows for
#   4 new event_types; prism-core modifications (new error variants in two enums).
red_gate_tests: 32
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

# S-DEMO-ENRICHMENT-PIVOT-002: ThreatIntel/NVD Infusion — Dual-Path Architecture

Author the `threatintel.infusion.toml` and `nvd.infusion.toml` infusion specs — grounded
against the actual DTU clone route surfaces per ADR-028/ADR-031 (DTU=True-DTU fidelity
principle) — and implement the dual-path enrichment architecture ratified in ADR-040 v2.0:

- **ThreatIntel → WASM Plugin (`type = "plugin"`):** IOC classification (ip/domain/hash)
  requires code; stays on the Plugin path. Includes the F-001 CRIT Val-lift fix in
  `PluginRuntime::enrich_single` (D2) and real `wit_bindgen::generate!` guest bindings (D3).
- **NVD → HttpLookup (`type = "http_lookup"`):** Single stateless GET → JSONPath → fields;
  zero custom logic; no WASM toolchain. Uses the new `InfusionType::HttpLookup` permanent
  built-in implemented in this story per ADR-040 D7–D8.
- **`prism-nvd-infusion` crate REMOVED:** NVD moves off WASM entirely; crate is dead code
  per ADR-040 D9.

**Sequencing context (D-1109, WO-D1109):** Slots AFTER S-DEMO-ENRICHMENT-PIVOT-001
(plugin bridge operational) and BEFORE S-DEMO-ENRICHMENT-PIVOT-003 (IOC stamping + pivot query).

**DTU grounding requirement (WO-D1109 §Story 2):** The infusion TOML specs MUST be
grounded against the actual DTU clone route surface, not assumed production API URLs.
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
| BC-2.19.001 v1.9 | Infusion Spec Loading — Each Field Registers Exactly One DataFusion Scalar UDF | Postcondition: each field in `[[infusion.fields]]` produces exactly one `InfusionUdfDescriptor` registered in `SessionContext`. Precondition: spec is structurally valid (`InfusionLoader::parse` validates field names and source type). Invariant: credential data does not leak through public API surface. Error cases: E-INFUSE-004 valid types: `maxmind_mmdb, csv, json_lookup, plugin, http_lookup` (v1.9 — `http_lookup` added at v1.8). |

**BC-2.19.001 v1.9 (current) — E-INFUSE-004 valid-types list status:**
BC-2.19.001 was amended at v1.8 (burst PIVOT-002-bc-amendment-http-lookup, 2026-06-17) to add
`http_lookup` to the E-INFUSE-004 valid-types list. The current v1.9 BC body already reads:
`E-INFUSE-004: "Unknown source type 'unknown'. Valid types: maxmind_mmdb, csv, json_lookup, plugin, http_lookup."`
The PO amendment is COMPLETE — no further PO routing is required for E-INFUSE-004.

**E-INFUSE-004 message sync obligation (architect ruling S-1.14-REDO Q3, 2026-06-18):**
When PIVOT-002 adds `InfusionType::HttpLookup` and the `"http_lookup"` arm in
`InfusionLoader::parse`, the `prism_core::error::InfusionError::UnknownSourceType` (or equivalent)
Display implementation in `error.rs` MUST also update its valid-types string to include
`, http_lookup` — matching the BC-2.19.001 v1.9 E-INFUSE-004 error message.
Until PIVOT-002 lands, `error.rs` intentionally omits `http_lookup` from the valid-types list
(the type does not yet exist). The implementer MUST update the error message in the same commit
that adds `InfusionType::HttpLookup` to `infusion/mod.rs`.
If editing the frontmatter `behavioral_contracts:` array risks merge conflicts with concurrent
BC amendments, this obligation is recorded here in the body only — the frontmatter array
update may be deferred to state-manager's post-merge burst.

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

### AC-002 — nvd.infusion.toml parses and loads as http_lookup-type infusion spec
(traces to BC-2.19.001 postcondition — each field registers exactly one UDF descriptor;
BC-2.19.001 precondition — spec is structurally valid; ADR-040 v2.0 D8.1/D8.3)

Given `{config_dir}/infusions/nvd.infusion.toml` (U16: canonical location per
InfusionLoader loader.rs:45 — NOT repo-root `specs/infusions/`) with:
- `[infusion] type = "http_lookup"` (NOT `"plugin"` — ADR-040 v2.0 D8.1)
- `[source.http]` block: `base_url = "https://services.nvd.nist.gov"`,
  `url_template = "/rest/json/cves/2.0?cveId=${input}"`, `method = "GET"`,
  `response_path = "$.vulnerabilities[0].cve.metrics.cvssMetricV31[0].cvssData"`
- `[source.credential]` block: `ref = "nvd.api_key"`, `env_var = "PRISM_NVD_API_KEY"`,
  `auth = "query_param"`, `param_name = "apiKey"` (AD-017 — no inline value)
- `[[infusion.fields]]` declaring:
  - `name = "cvss_base_score"`, `output_type = "float"`, `source_column = "baseScore"`
  - `name = "cvss_severity"`, `output_type = "string"`, `source_column = "baseSeverity"`
  - `name = "cvss_vector"`, `output_type = "string"`, `source_column = "vectorString"`

Grounded against `prism-dtu-nvd` route `GET /rest/json/cves/2.0?cveId=<id>` (SAP-2):
envelope `vulnerabilities[0].cve.metrics.cvssMetricV31[0].cvssData` (the JSONPath
in `response_path`); camelCase wire fields `baseScore` (f64), `baseSeverity` (String),
`vectorString` (String) per serde(rename_all=camelCase) confirmed 2026-06-12 types.rs.
NOTE (U17/Ruling 1b): the enrich input field is `device_cves_first` (scalar projected by
STORY-003 Armis generator), NOT `device_cves[0]`.

when `InfusionLoader::load_all` runs (with `PRISM_DTU_MODE=true` to bypass SSRF guard
since DTU base_url is loopback/private — AC-017 covers SSRF gate separately),
then the returned `InfusionRegistry` contains 3 `InfusionUdfDescriptor` entries for NVD fields,
`registry.is_api_backed("cvss_base_score")` returns `true` (D8.6 `HttpLookup` is API-backed),
and the source on each descriptor is an `HttpLookupSource` instance (not `NullSource` or
`PluginInfusionSource`).

Red Gate: `test_enrichment_pivot_002_nvd_toml_loads_as_http_lookup_and_registers_3_udfs`

### AC-003 — ThreatIntel WASM plugin: Val-lift fix + wit_bindgen bindings + DTU dispatch
(traces to BC-2.19.001 postcondition — plugin-type source executes via plugin bridge;
ADR-040 v2.0 D2/D3 — Val::String params + Val::Option result lift; F-001 CRIT closure)

Given:
1. `PluginRuntime::enrich_single` (in `crates/prism-spec-engine/src/plugin/mod.rs`)
   is fixed per ADR-040 D2:
   - Params passed as `[Val::String(input_value), Val::String(input_type)]` (NOT Val::S32)
   - Results buffer pre-populated as `vec![Val::Option(None)]`
   - After `func.call Ok(_)`, `results[0]` is lifted:
     `Val::Option(Some(Box<Val::String(json_str)>))` → `serde_json::from_str(json_str)` → `Ok(Some(v))`
     `Val::Option(None)` → `Ok(None)`
     unexpected Val variant → `Err(PluginError::EnrichCallFailed { .. })` (E-PLUGIN-023) +
     `event_type = "plugin_enrich_unexpected_val"` tracing
     JSON parse error → `Err(PluginError::EnrichCallFailed { .. })` (E-PLUGIN-023) +
     `event_type = "plugin_enrich_json_parse_error"` tracing
   - Two-phase function lookup: bare name ("enrich-single") first, then interface-scoped
     ("prism:infusion-plugin/infusion-plugin@0.1.0") per D2 function-resolution pattern
2. `prism-threatintel-infusion/src/lib.rs` implements `wit_bindgen::generate!` with
   `path = "wit"` (D3: WIT file copied into `prism-threatintel-infusion/wit/`);
   `struct Plugin` implements the generated `Guest` trait; `export!(Plugin)` at bottom

when `PluginInfusionSource::enrich_single` is called with an IOC value from
`ScenarioEntityCatalog.ioc_ips[0]` (a known-Malicious scenario IOC pre-populated in the
DTU `fixture_registry`),
then the plugin dispatches on input-type to the correct DTU route via host WIT `http_request`:
  - IP: `GET /v3/ip/:ip` (confirmed 2026-06-12 lookup.rs:162)
  - domain: `GET /v3/domain/:domain` (lookup.rs:187)
  - hash: `GET /v3/hash/:hash` (lookup.rs:214)
Auth: `?key=<api_key>` query param OR `Authorization: Bearer <token>` header (NOT X-Admin-Token).
Plugin returns JSON: `{"threat_score": N, "threat_is_known_malicious": true, "threat_sources": [...]}`
host lifts `Val::Option(Some(Box<Val::String(json_str))))` → `Ok(Some(value))`.
`threat_sources` is a JSON ARRAY (NOT `threat_source` singular string — confirmed lookup.rs).
Result: `threat_is_known_malicious = true` and `threat_score >= 75`.

Integration test requires the demo server running with `scenario.enabled = true`.

Red Gate: `test_enrichment_pivot_002_threatintel_plugin_resolves_scenario_ioc_as_malicious`

### AC-004 — NVD HttpLookupSource calls DTU CVE endpoint and returns CVSS fields
(traces to BC-2.19.001 postcondition — HttpLookup-type source executes via HttpLookupSource;
ADR-040 v2.0 D8.4 — HttpLookupSource.enrich_single; NOT via prism-nvd-infusion WASM plugin)

Given `HttpLookupSource` constructed from `nvd.infusion.toml` (D8.1: `type = "http_lookup"`,
`base_url = "https://services.nvd.nist.gov"`, `url_template = "/rest/json/cves/2.0?cveId=${input}"`,
`response_path = "$.vulnerabilities[0].cve.metrics.cvssMetricV31[0].cvssData"`),
and the NVD DTU clone running with CVE ID from `ScenarioEntityCatalog.device_cves[0]`
pre-populated in `cve_registry` with `cvss_base_score = 8.1`, `cvss_severity = "HIGH"`,
and `PRISM_DTU_MODE=true` (DTU loopback override for SSRF guard),
and `PRISM_NVD_API_KEY` env var set (AD-017 credential resolution):

when `HttpLookupSource::enrich_single` is called with the CVE ID:
1. Builds URL: `base_url + interpolate("/rest/json/cves/2.0?cveId=${input}", {input: cve_id})`
   via `Interpolator::interpolate` from `pipeline.rs` (reuse, do not re-implement).
2. Appends auth: `?apiKey=<resolved_credential_value>` (auth = query_param, param_name = "apiKey").
3. Issues GET via `self.client` (built with `build_http_client_with_timeout(30)` — CLAUDE.md).
4. Response body is JSON; `extract_at_path(body, "$.vulnerabilities[0].cve.metrics.cvssMetricV31[0].cvssData")`
   returns the CVSS data subtree.
5. Returns `Ok(Some(subtree))` — fields `baseScore`, `baseSeverity`, `vectorString` present.

then the UDF layer extracts `subtree["baseScore"]` → `cvss_base_score >= 7.0` (Float),
`subtree["baseSeverity"]` → `cvss_severity = "HIGH"` (String),
`subtree["vectorString"]` → `cvss_vector` (String, non-empty).
Note: `extract_at_path` from `pipeline.rs` handles the JSONPath notation — confirmed
confirmed cves.rs; NOT /nvd/cves/{id}; wire field names are camelCase per serde rename_all.

Integration test requires the demo server running with `scenario.enabled = true`.

Red Gate: `test_enrichment_pivot_002_nvd_http_lookup_resolves_scenario_cve_high_cvss`

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
**SUPERSEDED by AC-020 (v1.3 F-004 rigor tightening). AC-020 is the binding specification.
AC-010 retained for traceability with the original D-1179 finding; the Red Gate test
`test_enrichment_pivot_002_sec001_wasm_enrich_wraps_spawn_blocking` is now listed under AC-020.**
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

### AC-013 — InfusionType::HttpLookup + new config types added to infusion/mod.rs
(traces to BC-2.19.001 postcondition — InfusionLoader::parse handles "http_lookup" source type;
ADR-040 v2.0 D8.2/D8.3)

Given `InfusionLoader::parse` processes a TOML spec with `type = "http_lookup"`:
1. `InfusionType::HttpLookup` variant exists alongside `LocalLookup` and `Plugin` in
   `crates/prism-spec-engine/src/infusion/mod.rs`.
2. `HttpLookupAuthType` enum exists (`#[non_exhaustive]`) with variants:
   `QueryParam { param_name: String }`, `BearerHeader`, `ApiKeyHeader { header_name: String }`.
3. `HttpLookupCredentialConfig` struct exists (`#[non_exhaustive]`) with fields:
   `ref_name: String`, `env_var: String`, `auth: HttpLookupAuthType`.
4. `HttpLookupConfig` struct exists (`#[non_exhaustive]`) with fields:
   `base_url: String`, `url_template: String`, `method: String`, `response_path: String`,
   `credential: Option<HttpLookupCredentialConfig>`.
5. `InfusionSpec` has a new field `http_lookup_config: Option<HttpLookupConfig>`.
6. All four new public types carry `#[non_exhaustive]`.
7. `ci.yml` `EXPECTED` count incremented by 4 for the new non-exhaustive types.
8. CLAUDE.md sentence tracking the count updated accordingly.

when `InfusionLoader::parse` processes the NVD spec:
then `spec.infusion_type == InfusionType::HttpLookup` and `spec.http_lookup_config.is_some()`.

Also: validation enforced at parse time (D8.3):
- `url_template` must contain `${input}` — else `InfusionError::InvalidFieldSpec`.
- `method` must be `"GET"` or `"POST"` — else `InfusionError::InvalidFieldSpec`.
- `base_url` must be non-empty — else `InfusionError::InvalidFieldSpec`.
- `response_path` must be non-empty — else `InfusionError::InvalidFieldSpec`.

Red Gate:
`test_enrichment_pivot_002_http_lookup_infusion_type_parses_nvd_spec`
`test_enrichment_pivot_002_http_lookup_parse_rejects_missing_input_placeholder`
`test_enrichment_pivot_002_http_lookup_parse_rejects_invalid_method`

### AC-014 — PluginError::EnrichCallFailed + InfusionError::PluginCallFailed variants added
(traces to BC-2.19.001 postcondition — plugin-type source error propagation;
ADR-040 v2.0 D5 — E-PLUGIN-023 / E-INFUSE-008)

Given `prism-core/src/error.rs`:
1. `PluginError::EnrichCallFailed { plugin_id: String, reason: String }` variant exists.
   Display: `"plugin '{plugin_id}' enrich-single call failed: {reason}"`.
2. `InfusionError::PluginCallFailed { plugin_id: String, infusion_id: String, reason: String }`
   variant exists. Display: `"E-INFUSE-008: plugin infusion call failed for '{infusion_id}'
   via plugin '{plugin_id}': {reason}"`.
3. `map_plugin_error_to_infusion_error` in `plugin_bridge.rs` maps
   `PluginError::EnrichCallFailed { plugin_id, reason }` → `InfusionError::PluginCallFailed`.

when `PluginRuntime::enrich_single` returns `Err(PluginError::EnrichCallFailed { .. })`:
then `PluginInfusionSource::enrich_single` propagates it as
`Err(InfusionError::PluginCallFailed { .. })` with the correct message format.

Red Gate:
`test_enrichment_pivot_002_plugin_enrich_call_failed_maps_to_infusion_error`

### AC-015 — InfusionError HttpLookup variants added (E-INFUSE-009/010/011)
(traces to BC-2.19.001 postcondition — HttpLookup error propagation;
ADR-040 v2.0 D8.5 — E-INFUSE-009/010/011)

Given `prism-core/src/error.rs`:
1. `InfusionError::HttpLookupFailed { spec_path: String, status_code: Option<u16>, message: String }`
   exists. Display: `"E-INFUSE-009: HTTP lookup failed for infusion '{infusion_id}' (spec: '{spec_path}'): {message}"`.
   `message` MUST NOT contain credential values (AD-017).
2. `InfusionError::CredentialResolutionFailed { spec_path: String, credential_ref: String }`
   exists. Display: `"E-INFUSE-010: credential resolution failed for infusion '{infusion_id}' (spec: '{spec_path}'): credential '{credential_ref}' not available at call time"`.
   Env var name MUST NOT appear in message.
3. `InfusionError::SsrfRejected { spec_path: String }` exists.
   Display: `"E-INFUSE-011: SSRF protection rejected infusion '{infusion_id}' (spec: '{spec_path}'): base_url resolves to a private or loopback address; set PRISM_DTU_MODE=true to override for test/demo deployments"`.
   Resolved IP address MUST NOT appear in message (CWE-209).
4. Error taxonomy rows for E-INFUSE-009, E-INFUSE-010, E-INFUSE-011 exist in
   `prd-supplements/error-taxonomy.md` — already allocated at v1.88 (confirmed).
   Implementer must verify the taxonomy rows are present (no new allocation needed).

Red Gate:
`test_enrichment_pivot_002_http_lookup_failed_error_format_excludes_credentials`
`test_enrichment_pivot_002_credential_resolution_failed_excludes_env_var_name`
`test_enrichment_pivot_002_ssrf_rejected_error_excludes_resolved_ip`

### AC-016 — HttpLookupSource implements InfusionSource with credential AD-017 discipline
(traces to BC-2.19.001 postcondition — HttpLookup source executes enrich_single;
BC-2.19.001 invariant — credential data does not leak; ADR-040 v2.0 D8.4)

Given `HttpLookupSource` in `crates/prism-spec-engine/src/infusion/sources/http_lookup.rs`:
1. Constructed via `HttpLookupSource::new(client, config, spec_path)`.
   - `client` is built via `build_http_client_with_timeout(30)` from `pipeline.rs`
     (CLAUDE.md §Conventions — `.timeout(30s)` mandatory; do NOT call `reqwest::Client::new()`).
2. `enrich_single` steps (D8.4):
   a. Resolves credential from `env::var(&config.credential.env_var)` → `Err(E-INFUSE-010)` if absent.
   b. Builds URL via `Interpolator::interpolate` from `pipeline.rs` — do NOT re-implement.
   c. Applies auth (QueryParam / BearerHeader / ApiKeyHeader) — credential value used for HTTP,
      MUST NOT appear in logs or error messages after the call.
   d. Issues HTTP GET via `self.client`; non-2xx or network error → `Err(E-INFUSE-009)`.
   e. Parses response body as JSON; parse failure → `Err(E-INFUSE-009)`.
   f. Extracts `response_path` subtree via `extract_at_path` from `pipeline.rs` — do NOT re-implement.
      If subtree not found: `Ok(None)` (no enrichment data for this input, not an error).
   g. Returns `Ok(Some(subtree_value))`.
3. Credential value MUST NOT be stored in struct fields after construction (AD-017 / INV-INFUSE-005).
4. `tracing::warn!(event_type = "http_lookup_enrich_failed", ...)` emitted on E-INFUSE-009.
   `tracing::warn!(event_type = "http_lookup_ssrf_rejected", ...)` emitted on E-INFUSE-011.
   Both event_types registered in BC-2.16.002 §Postconditions catalog (SAP-1 obligation).

Red Gate (unit tests with mock HTTP server — no real NVD calls in CI):
`test_enrichment_pivot_002_http_lookup_source_enrich_single_calls_url_template`
`test_enrichment_pivot_002_http_lookup_source_extracts_response_path_fields`
`test_enrichment_pivot_002_http_lookup_source_returns_none_on_path_not_found`
`test_enrichment_pivot_002_http_lookup_source_returns_err_on_non_2xx`

### AC-017 — HttpLookupSource SSRF validation at construction time
(traces to BC-2.19.001 precondition — spec is structurally valid before UDF registration;
ADR-040 v2.0 D8.4 SSRF section; CWE-918)

Given `HttpLookupSource::new` is called with a `base_url` whose hostname resolves to:
- RFC-1918 private range (10.x, 172.16-31.x, 192.168.x)
- Loopback (127.x, ::1)
- Link-local (169.254.x)
and `PRISM_DTU_MODE` env var is NOT set:

then `HttpLookupSource::new` returns `Err(InfusionError::SsrfRejected { spec_path })`.
The resolved IP address MUST NOT appear in the error message (CWE-209).
`tracing::warn!(event_type = "http_lookup_ssrf_rejected", infusion_id = ..., spec_path = ..., ...)` is emitted.

Given `PRISM_DTU_MODE=true` is set:
then the same private/loopback `base_url` is accepted (DTU override for test isolation).

If DNS resolution fails at construction time: fail closed (return E-INFUSE-011), do not accept.

Red Gate:
`test_enrichment_pivot_002_ssrf_rejects_private_base_url_without_dtu_mode`
`test_enrichment_pivot_002_ssrf_accepts_private_base_url_with_dtu_mode`

### AC-018 — prism-nvd-infusion crate and build recipe removed (D9)
(traces to BC-2.19.001 postcondition — no dead-code plugin crate with NVD on HttpLookup path;
ADR-040 v2.0 D9 — NVD plugin crate disposal)

Given NVD enrichment is served by `InfusionType::HttpLookup`:
1. `crates/plugins/prism-nvd-infusion/` directory does NOT exist on the filesystem.
2. Root `Cargo.toml` `exclude` list does NOT contain `crates/plugins/prism-nvd-infusion`.
   (It was never in `members`, but any `exclude` entry must also be removed.)
3. `Justfile` does NOT contain a `build-plugin-nvd-infusion` recipe.
4. No CI workflow step builds `nvd-lookup.prx`.
5. No `.prism/plugins/nvd-lookup.prx` artifact is referenced anywhere.

Red Gate (script or compile check in `just check`):
`test_enrichment_pivot_002_nvd_plugin_crate_removed`
(This test verifies: `assert!(!Path::new("crates/plugins/prism-nvd-infusion").exists())` —
a unit test in prism-spec-engine that asserts the crate directory is gone.)

### AC-019 — Val-lift fix covered by a unit test exercising the real production path
(traces to BC-2.19.001 postcondition — plugin-type source Val-lift behaves correctly;
ADR-040 v2.0 D2; F-001 CRIT closure)

Given `PluginRuntime::enrich_single` with the D2 Val-lift fix applied:
1. A test invokes `PluginRuntime::enrich_single` directly with a test WAT fixture or
   minimal `.prx` Component Model binary that exports `enrich-single` returning
   `Val::Option(Some(Box<Val::String("{}".to_string()))))`.
2. The test asserts the return value is `Ok(Some(serde_json::Value::Object(_)))` —
   NOT `Ok(None)` (the pre-fix unconditional behavior).

This test MUST drive the PRODUCTION code path in `plugin/mod.rs` — it MUST NOT
re-implement the lift logic inline (F-003 test-rigor constraint: test asserts via
the real `PluginRuntime::enrich_single` function, not a reimplementation).

Additional sub-case: when the WAT/component returns `Val::Option(None)`:
then the result is `Ok(None)` (no enrichment data — not an error).

Additional sub-case: when `results[0]` is `Val::String("not-json")` (unexpected Val variant
inside Option<Some>):
then the result is `Err(PluginError::EnrichCallFailed { .. })` and
`event_type = "plugin_enrich_json_parse_error"` is emitted.

Red Gate:
`test_enrichment_pivot_002_val_lift_fix_option_some_returns_json_value`
`test_enrichment_pivot_002_val_lift_fix_option_none_returns_ok_none`
`test_enrichment_pivot_002_val_lift_fix_unexpected_val_returns_enrich_call_failed`

### AC-020 — spawn_blocking wraps the real InfusionAsyncUdf invoke path
(traces to BC-2.19.001 postcondition — plugin-type source executes without blocking tokio;
F-004 test-rigor constraint: test asserts via the real InfusionAsyncUdf::invoke_with_args path;
AC-010 test-rigor tightening per ADR-040 v2.0 scope boundary note)

Given `InfusionAsyncUdf::invoke_with_args` (in `prism-query`) calls
`InfusionSource::enrich_single` from an async DataFusion context:
the call to the synchronous `PluginInfusionSource::enrich_single` (which invokes the WASM
runtime synchronously) MUST be wrapped in `tokio::task::spawn_blocking`.

The Red Gate test MUST assert this by:
(a) Calling `InfusionAsyncUdf::invoke_with_args` from within a `tokio::test` runtime, OR
(b) Asserting the code path calls `spawn_blocking` via a test that would deadlock or
    time out if the call were blocking (preferred for definitive proof).

The test MUST NOT be satisfied by asserting that a helper function exists with the word
"spawn_blocking" in it — it must demonstrate that the async path does not block the
tokio runtime.

If `InfusionAsyncUdf` already uses `spawn_blocking` from PIVOT-001, this test verifies
and cites the location as a code-pointer. The test still MUST pass against the production
code path — do not skip it if it was "already done".

Red Gate:
`test_enrichment_pivot_002_sec001_wasm_enrich_wraps_spawn_blocking`

(This AC supersedes the same-named test from v1.2 AC-010 with explicit F-004 rigor constraint
that the test drives the real InfusionAsyncUdf path, not a reimplementation.)

### AC-021 — SAP-1: BC-2.16.002 catalog rows for 4 new event_types added in same commit
(traces to BC-2.19.001 postcondition — structured event catalog is kept in sync; SAP-1;
ADR-040 v2.0 D5 and D8.7)

Given the implementer adds tracing emissions for the following four `event_type` values:
| event_type | Source | Condition |
|------------|--------|-----------|
| `plugin_enrich_json_parse_error` | `PluginRuntime::enrich_single` D2 | JSON parse error on returned string |
| `plugin_enrich_unexpected_val` | `PluginRuntime::enrich_single` D2 | Unexpected Val variant in results[0] |
| `http_lookup_enrich_failed` | `HttpLookupSource::enrich_single` | E-INFUSE-009 condition |
| `http_lookup_ssrf_rejected` | `HttpLookupSource::new` | E-INFUSE-011 construction rejection |

then ALL FOUR must have corresponding rows in the Canonical Structured Event Catalog
in BC-2.16.002 §Postconditions (SAP-1 obligation), added in the SAME COMMIT as the
emission sites (PG-LP11-001 precedent).

Fields for `plugin_enrich_json_parse_error`: `plugin_id`, `error`.
Fields for `plugin_enrich_unexpected_val`: `plugin_id`.
Fields for `http_lookup_enrich_failed`: `infusion_id`, `spec_path`, `status_code` (optional u16).
Fields for `http_lookup_ssrf_rejected`: `infusion_id`, `spec_path`.
None of these fields may include credential values or resolved IP addresses.

Red Gate (adversary verification, not a Rust unit test):
Adversary must run `rg 'event_type\s*=' crates/ --type rust` and verify each of the
four values above has a matching BC-2.16.002 catalog row. Missing row = P1 finding.

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

| # | Test Name | Crate | BC Clause | Type | AC |
|---|-----------|-------|-----------|------|----|
| 1 | `test_enrichment_pivot_002_threatintel_toml_loads_and_registers_3_udfs` | prism-spec-engine | BC-2.19.001 postcondition | unit | AC-001 |
| 2 | `test_enrichment_pivot_002_nvd_toml_loads_as_http_lookup_and_registers_3_udfs` | prism-spec-engine | BC-2.19.001 postcondition | unit | AC-002 |
| 3 | `test_enrichment_pivot_002_threatintel_plugin_resolves_scenario_ioc_as_malicious` | prism-spec-engine integration | BC-2.19.001 postcondition | integration (demo server) | AC-003 |
| 4 | `test_enrichment_pivot_002_nvd_http_lookup_resolves_scenario_cve_high_cvss` | prism-spec-engine integration | BC-2.19.001 postcondition | integration (demo server) | AC-004 |
| 5 | `test_enrichment_pivot_002_enrich_threatintel_pipe_stage_returns_malicious_for_scenario_iocs` | prism-spec-engine or prism-query integration | BC-2.19.001 postcondition | integration (demo server) | AC-005 |
| 6 | `test_enrichment_pivot_002_enrich_nvd_pipe_stage_returns_high_cvss_for_scenario_cves` | prism-spec-engine or prism-query integration | BC-2.19.001 postcondition | integration (demo server) | AC-006 |
| 7 | `test_enrichment_pivot_002_sec001_udf_name_rejects_sql_injection_chars` | prism-spec-engine | BC-2.19.001 precondition | unit | AC-007 |
| 8 | `test_enrichment_pivot_002_sec001_udf_name_rejects_leading_digit` | prism-spec-engine | BC-2.19.001 precondition | unit | AC-007 |
| 9 | `test_enrichment_pivot_002_sec001_udf_name_accepts_valid_identifiers` | prism-spec-engine | BC-2.19.001 precondition | unit | AC-007 |
| 10 | `test_enrichment_pivot_002_sec002_plugin_infusion_source_config_not_pub` | prism-spec-engine | BC-2.19.001 invariant | unit/compile-fail | AC-008 |
| 11 | `test_enrichment_pivot_002_sec003_sandbox_violation_url_not_in_warn_log` | prism-spec-engine | BC-2.19.001 invariant | unit | AC-009 |
| 12 | `test_enrichment_pivot_002_sec001_wasm_enrich_wraps_spawn_blocking` | prism-spec-engine or prism-query | BC-2.19.001 postcondition | unit/integration | AC-020 (F-004 rigor: real InfusionAsyncUdf path) |
| 13 | `test_enrichment_pivot_002_sec003_path_traversal_rejected_for_dotdot_plugin_ref` | prism-spec-engine | BC-2.19.001 precondition | unit | AC-011 |
| 14 | `test_enrichment_pivot_002_sec003_path_within_plugin_dir_accepted` | prism-spec-engine | BC-2.19.001 precondition | unit | AC-011 |
| 15 | `test_enrichment_pivot_002_sec002_load_all_error_does_not_leak_absolute_path` | prism-spec-engine | BC-2.19.001 invariant | unit | AC-012 |
| 16 | `test_enrichment_pivot_002_http_lookup_infusion_type_parses_nvd_spec` | prism-spec-engine | BC-2.19.001 postcondition | unit | AC-013 |
| 17 | `test_enrichment_pivot_002_http_lookup_parse_rejects_missing_input_placeholder` | prism-spec-engine | BC-2.19.001 precondition | unit | AC-013 |
| 18 | `test_enrichment_pivot_002_http_lookup_parse_rejects_invalid_method` | prism-spec-engine | BC-2.19.001 precondition | unit | AC-013 |
| 19 | `test_enrichment_pivot_002_plugin_enrich_call_failed_maps_to_infusion_error` | prism-spec-engine | BC-2.19.001 postcondition | unit | AC-014 |
| 20 | `test_enrichment_pivot_002_http_lookup_failed_error_format_excludes_credentials` | prism-spec-engine | BC-2.19.001 invariant | unit | AC-015 |
| 21 | `test_enrichment_pivot_002_credential_resolution_failed_excludes_env_var_name` | prism-spec-engine | BC-2.19.001 invariant | unit | AC-015 |
| 22 | `test_enrichment_pivot_002_ssrf_rejected_error_excludes_resolved_ip` | prism-spec-engine | BC-2.19.001 invariant | unit | AC-015 |
| 23 | `test_enrichment_pivot_002_http_lookup_source_enrich_single_calls_url_template` | prism-spec-engine | BC-2.19.001 postcondition | unit (mock HTTP) | AC-016 |
| 24 | `test_enrichment_pivot_002_http_lookup_source_extracts_response_path_fields` | prism-spec-engine | BC-2.19.001 postcondition | unit (mock HTTP) | AC-016 |
| 25 | `test_enrichment_pivot_002_http_lookup_source_returns_none_on_path_not_found` | prism-spec-engine | BC-2.19.001 postcondition | unit (mock HTTP) | AC-016 |
| 26 | `test_enrichment_pivot_002_http_lookup_source_returns_err_on_non_2xx` | prism-spec-engine | BC-2.19.001 postcondition | unit (mock HTTP) | AC-016 |
| 27 | `test_enrichment_pivot_002_ssrf_rejects_private_base_url_without_dtu_mode` | prism-spec-engine | BC-2.19.001 precondition | unit | AC-017 |
| 28 | `test_enrichment_pivot_002_ssrf_accepts_private_base_url_with_dtu_mode` | prism-spec-engine | BC-2.19.001 precondition | unit | AC-017 |
| 29 | `test_enrichment_pivot_002_nvd_plugin_crate_removed` | prism-spec-engine | BC-2.19.001 postcondition | unit (fs assert) | AC-018 |
| 30 | `test_enrichment_pivot_002_val_lift_fix_option_some_returns_json_value` | prism-spec-engine | BC-2.19.001 postcondition | unit | AC-019 (F-003 rigor: real PluginRuntime path) |
| 31 | `test_enrichment_pivot_002_val_lift_fix_option_none_returns_ok_none` | prism-spec-engine | BC-2.19.001 postcondition | unit | AC-019 |
| 32 | `test_enrichment_pivot_002_val_lift_fix_unexpected_val_returns_enrich_call_failed` | prism-spec-engine | BC-2.19.001 postcondition | unit | AC-019 |

**Total Red Gate tests: 32** (v1.3; up from 15 in v1.2)

Integration tests (tests 3-6) require demo server running with scenario.enabled = true.
Per SID-1, these tests are NOT `#[ignore]`'d unless blocking on a live external service.
An in-process demo server harness is sufficient and should NOT be `#[ignore]`'d.

Security gate tests 7-15 are unit tests. They MUST pass before any PIVOT-002 code merges.
Tests 7-9 validate identifier-regex validation in `InfusionLoader::parse`. Test 10 validates
`pub(crate)` visibility. Test 11 uses `tracing_test` or equivalent span capture to assert URL
redaction (must drive the real `map_plugin_error_to_infusion_error` — not a reimplementation).
Test 12 (now AC-020) verifies `spawn_blocking` wrapping via the real `InfusionAsyncUdf::invoke_with_args`
path (F-004 rigor). Tests 13-14 verify path canonicalization in `InfusionRegistry::load_spec_with_runtime`
wiring point (D6). Test 15 verifies path stripping from MCP-surfaced errors.

HttpLookup tests (16-28) use a mock HTTP server (e.g., `wiremock` crate) — no real NVD calls in CI.
Tests 30-32 (Val-lift fix) use a WAT fixture or minimal Component Model binary that returns the
expected `Val::Option(Some(Box<Val::String(...))))` — must drive the production `PluginRuntime::enrich_single`
function, not reimplemented logic (F-003 rigor).

---

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file, v1.3) | ~7,000 |
| ADR-040 v2.0 (full — implementation directives) | ~3,000 |
| S-DEMO-ENRICHMENT-PIVOT-001 spec (context for plugin bridge) | ~1,500 |
| `prism-dtu-threatintel/src/types.rs` (response struct definitions) | ~800 |
| `prism-dtu-threatintel/src/routes/lookup.rs` (route shape) | ~600 |
| `prism-dtu-nvd/src/types.rs` (CveRecord struct + nested types) | ~1,000 |
| `prism-dtu-nvd/src/routes/cves.rs` (route shape) | ~600 |
| `crates/prism-spec-engine/src/pipeline.rs` (Interpolator + extract_at_path + build_http_client_with_timeout — HttpLookup reuse) | ~1,500 |
| `crates/prism-spec-engine/src/plugin/mod.rs` (PluginRuntime::enrich_single + enrich_batch — Val-lift fix site) | ~2,500 |
| `crates/prism-spec-engine/src/infusion/mod.rs` (InfusionType + new types location) | ~1,000 |
| `crates/prism-spec-engine/src/infusion/loader.rs` (InfusionLoader::parse) | ~1,200 |
| `crates/prism-spec-engine/src/infusion/plugin_bridge.rs` (security gate + Val-lift context) | ~1,200 |
| `crates/prism-core/src/error.rs` (new error variant locations) | ~800 |
| `threatintel.infusion.toml` (authored spec) | ~300 |
| `nvd.infusion.toml` (authored spec, http_lookup) | ~300 |
| `prism-threatintel-infusion/src/lib.rs` (WASM plugin with wit_bindgen) | ~1,500 |
| BC-2.19.001 v1.6 (full) | ~1,500 |
| BC-2.16.002 (catalog rows section — for 4 new event_types) | ~800 |
| Test files (32 stubs × ~50 lines each) | ~4,800 |
| `prism-infusion-plugin.wit` (WIT interface — reference for D2 Val types) | ~400 |
| Tool outputs (nextest, clippy, demo server integration, mock HTTP, tracing_test) | ~2,000 |
| **Total estimate** | **~35,800** |

At ~200k context window, this is ~17.9% — within the 20-30% ceiling.
Note: story was split into Sub-burst A (file creation) and Sub-burst B (index/traceability
update) per burst-splitting rule (>8 artifacts). Token budget above is for the implementer's
working context, not the story-writer burst size.

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
- [ ] Read `crates/prism-spec-engine/src/pipeline.rs` — locate `Interpolator::interpolate`,
  `extract_at_path`, `build_http_client_with_timeout` — these are REUSED in HttpLookupSource;
  understand their signatures before implementing D8.4
- [ ] For each TOML field, verify a corresponding field exists in the DTU response struct (SAP-2 check)

**Phase 0: Error variant foundation (ADR-040 D5 + D8.5 — MUST be first, all error paths depend on it)**

- [ ] Read `crates/prism-core/src/error.rs` — locate `PluginError` and `InfusionError` enums
- [ ] Add `PluginError::EnrichCallFailed { plugin_id: String, reason: String }` variant (D5).
  Display: `"plugin '{plugin_id}' enrich-single call failed: {reason}"`.
- [ ] Add `InfusionError::PluginCallFailed { plugin_id: String, infusion_id: String, reason: String }` (E-INFUSE-008).
  Display: `"E-INFUSE-008: plugin infusion call failed for '{infusion_id}' via plugin '{plugin_id}': {reason}"`.
- [ ] Add `InfusionError::HttpLookupFailed { spec_path: String, status_code: Option<u16>, message: String }` (E-INFUSE-009).
  Display: `"E-INFUSE-009: HTTP lookup failed for infusion '{infusion_id}' (spec: '{spec_path}'): {message}"`.
  `message` MUST NOT contain credential values.
- [ ] Add `InfusionError::CredentialResolutionFailed { spec_path: String, credential_ref: String }` (E-INFUSE-010).
  Display: `"E-INFUSE-010: credential resolution failed for infusion '{infusion_id}' (spec: '{spec_path}'): credential '{credential_ref}' not available at call time"`.
- [ ] Add `InfusionError::SsrfRejected { spec_path: String }` (E-INFUSE-011).
  Display: `"E-INFUSE-011: SSRF protection rejected infusion '{infusion_id}' (spec: '{spec_path}'): base_url resolves to a private or loopback address; set PRISM_DTU_MODE=true to override for test/demo deployments"`.
- [ ] Verify error-taxonomy v2.26 rows for E-INFUSE-009/010/011 and E-PLUGIN-023 are present
  (already allocated — implementer confirms, does NOT re-allocate)
- [ ] Update `map_plugin_error_to_infusion_error` in `plugin_bridge.rs` to map
  `PluginError::EnrichCallFailed` → `InfusionError::PluginCallFailed`
- [ ] Write failing tests 19, 20, 21, 22 (FAIL first) per AC-014/AC-015
- [ ] Verify all pass; run `just check-fast` to confirm no broken callsites

**Phase 1: HttpLookup type infrastructure (ADR-040 D8.2/D8.3/D8.6)**

- [ ] Write failing tests 16, 17, 18 (FAIL first) per AC-013
- [ ] Add `InfusionType::HttpLookup` variant to `infusion/mod.rs` with doc comment:
  "HTTP lookup (single GET → JSONPath extraction). PROHIBITED in detection rule filters
  (E-RULE-012) — API-backed."
- [ ] **E-INFUSE-004 message sync (architect ruling S-1.14-REDO Q3):** In the same commit,
  update `prism_core::error::InfusionError::UnknownSourceType` (or the equivalent `Display`
  implementation in `error.rs`) to include `, http_lookup` in the valid-types string so the
  error message matches BC-2.19.001 v1.9 E-INFUSE-004: `"Unknown source type '...'. Valid types:
  maxmind_mmdb, csv, json_lookup, plugin, http_lookup."`. Until this commit lands,
  `error.rs` intentionally omits `http_lookup`.
- [ ] Add `HttpLookupAuthType` (`#[non_exhaustive]`), `HttpLookupCredentialConfig` (`#[non_exhaustive]`),
  `HttpLookupConfig` (`#[non_exhaustive]`) to `infusion/mod.rs`
- [ ] Add `http_lookup_config: Option<HttpLookupConfig>` field to `InfusionSpec`
- [ ] Extend `InfusionLoader::parse` with `"http_lookup"` arm; parse `[source.http]` and
  `[source.credential]` blocks into `HttpLookupConfig` + `HttpLookupCredentialConfig`;
  apply all D8.3 validations (url_template must contain `${input}`, method GET/POST only,
  base_url non-empty, response_path non-empty)
- [ ] Update `EXPECTED=66` in `ci.yml` to account for new `#[non_exhaustive]` types count
  (4 new types: `InfusionType::HttpLookup` adds to the variant count; `HttpLookupAuthType`,
  `HttpLookupCredentialConfig`, `HttpLookupConfig` are the new structs/enums — count carefully;
  update BOTH `ci.yml` EXPECTED and CLAUDE.md sentence tracking the count)
- [ ] Verify tests 16, 17, 18 pass

**Phase 2: HttpLookupSource implementation (ADR-040 D8.4)**

- [ ] Create `crates/prism-spec-engine/src/infusion/sources/http_lookup.rs`
- [ ] Write failing tests 23, 24, 25, 26, 27, 28 (FAIL first) per AC-016/AC-017
  (use `wiremock` or equivalent mock HTTP server — no real NVD calls in CI)
- [ ] Implement `HttpLookupSource::new(client, config, spec_path)`:
  - `client` via `build_http_client_with_timeout(30)` from `pipeline.rs` (MUST NOT call `reqwest::Client::new()`)
  - SSRF validation at construction time: for non-DTU mode, check `base_url` hostname resolves
    outside RFC-1918/loopback/link-local; fail closed if DNS unavailable (E-INFUSE-011)
  - `PRISM_DTU_MODE=true` env var bypasses SSRF check
  - Emit `tracing::warn!(event_type = "http_lookup_ssrf_rejected", ...)` on rejection
- [ ] Implement `HttpLookupSource::enrich_single(input, input_type)`:
  - Resolve credential from `env::var` (E-INFUSE-010 on missing)
  - Build URL via `Interpolator::interpolate` from `pipeline.rs` (REUSE — do NOT re-implement)
  - Apply auth (QueryParam/BearerHeader/ApiKeyHeader)
  - Issue HTTP via `self.client` (30s timeout on client)
  - Non-2xx or network error → `Err(InfusionError::HttpLookupFailed)` (E-INFUSE-009) +
    `tracing::warn!(event_type = "http_lookup_enrich_failed", ...)`
  - Extract subtree via `extract_at_path` from `pipeline.rs` (REUSE)
  - `None` from extract_at_path → `Ok(None)` (no data, not an error)
  - Return `Ok(Some(subtree_value))`
  - Credential value MUST NOT appear in logs or error messages after use
- [ ] Wire `InfusionRegistry::load_spec_with_runtime` branch for `InfusionType::HttpLookup` (D8.6):
  construct `HttpLookupSource` using `build_http_client_with_timeout(30)` from `pipeline.rs`
- [ ] Update `InfusionRegistry::is_api_backed` to return `true` for `InfusionType::HttpLookup` (D8.6)
- [ ] Verify tests 23-28 pass

**Phase 3: NVD TOML spec — http_lookup type (ADR-040 D8.1)**

- [ ] Write failing test 2 (FAIL first):
  `test_enrichment_pivot_002_nvd_toml_loads_as_http_lookup_and_registers_3_udfs`
- [ ] Author `{config_dir}/infusions/nvd.infusion.toml` (U16: location per loader.rs:45 —
  NOT repo-root `specs/infusions/`) per ADR-040 D8.1 schema:
  - `[infusion] type = "http_lookup"` (NOT `"plugin"`)
  - `[source.http]` block per D8.1 (base_url, url_template, method, response_path)
  - `[source.credential]` block: ref, env_var, auth = "query_param", param_name = "apiKey"
  - `[[infusion.fields]]` for cvss_base_score/cvss_severity/cvss_vector with source_column
  - DTU column grounding per SAP-2 (baseScore/baseSeverity/vectorString — camelCase)
  - ALSO declare in Armis sensor TOML: `device_cves_first` column (String) per Ruling 1b
- [ ] Verify test 2 passes

**Phase 4: NVD plugin crate removal (ADR-040 D9)**

- [ ] Confirm `crates/plugins/prism-nvd-infusion/` does NOT exist (it was never built in
  v1.2; if it does exist, delete the directory)
- [ ] Remove any reference to `prism-nvd-infusion` from root `Cargo.toml` `exclude` list
- [ ] Remove `build-plugin-nvd-infusion` recipe from `Justfile` if it exists
- [ ] Remove any CI workflow step building `nvd-lookup.prx`
- [ ] Write failing test 29 (FAIL first): `test_enrichment_pivot_002_nvd_plugin_crate_removed`
  (assert `!Path::new("crates/plugins/prism-nvd-infusion").exists()`)
- [ ] Verify test 29 passes

**Phase 5: ThreatIntel TOML spec (type = "plugin")**

- [ ] Write failing test 1 (FAIL first):
  `test_enrichment_pivot_002_threatintel_toml_loads_and_registers_3_udfs`
- [ ] Author `{config_dir}/infusions/threatintel.infusion.toml` (U16: location per loader.rs:45 —
  NOT repo-root `specs/infusions/`; match table layout from
  `crates/prism-spec-engine/fixtures/threat_intel_plugin.infusion.toml`):
  - `infusion_id = "threat_intel"`
  - `[source] type = "plugin"`, `plugin_ref = "threatintel-lookup.prx"`
  - `[source.credential] ref = "<keyring-reference-per-ADR-032>"` — auth via ?key= or Bearer token
    (NOT X-Admin-Token; admin surface only per lookup.rs lines 20-53)
  - `[[infusion.fields]] name = "threat_is_known_malicious" type = "Boolean"`
  - `[[infusion.fields]] name = "threat_score" type = "Integer"`
  - `[[infusion.fields]] name = "threat_sources" type = "Json"` — array per lookup.rs response
  - `[pipe_stage] adds_columns = ["threat_is_known_malicious", "threat_score", "threat_sources"]`
  - DTU clone routes: GET /v3/ip/:ip, GET /v3/domain/:domain, GET /v3/hash/:hash
    (NOT a unified endpoint); plugin dispatches on input IOC type to correct route
  - No production API URL in TOML (ADR-028)
- [ ] Verify test 1 passes

**Phase 6: PluginRuntime::enrich_single Val-lift fix (ADR-040 D2 — F-001 CRIT)**

- [ ] Read `crates/prism-spec-engine/src/plugin/mod.rs` — locate `enrich_single` (lines ~903-984)
  and `enrich_batch`; locate `dispatch_plugin_acquire_token` (lines ~621-869) for two-phase
  function lookup pattern reference
- [ ] Write failing tests 30, 31, 32 (FAIL first) per AC-019:
  WAT fixture or minimal Component Model binary returning `Val::Option(Some(Box<Val::String("{}"))))`
  — test must drive PRODUCTION `PluginRuntime::enrich_single`, NOT reimplemented logic (F-003)
- [ ] Replace Component Model path of `enrich_single` per D2 contract:
  - Params: `[Val::String(input_value), Val::String(input_type)]` (NOT `Val::S32`)
  - Results buffer: `vec![Val::Option(None)]`
  - After `func.call Ok(_)`: lift `results[0]` via the pattern in D2 §Return-value lift
    (Val::Option(Some(Box<Val::String(json)))) → serde_json → Ok(Some(v));
     Val::Option(None) → Ok(None);
     unexpected Val → Err(PluginError::EnrichCallFailed) + tracing event)
  - Two-phase function lookup: bare "enrich-single" first, then interface-scoped name
- [ ] Apply same D2 treatment to `enrich_batch` method
- [ ] Verify tests 30, 31, 32 pass

**Phase 7: prism-threatintel-infusion WASM plugin (ADR-040 D3/D4)**

- [ ] Create `crates/plugins/prism-threatintel-infusion/` crate with standalone
  `Cargo.toml` (own `[workspace]` table, wasm32-wasip1 cdylib; mirror
  `crates/plugins/ocsf-complex-transforms/` structure; NO reqwest/tokio — HTTP via
  host WIT import host.http-request per prism-infusion-plugin.wit)
- [ ] Copy `prism-spec-engine/wit/prism-infusion-plugin.wit` into
  `prism-threatintel-infusion/wit/prism-infusion-plugin.wit` (D3 WIT file placement)
- [ ] Pre-flight: verify wit-bindgen 0.51 / wasm-tools 1.248.0 / wasmtime 44
  component-encoding alignment (U14/Ruling 4)
- [ ] Add exclusion to root `Cargo.toml` `exclude` list (NOT workspace members)
- [ ] Add Justfile recipe `build-plugin-threatintel-infusion` per D4:
  `cargo build --manifest-path crates/plugins/prism-threatintel-infusion/Cargo.toml
  --target wasm32-wasip1 --release && wasm-tools component new
  target/wasm32-wasip1/release/prism_threatintel_infusion.wasm
  --adapt wasi_snapshot_preview1.wasm -o .prism/plugins/threatintel-lookup.prx`
  (wasi adapter path must match `build-plugin-crowdstrike-oauth2` recipe)
- [ ] Write failing test 3 (FAIL first) in integration test
- [ ] Implement `src/lib.rs` with `wit_bindgen::generate!({ world: "infusion-plugin-world", path: "wit", exports: { "prism:infusion-plugin/infusion-plugin": Plugin } })` (D3);
  `struct Plugin` implements `Guest` trait; `fn enrich_single(input_value, input_type) -> Option<String>`:
  dispatch on input_type to correct DTU route via `host::http_request` WIT import;
  IP → GET /v3/ip/:ip, domain → GET /v3/domain/:domain, hash → GET /v3/hash/:hash;
  auth: ?key= query param OR Authorization: Bearer <token>;
  serialize response as JSON `{"threat_score": N, "threat_is_known_malicious": bool, "threat_sources": [...]}`
  (ARRAY not singular — confirmed lookup.rs); return `Some(json_string)`;
  `fn enrich_batch` delegates to `enrich_single` per item; `export!(Plugin)` at bottom
- [ ] Verify test 3 passes (with demo server running)

**Phase 8: pipe stage integration tests (AC-005/AC-006)**

- [ ] Write failing tests 5, 6 (FAIL first): enrich pipe stage integration tests
  (`test_enrichment_pivot_002_enrich_threatintel_pipe_stage_...` and
  `test_enrichment_pivot_002_enrich_nvd_pipe_stage_...`)
- [ ] Verify both tests pass with demo server at stage >= 3

**Phase 9: BC-2.16.002 catalog rows for 4 new event_types (SAP-1 — MUST be in same commit as emission sites)**

- [ ] Locate BC-2.16.002 §Postconditions Canonical Structured Event Catalog
- [ ] Add row for `plugin_enrich_json_parse_error`: fields `plugin_id`, `error`; recurrence one per
  failing invocation; ADR-040 v2.0 D5
- [ ] Add row for `plugin_enrich_unexpected_val`: fields `plugin_id`; recurrence one per failing
  invocation; ADR-040 v2.0 D5
- [ ] Add row for `http_lookup_enrich_failed`: fields `infusion_id`, `spec_path`, `status_code`
  (optional u16); recurrence one per failing call; ADR-040 v2.0 D8.7
- [ ] Add row for `http_lookup_ssrf_rejected`: fields `infusion_id`, `spec_path`; recurrence one
  per construction-time rejection; ADR-040 v2.0 D8.7
- [ ] Verify these four rows are committed in the same commit that introduces the emission sites
  (PG-LP11-001 precedent — implementer-owned)

**Phase 10: Final gates**

- [ ] SAP-2 post-implementation sweep: re-read DTU types.rs and routes — confirm each TOML
  column has a matching DTU struct field; document parity table in PR description
- [ ] SAP-1 probe: `rg 'event_type\s*=' crates/ --type rust` — verify all 4 new event_types
  have BC-2.16.002 catalog rows; any emission without a catalog row = P1 finding
- [ ] Run `just check` — all 32 Red Gate tests pass; zero clippy warnings; fmt clean
- [ ] Verify perimeter: `prism-threatintel-infusion` MUST NOT import `prism-spec-engine`,
  `prism-sensors`, or `prism-query` (plugin binary ABI is a separate compilation unit)
- [ ] Verify `crates/plugins/prism-nvd-infusion/` does NOT exist (AC-018 / D9)
- [ ] Verify `Justfile` does NOT contain `build-plugin-nvd-infusion` (AC-018 / D9)
- [ ] Verify `ci.yml` EXPECTED count updated for new `#[non_exhaustive]` types (AC-013)
- [ ] Verify CLAUDE.md non-exhaustive count sentence updated (AC-013)

---

## Previous Story Intelligence

**S-DEMO-ENRICHMENT-PIVOT-001 (direct predecessor):**
- `InfusionLoader::parse` for `source.type = "plugin"` is operational
- `plugin_bridge::enrich_via_plugin` is wired (or has annotated todo! for S-1.15)
- DataFusion UDF registration is wired in prism-query
- `is_api_backed()` is implemented for plugin-type UDFs
- `Interpolator::interpolate`, `extract_at_path`, `build_http_client_with_timeout` are in
  `pipeline.rs` — REUSED by HttpLookupSource (read their signatures before implementing D8.4)

**S-DEMO-DTU-LIVE-SCENARIO-001-B (substrate):**
- `ThreatIntelClone::new_with_scenario(entities)` pre-populates `fixture_registry` with
  scenario IOCs as `FixtureKey::Malicious` at construction time
- `NvdClone::new_with_scenario(entities)` pre-populates `cve_registry` with scenario CVEs
  at `cvss_base_score = 8.1`, `cvss_severity = "HIGH"`
- At stage >= 3 (Exfil), `ioc_hashes`, `ioc_ips`, `ioc_domains` are visible in StageMask

**ADR-040 v2.0 ratified canonical implementation order (23 steps):**
1. Add PluginError::EnrichCallFailed, InfusionError::PluginCallFailed (E-INFUSE-008)
2. Add InfusionError::HttpLookupFailed (E-INFUSE-009), CredentialResolutionFailed (E-INFUSE-010), SsrfRejected (E-INFUSE-011)
3. Verify E-PLUGIN-023/E-INFUSE-008/009/010/011 taxonomy rows present (v1.88)
4. Update map_plugin_error_to_infusion_error for EnrichCallFailed
5. Update map_plugin_error_to_infusion_error for EnrichCallFailed
6. Add InfusionType::HttpLookup, HttpLookupAuthType, HttpLookupCredentialConfig, HttpLookupConfig; add http_lookup_config to InfusionSpec
7. Extend InfusionLoader::parse for "http_lookup" (D8.3 validations)
8. Write failing Red Gate tests for HttpLookupSource (unit, mock HTTP server — tests 23-28)
9. Implement HttpLookupSource per D8.4 (reuse Interpolator + extract_at_path + build_http_client_with_timeout)
10. Wire InfusionRegistry::load_spec_with_runtime HttpLookup branch (D8.6)
11. Update InfusionRegistry::is_api_backed for HttpLookup (D8.6)
12. Update nvd.infusion.toml to type = "http_lookup" per D8.1
13. Remove prism-nvd-infusion crate and build-plugin-nvd-infusion recipe (D9)
14. Update EXPECTED=66 in ci.yml + CLAUDE.md count sentence for new #[non_exhaustive] types
15. Write failing Red Gate tests for Val-lift (tests 30-32, WAT/component fixture)
16. Fix PluginRuntime::enrich_single per D2 (Val::String params + Val::Option result lift)
17. Fix PluginRuntime::enrich_batch per D2 (same treatment)
18. Write validate_plugin_ref_path and its Red Gate tests (AC-011, D6, tests 13-14)
19. Copy WIT file into prism-threatintel-infusion/wit/ (D3)
20. Implement prism-threatintel-infusion/src/lib.rs with wit_bindgen::generate! (D3)
21. Add build-plugin-threatintel-infusion Justfile recipe (D4)
22. Add four BC-2.16.002 catalog rows (SAP-1, same commit as emission sites):
    plugin_enrich_json_parse_error, plugin_enrich_unexpected_val,
    http_lookup_enrich_failed, http_lookup_ssrf_rejected
23. Run just check — all 32 Red Gate tests pass

**From PLUGIN-MIGRATION-001-D/E lessons:**
- SAP-2: adversary reads DTU types.rs and routes/ — do not rely on story description alone
- SID-1: integration tests driving in-process demo server are NOT `#[ignore]`'d
- F-003: security gate test AC-009 must assert URL redaction via the real `map_plugin_error_to_infusion_error`
  — not a reimplementation of the redaction logic inline in the test
- F-004: spawn_blocking test (AC-020) must drive the real `InfusionAsyncUdf::invoke_with_args`
  path — a test that only checks the helper function exists does NOT satisfy this gate

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| TOML infusion specs MUST be grounded against DTU clone route surfaces — NOT assumed production API URLs | ADR-028/ADR-031 + WO-D1109 §Story 2 DTU grounding requirement | SAP-2 adversary probe (read DTU types.rs + routes/ before review) |
| Every TOML `[[infusion.fields]]` column MUST have a matching field in the corresponding DTU route response struct | SAP-2 (CLAUDE.md §SAP-2) | Adversary: read prism-dtu-threatintel/src/types.rs + routes/ and prism-dtu-nvd/src/types.rs + routes/ |
| `nvd.infusion.toml` MUST use `type = "http_lookup"` (NOT `"plugin"`) — ADR-040 v2.0 D8.1 | ADR-040 v2.0 D7/D9 | Adversary: confirm `[infusion] type = "http_lookup"` in nvd.infusion.toml |
| `prism-nvd-infusion` crate MUST NOT exist — NVD on HttpLookup path, plugin crate is dead code | ADR-040 v2.0 D9 | Adversary: `assert!(!Path::new("crates/plugins/prism-nvd-infusion").exists())` |
| `Justfile` MUST NOT contain `build-plugin-nvd-infusion` recipe | ADR-040 v2.0 D4 (retracted for NVD) | Adversary: `grep -n build-plugin-nvd-infusion Justfile` must produce no output |
| `HttpLookupSource` MUST use `build_http_client_with_timeout(30)` from `pipeline.rs` — NOT `reqwest::Client::new()` | CLAUDE.md §Conventions (forbidden pattern) | Adversary: grep for `Client::new()` in `sources/http_lookup.rs` |
| `HttpLookupSource` MUST use `Interpolator::interpolate` from `pipeline.rs` for URL templating — NOT re-implement | ADR-040 v2.0 D8.4 (reuse directive) | Adversary: verify no inline template interpolation in http_lookup.rs |
| `HttpLookupSource` MUST use `extract_at_path` from `pipeline.rs` for JSONPath extraction — NOT re-implement | ADR-040 v2.0 D8.4 (reuse directive) | Adversary: verify no inline JSONPath implementation in http_lookup.rs |
| `base_url` hostname SSRF check at construction time; fail closed if DNS unavailable; PRISM_DTU_MODE bypass | ADR-040 v2.0 D8.4 + D8.5 E-INFUSE-011 | AC-017; test_enrichment_pivot_002_ssrf_* |
| Credential value MUST NOT appear in any log field, error message, or struct field after the HTTP call | AD-017 + INV-INFUSE-005 | Adversary: code review of HttpLookupSource::enrich_single credential handling |
| `PluginRuntime::enrich_single` MUST pass `Val::String` params (NOT `Val::S32`) and lift `Val::Option` result | ADR-040 v2.0 D2 — F-001 CRIT | AC-019; test_enrichment_pivot_002_val_lift_fix_* |
| `InfusionRegistry::is_api_backed` MUST return `true` for `InfusionType::HttpLookup` (same as Plugin) | ADR-040 v2.0 D8.6; BC-2.19.003 / INV-INFUSE-003 / E-RULE-012 | AC-002 `registry.is_api_backed("cvss_base_score")` assertion |
| All four new `event_type` values MUST have BC-2.16.002 catalog rows in same commit as emission sites | SAP-1 / CLAUDE.md §SAP-1 / PG-LP11-001 | AC-021; adversary SAP-1 probe |
| Plugin crate `prism-threatintel-infusion` MUST NOT depend on `prism-spec-engine`, `prism-sensors`, or `prism-query` | INV-PERIMETER-COMPLIANCE-001 (BC-2.06.020) + ADR-036 v2.2 §2.5 | Adversary: grep Cargo.toml for forbidden deps |
| WASM guest crate MUST NOT use reqwest or tokio (no sockets in sandbox) | U9 research-confirmed 2026-06-12; HTTP via host WIT import host.http-request | Adversary: grep for reqwest/tokio in `prism-threatintel-infusion/Cargo.toml` |
| Credential references in TOML specs MUST use reference-based model (no inline credential values) | AD-017 + ADR-032 | Adversary AD-017 probe |
| `plugin_ref` path wiring for validate_plugin_ref_path MUST be in `InfusionRegistry::load_spec_with_runtime` (not in `load_plugin` or `load_all_plugins`) | ADR-040 v2.0 D6 | AC-011; adversary: verify call site is in load_spec_with_runtime |
| `InfusionField.name` MUST match `^[a-zA-Z][a-zA-Z0-9_]*$` — validated at parse time before UDF registration | DRIFT-PIVOT-UDFNAME-VALIDATION-001 (D-1190 SEC-001 CWE-20) | AC-007; test_enrichment_pivot_002_sec001_* |
| `PluginInfusionSource.config` field MUST be `pub(crate)` (not `pub`) before PIVOT-002 populates credentials | DRIFT-PIVOT-PLUGINCONFIG-PUB-FIELD-001 (D-1190 SEC-002 CWE-200) | AC-008 |
| `SandboxViolation.url` MUST NOT appear in WARN-level tracing; assertion via real `map_plugin_error_to_infusion_error` | DRIFT-PIVOT-SANDBOXVIOLATION-URL-LOG-001 (D-1190 SEC-003 CWE-209); F-003 | AC-009 |
| Sync WASM call in async UDF MUST be wrapped in `spawn_blocking`; assertion via real `InfusionAsyncUdf::invoke_with_args` | DRIFT-PIVOT-PLUGINID-INFUSIONID-001 SEC-001 MANDATORY gate (D-1179); F-004 | AC-020 |
| `plugin_ref` path MUST be canonicalized and verified against plugin_dir before file I/O | DRIFT-PIVOT-PLUGINPATH-TRAVERSAL-001 (D-1179 SEC-003 CWE-22) | AC-011 |
| `InfusionLoader::load_all` errors MUST NOT surface absolute filesystem paths in MCP responses | DRIFT-PIVOT-LOADALL-PATH-DISCLOSURE-001 (D-1179 SEC-002 CWE-209) | AC-012 |
| `ci.yml` EXPECTED count and CLAUDE.md non-exhaustive count sentence MUST be updated for new `#[non_exhaustive]` types | CLAUDE.md §Conventions `#[non_exhaustive]` discipline | AC-013 task; `just check` CI gate |

**Forbidden Dependencies:**
- `crates/plugins/prism-threatintel-infusion` MUST NOT depend on `prism-spec-engine`, `prism-sensors`,
  or `prism-query` (plugin binary ABI is a separate compilation unit; out-of-workspace crates have
  no workspace dependency access)
- `prism-threatintel-infusion` MUST NOT depend on `reqwest` or `tokio` (no sockets in WASM sandbox — U9);
  HTTP goes through host WIT import `host.http-request`
- `crates/plugins/prism-nvd-infusion/` MUST NOT exist — crate is retired per ADR-040 D9

---

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| `serde` / `serde_json` | 1.x (workspace — host side); guest: check ocsf-complex-transforms deps | DTU response deserialization; HttpLookupSource JSON response parsing |
| `wit-bindgen` | 0.51 (confirmed 2026-06-12) | WIT guest bindings for host.http-request WIT import in `prism-threatintel-infusion` WASM plugin only (D3) |
| `wasm-tools` | 1.248.0 (confirmed 2026-06-12) | Justfile `build-plugin-threatintel-infusion` pipeline: `wasm-tools component new --adapt wasi_snapshot_preview1.wasm` (ThreatIntel only; NVD uses HttpLookup, no wasm-tools needed for NVD path) |
| `wasmtime` | 44.x (host, already in prism-spec-engine) | Plugin loading and execution (host side); Val-lift fix (D2) uses `wasmtime::component::Val` |
| `toml` | 0.8.x (workspace) | Infusion spec TOML parsing (in prism-spec-engine, already present) |
| `reqwest` | HOST ONLY (project-pinned; `.timeout(30s)` mandatory via `build_http_client_with_timeout(30)`) | `HttpLookupSource.client` (HttpLookup path) AND host_functions.rs (Plugin path). MUST NOT call `reqwest::Client::new()`. NOT in WASM guest crates (U9: no sockets in WASM sandbox). |
| `tokio` | HOST ONLY (1.x workspace) | Async runtime for HOST reqwest client (HttpLookupSource + host_functions.rs) — NOT in WASM guest crates |
| `tracing-test` | 0.2.x (already in workspace Cargo.lock; used in prism-query) | Dev-dep for AC-009 WARN log assertion. Add `tracing-test = "0.2"` to `[dev-dependencies]` in `prism-spec-engine/Cargo.toml` if not already present. |
| `regex` or `once_cell` + `Lazy<Regex>` or `str::chars()` | workspace | Identifier validation regex for AC-007. Prefer `str::chars()` (zero-dep char-by-char validation); use `once_cell::sync::Lazy` (already workspace dep) only if a compiled `Regex` is cleaner. Do NOT add `regex` as a new production dep. |
| `wiremock` or `httpmock` | 0.6.x / 0.7.x (dev-dep only) | Mock HTTP server for AC-016/AC-017 unit tests (no real NVD calls in CI). Add to `[dev-dependencies]` in `prism-spec-engine/Cargo.toml`. Choose whichever is already in workspace Cargo.lock; if neither, use `wiremock = "0.6"`. |

**MSRV:** Rust stable per `rust-toolchain.toml` (host crates); wasm32-wasip1 target for
`prism-threatintel-infusion` guest crate only (out-of-workspace — standalone [workspace] per Ruling 3).
NVD path requires NO wasm32 toolchain (HttpLookup is pure host-side Rust).

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `{config_dir}/infusions/threatintel.infusion.toml` | CREATE | ThreatIntel plugin-type infusion spec grounded against prism-dtu-threatintel routes (U16: location per InfusionLoader loader.rs:45 — NOT `specs/infusions/`) |
| `{config_dir}/infusions/nvd.infusion.toml` | CREATE | NVD **http_lookup-type** infusion spec per ADR-040 D8.1 (NOT plugin-type); grounded against prism-dtu-nvd routes; includes device_cves_first Armis TOML column (U17/Ruling 1b) |
| `crates/prism-spec-engine/src/infusion/sources/http_lookup.rs` | CREATE | HttpLookupSource: implements InfusionSource; reuses Interpolator/extract_at_path/build_http_client_with_timeout from pipeline.rs; SSRF validation at construction; E-INFUSE-009/010/011 |
| `crates/prism-core/src/error.rs` | MODIFY | Add PluginError::EnrichCallFailed (E-PLUGIN-023), InfusionError::PluginCallFailed (E-INFUSE-008), InfusionError::HttpLookupFailed (E-INFUSE-009), CredentialResolutionFailed (E-INFUSE-010), SsrfRejected (E-INFUSE-011) |
| `crates/prism-spec-engine/src/infusion/mod.rs` | MODIFY | Add InfusionType::HttpLookup, HttpLookupAuthType, HttpLookupCredentialConfig, HttpLookupConfig (#[non_exhaustive] on all); add http_lookup_config field to InfusionSpec |
| `crates/prism-spec-engine/src/infusion/loader.rs` | MODIFY | Add "http_lookup" arm in InfusionLoader::parse; D8.3 validations (url_template ${input}, method, base_url, response_path) |
| `crates/prism-spec-engine/src/infusion/sources/mod.rs` (or equivalent) | MODIFY | Expose http_lookup module |
| `crates/prism-spec-engine/src/infusion/plugin_bridge.rs` | MODIFY | Update map_plugin_error_to_infusion_error for EnrichCallFailed → PluginCallFailed; SandboxViolation URL redaction (AC-009) |
| `crates/prism-spec-engine/src/plugin/mod.rs` | MODIFY | Val-lift fix for enrich_single + enrich_batch (D2); two-phase function lookup |
| `crates/prism-spec-engine/src/infusion/registry.rs` (or mod.rs) | MODIFY | load_spec_with_runtime: add HttpLookup branch (D8.6); validate_plugin_ref_path wiring before load_plugin (D6); is_api_backed update for HttpLookup |
| `crates/plugins/prism-threatintel-infusion/Cargo.toml` | CREATE | Out-of-workspace standalone crate (U18/Ruling 3); own [workspace]; excluded from root Cargo.toml; wasm32-wasip1 cdylib; wit-bindgen = "0.51"; serde_json = "1"; NO reqwest/tokio |
| `crates/plugins/prism-threatintel-infusion/wit/prism-infusion-plugin.wit` | CREATE | Copy of prism-spec-engine/wit/prism-infusion-plugin.wit (D3: self-contained plugin crate) |
| `crates/plugins/prism-threatintel-infusion/src/lib.rs` | CREATE | WASM guest plugin: wit_bindgen::generate!; dispatches IOC type → correct DTU route via host::http_request; threat_sources as JSON array; export!(Plugin) |
| `crates/plugins/prism-nvd-infusion/` | DO NOT CREATE (ADR-040 D9) | NVD uses HttpLookup built-in; this crate is retired before it exists |
| `Cargo.toml` (workspace root) | MODIFY | Add `crates/plugins/prism-threatintel-infusion` to `exclude` list (NOT members); remove any `prism-nvd-infusion` entry |
| `Justfile` | MODIFY | Add `build-plugin-threatintel-infusion` recipe per D4; ensure NO `build-plugin-nvd-infusion` recipe |
| `ci.yml` | MODIFY | Update `EXPECTED` count for new `#[non_exhaustive]` types added in this story |
| `CLAUDE.md` | MODIFY | Update non-exhaustive count sentence (AC-013) |
| Armis sensor TOML spec | MODIFY | Add `device_cves_first` column declaration (String; generator projection in STORY-003) |
| `prd-supplements/error-taxonomy.md` | VERIFY (no new rows needed) | Confirm E-INFUSE-009/010/011 and E-PLUGIN-023 rows present at v1.88 (already allocated); implementer verifies, does not re-allocate |
| Integration test file (TBD location) | CREATE | Tests 3-6 (pipe stage integration) + Val-lift unit tests (30-32) against demo server and WAT fixture |

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
| EC-011 | `nvd.infusion.toml` has `url_template` without `${input}` placeholder | `InfusionLoader::parse` returns `Err(InfusionError::InvalidFieldSpec)` at parse time; spec rejected (AC-013 D8.3 validation) |
| EC-012 | `HttpLookupSource::enrich_single` called but `PRISM_NVD_API_KEY` env var is unset | Returns `Err(InfusionError::CredentialResolutionFailed)` (E-INFUSE-010); message includes logical ref name `"nvd.api_key"` but NOT the env var name `PRISM_NVD_API_KEY` (AD-017) |
| EC-013 | NVD DTU returns HTTP 404 for an unknown CVE ID | `HttpLookupSource::enrich_single` returns `Err(InfusionError::HttpLookupFailed)` (E-INFUSE-009) with `status_code = Some(404)` |
| EC-014 | NVD DTU response body is not valid JSON | `HttpLookupSource::enrich_single` returns `Err(InfusionError::HttpLookupFailed)` (E-INFUSE-009) with `status_code = None`; `message` describes parse error but does NOT include raw response content exceeding 200 bytes |
| EC-015 | `response_path` in `nvd.infusion.toml` returns `None` from `extract_at_path` (e.g., no cvssMetricV31 data) | `HttpLookupSource::enrich_single` returns `Ok(None)` — no enrichment data available, not an error |
| EC-016 | `PluginRuntime::enrich_single` guest returns `Val::Option(None)` (no enrichment data) | Host correctly returns `Ok(None)` — NOT `Err(PluginError::EnrichCallFailed)` (AC-019 D2 lift; correct None branch) |
| EC-017 | `PluginRuntime::enrich_single` guest returns `Val::Option(Some(Box<Val::String("not-json"))))` | Host returns `Err(PluginError::EnrichCallFailed)` with `event_type = "plugin_enrich_json_parse_error"` (AC-019 D2 lift) |
| EC-018 | `base_url = "http://192.168.1.100"` in an http_lookup spec without `PRISM_DTU_MODE=true` | `HttpLookupSource::new` returns `Err(InfusionError::SsrfRejected)` at construction time; resolved IP NOT in error message (AC-017; CWE-209) |

---

## Architecture Mapping

| Component | Module | Pure/Effectful | Anchor |
|-----------|--------|---------------|--------|
| `{config_dir}/infusions/threatintel.infusion.toml` | `{config_dir}/infusions/` (config artifact per loader.rs:45 — NOT `specs/infusions/`) | Pure (declarative config) | WO-D1109 §Story 2; U16 |
| `{config_dir}/infusions/nvd.infusion.toml` | `{config_dir}/infusions/` (http_lookup type — ADR-040 D8.1) | Pure (declarative config) | WO-D1109 §Story 2; ADR-040 v2.0 D8.1 |
| `HttpLookupSource` | `prism-spec-engine/src/infusion/sources/http_lookup.rs` (SS-19) | Effectful (HTTP call to DTU/NVD; SSRF validation at construction) | ADR-040 v2.0 D8.4 |
| `prism-threatintel-infusion` plugin | `crates/plugins/prism-threatintel-infusion/` (SS-17, out-of-workspace) | Effectful (HTTP call to DTU via host WIT import) | AD-019 / CAP-032 / ADR-040 v2.0 D3 |
| `prism-nvd-infusion` plugin | RETIRED — does NOT exist (ADR-040 D9) | N/A | ADR-040 v2.0 D9 |
| `InfusionLoader` (loading these specs) | `prism-spec-engine/src/infusion/loader.rs` (SS-19) | Mixed (provided by 001; extended in this story for "http_lookup" arm) | BC-2.19.001 |
| `PluginRuntime::enrich_single` (Val-lift fix) | `prism-spec-engine/src/plugin/mod.rs` (SS-17) | Effectful (WASM call via wasmtime Component Model) | ADR-040 v2.0 D2; F-001 CRIT |

---

## SAP-2 Compliance Note

Per CLAUDE.md §SAP-2, the adversary for this story MUST:
1. Read `crates/prism-dtu-threatintel/src/routes/lookup.rs` — confirm three routes: GET /v3/ip/:ip,
   GET /v3/domain/:domain, GET /v3/hash/:hash (NOT a unified /threatintel/lookup/{v} endpoint);
   confirm auth: ?key= or Authorization: Bearer; confirm `threat_sources` is JSON array (NOT `threat_source` string)
2. Read `crates/prism-dtu-nvd/src/types.rs` and `routes/cves.rs` — confirm route: GET /rest/json/cves/2.0?cveId=;
   confirm wire field names are camelCase (baseScore/baseSeverity/vectorString — NOT snake_case in JSON);
   NVD auth: ?apiKey= param; confirm `response_path = "$.vulnerabilities[0].cve.metrics.cvssMetricV31[0].cvssData"`
   correctly addresses the subtree containing baseScore/baseSeverity/vectorString
3. For every column in `threatintel.infusion.toml` and `nvd.infusion.toml`:
   - Verify the column name matches a field in the DTU response struct (adversary reads actual source)
   - Verify the column type matches the DTU Rust type (threat_sources: Json array; cvss_base_score: Float)
   - For `nvd.infusion.toml`: verify `source_column` values (baseScore, baseSeverity, vectorString) are
     present in the CVSS data subtree addressed by `response_path` in DTU types.rs
4. Verify `prism-threatintel-infusion/Cargo.toml` does NOT list reqwest or tokio (U9 gate)
5. Verify `prism-threatintel-infusion` is in root Cargo.toml `exclude` list, NOT `members` (U18/Ruling 3 gate)
6. Verify `crates/plugins/prism-nvd-infusion/` does NOT exist on filesystem (ADR-040 D9 gate)
7. Verify `nvd.infusion.toml` uses `type = "http_lookup"` (NOT `"plugin"`) — ADR-040 D7/D9 gate
8. Verify `HttpLookupSource` implementation does NOT re-implement Interpolator or extract_at_path
   (must call functions from `pipeline.rs` — no inline URL template substitution or JSONPath logic)

Column in TOML with no DTU equivalent = **P1 CRITICAL** finding. `threat_source` (singular) as a declared column = **P1 CRITICAL** (wrong; JSON response has array field `threat_sources`). reqwest/tokio in WASM guest = **P1 CRITICAL** (U9). `nvd.infusion.toml` with `type = "plugin"` = **P1 CRITICAL** (ADR-040 D9 violation). `prism-nvd-infusion` crate existing = **P1 CRITICAL** (ADR-040 D9 violation).

---

## Story Changelog

| Version | Date | Change |
|---------|------|--------|
| v1.6 | 2026-07-08 | **Reconciling pin round (pass-4 closures): error-taxonomy v1.88→v2.26. One live version-pin cite updated: §Phase-0 tasks checklist item `Verify error-taxonomy v1.88 rows`. Lines 574 and 1303 (`error-taxonomy.md … at v1.88`) left unchanged as historical allocation facts. Also syncing frontmatter version 1.5→1.6 (no v1.5 changelog row was written for prior frontmatter bump; combined per POL-23). Historical changelog rows left unchanged per POL-29. AC semantics UNCHANGED.** |
| v1.4 | 2026-06-18 | **BC-2.19.001 v1.9 propagation + E-INFUSE-004 sync obligation (architect ruling S-1.14-REDO Q3).** (1) Frontmatter BC comment updated: PO amendment complete (v1.8 added `http_lookup` to E-INFUSE-004); `# BC status: pending PO authorship` marker replaced with resolved status note. (2) BC table row: BC-2.19.001 v1.6 → v1.9; E-INFUSE-004 valid types now include `http_lookup`. (3) Behavioral Contracts section: PO routing note replaced with E-INFUSE-004 sync obligation — when PIVOT-002 adds `InfusionType::HttpLookup`, implementer MUST update `InfusionError::UnknownSourceType` Display in `error.rs` to include `, http_lookup` in the same commit; until then, error.rs intentionally omits it. (4) Phase 1 tasks: bullet added to add `http_lookup` to the valid-types string in `error.rs` in the same commit as `InfusionType::HttpLookup`. Story version bumped v1.3→v1.4. |
| v1.3 | 2026-06-17 | ADR-040 v2.0 dual-path pivot re-scope (story-writer). Title updated. **NVD → HttpLookup (AC-002 rewrite):** AC-002 now specifies `type = "http_lookup"` (NOT `"plugin"`), full ADR-040 D8.1 TOML schema (url_template, response_path, source_column fields, PRISM_DTU_MODE bypass note), `HttpLookupSource` as the source (not `PluginInfusionSource`). Red Gate test 2 renamed to `..._nvd_toml_loads_as_http_lookup_...`. **New HttpLookup infrastructure ACs (AC-013 through AC-017):** InfusionType::HttpLookup variant + four new #[non_exhaustive] types (AC-013); PluginError::EnrichCallFailed + InfusionError::PluginCallFailed variants (AC-014); InfusionError::HttpLookupFailed/CredentialResolutionFailed/SsrfRejected (AC-015); HttpLookupSource implementation reusing Interpolator/extract_at_path/build_http_client_with_timeout from pipeline.rs (AC-016); SSRF validation at construction time with PRISM_DTU_MODE bypass (AC-017). **NVD plugin crate removal (AC-018):** explicit AC to assert `prism-nvd-infusion/` does not exist. **AC-003 updated (ThreatIntel WASM):** now includes Val::String params + Val::Option result lift contract (ADR-040 D2) + wit_bindgen::generate! requirement (D3). **AC-004 rewritten (NVD now HttpLookup):** drives HttpLookupSource.enrich_single via mock HTTP/DTU, not a WASM plugin. **F-001 CRIT Val-lift fix (AC-019):** new AC requiring PluginRuntime::enrich_single test to drive PRODUCTION path (F-003 rigor), covering Option::Some/None/unexpected-Val sub-cases. **AC-020 (spawn_blocking F-004 rigor):** supersedes v1.2 AC-010 with explicit requirement that test drives real InfusionAsyncUdf::invoke_with_args (not a reimplementation). **AC-021 (SAP-1 BC-2.16.002):** explicit AC for all 4 new event_types (plugin_enrich_json_parse_error, plugin_enrich_unexpected_val, http_lookup_enrich_failed, http_lookup_ssrf_rejected) to have BC-2.16.002 catalog rows in same commit. **F-005 file-location fix:** AC-002 now says `{config_dir}/infusions/nvd.infusion.toml` (NOT `specs/infusions/`); Architecture Mapping updated from stale `specs/infusions/` to `{config_dir}/infusions/`. **PO routing flag:** BC-2.19.001 E-INFUSE-004 valid-types list needs `http_lookup` addition — surfaced in Behavioral Contracts section. **Points:** 8 → 13 (scope expanded: HttpLookup built-in + Val-lift CRIT fix + 4 new error variants + SSRF validation; NVD WASM plugin removed). **Red Gate tests:** 15 → 32. **Tasks restructured:** 10 phases (Phase 0 error foundation first, then HttpLookup infrastructure, NVD crate removal, ThreatIntel TOML, Val-lift CRIT fix, ThreatIntel WASM plugin, pipe stage tests, BC-2.16.002 catalog rows, final gates). **Token budget:** ~20,350 → ~35,800 tokens (~17.9% of 200k, within ceiling). **ADR-040 v2.0 23-step implementation order** added to Previous Story Intelligence. **Frontmatter:** crates_touched adds prism-core (new error variants); comment updated for single new WASM plugin crate + NVD retirement note. |
| v1.2 | 2026-06-17 | D-1205 MANDATORY security gate fold-in (story-writer pre-TDD pass). Added AC-007 through AC-012 covering all 6 DRIFT items: DRIFT-PIVOT-UDFNAME-VALIDATION-001 (UDF name identifier validation, AC-007), DRIFT-PIVOT-PLUGINCONFIG-PUB-FIELD-001 (PluginInfusionSource.config encapsulation, AC-008), DRIFT-PIVOT-SANDBOXVIOLATION-URL-LOG-001 (SandboxViolation URL redaction, AC-009), DRIFT-PIVOT-PLUGINID-INFUSIONID-001 SEC-001 sync-WASM spawn_blocking gate (AC-010), DRIFT-PIVOT-PLUGINPATH-TRAVERSAL-001 (path traversal rejection, AC-011), DRIFT-PIVOT-LOADALL-PATH-DISCLOSURE-001 (path stripping in MCP errors, AC-012). Red Gate tests expanded from 6 to 15 (9 new security gate tests added). Tasks section expanded with MANDATORY SECURITY GATES section (6 security gate tasks, each with FAIL-first Red Gate test discipline). Architecture Compliance Rules updated with 6 security gate enforcement rows. Edge Cases EC-007 through EC-010 added. Token budget updated (~20,350 tokens, ~10.2% of 200k). Points remain 8 (security gates are implementation of existing code, not new functional scope). remove-uncertainty validate: wasmtime=44 (confirmed in prism-spec-engine/Cargo.toml), wasm-tools=1.248.0, wit-bindgen=0.51 — all confirmed correct in workspace. No new technology uncertainties found. |
| v1.1 | 2026-06-12 | D-1109 remove-uncertainty closure: U1/U9/U10/U11/U12/U13/U14/U15/U16/U17/U18 applied (scanner + research-agent + architect rulings 1-4, WO-D1109 v1.1). enrich syntax → function-call form. reqwest/tokio removed from WASM guest crates (host WIT import host.http-request). ThreatIntel endpoints corrected: three separate routes GET /v3/{ip,domain,hash}/:value (NOT unified). threat_sources declared as Json array (NOT string). NVD route corrected: GET /rest/json/cves/2.0?cveId= (confirmed cves.rs). NVD wire names corrected to camelCase. cargo-component removed; Justfile wasm-tools pipeline (Ruling 4). ThreatIntel auth corrected to ?key=/Bearer (NOT X-Admin-Token). NVD auth: ?apiKey=. Spec location: config_dir/infusions/ (loader.rs:45). Plugin crates relocated to crates/plugins/ out-of-workspace (Ruling 3). Ruling 1b: NVD enrich field device_cves_first; Armis TOML column scope in this story, generator projection in STORY-003. |
| v1.0 | 2026-06-12 | Initial draft per WO-D1109 §Story 2. Grounded against DTU clone route surfaces. Depends on 001; blocks 003. Two new plugin crates. SAP-2 compliance note included. BC-2.19.001 as primary anchor; PO to confirm at materialization. |
