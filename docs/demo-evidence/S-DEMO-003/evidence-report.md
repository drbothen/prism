# Demo Evidence Report — S-DEMO-003

**Story:** S-DEMO-003 — Demo Setup Scripts + `prism credential set`/`delete` CLI + Operator Runbook
**Version:** v1.17
**Branch:** feature/S-DEMO-003
**HEAD:** c61b61bd (LOCAL 3-CLEAN converged)
**Recorded:** 2026-06-07
**Recorder:** vsdd-factory:demo-recorder

---

## Keychain ACL Constraint (read first)

The real OS Keychain path on macOS triggers an unsigned-dev-binary Keychain ACL
prompt for any **cross-process** subprocess test that attempts to read/write the
system keychain with an unsigned binary. This is documented in the story under
Architecture Compliance Rules ("Tests use keyring mock-builder override ... NEVER
the real OS Keychain") and in SID-1.

As a result:
- **In-process unit tests** using `InMemoryCredentialStore` mock injection are the
  authoritative behavioral coverage for all Keychain-touching ACs.
- **Live subprocess paths** that need Keychain access are `#[ignore]`'d per SID-1
  with blocking-dependency rationale comments.
- **Live CLI paths that do NOT touch the Keychain** (e.g., exit-code checks with a
  missing config dir, `--help`, grep gates, shellcheck) are demonstrated directly.

All recordings use VHS with FiraCode Nerd Font Mono, 1200x700, Dracula theme.

---

## AC Coverage Table

| AC | Title | Evidence Artifact | Mode | Notes |
|----|-------|-------------------|------|-------|
| AC-001 | demo-setup.sh generates valid prism.toml | AC-001-demo-setup-toml.gif/.webm | Test-backed | Red Gate test `test_BC_2_06_001_demo_setup_generates_valid_prism_toml` — PASS |
| AC-002 | prism start boots with Tier-3a OrgId-keyed probe | AC-002-boot-probe-tier3a.gif/.webm | Test-backed | TV-BOOT-P-001: `test_BC_2_06_003_boot_probe_tier3a_finds_org_id_keyed_credential` — PASS; closes F-P14-CRIT-001 |
| AC-003 | demo-run.sh starts/stops DTU cleanly | AC-003-004-006-runbook.gif/.webm | Live (structure) | DTU binary requires full deployment; recording shows overlay generation code paths in script |
| AC-004 | DEMO-RUNBOOK.md §4 Connecting Claude Code | AC-003-004-006-runbook.gif/.webm | Live | grep confirms "Connecting Claude Code" section present with settings.json instructions |
| AC-005 | `prism credential set` reads from stdin, no echo | AC-005-credential-set-no-echo.gif/.webm | Live + test-backed | Live: `--value` flag rejected by clap; In-process: `test_handle_credential_set_writes_org_id_keyed_namespace` — PASS |
| AC-006 | Runbook §Troubleshooting covers 4 failure modes | AC-003-004-006-runbook.gif/.webm | Live | grep confirms E-CRED-008 present; §6(a)-§6(d) headers verified |
| AC-007 | demo-teardown.sh removes keyring entries | AC-007-credential-delete.gif/.webm | Test-backed | `test_handle_credential_delete_uses_org_id_keyed_namespace` (F-P10-HIGH-001) — PASS |
| AC-008 | All demo scripts pass shellcheck locally | AC-008-014-shellcheck.gif/.webm | Live | `shellcheck scripts/demo-*.sh` — 0 errors, 0 warnings |
| AC-009 | Credential written by set_by_org resolves at Tier-3 | AC-009-010-tier3-e2e-resolution.gif/.webm | Test-backed | RG-034-001 `test_BC_2_06_003_tier3_credential_written_by_set_by_org_is_resolved` — PASS; full DTU demo gated on deployed DTU + keychain permit (SID-1) |
| AC-010 | OrgId-keyed write == read; slug-keyed invisible | AC-009-010-tier3-e2e-resolution.gif/.webm | Test-backed | RG-034-004 `test_handle_credential_set_writes_org_id_keyed_namespace` — PASS; CRIT-2 regression: `test_BC_2_06_003_crit2_slug_keyed_write_invisible_to_org_id_keyed_read` — PASS |
| AC-011 | Tier-3 miss → Tier-4 fall-through; backend error → E-CRED-008 | AC-011-tier3-error-semantics.gif/.webm | Test-backed | RG-034-002 (miss case) + RG-034-005 (backend error case) — both PASS |
| AC-012 | Missing prism.toml + no --org-slug → exit 2 | AC-012-missing-toml-error.gif/.webm | Live + test-backed | Live: `prism credential set` with missing config dir → exit 2; RG-034-003 unit test — PASS |
| AC-013 | Retired env-var formats absent from scripts/docs | AC-013-env-format-grep-gates.gif/.webm | Live | Both grep gates return 0 matches (PASS) |
| AC-014 | shellcheck CI job in .github/workflows/ci.yml | AC-008-014-shellcheck.gif/.webm | Live | `shellcheck-demo-scripts` job confirmed in ci.yml |

---

## Red Gate Test Coverage

All 9 Red Gate tests defined in the story pass. Verified by nextest run:

| Test Name | Story RG ID | AC | Crate | Result |
|-----------|-------------|-----|-------|--------|
| `test_BC_2_06_001_demo_setup_generates_valid_prism_toml` | — | AC-001 | prism-bin | PASS |
| `test_BC_2_06_003_boot_probe_tier3a_finds_org_id_keyed_credential` | TV-BOOT-P-001 | AC-002/AC-009 | prism-bin | PASS |
| `test_handle_credential_set_writes_org_id_keyed_namespace` | RG-034-004 | AC-005/AC-010 | prism-bin | PASS |
| `test_handle_credential_delete_uses_org_id_keyed_namespace` | F-P10-HIGH-001 | AC-007 | prism-bin | PASS |
| `test_resolve_org_slug_errors_when_toml_missing_and_no_explicit_slug` | RG-034-003 | AC-012 | prism-bin | PASS |
| `test_BC_2_06_003_crit2_slug_keyed_write_invisible_to_org_id_keyed_read` | CRIT-2 | AC-010 | prism-bin | PASS |
| `test_BC_2_06_003_tier3_credential_written_by_set_by_org_is_resolved` | RG-034-001 | AC-009 | prism-credentials | PASS (with --features test-helpers) |
| `test_BC_2_06_003_tier3_miss_falls_through_to_tier4` | RG-034-002 | AC-011 Case A | prism-credentials | PASS (with --features test-helpers) |
| `test_BC_2_06_003_tier3_backend_error_returns_e_cred_008` | RG-034-005 | AC-011 Case B | prism-credentials | PASS (with --features test-helpers) |

**Total prism-bin tests:** 133 passed, 15 skipped (0 failed)
**Total prism-credentials tests:** 127 passed, 0 failed (3 tier3 tests require `--features test-helpers`)

---

## Gating Rationale by AC

### ACs backed by live CLI demos

- **AC-005 (--value rejected):** The `prism credential set --value secret` invocation
  is demonstrated live — clap rejects the unknown `--value` argument. The full keyring
  write path uses `InMemoryCredentialStore` injection in-process (SID-1 §4).

- **AC-012 (missing toml → exit 2):** Demonstrated live: `prism --config-dir /tmp/no-such-dir
  credential set` exits 2 with the exact error message specified in the story. The
  error path does not touch the Keychain.

- **AC-008 / AC-014 (shellcheck):** `shellcheck scripts/demo-*.sh` runs live, producing
  zero errors/warnings. The CI yml step is confirmed by grep.

- **AC-013 (grep gates):** Both grep patterns run live against `scripts/` and `docs/`
  with zero matches.

### ACs gated by Keychain ACL

- **AC-002 (prism start full boot):** A full `prism start` boot requires Keychain access
  to resolve OrgId-keyed credentials at step 5. The TV-BOOT-P-001 unit test provides
  authoritative behavioral coverage without the real Keychain.

- **AC-009 (full demo query path):** The end-to-end query path (DTU clones + prism start
  + MCP query) requires: (a) deployed DTU clones, (b) prism start reading from Keychain.
  RG-034-001 provides authoritative coverage of the Tier-3 resolution chain.

### ACs gated by DTU deployment

- **AC-003 (demo-run.sh starts DTU cleanly):** The DTU demo server binary is present
  at `target/release/prism-dtu-demo-server` but starting it live and waiting for
  `urls.json` within a VHS recording is not reliable (timing, port binding). The
  recording shows the overlay generation code paths in the script and the 30s poll
  mechanism.

---

## Recordings Index

```
docs/demo-evidence/S-DEMO-003/
  AC-001-demo-setup-toml.gif          (131KB) — AC-001
  AC-001-demo-setup-toml.webm         (121KB) — AC-001
  AC-001-demo-setup-toml.tape         — VHS source
  AC-002-boot-probe-tier3a.gif        (119KB) — AC-002
  AC-002-boot-probe-tier3a.webm       (127KB) — AC-002
  AC-002-boot-probe-tier3a.tape       — VHS source
  AC-003-004-006-runbook.gif          (148KB) — AC-003, AC-004, AC-006
  AC-003-004-006-runbook.webm         (190KB) — AC-003, AC-004, AC-006
  AC-003-004-006-runbook.tape         — VHS source
  AC-005-credential-set-no-echo.gif   (233KB) — AC-005
  AC-005-credential-set-no-echo.webm  (286KB) — AC-005
  AC-005-credential-set-no-echo.tape  — VHS source
  AC-007-credential-delete.gif        (124KB) — AC-007
  AC-007-credential-delete.webm       (128KB) — AC-007
  AC-007-credential-delete.tape       — VHS source
  AC-008-014-shellcheck.gif           (120KB) — AC-008, AC-014
  AC-008-014-shellcheck.webm          (139KB) — AC-008, AC-014
  AC-008-014-shellcheck.tape          — VHS source
  AC-009-010-tier3-e2e-resolution.gif (251KB) — AC-009, AC-010
  AC-009-010-tier3-e2e-resolution.webm(428KB) — AC-009, AC-010
  AC-009-010-tier3-e2e-resolution.tape — VHS source
  AC-011-tier3-error-semantics.gif    (111KB) — AC-011
  AC-011-tier3-error-semantics.webm   (156KB) — AC-011
  AC-011-tier3-error-semantics.tape   — VHS source
  AC-012-missing-toml-error.gif       (206KB) — AC-012
  AC-012-missing-toml-error.webm      (333KB) — AC-012
  AC-012-missing-toml-error.tape      — VHS source
  AC-013-env-format-grep-gates.gif    (123KB) — AC-013
  AC-013-env-format-grep-gates.webm   (125KB) — AC-013
  AC-013-env-format-grep-gates.tape   — VHS source
  evidence-report.md                  — this file
```

---

## Attestation

All 14 acceptance criteria are covered. Coverage is either live-demonstrated (CLI
invocations, grep gates, shellcheck) or test-backed with documented gating rationale
(Keychain ACL / DTU deployment dependency). No AC is silently skipped. The 9 Red Gate
tests all pass on the feature branch.
