---
document_type: adversarial-review-pass
level: L5
pass: 22
story: S-DEMO-DTU-LIVE-SCENARIO-001-B
pr: 185
head: 0863184a
streak_before: 2/3
streak_after: 0/3
clean_strict: false
clean_pr_merge: true
date: 2026-06-13
---

# PR-LEVEL Adversarial Review — Pass 22 (Convergence Pass)
## S-DEMO-DTU-LIVE-SCENARIO-001-B | PR #185 | HEAD 0863184a

**Pass type:** Convergence pass (streak was 2/3; one CLEAN strict needed for 3/3)
**Code status:** UNCHANGED from pass 20/21 at 0863184a. Diff unchanged since pass 20. No CI push.

---

## Findings

### BPRL-P22-01 MED — BC-2.06.020 VP Anchors prose stale: claimed VP-020-A..H / "all 8 VPs" but VP table defines A..L / 12 VPs

**Severity:** MED (spec-internal count discrepancy; no behavioral impact; code and tests correct)
**Category:** SPEC-ONLY (no code change; no CI required)
**Affected artifact:** `.factory/specs/behavioral-contracts/BC-2.06.020-demo-server-enrichment-correlation.md` — §VP Anchors section (line 553)

**Finding:**

The VP Anchors prose (line 553) read:

> "VP-020-A through VP-020-H (above) — verified by integration/unit tests in S-DEMO-DTU-LIVE-SCENARIO-001 (original enrichment-correlation delivery) and S-DEMO-DTU-LIVE-SCENARIO-001-B (AC implementations for all 8 VPs)"

However, the VP table in the same document defines 12 properties: VP-020-A through VP-020-L. VP-020-I through VP-020-L were added by D-1117 (v1.3) when the Cyberint CVE↔NVD correlation mechanism was introduced (PC-8, PC-9, INV-CYBERINT-ALERT-CVE-CORRELATION-001). The VP table was correctly extended to A..L at v1.3. The prose summary at §VP Anchors was not updated.

**Root cause:** D-1117 extended the VP table (rows VP-020-I..L) and the BC frontmatter `verification_properties` array, but did not sweep the prose summary sentence that stated the count/range of the amended entity set. This is the same class of miss as P14 (RNG range literal) and P15 (gate count) — a summary-count propagation gap on a multi-entity amendment.

**All other coherence checks CLEAN:**
- VP table VP-020-A..L (12 entries): correct
- PC-1..9 (9 postconditions): correct
- INVs (7 invariants): correct
- TV-020-001..015 (15 test vectors): correct
- EC-020-001..015 (15 edge cases): correct
- BC frontmatter `verification_properties` array (VP-020-A..L): correct
- Code, tests, story B, PIVOT-003: all correct and unchanged
- All BPRL-P1..P21 do-not-reflag items confirmed closed

**CLEAN(strict):** no
**CLEAN(PR-merge):** yes
**Streak:** RESET 2/3 → 0/3

---

## Closure

**SPEC-ONLY. No code change. No CI push. Feature HEAD UNCHANGED at 0863184a.**

**PO fix: BC-2.06.020 v1.4 → v1.5**

VP Anchors prose corrected:
- `VP-020-A through VP-020-H` → `VP-020-A through VP-020-L`
- `all 8 VPs` → `all 12 VPs`

**Orchestrator-caught regression during PO fix sweep:**

During the PO's exhaustive same-document summary-count sweep, the sweep had also mis-changed line 543 (Architecture Anchors paragraph) from `CVE-9999-{:05}` to `CVE-9999-{:04}`. The orchestrator verified the correct values against:
- Code: `crates/prism-dtu-common/src/scenario/mod.rs:449` — `gen_device_cves` doc comment `"CVE-9999-{seq:05}"` (5-digit catalog generator)
- Code: `crates/prism-dtu-cyberint/src/generator.rs:389` — `CVE-9999-{:04}` (4-digit Cyberint baseline generator, non-pivotable per PC-9)
- TV-020-012: `"CVE-9999-00001"` etc. (catalog IDs are 5-digit)
- SEC-001 test: `gen_device_cves must emit CVE-9999-{{seq:05}} format`

**Ruling: two distinct generators with different digit widths by design.** The 5-digit format (`{:05}`) is correct for the `gen_device_cves` catalog (used by `ScenarioEntityCatalog.device_cves`, drawn by Cyberint in scenario mode per PC-8, pre-loaded into NVD in scenario mode per PC-3). The 4-digit format (`{:04}`) is correct for the Cyberint baseline generator (`generator.rs:389`, used in non-scenario mode per PC-9, intentionally non-pivotable). These two distinct generators serve different roles and having different digit widths is intentional — unifying them would be a regression.

The PO reverted the catalog-format mis-change. Line 543 now correctly states `CVE-9999-{:05}` (catalog); line 297 correctly states `CVE-9999-{:04}` (Cyberint baseline). The v1.5 changelog was corrected to remove the erroneous "Architecture-Anchors mis-fix" note and add the revert note.

**PO exhaustive same-document sweep confirmed all counts/ranges consistent in v1.5:**
- PC-1..9 (9 postconditions) ✓
- INVs: 7 invariants ✓
- TV-020-001..015 (15 test vectors) ✓
- EC-020-001..015 (15 edge cases) ✓
- VP-020-A..L (12 VPs) ✓ (now correct in prose summary)
- Catalog `{:05}` format: 5-digit ✓
- Cyberint baseline `{:04}` format: 4-digit ✓

**Story-writer: story B v2.13 → v2.14**
- BC-2.06.020 pin v1.4 → v1.5 (3 sites: frontmatter `behavioral_contracts` implicit + §Behavioral Contracts BC table row + §Token Budget row; plus spec self-reference in changelog)

**Story-writer: PIVOT-003 v1.6 → v1.7**
- BC-2.06.020 pin v1.4 → v1.5 (§Behavioral Contracts BC table row + §Token Budget BC-2.06.020 context row)

---

## Index Updates

- BC-INDEX v6.40 → v6.41: row 120 `draft — v1.4` → `draft — v1.5`; rows 119+120 anchor story pin `ready v2.13` → `ready v2.14`; changelog row added
- STORY-INDEX v2.366 → v2.367: story B row `ready v2.13` → `ready v2.14`; PIVOT-003 row `draft v1.6` → `draft v1.7`; changelog row added
- STATE.md v7.776 → v7.777

---

## Pass Summary

| Axis | Result |
|------|--------|
| VP Anchors prose count/range | FINDING BPRL-P22-01 (stale A..H/8 → correct A..L/12) |
| PC count (9) | PASS |
| INV count (7) | PASS |
| TV count (15) | PASS |
| EC count (15) | PASS |
| All prior BPRL-P1..P21 closures | CONFIRMED CLOSED |
| SAP-1 tracing catalog | PASS |
| Code (HEAD 0863184a) | UNCHANGED |
| Orchestrator-caught: catalog `{:05}` vs baseline `{:04}` | TWO DISTINCT GENERATORS — `{:05}` in gen_device_cves, `{:04}` in generator.rs:389 — intentional by design; revert applied |

**CLEAN(strict):** no (BPRL-P22-01 VP-Anchors prose stale count/range)
**CLEAN(PR-merge):** yes
**Streak:** RESET 2/3 → 0/3
**Feature HEAD:** UNCHANGED 0863184a
**NEXT:** PR-LEVEL pass 23 at 0863184a (diff unchanged — reuse /tmp/pr185-pass20.diff; no CI push)
