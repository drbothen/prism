---
document_type: adversarial-review
pass_id: 23
window_position: "0/3"
disposition: BLOCKED
date: 2026-05-04
producer: adversary
cycle: wave-4-operations
phase: 4A
---

# Adversarial Review — Pass 23

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 2 |
| MEDIUM | 1 |
| LOW | 1 |
| INFO | 0 |
| **Total** | **4** |

**Verdict:** FINDINGS_REMAIN

**Window position:** 0/3 (unchanged — BLOCKED findings require remediation before window can advance)

**Disposition:** BLOCKED → REMEDIATED (architect burst applied; Pass 24 next)

---

## Findings

### F-P23-H-001 [HIGH — SUBSTANTIVE]

**Title:** operational-pipeline.md — 3 stale references missed by Pre-Pass-21 hand-curated sweep target list

**File:** `.factory/specs/architecture/operational-pipeline.md`

**Description:** The Pre-Pass-21 broad-sweep target list was hand-curated and did not include `operational-pipeline.md`. As a result, three stale architectural claims survived:
1. `16-permit` budget reference — should be `8-permit` per D-209 per-subsystem split
2. `Action Engine` label — should be `ActionDeliveryEngine` per the Pass-6 rename
3. `1-second tick` interval — should be `60s` per ADR-013 §2.1

These three claims contradict the canonical D-209 decision, the ADR-013 §2.1 tick, and the established `ActionDeliveryEngine` rename. All are substantive claim-vs-reality drift (not cosmetic).

**Claim-vs-reality:** operational-pipeline.md described an architecture that was superseded by D-209 (per-subsystem 8/8 semaphore split) and ADR-016 §1.1 (ActionDeliveryEngine naming).

**Resolution:** Architect bumped operational-pipeline.md v1.1 → v1.2. All 3 stale references corrected. ARCH-INDEX Document Map annotation updated to v1.2.

**Root cause (process gap):** Pre-Pass-21 sweep target list was hand-curated and omitted `operational-pipeline.md` despite it being an architecture foundation doc that references the operational subsystem constants. See TD-VSDD-048.

---

### F-P23-H-002 [HIGH — SUBSTANTIVE]

**Title:** actions.md Mermaid participant display labels still show "Action Engine" — claim-vs-reality drift in v1.1 changelog

**File:** `.factory/specs/architecture/actions.md`

**Description:** The v1.1 changelog (Pre-Pass-21 burst, F-PreP21-H-001) claimed the rename `ActionEngine→ActionDeliveryEngine` was applied to `actions.md`. However, the Mermaid diagram participant display labels in the file retained the stale `Action Engine` form. The frontmatter/prose rename was applied but the diagram `participant` declarations were missed.

**Claim-vs-reality:** ARCH-INDEX v2.18 changelog and the actions.md v1.1 changelog claimed full rename coverage; the Mermaid participant labels falsified that claim.

**Resolution:** Architect bumped actions.md v1.2 → v1.3. Two Mermaid participant display labels corrected to `ActionDeliveryEngine`. ARCH-INDEX Document Map annotation updated to v1.3.

---

### F-P23-M-001 [MEDIUM — SUBSTANTIVE]

**Title:** operational-pipeline.md changelog had no Wave 4 entries

**File:** `.factory/specs/architecture/operational-pipeline.md`

**Description:** The operational-pipeline.md changelog section contained no entries for Wave 4 changes (D-209 semaphore split, ADR-013 tick update, ActionDeliveryEngine rename). This is a changelog-completeness gap: the document was modified as part of the F-P23-H-001 fix but its internal changelog did not record the Wave 4 architectural decisions that drove those changes.

**Claim-vs-reality:** A reader consulting operational-pipeline.md changelog would see no indication of Wave 4 architectural changes, even though the doc reflects those changes.

**Resolution:** Architect added Wave 4 changelog entry as part of the v1.2 bump (F-P23-H-001 fix).

---

### F-P23-L-001 [LOW — process-gap]

**Title:** Broad-sweep target lists are hand-curated with no enforcement of exhaustiveness

**File:** Process (no single file)

**Description:** Each pre-pass broad-sweep dispatches against a hand-curated target list. The Pre-Pass-21 sweep list included `actions.md`, `module-decomposition.md`, `api-surface.md`, `data-layer.md`, and `verification-architecture.md` — but excluded `operational-pipeline.md`. There is no mechanical check that confirms all architecture docs containing a given architectural token (e.g., `16-permit`, `Action Engine`, `1-second tick`) were included in the sweep.

**Impact:** 3 stale references in `operational-pipeline.md` survived two pre-pass sweeps (Pre-Pass-21 and Pre-Pass-22) and were only caught in Pass 23.

**TD filed:** TD-VSDD-048 — Broad-sweep methodology must include exhaustive grep-completeness check.

---

## Cross-Cut Verification

12 cross-cut chains verified by adversary:

1. D-209 per-subsystem 8/8 split — confirmed in concurrency-architecture.md v1.1, BC-2.18.001/002/004 v1.4, actions.md v1.3, operational-pipeline.md v1.2
2. ADR-013 §2.1 60s tick — confirmed in ADR-013 v0.7, actions.md v1.3, S-4.01 v1.12, operational-pipeline.md v1.2
3. ActionDeliveryEngine rename — confirmed in interface-definitions.md v2.5, BC-2.18.003/008 v1.4, actions.md v1.3, operational-pipeline.md v1.2; vp-045 v1.2
4. ADR-016 §2.5 action_state CF key table — confirmed in actions.md v1.2 (5-row canonical form)
5. 17 column families — confirmed in data-layer.md v1.3, AD-004
6. D-209 8/8 split in data-layer.md — confirmed v1.3 per-subsystem concurrency claim
7. OrgId/ClientId CF key prefix — confirmed `{org_id}:` first-segment invariant across ADR-016, ADR-017, ADR-018
8. VP-045/047 priority P0 — confirmed VP-INDEX v1.26 per POL-9 sync
9. ADR-016 v0.12 — confirmed retry-state row \x04 + dead-letter row \x03 discriminators
10. ARCH-INDEX v2.22 annotations — confirmed operational-pipeline.md v1.2 + actions.md v1.3 rows
11. case_dedup_idx CF — confirmed in data-layer.md v1.3, AD-004 17 CF count
12. VP-137..VP-145 module assignments — confirmed verification-architecture v1.28 Mermaid P13 nodes

---

## Window Status

**Before remediation:** 0/3 (BLOCKED)
**After remediation:** 0/3 (window stays 0/3 — findings were BLOCKED class)
**Next pass:** Pass 24 (window 1/3 attempt)

---

## Pass Trajectory (cumulative)

`38→17→8→7→7→5→5→6→6→5→5→4→7→9→2→4→3→3(CLEAN)→18:CLEAN(1/3)→19:CLEAN(2/3)→20:BLOCKED(RESET 0/3)→PreSweep→21:BLOCKED→REMEDIATED(0/3)→PreP22Sweep(COMPLETE;0/3)→22:BLOCKED→REMEDIATED(1H+1M+1L;TD-VSDD-047)→23:BLOCKED→REMEDIATED(2H+1M+1L;sweep-target-list gap)`
