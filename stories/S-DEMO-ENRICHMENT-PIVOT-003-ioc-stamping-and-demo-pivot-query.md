---
document_type: story
story_id: S-DEMO-ENRICHMENT-PIVOT-003
title: "IOC Stamping (Cyberint + CrowdStrike) and Demo Pivot Query Validation"
wave: 5
epic_id: E-DEMO
priority: P2
status: draft
version: "1.3"
level: "L4"
producer: story-writer
timestamp: "2026-06-12T00:00:00Z"
created: "2026-06-12"
modified: "2026-06-12T20:00:00Z"
tdd_mode: strict
subsystems: [SS-01]
# Subsystem anchor justifications:
#   SS-01 (Sensor Adapters) owns all prism-dtu-* crates per ARCH-INDEX Subsystem Registry.
#   This story adds real-schema IOC fields to prism-dtu-cyberint and prism-dtu-crowdstrike,
#   modifies the Cyberint alerts route to replace the synthetic _ioc_value filter with
#   real-schema field matching, and validates the canonical pivot queries against the
#   demo server. All changes are in SS-01 scope.
target_module: prism-dtu-cyberint
crates_touched: [prism-dtu-cyberint, prism-dtu-crowdstrike, prism-dtu-demo-server]
behavioral_contracts: [BC-2.06.019, BC-2.06.020]
# BC array propagation:
# BC-2.06.019 governs per-sensor IOC-surface masking (PC-4), the Per-Sensor IOC-Surface
# Matrix, and the Route Coverage Table. This story implements the IOC-stamping scope
# deferred to it by BC-2.06.019 v1.6 §Interim State and §Per-Sensor IOC-Surface Matrix.
# BC-2.06.020 governs ThreatIntel/NVD enrichment correlation; the ScenarioEntityCatalog
# IOC values stamped here are the exact values that BC-2.06.020 pre-populates into
# ThreatIntel fixture_registry. The cross-DTU entity coherence path depends on both BCs.
# Both BCs cited by ACs below (bidirectional trace requirement satisfied).
verification_properties: []
# VP note (U24): VP-019-A, VP-019-B, VP-019-C are BC-2.06.019-internal sub-properties
# (no standalone VP-INDEX files). Setting verification_properties: [] avoids tooling
# resolution failure and collision with vp-019-diff-deterministic.md. The VP citations
# are preserved in the story body as BC-2.06.019-internal sub-properties prose references.
# VP-019-G/H are also BC-internal; exercised by route handler changes in this story.
depends_on:
  - S-DEMO-ENRICHMENT-PIVOT-002
  # Dependency anchor: 002 delivers the threat_intel and nvd infusion UDFs registered
  # in the query engine. The canonical pivot queries validated in this story —
  # | enrich threat_intel(ioc.value) and | enrich nvd(device_cves_first) — require
  # those UDFs to be operational. Without 002 merged, the pivot query validation in
  # this story cannot complete.
blocks: []
# Blocks: T11 (demo prep) and T13 (capstone demo segment) per WO-D1109 §Demo Objective
# Sequencing. These are demo objectives, not story IDs, so blocks[] is empty.
points: 8
# Points justification:
#   1. Cyberint Alert struct: add real-schema IOC fields
#      (ioc: { type, value }, iocs: Vec<Ioc>, alert_data.ip/domain/url): 1.5 pts
#   2. Cyberint fixture generator: stamp scenario IOCs onto alert records: 1 pt
#   3. Cyberint alerts route: remove _ioc_value synthetic filter atomically +
#      add real-schema filter (SAME commit — cannot coexist): 1.5 pts
#   4. CrowdStrike detections: stamp behaviors[].ioc_type/ioc_value/ioc_source/
#      ioc_description on fixture records: 1 pt
#   5. TOML sensor spec alignment (cyberint + crowdstrike detections): 0.5 pts
#   6. Canonical pivot query validation against demo server at stage >= 3: 1.5 pts
#   7. Red Gate test suite (~9 tests): 1 pt
#   Total: 8 pts
estimated_days: 3
risk: HIGH
# Risk justification: Removing the _ioc_value synthetic filter atomically with adding
# the real-schema filter is the root cause closure of BPRL-P4-01. The two MUST NOT
# coexist in the same route handler (BC-2.06.019 v1.6 §Interim State). SAP-2 must be
# run on both TOML sensor specs against DTU types.rs before writing any column declaration.
# The Cyberint Alert struct addition must not break existing route handlers or serializers.
# CrowdStrike behaviors[] array is on the Detection object (not on host/device) — do NOT
# confuse the two.
red_gate_tests: 9
estimated_passes: "3-5 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "_ioc_value removal (BC-2.06.019 v1.6 §Interim State): the synthetic _ioc_value /
     _ioc_type filter in crates/prism-dtu-cyberint/src/routes/alerts.rs MUST be removed
     in the SAME commit that adds the real-schema filter. They cannot coexist.
     BC-2.06.019 v1.6 §Interim State clause is explicit: 'The synthetic filter and the
     real-schema filter MUST NOT coexist.' Any implementer attempt to ship the synthetic
     filter as interim is a BC-2.06.019 v1.6 violation and a BPRL-P4-01 recurrence."
  - "CrowdStrike IOC scope (BC-2.06.019 v1.6 Per-Sensor IOC-Surface Matrix): behaviors[]
     carries ioc_type ∈ {hash, domain, filename, registry, cmdline} — NOT ipv4/ipv6.
     IPs only appear on streaming NetworkAccesses shape, not detection records. Do NOT
     stamp ipv4/ipv6 IOC types on detection records."
  - "Armis and Claroty: NO IOC stamping (permanent exclusion per BC-2.06.019 v1.6 matrix).
     Armis alert payloads are reference-only; Claroty has IP addresses only as free text.
     Fabricating IOC fields on these sensors violates ADR-031 DTU=True-DTU fidelity.
     Any implementer attempt to add IOC fields to Armis or Claroty records is a BC violation."
  - "TOML spec alignment (SAP-2): before declaring any new TOML column, run SAP-2 check
     against the DTU types.rs for cyberint and crowdstrike. Every new column MUST have a
     matching field in the DTU Alert or Detection struct after this story's changes."
  - "Route Coverage Table update (BC-2.06.019 v1.6 §Route Coverage Table standing rule):
     after implementing real-schema Cyberint filter, update the Route Coverage Table in
     BC-2.06.019 to change the Cyberint alerts row from INTERIM to ACTIVE. This is a
     MANDATORY same-commit change per the standing rule in BC-2.06.019 v1.6."
  - "Pivot query stage gate: queries must return non-empty results at stage >= 3 (Exfil)
     when ioc_ips, ioc_domains, ioc_hashes are visible in StageMask. Verify demo server
     is at stage >= 3 before running integration tests by setting scenario_start_secs to
     a past epoch so stage 3 is already active."
traces_to: [D-1109, WO-D1109, BPRL-P4-01]
supersedes: []
---

# S-DEMO-ENRICHMENT-PIVOT-003: IOC Stamping and Demo Pivot Query Validation

Root-cause closure of BPRL-P4-01: add real-schema IOC fields to the Cyberint `Alert`
struct and CrowdStrike `behaviors[]` array, remove the synthetic `_ioc_value` filter
atomically, and validate the canonical analyst pivot queries against the demo server.

**BC-2.06.019 v1.6 scope ownership:** This story implements the deferred IOC-stamping
work that BC-2.06.019 v1.6 assigned to `S-DEMO-ENRICHMENT-PIVOT-003` via the
Per-Sensor IOC-Surface Matrix and the §Interim State clause. The story also owns the
Route Coverage Table update in BC-2.06.019 (changing Cyberint alerts row from INTERIM
to ACTIVE).

**Sequencing context (D-1109, WO-D1109):** Slots AFTER S-DEMO-ENRICHMENT-PIVOT-002
(infusion specs + plugins operational) and BEFORE T11 and T13 capstone demo segment.

---

## Narrative

As a SOC analyst in the capstone demo (T13), I want to run
`FROM cyberint_alerts | where severity = "high" | enrich threat_intel(ioc.value) | where threat_is_known_malicious = true`
and see real-schema IOC fields from the Cyberint alert records resolve correctly against
the ThreatIntel enrichment service — not a synthetic injected field — so that the demo
faithfully demonstrates the production enrichment pivot workflow.

---

## Behavioral Contracts

| BC | Title | Key Clauses |
|----|-------|-------------|
| BC-2.06.019 v1.6 | Demo-Server Scenario Progression — Per-Sensor IOC-Surface Masking | PC-4 Per-Sensor IOC-Surface Matrix: Cyberint alerts and CrowdStrike detections carry real-schema IOC fields; Armis/Claroty permanently excluded; `_ioc_value` synthetic filter removed atomically; Route Coverage Table updated |
| BC-2.06.020 v1.2 | Demo-Server Enrichment Correlation — Scenario IOCs Resolve in ThreatIntel | INV-THREATINTEL-IOC-CORRELATION-001: scenario IOCs in ScenarioEntityCatalog resolve as Malicious in ThreatIntel; INV-CROSS-DTU-ENTITY-COHERENCE-001: entity IDs coherent across DTU clones |

**VP Citation (U24):** VP-019-A (pure function reproducibility), VP-019-B (stage monotonicity),
VP-019-C (StageMask completeness), VP-019-G, VP-019-H are BC-2.06.019-internal sub-properties
with no standalone VP-INDEX files. `verification_properties: []` is set in frontmatter to avoid
tooling resolution failure and collision with `vp-019-diff-deterministic.md`. These
BC-2.06.019-internal sub-properties must not regress when Cyberint alerts route and CrowdStrike
generator are modified.

---

## Acceptance Criteria

### AC-001 — Cyberint Alert struct adds real-schema IOC fields
(traces to BC-2.06.019 v1.6 postcondition 4 — Per-Sensor IOC-Surface Matrix, Cyberint row)

Given `crates/prism-dtu-cyberint/src/types.rs`,
when the `Alert` struct is inspected after this story,
then it includes:
- `ioc: Option<Ioc>` where `Ioc { ioc_type: String, value: String }` (single inline IOC)
- `iocs: Vec<Ioc>` (list of IOCs, same `Ioc` struct as used by `ThreatItem`)
- `alert_data: Option<AlertData>` where `AlertData { ip: Option<String>, domain: Option<String>, url: Option<String> }`

These fields match the real Cyberint API schema per the Per-Sensor IOC-Surface Matrix
(research-agent 2026-06-12 findings recorded in BC-2.06.019 v1.6).

Red Gate: `test_BC_2_06_019_cyberint_alert_struct_has_real_ioc_fields`

### AC-002 — Cyberint fixture generator stamps scenario IOCs onto alert records
(traces to BC-2.06.019 v1.6 postcondition 4 — IOC-surface fields populated from catalog)

Given a Cyberint clone constructed with `new_with_scenario` and `ScenarioEntityCatalog`
containing `ioc_ips`, `ioc_domains`, `ioc_hashes` entries,
when the fixture generator runs,
then scenario-enabled alert records in the `FixtureSet` have their `ioc.value` / `iocs[0].value`
field set to a value from the catalog's IOC lists, per the real Cyberint API schema.

Red Gate: `test_BC_2_06_019_cyberint_fixture_generator_stamps_scenario_iocs`

### AC-003 — Cyberint alerts route: _ioc_value synthetic filter REMOVED; real-schema filter ADDED (atomic)
(traces to BC-2.06.019 v1.6 postcondition 4 §Interim State — synthetic filter must be replaced atomically)

Given `crates/prism-dtu-cyberint/src/routes/alerts.rs` after this story:
- The `_ioc_value` / `_ioc_type` synthetic field filter is ABSENT (grep for `_ioc_value` returns
  no matches in the file)
- A new filter reads real-schema fields: `rec.ioc.value`, `rec.iocs[].value`,
  `rec.alert_data.ip`, `rec.alert_data.domain`
- When `ioc_hashes=false`: alert records where any IOC field matches a value in
  `catalog.ioc_hashes` are withheld from the response
- When `ioc_ips=false`: alert records where `ioc.value`, `iocs[].value`, or
  `alert_data.ip` matches `catalog.ioc_ips` values are withheld
- When `ioc_domains=false`: alert records where `ioc.value`, `iocs[].value`, or
  `alert_data.domain` matches `catalog.ioc_domains` values are withheld

INVARIANT: The synthetic filter and the real-schema filter MUST NOT coexist in the same
route handler. This is verified by grep: `grep -n '_ioc_value' crates/prism-dtu-cyberint/src/routes/alerts.rs`
MUST return 0 matches after this story merges.

Red Gate: `test_BC_2_06_019_cyberint_alerts_real_schema_ioc_filter_no_synthetic`

### AC-004 — CrowdStrike detection generator stamps behaviors[].ioc_type/ioc_value in JSON records
(traces to BC-2.06.019 v1.6 postcondition 4 — CrowdStrike detections row in Per-Sensor IOC-Surface Matrix)

NOTE (U19): CrowdStrike has NO typed `Detection` or `Behavior` structs in types.rs. All
detection records are untyped `serde_json::Value` built by `generator.rs` from
`fixtures/detections-detail.json` with a `behaviors[]` JSON array. The generator
`make_detection()` function currently produces NO `behaviors` key. This AC adds it.

Given a CrowdStrike clone constructed with `new_with_scenario` and `ScenarioEntityCatalog`
containing `ioc_hashes` entries,
when the fixture generator (`make_detection()` in `src/generator.rs`) populates scenario
detection records,
then the generated `serde_json::Value` detection records include a `"behaviors"` JSON array
key with at least one entry containing:
- `"ioc_type": "hash"` — JSON record stamping in the generator (NOT a Rust struct field edit)
- `"ioc_value": "<catalog.ioc_hashes[0]>"`
- `"ioc_source": "catalog"`
- `"ioc_description": "scenario IOC"`

GENERATOR SHAPE PARITY: Per the `make_detection()` doc comment and `review_2026_06_10_cs_parity.rs`
test, the flat scalar key set of generated detection records MUST equal the key set of static
fixtures (`fixtures/detections-detail.json`). Adding `behaviors[]` to generated records requires
the same addition to the static fixture JSON in the same commit.

CrowdStrike IOC type constraint: `ioc_type` is restricted to
`{hash, domain, filename, registry, cmdline}` — NOT `ipv4` or `ipv6` per the real
FalconPy SDK / CrowdStrike Detect API (BC-2.06.019 v1.6 matrix, CrowdStrike row).

Red Gate: `test_BC_2_06_019_crowdstrike_detection_behaviors_ioc_hash_stamped`

### AC-005 — CrowdStrike detections TOML spec declares behaviors[] IOC columns matching generator JSON shape
(traces to BC-2.06.019 v1.6 postcondition 4 — TOML spec alignment with real-schema fields)

NOTE (U19): CrowdStrike detection records are untyped serde_json::Value. The SAP-2 check reads
`src/generator.rs` `make_detection()` (and `fixtures/detections-detail.json`) — NOT types.rs
structs. The `#[non_exhaustive]` item for CrowdStrike detection struct is MOOT (no typed struct
exists). Adversary reads generator.rs and detections-detail.json to verify TOML column parity.

Given the CrowdStrike sensor TOML spec,
when the spec is inspected after this story,
then it declares columns for `behaviors[].ioc_type`, `behaviors[].ioc_value`,
`behaviors[].ioc_source`, `behaviors[].ioc_description` per the JSON keys stamped by
`make_detection()` in `src/generator.rs` (added by this story's AC-004 scope).

SAP-2 compliance: adversary reads `src/generator.rs` `make_detection()` return value AND
`fixtures/detections-detail.json` to verify column parity — NOT types.rs (no typed struct).

Red Gate: `test_BC_2_06_019_crowdstrike_detection_toml_spec_has_ioc_columns` (or SAP-2 parity assertion)

### AC-006 — Cyberint sensor TOML spec declares ioc, iocs[], alert_data.* columns
(traces to BC-2.06.019 v1.6 postcondition 4 — TOML spec alignment with real-schema fields)

Given the Cyberint sensor TOML spec,
when the spec is inspected after this story,
then it declares columns using the wire names from the Cyberint API (U22 — serde rename applies):
`ioc.type`, `ioc.value`, `iocs[].type`, `iocs[].value`,
`alert_data.ip`, `alert_data.domain`, `alert_data.url`.
NOTE: `iocs[].type` is the WIRE name (NOT `iocs[].ioc_type`). The serde rename maps the Rust
field `ioc_type` on the `Ioc` struct to `type` in JSON (serde rename). Verify the exact serde
annotation in types.rs before declaring the column name.

Each declared column has a matching field in the Cyberint DTU types.rs Alert struct
after this story's additions (SAP-2 compliance).

Red Gate: `test_BC_2_06_019_cyberint_alert_toml_spec_has_ioc_columns` (or SAP-2 parity assertion)

### AC-007 — Canonical ThreatIntel pivot query returns Malicious results at stage >= 3
(traces to BC-2.06.019 v1.6 postcondition 4 + BC-2.06.020 INV-THREATINTEL-IOC-CORRELATION-001)

Given demo server at stage >= 3 (Exfil; `ioc_ips`, `ioc_domains`, `ioc_hashes` visible),
when the following canonical query executes:
```prismql
FROM cyberint_alerts
| where severity = "high"
| enrich threat_intel(ioc.value)
| where threat_is_known_malicious = true
| sort threat_score desc
| head 10
```
then the result set is non-empty and all returned records have
`threat_is_known_malicious = true` with `threat_score >= 75`.

Red Gate: `test_BC_2_06_019_canonical_threatintel_pivot_query_returns_malicious_at_stage_3`

### AC-008 — Canonical NVD pivot query returns HIGH CVSS results at stage >= 3
(traces to BC-2.06.019 v1.6 postcondition 4 + BC-2.06.020 INV-NVD-CVE-CORRELATION-001)

Given demo server at stage >= 3 (Exfil; `device_cves` may be false at stage 3 per
BC-2.06.019 PC-2 table — verify at stage 4 Containment if device_cves requires it),
when the following canonical query executes:
```prismql
FROM armis_devices
| where has device_cves
| enrich nvd(device_cves_first)
| where cvss_base_score >= 7.0
| sort cvss_base_score desc
| head 10
```
then the result set is non-empty and all returned records have `cvss_base_score >= 7.0`.

NOTE: per BC-2.06.019 PC-2 table, `device_cves = false` at stages 0-3 and `true` only at
Containment (stage 4). This query may need to run at stage 4 or use a demo server
pre-configured to start at stage 4. The implementer must verify the stage requirement
and document the correct scenario_start_secs in the test harness.

Red Gate: `test_BC_2_06_019_canonical_nvd_pivot_query_returns_high_cvss_at_containment_stage`

### AC-009 — BC-2.06.019 Route Coverage Table updated: Cyberint alerts row INTERIM -> ACTIVE
(traces to BC-2.06.019 v1.6 §Route Coverage Table standing rule)

Given `BC-2.06.019-demo-server-scenario-progression.md` after this story,
when the Route Coverage Table is inspected,
then the Cyberint alerts row shows:
- Guard Mechanism: `Real-schema filter on ioc.value / iocs[].value / alert_data.ip/domain`
- Status: `ACTIVE` (NOT `INTERIM`)

This update is required in the SAME commit as AC-003 per the Route Coverage Table
standing rule (BC-2.06.019 v1.6: "any future story adding or modifying a StageMask-relevant
route MUST extend or update this table in the same commit").

NOTE: BC-2.06.019 is a `.factory/` artifact — it is edited via Write/Edit tools by
state-manager at post-merge burst. The implementer notes this requirement in the PR;
the state-manager burst applies it.

Red Gate: N/A (process check — adversary verifies BC Route Coverage Table status post-merge)

---

## Red Gate Test Plan

| # | Test Name | Crate | BC Clause | Type |
|---|-----------|-------|-----------|------|
| 1 | `test_BC_2_06_019_cyberint_alert_struct_has_real_ioc_fields` | prism-dtu-cyberint | BC-2.06.019 PC-4 matrix | unit |
| 2 | `test_BC_2_06_019_cyberint_fixture_generator_stamps_scenario_iocs` | prism-dtu-cyberint | BC-2.06.019 PC-4 / BC-2.06.020 PC-1 | unit |
| 3 | `test_BC_2_06_019_cyberint_alerts_real_schema_ioc_filter_no_synthetic` | prism-dtu-cyberint | BC-2.06.019 PC-4 §Interim State | unit |
| 4 | `test_BC_2_06_019_crowdstrike_detection_behaviors_ioc_hash_stamped` | prism-dtu-crowdstrike | BC-2.06.019 PC-4 matrix | unit |
| 5 | `test_BC_2_06_019_crowdstrike_detection_toml_spec_has_ioc_columns` | prism-dtu-crowdstrike or sensor spec tests | BC-2.06.019 PC-4 + SAP-2 | unit/parity |
| 6 | `test_BC_2_06_019_cyberint_alert_toml_spec_has_ioc_columns` | prism-dtu-cyberint or sensor spec tests | BC-2.06.019 PC-4 + SAP-2 | unit/parity |
| 7 | `test_BC_2_06_019_canonical_threatintel_pivot_query_returns_malicious_at_stage_3` | prism-query or prism-bin integration | BC-2.06.019 PC-4 + BC-2.06.020 INV-THREATINTEL-IOC-CORRELATION-001 | integration (demo server) |
| 8 | `test_BC_2_06_019_canonical_nvd_pivot_query_returns_high_cvss_at_containment_stage` | prism-query or prism-bin integration | BC-2.06.019 PC-4 + BC-2.06.020 INV-NVD-CVE-CORRELATION-001 | integration (demo server) |
| 9 | `test_BC_2_06_019_ioc_hashes_false_withholds_cyberint_alert_with_matching_hash` | prism-dtu-cyberint | BC-2.06.019 PC-4 ioc_hashes=false filtering | unit |

---

## Route Coverage Table (for StageMask IOC fields — per BC-2.06.019 v1.6 §Question 4)

This table must be kept in sync with BC-2.06.019 §Route Coverage Table. After this story
ships, the Cyberint row transitions from INTERIM to ACTIVE.

NOTE (U20, confirmed 2026-06-12 from actual routers in 001-B worktree): regenerated from
real route registrations. Rows referencing non-existent routes removed; correct paths inserted.

| StageMask Field | DTU Clone | Route File | Route Path | Guard Mechanism | Status after 003 |
|-----------------|-----------|------------|------------|-----------------|-----------------|
| `ioc_hashes`, `ioc_ips`, `ioc_domains` | prism-dtu-cyberint | `routes/alerts.rs` | `GET /api/v1/alerts` (confirmed clone.rs) | Real-schema filter on `ioc.value` / `iocs[].value` / `alert_data.ip/domain` (replaces `_ioc_value` synthetic filter) | ACTIVE |
| `ioc_hashes` | prism-dtu-crowdstrike | `routes/detections.rs` | `GET /detects/queries/detects/v1` (list IDs) + `POST /detects/entities/summaries/GET/v1` (get summaries — confirmed routes/mod.rs) | `stage_idx > 0` guard (bc0f36c5) + IOC JSON-key stamping in `make_detection()` on `behaviors[0].ioc_value` (this story) | ACTIVE (IOC-stamp added this story) |
| `primary_device`, `lateral_devices` | prism-dtu-armis | `routes/` | `GET /api/v1/devices` + `GET /api/v1/search` + `GET /api/v1/alerts` (confirmed clone.rs) | Stage index filter | ACTIVE (unchanged) |
| `primary_device`, `lateral_devices` | prism-dtu-crowdstrike | `routes/hosts.rs` | `GET /devices/queries/devices/v1` + `GET /devices/entities/devices/v2` (confirmed routes/mod.rs) | Stage index filter | ACTIVE (unchanged) |
| (no IOC surface) | prism-dtu-claroty | `routes/alerts.rs` | `POST /api/v1/alerts` (confirmed clone.rs) | EXEMPT — permanent (no structured IOC fields in real Claroty API) | PERMANENT EXEMPT |

REMOVED ROWS (U20 — routes do not exist in actual router):
- `routes/alerts_search.rs` / `GET /alerts/queries/alerts/v2`: NO such route file or path in
  prism-dtu-crowdstrike (CrowdStrike router has: oauth, detections, hosts, writes — no alerts_search module)
- `GET /xdome/api/v1/alerts`: Claroty serves `POST /api/v1/alerts` not a GET at /xdome/ prefix

---

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~5,000 |
| BC-2.06.019 v1.6 (full — authoritative for IOC matrix, Interim State, Route Coverage Table) | ~7,500 |
| `prism-dtu-crowdstrike/src/generator.rs` make_detection() function + fixtures/detections-detail.json (U19) | ~800 |
| `prism-dtu-armis/src/generator.rs` (device_cves_first projection, U17/Ruling 1b) | ~600 |
| BC-2.06.020 v1.2 (enrichment correlation context) | ~3,000 |
| `prism-dtu-cyberint/src/types.rs` (pre + post this story) | ~1,000 |
| `prism-dtu-cyberint/src/routes/alerts.rs` | ~800 |
| `prism-dtu-crowdstrike/src/types.rs` | ~1,000 |
| `prism-dtu-crowdstrike/src/routes/detections.rs` | ~800 |
| Cyberint sensor TOML spec | ~600 |
| CrowdStrike sensor TOML spec | ~600 |
| S-DEMO-DTU-LIVE-SCENARIO-001-B spec (scenario entity catalog context) | ~2,000 |
| Test files (9 stubs × ~50 lines each) | ~1,350 |
| Tool outputs (nextest, clippy, demo server integration) | ~1,500 |
| **Total estimate** | **~25,150** |

At ~200k context window, this is ~12.6% — within the 20-30% ceiling.

---

## Tasks

**Pre-flight: read substrate before writing anything**

- [ ] MATERIALIZATION-TIME RE-VERIFY (U23): PR #185 was IN-FLIGHT as of 2026-06-12.
  Before implementing, grep for `_ioc_value`, cyberint Alert field names, StageMask ioc_* fields,
  and ScenarioEntityCatalog field names in the merged develop branch:
  `grep -rn "_ioc_value\|ioc_ips\|ioc_domains\|ioc_hashes" crates/prism-dtu-cyberint/ crates/prism-dtu-common/`
  If PR #185 has since merged, the exact field names may differ from this spec; resolve before proceeding.
- [ ] Read `crates/prism-dtu-cyberint/src/types.rs` fully — confirm current `Alert` struct
  has NO `ioc`, `iocs`, or `alert_data` fields; also read `ThreatItem` to confirm `iocs`
  field shape already exists there (for Ioc struct reuse)
- [ ] Read `crates/prism-dtu-cyberint/src/routes/alerts.rs` fully — locate the `_ioc_value`
  synthetic filter; confirm its exact code pattern before removing
- [ ] Read `crates/prism-dtu-crowdstrike/src/generator.rs` `make_detection()` function (U19):
  confirm no `behaviors` key currently in the generated JSON; note flat scalar key set for
  parity test compliance; also read `fixtures/detections-detail.json` for static fixture parity
- [ ] Read `crates/prism-dtu-crowdstrike/tests/review_2026_06_10_cs_parity.rs` — understand
  the shape parity test requirement (static fixture + generator must have identical flat key sets)
- [ ] Read `crates/prism-dtu-armis/src/generator.rs` (or equivalent) — find where device records
  are built; identify the CVE-related fields for `device_cves_first` projection (U17/Ruling 1b)
- [ ] Run SAP-2 pre-check on both DTU crates (cyberint + crowdstrike)
- [ ] Read BC-2.06.019 v1.6 §Per-Sensor IOC-Surface Matrix and §Route Coverage Table — both
  are authoritative for this story's scope

**Phase 1: Cyberint Alert struct + fixture generator**

- [ ] Write failing test 1 (FAIL first): `test_BC_2_06_019_cyberint_alert_struct_has_real_ioc_fields`
- [ ] Add `Ioc { ioc_type: String, value: String }` struct to `types.rs` (or reuse from ThreatItem)
- [ ] Add `AlertData { ip: Option<String>, domain: Option<String>, url: Option<String> }` struct
- [ ] Add to `Alert` struct: `ioc: Option<Ioc>`, `iocs: Vec<Ioc>`, `alert_data: Option<AlertData>`
  (all with `#[serde(skip_serializing_if = ...)]` for None/empty)
- [ ] Check `#[non_exhaustive]` discipline: Alert is a public type in prism-dtu-cyberint.
  If Alert already carries `#[non_exhaustive]`, adding fields is fine. If not, add it per
  CLAUDE.md conventions. Update `ci.yml EXPECTED=` if new `#[non_exhaustive]` types are added.
- [ ] Write failing test 2 (FAIL first): `test_BC_2_06_019_cyberint_fixture_generator_stamps_scenario_iocs`
- [ ] Modify the Cyberint fixture generator (read `src/generator.rs` or equivalent before editing):
  for CompromisedEndpoint scenario alerts, set `ioc.value` and `iocs[0].value` to
  `catalog.ioc_ips[0]` (or `ioc_domains[0]`, etc.) and `ioc.ioc_type` to the matching type string
- [ ] Verify tests 1-2 pass

**Phase 2: Cyberint alerts route — remove _ioc_value, add real-schema filter (atomic)**

- [ ] Write failing test 3 (FAIL first): `test_BC_2_06_019_cyberint_alerts_real_schema_ioc_filter_no_synthetic`
  (test must assert: when `ioc_hashes=false` and alert has `ioc.value = catalog.ioc_hashes[0]`,
  the alert is NOT returned; AND grep for `_ioc_value` in the route file returns zero matches)
- [ ] Write failing test 9 (FAIL first): `test_BC_2_06_019_ioc_hashes_false_withholds_cyberint_alert_with_matching_hash`
- [ ] In `routes/alerts.rs`: REMOVE the `_ioc_value` / `_ioc_type` synthetic filter block
  AND ADD the real-schema filter in the SAME edit:
  - `ioc_hashes=false`: withhold alert if `ioc.value` or any `iocs[].value` matches `catalog.ioc_hashes`
  - `ioc_ips=false`: withhold alert if `ioc.value`, `iocs[].value`, or `alert_data.ip` matches `catalog.ioc_ips`
  - `ioc_domains=false`: withhold alert if `ioc.value`, `iocs[].value`, or `alert_data.domain` matches `catalog.ioc_domains`
- [ ] After edit: run `grep -n '_ioc_value' crates/prism-dtu-cyberint/src/routes/alerts.rs`
  — MUST return 0 matches
- [ ] Verify tests 3, 9 pass

**Phase 3: CrowdStrike detections IOC stamping (JSON record injection — no typed struct)**

- [ ] Write failing test 4 (FAIL first): `test_BC_2_06_019_crowdstrike_detection_behaviors_ioc_hash_stamped`
- [ ] Read `crates/prism-dtu-crowdstrike/src/routes/detections.rs` — confirm `stage_idx > 0`
  guard is present (bc0f36c5); IOC-field stamping in generator is the new work
- [ ] Modify `make_detection()` in `src/generator.rs` (U19: serde_json::Value JSON record, NOT a struct):
  for scenario-enabled detection records, add `"behaviors"` key to the JSON object with at least one entry:
  `json!([{ "ioc_type": "hash", "ioc_value": catalog.ioc_hashes[0], "ioc_source": "catalog", "ioc_description": "scenario IOC" }])`
- [ ] SHAPE PARITY: update `fixtures/detections-detail.json` in the SAME commit to add the
  `behaviors` key (review_2026_06_10_cs_parity.rs shape parity test will fail otherwise)
- [ ] Verify test 4 passes

**Phase 3b: Armis device_cves_first generator projection (U17/Ruling 1b)**

- [ ] Write failing test: `test_BC_2_06_019_armis_device_cves_first_scalar_projected`
- [ ] Modify the Armis fixture generator: for scenario-enabled device records, add scalar field
  `"device_cves_first": catalog.device_cves[0]` (String — first CVE ID from catalog list)
  on each device record that already has `device_cves` populated.
  This enables `| enrich nvd(device_cves_first)` without FieldPath bracket-index syntax.
- [ ] SHAPE PARITY: if Armis has a static fixture file for devices, update it with the new field
  in the same commit (check for analogous parity test in prism-dtu-armis tests)
- [ ] Verify test passes

**Phase 4: TOML sensor spec alignment**

- [ ] Write failing tests 5, 6 (FAIL first): TOML spec parity tests
- [ ] Update Cyberint sensor TOML spec: add `ioc.type`, `ioc.value`, `iocs[].ioc_type`,
  `iocs[].value`, `alert_data.ip`, `alert_data.domain`, `alert_data.url` columns
- [ ] Update CrowdStrike detections TOML spec: add `behaviors[].ioc_type`,
  `behaviors[].ioc_value`, `behaviors[].ioc_source`, `behaviors[].ioc_description` columns
- [ ] SAP-2 post-edit verification: for each new TOML column, confirm matching DTU struct field
- [ ] Verify tests 5, 6 pass

**Phase 5: Canonical pivot query integration tests**

- [ ] Write failing tests 7, 8 (FAIL first): canonical pivot queries against demo server
- [ ] Configure demo server integration test harness at stage >= 3 for ThreatIntel test
  (set `scenario_start_secs` to past epoch so stage 3 is active at test time)
- [ ] Configure at stage 4 for NVD test (device_cves only visible at Containment per BC-2.06.019 PC-2)
- [ ] Verify canonical ThreatIntel pivot query returns non-empty Malicious results (test 7)
- [ ] Verify canonical NVD pivot query returns non-empty HIGH CVSS results (test 8)

**Phase 6: Final gates**

- [ ] SAP-2 post-implementation sweep: compare TOML columns against DTU types.rs for both
  Cyberint alerts and CrowdStrike detections; document parity table in PR description
- [ ] SAP-1 probe: `rg 'event_type\s*=' crates/ --type rust` — verify any new emissions
  have BC-2.16.002 catalog rows
- [ ] Grep for `_ioc_value` across entire codebase: `grep -rn '_ioc_value' crates/` — MUST
  return 0 matches after this story (the synthetic field is completely removed)
- [ ] Note in PR description: "BC-2.06.019 Route Coverage Table update required — Cyberint
  alerts row INTERIM→ACTIVE. State-manager to apply at post-merge burst."
- [ ] Run `just check` — all 9 Red Gate tests pass; zero clippy warnings; fmt clean

---

## Previous Story Intelligence

**S-DEMO-ENRICHMENT-PIVOT-002 (direct predecessor):**
- `threatintel.infusion.toml` and `nvd.infusion.toml` are loaded and their UDFs registered
- `| enrich threat_intel(ioc_value)` pipe stage is operational (using interim field name from 002)
- This story changes the Cyberint field reference from interim `ioc_value` to `ioc.value` (real-schema)

**S-DEMO-DTU-LIVE-SCENARIO-001-B (substrate — PR #185 IN-FLIGHT as of 2026-06-12, not merged):**
MATERIALIZATION-TIME RE-VERIFY REQUIRED: before implementing, grep for `_ioc_value`,
cyberint Alert field names, Cyberint catalog fields, and StageMask ioc_* fields in the merged
result of PR #185 — the exact field names may shift on merge. See materialization task in §Tasks.

- `ScenarioEntityCatalog.ioc_ips`, `ioc_domains`, `ioc_hashes` are populated at construction
- `ThreatIntelClone::new_with_scenario` pre-populates `fixture_registry` with all catalog IOCs
  as `FixtureKey::Malicious`
- Cyberint `_ioc_value` synthetic filter is present (to be replaced by this story)
- CrowdStrike detections route has `stage_idx > 0` guard from bc0f36c5 but no IOC stamping

**From PLUGIN-MIGRATION-001-D/E lessons:**
- SAP-2: adversary reads DTU types.rs and routes/ — do not rely on story description alone
- SID-1: integration tests driving in-process demo server are NOT `#[ignore]`'d

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| `_ioc_value` and real-schema filter MUST NOT coexist in the same route handler | BC-2.06.019 v1.6 §Interim State | AC-003 + grep verification |
| CrowdStrike `behaviors[].ioc_type` restricted to `{hash, domain, filename, registry, cmdline}` — NOT `ipv4`/`ipv6` | BC-2.06.019 v1.6 Per-Sensor IOC-Surface Matrix (CrowdStrike row) | AC-004 + adversary probe |
| Armis and Claroty: NO IOC fields added (permanent exclusion) | ADR-031 DTU=True-DTU fidelity + BC-2.06.019 v1.6 matrix (Armis/Claroty: NO permanent) | Adversary: grep for ioc in prism-dtu-armis/prism-dtu-claroty types.rs |
| Route Coverage Table in BC-2.06.019 MUST be updated in same commit as route change | BC-2.06.019 v1.6 §Route Coverage Table standing rule | Noted in PR description; state-manager burst |
| Every new TOML column MUST have a matching DTU struct field (SAP-2) | CLAUDE.md §SAP-2 | Adversary SAP-2 probe post-implementation |
| All `event_type =` tracing emissions require BC-2.16.002 catalog rows | SAP-1 / CLAUDE.md §SAP-1 | Adversary SAP-1 probe |
| Pivot queries MUST return non-empty results at stage >= 3 (ThreatIntel) / stage 4 (NVD with device_cves) | BC-2.06.019 PC-2 StageMask table + BC-2.06.020 correlation invariants | Tests 7, 8 |
| `#[non_exhaustive]` on new public types in prism-dtu-cyberint (Ioc, AlertData if public) | CLAUDE.md §Conventions | ci.yml EXPECTED= bump |
| CrowdStrike generator JSON stamping: `behaviors[]` key added to `make_detection()` + static `fixtures/detections-detail.json` in SAME commit (generator-parity test) | review_2026_06_10_cs_parity.rs + U19 | test_f8_cs06_detection_shape_parity must pass |
| CrowdStrike: NO `#[non_exhaustive]` change needed (no typed Detection/Behavior struct exists; IOC fields are JSON keys in serde_json::Value records — U19) | U19 code grounding 2026-06-12 | Adversary verifies no struct added without justification |

**Forbidden patterns:**
- `_ioc_value` field references in any production route handler after this story
- `ioc_type = "ipv4"` or `ioc_type = "ipv6"` in CrowdStrike behaviors[] stamping
- IOC fields on Armis or Claroty records (ADR-031 violation)

---

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| `serde` / `serde_json` | 1.x (workspace) | Alert/Detection struct serialization with new IOC fields |
| `axum` | `0.7` (confirmed 2026-06-12 — matches DTU pins) | Route handlers in prism-dtu-cyberint and prism-dtu-crowdstrike |
| `tokio` | `1.x` (workspace) | Async route handlers |
| `chrono` | project-pinned | `Utc::now().timestamp()` for stage computation in route handlers |
| `reqwest` | project-pinned | Integration test HTTP clients; `.timeout(Duration::from_secs(30))` mandatory |

**MSRV:** Rust stable per `rust-toolchain.toml`.

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-dtu-cyberint/src/types.rs` | MODIFY | Add `Ioc`, `AlertData` structs; add `ioc`, `iocs`, `alert_data` fields to `Alert` |
| `crates/prism-dtu-cyberint/src/generator.rs` (or equivalent) | MODIFY | Stamp scenario IOC catalog values onto generated Alert records |
| `crates/prism-dtu-cyberint/src/routes/alerts.rs` | MODIFY | Remove `_ioc_value` synthetic filter; add real-schema filter (ATOMIC in same edit) |
| `crates/prism-dtu-crowdstrike/src/types.rs` | NO CHANGE (U19) | No typed Detection/Behavior struct exists; IOC fields are JSON keys only |
| `crates/prism-dtu-crowdstrike/src/generator.rs` | MODIFY | Add `"behaviors"` JSON array key to `make_detection()` return value with ioc_type/ioc_value/ioc_source/ioc_description JSON keys (U19: serde_json::Value only) |
| `crates/prism-dtu-crowdstrike/fixtures/detections-detail.json` | MODIFY | Add `behaviors[]` array with same keys in same commit (shape parity test requirement) |
| `crates/prism-dtu-armis/src/generator.rs` (or equivalent) | MODIFY | Project `device_cves_first` scalar (first CVE ID from catalog) onto scenario Armis device records (U17/Ruling 1b) |
| Cyberint sensor TOML spec | MODIFY | Add ioc/iocs[]/alert_data.* column declarations |
| CrowdStrike sensor TOML spec | MODIFY | Add behaviors[].ioc_type/ioc_value/ioc_source/ioc_description column declarations |
| Integration test file (TBD) | CREATE | Tests 7-8: canonical pivot queries against demo server |

---

## Edge Cases

| ID | Source | Description | Expected Behavior |
|----|--------|-------------|-------------------|
| EC-001 | BC-2.06.019 v1.6 §Interim State | `_ioc_value` filter present alongside real-schema filter | FORBIDDEN — not a valid edge case; this is a MUST NOT state per BC-2.06.019 v1.6 |
| EC-002 | BC-2.06.019 v1.6 Per-Sensor IOC-Surface Matrix | IOC stamping attempted on Armis or Claroty records | FORBIDDEN — permanent exclusion; must not occur |
| EC-003 | BC-2.06.019 PC-4 ioc_hashes=false | Cyberint alert has `ioc.value` matching a hash in `catalog.ioc_hashes` | Alert withheld from response |
| EC-004 | BC-2.06.019 PC-4 ioc_ips=false | Cyberint alert has `alert_data.ip` matching `catalog.ioc_ips[0]` | Alert withheld from response |
| EC-005 | BC-2.06.019 PC-4 | Cyberint alert with no IOC fields (None/empty) at any stage | Alert NOT withheld (no matching IOC values to filter on) |
| EC-006 | BC-2.06.019 v1.6 Per-Sensor IOC-Surface Matrix | CrowdStrike host/device record (not detection) — no IOC fields | Device records have NO IOC stamping; IOCs live on detection records only |
| EC-007 | BC-2.06.019 PC-2 table | Pivot query at stage < 3 (device_cves=false, ioc_*=false at stage 0-1) | Result set may be empty; no error |
| EC-008 | BC-2.06.019 PC-2 table | NVD pivot query at stage 3 (device_cves=false) | `device_cves` not visible → `has device_cves` filter returns no rows → empty result (correct behavior; test must use stage 4) |

---

## SAP-2 Compliance Note

Per CLAUDE.md §SAP-2, the adversary for this story MUST:
1. Read `crates/prism-dtu-cyberint/src/types.rs` post-implementation — verify `Alert.ioc`,
   `Alert.iocs`, `Alert.alert_data` fields exist with matching types to TOML spec columns;
   verify TOML wire names match: `ioc.type`/`ioc.value` (NOT `ioc.ioc_type`); `iocs[].type`/`iocs[].value`
   (NOT `iocs[].ioc_type` — serde rename `type` per U22)
2. Read `crates/prism-dtu-crowdstrike/src/generator.rs` `make_detection()` — verify `"behaviors"`
   JSON key is present in the returned serde_json::Value with `ioc_type`/`ioc_value`/`ioc_source`/
   `ioc_description` keys; also read `fixtures/detections-detail.json` — verify same keys present
   (shape parity test compliance). NOTE: NO types.rs Detection/Behavior struct exists in CrowdStrike
   (U19 code grounding 2026-06-12) — adversary MUST NOT look for Behavior struct; it does not exist.
3. Read `crates/prism-dtu-armis/src/generator.rs` — verify `device_cves_first` scalar key is
   present on scenario device records (U17/Ruling 1b)
4. Grep for `_ioc_value` in `crates/prism-dtu-cyberint/src/routes/alerts.rs` — MUST return 0 matches
5. Verify Claroty and Armis types.rs have NO new IOC fields (permanent exclusion per ADR-031)

Column in TOML with no DTU equivalent = **P1 CRITICAL**. `_ioc_value` in route handler post-merge = **P1 CRITICAL** (BPRL-P4-01 recurrence). `Behavior.ioc_type` typed struct added for CrowdStrike = **P1 CRITICAL** (wrong approach — must be JSON key in generator, not a typed struct — U19).

---

## Story Changelog

| Version | Date | Change |
|---------|------|--------|
| v1.3 | 2026-06-12 | Micro-sweep — BC-2.06.019 v1.5→v1.6 pin-sync (BPRL-P6-01 Claroty devices Route Coverage row + exhaustive inventory verification note; POL-23). All body-level BC-2.06.019 v1.5 citations updated to v1.6 (frontmatter comment block, §Narrative, §Architecture Compliance Rules, §Acceptance Criteria AC traces, §Token Budget, §Tasks, §Forbidden Dependencies, §Edge Cases). version 1.2→1.3. |
| v1.2 | 2026-06-12 | Micro-sweep — BC-2.06.019 v1.4→v1.5 pin-sync (BPRL-P5-01 Route Coverage Table corrections + PC-4 5-arg prose; POL-23). All body-level BC-2.06.019 v1.4 citations updated to v1.5 (frontmatter comment block, §Narrative, §Architecture Compliance Rules, §Acceptance Criteria AC traces, §Token Budget, §Tasks, §Forbidden Dependencies, §Edge Cases). version 1.1→1.2. |
| v1.1 | 2026-06-12 | D-1109 remove-uncertainty closure: U1/U17/U19/U20/U21/U22/U23/U24/U25 applied (scanner + research-agent + architect rulings 1-4, WO-D1109 v1.1). enrich syntax → function-call form throughout. CrowdStrike AC-004/AC-005 rewritten: no typed Detection/Behavior structs — IOC fields are JSON keys in generator.rs serde_json::Value records; static fixture parity requirement added; non_exhaustive item for CrowdStrike removed (moot). Route Coverage Table regenerated from actual routers (U20): alerts_search.rs/GET /alerts/queries/alerts/v2 removed (not in CrowdStrike router); GET /xdome/api/v1/alerts replaced with POST /api/v1/alerts (Claroty); CrowdStrike summaries corrected to POST /detects/entities/summaries/GET/v1. Cyberint TOML wire names corrected: iocs[].type (NOT iocs[].ioc_type). Armis device_cves_first generator projection task added (Ruling 1b). PR #185 IN-FLIGHT status noted; materialization-time re-verify task added. verification_properties: [] (BC-2.06.019-internal sub-properties; VP prose citation added to body). axum 0.7 confirmed 2026-06-12 annotation. |
| v1.0 | 2026-06-12 | Initial draft per WO-D1109 §Story 3 and BC-2.06.019 v1.5 §Per-Sensor IOC-Surface Matrix and §Interim State. Root cause closure of BPRL-P4-01. Depends on 002; blocks T11/T13 demo objectives. Full Route Coverage Table included (BC-2.06.019 §Question 4 requirement). 9 Red Gate tests. SAP-2 compliance note included. |
