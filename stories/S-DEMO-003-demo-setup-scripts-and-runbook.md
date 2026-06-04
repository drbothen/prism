---
document_type: story
story_id: S-DEMO-003
title: "scripts: Demo Setup Scripts + prism-credential-set CLI Subcommand + Operator Runbook"
wave: 5
epic_id: E-DEMO
priority: P1
status: ready
version: "1.1"
level: "L4"
producer: story-writer
timestamp: "2026-05-29T00:00:00Z"
tdd_mode: strict
subsystems: [SS-03, SS-06, SS-22]
# Subsystem anchor justifications:
#   SS-03 (Credential Management) owns the `prism credential set` subcommand that writes to
#     the OS keyring per AD-017; this story adds the first CLI-facing write path for credentials.
#   SS-06 (Client Configuration) owns the demo prism.toml + per-org overlay TOML generation;
#     the setup scripts create the config directory structure that prism-bin reads at startup.
#   SS-22 (Binary Entrypoint) owns the prism-bin CLI subcommand dispatch; `prism credential set`
#     is a new subcommand added alongside the existing `start` subcommand.
crates_touched: [prism-bin]
target_module: prism-bin
capabilities: [CAP-004, CAP-009, CAP-034]
behavioral_contracts:
  - BC-2.03.005  # Credential CRUD Operations via MCP Tools (Mutations Require Confirmation Token)
                 # The `prism credential set` subcommand is a CLI surface for the same credential
                 # write operation specified in BC-2.03.005 for MCP. The CLI path bypasses the
                 # confirmation token requirement (direct human operator, not AI agent).
  - BC-2.03.007  # Secret Redaction in Logs, Errors, and MCP Responses — credential values passed
                 # to `prism credential set` must never appear in logs or stderr output. The CLI
                 # reads the value from stdin (not from args) per AD-017.
  - BC-2.06.001  # TOML Configuration Loads and Deserializes at Startup — the demo setup script
                 # generates a valid prism.toml that BC-2.06.001 must accept without error.
  - BC-2.22.001  # Boot Orchestration — after setup scripts complete, `prism-bin start` must
                 # complete all boot steps and accept MCP connections per BC-2.22.001.
verification_properties: []
depends_on:
  - S-DEMO-001   # Adapter registration must work before the demo makes sense to run.
  - S-DEMO-002   # Smoke test must pass before runbook ships; runbook documents the green path.
blocks: []
points: 5
# Points justification:
#   - prism credential set CLI subcommand (new Clap subcommand + keyring write + stdin prompt): 2 pts
#   - scripts/demo-setup.sh (idempotent, build + config + overlay + credentials): 1.5 pts
#   - scripts/demo-run.sh + scripts/demo-teardown.sh: 0.5 pts
#   - docs/DEMO-RUNBOOK.md (comprehensive, all sections): 1 pt
#   Total: 5 points (~1-1.5 days)
#   Bundling decision: story-writer chose to bundle `prism credential set` into this story
#   (not split into S-DEMO-003A + S-DEMO-003B) because the CLI subcommand is a prerequisite
#   for the setup script (AC-005); splitting would add a story-to-story dep with no parallelism gain.
estimated_days: 1
risk: LOW
# Risk justification: Bash scripting + markdown + one new Clap subcommand. The Clap subcommand
# is the highest-risk item: it requires adding a new variant to the existing CLI enum in boot.rs
# or a separate clap::Parser derive. Main risk: AD-017 stdin-prompt credential read must not
# echo the value to the terminal (rpassword crate or raw terminal mode required).
acceptance_criteria_count: 8
red_gate_tests: 2
estimated_passes: "1-2 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "AD-017 compliance: `prism credential set` MUST NOT accept the credential value as a CLI
    arg (visible in process listing and shell history). Value must be read from stdin using
    rpassword or equivalent, which disables terminal echo."
  - "Shellcheck: all shell scripts must pass shellcheck CI gate. Add shellcheck to the
    pre-push Justfile recipe or CI matrix if not already present."
  - "Idempotency: demo-setup.sh must be safe to run multiple times. Directory creation uses
    mkdir -p; keyring writes overwrite existing entries; TOML generation overwrites files."
  - "Plugin artifact path: scripts/demo-setup.sh must locate crowdstrike-oauth2.prx correctly.
    S-PLUGIN-CI-001 committed the artifact; the exact path needs to be verified before scripting."
inputs:
  - "crates/prism-bin/src/boot.rs"
  - "crates/prism-bin/src/main.rs"
  - "crates/prism-sensors/specs/crowdstrike.sensor.toml"
  - "crates/prism-sensors/specs/armis.sensor.toml"
  - "crates/prism-sensors/specs/claroty.sensor.toml"
  - "crates/prism-sensors/specs/cyberint.sensor.toml"
  - ".factory/specs/behavioral-contracts/BC-2.03.005-credential-crud-mcp-tools.md"
  - ".factory/specs/behavioral-contracts/BC-2.03.007-secret-redaction.md"
  - ".factory/specs/behavioral-contracts/BC-2.06.001-toml-config-loading.md"
  - ".factory/specs/behavioral-contracts/BC-2.22.001-boot-orchestration.md"
  - ".factory/proposals/E2E-DEMO-WIRING-PLAN.md"
  - ".factory/stories/S-DEMO-002-e2e-subprocess-smoke-test-all-sensors.md"
input-hash: null
traces_to: []
cycle: "v1.0.0-brownfield"
phase: 3
---

# S-DEMO-003 — scripts: Demo Setup Scripts + `prism credential set` CLI + Operator Runbook

**Story ID:** S-DEMO-003
**Status:** draft
**Version:** v1.0
**Wave:** 5
**Priority:** P1
**Points:** 5

---

## Origin

New story required per E2E-DEMO-WIRING-PLAN.md §2 (i) "Install + setup runbook" and §4
"Install + Setup Outline". Credential bootstrap mechanism expanded to include a new
`prism credential set` CLI subcommand (AD-017 compliant) rather than relying on platform
keyring CLI tools. Story-writer bundled the CLI subcommand into this story rather than
splitting to avoid an unnecessary story dependency edge with no parallelism gain.

---

## Narrative

As an MSSP analyst or demo operator, I want a one-command setup script and a clear runbook
so that I can stand up the prism DTU demo environment in under 5 minutes on a fresh machine,
connect Claude Code as an MCP client, and issue live queries against all 4 sensor DTU clones
without reading source code.

---

## Behavioral Contracts

| BC ID | Title |
|-------|-------|
| BC-2.03.005 | Credential CRUD Operations via MCP Tools (Mutations Require Confirmation Token) |
| BC-2.03.007 | Secret Redaction in Logs, Errors, and MCP Responses |
| BC-2.06.001 | TOML Configuration Loads and Deserializes at Startup |
| BC-2.22.001 | Boot Orchestration — Sequencing, Exit-Code Map, and Pre-Traffic Gate |

---

## Acceptance Criteria

### AC-001: `scripts/demo-setup.sh` runs to completion on macOS and Linux (fresh workspace)
Given: A fresh clone of the prism repository with Rust toolchain installed per `rust-toolchain.toml`.
When: A user runs `bash scripts/demo-setup.sh` from the repo root.
Then: The script exits 0. The demo config directory (`~/.config/prism-demo/`) is created with
all required subdirectories; sensor TOML specs are copied; crowdstrike-oauth2.prx is copied
to the plugin dir; dummy credentials are bootstrapped in the OS keyring; a valid `prism.toml`
is generated. No manual steps required between running the script and `prism-bin start`.
(traces to BC-2.06.001 postcondition: "TOML config loads and deserializes at startup" — the
generated prism.toml must be schema-valid and accepted without error)
Red Gate test: `test_BC_2_06_001_demo_setup_generates_valid_prism_toml`

### AC-002: After setup, `prism-bin start` boots successfully and accepts MCP connections
Given: `scripts/demo-setup.sh` has completed successfully.
When: `./target/release/prism start --config ~/.config/prism-demo/` is executed.
Then: prism-bin completes all boot steps, emits `boot.step9a.adapter_registry_populated`
event with `sensor_count=4` and `org_count=1`, and accepts MCP connections (no error exit).
(traces to BC-2.22.001 postcondition: "The process is in steady state: all subsystem handles
available, traffic gate open")

### AC-003: `scripts/demo-run.sh` starts and stops DTU server cleanly
Given: `scripts/demo-setup.sh` has completed.
When: `bash scripts/demo-run.sh` is executed.
Then: `prism-dtu-demo-server start` launches in the background; a URL file appears within 10s;
the script prints the CrowdStrike, Armis, Claroty, and Cyberint DTU ports; the script completes
with exit 0. When `bash scripts/demo-teardown.sh` is run, the DTU server is killed cleanly.
(traces to BC-2.22.001 precondition: "The Prism binary has been invoked with the `start` subcommand")

### AC-004: `docs/DEMO-RUNBOOK.md` covers connect-from-Claude-Code instructions
Given: The runbook exists at `docs/DEMO-RUNBOOK.md`.
When: An operator reads §4 of the runbook.
Then: The section "Connecting Claude Code" contains exact steps to add prism-bin as an MCP server
in `~/.claude/settings.json` (or equivalent), including the exact binary path and invocation command.
(traces to BC-2.10.001: "rmcp ServerHandler Implementation" — the runbook documents how to
connect an MCP client to the stdio transport)

### AC-005: `prism credential set` subcommand writes to OS keyring; AD-017 compliant
Given: `prism-bin` is invoked as `prism credential set --sensor crowdstrike --name client_id`.
When: The subcommand runs.
Then: It prompts "Enter value: " on stderr (no terminal echo); reads the value from stdin;
writes it to the OS keyring under namespace `prism/crowdstrike/client_id` per BC-2.03.004;
exits 0; the value is NOT logged, NOT printed to stdout, and NOT visible in `ps aux` output
(i.e., NOT a CLI arg).
(traces to BC-2.03.007 postcondition: "Secret Redaction in Logs, Errors, and MCP Responses")
Red Gate test: `test_BC_2_03_007_prism_credential_set_does_not_echo_value_to_stdout`

### AC-006: Runbook documents troubleshooting for 3 common failure modes
Given: `docs/DEMO-RUNBOOK.md` §Troubleshooting section.
When: An operator reads it.
Then: The section covers at least:
  - (a) "Keyring access denied" — explains platform-specific keyring permission grant
  - (b) "Port already in use" — explains how to kill stale DTU server and re-run demo-run.sh
  - (c) "TOML spec not found" — explains how to verify spec_dir and sensor TOML filenames
(traces to BC-2.06.007: "Missing Required Fields Produce Actionable Error Messages" — runbook
supplements error messages with human-readable remediation steps)

### AC-007: `scripts/demo-teardown.sh` removes generated files and keyring entries
Given: The demo environment is set up and running.
When: `bash scripts/demo-teardown.sh` is executed.
Then: DTU server is killed; `~/.config/prism-demo/` is removed; the 8 OS keyring entries
(client_id + client_secret for each of 4 sensors) are deleted via `prism credential delete`
or the platform keyring CLI; exits 0.
(traces to BC-2.03.005: "Credential CRUD Operations via MCP Tools" — teardown exercises the
delete path of the credential subsystem)

### AC-008: All shell scripts pass `shellcheck` with no errors or warnings
Given: The 3 shell scripts (`demo-setup.sh`, `demo-run.sh`, `demo-teardown.sh`) are committed.
When: `shellcheck scripts/demo-*.sh` is executed in CI.
Then: Zero errors, zero warnings. If `shellcheck` is not already in CI, this story adds it to
the Justfile `check-ci` recipe.
(traces to BC-2.22.001 invariant: "boot orchestration is deterministic" — deterministic setup
scripts are a prerequisite for reproducible boot)

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| `prism credential set` value MUST come from stdin, not CLI arg | AD-017 AI-opaque credential model | `clap` arg for `--value` is FORBIDDEN; stdin read is mandatory |
| Credential namespace: `prism/{sensor_id}/{name}` | BC-2.03.004 Namespace Isolation | Use `CredentialStore::set(org_id, sensor_id, name, value)` via prism-credentials crate |
| `rpassword` or equivalent for no-echo stdin read | BC-2.03.007 Secret Redaction | `read -s` in bash is acceptable for shell scripts; Rust CLI must use `rpassword` crate |
| Runbook must not contain real credential values | AD-017 | Use placeholder strings `"<your-client-id>"` in examples; DTU demo uses `"demo-client"` only |
| Shell scripts use `#!/usr/bin/env bash` shebang | Portability | Required by shellcheck |
| crowdstrike-oauth2.prx path must be validated | Risk mitigation | Script checks file exists before copying; exits 1 with actionable message if not found |

---

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `clap` | workspace version | New `credential set` subcommand (derive mode, matches existing CLI pattern) |
| `rpassword` | `7.*` | Prompt for credential value with terminal echo disabled |
| `prism-credentials` | workspace path | `CredentialStore::set()` for keyring write |
| `shellcheck` | any stable | Shell script linting in CI |

Note: `rpassword` version should be checked against `crates.io` at implementation time if not
already in workspace `Cargo.toml`. If it is not present, add it as a dev-dep or feature-gated dep.

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `scripts/demo-setup.sh` | CREATE | Idempotent one-time setup: build, config, specs, plugin, credentials, overlay |
| `scripts/demo-run.sh` | CREATE | Daily launch: start DTU server, print ports, instructions |
| `scripts/demo-teardown.sh` | CREATE | Cleanup: kill DTU, remove config dir, delete keyring entries |
| `scripts/demo.toml` | CREATE | DTU demo server config (all 4 sensors, ephemeral ports) |
| `docs/DEMO-RUNBOOK.md` | CREATE | Comprehensive operator runbook (7 sections per scope) |
| `crates/prism-bin/src/cli.rs` (or `main.rs`) | MODIFY | Add `Credential(CredentialArgs)` variant to the Clap CLI enum |
| `crates/prism-bin/src/credential_cli.rs` | CREATE | `CredentialArgs` struct + `handle_credential_set()` async fn |

---

## `prism credential set` Subcommand Specification

The new CLI subcommand must follow the existing `prism start` pattern:

```
USAGE:
    prism credential set --sensor <SENSOR_ID> --name <CREDENTIAL_NAME>

ARGS:
    --sensor <SENSOR_ID>           Sensor ID (e.g., crowdstrike, armis, claroty, cyberint)
    --name <CREDENTIAL_NAME>       Credential name (e.g., client_id, client_secret, api_token)
    --org-slug <ORG_SLUG>          Org slug (default: first org in prism.toml, optional)

BEHAVIOR:
    Prompts "Enter value for prism/<sensor>/<name>: " on stderr.
    Reads value from stdin with terminal echo disabled (rpassword).
    Writes to OS keyring under namespace "prism/<sensor_id>/<name>" scoped to org_id.
    Prints "Credential stored successfully." to stdout on success.
    Exits 0 on success; exits 1 on keyring write failure with actionable error message on stderr.
```

The `--value` flag is explicitly FORBIDDEN (AD-017 compliance). The subcommand reads config
from the same `--config` flag as `prism start`.

---

## `docs/DEMO-RUNBOOK.md` Required Sections

The runbook must contain exactly these sections (in this order):

1. **Prerequisites** — Rust toolchain, `just`, `shellcheck`, `cargo nextest`
2. **One-time Setup** — run `demo-setup.sh`; what it does step-by-step
3. **Daily Demo Run** — run `demo-run.sh`; verify DTU clones are up
4. **Connecting Claude Code** — add prism-bin to `~/.claude/settings.json` MCP servers section; exact JSON snippet
5. **Example Queries** — one query per sensor (CrowdStrike, Armis, Claroty, Cyberint) with expected output shape
6. **Troubleshooting** — 3 failure modes (AC-006)
7. **Cleanup** — run `demo-teardown.sh`

---

## Tasks

1. **Read** `crates/prism-bin/src/main.rs` — understand existing Clap CLI structure before adding new subcommand.
2. **Read** `crates/prism-credentials/src/lib.rs` — understand `CredentialStore::set()` signature and OS keyring namespace format.
3. **Find** `crowdstrike-oauth2.prx` committed path (S-PLUGIN-CI-001 merged it at `crates/prism-spec-engine/wasm/` or similar; read that commit).
4. **Write Red Gate tests** for AC-001 and AC-005 (see test names above).
5. **Implement** `crates/prism-bin/src/credential_cli.rs` with `CredentialArgs` and `handle_credential_set()`.
6. **Add** `Credential(CredentialArgs)` variant to CLI enum in `main.rs` / `cli.rs`.
7. **Write** `scripts/demo-setup.sh` — sections: build → mkdir → copy specs → copy plugin → write prism.toml → write credentials → print instructions.
8. **Write** `scripts/demo-run.sh` — start DTU in background → poll urls.json → print ports → print prism-bin start command.
9. **Write** `scripts/demo-teardown.sh` — kill DTU → remove config dir → delete keyring entries.
10. **Write** `scripts/demo.toml` — DTU demo server config for all 4 sensors.
11. **Write** `docs/DEMO-RUNBOOK.md` — all 7 sections; include exact Claude Code settings.json snippet.
12. **Run** `shellcheck scripts/demo-*.sh` — fix all warnings.
13. **Run** Red Gate tests: `just iter prism-bin` — both must pass GREEN.
14. **Run** `just check` — final pre-push gate.

---

## Previous Story Intelligence

- **S-DEMO-001** (depends_on): The `boot.step9a.adapter_registry_populated` event with `sensor_count=4` is the boot-success signal that AC-002 validates. Read S-DEMO-001 for the event field names.
- **S-DEMO-002** (depends_on): The E2E smoke test's `bootstrap_credentials()` helper uses the same credential namespace (`prism/<sensor_id>/<name>`) as `prism credential set`. The two must be consistent.
- **S-CONFIG-MULTI-TENANT-OVERRIDE-001** (already merged): The per-org overlay format (`customers/demo-org/crowdstrike.sensor.toml` with `extends = "crowdstrike"` and `base_url = "http://127.0.0.1:<PORT>"`) is documented in that story. `demo-setup.sh` generates these overlay files after reading the DTU server ports from `urls.json`.
- **S-PLUGIN-CI-001** (merged): Committed `crowdstrike-oauth2.prx`. The demo-setup.sh script copies this file to the plugin dir. Implementer must find the committed path by reading that story's demo evidence or the git tree.

---

## Open Questions

1. **Plugin artifact committed path**: S-PLUGIN-CI-001 committed `crowdstrike-oauth2.prx` but the story-writer does not know the exact path (possibly `crates/prism-spec-engine/wasm/crowdstrike-oauth2.prx` or `plugins/crowdstrike-oauth2.prx`). Implementer must verify before writing `demo-setup.sh` copy command.

2. **`rpassword` in workspace**: Is `rpassword` already in `Cargo.toml` workspace deps? If not, does it belong in the workspace or as a prism-bin-only dep? Should it be feature-gated (e.g., `#[cfg(feature = "cli-credential")]`) to avoid pulling it into the library surface? Architect to confirm.

3. **`prism credential set` for non-CrowdStrike sensors**: Armis, Claroty, Cyberint use `bearer_static` auth. The credential stored is the bearer token (one entry per sensor: `prism/armis/api_token`, `prism/claroty/api_token`, `prism/cyberint/api_token`). For DTU, these are dummy values. The `demo-setup.sh` script should set one credential per sensor. The exact credential names must match what `BearerStaticAuthProvider` resolves from the keyring at fetch time — verify against `crates/prism-sensors/specs/` auth_type declarations.

4. **Org slug in `prism credential set`**: If prism.toml has a single org (`demo-org`), the subcommand should default to that org's org_id without requiring `--org-slug`. If multiple orgs are present, `--org-slug` is required. Confirm this UX with the architect.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | OS keyring not available (e.g., headless CI without keyring service) | `prism credential set` exits 1 with: "Keyring unavailable: <platform reason>. Use the encrypted file backend instead." |
| EC-002 | `demo-setup.sh` run twice in succession | Second run is idempotent: overwrite files, overwrite keyring entries; no error |
| EC-003 | crowdstrike-oauth2.prx not found at expected path | `demo-setup.sh` exits 1 with: "Plugin artifact not found at <path>. Run `cargo build -p prism-spec-engine --features wasm-plugins` first." |
| EC-004 | DTU server not started before demo-run.sh | `demo-run.sh` polls urls.json for 30s then exits 1 with: "DTU server did not start within 30s. Check demo.toml for port conflicts." |
| EC-005 | `prism credential set` called with `--value` flag (attempted AD-017 bypass) | Clap rejects: "error: unexpected argument '--value' found. Values must be provided interactively." |

---

## Token Budget Estimate

| Context source | Estimated tokens |
|----------------|-----------------|
| This story spec | ~4,000 |
| BC files (4 BCs) | ~6,000 |
| crates/prism-bin/src/main.rs (CLI structure) | ~3,000 |
| crates/prism-credentials/src/lib.rs | ~4,000 |
| crates/prism-sensors/specs/ (4 TOML files) | ~4,000 |
| ADR-028 §D10, AD-017 (auth + credential model) | ~3,000 |
| S-DEMO-001 + S-DEMO-002 (dependency context) | ~6,000 |
| **Total estimate** | **~30,000 tokens (~12% of 256K context)** |

Smallest story in the E-DEMO epic by token budget. Well within limit.

---

## Forbidden Dependencies

| Forbidden | Reason |
|-----------|--------|
| CLI `--value` argument for credential value | AD-017 — value must come from stdin, not args |
| Hardcoded port numbers in setup scripts | DTU binds to ephemeral ports; always read from urls.json |
| `echo` or `printf` of credential value to any file descriptor | BC-2.03.007 Secret Redaction |
| Credential values in `docs/DEMO-RUNBOOK.md` examples | AD-017 — use placeholder strings only |

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.1 | 2026-06-03 | state-manager | D-990 Phase-A-close: status draft→ready; depends_on S-DEMO-001 (merged PR #166) + S-DEMO-002 (merged PR #171) BOTH SATISFIED; BC-2.03.005 v1.6 active + BC-2.03.007 v1.3 active (consumer-ref to draft BC-2.06.001 does not block per PO ruling); S-7.01 gate CLEARED. |
| 1.0 | 2026-05-29 | story-writer | Initial draft — bundled CLI subcommand per complexity assessment; 4-sensor scope |
