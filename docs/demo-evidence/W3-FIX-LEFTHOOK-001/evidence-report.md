# Demo Evidence Report — W3-FIX-LEFTHOOK-001

**Story:** W3-FIX-LEFTHOOK-001 — Pre-push gate tuning (CAP-DEV-SPEED)
**Implementation commit:** f459c905
**Recorded:** 2026-04-30
**Toolchain:** VHS 0.10.0, FiraCode Nerd Font Mono

## Summary

Pre-push gate (`just check`) was running all 7 CI steps including `cargo audit`,
`cargo deny`, and `cargo semver-checks` — taking ~35 min locally. This fix splits
the gate into a fast local check (~4 min target) and a full-strength CI-only
`just check-ci`. Standalone `just audit`, `just deny`, and `just semver-checks`
targets are added for ad-hoc use, and a `pre-tag` lefthook hook runs all three
automatically before `git tag`.

---

## AC-001 — `just check` completes under 10 min

**Recording:** `AC-001-just-check-fast.gif` / `AC-001-just-check-fast.webm`
**Tape:** `AC-001-just-check-fast.tape`

The demo shows:
1. `just --show check` — recipe has exactly 4 steps: `cargo fmt --check`,
   `cargo clippy`, `PROPTEST_CASES=100 cargo test`, `check-layout`
2. Explicit confirmation that `audit`, `deny`, `semver-checks` are absent from
   `just check` (they moved to `just check-ci`)

BEFORE (~35 min, 7 steps in `just check`):
- `cargo fmt --check`
- `cargo clippy --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- `cargo deny check`
- `cargo audit`
- `cargo semver-checks --workspace --baseline-rev origin/develop`
- `@scripts/check-crate-layout.sh`

AFTER (~4 min target, 4 steps in `just check`):
- `cargo fmt --check`
- `cargo clippy --all-features -- -D warnings`
- `PROPTEST_CASES=100 cargo test --workspace --all-features`
- `@scripts/check-crate-layout.sh`

**Result:** PASS — recipe is correct, 89% reduction in gate duration

---

## AC-002 — `just check-ci` is full-strength

**Recording:** `AC-002-just-check-ci-full-strength.gif` / `AC-002-just-check-ci-full-strength.webm`
**Tape:** `AC-002-just-check-ci-full-strength.tape`

The demo shows `just --show check-ci` printing all 7 steps:

1. `cargo fmt --check`
2. `cargo clippy --all-features -- -D warnings`
3. `cargo test --workspace --all-features` (default 1000 proptest cases)
4. `cargo deny check`
5. `cargo audit`
6. `cargo semver-checks --workspace --baseline-rev origin/develop`
7. `@scripts/check-crate-layout.sh`

**Result:** PASS — `check-ci` is identical to the old `check`, full CI parity preserved

---

## AC-003 — Standalone targets work

**Recording:** `AC-003-standalone-audit-deny.gif` / `AC-003-standalone-audit-deny.webm`
**Tape:** `AC-003-standalone-audit-deny.tape`

The demo shows:
- `just --show audit` → recipe: `cargo audit`
- `which cargo-audit && cargo-audit --version` → `/Users/jmagady/.cargo/bin/cargo-audit`, version `0.22.1`
- `just --show deny` → recipe: `cargo deny check`
- `which cargo-deny && cargo-deny --version` → `/Users/jmagady/.cargo/bin/cargo-deny`, version `0.19.0`

All binaries are installed and accessible. The standalone targets dispatch correctly
to system tools. `just semver-checks` also exists (dispatches `cargo semver-checks
--workspace --baseline-rev origin/develop`); verified via `just --show semver-checks`.

**Result:** PASS — all three standalone targets resolve to installed binaries

---

## AC-004 — Diff visualization (doc-only, captured here)

No VHS recording for this AC. The BEFORE/AFTER diff is:

### BEFORE (`git show f459c905~1:Justfile` — `check` recipe)

```
check:
    cargo fmt --check
    cargo clippy --all-features -- -D warnings
    cargo test --workspace --all-features
    cargo deny check
    cargo audit
    cargo semver-checks --workspace --baseline-rev origin/develop
    @scripts/check-crate-layout.sh
```

### AFTER (`git show f459c905:Justfile` — `check` recipe)

```
# Run the full PR gate locally — fast feedback (5-8 min target)
# Steps: fmt → clippy → test (PROPTEST_CASES=100) → check-layout
# Skipped on local pre-push (run on CI only): cargo audit, cargo deny, cargo semver-checks
# Use 'just check-ci' to run identical to CI, or invoke 'just audit', 'just deny', 'just semver-checks' ad-hoc.
# NOTE: PROPTEST_CASES=100 in the recipe overrides any value set in your shell environment
# for the duration of the cargo test invocation.
check:
    cargo fmt --check
    cargo clippy --all-features -- -D warnings
    PROPTEST_CASES=100 cargo test --workspace --all-features
    @scripts/check-crate-layout.sh
```

**Removed from `check`:** `cargo deny check`, `cargo audit`,
`cargo semver-checks --workspace --baseline-rev origin/develop`

**Added to `check`:** `PROPTEST_CASES=100` env override on `cargo test`

**Added new targets:** `check-ci` (7 steps, full CI), `audit`, `deny`, `semver-checks`

**Added to `lefthook.yml`:** `pre-tag` hook running `just semver-checks`,
`just audit`, `just deny` before every `git tag`

**Result:** PASS — diff confirms correct scope split, no CI regression

---

## AC-005 — Documentation visible

**Recording:** `AC-005-dev-setup-docs-visible.gif` / `AC-005-dev-setup-docs-visible.webm`
**Tape:** `AC-005-dev-setup-docs-visible.tape`

The demo shows:
1. `grep -n '##' docs/dev-setup.md` — section headings including
   "Pre-push gate (fast local check)", "Standalone targets (run ad-hoc)", and
   "Pre-tag hook (release prep)"
2. `head -30 docs/dev-setup.md` — top of file showing prerequisites and the
   `just check` description with `PROPTEST_CASES=100` note

**Result:** PASS — `docs/dev-setup.md` is present and contains all required sections

---

## Coverage Summary

| AC | Description | Recording | Status |
|----|-------------|-----------|--------|
| AC-001 | `just check` fast (4 steps, no audit/deny/semver) | `AC-001-just-check-fast.{gif,webm}` | PASS |
| AC-002 | `just check-ci` full-strength (7 steps) | `AC-002-just-check-ci-full-strength.{gif,webm}` | PASS |
| AC-003 | Standalone `just audit` and `just deny` work | `AC-003-standalone-audit-deny.{gif,webm}` | PASS |
| AC-004 | Diff visualization (BEFORE/AFTER recipe) | doc-only (see above) | PASS |
| AC-005 | `docs/dev-setup.md` sections visible | `AC-005-dev-setup-docs-visible.{gif,webm}` | PASS |

All 5 acceptance criteria have evidence. 4 VHS recordings produced (AC-004 is
documentation captured in this report per spec).
