---
document_type: adversarial-review
pass: 72b
scope: local
perimeter: wave-a-spec-evolution (complementary coverage)
frozen_head: "24d4fc564"
develop_head_noted: "aa2a5fe6e"
timestamp: 2026-07-30T00:00:00Z
producer: adversary
CLEAN_strict: "no"
CLEAN_PR_merge: "no"
finding_counts: {HIGH: 3, MED: 4, LOW: 2, OBS: 2, total: 11}
novelty: HIGH
---

# Adversarial Review — LOCAL Pass 72b (Complementary Coverage)

```
CLEAN (strict):   no
CLEAN (PR-merge): no
```

---

## Findings Ledger

### F-WASE-P72B-HIGH-001 — VP-161 is absent from its own anchor story; its twin VP-160 is fully propagated

- **Severity:** HIGH
- **Artifact + anchor:** `S-WAVE-A-ENGINE-001-header-scheme-field-rule9-validation-auth-dispatch.md` — frontmatter `verification_properties:`, `§Token Budget Estimate`, `§Tasks → Merge gate`
- **Source-of-truth checked against:** `VP-INDEX.md` §Properties registry row for VP-161; `vp-161-rule9-error-message-echo-cap-and-ctl-escaping.md` §Source Contract; `verification-architecture.md` §Provable Properties Catalog
- **LIVE-vs-changelog occurrence counts:** VP-161 occurrences in the story = **0** (LIVE and changelog/Version-History alike). VP-160 occurrences = **6** (frontmatter array + 2 justification comments + §Token Budget row + 3 in §Merge gate). VP-153 occurrences = 30+.
- **Why it is a defect:** Three upstream artifacts independently register VP-161 with `Anchor Story = S-WAVE-A-ENGINE-001`, `priority: P0`, `proof_method: kani`, `module: prism-spec-engine`. VP-161's §Source Contract states the anchor justification in detail (`§Tasks T-B02` Steps 1–2; AC-024/RG-028; AC-025/RG-029) and I confirmed all five of those story anchors resolve. The story therefore **does** build the mechanisms — but it carries no VP-161 obligation anywhere: not in `verification_properties:`, not in the Token Budget (which lists VP-153 and VP-160), and not in the Merge gate (which carries a VP-160 entry). VP-161 was created in FB62 explicitly as "successor to VP-160 scope note deferral" — i.e. a documented **twin pair** created by one split, same BC, same anchor story, same tool, same priority. Only one half of the pair reached the tail of the chain. This is the exact rule-survives-head/dies-in-tail class: a P0 formal-verification property with zero story-side signal that its Kani harness is owed. POL-4 (semantic_anchoring_integrity), POL-9 (VP-INDEX propagation), POL-29 dimension 9a (sibling pair — "absence of the string is the failure mode, not evidence of cleanliness").
- **Routing:** `story-writer` (add VP-161 to `verification_properties:`, §Token Budget row, and a §Merge gate entry mirroring the VP-160 entry), then `state-manager` for the index/version sync.

---

### F-WASE-P72B-HIGH-002 — SAP-2 wire-parity treatment landed on 2 of 4 sibling sensor field-mapping BCs; Claroty and CrowdStrike untouched

- **Severity:** HIGH (blast radius 2 files)
- **Artifacts + anchors:** `BC-2.02.005-claroty-field-mapping.md` (whole body); `BC-2.02.003-crowdstrike-field-mapping.md`; compared against `BC-2.02.004-cyberint-field-mapping.md` §Postconditions / §Edge Cases EC-02-006b/EC-02-006c / §Canonical Test Vectors / §TOML Contract, and `BC-2.02.006-armis-field-mapping.md` §Generated-Records Path Coverage / §TOML Contract
- **Source-of-truth checked against:** `crates/prism-dtu-claroty/src/types.rs` (`ClarotyDevice`, `ClarotyAlert`); `crates/prism-sensors/specs/claroty.sensor.toml`; `BC-INDEX.md` registry rows for BC-2.02.003/004/005/006
- **LIVE occurrence counts:** In BC-2.02.005 the strings `SAP-2`, `deliberate`, `exclusion`, `excluded`, `mitre_technique`, `iot_devices_count` occur **0 times** (LIVE and changelog). BC-2.02.004 carries an explicit SAP-2 deliberate-exclusion sentinel edge case plus a §TOML Contract; BC-2.02.006 carries a §Generated-Records Path Coverage section plus a §TOML Contract with 7 anchored MUSTs.
- **Why it is a defect:** `claroty.sensor.toml` declares **6** columns on the `devices` table (`uid, asset_id, device_category, device_type, risk_score, retired`) against a `ClarotyDevice` struct carrying ~17 public fields (`assignees, device_subcategory, device_type_family, ip_list, labels, mac_list, model, network_list, os_category, vlan_list, tags`, …), and **8** columns on `alerts` against a `ClarotyAlert` struct carrying 16 (`iot_devices_count, it_devices_count, medical_devices_count, unresolved_devices_count, mitre_technique_enterprise_ids, mitre_technique_enterprise_names, mitre_technique_ics_ids, mitre_technique_ics_names` all omitted). Under SAP-2 Rule 4 each of these is a MEDIUM "field in DTU with no TOML column," and the rule states explicitly that **an undocumented deliberate exclusion causes this finding class to recur on every subsequent pass**. The owning BC documents none of them. Per BC-INDEX, the FB68 burst closed F-SAP2-MED-004 (BC-2.02.004) and F-SAP2-MED-005 (BC-2.02.006) *in the same burst*; BC-2.02.005 last advanced for an unrelated `scheduled_amendment_in` cleanup in FB38, as did BC-2.02.003. The sibling pair in the same architectural layer (`BC-2.02.00x` sensor→OCSF field mappings, same subsystem SS-02, same capability CAP-003) was not swept. POL-29 dimension 9a; Partial-Fix Regression Discipline (b).
- **Routing:** `product-owner` (author the SAP-2 exclusion sentinel + per-field coverage decision for BC-2.02.005 and BC-2.02.003, with POL-29 9c story/AC/RGT anchors where a column is contracted for addition).

---

### F-WASE-P72B-HIGH-003 — SAC-2 `anchor_stories:` key absent from 15 of 54 ADRs; FB62 swept only ADR-050..056

- **Severity:** HIGH (blast radius 15 files)
- **Artifact + anchor:** `specs/architecture/decisions/` — frontmatter of ADR-004, ADR-005, ADR-006, ADR-007, ADR-008, ADR-009, ADR-010, ADR-011, ADR-012, ADR-014, ADR-032, ADR-034, ADR-035, ADR-036, ADR-038
- **Source-of-truth checked against:** `CLAUDE.md` §SAC-2 ("The key MUST be **present** — a missing key is a defect, distinct from an empty one"); `document_type:` frontmatter of every file in the decisions directory
- **Derived classification (axis 8 — recomputed, see §Derived Counts):** 59 files in the directory; 39 carry `^anchor_stories:`; 20 do not. Of the 20 non-matching: 3 are `document_type: architecture-section` (ADR-001, ADR-002, ADR-003), 1 is `document_type: adr-amendment` (ADR-026-AMENDMENT), 1 is `document_type: hook-specifications` (hook-specs-bundle-a). **Remaining 15 are `document_type: adr`.** Total `document_type: adr` files = 54. All 8 perimeter ADRs (026, 028, 051–056) plus ADR-057 DO carry the key.
- **Why it is a defect:** SAC-2 is unconditional ("Every ADR MUST carry an `anchor_stories` frontmatter key") and carries no scoping clause, so there is no intent ambiguity to adjudicate. The FB62 remediation cited in the SAC-2 precedent populated ADR-050..056 only; the same-layer siblings were not swept. Without the key, ADR→story traceability is unidirectional for 15 ADRs and an ADR change cannot be swept to its dependents — the mechanism SAC-2 exists to close.
- **Note:** the *process* half of this gap is already tracked as `S-MAINT-ADR-ANCHOR-GATE-001` (draft, confirmed present on disk). The *content* half — the 15 files — is not closed by that story existing.
- **Routing:** `architect` (populate from `§Authority` ground truth per SAC-2 clause 2; `[]` only with a verified-empty annotation per clause 3).

---

### F-WASE-P72B-MED-001 — ADR-056 §D9 issues a `MUST NOT` against a code entity that does not exist, and restates a count that is stale by 4

- **Severity:** MEDIUM
- **Artifact + anchor:** `ADR-056-page-number-pagination-variant.md` §D9 — heading text and the bolded `MUST NOT` block
- **Source-of-truth checked against:** `scripts/check-non-exhaustive.sh` (header comment "EXPECTED count in this script is derived from that manifest automatically — do not edit EXPECTED here"; `EXPECTED` assigned from `check-non-exhaustive-per-symbol.py --count`); `scripts/check-non-exhaustive-per-symbol.py` (`EXPECTED_COUNT = len(EXPECTED_SYMBOLS)`; `EXPECTED_SYMBOLS` list holds **96** entries); `CLAUDE.md` §Conventions §non_exhaustive discipline
- **LIVE-vs-changelog occurrence counts:** 3 LIVE occurrences of the stale count in ADR-056 §D9 (heading "Unchanged at 92"; body "expected symbol count 92"; `MUST NOT` block "`EXPECTED=92`"). §Changelog rows not counted.
- **Why it is a defect:** ADR-056 is `status: accepted` and §D9 is a live normative directive to `S-WAVE-A-CYBERINT-SPEC-001` (still `status: draft`). It commands the implementer not to bump a literal `EXPECTED=92` in `scripts/check-non-exhaustive.sh` — that literal no longer exists; the script derives the value from the Python manifest and its header explicitly forbids editing it there. POL-22 Phase C (named-entity existence verification, HIGH scope). Separately, the count is now 96, and `CLAUDE.md` §Conventions states the manifest is the single source of truth and "Do NOT restate the count in prose anywhere." §D9's *substantive* claim (adding `PaginationConfig::PageNumber` and `PaginationType::Page` adds no new symbol) remains correct — that is why this is MEDIUM and not HIGH — but the mechanism it names is falsified, and an implementer following it will search for a string that is not there.
- **Routing:** `architect` (rewrite §D9 to cite `EXPECTED_SYMBOLS` in `check-non-exhaustive-per-symbol.py` as the sole registry, drop all three count restatements per the CLAUDE.md convention).

---

### F-WASE-P72B-MED-002 — Claroty `alerts.id` declared `string` against a `u32` wire field (SAP-2 Rule 2 pairing violation)

- **Severity:** MEDIUM
- **Artifact + anchor:** `crates/prism-sensors/specs/claroty.sensor.toml` — `[[tables]] table_name = "alerts"`, column `id`, `column_type = "string"`
- **Source-of-truth checked against:** `crates/prism-dtu-claroty/src/types.rs` `ClarotyAlert.id: u32`; wire-emission site `routes::alerts` (`Json(json!({"alerts": alerts, ...}))` — full struct serialization, so `id` reaches the wire as a JSON **number**); extraction site `build_column_array` `ColumnType::String` arm in `crates/prism-bin/src/spec_driven_adapter.rs`
- **Why it is a defect:** SAP-2 Rule 2's pairing table is `String↔String, Integer↔i64/u64, Float↔f64, Boolean↔bool`. `u32` pairs with `Integer`, not `String`. I traced the runtime consequence rather than asserting data loss: the `ColumnType::String` arm's fall-through (`other => Some(other.to_string())`) stringifies the number, so the value is **preserved** — this is why I rate MEDIUM, not P1 CRITICAL. The real harm is downstream typing: `claroty_alerts.id` materializes as Arrow `Utf8`, so PrismQL `WHERE id > 1000` and `ORDER BY id` become lexicographic (`"10"` sorts before `"9"`), producing silently wrong results for an LLM agent that reasonably treats a numeric alert ID as numeric. Note the sibling `alerts.devices_count` is correctly declared `integer` against `devices_count: u32` — so the file is internally inconsistent about the same Rust type.
- **Routing:** `product-owner` (adjudicate `integer` vs documented-string-by-design in BC-2.02.005 — ties to F-WASE-P72B-HIGH-002), then `implementer` for the TOML change with a Red Gate assertion on the Arrow `DataType`.

---

### F-WASE-P72B-MED-003 — POL-23 version-pin sweep not discharged: 2 upstream bumps left stale LIVE pins across 3 Wave-A stories

- **Severity:** MEDIUM
- **Artifacts + anchors:**
  - `S-WAVE-A-ENGINE-001` §Token Budget Estimate, §Tasks RG-030, and one §Architecture-Compliance site — quoted defect text `BC-2.16.009 v1.29` (**3 LIVE occurrences**; 3 further occurrences inside §Version History are exempt)
  - `S-WAVE-A-ENGINE-001` §Tasks RG-032 — quoted defect text `error-taxonomy.md v2.70` (**1 LIVE occurrence**)
  - `S-ADR054-WAVE-A-001` — quoted defect text `BC-2.16.009 v1.27` (1 occurrence)
  - `S-WAVE-A-CYBERINT-SPEC-001` — quoted defect text `BC-2.16.009 v1.28` (1 occurrence)
- **Source-of-truth checked against:** `BC-2.16.009-spec-file-validation.md` frontmatter version; `prd-supplements/error-taxonomy.md` frontmatter version
- **Why it is a defect:** POL-23 is marked **ACTIVE-DURING-TRANSITION** — the version-pin grep step is reinstated and live, and retires ONLY when both S-MAINT-ANTIPIN-SWEEP-001 and S-MAINT-ANTIPIN-SWEEP-002 have merged AND records-lint L11 is deployed. Both sweep stories are still `status: draft` on disk and POL-39 records L11 as proposed, NOT deployed. So the bump-time bidirectional grep obligation is currently binding, and the bumps to BC-2.16.009 and error-taxonomy discharged it against zero of the six LIVE citing sites. **Scope discipline:** the *presence* of narrative version pins in stories is approved tracked debt under `S-MAINT-ANTIPIN-SWEEP-001` ("Remove Narrative Version Pins from .factory/stories/ (83 Files)") and I am **not** filing that. What I am filing is that the pins are now factually **wrong**, which is a live falsehood in a `tdd_mode: strict` story directing a test-writer to load a superseded BC revision.
- **Routing:** `state-manager` (POL-23 discharge on the two bumps: either de-pin the six LIVE sites in-scope, or refresh them and record the grep verdict).

---

### F-WASE-P72B-MED-004 — [process-gap] Three sensor-name-conditional production blocks sit in the one engine file POL-36's crate list omits; a sensor-named `pub fn` lives in an in-scope crate

- **Severity:** MEDIUM
- **Artifacts + anchors:** `crates/prism-bin/src/spec_driven_adapter.rs` — the `fetch` path's query-filter seeding region: `if self.sensor_spec.spec.sensor_id.as_str() == "crowdstrike"` (CrowdStrike FQL time-window injection), `if self.sensor_spec.spec.sensor_id.as_str() == "armis" && (...)` (Armis AQL time-window augmentation), `if self.sensor_spec.spec.sensor_id.as_str() == "crowdstrike"` (CrowdStrike limit push-down). Plus `crates/prism-query/src/pushdown.rs` — `pub fn augment_armis_aql_with_time_window`.
- **Source-of-truth checked against:** `policies.yaml` POL-36 (`generalization_directive_no_sensor_conditional_engine_code`, HIGH): "Sensor-name-conditional control flow in engine code (prism-core, prism-spec-engine, prism-query, prism-sensors, prism-mcp) is FORBIDDEN."
- **LIVE occurrence counts (corpus-wide, recomputed):** production-source matches for `sensor_id == "<sensor>"` across `crates/**/src/**/*.rs` = **3**, all three in `prism-bin/src/spec_driven_adapter.rs`. The 2 further matches in `prism-query` (`engine.rs`, `materialization.rs`) and 1 in `prism-query/src/tests/parser_tests.rs` are test assertions, not control flow — checked and not counted.
- **Why it is a defect:** POL-36's enumerated scope does not include `prism-bin`, so under a strict reading these three blocks are legal — and that is precisely the problem. `spec_driven_adapter.rs` **is** the engine's spec-driven materialization boundary: it is the file the catalog scope statement in `BC-2.16.002 §Postconditions` names as the "PRIMARY production insertion point — the spec-driven adapter materialization path through which DataFusion receives and queries sensor data." POL-36's directive ("All fixes/features must be GENERAL mechanisms consumed via spec/config; sensor-specific behavior lives only in TOML sensor specs and DTU clones") is therefore unenforceable at the exact site where sensor-specific query synthesis actually happens. Secondarily, `augment_armis_aql_with_time_window` is a `pub fn` whose identifier encodes a sensor name and it lives in `prism-query`, which **is** in POL-36's list — the conditional in prism-bin merely selects it.
- **Why MEDIUM, not HIGH:** a POL-36 violation cannot be asserted without over-claiming, because prism-bin is genuinely outside the policy's written scope. The defect is the scope definition, and the correct disposition (extend POL-36's crate list vs. ratify prism-bin as the sanctioned adapter-wiring exception vs. generalize the three blocks into TOML-declared push-down grammar) requires architect adjudication.
- **Routing:** `architect` (adjudicate POL-36 scope; if extended, route the three blocks + the sensor-named symbol to `implementer` for generalization into spec-declared push-down grammar per ADR-033/ADR-057), then `state-manager` for the `policies.yaml` amendment.

---

### F-WASE-P72B-LOW-001 — Stale positional cite "row 91" survives in BC-2.16.002 LIVE normative text and in production code, against a 90-row catalog

> **TD-VSDD-091 transcription note:** This finding's subject references a positional cite ("row 91") that appears in the adversary's sourced artifact. The cite below is transcribed verbatim from the adversary's report as a description of a defect found in BC-2.16.002 §Postconditions — it is adversary output about a third artifact, not a positional cite authored here.

- **Severity:** LOW
- **Artifacts + anchors:** `BC-2.16.002-multi-step-fetch-pipeline.md` §Postconditions — Canonical Structured Event Catalog scope-statement bullet, parenthetical narrative ("row 91 now lists both PRIMARY and SECONDARY emission sites; catalog count unchanged at 91"). Corroborated in `crates/prism-bin/src/spec_driven_adapter.rs` — comment in the `ColumnType::String` arm's `ocsf.enum_label_unrecognized` emission ("BC-2.16.002 catalog row 91 spec order: sanitize → truncate").
- **Source-of-truth checked against:** the catalog table itself — **90** data rows counted, matching the same bullet's own assertion "The catalog currently contains 90 structured events." The FB84 delta recorded in that bullet is "catalog count 92→90."
- **LIVE-vs-changelog occurrence counts:** 1 LIVE occurrence in the §Postconditions scope bullet; 1 LIVE occurrence in production code comment. Occurrences inside the BC §Changelog and inside `BC-INDEX.md` NOTE/registry ledger text are immutable per POL-1 and not counted.
- **Why it is a defect:** the row the text directs a reader to (row 91) does not exist in a 90-row table, and the event it means (`ocsf.enum_label_unrecognized`) is now the 47th row. This is the TD-VSDD-091 decay class — a positional cite that self-invalidated when the same bullet's own edit removed two rows. It sits in the same sentence that records the removal.
- **Routing:** `spec-steward` (replace both positional cites with the symbol anchor `§Postconditions Canonical Structured Event Catalog — \`ocsf.enum_label_unrecognized\` row`).

---

### F-WASE-P72B-LOW-002 — POL-39 LIVE version pins inside BC-2.16.002 §Postconditions catalog rows (scope-confirmation only)

- **Severity:** LOW (pending scope confirmation against `S-MAINT-ANTIPIN-SWEEP-002`)
- **Artifact + anchor:** `BC-2.16.002-multi-step-fetch-pipeline.md` §Postconditions — Canonical Structured Event Catalog rows for `timestamp.fallback_to_now`, `ocsf.deprecated_class_alias`, `mcp.tool.called`, `push_down.inverted_time_range`, `http_lookup_enrich_failed`, `http_lookup_ssrf_rejected`, `plugin_enrich_json_parse_error`, `plugin_enrich_unexpected_val`, and the `write_tool_registration_after_boot` row
- **Source-of-truth checked against:** POL-39 (`anti_volatile_pin_versions`, HIGH) — exemptions are `## Changelog` rows, `## Version History`, YAML `changelog:` keys, and all lines inside the frontmatter block. A §Postconditions table row is not exempt.
- **LIVE-vs-changelog occurrence counts:** **11 LIVE version-pin occurrences across 8 distinct catalog rows**, plus one non-standard form (`per ADR-026 D7` with a version pin). All occurrences at or below the §Changelog boundary are excluded; the ~20 occurrences inside §Changelog are exempt.
- **Why filed as LOW:** `S-MAINT-ANTIPIN-SWEEP-002` ("specs-version-pin-sweep") is on disk in `draft` and is the tracked vehicle for spec-tier de-pinning. This is filed **only** to confirm these sites are inside that story's enumerated scope — POL-39 is HIGH-severity and BC-2.16.002 is the highest-traffic BC in the corpus. If SWEEP-002's scope list does not name BC-2.16.002 §Postconditions, this promotes to HIGH.
- **Routing:** `story-writer` (confirm/extend `S-MAINT-ANTIPIN-SWEEP-002` scope enumeration to name BC-2.16.002 §Postconditions).

---

### F-WASE-P72B-OBS-001 — [process-gap] SAP-2 Rule 2's phrasing excludes the ADR-028 §D8-B implicit `["iso8601"]` default, making it a false-finding generator for 7+ columns per pass

- **Severity:** OBS (process-gap)
- **Artifact + anchor:** `CLAUDE.md` §SAP-2 Rule 2 — "For datetimes, BOTH of these are valid pairings: `Datetime ↔ chrono DateTime`, OR `Datetime` ↔ an ISO-8601/epoch wire string **carrying a declared `timestamp_formats` parse chain**"
- **Source-of-truth checked against:** `crates/prism-spec-engine/src/pipeline.rs` — `effective_formats` (`if formats.is_empty() { vec!["iso8601"] }`, documented as ADR-028 §D8-B backward compatibility) consumed by `try_formats`, which is the sole parse path for both the primary-value and fallback-chain branches of `normalize_timestamp_fields`
- **Derived counts:** `timestamp_formats` occurrences = **0** in `claroty.sensor.toml`, **0** in `crowdstrike.sensor.toml`, **0** in `armis.sensor.toml`; **2 columns** declare it in `cyberint.sensor.toml`. Datetime columns lacking a declared chain enumerated: Claroty **3** (`alerts.detected_time`, `alerts.updated_time`, `audit_logs.timestamp`), CrowdStrike **4** (`detections.created_timestamp`, `devices.first_seen`, `devices.last_seen`, `incidents.created`). `armis.sensor.toml`'s datetime columns were not enumerated.
- **Why it is a process gap:** Rule 2 was amended 2026-07-27 specifically to stop false findings ("treating chrono as the only valid pairing mints false findings — it would have produced five or more across the Wave-A sensor TOMLs alone"). The amendment closed the chrono arm but introduced a new one: it makes a **declared** chain a precondition, while the implementation supplies `["iso8601"]` when the chain is empty. A literal Rule-2 application therefore mints 7+ false CRITICALs against Claroty and CrowdStrike on every future SAP-2 probe. This pass nearly filed exactly that before reading `effective_formats` — the rule as written points the probe at the wrong verdict.
- **Routing:** human / `orchestrator` (amend `CLAUDE.md` §SAP-2 Rule 2 to add the third valid arm: "…OR an ISO-8601 wire string with `timestamp_formats` **omitted**, which resolves to the implicit `[\"iso8601\"]` default per ADR-028 §D8-B / `effective_formats`").

---

### F-WASE-P72B-OBS-002 — [process-gap] CLAUDE.md restates the `#[non_exhaustive]` gate count in the same sentence that forbids restating it, and the restated value is stale by 4

- **Severity:** OBS (process-gap)
- **Artifact + anchor:** `CLAUDE.md` §Conventions §Highlights — `#[non_exhaustive]` discipline bullet: "92 types currently enforced via the compile-fail gate at `tests/external/non-exhaustive-violation/`" … "Do NOT restate the count in prose anywhere — including in this sentence."
- **Source-of-truth checked against:** `scripts/check-non-exhaustive-per-symbol.py` — `EXPECTED_SYMBOLS` holds **96** entries (95 unique; the file's own comment notes `ColumnType` appears twice and is deduplicated for the per-symbol layer), and `EXPECTED_COUNT = len(EXPECTED_SYMBOLS)`
- **Why it is a process gap:** the bullet is self-contradicting and the contradiction is load-bearing — it is the upstream source for the stale count in ADR-056 §D9 (F-WASE-P72B-MED-001). The 2026-07-27 collapse-to-one-source-of-truth change updated the mechanism but left the number in the governance doc.
- **Routing:** human (CLAUDE.md is human-owned per §Pipeline Authority; the orchestrator may surface but not edit).

---

## Probe Verdicts

| Probe | Verdict | Evidence |
|---|---|---|
| **SAP-1** (event-catalog completeness, corpus sweep) | **PASS** | Full corpus grep of `event_type\s*=\s*"…"` across `crates/`. **88** distinct values surfaced. Two were **comment-only false positives**, discarded after reading the sites: `timestamp_parse_failure` (a `pipeline.rs` comment *documenting the F-LP2-HIGH-001 removal* of that emission, with `?`-propagation rationale per D-765 / SAP-1 Rule 5) and one `infusion.coercion_failed` occurrence in a `prism-core/src/error.rs` **doc comment** on the `E-INFUSE-014` variant. Of the remaining 86 real production values: 84 are catalog-registered; `credential_access` is out of scope (BC-2.03.010, per the catalog's own exclusion clause) and `boot.audit.initialized` is excluded at event granularity (BC-2.05.012). **Zero unregistered production emissions. Zero orphan catalog rows** — all 87 distinct catalog names have at least one live production emission site. Test-file occurrences (`pipeline_http_integration.rs`, `table_registry_tests.rs`, `test_adapter_normalization.rs`, `bc_2_02_012_class_selector.rs`, `plugin_boot_tests.rs`, and a literal `"..."` in `bc_2_10_006_mcp_stdout_purity.rs`) are assertions, not emissions. |
| **SAP-2** (DTU↔TOML parity) | **PASS with 1 MEDIUM** (MED-002) + 1 HIGH on the spec-documentation layer (HIGH-002) | **Cyberint:** all 15 TOML columns pair with `Alert`/`Ioc`/`AlertData` fields. Rule 6 dual-path checked: the seeded/scenario paths emit generated records verbatim; the **static-fixture path hand-builds an 8-key `json!` envelope in `get_alerts` that omits `ioc`/`iocs`/`alert_data`**, so 8 IOC/network columns resolve to nothing there. **Verified already closed at the spec layer, not a new finding:** `S-WAVE-A-CYBERINT-SPEC-001` §Tasks T-05 item 5 explicitly contracts replacing the `json!` literal with `serde_json::to_value(a)`, with **RG-019 / AC-009** tagged "(F-SAP2-CRIT-001 story leg)" and the file-structure table naming the change. `affected_assets` (DTU field, no TOML column) is documented in BC-2.02.004 §TOML Contract with a real POL-29 9c anchor — confirmed `S-WAVE-A-CYBERINT-SPEC-001` **AC-011 / RG-021** (`test_cyberint_alerts_toml_affected_assets_column_is_json_type`) both exist. **Claroty:** `alerts.id` type mismatch → MED-002; `devices.risk_score`↔`String`, `devices.retired`↔`bool`, `alerts.devices_count`↔`u32` all correct; wire emission is full-struct (`json!({"alerts": alerts, …})`, `json!({"devices": devices, …})`, `json!({"audit_log": entries, …})`) so no field is dropped at the envelope. Undocumented DTU-field exclusions → HIGH-002. **CrowdStrike:** TOML columns and 2-step query/fetch pipelines read consistently; CrowdStrike struct-level pairing **NOT completed** — see coverage disclosure. **Armis `armis_devices`/`armis_alerts` full column sets: NOT REACHED** — see coverage disclosure. Rule 2 datetime arm applied correctly per OBS-001 (no false findings minted). |
| **SAP-3** (spec-arm reachability) | **PASS on the audited story; not exhaustive** | `S-WAVE-A-ENGINE-001` is exemplary: of its 40 Red Gate tests, the great majority carry an explicit `SAP-3: drives SpecLoader::parse() with raw TOML` annotation (parser-surface, not synthetic-struct), **RG-024** is an end-to-end at the `prism-spec-engine` public API (`add_sensor_spec(org_slug, toml_content)`), **RG-040** is the SAP-3 primary end-to-end through `PipelineExecutor::execute` with a wiremock wire-level header assertion, and **RG-030** carries an explicit `DEFENSE-IN-DEPTH note: NOT applicable here` reachability rationale — i.e. the rule-3 rationale-comment obligation is being honored affirmatively. The other 6 stories' RGTs were not audited arm-by-arm. |
| **SAC-1** (Red Gate list / density / ordering) | **PASS — all 7** | Every one of the 7 perimeter stories carries `tdd_mode: strict`, an enumerated `RG-001..RG-NNN` list with named test functions under a `### Red Gate tests (to be written by test-writer BEFORE implementation)` heading that **precedes** the `### Implementation tasks` heading, and a `BC-5.38.001` density-check paragraph. Densities: ENGINE-001 **40** (RG bullets recounted: 40 — matches the stated 40), MCP-001 7, CYBERINT-SPEC-001 16, CYBERINT-PATCH-001 3, ARMIS-REMEDIATION-001 10, ADR054-WAVE-A-001 24, ADR055-WAVE-A-001 11. All seven are `status: draft`, so the pre-`status: ready` obligation is not yet due in any case — but it is satisfied anyway. The FB61 remediation cited in the SAC-1 precedent propagated cleanly to all six previously-deficient stories. |
| **SAC-2** (ADR `anchor_stories`) | **FAIL** — HIGH-003 | 15 of 54 `document_type: adr` files lack the key. All perimeter ADRs pass. |
| **POL-31** (VP proof-harness symbol validation) | **PASS with a caveat, no finding filed** | VP-161 Harness 1 target `truncate_at_char_boundary` **exists** — `pub(crate) fn truncate_at_char_boundary(s: &str, max_chars: usize) -> &str` in `prism-spec-engine::validation` — matching VP-161's "CONFIRMED REAL" label; Harness 2 target is correctly marked `[PLANNED: escape_ctl_bytes_for_error_message]` with an explicit SYMBOL RESOLUTION block, AC-025/RG-029 anchor obligation, and Phase-5 resolution path. **Caveat deliberately not filed separately:** VP-160's §Proof Harness Skeleton cites plain target `prism_spec_engine::spec_parser::is_valid_cookie_name_tchar` with **no** `[PLANNED]` marker and no SYMBOL RESOLUTION block, and that symbol has **0 occurrences** anywhere in `crates/`. Under a literal POL-31 reading that is a MEDIUM sibling-asymmetry (VP-161 FB62 added exactly this marker for its own provisional symbol; the twin did not receive it). Not filed as a separate finding because it is subsumed by HIGH-001's routing — the VP-160/VP-161 pair needs one coordinated sweep, and splitting it would double-count the same POL-29 9a failure. Flagged here so the fixer sweeps both halves. |

---

## Derived Counts (axes 1–8) — recomputed from source

**Axis 1 — SAP-1 corpus event-catalog sweep**
- Catalog rows in `BC-2.16.002 §Postconditions`: **90** (table data rows counted). Distinct `event_type` names: **87** (three names appear twice by design — `auth_initial_acquired`, `auth_initial_acquired_empty`, `auth_initial_failed` each have a `PipelineExecutor::execute` row and a `PipelineExecutor::execute_step` row with differing field schemas). The bullet's own claim "contains 90 structured events" is internally consistent with the row count.
- Distinct `event_type` values from corpus grep: **88** raw → **86** real production values after discarding 2 comment-only false positives → **84** in-scope after 2 documented exclusions.
- Unregistered production emissions: **0**. Orphan catalog rows: **0**.

**Axis 2 — VP-INDEX arithmetic (all three numbers)**
- Frontmatter `total_vps: 161`; `active_vps: 148`; `retired_vps: 13` → 148 + 13 = **161** ✓
- §Summary per-tool: Kani 32 + Proptest 88 + Unit test 6 + Fuzz 6 + Integration test 29 = **161** ✓; P0 25+66+4+5+24 = **124** ✓; P1 7+22+2+1+5 = **37** ✓; 124+37 = **161** ✓
- Actual registry rows: 174 total `^| VP-NNN |` matches − **13** ADR-037 retirement-table rows (confirmed exactly 13 rows matching `^| VP-(095..107) | BC-3.3.`) = **161** ✓
- **All three figures agree at 161.** VP-PLUGIN-001..007 named-alias rows do not match the numeric pattern and correctly do not add to the count.

**Axis 3 — verification-coverage-matrix column sums (summed across all 17 module rows)**
- Kani: 12+5+4+0+4+4+0+0+1+1+0+0+1+0+0+0+0 = **32** ✓
- Proptest: 8+1+3+2+17+15+10+7+2+3+0+3+11+4+1+1+0 = **88** ✓
- Unit test: **6** ✓ · Fuzz: **6** ✓ · Integration test: 0+0+0+0+0+11+2+1+0+1+2+0+2+8+2+0+0 = **29** ✓
- Stated `**Totals**` row (32/88/6/6/29/161) and `**Total VPs**` row (161/124/37/13/148) both reconcile.
- Perimeter VPs cross-checked in both architecture anchors: VP-153 (proptest, prism-spec-engine), VP-159 (integration_test), VP-160, VP-161 (kani) all present in `verification-coverage-matrix.md` §Coverage by Module under `prism-spec-engine` **and** in `verification-architecture.md` §Provable Properties Catalog with matching module/method/priority. The prism-spec-engine row's per-tool counts reconcile against its own VP-ID list (kani 4, proptest 7+8 retired = 15, unit_test 4, fuzz 1, integration 10+1 retired = 11). TIER1 Mermaid node enumerates exactly **32** kani VP IDs. **No POL-9 drift found.**

**Axis 4 — Invariant-to-BC orphan detection**
- DI IDs declared in `invariants.md`: **34** (DI-001..DI-034). **4 are explicitly REMOVED/struck** (`~~DI-009~~`, `~~DI-010~~`, `~~DI-011~~`, `~~DI-013~~` — ephemeral-cursor retirements). **Live DI count: 30.**
- BC citations found for **all 30** live DIs. **Zero orphan invariants.**
- **Honest limitation:** verification was file-level presence of each `DI-NNN` in at least one real BC file, NOT that the citation sits specifically in that BC's §Traceability *L2 Invariants* field. POL-2's `verification_steps` require the field-level check; it was not performed. Some hits landed only in `BC-INDEX.md` or `SUBSYSTEMS-*-SUMMARY.md` alongside a real BC hit — in every such case at least one genuine BC file also matched.
- Minor note (not filed): the invariants table row order is non-monotonic (DI-027 appears after DI-032; DI-033 sits in a separate later section). POL-32 governs §Changelog tables, not invariant registries, so this is not a violation.

**Axis 5 — BC H1 ↔ BC-INDEX title sync**
**8 of 8 match verbatim.** BC-2.16.002 "Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation"; BC-2.16.008 "`add_sensor_spec` MCP Tool — Upload a New Sensor Spec at Runtime"; BC-2.16.009 "Spec File Validation — Schema Validation, Variable Reference Resolution, OCSF Field Validation"; BC-2.16.014 "Declarative Auth Acquisition Token Lifecycle"; BC-2.01.016 "SensorAuth Open Trait — Plugin-Implementable Auth Contract (No Sealed Marker)"; BC-2.01.017 "StaticCookieAuthProvider Contract — No-Login-Roundtrip Cookie Injection"; BC-2.02.004 "Cyberint Alert Field Mapping to OCSF"; BC-2.02.006 "Armis Centrix Field Mapping to OCSF (7 Data Sources)". POL-7 clean. Incidental L10 check: BC-2.16.002 frontmatter version matches its BC-INDEX pin.

**Axis 6 — Story frontmatter-body coherence**
Frontmatter `behavioral_contracts:` arrays read: MCP-001 [BC-2.16.008, BC-2.10.007]; CYBERINT-SPEC-001 [BC-2.01.006, BC-2.01.018, BC-2.06.003, BC-2.16.001, BC-2.16.002, BC-2.16.009]; CYBERINT-PATCH-001 [BC-2.16.009]; ARMIS-REMEDIATION-001 [BC-2.01.008, BC-2.06.003]; ADR054-WAVE-A-001 [BC-2.16.009, BC-2.01.017, BC-2.06.003]; ADR055-WAVE-A-001 [BC-2.16.009, BC-2.16.001, BC-2.16.007, BC-2.16.008, BC-2.16.002]; ENGINE-001 [BC-2.16.009, BC-2.01.017, BC-2.16.014, BC-2.01.016]. All 7 carry `status: draft` and `tdd_mode: strict`. **Verified for ENGINE-001 only:** §Behavioral Contracts table present, all four BCs represented in the §Token Budget table with per-BC relevance descriptions, and BC-titles consistent with the H1s confirmed in axis 5. **The per-BC AC-trace-line audit (POL-8 directions 2 and 3) and the STORY-INDEX status/version cross-check were NOT completed for any of the 7 stories** — see coverage disclosure.

**Axis 7 — SAC-1:** see Probe Verdicts. 7/7 pass.

**Axis 8 — SAC-2 corpus classification:** 59 files in `specs/architecture/decisions/`; 39 carry `^anchor_stories:`; 20 do not, classified as 15 × `adr`, 3 × `architecture-section`, 1 × `adr-amendment`, 1 × `hook-specifications`. `document_type: adr` total = 54. **Missing-key count among true ADRs = 15.**

---

## Counts NOT Verified (do not treat as established)

- **Workspace test count.** Tests were not run or counted. Any "5,483 tests" style figure elsewhere is unverified here.
- **`active_contracts` / `draft_contracts` / `total_contracts`** — quoted from BC-INDEX NOTE lines; **not recomputed** against the registry.
- **`total_vps`, `active_vps`, `retired_vps` semantics.** The three numbers are verified mutually consistent and matching the actual row count. NOT verified: that each row's `Status` cell actually reads retired/active in a way that produces 148/13; the 13 retirement-table rows were confirmed by count only.
- **`EXPECTED_SYMBOLS` = 96.** Derived from a count of lines matching a 4-space-indent quoted-string pattern in `check-non-exhaustive-per-symbol.py`. `--count` was not executed. The file's own comment implies 95 *unique* symbols (ColumnType duplicated). Treat 96 as "list-entry count," not as an executed value.
- **"E0639 struct literal violations (69 total)"** — an inline comment in the manifest; not recomputed.
- **ADR-056 §D9's substantive claim** that `PaginationConfig` and `PaginationType` already appear in both scripts' registries — **not verified** against `EXPECTED_SYMBOLS`.
- **Per-module VP-ID list completeness** in `verification-coverage-matrix.md` — verified only for the `prism-spec-engine` row; the other 16 rows' counts were summed but their ID lists were not cross-tallied.
- **Claroty/CrowdStrike DTU field totals.** "~17 `ClarotyDevice` fields" and "16 `ClarotyAlert` fields" are read off a paginated struct grep, not an exhaustive field enumeration. Treat as approximate; the *specific* omitted field names listed are individually grounded.
- **`prism-query/src/pushdown.rs` 43 occurrences of `armis|crowdstrike`** — a raw match count spanning comments, tests, and code. Only `augment_armis_aql_with_time_window` is verified as a sensor-named `pub fn` there. Do **not** treat 43 as a POL-36 violation count.
- **Whether `S-MAINT-ANTIPIN-SWEEP-002`'s scope enumeration names BC-2.16.002.** Unverified — the open question in LOW-002.
- **BC-INDEX row-level version pins** for BC-2.16.008/009/014, BC-2.01.016/017, BC-2.02.006. The L10 match was confirmed only for BC-2.16.002; BC-2.16.009/014/2.01.016/2.01.017 frontmatter versions were read for the MED-003 staleness check.

---

## Coverage Disclosure — perimeter items NOT reached

Capacity ran out before the perimeter was complete. The following were **not read** or **not completed**, and no finding in this report should be read as clearing them:

**BC bodies not read:** `BC-2.16.008` (full body — only the BC-INDEX row and H1), `BC-2.16.014` (only frontmatter version + BC-INDEX row), `BC-2.01.016`, `BC-2.01.017` (both: only frontmatter version, H1, BC-INDEX row), `BC-2.02.006` (only the BC-INDEX row ledger, which is extensive). `BC-2.16.009` body **not read** — only its frontmatter version and H1. `BC-2.16.002`: read §Description, §Preconditions, the first ~15 §Postconditions bullets, and the entire Canonical Structured Event Catalog. **Not read:** §Variable Scope and Lifetime, §Fan-Out Behavior, §Invariants, §Error Conditions, §Edge Cases, §Canonical Test Vectors, §Verification Properties, §Traceability. `BC-2.02.004`: read §Edge Cases/§TVs/§TOML Contract/§Changelog region only.

**ADR bodies not read:** **ADR-051, ADR-052, ADR-055** — frontmatter only (`supersedes`/`superseded_by`/`anchor_stories` presence/`status`/`version`). **ADR-053, ADR-054** — frontmatter only. **ADR-026, ADR-028** — frontmatter plus §D-section heading inventories (used to verify the supersession pair resolves). **ADR-056** — §heading inventory + §D9 body only; §D1–D8, §D10, §Rationale, §Consequences not read.

**Supersession-chain audit — partial.** Two pairs verified bidirectionally coherent with cited sections resolving: ADR-026 `superseded_by: ["ADR-028 §D2 …"]` ↔ ADR-028 `supersedes: ["ADR-026 §D3 …"]` (each names the *other* document's section — coherent, not asymmetric; ADR-028 §D2 "auth_type Grounding Rule" and ADR-026 §D3 "Runtime cross-sensor auth-composition prevention" both exist), and ADR-044 `superseded_by: "ADR-052 (§D4 only …)"` ↔ ADR-052 `supersedes: "ADR-044 §D4"`. **NOT audited: whether any superseded ADR still carries LIVE normative text reading as current without a supersession marker** — that requires the ADR bodies not reached. ADR-051, ADR-053, ADR-054, ADR-055, ADR-056 all declare `superseded_by: null`; ADR-053 declares a populated `supersedes:` block that was not expanded.

**VP bodies:** VP-161 read in full. VP-160 read via targeted greps (frontmatter + all `is_valid_cookie_name_tchar`/`PLANNED`/`CONFIRMED` sites) — §Property Statement, §Feasibility, §Proof Method not read in full. **VP-153 and VP-159 bodies NOT read at all** (registry rows only). POL-31 symbol validation for VP-153 and VP-159 is therefore **not performed**.

**`invariants.md`:** the full DI ID set was extracted and DI-009..DI-014 rows read. The remaining ~28 invariant bodies, enforcement-site columns, and violation-response columns were **not read** — so no semantic-anchoring check was run on them.

**Story bodies:** `S-WAVE-A-ENGINE-001` substantially covered (frontmatter, §heading inventory, §Token Budget, the full 40-item Red Gate list, density paragraph, start of implementation tasks). §Authority, §Narrative, §Behavioral Contracts table contents, all 7 AC Tiers, §Coherence Matrix, §Architecture Mapping, §Edge Cases, §Merge gate, §Previous Story Intelligence, §Architecture Compliance Rules, §File Structure, §ADR-054 §D11 rows: **not read.** The other **6 stories were only grepped** for frontmatter, Red Gate headings, density rows, and specific anchors — **no body was read.** Axis 6 directions 2/3 (AC-trace ↔ frontmatter bidirectionality) and POL-13 (STORY-INDEX status agreement) are **unperformed for all 7**.

**SAP-2 not reached:** (a) **`armis_devices` and `armis_alerts` full column sets** — the single largest assigned SAP-2 item, not started. `armis.sensor.toml` was touched only to count `timestamp_formats` (0). Given BC-2.02.006 advanced through FB85/FB89/FB92/FB93/FB95/FB100 with §Generated-Records Path Coverage work, this surface is likely converged, but there is **no evidence** for that here. (b) **CrowdStrike struct-level pairing** — `crowdstrike.sensor.toml`'s full column/step inventory was read but the crate has no `types.rs`; response types live in `routes/detections.rs`, `routes/hosts.rs`, `generator.rs`, which were not read. The 11 detections columns (incl. 5 `behaviors_*` `source_path` columns), 6 devices columns, and 4 incidents columns are **unverified against any DTU field or emission site.** (c) Claroty `audit_logs` entry type — the `json!({"audit_log": entries, …})` envelope was confirmed but the `entries` element type was not located, so `id/action/actor/timestamp/resource` are unverified. (d) Claroty `generator.rs` generated-records path was **not** read (Rule 6 dual-path incomplete for Claroty).

**Supporting docs:** `error-taxonomy.md` — frontmatter version only; **no E-SPEC-027 / E-SPEC-012 / E-SPEC-013 template-text comparison performed**, so POL-24 (`error_message_template_verbatim`) is **unrun** on this perimeter despite the ENGINE-001 Red Gate list asserting byte-verbatim messages at ~15 sites. `ARCH-INDEX.md` and `STORY-INDEX.md` were not opened except incidentally via corpus greps — **POL-6 (subsystem-name source-of-truth) and POL-13 are unrun.**

---

## Novelty Assessment

**Novelty: HIGH.**

The two highest-severity findings could not have been produced by per-artifact review:

- **F-WASE-P72B-HIGH-001** required walking VP-161 through four hops (VP-INDEX row → VP-161 §Source Contract → verification-architecture catalog → story frontmatter/body) and then noticing that the *twin* completed the walk while VP-161 stopped one hop short. Every individual artifact is internally correct: VP-161's anchor justification is precise and all five of its story anchors resolve. Only the reverse direction is broken. An anchor-resolution check passes here.
- **F-WASE-P72B-HIGH-002** required reading two DTU structs plus a TOML plus **four** sibling BCs and then reading the BC-INDEX ledger to establish that FB68 closed two of four in one burst. BC-2.02.005 read on its own looks fine — it simply says nothing.
- **F-WASE-P72B-MED-001** is the "citation resolves but is semantically wrong" class in its purest form: `scripts/check-non-exhaustive.sh` exists, so a file-existence check passes; only reading what the script *says* ("do not edit EXPECTED here") reveals the ADR is commanding compliance with a mechanism that was deliberately deleted.
- **F-WASE-P72B-OBS-001** is a meta-finding about the probe itself. The 2026-07-27 SAP-2 Rule 2 amendment was written to prevent false findings and instead relocated them. This pass nearly filed 7 false CRITICALs against Claroty and CrowdStrike before reading `effective_formats`. Any future pass applying Rule 2 literally will file them.

**What is genuinely converged** (reported as clean rather than re-litigated): the entire VP-INDEX ↔ architecture-anchor arithmetic chain (all three axes reconcile exactly, including per-module ID-list tallies for the perimeter crate); the 90-row structured event catalog against an 88-value corpus sweep; POL-7 title sync at 8/8; POL-2 at 30/30 live invariants; SAC-1 at 7/7 including a 40-RGT enumeration whose density paragraph was recounted and confirmed. The FB61 (SAC-1) remediation propagated to all six deficient stories without a sibling miss — a clean POL-29 discharge, worth recording as such.

**Where the residual risk concentrates:** the two remaining defect clusters are both sibling-sweep failures where a burst fixed the artifact it was dispatched against and stopped at the perimeter boundary — VP-160 but not VP-161's story leg; BC-2.02.004 and BC-2.02.006 but not BC-2.02.003 and BC-2.02.005; ADR-050..056 but not the other 15 ADRs. Three independent instances of the *same* POL-29 dimension-9a pattern in one pass meets the TD-VSDD-097 3-recurrence threshold on its own terms, and the dimension-9a clause ("absence of the string is the failure mode, not evidence of cleanliness") is precisely what a string-grep discharge misses in all three. That convergence is the signal, not the three findings individually.
