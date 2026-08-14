---
document_type: story
story_id: "S-MAINT-RECORDS-LINT-SHA-ARM-001"
title: "records-lint L9 Arm-6 — Git SHA Volatile-Cite Detection and Pre-Existing SHA De-Reference Sweep"
wave: tbd
epic_id: maintenance
priority: P2
status: draft
version: "0.1"
level: ops
producer: story-writer
timestamp: "2026-08-14"
modified: "2026-08-14"
input-hash: "[live-state]"
inputs: []
traces_to: ""
cycle: "DEFECT-ADAPTER-TLS-XDOME-LIVE-001"
phase: 3
tdd_mode: strict
track: "Platform Engineering"
subsystems: []
crates_touched: []
target_module: "scripts/records-lint.sh, .factory/specs/prd-supplements"
capabilities: []
behavioral_contracts: []
# BC status: pending PO authorship
verification_properties: []
depends_on: []
blocks: []
points: 3
estimated_days: 0.75
risk: MEDIUM
acceptance_criteria_count: 5
red_gate_tests: 0
estimated_passes: "tbd"
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
tags:
  - process-gap
  - factory-tooling
  - volatile-cite-enforcement
  - td-vsdd-091
  - td-vsdd-092
---

# S-MAINT-RECORDS-LINT-SHA-ARM-001: records-lint L9 Arm-6 — Git SHA Volatile-Cite Detection and Pre-Existing SHA De-Reference Sweep

## Origin

**Process-gap finding:** F-P29-OBS-001 (DEFECT-ADAPTER-TLS-XDOME-LIVE-001 cascade), recurrence of F-P21-OBS-001 and the pass-21 / pass-29 / pass-30 instances in the same defect cascade.

`scripts/records-lint.sh` L9 (`L9_CITE_PATTERN`, five arms) has no arm matching a bare feature-branch git SHA (7–40 lowercase hex chars). As a result, volatile SHA cites in `.factory/` records pass the gate undetected. The pattern has recurred across three named finding IDs and three separate adversary passes in the DEFECT-ADAPTER-TLS-XDOME-LIVE-001 cascade, meeting the 3-recurrence codification threshold established by TD-VSDD-097.

**Confirmed pre-existing instances** (surfaced by the F-P29-OBS-001 sweep; SHA values listed here as **search targets for the de-reference sweep in AC-004/AC-005**, not as record-tier location cites):

| File | Location (by section or version — no line numbers) | SHA value to replace |
|------|-----------------------------------------------------|----------------------|
| `interface-definitions.md` | v2.11 changelog row | `b39e21e9` |
| `interface-definitions.md` | v2.9 changelog row | `b39e21e9` |
| `interface-definitions.md` | v2.8 changelog row | `b39e21e9` |
| `error-taxonomy.md` | §INT: Internal Invariant Errors — `Internal.detail` convention note (live body prose, closing sentence) | `873b693f` |
| `error-taxonomy.md` | v2.35 changelog row | `51f071ff` |
| `error-taxonomy.md` | v1.75 changelog row | `873b693f` |
| `error-taxonomy.md` | v1.74 changelog row (corrected annotation appended at v1.75) | `873b693f` |
| `error-taxonomy.md` | v1.14 changelog row | `d36ecf22` |

**TD-VSDD-091 / TD-VSDD-092 anchor:** The five-arm `L9_CITE_PATTERN` in `scripts/records-lint.sh` is the sole enforcement mechanism for the TD-VSDD-091 amendment (2026-07-24) that retired volatile-cite exceptions for record-tier text. A missing arm class is a silent enforcement gap. This story closes the gap for the bare-hex-SHA cite class.

**Precedent gates:** S-MAINT-RG-LIST-GATE-001 (F-WASE-P64-MED-016), S-MAINT-ADR-ANCHOR-GATE-001 (F-WASE-P64-OBS-001). This story follows the same root-cause → gate-arm → self-probe → corpus-sweep pattern.

---

## Narrative

As an orchestrator running the records-lint gate before a `.factory/` commit,
I want `scripts/records-lint.sh` L9 to flag bare feature-branch git SHAs (7–40 lowercase hex chars) in staged additions to `.factory/` record files,
so that commit-SHAs cited as location anchors in adversary pass reports, changelog rows, and body text are blocked at commit time — preventing the SHA-cite recurrence pattern that has surfaced as F-P21-OBS-001 and F-P29-OBS-001 across multiple cascade passes.

---

## Acceptance Criteria

### AC-001 — L9 Arm-6 pattern: bare lowercase hex SHA detection in staged additions
(Traceability to BCs is pending PO authorship)

A new `_L9_ARM6` pattern MUST be added to `scripts/records-lint.sh` and incorporated into the combined `L9_CITE_PATTERN`. The pattern MUST:

- Match bare lowercase hex strings of exactly 7 to 40 characters (`[0-9a-f]{7,40}`) at word boundaries (`\b[0-9a-f]{7,40}\b`)
- NOT match all-uppercase or mixed-case hex strings (avoids false positives from UUID fragments, CSS-form color codes, and error code IDs such as `E-QUERY-NNN`)
- NOT match strings shorter than 7 characters (avoids false positives from short hex values used in non-SHA contexts)
- Be gated by the same staged-additions scope as existing arms: only new lines (prefixed `+` in `git diff --cached`) are checked; pre-existing (unchanged) lines are grandfathered per the ratchet model

The `_L9_ARM6` variable definition MUST include a comment block in the same format as Arm-1 through Arm-5, explaining: (a) what the arm matches, (b) the 7-char lower bound rationale, (c) the lowercase-only restriction rationale, and (d) a citation of this story (S-MAINT-RECORDS-LINT-SHA-ARM-001) and its origin findings (F-P29-OBS-001, F-P21-OBS-001).

If, during implementation, it is found that bare `L<NNN>` forms in changelog or body-row contexts have coverage gaps not addressed by existing Arm-5, the implementer MUST strengthen the relevant arm or document the gap with a concrete deferral against a named follow-up story. Undocumented coverage gaps may not be closed silently.

### AC-002 — Exclusion: `input-hash:` frontmatter line values
(Traceability to BCs is pending PO authorship)

Any hex string appearing as the VALUE on a frontmatter `input-hash:` line MUST NOT trigger Arm-6. Content hash fields carry hex strings that are not git location cites. The exclusion MUST be implemented either as:

- A per-line pre-filter in `run_l9` that skips lines matching the `^input-hash:` prefix before the pattern check, OR
- An inline exclusion applied after the Arm-6 match (analogous to the `L9_CHECK_NAME_EXEMPT` per-token exemption)

The chosen mechanism MUST be documented in a comment adjacent to `_L9_ARM6` in the script. The comment MUST name this story (S-MAINT-RECORDS-LINT-SHA-ARM-001) and the rationale for the exclusion.

### AC-003 — `--self-probe` PASS and FAIL cases for Arm-6
(Traceability to BCs is pending PO authorship)

`records-lint.sh --self-probe` MUST include all four of the following new cases after this story ships:

- **Arm-6 FAIL probe (staged git repo):** A staged addition containing a standalone bare 8-char lowercase hex SHA (a synthetic value matching the `[0-9a-f]{8}` form, not on an `input-hash:` line) produces an L9 FAIL. MUST use a temp git repo (same pattern as the existing Arm-1 full-repo probe) since L9 reads staged diff.
- **Arm-6 PASS probe — `input-hash:` exclusion:** A staged addition containing a line in the form `input-hash: "abc12340ef56789a"` does NOT produce an L9 FAIL.
- **Arm-6 PASS probe — uppercase exclusion:** A line containing an all-uppercase 8+ char hex string does NOT produce an L9 FAIL (string-test probe acceptable for this near-miss).
- **Arm-6 PASS probe — too-short exclusion:** A line containing a 6-char lowercase hex string does NOT produce an L9 FAIL (string-test probe acceptable).

Each probe case MUST include a diagnostic comment identifying the exclusion it exercises and citing this story. The total `--self-probe` verified-pass count increases by 4 (one FAIL probe confirmed + three PASS near-miss probes confirmed). The `run_self_probe` summary line (if one exists in the script header) MUST be updated to reflect the new count.

### AC-004 — Pre-existing SHA cites de-referenced in `interface-definitions.md`
(Traceability to BCs is pending PO authorship)

The three changelog rows in `interface-definitions.md` (versions v2.11, v2.9, and v2.8) that cite the bare SHA `b39e21e9` MUST be updated to replace the bare SHA with a durable anchor. Acceptable anchor forms: a D-NNN decision ID, a PR number (`#NNN`), or a named story ID — whichever correctly identifies the provenance of the change that was originally cited as the SHA.

All three rows MUST be updated in a single state-manager atomic burst (TD-VSDD-053 single-commit-per-burst). The `interface-definitions.md` frontmatter `version:` and changelog MUST be bumped to reflect the correction. After staging the updated file, `records-lint.sh --l9-only` MUST exit 0 (no Arm-6 violations in the staged additions).

### AC-005 — Pre-existing SHA cites de-referenced in `error-taxonomy.md`
(Traceability to BCs is pending PO authorship)

The following five locations in `error-taxonomy.md` that contain bare SHA values MUST be updated to replace each SHA with a durable anchor (D-NNN, PR number `#NNN`, or named story ID):

| Location (by section — no line numbers) | SHA to replace |
|------------------------------------------|----------------|
| §INT: Internal Invariant Errors — `Internal.detail` convention note (live body prose, closing sentence of the sweep-status paragraph) | `873b693f` |
| v2.35 changelog row | `51f071ff` |
| v1.75 changelog row | `873b693f` |
| v1.74 changelog row (the `[corrected at v1.75]` annotation appended inline) | `873b693f` |
| v1.14 changelog row | `d36ecf22` |

All five updates MUST go in a single state-manager burst (TD-VSDD-053). The `error-taxonomy.md` frontmatter `version:` and changelog MUST be bumped. After staging the updated file, `records-lint.sh --l9-only` MUST exit 0.

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| Arm-6 pattern + input-hash exclusion + self-probe cases | `scripts/records-lint.sh` | Effectful (blocks staged `.factory/` commits containing SHA cites) |
| Corpus sweep: SHA de-reference | `.factory/specs/prd-supplements/interface-definitions.md` | Pure (content correction, no behavioral change) |
| Corpus sweep: SHA de-reference | `.factory/specs/prd-supplements/error-taxonomy.md` | Pure (content correction, no behavioral change) |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | An `input-hash:` line carries a full 40-char content hash | Arm-6 MUST NOT flag — the `input-hash:` exclusion applies regardless of SHA length |
| EC-002 | A PR number in changelog text coincidentally forms a 7-char lowercase hex string (e.g., `#abc1234`) | The `#` character is not a word character; the `\b` boundary before `[0-9a-f]` prevents a match on the digits following `#`. Implementer MUST include a PASS near-miss probe for this form |
| EC-003 | A merged-commit SHA cited in PR provenance context (e.g., "merged as `abc1234f`") | This IS a genuine volatile cite and Arm-6 SHOULD flag it — durable form is a PR number, not a SHA. No automatic exemption for "merged" context |
| EC-004 | The story file itself (`S-MAINT-RECORDS-LINT-SHA-ARM-001`) contains the SHA strings from the de-reference table in §Origin | This story is committed before Arm-6 exists, so it is grandfathered by the ratchet model. If `--full-scan` is later extended to L9, the implementer MUST decide whether to: (a) exempt the de-reference work-item table from L9 scope, or (b) note it as an acceptable historical record. Document the decision in `run_l9` or the script header |
| EC-005 | A short hex string that is a valid OCSF class UID or UUID fragment (e.g., an 8-char lowercase hex segment from a UUID in a test vector) | 8-char lowercase hex WOULD trigger Arm-6. Implementer MUST evaluate frequency in `.factory/` corpus and either accept false positives as true violations (requiring re-anchoring) or add a documented exclusion for UUID-in-frontmatter patterns |
| EC-006 | The `--full-scan` flag is extended in a future story to include L9 scanning of pre-existing lines | Pre-existing grandfathered SHA cites not swept in T-08/T-09 MUST be reported as ADVISORY (not FAIL) in full-scan mode to preserve the ratchet model. If `--full-scan` currently does not run L9, no change is needed here |

---

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `scripts/records-lint.sh` (Arm-6 pattern + `run_l9` exclusion + self-probe) | effectful-shell | Reads git index, executes staged-diff inspection, exits non-zero to block commits; all side effects are deliberate gate enforcement |
| `interface-definitions.md` (content update) | pure-core | Static document content; no I/O side effects; update is a content correction only |
| `error-taxonomy.md` (content update) | pure-core | Static document content; no I/O side effects; update is a content correction only |

---

## Token Budget Estimate

| Item | Estimated tokens |
|------|-----------------|
| This story spec | ~3,500 |
| `scripts/records-lint.sh` (full read: Arm construction block + `run_l9` + `run_self_probe`) | ~8,000 |
| `S-MAINT-RG-LIST-GATE-001` (sibling gate structural reference) | ~2,000 |
| `interface-definitions.md` (changelog rows v2.11 / v2.9 / v2.8 for AC-004 de-reference) | ~2,000 |
| `error-taxonomy.md` (§INT body prose + v2.35 / v1.75 / v1.74 / v1.14 changelog rows for AC-005) | ~4,000 |
| Total | ~19,500 |

Well within a single agent context window. No split required.

---

## Tasks

### Red Gate tests (to be written by test-writer BEFORE implementation)

N/A — this story's deliverables are additions to a bash shell script and content corrections in two `.factory/specs/prd-supplements/` files. There is no Rust production code with `todo!()` stubs. The story is `tdd_mode: strict` but its implementation is a bash script extension. When the implementing agent is dispatched, test-writer MUST write failing tests for the script behavior (e.g., temp-git-repo probes that stage an 8-char lowercase hex addition and assert non-zero exit from `run_l9`, and probes that stage `input-hash:` lines and assert zero exit) BEFORE the Arm-6 logic is added to `scripts/records-lint.sh`.

**Red Gate density check** (BC-5.38.001): **0 pre-written named tests** at story-writing time. Tests will be enumerated when the implementing module (the `_L9_ARM6` integration point and `--self-probe` extension) is confirmed. Density check deferred to implementation-time pre-pass — the standard pattern for tooling stories where the test vehicle is a shell script (see S-MAINT-RG-LIST-GATE-001 and S-MAINT-ADR-ANCHOR-GATE-001 for the identical deferred-density precedent). This story's `status: draft` reflects the BC status pending PO authorship — it does not transition to `ready` until `behavioral_contracts:` is non-empty per S-7.01.

### Implementation tasks

- [ ] T-01: Read `scripts/records-lint.sh` in full: specifically the L9 pattern construction block (`_L9_ARM1` through `_L9_ARM5`, `L9_CHECK_NAME_EXEMPT`, `L9_CITE_PATTERN`), the `run_l9` function body, and the existing `run_self_probe` arm probes. Understand the integration points before writing Arm-6.
- [ ] T-02: Define `_L9_ARM6` in the L9 pattern construction block (7–40 lowercase hex at `\b` word boundaries). Include the full comment block (what matches, lower-bound rationale, lowercase-only rationale, origin citations S-MAINT-RECORDS-LINT-SHA-ARM-001 + F-P29-OBS-001).
- [ ] T-03: Extend `L9_CITE_PATTERN` to include `${_L9_ARM6}` in the union pattern (AC-001).
- [ ] T-04: Implement the `input-hash:` line exclusion for Arm-6 matches in `run_l9` (AC-002). Document the chosen mechanism in a comment. Verify the exclusion does not suppress legitimate SHA violations on non-`input-hash:` lines.
- [ ] T-05: Assess whether bare `L<NNN>` forms in changelog/body-row contexts have coverage gaps not addressed by existing Arm-5. If gaps exist, address them inline or document the gap with a deferral against a named follow-up story. Do NOT leave undocumented gaps.
- [ ] T-06: Add the four Arm-6 `--self-probe` cases to `run_self_probe` (AC-003): one full-repo FAIL probe + three near-miss PASS probes. Each MUST include a diagnostic comment citing S-MAINT-RECORDS-LINT-SHA-ARM-001 as origin.
- [ ] T-07: Run `scripts/records-lint.sh --self-probe` locally and confirm: (a) the new Arm-6 probes all report PASS, (b) no existing probes regress, (c) the total pass count increases by 4.
- [ ] T-08: De-reference the three SHA cites in `interface-definitions.md` changelog rows (versions v2.11, v2.9, v2.8 — the `b39e21e9` instances). Replace each with the appropriate D-NNN decision ID or `#NNN` PR number. Bump the `interface-definitions.md` version and changelog row (AC-004).
- [ ] T-09: De-reference the five SHA cites in `error-taxonomy.md` (§INT body prose convention note + v2.35 / v1.75 / v1.74 / v1.14 changelog rows). Replace each with a durable D-NNN or PR number anchor. Bump the `error-taxonomy.md` version and changelog row (AC-005).
- [ ] T-10: Stage the updated `interface-definitions.md` and `error-taxonomy.md`, then run `records-lint.sh --l9-only` and confirm exit 0 (no Arm-6 violations in the new staged text).
- [ ] T-11: (Conditional) If the SHA arm is a candidate for the upstream factory hook plugin: file an issue against `drbothen/vsdd-factory` documenting (a) F-P29-OBS-001 and the recurrence chain, (b) the Arm-6 specification, (c) a pointer to sibling gate stories S-MAINT-RG-LIST-GATE-001 and S-MAINT-ADR-ANCHOR-GATE-001. Record the upstream issue URL in §Deliverables.

---

## Previous Story Intelligence

**S-MAINT-RG-LIST-GATE-001** — direct structural precedent. FB61 process-gap follow-up story, same gate-story class. Established the pattern: identify root cause → specify gate → add self-probe PASS/FAIL cases → upstream issue. Frontmatter field set and section ordering are the normative template for this story.

**S-MAINT-ADR-ANCHOR-GATE-001** — sibling gate story (FB62, same cascade). Second precedent in the same class. An adversary pass will compare all three sibling gate stories for structural consistency; their frontmatter and section skeletons MUST be uniform.

**S-MAINT-VOLATILE-CITE-001, S-MAINT-VOLATILE-CITE-002** — corpus sweep (de-reference) precedent stories. Established the pattern for sweeping pre-existing volatile cites and bumping artifact versions as part of a single state-manager burst.

**S-MAINT-POL29-HOOK-001** — older factory tooling story. Provides additional task-list shape precedent.

**F-P29-OBS-001, F-P21-OBS-001** — the originating and recurrence findings. The 3-recurrence rule (TD-VSDD-097) required codification. This story is the resulting structural intervention.

**TD-VSDD-091 amendment (2026-07-24)** — retired volatile-cite exceptions for ALL record-tier text. The five existing L9 arms did not cover the bare-hex-SHA class; this story adds Arm-6 to close that gap.

**TD-VSDD-092** — `scripts/records-lint.sh` is the mechanical enforcement gate for TD-VSDD-091. Arm-6 extends coverage without changing the gate's semantic contract (L9 = staged-addition line-cite ban; Arm-6 adds a new match class).

---

## Architecture Compliance Rules

1. **No prism crate modifications.** This story MUST NOT add, remove, or edit any file under `crates/`. Scope is strictly `scripts/records-lint.sh` and `.factory/specs/prd-supplements/`.
2. **No STATE.md edits.** STATE.md is state-manager territory.
3. **No STORY-INDEX.md edits.** Index registration is a state-manager burst, not this story's deliverable.
4. **TD-VSDD-053 single-commit-per-burst applies.** The `scripts/records-lint.sh` edit and the two corpus-sweep edits (T-08/T-09) may be in the same burst or separate bursts, but each burst MUST be a single atomic commit. No multi-commit chains with "Stage 1 / Stage 2" subjects.
5. **No CLAUDE.md edits.** The TD-VSDD-091/TD-VSDD-092 entries in CLAUDE.md already cover the SHA cite prohibition. No new rule is needed there.
6. **Ratchet model applies.** Arm-6 is staged-additions-only (L9 scope). Only the eight instances listed in AC-004 and AC-005 are in-scope for the corpus sweep. Other pre-existing SHA cites (if any are discovered during implementation) require a separate deferral story with an explicit story ID anchor — they MUST NOT be added to the tech-debt register without all three conditions in CLAUDE.md Canonical Principle Rule 3 being met.

---

## Library & Framework Requirements

No Rust library dependencies. Deliverable is bash additions to `scripts/records-lint.sh`. Bash with standard POSIX tools (`grep -E`, `git diff --cached`, `git -C`). No additional toolchain dependencies beyond those already used by the script. The script's existing `set -uo pipefail` environment is binding for the new code.

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `scripts/records-lint.sh` | Edit — add `_L9_ARM6` definition, update `L9_CITE_PATTERN`, add `input-hash:` exclusion in `run_l9`, add 4 Arm-6 `--self-probe` cases | AC-001, AC-002, AC-003 |
| `.factory/specs/prd-supplements/interface-definitions.md` | Edit — de-reference 3 SHA cites in changelog rows v2.11/v2.9/v2.8; bump version + add changelog row | AC-004 |
| `.factory/specs/prd-supplements/error-taxonomy.md` | Edit — de-reference 1 body-prose SHA cite + 4 changelog row SHA cites (v2.35/v1.75/v1.74/v1.14); bump version + add changelog row | AC-005 |
| `drbothen/vsdd-factory` GitHub issue | Create (conditional on T-11 evaluation) | URL recorded in §Deliverables |

---

## §Deliverables

| Item | Status | Reference |
|------|--------|-----------|
| `_L9_ARM6` defined and merged into `L9_CITE_PATTERN` | Pending | T-02/T-03 |
| `input-hash:` exclusion documented and implemented | Pending | T-04 |
| Arm-6 `--self-probe` cases (4 cases; +4 to verified-pass count) | Pending | T-06/T-07 |
| `interface-definitions.md` SHA de-reference (3 changelog rows) + version bump | Pending | T-08 |
| `error-taxonomy.md` SHA de-reference (1 body + 4 changelog rows) + version bump | Pending | T-09 |
| Supplemental arm or documented gap (bare `L<NNN>` coverage assessment at T-05) | Pending (conditional) | T-05 |
| Upstream issue URL (conditional) | Pending | T-11 |

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 0.1 | 2026-08-14 | story-writer | F-P29-OBS-001 process-gap follow-up: new story registered — extends `scripts/records-lint.sh` L9 with Arm-6 for bare git SHA detection (7–40 lowercase hex at word boundary); adds `input-hash:` exclusion (AC-002); adds 4 self-probe cases (AC-003); sweeps 8 pre-existing SHA cites in `interface-definitions.md` (AC-004) and `error-taxonomy.md` (AC-005); recurrence precedents F-P21-OBS-001 and DEFECT-ADAPTER-TLS-XDOME-LIVE-001 pass-21/pass-29/pass-30; status: draft; behavioral_contracts: [] pending PO authorship per S-7.01 |
