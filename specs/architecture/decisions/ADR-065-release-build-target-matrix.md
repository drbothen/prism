---
document_type: adr
adr_id: "ADR-065"
title: "Release Build Target Matrix — 4-Platform, Apple-Silicon-only macOS"
status: ACCEPTED
date: "2026-09-07"
version: "1.1"
producer: architect
subsystems_affected: [SS-22]
supersedes: []
superseded_by: null
amends: null
anchor_stories:
  - S-REL-DROP-INTEL-MAC-001   # implementation story — to be authored by product-owner/story-writer
related_adrs:
  - ADR-062   # product version alignment — binary shipped from this matrix
  - ADR-063   # changelog architecture — release notes reference build targets
  - ADR-064   # pre-release binary version identity — references "5-platform" matrix (stale post this ADR)
related_bcs: []
locked_decisions: []
wiring_deferred_to: null
---

# ADR-065: Release Build Target Matrix — 4-Platform, Apple-Silicon-only macOS

## Status

ACCEPTED v1.1 (2026-09-07) — §Site Inventory backfill: `CHANGELOG.md` added to the Documentation
section. The `## [1.0.0-beta.1]` CHANGELOG section advertised "5-platform … x86_64-apple-darwin"
release artifacts and is fed to `gh release create --notes-file` as the PUBLISHED GitHub Release
body; this pending-section non-historical file was omitted from the v1.0 inventory, causing the
Dim-2 sweep gate to miss it. Required change: the pending `## [VERSION]` section MUST reflect
the 4-target matrix (no `x86_64-apple-darwin`); historical released `## [X.Y.Z]` sections are
IMMUTABLE. Strengthens D1/D2 coverage. TD-VSDD-097: Dim-1 CLEAR (ADR-063 checked — no
target-list references, CLEAR; ADR-064 Dim-1 sweep completed in v1.0, CLEAR). Dim-2 HANDOFF
to story-writer: S-REL-DROP-INTEL-MAC-001 AC-001/VF-001 grep scope MUST include `CHANGELOG.md`.
Dim-3: `CHANGELOG.md` pending section MUST reflect the 4-target matrix → S-REL-DROP-INTEL-MAC-001
AC-001 (covered by D1's "all downstream artifacts" mandate; explicitly noted for inventory
completeness).

ACCEPTED v1.0 (2026-09-07) — human-directed. Origin: human directive 2026-09-07 "only support
latest macOS, drop Intel macs." Architecture-only drop: `x86_64-apple-darwin` (Intel macOS)
removed from the release build matrix. No minimum macOS OS-version floor introduced — this is
an architecture (x86_64 vs ARM) decision only, confirmed by human. Anchored to
S-REL-DROP-INTEL-MAC-001 (implementation story to be authored).

---

## Context

The prism release build matrix was previously defined implicitly in `.github/workflows/release.yml`
with five targets:

| Target | Runner | Archive format |
|--------|--------|----------------|
| `aarch64-apple-darwin` | `macos-latest` | `.tar.gz` |
| `x86_64-apple-darwin` | `macos-15-intel` | `.tar.gz` |
| `x86_64-unknown-linux-gnu` | `ubuntu-latest` | `.tar.gz` |
| `x86_64-unknown-linux-musl` | `ubuntu-latest` | `.tar.gz` |
| `x86_64-pc-windows-msvc` | `windows-latest` | `.zip` |

No ADR previously defined which targets are supported, under what conditions a target may be
added or removed, or how macOS support is scoped. The target matrix was embedded across
`release.yml`, `rust-toolchain.toml`, `RELEASING.md`, install scripts, CI gate tests, and
documentation without a canonical authority document.

The human operator has directed (2026-09-07) that prism drops Intel macOS support for v1.0.0-beta.1
and all future releases. The rationale is "only support latest macOS" — Apple Silicon (aarch64) is
the current Apple Mac architecture; Intel Mac (x86_64) is a legacy architecture. The decision
applies only to macOS architecture: no minimum macOS OS-version floor (e.g., Ventura 13.x,
Sonoma 14.x) is specified in this ADR. A future ADR may establish an OS-version floor if
support requirements evolve.

Before this ADR there were no governing rules to prevent reintroduction of the Intel-mac target.
This ADR establishes that governance boundary.

---

## Decision

### D1 — Authoritative 4-Target Release Matrix

Effective v1.0.0-beta.1 and all subsequent releases, the prism release build matrix is exactly
the following four targets:

| Target | Platform | Archive format |
|--------|----------|----------------|
| `aarch64-apple-darwin` | macOS (Apple Silicon — M1/M2/M3/M4 and later) | `.tar.gz` |
| `x86_64-unknown-linux-gnu` | Linux (glibc — Ubuntu/Debian/RHEL standard) | `.tar.gz` |
| `x86_64-unknown-linux-musl` | Linux (static musl — Alpine/containers) | `.tar.gz` |
| `x86_64-pc-windows-msvc` | Windows (MSVC toolchain) | `.zip` |

This table is the single source of truth for the supported distribution targets. All downstream
artifacts (workflows, toolchain config, install scripts, documentation, CI gate tests) MUST
reflect this exact set — anchored to S-REL-DROP-INTEL-MAC-001 AC-001.

### D2 — x86_64-apple-darwin Is Retired

`x86_64-apple-darwin` (Intel macOS) is permanently removed from the release build matrix. It
MUST NOT be reintroduced without:
1. A superseding or amending ADR explicitly reversing or amending this decision, AND
2. Human approval of that ADR.

The `macos-15-intel` GitHub Actions runner and any equivalent Intel macOS runner MUST NOT
appear in `release.yml` or `ci.yml` build matrices — anchored to S-REL-DROP-INTEL-MAC-001 AC-002.

No macOS minimum OS-version floor is specified at this time. macOS support scope = all macOS
versions supported by the `aarch64-apple-darwin` target on `macos-latest` GitHub Actions runner.

### D3 — ADR-064 "5-Platform" References Are Stale

ADR-064 (Pre-Release Binary Version Identity) contains eight informational references to
"5-platform" and "5 build targets" / "5 legs" in its body (§Rationale, §Consequences). These are
descriptive references to the pipeline that existed at time of writing; they do not define the
target matrix. These references are now stale and MUST be updated to "4-platform" / "4 build
targets" / "4 legs" as part of the S-REL-DROP-INTEL-MAC-001 implementation burst.

S-REL-DROP-INTEL-MAC-001 owns this sweep; ADR-064 v2.2 will record the update — anchored to
S-REL-DROP-INTEL-MAC-001 AC-003.

---

## Rationale

Apple Silicon (aarch64-apple-darwin) has been the standard Mac architecture since 2020. As of
2026, the overwhelming majority of Mac users operate on Apple Silicon hardware. Intel Mac (x86_64)
is a legacy architecture facing natural attrition as Apple's official support lifecycle winds down.

Supporting Intel Mac requires:
- A dedicated `macos-15-intel` GitHub Actions runner (the only Intel macOS runner available as
  of this decision; GitHub plans to retire it Aug 2027 per existing release.yml comments)
- Increased CI wall-clock time (macOS runners are among the slowest in GitHub Actions)
- Ongoing maintenance of macOS Intel-specific issues (proptest_cases reduced, timeout margins)
- Install script complexity for the macOS x86_64 / aarch64 detection path

Dropping Intel Mac support:
- Reduces the build matrix from 5 to 4 targets
- Eliminates the `macos-15-intel` runner dependency (independent of GitHub's Aug 2027 EOL)
- Simplifies install scripts (macOS = `aarch64-apple-darwin`; no arch detection needed for Mac)
- Removes one of the two slowest CI runners from the test matrix

The decision is architecture-only. No minimum macOS OS-version floor is imposed, meaning
`aarch64-apple-darwin` users on any macOS version supported by the `macos-latest` runner are
unaffected. The scope confirmed by human: "drop Intel macs."

---

## Consequences

### Positive

- Build matrix reduced from 5 to 4 targets; CI wall-clock time improved.
- `macos-15-intel` runner dependency eliminated.
- Install script macOS path simplified: macOS always resolves to `aarch64-apple-darwin`.
- Maintenance surface reduced by one CI runner configuration.
- No operator impact for the MSSP target audience (enterprise SOC analysts on Apple Silicon
  MacBooks or Linux/Windows workstations).

### Negative / Trade-offs

- Intel Mac users (x86_64) can no longer install pre-built prism binaries. They may build
  from source via `cargo install` or `cargo build --release`. This is an acceptable trade-off
  given the target audience and the architecture lifecycle.
- Existing documentation, install scripts, and CI gate tests require a coordinated sweep
  update (tracked in the inventory below and executed by S-REL-DROP-INTEL-MAC-001).
- ADR-064 §Consequences "works identically on all 5 build targets" references become stale;
  must be updated to reflect 4 targets (D3 above).

### Status as of v1.0

PENDING implementation. S-REL-DROP-INTEL-MAC-001 (to be authored) owns:
- AC-001: Remove `x86_64-apple-darwin` from all release and CI workflows, toolchain targets,
  install scripts, gate tests, and documentation
- AC-002: Update all "5 platform" / "5 target" / "5 archive" counts to 4 across all sites
- AC-003: Update ADR-064 to replace "5-platform" / "5 build targets" / "5 legs" with "4-platform" /
  "4 build targets" / "4 legs" (D3 above)
- AC-004: Update CLAUDE.md toolchain header (authorized by human)
- AC-005: Confirm no functional regression on the 4 retained targets (CI green)

---

## Site Inventory (Dim-2 Downstream Copy Map)

The following files contain `x86_64-apple-darwin` or "5-platform" / "5-target" / "5 archive"
references that MUST be updated by S-REL-DROP-INTEL-MAC-001. This inventory is the authoritative
Dim-2 map per TD-VSDD-097. Sites marked [IMMUTABLE] are historical demo-evidence records and
must not be retroactively changed.

### Workflows and CI (devops-engineer)

| File | Reference | Required Change |
|------|-----------|-----------------|
| `.github/workflows/release.yml` | Line 1 comment "5-platform builds"; lines 37-39 `runner: macos-15-intel` + `target: x86_64-apple-darwin` | Remove Intel leg; update comment to "4-platform builds" |
| `.github/workflows/ci.yml` | Line 97 `target: x86_64-apple-darwin` + `runner: macos-15-intel`; line 83 timing comment citing `x86_64-apple-darwin`; lines 1973/1976 `AC-5: 5 targets` assertion | Remove Intel matrix leg; update timing comment; update AC-5 assertion floor from 5 to 4 |
| `.github/workflows/release-promote.yml` | Line 12 "full 5-platform build" | Update to "4-platform" |
| `.github/workflows/release-tag.yml` | Lines 7, 172, 193 "5-platform" references | Update all three to "4-platform" |

### Toolchain (devops-engineer)

| File | Reference | Required Change |
|------|-----------|-----------------|
| `rust-toolchain.toml` | `"x86_64-apple-darwin"` in `targets` list | Remove entry |
| `CLAUDE.md` | Toolchain header: `aarch64-apple-darwin, x86_64-apple-darwin, ...` | Remove `x86_64-apple-darwin` from list (human-authorized) |

### Documentation (devops-engineer / technical-writer)

| File | Reference | Required Change |
|------|-----------|-----------------|
| `CHANGELOG.md` | Pending `## [VERSION]` section describes release artifacts including platform targets; fed to `gh release create --notes-file` as the PUBLISHED GitHub Release body | Pending `## [VERSION]` section MUST reflect the 4-target matrix (remove `x86_64-apple-darwin` from any target listing); historical released `## [X.Y.Z]` sections (e.g., `## [1.0.0-beta.1]`) are IMMUTABLE — MUST NOT be retroactively changed |
| `RELEASING.md` | Test matrix list section; build matrix table section; "5 platform archives"/"5 legs" count mentions; macOS Intel install instructions snippet | Remove Intel row from table; update "5" counts to "4"; remove Intel install snippet |
| `README.md` | "macOS Apple Silicon/Intel" badge/text; macOS Intel table row in download table | Remove Intel references |
| `docs/SETUP.md` | "macOS Intel" text; macOS Intel table row | Remove Intel references |
| `docs/RELEASE-CHANNELS.md` | "5-platform" count mentions; "5-platform matrix" mention; macOS Intel table row | Remove Intel row from table; update "5-platform" to "4-platform" |
| `docs/dev-setup.md` | `x86_64-apple-darwin` PROPTEST_CASES note entry | Remove Intel entry |

### Install Scripts (devops-engineer)

| File | Reference | Required Change |
|------|-----------|-----------------|
| `scripts/install.sh` | Line 11 comment `x86_64-apple-darwin macOS (Intel)` (exact); `Darwin-x86_64` detection arm | Remove Intel comment; remove/error-out the `Darwin-x86_64` detection arm |

### CI Gate Tests (devops-engineer)

| File | Reference | Required Change |
|------|-----------|-----------------|
| `tests/ci-gate/test_AC-3_matrix-5-platforms.sh` | Lines 2/18/40/57: "5 platform"; `"x86_64-apple-darwin"` in TARGETS; `macos-15-intel` in RUNNERS | Remove Intel target and runner; update counts to 4; rename file to `test_AC-3_matrix-4-platforms.sh` |
| `tests/ci-gate/README.md` | "All 5 platform targets" reference | Update to 4 |
| `tests/release-gate/test_AC-006_matrix-targets.sh` | Lines 2/25/37-41: "5-platform"; `assert_contains ... x86_64-apple-darwin`; exact count=5 | Remove Intel assertion; change exact count to 4 |
| `tests/release-gate/README.md` | "5 platform targets" / "5 matrix entries" | Update to 4 |
| `tests/release-gate/test_AC-012_install-scripts.sh` | Lines 10/73-74: "5 targets"; Intel detection assertion | Remove `x86_64-apple-darwin` AC-002 assertion; update "5 targets" to "4" |

### .factory/ Specs (story-writer / spec-steward)

| File | Reference | Required Change |
|------|-----------|-----------------|
| `.factory/release-config.yaml` | Comment lines 48-52: "5-platform binary matrix (aarch64-apple-darwin, x86_64-apple-darwin, ...)" | Update to "4-platform" and remove `x86_64-apple-darwin` from the comment list |
| `.factory/stories/S-REL-003-install-scripts.md` | `x86_64-apple-darwin` occurrences in story acceptance criteria (multiple ACs) | Story-writer: update to reflect 4-target install script |
| `.factory/stories/S-REL-004-demo-bundle-packaging.md` | target table section | Story-writer: update |
| `.factory/stories/S-REL-005-releasing-runbook.md` | "5 platforms" with Intel mac references | Story-writer: update |
| `.factory/stories/W3-FIX-CI-001-ci-wall-clock-optimization.md` | `x86_64-apple-darwin: proptest_cases: 256` entry | Story-writer: remove Intel entry |
| `.factory/stories/STORY-INDEX.md` | S-MAINT-EDITION-SYNC-001 entry: "4 targets (aarch64-apple-darwin, x86_64-apple-darwin, ...)" | Story-writer: update to 3 cross-compile targets (x86_64-apple-darwin removed) |
| `.factory/specs/architecture/installation.md` | Build Matrix table `x86_64-apple-darwin` row | **Updated in this burst (architect)** |
| `.factory/planning/feature-release-engineering/delta-analysis.md` | macOS x86_64 row | Spec-steward: remove row |

### Historical Records (IMMUTABLE — do not change)

The following are historical demo-evidence or ingestion artifacts. They document what was true
at story execution time and MUST NOT be retroactively altered.

- `docs/demo-evidence/S-0.01/AC-3-matrix-5-platforms.md`
- `docs/demo-evidence/S-0.01/AC-4-cargo-audit.md`
- `docs/demo-evidence/S-0.01/AC-6-release-artifacts.md`
- `docs/demo-evidence/S-0.01/evidence-report.md`
- `docs/demo-evidence/S-REL-001/evidence-report.md`
- `docs/demo-evidence/S-REL-001/fork-tag-dry-run.md`
- `docs/demo-evidence/W3-FIX-CI-001/evidence-report.md`
- `.factory/phase-0-ingestion/recovered-architecture.md`
- `.factory/research/2026-04-30-ci-free-tier-optimization.md`
- `.factory/cycles/` (all adversarial review records are historical)
- `.factory/stories/S-0.01-ci-cd-pipeline.md` (completed story)
- `.factory/stories/S-0.02-developer-toolchain.md` (completed story)
- `.factory/stories/S-REL-001-release-yml-repair.md` (completed story)
- `.factory/stories/S-MAINT-CI-DISK-EXHAUSTION-001-ci-disk-exhaustion-hardening.md` (historical timing data)

---

## Alternatives Considered

- **Option A: Keep all 5 targets.** Rejected. Human-directed decision.

- **Option B: Keep Intel Mac with a "community support" tier (no CI, build from source).** Not
  requested. The human directive is a clean drop with no tiered support. Intel Mac users may
  build from source via `cargo build --release` on Rust stable.

- **Option C: Add a minimum macOS OS-version floor (e.g., Sonoma 14.x) alongside the architecture
  drop.** Not requested. The human explicitly confirmed no OS-version floor: "architecture drop
  only." A future ADR may add this if deployment targeting narrows.

---

## Source / Origin

- Human directive 2026-09-07: "only support latest macOS, drop Intel macs."
- Human confirmation 2026-09-07: "NO separate minimum-macOS-OS-version floor (architecture drop
  only)."
- Human directive 2026-09-07: "This lands in v1.0.0-beta.1 (human-directed: fold in now, before
  the tag)."

---

## TD-VSDD-097 Discharge

**Dim-1 (sibling ADR sweep):** ADR-064 is the sibling — it contains eight informational
"5-platform" / "5 build targets" / "5 legs" references. ADR-064 v2.1→v2.2 amended in this
burst: `related_adrs` extended with ADR-065; `anchor_stories` extended with
S-REL-DROP-INTEL-MAC-001; and a §Status note added that "5-platform" counts are stale post-ADR-065
and are updated by S-REL-DROP-INTEL-MAC-001 AC-003. ADR-062 and ADR-063 checked — no
target-list references (CLEAR).

**Dim-2 (downstream copy targets):** The Site Inventory above IS the Dim-2 map. Every file
that carries the target list or a derivative "5-platform" count is enumerated. v1.1 backfill:
`CHANGELOG.md` added to the Documentation section — its pending `## [VERSION]` section feeds
the published GitHub Release body via `release.yml` `--notes-file` and was omitted from the v1.0
inventory. Dim-2 HANDOFF to story-writer: S-REL-DROP-INTEL-MAC-001 AC-001/VF-001 grep scope MUST
include `CHANGELOG.md`.

**Dim-3 (mandate anchors):** Every MUST in this ADR is anchored:
- D1 MUST → S-REL-DROP-INTEL-MAC-001 AC-001 (covers all downstream artifacts including `CHANGELOG.md` pending section per "all downstream artifacts... MUST reflect this exact set")
- D2 MUST → S-REL-DROP-INTEL-MAC-001 AC-002
- D3 MUST → S-REL-DROP-INTEL-MAC-001 AC-003

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.1 | 2026-09-07 | architect | §Site Inventory backfill: `CHANGELOG.md` added to Documentation section. Pending `## [VERSION]` CHANGELOG section feeds `gh release create --notes-file` as the PUBLISHED GitHub Release body (e.g., `## [1.0.0-beta.1]` section described "5-platform … x86_64-apple-darwin" release artifacts) — non-historical file omitted from v1.0 inventory; Dim-2 sweep gate missed it. Required change: pending `## [VERSION]` section MUST reflect the 4-target matrix (no `x86_64-apple-darwin`); historical released `## [X.Y.Z]` sections are IMMUTABLE. Strengthens D1/D2 coverage. §TD-VSDD-097 Discharge: Dim-2 updated with CHANGELOG.md + Dim-2 HANDOFF to story-writer; Dim-3 CHANGELOG.md note added. TD-VSDD-097: Dim-1 CLEAR (ADR-063 checked — no target-list references, CLEAR; ADR-064 Dim-1 swept in v1.0, CLEAR). Dim-2 HANDOFF to story-writer: S-REL-DROP-INTEL-MAC-001 AC-001/VF-001 grep scope MUST include `CHANGELOG.md`. Dim-3: `CHANGELOG.md` pending section MUST reflect 4-target matrix → S-REL-DROP-INTEL-MAC-001 AC-001 (covered by D1 "all downstream artifacts" mandate; explicitly noted for inventory completeness). ARCH-INDEX v2.376→v2.377. |
| 1.0 | 2026-09-07 | architect | Initial. Human-directed 2026-09-07: drop x86_64-apple-darwin (Intel mac); 4-target matrix established; D1 authoritative matrix; D2 x86_64-apple-darwin MUST NOT be reintroduced; D3 ADR-064 "5-platform" stale-count sweep anchored to S-REL-DROP-INTEL-MAC-001 AC-003. Site inventory (Dim-2 map) documents all 40+ affected sites across workflows/toolchain/docs/tests/.factory/. TD-VSDD-097: Dim-1 ADR-064 sibling swept (v2.1→v2.2); Dim-2 inventory is the map; Dim-3 all MUSTs anchored to S-REL-DROP-INTEL-MAC-001 AC-001/AC-002/AC-003. ARCH-INDEX v2.375→v2.376. |
