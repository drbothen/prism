---
document_type: gate-step-report
gate_step: c
gate_step_name: code-review
cycle: wave-3-multi-tenant
gate: wave-3-integration-gate
scope: 6696e374^..a3bd5a0f (full Wave 3, 40 commits, 616 files, +61891/-522)
reviewer: vsdd-factory:code-reviewer
develop_sha: a3bd5a0f
date: 2026-05-01
phase: 3
wave: 3
step: c
verdict: APPROVE_WITH_CONCERNS
total_findings: 9
high: 2
medium: 4
low: 3
---

# Wave 3 Integration Gate — Gate Step C: Code Review

**Scope:** `6696e374^..a3bd5a0f` (Wave 3 full diff — 40 commits, 616 files, +61,891/-522 lines)
**Reviewer:** vsdd-factory:code-reviewer (Sonnet 4.6 — independent of adversary)
**Date:** 2026-05-01
**Verdict:** APPROVE_WITH_CONCERNS — 9 findings (2 HIGH, 4 MEDIUM, 3 LOW)

---

## Summary

Wave 3 is a substantial and well-structured delivery. The multi-tenant foundation
(`OrgId`, `OrgSlug`, `OrgRegistry`) is idiomatic Rust with clean invariant encoding.
The `prism-customer-config` crate is a highlight: multi-error collection, layered
validation pass order, `deny_unknown_fields` on all four serde structs, and explicit
error codes matching BC-3.3.003/004 are all correctly implemented. The DTU harness
isolation modes show strong architectural clarity. Two HIGH findings require
resolution before the wave can be declared fully converged — both are behavioral
gaps with observable consequences in test execution. The MEDIUM and LOW findings
are improvement opportunities.

---

## HIGH Findings

### CR-001: `with_failure(slug, DtuType, mode)` silently discards the DtuType argument

- **Severity:** HIGH
- **Category:** spec-fidelity
- **Location:** `crates/prism-dtu-harness/src/builder.rs:233-251`
- **BC Reference:** BC-3.6.001 (per-org failure injection, postcondition 1)
- **Description:** `HarnessBuilder::with_failure` accepts `(slug, dtu_type, mode)` — callers
  expect the failure to be scoped to a specific `(slug, DtuType)` pair. However, when the
  slug resolves to an existing `CustomerSpec`, the method stores `mode` in
  `spec.initial_failure: Option<FailureMode>` — a single field shared across all DtuTypes
  for that org. In `build()` Phase 4 (line 485), the failure is then injected for every
  `dtu_type in &spec.dtu_types`, ignoring which specific type was originally requested.
  The `dtu_type` argument is silently dropped on the immediate-resolution path. On the
  deferred path (lines 248-250), `_dtu_type` is stored in `pending_failures` but the
  resolution loop at line 296 extracts only the slug for the existence check and does not
  propagate the dtu_type into `initial_failure` either.
- **Evidence:**
  ```rust
  // builder.rs:233-251
  pub fn with_failure(mut self, slug: &str, dtu_type: DtuType, mode: FailureMode) -> Self {
      if let Some(existing) = self.customers.iter_mut().find(|s| ...) {
          existing.initial_failure = if matches!(mode, FailureMode::None) {
              None
          } else {
              Some(mode) // dtu_type argument never stored or used here
          };
      } else {
          self.pending_failures.push((slug.to_owned(), dtu_type, mode));
      }
      self
  }
  // builder.rs:483-488
  if let Some(ref mode) = spec.initial_failure {
      for &dtu_type in &spec.dtu_types { // injects into ALL types, not the requested one
          harness.inject_failure(slug, dtu_type, mode.clone()).await?;
      }
  }
  ```
- **Proposed Fix:** Change `CustomerSpec.initial_failure` from `Option<FailureMode>` to
  `HashMap<DtuType, FailureMode>` (or `Vec<(DtuType, FailureMode)>`). In `with_failure`,
  insert/overwrite only the entry for the specified `DtuType`. In Phase 4, iterate over
  this map rather than `spec.dtu_types`. This gives callers per-type failure scoping as
  the API signature promises.

---

### CR-002: `Harness::drop` aborts all tasks without the spec-promised 5-second graceful window

- **Severity:** HIGH
- **Category:** spec-fidelity
- **Location:** `crates/prism-dtu-harness/src/harness.rs:348-370`
- **BC Reference:** BC-3.5.001 EC-004 ("waits up to 5s for graceful exit")
- **Description:** The doc comment at line 19 of `harness.rs` states: "waits up to 5s for
  graceful exit, then calls `handle.abort()` on any that have not exited within the grace
  period (BC-3.5.001 EC-004)." The actual `Drop` implementation does not wait at all — it
  sends shutdown signals and then immediately calls `handle.abort()` on every task. In an
  async runtime this is correct for the happy path (Tokio tasks yield to the runtime on
  the next `.await` point), but the doc comment is contractually misleading and the
  implementation does not give the server tasks a grace period in which to flush in-flight
  requests or write final state. Any test that relies on the "5-second grace" contract
  will not receive it.
- **Evidence:**
  ```rust
  fn drop(&mut self) {
      for (_key, sender) in self.shutdown_senders.drain() {
          let _ = sender.send(()); // signal graceful shutdown
      }
      // IMMEDIATELY aborts — no 5-second wait
      for (_key, handle) in self.task_handles.drain() {
          handle.abort();
      }
  }
  ```
  `harness.rs:19` claims: "waits up to 5s for graceful exit" — this text is false.
- **Proposed Fix:** Two acceptable resolutions. Preferred: in `Drop`, send the shutdown
  signal and then do NOT abort immediately; axum's `with_graceful_shutdown` already
  handles orderly drain when the shutdown future resolves, so removing the abort calls
  is sufficient and the claim becomes accurate. Alternative: keep the abort but update
  the doc comment to read "sends shutdown signal, then aborts immediately (in drop context
  async wait is not possible)" — this is honest but removes a spec-promised contract.
  The doc claim "5s grace period" also appears in the harness test at
  `tests/logical_isolation_test.rs:890` which gates drop with a 5s timeout; ensure
  whatever is chosen is consistent.

---

## MEDIUM Findings

### CR-003: `OrgSlug` regex pattern not validated in `prism-customer-config` validator

- **Severity:** MEDIUM
- **Category:** spec-fidelity
- **Location:** `crates/prism-customer-config/src/validator.rs:429-435` (`validate_structural`)
- **BC Reference:** BC-3.3.004 R-CUST-002
- **Description:** R-CUST-002 checks only that `config.org_slug` equals the filename stem
  (case-sensitive). It does not validate the slug against the `^[a-zA-Z0-9_-]{1,64}$`
  pattern defined in `prism_core::tenant::ORG_SLUG_PATTERN`. A customer config file named
  `my org.toml` with `org_slug = "my org"` would pass R-CUST-002 (slug matches stem) but
  then `boot.rs:107` would call `OrgSlug::new("my org")` which returns an `Invalid`
  variant — and `OrgSlug::new` returns the `Invalid` state silently; any downstream call
  to `.as_str()` would panic. The gap also means a customer slug containing spaces or
  special characters would pass config validation but break at boot registration. Since
  `load_and_validate` is supposed to validate all invariants before registration, this is
  a spec-fidelity gap.
- **Evidence:** `validator.rs:429-435` — R-CUST-002 check does not include regex validation.
  `boot.rs:107` — `OrgSlug::new(&config.org_slug)` can return `Invalid` if the slug
  contains non-`[a-zA-Z0-9_-]` characters (including spaces, which are legal in
  filesystem stems on macOS/Linux). `tenant.rs:84-86` — `new_unchecked` exists but is
  only for test code; the production boot path uses `new` which is fallible.
- **Proposed Fix:** In `validate_structural`, after the R-CUST-002 stem check, add a
  pattern check: `if OrgSlug::new(&config.org_slug).is_err() { errors.push(...) }`.
  Define a new error code `E-CFG-018: InvalidOrgSlugPattern` for this condition. This
  ensures that `boot_org_registry` can safely call `OrgSlug::new(...)` and `.as_str()`
  without a risk of panic.

---

### CR-004: `start_clone` uses sequential if-chains instead of exhaustive dispatch

- **Severity:** MEDIUM
- **Category:** pattern-consistency
- **Location:** `crates/prism-dtu-harness/src/clone_server.rs:535-598`
- **BC Reference:** ADR-011 §2.2 (dispatch on DtuType)
- **Description:** `start_clone` dispatches on `DtuType` using two sequential `if dtu_type == X`
  guards rather than a `match`. The `build_router_for_type` helper at line 468 uses a
  proper `match`, but `start_clone` does not, creating an inconsistency. If a new
  DtuType is added and only `build_router_for_type` is updated, `start_clone` will silently
  fall through to the generic stub router without a compile-time warning. The `_ => build_router(state)` arm in `build_router_for_type` and the implicit fallthrough in `start_clone`
  are not the same — Claroty and Armis have dedicated startup functions in `start_clone`
  but not in `build_router_for_type`, and CrowdStrike/Cyberint are in `builder.rs` but
  not in `start_clone`. The dispatch is fragmented across three sites with no exhaustiveness
  guarantee.
- **Evidence:**
  ```rust
  // clone_server.rs:535-552
  if dtu_type == DtuType::Armis {
      return start_armis_clone(...).await;
  }
  if dtu_type == DtuType::Claroty {
      return start_claroty_clone(...).await;
  }
  // Falls through to generic stub for CrowdStrike, Cyberint — but
  // builder.rs dispatches those separately at line 414-421
  ```
- **Proposed Fix:** Consolidate the dispatch into a single `match dtu_type` in
  `start_clone` (or in a new `start_clone_for_type` helper) that exhaustively lists
  all variants, calling the appropriate specialized startup function for each. The `_`
  arm becomes the explicit generic stub path. This makes it a compile-time error to
  add a new `DtuType` without updating the dispatch.

---

### CR-005: `validate_all` returns `(configs, errors)` with partial configs when cross-file duplicates exist

- **Severity:** MEDIUM
- **Category:** code-quality
- **Location:** `crates/prism-customer-config/src/validator.rs:116-198`
- **BC Reference:** BC-3.3.004 Invariant 1 ("validate-before-register ordering")
- **Description:** `validate_all` appends cross-file duplicate errors (`E-CFG-011`,
  `E-CFG-012`) to `all_errors` after the per-file loop. However, `valid_configs` is built
  only during the per-file loop — when two files pass per-file validation but have duplicate
  `org_id`s, both are added to `valid_configs` before the cross-file check catches the
  collision. The caller (`load_and_validate` in `lib.rs`) returns `Err(errors)` when
  errors are non-empty, so the configs are discarded — but the tuple return type
  `(Vec<CustomerConfig>, Vec<ConfigError>)` implies partial configs are useful. If any
  downstream caller calls `validate_all` directly (bypassing `load_and_validate`) and
  uses the configs without checking errors, they receive duplicate-id configs that would
  then fail at `OrgRegistry::register`. The `boot.rs` path via `load_and_validate` is
  safe, but the public `validate_all` export creates a usability trap.
- **Evidence:** `validator.rs:116` — `validate_all` is `pub`; `lib.rs:47` — `load_and_validate`
  is the safe wrapper; `validator.rs:142-144` — configs are added to `valid_configs`
  before cross-file checks at line 148-196.
- **Proposed Fix:** Either (a) make `validate_all` crate-private (`pub(crate)`) since
  `load_and_validate` is the documented public entry point, or (b) filter `valid_configs`
  to remove entries whose `org_id` or `org_slug` triggered a duplicate error before
  returning. Option (a) is simpler.

---

### CR-006: `poll_test_hook` spins at 10ms interval with no backoff or cancellation token

- **Severity:** MEDIUM
- **Category:** performance
- **Location:** `crates/prism-dtu-harness/src/clone_server.rs:783-816`
- **BC Reference:** S-3.3.03 (startup budget), S-3.5.01 (crate layout)
- **Description:** `poll_test_hook` runs an infinite loop sleeping 10ms per iteration,
  checking `test_hook_signal` each time. For a 12-clone harness this spawns 12 concurrent
  polling loops consuming approximately 12 wake-ups every 10ms = 1,200 wake-ups/second
  while the harness is live (typically the entire test duration). For short CI tests this
  is negligible, but for longer integration tests or high-clone-count scenarios it is
  unnecessary overhead. The loop has no cancellation path — it only terminates when a
  signal fires. Since the test hook is a test-path-only feature (the vast majority of
  tests never trigger it), a less aggressive approach is preferable.
- **Evidence:** `clone_server.rs:786-788`:
  ```rust
  loop {
      tokio::time::sleep(std::time::Duration::from_millis(10)).await;
      let signal = state.test_hook_signal.lock()...
  ```
- **Proposed Fix:** Replace the Mutex-polled signal with a `tokio::sync::Notify` or a
  `watch::channel`; the hook handler notifies the watcher, and `poll_test_hook` awaits
  the notification instead of spinning. This eliminates the periodic wake-up entirely.
  As a lower-cost fix without the refactor: increase the sleep to 50ms (still responsive
  for test use) and add a comment explaining the trade-off.

---

## LOW Findings

### CR-007: `archetype` and `scale` override fields in `CustomerSpec` are declared but not wired

- **Severity:** LOW
- **Category:** maintainability
- **Location:** `crates/prism-dtu-harness/src/types.rs:188-206`
- **BC Reference:** S-3.3.05 (per-test override fields)
- **Description:** `CustomerSpec.archetype: Option<Archetype>` and
  `CustomerSpec.scale: Option<f64>` are documented with `# TODO (implementer)` comments
  and are stored but never read in the builder's `build()` path. Only `seed_override`
  and `initial_failure` are actually wired (lines 332 and 483 in `builder.rs`). The
  presence of these declared-but-dead fields creates a misleading public API: a test
  author who sets `spec.archetype = Some(Archetype::CompromisedEndpoint)` will get no
  observable effect with no error. The TODO comments are honest, but the fields are
  exported in the public `CustomerSpec` struct.
- **Evidence:** `types.rs:189-206` — `archetype` and `scale` doc comments both contain
  `# TODO (implementer)` sections. No reference to these fields appears in `builder.rs`
  beyond the doc examples.
- **Proposed Fix:** If these are intentionally deferred to a later story, annotate
  the struct fields with `#[doc(hidden)]` or gate them behind a
  `cfg(feature = "fixture-gen")` guard so that they are not silently accepted in stable
  API usage. Alternatively, add a compile-time or runtime check in `build()` that returns
  an error if `archetype` or `scale` is `Some(_)` while the wiring is absent.

---

### CR-008: Placeholder `CloneState` instances carry misleading `org_slug` strings

- **Severity:** LOW
- **Category:** maintainability
- **Location:** `crates/prism-dtu-harness/src/clone_server.rs:630-636`, `720-725`
- **BC Reference:** ADR-011 §2.2
- **Description:** `start_armis_clone` creates a `placeholder_state` with
  `org_slug = "__armis-placeholder-{org_slug}__"` (line 632) and `start_claroty_clone`
  creates a `hook_state` with `org_slug = "__claroty-hook__"` (line 721). These
  placeholder `CloneState` instances are stored in `StartedClone.state` to satisfy
  the struct contract. If a caller or future developer accesses `started_clone.state`
  and reads `org_slug` assuming it reflects the real org, they will get a sentinel
  string that could leak into device IDs, log lines, or error messages. The comment
  explains the intent but the pattern is fragile.
- **Evidence:**
  ```rust
  // clone_server.rs:631-636
  let placeholder_state = Arc::new(CloneState::new(
      format!("__armis-placeholder-{org_slug}__"), // sentinel leaks if read
      0,
      DtuType::Armis,
      admin_token.clone(),
  ));
  ```
- **Proposed Fix:** Change `StartedClone.state` to `Option<Arc<CloneState>>`. The
  `Armis` and `Claroty` paths return `None`; the generic path returns `Some(state)`.
  Update `builder.rs` accesses to handle the `None` case. This eliminates the sentinel
  sentinel entirely. As a lighter-weight alternative, add a `is_placeholder: bool` field
  to `CloneState` and assert in debug builds that `is_placeholder == false` before any
  use of `org_slug`.

---

### CR-009: Startup budget assertion is a wall-clock assertion susceptible to CI noise

- **Severity:** LOW
- **Category:** maintainability
- **Location:** `crates/prism-dtu-harness/tests/logical_isolation_test.rs:323-341`
- **BC Reference:** BC-3.5.001 postcondition 5 (AC-005, D-058)
- **Description:** `test_BC_3_5_001_twelve_clone_startup_under_200ms` measures
  `start.elapsed().as_millis() < 200` as a hard assertion. The burst-log notes this
  test "fails locally under high system load" — and correctly so, because wall-clock
  `Instant` measurement on a shared CI runner with OS scheduling jitter can easily
  exceed 200ms even for a fast 12-task parallel bind. The 200ms budget is an architectural
  intent (D-058), not a millisecond SLA that is reliably verifiable in all CI environments.
  MacOS and Windows CI runners in particular show higher variance than Linux.
- **Evidence:** `logical_isolation_test.rs:336-340`:
  ```rust
  assert!(
      elapsed.as_millis() < 200,
      "12-clone harness build took {}ms; must complete in < 200ms (AC-005; D-058)",
      elapsed.as_millis()
  );
  ```
  CI matrix includes `macos-latest`, `macos-15-intel`, `windows-latest` — all have
  unpredictable scheduling jitter on shared runners.
- **Proposed Fix:** Two options. Option A (preferred): change the assertion to a soft
  `#[ignore]` test that runs only on a dedicated performance tier, and add a PROPTEST
  or benchmark variant that proves the implementation is parallel (i.e., total time <
  sum of individual startup times). Option B: raise the budget to 1000ms for this
  specific test, with a comment explaining that the 200ms figure is an architectural
  target, not a CI timing SLA. The internal harness timeout at `builder.rs:427`
  (the actual enforcement) remains at 200ms regardless of this test's threshold.

---

## Positive Observations (Non-Finding)

**OrgRegistry design is idiomatic and correct.** `bimap::BiMap` wrapped in
`std::sync::RwLock`, private `inner` field, idempotent re-registration, and
separate `SlugConflict`/`IdConflict` error variants with operator-facing `Display` are
all well-executed. The `OrgRegistry` doc comment accurately describes the hot-path
invariant (read-only after startup). No finding here.

**`prism-customer-config` error design is exemplary.** 21 typed `ConfigError` variants
with E-CFG-NNN codes, `deny_unknown_fields` on all four serde structs, credential heuristic
scanning that never includes the field value in error output, and the validate-all-then-
register-all boot protocol are all correct implementations of the BC-3.3.003/004 contracts.
The `sanitize_error_message` redaction function for TOML parse snippets is thoughtful.

**DTU harness clones are appropriately self-contained.** Each clone module (`claroty.rs`,
`armis.rs`, `cyberint.rs`, `crowdstrike.rs`, `slack.rs`) correctly uses `include_str!` for
fixture data and does not import from the corresponding production DTU crate — this avoids
circular dev-dependency chains and is consistent across all five S-3.4.* migrations.

**`AuditEntry` org tagging is complete.** `org_id: OrgId` and `org_slug: OrgSlug` are
non-`Option` fields with `static_assertions::assert_fields!` enforcing their presence at
compile time. `aql_hash` uses `sha2::Sha256` for deterministic cross-restart fingerprinting.
All three fields are properly threaded through `AuditEntry::new`. No finding here.

**CI matrix design is coherent.** Per-platform `PROPTEST_CASES` tiering (linux-gnu=1000,
others=256), `mold` linker on Linux only via `RUSTFLAGS`, `taiki-e/install-action` for
prebuilt `cargo-nextest`, and correct JUnit artifact upload are all well-implemented.

---

## Convergence Verdict

`findings remain — iterate`

Two HIGH findings (CR-001, CR-002) require fixes before this wave can be declared
converged. CR-001 (`with_failure` discards DtuType) is a behavioral bug with observable
test consequences. CR-002 (`Drop` does not implement its documented 5s grace period)
is a contract/documentation misalignment that must be resolved in one of the two
directions described. The four MEDIUM findings are improvements; CR-003 has a potential
panic path and should be addressed concurrently with the HIGH fixes. CR-004, CR-005,
CR-006 can follow in a subsequent pass. LOW findings (CR-007, CR-008, CR-009) may be
deferred to Wave 4.
