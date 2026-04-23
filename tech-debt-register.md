---
document_type: tech-debt-register
producer: state-manager
version: "1.0"
last_updated: 2026-04-23T18:00:00
---

# Technical Debt Register

## Summary

| Priority | Count | Estimated Points |
|----------|-------|-----------------|
| P0 (next cycle) | 0 | 0 |
| P1 (within 3 cycles) | 11 | 17 |
| P2 (backlog) | 17 | 11 |

## Debt Items

| ID | Source | Description | Priority | Introduced | Cycle | Story | Due |
|----|--------|-------------|----------|-----------|-------|-------|-----|
| TD-WV0-01 | Phase 5 deferred | post-merge.yml triggers on main only; dev flow lands on develop | P1 | wave-0 | phase-3-dtu-wave-0 | S-TBD (fuzz) | — |
| TD-WV0-02 | Phase 5 deferred | S-0.01 evidence greps YAML strings, not runtime reachability | P1 | wave-0 | phase-3-dtu-wave-0 | first binary crate | — |
| TD-WV0-03 | Phase 5 deferred | FidelityValidator checks top-level fields only; docstring says JSON paths | P1 | wave-0 | phase-3-dtu-wave-0 | S-6.12 | — |
| TD-WV0-04 | Phase 5 deferred | configure() silently drops unknown keys; no strict schema | P1 | wave-0 | phase-3-dtu-wave-0 | first blackbox harness | — |
| TD-WV1-01 | S-6.07 test-writer gap | `FidelityCheck` in prism-dtu-common has no `headers` field; fidelity probes cannot send bearer tokens, blocking fidelity checks of auth-required endpoints | P1 | wave-1 | S-1.04-red-gate | S-6.07 | wave-2 or per arch decision |
| TD-WV1-02 | ADR-002 naming collision | ADR-002 §8 mandates `ac_N_fidelity_validator.rs` where N = last AC number; S-6.10 AC numbering ends mid-topic (AC-7 = reset, not fidelity), causing fidelity test to land in `tests/reset_state_invariants.rs` instead of the ADR-prescribed filename — propose ADR-002 amendment or accept divergence | P1 | wave-1 | S-1.04-red-gate | S-6.10 | wave-2 or per arch decision |
| TD-WV0-05 | Pattern inconsistency | DTU clone design drift: publish=false, description, /dtu/reset, serialization | ~~P1~~ RESOLVED | wave-0 | phase-3-dtu-wave-0 | S-6.07 | PR #28 (95c7ff15) |
| TD-S-1.07-01 | S-1.07 scope deferral | CRUD store is thread-local in-memory HashMap (crud.rs). Production wire-up to KeyringBackend/EncryptedFileBackend from S-1.06 deferred until MCP tool surface (task 7, prism-mcp) is implemented. | P1 | wave-1 | S-1.07 | task-7 / S-6.04 | before MCP surface lands |
| TD-S-1.07-02 | Security — weak token | uuid_v4_token() in crud.rs uses pid+nanos, not CSPRNG. Confirmation tokens must use CSPRNG (e.g. rand::thread_rng) before production. Scoped to same deferred wire-up as TD-S-1.07-01 since confirmation tokens are scaffolding until S-1.09 integration lands. | P1 | wave-1 | S-1.07 | before prism-mcp ship | before MCP surface lands |
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
| TD-WV1-03 | PR review suggestion | S-1.09 consume() marks tokens consumed=true in-place (DashMap get_mut) rather than removing the entry; consumed-but-unexpired tokens accumulate until next sweep_expired(). Functionally correct (VP-008 satisfied). Refactor: drop get_mut ref, call self.tokens.remove(token_id) for eager cleanup so active_count() can use store.len() directly. | P2 | wave-1 | S-1.09 | — | S-3.04 (first consumer) |
| TD-S112-001 | Security review finding | `generate_confirmation_token` in `add_sensor_spec.rs:186-198` uses `SystemTime::now()` as entropy source for the nonce. Predictable under VM clock skew. Fix: replace with `rand::thread_rng()` nonce. Severity: Important (not exploitable in local MCP context; no remote attack surface). | P2 | wave-1 | S-1.12 | — | S-1.16+ (write-path hardening) |
| TD-S112-002 | Security review finding | `add_sensor_spec.rs:252` uses `std::fs::write` directly. A crash mid-write leaves a partial `.sensor.toml` that poisons the next reload cycle. Fix: write to `.sensor.toml.tmp` then `fs::rename` (atomic on POSIX). Severity: Important (data integrity; no security boundary crossed). | P2 | wave-1 | S-1.12 | — | S-1.16+ (write-path hardening) |
| TD-S620-001 | Workspace hygiene | 6 crates missing from root `Cargo.toml` `[workspace] members`: prism-mcp, prism-ocsf, prism-security, prism-spec-engine, prism-storage, ocsf-proto-gen. Pre-existing debt; S-6.20 partially closed gap by adding 4 DTU crates. Fix: housekeeping sweep to add all 6. | P2 | wave-1 | S-6.20 | — | next maintenance sweep |
| TD-S620-002 | Security / correctness | TLS cert validity dates hardcoded to 2024 — already expired 2026-04-23. Fix: use `time::OffsetDateTime::now_utc()` + 365 days for NotBefore/NotAfter. Critical to fix before any stakeholder demo. | P1 | wave-1 | S-6.20 | — | before next demo |
| TD-S620-003 | Incomplete feature | TLS not wired into axum-server — HTTPS handshake unimplemented. `--tls` flag generates cert and prints fingerprint but axum still listens plain HTTP. Fix: integrate `axum-server` crate with `RustlsConfig`. | P1 | wave-1 | S-6.20 | — | before next demo |
| TD-S620-004 | Documentation | `crates/prism-dtu-demo-server/README.md` missing. Fix: add README covering binary usage, CLI flags, config format, and security model (TLS fingerprint verification). | P2 | wave-1 | S-6.20 | — | wave-2 maintenance |
| TD-S620-005 | Missing artifact | `scripts/start-demo.sh` referenced in spec but not shipped. Fix: add the launcher script for demo harness orchestration. | P2 | wave-1 | S-6.20 | — | wave-2 maintenance |
| TD-S620-006 | Correctness | `print_cert_fingerprint` uses `hex(base64(DER))` — spec calls for SHA-256 of raw DER bytes. Fix: `sha256::digest(der_bytes)` formatted as `sha256:<hex>`. | P2 | wave-1 | S-6.20 | — | wave-2 maintenance |

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

**TD-WV1-01** — `FidelityCheck` struct in `prism-dtu-common` has no `headers: HashMap<String, String>` field. Fidelity probes cannot send bearer tokens or other auth headers, which means `FidelityValidator` cannot check auth-required endpoints without pre-configuring the DTU to bypass auth. Fix options: (a) add `headers: HashMap<String, String>` field to `FidelityCheck` (preferred — enables general auth header injection); (b) add a dedicated fidelity-probe-bypass bearer token mechanism. Decision deferred to arch review. Flagged by S-6.07 test-writer during S-1.04 Red Gate.

**TD-S112-001** — `generate_confirmation_token()` in `add_sensor_spec.rs:186-198` hashes `SystemTime::now()` for the nonce component. In an in-process MCP server this is functionally safe (no network exposure), but is predictable under VM clock skew or if called in rapid succession. Fix: import `rand` crate, use `thread_rng().gen::<u64>()` as the nonce. Low exploitation risk in current deployment model; elevate to P1 if S-1.12 gains a remote-callable path.

**TD-S112-002** — `add_sensor_spec.rs:252` writes the sensor spec file with `std::fs::write(path, content)`. This is not crash-atomic: a power failure or SIGKILL mid-write leaves a truncated `.sensor.toml` that will cause the reload coordinator to emit `E-RELOAD-003` (parse error) on every subsequent reload until manually removed. Fix: write to `<path>.tmp`, call `fs::rename` to atomically replace. Rename is atomic on POSIX filesystems; add a Windows fallback using `MoveFileExW` with `MOVEFILE_REPLACE_EXISTING`.

**TD-WV1-02** — ADR-002 §8 specifies that the fidelity validator test file should be named `ac_N_fidelity_validator.rs` where N is the last AC number of the story. In S-6.10, AC numbering ends at AC-7 (reset state invariant), which is not the fidelity AC — resulting in the fidelity test landing in `tests/reset_state_invariants.rs` rather than an ADR-prescribed fidelity filename. Options: (a) amend ADR-002 to base fidelity test filename on AC semantic role rather than AC number; (b) reserve the last AC slot for fidelity in all DTU stories by convention. Flagged by S-6.10 test-writer during S-1.04 Red Gate.

**TD-S620-001** — Root `Cargo.toml` `[workspace] members` is missing 6 crates: prism-mcp, prism-ocsf, prism-security, prism-spec-engine, prism-storage, ocsf-proto-gen. These crates exist in the repo tree but are not registered as workspace members, which means `cargo build --workspace` and `cargo test --workspace` silently skip them. S-6.20 reduced the gap by registering the 4 DTU demo crates; the 6 remaining are pre-existing hygiene debt. Fix: single `Cargo.toml` edit adding the 6 paths. Deferred: non-blocking for Wave 1 deliverables; bundle with next maintenance sweep.

**TD-S620-002** — `prism-dtu-demo-server` cert generation hardcodes validity window 2024-01-01..2024-12-31, both dates already in the past as of 2026-04-23. Any TLS client doing validity-window checks will reject the cert immediately. Fix: replace hardcoded dates with `time::OffsetDateTime::now_utc()` as NotBefore and `+ Duration::days(365)` as NotAfter. Deferred: non-blocking for merge (demo server is test-only infra); MUST fix before any stakeholder demo.

**TD-S620-003** — The `--tls` CLI flag generates a self-signed cert and prints its fingerprint but never passes the resulting `RustlsConfig` to axum. The server still binds plain HTTP regardless of the flag. Fix: import `axum-server` crate, call `axum_server::bind_rustls(addr, config).serve(app.into_make_service())`. Deferred: functional HTTP paths fully tested (30/30 green over HTTP); TLS layer is aesthetic for demo purposes; fix before stakeholder demo.

**TD-S620-004** — `crates/prism-dtu-demo-server/` has no `README.md`. The spec (S-6.20 AC-13) references a README describing binary usage, flags, config format, and security model. Fix: write ~1-page README covering `--demo-config`, `--tls`, `--port` flags, TOML config schema, and the SHA-256 fingerprint verification workflow. Deferred: not a correctness issue; bundle with wave-2 documentation sweep.

**TD-S620-005** — `scripts/start-demo.sh` is referenced in the S-6.20 spec as the recommended one-command demo launcher but was not shipped with the PR. The spec describes it as wrapping the `prism-dtu-demo-server` binary invocation with default flags and printing the TLS fingerprint on startup. Fix: add the script under `scripts/`. Deferred: workaround is invoking the binary directly; bundle with wave-2 maintenance.

**TD-S620-006** — `print_cert_fingerprint()` in `prism-dtu-demo-server` encodes the cert DER as base64 then hex-encodes that base64 string. The spec (S-6.20 AC-12) calls for `sha256:<hex(sha256(DER))>`. Fix: `let digest = sha256::digest(&der_bytes); format!("sha256:{digest}")`. Deferred: fingerprint display is informational only; correctness matters for stakeholder identity verification; bundle with TD-S620-002/003 pre-demo fix sprint.

## Resolution History

| ID | Resolved In | Story | Resolution |
|----|------------|-------|------------|
| TD-WV0-05 | PR #28 (95c7ff15) | S-6.20 prereq | Mounted GET /dtu/health on NvdClone; GET /dtu/health + POST /dtu/reset on ThreatIntelClone. 3 new integration tests. Unblocks S-6.20 Task 3. |

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
