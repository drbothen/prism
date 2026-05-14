---
document_type: adversarial-pass-report
target_artifact: S-PLUGIN-PREREQ-D
pass_number: 38
burst_label: "Burst Y-reify"
verdict: BLOCKED
streak_before: 0/3
streak_after: 0/3
finding_summary: "0 CRIT / 0 HIGH / 2 MED / 0 LOW / 1 OBS"
findings_total: 2
combined_burst_note: "pass-38 reify + fix-burst-36 COMBINED in single commit per TD-VSDD-053 state-manager-domain consolidation (D-539)"
timestamp: 2026-05-14T00:00:00Z
producer: state-manager
---

# S-PLUGIN-PREREQ-D Adversarial Pass-38 Report

> **VERDICT: BLOCKED** — 0 CRIT / 0 HIGH / 2 MED / 0 LOW / 1 OBS
>
> **Combined-burst note:** This pass-38 report and fix-burst-36 closure report are co-committed in a single
> D-539 commit per TD-VSDD-053 single-commit discipline. Both findings are state-manager-domain (VP-INDEX and
> STORY-INDEX §Changelog schema corrections are state-manager artifacts per CLAUDE.md routing table), so
> consolidating reify + fix + closure into one atomic commit is context-budget-conserving and operationally
> correct. Same pattern as D-538 (fix-burst-35 combined burst).

---

## Verification Trail: Fix-Burst-35 Closures HELD

Before recording pass-38 findings, the adversary verified that fix-burst-35 (D-538) closures all held:

- **F-LP37-MED-001 HELD:** VP-INDEX:190 VP-PLUGIN-007 description reads "rejected at load time per AC-5 manifest
  gate (default-deny consumer is AC-7)" — canonical AC-5 anchor restored. `grep -n 'per AC-7 default-deny'
  .factory/specs/verification-properties/VP-INDEX.md` → ZERO active-body hits. All remaining hits are
  §Changelog historical rows (immutable per TD-VSDD-091).
- **VP-INDEX v1.36 frontmatter confirmed.**

However, during verification, the adversary discovered two §Changelog schema-corruption defects introduced
by D-533 (fix-burst-32) and D-538 (fix-burst-35) — both in state-manager-domain index artifacts.

---

## Findings

### F-LP38-MED-001 — VP-INDEX §Changelog v1.35 + v1.36 rows violate canonical 5-column schema

**Severity:** MEDIUM

**Location:** `.factory/specs/verification-properties/VP-INDEX.md` lines 234 (v1.36) and 235 (v1.35)

**Canonical schema (confirmed from header line 232):**
```
| Version | Burst | Date | Author | Change |
```
5 columns. Correct example (line 236, v1.34 row):
```
| 1.34 | F-LP2-LOW-006-fix | 2026-05-13 | architect | F-LP2-LOW-006 closure: ... |
```

**Defective row v1.36 (line 234, introduced by D-538):**
```
| 1.36 | 2026-05-14 | state-manager | F-LP37-MED-001 closure: ... | D-538 |
```
Analysis: Only 4 logical cells in the 5-column schema. The `Burst` column is entirely absent; `D-538` appears
as an orphaned 6th cell trailing the `Change` cell. This is structurally invalid for the VP-INDEX changelog.

**Defective row v1.35 (line 235, introduced by D-533):**
```
| 1.35 | 2026-05-14 | state-manager | F-LP34-LOW-001 closure: ... | D-533 |
```
Same defect: Burst column absent; `D-533` orphaned as trailing cell.

**Root cause:** Orchestrator dispatch prompt templates for D-533 and D-538 prescribed the wrong row format,
omitting the Burst column and placing the decision ID as a trailing orphan cell instead of folding it into
the Change cell as a prefix. The state-manager correctly followed the template verbatim, propagating the
schema error into both rows.

**Fix (in-scope — state-manager domain, combined burst):**
- v1.36 row: add "fix-burst-35" as Burst cell; fold "(D-538)" as prefix into Change cell; remove orphan trailing cell
- v1.35 row: add "fix-burst-32" as Burst cell; fold "(D-533)" as prefix into Change cell; remove orphan trailing cell
- Bump VP-INDEX version v1.36 → v1.37; add canonical 5-cell v1.37 changelog row

---

### F-LP38-MED-002 — STORY-INDEX §Changelog v2.102 row has extra trailing `| D-533 |` cell (3-col schema)

**Severity:** MEDIUM

**Location:** `.factory/stories/STORY-INDEX.md` line 932 (v2.102 row)

**Canonical schema (confirmed from header line 930):**
```
| Version | Date | Summary |
```
3 columns. Correct sibling rows (v2.101, v2.100, v2.99, v2.98, etc.) all use 3-col schema with D-NNN
decision references folded INTO the Summary cell.

**Defective row v2.102 (line 932, introduced by D-533):**
```
| v2.102 | 2026-05-14 | S-PLUGIN-PREREQ-D v1.31→v1.32 (fix-burst-32 closure: ...) per POL-9 | D-533 |
```
Analysis: 4 cells in a 3-column table. `D-533` appears as a 4th orphan cell trailing the Summary cell.
Inconsistent with all sibling rows in the table.

**Root cause:** Same D-533 dispatch prompt template defect as F-LP38-MED-001. The template prescribed
STORY-INDEX rows with a trailing `| D-NNN |` cell rather than folding the decision ID into the Summary cell.

**Fix (in-scope — state-manager domain, combined burst):**
- v2.102 row: remove `| D-533 |` trailing cell; prepend `(D-533)` as prefix in Summary cell
- Bump STORY-INDEX version v2.102 → v2.103; add canonical 3-cell v2.103 changelog row

---

### OBS-LP38-001 — [process-gap] §Changelog schema-integrity validator absent

**Severity:** OBSERVATION

**Location:** Process gap — orchestrator dispatch prompt templates

**Pattern:** §Changelog schema-corruption has now recurred in this cascade 4 times across 3 documents:
1. F-LP32-MED-002 (pass-32): PREREQ-D story §Changelog rows 1.27/1.28/1.29 missing Burst column (fix-burst-30 closed)
2. F-LP34-HIGH-001 (pass-34): PREREQ-D story §Changelog rows merged without inter-row newlines (fix-burst-32 closed)
3. F-LP38-MED-001 (this pass): VP-INDEX §Changelog rows 1.35/1.36 Burst column absent (this burst closes)
4. F-LP38-MED-002 (this pass): STORY-INDEX §Changelog row v2.102 trailing orphan cell (this burst closes)

Recurrences 3 and 4 share the same root cause (orchestrator dispatch prompt template for D-533 prescribed
incorrect schema for BOTH VP-INDEX and STORY-INDEX rows simultaneously).

**Proposed codification POL-26-candidate:** "Document-changelog schema-integrity validator: before committing
any §Changelog row addition, the state-manager MUST count the cells in the new row and verify it matches the
document's `| Version | ... |` header column count. Mismatches block the burst." The validator is a count
check: STORY-INDEX header = 3 cols → new row must have 3 cells; VP-INDEX header = 5 cols → new row must have
5 cells.

**Disposition:** Routed cycle-close session-reviewer adjudication. codification_candidates_active: 24 → 25.

**META-NOTE for future orchestrator dispatch templates:**
All future dispatch prompt template examples for §Changelog row additions MUST specify the document's
canonical schema and include a correctly-formatted example row:
- VP-INDEX: 5-col `| Version | Burst | Date | Author | Change |` with D-NNN folded into Change cell as prefix
- STORY-INDEX: 3-col `| Version | Date | Summary |` with D-NNN folded into Summary cell as prefix

---

## Trajectory Observation

Pass-38 trajectory: 4→1→4→5→1→1→3→4→5→5→5→2→1→**2** (passes 25..38).

The slight uptick from 1 to 2 (both MED, no CRIT/HIGH) is a §Changelog META-class recurrence, not a
novel semantic finding. The fix-burst-35 closures (AC-5 anchor) all held cleanly. The anchor-string class
that drove passes 32→33→34→37 is fully exhausted. Pass-39 adversary should:
1. Verify v1.37 + v2.103 changelog rows obey canonical schemas
2. Sweep all index documents for any remaining §Changelog schema deviations (sibling-sweep for META-class)
3. Verify F-LP38-MED-001/002 are fully closed (no residual orphan cells in VP-INDEX or STORY-INDEX changelogs)
4. Note OBS-LP38-001 POL-26 candidate for cycle-close queue

Convergence trajectory remains within the convergence zone. The META-class §Changelog schema-corruption
is now codified as a POL-26 candidate and mechanically preventable.

---

## Pass-38 vs Pass-37 Finding Class Comparison

| Finding | Pass-37 | Pass-38 |
|---------|---------|---------|
| AC-5 anchor restoration | F-LP37-MED-001 (closed) | HELD |
| §Changelog schema-corruption | — | F-LP38-MED-001/002 (new — META-class recurrence) |
| Process-gap observation | OBS-LP37-001 (POL-25) | OBS-LP38-001 (POL-26-candidate) |

Both findings are state-manager-domain and fully mechanical. No semantic drift, no BC divergence,
no story content defects.
