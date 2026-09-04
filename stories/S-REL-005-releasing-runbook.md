---
document_type: story
story_id: S-REL-005
title: "devops: RELEASING.md operator runbook + .factory/release-config.yaml"
wave: F-A
epic_id: E-REL
priority: P0
status: draft
version: "0.4"
level: "L4"
producer: story-writer
timestamp: "2026-07-19T00:00:00Z"
tdd_mode: strict
subsystems: []
# Subsystem anchor justification:
#   RELEASING.md and .factory/release-config.yaml are release-process documentation
#   and configuration artifacts. No ARCH-INDEX subsystem owns operator runbooks or release
#   config schemas. subsystems: [] per S-0.01 infra story precedent.
crates_touched: [prism-bin]
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
#     (bump prism-bin Cargo.toml per ADR-062); S-REL-002 establishes that procedure.
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
acceptance_criteria_count: 13
red_gate_tests: 2
estimated_passes: "1 LOCAL adversary pass"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "release-config.yaml schema version: schema: 1 is the initial version; document
    the schema inline in the file so future maintainers can extend it."
  - "RELEASING.md must reference actual file paths, not aspirational ones. All paths
    documented (scripts/install.sh, scripts/install.ps1, scripts/demo-bundle.sh, etc.)
    must exist after S-REL-001 through S-REL-007 are merged."
  - "quality_gates is a MAP with mode: vsdd-partial (U24/U25, D-2440): the vsdd release skill
    schema requires quality_gates to be a mapping with mode: vsdd-partial plus individual gate
    keys (require_convergence, min_convergence_dimensions, require_holdout,
    min_holdout_satisfaction, require_formal_verification, require_adversarial_passes,
    require_human_approval). Do NOT write quality_gates as a bare scalar.
    require_human_approval: true is load-bearing regardless of mode.
    Note: the original delta-analysis §9 spec said 'mode: vsdd-full' — this was a doc error;
    the correct value for a v1.0.0-rc.1 release gate is vsdd-partial (D-2440)."
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
input-hash: "92db479"
traces_to: []
cycle: "v1.0.0-release-engineering"
phase: "F3"
---

# S-REL-005 — devops: RELEASING.md operator runbook + .factory/release-config.yaml

**Story ID:** S-REL-005
**Status:** draft
**Version:** v0.4
**Wave:** F-A (terminal story — depends on all other F-A stories)
**Priority:** P0
**Points:** 2

---

## Origin

Delta-analysis §2 (missing artifacts): No `RELEASING.md` exists. No `release-config.yaml`
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
| `delta-analysis.md` §11 S-REL-005 scope | RELEASING.md + .factory/release-config.yaml |
| `delta-analysis.md` §9 (quality gates) | quality_gates MAP with mode: vsdd-partial; require_human_approval: true (load-bearing; prior vsdd-full was a doc error per D-2440) |
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

3. **Create `RELEASING.md`** at the repo root (NOT under `docs/`) with the following structure:

   ```markdown
   # Releasing Prism

   ## Prerequisites
   - Cargo workspace builds cleanly: `just check`
   - All VSDD quality gates passed (see `.factory/release-config.yaml`)
   - Human release approval obtained (PR review + merge of version bump PR)

   ## Version bump (prism-bin only)
   Per ADR-062: bump `crates/prism-bin/Cargo.toml` version to match the release tag.
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

   ## Tag-naming discipline
   The release workflow triggers on any `v*` tag. Only tags matching
   `vMAJOR.MINOR.PATCH` (GA) or `vMAJOR.MINOR.PATCH-rc.N` (prerelease) may be
   pushed to origin. Non-release tags that begin with `v` (e.g., `vendor-sync`)
   MUST NOT be pushed to origin — they would trigger the release pipeline and,
   containing `-`, publish as a prerelease.

   **Workflow trigger scope decision (F-REL001-PR6-003):** [Implementer must record
   the decision here — see Task 5. Do not leave this as a bare TODO.]

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
     mode: vsdd-partial    # vsdd-full was a doc error; vsdd-partial is the correct rc.1 gate (D-2440)
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

5. **Evaluate and document release.yml trigger scope (F-REL001-PR6-003):**
   The current `on: push: tags: v*` pattern triggers the release pipeline for any tag
   beginning with `v` — including hypothetical non-release tags such as `vendor-sync`.
   A tag containing `-` would be flagged as a prerelease. Evaluate whether to tighten
   the trigger to a semver-shaped glob (e.g., `v[0-9]+.[0-9]+.[0-9]+*`):
   - **If tightening:** update `.github/workflows/release.yml` `on.push.tags` accordingly,
     verify the existing test tag `v0.0.1-rc.test` still matches the new pattern, and
     record the change in RELEASING.md §Tag-naming discipline with rationale.
   - **If keeping `v*`:** record the rationale in RELEASING.md §Tag-naming discipline —
     e.g., "GitHub tags-filter does not support full regex; tag-naming discipline is the
     compensating control." The decision must be explicit text, not a placeholder.
   Either path: the decision is written into RELEASING.md before the story is closed
   (Canonical Principle Rule 3 — no bare TODO). AC-008 verifies this.

6. **Fix boot.rs relative path resolution for spec_dir and plugin_dir:**
   Read `crates/prism-bin/src/boot.rs`. Find the code that resolves `spec_dir` and
   `plugin_dir` from the config. If the current code resolves them relative to the process
   CWD (e.g., via `Path::new(spec_dir)` without joining to config_dir), change it to
   resolve relative to the config file's parent directory:
   ```rust
   // Instead of: PathBuf::from(&config.spec_dir)
   // Use: config_dir.join(&config.spec_dir)
   ```
   where `config_dir` = the parent directory of the prism.toml file that was loaded.
   Write a failing Red Gate test (RG-002) BEFORE making the code change: a test that starts
   prism with a relative `spec_dir` from a working directory DIFFERENT from config_dir and
   asserts that spec files are found (not that CWD/spec_dir is attempted).

7. **Create `prism.toml.example`** at the repo root with the following constraints:
   - UUID-v7 org_id guidance: document `uuidgen -7` (util-linux 2.41+) OR
     `python3 -c "import uuid; print(uuid.uuid7())"` (Python 3.14+ stdlib).
     Do NOT use `python3 -c "import uuid7; ..."` (third-party package — not stdlib).
   - credential_backend: document only `keyring` and `encrypted_file` types.
     Do NOT list `env` (nonexistent backend type).
   - Sensor configuration examples: Claroty xDome ONLY. Remove all crowdstrike, cyberint,
     armis sensor blocks, crowdstrike-oauth2.prx plugin_dir references.
     Keep infusions/threatintel examples.
   - spec_dir: document as a path that is resolved relative to the directory containing
     prism.toml (config_dir), not the process CWD. Add a comment to this effect.

---

## Acceptance Criteria

### AC-001: `RELEASING.md` exists at repo root with required sections
Given: The story is implemented.
When: `ls RELEASING.md` is run from the repo root (NOT `ls docs/RELEASING.md`).
Then: File exists at the repo root. It contains all required sections: Prerequisites,
Version bump, Tagging, What the release workflow does, Pre-release tags, Consumer install
paths, Demo bundle, Rollback, Hotfix releases (with S-REL-hotfix-001 anchor per U27 — no
bare TODO). Note: the file is at REPO_ROOT/RELEASING.md, not docs/RELEASING.md (D-2440).
(traces to delta-analysis.md §2: "RELEASING.md is missing"; D-2440: repo-root location)

### AC-002: RELEASING.md cites ADR-062 for the version bump procedure
Given: `RELEASING.md` is read.
When: `grep 'ADR-062' RELEASING.md` is run.
Then: At least one reference to ADR-062 is present, citing the policy that only prism-bin
version changes per release.
(traces to delta-analysis.md §2: "version policy documented in runbook"; D-2440 ADR renumber 053→062)

### AC-003: RELEASING.md documents both Unix and Windows install paths
Given: `RELEASING.md` is read.
When: `grep -i 'install.sh\|install.ps1' RELEASING.md` is run.
Then: Both `scripts/install.sh` (Unix) and `scripts/install.ps1` (Windows) are referenced
in the Consumer install paths section.
(traces to delta-analysis.md §11 S-REL-005: "consumer install paths documented")

### AC-004: `.factory/release-config.yaml` exists with schema 1
Given: The story is implemented.
When: `ls .factory/release-config.yaml` is run.
Then: File exists. `grep 'schema: 1' .factory/release-config.yaml` returns one match.
(traces to delta-analysis.md §11 S-REL-005: ".factory/release-config.yaml (schema 1)")

### AC-005: release-config.yaml has quality_gates as a MAP with mode: vsdd-partial
Given: `.factory/release-config.yaml` is read.
When: The quality_gates block is inspected.
Then: `quality_gates:` is a YAML mapping (not a scalar). The mapping contains
`mode: vsdd-partial` (NOT vsdd-full; the original delta-analysis §9 said vsdd-full but
that was a doc error — the correct mode for v1.0.0-rc.1 is vsdd-partial per D-2440).
The mapping also contains `require_convergence: true`, `min_convergence_dimensions: 7`,
`require_holdout: true`, `min_holdout_satisfaction: 1.0`, `require_formal_verification: true`,
`require_adversarial_passes: 3`.
The top-level scalar `quality_gates: vsdd-full` pattern is NOT present.
(traces to delta-analysis.md §9: "quality_gates MAP"; research U24/U25: MAP schema;
D-2440: mode: vsdd-partial correction)

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

### AC-008: RELEASING.md documents tag-naming discipline and records trigger-scope decision
Given: `RELEASING.md` is read.
When: The "Tag-naming discipline" section is inspected.
Then: The section (a) states that only `vMAJOR.MINOR.PATCH` (GA) and
`vMAJOR.MINOR.PATCH-rc.N` (prerelease) tags may be pushed to origin; (b) states that
non-release `v*` tags MUST NOT be pushed; (c) contains the trigger-scope evaluation
decision — either a rationale for keeping `v*` or a reference to a tightened pattern
with the updated workflow — as explicit prose (not a placeholder, not a bare TODO).
(traces to F-REL001-PR6-003: release.yml v* trigger matches any v-prefixed tag;
 a non-release v* tag with `-` would publish as an unintended prerelease)

### AC-009: boot.rs resolves relative spec_dir and plugin_dir against config_dir, not process CWD
Given: A `prism.toml` with `spec_dir = "specs"` (relative path) at `/etc/prism/prism.toml`.
When: `prism start --config /etc/prism/prism.toml` is run from a working directory other
  than `/etc/prism` (e.g., `cd /tmp && prism start --config /etc/prism/prism.toml`).
Then: prism resolves `spec_dir` as `/etc/prism/specs` (joining config_dir + spec_dir),
  NOT as `/tmp/specs` (process CWD + spec_dir). Boot succeeds; spec files are found.
  The Red Gate test (RG-002): starts prism from a CWD different from config_dir with a
  relative spec_dir, asserts the specs are found (not a CWD-relative path error).
(traces to boot.rs CWD bug: D-2440 — spec_dir/plugin_dir must use config_dir.join())

### AC-010: prism.toml.example UUID-v7 org_id guidance uses accurate stdlib commands
Given: `prism.toml.example` is read.
When: The org_id / UUID-v7 guidance comment or example is inspected.
Then: The documented commands are `uuidgen -7` / `uuidgen --time-v7` (util-linux 2.41+) OR
  `python3 -c "import uuid; print(uuid.uuid7())"` (Python 3.14+ stdlib).
  The broken `python3 -c "import uuid7; ..."` pattern is NOT present (uuid7 is a third-party
  package, not Python stdlib).
(traces to D-2440: accurate UUID-v7 guidance in prism.toml.example)

### AC-011: prism.toml.example credential_backend lists only supported types
Given: `prism.toml.example` is read.
When: Any `credential_backend` type values or comments are inspected.
Then: Only `keyring` and `encrypted_file` are listed as valid credential_backend types.
  The nonexistent `env` type is NOT present.
(traces to D-2440: env credential backend does not exist; only keyring + encrypted_file
  are implemented in prism-bin)

### AC-012: prism.toml.example sensor examples reference only Claroty xDome
Given: `prism.toml.example` is read.
When: All sensor configuration sections and comments are inspected.
Then: Only Claroty xDome sensor examples appear. References to `crowdstrike`, `cyberint`,
  `armis`, `crowdstrike-oauth2.prx`, and plugin_dir configurations for CrowdStrike are
  absent. Infusion configuration examples (`threatintel`) may be present.
(traces to D-2440 sensor-scope: v1.0.0-rc.1 ships Claroty xDome only;
  CrowdStrike returns via S-ADR054-WAVE-A-001)

### AC-013: prism.toml.example spec_dir guidance reflects config_dir-relative resolution
Given: `prism.toml.example` is read (after boot.rs fix from AC-009).
When: The spec_dir configuration example/comment is inspected.
Then: The example documents that a relative spec_dir value is resolved relative to the
  directory containing prism.toml (config_dir), not relative to the process CWD. A comment
  or note to this effect is present alongside the spec_dir example value.
(traces to AC-009: boot.rs fix; D-2440: spec_dir guidance must be accurate once fix lands)

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
| version bump = prism-bin only per ADR-062 | S-REL-002 + ADR-062 | AC-002 |
| quality_gates is a MAP with mode: vsdd-partial (not scalar, not vsdd-full) | Research U24/U25; D-2440 | AC-005 |
| packages/version_sources in release-config | Research U25 | AC-007 |
| No invented top-level schema keys | Research U24 | AC-007 comment-only check |
| require_human_approval: true (inside quality_gates) | delta-analysis §9 | AC-006 |
| Hotfix deferral uses story anchor S-REL-hotfix-001 | Research U27; Canonical Principle Rule 3 | AC-001 no bare TODO |
| spec_dir/plugin_dir resolved vs config_dir, not CWD | D-2440 boot.rs CWD bug | AC-009 Red Gate test |
| prism.toml.example: UUID-v7 stdlib only, keyring+encrypted_file, Claroty-only | D-2440 | AC-010/011/012 |

---

## Library & Framework Requirements

| Dependency | Version | Notes |
|------------|---------|-------|
| Rust toolchain | Per `rust-toolchain.toml` | For boot.rs fix only; no new crate deps |

The boot.rs path-resolution fix uses only existing `std::path::Path::join()` — no new crate dependencies.

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `RELEASING.md` | Create | Operator release runbook at repo root (NOT docs/); hotfix deferred to S-REL-hotfix-001 |
| `.factory/release-config.yaml` | Create | Machine-readable release quality gate config; schema 1 MAP format; mode: vsdd-partial |
| `crates/prism-bin/src/boot.rs` | Modify | Fix spec_dir/plugin_dir relative-path resolution — use config_dir.join() not process CWD |
| `prism.toml.example` | Create | Config template: UUID-v7 guidance (uuidgen -7 / Python stdlib), keyring+encrypted_file only, Claroty-only sensor examples, accurate spec_dir relative-path note |

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `RELEASING.md` | repo root | N/A (documentation) |
| `.factory/release-config.yaml` | `.factory/` | N/A (configuration) |
| `crates/prism-bin/src/boot.rs §spec_dir/plugin_dir resolution` | `prism-bin` | Effectful (reads config path, joins to produce absolute path) |
| `prism.toml.example` | repo root | N/A (configuration template) |

---

## Purity Classification

| Module | Classification | Justification |
|--------|----------------|---------------|
| `RELEASING.md` | N/A | Documentation — no Rust purity boundary applies |
| `.factory/release-config.yaml` | N/A | YAML configuration — no Rust purity boundary applies |
| `boot.rs §spec_dir/plugin_dir path resolution` | Effectful | Resolves paths against config_dir — touches filesystem-relative semantics |
| `prism.toml.example` | N/A | Configuration template — no Rust purity boundary applies |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Future schema extension (schema: 2) | Add schema version migration note to release-config.yaml comments |
| EC-002 | Release from main vs develop | RELEASING.md clarifies: tag targets develop; main is updated by PR merge |
| EC-003 | Hotfix release path | Deferred to story S-REL-hotfix-001 with explicit anchor in RELEASING.md Hotfix section; Canonical Principle Rule 3 compliant (anchor + concrete future dependency stated) |
| EC-004 | Non-release `v*` tag pushed to origin (e.g., `vendor-sync`) | Would trigger release pipeline; containing `-` would publish as unintended prerelease. Prevented by RELEASING.md tag-naming discipline (AC-008). Workflow trigger tightening decision recorded per Task 5 / F-REL001-PR6-003. |

---

## Forbidden Dependencies

- Rust code changes limited to `crates/prism-bin/src/boot.rs` (spec_dir/plugin_dir CWD fix only — no other production code changes)
- No changes to existing CI/CD files (those are owned by S-REL-001 and S-REL-004)
- No `quality_gates: vsdd-full` scalar form in release-config.yaml (must be MAP with mode: vsdd-partial)
- No bare TODO in RELEASING.md hotfix section (must reference S-REL-hotfix-001)
- prism.toml.example must NOT reference crowdstrike, cyberint, or armis sensors (Claroty-only per D-2440)
- prism.toml.example must NOT use `import uuid7` (third-party package; use stdlib uuid.uuid7())

---

## Changelog

| Version | Date | Summary |
|---------|------|---------|
| 0.4 | 2026-09-03 | D-2440 amendments: all docs/RELEASING.md refs → repo-root RELEASING.md; quality_gates vsdd-full→vsdd-partial (doc error corrected); ADR-053→ADR-062 in AC-002/Task-3/Behavioral-Contracts/Arch-Rules; boot.rs CWD bug fix added as Task-6 + AC-009 (RG-002); prism.toml.example Tasks-7 + ACs-010/011/012/013 (UUID-v7 stdlib, keyring+encrypted_file only, Claroty-only sensors, spec_dir relative guidance); crates_touched [prism-bin]; acceptance_criteria_count 8→13; red_gate_tests 1→2 |
| 0.3 | 2026-07-20 | F-REL001-PR6-003 (OBS): scope extended — RELEASING.md tag-naming discipline section added (only vMAJOR.MINOR.PATCH[-rc.N] tags to origin; non-release v* tags forbidden); Task 5 added to evaluate and record workflow trigger-scope decision (decision must be present in runbook, not deferred-TBD); AC-008 added; EC-004 non-release v* tag edge case; acceptance_criteria_count 7→8 |
| 0.2 | 2026-07-19 | Fix-burst: U24/U25 release-config.yaml quality_gates rewritten as MAP (mode+individual gate keys); packages/version_sources added; invented top-level keys (release_series/platforms/artifacts) moved to comments only; AC-005/007 updated to verify MAP structure; U27 EC-003 and RELEASING.md Hotfix section use S-REL-hotfix-001 story anchor instead of bare TODO; research file added to inputs |
| 0.1 | 2026-07-19 | Initial story creation (story-writer F3 burst) |
