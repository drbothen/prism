# Claroty xDome v1 — Delivered Feature Inventory (Live-Validation Substrate)

**Scope:** Every delivered Claroty-xDome-path feature as it exists on `develop @362e4f85`.
**Purpose:** Substrate for a v1 live-validation test matrix against the real xDome tenant.
**Mode:** READ-ONLY analysis. No code changed. No commit performed.
**Author:** codebase-analyzer (vsdd-factory)
**Date:** 2026-08-21

> **CRITICAL DELTA CONTEXT.** `S-ADR058-OCSF-ROUTING-001` (ADR-058 Stage 2, OCSF field-name
> routing) is **IN DELIVERY, NOT MERGED** (story `status: draft`, TDD gate open per STATE.md
> D-2263; depends on `S-ADR058-OCSF-COERCION-001` which merged @362e4f85 as PR #240). On
> **current develop**, `SensorSpec` has **no `ocsf_column_naming` field** (RG-001 is a
> compile-fail) — so every Claroty Arrow column is named by **`col.name`** and the
> `ocsf_field = "..."` annotations in `claroty.sensor.toml` are **dormant metadata**
> (surfaced only as `prism_describe` column *descriptions*). **Post-ROUTING-001**, Claroty
> flips to `ocsf_column_naming = true`: Tier-1 columns get OCSF underscore-flattened Arrow
> names (`finding_info.uid` → `finding_info_uid`), Tier-2 columns (no `ocsf_field`) collapse
> into ONE `raw_extensions` JSON column, and `class_uid` routing changes. **Validate against
> the naming regime you actually ship.** Every dimension below marks `[CURRENT develop]` vs
> `[POST-ROUTING-001 target]` where they differ.

---

## 0. Summary Table — Delivered Features

| Feature | Status (current develop) | Source file / symbol | How to verify live |
|---|---|---|---|
| Claroty sensor TOML spec (4 tables) | DELIVERED | `crates/prism-sensors/specs/claroty.sensor.toml` | `prism_describe` returns `claroty_alerts/audit_logs/devices/device_alert_relations` |
| `auth_type = bearer_static` | DELIVERED (D-747 LOCKED) | `claroty.sensor.toml §sensor_id`, header line 36 | Health probe with valid/invalid token → 200 vs 401/403 |
| Credential ref model (AD-017 opaque) | DELIVERED | `claroty.sensor.toml [[credential_refs]] name="bearer_token"`; `prism-credentials/src/resolution.rs` | Store token via credential CLI; token never in AI context |
| Base URL from env | DELIVERED | `base_url = "${env.CLAROTY_INSTANCE_URL}"` | Set env → boot resolves instance URL |
| POST-for-read + offset_limit pagination (all 4 tables) | DELIVERED (Gap-CL-004 closed, S-DEMO-CLAROTY-PAGINATION-001) | `claroty.sensor.toml [[tables.steps]]`; `spec-engine/src/pipeline.rs::build_request` | Query >1000 rows → multiple pages fetched |
| Trailing-slash route fidelity | DELIVERED (Gap-CL-001 closed) | TOML path_templates (`/api/v1/alerts/` etc.); DTU `prism-dtu-claroty/src/clone.rs NormalizePathLayer::trim_trailing_slash` | Live xDome accepts slashed paths |
| `audit_logs` time-box push-down + 7-day default | DELIVERED / MERGED PR #239 | `prism-bin/src/spec_driven_adapter.rs::build_claroty_audit_filter_by`; `pushdown.rs::extract_time_window_from_ast` | Query with/without `WHERE timestamp > 'T'` → POST body `filter_by` |
| `audit_logs.timestamp` INDEX eligibility | DELIVERED | `claroty.sensor.toml audit_logs.timestamp options=["INDEX"]` | Push-down only fires for INDEX datetime col |
| TLS hardening (rustls-tls + http2) | DELIVERED (ADR-050; PR #237, PR #241) | `spec_driven_adapter.rs::build_http_client_with_timeout`; RG-008 in `defect_adapter_tls_xdome_live_001.rs` | HTTPS to real tenant; no native-tls Keychain stall |
| 30s HTTP client timeout | DELIVERED | `spec_driven_adapter.rs::build_http_client_with_timeout` (`Duration::from_secs(30)`) | Slow endpoint → E-SENSOR-002 timeout |
| Sensor error/health fidelity (F9/F10) | DELIVERED PR #237 | `prism-mcp health::connectivity`, `health::check_one`; `defect_adapter_tls_xdome_live_001.rs` | 403→Up/auth-invalid; 5xx→Degraded; Down only if no TCP |
| MCP `prism_describe` (Claroty tables/columns) | DELIVERED | `prism-mcp/src/tools/prism_describe.rs` | Tool returns schema catalog for `claroty_*` |
| MCP `prism_query` (PrismQL over Claroty) | DELIVERED | `prism-query` engine; `prism-mcp/src/server.rs` | SELECT/WHERE/pipe/aggregate over `claroty_*` |
| MCP `check_sensor_health` (Claroty) | DELIVERED | `prism-mcp/src/server.rs::check_sensor_health`; `health/mod.rs::check_one` | Wire shape reachable/auth_valid/error |
| DTU clone (prism-dtu-claroty) | DELIVERED | `prism-dtu-claroty/src/{clone,routes/*,types,generator}.rs` | DTU exercises routes; NOT the real tenant |
| OCSF field-name routing (Arrow = OCSF names) | **NOT MERGED (in delivery)** | `S-ADR058-OCSF-ROUTING-001` (draft); `column_mapping.rs::ocsf_field_to_arrow_name` (to be added) | Post-merge only |
| DTU-parity migration (parity tests un-ignored) | **PARKED post-v1** | `S-ADR058-DTU-PARITY-MIGRATION-001` (blocked on ROUTING-001) | Deferred |
| `vulnerabilities` + `tags` tables | DTU-only, NOT surfaced in TOML | `prism-dtu-claroty/src/routes/{vulnerabilities,tags}.rs` | Not queryable via PrismQL |
| OcsfEvent/protobuf output path | UNWIRED (test-only) | `column_mapping.rs::ColumnMapper::map_record` (zero prod callers, D-924) | N/A for v1 Arrow query surface |

---

## 1. ONBOARDING — Adding a Client + Their xDome Sensor

### 1.1 The Claroty sensor TOML spec
`crates/prism-sensors/specs/claroty.sensor.toml` — bundled production spec.

- `sensor_id = "claroty"`, `name = "Claroty xDome"`, `version = "1.0.0"`
- `auth_type = "bearer_static"` — **D-747 LOCKED** value (legacy `cookie_roundtrip` label bug deleted by PLUGIN-MIGRATION-001-A)
- `base_url = "${env.CLAROTY_INSTANCE_URL}"` — resolved from env at boot
- `probe_table = "devices"` — routed to by the LIMIT-0 health probe (BC-2.08.001 postcond 5 / S-5.04 AC-10)
- `[[credential_refs]] name = "bearer_token"` — single bearer credential reference

**Four tables surfaced** (DTU has more; only these 4 are in the production TOML):

| Table | `ocsf_class` [CURRENT] | HTTP | path_template | response_path | pagination |
|---|---|---|---|---|---|
| `alerts` | `detection_finding` | POST | `/api/v1/alerts/` | `$.alerts` | offset_limit, page_size 1000 |
| `audit_logs` | `audit_activity` | POST | `/api/v1/audit_log/get` (NO trailing slash) | `$.audit_log` | offset_limit, page_size 1000 |
| `devices` | `device` | POST | `/api/v1/devices/` | `$.devices` | offset_limit, page_size 1000 |
| `device_alert_relations` | `detection_finding` | POST | `/api/v1/device_alert_relations/` | `$.devices_alerts` (NOT `device_alert_relations`) | offset_limit, page_size 1000 |

All are **POST-for-read**: body carries a REQUIRED `fields` projection (minItems ≥ 1 per the xDome OpenAPI parameter schemas). Offset/limit are injected into the **POST body** (`{"offset": N, "limit": 1000}`), NOT URL params, by `pipeline.rs::build_request` (BC-2.16.002 "OffsetLimit Pagination Dispatch: POST-body vs GET-URL", DRIFT-D850-001).

Column inventory (name → `ocsf_field` [dormant on current develop], type):
- **alerts:** `id→finding.uid`(str, REQUIRED), `alert_type_name→type_name`, `category→class_name`, `status→status`, `detected_time→time`(datetime), `updated_time→end_time`(datetime), `devices_count→count`(int), `description→message`, `alert_class`(Tier-2), `ot_devices_count`(Tier-2,int), `alert_name→finding.title`. Polymorphic `id` (int|UUID) normalized to string at parser boundary (EC-016-013-004).
- **audit_logs:** `id→activity_uid`(REQUIRED), `action→activity_name`, `user_display_name→actor.user.name`, `category→category_name`, `timestamp→time`(datetime, **INDEX**), `details→message`, `username→actor.user.uid`, `note→comment`. Real-API fields per LIVE-DRIFT-003 (actor/resource do NOT exist in real xDome audit_log).
- **devices:** 20 columns (PR #236). REQUIRED `uid→device.uid`; `asset_id`, `device_category→device.type`, `device_type→device.type_name`, `risk_score`, `retired→status_code`(bool), plus Tier-2 `purdue_level/site_name/device_subcategory/device_type_family/criticality/is_online/device_name→device.name/manufacturer/model/os_category→device.os.name`. Array columns `ip_list/mac_list/network_list/vlan_list` use ENRICH-1 `source_path = "$.X[*]"` → compact JSON-list string.
- **device_alert_relations (Tier 3):** 10 columns; `device_uid→device.uid`(REQUIRED), `alert_id→finding.uid`, `device_alert_detected_time→time`(datetime), `device_risk_score`, `network_signature_severity`, `network_signature_confidence`, `malicious_ip_severity`, `alert_note→comment`, `external_ip`, `device_alert_status→status`.

**`timestamp_formats`:** none of the Claroty tables declare an explicit chain. All datetime columns resolve via the implicit `["iso8601"]` default from `effective_formats` (ADR-028 §D8-B backward-compat). This is a valid config, not a defect (SAP-2 rule 2 arm (c)).

### 1.2 Credential reference model (AD-017 opaque)
`crates/prism-credentials/src/resolution.rs` — 3-tier resolution chain (BC-2.06.003):
1. env var, 2. config-supplied ref, 3. OS keyring via `CredentialStoreOrgId::get_by_org` (Tier 3, ADR-034, active only when both `org_id` + `keyring` present). Keyring unavailable → hard `E-CRED-008` (not silent fallthrough). Credential values never transit AI context (AD-017; `crates/prism-bin/src/credential_cli.rs` is the operator entry point).

### 1.3 Boot wiring
`crates/prism-bin/src/boot.rs` — bundled specs loaded for `["crowdstrike", "claroty", "cyberint", "armis"]` (`prism-spec-engine/src/add_sensor_spec.rs`). Adapter registered per-org into `AdapterRegistry`; hot-reload deregisters on empty snapshot (BC-2.16.007). Boot exit codes: `exit(4)` audit-init, `exit(5)` credential-init (ADR-022 §A).

---

## 2. FIELD MAPPING — OCSF Normalization State per Table

### 2.1 [CURRENT develop] — `col.name` naming, dormant OCSF annotations
- `SensorSpec` has **no `ocsf_column_naming`** field. Arrow schema field names = `col.name` unconditionally (`prism_describe.rs` maps `ColumnDescriptor.name = col.name`, `description = col.ocsf_field`).
- `class_uid` resolved by `prism-ocsf/src/class_selector.rs::select_by_class_name(ocsf_class)`:
  - `alerts` `detection_finding` → **2004**
  - `audit_logs` `audit_activity` → **3001** (`CLASS_UID_ACCOUNT_CHANGE`)
  - `devices` `device` → **5001** (`CLASS_UID_DEVICE_INVENTORY_INFO`)
  - `device_alert_relations` `detection_finding` → **2004**
- No `raw_extensions` synthesis. Tier-2 columns are ordinary named Arrow columns (by `col.name`).

### 2.2 [POST-ROUTING-001 target] — OCSF underscore-flattened names + Tier model
- `SensorSpec.ocsf_column_naming = true` for Claroty only (CrowdStrike/Armis/Cyberint stay `false`).
- **Tier-1** (`ocsf_field == Some`): Arrow name = `ocsf_field_to_arrow_name(ocsf_field)` (dots → underscores; e.g. `finding_info.uid` → `finding_info_uid`), description = dotted `ocsf_field`. New home: `prism-spec-engine::column_mapping::ocsf_field_to_arrow_name` (NOT prism-bin — dep-graph Rule 2).
- **Tier-2** (`ocsf_field == None`): NO individual ColumnDescriptor; collapse into exactly ONE `raw_extensions` JSON column (`col_type = Json`, `nullable = true`), multi-valued arrays serialized as **compact JSON-list strings** (EC-016-013-028).
- **Fail-closed guards:** duplicate flattened names (RG-009); flag-transition shadow where flattened name == a *different* column's `col.name` A≠B (RG-010/§J2); flattened name == reserved synth name (`class_uid`/`category_uid`/`_sensor`/`raw_extensions`) → `Err(ArrowError::SchemaError)` (RG-027/§J2).
- **class_uid changes** (`class_selector.rs` new arms): `audit_logs` `audit_activity`→`entity_management` = **3004** (new `CLASS_UID_ENTITY_MANAGEMENT`); `devices` `device`→`inventory_info` = **5001** (arm added; regression guard RG-017 — without the arm `.unwrap_or(0)` silently regresses to BASE_EVENT 0); Armis+Claroty `audit_log` `select()` arms 3001→3004 (RG-012/RG-023). `"audit_activity"` arm becomes dead code.
- **KF-01..KF-12 TOML corrections** land in the same story (e.g. KF-03 `alerts.id`→`finding_info.uid`, KF-05 `audit_logs.id` ocsf_field REMOVED→raw_extensions, KF-06 `devices.device_type`→`device.type_label`, KF-08/09/10 remove ocsf_field from `alerts.category/alert_type_name/devices_count`, KF-11 remove from `audit_logs.category`). `ocsf_field` count 31 pre-correction → 26 post-correction across four tables (§J4).
- **Query-breaking:** post-merge, live PrismQL must use OCSF Arrow names (`SELECT finding_info_uid FROM claroty_alerts`), NOT `SELECT id`.

### 2.3 Tier-2 `raw_extensions` handling
Current develop: not synthesized. Post-ROUTING-001: single JSON blob per row keyed by source `col.name`, arrays as compact JSON-list strings; synthesis locus is `pipeline_result_to_record_batch` in `prism-bin/src/spec_driven_adapter.rs` (gains `sensor_spec: &SensorSpec` param, RG-024).

---

## 3. QUERY SUPPORT — PrismQL Shapes Against Claroty Tables

PrismQL (Chumsky parser + DataFusion engine, `prism-query`) over the four `claroty_*` sources. Table names are **sensor-prefixed** in `prism_describe` (`claroty_alerts`, `claroty_audit_logs`, `claroty_devices`, `claroty_device_alert_relations`) so agents build `FROM claroty_alerts | ...`; a bare `FROM alerts` routes to `E-SENSOR-030` (BC-2.10.012 AUDIT-001).

**Working shapes** (engine-level, sensor-agnostic — all execute over materialized Claroty RecordBatches):
- `SELECT` (column projection), `WHERE` predicates (equality + range), pipe operators, aggregations / GROUP BY, `LIMIT`, `ORDER BY`, `enrich` (HTTP-lookup + ENRICH-1 array wildcard).
- Column names must match the active naming regime (§2): `col.name` on current develop; OCSF-flattened post-ROUTING-001.
- Multi-page fetch (>1000 rows) fully supported via offset_limit POST-body pagination (Gap-CL-004 closed).

**Partial / caveats:**
- Most predicates are evaluated **in-engine** (DataFusion post-filter), NOT pushed to xDome (see §4) — large tables can be expensive.
- `alerts` MITRE array columns (`mitre_technique_*`) are NOT surfaced (intentional exclusion; deferred to COERCION-001 MITRE scope).
- `vulnerabilities` and `tags` tables exist in the DTU but are NOT in the production TOML → NOT queryable via PrismQL in v1.
- Write-path sensor operations are feature-flagged (project memory: writes locked behind feature flags); `E-SENSOR-070` = write-not-implemented for adapters without a write path.

**Verify live:** issue each shape against `claroty_devices` / `claroty_alerts`; confirm row counts, projection, and that `WHERE` produces expected subsets.

---

## 4. PUSH-DOWN — Predicate/Time-box Push-down per Table

Mechanism: `prism-query/src/pushdown.rs::extract_time_window_from_ast` (ADR-033 Option T1) walks `Compare` nodes (`Gt/Ge/Lt/Le`) whose LHS is a **datetime + INDEX** column and RHS a Timestamp literal, returning `(start_time, end_time)` (first-wins). Inverted window (start>end) → `push_down.inverted_time_range` WARN, both bounds still returned (DataFusion post-filter backstop).

| Table | INDEX datetime col | Push-down state | Where handled |
|---|---|---|---|
| `audit_logs` | `timestamp` (`options=["INDEX"]`) | **PUSHED DOWN** — time-window → xDome `filter_by` POST body. MERGED PR #239 (S-CLAROTY-AUDITLOG-TIMEBOX-001). | `spec_driven_adapter.rs::build_claroty_audit_filter_by` + `pipeline.rs` JSON auto-parse of `_claroty_audit_filter_by` |
| `alerts` | none declared INDEX | **NO push-down** — all predicates in-engine | DataFusion post-filter |
| `devices` | none declared INDEX | **NO push-down** — all predicates in-engine | DataFusion post-filter |
| `device_alert_relations` | none declared INDEX | **NO push-down** — all predicates in-engine | DataFusion post-filter |

### 4.1 `audit_logs` time-box push-down (delivered detail)
`build_claroty_audit_filter_by(start, end)` → `serde_json::Value` filter object, 4 cases (BC-2.01.013 EC-01-030..033):
- **No bounds (EC-01-030):** single `greater_or_equal` at `now − 7 days` (**bounded default, never unbounded** — avoids E-QUERY-004 timeout).
- **start only (EC-01-031):** single `greater_or_equal` at explicit start (not capped).
- **end only (EC-01-032):** single `less_or_equal` at end; NO synthetic lower bound (avoids inverted window).
- **both (EC-01-033):** compound `{"operation":"and","operands":[gte,lte]}` — key MUST be `operands` (NOT `conditions`).

Filter field name = `"timestamp"`, operations `greater_or_equal`/`less_or_equal` — **research-validated, gated on live confirmation** (ASM-CLAROTY-AUDITLOG-001). DTU ground truth: `ApiQueryFilter = HashMap<String, serde_json::Value>`, `filter_by: Option<ApiQueryFilter>` on `GetAuditLogBody` (`prism-dtu-claroty/src/types.rs`). Guard is `sensor_id == "claroty"`; `_claroty_audit_filter_by` is inert for tables whose body_template doesn't reference it (only `audit_logs` does).

**INDEX eligibility:** push-down fires ONLY because `audit_logs.timestamp` carries `options = ["INDEX"]` (EC-01-034 / AC-INDEX-CLARO-001 / RG-007). Removing INDEX disables push-down (falls back to full-scan + 7-day-less in-engine).

**Verify live:** (a) query with no time filter → confirm POST body `filter_by` = `gte(now-7d)` and results bounded; (b) `WHERE timestamp > 'T'` → `gte(T)`; (c) `WHERE timestamp < 'T'` → `lte(T)`, no synthetic lower bound; (d) both → compound `operands`. Confirm the real xDome accepts field `timestamp` + those operation names.

---

## 5. MCP SURFACE — Analyst-Facing Tools/Resources against Claroty

- **`prism_describe`** (`prism-mcp/src/tools/prism_describe.rs`): returns `TableDescriptor` per `claroty_*` table with sensor-prefixed name, `sensor_type="claroty"`, `description=table.ocsf_class`, and `ColumnDescriptor[]` (`name`, `col_type` = `prism_core::column::ColumnType`, `description`, `nullable`). Also emits `example_query` + `example_note` (OCSF casing hint). [CURRENT] column `name = col.name`, `description = ocsf_field`. [POST-ROUTING-001] Tier-1 `name = ocsf_field_to_arrow_name`, Tier-2 collapse into one `raw_extensions` descriptor.
- **`prism_query`** (`prism-mcp/src/server.rs`): PrismQL execution surface (§3).
- **`check_sensor_health`** (`prism-mcp/src/server.rs::check_sensor_health` → `health/mod.rs::check_one` → `health::connectivity::probe_connectivity`): LIMIT-0 probe routed to `probe_table = "devices"`. Wire shape: `reachable`, `auth_valid`, `error`, `rate_limit`, envelope `summary_counts` + `overall_status`. Health fidelity semantics in §6.
- **Capability discovery / resources** (`prism-mcp/src/resources.rs`, `prompts.rs`): sensor resources with credential redaction (`proofs/sensor_resource_redaction.rs`); Claroty exposed as an available sensor. Reference prompts (`mcp_reference_prompts.rs`) guide agents on time-column names (`claroty_audit_logs→timestamp`, `claroty_alerts→detected_time`).

**Verify live:** `prism_describe claroty_*` returns 4 tables with correct columns for the shipped naming regime; `check_sensor_health` for claroty returns correct reachable/auth_valid against the real tenant.

---

## 6. LIVE-API FIDELITY — Known-Good vs Known-Gap vs Real xDome

### 6.1 Known-good (validated against DTU clone + wiremock, NOT the live tenant)
- **TLS (ADR-050):** production `reqwest` client is `rustls-tls`, `default-features=false`, `http2` feature active (PR #241 / RG-008 asserts `h2` in reqwest's Cargo.lock block), 30s timeout (`build_http_client_with_timeout`). native-tls forbidden (avoids ~65s macOS Keychain stall + MITM path).
- **Error/health fidelity (PR #237, F9/F10)** — `defect_adapter_tls_xdome_live_001.rs`, all wiremock (NO `#[ignore]`):
  - 403 → `ConnectivityStatus::Up`, `http_status=Some(403)`, `reachable=true`, `auth_valid=false` (RG-005/RG-007).
  - 401 → `Up`, `reachable=true`, `auth_valid=false` (LOW-1; CookieRoundtrip arm RG-015/F-P31).
  - 5xx (503) → `Degraded`, `reachable=true`, `auth_valid=true`, `error="service_unavailable"` (RG-014/RG-019); Degraded NOT counted as healthy in envelope (RG-020).
  - `Down` only when no TCP/HTTP exchange. `check_one`: `reachable = connectivity != Down` (HS-007 fix).
  - Error taxonomy: `map_spec_engine_error_to_sensor_error` → `SensorError::HttpError{status}` (E-SENSOR-001) for 4xx/5xx; CookieAuthFailed 401 → HttpError.
- **Trailing slash:** production TOML uses trailing-slash paths (`/api/v1/alerts/` etc.) matching real xDome (poller-bear reference); DTU wraps `NormalizePathLayer::trim_trailing_slash()` (STRIP-ONLY) so both forms work (Gap-CL-001 closed). `audit_log/get` deliberately has NO trailing slash (OpenAPI declares it so).
- **Pagination:** offset/limit in POST body verified (Gap-CL-004).

### 6.2 Known-gap / DTU-vs-live divergences
- **All fidelity is DTU/wiremock — never the real tenant.** No test hits the real xDome. LIVE-DRIFT-003 already corrected audit_log field assumptions (actor/resource don't exist; real fields are `user_display_name`/`username`/`note`). Tier-2 device columns verified only against **xDome OpenAPI 2026-06-20**, not live responses.
- **ASM-CLAROTY-AUDITLOG-001 unconfirmed live:** `filter_by` field=`timestamp` + ops `greater_or_equal`/`less_or_equal` are research-validated, explicitly gated on a one-line live check at demo prep. `ClarotyAuditLogFilter` is a phantom type — DTU ground truth is `ApiQueryFilter`.

### 6.3 `#[ignore]`'d live/e2e tests on the xDome path
**Directly Claroty-named (4):**
1. `prism-dtu-claroty/src/routes/audit_log.rs:602` — `test_..._pipeline_integration_ac_006`: requires prism-bin full-boot wiring; ungated after S-DEMO-002 (`todo!` body).
2. `prism-spec-engine/tests/parity/claroty.rs:117` — `test_BC_2_16_013_dtu_parity_claroty`: requires prism-dtu-claroty DTU clone + recorded reference OCSF fixtures (S-6.08 / DTU-EXT-001..004; tracking PLUGIN-MIGRATION-Wave-2).
3. `prism-spec-engine/tests/parity/claroty.rs:188` — second parity variant, same gate.
4. `prism-spec-engine/src/pipeline.rs:4541` — `test_BC_2_16_002_pagination_claroty_alerts_page_2_returns_data`: requires DTU clone with 102-entry alerts fixture; ungated after S-DEMO-CLAROTY-PAGINATION-001 fixture recorded.

**Cross-cutting E2E suites that include Claroty flows (not Claroty-exclusive):**
- `prism-bin/tests/e2e_smoke.rs` — 13 `#[ignore]` attrs (E2E-001: requires DTU server + prism binary; ungated via `e2e` nextest profile); references claroty.
- `prism-bin/tests/e2e_multi_org.rs` — 10 `#[ignore]` attrs (E2E-MULTI-001: requires multi-org DTU; ungated via `e2e-multi-org` profile); references claroty.

**Count:** 4 directly-Claroty live/e2e ignored tests; +23 cross-cutting E2E ignored tests (13 + 10) that exercise the Claroty path among multiple sensors. All are gated on DTU-clone / full-boot availability, none on the real tenant.

---

## 7. KNOWN GAPS / RISKS on the xDome Path

- **ROUTING-001 not merged (in delivery).** Arrow column naming flips from `col.name` to OCSF-flattened on merge; all live queries and `prism_describe` output change. `class_uid` for audit_logs (3001→3004) and devices (regression-guarded 5001) change. This is the single biggest v1-shape decision.
- **DTU-parity migration PARKED post-v1.** `S-ADR058-DTU-PARITY-MIGRATION-001` is blocked on ROUTING-001; `parity/claroty.rs` tests stay `#[ignore]`'d (reference OCSF fixtures not recorded).
- **OcsfEvent/protobuf path UNWIRED.** `ColumnMapper::map_record` has ZERO production callers (D-924 / STATE.md D-2101). Only the Arrow RecordBatch query surface is live; native OCSF-protobuf output is a future story (no story exists yet).
- **ASM-CLAROTY-AUDITLOG-001 unconfirmed against live** (see §6.2) — audit_logs push-down could silently return wrong/empty windows if the real field name or operation strings differ.
- **No push-down on alerts / devices / device_alert_relations** — large tenants → full-scan fan-out with only offset pagination (page_size 1000) + in-engine filter; risk of E-QUERY-004 timeout and 200MB per-query memory pressure.
- **`vulnerabilities` + `tags` tables not surfaced in TOML** — DTU routes exist (`routes/vulnerabilities.rs`, `routes/tags.rs`) but are not queryable in v1.
- **`alerts` MITRE arrays excluded** — deferred to COERCION-001 MITRE-column scope.
- **audit_log.rs AC-006 pipeline integration `todo!`** — deferred to S-DEMO-002 full-boot wiring.
- **Tier-2 device columns provenance is OpenAPI-doc only** (2026-06-20), not live-observed.

---

## 8. DRAFT Validation-Checklist Skeleton (for the Live-Test Matrix)

### A. Onboarding & Boot
- [ ] A1. Set `CLAROTY_INSTANCE_URL` + store `bearer_token` via credential CLI; confirm token never appears in AI context / logs (AD-017).
- [ ] A2. Boot resolves bundled `claroty.sensor.toml`; adapter registered per-org; `prism_describe` lists all 4 `claroty_*` tables.
- [ ] A3. Credential Tier resolution: env → config → keyring; keyring-unavailable → `E-CRED-008` (not silent).
- [ ] A4. Invalid/expired token → health `auth_valid=false` (not a boot crash).

### B. Schema / Field Mapping (run against the SHIPPED naming regime)
- [ ] B1. `prism_describe claroty_alerts/audit_logs/devices/device_alert_relations` → column names match regime ([CURRENT] `col.name`; [POST] OCSF-flattened + one `raw_extensions`).
- [ ] B2. `class_uid` on materialized rows: alerts=2004; audit_logs=[CURRENT 3001 / POST 3004]; devices=5001; device_alert_relations=2004.
- [ ] B3. [POST] Tier-2 columns absent as first-class Arrow fields; present inside `raw_extensions` JSON (arrays as compact JSON-list strings).
- [ ] B4. Polymorphic `alerts.id` (int vs UUID from live) → normalized to string.
- [ ] B5. Datetime columns parse via implicit iso8601 default against real xDome timestamp strings.

### C. Query Support
- [ ] C1. `SELECT` projection over each table returns expected columns.
- [ ] C2. `WHERE` equality + range filters over each table.
- [ ] C3. Pipe operators, GROUP BY / aggregations, `ORDER BY`, `LIMIT`.
- [ ] C4. `enrich` including ENRICH-1 array-wildcard columns (`ip_list` etc.).
- [ ] C5. `>1000` rows → multi-page offset_limit POST-body pagination fetches all pages.
- [ ] C6. Bare `FROM alerts` (no sensor prefix) → `E-SENSOR-030` (not silent empty).

### D. Push-down (audit_logs)
- [ ] D1. No time filter → POST body `filter_by = gte(now-7d)`; results bounded; no E-QUERY-004.
- [ ] D2. `WHERE timestamp > 'T'` → `gte(T)`.
- [ ] D3. `WHERE timestamp < 'T'` → `lte(T)`, no synthetic lower bound.
- [ ] D4. Both bounds → compound `{"operation":"and","operands":[gte,lte]}` (key `operands`).
- [ ] D5. Confirm real xDome accepts field `timestamp` + ops `greater_or_equal`/`less_or_equal` (**closes ASM-CLAROTY-AUDITLOG-001**).
- [ ] D6. alerts/devices/device_alert_relations filters evaluated in-engine, results correct (no push-down expected).

### E. MCP Surface
- [ ] E1. `prism_describe` example_query for each `claroty_*` table is executable verbatim.
- [ ] E2. `check_sensor_health` claroty against live: healthy 200 → reachable/auth_valid true.
- [ ] E3. Health wire fidelity: 403→reachable/auth_invalid; 401→auth_invalid; 5xx→Degraded reachable=true error=service_unavailable; Down only if unreachable.
- [ ] E4. Health probe routes to `probe_table=devices` with LIMIT-0.

### F. Transport / TLS / Error Fidelity (live)
- [ ] F1. HTTPS to real tenant via rustls (no native-tls Keychain stall); http2 negotiated.
- [ ] F2. 30s timeout → slow endpoint surfaces `E-SENSOR-002`.
- [ ] F3. Trailing-slash + non-slash paths both accepted by live xDome (`/api/v1/alerts/`, `/api/v1/audit_log/get`).
- [ ] F4. 4xx/5xx → structured `E-SENSOR-001`/`SensorError::HttpError` (no panic, no silent empty Vec).

### G. Regression / Data-loss Guards
- [ ] G1. [POST] devices `class_uid` stays 5001 (RG-017 guard — not silently 0).
- [ ] G2. [POST] no duplicate flattened Arrow names / no reserved-name collision → fail-closed SchemaError, not silent shadow-loss.
- [ ] G3. Partial fan-out failure propagates (no swallowed `Vec::new()`).
- [ ] G4. Live audit_log field drift vs LIVE-DRIFT-003 assumptions (user_display_name/username/note present).

---

## State Checkpoint
```yaml
artifact: xdome-v1-validation/feature-inventory.md
develop_head: 362e4f85
routing_001_status: draft (in delivery, NOT merged)
tables_surfaced: 4 (alerts, audit_logs, devices, device_alert_relations)
ignored_live_tests_claroty_direct: 4
ignored_e2e_crosscutting_including_claroty: 23
committed: false
timestamp: 2026-08-21
```
