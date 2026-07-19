---
document_type: story
story_id: S-REL-005
title: "devops: RELEASING.md operator runbook + .factory/release-config.yaml"
wave: F-A
epic_id: E-REL
priority: P0
status: draft
version: "0.2"
level: "L4"
producer: story-writer
timestamp: "2026-07-19T00:00:00Z"
tdd_mode: strict
subsystems: []
# Subsystem anchor justification:
#   docs/RELEASING.md and .factory/release-config.yaml are release-process documentation
#   and configuration artifacts. No ARCH-INDEX subsystem owns operator runbooks or release
#   config schemas. subsystems: [] per S-0.01 infra story precedent.
crates_touched: []
target_module: devops
capabilities: []
behavioral_contracts: []
# BC status: N/A — operator runbook and release-config are process documentation artifacts.
# No subsystem behavioral contract governs release procedures. Conforming per W3-FIX-CI-001 precedent.
verification_properties: []
depends_on: [S-REL-001, S-REL-002, S-REL-003, S-REL-004, S-REL-007]
# Dependency anchor justifications:
#   depends_on S-REL-001: RELEASING.md documents the release workflow; the repaired
#     release.yml (S-REL-001) is the workflow being documented. The runbook cannot
#     accurately describe steps that don't exist yet.
#   depends_on S-REL-002: RELEASING.md documents the version bump procedure
#     (bump prism-bin Cargo.toml per ADR-053); S-REL-002 establishes that procedure.
#   depends_on S-REL-003: RELEASING.md documents the install scripts as the consumer
#     download path; S-REL-003 creates those scripts.
#   depends_on S-REL-004: RELEASING.md documents the demo bundle as a required release
#     artifact; S-REL-004 creates the demo bundle packaging.
#   depends_on S-REL-007: RELEASING.md documents Windows demo parity; S-REL-007 creates
#     the .ps1 scripts that make Windows demo possible.
blocks: []
points: 2
estimated_days: 1
risk: LOW
# Risk justification: Documentation and configuration artifact only. No Rust code changes.
# YAML schema for release-config is simple (schema 1). RELEASING.md cannot break anything.
acceptance_criteria_count: 7
red_gate_tests: 1
estimated_passes: "1 LOCAL adversary pass"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "release-config.yaml schema version: schema: 1 is the initial version; document
    the schema inline in the file so future maintainers can extend it."
  - "RELEASING.md must reference actual file paths, not aspirational ones. All paths
    documented (scripts/install.sh, scripts/install.ps1, scripts/demo-bundle.sh, etc.)
    must exist after S-REL-001 through S-REL-007 are merged."
  - "quality_gates is a MAP (U24/U25): the vsdd release skill schema requires quality_gates
    to be a mapping with mode: vsdd-full plus individual gate keys (require_convergence,
    min_convergence_dimensions, require_holdout, min_holdout_satisfaction,
    require_formal_verification, require_adversarial_passes, require_human_approval).
    Do NOT write quality_gates as a bare scalar (quality_gates: vsdd-full is INVALID schema)."
  - "packages/version_sources (U25): the schema requires a packages array with version_sources
    pointing to crates/prism-bin/Cargo.toml (format: toml) so the release tooling knows
    where to read the canonical product version."
  - "No invented top-level keys (U24): keys release_series, platforms, artifacts are NOT
    part of the vsdd release-config schema v1. Move them to YAML comments if human-readable
    reference is desired, but do not include them as schema keys."
  - "Hotfix deferral anchor (U27): the RELEASING.md Rollback/Hotfix section must reference
    the story anchor S-REL-hotfix-001 (not a bare TODO). Bare TODO violates Canonical
    Principle Rule 3 (tech-debt-register requires explicit anchor)."
inputs:
  - ".factory/planning/feature-release-engineering/delta-analysis.md"
  - ".github/workflows/release.yml"
  - ".factory/planning/feature-release-engineering/prism-consumer-contract.md"
  - ".factory/research/release-engineering-uncertainties-2026.md"
input-hash: "da7e7c2"
traces_to: []
cycle: "v1.0.0-release-engineering"
phase: "F3"
---

# S-REL-005 — devops: RELEASING.md operator runbook + .factory/release-config.yaml

**Story ID:** S-REL-005
**Status:** draft
**Version:** v0.2
**Wave:** F-A (terminal story — depends on all other F-A stories)
**Priority:** P0
**Points:** 2

---

## Origin

Delta-analysis §2 (missing artifacts): No `docs/RELEASING.md` exists. No `release-config.yaml`
exists. An operator preparing the 1.0.0-rc.1 release has no documented procedure. These
artifacts are required for the RC gate: human approval must be anchored to a written runbook,
and the factory pipeline must have a machine-readable quality gate config.

---

## Narrative

As a release engineer, I want a documented step-by-step release runbook and a machine-readable
release-config.yaml, so that I can execute the 1.0.0-rc.1 release with confidence and the
pipeline can enforce quality gates before tagging.

---

## Behavioral Contracts

This story has no subsystem BCs — operator runbook and release config are process artifacts.

| Architecture Source | Clause |
|--------------------|--------|
| `delta-analysis.md` §2 (missing artifacts) | RELEASING.md and release-config.yaml both missing |
| `delta-analysis.md` §11 S-REL-005 scope | docs/RELEASING.md + .factory/release-config.yaml |
| `delta-analysis.md` §9 (quality gates) | quality_gates MAP with mode: vsdd-full; require_human_approval: true |
| `prism-consumer-contract.md` §5.2 | Version string pinned to 1.0.0-rc.1; runbook must cite this |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~2,500 |
| `delta-analysis.md` §2, §9, §11 | ~2,000 |
| `.github/workflows/release.yml` (post S-REL-001) | ~3,500 |
| `prism-consumer-contract.md` §5 | ~600 |
| `release-engineering-uncertainties-2026.md` U24/U25/U27 | ~600 |
| Total | ~9,200 |

Well within the 30% context window budget.

---

## Tasks

1. **Read `.github/workflows/release.yml`** (post S-REL-001 repairs) to document the exact
   workflow steps in RELEASING.md.

2. **Read `delta-analysis.md` §9** for the quality gate list.

3. **Create `docs/RELEASING.md`** with the following structure:

   ```markdown
   # Releasing Prism

   ## Prerequisites
   - Cargo workspace builds cleanly: `just check`
   - All VSDD quality gates passed (see `.factory/release-config.yaml`)
   - Human release approval obtained (PR review + merge of version bump PR)

   ## Version bump (prism-bin only)
   Per ADR-053: bump `crates/prism-bin/Cargo.toml` version to match the release tag.
   All other crates remain unchanged.

   Steps:
   1. Create branch: `git checkout -b release/v<VERSION>`
   2. Edit `crates/prism-bin/Cargo.toml`: set `version = "<VERSION>"`
   3. Verify: `./target/release/prism --version` → `prism <VERSION>`
   4. Commit: `git commit -m "chore(release): bump prism-bin to <VERSION>"`
   5. Open PR; get approval; merge

   ## Tagging
   After the version bump PR merges to develop:
   1. `git tag v<VERSION> develop`
   2. `git push origin v<VERSION>`
   The release workflow triggers automatically on `v*` tags.

   ## What the release workflow does
   The `.github/workflows/release.yml` workflow:
   1. Builds prism binary for 5 platforms (aarch64-apple-darwin, x86_64-apple-darwin,
      x86_64-unknown-linux-gnu, x86_64-unknown-linux-musl, x86_64-pc-windows-msvc)
   2. Generates SHA-256 checksums for each binary
   3. Creates OIDC attestations (sigstore) for each binary
   4. Builds WASM plugins (crowdstrike-oauth2.prx, threatintel-lookup.prx) once on Linux
   5. Assembles per-platform demo bundles (.tar.gz for Unix, .zip for Windows)
   6. Uploads all artifacts to the GitHub Release

   ## Pre-release tags (v*-rc.*)
   RC tags trigger the same workflow. The `--prerelease` flag is set explicitly in the
   release job for tags matching `v*-rc.*`. All 5 platform binaries and demo bundles
   are published.

   ## Consumer install paths
   - Unix: `curl -fsSL https://github.com/…/releases/download/v<VERSION>/install.sh | bash`
   - Windows: `irm https://…/install.ps1 | iex`
   - Direct download: binary + demo bundle from GitHub Release page
   See `scripts/install.sh` and `scripts/install.ps1` for checksum verification logic.

   ## Demo bundle
   Each release includes per-platform demo bundles: `.tar.gz` for Unix platforms,
   `.zip` for Windows. The bundle includes the DTU demo server, sensor specs, pre-built
   plugins, and demo scripts (bash for Unix, PowerShell for Windows).
   See `docs/DEMO-RUNBOOK.md`.

   ## Rollback
   GitHub Releases supports yanking/deleting a release. To roll back:
   1. Delete the GitHub Release (keeps the tag)
   2. Notify consumers via release notes of the issue
   3. Delete the tag: `git push --delete origin v<VERSION>` (requires explicit approval)

   ## Hotfix releases
   Hotfix release procedure (branching strategy, version bump protocol, emergency release
   checklist) is deferred to story S-REL-hotfix-001. See that story for the full procedure.
   Concrete dependency: hotfix tooling requires the base release workflow (S-REL-001 through
   S-REL-005) to exist first; S-REL-hotfix-001 builds on top of this foundation.
   ```

4. **Create `.factory/release-config.yaml`** with the correct vsdd release-config schema
   (U24/U25: quality_gates is a MAP, not a scalar; includes packages/version_sources):

   ```yaml
   # Prism release pipeline configuration
   # Schema version 1 — see schema notes below for extension guidance
   schema: 1

   packages:
     - name: prism
       version_sources:
         - path: crates/prism-bin/Cargo.toml
           format: toml

   quality_gates:
     mode: vsdd-full
     require_convergence: true
     min_convergence_dimensions: 7
     require_holdout: true
     min_holdout_satisfaction: 1.0
     require_formal_verification: true
     require_adversarial_passes: 3
     require_human_approval: true

   # Informational reference (NOT schema keys — for human readability only):
   # platforms: [aarch64-apple-darwin, x86_64-apple-darwin, x86_64-unknown-linux-gnu,
   #             x86_64-unknown-linux-musl, x86_64-pc-windows-msvc]
   # release_series: "1.0.0-rc"
   # artifacts: [binary+checksum+attestation, demo-bundle]
   ```

---

## Acceptance Criteria

### AC-001: `docs/RELEASING.md` exists with required sections
Given: The story is implemented.
When: `ls docs/RELEASING.md` is run.
Then: File exists. It contains all required sections: Prerequisites, Version bump, Tagging,
What the release workflow does, Pre-release tags, Consumer install paths, Demo bundle,
Rollback, Hotfix releases (with S-REL-hotfix-001 anchor per U27 — no bare TODO).
(traces to delta-analysis.md §2: "docs/RELEASING.md is missing")

### AC-002: RELEASING.md cites ADR-053 for the version bump procedure
Given: `docs/RELEASING.md` is read.
When: `grep 'ADR-053' docs/RELEASING.md` is run.
Then: At least one reference to ADR-053 is present, citing the policy that only prism-bin
version changes per release.
(traces to delta-analysis.md §2: "ADR-053 policy documented in runbook")

### AC-003: RELEASING.md documents both Unix and Windows install paths
Given: `docs/RELEASING.md` is read.
When: `grep -i 'install.sh\|install.ps1' docs/RELEASING.md` is run.
Then: Both `scripts/install.sh` (Unix) and `scripts/install.ps1` (Windows) are referenced
in the Consumer install paths section.
(traces to delta-analysis.md §11 S-REL-005: "consumer install paths documented")

### AC-004: `.factory/release-config.yaml` exists with schema 1
Given: The story is implemented.
When: `ls .factory/release-config.yaml` is run.
Then: File exists. `grep 'schema: 1' .factory/release-config.yaml` returns one match.
(traces to delta-analysis.md §11 S-REL-005: ".factory/release-config.yaml (schema 1)")

### AC-005: release-config.yaml has quality_gates as a MAP with mode: vsdd-full
Given: `.factory/release-config.yaml` is read.
When: The quality_gates block is inspected.
Then: `quality_gates:` is a YAML mapping (not a scalar). The mapping contains
`mode: vsdd-full`. The mapping also contains `require_convergence: true`,
`min_convergence_dimensions: 7`, `require_holdout: true`, `min_holdout_satisfaction: 1.0`,
`require_formal_verification: true`, `require_adversarial_passes: 3`.
The top-level scalar `quality_gates: vsdd-full` pattern is NOT present.
(traces to delta-analysis.md §9: "quality_gates: vsdd-full"; research U24/U25: MAP schema)

### AC-006: release-config.yaml has require_human_approval: true inside quality_gates
Given: `.factory/release-config.yaml` is read.
When: `grep 'require_human_approval: true' .factory/release-config.yaml` is run.
Then: Exactly one match, indented inside the quality_gates block.
(traces to delta-analysis.md §9: "require_human_approval: true")

### AC-007: release-config.yaml has packages/version_sources pointing to prism-bin
Given: `.factory/release-config.yaml` is read.
When: The packages block is inspected.
Then: A `packages:` array exists with one entry for `name: prism`. That entry has
`version_sources:` pointing to `path: crates/prism-bin/Cargo.toml` with `format: toml`.
No invented top-level keys (`release_series:`, `platforms:`, `artifacts:`) appear as
schema keys — they may appear only as YAML comments.
(traces to delta-analysis.md §11 S-REL-005; research U25: packages/version_sources)

---

## Previous Story Intelligence

This is the terminal F-A story — it depends on all other F-A stories. The implementer
should write RELEASING.md AFTER reading the actual post-S-REL-001 release.yml to describe
what the workflow actually does, not what this story says it does (in case implementation
details differ from spec).

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| version bump = prism-bin only per ADR-053 | S-REL-002 + ADR-053 | AC-002 |
| quality_gates is a MAP (not scalar) | Research U24/U25 | AC-005 no scalar form |
| packages/version_sources in release-config | Research U25 | AC-007 |
| No invented top-level schema keys | Research U24 | AC-007 comment-only check |
| require_human_approval: true (inside quality_gates) | delta-analysis §9 | AC-006 |
| Hotfix deferral uses story anchor S-REL-hotfix-001 | Research U27; Canonical Principle Rule 3 | AC-001 no bare TODO |

---

## Library & Framework Requirements

No code dependencies — documentation and YAML only.

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `docs/RELEASING.md` | Create | Operator release runbook; hotfix deferred to S-REL-hotfix-001 |
| `.factory/release-config.yaml` | Create | Machine-readable release quality gate config; schema 1 MAP format |

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `docs/RELEASING.md` | `docs/` | N/A (documentation) |
| `.factory/release-config.yaml` | `.factory/` | N/A (configuration) |

---

## Purity Classification

| Module | Classification | Justification |
|--------|----------------|---------------|
| `docs/RELEASING.md` | N/A | Documentation — no Rust purity boundary applies |
| `.factory/release-config.yaml` | N/A | YAML configuration — no Rust purity boundary applies |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Future schema extension (schema: 2) | Add schema version migration note to release-config.yaml comments |
| EC-002 | Release from main vs develop | RELEASING.md clarifies: tag targets develop; main is updated by PR merge |
| EC-003 | Hotfix release path | Deferred to story S-REL-hotfix-001 with explicit anchor in RELEASING.md Hotfix section; Canonical Principle Rule 3 compliant (anchor + concrete future dependency stated) |

---

## Forbidden Dependencies

- No Rust code changes (documentation story only)
- No changes to existing CI/CD files (those are owned by S-REL-001 and S-REL-004)
- No `quality_gates: vsdd-full` scalar form in release-config.yaml (must be MAP)
- No bare TODO in RELEASING.md hotfix section (must reference S-REL-hotfix-001)

---

## Changelog

| Version | Date | Summary |
|---------|------|---------|
| 0.2 | 2026-07-19 | Fix-burst: U24/U25 release-config.yaml quality_gates rewritten as MAP (mode+individual gate keys); packages/version_sources added; invented top-level keys (release_series/platforms/artifacts) moved to comments only; AC-005/007 updated to verify MAP structure; U27 EC-003 and RELEASING.md Hotfix section use S-REL-hotfix-001 story anchor instead of bare TODO; research file added to inputs |
| 0.1 | 2026-07-19 | Initial story creation (story-writer F3 burst) |
