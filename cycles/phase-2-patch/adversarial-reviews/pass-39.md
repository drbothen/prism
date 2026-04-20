---
document_type: adversarial-review
level: ops
version: "1.0"
producer: adversary
timestamp: 2026-04-19T00:00:00
phase: 2
inputs: []
input-hash: "[live-state]"
traces_to: prd.md
cycle: phase-2-patch
pass: 39
previous_review: pass-38.md
status: findings-open
novelty: Burst 40 cleanup validation surfaced Policy 8 propagation gaps — DI-028/DI-029 BC v1.1 lifts did not propagate to implementing stories S-4.03/S-5.05; S-4.01 error-code drift; VP-030 mis-anchor; Arch Mapping residuals in S-5.06; missing changelog sections across 71 stories
findings_total: 8
findings_crit: 0
findings_high: 5
findings_med: 2
findings_low: 0
findings_observational: 1
previous_pass: 38
convergence_counter: 0
date: 2026-04-19
---

# Adversarial Review: Prism (Pass 39)

## Finding ID Convention

`P3P39-A-{SEV}-NNN` where SEV is CRIT / HIGH / MED / LOW / OBS.

## Part A — Methodology

### Dimensions Scanned (15)

1. Burst 40 propagation validation — DI-028/DI-029 BC v1.1 lifts propagated to all implementing stories
2. Semantic anchoring integrity (Policy 4) — BC-ID / error-code / tool-name alignment across story bodies
3. Changelog discipline (Policy 2) — version bump changelog sections in story files
4. VP source-BC anchoring (Policy 7) — VP frontmatter source_bc cites correct canonical BC
5. Policy 8 bidirectional AC-to-BC trace — acceptance-criteria ↔ BC-INDEX cross-reference integrity
6. Architecture Mapping correctness (Policy 6) — subsystem ownership, tool names match canonical sources
7. BC traceability matrix co-ownership — story entries correct in STORY-INDEX matrix
8. Error-code alignment — story ACs use correct error codes from error-taxonomy.md
9. Cap value alignment — story ACs use correct cap values from invariants.md
10. DI citation forward-coverage — implementing stories cite DIs whose BCs were amended
11. Burst regression check — Burst 40 changes did not introduce new drift
12. OBS carryover — prior-pass observational items still valid / superseded
13. Convergence trajectory — finding-count trend vs prior passes
14. Changelog completeness — STORY-INDEX and story files reflect Burst 40 version bumps
15. Canonical tool-name alignment — Architecture Mapping uses tool names from api-surface.md

### Corpus

- BC-INDEX v4.10
- STORY-INDEX v1.27
- invariants.md (DI-028:47, DI-029)
- interface-definitions.md v2.1
- api-surface.md v1.3
- error-taxonomy.md v1.2
- VP-INDEX v1.3
- verification-coverage-matrix.md
- ARCH-INDEX (subsystem ownership)
- policies.yaml v1.1
- S-4.01, S-4.03, S-5.05, S-5.06, S-5.10 (implementing stories)
- BC-2.12.001 v1.1, BC-2.13.006 v1.1, BC-2.06.005 v1.1
- vp-030-schedule-rule-caps.md

---

## Part B — New Findings

### P3P39-A-HIGH-001 — S-4.01 uses wrong error code (E-SCHED-001 instead of E-SCHED-008) and wrong cap value (100 vs 500)

**Location:** `/Users/jmagady/Dev/prism/.factory/stories/S-4.01-schedule-crud.md` lines 136, 149, 202, 231

**Sites:**
- Line 136 Task 8 VP-030 description: "create_schedule with count = MAX_SCHEDULES always returns E-SCHED-001"
- Line 149 AC-2: "Given 100 existing schedules (at cap)...Then E-SCHED-001"
- Line 202 VP-030 table: "create returns E-SCHED-001 at limit"
- Line 231 Library table: "E-SCHED-001" in prism-core

**Canonical:** BC-2.12.001 v1.1 + DI-028 (invariants.md:47) specify E-SCHED-008 as the capacity-limit error code and 500 (`max_schedules`) as the cap value. E-SCHED-001 is a distinct error code with different semantics.

**Policy violations:** Policy 4 (semantic anchoring integrity) + Policy 8 (BC propagation discipline)

**Required fix:** Replace E-SCHED-001 with E-SCHED-008 and 100 with 500 at all 4 sites.

---

### P3P39-A-HIGH-002 — S-4.03 missing BC-2.13.006 v1.1 DI-028 rule-cap propagation (no VP-030 frontmatter, no rule-cap AC/Task, no E-RULE-011)

**Location:** `/Users/jmagady/Dev/prism/.factory/stories/S-4.03-detection-rules.md`

**Description:** Line 20 has BC-2.13.006 in frontmatter; line 21 `verification_properties: [VP-018]` only — VP-030 absent. Body contains zero occurrences of `DI-028`, `max_rules`, `E-RULE-011`, `VP-030`, or `rule cap`. Burst 40 lifted BC-2.13.006 to v1.1 adding a rule-cap postcondition and E-RULE-011 enforcement — but this propagation did not reach the primary implementing story.

**Canonical:** BC-2.13.006 v1.1:46 + DI-028 + VP-030 + verification-coverage-matrix.md:78 all require S-4.03 to carry the rule-cap AC and enforcement task.

**Policy violations:** Policy 8 (BC array changes propagate to body and ACs)

**Required fix:** Add VP-030 to frontmatter `verification_properties`; add AC for rule-cap check at 1000 returning E-RULE-011; add Task for E-RULE-011 enforcement.

---

### P3P39-A-HIGH-003 — S-5.05 missing BC-2.06.005 v1.1 DI-029 cross-validation propagation

**Location:** `/Users/jmagady/Dev/prism/.factory/stories/S-5.05-config-loading.md` lines 20, 56, 126, 191

**Description:** BC-2.06.005 is declared in frontmatter; body covers postcondition 1 (all-errors-in-one-pass). Zero occurrences of `DI-029`, `correlation`, `cross-validat`, `window`, or `interval` in the body. Burst 40 lifted BC-2.06.005 to v1.1 adding a DI-029 cross-validation postcondition (WARN when correlation/sequence window < schedule interval) — but this propagation did not reach S-5.05.

**Canonical:** BC-2.06.005 v1.1:37 adds the DI-029 cross-validation postcondition as a new enforcement requirement.

**Policy violations:** Policy 8 (BC array changes propagate to body and ACs)

**Required fix:** Add DI-029 Task and AC for correlation-window vs schedule-interval cross-validation WARN.

---

### P3P39-A-HIGH-004 — S-5.06 Architecture Mapping cites non-existent tools and mis-attributes subsystem ownership

**Location:** `/Users/jmagady/Dev/prism/.factory/stories/S-5.06-action-infusion-tools.md` lines 403-415

**Errors:**
1. `trigger_action` — not a tool; canonical name is `fire_action` (renamed Burst 33). Story already uses `fire_action` 12+ times elsewhere in its body.
2. `test_infusion` — not a tool; no such tool appears in api-surface.md or interface-definitions.md v2.1.
3. Architecture Mapping narrative says "SS-18 owned by prism-spec-engine and prism-operations" and the ownership table maps SS-18 to prism-spec-engine. ARCH-INDEX:109 states SS-18 is owned by prism-operations only; SS-19 is owned by prism-spec-engine only.

**Policy violations:** Policy 4 (semantic anchoring — tool names) + Policy 6 (architecture is subsystem-name source of truth)

**Required fix:** Rewrite Architecture Mapping narrative: `trigger_action` → `fire_action`; remove `test_infusion`; SS-18 → prism-operations only; SS-19 → prism-spec-engine only.

---

### P3P39-A-HIGH-005 — VP-030 source_bc mis-anchored to non-enforcing BC with invented title

**Location:** `/Users/jmagady/Dev/prism/.factory/specs/verification-properties/vp-030-schedule-rule-caps.md` lines 12, 44

**Description:**
- Line 12 frontmatter: `source_bc: BC-2.12.010`
- Line 44 body: "Source BC: BC-2.12.010 — Schedule/Rule Capacity Limits"

BC-INDEX:162 shows BC-2.12.010 actual title is "Schedule State Persistence — RocksDB Domain for Scheduling Metadata". The title "Schedule/Rule Capacity Limits" is invented and does not match any BC in BC-INDEX. Furthermore, BC-2.12.010 does not enforce the DI-028 schedule/rule cap; invariants.md:47 names BC-2.12.001 and BC-2.13.006 as the enforcers.

**Policy violations:** Policy 4 (invented BC title) + Policy 7 (BC H1 is title source of truth)

**Required fix:** `source_bc` → `[BC-2.12.001, BC-2.13.006]`; fix body Source Contract title to verbatim BC-INDEX canonical titles for both BCs.

---

### P3P39-A-MED-001 — S-5.10 subsystem anchor SS-06 semantically wrong for audit forwarding

**Location:** `/Users/jmagady/Dev/prism/.factory/stories/S-5.10-audit-trail-forwarding.md` lines 6, 258

**Description:** SS-06 per ARCH-INDEX:97 = "Client Configuration". Audit trail forwarding is not client configuration. Peer story S-5.09 uses SS-20 (Observability / External log forwarding), which is the correct semantic anchor for this story's subject matter.

**Policy violations:** Policy 6 (architecture is subsystem-name source of truth)

**Required fix:** Replace SS-06 with SS-20 (or remove SS-06 and keep only SS-05); update Architecture Mapping table row accordingly.

---

### P3P39-A-MED-002 — Changelog discipline inconsistent across Burst 40's 73 v1.0→v1.1 bumps

**Location:** `/Users/jmagady/Dev/prism/.factory/stories/` — 71 stories with v1.1 frontmatter but no `## Changelog` section; also STORY-INDEX has no Burst 40 changelog entry.

**Description:** Only 4 of 75 stories have `## Changelog` sections (S-5.06, S-4.08, S-1.14, S-1.15). The 3 amended BC files (BC-2.12.001, BC-2.13.006, BC-2.06.005) correctly added Burst 40 changelog rows — the convention IS to record version history. However, the 73 stories bumped from v1.0 to v1.1 in Burst 40 (Architecture Mapping fill) have no `## Changelog` section. Additionally, STORY-INDEX has no Burst 40 changelog entry despite 73 story version bumps occurring in that burst.

**Policy violations:** Policy 2 (changelog completeness)

**Required fix:** Add `## Changelog` section with v1.0→v1.1 Burst 40 row to 71 stories (4 already compliant); add STORY-INDEX Burst 40 entry.

---

### P3P39-A-OBS-001 — BC-2.13.006 Traceability does not cite DI-024 despite validation gating new rules

**Location:** `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-2.13.006-create-rule-tool.md:69`

**Description:** The L2 Invariants row in the Traceability section has only DI-004 and DI-028. BC-2.13.006 delegates validation to BC-2.13.001 which enforces DI-024 (rule syntax validation gates creation). Adding DI-024 to BC-2.13.006's Traceability would make the enforcement-scope delegation explicit and visible.

**Severity:** Observational — BC-2.13.001 cites DI-024, so delegation satisfies coverage. No behavioral gap.

**Optional fix:** Add DI-024 to BC-2.13.006 Traceability L2 Invariants row for enforcement-scope visibility.

---

## Summary

| ID | Severity | Description | Status |
|----|----------|-------------|--------|
| P3P39-A-HIGH-001 | HIGH | S-4.01 error code E-SCHED-001 should be E-SCHED-008; cap 100 should be 500 — 4 sites | open |
| P3P39-A-HIGH-002 | HIGH | S-4.03 missing VP-030 + rule-cap AC + E-RULE-011 Task (BC-2.13.006 v1.1 not propagated) | open |
| P3P39-A-HIGH-003 | HIGH | S-5.05 missing DI-029 cross-validation AC + Task (BC-2.06.005 v1.1 not propagated) | open |
| P3P39-A-HIGH-004 | HIGH | S-5.06 Architecture Mapping: trigger_action→fire_action, remove test_infusion, SS-18/SS-19 ownership fix | open |
| P3P39-A-HIGH-005 | HIGH | VP-030 source_bc mis-anchored to BC-2.12.010 (non-enforcing); title invented; correct: BC-2.12.001 + BC-2.13.006 | open |
| P3P39-A-MED-001 | MED | S-5.10 SS-06 wrong for audit forwarding; correct is SS-20 (Observability) | open |
| P3P39-A-MED-002 | MED | 71 stories missing ## Changelog for v1.0→v1.1 Burst 40 bump; STORY-INDEX missing Burst 40 entry | open |
| P3P39-A-OBS-001 | OBS | BC-2.13.006 Traceability omits DI-024 (delegation satisfies coverage; observational only) | open |

**Total findings: 8 (0 CRIT / 5 HIGH / 2 MED / 0 LOW / 1 OBS)**

**Convergence counter: 0/3** (HIGH findings block advance; prior counter was 0/3)

**Novelty assessment:** Burst 40 cleanup validation surfaced Policy 8 propagation gaps — DI-028/DI-029 BC v1.1 lifts did not propagate to implementing stories S-4.03/S-5.05; S-4.01 error-code drift predates Burst 40 but was surfaced by the DI-028 audit; VP-030 source_bc mis-anchor is a pre-existing anchor error; Architecture Mapping residual in S-5.06 is regression from Burst 33 rename; MED-002 changelog discipline gap is systemic across 71 stories.

### Sweeps Clean

- BC-2.12.001/BC-2.13.006/BC-2.06.005 DI-028/DI-029 citations + error case additions present ✓
- BC changelog rows for Burst 40 ✓
- interface-definitions.md v2.1 sections 1.34-1.49 schema-consistent ✓
- api-surface.md Mermaid 28/24 arithmetic ✓
- error-taxonomy.md E-SCHED-008 and E-RULE-011 present ✓
- policies.yaml v1.1 Policy 8 × 4 verification_steps use behavioral_contracts: ✓
- configure_credential_source rename propagated in interface-definitions.md §1.6 ✓
- VP-INDEX v1.3 arithmetic 39=20+11+6+2 ✓
- DI-028/DI-029 bidirectional cited by BCs (forward coverage) ✓
