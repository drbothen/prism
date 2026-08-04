---
document_type: story
story_id: "S-MAINT-DISPATCH-BRIEF-POINTER-001"
title: "Codify Dispatch Brief Pointer Discipline — Orchestrator Briefs Supply Verifiable Pointer Targets, Not Asserted Authority Content"
wave: maintenance
epic_id: maintenance
priority: P2
status: draft
version: "1.0"
updated: "2026-08-03"
level: ops
producer: story-writer
timestamp: "2026-08-03T00:00:00Z"
tdd_mode: facade
# tdd_mode: facade — this story's deliverable is a CLAUDE.md convention amendment
# plus an upstream issue. There is no executable artifact (no shell script, no Rust
# code, no WASM plugin) to write failing tests for. The Red Gate / SAC-1
# test-authoring requirement does not apply to pure-documentation deliverables.
# Adversarial review of the CLAUDE.md amendment against the conventions it claims to
# codify is the quality gate (analogue of mutation testing at wave gate for convention
# artifacts). Any attempt to set tdd_mode: strict on this story would require waiving
# SAC-1 on a refuted rationale — the same defect found elsewhere in this corpus
# (CLAUDE.md §SAC-1: "A prior story was found defective for waiving SAC-1 on a
# refuted rationale; do not repeat that").
subsystems: []
# Cross-cutting orchestrator governance convention; no single product subsystem owns it.
crates_touched: []
target_module: "CLAUDE.md, drbothen/vsdd-factory upstream"
capabilities: []
behavioral_contracts: []
# BC status: pending PO authorship.
# This story codifies an orchestrator dispatch convention, not a sensor / data / query
# behavioral contract. A governance-integrity BC covering orchestrator dispatch quality
# would need to be authored by the product-owner before this story can reach
# status: ready (S-7.01 gate).
verification_properties: []
holdout_scenarios: []
depends_on: []
blocks: []
points: 2
estimated_days: 0.5
risk: LOW
acceptance_criteria_count: 5
red_gate_tests: 0
estimated_passes: "tbd"
assumption_validations: []
risk_mitigations: []
tags:
  - process-gap
  - orchestrator-discipline
  - factory-convention
  - brief-quality
  - d-2098
---

# S-MAINT-DISPATCH-BRIEF-POINTER-001: Codify Dispatch Brief Pointer Discipline — Orchestrator Briefs Supply Verifiable Pointer Targets, Not Asserted Authority Content

## Authority

`CLAUDE.md §Companion Principle — Correct Agent Routing` — the primary governance
source for orchestrator dispatch behavior and the agent routing table. CLAUDE.md carries
no `status:` frontmatter; it is always current per project convention (it functions as a
standing instruction set, not a versioned artifact with a lifecycle field).

`CLAUDE.md §Standing Adversary Probes & Implementer Disciplines` — where SAC-1, SAC-2,
SAP-1, SAP-2, SAP-3, SID-1, and SID-2 are codified alongside their codification
precedents. Same status caveat as above.

`STATE.md D-2098 §[process-gap] FIVE ORCHESTRATOR BRIEF-QUALITY DEFECTS` — the
decision-log entry recording the five defect shapes and the mitigation adopted
mid-cascade on 2026-08-03. **Verbatim `status:` from STATE.md frontmatter: `in_progress`** (STATE.md v8.647).

`STORY-INDEX.md §Changelog row D-2089` — records the D-2087 precedent: orchestrator
briefs using a pre-POL-39 exemplar caused 82 volatile version pins to be authored across
46 story files, all requiring remediation. This is the established prior-cycle instance
of "writers complied correctly with the brief; the defect originated entirely with the
brief." **Verbatim `status:` from STORY-INDEX.md frontmatter: `draft`** (STORY-INDEX.md v2.773).

---

## Origin

**Process-gap finding:** D-2098 `[process-gap]` section — five orchestrator brief-quality
defects recorded during the PR #234 FINDING-R cascade on 2026-08-03 (a comment-only
change to four sensor TOML specs that in practice required nine passes and produced
multiple HIGH-class findings in newly-written content).

**Five defect shapes (verbatim from D-2098):**

1. A uniform comment template was applied to two non-uniform cases, producing a factual
   claim that the governing ADR directly contradicts.
2. Story anchors were taken from a finding's prose text rather than from the ADR's own
   `anchor_stories` frontmatter ground truth (SAC-2 violation), yielding anchors that
   pointed at zero-AC registration stubs rather than the implementing stories.
3. An over-broad sibling-exclusion rationale was included in the brief; a later review
   pass refuted it.
4. A "narrow change only" scope boundary left two of four sibling files half-converted,
   generating five findings in a single pass.
5. A contradictory scope constraint was left un-retired when scope legitimately expanded;
   a subagent correctly refused to resolve it silently.

**Common root cause:** the brief ASSERTED authority content that executing agents then
transcribed in good faith. The agents complied correctly with what they were told. The
defect was upstream every time — the orchestrator, not the executor.

**Mitigation adopted mid-cascade (D-2098):** briefs stopped asserting authority content
and instead supplied pointer TARGETS, with an explicit instruction that the executing
agent must verify each pointer resolves and governs its subject BEFORE writing it, and
must STOP and report rather than substitute a nearby section.

**Measured effect:** findings originating in newly-written content fell from 8 per review
cycle to 2 in the burst immediately following adoption. The executing agent independently
caught both of the two known-bad pointers and returned a verification table.

**Precedent — D-2087/D-2089:** Orchestrator briefs using a pre-POL-39 exemplar caused 82
volatile version pins to be authored across 46 story files before a remediation pass
(D-2087). The D-2089 session wrap records this as "orchestrator briefs used pre-POL-39
exemplar from S-ADR055-WAVE-A-001; 82 volatile version pins were authored then
remediated in D-2087." Same defect class: writers complied correctly; defect originated
entirely with the brief.

This maintenance story codifies the pointer-not-content discipline into CLAUDE.md as a
standing rule, making it as durable as SAP-1, SAP-2, and SAC-1/SAC-2 rather than an
ad-hoc mid-cascade correction.

---

## Narrative

As an orchestrator dispatching a specialist agent to write or fix a spec artifact,
I want CLAUDE.md to codify the dispatch brief discipline — briefs supply verifiable
pointer targets; executing agents verify each target before writing it — so that brief
quality defects cannot silently become agent output defects, and a brief's assertion
about authority content is never a sufficient reason for an agent to transcribe that
assertion without independent verification.

---

## Behavioral Contracts

| BC | Title | Version | Scope in This Story |
|----|-------|---------|---------------------|
| (none) | — | — | No behavioral contracts yet. See BC status comment in frontmatter. Story MUST remain `status: draft` until a governance-integrity BC is authored and anchored. |

---

## Acceptance Criteria

### AC-001 — CLAUDE.md codifies the pointer-not-content rule for dispatch briefs

`CLAUDE.md §Companion Principle — Correct Agent Routing` MUST gain an explicit paragraph
(or a new `§Dispatch Brief Discipline` sub-section) that states:

- Dispatch briefs supply **verifiable pointer targets** — section headings, BC IDs,
  symbol names, story IDs, decision IDs — rather than restating what those authorities
  say. The orchestrator does not write "BC-2.16.013 says X"; it writes "read
  BC-2.16.013 §Postconditions §1 before authoring the annotation."
- A brief that asserts authority content shifts defect responsibility to the brief, not
  to the executing agent who complied with it. This is the D-2098 / D-2087/D-2089
  defect class: writers complied correctly; the defect originated entirely with the
  brief.
- The rule applies to ALL agent dispatch briefs, not only orchestrator-to-story-writer
  dispatches. Every specialist dispatch that references authoritative content is in scope.

### AC-002 — CLAUDE.md requires executing agents to verify pointers and report a verification table

The same amendment MUST state:

- When a dispatch brief supplies a pointer target, the receiving agent MUST resolve the
  pointer — locate the referenced section or symbol in its stated file — and verify that
  it governs the subject being written, BEFORE writing any content derived from it.
- The receiving agent MUST include a **pointer verification table** in its output for
  every pointer supplied by the brief. Minimum columns: pointer as given in the brief,
  resolved file path, resolution status (RESOLVED / UNRESOLVED), governing-subject
  verdict (GOVERNS / DOES-NOT-GOVERN / AMBIGUOUS).
- A pointer that resolves to a file but does not govern the subject (e.g., the section
  exists but covers a different sensor than the one being annotated) MUST be treated as
  UNRESOLVED for the purpose of the verification obligation — resolving by filename alone
  is insufficient.

### AC-003 — CLAUDE.md requires executing agents to STOP on a non-resolving pointer, not substitute

The amendment MUST state:

- When a pointer supplied by a brief does NOT resolve (section absent, file does not
  exist, section exists but does not govern the subject), the executing agent MUST stop
  and report the non-resolution back to the orchestrator. The agent MUST NOT substitute
  a nearby section, a section from a different version, or its own paraphrase.
- Substitution is the failure mode. A substituted nearby section will often be
  superficially similar to the intended pointer target but will differ in the exact
  clause that matters — reproducing the same defect class as D-2098 defect shape (1):
  a factual claim that the governing authority directly contradicts.
- The agent's stop-and-report MUST name: the pointer as written in the brief, the
  resolution attempt (what was searched, what was found or not found), and a proposed
  corrected pointer if the agent can determine it with confidence.

### AC-004 — CLAUDE.md requires superseded scope constraints to be retired by name

The amendment MUST state:

- When scope legitimately changes mid-task (e.g., the orchestrator learns during a
  cascade pass that the original "narrow change only" constraint no longer matches the
  actual defect class), the brief update MUST explicitly retire the prior constraint by
  name or by quoting it. Silently superseding a constraint by adding a broader
  instruction does not remove the prior instruction from the receiving agent's context.
- Precedent: during the PR #234 FINDING-R cascade (D-2098), a subagent correctly
  refused to resolve a contradiction between a "narrow change only" scope constraint and
  a newly-issued broader instruction because the narrow constraint was never explicitly
  retired. The subagent's behavior was correct; the orchestrator's brief was defective.
  This AC codifies the correction so that the same refusal cannot recur.
- The retirement form is: "~~Prior constraint: [quoted text]~~ — superseded by this
  instruction because [reason]." A strikethrough with rationale is sufficient; an
  entirely new brief that omits the prior constraint without acknowledging it is NOT
  sufficient (the agent's context may already contain the prior constraint from an
  earlier message).

### AC-005 — Mechanizability assessment documented; upstream issue filed

A `§Mechanizability Assessment` section is added to the CLAUDE.md amendment (or
immediately adjacent to it in this story for reference) that honestly states the
mechanizability of each AC clause per the disclosure standard in CLAUDE.md
§TD-VSDD-092 (gate capability boundaries must be stated explicitly; a check that passes
without verifying anything is theatre).

A GitHub issue is filed against `drbothen/vsdd-factory` documenting: (a) the five
D-2098 defect shapes and their common root cause; (b) the pointer-not-content rule and
the verification-table requirement; (c) the scope-constraint retirement rule; (d) the
measured effectiveness evidence (8 findings per cycle → 2 after mid-cascade adoption);
(e) a reference to D-2087/D-2089 as the established prior-cycle precedent; and (f) a
reference to this story. The upstream issue URL is recorded in §Deliverables.

---

## Mechanizability Assessment

Each AC clause is evaluated against the disclosure standard in CLAUDE.md §TD-VSDD-092:
"a gate must be evidenced against the property it claims to enforce."

| Clause | Mechanizable? | Analysis |
|--------|--------------|---------|
| AC-001: Brief supplies pointer targets, not asserted content | **Non-mechanizable** | A script cannot distinguish "pointer target" (instructs agent to read X) from "asserted content" (tells agent what X says). Semantic analysis of orchestrator intent is required. |
| AC-002: Executing agent produces a verification table | **Partially mechanizable — output format only** | A script can detect the presence of a verification table heading and the expected columns in agent outputs. It cannot verify that the resolution verdicts are accurate or that the agent actually read the pointer target rather than writing a plausible table. This is a weak check: an agent can write a formal-looking table with inaccurate verdicts. |
| AC-003: Agent stops on non-resolving pointer; does not substitute | **Non-mechanizable** | Whether an agent stopped and reported, vs. silently substituted a nearby section, requires reading the agent's transcript and comparing its written content against the actual pointer target. No script can evaluate this. |
| AC-004: Superseded constraint retired by name in brief | **Non-mechanizable** | A script cannot determine whether a new instruction in a brief is intended to supersede an earlier one, or whether both constraints apply. Task context is required. |
| AC-005 verification table presence (output format gate) | **Partially mechanizable** as above for AC-002 | |

**Gate capability boundary (per TD-VSDD-092 disclosure obligation):** One of five
clauses is partially mechanizable at the output-format level. Zero clauses are fully
mechanizable with load-bearing verification. The substantive quality gate for this story
is **adversarial review of the CLAUDE.md amendment** against the conventions it claims
to codify, not a gate script. An adversary reading the amendment can verify: does the
text supply pointer examples rather than content examples? Are the stop-and-report
instructions unambiguous? Is the scope-constraint retirement form defined? These are
semantic checks that no script performs. The "partially mechanizable" output-format
check for the verification table is retained as documentation for tooling authors; it is
NOT the quality gate. This is an honest disclosure: the convention lives in agent
behavior, not in gate enforcement.

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| CLAUDE.md §Companion Principle amendment | `CLAUDE.md` (project root) | Pure (convention document; no runtime behavior) |
| Upstream issue | `drbothen/vsdd-factory` GitHub | Effectful (network; AC-005) |

No Rust crates, no shell scripts, no hook plugins. All deliverables are documentation.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | A brief cites a pointer that was recently restructured (section heading changed) | The executing agent reports UNRESOLVED with the attempted resolution path; the orchestrator updates the pointer. This is the desired behavior — the agent does not guess at the new heading. |
| EC-002 | A pointer resolves but the governing section is ambiguous (two equally-applicable subsections) | Agent reports AMBIGUOUS in the verification table and describes both candidates; orchestrator selects the specific sub-pointer before the agent proceeds. |
| EC-003 | The brief supplies no explicit pointer targets (all instructions are self-contained) | No verification table is required. The pointer-not-content rule is an obligation on pointer-bearing briefs; a brief that legitimately contains no pointer targets is out of scope. |
| EC-004 | A brief's scope constraint is implicitly superseded (the orchestrator sends a new brief without referencing the prior one) | The receiving agent's prior context still contains the old constraint. If the agent detects a contradiction, it MUST report it per AC-004 rather than silently resolving it. The convention does not require agents to infer implicit retirements. |
| EC-005 | An executing agent mistakes a version-pin citation for an authoritative pointer target | Version pins are not pointer targets (POL-39 / TD-VSDD-091). A brief that instructs "read BC-2.16.013 §Postconditions §1" is correct; a brief that asserts "BC-2.16.013 says X" is an asserted-content violation. The distinction is: does the instruction require the agent to READ the target, or does it paraphrase the target's content? |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~5,000 |
| `CLAUDE.md §Companion Principle — Correct Agent Routing` (read before amending) | ~4,000 |
| `CLAUDE.md §Standing Adversary Probes & Implementer Disciplines` (SAC pattern reference) | ~3,000 |
| CLAUDE.md amendment text (authoring) | ~1,500 |
| Upstream issue text (authoring) | ~1,000 |
| **Total** | ~14,500 |

Well within a single agent context window. No split required.

---

## Tasks

### Implementation tasks

- [ ] **T-01**: Read `CLAUDE.md §Companion Principle — Correct Agent Routing` and
  `CLAUDE.md §Standing Adversary Probes & Implementer Disciplines` in full before
  authoring the amendment. Confirm the precise location for the new text (new
  `§Dispatch Brief Discipline` sub-section within Companion Principle, or as a named
  block adjacent to the routing table). Record the location decision in the PR
  description.

- [ ] **T-02**: Author the CLAUDE.md amendment implementing AC-001, AC-002, AC-003, and
  AC-004. Include at least one concrete positive example (correct pointer-not-content
  brief form) and one negative example (incorrect asserted-content form) per the SAC-1/
  SAC-2 precedent pattern used in the Standing Adversary Probes section.

- [ ] **T-03**: Include the `§Mechanizability Assessment` from this story as an inline
  annotation in the CLAUDE.md amendment text (or as a comment block), so future readers
  understand the gate capability boundary without consulting this story file. The
  disclosure obligation from TD-VSDD-092 applies to CLAUDE.md governance sections.

- [ ] **T-04**: File upstream issue against `drbothen/vsdd-factory` per AC-005. Record
  URL in §Deliverables.

- [ ] **T-05**: Run the adversarial review of the CLAUDE.md amendment text before
  declaring done. Specifically verify: (a) every AC clause is addressed in the amendment
  text; (b) no example in the amendment is itself an asserted-content violation (examples
  must model the rule, not contradict it); (c) the scope-constraint retirement form
  (AC-004) is unambiguous enough to be applied without orchestrator-level judgment.

### Merge gate

- [ ] **MERGE-GATE-AMENDMENT-AC-COMPLETE**: CLAUDE.md amendment addresses all five ACs.
- [ ] **MERGE-GATE-EXAMPLE-INTEGRITY**: No example in the amendment is itself an
  asserted-content violation (examples model the rule).
- [ ] **MERGE-GATE-UPSTREAM-ISSUE**: Upstream issue URL recorded in §Deliverables (T-04
  complete).

---

## Previous Story Intelligence

First story in the dispatch-brief-pointer-discipline track. No predecessor in this
specific track.

Related prior art:

- `S-MAINT-RG-LIST-GATE-001` — process-gap follow-up establishing the pattern:
  identify root cause → specify enforceable convention (or gate) → upstream issue. This
  story follows the same pattern but delivers a CLAUDE.md convention rather than a gate
  script. Key structural precedent.

- `S-MAINT-ADR-ANCHOR-GATE-001` — process-gap follow-up for the ADR `anchor_stories`
  integrity gap (F-WASE-P64-OBS-001). Two-tier gate with honest mechanizability
  (hard-block vs warning). Structural precedent for the two-tier honest-gate pattern;
  this story extends the pattern to the non-mechanizable case.

- `S-MAINT-BURST-COMMIT-COUNT-GATE-001` — direct structural precedent for the "gate
  must be evidenced against the property it claims to enforce" principle (CLAUDE.md
  §TD-VSDD-092 disclosure pattern). This story applies the same disclosure standard to
  a convention instead of a gate script.

- **D-2087/D-2089 precedent (direct evidence):** orchestrator briefs using a pre-POL-39
  exemplar caused 82 volatile version pins to be authored across 46 story files, all
  requiring remediation. The D-2089 session wrap names this as "writers complied
  correctly; defect originated entirely with the brief." This story codifies the
  correction as a standing rule.

- **D-2098 measured evidence (direct trigger):** five brief-quality defects in a single
  comment-only cascade (PR #234); mid-cascade adoption of pointer-not-content briefs
  reduced cycle findings from 8 to 2; executing agent caught both remaining bad pointers
  independently and returned a verification table. This is the measured evidence that
  makes codification justified (not merely aspirational).

---

## Architecture Compliance Rules

1. **No Rust crate modifications.** This story MUST NOT add, remove, or edit any file
   under `crates/`. Scope is CLAUDE.md plus the upstream issue.

2. **No STATE.md edits.** STATE.md is state-manager territory.

3. **No STORY-INDEX.md edits.** Registration is a state-manager burst, not this
   story's deliverable.

4. **TD-VSDD-053 single-commit-per-burst applies.** All `.factory/` changes (if any)
   must go in one atomic commit. The story's own delivery is bound by the rule it
   partially addresses.

5. **CLAUDE.md is a project-root file, not a `.factory/` artifact.** Edits to CLAUDE.md
   are not subject to the factory-artifacts branch protocol; they go in a normal
   feature branch PR targeting develop. No `.factory/` commit is required for the
   CLAUDE.md change itself.

6. **Amendment must model the rule it codifies.** Any example brief in the CLAUDE.md
   amendment MUST use pointer-not-content form. An example that asserts authority
   content would be a self-contradicting amendment — the merge gate T-05 exists
   specifically to catch this.

---

## Library and Framework Requirements

No library dependencies. Deliverable is a CLAUDE.md text amendment. No changes to
Cargo.toml files, Rust crates, shell scripts, or hook plugins.

---

## File Structure Requirements

### Files to MODIFY

| File | Change |
|------|--------|
| `CLAUDE.md` | Add `§Dispatch Brief Discipline` or equivalent sub-section within `§Companion Principle — Correct Agent Routing`, covering AC-001 through AC-004 with examples and mechanizability annotation (AC-005) |

### Files to CREATE

None (the upstream issue is an external artifact, not a file in the repository).

### Files NOT to modify

| File | Reason |
|------|--------|
| Any `crates/**` file | Out of scope (Architecture Compliance Rule 1) |
| `.factory/STATE.md` | State-manager territory |
| `.factory/stories/STORY-INDEX.md` | Registration is a state-manager burst |
| Any `.factory/specs/**` file | Out of scope; this is a dispatch-convention story, not a BC/ADR/VP amendment |

### Forbidden Dependencies

None. This story adds no new build dependencies.

---

## §Deliverables

| Item | Status | Reference |
|------|--------|-----------|
| Upstream issue URL | Pending | (to be filled at T-04 completion) |

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-08-03 | story-writer | S-7.02 cycle-close follow-up per D-2098 process-gap: five brief-quality defects in PR #234 cascade; measured evidence (8 findings/cycle → 2 after mid-cascade adoption). Codifies pointer-not-content dispatch brief discipline in CLAUDE.md: pointer targets vs asserted content (AC-001), agent verification table (AC-002), stop-on-non-resolving-pointer (AC-003), superseded-constraint retirement by name (AC-004), mechanizability assessment (AC-005). tdd_mode: facade (no executable artifact; convention document amendment). status: draft; behavioral_contracts: [] pending PO authorship (S-7.01). |
