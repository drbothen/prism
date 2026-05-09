---
document_type: adr
adr_id: ADR-014
title: "Local Pre-Push vs CI Gate Asymmetry"
status: ACCEPTED
date: 2026-04-30
wave: 3
phase: 3.E
version: "1.1"
authors: [spec-steward]
related_decisions: [D-172, D-174]
related_adrs: [ADR-012]
anchored_capabilities: []
related_bcs_planned: []
subsystems_affected: []
supersedes: null
superseded_by: null
traces_to: specs/architecture/ARCH-INDEX.md
inputs:
  - Justfile
  - lefthook.yml
  - docs/dev-setup.md
  - .factory/stories/S-0.02-developer-toolchain.md
  - .factory/STATE.md (D-172, D-174)
runtime_deliverables:
  - Justfile::check  # fast local pre-push gate (fmt + clippy + nextest + crate-layout, ~5-8 min)
  - Justfile::check-ci  # full-strength CI gate (adds audit + deny + semver-checks)
  - lefthook.yml::pre-push  # hook wiring just check to every git push
runtime_deliverables_note: "Tooling/configuration only — no production Rust code introduced"
wiring_deferred_to: null  # All three deliverables confirmed present in Justfile and lefthook.yml (ratified PR #106 commit 7418f269)
---

# ADR-014: Local Pre-Push vs CI Gate Asymmetry

## Status

ACCEPTED — 2026-04-30. Ratified via W3-FIX-LEFTHOOK-001 (PR #106, commit 7418f269).
D-172 recorded in `.factory/STATE.md`.

---

## 1. Context

### 1.1 The Developer Experience Problem

The original S-0.02 developer toolchain story (v1.0) defined `just check` as a
7-step gate that ran identically to CI:

1. `cargo fmt --check`
2. `cargo clippy --all-features -- -D warnings`
3. `cargo test --workspace --all-features`
4. `cargo deny check`
5. `cargo audit`
6. `cargo semver-checks`
7. `scripts/check-crate-layout.sh`

By Wave 3, this gate was taking 30-45 minutes on developer workstations for a
full cold run — dominated by `cargo semver-checks` (which downloads and compiles
the baseline) and `cargo audit` / `cargo deny check` (which consult the advisory
database on each invocation). This is wired to the lefthook `pre-push` hook, meaning
every `git push` stalls for 30-45 minutes before the remote even receives the commits.

At this iteration rate, a developer pushing 6-8 times per day loses 3-6 hours
per day to the pre-push gate. This is incompatible with rapid iteration on a
project in active development.

### 1.2 The Constraint That Enables the Fix

Decision D-174 (recorded 2026-04-30) established that the Prism repository is
PUBLIC, making CI minutes unlimited and free on GitHub. There is no cost constraint
on running the full-strength gate in CI for every push and PR. The CI wall-clock
time (parallel matrix job, separate from the developer's workstation) does not block
the developer while they iterate locally.

This constraint removal is the enabler for the asymmetry: the developer's pre-push
gate can be fast because CI absorbs the full-strength validation at zero marginal cost,
and branch protection ensures that no PR merges without CI passing.

### 1.3 The Specific Problem with Advisory Checks Locally

`cargo audit` and `cargo deny check` are `Cargo.lock`-driven: they scan the locked
dependency tree against the RustSec advisory database. The advisory database rarely
changes between a developer's morning push and afternoon push. Running these checks
on every `git push` provides near-zero incremental value over running them once in CI
per branch. The check result is determined by `Cargo.lock`, which changes only when
dependencies are updated.

`cargo semver-checks` is similarly stable: it checks API compatibility against a
baseline (typically `origin/develop`). If the developer has not changed public APIs,
`semver-checks` will pass, and this is verifiable by inspection. If public APIs have
changed intentionally, the developer knows this and can run `just semver-checks`
ad hoc before pushing.

---

## 2. Decision

### 2.1 Split `just check` from `just check-ci`

**`just check` — local pre-push fast gate (~5-8 min):**

```just
check:
    cargo fmt --check
    cargo clippy --all-features -- -D warnings
    PROPTEST_CASES=100 cargo test --workspace --all-features
    scripts/check-crate-layout.sh
```

This target is wired to the lefthook `pre-push` hook. It runs in 5-8 minutes on
a typical developer workstation. `PROPTEST_CASES=100` overrides the default
(1000) for the duration of this invocation only — CI uses the default 1000.

**`just check-ci` — full-strength gate (original 7-step semantics):**

```just
check-ci:
    cargo fmt --check
    cargo clippy --all-features -- -D warnings
    cargo test --workspace --all-features
    cargo audit
    cargo deny check
    cargo semver-checks
    scripts/check-crate-layout.sh
```

This target is what CI executes on every push and PR. It is also available locally
for ad-hoc full validation (e.g., before a release branch cut or when investigating
an advisory).

### 2.2 Standalone Targets Remain Available

The following standalone targets allow developers to run individual checks on demand
without executing the full gate:

- `just audit` — `cargo audit` alone
- `just deny` — `cargo deny check` alone
- `just semver-checks` — `cargo semver-checks` alone (recommended before tagging)

### 2.3 Pre-Tag Hook Supplements the Gap

A `pre-tag` lefthook hook was added to run `just semver-checks`, `just audit`, and
`just deny` automatically before every `git tag` invocation. This ensures the advisory
and semver checks run at the last responsible moment before a release, even though
they are no longer part of the standard pre-push gate.

```yaml
# lefthook.yml (pre-tag block)
pre-tag:
  commands:
    semver-checks:
      run: just semver-checks
    audit:
      run: just audit
    deny:
      run: just deny
```

Requires lefthook >= 1.6 (which introduced the `pre-tag` hook type). Developers on
older lefthook versions should run these targets manually before tagging. See
`docs/dev-setup.md` for version verification instructions.

### 2.4 CI Is Unchanged

The `.github/workflows/ci.yml` pipeline was NOT modified by W3-FIX-LEFTHOOK-001.
CI continues to run the full-strength `check-ci` equivalent steps for every push
and PR. Branch protection rules require CI to pass before merge. This is the
authoritative gate; the local pre-push gate is a developer convenience, not a
substitute.

---

## 3. Rationale

**89% reduction in pre-push latency (30-45 min → 5-8 min) enables TDD workflows.**
The primary motivation is developer iteration speed. A gate that takes 30-45 minutes
punishes small, frequent commits — the opposite of TDD discipline. A 5-8 minute gate
is compatible with a commit-push-review cadence of every 20-30 minutes.

**Advisory checks are `Cargo.lock`-stable.** `cargo audit` and `cargo deny check`
produce the same result on every invocation until `Cargo.lock` changes. Running them
on every push adds latency with near-zero incremental value. Running them in CI on
every push (where the result is captured and visible in the PR) provides the same
safety net without blocking the developer.

**`PROPTEST_CASES=100` locally is sufficient for developer iteration.** Proptest with
100 cases catches the overwhelming majority of property violations. The 1000-case
setting in CI provides more exhaustive coverage for edge cases that may only appear
at higher iteration counts. This is the standard split between "fast local feedback"
and "thorough CI validation" for property-based testing.

**Unlimited free CI minutes (D-174) removes the cost objection.** The argument for
running full validation locally would be strongest if CI had a per-minute cost. On a
public repository with unlimited free GitHub Actions minutes, there is no cost reason
to prefer local execution of advisory checks. CI can absorb the full cost.

**Branch protection is the authoritative gate.** Because branch protection requires
CI to pass before merge, no advisory or semver violation can escape into `main` or
`develop`. The local pre-push gate is a fast feedback loop, not a security control.

---

## 4. Consequences

### Positive

- Pre-push latency reduced from 30-45 min to 5-8 min (89% reduction).
- Developer iteration rate increases proportionally.
- TDD workflows (short commit-test-push cycles) become viable.
- No change to CI — full-strength validation is preserved for every PR.
- Pre-tag hook ensures advisories are checked at release time.
- Standalone `just audit` / `just deny` / `just semver-checks` remain available for
  ad-hoc local verification.

### Negative / Accepted Trade-offs

- A developer may push a commit with an open RustSec advisory. CI will catch it before
  merge. This is an accepted trade-off because:
  (a) CI is required before merge per branch protection.
  (b) Advisories in `Cargo.lock` rarely change between a developer's pushes in the same
      day — the same advisory that CI catches was present before the push.
  (c) The pre-tag hook catches advisories before they reach a release tag.

- `PROPTEST_CASES=100` locally means some edge-case property violations may only
  surface in CI (at 1000 cases). This is an accepted trade-off: property violations
  at 100 cases are rare enough that the latency savings outweigh the risk.

- Developers on lefthook < 1.6 do not get the `pre-tag` hook. They must run
  `just semver-checks && just audit && just deny` manually before tagging. This is
  documented in `docs/dev-setup.md`.

### Unchanged

- CI gate: full-strength, unchanged.
- Branch protection: CI required before merge, unchanged.
- All standalone targets: `just audit`, `just deny`, `just semver-checks` available.
- The original `just check` semantics are now expressed by `just check-ci`.
  S-0.02 AC-1 was amended (v1.6) to reflect this rename.

---

## 5. Alternatives Considered

| Option | Description | Decision |
|--------|-------------|----------|
| **Keep slow pre-push** | Retain the 7-step gate as `just check` wired to pre-push | Rejected: 30-45 min pre-push gate blocks rapid iteration and punishes small, frequent commits. Incompatible with TDD workflows at Wave 3+ scale. |
| **Skip pre-push entirely** | Remove the lefthook pre-push hook; rely on CI only | Rejected: removes the fast fmt/clippy/test feedback loop that catches trivial errors before they reach CI. A 5-8 min gate is worth keeping. The CI queue latency (especially for a busy repo) can be 10-20 min on its own. |
| **Self-hosted runners** | Run full gate on a fast self-hosted runner triggered by push | Rejected: adds infrastructure cost and complexity (runner maintenance, secrets management, availability SLA). Not justified for a project of this scale. Public repo with free GitHub-hosted runners is sufficient. |
| **Parallel pre-push** | Run all 7 steps in parallel via `lefthook` parallel mode | Rejected: `cargo audit` and `cargo deny` have file-system locks on the advisory database; `cargo semver-checks` compiles a baseline that conflicts with `cargo test` compilation. Parallelism would cause lock contention and is harder to reason about than sequential steps. |
| **Pre-tag hook only (no check-ci split)** | Keep slow pre-push; add pre-tag as supplement | Rejected: the pre-tag hook alone does not solve the developer iteration problem. The pre-push bottleneck remains. |

---

## 6. References

- **D-172** — Pre-push gate split decision. `.factory/STATE.md` / decisions-archive.
- **D-174** — Repo is PUBLIC; CI cost is not a constraint. `.factory/STATE.md` / decisions-archive.
- **W3-FIX-LEFTHOOK-001** — PR #106, commit 7418f269. Implementation of this decision.
- **S-0.02 Amendment 1** — `.factory/stories/S-0.02-developer-toolchain.md` v1.6.
- **ADR-012** — Workspace `src/` Convention Normalization; established `just check-layout`
  as a gate step (which is included in both `just check` and `just check-ci`).
- **`docs/dev-setup.md`** — Canonical developer documentation for the gate split,
  standalone targets, pre-tag hook, and `CARGO_TARGET_DIR` caveats.

---

## Source / Origin

- **PO decision:** D-172 (pre-push gate split — `just check` fast ~5-8 min vs `just check-ci`
  full ~30-45 min; new pre-tag hook; CI unchanged) — recorded in `.factory/STATE.md`,
  Wave 3 E-3.5 devx fix batch, 2026-04-30.
- **PO decision:** D-174 (repo is PUBLIC — CI cost is not a constraint; unlimited free
  minutes for public repos) — recorded in `.factory/STATE.md`, 2026-04-30.
- **Implementation:** W3-FIX-LEFTHOOK-001, PR #106, commit 7418f269 merged to develop.
  Files changed: `Justfile` (split `check` / `check-ci`), `lefthook.yml` (added `pre-tag`
  hook), `docs/dev-setup.md` (new — developer documentation for the gate split).
- **Story amendment:** S-0.02 v1.6 — `.factory/stories/S-0.02-developer-toolchain.md`.
  AC-1 superseded; Amendments section documents the semantic change to `just check`.

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-04-30 | spec-steward | Initial ADR — ratifies W3-FIX-LEFTHOOK-001 (PR #106); documents local-vs-CI gate asymmetry, `just check` / `just check-ci` split, pre-tag hook, and trade-off acceptance. |
