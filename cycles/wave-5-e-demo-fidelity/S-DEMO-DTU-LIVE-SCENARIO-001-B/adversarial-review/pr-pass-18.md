---
document_type: adversarial-review-pass
pass: 18
level: PR-LEVEL
story: S-DEMO-DTU-LIVE-SCENARIO-001-B
pr: 185
head: 7ddc0a51
timestamp: 2026-06-13T06:00:00Z
streak_before: 2/3
streak_after: 0/3
clean_strict: false
clean_pr_merge: true
findings_count: 1
finding_ids: [BPRL-P18-01]
closure_burst: D-1124
novelty: LOW
---

# PR-LEVEL Pass 18 — S-DEMO-DTU-LIVE-SCENARIO-001-B

**Pass:** 18 (convergence pass) | **PR:** #185 | **HEAD:** 7ddc0a51 (CODE UNCHANGED — no code commits since D-1118)
**Streak before:** 2/3 | **Streak after:** 0/3
**CLEAN(strict):** NO | **CLEAN(PR-merge):** YES

---

## Summary

Pass 18 was the convergence pass at streak 2/3. All code, spec, and integration axes passed
cleanly. One finding surfaced in the demo-evidence artifacts
(`docs/demo-evidence/S-DEMO-DTU-LIVE-SCENARIO-001-B/`): three fabricated or inverted BC
identifiers in AC-019 evidence files (`.tape` header comment and `evidence-report.md` line 89).
The identifiers grep to nothing in the authoritative BC/spec/code. Underlying
tests/ACs/code/BCs/story are ALL CORRECT — only the evidence-artifact prose drifted.
All convergence-positive checks passed (19-AC traceability, POL-12 zero stub residue,
all BC PCs/INVs have RGT coverage, demo evidence 19/19 files present). Streak RESET 2/3 → 0/3.

BPRL-P18-01 was closed in-burst by demo-recorder (feature-branch commit 5d5484d0):
all three anchors corrected in both files. `rg` confirms fabricated names gone, canonical
present. NO re-render needed (anchors were header-comment-only, never displayed in
`.webm`/`.gif`).

---

## Finding

### BPRL-P18-01 — MED — Fabricated/inverted BC identifiers in AC-019 demo-evidence artifacts

**Severity:** MED
**Location:**
- `docs/demo-evidence/S-DEMO-DTU-LIVE-SCENARIO-001-B/AC-019-cyberint-cve-pivot.tape` (header comment)
- `docs/demo-evidence/S-DEMO-DTU-LIVE-SCENARIO-001-B/evidence-report.md` (line 89)

**Three defects (all in the same two files):**

1. **PC-8 and PC-9 labels inverted.** The evidence prose described PC-8 as "baseline namespace
   isolation" and PC-9 as "scenario catalog assignment." The canonical BC-2.06.020 v1.4
   definitions are the reverse: PC-8 = scenario catalog assignment (Cyberint scenario alerts draw
   `cve_id` from `catalog.device_cves`); PC-9 = baseline namespace isolation (non-pivotable
   `CVE-9999-{:04}` format). An analyst or auditor reading the evidence would understand the
   demonstrated behavior as testing the wrong postcondition.

2. **Fabricated invariant `INV-CYBERINT-CVE-PIVOT-001`.** The evidence cited this identifier
   as the governing invariant for the CVE pivot chain. This string does not exist anywhere in
   `.factory/specs/behavioral-contracts/` or `crates/`. The canonical identifier is
   `INV-CYBERINT-ALERT-CVE-CORRELATION-001` (BC-2.06.020 §Invariants, introduced at D-1117).
   A grep-based traceability audit would find BPRL-P18-01 as an orphaned invariant reference.

3. **Fabricated type `CveCorrelationCatalog`.** The evidence referenced this type name as the
   Rust struct that holds per-device CVE assignments. This type does not exist in the codebase.
   The canonical name is `ScenarioEntityCatalog` (introduced at D-1117;
   `prism-dtu-common/src/scenario/mod.rs`). The fabricated name is plausibly constructed but
   grep confirms zero occurrences anywhere in `crates/`.

**Root cause:** The demo-recorder hallucinated three BC identifiers when authoring the AC-019
evidence artifacts. Passes 1-17 audited code, specs, and behavioral contracts but did not
apply BC-anchor verification to the evidence-artifact prose itself (`.tape` comments and
`evidence-report.md` narrative). This is the first instance of this class.

**Impact:** Code, tests, BCs, story, and all 18 prior ACs are correct and unaffected.
The three fabricated identifiers exist only in evidence-artifact prose. A traceability
auditor running `grep INV-CYBERINT-CVE-PIVOT-001` or `grep CveCorrelationCatalog` would
find zero hits in the authoritative corpus and correctly flag the evidence as
non-compliant with the BC anchor it claims to demonstrate.

---

## Closure (D-1124 — demo-recorder, feature-branch commit 5d5484d0)

All three defects corrected in both files in a single commit:

1. **PC-8 ↔ PC-9 labels corrected:** PC-8 = scenario catalog assignment; PC-9 = baseline
   namespace isolation. Both files updated to canonical labels.

2. **`INV-CYBERINT-CVE-PIVOT-001` → `INV-CYBERINT-ALERT-CVE-CORRELATION-001`:** Both files
   updated. `rg INV-CYBERINT-CVE-PIVOT-001 docs/` returns zero hits.
   `rg INV-CYBERINT-ALERT-CVE-CORRELATION-001 docs/` confirms canonical present.

3. **`CveCorrelationCatalog` → `ScenarioEntityCatalog`:** Both files updated.
   `rg CveCorrelationCatalog docs/` returns zero hits.
   `rg ScenarioEntityCatalog docs/` confirms canonical present.

NO re-render of `.webm` or `.gif` required: the corrected anchors were in header comment
lines that are never displayed in the recorded output. Demo fidelity is unaffected.

Feature HEAD after commit: `5d5484d0` (= remote after push).

---

## Convergence-Positive Checks (all PASS before BPRL-P18-01 surfaced)

### 19-AC Traceability

All 19 ACs traced to Red Gate tests. No AC without an RGT. No RGT without an AC.
23 Red Gate tests present (frontmatter `red_gate_tests: 23`; Phase-6 gate "all 23 Red Gate
tests pass"; D-1121 BPRL-P15-01 closed). PASS.

### POL-12 Zero Stub Residue

No `todo!()`, `unimplemented!()`, `panic!("not yet implemented")`, or `TODO: implement`
markers in new code paths. PASS.

### All BC PCs/INVs Have RGT Coverage

BC-2.06.019 v1.7 all Postconditions covered by RGT rows.
BC-2.06.020 v1.4 all Postconditions (PC-1 through PC-9) + INV-CYBERINT-ALERT-CVE-CORRELATION-001
covered by RGT rows VP-020-I through VP-020-L and VP-020-A through VP-020-H. PASS.

### Demo Evidence File Count

19 evidence files present under
`docs/demo-evidence/S-DEMO-DTU-LIVE-SCENARIO-001-B/` at HEAD 7ddc0a51 (commit f75f3159
added AC-019; 19/19). PASS.

---

## Verification Axes Not Repeated (prior passes sufficient)

The following axes were verified exhaustively in passes 13-17 with code unchanged at
7ddc0a51. Not re-run at pass 18:

- Holdout-style behavioral trace (5 stages × 6 clones) — pass 17 PASS
- Cross-BC consistency (BC-2.06.019 v1.7 ↔ BC-2.06.020 v1.4) — pass 17 PASS
- `build_clone_pairs` wiring (guard order E-DEMO-002→006→003→004) — pass 17 PASS
- SAP-1 tracing emission catalog — passes 16-17 PASS
- S-7.01 SEC-001 sibling-drift (CVE-9999-{:05}) — pass 17 PASS
- Forbidden patterns sweep — pass 17 PASS
- POL-22 A+C — pass 16 PASS

---

## Do-Not-Reflag Carry Forward

All BPRL-P1 through BPRL-P17 do-not-reflag entries carry forward. BPRL-P18-01 added:

- **BPRL-P18-01 CLOSED (D-1124):** AC-019 evidence anchors corrected — PC-8 = scenario
  catalog assignment / PC-9 = baseline namespace isolation (canonical); fabricated
  `INV-CYBERINT-CVE-PIVOT-001` → `INV-CYBERINT-ALERT-CVE-CORRELATION-001`; fabricated
  `CveCorrelationCatalog` → `ScenarioEntityCatalog`. Feature HEAD advanced to 5d5484d0.
  **DO NOT re-raise "inverted PC-8/PC-9 labels", "INV-CYBERINT-CVE-PIVOT-001 not found",
  or "CveCorrelationCatalog not found" — CLOSED.**

---

## Pass Status

```
CLEAN (strict): NO — BPRL-P18-01 MED (fabricated BC anchors in AC-019 evidence artifacts)
CLEAN (PR-merge): YES — ZERO findings of CRIT + HIGH + MED in code/spec (evidence-prose only)
Streak: 2/3 → 0/3 (RESET by BPRL-P18-01)
Closure: D-1124 (demo-recorder commit 5d5484d0; all 3 anchors corrected; push complete)
NEXT: PR-LEVEL pass 19 at HEAD 5d5484d0
NOTE: diff CHANGED (5d5484d0 is 1 commit ahead of prior 7ddc0a51) — re-materialize via
      `gh pr diff 185` (do NOT reuse /tmp/pr185-pass13.diff or prior stale diffs)
```
