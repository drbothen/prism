---
document_type: verification-property
level: L4
version: "1.0"
status: draft
producer: architect
timestamp: 2026-04-20T00:00:00Z
phase: 2-patch
inputs:
  - specs/prd.md
  - specs/verification-properties/VP-INDEX.md
  - specs/behavioral-contracts/BC-2.17.002
input-hash: "3ff257e"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.17.002
module: prism-spec-engine
priority: P1
proof_method: kani
verification_method: kani
feasibility: conditional
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: cycle-2-patch
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-040: Plugin Linker Excludes All WASI Namespace Imports

## Property Statement

`PluginRuntime::build_linker()` produces a `wasmtime::component::Linker<HostState>` whose
complete set of linked import namespace names contains no entry with the prefix `wasi:`.
Any WASM Component that requires a WASI import will fail at `instantiate_pre` time rather
than at runtime, enforcing the sandbox constraint statically at linker construction.

## Source Contract

- **Anchor Story:** `S-1.15`
- **Source BC:** BC-2.17.002 — Plugin Sandbox: No Filesystem/Network
- **Module:** prism-spec-engine
- **Category:** Security / Sandboxing

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| kani (conditional) | Kani (latest) | Yes — all linker build code paths | All registered import namespaces |

**Feasibility caveat:** Kani-feasible only if `wasmtime::component::Linker` exposes an
import enumeration API visible to the model checker. If the `Linker` type uses opaque
`unsafe` internals that block symbolic reasoning, downgrade to Proptest: attempt
instantiation of a WASI-importing component binary and assert `Err(PluginError::SandboxViolation)`.
Phase 3 story author must confirm wasmtime API visibility before committing to Kani method.

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: kani (fallback: proptest if Linker is opaque)
// Target: prism_spec_engine::plugin::PluginRuntime::build_linker
//
// Kani sketch:
//   let engine = Engine::new(&Config::default()).unwrap();
//   let linker = PluginRuntime::build_linker(&engine);
//   let namespaces: Vec<&str> = linker.component().imports(); // hypothetical enumeration API
//   for ns in &namespaces {
//       kani::assert(!ns.starts_with("wasi:"), "WASI import must not appear in linker");
//   }
//
// Proptest fallback sketch:
//   proptest!(|(wasi_component_bytes in arb_wasi_component_binary())| {
//       let result = PluginRuntime::load_component(&wasi_component_bytes);
//       assert!(result.is_err(), "WASI-importing component must be rejected at load");
//       let err = result.unwrap_err();
//       assert!(matches!(err, PluginError::SandboxViolation), "Error must be SandboxViolation");
//   });
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | Fixed linker build function; no user-controlled branching |
| Tool support? | Conditional | Kani requires Linker import enumeration; fallback to proptest is fully feasible |
| Execution time budget | <5 minutes | Linker construction is cheap; small bounded space |
| Assumptions required | wasmtime Linker exposes import namespace list to Kani | If not, proptest path is the fallback |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-20 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.0 | pass-69-housekeeping | 2026-04-20 | architect | Initial draft. Resolves VP-TBD in BC-2.17.002. Method: Kani conditional on wasmtime Linker API visibility; proptest fallback documented. |
