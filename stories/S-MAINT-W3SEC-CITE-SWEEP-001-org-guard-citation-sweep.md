---
document_type: story
story_id: "S-MAINT-W3SEC-CITE-SWEEP-001"
title: "Cross-crate org-guard citation sweep — strip disavowed BC-3.5.002 precondition 3 from merged DTU crates and W3-FIX-SEC-001 story body"
wave: maintenance
epic_id: maintenance
priority: P2
status: in-progress
version: "1.1"
spec_version: "v1.1"
level: ops
producer: story-writer
timestamp: "2026-06-01"
modified: "2026-06-02"
input-hash: ""
inputs:
  - .factory/stories/W3-FIX-SEC-001-x-org-id-auth-enforcement.md
traces_to: "DRIFT-D943-001"
anchors: "DRIFT-D943-001"
cycle: "v1.0.0-greenfield"
phase: 3
tdd_mode: strict
track: "Platform Engineering"
subsystems: [SS-01]
crates_touched:
  - prism-dtu-crowdstrike
  - prism-dtu-cyberint
target_module: "crates/prism-dtu-crowdstrike, crates/prism-dtu-cyberint"
behavioral_contracts:
  - BC-3.5.002
# BC status: pending PO authorship — BC-3.5.002 is the authority cited for correction;
# AC traces are to the postcondition and the correction is a comment/doc sweep only.
verification_properties: []
depends_on: []
blocks: []
points: 2
estimated_days: 0.25
risk: LOW
acceptance_criteria_count: 4
red_gate_tests: 1
estimated_passes: "1"
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
---

# S-MAINT-W3SEC-CITE-SWEEP-001: Cross-crate org-guard citation sweep

## Narrative

As a Prism codebase maintainer, I want every org-guard authority citation in the
merged DTU crates and spec artifacts to reference **W3-FIX-SEC-001** (or
`BC-3.5.002 postcondition 2`) uniformly, so that the disavowed cite
`BC-3.5.002 precondition 3` — which actually governs `HarnessBuilder::build()` /
admin-token routing, not X-Org-Id→401 behavior — no longer appears in
post-merge code comments, spec-text, or story prose, and auditors reading any
org-guard site reach a valid authority immediately.

## Background

During the S-DEMO-CLAROTY-AUDIT-DTU-001 (PR #167) PR-LEVEL adversarial cascade,
the org-isolation guard's authority was found to be mis-cited as
`BC-3.5.002 precondition 3`. Precondition 3 governs routing-error observability
via HTTP 401 when a wrong-org request reaches the wrong **port** — this is about
the `HarnessBuilder::build()` / admin_token routing topology, not the per-handler
`X-Org-Id` header comparison that the DTU org-guard implements.

The correct authority for X-Org-Id→401 rejection behavior is:
- **W3-FIX-SEC-001** (the story that defined and mandated the instance-keyed
  validate_org_id pattern), and/or
- **BC-3.5.002 postcondition 2** (harness network isolation — the clause that
  specifies response behavior when org routing is violated at the handler layer).

The Claroty crate was corrected in PR #167 to cite **W3-FIX-SEC-001** uniformly.
The same disavowed cite remains in merged, out-of-PR-167-scope code:
- `crates/prism-dtu-crowdstrike/` — detections.rs, hosts.rs, state.rs, and any
  sibling org-guard sites (grep to locate by content, not by line number per
  TD-VSDD-091)
- `crates/prism-dtu-cyberint/` — org-guard sites (similar pattern; header name
  is `X-Prism-Org-Id` but the authority cite issue is identical)
- The merged W3-FIX-SEC-001 story body (`.factory/stories/W3-FIX-SEC-001-x-org-id-auth-enforcement.md`)
  — locate by content: search for `BC-3.5.002 precondition 3` in the prose

This is tracked as **DRIFT-D943-001** (cite-convention drift introduced during
the multi-crate W3-FIX-SEC-001 implementation before the precondition-3 disavowal
was established).

## Behavioral Contracts

| BC ID | Title | Relevant Clause |
|-------|-------|-----------------|
| BC-3.5.002 | Harness Network Isolation Invariants | Postcondition 2 (per-handler X-Org-Id→401 behavior — the CORRECT cite for org-guard handlers) |

Note: `BC-3.5.002 precondition 3` governs `HarnessBuilder::build()` / admin_token
routing (port-level routing topology) and is explicitly DISAVOWED as an authority
cite for per-handler org-guard code. This story removes all such disavowed cites.

## Acceptance Criteria

### AC-001: CrowdStrike and Cyberint crates contain zero disavowed cites (traces to BC-3.5.002 postcondition 2)

Running:

```
rg 'BC-3\.5\.002 precondition 3' crates/prism-dtu-crowdstrike/ crates/prism-dtu-cyberint/
```

returns **zero hits**. Every org-guard authority comment in those crates reads
`W3-FIX-SEC-001` (preserving any `(AC-NNN)` or `SEC-NNN` sub-clause suffixes),
matching the convention established in `crates/prism-dtu-claroty/` via PR #167.

### AC-002: W3-FIX-SEC-001 story body no longer embeds the disavowed cite (traces to BC-3.5.002 postcondition 2)

The merged story body at
`.factory/stories/W3-FIX-SEC-001-x-org-id-auth-enforcement.md` contains no
assertion that `BC-3.5.002 precondition 3` is the org-guard authority. Any
such occurrence is either removed or annotated with the correct authority
(`W3-FIX-SEC-001` or `BC-3.5.002 postcondition 2`).

Note: Precondition 3 may legitimately appear in the story's BC table row
describing the `HarnessBuilder::build()` behavior — that row is CORRECT and
must not be removed. Only occurrences that claim it as the authority for the
X-Org-Id→401 per-handler guard are disavowed.

### AC-003: No behavior change; full `just check` remains green (traces to BC-3.5.002 postcondition 2)

This is a comment, doc-string, and spec-text-only sweep. No Rust function body,
match arm, type definition, or test assertion is changed. Running `just check`
(fmt + clippy + nextest + doctests + crate-layout) succeeds with exit code 0
after the sweep. No test renames are required unless a test's name embeds the
string `BC-3.5.002 precondition 3` verbatim (grep first; adjust only if found).

### AC-004: Regression guard evaluated — decision recorded here (traces to BC-3.5.002 postcondition 2)

The implementer MUST evaluate whether a grep-based CI guard (e.g., a
`lefthook.yml` pre-commit step or `.factory/hooks/` script) is warranted to
prevent reintroduction of `BC-3.5.002 precondition 3` as an org-guard cite.

Decision criteria:
- If the disavowed string appears in fewer than 5 out of 3000+ source files
  post-sweep: **do not add a CI guard** (low reintroduction risk, high
  maintenance cost for a rare case; note the decision in this story's
  §Changelog instead).
- If the project already has a cite-pin lint hook (`validate-cite-pin-completeness.sh`
  per S-MAINT-POL29-HOOK-001): **extend it** with this disavowed string as a
  banned cite pattern rather than adding a standalone guard.
- Record the decision and reasoning in the commit message for this story.

Do not over-engineer. A `rg` one-liner in the pre-commit hook is acceptable
if warranted; a dedicated test file is not needed for a comment-only sweep.

## Tasks

1. **Locate all disavowed cites in CrowdStrike crate:**

   ```bash
   rg 'BC-3\.5\.002 precondition 3' crates/prism-dtu-crowdstrike/ --type rust
   ```

   For each hit: replace the cite with `W3-FIX-SEC-001` (preserving any
   parenthesized sub-clause such as `(AC-002)` or `(SEC-001)`).

2. **Locate all disavowed cites in Cyberint crate:**

   ```bash
   rg 'BC-3\.5\.002 precondition 3' crates/prism-dtu-cyberint/ --type rust
   ```

   Apply the same replacement pattern.

3. **Locate and correct the W3-FIX-SEC-001 story body:**

   ```bash
   rg 'BC-3\.5\.002 precondition 3' .factory/stories/W3-FIX-SEC-001-x-org-id-auth-enforcement.md
   ```

   For hits that claim precondition 3 as org-guard authority: update to
   `W3-FIX-SEC-001` or `BC-3.5.002 postcondition 2` as appropriate. Preserve
   the BC table row that correctly describes precondition 3's scope (port-level
   routing topology) — do not remove that row, only correct any authority-claim
   misuse.

4. **Verify workspace-wide clean sweep:**

   ```bash
   rg 'BC-3\.5\.002 precondition 3' crates/ .factory/stories/ --type rust
   ```

   Expected: zero hits in DTU org-guard contexts. (Legitimate appearances in
   BC files, ADRs, or architecture docs that actually govern HarnessBuilder
   topology are NOT targets of this sweep.)

5. **Evaluate CI guard (AC-004):** count remaining occurrences of the disavowed
   string workspace-wide, apply the decision criteria from AC-004, and record
   the decision in the commit message.

6. **Run `just check`** — must exit 0 with no test failures or clippy violations.

7. **Commit** with message citing `DRIFT-D943-001` and `S-MAINT-W3SEC-CITE-SWEEP-001`.
   No AI attribution per project git conventions.

## Architecture Mapping

| Component | Module | File(s) | Pure/Effectful |
|-----------|--------|---------|----------------|
| org-guard comment sites | prism-dtu-crowdstrike | `src/routes/detections.rs`, `src/routes/hosts.rs`, `src/routes/state.rs` (locate by grep) | N/A — comments only |
| org-guard comment sites | prism-dtu-cyberint | `src/routes/*.rs` (locate by grep) | N/A — comments only |
| Story body | .factory/stories | `W3-FIX-SEC-001-x-org-id-auth-enforcement.md` | N/A — spec text |

**Subsystem anchor justification:** SS-01 (Sensor Adapters) owns this story's scope
because all affected crates (`prism-dtu-crowdstrike`, `prism-dtu-cyberint`) are
Security Telemetry DTU adapters per the ARCH-INDEX Subsystem Registry definition
of SS-01. The `.factory/stories/` artifact edit is maintenance-scoped and follows
the crate boundary.

**Dependency anchor justification:** `depends_on: []` — the canonical convention
(org-guard authority is W3-FIX-SEC-001) is already established by PR #167
(S-DEMO-CLAROTY-AUDIT-DTU-001). This story requires no other story to land first;
PR #167 is already merged. `blocks: []` — this is a cleanup story; no downstream
story is gated on this sweep.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `BC-3.5.002 precondition 3` appears in a BC file governing HarnessBuilder | Do NOT change — that cite is correct in its native BC context. Scope of this sweep is DTU crate comments and story prose only. |
| EC-002 | A test name or test doc contains the disavowed string | Rename the test and update any references per TD-VSDD-060 sibling-site sweep. Record in commit message. |
| EC-003 | Cyberint org-guard comments use `X-Prism-Org-Id` rather than `X-Org-Id` — different header name, same disavowed cite | The header name difference is irrelevant to cite correction; apply the same `W3-FIX-SEC-001` replacement uniformly. |
| EC-004 | Post-sweep grep finds `BC-3.5.002 precondition 3` in an ADR or architecture doc | Out of scope for this story — record as a separate DRIFT item for the architect to triage. Do not expand scope here. |

## Previous Story Intelligence

- **W3-FIX-SEC-001** (merged, PR series): Established the `validate_org_id` pattern
  and the `W3-FIX-SEC-001` authority cite convention. The CrowdStrike and Cyberint
  implementations were written under this story before the precondition-3 disavowal
  was established, which is the root cause of this sweep.
- **S-DEMO-CLAROTY-AUDIT-DTU-001** (merged, PR #167): The Claroty crate was corrected
  to cite `W3-FIX-SEC-001` uniformly. The adversarial cascade that established the
  disavowal ran against this PR. Other crates were out of PR scope at that time.
- **S-MAINT-POL29-HOOK-001** (draft): If the CI guard evaluation (AC-004) concludes
  a grep-based hook is warranted, extend that hook's banned-cite registry rather than
  creating a standalone guard.
- **Lesson:** When a citation convention is corrected in one crate during an
  adversarial cascade, open a maintenance story immediately to propagate the correction
  to sibling crates. Do not rely on future cascades to catch workspace-wide drift.

## Architecture Compliance Rules

- **Comments only.** No Rust production logic, type definitions, or test assertions
  may change. `git diff` for this story must show only comment/doc-string/spec-text
  changes in `.rs` files and prose changes in `.md` files.
- **TD-VSDD-091 compliance.** Do not cite file:line-number anchors in the corrected
  comments. Reference the story ID (`W3-FIX-SEC-001`) as the behavioral authority.
- **TD-VSDD-060 sibling-site sweep.** If any test name contains the disavowed string,
  all callsites (in the same crate and adjacent crates if the test is pub) must be
  updated in the same commit.
- **No scope expansion.** If grep reveals the disavowed cite in ADRs, BC files, or
  architecture docs outside the two target DTU crates and the W3-FIX-SEC-001 story
  body, record those as new DRIFT items rather than expanding this story's scope.

## Library & Framework Requirements

No library changes. This story uses only `rg` (ripgrep) for discovery and Edit/Write
tools for corrections. No new Cargo dependencies.

| Tool | Purpose |
|------|---------|
| ripgrep (`rg`) | Locate disavowed cite occurrences before and after sweep |
| `just check` | Final verification that no compilation or test regressions were introduced |

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-dtu-crowdstrike/src/routes/detections.rs` | Modify (if hit) | Replace disavowed cite in comments |
| `crates/prism-dtu-crowdstrike/src/routes/hosts.rs` | Modify (if hit) | Same |
| `crates/prism-dtu-crowdstrike/src/routes/state.rs` | Modify (if hit) | Same |
| `crates/prism-dtu-crowdstrike/src/routes/*.rs` (other) | Modify (if hit) | Any additional files found by grep |
| `crates/prism-dtu-cyberint/src/routes/*.rs` | Modify (if hit) | Same replacement pattern |
| `.factory/stories/W3-FIX-SEC-001-x-org-id-auth-enforcement.md` | Modify | Correct org-guard authority prose; preserve HarnessBuilder row |

All changes are comment/spec-text-only. No new files created.

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~3 500 |
| BC-3.5.002 (relevant clauses) | ~800 |
| CrowdStrike route files (3-4 files, ~150 lines each) | ~2 000 |
| Cyberint route files (2-3 files, ~120 lines each) | ~1 500 |
| W3-FIX-SEC-001 story body | ~3 500 |
| grep output + diff verification | ~500 |
| `just check` output | ~500 |
| **Total** | **~12 300** |

Well within the 20-30% context window limit. No splitting required.

## §Progress

| Step | Status | Date | Notes |
|------|--------|------|-------|
| Code sweep — prism-dtu-crowdstrike + prism-dtu-cyberint | **DONE** | 2026-06-02 | 21 org-guard cite sites corrected on maintenance/w3sec-cite-sweep @ 9d4c48fd; `rg 'BC-3\.5\.002 precondition 3' crates/prism-dtu-crowdstrike crates/prism-dtu-cyberint` → zero hits post-sweep |
| Story-body cites — W3-FIX-SEC-001 | **DONE** | 2026-06-02 | W3-FIX-SEC-001 v1.0→v1.1 (D-953 factory burst): 3 stale `BC-3.5.002 precondition 3` mis-cites → `BC-3.5.002 postcondition 2` |
| LOCAL adversary pass | pending | -- | Runs against maintenance/w3sec-cite-sweep HEAD after D-953 burst commit |
| Push maintenance/w3sec-cite-sweep to origin | pending | -- | After LOCAL adversary CLEAN |
| PR creation + PR-LEVEL adversary + merge | pending | -- | fix-pr-delivery flow |

## §Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| v1.1 | 2026-06-02 | state-manager | D-953 factory burst: status draft→in-progress; §Progress table added; code sweep DONE @ 9d4c48fd (21 sites); story-body cites DONE (W3-FIX-SEC-001 v1.1). |
| v1.0 | 2026-06-01 | story-writer | Initial draft; anchors DRIFT-D943-001 |
