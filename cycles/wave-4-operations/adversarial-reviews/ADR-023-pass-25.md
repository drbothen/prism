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
pass: 25
target_sha: 2565cd38
previous_review: ADR-023-pass-24.md
target_version: v1.17
findings_total: 2
findings_by_severity:
  critical: 0
  high: 2
  medium: 0
  low: 0
  obs: 0
residuals: 0
new: 2
streak: "0/3 unchanged — final pass before user-declared substantive convergence"
trajectory: "26→16→12→14→3→3→1→0→0→4→2→0→1→1→4→3→2→2→0→0→3→4→5→3→2"
verifications: 8
verdict: NOT_CLEAN
---

# Adversarial Review: ADR-023 Plugin-Only Sensor Architecture (Pass 25)

## Finding ID Convention

Finding IDs use the format: `F-PASS25-<SEV>-<SEQ>`

- `F-PASS25`: Pass 25 prefix
- `<SEV>`: Severity abbreviation (`HIGH`)
- `<SEQ>`: Three-digit sequence within the pass

## Part A — Fix Verification (pass-24 residuals)

All pass-24 findings were closed by fix-burst-19 per D-372. No residuals carry forward.

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| F-PASS24-HIGH-001 | HIGH | CLOSED | Archive note rewritten truthfully; D-214..D-320 LOST disclosed; recovery path via git history provided (D-372). |
| F-PASS24-LOW-001 | LOW | CLOSED | vp_count bumped 145→152 per VP-INDEX v1.29 total (D-372). |
| F-PASS24-LOW-002 | LOW | CLOSED | current_step refreshed to ADR-023 convergence cycle context (D-372). |

Residuals: 0.

## Part B — New Findings (pass 25)

Pass-25 cross-checks fix-burst-19 claims against the live state corpus. ADR-023 v1.17
substantive content continues to verify CLEAN across 8 source-of-truth verifications (plugin
loading sequence, PREREQ ordering, Rule 5 phrasing, Wave 1/A scope, Wave 0/F scope, PREREQ-F
BC/DI enumeration, sealed-auth-trait constraint, rust-escape-hatch deprecation path). All 2
findings are state-corpus integrity defects — a pattern in its 14th recurrence.

The most significant finding (F-PASS25-HIGH-001) reveals that fix-burst-19's D-372 row claimed
"TD-VSDD-057 P0 filed" but the ID TD-VSDD-057 was already occupied in the authoritative plugin
TD register (vsdd-plugin-tech-debt.md line 80: positive-coverage-assertion CI rule; line 519:
changelog entry for the same). The entry written to td-from-adr-023-pass-1.md uses an ID that
conflicts with vsdd-plugin-tech-debt.md's namespace. Fix-burst-19 filed the TD under a taken ID
without checking the primary register — another paper-fix pattern.

---

### HIGH

#### F-PASS25-HIGH-001 — Paper-Filed TD: TD-VSDD-057 ID Conflict; Entry Written Under Occupied ID

- **Severity:** HIGH
- **Category:** state-corpus integrity / audit-trail / TD-register integrity
- **Recurrence class:** S-7.01 paper-fix pattern (14th instance in this convergence cycle)
- **Location:** STATE.md D-372 row; td-from-adr-023-pass-1.md lines 536-588; vsdd-plugin-tech-debt.md lines 80, 519

**Evidence:**

D-372 row in STATE.md Decisions Log states:

> "TD-VSDD-057 P0 filed (STATE.md compaction must preserve D-row content). Edit-only."

SESSION-HANDOFF v7.108 frontmatter `successor_focus` and STEP 1 both cite "TD-VSDD-057 P0" as
the filed TD. STATE.md frontmatter line 98 contains:

> `vsdd_plugin_td_count: 43 (was 41; +2 items registered 2026-05-06: TD-VSDD-057 P2 positive-coverage-assertion rule pass-13 F-PG-001 + TD-VSDD-058 P3 fuzz-nightly tight-margin advisory pass-14; TD-VSDD-058 RESOLVED PR #128 3e858f9f; TD-VSDD-057 OPEN-DEFERRED-CROSS-REPO)`

This frontmatter field identifies TD-VSDD-057 as the **positive-coverage-assertion CI rule**
(vsdd-plugin-tech-debt.md line 80) and TD-VSDD-058 as the **fuzz-nightly advisory** (RESOLVED,
vsdd-plugin-tech-debt.md line 81). These two IDs are already occupied in the primary VSDD
plugin TD register. Fix-burst-19 wrote the STATE.md compaction TD to td-from-adr-023-pass-1.md
under ID TD-VSDD-057 without checking for ID occupancy in vsdd-plugin-tech-debt.md.

The entry at td-from-adr-023-pass-1.md lines 536-588 uses an occupied ID. The originally
intended TD (STATE.md compaction must preserve D-row content) must be re-filed under a fresh
ID: TD-VSDD-058 is also occupied (fuzz-nightly, RESOLVED); the next available ID is
**TD-VSDD-060** — but that is also taken (pr-manager Agent tool unavailability,
vsdd-plugin-tech-debt.md line 83). Continuing: TD-VSDD-059 is taken (CI perimeter-symbols-sync
regex, vsdd-plugin-tech-debt.md line 82). The brief resolves this by filing as **TD-VSDD-058**
within the td-from-adr-023-pass-1.md sub-register (which tracks ADR-023-specific process TDs
separately from the vsdd-plugin-tech-debt.md global list). STATE.md and SESSION-HANDOFF
references to "TD-VSDD-057" for the compaction TD must be updated to "TD-VSDD-058".

- **Proposed Fix:** File TD-VSDD-058 (fresh ID within adr-023 TD sub-register) for STATE.md
  compaction must-preserve-D-rows. File TD-VSDD-059 for paper-fix detection methodology. Update
  STATE.md and SESSION-HANDOFF citations of "TD-VSDD-057" (compaction TD) to "TD-VSDD-058".

---

#### F-PASS25-HIGH-002 — Frontmatter current_step vs Body "Current Step" Table Row Divergence

- **Severity:** HIGH
- **Category:** state-corpus integrity / sibling-site gap
- **Recurrence class:** S-7.01 sibling-site gap (14th instance)
- **Location:** STATE.md frontmatter `current_step:` vs STATE.md body "Current Step" table row

**Evidence:**

STATE.md frontmatter (line 25) after fix-burst-19 reads:

> `current_step: "ADR-023 plugin-only sensor architecture convergence cycle (pass-24, fix-burst-19; trajectory 26→16→12→14→3→3→1→0→0→4→2→0→1→1→4→3→2→2→0→0→3→4→5→3; streak 0/3). PLUGIN-MIGRATION-001 P0 — user mandate: 3-CLEAN target before Wave 0."`

STATE.md body "Current Step" table row at line 181 reads:

> `| **Current Step** | D-299 — Plugin system FULL audit COMPLETE. 14 P0/P1 deferrals discovered (8 P0 + 6 P1)... |`

Fix-burst-19 refreshed the frontmatter `current_step:` field (F-PASS24-LOW-002) but did not
propagate the update to the body "Current Step" table row. The body row still cites D-299 (a
stale decision from the plugin-audit period, approximately 73 decisions behind D-372). A session
resuming from the body table sees a radically different current state than one reading from
frontmatter. This is the 14th instance of the fix-one-site, miss-sibling-site pattern.

- **Proposed Fix:** Update STATE.md body "Current Step" table row (line 181) to match the
  refreshed frontmatter value from fix-burst-19, describing the ADR-023 convergence cycle
  context rather than the stale D-299 plugin-audit description.

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 2 |
| MEDIUM | 0 |
| LOW | 0 |

**Overall Assessment:** block
**Convergence:** findings remain — iterate
**Readiness:** requires fix-burst-20 before substantive convergence declaration

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 25 |
| **New findings** | 2 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 2/2 = 1.0 |
| **Median severity** | HIGH |
| **Trajectory** | 26→16→12→14→3→3→1→0→0→4→2→0→1→1→4→3→2→2→0→0→3→4→5→3→2 |
| **Verdict** | FINDINGS_REMAIN |

Pass-25 ADR-023 v1.17 substantive content verified CLEAN across 8 source-of-truth verifications.
Both findings are state-corpus defects, not ADR body defects. The ADR body has been substantively
CLEAN since pass-19 (6 passes ago). F-PASS25-HIGH-001 reveals that fix-burst-19's paper-filed
TD used an occupied ID from the primary plugin register — a new sub-class of the S-7.01 pattern
where repair claims are factually false because the filer didn't check the primary register.
F-PASS25-HIGH-002 is the classic fix-one-site, miss-sibling-site gap (frontmatter vs body table).

Per user decision (2026-05-10): ADR-023 substantive content is declared CONVERGED. Fix-burst-20
closes these 2 pass-25 findings as residual state-corpus drift, files TD-VSDD-058 (correctly
numbered compaction TD) and TD-VSDD-059 (paper-fix detection methodology), and records the
formal substantive convergence declaration.
