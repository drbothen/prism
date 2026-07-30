---
document_type: adversarial-review
pass: 70
scope: wave-a-spec-evolution
frozen_head: ff674a1c
date: 2026-07-30
reviewer: vsdd-factory:adversary
---

# Local Adversarial Pass 70 — Wave-A Spec Evolution Perimeter

> **PROCESS NOTE — Passes 67, 68, 69 not persisted:** Pass reports for passes 67, 68, and 69 were never written to disk; the findings from those passes were addressed via fix-bursts (FB79..FB88) but the adversary report files were not created. This is a process gap (TD-VSDD-096 micro-burst discipline was not applied retroactively; records now lost to session compaction). Going-forward discipline: every adversary pass MUST produce a persisted report file in this directory before the cascade proceeds. This pass-70 report is the first persisted report since pass 66 (local-pass-66.md).

```
CLEAN (strict):   no
CLEAN (PR-merge): no
```

**Pass Statistics:** 1 CRIT · 6 HIGH · 10 MED · 5 LOW · 3 OBS = 25 total findings  
**Novelty:** HIGH — new finding classes (SAC-2 missing anchor_stories, ADR-057 §D7 ratification gap, POL-20/POL-23/input-hash frontmatter normalization)  
**Frozen HEAD:** ff674a1c (develop branch at pass-70 dispatch time)

---

## Probe Results

| Probe | Result | Notes |
|-------|--------|-------|
| SAP-1 (tracing emission catalog completeness) | PASS | No unreferenced `event_type =` sites in `crates/` |
| SAP-2 (DTU↔TOML schema parity — rule 1–6) | PASS | All TOML columns verified against DTU types.rs + wire-emission site |
| TD-VSDD-091/L9 (no volatile line cites) | PASS | No `filename.ext:NNN` patterns in staged additions |
| SAC-1 (enumerated Red Gate list on strict stories) | PASS | Both ARMIS stories have enumerated RG lists |
| SAC-2 (ADR anchor_stories frontmatter) | SEE MED-004 | ADR-023 + ADR-028 missing/stale `anchor_stories` for `S-WAVE-A-ARMIS-SPEC-001` |

---

## Finding Ledger

### CRITICAL

| ID | Severity | File | Summary | Fix Burst | Status |
|----|----------|------|---------|-----------|--------|
| F-WASE-P70-CRIT-001 | CRIT | `S-WAVE-A-ARMIS-SPEC-001` | T-03 files list and §Architecture Mapping omit `routes/devices.rs` and `routes/search.rs` — the actual implementation sites for the three-emitter mask-gate contract (AC-013..AC-015); the T-03 title named only `generator.rs §build_asset`; §Architecture Mapping rows for the two route handlers were absent | FB96 (story-writer) | CLOSED |

### HIGH

| ID | Severity | File | Summary | Fix Burst | Status |
|----|----------|------|---------|-----------|--------|
| F-WASE-P70-HIGH-001 | HIGH | `S-WAVE-A-ARMIS-SPEC-001` | AC-015/RG-015 missing ordering rationale for `device_cves` tombstone position at HighChurn `&limit=200`; without explicit `&limit=200` test assertion the mask-gate behavior at position [180..200] is invisible to the test suite | FB96 (story-writer) | CLOSED |
| F-WASE-P70-HIGH-002 | HIGH | `BC-2.02.014`, `S-WAVE-A-ARMIS-ACTIVITY-001` | §Description attributed fixture loading to `state.rs §ArmisState`; actual fixture-loading entry points are `clone.rs §new_with_seed_anchored` and `clone.rs §new_with_scenario`; `ArmisState::with_admin_token §ArmisState::with_admin_token` merely assigns the fixture to a field | FB95 (product-owner), FB96 (story-writer) | CLOSED |
| F-WASE-P70-HIGH-003 | HIGH | `STORY-INDEX.md` | §Full Story List ACTIVITY-001 row: bolded leading segment shows `draft v1.7` but disk frontmatter is `ready v1.8`; row shows `prism-sensors` only for crates (disk: `[prism-sensors, prism-spec-engine, prism-dtu-armis]`); Pts cell shows 5 (disk: 8). SPEC-001 row: bolded segment shows `draft v1.6` but disk frontmatter is `draft v1.8`; crates cell shows `prism-sensors, prism-dtu-armis` (missing `prism-spec-engine`); Pts cell shows 5 (disk: 8) | FB97 (state-manager) | CLOSED |
| F-WASE-P70-HIGH-004 | HIGH | `BC-INDEX.md` | BC-2.02.014 title cell reads `Armis Device Activity — Filter-Required Push-Down Fetch (Per-Device Activity Surface)` but file H1 is `Armis Device Activity Surface — Filter-Required Push-Down Fetch Contract`; POL-7 requires verbatim H1 in index title cell | FB97 (state-manager) | CLOSED |
| F-WASE-P70-HIGH-005 | HIGH | `ADR-057`, `S-WAVE-A-ARMIS-ACTIVITY-001` | ADR-057 lacked §D7 (Required-Filter Gate Mechanism); T-IMPL-02 in ACTIVITY-001 described the gate implementation as "candidate approaches (non-exhaustive)" without ratifying the mechanism; `required_filters` TOML field absent from T-IMPL-01 TOML block and §D5 step block | FB94 (architect), FB96 (story-writer) | CLOSED |
| F-WASE-P70-HIGH-006 | HIGH | `BC-2.02.006`, `S-WAVE-A-ARMIS-SPEC-001` | `risk_factors` mechanism decision unresolved in BC-2.02.006 §Generated-Records Path Coverage — defect bullet cited "no implementation" without ratifying which of the two proposed mechanisms to use; AC-016/RG-016 absent; SPEC-001 density check still cited 15 ACs/RGTs | FB95 (product-owner), FB96 (story-writer) | CLOSED |

### MEDIUM

| ID | Severity | File | Summary | Fix Burst | Status |
|----|----------|------|---------|-----------|--------|
| F-WASE-P70-MED-001 | MED | `BC-2.02.006` | `device_type` wire-key enumeration in §DeviceRecord block cited `"device_type"` but the serde rename annotation in `DeviceRecord` is `#[serde(rename = "type")]`; TOML column `name` must be `"type"` to match the wire | FB95 (product-owner) | CLOSED |
| F-WASE-P70-MED-002 | MED | `ADR-057` | §D5 downstream-consumer sentence cited `TV-014-001 through TV-014-006` (six phantom IDs); canonical test vector IDs are `TV-BC-2.02.014-001 through TV-BC-2.02.014-005` (five); AC/RG enumeration in §D5 also omitted AC-004/AC-008/RG-008 added by FB91 | FB94 (architect) | CLOSED |
| F-WASE-P70-MED-003 | MED | `S-WAVE-A-ARMIS-ACTIVITY-001` | AC-005, RG-005, and RG-006 used device ID `dev-001` (generated-style format `dev-{org_slug}-{seed}-{i}`) instead of `d-001` (fixture format from `fixtures/device-activity.json`); TV-BC-2.02.014-003 also carried the wrong format | FB96 (story-writer) | CLOSED |
| F-WASE-P70-MED-004 | MED | `ADR-023`, `ADR-028` | SAC-2 rule 1 violation: `ADR-023` `anchor_stories` key entirely absent from frontmatter; `ADR-028` `anchor_stories` list did not include `S-WAVE-A-ARMIS-SPEC-001` (verified `§Authority` citation for both ADRs in that story) | FB94 (architect) | CLOSED |
| F-WASE-P70-MED-005 | MED | `BC-2.02.014` | `introduced: cycle-FB73` in frontmatter violates POL-20 rule 2: fix-burst-introduced BCs must use `YYYY-MM-DD` ISO date; `cycle-FB73` is the opaque burst-ID form explicitly prohibited by POL-20 | FB97 (state-manager) | CLOSED |
| F-WASE-P70-MED-006 | MED | `BC-2.02.014` | Seeded-mode data-unreachability basis stated as "router layer" but subsequent investigation determined this is a generator-architecture gap (no activity builder in `generator.rs §build_asset`) rather than a routing decision; basis citation should acknowledge both the routing behavior (HTTP 200 + empty via matchit path) and the generator absence | DEFERRED Track-B | **DEFERRED** — human direction: superseded by generalized anti-volatile-pin governance; will not be closed until POL-39 is fully canonicalized |
| F-WASE-P70-MED-007 | MED | `BC-2.02.006` | Version-pin de-pinning violation (POL-39 early adoption): two live body sites contained literal version pins ("all seven survive unchanged through v1.5"; "present and unchanged in all versions through v1.5") that decay on subsequent story version advances | FB95 (product-owner) | CLOSED |
| F-WASE-P70-MED-008 | MED | `S-WAVE-A-ARMIS-SPEC-001` | §Behavioral Contracts table BC-2.02.006 title cited non-verbatim paraphrase instead of verbatim H1 "Armis Centrix Field Mapping to OCSF (7 Data Sources)"; POL-7 applies to BC→story reference in §Behavioral Contracts tables | FB96 (story-writer) | CLOSED |
| F-WASE-P70-MED-009 | MED | `S-WAVE-A-ARMIS-SPEC-001` | RG-001..RG-006 lacked `ocsf_field` assertion in test description; §Verification Properties table referenced RG-008..RG-012 but not the extended RG-008..RG-016 range added by FB96 | FB96 (story-writer) | CLOSED |
| F-WASE-P70-MED-010 | MED | `BC-2.02.014`, `S-WAVE-A-ARMIS-ACTIVITY-001` | "or equivalent" escape hatch in four sites across §Postconditions absent-filter bullet, §Error Cases required-behavior clause, TV-BC-2.02.014-002, and §TOML Contract required-filter paragraph; `SpecEngineError::HttpRequestFailed` is the canonical error (E-SPEC-029 per ADR-057 §D7 ratification); "or equivalent" eliminated | FB95 (product-owner), FB96 (story-writer) | CLOSED |

### LOW

| ID | Severity | File | Summary | Fix Burst | Status |
|----|----------|------|---------|-----------|--------|
| F-WASE-P70-LOW-001 | LOW | `S-WAVE-A-ARMIS-ACTIVITY-001` | AC-007 fixture alias `'no-activity-device'` did not match TV-BC-2.02.014-003 canonical ID `'no-such-device'`; RG-007 alias `'empty-device'` also mismatched | FB96 (story-writer) | CLOSED |
| F-WASE-P70-LOW-002 | LOW | `S-WAVE-A-ARMIS-SPEC-001` | AC-013 missing explicit preconditions block clarifying that mask-gate reachability requires BOTH `fixture_gen_seeded=true` AND `timeline: Some(...)` populated only via `ArmisClone::new_with_scenario §new_with_scenario`; a state built via `new_with_seed §new_with_seed` or `new_with_seed_anchored §new_with_seed_anchored` has `timeline: None` and never enters the mask-gate block | FB96 (story-writer) | CLOSED |
| F-WASE-P70-LOW-003 | LOW | `BC-2.02.014` | `timestamp: 2026-07-27T00:00:00` missing Z suffix; POL-23 step 4 P2-02 Direction A adjudicated 2026-06-10 requires full ISO-8601 with Z suffix for new BCs (created post-adjudication); BC-2.02.014 created 2026-07-27 is not grandfathered | FB97 (state-manager) | CLOSED |
| F-WASE-P70-LOW-004 | LOW | `BC-2.02.014`, `ADR-057` | `input-hash: ""` (empty string) in both artifacts; FB73/FB94 were instructed to populate the field but both missed it; input-hash must be populated per VSDD artifact freshness discipline | FB97 (state-manager) — **partial**: BC-2.02.014 `"de1a461"` and ADR-057 `"5439312"` both populated | CLOSED |
| F-WASE-P70-LOW-005 | LOW | `BC-2.02.006`, `S-WAVE-A-ARMIS-SPEC-001` | `device_cves` scope ambiguity: HIGH-001 constraint 1 and Adjudication paragraph did not clarify that `§generate_with_scenario_cves §generate_with_scenario_cves` stamps ALL `n_assets` CompromisedEndpoint assets, not only `catalog.primary_device_id_armis`; three sites required scope clarification | FB95 (product-owner), FB96 (story-writer) | CLOSED |

### OBS

| ID | Severity | File | Summary | Fix Burst | Status |
|----|----------|------|---------|-----------|--------|
| F-WASE-P70-OBS-001 | OBS | `BC-INDEX.md`, `BC-2.02.006` | CAP-003 capability anchor in BC-INDEX title cell description does not reference the exact verbatim text from `capabilities.md §CAP-003`; sweeping to full verbatim compliance would require restructuring the title cell for two enumeration docs | DEFERRED Track-B | **DEFERRED** — human direction: CAP-003 corpus sweep requires restructuring two enumeration doc cells; deferred to dedicated maintenance burst |
| F-WASE-P70-OBS-002 | OBS | `ADR-057` | `anchor_stories: [S-WAVE-A-ARMIS-ACTIVITY-001]` frontmatter was already populated from pass-69 SAC-2 verification (D-2065); probe confirmed correct — this OBS was a false-positive at dispatch time; no action needed | FB94 (architect re-verification) | CLOSED (false positive) |
| F-WASE-P70-OBS-003 | OBS | `STORY-INDEX.md` | §BC Traceability Matrix: `S-WAVE-A-ARMIS-SPEC-001` row in §BC Traceability Matrix showed only `BC-2.02.006` but SPEC-001 frontmatter `behavioral_contracts` contains only `BC-2.02.006`; row is correct as stated — `BC-2.02.014` is an ACTIVITY-001 contract; adversary misread cross-reference; no action needed | N/A | CLOSED (false positive — adversary misread) |

---

## Closure Summary

| Count | Before fix-burst | After FB94 | After FB95 | After FB96 | After FB97 |
|-------|-----------------|-----------|-----------|-----------|-----------|
| Open | 25 | 19 | 14 | 9 | 0 |
| Closed | 0 | 6 | 11 | 16 | 23 |
| Deferred Track-B | 0 | 0 | 0 | 2 | 2 |

**Fix-burst routing:**
- FB94 (architect): CRIT-001 via ADR-057 §D7 (HIGH-005), ADR-057 §D5 (MED-002), ADR-023/ADR-028 SAC-2 (MED-004); total 3 findings + OBS-002 confirmed false-positive
- FB95 (product-owner): HIGH-002(partial), HIGH-006(partial), MED-001, MED-007, MED-010(partial), LOW-005(partial)
- FB96 (story-writer): CRIT-001, HIGH-001, HIGH-002, HIGH-005, HIGH-006, MED-003, MED-008, MED-009, MED-010, LOW-001, LOW-002, LOW-005, OBS-003 confirmed false-positive
- FB97 (state-manager): HIGH-003, HIGH-004, MED-005, LOW-003, LOW-004
- Deferred: MED-006, OBS-001

---

## POL-29 Three-Dimension Sweep Summary

| Dimension | Verdict |
|-----------|---------|
| 9a — Named-twin sweep | REMEDIATED: BC-2.02.006 (named Armis twin) updated in same burst as BC-2.02.014; SPEC-001 and ACTIVITY-001 updated together |
| 9b — Downstream-copy sweep | REMEDIATED: §TOML Contract block (copy-source for ACTIVITY-001 T-IMPL-01) updated with `required_filters` in same cascade leg (FB95); ADR-057 §D5 (copy-source for BC-2.02.014) updated by FB94; all downstream copies refreshed |
| 9c — Mandate-anchor sweep | REMEDIATED: all new MUSTs carry story+AC+RGT anchors; AC-016/RG-016 anchored to `S-WAVE-A-ARMIS-SPEC-001` (FB95/FB96) |
