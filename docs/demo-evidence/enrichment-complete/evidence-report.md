---
scope_id: enrichment-complete
branch: fix/enrichment-complete
head_sha: d07025bf
recorded_at: 2026-06-23
stories:
  - S-DEMO-ENRICHMENT-PIVOT-001
  - S-DEMO-ENRICHMENT-PIVOT-002
  - S-DEMO-ENRICHMENT-PIVOT-003
  - ENRICH-1
  - ENRICH-2
  - ENRICH-3
  - ENRICH-4-B
rationale_for_shared_scope: >
  This evidence covers the integrated enrichment chain delivered across
  ENRICH-1/2/3/4-B on branch fix/enrichment-complete. The branch integrates
  work from multiple story branches (fix/enrich-23-dtu-wiring,
  fix/enrich-4b-pipe-execution) that collectively form a single shipped feature.
  No single STORY-INDEX story_id covers the full integration; per the demo
  runbook, docs/demo-evidence/enrichment-complete/ is the designated scope
  subfolder (T13-capstone-demo-runbook.md §Recording).
---

# Enrichment Chain — Demo Evidence Report

## Summary

This report documents the live demo evidence for the enrichment chain delivered on
`fix/enrichment-complete` (HEAD `d07025bf`). The LOCAL adversarial cascade has
converged (3-CLEAN strict). This evidence is for the PR and the T13 capstone demo.

**Both flagship enrichment values are reproduced:**
- `threat_score = 95` (Malicious, above the ≥75 threshold) via ThreatIntel DTU
- `cvss_base_score = 8.1` (HIGH) via NVD DTU

## Environment

| Parameter | Value |
|-----------|-------|
| Branch | `fix/enrichment-complete` |
| HEAD SHA | `d07025bf` |
| DTU mode | `PRISM_DTU_MODE=true` |
| Scenario seed | `org-c` seed=200, scenario Stage 4 (`scenario_start_secs` set to -700s for reproducibility) |
| ThreatIntel DTU | `http://127.0.0.1:59918` (ephemeral port from demo-run.sh) |
| NVD DTU | `http://127.0.0.1:59919` (ephemeral port from demo-run.sh) |
| Recording tool | VHS 0.11.0 |
| Font | FiraCode Nerd Font Mono |

## Acceptance Criterion Coverage

### AC-001: IOC Enrichment — threat_score(iocs_value) → 95

**Acceptance criterion source:** T13-capstone-demo-runbook.md §3 Step 3.2 + §5.5 dry-run checklist  
**BCs covered:** BC-2.19.001, BC-2.19.003

**Query executed:**
```
FROM cyberint_alerts | where iocs_value IS NOT NULL | enrich threat_score(iocs_value) | limit 3
```

**Client:** `org-c` (Cyberint + CrowdStrike + Armis + Claroty)

**Observed output (all 3 rows):**
```
Row 1:
  alert_id:     alert-0196f4b2-200-0
  iocs_value:   037f558b75744d1a2ccad89a6fa30b432b26a0f4...  (hash_sha256)
  iocs_type:    ["hash_sha256"]
  threat_score: threat_score=95 | malicious=True
  severity:     high

Row 2:
  alert_id:     alert-0196f4b2-200-1
  iocs_value:   037f558b75744d1a2ccad89a6fa30b432b26a0f4...  (hash_sha256)
  iocs_type:    ["hash_sha256"]
  threat_score: threat_score=95 | malicious=True
  severity:     high

Row 3:
  alert_id:     alert-0196f4b2-200-2
  iocs_value:   037f558b75744d1a2ccad89a6fa30b432b26a0f4...  (hash_sha256)
  iocs_type:    ["hash_sha256"]
  threat_score: threat_score=95 | malicious=True
  severity:     high
```

**Verification:** The `threat_score` column contains the full ThreatIntel enrichment
JSON object: `{"threat_score":95,"threat_is_known_malicious":true,"threat_sources":["virustotal"]}`.
The value `95 ≥ 75` satisfies the Malicious threshold per BC-2.06.020.

**Note on `iocs_value` format:** Per ENRICH-1 (source_path `$.iocs[*].value`),
`iocs_value` is a JSON-list string (e.g., `["037f558b..."]`). The enrichment UDF
correctly unpacks the list and returns a corresponding JSON-list of enrichment objects.

**Recordings:**
- `AC-001-threat-score-enrichment.gif` — 1200×600 GIF (106 KB)
- `AC-001-threat-score-enrichment.webm` — WebM (109 KB)
- `AC-001-threat-score-enrichment.tape` — VHS script source

**Chain path:** PrismQL `| enrich threat_score(iocs_value)` → DataFusion ScalarUDF
→ `InfusionAsyncUdf::invoke_async_with_args` (JSON-list unpacking per ENRICH-1)
→ `InfusionRegistry` → `PluginInfusionSource` → WASM plugin `prism-threatintel-infusion.prx`
→ DTU HTTP `/v3/hash/{hash}?key=...` at `http://127.0.0.1:59918`.

---

### AC-002: CVE Enrichment — cvss_base_score(device_cves_first) → 8.1

**Acceptance criterion source:** T13-capstone-demo-runbook.md §3 Step 3.5 + §5.5 dry-run checklist  
**BCs covered:** BC-2.19.004, ADR-040 HttpLookup path

**Query executed:**
```
FROM armis_devices | where device_cves_first IS NOT NULL | enrich cvss_base_score(device_cves_first) | limit 3
```

**Client:** `org-c`

**Observed output (all 3 rows):**
```
Row 1:
  device_id:          dev-0196f4b2-200-0
  device_cves_first:  CVE-9999-72859
  cvss_base_score:    8.1

Row 2:
  device_id:          dev-0196f4b2-200-1
  device_cves_first:  CVE-9999-72859
  cvss_base_score:    8.1

Row 3:
  device_id:          dev-0196f4b2-200-2
  device_cves_first:  CVE-9999-72859
  cvss_base_score:    8.1
```

**Verification:** `cvss_base_score = 8.1` satisfies `≥ 7.0` (HIGH threshold). CVE
`CVE-9999-72859` is a synthetic scenario CVE (collision-safe per `CVE-9999-NNNN`
format, BC-2.06.020). NVD DTU returns `baseScore: 8.1, baseSeverity: HIGH,
vectorString: CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:N`.

**Input field:** `device_cves_first` (scalar String, Ruling 1b per BC-2.06.019 §PC-4).
This is the first CVE ID projected as a scalar — NOT `device_cves` (which would be
a JSON array).

**Recordings:**
- `AC-002-cvss-enrichment.gif` — 1200×600 GIF (86 KB)
- `AC-002-cvss-enrichment.webm` — WebM (82 KB)
- `AC-002-cvss-enrichment.tape` — VHS script source

**Chain path:** PrismQL `| enrich cvss_base_score(device_cves_first)` → DataFusion
ScalarUDF → `InfusionAsyncUdf` → `InfusionRegistry` → `HttpLookupInfusionSource`
(ADR-040 HttpLookup path) → HTTP GET `{NVD_DTU_URL}/rest/json/cves/2.0?cveId={cve_id}&apiKey=...`
at `http://127.0.0.1:59919`.

---

## DTU Validation

Direct HTTP verification of enrichment values (pre-recording):

**ThreatIntel DTU hash lookup:**
```
GET http://127.0.0.1:59918/v3/hash/037f558b...?key=demo-threatintel-api-key
→ {"threat_score": 95, "threat_is_known_malicious": true, "threat_sources": ["virustotal"],
   "abuseipdb_confidence_score": 98, "greynoise_classification": "malicious",
   "virustotal_detections": 58}
```

**NVD DTU CVE lookup:**
```
GET http://127.0.0.1:59919/rest/json/cves/2.0?cveId=CVE-9999-72859&apiKey=demo-nvd-api-key
→ vulnerabilities[0].cve.metrics.cvssMetricV31[0].cvssData.baseScore = 8.1
  baseSeverity = "HIGH"
  vectorString = "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:N"
```

## Error Path Evidence

**No separate error-path recordings were created for these ACs** because:

1. The enrichment syntax errors (`E-QUERY-039` unknown UDF) would require removing the
   infusion TOML specs, which would require modifying the demo setup.
2. Prism's pedagogical error path for column gate (`E-QUERY-038`) is already covered
   by S-DEMO-PRISMQL-ONBOARDING-001-B evidence.
3. The primary negative path (IOC fields absent at Stage 0) cannot be reproduced
   without restarting the DTU server with scenario_start_secs set to the present,
   which would invalidate the Stage 4 positive path evidence.

The production enrichment code paths exercise error handling via unit tests
(pipe_execution_tests.rs, enrich_1_pivot_enrich_list_input_test.rs) and
the adversarial cascade (3-CLEAN strict converged).

## Notes on `scripts/demo.toml` modification

The field `scenario_start_secs = 1782214754` was added to the `[orgs.org-c.scenario]`
block in `scripts/demo.toml` to set the scenario clock to ~700 seconds in the past,
ensuring Stage 4 (Containment, elapsed ≥ 600s) is active during recording. This is
a demo configuration field (not production code), documented in the runbook
(T13-capstone-demo-runbook.md §6 capability caveat on scenario_start_secs).

The modification is committed with this evidence to ensure reproducibility.

## Coverage Summary

| AC | Query | Expected Value | Observed Value | Recording |
|----|-------|---------------|----------------|-----------|
| AC-001 | `\| enrich threat_score(iocs_value)` | `threat_score ≥ 75` | `threat_score = 95` | AC-001-threat-score-enrichment.gif/.webm |
| AC-002 | `\| enrich cvss_base_score(device_cves_first)` | `cvss_base_score ≥ 7.0` | `cvss_base_score = 8.1` | AC-002-cvss-enrichment.gif/.webm |
