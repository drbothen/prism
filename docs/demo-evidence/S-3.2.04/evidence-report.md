# Demo Evidence — S-3.2.04: prism-dtu-cyberint multi-tenant state segregation

**Story:** S-3.2.04 — prism-dtu-cyberint: Multi-tenant state segregation — alert_store + session_store re-keying
**Branch:** feature/S-3.2.04
**BC anchors:** BC-3.2.001, BC-3.2.003
**Tool:** VHS 0.10.0 (FiraCode Nerd Font Mono, Dracula theme)
**Implementation commit:** eaee5706

---

## Coverage Matrix

| AC | Description | BC anchor | Result | Recording |
|----|-------------|-----------|--------|-----------|
| AC-001 | All 15 multi_tenant tests GREEN | BC-3.2.001 + BC-3.2.003 | PASS | [gif](AC-001-all-multi-tenant-tests-green.gif) / [webm](AC-001-all-multi-tenant-tests-green.webm) |
| AC-002 | HTTP session token registered for org_A rejected by org_B | BC-3.2.003 postcondition 2 | PASS | [gif](AC-002-http-session-token-isolation.gif) / [webm](AC-002-http-session-token-isolation.webm) |

**Coverage: 2/2 demos recorded. All 15 multi_tenant tests green (0 failed, 0 ignored).**

---

## What the Demos Show

### AC-001 — All 15 multi_tenant tests GREEN

Runs: `cargo test -p prism-dtu-cyberint --features dtu --test multi_tenant`

The 15 tests cover the full acceptance criteria matrix:

| Test | Acceptance Criterion |
|------|---------------------|
| `test_BC_3_2_001_alert_cross_org_isolation_write_a_read_b_returns_none` | AC-001: alert_store cross-org isolation |
| `test_BC_3_2_003_session_cross_org_isolation_register_a_validate_b_returns_false` | AC-002: session cross-org isolation |
| `test_BC_3_2_003_identical_token_string_independent_per_org_contexts` | AC-003: same token, independent contexts |
| `test_BC_3_2_003_token_refresh_preserves_org_binding` | AC-004: token refresh preserves OrgId binding |
| `test_BC_3_2_001_build_alert_store_keys_are_org_composite` | AC-005: build_alert_store accepts OrgId parameter |
| `test_BC_3_2_001_reset_for_clears_both_stores_atomically_for_org_a` | AC-006: reset_for clears both stores for one org |
| `test_BC_3_2_001_reset_for_removes_org_a_alert_entries_preserves_org_b` | AC-006: reset_for selectivity (alerts) |
| `test_BC_3_2_003_reset_for_removes_org_a_session_tokens_preserves_org_b` | AC-006: reset_for selectivity (sessions) |
| `test_BC_3_2_001_invariant_cross_org_alert_lookup_always_none` | AC-007: OrgId-flip proptest (alert) |
| `test_BC_3_2_003_invariant_cross_org_session_validation_always_false` | AC-007: OrgId-flip proptest (session) |
| `test_BC_3_2_001_invariant_org_id_flip_kills_mutation` | AC-007: mutation kill — VP-079 |
| `test_BC_3_2_001_invariant_reset_for_selectivity` | AC-006: proptest selectivity invariant |
| `test_BC_3_2_003_http_session_token_registered_for_org_a_rejected_by_org_b` | AC-002: HTTP path isolation |
| `test_BC_3_2_001_http_reset_for_invalidates_org_a_preserves_org_b` | AC-006: HTTP reset_for |
| `test_BC_3_2_001_alert_independent_per_org_state_same_key` | AC-001: same alert_id, independent state |

### AC-002 — HTTP session token isolation

Runs: `cargo test -p prism-dtu-cyberint --features dtu --test multi_tenant test_BC_3_2_003_http_session -- --nocapture`

Isolates the single test that most directly embodies BC-3.2.003 postcondition 2:
a session token issued in org_A's HTTP auth flow is rejected when validated against org_B.
This is the key property that prevents lateral movement across tenant boundaries in the Cyberint DTU.

---

## Reproducing Locally

```bash
cd /path/to/prism/.worktrees/S-3.2.04

# Run either demo script directly:
bash docs/demo-evidence/S-3.2.04/AC-001-all-multi-tenant-tests-green.sh
bash docs/demo-evidence/S-3.2.04/AC-002-http-session-token-isolation.sh

# Re-record all tapes (requires VHS >= 0.10.0):
for tape in docs/demo-evidence/S-3.2.04/*.tape; do vhs "$tape"; done

# Run full test suite:
cargo test -p prism-dtu-cyberint --features dtu --test multi_tenant
```

---

## File List

| File | Type | Purpose |
|------|------|---------|
| `AC-001-all-multi-tenant-tests-green.{sh,tape,gif,webm}` | AC-001 | All 15 multi_tenant tests green |
| `AC-002-http-session-token-isolation.{sh,tape,gif,webm}` | AC-002 | BC-3.2.003 HTTP session isolation |
| `evidence-report.md` | Report | This file |
