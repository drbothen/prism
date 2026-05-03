---
document_type: adversarial-review-pass
phase: 4.A
pass_number: 2
producer: adversary (verbatim findings reconstructed by state-manager)
timestamp: 2026-05-02T22:30:00Z
predecessor: pass-1.md (BLOCKED 38 findings; remediated 2026-05-02)
verdict: BLOCKED
findings_count: 17
severity_breakdown: { CRITICAL: 0, HIGH: 4, MEDIUM: 7, LOW: 4, OBS: 2 }
window_status: 0/3 (reset)
remediation_status: COMPLETED_2026-05-02 (architect ADR fixes + story-writer alignment)
remediation_commits: [618b453e]
---

# Adversarial Review — Wave 4 Phase 4.A Pass 2

**Verdict: BLOCKED** — 17 findings (0C / 4H / 7M / 4L / 2OBS). Convergence window reset to 0/3.
All 17 findings remediated 2026-05-02 via architect v0.2 → v0.3 ADR fixes (ADR-013/015/016/017/018) and
story-writer alignment (S-4.03/S-4.05/S-4.06/S-4.07/S-4.08).

---

## HIGH Findings (4)

### P2-ADR-016-A-H-001 — Idempotency Key / Dedup Key Contradiction

**File:** `specs/architecture/decisions/ADR-016-action-delivery-framework.md`
**Severity:** HIGH
**Finding:** ADR-016 v0.2 used the terms `idempotency_key` and `dedup_key` interchangeably across different
sections. §4 (ActionSpec struct) defined a `dedup_key: Option<String>` field, while §7 (delivery guarantees)
referred to `idempotency_key` as the at-least-once deduplication anchor. §9 (RocksDB layout) used
`dedup_key` again. This dual-naming creates a semantic split: is the field called `dedup_key` or
`idempotency_key`? Any inconsistency will produce compile errors when story-writer implements S-4.08.
**Required Fix:** Canonicalize to a single field name throughout ADR-016. Pick one name and apply it
consistently across ActionSpec, delivery guarantee invariants, and RocksDB CF key layout.
**Resolution:** ADR-016 v0.3 — field canonicalized to `idempotency_key: Option<IdempotencyKey>` throughout.
`dedup_key` references removed. Added newtype `IdempotencyKey(String)` definition in §4.

---

### P2-ADR-016-A-H-002 — Case Dedup: `event_seq` Undefined; `timeline_entry_id` Resolution

**File:** `specs/architecture/decisions/ADR-016-action-delivery-framework.md`
**Severity:** HIGH
**Finding:** ADR-016 v0.2 §6 referenced `event_seq` as the tie-breaking field for case deduplication
when multiple events arrive within the same dedup window. `event_seq` was not defined anywhere in
ADR-016, ADR-017, or the existing prism-core::case module. Story S-4.06 also referenced `event_seq`
in its timeline entry model without a definition. This is an undefined-identifier defect that will
block implementation of S-4.06 and S-4.08.
**Required Fix:** Either define `event_seq` in the appropriate ADR/spec, or replace with an existing
identified field. The tie-breaking field for temporal ordering of case timeline entries must be
explicitly typed and sourced.
**Resolution:** ADR-016 v0.3 (and ADR-017 v0.3) — `event_seq` replaced by `timeline_entry_id: TimelineEntryId`
(a UUIDv7-based newtype, already latent in prism-core::case). `TimelineEntryId` is monotonic via UUIDv7
timestamp component; §6 of ADR-016 updated to reference it as the dedup tie-breaker. ADR-017 §5
updated to define `TimelineEntryId(Uuid)` in the timeline entry struct.

---

### P2-S-4.03-A-H-001 — Duplicate YAML Key in S-4.03 Frontmatter

**File:** `stories/S-4.03-detection-rules.md`
**Severity:** HIGH
**Finding:** S-4.03 v1.7 frontmatter contained a duplicate YAML key. The `subsystems:` key appeared
twice — once listing `[SS-13]` (Detection Engine) and once listing `[SS-13, SS-21]`. In strict YAML
parsers, the second value silently overwrites the first; in lenient parsers, behavior is undefined.
Duplicate keys in frontmatter are a correctness defect regardless of the current value set, as they
introduce parse-time non-determinism.
**Required Fix:** Remove the duplicate `subsystems:` key; retain the canonical value `[SS-13, SS-21]`.

**Secondary issue (same file):** S-4.03 v1.7 listed `bcs: [BC-2.13.001, BC-2.13.001, BC-2.13.006, ...]`
with BC-2.13.001 duplicated in the frontmatter array. Remove the duplicate BC entry.
**Resolution:** S-4.03 v1.8 — duplicate `subsystems:` key merged; BC-2.13.001 duplicate removed from
frontmatter `bcs:` array.

---

### P2-S-4.06-A-H-001 + P2-S-4.08-A-H-001 — VP-138 Missing from Frontmatter; Subsystem Mismatch

**File A:** `stories/S-4.06-case-management.md` (P2-S-4.06-A-H-001)
**File B:** `stories/S-4.08-action-delivery.md` (P2-S-4.08-A-H-001)
**Severity:** HIGH (both)

**P2-S-4.06-A-H-001:** S-4.06 v1.11 body §3 Verification Properties listed VP-138 (ADR-013 schedule
CRUD invalidation rule — added in Phase 4.A Phase 1 burst) as a covered VP under Case Management
traceability. However, VP-138 was absent from the `vps:` frontmatter array. Frontmatter must be the
canonical source of truth for traceability. The body reference is correct; the frontmatter omission
is the defect.
**Required Fix:** Add `VP-138` to S-4.06 `vps:` frontmatter array.

**P2-S-4.08-A-H-001:** S-4.08 v1.12 listed `subsystems: [SS-18, SS-12]` in frontmatter. SS-12 is
the Scheduling subsystem (prism-core::schedule). S-4.08 (Action Delivery) implements the outbound
delivery runtime; it consumes schedule context but is not an implementation site of the scheduler.
The correct subsystem set is `[SS-18, SS-20, SS-21]` — Action Delivery (SS-18), SIEM Formats (SS-20),
and Identity & Core Types (SS-21, for OrgId/ClientId scoping). SS-12 should not appear in S-4.08
frontmatter.
**Required Fix:** Remove SS-12 from S-4.08 `subsystems:`; add SS-20 (SIEM Formats, per prism-siem-formats
dependency in S-4.08 body AC-10).
**Resolution:** S-4.06 v1.12 — VP-138 added to `vps:` frontmatter. S-4.08 v1.13 — subsystems corrected
to `[SS-18, SS-20, SS-21]`.

---

## MEDIUM Findings (7)

### P2-ADR-013-A-M-001 — Splay Edge Case Undefined for Large Counts

**File:** `specs/architecture/decisions/ADR-013-schedule-execution-semantics.md`
**Severity:** MEDIUM
**Finding:** ADR-013 v0.2 §5 defined a splay distribution formula for schedule execution start times
to avoid thundering-herd: `start_time = base_time + rand(0, splay_window_ms)`. The splay window was
defined as `min(interval_ms * 0.1, 30_000)` (10% of interval, capped at 30 seconds). However, no
behavior was specified for the case where `num_active_schedules` exceeds the splay window resolution:
if 1000 schedules all have a 1-second interval, the splay window is 100ms and multiple schedules
will be assigned identical start offsets with high probability. This results in de facto thundering
herd despite the splay mechanism.
**Required Fix:** Define a minimum per-schedule offset increment when schedule density exceeds splay
resolution: e.g., `min_increment = splay_window_ms / num_active_schedules` with a floor of 1ms.
Or document that splay is probabilistic and best-effort only (acceptable if density is bounded by
the 8-permit semaphore).
**Resolution:** ADR-013 v0.3 §5 — added clarification that splay is probabilistic/best-effort; the
8-permit semaphore (D-209) provides the hard concurrency bound that limits thundering-herd impact.
Density-exceeding-resolution case documented as non-defect under semaphore constraint.

---

### P2-ADR-016-A-M-001 — `Created` Notification Not Invalidated by Schedule Change

**File:** `specs/architecture/decisions/ADR-016-action-delivery-framework.md`
**Severity:** MEDIUM
**Finding:** ADR-016 v0.2 §8 specified that a schedule CRUD operation invalidates cached dedup-window
resolutions for affected rules, triggering a rule reload per D-211. However, ADR-016 did not address
in-flight `ActionSpec` records whose `Created` status was set before the schedule change. If a schedule
change reduces the dedup window, previously-created ActionSpecs may now fall outside the new window
and should be re-evaluated. The spec left this case unaddressed, creating a potential for duplicate
or missed action delivery during schedule-change windows.
**Required Fix:** Define the schedule-change invalidation scope: does it affect only future
`ActionSpec` creation, or also pending (not-yet-delivered) `ActionSpec` records?
**Resolution:** ADR-016 v0.3 §8 — schedule-change invalidation scope defined: only affects future
`RuleCondition` evaluations. In-flight `ActionSpec` records with `Created` or `Pending` status are
not retroactively invalidated; they deliver to completion. Rationale: at-least-once guarantee;
duplicates tolerated over missed delivery.

---

### P2-ADR-016-A-M-002 — Auth Order Ambiguity in Multi-Destination Actions

**File:** `specs/architecture/decisions/ADR-016-action-delivery-framework.md`
**Severity:** MEDIUM
**Finding:** ADR-016 v0.2 §6 described action delivery as a pipeline: resolve destination →
authenticate → deliver. For `ActionSpec` records with multiple destinations (e.g., both Slack
and PagerDuty), the spec did not define whether authentication is attempted in parallel or
sequentially, and whether a failed auth on one destination blocks or skips the others.
**Required Fix:** Specify parallel vs. sequential auth and partial-failure semantics for
multi-destination `ActionSpec` delivery.
**Resolution:** ADR-016 v0.3 §6 — multi-destination auth is attempted in parallel (independent per
destination). Partial failure: failed destinations are marked `Failed` on their individual
`DeliveryAttempt` records; successful destinations proceed to delivery. No destination blocks another.

---

### P2-ADR-018-A-M-001 — WIT Fields Missing from DiffResult Schema

**File:** `specs/architecture/decisions/ADR-018-differential-result-pack-format.md`
**Severity:** MEDIUM
**Finding:** ADR-018 v0.2 §4 defined the `DiffResult` struct without WIT (WebAssembly Interface
Types) field annotations. S-4.02 body referenced WIT interface fields for WASM component embedding
(`diff_result.wit`), but ADR-018 contained no corresponding WIT schema section. This creates a
traceability gap: story-writer cannot confirm that the WIT interface matches the Rust struct.
**Required Fix:** Add a §X WIT interface schema to ADR-018 that mirrors the `DiffResult` Rust struct,
or explicitly note that WIT embedding is out of scope for ADR-018 and is defined solely in the story.
**Resolution:** ADR-018 v0.3 — added §9 WIT interface note: WIT field annotations are story-level
implementation detail; ADR-018 governs the canonical Rust struct definition. Story S-4.02 WIT
interface must mirror ADR-018 §4 struct exactly; any divergence is a story defect, not an ADR defect.

---

### P2-ADR-018-A-M-002 — Pack Name Uniqueness Constraint Scope Ambiguous

**File:** `specs/architecture/decisions/ADR-018-differential-result-pack-format.md`
**Severity:** MEDIUM
**Finding:** ADR-018 v0.2 §5 required `ResultPack` names to be unique within an execution run.
The scope of "execution run" was not defined: does uniqueness apply per-OrgId, per-schedule-entry,
per-sensor-type, or globally within the process? If uniqueness is process-global, orgs cannot use
the same pack name for independent schedules, which would be a usability defect.
**Required Fix:** Define the uniqueness scope for `ResultPack` name: per `(OrgId, ScheduleId)` tuple
is the expected constraint.
**Resolution:** ADR-018 v0.3 §5 — pack name uniqueness scoped to `(OrgId, ScheduleId)` tuple.
Two different orgs may use the same pack name; two schedules within the same org may not.

---

### P2-ADR-013-A-M-002 — Cron Field Count: 5-Field vs 6-Field Ambiguity

**File:** `specs/architecture/decisions/ADR-013-schedule-execution-semantics.md`
**Severity:** MEDIUM
**Finding:** ADR-013 v0.2 §3 specified cron expression format as "standard 5-field cron
(minute, hour, day-of-month, month, day-of-week)" but §7 example schedules used 6-field expressions
including a seconds field: `0 */5 * * * *`. This inconsistency between the spec prose and examples
would cause parser failures at implementation.
**Required Fix:** Standardize on either 5-field or 6-field cron. The `croner` crate (R-2 from
research-findings.md) supports 6-field by default; adopt 6-field with seconds or explicitly strip
the seconds field at parse time.
**Resolution:** ADR-013 v0.3 §3 — canonicalized to 6-field cron (seconds, minute, hour,
day-of-month, month, day-of-week) using `croner` crate. All examples updated to 6-field form.

---

### P2-ADR-017-A-M-001 — Case Dedup Index Race During Concurrent Transitions

**File:** `specs/architecture/decisions/ADR-017-case-lifecycle-invariants.md`
**Severity:** MEDIUM
**Finding:** ADR-017 v0.2 §6 described a `case_dedup_idx` RocksDB column family used to prevent
duplicate case creation for the same alert within a dedup window. The spec did not define the
locking protocol for concurrent transitions: if two threads simultaneously detect the same alert
and attempt case creation, both may read `case_dedup_idx` before either writes, resulting in
duplicate case creation (toctou race).
**Required Fix:** Specify the atomic check-and-set protocol for `case_dedup_idx` — either a
RocksDB merge operator CAS pattern, or a process-level Mutex guard over the dedup check-and-create
pair.
**Resolution:** ADR-017 v0.3 §6 — `case_dedup_idx` writes guarded by a per-OrgId `Mutex<()>` held
for the check-and-insert span. Process-level (not distributed) guard is sufficient since prism runs
as a single-process MCP server per analyst. RocksDB merge operator alternative noted as future
optimization (TD-W4-CASE-DEDUP-LOCK-001).

---

## LOW Findings (4)

### P2-S-4.05-A-L-001 — §5/Annex Duplication in Alert Generation Story

**File:** `stories/S-4.05-alert-generation.md`
**Severity:** LOW
**Finding:** S-4.05 v1.7 §5 (Verification Properties) and the Annex (BC Traceability Matrix) both
listed the same VP-028 entry with identical narrative text. The §5 VP block serves as the
authoritative traceability record; the Annex VP entry is redundant and risks diverging in future
edits.
**Required Fix:** Remove the duplicate VP-028 entry from the Annex, retaining it only in §5.
**Resolution:** S-4.05 v1.8 — duplicate VP-028 Annex entry removed.

---

### P2-S-4.07-A-L-001 — `mttd_approx` Field Without Source Definition

**File:** `stories/S-4.07-case-metrics.md`
**Severity:** LOW
**Finding:** S-4.07 v1.7 AC-4 described a `mttd_approx` (mean time to detect, approximate) metric
field on the `CaseMetrics` struct. The field was not defined in ADR-017 or any BC. The "approx"
qualifier suggested an estimated value derived from timestamps, but the derivation formula was not
specified (first alert created_at minus rule first_evaluated_at? alert created_at minus detection
rule last_fired_at?).
**Required Fix:** Define the `mttd_approx` derivation formula in S-4.07 AC-4, or reference the ADR
that defines it.
**Resolution:** S-4.07 v1.8 — AC-4 updated: `mttd_approx = case.first_alert_created_at -
case.detection_rule_first_fired_at` (both timestamps required on `Case`). ADR-017 v0.3 §5
updated to include both timestamp fields on `Case` struct definition.

---

### P2-S-4.03-A-L-001 — BC-2.09.004 Anchor Stale After Pass 1

**File:** `stories/S-4.03-detection-rules.md`
**Severity:** LOW
**Finding:** S-4.03 v1.7 BC Traceability Matrix listed BC-2.09.004 with anchor text
"Detection rule hot-reload without service restart". The BC-INDEX v4.27 had BC-2.09.004 retitled to
"Rule change propagation to active evaluators" in a Wave 2 pass-1 remediation. The story's BC title
was stale by one version.
**Required Fix:** Update BC-2.09.004 anchor text in S-4.03 to match BC-INDEX current title.
**Resolution:** S-4.03 v1.8 — BC-2.09.004 anchor text updated to "Rule change propagation to active evaluators".

---

### P2-S-4.03-A-L-002 — `acknowledged_by` Comment Misleading

**File:** `stories/S-4.03-detection-rules.md`
**Severity:** LOW
**Finding:** S-4.03 v1.7 AC-7 included an inline comment `// acknowledged_by: story-writer 2026-05-02`
referencing a story-writer internal working note. This comment type is a process artifact not intended
for the canonical story document. It creates reader confusion about whether `acknowledged_by` is a
struct field or a document annotation.
**Required Fix:** Remove the inline process comment from AC-7.
**Resolution:** S-4.03 v1.8 — inline `// acknowledged_by` comment removed from AC-7.

---

## OBS Findings (2)

### OBS-P2-001 — Process Gap: VP Files for New ADRs Still Deferred

**Observation (not blocking):** VP-137 and VP-138 were added as stub files during Phase 4.A Phase 1
ADR burst (Pass 1 remediation). As of Pass 2, these VP files remain as minimal stubs without full
property specifications. The adversary notes that VP stubs are acceptable during the ADR convergence
phase (consistent with prior wave patterns), but confirms: these stubs MUST be fully populated before
the story-writer begins implementation of S-4.01 (VP-137) and S-4.06/S-4.08 (VP-138).
**Not blocking.** Flag for Phase 4.A convergence checklist.

---

### OBS-P2-002 — Process Gap: `test_depends_on` Schema Not Validated

**Observation (not blocking):** S-4.06 v1.11 and S-4.08 v1.12 both used a `test_depends_on:` field
in their frontmatter to express cross-story test dependencies. This field is not defined in the
VSDD story schema (STORY-TEMPLATE.md). It appears to have been introduced ad hoc by the story-writer.
The field is semantically useful for implementation sequencing, but its absence from the schema
means the consistency-validator cannot validate it.
**Not blocking.** Recommend: add `test_depends_on:` as an optional array field to the VSDD story
schema in a future vsdd-factory plugin maintenance cycle (TD-VSDD-038 candidate).

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 2 |
| **New findings** | 17 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 1.00 (17/17 — all findings new; Pass 1 findings fully remediated) |
| **Median severity** | 2.0 (HIGH=4, MEDIUM=7, LOW=4, OBS=2) |
| **Trajectory** | 38 → 17 (delta −21; severity regression: 11H→4H) |
| **Verdict** | FINDINGS_REMAIN |

---

## Summary Table

| ID | Severity | File | Finding | Resolution |
|----|----------|------|---------|------------|
| P2-ADR-016-A-H-001 | HIGH | ADR-016 | idempotency_key/dedup_key contradiction | Canonicalized to idempotency_key throughout |
| P2-ADR-016-A-H-002 | HIGH | ADR-016 | event_seq undefined → timeline_entry_id | Replaced with TimelineEntryId (UUIDv7) |
| P2-S-4.03-A-H-001 | HIGH | S-4.03 | duplicate YAML key + duplicate BC | S-4.03 v1.8 dedup removed |
| P2-S-4.06-A-H-001 | HIGH | S-4.06 | VP-138 missing frontmatter | S-4.06 v1.12 VP-138 added |
| P2-S-4.08-A-H-001 | HIGH | S-4.08 | subsystem mismatch (SS-12 wrong) | S-4.08 v1.13 subsystems corrected |
| P2-ADR-013-A-M-001 | MEDIUM | ADR-013 | splay edge case undefined | Best-effort documented; semaphore bounds impact |
| P2-ADR-016-A-M-001 | MEDIUM | ADR-016 | Created notification invalidation | In-flight ActionSpecs not retroactively invalidated |
| P2-ADR-016-A-M-002 | MEDIUM | ADR-016 | auth order multi-destination | Parallel auth; per-destination failure semantics |
| P2-ADR-018-A-M-001 | MEDIUM | ADR-018 | WIT fields gap | WIT is story-level; ADR-018 governs Rust struct |
| P2-ADR-018-A-M-002 | MEDIUM | ADR-018 | pack name uniqueness scope | Scoped to (OrgId, ScheduleId) |
| P2-ADR-013-A-M-002 | MEDIUM | ADR-013 | cron 5 vs 6 field | Canonicalized to 6-field (croner crate) |
| P2-ADR-017-A-M-001 | MEDIUM | ADR-017 | case_dedup_idx race | Per-OrgId Mutex CAS guard; TD filed |
| P2-S-4.05-A-L-001 | LOW | S-4.05 | §5/Annex VP-028 duplication | Annex duplicate removed |
| P2-S-4.07-A-L-001 | LOW | S-4.07 | mttd_approx undefined derivation | Formula defined in AC-4 |
| P2-S-4.03-A-L-001 | LOW | S-4.03 | BC-2.09.004 stale anchor | Title updated to BC-INDEX current |
| P2-S-4.03-A-L-002 | LOW | S-4.03 | acknowledged_by comment | Removed from AC-7 |
| OBS-P2-001 | OBS | VP stubs | VP-137/138 still minimal stubs | Accepted; must populate before impl |
| OBS-P2-002 | OBS | S-4.06/4.08 | test_depends_on not in schema | TD-VSDD-038 candidate; not blocking |
