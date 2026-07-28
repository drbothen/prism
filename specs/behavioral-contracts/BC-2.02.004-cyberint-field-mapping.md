---
document_type: behavioral-contract
level: L3
version: "1.14"
status: active
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-02"
capability: "CAP-003"
lifecycle_status: active
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "76729b7"
traces_to: ["CAP-003"]
extracted_from: ".factory/specs/prd.md"
scheduled_amendment_in: null
amendment_lifecycle: null
introduced: cycle-1
modified: "2026-07-27"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.02.004: Cyberint Alert Field Mapping to OCSF

## Description

> **Amendment — ADR-023 (PLUGIN-MIGRATION-001-G):** This BC previously described a
> hardcoded Rust mapper module (`prism-ocsf/src/mappers/cyberint.rs`). That
> implementation was deleted in PLUGIN-MIGRATION-001-C (PR #158). The field-mapping behavior
> described here is now delivered by `SpecDrivenMapper` reading `ocsf_field` column
> annotations from the Cyberint TOML sensor spec. The behavioral contract itself
> is unchanged — the same OCSF field mappings must be produced; they are now
> data-driven via TOML annotations per ADR-023 Rule 1.

`SpecDrivenMapper` reads `ocsf_field` column annotations from the Cyberint TOML sensor spec and converts alert and asset records fetched from the Cyberint Argos API to OCSF Detection Finding (class 2004) or other appropriate event classes. Timestamps are pre-processed by the CyberintTime 3-format parser (RFC-3339/ISO-8601, Unix epoch seconds, Unix epoch millis) before OCSF mapping. Severity string values ("high", "medium", "low") are mapped to OCSF `severity_id` enum integers per OCSF v1.x (`"high"` → `4` = "High", `"critical"` → `5` = "Critical"), with unrecognized values mapped to 99 (Other). Cyberint-specific fields (e.g., `threat_type`, `digital_asset_type`) are preserved in `raw_extensions`.

## Preconditions
- A Cyberint alert or asset record has been fetched via the Cyberint Argos API
- Timestamps have been parsed through the CyberintTime 3-format parser (RFC-3339/ISO-8601, Unix epoch seconds, Unix epoch millis)

## Postconditions
- Cyberint alert fields map to OCSF Detection Finding (class 2004, Security Finding 2001 deprecated) or appropriate event class
- Cyberint severity string (e.g., "high", "medium", "low") maps to OCSF `severity_id` enum values per OCSF v1.x: `"high"` → `4` ("High"), `"critical"` → `5` ("Critical"), `"medium"` → `3` ("Medium"), `"low"` → `2` ("Low")
- Cyberint timestamp (parsed via CyberintTime) maps to OCSF `time` in RFC 3339 format
- Cyberint-specific fields (e.g., `threat_type`, `digital_asset_type`) are preserved in `raw_extensions`
- DTU `Alert.affected_assets: Vec<serde_json::Value>` (wire-key `"affected_assets"`, emitted in the per-record JSON object by `routes::alerts::get_alerts` via `Alert` struct serialization per story S-WAVE-A-CYBERINT-SPEC-001 §T-05 item 5) is exposed as TOML column `affected_assets` with `column_type = "json"` and `ocsf_field = "raw_extensions.affected_assets"`. The prior TOML deferral ("array column type not yet supported", TOML comment F-LP3-HIGH-001) is superseded by `prism_core::column::ColumnType::Json` availability per ADR-024. SOC analyst agent reasoning: an alert without affected-asset context is an unexplainable verdict — the agent cannot triage priority or assess blast radius from severity alone. Deliberate-exclusion tracking: if this field is re-excluded in any future cycle, the reason MUST be documented in EC-02-006c below.

## Invariants
- DI-005: OCSF schema validity

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Warning | Unknown Cyberint severity string (not in known set) | Mapped to OCSF `severity_id: 99` (Other); warning logged |
| Warning | CyberintTime parser fails on all 3 formats | OCSF `time` set to fetch timestamp; raw string preserved in `raw_extensions`; warning logged (DEC-015) |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-015 | Timestamp value that cannot be parsed by any of the 3 CyberintTime formats (unparseable by all 3 CyberintTime formats) | Parse fails gracefully; fetch timestamp used as fallback; record not dropped |
| EC-02-006 | Cyberint asset record (not alert) -- different field structure | Separate field mapping for assets; maps to appropriate OCSF event class |
| EC-02-006b | `affected_assets` is an empty array (`[]`) | Emitted as `"affected_assets": []` on the wire; column produces empty JSON array string `"[]"`; not null. Valid: a Cyberint alert may have no impacted assets catalogued yet. |
| EC-02-006c | SAP-2 deliberate-exclusion tracking sentinel for `affected_assets` | This EC exists solely to satisfy SAP-2 rule 4. `affected_assets` was previously excluded via TOML comment (F-LP3-HIGH-001: "array column type not yet supported"). That exclusion is now LIFTED — the column MUST appear in the TOML. If any future change re-excludes `affected_assets`, the rationale must replace this row's Expected Behavior text, citing the specific technical constraint and the future story that will resolve it. An undocumented re-exclusion is a SAP-2 recurrence. |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.02.004-001 | Cyberint alert with severity="high" and ISO 8601 timestamp | `severity_id: 4` (High), `time` in RFC 3339; alert mapped to Detection Finding 2004 |
| TV-BC-2.02.004-002 | Unknown severity string "extreme" | `severity_id: 99` (Other); warning logged with raw value |
| TV-BC-2.02.004-003 | Timestamp string that fails all 3 CyberintTime formats (DEC-015) — e.g., `"Jan 01 2026 12:00"` | Fetch timestamp used; raw string in `raw_extensions`; warning logged; record not dropped |
| TV-BC-2.02.004-004 | Cyberint asset record (different schema) | Asset-specific mapper applies; maps to appropriate OCSF class |
| TV-BC-2.02.004-005 | Alert with `affected_assets: [{"asset_id": "a1", "type": "server"}]` | `affected_assets` column contains JSON array string `[{"asset_id":"a1","type":"server"}]`; the `"affected_assets"` key is present in the per-record JSON object emitted by `routes::alerts::get_alerts` (`Alert.affected_assets: Vec<serde_json::Value>` from `prism-dtu-cyberint::types`) |
| TV-BC-2.02.004-006 | Alert with `affected_assets: []` (empty array) | `affected_assets` column contains `"[]"` (not null or absent); no error emitted |

## TOML Contract

The following column MUST be added to `crates/prism-sensors/specs/cyberint-alerts.sensor.toml` inside the `alerts` table `[[tables.columns]]` block (F-SAP2-MED-004 / FB68d resolution; F-WASE-P66-HIGH-003 re-target: `cyberint.sensor.toml` is DELETED by `S-WAVE-A-CYBERINT-SPEC-001` T-01; `cyberint-alerts.sensor.toml` is the post-migration successor file created by T-02).

**POL-29 9c anchor (F-WASE-P66-HIGH-003):** This MUST is bound to `S-WAVE-A-CYBERINT-SPEC-001` T-02 (carry-over list). story-writer MUST add `affected_assets` to the T-02 carry-over list and bind the following to T-02: **AC:** "`cyberint-alerts.sensor.toml` `alerts` table declares `affected_assets` column (`column_type = "json"`, `ocsf_field = "raw_extensions.affected_assets"`)"; **Red Gate test:** test asserting the parsed `cyberint-alerts.sensor.toml` spec has an `affected_assets` column with `ColumnType::Json`, mirroring the existing IOC column Red Gate test pattern. **Assignment verified 2026-07-27 (FB73 — ITEM 4):** `S-WAVE-A-CYBERINT-SPEC-001` v1.8 AC-011 / RG-021 (`test_cyberint_alerts_toml_affected_assets_column_is_json_type`). POL-29 9c discharged: this MUST is anchored to `S-WAVE-A-CYBERINT-SPEC-001` AC-011, enforced by RG-021 (`test_cyberint_alerts_toml_affected_assets_column_is_json_type`). Assignments confirmed from `S-WAVE-A-CYBERINT-SPEC-001` v1.8 §Tasks RG-021 and AC-011 rows (authored FB72 leg 1 Item 4).

```toml
  # F-SAP2-MED-004 / FB68d: affected_assets — wire-key "affected_assets" present in per-record
  # JSON object from routes::alerts::get_alerts (Alert.affected_assets: Vec<serde_json::Value>,
  # prism-dtu-cyberint::types; emitted via Alert struct serialization per story
  # S-WAVE-A-CYBERINT-SPEC-001 §T-05 item 5).
  # Prior TOML deferral comment ("F-LP3-HIGH-001: array column type not yet supported")
  # superseded by ColumnType::Json availability (prism_core::column::ColumnType::Json, ADR-024).
  # ocsf_field: no standard OCSF Detection Finding field for a vendor affected_assets array;
  # flows to raw_extensions per BC-2.02.007 preservation contract.
  [[tables.columns]]
  name = "affected_assets"
  column_type = "json"
  ocsf_field = "raw_extensions.affected_assets"
```

The stale deferral comment "F-LP3-HIGH-001: 'affected_assets' (Vec<serde_json::Value>) deferred: array column type is not yet supported in the column grammar — track as future story when Array ColumnType lands." in the TOML file MUST be removed or replaced with a comment noting the column is now declared above.

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| VP-016 | OCSF normalization: output is valid protobuf (proptest) |
| VP-017 | OCSF normalization: unmapped fields preserved (proptest) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 |
| L2 Invariants | DI-005 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.14 | FB73 | 2026-07-27 | product-owner | ITEM 4 POL-29 9c anchor backfill: discharged §TOML Contract `affected_assets` MUST. Real story+AC+RGT cited: `S-WAVE-A-CYBERINT-SPEC-001` v1.8 AC-011 / RG-021 (`test_cyberint_alerts_toml_affected_assets_column_is_json_type`), verified from story v1.8 §Tasks. POL-29 9a: no named twin (distinct sensor, not a split-event sibling). 9b: §TOML Contract `[[tables.columns]]` code block — no known downstream verbatim copy. 9c: anchor now discharges — real story+AC+RGT in place. |
| 1.13 | FB71 | 2026-07-27 | product-owner | F-WASE-P66-HIGH-003: re-targeted §TOML Contract file reference from `cyberint.sensor.toml` (DELETED by S-WAVE-A-CYBERINT-SPEC-001 T-01) to `cyberint-alerts.sensor.toml` (post-migration successor created by T-02). Added POL-29 9c anchor block directing story-writer to add `affected_assets` to T-02 carry-over list with explicit AC and Red Gate test requirements. POL-29 9a: no named twin for BC-2.02.004 (BC-2.02.003/005/006 are distinct sensors, not split-event twins); 9b: §TOML Contract references a `[[tables.columns]]` code block — no known downstream verbatim copy in the perimeter; 9c: story-writer handoff documented with exact AC and RGT content required; MUST now anchored to `S-WAVE-A-CYBERINT-SPEC-001` T-02. |
| 1.12 | FB70 | 2026-07-27 | product-owner | F-WASE-P66-OBS-002: replaced all positional key-ordinal cites (`8th key in the static-fixture json! envelope`) with key-presence / symbol anchors per TD-VSDD-091. Three sites fixed: (1) §Postconditions — `emitted as the 8th key in the static-fixture json! envelope inside routes::alerts::get_alerts` → `emitted in the per-record JSON object by routes::alerts::get_alerts via Alert struct serialization per story S-WAVE-A-CYBERINT-SPEC-001 §T-05 item 5`; (2) TV-005 — `the DTU emits this as the 8th key in the static-fixture json! envelope` → key-presence description via `routes::alerts::get_alerts` and `Alert.affected_assets` symbol; (3) §TOML Contract TOML comment — `8th wire key in the static-fixture json! envelope` → `wire-key "affected_assets" present in per-record JSON object from routes::alerts::get_alerts`. Positional cites were decay class TD-VSDD-091 — self-invalidate when story S-WAVE-A-CYBERINT-SPEC-001 §T-05 replaces the hand-built json! literal with `serde_json::to_value(a)`. POL-29 9a: BC-2.02.004 has no named twin (distinct sensor, not a same-split sibling). POL-29 9b: §TOML Contract references a code block — no known downstream verbatim copy in perimeter. POL-29 9c: no new MUST statements; existing MUST in §Postconditions deliberate-exclusion sentinel and §TOML Contract are pre-existing and unmodified. |
| 1.11 | wave-5-e-demo-fidelity-fix-burst-68d | 2026-07-27 | product-owner | F-SAP2-MED-004: `affected_assets` coverage decision — EXPOSE. Ground truth: `Alert.affected_assets: Vec<serde_json::Value>` emitted as the 8th key in the `json!` envelope of `routes::alerts::get_alerts` static-fixture path; present in DTU `types.rs`. Prior TOML deferral ("F-LP3-HIGH-001: array column type not yet supported") superseded by `ColumnType::Json` availability per ADR-024. Added: §Postconditions `affected_assets` bullet; EC-02-006b (empty array behavior), EC-02-006c (SAP-2 deliberate-exclusion sentinel); TV-BC-2.02.004-005 (non-empty array), TV-BC-2.02.004-006 (empty array); §TOML Contract (exact column spec for implementer, including stale-comment removal instruction). |
| 1.10 | wave-a-spec-evolution-fix-burst-39 | 2026-07-24 | product-owner | F-WASE-P50-LOW-001: Corrected intra-burst narrative inconsistency in v1.9 changelog row. The POL-29 census in v1.9 characterised BC-2.01.013 as "needs separate pass" — but BC-2.01.013 v1.14→v1.15 was adjudicated COMPLETE within fix-burst-38 itself (PREREQ-F v1.4 confirmed full ADR-023 amendment; banner→completion note; scheduled_amendment_in+amendment_lifecycle→null). Amended v1.9 census to: 1 legitimately-pending remainder (BC-2.16.004 only); BC-2.01.013 resolved same burst. POL-29 sibling sweep: "needs separate pass" phrase appeared only in this BC's v1.9 changelog row — no other live spec sites required correction (prd.md, BC-INDEX, VP files, ADR files: all references to BC-2.01.013 are non-census navigational cites — not affected). |
| 1.9 | wave-a-spec-evolution-fix-burst-38 | 2026-07-24 | product-owner | F-WASE-P49-LOW-001: `scheduled_amendment_in` cleared (ADR-023 amendment completed in v1.6 PLUGIN-MIGRATION-001-G, 2026-05-27); set to `null`; added `amendment_lifecycle: null` to match sibling BC-2.01.006 cleared-state convention. POL-29 sweep: 7 other BCs carry non-null `scheduled_amendment_in: ADR-023` — 5 confirmed stale (amendment completed per changelog, field not cleared; all outside Wave-A perimeter): BC-2.02.005 (Claroty field mapping), BC-2.02.003 (CrowdStrike field mapping), BC-2.01.005 (CrowdStrike oauth2), BC-2.02.006 (Armis field mapping), BC-2.01.007 (Claroty bearer); 1 legitimately pending: BC-2.16.004 (rust-escape-hatch, `amendment_lifecycle: pending` present, no PLUGIN-MIGRATION-001-G completion entry found). BC-2.01.013 (datasource-trait-adapter-pattern) was adjudicated COMPLETE within this same burst (v1.14→v1.15: PREREQ-F v1.4 confirmed full ADR-023 amendment; banner→completion note; scheduled_amendment_in+amendment_lifecycle→null) — the characterization "needs separate pass" was inaccurate; corrected by v1.10 (F-WASE-P50-LOW-001). |
| 1.8 | wave-a-spec-evolution-fix-burst-37 | 2026-07-24 | product-owner | F-WASE-P48-MED-001: §Edge Cases DEC-015 and §Canonical Test Vectors TV-BC-2.02.004-003 still contained "5th format" residue not covered by the v1.7 sweep (which fixed §Description, §Preconditions, §Error Cases only). DEC-015: "Timestamp in unexpected 5th format" → "Timestamp value that cannot be parsed by any of the 3 CyberintTime formats (unparseable by all 3 CyberintTime formats)". TV-003: "Timestamp in unknown 5th format (DEC-015)" → "Timestamp string that fails all 3 CyberintTime formats (DEC-015) — e.g., `"Jan 01 2026 12:00"`". Phrasing matches sibling BCs BC-2.01.006 and BC-2.01.018. POL-29 sweep: no remaining "4-format", "4 formats", "5th format", "fourth format" in live BC body (changelog rows exempt). |
| 1.7 | wave-a-rmu-amendment-burst-1 | 2026-07-23 | product-owner | POL-29 sweep: RU-Q5 parity amendment. "4-format" → "3-format" throughout (§Description, §Preconditions, §Error Cases). No "Cyberint custom format" exists per canonical Cyberint OpenAPI (`cyberint_alerts_openapi_06.20.2026.json`); parity with BC-2.01.018 v1.4 / BC-2.01.006 v1.7. |
| 1.6 | PLUGIN-MIGRATION-001-G | 2026-05-27 | product-owner | AC-002 amendment: removed PENDING AMENDMENT banner; added Amendment Note to Description; updated mechanism language from deleted `prism-ocsf/src/mappers/cyberint.rs` to SpecDrivenMapper + ocsf_field TOML annotations; bumped status draft→active; removed amendment_lifecycle: pending. |
| 1.5 | prereq-f | 2026-05-11 | product-owner | PREREQ-F prefix note: added PENDING AMENDMENT — ADR-023 callout under H1 per ADR-023 L370 wording; added scheduled_amendment_in: ADR-023 and amendment_lifecycle: pending to frontmatter. No semantic change to BC body. Full amendment in Wave 2/G. |
| 1.4 | S-1.04-red-gate-fix | 2026-04-22 | product-owner | Corrected TV-001 annotation: severity_id 4 = "High" (was "Critical") per OCSF v1.x; updated Description and Postconditions to enumerate full severity mapping. |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
