# threatintel-lookup Plugin Artifact

This directory contains the deployed WASM Component plugin artifact for the ThreatIntel
infusion enrichment source. It is **build output**, not a source tree.

Corrective: DEFECT-MCP-ROWSHAPE-NULLS-001 [H20] audit-reproducibility finding (F-MCPNULL-P1-MED-002).

---

## Source of Truth

The canonical source crate is:

```
crates/plugins/prism-threatintel-infusion/
```

This crate is a workspace member (see root `Cargo.toml` `members`). All Rust source,
WIT interfaces, and the canonical manifest live there. Do not edit files in this
directory directly — rebuild from source using the recipe below.

---

## Rebuild Command

```bash
just build-plugin-threatintel-infusion
```

### Pipeline breakdown

The Justfile recipe (`Justfile` lines 294–322) executes these steps:

1. **Cargo build (wasm32-wasip1 target)**
   ```bash
   cargo build \
       --manifest-path crates/plugins/prism-threatintel-infusion/Cargo.toml \
       --target wasm32-wasip1 \
       --release
   ```
   Emits: `crates/plugins/prism-threatintel-infusion/target/wasm32-wasip1/release/prism_threatintel_infusion.wasm`

2. **WASM Component Model lift** (`wasm-tools component new --adapt`)
   ```bash
   wasm-tools component new \
       crates/plugins/prism-threatintel-infusion/target/wasm32-wasip1/release/prism_threatintel_infusion.wasm \
       --adapt wasi_snapshot_preview1=tests/fixtures/wasi_snapshot_preview1.wasm \
       -o crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.prx
   ```
   The `--adapt` flag splices the WASI preview1 adapter shim (committed at
   `tests/fixtures/wasi_snapshot_preview1.wasm`) so the core module becomes a
   self-contained WASM Component (`.prx` extension = Prism plugin artifact).

3. **Component Model validation**
   ```bash
   wasm-tools validate --features=component-model threatintel-lookup.prx
   wasm-tools print threatintel-lookup.prx | grep -q '(component'
   ```

4. **Manifest deploy** (copy from source crate to this directory)
   ```bash
   cp crates/plugins/prism-threatintel-infusion/threatintel-lookup.manifest.toml \
       crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.manifest.toml
   ```

---

## Manifest Ownership

`threatintel-lookup.manifest.toml` in this directory is **deploy output** of step 4 above.
The canonical manifest lives with the source crate:

```
crates/plugins/prism-threatintel-infusion/threatintel-lookup.manifest.toml
```

Edit the manifest there, then run `just build-plugin-threatintel-infusion` to propagate
the change here. The manifest `name = "threat_intel"` field is load-bearing: it is the
plugin_id registered by `PluginRuntime::load_all_plugins` and must match the
`infusion_id` in `specs/infusions/threatintel.infusion.toml` (HIGH-1 fix,
S-DEMO-ENRICHMENT-PIVOT-002).

---

## Verifying the Committed Binary Matches Source

Rebuild and compare SHA-256 digests:

```bash
# Save committed digest
sha256sum crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.prx

# Rebuild
just build-plugin-threatintel-infusion

# Re-check — should be identical
sha256sum crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.prx
```

**CI enforcement:** The `wasm32-threatintel-staleness-check` job in `.github/workflows/ci.yml`
rebuilds the `.prx` on every CI run and fails if the rebuilt SHA-256 differs from the
committed artifact (F-MCPNULL-P2-OBS-002). A PR that updates plugin source without
recommitting the rebuilt `.prx` will be blocked at CI.

**Reproducibility status:** The build is **byte-for-byte reproducible** on the same
toolchain and host. Verified 2026-07-13: rebuild from the committed
`prism_threatintel_infusion.wasm` core module against the committed
`tests/fixtures/wasi_snapshot_preview1.wasm` adapter produced a `.prx` with SHA-256
`a18a45976a76bcf21311a04d5b364c628e07a4a020c4134f568999a015972ece`, identical to the
committed artifact (140 793 bytes). The determinism derives from:

- `--release` profile (no debug symbols with timestamps)
- `wasm-tools component new` with a pinned adapter wasm at a fixed path
- No embedded build timestamps in the WASM custom sections

Cross-toolchain reproducibility (different Rust channel or `wasm-tools` version) is
**not guaranteed**. If a rebuild on a different environment produces a different hash,
investigate the toolchain version (`rustup show`, `wasm-tools --version`) before
concluding the committed binary is tampered.

---

## Provenance Anchors

| Anchor | Value |
|--------|-------|
| Source crate first committed | S-DEMO-ENRICHMENT-PIVOT-002 @ `6c367356` |
| Artifact tracking precedent | S-PLUGIN-CI-001 (PR #159) — established pattern of committing `.prx` deploy output alongside source |
| Corrective audit finding | DEFECT-MCP-ROWSHAPE-NULLS-001 [H20] `F-MCPNULL-P1-MED-002` — supply-chain provenance breadcrumb missing |
| Artifact SHA-256 | `a18a45976a76bcf21311a04d5b364c628e07a4a020c4134f568999a015972ece` |
| Artifact size | 140 793 bytes |
