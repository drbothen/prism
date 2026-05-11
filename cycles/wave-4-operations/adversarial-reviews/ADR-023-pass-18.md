---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-10T23:59:00Z
phase: 5
inputs:
  - .factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md
input-hash: "ecd802d"
traces_to: .factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md
pass: 18
previous_review: ADR-023-pass-17.md
---

# Adversarial Review: ADR-023 Plugin-Only Sensor Architecture (Pass 18)

## Finding ID Convention

Finding IDs use the format: `F-PASS18-<SEV>-<SEQ>`

Target document: `ADR-023-plugin-only-sensor-architecture.md` v1.14 (target_sha `ed2e5db8`).
Verdict: NOT_CLEAN — 2 findings (0C+1H+0M+1L+0O). Streak: 0/3 unchanged — 9th S-7.01 recurrence. Trajectory: `26→16→12→14→3→3→1→0→0→4→2→0→1→1→4→3→2→2`.

**Residuals from pass-17:** Both pass-17 findings verified closed.

- F-PASS17-CRIT-001 (TD-FACTORY-HOOK-BYPASS-001 second recurrence escalation): TD-FACTORY-HOOK-BYPASS-001 is now P0 in the TD register. Process-Gap Awareness section updated. Action items 5+6 added. RESOLVED.
- F-PASS17-HIGH-001 (8th S-7.01 sibling-site recurrence at L297-298 + L567): Both sites now read "the fully-qualified call sites listed in C5" / "the three fully-qualified call-site cleanups listed in C5". RESOLVED.

**New findings this pass:** F-PASS18-HIGH-001 is a 9th S-7.01 sibling-site recurrence — the Process-Gap Awareness section at L1050 still cites TD-FACTORY-HOOK-BYPASS-001 as P1 after fix-burst-14 escalated it to P0. F-PASS18-LOW-001 is a lexical match on "lib.rs re-exports" at L957-958 requiring intent verification (different crate scope from C5/PREREQ-E's `prism-spec-engine/src/lib.rs`).

Number of source-of-truth verifications performed: 11

---

## Part A — Fix Verification

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| F-PASS17-CRIT-001 | CRITICAL | RESOLVED | TD-FACTORY-HOOK-BYPASS-001 escalated to P0 in TD register (action items 5+6 added). Process-Gap Awareness section updated in v1.14. |
| F-PASS17-HIGH-001 | HIGH | RESOLVED | L297-298 (Rule 5) now reads "the fully-qualified call sites listed in C5". L567 (C4) now reads "the three fully-qualified call-site cleanups listed in C5". Both sibling sites clean. |

---

## Part B — New Findings

### HIGH

#### F-PASS18-HIGH-001: Process-Gap Awareness L1050 cites TD-FACTORY-HOOK-BYPASS-001 as P1 after escalation to P0

- **Severity:** HIGH
- **Category:** spec-fidelity / S-7.01 sibling-site propagation gap / version-stamp drift
- **Location:** L1050 — Process-Gap Awareness section
- **Description:** The Process-Gap Awareness section states: "TD-FACTORY-HOOK-BYPASS-001 (P1) has been registered in the technical debt register". Fix-burst-14 (v1.14 amendment) escalated TD-FACTORY-HOOK-BYPASS-001 P1 → P0 and documented this in the TD register file and in fix-burst-14's changelog row. However the Process-Gap Awareness section body prose at L1050 was not updated to reflect the P0 escalation. This is the 9th recurrence of the S-7.01 sibling-site propagation pattern: a fix-burst updates the canonical site (TD register + changelog) but misses a reader-visible sibling site in the body prose.
- **Evidence:** L1050 reads: "TD-FACTORY-HOOK-BYPASS-001 (P1) has been registered". v1.14 changelog row reads: "F-PASS17-CRIT-001 (TD-FACTORY-HOOK-BYPASS-001 second recurrence) escalated to P0 with new action items 5+6 in TD register." Internal contradiction confirmed.
- **Proposed Fix:** Update L1050: change "(P1)" to "(P0, escalated 2026-05-10 on second recurrence per F-PASS17-CRIT-001)".

### LOW

#### F-PASS18-LOW-001: L957-958 "lib.rs re-exports" in Wave 1/A context — intent verification required

- **Severity:** LOW
- **Category:** spec-fidelity / lexical match / intent verification
- **Location:** L957-958 — Wave 1/A (LAST — CUTOVER) bullet
- **Description:** L957-958 reads: "Delete `prism-sensors/src/auth/{4 sensors}.rs` files, lib.rs re-exports, and `init_registry_for_org`." The phrase "lib.rs re-exports" is a lexical match on the sibling-site pattern from F-PASS16-LOW-002 / F-PASS15-MED-001 (fix-burst-12/13 fully qualified the `prism-spec-engine/src/lib.rs` path at C5+PREREQ-E). However, L957-958 operates in a different scope: Wave 1/A CUTOVER for `prism-sensors`, not PREREQ-E's `prism-spec-engine`. This may be an intentional use of "lib.rs re-exports" as shorthand for `prism-sensors/src/lib.rs` auth re-exports, distinct from the PREREQ-E path qualification. Intent verification required before determining if this is a defect or intentional distinction.
- **Evidence:** L957-958: "Delete `prism-sensors/src/auth/{4 sensors}.rs` files, lib.rs re-exports, and `init_registry_for_org`." Context: this is Wave 1/A CUTOVER, deleting sensor auth files from prism-sensors crate. The "lib.rs re-exports" here refers to `prism-sensors/src/lib.rs`, not the `prism-spec-engine/src/lib.rs` path that PREREQ-E qualifies.
- **Proposed Fix (pending intent verification):** If "lib.rs re-exports" here means prism-sensors/src/lib.rs auth re-exports (different crate from PREREQ-E), contextual disambiguation is sufficient — the surrounding context ("prism-sensors/src/auth/{4 sensors}.rs") makes the crate scope clear. If intent is to be maximally explicit: replace "lib.rs re-exports" with "`prism-sensors/src/lib.rs` auth re-exports". Recommend intent verification before applying fix.

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 1 |
| MEDIUM | 0 |
| LOW | 1 |

**Overall Assessment:** block
**Convergence:** findings remain — iterate
**Readiness:** requires revision — fix-burst-15 dispatched (close F-PASS18-HIGH-001 + defer F-PASS18-LOW-001 pending intent verification)

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 18 |
| **New findings** | 2 (F-PASS18-HIGH-001 + F-PASS18-LOW-001) |
| **Residuals from pass-17** | 0 |
| **Novelty score** | 1.0 (both genuinely new; no variants of prior findings) |
| **Median severity** | 1.5 (1 HIGH + 1 LOW) |
| **Trajectory** | `26→16→12→14→3→3→1→0→0→4→2→0→1→1→4→3→2→2` |
| **Streak** | 0/3 unchanged — 9th S-7.01 recurrence |
| **Verdict** | FINDINGS_REMAIN — 0/3 streak; fix-burst-15 closes F-PASS18-HIGH-001; F-PASS18-LOW-001 deferred as intentional; pass-19 targets streak 1/3 |
