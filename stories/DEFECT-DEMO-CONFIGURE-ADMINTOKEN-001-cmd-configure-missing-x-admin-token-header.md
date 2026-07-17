---
document_type: story
story_id: "DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001"
title: "cmd_configure missing X-Admin-Token header — POST /dtu/configure returns 401"
wave: maintenance
epic_id: maintenance
priority: P1
status: draft
version: "0.9"
level: ops
producer: story-writer
timestamp: "2026-07-16"
modified: "2026-07-16"
input-hash: "a1addc6"
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
subsystems: [SS-22]
# Subsystem anchor justification:
#   SS-22 (Binary Entrypoint) owns the `prism-dtu-demo-server` CLI subcommand surface.
#   `cmd_configure` is a subcommand of the `prism-dtu-demo-server` binary, and both the
#   primary fix site (`cmd_configure` in main.rs) and the supporting infrastructure
#   (`MultiInstanceServers` admin_token_map, token sidecar write/read) live in the
#   `prism-dtu-demo-server` crate. Per ARCH-INDEX Subsystem Registry SS-22.
crates_touched:
  - prism-dtu-demo-server
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
| `deny_unknown_fields` helper | `crates/prism-dtu-harness/tests/review_2026_06_10_deny_unknown.rs` | varies | YES (same pattern — lowercase `x-admin-token` via `harness.admin_token_for()`) | Correct |
| `Harness::inject_failure()` production method | `crates/prism-dtu-harness/src/harness.rs` | ~285-295 | YES (lowercase `x-admin-token`; token from `self.admin_tokens.get(...)`) | Correct — harness library implementation backing the `configure_failure` and `deny_unknown_fields` callers (rows 7-8 above) |
| `test_build_harness_http_client_timeout_is_load_bearing` unit test | `crates/prism-dtu-harness/src/builder.rs` | ~1224-1270 | N/A | N/A — synthetic hung-socket timeout test; POSTs to a raw `TcpListener` that never responds; no DTU handler present; header not applicable |
| `configure_without_token_returns_401` (all 8 `td_wv0_07_*` per-clone) | `crates/prism-dtu-{crowdstrike,claroty,armis,cyberint,nvd,pagerduty,jira,threatintel}/tests/td_wv0_07_configure_requires_admin_token.rs` | ~14-35 per file | NO (intentional) | Correct — negative test (test 1 of 3): verifies server returns 401 when `X-Admin-Token` header is absent; 8 instances × 1 crate each = 8 POST calls |
| `configure_with_wrong_token_returns_401` (all 8 `td_wv0_07_*` per-clone) | Same 8 `td_wv0_07_` files | ~36-58 per file | WRONG token (intentional) | Correct — negative test (test 2 of 3): verifies server returns 401 when an incorrect token is presented; 8 instances = 8 POST calls |
| All `td_wv0_04_configure_deny_unknown.rs` tests | `crates/prism-dtu-{crowdstrike,claroty,armis,nvd,cyberint,threatintel}/tests/td_wv0_04_configure_deny_unknown.rs` (6 files) | varies | YES (correct token per ADR-003 Amendment #5 update) | Correct — deny-unknown-fields tests; 2 tests × 6 clones = 12 POST calls; all use `clone.admin_token()` |
| Per-DTU-crate `harness_tests.rs` (positive configure paths) | `crates/prism-dtu-{crowdstrike,claroty,armis,cyberint,jira,pagerduty}/tests/harness_tests.rs` (6 files) | varies | YES | Correct — 38 POST calls across 6 crates; configure clone state before behavioral assertions |
| Per-DTU-crate rate-limit, error-injection, reset, and auth-mode `ac_*` tests | `crates/prism-dtu-{armis,claroty,cyberint,nvd,threatintel}/tests/ac_{6,7,8}_*.rs` and similar | varies | YES | Correct — 21 POST calls; configure state before testing rate-limit / error / auth AC scenarios |
| `sec_p3_003_constant_time_admin_token.rs` test A (correct token → 200) | `crates/prism-dtu-claroty/tests/sec_p3_003_constant_time_admin_token.rs` | ~121-145 | YES | Correct — positive test: verifies constant-time comparison returns 200 with correct token |
| `sec_p3_003_constant_time_admin_token.rs` test B (wrong token → 401) | `crates/prism-dtu-claroty/tests/sec_p3_003_constant_time_admin_token.rs` | ~280-305 | WRONG token (intentional) | Correct — negative test: verifies constant-time comparison still returns 401 with incorrect token |
| `edge_cases.rs` (claroty and cyberint) | `crates/prism-dtu-claroty/tests/edge_cases.rs`, `crates/prism-dtu-cyberint/tests/edge_cases.rs` | varies | YES | Correct — 7 POST calls (2 claroty + 5 cyberint); configure state for edge-case scenarios |
| `fidelity.rs` (jira) — positive configure tests | `crates/prism-dtu-jira/tests/fidelity.rs` | ~450-500 | YES | Correct — 2 POST calls setting up fidelity test state |
| `fidelity.rs` (jira) — negative tests (headerless + wrong-token) | `crates/prism-dtu-jira/tests/fidelity.rs` | ~1286-1325 | NO / WRONG (intentional) | Correct — 2 negative tests parallel to `td_wv0_07`: `test_dtu_configure_without_admin_token_returns_401` (headerless) and `test_dtu_configure_with_wrong_admin_token_returns_401` (wrong token) |
| `fidelity.rs` (pagerduty) — positive configure tests | `crates/prism-dtu-pagerduty/tests/fidelity.rs` | ~401-420, ~611-630, ~743-760 | YES | Correct — 3 POST calls setting up fidelity test state |
| `fidelity.rs` (pagerduty) — negative test (headerless → 401) | `crates/prism-dtu-pagerduty/tests/fidelity.rs` | ~800-820 | NO (intentional) | Correct — `test_configure_without_admin_token_returns_401` negative test |
| Misc test POST calls — `multi_tenant.rs` (cyberint), `reset_state_invariants.rs` (armis), `dtu_reset_mount.rs` (threatintel), `ac_4/6/7_*.rs` (threatintel), `ac_tests.rs` (slack) | Various DTU crates | varies | YES | Correct — ~11 POST calls total; all configure clone state for behavioral tests using `clone.admin_token()` |
| In-process `clone.configure()` method call — HTTP auth not applicable | `crates/prism-dtu-cyberint/tests/bc_2_01_017_access_token_auth.rs` (line 54: `clone.configure(serde_json::json!({"access_token": "demo-access-key"})).await`); similar setup-only calls in other per-DTU-crate AC test files | N/A | N/A | Not an HTTP POST — calls `BehavioralClone::configure(serde_json::Value)` trait method in-process; mutates clone state directly without sending an HTTP request. `X-Admin-Token` header requirement does not apply. Excluded from the 138 HTTP POST total. |
| `defect_demo_configure_admintoken_001.rs` Test A (contract lock) | `crates/prism-dtu-demo-server/tests/defect_demo_configure_admintoken_001.rs` | ~121-145 | NO (intentional) | Correct — defect-observability contract lock: POST without `X-Admin-Token` → asserts 401; passes both before and after fix |
| `defect_demo_configure_admintoken_001.rs` Test B (post-fix verification) | `crates/prism-dtu-demo-server/tests/defect_demo_configure_admintoken_001.rs` | ~229-250 | YES | Correct — load-bearing post-fix test: sidecar-sourced token → asserts 200; fails before fix, passes after |

> **Sweep stats (v0.9 reconciliation, 2026-07-16):** Reproducible commands: (1) `rg -n 'dtu/configure' crates/ --type rust | wc -l` → **451 total hits**; (2) `rg -n '\.post\(.*dtu/configure' crates/ --type rust | wc -l` → **131 same-line** HTTP POST calls (URL inline in `.post(format!(...))`, literal regex match on same line); (3) `rg -n 'let url = format.*dtu/configure' crates/ --type rust` → **6 dynamic-URL** construction lines (URL built into a variable, then `.post(&url)` on a separate line); (4) `cmd_configure` in `src/main.rs` calls `.post(&configure_url)` where the URL is resolved via `resolve_configure_url()` — 1 additional dynamic caller not captured by command (3); total dynamic-URL callers = **7**. Grand total: **131 same-line + 7 dynamic = 138 total HTTP POST client calls**. Per-class tally: production CLI POST (defect, now FIXED) 1 · harness library production POST with header 1 · test POST with correct X-Admin-Token 114 · test POST intentionally headerless / no token (negative test) 11 · test POST with wrong token (negative test) 10 · synthetic hung-socket timeout N/A 1. Sum: 138. Excluded from total: in-process `BehavioralClone::configure()` method calls (no HTTP request emitted — see "In-process" row in table above). Future passes: re-run commands (2)-(3) and verify totals match; any unexpected new headerless POST is a CRITICAL defect.

**Result:** `cmd_configure()` in `main.rs` was the ONLY affected client-side call site (the defect, now FIXED). The complete sweep (138 total HTTP POST client calls across all files) confirms no other caller is missing the required header: 114 are correct-token test POSTs, 11 are intentional no-token negative tests (verifying 401 behavior), 10 are wrong-token negative tests (verifying 401 on bad credentials), and 1 is a synthetic hung-socket timeout unit test with no DTU handler (header N/A). The harness library production method `Harness::inject_failure()` (harness.rs:~287) and the harness test callers that invoke it (`bc_3_6_001_ops_clone_failure_modes.rs`, `review_2026_06_10_deny_unknown.rs`) all correctly attach the `x-admin-token` header via `Harness::admin_token_for()`. No production binary or library path outside the demo server and test harness infrastructure calls `/dtu/configure` directly.

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
| BC-2.06.017 | Per-DTU-Instance Multi-Address Binding for Multi-Tenant Overlay Testing | v1.11 | Postcondition 1 governs `MultiInstanceServers` (returned by `start_instances()`). The fix extends `MultiInstanceServers` with `admin_token_map()` — a parallel accessor to the existing `socket_map()`. The admin token sidecar uses the same atomic-write pattern as the URL sidecar governed by this BC's §Sidecar-availability guarantee (GAP-3 from S-DEMO-LAUNCHER-CONSOLIDATION-001). v1.11 formally enumerates the `admin_token_map() -> &HashMap<String, String>` accessor and `TOKEN_MULTI_FILE` sidecar format in Postcondition 1. |

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
   the multi-org case (EC-007 recovery form)

3. **Includes `X-Admin-Token: <token>` in the POST request** — verified by a test that
   asserts the response is HTTP 200 (not 401) when calling a clone that requires the header

Token sidecar write requirements:
- `write_url_sidecar()` in `main.rs` MUST also write `TOKEN_FILE` atomically (tmp+rename,
  same GAP-3 guarantee as the URL sidecar) — format: `{clone_name: admin_token}` where
  each key matches the corresponding key in `URL_FILE`
- `cmd_start_multi()` (via a new helper, parallel to `write_multi_url_sidecar`) MUST write
  `TOKEN_MULTI_FILE` atomically — format: `{org_slug: {sensor_id: admin_token}}` mirroring
  `URL_MULTI_FILE`
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

| Code | State | Domain | Message template | Retryable | Description |
|------|-------|--------|-----------------|-----------|-------------|
| E-DEMO-007 | broken | configuration | `"configure: E-DEMO-007: admin token for clone '{clone_name}' could not be resolved: {reason}"` | No | Runtime error in `cmd_configure`: the admin token sidecar (`TOKEN_FILE` or `TOKEN_MULTI_FILE`) is absent or does not contain an entry for `{clone_name}`. Occurs when the configure subcommand is invoked but the demo server is not running or was started in a different working directory. Operator must ensure the demo server is running in the same working directory and that the appropriate sidecar file exists. |

### AC-004 — Sibling sweep evidence: all /dtu/configure POST sites confirmed covered
(traces to BC-2.06.017 postcondition 1 — the admin_token_map on MultiInstanceServers is
the closing mechanism for start-multi mode)

A comment block at the top of the `cmd_configure` function body documents the TD-VSDD-060
sibling sweep, naming each enumerated site and confirming only `cmd_configure` was the
defect site. The new test in AC-001 explicitly references this comment.

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
| `write_token_sidecar` (new helper) | prism-dtu-demo-server library | `src/main.rs` | Effectful (file write) |
| `MultiInstanceServers::admin_token_map()` | prism-dtu-demo-server library | `src/multi_instance.rs` | Pure (accessor) |
| `start_instances()` amendment | prism-dtu-demo-server library | `src/multi_instance.rs` | Effectful (token extraction before async move) |
| `write_multi_admin_token_sidecar_to_path` (new fn) | prism-dtu-demo-server library | `src/multi_org_cmd.rs` | Effectful (file write) |
| `TOKEN_FILE`, `TOKEN_MULTI_FILE` constants | prism-dtu-demo-server library | `src/lib.rs` | Pure (constants) |
| `defect_demo_configure_admintoken_001.rs` (Red Gate test) | prism-dtu-demo-server tests | `tests/defect_demo_configure_admintoken_001.rs` | Effectful (integration test) |

**Architecture Compliance Rules (from ADR-003 Amendment #5 and existing harness patterns):**
- Token sidecar files MUST use the atomic tmp+rename write pattern (same as URL sidecars, GAP-3 guarantee)
- Token values MUST be read via `BehavioralClone::admin_token()` — never hardcoded, never reused across process invocations
- The token sidecar format MUST be a flat JSON object `{name: token}` for `start` mode and a nested object `{org_slug: {sensor_id: token}}` for `start-multi` mode — exactly mirroring the URL sidecar shape
- `start_instances()` MUST extract `clone.admin_token().to_string()` from each `BoundInstance` BEFORE it is moved into the detached watcher task — OWNERSHIP: once the clone is moved into `tokio::spawn(async move { drop(clone) })`, it is unreachable from the `start_instances` call site; extraction must happen in the bind loop before the spawn (note: `BoundInstance` is already `Box<dyn BehavioralClone>` so `admin_token()` remains callable on the trait object, but OWNERSHIP prevents access after the move)
- No token values may appear in structured log fields (AD-017 credential-safety rule; demo tokens are ephemeral test values, not production credentials, but the pattern must be consistent)

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Token mismatch — someone manually edits the token sidecar with a wrong value | Clone returns HTTP 401; `cmd_configure` receives non-2xx; prints `"HTTP 401"` + body to stdout and exits with code 1 (the non-2xx status-print-and-exit block of `cmd_configure`) |
| EC-002 | Clone without admin token configured ("dev mode") | NOT APPLICABLE — all `BehavioralClone` implementations generate a random UUID v4 token via `uuid::Uuid::new_v4().to_string()` at construction time (verified in the `admin_token` field initialization in `CrowdstrikeState::default`). There is no dev-mode no-token path. |
| EC-003 | Token sidecar exists but clone name is absent (e.g., `configure threatintel` after `start` that did not include threatintel) | E-DEMO-007: "clone 'threatintel' not found in token sidecar '.prism-dtu-demo-server.admin-tokens.json'" — exits code 1 |
| EC-004 | Token sidecar missing (server not running or different cwd) | E-DEMO-007: "token sidecar not found (start the demo server first with start or start-multi)" — exits code 1 |
| EC-005 | Ambiguous bare sensor name in `start-multi` mode (multiple orgs have same sensor) | Same disambiguation rule as `resolve_configure_url`: E-DEMO-007 with message "Bare sensor name '{name}' is ambiguous — found in N orgs: ["org-a", "org-b"]. Use full '{org_slug}-{sensor_id}' form." — the org list is rendered via Rust `{:?}` on a `Vec<String>`, producing quoted strings (e.g. `["org-a", "org-b"]`), consistent with the sibling `resolve_configure_url` ambiguity arm (same `{:?}` format, verified in the multi-match ambiguity arm of `resolve_configure_token` in `multi_org_cmd.rs`). |
| EC-006 | Concurrent `start-multi` restarts the demo server; old token sidecar has stale tokens | New token sidecar is written atomically on each start; stale-token 401 from server is surfaced (existing AC-003 flow); operator must retry |
| EC-007 | TLS-enabled `start --tls` — clone URLs use `https://`; configure must still work | URL resolution unchanged (URL sidecar uses `https://`); token sidecar is independent of TLS; test harness uses `danger_accept_invalid_certs(true)` where needed |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `crates/prism-dtu-demo-server/src/main.rs` (`cmd_configure`, `write_token_sidecar`) | effectful-shell | Performs HTTP POST and file I/O |
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
| Atomic write (tmp+rename) for all sidecar files | GAP-3 (S-DEMO-LAUNCHER-CONSOLIDATION-001) | Token sidecars must use the same pattern; partial-write reads from demo-run.sh must be impossible |
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
| MODIFY | `crates/prism-dtu-demo-server/src/lib.rs` | Add `pub const TOKEN_FILE: &str` and `pub const TOKEN_MULTI_FILE: &str` declarations |
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
| v0.9 | 2026-07-16 | product-owner FIX-BURST-8 SPEC (F-ADMTOK-P9-MED-001 + F-ADMTOK-P9-LOW-001) | F-ADMTOK-P9-MED-001 (MED): v0.8 sweep tally was not reproducible — claimed "116 same-line + 7 dynamic = 123" using a flawed filter that subtracted manually-estimated comment false-positives; the precise command `rg -n '\.post\(.*dtu/configure' crates/ --type rust \| wc -l` returns 131 directly, giving 131+7=138. Footnote rewritten with verbatim commands and corrected per-class tally (114 correct-token, 11 no-token, 10 wrong-token, 1 prod CLI FIXED, 1 harness lib, 1 N/A → sum 138). Summary sentence count updated from 123 to 138; class counts updated (99→114 correct-token). F-ADMTOK-P9-LOW-001 (LOW): three phantom/misclassified rows fixed. (a) `bc_2_06_019_scenario_progression.rs` row removed — file has zero HTTP POST calls to /dtu/configure; it uses `admin_token()` only for Bearer auth on GET data-plane endpoints, confirmed by `rg -n 'dtu/configure'` showing no hits for this file. (b) `bc_2_01_017_access_token_auth.rs {cyberint,nvd}` citation removed from ac_* HTTP-POST group — cyberint line 54 is `clone.configure(serde_json::json!({...})).await` (in-process trait method, no HTTP emitted); nvd variant does not exist; count corrected 22→21. (c) `org_tagging.rs (jira)` removed from misc HTTP-POST group — file has no configure calls of any kind. Added new "In-process `clone.configure()` method call — HTTP auth not applicable" classification row anchoring these sites correctly. |
| v0.8 | 2026-07-16 | product-owner FIX-BURST-7 SPEC (F-ADMTOK-P8-LOW-001) | Full AC-004 sibling sweep reconciliation. Ran complete `rg -n 'dtu/configure' crates/ --type rust` (451 total hits, 123 POST client calls). Added 16 new rows to §Root Cause TD-VSDD-060 table covering: `Harness::inject_failure()` production method (harness.rs:~287); builder.rs hung-socket timeout test (N/A); td_wv0_07_* no-token negative tests ×8 clones; td_wv0_07_* wrong-token negative tests ×8 clones; td_wv0_04_* deny-unknown tests ×6 clones (12 POSTs); per-DTU harness_tests.rs ×6 crates (38 POSTs); ac_* rate-limit/error/auth tests ×5 crates (22 POSTs); sec_p3_003 correct/wrong-token tests (claroty); edge_cases.rs (claroty, cyberint); fidelity.rs positive tests and 401-validation negative tests (jira, pagerduty); misc tests (multi_tenant, reset_state_invariants, dtu_reset_mount, ac_tests slack, etc.); defect_demo Test A (contract lock) and Test B (post-fix). Added footnote with sweep stats and per-class tally for mechanical future verification. Updated summary sentence to reference all 123 call sites and final class counts. |
| v0.7 | 2026-07-16 | product-owner FIX-BURST-6 SPEC (F-ADMTOK-P7-LOW-001 + F-ADMTOK-P7-OBS-001) | F-ADMTOK-P7-LOW-001: §Root Cause TD-VSDD-060 sibling table was missing two harness-crate call sites — added `configure_failure` helper in `bc_3_6_001_ops_clone_failure_modes.rs` (the BC-3.6.001 test itself; lowercase `x-admin-token` via `Harness::admin_token_for()`) and `deny_unknown_fields` helper in `review_2026_06_10_deny_unknown.rs` (same token-source pattern); both correctly authenticated. Updated summary sentence to note the harness-token variant (`Harness::admin_token_for()` vs `clone.admin_token()`). F-ADMTOK-P7-OBS-001: AC-001 "The test MUST fail before the fix lands" sentence was internally contradictory — a POST-without-header / assert-401 test passes before AND after (server 401 gate predates the branch). Replaced with contract-lock explanation: steps 1-4 are a defect-observability lock (always passes); Red-Gate obligation lives in binary E2E exit-0 assertion and sidecar-existence tests. |
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
