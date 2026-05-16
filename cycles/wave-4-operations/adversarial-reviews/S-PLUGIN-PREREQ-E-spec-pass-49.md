---
document_type: adversarial-review-pass
pass: 49
cycle: S-PLUGIN-PREREQ-E-spec
date: 2026-05-16
reviewer: adversary
predecessor_pass: 48
predecessor_burst: "FB38 D-657"
verdict: BLOCKED
finding_count: { CRIT: 0, HIGH: 1, MED: 4, LOW: 1, OBS: 0 }
streak_status: "0/3 stays 0/3"
fix_burst: FB39
fix_burst_committed: pending
novelty: HIGH
---

# S-PLUGIN-PREREQ-E Spec — Adversarial Review Pass 49

## §1 Summary

BLOCKED. 1 HIGH + 4 MED + 1 LOW. F-LP49-HIGH-001 is the 13th+ POL-23 cascade-propagation recurrence — META-PATTERN match to F-LP48-HIGH-001. FB38 closed 4 declared sites but missed 5 lateral sites still pinning `error-taxonomy v1.30`. POL-29 codification candidate evidence overwhelming.

## §2 Methodology — 10 Rotated Vectors

1. FB38 close-watch Phase A — CLEAN (4 declared closures verified: ADR-026 line 300, story §Error Taxonomy Additions line 354, error-taxonomy E-PLUGIN-020 message+description, story §FSR Token Budget)
2. POL-25 final-pass workspace grep — surfaced F-LP49-HIGH-001 (5 stale error-taxonomy v1.30 pins in ADR-026, VP-153, HS-001, story)
3. error-taxonomy v1.30→v1.31 sibling propagation completeness — 5 stale pins confirmed across 4 artifacts
4. Token Budget arithmetic — 17,600 verified correct (FB38 reconciliation held)
5. PREREQ-D dependency check — CLEAN (no regressions into PREREQ-D territory)
6. Wave 1/A scope boundary — CLEAN
7. AC↔Postcondition completeness across 4 BCs — surfaced F-LP49-MED-001 (BC-2.01.016 Rule 2/B+2/C lack AC traces), F-LP49-MED-002 (E-SPEC-008 retirement AC verification absent), F-LP49-MED-003 (BC-2.16.012 P6 tracing event field schema not AC-asserted)
8. Red Gate vs Tasks ordering — CLEAN
9. §References vs frontmatter dependencies — surfaced F-LP49-MED-004 (ADR-022 in frontmatter but missing from §References Architecture Compliance) + F-LP49-LOW-001 (HSs in frontmatter but no §References Holdout Scenarios subsection)
10. Cross-changelog narrative consistency — CLEAN

## §3 Findings

### F-LP49-HIGH-001 — error-taxonomy v1.30→v1.31 POL-23 cascade gap (5 sites)

- **Severity:** HIGH (13th+ POL-23 recurrence)
- **Pattern:** META-PATTERN match to F-LP48-HIGH-001 (FB38 declared 4 sites but the POL-25 workspace-wide grep was not run across the full 19-artifact set; 5 lateral sites escaped)
- **Sites:** ADR-026 line 309 (cite v1.30 → needs v1.31), VP-153 line 167 (inline-comment cite v1.30 → needs v1.31), VP-153 line 210 (inline-comment cite v1.30 → needs v1.31), HS-PREREQ-E-001 line 98 (cite v1.30 → needs v1.31), story lines 231+232 (cite v1.30 → needs v1.31)
- **Closure:** FB39 architect (ADR-026 v1.13→v1.14; VP-153 v0.7→v0.8) + PO (story v1.20→v1.21; HS-PREREQ-E-001 v1.3→v1.4)

### F-LP49-MED-001 — BC-2.01.016 Rule 2/B + 2/C lack AC traces

- **Severity:** MED
- **Detail:** BC-2.01.016 defines Rule 2 with three sub-rules (A, B, C). Rule 2/A is covered by existing AC-3. Rules 2/B (error path when SensorAuth impl is valid but credential_type is unknown) and 2/C (panic-free guarantee under adversarial input) have no corresponding AC or Red Gate test in S-PLUGIN-PREREQ-E. The test-writer cannot write covering Red Gate tests without an AC to anchor to.
- **Closure:** FB39 PO — new AC-3b (verifies E-SPEC-013 error code for Rule 2/B) + new AC-3c (verifies E-SPEC-014 error code for Rule 2/C) added to story. Red Gate tests 11→14 (+2 new tests for AC-3b+3c, recount confirms 14 total).

### F-LP49-MED-002 — E-SPEC-008 retirement annotation lacks AC verification

- **Severity:** MED
- **Detail:** Story notes E-SPEC-008 as RETIRED (superseded by E-SPEC-012/013/014) but no AC verifies that the old error code is NOT emitted post-PREREQ-E implementation. A test that confirms E-SPEC-008 absence is the only way to guarantee the retirement is load-bearing rather than nominal.
- **Closure:** FB39 PO — new AC-11 added: "The E-SPEC-008 error code MUST NOT be emitted by prism-spec-engine post-PREREQ-E; any test invoking retired E-SPEC-008 MUST fail compilation." Red Gate test 14 assigned to AC-11.

### F-LP49-MED-003 — BC-2.16.012 P6 tracing event field schema not AC-asserted

- **Severity:** MED
- **Detail:** AC-9 tests that the WARN-level tracing event fires on WriteToolRegistrationAfterBoot, but does not assert the field schema (event_type, plugin_name, tool_name fields) as required by BC-2.16.002 row 33 v1.21. A test covering AC-9 could pass while emitting a malformed event that fails the BC-2.16.002 field audit.
- **Closure:** FB39 PO — AC-9 extended to assert tracing-test capture of WARN event fields per BC-2.16.002 row 33 v1.21. No new Red Gate test required (AC-9 expansion covers within existing RG-009 test).

### F-LP49-MED-004 — ADR-022 in frontmatter but missing from §References Architecture Compliance

- **Severity:** MED
- **Detail:** Story frontmatter lists `architectural_decisions: [ADR-022, ADR-026, ADR-027]` but the §References section Architecture Compliance subsection only cites ADR-026 and ADR-027. ADR-022 (Production Runtime Wiring) is directly relevant (boot sequence, infusion fate, prism-bin chassis) and is referenced in Task 7b/7c. Its omission from §References is an AC traceability gap.
- **Closure:** FB39 PO — §References Architecture Compliance subsection: ADR-022 entry added with brief description.

### F-LP49-LOW-001 — HSs in frontmatter but no §References Holdout Scenarios subsection

- **Severity:** LOW
- **Detail:** Story frontmatter lists `holdout_scenarios: [HS-PREREQ-E-001, HS-PREREQ-E-002, HS-PREREQ-E-003]` but §References has no Holdout Scenarios subsection. Sibling stories (S-PLUGIN-PREREQ-D, S-PLUGIN-PREREQ-B) both carry a §References Holdout Scenarios subsection. Missing subsection = convention asymmetry.
- **Closure:** FB39 PO — §References Holdout Scenarios subsection added with entries for HS-PREREQ-E-001/002/003.

## §4 FB38 Paper-Fix Audit

- 4 declared FB38 closures verified load-bearing: ADR-026 line 300 (v1.20→v1.21 cite updated), story §Error Taxonomy Additions E-PLUGIN-020 row (phrasing updated), error-taxonomy E-PLUGIN-020 message+description (phrasing updated), story §FSR + Token Budget (row added, 17,450→17,600).
- POL-25 workspace grep gap: 5 lateral sites not swept by FB38 architect-adjudication dispatch. POL-29 codification candidate evidence reinforces.

## §5 Sibling-Sweep + Lateral Analysis

13th+ POL-23 recurrence. The 5 stale sites (ADR-026, VP-153 ×2, HS-001, story) all cite `error-taxonomy v1.30` in inline-comment or prose form. FB38's architect-adjudication scope covered only the 4 declared ADR-026/story/error-taxonomy body sites; the POL-25 grep was not run across ADR-026 narrative, VP-153 harness comments, or HS-001 prose. POL-29 codification: mandatory POL-25 workspace grep across full 19-artifact set after every version pin bump, run by architect before completing dispatch.

## §6 Convergence Trajectory + Recommendation

Continue cascade. 6 findings closed by FB39 (1 HIGH + 4 MED + 1 LOW). Streak remains 0/3. Pass-50 begins next 3-CLEAN attempt. Novelty is HIGH on the AC-coverage axis (F-LP49-MED-001/002/003 vectors not exercised in prior 48 passes); AC-coverage findings are convergence-accelerating (each one produces a concrete Red Gate or AC-extension that eliminates the vector permanently).
