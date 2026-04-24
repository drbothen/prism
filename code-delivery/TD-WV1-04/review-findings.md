# Review Findings — TD-WV1-04

pr: 32
branch: fix/TD-WV1-04-tls-harness-wiring
merged_sha: 4a9dffb1f13464b93fa8a7cfdbfa7c0053bcc024
merged_at: 2026-04-24T04:45:44Z

## Convergence Table

| Cycle | Findings | Blocking | Fixed | Remaining |
|-------|----------|----------|-------|-----------|
| 1 | 4 | 1 | 4 | 0 |
| 2 | 0 | 0 | 0 | 0 → APPROVE |

## Cycle 1 Findings

| ID | Severity | Category | Description | Resolution |
|----|----------|----------|-------------|------------|
| MEDIUM-001 | MEDIUM | Code fix | TLS server task leaked by stop_all() — axum_server::Handle dropped after listening(); ports never released by stop_all() | FIXED commit cd6ae685: added tls_handle field (cfg-gated) to all 6 clone structs; stop() calls handle.graceful_shutdown(5s) before abort |
| SUGGESTION-001 | SUGGESTION | Behavioral asymmetry | TLS shutdown asymmetry vs HTTP graceful drain | DEFERRED as TD item post-merge |
| SUGGESTION-002 | SUGGESTION | Test gap | AC-5 test doesn't cover TLS + stop_all | DEFERRED as TD item post-merge |
| SUGGESTION-003 | SUGGESTION | Test comment | stdout pipe capture ordering in binary e2e test comment misleading | DEFERRED (acceptable) |

## CI Fix

Format check failed on first push (stable rustfmt collapsed wait_for_url_file signature
and reformatted fp_pos.expect chain in td_wv1_04_binary_tls_e2e.rs). Fixed in commit
0d9c02dc. All subsequent CI runs green.

## Final CI Result (both workflow runs)

| Check | Status |
|-------|--------|
| Format check | pass |
| Clippy (AD-008) | pass |
| Test (x86_64-unknown-linux-gnu) | pass |
| Test (x86_64-unknown-linux-musl) | pass |
| Test (x86_64-apple-darwin) | pass |
| Test (aarch64-apple-darwin) | pass |
| Test (x86_64-pc-windows-msvc) | pass |
| Test (no-default-features) | pass |
| Cargo deny (license + advisory) | pass |
| Cargo audit (RustSec) | pass |
| Semver compatibility | pass |
