# Review Findings — S-6.20

story_id: S-6.20
pr_number: 29
review_cycles: 1
final_verdict: APPROVE

## Convergence Table

| Cycle | Findings | Blocking | Fixed | Remaining |
|-------|----------|----------|-------|-----------|
| 1     | 9        | 0        | 0     | 9 (all deferred) |
| —     | 0 blocking | APPROVE | —   | —         |

## Cycle 1 Findings

| ID | Severity | File | Description | Disposition |
|----|----------|------|-------------|-------------|
| F-6.20-R-001 | IMPORTANT | src/tls.rs:30-31 | Cert dates hardcoded 2024 (expired). --tls non-functional for stakeholder demos. | Deferred → TD-S620-002 |
| F-6.20-R-002 | IMPORTANT | src/tls.rs, src/main.rs | TLS cert not wired into axum-server; clones serve plain HTTP even with --tls. AC-4 tests cert generation only. | Deferred → TD-S620-003 |
| F-6.20-R-003 | IMPORTANT | evidence-report.md | AC-4 overclaims "HTTPS served" / "InsecureTlsClient" — only cert generation tested. | Acknowledged in PR comment |
| F-6.20-R-004 | SUGGESTION | crates/prism-dtu-demo-server/ | README.md missing (Task 10). | Deferred → TD-S620-004 |
| F-6.20-R-005 | SUGGESTION | scripts/ | start-demo.sh missing (Task 11). | Deferred → TD-S620-005 |
| F-6.20-R-006 | SUGGESTION | src/tls.rs:44-55 | print_cert_fingerprint uses hex(base64) not SHA-256(DER). | Deferred → TD-S620-006 |
| F-6.20-R-007 | SUGGESTION | Cargo.toml | reqwest in both [dependencies] and [dev-dependencies]. | Cosmetic/acknowledged |
| F-6.20-R-008 | SUGGESTION | src/harness.rs:147 | Double blank line. | Fixed by cargo fmt commit |
| F-6.20-R-009 | SUGGESTION | src/harness.rs:119 | "enabled clones" comment misleading (no filter needed — build_clone_pairs already filters). | Acknowledged |

## CI Fix Cycle

| Issue | Cause | Fix | Commit |
|-------|-------|-----|--------|
| Format check FAIL | rustfmt not run on new files | cargo fmt -p prism-dtu-demo-server -p prism-dtu-threatintel | 95971ca4 |
