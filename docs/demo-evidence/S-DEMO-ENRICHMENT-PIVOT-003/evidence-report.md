# Evidence Report — S-DEMO-ENRICHMENT-PIVOT-003

**Story:** IOC Stamping (Cyberint + CrowdStrike) and Demo Pivot Query Validation
**Branch:** `feature/S-DEMO-ENRICHMENT-PIVOT-003`
**HEAD at evidence capture:** `62d4fcdb`
**Date:** 2026-06-20
**Status:** LOCAL 3-CLEAN converged; all 32 BC_2_06_019 tests pass
**BC:** BC-2.06.019 v1.13, BC-2.06.020 v1.6

---

## Evidence Summary

| Total ACs | Test-execution evidence | Coverage |
|-----------|-------------------------|----------|
| 9 (AC-001–009) | 9 ACs | 100% — every AC has at least one PASS test |

---

## Recording Artifacts

All recordings are VHS-generated terminal captures (VHS 0.11.0, FiraCode Nerd Font Mono, Catppuccin Mocha theme). Each `.gif` is suitable for PR embedding; each `.webm` is the archival format. The `.tape` file is the reproducible script source.

| Recording | ACs Covered | Artifact |
|-----------|-------------|----------|
| Cyberint IOC Struct + Fixture Generator | AC-001, AC-002 | `AC-001-002-cyberint-ioc-struct.{gif,webm,tape}` |
| Cyberint Real-Schema Filter (atomic swap) | AC-003 | `AC-003-real-schema-filter.{gif,webm,tape}` |
| CrowdStrike behaviors[] + TOML Specs | AC-004, AC-005, AC-006 | `AC-004-005-006-crowdstrike-toml-ioc.{gif,webm,tape}` |
| Canonical Pivot Queries (ThreatIntel + NVD) | AC-007, AC-008 | `AC-007-008-canonical-pivot-queries.{gif,webm,tape}` |
| Route Coverage Table served-route guards | AC-009 | `AC-009-route-coverage-served-route.{gif,webm,tape}` |
| Full Test Run | All 32 BC_2_06_019 tests | `full-test-run-transcript.txt` |

---

## Per-AC Evidence

### AC-001 — Cyberint Alert struct adds real-schema IOC fields with serde dual-alias

**Evidence form:** Test-execution capture (VHS recording)
**Tests:**
- `test_BC_2_06_019_cyberint_alert_struct_has_real_ioc_fields` (PASS 0.023s)
- `test_BC_2_06_019_cyberint_ioc_struct_dual_alias_deserializes_both_key_forms` (PASS 0.023s)

**Behavior observed:** `Alert` struct in `crates/prism-dtu-cyberint/src/types.rs` now carries
`ioc: Option<Ioc>`, `iocs: Vec<Ioc>`, `alert_data: Option<AlertData>`. The `Ioc` struct uses
serde dual-alias: `#[serde(rename = "type", alias = "ioc_type")]` on `ioc_type` field and
`#[serde(alias = "ioc_value")]` on `value` field. Deserialization test confirms both
`{"type":"domain","value":"evil.example.com"}` and `{"ioc_type":"domain","ioc_value":"evil.example.com"}`
yield the same `Ioc` struct value (BC-2.06.019 v1.11 INCONCLUSIVE inner-key dual-alias requirement).

**Artifacts:**
- `AC-001-002-cyberint-ioc-struct.gif` — VHS recording showing tests PASS
- `AC-001-002-cyberint-ioc-struct.webm` — archival recording
- `AC-001-002-cyberint-ioc-struct.tape` — VHS script source
- `full-test-run-transcript.txt` lines 4–5: PASS

---

### AC-002 — Cyberint fixture generator stamps scenario IOCs onto alert records

**Evidence form:** Test-execution capture (VHS recording)
**Test:** `test_BC_2_06_019_cyberint_fixture_generator_stamps_scenario_iocs` (PASS 0.025s)

**Behavior observed:** `CyberintClone::new_with_scenario(catalog: &ScenarioEntityCatalog)` fixture
generator produces alert records where `iocs[0].value` is set from `catalog.ioc_ips[0]` (or
`ioc_domains[0]`). Scenario-mode alerts carry real IOC field values from the `ScenarioEntityCatalog`.
The singleton `Alert.ioc` field is retained but not stamped (flagged for removal pending live-tenant
validation per BC-2.06.019 v1.11).

**Artifacts:**
- `AC-001-002-cyberint-ioc-struct.gif` — VHS recording showing test PASS
- `AC-001-002-cyberint-ioc-struct.webm` — archival recording
- `AC-001-002-cyberint-ioc-struct.tape` — VHS script source
- `full-test-run-transcript.txt` line 8: PASS

---

### AC-003 — Cyberint alerts route: _ioc_value synthetic filter REMOVED; real-schema filter ADDED (atomic)

**Evidence form:** Test-execution capture (VHS recording) + grep verification
**Tests:**
- `test_BC_2_06_019_cyberint_alerts_real_schema_ioc_filter_no_synthetic` (PASS 10.527s)
- `test_BC_2_06_019_ioc_hashes_false_withholds_cyberint_alert_with_matching_hash` (PASS 10.538s)
- `test_BC_2_06_019_fail_closed_malformed_alert_is_withheld` (PASS 10.498s)
- `test_BC_2_06_019_ip_domain_ioc_stage_gating_served_route` (PASS 10.551s)

**Behavior observed:**
- `grep -c '_ioc_value' crates/prism-dtu-cyberint/src/routes/alerts.rs` returns **0** (synthetic filter completely removed)
- Real-schema filter in `alerts.rs` checks `ioc_values_for` which inspects `iocs[].value` (dual-alias),
  `alert_data.ip`/`alert_data.domain`, and defensively the singleton `ioc.value` (inert — generator
  never stamps singleton, only `iocs[]`; harmless per BC-2.06.019 v1.13 F-PIVOT003-R10A-002)
- When `ioc_hashes=false`: alert with `iocs[0].value` matching `catalog.ioc_hashes[0]` is withheld from response
- Fail-closed: malformed alert records (undeserializable) are withheld, not silently passed through
  (BC-2.06.019 v1.11 PC-4 step 6 fail-closed mandate)

**Artifacts:**
- `AC-003-real-schema-filter.gif` — VHS recording showing grep PASS + test PASSes
- `AC-003-real-schema-filter.webm` — archival recording
- `AC-003-real-schema-filter.tape` — VHS script source
- `full-test-run-transcript.txt` lines 21–24: PASS

---

### AC-004 — CrowdStrike detection generator stamps behaviors[].ioc_type/ioc_value in JSON records

**Evidence form:** Test-execution capture (VHS recording)
**Tests:**
- `test_BC_2_06_019_crowdstrike_detection_behaviors_ioc_hash_stamped` (PASS 0.017s)
- `test_BC_2_06_019_crowdstrike_detection_behaviors_no_ioc_when_none` (PASS 0.025s)
- `test_BC_2_06_019_scenario_clone_detection_0_carries_ioc_value_from_catalog` (PASS 0.029s)
- `test_BC_2_06_019_crowdstrike_generator_scenario_parity_modulo_ioc_stamp` (PASS 0.023s)

**Behavior observed:** `make_detection()` in `crates/prism-dtu-crowdstrike/src/generator.rs` now
stamps a `"behaviors"` JSON array key on scenario detection records. Asserts:
- `behaviors[0]["ioc_type"] == "hash_sha256"` — algorithm-qualified token (NOT bare `"hash"`)
  (BC-2.06.019 v1.11 correction; corrected per ThreatQ/XSOAR CrowdStrike docs)
- `behaviors[0]["ioc_value"] == catalog.ioc_hashes[0]`
- `behaviors[0]["ioc_source"] == "catalog"`
- `behaviors[0]["ioc_description"] == "scenario IOC"`
- Error path (no catalog): detection record has no `"behaviors"` key when catalog is None
- Generator-fixture parity: `fixtures/detections-detail.json` carries the same `"behaviors"` key
  (shape parity test `test_BC_2_06_019_crowdstrike_generator_scenario_parity_modulo_ioc_stamp` PASS)

**Artifacts:**
- `AC-004-005-006-crowdstrike-toml-ioc.gif` — VHS recording showing all tests PASS
- `AC-004-005-006-crowdstrike-toml-ioc.webm` — archival recording
- `AC-004-005-006-crowdstrike-toml-ioc.tape` — VHS script source
- `full-test-run-transcript.txt` lines 1–2, 6–7, 9: PASS

---

### AC-005 — CrowdStrike detections TOML spec declares behaviors[] IOC columns matching generator JSON shape

**Evidence form:** Test-execution capture (VHS recording)
**Test:** `test_BC_2_06_019_crowdstrike_detection_toml_spec_has_ioc_columns` (PASS 0.014s)

**Behavior observed:** The CrowdStrike sensor TOML spec now declares columns:
`behaviors[].ioc_type`, `behaviors[].ioc_value`, `behaviors[].ioc_source`, `behaviors[].ioc_description`.
SAP-2 compliance: each column matches a JSON key in the `make_detection()` `serde_json::Value`
return value (no typed Detection/Behavior struct exists in CrowdStrike DTU per U19 — adversary
verified from `src/generator.rs` and `fixtures/detections-detail.json` directly).

**Artifacts:**
- `AC-004-005-006-crowdstrike-toml-ioc.gif` — VHS recording showing PASS
- `AC-004-005-006-crowdstrike-toml-ioc.webm` — archival recording
- `full-test-run-transcript.txt` line 1: PASS

---

### AC-006 — Cyberint sensor TOML spec declares ioc, iocs[], alert_data.* columns

**Evidence form:** Test-execution capture (VHS recording)
**Test:** `test_BC_2_06_019_cyberint_alert_toml_spec_has_ioc_columns` (PASS 0.020s)

**Behavior observed:** The Cyberint sensor TOML spec now declares columns:
`ioc.type`, `ioc.value` (singleton, flagged for removal), `iocs[].type`, `iocs[].value`,
`alert_data.ip`, `alert_data.domain`, `alert_data.url`.
Note: `iocs[].type` is the primary TOML column name matching `#[serde(rename = "type")]`; the
serde dual-alias (`ioc_type`) resolves at the Rust struct level, not in TOML column declarations.
SAP-2 compliance: each column has a matching field in the `Alert` struct after this story's additions.

**Artifacts:**
- `AC-004-005-006-crowdstrike-toml-ioc.gif` — VHS recording showing PASS
- `AC-004-005-006-crowdstrike-toml-ioc.webm` — archival recording
- `full-test-run-transcript.txt` line 3: PASS

---

### AC-007 — Canonical ThreatIntel pivot query returns Malicious results at stage >= 3

**Evidence form:** Test-execution capture (VHS recording)
**Tests:**
- `test_BC_2_06_019_canonical_threatintel_pivot_query_returns_malicious_at_stage_3` (PASS 15.508s)
- `test_BC_2_06_019_enrich_pipeline_e2e_threatintel_pivot_executes_udf_and_returns_malicious` (PASS 0.189s)

**Behavior observed:** Canonical query:
```prismql
FROM cyberint_alerts
| where severity = "high"
| enrich threat_intel(iocs[].value)
| where threat_is_known_malicious = true
| sort threat_score desc
| head 10
```
executed against demo server configured at stage 3 (Exfil; `ioc_ips`, `ioc_domains`, `ioc_hashes`
visible). Result set is non-empty; all returned records have `threat_is_known_malicious = true`
and `threat_score >= 75`.
Canonical pivot field is `iocs[].value` (plural list form per BC-2.06.019 v1.11 mandate;
NOT the stale singleton `ioc.value`).
E2E test drives the full `| enrich threat_intel(iocs[].value)` UDF execution path in-process.

**Artifacts:**
- `AC-007-008-canonical-pivot-queries.gif` — VHS recording showing tests PASS
- `AC-007-008-canonical-pivot-queries.webm` — archival recording
- `AC-007-008-canonical-pivot-queries.tape` — VHS script source
- `full-test-run-transcript.txt` lines 19–20, 25: PASS

---

### AC-008 — Canonical NVD pivot query returns HIGH CVSS results at stage 4 (Containment)

**Evidence form:** Test-execution capture (VHS recording)
**Tests:**
- `test_BC_2_06_019_canonical_nvd_pivot_query_returns_high_cvss_at_containment_stage` (PASS 0.023s)
- `test_BC_2_06_019_enrich_pipeline_e2e_nvd_pivot_executes_udf_and_returns_high_cvss` (PASS 0.187s)

**Behavior observed:** Canonical query:
```prismql
from armis.devices
| where has device_cves_first
| enrich nvd(device_cves_first)
| where cvss_base_score >= 7.0
| sort cvss_base_score desc
```
executed against demo server at stage 4 (Containment; `device_cves = true` in StageMask per
BC-2.06.019 v1.11 PC-2). `device_cves_first` scalar is stamped on device records at stage 4.
Result set is non-empty; all returned records have `cvss_base_score >= 7.0`.
Existence filter is `has device_cves_first` (NOT stale `has device_cves` — forbidden per
BC-2.06.019 v1.11 Ruling 1b; `device_cves` array is NEVER stamped on generated records).
CVSS filter is `cvss_base_score >= 7.0` (NOT `nvd_cvss_score` — stale/forbidden per infusions.md).

**Artifacts:**
- `AC-007-008-canonical-pivot-queries.gif` — VHS recording showing tests PASS
- `AC-007-008-canonical-pivot-queries.webm` — archival recording
- `AC-007-008-canonical-pivot-queries.tape` — VHS script source
- `full-test-run-transcript.txt` lines 10, 19: PASS

---

### AC-009 — BC-2.06.019 Route Coverage Table updated (StageMask served-route guards)

**Evidence form:** Test-execution capture (VHS recording) — served-route integration tests
**Tests:**
- `test_BC_2_06_019_served_route_to_enrich_pipeline_composed_full_chain` (PASS 15.629s) — Cyberint→ThreatIntel composed chain
- `test_BC_2_06_019_armis_served_route_to_nvd_enrich_pipeline_composed_full_chain` (PASS 15.633s) — Armis→NVD composed chain
- `test_BC_2_06_019_crowdstrike_ioc_bearing_detection_stagemask_served_route` (PASS 20.501s) — CrowdStrike ioc_hashes guard
- `test_BC_2_06_019_cyberint_alerts_stagemask_ioc_filter` (PASS 20.551s) — Cyberint real-schema guard (Row 6)
- `test_BC_2_06_019_crowdstrike_containment_visible_at_stage4_only` (PASS 20.473s) — stage mask validation

**Behavior observed:** All 5 Route Coverage Table rows that are ACTIVE post-S-DEMO-ENRICHMENT-PIVOT-003
have served-route tests confirming their guard mechanisms function correctly:
- Row 6 (Cyberint alerts): real-schema filter via `ioc_values_for` — ACTIVE
- Row 8 (Armis `device_cves`, `devices.rs`): `device_cves_first` scalar omitted when `!mask.device_cves` — ACTIVE
- Rows 9+10 (CrowdStrike `ioc_hashes`, list-IDs + summaries routes): detection withheld when `behaviors[].ioc_value` matches — ACTIVE
- Row 11 (Armis `device_cves`, `search.rs`): canonical `armis.devices` path, per-record guard — ACTIVE

Note: BC-2.06.019 Route Coverage Table update (Cyberint alerts INTERIM→ACTIVE, new Rows 8–11)
is a `.factory/` artifact change; noted in PR description for state-manager post-merge burst
per BC-2.06.019 v1.13 §Route Coverage Table standing rule.

**Artifacts:**
- `AC-009-route-coverage-served-route.gif` — VHS recording showing tests PASS
- `AC-009-route-coverage-served-route.webm` — archival recording
- `AC-009-route-coverage-served-route.tape` — VHS script source
- `full-test-run-transcript.txt` lines 28–32: PASS

---

## POL-10 Compliance

All evidence files are under `docs/demo-evidence/S-DEMO-ENRICHMENT-PIVOT-003/` (story-scoped subfolder).
No files were placed directly at `docs/demo-evidence/*.md`. Verified: `ls docs/demo-evidence/` shows
no `.md` files at the flat level from this story.

---

## Full Test Run Summary

```
cargo nextest run -p prism-dtu-cyberint -p prism-dtu-crowdstrike -p prism-dtu-demo-server \
  --features "dtu,fixture-gen" -E 'test(BC_2_06_019)' --no-fail-fast

Summary [20.552s] 32 tests run: 32 passed, 442 skipped
```

Total BC_2_06_019 tests for this story: 32 — all PASS.
No ignored tests. No deferred coverage.
