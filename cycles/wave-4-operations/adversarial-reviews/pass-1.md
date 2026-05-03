---
document_type: adversarial-review-pass
phase: 4.A
pass_number: 1
producer: adversary (verbatim output reconstructed by state-manager)
timestamp: 2026-05-02T03:00:00Z
verdict: BLOCKED
findings_count: 38
severity_breakdown: { CRITICAL: 0, HIGH: 11, MEDIUM: 17, LOW: 7, OBS: 3 }
window_status: "0/3 (reset)"
remediation_status: "COMPLETED_2026-05-02 (architect ADR fixes + story-writer alignment)"
remediation_commits: ["<Stage 1 SHA>"]
---

# Adversarial Review — Wave 4 Phase 4.A Pass 1

> **Verdict: BLOCKED** — 11 HIGH findings across ADRs and stories. Convergence window reset to 0/3.
> Remediation completed 2026-05-02 by architect (6 ADR upgrades v0.1→v0.2) and story-writer (8 story
> alignment passes in 4 parallel batches). Pass 2 queued.

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 1 |
| **New findings** | 38 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 1.00 (38/38 — first pass, all findings new) |
| **Median severity** | 3.0 (HIGH=11, MEDIUM=17, LOW=7, OBS=3) |
| **Trajectory** | — → 38 (baseline) |
| **Verdict** | FINDINGS_REMAIN |

---

## Summary Table

| Category | Count | Files Affected |
|----------|-------|----------------|
| HIGH | 11 | ADR-013(1H), ADR-015(4H), ADR-016(4H), ADR-017(1H), ADR-018(2H), ADR-019(1H), S-4.03(1H), S-4.04(1H), S-4.05(1H), cross-cutting(3H) |
| MEDIUM | 17 | Multiple ADRs and stories |
| LOW | 7 | Various |
| OBS | 3 | Process gaps — tagged VSDD plugin tech debt |
| **Total** | **38** | |

---

## HIGH Findings

### ADR-013: Schedule Execution Semantics (1 HIGH)

**H-001 [ADR-013-H-001]** — Subsystem anchor mismatch: ADR-013 §3 assigned the scheduler to SS-12
(Scheduler) but the per-subsystem semaphore design also implicates SS-14 (Alert & Case Management)
and the action delivery semaphore. The single-subsystem anchor caused VP-137 to be scoped too narrowly.
ADR-013 §2 referenced a stale semaphore budget table not aligned with D-209. The semaphore budget
entries did not enumerate the SIEM-output subsystem introduced via ADR-019.

*Remediation:* ADR-013 v0.2 expanded §3 to enumerate all semaphore-bearing subsystems by SS-ID;
corrected VP-137 source reference; reconciled retry-and-dedup timing with ADR-015 §5.

---

### ADR-015: Detection Rule Language (4 HIGH)

**H-002 [ADR-015-H-001]** — UDF Volatility annotation wrong: §6 declared all detection UDFs as
`Volatility::Volatile` (re-evaluated every row). Correct per ADR-015 §6 revision and DataFusion
53.x ScalarUDFImpl semantics: IOC-match and severity-threshold UDFs are `Volatility::Immutable`
(same inputs always produce same output). Using Volatile forces a re-evaluation on every row even
when the IOC dictionary is unchanged, defeating the pre-compiled IOC plan optimization.

*Remediation:* ADR-015 v0.2 §6 updated UDF Volatility to `Immutable` for all three detection UDF
classes; added rationale citing DataFusion optimizer behavior.

**H-003 [ADR-015-H-002]** — Dedup window reference contradicted itself: §5 described dedup
invalidation as "on rule reload" but §3 anchor to D-211 said "at scheduling time" — two mutually
exclusive semantics. S-4.04 had inherited the contradictory phrasing.

*Remediation:* ADR-015 v0.2 §5 adopted scheduling-time resolution as canonical (per D-211);
§3 cross-reference tightened; VP-140 scope updated.

**H-004 [ADR-015-H-003]** — aho-corasick / RegexSet split not specified at implementation
boundary: ADR-015 §4 described the split conceptually but did not define the per-rule routing
predicate (which rules go to aho-corasick vs. RegexSet). S-4.03 had underspecified Task 6.

*Remediation:* ADR-015 v0.2 §4 added routing predicate definition: literal-substring-only
patterns → aho-corasick; patterns containing Regex meta-characters → RegexSet. VP-139 scope
tightened to cover equivalence proof.

**H-005 [ADR-015-H-004]** — Three-scope UNION merge model not specified: §7 described the
multi-rule result aggregation as "intersection" which was wrong — multiple detection rules
evaluating the same scan produce independent result sets that must be UNIONed (not intersected)
before alert generation. S-4.03 and S-4.04 AC language had inherited "intersection".

*Remediation:* ADR-015 v0.2 §7 replaced "intersection" with "UNION merge model"; propagated
to S-4.03 + S-4.04.

---

### ADR-016: Action Delivery Framework (4 HIGH)

**H-006 [ADR-016-H-001]** — Manual trigger fire-and-forget undefined: §8 described manual
trigger actions but did not specify the at-most-once vs. at-least-once delivery contract for
manual triggers (which are operator-initiated, not scheduler-driven). Without a specification,
implementers would default to the scheduled-trigger contract (retry-with-dead-letter), which is
wrong for manual triggers.

*Remediation:* ADR-016 v0.2 §8 added explicit fire-and-forget contract for manual triggers;
distinguished from scheduled at-least-once delivery; AC updated in S-4.08.

**H-007 [ADR-016-H-002]** — Subsystem anchor for action delivery semaphore missing SS-18:
§11 defined the per-subsystem semaphore budget but listed only SS-12 and SS-13. SS-18 (Action
Delivery Engine) was absent despite being the primary consumer of the semaphore-protected
delivery queue. VP-143 was anchored to the wrong subsystem.

*Remediation:* ADR-016 v0.2 §11 added SS-18 to the semaphore-bearing subsystems table; VP-143
source reference corrected.

**H-008 [ADR-016-H-003]** — Retry reconciliation gap: §9 defined 5-attempt retry with
exponential backoff but did not specify behavior when a partial retry batch is interrupted by
SIGTERM (in-flight retries in progress when process restarts). S-4.08 AC-7 had no guidance for
this case.

*Remediation:* ADR-016 v0.2 §9 added SIGTERM-interrupt clause: in-flight attempts are
abandoned; the action_state CF record preserves attempt_count and next_retry_at so the next
process restart resumes correctly. S-4.08 AC-7 updated.

**H-009 [ADR-016-H-004]** — Discriminator collision: two action subtypes (`ScheduledReport`
and `AlertNotification`) shared the `"type"` discriminator field value `"alert"` in the SERDE
representation defined in §3. Deserializing a `"type": "alert"` payload was ambiguous.

*Remediation:* ADR-016 v0.2 §3 renamed `AlertNotification` discriminator to `"alert_notification"`;
`ScheduledReport` discriminator to `"scheduled_report"`. S-4.05 + S-4.08 updated.

---

### ADR-017: Case Lifecycle Invariants (1 HIGH)

**H-010 [ADR-017-H-001]** — INV-CASE-006 (reopen_count) not invariant-specified: §3.3 described
the `reopen_count` field as incrementing on Resolved/Closed → Investigating transitions but did
not state the monotonicity invariant (reopen_count MUST only increment, never reset or decrement).
Without an invariant, an implementer could legitimately reset reopen_count on full case close.
VP-145 gap identified.

*Remediation:* ADR-017 v0.2 §3.3 added INV-CASE-006 formal invariant statement; VP-145 added
to VP-INDEX (proptest, P1, prism-operations, S-4.06 anchor).

---

### ADR-018: Differential Result Pack Format (2 HIGH)

**H-011 [ADR-018-H-001]** — RocksDB key prefix order violation: §2 defined the RocksDB key
schema for diff_results CF as `<schedule_id>/<epoch>/<record_id>` but this ordering places the
most-selective prefix (schedule_id) last when doing range scans. The correct ordering for
efficient range scans over an epoch window is `<schedule_id>/<epoch>/<record_id>` with the
understanding that the epoch field uses big-endian fixed-width encoding to preserve lexicographic
sort order. The spec was silent on big-endian encoding, and the examples showed decimal string
encoding which breaks lexicographic sort.

*Remediation:* ADR-018 v0.2 §2 added encoding requirement: epoch MUST be encoded as 8-byte
big-endian u64 (not decimal string); example key updated; VP-141 scope tightened.

**H-012 [ADR-018-H-002]** — Pack expansion idempotence not stated: §3 described registering a
ScheduleEntry but did not state that double-registration of the same schedule_id produces an
identical ScheduleEntry set (idempotence). S-4.02 AC had no idempotence acceptance criterion.

*Remediation:* ADR-018 v0.2 §3 added idempotence invariant; S-4.02 AC updated; VP-142 scope
confirmed.

---

### ADR-019: SIEM Output Formats (1 HIGH)

**H-013 [ADR-019-H-001]** — prism-siem-formats crate not registered in ARCH-INDEX SS-18: §9
defined a task to register the new crate in ARCH-INDEX but the ADR text did not cross-reference
SS-18's crate column as the target. The ARCH-INDEX SS-18 row still listed only `prism-operations`.

*Remediation:* ADR-019 v0.2 §9 updated task reference to cite ARCH-INDEX SS-18 specifically;
ARCH-INDEX SS-18 crate column updated to add `prism-siem-formats`.

---

### Story-Level HIGH Findings

**H-014 [S-4.03-H-001]** — S-4.03 AC-6 referenced "intersection of detection results" — inherited
from ADR-015 H-005 above. The UNION merge model correction was not yet propagated to S-4.03 v1.6.

*Remediation:* S-4.03 v1.7 AC-6 updated to UNION merge model.

**H-015 [S-4.04-H-001]** — S-4.04 AC-3 referenced "deduplicate at rule reload" — inherited from
ADR-015 H-003 dedup-window contradiction. S-4.04 v1.6 had the wrong dedup invalidation semantics.

*Remediation:* S-4.04 v1.7 AC-3 updated to scheduling-time dedup resolution.

**H-016 [S-4.05-H-001]** — S-4.05 CF discriminator value `"alert"` was ambiguous per ADR-016
H-009. S-4.05 v1.6 used the stale discriminator.

*Remediation:* S-4.05 v1.7 updated discriminator references to `"alert_notification"`.

---

### Cross-Cutting HIGH Findings

**H-CC-001 [CROSS-H-001]** — Subsystem mis-anchor: VP-137, VP-138, VP-143 all cited incorrect
or incomplete SS-IDs in their source_adr fields. Resolved by ADR-013 v0.2, ADR-016 v0.2, and
VP-INDEX update.

**H-CC-002 [CROSS-H-002]** — Discriminator collision: `"alert"` discriminator shared between
AlertNotification and ScheduledReport. Resolved by ADR-016 v0.2 §3 rename cascade propagated
to S-4.05 v1.7 and S-4.08 v1.12.

**H-CC-003 [CROSS-H-003]** — RocksDB key prefix sort-order violation: epoch key encoding
ambiguity in ADR-018 propagated to S-4.02 acceptance criteria. Resolved by ADR-018 v0.2 §2.

---

## MEDIUM Findings (17)

**M-001 [ADR-013-M-001]** — §4 schedule execution timing diagram was misaligned with §3
semaphore budget table (semaphore names didn't match). v0.2 corrected names.

**M-002 [ADR-013-M-002]** — Dedup timing reference in §5 cited D-209 but should cite D-211
(dedup is a separate decision from semaphore sizing). v0.2 corrected cross-reference.

**M-003 [ADR-015-M-001]** — §3 IOC match mode table was missing the `any` combinator row.
v0.2 added the row.

**M-004 [ADR-015-M-002]** — §5 dedup window default value (300s) not justified. v0.2 added
rationale: 300s = 5-minute polling interval × 1 cycle, matching the default schedule cadence.

**M-005 [ADR-015-M-003]** — S-4.03 Task 7 referenced "compile to DataFusion LogicalPlan" but
ADR-015 §6 says "compile to ScalarUDF registered in SessionContext". Task 7 updated.

**M-006 [ADR-015-M-004]** — VP-140 title used "invalidation" but ADR-015 §5 uses "expiry".
Aligned to "expiry" in VP-INDEX entry (deferred to pass-2 polish if low priority).
*Remediation:* S-4.04 and VP-140 title aligned to "dedup window expiry".

**M-007 [ADR-016-M-001]** — §5 credential reference scheme list omitted the `env:` prefix.
ADR-016 v0.2 §5 added `env:` to the four allowed prefixes.

**M-008 [ADR-016-M-002]** — §10 action plugin interface defined a `post_fire` hook but S-4.08
had no corresponding AC. S-4.08 v1.12 added AC-9 for `post_fire` hook invocation.

**M-009 [ADR-016-M-003]** — §12 OrgId scoping requirement was stated but not cross-referenced
to ADR-006 §3. v0.2 added cross-reference.

**M-010 [ADR-016-M-004]** — S-4.05 AC-2 used "fires action" which conflates scheduling with
execution. v1.7 clarified to "enqueues action for delivery".

**M-011 [ADR-017-M-001]** — §3.1 disposition enum used `Resolved` variant in both the status
field and the disposition field — naming collision. v0.2 renamed disposition variant to
`Dismissed` to avoid confusion. S-4.06 updated.

**M-012 [ADR-017-M-002]** — §4 INV-CASE-004 (no orphan alerts) referenced `alert_id FK` but
the alerts CF key schema in ADR-018 doesn't use FK naming. Cross-reference clarified.

**M-013 [ADR-018-M-001]** — §4 merge_operator description said "idempotent CAS" but RocksDB
merge operators are not CAS. v0.2 corrected to "monotonic increment merge operator". VP-141
description aligned.

**M-014 [ADR-018-M-002]** — §6 ScheduleEntry serialization format used JSON but bincode is the
project-wide standard per AD-012. v0.2 corrected to bincode.

**M-015 [ADR-019-M-001]** — §5 CEF header field order didn't match the CEF v0 standard (Version|
Device Vendor|Device Product|Device Version|Device Event Class ID|Name|Severity|Extension).
v0.2 corrected field order.

**M-016 [ADR-019-M-002]** — §6 LEEF 2.0 timestamp format used ISO-8601 but LEEF requires QRadar
epoch-milliseconds. v0.2 corrected format; INV-LEEF-004 added.

**M-017 [S-4.01-M-001]** — S-4.01 AC-5 described the semaphore budget as "per-thread" but
ADR-013 §3 uses "per-subsystem". AC-5 updated in v1.9.

---

## LOW Findings (7)

**L-001 [ADR-013-L-001]** — §6 references "Tokio semaphore" but should reference
`tokio::sync::Semaphore` by full path for implementation clarity. v0.2 added full path.

**L-002 [ADR-015-L-001]** — IOC match table column header "Match Mode" should be "Rule
Subtype" to align with BC-2.13.001 terminology.

**L-003 [ADR-015-L-002]** — §5 dedup window configuration field is `dedup_window_secs` but
S-4.04 AC used `dedup_window` (no unit suffix). Aligned to `dedup_window_secs`.

**L-004 [ADR-016-L-001]** — §7 dead-letter queue description used "DLQ" acronym without
defining it. v0.2 added definition.

**L-005 [ADR-017-L-001]** — §3.2 TTR formula used `t_resolved - t_opened` but should account
for reopen cycles per INV-CASE-005. S-4.07 metric AC clarified.

**L-006 [ADR-018-L-001]** — §5 epoch counter value type stated as `u32` but big-endian u64 is
required for >4B epochs. v0.2 corrected to u64.

**L-007 [S-4.06-L-001]** — S-4.06 AC-8 used "non-null" for disposition check but INV-CASE-002
uses "present". Aligned to "present" terminology.

---

## OBS Findings (3 — Process Gaps / VSDD Plugin Tech Debt)

**OBS-001 [PROC-OBS-001]** — `test_depends_on` schema field: multiple stories (S-4.03, S-4.06)
reference story-level test dependencies that are not captured in a structured `test_depends_on`
field. This is a VSDD story-template gap. Tagged TD-VSDD-038 (plugin tech debt, not Wave 4 scope).

**OBS-002 [PROC-OBS-002]** — ADR template subsystem cross-check hook absent: the ADR template
has no validation hook that verifies the subsystem IDs cited in §§ match the ARCH-INDEX Subsystem
Registry. This gap caused H-CC-001 (subsystem mis-anchor). Tagged TD-VSDD-039 (plugin tech debt).

**OBS-003 [PROC-OBS-003]** — discriminator-registry.md missing: there is no canonical file
enumerating all `#[serde(tag = "type")]` discriminator values in the codebase. The collision
identified in H-CC-002 would have been caught by such a registry. Tagged TD-VSDD-040 (plugin
tech debt).

---

## Policy Roll-Up

| Policy | Findings |
|--------|----------|
| POL-4 (Spec completeness — all contract terms formally defined) | 31 (H-001..H-016, H-CC-001..H-CC-003, M-001..M-017) |
| POL-6 (Cross-document consistency) | 4 (H-CC-001, H-CC-002, H-CC-003, M-006) |
| POL-8 (Traceability — VPs trace to BCs or ADRs) | 2 (H-010 VP-145 gap, H-CC-001 VP mis-anchor) |

---

## Remediation Summary

All findings addressed by 2026-05-02:

| Actor | Action | Result |
|-------|--------|--------|
| Architect | Upgraded ADR-013, ADR-015, ADR-016, ADR-017, ADR-018, ADR-019 from v0.1 → v0.2 | All 11H + 17M root-cause ADR fixes applied |
| Architect | Added INV-CASE-006 to ADR-017 v0.2 §3.3 | VP-145 gap resolved |
| Story-writer | Batch 1 (S-4.01, S-4.02): v1.8→v1.9 and v1.5→v1.6 | M-017, M-012 propagation fixes |
| Story-writer | Batch 2 (S-4.03, S-4.04): v1.6→v1.7 | H-014, H-015, M-003..M-006 propagation |
| Story-writer | Batch 3 (S-4.05, S-4.06): v1.6→v1.7 and v1.10→v1.11 | H-016, M-010, M-011, L-007 |
| Story-writer | Batch 4 (S-4.07, S-4.08): v1.6→v1.7 and v1.11→v1.12 | L-005, M-008, H-CC-002 discriminator propagation |
| State-manager | VP-145 added to VP-INDEX, ARCH-INDEX, verification-* updated | INV-CASE-006 traceability complete |
| State-manager | ARCH-INDEX SS-18 `prism-siem-formats` added | H-013 index gap closed |

**Discriminator collision RESOLVED:** `"alert"` discriminator split → `"alert_notification"` + `"scheduled_report"`.
**RocksDB key prefix order FIXED:** epoch big-endian u64 encoding mandated in ADR-018 v0.2.
**UNION merge model ADOPTED:** ADR-015 v0.2 + S-4.03/4.04 v1.7 updated.
**UDF Volatility CORRECTED:** `Volatile` → `Immutable` in ADR-015 v0.2 §6.
**VP-145 ADDED:** reopen_count monotonic increment (INV-CASE-006) — proptest P1 prism-operations.

---

## Trajectory

Pass 1: 38 findings (0C/11H/17M/7L/3OBS) — BLOCKED

Convergence window: 0/3 (reset). Pass 2 queued on remediated specs.
