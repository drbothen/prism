---
document_type: adversarial-review-pass
pass: 23
level: PR-LEVEL
story: S-DEMO-DTU-LIVE-SCENARIO-001-B
pr: 185
head: 0863184a
diff: unchanged (same as pass 20; last code change pass 13 / D-1117/D-1118)
date: 2026-06-13
clean_strict: YES
clean_pr_merge: YES
streak: 1/3
findings: 0
novelty: LOW
recorded_by: state-manager (D-1130)
---

# PR-LEVEL Adversarial Pass 23 — S-DEMO-DTU-LIVE-SCENARIO-001-B

## Result

**CLEAN(strict): YES**
**CLEAN(PR-merge): YES**
**Streak: 1/3** (pass 23 advances streak from 0/3 to 1/3)
**Findings: 0**
**Novelty: LOW**

## Scope

- PR #185; HEAD `0863184a`; diff UNCHANGED since pass 20 (code logic unchanged since pass 13)
- Streak prior to this pass: 0/3 (reset by BPRL-P22-01 MED D-1128)
- Consistency-sweep D-1129 closed 3 MAJOR spec-text drifts (DRIFT-1/2/3) before this pass — consistency gate, streak unchanged at 0/3 entering pass 23

## Verification Axes (all PASS)

### 1. DRIFT-2/3 Closure Re-Verification (Cyberint new_with_scenario 6-arg consistency)

D-1129 closed DRIFT-2/3: story B §Tasks/FSR/build_clone_pairs Cyberint `new_with_scenario` 5-arg stale prose corrected to 6-arg. Adversary independently re-derived from four independent sources:

- **Code (0863184a):** `crates/prism-dtu-cyberint/src/clone.rs` — `new_with_scenario` signature is `(config, seed, org_id, time_anchor, catalog, state)` — **6-arg** CONFIRMED
- **Harness (0863184a):** `crates/prism-dtu-demo-server/src/harness.rs` call site ~line 776 — `CyberintClone::new_with_scenario(config, seed, org_id, time_anchor, &catalog, state)` — **6-arg** CONFIRMED
- **BC-2.06.020 PC-8:** `CyberintClone::new_with_scenario(config, seed, org_id, time_anchor, &catalog: &ScenarioEntityCatalog, state)` — **6-arg** CONFIRMED
- **Story B §Tasks build_clone_pairs (v2.15):** `CyberintClone::new_with_scenario(config, seed, org_id, time_anchor, &catalog, state)` — **6-arg** CONFIRMED (DRIFT-2/3 closed)

Other clones: Armis/Claroty/CrowdStrike = 5-arg (no catalog parameter — correct per PC-8 scope: only Cyberint requires catalog for CVE correlation). ThreatIntel/NVD = 1-arg (static fixture, no scenario progression). All consistent. **PASS.**

### 2. NVD CVE Pivot Case-Consistency Re-Derivation (VP-020-K soundness)

Fresh derivation of case-sensitivity soundness from first principles:

- **Registry path (0863184a):** `crates/prism-dtu-nvd/src/routes/vulnerabilities.rs` — `cve_id.to_uppercase()` applied before HashMap insert → registry keyed in UPPERCASE
- **Lookup path (0863184a):** `NvdState::lookup_and_count(cve_id)` — `cve_id.to_uppercase()` applied before lookup → lookup normalized to UPPERCASE
- **Scenario catalog (0863184a):** `gen_device_cves` emits `CVE-9999-{:05}` — already uppercase sentinel format
- **Conclusion:** Both registry and lookup normalize to uppercase independently → any case of incoming CVE ID resolves correctly; VP-020-K case-sound invariant holds by construction. **PASS.**

### 3. BC-2.06.020 PC-8 Cyclic Catalog Assignment Consistency

PC-8 specifies: scenario-mode Cyberint CVE IDs assigned cyclically from `catalog.device_cves`. Verified:

- `generate_cves` in `prism-dtu-common/src/scenario/mod.rs` uses `catalog.device_cves[i % catalog.device_cves.len()]` pattern — cyclic by index modulo length. **PASS.**
- `catalog.device_cves` is populated from `gen_device_cves` which emits `CVE-9999-{:05}` (5-digit sentinel). Same catalog entries are inserted into the NVD registry. End-to-end correlation chain is closed. **PASS.**

### 4. BC-2.06.020 PC-9 Baseline Namespace Isolation

PC-9 specifies: Cyberint baseline (non-scenario) CVE IDs use `CVE-9999-{:04}` (4-digit). These are intentionally non-pivotable (do not appear in NVD registry). Verified:

- `prism-dtu-cyberint/src/generator.rs` baseline generator uses `CVE-9999-{:04}` — **PASS**
- Distinction from `CVE-9999-{:05}` (5-digit, scenario catalog) is intentional; PC-9 by-design; `\d{4}` invariant consistent with `0..10000` range per BPRL-P14-01 closure. **PASS.**

### 5. E-DEMO-002 Prescan Guard Ordering

BC-2.06.019 PC-2 and story B AC-005 require E-DEMO-002 prescan (org_id validation) before E-DEMO-003 (stage_mask compute) and before E-DEMO-004 (clone routing). Verified guard ordering in `build_clone_pairs` / harness entry path (0863184a):

- Guard order: E-DEMO-002 org_id prescan → E-DEMO-006 org_id guard → E-DEMO-003 stage_mask → E-DEMO-004 clone routing. **PASS.**

### 6. SAP-1 — Structured Event Catalog Completeness

Grepped `event_type =` across `crates/` for Story B additions. All `event_type` values touched by PR #185 diff are either:
- Pre-existing catalog entries (no new values added), or
- Diagnostic `tracing::error!` / `tracing::warn!` without `event_type` field (SAP-1 scope-exempt per PO OBS-1 ruling)

No new `event_type =` emission sites without BC-2.16.002 catalog rows detected. **PASS.**

### 7. All BPRL-P1 through BPRL-P22 Do-Not-Reflag Items

Confirmed all closures in the do-not-reflag list (SESSION-HANDOFF §4) remain closed:

- BPRL-P1-01 through BPRL-P4-02: code closures at commits 45323267/4eadb027/2323cf37/13efc875/bc0f36c5 — verified load-bearing tests present. **PASS.**
- SEC-001: `CVE-9999-{:05}` sentinel in place of `CVE-202x-*` format — **PASS.**
- BPRL-P12-01: VP-020-K genuine integration test at `prism-dtu-demo-server/tests/bc_2_06_020_cyberint_nvd_pivot.rs` — confirmed present 0863184a. **PASS.**
- BPRL-P14-01: BC-2.06.020 PC-9 `0..10000` consistent with code and `\d{4}` invariant — **PASS.**
- BPRL-P22-01: BC-2.06.020 §VP Anchors prose reads `VP-020-A through VP-020-L` / `all 12 VPs` (v1.5) — **PASS.**
- DRIFT-1/2/3 (D-1129): STORY-INDEX pin `v1.5`; story B §Tasks/FSR/build_clone_pairs Cyberint `new_with_scenario` 6-arg — **PASS.**

### 8. Artifact Cluster Convergence Assessment

After 23 passes and the D-1129 consistency-sweep:

- Code at 0863184a: unchanged since pass 13 (demo-recorder re-record only change since pass 12 fix-burst)
- BC-2.06.019 v1.7: Route Coverage Table 8-row exhaustive, all rows verified accurate
- BC-2.06.020 v1.5: PC-8 6-arg catalog, PC-9 4-digit baseline, VP Anchors `A..L` / 12 VPs, all invariants consistent
- Story B v2.15: 19 ACs, 23 RGTs, 6-arg Cyberint in all §Tasks prose sites
- BC-INDEX v6.42: rows 119/120 pin v2.15, both accurate
- STORY-INDEX v2.368: PIVOT-003 row annotation `v1.5`, accurate
- Demo evidence 19/19 ACs complete; VP-020-K covered in two-crate split recording

**The artifact cluster has converged.** All specification prose, behavioral contracts, verification properties, demo evidence, and code are mutually consistent. No novel angles remain unexplored at the current scope boundary.

## Conclusion

Zero findings of any severity. CLEAN(strict)=YES; CLEAN(PR-merge)=YES. Streak advances to **1/3**.

**NEXT: PR-LEVEL pass 24 at HEAD 0863184a** (diff UNCHANGED — reuse /tmp/pr185-pass20.diff or `gh pr diff 185`; no CI push; do-not-reflag list = all prior closures including DRIFT-1/2/3 D-1129 and all BPRL-P1..P22 closures).
