---
document_type: pr-level-cascade-disposition
story: S-DEMO-ENRICHMENT-PIVOT-003
pr: "#196"
pr_head: "192428db"
round: 1
date: 2026-06-20
decision: D-1257
state_version: "7.885"
---

# PIVOT-003 PR #196 — PR-LEVEL Cascade Round 1 Disposition

## Summary

PR-LEVEL cascade round 1 on frozen pushed HEAD `192428db` (PR #196,
feature/S-DEMO-ENRICHMENT-PIVOT-003 → develop). 3 fresh-context independent
adversary passes dispatched per BC-5.39.001 + DRIFT-ORCH-ADVERSARY-TUPLE-001.

## Pass Results

| Pass | CLEAN(strict) | CLEAN(PR-merge) | Findings |
|------|---------------|-----------------|----------|
| Pass 1 | no | yes | OBS-1 (evidence-report HEAD citation), OBS-2 (NVD registry test approach) |
| Pass 2 | yes | yes | none |
| Pass 3 | no | yes | OBS-PASS3-001 (NVD premise extension of OBS-2) |

**Streak: 0/3 strict** (DRIFT-ORCH-PRLEVEL-PUSH-001 — HEAD 192428db frozen, no
new pushes; streak resets only if findings emerge and are genuinely unresolved).

## OBS Findings and Adjudications

### OBS-1 (Pass 1) — Evidence-report cites code HEAD 62d4fcdb vs PR HEAD 192428db

**Finding:** The evidence-report.md stored at
`docs/demo-evidence/S-DEMO-ENRICHMENT-PIVOT-003/` cites the code HEAD as
`62d4fcdb` while the PR HEAD (192428db) is a different commit. The adversary
raised this as a potential evidence staleness concern.

**Orchestrator Adjudication: SETTLED NON-DEFECT — DO-NOT-REFLAG**

Evidence:
```
git diff 62d4fcdb..192428db
```
Result: 17 files changed, 597 insertions(+), 0 deletions(−). **Every single
changed file is under `docs/demo-evidence/S-DEMO-ENRICHMENT-PIVOT-003/`.**
Zero source files, zero spec files, zero test files changed.

`192428db` IS the demo-evidence commit itself. The evidence-report was captured
against code HEAD `62d4fcdb` (the LOCAL 3-CLEAN converged HEAD) before the
evidence commit existed. The cited code-HEAD in the evidence-report is
correct-by-construction: the evidence is always captured against the code HEAD,
and the evidence commit is created after. The resulting PR HEAD (`192428db`) is
the code HEAD + the evidence commit. There is no staleness and no coverage gap.

**DO-NOT-REFLAG identifier:** `PIVOT-003-PRLEVEL-OBS-1`

### OBS-2 (Pass 1) + OBS-PASS3-001 (Pass 3) — NVD pivot tests use hand-built NvdState registry

**Finding (Pass 1 / OBS-2):** The BC-2.06.019 NVD pivot acceptance tests
construct a hand-built `NvdState` registry directly rather than using
`NvdClone::new_with_scenario`. The adversary questioned whether this covers
the `NvdClone::new_with_scenario` constructor path.

**Finding Extension (Pass 3 / OBS-PASS3-001):** Pass 3 extended the premise
to claim no load-bearing test exists anywhere in the codebase for
`NvdClone::new_with_scenario`.

**Orchestrator Adjudication: SETTLED NON-DEFECT / CROSS-STORY COVERED — DO-NOT-REFLAG**

Evidence — `NvdClone::new_with_scenario` IS load-bearing-tested:

File: `crates/prism-dtu-nvd/tests/bc_2_06_020_nvd_enrichment.rs`
Test: `test_BC_2_06_020_nvd_cve_correlation_high_cvss_base_score` (RGT #14)

- **L61:** Constructs `NvdClone::new_with_scenario` (the constructor under question)
- **L120:** Asserts `baseScore >= 7.0` (load-bearing)
- **L136:** Asserts `baseSeverity == "HIGH"` (load-bearing)
- **L145-180:** For every `device_cves[i]`, asserts HTTP 200 response with
  `baseScore >= 7.0` (load-bearing; exercises the DTU clone route with
  scenario-injected data)

The BC-2.06.019 pivot tests intentionally isolate the PrismQL query path (the
thing BC-2.06.019 contracts) with a deterministic hand-built registry. This is
correct test design: the PIVOT-003 story tests the query-layer; the DTU-layer
invariant (INV-NVD-CVE-CORRELATION-001: `new_with_scenario` provides correct
CVE data to the query engine) is owned by BC-2.06.020, which was delivered and
merged in predecessor story PIVOT-002 (PR #195).

Pass 3's factual premise was wrong. No coverage gap exists.

**DO-NOT-REFLAG identifier:** `PIVOT-003-PRLEVEL-OBS-2`

## Conclusion

Round 1 disposition: **0/3 strict streak** (two OBS finding classes surfaced,
both adjudicated as verified non-defects). PR remains CLEAN(PR-merge) for all
3 passes. Both DO-NOT-REFLAG entries registered in SESSION-HANDOFF §DO-NOT-REFLAG
and STATE.md D-1257.

**Next:** PR-LEVEL round 2 — 3 independent fresh-context adversary passes on
frozen 192428db. Adversary dispatchers MUST apply PIVOT-003-PRLEVEL-OBS-1 and
PIVOT-003-PRLEVEL-OBS-2 DO-NOT-REFLAG before dispatching to avoid repeat cycles
on settled findings.
