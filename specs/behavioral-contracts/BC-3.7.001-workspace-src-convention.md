---
document_type: behavioral-contract
level: L3
bc_id: BC-3.7.001
title: Workspace src/ Convention Lint Enforcement
version: "0.8"
status: PROPOSED
producer: product-owner
timestamp: 2026-04-27T00:00:00
phase: 3.A
wave: 3
inputs: [.factory/specs/architecture/decisions/ADR-012-src-convention.md]
input-hash: "0c71b16"
traces_to: ".factory/specs/architecture/decisions/ADR-012-src-convention.md"
origin: greenfield
extracted_from: null
subsystem: SS-01
capability: CAP-037
authors: [product-owner]
related_decisions: [D-046]
related_adrs: [ADR-012]
inherits_from: null
superseded_by: null
lifecycle_status: active
introduced: wave-3
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-3.7.001: Workspace src/ Convention Lint Enforcement

## Description

Every workspace crate must conform to the canonical layout defined in ADR-012 §2.1:
`Cargo.toml` at the crate root, `src/lib.rs` (or `src/main.rs` for binaries) as the
entry point, integration tests in `tests/`, fixture data in `fixtures/` at the crate
root (not under `tests/fixtures/`), and no loose `.rs` files at the crate root except
`build.rs`. The `scripts/check-crate-layout.sh` script validates this layout for all
workspace crates; `just check-layout` runs it; CI runs `just check-layout` on every
push. A pre-commit hook in `lefthook.yml` runs the script when files under `crates/**`
are staged. Non-conformant crates introduced by new stories are rejected at the pre-commit
and CI gates before merge.

## Preconditions

1. The workspace contains one or more crates under `crates/`.
2. `scripts/check-crate-layout.sh` exists and is executable.
3. `just check-layout` invokes `scripts/check-crate-layout.sh` as its sole action.
4. `lefthook.yml` contains a `layout` pre-commit command that runs `scripts/check-crate-layout.sh`
   when files under `crates/**` are staged (ADR-012 §2.5).
5. `just check` includes `just check-layout` as a step that runs in CI (ADR-012 §2.6).
6. `docs/CRATE-LAYOUT.md` exists and documents the canonical shape, rules, FAQ, and
   the 22-crate conformance table (ADR-012 §2.3).

## Postconditions

1. For every conformant crate `crates/<name>/` in the workspace, `just check-layout`
   exits 0 and produces no per-crate violation lines for that crate.
2. For a deliberately non-conformant crate (e.g., `Cargo.toml` + `lib.rs` at crate root,
   no `src/` directory), `just check-layout` exits non-zero and prints a violation line
   identifying the offending crate and the specific rule violated.
3. The violation output names the crate path and the violated rule from ADR-012 §2.1
   (e.g., `"crates/bad-crate: no src/lib.rs or src/main.rs found"`).
4. A PR that introduces a non-conformant crate is blocked by the pre-commit hook before
   `git commit` completes, and blocked by CI before merge.
5. `prism-ocsf` has no `tests/` directory; `just check-layout` treats this as conformant
   (the `tests/` directory is optional per ADR-012 §2.1 Rule 1 — documented in
   `docs/CRATE-LAYOUT.md` as a valid exception).
6. `crates/prism-spec-engine/tests/fixtures/` does not exist after migration; fixture
   data for that crate resides at `crates/prism-spec-engine/fixtures/` (ADR-012 §2.2).
7. `build.rs` at any crate root is not flagged as a violation — the script explicitly
   excludes `build.rs` from the "no loose .rs at crate root" check (ADR-012 §7 OQ-1).

## Invariants

1. The canonical layout rules are enforced identically for every workspace crate, including
   newly created crates added by future stories.
2. `check-crate-layout.sh` does not modify any file — it is a read-only validator that
   exits 0 (pass) or non-zero (fail) with human-readable output.
3. The pre-commit hook runs the full workspace scan once per commit, not per staged file —
   runtime is under 1 second for the current 22-crate workspace (ADR-012 §5 Negative).
4. Inline `#[cfg(test)]` modules within `src/` files are not flagged as violations — they
   are idiomatic Rust and explicitly permitted by ADR-012 §2.1 Rule 4.
5. `src/tests/` subdirectories within a crate are not flagged as violations — they are
   permitted for crates that already use the dedicated test module pattern (e.g., `prism-core`)
   per ADR-012 §2.1 Rule 5.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | New crate created with `lib.rs` at crate root (no `src/`) | `just check-layout` exits non-zero; violation line names the crate and rule; pre-commit hook blocks commit |
| EC-002 | New crate created with `src/lib.rs` and a `tests/fixtures/` directory | `just check-layout` exits non-zero; violation line says fixtures should be in `fixtures/` not `tests/fixtures/` |
| EC-003 | `prism-ocsf` has no `tests/` directory | `just check-layout` exits 0; no violation (optional directory) |
| EC-004 | `build.rs` exists at crate root alongside `src/lib.rs` | `just check-layout` exits 0; `build.rs` is excluded from the loose-rs check |
| EC-005 | Workspace grows to 30 crates; no conformance drift | `just check-layout` still exits 0; runtime remains under 1s |
| EC-006 | `src/tests/mod.rs` exists in `prism-core` (existing pattern) | Not flagged; `src/tests/` is a permitted dedicated test submodule pattern |
| EC-007 | `.rs` file at crate root with a name other than `build.rs` (e.g., `helpers.rs`) | `just check-layout` exits non-zero; violation: "loose .rs file at crate root: helpers.rs" |

## Canonical Test Vectors

| Scenario | Fixture Crate State | Command | Expected Exit Code | Expected Output |
|----------|--------------------|---------|--------------------|-----------------|
| TV-1: Conformant crate passes | `prism-storage`: `Cargo.toml`, `src/lib.rs`, `tests/`, `fixtures/` | `just check-layout` | 0 | No violation lines for prism-storage |
| TV-2: Non-conformant crate fails | test fixture: `Cargo.toml` + `lib.rs` at root, no `src/` | `just check-layout` | non-zero | Violation line naming the fixture crate and missing `src/lib.rs` |
| TV-3: `tests/fixtures/` triggers violation | test fixture: `src/lib.rs` + `tests/fixtures/data.json` | `just check-layout` | non-zero | Violation: "fixtures should be in fixtures/, not tests/fixtures/" |
| TV-4: prism-ocsf no tests dir | `prism-ocsf`: `Cargo.toml`, `src/lib.rs`, no `tests/` | `just check-layout` | 0 | No violation (tests/ is optional) |
| TV-5: build.rs at root is permitted | any crate: `Cargo.toml`, `src/lib.rs`, `build.rs` | `just check-layout` | 0 | No violation for build.rs |
| TV-6: Pre-commit hook blocks bad crate | stage files including `crates/new-bad/lib.rs` | `git commit` | blocked | Pre-commit hook prints violation and exits non-zero |
| TV-7: CI gate rejects non-conformant PR | PR introduces crate with `lib.rs` at root | `just check` in CI | non-zero | CI step fails; PR blocked from merge |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-134 | `check-crate-layout.sh` exits 0 for all 22 existing workspace crates after prism-spec-engine fixture migration | integration test (run script against workspace) |
| VP-135 | `check-crate-layout.sh` exits non-zero for a synthetic non-conformant crate fixture (lib.rs at root, no src/) | proptest (run script against fixture directory) |
| VP-136 | `check-crate-layout.sh` is read-only — no file is created, modified, or deleted by the script | integration_test (check filesystem state before vs after) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-037 ("Workspace Crate Layout Convention") per capabilities.md §CAP-037 |
| Capability Anchor Justification | CAP-037 ("Workspace Crate Layout Convention") per capabilities.md §CAP-037 — this BC specifies the lint-enforced workspace layout convention, automated detection of violations, and CI gating, which together constitute the proposed CAP-037 capability. No existing CAP-001 through CAP-035 covers workspace layout governance. |
| L2 Domain Invariants | n/a (housekeeping convention; no DI-NNN enforced) |
| Cross-Cutting Note | Primary subsystem: SS-01. Cross-cutting: this convention applies to all 22 workspace crates regardless of their primary subsystem affiliation. (per D-060, ADR-012 §D-060) |
| Architecture Module | scripts/check-crate-layout.sh; Justfile (check-layout target); lefthook.yml (layout pre-commit command); docs/CRATE-LAYOUT.md |
| Stories | S-3.5.01 |

## Related BCs

- BC-3.5.001 — `prism-dtu-harness` (created in Wave 3 per ADR-011 §2.9) must pass this layout check as part of its acceptance criteria (ADR-012 §2.7)
- BC-3.5.002 — same; network-mode harness crate subject to same layout enforcement

## Architecture Anchors

- `architecture/decisions/ADR-012-src-convention.md#21-canonical-crate-layout` — defines the six layout rules this BC enforces
- `architecture/decisions/ADR-012-src-convention.md#24-lint-enforcement-just-check-layout` — defines the script, exit codes, and checks
- `architecture/decisions/ADR-012-src-convention.md#25-pre-commit-hook-integration` — lefthook.yml `layout` command
- `architecture/decisions/ADR-012-src-convention.md#26-ci-integration` — `just check` CI step

## Story Anchor

S-3.5.01

## VP Anchors

- VP-134 — integration_test: check-crate-layout.sh exits 0 for all 22 workspace crates after fixture migration
- VP-135 — proptest: check-crate-layout.sh exits non-zero for synthetic non-conformant crate
- VP-136 — integration_test: check-crate-layout.sh is read-only (no files created, modified, or deleted)

## Open Questions

- Subsystem assignment for a cross-cutting workspace convention BC: **Resolved — see ADR-012 §Decision Refinements (D-060).** Primary subsystem is SS-01; cross-cutting note acknowledges all 22 workspace crates are affected. Architecture module field: `scripts/check-crate-layout.sh; Justfile; lefthook.yml; docs/CRATE-LAYOUT.md`.

## BC Changelog

| Version | Change |
|---------|--------|
| v0.8 | m-31-001 (Pass 31): Open Questions line 160 updated — "all 7 subsystems are affected (SS-01 through SS-06 and SS-21)" → "all 22 workspace crates" to mirror Traceability Cross-Cutting Note at line 132. |
| v0.7 | pass-30-remediation: m-30-003: Cross-Cutting Note generalized — "SS-01 through SS-06 and SS-21" → "all 22 workspace crates regardless of their primary subsystem affiliation". ADR-012 frontmatter `subsystems_affected` retains primary list for spec-engine scope; BC body is the authoritative cross-cutting statement. |
| v0.6 | pass-20-remediation + pass-21-remediation: Traceability Cross-Cutting Note row and Open Questions updated — SS-list expanded from "SS-01 through SS-06" to "SS-01 through SS-06 and SS-21" per D-060 / m-21-001. |
| v0.5 | m-005 (pass-7-remediation): VP-136 Proof Method column corrected from "manual review + integration test" to "integration_test" — canonical form matches VP-INDEX and verification-architecture.md; parenthetical hint preserved inline. |
| v0.4 | m-001 (Pass 6): `input-hash` populated: SHA1 of input file path (first 7 chars = `0c71b16`). |
| v0.3 | M-004/Audit-5 (Pass 5): Frontmatter `title:` corrected to title-case to match H1 heading. `traces_to:` corrected from `specs/domain-spec/capabilities.md` to `.factory/specs/architecture/decisions/ADR-012-src-convention.md`. |
| v0.2 | Initial authoring from ADR-012. |
