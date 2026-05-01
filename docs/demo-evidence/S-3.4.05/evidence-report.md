# Demo Evidence Report — S-3.4.05

**Story:** S-3.4.05 — Migrate prism-dtu-slack/pagerduty/jira tests to prism-dtu-harness (shared-mode)
**Anchor BCs:** BC-3.2.004, BC-3.3.001, BC-3.5.001
**Track:** Platform Engineering / MSSP Coordination DTU Migration
**Recorded:** 2026-04-30
**Tool:** VHS 0.10.0 (CLI recordings)
**Branch:** feature/S-3.4.05

---

## Coverage Map

| Demo ID | Acceptance Criterion | BC Postcondition | VP | Pass/Fail | Artifacts |
|---------|---------------------|------------------|----|-----------|-----------|
| AC-001 | All three harness_tests suites green (24+28+39=91 tests) | BC-3.5.001 PC-1 | VP-087, VP-088, VP-089 | PASS | [gif](AC-001-harness-migration-tests-green.gif) [webm](AC-001-harness-migration-tests-green.webm) [tape](AC-001-harness-migration-tests-green.tape) |
| AC-002 | OrgId UUID in payload body, not in HTTP headers/URL (BC-3.2.004) | BC-3.2.004 PC-1 | VP-090 | PASS | [gif](AC-002-shared-mode-org-id-tagging.gif) [webm](AC-002-shared-mode-org-id-tagging.webm) [tape](AC-002-shared-mode-org-id-tagging.tape) |
| AC-003 | Multi-org logical isolation — payloads from distinct OrgIds not mixed | BC-3.5.001 PC-2 | VP-122 | PASS | [gif](AC-003-multi-org-logical-isolation.gif) [webm](AC-003-multi-org-logical-isolation.webm) [tape](AC-003-multi-org-logical-isolation.tape) |
| AC-004 | client-mode override does not produce startup error (BC-3.3.001 EC-003) | BC-3.5.001 Pre-2, BC-3.3.001 EC-003 | VP-123 | PASS | [gif](AC-004-client-mode-override-no-startup-error.gif) [webm](AC-004-client-mode-override-no-startup-error.webm) [tape](AC-004-client-mode-override-no-startup-error.tape) |
| AC-005 | Harness regression-safe — 91/91 harness_tests clean | BC-3.5.001 PC-1 | VP-124 | PASS | [gif](AC-005-harness-regression-safe.gif) [webm](AC-005-harness-regression-safe.webm) [tape](AC-005-harness-regression-safe.tape) |
| AC-006 | Legacy tests still pass — ac_tests + fidelity + org_tagging all green | BC-3.5.001 PC-1 | VP-087, VP-088, VP-089 | PASS | [gif](AC-006-legacy-tests-still-pass.gif) [webm](AC-006-legacy-tests-still-pass.webm) [tape](AC-006-legacy-tests-still-pass.tape) |

**Coverage: 6/6 must-demo criteria recorded**

---

## AC-001 — Harness Migration Tests Green (All 3 Crates)

**Traces to:** BC-3.5.001 postcondition 1, VP-087, VP-088, VP-089
**Command:** `cargo test -p prism-dtu-slack -p prism-dtu-pagerduty -p prism-dtu-jira --test harness_tests --features dtu 2>&1 | grep -E 'running [0-9]+ test|test result'`
**Expected:** 24 + 28 + 39 = 91 tests pass; 0 failed across all three crates
**Observed:**
```
running 24 tests
test result: ok. 24 passed; 0 failed; 0 ignored; finished in 0.16s
running 28 tests
test result: ok. 28 passed; 0 failed; 0 ignored; finished in 0.09s
running 39 tests
test result: ok. 39 passed; 0 failed; 0 ignored; finished in 0.50s
```

**Recording:** `AC-001-harness-migration-tests-green.gif` / `.webm`

All three `harness_tests.rs` suites pass under `IsolationMode::Logical` with `mode = "shared"`.
The Slack suite includes 24 tests, PagerDuty 28, and Jira 39. Each test builds the harness
via `HarnessBuilder` and exercises the MSSP Coordination DTU as a shared-mode clone.

---

## AC-002 — Shared-Mode X-Prism-Org-Id Tagging (BC-3.2.004)

**Traces to:** BC-3.2.004 postcondition (OrgId in payload body not routing headers)
**Command:** `cargo test -p prism-dtu-slack -p prism-dtu-pagerduty -p prism-dtu-jira --test org_tagging --features dtu 2>&1 | grep -E 'org_id|running [0-9]+ test|test result'`
**Expected:** org_tagging suites pass; 7+8+8=23 tests green; OrgId UUID in body assertions all pass
**Observed:** 7 Slack org_tagging + 8 PagerDuty org_tagging + 8 Jira org_tagging = 23 tests, all green

**Recording:** `AC-002-shared-mode-org-id-tagging.gif` / `.webm`

Key tests recorded:
- `test_BC_3_2_004_org_id_in_payload_body` (Slack) — UUID in Block Kit context block
- `test_BC_3_2_004_ac001_org_id_in_incident_record` (PagerDuty) — UUID in `custom_details`
- `test_BC_3_2_004_ac001_org_id_in_issue_record` (Jira) — UUID in designated issue field
- `test_BC_3_2_004_org_id_not_in_http_url` (Slack) — confirms absence from URL/headers
- `test_BC_3_2_004_ac002_org_id_absent_from_routing` (PD/Jira) — absence from routing confirmed

---

## AC-003 — Multi-Org Logical Isolation in Shared Mode

**Traces to:** BC-3.5.001 postcondition 2, VP-122
**Command:** `cargo test -p prism-dtu-slack -p prism-dtu-pagerduty -p prism-dtu-jira --features dtu ac_multi_org_logical_isolation_shared_mode 2>&1 | grep -E 'ac_multi_org|test result'`
**Expected:** `ac_multi_org_logical_isolation_shared_mode` passes in all three crates
**Observed:** 3 × `test result: ok. 1 passed` — one per crate

**Recording:** `AC-003-multi-org-logical-isolation.gif` / `.webm`

Each `ac_multi_org_logical_isolation_shared_mode` test dispatches payloads for two distinct
`OrgId` values in sequence through the single shared clone. The test asserts that both
payloads are captured, that each contains the correct `OrgId` UUID, and that neither payload
contains the other org's UUID. This verifies EC-002 (two orgs in sequence) from the story.

---

## AC-004 — Client-Mode Override Does Not Produce Startup Error (BC-3.3.001)

**Traces to:** BC-3.5.001 precondition 2, BC-3.3.001-startup EC-003
**Command:** `cargo test -p prism-dtu-slack -p prism-dtu-pagerduty -p prism-dtu-jira --features dtu ac_client_mode_override_does_not_produce_startup_error 2>&1 | grep -E 'client_mode|test result'`
**Expected:** `ac_client_mode_override_does_not_produce_startup_error` passes in all three crates
**Observed:** 3 × `test result: ok. 1 passed`

**Recording:** `AC-004-client-mode-override-no-startup-error.gif` / `.webm`

MSSP Coordination types (Slack, PagerDuty, Jira) permit `mode = "client"` override in a
`CustomerSpec`. Unlike Security Telemetry DTUs, the shared-mode guard in BC-3.3.001-startup
does not apply. The harness builds successfully with `mode = "client"` and the DTU clone
starts without error (EC-003).

---

## AC-005 — Harness Regression-Safe (91/91)

**Traces to:** BC-3.5.001 (VP-123, VP-124)
**Command:** `cargo test -p prism-dtu-slack -p prism-dtu-pagerduty -p prism-dtu-jira --test harness_tests --features dtu 2>&1 | grep -E 'running [0-9]+ test|test result: ok'`
**Expected:** All 91 harness-based tests pass; no regressions
**Observed:** 24 + 28 + 39 = 91 tests, all `test result: ok`

**Recording:** `AC-005-harness-regression-safe.gif` / `.webm`

The `prism-dtu-harness` library was not modified by this migration story. Evidence of
regression-safety is the full 91-test green run across all three migrated crates, each
consuming `prism-dtu-harness` as a dev-dependency.

---

## AC-006 — Legacy Tests Still Pass

**Traces to:** BC-3.5.001 postcondition 1 — no regression in ac_tests/fidelity/org_tagging
**Command:** `cargo test -p prism-dtu-slack -p prism-dtu-pagerduty -p prism-dtu-jira --features dtu 2>&1 | grep -E 'running [0-9]+ test|test result: ok'`
**Expected:** All legacy suites pass alongside the new harness_tests
**Observed:**
- Slack: ac_tests (14), fidelity (1), org_tagging (7), harness_tests (24) = 46 total
- PagerDuty: fidelity (17), org_tagging (8), harness_tests (28) = 53 total
- Jira: fidelity (28), org_tagging (8), harness_tests (39) = 75 total

**Recording:** `AC-006-legacy-tests-still-pass.gif` / `.webm`

The migration to harness is additive. All pre-existing `ac_tests.rs`, `fidelity.rs`, and
`org_tagging.rs` tests continue to pass unchanged. The harness tests are a new layer on top.

---

## Artifacts Inventory

| File | Type | Size | Purpose |
|------|------|------|---------|
| `AC-001-harness-migration-tests-green.tape` | VHS script | 1.1 KB | Reproducible recording source |
| `AC-001-harness-migration-tests-green.gif` | GIF recording | 94 KB | PR embed |
| `AC-001-harness-migration-tests-green.webm` | WebM recording | 87 KB | Archival |
| `AC-002-shared-mode-org-id-tagging.tape` | VHS script | 1.2 KB | Reproducible recording source |
| `AC-002-shared-mode-org-id-tagging.gif` | GIF recording | 61 KB | PR embed |
| `AC-002-shared-mode-org-id-tagging.webm` | WebM recording | 58 KB | Archival |
| `AC-003-multi-org-logical-isolation.tape` | VHS script | 1.1 KB | Reproducible recording source |
| `AC-003-multi-org-logical-isolation.gif` | GIF recording | 206 KB | PR embed |
| `AC-003-multi-org-logical-isolation.webm` | WebM recording | 97 KB | Archival |
| `AC-004-client-mode-override-no-startup-error.tape` | VHS script | 1.2 KB | Reproducible recording source |
| `AC-004-client-mode-override-no-startup-error.gif` | GIF recording | 211 KB | PR embed |
| `AC-004-client-mode-override-no-startup-error.webm` | WebM recording | 97 KB | Archival |
| `AC-005-harness-regression-safe.tape` | VHS script | 1.2 KB | Reproducible recording source |
| `AC-005-harness-regression-safe.gif` | GIF recording | 95 KB | PR embed |
| `AC-005-harness-regression-safe.webm` | WebM recording | 89 KB | Archival |
| `AC-006-legacy-tests-still-pass.tape` | VHS script | 1.2 KB | Reproducible recording source |
| `AC-006-legacy-tests-still-pass.gif` | GIF recording | 66 KB | PR embed |
| `AC-006-legacy-tests-still-pass.webm` | WebM recording | 65 KB | Archival |
| `evidence-report.md` | This file | — | Coverage mapping |

---

## Reproducibility Notes

- All tapes use `cd /Users/jmagady/Dev/prism/.worktrees/S-3.4.05` hidden in setup (not visible in recording).
- All tapes use `Wait+Screen /test result/` — waits for actual command completion, not a fixed sleep.
- Recordings pipe through `grep` to trim cargo build noise; only test result lines are shown.
- Font: `FiraCode Nerd Font Mono` (confirmed installed at `/Users/jmagady/Library/Fonts/`).
- VHS `Wait+Line` does not work reliably in VHS 0.10.0 with piped commands; `Wait+Screen` used instead.
- All recordings produced at 1200x700 with Dracula theme, 20px padding, 14pt font.
- `prism-dtu-harness` itself has compile errors on this branch (unrelated `FailureMode` import from
  a different story's changes); the regression evidence (AC-005) is captured via the 91 harness_tests
  that run through the consumer crates' dev-dep path, which compiles cleanly with `--features dtu`.
