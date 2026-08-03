---
document_type: story
story_id: "S-AUDIT-INVARIANTS-001"
title: "Extract 300+ in-file F-AUD-PN citations into a structured audit-invariants.md companion spec"
wave: maintenance
epic_id: maintenance
priority: P2
status: draft
version: "0.2"
spec_version: "v0.2"
level: ops
producer: story-writer
timestamp: "2026-07-12"
modified: "2026-07-12"
input-hash: ""
inputs:
  - scripts/t13-preflight-audit.py
  - .factory/stories/S-AUDIT-PROCESS-CONVENTIONS-001-audit-check-pass-predicates-and-error-grading.md
traces_to: "F-AUD-P29-OBS-001"
origin_finding: "F-AUD-P29-OBS-001 [process-gap]"
origin_cascade: "AUDIT-COVERAGE-001 B-hardening; D-1698 (pass 29); LOCAL 3-CLEAN converged D-1713 (2026-07-12)"
cycle: "v1.0.0-greenfield"
phase: 3
tdd_mode: strict
track: "Platform Engineering"
subsystems: []
crates_touched: []
target_module: "scripts/audit-invariants.md"
behavioral_contracts: []
# BC status: pending PO authorship
# F-AUD-P29-OBS-001 targets the documentation architecture of the audit script's
# behavioral invariants. No pre-existing BC governs how audit-script invariants are
# documented or structured. PO may author a BC covering audit documentation completeness,
# or this story may be implemented as a pure process-improvement without a BC (common
# for toolchain documentation stories).
# Status must remain draft until either a BC is authored (S-7.01 gate) or the PO
# explicitly waives the BC requirement for toolchain documentation stories.
verification_properties: []
depends_on: [S-AUDIT-PROCESS-CONVENTIONS-001]
blocks: []
points: 5
estimated_days: 1.0
risk: LOW
acceptance_criteria_count: 4
red_gate_tests: 0
estimated_passes: "1-2"
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
---

# S-AUDIT-INVARIANTS-001: Extract 300+ in-file F-AUD-PN citations into a structured audit-invariants.md companion spec

## §Origin — [process-gap] F-AUD-P29-OBS-001

**Cascade:** AUDIT-COVERAGE-001 B-hardening; finding surfaced at pass 29
**Session record:** D-1698 (SESSION WRAP pass 29; item 9 added to S-7.02 queue at cascade convergence)
**Convergence:** LOCAL 3-CLEAN(strict) D-1713 (2026-07-12); S-7.02 codification gate now due

At pass 29 of the AUDIT-COVERAGE-001 cascade (the penultimate pass before final 3-CLEAN convergence),
the adversary observed that `scripts/t13-preflight-audit.py` had accumulated approximately 300+
inline F-AUD-PN comment citations scattered throughout the 2,000+ line script. These citations encode
the behavioral invariants that each audit check was added to enforce:

```python
# F-AUD-P12-OBS-001: C7 LIMIT 5 → vacuous sort window; widened to LIMIT 12
# F-AUD-P21-OBS-003: parse_envelope five-layer isinstance-guard added
# F-AUD-P24-CRIT-001: NameError crash in check A17 — rename missed call site
```

The problem: these citations exist purely as inline comments, making them:
1. **Unsearchable at the spec level** — there is no single document that lists all invariants by finding ID
2. **Undiscoverable by new maintainers** — requires reading 2,000 lines to understand why each check exists
3. **Un-verifiable by review** — an adversary cannot confirm "all high-severity findings from passes 1–44 have corresponding guards in the audit script" without a cross-reference document

The fix: extract these citations into a structured `scripts/audit-invariants.md` companion spec that:
- Lists each audit check (A1–C8 or equivalent) with its behavioral invariant
- Records which finding ID motivated each guard
- Notes the pass at which it was added
- Enables adversarial verification by cross-reference (not by grep)

The inline comments in `t13-preflight-audit.py` remain as-is (they serve as code navigation aids);
`audit-invariants.md` provides the structured spec view.

## Narrative

As a Prism developer or adversary maintaining `scripts/t13-preflight-audit.py` after the
AUDIT-COVERAGE-001 cascade, I want a single `scripts/audit-invariants.md` file that maps every
audit check to the finding ID that motivated it and the behavioral invariant it enforces, so that I
can verify that no high-severity finding from the 44-pass cascade went without an audit guard, and
so that new findings added to the script are traceable to their origin.

## Authority

No numbered ADR governs audit-script documentation structure. The governing authorities for this story are:

**Origin finding:** F-AUD-P29-OBS-001 (AUDIT-COVERAGE-001 cascade, pass 29) is the process-gap that triggered this story. Session record D-1698 contains the authoritative finding text: approximately 300+ inline F-AUD-PN citations in `scripts/t13-preflight-audit.py` are not navigable at spec level. The companion spec `scripts/audit-invariants.md` addresses this gap.

**CLAUDE.md §Operational Discipline TDs — TD-VSDD-091:** The anti-volatile-pin rule requires that behavioral invariants be cited by function name and finding ID, not line number. This is precisely the convention that `audit-invariants.md` enforces for the audit script's per-check invariant records — each row cites a check name (e.g., `A23`, `C7`) and a finding ID, never a line number.

**S-AUDIT-PROCESS-CONVENTIONS-001** (prerequisite story) creates `scripts/audit-conventions.md` — the file that AC-003 extends with an `audit-invariants.md` cross-reference. This story must not be implemented before S-AUDIT-PROCESS-CONVENTIONS-001 is complete.

No product BCs govern this story. The `behavioral_contracts: []` status is intentional per the frontmatter note; PO authorship or explicit waiver required before `status: ready` (S-7.01).

---

## Behavioral Contracts

No active BCs govern audit-script documentation structure. See frontmatter note.

## Acceptance Criteria

### AC-001 — scripts/audit-invariants.md exists and covers all audit check groups
(pending BC trace — BC authorship or PO waiver required before status=ready)

`scripts/audit-invariants.md` is created as a companion spec to `scripts/t13-preflight-audit.py`.
The document is structured in check-group sections matching the audit script's logical groupings
(e.g., §Group A: Tool Availability Checks, §Group B: Schema Fidelity Checks, §Group C: Query
Semantics Checks, or whatever groupings the script uses). Each group section contains a table:

```markdown
| Check ID | Check Name | Behavioral Invariant | Motivated By | Pass Added |
|----------|-----------|---------------------|--------------|------------|
| A1  | tool_available  | MCP server responds to tools/list | F-AUD-P1-HIGH-001 | pass 1 |
| A23 | nya_stubs       | Each NYA stub returns -32003 individually (SAR-1) | F-AUD-P26-OBS-002 | pass 26 |
| C7  | sort_detections | Detection severity sort uses LIMIT 12 (SAR-2) | F-AUD-P28-OBS-005 | pass 28 |
```

The table must cover at minimum:
- All checks that have inline F-AUD-PN citations in `t13-preflight-audit.py`
- All checks that were added or modified during the AUDIT-COVERAGE-001 cascade (passes 1–44)

Checks with no inline citation are included with "Motivated By: baseline" and "Pass Added: initial".

### AC-002 — Every HIGH or CRIT finding from AUDIT-COVERAGE-001 is covered by at least one table row
(pending BC trace — BC authorship or PO waiver required before status=ready)

Cross-reference verification: for each HIGH or CRIT finding recorded in the cascade session records
(D-1694 through D-1713), verify that `audit-invariants.md` has at least one table row in the
"Motivated By" column referencing that finding ID (or a finding ID from the same pass that addresses
the same check).

This is the completeness gate that proves the document is not partial. LOW/OBS/PROCESS-GAP findings
are included on a best-effort basis but are not gated here — the gate is specifically on HIGH and CRIT
severity findings (which represent actual behavioral gaps that were actively guarded in the script).

The implementer queries the session records (STATE.md D-1694–D-1713 decision entries) to produce
the list of HIGH/CRIT finding IDs, then verifies each appears in the document. Any HIGH/CRIT finding
without a corresponding row is a completeness gap that must be filled before this AC is satisfied.

### AC-003 — audit-invariants.md is referenced from the audit-conventions.md §Conventions header
(pending BC trace — BC authorship or PO waiver required before status=ready)

`scripts/audit-conventions.md` (created by S-AUDIT-PROCESS-CONVENTIONS-001) gains a cross-reference
at the top of the document:

```markdown
> **See also:** `audit-invariants.md` — per-check behavioral invariant catalog mapping every
> audit check to its originating finding ID and the pass at which it was added.
```

This ensures that a developer who reads `audit-conventions.md` can navigate to the invariant catalog.

### AC-004 — t13-preflight-audit.py module docstring references audit-invariants.md
(pending BC trace — BC authorship or PO waiver required before status=ready)

The module-level docstring at the top of `scripts/t13-preflight-audit.py` gains a reference:

```python
"""
T13 Preflight Audit Script — AUDIT-COVERAGE-001 B-Hardening

Behavioral invariants for each check are documented in scripts/audit-invariants.md.
Conventions for PASS predicates, error grading, and review axes are in scripts/audit-conventions.md.
"""
```

The existing docstring (if any) is updated to include the `audit-invariants.md` reference; if no
module docstring exists, one is added. The content must not conflict with existing inline comments.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| audit-invariants.md | `scripts/audit-invariants.md` | Pure data (companion spec document) |
| audit-conventions.md cross-reference | `scripts/audit-conventions.md` | Modify (add cross-reference) |
| t13-preflight-audit.py docstring | `scripts/t13-preflight-audit.py` | Modify (module docstring only) |

Architecture section references: N/A — scripts-only, no Rust crates or subsystems.

**Anchor justifications:**
- No subsystem anchor: `scripts/` tooling is not in the ARCH-INDEX Subsystem Registry.
- `depends_on: [S-AUDIT-PROCESS-CONVENTIONS-001]`: `audit-conventions.md` must exist before AC-003
  can add a cross-reference to it. Furthermore, the conventions document establishes the PASS-grounding
  and error-grading conventions that individual `audit-invariants.md` rows should cite where applicable.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | A check in the script has no F-AUD-PN citation and was part of the original baseline | Included in `audit-invariants.md` as "Motivated By: baseline" + "Pass Added: initial"; not treated as a gap |
| EC-002 | A finding ID is referenced in `audit-invariants.md` but the finding number cannot be resolved to a specific STATE.md entry | Include the row with the finding ID as-is; add a "Note: pre-D-1694 finding; may be in earlier session records" annotation |
| EC-003 | The script is modified after `audit-invariants.md` is created (new checks added in a future story) | New checks must add a corresponding row to `audit-invariants.md` in the same PR (enforced by audit-conventions.md convention, AC-003 cross-reference serves as the reminder) |
| EC-004 | The audit script has been reorganized (check IDs A1-A23, B1-Bx, C1-C8 may differ from exact groupings) | The implementer reads the actual script to discover the true check groupings and IDs; do NOT assume any particular check-ID naming scheme |

## Token Budget Estimate

| Item | Lines | Tokens (est.) |
|------|-------|--------------|
| Story spec (this file) | ~160 | ~2,250 |
| scripts/t13-preflight-audit.py (scan for F-AUD-PN citations, ~2000 lines) | ~2,000 | ~28,000 |
| STATE.md D-1694–D-1713 (recover HIGH/CRIT finding IDs for AC-002) | ~200 | ~2,800 |
| scripts/audit-conventions.md (read before adding cross-reference) | ~100 | ~1,400 |
| audit-invariants.md (to create, ~150 rows × ~30 chars/row) | ~170 | ~2,400 |
| **Total estimate** | | **~36,850 tokens** |

This estimate is dominated by reading `t13-preflight-audit.py` (~28k tokens). Fits within a 100k-token
agent context window (~37%). No split required, but the implementer should read the script in targeted
passes (grep for F-AUD-PN comments first, then read the full script once) to work efficiently.

## Tasks

- [ ] Run `grep -n 'F-AUD-P' scripts/t13-preflight-audit.py` to collect all inline F-AUD-PN citation comments and their line numbers.
- [ ] Run `grep -n 'def check_\|def verify_\|def audit_\|^# ===\|^# ---\|^# Group' scripts/t13-preflight-audit.py` to discover the script's check groupings and naming conventions.
- [ ] Read STATE.md D-1694–D-1713 entries to enumerate HIGH/CRIT finding IDs from the cascade (AC-002 gate).
- [ ] Create `scripts/audit-invariants.md` with check-group sections and the full per-check table (AC-001).
- [ ] Verify all HIGH/CRIT finding IDs from D-1694–D-1713 appear in at least one table row (AC-002).
- [ ] Add `audit-invariants.md` cross-reference to `scripts/audit-conventions.md` top (AC-003). Verify `audit-conventions.md` exists first (created by S-AUDIT-PROCESS-CONVENTIONS-001).
- [ ] Add or update module docstring in `scripts/t13-preflight-audit.py` (AC-004).
- [ ] Run `python3 -c "import ast; ast.parse(open('scripts/t13-preflight-audit.py').read()); print('parse OK')"` to confirm the docstring edit did not break the script.

## Previous Story Intelligence

**Prerequisite story:** S-AUDIT-PROCESS-CONVENTIONS-001 creates `scripts/audit-conventions.md`.
AC-003 adds a cross-reference to that file. Implement only after S-AUDIT-PROCESS-CONVENTIONS-001 is
complete.

Prior cascade context:
- The AUDIT-COVERAGE-001 cascade ran 44 passes and 38 fix-bursts; the script grew from an initial
  ~500 lines to ~2,000+ lines through continuous hardening.
- At pass 29 the adversary noted the growing F-AUD-PN citation density had made the script
  "self-documenting in aggregate but not navigable at the spec level."
- The specific wording in D-1698: "300+ in-file F-AUD-PN citations → extract to audit-invariants.md"
  — the ~300 figure is approximate; the implementer's grep scan will produce the authoritative count.
- This story intentionally does NOT restructure or refactor `t13-preflight-audit.py`; it only
  adds a companion documentation file + minimal cross-references.

## Architecture Compliance Rules

- **TD-VSDD-091:** Cite check names (A23, C7) and finding IDs (F-AUD-P29-OBS-001), NOT line numbers.
- **No changes to Rust code:** This story touches only `scripts/` files; no Cargo.toml, no crate changes.
- **py_compile gate:** Any modification to `t13-preflight-audit.py` (AC-004 docstring) must pass `python3 -c "import ast; ast.parse(open('scripts/t13-preflight-audit.py').read())"` before committing.
- **Single-commit-per-burst (TD-VSDD-053):** The three file writes (audit-invariants.md, audit-conventions.md update, t13-preflight-audit.py docstring) are committed as ONE commit.

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| Python | 3.x (workspace standard) | Used for `ast.parse` docstring validation |

No new dependencies.

**Forbidden dependencies:** None applicable — scripts-only story.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `scripts/audit-invariants.md` | Create | Per-check behavioral invariant catalog |
| `scripts/audit-conventions.md` | Modify | Add `audit-invariants.md` cross-reference at top |
| `scripts/t13-preflight-audit.py` | Modify | Add/update module docstring only (AC-004) |

No Rust files, no Cargo.toml changes, no new Python scripts.

## Changelog

| Version | Burst | Date | Author | Changes |
|---------|-------|------|--------|---------|
| 0.2 | DRIFT-STORY-AUTHORITY-ABSENT-CORPUS-001-R6 | 2026-08-02 | story-writer | Add §Authority section (D-2084 Round 6 DRIFT-STORY-AUTHORITY-ABSENT-CORPUS-001). No numbered ADR governs; authority is origin finding F-AUD-P29-OBS-001, CLAUDE.md §TD-VSDD-091, and S-AUDIT-PROCESS-CONVENTIONS-001 prerequisite. |
| 0.1 | — | 2026-07-12 | story-writer | Initial story creation. |
