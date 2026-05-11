---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-10T00:00:00Z
phase: 5
inputs:
  - .factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md
input-hash: "7d38067"
traces_to: prd.md
pass: 24
target_sha: 87e80736
previous_review: ADR-023-pass-23.md
target_version: v1.17
findings_total: 3
findings_by_severity:
  critical: 0
  high: 1
  medium: 0
  low: 2
  obs: 0
residuals: 0
new: 3
streak: "0/3 unchanged"
trajectory: "26→16→12→14→3→3→1→0→0→4→2→0→1→1→4→3→2→2→0→0→3→4→5→3"
verifications: 4
verdict: NOT_CLEAN
---

# Adversarial Review: ADR-023 Plugin-Only Sensor Architecture (Pass 24)

## Finding ID Convention

Finding IDs use the format: `F-PASS24-<SEV>-<SEQ>`

- `F-PASS24`: Pass 24 prefix
- `<SEV>`: Severity abbreviation (`HIGH`, `LOW`)
- `<SEQ>`: Three-digit sequence within the pass

## Part A — Fix Verification (pass-23 residuals)

All pass-23 findings were closed by fix-burst-18 per D-369/D-370. No residuals carry forward.

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| F-PASS23-HIGH-001 | HIGH | CLOSED | ARCH-INDEX v2.38→v2.39; ADR-023 row version synced v1.16→v1.17 (D-370). |
| F-PASS23-HIGH-002 | HIGH | CLOSED | SESSION-HANDOFF body refreshed; STATE.md L284 corrected (D-369). |
| F-PASS23-MED-001 | MEDIUM | CLOSED | Archive note rewritten per fix-burst-18 — pointed at predecessor_session D-321..D-344 text (D-369). |
| F-PASS23-MED-002 | MEDIUM | CLOSED | D-331 restored to predecessor_session field (D-369). |
| F-PASS23-LOW-001 | LOW | CLOSED | last_updated currency corrected (D-369 — this row). |

Residuals: 0.

## Part B — New Findings (pass 24)

Pass-24 cross-checks STATE.md corpus claims against their stated sources. The ADR-023 v1.17
substantive content continues to verify CLEAN. All 3 findings below are state-corpus integrity
defects — a pattern now in its 13th recurrence (S-7.01 sibling-site drift class). The most
significant finding (F-PASS24-HIGH-001) reveals that fix-burst-18's archive note "fix" was
itself factually false, compounding the defect lineage from fix-burst-17.

---

### HIGH

#### F-PASS24-HIGH-001 — Archive Note False Claim: D-214..D-320 LOST, Not "Retained in predecessor_session"

- **Severity:** HIGH
- **Category:** state-corpus integrity / audit-trail
- **Recurrence class:** S-7.01 sibling-site gap (13th instance in this convergence cycle)
- **Location:** STATE.md line 219, Decisions Log archive note

**Evidence:**

STATE.md L219 archive note (post-fix-burst-18) reads:

> "D-214..D-325 retained in inline `predecessor_session` field of SESSION-HANDOFF v7.107"

Direct inspection of SESSION-HANDOFF v7.107 `predecessor_session` field reveals the following
D-rows are present: D-321, D-322, D-323, D-324, D-325, D-326, D-327, D-328, D-329, D-330,
D-331, D-332, D-333, D-334, D-335, D-336, D-337, D-338, D-339, D-340, D-341, D-342, D-343,
D-344.

The earliest decision entry in predecessor_session is **D-321**. The archive note claims
D-214 through D-325 are retained there. D-214 through D-320 are **absent** from
predecessor_session. They are also absent from:
- `cycles/wave-4-operations/burst-log.md` (Burst 1 covers D-200..D-213 only)
- `cycles/wave-3-multi-tenant/decisions-archive-d047-d114.md`
- `cycles/phase-3-dtu-wave-2/decisions-archive-d001-d032.md`
- Any other reachable cycle artifact

Fix-burst-18 (D-369) rewrote the archive note — the prior note was acknowledged as false —
but the replacement text repeated the false claim in a different form. The note cites
"D-214..D-325 retained in inline `predecessor_session`" where predecessor_session actually
starts at D-321. D-214 through D-320 (107 decisions spanning the Wave-4 Phase-4A adversary
cascade and Bundle-B initiation period) are **LOST from the live state corpus**.

The root cause is fix-burst-17 STATE.md compaction (502→297 lines), which discarded inline
D-rows without prior archival to burst-log. Fix-burst-18's corrective note was authored
without verifying the predecessor_session starting entry, replicating the false claim.

This is the 13th instance of the S-7.01 sibling-site recurrence pattern: a fix targeting a
false claim introduces a replacement claim that is also false, because the fix-burst did not
verify the target-of-claim against the cited source-of-truth before authoring the replacement.

- **Proposed Fix:** Rewrite STATE.md L219 archive note to truthfully disclose the loss:
  D-214..D-320 are LOST; predecessor_session starts at D-321. Provide recovery path (git
  history retrieval from factory-artifacts SHA prior to fix-burst-17). File TD-VSDD-057 for
  STATE.md compaction protocol (STATE.md compaction must preserve D-row content before discard).

---

### LOW

#### F-PASS24-LOW-001 — STATE.md vp_count: 145 Stale vs VP-INDEX Total 152

- **Severity:** LOW
- **Category:** state-corpus drift / pending-intent
- **Location:** STATE.md frontmatter, `vp_count: 145` (line 147)

**Evidence:**

STATE.md frontmatter: `vp_count: 145`

VP-INDEX.md Total row (v1.29): `| **Total** | **152** | **120** | **32** |`

The `vp_count` field has not been updated since before VP-146 through VP-152 were registered.
Seven VPs are unaccounted for in the state frontmatter. This may reflect a deliberate baseline
of "production VPs excluding plugin-alias VPs" (VP-146..VP-152 are PLUGIN-alias VPs registered
in Wave 4 Phase 4.A), in which case the field annotation should clarify scope. As written, the
claim is stale relative to VP-INDEX.

- **Proposed Fix:** Bump `vp_count` to 152 to match VP-INDEX total, OR add inline comment
  clarifying the lower count is "non-alias VPs only (pre-Wave-4 baseline)".

---

#### F-PASS24-LOW-002 — STATE.md current_step Cites D-301; Latest Decision is D-370

- **Severity:** LOW
- **Category:** state-corpus drift / pending-intent
- **Location:** STATE.md frontmatter, `current_step:` field (line 25)

**Evidence:**

STATE.md frontmatter `current_step:` reads:

> "D-301: Workspace 8-dimensional audit COMPLETE. 14 P0/P1 deferrals discovered..."

The latest recorded decision is D-370 (ARCH-INDEX v2.39 sync). D-301 is 69 decisions behind
the current corpus head. A session resuming from STATE.md will read D-301 as the live current
action, which is significantly stale.

- **Proposed Fix:** Refresh `current_step` to reflect the current convergence cycle phase,
  e.g., `"ADR-023 plugin-only sensor architecture convergence cycle (pass-24, fix-burst-19;
  trajectory 26→16→12→14→3→3→1→0→0→4→2→0→1→1→4→3→2→2→0→0→3→4→5→3; streak 0/3)"`.

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 1 |
| MEDIUM | 0 |
| LOW | 2 |

**Overall Assessment:** block
**Convergence:** findings remain — iterate
**Readiness:** requires fix-burst-19 before pass-25

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 24 |
| **New findings** | 3 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 3/3 = 1.0 |
| **Median severity** | LOW (2 LOW, 1 HIGH) |
| **Trajectory** | 26→16→12→14→3→3→1→0→0→4→2→0→1→1→4→3→2→2→0→0→3→4→5→3 |
| **Verdict** | FINDINGS_REMAIN |

Pass-24 ADR-023 v1.17 substantive content verified CLEAN across 4 spot-checks (plugin loading
sequence, PREREQ ordering, Rule 5 phrasing, Wave 1/A scope). All 3 findings are state-corpus
defects, not ADR body defects. The ADR body has been substantively CLEAN since pass-19 (5 passes
ago). The 13th S-7.01 recurrence (fix-burst-18's fix was itself false) indicates that
remediation-induced defects in archive-note prose persist as the dominant convergence
impediment. TD-VSDD-057 P0 warranted for STATE.md compaction protocol gap.
