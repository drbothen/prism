# component_model_dispatch fixture — Build Recipe

This document records the authoritative build recipe for the
`tests/fixtures/component_model_dispatch.prx` Component Model binary fixture.

## Source files

| File | Purpose |
|------|---------|
| `component_model_dispatch.wit` | WIT interface (`prism:dispatch-test@0.1.0` world) |
| `component_model_dispatch.core.wat` | Core wasm module source (canonical ABI imports + call-blocked impl) |

## Toolchain requirement

- **wasm-tools 1.248.0** (the version used to build the committed binary)

Install:

```
cargo install wasm-tools --version 1.248.0
```

Or check current version:

```
wasm-tools --version
```

## Build recipe (from workspace root)

```bash
# Step 1: Embed WIT metadata into the core WAT module.
# This annotates the core .wat with the WIT world type information.
wasm-tools component embed \
  --world dispatch-test \
  tests/fixtures/src/component_model_dispatch.wit \
  tests/fixtures/src/component_model_dispatch.core.wat \
  -o /tmp/component_model_dispatch.embedded.wasm

# Step 2: Create the Component Model binary from the embedded module.
# This wraps the core module with canonical ABI adapters (shim + fixup modules).
wasm-tools component new \
  /tmp/component_model_dispatch.embedded.wasm \
  -o tests/fixtures/component_model_dispatch.prx
```

The two-step process matches wasm-tools component embed + component new semantics.
Equivalent `just` target:

```bash
just build-fixture-component_model_dispatch
```

## Verification

After rebuilding, verify the fixture is a valid Component Model binary:

```bash
# Check magic bytes (should print component type)
wasm-tools print tests/fixtures/component_model_dispatch.prx | head -5

# Extract WIT to verify interface is correct
wasm-tools component wit tests/fixtures/component_model_dispatch.prx

# Run the load-bearing test
cargo nextest run -p prism-spec-engine \
  --test plugin_integration_tests \
  --features test-helpers \
  -E 'test(test_F_PASS5_HIGH_001)'
```

Expected WIT output:

```
package root:component;

world root {
  import host: interface {
    record http-response {
      status: u16,
      headers: list<tuple<string, string>>,
      body: list<u8>,
    }

    http-request: func(method: string, url: string, headers: list<tuple<string, string>>, body: option<list<u8>>) -> http-response;
  }

  export call-blocked: func() -> u16;
}
```

## When to rebuild

Rebuild `component_model_dispatch.prx` when:

1. The wasmtime version is bumped and the binary format changes (wasmtime rejects old binary).
2. The production `host::http-request` interface changes (different parameter types or record fields).
3. wasm-tools introduces a new canonical ABI encoding that invalidates the old binary.

After rebuilding, commit the new `.prx` binary along with any source file changes.

## Size reference

The committed binary is approximately 1314 bytes (rebuilt from documented WAT sources
in fix-burst-impl-6). Significant deviations suggest a toolchain change affected the
output format. Byte-exact reproducibility is not guaranteed across wasm-tools versions;
functional equivalence (same WIT interface, same behavioral contract) is the goal.

## Test traceability

- Test: `test_F_PASS5_HIGH_001_production_linker_dispatch_via_build_linker_route_a`
- File: `crates/prism-spec-engine/tests/plugin_integration_tests.rs`
- BC: BC-2.17.001 (http-request host function)
- Finding closed: F-PASS5-HIGH-001 (production-linker dispatch test via Route A)
- Fixture sources committed: fix-burst-impl-6 (F-PASS6-MED-001)
