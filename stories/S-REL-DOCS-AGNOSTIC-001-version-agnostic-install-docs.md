---
document_type: story
story_id: S-REL-DOCS-AGNOSTIC-001
title: "docs: make SETUP.md + install scripts version-agnostic; update RELEASING.md §1 for pre-release exception"
wave: F-A
epic_id: E-REL-IDENTITY
priority: P0
status: draft
version: "1.0"
level: "L4"
producer: story-writer
timestamp: "2026-09-05T00:00:00Z"
tdd_mode: facade
# tdd_mode: facade — this story is docs/scripts only; no Rust code.
# No Red Gate tests needed.
subsystems: [SS-22]
# Subsystem anchor justification:
#   SS-22 (Process Lifecycle) owns operator-facing release and install documentation.
crates_touched: []
target_module: devops
capabilities: []
behavioral_contracts: []
# BC status: N/A — docs/install scripts; no subsystem behavioral contract.
verification_properties: []
depends_on: [S-REL-DEV-RESET-001]
blocks: []
# Dependency anchor justification:
#   depends_on S-REL-DEV-RESET-001: RELEASING.md §1 pre-release exception documents
#     that develop carries 1.0.0-dev (a decision made concrete by DEV-RESET). Must be
#     authored after the 1.0.0-dev reset is specced so the text is accurate.
points: 3
estimated_days: 1
risk: LOW
acceptance_criteria_count: 6
red_gate_tests: 0
# red_gate_tests: 0 — facade mode. No Rust code.
estimated_passes: 1
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "OPERATOR-DECIDED: 'download the newest PRE-RELEASE from the GitHub Releases page'
    instruction (NOT /releases/latest/download/ which excludes pre-releases). This is
    a human-directed approach; do not substitute latest-release URLs."
  - "Grep verification: after all edits, grep SETUP.md + install scripts for any
    remaining hardcoded version strings (v1.0.0-rc.1, v1.0.0-beta.1, etc.) as an
    AC gate — AC-006."
  - "install.sh and install.ps1 USAGE comment examples that include version strings
    must be updated to use a placeholder like '<tag>' or
    'latest pre-release tag from Releases page'."
inputs:
  - "docs/SETUP.md"
  - "scripts/install.sh"
  - "scripts/install.ps1"
  - "RELEASING.md"
  - ".factory/specs/architecture/decisions/ADR-064-pre-release-binary-version-identity.md"
input-hash: "[pending-recompute]"
traces_to: []
cycle: "v1.0.0-beta.1-release-identity"
phase: "3"
---

# S-REL-DOCS-AGNOSTIC-001 — Version-Agnostic Install Docs + RELEASING.md Pre-Release Exception

**Story ID:** S-REL-DOCS-AGNOSTIC-001
**Status:** draft
**Version:** v1.0
**Wave:** F-A
**Priority:** P0
**Points:** 3
**Beta.1-blocking:** YES — install docs must not contain v1.0.0-rc.1 strings before
the beta.1 release PR is merged.

---

## Origin

`docs/SETUP.md`, `scripts/install.sh`, and `scripts/install.ps1` contain approximately
20 hardcoded references to `v1.0.0-rc.1`. After the version is reset to `1.0.0-dev`
(S-REL-DEV-RESET-001) and we begin shipping pre-releases (beta.1), any hardcoded
version string in install docs is immediately stale.

ADR-064 D2 establishes that `develop` carries `1.0.0-dev` permanently between
pre-releases; there is no Cargo.toml bump between pre-releases. `RELEASING.md §1`
must document this pre-release exception so future release engineers understand they
do NOT need to bump `prism-bin/Cargo.toml` between pre-releases.

Operator-decided approach (human-approved): SETUP.md and install scripts must tell
operators to "download the newest pre-release from the GitHub Releases page" — NOT
use `/releases/latest/download/` URLs (which GitHub resolves to the most recent
_stable_ release, excluding pre-releases).

---

## Narrative

As a prism operator who wants to install the beta release, I want the SETUP.md guide
to give me accurate, version-independent install instructions, so that I do not follow
stale instructions that point to a non-existent version string or the wrong release.

As a release engineer, I want RELEASING.md §1 to document the pre-release exception
(develop carries `1.0.0-dev`; no Cargo.toml bump between pre-releases), so that future
engineers understand the correct release process and do not accidentally bump the
crate version when cutting a new beta.

---

## Authority

- ADR-064 D1 — prism-bin Cargo.toml reset to 1.0.0-dev
- ADR-064 D2 — `PRISM_VERSION` via build.rs; develop carries 1.0.0-dev permanently
- Human-approved operator decision: "download newest pre-release from GitHub Releases
  page" instruction (NOT /releases/latest/download/)

(No BC: documentation/install scripts; no subsystem behavioral contract.)

---

## Behavioral Contracts

This story has no subsystem behavioral contracts. Authority is ADR-064 D1/D2.

| Architecture Source | Clause |
|---------------------|--------|
| ADR-064 D1 | Cargo.toml is reset to 1.0.0-dev; docs must not contradict this |
| ADR-064 D2 | develop carries 1.0.0-dev between pre-releases; no Cargo.toml bump for pre-releases |
| Operator decision | Install instructions use GitHub Releases page (pre-releases visible); NOT /latest/ |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~2,500 |
| `docs/SETUP.md` | ~4,000 |
| `scripts/install.sh` | ~1,500 |
| `scripts/install.ps1` | ~1,500 |
| `RELEASING.md` | ~3,000 |
| ADR-064 D2 section | ~800 |
| Total | ~13,300 |

Well within the 30% context window budget.

---

## Red Gate Test List (SAC-1)

**tdd_mode: facade — no Rust Red Gate tests.** Verification is grep-based:
- AC-006 requires `grep -r 'v1\.0\.0-rc\.' docs/ scripts/RELEASING.md` returns 0 matches
  after all edits are complete.

---

## Tasks

1. **Read `docs/SETUP.md` in full.** Identify every hardcoded version string:
   - `v1.0.0-rc.1` or `1.0.0-rc.1` in download URLs
   - `v1.0.0-rc.1` or `1.0.0-rc.1` in example commands
   - Any `/releases/latest/download/` URL that assumes stable releases only

2. **Update `docs/SETUP.md`:**
   - Replace every hardcoded version string with version-agnostic instructions.
   - For download URLs: replace hardcoded `v1.0.0-rc.1` URL examples with prose
     directing the operator to the GitHub Releases page:
     > Download the newest pre-release from the
     > [GitHub Releases page](https://github.com/<owner>/prism/releases).
     > Pre-releases appear in the list but are not returned by
     > `/releases/latest/download/`.
   - For any `curl` or `wget` examples with hardcoded tags, replace with a generic
     `PRISM_TAG=<tag-from-releases-page>` variable pattern.
   - Preserve all other content; do not rewrite prose that doesn't contain version
     strings.

3. **Read `scripts/install.sh` in full.** Identify:
   - Hardcoded version strings in USAGE comments or default values
   - Any `/releases/latest/download/` URLs (which exclude pre-releases)

4. **Update `scripts/install.sh`:**
   - Replace USAGE comment examples: replace hardcoded version string with
     `<tag>` placeholder (e.g., `v1.0.0-beta.1`).
   - Replace any `/releases/latest/download/` URL pattern with the Releases page
     URL or a variable-based `github.com/.../<tag>/prism-...` pattern.
   - Preserve all logic and flag handling.

5. **Read `scripts/install.ps1` in full.** Same analysis as install.sh.

6. **Update `scripts/install.ps1`:**
   - Same changes as install.sh but for PowerShell syntax.

7. **Read `RELEASING.md §1` (Pre-release and stable release section).**

8. **Update `RELEASING.md §1` — Pre-release exception paragraph:**
   Add (or update) a paragraph documenting:
   > **Pre-release exception (ADR-064 D2):** Between pre-release tags (e.g.,
   > `v1.0.0-beta.1`, `v1.0.0-beta.2`), the `develop` branch carries
   > `version = "1.0.0-dev"` in `prism-bin/Cargo.toml`. Do NOT bump this to the
   > next pre-release semver string. The pre-release version is set at CI time via
   > the `GITHUB_REF_NAME` environment variable injected by `build.rs`
   > (see `S-REL-BVERSION-INJECT-001` / ADR-064 D2). The `Cargo.toml` version is
   > only bumped when cutting a stable release (`v1.0.0` or later), which is handled
   > by `S-REL-VBUMP-001`.

9. **Run verification grep (AC-006):**
   ```bash
   grep -r 'v1\.0\.0-rc\.' docs/ scripts/ RELEASING.md
   ```
   Expected: zero matches. If any remain, fix them before declaring done.

10. **Verify AC-001 through AC-006.**

---

## Acceptance Criteria

### AC-001: SETUP.md contains no hardcoded v1.0.0-rc.1 strings
`grep 'v1\.0\.0-rc\.' docs/SETUP.md` returns 0 matches.
(traces to ADR-064 D1 — Cargo.toml reset to 1.0.0-dev; docs must not contradict)

### AC-002: SETUP.md does not contain /releases/latest/download/ URL
`grep '/releases/latest/download/' docs/SETUP.md` returns 0 matches.
The `/latest/` endpoint returns the most recent stable release and excludes
pre-releases, which would be wrong during the beta phase.
(traces to operator-approved decision — pre-releases must be accessible)

### AC-003: SETUP.md contains a reference to GitHub Releases page for downloads
`grep -i 'releases' docs/SETUP.md` returns at least one match directing operators
to the GitHub Releases page.
(traces to operator-approved decision — correct install path for pre-releases)

### AC-004: install.sh and install.ps1 contain no hardcoded v1.0.0-rc.1 strings
`grep 'v1\.0\.0-rc\.' scripts/install.sh scripts/install.ps1` returns 0 matches.
(traces to ADR-064 D1 — version strings must not be hardcoded)

### AC-005: RELEASING.md §1 documents the pre-release exception
`grep -A5 'Pre-release exception' RELEASING.md` returns the ADR-064 D2 paragraph
explaining that `develop` carries `1.0.0-dev` and that no Cargo.toml bump is needed
between pre-releases.
(traces to ADR-064 D2 — develop carries 1.0.0-dev; pre-release path does not bump
Cargo.toml)

### AC-006: Full sweep — no hardcoded release-version strings in docs/ or scripts/
```bash
grep -r 'v1\.0\.0-rc\.' docs/ scripts/ RELEASING.md
```
Returns 0 matches. This is the final gate; it must pass after Tasks 1-9.
(traces to ADR-064 D1 — version-agnostic docs are required before beta.1 release)

---

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| S-REL-DEV-RESET-001 | Cargo.toml version becomes 1.0.0-dev | Foundation for the pre-release exception narrative in RELEASING.md | Lockfiles also need updating — not in scope here |
| S-REL-002 (merged) | Version alignment work; RELEASING.md already exists | RELEASING.md exists and has a §1 section | Check existing §1 structure before inserting to avoid duplicate content |

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| No /releases/latest/download/ in install docs | Operator decision | AC-002 grep check |
| GitHub Releases page for pre-release downloads | Operator decision | AC-003 grep check |
| Pre-release exception documented in RELEASING.md §1 | ADR-064 D2 | AC-005 grep check |
| develop carries 1.0.0-dev; no Cargo.toml bump for pre-releases | ADR-064 D2 | AC-005 prose check |

---

## Library & Framework Requirements

No new dependencies. This story modifies documentation and shell scripts only.

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `docs/SETUP.md` | Modify | Remove ~20 hardcoded v1.0.0-rc.1 strings; add Releases page download instructions |
| `scripts/install.sh` | Modify | Replace hardcoded version strings in USAGE comments and URL patterns |
| `scripts/install.ps1` | Modify | Same as install.sh, PowerShell syntax |
| `RELEASING.md` | Modify | Add pre-release exception paragraph to §1 |

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `docs/SETUP.md` | docs | Pure (text document; no runtime behavior) |
| `scripts/install.sh` | scripts | Effectful (shell script; downloads binary from GitHub Releases) |
| `scripts/install.ps1` | scripts | Effectful (PowerShell; downloads binary from GitHub Releases) |
| `RELEASING.md` | repo root | Pure (process documentation) |

---

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `docs/SETUP.md` | pure-core | Static markdown documentation |
| `scripts/install.sh` | effectful-shell | Downloads and installs binary at runtime |
| `scripts/install.ps1` | effectful-shell | Downloads and installs binary at runtime |
| `RELEASING.md` | pure-core | Static process documentation |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | SETUP.md contains a version string in a code-block example that must remain as a concrete example | Replace with a `PRISM_TAG=<tag>` variable pattern rather than a literal version; never leave v1.0.0-rc.1 |
| EC-002 | install.sh or install.ps1 uses the version string as a default variable value | Replace default with empty string and add a required-arg check, or replace with a "check releases page" instruction |
| EC-003 | RELEASING.md §1 already contains a pre-release paragraph that conflicts | Read §1 first; merge the ADR-064 D2 clause into the existing paragraph rather than duplicating |
| EC-004 | AC-006 grep still shows v1.0.0-rc.1 after all edits | Investigate grep output, find missed occurrence, fix and re-verify |
| EC-005 | install.sh has both a USAGE comment version string AND an inline default | Both must be updated; grep must return 0 before task complete |

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-09-05 | story-writer | Initial — version-agnostic install docs + RELEASING.md pre-release exception |
