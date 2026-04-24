---
document_type: tech-debt-register
producer: state-manager
version: "1.4"
last_updated: 2026-04-24T00:00:00
pr_30_merged: 2026-04-23T21:57:32Z
wave_1_gate_pass_1_remediation: "PR #30 (f290f450) merged 2026-04-23"
td_wv1_04_resolved: "PR #32 (4a9dffb1) merged 2026-04-23"
wave_1_5_pr_a_merged: "PR #33 (53931c15) merged 2026-04-24 — closed TD-WV0-01,02,09,10,11,12"
wave_1_5_pr_a1_merged: "PR #34 (5341a43e) merged 2026-04-24 — closed TD-WV05-PR33-001/002/003/004"
wave_1_5_pr_b_merged: "PR #35 (75c58838) merged 2026-04-24 — closed TD-WV0-03,04,06"
wave_1_5_pr_c_merged: "PR #36 (01243a8f) merged 2026-04-24 — closed TD-WV0-08,TD-WV1-03"
wave_1_5_pr_d_merged: "PR #37 (36282777) merged 2026-04-24 — closed TD-S620-004,TD-S620-005"
wave_1_5_pr_d1_merged: "PR #38 (2544645a) merged 2026-04-24 — closed IMPORTANT-001"
wave_1_5_pr_e_merged: "PR #39 (ed41f741) merged 2026-04-24 — closed TD-WV1-04-FU-001/002/003"
wave_1_5_pr_f_merged: "PR #40 (5a2d1c8c) merged 2026-04-24 — closed TD-WV1-01,TD-WV1-02,TD-WV0-07"
wave_1_5_complete: "2026-04-24 — 8 PRs, 24 TDs resolved, 1000 tests"
---

# Technical Debt Register

## Summary

| Priority | Count | Estimated Points |
|----------|-------|-----------------|
| P0 (next cycle) | 0 | 0 |
| P1 (within 3 cycles) | 1 | 3 |
| P2 (backlog) | 5 | 3 |

_Active items: 6. Wave 1.5 debt-reduction sprint (8 PRs, 2026-04-24) resolved 24 items total: 19 pre-existing Wave 1 TDs + 4 PR-A review followups (TD-WV05-PR33-001/002/003/004) + 1 PR-D important closure (IMPORTANT-001). Remaining P1: TD-S-1.07-01 (Wave 5 deferral — DO NOT CLOSE until prism-mcp crate lands). New P2 items registered from Wave 1.5 PR reviews: TD-WV15-PR35-001/002 (PR B deferred), TD-WV15-PR36-001/002 (PR C deferred), TD-WV15-PR40-001 (PR F deferred)._

## Debt Items

| ID | Source | Description | Priority | Introduced | Cycle | Story | Due |
|----|--------|-------------|----------|-----------|-------|-------|-----|
| TD-WV0-01 | Phase 5 deferred | post-merge.yml triggers on main only; dev flow lands on develop | ~~P1~~ RESOLVED | wave-0 | phase-3-dtu-wave-0 | — | PR #33 (53931c15) |
| TD-WV0-02 | Phase 5 deferred | S-0.01 evidence greps YAML strings, not runtime reachability | ~~P1~~ RESOLVED | wave-0 | phase-3-dtu-wave-0 | — | PR #33 (53931c15) |
| TD-WV0-03 | Phase 5 deferred | FidelityValidator checks top-level fields only; docstring says JSON paths | ~~P1~~ RESOLVED | wave-0 | phase-3-dtu-wave-0 | S-6.12 | PR #35 (75c58838) |
| TD-WV0-04 | Phase 5 deferred | configure() silently drops unknown keys; no strict schema | ~~P1~~ RESOLVED | wave-0 | phase-3-dtu-wave-0 | first blackbox harness | PR #35 (75c58838) |
| TD-WV1-01 | S-6.07 test-writer gap | `FidelityCheck` in prism-dtu-common has no `headers` field; fidelity probes cannot send bearer tokens, blocking fidelity checks of auth-required endpoints | ~~P1~~ RESOLVED | wave-1 | S-1.04-red-gate | S-6.07 | PR #40 (5a2d1c8c) |
| TD-WV1-02 | ADR-002 naming collision | ADR-002 §8 mandates `ac_N_fidelity_validator.rs` where N = last AC number; S-6.10 AC numbering ends mid-topic (AC-7 = reset, not fidelity), causing fidelity test to land in `tests/reset_state_invariants.rs` instead of the ADR-prescribed filename — propose ADR-002 amendment or accept divergence | ~~P1~~ RESOLVED | wave-1 | S-1.04-red-gate | S-6.10 | PR #40 (5a2d1c8c) |
| TD-WV0-05 | Pattern inconsistency | DTU clone design drift: publish=false, description, /dtu/reset, serialization | ~~P1~~ RESOLVED | wave-0 | phase-3-dtu-wave-0 | S-6.07 | PR #28 (95c7ff15) |
| TD-S-1.07-01 | S-1.07 scope deferral | CRUD store is thread-local in-memory HashMap (crud.rs). Production wire-up to KeyringBackend/EncryptedFileBackend from S-1.06 deferred until MCP tool surface (task 7, prism-mcp) is implemented. **DEFERRED TO WAVE 5 (2026-04-23 Wave 1.5 sprint).** DO NOT CLOSE until Wave 5 prism-mcp crate lands AND configure_credential_source MCP tool is implemented (S-5.01 or S-5.02 scope). | P1 | wave-1 | S-1.07 | S-5.01 or S-5.02 (prism-mcp crate) | Wave 5 gate |
| TD-WV1-04 | PR review finding | `--tls` flag in prism-dtu-demo-server generates cert + prints fingerprint but does not wire `RustlsConfig` through to each clone's `start_on`. Clones still bind plain HTTP via `axum::serve` when `--tls` is set. AC-4 library-level test passes because it bypasses the binary and calls `bind_rustls` directly; the binary's user-observable `--tls` flag remains cosmetic. `build_rustls_config()` helper is already present; wiring is the remaining step. Fix: extend `BehavioralClone::start_on` to accept `Option<Arc<RustlsConfig>>` and update all 6 clone impls. Noted as LOW-001 in PR #30 review (pr-reviewer approved with deferral). | ~~P1~~ RESOLVED | wave-1-gate-remediation | S-6.20 | — | PR #32 (4a9dffb1) |
| TD-WV1-04-FU-001 | PR #32 review (SUGGESTION-001) | TLS shutdown asymmetry vs HTTP graceful drain: stop() on TLS path calls handle.graceful_shutdown(5s) but HTTP path uses JoinHandle::abort() directly. Asymmetry may cause in-flight request loss on TLS clones vs graceful drain on HTTP. Fix: unify shutdown path to always attempt graceful drain before abort, regardless of TLS mode. | ~~P2~~ RESOLVED | wave-1 | S-6.20 / TD-WV1-04 | — | PR #39 (ed41f741) |
| TD-WV1-04-FU-002 | PR #32 review (SUGGESTION-002) | AC-5 (start/stop lifecycle) test does not cover TLS mode + stop_all() sequence. Only HTTP path is exercised in AC-5 test. Fix: add TLS variant to AC-5 test that starts clone with --tls, calls stop_all(), and verifies port is released. | ~~P2~~ RESOLVED | wave-1 | S-6.20 / TD-WV1-04 | — | PR #39 (ed41f741) |
| TD-WV1-04-FU-003 | PR #32 review (SUGGESTION-003) | stdout pipe capture ordering comment in binary e2e test (td_wv1_04_binary_tls_e2e.rs) is misleading — comment implies synchronous capture but actual impl uses async mpsc; can confuse future maintainers. Fix: update comment to accurately describe async channel-based capture pattern. | ~~P2~~ RESOLVED | wave-1 | TD-WV1-04 | — | PR #39 (ed41f741) |
| TD-WV0-06 | Maintenance sweep | clippy::unwrap_used: no workspace-level deny policy | ~~P2~~ RESOLVED | wave-0 | phase-3-dtu-wave-0 | — | PR #35 (75c58838) |
| TD-WV0-07 | Phase 6 deferred | /dtu/configure endpoint unauthenticated on loopback | ~~P2~~ RESOLVED | wave-0 | phase-3-dtu-wave-0 | — | PR #40 (5a2d1c8c) |
| TD-WV0-08 | Phase 6 deferred | SyslogReceiver does not validate source address | ~~P2~~ RESOLVED | wave-0 | phase-3-dtu-wave-0 | — | PR #36 (01243a8f) |
| TD-WV0-09 | Dependency | Release workflow uses same-run SHA; no OIDC attestation | ~~P2~~ RESOLVED | wave-0 | phase-3-dtu-wave-0 | — | PR #33 (53931c15) |
| TD-WV0-10 | Dependency | GitHub Actions pinned to major tags, not immutable SHAs | ~~P2~~ RESOLVED | wave-0 | phase-3-dtu-wave-0 | — | PR #33 (53931c15) |
| TD-WV0-11 | Phase 6 deferred | Secrets at job-level env; should be step-scoped | ~~P2~~ RESOLVED | wave-0 | phase-3-dtu-wave-0 | — | PR #33 (53931c15) |
| TD-WV0-12 | Maintenance sweep | prism-no-log-secret semgrep rule misses tracing/log macros | ~~P2~~ RESOLVED | wave-0 | phase-3-dtu-wave-0 | — | PR #33 (53931c15) |
| TD-CV-01 | Maintenance sweep | Merged story frontmatter shows status: draft | ~~P2~~ RESOLVED | wave-0 | phase-3-dtu-wave-0 | — | state-manager burst 2026-04-23 (factory-artifacts e6ac1059) |
| TD-CV-02 | Maintenance sweep | STORY-INDEX phase field stale (shows 2, should be 3) | ~~P2~~ RESOLVED | wave-0 | phase-3-dtu-wave-0 | — | audited clean 2026-04-23 — P3WV1B-A-M-002 |
| TD-CV-03 | Maintenance sweep | .factory/current-cycle file stale (shows phase-2-patch) | ~~P2~~ RESOLVED | wave-0 | phase-3-dtu-wave-0 | — | audited clean 2026-04-23 — P3WV1B-A-M-002 |
| TD-CV-04 | Maintenance sweep | wave_0a_complete date off-by-one in STATE.md | ~~P2~~ RESOLVED | wave-0 | phase-3-dtu-wave-0 | — | state-manager burst 2026-04-23 (P3WV1C-A-OBS-001) |
| TD-WV1-03 | PR review suggestion | S-1.09 consume() marks tokens consumed=true in-place (DashMap get_mut) rather than removing the entry; consumed-but-unexpired tokens accumulate until next sweep_expired(). Functionally correct (VP-008 satisfied). Refactor: drop get_mut ref, call self.tokens.remove(token_id) for eager cleanup so active_count() can use store.len() directly. | ~~P2~~ RESOLVED | wave-1 | S-1.09 | — | PR #36 (01243a8f) |
| TD-S620-001 | Workspace hygiene | 6 crates missing from root `Cargo.toml` `[workspace] members`: prism-mcp, prism-ocsf, prism-security, prism-spec-engine, prism-storage, ocsf-proto-gen. Pre-existing debt; S-6.20 partially closed gap by adding 4 DTU crates. Fix: housekeeping sweep to add all 6. | ~~P2~~ RESOLVED | wave-1 | S-6.20 | — | PR #30 (f290f450, commit 1ace1367) |
| TD-WV05-PR33-001 | PR #33 review (S-001) | `post-merge.yml` header comment says "runs on push to main only" after TD-WV0-01 added `develop` trigger. Fix: update comment to "runs on push to main + develop". | ~~P2~~ RESOLVED | wave-1.5 | wave-1-5-pr-a | — | PR #34 (5341a43e) |
| TD-WV05-PR33-002 | PR #33 review (S-002) | `verify-workflow-structure` AC-5 assertion uses `grep -c 'target: '` which self-counts the grep command line (returns 6 instead of 5). Threshold >=5 passes correctly but is imprecise. Fix: tighten pattern to `'target: [a-z]'` to match only matrix target values. | ~~P2~~ RESOLVED | wave-1.5 | wave-1-5-pr-a | — | PR #34 (5341a43e) |
| TD-WV05-PR33-003 | PR #33 review (S-003) | `.semgrep/tests/prism-no-log-secret.rs` uses `// ok` (no colon) for negative case annotation. Canonical Semgrep test annotation is `// ok:`. Fix: add colon so `semgrep --test` correctly recognizes the suppressed case. | ~~P2~~ RESOLVED | wave-1.5 | wave-1-5-pr-a | — | PR #34 (5341a43e) |
| TD-WV05-PR33-004 | PR #33 review (observation) | Semgrep rule `prism-no-log-secret` covers qualified macros (`tracing::info!()`) but not unqualified form (`info!()` via `use tracing::info`). Codebase uses unqualified form extensively. Fix: add patterns for unqualified `trace!/debug!/info!/warn!/error!` macros. | ~~P2~~ RESOLVED | wave-1.5 | wave-1-5-pr-a | — | PR #34 (5341a43e) |
| TD-S620-004 | Documentation | `crates/prism-dtu-demo-server/README.md` missing. Fix: add README covering binary usage, CLI flags, config format, and security model (TLS fingerprint verification). | ~~P2~~ RESOLVED | wave-1 | S-6.20 | — | PR #37 (36282777) |
| TD-S620-005 | Missing artifact | `scripts/start-demo.sh` referenced in spec but not shipped. Fix: add the launcher script for demo harness orchestration. | ~~P2~~ RESOLVED | wave-1 | S-6.20 | — | PR #37 (36282777) |
| TD-WV15-PR35-001 | PR #35 review (deferred) | Fidelity unit test refactor: FidelityValidator tests in Wave 1 clones use integration test fixtures; some fidelity unit tests could be refactored to pure unit tests for faster CI feedback. Deferred from PR B scope. | P2 | wave-1.5 | wave-1-5-pr-b | — | wave-2 maintenance |
| TD-WV15-PR35-002 | PR #35 review (deferred) | NVD ConfigPayload scope: deny_unknown_fields on NvdConfigPayload rejects optional fields used in some NVD API response variants. Audit scope may need narrowing. Deferred from PR B scope. | P2 | wave-1.5 | wave-1-5-pr-b | — | wave-2 maintenance |
| TD-WV15-PR36-001 | PR #36 review (deferred) | TOCTOU comment in SyslogReceiver: loopback gate added (TD-WV0-08 fix) has a TOCTOU window between bind and accept; add a code comment documenting this known limitation and why it is acceptable in the DTU test context. Deferred from PR C scope. | P2 | wave-1.5 | wave-1-5-pr-c | — | wave-2 maintenance |
| TD-WV15-PR36-002 | PR #36 review (deferred) | No-op test rename: one consume() test function retained a legacy name from before the eager-removal refactor; rename for clarity. Deferred from PR C scope. | P2 | wave-1.5 | wave-1-5-pr-c | — | wave-2 maintenance |
| TD-WV15-PR40-001 | PR #40 review (deferred) | Cosmetic #[derive(Default)] opportunity: FidelityCheck struct qualifies for #[derive(Default)] but the manual Default impl is retained; switch to derive macro for conciseness. Deferred from PR F scope. | P2 | wave-1.5 | wave-1-5-pr-f | — | wave-2 maintenance |

### Wave 1.5 PR Review Followup Detail (Active)

**TD-WV15-PR35-001** — PR B (config/workspace hardening) review flagged that the FidelityValidator test assertions in Wave 1 clone fidelity tests are embedded in integration test scaffolding (real server started per test). A pure unit test approach exercising check logic against a mock server would run faster and be less flaky. No correctness issue. Deferred to wave-2 maintenance sweep.

**TD-WV15-PR35-002** — PR B added `#[serde(deny_unknown_fields)]` to NvdConfigPayload. The NVD API response envelope has optional extended fields in some response variants not exercised by current fixture data; deny_unknown_fields may fail on live NVD API responses if those optional fields are encountered. Needs audit of NVD API schema scope before using real NVD endpoint. Deferred: does not affect DTU clone behavior (fixtures controlled by test harness). Wave-2 maintenance.

**TD-WV15-PR36-001** — PR C added `src.ip().is_loopback()` check in SyslogReceiver. There is a TOCTOU window between the loopback check and the packet processing: a non-loopback packet received in the window between bind and accept could pass through. Add an inline comment in SyslogReceiver::recv() documenting this known TOCTOU limitation and why it is acceptable (DTU test context only; loopback-only network; no production exposure). Deferred: documentation-only; no correctness impact on test infrastructure.

**TD-WV15-PR36-002** — PR C refactored S-1.09 consume() to use eager removal. One test function (`test_consume_sets_consumed_flag`) retained its legacy name that references "sets consumed flag" — behavior no longer accurate after the remove() refactor. Rename to `test_consume_removes_entry`. Deferred: no correctness impact; cosmetic rename only.

**TD-WV15-PR40-001** — PR F added manual `Default` impl for `FidelityCheck` (required for the headers field to default to empty vec). The struct now qualifies for `#[derive(Default)]` if all field types implement `Default`. Switch from manual impl to derive macro for conciseness and maintenance clarity. Deferred from PR F scope; cosmetic only.

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

**TD-S-1.07-01** — CRUD store is thread-local in-memory HashMap (crud.rs). Production wire-up to KeyringBackend/EncryptedFileBackend from S-1.06 deferred until MCP tool surface (task 7, prism-mcp) is implemented.

**DO NOT CLOSE THIS ITEM until Wave 5 `prism-mcp` crate lands AND `configure_credential_source` MCP tool is implemented (S-5.01 or S-5.02 scope).**

**Wave 5 prerequisite check:** Before Wave 5 kickoff, confirm this item is on the Wave 5 scope list. Before Wave 5 closure, this item MUST be resolved.

Rationale for deferral (2026-04-23 Wave 1.5 sprint):
- MCP tool surface (prism-mcp crate) does not yet exist; TD-S-1.07-01 cannot be implemented without it
- Implementing prism-mcp now would effectively pull Wave 5 forward by 1 full wave (~2-3 weeks)
- Phase 4 holdout evaluation doesn't run until after all waves complete, so this deferral does NOT affect Phase 4 timing
- Human approved deferral via Q3 answer (Wave 1.5 scope = 19 actionable items; TD-S-1.07-01 excluded)

**TD-WV1-01** — RESOLVED PR #40 (5a2d1c8c). FidelityCheck.headers field added; ADR-003 Amendment #3. See Resolution History.

**TD-WV1-02** — RESOLVED PR #40 (5a2d1c8c). Fidelity test filename convention updated; ADR-003 Amendment #4. See Resolution History.

**TD-S620-001** — Root `Cargo.toml` `[workspace] members` is missing 6 crates: prism-mcp, prism-ocsf, prism-security, prism-spec-engine, prism-storage, ocsf-proto-gen. These crates exist in the repo tree but are not registered as workspace members, which means `cargo build --workspace` and `cargo test --workspace` silently skip them. S-6.20 reduced the gap by registering the 4 DTU demo crates; the 6 remaining are pre-existing hygiene debt. Fix: single `Cargo.toml` edit adding the 6 paths. Deferred: non-blocking for Wave 1 deliverables; bundle with next maintenance sweep.


**TD-S620-004** — RESOLVED PR #37 (36282777). README added to crates/prism-dtu-demo-server/. See Resolution History.

**TD-S620-005** — RESOLVED PR #37 (36282777). scripts/start-demo.sh added. See Resolution History.

**TD-WV1-04-FU-001** — RESOLVED PR #39 (ed41f741). TLS/HTTP shutdown path unified. See Resolution History.

**TD-WV1-04-FU-002** — RESOLVED PR #39 (ed41f741). AC-5 TLS + stop_all() test variant added. See Resolution History.

**TD-WV1-04-FU-003** — RESOLVED PR #39 (ed41f741). Async mpsc comment corrected. See Resolution History.


## Resolution History

| ID | Resolved In | Story | Resolution |
|----|------------|-------|------------|
| TD-WV0-01 | PR #33 (53931c15) | wave-1-5/pr-a | post-merge.yml develop branch trigger added. |
| TD-WV0-02 | PR #33 (53931c15) | wave-1-5/pr-a | verify-workflow-structure AC reachability improved. |
| TD-WV0-09 | PR #33 (53931c15) | wave-1-5/pr-a | actions/attest-build-provenance OIDC attestation added to release workflow. |
| TD-WV0-10 | PR #33 (53931c15) | wave-1-5/pr-a | GitHub Actions pinned to immutable SHAs (27 refs updated). |
| TD-WV0-11 | PR #33 (53931c15) | wave-1-5/pr-a | Secrets moved from job-level to step-level env scoping. |
| TD-WV0-12 | PR #33 (53931c15) | wave-1-5/pr-a | semgrep prism-no-log-secret rule extended to cover tracing/log macros. |
| TD-WV05-PR33-001 | PR #34 (5341a43e) | wave-1-5/pr-a.1 | post-merge.yml header comment updated to "runs on push to main + develop". |
| TD-WV05-PR33-002 | PR #34 (5341a43e) | wave-1-5/pr-a.1 | verify-workflow-structure grep pattern tightened to 'target: [a-z]' eliminating self-count false positive. |
| TD-WV05-PR33-003 | PR #34 (5341a43e) | wave-1-5/pr-a.1 | Semgrep test annotation corrected to '// ok:' (with colon) for proper --test recognition. |
| TD-WV05-PR33-004 | PR #34 (5341a43e) | wave-1-5/pr-a.1 | semgrep rule extended with patterns for unqualified trace!/debug!/info!/warn!/error! macros. |
| TD-WV0-03 | PR #35 (75c58838) | wave-1-5/pr-b | FidelityValidator JSON pointer traversal implemented; docstring updated to match impl. |
| TD-WV0-04 | PR #35 (75c58838) | wave-1-5/pr-b | #[serde(deny_unknown_fields)] added to configure() payload structs across all 6 DTU clones. |
| TD-WV0-06 | PR #35 (75c58838) | wave-1-5/pr-b | clippy::unwrap_used and expect_used set to deny at workspace level; per-test #[allow] annotations added where needed. |
| TD-WV0-08 | PR #36 (01243a8f) | wave-1-5/pr-c | SyslogReceiver loopback gate added: src.ip().is_loopback() check on recv_from. |
| TD-WV1-03 | PR #36 (01243a8f) | wave-1-5/pr-c | S-1.09 consume() refactored to call self.tokens.remove(token_id) for eager cleanup; active_count() now uses store.len() directly. |
| TD-S620-004 | PR #37 (36282777) | wave-1-5/pr-d | crates/prism-dtu-demo-server/README.md added: binary usage, --demo-config/--tls/--port flags, TOML config schema, TLS fingerprint verification workflow. |
| TD-S620-005 | PR #37 (36282777) | wave-1-5/pr-d | scripts/start-demo.sh added: one-command demo launcher wrapping prism-dtu-demo-server with default flags + TLS fingerprint on startup. |
| IMPORTANT-001 | PR #38 (2544645a) | wave-1-5/pr-d.1 | DEMO_FAKE_* environment variable exports added to start-demo.sh; PR D review important closure. |
| TD-WV1-04-FU-001 | PR #39 (ed41f741) | wave-1-5/pr-e | TLS stop() shutdown path unified with HTTP graceful drain path; both now attempt graceful_shutdown before abort. |
| TD-WV1-04-FU-002 | PR #39 (ed41f741) | wave-1-5/pr-e | AC-5 test extended with TLS + stop_all() variant: starts clone with --tls, calls stop_all(), verifies port released. |
| TD-WV1-04-FU-003 | PR #39 (ed41f741) | wave-1-5/pr-e | stdout pipe capture comment in td_wv1_04_binary_tls_e2e.rs rewritten to accurately describe async mpsc channel pattern. |
| TD-WV1-01 | PR #40 (5a2d1c8c) | wave-1-5/pr-f | FidelityCheck headers: Vec<(String, String)> field added + Default impl + FidelityValidator::run header injection loop. ADR-003 Amendment #3. All 6 clone fidelity test files updated with ..Default::default(). |
| TD-WV1-02 | PR #40 (5a2d1c8c) | wave-1-5/pr-f | Fidelity test filename convention changed to tests/fidelity_validator.rs for all levels (ADR-003 Amendment #4); ac_N_fidelity_validator.rs pattern retired. Retroactive renames applied to 4 clone test files. |
| TD-WV0-07 | PR #40 (5a2d1c8c) | wave-1-5/pr-f | /dtu/configure admin token auth implemented: BehavioralClone::admin_token() trait method; per-clone UUID v4 field; X-Admin-Token header checked in all 6 configure handlers; 18 new auth tests (td_wv0_07_configure_requires_admin_token.rs × 6 clones); 12 existing configure tests updated. ADR-003 Amendment #5. |
| TD-WV0-05 | PR #28 (95c7ff15) | S-6.20 prereq | Mounted GET /dtu/health on NvdClone; GET /dtu/health + POST /dtu/reset on ThreatIntelClone. 3 new integration tests. Unblocks S-6.20 Task 3. |
| TD-CV-01 | state-manager burst 2026-04-23 (factory-artifacts e6ac1059) | Wave 1 integration gate Pass 1 | Bulk-updated 17 Wave 1 story frontmatters from status: draft → status: merged, plus 3 additional story updates. Remediated P3WV1-A-M-001. |
| TD-S-1.07-02 | PR #30 (f290f450) | S-1.07 | Replaced uuid_v4_token() pid+nanos entropy with uuid v7 (CSPRNG-seeded monotonic). Commit f150d424. |
| TD-S112-001 | PR #30 (f290f450) | S-1.12 | Replaced SystemTime::now() nonce in generate_confirmation_token with uuid v7 (CSPRNG). Commit f150d424. |
| TD-S112-002 | PR #30 (f290f450) | S-1.12 | Replaced direct std::fs::write with tmp-file + fs::rename for atomic POSIX write in add_sensor_spec.rs. Commit 38e73b99. |
| TD-S620-002 | PR #30 (f290f450) | S-6.20 | Replaced hardcoded 2024 cert validity dates with now_utc() + 365 days. Commit 6ba9f697. |
| TD-S620-003 | PR #30 (f290f450) | S-6.20 | Wired RustlsConfig into axum via axum_server::bind_rustls. HTTPS handshake now active when --tls flag is set. Commit 6ba9f697. |
| TD-S620-006 | PR #30 (f290f450) | S-6.20 | Fixed print_cert_fingerprint to use sha256(DER) formatted as sha256:<hex> per spec AC-12. Commit 6ba9f697. |
| TD-S620-001 | PR #30 (f290f450) | S-6.20 | Added 6 missing crates (prism-mcp, prism-ocsf, prism-security, prism-spec-engine, prism-storage, ocsf-proto-gen) to [workspace.members]. All 16 crates now participate in workspace CI. Test suite: 952 tests (was 428). Commit 1ace1367. |
| TD-CV-02 | state-manager burst 2026-04-23 (P3WV1B-A-M-002) | Wave 1 gate Pass 2 | Superseded by prior sweep. Audited clean 2026-04-23: STORY-INDEX.md frontmatter shows phase: 3. Condition (phase stale showing 2) no longer holds. |
| TD-CV-03 | state-manager burst 2026-04-23 (P3WV1B-A-M-002) | Wave 1 gate Pass 2 | Superseded by prior sweep. Audited clean 2026-04-23: .factory/current-cycle contains phase-3-dtu-wave-1. Condition (stale showing phase-2-patch) no longer holds. |
| TD-CV-04 | state-manager burst 2026-04-23 (P3WV1C-A-OBS-001) | Wave 1 gate Pass 3 | Reconciled via P3WV1C-A-OBS-001 remediation; STATE.md wave_0a_complete updated to 2026-04-22 to match wave-state.yaml gate_date (authoritative source). Off-by-one condition no longer holds. |
| TD-WV1-04 | PR #32 (4a9dffb1) | S-6.20 / Wave 1 scope | Wire TLS from CLI --tls flag through BehavioralClone::start_on (ADR-002 Amendment #2 — start_on now accepts Option<Arc<RustlsConfig>>) to all 6 DTU clones (prism-dtu-crowdstrike, claroty, cyberint, armis, threatintel, nvd) + DemoHarness + main.rs. MEDIUM-001 TLS handle leak on stop_all() fixed in review cycle 2 (commit cd6ae685: tls_handle field + graceful_shutdown 5s). 7 new TLS tests added (952 → 959 workspace tests). Library and binary HTTPS both verified working. User elected to fix in Wave 1 scope rather than defer. |

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
