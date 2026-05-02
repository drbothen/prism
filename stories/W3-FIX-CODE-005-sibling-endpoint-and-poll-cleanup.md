---
story_id: W3-FIX-CODE-005
title: "DTU harness + Armis/CrowdStrike: sibling poll-backoff propagation and missing org-id guards"
wave: 3.3
level: "L4"
target_module: prism-dtu-harness
subsystems: [SS-01]
priority: P1
depends_on: []
blocks: []
estimated_days: 2
points: 5
status: merged
document_type: story
version: "1.0"
producer: story-writer
timestamp: "2026-05-02T00:00:00Z"
input-hash: ""
inputs:
  - .factory/cycles/wave-3-multi-tenant/gate-step-c-code-review-pass3.md
  - .factory/specs/behavioral-contracts/BC-3.5.001-harness-logical-isolation.md
  - .factory/specs/behavioral-contracts/BC-3.5.002-harness-network-isolation.md
  - .factory/specs/behavioral-contracts/BC-3.2.001-per-org-sensor-data-isolation.md
  - .factory/specs/behavioral-contracts/BC-3.6.001-per-org-failure-injection.md
traces_to: []
cycle: "v1.0.0-greenfield"
epic_id: "E-3.5"
phase: 3
behavioral_contracts:
  - BC-3.5.001
  - BC-3.5.002
  - BC-3.2.001
  - BC-3.6.001
verification_properties: [VP-124, VP-125, VP-126, VP-128]
assumption_validations: []
risk_mitigations: []
anchor_bcs: [BC-3.5.001, BC-3.5.002, BC-3.2.001, BC-3.6.001]
anchor_capabilities: [CAP-036]
anchor_subsystem: ["SS-01"]
tdd_mode: strict
parent_finding: "CR-016 (M), CR-017 (M), CR-018 (M), CR-020 (L), L-50-004 (L)"
# BC status: anchored — all BCs fully authored
---

# W3-FIX-CODE-005: DTU harness + Armis/CrowdStrike — sibling poll-backoff propagation and missing org-id guards

## Narrative

As a Prism maintainer, I want the pass-50 MEDIUM sibling-propagation gaps in
`prism-dtu-harness`, `prism-dtu-armis`, and `prism-dtu-crowdstrike` remediated, so
that the Wave 3 integration gate proceeds to an APPROVE verdict with no residual
guard-bypass asymmetries on org-keyed clone endpoints and no wasted CPU cycles from
un-migrated 10ms poll loops.

## Objective

Pass 3 of Gate Step C (`gate-step-c-code-review-pass3.md`) identified three new MEDIUM
findings and two LOW findings against the W3.2 mass merge. Each is a partial-fix gap:
a prior fix story correctly addressed the root location but did not propagate to sibling
files. The items in scope are bundled here to avoid per-finding story overhead.

**Items in scope:**

| ID | Severity | Crate | One-line description |
|----|----------|-------|----------------------|
| CR-016 | MEDIUM | prism-dtu-harness | 3 clone-specific `poll_test_hook` mirrors still sleep 10ms; `clone_server.rs` was fixed but siblings missed |
| CR-017 / M-50-001 | MEDIUM | prism-dtu-armis | `validate_org_id` dual-mode guard absent from `tags.rs` (post/delete) and `alerts.rs` (get) |
| CR-018 | MEDIUM | prism-dtu-crowdstrike | `validate_org_id` nil-instance guard absent from `detections.rs` (list_detection_ids, get_detection_summaries) |
| CR-020 | LOW | prism-customer-config | AC-005 deviation (pub vs pub(crate)) undocumented at `validate_spec_path` call site |
| L-50-004 | LOW | prism-dtu-harness | TD-W3-POLL-NOTIFY-001 follow-up for Notify-based cancellation not filed |

## Behavioral Contracts

| BC ID | Title | Relevant Clause |
|-------|-------|-----------------|
| BC-3.5.001 | Harness Logical Isolation Invariants | Postcondition 5 (200ms startup budget — excess wake-ups degrade CI timing); CR-016 fix reduces harness wake-up rate from 300/s to 60/s |
| BC-3.5.002 | Harness Network Isolation Invariants | Precondition 3 (authentication middleware enforces per-clone `instance_org_id`); Precondition 6 (each clone's auth initialized with own `admin_token`) — CR-017/CR-018 close guard-bypass gaps on tag/alert/detection endpoints |
| BC-3.2.001 | Per-Org Sensor Data Isolation via Composite HashMap Key | Precondition 4 (dispatch layer verifies OrgId match before invoking DTU method) — CR-017/CR-018 ensure HTTP layer enforces the same boundary before any state access |
| BC-3.6.001 | Per-Org Failure Injection | Invariant 1 (failure state scoped strictly to target `(OrgId, DtuType)` clone) — poll-cadence fix ensures harness CPU budget does not degrade concurrent failure-injection tests |

## Acceptance Criteria

### AC-001: CR-016 — All three clone-specific poll mirrors updated to 50ms (traces to BC-3.5.001 postcondition 5)

The following three functions are each updated from `tokio::time::sleep(Duration::from_millis(10))` to
`tokio::time::sleep(Duration::from_millis(50))`:

**Scope coverage table (CR-016):**

| File | Function | Line (approx) | Before | After |
|------|----------|---------------|--------|-------|
| `crates/prism-dtu-harness/src/clones/armis.rs` | `poll_armis_test_hook` | 901 | `from_millis(10)` | `from_millis(50)` |
| `crates/prism-dtu-harness/src/clones/claroty.rs` | `poll_claroty_test_hook` | 850 | `from_millis(10)` | `from_millis(50)` |
| `crates/prism-dtu-harness/src/clones/crowdstrike.rs` | `poll_test_hook_crowdstrike` | 1161 | `from_millis(10)` | `from_millis(50)` |

Each updated sleep call carries the identical comment used in the `clone_server.rs` fix:
```rust
// 50ms polling cadence (CR-006 / W3-FIX-CODE-002 AC-004);
// replace with tokio::sync::Notify in a future pass (TD-W3-POLL-NOTIFY-001).
tokio::time::sleep(std::time::Duration::from_millis(50)).await;
```

Combined effect: a 12-clone harness (2× Armis + 2× Claroty + 2× CrowdStrike) drops from
~300 mirror wake-ups/second to ~60 wake-ups/second across all clone-specific poll functions.

### AC-002: CR-017 — Armis `validate_org_id` dual-mode guard applied to all remaining tag and alert endpoints (traces to BC-3.5.002 precondition 3 and BC-3.2.001 precondition 4)

The same `is_real_org` dual-mode guard already applied to `get_or_post_devices` and
`post_devices` in `devices.rs` (by W3-FIX-CODE-004) is now applied to ALL remaining
Armis route handlers that access org-keyed state.

**Scope coverage table (CR-017) — complete list of handlers receiving the guard:**

| File | Handler | Endpoint | Status before this story |
|------|---------|----------|--------------------------|
| `crates/prism-dtu-armis/src/routes/tags.rs` | `post_device_tag` | `POST /api/v1/devices/{id}/tags/` | guard absent (only `check_bearer_auth`) |
| `crates/prism-dtu-armis/src/routes/tags.rs` | `delete_device_tag` | `DELETE /api/v1/devices/{id}/tags/{key}` | guard absent (only `check_bearer_auth`) |
| `crates/prism-dtu-armis/src/routes/alerts.rs` | `get_alerts` | `GET /api/v1/alerts` | guard absent (only `check_bearer_auth`) |

Guard pattern applied (identical to `devices.rs:89-94`):
```rust
let is_real_org = state.instance_org_id != crate::state::DTU_DEFAULT_INSTANCE_ORG_ID;
if is_real_org || headers.get("x-org-id").is_some() {
    if let Err((status, body)) = validate_org_id(&headers, state.instance_org_id) {
        return (status, body).into_response();
    }
}
```

After this fix:
- A real-org Armis clone rejects requests to tag write/delete and alert read endpoints
  that omit the `X-Org-Id` header with HTTP 401, matching the behavior of `devices.rs`.
- Default-instance clones retain backward-compatibility: validation fires only when the
  header is present.

Tests: at least three new test functions added (one per handler) to
`tests/cr012_validate_org_id_consistency.rs` (or a new sibling test file), verifying:
- Real-org clone + absent `X-Org-Id` → HTTP 401.
- Real-org clone + correct `X-Org-Id` → HTTP 200.
- Default-instance clone + absent `X-Org-Id` → HTTP 200 (backward compat).

No existing test may be removed or modified to pass; all new tests are additive.

### AC-003: CR-018 — CrowdStrike `validate_org_id` nil-instance guard applied to both detection handlers (traces to BC-3.5.002 precondition 3 and BC-3.2.001 precondition 4)

The same nil-instance guard applied to `hosts.rs` and `writes.rs` in W3-FIX-SEC-001 is
now applied to the two detection handlers.

**Scope coverage table (CR-018) — complete list of handlers receiving the guard:**

| File | Handler | Endpoint | Registered in | Status before this story |
|------|---------|----------|---------------|--------------------------|
| `crates/prism-dtu-crowdstrike/src/routes/detections.rs` | `list_detection_ids` | `GET /detections/queries/detections/v1` | `mod.rs:181` | guard absent |
| `crates/prism-dtu-crowdstrike/src/routes/detections.rs` | `get_detection_summaries` | `POST /detections/entities/detections/v2` | `mod.rs:184` | guard absent |

Guard pattern applied (identical to `hosts.rs:146-150`):
```rust
if state.instance_org_id != OrgId::from_uuid(uuid::Uuid::nil()) {
    if let Err((status, body)) = validate_org_id(&headers, state.instance_org_id) {
        return (status, body).into_response();
    }
}
```

After this fix:
- A real-org CrowdStrike clone rejects detection query requests from callers that
  omit or supply an incorrect `X-Org-Id` header with HTTP 401, consistent with `hosts.rs`
  and `writes.rs`.
- Default-instance (nil `instance_org_id`) clones are unaffected; the guard is
  a no-op for those.

Tests: at least two new test functions added to
`crates/prism-dtu-crowdstrike/tests/` (mirroring the `dtu_reset_auth.rs` structure),
verifying:
- Real-org clone + absent `X-Org-Id` on `GET /detections/queries/detections/v1` → HTTP 401.
- Real-org clone + absent `X-Org-Id` on `POST /detections/entities/detections/v2` → HTTP 401.
- Real-org clone + correct `X-Org-Id` on each endpoint → HTTP 200.

### AC-004: CR-020 — `validate_spec_path` pub-vs-pub(crate) deviation documented at call site (traces to BC-3.3.004 invariant 1)

A comment block is added immediately above the `#[doc(hidden)]` attribute on
`validate_spec_path` in `crates/prism-customer-config/src/validator.rs`:

```rust
// AC-005 deviation (W3-FIX-CODE-004): story spec required pub(crate) but
// integration tests in tests/path_traversal.rs (external binaries in Rust's
// visibility model) call this function directly. pub(crate) would exclude
// integration test binaries from the allowed caller set and break those tests.
// pub + #[doc(hidden)] is the correct idiomatic Rust compromise:
//   - #[doc(hidden)] prevents accidental stable-API coupling by excluding the
//     function from rustdoc output and auto-complete suggestions.
//   - Any intentional external caller must explicitly navigate to this symbol.
// See W3-FIX-CODE-004 decision record for full rationale.
#[doc(hidden)]
pub fn validate_spec_path(
```

No functional change. No test change required.

### AC-005: L-50-004 — Tech debt record TD-W3-POLL-NOTIFY-001 filed (traces to BC-3.5.001 postcondition 5)

A tech debt entry is appended to `.factory/specs/tech-debt.md` (or created if absent)
with the following content:

```markdown
## TD-W3-POLL-NOTIFY-001: Replace poll_test_hook busy-wait with tokio::sync::Notify

**Filed:** 2026-05-02
**Source:** CR-006 (pass-1 code review), closed by W3-FIX-CODE-002 + W3-FIX-CODE-005 (50ms cadence)
**Wave:** Wave 4 candidate
**Severity:** LOW (performance; no correctness impact)
**Description:** The `poll_test_hook`, `poll_armis_test_hook`, `poll_claroty_test_hook`,
and `poll_test_hook_crowdstrike` functions use a 50ms `tokio::time::sleep` busy-wait loop
to detect harness test-hook notifications. A `tokio::sync::Notify`-based approach would
eliminate the residual 60 wake-ups/second in a 12-clone harness and make cancellation
instant. The 50ms cadence is an acceptable short-term workaround but should be replaced
before the harness scales beyond 12 clones per CI run.
**Affected files:**
- `crates/prism-dtu-harness/src/clone_server.rs` (`poll_test_hook`)
- `crates/prism-dtu-harness/src/clones/armis.rs` (`poll_armis_test_hook`)
- `crates/prism-dtu-harness/src/clones/claroty.rs` (`poll_claroty_test_hook`)
- `crates/prism-dtu-harness/src/clones/crowdstrike.rs` (`poll_test_hook_crowdstrike`)
**Resolution:** Replace `loop { sleep(50ms); if flag { break; } }` with
`notify.notified().await` and a corresponding `notify.notify_one()` at the
notification call site.
```

## Tasks

### Part A: Poll cadence propagation (CR-016)

1. Read `crates/prism-dtu-harness/src/clones/armis.rs` around line 901 — locate
   `poll_armis_test_hook` and its `sleep(10ms)` call.
2. Replace `from_millis(10)` with `from_millis(50)` and add the CR-006/TD-W3-POLL-NOTIFY-001
   comment (AC-001 exact wording).
3. Read `crates/prism-dtu-harness/src/clones/claroty.rs` around line 850 — locate
   `poll_claroty_test_hook` and its `sleep(10ms)` call.
4. Apply the same fix.
5. Read `crates/prism-dtu-harness/src/clones/crowdstrike.rs` around line 1161 — locate
   `poll_test_hook_crowdstrike` and its `sleep(10ms)` call.
6. Apply the same fix.
7. Verify no other `from_millis(10)` occurrences remain in `crates/prism-dtu-harness/src/clones/`
   (`grep -r "from_millis(10)" crates/prism-dtu-harness/src/clones/` should produce zero output).

### Part B: Armis tag/alert org-id guard (CR-017)

8. Read `crates/prism-dtu-armis/src/routes/devices.rs` lines 85-100 — confirm the exact
   form of the `is_real_org` guard already applied to `get_or_post_devices`.
9. Read `crates/prism-dtu-armis/src/state.rs` — confirm `DTU_DEFAULT_INSTANCE_ORG_ID`
   constant name (NOT `OrgId::nil()` — Armis uses a non-nil default sentinel).
10. Read `crates/prism-dtu-armis/src/routes/tags.rs` lines 30-80 — locate `post_device_tag`
    (line ~32) and `delete_device_tag` (line ~65).
11. Apply the `is_real_org` guard to both handlers immediately after `check_bearer_auth`
    (AC-002 exact pattern).
12. Read `crates/prism-dtu-armis/src/routes/alerts.rs` — locate `get_alerts` (line ~38 if
    present; skip silently if not present and document the skip as a finding in PR description).
13. If `get_alerts` is present, apply the same guard.
14. Write at least three new test functions in
    `crates/prism-dtu-armis/tests/cr012_validate_org_id_consistency.rs` (or
    `tests/cr017_tag_alert_org_id_guard.rs`):
    - `test_post_device_tag_real_org_absent_header_returns_401`
    - `test_delete_device_tag_real_org_absent_header_returns_401`
    - `test_get_alerts_real_org_absent_header_returns_401` (if handler exists)
    Each test follows the existing structure: construct a real-org clone, send request
    without `X-Org-Id` header, assert HTTP 401.

### Part C: CrowdStrike detection guard (CR-018)

15. Read `crates/prism-dtu-crowdstrike/src/routes/detections.rs` lines 100-200 — locate
    `list_detection_ids` and `get_detection_summaries` and confirm they lack `validate_org_id`.
16. Read `crates/prism-dtu-crowdstrike/src/routes/hosts.rs` lines 142-155 — capture the
    exact form of the nil-instance guard.
17. Apply the nil-instance guard to both handlers in `detections.rs` (AC-003 exact pattern).
18. Write at least four new test functions in
    `crates/prism-dtu-crowdstrike/tests/cr018_detections_org_id_guard.rs`:
    - `test_list_detection_ids_real_org_absent_header_returns_401`
    - `test_list_detection_ids_real_org_correct_header_returns_200`
    - `test_get_detection_summaries_real_org_absent_header_returns_401`
    - `test_get_detection_summaries_real_org_correct_header_returns_200`

### Part D: CR-020 comment and tech debt record

19. Read `crates/prism-customer-config/src/validator.rs` around line 741 — locate the
    `#[doc(hidden)] pub fn validate_spec_path` declaration.
20. Insert the deviation comment block immediately above `#[doc(hidden)]` (AC-004 exact text).
21. Read `.factory/specs/tech-debt.md` if it exists; append TD-W3-POLL-NOTIFY-001 entry.
    If it does not exist, create it with a header and the entry (AC-005).

### Part E: Integration

22. Run `cargo test -p prism-dtu-harness -p prism-dtu-armis -p prism-dtu-crowdstrike
    -p prism-customer-config --all-features` — all tests pass including the new ones.
23. Run `grep -r "from_millis(10)" crates/prism-dtu-harness/src/clones/` — zero matches.
24. Run `cargo clippy --workspace -- -D warnings` — no new warnings.
25. Open PR to `develop`.

## Architecture Mapping

| Component | Module | File(s) | Pure/Effectful |
|-----------|--------|---------|----------------|
| `poll_armis_test_hook` cadence | prism-dtu-harness | `crates/prism-dtu-harness/src/clones/armis.rs:901` | Pure (comment + constant change) |
| `poll_claroty_test_hook` cadence | prism-dtu-harness | `crates/prism-dtu-harness/src/clones/claroty.rs:850` | Pure (comment + constant change) |
| `poll_test_hook_crowdstrike` cadence | prism-dtu-harness | `crates/prism-dtu-harness/src/clones/crowdstrike.rs:1161` | Pure (comment + constant change) |
| Armis `post_device_tag` org-id guard | prism-dtu-armis | `crates/prism-dtu-armis/src/routes/tags.rs:32` | Pure (guard expression change) |
| Armis `delete_device_tag` org-id guard | prism-dtu-armis | `crates/prism-dtu-armis/src/routes/tags.rs:65` | Pure (guard expression change) |
| Armis `get_alerts` org-id guard | prism-dtu-armis | `crates/prism-dtu-armis/src/routes/alerts.rs:38` | Pure (guard expression change) |
| CrowdStrike `list_detection_ids` guard | prism-dtu-crowdstrike | `crates/prism-dtu-crowdstrike/src/routes/detections.rs:105` | Pure (guard expression change) |
| CrowdStrike `get_detection_summaries` guard | prism-dtu-crowdstrike | `crates/prism-dtu-crowdstrike/src/routes/detections.rs:183` | Pure (guard expression change) |
| `validate_spec_path` deviation comment | prism-customer-config | `crates/prism-customer-config/src/validator.rs:741` | Pure (comment only) |
| TD-W3-POLL-NOTIFY-001 record | factory-specs | `.factory/specs/tech-debt.md` | N/A (documentation) |

**Subsystem anchor justification:** SS-01 (Sensor Adapters) owns this story's scope
because all runtime changes are in `prism-dtu-harness`, `prism-dtu-armis`, and
`prism-dtu-crowdstrike` — all Sensor Adapter subsystem crates per the ARCH-INDEX
Subsystem Registry. The `prism-customer-config` comment (CR-020) co-rides this story
for bundling efficiency; its subsystem (SS-06) contribution is documentation-only and
does not justify a separate story.

**Dependency anchor justification:** `depends_on: []` — all five items are
self-contained and require no other W3.3 story to land first. W3-FIX-CODE-004 already
applied the guard to `get_or_post_devices` and `post_devices`; this story extends to
the missed siblings. `blocks: []` — no downstream story gates on these fixes.

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `clones/armis.rs` poll cadence | pure-core | Constant change (`10` → `50`) + comment; no I/O, no state |
| `clones/claroty.rs` poll cadence | pure-core | Constant change + comment; no I/O, no state |
| `clones/crowdstrike.rs` poll cadence | pure-core | Constant change + comment; no I/O, no state |
| `tags.rs` `post_device_tag` guard | pure-core | Early-return conditional expression; validation returns `Result`; no I/O |
| `tags.rs` `delete_device_tag` guard | pure-core | Early-return conditional expression; validation returns `Result`; no I/O |
| `alerts.rs` `get_alerts` guard | pure-core | Early-return conditional expression; validation returns `Result`; no I/O |
| `detections.rs` `list_detection_ids` guard | pure-core | Early-return conditional expression; validation returns `Result`; no I/O |
| `detections.rs` `get_detection_summaries` guard | pure-core | Early-return conditional expression; validation returns `Result`; no I/O |
| `validator.rs` deviation comment | pure-core | Comment text only; zero runtime effect |
| `tech-debt.md` TD entry | N/A | Documentation artifact; no runtime classification |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `poll_armis_test_hook` / `poll_claroty_test_hook` / `poll_test_hook_crowdstrike` called in 12-clone harness | Combined wake-up rate drops from ~300/s to ~60/s across all clone-specific poll functions; CI timing budget headroom increases |
| EC-002 | `post_device_tag` sent to real-org Armis clone without `X-Org-Id` | HTTP 401 returned before any state mutation; tag write does not proceed |
| EC-003 | `delete_device_tag` sent to real-org Armis clone without `X-Org-Id` | HTTP 401 returned before any state mutation; no deletion occurs |
| EC-004 | `get_alerts` sent to real-org Armis clone without `X-Org-Id` | HTTP 401 returned; no alert data exposed |
| EC-005 | `list_detection_ids` sent to real-org CrowdStrike clone without `X-Org-Id` | HTTP 401 returned; detection store not queried |
| EC-006 | `get_detection_summaries` sent to real-org CrowdStrike clone without `X-Org-Id` | HTTP 401 returned; detection store not queried |
| EC-007 | `list_detection_ids` sent to nil-instance CrowdStrike clone (default/test) without `X-Org-Id` | Guard condition `instance_org_id != Uuid::nil()` is false; request proceeds normally (backward compat) |
| EC-008 | Armis `post_device_tag` sent to default-instance clone without `X-Org-Id` | Guard condition `instance_org_id != DTU_DEFAULT_INSTANCE_ORG_ID` is false; request proceeds normally |
| EC-009 | `alerts.rs::get_alerts` handler not present in current codebase | Task 13 is skipped; PR description documents the absence; no test is written for a non-existent handler |
| EC-010 | CR-020 comment read by future developer | Comment explains the public-vs-pub(crate) tradeoff; developer does not incorrectly conclude the function was left public by accident |

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~6 000 |
| BC files (4 BCs) | ~8 000 |
| `clones/armis.rs` (line 895-910, ~15 lines) | ~300 |
| `clones/claroty.rs` (line 844-860, ~15 lines) | ~300 |
| `clones/crowdstrike.rs` (line 1155-1170, ~15 lines) | ~300 |
| `prism-dtu-armis/src/routes/tags.rs` (lines 25-80, ~55 lines) | ~700 |
| `prism-dtu-armis/src/routes/alerts.rs` (lines 30-50, ~20 lines) | ~400 |
| `prism-dtu-armis/src/routes/devices.rs` (lines 85-100 for guard pattern) | ~300 |
| `prism-dtu-armis/src/state.rs` (DTU_DEFAULT_INSTANCE_ORG_ID constant) | ~200 |
| `prism-dtu-crowdstrike/src/routes/detections.rs` (lines 100-200) | ~1 000 |
| `prism-dtu-crowdstrike/src/routes/hosts.rs` (lines 142-155 for guard pattern) | ~300 |
| `prism-customer-config/src/validator.rs` (lines 735-750) | ~300 |
| `tests/cr012_validate_org_id_consistency.rs` (existing tests for reference) | ~800 |
| New test files (3 Armis + 4 CrowdStrike, ~40 lines each) | ~1 400 |
| `cargo test` + `cargo clippy` output | ~1 000 |
| **Total** | **~21 300** |

Within single-agent context window. Load only the specific line ranges listed above;
do not load entire crate source files.

## Previous Story Intelligence

- **W3-FIX-CODE-002** (PR #120): fixed `clone_server.rs:838` from 10ms to 50ms (CR-006).
  The fix was scoped to that one file. CR-016 is the propagation gap — three sibling
  poll functions in per-clone modules were not updated. The fix here is mechanically
  identical: same constant, same comment.
- **W3-FIX-CODE-004** (PR #118): applied Armis dual-mode `is_real_org` guard to
  `devices.rs`. CR-017 is the incomplete-propagation gap — the AC-003 story spec
  said "all tag endpoints" but the implementation covered only `get_or_post_devices`
  and `post_devices`. The guard pattern is already correct and tested; this story
  extends it to the three missed handlers.
- **W3-FIX-SEC-001** (PR #113): applied nil-instance guard to CrowdStrike `hosts.rs`
  and `writes.rs`. CR-018 is the detection-endpoint gap — `detections.rs` registers
  two routes in `mod.rs` but was not included in the W3-FIX-SEC-001 scope. The guard
  pattern is already proven; this story extends it.
- **Pattern lesson:** Every fix story should include an explicit "sibling scan" step:
  `grep -r "<fixed_pattern_before>" crates/<crate>/src/` to confirm there are no
  other occurrences of the old pattern in the same crate before closing the story.
  The absence of this step in CODE-002, CODE-004, and SEC-001 caused all three
  sibling gaps identified in pass-50.

## Architecture Compliance Rules

- The Armis guard MUST compare against `crate::state::DTU_DEFAULT_INSTANCE_ORG_ID`,
  NOT against `OrgId::from_uuid(Uuid::nil())`. Armis uses a non-nil default sentinel
  — using nil would break legacy single-tenant clone tests.
- The CrowdStrike guard MUST use `OrgId::from_uuid(uuid::Uuid::nil())` — the
  CrowdStrike default instance IS the nil UUID (unlike Armis). Mixing the two
  sentinel constants is a correctness error.
- Poll cadence comment MUST reference both `CR-006 / W3-FIX-CODE-002 AC-004` and
  `TD-W3-POLL-NOTIFY-001` to ensure the tech debt record is traceable from the call site.
- Do NOT change the sleep duration in `clone_server.rs` — that file was already fixed
  by W3-FIX-CODE-002 and must remain at 50ms unchanged.
- The deviation comment for `validate_spec_path` MUST be placed above `#[doc(hidden)]`,
  not above the `pub fn` keyword, so it reads as an intent comment rather than doc text.
- All new tests MUST follow the naming convention `test_<handler>_<condition>_<result>`
  (e.g., `test_post_device_tag_real_org_absent_header_returns_401`) consistent with
  the existing `cr012_validate_org_id_consistency.rs` corpus.

## Library & Framework Requirements

| Library | Version (workspace pin) | Purpose |
|---------|------------------------|---------|
| `axum` | workspace pin | Route handler extractors (Armis/CrowdStrike guard changes) |
| `tokio::time` | workspace pin | `Duration::from_millis(50)` in poll functions |
| `uuid` | workspace pin | `uuid::Uuid::nil()` in CrowdStrike guard |

No new Cargo dependencies introduced by this story.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-dtu-harness/src/clones/armis.rs` | Modify | Line ~901: 10ms → 50ms + comment (AC-001) |
| `crates/prism-dtu-harness/src/clones/claroty.rs` | Modify | Line ~850: 10ms → 50ms + comment (AC-001) |
| `crates/prism-dtu-harness/src/clones/crowdstrike.rs` | Modify | Line ~1161: 10ms → 50ms + comment (AC-001) |
| `crates/prism-dtu-armis/src/routes/tags.rs` | Modify | Add `is_real_org` guard to `post_device_tag` (~line 32) and `delete_device_tag` (~line 65) (AC-002) |
| `crates/prism-dtu-armis/src/routes/alerts.rs` | Modify (if handler present) | Add `is_real_org` guard to `get_alerts` (~line 38) if handler exists (AC-002) |
| `crates/prism-dtu-armis/tests/cr017_tag_alert_org_id_guard.rs` | Create | 3+ new test functions (AC-002) |
| `crates/prism-dtu-crowdstrike/src/routes/detections.rs` | Modify | Add nil-instance guard to `list_detection_ids` (~line 105) and `get_detection_summaries` (~line 183) (AC-003) |
| `crates/prism-dtu-crowdstrike/tests/cr018_detections_org_id_guard.rs` | Create | 4 new test functions (AC-003) |
| `crates/prism-customer-config/src/validator.rs` | Modify | Line ~741: add deviation comment above `#[doc(hidden)]` (AC-004) |
| `.factory/specs/tech-debt.md` | Create or Modify | Append TD-W3-POLL-NOTIFY-001 entry (AC-005) |

## Forbidden Dependencies

- Do NOT add any new external crate dependencies to any crate modified by this story.
- Do NOT modify `clone_server.rs` poll cadence — it is already correctly fixed at 50ms.
- Do NOT change the public API of any handler — the `validate_org_id` guard is an early
  return inside the handler body; no function signature changes.
- Do NOT use `OrgId::from_uuid(Uuid::nil())` in Armis handlers, and do NOT use
  `crate::state::DTU_DEFAULT_INSTANCE_ORG_ID` in CrowdStrike handlers. The two
  sentinel forms are crate-specific; mixing them is a correctness error.
- Do NOT remove or alter the existing tests in `cr012_validate_org_id_consistency.rs` —
  all additions must be in a new test file or appended as new functions.
