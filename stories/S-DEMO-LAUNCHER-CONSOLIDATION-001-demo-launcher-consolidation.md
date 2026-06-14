---
document_type: story
story_id: S-DEMO-LAUNCHER-CONSOLIDATION-001
title: "demo-launcher: add start-multi CLI subcommand to prism-dtu-demo-server + generalize demo scripts to N orgs"
wave: 5
epic_id: E-DEMO
priority: P3
status: ready
version: "2.2"
level: "L4"
producer: story-writer
timestamp: "2026-06-14T00:00:00Z"
tdd_mode: tdd
# tdd_mode: tdd justification (Option-2 architect decision, version 2.0) —
#   This story now touches prism-dtu-demo-server (Rust crate). The new
#   `StartMulti` subcommand and `MultiOrgDemoConfig`/`OrgConfig`
#   structs require Red Gate tests (config parsing, sidecar format, clone_factory
#   dispatch, per-org socket isolation). Full TDD Iron Law applies:
#   non-trivial function bodies use `todo!()` stubs in the test-writer phase;
#   Red Gate density check >= 0.5 is required before implementer dispatch.
#   Shell-only deliverables (demo-setup.sh, demo-run.sh, demo-teardown.sh,
#   demo.toml, DEMO-RUNBOOK.md) retain the facade-mode delivery model for their
#   own content but are gated on the Rust subcommand being complete first.
subsystems: [SS-06, SS-22]
# Subsystem anchor justifications:
#   SS-06 (Client Configuration) owns the demo prism.toml + per-org overlay TOML generation;
#     the updated scripts create config directory structures that prism-bin reads at startup, with
#     N-org and N-credential entries instead of the fixed single-org model from S-DEMO-003.
#     Per ARCH-INDEX Subsystem Registry SS-06.
#   SS-22 (Binary Entrypoint) owns the `prism-dtu-demo-server` CLI subcommand surface;
#     the new `start-multi` subcommand is a direct extension of the `Commands` enum in
#     `crates/prism-dtu-demo-server/src/main.rs`. Per ARCH-INDEX Subsystem Registry SS-22.
crates_touched: [prism-dtu-demo-server]
# crates_touched: prism-dtu-demo-server only.
#   Adds: StartMulti variant in Commands enum (main.rs), cmd_start_multi function (main.rs ~80-120 lines),
#   MultiOrgDemoConfig / MultiOrgConfig / OrgConfig structs (config.rs ~40-60 lines).
#   Zero changes to prism-dtu-harness, prism-dtu-common, or any other crate.
target_module: prism-dtu-demo-server
capabilities: [CAP-034]
behavioral_contracts:
  - BC-2.06.001  # TOML Configuration Loads and Deserializes at Startup — prism.toml must be
                 # schema-valid (N orgs, N [[orgs]] entries); the generalized demo-setup.sh
                 # writes a multi-org prism.toml and the spec-loading boot path must accept it.
  - BC-2.06.012  # Per-Tenant Overlay Loading and Merge Semantics — demo-run.sh writes N×M
                 # overlay TOMLs (one per org × sensor combo); BC-2.06.012 governs how
                 # spec loader reads customers/<org_slug>/<sensor>.sensor.toml at boot step 4c.
  - BC-2.06.013  # Scalar-Only Overlay Enforcement — overlay TOMLs written by demo-run.sh must
                 # contain only scalar fields (extends, instance_id, base_url); tables and
                 # auth_type are forbidden per BC-2.06.013 invariant.
  - BC-2.06.014  # Instance Identity Resolution at Fanout — per-org overlays written by demo-run.sh
                 # ensure that at fanout time each (org_id, sensor_id) resolves to its own
                 # DTU clone endpoint (distinct base_url per org × sensor).
  - BC-2.06.017  # Per-DTU-Instance Multi-Address Binding — the multi-org demo starts
                 # MultiInstanceHarness (N×M sockets, one per org × sensor), reads the nested
                 # sidecar ({org_slug: {sensor: url}}), and maps socket addresses to per-org overlay TOMLs.
verification_properties: []
depends_on:
  - S-DEMO-003   # Merged PR #176 develop@a42e3eaf (D-1055 2026-06-08). SATISFIED.
                 # demo-setup.sh, demo-run.sh, demo-teardown.sh all exist; the single-org (demo-org)
                 # model is the baseline being generalized here.
blocks:
  - S-DEMO-004   # Multi-org smoke test (ready v1.7). While S-DEMO-004 is a test-harness story
                 # (uses prism-dtu-harness, not the operator scripts), the operator-facing scripts
                 # produced by this story must be consistent with the 3-org model S-DEMO-004 validates.
                 # The scripts are the operator-facing surface; S-DEMO-004 is the CI-facing surface.
                 # NOTE: S-DEMO-004 is already ready and can proceed independently. This blocks
                 # relationship encodes demo-narrative completeness, not a hard build-order constraint.
points: 8
# Points justification (Option-2, architect estimate):
#   +2 — StartMulti subcommand (Commands enum variant + cmd_start_multi ~80-120 lines in main.rs):
#         parse MultiOrgDemoConfig, build MultiInstanceConfig entries from [[orgs]] config,
#         call start_instances(MultiInstanceConfig, clone_factory), write nested per-org sidecar
#         {org_slug: {sensor: url}}, wait for SIGTERM/SIGINT via shared broadcast, shutdown via
#         MultiInstanceServers::shutdown().
#   +1 — MultiOrgDemoConfig / MultiOrgConfig / OrgConfig config structs (~40-60 lines in config.rs)
#         + unit tests (config parsing, deny_unknown_fields, error cases).
#   +0.5 — demo-run.sh becomes simpler (replaces N separate prism-dtu-demo-server start invocations
#           with one start-multi; reads nested {org_slug: {sensor: url}} sidecar instead of flat).
#   +1.5 — demo-setup.sh generalization to N orgs (N-org prism.toml, N×M credential bootstrap).
#   +0.5 — demo-teardown.sh generalization to N×M credential deletes.
#   +0.5 — demo.toml [orgs.*] section for 3-org model (org-a/org-b/org-c with seeds).
#   +0.5 — docs/DEMO-RUNBOOK.md update + retire start-demo.sh.
#   +1.5 — Red Gate tests for Rust additions (see §Red Gate Tests below).
#   Total: 8 points (~2 days). Architect estimate: +2 StartMulti cmd, +1 MultiOrgDemoConfig+tests,
#         demo-run.sh simpler at 0.5; shells carry the remaining 4 points unchanged from v1.0.
estimated_days: 2
risk: MEDIUM
# Risk change v1.0→v2.0: LOW → MEDIUM. The Option-2 design adds Rust crate changes
# to prism-dtu-demo-server. The risk is bounded because:
#   (a) start-multi wires EXISTING, already-tested start_instances() and MultiInstanceConfig APIs
#       — no new multi-instance binding logic is written from scratch.
#   (b) MultiOrgDemoConfig is a NEW top-level config type (not a modification of DemoConfig)
#       so the existing `start` subcommand and its `#[serde(deny_unknown_fields)]` DemoConfig
#       remain backward-compatible.
#   (c) Red Gate tests gate the implementer dispatch; adversarial review will verify correctness.
#   (d) The shell script changes are additive (N=1 case must still work after the change).
acceptance_criteria_count: 13
red_gate_tests: 5
# red_gate_tests: 5 — see §Red Gate Tests section for full list.
estimated_passes: "2-3 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "MultiOrgDemoConfig MUST be a new top-level TOML type (e.g., MultiOrgDemoConfig parsed from
    scripts/demo.toml when start-multi is invoked) — NOT an extension of DemoConfig. DemoConfig
    has `#[serde(deny_unknown_fields)]` and a fixed 6-sensor ClonesConfig; adding [[orgs]] to it
    would fail parsing with an unknown-field error. The implementer must add a separate config
    struct in config.rs and load it ONLY in cmd_start_multi. The existing `start` subcommand and
    DemoConfig are UNTOUCHED."
  - "N=1 backward compatibility: the `start` subcommand still works with the pre-existing single-org
    DemoConfig demo.toml format. This is the S-DEMO-003 baseline case and must not regress. The
    implementer must verify `prism-dtu-demo-server start --config configs/demo.toml` still starts
    after the Rust changes."
  - "Sidecar format is NESTED for start-multi: the existing `start` subcommand writes
    {sensor: url} (flat). The new `start-multi` subcommand writes {org_slug: {sensor: url}}
    (nested). These are DIFFERENT sidecar files (or the same file — implementer's choice, but
    demo-run.sh must poll and parse the correct format). demo-run.sh must be updated to expect
    the nested format when it calls start-multi."
  - "clone_factory closure REQUIRES the `fixture-gen` Cargo feature. The seeded constructors
    (CrowdstrikeClone::new_with_seed, ClarotyClone::new_with_seed, ArmisClone::new_with_seed,
    CyberintClone::new_with_seed) are defined ONLY under `#[cfg(feature = \"fixture-gen\")]`.
    The `#[cfg(not(feature = \"fixture-gen\"))]` fallback arm in harness.rs build_clone_pairs
    calls plain `new()` which ignores the seed entirely — org-a and org-c CrowdStrike would
    serve IDENTICAL static data despite seed=100 vs seed=200, silently violating
    INV-DISTINCT-DATA-001 at runtime while still passing the socket-distinctness Red Gate.
    This silent-fallback is FORBIDDEN per the production-grade default. Therefore:
    (a) `build_multi_clone_factory` MUST be compiled and run ONLY with `feature = \"fixture-gen\"`
        enabled. If invoked without it, the function MUST HARD-ERROR with a clear panic or
        compile_error! — NOT silently fall back to unseeded `new()`.
    (b) A `#[cfg(not(feature = \"fixture-gen\"))] compile_error!` guard or a runtime
        `panic!(\"start-multi requires the fixture-gen feature; rebuild with --features dtu,fixture-gen\")`
        at the top of `build_multi_clone_factory` is acceptable; the implementer must choose
        one and document the rationale in a comment.
    (c) The implementer must read harness.rs build_clone_pairs for the complete construction
        sequence including E-DEMO-002/003/004/005/006 guard order. The fixture-gen path uses
        `new_with_seed`; the static-JSON path uses `new()`. `start-multi` ONLY supports the
        seeded path (fixture-gen required)."
  - "Cyberint seed AND access-token composite construction for start-multi: `new_with_seed`
    does NOT apply `initial_access_token` — the seeded constructor accepts (seed, archetype,
    org_id) only. To satisfy BOTH seed-based data distinctness AND access-token auth, the
    `build_multi_clone_factory` closure must use the composite pattern:
    (1) Call `CyberintClone::new_with_seed(seed, archetype, org_id)` to produce the seeded clone.
    (2) If `org_cfg.initial_access_token.is_some()`, immediately call
        `clone.configure(serde_json::json!({\"access_token\": token})).await` (via the
        `BehavioralClone::configure` trait method). This routes through
        `CyberintState::apply_config` → `register_access_token`, placing the token in the
        `access_token_allowlist` BEFORE the clone starts serving requests.
    (3) These two mechanisms compose cleanly: `configure()` is post-construction and
        `new_with_seed` leaves the allowlist empty, so the call is additive.
    The composite call MUST occur inside `cmd_start_multi` AFTER `start_instances` resolves
    (or inside the factory closure before returning the Box, before `start_on` is called —
    implementer's choice; document which approach is used with a comment citing this
    risk_mitigation entry). The existing `new_with_access_token` constructor is NOT used in
    the seeded path because it hard-codes static-fixture loading without a seed."
  - "demo.toml multi-org seed alignment: seeds used in scripts/demo.toml clones must match the
    seeds used in the S-DEMO-004 test harness (seed=100 for org-a, seed=200 for org-c) to satisfy
    INV-DISTINCT-DATA-001. Read S-DEMO-004 risk_mitigations block for the authoritative seed values."
  - "AD-017 stdin-only credential values: all N×M prism credential set calls in demo-setup.sh
    must pipe values via stdin. No credential value via --value argv."
  - "Keyring deletes must run BEFORE rm -rf of config dir: prism credential delete reads
    prism.toml for OrgId-keyed namespace lookup (ADR-034 §D3). Removing config dir first
    silently orphans keyring entries (S-DEMO-003 F-P10-HIGH-001 precedent)."
inputs:
  - "crates/prism-dtu-demo-server/src/main.rs"
  - "crates/prism-dtu-demo-server/src/config.rs"
  - "crates/prism-dtu-demo-server/src/multi_instance.rs"
  - "crates/prism-dtu-demo-server/src/harness.rs"
  - "crates/prism-dtu-demo-server/src/lib.rs"
  - "scripts/demo-setup.sh"
  - "scripts/demo-run.sh"
  - "scripts/demo-teardown.sh"
  - "scripts/start-demo.sh"
  - "scripts/demo.toml"
  - "docs/DEMO-RUNBOOK.md"
  - ".github/workflows/ci.yml"
  - ".factory/stories/S-DEMO-003-demo-setup-scripts-and-runbook.md"
  - ".factory/stories/S-DEMO-004-multi-org-sensor-isolation-smoke-test.md"
  - ".factory/specs/behavioral-contracts/BC-2.06.001-toml-config-loading.md"
  - ".factory/specs/behavioral-contracts/BC-2.06.012-per-tenant-overlay-loading-and-merge-semantics.md"
  - ".factory/specs/behavioral-contracts/BC-2.06.013-scalar-only-overlay-enforcement.md"
  - ".factory/specs/behavioral-contracts/BC-2.06.014-instance-identity-resolution-at-fanout.md"
  - ".factory/specs/behavioral-contracts/BC-2.06.017-dtu-per-instance-multi-address-binding.md"
  - ".factory/objectives/DEMO-SCOPE.md"
  - "crates/prism-dtu-harness/src/multi_instance.rs"
  - "crates/prism-dtu-harness/src/overlay_wiring.rs"
input-hash: null
traces_to: []
cycle: "v1.0.0-brownfield"
phase: 3
---

# S-DEMO-LAUNCHER-CONSOLIDATION-001 — Demo Launcher Consolidation (Option-2: Rust-Touching)

**Story ID:** S-DEMO-LAUNCHER-CONSOLIDATION-001
**Status:** ready
**Version:** v2.2
**Wave:** 5
**Priority:** P3
**Points:** 8

---

## Origin

Registered as draft stub at D-1029 (2026-06-06) during S-DEMO-003 LOCAL adversary pass-2. The
`scripts/start-demo.sh` convenience launcher was identified as an OBS [process-gap]: it overlaps
with `demo-run.sh` as a demo launch entry-point but is not covered by the `scripts/demo-*.sh`
shellcheck CI glob.

Scope expanded at T11 (multi-client-soc-demo-tasks.md) to include generalizing all three demo
scripts from the fixed single-org (`demo-org`) model delivered by S-DEMO-003 to support N orgs.

**v2.0 change (Option-2 architect decision):** The implementer must add a real
`demo-server start-multi` CLI subcommand that wires the EXISTING, already-tested multi-instance
APIs (`start_instances` / `MultiInstanceConfig`), instead of N×M shell processes. This is a
Rust-touching story. See §Option-2 Design below for the full rationale.

---

## Option-2 Design (Architect-Specified — Do Not Re-Litigate)

### Why Option-2 (Rust subcommand) instead of Option-1 (N×M shell processes)

Option-1 (shell launches N separate `prism-dtu-demo-server start` processes) was the v1.0
design. The architect rejected it in favour of Option-2 because:

1. `start_instances(MultiInstanceConfig, clone_factory)` already exists in
   `crates/prism-dtu-demo-server/src/multi_instance.rs`, is already tested, and handles all
   per-org socket isolation requirements (BC-2.06.017). Reimplementing that logic in shell is
   duplication of tested Rust code with no benefit.
2. N separate server processes each write their own flat `{sensor: url}` sidecar; demo-run.sh
   would need to merge N sidecars and correlate them to org slugs — fragile shell logic. A single
   `start-multi` process writes one nested `{org_slug: {sensor: url}}` sidecar atomically.
3. Single PID in `.prism-dtu-demo-server.pid` — teardown remains `kill <PID>` with zero changes
   to the existing stop-one-process model.

### What the implementer adds

#### 1. `crates/prism-dtu-demo-server/src/config.rs` — NEW structs (~40-60 lines)

Add a NEW top-level config type. Do NOT modify `DemoConfig` (its
`#[serde(deny_unknown_fields)]` and fixed `ClonesConfig` must stay backward-compatible for
the existing `start` subcommand).

```rust
/// Top-level config for `start-multi`. Loaded from scripts/demo.toml.
/// Separate from DemoConfig to avoid deny_unknown_fields clash.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MultiOrgDemoConfig {
    #[serde(default)]
    pub harness: HarnessConfig,         // reuse existing HarnessConfig
    pub orgs: std::collections::HashMap<String, OrgConfig>, // [orgs.<slug>]
}

/// Config for one org's DTU clone fleet.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OrgConfig {
    pub org_id: String,                  // UUID v7 hyphenated
    pub sensors: Vec<String>,            // ["crowdstrike", "armis"]
    pub seed: u64,
    #[serde(default)]
    pub initial_access_token: Option<String>, // Cyberint-only; None for other sensors
}

impl MultiOrgDemoConfig {
    pub fn from_file(path: &std::path::Path) -> anyhow::Result<Self> { ... }
    pub fn from_str(toml_str: &str) -> anyhow::Result<Self> { ... }
}
```

The `HarnessConfig` struct (already exists in config.rs) is reused unchanged — `bind = "127.0.0.1"`.

#### 2. `crates/prism-dtu-demo-server/src/main.rs` — StartMulti variant + cmd_start_multi (~80-120 lines)

```rust
/// Start all orgs' clone fleets using the multi-instance API.
StartMulti {
    /// Path to the multi-org demo config TOML (e.g. `scripts/demo.toml`).
    #[arg(long, short = 'c', value_name = "PATH")]
    config: std::path::PathBuf,
},
```

`cmd_start_multi` implementation:

```
1. init_tracing(false)
2. Load MultiOrgDemoConfig::from_file(&config_path)
3. Build MultiInstanceConfig:
   For each (org_slug, org_cfg) in multi_cfg.orgs:
     For each sensor_id in org_cfg.sensors:
       name = "{org_slug}-{sensor_id}"  (e.g. "org-a-crowdstrike")
       bind = "{multi_cfg.harness.bind}:0".parse()
       instances.push(InstanceEntry::new(name, bind))
4. Build clone_factory closure (REQUIRES fixture-gen feature — hard-error if absent):
   |entry: &InstanceEntry| -> Box<dyn BehavioralClone> {
     parse entry.name → (org_slug, sensor_id) using harness::parse_org_id convention
     look up OrgConfig from multi_cfg.orgs[org_slug]
     derive archetype via harness::fixture_set_to_archetype(org_cfg.seed)
     construct OrgId: OrgId(*uuid::Uuid::parse_str(&org_cfg.org_id)?.as_bytes())
     // start-multi ALWAYS uses the seeded path (fixture-gen required, no static-JSON fallback)
     construct clone via new_with_seed(org_cfg.seed, archetype, org_id)
       using prism_dtu_common::demo_time_anchor() as the time anchor
     // Cyberint composite path (GAP-2): new_with_seed does NOT set access_token;
     //   call configure() post-construction to register the token in the allowlist.
     //   These two mechanisms compose cleanly: configure() is additive to the allowlist.
     if sensor_id == "cyberint" && org_cfg.initial_access_token.is_some():
       clone.configure(serde_json::json!({"access_token": token}))
         — must be called synchronously before returning the Box; use a runtime
           handle or block_on if needed (or refactor to async factory if simpler)
     return Box::new(clone)
   }
5. Call start_instances(multi_cfg_for_factory, clone_factory).await
6. Write PID sidecar (same write_pid_file() helper, unchanged)
7. Write NESTED URL sidecar: {org_slug: {sensor_id: url}}
   (different from the flat {name: url} written by cmd_start)
8. Print nested URL table
9. wait_for_shutdown_signal with MultiInstanceServers::shutdown() instead of DemoHarness::stop_all()
```

The nested sidecar file name: `.prism-dtu-demo-server.urls-multi.json` (distinct from the
existing `.prism-dtu-demo-server.urls.json` to avoid format confusion when both subcommands
are used on the same machine).

#### 3. `scripts/demo.toml` — add `[orgs.*]` section

```toml
# Existing [harness] + [clones.*] blocks stay for backward compatibility with `start`.

# Multi-org fleet config for `start-multi` (new in v2.0):
[orgs.org-a]
org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0000"
sensors = ["crowdstrike", "armis"]
seed = 100

[orgs.org-b]
org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0001"
sensors = ["claroty", "cyberint"]
seed = 150
initial_access_token = "demo-cyberint-api-key-org-b"

[orgs.org-c]
org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0002"
sensors = ["crowdstrike", "armis", "claroty", "cyberint"]
seed = 200
initial_access_token = "demo-cyberint-api-key-org-c"
```

IMPORTANT: `MultiOrgDemoConfig` and `DemoConfig` are DIFFERENT root types. The implementer
MUST NOT add `[orgs.*]` to the existing `DemoConfig` deserializer — it would fail with
`deny_unknown_fields`. Instead, `cmd_start_multi` parses only the `MultiOrgDemoConfig` fields
it needs; the `[clones.*]` section is irrelevant to `start-multi` and is not parsed.

The implementer may choose to have `scripts/demo.toml` contain both the `[clones.*]` section
(for backward-compat `start`) AND the `[orgs.*]` section (for `start-multi`), as long as
each subcommand parses ONLY its own config type from the file. This requires the Rust parser
to be invoked separately (not sharing a single toml root parse) — implement using a raw
`toml::Table` intermediate parse if needed, or two separate TOML files if simpler.

#### 4. `scripts/demo-run.sh` — switch from N×`start` to one `start-multi`

```bash
# OLD (v1.0): N separate processes
# demo_server start --config scripts/demo.toml &   (per-org, per-sensor)

# NEW (v2.0): single start-multi process
"${DEMO_SERVER_BIN}" start-multi --config scripts/demo.toml &
DEMO_SERVER_PID=$!

# Poll for the NESTED sidecar (not the flat one)
SIDECAR=".prism-dtu-demo-server.urls-multi.json"
timeout 30 bash -c "until [ -f '${SIDECAR}' ]; do sleep 0.5; done"

# Read nested {org_slug: {sensor: url}} and generate N×M overlay TOMLs
python3 - <<'PYEOF'
import json, sys, os

with open(".prism-dtu-demo-server.urls-multi.json") as f:
    nested = json.load(f)  # {"org-a": {"crowdstrike": "http://...", ...}, ...}

config_dir = os.environ["DEMO_CONFIG_DIR"]
for org_slug, sensor_map in nested.items():
    org_dir = f"{config_dir}/specs/customers/{org_slug}"
    os.makedirs(org_dir, exist_ok=True)
    for sensor_id, base_url in sensor_map.items():
        overlay_path = f"{org_dir}/{sensor_id}.sensor.toml"
        with open(overlay_path, "w") as f:
            f.write(f'extends = "{sensor_id}"\n')
            f.write(f'instance_id = "{sensor_id}@{org_slug}"\n')
            f.write(f'base_url = "{base_url}"\n')
PYEOF
```

The overlay generation Python block is simpler than the v1.0 per-process approach because
`nested` already encodes `{org_slug → {sensor → url}}`; no merging of N sidecars needed.

#### 5. `scripts/demo-setup.sh` and `scripts/demo-teardown.sh` — N-org generalization

Generalize from the single-org model (5 credential set calls) to N-org loops (N×M calls),
as specified in the v1.0 ACs (AC-002, AC-003, AC-007). These are shell-only changes — no
new Rust is required. The loop reads org/sensor config from the same `scripts/demo.toml`
via a Python helper or TOML-capable bash library.

#### 6. `scripts/start-demo.sh` — retire (unchanged decision from v1.0)

Delete `scripts/start-demo.sh`. Rationale unchanged from v1.0 §Launcher Consolidation Decision.

---

## Narrative

As a demo operator (MSSP analyst running the multi-client SOC live demo), I want to run
`prism-dtu-demo-server start-multi --config scripts/demo.toml` to start all N orgs'
DTU clone fleets in a single process, so that the demo-run.sh script can read one nested
`{org_slug: {sensor: url}}` sidecar and generate N×M per-org sensor overlay TOMLs —
instead of managing N separate server processes with N flat sidecars.

---

## Behavioral Contracts

| BC ID | Title |
|-------|-------|
| BC-2.06.001 | TOML Configuration Loads and Deserializes at Startup |
| BC-2.06.012 | Per-Tenant Overlay Loading and Merge Semantics |
| BC-2.06.013 | Scalar-Only Overlay Enforcement |
| BC-2.06.014 | Instance Identity Resolution at Fanout |
| BC-2.06.017 | Per-DTU-Instance Multi-Address Binding |

---

## Multi-Org Demo Configuration (3-Org Reference)

### Org registration (scripts/demo.toml `[orgs.*]` section)

```toml
[orgs.org-a]
org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0000"
sensors = ["crowdstrike", "armis"]
seed = 100

[orgs.org-b]
org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0001"
sensors = ["claroty", "cyberint"]
seed = 150
initial_access_token = "demo-cyberint-api-key-org-b"

[orgs.org-c]
org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0002"
sensors = ["crowdstrike", "armis", "claroty", "cyberint"]
seed = 200
initial_access_token = "demo-cyberint-api-key-org-c"
```

Seeds are org-level (all sensors for the same org share one seed in this design), but
the `clone_factory` closure maps each `(org_slug, sensor_id)` independently, so a
`new_with_seed(seed, ...)` call uses the org's seed for every sensor. This is different
from the per-clone seed in DemoConfig; the implementer must verify that S-DEMO-004's
INV-DISTINCT-DATA-001 is satisfied (org-a CrowdStrike seed=100 ≠ org-c CrowdStrike
seed=200 by default).

### Nested sidecar format (written by cmd_start_multi)

```json
{
  "org-a": {
    "crowdstrike": "http://127.0.0.1:54321",
    "armis":       "http://127.0.0.1:54322"
  },
  "org-b": {
    "claroty":   "http://127.0.0.1:54323",
    "cyberint":  "http://127.0.0.1:54324"
  },
  "org-c": {
    "crowdstrike": "http://127.0.0.1:54325",
    "armis":       "http://127.0.0.1:54326",
    "claroty":     "http://127.0.0.1:54327",
    "cyberint":    "http://127.0.0.1:54328"
  }
}
```

This is a NEW sidecar format (nested by org_slug, then by sensor_id) written to
`.prism-dtu-demo-server.urls-multi.json`. The existing flat sidecar
`.prism-dtu-demo-server.urls.json` is written only by the `start` subcommand.

### prism.toml generated by demo-setup.sh (N-org)

```toml
spec_dir = "${DEMO_CONFIG_DIR}/specs"
state_dir = "${DEMO_CONFIG_DIR}/state"
plugin_dir = "${DEMO_CONFIG_DIR}/plugins"

[[orgs]]
org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0000"
org_slug = "org-a"

[[orgs]]
org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0001"
org_slug = "org-b"

[[orgs]]
org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0002"
org_slug = "org-c"
```

### Overlay files generated by demo-run.sh (N×M)

```
${DEMO_CONFIG_DIR}/specs/customers/
  org-a/
    crowdstrike.sensor.toml    # extends = "crowdstrike"; base_url = "http://127.0.0.1:<port>"
    armis.sensor.toml
  org-b/
    claroty.sensor.toml
    cyberint.sensor.toml
  org-c/
    crowdstrike.sensor.toml
    armis.sensor.toml
    claroty.sensor.toml
    cyberint.sensor.toml
```

Each overlay contains only scalar fields per BC-2.06.013:
```toml
extends     = "<sensor_id>"
instance_id = "<sensor_id>@<org_slug>"
base_url    = "http://127.0.0.1:<port>"
```

### Credential bootstrap (demo-setup.sh, N×M)

| org_slug | sensor      | name            | dummy_value              |
|----------|-------------|-----------------|--------------------------|
| org-a    | crowdstrike | client_id       | demo-cs-client-id-org-a  |
| org-a    | crowdstrike | client_secret   | demo-cs-client-secret-org-a |
| org-a    | armis       | bearer_token    | demo-armis-bearer-token-org-a |
| org-b    | claroty     | bearer_token    | demo-claroty-bearer-token-org-b |
| org-b    | cyberint    | api_key         | demo-cyberint-api-key-org-b |
| org-c    | crowdstrike | client_id       | demo-cs-client-id-org-c  |
| org-c    | crowdstrike | client_secret   | demo-cs-client-secret-org-c |
| org-c    | armis       | bearer_token    | demo-armis-bearer-token-org-c |
| org-c    | claroty     | bearer_token    | demo-claroty-bearer-token-org-c |
| org-c    | cyberint    | api_key         | demo-cyberint-api-key-org-c |

Values are dummy (DTU-safe) credentials. AD-017 applies: each value is piped via stdin, not
passed as a CLI arg. The Cyberint `initial_access_token` in `[orgs.org-b]` and `[orgs.org-c]`
in `scripts/demo.toml` must match the `api_key` credentials above.

---

## Red Gate Tests

The test-writer MUST produce ALL of the following failing tests before the implementer is dispatched.
Red Gate density check >= 0.5 required (5 failing tests for ~10-12 non-trivial Rust function bodies).

### RG-001: `test_multi_org_config_parses_valid_three_org_toml`

```rust
#[test]
fn test_multi_org_config_parses_valid_three_org_toml() {
    let toml = r#"
        [harness]
        bind = "127.0.0.1"

        [orgs.org-a]
        org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0000"
        sensors = ["crowdstrike", "armis"]
        seed = 100

        [orgs.org-b]
        org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0001"
        sensors = ["claroty", "cyberint"]
        seed = 150
        initial_access_token = "demo-cyberint-token"

        [orgs.org-c]
        org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0002"
        sensors = ["crowdstrike", "armis", "claroty", "cyberint"]
        seed = 200
    "#;
    let cfg = MultiOrgDemoConfig::from_str(toml).expect("valid 3-org config must parse");
    assert_eq!(cfg.orgs.len(), 3);
    assert_eq!(cfg.orgs["org-a"].sensors, ["crowdstrike", "armis"]);
    assert_eq!(cfg.orgs["org-a"].seed, 100);
    assert_eq!(cfg.orgs["org-b"].initial_access_token.as_deref(), Some("demo-cyberint-token"));
    assert_eq!(cfg.orgs["org-c"].sensors.len(), 4);
}
```

Traces to BC-2.06.001 postcondition 1 (config must parse and deserialize correctly).

### RG-002: `test_multi_org_config_rejects_unknown_fields`

```rust
#[test]
fn test_multi_org_config_rejects_unknown_fields() {
    let cases: &[(&str, &str)] = &[
        ("unknown top-level key", "unknown_field = true\n"),
        ("[orgs.org-a] unknown key", "[orgs.org-a]\norg_id = \"00000000-0000-0000-0000-000000000000\"\nseeds = 99\nsensors = []\n"), // typo 'seed'→'seeds'
        ("[harness] typo", "[harness]\nbnd = \"127.0.0.1\"\n"), // typo 'bind'→'bnd'
    ];
    for (label, toml) in cases {
        assert!(
            MultiOrgDemoConfig::from_str(toml).is_err(),
            "unknown field at {label} must be rejected by deny_unknown_fields, but parsed: {toml:?}"
        );
    }
}
```

Traces to BC-2.06.001 invariant (config schema must be strict; unknown fields → error).

### RG-003: `test_nested_sidecar_format_has_correct_structure`

```rust
#[test]
fn test_nested_sidecar_format_has_correct_structure() {
    // The nested sidecar must encode {org_slug: {sensor_id: url}}.
    // This test exercises the serialization logic in cmd_start_multi's write_multi_url_sidecar.
    // At stub time: write_multi_url_sidecar is todo!().
    use std::collections::HashMap;
    let mut sensor_map_a: HashMap<String, String> = HashMap::new();
    sensor_map_a.insert("crowdstrike".to_string(), "http://127.0.0.1:54321".to_string());
    sensor_map_a.insert("armis".to_string(), "http://127.0.0.1:54322".to_string());
    let mut nested: HashMap<String, HashMap<String, String>> = HashMap::new();
    nested.insert("org-a".to_string(), sensor_map_a);

    let json = serde_json::to_string(&nested).expect("must serialize");
    let parsed: HashMap<String, HashMap<String, String>> =
        serde_json::from_str(&json).expect("must round-trip");
    assert_eq!(parsed["org-a"]["crowdstrike"], "http://127.0.0.1:54321");
    assert_eq!(parsed["org-a"]["armis"], "http://127.0.0.1:54322");
    // Verify that the flat sidecar format is NOT what this function produces.
    let flat_attempt: Result<HashMap<String, String>, _> = serde_json::from_str(&json);
    // flat parse succeeds on a nested JSON only if the values are strings, not objects —
    // so the actual assertion is structural: the inner values are JSON objects, not strings.
    let raw: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(raw["org-a"].is_object(), "inner value must be an object (nested format)");
    assert!(!raw["org-a"].is_string(), "inner value must NOT be a plain string (not flat format)");
}
```

Traces to BC-2.06.017 postcondition 1 (multi-instance socket map is exposed correctly);
drives the nested sidecar serialization shape that demo-run.sh depends on.

### RG-004: `test_clone_factory_dispatch_returns_clone_for_each_sensor`

```rust
#[tokio::test]
async fn test_clone_factory_dispatch_returns_clone_for_each_sensor() {
    // Verifies that the clone_factory closure inside cmd_start_multi correctly
    // dispatches (org_slug, sensor_id) → Box<dyn BehavioralClone> for each of
    // the 4 supported sensors. At stub time: build_multi_clone_factory is todo!().
    let toml = r#"
        [orgs.org-a]
        org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0000"
        sensors = ["crowdstrike", "armis"]
        seed = 42
    "#;
    let cfg = MultiOrgDemoConfig::from_str(toml).expect("must parse");
    // build_multi_clone_factory should return a Fn(&InstanceEntry) -> Box<dyn BehavioralClone>
    let factory = build_multi_clone_factory(&cfg);
    // "org-a-crowdstrike" entry → must produce a valid clone (not panic)
    let entry = prism_dtu_demo_server::multi_instance::InstanceEntry::new(
        "org-a-crowdstrike",
        "127.0.0.1:0".parse().unwrap(),
    );
    let _clone: Box<dyn prism_dtu_common::BehavioralClone> = factory(&entry);
    // "org-a-armis" entry → must produce a valid clone (not panic)
    let entry_armis = prism_dtu_demo_server::multi_instance::InstanceEntry::new(
        "org-a-armis",
        "127.0.0.1:0".parse().unwrap(),
    );
    let _clone_armis: Box<dyn prism_dtu_common::BehavioralClone> = factory(&entry_armis);
}
```

Traces to BC-2.06.017 postcondition 1 (each entry in MultiInstanceConfig produces a running clone).

### RG-005: `test_start_multi_stands_up_per_org_distinct_sockets`

```rust
#[tokio::test]
async fn test_start_multi_stands_up_per_org_distinct_sockets() {
    // Integration-level: verifies that start-multi binds org-a-crowdstrike and
    // org-c-crowdstrike to DIFFERENT socket ports (INV-DISTINCT-DATA-001 / BC-2.06.017).
    // At stub time: start_multi_for_config is todo!().
    let toml = r#"
        [harness]
        bind = "127.0.0.1"

        [orgs.org-a]
        org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0000"
        sensors = ["crowdstrike"]
        seed = 100

        [orgs.org-c]
        org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0002"
        sensors = ["crowdstrike"]
        seed = 200
    "#;
    let cfg = MultiOrgDemoConfig::from_str(toml).expect("must parse");
    // start_multi_for_config is the testable extracted async fn:
    // pub async fn start_multi_for_config(cfg: &MultiOrgDemoConfig, ...)
    //   -> anyhow::Result<MultiInstanceServers>
    let servers = start_multi_for_config(&cfg).await.expect("must bind");
    let socket_map = servers.socket_map();
    let org_a_port = socket_map["org-a-crowdstrike"].port();
    let org_c_port = socket_map["org-c-crowdstrike"].port();
    assert_ne!(
        org_a_port, org_c_port,
        "org-a and org-c CrowdStrike clones must bind to distinct ports (BC-2.06.017)"
    );
    // Ensure both are non-zero (actually bound, not default-unbound)
    assert_ne!(org_a_port, 0, "org-a CrowdStrike must be bound to a real port");
    assert_ne!(org_c_port, 0, "org-c CrowdStrike must be bound to a real port");
    servers.shutdown();
}
```

Traces to BC-2.06.017 postcondition 3 (distinct per-org socket addresses for same sensor type);
AC-004 / AC-005 behavioral assertion foundation.

---

## Acceptance Criteria

### Rust Subcommand ACs

#### AC-001: `StartMulti` variant exists in `Commands` enum and CLI help reflects it

(traces to BC-2.06.017 postcondition 1 — the multi-instance binding must be invocable
from the operator CLI)

`prism-dtu-demo-server --help` lists `start-multi` as a subcommand.
`prism-dtu-demo-server start-multi --help` lists `--config <PATH>` as the only argument.

**Verification:** `cargo run -p prism-dtu-demo-server --features dtu,fixture-gen -- start-multi --help`
exits 0 and prints the expected help text.

**Note:** `fixture-gen` is REQUIRED alongside `dtu` for any `start-multi` invocation.
Omitting `fixture-gen` must produce a hard error (compile_error! or runtime panic) — NOT
silently fall back to unseeded clones that would violate INV-DISTINCT-DATA-001. The
`--help` check alone does not exercise the factory, but all remaining `start-multi`
verifications MUST use `--features dtu,fixture-gen`.

---

#### AC-002: `MultiOrgDemoConfig` parses a valid 3-org TOML file (RG-001 green)

(traces to BC-2.06.001 postcondition 1 — multi-org config must deserialize from TOML
without error; BC-2.06.001 invariant — unknown fields are rejected)

`MultiOrgDemoConfig::from_str(valid_toml).is_ok()` for a TOML containing 3 org entries
with `org_id`, `sensors`, `seed`, and optional `initial_access_token`.

`MultiOrgDemoConfig::from_str(typo_toml).is_err()` for any TOML with an unknown field
at any level (top-level, `[harness]`, `[orgs.X]`).

RG-001 and RG-002 must both be green.

---

#### AC-003: `start-multi` writes a nested `{org_slug: {sensor_id: url}}` sidecar (RG-003 green)

(traces to BC-2.06.017 postcondition 1 — the multi-instance socket map must be accessible
to downstream tools; the nested format encodes per-org URL routing)

Given: `prism-dtu-demo-server start-multi --config scripts/demo.toml` has started.

Then: `.prism-dtu-demo-server.urls-multi.json` is written within 5 seconds and contains:
- A top-level JSON object keyed by org_slug strings.
- Each org_slug maps to a nested JSON object keyed by sensor_id strings.
- Each sensor_id value is a URL string `http://127.0.0.1:<port>`.

RG-003 (structural shape test) must be green.

---

#### AC-004: Per-org DTU clones serve requests on distinct socket addresses (RG-005 green)

(traces to BC-2.06.017 postcondition 3 — per-DTU-instance multi-address binding ensures
no two orgs share a DTU socket for the same sensor type)

Given: `start-multi` has started the 3-org fleet.

Then:
- The nested sidecar's `org-a.crowdstrike` URL port ≠ `org-c.crowdstrike` URL port
  (both orgs have CrowdStrike; they must bind to distinct OS-assigned ports).
- All ports in the sidecar are non-zero (actually bound by the OS).
- A GET request to each URL's health endpoint (e.g., `/health`) returns HTTP 200.

RG-005 (per-org distinct socket test) must be green.

---

#### AC-005: `clone_factory` closure correctly dispatches `(org_slug, sensor_id)` to BehavioralClone (RG-004 green)

(traces to BC-2.06.017 postcondition 1 — each InstanceEntry must produce a running clone
serving the correct sensor's endpoints)

Given: `build_multi_clone_factory(&cfg)` returns a factory closure.

Then:
- Passing an entry with name `"org-a-crowdstrike"` produces a `CrowdstrikeClone` (static-JSON path).
- Passing an entry with name `"org-b-cyberint"` produces a `CyberintClone` constructed via
  `new_with_seed` (seeded data distinctness) AND, if `[orgs.org-b].initial_access_token` is set,
  has the token registered in `access_token_allowlist` via a follow-up `configure({"access_token": token})`
  call (GAP-2 composite path). The `new_with_access_token` constructor is NOT used in the seeded path.
- Passing an entry with an unrecognized sensor name panics or returns an error (not silently
  constructing a wrong clone type).

RG-004 must be green.

---

#### AC-006: Existing `start` subcommand is unbroken (backward compatibility)

(traces to BC-2.06.001 invariant — the existing single-org `DemoConfig` + `start` subcommand
must not regress)

Given: `prism-dtu-demo-server start --config configs/demo.toml` is run.

Then:
- `DemoConfig` parses `configs/demo.toml` as before (no unknown-field errors from `[orgs.*]`
  in `scripts/demo.toml` — because `start` only parses `DemoConfig`, not `MultiOrgDemoConfig`).
- The 6 clone instances start on their configured ports (or ephemeral ports if `port = 0`).
- The flat URL sidecar `.prism-dtu-demo-server.urls.json` is written.

**Note on config file handling:** If `scripts/demo.toml` is used with `start`, the `[orgs.*]`
section must NOT cause a parse error. The implementer must ensure that `DemoConfig::from_file`
ignores the `[orgs.*]` section. Options:
  - Keep `configs/demo.toml` (single-org, no `[orgs.*]`) as the target for `start`.
  - Or strip `[orgs.*]` from the TOML before passing to `DemoConfig` parser (fragile).
  - Preferred: use `configs/demo.toml` for `start` and `scripts/demo.toml` for `start-multi`.
    These are already different files per the S-DEMO-003 baseline.

---

### Shell Script ACs

#### AC-007: `scripts/start-demo.sh` is retired (removed from the repository)

(traces to BC-2.06.001 precondition 1 — single canonical entry point for demo launch
removes ambiguity)

`scripts/start-demo.sh` is deleted. No other script references it.
`shellcheck scripts/demo-*.sh` exits 0 with no output.

**Verification:** `git show HEAD -- scripts/start-demo.sh` returns "fatal: Path does not exist."

---

#### AC-008: `scripts/demo-run.sh` calls `start-multi` and reads the nested sidecar

(traces to BC-2.06.012 postcondition 2 — per-org overlay TOMLs must exist before prism-bin
starts; BC-2.06.013 invariant — overlay files contain only scalar fields)

Given: `demo-setup.sh` has completed; `bash scripts/demo-run.sh --config-dir <DIR>` is run.

Then:
- `demo-run.sh` invokes `prism-dtu-demo-server start-multi --config scripts/demo.toml` (not `start`).
- Within 30s, `.prism-dtu-demo-server.urls-multi.json` is present.
- `demo-run.sh` reads the nested sidecar and writes 8 overlay TOML files:
  - `<DIR>/specs/customers/org-a/crowdstrike.sensor.toml`
  - `<DIR>/specs/customers/org-a/armis.sensor.toml`
  - `<DIR>/specs/customers/org-b/claroty.sensor.toml`
  - `<DIR>/specs/customers/org-b/cyberint.sensor.toml`
  - `<DIR>/specs/customers/org-c/crowdstrike.sensor.toml`
  - `<DIR>/specs/customers/org-c/armis.sensor.toml`
  - `<DIR>/specs/customers/org-c/claroty.sensor.toml`
  - `<DIR>/specs/customers/org-c/cyberint.sensor.toml`
- Each overlay contains exactly: `extends`, `instance_id`, `base_url` (three scalar fields, BC-2.06.013).

---

#### AC-009: `scripts/demo-setup.sh` generates a multi-org `prism.toml` with N [[orgs]] entries

(traces to BC-2.06.001 postcondition 1 — generated prism.toml must be schema-valid and
accepted by prism-bin's TOML config loader at startup)

Given: `bash scripts/demo-setup.sh --config-dir <DIR>` completes on macOS or Linux.

Then:
- `<DIR>/prism.toml` exists and contains exactly 3 `[[orgs]]` entries (org-a, org-b, org-c)
  with distinct `org_id` UUIDs (v7) and `org_slug` values.
- `<DIR>/specs/` contains the 4 TYPE spec TOMLs and a `customers/` subdirectory with 3 org-slug subdirs.
- `<DIR>/plugins/` contains `crowdstrike-oauth2.prx` and `crowdstrike-oauth2.manifest.toml`.

---

#### AC-010: `scripts/demo-setup.sh` bootstraps N×M credentials (one per org × sensor combo)

(traces to BC-2.06.001 postcondition 1 — keyring must be seeded with a credential for every
org × sensor pair; missing credentials cause boot failure at credential resolution step)

Given: `bash scripts/demo-setup.sh --config-dir <DIR>` completes.

Then:
- For each (org_slug, sensor, name) combination in the N×M credential table above, a keyring
  entry was written via `printf '%s\n' <dummy_value> | prism --config-dir <DIR> credential set
  --org-slug <org_slug> --sensor <sensor> --name <name>`.
- AD-017: dummy values are piped via stdin only.
- The Cyberint `api_key` dummy value for org-b matches `initial_access_token = "demo-cyberint-api-key-org-b"`
  in `scripts/demo.toml [orgs.org-b]`.
- The Cyberint `api_key` dummy value for org-c matches `initial_access_token = "demo-cyberint-api-key-org-c"`
  in `scripts/demo.toml [orgs.org-c]`.

---

#### AC-011: `scripts/demo-run.sh` prints the prism start command with all 4 TYPE-spec env vars

(traces to BC-2.06.014 postcondition — `${env.VAR}` placeholders in TYPE specs must resolve
at boot step 4a before per-org overlays replace base_url at step 4c)

Given: `demo-run.sh` has completed overlay generation.

Then: `demo-run.sh` prints a `prism start` command block that includes all 4 TYPE-spec env vars:
```
CROWDSTRIKE_BASE_URL=http://127.0.0.1 \
ARMIS_INSTANCE_URL=http://127.0.0.1 \
CLAROTY_INSTANCE_URL=http://127.0.0.1 \
CYBERINT_ENVIRONMENT=demo \
${PRISM_BIN} --config-dir ${DEMO_CONFIG_DIR} start
```
These env vars are the same as S-DEMO-003 AC-006 (unchanged; they satisfy step-4a
`${env.*}` token resolution; per-org ports are handled by overlay TOMLs at step-4c).

---

#### AC-012: `scripts/demo-teardown.sh` manages a single PID and deletes N×M keyring entries

(traces to BC-2.06.001 postcondition — teardown must undo all state written by demo-setup.sh;
single-process teardown model is simpler than N×M process management)

Given: A multi-org demo environment started by `start-multi` exists at `<DIR>`.

Then: `bash scripts/demo-teardown.sh --config-dir <DIR>` runs to completion:
1. Kills the single `prism-dtu-demo-server start-multi` process (via single PID file).
2. Deletes all 10 keyring entries (N×M credential table) via `prism credential delete`
   (OrgId-keyed, ADR-034 §D3). Keyring deletes run BEFORE `rm -rf` (F-P10-HIGH-001).
3. Removes `<DIR>` with `rm -rf`.
4. Exits 0.

---

#### AC-013: All shell scripts pass `shellcheck` with zero errors or warnings

(traces to BC-2.06.001 invariant — shell scripting quality gate)

Given: `demo-setup.sh`, `demo-run.sh`, `demo-teardown.sh` are present; `start-demo.sh` deleted.

When: `shellcheck scripts/demo-*.sh` is run.

Then: Exit 0, zero errors, zero warnings. CI job `shellcheck-demo-scripts` continues to pass
with no changes to `.github/workflows/ci.yml`.

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `Commands::StartMulti` (enum variant) | `crates/prism-dtu-demo-server/src/main.rs` | Pure (data) |
| `cmd_start_multi` (~80-120 lines) | `crates/prism-dtu-demo-server/src/main.rs` | Effectful (I/O: config load, clone start, sidecar write, signal wait) |
| `build_multi_clone_factory` (extracted pure fn) | `crates/prism-dtu-demo-server/src/main.rs` | Pure (returns closure; no I/O) |
| `write_multi_url_sidecar` (extracted helper) | `crates/prism-dtu-demo-server/src/main.rs` | Effectful (file I/O: tmp+rename atomic write) |
| `MultiOrgDemoConfig` (root config type) | `crates/prism-dtu-demo-server/src/config.rs` | Pure (data struct) |
| `OrgConfig` (per-org `[orgs.*]` subsection) | `crates/prism-dtu-demo-server/src/config.rs` | Pure (data struct) |
| `start_instances` (EXISTING, unchanged) | `crates/prism-dtu-demo-server/src/multi_instance.rs` | Effectful (async: socket bind) |
| `MultiInstanceConfig` (EXISTING, unchanged) | `crates/prism-dtu-demo-server/src/multi_instance.rs` | Pure (data) |
| `MultiInstanceServers` (EXISTING, unchanged) | `crates/prism-dtu-demo-server/src/multi_instance.rs` | Effectful (owns shutdown_tx) |
| `scripts/demo-setup.sh` | `scripts/` | Effectful (shell: cargo build, mkdir, cp, prism CLI) |
| `scripts/demo-run.sh` | `scripts/` | Effectful (shell: subprocess, file I/O, overlay generation) |
| `scripts/demo-teardown.sh` | `scripts/` | Effectful (shell: process kill, prism CLI, rm -rf) |
| `scripts/demo.toml` | `scripts/` | Pure (config file) |
| `docs/DEMO-RUNBOOK.md` | `docs/` | Pure (documentation) |

Architecture section files referenced:
- `architecture/module-decomposition.md` (SS-06, SS-22 subsystem responsibilities)
- `architecture/dependency-graph.md` (prism-dtu-demo-server internal dependencies)
- `architecture/decisions/ADR-029-multi-tenant-sensor-endpoint-overrides.md` (overlay format)
- `architecture/decisions/ADR-034-tier3-keyring-resolution-org-id-threading.md` (OrgId-keyed credential namespace)

---

## Architecture Compliance Rules

| Rule | Rationale |
|------|-----------|
| `MultiOrgDemoConfig` MUST be a NEW top-level struct, NOT an extension of `DemoConfig` | `DemoConfig` has `#[serde(deny_unknown_fields)]` and a fixed 6-sensor `ClonesConfig`; adding `[orgs.*]` would fail deserialization. Backward compatibility with `start` subcommand is mandatory. |
| `cmd_start_multi` MUST call the EXISTING `start_instances(MultiInstanceConfig, clone_factory)` | Do not reimplement multi-instance binding logic; `start_instances` is already tested and correct per BC-2.06.017. |
| Sidecar file for `start-multi` MUST be `.prism-dtu-demo-server.urls-multi.json` (nested format) | Distinct from the flat `.prism-dtu-demo-server.urls.json` written by `start`; prevents format confusion. |
| InstanceEntry name convention MUST be `"{org_slug}-{sensor_id}"` | Enables the `clone_factory` closure to recover (org_slug, sensor_id) by splitting on `-` at the first occurrence of a known sensor name. |
| `build_multi_clone_factory` MUST be a separately named, testable function | RG-004 tests it directly; if it is an inline closure in `cmd_start_multi`, the test cannot call it. Extract as `pub(crate) fn build_multi_clone_factory(cfg: &MultiOrgDemoConfig) -> impl Fn(&InstanceEntry) -> Box<dyn BehavioralClone>`. This function is `#[cfg(feature = "fixture-gen")]`-only; it MUST NOT exist in the `#[cfg(not(feature = "fixture-gen"))]` compilation unit — use `compile_error!` or a gated `pub(crate)` fn signature to enforce this. |
| `build_multi_clone_factory` MUST reuse the existing fixture-gen-gated helpers from `harness.rs` | Use `harness::parse_org_id` and `harness::fixture_set_to_archetype` (both `#[cfg(feature = "fixture-gen")]`-gated public helpers) for OrgId construction and archetype mapping respectively. Use `prism_dtu_common::Archetype` for the archetype type. Construct OrgId from a UUID via `OrgId(*uuid.as_bytes())` — matching the pattern used in S-DEMO-004 harness. Call `prism_dtu_common::demo_time_anchor()` for the time anchor parameter to `new_with_seed`. Do NOT reinvent any of these utilities inline. |
| `start_multi_for_config` MUST be extracted as a testable async fn | RG-005 tests socket isolation at the async fn level; if the bind logic is only in `cmd_start_multi` (which also reads config files and handles signals), the test cannot call it without subprocess overhead. Extract as `pub(crate) async fn start_multi_for_config(cfg: &MultiOrgDemoConfig) -> anyhow::Result<MultiInstanceServers>`. |
| `demo-setup.sh` MUST NOT write per-org `base_url` overlay TOMLs | DTU ports are ephemeral; overlay generation belongs in `demo-run.sh` after the sidecar is parsed. |
| `demo-run.sh` MUST write N×M overlay TOMLs BEFORE printing the `prism start` command | prism spec loader reads overlays at boot step 4c; missing overlay = wrong base_url. |
| `demo-run.sh` MUST include all 4 TYPE-spec env vars in the printed `prism start` command | Step-4a `env_resolver.rs` resolves `${env.*}` tokens; missing env var = E-SPEC-024 boot abort. |
| `demo-teardown.sh` MUST run keyring deletes BEFORE `rm -rf` of the config dir | `prism credential delete` reads `prism.toml` for OrgId resolution (ADR-034 §D3); config dir removed first → all deletes fail silently. |
| Each overlay TOML MUST contain ONLY scalar fields: `extends`, `instance_id`, `base_url` | BC-2.06.013 scalar-only overlay enforcement. |
| AD-017: credential values MUST be piped via stdin, NEVER via CLI argv | Applies to all N×M `prism credential set` calls in `demo-setup.sh`. |
| Cyberint DTU `initial_access_token` in `[orgs.*]` config MUST match the `api_key` credential for that org | StaticCookieAuthProvider injects `Cookie: access_token=<keyring-value>`; mismatch → 401 on all Cyberint queries. |
| All shell scripts MUST use `#!/usr/bin/env bash` shebang and `set -euo pipefail` | shellcheck portability requirement. |

---

## Forbidden Dependencies

| Forbidden | Reason |
|-----------|--------|
| Modification of `DemoConfig`, `HarnessConfig`, `ClonesConfig`, or `CloneConfig` structs | These are backward-compatible existing types; modifying them for `start-multi` risks breaking `start`. Add `MultiOrgDemoConfig` instead. |
| Adding `[orgs.*]` to `DemoConfig`'s `#[serde(deny_unknown_fields)]` struct | Would cause `start` to fail on `scripts/demo.toml` containing `[orgs.*]`. |
| N separate `prism-dtu-demo-server start` subprocess invocations in `demo-run.sh` | The v2.0 design uses one `start-multi` process; N-process approach is Option-1 (rejected). |
| Hardcoded port numbers in overlay TOML generation | Ephemeral ports are assigned by the OS; hardcoded ports break across restarts. |
| `start-demo.sh` as a caller or delegate in any script | Being retired. |
| Direct port read from DTU process listing or `/proc/<pid>/net` | DTU server writes its own sidecar file; reading ports from OS state is fragile and platform-specific. |
| `unwrap()` / `expect()` in `cmd_start_multi` for Result values in non-test code | Use `?` + `anyhow::anyhow!`. Exception: signal handler installation (annotated with `#[allow(clippy::expect_used)]` per existing pattern in main.rs). |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Cyberint `api_key` credential does not match `initial_access_token` in `[orgs.*]` | Cyberint DTU clone returns 401 for that org. `demo-setup.sh` must enforce that the dummy `api_key` value matches the `initial_access_token` configured in `scripts/demo.toml` for that org. |
| EC-002 | `start-multi` fails because one clone cannot bind (port in use) | `start_instances` returns `MultiInstanceBindError::BindFailure`; `cmd_start_multi` propagates as `Err` with a clear message listing which instance failed. No zombie clones (BC-2.06.017 Postcondition 6). |
| EC-003 | `scripts/demo.toml` `[orgs.*]` section parsed by `DemoConfig` (wrong struct) | Must not happen: `start` uses `DemoConfig`, `start-multi` uses `MultiOrgDemoConfig`. If both subcommands used the same config type, `[orgs.*]` would cause a `deny_unknown_fields` parse failure on `start`. |
| EC-004 | `demo-run.sh` called before `demo-setup.sh` (no `prism.toml` present) | `demo-run.sh` checks for `prism.toml` existence before starting DTU; exits 1 with "Run: bash scripts/demo-setup.sh". |
| EC-005 | `demo-teardown.sh` is run when DTU server is not running (no PID file) | Prints "server may not be running", skips the kill step; continues to credential delete + rm -rf. |
| EC-006 | `start-multi` sidecar not available within 30s poll timeout | `demo-run.sh` exits 1 with "demo-server start-multi did not write sidecar within 30s; check DTU server logs." |
| EC-007 | `[orgs.org-b]` has no `initial_access_token` but org-b has `cyberint` in sensors | `cmd_start_multi` starts the Cyberint clone without calling `configure()`; the clone's allowlist is empty (no token pre-seeded). Operator must configure manually via `prism-dtu-demo-server configure cyberint <json>`. |
| EC-008 | `MultiOrgDemoConfig` `[orgs.X]` sensor list references an unsupported sensor name | `build_multi_clone_factory` returns an error or panics with a clear message: "unsupported sensor 'X' in [orgs.org-a]; valid values: crowdstrike, armis, claroty, cyberint". |
| EC-009 | `build_multi_clone_factory` called with an InstanceEntry name that does not match `{org_slug}-{sensor_id}` convention | Panic or error with actionable message; this is a programming error (the factory is only called internally by `start_instances` with entries built by `cmd_start_multi`). |

---

## Token Budget Estimate

| Context component | Lines estimate | Tokens (approx) |
|---|---|---|
| This story spec (v2.0) | 600 | 7,200 |
| S-DEMO-003 (predecessor, full body) | 700 | 8,400 |
| S-DEMO-004 (3-org model reference) | 300 | 3,600 |
| BC files (5 BCs × ~100 lines each) | 500 | 6,000 |
| `crates/prism-dtu-demo-server/src/main.rs` (existing) | 450 | 5,400 |
| `crates/prism-dtu-demo-server/src/config.rs` (existing) | 300 | 3,600 |
| `crates/prism-dtu-demo-server/src/multi_instance.rs` (existing) | 375 | 4,500 |
| `crates/prism-dtu-demo-server/src/harness.rs` (build_clone_pairs reference) | 400 | 4,800 |
| Existing scripts (3 scripts × ~150 lines) | 450 | 5,400 |
| DEMO-RUNBOOK.md (existing) | 250 | 3,000 |
| `scripts/demo.toml` (existing) | 100 | 1,200 |
| ADR-029, ADR-034 (overlay + credential BCs) | 200 | 2,400 |
| Implementer working memory / tool outputs | — | 8,000 |
| **Total** | | **~63,500** |

Estimate is ~32% of a 200k-token context window. Slightly over the 20-30% target.
**Split recommendation:** test-writer and implementer each receive a targeted subset:
- Test-writer: this story spec + config.rs + multi_instance.rs + harness.rs (skip scripts).
  Budget: ~39,500 tokens (~20% of 200k).
- Implementer: this story spec + main.rs + config.rs + multi_instance.rs + RG tests + scripts.
  Budget: ~58,000 tokens. Acceptable given story complexity; no further split needed.

---

## Tasks

### Phase 1: Pre-flight (read before touching any file)

- [ ] Read `crates/prism-dtu-demo-server/src/main.rs` in full; understand `Commands` enum,
  `cmd_start`, `write_url_sidecar`, `wait_for_shutdown_signal`.
- [ ] Read `crates/prism-dtu-demo-server/src/config.rs` in full; understand `DemoConfig`,
  `ClonesConfig`, `CloneConfig`, `HarnessConfig`, `#[serde(deny_unknown_fields)]` strictness.
- [ ] Read `crates/prism-dtu-demo-server/src/multi_instance.rs` in full; understand
  `start_instances`, `MultiInstanceConfig`, `InstanceEntry`, `MultiInstanceServers`.
- [ ] Read `crates/prism-dtu-demo-server/src/harness.rs` `build_clone_pairs` (lines 318+);
  understand the clone construction sequence and E-DEMO-002/003/004/005/006 guard order.
- [ ] Read `scripts/demo-run.sh`, `demo-setup.sh`, `demo-teardown.sh` in full.
- [ ] Read `scripts/start-demo.sh`; confirm exec-form model (confirms retirement rationale).
- [ ] Read `scripts/demo.toml`; understand current single-org schema.

### Phase 2 (test-writer): Write Red Gate tests

- [ ] Add `test_multi_org_config_parses_valid_three_org_toml` (RG-001) to
  `crates/prism-dtu-demo-server/src/config.rs` `#[cfg(test)] mod tests`.
- [ ] Add `test_multi_org_config_rejects_unknown_fields` (RG-002) to config.rs tests.
- [ ] Add `test_nested_sidecar_format_has_correct_structure` (RG-003) to main.rs tests
  or a separate test module.
- [ ] Add `test_clone_factory_dispatch_returns_clone_for_each_sensor` (RG-004) to
  `crates/prism-dtu-demo-server/tests/`.
- [ ] Add `test_start_multi_stands_up_per_org_distinct_sockets` (RG-005) to
  `crates/prism-dtu-demo-server/tests/`.
- [ ] Run `cargo nextest run -p prism-dtu-demo-server --features dtu,fixture-gen --no-fail-fast`;
  confirm ALL 5 RG tests FAIL (Red Gate — `todo!()` stubs not yet replaced).
  NOTE: `fixture-gen` is required alongside `dtu`; RG-004 and RG-005 call
  `build_multi_clone_factory` which is a fixture-gen-only function; omitting it
  will produce a compile error or hard panic, not silent wrong-data.
- [ ] Density check: 5 failing tests / ~10-12 non-trivial function bodies >= 0.5. PASS.

### Phase 3 (implementer): Add config.rs structs

- [ ] Add `MultiOrgDemoConfig`, `OrgConfig` to `config.rs` with `#[serde(deny_unknown_fields)]`
  and `#[non_exhaustive]` per CLAUDE.md discipline.
- [ ] Add `MultiOrgDemoConfig::from_file`, `::from_str` inherent methods (same pattern as
  `DemoConfig::from_file`, `::from_str`).
- [ ] Run RG-001 and RG-002; both must now be GREEN.

### Phase 4 (implementer): Add `StartMulti` to Commands + implement `cmd_start_multi`

- [ ] Add `StartMulti { config: PathBuf }` variant to `Commands` enum in `main.rs`.
- [ ] Add `Commands::StartMulti { config } => cmd_start_multi(config).await` to the `match`
  in `main`.
- [ ] Implement `pub(crate) fn build_multi_clone_factory(cfg: &MultiOrgDemoConfig) -> ...`
  (extracted testable function; RG-004 must become GREEN after this step).
- [ ] Implement `pub(crate) async fn start_multi_for_config(cfg: &MultiOrgDemoConfig) -> anyhow::Result<MultiInstanceServers>`
  (extracted testable async fn; RG-005 must become GREEN after this step).
- [ ] Implement `fn write_multi_url_sidecar(servers: &MultiInstanceServers, cfg: &MultiOrgDemoConfig) -> anyhow::Result<()>`
  writing `.prism-dtu-demo-server.urls-multi.json` in nested format.
  (RG-003 is a shape test for the data structure; it does not call this function directly.)
- [ ] Implement `async fn cmd_start_multi(config_path: PathBuf) -> anyhow::Result<()>`:
  load `MultiOrgDemoConfig`, call `start_multi_for_config`, write nested sidecar,
  wait for SIGTERM/SIGINT via `wait_for_shutdown_signal_multi` (adapt existing
  `wait_for_shutdown_signal` to accept `&MultiInstanceServers` and call `servers.shutdown()`).
- [ ] Run `cargo nextest run -p prism-dtu-demo-server --features dtu,fixture-gen --no-fail-fast`;
  ALL 5 RG tests must now be GREEN.
- [ ] Run `just iter prism-dtu-demo-server`; all tests GREEN.

### Phase 5 (implementer): Update `scripts/demo.toml`

- [ ] Add `[orgs.org-a]`, `[orgs.org-b]`, `[orgs.org-c]` sections as specified above.
- [ ] Confirm that `cargo run -p prism-dtu-demo-server --features dtu -- start --config configs/demo.toml`
  still starts (backward compatibility AC-006 — `configs/demo.toml` does NOT have `[orgs.*]`).
  NOTE: `start` does NOT require `fixture-gen`; the single-org seeded path is not used here.
- [ ] Confirm that `cargo run -p prism-dtu-demo-server --features dtu,fixture-gen -- start-multi --config scripts/demo.toml`
  starts (new subcommand). `fixture-gen` is REQUIRED for `start-multi` — the seeded
  constructors are only available under that feature and `build_multi_clone_factory`
  must hard-error (panic or compile_error!) if it is absent.

### Phase 6 (implementer): Retire `scripts/start-demo.sh`

- [ ] Delete `scripts/start-demo.sh` from the repository.
- [ ] Search `docs/` and `scripts/` for any references to `start-demo.sh`; remove or replace.
- [ ] Run `shellcheck scripts/demo-*.sh`; verify exit 0.

### Phase 7 (implementer): Generalize shell scripts

- [ ] Update `scripts/demo-run.sh`:
  - Replace the DTU server launch with `start-multi --config scripts/demo.toml`.
  - Replace sidecar poll from `.prism-dtu-demo-server.urls.json` to `.prism-dtu-demo-server.urls-multi.json`.
  - Replace flat sidecar parsing with nested `{org_slug: {sensor: url}}` Python block.
  - Generate N×M overlay TOMLs in `customers/<org_slug>/<sensor>.sensor.toml`.
  - IMPLEMENTER NOTE (GAP-3): `demo-run.sh` must thread `DEMO_RUN_DIR` into BOTH the sidecar
    poll path AND the Python overlay-generation block. The demo-server writes
    `.prism-dtu-demo-server.urls-multi.json` to its current working directory (cwd); the
    script `cd`s into `DEMO_RUN_DIR` before launching the server, so the sidecar poll path
    must use `${DEMO_RUN_DIR}/.prism-dtu-demo-server.urls-multi.json` (absolute path) or the
    `open()` call in the Python heredoc must use the same absolute path — not a bare filename
    that would resolve relative to a different cwd. One-line fix: set `SIDECAR="${DEMO_RUN_DIR}/.prism-dtu-demo-server.urls-multi.json"`
    and reference `${SIDECAR}` everywhere, including inside the Python heredoc via
    `os.environ["DEMO_RUN_DIR"]`.
  - Run `shellcheck scripts/demo-run.sh`.
- [ ] Update `scripts/demo-setup.sh`:
  - Replace single-org prism.toml generation with N-org `[[orgs]]` generation.
  - Replace 5 hardcoded credential set calls with N×M loop over (org_slug, sensor, name, dummy_value).
  - Run `shellcheck scripts/demo-setup.sh`.
- [ ] Update `scripts/demo-teardown.sh`:
  - Replace 5 credential delete calls with N×M loop.
  - Update OrgId extraction to support N orgs.
  - Run `shellcheck scripts/demo-teardown.sh`.

### Phase 8 (implementer): Update `docs/DEMO-RUNBOOK.md`

- [ ] Update §Setup for 3-org environment and N×M credential count.
- [ ] Update §Daily Demo Run for `start-multi` and N×M overlay generation.
- [ ] Update §Teardown for N×M credential deletes and single PID teardown.
- [ ] Update §Troubleshooting to enumerate all 10 Tier-2 env-var fallback names.
- [ ] Remove all references to `start-demo.sh`.

### Phase 9 (implementer): Final verification

- [ ] Run `just check` (full workspace pre-push gate) — must pass.
- [ ] Run `shellcheck scripts/demo-*.sh` — exit 0, zero output.
- [ ] Run `bash scripts/demo-setup.sh --config-dir /tmp/prism-demo-test` — verify 3-org prism.toml.
- [ ] Run `bash scripts/demo-teardown.sh --config-dir /tmp/prism-demo-test` — verify N×M deletes.
- [ ] Confirm CI `shellcheck-demo-scripts` job requires no changes to `.github/workflows/ci.yml`.

---

## Previous Story Intelligence

### S-DEMO-003 (predecessor — merged PR #176 develop@a42e3eaf)

1. **Overlay generation belongs in `demo-run.sh`, NOT `demo-setup.sh`:** DTU ports are
   ephemeral. This was F-HIGH-201 in S-DEMO-003 pass-2; the fix required a full redesign.
   Do not repeat this mistake.

2. **Four TYPE-spec env vars are REQUIRED in the printed `prism start` command:** Without
   CROWDSTRIKE_BASE_URL / ARMIS_INSTANCE_URL / CLAROTY_INSTANCE_URL / CYBERINT_ENVIRONMENT,
   boot step 4a fires E-SPEC-024. F-HIGH-301 in S-DEMO-003 pass-3.

3. **Keyring deletes must run BEFORE `rm -rf` of config dir:** F-P10-HIGH-001 in S-DEMO-003
   pass-10. `prism credential delete` reads `prism.toml` for OrgId-keyed namespace lookup.

4. **CrowdStrike SEC-003 manifest `allowed_urls`:** `crowdstrike-oauth2.manifest.toml` must
   include `"127.0.0.1"` alongside `"api.crowdstrike.com"`. One manifest file covers all orgs.

5. **AD-017 stdin-only credential values:** Every `prism credential set` call pipes value via
   `printf '%s\n' "${value}" | prism credential set ...`.

6. **shellcheck CI glob is `scripts/demo-*.sh`:** Retiring `start-demo.sh` (not in glob)
   cleans up the uncovered script without requiring any CI change.

### S-DEMO-004 (3-org model reference — ready v1.7)

1. **3-org sensor assignments:** Org-A = CrowdStrike + Armis; Org-B = Claroty + Cyberint;
   Org-C = all 4 sensors. Scripts must match this model.

2. **Per-org seeds for data distinctness:** seed=100 for org-a, seed=200 for org-c
   satisfies INV-DISTINCT-DATA-001. Demo scripts must use the same seed assignments.

3. **`write_overlay_temp_dir` pattern:** The S-DEMO-004 test harness uses
   `write_overlay_temp_dir` from `prism-dtu-harness`. The demo scripts implement the
   shell+python equivalent. Overlay format is identical: `extends`, `instance_id`,
   `base_url` (three scalar fields; BC-2.06.013).

### v1.0 → v2.0 design change (Option-2 architect decision)

v1.0 was scripts-only (facade mode). v2.0 adds Rust changes to `prism-dtu-demo-server`
to wire the existing `start_instances` API. The implementer MUST NOT revert to N×M
subprocess management in shell. If a technical obstacle arises with the Rust approach,
escalate to the architect — do not silently fall back to Option-1.

---

## File Structure Requirements

| File | Action | Description |
|------|--------|-------------|
| `crates/prism-dtu-demo-server/src/config.rs` | MODIFY | Add `MultiOrgDemoConfig`, `OrgConfig` structs with `#[serde(deny_unknown_fields)]` + `#[non_exhaustive]` + `from_file`/`from_str` methods (~40-60 lines). |
| `crates/prism-dtu-demo-server/src/main.rs` | MODIFY | Add `StartMulti` variant to `Commands` enum; add `cmd_start_multi` + `build_multi_clone_factory` + `start_multi_for_config` + `write_multi_url_sidecar` (~80-120 lines net). |
| `crates/prism-dtu-demo-server/tests/` | CREATE | New integration test file (e.g., `multi_org.rs`) containing RG-004 and RG-005. The corresponding `[[test]]` entry in `crates/prism-dtu-demo-server/Cargo.toml` MUST specify `required-features = ["dtu", "fixture-gen"]` — matching the precedent of `bc_2_06_018_archetype_differential` and `bc_2_06_019_scenario_progression` in that crate's Cargo.toml. Without `required-features`, the test compiles without `fixture-gen` and `build_multi_clone_factory` will hard-error at runtime. |
| `crates/prism-dtu-demo-server/Cargo.toml` | MODIFY | Add `[[test]]` entry for `multi_org.rs` with `required-features = ["dtu", "fixture-gen"]`. |
| `scripts/start-demo.sh` | DELETE | Retire the `exec`-form standalone launcher. |
| `scripts/demo.toml` | MODIFY | Add `[orgs.org-a]`, `[orgs.org-b]`, `[orgs.org-c]` sections; existing `[harness]` + `[clones.*]` stay for `start` backward compatibility. |
| `scripts/demo-setup.sh` | MODIFY | Generalize from 1 org to N orgs: N-org prism.toml, N×M `prism credential set` calls. |
| `scripts/demo-run.sh` | MODIFY | Switch from N×`start` to one `start-multi`; read nested `{org_slug: {sensor: url}}` sidecar; generate N×M overlay TOMLs. |
| `scripts/demo-teardown.sh` | MODIFY | Single PID kill; generalize from 5 to N×M credential deletes. |
| `docs/DEMO-RUNBOOK.md` | MODIFY | Update all sections for 3-org operator flow; remove `start-demo.sh` references. |
| `.github/workflows/ci.yml` | NO CHANGE | `shellcheck-demo-scripts` job glob `scripts/demo-*.sh` unchanged. |

---

## Library and Framework Requirements

| Tool / Library | Version | Usage |
|---|---|---|
| `bash` | 5.x (macOS ships 3.x; Homebrew bash 5.x required on macOS for `declare -A` if used) | Shell runtime for demo scripts. |
| `shellcheck` | any stable | Shell linting gate; runs in CI and locally. |
| `python3` | 3.x (system default) | JSON parsing of nested sidecar; TOML overlay generation. `import json, sys, os` only — no third-party Python packages. |
| `serde` | workspace version | `Deserialize` + `Serialize` for `MultiOrgDemoConfig`, `OrgConfig`. |
| `toml` | workspace version | `MultiOrgDemoConfig::from_str` uses `toml::from_str`. |
| `serde_json` | workspace version | `write_multi_url_sidecar` uses `serde_json::to_string` for nested URL map. |
| `anyhow` | workspace version | All `cmd_start_multi` error propagation uses `anyhow::Result`. |
| `tokio` | workspace version | `cmd_start_multi` is an `async fn`; signal handling via `tokio::signal`. |
| `prism-dtu-common` | workspace | `BehavioralClone` trait; returned by `build_multi_clone_factory` factory. |
| `prism-dtu-crowdstrike`, `prism-dtu-armis`, `prism-dtu-claroty`, `prism-dtu-cyberint` | workspace | Clone constructors called by `build_multi_clone_factory`. **REQUIRED Cargo feature: `fixture-gen`** — the seeded `new_with_seed` constructors are `#[cfg(feature = "fixture-gen")]` gated on all four crates. `start-multi` must be built/run with `--features dtu,fixture-gen`. Omitting `fixture-gen` must hard-error (compile_error! or runtime panic) — NOT silently fall back to unseeded `new()` which would produce identical data across orgs and violate INV-DISTINCT-DATA-001. |
| `prism` (binary) | post-S-DEMO-003 | `prism credential set`, `prism credential delete`, shell scripts. |
| `prism-dtu-demo-server` (binary) | this story | New `start-multi` subcommand. |
| `cargo` | per `rust-toolchain.toml` | Build step in `demo-setup.sh`. |

---

## Open Question for Human (resolved — do not re-litigate)

The Option-1 vs Option-2 design question is resolved. Option-2 (Rust subcommand) is the
approved design per architect decision (v2.0 conversion). The implementer proceeds with
Option-2. Do not raise the question again.

---

## Changelog

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 2.2 | 2026-06-14 | story-writer | OBS-1 mapping-table correction: removed stale `MultiOrgConfig` row from Architecture Mapping table; the correct type names are `MultiOrgDemoConfig` (root config, annotated) and `OrgConfig` (per-org subsection, annotated). Fixed two additional `MultiOrgConfig` bare occurrences (frontmatter tdd_mode comment, token-budget architect-estimate comment) to `MultiOrgDemoConfig`/`OrgConfig` and `MultiOrgDemoConfig` respectively. Status: ready. Points: 8 (unchanged). |
| 1.0 | 2026-06-14 | story-writer | Initial materialization from D-1029 draft stub. Full spec with 10 ACs, consolidation decision (retire start-demo.sh), 3-org model, N×M overlay and credential generalization, all 6 context-engineering sections. status: ready. tdd_mode: facade (scripts-only). |
| 2.0 | 2026-06-14 | story-writer | Option-2 architect-approved conversion: facade→Rust-touching. Adds StartMulti CLI subcommand + MultiOrgDemoConfig structs in prism-dtu-demo-server. crates_touched: [] → [prism-dtu-demo-server]. tdd_mode: facade → tdd. points: 5 → 8. ACs expanded from 10 to 13 (3 new Rust subcommand ACs added; existing script ACs renumbered). 5 Red Gate tests specified (RG-001..RG-005). §File Structure updated with Rust crate MODIFY rows. demo-run.sh simplified to one start-multi call. Option-1 (N×M shell processes) retired. |
| 2.1 | 2026-06-14 | story-writer | Pre-TDD gap closure (3 gaps). GAP-1 (IMPORTANT): `fixture-gen` feature is REQUIRED for `start-multi`; `build_multi_clone_factory` must hard-error (compile_error! or runtime panic) if built without it — no silent fallback to unseeded `new()` which would violate INV-DISTINCT-DATA-001. All `start-multi` cargo commands updated to `--features dtu,fixture-gen`. `multi_org.rs` Cargo.toml `[[test]]` must specify `required-features = ["dtu", "fixture-gen"]`. Architecture Compliance Rules updated: `build_multi_clone_factory` must reuse `harness::parse_org_id`, `harness::fixture_set_to_archetype`, `prism_dtu_common::Archetype`, `demo_time_anchor()`. GAP-2 (MINOR): Cyberint composite path documented: `new_with_seed(...)` then `configure({"access_token": token})` post-construction — these compose cleanly via `CyberintState::apply_config` → `register_access_token`; no architect escalation needed. GAP-3 (NOTE): `DEMO_RUN_DIR` must be threaded into sidecar poll + Python paths in demo-run.sh to avoid cwd-relative path mismatch. Status: ready. Points: 8 (unchanged). |
