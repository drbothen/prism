---
document_type: story
story_id: S-REL-AGENT-VERSION-001
title: "devops: migrate agent-facing version surfaces to PRISM_VERSION — prism-mcp serverInfo.version runtime wiring + prism-spec-engine per-crate build.rs injection"
wave: F-A
epic_id: E-REL-IDENTITY
priority: P1
status: draft
version: "1.1"
level: "L4"
producer: product-owner
timestamp: "2026-09-05T00:00:00Z"
tdd_mode: strict
subsystems: [SS-22]
# Subsystem anchor justification:
#   SS-22 (Process Lifecycle) owns prism-mcp/src/server.rs (MCP handshake protocol),
#   prism-spec-engine/src/pipeline.rs (spec-driven sensor orchestration), and
#   prism-bin/src/boot.rs (boot step 9 PrismServer::with_deps call site). All three
#   injection sites live within SS-22's product lifecycle boundary.
crates_touched: [prism-mcp, prism-spec-engine, prism-bin]
target_module: prism-mcp, prism-spec-engine
capabilities: []
behavioral_contracts: []
# BC status: N/A — agent-facing version surface migration is a build-infrastructure change.
# No subsystem behavioral contract governs env!() macro source selection or PrismServer
# constructor parameter threading. Conforming per S-REL-002 precedent
# (behavioral_contracts: [] on version alignment stories). POL-14 NO-OP.
verification_properties: []
depends_on: [S-REL-BVERSION-INJECT-001]
blocks: []
# Dependency anchor justifications:
#   depends_on S-REL-BVERSION-INJECT-001: D2 build.rs pattern is the authoritative
#     template for the prism-spec-engine/build.rs algorithm (ADR-064 D4 §Surface B:
#     "The build.rs algorithm is identical to D2. The D2 normative contract in this ADR
#     is the single authoritative spec."). D2 must land before D4 so the shared
#     algorithm is established and tested. Also: boot.rs step-9 (Surface A) is
#     already touched by D2; D4 extends the same call site.
#   NOTE: This story has human-directed beta.1 scope (S-1, 2026-09-05) overriding the
#     ADR-064 D4 "not blocking for beta.1" note. Must land on develop in its OWN PR,
#     SEPARATE from PR #262 (S-REL-BVERSION-INJECT-001), BEFORE the v1.0.0-beta.1 tag.
points: 5
estimated_days: 1
risk: MEDIUM
acceptance_criteria_count: 8
red_gate_tests: 5
estimated_passes: "2-3 LOCAL adversary passes"
holdout_scenarios:
  - HS-032-001
  - HS-032-002
  - HS-032-003
assumption_validations: []
risk_mitigations:
  - "Surface A (prism-mcp): PrismServer::get_info explicitly overrides the
    #[tool_handler(version = \"0.1.0\")] proc macro attribute value. The explicit
    get_info return value is authoritative for the MCP handshake serverInfo.version;
    the proc macro attribute is superseded by the override. Do NOT attempt to use
    env!() in the proc macro attribute position (macro attributes accept only string
    literals). The implementer SHOULD update the proc macro attribute to reflect
    the crate version (verify whether rmcp accepts env!(\"CARGO_PKG_VERSION\")
    in attribute position; if not, leave it at \"0.1.0\" with a comment that it is
    superseded by get_info). ADR-064 D4 §Note."
  - "Surface A (runtime wiring vs compile-time): product_version is threaded through
    PrismServer::with_deps as &'static str. env!(\"PRISM_VERSION\") is a compile-time
    constant that expands to a &'static str — the lifetimes are compatible. No heap
    allocation required. ADR-064 D4 §Surface A — 'Adding a product_version: &'static
    str parameter is wiring, not redesign (ADR-022 §C).' "
  - "Surface B (prism-spec-engine): build.rs algorithm is identical to D2 normative
    contract (ADR-064 D4 §Surface B). Parallel implementations are acceptable when the
    spec is authoritative (D4: '20 lines, stable algorithm'). Do NOT try to share code
    between crates via include! — that would create a build-time dependency. Each
    build.rs is self-contained."
  - "Surface B (prism-spec-engine): runtime wiring was evaluated and rejected per ADR-064 D4
    §Surface B ('threading version through the infusion call chain: 3 callers in
    infusion/mod.rs → build_http_client_with_timeout would require 3+ pub(crate) function
    signature changes with no architectural gain over compile-time injection for a closed
    crate'). The per-crate build.rs is the ratified mechanism."
  - "PRISM_VERSION resolution: on local dev builds (no GITHUB_REF_NAME), PRISM_VERSION ==
    CARGO_PKG_VERSION == '1.0.0-dev' (after S-REL-DEV-RESET-001 merged). The MCP handshake
    will report '1.0.0-dev' locally and '1.0.0-beta.1' (or the tagged version) in CI release
    builds. This is correct and intentional per ADR-064 D4 design."
  - "ADR-022 §C wiring discipline: adding product_version field + with_deps parameter is
    wiring, not redesign. The Canonical Principle Standing Rule 3 §4 explicitly approves
    adding Arc<dyn Foo> or similar parameters to constructors that lacked them. Same applies
    to a &'static str version field."
inputs:
  - "crates/prism-mcp/src/server.rs"
  - "crates/prism-spec-engine/src/pipeline.rs"
  - "crates/prism-bin/src/boot.rs"
  - ".factory/specs/architecture/decisions/ADR-064-pre-release-binary-version-identity.md"
  - ".factory/specs/architecture/decisions/ADR-050-workspace-reqwest-tls-backend.md"
input-hash: "[pending-recompute]"
traces_to: []
cycle: "v1.0.0-beta.1-release-identity"
phase: "3"
---

# S-REL-AGENT-VERSION-001 — Migrate Agent-Facing Version Surfaces to PRISM_VERSION

**Story ID:** S-REL-AGENT-VERSION-001
**Status:** draft
**Version:** v1.1
**Wave:** F-A
**Priority:** P1 (HIGH — human-directed beta.1 scope, S-1 decision 2026-09-05)
**Points:** 5
**Beta.1-blocking:** YES — human-directed decision (S-1, 2026-09-05) overrides ADR-064 D4
"not blocking for beta.1" note. This story MUST merge to develop in its OWN PR BEFORE the
v1.0.0-beta.1 tag is cut. It is delivered as a SEPARATE PR from PR #262
(S-REL-BVERSION-INJECT-001).

---

## Origin

After S-REL-BVERSION-INJECT-001 ships, five of the six prism-bin version-report sites
will correctly emit `PRISM_VERSION` from the D2 build.rs injection. However, two
agent-facing surfaces remain on stale version strings:

1. **prism-mcp `serverInfo.version`** — the MCP `initialize` handshake response,
   the most agent-visible surface. Currently hardcoded to `"0.1.0"` in
   `PrismServer::get_info` via `Implementation::new("prism", "0.1.0")`. An LLM agent
   reads this field to identify which prism server it is connected to. After v1.0.0-beta.1
   ships, agents would see `serverInfo.version = "0.1.0"` — a misleading pre-release
   identifier.

2. **prism-spec-engine HTTP user-agent** — `build_http_client_with_timeout` in
   `crates/prism-spec-engine/src/pipeline.rs` uses `env!("CARGO_PKG_VERSION")`, which
   resolves to the prism-spec-engine LIBRARY crate version (not the product version),
   emitting `"prism/0.9.0"` on every outbound HTTP call to enrichment and threat-intel
   endpoints. Sensor tenants observing access logs would see `prism/0.9.0` on a
   beta.1 build — inconsistent with the product version.

ADR-064 D4 mandates addressing both surfaces. The human-directed S-1 decision
(2026-09-05) classifies this as v1.0.0-beta.1 scope for coherent product-version
self-identification.

---

## Narrative

As a release engineer, I want the prism MCP server to announce the correct product
version in its MCP initialize handshake, and all outbound HTTP calls from prism-spec-engine
to carry the correct product version in the User-Agent header, so that LLM agents and
sensor tenants both see coherent, accurate version identity on every v1.0.0-beta.1 build.

---

## Authority

- ADR-064 v1.8 D4 — Agent-Facing Version Surface Expansion (primary authority)
  - §Surface A: prism-mcp serverInfo.version runtime wiring mechanism
  - §Surface B: prism-spec-engine per-crate build.rs mechanism
- ADR-050 v2.4 D6 — `build_http_client_with_timeout` MUST use
  `env!("PRISM_VERSION")` (normative call updated from `CARGO_PKG_VERSION` in v2.4)
- ADR-022 §C — wiring not redesign; adding a parameter to a constructor that lacked it
  is permitted plumbing

(No BC: build-infrastructure + constructor-wiring change; conforming per S-REL-002
precedent.)

---

## Behavioral Contracts

This story has no subsystem behavioral contracts. Authority is ADR-064 D4 + ADR-050 D6.

| Architecture Source | Clause |
|---------------------|--------|
| ADR-064 v1.8 D4 §Surface A | `PrismServer` struct gains `product_version: &'static str` field; `PrismServer::with_deps` gains `product_version: &'static str` as final parameter; `PrismServer::new` uses `product_version: "0.0.0-test"` default (distinguishable from any release); `PrismServer::get_info` returns `Implementation::new("prism", self.product_version)` |
| ADR-064 v1.8 D4 §Surface A | `crates/prism-bin/src/boot.rs` boot step 9 `PrismServer::with_deps` call site passes `env!("PRISM_VERSION")` as final argument |
| ADR-064 v1.8 D4 §Surface B | New `crates/prism-spec-engine/build.rs` implements D2-conformant fallback chain (PRISM_BUILD_VERSION → GITHUB_REF_NAME gated on GITHUB_REF_TYPE=="tag" via `and_then` post-strip empty guard → CARGO_PKG_VERSION); emits `cargo:rustc-env=PRISM_VERSION={version}`; four `cargo:rerun-if-env-changed` directives |
| ADR-064 v1.8 D4 §Surface B | `build_http_client_with_timeout` in `crates/prism-spec-engine/src/pipeline.rs` changes `env!("CARGO_PKG_VERSION")` to `env!("PRISM_VERSION")` |
| ADR-050 v2.4 D6 | All three scoped outbound HTTP client builders emit `prism/{PRISM_VERSION}` user-agent; after D4 ships, full coherence across prism-bin (D2) + prism-spec-engine (D4) builders |
| ADR-022 §C | Adding `product_version: &'static str` to `PrismServer::with_deps` is wiring, not redesign — permitted plumbing; no Arc type replacement |

---

## Story-Level Holdout Gate

**3 HIDDEN, SINGLE-USE holdout scenarios (HS-032 group)** are stored under
`.factory/holdout-scenarios/` — the directory that test-writer and implementer
MUST NEVER READ. The holdout-evaluator runs these scenarios AFTER LOCAL 3-CLEAN
convergence and BEFORE demo recording/PR push.

Scenario files:
- `.factory/holdout-scenarios/S-REL-AGENT-VERSION-001-HS-001-mcp-serverinfo-version-wire-shape.md`
- `.factory/holdout-scenarios/S-REL-AGENT-VERSION-001-HS-002-spec-engine-user-agent-wire-header.md`
- `.factory/holdout-scenarios/S-REL-AGENT-VERSION-001-HS-003-coherent-product-version-both-surfaces.md`

The gate is **BLOCKING**: any unsatisfied scenario routes findings as OBSERVED BEHAVIOR
ONLY (no scenario text — contamination control) and resets the LOCAL 3-CLEAN streak.

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~4,000 |
| `crates/prism-mcp/src/server.rs` (PrismServer struct + with_deps + get_info) | ~3,000 |
| `crates/prism-bin/src/boot.rs` (boot step 9 region) | ~2,000 |
| `crates/prism-spec-engine/src/pipeline.rs` (`build_http_client_with_timeout`) | ~2,500 |
| ADR-064 D4 + ADR-050 D6 | ~2,500 |
| Total | ~14,000 |

Well within the 30% context window budget.

---

## Red Gate Test List (SAC-1)

Five Red Gate tests required. Write ALL as failing tests before any implementation.

**RED-THEN-GREEN is mandatory:** all RG-001..RG-005 must be written and confirmed FAILING
before any implementation task (Task 7 onwards) begins.

| ID | Test Name | What It Asserts | Crate |
|----|-----------|----------------|-------|
| RG-001 | `test_server_info_version_is_product_version` | `PrismServer::new().get_info().server_info.version` == `"0.0.0-test"` (the runtime-wired default set by `PrismServer::new`); fails RED because current `get_info` returns `Implementation::new("prism", "0.1.0")` — `"0.1.0"` != `"0.0.0-test"`. `env!("PRISM_VERSION")` MUST NOT appear in prism-mcp tests: prism-mcp has no build.rs (ADR-064 D4 §Surface A architecture compliance rule) | `prism-mcp` |
| RG-002 | `test_server_info_version_mcp_initialize_wire_shape` | WIRE-SHAPE assertion: the serialized `initialize` response JSON `serverInfo.version` field equals `"0.0.0-test"` (the `PrismServer::new` runtime default), not `"0.1.0"`; asserts on the actual JSON bytes per CLAUDE.md §Conventions §Wire-shape assertion discipline. `env!("PRISM_VERSION")` MUST NOT appear in prism-mcp tests (prism-mcp has no build.rs per ADR-064 D4 §Surface A) | `prism-mcp` |
| RG-003 | `test_prism_spec_engine_prism_version_env_var_is_available` | `env!("PRISM_VERSION")` compiles and is non-empty in prism-spec-engine context; fails RED (compilation error) before `crates/prism-spec-engine/build.rs` exists | `prism-spec-engine` |
| RG-004 | `test_spec_engine_build_rs_fallback_uses_cargo_pkg_version_when_no_env` | On local dev (no `GITHUB_REF_NAME`/`PRISM_BUILD_VERSION`), `env!("PRISM_VERSION")` == `env!("CARGO_PKG_VERSION")` in prism-spec-engine context; fails RED before build.rs exists | `prism-spec-engine` |
| RG-005 | `test_infusion_http_client_sends_prism_product_version_user_agent` | `build_http_client_with_timeout` constructs a client whose user-agent header string matches `concat!("prism/", env!("PRISM_VERSION"))`; fails RED because current code uses `env!("CARGO_PKG_VERSION")` (resolves to prism-spec-engine library version, not product version) | `prism-spec-engine` |

**BC-5.38.001 density check:** 5 Red Gate tests / 8 ACs = 0.625 — above the minimum 0.50
floor. The five tests gate: serverInfo.version Rust struct value (RG-001), serverInfo.version
wire-level JSON bytes (RG-002), build.rs compilation (RG-003), fallback chain semantics
(RG-004), and user-agent string value (RG-005).

---

## Tasks

**RED GATE FIRST — write ALL failing tests (Tasks 1–6) before any implementation.**

### Task 1: Write RG-001 (failing) — prism-mcp server_info struct assertion

In `crates/prism-mcp/src/server.rs` (or a new `crates/prism-mcp/tests/version_identity.rs`),
add:

```rust
#[test]
fn test_server_info_version_is_product_version() {
    let server = PrismServer::new(/* ... minimal construction ... */);
    let info = server.get_info();
    // prism-mcp has no build.rs — env!("PRISM_VERSION") MUST NOT appear here.
    // (ADR-064 D4 §Surface A: "No CI env-var resolution logic in library crates".)
    // PrismServer::new() sets product_version: "0.0.0-test" (the runtime-wired default).
    // Fails RED: current get_info returns Implementation::new("prism", "0.1.0");
    // "0.1.0" != "0.0.0-test", so the assertion below fails before any migration.
    assert_eq!(
        info.server_info.version,
        "0.0.0-test",
        "serverInfo.version via new() must equal \"0.0.0-test\" (runtime default); got {:?}",
        info.server_info.version
    );
}
```

Verify it FAILS with the pre-migration `"0.1.0"` value. Do NOT commit yet.

### Task 2: Write RG-002 (failing) — wire-shape JSON assertion

In the same test file, add a wire-shape test per CLAUDE.md §Conventions §Wire-shape
assertion discipline (mandatory for MCP-visible surfaces):

```rust
#[test]
fn test_server_info_version_mcp_initialize_wire_shape() {
    // Serialize the server info response to JSON and assert on the wire bytes.
    // This ensures the LLM agent sees the correct version at the protocol level,
    // not just at the Rust struct level (BC-2.11.001 null-not-absent wire discipline).
    // NOTE: prism-mcp has no build.rs — env!("PRISM_VERSION") MUST NOT appear here.
    // (ADR-064 D4 §Surface A: "No CI env-var resolution logic in library crates".)
    // PrismServer::new() injects product_version: "0.0.0-test" as the runtime default.
    let server = PrismServer::new(/* ... */);
    let info = server.get_info();
    let json = serde_json::to_string(&info).expect("server info must serialize");
    // Must NOT contain the stale hardcoded value:
    assert!(
        !json.contains("\"0.1.0\""),
        "wire output must not contain stale \"0.1.0\" version; got: {json}"
    );
    // Must contain the new() runtime default "0.0.0-test" (not env!("PRISM_VERSION"),
    // which MUST NOT appear in prism-mcp tests per ADR-064 D4 §Surface A):
    assert!(
        json.contains("\"0.0.0-test\""),
        "wire output must contain new() default \"0.0.0-test\"; got: {json}"
    );
}
```

Verify it FAILS because the current wire output contains `"0.1.0"`. Do NOT commit yet.

### Task 3: Write RG-003 (failing) — prism-spec-engine build.rs compilation gate

In `crates/prism-spec-engine/src/lib.rs` or a new
`crates/prism-spec-engine/tests/version_identity.rs`, add:

```rust
#[test]
fn test_prism_spec_engine_prism_version_env_var_is_available() {
    // build.rs must emit PRISM_VERSION; env!() fails to compile if absent.
    // This test fails with a compilation error before build.rs exists.
    let v: &str = env!("PRISM_VERSION");
    assert!(!v.is_empty(), "PRISM_VERSION must be non-empty");
}
```

Verify it FAILS to compile (compilation error: `PRISM_VERSION` not emitted by build.rs).
Do NOT commit yet.

### Task 4: Write RG-004 (failing) — fallback chain local-dev test

In the same spec-engine test file, add:

```rust
#[test]
fn test_spec_engine_build_rs_fallback_uses_cargo_pkg_version_when_no_env() {
    // In local dev (no GITHUB_REF_NAME), PRISM_VERSION must equal CARGO_PKG_VERSION
    // (the D2-conformant fallback chain step 3; ADR-064 D4 §Surface B).
    // After S-REL-DEV-RESET-001, CARGO_PKG_VERSION of prism-bin is "1.0.0-dev".
    // prism-spec-engine's own CARGO_PKG_VERSION is its library version, not "1.0.0-dev".
    // The fallback resolves to the LIBRARY's CARGO_PKG_VERSION; that's correct — it shows
    // the D2 chain is functioning even if the library value isn't the product version.
    // RG-003 already verifies non-empty. This test verifies fallback semantics.
    assert_eq!(env!("PRISM_VERSION"), env!("CARGO_PKG_VERSION"),
        "on local dev, PRISM_VERSION must fall through to CARGO_PKG_VERSION");
}
```

Verify it FAILS to compile (same reason: `PRISM_VERSION` not emitted). Do NOT commit yet.

### Task 5: Write RG-005 (failing) — spec-engine user-agent assertion

In the spec-engine test file, add a test exercising `build_http_client_with_timeout`'s
user-agent string:

```rust
#[test]
fn test_infusion_http_client_sends_prism_product_version_user_agent() {
    // build_http_client_with_timeout must use PRISM_VERSION (the product version)
    // not CARGO_PKG_VERSION (the library version "0.9.0").
    // This test verifies the user-agent constant compiled into the client builder.
    let expected_ua = concat!("prism/", env!("PRISM_VERSION"));
    // Access the user-agent constant from the builder or verify via the compiled string.
    // Implementer note: if build_http_client_with_timeout is not directly testable at
    // user-agent level without reqwest running, test via the compiled constant:
    assert!(
        expected_ua.starts_with("prism/"),
        "user-agent must start with prism/; got: {expected_ua}"
    );
    assert!(
        !expected_ua.contains("CARGO_PKG_VERSION"),
        "user-agent must not be the literal macro name"
    );
    // The actual value check: verify PRISM_VERSION is NOT the library crate version.
    // After build.rs ships, PRISM_VERSION will equal CARGO_PKG_VERSION of the
    // spec-engine library on local dev, not "0.9.0" (that's the stale value if the
    // old CARGO_PKG_VERSION site hadn't been migrated). This primarily verifies
    // the compilation path uses env!("PRISM_VERSION") not env!("CARGO_PKG_VERSION").
    let _ = expected_ua; // compilation succeeds only if PRISM_VERSION is emitted
}
```

Verify it FAILS to compile (no build.rs yet). Do NOT commit yet.

### Task 6: RED GATE CHECKPOINT

Confirm ALL five tests (RG-001..RG-005) are failing (either compile error or assertion
failure). Do NOT proceed to Task 7 until this checkpoint is verified.

Run: `just iter prism-mcp` — should show compilation errors.
Run: `just iter prism-spec-engine` — should show compilation errors.

If any test is accidentally passing, investigate before continuing.

---

### Task 7: Surface A — Add product_version field to PrismServer struct

In `crates/prism-mcp/src/server.rs`, locate `PrismServer` struct definition. Add:

```rust
pub struct PrismServer {
    // ... existing fields ...
    product_version: &'static str,
}
```

This is wiring, not redesign (ADR-022 §C; ADR-064 D4 §Surface A).

### Task 8: Surface A — Add product_version param to PrismServer::with_deps

Locate `PrismServer::with_deps` constructor. Add `product_version: &'static str` as the
**final parameter** (ADR-064 D4 §Surface A injection site table):

```rust
pub fn with_deps(
    // ... existing params ...
    product_version: &'static str,
) -> Self {
    Self {
        // ... existing fields ...
        product_version,
    }
}
```

### Task 9: Surface A — Update PrismServer::new with "0.0.0-test" default

Locate `PrismServer::new` (the test constructor). Add `product_version: "0.0.0-test"`:

```rust
pub fn new(/* ... */) -> Self {
    Self {
        // ... existing fields ...
        product_version: "0.0.0-test",
    }
}
```

The value `"0.0.0-test"` is distinguishable from any release and indicates test context
(ADR-064 D4 §Surface A; prevents tests from accidentally passing version assertions
against a production-like string).

### Task 10: Surface A — Update get_info to use self.product_version

Locate `PrismServer::get_info`. Change:

```rust
// Before (stale hardcoded value):
Implementation::new("prism", "0.1.0")

// After (runtime-wired from boot step 9):
Implementation::new("prism", self.product_version)
```

**Note on proc macro attribute:** The `#[tool_handler(version = "0.1.0")]` attribute
may still be present. This attribute accepts only string literals — `env!()` expressions
are not valid in this position per Rust proc macro rules. Since `get_info` is explicitly
overridden, the explicit return value is authoritative for the MCP handshake; the proc
macro attribute value is superseded. The implementer SHOULD check whether rmcp accepts
`env!("CARGO_PKG_VERSION")` in the attribute position; if the macro only accepts
`Lit::Str`, update the attribute to `"0.1.0"` with a code comment:

```rust
// #[tool_handler(version = "0.1.0")]
// NOTE: This proc macro attribute value is superseded by the explicit get_info override.
// The actual MCP serverInfo.version is set to self.product_version (threaded from boot step 9).
// env!() is not valid in proc macro attribute position (accepts Lit::Str only).
```

### Task 11: Surface A — Update boot.rs step-9 call site

Locate boot step 9 in `crates/prism-bin/src/boot.rs` — the `PrismServer::with_deps`
call site. Add `env!("PRISM_VERSION")` as the final argument:

```rust
// Before (no product_version arg):
PrismServer::with_deps(
    // ... existing args ...
)

// After (product_version threaded from D2 PRISM_VERSION env var):
PrismServer::with_deps(
    // ... existing args ...
    env!("PRISM_VERSION"),
)
```

`env!("PRISM_VERSION")` resolves at compile time in prism-bin context because
`crates/prism-bin/build.rs` (D2, S-REL-BVERSION-INJECT-001) emits `PRISM_VERSION`.
This is a prism-bin site, not a prism-mcp site — prism-mcp does NOT have its own build.rs;
it receives the version via the `&'static str` parameter.

Verify: `PrismServer::with_deps` call in boot.rs compiles with the new parameter.

### Task 12: Surface B — Create crates/prism-spec-engine/build.rs

Create a NEW file `crates/prism-spec-engine/build.rs` implementing the D2-conformant
fallback chain (ADR-064 D4 §Surface B: "The build.rs algorithm is identical to D2. The
D2 normative contract in this ADR is the single authoritative spec."):

```rust
fn main() {
    println!("cargo:rerun-if-env-changed=PRISM_BUILD_VERSION");
    println!("cargo:rerun-if-env-changed=GITHUB_REF_NAME");
    println!("cargo:rerun-if-env-changed=GITHUB_REF_TYPE");
    println!("cargo:rerun-if-env-changed=GITHUB_REF");

    let cargo_version = std::env!("CARGO_PKG_VERSION");

    // GITHUB_REF_NAME is set on ALL GitHub Actions runs — branch name on
    // push/pull_request events, tag name only on tag-push events.
    // Gate on GITHUB_REF_TYPE == "tag" to avoid baking "develop" / "feature/..."
    // into the binary on non-release CI runs (F-VID-P1-CRIT-001).
    let is_tag_build = std::env::var("GITHUB_REF_TYPE")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(|t| t.trim() == "tag")
        .unwrap_or_else(|| {
            std::env::var("GITHUB_REF")
                .ok()
                .filter(|s| !s.trim().is_empty())
                .map(|r| r.starts_with("refs/tags/"))
                .unwrap_or(false)
        });

    let version = std::env::var("PRISM_BUILD_VERSION")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            if is_tag_build {
                // Strip a SINGLE leading 'v' via strip_prefix (not trim_start_matches).
                // Post-strip empty guard: degenerate tag "v" → stripped "" → None →
                // falls through to CARGO_PKG_VERSION (ADR-064 v1.7 pass-5 OBS-1).
                std::env::var("GITHUB_REF_NAME")
                    .ok()
                    .filter(|s| !s.trim().is_empty())
                    .and_then(|r| {
                        let name = r.trim();
                        let stripped = name.strip_prefix('v').unwrap_or(name);
                        if stripped.is_empty() { None } else { Some(stripped.to_string()) }
                    })
            } else {
                None
            }
        })
        .unwrap_or_else(|| cargo_version.to_string());

    println!("cargo:rustc-env=PRISM_VERSION={}", version);
}
```

This is a self-contained implementation. No `include!` macro — prism-spec-engine is a
separate crate and cannot share `src/version_resolver.rs` from prism-bin at build time.
The algorithm is stable and small enough (20 lines) for parallel implementation.

### Task 13: Surface B — Migrate pipeline.rs user-agent

In `crates/prism-spec-engine/src/pipeline.rs`, locate `build_http_client_with_timeout`.
Change the user-agent call:

```rust
// Before (library version — incorrect product version):
.user_agent(concat!("prism/", env!("CARGO_PKG_VERSION")))

// After (product version from build.rs):
.user_agent(concat!("prism/", env!("PRISM_VERSION")))
```

ADR-050 v2.4 D6 normative form: `concat!("prism/", env!("PRISM_VERSION"))`.

### Task 14: Verify RG-001..RG-005 green

```bash
just iter prism-mcp     # RG-001, RG-002 must pass
just iter prism-spec-engine  # RG-003, RG-004, RG-005 must pass
```

If any RG test fails, diagnose and fix before proceeding.

### Task 15: Verify no CARGO_PKG_VERSION in pipeline.rs user-agent

```bash
grep 'CARGO_PKG_VERSION' crates/prism-spec-engine/src/pipeline.rs
```

Must return NO output at the user-agent call site in `build_http_client_with_timeout`.
(Other legitimate uses of `CARGO_PKG_VERSION` in the file, if any, are out of scope —
this migration targets only the user-agent site per ADR-064 D4.)

### Task 16: Run just check

```bash
just check
```

All workspace tests must pass. The new `crates/prism-spec-engine/build.rs` must not
break any existing tests. The `PrismServer::with_deps` signature change must compile
at all call sites (primarily `crates/prism-bin/src/boot.rs` boot step 9).

---

## Acceptance Criteria

### AC-001: PrismServer struct has product_version field
`crates/prism-mcp/src/server.rs` `PrismServer` struct contains `product_version: &'static str`
field. (traces to ADR-064 v1.8 D4 §Surface A injection site table — `PrismServer` struct row)

### AC-002: PrismServer::with_deps has product_version final parameter
`PrismServer::with_deps` signature includes `product_version: &'static str` as its final
parameter. (traces to ADR-064 v1.8 D4 §Surface A — `with_deps` row: "no version param" →
"`product_version: &'static str` final parameter added")

### AC-003: PrismServer::new uses "0.0.0-test" default
`PrismServer::new` sets `product_version: "0.0.0-test"`. The string `"0.0.0-test"` is
present in the `new` constructor body. (traces to ADR-064 v1.8 D4 §Surface A — `PrismServer::new`
row: "no version" → "`product_version: \"0.0.0-test\"` default")

### AC-004: get_info uses self.product_version
`PrismServer::get_info` calls `Implementation::new("prism", self.product_version)` (not
the stale `"0.1.0"`). `grep '"0.1.0"' crates/prism-mcp/src/server.rs` in the `get_info`
body returns no match. (traces to ADR-064 v1.8 D4 §Surface A — `get_info` row:
"`Implementation::new(\"prism\", \"0.1.0\")`" → `Implementation::new("prism", self.product_version)`)

### AC-005: boot.rs step-9 passes env!("PRISM_VERSION")
`PrismServer::with_deps` call site in `crates/prism-bin/src/boot.rs` (boot step 9) includes
`env!("PRISM_VERSION")` as the final argument. (traces to ADR-064 v1.8 D4 §Surface A —
`boot.rs` row: "no version arg" → `env!("PRISM_VERSION")` passed as final argument)

### AC-006: prism-spec-engine/build.rs exists with D2-conformant fallback chain
`ls crates/prism-spec-engine/build.rs` exits 0. File contains `PRISM_BUILD_VERSION`,
`GITHUB_REF_NAME`, `GITHUB_REF_TYPE`, `GITHUB_REF`, `CARGO_PKG_VERSION`, `is_tag_build`,
`and_then`, and `cargo:rustc-env=PRISM_VERSION` per the ADR-064 v1.7 D2 normative contract
(as required by ADR-064 D4 §Surface B: "The build.rs algorithm is identical to D2"). Four
`cargo:rerun-if-env-changed` lines present. (traces to ADR-064 v1.8 D4 §Surface B injection
site table — `crates/prism-spec-engine/build.rs` row: "(new file)" → D2-conformant build script)

### AC-007: pipeline.rs build_http_client_with_timeout uses PRISM_VERSION
`grep 'CARGO_PKG_VERSION' crates/prism-spec-engine/src/pipeline.rs` at the
`build_http_client_with_timeout` user-agent call site returns NO match. The user-agent
call uses `env!("PRISM_VERSION")` per ADR-050 v2.4 D6 normative form. (traces to
ADR-064 v1.8 D4 §Surface B — `pipeline.rs` row: "`env!(\"CARGO_PKG_VERSION\")`" →
"`env!(\"PRISM_VERSION\")`" AND ADR-050 v2.4 D6 normative call)

### AC-008: just check passes
All workspace tests pass after the Surface A + Surface B migrations. The new
`crates/prism-spec-engine/build.rs` integrates cleanly. The `PrismServer::with_deps`
parameter change compiles at all call sites. (traces to ADR-064 D4 §Consequences —
coherent product version across all four surfaces)

---

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| S-REL-DEV-RESET-001 | Set prism-bin Cargo.toml to `1.0.0-dev` | `crates/prism-bin/tests/version_identity.rs` test file | `just check` does not run cargo-semver-checks; `just iter <crate>` for fast inner loop |
| S-REL-BVERSION-INJECT-001 | D2 build.rs pattern (PRISM_BUILD_VERSION → GITHUB_REF_NAME tag-gated → CARGO_PKG_VERSION); 4 rerun-if-env-changed; and_then post-strip empty guard; shared-resolver structure in prism-bin | D2 normative contract is the single authoritative spec for any D4 build.rs | prism-spec-engine build.rs must NOT use include! from prism-bin (separate crate; build.rs runs in prism-spec-engine context only). build.rs uses std::env::var() at build time, not env!() (which runs at compile-of-library time). |

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| product_version threaded via PrismServer::with_deps, NOT resolved inside prism-mcp | ADR-064 D4 §Surface A "No CI env-var resolution logic in library crates" | prism-mcp/build.rs must NOT exist; no env!("PRISM_VERSION") in prism-mcp source |
| boot.rs step-9 passes env!("PRISM_VERSION") (from D2 build.rs in prism-bin) | ADR-064 D4 §Surface A boot.rs row | AC-005 grep assertion |
| prism-spec-engine/build.rs algorithm identical to D2 normative contract | ADR-064 D4 §Surface B "The D2 normative contract in this ADR is the single authoritative spec" | AC-006 verifies presence of D2-canonical identifiers |
| pipeline.rs user-agent uses PRISM_VERSION not CARGO_PKG_VERSION | ADR-050 v2.4 D6 + ADR-064 D4 §Surface B | AC-007 grep check |
| PrismServer::new uses "0.0.0-test" (not a real version) | ADR-064 D4 §Surface A "distinguishable from any release, indicates test context" | AC-003 |
| Wire-shape assertion required for MCP-visible surface | CLAUDE.md §Conventions §Wire-shape assertion discipline (RG-002 mandate) | RG-002 asserts on serialized JSON bytes |
| PRISM_VERSION never empty string | ADR-064 v1.7 D2 (pass-5 OBS-1; and_then post-strip guard for degenerate "v" tag) | EC-002 covers this; and_then in build.rs enforces it |

---

## Library & Framework Requirements

| Dependency | Version | Notes |
|------------|---------|-------|
| Rust toolchain | Per `rust-toolchain.toml` | No change; both build.rs files use std only |
| `std::env::var` | std | Fallback chain reads env vars at build time in build.rs |
| `cargo:rustc-env` | Cargo build script protocol | Emits compile-time env var for env!() macro |
| rmcp (prism-mcp) | Workspace version | No version change; PrismServer struct field addition only |

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-mcp/src/server.rs` | Modify | Add product_version field + with_deps param + new default + get_info change |
| `crates/prism-bin/src/boot.rs` | Modify | Boot step 9 call site: add env!("PRISM_VERSION") as final arg to with_deps |
| `crates/prism-spec-engine/build.rs` | Create | D2-conformant build script; new file |
| `crates/prism-spec-engine/src/pipeline.rs` | Modify | build_http_client_with_timeout user-agent: CARGO_PKG_VERSION → PRISM_VERSION |
| `crates/prism-mcp/tests/version_identity.rs` | Create (or extend) | RG-001 + RG-002 tests |
| `crates/prism-spec-engine/tests/version_identity.rs` | Create (or extend) | RG-003 + RG-004 + RG-005 tests |

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `crates/prism-spec-engine/build.rs` | build script | Pure (reads env vars at build time; emits compile-time directive) |
| `PrismServer::product_version` field | `crates/prism-mcp/src/server.rs` | Pure (compile-time constant threaded via constructor) |
| `PrismServer::get_info` | `crates/prism-mcp/src/server.rs` | Pure (returns `ServerInfo` struct; no I/O) |
| boot step 9 `PrismServer::with_deps` call | `crates/prism-bin/src/boot.rs` | Effectful (boot lifecycle; wires runtime deps) |
| `build_http_client_with_timeout` user-agent | `crates/prism-spec-engine/src/pipeline.rs` | Effectful (reqwest HTTP client construction) |

---

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `crates/prism-spec-engine/build.rs` | pure-core | Reads env vars and emits build directives only; no I/O at runtime |
| `PrismServer` product_version field | pure-core | `&'static str` compile-time constant; no heap allocation |
| `PrismServer::get_info` | pure-core | Returns a struct value derived from compile-time constants; no side effects |
| `crates/prism-bin/src/boot.rs` (step 9) | effectful-shell | Boot lifecycle; spawns subsystem dependencies |
| `build_http_client_with_timeout` | effectful-shell | Constructs reqwest::Client; I/O-capable |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | prism-mcp test code uses `PrismServer::new` (not `with_deps`) | `product_version = "0.0.0-test"` default ensures tests always get a non-production string; tests asserting `env!("PRISM_VERSION")` must use the `with_deps` constructor path or explicitly pass the expected version |
| EC-002 | prism-spec-engine/build.rs degenerate tag `"v"` | `and_then` post-strip guard: `"v"` → stripped `""` → `None` → falls through to `CARGO_PKG_VERSION` (ADR-064 v1.7 pass-5 OBS-1); `PRISM_VERSION` is never empty |
| EC-003 | GITHUB_REF_NAME is set to branch name (`develop`, `feature/...`) on non-tag CI run | `is_tag_build` is `false`; `GITHUB_REF_NAME` arm skipped entirely; `PRISM_VERSION` falls through to `CARGO_PKG_VERSION` (D2-conformant, ADR-064 F-VID-P1-CRIT-001) |
| EC-004 | All three env vars absent in prism-spec-engine build (local dev) | Falls through to `CARGO_PKG_VERSION` — the spec-engine library's own crate version (not the product version); this is correct locally since the MCP wire output is determined by the boot step 9 param, not the spec-engine build |
| EC-005 | Product_version field changes across PrismServer instances | Field is `&'static str` — value is fixed at compile time; all instances within a binary share the same product version injected at boot step 9 |

---

## Downstream Story Impact (TD-VSDD-097 Dim-2)

S-REL-BVERSION-INJECT-001 AC-004 ("prism-spec-engine/pipeline.rs user-agent NOT changed")
is a downstream copy of the ADR-064 D2 out-of-scope scope boundary. That boundary is
superseded by D4. See the AC-004 amendment appended to
`S-REL-BVERSION-INJECT-001-build-rs-prism-version-injection.md` §AC-004 per ADR-064 v1.8
D4 "TD-VSDD-097 Dim-2 flagged" directive:

- **UNTIL S-REL-AGENT-VERSION-001 merges:** AC-004 of S-REL-BVERSION-INJECT-001 remains
  ACTIVE as a guard against premature migration in that story's scope.
- **AFTER S-REL-AGENT-VERSION-001 merges:** AC-004 is inverted — the guard confirms the
  migration DID occur (owned by this story). The AC remains in S-REL-BVERSION-INJECT-001
  with the note updated per its amendment.

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.1 | 2026-09-06 | story-writer | OBS-2 fix (LOCAL adversary): corrected Task 1 / Task 2 prism-mcp test snippets and RG-001 / RG-002 table rows to assert `"0.0.0-test"` (the `PrismServer::new` runtime-wired default) instead of `env!("PRISM_VERSION")`. Consistent with ADR-064 D4 §Surface A architecture compliance rule: prism-mcp is a library crate with no build.rs; `env!("PRISM_VERSION")` MUST NOT appear in prism-mcp source or tests. Implementation was already correct; only the story task prose and RG table rows were stale. |
| 1.0 | 2026-09-05 | product-owner | Initial authoring. ADR-064 D4 materialization (S-1 human-directed beta.1 scope). Surface A: PrismServer runtime wiring + boot.rs step-9 pass-through. Surface B: prism-spec-engine/build.rs D2-conformant per-crate injection + pipeline.rs user-agent migration. SAC-1 compliant: RG-001..RG-005 enumerated, density 5/8=0.625, red-then-green ordering enforced. AC-004 amendment applied to S-REL-BVERSION-INJECT-001. 3 holdout scenarios HS-032-001..003 authored. ADR-050 v2.4 D6 traced. TD-VSDD-097: Dim-1 CLEAR (S-REL-BVERSION-INJECT-001 is the sibling story; swept and amended AC-004 in same burst per anchor-back rule). Dim-2 CLEAR (no verbatim copy-source section introduced). Dim-3: all MUSTs anchored to story ACs + ADR-064 D4 authority. |
