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

The Justfile recipe executes these steps:

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

5. **Source tree hash sidecar** (CI staleness gate, F-MCPNULL-P2-OBS-002)
   ```bash
   python3 scripts/hash-plugin-source.py crates/plugins/prism-threatintel-infusion \
       > crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.prx.src-tree-hash
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

## Staleness Gate

The CI job `wasm32-threatintel-staleness-check` enforces two independent checks:

### Check 1 — Source-hash freshness (F-MCPNULL-P2-OBS-002)

Compares `threatintel-lookup.prx.src-tree-hash` against a freshly computed hash of all
tracked files in `crates/plugins/prism-threatintel-infusion/`. A mismatch means source
changed without rebuilding the `.prx`.

`Cargo.lock` is tracked in this directory (human ruling F-MCPRS-PRL10-OBS-001,
2026-07-14; plugins ship compiled binaries — same app-binary rationale as locking a
main workspace binary). As a result, transitive dependency drift is caught by this
check: any change to the resolved dependency graph changes the lockfile, changing the
source hash, failing this gate. The transitive-dep drift attack vector is **closed**.

### Check 2 — Sidecar ancestry (F-MCPRS-PRL1-MED-001)

Enforces the invariant: the sidecar's last-touching commit must be an
**ancestor-of-or-equal-to** the `.prx`'s last-touching commit. In plain terms, every
sidecar introduction or update must be followed by (or included in) a `.prx` rebuild.

**Passes** when: `.prx` was rebuilt after (or at the same time as) the sidecar was last
updated. This includes rebuild-only commits that touch only the `.prx` binary (because
source was unchanged, the sidecar content stays the same and its last-touching commit
remains older — which is the intended state).

**Fails** when: the sidecar's last commit is newer than the `.prx`'s last commit — i.e.,
the sidecar was updated or introduced more recently than the last `.prx` rebuild. This
indicates the binary may be stale relative to what the sidecar records.

**Fix:** Run `just build-plugin-threatintel-infusion` and commit the updated `.prx`. The
sidecar content is unchanged when source is unchanged; only the `.prx` binary needs to
be staged.

```bash
just build-plugin-threatintel-infusion
git add crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.prx
git commit -m "chore: rebuild threatintel-lookup.prx to restore sidecar ancestry (F-MCPRS-PRL1-MED-001)"
```

If the rebuild produces no byte change (identical toolchain + unchanged source on the
same platform), the only repair is a history rewrite to reorder the sidecar introduction
commit prior to the last `.prx` rebuild. This requires human-directed force-push approval.

### Residual Risk

Two attack surfaces survive all three CI checks and are accepted as out-of-scope for this
gate:

**Vector 1 — .prx-only byte substitution (simpler, lower friction).**
An actor with repository write access submits a commit that modifies only the committed
`.prx` binary — leaving the source crate and `threatintel-lookup.prx.src-tree-hash` sidecar
untouched. Under this scenario:
- Check 1 (source-hash freshness) passes: the sidecar was not updated, so the hash still
  matches the unmodified source.
- Check 2 (sidecar ancestry) passes: the `.prx` commit is now newer than the sidecar
  commit, so `git merge-base --is-ancestor sidecar prx` succeeds.
- Check 3 (manifest parity, F-MCPNULL-P3-LOW-004) passes: it compares manifest text
  only, never hashes `.prx` bytes.

A byte-comparison 4th check is infeasible: macOS aarch64 and Linux x86_64 produce
different `.prx` bytes from the same source (LLVM WASM codegen is not byte-reproducible
across CPU architectures — this is the founding constraint of this gate per architect
adjudication). There is no canonical expected-bytes reference to compare against.

**Vector 2 — coordinated sidecar + .prx manipulation.**
An actor updates the sidecar to a hash matching a different source snapshot AND submits
a `.prx` built from that snapshot in the same or a subsequent commit. All three checks
pass because the internal self-consistency of the artifact set is preserved. Note: with
`Cargo.lock` now tracked, the source snapshot must include the lockfile — an actor
cannot silently substitute a snapshot with different transitive deps without the sidecar
hash reflecting the modified lockfile.

**Accepted mitigation boundary:** Both vectors require write access to the repository
and a commit that touches the `.prx` binary. The compensating control is mandatory code
review of all commits touching binary artifacts, enforced by branch protection (required
reviews before merge). Future mitigation: SLSA Level 2+ provenance attestation, which
produces a signed build provenance record tied to a specific source commit and build
environment, making substitution detectable out-of-band.

### Cross-Platform Note

The committed `.prx` binary may differ in bytes between macOS aarch64 and Linux x86_64
builds from the same source. This is expected: LLVM-backed WASM codegen is not
byte-reproducible across CPU architectures. Neither staleness check relies on
byte-equality; the source-hash check uses source-content hashing, and the ancestry
check uses git commit identity. The binary is validated structurally via
`wasm-tools validate --features=component-model` on the committed artifact.

---

## Provenance Anchors

| Anchor | Value |
|--------|-------|
| Source crate first committed | S-DEMO-ENRICHMENT-PIVOT-002 @ `6c367356` |
| Artifact tracking precedent | S-PLUGIN-CI-001 (PR #159) — established pattern of committing `.prx` deploy output alongside source |
| Corrective audit finding | DEFECT-MCP-ROWSHAPE-NULLS-001 [H20] `F-MCPNULL-P1-MED-002` — supply-chain provenance breadcrumb missing |
| Staleness gate design | DEFECT-MCP-ROWSHAPE-NULLS-001 fix-burst 13 — source-hash sidecar per architect adjudication (F-MCPNULL-P2-OBS-002) |
| Current plugin version | `1.0.2` — re-cut from 1.0.1 to track `Cargo.lock` for full-graph reproducibility (F-MCPRS-PRL10-OBS-001, human ruling 2026-07-14); source of truth: `fn version()` in `crates/plugins/prism-threatintel-infusion/src/lib.rs`. Update this row and that function together on each future re-cut. |
