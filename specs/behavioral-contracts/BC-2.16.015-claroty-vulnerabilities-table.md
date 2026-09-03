---
document_type: behavioral-contract
level: L3
version: "2.0"
status: active
producer: product-owner
timestamp: 2026-08-24T00:00:00Z
phase: 3
origin: brownfield
subsystem: "SS-16"
capability: "CAP-029"
lifecycle_status: active
inputs:
  - ".factory/objectives/xdome-endpoint-expansion-plan.md"
  - ".factory/objectives/xdome-v1-validation/endpoint-spike-findings.md"
  - ".factory/specs/domain-spec/capabilities.md"
  - ".factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md"
  - "crates/prism-sensors/specs/claroty.sensor.toml"
input-hash: "2e91a4e"
# input-hash: updated 2026-08-31 — claroty.sensor.toml modified by S-CLAROTY-VULNS-001 (vulnerabilities [[tables]] block added; prior c913a02 stale); lifecycle_status promoted draft→active per POL-14 (PR #245 squash-merged D-2387).
traces_to: ["CAP-029"]
extracted_from: ".factory/objectives/xdome-v1-validation/endpoint-spike-findings.md"
introduced: "2026-08-24"
modified: "2026-09-02"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.16.015: Claroty xDome Vulnerability Findings Table — Queryable Surface and OCSF vulnerability_finding Mapping

## Description

The `claroty_vulnerabilities` TOML table block in `claroty.sensor.toml` exposes Claroty xDome
vulnerability findings as a queryable PrismQL table. The table follows the standard Claroty
POST-for-read pattern with offset/limit pagination, using `vulnerability_finding` (class_uid 2002)
as its OCSF class. Under `ocsf_column_naming = true`, the primary key field `name` is a Tier-1
column (exposed as `finding_info_title`), `description` maps to `message`, and all remaining
17 columns are Tier-2 aggregated into `raw_extensions`. An opaque Claroty-internal `id`
(CJYASHKR-format) is captured via `source_path = "$.id"` as a secondary Tier-2 identifier
to enable joins against `claroty_device_vulnerability_relations`.

## Preconditions

- `claroty.sensor.toml` includes the `claroty_vulnerabilities` [[tables]] block as specified
  in S-CLAROTY-VULNS-001
- `ocsf_column_naming = true` is declared at the sensor level in `claroty.sensor.toml`
- The `vulnerability_finding` / class_uid 2002 arm exists in
  `prism-ocsf/src/class_selector.rs::select_by_class_name` (existing arm — no new arm
  required per spike findings §Overall Verdict)
- The Claroty bearer token credential is configured for the requesting client
- S-PLUGIN-PREREQ-A through S-PLUGIN-PREREQ-E have all merged (spec-engine pipeline active)

## Postconditions

### 1. TOML Table Contract

The `claroty_vulnerabilities` table MUST be declared in `claroty.sensor.toml` with:

```toml
[[tables]]
table_name = "vulnerabilities"
ocsf_class = "vulnerability_finding"   # class_uid 2002 (existing arm)
```

> **Registration note:** `table_name = "vulnerabilities"` registers in DataFusion as
> `claroty_vulnerabilities` (`{sensor_id}_{table_name}` per `table_registry.rs`), consistent
> with the sibling convention (`claroty_alerts`, `claroty_audit_logs`, `claroty_devices`,
> `claroty_device_alert_relations`). PrismQL queries use the registered flat name
> `claroty_vulnerabilities`.

**Step definition:**

```text
[[tables.steps]]
name = "fetch_vulnerabilities"
method = "POST"
path_template = "/api/v1/vulnerabilities/"
body_template = '{"fields": ["name", "vulnerability_type", "cve_ids", "cvss_v3_score", \
  "cvss_v3_exploitability_subscore", "cvss_v3_vector_string", "cvss_v2_score", \
  "description", "is_known_exploited", "affected_devices_count", \
  "affected_ot_devices_count", "published_date", "epss_score", \
  "adjusted_vulnerability_score", "adjusted_vulnerability_score_level", \
  "exploits_count", "source_name", "source_url"], \
  "sort_by": [{"field":"adjusted_vulnerability_score","order":"desc"},{"field":"name","order":"asc"}]}'
response_path = "$.vulnerabilities"
variables_produced = []
[tables.steps.pagination]
type = "offset_limit"
page_size = 1000
```
_(Illustrative only — not copy-paste-ready TOML. Normative source: `crates/prism-sensors/specs/claroty.sensor.toml`. TOML single-quoted literals do not support backslash line-continuation; the shipped spec uses a single-line `body_template` value.)_

**Pagination note:** The `vulnerabilities` envelope also carries a `count` field which is
nullable per the OpenAPI schema. The spec-engine handles nullable `count` via the empty-page
check — pagination halts when the returned page is empty, regardless of whether `count` is
present (EC-016-015-003).

**Sort-by postcondition (DEFECT-CLAROTY-SORTBY-DETERMINISM-001):** The `body_template` MUST
include a `sort_by` key with the value
`[{"field":"adjusted_vulnerability_score","order":"desc"},{"field":"name","order":"asc"}]`
(per `DEFECT-CLAROTY-SORTBY-DETERMINISM-001` RG-001 `test_rg_vulnerabilities_sort_by_in_request_body`).
Both `adjusted_vulnerability_score` and `name` are confirmed members of
`Vulnerability__sortable_fields_enum` (xDome OpenAPI schema `ValidatingSortClause__6`).
**DI-019 truncation rationale:** when the full vulnerability set exceeds 10,000 records, the
pipeline halts at the 10K cap; `adjusted_vulnerability_score desc` as the primary sort ensures
that the highest-risk vulnerabilities are the records that survive truncation rather than an
arbitrary API-ordered subset. The `name asc` tiebreaker (CVE-ID or advisory title, provably
unique per OCSF `finding_info.title` semantics) eliminates duplicate-or-skipped records across
offset page boundaries (EC-016-015-009; `DEFECT-CLAROTY-SORTBY-DETERMINISM-001` RG-008
`test_rg_vulnerabilities_sort_by_tiebreaker_is_unique_field`). ORDER BY push-down to the API
request body is the sole mechanism — the spec-engine does NOT inject `sort_by` automatically;
it MUST appear in `body_template`. Push-down of user PrismQL ORDER BY to the API is deferred
to `TD-SENSOR-SORTBY-PUSHDOWN-001`.

### 2. Column Tier Classification (ADR-058)

Under `ocsf_column_naming = true`, columns are classified as follows:

**Tier-1 columns** (have `ocsf_field`; exposed as Arrow field name =
`ocsf_field_to_arrow_name(ocsf_field)`):

| Column (TOML name) | ColumnType | ocsf_field | Arrow Field Name | Options |
|--------------------|-----------|------------|-----------------|---------|
| `name` | String | `finding_info.title` | `finding_info_title` | REQUIRED |
| `description` | String | `message` | `message` | — |

**Tier-2 columns** (no `ocsf_field`; values aggregate into `raw_extensions` JSON object):

| Column (TOML name) | ColumnType | Source | Notes |
|--------------------|-----------|--------|-------|
| `vulnerability_type` | String | fields projection | "Platform", "Clinical", "Configuration", etc. |
| `cve_ids` | Json | fields projection | Array of CVE IDs; NATIVE JSON array on the wire — empty: `[]`, populated: `["CVE-2024-1234", ...]`; NOT a JSON-encoded string (NOT `"[]"`) |
| `cvss_v3_score` | Float | fields projection | Primary CVSS v3 base score |
| `cvss_v3_exploitability_subscore` | Float | fields projection | |
| `cvss_v3_vector_string` | String | fields projection | |
| `cvss_v2_score` | Float | fields projection | CVSS v2 fallback |
| `is_known_exploited` | Boolean | fields projection | CISA KEV indicator |
| `affected_devices_count` | Integer | fields projection | Total devices affected |
| `affected_ot_devices_count` | Integer | fields projection | OT-specific device count |
| `published_date` | Datetime | fields projection | ISO 8601; ADR-028 §D8-B implicit iso8601 default |
| `epss_score` | Float | fields projection | EPSS exploit probability |
| `adjusted_vulnerability_score` | Float | fields projection | Claroty composite risk score |
| `adjusted_vulnerability_score_level` | String | fields projection | "High" / "Medium" / "Low" |
| `exploits_count` | Integer | fields projection | Known exploit count |
| `source_name` | String | fields projection | NVD, ICS-CERT, etc. |
| `source_url` | String | fields projection | Advisory URL |
| `id` | String | `source_path = "$.id"` | Opaque Claroty internal ID (CJYASHKR format); NOT in fields projection; optional secondary identifier for device_vulnerability_relations joins |

**Total declared columns:** 19 (2 Tier-1, 17 Tier-2).

### 3. Primary Key and OCSF Mapping Rationale

`name` is the canonical primary key (not `id`) per spike findings §Spike 1:

1. `name` is in the Vulnerability fields_enum (requestable via `fields` projection); `id` is not.
2. `name` carries the industry-standard identifier: `"CVE-2021-31998"` for CVE-based
   vulnerabilities, `"ICSMA-21-161-01 (ZOLL Defibrillator Dashboard)"` for advisories.
3. `cve_ids` is Json (array) — cannot serve as a scalar PK.
4. Cross-table joins via `vulnerability_name` in `device_vulnerability_relations` do not
   require the opaque `id`.
5. OCSF vulnerability_finding: `name` maps to `finding_info.title` — the finding's title/identifier.

The opaque `id` (CJYASHKR format) is captured as a secondary identifier via
`source_path = "$.id"` — it appears outside the fields projection in the raw API response.
It is NOT marked REQUIRED: its presence on the live monroe sensor must be confirmed before
treating it as guaranteed. Until confirmed, it remains optional — absent from `raw_extensions`
when the API omits the key, or stored as JSON `null` only if the API sends an EXPLICIT null value.

The `id` column is OPTIONAL. When the key is absent from a row, the pre-existing ENRICH-1
mechanism (`prism-spec-engine` `column_mapping.rs`) currently emits
`column_source_path_extraction_failed` at WARN level (pre-existing sensor-agnostic behavior).
Correcting this to distinguish absent-optional-key (debug) from genuine extraction failure (warn)
is tracked as story `S-ENGINE-SOURCE-PATH-ABSENT-KEY-LOGLEVEL-001` (deferred;
architect-ruled out of this story's engine scope).

### 4. SAP-2 DTU Parity

SAP-2 probe is **applicable** for G1 (DTU exists: `prism-dtu-claroty`). The
`claroty_vulnerabilities` table MUST be registered in the DTU clone route at
`crates/prism-dtu-claroty/src/clone.rs::build_router()` for DTU-grounded parity tests.

> **Deferral — D-2200 / D-2264 (v1 scope):** SAP-2 DTU-parity registration and validation for
> `claroty_vulnerabilities` is **DEFERRED post-v1** per governing decision D-2200 (v1-scope
> confirmed by D-2264); the parity work is anchored to story S-ADR058-DTU-PARITY-MIGRATION-001.
> Until that story executes, S-CLAROTY-VULNS-001 delivers the table WITHOUT DTU-parity validation
> (the `prism-dtu-claroty` DTU clone route exists but serves a legacy stub envelope; alignment
> to `claroty_vulnerabilities` is the parked parity work). The `MUST be registered` requirement
> retains full force and is binding when S-ADR058-DTU-PARITY-MIGRATION-001 executes. SAP-2
> adversary passes MUST NOT mint DTU-parity-registration findings against this table until that
> story merges.

**SAP-2 exclusion documentation:** The `Vulnerability__fields_enum` contains 32 queryable
fields. The contracted subset is 18 fields (plus the non-enum `id` via source_path). The
remaining 14 fields are deliberately excluded from the first-cut column set to keep the initial
implementation focused. SAP-2 future passes MUST NOT mint "field in API with no TOML column"
findings for the 14 excluded fields — the deliberate exclusion is documented here:
`affected_medical_devices_count`, `affected_iot_devices_count`, `affected_it_devices_count`,
`affected_fixed_devices_count`, `affected_confirmed_devices_count`,
`affected_potentially_relevant_devices_count`, `affected_irrelevant_devices_count`,
`vulnerability_labels`, `vulnerability_assignees`, `vulnerability_note`,
`vulnerability_priority_group`, `sources` (array, Json candidate), `cvss_v2_exploitability_subscore`,
`cvss_v2_vector_string`. Any story adding columns from this excluded set MUST amend this BC and
update the exclusion count.

## Invariants

- DI-005: OCSF schema validity — `vulnerability_finding` class_uid 2002 is a valid OCSF class
- `name` carries `ColumnOptions::Required`; REQUIRED marks `name` push-down-eligible in
  `pushdown.rs` — it does NOT enforce presence at ingest; when the API omits `name`, the
  `finding_info_title` Arrow column is null (default nullable behavior); this is NOT a hard error
- Opaque `id` sourced via `source_path = "$.id"` is optional — when the source `id` key is ABSENT it is simply absent from `raw_extensions` (column skipped, no error); it is stored as JSON `null` only if the API sends an EXPLICIT null value; does NOT block pagination

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-SENSOR-001` | Claroty API returns non-200 HTTP for POST /api/v1/vulnerabilities/ | Non-200 propagates via `?` in the fetch loop; the entire fetch returns the structured error (sensor=claroty, status, body) and no partial/accumulated pages are returned (atomic-fail; Option-A fail-fast) |
| `E-QUERY-001` | Query references `vulnerability_type`, `cvss_v3_score` or any other Tier-2 column by its raw TOML name (not `raw_extensions`) | E-QUERY-038 column-not-found at plan time; available_columns includes `raw_extensions`, `finding_info_title`, `message`, `class_uid`, `_sensor` |
| `E-SPEC-018` | Datetime parse failure on `published_date` for a PRESENT non-ISO-8601 value | `normalize_timestamp_fields` runs post-accumulation (after the pagination loop completes); on parse failure it returns `Err(SpecEngineError::TimestampParseFailure)` (attempted_formats + value capped at 50 chars per SEC-002/AD-017); the `?` discards the entire accumulated result — the fetch fails atomically and NO partial pages are returned (Option-A fail-fast) |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-016-015-001 | Row missing `name` field (REQUIRED) | Null row produced; no hard error; subsequent rows continue |
| EC-016-015-002 | Opaque `id` absent from API response envelope | the `id` key is absent from `raw_extensions` (no standalone `id` column is materialized); no error is raised; pagination is unaffected |
| EC-016-015-003 | `count` is null or absent in the response envelope | Pagination continues via empty-page check (empty page → halt); not an error |
| EC-016-015-004 | CVE ID format varies (CVE-YYYY-NNNNN vs advisory title format) | Preserved as-is in `finding_info_title`; no normalization |
| EC-016-015-005 | `cve_ids` field is an empty array `[]` | MUST serialize as the native JSON array `[]` in `raw_extensions` (NOT the string `"[]"`, NOT JSON null, NOT the string `"null"`); a populated `cve_ids` MUST serialize as a native JSON array `["CVE-2024-1234", ...]` — never a stringified representation |
| EC-016-015-006 | `published_date` is null | Null value stored in `raw_extensions` for the Tier-2 `published_date` field (no standalone Datetime column is materialized); ADR-028 §D8-B implicit iso8601 default applies only to non-null present values |
| EC-016-015-007 | `LIMIT 1` early-stop: page_size=1000, ~1.1 MB/page; only 1 record requested | Early-stop fires after the first complete page (1000 raw records fetched and fully parsed); `PipelineResult.truncated=false` (early-stop is NOT a DI-019 capacity overflow — `truncated` is reserved for DI-019 only, per ADR-060 §D8.3); DataFusion trims the 1000-record batch to 1 row; second and subsequent HTTP POST requests are NOT issued (ADR-060 §D8.2 check fires at COMPLETE page boundary, immediately after DI-019 check). Anchor: BC-2.16.002 §Postconditions LIMIT-Aware Early-Stop (ADR-060 §D8). **UNAFFECTED by §D8.7 plan-shape gate** — `SELECT *` with no aggregation/GROUP BY/DISTINCT/WHERE equality has `ast_is_reducing_plan = false`; early-stop fires normally. |
| EC-016-015-008 | `SELECT COUNT(*) FROM claroty_vulnerabilities` or `SELECT COUNT(*) FROM claroty_vulnerabilities LIMIT N` — aggregation suppresses early-stop; full count returned | `ast_is_reducing_plan = true` (Condition A: `FuncCall::Aggregate` COUNT node); `fetch_limit = 0`; `FetchContext::early_stop_limit = None`; early-stop suppressed; pipeline fetches ALL pages through full pagination (up to DI-019 10K cap); COUNT result reflects the true total vulnerability count across the complete dataset, NOT a 1-page partial (~1000 records). Without §D8.7, early-stop would fire after 1 page and COUNT would report ~1000 instead of the true total (F-R11-CRIT-001). EC-016-015-007 and TV-BC-2.16.015-006 (bare `SELECT * LIMIT 1`) are UNAFFECTED — bare projection has `ast_is_reducing_plan = false` → early-stop fires normally. Anchor: BC-2.16.002 §Postconditions LIMIT-Aware Early-Stop Plan-Shape Gate (ADR-060 §D8.7 Condition A); implementing story S-ENGINE-LIMIT-EARLY-STOP-001. |
| EC-016-015-009 | Offset pagination determinism: two sequential pages of `claroty_vulnerabilities` must not duplicate or skip records | The `body_template` MUST contain `"sort_by": [{"field":"adjusted_vulnerability_score","order":"desc"},{"field":"name","order":"asc"}]` — the `name` tiebreaker (provably unique CVE ID / advisory title) makes the sort order total, ensuring deterministic page boundaries under offset pagination. Without a unique tiebreaker, the xDome API default sort (`published_date desc`) is non-unique and causes page-boundary instability on repeated fetches. Anchor: `DEFECT-CLAROTY-SORTBY-DETERMINISM-001` RG-001 (`test_rg_vulnerabilities_sort_by_in_request_body`) + RG-008 (`test_rg_vulnerabilities_sort_by_tiebreaker_is_unique_field`). |

## Related BCs

- BC-2.16.013: Bundled Sensor Spec Authoring — parent spec for the Claroty sensor; this BC adds the `claroty_vulnerabilities` table to the Claroty sensor surface (depends on)
- BC-2.02.005: Claroty xDome Field Mapping to OCSF (9 Data Sources) — OCSF class mapping for all Claroty sources; `vulnerability_finding` class_uid 2002 covered (composes with)
- BC-2.01.007: Claroty Bearer Token Auth — auth mechanism unchanged; preconditions satisfied (depends on)
- BC-2.16.002: Multi-Step Fetch Pipeline Execution — LIMIT-Aware Early-Stop Pagination postcondition (ADR-060 §D8) is the contract anchor for EC-016-015-007 and TV-BC-2.16.015-006 (depends on)

## Architecture Anchors

- `crates/prism-sensors/specs/claroty.sensor.toml` — TOML spec file authoring target
- `crates/prism-spec-engine/src/spec_parser.rs` — ColumnSpec, FetchStep deserialization
- `crates/prism-spec-engine/src/pipeline.rs` — OffsetLimit POST-body injection
- `crates/prism-ocsf/src/class_selector.rs::select_by_class_name` — `"vulnerability_finding"` arm (existing)
- `crates/prism-bin/src/spec_driven_adapter.rs` — `pipeline_result_to_record_batch`
- `.factory/objectives/xdome-v1-validation/endpoint-spike-findings.md §Spike 1` — PK decision authority

## Story Anchor

S-CLAROTY-VULNS-001 (draft — Wave A)

## VP Anchors

(none — no formal verification properties defined; standard structural tests via story RG list)

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.16.015-001 | `SELECT finding_info_title FROM claroty_vulnerabilities LIMIT 5` against live or DTU | Succeeds (no E-QUERY-038); rows have non-null `finding_info_title` for CVE-named vulnerabilities |
| TV-BC-2.16.015-002 | `SELECT * FROM claroty_vulnerabilities LIMIT 1` | Response wire JSON contains `class_uid = 2002`; `finding_info_title` present; `raw_extensions` object present with Tier-2 fields |
| TV-BC-2.16.015-003 | `SELECT vulnerability_type FROM claroty_vulnerabilities LIMIT 1` | E-QUERY-038; `available_columns` contains `finding_info_title`, `message`, `raw_extensions`; does NOT contain `vulnerability_type` |
| TV-BC-2.16.015-004 | `SELECT raw_extensions FROM claroty_vulnerabilities LIMIT 5` | Succeeds; raw_extensions JSON contains `vulnerability_type`, `cvss_v3_score` keys |
| TV-BC-2.16.015-005 | Response envelope with null `count` field | Pagination terminates on empty page; no error |
| TV-BC-2.16.015-006 | `SELECT * FROM claroty_vulnerabilities \| LIMIT 1` — LIMIT early-stop: page_size=1000, ~1.1 MB/page | Exactly 1 HTTP POST request issued (1 complete page fetched); `PipelineResult.truncated=false`; DataFusion trims result to 1 row; elapsed time within per-page budget (ADR-060 §D8 + §Consequences) |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| (none) | No VP directly verifies this BC — structural tests cover via story RG list per S-CLAROTY-VULNS-001 |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-029 |
| Capability Anchor Justification | CAP-029 ("Config-Driven Sensor Adapters") per capabilities.md §CAP-029 — this BC specifies the TOML table contract for the Claroty xDome `claroty_vulnerabilities` table, defining columns (typed with ColumnOptions and OCSF mappings), multi-step fetch pipeline (POST-for-read, offset_limit pagination), response parsing rules, and Tier-1/Tier-2 OCSF column classification. This is exactly what CAP-029 defines: sensor adapters defined in TOML spec files with tables, columns, pipelines, and pagination config. |
| L2 Invariants | DI-005 |
| Priority | P0 |
| Story | S-CLAROTY-VULNS-001 |
| DTU Status | EXISTS — `prism-dtu-claroty`; SAP-2 parity probe applicable; **parity registration DEFERRED post-v1 per D-2200/D-2264 → S-ADR058-DTU-PARITY-MIGRATION-001** |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 2.0 | defect-claroty-sortby-determinism-bc-amendments | 2026-09-02 | product-owner | Human-directed 2026-09-02: add deterministic `sort_by` postcondition to `claroty_vulnerabilities` `body_template` to fix D-001 offset pagination instability (non-unique `published_date desc` API default). (1) §Postconditions §1 illustrative body_template block: `sort_by` array added after the `fields` array: `[{"field":"adjusted_vulnerability_score","order":"desc"},{"field":"name","order":"asc"}]`. Both fields confirmed in `Vulnerability__sortable_fields_enum` (xDome OpenAPI `ValidatingSortClause__6`). DI-019 truncation rationale: with >10K vulns, `adjusted_vulnerability_score desc` primary sort ensures highest-risk records are returned under the cap; `name asc` tiebreaker (CVE ID, provably unique) enforces deterministic page boundaries. (2) **Sort-by postcondition** note added after the existing Pagination note in §Postconditions §1. (3) EC-016-015-009 added: offset pagination determinism guarantee with MUST anchor. TD-VSDD-097: (1) Sibling pair — `claroty_vulnerabilities` has no BC sibling created in the same split; the 6 remaining BCs amended in this same burst (BC-2.16.013, BC-2.16.019, BC-2.16.020, BC-2.16.021) each cover distinct tables with distinct sort criteria; CLEAR. (2) Downstream copy target — `claroty.sensor.toml` `fetch_vulnerabilities` `body_template` is the downstream copy target requiring an identical `sort_by` addition; its update is deferred to implementing story `DEFECT-CLAROTY-SORTBY-DETERMINISM-001` per human-directed task scope boundary. (3) Mandate anchor — EC-016-015-009 and §Sort-by postcondition MUSTs anchored to `DEFECT-CLAROTY-SORTBY-DETERMINISM-001` RG-001 (`test_rg_vulnerabilities_sort_by_in_request_body`) + RG-008 (`test_rg_vulnerabilities_sort_by_tiebreaker_is_unique_field`). |
| 1.9 | claroty-vulns-cve-native-array | 2026-08-30 | product-owner | Cross-story reconciliation (human-approved): `cve_ids` (column_type=json) standardized to NATIVE JSON array on the wire; was ambiguous ("serialized as `[]` JSON"), now explicitly contracts a native array. Aligns with BC-2.16.016 EC-002 (related_alert_ids, also column_type=json, already native-array) and canonical DD-2 engine behavior. Two content sites changed: (1) EC-016-015-005 — now contracts MUST serialize as native JSON array `[]` (NOT string `"[]"`, NOT JSON null, NOT string `"null"`); populated → `["CVE-2024-1234", ...]` native array; (2) Tier-2 column table `cve_ids` Notes — updated to state NATIVE JSON array explicitly. Cross-ref: G2 blast-radius CRIT-1. TD-VSDD-097: (1) Sibling pair — BC-2.16.016 (related_alert_ids, json column_type) is the sibling; already contracts native array; no mirror edit required; CLEAR. (2) Downstream copy target — cve_ids Notes and EC-016-015-005 are not verbatim copy-sources in any downstream artifact; the `claroty.sensor.toml` inline comment ("serialized as a JSON array string") is a downstream target requiring correction — delegated to implementer (VULNS branch) per task scope boundary; CLEAR for this burst. (3) Mandate anchor — EC-016-015-005 MUST is anchored to S-CLAROTY-VULNS-001 (implementing story); no unanchored MUSTs introduced. |
| 1.8 | adr060-d8-7-plan-shape-gate | 2026-08-26 | product-owner | **ADR-060 §D8.7 plan-shape gate — EC-016-015-008 added.** F-R11-CRIT-001 remediation: `SELECT COUNT(*) FROM claroty_vulnerabilities` (and with LIMIT N) suppresses early-stop via `ast_is_reducing_plan = true` (Condition A: aggregate node); pipeline fetches ALL pages; COUNT reflects the true total. EC-016-015-007 and TV-BC-2.16.015-006 (bare `SELECT * LIMIT 1`) are explicitly noted UNAFFECTED — bare projection has `ast_is_reducing_plan = false` → early-stop fires normally. **F-R11-OBS-001 adjudication:** BC-2.16.015 stays draft; S-CLAROTY-VULNS-001 is the primary delivery anchor and promotes BC-2.16.015 to active on that story's merge per POL-14. BC-2.16.015 must be removed from S-ENGINE-LIMIT-EARLY-STOP-001 `behavioral_contracts:` frontmatter and moved to trace-only reference — story-writer to propagate. TD-VSDD-097: (1) Sibling pair — BC-2.16.015 (vulnerabilities) and BC-2.16.003 (devices) reviewed; BC-2.16.003 does not carry a LIMIT early-stop EC anchored to F-R11-CRIT-001; no mirror EC needed at this time; CLEAR. (2) Downstream copy target — EC-016-015-008 is a new EC row, not a verbatim copy-source in any downstream artifact; CLEAR. (3) Mandate anchor — EC-016-015-008 anchored to ADR-060 §D8.7 Condition A and S-ENGINE-LIMIT-EARLY-STOP-001; plan-shape gate Red Gate tests (RG-PSG-001 through RG-PSG-007) to be added by story-writer; no unanchored MUSTs. |
| 1.7 | adr060-limit-early-stop-vuln | 2026-08-26 | product-owner | **ADR-060 §D8 LIMIT-aware early-stop — TV-BC-2.16.015-006 and EC-016-015-007.** (1) TV-BC-2.16.015-006: canonical test vector for `SELECT * FROM claroty_vulnerabilities \| LIMIT 1` — 1 complete page fetched (page_size=1000, ~1.1 MB), `PipelineResult.truncated=false`, DataFusion trims to 1 row (ADR-060 §D8 + §Consequences). (2) EC-016-015-007: "LIMIT 1 fetches 1 page — early-stop" — early-stop fires after first complete page, `truncated=false`, DataFusion trims, no second HTTP POST issued; anchor: BC-2.16.002 §Postconditions LIMIT-Aware Early-Stop (ADR-060 §D8). (3) Related BCs: BC-2.16.002 added as dependency (early-stop contract anchor). TD-VSDD-097: (1) Sibling pair — BC-2.16.015 (vulnerabilities) and BC-2.16.003 (devices) are distinct table-contract BCs; BC-2.16.003 uses OffsetLimit pagination but has no LIMIT early-stop BC anchor requiring a mirror EC at this time; CLEAR. (2) Downstream copy target — TV and EC rows are not verbatim copy-sources in downstream artifacts; CLEAR. (3) Mandate anchors — EC-016-015-007 explicitly anchors to BC-2.16.002 §Postconditions LIMIT-Aware Early-Stop; TV-BC-2.16.015-006 references ADR-060 §D8 + §Consequences; implementing story is S-ENGINE-LIMIT-EARLY-STOP-001; no unanchored MUSTs introduced. |
| 1.6 | s-claroty-vulns-001-r4c-fix | 2026-08-25 | product-owner | F-VULNS-R4C-LOW-001/F-R4A-LOW-001: EC-016-015-002 + §Invariants null→absent precision corrected (absent `id` key = absent from `raw_extensions`, not null; mirrors EC-016-015-006 §published_date precision; TD-VSDD-097 dim-1 sibling sweep). §Postconditions §3 existing "null when absent" phrase corrected to match. F-R4A-OBS-001: §Postconditions §3 interim note added — pre-existing ENRICH-1 `column_source_path_extraction_failed` WARN behavior documented; deferred correction anchored to story `S-ENGINE-SOURCE-PATH-ABSENT-KEY-LOGLEVEL-001` (architect-ruled out of scope). input-hash refreshed (c913a02) to reflect architect ADR-058/ADR-028 edits in same burst. |
| 1.5 | s-claroty-vulns-001-pass-6-fix | 2026-08-25 | product-owner | F-VULNS-PC-MED-001: §Description "all remaining 18 columns are Tier-2" corrected to "17 Tier-2" (19 total − 2 Tier-1 = 17; consistent with §Postconditions §2 and RG-002). F-VULNS-PA-O01: EC-016-015-006 reworded — published_date aggregates into raw_extensions as a Tier-2 field; no standalone Datetime Arrow column is materialized. F-VULNS-PA-O02: §1 body_template illustrative block switched from toml to text fence — TOML single-quoted literals do not support backslash line-continuation; presentation is now explicitly illustrative-only. |
| 1.4 | s-claroty-vulns-001-pass-5-fix | 2026-08-25 | product-owner | F-VULNS-P5-001: §1 table_name claroty_vulnerabilities→vulnerabilities (registers as claroty_vulnerabilities per sibling convention); queryable-name refs corrected. F-VULNS-P5-002: §Error Cases atomic-fail correction (normalize post-accumulation → whole-result Err; no partial pages; consistent with Option-A fail-fast). |
| 1.3 | s-claroty-vulns-001-pass-4-fix | 2026-08-25 | product-owner | F-VULNS-ADV-001: §Invariants REQUIRED-semantics misattribution corrected (REQUIRED=push-down eligibility, not presence guarantee). EC-007/§Error-Cases E-SPEC-018: corrected demote-to-null→hard-error on present-unparseable datetime to match canonical engine (human-approved Option A). |
| 1.2 | s-claroty-vulns-001-pass-3-fix | 2026-08-25 | product-owner | F-VULNS-ANCHOR-001: §Architecture Anchors spec_driven_adapter.rs crate corrected prism-spec-engine→prism-bin (ground-truth: pipeline_result_to_record_batch lives in prism-bin). |
| 1.1 | s-claroty-vulns-001-pass-2-fix | 2026-08-25 | product-owner | F-VULNS-P1-004: §4 SAP-2 DTU-parity mandate annotated with D-2200 deferral + S-ADR058-DTU-PARITY-MIGRATION-001 anchor (TD-VSDD-097 dim-3). DTU Status traceability row updated to record deferral. |
| 1.0 | xdome-wave-a-f2-spec-evolution | 2026-08-24 | product-owner | Initial authoring — Claroty xDome vulnerabilities queryable surface contract per xdome-endpoint-expansion-plan.md Wave A G1 and spike-findings §Spike 1. TOML table contract, 19-column Tier-1/Tier-2 classification per ADR-058, PK rationale (name > id), SAP-2 exclusion documentation for 14 excluded fields. |
