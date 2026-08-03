---
document_type: story
story_id: S-DEMO-003
title: "scripts: Demo Setup Scripts + prism credential set/delete CLI Subcommands (Tier-3 Keyring) + Operator Runbook"
wave: 5
epic_id: E-DEMO
priority: P1
status: merged
version: "1.18"
level: "L4"
producer: story-writer
timestamp: "2026-05-29T00:00:00Z"
tdd_mode: strict
subsystems: [SS-03, SS-06, SS-08, SS-22]
# Subsystem anchor justifications:
#   SS-03 (Credential Management) owns the `prism credential set` subcommand that writes to
#     the OS keyring per AD-017, AND the Tier-3 resolver branch in resolve_credential
#     (prism-credentials::resolution.rs). The OrgId-keyed namespace reconciliation (CRIT-2)
#     and BackendUnavailable error semantics (E-CRED-008) are both Credential Management concerns.
#     Per ARCH-INDEX Subsystem Registry SS-03.
#   SS-06 (Client Configuration) owns the demo prism.toml + per-org overlay TOML generation;
#     the setup scripts create the config directory structure that prism-bin reads at startup.
#     Also owns the prism.toml load in credential_cli.rs for slug→OrgId mapping (ADR-034 §D3).
#     Per ARCH-INDEX Subsystem Registry SS-06.
#   SS-08 (Spec-Driven Adapter) owns the auth provider construction sites in
#     spec_driven_adapter.rs (step 9A) where PrismCredentialResolver gains Arc<OrgRegistry>
#     and Arc<dyn CredentialStoreOrgId> wiring (ADR-034 §D5). StaticCookieAuthProvider and
#     PluginAuthProvider are SS-08 components. Per ARCH-INDEX Subsystem Registry SS-08.
#   SS-22 (Binary Entrypoint) owns the prism-bin CLI subcommand dispatch; `prism credential set`
#     is a new subcommand added alongside the existing `start` subcommand. Also owns boot.rs
#     BootContext expansion (credential_store_org_id field, ADR-034 §D5). Per ARCH-INDEX SS-22.
crates_touched: [prism-bin, prism-credentials, prism-spec-engine]
# crates_touched justification (ADR-034 §File Create / Modify List):
#   prism-bin: credential_cli.rs (OrgId-keyed write + HIGH-3 fix), boot.rs (BootContext expansion),
#     spec_driven_adapter.rs (step 9A auth provider wiring), scripts/, docs/, .github/workflows/ci.yml
#   prism-credentials: resolution.rs (Tier-3 branch + signature), lib.rs (re-export update)
#   prism-spec-engine: auth_provider.rs (PrismCredentialResolver struct + 3 test doubles),
#     plugin_auth_provider.rs (PluginAuthProvider DI fields)
target_module: prism-bin
capabilities: [CAP-004, CAP-009, CAP-034]
behavioral_contracts:
  - BC-2.03.005  # Credential CRUD Operations via MCP Tools (Mutations Require Confirmation Token)
                 # The `prism credential set` subcommand is a CLI surface for the same credential
                 # write operation; CLI path bypasses confirmation token (direct human operator).
  - BC-2.03.007  # Secret Redaction in Logs, Errors, and MCP Responses — credential values passed
                 # to `prism credential set` must never appear in logs or stderr output (AD-017).
                 # Also: E-CRED-008 detail string must NOT contain a credential value (D4).
  - BC-2.06.001  # TOML Configuration Loads and Deserializes at Startup — the generated prism.toml
                 # must be schema-valid and accepted without error.
  - BC-2.06.003  # Credential Reference Resolution — Tier-3 OS-keyring resolution IMPLEMENTED per
                 # ADR-034. OrgId-keyed namespace (namespace_key_by_org_id). resolve_credential
                 # gains org_id + keyring parameters. set_by_org is the write path (CRIT-2 fix).
  - BC-2.22.001  # Boot Orchestration — after setup scripts complete, `prism-bin start` must
                 # complete all boot steps; BootContext gains credential_store_org_id (ADR-034 §D5).
verification_properties: []
depends_on:
  - S-DEMO-001   # Adapter registration must work before the demo makes sense to run.
  - S-DEMO-002   # Smoke test must pass before runbook ships; runbook documents the green path.
blocks: []
points: 8
# Points justification (ADR-034 §D7 — Option-A scope expansion):
#   - prism credential set: OrgId-keyed write via CredentialStoreOrgId::set_by_org +
#     prism.toml load for slug→OrgId + HIGH-3 error fix (no demo-org fallback): 2 pts
#   - resolve_credential: Tier-3 branch + signature change (org_id + keyring params): 1.5 pts
#   - PrismCredentialResolver → struct with fields; 3 test double sibling sweep: 1 pt
#   - StaticCookieAuthProvider + PluginAuthProvider DI fields; spec_driven_adapter.rs callsites: 1 pt
#   - boot.rs BootContext.credential_store_org_id + step 5 wiring: 0.5 pts
#   - Red Gate tests (RG-034-001..005 + F-P10-HIGH-001 delete test): 1 pt
#   - scripts (demo-setup.sh, demo-run.sh, demo-teardown.sh) + demo.toml: 0.5 pts
#   - docs/DEMO-RUNBOOK.md + HIGH-1 env format + HIGH-2 shellcheck CI: 0.5 pts
#   Total: 8 points (~1.5-2 days); ADR-034 §D7 gives 8-10 range — 8 chosen as lower bound
#   (all ADR-034 work items listed; no scope uncertainty remains after human Option-A decision).
estimated_days: 2
risk: MEDIUM
# Risk justification (ADR-034 §D7): resolve_credential signature change is a sibling-site blast
# radius change (TD-VSDD-060) across 3 crates. The Tier-3 keyring call is async (spawn_blocking
# internally in KeyringBackend). Well-understood DI injection pattern (ADR-022 §C). 4 Red Gate
# tests bound the risk at the test level. Risk level MEDIUM per ADR-034 §D7 (signature blast
# radius across 3 crates; well-understood pattern).
acceptance_criteria_count: 14
red_gate_tests: 9
estimated_passes: "2-3 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "AD-017 compliance: `prism credential set` MUST NOT accept the credential value as a CLI
    arg (visible in process listing and shell history). Value must be read from stdin using
    rpassword or equivalent, which disables terminal echo."
  - "Shellcheck: all shell scripts must pass shellcheck in GitHub CI (.github/workflows/ci.yml),
    not only in the Justfile. HIGH-2 fix: add shellcheck step to ci.yml matrix."
  - "Idempotency: demo-setup.sh must be safe to run multiple times. Directory creation uses
    mkdir -p; keyring writes overwrite existing entries; TOML generation overwrites files."
  - "Plugin artifact path: scripts/demo-setup.sh must locate crowdstrike-oauth2.prx correctly.
    S-PLUGIN-CI-001 committed the artifact; the exact path needs to be verified before scripting."
  - "TD-VSDD-060 sibling-site sweep: resolve_credential signature change (2 new params) must
    be applied to ALL callsites in prism-credentials, prism-spec-engine, and prism-bin.
    Grep for 'resolve_credential' before declaring implementation done."
  - "E-CRED-008 detail string: must contain only the keyring-rs system error string (e.g.,
    'access denied', 'D-Bus unavailable') — never a credential value. AD-017 + BC-2.03.007."
inputs:
  - "crates/prism-bin/src/boot.rs"
  - "crates/prism-bin/src/main.rs"
  - "crates/prism-credentials/src/resolution.rs"
  - "crates/prism-credentials/src/lib.rs"
  - "crates/prism-credentials/src/keyring.rs"
  - "crates/prism-credentials/src/namespace.rs"
  - "crates/prism-spec-engine/src/auth_provider.rs"
  - "crates/prism-spec-engine/src/plugin_auth_provider.rs"
  - "crates/prism-bin/src/spec_driven_adapter.rs"
  - "crates/prism-sensors/specs/crowdstrike.sensor.toml"
  - "crates/prism-sensors/specs/armis.sensor.toml"
  - "crates/prism-sensors/specs/claroty.sensor.toml"
  - "crates/prism-sensors/specs/cyberint.sensor.toml"
  - ".factory/specs/behavioral-contracts/BC-2.03.005-credential-crud-mcp-tools.md"
  - ".factory/specs/behavioral-contracts/BC-2.03.007-secret-redaction.md"
  - ".factory/specs/behavioral-contracts/BC-2.06.001-toml-config-loading.md"
  - ".factory/specs/behavioral-contracts/BC-2.06.003-credential-reference-resolution.md"
  - ".factory/specs/behavioral-contracts/BC-2.22.001-boot-orchestration.md"
  - ".factory/specs/architecture/decisions/ADR-034-tier3-keyring-resolution-org-id-threading.md"
  - ".factory/proposals/E2E-DEMO-WIRING-PLAN.md"
  - ".factory/stories/S-DEMO-002-e2e-subprocess-smoke-test-all-sensors.md"
input-hash: null
traces_to: []
cycle: "v1.0.0-brownfield"
phase: 3
---

# S-DEMO-003 — scripts: Demo Setup Scripts + `prism credential set` / `prism credential delete` CLI (Tier-3 Keyring) + Operator Runbook

**Story ID:** S-DEMO-003
**Status:** in_progress
**Version:** v1.18
**Wave:** 5
**Priority:** P1
**Points:** 8

---

## Authority

ADR-034 is the authoritative design document for this story. It defines Tier-3 OS-keyring
resolution (§D3 OrgId-keyed branch in resolve_credential), OrgId-keyed write reconciliation
(§D4 set_by_org via CredentialStoreOrgId), BootContext expansion (§D5), and
PrismCredentialResolver Arc-DI wiring (§D5). Read it before implementing:
`.factory/specs/architecture/decisions/ADR-034-tier3-keyring-resolution-org-id-threading.md`.

ADR-035 §D5 defines the canonical E-CRED-001..010 error codes. The Tier-3 backend-unavailable
path is E-CRED-008 (BackendUnavailable); the write-path failure is E-CRED-004. All error
codes in this story must match the ADR-035 taxonomy. Read §D5 before implementing error paths:
`.factory/specs/architecture/decisions/ADR-035-e-cred-namespace-reconciliation.md`.

---

## Origin

New story required per E2E-DEMO-WIRING-PLAN.md §2 (i) "Install + setup runbook" and §4
"Install + Setup Outline". Credential bootstrap mechanism expanded to include a new
`prism credential set` CLI subcommand (AD-017 compliant) that is now the **load-bearing
end-to-end credential path** for the demo — not merely a convenience writer.

**Option-A scope expansion (2026-06-06):** Human approved implementing full Tier-3 OS-keyring
credential resolution in `resolve_credential` so that `prism credential set` is the canonical
credential bootstrap channel for the demo. ADR-034 (accepted 2026-06-06) governs the design.
Two critical gaps addressed: CRIT-1 (missing Tier-3 resolver branch) and CRIT-2 (namespace
mismatch between slug-keyed write and OrgId-keyed read). Points increased 5→8. Risk elevated
LOW→MEDIUM. Three additional crates in scope: `prism-credentials`, `prism-spec-engine`.

---

## Narrative

As an MSSP analyst or demo operator, I want a one-command setup script and a clear runbook
so that I can stand up the prism DTU demo environment in under 5 minutes on a fresh machine,
connect Claude Code as an MCP client, and issue live queries against all 4 sensor DTU clones
without reading source code — using `prism credential set` as the canonical credential
bootstrap that writes to the OS keyring and is resolved end-to-end at query time.

---

## Behavioral Contracts

| BC ID | Title |
|-------|-------|
| BC-2.03.005 | Credential CRUD Operations via MCP Tools (Mutations Require Confirmation Token) |
| BC-2.03.007 | Secret Redaction in Logs, Errors, and MCP Responses |
| BC-2.06.001 | TOML Configuration Loads and Deserializes at Startup |
| BC-2.06.003 | Credential References in Config Resolve to Credential Store Entries |
| BC-2.22.001 | Boot Orchestration — Sequencing, Exit-Code Map, and Pre-Traffic Gate |

---

## Acceptance Criteria

### AC-001: `scripts/demo-setup.sh` runs to completion on macOS and Linux (fresh workspace)
Given: A fresh clone of the prism repository with Rust toolchain installed per `rust-toolchain.toml`.
When: A user runs `bash scripts/demo-setup.sh` from the repo root.
Then: The script exits 0. The demo config directory (`~/.config/prism-demo/`) is created with
all required subdirectories; sensor TOML specs are copied; crowdstrike-oauth2.prx is copied
to the plugin dir; credentials are bootstrapped in the OS keyring via `prism credential set`
(OrgId-keyed, ADR-034 §D3); a valid `prism.toml` is generated. No manual steps required
between running the script and subsequently running `demo-run.sh` (which handles DTU launch
and per-org `base_url` overlay generation). `demo-setup.sh` does NOT write per-org
`base_url` overlay TOMLs — those require ephemeral DTU ports that are only available
post-launch and are written by `demo-run.sh`.
(traces to BC-2.06.001 postcondition: "TOML config loads and deserializes at startup" — the
generated prism.toml must be schema-valid and accepted without error)
Red Gate test: `test_BC_2_06_001_demo_setup_generates_valid_prism_toml`

### AC-002: After setup, `prism-bin start` boots successfully and accepts MCP connections
Given: `scripts/demo-setup.sh` has completed successfully (credentials written OrgId-keyed).
When: `./target/release/prism --config-dir ~/.config/prism-demo/ start` is executed.
Then: prism-bin completes all boot steps, emits `boot.step9a.adapter_registry_populated`
event with `sensor_count=4` and `org_count=1`, and accepts MCP connections (no error exit).
BootContext includes `credential_store_org_id: Arc<dyn CredentialStoreOrgId>` (ADR-034 §D5).
Boot step 5 (`KeyringCredentialProbe::probe`) resolves OrgId-keyed credentials (Tier 3a,
`get_by_org` per registered org) as the PRIMARY keyring check — credentials written by
`prism credential set` are found at this probe, not only at the legacy Tier 3b fallback
(BC-2.06.003 v1.8 §Boot-Step-5 Probe Alignment; closes F-P14-CRIT-001).
(traces to BC-2.22.001 postcondition: "The process is in steady state: all subsystem handles
available, traffic gate open")

### AC-003: `scripts/demo-run.sh` starts and stops DTU server cleanly
Given: `scripts/demo-setup.sh` has completed.
When: `bash scripts/demo-run.sh` is executed.
Then: `prism-dtu-demo-server start` launches in the background; a URL file appears within 30s;
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

### AC-005: `prism credential set` writes OrgId-keyed keyring entry via `CredentialStoreOrgId::set_by_org`; AD-017 compliant
Given: A `prism.toml` exists in the config dir with one org entry containing an `org_id` UUID.
`prism-bin` is invoked as `prism credential set --sensor crowdstrike --name client_id`.
When: The subcommand runs.
Then:
  - It prompts "Enter value for prism/<sensor>/<name>: " on stderr (no terminal echo, via `eprint!`); reads the value from stdin (rpassword);
  - Maps `--org-slug` (or the single org when unambiguous) → `OrgId` UUID by reading `PrismConfig.orgs[n].org_id` from prism.toml;
  - Writes via `CredentialStoreOrgId::set_by_org(&org_id, sensor, &cred_name, value)` producing keyring key `"{org_id_uuid}/{sensor}/{name}"` (ADR-034 §D3);
  - Does NOT write via legacy `CredentialStore::set` (slug-keyed `"{slug}/{sensor}/{name}"`);
  - `--org-slug` is required when `config.orgs.len() > 1`; when exactly 1 org, defaults to that org;
  - Exits 0; the value is NOT logged, NOT printed to stdout, and NOT visible in `ps aux` output.
(traces to BC-2.03.007 postcondition: "Secret Redaction in Logs, Errors, and MCP Responses";
traces to BC-2.06.003 Tier-3 postcondition: OrgId-keyed write via `set_by_org` — ADR-034 §D3)
Red Gate tests: `test_BC_2_03_007_prism_credential_set_does_not_echo_value_to_stdout`;
  `test_handle_credential_set_writes_org_id_keyed_namespace` (RG-034-004)

### AC-009 (Tier-3 end-to-end — CRIT-1 gap closure): A credential written by `prism credential set` is resolved at Tier-3 by `resolve_credential`; demo queries succeed against all 4 sensors.
Given: `prism credential set` has been called for all sensor/credential combos (OrgId-keyed write).
No env vars (`PRISM_CLIENTS_*`) are set for these credentials. Tier-4 CRUD store is empty.
**`demo-run.sh` has been executed** and has established the following two-part data-path precondition:

**(a) TYPE-spec env vars (boot gate — `env_resolver.rs` step 4a):** Before launching `prism-bin`,
`demo-run.sh` exports the 4 sensor `base_url` TYPE-spec env vars in the `prism start` command
environment so that step 4a env-resolution (env_resolver.rs) can substitute `${env.*}` placeholders
in the TYPE-level sensor specs:
- `CROWDSTRIKE_BASE_URL=http://127.0.0.1`
- `ARMIS_INSTANCE_URL=http://127.0.0.1`
- `CLAROTY_INSTANCE_URL=http://127.0.0.1`
- `CYBERINT_ENVIRONMENT=demo`

Without these env vars, boot STEP 4a fires `E-SPEC-024` (unresolved env placeholder) and
`boot.rs` step 4b hard-aborts the process BEFORE step 4c overlay processing runs — meaning the
per-org `base_url` overlays written in (b) are never reached. These env vars are the true
AC-002/AC-009 boot gate.

**(b) Step-4c per-org `base_url` overlays (port override):** After launching the DTU server and
parsing ephemeral ports from `urls.json`, `demo-run.sh` writes per-org overlay TOMLs
(`specs/customers/demo-org/<sensor>.sensor.toml` with `extends = "<sensor>"` and
`base_url = "http://127.0.0.1:<PORT>"`) for all 4 sensors. These step-4c overlays override the
TYPE-level `base_url` (set generically by the env vars in (a)) to the specific ephemeral DTU port.

**(c) CrowdStrike OAuth2 plugin SEC-003 (plugin-only):** The CrowdStrike OAuth2 plugin's
token-endpoint host is gated by the plugin manifest `allowed_urls` list (SEC-003,
`crates/prism-spec-engine/src/plugin/host_functions.rs`). This list must include `http://127.0.0.1`
for the DTU token endpoint to be reachable. This allowlist is written into
`crowdstrike-oauth2.manifest.toml` by `demo-setup.sh` — it applies to the CrowdStrike OAuth2
plugin ONLY, not to Armis, Claroty, or Cyberint (which use plain `reqwest::Client` with no
per-org/host egress gating).

Without the overlay-generation step (b), `prism-bin` resolves `base_url` generically (step 4a)
but routes to the wrong port at query time.
When: `resolve_credential(org_slug, sensor_id, cred_name, Some(&org_id), Some(&keyring))` is called
for each credential (as wired via `PrismCredentialResolver` in `prism-spec-engine`).
Then: `resolve_credential` returns `Ok(SecretString)` sourced from the Tier-3 keyring branch for
every credential — not `CredentialResolutionError::NotFound`. Demo MCP queries against all 4 sensors
(CrowdStrike, Armis, Claroty, Cyberint) return data rows, not authentication errors.
(traces to BC-2.06.003 Tier-3 postcondition: `get_by_org` returns `Ok(Some(secret))` → return
`Ok(secret)` + audit "keyring" — ADR-034 §D2; closes CRIT-1)
Red Gate test: `test_BC_2_06_003_tier3_credential_written_by_set_by_org_is_resolved` (RG-034-001)

### AC-010 (Namespace reconciliation — CRIT-2): Write key == Read key (OrgId-UUID); legacy slug-keyed namespace is NOT used.
Given: `handle_credential_set` is called with a known `org_id` UUID.
When: The credential is written and then read back.
Then:
  - The keyring entry exists under key `"{org_id_uuid}/{sensor}/{name}"` (verifiable via `KeyringBackend::get_by_org`);
  - The keyring entry does NOT exist under the legacy key `"{org_slug}/{sensor}/{name}"`;
  - `resolve_credential` with matching `org_id` finds the entry at Tier 3 (no fall-through to Tier 4).
(traces to BC-2.06.003 Tier-3 postcondition: OrgId-keyed key `{org_id_uuid}/{sensor_id}/{ref_name}`
via `namespace_key_by_org_id` — canonical namespace; legacy slug-keyed namespace NOT used — ADR-034 §D3)
Red Gate test: `test_handle_credential_set_writes_org_id_keyed_namespace` (RG-034-004 — also covers AC-005)

### AC-011 (Tier-3 error semantics): Keyring miss → silent fall-through to Tier-4; keyring backend error → hard `BackendUnavailable` / E-CRED-008; no value leak.
Given: `resolve_credential` is called with `Some(&org_id)` and `Some(&keyring)`.
When:
  - Case A: keyring has no entry for the OrgId-keyed key (`Ok(None)` / `NoEntry`), and CRUD store is empty.
  - Case B: keyring returns `Err(NoStorageAccess)` (backend locked/unavailable).
Then:
  - Case A: `resolve_credential` falls through silently to Tier 4; returns `CredentialResolutionError::NotFound`
    (not `BackendUnavailable`). (traces to BC-2.06.003 Tier-3 postcondition: `get_by_org Ok(None)` → fall through)
  - Case B: `resolve_credential` returns `CredentialResolutionError::BackendUnavailable { detail:
    "E-CRED-008: OS keyring unavailable: backend=<backend>: <reason>. Check keyring access
    (macOS Keychain / Linux libsecret). Use Tier 1/2 env vars as an alternative (BC-2.06.003)." }`.
    The inner detail is formatted as `"backend={backend}: {reason}"` (from `PrismError::CredentialStoreError`
    destructure in resolution.rs — F-P6-OBS-003 fix strips the E-CRED-004 prefix to avoid double-prefix output).
    Does NOT fall through to Tier 4.
    The detail string contains backend identity and the system error reason ONLY — no credential value (AD-017; BC-2.03.007).
(traces to BC-2.06.003 Tier-3 postcondition: `get_by_org Err(...)` → hard error E-CRED-008 — ADR-034 §D4 / ADR-035 §D5)
Red Gate tests: `test_BC_2_06_003_tier3_miss_falls_through_to_tier4` (RG-034-002, Case A);
  `test_BC_2_06_003_tier3_backend_error_returns_e_cred_008` (RG-034-005, Case B)

### AC-012 (HIGH-3 — `resolve_org_slug_and_id` error on missing/invalid prism.toml): When `--org-slug` is absent and prism.toml is missing or unparseable, `prism credential set` errors clearly — no silent `"demo-org"` fallback.
Given: The config directory contains no `prism.toml` (or an unparseable one), and `--org-slug` was not provided.
When: `prism credential set --sensor armis --name bearer_token` is invoked.
Then: The subcommand exits 2 (EXIT_CONFIG_INVALID — ADR-022 §A) with an actionable error message:
"Could not load prism.toml from '<config_dir>': <reason>. Ensure prism.toml exists (run
demo-setup.sh or create it manually) before running `prism credential set`." The
`"demo-org"` string MUST NOT appear as a default return value anywhere in this code path (SOUL.md §4).
(traces to BC-2.06.003 precondition: "caller supplies a valid client_id"; SOUL.md §4 swallow-error
prohibition; ADR-034 §D3 HIGH-3 remediation)
Red Gate test: `test_resolve_org_slug_errors_when_toml_missing_and_no_explicit_slug` (RG-034-003)

### AC-013 (HIGH-1 — env var format discipline): All demo scripts and runbook use ONLY the `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}` format; the retired credential env-var form `DEMO_ORG_*_SENSORS_*` is absent.
Given: The scripts (`demo-setup.sh`, `demo-run.sh`, `demo-teardown.sh`) and `docs/DEMO-RUNBOOK.md` are committed.
When: The following greps are run against all files under `scripts/` and `docs/`:
  - `grep -rE 'DEMO_ORG_[A-Z_]+_SENSORS' scripts/ docs/` — targets the retired credential
    env-var format (e.g., `DEMO_ORG_SLUG_SENSORS_ARMIS_BEARER_TOKEN`). This pattern does NOT
    match legitimate bash local variables `DEMO_ORG_SLUG` or `DEMO_ORG_ID` (which lack the
    `_SENSORS` infix and are NOT credential env-vars).
  - `grep -rE '^(export )?[A-Z]+_BEARER_TOKEN=' scripts/ docs/` — targets non-prefixed global
    credential exports (e.g., `ARMIS_BEARER_TOKEN=`). The `export` prefix is optional; the
    leading `^` anchors to line start to avoid matching embedded substrings.
Then: Zero matches for both greps. Every credential env var reference uses the canonical
`PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}` format where `{ID}` is the SCREAMING_SNAKE slug
(e.g., `PRISM_CLIENTS_DEMO_ORG_SENSORS_ARMIS_BEARER_TOKEN`).
Note: Local bash variables `DEMO_ORG_SLUG` and `DEMO_ORG_ID` (used as intermediate script
variables, NOT exported credential env-vars) are explicitly permitted and will NOT be matched
by either grep pattern above.
(traces to BC-2.06.003 §Description: canonical multi-tenant credential convention for Prism;
traces to BC-2.06.003 §Env-Var Name Derivation — `{ID}` = slug SCREAMING_SNAKE transform)

### AC-014 (HIGH-2 — shellcheck in GitHub CI): `shellcheck` runs on `scripts/demo-*.sh` in `.github/workflows/ci.yml`, not only in the Justfile.
Given: The GitHub CI workflow file `.github/workflows/ci.yml` is updated.
When: The CI pipeline runs on any PR or push.
Then: A dedicated step runs `shellcheck scripts/demo-*.sh` (or equivalent) in the CI matrix.
Zero shellcheck errors or warnings are acceptable (same gate as AC-008 — this AC confirms CI
enforcement, AC-008 confirms local enforcement).
(traces to BC-2.22.001 invariant: "boot orchestration is deterministic" — deterministic setup
scripts are a prerequisite; CI gate enforces the invariant across contributors)

### AC-006: Runbook documents troubleshooting for 4 common failure modes
Given: `docs/DEMO-RUNBOOK.md` §Troubleshooting section.
When: An operator reads it.
Then: The section covers at least:
  - (a) "Keyring write failure at credential-set time — E-CRED-004" — explains that
    `prism credential set` surfaces the write failure as E-CRED-004
    (`PrismError::CredentialStoreError`); provides platform-specific fix (macOS Keychain unlock,
    Linux D-Bus start) and the `PRISM_CLIENTS_*` env-var fallback for headless/CI environments.
  - (b) "Keyring read failure at query time — E-CRED-008" — explains that credential resolution
    at query time can fail with E-CRED-008 when the OS keyring is inaccessible; provides
    platform-specific fix and the `PRISM_CLIENTS_*` env-var alternative (Tier-1/2 resolution).
  - (c) "Port already in use" — explains how to kill stale DTU server and re-run demo-run.sh
  - (d) "TOML spec not found" — explains how to verify spec_dir and sensor TOML filenames
(traces to BC-2.06.007: "Missing Required Fields Produce Actionable Error Messages" — runbook
supplements error messages with human-readable remediation steps)

### AC-007: `scripts/demo-teardown.sh` removes generated files and keyring entries
Given: The demo environment is set up and running.
When: `bash scripts/demo-teardown.sh` is executed.
Then: DTU server is killed; the OS keyring entries (all OrgId-keyed entries written by
`prism credential set` for each of 4 sensors / 5 credentials) are deleted via
`prism credential delete` (F-P10-HIGH-001 — using `delete_by_org`, the same OrgId-keyed
namespace as the write path); keyring deletes run BEFORE `~/.config/prism-demo/` is removed
(because `prism credential delete` reads `prism.toml` to resolve the OrgId UUID — the config
dir must still exist at delete time); `~/.config/prism-demo/` is removed; exits 0.
Note: the `--org-slug` flag is optional for single-org configs; `demo-teardown.sh` does NOT
pass `--org-slug` — it relies on single-org auto-resolution (the `delete_by_org` call in
`handle_credential_delete_with_store` resolves the sole org from prism.toml without an explicit
slug). The script reads `DEMO_ORG_ID` directly from prism.toml only to decide whether to
attempt the `prism credential delete` invocations at all (guard condition).
(traces to BC-2.03.005: "Credential CRUD Operations via MCP Tools" — teardown exercises the
delete path of the credential subsystem via `delete_by_org`)

### AC-008: All shell scripts pass `shellcheck` with no errors or warnings (local gate)
Given: The 3 shell scripts (`demo-setup.sh`, `demo-run.sh`, `demo-teardown.sh`) are committed.
When: `shellcheck scripts/demo-*.sh` is executed locally (`just check-ci` or directly).
Then: Zero errors, zero warnings.
(traces to BC-2.22.001 invariant: "boot orchestration is deterministic"; see AC-014 for CI gate)

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| `prism credential set` value MUST come from stdin, not CLI arg | AD-017 AI-opaque credential model | `clap` arg for `--value` is FORBIDDEN; stdin read is mandatory |
| Write path: `CredentialStoreOrgId::set_by_org(&org_id, sensor, &name, value)` — NOT `CredentialStore::set` | ADR-034 §D3 / BC-2.06.003 Tier-3 | The legacy slug-keyed `CredentialStore::set` path MUST NOT be called from `credential_cli.rs` |
| OrgId resolution: load `PrismConfig.orgs[n].org_id` from prism.toml; map slug → UUID | ADR-034 §D3 | `credential_cli.rs` reads prism.toml via the boot-step-2 config path; no `OrgRegistry` import in `prism-credentials` (architecture compliance rule in `trait_.rs:84–85`) |
| `resolve_credential` new signature: `(client_id, sensor_id, cred_name, org_id: Option<&OrgId>, keyring: Option<&Arc<dyn CredentialStoreOrgId>>)` | ADR-034 §D1 + D2 | All callers in prism-spec-engine must pass both parameters; callers without Tier-3 may pass `None` |
| Slug→OrgId resolution in `PrismCredentialResolver` (in `prism-spec-engine`) — NOT inside `prism-credentials` | `crates/prism-credentials/src/trait_.rs:84–85` architecture compliance rule | `prism-credentials` MUST NOT import `OrgRegistry`; violation = compile error |
| `PrismCredentialResolver` is a struct with `org_registry: Arc<OrgRegistry>` and `keyring: Arc<dyn CredentialStoreOrgId>` | ADR-034 §D1 | Unit-struct form is removed; `PrismCredentialResolver::new(org_registry, keyring)` is the only constructor |
| `BootContext` gains `credential_store_org_id: Arc<dyn CredentialStoreOrgId>` alongside existing `credential_store: Arc<dyn CredentialStore>` | ADR-034 §D5 | Step 5 exposes `Arc<KeyringBackend>` via both traits; same instance, no state duplication |
| Boot probe must use OrgId-keyed probe (Tier 3a, `get_by_org` per registered org) as PRIMARY keyring check; legacy `{sensor_id}/{ref_name}` is Tier 3b FALLBACK only | BC-2.06.003 v1.9 §OrgRegistry and KeyringStore Threading | `KeyringCredentialProbe` gains `keyring: Arc<dyn CredentialStoreOrgId>` field; `step5_init_credential_store` passes the shared `Arc<KeyringBackend>` at construction. `CredentialRefProbe::probe` was converted to `async` (via `#[async_trait]`) in pass-14 (commit 0941c0e0) — this IS a method-signature change; all 5 impls and all call sites were updated. The `org_registry: &OrgRegistry` param was already present from v1.3 (unchanged). See BC-2.06.003 v1.9 §OrgRegistry and KeyringStore Threading. |
| Tier-3 error: keyring backend error → hard `BackendUnavailable { detail: "E-CRED-008: OS keyring unavailable: {reason}" }` — do NOT fall through | ADR-034 §D4 / ADR-035 §D5 / BC-2.06.003 / SOUL.md §4 | `reason` from keyring-rs is a system error string; must never contain a credential value |
| `rpassword` or equivalent for no-echo stdin read | BC-2.03.007 Secret Redaction | `read -s` in bash is acceptable for shell scripts; Rust CLI must use `rpassword` crate |
| Real OS keyring backends MUST be feature-enabled in `prism-credentials` (F-P10-CRIT-001) | Cross-process credential visibility | keyring-rs silently falls back to in-process mock when no backend feature is enabled; 4 `compile_error!` guards in `crates/prism-credentials/src/lib.rs` prevent silent reversion (apple-native on macOS/iOS, windows-native on Windows, linux-native-sync-persistent or linux-native on Linux). Removing a backend feature without removing the corresponding pass-through in `[features]` trips the guard on next build. |
| Tests use keyring mock-builder override (`install_keyring_mock()` in `crates/prism-credentials/src/tests/mod.rs`) + `InMemoryCredentialStore` injection — NEVER the real OS Keychain | macOS unsigned-test-binary ACL constraint + SID-1 compliance | `just check` must never touch the real macOS Keychain; real-keyring cross-process tests are `#[ignore]`'d per SID-1 §4 with blocking-dependency rationale comments. |
| `prism credential delete` MUST use `delete_by_org` (OrgId-keyed namespace) — NOT platform-native CLI tools | F-P10-HIGH-001 namespace match | Platform-native CLI tools (macOS `security delete-generic-password`, Linux `secret-tool`) use the `account` attribute which does NOT match the OrgId-keyed namespace written by `set_by_org`; they silently fail to delete the correct entry. `prism credential delete --org-slug <slug> --sensor <s> --name <n>` calls `CredentialStoreOrgId::delete_by_org` — exact same namespace. |
| Keyring deletes in `demo-teardown.sh` MUST run BEFORE `rm -rf` of config dir | `prism credential delete` reads prism.toml for OrgId resolution | If config dir is removed first, `prism.toml` is unavailable → OrgId lookup fails → credential deletes fail silently. Script ordering: kill DTU → delete keyring entries → remove config dir. |
| All demo env vars use `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}` format | BC-2.06.003 §Env-Var Name Derivation | Grep gate: (1) zero matches for `DEMO_ORG_[A-Z_]+_SENSORS` (retired credential form; does NOT match bare `DEMO_ORG_SLUG`/`DEMO_ORG_ID` local vars); (2) zero matches for `^(export )?[A-Z]+_BEARER_TOKEN=` (non-prefixed global credential exports) — in scripts/ and docs/ |
| `demo-setup.sh` MUST NOT write per-org `base_url` overlay TOMLs | DTU ports are ephemeral (only known post-launch) | Overlay generation belongs exclusively in `demo-run.sh`, after `urls.json` is parsed; `demo-setup.sh` writing a `base_url` overlay with a hardcoded port is a Forbidden Dependencies violation (see §Forbidden Dependencies: "Hardcoded port numbers in setup scripts") |
| `demo-run.sh` MUST export TYPE-spec env vars AND write per-org `base_url` overlay TOMLs before launching `prism-bin` | AC-009 two-part data-path precondition | **(a) Env vars (step-4a boot gate):** Export `CROWDSTRIKE_BASE_URL=http://127.0.0.1`, `ARMIS_INSTANCE_URL=http://127.0.0.1`, `CLAROTY_INSTANCE_URL=http://127.0.0.1`, `CYBERINT_ENVIRONMENT=demo` in the `prism start` command environment — without these, step-4a env_resolver.rs fires E-SPEC-024 and boot.rs step-4b hard-aborts before overlays are reached. **(b) Overlays (step-4c port override):** For each sensor, write `specs/customers/demo-org/<sensor>.sensor.toml` with `extends = "<sensor>"` and `base_url = "http://127.0.0.1:<PORT>"` (port from `urls.json`). **(c) SEC-003 (crowdstrike only):** The `allowed_urls` list in `crowdstrike-oauth2.manifest.toml` (written by `demo-setup.sh`) covers the CrowdStrike OAuth2 plugin token endpoint — this is a plugin host-function gate, not a per-org egress allowlist. Armis/Claroty/Cyberint use plain `reqwest::Client` with no host gating. |
| Shell scripts use `#!/usr/bin/env bash` shebang | Portability | Required by shellcheck |
| crowdstrike-oauth2.prx path must be validated | Risk mitigation | Script checks file exists before copying; exits 1 with actionable message if not found |
| `--org-slug` required when `config.orgs.len() > 1`; error clearly on missing/invalid prism.toml | ADR-034 §D3 HIGH-3 | `resolve_org_slug_and_id` MUST NOT return `"demo-org"` as a silent default; SOUL.md §4 |
| Runbook must not contain real credential values | AD-017 | Use placeholder strings `"<your-client-id>"` in examples; DTU demo uses `"demo-client"` only |

---

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `clap` | workspace version | New `credential set` subcommand (derive mode, matches existing CLI pattern) |
| `rpassword` | `7.*` | Prompt for credential value with terminal echo disabled |
| `prism-credentials` | workspace path | `CredentialStoreOrgId::set_by_org()` for OrgId-keyed keyring write; `resolve_credential` (updated signature) |
| `prism-spec-engine` | workspace path | `PrismCredentialResolver` (struct with fields); `StaticCookieAuthProvider`; `PluginAuthProvider` |
| `shellcheck` | any stable | Shell script linting in local CI (`just check-ci`) and GitHub CI (`ci.yml`) |

Note: `rpassword` version should be checked against `crates.io` at implementation time if not
already in workspace `Cargo.toml`. If it is not present, add it as a prism-bin dep.

---

## File Structure Requirements

| File | Action | Crate | Purpose |
|------|--------|-------|---------|
| `crates/prism-credentials/src/resolution.rs` | MODIFY | prism-credentials | Add Tier-3 branch between env-var resolution and Tier-4 CRUD lookup; update signature (2 new params: `org_id: Option<&OrgId>`, `keyring: Option<&Arc<dyn CredentialStoreOrgId>>`) |
| `crates/prism-credentials/src/lib.rs` | MODIFY | prism-credentials | Re-export `resolve_credential` with updated signature |
| `crates/prism-credentials/tests/bc_2_06_003_tier3_keyring_resolution.rs` | CREATE | prism-credentials | RG-034-001 (end-to-end write→resolve), RG-034-002 (miss→Tier-4 fallthrough) |
| `crates/prism-spec-engine/src/auth_provider.rs` | MODIFY | prism-spec-engine | `PrismCredentialResolver` → struct with `org_registry + keyring` fields; update `new()`; update 3 test double impls (`MockCredentialResolver`, `NotFoundCredentialResolver`, `BackendUnavailableCredentialResolver`); update `StaticCookieAuthProvider::new()` to accept `Arc<OrgRegistry>` + `Arc<dyn CredentialStoreOrgId>` |
| `crates/prism-spec-engine/src/plugin_auth_provider.rs` | MODIFY | prism-spec-engine | `PluginAuthProvider` gains `org_registry: Arc<OrgRegistry>` + `keyring: Arc<dyn CredentialStoreOrgId>`; update `new()`; update 2 `prism_credentials::resolve_credential` callsites inside `PluginAuthProvider::acquire_token` (both pass `org_id.as_ref()` + `Some(&self.keyring)` per ADR-034 §D1) |
| `crates/prism-bin/src/boot.rs` | MODIFY | prism-bin | `BootContext` gains `credential_store_org_id: Arc<dyn CredentialStoreOrgId>`; step 5 exposes `Arc<KeyringBackend>` alongside `Arc<dyn CredentialStore>` |
| `crates/prism-bin/src/spec_driven_adapter.rs` | MODIFY | prism-bin | Auth provider construction sites (step 9A) gain `Arc::clone(&ctx.org_registry)` + `Arc::clone(&ctx.credential_store_org_id)` parameters for `PrismCredentialResolver::new` |
| `crates/prism-credentials/Cargo.toml` | MODIFY | prism-credentials | F-P10-CRIT-001: enable real OS keyring backends — pass-through features `keyring-apple-native`, `keyring-windows-native`, `keyring-linux-native-sync-persistent`, `keyring-linux-native` declared in `[features]` default; `[dependencies].keyring` updated with corresponding `apple-native`, `windows-native`, `linux-native-sync-persistent`/`crypto-rust`, `linux-native` feature flags |
| `crates/prism-credentials/src/lib.rs` | MODIFY | prism-credentials | F-P10-CRIT-001: 4 `compile_error!` regression guards (per-OS `cfg` checks) added at crate root — prevent silent reversion to in-process mock keystore when backend features are absent |
| `crates/prism-credentials/src/tests/mod.rs` | CREATE | prism-credentials | `install_keyring_mock()` function (keyring-rs mock-builder override) + serializing Mutex for concurrent test safety — allows in-process tests to use mock keystore without real OS Keychain (SID-1 compliance) |
| `crates/prism-credentials/src/tests/store_tests.rs` | CREATE | prism-credentials | In-process unit tests for KeyringBackend that use `install_keyring_mock()` — exercises `set_by_org`, `get_by_org`, `delete_by_org` without real OS Keychain |
| `crates/prism-bin/src/credential_cli.rs` | CREATE | prism-bin | `CredentialArgs` struct + `handle_credential_set()` (OrgId-keyed write via `CredentialStoreOrgId::set_by_org`; prism.toml load; HIGH-3 error on missing toml) + `CredentialDeleteArgs` struct + `handle_credential_delete()` / `handle_credential_delete_with_store()` (F-P10-HIGH-001: `delete_by_org`; idempotent; exits 0/1/2) |
| `crates/prism-bin/tests/bc_2_03_007_credential_set_org_id_keyed.rs` | CREATE | prism-bin | RG-034-004: OrgId-keyed write + CRIT-2 regression (entry NOT under slug-keyed namespace) |
| `scripts/demo-setup.sh` | CREATE | — | Idempotent one-time pre-launch setup: build → mkdir → copy specs → copy plugin → write `prism.toml` → set credentials via `prism credential set` (OrgId-keyed) → write `crowdstrike-oauth2.manifest.toml` (includes `allowed_urls = ["api.crowdstrike.com", "127.0.0.1"]` for the CrowdStrike OAuth2 plugin token endpoint — SEC-003 plugin host-function gate validates hostnames only (no scheme prefix); NOT a per-org egress allowlist; Armis/Claroty/Cyberint have no equivalent gate) → print instructions. Does NOT write per-org `base_url` overlay TOMLs (DTU ports are not yet known; overlays are written by `demo-run.sh` post-launch). Does NOT export TYPE-spec env vars (those are written by `demo-run.sh` into the `prism start` command). |
| `scripts/demo-run.sh` | CREATE | — | Daily launch: (1) start DTU in background → poll `urls.json` → parse ephemeral ports; (2) **export TYPE-spec env vars** (`CROWDSTRIKE_BASE_URL=http://127.0.0.1`, `ARMIS_INSTANCE_URL=http://127.0.0.1`, `CLAROTY_INSTANCE_URL=http://127.0.0.1`, `CYBERINT_ENVIRONMENT=demo`) — required by step-4a `env_resolver.rs` to satisfy `${env.*}` placeholders in TYPE-level sensor specs; without these, boot fires E-SPEC-024 and aborts before step-4c overlays are reached; (3) **write per-org `base_url` overlay TOMLs** (`specs/customers/demo-org/<sensor>.sensor.toml` with `extends` + `base_url=http://127.0.0.1:<PORT>`) for all 4 sensors (step-4c port override, done HERE because DTU ports are ephemeral); (4) print ports → print `prism start` command (with env vars pre-populated). There is NO per-org/host egress allowlist for Armis/Claroty/Cyberint — they use plain `reqwest::Client`; only the CrowdStrike OAuth2 plugin has an `allowed_urls` gate (SEC-003), already handled by `demo-setup.sh`'s `crowdstrike-oauth2.manifest.toml`. |
| `scripts/demo-teardown.sh` | CREATE | — | Cleanup (F-P10-HIGH-001 fix): kill DTU → delete OrgId-keyed keyring entries via `prism credential delete` on ALL platforms (macOS + Linux + Windows) → remove config dir. ORDERING: keyring deletes BEFORE `rm -rf` (prism.toml must be present for OrgId resolution). Previous platform-native CLI approach (`security delete-generic-password` / `secret-tool`) replaced — those tools used wrong namespace attributes and silently orphaned entries. |
| `scripts/demo.toml` | CREATE | — | DTU demo server config (all 4 sensors, ephemeral ports) |
| `docs/DEMO-RUNBOOK.md` | CREATE | — | Comprehensive operator runbook (7 sections per scope); `PRISM_CLIENTS_*` format only; references E-CRED-008 in Troubleshooting |
| `.github/workflows/ci.yml` | MODIFY | — | Add shellcheck step for `scripts/demo-*.sh` (HIGH-2 remediation; CI gate separate from local `just check-ci`) |

---

## `prism credential` Subcommand Specification

The `prism credential` subcommand group exposes two sub-subcommands. Both follow the existing
`prism start` pattern.

### `prism credential set`

```
USAGE:
    prism credential set --sensor <SENSOR_ID> --name <CREDENTIAL_NAME> [--org-slug <ORG_SLUG>]

ARGS:
    --sensor <SENSOR_ID>           Sensor ID (e.g., crowdstrike, armis, claroty, cyberint)
    --name <CREDENTIAL_NAME>       Credential name (e.g., client_id, client_secret, bearer_token)
    --org-slug <ORG_SLUG>          Org slug — required when prism.toml has >1 org; optional for single-org configs

BEHAVIOR:
    Loads PrismConfig from config_dir/prism.toml.
    Resolves org slug → OrgId UUID from PrismConfig.orgs[n].org_id.
    If --org-slug absent and len(orgs) > 1: exits 2 with error "Multiple orgs configured in '<path>' — use --org-slug <slug> to select one. Configured orgs: [<slugs>]".
    If --org-slug absent and len(orgs) == 1: use single org's org_id.
    If prism.toml missing or unparseable: hard error (no demo-org fallback).
    Prompts "Enter value for prism/<sensor>/<name>: " on stderr.
    Reads value from stdin with terminal echo disabled (rpassword).
    Writes to OS keyring via CredentialStoreOrgId::set_by_org(&org_id, sensor, &name, value).
    Keyring key format: "{org_id_uuid}/{sensor}/{name}" (namespace_key_by_org_id).
    Prints "Credential stored successfully." to stdout on success.
    Exits 0 on success; exits 1 (EXIT_GENERIC_ERROR) on keyring write failure with actionable error on stderr;
    exits 2 (EXIT_CONFIG_INVALID) on config-invalid (prism.toml missing / unparseable / org not found).
```

The `--value` flag is explicitly FORBIDDEN (AD-017 compliance).

### `prism credential delete` (F-P10-HIGH-001)

```
USAGE:
    prism credential delete --sensor <SENSOR_ID> --name <CREDENTIAL_NAME> [--org-slug <ORG_SLUG>]

ARGS:
    --sensor <SENSOR_ID>           Sensor ID
    --name <CREDENTIAL_NAME>       Credential name
    --org-slug <ORG_SLUG>          Org slug — same resolution logic as `set`

BEHAVIOR:
    Loads PrismConfig from config_dir/prism.toml (same as `set` — reads OrgId UUID).
    Resolves org slug → OrgId UUID.
    Deletes the entry at namespace key "{org_id_uuid}/{sensor}/{name}" via
      CredentialStoreOrgId::delete_by_org(&org_id, sensor, &cred_name).
    Exits 0 if deleted OR already absent (idempotent — teardown scripts may call delete
      even when entries were never written or were previously removed).
    Exits 1 on keyring backend error (actionable error on stderr).
    Exits 2 on config-invalid (prism.toml missing or OrgId resolution failure).
```

The `delete` subcommand is implemented in `handle_credential_delete` /
`handle_credential_delete_with_store` in `crates/prism-bin/src/credential_cli.rs`. The inner
`_with_store` variant accepts an injected `Arc<dyn CredentialStoreOrgId>` for testability (SID-1).
`demo-teardown.sh` uses `prism credential delete` on ALL platforms (macOS + Linux + Windows),
replacing the prior platform-native CLI approach (`security delete-generic-password` / `secret-tool`)
which used wrong namespace attributes and silently orphaned keyring entries.

---

## `docs/DEMO-RUNBOOK.md` Required Sections

The runbook must contain exactly these sections (in this order):

1. **Prerequisites** — Rust toolchain, `just`, `shellcheck`, `cargo nextest`
2. **One-time Setup** — run `demo-setup.sh`; what it does step-by-step; credential format used (`PRISM_CLIENTS_*` / `prism credential set`)
3. **Daily Demo Run** — run `demo-run.sh`; verify DTU clones are up
4. **Connecting Claude Code** — add prism-bin to `~/.claude/settings.json` MCP servers section; exact JSON snippet
5. **Example Queries** — one query per sensor (CrowdStrike, Armis, Claroty, Cyberint) with expected output shape
6. **Troubleshooting** — 4 failure modes per AC-006: §6(a) keyring write / E-CRED-004, §6(b) keyring read / E-CRED-008, §6(c) port already in use, §6(d) TOML spec not found; NO `DEMO_ORG_*` env var format in examples
7. **Cleanup** — run `demo-teardown.sh`

---

## Red Gate Tests

The following tests MUST be written as failing Red Gates before any implementation. Tests marked
(RG-034-NNN) are specified in ADR-034 §Red Gate Tests and are authoritative.

| Test Name | File | Gate | Coverage |
|-----------|------|------|----------|
| `test_BC_2_06_001_demo_setup_generates_valid_prism_toml` | `crates/prism-bin/tests/` or `src/credential_cli.rs #[cfg(test)]` | AC-001 | Generated prism.toml is schema-valid |
| `test_BC_2_03_007_prism_credential_set_does_not_echo_value_to_stdout` | `crates/prism-bin/tests/` | AC-005 | Value NOT in stdout; stdin-prompt path |
| `test_BC_2_06_003_tier3_credential_written_by_set_by_org_is_resolved` (RG-034-001) | `crates/prism-credentials/tests/bc_2_06_003_tier3_keyring_resolution.rs` | AC-009 | CRIT-1 gap closure: write→resolve end-to-end |
| `test_BC_2_06_003_tier3_miss_falls_through_to_tier4` (RG-034-002) | `crates/prism-credentials/tests/bc_2_06_003_tier3_keyring_resolution.rs` | AC-011 Case A | Tier-3 miss → Tier-4 (not BackendUnavailable) |
| `test_resolve_org_slug_errors_when_toml_missing_and_no_explicit_slug` (RG-034-003) | `crates/prism-bin/src/credential_cli.rs #[cfg(test)] mod tests` | AC-012 | HIGH-3: no demo-org fallback |
| `test_handle_credential_set_writes_org_id_keyed_namespace` (RG-034-004) | `crates/prism-bin/tests/bc_2_03_007_credential_set_org_id_keyed.rs` | AC-010 / AC-005 | CRIT-2 regression: entry at OrgId-keyed key; NOT at slug-keyed key |
| `test_BC_2_06_003_tier3_backend_error_returns_e_cred_008` (RG-034-005) | `crates/prism-credentials/tests/bc_2_06_003_tier3_keyring_resolution.rs` | AC-011 Case B | Keyring backend `Err` → hard `BackendUnavailable`/E-CRED-008; no Tier-4 fall-through; no credential-value leak in detail; uses `InMemoryCredentialStore` error-injection mode (test-helpers-gated) |
| `test_handle_credential_delete_uses_org_id_keyed_namespace` (F-P10-HIGH-001) | `crates/prism-bin/tests/bc_2_03_007_credential_set_org_id_keyed.rs` | AC-007 | F-P10-HIGH-001: `handle_credential_delete_with_store` calls `delete_by_org` (OrgId-keyed); entry absent after delete; idempotent second delete returns exit 0; uses `InMemoryCredentialStore` injection (no real OS Keychain) — BC-2.03.005 delete path / ADR-034 §D3 |
| `test_BC_2_06_003_boot_probe_tier3a_finds_org_id_keyed_credential` (TV-BOOT-P-001) | `crates/prism-bin/src/boot.rs` `#[cfg(test)] mod tests` | AC-002 / AC-009 / BC-2.06.003 v1.8 | F-P14-CRIT-001 fix: creates `KeyringCredentialProbe` with `InMemoryCredentialStore`, calls `set_by_org` to write an OrgId-keyed credential, then calls `probe(sensor_id, ref_name, &org_registry)` — asserts probe returns `Ok(None)`. Validates that the boot probe finds credentials written by `prism credential set` (OrgId-keyed, Tier 3a). In-process unit test; no real OS Keychain. |

Note: The existing `test_BC_2_03_007_prism_credential_set_does_not_echo_value_to_stdout` test may
require an OrgId-keyed variant per ADR-034 (the write path changes from `CredentialStore::set` to
`CredentialStoreOrgId::set_by_org`). The test-writer must review the existing test stub (if any)
and produce the OrgId-keyed variant in `bc_2_03_007_credential_set_org_id_keyed.rs`.

---

## Tasks

1. **Read** `crates/prism-bin/src/main.rs` — understand existing Clap CLI structure before adding new subcommand.
2. **Read** `crates/prism-credentials/src/resolution.rs` — understand current Tier-1/2/4 chain, existing signature, and the two "not implemented" comments at lines 18, 92.
3. **Read** `crates/prism-credentials/src/keyring.rs` lines 248–285 — understand `CredentialStoreOrgId::get_by_org` and `spawn_blocking` encapsulation.
4. **Read** `crates/prism-credentials/src/namespace.rs` — understand `namespace_key_by_org_id` format (`"{org_id_uuid}/{sensor}/{name}"`).
5. **Read** `crates/prism-spec-engine/src/auth_provider.rs` — identify all 3 test doubles implementing `CredentialResolver` (`MockCredentialResolver`, `NotFoundCredentialResolver`, `BackendUnavailableCredentialResolver`); identify `PrismCredentialResolver` current struct form.
6. **Read** `crates/prism-spec-engine/src/plugin_auth_provider.rs` — identify 2 `prism_credentials::resolve_credential` callsites inside `PluginAuthProvider::acquire_token` (the `resolved_client_id` and `resolved_client_secret` bindings).
7. **Read** `crates/prism-bin/src/boot.rs` — understand `BootContext` struct and step-5 credential store initialization.
8. **Read** `crates/prism-bin/src/spec_driven_adapter.rs` — identify step-9A auth provider construction sites.
9. **Find** `crowdstrike-oauth2.prx` committed path (S-PLUGIN-CI-001 merged it; verify path from git tree before scripting).
10. **Write Red Gate tests** for all 9 tests listed in §Red Gate Tests. All must FAIL before implementation.
11. **Implement** `resolve_credential` Tier-3 branch in `crates/prism-credentials/src/resolution.rs` with updated signature.
12. **Update** `crates/prism-credentials/src/lib.rs` re-export to match new signature.
13. **Update** `crates/prism-spec-engine/src/auth_provider.rs`: `PrismCredentialResolver` → struct with fields; update `new()`; sibling-sweep all 3 test doubles (`MockCredentialResolver`, `NotFoundCredentialResolver`, `BackendUnavailableCredentialResolver`).
14. **Update** `crates/prism-spec-engine/src/plugin_auth_provider.rs`: add `org_registry` + `keyring` fields; update 2 `resolve_credential` callsites.
15. **Update** `crates/prism-bin/src/boot.rs`: `BootContext.credential_store_org_id`; step-5 `Arc<KeyringBackend>` exposure.
16. **Update** `crates/prism-bin/src/spec_driven_adapter.rs`: step-9A construction sites gain 2 new params.
17. **Implement** `crates/prism-bin/src/credential_cli.rs`: `CredentialArgs` + `handle_credential_set()` with OrgId-keyed write + HIGH-3 error handling.
17a. **Implement** `handle_credential_delete` / `handle_credential_delete_with_store` in `credential_cli.rs` (F-P10-HIGH-001): `CredentialDeleteArgs` struct; `delete_by_org` dispatch; idempotent `Ok(false)` path exits 0; backend error exits 1; config-invalid exits 2.
18. **Add** `Credential(CredentialArgs)` variant and `Delete(CredentialDeleteArgs)` subcommand to CLI enum in `main.rs` / `cli.rs`.
19. **Write** `scripts/demo-setup.sh` — uses `prism credential set` for credential bootstrap; ONLY `PRISM_CLIENTS_*` format in any env var references; idempotent.
20. **Write** `scripts/demo-run.sh` — launch DTU in background → poll urls.json → print ports.
21. **Write** `scripts/demo-teardown.sh` — kill DTU → remove config dir → delete OrgId-keyed keyring entries.
22. **Write** `scripts/demo.toml` — DTU demo server config for all 4 sensors.
23. **Write** `docs/DEMO-RUNBOOK.md` — all 7 sections; references E-CRED-008 in Troubleshooting; no `DEMO_ORG_*` format.
24. **Modify** `.github/workflows/ci.yml` — add shellcheck step for `scripts/demo-*.sh` (HIGH-2).
25. **Run** `shellcheck scripts/demo-*.sh` — fix all warnings (AC-008).
26. **TD-VSDD-060 sibling-site sweep** — grep `resolve_credential` across ALL crates; verify every callsite updated.
27. **Run** Red Gate tests: `just iter prism-credentials` and `just iter prism-bin` — all 9 must pass GREEN.
28. **Run** `just check` — final pre-push gate.

---

## Previous Story Intelligence

- **S-DEMO-001** (depends_on): The `boot.step9a.adapter_registry_populated` event with `sensor_count=4` is the boot-success signal that AC-002 validates. Read S-DEMO-001 for the event field names.
- **S-DEMO-002** (depends_on): The E2E smoke test's `bootstrap_credentials()` helper uses the per-client env-var format (`PRISM_CLIENTS_*`). If S-DEMO-002 tests set env vars, they use Tier-2. This story adds Tier-3 as the demo bootstrap; the two tiers coexist.
- **S-CONFIG-MULTI-TENANT-OVERRIDE-001** (already merged): The per-org overlay format (`customers/demo-org/crowdstrike.sensor.toml` with `extends = "crowdstrike"` and `base_url = "http://127.0.0.1:<PORT>"`) is documented in that story. **`demo-run.sh` generates these per-org `base_url` overlay TOMLs** (`specs/customers/demo-org/<sensor>.sensor.toml`) AFTER launching the DTU server and reading the ephemeral ports from `urls.json`; overlay generation cannot happen earlier because DTU ports are not known until post-launch. `demo-setup.sh` handles pre-launch config (directory structure, prism.toml, credential bootstrap via `prism credential set`) and does NOT write `base_url` overlays.
- **S-PLUGIN-CI-001** (merged): Committed `crowdstrike-oauth2.prx`. The demo-setup.sh script copies this file to the plugin dir. Implementer must find the committed path by reading that story's demo evidence or the git tree.
- **ADR-034 (new)**: The authoritative source for all implementation decisions in this story. Read it in full before writing any code. The architecture compliance rule at `crates/prism-credentials/src/trait_.rs:84–85` (prism-credentials must NOT import OrgRegistry) is the design constraint that makes `PrismCredentialResolver` the right DI boundary.

### OBS [process-gap] — `scripts/start-demo.sh` launcher overlap (tracked follow-up; out of scope here)

A pre-existing `scripts/start-demo.sh` convenience launcher (6 DTU clone processes, hardcoded ports)
overlaps confusingly with this story's `demo-run.sh` (4 DTU clones, ephemeral ports). The two
scripts serve different purposes and use different port models, but the naming is ambiguous. Additionally,
`start-demo.sh` is NOT covered by this story's `scripts/demo-*.sh` shellcheck glob (AC-008 and
AC-014), creating a lint gap.

**Follow-up story:** `S-DEMO-LAUNCHER-CONSOLIDATION-001` (draft) — reconcile `start-demo.sh` and
`demo-run.sh` into a single launcher entry point; add `start-demo.sh` to shellcheck glob or retire it.
This story (S-DEMO-003) does NOT modify `start-demo.sh`; the consolidation is deferred explicitly
pending human prioritization of S-DEMO-LAUNCHER-CONSOLIDATION-001.

---

## Open Questions

1. **Plugin artifact committed path**: S-PLUGIN-CI-001 committed `crowdstrike-oauth2.prx` but the story-writer does not know the exact path. Implementer must verify before writing `demo-setup.sh` copy command.

2. **`rpassword` in workspace**: Is `rpassword` already in `Cargo.toml` workspace deps? If not, add it as a prism-bin dep.

3. **`prism credential set` for non-CrowdStrike sensors**: Armis uses `bearer_static` auth with `bearer_token` as the credential name. Claroty uses `bearer_static` auth with `bearer_token` as the credential name. Cyberint uses `cookie_roundtrip` auth with `api_key` as the credential name. CrowdStrike uses `oauth2_client_credentials` auth (via the crowdstrike-oauth2 WASM plugin) with `client_id` and `client_secret`. The `demo-setup.sh` calls `prism credential set` once per (sensor, credential_name) pair — 5 invocations total (crowdstrike/client_id, crowdstrike/client_secret, armis/bearer_token, claroty/bearer_token, cyberint/api_key). These auth_type and credential name values are D-747 LOCKED per `crates/prism-sensors/specs/*.sensor.toml`.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | OS keyring not available (e.g., headless CI without keyring service) — **write path** (`prism credential set`): keyring write fails | `prism credential set` exits 1; `handle_credential_set` surfaces the keyring write failure via "Keyring unavailable: {e}…" (E-CRED-004 — `PrismError::CredentialStoreError`; credential_cli.rs handle_credential_set). Operator is directed to set the per-client env var fallback (`PRISM_CLIENTS_<ORG>_SENSORS_<SENSOR>_<REF>`) as an alternative to the OS keyring. |
| EC-001b | OS keyring not available — **read path** (`resolve_credential` / Tier-3): keyring read returns `Err(NoStorageAccess)` | `resolve_credential` returns `CredentialResolutionError::BackendUnavailable { detail: "E-CRED-008: OS keyring unavailable: {reason}" }`. Hard error — does NOT fall through to Tier 4 (ADR-034 §D4 / ADR-035 §D5). The detail string contains the keyring-rs system error string only; no credential value (AD-017). |
| EC-002 | `demo-setup.sh` run twice in succession | Second run is idempotent: overwrite files, overwrite keyring entries (`set_by_org` overwrites); no error |
| EC-003 | crowdstrike-oauth2.prx not found at expected path | `demo-setup.sh` exits 1 with: "ERROR: Plugin artifact not found at <path>" followed by "Run: cargo build -p prism-spec-engine --features wasm-plugins" and "Then re-run this script." (three separate stderr lines, then `exit 1`). |
| EC-004 | DTU server not started before demo-run.sh | `demo-run.sh` polls urls.json for 30s then exits 1 with: "ERROR: DTU server did not start within 30s. Check <run_dir>/dtu-server.log for details. Common cause: port conflict — stop other services on the demo ports." |
| EC-005 | `prism credential set` called with `--value` flag (attempted AD-017 bypass) | Clap rejects: "error: unexpected argument '--value' found. Values must be provided interactively." |
| EC-006 | `--org-slug` provided but slug not found in prism.toml `[[orgs]]` | `handle_credential_set` exits 2 (EXIT_CONFIG_INVALID): "--org-slug '<slug>' not found in prism.toml '<path>'. Configured orgs: [<slugs>]" (resolve_org_slug_and_id returns Err; handle_credential_set_with_store returns EXIT_CONFIG_INVALID). |
| EC-007 | `org_id: None` passed to `resolve_credential` (caller lacks Tier-3 capability) | Tier 3 skipped silently; falls through to Tier 4 (BC-2.06.003 Tier-3 postcondition row 1) |
| EC-008 | Keyring backend panics inside `spawn_blocking` | `KeyringBackend::get_by_org` catches spawn panic; `resolve_credential` receives `Err(...)` → hard `BackendUnavailable` / E-CRED-008 (ADR-034 §D4 / ADR-035 §D5) |

---

## Token Budget Estimate

| Context source | Estimated tokens |
|----------------|-----------------|
| This story spec | ~8,000 |
| BC files (5 BCs: BC-2.03.005, BC-2.03.007, BC-2.06.001, BC-2.06.003, BC-2.22.001) | ~9,000 |
| ADR-034 (Tier-3 decision authority) | ~4,000 |
| `crates/prism-credentials/src/resolution.rs` | ~3,000 |
| `crates/prism-credentials/src/keyring.rs` + `namespace.rs` | ~3,000 |
| `crates/prism-credentials/src/lib.rs` (compile_error! guards — F-P10-CRIT-001) | ~1,000 |
| `crates/prism-spec-engine/src/auth_provider.rs` | ~4,000 |
| `crates/prism-spec-engine/src/plugin_auth_provider.rs` | ~2,000 |
| `crates/prism-bin/src/boot.rs` + `spec_driven_adapter.rs` | ~5,000 |
| `crates/prism-bin/src/main.rs` + `credential_cli.rs` (set + delete) | ~4,000 |
| `crates/prism-sensors/specs/` (4 TOML files) | ~3,000 |
| S-DEMO-001 + S-DEMO-002 (dependency context) | ~5,000 |
| **Total estimate** | **~51,000 tokens (~20% of 256K context)** |

Within the 20-30% agent context limit. Increased from ~47K (v1.9) by ~4K due to F-P10-CRIT-001
(compile_error! guards + keyring Cargo.toml) and F-P10-HIGH-001 (credential delete subcommand)
additions. Still within limit.

---

## Forbidden Dependencies

| Forbidden | Reason |
|-----------|--------|
| CLI `--value` argument for credential value | AD-017 — value must come from stdin, not args |
| `CredentialStore::set` (slug-keyed) in `credential_cli.rs` | CRIT-2: slug-keyed write is permanently invisible to Tier-3 OrgId-keyed read |
| `OrgRegistry` import in `prism-credentials` | Architecture compliance rule `trait_.rs:84–85` — callers pre-resolve slug→OrgId |
| Silent `"demo-org"` default in `resolve_org_slug_and_id` | SOUL.md §4 swallow-error prohibition; ADR-034 §D3 HIGH-3 |
| `DEMO_ORG_*` or global `{SENSOR}_{REF}` format in scripts/docs | BC-2.06.003 — canonical format is `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}` |
| Hardcoded port numbers in setup scripts | DTU binds to ephemeral ports; always read from urls.json |
| `echo` or `printf` of credential value to any file descriptor | BC-2.03.007 Secret Redaction |
| Credential values in `docs/DEMO-RUNBOOK.md` examples | AD-017 — use placeholder strings only |
| `security delete-generic-password` / `secret-tool` for keyring deletion in demo scripts | F-P10-HIGH-001: platform-native tools use the `account` attribute which does NOT match the OrgId-keyed namespace from `set_by_org` — they silently orphan entries. Use `prism credential delete` exclusively. |
| Zero keyring backend features in `crates/prism-credentials/Cargo.toml` | F-P10-CRIT-001: zero features → in-process mock keystore → cross-process credential writes invisible; 4 `compile_error!` guards prevent this pattern from reaching the build. |
| Calling `prism-credentials` from `prism-spec-engine` across the wrong direction | Dependency direction: prism-spec-engine depends on prism-credentials; not the reverse |

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.18 | 2026-08-02 | story-writer | Added ## Authority section (DRIFT-STORY-AUTHORITY-ABSENT-CORPUS-001 Round 6, D-2084). Synced stale `**Version:**` pseudo-field from v1.17 to v1.18 to match frontmatter (TD-VSDD-060 sibling-sweep correction, orchestrator-authorized). |
| 1.17 | 2026-06-08 | state-manager | **MERGED — PR #176 squash-merged develop@a42e3eaf (D-1055 POL-14 post-merge burst).** Status `in_progress → merged`. All gates satisfied: LOCAL 3-CLEAN converged passes 17/18/19 (D-1053; BC-5.39.001 D-779) + PR-LEVEL 3-CLEAN converged passes 1/2/3 (D-1054; BC-5.39.001 D-779) + pr-reviewer APPROVE + security SECURITY-CLEAR-TO-MERGE + CI 43/43 GREEN. POL-14 BC promotions: BC-2.06.001 v1.2→v1.3 draft→active; BC-2.06.003 v1.10→v1.11 draft→active; BC-2.03.005/007/BC-2.22.001 idempotent no-ops (already active). Phase B Lane 4 COMPLETE. Cascade CLOSED. |
| 1.17 | 2026-06-07 | story-writer | **F-P15-HIGH-002 — async-signature claim correction (BC-2.06.003 v1.9 propagation):** Two story-side false claims that "`CredentialRefProbe` trait signature UNCHANGED" corrected to accurately reflect the pass-14 fix (commit 0941c0e0). **(1) Architecture Compliance Rules** — boot-probe row Enforcement cell: replaced "UNCHANGED" assertion with: "`CredentialRefProbe::probe` converted to `async` (via `#[async_trait]`) in pass-14 (commit 0941c0e0); all 5 impls and call sites updated; `org_registry: &OrgRegistry` param unchanged from v1.3; see BC-2.06.003 v1.9 §OrgRegistry and KeyringStore Threading." Source citation updated from v1.8 to v1.9. **(2) v1.16 Changelog row** — "UNCHANGED" phrase in the Enforcement narrative amended to the same accurate async-conversion description. No other story content changed. `acceptance_criteria_count` (14), `red_gate_tests` (9), and `status` (in_progress) all unchanged. No STORY-INDEX, STATE.md, BC-INDEX, or sprint-state files touched — state-manager owns those. |
| 1.16 | 2026-06-07 | story-writer | **F-P14-CRIT-001 boot-probe BC-2.06.003 v1.8 propagation:** Closed F-P14-CRIT-001 at the story level by propagating the BC-2.06.003 v1.8 §Boot-Step-5 Probe Alignment amendment. Changes: **(1) Architecture Compliance Rules** — added row: "Boot probe must use OrgId-keyed probe (Tier 3a, `get_by_org` per registered org) as PRIMARY keyring check; legacy `{sensor_id}/{ref_name}` is Tier 3b FALLBACK only" — Source: BC-2.06.003 v1.8 §Boot-Step-5 Probe Alignment; Enforcement: `KeyringCredentialProbe` gains `keyring: Arc<dyn CredentialStoreOrgId>` field; `step5_init_credential_store` passes the shared `Arc<KeyringBackend>` at construction; `CredentialRefProbe::probe` converted to `async` (via `#[async_trait]`) in pass-14 (commit 0941c0e0) — all 5 impls and call sites updated; `org_registry: &OrgRegistry` param unchanged from v1.3 — see BC-2.06.003 v1.9 §OrgRegistry and KeyringStore Threading for authoritative correction. **(2) Red Gate Tests** — added TV-BOOT-P-001 row: `test_BC_2_06_003_boot_probe_tier3a_finds_org_id_keyed_credential` in `crates/prism-bin/src/boot.rs` `#[cfg(test)] mod tests` — in-process unit test using `InMemoryCredentialStore`, calls `set_by_org`, asserts `probe(...)` returns `Ok(None)`; maps to AC-002 / AC-009 / BC-2.06.003 v1.8. **(3) `red_gate_tests` frontmatter** 8→9. **(4) AC-002** — added minimal clarifying note (3 sentences) that boot step 5 resolves OrgId-keyed credentials (Tier 3a PRIMARY) per BC-2.06.003 v1.8 §Boot-Step-5 Probe Alignment; closes F-P14-CRIT-001. **(5) Tasks 10 and 27** — Red Gate test count updated 8→9. No new ACs added (`acceptance_criteria_count` unchanged at 14 — existing AC-002/AC-009 are satisfied by the BC amendment per PO assessment). No STORY-INDEX/STATE.md/sprint-state changes (state-manager owns those). |
| 1.15 | 2026-06-07 | story-writer | **E-CRED-005 → E-CRED-008 re-align (Tier-3 keyring backend path) per ADR-035 §D5 / error-taxonomy v1.62:** S-MAINT-ECRED-TAXONOMY-SYNC-001 (merged develop@c603741d) established the canonical E-CRED-001..010 namespace. The Tier-3 keyring/backend-unavailable path is canonically E-CRED-008 (`BackendUnavailable`) per error-taxonomy.md v1.62 §E-CRED-008 and ADR-035 §D5. The old E-CRED-005 cite for this path was a collision with `CredentialFileIo` (Tier-1 file I/O), now canonically E-CRED-005. All Tier-3/keyring-backend occurrences of E-CRED-005 flipped to E-CRED-008 across: frontmatter subsystem comment (SS-03 anchor), BC-2.03.007 frontmatter comment, `risk_mitigations` detail-string item, AC-011 header, AC-011 Case B detail string, AC-011 BC-trace clause, AC-006 §6(b) runbook header and body, Architecture Compliance Rules Tier-3 error row (source citation expanded: added ADR-035 §D5), FSR `docs/DEMO-RUNBOOK.md` row, DEMO-RUNBOOK §6 description, Red Gate Tests table (RG-034-005 test name `...e_cred_005` → `...e_cred_008`), Task 23, EC-001b detail string (source citation expanded: added ADR-035 §D5), EC-008 error code (source citation expanded: added ADR-035 §D5). Red Gate test name aligned with implementer commit 3bed8ea1: `test_BC_2_06_003_tier3_backend_error_returns_e_cred_008`. No Tier-1 file-I/O E-CRED-005 references exist in this story — no preservation was needed. Changelog historical rows (v1.6, v1.12, etc.) left unchanged as immutable history. |
| 1.14 | 2026-06-06 | story-writer | **F-002/F-003/F-004 consistency-audit fixes:** **(F-002 MED)** FSR `scripts/demo-setup.sh` row: `allowed_urls` value corrected from `["http://127.0.0.1"]` to `["api.crowdstrike.com", "127.0.0.1"]` — matching `scripts/demo-setup.sh:150` exactly; added note that SEC-003 validates hostnames only (no scheme prefix). **(F-003 MED)** AC-003 poll timeout corrected from "within **10s**" to "within **30s**" — matching `demo-run.sh` `POLL_TIMEOUT=30` and EC-004 (both authoritative). **(F-004 LOW)** AC-011 Case B detail string updated from the terse/incorrect `"E-CRED-005: OS keyring unavailable: NoStorageAccess"` to reflect the real code format: `"E-CRED-005: OS keyring unavailable: backend={backend}: {reason}. Check keyring access (macOS Keychain / Linux libsecret). Use Tier 1/2 env vars as an alternative (BC-2.06.003)."` — sourced from `resolution.rs:237-259` (F-P6-OBS-003 `inner_detail` pattern + guidance suffix). Inner detail note added explaining the E-CRED-004-prefix-strip rationale. No BC/code/script/STORY-INDEX changes. |
| 1.13 | 2026-06-06 | story-writer | **Runbook §6 structure alignment (pass-17 proactive propagation):** DEMO-RUNBOOK.md §6 was split from 3 subsections into 4 by commit 5676b5fc — keyring error split into §6(a) write-fail/E-CRED-004 and §6(b) read-fail/E-CRED-005; port already in use renumbered §6(c); TOML not found renumbered §6(d). AC-006 updated: count 3→4 failure modes; AC-006 item list rewritten to match the 4 subsections with correct §-letter assignments (write→§6a/E-CRED-004, read→§6b/E-CRED-005, port→§6c, TOML→§6d). "docs/DEMO-RUNBOOK.md Required Sections" §6 description updated from "3 failure modes" to "4 failure modes" with explicit subsection enumeration. No BC/code/script/STORY-INDEX changes. |
| 1.12 | 2026-06-06 | story-writer | **F-P16-MED-002 (EC-001 write-path/read-path error-code split + env-var-fallback alignment):** EC-001 conflated two distinct error codes on two distinct paths. Fixed by splitting into EC-001 (write path) and EC-001b (read path): **(write path — EC-001)** `prism credential set` keyring write failure surfaces via `handle_credential_set` as "Keyring unavailable: {e}…" — this is E-CRED-004 (`PrismError::CredentialStoreError`; credential_cli.rs), NOT E-CRED-005. Operator guidance updated to reference the per-client env var fallback (`PRISM_CLIENTS_<ORG>_SENSORS_<SENSOR>_<REF>`, per DEMO-RUNBOOK §6a / demo-setup.sh) instead of an "encrypted file backend." **(read path — EC-001b)** `resolve_credential` Tier-3 backend error (`Err(NoStorageAccess)`) returns `CredentialResolutionError::BackendUnavailable` with E-CRED-005 detail string — hard error, no Tier-4 fallthrough (ADR-034 §D4). E-CRED-005 reference now attaches ONLY to the Tier-3 read clause; the write-failure clause references E-CRED-004. AC-006(a) left unchanged — it correctly covers the runtime read-path E-CRED-005 (keyring access denied during credential resolution, not during setup). No BC/code/script/STORY-INDEX changes. |
| 1.11 | 2026-06-06 | story-writer | **F-P12-MED-001 + F-P13-MED-001 + F-P12-LOW-001 + comprehensive behavioral audit (pass-12/13 fixes):** **(1) F-P12-MED-001 (AC-012 exit code + error message):** AC-012 corrected: "exits 1" → "exits 2" (EXIT_CONFIG_INVALID — ADR-022 §A; code path: resolve_org_slug_and_id Err → handle_credential_set_with_store returns EXIT_CONFIG_INVALID); quoted error message replaced with the ACTUAL string from credential_cli.rs:503-508: "Could not load prism.toml from '<config_dir>': <reason>. Ensure prism.toml exists (run demo-setup.sh or create it manually) before running `prism credential set`." (previous text "Provide --org-slug explicitly or ensure prism.toml is present." was invented, never in code). `prism credential set` BEHAVIOR spec updated to explicitly list exit 2 for config-invalid (was omitted). Multi-org error message in BEHAVIOR spec updated to match code (added path and "Configured orgs:" suffix). **(2) F-P13-MED-001 (AC-007 note: demo-teardown.sh --org-slug claim):** AC-007 Note corrected: "demo-teardown.sh passes `--org-slug $DEMO_ORG_SLUG` for explicitness" is FALSE — grep of scripts/demo-teardown.sh shows zero `--org-slug` references. The script relies on single-org auto-resolution. Note rewritten to match actual behavior. **(3) F-P12-LOW-001 (BC table title BC-2.06.003):** Dropped the " (Tier-3 IMPLEMENTED — ADR-034)" enrichment suffix from the BC-2.06.003 title cell in the Behavioral Contracts table. POL-7 requires verbatim H1 match; BC-INDEX:103 H1 is "Credential References in Config Resolve to Credential Store Entries" (no suffix). Tier-3 context preserved in the BC frontmatter comment in the YAML block. **(4) Comprehensive behavioral audit — 7 additional drift items corrected:** (a) AC-005 prompt text: "Enter value: " → "Enter value for prism/<sensor>/<name>: " (code: credential_cli.rs:199 `format!("Enter value for prism/{}/{}: ", args.sensor, args.name)` + `eprint!`). (b) EC-006 exit code: "exits 1" → "exits 2 (EXIT_CONFIG_INVALID)" (same Err path as AC-012). (c) EC-006 message: "Org slug '<slug>' not found in prism.toml. Available: [<slugs>]" → "--org-slug '<slug>' not found in prism.toml '<path>'. Configured orgs: [<slugs>]" (code: credential_cli.rs:530-533). (d) EC-003 message: single-line quote → 3-line stderr output matching actual script. (e) EC-004 message: "DTU server did not start within 30s. Check demo.toml for port conflicts." → actual 3-line message from demo-run.sh including log-file path and "Common cause: port conflict" suffix. Items verified MATCH (no drift): AC-001 (exit 0 + dirs created), AC-002 (--config-dir flag, boot step event), AC-003 (DTU launch + urls.json poll), AC-004 (DEMO-RUNBOOK.md §4), AC-005 (set_by_org, OrgId-keyed key, AD-017 no-echo, exit 0), AC-007 (delete_by_org, ordering, exit 0), AC-008 (shellcheck zero warnings), AC-009 (env vars in prism start command env, overlay TOMLs written), AC-010 (OrgId-keyed key; NOT slug-keyed), AC-011 (Tier-3 miss falls through; backend error → BackendUnavailable/E-CRED-005), AC-013 (zero grep matches for retired formats), AC-014 (shellcheck in ci.yml), 8 Red Gate test names (all verified against worktree test files), env var names (CROWDSTRIKE_BASE_URL/ARMIS_INSTANCE_URL/CLAROTY_INSTANCE_URL/CYBERINT_ENVIRONMENT), namespace format "{org_id_uuid}/{sensor}/{name}", DEMO UUID "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0b1c", AC count 14, red_gate_tests 8, demo-setup.sh steps 1-8 / credential bootstrap / --org-slug passing / set_cred helper. No BC/code/script/STORY-INDEX changes. |
| 1.10 | 2026-06-06 | story-writer | **F-P10-CRIT-001 + F-P10-HIGH-001 (pass-10 fixes):** (1) F-P10-CRIT-001: documented keyring real-backend feature enablement in `crates/prism-credentials/Cargo.toml` (apple-native / windows-native / linux-native-sync-persistent / linux-native) + 4 `compile_error!` regression guards added to `crates/prism-credentials/src/lib.rs` that fire on zero-backend builds, preventing silent reversion to the in-process mock keystore. Tests use `install_keyring_mock()` (prism-credentials/src/tests/mod.rs) + `InMemoryCredentialStore` injection — no real OS Keychain in CI. FSR gains 4 new file rows (prism-credentials Cargo.toml, lib.rs guards, tests/mod.rs, tests/store_tests.rs). Architecture Compliance Rules gains 2 new rows (backend enablement guard; test mock-override). Forbidden Dependencies gains 1 new row (zero-feature guard). (2) F-P10-HIGH-001: `prism credential delete --org-slug <slug> --sensor <s> --name <n>` subcommand added — `handle_credential_delete` / `handle_credential_delete_with_store` in `credential_cli.rs`; calls `CredentialStoreOrgId::delete_by_org` (OrgId-keyed, matches write path exactly); idempotent (Ok(false) → exit 0); exits 1 on backend error, exits 2 on config-invalid. `demo-teardown.sh` now uses `prism credential delete` on ALL platforms, replacing wrong platform-native CLI calls. New Red Gate test `test_handle_credential_delete_uses_org_id_keyed_namespace` added to `bc_2_03_007_credential_set_org_id_keyed.rs` (AC-007 / BC-2.03.005). `red_gate_tests` 7→8. AC-007 rewritten to reference `prism credential delete` and the ordering constraint (deletes before `rm -rf`). Subcommand spec section expanded to include `prism credential delete`. Architecture Compliance Rules gains 2 new rows (delete uses `delete_by_org`; ordering constraint). Forbidden Dependencies gains 1 new row (platform-native CLI for deletion). FSR `credential_cli.rs` and `demo-teardown.sh` rows updated. Token budget updated ~47K→~51K. Title updated to include `set/delete`. Tasks 10, 27 count updated 7→8; task 17a added; task 18 updated. No BC/code/script/STORY-INDEX changes. |
| 1.9 | 2026-06-06 | story-writer | **F-P8-MED-001 + F-P8-MED-002 + comprehensive citation audit (pass-8 fixes):** (1) F-P8-MED-001: Open Question 3 corrected — Cyberint auth_type is `cookie_roundtrip` / credential name is `api_key` (not `bearer_static`/`bearer_token`). Only Armis + Claroty are `bearer_static`/`bearer_token`. CrowdStrike is `oauth2_client_credentials`/`client_id`+`client_secret`. All auth_type and cred-name values are D-747 LOCKED per `crates/prism-sensors/specs/*.sensor.toml`. (2) F-P8-MED-002: Body H1 header `**Version:** v1.7` corrected to `v1.9` (was 2 versions behind frontmatter). (3) Comprehensive citation audit: 45 literal sites checked across 8 classes. Stale citations corrected: `plugin_auth_provider.rs` FSR row "lines 135, 145" removed (volatile line pins per TD-VSDD-091) — replaced with function-name anchor `PluginAuthProvider::acquire_token`; Tasks §6 "lines 130–150" similarly replaced with behavioral anchor. "5 test doubles" → "3 test doubles" at 3 body sites (FSR table line 416, Task 5 line 504, Task 13 line 512) — actual count verified: `MockCredentialResolver`, `NotFoundCredentialResolver`, `BackendUnavailableCredentialResolver`. All other literal classes (7 RG test names, CLI flags, file paths, env vars, error codes, namespace format, demo UUID) verified CURRENT against worktree. No BC/code/script/STORY-INDEX changes. |
| 1.8 | 2026-06-06 | story-writer | **F-P7-MED-001 pass-7 fix:** Corrected stale production-function name `resolve_org_slug` → `resolve_org_slug_and_id` at AC-012 header, Architecture Compliance Rules row, and Forbidden Dependencies row. The bare `resolve_org_slug` helper was removed in F-LOW-003 (dead code); story body was not propagated until now. Behavior unchanged — hard-error on missing/invalid prism.toml, no demo-org fallback. No BC/code/script/STORY-INDEX changes. |
| 1.7 | 2026-06-06 | story-writer | **F-P6-MED-001+002 pass-6 fix:** AC-002 `--config` → `--config-dir` (real CLI flag; global flag before subcommand per cli.rs `#[arg(long, global = true)]`); corrected invocation is `./target/release/prism --config-dir ~/.config/prism-demo/ start`. RG-034-004 cited test name → `test_handle_credential_set_writes_org_id_keyed_namespace` at all 3 story locations (Red Gate Tests table, AC-005, AC-010); name verified against `.worktrees/S-DEMO-003/crates/prism-bin/tests/bc_2_03_007_credential_set_org_id_keyed.rs:121`. No BC/code/script/STORY-INDEX changes. |
| 1.6 | 2026-06-06 | story-writer | **F-P5-MED-001 pass-5 fix:** Added RG-034-005 (`test_BC_2_06_003_tier3_backend_error_returns_e_cred_005`) in `crates/prism-credentials/tests/bc_2_06_003_tier3_keyring_resolution.rs` — covers AC-011 Case B (keyring backend `Err` → `CredentialResolutionError::BackendUnavailable`/E-CRED-005; no Tier-4 fall-through; no credential-value leak in detail). Test uses `InMemoryCredentialStore` error-injection mode (test-helpers-gated seam). AC-011 Case B now explicitly cites RG-034-005. Red Gate Tests table gains RG-034-005 row. `red_gate_tests` frontmatter 6→7. Tasks 10 and 27 updated (6→7 count). Points justification comment updated (`RG-034-001..004` → `RG-034-001..005`). No BC/code/script/STORY-INDEX changes. |
| 1.5 | 2026-06-06 | story-writer | **F-MED-302 pass-3 mechanism mis-characterisation fix:** Removed the non-existent "per-org egress allowlist to include 127.0.0.1" language from AC-009 Given precondition, the Architecture Compliance Rules `demo-run.sh` row, and the §FSR `demo-run.sh` row. Replaced with the accurate two-part data-path mechanism: **(a) TYPE-spec env vars (step-4a boot gate)** — `demo-run.sh` must export `CROWDSTRIKE_BASE_URL`, `ARMIS_INSTANCE_URL`, `CLAROTY_INSTANCE_URL`, `CYBERINT_ENVIRONMENT` before invoking `prism start`; without these env_resolver.rs fires E-SPEC-024 and boot.rs step-4b hard-aborts before step-4c overlays are reachable; **(b) step-4c per-org overlays** — `demo-run.sh` writes `specs/customers/demo-org/<sensor>.sensor.toml` with ephemeral DTU port after parsing `urls.json`; **(c) CrowdStrike OAuth2 plugin SEC-003 only** — `allowed_urls` in `crowdstrike-oauth2.manifest.toml` (written by `demo-setup.sh`) gates only the CrowdStrike plugin host-function; Armis/Claroty/Cyberint use plain `reqwest::Client` with NO per-org/host egress allowlist. §FSR `demo-setup.sh` row updated to clarify SEC-003 nature of the manifest `allowed_urls`. No BC/code/script/STORY-INDEX changes. |
| 1.4 | 2026-06-06 | story-writer | **F-HIGH-201 pass-2 spec reconciliation:** Overlay generation responsibility moved from `demo-setup.sh` to `demo-run.sh` — DTU ports are ephemeral (only known post-launch), so per-org `base_url` overlay TOMLs (`specs/customers/demo-org/<sensor>.sensor.toml`) must be written by `demo-run.sh` after `urls.json` is parsed. AC-001 clarified to explicitly exclude overlay-generation from `demo-setup.sh` scope. AC-009 "Given" block expanded with explicit precondition that `demo-run.sh` has written overlays and extended the egress allowlist before queries are issued. File Structure Requirements `demo-run.sh` row updated with overlay-generation detail; `demo-setup.sh` row clarified as pre-launch-only. Two new Architecture Compliance Rules added: `demo-setup.sh MUST NOT write overlays` and `demo-run.sh MUST write overlays before prism-bin launch`. Previous Story Intelligence `S-CONFIG-MULTI-TENANT-OVERRIDE-001` entry corrected to name `demo-run.sh` as the overlay author. OBS [process-gap] note added for `scripts/start-demo.sh` overlap with follow-up story reference `S-DEMO-LAUNCHER-CONSOLIDATION-001` (draft). |
| 1.3 | 2026-06-06 | story-writer | **F-LOW-002 pass-1 fix:** Tightened AC-013 env-format verification grep to avoid false-positive on `DEMO_ORG_SLUG`/`DEMO_ORG_ID` bash local vars. Retired-credential grep changed from `DEMO_ORG_` (overbroad — matches local bash vars) to `DEMO_ORG_[A-Z_]+_SENSORS` (targets only the retired credential env-var infix `_SENSORS`). Non-prefixed global format grep changed from `^[A-Z]+_BEARER_TOKEN` to `^(export )?[A-Z]+_BEARER_TOKEN=` (anchored with optional `export` prefix and trailing `=` to avoid substring matches). Architecture Compliance Rules table grep-gate column updated to match. |
| 1.2 | 2026-06-06 | story-writer | **Option-A scope expansion (ADR-034; human approved 2026-06-06):** Tier-3 OS-keyring credential resolution now fully in scope. AC-005 rewritten: write path is `CredentialStoreOrgId::set_by_org` (OrgId-keyed); `--org-slug` required for multi-org; HIGH-3 error on missing prism.toml enforced. New ACs added: AC-009 (CRIT-1 end-to-end), AC-010 (CRIT-2 namespace regression), AC-011 (Tier-3 error semantics / E-CRED-005), AC-012 (HIGH-3 toml error), AC-013 (HIGH-1 env format), AC-014 (HIGH-2 shellcheck CI). Points 5→8 (ADR-034 §D7 range 8-10; 8 chosen). Risk LOW→MEDIUM. `crates_touched` expanded: add `prism-credentials`, `prism-spec-engine`. `subsystems` expanded: add SS-08 (Spec-Driven Adapter). `acceptance_criteria_count` 8→14. `red_gate_tests` 2→6. Version bumped 1.1→1.2. Error code E-CRED-005 used throughout (E-CRED-003 already allocated per ADR-034 §D4 annotation). |
| 1.1 | 2026-06-03 | state-manager | D-990 Phase-A-close: status draft→ready; depends_on S-DEMO-001 (merged PR #166) + S-DEMO-002 (merged PR #171) BOTH SATISFIED; BC-2.03.005 v1.6 active + BC-2.03.007 v1.3 active; S-7.01 gate CLEARED. |
| 1.0 | 2026-05-29 | story-writer | Initial draft — bundled CLI subcommand per complexity assessment; 4-sensor scope |
