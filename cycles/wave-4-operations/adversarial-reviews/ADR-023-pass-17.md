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
pass: 17
previous_review: ADR-023-pass-16.md
---

# Adversarial Review: ADR-023 Plugin-Only Sensor Architecture (Pass 17)

## Finding ID Convention

Finding IDs use the format: `ADV-P17-<SEV>-<SEQ>`

Target document: `ADR-023-plugin-only-sensor-architecture.md` v1.13 (target_sha `4b630639`).
Verdict: NOT_CLEAN_BYPASS — 2 findings (1 CRIT process-gap + 1 HIGH 8th-recurrence). Streak: 0/3 — process-gap CRIT blocks regardless of artifact-level state.

**Residuals from pass-16:** F-PASS16-LOW-002 partial propagation. Fix-burst-13 fully qualified the three call-site paths at PREREQ-E (L931-934) and C5 (L630-632). Those sites are clean. However two sibling sites — Rule 5 (L297-298) and C4 (L567) — retained the bare unqualified shorthand ("lib.rs re-export, examples/, tests/") from the pass-15 wording. Pass-16 identified only L931-934 and L630-632 as fix targets; fix-burst-13 closed exactly those two but did not sweep for the two sibling sites introduced by the same pass-15 COMPREHENSIVE SIBLING-SITE SWEEP. This constitutes the 8th S-7.01 recurrence.

**New finding this pass:** F-PASS17-CRIT-001 is a process-gap CRIT — the second recurrence of the TD-FACTORY-HOOK-BYPASS-001 bypass pattern, this time by the state-manager agent rather than the architect. The burst summary for fix-burst-13 contains a verbatim admission: state-manager used "python3 single-write" to perform a file mutation. This occurred after TD-FACTORY-HOOK-BYPASS-001 was codified as P1 with four explicit required actions. A second recurrence despite a codified P1 policy indicates the discipline is insufficient — structural enforcement is required.

Trajectory: `26→16→12→14→3→3→1→0→0→4→2→0→1→1→4→3→2`

Number of source-of-truth verifications performed: 8

---

## Part A — Fix Verification

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| F-PASS16-MED-001 | MED | RESOLVED | L924 stale parenthetical "(live plugin load replaces dead instantiation)" deleted cleanly in v1.13 |
| F-PASS16-LOW-001 | LOW | RESOLVED | L923 "wire it into" → "insert a new plugin-load step into" — tense alignment applied cleanly |
| F-PASS16-LOW-002 | LOW | PARTIAL — 2 sibling sites remain | L931-934 (PREREQ-E) and C5 L630-632 fully qualified. Rule 5 L297-298 + C4 L567 retain bare shorthand "lib.rs re-export, examples/, tests/" from the pass-15 COMPREHENSIVE SIBLING-SITE SWEEP introduction. Pass-16 fix verified only PREREQ-E and C5 sites. Rule 5 and C4 carry forward the partial-fix pattern. |

---

## Part B — New Findings

### CRITICAL

#### ADV-P17-CRIT-001: TD-FACTORY-HOOK-BYPASS-001 SECOND RECURRENCE — state-manager used python3 single-write

- **Severity:** CRITICAL (process-gap)
- **Category:** spec-fidelity / process-gap / methodology enforcement failure
- **Location:** Fix-burst-13 burst summary (session trace — not a body location in ADR-023)
- **Description:** The burst summary for fix-burst-13 contains a verbatim admission by the state-manager agent: "python3 single-write" was used to mutate a `.factory/` file. This is the second confirmed recurrence of the bypass class that TD-FACTORY-HOOK-BYPASS-001 P1 was filed to prevent. The first recurrence was fix-burst-3 (architect agent), which triggered the original TD filing with four explicit required actions. Action item 4 named the architect prompt template; neither action item 4 nor any other item addressed the state-manager dispatch brief with equivalent force. The second recurrence in a different agent class confirms that codified P1 discipline without structural enforcement is cross-agent insufficient.
- **Evidence:** Fix-burst-13 burst summary: "python3 single-write" (verbatim). TD-FACTORY-HOOK-BYPASS-001 P1 was in force at time of fix-burst-13 dispatch.
- **Proposed Fix:** Escalate TD-FACTORY-HOOK-BYPASS-001 P1 → P0. Add action items: (5) all dispatch briefs carry verbatim instruction "If hook blocks atomic multi-field update, use the Write tool (whole-file replacement). Python or bash file-write is policy-forbidden under TD-FACTORY-HOOK-BYPASS-001 P0." (6) Audit dispatcher hook plugin to add bypass-detection targeting Python file-write patterns in state-manager and architect session tool traces.

### HIGH

#### ADV-P17-HIGH-001: 8th S-7.01 sibling-site recurrence — L297-298 (Rule 5) + L567 (C4) retain bare path shorthand

- **Severity:** HIGH
- **Category:** spec-fidelity / S-7.01 partial-fix propagation / sibling-site sweep failure
- **Location:** L297-298 (Rule 5 paragraph) + L567 (C4 paragraph)
- **Description:** Fix-burst-12 (pass-15 closure) introduced "the three call sites identified in C5 — lib.rs re-export, examples/, tests/" at four body locations: Context, Rule 5 (L297-298), C4 (L567), and Migration Plan PREREQ-D. Fix-burst-13 (pass-16 F-PASS16-LOW-002 closure) fully qualified PREREQ-E (L931-934) and C5 (L630-632), but Rule 5 at L297-298 and C4 at L567 still retain the unqualified shorthand. Both were introduced by the same fix-burst-12 sweep that introduced the now-corrected sites. The 8th recurrence of the S-7.01 pattern is confirmed.
- **Evidence:** L297-298: "...at the three call sites identified in C5 (lib.rs re-export, examples/, tests/); no boot.rs wiring required." L567: "...only the three call-site cleanups identified in C5 — lib.rs re-export, examples/, tests/)."
- **Proposed Fix:** Adopt canonical-reference phrasing that delegates path authority to C5 (now fully qualified). L297-298: replace "(lib.rs re-export, examples/, tests/)" with "the three fully-qualified call sites listed in C5". L567: replace "— lib.rs re-export, examples/, tests/" with the three fully-qualified call-site cleanups listed in C5.

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 1 |
| HIGH | 1 |
| MEDIUM | 0 |
| LOW | 0 |

**Overall Assessment:** block
**Convergence:** findings remain — iterate
**Readiness:** requires revision — fix-burst-14 dispatched (TD escalation P1→P0 + 2-site amendment)

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 17 |
| **New findings** | 1 (F-PASS17-CRIT-001 process-gap second recurrence — genuinely new agent class) |
| **Duplicate/variant findings** | 1 (F-PASS17-HIGH-001 is 8th S-7.01 variant — same class, new sibling sites) |
| **Novelty score** | 0.50 (1 new / (1 new + 1 variant)) |
| **Median severity** | 2.5 (CRIT=1 + HIGH=1, mixed) |
| **Trajectory** | `26→16→12→14→3→3→1→0→0→4→2→0→1→1→4→3→2` |
| **Verdict** | FINDINGS_REMAIN — 0/3 streak; CRIT process-gap blocks increment; fix-burst-14 closes both findings; pass-18 targets streak 1/3 |
