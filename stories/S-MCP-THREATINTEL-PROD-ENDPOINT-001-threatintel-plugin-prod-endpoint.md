---
document_type: story
story_id: S-MCP-THREATINTEL-PROD-ENDPOINT-001
title: "Replace dev-only allowed_urls in threatintel-lookup manifest with production ThreatIntel API endpoint"
wave: unscheduled
# Wave assignment: unscheduled — cannot schedule until the production ThreatIntel service
# endpoint is identified by the product owner / business team. No code dependency blocks
# implementation once the endpoint is known; scheduling is gated solely on that business decision.
epic_id: maintenance
priority: P2
# P2: must complete before ThreatIntel enrichment is used against live production sensor data.
# Current manifest lists dev-only endpoints ("localhost", "127.0.0.1"), so the WASM plugin
# would be unable to call the real production ThreatIntel API — all enrichment calls would be
# blocked by BC-2.17.002's sandbox enforcement (plugin_http_request_blocked).
status: draft
# BC status: BC-2.17.007 v1.5 ACTIVE, BC-2.17.002 ACTIVE. S-7.01 gate satisfied.
# behavioral_contracts array is non-empty; story is ready for PO review once
# the production endpoint URL is identified and AC-001 precondition can be filled in.
version: "0.2"
spec_version: "v0.2"
level: ops
producer: product-owner
timestamp: "2026-07-13"
modified: "2026-07-14"
input-hash: ""
inputs:
  - crates/plugins/prism-threatintel-infusion/threatintel-lookup.manifest.toml
  - crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.manifest.toml
  - crates/prism-spec-engine/plugins/threatintel-lookup/README.md
  - .factory/specs/behavioral-contracts/BC-2.17.007-plugin-manifest-schema-validation.md
  - .factory/specs/behavioral-contracts/BC-2.17.002-plugin-sandbox-no-filesystem-network.md
  - .factory/specs/architecture/decisions/ADR-040-wasm-infusion-plugin-host-decode-path.md
origin_finding: "DEFECT-MCP-ROWSHAPE-NULLS-001 PR #222 PR-LEVEL adversarial cascade: F-MCPRS-PRL1-OBS-001 [config-placeholder] — allowed_urls lists dev-only endpoints with no story anchor for production endpoint migration; violates Canonical Principle Rule 3 deferral requirements"
origin_cascade: "DEFECT-MCP-ROWSHAPE-NULLS-001; PR #222 PR-LEVEL cascade fix-burst 14 (2026-07-13)"
human_deferral: "2026-07-13 — Finding F-MCPRS-PRL1-OBS-001 raised during PR #222 adversarial review; story anchor created per orchestrator direction (fix-burst 14). Satisfies Canonical Principle Rule 3: explicit human direction (orchestrator per human session 2026-07-13) + concrete external dependency (production ThreatIntel service endpoint not yet identified) + story anchor (this file)."
cycle: "v1.0.0-greenfield"
phase: 3
tdd_mode: strict
track: "Platform Engineering"
subsystems: [SS-17]
# Subsystem anchor justification:
#   SS-17 (WASM Plugin Runtime) owns this story's scope per ARCH-INDEX Subsystem Registry because
#   the work modifies the plugin manifest (allowed_urls field) and regenerates the plugin artifact
#   (.prx + .prx.src-tree-hash). Both are SS-17 artifacts. The manifest's allowed_urls field is
#   governed exclusively by BC-2.17.007 and BC-2.17.002, which are both SS-17 BCs (CAP-032).
#   No other subsystem boundary is crossed.
crates_touched:
  - prism-spec-engine
  - prism-threatintel-infusion
target_module: "crates/plugins/prism-threatintel-infusion"
behavioral_contracts: [BC-2.17.007, BC-2.17.002]
# BC-2.17.007 v1.5 ACTIVE — Plugin Manifest Schema Validation Before WIT Validation.
#   Postcondition 4: allowed_urls must be an explicit list (not None/absent).
#   Invariant: allowed_urls None is never a valid loaded state.
#   This story ensures the production manifest carries a real, non-dev-only allowed_urls list.
# BC-2.17.002 ACTIVE — Plugin Sandbox — No Direct Filesystem or Network Access.
#   Governs the host-side HTTP allowlist enforcement. Any URL not in allowed_urls is blocked
#   with E-PLUGIN-005 (plugin_http_request_blocked event). Updating allowed_urls from
#   localhost/127.0.0.1 to the production endpoint is required for the ThreatIntel plugin
#   to operate at all against the live production service.
verification_properties: []
depends_on: []
# No code-level dependencies. The manifest update and plugin rebuild are independent of
# any other story. Business prerequisite (production endpoint identification) is not a
# story dependency — it is tracked as a precondition in AC-001.
blocks: []
points: 2
# 2 points breakdown:
#   Manifest update in canonical source crate (1 file, 1 field): 0.5 pt
#   `just build-plugin-threatintel-infusion` rebuild + 3-file commit: 0.5 pt
#   Net-new Red Gate test (RGT-001, grep-gate against manifest): 0.5 pt
#   Integration smoke-test pass confirmation: 0.5 pt
estimated_days: 0.25
risk: P2
acceptance_criteria_count: 3
red_gate_tests: 1
estimated_passes: "1"
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
---

# S-MCP-THREATINTEL-PROD-ENDPOINT-001: Replace dev-only allowed_urls in threatintel-lookup manifest with production endpoint

## §Origin — DEFECT-MCP-ROWSHAPE-NULLS-001 PR #222 PR-LEVEL cascade; finding F-MCPRS-PRL1-OBS-001

**Cascade:** DEFECT-MCP-ROWSHAPE-NULLS-001 (fix/DEFECT-MCP-ROWSHAPE-NULLS-001, PR #222)
**Finding:** F-MCPRS-PRL1-OBS-001 [config-placeholder] — 2026-07-13
**Human deferral:** 2026-07-13 — story anchor created (this file) per orchestrator direction in fix-burst 14

Canonical Principle Rule 3 is satisfied:
- Explicit human direction: orchestrator dispatch per human session 2026-07-13
- Concrete future dependency: production ThreatIntel service endpoint not yet identified (external business decision)
- Story anchor: this file (S-MCP-THREATINTEL-PROD-ENDPOINT-001)

### Background

`crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.manifest.toml`
(deploy copy; canonical source at `crates/plugins/prism-threatintel-infusion/threatintel-lookup.manifest.toml`)
contains:

```toml
allowed_urls = ["localhost", "127.0.0.1"]
```

These are dev-only DTU clone endpoints. The inline comment in the manifest acknowledges this and
notes that `threat-intel.example.com` is a placeholder to be replaced "when the production service
is identified." The production ThreatIntel API service endpoint has not been identified as of
2026-07-13 — this is a business-side decision outside the scope of the current cascade.

Until this story is implemented:
- The WASM plugin's sandbox (BC-2.17.002) blocks all HTTP calls to non-localhost URLs
- ThreatIntel enrichment works only against the DTU clone (dev/test environment)
- Production deployment of ThreatIntel IOC enrichment is impossible

Per ADR-040 §D9, `prism-threatintel-infusion` WASM plugin is retained (ThreatIntel stays on
the WASM Plugin path; IOC classification requires code). The production endpoint will be the
real ThreatIntel API service, not an HttpLookup target.

AD-017 applies: credentials are NEVER stored in the manifest. The `allowed_urls` field
is a URL hostname/IP allowlist only. API tokens are resolved at call time via `config_map`.

### Precondition: Production ThreatIntel Endpoint Required (AC-001)

This story cannot be implemented until the following question is resolved by the
product owner / business team:

> **What is the production ThreatIntel API endpoint hostname (e.g., `api.example.com`)?**

Once known, replace `threat-intel.example.com` in AC-001 with the real hostname.

---

## Narrative

As a Prism operator deploying ThreatIntel IOC enrichment against live production sensor data,
I want the `threatintel-lookup.manifest.toml` to list the real production API endpoint in
`allowed_urls` so that the WASM plugin sandbox permits outbound calls to the production service
and blocks calls to all other hosts.

## Behavioral Contracts

| BC | Title | Version | Relevance |
|----|-------|---------|-----------|
| BC-2.17.007 | Plugin Manifest Schema Validation Before WIT Validation | v1.5 | Postcondition 4: `allowed_urls` must be an explicit list; invariant: `allowed_urls` None is never a valid loaded state. The production manifest must carry a non-empty, non-dev allowlist. |
| BC-2.17.002 | Plugin Sandbox — No Direct Filesystem or Network Access | active | Governs `allowed_urls` enforcement: the WASM host blocks HTTP calls to any URL not in the allowlist (`plugin_http_request_blocked` event, E-PLUGIN-005). The story ensures only the production endpoint is in the list. |

## Acceptance Criteria

### AC-001 — Production endpoint replaces dev-only entries
(traces to BC-2.17.007 v1.5 postcondition 4 — allowed_urls explicit list with real production values;
 traces to BC-2.17.002 invariant — sandbox enforcement allows production endpoint, blocks localhost)

After this story is implemented, the canonical source manifest at
`crates/plugins/prism-threatintel-infusion/threatintel-lookup.manifest.toml` contains:

```toml
allowed_urls = ["<PRODUCTION-HOSTNAME>"]
```

Where `<PRODUCTION-HOSTNAME>` is the real production ThreatIntel API hostname (e.g.,
`api.example.com`). The entries `"localhost"` and `"127.0.0.1"` are ABSENT from the
production manifest.

**Implementer directive:** Before beginning, read `allowed_urls` in the canonical source
manifest to confirm it still reads `["localhost", "127.0.0.1"]`. If a production endpoint
is already present, this story may already be implemented — verify AC-002 and AC-003 hold.

### AC-002 — Plugin rebuild succeeds; three artifacts updated and committed atomically
(traces to BC-2.17.007 v1.5 postcondition 4 — manifest valid after update; invariant — rebuild
 propagates manifest to deploy copy)

After running `just build-plugin-threatintel-infusion`, all three output files are updated:

| File | Action |
|------|--------|
| `crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.prx` | Rebuilt from source |
| `crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.manifest.toml` | Deploy copy of updated manifest |
| `crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.prx.src-tree-hash` | Updated source tree hash |

All three files are committed atomically in a single commit (per README §Staleness Gate rebuild
commit recipe). The CI staleness gate (`wasm32-threatintel-staleness-check`) must pass.

`wasm-tools validate --features=component-model threatintel-lookup.prx` exits 0.

### AC-003 — Grep-gate: no localhost or 127.0.0.1 in canonical source manifest
(traces to BC-2.17.002 invariant — sandbox must not permit dev-only loopback addresses in
 production manifest; traces to BC-2.17.007 postcondition 4 — explicit list with real values only)

After this story is implemented, the following assertion passes:

```bash
python3 -c "
import tomllib
with open('crates/plugins/prism-threatintel-infusion/threatintel-lookup.manifest.toml', 'rb') as f:
    manifest = tomllib.load(f)
urls = manifest.get('allowed_urls', [])
dev_urls = [u for u in urls if u in ('localhost', '127.0.0.1')]
assert len(dev_urls) == 0, f'Dev-only URLs found in production manifest: {dev_urls}'
assert len(urls) >= 1, 'allowed_urls must be non-empty for a production manifest'
print(f'PASS: allowed_urls = {urls}')
"
```

The net-new RGT-001 test (`test_no_dev_urls_in_threatintel_production_manifest`) encodes this
assertion as a Rust test so it runs in `just check`. It FAILS before this story is implemented
(dev URLs present) and PASSES after.

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| Canonical source manifest | `crates/plugins/prism-threatintel-infusion/threatintel-lookup.manifest.toml` | Pure (config file) |
| Deploy copy manifest | `crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.manifest.toml` | Pure (generated by `just build-plugin-threatintel-infusion`) |
| Plugin artifact | `crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.prx` | Effectful (WASM Component; calls ThreatIntel API via host_functions.rs) |
| Source tree hash sidecar | `crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.prx.src-tree-hash` | Pure (hash of plugin source tree) |
| RGT-001 | `crates/prism-spec-engine/tests/` or inline in manifest validation test module | Pure (reads manifest file, asserts no dev-only URLs) |

Architecture section reference:
- `.factory/specs/architecture/decisions/ADR-040-wasm-infusion-plugin-host-decode-path.md` §D9 (ThreatIntel Plugin retained)
- BC-2.17.007 §Architecture Anchors: ADR-023 §C4 (allowed_urls absence is rejection not default)
- `crates/prism-spec-engine/plugins/threatintel-lookup/README.md` (canonical build procedure)

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Production endpoint is a URL path (e.g., `api.example.com/v3`) rather than just hostname | The WASM host allowlist uses exact host-string matching. Only the hostname portion (no path) goes in `allowed_urls`. If the API uses a non-standard port, include it: `api.example.com:443`. Verify with a manual call against the DTU before committing. |
| EC-002 | Multiple production hostnames needed (e.g., primary + fallback) | Both hostnames may be listed: `allowed_urls = ["primary.example.com", "fallback.example.com"]`. No localhost entries permitted in production. |
| EC-003 | Rebuilt `.prx` differs in bytes from the committed `.prx` on macOS vs Linux | Expected — LLVM WASM codegen is not byte-reproducible across CPU architectures. The CI staleness gate validates via source-hash sidecar, not byte-equality. Commit the macOS build; CI re-validates. |
| EC-004 | DTU environment still needs localhost/127.0.0.1 access | DTU environments use a separate `.prism/` config directory. A dev-overlay manifest (allowed_urls with localhost) is the correct approach — do NOT reintroduce localhost to the canonical production manifest. Future story: consider environment-specific manifest overlays. |

## Red Gate Tests

| # | Test Name | Location | Status | Target AC | Fails Without |
|---|-----------|----------|--------|-----------|---------------|
| RGT-001 | `test_no_dev_urls_in_threatintel_production_manifest` | `crates/prism-spec-engine/tests/manifest_sanity.rs` (or adjacent manifest-validation test file) | **NET-NEW** | AC-001, AC-003 | Any of `localhost` or `127.0.0.1` present in canonical source manifest |

**RGT-001** reads `crates/plugins/prism-threatintel-infusion/threatintel-lookup.manifest.toml`
via `include_str!` or `std::fs::read` at test time and asserts the production manifest carries
no dev-only loopback entries. It FAILS before the manifest is updated (current state: dev URLs
present) and PASSES after:

```rust
#[test]
fn test_no_dev_urls_in_threatintel_production_manifest() {
    // S-MCP-THREATINTEL-PROD-ENDPOINT-001 AC-001/AC-003 production-readiness gate.
    // Fails if the production manifest still contains dev-only loopback entries.
    // Source: F-MCPRS-PRL1-OBS-001 (DEFECT-MCP-ROWSHAPE-NULLS-001 PR #222 cascade).
    let manifest_src = include_str!(
        "../../plugins/prism-threatintel-infusion/threatintel-lookup.manifest.toml"
    );
    let dev_patterns = ["\"localhost\"", "\"127.0.0.1\""];
    for pat in &dev_patterns {
        assert!(
            !manifest_src.contains(pat),
            "Dev-only URL {pat} found in production manifest \
             (crates/plugins/prism-threatintel-infusion/threatintel-lookup.manifest.toml). \
             Update allowed_urls to the real production ThreatIntel API endpoint per \
             S-MCP-THREATINTEL-PROD-ENDPOINT-001 AC-001."
        );
    }
}
```

**Red Gate execution:**
```bash
# RGT-001 (net-new; fails before manifest update, passes after):
cargo nextest run -p prism-spec-engine -E 'test(no_dev_urls_in_threatintel_production_manifest)'
```

---

## §Token Budget Estimate

| Source | Estimated Tokens |
|--------|-----------------|
| Story spec (this file) | ~4,000 |
| BC-2.17.007 v1.5 (postconditions, invariants) | ~4,000 |
| BC-2.17.002 (sandbox enforcement, allowed_urls) | ~3,000 |
| ADR-040 §D9 + §Scope Boundary (ThreatIntel WASM path) | ~2,500 |
| `crates/plugins/prism-threatintel-infusion/threatintel-lookup.manifest.toml` (27 lines) | ~500 |
| `crates/prism-spec-engine/plugins/threatintel-lookup/README.md` (128 lines) | ~2,000 |
| Justfile `build-plugin-threatintel-infusion` recipe | ~800 |
| **Total** | **~16,800 tokens** |

Context window usage: ~16,800 / 200,000 = **~8%** (well within the 20-30% limit)

---

## §Tasks

**Step 1 — Prerequisite: identify production endpoint (human/PO task)**
- [ ] Product owner or business team identifies the production ThreatIntel API hostname
- [ ] Implementer confirms the hostname with PO before changing any file
- [ ] Record the hostname in a comment on this story or a D-NNN decision log entry

**Step 2 — Add RGT-001 (net-new; must FAIL before manifest change)**
- [ ] Add `test_no_dev_urls_in_threatintel_production_manifest` to appropriate test location in `crates/prism-spec-engine`
- [ ] Run it: `cargo nextest run -p prism-spec-engine -E 'test(no_dev_urls_in_threatintel_production_manifest)'`
- [ ] Confirm it FAILS (dev URLs present — Red Gate established)

**Step 3 — Update canonical source manifest (AC-001)**
- [ ] Edit `crates/plugins/prism-threatintel-infusion/threatintel-lookup.manifest.toml`
- [ ] Replace `allowed_urls = ["localhost", "127.0.0.1"]` with `allowed_urls = ["<PRODUCTION-HOSTNAME>"]`
- [ ] Remove the dev-only inline comment; add a comment citing this story ID and the production service name
- [ ] Preserve all other fields verbatim (`name`, `version`, `format_version`, `plugin_type`)

**Step 4 — Rebuild plugin artifacts (AC-002)**
- [ ] Run `just build-plugin-threatintel-infusion`
- [ ] Confirm exit 0; check that `wasm-tools validate` passes (printed by recipe)
- [ ] Verify three files updated:
  - `crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.prx`
  - `crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.manifest.toml`
  - `crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.prx.src-tree-hash`

**Step 5 — Verify AC-003 grep-gate**
- [ ] Run `cargo nextest run -p prism-spec-engine -E 'test(no_dev_urls_in_threatintel_production_manifest)'`
- [ ] Confirm it PASSES (no dev URLs; count = 0)

**Step 6 — Pre-push gate**
- [ ] `just check` (full workspace gate: fmt + clippy + nextest + doctests + crate-layout)
- [ ] CI staleness gate must pass (`wasm32-threatintel-staleness-check`)

**Step 7 — Commit atomically (three artifact files + source manifest + test)**
- [ ] Stage all four changed/new files per README §Staleness Gate commit recipe:
  ```bash
  git add crates/plugins/prism-threatintel-infusion/threatintel-lookup.manifest.toml
  git add crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.prx
  git add crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.manifest.toml
  git add crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.prx.src-tree-hash
  git add <RGT-001 test file>
  ```

---

## §Previous Story Intelligence

**Predecessor stories:** S-DEMO-ENRICHMENT-PIVOT-002 (ADR-040 v1.0 + v2.0; ThreatIntel WASM plugin
build pipeline, HIGH-1 fix, manifest name identity). The build recipe and three-file commit
convention were established by PIVOT-002. The staleness gate (source-hash sidecar) was added by
DEFECT-MCP-ROWSHAPE-NULLS-001 fix-burst 13 (F-MCPNULL-P2-OBS-002).

**Key lessons from predecessor stories:**
1. `allowed_urls` uses exact host-string matching — `"localhost"` and `"127.0.0.1"` are distinct
   entries that must both be listed for DTU (FIX-6, S-DEMO-ENRICHMENT-PIVOT-002). The production
   list should contain only the real production hostname.
2. The `name = "threat_intel"` field is load-bearing and must NOT change — it must match
   `infusion_id` in `specs/infusions/threatintel.infusion.toml`. Do not modify `name`.
3. The deploy copy (`crates/prism-spec-engine/plugins/threatintel-lookup/`) is generated output;
   always edit the canonical source (`crates/plugins/prism-threatintel-infusion/`) and rebuild.
4. The `.prx` binary may differ in bytes across CPU architectures — this is expected. The CI
   staleness gate validates via source-hash, not byte-equality.

---

## §Architecture Compliance Rules

Extracted from ADR-040, BC-2.17.007, and CLAUDE.md:

1. **`name = "threat_intel"` is load-bearing.** Do NOT change it. It is the plugin_id registered
   by `PluginRuntime::load_all_plugins` and must match `infusion_id = "threat_intel"` in
   `specs/infusions/threatintel.infusion.toml`. Changing `name` breaks the plugin resolution.

2. **Edit the canonical source manifest, not the deploy copy.** The deploy copy at
   `crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.manifest.toml` is
   generated by `just build-plugin-threatintel-infusion` step 4 (cp). Editing it directly
   will be overwritten on the next rebuild and will trigger the CI staleness gate.

3. **AD-017: credentials never in the manifest.** The `allowed_urls` field is a hostname allowlist
   only. API keys, tokens, or secrets must NEVER appear here. Credentials are resolved at call
   time via `config_map` per AD-017.

4. **Three-file atomic commit.** Per README §Staleness Gate, the `.prx`, `.manifest.toml`
   (deploy copy), and `.prx.src-tree-hash` must be committed together. Committing only the
   manifest without rebuilding will cause the CI staleness gate to fail.

5. **No new Rust code in this story.** The only Rust addition is RGT-001 (a test). No production
   code paths change. The `allowed_urls` field is read by the WASM host at load time via the
   manifest TOML parser — no code changes are required.

6. **Forbidden pattern: localhost/127.0.0.1 in production manifest.** After this story merges,
   a future PR that re-introduces `"localhost"` or `"127.0.0.1"` to the canonical source
   manifest is a regression — RGT-001 will catch it automatically.

---

## §Library & Framework Requirements

| Library / Tool | Version / Source | Source of truth |
|----------------|-----------------|-----------------|
| `wasm-tools` | version pinned in SHA-256 per HUMAN-APPROVED SEC-001 (DEFECT-MCP-ROWSHAPE-NULLS-001 fix-burst 13) | `.github/workflows/` or `Justfile` `wasm-tools-sha256` pin |
| `python3` (tomllib) | 3.11+ (stdlib; `tomllib` available since 3.11) | `rust-toolchain.toml` dev environment |
| `just` | workspace task runner | `Justfile` |

No new external Rust crate dependencies are introduced by this story.

---

## §File Structure Requirements

| File | Action | Change Description |
|------|--------|-------------------|
| `crates/plugins/prism-threatintel-infusion/threatintel-lookup.manifest.toml` | Modify | Update `allowed_urls` — replace `["localhost", "127.0.0.1"]` with `["<PRODUCTION-HOSTNAME>"]` |
| `crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.manifest.toml` | Regenerated | Deploy copy; updated by `just build-plugin-threatintel-infusion` step 4 (cp); DO NOT edit directly |
| `crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.prx` | Regenerated | WASM Component binary; rebuilt by `just build-plugin-threatintel-infusion` step 1-3 |
| `crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.prx.src-tree-hash` | Regenerated | Source tree hash sidecar; updated by `just build-plugin-threatintel-infusion` step 5 |
| `crates/prism-spec-engine/tests/manifest_sanity.rs` (or adjacent) | Create or Modify | Add RGT-001: `test_no_dev_urls_in_threatintel_production_manifest` |

No other files are touched. The canonical infusion spec (`specs/infusions/threatintel.infusion.toml`)
and the WASM plugin Rust source (`crates/plugins/prism-threatintel-infusion/src/`) are NOT in scope.

**Scope addition (F-MCPRS-PRL6, DEFECT-MCP-ROWSHAPE-NULLS-001 PR-LEVEL pass-6 out-of-scope observation):** When updating `allowed_urls`, the implementer must also correct the inline comment in the canonical source manifest (`crates/plugins/prism-threatintel-infusion/threatintel-lookup.manifest.toml`) that references `"threat-intel.example.com placeholder"` — this manifest comment drift was confirmed out-of-scope for the DEFECT-MCP-ROWSHAPE-NULLS-001 cascade (frozen HEAD; comment edit would force a plugin re-cut cycle) and is explicitly anchored to this story's scope.

## §Changelog

| Version | Date | Change | Source |
|---------|------|--------|--------|
| v0.2 | 2026-07-14 | Scope addition: manifest comment drift correction anchored to this story (F-MCPRS-PRL6, DEFECT-MCP-ROWSHAPE-NULLS-001 PR-LEVEL pass-6 out-of-scope observation). The inline comment in `threatintel-lookup.manifest.toml` referencing `"threat-intel.example.com placeholder"` must be corrected atomically with the `allowed_urls` update. `modified:` updated 2026-07-13→2026-07-14. | DEFECT-MCP-ROWSHAPE-NULLS-001 fix-burst 18 (spec-only) |
| v0.1 | 2026-07-13 | Initial draft — 3 ACs, 1 RGT, manifest update + plugin rebuild scope. | DEFECT-MCP-ROWSHAPE-NULLS-001 fix-burst 14 (F-MCPRS-PRL1-OBS-001 story anchor) |
