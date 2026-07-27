---
document_type: adversarial-review
review_id: wave-a-sap2-probe-pass-65
probe_type: SAP-2
perimeter: wave-a-spec-evolution
mode: standalone DTU-TOML parity probe
verdict: SAP-2 FAILED
findings_count: 9
severity_breakdown:
  critical: 2
  high: 1
  medium: 6
  low: 0
  observation: 3
  process_gap: 1
related_state_decision: D-2043
related_pass: local-pass-65
date: 2026-07-27
open_findings: [SAP2-CRIT-001, SAP2-CRIT-002, SAP2-HIGH-001, SAP2-MED-001, SAP2-MED-002, SAP2-MED-003, SAP2-MED-004, SAP2-MED-005, SAP2-MED-006]
version: "1.0"
changelog:
  - version: "1.0"
    date: 2026-07-27
    author: state-manager
    note: "Initial persistence of standalone SAP-2 DTU-TOML parity probe (FB63). Probe dispatched per human direction to close pass 65's SAP-2 NOT-REACHED gap before fix cascade."
---

# SAP-2 Probe — DTU↔TOML Schema Parity (Wave-A Spec-Evolution)

**Verdict: SAP-2 FAILED — 2 P1 CRITICAL, 1 HIGH, 6 MEDIUM, 3 OBSERVATION. All 9 counting findings OPEN.**

---

## Headline Methodological Finding

**Struct-level parity is not wire-level parity.**

The Cyberint list-alerts handler does not serialize the `Alert` struct — it hand-builds a `serde_json::json!` envelope enumerating exactly eight keys (`alert_id`, `title`, `severity`, `status`, `created_at`, `source`, `type`, `affected_assets`). Verifying only `prism-dtu-cyberint::types` produces a FALSE PASS.

The Cyberint TOML carries the comment: `# SAP-2 compliance: all columns have matching fields in prism-dtu-cyberint/src/types.rs` — a claim simultaneously true (struct-level) and false (wire-level). This is the exact scenario SAP-2 rule 5 ("read the Rust") was designed to catch, but the probe's type-mapping rule must be read as "read the *emission site*, not just the type definition."

Additionally, `prism-dtu-cyberint::routes` registers only `alerts`, `dtu`, and `threats` — no `Asset` struct exists anywhere in the crate, making all 11 Cyberint assets columns unbacked.

---

## Critical Findings

### F-SAP2-CRIT-001 — Eight IOC columns resolve to nothing on the DTU static-fixture path; the primary demo enrichment input field is silently empty [OPEN — implementer, story-writer]

**Artifacts:** `prism-dtu-cyberint` list-alerts handler `json!` envelope (8-key literal); `cyberint-alerts.sensor.toml` §columns block (ioc and alert_data column group)

**Defect.** Eight IOC columns (`$.ioc.type`, `$.ioc.value`, `$.iocs[*].type`, `$.iocs[*].value`, `$.iocs[0].value`, `$.alert_data.ip`, `$.alert_data.domain`, `$.alert_data.url`) resolve to nothing on the DTU static-fixture path, because the alerts handler's `json!` literal includes only the eight top-level keys named above — none of which is a nested IOC field. Notably:

- `iocs_value` is annotated in the TOML itself as "THIS IS THE PRIMARY DEMO COLUMN used in pivot_enrich"
- `iocs_value_first` is the declared `input_field` for typed enrichment

The IOC keys DO exist on the `generated_records` generator paths, creating path-dependence: seeded demo scenarios pass, unseeded production runs silently yield nothing.

The TOML false-assurance comment (`# SAP-2 compliance: all columns have matching fields in prism-dtu-cyberint/src/types.rs`) is simultaneously true of the `Alert` struct and false of the wire. Story §Tasks T-05 re-authors this handler's envelope but specifies only top-level keys, so an implementer following T-05 will carry the eight-key record literal forward.

**Non-exhaustive note (not a separate finding):** Adding the IOC nested-field columns does not affect the `#[non_exhaustive]` three-site count (EXPECTED=92) since this change is to a TOML spec file and a route handler, not to a public API struct.

**Routing:** implementer (alerts handler `json!` envelope + TOML `# SAP-2 compliance` comment correction); story-writer (T-05 per-record nested key requirement + wire-shape assertion per CLAUDE.md §Wire-shape assertion discipline). **OPEN.**

---

### F-SAP2-CRIT-002 — All 11 Cyberint assets columns are unbacked; no `Asset` struct exists and no assets route is registered [OPEN — story-writer, implementer]

**Artifacts:** `cyberint-assets.sensor.toml` §tables block (11 columns); `prism-dtu-cyberint::routes` module registration; `S-WAVE-A-CYBERINT-SPEC-001` §Tasks T-04, T-08, §File Structure Requirements, §Acceptance Criteria

**Defect.** `prism-dtu-cyberint::routes` registers only `alerts`, `dtu`, and `threats`. No `Asset` struct exists anywhere in the crate. All 11 assets columns (`id`, `name`, `type`, `ip_addresses`, `mac_address`, `os`, `os_version`, `site`, `risk_score`, `discovery_precision`, `vulnerabilities`) are unbacked.

The story's own T-03 gate ("Missing-column-in-DTU = P1 CRITICAL per CLAUDE.md §Conventions SAP-2") is not satisfiable, because T-03 defers the DTU types dependency to T-04, which is scoped to `clone.rs` and alert route paths only (no `Asset` struct). T-08 is the only task that would create the assets route, but: T-08 appears in §Tasks with a dead conditional ("if Assets OpenAPI available … else stub"); T-08 has no row in §File Structure Requirements; T-08 specifies no struct/fields/types; T-08 is referenced by no Acceptance Criterion. The story is therefore deliverable green with all 11 assets columns silently empty.

Compounds with GAP-ASSETS-PAG-001 (registered D-2017): the assets surface also has a first-page-only truncation risk.

**`#[non_exhaustive]` three-site discipline:** Adding `Asset` as a new public type in `prism-dtu-cyberint` requires updating: (1) `scripts/check-non-exhaustive.sh` EXPECTED count (currently 92); (2) `scripts/check-non-exhaustive-per-symbol.py` `EXPECTED_COUNT` constant AND appending `Asset` to `EXPECTED_SYMBOLS`; (3) the CLAUDE.md count sentence (currently reads "92 types currently enforced"). All three sites must be updated atomically per CLAUDE.md §Conventions. New `EXPECTED` value = 93.

**Routing:** story-writer (T-04 scope correction to include `Asset` struct; T-08 task body with struct/fields/types; §File Structure Requirements entry for `routes/assets.rs`; AC coverage for the assets surface); implementer (DTU `Asset` struct + `assets.rs` route + three-site non-exhaustive update). **OPEN.**

---

## Important Findings

### F-SAP2-HIGH-001 — T-08's dead conditional ("if Assets OpenAPI available") still licenses an `assets_stub.rs` returning empty 200 [OPEN — story-writer]

**Artifact:** `S-WAVE-A-CYBERINT-SPEC-001` §Tasks T-08

**Defect.** T-08 contains: "If Assets OpenAPI available: create full route… else: assets_stub.rs returning empty 200." Its own trailing note acknowledges the OpenAPI file is confirmed present (`cyberint_assets_openapi_06.20.2026.json`). The story §Version History claims a prior pass "removed both stub placeholders," but this branch survived, contradicting T-03's rule that "no stub is acceptable." An empty-200 is wire-indistinguishable from "zero assets" — the same CWE-390 silent-truncation class as F-SAP2-CRIT-002.

**Routing:** story-writer (delete the dead conditional branch entirely; the OpenAPI file is confirmed present). **OPEN.**

---

## Medium Findings

### F-SAP2-MED-001 — T-03's DTU-types dependency is mis-anchored to T-04 instead of T-08 [OPEN — story-writer]

**Artifact:** `S-WAVE-A-CYBERINT-SPEC-001` §Tasks T-03

**Defect.** T-03 derives its DTU-types presence check from T-04, which is scoped to `clone.rs` and alert route paths only. No `Asset` struct is created in T-04. This mis-anchor is the mechanism by which F-SAP2-CRIT-002 escapes detection at implementation time — T-03 runs before T-08, so when T-03 checks for DTU types, the assets struct does not yet exist, and the gate erroneously passes. T-03 should anchor its asset-column dependency to T-08.

**Routing:** story-writer. **OPEN.**

---

### F-SAP2-MED-002 — Cyberint assets OpenAPI has divergent types for `discovery_precision` and `id`; T-08 does not name the grounding schema [OPEN — story-writer]

**Artifact:** Cyberint assets OpenAPI `cyberint_assets_openapi_06.20.2026.json`; `S-WAVE-A-CYBERINT-SPEC-001` §Tasks T-08

**Defect.** The OpenAPI defines `discovery_precision` twice with divergent types (`integer|null` in the `Asset` schema, `number` in a sibling schema) and `id` twice (`integer` in `Asset`, `string` in `Threat`). T-03 correctly derived types from `Asset`; T-08 says only "same shape as the real API" without naming the grounding schema, risking a pre-baked TYPE-MISMATCH when an implementer resolves the ambiguity differently.

**Routing:** story-writer (T-08 body must name `Asset` as the grounding schema explicitly). **OPEN.**

---

### F-SAP2-MED-003 — T-02's "carry over exactly" would propagate stale DTU-parity comments describing the retired cursor wire shape [OPEN — story-writer, implementer]

**Artifact:** `S-WAVE-A-CYBERINT-SPEC-001` §Tasks T-02

**Defect.** T-02 instructs the implementer to "carry over exactly" from the existing alerts TOML. The existing TOML contains stale comments describing the retired cursor pagination wire shape (`{"data": […], "next_cursor": …}`, `# Cursor-based pagination`, `DTU route: GET /api/v1/alerts`, and a `page_size: OMITTED` rationale citing DTU-EXT-005). Carrying these over verbatim would create false ground truth for future SAP-2 passes.

**Routing:** story-writer (T-02 instruction must specify which parts to carry and which to omit); implementer (TOML text must not carry stale cursor-era comments). **OPEN.**

---

### F-SAP2-MED-004 — `affected_assets` is emitted on the wire but has no consuming TOML column [OPEN — product-owner]

**Artifacts:** `prism-dtu-cyberint` alerts handler `json!` envelope (eighth key: `affected_assets`); `cyberint-alerts.sensor.toml` §columns block

**Defect.** The alerts handler includes `affected_assets` as the eighth key in its `json!` literal. No TOML column in `cyberint-alerts.sensor.toml` declares this field. The wire data is available but unreachable from any spec-driven query. SAP-2 rule 4 ("Field in DTU with no TOML column = MEDIUM").

**Routing:** product-owner (OCSF mapping decision for `affected_assets`); implementer (TOML column addition if product-owner approves). **OPEN.**

---

### F-SAP2-MED-005 — Six `DeviceRecord` fields have no Armis TOML column; `risk_factors` is the explanatory companion of `risk_score` [OPEN — product-owner]

**Artifacts:** `prism-dtu-armis::types` `DeviceRecord`; `armis.sensor.toml` (current or successor spec) §columns block

**Defect.** Six `DeviceRecord` fields have no TOML column: `os_version`, `risk_factors`, `network_id`, `site`, `tags`, `device_cves`. Of these, `risk_factors` is the explanatory companion of the covered `risk_score` field — a query user seeing `risk_score = 8.4` with no `risk_factors` column has no path to understanding the contributing factors. Pre-existing coverage debt.

**Routing:** product-owner (OCSF mapping and coverage decision for these six fields). **OPEN.**

---

### F-SAP2-MED-006 — `ActivityRecord`/`ActivityResponse`/`ActivityData` exist for the `armis_device_activity` adapter but no TOML table declares that surface [OPEN — product-owner]

**Artifacts:** `prism-dtu-armis::types` (activity response types); Armis TOML spec tables

**Defect.** `ActivityRecord`, `ActivityResponse`, and `ActivityData` exist in `prism-dtu-armis::types` so the `armis_device_activity` adapter can deserialize activity data. However, no TOML table declares the `armis_device_activity` surface, making the route unreachable from any spec-driven query.

**Routing:** product-owner (decision on whether this surface should be exposed; if yes, story-writer for TOML table addition). **OPEN.**

---

## Observations

### F-SAP2-OBS-001 — The Armis `aql` pseudo-column is correctly by-design (BC-2.11.007 §Mechanism B); naive SAP-2 would mint two false P1s [informational]

The Armis `aql` pseudo-column carries `options = ["INDEX"]` and no `ocsf_field`. A naive application of SAP-2 rule 1 ("Column in TOML with no DTU equivalent = P1 CRITICAL") would flag it as unbacked. In fact, BC-2.11.007 §Mechanism B governs verbatim-AQL pushdown — `aql` is a query-construction pseudo-column, not a data column. This observation is recorded so future SAP-2 passes do not mint the same false findings.

---

### F-SAP2-OBS-002 [process-gap] — SAP-2's type-mapping rule treats `Datetime ↔ chrono DateTime` as the only valid pairing, producing false findings for timestamp strings with declared `timestamp_formats` [PENDING HUMAN DECISION — do NOT edit CLAUDE.md]

The current SAP-2 CLAUDE.md type-mapping rule reads: "Datetime ↔ chrono DateTime." No DTU field uses chrono in its wire shape — JSON has no native datetime type, and the ratified normalization path is a wire string plus a `timestamp_formats` parse chain (e.g., `cyberint-alerts.sensor.toml` `created_at` declares `["iso8601","unix_epoch_seconds"]` with E-SPEC-018 on total failure). Applying the current literal rule would force five or more false findings across Wave-A sensor TOMLs.

**Recommendation:** Amend the SAP-2 type-mapping rule to read: "`Datetime ↔ chrono DateTime` OR `Datetime ↔ ISO-8601/epoch string with a declared \`timestamp_formats\` chain`." This is a CLAUDE.md edit and therefore a human gate — record as pending human decision. The state-manager must NOT edit CLAUDE.md in response to this observation.

---

### F-SAP2-OBS-003 — Cyberint `ThreatItem` loses its last spec consumer in the dual-surface split; dropping `incidents` per AC-007 retires EC-016-013-002 [informational]

The dual-surface split in S-WAVE-A-CYBERINT-SPEC-001 drops the `incidents` surface (AC-007), which was the last spec consumer of `ThreatItem` in `prism-dtu-cyberint::types`. This simultaneously retires the documented gap EC-016-013-002 (where the spec ran ahead of a nonexistent DTU route). No action required; recorded for audit continuity.

---

## Coverage Declaration

| Probe axis | Status | Notes |
|---|---|---|
| `prism-dtu-cyberint::types` struct field inventory vs TOML columns | FULLY-CHECKED for `Alert` (alerts surface) and `Asset` comparison | |
| `prism-dtu-cyberint` route handler emission literals | FULLY-CHECKED for the list-alerts handler `json!` envelope | Critical gap found (8-key literal vs full struct) |
| `prism-dtu-cyberint::routes` module registration | FULLY-CHECKED | Only `alerts`, `dtu`, `threats` registered |
| `prism-dtu-armis::types` struct field inventory vs TOML columns | PARTIALLY-CHECKED | `DeviceRecord` fields checked; `AlertRecord` / `AqlResponse` not verified end-to-end |
| `prism-dtu-armis` route handler emission literals (`devices.rs`, `alerts.rs`) | **NOT CHECKED end-to-end** | Per-route emission literals NOT read; a `json!`-literal field-dropping pattern analogous to the Cyberint alerts handler cannot be ruled out. Recommended follow-up narrow probe. |
| `device_cves_first` generator projection gating | **NOT TRACED** | Gated to `Archetype::CompromisedEndpoint`; not traced to the `/api/v1/search` path the TOML actually fetches. Recommended follow-up narrow probe. |
| SAP-2-OBS-002 type-mapping rule amendment | RECORDED | Pending human gate; no CLAUDE.md edit performed. |

**Residual risk statement:** The `prism-dtu-armis` per-route emission literals were NOT read end-to-end. A `json!`-literal field-dropping pattern analogous to the Cyberint alerts handler cannot be ruled out. Both the `devices.rs`/`alerts.rs` emission path and the `device_cves_first` generator projection are recommended as narrow follow-up probes before the Armis story enters implementation.

---

```
SAP-2 RESULT: FAILED
Counting findings: 9 (2 CRIT + 1 HIGH + 6 MED)
Observations: 3 (1 process-gap)
All counting findings: OPEN
```
