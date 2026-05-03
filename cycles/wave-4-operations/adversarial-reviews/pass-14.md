---
document_type: adversary-review
pass_id: 14
wave: wave-4
phase: 4.A
window_position: "0/3 (BLOCKED)"
producer: vsdd-factory:adversary
date: 2026-05-03
disposition: BLOCKED
---

# Wave 4 Phase 4.A — Adversary Pass 14

**Date:** 2026-05-03
**Disposition:** BLOCKED (2 HIGH + 4 MEDIUM + 3 LOW + 2 INFO)
**Window:** 0/3 (BLOCKED → REMEDIATED; Pass 15 required)

---

## Tally

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 2 |
| MEDIUM | 4 |
| LOW | 3 |
| INFO | 2 |
| **Total** | **11** |

---

## Findings

### F-P14-H-001 [content-defect] — Audit-event terminology mis-anchoring

**Severity:** HIGH
**File:** S-4.01, S-4.08 (Task 5 + EC-12-006)
**Class:** Cross-document audit-event terminology consistency

ADR-013 §2.4 defines the canonical audit event for semaphore-exhausted schedule skips as `ScheduleFireMissed { miss_reason: SemaphoreExhausted }`. S-4.01 Task 5 and EC-12-006 used the non-canonical term `ScheduleFireSkipped`, which conflicts with the ADR-013 §2.4 definition. The `Skipped` vs `Missed` distinction is load-bearing: `ScheduleFireMissed` is the deliberate VSDD-chosen signal name for externally-observable missed-fire events. Using `ScheduleFireSkipped` in story tasks and acceptance criteria would cause implementers to emit the wrong audit event token, producing a runtime divergence from the ADR.

**Resolution:** story-writer: `ScheduleFireSkipped` → `ScheduleFireMissed{miss_reason: SemaphoreExhausted}` in S-4.01 Task 5 and EC-12-006. S-4.01 bumped v1.10 → v1.12.

---

### F-P14-H-002 [content-defect] — BC-2.12.004 frontmatter future-dated

**Severity:** HIGH
**File:** BC-2.12.004 frontmatter `modified` field + v1.7 changelog rows
**Class:** Date integrity

BC-2.12.004 v1.7 frontmatter contained `modified: 2026-05-04` — a future date relative to the actual edit date of 2026-05-03. The v1.7 changelog row and the preceding v1.6 row also carried incorrect dates. A future-dated `modified` field violates document integrity: any date-order check, audit trail review, or session-resume chronology scan would flag this document as anomalous.

**Resolution:** product-owner: `2026-05-04` → `2026-05-03` in frontmatter `modified` field; v1.7 changelog row date corrected; v1.6 row also corrected. BC-2.12.004 bumped v1.7 → v1.8.

---

### F-P14-M-001 [content-defect] — DashMap key-resolution gap and enum tuple form

**Severity:** MEDIUM
**File:** ADR-013 §2.7 + 13 sister sites (ADR-015, ADR-018, S-4.01, S-4.02)
**Class:** Enum variant structural correctness + DashMap key-resolution semantics

ADR-013 §2.7 described `ScheduleChangeNotification` variants without the `(OrgId, ScheduleId)` tuple payload form. The DashMap entry_or_insert pattern for Deleted handler requires the full `(OrgId, ScheduleId)` key to locate and remove the correct per-schedule cache entry; the naked variant form left the key-resolution algorithm underspecified and vulnerable to "wrong-org eviction" silent correctness bugs.

Additionally, the tuple form for the three variants (`Created`, `Updated`, `Deleted`) was inconsistently expressed across 13 sites: ADR-015 (5 sites), ADR-018 (3 sites), S-4.01 (4 sites), S-4.02 (1 site).

**Resolution:** architect: enum tuple form `ScheduleChangeNotification::{Created,Updated,Deleted}(OrgId, ScheduleId)` established in ADR-013 §2.7 with DashMap key-resolution paragraph added. Full 13-site cascade: ADR-015 v0.4→v0.5 (5 sites), ADR-018 v0.4→v0.5 (3 sites), S-4.01 v1.10→v1.12 (4 sites), S-4.02 v1.9→v1.11 (1 site). ADR-013 bumped v0.6→v0.7.

**NOTE:** This finding triggered the largest cascade in Wave 4 Phase 4.A: 13 total sites across 4 documents. All sites fixed atomically in the same burst to prevent partial-fix regression recurrence.

---

### F-P14-M-002 [content-defect] — Updated variant producer ambiguity

**Severity:** MEDIUM
**File:** ADR-013 §2.7
**Class:** Producer attribution clarity

ADR-013 §2.7 did not clearly specify which layer is responsible for emitting `ScheduleChangeNotification::Updated(OrgId, ScheduleId)` events — the CRUD handler, the execution loop, or both. Without a canonical producer attribution, two independent implementers could double-emit or one could rely on the other, producing undefined behavior in cache invalidation sequences.

**Resolution:** architect: producer attribution paragraph added to ADR-013 §2.7 specifying the CRUD layer as the sole emitter of `Updated` notifications. ADR-013 v0.7 (combined with F-P14-M-001 resolution).

---

### F-P14-M-003 [content-defect] — pack_id tuple semantic redundancy

**Severity:** MEDIUM
**File:** S-4.02 Task 7 collision text
**Class:** Semantic clarity on composite key construction

S-4.02 Task 7 described `pack_id = (org_id, pack.name)` without making explicit that `org_id` in this composite key is the same `org_id` as the ambient schedule execution context. This left ambiguity: could `pack_id.org_id` differ from the executing org's `OrgId` (e.g., if a cross-org pack reference were introduced later)? The collision-detection text needed a clarifying sentence that the `org_id` component of `pack_id` MUST equal the executing `OrgId` at schedule-fire time.

**Resolution:** story-writer: `pack_id = (org_id, pack.name)` collision text clarified to state `org_id` is the same org_id as the schedule execution context. S-4.02 bumped v1.9→v1.11 (combined with F-P14-M-001 cascade).

---

### F-P14-M-004 [content-defect] — OCSF→CEF severity mapping prose inverse/ambiguous

**Severity:** MEDIUM
**File:** S-4.08 line 188 + Dev Notes
**Class:** Cross-document canonical table adherence

S-4.08 contained OCSF→CEF severity mapping prose that was either inverted or underspecified compared to the canonical ADR-019 §3 mapping table. The canonical table is: OCSF 0→CEF 0 (Unknown), OCSF 1→CEF 1 (Low), OCSF 2→CEF 3 (Medium), OCSF 3→CEF 7 (High), OCSF 4→CEF 9 (Very High), OCSF 5→CEF 10 (Critical). S-4.08's prose read in a direction suggesting the inverse (CEF→OCSF) or used different ordinal values.

**Resolution:** story-writer: full canonical ADR-019 §3 mapping table reproduced verbatim in S-4.08 Dev Notes; ambiguous prose removed. S-4.08 bumped v1.20→v1.21.

---

### F-P14-L-001 [content-defect] — Stale `detection_state` CF ref (adversary attribution error)

**Severity:** LOW
**File:** S-4.05 EC-007 (NOTE: adversary initially attributed to S-4.07 — INCORRECT; defect is in S-4.05)
**Class:** Stale column-family name reference

A stale `detection_state` column-family reference survived in S-4.05 EC-007. Per the Wave 4 Phase 4.A CF-name cleanup, the canonical name adopted in Pass 5 for the alert state CF is `action_state` (formerly `detection_state`). S-4.05 EC-007 had not been updated.

**ADVERSARY ATTRIBUTION ERROR DOCUMENTED:** The adversary initially attributed this finding to S-4.07. The defect was located in S-4.05 EC-007. S-4.07 was reviewed and confirmed UNCHANGED (v1.8 — no defect present). The attribution error is a process artifact; the actual defect was correctly remediated in S-4.05.

**Resolution:** story-writer: `detection_state` → `action_state` in S-4.05 EC-007. S-4.05 bumped v1.11→v1.12. S-4.07 UNCHANGED at v1.8.

---

### F-P14-L-002 [content-defect] — ADR-013 Status H2 line stale version

**Severity:** LOW
**File:** ADR-013 line 56 (Status H2 heading)
**Class:** Body-prose vs frontmatter version synchronization

ADR-013 §2 Status section H2 line still referenced `v0.5` while the frontmatter `version:` field had been bumped to `v0.6` in the Pass 13 remediation burst. This Status H2 line is the human-readable "version badge" that session-resume readers rely on to quickly identify the document's current version without parsing YAML frontmatter.

**Resolution:** architect: Status H2 `v0.5` → `v0.7` (synchronized to the v0.7 bump triggered by F-P14-M-001/M-002 in the same burst). ADR-013 v0.7.

---

### F-P14-INFO-001 — Pre-Pass-14 sweep methodology positive process artifact

**Severity:** INFO
**Class:** Process observation (positive)

The TD-VSDD-039 codified methodology (pre-pass sweep for CF-key prefix order + VP module-column cross-check) executed cleanly before Pass 14 dispatch. The F-PreP14-H-003 and F-PreP14-H-004 findings from that sweep were both remediated before this pass. The adversary notes this as a positive convergence signal: the sweep methodology is reducing the HIGH-severity finding count per pass. Continued pre-pass sweep execution is recommended before Pass 15 and all subsequent passes.

**Action required:** None. Informational.

---

### F-P14-INFO-002 — Convergence trajectory novelty assessment

**Severity:** INFO
**Class:** Process observation (convergence analysis)

Pass 14 trajectory: 38→17→8→7→7→5→5→6→6→5→5→4→7→11(P14). The P14 finding count (11 total: 2H+4M+3L+2I) is higher than Pass 13 (7) but the severity composition changed: the 2H findings (F-P14-H-001 audit-event terminology, F-P14-H-002 future-dated BC) are fresh defect classes not previously seen in Wave 4 Phase 4.A passes. This suggests the convergence process is surfacing a new defect class layer (audit-event-terminology consistency + date integrity) rather than exhibiting true regression. The 13-site enum tuple cascade (F-P14-M-001) is a large remediation but represents a single structural gap.

**Recommendation:** Extend pre-pass sweep to include audit-event-name cross-checking (ADR §X.Y declared event names vs story Task body emit-call names) to catch F-P14-H-001 class findings before Pass 15 dispatch. See TD-VSDD-041.

**Action required:** None (recommendation captured in TD-VSDD-041). Informational.

---

### F-P14-L-003 [speculative] — BC-2.13.013 sibling-file flag

**Severity:** LOW (speculative)
**File:** BC-2.13.013 (not confirmed; flagged for pre-Pass-15 sweep)
**Class:** Speculative sibling-file flag

The adversary notes that BC-2.13.013 (Alert Generation BC family) may share the same `detection_state` vs `action_state` CF naming drift that was found in S-4.05 EC-007 (F-P14-L-001). This is a speculative flag — the adversary did not confirm the defect presence in BC-2.13.013 during this pass. It is raised as a pre-Pass-15 sweep target.

**Action required:** Pre-Pass-15 sweep: grep BC-2.13.013 for `detection_state`; if found, remediate before Pass 15 dispatch.

---

## Process-gap Codification Recommendation

**TD-VSDD-039 sweep methodology** (CF-key + VP module) caught some items before Pass 14 but missed the audit-event terminology consistency check class (ADR §X.Y → story Task body audit event names). F-P14-H-001 (`ScheduleFireSkipped` vs `ScheduleFireMissed` in S-4.01) was the trigger.

**Recommendation:** Extend standard pre-pass sweep checklist to include audit-event-name cross-checking:
- For each ADR in scope: grep declared audit event names in §X.Y Event Taxonomy sections.
- For each story in scope: grep Task body emit calls and acceptance criteria audit event token literals.
- Flag any mismatch as candidate HIGH finding before adversary dispatch.

This check would have caught F-P14-H-001 in the pre-pass sweep, saving one HIGH finding in this pass. Codify as TD-VSDD-041 (filed in vsdd-plugin-tech-debt.md).

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 14 |
| **New findings** | 9 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 9/9 = 1.00 |
| **Median severity** | 2.0 (MEDIUM) |
| **Trajectory** | 38→17→8→7→7→5→5→6→6→5→5→4→7→9 |
| **Verdict** | FINDINGS_REMAIN |

---

## Resolution Burst Summary

| Finding | Severity | Site | Agent | Version Bump |
|---------|----------|------|-------|-------------|
| F-P14-H-001 | HIGH | S-4.01 Task 5 + EC-12-006 | story-writer | S-4.01 v1.10→v1.12 |
| F-P14-H-002 | HIGH | BC-2.12.004 frontmatter + changelog rows | product-owner | BC-2.12.004 v1.7→v1.8 |
| F-P14-M-001 (13-site cascade) | MEDIUM | ADR-013 §2.7 + ADR-015 (5) + ADR-018 (3) + S-4.01 (4) + S-4.02 (1) | architect | ADR-013 v0.6→v0.7, ADR-015 v0.4→v0.5, ADR-018 v0.4→v0.5, S-4.01 v1.10→v1.12, S-4.02 v1.9→v1.11 |
| F-P14-M-002 | MEDIUM | ADR-013 §2.7 | architect | ADR-013 v0.7 (combined) |
| F-P14-M-003 | MEDIUM | S-4.02 Task 7 | story-writer | S-4.02 v1.9→v1.11 (combined) |
| F-P14-M-004 | MEDIUM | S-4.08 line 188 + Dev Notes | story-writer | S-4.08 v1.20→v1.21 |
| F-P14-L-001 (attribution-corrected) | LOW | S-4.05 EC-007 (not S-4.07) | story-writer | S-4.05 v1.11→v1.12; S-4.07 UNCHANGED v1.8 |
| F-P14-L-002 | LOW | ADR-013 line 56 | architect | ADR-013 v0.7 (combined) |
| F-P14-INFO-001 | INFO | — | — | No action |
| F-P14-INFO-002 | INFO | — | — | TD-VSDD-041 filed |
| F-P14-L-003 | LOW (speculative) | BC-2.13.013 | sweep agent (pre-P15) | Pending pre-P15 sweep |

**Total remediated: 2 HIGH + 4 MEDIUM + 2 LOW (F-P14-L-001 attribution-corrected). F-P14-L-003 (speculative) deferred to pre-Pass-15 sweep.**

**13-site enum tuple cascade (F-P14-M-001) executed atomically to prevent partial-fix regression recurrence — pattern from Pass 5/8/9/10/11/12 treadmill.**

**Window status:** 0/3 (BLOCKED → REMEDIATED). Pass 15 required to resume window toward 3-clean convergence target.

**Index bumps:** STORY-INDEX v1.95→v1.96, ARCH-INDEX v2.11→v2.12, BC-INDEX v4.29→v4.30.
