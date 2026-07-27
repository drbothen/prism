---
document_type: adversarial-review
review_id: wave-a-spec-pass-66
pass_number: 66
frozen_head: "factory-artifacts@7426696b3"
perimeter: wave-a-spec-evolution
mode: spec-review with code grounding
streak_pre_pass: "0/3"
streak_post_pass: "0/3"
verdict: BLOCKED
findings_count: 16
severity_breakdown:
  critical: 1
  high: 5
  medium: 4
  low: 3
  observation: 2
  process_gap: 1
novelty: HIGH
related_state_decision: D-2051
date: 2026-07-27
open_findings: [CRIT-001, HIGH-001, HIGH-002, HIGH-003, HIGH-004, HIGH-005, MED-001, MED-002, MED-003, MED-004, LOW-001, LOW-002, LOW-003, PROCESS-GAP-001, OBS-001, OBS-002]
version: "1.0"
changelog:
  - version: "1.0"
    date: 2026-07-27
    author: state-manager
    note: "Initial persistence of LOCAL adversary pass 66 report."
---

# Adversarial Review — Wave-A Spec-Evolution Perimeter, LOCAL Pass 66

**Frozen `factory-artifacts` HEAD:** `7426696b3` · **Mode:** spec-review with code grounding · **Context:** fresh, no prior passes read

---

## Critical Findings

### F-WASE-P66-CRIT-001 — BC-2.01.006 v1.8 (status active) contracts `(Timestamp, AssetID)` cursor pagination for a Cyberint Assets API that has no cursor parameter

**Artifacts:** `BC-2.01.006` §Description, §Postconditions, TV-001, TV-002, EC-01-009; `BC-2.01.018` §Related BCs; `S-WAVE-A-CYBERINT-SPEC-001` §"Pagination treatment — GAP-ASSETS-PAG-001 [EXPLICIT BLOCKER]"

**Defect.** BC-2.01.006 §Description/§Postconditions/TV-001/TV-002/EC-01-009 all mandate a `(Timestamp, AssetID)` 2-tuple cursor. Ground truth: a case-insensitive search for `cursor` across `reference/api-specs/cyberint_assets_openapi_06.20.2026.json` returns zero matches; `GetAssetsRequest` is `{customer_id, page_number, type, ...}` and `GetAssetsResponse` is `{total_assets, page_number, assets}` — no cursor field exists in the schema. Story T-03 §"Pagination treatment — GAP-ASSETS-PAG-001 [EXPLICIT BLOCKER]" mandates the opposite: author WITHOUT a `[tables.steps.pagination]` block.

Regression axis: FB66 removed all cursor language from the twin BC-2.01.018 (Alerts) on exactly this rationale, but never swept BC-2.01.006 (Assets) — same ADR-053 §D3 split, same subsystem SS-01, same capability CAP-001. The FB66 edit compounded the defect by writing into BC-2.01.018 §Related BCs that the "Assets surface retains `(Timestamp, AssetID)` cursor pagination," propagating a false cross-reference in the same burst. This is the first instance of the within-burst sibling-sweep failure pattern recorded in TREND-POL29-SIBLING-SWEEP-001.

**Routing:** product-owner (BC-2.01.006 §Description, §Postconditions, TV-001, TV-002, EC-01-009 AND BC-2.01.018 §Related BCs).

---

## Important Findings

### F-WASE-P66-HIGH-001 — ADR-056 §D10 and story RG-018 assert the wire literal `"pagination_type": "page"`, but `PaginationType` cannot emit it

**Artifacts:** `ADR-056-page-number-pagination-variant.md` §D10, §Consequences; `S-WAVE-A-CYBERINT-SPEC-001` T-09 step 6 + RG-018

**Defect.** ADR-056 §D10 and story RG-018 assert the MCP wire literal `"pagination_type": "page"`. Ground truth: `prism-spec-engine::types` `PaginationType` derives `Serialize` with no `rename_all` attribute and no per-variant rename, so externally-tagged serde emits variant identifiers verbatim. The live MCP wire vocabulary is `"Cursor"` / `"Offset"` / `"None"`, and adding a `Page` variant yields `"Page"` — not `"page"`. Contrast `PaginationConfig` in `spec_parser`, which DOES carry `#[serde(tag = "type", rename_all = "snake_case")]` — that is why ADR-056 §D1's separate `page_number` TOML-tag claim is correct. That reasoning was transferred to `PaginationType` where it does not hold.

Three bad options arise, none acknowledged by ADR or story: (a) implement as specified and RG-018 fails permanently; (b) add `rename_all = "snake_case"` and silently break the existing `"Cursor"` / `"Offset"` MCP wire values; (c) rename `Page` alone, yielding the incoherent mixed vocabulary `{"Cursor", "Offset", "None", "page"}`.

**Routing:** architect (ADR-056 §D10 — state which mechanism produces the literal and whether the three existing wire values change), then story-writer (T-09 step 6 + RG-018).

---

### F-WASE-P66-HIGH-002 — BC-2.02.006 v1.7 §TOML Contract mandates six columns be added to `armis.sensor.toml`; none exist and no story/AC/RGT/deferral anchors the work (POL-38)

**Artifacts:** `BC-2.02.006` §TOML Contract; `armis.sensor.toml` `devices` table column list; `.factory/stories/` (grep for `risk_factors`, `os_version`, `F-SAP2-MED-005`, `FB68d`)

**Defect.** BC-2.02.006 §TOML Contract mandates adding `os_version`, `risk_factors`, `network_id`, `site`, `tags`, and `device_cves` to the Armis devices TOML spec. The `devices` table currently declares `device_id`, `name`, `type`, `manufacturer`, `last_seen`, `first_seen`, `ip_address`, `mac_address`, `os_name`, `risk_score`, `aql`, and `device_cves_first` — none of the six mandated columns appear. A grep of `.factory/stories/` for any of `risk_factors`, `os_version`, `F-SAP2-MED-005`, or `FB68d` finds only historical stories and the CrowdStrike analogue `S-DRIFT-SAP2-DEVICES-TOML-SURFACE-001`. No story or AC/RGT owns the work.

The same amendment correctly anchored its sibling deferral EC-02-014 to a real story, so the MUST-with-anchor discipline was available and applied asymmetrically. This is the second instance of the within-burst sibling-sweep failure pattern recorded in TREND-POL29-SIBLING-SWEEP-001.

**Routing:** story-writer (create the Armis TOML-surface story mirroring `S-DRIFT-SAP2-DEVICES-TOML-SURFACE-001`), then product-owner (anchor reference in BC-2.02.006 §TOML Contract), then state-manager (STORY-INDEX).

---

### F-WASE-P66-HIGH-003 — BC-2.02.004 v1.11 §TOML Contract mandates adding `affected_assets` to a file the story DELETES, and T-02's carry-over list omits it

**Artifacts:** `BC-2.02.004` §TOML Contract; `cyberint.sensor.toml` (DELETED by T-01); `S-WAVE-A-CYBERINT-SPEC-001` T-01, T-02 carry-over list, §File Structure Requirements; `prism-dtu-cyberint::types` `Alert.affected_assets`

**Defect.** Three interlocking parts: (1) Unanchored MUST — no `affected_assets` column exists in `cyberint.sensor.toml`; the stale `F-LP3-HIGH-001 ... deferred` comment survives verbatim, and no story/AC/RGT owns delivery; (2) T-01 is literally "Delete cyberint.sensor.toml" and §File Structure Requirements marks it DELETE — so the §TOML Contract mandate is targeted at a file about to be deleted; (3) T-02's enumerated carry-over list names the 8 IOC columns but NOT `affected_assets`, so the mandate is silently dropped by the migration.

The DTU side is correctly in place: `prism-dtu-cyberint::types` `Alert.affected_assets: Vec<serde_json::Value>` exists and the static-fixture `json!` literal emits it as its last key. Routing: story-writer (add `affected_assets` to T-02 carry-over list + covering AC/RGT), then product-owner (re-target BC-2.02.004 §TOML Contract to `cyberint-alerts.sensor.toml` + add story anchor).

---

### F-WASE-P66-HIGH-004 — `armis.sensor.toml` column `device_cves_first` has no `source_path` and no backing `DeviceRecord` field; NULL on both static-fixture paths (SAP-2 §Rule 6)

**Artifacts:** `armis.sensor.toml` `devices` table `device_cves_first` column; `prism-dtu-armis::types` `DeviceRecord`; `crates/prism-dtu-armis/src/routes/search.rs` static branch; `crates/prism-dtu-armis/src/routes/devices.rs` static branch; `BC-2.02.006` EC-02-013, TV-BC-2.02.006-011

**Defect.** `DeviceRecord` has `device_cves: Vec<String>` and no `device_cves_first` field. Both static paths serialize the full struct — the `search.rs::get_search` static branch and the `devices.rs` static branch both call `serde_json::to_value(&merged)`, emitting `device_cves` but never `device_cves_first`. The key exists only on the generated-records path, where `search.rs` conditionally strips it. Verbatim SAP-2 §Rule 6 path-dependence: seeded demos pass; unseeded production silently emits NULL.

The correct sibling pattern already ships in the workspace: `cyberint.sensor.toml` `iocs_value_first` uses `source_path = "$.iocs[0].value"`; the Armis analogue is `source_path = "$.device_cves[0]"`. Partial-documentation-not-discharge: BC-2.02.006 EC-02-013 covers only the `device_cves = []` case, and TV-BC-2.02.006-011 asserts the opposite (populated array implies `device_cves_first` set), which is unsatisfiable on the static path.

**Routing:** story-writer (add `source_path` task to the HIGH-002 story), product-owner (EC-02-013 and TV-BC-2.02.006-011 must state the mechanism governing the static path).

---

### F-WASE-P66-HIGH-005 — Phantom interpolation namespace `${variable.*}` asserted in active BC-2.02.006 §Postconditions and propagated to two further artifacts; the real mechanism `${step_name.field}` ships today

**Artifacts:** `BC-2.02.006` §Postconditions; `S-WAVE-A-ARMIS-ACTIVITY-001` §Blocking Dependency, §Architecture Compliance Rules §4, T-01; `STORY-INDEX.md` description cell for S-WAVE-A-ARMIS-ACTIVITY-001

**Defect.** A grep of `\$\{variable\.` across `crates/` returns zero matches; across `.factory/` it returns 6 hits in `S-WAVE-A-ARMIS-ACTIVITY-001`, 1 in BC-2.02.006 §Postconditions, and 1 in STORY-INDEX. `prism-spec-engine::interpolation` has no `variable` namespace. The contracted mechanism is BC-2.16.002 §Postconditions (`${query_ids.resource_ids}` resolves from the step named `query_ids`; array-valued variables trigger fan-out) and BC-2.16.009 §Validation Rule 6 confirms `${step.field}` is the runtime namespace. The two-step pattern already ships in `crowdstrike.sensor.toml` (`query_detection_ids` → `fetch_detections` with `body_template = '{"ids": ${query_detection_ids.resources}}'`).

Consequences: an active BC asserts a nonexistent grammar token (POL-22 Phase C); the architect is being asked the wrong question, so the answer cannot unblock the story; Canonical Principle §Rule 6 violation (a mechanical question deferred to architect review when answerable from BC-2.16.002 + BC-2.16.009 + the shipped CrowdStrike spec). The genuine residual question — which the story does not ask — is whether `path_template` array fan-out dispatches one request per scalar value or one per batch of up to 100 (BC-2.16.002 says "once per batch"), since a per-device path needs the former.

**Routing:** story-writer (rewrite §Blocking Dependency to cite `${step_name.field}` and narrow the architect question to per-scalar-vs-per-batch fan-out; fix §Architecture Compliance Rules §4 and T-01), product-owner (BC-2.02.006 §Postconditions), state-manager (STORY-INDEX description cell).

---

## Medium Findings

### F-WASE-P66-MED-001 — ADR-056 §D10 CE-2 creates a new spec-load validation behavior with an exact message and RG-017, but no BC contracts it

**Artifacts:** `ADR-056-page-number-pagination-variant.md` §D10 CE-2, §Consequences; `BC-2.16.009` §Validation Rule 4; `BC-2.16.002` §Postconditions PageNumber row

**Defect.** ADR-056 §D10 CE-2 specifies a spec-load-time validation behavior with a precise error message and cites RG-017 as the test. BC-2.16.009 §Validation Rule 4 enumerates only `cursor_token`, `offset_limit`, and `none` rules — no `page_number` row exists. Two compounding gaps in the same amendment: (a) ADR-056 §Consequences "For the product-owner" says only "Author the BC-2.16.002 §Postconditions row specified in D8 … No BC content is authored here," so the PO leg was structurally incapable of closing the §Validation Rule 4 gap; (b) ADR-056 §D8 still carries the pre-v0.2 phrasing ("`page_size = 0` skips injection") that §D3/§D5 explicitly set out to correct, and §D8 is the verbatim text the PO copies into BC-2.16.002 — so the corrected ADR propagates its own pre-correction phrasing downstream. This is the third instance of the within-burst sibling-sweep failure pattern recorded in TREND-POL29-SIBLING-SWEEP-001 (the "downstream copy-target" arm).

**Routing:** architect (ADR-056 §D8 + §Consequences correction), then product-owner (BC-2.16.009 §Validation Rule 4 `page_number` row; BC-2.16.002 two-layer restatement).

---

### F-WASE-P66-MED-002 — ADR-053 `anchor_stories` carries per-entry `§Authority:` verification annotations for two stories that contain no §Authority section (SAC-2 / POL-22 Phase A)

**Artifacts:** `ADR-053-declarative-http-auth-provider-split.md` `anchor_stories` block; `S-WAVE-A-ENGINE-001`; `S-WAVE-A-CYBERINT-SPEC-001`

**Defect.** Ground truth: `## Authority` exists in only `S-WAVE-A-MCP-001` and `S-WAVE-A-ARMIS-REMEDIATION-001`. `S-WAVE-A-ENGINE-001` contains the word "Authority" zero times; `S-WAVE-A-CYBERINT-SPEC-001` contains it once in the unrelated phrase "(human-owned per CLAUDE.md §Pipeline Authority)". SAC-2's stated ground-truth basis therefore does not exist for two of four entries, and the annotations assert a verification that could not have been performed.

Note: `S-WAVE-A-CYBERINT-PATCH-001` is correctly ABSENT per SAC-2 §Rule 2 (no §Authority section naming ADR-053).

**Routing:** architect (correct or drop the false `§Authority:` verification claims in `anchor_stories`), story-writer (add real `## Authority` sections to the two affected stories).

---

### F-WASE-P66-MED-003 — VP-157 v1.0 declares `status: draft` with a `[TODO]` harness deferred to a story merged 2026-04-29, while the property is already proven by multiple citing tests; skeleton cites three nonexistent symbols (POL-31, POL-14, TD-VSDD-059)

**Artifacts:** `vp-157-dtu-clone-unsupported-failure-mode-propagation.md` §Status, §Proof Harness Skeleton SYMBOL RESOLUTION block; `crates/prism-dtu-harness/tests/bc_3_6_001_ops_clone_failure_modes.rs`; `VP-INDEX.md` VP-157 row; anchor story `S-3.6.01`

**Defect.** The anchor story `S-3.6.01` is recorded MERGED (PR #83) in STORY-INDEX — a merged story cannot author a future harness. The property is demonstrably proven: `crates/prism-dtu-harness/tests/bc_3_6_001_ops_clone_failure_modes.rs` cites VP-157 and asserts exact bodies across Jira/PagerDuty/Slack/Claroty/Armis clones; `unsupported_failure_mode` is implemented in all seven `clones/*.rs` and `clone_server.rs`.

POL-31 violation: the skeleton names `build_test_harness_single_jira_clone`, `post_dtu_configure`, and `issue_creation_request_jira` — a grep of `crates/` returns zero occurrences of any of these symbols. TD-VSDD-059 paper-fix signature: the "file gap closed" burst transcribed VP-INDEX metadata without checking implementation state, producing a document that misrepresents verification status.

**Routing:** architect (promote VP-157 status from draft to active, replace phantom symbols with real test identifiers from `bc_3_6_001_ops_clone_failure_modes.rs`, and align the proof method with the actual test evidence).

---

### F-WASE-P66-MED-004 — STORY-INDEX registers `S-WAVE-A-ARMIS-ACTIVITY-001` under BC-2.02.006's reverse map while the story's `behavioral_contracts` frontmatter is `[]` (POL-8/POL-13/POL-37)

**Artifacts:** `STORY-INDEX.md` §BC Traceability Matrix BC-2.02.006 row; `S-WAVE-A-ARMIS-ACTIVITY-001` frontmatter `behavioral_contracts:`, §Behavioral Contracts table

**Defect.** Three-way inconsistency: the STORY-INDEX matrix row asserts the BC-2.02.006 linkage, the story frontmatter `behavioral_contracts` is `[]`, and the story body §Behavioral Contracts table lists BC-2.02.006 v1.7. Sibling STORY-INDEX rows are frontmatter-derived per POL-37.

Pending intent verification: the empty array is deliberate and annotated (S-7.01 spec-first gate) and the body row frames the reference in terms of EC-02-014, so the STORY-INDEX matrix row may be the defect rather than the frontmatter. Orchestrator adjudication required before the owning specialist acts. Secondary: the frontmatter has no `holdout_scenarios:` key at all; POL-35's exemption is keyed on `holdout_scenarios: []`, so an absent key is outside the stated escape.

**Routing:** state-manager (STORY-INDEX matrix row) pending orchestrator adjudication, or story-writer (if the body §Behavioral Contracts table is the side to change).

---

## Low Findings

### F-WASE-P66-LOW-001 — ADR-056's changelog heading is `## §Changelog`; all six sibling ADRs (ADR-050 through ADR-055) use `## Changelog`, so `ADR-056 §Changelog` citations cannot resolve (POL-21)

**Artifact:** `ADR-056-page-number-pagination-variant.md` §Changelog heading

**Defect.** ADR-056 uses `## §Changelog` while all six sibling ADRs (ADR-050, ADR-051, ADR-052, ADR-053, ADR-054, ADR-055) use `## Changelog`. Citations to `ADR-056 §Changelog` cannot resolve to the correct section because the heading diverges from the established workspace convention. Blast radius: 1 file.

**Routing:** architect.

---

### F-WASE-P66-LOW-002 — Story AC-003/T-05 prescribe a DTU alerts envelope key `page` that `GetAlertsResponse` does not contain

**Artifacts:** `S-WAVE-A-CYBERINT-SPEC-001` AC-003, T-05; `prism-dtu-cyberint::types` `GetAlertsResponse`

**Defect.** Ground truth: `GetAlertsResponse` has exactly `{total, alerts}`. The story's own §Architecture Compliance Rules §1 makes ADR-028 §D1 DTU-wire-parity binding, and T-05 justifies itself on that rule. The Assets response does carry `page_number`; that is the likely borrow source for the incorrect `page` key in the Alerts handler. Harmless to `$.alerts` extraction but a fidelity divergence a dtu-validator run would flag.

**Routing:** story-writer.

---

### F-WASE-P66-LOW-003 — VP-158 has zero citations in `crates/`, and VP-157/VP-158 VP-INDEX rows carry no version pin while same-era siblings VP-160/VP-161 do

**Artifacts:** `VP-INDEX.md` VP-157 row, VP-158 row; `crates/prism-dtu-demo-server` harness; `vp-158-dtu-demo-e2e-scenario-progression.md`

**Defect.** `E-DEMO-006` ships (`prism-dtu-demo-server::harness`, exercised in `bc_2_06_019_scenario_progression.rs`) but the proof is untraceable from the VP — no test or implementation cites VP-158. Unpinned VP-INDEX rows count as "unverifiable, not clean" per the cross-document index check capability disclosure; 434 of 496 BC-INDEX rows share this status, but VP-157/VP-158 are specifically flagged because same-era siblings VP-160 and VP-161 carry version pins.

**Routing:** architect (VP-158 proof anchor: cite `bc_2_06_019_scenario_progression.rs` and the harness symbol in §Proof Method) + state-manager (VP-INDEX version pin cells for VP-157 and VP-158).

---

## Process Gaps

### F-WASE-P66-PROCESS-GAP-001 `[process-gap]` — CLAUDE.md SAP-2 §Rule 2 cites `cyberint-alerts.sensor.toml` as an existing exemplar; no such file exists

**Artifact:** `CLAUDE.md` SAP-2 §Rule 2 exemplar citation

**Defect.** A glob of `**/*.sensor.toml` returns 22 paths, none named `cyberint-alerts.sensor.toml`. The file actually carrying the `created_at` / `timestamp_formats = ["iso8601","unix_epoch_seconds"]` declaration is `crates/prism-sensors/specs/cyberint.sensor.toml`; `cyberint-alerts.sensor.toml` is a FUTURE artifact created by draft story T-02. Why this matters: SAP-2 §Rule 2 exists specifically to prevent false findings by grounding the probe in an existing file. An adversary following the citation reads a nonexistent path — the probe that guards against false findings currently seeds one. Same class as the defects SAP-2 §Rule 6 was written to catch: a statement true of the intended future state and false of the wire.

**Attribution note:** This sentence was authored by the orchestrator during the 2026-07-27 human-approved CLAUDE.md amendment — record that attribution.

**Routing:** human / orchestrator (CLAUDE.md is human-owned per §Pipeline Authority). Suggested correction: cite `cyberint.sensor.toml` as the live exemplar and note `cyberint-alerts.sensor.toml` as the post-migration successor. Recorded as PENDING item (h) in STATE.md.

---

## Observations

### F-WASE-P66-OBS-001 — BC-2.02.006 grounds its wire-emission evidence on `routes::devices::paginate_devices`, which is not the route the pipeline calls for the `devices` table

**Artifact:** `BC-2.02.006` §Postconditions wire-emission evidence citation

**Defect.** `armis.sensor.toml` declares `path_template = "/api/v1/search?aql=${query.filter.aql}"`, so `PipelineExecutor` calls `routes::search::get_search`; `search.rs` states explicitly: "This is the CANONICAL path for `from armis.devices` queries … NOT GET /api/v1/devices". The conclusion holds on both paths (both call `serde_json::to_value(&merged)` over `DeviceRecord`) so no wrong behavior follows, but the authoritative emission site is mis-named and SAP-2 §Rule 6 requires naming the governing path.

**Routing:** product-owner (correct BC-2.02.006 §Postconditions to cite `routes::search::get_search` as the governing emission site for the `devices` table).

---

### F-WASE-P66-OBS-002 — BC-2.02.004 TV-005's "8th key in the static-fixture `json!` envelope" self-invalidates when story T-05 lands

**Artifact:** `BC-2.02.004` TV-005 positional key citation

**Defect.** TV-005 accurately describes the current static-fixture `json!` envelope and its 8th key ordinal. However, T-05 item 5 mandates replacing the 8-key literal with `serde_json::to_value(a)`, after which the envelope and its key ordinal cease to exist. A positional cite to a construct scheduled for deletion is the same decay class TD-VSDD-091 targets — accurate today, stale on the day T-05 merges, and invisible to automated gates.

**Routing:** product-owner (restate TV-005 in terms of key presence rather than ordinal position).

---

## Dismissed

Findings investigated and rejected as non-defects — recorded for future-pass reference to prevent re-investigation:

| Candidate | Why dismissed |
|---|---|
| SAP-1 catalog completeness | Full workspace grep of `event_type =` across `crates/**/src/**/*.rs`; every value resolves to a BC-2.16.002 catalog row. The apparent `infusion.coercion_failed` outlier is inside a `///` doc comment on the `E-INFUSE-014` variant, not a tracing emission. PASS. |
| SAP-2 §Rule 2 datetime pairing for `cyberint created_at` | `cyberint.sensor.toml` `created_at` is `column_type = "datetime"` against a `serde_json::Value` wire field with a declared `timestamp_formats` chain — explicitly permitted by the amended §Rule 2. PASS. |
| `armis` column `type` shadowing Rust keyword | `DeviceRecord` uses `#[serde(rename = "type")]` on `device_type`. PASS. |
| `armis` column `aql` coverage | Documented push-down pseudo-column; BC-2.11.007 §Mechanism B. PASS. |
| POL-24 on CE-2 message form | Byte-parallel to the shipped `OffsetLimit` arm in `validation.rs`; pre-existing workspace convention for `SpecErrorCode::ESpec001`. PASS. |
| POL-22 on Assets OpenAPI schema | File exists at `reference/api-specs/cyberint_assets_openapi_06.20.2026.json`; `Asset.required = ["id","created","updated"]`; exactly 12 properties → 11 columns after the ratified `compensating_controls` exclusion. Story T-03 table and T-04 struct both accurate. PASS. |
| POL-22 on GAP-ASSETS-PAG-001 framing | `GetAssetsRequest` has `page_number` with no `page_size`; the CWE-390 blocker framing is correct. PASS. |
| POL-22 on Assets endpoint path | Verified against OpenAPI. PASS. |
| POL-22 on T-02 `GetAlertsRequest` claims | Verified byte-for-byte against DTU types. PASS. |
| ADR-056 §D10 CE-1 exhaustiveness | `sensor_table_descriptor_from_table_spec` matches exhaustively with no wildcard arm. PASS. |
| ADR-056 §D1 serde tag on `PaginationConfig` | `#[serde(tag = "type", rename_all = "snake_case")]` present. PASS. |
| ADR-056 §D9 `#[non_exhaustive]` presence | Attribute present on `PaginationConfig`. PASS. |
| SAC-1 on S-WAVE-A-ARMIS-ACTIVITY-001 and S-WAVE-A-CYBERINT-SPEC-001 | ARMIS-ACTIVITY is draft with explicitly-annotated PENDING RG list. CYBERINT-SPEC has 20 enumerated RGTs, a density-check paragraph, and red-then-green task ordering. PASS. |
| SAC-2 on S-WAVE-A-CYBERINT-PATCH-001 absence from ADR-053 `anchor_stories` | Derivation rule is "verified from §Authority citations in each story." PATCH-001 has no §Authority naming ADR-053; cites BC-2.16.009 and mentions ADR-053 only to exclude itself. Correctly absent. PASS. |
| POL-37 / cross-document index check index pins | Version pins present for verified rows. PASS. |
| VP `timestamp` / `modified` convention across VP-157/158/159/160/161 | `modified: []` convention for VPs is deliberate; `timestamp` fields match the DRIFT-VP-MODIFIED-CORPUS-001 correction. PASS. |
| VP-160 77-character tchar set and E-SPEC-027(a) charset | Recomputed 15+26+26+10 = 77. PASS. |
| TD-VSDD-091 on newly-written perimeter text | All new cites use symbol/section/anchor form. PASS. |
| Known/accepted per dispatch | The 8 IOC columns absent from the alerts static path (tracked by T-05/AC-009/RG-019), `armis_device_activity` deferral, `compensating_controls` exclusion, the stale "92→93" wording in T-04, and the 39 L1 + 86 L7 corpus debt. Not re-reported. |

---

## Severity Breakdown

| Severity | Count | IDs | Status |
|---|---|---|---|
| CRITICAL | 1 | CRIT-001 | OPEN |
| HIGH | 5 | HIGH-001 … HIGH-005 | All OPEN |
| MEDIUM | 4 | MED-001 … MED-004 | All OPEN |
| LOW | 3 | LOW-001, LOW-002, LOW-003 | All OPEN |
| PROCESS-GAP | 1 | PROCESS-GAP-001 | OPEN (human gate / PENDING item (h)) |
| OBSERVATION | 2 | OBS-001, OBS-002 | OPEN |
| **Total** | **16** | | **16 OPEN / 0 closed** |

---

## Novelty Assessment

**Novelty: HIGH.** Three findings (CRIT-001, HIGH-002, MED-001) share ONE generative pattern that no single-artifact review surfaces: **a fix-burst amends one artifact of a sibling pair, or writes a MUST into a BC, and the counterpart artifact or downstream copy-target is never swept.** (i) FB66 removed cursor language from BC-2.01.018 and left it standing in its ADR-053 §D3 twin BC-2.01.006, while simultaneously writing a new false cross-reference into BC-2.01.018 §Related BCs pointing back at BC-2.01.006 as the cursor-holding artifact. (ii) FB68d wrote two §TOML Contract MUST blocks with zero story anchors in BC-2.02.006 while correctly anchoring the sibling EC-02-014 in the same amendment — discipline applied asymmetrically. (iii) ADR-056 v0.2 corrected §D3/§D4/§D5/§D9 but did not sweep §D8 — the verbatim text the product-owner copies downstream into BC-2.16.002 — so the corrected ADR propagates its own pre-correction phrasing.

That is a **POL-29 within-burst sibling-sweep pattern at 3+ recurrences in a single perimeter, meeting the project's 3-recurrence codification threshold.** Registered as TREND-POL29-SIBLING-SWEEP-001; requires a structural intervention (mandatory sibling-pair / downstream-copy-target / mandate-anchor checklist at every future fix-burst dispatch) rather than another content fix-burst.

Two further findings (HIGH-001 serde encoding, HIGH-005 phantom `${variable.*}`) are ground-truth-only discoveries: both artifacts read internally coherent and pass document-level review; only reading `prism-spec-engine::types` and `prism-spec-engine::interpolation` exposes them.

Two of the conventions codified 2026-07-27 produced first-application defects: SAC-2 produced MED-002 (false `§Authority:` verification annotations for two stories lacking the section), and SAP-2 §Rule 2 produced PROCESS-GAP-001 (cite points at future artifact rather than live file). Expected signal for a fresh convention; worth a codification follow-up in lessons.md.

---

```
CLEAN (strict): no
CLEAN (PR-merge): no
```

**BC-5.39.001 consequence:** 3-CLEAN streak 0/3 (unchanged). Fix-burst cascade required on STRICT criterion.

**TD-VSDD-096 NOT applicable** — finding set includes contract-semantics, API-contract, algorithm, and mandate-anchor defects; full cascade ceremony applies.
