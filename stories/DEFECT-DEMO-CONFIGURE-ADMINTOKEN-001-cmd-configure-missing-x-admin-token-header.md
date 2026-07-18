---
document_type: story
story_id: "DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001"
title: "cmd_configure missing X-Admin-Token header — POST /dtu/configure returns 401"
wave: maintenance
epic_id: maintenance
priority: P1
status: merged
merged_sha: "277b7844"
merged_pr: "225"
merged_date: "2026-07-18"
version: "0.21"
level: ops
producer: story-writer
timestamp: "2026-07-16"
modified: "2026-07-18"
input-hash: "0b092d4"
inputs:
  - crates/prism-dtu-demo-server/src/main.rs
  - crates/prism-dtu-demo-server/src/multi_instance.rs
  - crates/prism-dtu-demo-server/src/multi_org_cmd.rs
  - crates/prism-dtu-demo-server/src/harness.rs
  - crates/prism-dtu-demo-server/src/lib.rs
  - .factory/specs/behavioral-contracts/BC-3.6.001-per-org-failure-injection.md
  - .factory/specs/behavioral-contracts/BC-2.06.017-dtu-per-instance-multi-address-binding.md
  - .factory/specs/architecture/decisions/ADR-003-dtu-reset-lookup-and-fidelity-auth.md
  - .factory/specs/prd-supplements/error-taxonomy.md
traces_to: ""
origin_finding: "DRIFT-DEMO-CONFIGURE-ADMINTOKEN-001"
origin_cascade: "Human-authorized 2026-07-16 (D-TBD); P1 demo-blocking — North Star is multi-client SOC-analyst live demo"
cycle: "v1.0.0-greenfield"
phase: 3
tdd_mode: strict
track: "Platform Engineering"
subsystems: [SS-01]
# Subsystem anchor justification:
#   SS-01 (Sensor Adapters) owns this story's scope because `prism-dtu-demo-server`
#   is explicitly listed in the SS-01 crate column of the ARCH-INDEX Subsystem Registry
#   (v2.193, Subsystem Registry table, line 154). SS-01 crates include all DTU crates:
#   prism-dtu-common, prism-dtu-claroty, prism-dtu-armis, prism-dtu-crowdstrike,
#   prism-dtu-cyberint, prism-dtu-slack, prism-dtu-pagerduty, prism-dtu-jira,
#   prism-dtu-nvd, prism-dtu-threatintel, prism-dtu-demo-server, prism-dtu-harness.
#   SS-22 (Process Lifecycle) is scoped exclusively to prism-bin boot orchestration
#   (ADR-022 §B 11-step boot, startup failure exit-code map, traffic gate signal
#   handlers) per ARCH-INDEX v2.193 line 175 — it does not cover prism-dtu-demo-server.
#   Per ARCH-INDEX Subsystem Registry SS-01 (line 154) and SS-22 (line 175).
crates_touched:
  - prism-dtu-demo-server
  - prism-bin # doc-comment-only test-helper cleanup (helpers/mod.rs stale-reference removal, TD-VSDD-060 sibling-sweep byproduct; no functional surface)
target_module: "crates/prism-dtu-demo-server"
behavioral_contracts:
  - BC-3.6.001
  - BC-2.06.017
# BC status: Both BCs are active (lifecycle_status: active).
#
# BC-3.6.001 ("Per-Org Failure Injection") Precondition 4 explicitly states:
#   "The inject_failure call uses POST /dtu/configure on the clone's admin endpoint,
#   authenticated with that clone's admin_token (ADR-003 Amendment §5)."
#   This is the normative anchor for the X-Admin-Token requirement on ANY configure call
#   from client code. cmd_configure is a direct caller — its missing header is a
#   violation of this precondition.
#
# BC-2.06.017 ("Per-DTU-Instance Multi-Address Binding for Multi-Tenant Overlay Testing")
#   Postcondition 1 governs MultiInstanceServers (start_instances return type).
#   The defect fix requires adding admin_token_map() to MultiInstanceServers so that
#   cmd_configure can obtain tokens in start-multi mode. This is an extension of the
#   BC's runtime deliverable. BC-2.06.017 also owns the URL sidecar infrastructure
#   that cmd_configure reads; the admin token sidecar is a parallel artifact in the
#   same mechanical pattern.
#
# AC↔BC bidirectional traces required before status=ready (S-7.01 gate).
verification_properties: []
depends_on: []
blocks: []
points: 5
estimated_days: 1.5
risk: HIGH
acceptance_criteria_count: 4
red_gate_tests: 4
estimated_passes: "3-4"
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
---

# DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001: cmd_configure missing X-Admin-Token header — POST /dtu/configure returns 401

## Narrative
- **As a** SOC-analyst demo operator
- **I want to** run `prism-dtu-demo-server configure <clone> ...` to inject failure modes or reset clone state mid-demo
- **So that** the demo server's operator recovery path (EC-007 in `resolve_configure_url`) functions correctly instead of silently failing with HTTP 401 on every invocation

## §Origin — [defect] DRIFT-DEMO-CONFIGURE-ADMINTOKEN-001

**Human authorization:** D-TBD (2026-07-16); P1 demo-blocking  
**North Star impact:** Multi-client SOC-analyst live demo — `prism-dtu-demo-server configure`
is the operator recovery path (EC-007 documented in `resolve_configure_url`) for injecting
failure modes and resetting clone state mid-demo. It is broken on every clone for every
`start` and `start-multi` invocation.

The `configure` subcommand (`cmd_configure` in `crates/prism-dtu-demo-server/src/main.rs`)
POSTs to a DTU clone's `/dtu/configure` endpoint without the `X-Admin-Token` header required
by ADR-003 Amendment #5. Every DTU clone rejects the request with HTTP 401 and JSON body
`{"error": "missing or invalid X-Admin-Token"}`. The error is then surfaced to the CLI as a
non-zero exit, but the reconfiguration never takes effect.

### Root Cause

ADR-003 Amendment #5 §Decision states:

> `POST /dtu/configure` on every DTU clone MUST require a valid `X-Admin-Token`
> header. The token value is a per-instance UUID v4 generated at clone construction
> time and accessible via the new `BehavioralClone::admin_token()` trait method.
> Requests missing the header, or presenting an incorrect token, receive HTTP 401
> with `{"error": "missing or invalid X-Admin-Token"}`.

ADR-003 Amendment #5 §Implementation item 4 states:

> All 12 existing `td_wv0_04` configure tests (2 per clone) and all other
> integration tests calling `/dtu/configure`: updated to include
> `.header("X-Admin-Token", clone.admin_token())`.

The test callers in `tests/ac_3_configure_endpoint.rs` were implemented correctly (they call
`ClonePair::admin_token()` which delegates to `BehavioralClone::admin_token()`).
The `cmd_configure` CLI path in `main.rs` was not implemented — it is not an integration test
and was therefore not covered by item 4's update scope.

**Secondary root cause:** There is no mechanism for `cmd_configure` — which runs in a
SEPARATE process invocation after the demo server is already running — to obtain each
clone's admin token. The URL sidecar (`.prism-dtu-demo-server.urls.json`) provides clone
base-URLs but not tokens. An admin token sidecar must be written at server-start time and
read by `cmd_configure`. For `start-multi` mode, `MultiInstanceServers` moves clones into
detached watcher tasks before the token could be read, so token extraction must happen BEFORE
the move.

### TD-VSDD-060 Full Sibling Enumeration — All `/dtu/configure` POST Sites in Client Code

| Site | File | Line range (approx.) | X-Admin-Token present? | Status |
|------|------|---------------------|----------------------|--------|
| `cmd_configure()` | `crates/prism-dtu-demo-server/src/main.rs` | ~561-572 | **NO → YES (FIXED)** | **DEFECT SITE** — was missing header; T-08 adds `resolve_configure_token` + `X-Admin-Token` |
| Test `ac_3_configure_called_on_clone_port_directly` | `crates/prism-dtu-demo-server/tests/ac_3_configure_endpoint.rs` | ~44 | YES | Correct |
| Test `ac_3_no_harness_proxy_for_configure` | `crates/prism-dtu-demo-server/tests/ac_3_configure_endpoint.rs` | ~94 | YES | Correct |
| Test `configure_with_correct_token_returns_200` | `crates/prism-dtu-crowdstrike/tests/td_wv0_07_configure_requires_admin_token.rs` | ~74 | YES | Correct |
| `configure_with_correct_token_returns_200` (claroty/cyberint/armis/nvd/threatintel/jira/pagerduty `td_wv0_07_*`) | Each DTU crate's `td_wv0_07_configure_requires_admin_token.rs` | ~74 per file | YES | Correct — 7 additional clone-specific positive tests (test 3 of 3 per file) |
| `configure_failure` helper (the BC-3.6.001 test itself) | `crates/prism-dtu-harness/tests/bc_3_6_001_ops_clone_failure_modes.rs` | varies | YES (lowercase `x-admin-token`; token sourced via `Harness::admin_token_for()`, not `clone.admin_token()`) | Correct |
| `assert_configure_strict` helper (deny-unknown-fields test) | `crates/prism-dtu-harness/tests/review_2026_06_10_deny_unknown.rs` | varies | YES (same pattern — lowercase `x-admin-token` via `harness.admin_token_for()`) | Correct |
| `Harness::inject_failure()` production method | `crates/prism-dtu-harness/src/harness.rs` | ~285-295 | YES (lowercase `x-admin-token`; token from `self.admin_tokens.get(...)`) | Correct — harness library implementation backing the `configure_failure` and `assert_configure_strict` callers (rows 7-8 above) |
| `test_build_harness_http_client_timeout_is_load_bearing` unit test | `crates/prism-dtu-harness/src/builder.rs` | ~1224-1270 | N/A | N/A — synthetic hung-socket timeout test; POSTs to a raw `TcpListener` that never responds; no DTU handler present; header not applicable |
| `configure_without_token_returns_401` (all 8 `td_wv0_07_*` per-clone) | `crates/prism-dtu-{crowdstrike,claroty,armis,cyberint,nvd,pagerduty,jira,threatintel}/tests/td_wv0_07_configure_requires_admin_token.rs` | ~14-35 per file | NO (intentional) | Correct — negative test (test 1 of 3): verifies server returns 401 when `X-Admin-Token` header is absent; 8 instances × 1 crate each = 8 POST calls |
| `configure_with_wrong_token_returns_401` (all 8 `td_wv0_07_*` per-clone) | Same 8 `td_wv0_07_` files | ~36-58 per file | WRONG token (intentional) | Correct — negative test (test 2 of 3): verifies server returns 401 when an incorrect token is presented; 8 instances = 8 POST calls |
| All `td_wv0_04_configure_deny_unknown.rs` tests | `crates/prism-dtu-{crowdstrike,claroty,armis,nvd,cyberint,threatintel}/tests/td_wv0_04_configure_deny_unknown.rs` (6 files) | varies | YES (correct token per ADR-003 Amendment #5 update) | Correct — deny-unknown-fields tests; 2 tests × 6 clones = 12 POST calls; all use `clone.admin_token()` |
| Per-DTU-crate `harness_tests.rs` — positive configure paths | `crates/prism-dtu-{crowdstrike,claroty,armis,cyberint}/tests/harness_tests.rs` (4 crates; jira and pagerduty have zero positive configure calls) | varies | YES | Correct — 27 POST calls: claroty:12, armis:7, cyberint:5, crowdstrike:3; configure clone state before behavioral assertions |
| Per-DTU-crate `harness_tests.rs` — negative tests (BC-3.5.001/TD-WV0-07 migrated) | `crates/prism-dtu-{crowdstrike,claroty,armis,cyberint,jira,pagerduty}/tests/harness_tests.rs` (6 files; verified at crowdstrike:2157+2181, cyberint:1218+1248, pagerduty:790, armis:1712+1734, claroty:1692+1714, jira:1057+1079) | varies | NO / WRONG (intentional) | Correct — 11 POST calls: 6 no-token (crowdstrike:1, cyberint:1, pagerduty:1, armis:1, claroty:1, jira:1) + 5 wrong-token (crowdstrike:1, claroty:1, cyberint:1, armis:1, jira:1); migrated BC-3.5.001/TD-WV0-07 negative tests |
| Per-DTU-crate rate-limit, error-injection, reset, and auth-mode `ac_*` tests | `crates/prism-dtu-{armis,claroty,cyberint,nvd,threatintel}/tests/ac_{6,7,8}_*.rs` and similar | varies | YES | Correct — 21 POST calls; configure state before testing rate-limit / error / auth AC scenarios |
| `sec_p3_003_constant_time_admin_token.rs` test A (correct token → 200) | `crates/prism-dtu-claroty/tests/sec_p3_003_constant_time_admin_token.rs` | ~121-145 | YES | Correct — positive test: verifies constant-time comparison returns 200 with correct token |
| `sec_p3_003_constant_time_admin_token.rs` test B (wrong token → 401) | `crates/prism-dtu-claroty/tests/sec_p3_003_constant_time_admin_token.rs` | ~280-305 | WRONG token (intentional) | Correct — negative test: verifies constant-time comparison still returns 401 with incorrect token |
| `edge_cases.rs` (claroty and cyberint) | `crates/prism-dtu-claroty/tests/edge_cases.rs`, `crates/prism-dtu-cyberint/tests/edge_cases.rs` | varies | YES | Correct — 7 POST calls (2 claroty + 5 cyberint); configure state for edge-case scenarios |
| `fidelity.rs` (jira) — positive configure tests | `crates/prism-dtu-jira/tests/fidelity.rs` | ~450-500 | YES | Correct — 2 POST calls setting up fidelity test state |
| `fidelity.rs` (jira) — negative tests (headerless + wrong-token) | `crates/prism-dtu-jira/tests/fidelity.rs` | ~1286-1325 | NO / WRONG (intentional) | Correct — 2 negative tests parallel to `td_wv0_07`: `test_dtu_configure_without_admin_token_returns_401` (headerless) and `test_dtu_configure_with_wrong_admin_token_returns_401` (wrong token) |
| `fidelity.rs` (pagerduty) — positive configure tests | `crates/prism-dtu-pagerduty/tests/fidelity.rs` | ~401-420, ~611-630, ~743-760 | YES | Correct — 3 POST calls setting up fidelity test state |
| `fidelity.rs` (pagerduty) — negative test (headerless → 401) | `crates/prism-dtu-pagerduty/tests/fidelity.rs` | ~800-820 | NO (intentional) | Correct — `test_configure_without_admin_token_returns_401` negative test |
| Misc test POST calls — `multi_tenant.rs` (cyberint), `reset_state_invariants.rs` (armis), `dtu_reset_mount.rs` (threatintel), `ac_4/6/7_*.rs` (threatintel), `ac_tests.rs` (slack) | Various DTU crates | varies | YES | Correct — ~11 POST calls total; all configure clone state for behavioral tests using `clone.admin_token()` |
| `FidelityCheck`-based configure callers — per-DTU `fidelity_validator.rs` files | `crates/prism-dtu-{nvd,claroty,armis,threatintel,cyberint}/tests/fidelity_validator.rs` (5 files) | varies | YES (`method: http::Method::POST`, `headers` field includes `X-Admin-Token`; ADR-003 Amendment #3) | Correct — 5 POST calls (1 per file); `FidelityCheck` struct sets `endpoint: "/dtu/configure"` with correct token in `headers`; `FidelityValidator::run` issues the HTTP POST via `client.request()` internally — not captured by `.post(.*dtu/configure)` grep |
| `FidelityCheck`-based configure callers — `harness_tests.rs` AC-002 FidelityValidator tests | `crates/prism-dtu-{armis,claroty,cyberint}/tests/harness_tests.rs` (3 files) | varies | YES (`method: http::Method::POST`, `headers` field includes `X-Admin-Token`; ADR-003 Amendment #3) | Correct — 3 POST calls (1 per file); same `FidelityCheck`/`FidelityValidator::run` pattern as `fidelity_validator.rs` above; AC-002 fidelity validation tests |
| In-process `clone.configure()` method call — HTTP auth not applicable | `crates/prism-dtu-cyberint/tests/bc_2_01_017_access_token_auth.rs` (line 54: `clone.configure(serde_json::json!({"access_token": "demo-access-key"})).await`); similar setup-only calls in other per-DTU-crate AC test files | N/A | N/A | Not an HTTP POST — calls `BehavioralClone::configure(serde_json::Value)` trait method in-process; mutates clone state directly without sending an HTTP request. `X-Admin-Token` header requirement does not apply. Excluded from the 146 HTTP POST total. |
| `defect_demo_configure_admintoken_001.rs` Test A (contract lock) | `crates/prism-dtu-demo-server/tests/defect_demo_configure_admintoken_001.rs` | ~121-145 | NO (intentional) | Correct — defect-observability contract lock: POST without `X-Admin-Token` → asserts 401; passes both before and after fix |
| `defect_demo_configure_admintoken_001.rs` Test B (post-fix verification) | `crates/prism-dtu-demo-server/tests/defect_demo_configure_admintoken_001.rs` | ~229-250 | YES | Correct — load-bearing post-fix test: sidecar-sourced token → asserts 200; fails before fix, passes after |

> **Sweep stats (v0.12 reconciliation, 2026-07-17):** SWEEP-MIRROR convention: mirror-comment lines in `cmd_configure` (main.rs) and the defect test module header that enumerate `dtu/configure` call-site counts are tagged `// SWEEP-MIRROR`; every command below appends `| grep -v SWEEP-MIRROR` to exclude them and hold counts stable regardless of future mirror additions. These command forms are IDENTICAL to those embedded in the code mirrors. Reproducible commands (run from worktree root, `crates/` subdirectory): (1) `rg 'dtu/configure' crates/ --type rust | grep -v SWEEP-MIRROR | wc -l` → **447 total hits**; (2) `rg '\.post\(.*dtu/configure' crates/ --type rust | grep -v SWEEP-MIRROR | wc -l` → **131 same-line** HTTP POST calls (URL inline in `.post(format!(...))`, literal regex match on same line); (3) `rg 'let url = format.*dtu/configure' crates/ --type rust | grep -v SWEEP-MIRROR | wc -l` → **6 dynamic-URL** construction lines (URL built into a variable, then `.post(&url)` on a separate line); (4) `rg 'endpoint.*"/dtu/configure"' crates/ --type rust | grep -v SWEEP-MIRROR | wc -l` → **8 FidelityCheck-based** configure callers (5 in per-DTU `fidelity_validator.rs` + 3 in `harness_tests.rs` AC-002 tests; `FidelityValidator::run` issues the POST via `client.request()` — not captured by `.post()` grep); dynamic-URL `.post()` callers note: `cmd_configure` in `src/main.rs` calls `.post(&configure_url)` where the URL is resolved via `resolve_configure_url()` — 1 additional dynamic caller not captured by command (3); total dynamic-URL callers = **7**. Grand total: **131 same-line + 7 dynamic + 8 FidelityCheck-based = 146 total HTTP POST client calls**. Per-class tally (v0.12): production CLI POST (defect, now FIXED) 1 · harness library production POST with header 1 · test POST with correct X-Admin-Token 111 · test POST intentionally headerless / no token (negative test) 17 · test POST with wrong token (negative test) 15 · synthetic hung-socket timeout N/A 1. Sum: 146. Arithmetic: prior v0.11 tally 138 + 8 FidelityCheck-based correct-token = 146; correct-token class: 103 + 8 = 111. Note: 11 negative tests (6 no-token + 5 wrong-token) that were previously miscounted as "positive configure paths" in harness_tests.rs are now correctly classified — see the harness_tests negative row in the table above. Excluded from total: in-process `BehavioralClone::configure()` method calls (no HTTP request emitted — see "In-process" row in table above). Future passes: re-run commands (1)-(4) with the SWEEP-MIRROR filter and verify totals match; any unexpected new headerless POST is a CRITICAL defect.

**Result:** `cmd_configure()` in `main.rs` was the ONLY affected client-side call site (the defect, now FIXED). The complete sweep (146 total HTTP POST client calls across all files) confirms no other caller is missing the required header: 111 are correct-token test POSTs (including 8 FidelityCheck-based callers in `fidelity_validator.rs` and `harness_tests.rs` AC-002 tests that pass `X-Admin-Token` via the `FidelityCheck.headers` field; `FidelityValidator::run` issues these POSTs via `client.request()` — they are indirect callers invisible to the `.post()` grep), 17 are intentional no-token negative tests (verifying 401 behavior), 15 are wrong-token negative tests (verifying 401 on bad credentials), and 1 is a synthetic hung-socket timeout unit test with no DTU handler (header N/A). The harness library production method `Harness::inject_failure()` (harness.rs:~287) and the harness test callers that invoke it (`bc_3_6_001_ops_clone_failure_modes.rs`, `review_2026_06_10_deny_unknown.rs`) all correctly attach the `x-admin-token` header via `Harness::admin_token_for()`. No production binary or library path outside the demo server and test harness infrastructure calls `/dtu/configure` directly.

### Excluded from scope (OUT OF SCOPE)

**DRIFT-HARNESS-ADMIN-TOKEN-CT-001** (constant-time comparison on the SERVER side, CWE-208) is
explicitly excluded. That drift item is folded into **S-DRIFT-SAP2-DEVICES-TOML-SURFACE-001**
(D-1666, 2026-07-10) which adds `ct_compare_tokens` to `prism-dtu-harness`. The current story
is CLIENT-SIDE only (`cmd_configure` must send the correct token; the server-side comparison
algorithm is out of scope here).

## Behavioral Contracts

| BC | Title | Version | Relevant Clause |
|----|-------|---------|----------------|
| BC-3.6.001 | Per-Org Failure Injection | v0.8 | Precondition 4: "The inject_failure call uses POST /dtu/configure on the clone's admin endpoint, authenticated with that clone's admin_token (ADR-003 Amendment §5)." This is the normative statement that ALL callers of /dtu/configure must present the admin token. cmd_configure is a caller; its missing header violates this precondition. |
| BC-2.06.017 | Per-DTU-Instance Multi-Address Binding for Multi-Tenant Overlay Testing | v1.12 | Postcondition 1 governs `MultiInstanceServers` (returned by `start_instances()`). The fix extends `MultiInstanceServers` with `admin_token_map()` — a parallel accessor to the existing `socket_map()`. The admin token sidecar uses the same atomic-write pattern as the URL sidecar (atomic tmp+rename; cf. GAP-3 sidecar-poll note, S-DEMO-LAUNCHER-CONSOLIDATION-001 Changelog v2.1). v1.12 corrects a phantom POL-21 anchor citation from v1.11; substance unchanged: enumerates the `admin_token_map() -> &HashMap<String, String>` accessor and `TOKEN_MULTI_FILE` sidecar format in Postcondition 1. |

## Acceptance Criteria

### AC-001 — Failing RED tests: 401 reproduced at each affected call site
(traces to BC-3.6.001 precondition 4 — configure requests must be authenticated)

A new test file `crates/prism-dtu-demo-server/tests/defect_demo_configure_admintoken_001.rs`
contains a **failing** (RED Gate) test that:

1. Starts a harness with at least one clone (e.g., CrowdStrike)
2. Obtains the clone's URL from the harness
3. POSTs `{"auth_mode": "accept"}` to `{url}/dtu/configure` WITHOUT the `X-Admin-Token` header,
   replicating the current `cmd_configure` behavior
4. Asserts the response is HTTP 401 — proving the defect is observable

Note: steps 1-4 form a **defect-observability contract lock** — they pass both before and after the fix (the server already correctly returns 401 for unauthenticated requests; that gate predates this branch). The Red-Gate obligation for this AC is carried by the binary E2E test (asserting `cmd_configure` exits 0 after adding the token header) and the token-sidecar-existence assertions; those tests fail before the fix and pass after.

Additionally, a unit test in the `#[cfg(test)] mod tests` block of
`crates/prism-dtu-demo-server/src/main.rs` (or a suitably scoped integration test)
verifies that `cmd_configure` sends the `X-Admin-Token` header; a test without the header
must return 401 before the fix.

### AC-002 — X-Admin-Token attached from the correct token source at every enumerated site
(traces to BC-3.6.001 precondition 4 — admin_token must be the per-clone random token)

After the fix, `cmd_configure()` in `main.rs`:

1. **Reads the admin token from a token sidecar** — one of:
   - `TOKEN_FILE = ".prism-dtu-demo-server.admin-tokens.json"` (flat `{name: token}`,
     written by `cmd_start` alongside `URL_FILE`)
   - `TOKEN_MULTI_FILE = ".prism-dtu-demo-server.admin-tokens-multi.json"` (nested
     `{org_slug: {sensor_id: token}}`, written by `cmd_start_multi` alongside `URL_MULTI_FILE`)

2. **Applies the same sidecar-lookup precedence** as `resolve_configure_url`:
   flat sidecar first; nested sidecar fallback; bare sensor name disambiguation for
   the multi-org case (EC-007 recovery form documented in `resolve_configure_url` — distinct from this story's §Edge Cases EC-007)

3. **Includes `X-Admin-Token: <token>` in the POST request** — verified by a test that
   asserts the response is HTTP 200 (not 401) when calling a clone that requires the header

Token sidecar write requirements:
- `cmd_start()` MUST write `TOKEN_FILE` atomically immediately after `write_url_sidecar` —
  via the `write_token_sidecar` binary wrapper in `main.rs`, which delegates to
  `write_token_sidecar_to_path` in `src/harness.rs` (re-exported via `lib.rs`);
  uses tmp+rename atomic pattern — format: `{clone_name: admin_token}` where
  each key matches the corresponding key in `URL_FILE`
- `cmd_start_multi()` (via a new helper, parallel to `write_multi_url_sidecar`) MUST write
  `TOKEN_MULTI_FILE` atomically — format: `{org_slug: {sensor_id: admin_token}}` mirroring
  `URL_MULTI_FILE`
- `TOKEN_MULTI_FILE` MUST include the reserved `_global` key carrying admin tokens for all
  enabled enrichment DTU clones (`KNOWN_ENRICHMENT_CLONES`), mirroring the `URL_MULTI_FILE`
  `_global` pattern (ENRICH-3); `write_multi_admin_token_sidecar_to_path` MUST fail-loud
  (propagate an error) when an enabled enrichment clone's token is absent from
  `admin_token_map` — contract-locked by Test K (enrichment `_global` token lock)
- `MultiInstanceServers` MUST expose `admin_token_map() -> &HashMap<String, String>` —
  tokens captured from each clone via `clone.admin_token().to_string()` BEFORE the clone is
  moved into its detached watcher task in `start_instances()` (in the watcher-spawn loop of
  `start_instances` in `multi_instance.rs`)
- Token sidecar files MUST be removed on shutdown alongside the corresponding URL sidecar
  files in `wait_for_shutdown_signal()` and `wait_for_shutdown_signal_multi()`
- `TOKEN_FILE` and `TOKEN_MULTI_FILE` constants MUST be declared as `pub const` in `lib.rs`
  alongside `URL_FILE` / `URL_MULTI_FILE`

### AC-003 — Missing/empty token config → E-DEMO-007 structured error, NOT panic, NOT silent 401 swallow
(traces to BC-3.6.001 precondition 4 — misconfiguration must surface as a diagnostic, not a
silent/opaque failure)

If `cmd_configure()` cannot resolve the admin token for the requested clone (because no token
sidecar exists, or the clone name is not present in the sidecar), it MUST:

1. Return `Err(anyhow::Error)` with message matching: `"configure: E-DEMO-007: admin token for clone '{clone_name}' could not be resolved: {reason}"`
2. Print the error to stderr and exit with code 1 (the existing `anyhow::Result<()>` pattern)
3. NOT proceed to POST `/dtu/configure` (which would silently receive 401 and leave the user
   puzzled about the server's response)
4. NOT panic

Error reasons include:
- `"token sidecar not found (start the demo server first with start or start-multi)"`
- `"clone '{clone_name}' not found in token sidecar '{path}'"`

**§Error Taxonomy Addition** (NEW — must be registered in `.factory/specs/prd-supplements/error-taxonomy.md`
under `## DEMO: Demo-Server Errors` before this story merges, per POL-24):

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|----------------|-----------|-------------|
| E-DEMO-007 | broken | configuration | `"configure: E-DEMO-007: admin token for clone '{clone_name}' could not be resolved: {reason}"` | No | Runtime error in `cmd_configure`: the admin token sidecar (`TOKEN_FILE` or `TOKEN_MULTI_FILE`) is absent or does not contain an entry for `{clone_name}`. Occurs when the configure subcommand is invoked but the demo server is not running or was started in a different working directory. Operator must ensure the demo server is running in the same working directory and that the appropriate sidecar file exists. |

### AC-004 — Sibling sweep evidence: all /dtu/configure POST sites confirmed covered
(traces to BC-2.06.017 postcondition 1 — the admin_token_map on MultiInstanceServers is
the closing mechanism for start-multi mode)

A SWEEP-MIRROR comment block in the `cmd_configure` function body (and a corresponding
mirror block in `defect_demo_configure_admintoken_001.rs`) documents the TD-VSDD-060
sibling sweep. Per the v0.10 ratified SWEEP-MIRROR convention, the code mirrors carry:
(a) byte-identical reproducible command forms (commands (1)-(4) from §Root Cause, each
filtered with `| grep -v SWEEP-MIRROR | wc -l`), (b) stable counts (447/131/6/8), and
(c) a condensed site-group table. The exhaustive per-site enumeration lives in the story
§Root Cause TD-VSDD-060 table (~24 live rows / 146 HTTP POST client calls), which is the
source of truth for the sweep. `cmd_configure` was the ONLY defect site — confirmed by
the complete sweep of 146 total HTTP POST client calls. The new test in AC-001 explicitly
references this SWEEP-MIRROR block.

The implementer MUST execute the following two-step reconciliation sweep and include the
full output of both commands plus the reconciliation table in the PR description:

**Step 1 — All textual references to the configure endpoint:**
`rg -n 'dtu/configure' crates/ --type rust`

**Step 2 — All X-Admin-Token header attachments:**
`rg -n 'X-Admin-Token' crates/ --type rust`

**Reconciliation:** For each POST call site referencing `dtu/configure`, confirm that
a corresponding `X-Admin-Token` header attachment appears on the same or an adjacent
request-builder line. Dynamic-URL call sites — where the URL is constructed into a
variable (e.g., `client.post(&configure_url)`) rather than typed as a string literal —
will NOT be found by searching for `dtu/configure` on the POST line. These sites MUST
be enumerated by reading the callers of `resolve_configure_url` (or the equivalent
URL-builder function) directly, not by regex. The PR description MUST include the full
output of both greps AND an explicit reconciliation table mapping each POST call site
to its header status, consistent with the sibling enumeration table in §Root Cause above.

## Architecture Mapping

| Component | Module | File | Pure/Effectful |
|-----------|--------|------|----------------|
| `cmd_configure` (primary fix) | prism-dtu-demo-server binary | `src/main.rs` | Effectful (HTTP POST + file read) |
| `write_url_sidecar` extension | prism-dtu-demo-server binary | `src/main.rs` | Effectful (file write) |
| `write_token_sidecar_to_path` (library helper, load-bearing) | prism-dtu-demo-server library | `src/harness.rs` (re-exported via `lib.rs`) | Effectful (file write — atomic tmp+rename, 0600 on Unix) |
| `write_token_sidecar` (binary wrapper) | prism-dtu-demo-server binary | `src/main.rs` | Effectful (thin wrapper; delegates to `write_token_sidecar_to_path`) |
| `DemoHarness::token_map()` (new method) | prism-dtu-demo-server library | `src/harness.rs` | Pure (read-only accessor over bound clone pairs) |
| `MultiInstanceServers::admin_token_map()` | prism-dtu-demo-server library | `src/multi_instance.rs` | Pure (accessor) |
| `start_instances()` amendment | prism-dtu-demo-server library | `src/multi_instance.rs` | Effectful (token extraction before async move) |
| `write_multi_admin_token_sidecar_to_path` (new fn) | prism-dtu-demo-server library | `src/multi_org_cmd.rs` | Effectful (file write) |
| `TOKEN_FILE`, `TOKEN_MULTI_FILE` constants | prism-dtu-demo-server library | `src/lib.rs` | Pure (constants) |
| `defect_demo_configure_admintoken_001.rs` (Red Gate test) | prism-dtu-demo-server tests | `tests/defect_demo_configure_admintoken_001.rs` | Effectful (integration test) |

**Architecture Compliance Rules (from ADR-003 Amendment #5 and existing harness patterns):**
- Token sidecar files MUST use the atomic tmp+rename write pattern (same as URL sidecars per S-DEMO-LAUNCHER-CONSOLIDATION-001; cf. GAP-3 is the cwd-threading note, not the atomic-write requirement)
- Token values MUST be read via `BehavioralClone::admin_token()` — never hardcoded, never reused across process invocations
- The token sidecar format MUST be a flat JSON object `{name: token}` for `start` mode and a nested object `{org_slug: {sensor_id: token}}` for `start-multi` mode — exactly mirroring the URL sidecar shape
- `start_instances()` MUST extract `clone.admin_token().to_string()` from each `BoundInstance` BEFORE it is moved into the detached watcher task — OWNERSHIP: once the clone is moved into `tokio::spawn(async move { drop(clone) })`, it is unreachable from the `start_instances` call site; extraction must happen in the bind loop before the spawn (note: `BoundInstance` is already `Box<dyn BehavioralClone>` so `admin_token()` remains callable on the trait object, but OWNERSHIP prevents access after the move)
- No token values may appear in structured log fields (AD-017 credential-safety rule; demo tokens are ephemeral test values, not production credentials, but the pattern must be consistent)

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Token mismatch — someone manually edits the token sidecar with a wrong value | Clone returns HTTP 401; `cmd_configure` receives non-2xx; prints `"HTTP 401"` + body to stdout and exits with code 1 (the non-2xx status-print-and-exit block of `cmd_configure`) |
| EC-002 | Clone without admin token configured ("dev mode") | NOT APPLICABLE — all `BehavioralClone` implementations generate a random UUID v4 token via `uuid::Uuid::new_v4().to_string()` at construction time (verified in the `admin_token` field initialization in `CrowdstrikeState::default`). There is no dev-mode no-token path. |
| EC-003 | Token sidecar exists but clone name is absent (e.g., `configure threatintel` after `start` that did not include threatintel) | `cmd_configure` calls `resolve_configure_url` before `resolve_configure_token`. In the canonical consistent-sidecar scenario (clone absent from both sidecars), `resolve_configure_url` surfaces the clone-not-found anyhow error first — no E-DEMO-007 code. The `resolve_configure_token` E-DEMO-007 arm — `"configure: E-DEMO-007: admin token for clone 'threatintel' could not be resolved: clone 'threatintel' not found in token sidecar '.prism-dtu-demo-server.admin-tokens.json'"` — is defense-in-depth for skewed-sidecar states where the URL sidecar resolves the clone but the token sidecar does not. This arm is contract-locked in isolation by Tests C/H/I, which exercise `resolve_configure_token` independently. Operator exits code 1. |
| EC-004 | Token sidecar missing (server not running or different cwd) | `cmd_configure` calls `resolve_configure_url` before `resolve_configure_token`. In the canonical consistent-sidecar scenario (no sidecars at all), `resolve_configure_url` surfaces the no-URL-sidecar anyhow error first — no E-DEMO-007 code. The `resolve_configure_token` E-DEMO-007 arm — `"configure: E-DEMO-007: admin token for clone '{clone_name}' could not be resolved: token sidecar not found (start the demo server first with start or start-multi)"` — is defense-in-depth for skewed-sidecar states where the URL sidecar exists but the token sidecar is absent. This arm is contract-locked in isolation by Tests C/H/I, which exercise `resolve_configure_token` independently. Operator exits code 1 with guidance to start the demo server first. |
| EC-005 | Ambiguous bare sensor name in `start-multi` mode (multiple orgs have same sensor) | `cmd_configure` calls `resolve_configure_url` before `resolve_configure_token`. In the canonical scenario (consistent sidecars), `resolve_configure_url` returns a plain anyhow ambiguity error first — no E-DEMO-007 code. The E-DEMO-007 ambiguity arm in `resolve_configure_token` — message template: `"Bare sensor name '{name}' is ambiguous — found in N orgs: ["org-a", "org-b"]. Use full '{org_slug}-{sensor_id}' form."` (org list rendered via Rust `{:?}` on a sorted `Vec<String>`, e.g. `["org-a", "org-b"]`; verified in the multi-match ambiguity arm of `resolve_configure_token` in `multi_org_cmd.rs`) — is defense-in-depth for skewed-sidecar states where URL and token sidecars are inconsistent. This arm is contract-locked by Test D (`test_BC_3_6_001_e_demo_007_ec005_ambiguous_bare_sensor_name`), which exercises `resolve_configure_token` in isolation. |
| EC-006 | Concurrent `start-multi` restarts the demo server; old token sidecar has stale tokens | New token sidecar is written atomically on each start; stale-token 401 from server is surfaced (existing EC-001 non-2xx status-print-and-exit flow); operator must retry |
| EC-007 | TLS-enabled `start --tls` — clone URLs use `https://`; configure must still work | URL resolution unchanged (URL sidecar uses `https://`); token sidecar is independent of TLS; test harness uses `danger_accept_invalid_certs(true)` where needed |
| EC-008 | Invalid characters in `clone_name` argument — CWE-117 log-injection surface (SEC-001, 2026-07-17 one-time accelerated-convergence exception, HEAD 5c9458d6); empty-string vacuous-truth gate (NEW-001, CWE-20, @dac830d1) | `validate_clone_name()` runs as the FIRST line of `cmd_configure`, BEFORE `resolve_configure_url`, any sidecar I/O, token lookup, or tracing. Rejects clone names containing characters outside `[a-zA-Z0-9_-]` with error: `"configure: invalid clone name '<sanitized>': clone names may contain only alphanumerics, '-' and '_'"` where `<sanitized>` replaces each disallowed character with `'?'` via `sanitize_clone_name()`. Exits code 1. This rejection happens BEFORE `resolve_configure_url` is called — it precedes all sidecar resolution, HTTP I/O, and tracing. The validation error is NOT an E-DEMO-NNN code and is intentionally excluded from error-taxonomy registration: it is an argument-validation gate, not a runtime state-resolution failure; AC-003's E-DEMO-007 taxonomy scope (unresolvable-token path) is unaffected. Contract-locked by three unit tests in `crates/prism-dtu-demo-server/src/main.rs` `#[cfg(test)] mod tests`: `test_validate_clone_name_rejects_invalid` (inputs: embedded newline `\n`, null byte `\0`, ANSI escape sequence `\x1b[31m`, forward slash `/`, space ` `); `test_validate_clone_name_accepts_valid` (as-built inputs: plain sensor names `crowdstrike`, `cyberint`, `armis`; org-slug composites `org-a-crowdstrike`, `org-b-cyberint`; underscore+hyphen `sensor_name-v2`; mixed case+digits `Sensor42`); `test_validate_clone_name_rejects_empty` (NEW-001/CWE-20 vacuous-truth regression gate: `"".chars().all(predicate)` is vacuously true in Rust — the explicit `if name.is_empty()` guard runs as FIRST line of `validate_clone_name`, before the charset check; error verbatim: `"configure: clone name must not be empty"`; plain `anyhow::bail!`, intentionally NOT an E-DEMO-NNN code, rejection precedes all URL resolution, sidecar I/O, and tracing; @dac830d1). |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `crates/prism-dtu-demo-server/src/main.rs` (`cmd_configure`, `write_token_sidecar` binary wrapper) | effectful-shell | `cmd_configure`: HTTP POST + file read; `write_token_sidecar`: thin binary wrapper delegating file I/O to `write_token_sidecar_to_path` in `harness.rs` |
| `crates/prism-dtu-demo-server/src/harness.rs` (`write_token_sidecar_to_path`, `DemoHarness::token_map`) | effectful-shell / pure-core | `write_token_sidecar_to_path`: effectful (atomic tmp+rename file write); `token_map()`: pure-core (read-only accessor) |
| `crates/prism-dtu-demo-server/src/multi_instance.rs` (`admin_token_map`) | pure-core | Read-only accessor over in-memory map |
| `crates/prism-dtu-demo-server/src/multi_instance.rs` (`start_instances` amendment) | effectful-shell | Spawns async tasks, binds ports |
| `crates/prism-dtu-demo-server/src/multi_org_cmd.rs` (`write_multi_admin_token_sidecar_to_path`, `resolve_configure_token`) | effectful-shell | File write / file read |
| `crates/prism-dtu-demo-server/src/lib.rs` (`TOKEN_FILE`, `TOKEN_MULTI_FILE`) | pure-core | Constants only |

## Token Budget Estimate

| Artifact | Estimated tokens |
|----------|-----------------|
| This story spec | ~3,500 |
| `crates/prism-dtu-demo-server/src/main.rs` | ~8,000 |
| `crates/prism-dtu-demo-server/src/multi_instance.rs` | ~3,500 |
| `crates/prism-dtu-demo-server/src/multi_org_cmd.rs` | ~6,000 |
| `crates/prism-dtu-demo-server/src/harness.rs` | ~5,000 |
| `crates/prism-dtu-demo-server/src/lib.rs` | ~500 |
| BC files (2 BCs: BC-3.6.001, BC-2.06.017) | ~3,000 |
| Error taxonomy (§DEMO section) | ~1,500 |
| Test file context (`ac_3_configure_endpoint.rs`, `td_wv0_07_configure_requires_admin_token.rs`) | ~2,000 |
| Tool outputs (cargo nextest, rg sibling sweep) | ~1,000 |
| **Total estimated** | **~34,000 tokens** |

This is well within the 20-30% context window guideline for a single story. No split required.

## Tasks

- [ ] T-01: Add `TOKEN_FILE` and `TOKEN_MULTI_FILE` `pub const` declarations to `src/lib.rs`
- [ ] T-02: In `src/multi_instance.rs: start_instances()`, capture `clone.admin_token().to_string()` from each `BoundInstance` BEFORE the `tokio::spawn(async move { drop(clone); })` block; build `token_map: HashMap<String, String>`; add `token_map` field to `MultiInstanceServers`; add `pub fn admin_token_map(&self) -> &HashMap<String, String>` accessor
- [ ] T-03: Add `write_token_sidecar()` helper in `src/main.rs` (mirrors `write_url_sidecar()`, writes `TOKEN_FILE` atomically from `harness.pairs` `ClonePair::admin_token()` data)
- [ ] T-04: Call `write_token_sidecar(&harness)?` in `cmd_start()` immediately after `write_url_sidecar(&harness)?`
- [ ] T-05: Add `write_multi_admin_token_sidecar_to_path()` in `src/multi_org_cmd.rs` (mirrors `write_multi_url_sidecar_to_path()`; reads from `servers.admin_token_map()`)
- [ ] T-06: Call `write_multi_admin_token_sidecar_to_path(&servers, &cfg, Path::new(TOKEN_MULTI_FILE))` in `cmd_start_multi()` immediately after `write_multi_url_sidecar(&servers, &cfg)?`
- [ ] T-07: Add `resolve_configure_token(clone_name, flat_token_path, nested_token_path) -> anyhow::Result<String>` in `src/multi_org_cmd.rs` (same lookup logic as `resolve_configure_url` but over the token sidecar; returns E-DEMO-007 messages on miss)
- [ ] T-08: In `cmd_configure()` in `src/main.rs`, after resolving `configure_url`, call `resolve_configure_token()` and attach `X-Admin-Token: <token>` header to the POST request
- [ ] T-09: Remove token sidecar files on shutdown: add `let _ = std::fs::remove_file(TOKEN_FILE)` to `wait_for_shutdown_signal()` alongside `URL_FILE` removal; add `let _ = std::fs::remove_file(TOKEN_MULTI_FILE)` to `wait_for_shutdown_signal_multi()` alongside `URL_MULTI_FILE` removal
- [ ] T-10: Write RED Gate test in `tests/defect_demo_configure_admintoken_001.rs` (starts harness, POSTs without header, asserts 401 before fix; asserts 200 after fix; tests E-DEMO-007 sidecar-missing path)
- [ ] T-11: Register `E-DEMO-007` in `.factory/specs/prd-supplements/error-taxonomy.md` under `## DEMO: Demo-Server Errors` per POL-24 (exact message template verbatim)
- [ ] T-12: Execute the two-step TD-VSDD-060 sibling sweep from AC-004: (a) `rg -n 'dtu/configure' crates/ --type rust`; (b) `rg -n 'X-Admin-Token' crates/ --type rust`; manually reconcile each POST call site — dynamic-URL sites (e.g., `client.post(&configure_url)`) must be traced by reading callers of `resolve_configure_url` directly rather than by regex; include both grep outputs plus a reconciliation table in the PR description

## Previous Story Intelligence

S-DEMO-LAUNCHER-CONSOLIDATION-001 (merged) introduced `start-multi`, `resolve_configure_url`,
and the nested URL sidecar pattern. The admin token sidecar follows exactly the same
architectural pattern: flat for `start`, nested for `start-multi`, atomic write, cleanup on
shutdown. The `write_multi_url_sidecar_to_path` function is the direct template for
`write_multi_admin_token_sidecar_to_path`.

The `MultiInstanceServers` struct was last modified by the D-1075-API-GAP-001 amendment
(added `socket_map`, `shutdown_tx`, `task_handles`). The `admin_token_map` field follows
the same ownership pattern as `socket_map`: populated at bind time, never mutated after.

The test pattern from `ac_3_configure_endpoint.rs` is the gold standard for how tests
call `/dtu/configure`: `cy_pair.admin_token().to_string()` → `.header("X-Admin-Token", &token)`.
The defect fix replicates this pattern for the CLI code path by reading from the token sidecar.

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Atomic write (tmp+rename) for all sidecar files | S-DEMO-LAUNCHER-CONSOLIDATION-001 (atomic write pattern for sidecars; note: GAP-3 in that story is the cwd-threading note for demo-run.sh, not the atomic-write specification) | Token sidecars must use the same pattern; partial-write reads from demo-run.sh must be impossible |
| `BehavioralClone::admin_token()` is the ONLY source of truth for token values | ADR-003 Amendment #5 | Never hardcode, derive, or guess token values; always call `clone.admin_token()` |
| reqwest clients must set `.timeout(Duration::from_secs(30))` | CLAUDE.md §Conventions | `cmd_configure`'s existing client already sets `timeout(10s)` — maintain this; do not reduce below 10s. Exception rationale: `prism-dtu-demo-server` is test/demo infrastructure (feature-gated `#[cfg(any(test, feature = "dtu"))]` in `lib.rs`), not a production client; the CLAUDE.md 30s mandate applies to production crates only. 10s is the ratified crate-local value for this demo server. Do not treat this as precedent for production crates. |
| reqwest clients must use `default-features = false, features = ["rustls-tls"]` | ADR-050 D1/D2 | The `prism-dtu-demo-server` `Cargo.toml` already declares `rustls-tls`; do not add a new reqwest dependency entry |
| No `println!` in production code | CLAUDE.md §Conventions | `cmd_configure` uses `println!` for HTTP status display — this is the CLI output formatter, which is the ratified exception for CLI formatting helpers |
| Token values MUST NOT appear in structured log fields | AD-017 (CLAUDE.md) | `tracing::debug!` calls showing token-related activity must use a placeholder like `token_present=true`, not the token value |

## Library & Framework Requirements

| Library | Version | Source of truth |
|---------|---------|----------------|
| `serde_json` | `1` (workspace) | Workspace-level dep in root `Cargo.toml` — used for token sidecar serialization |
| `reqwest` | `0.12` (per-crate), `rustls-tls`, `default-features = false` | Per-crate dep in `crates/prism-dtu-demo-server/Cargo.toml`; existing `cmd_configure` client — no new dep entry needed |
| `uuid` | `1` (per-crate) | Per-crate dep in `crates/prism-dtu-demo-server/Cargo.toml`; admin tokens are UUID v4 strings; no new usage in `cmd_configure` (token is a `&str` read from sidecar) |
| `tokio` | `1` (per-crate) | Per-crate dep in `crates/prism-dtu-demo-server/Cargo.toml`; async runtime — no change |

## File Structure Requirements

| Action | File | Change |
|--------|------|--------|
| MODIFY | `crates/prism-dtu-demo-server/src/lib.rs` | Add `pub const TOKEN_FILE: &str` and `pub const TOKEN_MULTI_FILE: &str` declarations; add `pub use harness::write_token_sidecar_to_path` re-export |
| MODIFY | `crates/prism-dtu-demo-server/src/harness.rs` | Add `DemoHarness::token_map()` accessor; add `write_token_sidecar_to_path()` library helper (re-exported from `lib.rs`) |
| MODIFY | `crates/prism-dtu-demo-server/src/multi_instance.rs` | Add `token_map` field to `MultiInstanceServers`; add `admin_token_map()` accessor; extract tokens in `start_instances()` before watcher spawn |
| MODIFY | `crates/prism-dtu-demo-server/src/main.rs` | Add `write_token_sidecar()` helper; call it in `cmd_start()`; call `write_multi_admin_token_sidecar_to_path()` in `cmd_start_multi()`; read token and include header in `cmd_configure()`; clean up token sidecars in shutdown handlers |
| MODIFY | `crates/prism-dtu-demo-server/src/multi_org_cmd.rs` | Add `write_multi_admin_token_sidecar_to_path()` function (mirror of `write_multi_url_sidecar_to_path`); add `resolve_configure_token()` function (mirror of `resolve_configure_url` for the token sidecar) |
| CREATE | `crates/prism-dtu-demo-server/tests/defect_demo_configure_admintoken_001.rs` | RED Gate test file |
| MODIFY | `.factory/specs/prd-supplements/error-taxonomy.md` | Add E-DEMO-007 row under `## DEMO: Demo-Server Errors` section |

## Forbidden Dependencies

`prism-dtu-demo-server` is test/demo infrastructure only (gated `#[cfg(any(test, feature = "dtu"))]`
in `lib.rs`). No new production crate dependencies are required. If the implementer is tempted to
add a new dep to fix this story, that is a signal the approach is wrong — the fix uses only
already-present workspace dependencies (`serde_json`, `reqwest`, `tokio`, `std`).

## §Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| v0.21 | 2026-07-18 | product-owner SPEC-SYNC F-ADMTOK-PR21-OBS-001 + NEW-001 (CWE-20, @dac830d1) | F-ADMTOK-PR21-OBS-001 (OBS): EC-008 `test_validate_clone_name_accepts_valid` example enumeration corrected to as-built literals. Old (v0.20): plain sensors `crowdstrike` / `armis` (missing `cyberint`); composite `acme-crowdstrike` (actual: `org-a-crowdstrike` / `org-b-cyberint`); underscore `my_sensor` / hyphen `my-sensor` (actual: `sensor_name-v2`; `Sensor42` absent). New (v0.21 as-built): plain sensor names `crowdstrike`, `cyberint`, `armis`; org-slug composites `org-a-crowdstrike`, `org-b-cyberint`; underscore+hyphen `sensor_name-v2`; mixed case+digits `Sensor42`. Code was correct; prose was stale. NEW-001 (CWE-20, @dac830d1): empty-string vacuous-truth gate codified in EC-008. `validate_clone_name` rejects `""` as FIRST line — `"".chars().all(predicate)` is vacuously true in Rust; without the explicit `if name.is_empty()` guard an empty name passes the charset check and reaches `resolve_configure_url`. Error message verbatim: `"configure: clone name must not be empty"`. Plain `anyhow::bail!` — intentionally NOT an E-DEMO-NNN code (argument-validation gate, not runtime state-resolution failure; same rationale as charset rejection). Rejection precedes all URL resolution, sidecar I/O, and tracing. Load-bearing test: `test_validate_clone_name_rejects_empty`. EC-008 now covers three gated behaviors: charset rejection (CWE-117, SEC-001), accepts-valid smoke tests (now with as-built literals), empty-string rejection (CWE-20, NEW-001). EC-008 description cell extended with `; empty-string vacuous-truth gate (NEW-001, CWE-20, @dac830d1)`. Test count in EC-008 Expected Behavior cell: "two unit tests" → "three unit tests". POL-29 in-file sweep: searched body for `v0.20` as current-version pin — zero hits outside the frontmatter (now bumped to v0.21) and the historical §Changelog v0.20 row (preserved per append-only ID and slug policy). |
| v0.20 | 2026-07-17 | product-owner SPEC-SYNC SEC fix-burst (SEC-001 CWE-117 + SEC-002, HEAD 5c9458d6, human-approved 2026-07-17 one-time accelerated-convergence exception) | SEC-001 (CWE-117): EC-008 added — `validate_clone_name()` input-validation gate runs as FIRST line of `cmd_configure`, BEFORE `resolve_configure_url` / sidecar I/O / tracing, rejecting clone names with characters outside `[a-zA-Z0-9_-]`. Error form: `"configure: invalid clone name '<sanitized>': clone names may contain only alphanumerics, '-' and '_'"` (sanitized via `sanitize_clone_name()` replacing disallowed chars with `'?'`). Exits code 1. The validation error is NOT an E-DEMO-NNN code — excluded from error-taxonomy registration; it is an argument-validation gate, not a runtime state-resolution failure; AC-003's E-DEMO-007 taxonomy scope is unaffected. Contract-locked by two unit tests in `src/main.rs` `#[cfg(test)] mod tests`: `test_validate_clone_name_rejects_invalid` (newline / null byte / ANSI escape / slash / space) and `test_validate_clone_name_accepts_valid` (bare sensors, org-slug composites, underscores, hyphens). SEC-002 (advisory): rationale comment added above the 10s reqwest timeout confirming the ratified crate-local value for prism-dtu-demo-server (test/demo infrastructure; CLAUDE.md 30s mandate applies to production clients only; already documented in §Architecture Compliance Rules). POL-29 sweep: searched for `cmd_configure` error-surface sites across full story body; all identified: §Origin: bibliographic "non-zero exit" note (no enumeration claim, no change); AC-003: E-DEMO-007 isolation-contract for token-resolution failures — untouched per task; EC-001: non-2xx status-print-and-exit for server-origin 401; EC-003: url-resolution-first ordering caveat (clone absent from both sidecars); EC-004: url-resolution-first ordering caveat (no sidecars at all); EC-005: url-ambiguity-first ordering caveat; EC-006: stale-token 401 surfaced via EC-001 flow; EC-007: TLS case (no error path); EC-008 (NEW): validation-gate rejection. No site claims to enumerate ALL cmd_configure failure modes — EC-008 with explicit `NOT an E-DEMO code` note is sufficient to prevent future taxonomy-registration findings. |
| v0.19 | 2026-07-17 | product-owner FIX-BURST F-ADMTOK-PR17-LOW-001 | F-ADMTOK-PR17-LOW-001 (LOW): EC-006 Expected Behavior parenthetical corrected. Old: "(existing AC-003 flow)"; New: "(existing EC-001 non-2xx status-print-and-exit flow)". AC-003 is exclusively the PRE-POST unresolvable-token path (E-DEMO-007; token lookup fails before any POST is issued); in EC-006 the token resolves (stale-but-present) so a POST is issued and the server returns 401 — that 401 is surfaced by the EC-001 non-2xx status-print-and-exit block (main.rs:700-705), not by AC-003's E-DEMO-007 exit. The old parenthetical incorrectly implied AC-003's pre-POST error path handles the server-origin 401. POL-29 sweep: grep for "AC-003 flow", "401.*AC-003", "AC-003.*401" across entire story body yielded three AC-003 occurrences — (1) line 247: `### AC-003` section heading (isolation-contract definition, STAY); (2) line 336: EC-006 target (FIXED); (3) line 442: v0.18 changelog prose "AC-003 `resolve_configure_token`-in-isolation contract...left unmodified" (bibliographic isolation-contract reference, STAY). Zero additional server-origin-401-to-AC-003 attributions found. |
| v0.18 | 2026-07-17 | product-owner FIX-BURST F-ADMTOK-PR14-MED-001 + F-ADMTOK-PR14-LOW-001 | F-ADMTOK-PR14-MED-001 (MED): EC-003 and EC-004 Expected Behavior cells corrected to reflect resolution-ordering precedence — mirrors EC-005 v0.13 precedent (F-ADMTOK-P12-LOW-003). As-built, `cmd_configure` calls `resolve_configure_url` (main.rs:647) before `resolve_configure_token` (:669); URL and token sidecars carry identical key sets in the canonical case, so in EC-003 (clone absent from both sidecars) `resolve_configure_url` surfaces the clone-not-found anyhow error first (no E-DEMO-007 code), and in EC-004 (no sidecars at all) `resolve_configure_url` surfaces the no-URL-sidecar anyhow error first (no E-DEMO-007 code). Old EC-003 cell: "E-DEMO-007: 'clone threatintel not found in token sidecar…' — exits code 1"; Old EC-004 cell: "E-DEMO-007: 'token sidecar not found…' — exits code 1". Both replaced with three-part ordering-caveat wording (URL-resolution-first note, defense-in-depth classification of the E-DEMO-007 arm for skewed-sidecar states, isolation-contract-lock via Tests C/H/I) while preserving functional outcomes (exit 1; EC-004 retains "start the demo server first" guidance). AC-003 `resolve_configure_token`-in-isolation contract (lines 246–268, including taxonomy row) left unmodified — the ordering caveat applies only to end-to-end/operator-visible EC claims, not the isolation function contract. F-ADMTOK-PR14-LOW-001 (LOW): `crates_touched` frontmatter extended — `prism-bin` added with inline annotation noting it is a doc-comment-only test-helper cleanup (`helpers/mod.rs` stale-reference removal, TD-VSDD-060 sibling-sweep byproduct; no functional surface). POL-29 sweep: grep for E-DEMO-007 yielded 12 occurrences across 9 distinct lines; only lines 332–333 (EC-003/EC-004 Expected Behavior cells) asserted E-DEMO-007 as end-to-end operator-visible outcomes; all other occurrences are isolation-contract or bibliographic references — no further changes needed. |
| v0.17 | 2026-07-17 | product-owner FIX-BURST F-ADMTOK-PR11-LOW-001 | F-ADMTOK-PR11-LOW-001 (LOW): AC-003 §Error Taxonomy Addition mirror-table header aligned to canonical form. Old: `\| Code \| State \| Domain \| Message template \| Retryable \| Description \|`; New: `\| Code \| Severity \| Category \| Message Format \| Retryable \| Description \|`. Row values are byte-correct and unchanged (per POL-24). POL-29 sibling sweep: `\| State \|` found only at line 266 (the fixed header); `\| Domain \|` found only at line 266; `Message template` as column header found only at line 266 — zero additional taxonomy-mirroring occurrences. Prose occurrence of "broken" at line 94 is English "not working" sense, not taxonomy Severity vocabulary — no change. `modified:` was already `2026-07-17`; `input-hash` updated `6982f3b` → `0b092d4` (hook-computed). |
| v0.16 | 2026-07-17 | story-writer FIX-BURST F-ADMTOK-PR4-HIGH-001 | F-ADMTOK-PR4-HIGH-001 (HIGH): subsystems anchor corrected SS-22 → SS-01. SS-22 is canonically "Process Lifecycle" (not "Binary Entrypoint") and is scoped exclusively to prism-bin boot orchestration (ADR-022 §B 11-step boot, startup failure exit-code map, traffic gate signal handlers) per ARCH-INDEX v2.193 line 175. SS-22 does not own prism-dtu-demo-server. SS-01 (Sensor Adapters) is the correct anchor: ARCH-INDEX v2.193 Subsystem Registry line 154 explicitly lists prism-dtu-demo-server in SS-01's crate column. Old→new: `subsystems: [SS-22]` → `subsystems: [SS-01]`; justification comment rewritten. POL-29 sibling sweep: grep confirmed all three SS-22/"Binary Entrypoint" occurrences were confined to frontmatter lines 32–38; zero body occurrences. input-hash updated 5bb76c5 → 6982f3b (hook-computed current hash). |
| v0.15 | 2026-07-17 | product-owner FIX-BURST-14 SPEC (F-ADMTOK-P15-MED-001, F-ADMTOK-P15-OBS-002) | F-ADMTOK-P15-MED-001 (MED, spec facet): AC-002 did not specify the `_global` enrichment-token section of TOKEN_MULTI_FILE. BC-2.06.017 v1.12 Postcondition 1 (sidecar format: nested JSON mirroring URL_MULTI_FILE) makes the `_global` key contract-mandated because URL_MULTI_FILE carries `_global` per ENRICH-3. Added bullet to AC-002 "Token sidecar write requirements": TOKEN_MULTI_FILE MUST include the reserved `_global` key carrying admin tokens for all enabled enrichment DTU clones (KNOWN_ENRICHMENT_CLONES), mirroring the URL_MULTI_FILE `_global` pattern (ENRICH-3); `write_multi_admin_token_sidecar_to_path` MUST fail-loud (propagate an error) when an enabled enrichment clone's token is absent from admin_token_map; contract-locked by Test K (enrichment `_global` token lock). F-ADMTOK-P15-OBS-002 (OBS, EC-007 namespace collision): AC-002 item 2 cited "(EC-007 recovery form)" without qualification — within this story's §Edge Cases namespace, EC-007 is the TLS-enabled `start --tls` case, not the operator recovery form in `resolve_configure_url`. Qualified as "(EC-007 recovery form documented in `resolve_configure_url` — distinct from this story's §Edge Cases EC-007)". POL-25 sweep: two other EC-007 references in §Narrative and §Origin are already qualified with "in `resolve_configure_url`" and "documented in `resolve_configure_url`" respectively — no further changes needed. |
| v0.14 | 2026-07-17 | product-owner FIX-BURST-12 SPEC (F-ADMTOK-P13-LOW-002, F-ADMTOK-P13-LOW-003) | F-ADMTOK-P13-LOW-002 (LOW, POL-22 Phase C + POL-25 sweep): corrected story structural tables to reflect as-built file layout. (i) AC-002 sidecar write bullet reworded: `cmd_start()` calls `write_token_sidecar` immediately after `write_url_sidecar`; the load-bearing helper `write_token_sidecar_to_path` lives in `src/harness.rs` (not `src/main.rs`), re-exported via `lib.rs`, with `write_token_sidecar` in `main.rs` as a thin binary wrapper delegating to it (confirmed: harness.rs `token_map()` at `DemoHarness::token_map`, `write_token_sidecar_to_path` at line 361; main.rs wrapper at `write_token_sidecar` delegates to `prism_dtu_demo_server::write_token_sidecar_to_path`; lib.rs re-export confirmed `pub use harness::{write_token_sidecar_to_path, ...}`). (ii) Added `MODIFY crates/prism-dtu-demo-server/src/harness.rs` row to §File Structure Requirements (`DemoHarness::token_map()` accessor + `write_token_sidecar_to_path` helper + re-export via `lib.rs`); updated `lib.rs` row to include re-export. (iii) §Architecture Mapping: split the incorrect `write_token_sidecar (new helper) \| library \| src/main.rs` row into `write_token_sidecar_to_path` (library helper, harness.rs), `write_token_sidecar` (binary wrapper, main.rs), and `DemoHarness::token_map()` (pure accessor, harness.rs). §Purity Classification: updated main.rs row to note binary-wrapper nature of `write_token_sidecar`; added harness.rs row for `write_token_sidecar_to_path` (effectful-shell) and `token_map()` (pure-core). F-ADMTOK-P13-LOW-003 (LOW, SWEEP-MIRROR convention codification): AC-004 ¶1 reworded from "naming each enumerated site" to codify the v0.10 ratified SWEEP-MIRROR convention as built — code mirrors carry byte-identical command forms + stable counts (447/131/6/8) + condensed site-group table; the story §Root Cause TD-VSDD-060 table (~24 live rows / 146 POSTs) is the source of truth; `cmd_configure` confirmed ONLY defect site. POL-29 sweep: "naming each enumerated site" phrase found only in the fixed AC-004 ¶1 location. |
| v0.13 | 2026-07-17 | product-owner FIX-BURST-11 SPEC (F-ADMTOK-P12-MED-001, F-ADMTOK-P12-LOW-001, F-ADMTOK-P12-LOW-002, F-ADMTOK-P12-LOW-003) | F-ADMTOK-P12-MED-001 (MED, POL-21 phantom §-anchor + POL-4 GAP-3 semantic mismatch): stripped `§Sidecar-availability guarantee (GAP-3 from S-DEMO-LAUNCHER-CONSOLIDATION-001)` from BC-2.06.017 Relevant Clause cell — "Sidecar-availability" is not a heading anywhere in `.factory/`; the launcher story heading is `## Changelog` (no §-sigil), and GAP-3 in that story's Changelog v2.1 is a cwd-path-threading note for demo-run.sh, not an atomic-write guarantee. Replaced with: `the atomic tmp+rename sidecar write pattern (established for URL sidecars; cf. GAP-3 sidecar-poll note, S-DEMO-LAUNCHER-CONSOLIDATION-001 Changelog v2.1)`. BC-2.06.017 version pin updated v1.11 → v1.12 (BC updated in parallel). Code comments in main.rs/multi_org_cmd.rs may retain the informal phrase (accepted-informal, out of spec scope). F-ADMTOK-P12-LOW-001 (LOW): aligned three footnote command forms byte-for-byte with code mirrors — dropped `-n` flag from commands (1)(2)(3); command (3) gained trailing `wc -l` count suffix. IDENTICAL claim is now true for all four commands; choice: option (a) — byte-identical alignment; counts unchanged (447, 131, 6, 8). F-ADMTOK-P12-LOW-002 (LOW, POL-22 Phase C, POL-25 sweep): renamed helper function name `deny_unknown_fields` → `assert_configure_strict` (deny-unknown-fields test) in two live table cells (Site cell row 137, Harness::inject_failure() row 138) and in the v0.7 changelog narrative (POL-25: all three occurrences corrected). F-ADMTOK-P12-LOW-003 (LOW): EC-005 Expected Behavior corrected — `resolve_configure_url` returns a plain anyhow ambiguity error first (no E-DEMO-007 code) in the canonical scenario; the E-DEMO-007 arm of `resolve_configure_token` is defense-in-depth for skewed-sidecar states, contract-locked by Test D (`test_BC_3_6_001_e_demo_007_ec005_ambiguous_bare_sensor_name`). E-DEMO-007 message template retained byte-verbatim per POL-24. |
| v0.12 | 2026-07-17 | product-owner FIX-BURST-10 SPEC (F-ADMTOK-P11-MED-001 spec side) | 8 FidelityCheck-based indirect POST callers of `/dtu/configure` enumerated: 2 new rows added to §Root Cause TD-VSDD-060 table — (a) 5 per-DTU `fidelity_validator.rs` files (nvd, claroty, armis, threatintel, cyberint) and (b) 3 `harness_tests.rs` AC-002 FidelityValidator tests (armis, claroty, cyberint); all correct-token class (token in `FidelityCheck.headers` field, ADR-003 Amendment #3). Footnote: 4th sweep command added (`rg 'endpoint.*"/dtu/configure"' crates/ --type rust \| grep -v SWEEP-MIRROR \| wc -l` → 8). Grand total updated 138 → 146 (arithmetic: 138 + 8 = 146; correct-token class: 103 + 8 = 111). Summary sentence updated to 146 total / 111 correct-token. In-process row exclusion footnote updated 138 → 146. These 8 sites are invisible to the `.post(.*dtu/configure)` grep because `FidelityValidator::run` issues the HTTP POST via `client.request()` internally. |
| v0.11 | 2026-07-16 | product-owner MICRO-FIX SPEC (mirror-mismatch) | Command (1) count corrected 451 → 447 (implementer commit 0aa0c6ed tagged 8 SWEEP-MIRROR lines total, not 4 as assumed; 455 raw − 8 = 447). Commands (2) and (3) updated to add `\| grep -v SWEEP-MIRROR` for form-parity with code mirrors (counts unchanged: 131 / 6). All three artifacts (story footnote, main.rs mirror, defect_demo mirror) now quote identical command forms. |
| v0.10 | 2026-07-16 | product-owner FIX-BURST-9 SPEC (F-ADMTOK-P10-MED-001 + F-ADMTOK-P10-LOW-001) | F-ADMTOK-P10-MED-001 (MED): per-class tally reclassified. The 6 per-DTU `harness_tests.rs` files contain 11 migrated BC-3.5.001/TD-WV0-07 negative tests that were miscounted as "positive configure paths" (6 no-token: crowdstrike:2157, cyberint:1218, pagerduty:790, armis:1712, claroty:1692, jira:1057; 5 wrong-token: crowdstrike:2181, claroty:1714, cyberint:1248, armis:1734, jira:1079; verified by reading each file). Correct split: 103 correct-token / 17 no-token / 15 wrong-token / 1 prod-CLI-FIXED / 1 harness-lib / 1 N/A = 138. "Per-DTU-crate harness_tests.rs" row split into positive (27 calls, 4 crates) + negative (11 calls, 6 crates) sub-rows. Summary sentence and footnote per-class tally updated. F-ADMTOK-P10-LOW-001 (LOW/structural): SWEEP-MIRROR convention adopted to keep command (1) stable. The implementer's F-ADMTOK-P9-LOW-002 commit added mirror-comment blocks to `main.rs` and `defect_demo_configure_admintoken_001.rs` that contain `dtu/configure` text, inflating the naive grep count from 451 to 455. Convention: these mirror lines are tagged `// SWEEP-MIRROR`; command (1) updated to `rg -n 'dtu/configure' crates/ --type rust \| grep -v SWEEP-MIRROR \| wc -l` → **447** (stable; 455 raw − 8 tagged mirror lines). Commands (2) and (3) also updated to add `\| grep -v SWEEP-MIRROR` for form-parity with code mirrors; counts unchanged at 131 / 6. |
| v0.9 | 2026-07-16 | product-owner FIX-BURST-8 SPEC (F-ADMTOK-P9-MED-001 + F-ADMTOK-P9-LOW-001) | F-ADMTOK-P9-MED-001 (MED): v0.8 sweep tally was not reproducible — claimed "116 same-line + 7 dynamic = 123" using a flawed filter that subtracted manually-estimated comment false-positives; the precise command `rg -n '\.post\(.*dtu/configure' crates/ --type rust \| wc -l` returns 131 directly, giving 131+7=138. Footnote rewritten with verbatim commands and corrected per-class tally (114 correct-token, 11 no-token, 10 wrong-token, 1 prod CLI FIXED, 1 harness lib, 1 N/A → sum 138). Summary sentence count updated from 123 to 138; class counts updated (99→114 correct-token). F-ADMTOK-P9-LOW-001 (LOW): three phantom/misclassified rows fixed. (a) `bc_2_06_019_scenario_progression.rs` row removed — file has zero HTTP POST calls to /dtu/configure; it uses `admin_token()` only for Bearer auth on GET data-plane endpoints, confirmed by `rg -n 'dtu/configure'` showing no hits for this file. (b) `bc_2_01_017_access_token_auth.rs {cyberint,nvd}` citation removed from ac_* HTTP-POST group — cyberint line 54 is `clone.configure(serde_json::json!({...})).await` (in-process trait method, no HTTP emitted); nvd variant does not exist; count corrected 22→21. (c) `org_tagging.rs (jira)` removed from misc HTTP-POST group — file has no configure calls of any kind. Added new "In-process `clone.configure()` method call — HTTP auth not applicable" classification row anchoring these sites correctly. |
| v0.8 | 2026-07-16 | product-owner FIX-BURST-7 SPEC (F-ADMTOK-P8-LOW-001) | Full AC-004 sibling sweep reconciliation. Ran complete `rg -n 'dtu/configure' crates/ --type rust` (451 total hits, 123 POST client calls). Added 16 new rows to §Root Cause TD-VSDD-060 table covering: `Harness::inject_failure()` production method (harness.rs:~287); builder.rs hung-socket timeout test (N/A); td_wv0_07_* no-token negative tests ×8 clones; td_wv0_07_* wrong-token negative tests ×8 clones; td_wv0_04_* deny-unknown tests ×6 clones (12 POSTs); per-DTU harness_tests.rs ×6 crates (38 POSTs); ac_* rate-limit/error/auth tests ×5 crates (22 POSTs); sec_p3_003 correct/wrong-token tests (claroty); edge_cases.rs (claroty, cyberint); fidelity.rs positive tests and 401-validation negative tests (jira, pagerduty); misc tests (multi_tenant, reset_state_invariants, dtu_reset_mount, ac_tests slack, etc.); defect_demo Test A (contract lock) and Test B (post-fix). Added footnote with sweep stats and per-class tally for mechanical future verification. Updated summary sentence to reference all 123 call sites and final class counts. |
| v0.7 | 2026-07-16 | product-owner FIX-BURST-6 SPEC (F-ADMTOK-P7-LOW-001 + F-ADMTOK-P7-OBS-001) | F-ADMTOK-P7-LOW-001: §Root Cause TD-VSDD-060 sibling table was missing two harness-crate call sites — added `configure_failure` helper in `bc_3_6_001_ops_clone_failure_modes.rs` (the BC-3.6.001 test itself; lowercase `x-admin-token` via `Harness::admin_token_for()`) and `assert_configure_strict` helper (deny-unknown-fields test) in `review_2026_06_10_deny_unknown.rs` (same token-source pattern); both correctly authenticated. Updated summary sentence to note the harness-token variant (`Harness::admin_token_for()` vs `clone.admin_token()`). F-ADMTOK-P7-OBS-001: AC-001 "The test MUST fail before the fix lands" sentence was internally contradictory — a POST-without-header / assert-401 test passes before AND after (server 401 gate predates the branch). Replaced with contract-lock explanation: steps 1-4 are a defect-observability lock (always passes); Red-Gate obligation lives in binary E2E exit-0 assertion and sidecar-existence tests. |
| v0.6 | 2026-07-16 | product-owner FIX-BURST-5 SPEC (F-ADMTOK-P6-LOW-003) | TD-VSDD-091 full sibling sweep — 3 volatile line pins replaced with behavioral anchors. (1) AC-002 bullet: "line ~354-365 of multi_instance.rs" → "in the watcher-spawn loop of start_instances in multi_instance.rs". (2) EC-001: "cmd_configure lines ~583-585" → "the non-2xx status-print-and-exit block of cmd_configure". (3) EC-002: "CrowdstrikeState::default() line ~415" → "the admin_token field initialization in CrowdstrikeState::default". Excepted (unchanged): §Root Cause TD-VSDD-060 enumeration table "Line range (approx.)" column — justified citation table. |
| v0.5 | 2026-07-16 | product-owner FIX-BURST-4 SPEC (F-ADMTOK-P4-MED-001) | POL-22 Phase A violation: §Root Cause block-quoted "ADR-003 Amendment #5, item 5" with fabricated text that does not exist in the ADR. Replaced with two verbatim ADR anchors: (1) Amendment #5 §Decision paragraph ("POST /dtu/configure on every DTU clone MUST require a valid X-Admin-Token header…") establishing the server-side requirement; (2) Amendment #5 §Implementation item 4 ("All 12 existing td_wv0_04 configure tests…and all other integration tests calling /dtu/configure: updated to include .header(…)") showing item 4 scoped to integration tests only — cmd_configure is a CLI path, not a test, so it was not covered. Narrative conclusion (cmd_configure was the un-updated caller) preserved. POL-25 sweep: one "item 5" citation in the file (line 100) — corrected. |
| v0.4 | 2026-07-16 | product-owner FIX-BURST-2 SPEC (F-ADMTOK-P2-LOW-001) | TD-VSDD-091: volatile line pin "lines 1068-1076" in EC-005 replaced with behavioral anchor "the multi-match ambiguity arm of `resolve_configure_token` in `multi_org_cmd.rs`" — line numbers decay on subsequent diffs; function + arm name is stable. |
| v0.3 | 2026-07-16 | product-owner FIX-BURST-1 SPEC | F-ADMTOK-P1-MED-001: BC-2.06.017 version pin v1.10 → v1.11 (Relevant Clause cell updated to note v1.11 formally enumerates admin_token_map and TOKEN_MULTI_FILE). F-ADMTOK-P1-LOW-001: EC-005 message template reconciled — org list rendering changed from unquoted `[org-a, org-b]` to `{:?}`-quoted `["org-a", "org-b"]` to match `resolve_configure_token` code (multi_org_cmd.rs lines 1068-1076) and be consistent with sibling `resolve_configure_url` ambiguity arm. |
| v0.2 | 2026-07-16 | remove-uncertainty pass (D-1110) | U1 (HIGH): AC-004 + T-12 sibling-sweep rewrite — replaced broken BRE-pipe regex with two-step `rg -n 'dtu/configure'` + `rg -n 'X-Admin-Token'` reconciliation; explicit note that dynamic-URL sites must be traced via callers of `resolve_configure_url`. U2 (MED): reqwest-timeout enforcement cell now includes explicit exception rationale (DTU is test/demo infra, not production; 10s is the ratified crate-local value). U3 (LOW): dep provenance labels corrected — `reqwest`, `uuid`, `tokio` are per-crate in `prism-dtu-demo-server/Cargo.toml`; only `serde_json` is workspace-level. U4 (LOW): `start_instances()` Architecture Compliance Rules bullet rationale corrected from "concrete type is erased" to OWNERSHIP — the clone is moved into the watcher task and becomes unreachable from the call site. Template conformance: added `## Narrative`, `## Purity Classification`, renamed `Library and Framework Requirements` → `Library & Framework Requirements`. |
| v0.1 | 2026-07-16 | story-writer | Initial story decomposition for DRIFT-DEMO-CONFIGURE-ADMINTOKEN-001. |

## §References

| Reference | Type | Purpose |
|-----------|------|---------|
| `crates/prism-dtu-demo-server/src/main.rs` `cmd_configure()` | Source | Primary defect site — POST without X-Admin-Token |
| `crates/prism-dtu-demo-server/src/main.rs` `write_url_sidecar()` | Source | Template for write_token_sidecar |
| `crates/prism-dtu-demo-server/src/multi_instance.rs` `start_instances()` | Source | Site where token must be extracted before clone move |
| `crates/prism-dtu-demo-server/src/multi_org_cmd.rs` `write_multi_url_sidecar_to_path()` | Source | Template for write_multi_admin_token_sidecar_to_path |
| `crates/prism-dtu-demo-server/src/multi_org_cmd.rs` `resolve_configure_url()` | Source | Template for resolve_configure_token |
| `crates/prism-dtu-demo-server/tests/ac_3_configure_endpoint.rs` | Source | Working pattern for X-Admin-Token in tests |
| `crates/prism-dtu-crowdstrike/tests/td_wv0_07_configure_requires_admin_token.rs` | Source | 401 behavior confirmed: no-token → 401, correct-token → 200 |
| `.factory/specs/architecture/decisions/ADR-003-dtu-reset-lookup-and-fidelity-auth.md` §Amendment #5 | Architecture | Normative source of X-Admin-Token requirement for all callers of POST /dtu/configure |
| BC-3.6.001: Per-Org Failure Injection | Behavioral contract | Precondition 4: configure calls require admin_token authentication |
| BC-2.06.017: Per-DTU-Instance Multi-Address Binding for Multi-Tenant Overlay Testing | Behavioral contract | Postcondition 1: MultiInstanceServers is the BC's runtime deliverable; adding admin_token_map is an extension |
| `.factory/specs/prd-supplements/error-taxonomy.md` §DEMO | Spec | E-DEMO-001..006 existing; E-DEMO-007 NEW to add |
