---
document_type: tech-debt-register
producer: state-manager
version: "2.3"
last_updated: 2026-05-11T04:00:00
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
wave_2_gate_pass_3_closed: "2026-04-26 — Pass 3 CONVERGED (0 findings); first clean pass of Wave 2 gate"
wave_2_gate_pass_4_closed: "2026-04-26 — Pass 4 CONVERGED (0 findings); parallel with Pass 5"
wave_2_gate_pass_5_closed: "2026-04-26 — Pass 5 FINDINGS_OPEN (3 LOW); W2-P5-A-001 redaction doc drift; W2-P5-A-002 stale todo!() narrative in 6 test files; W2-P5-A-003 S-2.06 RED ratio gap; PR-FIX-W2-F in flight (A-001+A-002); TD-W2-MUTATE-005 filed for A-003"
wave_2_gate_step_h_mutation_testing: "2026-04-27 — CONDITIONAL_PASS. prism-audit 80% (5 missed: Tower poll_ready/call + to_json x2 + resolve_host); prism-dtu-pagerduty/jira/slack 0% (115 missed, structural fidelity-only pattern); prism-sensors-scoped KILLED (rocksdb-sys C++ baseline 17min/0 mutants, Option B→Option C escalated); TD-W2-MUTATE-AUDIT-001 + TD-DTU-MUTATE-COVERAGE-001 filed (P3); TD-W2-MUTATE-005 escalated P3→P2 (Option C). TD count 53→55."
wave_2_gate_step_c_code_review: "2026-04-26 — 14 findings (2 HIGH: WGC-W2-001 S-2.05 audit emitters silently non-functional, WGC-W2-002 evict_expired no backend scan; 6 MEDIUM; 6 LOW); filed TD-W2-CODE-MED-001..006, TD-W2-CODE-LOW-001..006"
w2_fix_h_merged: "2026-04-27 — PR #68 (bc65d691) W2-FIX-H: audit emitter persistence + evict_expired backend scan; +7 tests; workspace 1489; WGC-W2-001 + WGC-W2-002 CLOSED; filed TD-W2-FIX-H-001 (lefthook fmt hook) + TD-W2-FIX-H-002 (known_prefixes post-restart false-negative)"
wave_2_gate_step_d_security_review: "2026-04-26 — 8 findings (0 CRITICAL, 2 HIGH: WGS-W2-001 AQL injection, WGS-W2-002 bearer tokens cleartext; 3 MEDIUM; 3 LOW); filed TD-W2-SEC-MED-001..003, TD-W2-SEC-LOW-001..003"
wave_2_gate_step_e_consistency_validation: "2026-04-26 — CONDITIONAL_FAIL: WGCV-W2-001 CRITICAL (11 stories status:draft), WGCV-W2-002 HIGH (S-2.01 annotation gap); filed TD-W2-CONS-001, TD-W2-DOC-001"
w2_fix_k_merged: "2026-04-27 — PR #71 (cf4fb34b) W2-FIX-K: strip token_id from generated/expired audit entries + replace tautology test; prism-audit 111→113 tests; workspace 1499; P7 HIGH-001 + HIGH-003 CLOSED; filed TD-W2-FIXK-001 (process-gap: validate-consistency tautology-detector + BC-TV field-exclusion check)"
w2_fix_l_merged: "2026-04-27 — PR #72 (37c620f7) W2-FIX-L: AQL HIGH-002 validator bypass closure; match_indices multi-select + blanket single-quote rejection at armis.rs:212-232/:257-263; +6 tests; workspace 1499→1505; P7 HIGH-002 CLOSED"
wave_2_integration_gate_converged: "2026-04-27 — Pass 8 CLEAN (0C+0H+0M+1L); all P7 HIGH closures verified; 1505 tests passing; WAVE 2 GATE CLOSED. TD-W2-FIXK-002 filed for Pass 8 P8-001. TD count 56→57."
s_3_0_01_merged: "2026-04-28 — PR #73 (6696e374) S-3.0.01: lefthook fmt hook — cargo fmt --all --check; TD-W2-FIX-H-001 CLOSED; 4 TAP checks added; LEFTHOOK=0 bypass workaround no longer needed; first Wave 3 PR merged"
s_3_0_02_merged: "2026-04-28 — PR #74 (373baf78) S-3.0.02: DTU_DEFAULT_MODE registry in prism-core; BC-3.2.005 implemented; VP-091..094 GREEN; +17 tests; S-3.3.01 unblocked; filed TD-W3-S-3.0.02-DOC-001 (P3 suggestion — marker comment text wording in story v0.6)"
s_3_7_00_merged: "2026-04-29 — PR #75 (79f67c93) S-3.7.00: schema derivation artifacts (.references/schemas/{armis,crowdstrike}/types.rs + DERIVATION.md); BC-3.4.002/BC-3.4.003 implemented; VP-112/VP-114 GREEN; 25 TAP shell assertions; .gitignore narrow exception; 1 review cycle (0 findings); no TDs filed"
s_3_7_01_merged: "2026-04-29 — PR #76 (0bb7735d) S-3.7.01: Archetype/GenOpts foundation in prism-dtu-common behind fixture-gen feature; BC-3.4.001/BC-3.4.003 implemented; VP-108/VP-111/VP-115/VP-116/VP-117 GREEN; 39 gated integration tests; 2 review cycles (F-001 BLOCKING resolved at 82473db3; F-002 doc; F-003→TD); filed TD-W3-S-3.7.01-001 (F-003: bare constants in pagination.rs)"
s_3_1_01_merged: "2026-04-29 — PR #81 (39125a3e) S-3.1.01: OrgId(Uuid v7) newtype; BC-3.1.001 implemented; +11 tests; OrgId foundation for E-3.1 multi-tenant chain"
s_3_6_02_merged: "2026-04-29 — PR #84 (73d1c348) S-3.6.02: HS-007 holdout refresh; +5 tests; HS-007 anchored to current BCs"
s_3_6_01_merged: "2026-04-29 — PR #83 (36a40f59) S-3.6.01: HS-006 holdout refresh; +5 tests; HS-006 anchored to current BCs"
s_3_5_01_merged: "2026-04-29 — PR #82 (c4287aef) S-3.5.01: crate-layout sweep; BC-3.7.001 implemented; +12 Rust + 24 TAP tests; 2 force-push rebases (sibling-merge pattern D-148); filed TD-S3501-W3-001 (pre-existing clippy errors in sensor DTU crates)"
---

# Technical Debt Register

## Summary

| Priority | Count | Estimated Points |
|----------|-------|-----------------|
| P0 (next cycle) | 0 | 0 |
| P1 (within 3 cycles) | 4 | 10 |
| P2 (backlog) | 25 | 24 |
| P3 (post-feature follow-up) | 50 | 50 |

_Active items: 78 (77 prior + 1: TD-S-PLUGIN-PREREQ-A-006 P3 filed 2026-05-11 — cross-newtype audit pub-API validation-bypass entry points (OrgSlug::new_unchecked), F-LP7-LOW-002 source, S-PLUGIN-PREREQ-A pass-7; defer to post-PREREQ-A maintenance or cross-newtype hardening story. Prior 77: 76 prior + 1: TD-S-PLUGIN-PREREQ-A-005 P3 filed 2026-05-11 — EXPLAIN silent-skip UX inconsistency vs E-QUERY-031, explain.rs:665 TODO, S-PLUGIN-PREREQ-A pass-5 fix-burst-5; defer to PLUGIN-MIGRATION-001-B or successor. Prior 76: 75 prior + 1: TD-S-PLUGIN-PREREQ-A-004 P1 filed 2026-05-11 — boot.rs step8 AdapterRegistry assertion deferred until step8 wired, S-PLUGIN-PREREQ-A pass-3 fix-burst-3; tracked as successor-story obligation. Prior 75: 74 prior + 1: TD-S-PLUGIN-PREREQ-A-003 P1 filed 2026-05-11 — runtime extensibility for WriteToolInvalidationMap, S-PLUGIN-PREREQ-A fix-burst-2; defer to PREREQ-E. Prior 74: 73 prior + 1: TD-S-PLUGIN-PREREQ-A-002 P1 filed 2026-05-11 — sentinel-nil OrgId in WriteDispatcher, S-PLUGIN-PREREQ-A fix-burst-1; depends on W3-FIX-S307-002 unblock. Prior 73: 57 prior + 9 S-3.04 story TDs filed 2026-05-07 per F-005 closure: TD-S304-001 P2, TD-S304-002 P2, TD-S304-003 P3, TD-S304-AUDIT-001 P3, TD-S304-FILEPERMS-001 P3, TD-S304-FUZZ-001 P3, TD-S304-TMPNAME-001 P3, TD-S304-VISIBILITY-001 P3, TD-S304-VP013-001 P3 — per-file detail in `.factory/tech-debt/TD-S304-*.md`. Prior 57: 70 prior − 13 VSDD/methodology items extracted to vsdd-plugin-tech-debt.md 2026-05-02. Prior 70: 69 prior + 1: TD-VSDD-034 filed P3 suggestion 2026-05-02 — gate-step pass-N completeness policy for non-impacted steps, surfaced by PG-53-001 in pass-53. Prior 69: 68 prior + 1: TD-W3-QUOTA-SOAK-001 filed P3 suggestion 2026-05-02 — cross-tenant API quota soak test gap, surfaced by holdout-evaluator pass-5 BELOW_BAR-002 HS-003-06. Prior 68: 67 prior + 1: TD-W3-CT-EQ-COVERAGE-001 filed P3 suggestion 2026-05-02 — non-DTU non-constant comparison audit pattern, surfaced by PR #125 R1-001 fc467937. Prior 67: 66 prior + 1: TD-W3-CI-MSVC-001 filed P3 observation 2026-04-29 — Windows MSVC flake in prism-sensors semaphore test, surfaced by S-3.2.05 CI Run 1, non-blocking. Prior 66: 65 prior + 1: TD-S3501-W3-001 filed P3 suggestion 2026-04-29 — pre-existing clippy errors in sensor DTU crates, surfaced by S-3.5.01 crate-layout sweep, workspace-wide clippy gate gap. Prior 65: 64 prior + 1: TD-W3-S-3.7.01-001 filed P3 suggestion 2026-04-29 — bare constants in pagination.rs, F-003 from PR #76 review, non-blocking. Prior 64: 63 prior + 1: TD-W3-S-3.0.02-DOC-001 filed P3 suggestion 2026-04-28 — marker comment text wording in story v0.6, non-blocking. Prior 63: 64 prior − 1: TD-W2-FIX-H-001 CLOSED PR #73 2026-04-28; 64 = 61 prior +3: TD-W4-AUDIT-QUERY-REPLAY-001 P2, TD-W4-LOG-FORWARDING-001 P2, TD-W4-ALERTING-WORKFLOWS-001 P2 — Wave 4+ capability TDs filed at Phase 3.A approval gate Q1 per D-136. Prior 58: +1: TD-VSDD-029 P3 state-manager.md parallel-changelog symmetry guardrail — vsdd-factory plugin separate-repo; M-35-001 Pass 35 process-gap codification. Prior 57: unchanged — TD-VSDD-019 is a process-gap item tracked in STATE.md Process Improvements Backlog, not counted as open TD per TD-VSDD-014..018 convention. Prior 57: 56 prior + 1 from Pass 8: TD-W2-FIXK-002 P3 BC-named tests assert only result.is_ok() — BC postcondition never verified against backend). Prior 56: (55 prior + 1 from W2-FIX-K: TD-W2-FIXK-001 P3 process-gap validate-consistency tautology-detector + BC-TV field-exclusion check). Prior 55: (53 prior + 2 from W2 gate-step-h mutation testing: TD-W2-MUTATE-AUDIT-001 P3 + TD-DTU-MUTATE-COVERAGE-001 P3; TD-W2-MUTATE-005 status changed Option B → Option C — not new). Wave 1.5 debt-reduction sprint (8 PRs, 2026-04-24) resolved 24 items total: 19 pre-existing Wave 1 TDs + 4 PR-A review followups (TD-WV05-PR33-001/002/003/004) + 1 PR-D important closure (IMPORTANT-001). Remaining P1: TD-S-1.07-01 (Wave 5 deferral — DO NOT CLOSE until prism-mcp crate lands). New P2 items from Wave 1.5 PR reviews: TD-WV15-PR35-001/002 + TD-WV15-PR36-001/002 + TD-WV15-PR40-001. Wave 2 S-2.01 PR #43: TD-S201-001/002 (P2) + TD-S201-003 (P1). Hotfix #3 (PR #47): TD-FUZZ-001/002/003 + TD-KANI-001 (P3). 2026-04-25: TD-CICD-001 (P2). Wave 2 S-2.03 PR #53: TD-S203-001/002/003 (P3). Wave 2 parallel batch 2026-04-25: TD-VSDD-001/002/003 (P2) + TD-VSDD-004 (P2) + TD-S204-001 (P3) + TD-S612-001 (P3) + TD-S613-001 (P3) — stub-as-impl anti-pattern prevention layers + mutation testing follow-ups. Wave 2 S-2.05 PR #59 2026-04-26: TD-S205-001 (P3) — QueryContext unification refactor. Wave 2 S-2.08 PR #61 2026-04-26: TD-S208-001 (P3) + TD-S208-002 (P2) — HTTP 429 mock test deferred + EventBufferStore cache concurrent-write validation. Wave 2 gate Pass 1 closure 2026-04-26: TD-W2-MUTATE-001/002/003/004 (P3) — retroactive mutation testing for 4 stub-as-impl stories; TD-W2-ULID-001 (P3) — replace 4-byte nanos suffix with real 16-byte ULID; TD-W2-PASS1-TOOLING-001 (P2) — adversary dispatch must include full tool access (process gap: Pass 1 ran with Read-only tools). Wave 2 gate Pass 2 closure 2026-04-26: TD-W2-CICD-SCOPE-001 (P2) — CI hotfix PR scope discipline (product-code-creep prevention); TD-VSDD-005 (P2) — vsdd-factory:adversary runtime tool-binding defect (only Read bound at dispatch). Wave 2 gate Pass 5 closure 2026-04-26: TD-W2-MUTATE-005 (P3) — S-2.06 RED ratio 21.6% carve-out vs mutation-set decision needed (housekeeping pause). Wave 2 gate step c (code-review) 2026-04-26: TD-W2-CODE-MED-001..006 (P3) + TD-W2-CODE-LOW-001..006 (P3) — MEDIUM/LOW code quality findings. Wave 2 gate step d (security-review) 2026-04-26: TD-W2-SEC-MED-001..003 (P2/P3) + TD-W2-SEC-LOW-001..003 (P3) — security findings. Wave 2 gate step e (consistency-validation) 2026-04-26: TD-W2-CONS-001 (P3) + TD-W2-DOC-001 (P3). HIGH findings WGC-W2-001 + WGC-W2-002 + WGS-W2-001 + WGS-W2-002 + WGCV-W2-001 + WGCV-W2-002 require fix-PRs (W2-FIX-G/H/I). ADR-005 PO sign-off 2026-04-26: TD-ADR005-001 (P2) — CODEOWNERS security reviewer entry for `crates/prism-sensors/src/auth/` before production deployment. Wave 2 holdout gate triage 2026-04-27: TD-HOLDOUT-W2-001 (P3) — MCP server binary out of Wave 2 scope (Phase 3 milestone); TD-HOLDOUT-W2-002 (P2) — HS-006/HS-007 scenario PO refresh required (stale BC references BC-2.07.007–010 retired in v4.3). Wave 2 gate CONVERGED 2026-04-27: TD-W2-FIXK-002 (P3) — BC-named tests assert only result.is_ok() without backend-shape assertion (Pass 8 finding P8-001)._

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
| TD-VSDD-001 | MOVED | MOVED to vsdd-plugin-tech-debt.md per 2026-05-02 user directive — vsdd-factory deliver-story anti-precedent guard | — | — | — | — | vsdd-plugin-tech-debt.md |
| TD-VSDD-002 | MOVED | MOVED to vsdd-plugin-tech-debt.md per 2026-05-02 user directive — vsdd-factory Red Gate density check | — | — | — | — | vsdd-plugin-tech-debt.md |
| TD-VSDD-003 | MOVED | MOVED to vsdd-plugin-tech-debt.md per 2026-05-02 user directive — vsdd-factory tdd_mode frontmatter field | — | — | — | — | vsdd-plugin-tech-debt.md |
| TD-VSDD-004 | MOVED | MOVED to vsdd-plugin-tech-debt.md per 2026-05-02 user directive — vsdd-factory mutation testing gate for facade stories | — | — | — | — | vsdd-plugin-tech-debt.md |
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
| TD-W2-MUTATE-005 | Wave 2 gate Pass 5 finding W2-P5-A-003 | S-2.06 (prism-sensors DataSource trait + adapter registry) shipped with RED ratio 11/51 ≈ 21.6%, below the Layer-2 ≥0.5 threshold. S-2.06 evidence-report.md:13-18 discloses that 40 of 51 GBD tests are pure-data assertions (struct shape, enum variants, constants) driven by 5 RED tests + 6 GREEN-by-design algorithmic helpers (retry_with_backoff, fan_out, semaphore). Decision needed: does this data-structure-heavy carve-out distinguish S-2.06 from TD-W2-MUTATE-001..004 (stub-as-impl stories)? Origin: W2-P5-A-003. | P3 | wave-2-gate-pass-5 | S-2.06 | prism-sensors | housekeeping pause (before Wave 3) |
| TD-W2-ULID-001 | Wave 2 gate Pass 1 finding W2-P1-A-005 | EventBufferStore event keys use a 4-byte `subsec_nanos` suffix instead of a 16-byte ULID. PR-FIX-W2-A aligned the docs with the actual 4-byte impl, but the underlying collision risk under sustained ingest remains. Workspace already includes `uuid` v7 in multiple crates; consider taking a real ULID dep or adding a sequence counter to disambiguate within-microsecond writes. Files affected: `crates/prism-sensors/src/event_buffer.rs`. Origin: W2-P1-A-005. | P3 | wave-2-gate-pass-1 | S-2.08 | prism-sensors | Before high-throughput sensor onboarding (Wave 3 or later) |
| TD-W2-PASS1-TOOLING-001 | MOVED | MOVED to vsdd-plugin-tech-debt.md per 2026-05-02 user directive — adversary dispatch tooling (root cause: VSDD plugin dispatch path) | — | — | — | — | vsdd-plugin-tech-debt.md |
| TD-W2-CICD-SCOPE-001 | Wave 2 gate Pass 2 finding W2-P2-A-003 + Architect decision | CI hotfix PR scope discipline. PR #45 (`7903da15`) was a CI hotfix nominally scoped to workflow files but added `#[cfg_attr(kani, derive(kani::Arbitrary))]` to `crates/prism-core/src/case.rs:50`. Architect decision: KEEP the change (load-bearing for VP-005/006/051 proofs). However, the change should have been in a separate product PR. Establish a CI-hotfix PR checklist: diffs must be limited to `.github/workflows/**`, `fuzz/Cargo.toml`, and test-fixture files only. Any product-code change (even a one-line attribute macro) requires a full story/feature PR. ADR-004 stub created to retroactively document the kani::Arbitrary policy for types used in proofs. | P2 | wave-2-gate-pass-2 | — | orchestrator + pr-manager (enforce via review) | Before next CI hotfix burst |
| TD-VSDD-005 | MOVED | MOVED to vsdd-plugin-tech-debt.md per 2026-05-02 user directive — vsdd-factory:adversary runtime tool-binding defect | — | — | — | — | vsdd-plugin-tech-debt.md |
| TD-VSDD-029 | MOVED | MOVED to vsdd-plugin-tech-debt.md per 2026-05-02 user directive — state-manager.md parallel-changelog symmetry guardrail | — | — | — | — | vsdd-plugin-tech-debt.md |
| TD-W2-DOC-001 | Pass 6 finding W2-P6-A-001 | 15 more test files with stale `todo!()` narrative beyond W2-FIX-F sweep — broader stub-state prose cleanup not covered by PR-FIX-W2-F grep. Wave 3 cleanup sweep. | P3 | wave-2-gate-pass-6 | — | Wave 3 start | Wave 3 start |
| TD-W2-CODE-MED-001 | WGC-W2-003 | Hardcoded `SensorType::CrowdStrike` in `fanout.rs` panic handler (line 354-362) — misleading telemetry when a non-CrowdStrike sensor task panics. Fix: move `client_id` and `sensor_type` into task result to preserve across `JoinHandle` boundary. | P3 | wave-2-gate-step-c | prism-sensors | fanout.rs | Wave 3 |
| TD-W2-CODE-MED-002 | WGC-W2-004 | `CrowdStrikeAdapter::new` uses `unwrap_or_default()` on HTTP client build — silently drops TLS/security options on failure. Fix: change to `try_new -> Result` or `.expect(...)`. | P3 | wave-2-gate-step-c | prism-sensors | auth/crowdstrike.rs | Wave 3 |
| TD-W2-CODE-MED-003 | WGC-W2-005 | `event_key` calls `SystemTime::now()` twice — derives microsecond prefix and nanos suffix from separate calls. Fix: derive both from same `record.ingested_at` value. | P3 | wave-2-gate-step-c | prism-sensors | event_buffer.rs | Wave 3 |
| TD-W2-CODE-MED-004 | WGC-W2-006 | `CredentialAccessType::Rotate` doc comment says "List credentials" — copy-paste error. Fix: rename variant to `List` or fix doc to describe rotate semantics. | P3 | wave-2-gate-step-c | prism-audit | credential_events.rs | Wave 3 |
| TD-W2-CODE-MED-005 | WGC-W2-007 | Duplicate `CapabilityCheckResult` type in prism-audit (`audit_entry.rs:87` and `write_audit.rs:17`) — naming collision. Fix: consolidate to one or rename one to `WriteCapabilityCheckResult`. | P3 | wave-2-gate-step-c | prism-audit | audit_entry.rs, write_audit.rs | Wave 3 |
| TD-W2-CODE-MED-006 | WGC-W2-008 | TOCTOU race in `CrowdStrikeAdapter` token cache (read-lock check then write-lock replace). Fix: double-checked locking with write lock + re-check, or `tokio::sync::OnceCell`. | P3 | wave-2-gate-step-c | prism-sensors | auth/crowdstrike.rs | Wave 3 |
| TD-W2-CODE-LOW-001 | WGC-W2-009 | `fanout.rs` dead `execute_target` function duplicates fan-out logic and is never called. Remove or justify. | P3 | wave-2-gate-step-c | prism-sensors | fanout.rs | Wave 3 cleanup |
| TD-W2-CODE-LOW-002 | WGC-W2-010 | Multiple files use deprecated `DateTime::from_timestamp` (deprecated in chrono 0.4.23+). Migrate to `DateTime::from_timestamp_opt` or equivalent non-deprecated API. | P3 | wave-2-gate-step-c | prism-sensors, prism-audit | various | Wave 3 cleanup |
| TD-W2-CODE-LOW-003 | WGC-W2-011 | `retry_forward_entry` in `audit_buffer.rs` permanently stubbed — always returns `Err(AuditPersistenceFailed)`. No tracking issue and no corresponding story. File story or remove stub. | P3 | wave-2-gate-step-c | prism-audit | audit_buffer.rs | Wave 3 |
| TD-W2-CODE-LOW-004 | WGC-W2-012 | `DecorationStore` fields and helper suppressed with `#[allow(dead_code)]`. Remove dead code or move forward-declarations to the story that uses them. | P3 | wave-2-gate-step-c | prism-storage | decoration_store.rs | Wave 3 cleanup |
| TD-W2-CODE-LOW-005 | WGC-W2-013 | `paginate_claroty` silently returns `Ok(vec![])` on `total_count == 0` — legitimate empty result and misconfigured query are indistinguishable. Add logging or separate error path. | P3 | wave-2-gate-step-c | prism-sensors | auth/claroty.rs | Wave 3 |
| TD-W2-CODE-LOW-006 | WGC-W2-014 | `AuditEmitterService::call` reconstructs `AuditedResponse` redundantly on inner error — second construction immediately overwritten by `Err(...)` return. Clean up dead binding. | P3 | wave-2-gate-step-c | prism-audit | audit_entry.rs | Wave 3 cleanup |
| TD-W2-SEC-MED-001 | WGS-W2-003 | DTU `POST /dtu/reset` unauthenticated on Slack/PagerDuty/Jira clones — same sensitivity as `POST /dtu/configure` which requires `X-Admin-Token`. Apply `X-Admin-Token` gate to all three `/dtu/reset` endpoints. Fix in W2-FIX-I or follow-up before production deployment. | P2 | wave-2-gate-step-d | prism-dtu-slack, prism-dtu-pagerduty, prism-dtu-jira | routes/dtu.rs | Before production deployment |
| TD-W2-SEC-MED-002 | WGS-W2-004 | Event buffer key injection via `table_name` and `client_id` containing `/` — slash validation exists for `sensor_id` but not these fields. Add identical slash rejection to `table_name` and `client_id` validation in `write_events` and `scope_prefix`. | P2 | wave-2-gate-step-d | prism-sensors | event_buffer.rs | Wave 3 |
| TD-W2-SEC-MED-003 | WGS-W2-005 | `SensorError::HttpError { body }` propagates raw API response bodies into error messages — may contain auth challenge details or internal system identifiers. Truncate and sanitize body before `HttpError` construction; log raw body at `TRACE` only. | P3 | wave-2-gate-step-d | prism-sensors | adapter.rs | Wave 3 |
| TD-W2-SEC-LOW-001 | WGS-W2-006 | `emit_credential_event` logs `parameters` JSON via `tracing::info!` without redaction pipeline — related to WGC-W2-001 (audit emitters non-functional). Log-level audit leakage until persistence path implemented. Resolved when WGC-W2-001 fix lands. | P3 | wave-2-gate-step-d | prism-audit | credential_events.rs | With WGC-W2-001 fix (W2-FIX-H) |
| TD-W2-SEC-LOW-002 | WGS-W2-007 | `unsafe impl Sync for RocksDbBackend` documented with safety invariant + DEV-004 tracking. Resolve DEV-004 before high-concurrency production deployment. | P3 | wave-2-gate-step-d | prism-storage | backend.rs | Before high-concurrency production |
| TD-W2-SEC-LOW-003 | WGS-W2-008 | `token_events.rs` emitters log `token_id` at `tracing::info!` level — compliance-relevant events (SOC 2 CC6.1) currently only in transient logs. Resolved when WGC-W2-001 fix lands. | P3 | wave-2-gate-step-d | prism-audit | token_events.rs | With WGC-W2-001 fix (W2-FIX-H) |
| TD-W2-CONS-001 | WGCV-W2-007 | `RouteDecision` defined in `prism-sensors` but consumed by `prism-query` — Cargo.toml dependency not documented in S-3.02 spec or `dependency-graph.md`. S-3.02 will need to add `prism-sensors` as a Cargo.toml dependency in `prism-query`. Document in S-3.02 spec pre-work. | P3 | wave-2-gate-step-e | — | S-3.02 spec | S-3.02 spec authoring |
| ~~TD-W2-FIX-H-001~~ | W2-FIX-H PR delivery | CLOSED — PR #73 (6696e374) 2026-04-28: `cargo fmt --all --check` now in `lefthook.yml`; `stage_fixed` removed; 4 TAP checks added; `LEFTHOOK=0` bypass no longer needed. | P3 | S-3.0.01 | — | lefthook.yml | CLOSED |
| TD-W2-FIX-H-002 | W2-FIX-H review (non-blocking) | `evict_expired` `known_prefixes` pruning after backend-only eviction only scans `write_cache` (not the backend). After a full restart + backend-only eviction cycle, prefixes are pruned correctly (empty cache → retain drops all). But if only *some* backend keys for a client were stale and others remain fresh (backend-only), the prefix will be removed from `known_prefixes` because the cache is empty — causing `has_data()` to return a false-negative for that client until the next `write_events` call repopulates the cache. Pre-existing limitation of the `known_prefixes` design (AC-5b deferred). Fix: after backend eviction, re-scan the backend for remaining keys to decide prefix retention. | P3 | W2-FIX-H | prism-sensors | event_buffer.rs | Before AC-5b implementation |
| TD-ADR005-001 | ADR-005 Q2 decision | CODEOWNERS stub (`* @1898co/prism-core`) does not include a dedicated `@1898co/security` required-reviewer entry for `crates/prism-sensors/src/auth/`. Because `validate_aql()` is the primary runtime enforcement control for CWE-943 (Q1: no privileged-role gate exists above it), any change to the allowlist or the validator logic should require an explicit security reviewer approval. Fix: add `crates/prism-sensors/src/auth/ @1898co/prism-core @1898co/security` to `.github/CODEOWNERS` before production deployment with real Armis credentials. | P2 | ADR-005 | — | .github/CODEOWNERS | Before production deployment |
| TD-HOLDOUT-W2-001 | W2 holdout gate gap #1 | MCP server binary (`prism-mcp` `[[bin]]` target / `pub fn run()`) not yet built. Holdout HS-001 cannot exercise end-to-end MCP entrypoint without it. Phase 3 milestone — track as scope rather than debt; will land with Wave 3+ MCP server stories. | P3 | wave-2-holdout-gate | prism-mcp | Phase 3 MCP server stories | Wave 3+ |
| TD-HOLDOUT-W2-002 | W2 holdout gate gap #4 | HS-006 and HS-007 holdout scenarios reference retired persistent-cursor BCs (BC-2.07.007–010) and are partially out of Wave 2 scope. PO refresh required during Wave 3 housekeeping pause: re-anchor scenarios to current BC index and either narrow scope to Wave 2 or expand acceptance criteria to Wave 3 deliverables. | P2 | wave-2-holdout-gate | — | holdout-scenarios/HS-006 + HS-007 | Wave 3 housekeeping pause |
| TD-W2-MUTATE-AUDIT-001 | W2 gate-step-h mutation testing | `prism-audit` mutation testing caught 80% (20/25 viable) — 5 missed in `audit_emitter.rs:164,260` (Tower Service `poll_ready`/`call` computed output not asserted), `vector_compat.rs:55,151` (`to_json` serialization / `resolve_host` negation not asserted), `write_audit.rs:100` (`to_json` computed output not asserted). These are pre-existing S-2.05 Tower middleware + serialization gaps; NOT W2-FIX-H regressions (the 5 new W2-FIX-H emitter persistence tests are mutation-clean). Add unit tests asserting on Service trait method computed outputs and serialization return values. Estimated effort: 1 day. Origin: W2 gate-step-h. | P3 | wave-2-gate-step-h | S-2.05 | prism-audit | Wave 3 hardening or sooner if Tower middleware behavior changes |
| TD-DTU-MUTATE-COVERAGE-001 | W2 gate-step-h mutation testing | All 3 Wave 2 DTU clones achieve 0% mutation caught rate: prism-dtu-pagerduty (39 missed), prism-dtu-jira (40 missed), prism-dtu-slack (36 missed) — 115 total missed mutations. Root cause: tests are fidelity-only (validate clone vs real upstream API); no internal unit assertions on `BehavioralClone` trait impls, state-machine transitions, or route handler return values. `cargo mutants` cannot invoke the fidelity validator. Recommended remediation: add targeted unit assertions for `stop`/`reset`/`configure`/`is_tls_active`/`admin_token`/`base_url`/state machine accessors + route handler internal branches across all three clone crates. Per-crate specifics: PagerDuty `incidents_snapshot()`, Jira `issues_snapshot()`, Slack `webhooks_received()`. Estimated effort: 2-3 days per clone. Origin: W2 gate-step-h. | P3 | wave-2-gate-step-h | S-6.11, S-6.12, S-6.13 | prism-dtu-pagerduty, prism-dtu-jira, prism-dtu-slack | Wave 3 hardening |
| TD-W2-FIXK-001 | MOVED | MOVED to vsdd-plugin-tech-debt.md per 2026-05-02 user directive — vsdd-factory validate-consistency tautology-detector + BC-TV field-exclusion check | — | — | — | — | vsdd-plugin-tech-debt.md |
| TD-W2-FIXK-002 | Pass 8 finding P8-001 | BC-named emit-path tests at `specialized_event_tests.rs:58, :540, :897` (e.g., `test_BC_2_05_010_token_generated_result_summary_is_confirmation_token_issued`) call the emitter but assert only `result.is_ok()` — the BC postcondition encoded in the test name is never verified against the persisted entry. Same class as Pass 7 HIGH-003 tautology but weaker (emitter IS called; assertion gap is backend-shape, not call-site). Remediation: add backend-shape assertions to existing tests OR extend `validate-consistency` skill (TD-W2-FIXK-001) with a "test-name-vs-assertions" check that flags any `test_BC_*` or `test_TV_*` test that doesn't include backend-shape assertions when the corresponding emit function persists. Origin: Pass 8 finding P8-001. | P3 | wave-2-gate-pass-8 | — | crates/prism-audit/src/tests/specialized_event_tests.rs | Wave 3 housekeeping pause or Wave 3 hardening |
| TD-W4-AUDIT-QUERY-REPLAY-001 | phase-3a-approval-q1 | Wave 4+ Capability: Audit Query and Replay. CAPABILITY: Query historical audit events and replay them through alternate handlers. Four use cases: (1) forensic investigation — query past tenant activity to reconstruct incidents (credential rotations + sensor write attempts for org X in time range); (2) regulatory audit response — produce immutable evidence packages for SOC2/HIPAA/FedRAMP auditors with cryptographic chain-of-custody; (3) ML training corpus — replay anonymized audit streams through detection rule engines for offline tuning + false-positive analysis; (4) debugging issue traces — replay a session audit trail through a debug handler to reproduce production issues without mocking. KEY DESIGN QUESTIONS: Query language (extend PrismQL with temporal/audit dialect, or separate AQL?); replay semantics (fully idempotent vs shadow-mode vs live re-execution); retention (Wave 3 per-org TTL; Wave 4+ may need cold-storage S3/GCS Parquet tier); multi-tenant (replay queries scoped to caller org_id per CAP-038); indexing (RocksDB hot tier + columnar cold tier). SCOPE TARGET: Wave 4+. ORIGIN: Q1 of Phase 3.A approval gate (D-136). | P2 | phase-3a-approval-q1 | wave-3-multi-tenant | prism-audit, prism-query, prism-spec-engine, future prism-replay crate | Wave 4+ |
| TD-W4-LOG-FORWARDING-001 | phase-3a-approval-q1 | Wave 4+ Capability: Log Forwarding. CAPABILITY: Outbound log/audit/event forwarding to external sinks, per-org configurable. Three use cases: (1) customer SOC integration — normalized OCSF events fed to customer Splunk/Sentinel/Elastic SIEM in real-time; (2) MSSP centralized monitoring — aggregated cross-customer event stream for 1898 & Co SOC analysts; (3) regulatory retention — forwarding to compliance archive vendors (Mimecast, Cohesity, Veeam). KEY DESIGN QUESTIONS: Protocol support (syslog TCP/UDP/TLS, HTTPS webhooks, Kafka producers, Vector pipelines, S3/GCS object writes; multiple sinks per org?); format mapping (raw OCSF JSON, syslog RFC 5424, custom field mappings, protobuf serialization); buffering and retry (durability when sink offline; local disk buffer + exponential backoff; dead-letter queue?); per-org credentials (TLS client cert, bearer token, AWS IAM role — reuses prism-credentials AI-opaque credential model from Wave 1); throughput sizing. SCOPE TARGET: Wave 4 or 5 — depends on Wave 3 sensor stabilization. ORIGIN: Q1 of Phase 3.A approval gate (D-136). | P2 | phase-3a-approval-q1 | wave-3-multi-tenant | future prism-forwarder crate, integrates with prism-audit + prism-spec-engine | Wave 4+ |
| TD-W4-ALERTING-WORKFLOWS-001 | phase-3a-approval-q1 | Wave 4+ Capability: Alerting Workflows. CAPABILITY: Detection rule engine + alert routing + escalation policies + notification fan-out, per-org. Five use cases: (1) real-time SOC alerting — triaged alerts with deduplication, suppression windows, and severity scoring; (2) on-call rotation integration — PagerDuty/OpsGenie/Squadcast for after-hours escalation; (3) custom severity policies — org-specific rules overriding default severity; (4) alert deduplication — N alerts in window collapse to one notification; (5) multi-channel notification fan-out (Slack + Jira + email digest). KEY DESIGN QUESTIONS: Rule language (Sigma rules for portability + custom DSL extensions, or SPL-style proprietary, or hybrid?); evaluation engine (streaming vs micro-batch vs both); state management (suppression windows, dedup keys, alert grouping by host/user/rule_id); escalation logic (time-based + severity-based); DTU integration (Wave 2 DTU clones for PagerDuty/Slack/Jira used as notification sinks). SCOPE TARGET: Wave 4 or 5 — likely after TD-W4-LOG-FORWARDING-001 lands. ORIGIN: Q1 of Phase 3.A approval gate (D-136). | P2 | phase-3a-approval-q1 | wave-3-multi-tenant | future prism-alerting + prism-detection crates, integrates with prism-dtu-slack/pagerduty/jira | Wave 4+ |
| TD-W3-S-3.0.02-DOC-001 | S-3.0.02 pr-manager review | Story S-3.0.02 §Tasks item 3 and §Architecture-Compliance-Rules suggest adding a marker comment containing the literal string `DTU_DEFAULT_MODE` to each `prism-dtu-*` crate's `lib.rs`. That literal substring would trip AC-8's grep test, causing a false-positive CI failure. Implementer correctly omitted marker comments under TDD discipline. Fix: update story S-3.0.02 to v0.6 — change suggested comment from `// Classification lives in prism-core::DTU_DEFAULT_MODE (ADR-007 §2.3)` to a form that does not contain the grep target (e.g., `// Mode classification lives in prism_core::dtu (ADR-007 §2.3)`). Documentation-polish only; no code changes needed; AC-8 already passes. Non-blocking — merge completed at 373baf78. Detail file: .factory/tech-debt/TD-W3-S-3.0.02-DOC-001.md | P3 | wave-3-s-3.0.02-review | wave-3-multi-tenant | S-3.0.02 story spec v0.6 | Wave 3 housekeeping (story-writer task) |
| TD-W3-S-3.7.01-001 | S-3.7.01 pr-manager review (F-003) | pagination.rs in prism-dtu-common/src/generator/ uses bare integer constants (e.g., `100`, `1000`) for default_page_size and max_page_size values rather than named constants. Non-blocking polish: extract to named const DEFAULT_PAGE_SIZE: u32 = 100 and MAX_PAGE_SIZE: u32 = 1000 for readability and future configurability. F-001 (BLOCKING — optional deps AC-007 violation) resolved at 82473db3; F-002 (doc comment) resolved in PR; F-003 deferred to this TD. Merge completed at 0bb7735d (PR #76). | P3 | wave-3-s-3.7.01-review | wave-3-multi-tenant | crates/prism-dtu-common/src/generator/pagination.rs | Wave 3 housekeeping |
| TD-S3705-001 | S-3.7.05 pr-manager review | prism-dtu-crowdstrike's Cargo.toml declares `prism-core` as a mandatory dependency, but the crate only uses it for shared types that could be made optional. Making prism-core an optional dep (enabled by `fixture-gen` feature flag) would reduce compilation overhead for downstream crates that consume prism-dtu-crowdstrike without fixture generation. Suggestion-level — correctness unaffected. Non-blocking; merge completed at 89fa8dea (PR #80). | P4 | wave-3-s-3.7.05-review | wave-3-multi-tenant | crates/prism-dtu-crowdstrike/Cargo.toml | Wave 3 housekeeping (suggestion) |
| TD-S3501-W3-001 | S-3.5.01 implementer observation | Pre-existing clippy errors in `prism-dtu-claroty`, `prism-dtu-armis`, and `prism-dtu-crowdstrike` crates. Workspace clippy gate currently passes per-crate via lefthook but not workspace-wide (`cargo clippy --workspace` is not part of the default CI path). Verify on develop with `cargo clippy --workspace 2>&1 \| grep "^error"` and file a fix story if errors are present. Severity: P3/suggestion. Surfaced by S-3.5.01 implementer during crate-layout sweep. | P3 | wave-3-s-3.5.01-implementation | wave-3-multi-tenant | prism-dtu-claroty, prism-dtu-armis, prism-dtu-crowdstrike | Wave 3 housekeeping — verify and file fix story if needed |
| TD-W3-CI-MSVC-001 | S-3.2.05 PR CI Run 1 observation | Pre-existing Windows MSVC flake in `prism-sensors` crate: `test_BC_2_01_http_semaphore_acquire_succeeds_when_permits_available` failed intermittently in CI Run 1 during S-3.2.05 delivery; CI Run 2 passed without code changes. Not caused by Wave 3 changes — pre-existing timing sensitivity in semaphore acquire test under Windows MSVC toolchain. Track for future stabilization. Severity: P3 (non-blocking; Linux CI path unaffected). | P3 | wave-3-s-3.2.05-ci-observation | wave-3-multi-tenant | crates/prism-sensors/src/ (Windows MSVC CI path only) | Wave 3 housekeeping — stabilize or gate-skip on MSVC |
| TD-W3-TIMING-001 | W3-FIX-SEC-001 / pass-49 L-002 | BC-3.5.001/002 wall-clock budget tests marked `#[ignore]` in PR #113 due to fragility under workspace nextest parallelism. Follow-up required: formally amend BC-3.5.001/ADR-011 D-058 OR optimize harness build time OR migrate to Criterion benchmark. Until resolved the wall-clock invariant is not runtime-enforced. | P2 | wave-3-1-fix | wave-3-multi-tenant | crates/prism-dtu-harness/tests/ | Wave 3.3 or Wave 4 — before harness performance regressions become observable |
| ~~TD-W3-CREDS-001~~ | W3-FIX-CREDS-001 / pass-49 | CLOSED — CredentialStoreOrgId trait methods were todo!() stubs (BC-3.2.002 unimplemented). Confirmed false positive after W3-FIX-CREDS-001 PR #121 (9d04235d): implementation provides regression coverage proving the BC behavior is satisfied. TD CLOSED 2026-05-02. | ~~HIGH~~ CLOSED | wave-3-1-fix | wave-3-multi-tenant | crates/prism-credentials/ | CLOSED PR #121 |
| TD-W3-POLL-NOTIFY-001 | pass-50 L-50-004 | `tokio::sync::Notify`-based cancellation not used in poll loop — current implementation uses a channel-based pattern that has a latency gap at shutdown. Follow-up: migrate poll loop cancellation to `Notify`-based pattern for lower-latency shutdown signalling. Non-blocking (current behavior meets BC spec); optimization. Surfaced by pass-50 observation L-50-004. | P3 | wave-3-integration-gate-pass-50 | wave-3-multi-tenant | prism-sensors poll loop | Wave 4 planning or later |
| TD-W3-CT-EQ-COVERAGE-001 | PR #125 R1-001 (fc467937) | Non-DTU code paths can also contain non-constant comparisons that warrant audit. Surfaced by pr-reviewer cycle 2 finding R1-001 on W3-FIX-SEC-005 (PR #125): ThreatIntel `lookup` handler in `crates/prism-dtu-threatintel/src/routes/lookup.rs` had a non-constant comparison that was fixed in fc467937. Pattern: W3-FIX-SEC-004 applied `subtle::ConstantTimeEq` to DTU clone handlers; R1-001 found that ThreatIntel's non-clone lookup handler also had a timing-sensitive comparison. Systematic audit: sweep all crates for string/byte comparisons involving security tokens that do not use `subtle::ConstantTimeEq` or equivalent; not limited to DTU clone routes. | P3 | wave-3-4-fix | wave-3-multi-tenant | All crates — non-clone handler code paths | Wave 4 planning or security audit sprint |
| TD-W3-QUOTA-SOAK-001 | holdout-evaluator pass-5 BELOW_BAR-002 (HS-003-06) | Cross-tenant API quota soak test gap. HS-003-06 (Per-Tenant Rate Limiting) scored BELOW_BAR in pass-5 holdout evaluation: rate-limit key structure verified (org_id, sensor_type) but no 60s soak test confirms Tenant A high-frequency polling does not exhaust Tenant B's API quota. Requires: (1) soak test infrastructure in prism-dtu-harness (configurable per-DTU rate-limit counter); (2) integration test that simulates 60s high-frequency Tenant A + normal Tenant B calls and asserts Tenant B throughput unaffected. Deferred to Wave 4 per D-191 (non-blocking carry-forward). | P3 | wave-3-integration-gate-pass-52 | wave-3-multi-tenant | crates/prism-dtu-harness + per-DTU rate-limit layer | Wave 4 planning |
| TD-S304-001 | S-3.04 implementer observation | ConfirmationTokenStore full integration for alias delete deferred. `delete_alias` uses a synthetic `ConfirmationToken` instead of calling `ConfirmationTokenStore::generate()` + `ConfirmationTokenStore::consume()`. Full token lifecycle requires MCP routing layer (not yet built). Detail: `.factory/tech-debt/TD-S304-001.md`. | P2 | wave-3 | S-3.04 | prism-query alias_tools.rs | Wave 3 alias routing story |
| TD-S304-002 | S-3.04 implementer observation | `alias.write` capability gate compile-time feature flag integration deferred. `create_alias` and `delete_alias` skip `FeatureFlagEvaluator` gate because evaluator not threaded into tool handler context. Detail: `.factory/tech-debt/TD-S304-002.md`. | P2 | wave-3 | S-3.04 | prism-query alias_capability.rs | Wave 3 capability-gate story |
| TD-S304-003 | S-3.04 implementer observation | BC-2.11.009 dual-recording (original + expanded query) not implemented — `query_context` infrastructure not yet present. Detail: `.factory/tech-debt/TD-S304-003.md`. | P3 | wave-3 | S-3.04 | prism-query alias_tools.rs | query_context milestone |
| TD-S304-AUDIT-001 | S-3.04 implementer observation | Alias audit trail missing — alias CRUD operations do not emit structured audit events (BC-2.11.008/013/014 postconditions). Deferred until prism-audit MCP integration lands. Detail: `.factory/tech-debt/TD-S304-AUDIT-001.md`. | P3 | wave-3 | S-3.04 | prism-query, prism-audit | prism-audit MCP integration story |
| TD-S304-FILEPERMS-001 | S-3.04 implementer observation | `aliases.toml` file permissions not set to 0600 after atomic write — file is created with process umask permissions. Fix: call `set_permissions(0o600)` after `rename` in the atomic write helper. Detail: `.factory/tech-debt/TD-S304-FILEPERMS-001.md`. | P3 | wave-3 | S-3.04 | prism-query alias_store.rs | Wave 3 hardening |
| TD-S304-FUZZ-001 | S-3.04 implementer observation | VP-037 fuzz target stub does not construct structured alias store inputs — exercises only raw query strings without alias token injection. Implement `FuzzAliasInput` decoder. Detail: `.factory/tech-debt/TD-S304-FUZZ-001.md`. | P3 | wave-3 | S-3.04 | fuzz/fuzz_targets/ | VP-037 fuzz milestone |
| TD-S304-TMPNAME-001 | S-3.04 implementer observation | Atomic write temp file uses `aliases.toml.tmp` (fixed name) — concurrent writes could collide. Fix: use a random suffix (e.g., `aliases.toml.{uuid}.tmp`). Detail: `.factory/tech-debt/TD-S304-TMPNAME-001.md`. | P3 | wave-3 | S-3.04 | prism-query alias_store.rs | Wave 3 hardening |
| TD-S304-VISIBILITY-001 | S-3.04 implementer observation | `AliasStore` internal fields are pub(crate) but some helpers could be further restricted to pub(super). Low-impact visibility cleanup. Detail: `.factory/tech-debt/TD-S304-VISIBILITY-001.md`. | P3 | wave-3 | S-3.04 | prism-query alias_store.rs | Wave 3 housekeeping |
| TD-S304-VP013-001 | S-3.04 implementer observation | VP-013 proptest for cycle detection uses fixed-seed RNG for reproducibility but does not cover all graph topologies for 10-node alias graphs. Extend with structured graph generation. Detail: `.factory/tech-debt/TD-S304-VP013-001.md`. | P3 | wave-3 | S-3.04 | crates/prism-query/src/proofs/ | Wave 3 hardening |
| TD-S-PLUGIN-PREREQ-A-002 | S-PLUGIN-PREREQ-A fix-burst-1 implementer observation | Sentinel-nil OrgId in WriteDispatcher — replace with proper OrgRegistry lookup. After S-PLUGIN-PREREQ-A, the AdapterRegistry uses (OrgId, SensorId) composite key. WriteDispatcher.execute_at(...) currently uses OrgId::from_uuid(uuid::Uuid::nil()) as a sentinel placeholder (write_dispatch.rs:289). In production, this always returns None from the registry lookup, structurally breaking write dispatch for any non-test caller. Resolution requires threading OrgRegistry: Arc<OrgRegistry> through WriteDispatcher and resolving context.org_slug → OrgId via the registry. Depends on: graduation of W3-FIX-S307-002 (write dispatch wiring) from BLOCKED status; OrgRegistry boot wiring. Acceptance criteria: production WriteDispatcher resolves OrgId from OrgSlug at execute-time; integration test exercising real write tool with valid OrgId returns Ok. Source: write_dispatch.rs:283-290 TODO comment. | P1 | plugin-migration | S-PLUGIN-PREREQ-A | prism-query write_dispatch.rs | W3-FIX-S307-002 unblock |
| TD-S-PLUGIN-PREREQ-A-004 | S-PLUGIN-PREREQ-A pass-3 fix-burst-3 state-manager | Runtime assertion that AdapterRegistry has ≥1 adapter before query engine serves (boot.rs step8). F-LP2-MED-003 (pass-2) requested defense-in-depth: ensure AdapterRegistry is populated before the query engine accepts queries, so a silent boot-failure does not propagate as silent empty results (regressing the pass-58 unknown-table fix). Fix-burst-2 added doc-comment documentation to crates/prism-bin/src/boot.rs:817-833 explaining the contract, but step8 (init_query_engine) body is still todo!() awaiting S-WAVE5-PREP-01 step 8 / S-3.02-FOLLOWUP-RUNTIME query-engine wiring. The assertion is intrinsically deferred until step8 has a non-stub body. This TD tracks the assertion as a successor-story obligation for whatever story actually wires step8. Acceptance criteria: when step8 is wired, the first thing it does after receiving the AdapterRegistry is check is_empty() and emit a fatal BootError if true (in production mode; test-mode short-circuit acceptable). The materialization.rs:653 is_empty() short-circuit remains as defense-in-depth but is not sufficient alone. Filed: 2026-05-11 (S-PLUGIN-PREREQ-A pass-3 fix-burst-3). Source: crates/prism-bin/src/boot.rs:817-838. | P1 | plugin-migration | S-PLUGIN-PREREQ-A | prism-bin boot.rs:817-838 | step8 wiring (S-WAVE5-PREP-01 or successor) |
| TD-S-PLUGIN-PREREQ-A-003 | S-PLUGIN-PREREQ-A fix-burst-2 implementer observation | Runtime extensibility for WriteToolInvalidationMap — plugin-registered write tool support. After S-PLUGIN-PREREQ-A fix-burst-2, WriteToolInvalidationMap was converted from `&[...]` static slice to `LazyLock<Vec<...>>` to address closed-set concerns. However, `LazyLock<Vec<T>>` provides only `Deref<Target=Vec<T>>` (read-only access). The doc-comment claim "future plugin-registered write tools can extend it at runtime" was honestly rewritten to acknowledge the static set in fix-burst-2 (F-LP2-HIGH-002 closure). True runtime extensibility requires `RwLock<Vec<WriteToolInvalidationMap>>` or `OnceLock<RwLock<...>>` + a registration API (e.g., `pub fn register_write_tool(entry: WriteToolInvalidationMap)`). Defer to PREREQ-E which retires the CustomAdapter trait and is the natural home for runtime-plugin write-tool registration. Acceptance criteria: plugin loaded at boot registers a custom write tool via PluginRuntime; cache invalidation for that write tool fires correctly when a write occurs in the plugin's sensor. Filed: 2026-05-11 (S-PLUGIN-PREREQ-A fix-burst-2). Source: crates/prism-query/src/invalidation.rs:38-57. | P1 | plugin-migration | S-PLUGIN-PREREQ-A | prism-query invalidation.rs | PREREQ-E runtime-plugin write-tool registration |
| TD-S-PLUGIN-PREREQ-A-006 | S-PLUGIN-PREREQ-A pass-7 adversary (F-LP7-LOW-002) | Cross-newtype audit: pub-API validation-bypass entry points (e.g., OrgSlug::new_unchecked). F-LP7-LOW-002 (pass-7 cross-newtype audit pattern) identified that OrgSlug::new_unchecked at crates/prism-core/src/tenant.rs:77-86 is a pub-API validation-bypass entry point — same defect class as the F-LP6-HIGH-001 closure pattern applied to SensorId::new in fix-burst-6. The doc-comment "MUST NOT be called from production code" is a paper-fence; visibility is pub. Other newtypes in prism-core (AnalystId, CredentialName, etc.) should be audited for similar bypasses. Acceptance criteria: for each pub newtype with a validation-on-construct invariant in prism-core, audit all pub constructors; demote or test-gate any constructor that bypasses validation. Successor: post-PREREQ-A maintenance pass or dedicated cross-newtype hardening story. | P3 | plugin-migration | S-PLUGIN-PREREQ-A pass-7 | prism-core tenant.rs:77-86 (one specific example; full audit scope = all pub newtypes in prism-core) | post-PREREQ-A maintenance or cross-newtype hardening story |
| TD-S-PLUGIN-PREREQ-A-005 | S-PLUGIN-PREREQ-A pass-5 fix-burst-5 state-manager (F-LP5-LOW-001 closure) | EXPLAIN response UX — distinguish invalid sensor name from non-external source. After S-PLUGIN-PREREQ-A fix-burst-2 (F-LP2-CRIT-002), write_dispatch.rs:282 emits E-QUERY-031 for invalid sensor names. The EXPLAIN path at explain.rs:665 uses `SensorId::try_from_str(&lower).ok()` which silently drops invalid sensor names from `raw_sources`. The two PrismQL entry points (EXPLAIN and write) have inconsistent error UX for the same class of invalid input: write emits a typed error code; EXPLAIN silently skips. Acceptance criteria: EXPLAIN response surfaces non-fatal `ExplainWarning::UnknownSensor` (or equivalent typed warning) for invalid/unknown sensor names rather than silently skipping. Successor target: PLUGIN-MIGRATION-001-B (prism-query dispatch site conversion to spec-catalog lookup) or its successor. Filed: 2026-05-11 (S-PLUGIN-PREREQ-A pass-5 fix-burst-5). Source: crates/prism-query/src/explain.rs:665-667 TODO comment. | P3 | plugin-migration | S-PLUGIN-PREREQ-A | prism-query explain.rs:665-667 | PLUGIN-MIGRATION-001-B or successor |
| TD-VSDD-030 | MOVED | MOVED to vsdd-plugin-tech-debt.md per 2026-05-02 user directive — ADR §2 Status block ↔ frontmatter linter | — | — | — | — | vsdd-plugin-tech-debt.md |
| TD-VSDD-031 | MOVED | MOVED to vsdd-plugin-tech-debt.md per 2026-05-02 user directive — cycle-manifest epic membership ↔ story epic_id linter | — | — | — | — | vsdd-plugin-tech-debt.md |
| TD-VSDD-032 | MOVED | MOVED to vsdd-plugin-tech-debt.md per 2026-05-02 user directive — adversary review file persistence guard | — | — | — | — | vsdd-plugin-tech-debt.md |
| TD-VSDD-033 | MOVED | MOVED to vsdd-plugin-tech-debt.md per 2026-05-02 user directive — AC scope-coverage matrix template | — | — | — | — | vsdd-plugin-tech-debt.md |
| TD-VSDD-034 | MOVED | MOVED to vsdd-plugin-tech-debt.md per 2026-05-02 user directive — gate-step pass-N completeness policy | — | — | — | — | vsdd-plugin-tech-debt.md |

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

### TD-VSDD-005 — MOVED to vsdd-plugin-tech-debt.md

See `.factory/vsdd-plugin-tech-debt.md` for full detail. Moved 2026-05-02 per user directive. Item: vsdd-factory:adversary runtime tool-binding defect — agent declares `Tools: Read, Grep, Glob` but only `Read` bound at runtime. P2. vsdd-factory plugin maintainer scope.

### TD-W2-MUTATE-005 — prism-sensors Mutation Testing — ESCALATED Option B → Option C (P2)

**Severity**: P2 (escalated — Option B killed mid-run; full deferral to Wave 3 required)
**Status**: OPTION_C_DEFERRED — awaiting Wave 3 hardening
**Opened**: 2026-04-26
**Escalated**: 2026-04-27
**Origin**: Wave 2 gate Pass 5 finding W2-P5-A-003; gate-step-h execution
**Owner**: test-writer / general-purpose implementer (Wave 3 hardening window)

**Problem**

S-2.06 (prism-sensors DataSource trait + adapter registry) shipped with RED ratio 11/51 ≈ 21.6%, below the Layer-2 ≥0.5 threshold. The architect approved Option B: scoped run against 5 Wave 2 files (pagination.rs, timestamp.rs, auth/armis.rs, auth/claroty.rs, auth/crowdstrike.rs) with 15-40 minute estimated runtime. Option B was started but **killed at 17 minutes with 0 mutants tested** — the run was still in the rocksdb-sys C++ baseline build phase.

Root cause: the architect's 15-40 minute estimate correctly modeled Rust incremental build costs (12-25s per mutant) but did not account for the rocksdb-sys transitive C++ rebuild cost. The C++ baseline dominates per-mutant overhead regardless of which Rust source files are scoped. Extrapolated total runtime: 2-4 hours, materially blocking Wave 2 close.

**Option C escalation — full deferral to Wave 3 hardening**

Scope changed from Option B (architect's targeted 5-file list) to Option C (full deferral). The original coverage targets remain valid for the Wave 3 run:
- `pagination.rs` (S-2.07 pure algorithmic pagination + cursor logic)
- `timestamp.rs` (S-2.07 multi-format fallback chain)
- `auth/armis.rs` (W2-FIX-I AQL validator — security-critical, CWE-943)
- `auth/claroty.rs` (W2-FIX-I SecretString plumbing)
- `auth/crowdstrike.rs` (W2-FIX-I SecretString plumbing)

These targets are no longer gating Wave 2 close.

**Wave 3 execution plan**

1. Run `cargo mutants -p prism-sensors` once in an isolated worktree with extended timeout.
2. After the rocksdb-sys C++ baseline builds for the first mutant, subsequent mutants see only Rust incremental overhead — the 2-4hr estimate applies to the full initial run; incremental subsequent runs are much faster.
3. Schedule as unattended overnight job with output tee'd to `.factory/cycles/phase-3-dtu-wave-2/mutation-results-w2-mutate-005.txt`.
4. Parent TD: TD-W2-SENSORS-FULL-001 (full crate sweep, P3, Wave 3 hardening after S-3.02).

**Decision document**: `cycles/phase-3-dtu-wave-2/decision-w2-mutate-005-carveout.md` (status updated to option_b_killed_option_c_escalated).

**Target resolution**: Wave 3 hardening (after S-3.02 merges; see TD-W2-SENSORS-FULL-001)

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

---

## Moved Items Appendix

_Items extracted to `.factory/vsdd-plugin-tech-debt.md` per user directive 2026-05-02. These are VSDD/methodology/pipeline-mechanics items outside the scope of Wave 4 product execution._

| ID | Summary | Moved Date | Reason |
|----|---------|-----------|--------|
| TD-VSDD-001 | vsdd-factory deliver-story anti-precedent guard (Layer 1) | 2026-05-02 | VSDD plugin — methodology scope |
| TD-VSDD-002 | vsdd-factory Red Gate density check gate (Layer 2) | 2026-05-02 | VSDD plugin — methodology scope |
| TD-VSDD-003 | vsdd-factory tdd_mode frontmatter field (Layer 3) | 2026-05-02 | VSDD plugin — methodology scope |
| TD-VSDD-004 | vsdd-factory mutation testing gate for facade stories (Layer 4) | 2026-05-02 | VSDD plugin — methodology scope |
| TD-VSDD-005 | vsdd-factory:adversary runtime tool-binding defect | 2026-05-02 | VSDD plugin — agent dispatch defect |
| TD-W2-PASS1-TOOLING-001 | Adversary dispatch tooling (Pass 1 Read-only tools) | 2026-05-02 | VSDD plugin — orchestrator/adversary dispatch path |
| TD-VSDD-029 | state-manager.md parallel-changelog symmetry guardrail | 2026-05-02 | VSDD plugin — state-manager guardrail |
| TD-VSDD-030 | ADR §2 Status block ↔ frontmatter linter | 2026-05-02 | VSDD plugin — process-gap linter |
| TD-VSDD-031 | cycle-manifest epic membership ↔ story epic_id linter | 2026-05-02 | VSDD plugin — process-gap linter |
| TD-VSDD-032 | Adversary review file persistence guard | 2026-05-02 | VSDD plugin — wave-gate skill checklist |
| TD-VSDD-033 | AC scope-coverage matrix template | 2026-05-02 | VSDD plugin — story template + consistency-validator |
| TD-VSDD-034 | gate-step pass-N completeness policy | 2026-05-02 | VSDD plugin — wave-gate skill policy |
| TD-W2-FIXK-001 | validate-consistency tautology-detector + BC-TV field-exclusion check | 2026-05-02 | VSDD plugin — validate-consistency skill |

Full detail for all 13 items: `.factory/vsdd-plugin-tech-debt.md`

---

## Changelog

| Version | Date | Change |
|---------|------|--------|
| 2.3 | 2026-05-11 | Filed TD-S-PLUGIN-PREREQ-A-006 P3 (cross-newtype audit: pub-API validation-bypass entry points; OrgSlug::new_unchecked at tenant.rs:77-86; F-LP7-LOW-002 source). Active items: 77 → 78. P3 count: 49 → 50. |
| 2.2 | 2026-05-11 | Filed TD-S-PLUGIN-PREREQ-A-005 P3 (EXPLAIN silent-skip UX vs E-QUERY-031; explain.rs:665 orphan citation closed). Active items: 76 → 77. P3 count: 48 → 49. |
| 2.1 | 2026-05-02 | Extracted 13 VSDD/methodology items (TD-VSDD-001/002/003/004/005, TD-W2-PASS1-TOOLING-001, TD-VSDD-029/030/031/032/033/034, TD-W2-FIXK-001) to vsdd-plugin-tech-debt.md per user directive. Active product register count: 70 → 57. TD-VSDD-005 detail section replaced with cross-reference stub. Moved Items Appendix added. |
| 2.0 | 2026-05-02 | Wave 3 integration gate CONVERGED. TD-VSDD-034 filed. Total active items: 70. |
