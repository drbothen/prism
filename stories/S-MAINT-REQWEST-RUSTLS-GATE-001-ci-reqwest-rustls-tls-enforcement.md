---
document_type: story
story_id: S-MAINT-REQWEST-RUSTLS-GATE-001
title: "CI gate: enforce ADR-050 reqwest rustls-tls-only workspace rule"
wave: tbd
epic_id: maintenance
priority: P2
status: draft
version: "0.1"
level: ops
producer: story-writer
timestamp: "2026-07-02"
modified: "2026-07-02"
input-hash: "[live-state]"
inputs: []
traces_to: ""
cycle: "v1.0.0-greenfield"
phase: 3
tdd_mode: strict
track: "Platform Engineering"
subsystems: []
# Subsystem anchor: this story touches only the CI toolchain (Justfile + ci.yml),
# not any functional subsystem. No SS-NN anchor is appropriate; devops/CI stories
# (e.g., S-0.01) follow the same pattern of subsystems: [].
crates_touched: []
target_module: devops
behavioral_contracts: []
# BC status: pending PO authorship
verification_properties: []
depends_on: []
blocks: []
points: 3
estimated_days: 1
risk: LOW
acceptance_criteria_count: 4
red_gate_tests: 2
estimated_passes: "1"
holdout_scenarios: []
assumption_validations: []
triggered_by: ADR-050
---

# S-MAINT-REQWEST-RUSTLS-GATE-001: CI gate — enforce ADR-050 reqwest rustls-tls-only workspace rule

## §Origin

ADR-050 (ACCEPTED 2026-07-02, ARCH-INDEX v2.155) established the workspace rule: every
`reqwest` entry in any `Cargo.toml` (including `[dev-dependencies]` and feature-gated
entries) MUST declare `default-features = false, features = ["rustls-tls"]`. The
`native-tls` backend is forbidden because it triggers ~65 s macOS Keychain
initialization, which deterministically exceeds the DTU stage-0 50 s window
(BC-2.06.019).

The current workspace is fully compliant — S-DEMO-FIDELITY-REMEDIATION-001 (commit
cf66151f) corrected 11 dev-dep entries and 1 optional-dep entry (Cargo.lock −151
lines). ADR-050 defers the enforcement gate to a fast-follow because a robust
multi-line TOML-aware check (`cargo metadata --format-version 1 | jq`) is its own
scope.

This story is that fast-follow. It creates the gate so that any future reqwest entry
added without the correct flags fails CI immediately rather than silently reopening the
65 s regression.

## §Narrative

As a Prism contributor, I want CI to reject any `reqwest` dependency entry that lacks
`default-features = false` + `rustls-tls`, so that the native-tls boot regression
fixed in S-DEMO-FIDELITY-REMEDIATION-001 can never be silently reintroduced.

## §Acceptance Criteria

### AC-001 — Gate passes on clean workspace
The `check-reqwest-tls` gate exits 0 when run against the current workspace (all
`reqwest` entries already comply per commit cf66151f).

### AC-002 — Gate fails on a violating fixture
A test fixture `tests/ci-gates/reqwest-native-tls-fixture/Cargo.toml` containing a
deliberate `reqwest = { version = "0.12", features = ["native-tls"] }` entry causes
the gate script to exit non-zero and emit a human-readable message identifying the
offending file and entry.

### AC-003 — Justfile recipe exposed
`just check-reqwest-tls` invokes the gate and is documented in `just --list`.

### AC-004 — CI step wired
`ci.yml` includes a `check-reqwest-tls` step in the PR gate job (same tier as the
existing `cargo deny` step) so branch protection blocks merges that violate ADR-050.

## §Implementation Notes (draft-level guidance)

The gate must handle multi-line TOML correctly — a single grep of Cargo.toml text is
insufficient because `reqwest` can be split across lines:

```toml
# This is a violation — default-features not set to false:
reqwest = { version = "0.12", features = ["rustls-tls"] }

# This is a violation — native-tls present:
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "native-tls"] }
```

Recommended approach: `cargo metadata --format-version 1 | jq` to read the resolved
dependency graph, then grep the raw Cargo.toml text only for the workspace members
that do declare reqwest, verifying each declaration's `default-features` and
`features` keys. Alternatively, a Python/awk script that parses TOML sections.

The fixture in `tests/ci-gates/reqwest-native-tls-fixture/` does NOT need to be a
real Cargo workspace — it can be a static TOML snippet that the gate script is
invoked against via a flag (e.g., `--check-file <path>`).

## §Token Budget Estimate

| Item | Tokens (approx.) |
|------|-----------------|
| This story spec | ~2 k |
| ADR-050 | ~3 k |
| ci.yml (current) | ~4 k |
| Justfile (current) | ~2 k |
| Gate script (new, ~100 LOC) | ~1 k |
| Fixture file (new, ~20 LOC) | <1 k |
| **Total** | **~12 k** |

Well within context window; no splitting required.

## §Tasks

- [ ] Write `scripts/check-reqwest-tls.sh` (or equivalent) using `cargo metadata | jq`
      to enumerate workspace Cargo.toml files and verify each `reqwest` entry has
      `default-features = false` and `features` contains `rustls-tls` but NOT `native-tls`
- [ ] Add test fixture `tests/ci-gates/reqwest-native-tls-fixture/Cargo.toml` with a
      deliberate violation
- [ ] Add unit test (or integration test) that invokes the script against the fixture
      and asserts exit non-zero (AC-002 red gate)
- [ ] Add unit test that invokes against the real workspace root and asserts exit 0 (AC-001)
- [ ] Add `check-reqwest-tls` recipe to `Justfile`
- [ ] Wire the step into `ci.yml` PR gate job after the `cargo deny` step

## §Previous Story Intelligence

N/A — first story in this CI-gate epic. Adjacent precedents for CI gate stories:

- S-0.01 (ci-cd-pipeline): template for ci.yml structure
- S-MAINT-001 (clippy enforcement): pattern for adding a Justfile recipe + lefthook gate
- S-MAINT-POL29-HOOK-001 (lint hook): pattern for script-based enforcement gate

## §Architecture Compliance Rules

- ADR-050 §D1: every `reqwest` entry workspace-wide (including `[dev-dependencies]`,
  optional, and feature-gated) MUST declare `default-features = false, features = ["rustls-tls"]`
- ADR-050 §D2: `native-tls` and its aliases (`native-tls-alpn`, `default-tls`) are
  forbidden workspace-wide
- ADR-050 §D4: widening stage-0 timing budgets to mask native-tls overhead is
  forbidden — the gate must FAIL, not the budget be raised
- The gate script MUST NOT use a simple single-line grep — multi-line TOML splits
  will produce false negatives (ADR-050 §Implementation Guidance)

## §Library & Framework Requirements

No new Rust dependencies. Gate tooling:

| Tool | Version constraint | Justification |
|------|--------------------|---------------|
| `cargo metadata` | ships with pinned toolchain | JSON manifest source of truth |
| `jq` | ≥ 1.6 (already required by existing CI scripts) | JSON filtering |
| bash | ≥ 3.2 (macOS system bash) | gate script runtime |

If `jq` is not available in CI, fall back to a Python 3 script (Python 3 ships on all
GitHub-hosted runners). Do not add a new Cargo dev-dependency for this gate.

## §File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `scripts/check-reqwest-tls.sh` | CREATE | Gate script; exits 1 with offending file list on violation |
| `tests/ci-gates/reqwest-native-tls-fixture/Cargo.toml` | CREATE | Static violation fixture for AC-002 |
| `Justfile` | MODIFY | Add `check-reqwest-tls` recipe |
| `.github/workflows/ci.yml` | MODIFY | Add step after `cargo deny` in PR gate job |

## §Forbidden Dependencies

The gate script must NOT:
- Introduce any new Cargo dependency (not even `[dev-dependencies]`)
- Use Python packages not available on stock GitHub ubuntu-latest runners
- Require the full Rust build to run (should run on `cargo metadata` output alone,
  no compilation needed)
