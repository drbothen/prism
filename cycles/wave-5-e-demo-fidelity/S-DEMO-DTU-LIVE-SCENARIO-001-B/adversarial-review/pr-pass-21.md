---
document_type: adversarial-review-pass
pass: 21
level: PR-LEVEL
story: S-DEMO-DTU-LIVE-SCENARIO-001-B
pr: 185
head: 0863184a
timestamp: 2026-06-13T10:00:00Z
streak_before: 1/3
streak_after: 2/3
clean_strict: true
clean_pr_merge: true
findings_count: 0
finding_ids: []
closure_burst: null
novelty: LOW
---

# PR-LEVEL Pass 21 — S-DEMO-DTU-LIVE-SCENARIO-001-B

**Pass:** 21 | **PR:** #185 | **HEAD:** 0863184a (CODE LOGIC UNCHANGED since pass 13)
**Streak before:** 1/3 | **Streak after:** 2/3
**CLEAN(strict):** YES | **CLEAN(PR-merge):** YES

---

## Summary

Pass 21 ran 8 independent re-derivation axes from scratch — full stage-timing
math, StageMask table, error-taxonomy completeness, ADR-036 constructor-signature
conformance, BC frontmatter crates: cross-check, AC-019 cyclic-catalog determinism,
and SAP-1. Every property matched exactly. Zero findings of any severity.
Adversary characterizes the artifact as "genuinely converged."

**8 independent re-derivation axes verified from scratch:**

1. **Stage-timing math (6 Test Vectors from scratch):** Full independent
   re-derivation of TV-020-A through TV-020-F (stage boundaries × seed × clock
   offset → expected events per stage). All 6 matched BC-2.06.019 PC-2 byte-for-byte.

2. **StageMask table (5×6 from scratch):** Independent construction of the
   5-stage × 6-clone StageMask activation matrix. All 30 cells matched
   BC-2.06.019 PC-2 Route Coverage Table exactly.

3. **Error-taxonomy E-DEMO-001..006 completeness:** Each error code traced through
   error-taxonomy.md → BC-2.06.019/020 postcondition anchor → implementation guard
   in `build_clone_pairs`. All 6 entries present with correct namespace, code,
   message pattern, and guard location. PASS.

4. **ADR-036 v2.3 §2.4 constructor-signature conformance (all 6 clones):** Each
   of the 6 DTU clone constructors (Armis, Claroty, CrowdStrike, Cyberint, NVD,
   ThreatIntel) re-verified against ADR-036 v2.3 §2.4 `new_with_scenario` signature
   requirement: `(&catalog, seed, org_id)` 3-arg form. All 6 conformant. PASS.

5. **BC frontmatter `crates:` cross-check:** BC-2.06.019 and BC-2.06.020 frontmatter
   `crates:` arrays re-verified against the actual production modules touched by
   each BC's postconditions. No phantom crate entries; no missing crate entries.
   PASS.

6. **AC-019 cyclic-catalog determinism:** Independent re-derivation of the cyclic
   assignment algorithm (device CVE catalog assignment `cve_id = catalog.device_cves[i % len]`)
   — verified deterministic for fixed seed + catalog size; verified catalog is
   populated before cyclic assignment in scenario-mode Cyberint clone. PASS.

7. **SAP-1 (tracing emission catalog completeness):** Full grep of `event_type =`
   across entire crates/ workspace (not just diff). Zero new `event_type` values
   introduced by PR diff. All existing catalog rows remain complete. PASS.

8. **All BPRL-P1 through BPRL-P20 do-not-reflag items:** Every prior closure
   independently re-confirmed still intact at HEAD 0863184a. PASS.

---

## Convergence-Positive Checks (all PASS)

All prior convergence-positive checks from passes 13-20 carried forward.
Feature HEAD at 0863184a is code-unchanged since 7ddc0a51 (D-1117/P12/P14/P15/
P18/P19 changed only evidence artifacts, spec prose, and demo recordings — no
production Rust code changes after 7ddc0a51).

- BC-2.06.019 v1.7 all Postconditions covered by RGT rows. PASS.
- BC-2.06.020 v1.4 all Postconditions (PC-1 through PC-9) + INV-CYBERINT-ALERT-CVE-CORRELATION-001
  covered by RGT rows VP-020-A through VP-020-L. PASS.
- BC-INDEX rows 119/120 both annotate story pin `ready v2.13 (D-1121 2026-06-13)`.
  PASS.
- Story B v2.13 Phase-6 gate instruction reads "all 23 Red Gate tests pass". PASS.
- SAP-1 (tracing emission catalog): no new `event_type` values in diff. PASS.
- SAP-2 (DTU↔TOML schema parity): N/A — no sensor TOML in diff. PASS.
- Forbidden-pattern sweep: no `reqwest::Client::new()` without timeout, no
  `unwrap()` in critical paths, no `println!` in production code. PASS.
- POL-12 zero stub residue: no `todo!()`, `unimplemented!()`. PASS.
- POL-22 A+C PASS.
- Demo evidence file count: 19/19. PASS.
- All BPRL-P1 through BPRL-P20 do-not-reflag items confirmed still closed. PASS.

---

## Do-Not-Reflag Carry Forward

All BPRL-P1 through BPRL-P20 do-not-reflag entries carry forward unchanged.
No new entries added this pass (zero findings).

---

## Pass Status

```
CLEAN (strict): YES — ZERO findings of ANY severity
CLEAN (PR-merge): YES — ZERO findings of CRIT + HIGH + MED severity
Streak: 1/3 → 2/3
Novelty: LOW — "the artifact has genuinely converged"
NEXT: PR-LEVEL pass 22 (convergence pass) at HEAD 0863184a (diff unchanged — reuse /tmp/pr185-pass20.diff; code unchanged; NO CI push needed)
Post-convergence sequence (after 3/3): pr-reviewer RE-RUN + security-reviewer RE-RUN on 0863184a
(code changed via D-1117/P12/P14/P15/P18/P19 since pass-11 reviews on bc0f36c5) → CI green →
admin squash-merge → POL-14 burst (BC-2.06.019 v1.7 + BC-2.06.020 v1.4 draft→active).
CLAUDE.md EXPECTED 50→52 already in-PR (D-1108).
```
