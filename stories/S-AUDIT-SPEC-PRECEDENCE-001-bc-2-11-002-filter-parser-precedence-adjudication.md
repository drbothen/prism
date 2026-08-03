---
document_type: story
story_id: "S-AUDIT-SPEC-PRECEDENCE-001"
title: "BC-2.11.002 filter-parser mode-detection precedence adjudication and code alignment"
wave: maintenance
epic_id: maintenance
priority: P1
status: draft
version: "0.2"
spec_version: "v0.2"
level: ops
producer: story-writer
timestamp: "2026-07-12"
modified: "2026-07-12"
input-hash: ""
inputs:
  - .factory/specs/behavioral-contracts/BC-2.11.002-prismql-filter-mode.md
  - crates/prism-query/src/filter_parser.rs
traces_to: "F-AUD-P24-MED-004"
origin_finding: "F-AUD-P24-MED-004 [spec-vs-code drift]"
origin_cascade: "AUDIT-COVERAGE-001 B-hardening; D-1696 (passes 22–25); LOCAL 3-CLEAN converged D-1713 (2026-07-12)"
cycle: "v1.0.0-greenfield"
phase: 3
tdd_mode: strict
track: "Platform Engineering"
subsystems: [SS-11]
crates_touched:
  - prism-query
target_module: "crates/prism-query/src/filter_parser.rs"
behavioral_contracts:
  - BC-2.11.002
# BC-2.11.002 v1.6 is active. It defines mode-detection precedence:
# pipe > SQL(SELECT) > SQL(FROM) > filter.
# F-AUD-P24-MED-004 reports a divergence between this spec and the
# filter_parser.rs implementation. PO adjudication is the first required action (AC-001).
# Per CLAUDE.md §Source-of-Truth Precedence: the SPEC wins unless the human
# authorizes amendment. The code must be brought into alignment (AC-002/003)
# unless the PO amends BC-2.11.002 to match the current code behavior.
# AC↔BC bidirectional traces will be completed once the adjudication outcome is known.
verification_properties: []
depends_on: []
blocks: []
points: 5
estimated_days: 1.5
risk: MEDIUM
acceptance_criteria_count: 4
red_gate_tests: 4
estimated_passes: "2-3"
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
---

# S-AUDIT-SPEC-PRECEDENCE-001: BC-2.11.002 filter-parser mode-detection precedence adjudication and code alignment

## §Origin — [spec-vs-code drift] F-AUD-P24-MED-004

**Cascade:** AUDIT-COVERAGE-001 B-hardening; finding surfaced at pass 24
**Session record:** D-1696 (SESSION WRAP passes 22–25; 4 MED findings at pass-24 including F-AUD-P24-MED-004)
**Convergence:** LOCAL 3-CLEAN(strict) D-1713 (2026-07-12); S-7.02 codification gate now due

At pass 24 the adversary found that `crates/prism-query/src/filter_parser.rs` implements
mode-detection precedence logic that diverges from the canonical ordering specified in
BC-2.11.002 §Preconditions.

**BC-2.11.002 §Preconditions specifies (in order, first match wins):**
1. If the query contains `|` outside string literals → **pipe mode** (BC-2.11.004)
2. If the query starts with `SELECT` (case-insensitive) → **SQL mode** (BC-2.11.003)
3. If the query starts with `FROM` (case-insensitive) and has no `|` → **SQL mode** (BC-2.11.003)
4. Otherwise → **filter mode** (BC-2.11.002)

Per CLAUDE.md §Source-of-Truth Precedence (rule 1), the spec wins over the code for contract
semantics disputes. The code must be brought into alignment unless the product-owner explicitly
authorizes amending BC-2.11.002 to match the current code behavior. This story gates the
implementation work on PO adjudication (AC-001) and then aligns whichever artifact is authoritative
with the other (AC-002/003).

**Important:** The specific nature of the divergence (which precedence rules differ, in what cases)
must be determined by the implementer reading both the BC and `filter_parser.rs` as the first
task. The finding ID (F-AUD-P24-MED-004) records that a divergence exists; the exact delta is
to be confirmed by the implementer's direct examination of both artifacts.

## Narrative

As a Prism developer querying the system, I want mode-detection to behave exactly as documented
in BC-2.11.002 §Preconditions (pipe > SELECT-SQL > FROM-SQL > filter), so that the correct query
parser is invoked in all boundary cases (e.g., a query starting with `FROM` but containing `|`
is always treated as pipe mode, not SQL mode).

## Authority

BC-2.11.002 is the primary authoritative contract for this story. Read it before implementing:
`.factory/specs/behavioral-contracts/BC-2.11.002-prismql-filter-mode.md` (status: `active`).

BC-2.11.002 §Preconditions defines the four-rule mode-detection precedence ordering (pipe > SELECT-SQL > FROM-SQL > filter). This is the canonical contract that `crates/prism-query/src/filter_parser.rs` must implement. The story's Red Gate tests (AC-003) assert this precedence at the public parser surface.

**CLAUDE.md §Source-of-Truth Precedence rule 1** governs the adjudication in AC-001: for code-vs-spec conflicts, the spec wins unless the product-owner explicitly authorizes amendment. BC-2.11.002 §Preconditions is therefore authoritative over `filter_parser.rs` until the product-owner rules otherwise. Option B (spec amendment) requires the product-owner to act before any code is written.

ADR-047 is cited in BC-2.11.002 §Postconditions for case-insensitive operator support (`IEQ`, `IIN`, `INE`). ADR-047 does not affect mode-detection precedence — it governs operator semantics within filter mode and is not directly binding on this story's scope.

The code artifact to align: `crates/prism-query/src/filter_parser.rs` (SS-11 per `architecture/module-decomposition.md §Subsystem Registry`). See also `architecture/module-decomposition.md §SS-11 Query Execution`.

---

## Behavioral Contracts

| BC | Title | Version | Relevance |
|----|-------|---------|-----------|
| BC-2.11.002 | PrismQL Filter Mode Parsing | v1.6 | §Preconditions defines the 4-rule mode-detection precedence ordering. The code divergence identified by F-AUD-P24-MED-004 must be resolved by either aligning the code to this spec or amending the spec to match the code (PO decision). |

## Acceptance Criteria

### AC-001 — Product-owner adjudicates the BC-2.11.002 vs filter_parser.rs precedence divergence
(traces to BC-2.11.002 v1.6 §Preconditions — mode-detection precedence rules; **FIRST ACTION REQUIRED**)

**This AC requires PO action before implementation begins.** The implementer first reads:
- BC-2.11.002 §Preconditions (mode-detection precedence rules 1–4)
- `crates/prism-query/src/filter_parser.rs` (the current mode-detection logic)

The implementer documents the exact divergence in a comment in this story's §Implementation Notes.
The product-owner then makes one of two decisions:

**Option A (Spec wins, default per CLAUDE.md §Source-of-Truth Precedence):**
The code in `filter_parser.rs` is incorrect; it must be updated to implement the 4-rule
precedence as written in BC-2.11.002. No BC amendment required. The implementer proceeds
to AC-002 as a code-only fix.

**Option B (PO authorizes spec amendment):**
The code in `filter_parser.rs` implements the correct intended behavior; BC-2.11.002 has
drifted from the implementation intent. The PO authors a BC-2.11.002 amendment before
the implementer writes any code. The amendment must fully specify the correct precedence rules
as the new canonical ordering. The implementer then proceeds to AC-002 verifying the code
matches the amended spec.

**Default if PO is unreachable:** Option A applies per CLAUDE.md §Source-of-Truth Precedence.
Do not proceed to AC-002 without a recorded adjudication decision.

### AC-002 — filter_parser.rs implements the adjudicated precedence order
(traces to BC-2.11.002 §Preconditions — mode-detection precedence, adjudicated version)

After AC-001 adjudication:

Under **Option A:** `crates/prism-query/src/filter_parser.rs` is updated to implement the
BC-2.11.002 four-rule precedence in the exact order: (1) `|` outside strings → pipe, (2) starts
with `SELECT` → SQL, (3) starts with `FROM` and no `|` → SQL, (4) otherwise → filter. Any
code path that returns a different mode for a query matching rules 1–3 is a bug.

Under **Option B:** `crates/prism-query/src/filter_parser.rs` is verified to match the
amended BC-2.11.002 §Preconditions; no code change required if the code already matches the
amended spec.

Red Gate test: `test_BC_2_11_002_mode_detection_precedence_pipe_beats_sql` asserts that a
query containing `|` and starting with `SELECT` is parsed as pipe mode, not SQL mode (tests the
rule-1 > rule-2 precedence).

### AC-003 — Test coverage for all four mode-detection precedence rules
(traces to BC-2.11.002 §Preconditions — mode-detection rules 1–4, adjudicated version)

`crates/prism-query/tests/filter_mode.rs` (or equivalent test module) gains four tests covering
each of the four rules and their boundary interactions:

| Test | Rule | Input | Expected mode |
|------|------|-------|---------------|
| `test_BC_2_11_002_rule1_pipe_wins_over_select` | Rule 1 > Rule 2 | `SELECT * FROM t | where x = 1` | pipe |
| `test_BC_2_11_002_rule2_select_wins_over_filter` | Rule 2 > Rule 4 | `SELECT * FROM t` | SQL |
| `test_BC_2_11_002_rule3_from_with_no_pipe_is_sql` | Rule 3 > Rule 4 | `FROM t | limit 10` (pipe present) vs `FROM t LIMIT 10` (no pipe) | pipe / SQL |
| `test_BC_2_11_002_rule4_plain_expression_is_filter` | Rule 4 | `severity >= 5 AND host = "x"` | filter |

The test file must reference BC-2.11.002 §Preconditions in its module-level comment for
bidirectional traceability (TD-VSDD-091 function-name anchoring, POL-4).

### AC-004 — Bidirectional BC trace updated in this story file
(traces to BC-2.11.002 adjudicated version — bidirectional trace update requirement)

After AC-001 adjudication resolves which version of BC-2.11.002 is canonical, this story file
is updated:
- `behavioral_contracts:` frontmatter list gains the post-adjudication BC version pin if a
  BC amendment was made (Option B), or retains `BC-2.11.002` at the existing version if no
  amendment was needed (Option A).
- The §Behavioral Contracts table is updated with the current BC version and title.
- Each AC's trace annotation is updated to reference the correct BC clause once the adjudicated
  text is known.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| Mode-detection logic | `crates/prism-query/src/filter_parser.rs` | Pure (string analysis, returns `QueryMode` enum or equivalent) |
| Mode-detection tests | `crates/prism-query/tests/filter_mode.rs` | Pure unit tests |
| BC-2.11.002 (PO action) | `.factory/specs/behavioral-contracts/BC-2.11.002-prismql-filter-mode.md` | Spec artifact (PO-owned) |

Architecture section references:
- `architecture/module-decomposition.md` §SS-11 Query Execution

**Anchor justifications (POL-4/POL-5):**
- SS-11 owns this story's scope because `filter_parser.rs` is in `crates/prism-query` (SS-11 per
  ARCH-INDEX Subsystem Registry).
- No `depends_on` dependencies: mode-detection is self-contained in `filter_parser.rs`; no other
  story must complete first.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Query starts with `FROM` and contains `|` in a string literal (e.g., `FROM t WHERE x = "a|b"`) | BC-2.11.002 rule 1: `|` must be outside string literals; this query has `|` only inside a string, so rule 1 does NOT trigger; rule 3 applies (FROM with no outside-literal `|`); result is SQL mode |
| EC-002 | Empty string query | BC-2.11.002 §Error Cases: E-QUERY-001 "Query string is empty" — none of rules 1–4 apply in the normal precedence sense; the validator returns an error before mode detection |
| EC-003 | Query starts with `select` (lowercase) | BC-2.11.002 rule 2 states `SELECT` case-insensitive; `select` triggers SQL mode |
| EC-004 | Query starts with `FROM` with no pipe but has `|` in a nested subquery string | Same as EC-001 — `|` in a string literal does not trigger rule 1 |

## Token Budget Estimate

| Item | Lines | Tokens (est.) |
|------|-------|--------------|
| Story spec (this file) | ~150 | ~2,100 |
| BC-2.11.002 (mode-detection preconditions) | ~70 | ~1,000 |
| crates/prism-query/src/filter_parser.rs (mode detection region) | ~100 | ~1,400 |
| crates/prism-query/tests/filter_mode.rs (existing tests for context) | ~80 | ~1,100 |
| **Total estimate** | | **~5,600 tokens** |

Fits within a 100k-token agent context window (~6%). No split required.

## Tasks

- [ ] Read BC-2.11.002 §Preconditions (mode-detection precedence, 4 rules).
- [ ] Read `crates/prism-query/src/filter_parser.rs` mode-detection implementation.
- [ ] Document the exact divergence in §Implementation Notes below; present to PO for AC-001 adjudication.
- [ ] Record PO's Option A or Option B decision.
- [ ] If Option B (spec amendment): wait for PO to amend BC-2.11.002 before writing code.
- [ ] Write 4 Red Gate tests (AC-003) BEFORE any code change (TDD strict).
- [ ] Implement code fix or spec alignment per adjudication outcome (AC-002).
- [ ] Update frontmatter `behavioral_contracts:` and §Behavioral Contracts table with final BC version (AC-004).
- [ ] Run `just iter prism-query` to confirm GREEN.
- [ ] Run `just check` (full workspace) before declaring done.

## Previous Story Intelligence

N/A — first story targeting the BC-2.11.002 vs filter_parser precedence drift. Prior context:
- BC-2.11.002 was last amended at v1.6 (2026-07-08) to add ADR-047 IEQ/IIN/INE operator support.
  The mode-detection precedence rules in §Preconditions were present from v1.0 and have not been
  amended since cycle-1-burst-45 (2026-04-19).
- `crates/prism-query/src/filter_parser.rs` exists at HEAD acf7ded0; the exact divergence from
  BC-2.11.002 §Preconditions was observed by the adversary at pass 24 but the precise delta was
  not recorded in the finding — the implementer must diagnose it directly.
- Related stories S-3.01 (PrismQL parser) and S-PRISMQL-CASE-INSENSITIVE-001 both touched
  filter_parser.rs; any precedence changes in those stories should be verified against BC-2.11.002.

## §Implementation Notes

*To be populated by the implementer at task-start after reading both artifacts.*

The exact divergence between BC-2.11.002 §Preconditions rule ordering and the current
`filter_parser.rs` mode-detection logic is recorded here before PO adjudication:

> [IMPLEMENTER: fill in the specific difference found — e.g., "Rule 3 (FROM with no pipe) is
> checked before rule 2 (SELECT) in the code, reversing the spec's order" or "Pipe detection
> does not correctly exclude `|` inside string literals".]

## Architecture Compliance Rules

- **CLAUDE.md §Source-of-Truth Precedence rule 1:** Story spec (scope) vs BC (contract semantics): for mode-detection, BC-2.11.002 is the canonical contract. The code must align to the BC unless PO amends it. Do NOT silently amend the spec in code or documentation without PO action.
- **`#[non_exhaustive]` discipline:** If a `QueryMode` enum is modified, check if it is `#[non_exhaustive]`; if it is public and in `prism-query`, it must be.
- **TD-VSDD-091:** Cite function names (mode-detection function name to be determined at implementation time), NOT line numbers.
- **No `unwrap()` / `expect()` in production code:** Mode detection must use `?` or pattern matching; no panics on edge-case inputs.

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `nextest` | workspace-pinned | `just iter prism-query` for fast inner loop |
| `chumsky` | workspace-pinned | Filter mode parser; no version change expected |

No new dependencies.

**Forbidden dependencies (build-time enforcement):** `prism-query` mode-detection MUST NOT
depend on `prism-mcp` or `prism-spec-engine`. The existing perimeter gates apply.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-query/src/filter_parser.rs` | Modify (Option A) | Fix mode-detection precedence to match BC-2.11.002 §Preconditions |
| `crates/prism-query/tests/filter_mode.rs` | Modify | Add 4 Red Gate tests (AC-003) |
| `.factory/specs/behavioral-contracts/BC-2.11.002-prismql-filter-mode.md` | Modify (Option B, PO action) | Amend §Preconditions if code is adjudicated correct |
| `.factory/stories/S-AUDIT-SPEC-PRECEDENCE-001-....md` (this file) | Modify | Update `behavioral_contracts:` pin + AC traces after adjudication (AC-004) |

## Changelog

| Version | Burst | Date | Author | Changes |
|---------|-------|------|--------|---------|
| 0.2 | DRIFT-STORY-AUTHORITY-ABSENT-CORPUS-001-R6 | 2026-08-02 | story-writer | Add §Authority section (D-2084 Round 6 DRIFT-STORY-AUTHORITY-ABSENT-CORPUS-001). BC-2.11.002 §Preconditions cited as governing contract; CLAUDE.md §Source-of-Truth Precedence rule 1 cited for adjudication authority; ADR-047 scope-exclusion noted. |
| 0.1 | — | 2026-07-12 | story-writer | Initial story creation. |
