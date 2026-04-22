---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-04-22T21:00:00Z
phase: 3
inputs:
  - .factory/stories/S-6.20-dtu-demo-server.md (v1.2)
  - .factory/cycles/phase-3-dtu-wave-1/adversary-S-6.20-spec-review.md (Pass 1)
  - .factory/cycles/phase-3-dtu-wave-1/adversary-S-6.20-spec-review-pass2.md (Pass 2)
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/architecture/decisions/ADR-002-l2-dtu-clone-template.md
  - .factory/specs/architecture/decisions/ADR-003-dtu-reset-lookup-and-fidelity-auth.md
  - .factory/specs/architecture/dtu-assessment.md
  - crates/prism-dtu-common/src/clone.rs
  - crates/prism-dtu-common/src/config.rs
  - crates/prism-dtu-cyberint/src/clone.rs
  - crates/prism-dtu-crowdstrike/src/clone.rs
  - crates/prism-dtu-claroty/src/clone.rs
  - crates/prism-dtu-armis/src/clone.rs
  - crates/prism-dtu-nvd/src/clone.rs
  - crates/prism-dtu-threatintel/src/clone.rs
input-hash: ""
traces_to: S-6.20-dtu-demo-server.md
pass: 3
previous_review: adversary-S-6.20-spec-review-pass2.md
cycle: phase-3-dtu-wave-1
recommendation: CONDITIONAL
---

# S-6.20 Adversarial Review — Pass 3

## Finding ID Convention

Finding IDs use the format: `ADV-WV1-P03-<SEV>-<SEQ>`

- `ADV`: Fixed prefix identifying adversarial findings
- `WV1`: Cycle prefix for phase-3-dtu-wave-1
- `P03`: Pass 3
- `<SEV>`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`)
- `<SEQ>`: Three-digit sequence within the pass (e.g., `001`)

Pass 1 findings retain their original IDs (`F-6.20-NN`). Pass 2 findings use
`F-6.20-P02-*` shorthand. Pass 3 new findings use `ADV-WV1-P03-*` (aliased as
`F-6.20-P03-*` for cross-reference continuity in this document).

## Part A — Fix Verification (pass >= 2 only)

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| F-6.20-P02-H-001 (credential_ref) | HIGH | RESOLVED | v1.2 adds `credential_ref: Option<CredentialRef>` to StubConfig; wired through `apply_config()`. No residual gap. |
| F-6.20-12 (stop_all mechanism) | HIGH | PARTIALLY_RESOLVED | Mechanism named in v1.2 but mechanically wrong — runtime-drop does not abort `tokio::spawn`'d tasks; `JoinHandle` drop detaches, not aborts. Carried forward as F-6.20-P03-H-001. |
| F-6.20-P02-M-001 (bind field) | MED | PARTIALLY_RESOLVED | AC added on paper but un-implementable: all 6 clones hardcode `TcpListener::bind("127.0.0.1:0")`; `StubConfig` has no `port`/`bind` field. Carried forward as F-6.20-P03-H-002 (elevated to HIGH). |
| F-6.20-P02-M-002 (seed endpoint) | MED | RESOLVED | v1.2 specifies `POST /seed` with body schema and error codes; present in route modules. No residual gap. |
| F-6.20-19 (axum-server dep) | MED | RESOLVED | Story no longer references `axum-server`; axum 0.7 built-in serve used throughout. No residual gap. |

## Part B — New Findings (or all findings for pass 1)

### CRITICAL

None.

### HIGH

#### F-6.20-P03-H-001 (ADV-WV1-P03-HIGH-001): Runtime-drop teardown is mechanically wrong

- **Severity:** HIGH
- **Category:** concurrency
- **Location:** `crates/prism-dtu-common/src/clone.rs:13-19`, S-6.20 AC-5, EC-010
- **Description:** The story's shutdown narrative states "runtime-drop tears down
  all tasks." This is false for per-clone `stop_all()`. `tokio::spawn` returns a
  `JoinHandle`; dropping the `JoinHandle` *detaches* the task — the task
  continues running. Dropping `Box<dyn BehavioralClone>` does not reach spawned
  tasks. The trait's `stop_all(&mut self) -> Result<()>` has no mechanism to
  abort them.
- **Evidence:**
  ```rust
  // crates/prism-dtu-common/src/clone.rs:13-19
  pub trait BehavioralClone: Send + Sync {
      fn name(&self) -> &str;
      fn apply_config(&mut self, config: &StubConfig) -> Result<()>;
      fn start(&mut self) -> Result<()>;
      fn stop_all(&mut self) -> Result<()>;
  }
  ```
  `start()` internally calls `tokio::spawn`. No `JoinHandle` is stored or
  returned. `stop_all()` cannot call `handle.abort()` on a handle it doesn't
  hold. The HTTP listener continues accepting connections after `stop_all()`
  returns `Ok(())`.
- **Proposed Fix:** Extend the trait (preferred — also resolves H-002):
  ```rust
  fn start(
      &mut self,
      bind: SocketAddr,
      shutdown_rx: oneshot::Receiver<()>,
  ) -> Result<JoinHandle<()>>;
  ```
  `DtuDemoServer` owns all `JoinHandle`s and calls `handle.abort()` in its
  `stop_all()` implementation. Alternative: pass a `CancellationToken` into
  each clone at construction; `stop_all()` calls `token.cancel()`.

---

#### F-6.20-P03-H-002 (ADV-WV1-P03-HIGH-002): Configured ports (17080–17085) un-plumbable — all 6 clones hardcode bind

- **Severity:** HIGH
- **Category:** interface-gaps
- **Location:** AC-1; all six clone crate `start()` implementations; `crates/prism-dtu-common/src/config.rs:5-12`
- **Description:** AC-1 requires each clone to listen on a configured loopback
  port (17080–17085). No path exists from the spec's AC to the actual bind call:
  `StubConfig` has no `port` or `bind` field, and all six clones hardcode port 0.
- **Evidence:**

  | File | Line | Hardcoded value |
  |------|------|-----------------|
  | `crates/prism-dtu-cyberint/src/clone.rs` | 82 | `TcpListener::bind("127.0.0.1:0")` |
  | `crates/prism-dtu-crowdstrike/src/clone.rs` | 73 | `TcpListener::bind("127.0.0.1:0")` |
  | `crates/prism-dtu-claroty/src/clone.rs` | 97 | `TcpListener::bind("127.0.0.1:0")` |
  | `crates/prism-dtu-armis/src/clone.rs` | 99 | `TcpListener::bind("127.0.0.1:0")` |
  | `crates/prism-dtu-nvd/src/clone.rs` | 77 | `TcpListener::bind("127.0.0.1:0")` |
  | `crates/prism-dtu-threatintel/src/clone.rs` | 65 | `TcpListener::bind("127.0.0.1:0")` |

  `crates/prism-dtu-common/src/config.rs:5-12` — `StubConfig` has no `port` or
  `bind` field.

- **Proposed Fix:** Add `bind: SocketAddr` to `StubConfig` (or pass it as a
  `start()` parameter per the preferred fix in H-001). Remove all six
  `TcpListener::bind("127.0.0.1:0")` hardcodes; use the configured address.

---

### MEDIUM

#### F-6.20-P03-M-001 (ADV-WV1-P03-MED-001): Per-clone apply_config() cannot enforce loopback 403

- **Severity:** MEDIUM
- **Category:** spec-fidelity
- **Location:** AC-3; `BehavioralClone::apply_config` signature
- **Description:** AC-3 claims "per-clone `apply_config()` enforces loopback 403
  for non-loopback sources." The method runs before `start()` and before the
  axum router is constructed; it has no access to `ConnectInfo<SocketAddr>` and
  cannot inject middleware. The stated enforcement mechanism is impossible.
- **Evidence:** `apply_config(&mut self, config: &StubConfig) -> Result<()>` —
  no router handle, no middleware slot, no access to peer address.
- **Proposed Fix:** Either (a) remove the "apply_config enforces loopback 403"
  claim and state instead that loopback enforcement is via OS-level bind to
  `127.0.0.1`, or (b) specify an axum `middleware::from_fn` inspecting
  `Extension<ConnectInfo<SocketAddr>>` and return 403 for non-loopback remotes,
  with the middleware added in `start()`.

---

#### F-6.20-P03-M-002 (ADV-WV1-P03-MED-002): EC-008 "graceful shutdown within 5s" contradicts AC-5

- **Severity:** MEDIUM
- **Category:** contradictions
- **Location:** EC-008 vs. AC-5
- **Description:** EC-008 states "DtuDemoServer shuts down gracefully within 5
  seconds, completing or draining in-flight requests." AC-5 states "in-flight
  HTTP requests may be aborted mid-response." These are mutually exclusive: if
  requests are drained, shutdown latency is unbounded (a slow client can hold it
  open indefinitely); if requests are aborted, EC-008's "completing" language is
  false.
- **Evidence:** Direct textual contradiction between EC-008 and AC-5 in
  S-6.20 v1.2.
- **Proposed Fix:** Rewrite EC-008 to match AC-5:
  > EC-008: DtuDemoServer shuts down within 5 seconds. In-flight HTTP requests
  > are aborted; the 5-second bound is unconditional.
  Or, if graceful drain is intended, remove the 5s hard bound and replace with a
  best-effort timeout after which connections are forcibly closed.

---

### LOW

#### F-6.20-P03-L-001 (ADV-WV1-P03-LOW-001): Missing bind-address comment in clarity code sample

- **Severity:** LOW
- **Category:** ambiguous-language
- **Location:** S-6.20 implementation notes / clarity sample
- **Description:** The clarity sample shows `TcpListener::bind("127.0.0.1:0")`
  without a comment marking port 0 as a placeholder. Given AC-1 requires
  configured ports, a reader following the sample will reproduce the hardcode bug.
- **Proposed Fix:** Add `// TODO: use configured port per AC-1` or update the
  sample once H-002 is resolved.

---

#### F-6.20-P03-L-002 (ADV-WV1-P03-LOW-002): axum + rustls transitive version compatibility not verified

- **Severity:** LOW
- **Category:** verification-gaps
- **Location:** S-6.20 dependency section
- **Description:** The story specifies axum 0.7 but does not assert a
  Cargo.lock-verified dependency tree. axum 0.7 + rustls 0.22 + tower 0.4 have
  a known incompatibility when both `hyper-rustls` and `tokio-rustls` appear in
  the graph. This is a build-time failure, not a runtime bug, but it must be
  verified before implementation begins.
- **Proposed Fix:** Add a build-verification note: "Run `cargo check -p
  prism-dtu-demo-server` on a clean clone before declaring spec complete.
  Resolve any feature-flag conflicts in the TLS dependency chain."

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 2 |
| MEDIUM | 2 |
| LOW | 2 |

**Overall Assessment:** pass-with-findings
**Convergence:** findings remain — iterate
**Readiness:** requires revision; worktree creation NOT approved

### Observations

| # | Observation |
|---|-------------|
| OBS-1 | Story does not specify which crate owns `DtuDemoServer`. Clarify to avoid implementation ambiguity. |
| OBS-2 | No AC for `stop_all` idempotency (calling twice). Worth a sentence. |
| OBS-3 | Port range 17080–17085 is not reserved. Add note that these must not conflict with other prism services or OS ephemeral ranges on analyst machines. |
| OBS-4 | Behavior unspecified when a clone's `start()` fails mid-way through starting all 6 clones. Should `DtuDemoServer` tear down already-started clones? |
| OBS-5 | `apply_config()` takes `&StubConfig` — if config is heap-allocated, per-clone cloning may be warranted. Mention ownership in trait contract. |
| OBS-6 | Story references `prism-dtu-demo-server` binary but no `[[bin]]` target in any Cargo.toml is shown. Confirm binary target is declared. |

### Required Actions Before Pass 4

**RQ-1 (resolves H-001):** Fix `stop_all` mechanism. Recommended: extend trait
to `start(&mut self, bind: SocketAddr, shutdown_rx: oneshot::Receiver<()>) ->
Result<JoinHandle<()>>`. `DtuDemoServer` owns all `JoinHandle`s and calls
`handle.abort()`. Simultaneously resolves H-002.

**RQ-2 (resolves H-002, if not resolved by RQ-1):** Add `bind: SocketAddr` to
`StubConfig`; plumb through each clone's `start()`. Remove all six
`TcpListener::bind("127.0.0.1:0")` hardcodes.

**RQ-3 (resolves M-001):** Remove "apply_config enforces loopback 403" from
AC-3, OR specify axum `ConnectInfo` middleware in `start()`.

**RQ-4 (resolves M-002):** Rewrite EC-008 to align with AC-5.

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 3 |
| **New findings** | 4 (2 HIGH + 2 MED — carried forward from PARTIAL resolutions, re-examined against code) |
| **Duplicate/variant findings** | 2 (L-001 and L-002 are minor variants of prior clarity gaps) |
| **Novelty score** | 4 / (4 + 2) = 0.67 |
| **Median severity** | 3.0 (HIGH = 4, MED = 3, LOW = 2; median of [4,4,3,3,2,2] = 3.0) |
| **Trajectory** | 10→8→6 (total findings across passes 1–3) |
| **Verdict** | FINDINGS_REMAIN |
