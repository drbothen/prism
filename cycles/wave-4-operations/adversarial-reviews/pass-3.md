---
document_type: adversarial-review-pass
phase: 4.A
pass_number: 3
producer: adversary (verbatim findings reconstructed by state-manager)
timestamp: 2026-05-02T23:00:00Z
predecessor: pass-2.md (BLOCKED 17 findings; remediated)
verdict: BLOCKED
findings_count: 8
severity_breakdown: { CRITICAL: 0, HIGH: 3, MEDIUM: 4, LOW: 1, OBS: 0 }
window_status: 0/3 (reset)
remediation_status: COMPLETED_2026-05-02
remediation_commits: [15fa97e6]
---

# Adversarial Review — Phase 4.A Pass 3

**Verdict: BLOCKED** — 8 findings (0C / 3H / 4M / 1L / 0OBS). Convergence window reset to 0/3. Trajectory: 38 → 17 → **8** (continued descent).

---

## HIGH Findings (3)

### P3-ADR-018-A-H-001 — CF Key Prefix Order Violation

**Severity:** HIGH
**Artifact:** ADR-018-differential-result-pack-format.md
**Location:** §4 (RocksDB column family schema) / CF key prefix specification

**Finding:** The documented CF key prefix layout in ADR-018 §4 places `pack_id` before `sensor_id` in the composite key ordering for the `diff_packs` column family. This ordering conflicts with the access pattern declared in §6 (range-scan by sensor_id to collect all packs for a given sensor before computing the diff epoch). RocksDB prefix bloom filters are constructed over the leading key bytes; with `pack_id` first the bloom filter provides no benefit on the primary hot path (sensor-scoped range scan), and the `merge_operator` accumulation pattern described in §7 requires contiguous sensor-scoped key ranges. The correct prefix order is `[sensor_id][pack_id]`.

**Remediation required:** Correct §4 CF key prefix order to `[org_id][sensor_id][pack_id][epoch_seq]`. Update §7 merge_operator narrative to reference corrected key layout. Bump ADR-018 to v0.4.

---

### P3-XSTORY-A-H-002 — VP Frontmatter Omissions (5 VPs across 4 Stories)

**Severity:** HIGH
**Artifacts:** S-4.01, S-4.02, S-4.03, S-4.04
**Location:** Story frontmatter `verification_properties:` fields

**Finding:** Five verification properties are absent from story frontmatter despite being the implementation anchor for those stories per VP-INDEX. Specifically:

| VP | Expected Story | Frontmatter Status |
|----|---------------|-------------------|
| VP-137 | S-4.01 | Missing from `verification_properties:` list |
| VP-141 | S-4.02 | Missing from `verification_properties:` list |
| VP-142 | S-4.02 | Missing from `verification_properties:` list |
| VP-139 | S-4.03 | Missing from `verification_properties:` list |
| VP-140 | S-4.03, S-4.04 | Missing from both stories' `verification_properties:` lists |

Per VSDD Policy 8 (traceability completeness), every VP whose story anchor column in VP-INDEX names a story must appear in that story's frontmatter `verification_properties:` field. The absence of these five entries creates a bidirectional traceability break: VP-INDEX says the VP is anchored to the story; the story says it has no knowledge of the VP.

**Remediation required:** Add VP-137 to S-4.01 `verification_properties:`. Add VP-141, VP-142 to S-4.02 `verification_properties:`. Add VP-139, VP-140 to S-4.03 `verification_properties:`. Add VP-140 to S-4.04 `verification_properties:`. Bump all four stories to their next version.

---

### P3-ADR-019-A-H-003 — §10 vs §2.10 Section Mis-Anchor in ADR-016 Reference

**Severity:** HIGH
**Artifact:** ADR-019-siem-output-formats.md
**Location:** §3 (Context / Rationale) cross-reference to ADR-016

**Finding:** ADR-019 §3 cites "ADR-016 §10 (action-delivery retry semantics)" as justification for the at-least-once delivery guarantee applied to SIEM output records. However, ADR-016 does not contain a §10 at v0.3; the retry semantics content resides in §2.10 (a subsection of §2 "Decision Detail"). The §10 anchor resolves to nothing in the rendered document (section does not exist at the top level), making the cross-reference unverifiable. A reader following the citation cannot locate the normative text being referenced.

**Remediation required:** Correct ADR-019 §3 cross-reference from "ADR-016 §10" to "ADR-016 §2.10". Verify that §2.10 is the correct subsection (retry/at-least-once delivery). Bump ADR-019 to v0.3.

---

## MEDIUM Findings (4)

### P3-ADR-016-A-M-001 — Manual-Trigger Dedup Contradiction

**Severity:** MEDIUM
**Artifact:** ADR-016-action-delivery-framework.md
**Location:** §2.6 (manual trigger semantics) vs §2.8 (deduplication window)

**Finding:** ADR-016 §2.6 states that manual trigger actions are dispatched "fire-and-forget with no dedup applied." ADR-016 §2.8 defines the dedup window as applying to "all action delivery events within a subsystem." These two statements are contradictory when a manual trigger falls within an active dedup window for the same (org_id, sensor_id, action_type) composite key. The v0.3 text does not resolve whether manual triggers bypass the dedup key lookup entirely (fire-and-forget as stated) or are evaluated against the window but always accepted (alternative interpretation). An implementer cannot determine the correct behavior from the current text.

**Remediation required:** Add a disambiguating note to §2.6 clarifying: manual triggers bypass the dedup window lookup entirely (not subject to §2.8); the dedup CF is not consulted. Alternatively, if the intent is that manual triggers also consume a dedup slot (preventing rapid re-fire), state that explicitly and reconcile §2.6 language. Bump ADR-016 to v0.4.

---

### P3-ADR-013-A-M-002 — Global Rule Detection_State Key Not Specified

**Severity:** MEDIUM
**Artifact:** ADR-013-schedule-execution-semantics.md
**Location:** §5 (RocksDB state layout) / global rule subsection

**Finding:** ADR-013 §5 defines per-schedule RocksDB keys for execution state (last_run_at, next_run_at, splay_seed, execution_count) but does not specify a key format for detection rule state that spans multiple schedules. Global detection rules (those triggered by any schedule match, not pinned to a single schedule_id) require a lookup key that is independent of schedule_id. The current §5 schema would require a full scan of all schedule keys to collect global rule hits — an O(n) scan on the hot path. The key format for global-rule detection_state is unspecified.

**Remediation required:** Add a subsection to §5 specifying the CF key prefix for global-rule detection_state: proposed `[org_id][GLOBAL][rule_id]` (where GLOBAL is a fixed discriminant byte distinguishing global from per-schedule keys). If global rules are out of scope for ADR-013, add an explicit out-of-scope callout citing the deferred ADR. Bump ADR-013 to v0.4.

---

### P3-ADR-013-A-M-003 — next_run_at Lag Annotation Missing

**Severity:** MEDIUM
**Artifact:** ADR-013-schedule-execution-semantics.md
**Location:** §3 (Scheduling invariants) / next_run_at computation

**Finding:** ADR-013 §3 specifies that `next_run_at` is computed as `prev_run_at + interval + splay_offset` and stored atomically before execution begins. The document does not annotate the behavior when executor lag causes the scheduled fire time to be in the past: specifically, whether `next_run_at` is computed from the wall-clock fire time (current_time + interval) or from the nominal scheduled time (nominal_next + interval). The "cron catch-up" vs "cron skip" distinction is load-bearing for correctness under sustained lag — catch-up causes rapid burst execution; skip causes gaps. The v0.3 text does not specify which behavior is required.

**Remediation required:** Add an explicit annotation to §3: when `current_time > nominal_next_run_at` (executor lag), `next_run_at` MUST be computed as `current_time + splay_offset` (skip semantics — no catch-up). Reference the "best-effort splay" decision (D-209) as the rationale. Bump ADR-013 to v0.4.

---

### P3-VPINDEX-A-M-004 — VP-138 Anchor Ambiguity (S-4.06 vs S-4.07)

**Severity:** MEDIUM
**Artifact:** VP-INDEX.md
**Location:** Line 159, VP-138 story anchor column

**Finding:** VP-138 (cross-org case access denied invariant: INV-CASE-003, Wave 4 case-management isolation) lists both "S-4.06, S-4.07" as the implementation anchor. However, per ADR-017 §3.5 (case boundary invariants) and §8 (implementation notes), the cross-org isolation enforcement is implemented at the case-fetch boundary in S-4.06 (CaseStore org-scoped reads). S-4.07 covers case metrics and alert acknowledgement — it consumes cases via S-4.06 read paths and does not implement org-isolation enforcement. The dual anchor creates ambiguity about which story owns the VP harness and which story a contributor should look to for the test implementation. ADR-017 §3.5 is unambiguous: isolation is a S-4.06 invariant.

**Remediation required:** Change VP-138 story anchor from "S-4.06, S-4.07" to "S-4.06" only. Bump VP-INDEX to its next version.

---

## LOW Findings (1)

### P3-ADR-016-A-L-001 — MockScheduleStore Harness Skeleton Only

**Severity:** LOW
**Artifact:** ADR-016-action-delivery-framework.md
**Location:** §9 (Test Harness Notes)

**Finding:** ADR-016 §9 references a `MockScheduleStore` test harness for verifying action delivery non-starvation (VP-143). The description is a skeleton: it names the struct and the test method but does not specify whether `MockScheduleStore` is expected to implement the full `ScheduleStore` trait or only a subset interface. Given that VP-143 requires a proptest harness exercising per-subsystem semaphore fairness, a partial interface may be insufficient. The missing specification is low risk (test harness is implementation detail) but creates ambiguity for the story-writer authoring S-4.08.

**Remediation required (optional / story-writer may resolve):** Add a one-sentence note to §9 specifying that `MockScheduleStore` implements the full `ScheduleStore` trait using an in-memory `HashMap<ScheduleId, ScheduleEntry>` backing. This removes the ambiguity at spec time rather than leaving it to story-writer interpretation.

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 3 |
| **New findings** | 8 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 1.00 (8/8 — all findings new; Pass 2 findings fully remediated) |
| **Median severity** | 1.5 (HIGH=3, MEDIUM=4, LOW=1, OBS=0) |
| **Trajectory** | 38 → 17 → 8 (delta −9; severity improving: 4H→3H) |
| **Verdict** | FINDINGS_REMAIN |

---

## Trajectory Note

| Pass | Findings | Delta |
|------|----------|-------|
| Pass 1 | 38 | — |
| Pass 2 | 17 | −21 |
| Pass 3 | 8 | −9 |

Continued descent. 0 CRITICAL in any pass. 3 HIGH in this pass (all remediable by targeted ADR edits + story VP frontmatter sweep). No new finding classes introduced; all findings are specification-consistency issues resolvable without architectural rethink.

**Next action:** Remediate all 8 findings (architect: ADR-013 §3/§5, ADR-016 §2.6, ADR-018 §4, ADR-019 §3; story-writer: S-4.01/4.02/4.03/4.04 VP frontmatter; state-manager: VP-INDEX line 159). Then dispatch Pass 4. Target: CLEAN to open convergence window 1/3.
