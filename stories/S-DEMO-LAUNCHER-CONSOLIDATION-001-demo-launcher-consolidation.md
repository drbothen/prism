---
document_type: story
story_id: S-DEMO-LAUNCHER-CONSOLIDATION-001
title: "scripts: Demo Launcher Consolidation — generalize demo-setup/demo-run/demo-teardown to N orgs; reconcile scripts/start-demo.sh vs demo-run.sh overlap"
wave: 5
epic_id: E-DEMO
priority: P3
status: ready
version: "1.0"
level: "L4"
producer: story-writer
timestamp: "2026-06-14T00:00:00Z"
tdd_mode: facade
# tdd_mode: facade justification —
#   This story produces ONLY shell scripts (demo-setup.sh, demo-run.sh, demo-teardown.sh,
#   demo.toml) and documentation (DEMO-RUNBOOK.md). There is no Rust source under TDD
#   discipline; all behavior is exercised by running the scripts themselves. The combined
#   scaffold+impl delivery model is appropriate for shell scripts, config files, and runbooks.
#   Mutation testing at wave gate replaces Red Gate density check as quality gate (BC-8.30.001
#   facade mode). No todo!() stubs are applicable.
subsystems: [SS-06, SS-22]
# Subsystem anchor justifications:
#   SS-06 (Client Configuration) owns the demo prism.toml + per-org overlay TOML generation;
#     the updated scripts create config directory structures that prism-bin reads at startup, with
#     N-org and N-credential entries instead of the fixed single-org model from S-DEMO-003.
#     Per ARCH-INDEX Subsystem Registry SS-06.
#   SS-22 (Binary Entrypoint) owns the `prism` CLI invocation surface; the consolidated launcher
#     prints the correct `prism start` command with per-org env vars; the retired start-demo.sh
#     called `prism-dtu-demo-server start` directly in exec-form, which overlaps with demo-run.sh's
#     background-launch model. Per ARCH-INDEX Subsystem Registry SS-22.
crates_touched: []
# crates_touched: empty — this story is scripts/ + docs/ ONLY. No Rust crate changes.
# Zero crate-conflict risk with concurrent demo stories (S-DEMO-004, PIVOT-001/002/003).
target_module: scripts
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
                 # MultiInstanceHarness (N×M sockets, one per org × sensor), reads urls.json
                 # (or multi-instance equivalent), and maps socket addresses to per-org overlay TOMLs.
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
points: 5
# Points justification:
#   - Consolidation decision (retire start-demo.sh; update demo-run.sh to call prism-dtu-demo-server
#     start instead of exec-form; document rationale): 0.5 pts
#   - Generalize demo-setup.sh from 1 org to N orgs (prism.toml N-org generation, N×M credential
#     bootstrapping, N-org customers/ dir scaffold, shellcheck): 1.5 pts
#   - Generalize demo-run.sh from 1 org to N orgs (multi-org urls.json or multi-instance parsing,
#     N×M overlay generation, N×M env vars in printed prism start command): 1.5 pts
#   - Generalize demo-teardown.sh from 1 org to N orgs (N×M credential deletes, multi-org
#     OrgId extraction): 0.5 pts
#   - Update demo.toml for multi-org demo (6 sensor DTU clones: 4 operational + ThreatIntel +
#     NVD; set seeds per org model derived from S-DEMO-004 3-org spec): 0.5 pts
#   - Update shellcheck CI glob to include start-demo.sh (or remove it after retirement): 0.5 pts
#   - Update DEMO-RUNBOOK.md for multi-org operational model: 0.5 pts (documentation)
#   - No Rust changes; shell scripts are well-understood; testing via shellcheck + manual run: low complexity.
#   Total: 5 points (~1 day). Gene-transfusion not applicable; no algorithm to port.
estimated_days: 1
risk: LOW
# Risk justification: scripts/ only, no Rust changes, no cross-crate blast radius. The
# generalization is additive (N=1 case must still work after the change). The consolidation
# decision (retire start-demo.sh → delegate to demo-run.sh) is non-breaking because
# start-demo.sh has no callers in CI. The prism.toml N-org pattern is already validated by
# the S-DEMO-004 test harness (same schema). Shellcheck gate prevents shell syntax regressions.
acceptance_criteria_count: 10
red_gate_tests: 0
# red_gate_tests: 0 — facade mode story; no Rust test stubs. Shell script correctness
# validated by shellcheck (CI gate) + manual demo dry-run (AC-008/AC-009 local verification).
estimated_passes: "1-2 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "N=1 backward compatibility: the generalized scripts must still work when ORG_CONFIG
    contains exactly one org entry (demo-org). This is the S-DEMO-003 baseline case and must
    not regress. Implementer must verify with a single-org dry-run after the multi-org path
    is implemented."
  - "shellcheck gate expansion: after retiring start-demo.sh OR after updating the shellcheck
    CI glob, run shellcheck on ALL files in the new glob scope before committing. The existing
    ci.yml shellcheck-demo-scripts job runs 'shellcheck scripts/demo-*.sh'; if start-demo.sh
    is retired (removed), no CI change is needed. If start-demo.sh is repurposed as a thin
    delegate, it must be added to the glob (rename to demo-start.sh) OR the glob must be
    widened to 'shellcheck scripts/*.sh'. Do NOT leave a shell script un-shellchecked."
  - "urls.json multi-org structure: the current demo-run.sh reads a flat urls.json
    ({sensor: url}). The multi-org multi-instance launch may produce a nested structure
    ({org_slug: {sensor: url}} or an array). Read the actual output format from
    prism-dtu-demo-server when launched with a multi-org config BEFORE scripting the overlay
    generation loop. Do not assume the S-DEMO-003 flat format extends unchanged."
  - "Credential bootstrap N×M: demo-setup.sh currently sets 5 credentials for 1 org.
    The multi-org case sets N×M credentials. The prism credential set subcommand takes
    --org-slug per-call; the script must loop over orgs × sensors. Idempotency is preserved
    (credential writes overwrite). AD-017 compliance (stdin, not argv) is unchanged."
  - "demo.toml multi-org seed alignment: the seeds used in demo.toml clones must match the
    seeds used in the S-DEMO-004 test harness (seed=100 for org-a, seed=200 for org-b/org-c
    or as specified in the 3-org model) so the demo scripts and the CI test tell the same story.
    Read S-DEMO-004 risk_mitigations block for the authoritative seed values."
inputs:
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
  - "crates/prism-dtu-demo-server/src/harness.rs"
  - "crates/prism-dtu-harness/src/multi_instance.rs"
  - "crates/prism-dtu-harness/src/overlay_wiring.rs"
input-hash: null
traces_to: []
cycle: "v1.0.0-brownfield"
phase: 3
---

# S-DEMO-LAUNCHER-CONSOLIDATION-001 — Demo Launcher Consolidation

**Story ID:** S-DEMO-LAUNCHER-CONSOLIDATION-001
**Status:** ready
**Version:** v1.0
**Wave:** 5
**Priority:** P3
**Points:** 5

---

## Origin

Registered as draft stub at D-1029 (2026-06-06) during S-DEMO-003 LOCAL adversary pass-2. The
`scripts/start-demo.sh` convenience launcher was identified as an OBS [process-gap]: it overlaps
with `demo-run.sh` as a demo launch entry-point but is not covered by the `scripts/demo-*.sh`
shellcheck CI glob (AC-008 and AC-014 in S-DEMO-003).

Scope expanded at T11 (multi-client-soc-demo-tasks.md) to include generalizing all three demo
scripts from the fixed single-org (`demo-org`) model delivered by S-DEMO-003 to support N orgs
with mixed sensor combos and per-org DTU clones, consistent with the 3-org model defined in
S-DEMO-004 and the demo scope in DEMO-SCOPE.md.

This story is scripts/ and docs/ ONLY. Zero Rust crate changes.

---

## Launcher Consolidation Decision

### The overlap

`scripts/start-demo.sh` was written as a standalone convenience launcher for
`prism-dtu-demo-server`. It builds the binary and then `exec`s into
`prism-dtu-demo-server start --config ...` as the final step — replacing the shell process.
This means it CANNOT be used to manage the demo server in the background, read its
`urls.json` sidecar, or perform follow-on steps (overlay generation, printing the prism
start command).

`scripts/demo-run.sh` (delivered by S-DEMO-003) is the complete daily launch script: it
starts the demo server IN THE BACKGROUND, polls for `urls.json`, generates per-org overlay
TOMLs, and prints the `prism start` command. This is the correct flow for the multi-client
demo.

### Decision: retire `scripts/start-demo.sh` and update `scripts/demo-run.sh`

**Rationale (four reasons):**

1. **`start-demo.sh` is architecturally incompatible with the demo flow.** Its `exec` form
   replaces the shell process; it cannot poll `urls.json` (which arrives asynchronously after
   the server binds its ports), cannot write per-org overlay TOMLs, and cannot print the
   `prism start` command. Any demo that needs those steps (i.e., every multi-org demo) must
   use `demo-run.sh`. Making `start-demo.sh` a thin delegate to `demo-run.sh` would be empty
   redirection — the user would just call `demo-run.sh` directly.

2. **`start-demo.sh` targets a different binary + different config file.** It defaults to
   `crates/prism-dtu-demo-server/configs/demo.toml` (6 DTU clones, hardcoded ports,
   development config); `demo-run.sh` uses `scripts/demo.toml` (4 operational sensors,
   ephemeral ports, production-equivalent demo config). Having two launch scripts pointing at
   two different TOML configs is a confusion surface for operators.

3. **`start-demo.sh` exports fake credentials as env vars in its own body.** S-DEMO-003
   established that `demo-setup.sh` (keyring bootstrap) + `demo-run.sh` (ephemeral env vars
   printed for operator to execute) is the correct credential flow. `start-demo.sh`'s own
   credential-export block is a parallel, non-canonical, and potentially confusing credential
   path.

4. **Shellcheck CI coverage.** After retirement, the shellcheck CI glob `scripts/demo-*.sh`
   continues to cover all remaining demo scripts (setup, run, teardown) with no changes to
   `ci.yml` needed.

**Retirement mechanics:** Delete `scripts/start-demo.sh` from the repository. The demo operator
runbook (`docs/DEMO-RUNBOOK.md`) already points operators to `demo-run.sh`; no external callers
in CI reference `start-demo.sh`. The binary `prism-dtu-demo-server` remains fully functional and
accessible via `demo-run.sh`.

**Product question surfaced (human must decide — do not decide unilaterally):** DEMO-RUNBOOK.md
currently points operators to `bash scripts/demo-run.sh` for the daily launch. After generalizing
to N orgs, the operator flow changes: the operator must provide an org configuration (org slugs,
sensor combos, seed assignments). The two UX options are:

- **Option A (config file):** `demo-run.sh` reads a `demo-orgs.toml` file (or extends the
  existing `demo.toml`) that lists orgs, their sensor combos, and seeds. The operator edits
  this file once; subsequent runs read it without flags.
- **Option B (CLI flags):** `demo-run.sh --orgs org-a:crowdstrike,armis --orgs org-b:claroty,cyberint`
  with sensor combos passed at invocation time. More flexible; harder to document.

**For this story's implementation, the implementer should use Option A (config file)** with the
3-org model hard-coded in `scripts/demo.toml` as the default. This aligns with the existing
pattern (`demo.toml` already drives the single-org demo) and defers operator-parameterization
to future work. If the human prefers Option B before merge, the story ACs accommodate that change
at review time.

---

## Narrative

As a demo operator (MSSP analyst running the multi-client SOC live demo), I want a single
consolidated set of demo scripts (`demo-setup.sh`, `demo-run.sh`, `demo-teardown.sh`) that
can bootstrap, launch, and tear down a multi-org (N-org) demo environment — with per-org DTU
clone instances, per-org credential bootstrapping, per-org sensor overlay generation, and
consistent shellcheck CI coverage — so that I can stand up the 3-org × mixed-sensor demo in
one command sequence without ambiguity about which launcher script to use.

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

The implementer should use the following 3-org model as the target for `scripts/demo.toml`
and the generalized scripts. This is the same model defined in S-DEMO-004.

### Org registration

```toml
# scripts/demo.toml — updated for 3-org model
[harness]
bind = "127.0.0.1"

# Org A: CrowdStrike + Armis
[orgs.org-a]
org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0000"   # UUID v7; must be used consistently
sensors = ["crowdstrike", "armis"]
seed = 100

# Org B: Claroty + Cyberint
[orgs.org-b]
org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0001"   # UUID v7
sensors = ["claroty", "cyberint"]
seed = 150

# Org C: all 4 operational sensors
[orgs.org-c]
org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0002"   # UUID v7
sensors = ["crowdstrike", "armis", "claroty", "cyberint"]
seed = 200
```

IMPORTANT: The demo.toml schema extension above is a PROPOSED format for this story.
The `prism-dtu-demo-server` parses `demo.toml` via its own config struct (not via
`PrismConfig`). The implementer MUST read `crates/prism-dtu-demo-server/src/` to understand
the actual config schema before writing the TOML. If `demo.toml` does not support an
`[orgs.*]` block natively, the implementer must:

1. Add the per-org table to the demo-server config struct (a non-Rust-crate-touching path is
   preferred; if demo-server Rust changes are required, the implementer must flag this as a
   scope expansion before proceeding), OR
2. Keep `demo.toml` for DTU clone configuration only (as today) and drive the N-org model
   from `scripts/demo-setup.sh` and `scripts/demo-run.sh` shell logic that reads a separate
   `scripts/demo-orgs.toml`.

**Scope clarification (implementer pre-flight check):** Read
`crates/prism-dtu-demo-server/configs/demo.toml` and `scripts/demo.toml` to determine the
current schema. The TOML in this story is illustrative, not authoritative. The authoritative
schema is the Rust struct in `crates/prism-dtu-demo-server/src/config.rs` (or equivalent).

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

After DTU server starts and urls.json is ready:

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

For 3 orgs with the sensor combos above, the credential set calls are:

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
passed as a CLI arg. The `initial_access_token` for each Cyberint DTU clone must match the
`api_key` set for that org (e.g., `demo-cyberint-api-key-org-b` for org-b's clone).

---

## Acceptance Criteria

### AC-001: `scripts/start-demo.sh` is retired (removed from the repository)

(traces to BC-2.06.001 precondition 1 — single canonical entry point for demo launch removes
ambiguity that would lead operators to use an incompatible launch path)

`scripts/start-demo.sh` is deleted. The file does not exist on the post-merge `develop` branch.
`docs/DEMO-RUNBOOK.md` does not reference `start-demo.sh`. The `ci.yml` shellcheck glob
`scripts/demo-*.sh` continues to pass with zero shellcheck errors or warnings (git removes the
file from the workspace; no glob change needed since `start-demo.sh` was never in `demo-*.sh`).

**Verification:** `git show HEAD -- scripts/start-demo.sh` returns "fatal: Path does not exist."
`shellcheck scripts/demo-*.sh` exits 0 with no output.

---

### AC-002: `scripts/demo-setup.sh` generates a multi-org `prism.toml` with N [[orgs]] entries

(traces to BC-2.06.001 postcondition 1 — generated prism.toml must be schema-valid and
accepted by prism-bin's TOML config loader at startup)

Given: `bash scripts/demo-setup.sh --config-dir <DIR>` completes on macOS or Linux.

Then:
- `<DIR>/prism.toml` exists and contains exactly 3 `[[orgs]]` entries (org-a, org-b, org-c)
  with distinct `org_id` UUIDs (v7) and `org_slug` values.
- `prism --config-dir <DIR> --dry-run` (or equivalent config validation mode) exits 0.
- `<DIR>/specs/` contains the 4 TYPE spec TOMLs (crowdstrike, armis, claroty, cyberint) and
  a `customers/` subdirectory with 3 org-slug subdirs (empty at setup time — overlays are
  written later by demo-run.sh).
- `<DIR>/plugins/` contains `crowdstrike-oauth2.prx` and `crowdstrike-oauth2.manifest.toml`
  (unchanged from S-DEMO-003; manifest `allowed_urls = ["api.crowdstrike.com", "127.0.0.1"]`).

---

### AC-003: `scripts/demo-setup.sh` bootstraps N×M credentials (one per org × sensor combo)

(traces to BC-2.06.001 postcondition 1 — keyring must be seeded with a credential for every
org × sensor pair that appears in the multi-org config; missing credentials cause boot failure
at step 9A credential resolution)

Given: `bash scripts/demo-setup.sh --config-dir <DIR>` completes.

Then:
- For each (org_slug, sensor, name) combination in the N×M credential table above, a keyring
  entry was written via `printf '%s\n' <dummy_value> | prism --config-dir <DIR> credential set
  --org-slug <org_slug> --sensor <sensor> --name <name>`.
- AD-017: dummy values are piped via stdin only; no credential value appears in the process
  command line.
- If keyring write fails, the script prints the Tier-2 env-var fallback name
  (PRISM_CLIENTS_<ORG_UPPER>_SENSORS_<SENSOR_UPPER>_<NAME_UPPER>) to stderr and continues
  (idempotent behavior unchanged from S-DEMO-003).
- The Cyberint `api_key` dummy value for each org matches the `initial_access_token` configured
  in the corresponding Cyberint DTU clone in `scripts/demo.toml`.

---

### AC-004: `scripts/demo-run.sh` starts DTU server and generates N×M overlay TOMLs

(traces to BC-2.06.012 postcondition 2 — per-org overlay TOMLs must exist under
`customers/<org_slug>/` before prism-bin starts; BC-2.06.013 invariant — overlay files contain
only scalar fields)

Given: `demo-setup.sh` has completed; `bash scripts/demo-run.sh --config-dir <DIR>` is run.

Then:
- `prism-dtu-demo-server` starts in background with a multi-org-aware config (N DTU clone
  instances per sensor, or a multi-instance harness config).
- Within 30s, the DTU server writes a URLs sidecar file.
- `demo-run.sh` reads the URLs sidecar and writes per-org overlay TOMLs for every
  (org_slug, sensor_id) combination that org has a registered sensor:
  - `<DIR>/specs/customers/org-a/crowdstrike.sensor.toml`
  - `<DIR>/specs/customers/org-a/armis.sensor.toml`
  - `<DIR>/specs/customers/org-b/claroty.sensor.toml`
  - `<DIR>/specs/customers/org-b/cyberint.sensor.toml`
  - `<DIR>/specs/customers/org-c/crowdstrike.sensor.toml`
  - `<DIR>/specs/customers/org-c/armis.sensor.toml`
  - `<DIR>/specs/customers/org-c/claroty.sensor.toml`
  - `<DIR>/specs/customers/org-c/cyberint.sensor.toml`
- Each overlay contains exactly: `extends`, `instance_id`, `base_url` (three scalar fields).
  No `[[tables]]`, no `auth_type`, no additional keys (BC-2.06.013).
- Each org's CrowdStrike overlay `base_url` points to a DIFFERENT socket address (port) from
  org-c's CrowdStrike overlay (distinct per-org DTU clone instances, BC-2.06.017).

---

### AC-005: Each org's DTU clone uses a distinct socket address (per-org port isolation)

(traces to BC-2.06.017 postcondition 3 — per-DTU-instance multi-address binding ensures no
two orgs share a DTU socket for the same sensor type)

Given: `demo-run.sh` has started the DTU server with the multi-org config.

Then:
- The URLs sidecar contains distinct socket addresses for each (org_slug, sensor_id) pair.
- Org-A's CrowdStrike clone port ≠ Org-C's CrowdStrike clone port (both have CrowdStrike;
  they must not share a socket).
- Each generated overlay `base_url` port is unique across all orgs for the same sensor type.

---

### AC-006: `scripts/demo-run.sh` prints the prism start command with all N×M env vars

(traces to BC-2.06.014 postcondition — `${env.VAR}` placeholders in TYPE specs must resolve
at boot step 4a before per-org overlays replace base_url at step 4c; all 4 sensor env vars
must be present)

Given: `demo-run.sh` has completed overlay generation.

Then: `demo-run.sh` prints a `prism start` command block that includes:
```
CROWDSTRIKE_BASE_URL=http://127.0.0.1 \
ARMIS_INSTANCE_URL=http://127.0.0.1 \
CLAROTY_INSTANCE_URL=http://127.0.0.1 \
CYBERINT_ENVIRONMENT=demo \
${PRISM_BIN} --config-dir ${DEMO_CONFIG_DIR} start
```
These 4 env vars are the same TYPE-spec env vars that S-DEMO-003 established as required
(step-4a boot gate). The per-org DTU port is handled by the overlay TOMLs (step-4c), not
by the env vars. The env vars provide placeholder values to satisfy the `${env.*}` tokens in
the TYPE spec before any query is dispatched.

---

### AC-007: `scripts/demo-teardown.sh` deletes N×M keyring entries and removes config dir

(traces to BC-2.06.001 postcondition — teardown must undo all state written by demo-setup.sh;
no orphaned keyring entries after teardown)

Given: A multi-org demo environment exists at `<DIR>`.

Then: `bash scripts/demo-teardown.sh --config-dir <DIR>` runs to completion:
1. Kills the DTU server (via PID file).
2. Deletes every keyring entry listed in the N×M credential table (all 10 entries for the
   3-org configuration) via `prism credential delete` (OrgId-keyed, per ADR-034 §D3).
   Keyring deletes run BEFORE `rm -rf` (F-P10-HIGH-001 ordering invariant from S-DEMO-003).
3. Removes `<DIR>` with `rm -rf`.
4. Exits 0.
If keyring delete fails (e.g., entry already absent), the failure is logged to stderr and
teardown continues (idempotent).

---

### AC-008: All shell scripts pass `shellcheck` with zero errors or warnings

(traces to BC-2.06.001 invariant — shell scripting quality gate prevents deployment-blocking
shell syntax errors in the field)

Given: `demo-setup.sh`, `demo-run.sh`, `demo-teardown.sh` are present in `scripts/`;
`start-demo.sh` has been deleted.

When: `shellcheck scripts/demo-*.sh` is run.

Then: Exit 0, zero errors, zero warnings. The existing `shellcheck-demo-scripts` CI job in
`.github/workflows/ci.yml` continues to pass. No `ci.yml` changes are required (the glob
`scripts/demo-*.sh` already covers the 3 scripts; retiring `start-demo.sh` removes the
uncovered script).

---

### AC-009: `scripts/demo-setup.sh` is idempotent (safe to re-run)

(traces to BC-2.06.001 invariant — idempotency was an S-DEMO-003 requirement; must not
regress in the generalized version)

Given: `demo-setup.sh` has already been run once for the 3-org config.

When: `bash scripts/demo-setup.sh --config-dir <DIR>` is run again.

Then: The script completes without error. Directory creation uses `mkdir -p`. TOML files are
overwritten. Keyring writes overwrite existing entries. The post-run state is identical to
after the first run.

---

### AC-010: `docs/DEMO-RUNBOOK.md` is updated for the multi-org operator flow

(traces to BC-2.06.001 — the runbook is the authoritative operator reference; it must match
the generalized scripts)

Given: The 3 scripts have been updated.

Then: `docs/DEMO-RUNBOOK.md` reflects the multi-org workflow:
- §Setup section: `demo-setup.sh` bootstraps 3 orgs; the N×M credential set calls are
  documented.
- §Daily Demo Run section: `demo-run.sh` starts a multi-org DTU fleet; N×M overlay files are
  generated; the operator is given the `prism start` command with 4 env vars.
- §Teardown section: `demo-teardown.sh` deletes N×M credentials; ordering (delete before
  rm -rf) is documented.
- §Troubleshooting section: keyring fallback env vars use the org-scoped Tier-2 format
  (PRISM_CLIENTS_<ORG_UPPER>_SENSORS_...) for all N×M combinations.
- No references to `start-demo.sh` remain.

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `scripts/demo-setup.sh` | scripts/ | Effectful (shell: cargo build, mkdir, cp, prism CLI) |
| `scripts/demo-run.sh` | scripts/ | Effectful (shell: subprocess DTU server, file I/O, overlay generation) |
| `scripts/demo-teardown.sh` | scripts/ | Effectful (shell: process kill, prism CLI, rm -rf) |
| `scripts/demo.toml` | scripts/ | Pure (config file; read by DTU server) |
| `docs/DEMO-RUNBOOK.md` | docs/ | Pure (documentation) |

Architecture section files referenced:
- `architecture/module-decomposition.md` (SS-06, SS-22 subsystem responsibilities)
- `architecture/dependency-graph.md` (scripts/ has no Rust crate dependencies)
- `architecture/decisions/ADR-029-multi-tenant-sensor-endpoint-overrides.md` (overlay format)
- `architecture/decisions/ADR-034-tier3-keyring-resolution-org-id-threading.md`
  (OrgId-keyed credential namespace; teardown delete-by-org)

---

## Architecture Compliance Rules

| Rule | Rationale |
|------|-----------|
| `demo-setup.sh` MUST NOT write per-org `base_url` overlay TOMLs | DTU ports are ephemeral (only known post-launch); overlay generation belongs exclusively in `demo-run.sh` after `urls.json` is parsed. Inherited from S-DEMO-003 AC-001 + Architecture Compliance. |
| `demo-run.sh` MUST write N×M overlay TOMLs BEFORE printing the `prism start` command | prism spec loader reads overlays at boot step 4c; missing overlay = wrong base_url = E-SPEC-024 or silent wrong endpoint. |
| `demo-run.sh` MUST include all 4 TYPE-spec env vars in the printed `prism start` command | Step-4a `env_resolver.rs` resolves `${env.*}` tokens; missing env var = E-SPEC-024 boot abort before step-4c overlays apply (S-DEMO-003 F-HIGH-301 precedent). |
| `demo-teardown.sh` MUST run keyring deletes BEFORE `rm -rf` of the config dir | `prism credential delete` reads `prism.toml` for OrgId resolution (ADR-034 §D3); if config dir is removed first, `prism.toml` is unavailable and all deletes fail silently (S-DEMO-003 F-P10-HIGH-001). |
| Each overlay TOML MUST contain ONLY scalar fields: `extends`, `instance_id`, `base_url` | BC-2.06.013 scalar-only overlay enforcement; tables, `auth_type`, and `version` must not appear. |
| `demo-setup.sh` MUST NOT write overlays that hardcode port numbers | Ports are ephemeral; hardcoded ports break across machine reboots and concurrent demo runs. |
| `start-demo.sh` MUST be deleted (not converted to a delegate) | `exec`-form launcher is architecturally incompatible with the background-launch + polling model; a delegate wrapper would be dead weight. |
| AD-017: credential values MUST be piped via stdin, NEVER via CLI argv | Applies to all N×M `prism credential set` calls in `demo-setup.sh`. |
| Cyberint DTU `initial_access_token` MUST match the `api_key` credential for that org | StaticCookieAuthProvider injects `Cookie: access_token=<keyring-value>` on each request; if the keyring value and the DTU allowlist seed differ, all Cyberint queries return 401 (S-DEMO-003 precedent). |
| All shell scripts MUST use `#!/usr/bin/env bash` shebang and `set -euo pipefail` | shellcheck portability requirement; error-exit on any unset variable or pipeline failure. |

---

## Forbidden Dependencies

| Forbidden | Reason |
|-----------|--------|
| Hardcoded port numbers in overlay TOML generation | Ephemeral ports are assigned by the OS; hardcoded ports break across restarts and concurrent runs. Only `urls.json` (or equivalent sidecar) carries the authoritative port numbers after DTU server start. |
| `start-demo.sh` as a caller or delegate in any script | The file is being retired; no other script should depend on it. |
| Rust crate changes | This story is scripts/ + docs/ only. If the implementer discovers that generalizing the DTU server requires a Rust change, the implementer must flag this as a scope expansion and request human approval before proceeding. |
| Direct port read from DTU process listing or `/proc/<pid>/net` | DTU server writes its own sidecar file; reading ports from OS state is fragile and platform-specific. |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Cyberint `api_key` dummy value does not match DTU `initial_access_token` for that org | Cyberint DTU clone returns 401 on all requests for that org. Demo fails silently (wrong data). The script must enforce that the dummy value used in `credential set` matches the `initial_access_token` configured in `demo.toml` for that org's Cyberint clone. |
| EC-002 | DTU server does not start within 30s (port conflict, binary not built) | `demo-run.sh` prints an error with the DTU server log path and exits 1 (same behavior as S-DEMO-003; timeout unchanged at 30s). |
| EC-003 | `crowdstrike-oauth2.prx` not found | `demo-setup.sh` exits 1 with actionable message: "Run: cargo build -p prism-spec-engine --features wasm-plugins". Same as S-DEMO-003 EC-003. |
| EC-004 | `demo-teardown.sh` is run when config dir does not exist | Script prints "already removed" and exits 0 (idempotent). |
| EC-005 | `demo-teardown.sh` is run when DTU server is not running (no PID file) | Script prints "server may not be running" and skips the kill step; continues to credential delete + rm -rf. |
| EC-006 | Keyring backend unavailable during `demo-setup.sh` credential bootstrap | Per-credential failure: prints Tier-2 env-var fallback name to stderr, continues to next credential. Setup does not abort. Operator is responsible for setting fallback env vars if keyring is unavailable. |
| EC-007 | `demo-run.sh` called before `demo-setup.sh` (no `prism.toml` present) | `demo-run.sh` checks for `prism.toml` existence before starting DTU; exits 1 with "Run: bash scripts/demo-setup.sh". |
| EC-008 | Multi-org config omits org-slug directory under `customers/` | `demo-run.sh` creates the directory with `mkdir -p <DIR>/specs/customers/<org_slug>` before writing overlays. |
| EC-009 | DTU multi-org sidecar format differs from single-org `urls.json` flat format | Implementer must read the actual sidecar format produced by the multi-org DTU server (may be nested: `{org_slug: {sensor: url}}`). Overlay generation Python script must match the actual format. AC-004 verifies overlay content; if sidecar format is wrong, overlays will not be generated correctly. |

---

## Token Budget Estimate

| Context component | Lines estimate | Tokens (approx) |
|---|---|---|
| This story spec | 450 | 5,400 |
| S-DEMO-003 (predecessor, full body) | 700 | 8,400 |
| S-DEMO-004 (3-org model reference) | 300 | 3,600 |
| BC files (5 BCs × ~100 lines each) | 500 | 6,000 |
| Existing scripts (4 scripts × ~150 lines) | 600 | 7,200 |
| DEMO-RUNBOOK.md (existing) | 250 | 3,000 |
| ci.yml (shellcheck section) | 100 | 1,200 |
| `crates/prism-dtu-demo-server/src/` (config struct) | 200 | 2,400 |
| ADR-029, ADR-034 (overlay + credential BCs) | 200 | 2,400 |
| Implementer working memory / tool outputs | — | 5,000 |
| **Total** | | **~44,600** |

Estimate is ~22% of a 200k-token context window. Within the 20-30% per-story limit.

---

## Tasks

All tasks are facade-mode (combined scaffold + impl delivery). No `todo!()` stubs.

### Phase 1: Pre-flight (read before touching any file)

- [ ] Read `scripts/start-demo.sh` in full; confirm the `exec`-form launch model and the
  credential-export pattern (confirms retirement decision rationale).
- [ ] Read `scripts/demo-run.sh` in full; understand the single-org overlay generation loop
  (the N-org generalization extends this loop).
- [ ] Read `scripts/demo-setup.sh` in full; understand the single-org prism.toml generation
  and the 5 credential set calls.
- [ ] Read `scripts/demo-teardown.sh` in full; understand the credential delete loop and
  the delete-before-rm-rf ordering invariant.
- [ ] Read `scripts/demo.toml`; understand the current single-instance DTU config schema.
- [ ] Read `crates/prism-dtu-demo-server/src/config.rs` (or equivalent); understand what
  TOML fields the demo-server actually supports. Determine if a multi-org section is already
  present or needs to be added (scope expansion decision point).
- [ ] Read `docs/DEMO-RUNBOOK.md`; note all sections that must be updated.

### Phase 2: Decision — DTU multi-org launch mechanism

- [ ] Determine how the DTU demo server supports multiple per-org clone instances:
  - If `prism-dtu-demo-server` supports a multi-instance config section in its TOML (from
    S-DEMO-MULTI-TENANT-DTU-001 BC-2.06.017), use that.
  - If not, determine whether demo-run.sh must launch N×M separate DTU server processes
    (one per org × sensor) or if there is a harness binary that handles multi-instance.
  - Document the decision inline in `demo-run.sh` header comment.
- [ ] Determine the actual URLs sidecar format for multi-org launch (flat vs nested JSON).
  This controls the overlay generation script in demo-run.sh.

### Phase 3: Retire `scripts/start-demo.sh`

- [ ] Delete `scripts/start-demo.sh` from the repository.
- [ ] Search `docs/` and `scripts/` for any references to `start-demo.sh`; remove or replace
  with `demo-run.sh`.
- [ ] Run `shellcheck scripts/demo-*.sh`; verify exit 0 (retiring the file should not affect
  the existing glob since `start-demo.sh` was never in `demo-*.sh`).

### Phase 4: Generalize `scripts/demo.toml`

- [ ] Update `scripts/demo.toml` to configure the multi-org DTU fleet (3 orgs, 8 clone
  instances total: org-a×{cs,armis}, org-b×{claroty,cyberint}, org-c×{cs,armis,claroty,cyberint}).
  Use distinct seeds per org (100, 150, 200) to satisfy BC-2.06.018 + S-DEMO-004 INV-DISTINCT-DATA-001.
  If the demo-server Rust config does not support an org-keyed section, use the closest
  available mechanism (e.g., named clone groups or separate bind configs).

### Phase 5: Generalize `scripts/demo-setup.sh`

- [ ] Replace the hardcoded `DEMO_ORG_ID` + `DEMO_ORG_SLUG` with N-org arrays (or a loop over
  `demo.toml` org definitions read via `python3` or TOML parsing).
- [ ] Update prism.toml generation to emit N `[[orgs]]` entries.
- [ ] Update `customers/` scaffolding to create N org-slug subdirectories.
- [ ] Replace the 5 hardcoded `set_cred` calls with an N×M loop over (org_slug, sensor, name,
  dummy_value) tuples.
- [ ] Ensure Cyberint `api_key` dummy values match `initial_access_token` in `demo.toml`
  per-org clone config (EC-001).
- [ ] Run `shellcheck scripts/demo-setup.sh`; fix any warnings.

### Phase 6: Generalize `scripts/demo-run.sh`

- [ ] Remove any hardcoded `DEMO_ORG_SLUG="demo-org"` references; replace with N-org loop.
- [ ] Update the overlay generation Python block to:
  - Read the (possibly nested) URLs sidecar.
  - Iterate over all (org_slug, sensor_id, url) triples.
  - Write `customers/<org_slug>/<sensor_id>.sensor.toml` with correct scalar-only format.
- [ ] Update the `prism start` command print block to include all 4 TYPE-spec env vars
  (unchanged from S-DEMO-003; these are not per-org).
- [ ] Run `shellcheck scripts/demo-run.sh`; fix any warnings.

### Phase 7: Generalize `scripts/demo-teardown.sh`

- [ ] Replace the 5 hardcoded `delete_keyring_entry` calls with an N×M loop over
  (org_slug, sensor, name) tuples matching the N×M set calls in demo-setup.sh.
- [ ] Update OrgId extraction to support N orgs (extract one OrgId per org_slug from
  prism.toml; delete credentials per-org keyed by that OrgId).
- [ ] Run `shellcheck scripts/demo-teardown.sh`; fix any warnings.

### Phase 8: Update `docs/DEMO-RUNBOOK.md`

- [ ] Update §Setup to reflect 3-org environment, N×M credential count.
- [ ] Update §Daily Demo Run to reflect multi-org DTU fleet and N×M overlay generation.
- [ ] Update §Teardown for N×M credential deletes.
- [ ] Update §Troubleshooting to enumerate all 10 Tier-2 env-var fallback names.
- [ ] Remove all references to `start-demo.sh`.

### Phase 9: CI verification

- [ ] Confirm `shellcheck scripts/demo-*.sh` exits 0 with zero output locally.
- [ ] Confirm the existing `shellcheck-demo-scripts` CI job in `.github/workflows/ci.yml`
  does NOT need modification (glob `scripts/demo-*.sh` is unchanged; no new scripts added
  that are outside the glob).
- [ ] Run `bash scripts/demo-setup.sh --config-dir /tmp/prism-demo-test` on a local machine;
  verify 3-org prism.toml is generated.
- [ ] Run `bash scripts/demo-teardown.sh --config-dir /tmp/prism-demo-test`; verify N×M
  credential deletes and directory removal.

---

## Previous Story Intelligence

### S-DEMO-003 (predecessor — merged PR #176 develop@a42e3eaf)

1. **Overlay generation belongs in `demo-run.sh`, NOT `demo-setup.sh`:** DTU ports are
   ephemeral (assigned post-launch). Writing overlay TOMLs with a hardcoded port in setup
   is incorrect. This was the F-HIGH-201 finding in S-DEMO-003 pass-2; the fix required
   a full redesign of the overlay flow. Do not repeat this mistake.

2. **Four TYPE-spec env vars are REQUIRED in the printed `prism start` command:** Without
   CROWDSTRIKE_BASE_URL / ARMIS_INSTANCE_URL / CLAROTY_INSTANCE_URL / CYBERINT_ENVIRONMENT,
   boot step 4a fires E-SPEC-024 and aborts before step-4c overlays apply. This was
   F-HIGH-301 in S-DEMO-003 pass-3. These four env vars are not per-org — they provide
   placeholder values for the TYPE specs; per-org base_urls are overridden by overlays.

3. **Keyring deletes must run BEFORE `rm -rf` of config dir:** `prism credential delete`
   reads `prism.toml` for OrgId-keyed namespace lookup (ADR-034 §D3). Removing the config
   dir first silently orphans keyring entries. This was F-P10-HIGH-001 in S-DEMO-003
   pass-10.

4. **CrowdStrike SEC-003 manifest `allowed_urls`:** The `crowdstrike-oauth2.manifest.toml`
   in `$DEMO_PLUGINS_DIR` must include `"127.0.0.1"` alongside `"api.crowdstrike.com"`.
   This is a PLUGIN-level SEC-003 host-function gate, not per-org egress. One manifest
   file covers all orgs.

5. **AD-017 stdin-only credential values:** Every `prism credential set` call pipes the
   dummy value via `printf '%s\n' "${value}" | prism credential set ...`. Do not add the
   credential value as a `--value` CLI arg.

6. **shellcheck CI glob is `scripts/demo-*.sh`:** AC-014 in S-DEMO-003 enforces this.
   Retiring `start-demo.sh` (which was NOT in this glob) cleans up the uncovered script
   without requiring any CI change.

### S-DEMO-004 (3-org model reference — ready v1.7)

1. **3-org sensor assignments:** Org-A = CrowdStrike + Armis; Org-B = Claroty + Cyberint;
   Org-C = all 4 sensors. These are the canonical assignments for the demo; scripts must
   match this model.

2. **Per-org seeds for data distinctness:** The test harness uses distinct seeds per org
   (seed=100 for org-a's CrowdStrike, seed=200 for org-c's CrowdStrike) to satisfy
   INV-DISTINCT-DATA-001 (Org A data ≠ Org C data). Demo scripts must use the same
   seed assignments in `demo.toml` clone configs so the operator demo and the CI test
   are in sync.

3. **`write_overlay_temp_dir` call pattern:** The S-DEMO-004 test harness uses
   `write_overlay_temp_dir(&harness, tempdir.path())` from `prism-dtu-harness`. The demo
   scripts implement the equivalent in shell+python. The overlay format is identical:
   `extends`, `instance_id`, `base_url` (three scalar fields; BC-2.06.013).

---

## File Structure Requirements

| File | Action | Description |
|------|--------|-------------|
| `scripts/start-demo.sh` | DELETE | Retire the `exec`-form standalone launcher (see consolidation decision above). |
| `scripts/demo-setup.sh` | MODIFY | Generalize from 1 org to N orgs: N-org prism.toml, N-org `customers/` scaffold, N×M `prism credential set` calls. |
| `scripts/demo-run.sh` | MODIFY | Generalize overlay generation from 1 org to N orgs: N×M overlay TOML writes (one per org × sensor); update Python block to handle multi-org URLs sidecar. |
| `scripts/demo-teardown.sh` | MODIFY | Generalize from 5 credential deletes to N×M credential deletes; N OrgId extractions from prism.toml. |
| `scripts/demo.toml` | MODIFY | Update DTU clone config for 3-org × mixed-sensor fleet (8 clone instances); add per-org seeds (100, 150, 200). |
| `docs/DEMO-RUNBOOK.md` | MODIFY | Update all sections for 3-org operator flow; remove `start-demo.sh` references. |
| `.github/workflows/ci.yml` | NO CHANGE | `shellcheck-demo-scripts` job glob `scripts/demo-*.sh` is unchanged; retiring `start-demo.sh` (not in glob) requires no CI update. |

---

## Library and Framework Requirements

| Tool / Library | Version | Usage |
|---|---|---|
| `bash` | 5.x (macOS ships 3.x; Homebrew bash 5.x required on macOS for `declare -A` assoc arrays if used) | Shell runtime for all demo scripts. If `declare -A` is needed for N-org mapping, require bash 5.x explicitly in script header. Alternative: use positional arrays to avoid the macOS bash 3.x limitation. |
| `shellcheck` | any stable (per S-DEMO-003 AC-014) | Shell linting gate; runs in CI and locally. |
| `python3` | 3.x (system default on macOS/Linux) | JSON parsing of URLs sidecar; TOML overlay generation. `import json, sys, os` only — no third-party Python packages. |
| `prism` (binary) | post-S-DEMO-003 (develop HEAD) | `prism credential set`, `prism credential delete`, `prism --dry-run` (if available). |
| `prism-dtu-demo-server` (binary) | post-S-DEMO-003 | Multi-org DTU fleet launch. The binary must support multi-org/multi-instance config. |
| `cargo` | per `rust-toolchain.toml` | Build step in `demo-setup.sh`. |
| `just` | not required directly | Demo scripts do not call `just`; operators may use it for convenience. |

**Bash 3.x vs 5.x concern:** macOS ships `/bin/bash` at version 3.2 which does not support
`declare -A` associative arrays. S-DEMO-003 scripts avoid this by using positional variables.
If the N-org generalization requires an associative array (org_slug → org_id mapping), either
use `#!/usr/bin/env bash` with the Homebrew bash 5.x in PATH, OR implement the mapping with
positional arrays and `case` statements. Document the choice inline.

---

## Open Question for Human (non-blocking — story can proceed with default)

**Script parametrization UX (default: Option A is implemented):**

Should the 3-org demo configuration be hard-coded in `scripts/demo.toml` (the implementer's
default, aligned with the existing `demo.toml` pattern), or should `demo-setup.sh` and
`demo-run.sh` accept a `--orgs` flag for operator-supplied org definitions?

- **Option A (default — hard-coded in demo.toml):** No new CLI flags. Operators edit
  `scripts/demo.toml` to change org assignments. Simple, consistent with existing pattern.
- **Option B (CLI-parameterized):** `demo-setup.sh --orgs org-a:crowdstrike,armis ...`.
  More flexible; harder to document; not needed for the 3-org demo.

The implementer proceeds with Option A. If the human prefers Option B before merge, this
can be addressed at PR review time without restarting the story.

---

## Changelog

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-06-14 | story-writer | Initial materialization from D-1029 draft stub. Full spec with 10 ACs, consolidation decision (retire start-demo.sh), 3-org model, N×M overlay and credential generalization, all 6 context-engineering sections. status: ready. |
