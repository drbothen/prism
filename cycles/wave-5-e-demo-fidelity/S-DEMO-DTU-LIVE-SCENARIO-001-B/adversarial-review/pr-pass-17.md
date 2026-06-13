---
document_type: adversarial-review-pass
pass: 17
level: PR-LEVEL
story: S-DEMO-DTU-LIVE-SCENARIO-001-B
pr: 185
head: 7ddc0a51
timestamp: 2026-06-13T05:00:00Z
streak_before: 1/3
streak_after: 2/3
clean_strict: true
clean_pr_merge: true
findings_count: 0
finding_ids: []
closure_burst: D-1123
novelty: LOW
---

# PR-LEVEL Pass 17 — S-DEMO-DTU-LIVE-SCENARIO-001-B

**Pass:** 17 | **PR:** #185 | **HEAD:** 7ddc0a51 (CODE UNCHANGED — no code commits since D-1118)
**Streak before:** 1/3 | **Streak after:** 2/3
**CLEAN(strict):** YES | **CLEAN(PR-merge):** YES

---

## Summary

Pass 17 ran a full holdout-style behavioral trace across all 5 attack stages × all 6 DTU clones,
cross-BC consistency verification (BC-2.06.019↔BC-2.06.020 shared catalog/seed), full
`build_clone_pairs` wiring verification (guard order E-DEMO-002→006→003→004, all 6 constructors),
POL-12 stub-residue check, SAP-1 tracing catalog check, and S-7.01 partial-fix regression check
(SEC-001 CVE-sentinel propagation, no sibling drift). Zero findings of any severity. Adversary
characterizes the diff as "genuinely coherent end-to-end."

---

## Verification Axes (all PASS)

### Holdout-Style Behavioral Trace (5 attack stages × 6 clones)

All five attack stages (recon → lateral movement → exfil → containment → post-containment)
traced through all six clone pairs under scenario engine:

**Armis clone:**
- Stage 0 (recon): device inventory with no StageMask guard — unguarded route, baseline device
  records populated. PASS.
- Stage 1 (lateral movement): `mask.lateral_devices` guard fires; devices route returns enriched
  device set. StageMask projection applied via `with_stage_mask_projection`. PASS.
- Stage 2+ (exfil/containment/post): stage-saturation arithmetic (ADR-036 5-arg ruling) preserves
  terminal stage; no index-out-of-bounds. PASS.

**Claroty clone:**
- `GET /api/v2/devices` StageMask-guarded per BC-2.06.019 v1.7 Route Coverage Table row 8
  (Claroty devices; stage: `mask.lateral_devices`; GUARDED). PASS.
- `GET /api/v1/audit_log` unguarded (baseline); stage-invariant fixture data returned at all stages.
  PASS.
- org_id guard (PRE-6 per BC-2.06.019 v1.7) fires before stage_mask compute — correct guard order
  E-DEMO-006 before E-DEMO-003/004. PASS.

**CrowdStrike clone:**
- Detections route: stage 0 guard (`mask.containment` check, guarded per Route Coverage Table row 3)
  returns empty at stage 0; populates at stage >= containment. PASS.
- Summaries route: `POST /detects/entities/summaries/GET/v1` method+path matches Route Coverage
  Table row 4 (BC-2.06.019 v1.5 correction). PASS.
- Hosts route: `GET /devices/v1/devices/scroll` UNGUARDED; returns device inventory at all stages.
  PASS.

**Cyberint clone:**
- Scenario-mode alerts carry `cve_id` drawn from `catalog.device_cves` (cyclic assignment per
  BC-2.06.020 PC-8 + INV-CYBERINT-ALERT-CVE-CORRELATION-001). End-to-end NVD pivot chain intact:
  alert.cve_id → NVD lookup → CVE record resolves. PASS.
- `CyberintClone::new_with_scenario` takes `&catalog` parameter (D-1117 wiring). PASS.
- IOC Interim State clause (BC-2.06.019 v1.4): production `_ioc_value` sentinel acknowledged;
  BPRL-P4-01 CLOSED-BY-DEFERRAL per D-1109 — NOT re-raised. PASS.

**NVD clone:**
- Scenario-correlated CVE-9999-{:05} sentinel IDs (year 9999; SEC-001 closed D-1117) populate
  `device_cves` catalog. NVD `lookup_and_count` resolves these IDs to fixture CVE records. PASS.
- Genuine demo-server integration test at
  `crates/prism-dtu-demo-server/tests/bc_2_06_020_cyberint_nvd_pivot.rs::
  test_BC_2_06_020_cyberint_alert_cve_resolves_in_nvd` (commit 9219ce76). PASS.

**ThreatIntel clone:**
- Static-fixture enrichment DTU; scenario-correlated IOC seeding acknowledged as Interim State
  (BPRL-P4-01 CLOSED-BY-DEFERRAL). Not re-raised. PASS.

---

### Cross-BC Consistency (BC-2.06.019 ↔ BC-2.06.020)

- Shared `CatalogSeed` / `DeviceCveCatalog` contract: BC-2.06.019 PRE-6 org_id guard applied before
  catalog lookup; BC-2.06.020 INV-CYBERINT-ALERT-CVE-CORRELATION-001 catalog draw requires
  `new_with_scenario(&catalog)` wiring. Both BCs reference same catalog shape — no contradiction.
  PASS.
- BC-2.06.019 v1.7 Route Coverage Table (8 rows exhaustive) ↔ BC-2.06.020 v1.4 PC-8/PC-9: no
  overlap or contradiction. PASS.
- BC-2.06.020 PC-9 `rng.gen_range(0..10000)` ↔ `^CVE-9999-\d{4}$` invariant ↔ TV-020-011 ↔
  shipped code: all four surfaces consistent (BPRL-P14-01 closed D-1120). PASS.

---

### `build_clone_pairs` Wiring (Guard Order Verification)

Full constructor verification for all 6 DTU clones in `build_clone_pairs`:

1. E-DEMO-002 (seed/catalog init) fires first — catalog populated before any clone constructor
   receives `&catalog`. PASS.
2. E-DEMO-006 (org_id guard, PRE-6) fires before E-DEMO-003 (stage_mask compute). PASS.
3. E-DEMO-003 (stage_mask compute) fires before E-DEMO-004 (route dispatch). PASS.
4. E-DEMO-004 (route dispatch) resolves last — all guards satisfied before dispatch. PASS.

All 6 clone constructors (`ArmisClone`, `ClarotyClone`, `CrowdStrikeClone`, `CyberintClone`,
`NvdClone`, `ThreatIntelClone`) verified wired with correct parameter order and `Arc<>`
threading. No `Arc::new(SomeThing::placeholder())` boot-path stubs. PASS.

---

### POL-12 Stub-Residue Check

No `todo!()`, `unimplemented!()`, or `panic!("not yet implemented")` macros in new code paths.
No doc-comment `TODO: implement` markers on shipped behavior. PASS.

---

### SAP-1 (Tracing Emission Catalog Completeness)

`rg 'event_type\s*=' crates/ --type rust` — zero new unregistered `event_type` values in PR diff
at HEAD 7ddc0a51. All tracing emissions in new code paths either (a) use pre-registered
catalog values or (b) are non-audit diagnostic `tracing::error!` entries explicitly exempt per
PO OBS-1 ruling (do-not-reflag). PASS.

---

### S-7.01 Partial-Fix Regression Check (SEC-001 Propagation)

SEC-001 (`gen_device_cves` CVE-202x-* → CVE-9999-{:05} sentinel, D-1117) verified:

- `prism-dtu-common/src/scenario/mod.rs` `gen_device_cves`: uses `CVE-9999-{:05}` format. PASS.
- All 6 DTU clone scenario engines that call `gen_device_cves` (or receive the catalog from it):
  no sibling using the old `CVE-202x-*` format. Sibling-sweep PASS (TD-VSDD-060).
- BC-2.06.020 PC-9 baseline (non-scenario) uses `CVE-9999-{:04}` format (intentionally
  non-pivotable per PC-9 by-design; do-not-reflag). PASS.

No regressions or missed sibling sites from the SEC-001 fix. PASS.

---

### Forbidden Patterns Sweep

- No `prism_spec_engine::types::ColumnType` shadow enum variants. PASS.
- No `lifecycle: active` BC frontmatter (all use `lifecycle_status`). PASS.
- No `OrgSlug::new_unchecked` in production code paths. PASS.
- No `Arc::new(SomeThing::placeholder())` boot-path stubs. PASS.
- No `reqwest::Client::new()` without `.timeout()` in production code. PASS.
- No `unwrap()`/`expect()` on `Result` in non-test code paths. PASS.
- No silent `Vec::new()` returns where partial-failure data should propagate. PASS.
- No `println!` in production code. PASS.

---

### Prior Closures Verified (Do-Not-Reflag)

All BPRL-P1 through BPRL-P16 do-not-reflag items confirmed still closed. Key closures verified:

- BPRL-P4-01 CLOSED-BY-DEFERRAL (IOC-surface Interim State; D-1109). NOT re-raised. PASS.
- SEC-001 CLOSED (CVE-9999-{:05} sentinel; D-1117). NOT re-raised. PASS.
- D-1117 CVE↔NVD correlation CLOSED (`new_with_scenario(&catalog)` wiring; D-1117). NOT re-raised. PASS.
- BPRL-P12-01 CLOSED (VP-020-K false-green; D-1118). NOT re-raised. PASS.
- BPRL-P14-01 CLOSED (BC-2.06.020 v1.4 PC-9 `0..10000`; D-1120). NOT re-raised. PASS.
- BPRL-P15-01 CLOSED (story B Phase-6 gate "all 23 Red Gate tests pass"; D-1121). NOT re-raised. PASS.
- Pass-13 cosmetic nit (stale doc-comment in bc_2_06_020_cyberint_nvd_pivot.rs; D-1119
  adjudication). NOT re-raised. PASS.
- Pass-16 sub-threshold disposition (story line-47 "~16 tests" tilde-qualified estimate;
  D-1122 adjudication). NOT re-raised. PASS.

---

### Demo Evidence

19/19 ACs complete (commit f75f3159; VHS recordings in
`docs/demo-evidence/S-DEMO-DTU-LIVE-SCENARIO-001-B/`). PASS.

---

### Novelty Assessment

Novelty: LOW. Pass 17 applied behavioral-trace axes (5 stages × 6 clones) and cross-BC
consistency angles not previously covered as a combined structured pass. The diff has been
reviewed exhaustively across 17 passes — the code is production-grade and the spec
surfaces are fully consistent. No new attack surfaces identified.

---

## Pass Status

```
CLEAN (strict): YES — ZERO findings of ANY severity (CRIT + HIGH + MED + LOW + OBS + PROCESS-GAP)
CLEAN (PR-merge): YES — ZERO findings of CRIT + HIGH + MED severity
Streak: 1/3 → 2/3
NEXT: PR-LEVEL pass 18 (convergence pass) at 7ddc0a51 (diff UNCHANGED — reuse /tmp/pr185-pass13.diff or `gh pr diff 185`; NO CI push needed)
If pass 18 CLEAN(strict)=YES → 3/3 PR-LEVEL CONVERGENCE → post-convergence sequence:
  1. pr-reviewer re-run APPROVE (on head 7ddc0a51 — code changed via D-1117 since pass-11 reviews on bc0f36c5; MUST re-run)
  2. security-reviewer re-run MAY PROCEED (same reason — MUST re-run on 7ddc0a51)
  3. CI green on 7ddc0a51
  4. squash-merge to develop
  5. Post-merge state-manager burst: POL-14 (BC-2.06.019 v1.7 + BC-2.06.020 v1.4 draft→active); STORY-INDEX status; STATE bump
  NOTE: CLAUDE.md EXPECTED 50→52 DONE in-PR per D-1108 — NO post-merge human edit needed
```
