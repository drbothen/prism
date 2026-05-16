---
document_type: adr
adr_id: "ADR-026"
title: "SensorAuth Trait Un-Sealing — Remove private::Sealed, Enable Plugin Auth Implementations"
status: Proposed
date: "2026-05-15"
version: "1.1"
producer: architect
subsystems_affected: [SS-01, SS-17, SS-16]
supersedes: null
superseded_by: null
amends: ADR-023
anchor_stories: [S-PLUGIN-PREREQ-E]
runtime_deliverables:
  - "Remove private::Sealed and mod private from crates/prism-sensors/src/auth/mod.rs"
  - "Remove #[non_exhaustive] seal-workaround doc comments from SensorAuth"
  - "Add SensorAuth re-export to prism-sensors public API surface"
  - "Delete CustomAuth placeholder duplicate from crates/prism-spec-engine/src/custom_adapter.rs"
  - "Validate PluginRuntime::load_plugin wiring path calls SensorAuth-implementing types"
wiring_deferred_to: null
---

# ADR-026: SensorAuth Trait Un-Sealing — Remove private::Sealed, Enable Plugin Auth Implementations

## Status

Proposed 2026-05-15, v1.0. Governs the PLUGIN-PREREQ-E delivery of the SensorAuth unsealing
(Constraint C5 per ADR-023 §Architectural Constraints). Implementation is tracked by
S-PLUGIN-PREREQ-E.

---

## Context

ADR-023 Rule 2 (SensorAuth Trait Un-Sealing) mandated removal of the `private::Sealed` marker
in `crates/prism-sensors/src/auth/mod.rs`. The audit (PLUGIN-AUDIT-001, 2026-05-10) surfaced
that `SensorAuth` is sealed via `private::Sealed`, which actively prevents plugin authors
from implementing the trait. This makes the "external TOML authorship" and "plugin
extensibility" claims false at the type level: no plugin can provide its own auth
implementation because the trait bound is unreachable from outside the crate.

ADR-023 established the enforcement model transition (DI-012 amendment): cross-sensor
auth-composition prevention moves from compile-time type-system enforcement to runtime
spec-validation enforcement at TOML load time. This ADR specifies the concrete design
decisions for the un-sealing operation that ADR-023 mandated but did not fully detail.

### Current State (as-built, 2026-05-10)

```
crates/prism-sensors/src/auth/mod.rs:
  mod private { pub trait Sealed {} }
  pub trait SensorAuth: private::Sealed + Send + Sync + 'static { ... }
  // Four concrete types: CrowdStrikeAuth, CyberintAuth, ClarotyAuth, ArmisAuth
  // Each implements private::Sealed, making them the only valid implementors

crates/prism-spec-engine/src/custom_adapter.rs:
  pub trait SensorAuth: Send + Sync {}
  // Placeholder duplicate — created because sealed trait is unreachable from spec-engine
```

The `CustomAuth` duplicate in `custom_adapter.rs` is itself evidence of the unsealing
need: the spec-engine required a parallel auth surface ONLY because it could not implement
the sealed one.

### BC-2.17.* Plugin Loader Contract (reference)

The BC-2.17 trio (BC-2.17.001 panic isolation, BC-2.17.003 memory limit, BC-2.17.007
manifest schema validation) governs the WASM plugin lifecycle. Plugin auth implementations
must comply with these BCs: plugin-provided `SensorAuth` impls are loaded by `PluginRuntime`,
so they inherit the sandbox constraints (no raw WASI network, memory limit, panic isolation).

---

## Decision

### D1 — Remove private::Sealed entirely

The `mod private` block and `private::Sealed` supertrait bound are removed from
`crates/prism-sensors/src/auth/mod.rs`. The revised `SensorAuth` definition is:

```rust
/// Sealed authentication credential for a sensor adapter.
///
/// Implementors may be defined in any crate — plugin authors implement this trait
/// to provide custom auth flows. Runtime cross-sensor composition is prevented by
/// spec-validation rules (ADR-023 Rule 2; ADR-026 D3) rather than compile-time sealing.
///
/// Credentials MUST NOT appear in `Debug` output or log output at any level
/// (AI-opaque credential model per AD-017).
///
/// Story: S-PLUGIN-PREREQ-E | BC-2.01.016 (primary) | BC-2.01.013 (parent pattern) | ADR-026
pub trait SensorAuth: Send + Sync + 'static {
    fn as_any(&self) -> &dyn std::any::Any;
    fn auth_type_name(&self) -> &'static str;
}
```

This makes `SensorAuth` a standard open trait with no crate-private supertrait. Any crate —
including `.prx` WASM plugin host shim crates — may implement it.

### D2 — Keep existing internal impls unchanged

`CrowdStrikeAuth`, `CyberintAuth`, `ClarotyAuth`, and `ArmisAuth` in
`crates/prism-sensors/src/auth/{crowdstrike,cyberint,claroty,armis}.rs` implement `SensorAuth`
and continue to do so. Their `impl SensorAuth` bodies are unaffected by the unsealing.
The four concrete types are scheduled for deletion in Wave 1/A (PLUGIN-MIGRATION-001-A), but
they must remain functional through the Wave 0 prereq window. The unsealing operation does NOT
delete the internal impls.

**Timing model:** internal impls exist and compile → sealed trait removed → external impls
now possible → internal impls deleted at Wave 1/A. No intermediate broken state.

### D3 — Runtime cross-sensor auth-composition prevention (DI-012 replacement enforcement)

The three runtime rules that replace compile-time sealing (ADR-023 Rule 2) are enforced by
`prism-spec-engine`'s TOML load-time validation:

1. `auth_type` in a sensor spec must be a single value from the enumerated set
   `{oauth2_client_credentials, bearer_static, cookie_roundtrip, api_key, custom_via_plugin}`.
   Multiple values or values outside the enumerated set are rejected at spec-load time.
   Error code: **E-SPEC-012** (auth_type cross-composition; see §Error Code Assignment Note below).
2. `credential_refs` must reference exactly one credential per auth method.
   Multiple bindings per auth method are rejected at spec-load time.
   Error code: **E-SPEC-013** (multiple credential_refs per auth method).
3. The resolved credential type must structurally match the spec's `auth_type` variant.
   Mismatches are rejected at credential-resolution time, before any HTTP request.
   Error code: **E-SPEC-014** (auth_type / credential structural mismatch).

These three checks happen in `spec_parser.rs` or `pipeline.rs` validation pass, before any
HTTP request. They are the load-time gates that prevent the same cross-composition attacks the
sealed trait prevented at compile time.

**Error Code Assignment Note:** E-SPEC-010 is already allocated to "Variable interpolation
failed" and E-SPEC-011 to "Reserved keyword pipe_verb" in `error-taxonomy.md`. The auth-type
rejection rules require three NEW codes: E-SPEC-012, E-SPEC-013, E-SPEC-014. These codes
MUST be added to `error-taxonomy.md` by the product-owner in S-PLUGIN-PREREQ-E scope before
the test-writer authors AC-3 and VP-153 harness tests. This is a routed handoff (not a defer);
the product-owner owns `error-taxonomy.md` per Agent Routing Table. The story `inputs:` list
already cites `error-taxonomy.md` — this is an in-scope amendment.

Verification: VP-153 (this ADR) specifies a proptest harness over `(auth_type, credential_type)`
mismatches that verifies load-time rejection for all invalid combinations. VP-153 §Property
Statement Rule A references E-SPEC-012; its current "E-CFG-NNN placeholder" notation is
resolved to E-SPEC-012 pending PO's error-taxonomy amendment (see VP-153 §Open Issues).

### D4 — Delete CustomAuth placeholder

The `pub trait SensorAuth: Send + Sync {}` duplicate in
`crates/prism-spec-engine/src/custom_adapter.rs` is deleted as part of S-PLUGIN-PREREQ-E.
Once `prism-sensors::auth::SensorAuth` is unsealed, `custom_adapter.rs` can (in principle)
import the real trait; but since `custom_adapter.rs` is also being deleted in PREREQ-E (Rule 5
of ADR-023), the practical resolution is: delete both together in the same commit.

### D5 — Dispatch model: Box<dyn SensorAuth> at runtime

Plugin-provided auth implementations are dispatched as `Box<dyn SensorAuth>` at runtime.
This is consistent with the existing internal dispatch path in `SensorAdapter::auth()`, which
returns `&dyn SensorAuth`. No generics-based monomorphization is required or desirable here:

- Plugin implementations are loaded at runtime from WASM, so the concrete type is never known
  at compile time. A generic `fn execute<A: SensorAuth>()` cannot be instantiated across a
  WASM ABI boundary.
- Vtable dispatch overhead for auth credential access is negligible compared to the HTTP
  round-trip latency of an actual auth token acquisition. This is not a hot path.
- `Box<dyn SensorAuth>` matches the existing `Box<dyn CustomAdapter>` dispatch pattern
  already in the codebase; no new patterns introduced.

### D7 — WriteToolInvalidationMap container type: RwLock<Vec<...>> (no OnceLock)

`WriteToolInvalidationMap` in `crates/prism-query/src/invalidation.rs` (TD-A-003 closure) uses
`std::sync::RwLock<Vec<WriteToolInvalidationMap>>` initialized at program start as
`RwLock::new(Vec::new())`. **No `OnceLock<RwLock<...>>` wrapper is needed.**

Boot-step rationale (ADR-022 §B, step table): plugin-load is step 7.5 (BLOCKING); query-engine
init is step 8 (BLOCKING); MCP traffic is gated until step 8 completes. Write calls
(`register_write_tool`) happen exclusively during step 7.5 plugin-load. Read calls (invalidation
check on the query hot path) happen only during step 8+ query execution. The write window is
fully closed before any reader can acquire a read guard — the ADR-022 boot ordering enforces
this at the structural level. There is no initialization-race risk that `OnceLock` would
resolve: the `RwLock` is fully initialized at binary start before any thread acquires it.

`OnceLock<RwLock<...>>` would be warranted only if lazy initialization were needed (e.g., the
container must not allocate until first use). For a boot-time-written, query-time-read structure,
eager initialization with `RwLock::new(Vec::new())` is simpler, more readable, and avoids the
`OnceLock::get_or_init` unwrap pattern that can panic if called before init in test contexts.

**API:** `pub fn register_write_tool(entry: WriteToolInvalidationMap)` acquires `RwLock::write()`,
pushes the entry, and releases the guard. All read-side callers use `RwLock::read().unwrap()`
(infallible if no writer panics while holding the lock; boot-phase write calls are synchronous
and non-panicking by production-grade default). A `WARN`-level tracing event is emitted if
`register_write_tool` is called after step 8 starts (detected via an `AtomicBool` query-phase
flag set by the query engine init).

Anchor: ADR-022 §B step 7.5 (plugin-load before query-engine init). BC-2.16.012 postcondition
INV-INVALIDATION-EXT-001 (TD-A-003 closure). S-PLUGIN-PREREQ-E AC-9.

### D6 — #[non_exhaustive] and pub visibility

`SensorAuth` is `pub` in `prism-sensors`, accessible from any crate in the workspace and
from downstream consumers. It is NOT marked `#[non_exhaustive]` — `#[non_exhaustive]` applies
to types (structs, enums) that can be pattern-matched; traits do not support this attribute.
New required methods added to `SensorAuth` are a breaking change; any such addition requires
a new ADR and a semver bump (the trait is part of the plugin ABI surface once PREREQ-E ships).
This constraint is documented in the `SensorAuth` trait doc comment.

**Soundness gates:**
- Plugin-provided impls run inside the WASM sandbox per BC-2.17.001 (panic isolation).
  A panicking plugin `SensorAuth` impl cannot terminate the host process.
- `as_any()` downcast is available for concrete type recovery where needed; impls that return
  an incorrect concrete type from `as_any()` produce a failed downcast (`None`), not UB.
- `SensorAuth: 'static` is required because auth credentials are stored in `Arc<dyn SensorAuth>`
  and must outlive the token-acquisition call stack.

---

## Rationale

**Open trait is the correct primitive.** The sealed-trait pattern exists to prevent unintended
external implementations. In the plugin architecture, external implementations are the entire
point. The sealed trait directly contradicts the declared architecture goal of "plugin authors
implement SensorAuth." Removing the seal is the minimum necessary change to enable the goal.

**Runtime enforcement is semantically equivalent.** The sealed trait's threat model was: prevent
cross-sensor credential routing (e.g., CrowdStrike OAuth tokens routed through Cyberint cookie
middleware). The three load-time validation rules in D3 reproduce this invariant with the same
behavioral outcome — invalid combinations are rejected before any token is acquired or any HTTP
request is issued. Enforcement moves earlier in some respects (spec load versus first request)
and later in others (not caught at compile time). The net risk posture is equivalent.

**Box<dyn SensorAuth> over generics.** Generics-based dispatch requires knowing the concrete
type at compile time. WASM plugin-provided types do not exist in the host binary's type system;
they are instantiated at runtime through the WASM component interface. `Box<dyn SensorAuth>` is
the only viable dispatch mechanism for this architecture.

**No deprecation period for internal impls.** ADR-023 Rule 5 confirmed: `prism-spec-engine`
has never been published to crates.io with `CustomAdapter` or the sealed `SensorAuth` exposed
externally. No external consumers exist. The unsealing and simultaneous `CustomAuth` deletion
are safe in a single burst.

---

## Consequences

### Positive

- Plugin authors can implement `SensorAuth` for custom auth flows (bearer token from secrets
  manager, mTLS client-cert flow, HMAC-signed request auth, etc.)
- `CustomAuth` placeholder in `custom_adapter.rs` is deleted — the duplicate is gone
- DI-012 threat model is preserved with runtime enforcement that is at least as strong
- The `SensorAuth` trait API is now part of the plugin ABI surface, documented as such
- VP-153 proptest harness provides ongoing regression coverage for the runtime enforcement rules

### Negative / Trade-offs

- `SensorAuth` additions are now breaking changes for plugin consumers. Future method additions
  require providing a default impl (e.g., `fn extra_method(&self) -> X { default }`) or a new
  ADR + semver bump. This constraint is documented in the trait's module-level rustdoc.
- Compile-time guarantee of DI-012 is lost. The runtime enforcement substitute is correct
  (see D3) but cannot be caught by the compiler. Adversarial review of the spec-validation
  code path (VP-153 harness) is the compensating control.
- If a plugin provides a `SensorAuth` impl that passes spec validation but produces malformed
  tokens at runtime, the error propagates to the HTTP layer (401 from the sensor API). This is
  the same failure mode as any credential misconfiguration. The BC-2.17.001 panic isolation
  guarantees the host process remains alive.

---

## Verification Property Anchors

- **VP-153** — `SensorAuth` runtime cross-composition prevention: proptest over the Cartesian
  product of `(auth_type, credential_type)` pairs verifies that all invalid combinations are
  rejected at spec-load time and all valid combinations are accepted. Module: prism-spec-engine.
  Method: proptest. Priority: P0. Anchor story: S-PLUGIN-PREREQ-E.
  Primary BC anchor: **BC-2.01.016** (SensorAuth Open Trait — Plugin-Implementable Auth Contract).
  Parent pattern BC: BC-2.01.013 (DataSource Trait Adapter Pattern, amended by ADR-023 to remove
  sealed-trait language — not amended further in PREREQ-E; BC-2.01.016 is the new open-auth-trait
  contract that operationalizes the ADR-023 amendment).

---

## Alternatives Considered

**Option A: Keep sealing; expose a separate PluginSensorAuth trait.** Rejected. Two parallel
auth traits that serve the same purpose create the exact problem already observed
(`SensorAuth` sealed + `CustomAuth` duplicate). The plugin ABI must use the same type surface
as the internal adapters; a separate trait would require conversion shims at every call site.

**Option B: Generics-based dispatch (fn execute<A: SensorAuth>(auth: A)).**  Rejected.
WASM plugin ABI does not support monomorphized generic instantiation at the host boundary.
Plugin types are not known at compile time of the host crate. `Box<dyn Trait>` is the correct
pattern.

**Option C: Keep sealed; pass credentials as opaque byte blobs across the plugin boundary.**
Rejected. Opaque byte blobs require a serialization protocol (JSON, protobuf) for credentials,
which must be designed and versioned. This adds a new protocol surface with its own failure
modes and security implications. The open trait approach reuses the existing type system.

---

## Source / Origin

- ADR-023 Rule 2 — SensorAuth Trait Un-Sealing (mandate for this decision)
- ADR-023 §C5 — PLUGIN-PREREQ-E scope (three dead-code call sites)
- `crates/prism-sensors/src/auth/mod.rs` — sealed trait current implementation
- `crates/prism-spec-engine/src/custom_adapter.rs` — `CustomAuth` placeholder duplicate
- PLUGIN-AUDIT-001 (2026-05-10) — surfaced the sealed-trait / CustomAuth duplication
- BC-2.17.001, BC-2.17.003, BC-2.17.007 — plugin sandbox contracts bounding plugin auth impls
- DI-012 — sealed-auth-trait domain invariant (downgraded to runtime enforcement per ADR-023)
- **BC-2.01.016** — SensorAuth Open Trait (NEW; authored in S-PLUGIN-PREREQ-E by product-owner;
  primary behavioral contract for the auth surface opened by this ADR; supersedes the
  "auth-trait amendment" role originally attributed to BC-2.01.013 for this ADR's scope)
- BC-2.01.013 — DataSource Trait Adapter Pattern (parent pattern; amended in PREREQ-F to remove
  sealed-trait language per ADR-023 Rule 2; no further amendment in PREREQ-E; BC-2.01.016
  carries the open-auth-trait contract going forward)

---

## Related ADRs

| ADR | Relationship |
|-----|-------------|
| **ADR-023** | This ADR is the detailed specification of ADR-023 Rule 2 / Constraint C5 for SensorAuth unsealing |
| **ADR-022** | Boot sequence — SensorAuth-implementing types are wired via PluginRuntime at boot step 7.5 |
| **ADR-027** | CustomAdapter retirement — complements this ADR by specifying the deprecation/deletion pathway |

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-05-15 | architect | Initial proposal — SensorAuth unsealing design for S-PLUGIN-PREREQ-E |
| 1.1 | 2026-05-15 | architect | Q1 resolution: add D7 (WriteToolInvalidationMap RwLock<Vec<...>> — no OnceLock; boot-step 7.5 contract cited). Q2 resolution: add BC-2.01.016 as primary VP-153 BC anchor; BC-2.01.013 noted as parent pattern; no amendment to BC-2.01.013 in PREREQ-E. Q3 resolution: D3 revised to assign E-SPEC-012/013/014 for auth-type rejection rules; E-SPEC-010 collision documented; error-taxonomy amendment routed to PO. §Source/Origin amended to reference BC-2.01.016. |
