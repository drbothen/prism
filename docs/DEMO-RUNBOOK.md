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

1. **Checks** prerequisites (`cargo` must be available)
2. **Builds** `prism` and `prism-dtu-demo-server` in release mode
3. **Creates** `~/.config/prism-demo/` with `specs/`, `state/`, `plugins/`, and `run/` subdirectories
4. **Copies** the four sensor TOML specs (`crowdstrike`, `armis`, `claroty`, `cyberint`) to `~/.config/prism-demo/specs/`
5. **Copies** `crowdstrike-oauth2.prx` plugin artifact to `~/.config/prism-demo/plugins/`
6. **Writes** `~/.config/prism-demo/plugins/crowdstrike-oauth2.manifest.toml` — a DTU-safe plugin manifest that extends `allowed_urls` with `"127.0.0.1"` so the SEC-003 OAuth2 token-endpoint host check passes against the local DTU clone (the production plugin manifest at `api.crowdstrike.com` is not modified)
7. **Writes** `~/.config/prism-demo/prism.toml` with the demo org (`org_slug = "demo-org"`, UUID v7 `org_id`)
8. **Stores** dummy credentials in the OS keyring for all four sensors via `prism credential set`; prints next-step instructions

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
1. Starts `prism-dtu-demo-server` in the background on four ephemeral ports (port = 0 in demo.toml; no port conflicts between runs)
2. Polls for the URL sidecar file (`~/.config/prism-demo/run/.prism-dtu-demo-server.urls.json`) for up to 30 seconds
3. Prints the CrowdStrike, Armis, Claroty, and Cyberint DTU URLs (parsed from the urls.json sidecar)
4. **Generates per-org sensor overlays** — reads the urls.json sidecar and writes `~/.config/prism-demo/specs/customers/demo-org/<sensor>.sensor.toml` for each of the four sensors. Each overlay sets `base_url = "http://127.0.0.1:<ephemeral-port>"` so prism routes its fetch requests to the local DTU clone instead of the real sensor API. This step is required for AC-009 ("demo queries return data rows"). prism's spec loader derives the overlay directory as `spec_dir + "/customers"` at boot step 4c.
5. Prints the exact command to start `prism`

Start `prism` in a **new terminal** using the command printed by `demo-run.sh`:

```bash
target/release/prism --config-dir ~/.config/prism-demo start
# (use the exact command printed by demo-run.sh — it includes the full absolute path)
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
      "env": {
        "CROWDSTRIKE_BASE_URL": "http://127.0.0.1",
        "ARMIS_INSTANCE_URL": "http://127.0.0.1",
        "CLAROTY_INSTANCE_URL": "http://127.0.0.1",
        "CYBERINT_ENVIRONMENT": "demo"
      }
    }
  }
}
```

Replace `/Users/YOUR_USERNAME` with your actual home directory path (use `echo $HOME` to find it).

**Why these env vars are required** (E-SPEC-024 at boot step 4a):
The four sensor TYPE specs use `${env.VAR}` tokens in their `base_url` fields.
Prism resolves these tokens at boot step 4a (env resolution, before step 4c overlay
loading). If any variable is absent or empty, the boot process hard-aborts with exit
code 2 before the per-org overlay can wire the DTU clone URL. The values above satisfy
step 4a resolution — the per-org sensor overlays (generated by `demo-run.sh` at step 4c)
replace `base_url` with the actual DTU clone port before any HTTP request is made.

`CROWDSTRIKE_BASE_URL` must be `http://127.0.0.1` specifically: the `crowdstrike-oauth2`
plugin manifest's `allowed_urls` includes `["api.crowdstrike.com", "127.0.0.1"]`. SEC-003
validates the TYPE spec `base_url` host against this list at step 7.5b. The DTU-safe
manifest written by `demo-setup.sh` already includes `127.0.0.1`.

**Exact shell invocation** (substitute your path):

```bash
CROWDSTRIKE_BASE_URL=http://127.0.0.1 \
ARMIS_INSTANCE_URL=http://127.0.0.1 \
CLAROTY_INSTANCE_URL=http://127.0.0.1 \
CYBERINT_ENVIRONMENT=demo \
prism --config-dir ~/.config/prism-demo start
```

Verify the MCP connection from Claude Code:

```
/mcp list
```

You should see `prism` listed as a connected server with `query` available (the canonical MCP tool name registered via `pub async fn query` in prism-mcp; BC-2.11.001).

---

## 5. Example Queries

Once connected, issue PrismQL queries via the `query` MCP tool (the canonical tool name;
`FROM x LIMIT n` is pipe syntax and is rejected — use SQL form `SELECT * FROM x LIMIT n`).
Use `/mcp` in Claude Code to invoke MCP tools.

**CrowdStrike — recent detections:**

```sql
SELECT * FROM crowdstrike_detections LIMIT 5
```

Expected output: OCSF-normalized Arrow batch with columns `sensor`, `_time`, and detection fields.

**Armis — device inventory:**

```sql
SELECT * FROM armis_devices LIMIT 5
```

Expected output: OCSF-normalized rows with device attributes.

**Claroty — OT asset inventory:**

```sql
SELECT * FROM claroty_assets LIMIT 5
```

Expected output: OCSF-normalized rows with OT asset metadata.

**Cyberint — threat intelligence alerts:**

```sql
SELECT * FROM cyberint_alerts LIMIT 5
```

Expected output: OCSF-normalized alert rows.

All queries fan out to the appropriate DTU clone running on localhost. No real external
API calls are made during demo mode.

---

## 6. Troubleshooting

### (a) Keyring write failure at credential-set time — E-CRED-004

**Symptom:** `prism credential set` prints `Keyring unavailable: ...` and exits
non-zero. The credential is NOT stored. This is the **write path** error.

**Error code:** `E-CRED-004` (`PrismError::CredentialStoreError`, displayed as
`"E-CRED-004: credential store error (backend=...): {reason}"`), surfaced by
`handle_credential_set` in `credential_cli.rs` — AC-006(a) of S-DEMO-003 (EC-001).

**Cause:** The OS keyring service is unavailable at credential-set time (headless
server, Docker container, missing Gnome Keyring on Linux, or locked macOS Keychain).

**Fix (macOS):** Unlock the macOS Keychain:
```bash
security unlock-keychain ~/Library/Keychains/login.keychain-db
```
Then re-run `prism credential set ...`.

**Fix (Linux):** Start the D-Bus keyring service:
```bash
eval "$(dbus-launch --sh-syntax)"
gnome-keyring-daemon --start --components=secrets
```
Then re-run `prism credential set ...`.

**Fallback (CI / headless):** Use environment variables instead of the keyring.
Prism resolves credentials from env vars before checking the keyring (Tier 1/2 in
the four-tier resolution chain — BC-2.06.003). The canonical format is:
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

### (b) Keyring read failure at query time — E-CRED-005

**Symptom:** A query against a demo sensor fails at authentication time with:
`BackendUnavailable { detail: "E-CRED-005: OS keyring unavailable: ..." }`.
This is the **read path** error — the credential was previously stored (or was
expected to be) but the OS keyring is inaccessible during Tier-3 resolution.

**Error code:** `E-CRED-005` (surfaced by `resolution.rs` Tier-3 keyring backend
when the OS keyring is inaccessible — AC-006(b) of S-DEMO-003 (EC-001b)). There is
no Tier-4 fallthrough: if the keyring is unavailable at read time, the error is
hard-returned with no credential value leak.

**Cause:** The OS keyring service became unavailable between credential-set and
query time (daemon restart, session lock, headless re-entry).

**Fix (macOS):** Unlock the macOS Keychain:
```bash
security unlock-keychain ~/Library/Keychains/login.keychain-db
```

**Fix (Linux):** Restart the D-Bus keyring service:
```bash
eval "$(dbus-launch --sh-syntax)"
gnome-keyring-daemon --start --components=secrets
```

**Alternative:** Use the `PRISM_CLIENTS_*` environment variable fallback described
in §6 Troubleshooting (a) above — Tier-1/Tier-2 resolution takes precedence over the
keyring, so env vars will satisfy the read path even when the keyring is unavailable.

### (c) Port already in use

**Symptom:** `demo-run.sh` exits with "DTU server did not start within 30s" or the DTU log
shows `Address already in use (os error 98)`.

**Cause:** A previous DTU server is still running, or another process holds the port.

**Fix:**

1. Kill any stale DTU server:
   ```bash
   cat ~/.config/prism-demo/run/.prism-dtu-demo-server.pid | xargs kill 2>/dev/null || true
   ```

2. Find and kill other processes on the DTU ports. The demo uses **ephemeral ports**
   (bound at runtime), so read the actual port numbers from the urls.json sidecar
   that `demo-run.sh` writes:
   ```bash
   URLS=~/.config/prism-demo/run/.prism-dtu-demo-server.urls.json
   if [[ -f "$URLS" ]]; then
     grep -oP '(?<=:)\d+(?=")' "$URLS" \
       | xargs -I{} lsof -ti :{} 2>/dev/null \
       | sort -u \
       | xargs kill -9 2>/dev/null || true
   else
     echo "urls.json not found — no ephemeral ports to clear"
   fi
   ```
   If `demo-run.sh` was run with a custom `--config-dir`, replace
   `~/.config/prism-demo` with that directory.

3. Re-run `demo-run.sh`.

### (d) TOML spec not found

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

1. Reads `org_id` from `~/.config/prism-demo/prism.toml` (required for OrgId-keyed keyring delete)
2. Kills the DTU demo server (via PID file at `~/.config/prism-demo/run/.prism-dtu-demo-server.pid`)
3. Deletes the 5 demo OS keyring entries under the OrgId-UUID-keyed namespace
   `{org_id_uuid}/{sensor}/{name}` via `prism credential delete` (matching the write path used by
   `prism credential set` → `CredentialStoreOrgId::set_by_org`, ADR-034 §D3). Credential names:
   `crowdstrike/client_id`, `crowdstrike/client_secret`, `armis/bearer_token`,
   `claroty/bearer_token`, `cyberint/api_key`.
   **Keyring deletes run BEFORE config-dir removal because `prism credential delete` reads
   `prism.toml` for OrgId resolution — removing the config dir first would make `prism.toml`
   unavailable and cause all 5 deletes to fail (F-P10-HIGH-001 invariant).**
4. Removes `~/.config/prism-demo/` entirely (all config, specs, state, plugins, logs)

The binary artifacts (`target/release/prism`, `target/release/prism-dtu-demo-server`) are NOT
removed — run `cargo clean` separately if needed.

Override the config directory:

```bash
bash scripts/demo-teardown.sh --config-dir /custom/path/prism-demo
```
