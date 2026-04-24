---
adr_id: ADR-002
title: "Canonical L2 DTU Clone Template"
document_type: architecture-section
level: ADR
section: decisions/ADR-002-l2-dtu-clone-template
version: "1.0"
status: accepted
producer: architect
timestamp: 2026-04-22T00:00:00
phase: phase-3-dtu-wave-1
inputs:
  - .factory/tech-debt-register.md
  - .factory/specs/architecture/dtu-assessment.md
  - .factory/specs/architecture/decisions/ADR-001-dtu-rate-limit-pattern.md
  - crates/prism-dtu-threatintel/ (origin/develop)
  - crates/prism-dtu-nvd/ (origin/develop)
  - crates/prism-dtu-common/ (origin/develop)
traces_to: ARCH-INDEX.md
closes_debt: TD-WV0-05
---

# ADR-002: Canonical L2 DTU Clone Template

> **Sizing guidance:** Each architecture section file targets 800-1,200 tokens
> (~50-80 lines of markdown). If this section exceeds 1,500 tokens, consider
> splitting it further into sub-sections.

## [Section Content]

## Status

Accepted

## Context

Wave 0c shipped two L2-fidelity DTU clones: `prism-dtu-threatintel` (S-6.14) and
`prism-dtu-nvd` (S-6.15). Reviewing the two crates after merge reveals structural
drift in four areas:

1. `ThreatIntelState` has no `apply_config()` method — configure logic lives inline
   in `clone.rs`. `NvdState` correctly places this in `state.rs::apply_config()`.
2. `prism-dtu-threatintel` has no `POST /dtu/reset` HTTP endpoint. `prism-dtu-nvd`
   exposes one, enabling test harnesses to reset state over HTTP without holding a
   direct reference to the clone object.
3. `prism-dtu-threatintel` omits the explicit `[lib]` name in `Cargo.toml`.
   `prism-dtu-nvd` declares `[lib] name = "prism_dtu_nvd"` explicitly.
4. `prism-dtu-threatintel` exports only the clone struct from `lib.rs`. `prism-dtu-nvd`
   also re-exports the state type and key error/response types.

Both crates already satisfy `publish = false`, `description`, the `dtu` feature gate,
`[lints] workspace = true`, fixture directory layout, and the `Json(body)` serialization
convention. Those items are confirmed correct and included in this template for
completeness.

Wave 1 introduces two more L2 clones: `prism-dtu-cyberint` (S-6.09) and
`prism-dtu-armis` (S-6.10). Waves 2 and 3 add seven more. Without a canonical
template, each new clone risks introducing further structural variance. This ADR
locks the template before S-6.09 is implemented and mandates retroactive alignment
of the two existing clones.

## Decision

Every L2-fidelity DTU clone crate (those sharing the pattern established by S-6.14
and S-6.15) MUST conform to the structural template defined in this ADR.

### 1. Directory Layout

```
crates/prism-dtu-<name>/
├── Cargo.toml
├── fixtures/
│   └── <name>.json          # or multiple fixture files
├── src/
│   ├── lib.rs
│   ├── clone.rs             # BehavioralClone impl
│   ├── routes/
│   │   ├── mod.rs
│   │   ├── <api>.rs         # vendor API endpoint handlers
│   │   └── dtu.rs           # /dtu/* internal test endpoints
│   ├── state.rs             # shared Arc<State> struct
│   └── types.rs             # request/response structs
└── tests/
    ├── ac_1_*.rs
    ├── ...
    └── ac_N_fidelity_validator.rs
```

Sub-splitting `routes/` further (e.g. nested dirs) is permitted when a vendor API
has more than four distinct route groups. The `fixtures/` directory name is fixed.

### 2. Cargo.toml Required Fields

```toml
[package]
name = "prism-dtu-<name>"
version = "0.1.0"
edition = "2021"
license = "MIT"
publish = false
description = "<fidelity-level>-fidelity behavioral clone of the <Vendor> API"

[lib]
name = "prism_dtu_<name>"

[features]
dtu = []

[lints]
workspace = true
```

Rules:

- `publish = false` — REQUIRED. No DTU crate is ever published to crates.io.
- `description` — REQUIRED. Format: `"<L1|L2|L3|L4>-fidelity behavioral clone of
  the <Vendor> <Product> API"`. Copy the DTU assessment's fidelity classification.
- `[lib] name` — REQUIRED. Set to the snake_case crate name (underscores). Cargo's
  derived default sometimes diverges from the explicit name; make it explicit.
- `[features] dtu = []` — REQUIRED. All clone code is gated behind
  `#[cfg(any(test, feature = "dtu"))]`.
- `[lints] workspace = true` — REQUIRED. Inherits workspace clippy policy.

### 3. src/lib.rs Required Shape

```rust
#![cfg(any(test, feature = "dtu"))]

pub mod clone;
pub mod routes;
pub mod state;
pub mod types;

pub use clone::<Name>Clone;
pub use state::<Name>State;
```

Rules:

- The `#![cfg(...)]` attribute MUST be the first non-comment item in the file.
- `pub use clone::<Name>Clone` — always re-export the clone struct.
- `pub use state::<Name>State` — always re-export the state struct.
- Additional `pub use` for domain error types or key response types is permitted.
- No route handlers are re-exported from `lib.rs`.

### 4. src/clone.rs Required Shape

The clone struct holds `state: Arc<<Name>State>` and `bound_addr: Option<SocketAddr>`.

```rust
pub struct <Name>Clone {
    state: Arc<<Name>State>,
    bound_addr: Option<SocketAddr>,
}

impl <Name>Clone {
    pub fn new() -> anyhow::Result<Self> { ... }
    fn build_router(&self) -> Router { ... }
}

#[async_trait]
impl BehavioralClone for <Name>Clone {
    async fn start(&mut self) -> anyhow::Result<()> { ... }
    async fn reset(&self) -> anyhow::Result<()> {
        self.state.reset();
        Ok(())
    }
    async fn configure(&self, config: serde_json::Value) -> anyhow::Result<()> {
        self.state.apply_config(&config)  // delegate — no inline logic here
    }
    fn bound_addr(&self) -> SocketAddr { ... }
}
```

Rules:

- `reset()` MUST delegate to `self.state.reset()` with no additional logic.
- `configure()` MUST delegate to `self.state.apply_config(&config)` — no inline
  JSON field inspection in `clone.rs`. All config interpretation lives in `state.rs`.
- `build_router()` MUST mount `POST /dtu/configure` and `POST /dtu/reset` alongside
  vendor API routes (see Section 6).

### 5. src/state.rs Required Shape

The state struct MUST expose these three methods:

```rust
impl <Name>State {
    pub fn new(...) -> Self { ... }

    /// Reset all mutable state to initial values (called by BehavioralClone::reset).
    pub fn reset(&self) { ... }

    /// Apply a JSON configuration patch (from POST /dtu/configure).
    pub fn apply_config(&self, config: &serde_json::Value) -> anyhow::Result<()> { ... }
}
```

Rules:

- `reset()` MUST restore ALL mutable fields to their post-`new()` values.
  Immutable fields (e.g. pre-loaded fixture registries) are not affected.
- `apply_config()` returns `anyhow::Result<()>`. Unknown keys MUST be silently
  ignored (see TD-WV0-04; strict schema enforcement is deferred to a later wave).
- No HTTP-layer types (`axum::Json`, `axum::extract::*`) appear in `state.rs`.
  The state struct is pure Rust — no Axum dependency for its public methods.

### 6. Required HTTP Endpoints

Every L2 clone MUST expose these four endpoints:

| Method | Path | Handler location | Purpose |
|--------|------|-----------------|---------|
| (vendor-specific) | (vendor path) | `routes/<api>.rs` | Real API surface |
| `POST` | `/dtu/configure` | `routes/dtu.rs` | Runtime reconfiguration |
| `POST` | `/dtu/reset` | `routes/dtu.rs` | Reset all mutable state |
| `GET` | `/dtu/health` | `routes/dtu.rs` | Liveness check for test setup |

The `/dtu/health` endpoint returns `HTTP 200 {"status": "ok"}` with no state access —
useful for test-harness readiness polling without side effects.

Route registration in `build_router()`:

```rust
fn build_router(&self) -> Router {
    Router::new()
        // vendor routes …
        .route("/dtu/configure", post(post_configure))
        .route("/dtu/reset",     post(post_reset))
        .route("/dtu/health",    get(get_health))
        .with_state(self.state.clone())
}
```

Note: clone-specific introspection endpoints (e.g. NVD's
`GET /dtu/request-count/:cve_id`) are permitted as additions; they are not
part of the canonical required set.

### 7. Serialization Convention

Route handlers MUST use the `Json(body)` extractor and `Json(...)` response
constructor directly. Manual string construction or `serde_json::to_string` in
handler code is not permitted.

Canonical configure handler shape:

```rust
pub async fn post_configure(
    State(state): State<Arc<<Name>State>>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    match state.apply_config(&body) {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
```

Canonical reset handler shape:

```rust
pub async fn post_reset(State(state): State<Arc<<Name>State>>) -> impl IntoResponse {
    state.reset();
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response()
}
```

### 8. Fidelity Validator Test

Every L2 clone MUST include a fidelity test file named `tests/ac_N_fidelity_validator.rs`
(where N is the last AC number) that calls `FidelityValidator::run` and asserts
`checks_failed == 0`.

Canonical shape:

```rust
#[cfg(feature = "dtu")]
#[tokio::test]
async fn fidelity_validator_passes() {
    let mut clone = <Name>Clone::new().expect("clone init");
    clone.start().await.expect("clone start");
    let base_url = clone.base_url();

    let checks = vec![
        FidelityCheck {
            endpoint: "/path/to/endpoint".to_string(),
            method: http::Method::GET,
            body: None,
            expected_status: 200,
            required_fields: vec!["field_name".to_string()],
        },
        // … one check per AC endpoint shape requirement
    ];

    let report = FidelityValidator::run(&base_url, checks).await;
    assert_eq!(
        report.checks_failed, 0,
        "fidelity failures: {:?}", report.failures
    );
}
```

The fidelity test validates endpoint shape, not business logic. AC tests validate
business logic. Both are required.

### 9. Fixture Directory

- All fixture files live in `fixtures/` at the crate root.
- Load fixtures with `prism_dtu_common::load_fixture_as(env!("CARGO_MANIFEST_DIR"), "<name>")`.
- Fixture file names match the entity type (e.g. `cves.json`, `alerts.json`).
- Fixture content covers the full relevant severity/status spectrum required by story ACs.

## Compliance Checklist

The following items are mechanically checkable and MUST be true for every L2 clone.
Paste this list into story ACs for S-6.09, S-6.10, and all future L2 clones.

```
[ ] Cargo.toml: `publish = false` present
[ ] Cargo.toml: `description` field present and matches fidelity format
[ ] Cargo.toml: `[lib] name = "prism_dtu_<name>"` declared explicitly
[ ] Cargo.toml: `[features] dtu = []` declared
[ ] Cargo.toml: `[lints] workspace = true` declared
[ ] src/lib.rs: `#![cfg(any(test, feature = "dtu"))]` is first non-comment attribute
[ ] src/lib.rs: `pub use clone::<Name>Clone` present
[ ] src/lib.rs: `pub use state::<Name>State` present
[ ] src/clone.rs: `configure()` delegates to `self.state.apply_config(&config)` — no inline JSON parsing
[ ] src/clone.rs: `reset()` delegates to `self.state.reset()` — no inline logic
[ ] src/state.rs: `reset()` method present
[ ] src/state.rs: `apply_config()` method present, returns `anyhow::Result<()>`
[ ] src/routes/dtu.rs: `POST /dtu/configure` handler present, uses `Json(body)` extractor
[ ] src/routes/dtu.rs: `POST /dtu/reset` handler present
[ ] src/routes/dtu.rs: `GET /dtu/health` handler present
[ ] Route handlers: no manual `serde_json::to_string` — use `Json(...)` constructor
[ ] fixtures/ directory present with at least one fixture file
[ ] Fixture loaded via `prism_dtu_common::load_fixture_as`
[ ] tests/ac_N_fidelity_validator.rs: FidelityValidator used, asserts `checks_failed == 0`
[ ] All `[[test]]` entries carry `required-features = ["dtu"]`
```

## Deviation Policy

### L4 Clones (CrowdStrike S-6.07, Claroty S-6.08, PagerDuty S-6.12, Jira S-6.13)

L4-adversarial clones implement more complex behavioral models. The following
deviations are permitted for L4 clones:

- `state.rs` MAY have multiple state structs or a layered state model if the vendor
  API has distinct auth-state and data-state lifecycles.
- `clone.rs::configure()` MAY delegate to multiple `apply_*` methods on state
  rather than a single `apply_config()`, provided each method has a doc comment
  explaining what aspect it configures.
- The fidelity validator test MUST still be present but MAY be split across multiple
  test files (one per behavioral dimension) rather than a single file.
- Additional `/dtu/*` introspection routes are encouraged (L4 complexity warrants
  richer test observability).

All other items in the compliance checklist apply equally to L4 clones.

### L1 Clones

No L1 clones are planned in the current DTU assessment. If introduced, they are
exempt from the fidelity-validator test requirement (L1 = API shape only; the
validator itself provides L1 coverage).

## Retroactive Cleanup

`prism-dtu-threatintel` (S-6.14) and `prism-dtu-nvd` (S-6.15) diverge from this
template in the following ways that MUST be corrected in the Wave 1 maintenance PR:

### prism-dtu-threatintel gaps

1. **No `POST /dtu/reset` endpoint** — add `post_reset` handler in new
   `src/routes/dtu.rs`; mount at `/dtu/reset` in `build_router`.
2. **No `GET /dtu/health` endpoint** — add `get_health` handler.
3. **`configure()` inline logic in `clone.rs`** — extract JSON field inspection
   into `ThreatIntelState::apply_config()` and make `clone.rs::configure()`
   delegate to it.
4. **Missing `[lib] name`** — add `[lib] name = "prism_dtu_threatintel"` to Cargo.toml.
5. **Missing `pub use state::ThreatIntelState`** — add to `lib.rs`.

### prism-dtu-nvd gaps

1. **No `GET /dtu/health` endpoint** — add `get_health` handler to `routes/dtu.rs`;
   mount at `/dtu/health` in `build_router`.

The retroactive cleanup is tracked under TD-WV0-05. Upon merging the maintenance PR,
TD-WV0-05 is resolved.

## Consequences

- Stories S-6.09 and S-6.10 (Wave 1) will reference this ADR in their ACs.
- Stories S-6.11 through S-6.19 will also reference this ADR as applicable.
- Code reviewers MUST check every checklist item before approving a DTU clone PR.
- The maintenance PR for TD-WV0-05 MUST make `prism-dtu-threatintel` and
  `prism-dtu-nvd` fully compliant before Wave 1 gate.

## Alternatives Considered

- **(A) Macro-based code generation:** Generate `configure` and `reset` boilerplate
  via a derive macro. Rejected — the complexity of a proc macro outweighs the benefit
  for 14 crates; the template is straightforward enough to enforce through review.
- **(B) Default method implementations on `BehavioralClone`:** Add `reset_state` and
  `apply_config` as default methods on the trait using associated types. Rejected —
  forces trait-level awareness of state internals; breaks the separation between the
  trait (in `prism-dtu-common`) and per-clone state models.
- **(C) Status quo (no template):** Rejected — with 9 more L2 clones in the pipeline,
  drift compounds and retroactive cleanup cost grows non-linearly.

## Amendment #2: TLS Propagation (TD-WV1-04)

**Added:** 2026-04-23 (PR #32, 4a9dffb1, wave-1-gate-re-convergence pass-16-remediation, P3WV1P-A-L-001)

### Context

After Wave 1 Pass 15 convergence, the `prism-dtu-demo-server` binary's `--tls` CLI flag was found to be cosmetic: it generated a self-signed cert + printed a fingerprint but discarded the `RustlsConfig` and clones still served plain HTTP. TD-WV1-04 was filed to close this gap.

### Decision

Extend the `BehavioralClone::start_on` trait signature with an optional TLS config parameter:

```rust
async fn start_on(
    &mut self,
    bind: SocketAddr,
    shutdown: Option<broadcast::Receiver<()>>,
    #[cfg(feature = "tls")] tls: Option<Arc<RustlsConfig>>,
    #[cfg(not(feature = "tls"))] tls: Option<()>,
) -> anyhow::Result<SocketAddr>;
```

**Rationale for Option<()> fallback:** The parameter exists in both feature modes (preserves signature uniformity), but the meaningful `Arc<RustlsConfig>` type is only available when the `tls` feature is active. Under no-tls, `Option<()>` always accepts `None`.

### Required implementations

Each clone crate MUST:
1. Carry a `tls_active: bool` field (plain; defaults false)
2. Under `#[cfg(feature = "tls")]`, also carry a `tls_handle: Option<axum_server::Handle>` field
3. Override `start_on` to branch on `Some(rustls_cfg)` → `axum_server::bind_rustls` vs `None` → `axum::serve`
4. Override `is_tls_active(&self) -> bool` to return `self.tls_active`
5. Ensure `stop()` calls `handle.graceful_shutdown(Some(5s))` on TLS path before `server_handle.abort()`

The trait's default `base_url()` uses `is_tls_active()` to branch the URL scheme — no override needed at the clone layer unless a clone overrides base_url directly.

### Backward compatibility

The `start()` default impl delegates to `start_on(addr, None, None)`, so existing callers that don't use the 3-arg form continue to work unchanged.

### Feature gating

The TLS code paths are entirely feature-gated. Binaries compiled without `--features tls` include no `axum_server` dependency, no `rustls`, and no TLS code. Passing `--tls` to such a binary errors at runtime with a clear message.

### Trace

- Trait: `crates/prism-dtu-common/src/clone.rs`
- Clone implementations: `crates/prism-dtu-<name>/src/clone.rs` for crowdstrike, claroty, cyberint, armis, threatintel, nvd
- Harness: `crates/prism-dtu-demo-server/src/harness.rs` (start_all accepts tls, propagates to each clone)
- Main: `crates/prism-dtu-demo-server/src/main.rs` (handle_tls returns Option<Arc<RustlsConfig>>)
- Tests: `crates/prism-dtu-demo-server/tests/td_wv1_04_harness_tls.rs`, `tests/td_wv1_04_binary_tls_e2e.rs`

### Follow-up TDs

- TD-WV1-04-FU-001: TLS shutdown asymmetry vs HTTP graceful drain
- TD-WV1-04-FU-002: AC-5 test doesn't cover TLS + stop_all
- TD-WV1-04-FU-003: stdout pipe capture ordering comment misleading

---

## Addendum: `level:` Frontmatter Semantics

**Added:** 2026-04-23 (wave-1-gate-pass-5-batch-remediation, P3WV1E-A-OBS-002)

The `level:` frontmatter field carries two distinct semantic meanings depending on the
story type. Context determines correct interpretation.

### For DTU stories (S-6.06 through S-6.20 and any future DTU stories)

`level:` carries the **DTU fidelity tier** as defined in `dtu-assessment.md §1a
Fidelity Taxonomy`:

| Value | Fidelity Tier | Meaning |
|-------|--------------|---------|
| `"L0"` | Static fixture | Fixed JSON responses; no state |
| `"L1"` | Shape-correct | Correct API shape, minimal logic |
| `"L2"` | Stateful | In-memory state, fixture registry, rate-limit |
| `"L3"` | Behavioral | Full behavioral model (state machine, lifecycle) |
| `"L4"` | Adversarial | Active fault injection, malicious response simulation |

The fidelity tier for each DTU story is authoritative in `dtu-assessment.md` and is
restated in the story title, H1, and Dev Notes section. The `level:` frontmatter field
MUST match these sources.

### For non-DTU stories (all product stories, architecture docs, spec files)

`level:` carries the **VSDD document hierarchy level** (L0-L5), indicating where the
document sits in the specification hierarchy (L0 = product vision, L5 = implementation
detail).

### Why the ambiguity arose

The two taxonomies coincidentally share the same label space (L0..L5). In the v1.3
bulk correction pass (2026-04-20), all DTU stories had `level:` set to `"L4"` under
the erroneous interpretation that `level:` always carries the VSDD hierarchy level.
Because `"L4"` is a valid value in both taxonomies, the error was not caught by
label-range validation. Four stories required correction at Wave 0 and Wave 1 gates
(S-6.09 pass 1, S-6.10 pass 4, S-6.14/S-6.15 pass 5).

### Sub-rule: Shared-Infrastructure DTU Stories

**Added:** 2026-04-23 (wave-1-gate-pass-7-remediation, P3WV1G-A-H-001)

S-6.06 (prism-dtu-common) is a DTU story but does NOT implement a behavioral clone — it provides shared infrastructure (trait definitions, test helpers, fixture loaders). It has no DTU fidelity tier (dtu-assessment.md §1 marks it "N/A (shared harness)").

For such shared-infrastructure DTU stories:
- Set `level:` to `null` in frontmatter (or omit the field entirely).
- Rationale: no L0–L4 value applies; a VSDD hierarchy value would contradict the DTU-story rule above.
- This sub-rule applies to S-6.06 and S-6.20 today and to any future shared-infrastructure DTU story added under a similar pattern.

### Rule for story authors

When writing or reviewing a DTU story:
- Set `level:` to the fidelity tier from `dtu-assessment.md §1a` (match the title).
- Do NOT set `level:` to the VSDD hierarchy level for DTU stories.
- Exception: if the story provides shared infrastructure with no fidelity tier (like S-6.06), set `level: null` per the sub-rule above.

When writing or reviewing a non-DTU story:
- Set `level:` to the VSDD hierarchy level.
- DTU fidelity tier is not applicable.

## Related

- TD-WV0-05 (closed by this ADR + maintenance PR)
- ADR-001 — per-clone rate-limit semantics (each clone owns its rate-limit; this ADR
  does not override that decision)
- Stories: S-6.09, S-6.10 (first consumers of this template)
- Stories: S-6.14, S-6.15 (retroactive cleanup target)
- `prism-dtu-common::BehavioralClone` trait (canonical interface)
- `prism-dtu-common::FidelityValidator` (required in every clone's test suite)
