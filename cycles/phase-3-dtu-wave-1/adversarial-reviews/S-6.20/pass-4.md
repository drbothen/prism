---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-04-22T00:00:00Z
phase: 3
inputs:
  - .factory/policies.yaml
  - Cargo.toml
  - crates/prism-dtu-common/src/lib.rs
  - crates/prism-dtu-common/src/clone.rs
  - crates/prism-dtu-common/src/config.rs
  - crates/prism-dtu-crowdstrike/src/lib.rs
  - crates/prism-dtu-crowdstrike/src/clone.rs
  - crates/prism-dtu-claroty/src/clone.rs
  - crates/prism-dtu-cyberint/src/clone.rs
  - crates/prism-dtu-armis/src/clone.rs
  - crates/prism-dtu-threatintel/src/lib.rs
  - crates/prism-dtu-threatintel/src/clone.rs
  - crates/prism-dtu-nvd/src/clone.rs
input-hash: "7c56513"
traces_to: S-6.20-unified-demo-harness.md
pass: 4
previous_review: ../adversary-S-6.20-spec-review-pass3.md
cycle: phase-3-dtu-wave-1
target: "S-6.20 v1.3 @ e5a211f"
verdict: BLOCKED
---

# Adversarial Review: S-6.20 Unified Multi-Clone Demo Harness (Pass 4)

**Verdict: BLOCKED** — spec artifacts inaccessible to reviewer (process gap); substantive defects found in source-vs-spec drift.

## Input Availability (Level 2 Escalation)

The adversary could not read the S-6.20 v1.3 spec, ADR-002 + amendment, or ADR-003 because those artifacts live on `factory-artifacts` branch with no worktree mounted from `develop`. Source-code findings below are grounded; spec-text findings could not be produced.

**Artifacts NOT reviewed (inaccessible):**
- `.factory/stories/S-6.20-unified-demo-harness.md` (spec v1.3)
- `.factory/specs/architecture/decisions/ADR-002-l2-clone-template.md` (+ amendment)
- `.factory/specs/architecture/decisions/ADR-003-dtu-fidelity-scoping.md`

**Policy rubric partially applied** (from `.factory/policies.yaml` v1.2): POL-001 through POL-010 could not be fully evaluated without spec text. POL-004 (semantic_anchoring_integrity) and POL-010 (demo_evidence_story_scoped) partially evaluated via source inspection only.

## Finding ID Convention

Finding IDs use the format: `ADV-WV1-P04-<SEV>-<SEQ>`

- `ADV`: Fixed prefix identifying adversarial findings
- `WV1`: Cycle prefix for phase-3-dtu-wave-1
- `P04`: Pass 4
- `<SEV>`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`)
- `<SEQ>`: Three-digit sequence within the pass (e.g., `001`)

Prior pass findings referenced below retain their original IDs (`F-6.20-P03-*`).

## Part A — Fix Verification (pass >= 2 only)

Pass 3 required actions RQ-1 through RQ-4. Verification is **source-only** (spec v1.3 unreadable). Assessment based on develop @ e5a211f.

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| F-6.20-P03-H-001 (stop_all mechanism) | HIGH | UNRESOLVED | All 6 clones still spawn detached tasks with no `JoinHandle` stored. `server_handle` present only in crowdstrike/claroty. cyberint/armis/threatintel/nvd discard spawn handle at lines 88, 105, 71, 83 respectively. RQ-1 not implemented. |
| F-6.20-P03-H-002 (bind hardcode) | HIGH | UNRESOLVED | All 6 clones still hardcode `TcpListener::bind("127.0.0.1:0")`. `StubConfig` has no `bind`/`port` field per `config.rs:4-12`. RQ-2 not implemented. |
| F-6.20-P03-M-001 (apply_config loopback 403) | MEDIUM | UNRESOLVED | Cannot verify spec change (v1.3 unreadable). Source: `apply_config` signature unchanged. |
| F-6.20-P03-M-002 (EC-008 vs AC-5 contradiction) | MEDIUM | UNRESOLVED | Cannot verify spec change (v1.3 unreadable). |

## Part B — New Findings (or all findings for pass 1)

### CRITICAL

#### ADV-WV1-P04-CRIT-001: Task 14 "one-line update" structurally impossible — 4 of 6 clones lack `server_handle`

- **Severity:** CRITICAL
- **Category:** spec-fidelity
- **Location:** S-6.20 v1.3 Task 14; `crates/prism-dtu-cyberint/src/clone.rs:30-34`; `crates/prism-dtu-armis/src/clone.rs:30-34`; `crates/prism-dtu-threatintel/src/clone.rs:21-25`; `crates/prism-dtu-nvd/src/clone.rs:29-32`
- **Description:** S-6.20 v1.3 Task 14 describes adding `start_on(addr)` and `stop()` to `BehavioralClone` as a "one-line update" across six existing clone crates. This is structurally impossible for at least 4 of the 6 crates. Adding `stop()` requires a stored `JoinHandle` to call `.abort()`. Only crowdstrike and claroty have `server_handle: Option<JoinHandle<()>>` in their struct. cyberint, armis, threatintel, and nvd discard the spawn handle immediately and have no field to hold it.
- **Evidence:**
  - `prism-dtu-crowdstrike/src/clone.rs:25` — `server_handle: Option<JoinHandle<()>>` present
  - `prism-dtu-claroty/src/clone.rs:33` — `server_handle: Option<JoinHandle<()>>` present
  - `prism-dtu-cyberint/src/clone.rs:88` — `tokio::spawn(...)` result discarded; no handle field
  - `prism-dtu-armis/src/clone.rs:105` — `tokio::spawn(...)` result discarded; no handle field
  - `prism-dtu-threatintel/src/clone.rs:71` — `tokio::spawn(...)` result discarded; no handle field
  - `prism-dtu-nvd/src/clone.rs:83` — `tokio::spawn(...)` result discarded; no handle field
- **Proposed Fix:** Rescope Task 14 per-crate. Crates with `server_handle` (crowdstrike, claroty) require 1 trait-method body addition each. Crates without (cyberint, armis, threatintel, nvd) each require: add `server_handle: Option<JoinHandle<()>>` struct field, wire `start()` to store the handle, add `stop()` body calling `handle.abort()`. Estimate: 4–8 lines per crate, not 1.

#### ADV-WV1-P04-CRIT-002: S-6.20 references clone crates absent from workspace (`prism-dtu-ocsf`, `prism-dtu-osquery`)

- **Severity:** CRITICAL
- **Category:** missing-story
- **Location:** S-6.20 v1.3 Task 14 (inferred); `Cargo.toml:3-18`
- **Description:** S-6.20 v1.3 claims Task 14 applies to "six existing clone crates." The workspace (develop @ e5a211f) contains only: prism-dtu-common, prism-dtu-threatintel, prism-dtu-nvd, prism-dtu-cyberint, prism-dtu-crowdstrike, prism-dtu-claroty, prism-dtu-armis (7 members including common). Neither `prism-dtu-ocsf` nor `prism-dtu-osquery` is a workspace member. `prism-ocsf` exists but is the real OCSF library crate, not a DTU clone. Task 14 cannot modify crates that do not exist.
- **Evidence:** `Cargo.toml:3-18` — workspace members list; no `prism-dtu-ocsf` or `prism-dtu-osquery` entry.
- **Proposed Fix:** Reconcile S-6.20 v1.3 "six existing clone crates" claim with workspace reality. Either: (a) gate Task 14 on S-6.14/S-6.15 merges and document the merge dependency explicitly, or (b) scope Task 14 to the 6 present crates (crowdstrike, claroty, cyberint, armis, threatintel, nvd) and remove references to ocsf/osquery clones.

### HIGH

#### ADV-WV1-P04-HIGH-001: `Send + Sync + 'static` bounds on `BehavioralClone` + `&mut self` force harness-side mutex; conflicts with `await_holding_lock = "deny"`

- **Severity:** HIGH
- **Category:** concurrency
- **Location:** `crates/prism-dtu-common/src/clone.rs:11`; `Cargo.toml:29`
- **Description:** `BehavioralClone: Send + Sync + 'static` with `async fn start(&mut self)` means a harness holding `Box<dyn BehavioralClone>` and driving it concurrently must wrap it in `Arc<tokio::sync::Mutex<_>>`. Holding that mutex across an await point will trip the `await_holding_lock = "deny"` workspace lint. The spec does not specify the harness ownership model.
- **Evidence:** `clone.rs:11` — trait bounds; `Cargo.toml:29` — `await_holding_lock = "deny"`.
- **Proposed Fix:** Spec must specify harness ownership model explicitly: by-value (sequential), `Arc<Mutex<_>>` with lint suppression justification, or redesign to owned-by-task pattern.

#### ADV-WV1-P04-HIGH-002: `stop()` shutdown semantics undefined; all 6 current `start()` bodies use `.expect()` with no graceful shutdown hook

- **Severity:** HIGH
- **Category:** interface-gaps
- **Location:** All 6 clone `start()` implementations
- **Description:** All 6 `start()` impls call `axum::serve(listener, router).await.expect("server crashed")` with no `with_graceful_shutdown()`. Adding `stop()` to abort these requires a full per-clone axum shutdown refactor, not a trait method addition. The spec does not specify abort-vs-graceful semantics.
- **Evidence:** Pattern present in crowdstrike, claroty, cyberint, armis, threatintel, nvd `start()` bodies — none use `with_graceful_shutdown`.
- **Proposed Fix:** Spec must specify: abort (immediate, via `JoinHandle::abort()`) vs graceful (drain in-flight, via `with_graceful_shutdown`). Enumerate the per-crate refactor required in each case.

#### ADV-WV1-P04-HIGH-003: Adding `StubConfig.bind` is a breaking struct-literal change; `Default` impl requires update

- **Severity:** HIGH
- **Category:** interface-gaps
- **Location:** `crates/prism-dtu-common/src/config.rs:4-22`
- **Description:** `StubConfig` has 3 fields with a `Default` impl using record-update syntax. Adding `bind: Option<SocketAddr>` is a breaking change for all `StubConfig { field, .. }` struct-literal sites. The spec must enumerate callsites and specify the `None` semantic (e.g., `127.0.0.1:0`).
- **Evidence:** `config.rs:4-12` — 3-field struct; `config.rs:14-22` — Default impl.
- **Proposed Fix:** Spec must enumerate `StubConfig` construction callsites, define `bind: None` as "OS-assigned port," and require `Default` impl to set `bind: None`.

#### ADV-WV1-P04-HIGH-004: Port collision handling unspecified for `start_on(addr)`

- **Severity:** HIGH
- **Category:** missing-edge-cases
- **Location:** S-6.20 Task 14 / harness startup spec
- **Description:** All 6 clones currently use port 0 (OS-assigned). `start_on(addr)` changes to caller-specified addresses (17080–17085 per pass 3). The spec does not define EADDRINUSE recovery: retry policy, failure propagation, or partial-startup cleanup when one of the 6 clones fails to bind.
- **Evidence:** All 6 `start()` impls use `TcpListener::bind("127.0.0.1:0")` — port collision impossible today but becomes possible with fixed ports.
- **Proposed Fix:** Spec must define: port-selection strategy, retry limit, and behavior when clone N of 6 fails to bind (roll back already-started clones, or propagate error and leak).

#### ADV-WV1-P04-HIGH-005: Partial-startup failure cleanup unspecified; leaks N-1 listeners and tasks

- **Severity:** HIGH
- **Category:** missing-edge-cases
- **Location:** S-6.20 harness lifecycle spec; all 6 clone `start()` impls
- **Description:** Every `start()` spawns a detached `tokio::task`. No harness `Drop` implementation or `cleanup_started()` method exists. If clone 4-of-6 fails to start, the 3 already-started clones have live listeners and detached tasks with no cleanup path. The spec does not address partial-startup failure containment.
- **Evidence:** No harness cleanup pattern present in any reviewed crate source.
- **Proposed Fix:** Spec harness `Drop` semantics must specify: call `stop()` on all successfully-started clones when any `start()` fails, with an AC asserting no listener leaks on partial-startup failure.

### MEDIUM

#### ADV-WV1-P04-MED-001: `bound_addr(&self)` panics before `start()` — harness startup logging unsafe

- **Severity:** MEDIUM
- **Category:** missing-edge-cases
- **Location:** All 6 clone `bound_addr()` implementations
- **Description:** All 6 impls use `.expect("called before start()")` in `bound_addr()`. A harness that logs bound addresses for diagnostics before confirming each clone started will panic. Panics in harness startup are non-recoverable in a `#[tokio::test]` context.
- **Evidence:** Pattern confirmed in all 6 clone crate `bound_addr()` bodies.
- **Proposed Fix:** Change trait return type to `fn bound_addr(&self) -> Option<SocketAddr>` (returns `None` before start), or document the precondition as an AC and gate harness logging on start completion.

#### ADV-WV1-P04-MED-002: `configure()` serializes across all clones under `std::sync::Mutex` inside async context

- **Severity:** MEDIUM
- **Category:** concurrency
- **Location:** `crates/prism-dtu-threatintel/src/clone.rs:89-95`
- **Description:** `configure()` acquires a `std::sync::Mutex` inside an async function. The `await_holding_lock = "deny"` lint barely passes today due to careful drop ordering, but any refactor adding an await between lock acquisition and release will trigger a denied lint. The serialized-broadcast semantic is undocumented.
- **Evidence:** `threatintel/src/clone.rs:89-95` — `std::sync::Mutex::lock()` inside async `configure()`.
- **Proposed Fix:** Document the serialized-broadcast semantic explicitly, or redesign to spawn a per-clone configuration task (async-safe).

#### ADV-WV1-P04-MED-003: Clone constructors have asymmetric fallibility — breaks harness factory pattern

- **Severity:** MEDIUM
- **Category:** interface-gaps
- **Location:** crowdstrike `clone.rs:30-33`; claroty `clone.rs:38-44`; threatintel `clone.rs:29-34`; cyberint `clone.rs:37-50`; armis `clone.rs:39-51`; nvd `clone.rs:38-52`
- **Description:** Three clones (crowdstrike, claroty, threatintel) have infallible `new() -> Self` constructors. Three clones (cyberint, armis, nvd) have fallible `new() -> Result<Self>` constructors (due to `load_fixture_as` fixture loading). A harness factory function cannot uniformly construct all 6 without handling the asymmetry. The "one-line update" claim in Task 14 cannot unify this.
- **Evidence:** Constructor signatures as noted above across 6 crates.
- **Proposed Fix:** Spec must specify a `ClonePair` factory pattern, a `impl_new_fallible!` shim, or require all constructors to be fallible (`new() -> Result<Self>`).

#### ADV-WV1-P04-MED-004: `StubConfig.bind` field no-op unless all `start()` bodies are rewritten to read it

- **Severity:** MEDIUM
- **Category:** spec-fidelity
- **Location:** All 6 clone `start()` implementations; `crates/prism-dtu-common/src/config.rs`
- **Description:** Adding `StubConfig.bind` is meaningless unless each clone's `start()` body is updated to read `self.config.bind` instead of hardcoding `"127.0.0.1:0"`. cyberint and armis do not even store a `StubConfig`. The spec conflates two separate mechanisms (field addition vs. call-site rewrites) without enumerating the latter.
- **Evidence:** All 6 `start()` bodies hardcode `"127.0.0.1:0"`; cyberint and armis have no `config` field.
- **Proposed Fix:** Either drop `StubConfig.bind` (use `start_on(addr)` parameter exclusively), or specify the per-crate `start()` rewrite that reads `self.config.bind`.

#### ADV-WV1-P04-MED-005: ADR-003 unauthenticated-only scope possibly violated by Cyberint `/login` route and Crowdstrike OAuth feature flag

- **Severity:** MEDIUM
- **Category:** spec-fidelity
- **Location:** `crates/prism-dtu-cyberint/src/clone.rs:61`; `crates/prism-dtu-crowdstrike/Cargo.toml:71-73`; `crates/prism-dtu-common/src/config.rs:39-40`
- **Description:** ADR-003 (unread — spec inaccessible) likely scopes DTU clone fidelity to unauthenticated endpoints only. Cyberint exposes a `/login` route; Crowdstrike has an `ac_5_oauth` feature flag; `StubConfig` has `FailureMode::AuthReject`. S-6.20 v1.3 may need to explicitly justify exercising auth-adjacent routes, or confirm they are out of scope for the demo harness.
- **Confidence:** LOW — ADR-003 unread.
- **Evidence:** `cyberint/src/clone.rs:61` — `/login` route; `crowdstrike/Cargo.toml:71-73` — `ac_5_oauth` feature; `config.rs:39-40` — `FailureMode::AuthReject`.
- **Proposed Fix:** After mounting factory-artifacts worktree, verify S-6.20 v1.3 exercises only unauthenticated endpoints or provides justification for auth use against ADR-003.

### LOW

#### ADV-WV1-P04-LOW-001: Crowdstrike `start()` holds `std::sync::Mutex` before `TcpListener::bind().await` — fragile lint boundary

- **Severity:** LOW
- **Category:** concurrency
- **Location:** `crates/prism-dtu-crowdstrike/src/clone.rs:64-75`
- **Description:** Crowdstrike `start()` acquires a `std::sync::Mutex` before an async `TcpListener::bind()` call. Correct today due to drop ordering, but fragile against any refactor that adds an await between lock and drop.
- **Evidence:** `crowdstrike/src/clone.rs:64-75`.
- **Proposed Fix:** Document the invariant, or restructure to release the lock before the first await point.

#### ADV-WV1-P04-LOW-002: `prism-dtu-common/src/lib.rs:6` doc claims "All 13 per-surface DTU crates" but workspace has 6

- **Severity:** LOW
- **Category:** ambiguous-language
- **Location:** `crates/prism-dtu-common/src/lib.rs:6`
- **Description:** The lib.rs doc comment claims "All 13 per-surface DTU crates (S-6.07–S-6.19)" but the workspace currently has 6 DTU clone crates. This will mislead implementers and reviewers.
- **Evidence:** `lib.rs:6` vs `Cargo.toml:11-17`.
- **Proposed Fix:** Update doc comment to reflect current workspace membership, or mark as "target count" with a note that crates land incrementally.

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 2 |
| HIGH | 5 |
| MEDIUM | 5 |
| LOW | 2 |

**Overall Assessment:** block
**Convergence:** findings remain — iterate
**Readiness:** requires revision; spec v1.3 must be updated to v1.4 before Pass 5

### Observations

| # | Observation |
|---|-------------|
| O1 | `StubConfig` has `Debug+Clone` derives; a new `bind` field preserves both. No blocker on derive side. |
| O2 | cyberint/armis/nvd `new() -> Result<Self>` pattern (fixture-loading) will likely apply to future OCSF/osquery clones too — worth establishing a uniform factory convention now. |
| O3 | `await_holding_lock = "deny"` workspace lint will catch naïve harness `Arc<std::sync::Mutex<dyn BehavioralClone>>` immediately. |

### To Unblock Pass 5

1. Mount factory-artifacts worktree alongside develop: `git worktree add .worktrees/factory-artifacts factory-artifacts` — re-run Pass 5 with spec in view.
2. S-6.20 v1.3 → v1.4 must address:
   - Resize Task 14 from "one-line" to per-crate delta enumeration (CRIT-001)
   - Reconcile "six crates" with workspace reality; document merge dependency on S-6.14/S-6.15 (CRIT-002)
   - Specify harness ownership model: by-value vs `Arc<Mutex<_>>` (HIGH-001)
   - Specify stop() shutdown semantics: abort vs graceful drain (HIGH-002)
   - Enumerate `StubConfig.bind` migration callsites; define `None` semantic (HIGH-003)
   - Define port-collision handling and EADDRINUSE recovery (HIGH-004)
   - Specify harness `Drop` cleanup for partial-startup failure (HIGH-005)
   - Change `bound_addr()` return to `Option<SocketAddr>` or document precondition (MED-001)
   - Add `ClonePair` factory or unify constructor fallibility (MED-003)
   - Clarify `StubConfig.bind` vs `start_on(addr)` role; enumerate `start()` rewrites (MED-004)

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 4 |
| **New findings** | 8 (CRIT-001, CRIT-002, HIGH-001, HIGH-002, HIGH-003, MED-001, MED-002, MED-003) |
| **Duplicate/variant findings** | 6 (HIGH-004, HIGH-005 are EC-class overlaps; MED-004 carries H-002; MED-005 is mandate-driven; LOW-001, LOW-002 are minor clarity variants) |
| **Novelty score** | 8 / (8 + 6) = 0.57 |
| **Median severity** | 3.5 (CRIT=5×2, HIGH=4×5, MED=3×5, LOW=2×2; median of 14 values = 3.5) |
| **Trajectory** | 10→8→6→14 (REGRESSION — spec v1.3 introduced new scope; source access in Pass 4 uncovered structural defects) |
| **Verdict** | FINDINGS_REMAIN |
