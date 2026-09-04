---
document_type: adr
adr_id: "ADR-062"
title: "Product Version Alignment — Git Tag as Canonical Version; prism-bin Aligned; Crates Independent"
status: ACCEPTED
date: "2026-09-03"
version: "1.0"
producer: architect
subsystems_affected: [SS-22]
supersedes: []
superseded_by: null
amends: null
anchor_stories:
  - S-REL-002
related_adrs: []
related_bcs: []
locked_decisions: []
wiring_deferred_to: null
---

# ADR-062: Product Version Alignment — Git Tag as Canonical Version; prism-bin Aligned; Crates Independent

## Status

ACCEPTED v1.0 (2026-09-03) — human-approved. Anchored to S-REL-002 (prism-bin version alignment
to 1.0.0-rc.1). Note: S-REL-002 was authored when ADR-053 was available; that slot is now
occupied by the Wave-A sensor-fidelity ADR. This ADR is issued as ADR-062, the next available
sequential ID at time of creation.

---

## Context

The prism workspace contains 24 crates with independently evolved version numbers, the result of
building a large system incrementally. Each internal crate (`prism-core`, `prism-query`,
`prism-sensors`, etc.) is `publish = false` and carries no external semver contract; consumers
never pin individual internal crates.

The product ships as a single `prism` binary built from `prism-bin`. The secops-factory `activate`
skill asserts `prism --version >= 1.0.0-rc.1` as the minimum required version before activating
an analyst environment. Without a versioning policy, the version string reported by `prism --version`
is whatever `prism-bin/Cargo.toml` happens to say, which decouples binary self-identification from
the release tag — a usability and traceability hazard for both analysts and automation.

No `[workspace.package]` was introduced during development. Each crate's version reflects
independent evolution history. Forcing uniform versions via `[workspace.package]` would either
require a large-scale version bump of all internal crates or mask genuine version divergence;
neither outcome is desirable for an `publish = false` workspace.

`cargo-semver-checks` treats the 0.1.0→1.0.0-rc.1 transition as a MAJOR bump (MAJOR 0→1).
It runs checks for major bumps but permits breaking changes, so the check passes trivially.
`cargo-semver-checks` runs in `just check-ci` and the pre-tag hook — not in `just check`.
There is no documented prerelease exemption that causes the check to be skipped on
pre-release suffixes such as `-rc.1`.

---

## Decision

### D1 — Product Version = Git Tag

The product/distribution version is defined by the git tag at time of release. The first
production release tag is `v1.0.0-rc.1`; the subsequent production release is `v1.0.0`.
The tag is the single source of truth for the version of the shipped binary.

### D2 — prism-bin Tracks the Product Tag

`crates/prism-bin/Cargo.toml` `version` field is bumped to match the intended release tag
**before** the tag is created. The mechanism is: `prism-bin` uses `env!("CARGO_PKG_VERSION")`
to satisfy the `--version` flag (via clap auto-derivation). No code change to `main.rs` is
required for each release — only the `Cargo.toml` field changes.

### D3 — All Other Workspace Crates Retain Independent Versioning

No crate other than `prism-bin` changes its version for a product release. Each internal crate
(`prism-core`, `prism-query`, `prism-sensors`, DTU clones, etc.) evolves its version
independently according to its own change history. All internal crates are `publish = false`
and carry no user-visible semver contract.

### D4 — No `[workspace.package]` Introduced

No `[workspace.package]` version unification is adopted. Non-uniform crate versions reflect
genuine independent evolution and must not be collapsed to a synthetic uniform baseline. Adding
`[workspace.package]` would require either a mass version bump of all internal crates (introducing
misleading version churn) or permanently suppressing internal version tracking.

### D5 — Canonical Version Check Form

The canonical form for checking the installed prism version is:

```
prism --version
```

This outputs `prism <version>` where `<version>` matches `crates/prism-bin/Cargo.toml`
`version`. Both `prism --version` (clap auto-flag) and `prism version` (subcommand) output
the same string.

---

## Rationale

This policy directly closes the consumer-contract gap: the secops-factory `activate` skill
asserts `prism --version >= 1.0.0-rc.1`. Without bumping `prism-bin`, the version reported
is `0.1.0`, which fails the skill's version gate and prevents analyst environment activation.

Aligning `prism-bin` alone — rather than all 24 crates — is the minimal, correct change.
The `publish = false` crates carry no external semver contract; bumping them produces no
consumer benefit and adds commit noise with no semantic content. Future release cycles
(v1.0.0, v1.1.0, etc.) follow the same pattern: bump `prism-bin`, tag, release.

The `cargo-semver-checks` behavior on a major bump (MAJOR 0→1) is that checks run but
breaking changes are explicitly permitted, so the check passes trivially regardless of any
API changes in the codebase. This was verified via research (S-REL-002 `release-engineering-uncertainties-2026.md`
uncertainty U6).

---

## Consequences

### Positive

- `prism --version` reliably reports the product release version, satisfying the
  secops-factory `activate` skill version gate.
- Release process is minimal: bump `prism-bin/Cargo.toml`, run `just check`, tag. No
  multi-crate version coordination.
- Internal crate version histories remain meaningful and independently auditable.
- `cargo-semver-checks` passes trivially on every MAJOR bump (e.g., 0.1.0→1.0.0-rc.1,
  1.0.0-rc.1→1.0.0) without requiring additional justification.

### Negative / Trade-offs

- Operators who need to identify the exact internal crate versions (for dependency auditing)
  cannot rely on `prism --version` alone; they must inspect individual `Cargo.toml` files.
  This is acceptable for an `publish = false` workspace where no third party pins internal crate versions.
- The non-uniform version map requires release engineers to remember that only `prism-bin`
  changes for a product release. Mitigated by this ADR as permanent documentation.

### Status as of v1.0

In-effect. S-REL-002 implements D2 (bump `prism-bin` to `1.0.0-rc.1`) and registers this ADR.
All decisions D1–D5 apply immediately upon story merge.

---

## Alternatives Considered

- **Option A: Introduce `[workspace.package]` for uniform versioning.** Considered and rejected.
  Forcing all 24 crates to share a single version number would (a) produce misleading churn in
  crate versions that have not changed, (b) permanently suppress independent crate version
  tracking, and (c) require a mass-amendment of all `Cargo.toml` files for every product release.
  There is no consumer benefit given all crates are `publish = false`.

- **Option B: Add a `BUILD_VERSION` env var override separate from `CARGO_PKG_VERSION`.** Rejected.
  Introducing a parallel version mechanism alongside the standard Cargo version field creates
  confusion about which field is authoritative. The Cargo mechanism is idiomatic and sufficient.

- **Option C: Derive product version from a separate `VERSION` file at build time.** Rejected.
  Adds build complexity (custom `build.rs`) and a third source-of-truth that must be kept in sync
  with `Cargo.toml`. The standard `version` field in `Cargo.toml` is the idiomatic Rust
  mechanism for this purpose.

---

## Source / Origin

- `delta-analysis.md` §4 (`.factory/planning/feature-release-engineering/delta-analysis.md`)
  — version alignment strategy; explicit rejection of `[workspace.package]`.
- `prism-consumer-contract.md` §5.2 (`.factory/planning/feature-release-engineering/prism-consumer-contract.md`)
  — `prism --version` canonical form; expected output `prism 1.0.0-rc.1`.
- S-REL-002 acceptance criteria AC-001..AC-006 — implementation obligations derived from
  this ADR.
- `release-engineering-uncertainties-2026.md` uncertainty U6 (`.factory/research/`)
  — `cargo-semver-checks` MAJOR bump behavior confirmed: runs checks, allows breaking changes,
  no prerelease exemption that skips the check.

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-09-03 | architect | Initial. D1 product version = git tag; D2 prism-bin tracks tag; D3 other crates independent; D4 no workspace.package; D5 canonical form `prism --version`. Anchored to S-REL-002. ADR-062 issued (ADR-053 slot occupied by Wave-A sensor-fidelity ADR). |
