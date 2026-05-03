---
document_type: adversarial-review-pass
phase: 4.A
pass_number: 5
producer: adversary (verbatim findings reconstructed by state-manager)
timestamp: 2026-05-03T01:30:00Z
predecessor: pass-4.md (BLOCKED 7 findings; remediated 2026-05-03)
verdict: BLOCKED
findings_count: 7
severity_breakdown: { CRITICAL: 0, HIGH: 4, MEDIUM: 2, LOW: 0, OBS: 1 }
window_status: 0/3 (reset)
remediation_status: COMPLETED_2026-05-03
remediation_commits: [<Stage 1 SHA>]
---

# Adversarial Review — Wave 4 Phase 4.A Pass 5

**Verdict: BLOCKED** — 7 findings (0C / 4H / 2M / 0L / 1OBS)

**Trajectory:** 38→17→8→7→7 (descent flattened; HIGH issues are partial-fix-regression class)

**Predecessor:** Pass 4 BLOCKED (7 findings); remediated 2026-05-03 before this pass.

---

## HIGH Findings

### P5-VARCH-A-H-001 — Stale aggregate counts in verification-architecture.md

**Severity:** HIGH
**File:** `.factory/specs/architecture/verification-architecture.md`
**Finding:** The SAFE (provable properties) aggregate and Tier 2 aggregate counts were stale. The document showed SAFE=138 and Tier 2=79 after Wave 4 VPs were registered (VP-137..VP-145); the correct post-registration values are SAFE=145 and Tier 2=86. The P31/P32 node counts in the verification graph were also not updated to reflect the 9 new Wave 4 VPs.
**Resolution:** verification-architecture.md updated — SAFE 138→145, Tier 2 79→86, P31/P32 node counts corrected. Committed in Stage 1 of this burst.

---

### P5-VCOV-A-H-002 — Coverage matrix totals row shows 144 instead of 145

**Severity:** HIGH
**File:** `.factory/specs/architecture/verification-coverage-matrix.md`
**Finding:** The Summary/Totals row in the verification coverage matrix showed 144 total VPs. This was stale by 1 after VP-145 was added in the Pass 1 remediation burst (b881b0d2 / S-4.06 Task 9b auto-case-dedup). The Proptest sub-total also showed 85 instead of 86.
**Resolution:** verification-coverage-matrix.md totals row corrected — Total 144→145, Proptest 85→86. Committed in Stage 1 of this burst.

---

### P5-S408-A-H-003 — S-4.08 missing VP-144 in frontmatter vps_assigned

**Severity:** HIGH
**File:** `.factory/stories/S-4.08-action-delivery.md`
**Finding:** VP-144 (CEF v0 + LEEF 2.0 encoder correctness — 13 proptest invariants) was added to VP-INDEX with anchor `S-4.08` in the Phase 3 ADR burst (e4315c91), but S-4.08 frontmatter `vps_assigned:` list did not include VP-144. The story claims ownership of the verification property but the frontmatter is not authoritative.
**Resolution:** S-4.08 frontmatter updated — VP-144 added to `vps_assigned`. Story version bumped v1.13→v1.14. Committed in pre-existing work for this burst.

---

### P5-S408-A-H-004 — S-4.08 missing VP-137 in frontmatter vps_assigned

**Severity:** HIGH
**File:** `.factory/stories/S-4.08-action-delivery.md`
**Finding:** VP-137 (Schedule executor liveness: per-subsystem semaphore non-starvation) was added to VP-INDEX with anchor `S-4.01, S-4.08` in the Phase 1 ADR burst (6d6fbfb6), but S-4.08 frontmatter `vps_assigned:` list did not include VP-137. Co-ownership with S-4.01 is correct, but S-4.08 must carry the VP in its own frontmatter as a co-implementor.
**Resolution:** S-4.08 frontmatter updated — VP-137 added to `vps_assigned`. Covered in the same v1.13→v1.14 bump as P5-S408-A-H-003.

---

## MEDIUM Findings

### P5-S407-A-M-005 — VP-145 anchor missing S-4.07 co-implementor

**Severity:** MEDIUM
**File:** `.factory/specs/verification-properties/VP-INDEX.md`
**Finding:** VP-145 (Case reopen_count monotonic increment, INV-CASE-006) showed anchor `S-4.06` only. S-4.07 (Case Metrics and Acknowledge Alert) exercises the same reopen_count invariant via the acknowledge-alert path, which increments reopen_count when a Resolved case is re-opened through alert acknowledgment. S-4.07 is a co-implementor and the anchor column should be dual: `S-4.06, S-4.07`.
**Resolution:** VP-INDEX VP-145 anchor updated to `S-4.06, S-4.07`. VP-INDEX version bumped v1.24→v1.25. Committed in Stage 1 of this burst.

---

### P5-XADR-A-M-006 — AD-004 column family list missing case_dedup_idx

**Severity:** MEDIUM
**File:** `.factory/specs/architecture/ARCH-INDEX.md`
**Finding:** AD-004 (RocksDB with 17 column families) was amended in the Pass 1 remediation to record 17 CFs and list all CF names including `case_dedup_idx`. However, the ARCH-INDEX changelog did not contain a row for this amendment, making the audit trail incomplete. The AD-004 body content (the CF list in the Architecture Decisions table) was correct as of the prior burst, but the version number and changelog record were missing.
**Resolution:** ARCH-INDEX version bumped v2.4→v2.5. Changelog row added: "AD-004 amended — 16→17 column families; added case_dedup_idx (per S-4.06 Task 9b auto-case-dedup secondary index)." Committed in Stage 1 of this burst.

---

## OBSERVATION

### P5-VARCH-A-OBS-001 — Tier 1 Wave 4 VPs are Kani-eligible candidates

**Severity:** OBS (informational)
**File:** `.factory/specs/architecture/verification-architecture.md`
**Finding:** Several of the newly-registered Wave 4 VPs (VP-137 semaphore non-starvation, VP-141 epoch counter merge_operator atomicity, VP-142 pack expansion idempotence) are logically pure enough to be Kani proof candidates under the existing Kani Arbitrary Policy (ADR-004). The verification-architecture.md Tier classification placed these as Tier 2 (proptest), which is correct given the current Wave 4 scope. However, a future adversarial pass or Wave 5 hardening cycle should revisit whether VP-137/141/142 warrant promotion to Tier 1 Kani proofs, as their invariants are finite-state and bounded.
**Resolution:** No action required in this burst. Noted for Wave 5 formal hardening consideration.

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 5 |
| **New findings** | 6 |
| **Duplicate/variant findings** | 1 |
| **Novelty score** | 0.86 (6/7) |
| **Median severity** | 3.0 (HIGH) |
| **Trajectory** | 38→17→8→7→7 |
| **Verdict** | FINDINGS_REMAIN |

---

## Remediation Summary

| Finding | Severity | File | Resolution | Commit |
|---------|----------|------|-----------|--------|
| P5-VARCH-A-H-001 | HIGH | verification-architecture.md | SAFE 138→145, Tier 2 79→86, P31/P32 updated | Stage 1 |
| P5-VCOV-A-H-002 | HIGH | verification-coverage-matrix.md | Totals 144→145, Proptest 85→86 | Stage 1 |
| P5-S408-A-H-003 | HIGH | S-4.08-action-delivery.md | VP-144 added to vps_assigned; v1.14 | Stage 1 |
| P5-S408-A-H-004 | HIGH | S-4.08-action-delivery.md | VP-137 added to vps_assigned; v1.14 | Stage 1 |
| P5-S407-A-M-005 | MEDIUM | VP-INDEX.md | VP-145 anchor S-4.06 → S-4.06, S-4.07 | Stage 1 |
| P5-XADR-A-M-006 | MEDIUM | ARCH-INDEX.md | v2.5; AD-004 changelog row added | Stage 1 |
| P5-VARCH-A-OBS-001 | OBS | verification-architecture.md | No action — Wave 5 candidate noted | N/A |

All 6 actionable findings REMEDIATED. Pass 6 queued.
