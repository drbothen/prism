# Prism DTU Demo Runbook

This runbook guides an MSSP analyst or demo operator through standing up the Prism DTU demo
environment, connecting Claude Code as an MCP client, and issuing live PrismQL queries against
all four sensor DTU clones (CrowdStrike, Armis, Claroty, Cyberint).

Story: S-DEMO-003 | ACs: AC-004, AC-006

---

## 1. Prerequisites

Before running setup, ensure the following are installed:

| Tool | Check | Install |
|------|-------|---------|
| Rust toolchain | `cargo --version` | `curl https://sh.rustup.rs -sSf \| sh` (then follow `rust-toolchain.toml`) |
| `just` | `just --version` | `cargo install just` |
| `cargo-nextest` | `cargo nextest --version` | `cargo install cargo-nextest` |
| `shellcheck` | `shellcheck --version` | `brew install shellcheck` (macOS) or `apt install shellcheck` |
| macOS Keychain or libsecret | OS-provided | Built-in on macOS; `apt install gnome-keyring` on Debian/Ubuntu |

Verify the repo is cloned and the Rust toolchain matches `rust-toolchain.toml`:

```bash
cargo --version  # should print the pinned stable version
just --list      # should list available recipes
```

---

## 2. One-time Setup

Run the setup script from the repo root (idempotent — safe to run multiple times):

```bash
bash scripts/demo-setup.sh
```

What the script does:

1. **Builds** `prism` and `prism-dtu-demo-server` in release mode
2. **Creates** `~/.config/prism-demo/` with `specs/`, `state/`, `plugins/`, and `run/` subdirectories
3. **Copies** the four sensor TOML specs (`crowdstrike`, `armis`, `claroty`, `cyberint`) to `~/.config/prism-demo/specs/`
4. **Copies** `crowdstrike-oauth2.prx` plugin artifact to `~/.config/prism-demo/plugins/`
5. **Writes** `~/.config/prism-demo/prism.toml` with the demo org (`org_slug = "demo-org"`, UUID v7 `org_id`)
6. **Stores** dummy credentials in the OS keyring for all four sensors via `prism credential set`
7. **Prints** next-step instructions

Override the config directory if needed:

```bash
bash scripts/demo-setup.sh --config-dir /custom/path/prism-demo
```

---

## 3. Daily Demo Run

After setup, start the DTU server and follow the instructions printed:

```bash
bash scripts/demo-run.sh
```

The script:
1. Starts `prism-dtu-demo-server` in the background on four ephemeral ports
2. Polls for the URL sidecar file (up to 30 seconds)
3. Prints the CrowdStrike, Armis, Claroty, and Cyberint DTU URLs
4. Prints the exact command to start `prism`

Start `prism` in a **new terminal** using the command printed by `demo-run.sh`:

```bash
~/.cargo/bin/prism --config-dir ~/.config/prism-demo start
# (the exact path is printed by demo-run.sh; use that)
```

`prism start` boots all 11 boot steps and emits `boot.step9a.adapter_registry_populated`
with `sensor_count=4, org_count=1` when all DTU adapters are registered. It then accepts
MCP connections via stdio transport.

---

## 4. Connecting Claude Code

Add `prism` as an MCP server in your Claude Code settings:

**File location:** `~/.claude/settings.json`

**Add the following JSON** under the `mcpServers` key (create the key if absent):

```json
{
  "mcpServers": {
    "prism": {
      "command": "/Users/YOUR_USERNAME/Dev/prism/target/release/prism",
      "args": ["--config-dir", "/Users/YOUR_USERNAME/.config/prism-demo", "start"],
      "env": {}
    }
  }
}
```

Replace `/Users/YOUR_USERNAME` with your actual home directory path (use `echo $HOME` to find it).

**Exact invocation** (substitute your path):

```bash
prism --config-dir ~/.config/prism-demo start
```

Verify the MCP connection from Claude Code:

```
/mcp list
```

You should see `prism` listed as a connected server with `tool_query` available.

---

## 5. Example Queries

Once connected, issue PrismQL queries via the `tool_query` MCP tool. Use `/mcp` in Claude Code
to invoke MCP tools.

**CrowdStrike — recent detections:**

```sql
FROM crowdstrike_detections LIMIT 5
```

Expected output: OCSF-normalized Arrow batch with columns `sensor`, `_time`, and detection fields.

**Armis — device inventory:**

```sql
FROM armis_devices LIMIT 5
```

Expected output: OCSF-normalized rows with device attributes.

**Claroty — OT asset inventory:**

```sql
FROM claroty_assets LIMIT 5
```

Expected output: OCSF-normalized rows with OT asset metadata.

**Cyberint — threat intelligence alerts:**

```sql
FROM cyberint_alerts LIMIT 5
```

Expected output: OCSF-normalized alert rows.

All queries fan out to the appropriate DTU clone running on localhost. No real external
API calls are made during demo mode.

---

## 6. Troubleshooting

### (a) Keyring access denied

**Symptom:** `prism credential set` prints `Keyring unavailable:` or the demo sensors
fail to authenticate at query time.

**Cause:** The OS keyring service is unavailable (headless server, Docker container,
missing Gnome Keyring on Linux).

**Fix (macOS):** Unlock the macOS Keychain:
```bash
security unlock-keychain ~/Library/Keychains/login.keychain-db
```

**Fix (Linux):** Start the D-Bus keyring service:
```bash
eval "$(dbus-launch --sh-syntax)"
gnome-keyring-daemon --start --components=secrets
```

**Fallback (CI / headless):** Use environment variables instead of the keyring.
Prism resolves credentials from env vars before checking the keyring (Tier 1/2 in the
four-tier resolution chain — BC-2.06.003). The canonical format is:
`PRISM_CLIENTS_{ORG_SLUG_UPPER}_SENSORS_{SENSOR_UPPER}_{REF_UPPER}`
where `{ORG_SLUG_UPPER}` is the org slug in SCREAMING_SNAKE_CASE (hyphens → underscores).
For the demo org `demo-org` (`DEMO_ORG`):
```bash
export PRISM_CLIENTS_DEMO_ORG_SENSORS_CROWDSTRIKE_CLIENT_ID="demo-cs-client-id"
export PRISM_CLIENTS_DEMO_ORG_SENSORS_CROWDSTRIKE_CLIENT_SECRET="demo-cs-client-secret"
export PRISM_CLIENTS_DEMO_ORG_SENSORS_ARMIS_BEARER_TOKEN="demo-armis-bearer-token"
export PRISM_CLIENTS_DEMO_ORG_SENSORS_CLAROTY_BEARER_TOKEN="demo-claroty-bearer-token"
export PRISM_CLIENTS_DEMO_ORG_SENSORS_CYBERINT_API_KEY="demo-cyberint-api-key"
```
NOTE: The old format `DEMO_ORG_*` is RETIRED (ADR-032 / BC-2.06.003 v1.3). Use only
the `PRISM_CLIENTS_*` format above.

### (b) Port already in use

**Symptom:** `demo-run.sh` exits with "DTU server did not start within 30s" or the DTU log
shows `Address already in use (os error 98)`.

**Cause:** A previous DTU server is still running, or another process holds the port.

**Fix:**

1. Kill any stale DTU server:
   ```bash
   cat ~/.config/prism-demo/run/.prism-dtu-demo-server.pid | xargs kill 2>/dev/null || true
   ```

2. Find and kill other processes on the ports:
   ```bash
   lsof -i :17080 -i :17081 -i :17082 -i :17083 2>/dev/null | awk 'NR>1 {print $2}' | xargs kill -9 2>/dev/null || true
   ```
   (Use port numbers from the DTU server log if they differ.)

3. Re-run `demo-run.sh`.

### (c) TOML spec not found

**Symptom:** `prism start` exits 2 with an error like `spec_dir does not exist` or
`parse_spec_directory: no .sensor.toml files found`.

**Cause:** The `spec_dir` in `prism.toml` points to an empty or nonexistent directory,
or the sensor TOML files were not copied by `demo-setup.sh`.

**Fix:**

1. Verify `spec_dir` in `~/.config/prism-demo/prism.toml`:
   ```bash
   cat ~/.config/prism-demo/prism.toml | grep spec_dir
   ```

2. List sensor TOML files in the specs directory:
   ```bash
   ls "$(cat ~/.config/prism-demo/prism.toml | grep spec_dir | awk -F'"' '{print $2}')"
   ```
   Expected: `armis.sensor.toml  claroty.sensor.toml  crowdstrike.sensor.toml  cyberint.sensor.toml`

3. If files are missing, re-run setup:
   ```bash
   bash scripts/demo-setup.sh
   ```

---

## 7. Cleanup

To completely remove the demo environment:

```bash
bash scripts/demo-teardown.sh
```

What the script does:

1. Kills the DTU demo server (via PID file at `~/.config/prism-demo/run/.prism-dtu-demo-server.pid`)
2. Removes `~/.config/prism-demo/` entirely (all config, specs, state, plugins, logs)
3. Deletes the 5 demo OS keyring entries (`crowdstrike/client_id`, `crowdstrike/client_secret`,
   `armis/bearer_token`, `claroty/bearer_token`, `cyberint/api_key`)

The binary artifacts (`target/release/prism`, `target/release/prism-dtu-demo-server`) are NOT
removed — run `cargo clean` separately if needed.

Override the config directory:

```bash
bash scripts/demo-teardown.sh --config-dir /custom/path/prism-demo
```
