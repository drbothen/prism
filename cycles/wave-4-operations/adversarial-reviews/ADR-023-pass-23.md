---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-10T23:55:00Z
phase: 5
inputs:
  - .factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md
input-hash: "7d38067"
traces_to: prd.md
pass: 23
target_sha: f243867f
previous_review: ADR-023-pass-22.md
target_version: v1.17
findings_total: 5
findings_by_severity:
  critical: 0
  high: 2
  medium: 2
  low: 1
  obs: 0
residuals: 0
new: 5
streak: "0/3 unchanged"
trajectory: "26→16→12→14→3→3→1→0→0→4→2→0→1→1→4→3→2→2→0→0→3→4→5"
verifications: 22
verdict: NOT_CLEAN
---

# Adversarial Review: ADR-023 Plugin-Only Sensor Architecture (Pass 23)

## Finding ID Convention

Finding IDs use the format: `F-PASS23-<SEV>-<SEQ>`

- `F-PASS23`: Pass 23 prefix
- `<SEV>`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`)
- `<SEQ>`: Three-digit sequence within the pass

## Part A — Fix Verification (pass-22 residuals)

All pass-22 findings verified closed or carry-forward accounted for in v1.17.

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| F-PASS22-CRIT-001 | CRITICAL | CARRY-FORWARD (TD-VSDD-055/056) | 3rd hook-bypass recurrence cannot be closed by doc-fix; structural enforcement tracked in TD-VSDD-055 (validate-write-tool-only hook) + TD-VSDD-056 (maintenance-burst dispatch type). Not a v1.17 residual. |
| F-PASS22-HIGH-001 | HIGH | CLOSED | Process-Gap section updated in v1.16→v1.17 to acknowledge 3rd recurrence. |
| F-PASS22-HIGH-002 | HIGH | CLOSED | v1.16 changelog entry corrected to honestly document the sed bypass path. |
| F-PASS22-MED-001 | MEDIUM | CLOSED | ARCH-INDEX Decision Records row title synced; frontmatter + H1 + ARCH-INDEX tagline aligned. |

Residuals: 0. Content findings closed. CRIT-001 tracked structurally via TDs.

## Part B — New Findings (pass 23)

All 5 findings below are caused by compaction and sibling-site sweep gaps introduced during fix-burst-17's STATE.md compaction or the ARCH-INDEX version-sync procedure.

---

### F-PASS23-HIGH-001 — ARCH-INDEX Decision Records Row: ADR-023 Version Stamp Lag (v1.16 vs v1.17)

**Severity:** HIGH

**Location:** `.factory/specs/architecture/ARCH-INDEX.md` line 91

**Evidence:**

The ADR Registry table row for ADR-023 reads:

```
| ADR-023 | Plugin-Only Sensor Architecture ... | COMMITTED v1.16 | 2026-05-10 | decisions/ADR-023-plugin-only-sensor-architecture.md |
```

The fix-burst-17 state-manager dispatch bumped ADR-023 from v1.16 to v1.17 (D-365), but the ARCH-INDEX sibling-site was not updated. ARCH-INDEX v2.38 changelog (the fix-burst-16 entry) documents `COMMITTED v1.16` as the entry timestamp; there is no v2.39 changelog row recording the v1.16→v1.17 bump.

**Source of truth:** ADR-023 frontmatter at target SHA reads `version: "1.17"`. ARCH-INDEX at the same HEAD reads `COMMITTED v1.16`. This is a sibling-site version-stamp propagation gap — the 12th documented S-7.01 recurrence.

**Impact:** ARCH-INDEX is the canonical navigation index. Readers inspecting ARCH-INDEX will believe ADR-023 is at v1.16 when it is at v1.17. Any automated sweep checking ARCH-INDEX version stamps will report stale data.

**Proposed Fix:** Update ARCH-INDEX L91 to `COMMITTED v1.17`. Add ARCH-INDEX changelog row v2.39 documenting the sync. Bump ARCH-INDEX frontmatter `version: "2.39"`.

---

### F-PASS23-HIGH-002 — SESSION-HANDOFF Body Staleness + STATE.md Contradiction

**Severity:** HIGH

**Location 1:** `.factory/SESSION-HANDOFF.md` body STEP 1, line 10

**Location 2:** `.factory/SESSION-HANDOFF.md` KEY REFERENCES section, lines 19–34

**Location 3:** `.factory/STATE.md` Session Resume Checkpoint, line 284

**Evidence (Location 1 — STEP 1 stale):**

SESSION-HANDOFF STEP 1 reads:

> "STEP 1 (START HERE — PASS-21 NOT_CLEAN + FIX-BURST-16 COMPLETE): ... Dispatch adversary for pass-22..."

This narrative dates from before pass-22 and fix-burst-17. The current state is: pass-22 NOT_CLEAN_BYPASS (D-364), fix-burst-17 complete (D-365/D-366/D-367, ADR-023 v1.17), pass-23 NOT_CLEAN (this review). STEP 1 should reflect the current reality: pass-23 NOT_CLEAN, fix-burst-18 in progress, pass-24 is next target.

**Evidence (Location 2 — KEY REFERENCES stale):**

The KEY REFERENCES section body (SESSION-HANDOFF lines 19–34) still describes `ADR-023 at v1.13 streak 0/3 (assertion-check sweep applied)` and closes with `NEXT: pass-17`. This block was never updated through the fix-burst-17/pass-22/pass-23 cascade. The section cites D-356/355/354 (passes 17/16) as the most recent events, when the current state is D-367 (pass-22 NOT_CLEAN_BYPASS) and this pass-23 outcome.

**Evidence (Location 3 — STATE.md contradiction):**

STATE.md Session Resume Checkpoint reads:

> `STATE v7.106 SESSION-HANDOFF v7.105 (pending update)`

But SESSION-HANDOFF frontmatter already shows `version: "7.106"`. The claim that SESSION-HANDOFF is at v7.105 is factually incorrect — the SESSION-HANDOFF was bumped to v7.106 but STATE.md still cites v7.105.

**Impact:** A session starting fresh from SESSION-HANDOFF will be directed to dispatch pass-22 (already done and NOT_CLEAN_BYPASS). The KEY REFERENCES section is so stale it cannot be trusted for current version anchors. The STATE/SESSION-HANDOFF version contradiction will confuse any script or agent that checks version alignment.

**Proposed Fix:** Refresh SESSION-HANDOFF STEP 1 to reflect: pass-22 NOT_CLEAN_BYPASS COMPLETE, fix-burst-17 COMPLETE (ADR-023 v1.17 + TD-VSDD-055/056), pass-23 NOT_CLEAN (this review), fix-burst-18 in progress. Update KEY REFERENCES to current versions. Correct STATE.md L284 SESSION-HANDOFF version cite to v7.107 (after the refresh bump).

---

### F-PASS23-MED-001 — STATE.md Archive Note False Claim (D-200..D-325)

**Severity:** MEDIUM

**Location:** `.factory/STATE.md` Decisions Log preamble, line 209

**Evidence:**

STATE.md Decisions Log reads:

> `_D-200..D-325 archived: cycles/wave-4-operations/burst-log.md (v7.106 compaction — W4 gate decisions, W3 impl cascade, PR review passes)._`

This claim was introduced during the v7.106 compaction burst (fix-burst-17). Inspection of the burst-log reveals that only D-200..D-213 are present in the burst-log body. D-214..D-325 were NOT written to burst-log; they are carried in the `predecessor_session:` inline field of SESSION-HANDOFF. The archive note is therefore factually incorrect — it claims 126 decisions were archived to burst-log when the actual burst-log contains only 14 (D-200..D-213).

**Why this matters:** An agent or human reading the Decisions Log preamble will believe the burst-log is the recovery path for D-214..D-325. It is not. If the SESSION-HANDOFF predecessor_session blob were ever truncated or overwritten, D-214..D-325 would be unrecoverable from the burst-log. The false claim conceals this audit-trail risk.

**Proposed Fix:** Correct the archive note to reflect the truthful claim: "D-200..D-213 archived to burst-log.md (Burst 1); D-214..D-325 retained in inline `predecessor_session` field of SESSION-HANDOFF v7.107. Future compaction work: extend burst-log to capture D-214..D-325 with full text."

---

### F-PASS23-MED-002 — D-331 Lost from SESSION-HANDOFF predecessor_session Blob (Audit-Trail Integrity)

**Severity:** MEDIUM

**Location:** `.factory/SESSION-HANDOFF.md` frontmatter `predecessor_session:` field

**Evidence:**

The predecessor_session blob contains D-330 and then jumps directly to D-332, with D-331 entirely absent. D-331 is the merge event for PR #141 (S-3.02-FOLLOWUP-RUNTIME merged → develop@c6dd6602). This is documented in STATE.md `wave_3_implementation_status` as the decision record that established the current develop HEAD.

D-331 was dropped during the predecessor_session blob compaction in fix-burst-17. This is an audit-trail integrity defect: the authoritative source for the current develop HEAD (c6dd6602) is D-331, and D-331 is missing from the decision log.

**Material concern:** The develop HEAD `c6dd6602` appears throughout the spec corpus (STATE.md, SESSION-HANDOFF body, ARCH-INDEX references). D-331 is the decision row that traces this SHA to its merge event (PR #141). Without D-331, the audit trail cannot explain why develop is at c6dd6602 without referencing the STATE.md `wave_3_implementation_status` prose narrative. This violates the integrity principle that the Decisions Log should be self-sufficient for audit purposes.

**Proposed Fix:** Restore D-331 to the predecessor_session blob at the correct ordinal position (between D-330 and D-332). Content: "D-331: PR #141 S-3.02-FOLLOWUP-RUNTIME merged → develop@c6dd6602 — authoritative source for current develop HEAD across spec citations."

---

### F-PASS23-LOW-001 — STATE.md last_updated Does Not Reflect Pass-23 Dispatch

**Severity:** LOW

**Location:** `.factory/STATE.md` Project Metadata table, "Last Updated" row

**Evidence:**

The "Last Updated" field reads:

> `2026-05-10 (D-367 — ADR-023 pass-22 NOT_CLEAN_BYPASS 4 findings (1C+2H+1M), fix-burst-17 closes 3; TD-VSDD-055/056 filed; STATE v7.105→v7.106)`

Pass-23 has now been dispatched and completed (this review). The last_updated field has not been updated to reflect pass-23 or the fix-burst-18 burst that will close these findings.

**Impact:** LOW — the field is audit-trail metadata, not operationally critical. However, it is the first-glance currency indicator for STATE.md and should reflect the most recent completed event.

**Proposed Fix:** Update last_updated to reflect the fix-burst-18 completion (D-368/D-369 rows) and the STATE v7.106→v7.107 bump that accompanies this burst.

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 2 |
| MEDIUM | 2 |
| LOW | 1 |

**Overall Assessment:** block
**Convergence:** findings remain — iterate
**Readiness:** requires revision (fix-burst-18 needed before pass-24)

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 23 |
| **New findings** | 5 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 5/5 = 1.0 |
| **Median severity** | HIGH (3.0 on 1.0–5.0 scale) |
| **Trajectory** | 26→16→12→14→3→3→1→0→0→4→2→0→1→1→4→3→2→2→0→0→3→4→5 |
| **Verdict** | FINDINGS_REMAIN |

Note: All 5 findings are a new defect class — compaction-induced audit-trail gaps in STATE.md / SESSION-HANDOFF / ARCH-INDEX. ADR-023 v1.17 content itself (all 22 verifications) is CLEAN. The novelty score of 1.0 reflects a fresh defect category introduced by fix-burst-17's compaction procedure, not by regressions in the ADR body. Future compactions MUST preserve D-row full text in burst-log rather than relying solely on the predecessor_session inline blob.
