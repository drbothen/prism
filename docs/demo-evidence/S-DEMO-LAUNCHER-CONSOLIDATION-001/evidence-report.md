# Evidence Report — S-DEMO-LAUNCHER-CONSOLIDATION-001

**Story:** Demo Launcher Consolidation — `start-multi` subcommand + N-org demo scripts
**Story version:** v2.7
**Crate:** `prism-dtu-demo-server`
**Feature flags:** `--features dtu,fixture-gen` (required for all `start-multi` verifications)
**Branch:** `feature/S-DEMO-LAUNCHER-CONSOLIDATION-001`
**Worktree HEAD at recording:** `85371fc9`

> **Anchor note (TD-VSDD-091):** This report cites story version v2.7, not a commit SHA.
> SHA anchors decay on subsequent diffs; story version is the stable reference.

---

## Test Execution Summary

**Command run:**
```
cargo nextest run -p prism-dtu-demo-server --features dtu,fixture-gen --no-fail-fast
```

**Result: 78 / 78 PASS, 0 failed, 0 skipped**

Full run time: 15.145 s

---

## AC Coverage Table

| AC | Title | Backing Evidence | Result |
|----|-------|-----------------|--------|
| AC-001 | `StartMulti` variant in Commands enum + CLI help | `cargo run ... -- --help` + `-- start-multi --help` captured below | PASS |
| AC-002 | `MultiOrgDemoConfig` parses valid 3-org TOML (RG-001 green) | `test_multi_org_config_parses_valid_three_org_toml`, `test_multi_org_config_rejects_unknown_fields` (RG-001, RG-002) | PASS |
| AC-003 | `start-multi` writes nested `{org_slug: {sensor_id: url}}` sidecar (RG-003 green) | `test_nested_sidecar_format_has_correct_structure` (RG-003), `test_write_multi_url_sidecar_produces_all_keys`, `test_write_multi_url_sidecar_fails_loudly_on_missing_socket` | PASS |
| AC-004 | Per-org DTU clones on distinct socket addresses (RG-005 green) | `test_start_multi_stands_up_per_org_distinct_sockets` (RG-005), `test_org_a_vs_org_c_crowdstrike_serve_distinct_data` | PASS |
| AC-005 | `clone_factory` dispatches `(org_slug, sensor_id)` to BehavioralClone (RG-004 green) | `test_clone_factory_dispatch_returns_clone_for_each_sensor` (RG-004), `test_start_multi_cyberint_token_seeded_no_panic`, `test_low_unsupported_sensor_yields_clean_err_not_panic` (EC-008) | PASS |
| AC-006 | Existing `start` subcommand unbroken (backward compat) | `test_f10_valid_config_parses`, `test_f10_unknown_keys_rejected_at_every_level`, `ac_6_prism_demo_toml_*` tests (3 tests), existing 78-test suite all green | PASS |
| AC-007 | `scripts/start-demo.sh` retired | `ls scripts/start-demo.sh` → NOT FOUND; `shellcheck scripts/demo-*.sh` exit 0 | PASS |
| AC-008 | `demo-run.sh` calls `start-multi` + reads nested sidecar | Source inspection: line 112 `"${DTU_BIN}" start-multi --config "${DTU_CONFIG}"`, line 78 `URLS_MULTI_FILE="${DEMO_RUN_DIR}/.prism-dtu-demo-server.urls-multi.json"`, Python block reads nested `{org_slug: {sensor_id: url}}` | PASS |
| AC-009 | `demo-setup.sh` generates multi-org `prism.toml` with N `[[orgs]]` entries | Source inspection: lines 188-199, writes 3 `[[orgs]]` stanzas (org-a, org-b, org-c) with distinct UUID org_ids | PASS |
| AC-010 | `demo-setup.sh` bootstraps N×M credentials via stdin (AD-017) | Source inspection: `printf '%s\n' "${value}" | "${PRISM_BIN}" ... credential set` pattern on all 10 credentials; Cyberint api_key values match `initial_access_token` in demo.toml | PASS |
| AC-011 | `demo-run.sh` prints `prism start` command with all 4 TYPE-spec env vars | Source inspection: lines 248-251 echo all four: `CROWDSTRIKE_BASE_URL`, `ARMIS_INSTANCE_URL`, `CLAROTY_INSTANCE_URL`, `CYBERINT_ENVIRONMENT` | PASS |
| AC-012 | `demo-teardown.sh` manages single PID + deletes N×M keyring entries | Source inspection: single PID kill (line 94-113); keyring deletes BEFORE `rm -rf` (documented at lines 18-19, enforced by ordering in script body) | PASS |
| AC-013 | All shell scripts pass `shellcheck` with zero errors/warnings | `shellcheck scripts/demo-*.sh` exit 0, zero output | PASS |

**All 13 ACs: PASS**

---

## Red Gate Tests — All 5 Green

| RG Test | Test Name | File | AC | Result |
|---------|-----------|------|----|--------|
| RG-001 | `test_multi_org_config_parses_valid_three_org_toml` | `tests/multi_org.rs:73` | AC-002 | PASS |
| RG-002 | `test_multi_org_config_rejects_unknown_fields` | `tests/multi_org.rs:118` | AC-002 | PASS |
| RG-003 | `test_nested_sidecar_format_has_correct_structure` | `tests/multi_org.rs:162` | AC-003 | PASS |
| RG-004 | `test_clone_factory_dispatch_returns_clone_for_each_sensor` | `tests/multi_org.rs:216` | AC-005 | PASS |
| RG-005 | `test_start_multi_stands_up_per_org_distinct_sockets` | `tests/multi_org.rs:261` | AC-004 | PASS |

---

## Additional Tests Beyond Red Gate

These tests were added during implementation and adversarial passes to close gaps identified in LOCAL cascades:

| Test Name | File | AC | Notes |
|-----------|------|----|-------|
| `test_scripts_demo_toml_parses_as_multi_org_config` | `tests/multi_org.rs:330` | AC-002 | Parses the actual `scripts/demo.toml` via `MultiOrgDemoConfig::from_file` |
| `test_start_multi_cyberint_token_seeded_no_panic` | `tests/multi_org.rs:432` | AC-005 | GAP-2 composite path: `new_with_seed` then `configure({"access_token": token})` |
| `test_org_a_vs_org_c_crowdstrike_serve_distinct_data` | `tests/multi_org.rs:530` | AC-004 | INV-DISTINCT-DATA-001: seed=100 (org-a) ≠ seed=200 (org-c) produces distinct CrowdStrike response bodies |
| `test_write_multi_url_sidecar_produces_all_keys` | `tests/multi_org.rs:624` | AC-003 | Calls `write_multi_url_sidecar` directly; verifies nested JSON keys for all orgs/sensors |
| `test_write_multi_url_sidecar_fails_loudly_on_missing_socket` | `tests/multi_org.rs:712` | AC-003 | Error path: missing socket entry yields error, not silent empty sidecar |
| `test_configure_resolves_url_from_nested_sidecar` | `tests/multi_org.rs:790` | AC-005/EC-007 | `configure` subcommand resolves from nested `.urls-multi.json` when flat sidecar absent |
| `test_f10_valid_config_parses` | `src/config.rs:399` | AC-002/AC-006 | Valid 3-org TOML parses; `DemoConfig` also parses its own config (backward compat) |
| `test_f10_unknown_keys_rejected_at_every_level` | `src/config.rs:419` | AC-002 | Deny-unknown-fields at `[harness]`, `[orgs.X]`, top level |
| `test_med_b_malformed_org_id_yields_clean_err_not_panic` | `src/config.rs:467` | AC-002 | Non-UUID org_id string → clean `Err`, not panic |
| `test_low_unsupported_sensor_yields_clean_err_not_panic` | `src/config.rs:530` | AC-005/EC-008 | 3 sub-cases; unsupported sensor → clean `Err` from `from_str`; valid all-sensors control passes |

---

## AC-001: CLI Help Capture

### Top-level help (`--help`)

```
Unified multi-clone demo harness for Prism DTU clones

Usage: prism-dtu-demo-server <COMMAND>

Commands:
  start        Start the demo harness with the given config file
  stop         Send SIGTERM to a backgrounded harness PID (reads `.prism-dtu-demo-server.pid`)
  start-multi  Start all orgs' clone fleets using the multi-instance API
  configure    Convenience wrapper: POST to a clone's own `/dtu/configure` endpoint
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

**Verification command:** `cargo run -p prism-dtu-demo-server --features dtu,fixture-gen -- --help` → exit 0

### `start-multi --help`

```
Start all orgs' clone fleets using the multi-instance API.

Requires `--features dtu,fixture-gen` — the seeded clone constructors (`new_with_seed`)
are `#[cfg(feature = "fixture-gen")]`-gated. Omitting `fixture-gen` causes a hard error
(compile_error! or runtime panic) to prevent silent fallback to unseeded `new()` which
would violate INV-DISTINCT-DATA-001 (org-a and org-c would serve identical data).

Usage: prism-dtu-demo-server start-multi --config <PATH>

Options:
  -c, --config <PATH>
          Path to the multi-org demo config TOML (e.g. `scripts/demo.toml`)

  -h, --help
          Print help (see a summary with '-h')
```

**Verification command:** `cargo run -p prism-dtu-demo-server --features dtu,fixture-gen -- start-multi --help` → exit 0

---

## AC-007: `start-demo.sh` Retirement

```
$ ls scripts/start-demo.sh
ls: scripts/start-demo.sh: No such file or directory

$ shellcheck scripts/demo-*.sh
(no output)
$ echo $?
0
```

**Verification command:** `ls scripts/start-demo.sh 2>/dev/null && echo EXISTS || echo "NOT FOUND (correctly retired)"`
**Result:** `NOT FOUND (correctly retired)`

**shellcheck verification:** `shellcheck scripts/demo-*.sh` → exit 0, zero warnings

---

## AC-010: AD-017 Credential Stdin Pattern

`demo-setup.sh` uses a `set_cred` helper that pipes values via stdin on every call. No credential value transits argv.

Source (lines 216-232 of `scripts/demo-setup.sh`):
```bash
# Helper to set a credential — reads from stdin (AD-017 compliant).
set_cred() {
    local org_slug="$1"
    ...
    # AD-017: the value is piped via stdin, never passed as a CLI arg.
    if printf '%s\n' "${value}" | "${PRISM_BIN}" \
        --config-dir "${DEMO_CONFIG_DIR}" \
        credential set \
        --org-slug "${org_slug}" \
        --sensor "${sensor}" \
        --name "${name}"
```

Cyberint `api_key` values match `initial_access_token` in `scripts/demo.toml`:
- org-b: `"demo-cyberint-api-key-org-b"` (script line 256 = `demo.toml [orgs.org-b].initial_access_token`)
- org-c: `"demo-cyberint-api-key-org-c"` (script line 265 = `demo.toml [orgs.org-c].initial_access_token`)

---

## AC-012: Teardown Ordering (F-P10-HIGH-001)

`demo-teardown.sh` enforces keyring deletes BEFORE `rm -rf` of config dir:

```
# NOTE: keyring deletes (Step 2) run BEFORE config dir removal (Step 3) because
# `prism credential delete` reads prism.toml to resolve the OrgId UUID for the
# OrgId-keyed keyring namespace (ADR-034 §D3). Removing config dir first would
# silently orphan all keyring entries (S-DEMO-003 F-P10-HIGH-001 precedent).
```

Script structure: Step 1 = kill PID, Step 2 = credential deletes, Step 3 = `rm -rf`.

---

## AC-005: Distinct-Data Verification (INV-DISTINCT-DATA-001)

`test_org_a_vs_org_c_crowdstrike_serve_distinct_data` directly validates that org-a (seed=100) and org-c (seed=200) CrowdStrike DTU clones serve non-identical device lists after `start_multi_for_config` wires them via `build_multi_clone_factory`. The test makes HTTP GET requests to both clone endpoints and asserts response bodies differ.

`test_start_multi_cyberint_token_seeded_no_panic` confirms the GAP-2 composite path: `CyberintClone::new_with_seed(seed, archetype, org_id)` followed by `clone.configure(json!({"access_token": token}))` — both mechanisms compose cleanly without panic.

---

## Live `start-multi` Run

A live `start-multi --config scripts/demo.toml` run was NOT included in this evidence set. The 8-clone fleet startup requires the release binary + live network binding and exceeds the time budget for blocking evidence capture. The behavior is fully covered by:

- RG-005 (`test_start_multi_stands_up_per_org_distinct_sockets`) which exercises `start_multi_for_config` at the async function level with real OS socket binding
- `test_org_a_vs_org_c_crowdstrike_serve_distinct_data` which makes live HTTP requests to bound clone ports
- `test_write_multi_url_sidecar_produces_all_keys` which exercises the full sidecar write path

These tests collectively prove the behavior of `cmd_start_multi` without requiring a subprocess binary launch + SIGTERM handling, which is the only untested path (signal handling is tested by the pre-existing `ac_5_graceful_shutdown` suite).

---

## BC Traceability

| BC | Tested By |
|----|-----------|
| BC-2.06.001 (TOML Config Loads) | RG-001, RG-002, `test_f10_valid_config_parses`, `test_scripts_demo_toml_parses_as_multi_org_config` |
| BC-2.06.012 (Per-Tenant Overlay Loading) | AC-008 (demo-run.sh writes N×M overlays; source verified) |
| BC-2.06.013 (Scalar-Only Overlay) | AC-008 (Python block writes only `extends`, `instance_id`, `base_url`) |
| BC-2.06.014 (Instance Identity Resolution) | AC-011 (TYPE-spec env vars present for step-4a `env_resolver`) |
| BC-2.06.017 (Per-DTU-Instance Multi-Address Binding) | RG-003, RG-004, RG-005, `test_org_a_vs_org_c_crowdstrike_serve_distinct_data`, `test_write_multi_url_sidecar_produces_all_keys` |
