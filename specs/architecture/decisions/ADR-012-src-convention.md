---
document_type: adr
adr_id: ADR-012
title: "Workspace src/ Convention Normalization — Canonical Crate Layout"
status: PROPOSED
date: 2026-04-27
wave: 3
phase: 3.A
version: "0.7"
authors: [architect]
related_decisions: [D-046, D-060]
related_adrs: []
anchored_capabilities: [CAP-037]
related_bcs_planned: [BC-3.7.001]
subsystems_affected: [SS-01, SS-02, SS-03, SS-04, SS-05, SS-06]
supersedes: null
superseded_by: null
traces_to: specs/architecture/ARCH-INDEX.md
inputs:
  - Cargo.toml (workspace members)
  - lefthook.yml
  - Justfile
  - crates/ (all 22 workspace crates — layout audit)
  - .factory/STATE.md (D-046)
---

# ADR-012: Workspace `src/` Convention Normalization — Canonical Crate Layout

## Status

PROPOSED — decision D-046 recorded (Track 1 housekeeping item #12).
BCs authored at v0.3+ during Phase 3.A; see BC-INDEX.
Implementation is NOT BLOCKED by D-045 (spec-first phasing) — this is a
housekeeping normalization, not a behavioral feature. May proceed in parallel
with spec authoring.

---

## 1. Context

### 1.1 The Convention Gap

The Prism workspace contains 22 crates across six subsystems. Rust's default toolchain
conventions (`cargo new --lib`, `rustfmt`, `cargo-mutants`, `cargo-llvm-cov`) all assume
a canonical crate layout: source in `src/`, integration tests in `tests/`, benchmarks
in `benches/`, examples in `examples/`. Deviation from this layout requires tool
configuration and creates cognitive overhead for contributors navigating the codebase.

Decision D-046 (Track 1 housekeeping item #12) records the user's explicit mandate:
"adopt the `src/` folder convention for this project." This ADR specifies what that
convention means precisely, audits the current state, identifies any deviations that
require remediation, and establishes the lint enforcement mechanism that prevents
future drift.

### 1.2 Current State Audit

An audit of all 22 workspace crates against the canonical shape was conducted as part
of this ADR's authoring. Findings:

**Conformant crates (no action required):**
All 22 crates have `src/lib.rs` (or `src/main.rs` for the `prism-dtu-demo-server`
binary). No crate has a loose `lib.rs` or `main.rs` at the crate root. This is
consistent with `cargo new` defaults — the workspace was started on Rust conventions.

**One exception — `build.rs` at crate root:**
`crates/ocsf-proto-gen/build.rs` is a Cargo build script, not an application source
file. `build.rs` at the crate root is the Cargo-mandated location for build scripts;
this is not a deviation from the `src/` convention. No action required.

**Fixture directory placement — inconsistency identified:**
Nine crates use a top-level `fixtures/` directory (e.g., `crates/prism-dtu-claroty/fixtures/`).
One crate uses `tests/fixtures/` (e.g., `crates/prism-spec-engine/tests/fixtures/`).
This inconsistency means contributors must check two locations for fixture data. The
canonical placement must be decided and enforced.

**Unit test module placement — mixed pattern:**
`crates/prism-core/src/tests/` contains a dedicated test module directory with
`mod.rs` and per-module test files (`test_ids.rs`, `test_alert_severity.rs`, etc.).
Most other crates embed `#[cfg(test)]` modules inline within `src/` files
(e.g., `crates/prism-audit/src/audit_emitter.rs`, `crates/prism-credentials/src/namespace.rs`).
Both patterns are valid Rust conventions; they differ in discoverability. This ADR
does not mandate migration of embedded `#[cfg(test)]` modules — that would be an
unnecessary churn. The rule is: new test modules follow the inline `#[cfg(test)]`
pattern unless the crate already uses the dedicated `src/tests/` pattern
(consistency within a crate takes precedence).

**No integration tests in `src/` — confirmed clean.**
No `src/` files contain integration-test-style cross-crate assertions. All cross-crate
tests are in `tests/` directories.

### 1.3 Scope

This ADR addresses housekeeping only. It does not restructure module hierarchies,
rename types, or change any behavioral logic. The changes are:

1. Canonicalize `fixtures/` placement to the crate root (top-level `fixtures/`
   directory, not nested under `tests/`).
2. Document the canonical crate layout in `docs/CRATE-LAYOUT.md`.
3. Add a `just check-layout` target that validates conformance.
4. Add a `lefthook` pre-commit check for new crate layout conformance.

---

## 2. Decision

### 2.1 Canonical Crate Layout

Every workspace crate MUST conform to the following shape:

```
crates/<crate-name>/
├── Cargo.toml              # required; [package] manifest
├── src/
│   ├── lib.rs              # required for libraries (OR main.rs for binaries)
│   └── <module>/           # optional; subdirectory for each top-level module
│       ├── mod.rs          # optional; use inline module files (lib.rs declares `mod foo;`)
│       └── tests/          # optional; dedicated test submodule for complex unit tests
│           └── mod.rs
├── tests/                  # optional; integration tests (cross-crate, require #[cfg(test)])
│   └── <test_name>.rs
├── fixtures/               # optional; static test fixture data (JSON, TOML, etc.)
│   └── <fixture>.json
├── benches/                # optional; criterion benchmarks
│   └── <bench>.rs
└── examples/               # optional; runnable examples
    └── <example>.rs
```

Rules:

1. **`src/lib.rs` or `src/main.rs` is the entry point.** No loose `.rs` files at the
   crate root (except `build.rs`, which is a Cargo-mandated build script location).

2. **Integration tests go in `tests/`.** Files in `tests/` are compiled as separate
   crates and may use `extern crate` or `use <crate>::...` imports. They must NOT
   be placed in `src/`.

3. **Fixture files go in `fixtures/` at the crate root.** The path
   `crates/<crate-name>/fixtures/<name>.json` is canonical. The path
   `tests/fixtures/` is non-canonical and should be migrated to `fixtures/` when
   the crate is touched in Wave 3 or later.

4. **Unit tests (inline `#[cfg(test)]`) are permitted within `src/` files.** This is
   idiomatic Rust and is not a deviation. Inline unit tests do not need to be extracted.

5. **`src/tests/` is permitted for crates that already use it** (currently: `prism-core`).
   New crates should default to inline `#[cfg(test)]` unless the test module grows
   beyond 200 lines, at which point a dedicated `src/<module>/tests/` submodule is
   appropriate.

6. **No `mod.rs` required for flat module organization.** Rust 2018+ module system
   supports `src/foo.rs` instead of `src/foo/mod.rs`. Both are permitted; do not
   mandate a migration between the two styles.

### 2.2 Fixture Directory Normalization

The one actionable inconsistency from the audit: `crates/prism-spec-engine/tests/fixtures/`
must be migrated to `crates/prism-spec-engine/fixtures/`. This is a path-only change;
no fixture content changes. The migration is a single-story action (one file move,
one Rust path constant update in the test code).

All other fixture directories (`crates/prism-dtu-*/fixtures/`) are already conformant.
No action required for them.

### 2.3 Convention Document: `docs/CRATE-LAYOUT.md`

A short reference document is created at `docs/CRATE-LAYOUT.md` describing the
canonical shape, examples, rationale, and the "when to use `src/tests/` vs inline"
heuristic. This document is the single source of truth for onboarding contributors.

Format: Markdown, under 400 lines. Content:
- Canonical shape with directory tree (as in Section 2.1)
- Rule table (rules 1-6 from Section 2.1)
- FAQ: "Where do fixture files go?", "Where do integration tests go?",
  "Inline unit tests or `src/tests/`?"
- List of all 22 crates with their current conformance status (auto-generated
  section, refreshed by `just check-layout`).

### 2.4 Lint Enforcement: `just check-layout`

A new Justfile target validates the canonical shape against all workspace crates:

```just
# Validate that all workspace crates conform to the canonical src/ layout.
# Exits non-zero and prints offending crates if any deviation is found.
check-layout:
    @scripts/check-crate-layout.sh
```

The `scripts/check-crate-layout.sh` script checks:
1. No `.rs` files at the crate root (except `build.rs`).
2. `src/lib.rs` or `src/main.rs` exists.
3. No `.rs` files directly in `tests/` that look like unit tests
   (heuristic: contains `#[cfg(test)]` — integration test files should not have this).
4. No fixture JSON/TOML files under `tests/fixtures/` (suggests they should be in
   `fixtures/`).

The script exits 0 if all crates conform, non-zero with a per-crate violation list
if any deviate.

### 2.5 Pre-Commit Hook Integration

The `check-layout` script is added to `lefthook.yml` as a pre-commit command:

```yaml
pre-commit:
  parallel: true
  commands:
    fmt:
      glob: "*.rs"
      run: cargo fmt --check {staged_files}
      stage_fixed: true
    clippy:
      glob: "*.rs"
      run: cargo clippy --all-features -- -D warnings
    layout:
      glob: "crates/**"
      run: scripts/check-crate-layout.sh
```

The `layout` command runs only when files under `crates/` are staged, keeping
pre-commit overhead minimal for non-crate changes.

### 2.6 CI Integration

`check-layout` is added to the `check` Justfile target (which runs in CI pre-push):

```just
check:
    cargo fmt --check
    cargo clippy --all-features -- -D warnings
    cargo test --workspace --all-features
    cargo deny check
    cargo audit
    cargo semver-checks
    just check-layout   # new
```

This ensures new crates added by future stories are validated before merging.

### 2.7 New Crates Added by Wave 3

ADR-011 adds `crates/prism-dtu-harness`. That crate MUST conform to the canonical
shape on creation — no migration needed if it is created correctly. The story
implementing `prism-dtu-harness` (scoped from ADR-011) must include `check-layout`
passing as part of its acceptance criteria.

ADR-009 adds `crates/prism-dtu-common/src/generator/` (a module, not a new crate).
Modules within `src/` are not subject to the crate-root rules; module organization
within `src/` follows the existing crate's conventions.

---

## Rationale

**Cognitive load reduction is the primary motivation.** An MSSP security tooling
project like Prism is maintained by a small team (memory: `user_role.md` — Joshua at
1898 & Co). Contributors navigating 22 crates must be able to find test code, fixture
data, and integration tests predictably. "Tests are always in `tests/` or
`src/<module>/tests/`" and "Fixtures are always in `fixtures/`" are rules that can be
internalized in one reading of `docs/CRATE-LAYOUT.md`. "Sometimes tests are in
`tests/`, sometimes in `tests/fixtures/`, sometimes inline" requires checking every
crate individually.

**Tooling works uniformly on the canonical shape.** `cargo-mutants` assumes
`src/` contains the code under mutation and `tests/` contains the integration test
harness. `cargo-llvm-cov` generates coverage reports against `src/` files. `cargo-fuzz`
targets live in the workspace `fuzz/` directory and link against crate source from
`src/`. Fixture files outside `tests/fixtures/` (i.e., in `fixtures/`) are simply
not compiled — they are loaded by path at test runtime. Keeping fixtures out of `tests/`
means they are not accidentally included in integration test compilation units.

**The lint enforcement mechanism is proportionate to the violation risk.** This
workspace already conforms on the most important conventions (all crates use
`src/lib.rs`). The only active deviation is `prism-spec-engine/tests/fixtures/`.
A heavy-handed enforcement approach (CI failure on any deviation) would be appropriate
if violations were widespread. Since only one crate deviates, the enforcement is
added as a pre-commit hook and CI step — it prevents future drift without requiring
a disruptive multi-crate migration.

**Not mandating migration of inline `#[cfg(test)]` modules is deliberate.** Moving
inline unit tests to separate files is a churn with no behavioral benefit. The value
of this ADR is in fixture placement consistency and the canonical shape document —
not in imposing a specific unit test organization style. The rule that crate-internal
style takes precedence (don't mix inline and `src/tests/` within one crate) is
sufficient to prevent the confusion of mixed patterns.

---

## 3. Migration Plan

The only active migration required is `prism-spec-engine`:

**Step 1 — Migrate `prism-spec-engine` fixtures.**
Move `crates/prism-spec-engine/tests/fixtures/` to `crates/prism-spec-engine/fixtures/`.
Update all path references in `tests/` test files that currently use
`"tests/fixtures/<name>"` to use `"fixtures/<name>"` (or better: use
`env!("CARGO_MANIFEST_DIR")` + `"/fixtures/<name>"` for portability).
Gate: `cargo test -p prism-spec-engine` green.

**Step 2 — Add `scripts/check-crate-layout.sh`.**
Implement the validation script (Section 2.4). Verify it passes on the current
workspace after Step 1. Gate: `just check-layout` exits 0.

**Step 3 — Add `just check-layout` to `check` target.**
Update `Justfile` to add `just check-layout` as the final step in `check`.
Gate: `just check` passes in CI.

**Step 4 — Add `lefthook` `layout` command.**
Update `lefthook.yml` to add the `layout` pre-commit command (Section 2.5).
Gate: creating a test file at `crates/prism-core/bad.rs` triggers the hook and
produces a violation message.

**Step 5 — Write `docs/CRATE-LAYOUT.md`.**
Document the canonical shape, rules, and FAQ. Include the 22-crate conformance
table (auto-generated by `check-layout --report`).
Gate: document exists; no broken links.

All five steps are small and independent. They can be combined into a single PR
or split across two (migration + enforcement). Either approach is acceptable.

---

## 4. Alternatives Considered

| Option | Description | Decision |
|--------|-------------|----------|
| **Status quo — no enforcement** | Document conventions in CLAUDE.md; rely on review | Rejected: conventions that are not enforced drift. The current `tests/fixtures/` inconsistency is evidence that undocumented conventions are not self-sustaining. |
| **Mandate `src/tests/` everywhere** | Require all unit tests to be in dedicated `src/<module>/tests/` directories | Rejected: forces migration of every inline `#[cfg(test)]` module — high churn, zero behavioral benefit. `prism-core`'s `src/tests/` pattern is a valid choice but not the only valid choice. |
| **Workspace-level `fixtures/` directory** | Single `fixtures/` at workspace root, shared by all crates | Rejected: crate isolation principle — crate `A`'s test fixtures should live with crate `A`, not in a shared directory that `cargo test -p A` cannot trivially locate. |
| **Cargo workspace `lints` enforcement** | Use `[workspace.lints]` to enforce layout | Not applicable: Cargo workspace lints apply to Rust code patterns, not filesystem layout. A shell script is the correct tool for filesystem layout validation. |
| **Move all tests to `tests/`** | Eliminate inline `#[cfg(test)]` in favor of integration tests only | Rejected: inline unit tests are idiomatic Rust for testing private functions. Moving them to `tests/` would require making private functions `pub(crate)` — a significant API surface change with no benefit. |

---

## 5. Consequences

### Positive

- One rule for fixture locations: `fixtures/` at crate root. No more checking two
  directories.
- `docs/CRATE-LAYOUT.md` is the canonical onboarding reference for workspace layout.
- `just check-layout` provides a local validation target; CI enforces it on every
  push.
- New crates added by future stories are automatically validated on creation.
- `prism-spec-engine` test path consistency restored.

### Negative

- One small migration: `prism-spec-engine/tests/fixtures/` → `prism-spec-engine/fixtures/`.
  Low risk, one PR.
- New `scripts/check-crate-layout.sh` script is shell code that must be maintained
  if the workspace grows (new optional directories, new exception patterns). Risk
  is low — the script logic is simple (`find` + conditional checks).
- `lefthook` pre-commit runs on all `crates/**` staged files. If a contributor
  stages 50 files across 10 crates, the layout check runs once (it scans the whole
  workspace, not per-file). The runtime is under 1 second.

### Unchanged

- All 22 crates retain their current source layout (all already use `src/lib.rs`).
- No module renames, type renames, or behavioral changes.
- `build.rs` at crate root remains; it is a Cargo convention, not a deviation.
- Inline `#[cfg(test)]` modules in `src/` files remain; no migration required.

---

## 6. Behavioral Contracts Scoped by This ADR

The following BCs were authored during Phase 3.A; see BC-INDEX for canonical metadata.

| BC ID | Title | Postcondition summary |
|-------|-------|-----------------------|
| BC-3.7.001 | Workspace src/ Convention Lint Enforcement | For all workspace crates, `just check-layout` exits 0. No `.rs` files exist at any crate root (except `build.rs`). All fixture data is in `crates/<name>/fixtures/`, not `crates/<name>/tests/fixtures/`. |

---

## 7. Open Questions for Next Dispatch

1. **`check-crate-layout.sh` exception list.** Should the script have an explicit
   exception list for `build.rs`? Or should the rule be stated as "no `.rs` files
   at crate root whose name is not `build.rs`"? The latter is cleaner — recommend
   the explicit exclusion rather than a separate exceptions file.

2. **`docs/CRATE-LAYOUT.md` auto-generated conformance table.** The spec says the
   table is "auto-generated by `check-layout --report`". Should `check-layout` support
   a `--report` flag that outputs Markdown? Or is a separate `just layout-report`
   target cleaner? Recommend: `just layout-report` as a separate target that calls
   `check-crate-layout.sh --markdown > docs/CRATE-LAYOUT.md`; the `check` target
   calls the non-Markdown mode.

3. **`prism-ocsf` has no `tests/` directory.** The audit shows `prism-ocsf: lib` with
   no `tests/` entry. This is conformant — `tests/` is optional. The conformance table
   in `docs/CRATE-LAYOUT.md` should explicitly note that `tests/` is optional to avoid
   false violations being filed against `prism-ocsf`.

4. **New crate naming convention.** ADR-012 establishes layout conventions but not
   naming conventions (e.g., is `prism-dtu-harness` the right name pattern, or should
   it be `prism-test-harness`?). Naming conventions are out of scope for this ADR but
   should be addressed in `docs/CRATE-LAYOUT.md` with a forward reference to wherever
   naming is governed (currently, no ADR covers crate naming).

---

## 8. ADR Chain — Related Documents

This ADR is a housekeeping baseline for all Wave 3 crate development.

- **ADR-009:** The generator module (`prism-dtu-common/src/generator/`) is a new module,
  not a new crate. It follows the `src/` internal module convention, not the crate-root
  layout rules. No cross-impact.
- **ADR-011:** The `prism-dtu-harness` crate is a new crate created in Wave 3. Its
  acceptance criteria require `just check-layout` passing, enforcing conformance from
  the moment of creation.

---

## Source / Origin

- **PO decision:** D-046 (Track 1 housekeeping item #12 — adopt `src/` folder
  convention) — recorded in `.factory/STATE.md`, Wave 3 kickoff 2026-04-27.
- **Code as-built — workspace crate audit:**
  All 22 workspace crates confirmed conformant on `src/lib.rs` placement. One
  deviation identified: `crates/prism-spec-engine/tests/fixtures/` vs canonical
  `crates/prism-spec-engine/fixtures/`.
- **Code as-built — existing CI/lint infrastructure:**
  `lefthook.yml` — existing `pre-commit.fmt` and `pre-commit.clippy` commands;
  `layout` command added alongside them.
  `Justfile` — existing `check` target; `just check-layout` appended.
- **Behavioral contract:** BC-3.7.001 — scoped by this ADR; to be authored by
  spec-writer in Phase 3.A.

---

## Decision Refinements (2026-04-27)

The following questions surfaced during BC authoring (Phase 3.A) and were resolved by the orchestrator on 2026-04-27. Each refinement is recorded here for historical traceability and is binding for Wave 3 implementation.

### D-060 — BC-3.7.001 subsystem is SS-01 primary with cross-cutting note

**Question:** BC-3.7.001 (workspace layout conformance) affects all 6 subsystems equally — it is a workspace-wide governance concern, not specific to any single subsystem. What subsystem should it be assigned to?

**Resolution:** BC-3.7.001 subsystem assignment is SS-01 (Sensor Adapters) as primary, with a cross-cutting note acknowledging that the convention affects all 6 subsystems. The note in BC-3.7.001's Traceability section reads: "Primary subsystem: SS-01. Cross-cutting: this convention applies to all workspace crates across SS-01 through SS-06."

**Rationale:** Every BC must have a primary subsystem assignment for routing, ownership, and story decomposition purposes. SS-01 (Sensor Adapters) per ARCH-INDEX owns `prism-sensors` and `prism-spec-engine` — the two crates most directly impacted by new-crate creation in Wave 3 (the harness crate `prism-dtu-harness` from ADR-011 anchors to SS-01 scope). The `prism-dtu-*` behavioral clone crates are test-only infrastructure whose layout conformance is enforced by the same `check-layout` script. Marking the BC as "cross-cutting" in its Traceability section preserves the visibility that all subsystem owners are affected. This pattern is consistent with how ADR-012 itself lists `subsystems_affected: [SS-01, SS-02, SS-03, SS-04, SS-05, SS-06]` — SS-01 appears first as the driving subsystem.

**Affected BCs:** BC-3.7.001

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 0.7 | 2026-04-27 | product-owner | M-003 (pass-13-remediation): Status block updated — "BCs to be authored in subsequent Phase 3.A spec-writer dispatch" → "BCs authored at v0.3+ during Phase 3.A; see BC-INDEX." §6 preamble updated to match. ADR-012 §1.1 crate count confirmed at 22 (consistent with AD-001 v1.6 fix). |
| 0.6 | 2026-04-27 | product-owner | M-001 (pass-11-remediation): §6 BC table title aligned to BC-INDEX canonical Title Case. BC-3.7.001: "Workspace layout conformance" → "Workspace src/ Convention Lint Enforcement". |
| 0.5 | 2026-04-27 | product-owner | M-003 (pass-6-remediation): Frontmatter `title:` corrected to Title Case to match H1 heading (POL 7 H1 source-of-truth). |
| 0.4 | 2026-04-27 | product-owner | m-001 fix: added `anchored_capabilities: [CAP-037]` to frontmatter (per adversary Pass 2 minor finding). |
| 0.3 | 2026-04-27 | product-owner | M-003 fix: D-060 rationale corrected — SS-01 per ARCH-INDEX owns `prism-sensors` and `prism-spec-engine`, not the `prism-dtu-*` family (which is test-only infrastructure). Rationale updated to correctly describe SS-01's scope and why it is the primary subsystem for BC-3.7.001. |
| 0.2 | 2026-04-27 | architect | Decision Refinements: D-060 (BC-3.7.001 subsystem = SS-01 primary with cross-cutting note for all 6 subsystems) |
| 0.1 | 2026-04-27 | architect | Initial draft — scopes D-046; status PROPOSED |
