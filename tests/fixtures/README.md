# tests/fixtures

This directory contains binary fixtures required for the WASM plugin build pipeline.

## `wasi_snapshot_preview1.wasm`

**Source:** Wasmtime 44.0.1 release — WASI Preview 1 reactor adapter
(`wasi_snapshot_preview1.wasm` from the Wasmtime GitHub releases page at
https://github.com/bytecodealliance/wasmtime/releases/tag/v44.0.1)

This is the WASI adapter used by `wasm-tools component new --adapt` to lift a
WASI Preview 1 reactor (produced by `cargo build --target wasm32-wasip1`) into a
WASM Component Model binary.

**Pinned `wasm-tools` version:** 1.248.0

The WASI adapter and wasm-tools version are pinned together. The adapter must be
compatible with the wasm-tools version used to wrap it. Changing either requires
re-verifying the full build pipeline end-to-end.

### How to update

1. Download the new `wasi_snapshot_preview1.wasm` from the target Wasmtime release page.
2. Update the wasm-tools version pin in:
   - `.github/workflows/ci.yml` (`WASM_TOOLS_VERSION` in the `wasm32-compile-check` job)
   - `Justfile` (`wasm-tools 1.248.0+` prerequisites comment in `build-plugin-crowdstrike-oauth2`)
3. Rebuild the `.prx` artifact: `just build-plugin-crowdstrike-oauth2`
4. Validate: `wasm-tools validate --features=component-model crates/prism-spec-engine/plugins/crowdstrike-oauth2/crowdstrike-oauth2.prx`
5. Commit `wasi_snapshot_preview1.wasm` and the rebuilt `.prx` together in one commit.

## `crates/prism-spec-engine/plugins/crowdstrike-oauth2/crowdstrike-oauth2.prx`

**What it is:** The compiled WASM Component Model binary for the crowdstrike-oauth2
authentication plugin. This is the artifact loaded by `PluginRuntime::load_all_plugins`
at boot step 7.5.

**How to rebuild:**

```bash
just build-plugin-crowdstrike-oauth2
```

Prerequisites:
- `rustup target add wasm32-wasip1`
- `cargo install wasm-tools --version 1.248.0`

**When to rebuild:** Whenever any of the following change in the
`crates/crowdstrike-oauth2-plugin/` crate:
- `src/lib.rs` (plugin logic)
- `wit/sensor-auth.wit` (WIT interface definition)
- `plugin.toml` (plugin manifest)

The `.prx` is checked into the repository so that the integration test
`test_PLUGIN_MIGRATION_001_E_med_001_built_prx_loads_via_plugin_runtime` can run
without requiring a WASM toolchain on every developer machine.

**Staleness warning:** The committed `.prx` reflects the plugin source at the time
it was last built. If the plugin source changes and the `.prx` is not rebuilt,
the integration test will load a stale binary that does not reflect the current
source. CI rebuilds the `.prx` in the `wasm32-compile-check` job to catch this.
Rebuild locally with `just build-plugin-crowdstrike-oauth2` before committing
plugin source changes.

## `src/` (WAT source fixtures)

The `src/` subdirectory contains WAT (WebAssembly Text Format) source files used
by unit tests in `prism-spec-engine`. These are compiled at test time via
`wat::parse_file` — they are source files, not pre-built binaries.
