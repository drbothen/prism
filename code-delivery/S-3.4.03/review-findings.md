# Review Findings — S-3.4.03

**PR:** #109
**Story:** Migrate prism-dtu-crowdstrike tests to prism-dtu-harness
**Reviewer:** pr-review-triage skill (claude-sonnet-4-6)
**Review Date:** 2026-04-30

## Convergence Tracking

| Cycle | Findings | Blocking | Fixed | Remaining | Verdict |
|-------|----------|----------|-------|-----------|---------|
| 1 | 0 | 0 | 0 | 0 | APPROVE |

## Cycle 1 — Verdict: APPROVE

**No blocking findings. No non-blocking findings. PR is APPROVED for merge.**

---

## Review Checklist

### Architecture Compliance (ADR-011)

- [x] `prism-dtu-harness` is `[dev-dependency]` only in `prism-dtu-crowdstrike/Cargo.toml` — confirmed by diff context.
- [x] No `use prism_dtu_crowdstrike` import in `harness_tests.rs` — confirmed by grep.
- [x] No `CrowdStrikeClone::start()` call anywhere in the test file — AC-007 satisfied.
- [x] `clones/crowdstrike.rs` is inside `prism-dtu-harness` (internal to harness crate), not imported by production code.

### Spec Traceability

- [x] AC-001 (13 original CrowdStrike ACs via harness): 28 test functions covering ac_1 through ac_8, both integration tests (integration_vp033, integration_vp036 as smoke + ignores), fidelity_validator, TD tests, edge cases. All map correctly to S-6.07 original ACs.
- [x] AC-002 (2-org logical disjoint): `test_BC_3_5_001_ac_multi_org_logical_isolation` — uses distinct seeds 42 vs 99, asserts pairwise intersection of both detection IDs and host IDs is empty. Correct.
- [x] AC-003 (network 401 cross-creds): `test_BC_3_5_002_ac_network_cross_creds_401` — builds 2-org Network harness, sends org_a token to org_b endpoint, asserts HTTP 401. Correct.
- [x] AC-004 (fidelity validator checks_failed == 0): `test_BC_3_5_001_fidelity_validator_checks_failed_zero` — uses `cs_base_url(&harness, "test-tenant")` (from harness, not hardcoded). 3 checks (oauth, /dtu/health, /dtu/reset). Correct.
- [x] AC-005 (harness regression-safe): 49 harness tests + 14 multi_tenant tests documented; AC-005 demo recording confirms 70/70 green.
- [x] AC-006 (CrowdStrike legacy 105 pass): AC-006 demo recording confirms 105 total passing.
- [x] AC-007 (no direct clone instantiation): verified by grep — no `use prism_dtu_crowdstrike` import in harness_tests.rs.

### Test Quality

- [x] Each test builds its own harness (no shared state across tests — correct).
- [x] `build_cs_harness` helper uses seed=42 consistently for deterministic fixture generation.
- [x] `build_cs_harness_with_seed` and `build_cs_harness_with_failure` helpers are correct.
- [x] 2 tests gated with `#[ignore = "needs-prism-audit"]` — correct and intentional per story EC-002.
- [x] 49 total `#[tokio::test]` annotations — matches story claim of 47 active + 2 ignored.

### Clone Router Quality (crowdstrike.rs)

- [x] `CrowdStrikeHarnessState` is `Send + Sync` (all mutable fields are `Mutex`-guarded).
- [x] `admin_token` is generated per-instance as UUID v4 — unpredictable, non-hardcoded.
- [x] LRU session registry capacity is bounded at 1,000 — no unbounded growth.
- [x] `ConfigureBody` uses `#[serde(deny_unknown_fields)]` — rejects unexpected configure fields.
- [x] Network-mode bearer guard (`check_network_bearer`) correctly enforces per-org credential binding.
- [x] `url_decode()` is correct and safe — decoded values only used as HashMap keys, never evaluated.
- [x] ID generation format `det-{org_slug}-{seed}-{i:03}` and `h-{org_slug}-{seed}-{i:03}` guarantees pairwise disjoint sets across orgs (BC-3.5.001 PC-2, TV-2).
- [x] `session_registry` keyed by bare `String` (D-048 compliant — structural separation at query-engine layer).
- [x] `reset_all()` clears containment, detection status, and session registry — does not reset auth_mode or request_counter (matches production semantics, as documented in comment).

### Builder Dispatch (builder.rs)

- [x] New `match dtu_type { DtuType::CrowdStrike => ... _ => ... }` branches are additive — no other DTU startup paths modified.
- [x] Comments include `S-3.4.03 CONFLICT-AVOIDANCE` note — good merge hygiene.
- [x] Network-mode dispatch similarly additive.

### Cargo.toml Changes

- [x] `lru`, `rand`, `rand_chacha` added to `prism-dtu-harness` production dependencies — needed for `crowdstrike.rs` which is part of the harness library. Correct placement (not dev-dep because harness lib code uses them).
- [x] `prism-dtu-harness` added to `prism-dtu-crowdstrike` as dev-dep only. Correct.
- [x] New `[[test]]` harness_tests section with `required-features = ["dtu"]`. Correct.

### D-048 Session Isolation Test

- [x] `test_BC_3_2_003_ac_session_registry_per_org_isolation` uses synthetic session_id `"org-a-xor-uuid-v7-session-0000001a"` — clearly scoped to org_a by convention, structurally distinct from any org_b session_id. Sends this to org_b's endpoint and asserts 200 with empty resources. Correctly exercises the D-048 structural separation property.

### Demo Evidence

- [x] 6/6 AC recordings present: AC-001 through AC-006 (gif + webm + tape each).
- [x] `evidence-report.md` present with full coverage map.
- [x] All recordings show PASS.

---

## Summary

The PR is complete, correct, and well-structured. All 7 ACs are verifiably covered by test functions. The CrowdStrike clone router is the most complex file (1450 lines) and is correctly implemented with proper state isolation, auth guards, LRU session management, and failure injection. The D-048 session isolation test is the most nuanced addition and correctly exercises the structural separation property without re-keying the registry at the clone layer.

**VERDICT: APPROVE — ready for merge after CI passes.**
