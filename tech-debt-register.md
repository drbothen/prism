---
document_type: tech-debt-register
producer: state-manager
version: "2.0"
last_updated: 2026-04-26T20:30:00
hotfix_3_pr_47: "pending — fix/post-merge-fuzz-kani-scope — registered TD-FUZZ-001/002/003 + TD-KANI-001"
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
wave_2_s201_merged: "PR #43 (0d24ab79) merged 2026-04-25 — prism-storage RocksDB foundation; registered TD-S201-001/002/003"
post_merge_cascade_closed: "2026-04-25 — 7-layer hotfix cascade (PRs #44,#45,#47,#48,#49) closed; post-merge.yml disabled to workflow_dispatch; TD-CICD-001 registered"
obs_001_resolved: "2026-04-25 — PR #51 (8eafb7b7) added default=[\"dtu\"] to prism-dtu-demo-server Cargo.toml; 255 tests restored; pre_wave_2_audit_findings_deferred → 0"
wave_2_s202_merged: "2026-04-25 — PR #52 (9de6b3d8) S-2.02 audit buffer + watchdog; 25 tests added; workspace 1039"
wave_2_s203_merged: "2026-04-25 — PR #53 (f13b5c76) S-2.03 decorators + internal tables; 19 tests added; workspace 1058; registered TD-S203-001/002/003"
wave_2_parallel_batch_merged: "2026-04-25 — PRs #55/56/57/58/54 (S-6.12/S-6.13/S-6.11/S-2.04/S-2.06); +183 tests; workspace 1241; registered TD-VSDD-001/002/003/004 + TD-S204-001 + TD-S612-001 + TD-S613-001"
wave_2_s205_merged: "2026-04-26 — PR #59 (c828e8af) S-2.05 specialized audit events; 35 tests added; workspace 1276; registered TD-S205-001"
wave_2_s208_merged: "2026-04-26 — PR #61 (0be11cd6) S-2.08 event tables; 92 tests added; workspace 1480; WAVE 2 CLOSED; registered TD-S208-001 + TD-S208-002"
wave_2_gate_pass_1_closed: "2026-04-26 — 4 fix-PRs (#62/#64/#63/#65) merged; 11/16 findings closed; 5 filed as TDs: TD-W2-MUTATE-001..004 + TD-W2-ULID-001 + TD-W2-PASS1-TOOLING-001; develop 0be11cd6 → 901dbbba; workspace 1480 → 1482"
wave_2_gate_pass_2_closed: "2026-04-26 — Pass 2 FINDINGS_OPEN (1M+4L+1residual); Architect KEEP on kani::Arbitrary (W2-P2-A-003); PO Option 1 on inherited_bcs schema (W2-P2-A-005); state-manager narrative reconciliation (W2-P2-A-004); W2-FIX-E in flight (A-001+A-002); TD-W2-CICD-SCOPE-001 + TD-VSDD-005 registered"
---

# Technical Debt Register

## Summary

| Priority | Count | Estimated Points |
|----------|-------|-----------------|
| P0 (next cycle) | 0 | 0 |
| P1 (within 3 cycles) | 2 | 5 |
| P2 (backlog) | 15 | 14 |
| P3 (post-feature follow-up) | 18 | 18 |

_Active items: 35. Wave 1.5 debt-reduction sprint (8 PRs, 2026-04-24) resolved 24 items total: 19 pre-existing Wave 1 TDs + 4 PR-A review followups (TD-WV05-PR33-001/002/003/004) + 1 PR-D important closure (IMPORTANT-001). Remaining P1: TD-S-1.07-01 (Wave 5 deferral — DO NOT CLOSE until prism-mcp crate lands). New P2 items from Wave 1.5 PR reviews: TD-WV15-PR35-001/002 + TD-WV15-PR36-001/002 + TD-WV15-PR40-001. Wave 2 S-2.01 PR #43: TD-S201-001/002 (P2) + TD-S201-003 (P1). Hotfix #3 (PR #47): TD-FUZZ-001/002/003 + TD-KANI-001 (P3). 2026-04-25: TD-CICD-001 (P2). Wave 2 S-2.03 PR #53: TD-S203-001/002/003 (P3). Wave 2 parallel batch 2026-04-25: TD-VSDD-001/002/003 (P2) + TD-VSDD-004 (P2) + TD-S204-001 (P3) + TD-S612-001 (P3) + TD-S613-001 (P3) — stub-as-impl anti-pattern prevention layers + mutation testing follow-ups. Wave 2 S-2.05 PR #59 2026-04-26: TD-S205-001 (P3) — QueryContext unification refactor. Wave 2 S-2.08 PR #61 2026-04-26: TD-S208-001 (P3) + TD-S208-002 (P2) — HTTP 429 mock test deferred + EventBufferStore cache concurrent-write validation. Wave 2 gate Pass 1 closure 2026-04-26: TD-W2-MUTATE-001/002/003/004 (P3) — retroactive mutation testing for 4 stub-as-impl stories; TD-W2-ULID-001 (P3) — replace 4-byte nanos suffix with real 16-byte ULID; TD-W2-PASS1-TOOLING-001 (P2) — adversary dispatch must include full tool access (process gap: Pass 1 ran with Read-only tools). Wave 2 gate Pass 2 closure 2026-04-26: TD-W2-CICD-SCOPE-001 (P2) — CI hotfix PR scope discipline (product-code-creep prevention); TD-VSDD-005 (P2) — vsdd-factory:adversary runtime tool-binding defect (only Read bound at dispatch)._

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
| TD-CICD-001 | Post-merge 7-layer cascade | `.github/workflows/post-merge.yml` speculatively authored without end-to-end validation. Failed 100% of develop pushes for unknown duration. 7-layer hotfix cascade (PRs #44, #45, #47, #48, #49) closed real defects but each fix exposed the next with no convergence. 5 architectural defects identified: (1) speculative fuzz harness inventory, (2) toolchain selection conflict, (3) zero shared infra with ci.yml, (4) no notification/consumption mechanism, (5) per-step time budget vs job timeout never reconciled. Disabled to workflow_dispatch pending redesign session with architect + adversary. ci.yml unaffected. | P2 | wave-2 | — | — | Wave 2/3 natural pause |
| TD-S201-001 | PR #43 review (R-001) | `remove_range` absent from `RocksStorageBackend` trait — BC-2.15.002 specifies `remove_range(domain, start_key, end_key)` using RocksDB `DeleteRange`. Story S-2.01 spec scoped this out (Task 7 omits it). Additive: add `remove_range` to `RocksStorageBackend` trait + `RocksDbBackend` impl + `InMemoryBackend` impl in a follow-up story or Wave 2 maintenance. | P2 | wave-2 | S-2.01 | S-2.01 follow-up | wave-2 maintenance |
| TD-S201-002 | PR #43 review (R-002) | `scan` and `scan_range` missing `limit` parameter — BC-2.15.002 specifies `scan(domain, prefix, limit)` and `scan_range(domain, start_key, end_key, limit)`. Without `limit`, a large-CF prefix scan loads all matching entries into memory (EC-15-007: 10k-key scenario). Story S-2.01 spec omits `limit`; implementation is faithful to spec. Add `limit: Option<usize>` to both methods in a follow-up. | P2 | wave-2 | S-2.01 | S-2.01 follow-up | wave-2 maintenance |
| TD-S201-003 | PR #43 review (R-004) | `set_dirty` stores u64 timestamp only, not full `DirtyBitEntry` — BC-2.15.005 specifies value = serialized `DirtyBitEntry { query_hash, query_source, started_at, consecutive_crashes }`. Current impl stores only a LE u64 timestamp; `check_dirty_on_startup()` returns `Vec<String>` (IDs only). Downstream stories implementing full recovery protocol (increment consecutive_crashes, add to watchdog denylist if >=3) will need a schema migration or new dirty-bit storage format. S-2.01 story spec scoped to `query_id: &str` deliberately. Extend in a follow-up story or Wave 2 story for dirty-bit protocol completion. | P1 | wave-2 | S-2.01 | S-4.01 or new S-2.x | wave-2 planning |
| TD-S203-001 | PR #53 stub-stage deviation | Story v1.3 says "extend ColumnType" but codebase has TWO `ColumnType` enums. Story v1.4 should clarify use of `InternalColumnType` (alias for `types::ColumnType`) for internal table schemas. Pure doc-correctness; no behavioral fix required. | P3 | wave-2 | S-2.03 | S-2.03 v1.4 cleanup | wave-2 retrospective |
| TD-S203-002 | PR #53 stub-stage deviation | Story v1.3 says "if `audit_read` field doesn't exist, add it" — based on misreading existing `ClientCapabilities` BTreeMap architecture. Story v1.4 should reference `is_allowed("audit.read")` pattern. Pure doc-correctness; no behavioral fix required. | P3 | wave-2 | S-2.03 | S-2.03 v1.4 cleanup | wave-2 retrospective |
| TD-S203-003 | PR #53 stub-stage deviation | Story v1.3 calls for `&[InternalTableDescriptor]` static; impl uses `OnceLock<Vec<InternalTableDescriptor>>` (semantically equivalent, heap-allocated lazy-init). Story v1.4 should align language to the heap-allocated lazy-init pattern. Pure doc-correctness; no behavioral fix required. | P3 | wave-2 | S-2.03 | S-2.03 v1.4 cleanup | wave-2 retrospective |
| TD-FUZZ-001 | Hotfix #3 / CI scope | `fuzz_prismql_parser` fuzz harness removed from post-merge.yml — target `fuzz/fuzz_targets/fuzz_prismql_parser.rs` never existed; PrismQL parser infrastructure is mid-development. Re-add to workflow when PrismQL parser crate (prism-query) ships and parser fuzz harness is authored in fuzz/. Owner: test-writer. | P3 | hotfix-3 | — | PrismQL parser epic | PrismQL parser milestone |
| TD-FUZZ-002 | Hotfix #3 / CI scope | `fuzz_alias_expansion` fuzz harness removed from post-merge.yml — target `fuzz/fuzz_targets/fuzz_alias_expansion.rs` never existed; alias expansion infrastructure not yet built. Re-add to workflow when alias expansion ships and fuzz harness is authored. Owner: test-writer. | P3 | hotfix-3 | — | alias expansion epic | alias expansion milestone |
| TD-FUZZ-003 | Hotfix #3 / CI scope | `fuzz_template_interpolation` fuzz harness removed from post-merge.yml — target `fuzz/fuzz_targets/fuzz_template_interpolation.rs` never existed; template interpolation infrastructure not yet built. Re-add to workflow when template interpolation ships and fuzz harness is authored. Owner: test-writer. | P3 | hotfix-3 | — | template interpolation epic | template interpolation milestone |
| TD-KANI-001 | Hotfix #3 / CI scope | `cargo kani --workspace` replaced with `-p prism-core -p prism-spec-engine -p prism-security -p prism-storage` — DTU crates require `--features=dtu` to compile and have no `#[kani::proof]` attributes, causing workspace-mode compilation to fail. As more crates add proofs, add additional `-p <crate>` flags or switch to `--workspace --features=...` once DTU feature-gate issue is resolved. Owner: devops-engineer + formal-verifier. | P3 | hotfix-3 | — | — | as proofs are added per-crate |
| TD-VSDD-001 | Wave 2 stub-as-impl anti-pattern (Layer 1) | Add anti-precedent guard text to vsdd-factory deliver-story SKILL.md and per-story-delivery.md so stub-architect agents don't copy stub-as-impl patterns from sibling crates. Identified in Wave 2 parallel batch: S-2.04/S-6.12/S-6.13 copied prism-dtu-armis and other Wave 1 DTU crate precedents. 3 of 5 stories affected. | P2 | wave-2-parallel-batch | — | vsdd-factory plugin | Wave 2/3 natural pause |
| TD-VSDD-002 | Wave 2 stub-as-impl anti-pattern (Layer 2) | Add Red Gate density check to vsdd-factory per-story-delivery.md as a mandatory orchestrator gate between Step 3 (Red Gate) and Step 4 (Implementer). Threshold: RED_TESTS / TOTAL_NEW_TESTS ≥ 0.5 unless documented. Catches stub-as-impl before implementer dispatch becomes no-op. | P2 | wave-2-parallel-batch | — | vsdd-factory plugin | Wave 2/3 natural pause |
| TD-VSDD-003 | Wave 2 stub-as-impl anti-pattern (Layer 3) | Add `tdd_mode: strict \| facade` frontmatter field to vsdd-factory story template; route facade-mode stories through different per-story-delivery flow with explicit acknowledgment and mutation testing gate. | P2 | wave-2-parallel-batch | — | vsdd-factory plugin | Wave 3 kickoff |
| TD-VSDD-004 | Wave 2 stub-as-impl anti-pattern (Layer 4) | Wire mutation testing gate (cargo mutants ≥ 80% kill rate) into vsdd-factory wave-gate skill for facade-mode stories (tdd_mode: facade). Validates that stub-as-impl tests catch real regressions before wave gate closes. | P2 | wave-2-parallel-batch | — | vsdd-factory plugin | Wave 2 gate |
| TD-S204-001 | S-2.04 mutation testing | Run `cargo mutants -p prism-audit` at Wave 2 gate to validate test robustness given S-2.04 stub-as-impl pattern. 54 of 72 tests are GBD (green-by-design); mutation coverage needs validation before wave gate. | P3 | wave-2-parallel-batch | S-2.04 | prism-audit | Wave 2 gate |
| TD-S612-001 | S-6.12 mutation testing | Run `cargo mutants -p prism-dtu-pagerduty` at Wave 2 gate. All 17 tests are stub-as-impl; mutation kill rate validates whether tests catch behavioral regressions. | P3 | wave-2-parallel-batch | S-6.12 | prism-dtu-pagerduty | Wave 2 gate |
| TD-S613-001 | S-6.13 mutation testing | Run `cargo mutants -p prism-dtu-jira` at Wave 2 gate. All 28 tests are stub-as-impl; mutation kill rate validates whether tests catch behavioral regressions. | P3 | wave-2-parallel-batch | S-6.13 | prism-dtu-jira | Wave 2 gate |
| TD-S205-001 | S-2.05 spec-vs-impl QueryContext gap | Story v1.3 references `prism_core::QueryContext` but this type does not exist in code. Stub-author created 3 local interim context types: `RequestingContext`, `FlagEvalContext`, `TokenEventContext`. Tests and impl anchor to these interim types. Refactor: unify the 3 local context types into a single `prism_core::QueryContext`; align story v1.4 to reflect actual API. Pure refactor — no behavioral change required. | P3 | wave-2 | S-2.05 | prism-core / prism-audit | story v1.4 refactor |
| TD-S208-001 | S-2.08 AC-6 HTTP 429 WARN+continue mock-adapter test deferred | AC-6 requires HTTP 429 responses from the sensor API to be logged as WARN and execution to continue (BC-2.01.014 covers retry/backoff). A full mock-adapter test for this behavior was deferred during S-2.08 implementation because the mock sensor adapter infrastructure is not yet present. Add the test when sensor mock infrastructure lands (likely Wave 3 or early Wave 4). | P3 | wave-2 | S-2.08 | prism-sensors / prism-query | sensor mock infra milestone |
| TD-S208-002 | S-2.08 EventBufferStore in-memory cache concurrent-write validation | EventBufferStore uses an in-memory `Mutex<BTreeMap>` as a write-through cache layer added for test consistency; production path goes through the storage backend. Verify there is no behavioral divergence between cache and backend under concurrent write/scan workloads at wave gate. Particularly: ensure scan operations do not miss buffered-but-not-yet-flushed entries under concurrent writers. | P2 | wave-2 | S-2.08 | prism-sensors / prism-query | Wave 2 integration gate |
| TD-W2-MUTATE-001 | Wave 2 gate Pass 1 finding W2-P1-A-007 | S-2.04 shipped with RED ratio 25% (18/72), below the Layer-2 ≥0.5 threshold. The implementer disclosed this and recommended mutation testing at wave gate. Run `cargo mutants -p prism-audit` before Wave 3 close to compensate for the weakened TDD signal. Origin: W2-P1-A-007. | P3 | wave-2-gate-pass-1 | S-2.04 | prism-audit | Wave 3 close |
| TD-W2-MUTATE-002 | Wave 2 gate Pass 1 finding W2-P1-A-003 | S-6.12 shipped with RED ratio 0/17 (0%) — stub-as-impl; entire implementation green-by-design at stub commit. Compensating mutation coverage required. Run `cargo mutants -p prism-dtu-pagerduty` before Wave 3 close. Origin: W2-P1-A-003. | P3 | wave-2-gate-pass-1 | S-6.12 | prism-dtu-pagerduty | Wave 3 close |
| TD-W2-MUTATE-003 | Wave 2 gate Pass 1 finding W2-P1-A-003 | S-6.13 shipped with RED ratio 0/28 (0%) — stub-as-impl pattern. Compensating mutation coverage required. Run `cargo mutants -p prism-dtu-jira` before Wave 3 close. Origin: W2-P1-A-003. | P3 | wave-2-gate-pass-1 | S-6.13 | prism-dtu-jira | Wave 3 close |
| TD-W2-MUTATE-004 | Wave 2 gate Pass 1 finding W2-P1-A-016 | S-6.11 shipped with RED ratio 1/14 (~7%) — escaped Layer-2 enforcement (introduced mid-cycle). Compensating mutation coverage required to validate behavioral fidelity. Run `cargo mutants -p prism-dtu-slack` before Wave 3 close. Origin: W2-P1-A-016. | P3 | wave-2-gate-pass-1 | S-6.11 | prism-dtu-slack | Wave 3 close |
| TD-W2-ULID-001 | Wave 2 gate Pass 1 finding W2-P1-A-005 | EventBufferStore event keys use a 4-byte `subsec_nanos` suffix instead of a 16-byte ULID. PR-FIX-W2-A aligned the docs with the actual 4-byte impl, but the underlying collision risk under sustained ingest remains. Workspace already includes `uuid` v7 in multiple crates; consider taking a real ULID dep or adding a sequence counter to disambiguate within-microsecond writes. Files affected: `crates/prism-sensors/src/event_buffer.rs`. Origin: W2-P1-A-005. | P3 | wave-2-gate-pass-1 | S-2.08 | prism-sensors | Before high-throughput sensor onboarding (Wave 3 or later) |
| TD-W2-PASS1-TOOLING-001 | Wave 2 gate Pass 1 process-gap disclosure | The Pass 1 adversary ran with Read-only tool access (no Glob/Grep/Bash), which prevented full verification of policies POL-1, POL-2, POL-5, POL-6, POL-7, POL-8, POL-9 (all index-file enumeration policies). Pass 2+ must dispatch with full tool access. Investigate root cause: agent definition declares `Tools: Read, Grep, Glob, Bash` but only Read was operative in this session. May be a session-specific harness issue or a bug in the orchestrator's adversary dispatch path. Affected agent: vsdd-factory:adversary. | P2 | wave-2-gate-pass-1 | — | orchestrator / adversary dispatch | Before Pass 2 of Wave 2 gate (immediate) |
| TD-W2-CICD-SCOPE-001 | Wave 2 gate Pass 2 finding W2-P2-A-003 + Architect decision | CI hotfix PR scope discipline. PR #45 (`7903da15`) was a CI hotfix nominally scoped to workflow files but added `#[cfg_attr(kani, derive(kani::Arbitrary))]` to `crates/prism-core/src/case.rs:50`. Architect decision: KEEP the change (load-bearing for VP-005/006/051 proofs). However, the change should have been in a separate product PR. Establish a CI-hotfix PR checklist: diffs must be limited to `.github/workflows/**`, `fuzz/Cargo.toml`, and test-fixture files only. Any product-code change (even a one-line attribute macro) requires a full story/feature PR. ADR-004 stub created to retroactively document the kani::Arbitrary policy for types used in proofs. | P2 | wave-2-gate-pass-2 | — | orchestrator + pr-manager (enforce via review) | Before next CI hotfix burst |
| TD-VSDD-005 | Wave 2 gate Pass 2 adversary dispatch failure | vsdd-factory:adversary subagent has a runtime tool-binding defect. Agent definition declares `Tools: Read, Grep, Glob` but at runtime only `Read` is bound. Pass 2 had to fall back to general-purpose-as-adversary workaround. This blocks the canonical vsdd-factory adversarial discipline. Accumulating alongside earlier session Skill-tool-empty-body bug (fix-prompt deleted per user request). These are vsdd-factory plugin-level defects to address during the housekeeping pause before Wave 3. | P2 | wave-2-gate-pass-2 | — | vsdd-factory plugin maintainer (separate session) | Before next adversarial review (Wave 3 gate at latest) |

### Wave 1.5 PR Review Followup Detail (Active)

**TD-WV15-PR35-001** — PR B (config/workspace hardening) review flagged that the FidelityValidator test assertions in Wave 1 clone fidelity tests are embedded in integration test scaffolding (real server started per test). A pure unit test approach exercising check logic against a mock server would run faster and be less flaky. No correctness issue. Deferred to wave-2 maintenance sweep.

**TD-WV15-PR35-002** — PR B added `#[serde(deny_unknown_fields)]` to NvdConfigPayload. The NVD API response envelope has optional extended fields in some response variants not exercised by current fixture data; deny_unknown_fields may fail on live NVD API responses if those optional fields are encountered. Needs audit of NVD API schema scope before using real NVD endpoint. Deferred: does not affect DTU clone behavior (fixtures controlled by test harness). Wave-2 maintenance.

**TD-WV15-PR36-001** — PR C added `src.ip().is_loopback()` check in SyslogReceiver. There is a TOCTOU window between the loopback check and the packet processing: a non-loopback packet received in the window between bind and accept could pass through. Add an inline comment in SyslogReceiver::recv() documenting this known TOCTOU limitation and why it is acceptable (DTU test context only; loopback-only network; no production exposure). Deferred: documentation-only; no correctness impact on test infrastructure.

**TD-WV15-PR36-002** — PR C refactored S-1.09 consume() to use eager removal. One test function (`test_consume_sets_consumed_flag`) retained its legacy name that references "sets consumed flag" — behavior no longer accurate after the remove() refactor. Rename to `test_consume_removes_entry`. Deferred: no correctness impact; cosmetic rename only.

**TD-WV15-PR40-001** — PR F added manual `Default` impl for `FidelityCheck` (required for the headers field to default to empty vec). The struct now qualifies for `#[derive(Default)]` if all field types implement `Default`. Switch from manual impl to derive macro for conciseness and maintenance clarity. Deferred from PR F scope; cosmetic only.

### TD-CICD-001 — Post-Merge Verification Workflow Redesign (Active P2)

**Severity**: P2 (medium — workflow disabled, no functional impact, redesign needed)
**Status**: OPEN
**Opened**: 2026-04-25
**Owner**: TBD (architect lead, devops-engineer + adversary support)

**Problem**

`.github/workflows/post-merge.yml` was speculatively authored without end-to-end validation. Failed 100% of develop pushes for unknown duration. 7-layer hotfix cascade (PRs #44, #45, #47, #48, #49 + layer 7 protoc) closed real defects but each fix exposed the next, with no convergence. False cascade-closer determinations made 3 of 3 times. Disabled to `workflow_dispatch` only, pending redesign.

**5 architectural defects discovered (NOT 7 sequential bugs)**

1. **Speculative fuzz harness inventory** — Workflow referenced 6 fuzz targets; only 3 exist as `[[bin]]` entries. Aspirational targets (`fuzz_prismql_parser`, `fuzz_alias_expansion`, `fuzz_template_interpolation`) tracked separately as TD-FUZZ-001/002/003. Indicates: no manifest discipline; workflow drift from reality.

2. **Toolchain selection conflict** — `rust-toolchain.toml` pins `channel = "stable"` at workspace root. `dtolnay/rust-toolchain` action installs nightly but `rust-toolchain.toml` overrides. Required `RUSTUP_TOOLCHAIN: nightly` env var as escape hatch (PR #45). Indicates: no documented strategy for workflows that need a different toolchain than the workspace default.

3. **Zero shared infrastructure with ci.yml** — ci.yml installs `arduino/setup-protoc` in 4 jobs (clippy, test matrix, test-no-default-features, semver); post-merge.yml has 0 protoc installs. prism-ocsf has `build.rs` requiring protoc — both workflows compile prism-ocsf but only ci.yml knows. Indicates: no composite action / reusable workflow factoring out common setup.

4. **No notification / consumption mechanism** — Artifact uploads use `if-no-files-found: ignore` leading to silent failure. No Slack, email, issue creation, dashboard, or on-call rotation. Workflow ran red for unknown duration; nobody read artifacts. Indicates: workflow is aspirational, not load-bearing.

5. **Per-step time budget vs job timeout unreconciled** — Kani per-proof timeout: 300s (5 min). Job `timeout-minutes`: 120 (2 hours). Fuzz steps: 6 x `-max_total_time=1800` = 3 hours theoretical max. Math never computed; nobody asked "can the workflow finish in its budget?" Indicates: no time-budget design discipline.

**Resolution criteria (for the redesign session)**

- [ ] Manifest of expected fuzz harnesses (one source of truth, diffed against `fuzz/Cargo.toml` at workflow-fmt step)
- [ ] Manifest of expected Kani proofs + per-crate scope (replaces `--workspace` wildcard)
- [ ] Composite action or reusable workflow extracted from ci.yml + post-merge.yml common setup (checkout, toolchain, protoc)
- [ ] Time budget designed: per-step timeouts x max parallel <= job timeout-minutes
- [ ] Notification mechanism: failures must page a human (Slack, GitHub issue, or equivalent)
- [ ] Disabled-state escape hatch: `workflow_dispatch` retained for manual investigation runs
- [ ] Re-enable: PR re-runs the workflow on develop and confirms green status before flipping `on: push` back on

**Redesign trigger**

Wave 2 natural pause point. Estimated session: 1-2 days with architect + adversary + devops-engineer.

**Evidence (cascade artifacts)**

- PR #44: https://github.com/drbothen/prism/pull/44 (workflow YAML + Kani CLI)
- PR #45: https://github.com/drbothen/prism/pull/45 (RUSTUP_TOOLCHAIN + Arbitrary)
- PR #47: https://github.com/drbothen/prism/pull/47 (fuzz target alignment + Kani -p)
- PR #48: https://github.com/drbothen/prism/pull/48 (--target gnu)
- PR #49: https://github.com/drbothen/prism/pull/49 (fuzz/Cargo.toml deps)

**Related TDs**

- TD-FUZZ-001/002/003: re-add aspirational fuzz harnesses when their underlying features ship
- TD-KANI-001: expand `cargo kani -p` list as more crates add proofs

### TD-W2-CICD-SCOPE-001 — CI Hotfix PR Product-Code-Creep Discipline (Active P2)

**Severity**: P2 (medium — process discipline gap; no functional regression)
**Status**: OPEN
**Opened**: 2026-04-26
**Owner**: orchestrator + pr-manager (enforce via review)

**Problem**

PR #45 (`7903da15`) was nominally a CI hotfix scoped to `.github/workflows/post-merge.yml`. However, the diff also includes `crates/prism-core/src/case.rs:50`:

```rust
#[cfg_attr(kani, derive(kani::Arbitrary))]
pub enum CaseStatus {
```

This is a product-code change (attribute macro on a production type). It landed in a CI hotfix PR without a story, without a Red Gate stub, and without story-level review. The change itself is **correct and load-bearing** (VP-005, VP-006, VP-051 Kani proofs require `CaseStatus: kani::Arbitrary`). Architect decision 2026-04-26: KEEP the change.

**Root cause**

No checklist enforcement on CI hotfix PR scope. Diff review did not flag the `case.rs` line as out-of-scope.

**Resolution criteria**

Establish a CI-hotfix PR checklist enforced at pr-manager dispatch:
- PR diff MUST be limited to `.github/workflows/**`, `fuzz/Cargo.toml`, and test-fixture files
- Any product-code change (even a one-line attribute macro) requires a full story/feature PR
- Retroactive policy documented in ADR-004 for kani::Arbitrary specifically

**ADR reference**: ADR-004 stub created 2026-04-26 — `.factory/specs/architecture/decisions/ADR-004-kani-arbitrary-policy.md`

**Origin**: W2-P2-A-003 finding; Architect decision KEEP (load-bearing for VP-005/006/051).

### TD-VSDD-005 — vsdd-factory:adversary Runtime Tool-Binding Defect (Active P2)

**Severity**: P2 (medium — blocks canonical adversarial discipline; workaround available)
**Status**: OPEN
**Opened**: 2026-04-26
**Owner**: vsdd-factory plugin maintainer (separate session)

**Problem**

The `vsdd-factory:adversary` subagent has a runtime tool-binding defect. The agent definition declares `Tools: Read, Grep, Glob` but at runtime only `Read` is bound. This caused:

- Wave 2 gate Pass 1: adversary ran Read-only; POL-1/2/5/6/7/8/9 not fully verified (filed as TD-W2-PASS1-TOOLING-001)
- Wave 2 gate Pass 2: adversary could not be dispatched; fallback to general-purpose-as-adversary

This is the second vsdd-factory plugin-level defect identified in this session (an earlier Skill-tool-empty-body bug was discovered and its fix-prompt deleted per user request).

**Pattern**

These are accumulating plugin-level defects in the vsdd-factory plugin suite. They reduce the reliability of the automated adversarial loop. Until resolved, the workaround is to dispatch `general-purpose` with the adversary role instructions inline.

**Resolution criteria**

- Identify whether tool binding fails at agent-definition parse time or at skill-invocation time
- Fix the vsdd-factory:adversary skill definition to correctly bind Read + Grep + Glob at runtime
- Verify with a test dispatch before Wave 3 gate begins
- Consider adding a tool-verification preamble to every adversarial pass (Pass 2 did this manually — adopted as a permanent check)

**Workaround** (immediate): Use `general-purpose` agent with adversary instructions + `tools_available: Read, Grep, Glob, Bash` preamble until fixed. Verified working in Pass 2.

### S-2.03 Spec-vs-Impl Deviations (Active P3 — Doc Cleanup for v1.4)

**TD-S203-001** — Story v1.3 says "extend ColumnType" but prism-storage has TWO `ColumnType` enums (one in `prism-core::types` and one defined locally). The implementation correctly uses `InternalColumnType` as a type alias for `types::ColumnType` (the core enum). Story v1.4 should clarify this alias pattern explicitly so future stub authors do not repeat the ambiguity. No behavioral fix; pure doc-correctness.

**TD-S203-002** — Story v1.3 says "if `audit_read` field doesn't exist, add it" to `ClientCapabilities`. This was based on a misreading of the existing `ClientCapabilities` BTreeMap architecture — the correct pattern is `is_allowed("audit.read")` which queries the BTreeMap dynamically. Story v1.4 should reference the `is_allowed()` pattern explicitly. No behavioral fix; pure doc-correctness.

**TD-S203-003** — Story v1.3 calls for a `&[InternalTableDescriptor]` static (a Rust `static` with a slice reference). The implementation uses `OnceLock<Vec<InternalTableDescriptor>>` (semantically equivalent: heap-allocated lazy-init that returns a slice reference). The `OnceLock` pattern is architecturally correct for prism-storage given the dynamic initialization requirements. Story v1.4 should align language to the `OnceLock<Vec<T>>` lazy-init pattern. No behavioral fix; pure doc-correctness.

All three items are tracked under D-015 (Decisions Log). Source: stub-stage discovery during S-2.03 TDD (before Red Gate). All implementation choices preserved as architectural decisions.

### Hotfix #3 Fuzz + Kani Detail (Active P3)

**TD-FUZZ-001** — post-merge.yml listed `cargo fuzz run fuzz_prismql_parser` but `fuzz/fuzz_targets/fuzz_prismql_parser.rs` never existed. The prior toolchain failures (hotfix #1 and #2) masked this by aborting the fuzz job before target lookup. PrismQL parser (prism-query crate) is planned but not yet implemented. When the parser ships, the test-writer agent should author a fuzz harness in `fuzz/fuzz_targets/fuzz_prismql_parser.rs` covering token boundary injection and malformed PQL syntax, register the `[[bin]]` in `fuzz/Cargo.toml`, and restore this step in post-merge.yml.

**TD-FUZZ-002** — post-merge.yml listed `cargo fuzz run fuzz_alias_expansion` but `fuzz/fuzz_targets/fuzz_alias_expansion.rs` never existed. Alias expansion infrastructure (PrismQL macro/alias system) is planned but not yet built. When alias expansion ships, the test-writer agent should author a harness covering recursive alias cycles and injection vectors, register it in `fuzz/Cargo.toml`, and restore this step in post-merge.yml.

**TD-FUZZ-003** — post-merge.yml listed `cargo fuzz run fuzz_template_interpolation` but `fuzz/fuzz_targets/fuzz_template_interpolation.rs` never existed. Template interpolation infrastructure (sensor spec template system) is planned but not yet built. When it ships, the test-writer agent should author a harness covering format string injection and malformed template syntax, register it in `fuzz/Cargo.toml`, and restore this step in post-merge.yml.

**TD-KANI-001** — `cargo kani --workspace` fails because DTU crates (e.g. prism-dtu-cyberint) require `--features=dtu` to compile under Kani and have no `#[kani::proof]` attributes. Scoped to 4 crates with proofs: prism-core (6 proofs: case_status, cursor, credential_name, capability, case_status_exhaustive, tenant_id), prism-spec-engine (infusion_spec), prism-security (token_proofs, feature_flag_proof), prism-storage (crash_recovery). As future stories add proofs to other crates, add `-p <crate>` flags to the `cargo kani` command in post-merge.yml. Long-term: once DTU feature-gate compilation under Kani is resolved, consider reverting to `--workspace`.

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
