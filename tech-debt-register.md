---
document_type: tech-debt-register
producer: state-manager
version: "1.0"
last_updated: 2026-04-22T00:00:00
---

# Technical Debt Register

## Summary

| Priority | Count | Estimated Points |
|----------|-------|-----------------|
| P0 (next cycle) | 0 | 0 |
| P1 (within 3 cycles) | 5 | 8 |
| P2 (backlog) | 11 | 6 |

## Debt Items

| ID | Source | Description | Priority | Introduced | Cycle | Story | Due |
|----|--------|-------------|----------|-----------|-------|-------|-----|
| TD-WV0-01 | Phase 5 deferred | post-merge.yml triggers on main only; dev flow lands on develop | P1 | wave-0 | phase-3-dtu-wave-0 | S-TBD (fuzz) | — |
| TD-WV0-02 | Phase 5 deferred | S-0.01 evidence greps YAML strings, not runtime reachability | P1 | wave-0 | phase-3-dtu-wave-0 | first binary crate | — |
| TD-WV0-03 | Phase 5 deferred | FidelityValidator checks top-level fields only; docstring says JSON paths | P1 | wave-0 | phase-3-dtu-wave-0 | S-6.12 | — |
| TD-WV0-04 | Phase 5 deferred | configure() silently drops unknown keys; no strict schema | P1 | wave-0 | phase-3-dtu-wave-0 | first blackbox harness | — |
| TD-WV0-05 | Pattern inconsistency | DTU clone design drift: publish=false, description, /dtu/reset, serialization | P1 | wave-0 | phase-3-dtu-wave-0 | S-6.07 | wave-1 |
| TD-WV0-06 | Maintenance sweep | clippy::unwrap_used: no workspace-level deny policy | P2 | wave-0 | phase-3-dtu-wave-0 | — | wave-1 maintenance |
| TD-WV0-07 | Phase 6 deferred | /dtu/configure endpoint unauthenticated on loopback | P2 | wave-0 | phase-3-dtu-wave-0 | — | if blackbox harness added |
| TD-WV0-08 | Phase 6 deferred | SyslogReceiver does not validate source address | P2 | wave-0 | phase-3-dtu-wave-0 | — | wave-1 maintenance |
| TD-WV0-09 | Dependency | Release workflow uses same-run SHA; no OIDC attestation | P2 | wave-0 | phase-3-dtu-wave-0 | — | pre-first-release |
| TD-WV0-10 | Dependency | GitHub Actions pinned to major tags, not immutable SHAs | P2 | wave-0 | phase-3-dtu-wave-0 | — | pre-first-release |
| TD-WV0-11 | Phase 6 deferred | Secrets at job-level env; should be step-scoped | P2 | wave-0 | phase-3-dtu-wave-0 | — | pre-first-release |
| TD-WV0-12 | Maintenance sweep | prism-no-log-secret semgrep rule misses tracing/log macros | P2 | wave-0 | phase-3-dtu-wave-0 | — | first tracing usage |
| TD-CV-01 | Maintenance sweep | Merged story frontmatter shows status: draft | P2 | wave-0 | phase-3-dtu-wave-0 | — | next state-manager burst |
| TD-CV-02 | Maintenance sweep | STORY-INDEX phase field stale (shows 2, should be 3) | P2 | wave-0 | phase-3-dtu-wave-0 | — | next state-manager burst |
| TD-CV-03 | Maintenance sweep | .factory/current-cycle file stale (shows phase-2-patch) | P2 | wave-0 | phase-3-dtu-wave-0 | — | next state-manager burst |
| TD-CV-04 | Maintenance sweep | wave_0a_complete date off-by-one in STATE.md | P2 | wave-0 | phase-3-dtu-wave-0 | — | next state-manager burst |

### Source Types

| Source | Detection Agent | Description |
|--------|----------------|-------------|
| Phase 5 deferred | adversary | Finding deferred as "fix later" from adversarial review |
| Phase 6 deferred | formal-verifier | Finding deferred from formal hardening |
| Spec drift | spec-steward | BC postcondition not enforced in code |
| Dependency | security-reviewer | Major version bump available or vulnerability |
| DTU fidelity | dtu-validator | Real API changed, clone is stale |
| Pattern inconsistency | code-reviewer | Legacy pattern in older code |
| Holdout decay | holdout-evaluator | Scenario tests removed/changed feature |
| Maintenance sweep | consistency-validator | Anti-pattern or code smell detected |

### Wave 0 Deferral Detail

**TD-WV0-01** — post-merge.yml triggers on `push.branches: [main]`. Dev flow lands on `develop`; post-merge verification runs late. Fix: add `develop` to branches OR create `nightly.yml`. Deferred: post-merge is stubbed until fuzz/kani scaffolding lands.

**TD-WV0-02** — S-0.01 AC-5/6/7/8 evidence greps workflow YAML; doesn't verify runtime reachability. Fix: `act` dry-run or synthetic event tests. Deferred: CI passed on 5 platforms; gap is narrow; `act` investment not justified until binary crate exists.

**TD-WV0-03** — `required_fields: Vec<String>` docstring says "JSON field paths"; impl only checks top-level keys via `body.get(field)`. Fix: tighten docstring OR add JSON pointer traversal. Deferred: current clones have flat response shapes; nested-path not exercised until S-6.12.

**TD-WV0-04** — `configure()` drops unknown keys silently; accepted subset varies per clone. Fix: `#[serde(deny_unknown_fields)]` + per-clone schema doc. Deferred: integration tests control all payloads; no observed drift.

**TD-WV0-05** — Bundle of cosmetic/structural inconsistencies between prism-dtu-threatintel and prism-dtu-nvd. Fix: Canonical L2 Clone Template doc before S-6.07, retroactive application. Deferred: drift doesn't break any AC; template prevents compounding across 11 more clones.

**TD-WV0-06** — `clippy::unwrap_used` has no workspace-level deny policy. Fix: workspace `deny` + per-test `#[allow]`. Deferred: test-file unwraps fixed via `.expect()` in wave-0-gate-fix PR; policy change is a CI-wide concern bundled with first maintenance sweep.

**TD-WV0-07** — `/dtu/configure` has no auth; any loopback process can reconfigure mid-test. Fix: shared-secret token via `BehavioralClone::admin_token()`. Deferred: loopback-only, not externally exploitable; low priority for test infra.

**TD-WV0-08** — `recv_from` doesn't validate `_src` address in SyslogReceiver. Fix: `_src.ip().is_loopback()` check. Deferred: test pollution risk only; bundle with Wave 1 DTU maintenance.

**TD-WV0-09** — Release workflow uses same-run artifact checksum; no out-of-band verification. Fix: `actions/attest-build-provenance` OIDC attestation. Deferred: no public release yet.

**TD-WV0-10** — `actions/checkout@v5` etc. are major tags, not immutable SHAs. Fix: pin to full commit SHAs. Deferred: internal dev; Dependabot configured for SHA rotation once pinned.

**TD-WV0-11** — `HOMEBREW_TAP_TOKEN`, `CARGO_REGISTRY_TOKEN`, `CHOCOLATEY_API_KEY` at job-level env. Fix: move to step-level `env:`. Deferred: no release job running; audit pre-release.

**TD-WV0-12** — Semgrep rule `prism-no-log-secret` misses `tracing::info!`, `log::debug!`, `eprintln!`. Fix: extend patterns. Deferred: wave-0 fix PR narrows rule to credential-named format strings; full macro coverage is Wave 1+ refinement.

**TD-CV-01..04** — Stale state items (story frontmatter status, STORY-INDEX phase, current-cycle file, wave date). Fix: state-manager sweep in wave-0 closeout commit.

## Resolution History

| ID | Resolved In | Story | Resolution |
|----|------------|-------|------------|
| — | — | — | (no items resolved yet) |

## Tech Debt as Feature Mode Cycles

When P0 items accumulate, they become a Feature Mode cycle (Path 3) with
cycle type "refactor":

```
orchestrator: "Tech debt P0 items need attention"
  -> Path 3 (Feature Mode) with cycle type "refactor"
  -> cycles/vX.Y.Z-refactor-[name]/
  -> Same VSDD rigor: specs updated, tests updated, adversarial review
  -> Release: PATCH (no new features) or MINOR (if public behavior changes)
```
