---
document_type: story
story_id: S-DEMO-ENRICHMENT-PIVOT-003
title: "IOC Stamping (Cyberint + CrowdStrike) and Demo Pivot Query Validation"
wave: 5
epic_id: E-DEMO
priority: P2
status: draft
version: "2.5"
level: "L4"
producer: story-writer
timestamp: "2026-06-12T00:00:00Z"
created: "2026-06-12"
modified: "2026-07-10T00:00:00Z"
tdd_mode: strict
subsystems: [SS-01]
# Subsystem anchor justifications:
#   SS-01 (Sensor Adapters) owns all prism-dtu-* crates per ARCH-INDEX Subsystem Registry.
#   This story adds real-schema IOC fields to prism-dtu-cyberint and prism-dtu-crowdstrike,
#   modifies the Cyberint alerts route to replace the synthetic _ioc_value filter with
#   real-schema field matching, and validates the canonical pivot queries against the
#   demo server. All changes are in SS-01 scope.
target_module: prism-dtu-cyberint
crates_touched: [prism-dtu-cyberint, prism-dtu-crowdstrike, prism-dtu-armis, prism-dtu-demo-server]
behavioral_contracts: [BC-2.06.019, BC-2.06.020]
# BC array propagation:
# BC-2.06.019 governs per-sensor IOC-surface masking (PC-4), the Per-Sensor IOC-Surface
# Matrix, and the Route Coverage Table. This story implements the IOC-stamping scope
# deferred to it by BC-2.06.019 v1.11 §Interim State and §Per-Sensor IOC-Surface Matrix.
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
  # | enrich threat_intel(iocs[].value) and | enrich nvd(device_cves_first) — require
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
# coexist in the same route handler (BC-2.06.019 v1.11 §Interim State). SAP-2 must be
# run on both TOML sensor specs against DTU types.rs before writing any column declaration.
# The Cyberint Alert struct addition must not break existing route handlers or serializers.
# CrowdStrike behaviors[] array is on the Detection object (not on host/device) — do NOT
# confuse the two.
red_gate_tests: 10
estimated_passes: "3-5 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "_ioc_value removal (BC-2.06.019 v1.11 §Interim State): the synthetic _ioc_value /
     _ioc_type filter in crates/prism-dtu-cyberint/src/routes/alerts.rs MUST be removed
     in the SAME commit that adds the real-schema filter. They cannot coexist.
     BC-2.06.019 v1.11 §Interim State clause is explicit: 'The synthetic filter and the
     real-schema filter MUST NOT coexist.' Any implementer attempt to ship the synthetic
     filter as interim is a BC-2.06.019 v1.11 violation and a BPRL-P4-01 recurrence."
  - "CrowdStrike IOC scope (BC-2.06.019 v1.11 Per-Sensor IOC-Surface Matrix): behaviors[]
     carries ioc_type ∈ {hash_sha256, hash_md5, domain, filename, registry_key} — NOT bare
     'hash' (algorithm-qualified only), NOT bare 'registry' (must be 'registry_key'), NOT
     'cmdline' (cmdline is a SEPARATE sibling field behaviors[].cmdline, never an ioc_type
     value), NOT ipv4/ipv6 (IPs only appear on separate custom-IOC / device-query surfaces,
     not on detection behaviors[]). Tolerant-unknown-type policy: treat unknown tokens as
     non-fatal (log + preserve raw string) rather than rejecting, as CrowdStrike publishes
     no normative exhaustive enum. Do NOT stamp ipv4/ipv6 IOC types on detection records."
  - "Armis and Claroty: NO IOC stamping (permanent exclusion per BC-2.06.019 v1.11 matrix).
     Armis alert payloads are reference-only; Claroty has IP addresses only as free text.
     Fabricating IOC fields on these sensors violates ADR-031 DTU=True-DTU fidelity.
     Any implementer attempt to add IOC fields to Armis or Claroty records is a BC violation."
  - "TOML spec alignment (SAP-2): before declaring any new TOML column, run SAP-2 check
     against the DTU types.rs for cyberint and crowdstrike. Every new column MUST have a
     matching field in the DTU Alert or Detection struct after this story's changes."
  - "Route Coverage Table update (BC-2.06.019 v1.11 §Route Coverage Table standing rule):
     after implementing real-schema Cyberint filter, update the Route Coverage Table in
     BC-2.06.019 to change the Cyberint alerts row from INTERIM to ACTIVE. This is a
     MANDATORY same-commit change per the standing rule in BC-2.06.019 v1.11."
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

**BC-2.06.019 v1.13 scope ownership:** This story implements the deferred IOC-stamping
work that BC-2.06.019 assigned to `S-DEMO-ENRICHMENT-PIVOT-003` via the Per-Sensor
IOC-Surface Matrix and the §Interim State clause. The story also owns the Route Coverage
Table update in BC-2.06.019 (Cyberint alerts row INTERIM→ACTIVE; plus four new rows added
in v1.11/v1.12: Armis `device_cves` guard (`devices.rs`, Row 8), CrowdStrike `ioc_hashes`
list-IDs guard (Row 9), CrowdStrike `ioc_hashes` get-summaries guard (Row 10) — all ACTIVE
per F-PIVOT003-R7A-001/R7A-002 / POL-33; plus Row 11 added v1.12 — Armis `device_cves` +
entity-visibility guards on `search.rs` (canonical `armis.devices` query path)
per F-PIVOT003-R8C-001 / POL-33).
BC-2.06.019 v1.13 mandates `iocs[].value` (not singleton `ioc.value`) as the canonical
ThreatIntel pivot field, PC-4 step 6 mandates fail-closed on undeserializable alert
records, and PC-4 §General Filtering Semantics (Ruling 1b) mandates `device_cves_first`
(scalar projection) as the NVD pivot existence filter — the `device_cves` array is NEVER
stamped on generated records and is NOT declared as a TOML column. The authoritative route
for the `armis.devices` table is `GET /api/v1/search` (`src/routes/search.rs`); both
`search.rs` (Row 11) and `devices.rs` (Row 8) carry the `device_cves_first` guard.
The Cyberint alerts real-schema IOC filter (AC-003) checks `iocs[].value` (dual-alias),
`alert_data.ip`/`alert_data.domain`, AND defensively the singleton `ioc.value`; the
singleton check is inert (generator never stamps it) but harmless (BC v1.13 prose-vs-code
coherence fix F-PIVOT003-R10A-002; singleton `Alert.ioc` field remains flagged for removal).

**Sequencing context (D-1109, WO-D1109):** Slots AFTER S-DEMO-ENRICHMENT-PIVOT-002
(infusion specs + plugins operational) and BEFORE T11 and T13 capstone demo segment.

---

## Narrative

As a SOC analyst in the capstone demo (T13), I want to run
`FROM cyberint_alerts | where severity = "high" | enrich threat_intel(iocs[].value) | where threat_is_known_malicious = true`
and see real-schema IOC fields from the Cyberint alert records resolve correctly against
the ThreatIntel enrichment service — not a synthetic injected field — so that the demo
faithfully demonstrates the production enrichment pivot workflow.

---

## Behavioral Contracts

| BC | Title | Key Clauses |
|----|-------|-------------|
| BC-2.06.019 v1.13 | Demo-Server Scenario Progression — Per-Sensor IOC-Surface Masking | PC-4 Per-Sensor IOC-Surface Matrix: Cyberint alerts and CrowdStrike detections carry real-schema IOC fields; Armis/Claroty permanently excluded; `_ioc_value` synthetic filter removed atomically; Route Coverage Table updated. **CrowdStrike ioc_type corrected (v1.8+):** `{hash_sha256, hash_md5, domain, filename, registry_key}` — tolerant-unknown-type policy applies. **Cyberint inner-key INCONCLUSIVE (v1.8+):** serde dual-alias required (`type`/`ioc_type` and `value`/`ioc_value`); singleton `ioc` field flagged for removal. **v1.9:** canonical ThreatIntel pivot targets `iocs[].value` (not singleton `ioc.value`); PC-4 step 6 mandates fail-closed on undeserializable alert records. **v1.10 (Ruling 1b):** `device_cves` array is NEVER stamped on generated records and is NOT declared as a TOML column; only the scalar projection `device_cves_first` is surfaced; canonical NVD pivot query MUST use `has device_cves_first` as existence filter; any reference to `has device_cves` or `nvd_cvss_score` is stale and a P1 finding. **v1.11:** Route Coverage Table Rows 8–10 added — Armis `device_cves` guard (`device_cves_first` field omitted when `!mask.device_cves`); CrowdStrike list-IDs `ioc_hashes` real-schema filter; CrowdStrike get-summaries `ioc_hashes` real-schema filter. All three routes now ACTIVE. Cyberint alerts and CrowdStrike IOC-stamp routes confirmed ACTIVE (post-S-DEMO-ENRICHMENT-PIVOT-003 state). **v1.12 (F-PIVOT003-R8C-001):** Route Coverage Table Row 11 added — `device_cves, primary_device, lateral_devices` guard on `src/routes/search.rs` (device branch, `GET /api/v1/search?aql=...`). The `search.rs` route is the authoritative path for `armis.devices` sensor-spec `path_template`; `/api/v1/devices` (`devices.rs`, Row 8) is a secondary route. Both routes carry the `device_cves_first` guard; `search.rs` is canonical for the `from armis.devices` PrismQL query. **v1.13 (F-PIVOT003-R10A-002):** Prose-vs-code coherence fix — Cyberint alerts route guard correctly checks `iocs[].value` (dual-alias), `alert_data.ip`/`alert_data.domain`, AND defensively the singleton `ioc.value` (consistent with AC-003 `ioc_values_for` implementation). The singleton check is inert (generator never stamps it) but harmless; singleton `Alert.ioc` field remains flagged for removal. |
| BC-2.06.020 v1.6 | Demo-Server Enrichment Correlation — Scenario IOCs/CVEs Resolve in ThreatIntel/NVD; Cyberint Alert CVEs Use Catalog IDs (Collision-Safe in All Modes) | INV-THREATINTEL-IOC-CORRELATION-001: scenario IOCs in ScenarioEntityCatalog resolve as Malicious in ThreatIntel; INV-CROSS-DTU-ENTITY-COHERENCE-001: entity IDs coherent across DTU clones; INV-CYBERINT-ALERT-CVE-CORRELATION-001: Cyberint CVE records use catalog IDs in scenario mode, CVE-9999- namespace in all modes |

**VP Citation (U24):** VP-019-A (pure function reproducibility), VP-019-B (stage monotonicity),
VP-019-C (StageMask completeness), VP-019-G, VP-019-H are BC-2.06.019-internal sub-properties
with no standalone VP-INDEX files. `verification_properties: []` is set in frontmatter to avoid
tooling resolution failure and collision with `vp-019-diff-deterministic.md`. These
BC-2.06.019-internal sub-properties must not regress when Cyberint alerts route and CrowdStrike
generator are modified.

---

## Acceptance Criteria

### AC-001 — Cyberint Alert struct adds real-schema IOC fields with serde dual-alias
(traces to BC-2.06.019 v1.11 postcondition 4 — Per-Sensor IOC-Surface Matrix, Cyberint row; inner-key dual-alias requirement)

Given `crates/prism-dtu-cyberint/src/types.rs`,
when the `Alert` struct is inspected after this story,
then it includes:
- `iocs: Vec<Ioc>` (CONFIRMED — list of IOCs, plural-form, per Check Point sk182975)
- `alert_data: Option<AlertData>` where `AlertData { ip: Option<String>, domain: Option<String>, url: Option<String> }`
  (`url` CONFIRMED; `ip`/`domain` plausible-unconfirmed per BC-2.06.019 v1.11)
- `ioc: Option<Ioc>` — RETAINED for now but flagged for removal: no public-documentation
  basis found for a singleton top-level `ioc` field in the real Cyberint API
  (BC-2.06.019 v1.11 Cyberint row — "UNCONFIRMED, flagged for likely removal"). Confirm via
  live-tenant validation before removing. If a live tenant never returns this field, remove it.

The `Ioc` struct MUST use serde dual-alias to tolerate both documented Cyberint naming conventions
(inner-key forms are INCONCLUSIVE-pending-live-tenant-validation per BC-2.06.019 v1.11):
```rust
pub struct Ioc {
    #[serde(rename = "type", alias = "ioc_type")]
    pub ioc_type: String,   // tolerates "type" (short form) and "ioc_type" (feed convention)
    #[serde(alias = "ioc_value")]
    pub value: String,      // tolerates "value" (short form) and "ioc_value" (feed convention)
}
```

This dual-alias approach ensures the DTU parser is robust against whichever key convention the
live Cyberint API uses for `iocs[]` elements, avoiding hard-coded bet on unverified inner keys
(BC-2.06.019 v1.11 research: uncertainty-pivot003-s504-2026-06-19.md §Item 2).

Red Gate: `test_BC_2_06_019_cyberint_alert_struct_has_real_ioc_fields`
Red Gate: `test_BC_2_06_019_cyberint_ioc_struct_dual_alias_deserializes_both_key_forms`

### AC-002 — Cyberint fixture generator stamps scenario IOCs onto alert records
(traces to BC-2.06.019 v1.11 postcondition 4 — IOC-surface fields populated from catalog)

Given a Cyberint clone constructed with `new_with_scenario(catalog: &ScenarioEntityCatalog)`
where `catalog` contains `ioc_ips`, `ioc_domains`, `ioc_hashes` entries,
when the fixture generator runs,
then scenario-enabled alert records in the `FixtureSet` have their `iocs[0].value`
field set to a value from the catalog's IOC lists, per the real Cyberint API schema.

NOTE: `CywberintClone::new_with_scenario` now takes a `catalog: &ScenarioEntityCatalog`
argument (introduced to wire scenario IOC/CVE stamping through the production clone path —
R2 adversary closure). The catalog is passed at construction time and held by the clone
for use during fixture generation. Any call site referencing `new_with_scenario` without
a `catalog` argument must be updated (behavioral anchor: `CyberintClone::new_with_scenario`,
not file:line per TD-VSDD-091).

NOTE: The `Ioc.value` field is used for fixture generation output (serialized as `"value"` in
JSON via the default serde field name); the dual-alias in AC-001 applies to deserialization
(reading from live Cyberint API responses), not to fixture generation output.

Red Gate: `test_BC_2_06_019_cyberint_fixture_generator_stamps_scenario_iocs`

### AC-003 — Cyberint alerts route: _ioc_value synthetic filter REMOVED; real-schema filter ADDED (atomic)
(traces to BC-2.06.019 v1.11 postcondition 4 §Interim State — synthetic filter must be replaced atomically)

Given `crates/prism-dtu-cyberint/src/routes/alerts.rs` after this story:
- The `_ioc_value` / `_ioc_type` synthetic field filter is ABSENT (grep for `_ioc_value` returns
  no matches in the file)
- A new filter reads real-schema fields via the `Ioc.value` accessor (which is
  the deserialized form regardless of whether the wire key was `"value"` or `"ioc_value"` —
  the dual-alias from AC-001 handles both at deserialization time):
  `rec.ioc.as_ref().map(|i| &i.value)`, `rec.iocs[].value`, `rec.alert_data.ip`, `rec.alert_data.domain`
- When `ioc_hashes=false`: alert records where any IOC field matches a value in
  `catalog.ioc_hashes` are withheld from the response
- When `ioc_ips=false`: alert records where `ioc.value` (if present), `iocs[].value`, or
  `alert_data.ip` matches `catalog.ioc_ips` values are withheld
- When `ioc_domains=false`: alert records where `ioc.value` (if present), `iocs[].value`, or
  `alert_data.domain` matches `catalog.ioc_domains` values are withheld

INVARIANT: The synthetic filter and the real-schema filter MUST NOT coexist in the same
route handler. This is verified by grep: `grep -n '_ioc_value' crates/prism-dtu-cyberint/src/routes/alerts.rs`
MUST return 0 matches after this story merges.

Red Gate: `test_BC_2_06_019_cyberint_alerts_real_schema_ioc_filter_no_synthetic`

### AC-004 — CrowdStrike detection generator stamps behaviors[].ioc_type/ioc_value in JSON records
(traces to BC-2.06.019 v1.11 postcondition 4 — CrowdStrike detections row in Per-Sensor IOC-Surface Matrix; corrected ioc_type enum)

NOTE (U19): CrowdStrike has NO typed `Detection` or `Behavior` structs in types.rs. All
detection records are untyped `serde_json::Value` built by `generator.rs` from
`fixtures/detections-detail.json` with a `behaviors[]` JSON array. The generator
`make_detection()` function currently produces NO `behaviors` key. This AC adds it.

Given a CrowdStrike clone constructed with `new_with_scenario(catalog: &ScenarioEntityCatalog)`
where `catalog` contains `ioc_hashes` entries,

NOTE: `CrowdstrikeClone::new_with_scenario` now takes a `catalog: &ScenarioEntityCatalog`
argument (same pattern as Cyberint — R2 adversary closure, catalog-threaded signature
for production clone path). Behavioral anchor: `CrowdstrikeClone::new_with_scenario`
(not file:line per TD-VSDD-091).
when the fixture generator (`make_detection()` in `src/generator.rs`) populates scenario
detection records,
then the generated `serde_json::Value` detection records include a `"behaviors"` JSON array
key with at least one entry containing:
- `"ioc_type": "hash_sha256"` — algorithm-qualified token per real CrowdStrike API
  (BC-2.06.019 v1.11 correction — bare `"hash"` is WRONG; real API uses algorithm-qualified
  tokens `hash_sha256` and `hash_md5` per ThreatQ CrowdStrike Insight EDR CDF + XSOAR Falcon
  integration docs; source: uncertainty-pivot003-s504-2026-06-19.md §Item 1)
- `"ioc_value": "<catalog.ioc_hashes[0]>"`
- `"ioc_source": "catalog"`
- `"ioc_description": "scenario IOC"`

**CrowdStrike ioc_type enum (CORRECTED in v1.8, confirmed v1.9):**
The stamped `ioc_type` values MUST use the algorithm-qualified / `_key`-suffixed tokens from
the real Falcon API. Valid values: `hash_sha256`, `hash_md5`, `domain`, `filename`,
`registry_key`. FORBIDDEN values: bare `hash` (not emitted by real API), bare `registry`
(must be `registry_key`), `cmdline` (NOT an ioc_type value — it is a SEPARATE sibling behavior
field `behaviors[].cmdline`), `ipv4`, `ipv6` (not present on detection behaviors[]).

**Tolerant-unknown-type policy:** The DTU clone parser MUST treat unknown `ioc_type` tokens
as non-fatal: log the unexpected token (debug level) and preserve the raw string rather than
returning a parse error. CrowdStrike publishes no normative exhaustive enum for
`behaviors[].ioc_type`; there may be undocumented or licence-gated types. A hard parse failure
on unknown tokens would break the clone if new types appear.

**Red Gate test expectation update (v1.9 correction):** The Red Gate test
`test_BC_2_06_019_crowdstrike_detection_behaviors_ioc_hash_stamped` MUST assert:
- `behaviors[0]["ioc_type"] == "hash_sha256"` (NOT `"hash"`)
- Any fixture in `fixtures/detections-detail.json` that previously used `"ioc_type": "hash"`
  must be updated to `"ioc_type": "hash_sha256"` in the same commit.

GENERATOR SHAPE PARITY: Per the `make_detection()` doc comment and `review_2026_06_10_cs_parity.rs`
test, the flat scalar key set of generated detection records MUST equal the key set of static
fixtures (`fixtures/detections-detail.json`). Adding `behaviors[]` to generated records requires
the same addition to the static fixture JSON in the same commit.

Red Gate: `test_BC_2_06_019_crowdstrike_detection_behaviors_ioc_hash_stamped`

### AC-005 — CrowdStrike detections TOML spec declares behaviors[] IOC columns matching generator JSON shape
(traces to BC-2.06.019 v1.11 postcondition 4 — TOML spec alignment with real-schema fields)

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
(traces to BC-2.06.019 v1.11 postcondition 4 — TOML spec alignment with real-schema fields)

Given the Cyberint sensor TOML spec,
when the spec is inspected after this story,
then it declares columns using the primary wire names that the Cyberint API is most likely
to emit (INCONCLUSIVE-pending-live-validation per BC-2.06.019 v1.11 — dual-alias toleration
handles both forms at runtime):

`ioc.type`, `ioc.value`, `iocs[].type`, `iocs[].value`,
`alert_data.ip`, `alert_data.domain`, `alert_data.url`.

NOTE: `iocs[].type` is the SHORT-FORM wire key, declared as the primary column name because
the `Ioc` struct uses `#[serde(rename = "type", alias = "ioc_type")]`. The dual-alias
means the DTU can deserialize either `"type"` or `"ioc_type"` at runtime; the TOML column
name tracks the primary wire key (`type`). The TOML spec does NOT need a separate
`iocs[].ioc_type` column — the serde alias handles that transparently.

TOML column MUST NOT declare `iocs[].ioc_type` as a separate column (alias is resolved at
the Rust struct level, not at the TOML schema level). If live-tenant validation later
confirms the wire key is `ioc_type`, the TOML column is renamed from `iocs[].type` to
`iocs[].ioc_type` in a follow-on spec update.

Each declared column has a matching field in the Cyberint DTU types.rs Alert struct
after this story's additions (SAP-2 compliance).

Red Gate: `test_BC_2_06_019_cyberint_alert_toml_spec_has_ioc_columns` (or SAP-2 parity assertion)

### AC-007 — Canonical ThreatIntel pivot query returns Malicious results at stage >= 3
(traces to BC-2.06.019 v1.11 postcondition 4 + BC-2.06.020 INV-THREATINTEL-IOC-CORRELATION-001)

Given demo server at stage >= 3 (Exfil; `ioc_ips`, `ioc_domains`, `ioc_hashes` visible),
when the following canonical query executes:
```prismql
FROM cyberint_alerts
| where severity = "high"
| enrich threat_intel(iocs[].value)
| where threat_is_known_malicious = true
| sort threat_score desc
| head 10
```
then the result set is non-empty and all returned records have
`threat_is_known_malicious = true` with `threat_score >= 75`.

NOTE (BC-2.06.019 v1.11): The canonical pivot field is `iocs[].value` (list form — plural
`iocs` array). The singleton `ioc.value` form was the v1.8 interim target; v1.9 mandates
the plural list form to match the real Cyberint API schema (`iocs: Vec<Ioc>`). Any test
or demo script still referencing `enrich threat_intel(ioc.value)` must be updated to
`enrich threat_intel(iocs[].value)`. PC-4 step 6 additionally mandates fail-closed
behavior on undeserializable alert records — the route handler MUST NOT silently drop
or skip records that fail to deserialize; they MUST produce an error.

Red Gate: `test_BC_2_06_019_canonical_threatintel_pivot_query_returns_malicious_at_stage_3`

### AC-008 — Canonical NVD pivot query returns HIGH CVSS results at stage 4 (Containment)
(traces to BC-2.06.019 v1.11 postcondition 4 + BC-2.06.020 INV-NVD-CVE-CORRELATION-001)

Given demo server at stage 4 (Containment; `device_cves=true` in StageMask per
BC-2.06.019 v1.11 PC-2 table — `device_cves_first` scalar is only present on device
records when `device_cves=true`),
when the following canonical query executes:
```prismql
from armis.devices
| where has device_cves_first
| enrich nvd(device_cves_first)
| where cvss_base_score >= 7.0
| sort cvss_base_score desc
```
then the result set is non-empty and all returned records have `cvss_base_score >= 7.0`.

NOTE (BC-2.06.019 v1.11 Ruling 1b): The existence filter MUST use `has device_cves_first`
(scalar String field — first CVE ID from `catalog.device_cves`). The `device_cves` array
field is NEVER stamped on generated records and is NOT declared as a TOML column; any
query using `has device_cves` as the existence filter returns zero rows. The CVSS filter
MUST use `cvss_base_score >= 7.0` — NOT `nvd_cvss_score` (stale column name), NOT strict
`>` inequality (canonical form is `>=`). Both `has device_cves` and `nvd_cvss_score` are
stale/forbidden forms and MUST be treated as P1 findings in adversarial review.

Per BC-2.06.019 PC-2 table, `device_cves = false` at stages 0-3 and `true` only at
stage 4 (Containment). The test harness MUST configure `scenario_start_secs` to a past
epoch that places the demo server at stage 4 — not stage 3 — for this integration test.

Red Gate: `test_BC_2_06_019_canonical_nvd_pivot_query_returns_high_cvss_at_containment_stage`

### AC-009 — BC-2.06.019 Route Coverage Table updated: 4 new rows (v1.11 Rows 8–10 + v1.12 Row 11) + Cyberint alerts ACTIVE
(traces to BC-2.06.019 v1.13 §Route Coverage Table standing rule)

Given `BC-2.06.019-demo-server-scenario-progression.md` after this story (v1.13),
when the Route Coverage Table is inspected,
then:
1. The Cyberint alerts row shows:
   - Guard Mechanism: real-schema filter checks `iocs[].value` (dual-alias: `value` or
     `ioc_value`), `alert_data.ip`/`alert_data.domain`, AND defensively the singleton
     `ioc.value` — the singleton check is inert (scenario generator never stamps the
     singleton `ioc` field; only `iocs[]` is populated) but harmless per AC-003
     `ioc_values_for` implementation (BC-2.06.019 v1.13 prose-vs-code coherence fix,
     F-PIVOT003-R10A-002); singleton `ioc.value` is not the canonical pivot target
     (`iocs[].value` is canonical per v1.10+; singleton `Alert.ioc` remains flagged for
     removal)
   - Status: `ACTIVE` (NOT `INTERIM`)
2. Three new Route Coverage Table rows are present (added v1.11, per F-PIVOT003-R7A-001/R7A-002):

   | StageMask Field | DTU Clone | Route File | Route Path | Guard Mechanism | Status |
   |-----------------|-----------|------------|------------|-----------------|--------|
   | `device_cves` | prism-dtu-armis | `src/routes/devices.rs` | `GET /api/v1/devices` | Per-record `device_cves_first` field omitted (`obj.remove("device_cves_first")`) when `!mask.device_cves`; applied in the `paginate_devices` scenario path | ACTIVE (added S-DEMO-ENRICHMENT-PIVOT-003) |
   | `ioc_hashes` | prism-dtu-crowdstrike | `src/routes/detections.rs` | `GET /detects/queries/detects/v1` | Detection withheld (filter_map returns None) when any `behaviors[].ioc_value` ∈ `catalog.ioc_hashes` AND `!mask.ioc_hashes`; list-ID scenario path | ACTIVE (added S-DEMO-ENRICHMENT-PIVOT-003) |
   | `ioc_hashes` | prism-dtu-crowdstrike | `src/routes/detections.rs` | `POST /detects/entities/summaries/GET/v1` | Detection withheld (filter_map returns None) when any `behaviors[].ioc_value` ∈ `catalog.ioc_hashes` AND `!mask.ioc_hashes`; get-summaries scenario path | ACTIVE (added S-DEMO-ENRICHMENT-PIVOT-003) |

3. One additional Row (Row 11) is present (added v1.12, F-PIVOT003-R8C-001 / POL-33):

   | StageMask Field | DTU Clone | Route File | Route Path | Guard Mechanism | Status |
   |-----------------|-----------|------------|------------|-----------------|--------|
   | `device_cves`, `primary_device`, `lateral_devices` | prism-dtu-armis | `src/routes/search.rs` | `GET /api/v1/search?aql=...` (device branch, scenario path; this is the route `armis.devices` sensor-spec `path_template` targets — canonical path for `from armis.devices` PrismQL queries) | Per-record `device_cves_first` omitted (`obj.remove("device_cves_first")`) when `!mask.device_cves` (device branch); entity-visibility guards: `primary_device` and `lateral_devices` applied on device-branch records per-stage | ACTIVE (added S-DEMO-ENRICHMENT-PIVOT-003, F-PIVOT003-R8C-001) |

   NOTE (v1.12 clarification): `/api/v1/devices` (`paginate_devices` in `src/routes/devices.rs`,
   Row 8) is a secondary route that also carries the `device_cves_first` guard. It is NOT the
   authoritative route for the `armis.devices` table. The canonical analyst query path
   (`from armis.devices`) targets `GET /api/v1/search` via the sensor-spec `path_template`.
   Both routes are guarded; `search.rs` (Row 11) is authoritative.

This update is required in the SAME commit as AC-003 per the Route Coverage Table
standing rule (BC-2.06.019 v1.12: "any future story adding or modifying a StageMask-relevant
route MUST extend or update this table in the same commit").

NOTE: BC-2.06.019 is a `.factory/` artifact — it is edited via Write/Edit tools by
state-manager at post-merge burst. The implementer notes this requirement in the PR;
the state-manager burst applies it.

Red Gate: N/A (process check — adversary verifies BC Route Coverage Table status post-merge)

---

## Red Gate Test Plan

| # | Test Name | Crate | BC Clause | Type |
|---|-----------|-------|-----------|------|
| 1 | `test_BC_2_06_019_cyberint_alert_struct_has_real_ioc_fields` | prism-dtu-cyberint | BC-2.06.019 v1.11 PC-4 matrix | unit |
| 2 | `test_BC_2_06_019_cyberint_ioc_struct_dual_alias_deserializes_both_key_forms` | prism-dtu-cyberint | BC-2.06.019 v1.11 PC-4 Cyberint row — INCONCLUSIVE inner-key dual-alias | unit |
| 3 | `test_BC_2_06_019_cyberint_fixture_generator_stamps_scenario_iocs` | prism-dtu-cyberint | BC-2.06.019 v1.11 PC-4 / BC-2.06.020 PC-1 | unit |
| 4 | `test_BC_2_06_019_cyberint_alerts_real_schema_ioc_filter_no_synthetic` | prism-dtu-cyberint | BC-2.06.019 v1.11 PC-4 §Interim State | unit |
| 5 | `test_BC_2_06_019_crowdstrike_detection_behaviors_ioc_hash_stamped` | prism-dtu-crowdstrike | BC-2.06.019 v1.11 PC-4 matrix — asserts `ioc_type == "hash_sha256"` (corrected) | unit |
| 6 | `test_BC_2_06_019_crowdstrike_detection_toml_spec_has_ioc_columns` | prism-dtu-crowdstrike or sensor spec tests | BC-2.06.019 v1.11 PC-4 + SAP-2 | unit/parity |
| 7 | `test_BC_2_06_019_cyberint_alert_toml_spec_has_ioc_columns` | prism-dtu-cyberint or sensor spec tests | BC-2.06.019 v1.11 PC-4 + SAP-2 | unit/parity |
| 8 | `test_BC_2_06_019_canonical_threatintel_pivot_query_returns_malicious_at_stage_3` | prism-query or prism-bin integration | BC-2.06.019 v1.11 PC-4 + BC-2.06.020 INV-THREATINTEL-IOC-CORRELATION-001 | integration (demo server) |
| 9 | `test_BC_2_06_019_canonical_nvd_pivot_query_returns_high_cvss_at_containment_stage` | prism-query or prism-bin integration | BC-2.06.019 v1.11 PC-4 + BC-2.06.020 INV-NVD-CVE-CORRELATION-001 — query uses `has device_cves_first` (Ruling 1b); `cvss_base_score >= 7.0` | integration (demo server) |
| 10 | `test_BC_2_06_019_ioc_hashes_false_withholds_cyberint_alert_with_matching_hash` | prism-dtu-cyberint | BC-2.06.019 v1.11 PC-4 ioc_hashes=false filtering | unit |

NOTE: Red Gate count is 10 (bumped from 9 at v1.9 — test #2 for dual-alias deserialization added).
Frontmatter `red_gate_tests: 10` already reflects this; no further update required.

---

## Route Coverage Table (for StageMask IOC fields — per BC-2.06.019 v1.13 §Route Coverage Table)

This table must be kept in sync with BC-2.06.019 §Route Coverage Table. After this story
ships (v1.13 state), all rows are ACTIVE. Three new rows (Rows 8–10 in BC v1.11) and one
additional row (Row 11 in BC v1.12) were added by S-DEMO-ENRICHMENT-PIVOT-003 via
POL-33 + F-PIVOT003-R7A-001/R7A-002 (v1.11) and F-PIVOT003-R8C-001 (v1.12).
BC v1.13 (F-PIVOT003-R10A-002) corrected the Cyberint alerts Row 6 guard description to
accurately reflect that the implemented `ioc_values_for` function defensively includes the
singleton `ioc.value` check alongside `iocs[].value` and `alert_data.ip`/`alert_data.domain`.

NOTE (U20, confirmed 2026-06-12 from actual routers in 001-B worktree): regenerated from
real route registrations. Rows referencing non-existent routes removed; correct paths inserted.

| StageMask Field | DTU Clone | Route File | Route Path | Guard Mechanism | Status after 003 |
|-----------------|-----------|------------|------------|-----------------|-----------------|
| `ioc_hashes`, `ioc_ips`, `ioc_domains` | prism-dtu-cyberint | `routes/alerts.rs` | `GET /api/v1/alerts` (confirmed clone.rs) | Real-schema filter: checks `iocs[].value` (dual-alias: `value` or `ioc_value`), `alert_data.ip`/`alert_data.domain`, AND defensively the singleton `ioc.value` via `ioc_values_for` helper (replaces `_ioc_value` synthetic filter; singleton check is inert — generator never stamps it — but harmless per AC-003 + BC-2.06.019 v1.13 F-PIVOT003-R10A-002; canonical pivot field is `iocs[].value`) | ACTIVE |
| `primary_device`, `lateral_devices` | prism-dtu-armis | `routes/` | `GET /api/v1/devices` + `GET /api/v1/search` + `GET /api/v1/alerts` (confirmed clone.rs) | Stage index filter | ACTIVE (unchanged) |
| `primary_device`, `lateral_devices` | prism-dtu-crowdstrike | `routes/hosts.rs` | `GET /devices/queries/devices/v1` + `POST /devices/entities/devices/v2` (spec-driven step 2 — ratified POST per DEFECT-CSDEVICES-EMPTY-PIPELINE-001 2026-07-10; body `{"ids": [...]}`; GET route preserved at same path but no longer the spec-driven path) | Stage index filter | ACTIVE (unchanged) |
| (no IOC surface) | prism-dtu-claroty | `routes/alerts.rs` | `POST /api/v1/alerts` (confirmed clone.rs) | EXEMPT — permanent (no structured IOC fields in real Claroty API) | PERMANENT EXEMPT |
| `device_cves` | prism-dtu-armis | `src/routes/devices.rs` | `GET /api/v1/devices` | Per-record `device_cves_first` field omitted (`obj.remove("device_cves_first")`) when `!mask.device_cves`; applied in the `paginate_devices` scenario path (secondary route — NOT authoritative for `armis.devices` table; see Row 11 below) | ACTIVE (added S-DEMO-ENRICHMENT-PIVOT-003, BC v1.11 Row 8) |
| `ioc_hashes` | prism-dtu-crowdstrike | `src/routes/detections.rs` | `GET /detects/queries/detects/v1` | Detection withheld (filter_map returns None) when any `behaviors[].ioc_value` ∈ `catalog.ioc_hashes` AND `!mask.ioc_hashes`; list-ID scenario path | ACTIVE (added S-DEMO-ENRICHMENT-PIVOT-003, BC v1.11 Row 9) |
| `ioc_hashes` | prism-dtu-crowdstrike | `src/routes/detections.rs` | `POST /detects/entities/summaries/GET/v1` | Detection withheld (filter_map returns None) when any `behaviors[].ioc_value` ∈ `catalog.ioc_hashes` AND `!mask.ioc_hashes`; get-summaries scenario path | ACTIVE (added S-DEMO-ENRICHMENT-PIVOT-003, BC v1.11 Row 10) |
| `device_cves`, `primary_device`, `lateral_devices` | prism-dtu-armis | `src/routes/search.rs` | `GET /api/v1/search?aql=...` (device branch, scenario path; this is the route `armis.devices` sensor-spec `path_template` targets — the canonical path for `from armis.devices` PrismQL queries) | Per-record `device_cves_first` omitted (`obj.remove("device_cves_first")`) when `!mask.device_cves` (device branch); entity-visibility guards: `primary_device` and `lateral_devices` applied on device-branch records per-stage | ACTIVE (added S-DEMO-ENRICHMENT-PIVOT-003, BC v1.12 Row 11, F-PIVOT003-R8C-001) |

REMOVED ROWS (U20 — routes do not exist in actual router):
- `routes/alerts_search.rs` / `GET /alerts/queries/alerts/v2`: NO such route file or path in
  prism-dtu-crowdstrike (CrowdStrike router has: oauth, detections, hosts, writes — no alerts_search module)
- `GET /xdome/api/v1/alerts`: Claroty serves `POST /api/v1/alerts` not a GET at /xdome/ prefix

---

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~5,000 |
| BC-2.06.019 v1.13 (full — authoritative for IOC matrix, Route Coverage Table Rows 1–11, corrected CrowdStrike ioc_type + Cyberint dual-alias + `iocs[].value` canonical pivot + fail-closed PC-4 step 6 + Ruling 1b `device_cves_first` scalar NVD pivot + post-S-DEMO-ENRICHMENT-PIVOT-003 ACTIVE state all routes; Row 11 adds `device_cves`+entity-visibility guards on `search.rs`, F-PIVOT003-R8C-001; v1.13 Row 6 guard description corrected to include defensive singleton `ioc.value` check, F-PIVOT003-R10A-002) | ~7,500 |
| `prism-dtu-crowdstrike/src/generator.rs` make_detection() function + fixtures/detections-detail.json (U19) | ~800 |
| `prism-dtu-armis/src/generator.rs` (device_cves_first projection, U17/Ruling 1b) | ~600 |
| BC-2.06.020 v1.6 (enrichment correlation context) | ~3,600 |
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

- [x] MATERIALIZATION-TIME RE-VERIFY (U23 — RESOLVED 2026-06-19): PR #185
  (S-DEMO-DTU-LIVE-SCENARIO-001-B) MERGED 2026-06-13 (develop@7fd35b77). Field names
  CONFIRMED on develop:
  - `ioc_ips` / `ioc_domains` / `ioc_hashes` — `ScenarioEntityCatalog` fields
    (crates/prism-dtu-common/src/scenario/mod.rs)
  - `_ioc_value` / `_ioc_type` — synthetic-mask fields injected by injection tests only
    (crates/prism-dtu-cyberint/src/routes/alerts.rs — confirmed present, to be removed by AC-003)
  No field name discrepancies found. Implementation may proceed using the names above.
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
- [ ] Read BC-2.06.019 v1.13 §Per-Sensor IOC-Surface Matrix and §Route Coverage Table — both
  are authoritative for this story's scope (v1.13 has corrected CrowdStrike ioc_type enum,
  Cyberint dual-alias requirement, `iocs[].value` canonical pivot, fail-closed PC-4 step 6,
  Ruling 1b: `device_cves_first` scalar is the only NVD pivot field; `device_cves` array
  is NEVER stamped on generated records; Rows 8–10 enumerate Armis device_cves and CrowdStrike
  ioc_hashes list/summaries guards; Row 11 adds `device_cves`+entity-visibility guards on
  `src/routes/search.rs` — the canonical `armis.devices` query path per F-PIVOT003-R8C-001;
  v1.13 Row 6 corrects guard description to include defensive singleton `ioc.value` check
  in `ioc_values_for` per F-PIVOT003-R10A-002)

**Phase 1: Cyberint Alert struct + fixture generator**

- [ ] Write failing test 1 (FAIL first): `test_BC_2_06_019_cyberint_alert_struct_has_real_ioc_fields`
- [ ] Write failing test (FAIL first): `test_BC_2_06_019_cyberint_ioc_struct_dual_alias_deserializes_both_key_forms`
  (test must deserialize `{"type": "domain", "value": "evil.example.com"}` AND
  `{"ioc_type": "domain", "ioc_value": "evil.example.com"}` — both MUST yield the same `Ioc` value)
- [ ] Add `Ioc` struct to `types.rs` (or reuse from ThreatItem if already has dual-alias) with
  MANDATORY dual-alias serde annotations (BC-2.06.019 v1.11 INCONCLUSIVE inner-key requirement):
  ```rust
  pub struct Ioc {
      #[serde(rename = "type", alias = "ioc_type")]
      pub ioc_type: String,
      #[serde(alias = "ioc_value")]
      pub value: String,
  }
  ```
- [ ] Add `AlertData { ip: Option<String>, domain: Option<String>, url: Option<String> }` struct
- [ ] Add to `Alert` struct: `ioc: Option<Ioc>` (flagged for removal — retain pending live-tenant
  validation), `iocs: Vec<Ioc>`, `alert_data: Option<AlertData>`
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
  for scenario-enabled detection records, add `"behaviors"` key to the JSON object with at least one entry
  using the CORRECTED algorithm-qualified ioc_type token (BC-2.06.019 v1.11 correction):
  `json!([{ "ioc_type": "hash_sha256", "ioc_value": catalog.ioc_hashes[0], "ioc_source": "catalog", "ioc_description": "scenario IOC" }])`
  DO NOT use `"ioc_type": "hash"` (bare, incorrect per real CrowdStrike API)
- [ ] SHAPE PARITY: update `fixtures/detections-detail.json` in the SAME commit to add the
  `behaviors` key (review_2026_06_10_cs_parity.rs shape parity test will fail otherwise)
- [ ] Verify test 4 passes

**Phase 3b: Armis device_cves_first generator projection (U17/Ruling 1b)**

- [ ] Write failing test: `test_BC_2_06_019_armis_device_cves_first_scalar_projected`
- [ ] Modify the Armis fixture generator: for scenario-enabled device records at stage 4
  (when `device_cves=true` in StageMask per BC-2.06.019 v1.11 PC-2), add scalar field
  `"device_cves_first": catalog.device_cves[0]` (String — first CVE ID from catalog list).
  NOTE (Ruling 1b): do NOT add a `device_cves` array field — only the scalar
  `device_cves_first` is surfaced; the array is never stamped on generated records.
  This enables `| where has device_cves_first | enrich nvd(device_cves_first)` correctly.
- [ ] SHAPE PARITY: if Armis has a static fixture file for devices, update it with the new field
  in the same commit (check for analogous parity test in prism-dtu-armis tests)
- [ ] Verify test passes

**Phase 4: TOML sensor spec alignment**

- [ ] Write failing tests 5, 6 (FAIL first): TOML spec parity tests
- [ ] Update Cyberint sensor TOML spec: add `ioc.type`, `ioc.value`, `iocs[].type`,
  `iocs[].value`, `alert_data.ip`, `alert_data.domain`, `alert_data.url` columns
  (NOTE: use `iocs[].type` as the primary TOML column name — matches the `#[serde(rename = "type")]`
  primary key; dual-alias `ioc_type` is resolved at the serde level, not in TOML column declarations)
- [ ] Update CrowdStrike detections TOML spec: add `behaviors[].ioc_type`,
  `behaviors[].ioc_value`, `behaviors[].ioc_source`, `behaviors[].ioc_description` columns
- [ ] SAP-2 post-edit verification: for each new TOML column, confirm matching DTU struct field
- [ ] Verify tests 5, 6 pass

**Phase 5: Canonical pivot query integration tests**

- [ ] Write failing tests 7, 8 (FAIL first): canonical pivot queries against demo server
- [ ] Configure demo server integration test harness at stage >= 3 for ThreatIntel test
  (set `scenario_start_secs` to past epoch so stage 3 is active at test time)
- [ ] Configure at stage 4 (Containment) for NVD test: `device_cves_first` scalar only
  present at stage 4 per BC-2.06.019 v1.11 PC-2 StageMask table; query uses
  `has device_cves_first` as existence filter (NOT `has device_cves` — stale/forbidden)
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
- [ ] Run `just check` — all 10 Red Gate tests pass; zero clippy warnings; fmt clean

---

## Previous Story Intelligence

**S-DEMO-ENRICHMENT-PIVOT-002 (direct predecessor):**
- `threatintel.infusion.toml` and `nvd.infusion.toml` are loaded and their UDFs registered
- `| enrich threat_intel(ioc_value)` pipe stage is operational (using interim field name from 002)
- This story changes the canonical ThreatIntel pivot from interim `ioc_value` to `iocs[].value`
  (plural list form per BC-2.06.019 v1.11 canonical mandate — `ioc.value` singleton was the
  v1.8 interim target; v1.9 supersedes it with `iocs[].value`; v1.10 additionally mandates
  `has device_cves_first` as NVD pivot existence filter per Ruling 1b)

**S-DEMO-DTU-LIVE-SCENARIO-001-B (substrate — PR #185 MERGED 2026-06-13, develop@7fd35b77):**
Field names confirmed on develop (U23 resolved 2026-06-19):

- `ScenarioEntityCatalog.ioc_ips`, `ioc_domains`, `ioc_hashes` — confirmed in
  `crates/prism-dtu-common/src/scenario/mod.rs`
- `ThreatIntelClone::new_with_scenario` pre-populates `fixture_registry` with all catalog IOCs
  as `FixtureKey::Malicious`
- Cyberint `_ioc_value` / `_ioc_type` synthetic filter — confirmed present in
  `crates/prism-dtu-cyberint/src/routes/alerts.rs`; matched only by injection tests,
  not by real-schema alert records (no `_ioc_value` field in real Cyberint API)
- CrowdStrike detections route has `stage_idx > 0` guard from commit bc0f36c5 but no IOC-field
  stamping on `behaviors[]` yet — that is this story's scope

**From PLUGIN-MIGRATION-001-D/E lessons:**
- SAP-2: adversary reads DTU types.rs and routes/ — do not rely on story description alone
- SID-1: integration tests driving in-process demo server are NOT `#[ignore]`'d

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| `_ioc_value` and real-schema filter MUST NOT coexist in the same route handler | BC-2.06.019 v1.11 §Interim State | AC-003 + grep verification |
| CrowdStrike `behaviors[].ioc_type` restricted to algorithm-qualified tokens `{hash_sha256, hash_md5, domain, filename, registry_key}` — bare `hash` WRONG, bare `registry` WRONG, `cmdline` is a SEPARATE sibling field (NOT an ioc_type), NOT `ipv4`/`ipv6` | BC-2.06.019 v1.11 Per-Sensor IOC-Surface Matrix (CrowdStrike row; corrected 2026-06-19 per ThreatQ + XSOAR research) | AC-004 + adversary probe; Red Gate asserts `ioc_type == "hash_sha256"` |
| CrowdStrike ioc_type parser MUST be tolerant of unknown tokens — log + preserve raw string, non-fatal | BC-2.06.019 v1.11 CrowdStrike row — tolerant-unknown-type policy | Adversary: verify no hard-reject branch on unrecognized ioc_type in generator.rs or route handler |
| Cyberint `Ioc` struct MUST use serde dual-alias: `#[serde(rename = "type", alias = "ioc_type")]` on `ioc_type` field; `#[serde(alias = "ioc_value")]` on `value` field | BC-2.06.019 v1.11 Cyberint row — INCONCLUSIVE inner-key, dual-alias required | AC-001 + adversary reads types.rs after implementation |
| Cyberint singleton `ioc: Option<Ioc>` on `Alert` struct — retain but flag: confirm via live-tenant validation whether this field ever appears; remove if absent | BC-2.06.019 v1.11 Cyberint row — no public-documentation basis for singleton field | Adversary: note as PENDING-LIVE-VALIDATION; do not add code that hard-depends on it |
| Canonical ThreatIntel pivot field is `iocs[].value` (plural list form) — NOT singleton `ioc.value` | BC-2.06.019 v1.11 PC-4 step 6 canonical pivot mandate | AC-007 canonical query; any `enrich threat_intel(ioc.value)` reference is stale and must be updated |
| PC-4 step 6: fail-closed on undeserializable alert records — route handler MUST produce an error, not silently skip | BC-2.06.019 v1.11 PC-4 step 6 fail-closed mandate | AC-007 extended note; adversary verifies no silent skip/drop in route handler |
| `CyberintClone::new_with_scenario` and `CrowdstrikeClone::new_with_scenario` take `catalog: &ScenarioEntityCatalog` argument | R2 adversary closure — catalog-threaded constructor signature for production IOC/CVE stamping | AC-002, AC-004; adversary verifies call sites pass catalog reference |
| Armis and Claroty: NO IOC fields added (permanent exclusion) | ADR-031 DTU=True-DTU fidelity + BC-2.06.019 v1.11 matrix (Armis/Claroty: NO permanent) | Adversary: grep for ioc in prism-dtu-armis/prism-dtu-claroty types.rs |
| Route Coverage Table in BC-2.06.019 MUST be updated in same commit as route change | BC-2.06.019 v1.11 §Route Coverage Table standing rule | Noted in PR description; state-manager burst |
| Every new TOML column MUST have a matching DTU struct field (SAP-2) | CLAUDE.md §SAP-2 | Adversary SAP-2 probe post-implementation |
| All `event_type =` tracing emissions require BC-2.16.002 catalog rows | SAP-1 / CLAUDE.md §SAP-1 | Adversary SAP-1 probe |
| NVD pivot MUST use `has device_cves_first` as existence filter (scalar String — first CVE ID); `device_cves` array NEVER stamped on generated records (Ruling 1b) | BC-2.06.019 v1.11 PC-4 §General Filtering Semantics, Ruling 1b | AC-008 canonical query; any `has device_cves` reference is stale P1 finding |
| NVD pivot MUST use `cvss_base_score >= 7.0` as CVSS filter — NOT `nvd_cvss_score`, NOT strict `>` | BC-2.06.019 v1.11 PC-4 canonical NVD pivot query (infusions.md §NVD + ADR-040 §3.2) | AC-008 canonical query; `nvd_cvss_score` or `>` 7.0 = P1 finding |
| Pivot queries MUST return non-empty results at stage >= 3 (ThreatIntel) / stage 4 (NVD with device_cves_first) | BC-2.06.019 v1.11 PC-2 StageMask table + BC-2.06.020 correlation invariants | Tests 8, 9 |
| `#[non_exhaustive]` on new public types in prism-dtu-cyberint (Ioc, AlertData if public) | CLAUDE.md §Conventions | ci.yml EXPECTED= bump |
| CrowdStrike generator JSON stamping: `behaviors[]` key added to `make_detection()` + static `fixtures/detections-detail.json` in SAME commit (generator-parity test) | review_2026_06_10_cs_parity.rs + U19 | test_f8_cs06_detection_shape_parity must pass |
| CrowdStrike: NO `#[non_exhaustive]` change needed (no typed Detection/Behavior struct exists; IOC fields are JSON keys in serde_json::Value records — U19) | U19 code grounding 2026-06-12 | Adversary verifies no struct added without justification |

**Forbidden patterns:**
- `_ioc_value` field references in any production route handler after this story
- `enrich threat_intel(ioc.value)` in canonical queries or test harnesses — use `enrich threat_intel(iocs[].value)` (BC-2.06.019 v1.11 canonical pivot mandate)
- `| where has device_cves` as NVD pivot existence filter — use `| where has device_cves_first` (BC-2.06.019 v1.11 Ruling 1b; `device_cves` array is NEVER stamped on generated records)
- `nvd_cvss_score` as CVSS column name — use `cvss_base_score` (canonical per infusions.md §NVD + ADR-040 §3.2; BC-2.06.019 v1.11 P1 finding if used)
- `cvss_base_score > 7.0` (strict `>`) — use `cvss_base_score >= 7.0` (canonical form is `>=`)
- `ioc_type = "hash"` (bare, not algorithm-qualified) in CrowdStrike behaviors[] stamping — use `"hash_sha256"` or `"hash_md5"` (BC-2.06.019 v1.11 correction)
- `ioc_type = "registry"` (bare) in CrowdStrike behaviors[] stamping — use `"registry_key"` (BC-2.06.019 v1.11 correction)
- `ioc_type = "cmdline"` in CrowdStrike behaviors[] stamping — `cmdline` is a SEPARATE sibling field, NOT an ioc_type value (BC-2.06.019 v1.11 correction)
- `ioc_type = "ipv4"` or `ioc_type = "ipv6"` in CrowdStrike behaviors[] stamping
- Hard-reject / parse-error on unknown CrowdStrike `ioc_type` tokens (must log + preserve, non-fatal)
- Silent skip/drop of undeserializable alert records in route handler — must fail-closed per BC-2.06.019 v1.11 PC-4 step 6
- `Ioc` struct without dual-alias serde annotation in prism-dtu-cyberint (single hard-coded key bet on INCONCLUSIVE field)
- IOC fields on Armis or Claroty records (ADR-031 violation)
- `CyberintClone::new_with_scenario` or `CrowdstrikeClone::new_with_scenario` called without `catalog: &ScenarioEntityCatalog` argument (R2 catalog-threaded signature is now the canonical form)
- Adding a `device_cves` array field to Armis device records — ONLY the scalar `device_cves_first` is surfaced (BC-2.06.019 v1.11 Ruling 1b)

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
| EC-001 | BC-2.06.019 v1.11 §Interim State | `_ioc_value` filter present alongside real-schema filter | FORBIDDEN — not a valid edge case; this is a MUST NOT state per BC-2.06.019 v1.11 |
| EC-002 | BC-2.06.019 v1.11 Per-Sensor IOC-Surface Matrix | IOC stamping attempted on Armis or Claroty records | FORBIDDEN — permanent exclusion; must not occur |
| EC-003 | BC-2.06.019 PC-4 ioc_hashes=false | Cyberint alert has `ioc.value` matching a hash in `catalog.ioc_hashes` | Alert withheld from response |
| EC-004 | BC-2.06.019 PC-4 ioc_ips=false | Cyberint alert has `alert_data.ip` matching `catalog.ioc_ips[0]` | Alert withheld from response |
| EC-005 | BC-2.06.019 PC-4 | Cyberint alert with no IOC fields (None/empty) at any stage | Alert NOT withheld (no matching IOC values to filter on) |
| EC-006 | BC-2.06.019 v1.11 Per-Sensor IOC-Surface Matrix | CrowdStrike host/device record (not detection) — no IOC fields | Device records have NO IOC stamping; IOCs live on detection records only |
| EC-007 | BC-2.06.019 PC-2 table | Pivot query at stage < 3 (device_cves=false, ioc_*=false at stage 0-1) | Result set may be empty; no error |
| EC-008 | BC-2.06.019 v1.11 PC-2 table + Ruling 1b | NVD pivot query at stage 3 (device_cves=false in StageMask) | `device_cves_first` scalar NOT stamped on device records at stage 3 → `has device_cves_first` filter returns no rows → empty result (correct behavior; canonical query uses `has device_cves_first` per Ruling 1b; test must configure demo server at stage 4 Containment where `device_cves=true` causes `device_cves_first` to be present) |

---

## SAP-2 Compliance Note

Per CLAUDE.md §SAP-2, the adversary for this story MUST:
1. Read `crates/prism-dtu-cyberint/src/types.rs` post-implementation — verify `Alert.ioc`,
   `Alert.iocs`, `Alert.alert_data` fields exist with matching types to TOML spec columns;
   verify `Ioc` struct has dual-alias serde annotations: `#[serde(rename = "type", alias = "ioc_type")]`
   on `ioc_type` field, `#[serde(alias = "ioc_value")]` on `value` field (BC-2.06.019 v1.11
   INCONCLUSIVE inner-key requirement); verify TOML wire names match primary key form:
   `ioc.type`/`ioc.value`; `iocs[].type`/`iocs[].value`
2. Read `crates/prism-dtu-crowdstrike/src/generator.rs` `make_detection()` — verify `"behaviors"`
   JSON key is present in the returned serde_json::Value with `ioc_type`/`ioc_value`/`ioc_source`/
   `ioc_description` keys; **VERIFY `ioc_type` value is `"hash_sha256"` (NOT bare `"hash"`) for
   catalog hash IOCs** (BC-2.06.019 v1.11 correction; bare `"hash"` = P1 CRITICAL); also read
   `fixtures/detections-detail.json` — verify same keys present AND `ioc_type` uses
   algorithm-qualified token (shape parity test compliance). NOTE: NO types.rs Detection/Behavior
   struct exists in CrowdStrike (U19 code grounding 2026-06-12) — adversary MUST NOT look for
   Behavior struct; it does not exist.
3. Read `crates/prism-dtu-armis/src/generator.rs` — verify `device_cves_first` scalar key is
   present on scenario device records (U17/Ruling 1b)
4. Grep for `_ioc_value` in `crates/prism-dtu-cyberint/src/routes/alerts.rs` — MUST return 0 matches
5. Verify Claroty and Armis types.rs have NO new IOC fields (permanent exclusion per ADR-031)

Column in TOML with no DTU equivalent = **P1 CRITICAL**. `_ioc_value` in route handler post-merge = **P1 CRITICAL** (BPRL-P4-01 recurrence). `Behavior.ioc_type` typed struct added for CrowdStrike = **P1 CRITICAL** (wrong approach — must be JSON key in generator, not a typed struct — U19). `ioc_type = "hash"` (bare) in CrowdStrike behaviors[] = **P1 CRITICAL** (BC-2.06.019 v1.11 correction; must be `"hash_sha256"` or `"hash_md5"`). `Ioc` struct in prism-dtu-cyberint without dual-alias serde annotation = **HIGH** (INCONCLUSIVE inner-key; hard-coded single-form bet will fail 50% of the time depending on live API convention). `enrich threat_intel(ioc.value)` in canonical query = **HIGH** (BC-2.06.019 v1.11 mandate; use `iocs[].value`). Silent skip/drop of undeserializable alert record = **P1 CRITICAL** (BC-2.06.019 v1.11 PC-4 step 6 fail-closed mandate). `has device_cves` as NVD pivot existence filter = **P1 CRITICAL** (BC-2.06.019 v1.11 Ruling 1b; `device_cves` array never stamped on generated records; use `has device_cves_first`). `nvd_cvss_score` as CVSS column name = **P1 CRITICAL** (stale/forbidden per BC-2.06.019 v1.11; use `cvss_base_score`).

---

## Story Changelog

| Version | Date | Change |
|---------|------|--------|
| v2.5 | 2026-07-10 | **DEFECT-CSDEVICES-EMPTY-PIPELINE-001 GET→POST ratification propagation.** Architect ratified converting CrowdStrike `fetch_devices` step from `GET /devices/entities/devices/v2` (query-param IDs) to `POST /devices/entities/devices/v2` (body `{"ids": [...]}`) — ratification artifact: `defect-csdevices-empty-pipeline-rootcause-2026-07-10.md` §Architect Ratification; BC-2.16.013 (v1.26) and BC-2.06.019 (v1.16) updated by product-owner. **(1) Frontmatter** `version: 2.4→2.5`; `modified` timestamp updated. **(2) Story Route Coverage Table Row 4** (CrowdStrike `primary_device`/`lateral_devices`): step-2 method corrected from `GET /devices/entities/devices/v2` to `POST /devices/entities/devices/v2`; ratification note added (GET route preserved at same path; POST is now the spec-driven step-2 shape; body: `{"ids": [...]}`). Historical changelog rows left at their original version numbers. |
| v2.4 | 2026-06-19 | **BC-2.06.019 v1.12→v1.13 pin-sync — Row 6 Cyberint guard prose corrected (F-PIVOT003-R10A-002, BC v1.13).** Root cause: BC v1.13 PO prose-vs-code coherence fix: the Route Coverage Table Row 6 (Cyberint alerts) guard mechanism description previously stated `Real-schema filter on iocs[].value / alert_data.ip/domain`, omitting the defensive singleton `ioc.value` check that `ioc_values_for` in `crates/prism-dtu-cyberint/src/routes/alerts.rs` actually performs. The implemented filter checks `iocs[].value` (dual-alias), `alert_data.ip`/`alert_data.domain`, AND defensively `ioc.value`; the singleton check is inert (generator never stamps `Alert.ioc`, only `Alert.iocs[]`) but harmless, consistent with AC-003 prose. The singleton `Alert.ioc` field remains flagged for removal (no public-documentation basis; F-PIVOT003-R2-004). **(1) Frontmatter** `version: 2.3→2.4`; `modified` timestamp updated. **(2) Scope ownership prose** updated v1.12→v1.13; defensive singleton note added. **(3) Behavioral Contracts table row** updated to v1.13 with v1.13 bullet describing the guard correction. **(4) AC-009** given-clause updated to v1.13; Cyberint guard mechanism description corrected to state the filter checks `iocs[].value`, `alert_data.ip`/`alert_data.domain`, AND defensively `ioc.value` (inert-but-harmless, with `ioc_values_for` citation). **(5) Story Route Coverage Table** header updated to v1.13; Cyberint Row guard mechanism description updated to include `ioc.value` defensive check with inert-harmless qualification and F-PIVOT003-R10A-002 reference. **(6) Token Budget** BC-2.06.019 row updated to v1.13. **(7) Tasks pre-flight** BC read note updated to v1.13 with Row 6 correction note. Historical changelog rows left at their original version numbers. |
| v2.3 | 2026-06-19 | **BC-2.06.019 v1.11→v1.12 pin-sync — Row 11 mirrored (POL-33 + F-PIVOT003-R8C-001).** BC-2.06.019 bumped to v1.12 by product-owner, adding Route Coverage Table Row 11: `device_cves, primary_device, lateral_devices` × prism-dtu-armis × `src/routes/search.rs` × `GET /api/v1/search?aql=...` (device branch, scenario path) × per-record `device_cves_first` omitted when `!mask.device_cves`; entity-visibility guards for primary/lateral devices × ACTIVE (F-PIVOT003-R8C-001). Root cause corrected in BC v1.12: the `armis.devices` sensor-spec `path_template` targets `GET /api/v1/search` (`search.rs`), not `GET /api/v1/devices` (`devices.rs`). The v1.11 Row 8 (`devices.rs`) guard covers the secondary `paginate_devices` path only. **(1) AC-009** title broadened from "3 new rows (v1.11)" to "4 new rows (v1.11 Rows 8–10 + v1.12 Row 11)"; body updated: added Row 11 sub-table (matching BC v1.12 verbatim); given-clause updated to v1.12; added NOTE clarifying `/api/v1/devices` is secondary route, not authoritative for `armis.devices`. **(2) Story Route Coverage Table** extended: Row 11 added (`device_cves, primary_device, lateral_devices` × search.rs × ACTIVE BC v1.12 Row 11 F-PIVOT003-R8C-001); Row 8 note amended to state it is the secondary route, not authoritative; table header updated to v1.12. **(3) Scope ownership prose** updated: v1.11 → v1.12; "three new rows" → "four new rows"; Row 11 narrative added; authoritative-route clarification added. **(4) Behavioral Contracts table row** updated to v1.12 with v1.12 bullet. **(5) Token Budget** BC-2.06.019 row updated to v1.12, Rows 1–11. **(6) Tasks pre-flight BC read note** updated to v1.12 + Row 11 reference. **(7) All live body BC-2.06.019 v1.11 citations for the BC table, AC-009, Route Coverage Table header updated to v1.12.** Historical changelog rows left at their original version numbers. |
| v2.2 | 2026-06-19 | **BC-2.06.019 v1.10→v1.11 pin-sync — 3 new Route Coverage Table rows (POL-33 + F-PIVOT003-R7A-001/R7A-002).** BC-2.06.019 bumped to v1.11 by product-owner, adding Rows 8–10 to the Route Coverage Table: Row 8 — Armis `device_cves` guard (`device_cves_first` field omitted via `obj.remove("device_cves_first")` when `!mask.device_cves` in `paginate_devices` scenario path, `prism-dtu-armis/src/routes/devices.rs`, ACTIVE); Row 9 — CrowdStrike `ioc_hashes` list-IDs guard (detection withheld by filter_map when `behaviors[].ioc_value` ∈ `catalog.ioc_hashes` AND `!mask.ioc_hashes`, `GET /detects/queries/detects/v1`, ACTIVE); Row 10 — CrowdStrike `ioc_hashes` get-summaries guard (same filter logic, `POST /detects/entities/summaries/GET/v1`, ACTIVE). **(1) AC-009** updated: title broadened to "3 new rows (v1.11) + Cyberint alerts ACTIVE"; body table extended with explicit Rows 8–10 matching BC v1.11 verbatim. **(2) Story Route Coverage Table** updated: CrowdStrike detections combined row split into two discrete rows matching BC v1.11 Rows 9–10; new Armis `device_cves` row added (BC v1.11 Row 8). **(3) All live body BC-2.06.019 v1.10 citations updated to v1.11** (scope ownership prose, Behavioral Contracts table row, all AC traces, Red Gate Plan table, Route Coverage Table header, Token Budget row, Architecture Compliance Rules, Forbidden patterns, SAP-2, Tasks). Historical changelog rows left at their original version numbers. **(4) Frontmatter** `version: 2.1→2.2`, `modified` timestamp updated. |
| v2.1 | 2026-06-19 | **BC-2.06.019 v1.10 canonical NVD query reconciliation (F-PIVOT003-R5B-001 MED + F-PIVOT003-R5B-002 OBS).** Root cause: AC-008 referenced `where has device_cves` as the NVD pivot existence filter, but the `device_cves` array is NEVER stamped on generated records and is NOT declared as a TOML column (Ruling 1b, BC-2.06.019 v1.10 §PC-4 General Filtering Semantics). The filter would match zero records, making the canonical NVD pivot query return 0 rows. **(1) AC-008 canonical query corrected:** `| where has device_cves` → `| where has device_cves_first`; query block updated to match BC-2.06.019 v1.10 §PC-4 canonical form `from armis.devices / where has device_cves_first / enrich nvd(device_cves_first) / where cvss_base_score >= 7.0 / sort cvss_base_score desc`. AC-008 title updated to reflect stage 4 (Containment) requirement. AC-008 trace updated to v1.10. AC-008 NOTE expanded with Ruling 1b explanation and explicit forbidden forms. **(2) EC-008 updated:** explanation now references `has device_cves_first` scalar projection and explains why the stale `has device_cves` filter returns zero rows. **(3) All live body BC-2.06.019 v1.9 citations updated to v1.10** (scope ownership prose, Behavioral Contracts table row + v1.10 bullet, all AC traces, Red Gate Plan table, Route Coverage Table header and row, Token Budget row, Architecture Compliance Rules, Forbidden patterns, SAP-2, Tasks). Historical changelog rows left at their original version numbers. **(4) Architecture Compliance Rules:** two new rows added — `has device_cves_first` mandatory existence filter (Ruling 1b); `cvss_base_score >= 7.0` canonical CVSS filter. **(5) Forbidden patterns:** three new entries — `has device_cves` (stale), `nvd_cvss_score` (stale column name), `cvss_base_score > 7.0` (strict inequality). **(6) SAP-2 closing paragraph:** two new P1 CRITICAL findings — `has device_cves` and `nvd_cvss_score`. |
| v2.0 | 2026-06-19 | **R2 adversary closures + BC-2.06.019 v1.9 propagation.** **(R2-001) crates_touched:** added `prism-dtu-armis` (AC-008 Armis CVE work in `crates/prism-dtu-armis/src/generator.rs` + `lib.rs`). **(R2-002) Canonical ThreatIntel pivot updated:** `enrich threat_intel(ioc.value)` → `enrich threat_intel(iocs[].value)` throughout story — Narrative, AC-007 canonical query + extended NOTE, AC-009 guard mechanism text, Route Coverage Table body row, Forbidden patterns (new entry), SAP-2 closing paragraph (new HIGH finding). BC-2.06.019 v1.9 mandate: `iocs[].value` is the canonical pivot field (plural list); singleton `ioc.value` is stale. **(R2-003) Fail-closed mandate propagated:** BC-2.06.019 v1.9 PC-4 step 6 — route handler MUST NOT silently skip/drop undeserializable alert records; must fail-closed. Added to AC-007 extended NOTE, Architecture Compliance Rules (new row), Forbidden patterns (new entry), SAP-2 closing paragraph (new P1 CRITICAL). **(R2-004) `new_with_scenario` signature note:** `CyberintClone::new_with_scenario` and `CrowdstrikeClone::new_with_scenario` now take `catalog: &ScenarioEntityCatalog` argument. Updated AC-002 and AC-004 to name the catalog-threaded parameter form; added Architecture Compliance Rules row; added Forbidden patterns entry. Behavioral anchors used per TD-VSDD-091. **(R2-005) Stale Red Gate count note resolved:** body note at Red Gate Test Plan footer was pending-instruction voice ("Update `red_gate_tests: 9`…"); rewritten as completed statement since frontmatter already reads `red_gate_tests: 10`. **(R2-006) All body-level BC-2.06.019 v1.8 citations updated to v1.9** (scope ownership prose, Behavioral Contracts table row, all AC traces, Red Gate Plan table, Route Coverage Table header, Architecture Compliance Rules, Forbidden patterns, Edge Cases, SAP-2, Tasks). Historical changelog rows left at their original version numbers. **(R2-007) BC scope ownership prose updated:** v1.8 sentence replaced with v1.9 sentence noting `iocs[].value` canonical pivot and fail-closed PC-4 step 6. |
| v1.9 | 2026-06-19 | Uncertainty-pivot003 corrections propagated from BC-2.06.019 v1.8 + research doc uncertainty-pivot003-s504-2026-06-19.md. **(1) PR #185 caveat RESOLVED:** U23 materialization-time re-verify task updated to DONE; field names confirmed on develop@7fd35b77 (`ioc_ips`/`ioc_domains`/`ioc_hashes` in prism-dtu-common/src/scenario/mod.rs; `_ioc_value`/`_ioc_type` in prism-dtu-cyberint/src/routes/alerts.rs). Previous Story Intelligence section updated to reflect merge. **(2) CrowdStrike ioc_type enum CORRECTED (HIGH confidence):** `behaviors[].ioc_type` value set changed from old `{hash, domain, filename, registry, cmdline}` to corrected `{hash_sha256, hash_md5, domain, filename, registry_key}`. Bare `hash` → algorithm-qualified `hash_sha256`/`hash_md5`. Bare `registry` → `registry_key`. `cmdline` removed (it is a SEPARATE sibling field, not an ioc_type value). Tolerant-unknown-type policy added (log + preserve raw string, non-fatal). Changed: AC-004 body + Red Gate test assertion, Architecture Compliance Rules, Forbidden patterns, SAP-2 note, Phase 3 task, frontmatter risk_mitigations. **(3) Cyberint inner-key INCONCLUSIVE → dual-alias (BC-2.06.019 v1.8):** AC-001 rewritten to require `#[serde(rename = "type", alias = "ioc_type")]` + `#[serde(alias = "ioc_value")]` on `Ioc` struct. New Red Gate test `test_BC_2_06_019_cyberint_ioc_struct_dual_alias_deserializes_both_key_forms` added. Singleton `ioc: Option<Ioc>` field flagged for likely removal pending live-tenant validation. TOML column AC-006 corrected: `iocs[].type` is primary column name (not `iocs[].ioc_type`). AC-003 filter note updated to use `Ioc.value` accessor (dual-alias resolves at deserialization). red_gate_tests frontmatter bumped 9→10. **(4) All BC-2.06.019 v1.7 body citations updated to v1.8.** Source: uncertainty-pivot003-s504-2026-06-19.md §Item 1 (HIGH) + §Item 2 (INCONCLUSIVE) + BC-2.06.019 v1.8 Per-Sensor IOC-Surface Matrix. |
| v1.8 | 2026-06-13 | BPRL-P24-01: BC-2.06.020 v1.5→v1.6 pin-sync (DTU-perimeter enforcement prose corrected: structural Cargo/E0432, not the prism-query perimeter-violation gate). Two live pin sites updated: §Behavioral Contracts BC table row and §Token Budget BC-2.06.020 context row. Historical changelog rows left untouched. version 1.7→1.8. |
| v1.7 | 2026-06-13 | BC-2.06.020 v1.4→v1.5 pin-sync (BPRL-P22-01: VP Anchors prose A-H→A-L / 8→12 VPs; no behavior change). Two live pin sites updated: §Behavioral Contracts BC table row and §Token Budget BC-2.06.020 context row. Historical changelog rows left untouched. version 1.6→1.7. |
| v1.6 | 2026-06-13 | Pin-sync — BC-2.06.020 v1.3→v1.4 (BPRL-P14-01 RNG range literal correction; POL-23). Two live pin sites updated: §Behavioral Contracts BC table row and §Token Budget BC-2.06.020 context row. Historical changelog rows left untouched. version 1.5→1.6. |
| v1.5 | 2026-06-12 | Pin-sync — BC-2.06.020 v1.2→v1.3 (D-1117 Cyberint CVE correlation + SEC-001 collision-safety; POL-23). Two live pin sites updated: §Behavioral Contracts BC table row (updated title and added INV-CYBERINT-ALERT-CVE-CORRELATION-001 to key clauses) and §Token Budget BC-2.06.020 context row (~3,000→~3,600). Historical changelog rows left untouched. version 1.4→1.5. |
| v1.4 | 2026-06-12 | Micro-sweep — BC-2.06.019 v1.6→v1.7 pin-sync (BPRL-P7-01 inventory-prose correction; POL-23). All body-level BC-2.06.019 v1.6 citations updated to v1.7 (frontmatter comment block, §Narrative, §Architecture Compliance Rules, §Acceptance Criteria AC traces, §Token Budget, §Tasks, §Forbidden Dependencies, §Edge Cases). version 1.3→1.4. |
| v1.3 | 2026-06-12 | Micro-sweep — BC-2.06.019 v1.5→v1.6 pin-sync (BPRL-P6-01 Claroty devices Route Coverage row + exhaustive inventory verification note; POL-23). All body-level BC-2.06.019 v1.5 citations updated to v1.6 (frontmatter comment block, §Narrative, §Architecture Compliance Rules, §Acceptance Criteria AC traces, §Token Budget, §Tasks, §Forbidden Dependencies, §Edge Cases). version 1.2→1.3. |
| v1.2 | 2026-06-12 | Micro-sweep — BC-2.06.019 v1.4→v1.5 pin-sync (BPRL-P5-01 Route Coverage Table corrections + PC-4 5-arg prose; POL-23). All body-level BC-2.06.019 v1.4 citations updated to v1.5 (frontmatter comment block, §Narrative, §Architecture Compliance Rules, §Acceptance Criteria AC traces, §Token Budget, §Tasks, §Forbidden Dependencies, §Edge Cases). version 1.1→1.2. |
| v1.1 | 2026-06-12 | D-1109 remove-uncertainty closure: U1/U17/U19/U20/U21/U22/U23/U24/U25 applied (scanner + research-agent + architect rulings 1-4, WO-D1109 v1.1). enrich syntax → function-call form throughout. CrowdStrike AC-004/AC-005 rewritten: no typed Detection/Behavior structs — IOC fields are JSON keys in generator.rs serde_json::Value records; static fixture parity requirement added; non_exhaustive item for CrowdStrike removed (moot). Route Coverage Table regenerated from actual routers (U20): alerts_search.rs/GET /alerts/queries/alerts/v2 removed (not in CrowdStrike router); GET /xdome/api/v1/alerts replaced with POST /api/v1/alerts (Claroty); CrowdStrike summaries corrected to POST /detects/entities/summaries/GET/v1. Cyberint TOML wire names corrected: iocs[].type (NOT iocs[].ioc_type). Armis device_cves_first generator projection task added (Ruling 1b). PR #185 IN-FLIGHT status noted; materialization-time re-verify task added. verification_properties: [] (BC-2.06.019-internal sub-properties; VP prose citation added to body). axum 0.7 confirmed 2026-06-12 annotation. |
| v1.0 | 2026-06-12 | Initial draft per WO-D1109 §Story 3 and BC-2.06.019 v1.5 §Per-Sensor IOC-Surface Matrix and §Interim State. Root cause closure of BPRL-P4-01. Depends on 002; blocks T11/T13 demo objectives. Full Route Coverage Table included (BC-2.06.019 §Question 4 requirement). 9 Red Gate tests. SAP-2 compliance note included. |
