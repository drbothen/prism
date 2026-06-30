---
document_type: story
story_id: S-PERF-GATE-005
title: "DTU clone stop() graceful-shutdown wiring — internal broadcast channel so stop() returns in < 500ms instead of always hitting the 5s hard-abort path"
epic_id: EPIC-MAINTENANCE
version: "1.1"
status: draft
producer: story-writer
phase: 3
wave: maintenance
priority: P1
points: 5
tdd_mode: "strict"
# tdd_mode rationale: production Rust code changes across 10 crates (prism-dtu-common +
# 9 clone crates). Red Gate test is required: a `#[tokio::test]` that calls start() then
# stop() on an idle CyberintClone and asserts wall-clock < 500ms. This FAILS before the
# fix (stop() takes ~5s via the always-hit 5s select timeout) and passes after.
target_module: "prism-dtu-common (spawn helpers), prism-dtu-{cyberint,armis,claroty,crowdstrike,nvd,pagerduty,slack,jira,threatintel} (apply helpers)"
subsystems: []
depends_on: [S-PERF-GATE-004]
blocks: []
behavioral_contracts: [BC-5.39.001]
# BC status: BC-5.39.001 is the delivery-quality contract (3-CLEAN convergence protocol).
# This story has no product behavioral contracts for the BehavioralClone trait's stop()
# semantics — the lifecycle contract is governed by ADR-002 Amendment #2 (TD-WV1-04) +
# the BehavioralClone docstring in prism-dtu-common/src/clone.rs.
# An architect/PO may wish to author a formal BC for start_on/stop lifetime guarantees
# (e.g., BC-2.06.NNN "BehavioralClone lifecycle: stop() must complete promptly for idle
# clones") — this story proceeds with BC-5.39.001 as the single delivery gate; the new
# BC can be authored in a follow-up if needed, then back-anchored to this story.
# BC-5.39.001 is already ACTIVE. POL-14 will be a NO-OP at merge.
verification_properties: []
assumption_validations: []
risk_mitigations: []
red_gate_tests: 1
estimated_days: "1.0"
---

# S-PERF-GATE-005: DTU Clone `stop()` Graceful Shutdown Wiring

Internal broadcast channel so `stop()` returns in < 500ms instead of always hitting the 5-second hard-abort path

## Narrative

As a Prism developer, I want `clone.stop()` to return promptly (< 500ms for an idle
clone) by wiring an internal graceful-shutdown channel when `start_on(shutdown=None)` is
used, so that every DTU integration test that calls `clone.stop()` stops paying a
mandatory 5-second hard-abort penalty — collapsing the dominant per-test latency cost
across all 9 production DTU clone crates.

## §Evidence

Phase timers on `test_BC_2_06_019_cyberint_ioc_value_without_ioc_type_withheld` (idle
machine, warm build, `--profile prepush`) — breakdown of total ~5.4 s:

| Phase | Time |
|-------|------|
| fixture catalog setup | 0.4 ms |
| `CyberintClone::new_with_scenario(...)` | 2 ms |
| `clone.start()` (bind + spawn server task) | 3 ms |
| `reqwest::Client::builder()...build()` | 380 ms |
| `client.get(...).send().await` (actual HTTP request) | 2 ms |
| **`clone.stop()`** | **~5.0 s** |
| **Total** | **~5.4 s** |

Everything before `stop()` is trivial. The stop() cost is the dominant term.

**Verification of root mechanism** (result of bisecting two hypotheses):

- Cutting both (a) the `select!` sleep from 5s to 100ms AND (b) the TLS
  `graceful_shutdown(Some(5s))` to `Some(100ms)`: test dropped from **5.40s → 0.49s** (~11×).
- Cutting only one dimension had no effect (earlier inconclusive experiment).
- A no-keepalive test client (`pool_max_idle_per_host(0)`) made NO difference (5.7s) —
  REJECTED: the cost is the server-never-completes mechanism, not a lingering client
  connection.

**Pattern is identical across all 9 production DTU clones** — confirmed by counting
`from_secs(5)` in each clone's `clone.rs`:

```
grep -c "from_secs(5)" crates/prism-dtu-{armis,claroty,crowdstrike,cyberint,nvd,pagerduty,slack,jira,threatintel}/src/clone.rs
```

Each returns 2 (one for TLS `graceful_shutdown(Some(5s))`, one for the `select!` timeout).

### §Measurement (post-implementation, clean machine, shutdown fix HEAD `38206c2d`)

| Metric | Before | After |
|--------|--------|-------|
| `clone.stop()` — idle (no in-flight request) | 5.002s | 0.019s |
| `clone.stop()` — after one request | 5.002s | 0.326s |
| `bc_2_06_019_scenario_stagemask` 3-test suite (cyberint) | ~49s (30s + 10s + 9s) | 49ms total |
| Full workspace `nextest --profile prepush` (4976 tests) | ~hours (DTU serialization) | 86.4s |
| Full `just check` | — | ~5:47 |

**S-PERF-GATE-004 cap=4 reassessment — RESOLVED (keep cap=4):**

| cap setting | Full workspace nextest | Test failures |
|-------------|----------------------|---------------|
| cap=4 (current) | 86.4s | 0 |
| cap=8 | 91.5s | 1 (flake under higher concurrency) |
| cap=16 | 97.5s | 0 |

Conclusion: cap=4 is optimal. Higher caps increase concurrency overhead and introduce
flakes without improving throughput. No change to S-PERF-GATE-004 — cap-reassessment
follow-up is RESOLVED.

Implementation: shared generic helper `prism_dtu_common::server::spawn_with_internal_shutdown`
wired into 9 clones; 23 constructor call sites updated; HEAD `38206c2d`.

## Background

### Root cause

`BehavioralClone::start()` (the default method on the trait) delegates to
`start_on(addr, None, None)`. In `start_on`'s HTTP path, when `shutdown` is `None`:

```rust
let handle = tokio::spawn(async move {
    let server = axum::serve(listener, router);
    if let Some(mut rx) = shutdown {
        let serve_future = server.with_graceful_shutdown(async move { let _ = rx.recv().await; });
        serve_future.await.expect("...");
    } else {
        // shutdown = None => NO with_graceful_shutdown future => server runs FOREVER
        server.await.expect("...");
    }
});
```

The server task spawned by the `else` branch has no completion condition — it runs until
hard-aborted. Then `stop()` is:

```rust
async fn stop(&mut self) -> anyhow::Result<()> {
    // TLS path: signal via axum_server::Handle (fires only if tls_handle is Some)
    #[cfg(feature = "tls")]
    if let Some(h) = self.tls_handle.take() {
        h.graceful_shutdown(Some(std::time::Duration::from_secs(5)));
    }

    // HTTP path: await OR timeout
    if let Some(mut handle) = self.server_handle.take() {
        tokio::select! {
            _ = &mut handle => { /* clean */ }
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                handle.abort();  // ← always fires on direct-start() path
            }
        }
    }
    Ok(())
}
```

Because the server task never self-completes, the `select!` **always** falls through to
the 5-second sleep arm, then hard-aborts. `stop()` is effectively "wait 5 seconds, then
kill" on the direct `start()` path. The graceful-drain branch (`handle => { }`) is
**dead code** on this path.

Every DTU test that calls `clone.stop()` (which is every test that uses
`clone.start()` — the standard non-harness path) pays this 5-second tax.

### The correct fix

1. **Add `internal_shutdown_tx: Option<tokio::sync::broadcast::Sender<()>>`** to each
   clone struct (alongside `server_handle` and `tls_handle`).

2. **In `start_on`, when `shutdown` is `None`**, create a fresh broadcast channel
   `tokio::sync::broadcast::channel::<()>(1)`, store the sender in
   `self.internal_shutdown_tx`, and wire the receiver into the graceful shutdown future:

   ```rust
   let (tx, rx) = tokio::sync::broadcast::channel::<()>(1);
   self.internal_shutdown_tx = Some(tx);
   let serve_future = axum::serve(listener, router)
       .with_graceful_shutdown(async move { let _ = rx.recv().await; });
   let handle = tokio::spawn(async move { serve_future.await.expect("..."); });
   self.server_handle = Some(handle);
   ```

   When `shutdown` is `Some` (the demo-harness path), keep using the provided receiver
   as-is — no change to harness behavior.

3. **In `stop()`**, fire `internal_shutdown_tx.send(())` BEFORE the `select!` so the
   server task completes its graceful-shutdown future immediately (no in-flight requests
   → instant):

   ```rust
   // Signal internal shutdown (direct-start() path).
   if let Some(tx) = self.internal_shutdown_tx.take() {
       let _ = tx.send(());
   }
   // Short safety timeout (crash-safety fallback only, NOT the common path).
   if let Some(mut handle) = self.server_handle.take() {
       tokio::select! {
           _ = &mut handle => { /* server completed gracefully — common case */ }
           _ = tokio::time::sleep(std::time::Duration::from_millis(250)) => {
               handle.abort();  // hard-abort only if server is hung
           }
       }
   }
   ```

4. **TLS path**: verify `axum_server::Handle::graceful_shutdown` completes promptly when
   idle — reduce or remove the 5s drain parameter OR also pair TLS with an internal
   sender. The `tls_handle.graceful_shutdown(Some(Duration::from_millis(250)))` call
   is sufficient for the TLS path since the `axum_server::Handle` IS the signal
   mechanism there.

5. **Factor shared HTTP-path logic into `prism-dtu-common`** — add a module
   `prism_dtu_common::server` (new file `crates/prism-dtu-common/src/server.rs`) with
   a helper function:

   ```rust
   /// Spawn an HTTP server with an internal graceful-shutdown channel.
   ///
   /// Returns `(JoinHandle<()>, Sender<()>)`. The caller stores the sender as
   /// `internal_shutdown_tx` and later calls `sender.send(())` from `stop()`.
   pub async fn spawn_with_internal_shutdown(
       listener: tokio::net::TcpListener,
       router: axum::Router,
       error_context: &'static str,
   ) -> anyhow::Result<(SocketAddr, tokio::task::JoinHandle<()>, tokio::sync::broadcast::Sender<()>)>
   ```

   Every clone's `start_on` HTTP path calls this helper instead of the inline logic.
   Fix once; all 9 clones inherit the correct behavior.

### Why demo-server is NOT affected

`prism-dtu-demo-server` already wires an external `shutdown_tx: broadcast::Sender<()>`,
subscribes receivers per-clone, and passes them to `start_on(addr, Some(rx), ...)`.
Its `MultiInstanceServers::shutdown()` fires the sender and the servers drain promptly.
The demo-server path already works correctly; no changes needed there.

### ADR-002 Amendment context

The `BehavioralClone::stop()` docstring references
`# TD-WV1-04-FU-001 — shutdown symmetry`. This story closes the final gap in that
amendment: the HTTP direct-start() path was documented as if the harness already sent
the broadcast signal before `stop()` is called — which is true for the harness path
(the harness fires the external sender first), but false for the direct `start()` path
(there IS no external sender). This story makes the HTTP path self-contained.

## Scope

Production Rust code in 10 crates. No config file or Justfile changes.

| File | Change type | Details |
|------|-------------|---------|
| `crates/prism-dtu-common/src/server.rs` | New file | `spawn_with_internal_shutdown` helper |
| `crates/prism-dtu-common/src/lib.rs` | Modify | `pub mod server;` export |
| `crates/prism-dtu-cyberint/src/clone.rs` | Modify | Add `internal_shutdown_tx` field; call helper in `start_on` HTTP path; fire tx in `stop()` |
| `crates/prism-dtu-armis/src/clone.rs` | Modify | Same pattern |
| `crates/prism-dtu-claroty/src/clone.rs` | Modify | Same pattern |
| `crates/prism-dtu-crowdstrike/src/clone.rs` | Modify | Same pattern |
| `crates/prism-dtu-nvd/src/clone.rs` | Modify | Same pattern |
| `crates/prism-dtu-pagerduty/src/clone.rs` | Modify | Same pattern |
| `crates/prism-dtu-slack/src/clone.rs` | Modify | Same pattern |
| `crates/prism-dtu-jira/src/clone.rs` | Modify | Same pattern |
| `crates/prism-dtu-threatintel/src/clone.rs` | Modify | Same pattern |

**NOT in scope:**

- `crates/prism-dtu-demo-server/` — already wires external shutdown correctly; no change
- `crates/prism-dtu-harness/` — harness always passes an external shutdown receiver; no change
- `crates/prism-dtu-common/src/clone.rs` (trait definition) — no signature change needed;
  note in story that an architect review of the trait's stop() postcondition doc is welcome
- `.config/nextest.toml` / `Justfile` — no change (S-PERF-GATE-004 cap=4 remains;
  see AC-6 note on follow-up reassessment)
- Any `.factory/` file — state-manager handles index registration post-delivery

## Acceptance Criteria

### AC-001 — `start_on(shutdown=None)` HTTP path wires internal shutdown channel (traces to BC-5.39.001 postcondition — delivery quality)

After the change, when `start_on(bind, None, ...)` is called on any of the 9 production
clone crates:

- The clone struct retains an `internal_shutdown_tx: Some(broadcast::Sender<()>)`.
- The server task's `axum::serve(...)` is wrapped with `.with_graceful_shutdown(...)`,
  using a receiver from that sender.
- The `else { server.await.expect("..."); }` bare-server branch is REMOVED from the
  `start_on` HTTP path in every clone.

Verifiable by code review: `grep -c 'internal_shutdown_tx' crates/prism-dtu-cyberint/src/clone.rs` returns ≥ 1.

### AC-002 — `stop()` fires internal sender before select (traces to BC-5.39.001 postcondition — delivery quality)

After the change, `stop()` in every affected clone:

1. Takes the `internal_shutdown_tx` out of `self.internal_shutdown_tx`.
2. Calls `tx.send(())` (ignores the `RecvError` result — no receivers = server already
   stopped, not an error).
3. THEN runs the `select!` with a SHORT safety timeout (≤ 250ms), not 5s.

Verifiable by code review: `grep -c 'internal_shutdown_tx.take()' crates/prism-dtu-cyberint/src/clone.rs` returns 1.
And: `grep -c 'from_secs(5)' crates/prism-dtu-cyberint/src/clone.rs` returns 0 for the HTTP path.

### AC-003 — Representative DTU test drops from ~5.4s to < 1s (traces to BC-5.39.001 postcondition — delivery quality)

```
cargo nextest run -p prism-dtu-cyberint \
  --test bc_2_06_019_ioc_stamping \
  -E 'test(test_BC_2_06_019_cyberint_ioc_value_without_ioc_type_withheld)' \
  --profile prepush
```

Expected wall-clock: < 1s (measured ~5.4s before fix; target ~ 0.5s after; previously
measured 0.49s when the select timeout was cut to 100ms in the diagnosis experiment).

### AC-004 — Graceful behavior preserved: in-flight request is drained before stop returns (traces to BC-5.39.001 postcondition — delivery quality)

A test (or code-review assertion) verifying that:

- A clone with one in-flight request (a long-running handler sleeping 50ms) does NOT
  cause `stop()` to return before the handler completes AND before the 250ms safety
  bound.
- No abandoned/leaked server tasks after `stop()` returns.

Acceptable verification: the 250ms safety timeout ensures that even a 50ms in-flight
handler completes within the safety window — the graceful-shutdown signal fires, axum
begins draining, the in-flight handler completes (~50ms), the server task exits, the
`select!` handle-done arm fires before the 250ms timer.

This AC is verified by the Red Gate test structure: the Red Gate test uses an **idle**
clone (no in-flight requests). A separate edge-case entry (EC-003) documents the
in-flight-request behavior.

### AC-005 — Shared helper in `prism-dtu-common::server`; no 9× divergent inline copy (traces to BC-5.39.001 postcondition — delivery quality)

The implementation strategy:

```
crates/prism-dtu-common/src/server.rs   ← new file with spawn_with_internal_shutdown
crates/prism-dtu-common/src/lib.rs      ← pub mod server;
```

Each of the 9 clone crates calls `prism_dtu_common::server::spawn_with_internal_shutdown(listener, router, "CloakName DTU server")` from its `start_on` HTTP path. The helper returns `(SocketAddr, JoinHandle<()>, broadcast::Sender<()>)`.

Verifiable: `grep -rn 'spawn_with_internal_shutdown' crates/prism-dtu-*/src/clone.rs | wc -l` returns 9 (one call site per clone crate).

### AC-006 — Full DTU suite re-measured; before/after wall-clock reported; S-PERF-GATE-004 cap reassessment noted (traces to BC-5.39.001 postcondition — delivery quality)

After delivery, run:

```
time cargo nextest run -p prism-dtu-cyberint -p prism-dtu-armis -p prism-dtu-crowdstrike \
  -p prism-dtu-claroty -p prism-dtu-nvd -p prism-dtu-pagerduty \
  -p prism-dtu-slack -p prism-dtu-jira -p prism-dtu-threatintel \
  --profile prepush
```

**Measured results (HEAD `38206c2d`, clean machine) — see §Evidence §Measurement table for full breakdown:**

| Metric | Before | After |
|--------|--------|-------|
| `clone.stop()` idle | 5.002s | 0.019s |
| `clone.stop()` after one request | 5.002s | 0.326s |
| cyberint scenario 3-test suite | ~49s | 49ms |
| Workspace nextest (4976 tests) | ~hours | 86.4s |
| `just check` | — | ~5:47 |

**S-PERF-GATE-004 cap=4 reassessment — RESOLVED (keep cap=4, no story change needed).**
cap=8 measured at 91.5s + 1 test flake; cap=16 at 97.5s. cap=4 is optimal.
Do NOT change nextest config — cap-reassessment follow-up is CLOSED.

### AC-007 — No bare un-graceful `axum::serve` in any clone's None branch; all 9 clones wired to shared helper (traces to BC-5.39.001 postcondition — delivery quality)

Two complementary verifications:

**1. Positive: all 9 clones use the shared helper**

```
rg -l 'spawn_with_internal_shutdown' crates/prism-dtu-*/src/clone.rs | wc -l
```

Returns `9` (one match per clone crate). This is the reliable positive invariant — the
helper is the only code path that wires the graceful-shutdown future for the HTTP
direct-start path, so its presence in all 9 clone files proves all 9 are wired.

**2. Negative: bare `axum::serve(...).await` without graceful-shutdown absent from every None branch**

```
rg 'server\.await\.expect\|axum::serve.*\.await\.expect' crates/prism-dtu-*/src/clone.rs
```

Returns zero matches. All remaining `axum::serve` invocations in clone files go through
`spawn_with_internal_shutdown`, which always wires `.with_graceful_shutdown(...)`.

**Why `server.await.expect` was unreliable (D1 justification):** Before the fix, 4 of 9
clones used a split `let result = server.await; result.expect(...)` form and claroty used
`make_service` — so a literal `server.await.expect` grep returned 5/9 matches even BEFORE
the fix, falsely appearing to show partial compliance. The two checks above are positive
invariants: either the helper is present in all 9 (count = 9), or it is not.

## Red Gate

**1 Red Gate test.** Name: `test_PERF_GATE_005_stop_completes_promptly_for_idle_clone`

Location: `crates/prism-dtu-cyberint/src/clone.rs` → `#[cfg(test)] mod tests` block

```rust
/// Red Gate test for S-PERF-GATE-005: stop() must complete in < 500ms for an idle
/// clone started via the direct start() path (shutdown=None internal channel).
///
/// BEFORE FIX: fails — stop() always hits the 5s select timeout and hard-aborts,
/// taking ~5.0s. The assertion fires because 5000ms >> 500ms.
///
/// AFTER FIX: passes — stop() fires the internal shutdown sender, the server task
/// completes its graceful-shutdown future immediately (no in-flight requests), the
/// select! handle-done arm fires in < 10ms, total < 500ms.
#[tokio::test]
async fn test_PERF_GATE_005_stop_completes_promptly_for_idle_clone() {
    let mut clone = CyberintClone::new();
    clone.start().await.expect("CyberintClone::start failed");

    let t = std::time::Instant::now();
    clone.stop().await.expect("CyberintClone::stop failed");
    let elapsed = t.elapsed();

    assert!(
        elapsed < std::time::Duration::from_millis(500),
        "stop() took {:?}, expected < 500ms; S-PERF-GATE-005 fix not applied or broken",
        elapsed
    );
}
```

**Red Gate validation**: Run before writing any production code:

```
cargo nextest run -p prism-dtu-cyberint \
  -E 'test(test_PERF_GATE_005_stop_completes_promptly_for_idle_clone)' \
  --profile prepush
```

Expected BEFORE fix: test FAILS with message `stop() took ~5s, expected < 500ms`.
Expected AFTER fix: test PASSES with stop() measured at < 50ms in practice.

## Behavioral Contracts

| BC | Title | Role in this story |
|----|-------|--------------------|
| BC-5.39.001 | 3-CLEAN convergence protocol | Delivery-quality gate — this story's own PR must pass 3-CLEAN before merge |

This story has no product behavioral contracts. The `stop()` latency guarantee is
currently governed by the doc comment on `BehavioralClone::stop()` in
`prism-dtu-common/src/clone.rs` and ADR-002 Amendment #2 (TD-WV1-04). An architect or
product-owner MAY choose to author a formal BC (e.g., BC-2.06.NNN "BehavioralClone
start/stop lifecycle timing") and back-anchor it to this story; if so, update this story's
`behavioral_contracts` frontmatter and add the AC↔BC traces before promoting to `ready`.

## Tasks

1. **Read** `crates/prism-dtu-common/src/clone.rs` to confirm current `start()` default
   implementation delegates to `start_on(addr, None, None)` and that `start_on`'s HTTP
   `else` branch has no graceful-shutdown future.

2. **Read** `crates/prism-dtu-common/src/lib.rs` to confirm the current module structure
   and where `pub mod server;` should be added.

3. **Read** `crates/prism-dtu-cyberint/src/clone.rs` (the representative clone) in full
   to understand the current struct fields (`server_handle`, `tls_handle`, `tls_active`,
   `bound_addr`, `admin_token`) and the exact `start_on` + `stop()` code.

4. **Write Red Gate test** in `crates/prism-dtu-cyberint/src/clone.rs`
   (`#[cfg(test)] mod tests` block): `test_PERF_GATE_005_stop_completes_promptly_for_idle_clone`.
   Run it — confirm it FAILS with `stop() took ~5s, expected < 500ms`.

5. **Write** `crates/prism-dtu-common/src/server.rs` with `spawn_with_internal_shutdown`:
   - Takes `listener: tokio::net::TcpListener`, `router: axum::Router`,
     `error_context: &'static str`
   - Returns `anyhow::Result<(SocketAddr, JoinHandle<()>, broadcast::Sender<()>)>`
   - Creates a broadcast channel, wires receiver into `with_graceful_shutdown`, spawns
     the server task, returns the addr + handle + sender.

6. **Edit** `crates/prism-dtu-common/src/lib.rs`: add `pub mod server;`.

7. **Edit** `crates/prism-dtu-cyberint/src/clone.rs`:
   - Add `internal_shutdown_tx: Option<tokio::sync::broadcast::Sender<()>>` field to
     `CyberintClone` struct.
   - Initialize to `None` in all `CyberintClone::new*` constructors.
   - In `start_on` HTTP path: replace inline bare-server with call to
     `prism_dtu_common::server::spawn_with_internal_shutdown(listener, router, "Cyberint DTU server")?`;
     store `internal_shutdown_tx = Some(sender)`.
   - In `stop()`: take + fire `internal_shutdown_tx` before the `select!`; change the
     sleep from `from_secs(5)` to `from_millis(250)` for both HTTP and TLS paths.
   - Run Red Gate test — confirm it PASSES.

8. **Edit** the remaining 8 clone crates (`armis`, `claroty`, `crowdstrike`, `nvd`,
   `pagerduty`, `slack`, `jira`, `threatintel`) following the identical pattern:
   - Add `internal_shutdown_tx` field
   - Call `prism_dtu_common::server::spawn_with_internal_shutdown` in `start_on` HTTP path
   - Fire tx + use 250ms timeout in `stop()`

9. **Verify** AC-001 through AC-005 grep commands return expected values.

10. **Run** the representative test timing for AC-003:
    ```
    cargo nextest run -p prism-dtu-cyberint \
      --test bc_2_06_019_ioc_stamping \
      -E 'test(test_BC_2_06_019_cyberint_ioc_value_without_ioc_type_withheld)' \
      --profile prepush
    ```
    Record wall-clock for AC-003 confirmation (expect < 1s).

11. **Run** full DTU suite timing for AC-006:
    ```
    time cargo nextest run -p prism-dtu-cyberint -p prism-dtu-armis \
      -p prism-dtu-crowdstrike -p prism-dtu-claroty -p prism-dtu-nvd \
      -p prism-dtu-pagerduty -p prism-dtu-slack -p prism-dtu-jira \
      -p prism-dtu-threatintel --profile prepush
    ```
    Record before/after wall-clock.

12. **Run** `just check` to confirm exit 0, all tests pass, `just check` < 5 min.

13. **Confirm** the ONLY modified files are the 11 listed in File Structure Requirements
    (no story-index changes — state-manager handles index registration).

## Token Budget Estimate

| Context component | Estimated tokens |
|-------------------|-----------------|
| This story spec (v1.1, ~430 lines) | ~5,500 |
| `crates/prism-dtu-common/src/clone.rs` (144 lines — read only) | ~1,600 |
| `crates/prism-dtu-common/src/lib.rs` (scan for module structure) | ~500 |
| `crates/prism-dtu-cyberint/src/clone.rs` (read + edit, ~490 lines) | ~5,500 |
| New `crates/prism-dtu-common/src/server.rs` (~50 lines) | ~600 |
| 8 remaining clone `clone.rs` files (edit only, ~400 lines avg each) | ~14,000 |
| `cargo nextest run` outputs (Red Gate + representative + full-suite) | ~2,000 |
| **Total** | **~28,700** |

Note: reading all 8 remaining clone files in full is unnecessary — they share the same
pattern as cyberint. The implementer MUST read the struct definition in each (first 80
lines) to confirm field names, then apply the identical edit pattern. Skimming (not
full-read) is sufficient for fields-and-constructors discovery.

## Previous Story Intelligence

### From S-PERF-GATE-001 (PR #204)

- The `#[cfg(test)] mod tests` block in production source files is the correct location
  for in-process Red Gate tests (vs. `tests/` integration test files which are
  process-per-test and more expensive).
- Red Gate must FAIL before writing production code — run the test, confirm the failure
  message shows the ~5s elapsed time. Do NOT write the production fix first.

### From S-PERF-GATE-002 (PR #206)

- The LazyLock/process-per-test lesson is NOT relevant here — this story adds a
  broadcast channel (not a shared static) per clone instance. Each test creates its own
  `CyberintClone::new()` with its own `internal_shutdown_tx`. No cross-test state.

### From S-PERF-GATE-004 (PR #209, develop@e3148007)

- S-PERF-GATE-004 is MERGED. The `dtu-cap = { max-threads = 4 }` constraint is active
  on `develop`. This story's implementation should branch off develop post-e3148007.
- The `dtu-cap` constraint serializes the 9 DTU packages to ≤4 concurrent nextest threads.
  With this story's fix, `stop()` overhead drops ~10× per test. The cap may no longer be
  necessary — see AC-006 note. Do NOT change the cap in this story.
- The probe-experiment result (cutting BOTH 5s timeout → 100ms: 5.40s → 0.49s) confirms
  the mechanism is correct. The 250ms safety timeout is MORE conservative than 100ms
  (more margin for slow CI machines) but still orders-of-magnitude faster than 5s.

### BehavioralClone trait — no signature change

The `BehavioralClone::start_on` and `stop` signatures on the TRAIT itself do NOT change.
This story adds `internal_shutdown_tx` as a private STRUCT field on each implementing
clone. The trait contract remains:
- `start_on(bind, shutdown=None, tls=None)` — unchanged signature
- `stop()` — same signature, but postcondition strengthened: returns in < 500ms for
  idle clones (previously: "eventually returns, usually after 5s")

### broadcast::channel vs oneshot::channel

A `tokio::sync::oneshot::Sender` would also work for a single send. A
`broadcast::Sender<()>` is used because `start_on` already uses
`broadcast::Receiver<()>` in the `Some(shutdown)` branch — staying with broadcast keeps
the same type throughout the clone's shutdown plumbing and matches the demo-server's
`shutdown_tx: broadcast::Sender<()>`. Use broadcast capacity = 1.

## Architecture Compliance Rules

Extracted from architecture sections and ADRs relevant to this story:

1. **ADR-022 (Arc-DI wiring)**: Not applicable — these are test-only instantiations.
   The `prism-dtu-*` crates are not wired via the production Arc-DI boot path.

2. **ADR-002 Amendment #2 (TD-WV1-04, shutdown symmetry)**: This story closes the HTTP
   direct-start() gap. After this story, HTTP and TLS paths are symmetric:
   - TLS: `axum_server::Handle::graceful_shutdown(250ms)` signals drain, then select 250ms.
   - HTTP: `internal_shutdown_tx.send(())` signals drain, then select 250ms.

3. **Single-workspace MSRV (rust-toolchain.toml)**: All 10 crates compile under the single
   pinned toolchain. No per-crate MSRV change.

4. **TD-VSDD-053 (single-commit-per-burst)**: The implementer must deliver this story's
   changes in a SINGLE feature branch PR. No multi-step "Stage 1/Stage 2" commits.

5. **No `--no-verify` hook bypass**: `just check` must pass normally before the PR.

6. **`#[non_exhaustive]` discipline**: No new pub-API surface types are introduced by this
   story (`spawn_with_internal_shutdown` returns concrete types). If a new struct is added
   to `prism-dtu-common::server`, add `#[non_exhaustive]` per project convention.

7. **No AI attribution in commits** per project git conventions (CLAUDE.md).

8. **Red Gate before production code** (BC-8.30.001 tdd_mode: strict): write the
   `test_PERF_GATE_005_stop_completes_promptly_for_idle_clone` test FIRST, confirm it
   FAILS, THEN write the production code.

9. **`.factory/` not modified by this story**: state-manager handles index registration.

10. **Architect review note (optional)**: The `BehavioralClone::stop()` docstring in
    `prism-dtu-common/src/clone.rs` says "Forcibly abort the server task via
    `JoinHandle::abort()`" which is now inaccurate after this fix (stop() no longer
    force-aborts on the common path). Update the doc comment inline as part of this
    story's implementation — this is implementer scope, not architect scope. A separate
    architect-authored BC for lifecycle timing is welcome but not required for `ready`.

## Library and Framework Requirements

This story uses existing dependencies already present in `prism-dtu-*` crates:

| Dependency | Version | Usage |
|------------|---------|-------|
| `tokio` | workspace pin (≥ 1.37) | `broadcast::channel`, `spawn`, `select!`, `sleep`, `time::Instant` |
| `axum` | workspace pin | `.with_graceful_shutdown(...)`, `serve` |
| `anyhow` | ≥ 1.0.103 (post-PR #209 constraint) | `anyhow::Result<...>` return type |
| `async-trait` | workspace pin | `#[async_trait]` on `BehavioralClone` impls |

No new dependencies. No version pins change.

## File Structure Requirements

| File | Change type | Details |
|------|-------------|---------|
| `crates/prism-dtu-common/src/server.rs` | New | `spawn_with_internal_shutdown` helper function |
| `crates/prism-dtu-common/src/lib.rs` | Modify | `pub mod server;` added |
| `crates/prism-dtu-cyberint/src/clone.rs` | Modify | `internal_shutdown_tx` field; helper call; stop() sender fire; 250ms timeout; Red Gate test |
| `crates/prism-dtu-armis/src/clone.rs` | Modify | Same pattern |
| `crates/prism-dtu-claroty/src/clone.rs` | Modify | Same pattern |
| `crates/prism-dtu-crowdstrike/src/clone.rs` | Modify | Same pattern |
| `crates/prism-dtu-nvd/src/clone.rs` | Modify | Same pattern |
| `crates/prism-dtu-pagerduty/src/clone.rs` | Modify | Same pattern |
| `crates/prism-dtu-slack/src/clone.rs` | Modify | Same pattern |
| `crates/prism-dtu-jira/src/clone.rs` | Modify | Same pattern |
| `crates/prism-dtu-threatintel/src/clone.rs` | Modify | Same pattern |

**Files explicitly excluded from this story:**

- `crates/prism-dtu-demo-server/` — already correct; no change
- `crates/prism-dtu-harness/` — uses external shutdown receiver; no change
- `crates/prism-dtu-common/src/clone.rs` — trait signature unchanged; doc-comment
  update in `stop()` IS in scope (implementer task 7 step for cyberint); the trait-level
  docstring update for `stop()` in `clone.rs` MAY also be done in-scope
- `.config/nextest.toml` — no change; dtu-cap=4 stays until AC-006 assessment
- Any `.factory/` file — state-manager handles STORY-INDEX registration post-delivery

## Scheduling Note

**S-PERF-GATE-004 ALREADY MERGED (PR #209, develop@e3148007). Hard dependency satisfied.**

The implementer must branch `feature/S-PERF-GATE-005` off develop HEAD e3148007 (or
later). No merge conflict risk on `clone.rs` files — S-PERF-GATE-004 only modified
`.config/nextest.toml`.

Correct branching order:
```
develop (after S-PERF-GATE-004 merge — e3148007)
  └── feature/S-PERF-GATE-005   ← branch from here
        ├── New: crates/prism-dtu-common/src/server.rs
        ├── Edit: crates/prism-dtu-common/src/lib.rs
        └── Edit: crates/prism-dtu-{cyberint,armis,claroty,crowdstrike,nvd,pagerduty,slack,jira,threatintel}/src/clone.rs
```

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `internal_shutdown_tx.send(())` called when all receivers have been dropped (server task already exited) | `broadcast::Sender::send` returns `Err(SendError)` — the `let _ = tx.send(())` discards this. The `select!` then immediately sees `handle` completed. No issue. |
| EC-002 | `stop()` called before `start()` (no server_handle, no internal_shutdown_tx) | Both are `None`. `stop()` takes `None` from `internal_shutdown_tx` (no-op), takes `None` from `server_handle` (no-op), returns `Ok(())`. Identical to current behavior. |
| EC-003 | `stop()` called with one in-flight request taking 50ms | `internal_shutdown_tx.send(())` fires. Axum begins graceful drain (stops accepting new connections, waits for in-flight to complete). The 50ms handler completes. Server task exits. `select!` handle-done arm fires in ~50ms. Total `stop()` < 250ms safety bound. No hard-abort needed. |
| EC-004 | `stop()` called with one in-flight request that HANGS (> 250ms) | The 250ms safety timeout fires, `handle.abort()` is called. The in-flight handler is killed. This is the safety-fallback scenario; it was the COMMON case before the fix (5s timeout); it should now be rare (only stuck/hung handlers). |
| EC-005 | TLS path: `axum_server::Handle::graceful_shutdown(Some(250ms))` — does idle TLS server complete in < 250ms? | Yes — an idle TLS server (no in-flight requests) drains immediately once the shutdown signal is sent. The `axum_server` handle triggers the server to stop accepting new connections and the task exits in < 10ms when idle. The 250ms is a conservative safety bound. |
| EC-006 | Two simultaneous `stop()` calls (race condition in tests) | Second call takes `None` from `internal_shutdown_tx` and `None` from `server_handle` (already taken by first call) — both no-ops. Returns `Ok(())`. No double-abort. |
| EC-007 | New `prism-dtu-*` clone added AFTER this story without applying the pattern | New clone using the old bare-server `else` branch will regress to ~5s stop() behavior. Run the AC-007 verifications: `rg -l 'spawn_with_internal_shutdown' crates/prism-dtu-*/src/clone.rs \| wc -l` must still return 9 (a new unwired clone drops the count below 9); the negative bare-server check also catches regressions. This is NOT enforced by a compile-fail gate (no structural issue), but is visible in Red Gate test failure if the new clone's tests call stop(). |

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.1 | 2026-06-30 | story-writer | AC-007 regression-lint corrected (D1): replaced unreliable `server.await.expect` grep with `spawn_with_internal_shutdown` coverage count (9/9) + bare-server absence check; §Evidence §Measurement added with measured results (idle stop 5.002s→0.019s, post-request 0.326s, cyberint suite 49s→49ms, workspace nextest 86.4s, `just check` ~5:47); AC-006 cap-reassessment follow-up RESOLVED (keep cap=4; cap=8 = 91.5s + 1 flake, cap=16 = 97.5s). |
| 1.0 | 2026-06-30 | story-writer | Initial draft |
